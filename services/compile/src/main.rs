//! Compile-on-demand HTTP service for the G-Basic playground.
//!
//! Wraps `gbasic --target web` and returns `{wasm, js, errors?}` so a
//! browser editor can submit source and receive a runnable bundle.
//!
//! Sandboxing is intentionally light at this layer — defense-in-depth
//! comes from running this binary inside a constrained container
//! (see Dockerfile): tmpfs root, no network egress, low memory cap,
//! no privileges. Per-request controls here:
//!   - 1 MB source ceiling (axum body limit + explicit check)
//!   - 5 s wall-clock timeout
//!   - 5 MB output ceiling
//!   - per-request `TempDir`, dropped after response

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{
    Router,
    extract::{ConnectInfo, DefaultBodyLimit, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use tokio::process::Command;
use tokio::time::timeout;
use tower_http::cors::{Any, CorsLayer};

const MAX_SOURCE_BYTES: usize = 1024 * 1024;
const MAX_OUTPUT_BYTES: u64 = 5 * 1024 * 1024;
const COMPILE_TIMEOUT: Duration = Duration::from_secs(5);
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
const RATE_LIMIT_MAX: usize = 10;

#[derive(Clone, Default)]
struct AppState {
    rate_limits: Arc<Mutex<HashMap<String, VecDeque<std::time::Instant>>>>,
    telemetry: Arc<Mutex<TelemetryCounts>>,
}

#[derive(Default)]
struct TelemetryCounts {
    compile_succeeded: u64,
    compile_failed: u64,
    lesson_completed: u64,
}

fn gbasic_bin() -> String {
    std::env::var("GBASIC_BIN").unwrap_or_else(|_| "gbasic".to_string())
}

#[derive(Deserialize)]
struct CompileRequest {
    source: String,
}

#[derive(Deserialize)]
struct TelemetryRequest {
    event: String,
}

#[derive(Serialize, Default)]
struct CompileResponse {
    /// base64-encoded WASM bytes (None on compile error).
    #[serde(skip_serializing_if = "Option::is_none")]
    wasm: Option<String>,
    /// JS runtime glue text (None on compile error).
    #[serde(skip_serializing_if = "Option::is_none")]
    js: Option<String>,
    /// Human-readable error output from gbasic (None on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState::default();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/compile", post(compile))
        .route("/telemetry", post(telemetry))
        .layer(DefaultBodyLimit::max(MAX_SOURCE_BYTES + 4096))
        .layer(cors)
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    tracing::info!(%addr, "compile-service listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("serve");
}

async fn compile(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<CompileRequest>,
) -> (StatusCode, Json<CompileResponse>) {
    if !allow_request(&state, addr.ip().to_string()) {
        return error(
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded (10 compiles/min)",
        );
    }

    if req.source.len() > MAX_SOURCE_BYTES {
        return error(StatusCode::PAYLOAD_TOO_LARGE, "source exceeds 1MB limit");
    }

    let dir = match TempDir::new() {
        Ok(d) => d,
        Err(e) => return error(StatusCode::INTERNAL_SERVER_ERROR, &format!("tmpdir: {e}")),
    };

    let in_path = dir.path().join("in.gb");
    if let Err(e) = tokio::fs::write(&in_path, &req.source).await {
        return error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("write source: {e}"),
        );
    }

    let out_dir = dir.path().join("out");
    if let Err(e) = tokio::fs::create_dir(&out_dir).await {
        return error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("mkdir out: {e}"),
        );
    }

    let mut cmd = Command::new(gbasic_bin());
    cmd.arg(&in_path)
        .arg("--target")
        .arg("web")
        .arg("-o")
        .arg(&out_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let output = match timeout(COMPILE_TIMEOUT, cmd.output()).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => return error(StatusCode::INTERNAL_SERVER_ERROR, &format!("spawn: {e}")),
        Err(_) => return error(StatusCode::REQUEST_TIMEOUT, "compile timed out (5s)"),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let combined = if stderr.is_empty() { stdout } else { stderr };
        // Compile error is a normal user outcome, not an HTTP error.
        return (
            StatusCode::OK,
            Json(CompileResponse {
                errors: Some(combined),
                ..Default::default()
            }),
        );
    }

    let wasm_path = pick_wasm(&out_dir).await;
    let js_path = out_dir.join("runtime.js");

    let wasm_bytes = match read_with_cap(&wasm_path).await {
        Ok(b) => b,
        Err(e) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("read wasm: {e}"),
            );
        }
    };
    let js_text = match tokio::fs::read_to_string(&js_path).await {
        Ok(s) => s,
        Err(e) => return error(StatusCode::INTERNAL_SERVER_ERROR, &format!("read js: {e}")),
    };

    (
        StatusCode::OK,
        Json(CompileResponse {
            wasm: Some(base64::engine::general_purpose::STANDARD.encode(&wasm_bytes)),
            js: Some(js_text),
            errors: None,
        }),
    )
}

/// Prefer the asyncified wasm if `wasm-opt` produced one, else the raw output.
async fn pick_wasm(out_dir: &Path) -> std::path::PathBuf {
    let asyncified = out_dir.join("game_async.wasm");
    if tokio::fs::metadata(&asyncified).await.is_ok() {
        return asyncified;
    }
    out_dir.join("game.wasm")
}

async fn read_with_cap(path: &Path) -> Result<Vec<u8>, String> {
    let meta = tokio::fs::metadata(path).await.map_err(|e| e.to_string())?;
    if meta.len() > MAX_OUTPUT_BYTES {
        return Err(format!("output exceeds {MAX_OUTPUT_BYTES} bytes"));
    }
    tokio::fs::read(path).await.map_err(|e| e.to_string())
}

fn error(status: StatusCode, msg: &str) -> (StatusCode, Json<CompileResponse>) {
    (
        status,
        Json(CompileResponse {
            errors: Some(msg.to_string()),
            ..Default::default()
        }),
    )
}

async fn telemetry(
    State(state): State<AppState>,
    Json(req): Json<TelemetryRequest>,
) -> (StatusCode, &'static str) {
    if let Ok(mut counts) = state.telemetry.lock() {
        match req.event.as_str() {
            "compile_succeeded" => counts.compile_succeeded += 1,
            "compile_failed" => counts.compile_failed += 1,
            "lesson_completed" => counts.lesson_completed += 1,
            _ => {}
        }
    }
    (StatusCode::ACCEPTED, "ok")
}

fn allow_request(state: &AppState, key: String) -> bool {
    let now = std::time::Instant::now();
    let Ok(mut limits) = state.rate_limits.lock() else {
        return true;
    };
    let hits = limits.entry(key).or_default();
    while hits
        .front()
        .is_some_and(|t| now.duration_since(*t) > RATE_LIMIT_WINDOW)
    {
        hits.pop_front();
    }
    if hits.len() >= RATE_LIMIT_MAX {
        return false;
    }
    hits.push_back(now);
    true
}

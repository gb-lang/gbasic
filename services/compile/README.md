# gbasic-compile-service

HTTP service that wraps `gbasic --target web` so the playground can
submit source from the browser and receive a runnable WASM bundle.

## Endpoints

### `POST /compile`

Request:

```json
{ "source": "print(\"Hello!\")" }
```

Success (200):

```json
{
  "wasm": "<base64>",
  "js":   "<runtime.js text>"
}
```

Compile error (200 with `errors` set):

```json
{ "errors": "syntax error: ..." }
```

Other errors (4xx/5xx):

```json
{ "errors": "source exceeds 1MB limit" }
```

### `GET /healthz`

Returns `200 ok` for liveness probes.

### `POST /telemetry`

Anonymous event counter endpoint for playground metrics. The server accepts
events such as `compile_succeeded`, `compile_failed`, and `lesson_completed`.
It stores in-memory counters only; no PII, cookies, or user identifiers.

## Limits

- Source: ≤ 1 MB
- Output: ≤ 5 MB
- Compile wall time: 5 s
- Compile rate: 10 requests/minute/IP
- Per-request `TempDir` is dropped on response

## Local dev

Requires the `gbasic` binary on `PATH` (or pointed at via `GBASIC_BIN`).

```sh
# Build the gbasic compiler with LLVM (one-time, slow)
cargo build --release -p gbasic --features llvm
export GBASIC_BIN="$(pwd)/target/release/gbasic"

# Run the service
cargo run -p gbasic-compile-service
```

The service listens on `0.0.0.0:8080` (override with `PORT`).

Smoke test:

```sh
curl -s http://localhost:8080/healthz
curl -s http://localhost:8080/compile \
  -H 'content-type: application/json' \
  -d '{"source":"print(\"hi\")"}' | jq '.errors // "ok"'
```

## Container

```sh
docker build -t gbasic-compile-service -f services/compile/Dockerfile .
docker run -p 8080:8080 gbasic-compile-service
```

## Deploy targets

Hosting decision is deferred to Day 6 (Chibueze chooses between Fly.io
and Cloudflare Containers). The Dockerfile works on either.

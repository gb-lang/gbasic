//! End-to-end tests for web/WASM compilation target.
//! These tests verify that .gb programs compile to valid .wasm + HTML/JS.

use std::io::Write;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_dir() -> std::path::PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let tid = format!("{:?}", std::thread::current().id());
    let dir = std::env::temp_dir().join(format!("gbasic_web_{id}_{tid}"));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn compile_to_web(source: &str) -> Result<std::path::PathBuf, String> {
    let dir = unique_dir();
    let src_path = dir.join("test.gb");
    let out_dir = dir.join("web_out");

    let mut f = std::fs::File::create(&src_path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_gbasic"))
        .arg(src_path.to_str().unwrap())
        .arg("-o")
        .arg(out_dir.to_str().unwrap())
        .arg("--target")
        .arg("web")
        .output()
        .expect("failed to run gbasic");

    if !compile.status.success() {
        return Err(String::from_utf8_lossy(&compile.stderr).to_string());
    }

    Ok(out_dir)
}

fn assert_valid_wasm(dir: &std::path::Path) {
    let wasm = std::fs::read(dir.join("game.wasm")).expect("game.wasm should exist");
    assert!(wasm.len() >= 4, "wasm file too small");
    assert_eq!(&wasm[0..4], b"\0asm", "missing WASM magic bytes");
}

fn assert_web_files(dir: &std::path::Path) {
    assert!(dir.join("game.wasm").exists(), "game.wasm missing");
    assert!(dir.join("index.html").exists(), "index.html missing");
    assert!(dir.join("runtime.js").exists(), "runtime.js missing");
}

#[test]
fn test_web_hello() {
    let dir = compile_to_web(r#"print("Hello!")"#).unwrap();
    assert_web_files(&dir);
    assert_valid_wasm(&dir);
}

#[test]
fn test_web_arithmetic() {
    let dir = compile_to_web("print(1 + 2 * 3)").unwrap();
    assert_web_files(&dir);
    assert_valid_wasm(&dir);
}

#[test]
fn test_web_control_flow() {
    let dir = compile_to_web(
        r#"let x = 10
if x > 5 {
    print("big")
}"#,
    )
    .unwrap();
    assert_web_files(&dir);
    assert_valid_wasm(&dir);
}

#[test]
fn test_web_function() {
    let dir = compile_to_web(
        r#"fun add(a: Int, b: Int) -> Int { return a + b }
print(add(3, 4))"#,
    )
    .unwrap();
    assert_web_files(&dir);
    assert_valid_wasm(&dir);
}

#[test]
fn test_web_string_interp() {
    let dir = compile_to_web(
        r#"
let name = "world"
print("Hello, {name}!")
"#,
    )
    .unwrap();
    assert_web_files(&dir);
    assert_valid_wasm(&dir);
}

#[test]
fn test_web_all_examples() {
    let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples");

    let mut count = 0;
    for entry in std::fs::read_dir(&examples_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "gb") {
            let _source = std::fs::read_to_string(&path).unwrap();
            let dir = unique_dir().join("web_out");
            let compile = Command::new(env!("CARGO_BIN_EXE_gbasic"))
                .arg(path.to_str().unwrap())
                .arg("-o")
                .arg(dir.to_str().unwrap())
                .arg("--target")
                .arg("web")
                .output()
                .expect("failed to run gbasic");

            assert!(
                compile.status.success(),
                "Failed to compile {}: {}",
                path.display(),
                String::from_utf8_lossy(&compile.stderr)
            );

            assert_valid_wasm(&dir);
            count += 1;
        }
    }
    assert!(count >= 10, "Expected at least 10 examples, found {count}");
}

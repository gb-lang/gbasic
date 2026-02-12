//! End-to-end integration tests for the G-Basic compiler.
//! These tests require the gbasic binary and LLVM to be available.
//! Run with: cargo test --test e2e

use std::io::Write;
use std::process::Command;

fn compile_and_run(source: &str) -> Result<String, String> {
    let dir = std::env::temp_dir().join("gbasic_e2e");
    let _ = std::fs::create_dir_all(&dir);
    let src_path = dir.join("test.gb");
    let out_path = dir.join("test_bin");

    let mut f = std::fs::File::create(&src_path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    // Compile
    let compile = Command::new(env!("CARGO_BIN_EXE_gbasic"))
        .arg(src_path.to_str().unwrap())
        .arg("-o")
        .arg(out_path.to_str().unwrap())
        .arg("--skip-typecheck")
        .output()
        .expect("failed to run gbasic");

    if !compile.status.success() {
        return Err(String::from_utf8_lossy(&compile.stderr).to_string());
    }

    // Run
    let run = Command::new(out_path.to_str().unwrap())
        .output()
        .expect("failed to run compiled binary");

    Ok(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

#[test]
#[ignore]
fn test_hello_world() {
    let out = compile_and_run(r#"print("Hello!")"#).unwrap();
    assert_eq!(out, "Hello!");
}

#[test]
#[ignore]
fn test_arithmetic() {
    let out = compile_and_run("print(1 + 2 * 3)").unwrap();
    assert_eq!(out, "7");
}

#[test]
#[ignore]
fn test_for_range() {
    let out = compile_and_run("for i in 0..3 { print(i) }").unwrap();
    assert_eq!(out, "0\n1\n2");
}

#[test]
#[ignore]
fn test_function_call() {
    let out = compile_and_run(
        "fun double(x: Int) -> Int { return x * 2 }\nprint(double(5))",
    )
    .unwrap();
    assert_eq!(out, "10");
}

#[test]
#[ignore]
fn test_type_error_rejected() {
    let result = compile_and_run(r#"let x: Int = "bad""#);
    assert!(result.is_err(), "Should fail to compile type mismatch");
}

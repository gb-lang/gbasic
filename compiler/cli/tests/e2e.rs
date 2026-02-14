//! End-to-end integration tests for the G-Basic compiler.
//! These tests require the gbasic binary and LLVM to be available.
//! Run with: cargo test --test e2e

use std::io::Write;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_dir() -> std::path::PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let tid = format!("{:?}", std::thread::current().id());
    let dir = std::env::temp_dir().join(format!("gbasic_e2e_{id}_{tid}"));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn compile_and_run(source: &str) -> Result<String, String> {
    let dir = unique_dir();
    let src_path = dir.join("test.gb");
    let out_path = dir.join("test_bin");

    let mut f = std::fs::File::create(&src_path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    // Compile
    let compile = Command::new(env!("CARGO_BIN_EXE_gbasic"))
        .arg(src_path.to_str().unwrap())
        .arg("-o")
        .arg(out_path.to_str().unwrap())
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

fn compile_only(source: &str) -> Result<(), String> {
    let dir = unique_dir();
    let src_path = dir.join("test.gb");
    let out_path = dir.join("test_bin");

    let mut f = std::fs::File::create(&src_path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_gbasic"))
        .arg(src_path.to_str().unwrap())
        .arg("-o")
        .arg(out_path.to_str().unwrap())
        .output()
        .expect("failed to run gbasic");

    if compile.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&compile.stderr).to_string())
    }
}

#[test]
fn test_hello_world() {
    let out = compile_and_run(r#"print("Hello!")"#).unwrap();
    assert_eq!(out, "Hello!");
}

#[test]
fn test_arithmetic() {
    let out = compile_and_run("print(1 + 2 * 3)").unwrap();
    assert_eq!(out, "7");
}

#[test]
fn test_for_range() {
    let out = compile_and_run("for i in 0..3 { print(i) }").unwrap();
    assert_eq!(out, "0\n1\n2");
}

#[test]
fn test_function_call() {
    let out = compile_and_run(
        "fun double(x: Int) -> Int { return x * 2 }\nprint(double(5))",
    )
    .unwrap();
    assert_eq!(out, "10");
}

#[test]
fn test_type_error_rejected() {
    let result = compile_only(r#"let x: Int = "bad""#);
    assert!(result.is_err(), "Should fail to compile type mismatch");
}

#[test]
fn test_string_interpolation() {
    let out = compile_and_run(
        r#"let name = "World"
print("Hello, {name}!")"#,
    )
    .unwrap();
    assert_eq!(out, "Hello, World!");
}

#[test]
fn test_if_else() {
    let out = compile_and_run(
        r#"let x = 10
if x > 5 {
    print("big")
} else {
    print("small")
}"#,
    )
    .unwrap();
    assert_eq!(out, "big");
}

#[test]
fn test_while_loop() {
    let out = compile_and_run(
        r#"let x = 0
while x < 3 {
    print(x)
    x = x + 1
}"#,
    )
    .unwrap();
    assert_eq!(out, "0\n1\n2");
}

#[test]
fn test_match() {
    let out = compile_and_run(
        r#"let x = 2
match x {
    1 -> { print("one") }
    2 -> { print("two") }
    _ -> { print("other") }
}"#,
    )
    .unwrap();
    assert_eq!(out, "two");
}

#[test]
fn test_bool_ops() {
    let out = compile_and_run(
        r#"let a = true
let b = false
if a and not b {
    print("yes")
} else {
    print("no")
}"#,
    )
    .unwrap();
    assert_eq!(out, "yes");
}

#[test]
fn test_nested_functions() {
    let out = compile_and_run(
        r#"fun add(a: Int, b: Int) -> Int { return a + b }
fun mul(a: Int, b: Int) -> Int { return a * b }
print(add(mul(2, 3), 4))"#,
    )
    .unwrap();
    assert_eq!(out, "10");
}

#[test]
fn test_for_to_range() {
    let out = compile_and_run(
        r#"for i in 1 to 3 {
    print(i)
}"#,
    )
    .unwrap();
    assert_eq!(out, "1\n2\n3");
}

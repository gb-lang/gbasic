//! Error message golden file tests.
//! Verifies that specific bad programs produce expected error messages.
//! Run with: cargo test --test error_golden

use std::io::Write;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn compile_stderr(source: &str) -> String {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("gbasic_err_{id}"));
    let _ = std::fs::create_dir_all(&dir);
    let src_path = dir.join("test_err.gb");
    let out_path = dir.join("test_err_bin");

    let mut f = std::fs::File::create(&src_path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_gbasic"))
        .arg(src_path.to_str().unwrap())
        .arg("-o")
        .arg(out_path.to_str().unwrap())
        .output()
        .expect("failed to run gbasic");

    assert!(!compile.status.success(), "Expected compilation to fail");
    String::from_utf8_lossy(&compile.stderr).to_string()
}

#[test]
fn test_type_mismatch_error() {
    let stderr = compile_stderr(r#"let x: Int = "hello""#);
    assert!(
        stderr.contains("Type error") || stderr.contains("type mismatch"),
        "Expected type error, got: {stderr}"
    );
}

#[test]
fn test_undefined_variable_error() {
    let stderr = compile_stderr("print(undefined_var)");
    assert!(
        stderr.contains("Name error") || stderr.contains("undefined") || stderr.contains("not defined"),
        "Expected name error, got: {stderr}"
    );
}

#[test]
fn test_wrong_arg_count_error() {
    let stderr = compile_stderr(
        "fun greet(name: String) { print(name) }\ngreet()",
    );
    assert!(
        stderr.contains("argument") || stderr.contains("parameter"),
        "Expected argument count error, got: {stderr}"
    );
}

#[test]
fn test_syntax_error() {
    let stderr = compile_stderr("let = 5");
    assert!(
        stderr.contains("Syntax error") || stderr.contains("unexpected"),
        "Expected syntax error, got: {stderr}"
    );
}

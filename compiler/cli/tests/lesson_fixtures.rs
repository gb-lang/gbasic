//! Smoke checks for playground lesson starter and solution programs.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn lesson_fixtures_typecheck() {
    let lessons_dir = repo_root().join("playground").join("lessons");
    let mut checked = 0;

    for entry in std::fs::read_dir(&lessons_dir).expect("lessons dir exists") {
        let path = entry.expect("lesson entry").path();
        if path.extension().is_some_and(|ext| ext == "gb") {
            let output = Command::new(env!("CARGO_BIN_EXE_gbasic"))
                .arg(&path)
                .arg("--check")
                .output()
                .expect("failed to run gbasic");

            assert!(
                output.status.success(),
                "{} failed --check:\n{}",
                path.display(),
                String::from_utf8_lossy(&output.stderr)
            );
            checked += 1;
        }
    }

    assert_eq!(checked, 12, "expected starter+solution for 6 lessons");
}

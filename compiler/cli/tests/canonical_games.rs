//! Smoke checks for the flagship game examples.
//!
//! These are intentionally conservative in CI: `--check` verifies the canonical
//! examples stay parseable and type-checkable without trying to open an SDL
//! window on a headless runner. Full frame-run verification remains a launch
//! checklist item for a machine with LLVM 18 + SDL2 display support.

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
fn canonical_games_typecheck() {
    for game in ["pong.gb", "flappy.gb", "angrybirds.gb"] {
        let path = repo_root().join("examples").join(game);
        let output = Command::new(env!("CARGO_BIN_EXE_gbasic"))
            .arg(&path)
            .arg("--check")
            .output()
            .expect("failed to run gbasic");

        assert!(
            output.status.success(),
            "{} failed --check:\n{}",
            game,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

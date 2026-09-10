#![forbid(unsafe_code)]

//! The outer test compiles against the existing public API and launches the
//! external consumer. Before the auto-repeat content return type is widened,
//! its failure is nested consumer compilation RED, not an executed behavioral
//! failure inside that consumer. The separate grammar target executes runtime
//! assertions against the existing API.
//! The nested offline Cargo process inherits the outer bounded process group
//! and exclusively owns its temporary target directory.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn owned_target_directory() -> std::io::Result<PathBuf> {
    for _ in 0..16 {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "surgeist-css-grid3-auto-repeat-consumer-{}-{sequence}",
            std::process::id()
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not reserve an owned consumer target directory in 16 attempts",
    ))
}

#[test]
fn public_auto_repeat_consumer_preserves_general_bodies_and_fixed_surroundings() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let target = owned_target_directory().expect("create an exclusively owned target directory");
    let output = Command::new(env!("CARGO"))
        .args(["run", "--offline", "--locked", "-j", "1", "--manifest-path"])
        .arg(&manifest)
        .args([
            "-p",
            "surgeist-css",
            "--example",
            "grid3_auto_repeat_consumer",
            "--target-dir",
        ])
        .arg(&target)
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TERM_COLOR", "never")
        .stdin(Stdio::null())
        .output();
    let cleanup = std::fs::remove_dir_all(&target);
    let output = output.unwrap_or_else(|error| {
        panic!(
            "consumer spawn failed: {error}\nowned target cleanup at {}: {cleanup:?}",
            target.display()
        )
    });
    assert!(
        output.status.success() && cleanup.is_ok(),
        "public auto-repeat consumer status: {}\nowned target cleanup at {}: {cleanup:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        target.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "general automatic body and fixed surroundings: ok\n",
            "symbolic automatic calculation and line names: ok\n",
            "checked component grammar and preserved origins: ok\n",
            "explicit axes and implicit track roles: ok\n",
        )
    );
}

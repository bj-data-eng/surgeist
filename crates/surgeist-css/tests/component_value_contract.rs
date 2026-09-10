#![forbid(unsafe_code)]

//! Positive public-consumer compilation and runtime contract.
//!
//! This harness itself compiles before the new API exists. Its RED is the
//! executed assertion that the real external example consumer must compile;
//! after implementation the same consumer must also satisfy its runtime cases.
//! Run the selected test through the repository's bounded process wrapper. The
//! nested Cargo command inherits that process group and has its own target
//! directory, while using the current product workspace and committed lockfile.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn owned_target_directory() -> std::io::Result<PathBuf> {
    for _ in 0..16 {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "surgeist-css-component-consumer-{}-{sequence}",
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
fn public_component_value_consumer_compiles_and_preserves_tokens_and_origins() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let target = owned_target_directory().expect("create an exclusively owned target directory");
    let output = Command::new(env!("CARGO"))
        .args(["run", "--offline", "--locked", "-j", "1", "--manifest-path"])
        .arg(&manifest)
        .args([
            "-p",
            "surgeist-css",
            "--example",
            "component_value_contract",
            "--target-dir",
        ])
        .arg(&target)
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TERM_COLOR", "never")
        .stdin(Stdio::null())
        .output();
    // Command::output waits for the direct child before this owned-directory
    // removal. An outer timeout terminates the complete inherited process group.
    let cleanup = std::fs::remove_dir_all(&target);
    let output = output.unwrap_or_else(|error| {
        panic!(
            "consumer spawn failed: {error}\nowned target cleanup at {}: {cleanup:?}",
            target.display()
        )
    });
    assert!(
        output.status.success() && cleanup.is_ok(),
        "public component-value consumer status: {}\nowned target cleanup at {}: {cleanup:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        target.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "token boundaries: ok\n",
            "checked construction: ok\n",
            "whitespace and closures: ok\n",
            "origins and limits: ok\n",
        )
    );
}

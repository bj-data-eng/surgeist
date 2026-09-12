#![forbid(unsafe_code)]

//! This outer test executes against the existing public API and requires the
//! external consumer to compile and complete the independent behavioral cases.
//! Before the media-grouping semantic API exists, its failure is nested consumer
//! compilation RED; the consumer's behavioral assertions have not run yet.
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
            "surgeist-css-media-grouping-semantic-consumer-{}-{sequence}",
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
fn public_media_grouping_semantic_consumer_preserves_operators_groups_and_feature_grammar() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let target = owned_target_directory().expect("create an exclusively owned target directory");
    let output = Command::new(env!("CARGO"))
        .args(["run", "--offline", "--locked", "-j", "1", "--manifest-path"])
        .arg(&manifest)
        .args([
            "-p",
            "surgeist-css",
            "--example",
            "media_grouping_semantic_consumer",
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
        "public media-grouping semantic consumer status: {}\nowned target cleanup at {}: {cleanup:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        target.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        "media grouping semantic contract: ok\n"
    );
}

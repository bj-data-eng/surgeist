#![forbid(unsafe_code)]

//! This outer test executes against the existing public API and requires the
//! external consumer to compile and complete the independent behavioral cases.
//! Before the property metadata API exists, its failure is nested consumer
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
            "surgeist-css-property-metadata-consumer-{}-{sequence}",
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
fn public_property_metadata_consumer_preserves_initials_and_grammar_identity() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let target = owned_target_directory().expect("create an exclusively owned target directory");
    let output = Command::new(env!("CARGO"))
        .args(["run", "--offline", "--locked", "-j", "1", "--manifest-path"])
        .arg(&manifest)
        .args([
            "-p",
            "surgeist-css",
            "--example",
            "property_metadata_consumer",
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
        "public property metadata consumer status: {}\nowned target cleanup at {}: {cleanup:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        target.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "metadata recognition, inheritance and intrinsic initial values: ok\n",
            "terminal membership, omissions, reset-only values and all exclusions: ok\n",
            "legacy grammar identity and atomic reusable reentry: ok\n",
            "parsed and constructed provenance with ordered color normalization: ok\n",
        )
    );
}

// The integration target compiles the production library without cfg(test).
// Its child selects the real expansion-owned unit test under cfg(test), so a
// missing owner transition produces nested unit-compilation RED, never a claim
// that the not-yet-compiling owner assertions ran.
#[test]
fn expansion_owner_preserves_user_agent_initial_requirement() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let target = owned_target_directory().expect("owned target");
    let name = "expansion::metadata_initial_tests::omitted_initial_preserves_user_agent_requirement_and_fixed_value";
    let output = Command::new(env!("CARGO"))
        .args([
            "test",
            "--offline",
            "--locked",
            "-j",
            "1",
            "--manifest-path",
        ])
        .arg(manifest)
        .args(["-p", "surgeist-css", "--lib", "--target-dir"])
        .arg(&target)
        .args([name, "--", "--exact", "--nocapture", "--test-threads=1"])
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TERM_COLOR", "never")
        .stdin(Stdio::null())
        .output();
    let cleanup = std::fs::remove_dir_all(&target);
    let output = output.unwrap_or_else(|error| {
        panic!(
            "owner test spawn failed: {error}; target {} cleanup: {cleanup:?}",
            target.display()
        )
    });
    assert!(
        output.status.success() && cleanup.is_ok(),
        "owner unit test: {}; cleanup: {cleanup:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(&format!("test {name} ... ok")),
        "the exact selected owner test must execute: {stdout}"
    );
}

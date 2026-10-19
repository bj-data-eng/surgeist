//! Private lifecycle for real external CSS consumers.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn owned_target_directory(label: &str) -> std::io::Result<PathBuf> {
    reserve_target_directory(&std::env::temp_dir(), label, &FIXTURE_SEQUENCE)
}

fn reserve_target_directory(
    base: &Path,
    label: &str,
    sequence: &AtomicU64,
) -> std::io::Result<PathBuf> {
    for _ in 0..16 {
        let sequence = sequence.fetch_add(1, Ordering::Relaxed);
        let path = base.join(format!(
            "surgeist-css-{label}-{}-{sequence}",
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

fn run_owned_cargo(
    label: &str,
    subcommand: &str,
    configure: impl FnOnce(&mut Command, &Path),
) -> Output {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let target =
        owned_target_directory(label).expect("create an exclusively owned target directory");
    let mut command = Command::new(env!("CARGO"));
    command
        .args([
            subcommand,
            "--offline",
            "--locked",
            "-j",
            "1",
            "--manifest-path",
        ])
        .arg(manifest)
        .args(["-p", "surgeist-css"])
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TERM_COLOR", "never")
        .stdin(Stdio::null());
    configure(&mut command, &target);

    run_owned_command(command, &target, label)
}

fn run_owned_command(mut command: Command, target: &Path, label: &str) -> Output {
    // Command::output waits for the direct child before removal. The outer
    // bounded process wrapper owns the inherited process group and its timeout.
    let output = command.output();
    let cleanup = std::fs::remove_dir_all(target);
    let output = output.unwrap_or_else(|error| {
        panic!(
            "{label} spawn failed: {error}\nowned target cleanup at {}: {cleanup:?}",
            target.display()
        )
    });
    assert!(
        output.status.success() && cleanup.is_ok(),
        "{label} status: {}\nowned target cleanup at {}: {cleanup:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        target.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

pub(crate) fn run_example(example: &str) -> Output {
    run_owned_cargo(example, "run", |command, target| {
        command
            .args(["--example", example, "--target-dir"])
            .arg(target);
    })
}

// Only the metadata integration target selects a nested library test.
#[allow(dead_code)]
pub(crate) fn run_selected_library_test(name: &str) -> Output {
    run_owned_cargo("property-metadata-owner", "test", |command, target| {
        command.args(["--lib", "--target-dir"]).arg(target).args([
            name,
            "--",
            "--exact",
            "--nocapture",
        ]);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    fn test_directory() -> PathBuf {
        static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "surgeist-css-consumer-helper-test-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }

    fn panic_message(result: Result<Output, Box<dyn std::any::Any + Send>>) -> String {
        let error = result.expect_err("consumer failure must be reported");
        if let Some(message) = error.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = error.downcast_ref::<&str>() {
            (*message).to_owned()
        } else {
            panic!("unexpected panic payload")
        }
    }

    #[test]
    fn reservation_skips_an_existing_candidate_and_owns_the_next_one() {
        let root = test_directory();
        let first = root.join(format!("surgeist-css-collision-{}-0", std::process::id()));
        std::fs::create_dir(&first).unwrap();
        let sequence = AtomicU64::new(0);
        let selected = reserve_target_directory(&root, "collision", &sequence).unwrap();
        assert_eq!(
            selected,
            root.join(format!("surgeist-css-collision-{}-1", std::process::id()))
        );
        assert!(selected.is_dir());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spawn_failure_still_removes_the_owned_target() {
        let root = test_directory();
        let target = root.join("target");
        std::fs::create_dir(&target).unwrap();
        let command = Command::new(root.join("missing-program"));
        let message = panic_message(catch_unwind(AssertUnwindSafe(|| {
            run_owned_command(command, &target, "spawn probe")
        })));
        assert!(message.contains("spawn failed"));
        assert!(message.contains("owned target cleanup"));
        assert!(!target.exists());
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    fn child_failure_reports_output_and_removes_the_owned_target() {
        let root = test_directory();
        let target = root.join("target");
        std::fs::create_dir(&target).unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.arg("--invalid-test-harness-option");
        let message = panic_message(catch_unwind(AssertUnwindSafe(|| {
            run_owned_command(command, &target, "child probe")
        })));
        assert!(message.contains("child probe status:"));
        assert!(message.contains("stderr:"));
        assert!(!target.exists());
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn cleanup_failure_is_reported_even_after_a_successful_child() {
        let root = test_directory();
        let target = root.join("unreserved-target");
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.arg("--help");
        let message = panic_message(catch_unwind(AssertUnwindSafe(|| {
            run_owned_command(command, &target, "cleanup probe")
        })));
        let success = std::process::ExitStatus::default();
        assert!(message.contains(&format!("cleanup probe status: {success}")));
        assert!(message.contains("owned target cleanup"));
        assert!(message.contains("Err("));
        std::fs::remove_dir(root).unwrap();
    }
}

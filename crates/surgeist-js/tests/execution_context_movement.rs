#![allow(unused_crate_dependencies)]

use std::process::Command;

use surgeist_js::{Context, execution::ExecutionOptions};

const CHILD_ENV: &str = "SURGEIST_EXECUTION_CONTEXT_MOVEMENT_CHILD";
const TEST_NAME: &str = "moving_context_from_safe_execution_callback_does_not_abort_the_host";

fn bounded_diagnostic(bytes: &[u8]) -> String {
    const LIMIT: usize = 4096;
    let prefix = String::from_utf8_lossy(&bytes[..bytes.len().min(LIMIT)]);
    format!(
        "{prefix}\n({} bytes total; at most {LIMIT} shown)",
        bytes.len()
    )
}

#[test]
fn moving_context_from_safe_execution_callback_does_not_abort_the_host() {
    if std::env::var_os(CHILD_ENV).as_deref() == Some(std::ffi::OsStr::new("1")) {
        let mut context = Context::default();
        let outcome = context.with_execution(ExecutionOptions::new(0), |context| {
            Ok(std::mem::take(context))
        });
        // No guest work or panicking user destructor is involved. Either a
        // returned outcome or a typed refusal must remain normally disposable.
        drop(outcome);
        return;
    }

    // Isolate process termination so an abort becomes an ordinary assertion
    // failure. The child inherits the supervising invocation's process group.
    let output = Command::new(std::env::current_exe().expect("current test executable"))
        .args(["--exact", TEST_NAME, "--nocapture"])
        .env(CHILD_ENV, "1")
        .env("RUST_BACKTRACE", "0")
        .output()
        .expect("run the isolated Context movement test");
    assert!(
        output.status.success(),
        "safe Context movement terminated the host: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        bounded_diagnostic(&output.stdout),
        bounded_diagnostic(&output.stderr),
    );
}

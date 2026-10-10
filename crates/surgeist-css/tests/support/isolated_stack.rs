#![forbid(unsafe_code)]
//! Reexecute one integration test and join its explicitly sized worker before
//! reporting completion. Workloads and their expectations remain in callers.
//! Run callers through the repository bounded process wrapper so an abort or
//! stalled child remains inside its inherited process group and deadline.

use std::process::{Command, Output, Stdio};

/// The parent receives the reaped child's output; the selected child returns
/// only after the worker (including its captured resources) has been destroyed.
#[track_caller]
pub(crate) fn run(
    test: &str,
    child_environment: &str,
    completed: &str,
    thread_name: Option<&str>,
    operation: fn(),
) -> Option<Output> {
    if std::env::var(child_environment).as_deref() == Ok(test) {
        let mut builder = std::thread::Builder::new().stack_size(2 * 1024 * 1024);
        if let Some(name) = thread_name {
            builder = builder.name(name.into());
        }
        let worker = builder
            .spawn(operation)
            .expect("start explicitly sized stack worker");
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("{completed}");
        return None;
    }
    let output = child_output(test, child_environment);
    assert!(
        output.status.success(),
        "isolated stack child failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(completed),
        "the selected child must execute its assertions and join its worker\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    Some(output)
}

#[track_caller]
fn child_output(test: &str, child_environment: &str) -> Output {
    Command::new(std::env::current_exe().expect("locate this integration test"))
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env(child_environment, test)
        .env_remove("RUST_MIN_STACK")
        .stdin(Stdio::null())
        .output()
        .expect("run and reap isolated stack child")
}

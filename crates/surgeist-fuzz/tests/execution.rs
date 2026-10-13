#![forbid(unsafe_code)]
#![cfg(feature = "execution")]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use surgeist_fuzz::execution::*;

fn limits(timeout: Duration) -> WorkerLimits {
    WorkerLimits::new(
        2 * 1024 * 1024,
        64 * 1024,
        64 * 1024,
        timeout,
        Duration::from_millis(20),
        Duration::from_secs(2),
        MemoryPolicy::Unbounded,
    )
    .unwrap()
}
fn spec(args: &[&str], input: Vec<u8>, limits: WorkerLimits) -> WorkerSpec {
    WorkerSpec::new(
        PathBuf::from(env!("CARGO_BIN_EXE_surgeist-fuzz-process-worker")),
        std::env::current_dir().unwrap(),
        args.iter().map(OsString::from).collect(),
        BTreeMap::new(),
        input,
        limits,
    )
    .unwrap()
}
fn run(args: &[&str], input: Vec<u8>, limit: WorkerLimits) -> ProcessOutcome {
    execute(&spec(args, input, limit), &AtomicBool::new(false)).unwrap()
}
fn gone(outcome: &ProcessOutcome) {
    assert_eq!(outcome.cleanup, CleanupStatus::Complete, "{outcome:?}");
    #[cfg(unix)]
    {
        let group =
            rustix::process::Pid::from_raw(i32::try_from(outcome.process_group).unwrap()).unwrap();
        assert_eq!(
            rustix::process::test_kill_process_group(group),
            Err(rustix::io::Errno::SRCH)
        );
    }
}

#[test]
fn input_and_deadlines_are_admitted_before_spawn() {
    assert!(matches!(
        WorkerLimits::new(
            1,
            1,
            1,
            Duration::ZERO,
            Duration::ZERO,
            Duration::from_secs(1),
            MemoryPolicy::Unbounded
        ),
        Err(ExecutionError::InvalidSpec(_))
    ));
    assert!(matches!(
        WorkerSpec::new(
            PathBuf::from("relative"),
            std::env::current_dir().unwrap(),
            vec![],
            BTreeMap::new(),
            vec![],
            limits(Duration::from_secs(1))
        ),
        Err(ExecutionError::InvalidSpec(_))
    ));
    let small = WorkerLimits::new(
        1,
        1,
        1,
        Duration::from_secs(1),
        Duration::ZERO,
        Duration::from_secs(1),
        MemoryPolicy::Unbounded,
    )
    .unwrap();
    assert!(matches!(
        WorkerSpec::new(
            PathBuf::from(env!("CARGO_BIN_EXE_surgeist-fuzz-process-worker")),
            std::env::current_dir().unwrap(),
            vec![],
            BTreeMap::new(),
            vec![1, 2],
            small
        ),
        Err(ExecutionError::InputLimit {
            actual: 2,
            limit: 1
        })
    ));
}

#[test]
fn unsupported_hosts_fail_before_starting_worker() {
    if execution_capabilities().supported_host {
        return;
    }
    assert!(matches!(
        execute(
            &spec(&["echo"], vec![], limits(Duration::from_secs(1))),
            &AtomicBool::new(false)
        ),
        Err(ExecutionError::UnsupportedHost)
    ));
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod supported {
    use super::*;
    #[test]
    fn exact_binary_input_and_both_streams_survive_pipe_capacity() {
        let input: Vec<_> = (0..300_000).map(|i| (i % 256) as u8).collect();
        let limit = WorkerLimits::new(
            input.len(),
            input.len(),
            1024,
            Duration::from_secs(4),
            Duration::from_millis(20),
            Duration::from_secs(2),
            MemoryPolicy::Unbounded,
        )
        .unwrap();
        let outcome = run(&["echo"], input.clone(), limit);
        assert_eq!(outcome.stop, ProcessStop::Exited);
        assert_eq!(outcome.exit, ProcessExit::Success);
        assert_eq!(outcome.stdout.bytes, input);
        assert_eq!(outcome.stderr.bytes, b"controlled stderr");
        assert_eq!(outcome.input_bytes_written, 300_000);
        assert!(outcome.stdout.eof && outcome.stderr.eof);
        gone(&outcome);
    }
    #[test]
    fn only_explicit_environment_reaches_worker() {
        let worker = WorkerSpec::new(
            PathBuf::from(env!("CARGO_BIN_EXE_surgeist-fuzz-process-worker")),
            std::env::current_dir().unwrap(),
            vec!["environment".into()],
            BTreeMap::from([("SURGEIST_WORKER_VALUE".into(), "exact".into())]),
            vec![],
            limits(Duration::from_secs(2)),
        )
        .unwrap();
        let outcome = execute(&worker, &AtomicBool::new(false)).unwrap();
        assert_eq!(outcome.stdout.bytes, b"exact:false");
        gone(&outcome);
    }
    #[test]
    fn exits_distinguish_nonzero_panic_and_signal() {
        let nonzero = run(&["nonzero"], vec![], limits(Duration::from_secs(2)));
        assert_eq!(nonzero.exit, ProcessExit::Nonzero(17));
        gone(&nonzero);
        let panic = run(&["panic"], vec![], limits(Duration::from_secs(2)));
        assert_eq!(panic.exit, ProcessExit::Nonzero(101));
        assert!(String::from_utf8_lossy(&panic.stderr.bytes).contains("controlled worker panic"));
        gone(&panic);
        let abort = run(&["abort"], vec![], limits(Duration::from_secs(2)));
        assert!(matches!(abort.exit, ProcessExit::Signal(_)));
        gone(&abort);
    }
    #[test]
    fn deadline_interrupts_worker_that_never_reads_large_stdin() {
        let outcome = run(
            &["hang"],
            vec![7; 1024 * 1024],
            limits(Duration::from_millis(100)),
        );
        assert_eq!(outcome.stop, ProcessStop::Deadline);
        assert!(outcome.input_bytes_written < 1024 * 1024);
        assert!(outcome.duration < Duration::from_secs(3));
        gone(&outcome);
    }
    #[test]
    fn term_ignoring_worker_requires_kill_after_grace_and_is_reaped() {
        let outcome = run(&["ignore-term"], vec![], limits(Duration::from_millis(500)));
        assert_eq!(outcome.stop, ProcessStop::Deadline);
        assert_eq!(outcome.stdout.bytes, b"ready");
        assert_eq!(outcome.exit, ProcessExit::Signal(9));
        gone(&outcome);
    }
    #[test]
    fn unavailable_executable_is_spawn_failure_without_a_worker_group() {
        let worker = WorkerSpec::new(
            std::env::temp_dir().join("surgeist-nonexistent-worker-executable"),
            std::env::current_dir().unwrap(),
            vec![],
            BTreeMap::new(),
            vec![],
            limits(Duration::from_secs(2)),
        )
        .unwrap();
        assert!(matches!(
            execute(&worker, &AtomicBool::new(false)),
            Err(ExecutionError::Spawn(_))
        ));
    }
    #[test]
    fn output_flood_is_bounded_and_stops_worker_on_either_stream() {
        for (stream, args) in [
            (OutputStream::Stdout, vec!["flood"]),
            (OutputStream::Stderr, vec!["flood", "stderr"]),
        ] {
            let outcome = run(&args, vec![], limits(Duration::from_secs(2)));
            assert_eq!(outcome.stop, ProcessStop::OutputLimit(stream));
            let capture = if stream == OutputStream::Stdout {
                &outcome.stdout
            } else {
                &outcome.stderr
            };
            assert_eq!(capture.bytes.len(), 64 * 1024);
            assert!(capture.truncated);
            gone(&outcome);
        }
    }
    #[test]
    fn cancellation_after_worker_readiness_cleans_descendants() {
        let marker =
            std::env::temp_dir().join(format!("surgeist-executor-ready-{}", std::process::id()));
        let _ = std::fs::remove_file(&marker);
        let cancellation = AtomicBool::new(false);
        std::thread::scope(|scope| {
            let cancel = &cancellation;
            let ready = &marker;
            let thread = scope.spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(2);
                while !ready.exists() {
                    assert!(Instant::now() < deadline, "worker readiness never arrived");
                    std::thread::sleep(Duration::from_millis(2));
                }
                cancel.store(true, Ordering::Release);
            });
            let marker_arg = marker.to_str().unwrap();
            let outcome = run_with_cancel(&["descendant", marker_arg], &cancellation);
            thread.join().unwrap();
            assert_eq!(outcome.stop, ProcessStop::Cancelled);
            gone(&outcome);
        });
        std::fs::remove_file(marker).unwrap();
    }
    fn run_with_cancel(args: &[&str], cancel: &AtomicBool) -> ProcessOutcome {
        execute(&spec(args, vec![], limits(Duration::from_secs(3))), cancel).unwrap()
    }
    #[test]
    fn successful_parent_exit_still_terminates_owned_descendant_group() {
        let outcome = run(&["descendant-exit"], vec![], limits(Duration::from_secs(2)));
        assert_eq!(outcome.stop, ProcessStop::Exited);
        assert_eq!(outcome.exit, ProcessExit::Success);
        let descendant: i32 = String::from_utf8(outcome.stdout.bytes.clone())
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let pid = rustix::process::Pid::from_raw(descendant).unwrap();
        assert_eq!(
            rustix::process::test_kill_process(pid),
            Err(rustix::io::Errno::SRCH)
        );
        gone(&outcome);
    }
    #[test]
    fn cancelling_one_group_preserves_an_unrelated_owned_group() {
        use std::os::unix::process::CommandExt;
        let mut unrelated =
            std::process::Command::new(env!("CARGO_BIN_EXE_surgeist-fuzz-process-worker"))
                .arg("hang")
                .process_group(0)
                .spawn()
                .unwrap();
        let outcome = run(&["descendant"], vec![], limits(Duration::from_millis(100)));
        let unrelated_alive = unrelated.try_wait().unwrap().is_none();
        unrelated.kill().unwrap();
        unrelated.wait().unwrap();
        assert!(unrelated_alive);
        gone(&outcome);
    }
    #[test]
    fn mandatory_memory_policy_and_pre_spawn_cancellation_are_explicit() {
        let hard = WorkerLimits::new(
            1,
            1,
            1,
            Duration::from_secs(1),
            Duration::ZERO,
            Duration::from_secs(1),
            MemoryPolicy::RequireHardLimit(1024),
        )
        .unwrap();
        assert!(matches!(
            execute(&spec(&["echo"], vec![], hard), &AtomicBool::new(false)),
            Err(ExecutionError::UnsupportedMemoryPolicy)
        ));
        assert!(matches!(
            execute(
                &spec(&["echo"], vec![], limits(Duration::from_secs(1))),
                &AtomicBool::new(true)
            ),
            Err(ExecutionError::CancelledBeforeSpawn)
        ));
        assert!(!execution_capabilities().sampled_rss);
    }
}

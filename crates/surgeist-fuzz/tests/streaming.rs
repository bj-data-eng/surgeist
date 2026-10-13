#![forbid(unsafe_code)]
#![cfg(all(feature = "execution", target_os = "macos", target_arch = "aarch64"))]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use surgeist_fuzz::execution::*;
use surgeist_fuzz::worker::{ResponseDecoder, WorkerResponse, write_frame};

fn spec(args: &[&str], stdout: usize, stderr: usize) -> WorkerSpec {
    WorkerSpec::new(
        PathBuf::from(env!("CARGO_BIN_EXE_surgeist-fuzz-process-worker")),
        std::env::current_dir().unwrap(),
        args.iter().map(Into::into).collect(),
        BTreeMap::new(),
        Vec::new(),
        WorkerLimits::new(
            0,
            stdout,
            stderr,
            Duration::from_secs(20),
            Duration::from_millis(20),
            Duration::from_secs(2),
            MemoryPolicy::Unbounded,
        )
        .unwrap(),
    )
    .unwrap()
}

fn gone(outcome: &ProcessOutcome) {
    assert_eq!(outcome.cleanup, CleanupStatus::Complete, "{outcome:?}");
    let group =
        rustix::process::Pid::from_raw(i32::try_from(outcome.process_group).unwrap()).unwrap();
    assert_eq!(
        rustix::process::test_kill_process_group(group),
        Err(rustix::io::Errno::SRCH)
    );
}

#[test]
fn output_beyond_sixteen_mib_streams_exactly_with_bounded_stderr_tail() {
    let stdout_bytes = 17 * 1024 * 1024 + 123;
    let stderr_bytes = 80 * 1024 + 11;
    let worker = spec(
        &[
            "stream",
            &stdout_bytes.to_string(),
            &stderr_bytes.to_string(),
        ],
        64,
        32,
    );
    let mut stdout_total = 0_u64;
    let mut stderr_total = 0_u64;
    let outcome = execute_streaming(&worker, &AtomicBool::new(false), &mut |stream, bytes| {
        assert!(!bytes.is_empty() && bytes.len() <= 8192);
        match stream {
            OutputStream::Stdout => {
                assert!(bytes.iter().all(|byte| *byte == b'x'));
                stdout_total += bytes.len() as u64;
            }
            OutputStream::Stderr => stderr_total += bytes.len() as u64,
        }
        Ok::<_, ()>(())
    })
    .unwrap();
    assert!(outcome.consumer_failure.is_none());
    assert_eq!(outcome.process.stop, ProcessStop::Exited);
    assert_eq!(outcome.process.exit, ProcessExit::Success);
    assert_eq!(stdout_total, stdout_bytes as u64);
    assert_eq!(outcome.stdout_bytes, stdout_total);
    assert_eq!(
        stderr_total,
        stderr_bytes as u64 + b"last diagnostics".len() as u64
    );
    assert_eq!(outcome.stderr_bytes, stderr_total);
    assert!(outcome.process.stdout.bytes.is_empty());
    assert!(outcome.process.stdout.truncated && outcome.process.stdout.eof);
    let mut tail = vec![b'z'; 32 - b"last diagnostics".len()];
    tail.extend_from_slice(b"last diagnostics");
    assert_eq!(outcome.process.stderr.bytes, tail);
    assert!(outcome.process.stderr.truncated && outcome.process.stderr.eof);
    gone(&outcome.process);
}

#[test]
fn thousands_of_framed_responses_above_sixteen_mib_are_consumed_incrementally() {
    let frames = 4500;
    let response = WorkerResponse::Generated {
        canonical: vec![3; 2048],
    };
    let mut frame = Vec::new();
    write_frame(&mut frame, &response, 8192).unwrap();
    let expected_bytes = frame.len() as u64 * frames;
    assert!(expected_bytes > 16 * 1024 * 1024);
    let worker = spec(&["framed-stream", &frames.to_string()], 0, 64);
    let mut decoder = ResponseDecoder::new(8192).unwrap();
    let mut accepted = 0;
    let outcome = execute_streaming(&worker, &AtomicBool::new(false), &mut |stream, bytes| {
        if stream == OutputStream::Stdout {
            decoder.push(bytes, &mut |actual| {
                assert_eq!(actual, response);
                accepted += 1;
                Ok(())
            })?;
        }
        Ok::<_, surgeist_fuzz::FuzzError>(())
    })
    .unwrap();
    decoder.finish().unwrap();
    assert!(outcome.consumer_failure.is_none());
    assert_eq!(accepted, frames);
    assert_eq!(outcome.stdout_bytes, expected_bytes);
    assert_eq!(outcome.process.stop, ProcessStop::Exited);
    assert!(outcome.process.stdout.bytes.is_empty());
    gone(&outcome.process);
}

#[test]
fn consumer_error_is_returned_after_descendant_cleanup_and_disables_callbacks() {
    let worker = spec(&["descendant"], 0, 64);
    let mut calls = 0;
    let outcome = execute_streaming(&worker, &AtomicBool::new(false), &mut |_, _| {
        calls += 1;
        Err("evidence store rejected capture")
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert!(matches!(
        outcome.consumer_failure,
        Some(ConsumerFailure::Error("evidence store rejected capture"))
    ));
    assert!(matches!(
        outcome.process.stop,
        ProcessStop::InfrastructureFailure {
            operation: "consume output",
            ..
        }
    ));
    gone(&outcome.process);
}

#[test]
fn consumer_panic_is_reported_after_descendant_cleanup() {
    let worker = spec(&["descendant"], 0, 64);
    let mut calls = 0;
    let outcome = execute_streaming(&worker, &AtomicBool::new(false), &mut |_,
                                                                            _|
     -> Result<
        (),
        (),
    > {
        calls += 1;
        panic!("controlled consumer panic")
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert!(matches!(
        outcome.consumer_failure,
        Some(ConsumerFailure::Panicked)
    ));
    assert!(matches!(
        outcome.process.stop,
        ProcessStop::InfrastructureFailure {
            operation: "consume output",
            ..
        }
    ));
    gone(&outcome.process);
}

#[test]
fn cancellation_from_observed_readiness_preserves_stream_and_cleans_descendants() {
    let worker = spec(&["descendant"], 0, 0);
    let cancelled = AtomicBool::new(false);
    let mut received = 0_u64;
    let outcome = execute_streaming(&worker, &cancelled, &mut |stream, bytes| {
        if stream == OutputStream::Stdout {
            received += bytes.len() as u64;
            cancelled.store(true, Ordering::Release);
        }
        Ok::<_, ()>(())
    })
    .unwrap();
    assert!(received > 0);
    assert_eq!(outcome.stdout_bytes, received);
    assert_eq!(outcome.process.stop, ProcessStop::Cancelled);
    assert!(outcome.consumer_failure.is_none());
    gone(&outcome.process);
}

#[test]
fn shutdown_drain_delivers_output_queued_before_cancellation() {
    let marker = std::env::temp_dir().join(format!(
        "surgeist-stream-drain-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let worker = spec(&["stream-drain", marker.to_str().unwrap()], 0, 0);
    let cancelled = AtomicBool::new(false);
    let mut before_cancellation = 0;
    let mut after_cancellation = 0;
    let outcome = execute_streaming(&worker, &cancelled, &mut |stream, bytes| {
        if stream == OutputStream::Stdout {
            assert!(bytes.iter().all(|byte| *byte == b'x'));
            if cancelled.load(Ordering::Acquire) {
                after_cancellation += bytes.len();
            } else {
                before_cancellation += bytes.len();
                // Observe that all 16 KiB reached the pipe before cancellation.
                // The supervisor reads one bounded chunk per stream per turn,
                // so the remaining bytes must be delivered during cleanup.
                let deadline = Instant::now() + Duration::from_secs(2);
                while !marker.exists() {
                    assert!(
                        Instant::now() < deadline,
                        "worker did not finish queued output"
                    );
                    std::thread::sleep(Duration::from_millis(2));
                }
                cancelled.store(true, Ordering::Release);
            }
        }
        Ok::<_, ()>(())
    })
    .unwrap();
    std::fs::remove_file(&marker).unwrap();
    assert_eq!(outcome.process.stop, ProcessStop::Cancelled);
    assert!(outcome.consumer_failure.is_none());
    assert!(before_cancellation > 0 && after_cancellation > 0);
    assert_eq!(before_cancellation + after_cancellation, 16 * 1024);
    assert_eq!(outcome.stdout_bytes, 16 * 1024);
    gone(&outcome.process);
}

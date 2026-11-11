//! The harness reexecutes this test binary as the controlled worker.
#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, exit},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use surgeist_js_wpt::{
    CleanupStatus, ExecutionError, FixtureId, FixtureUrlError, GuestDiagnostic, HarnessResult,
    HarnessStatus, MemoryPolicy, OutputStream, ProcessExit, ProcessStop, ProtocolError,
    RelativePath, Report, Run, SourceRevision, StartError, Subtest, SubtestStatus, WorkerFailure,
    WorkerLimits, WorkerResult, execution_capabilities, read_request, run, write_report,
};

const REVISION: &str = "69f2bba61ce7e9469ccbce53de4644a89f6b7dff";
const FIXTURE: &str = "dom/controlled.window.js";
const URL: &str = "/dom/controlled.window.html?example=%23#Exact";
const MAX_FRAME: u32 = 32_768;

fn fixture(url: &str) -> FixtureId {
    FixtureId::new(
        SourceRevision::new(REVISION).unwrap(),
        RelativePath::new(FIXTURE).unwrap(),
        url.into(),
    )
    .unwrap()
}

fn diagnostic() -> GuestDiagnostic {
    GuestDiagnostic {
        message: Some(vec![0xd800, 0x0061, 0xdc00]),
        stack: Some(vec![]),
    }
}

fn mixed_result(status: HarnessStatus) -> WorkerResult {
    let subtests = [
        SubtestStatus::Pass,
        SubtestStatus::Fail,
        SubtestStatus::Timeout,
        SubtestStatus::NotRun,
        SubtestStatus::PreconditionFailed,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, status)| Subtest {
        index: u32::try_from(index).unwrap(),
        name: vec![0xd800],
        status,
        diagnostic: diagnostic(),
    })
    .collect();
    WorkerResult::Harness(HarnessResult {
        status,
        diagnostic: diagnostic(),
        subtests,
    })
}

fn worker_failure(mode: &str) -> Option<WorkerFailure> {
    Some(match mode {
        "syntax" => WorkerFailure::GuestSyntax(diagnostic()),
        "runtime" => WorkerFailure::GuestRuntime(diagnostic()),
        "native" => WorkerFailure::NativeRefusal {
            owner: "DOM".into(),
            code: "admission".into(),
            diagnostic: diagnostic(),
        },
        "capability" => WorkerFailure::UnavailableCapability {
            capability: "Window load".into(),
            diagnostic: diagnostic(),
        },
        "engine" => WorkerFailure::EngineIncompatibility {
            evidence: "controlled classification".into(),
            diagnostic: diagnostic(),
        },
        _ => return None,
    })
}

fn limits(input: usize, output: usize, timeout: Duration) -> WorkerLimits {
    WorkerLimits::new(
        input,
        output,
        1024,
        timeout,
        Duration::from_millis(100),
        Duration::from_secs(3),
        MemoryPolicy::Unbounded,
    )
    .unwrap()
}

fn default_limits() -> WorkerLimits {
    limits(4096, 4096, Duration::from_secs(5))
}

fn invoke(
    cwd: &Path,
    mode: &str,
    limits: WorkerLimits,
    cancel: &AtomicBool,
) -> Result<Run, StartError> {
    run(
        &fixture(URL),
        env::current_exe().unwrap(),
        cwd.to_owned(),
        vec!["worker".into(), mode.into()],
        BTreeMap::new(),
        limits,
        cancel,
    )
}

fn clean(run: &Run) {
    assert_eq!(run.process.cleanup, CleanupStatus::Complete, "{run:?}");
    assert!(run.process.stdout.eof && run.process.stderr.eof, "{run:?}");
    assert_eq!(run.process.sampled_peak_rss_bytes, None);
}

fn received(run: &Run, expected: &WorkerResult) {
    clean(run);
    assert_eq!(run.fixture, fixture(URL));
    assert_eq!(run.process.stop, ProcessStop::Exited);
    assert_eq!(run.process.exit, ProcessExit::Success);
    assert!(
        matches!(&run.report, Report::Received(actual) if actual == expected),
        "{run:?}"
    );
}

fn validate_identity() {
    let identity = fixture(URL);
    assert_eq!(identity.revision().as_str(), REVISION);
    assert_eq!(identity.path().as_str(), FIXTURE);
    assert_eq!(identity.url(), URL);
    assert!(SourceRevision::new("abcd").is_err());
    assert!(RelativePath::new("dom/../outside.js").is_err());
    for url in [
        "/dom/controlled.window.html",
        "/dom/controlled.window.html#Case",
    ] {
        assert_eq!(fixture(url).url(), url);
    }
    for (url, expected) in [
        ("https://example.invalid/test.html", FixtureUrlError::Path),
        ("//example.invalid/test.html", FixtureUrlError::Path),
        ("/dom/../test.html", FixtureUrlError::Path),
        ("/dom/test.html?", FixtureUrlError::Variant),
        ("/dom/test.html#", FixtureUrlError::Variant),
        ("/dom/test.html?#case", FixtureUrlError::Variant),
        ("/dom/test.html?case\n", FixtureUrlError::Variant),
    ] {
        assert_eq!(
            FixtureId::new(
                identity.revision().clone(),
                identity.path().clone(),
                url.into()
            ),
            Err(expected),
            "{url:?}",
        );
    }
}

fn verify_worker(cwd: &Path) {
    let cancel = AtomicBool::new(false);
    let run = invoke(cwd, "mixed", default_limits(), &cancel).unwrap();
    received(&run, &mixed_result(HarnessStatus::Error));
    assert_eq!(run.process.stderr.bytes, b"controlled diagnostic");

    let raw = invoke(cwd, "raw-result", default_limits(), &cancel).unwrap();
    received(
        &raw,
        &WorkerResult::Harness(HarnessResult {
            status: HarnessStatus::Ok,
            diagnostic: GuestDiagnostic {
                message: None,
                stack: None,
            },
            subtests: vec![Subtest {
                index: 0,
                name: vec![0xd800],
                status: SubtestStatus::NotRun,
                diagnostic: GuestDiagnostic {
                    message: None,
                    stack: Some(vec![]),
                },
            }],
        }),
    );
    let timeout = invoke(cwd, "harness-timeout", default_limits(), &cancel).unwrap();
    received(&timeout, &mixed_result(HarnessStatus::Timeout));
    for mode in ["syntax", "runtime", "native", "capability", "engine"] {
        let observed = invoke(cwd, mode, default_limits(), &cancel).unwrap();
        received(
            &observed,
            &WorkerResult::Failure(worker_failure(mode).unwrap()),
        );
    }

    let input = run.process.input_bytes_written;
    let output = run.process.stdout.bytes.len();
    let bounds = |input, output| limits(input, output, Duration::from_secs(5));
    let exact = invoke(cwd, "mixed", bounds(input, output), &cancel).unwrap();
    received(&exact, &mixed_result(HarnessStatus::Error));
    assert!(matches!(
        invoke(cwd, "mixed", bounds(input - 1, output), &cancel),
        Err(StartError::Request(ProtocolError::Frame(_)))
    ));
    let output_over = invoke(cwd, "reply-then-hang", bounds(input, output - 1), &cancel).unwrap();
    clean(&output_over);
    assert_eq!(
        output_over.process.stop,
        ProcessStop::OutputLimit(OutputStream::Stdout)
    );
    assert!(output_over.process.stdout.truncated);
    assert!(matches!(output_over.report, Report::TransportFailed));

    for mode in [
        "missing", "partial", "version", "bad-path", "trailing", "extra", "mismatch",
    ] {
        let run = invoke(cwd, mode, default_limits(), &cancel).unwrap();
        clean(&run);
        assert_eq!(run.process.stop, ProcessStop::Exited);
        assert_eq!(run.process.exit, ProcessExit::Success);
        let Report::ProtocolFailure(error) = &run.report else {
            panic!("{mode}: {run:?}");
        };
        assert!(
            match mode {
                "missing" => matches!(error, ProtocolError::MissingFrame),
                "partial" | "version" | "bad-path" => matches!(error, ProtocolError::Frame(_)),
                "trailing" | "extra" => matches!(error, ProtocolError::TrailingData),
                "mismatch" => matches!(error, ProtocolError::FixtureMismatch),
                _ => false,
            },
            "{mode}: {run:?}"
        );
    }

    let stderr_over = invoke(cwd, "stderr-overflow", default_limits(), &cancel).unwrap();
    clean(&stderr_over);
    assert_eq!(
        stderr_over.process.stop,
        ProcessStop::OutputLimit(OutputStream::Stderr)
    );
    assert!(stderr_over.process.stderr.truncated);
    assert!(matches!(stderr_over.report, Report::TransportFailed));

    let nonzero = invoke(cwd, "nonzero", default_limits(), &cancel).unwrap();
    clean(&nonzero);
    assert_eq!(nonzero.process.stop, ProcessStop::Exited);
    assert_eq!(nonzero.process.exit, ProcessExit::Nonzero(17));
    assert!(!nonzero.process.stdout.bytes.is_empty());
    assert!(matches!(nonzero.report, Report::TransportFailed));
    let crash = invoke(cwd, "abort", default_limits(), &cancel).unwrap();
    clean(&crash);
    assert_eq!(crash.process.stop, ProcessStop::Exited);
    assert!(
        matches!(crash.process.exit, ProcessExit::Signal(_)),
        "{crash:?}"
    );
    assert!(matches!(crash.report, Report::TransportFailed));

    let deadline = invoke(
        cwd,
        "reply-then-hang",
        limits(4096, 4096, Duration::from_secs(2)),
        &cancel,
    )
    .unwrap();
    clean(&deadline);
    assert_eq!(deadline.process.stop, ProcessStop::Deadline);
    assert_eq!(deadline.process.stdout.bytes, run.process.stdout.bytes);
    assert!(matches!(deadline.report, Report::TransportFailed));
    println!("qualified result framing, typed outcomes, quotas, crashes and deadline");
}

fn wait_ready(path: &Path) {
    let start = Instant::now();
    loop {
        match fs::read(path) {
            Ok(bytes) if bytes == b"ready" => return,
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => panic!("reading readiness at {path:?}: {error}"),
        }
        assert!(
            start.elapsed() < Duration::from_secs(4),
            "worker readiness at {path:?} after {:?}",
            start.elapsed()
        );
        thread::park_timeout(Duration::from_millis(5));
    }
}

fn verify_cancellation(cwd: &Path) {
    let cancel = AtomicBool::new(true);
    assert!(matches!(
        invoke(cwd, "mixed", default_limits(), &cancel),
        Err(StartError::Process(ExecutionError::CancelledBeforeSpawn))
    ));
    cancel.store(false, Ordering::Release);
    thread::scope(|scope| {
        let observer = scope.spawn(|| {
            wait_ready(&cwd.join("descendant-ready"));
            cancel.store(true, Ordering::Release);
        });
        let run = invoke(cwd, "descendant", default_limits(), &cancel).unwrap();
        observer.join().unwrap();
        clean(&run);
        assert_eq!(run.process.stop, ProcessStop::Cancelled);
        assert!(
            matches!(run.process.exit, ProcessExit::Signal(_)),
            "{run:?}"
        );
        assert!(matches!(run.report, Report::TransportFailed));
    });
    println!("qualified cancellation after an owned descendant inherited the worker pipes");
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("surgeist-js-wpt-{}-{unique}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0)
            && error.kind() != io::ErrorKind::NotFound
            && !thread::panicking()
        {
            panic!("removing worker test scratch {:?}: {error}", self.0);
        }
    }
}

fn park() -> ! {
    loop {
        thread::park();
    }
}

#[expect(
    clippy::zombie_processes,
    reason = "controlled descendant stays in the worker group for supervised cleanup"
)]
fn start_descendant() {
    Command::new(env::current_exe().unwrap())
        .arg("descendant")
        .spawn()
        .unwrap();
}

fn write_raw(output: &mut impl Write, json: &[u8]) {
    output
        .write_all(&u32::try_from(json.len()).unwrap().to_be_bytes())
        .unwrap();
    output.write_all(json).unwrap();
}

fn worker(mode: &str) {
    let identity = read_request(&mut io::stdin().lock(), MAX_FRAME).unwrap();
    assert_eq!(identity, fixture(URL));
    let mut stdout = io::stdout().lock();
    let mixed = mixed_result(HarnessStatus::Error);
    if let Some(failure) = worker_failure(mode) {
        write_report(
            &mut stdout,
            &identity,
            &WorkerResult::Failure(failure),
            MAX_FRAME,
        )
        .unwrap();
        return;
    }
    match mode {
        "mixed" => {
            io::stderr().write_all(b"controlled diagnostic").unwrap();
            write_report(&mut stdout, &identity, &mixed, MAX_FRAME).unwrap();
        }
        "harness-timeout" => {
            write_report(
                &mut stdout,
                &identity,
                &mixed_result(HarnessStatus::Timeout),
                MAX_FRAME,
            )
            .unwrap();
        }
        "raw-result" => {
            let json = format!(
                r#"{{"version":"V1","fixture":{{"revision":"{REVISION}","path":"{FIXTURE}","url":"{URL}"}},"result":{{"harness":{{"status":"OK","diagnostic":{{"message":null,"stack":null}},"subtests":[{{"index":0,"name":[55296],"status":"NOTRUN","diagnostic":{{"message":null,"stack":[]}}}}]}}}}}}"#
            );
            write_raw(&mut stdout, json.as_bytes());
        }
        "missing" => {}
        "partial" => stdout.write_all(&[0, 0, 0, 8, b'{']).unwrap(),
        "version" => write_raw(
            &mut stdout,
            br#"{"version":"V2","fixture":null,"result":null}"#,
        ),
        "bad-path" => {
            let json = format!(
                r#"{{"version":"V1","fixture":{{"revision":"{REVISION}","path":"../bad.js","url":"{URL}"}},"result":null}}"#
            );
            write_raw(&mut stdout, json.as_bytes());
        }
        "trailing" => {
            write_report(&mut stdout, &identity, &mixed, MAX_FRAME).unwrap();
            stdout.write_all(b"x").unwrap();
        }
        "extra" => {
            write_report(&mut stdout, &identity, &mixed, MAX_FRAME).unwrap();
            write_report(&mut stdout, &identity, &mixed, MAX_FRAME).unwrap();
        }
        "mismatch" => {
            write_report(
                &mut stdout,
                &fixture("/dom/controlled.window.html?other"),
                &mixed,
                MAX_FRAME,
            )
            .unwrap();
        }
        "stderr-overflow" => {
            io::stderr().write_all(&[b'x'; 1025]).unwrap();
            park();
        }

        "nonzero" => {
            write_report(&mut stdout, &identity, &mixed, MAX_FRAME).unwrap();
            exit(17);
        }
        "abort" => std::process::abort(),
        "reply-then-hang" => {
            write_report(&mut stdout, &identity, &mixed, MAX_FRAME).unwrap();
            park();
        }
        "descendant" => {
            start_descendant();
            park();
        }
        _ => panic!("unknown controlled mode {mode}"),
    }
    stdout.flush().unwrap();
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("worker") => worker(&args.next().unwrap()),
        Some("descendant") => {
            fs::write("descendant-ready", b"ready").unwrap();
            park();
        }
        _ => {
            validate_identity();
            let scratch = Scratch::new();
            if execution_capabilities().supported_host {
                verify_worker(&scratch.0);
                verify_cancellation(&scratch.0);
            } else {
                assert!(matches!(
                    invoke(
                        &scratch.0,
                        "mixed",
                        default_limits(),
                        &AtomicBool::new(false)
                    ),
                    Err(StartError::Process(ExecutionError::UnsupportedHost))
                ));
            }
        }
    }
}

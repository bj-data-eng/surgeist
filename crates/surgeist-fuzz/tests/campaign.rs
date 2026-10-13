#![forbid(unsafe_code)]
#![cfg(all(feature = "execution", target_os = "macos", target_arch = "aarch64"))]

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, Instant},
};
use surgeist_fuzz::{
    ArtifactStore, RelativePath, StorageLimits,
    campaign::{CampaignStatus, CampaignSummary, run_cli},
    execution::{
        CleanupStatus, MemoryPolicy, ProcessExit, ProcessStop, WorkerLimits, WorkerSpec, execute,
    },
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Snapshot {
    root: PathBuf,
    source: PathBuf,
    work: PathBuf,
}
impl Snapshot {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "surgeist-campaign-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let source = root.join("source");
        fs::create_dir_all(source.join("crates")).unwrap();
        let original = Path::new(env!("CARGO_MANIFEST_DIR"));
        copy_dir(original, &source.join("crates/surgeist-fuzz"));
        copy_dir(
            &original.join("../surgeist-generator"),
            &source.join("crates/surgeist-generator"),
        );
        fs::write(source.join("Cargo.toml"),"[workspace]\nresolver='3'\nmembers=['crates/surgeist-fuzz','crates/surgeist-generator']\n[workspace.package]\nedition='2024'\nrust-version='1.97'\nlicense='MIT'\nrepository='https://github.com/bj-data-eng/surgeist'\n").unwrap();
        fs::write(source.join(".gitignore"), "target/\n").unwrap();
        fs::copy(original.join("../../Cargo.lock"), source.join("Cargo.lock")).unwrap();
        let fixture = source.join("crates/surgeist-fuzz/tests/fixtures/subject");
        tool(
            "cargo",
            &source,
            &[
                "generate-lockfile",
                "--offline",
                "--manifest-path",
                fixture.join("Cargo.toml").to_str().unwrap(),
            ],
        );
        tool("git", &source, &["init", "-q"]);
        tool("git", &source, &["config", "user.name", "Surgeist fixture"]);
        tool(
            "git",
            &source,
            &["config", "user.email", "fixture@example.invalid"],
        );
        tool("git", &source, &["add", "."]);
        tool(
            "git",
            &source,
            &[
                "commit",
                "-q",
                "-m",
                "test(fuzz): record isolated fixture basis",
            ],
        );
        Self {
            work: root.join("work"),
            root,
            source,
        }
    }
    fn args(&self, command: &str, target: &str) -> Vec<OsString> {
        vec![
            command.into(),
            "--source".into(),
            self.source.as_os_str().to_owned(),
            "--manifest".into(),
            "crates/surgeist-fuzz/tests/fixtures/subject/Cargo.toml".into(),
            "--target".into(),
            target.into(),
            "--work".into(),
            self.work.as_os_str().to_owned(),
            "--memory-policy".into(),
            "unbounded".into(),
            "--cases".into(),
            "1".into(),
            "--wall-ms".into(),
            "5000".into(),
            "--case-ms".into(),
            "500".into(),
            "--build-ms".into(),
            "180000".into(),
        ]
    }
    fn invoke(&self, args: Vec<OsString>) -> CampaignSummary {
        run_cli(&args, &AtomicBool::new(false)).unwrap()
    }
    fn input(&self, bytes: &[u8]) -> PathBuf {
        let path = self.root.join("input.case");
        fs::write(&path, bytes).unwrap();
        path
    }
}
impl Drop for Snapshot {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!(
                "failed campaign evidence retained at {}",
                self.root.display()
            );
        } else {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if ["target", "tmp", ".git"].iter().any(|n| name == *n) {
            continue;
        }
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            copy_dir(&entry.path(), &to.join(name));
        } else if kind.is_file() {
            fs::copy(entry.path(), to.join(name)).unwrap();
        } else {
            panic!("unexpected source symlink");
        }
    }
}
fn tool(name: &str, cwd: &Path, args: &[&str]) {
    let path = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|p| p.join(name))
        .find(|p| p.is_file())
        .unwrap();
    let mut env = BTreeMap::new();
    for key in ["PATH", "HOME", "RUSTUP_HOME", "CARGO_HOME", "TMPDIR"] {
        if let Some(v) = std::env::var_os(key) {
            env.insert(key.into(), v);
        }
    }
    env.insert("CARGO_NET_OFFLINE".into(), "true".into());
    env.insert("CARGO_BUILD_JOBS".into(), "1".into());
    let limits = WorkerLimits::new(
        0,
        1024 * 1024,
        1024 * 1024,
        Duration::from_secs(60),
        Duration::from_millis(100),
        Duration::from_secs(3),
        MemoryPolicy::Unbounded,
    )
    .unwrap();
    let spec = WorkerSpec::new(
        path,
        cwd.to_owned(),
        args.iter().map(OsString::from).collect(),
        env,
        Vec::new(),
        limits,
    )
    .unwrap();
    let result = execute(&spec, &AtomicBool::new(false)).unwrap();
    assert_eq!(result.cleanup, CleanupStatus::Complete);
    assert_eq!(result.stop, ProcessStop::Exited);
    assert_eq!(
        result.exit,
        ProcessExit::Success,
        "{}",
        String::from_utf8_lossy(&result.stderr.bytes)
    );
}

#[test]
fn parser_and_no_write_preflight_reject_ambiguous_or_unavailable_requests() {
    let root = std::env::temp_dir().join(format!("surgeist-preflight-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let work = root.join("must-not-exist");
    for args in [
        vec!["unknown".into()],
        vec!["check".into(), "--workers".into(), "2".into()],
        vec!["check".into(), "--cases".into(), "0".into()],
        vec![
            "check".into(),
            "--cases".into(),
            "1".into(),
            "--cases".into(),
            "2".into(),
        ],
        vec!["check".into(), "--memory-policy".into(), "hard".into()],
        vec!["run".into(), "--toolchain".into(), "stable".into()],
        vec![
            "run".into(),
            "--continuous".into(),
            "--cases".into(),
            "1".into(),
        ],
        vec![
            "run".into(),
            "--continuous".into(),
            "--wall-ms".into(),
            "1".into(),
        ],
        vec![
            "run".into(),
            "--continuous".into(),
            "--workers".into(),
            "2".into(),
        ],
        vec!["check".into(), "--continuous".into()],
    ] {
        let mut args: Vec<OsString> = args;
        args.extend(["--work".into(), work.as_os_str().to_owned()]);
        assert!(run_cli(&args, &AtomicBool::new(false)).is_err());
        assert!(!work.exists());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn list_and_validate_discover_without_building_or_starting_campaigns() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    for command in ["list", "validate"] {
        let args: Vec<OsString> = [
            command,
            "--source",
            source.to_str().unwrap(),
            "--manifest",
            "crates/surgeist-fuzz/tests/fixtures/subject/Cargo.toml",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        let result = run_cli(&args, &AtomicBool::new(false)).unwrap();
        assert_eq!(result.status, CampaignStatus::Sufficient);
        assert_eq!(result.targets.len(), 3);
        assert!(result.builds.is_empty());
        assert!(result.runs.is_empty());
    }
}

#[test]
fn coverage_session_controls_reject_zero_budgets_before_writing() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-session-preflight-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let work = root.join("must-not-exist");
    for (request, expected) in [
        (vec!["run", "--continuous", "--cases", "1"], "--continuous"),
        (
            vec!["run", "--continuous", "--wall-ms", "1"],
            "--continuous",
        ),
        (vec!["run", "--continuous", "--workers", "2"], "one worker"),
        (vec!["check", "--continuous"], "requires run"),
    ] {
        let mut args: Vec<OsString> = request.into_iter().map(Into::into).collect();
        args.extend(["--work".into(), work.as_os_str().to_owned()]);
        let error = run_cli(&args, &AtomicBool::new(false)).unwrap_err();
        assert!(error.detail().contains(expected), "{error}");
        assert!(!work.exists());
    }
    for option in [
        "--round-cases",
        "--round-ms",
        "--retained-entries",
        "--session-bytes",
        "--session-entries",
        "--log-bytes",
        "--progress-ms",
    ] {
        let args: Vec<OsString> = vec![
            "run".into(),
            "--continuous".into(),
            "--toolchain".into(),
            "nightly-2026-09-01".into(),
            "--work".into(),
            work.as_os_str().to_owned(),
            option.into(),
            "0".into(),
        ];
        let error = run_cli(&args, &AtomicBool::new(false)).unwrap_err();
        assert!(error.detail().contains("positive"), "{option}: {error}");
        assert!(!work.exists(), "{option} admitted campaign writes");
    }
    for cases in ["1", "2"] {
        let args: Vec<OsString> = vec![
            "run".into(),
            "--continuous".into(),
            "--round-cases".into(),
            cases.into(),
            "--work".into(),
            work.as_os_str().to_owned(),
        ];
        let error = run_cli(&args, &AtomicBool::new(false)).unwrap_err();
        assert!(error.detail().contains("at least three"), "{error}");
        assert!(!work.exists());
    }
    fs::remove_dir_all(root).unwrap();
}

fn set_option(args: &mut [OsString], option: &str, value: &str) {
    let index = args.iter().position(|arg| arg == option).unwrap();
    args[index + 1] = value.into();
}

fn streaming_args(snapshot: &Snapshot) -> Vec<OsString> {
    let mut args = snapshot.args("run", "surgeist-fixture::streaming");
    set_option(&mut args, "--case-ms", "1000");
    args.extend([
        "--toolchain".into(),
        "nightly-2026-09-01".into(),
        "--retained-entries".into(),
        "2".into(),
        "--log-bytes".into(),
        "4096".into(),
    ]);
    args
}

fn checkpoint(summary: &CampaignSummary) -> serde_json::Value {
    let path = Path::new(&summary.runs[0]).join("session-state/progress.json");
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn regular_file_usage(root: &Path) -> (u64, u64) {
    if !root.exists() {
        return (0, 0);
    }
    let mut entries = 0;
    let mut bytes = 0;
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let metadata = fs::symlink_metadata(entry.path()).unwrap();
        assert!(!metadata.file_type().is_symlink());
        if metadata.is_dir() {
            let (nested_entries, nested_bytes) = regular_file_usage(&entry.path());
            entries += nested_entries;
            bytes += nested_bytes;
        } else {
            assert!(metadata.is_file());
            entries += 1;
            bytes += metadata.len();
        }
    }
    (entries, bytes)
}

fn await_round_checkpoint(
    snapshot: &Snapshot,
    running: &std::thread::JoinHandle<surgeist_fuzz::Result<CampaignSummary>>,
    minimum: u64,
    previous_sessions: &BTreeSet<PathBuf>,
) -> Option<serde_json::Value> {
    let start = Instant::now();
    let mut observed = None;
    while start.elapsed() < Duration::from_secs(240) && !running.is_finished() {
        if let Ok(sessions) = fs::read_dir(snapshot.work.join("sessions")) {
            for session in sessions.flatten() {
                if previous_sessions.contains(&session.path()) {
                    continue;
                }
                let path = session.path().join("session-state/progress.json");
                if let Ok(bytes) = fs::read(path)
                    && let Ok(progress) = serde_json::from_slice::<serde_json::Value>(&bytes)
                {
                    let enough = progress["completed_rounds"].as_u64().unwrap_or(0) >= minimum;
                    observed = Some(progress);
                    if enough {
                        return observed;
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    observed
}

#[test]
#[ignore = "compiles an isolated instrumented owner and streams 20,000 real evaluations"]
fn twenty_thousand_coverage_evaluations_keep_exact_totals_without_success_archives() {
    let snapshot = Snapshot::new();
    let mut args = streaming_args(&snapshot);
    set_option(&mut args, "--cases", "20000");
    set_option(&mut args, "--wall-ms", "120000");
    args.extend(["--round-cases".into(), "1000".into()]);
    let summary = snapshot.invoke(args);
    assert_eq!(summary.status, CampaignStatus::Sufficient, "{summary:?}");
    assert_eq!(summary.evaluated, 20_000);
    assert_eq!(summary.generation_rejected, 0);
    assert_eq!(summary.cleanup, "complete");
    // Every successful evaluation is preceded by a 1024-byte canonical capture,
    // independently forcing the wire payload beyond the former stdout ceiling.
    assert!(summary.evaluated * 1024 > 16 * 1024 * 1024);
    assert_eq!(summary.builds.len(), 1);
    assert_eq!(summary.runs.len(), 1);
    assert_eq!(summary.corpora.len(), 1);
    let progress = checkpoint(&summary);
    assert_eq!(progress["mode"], "finite");
    assert_eq!(progress["evaluated"], 20_000);
    assert_eq!(progress["generation_rejected"], 0);
    assert_eq!(progress["terminal"], "sufficient");
    assert!(progress["stdout_bytes"].as_u64().unwrap() > 16 * 1024 * 1024);
    assert_eq!(
        progress["checked_properties"],
        serde_json::json!(["surgeist-fixture::streaming-byte-count"])
    );
    assert_eq!(
        progress["build"],
        serde_json::to_value(&summary.builds[0]).unwrap()
    );
    assert!(progress["retained_entries"].as_u64().unwrap() <= 2);
    let (entries, bytes) = regular_file_usage(Path::new(&summary.corpora[0]));
    assert!(entries <= 2);
    assert!(bytes <= 2 * 4096);
    let (session_entries, session_bytes) =
        regular_file_usage(&Path::new(&summary.runs[0]).join("session-state"));
    assert!(session_entries <= 4096);
    assert!(session_bytes <= 256 * 1024 * 1024);
    assert!(
        session_entries <= 32,
        "success state accumulated {session_entries} files"
    );
    assert!(
        session_bytes <= 128 * 1024,
        "success state accumulated {session_bytes} bytes"
    );
    eprintln!(
        "finite coverage: {} evaluations, {} engine campaign ms, {session_entries} persisted session files, {session_bytes} persisted session bytes; build/setup time excluded",
        summary.evaluated, progress["elapsed_millis"]
    );
    assert_eq!(
        regular_file_usage(&snapshot.work.join("artifacts/corpus")),
        (0, 0)
    );
    assert_eq!(
        regular_file_usage(&snapshot.work.join("artifacts/failures")),
        (0, 0)
    );
    assert_eq!(
        regular_file_usage(&snapshot.work.join("artifacts/captures")),
        (0, 0)
    );
    let report = snapshot.invoke(snapshot.args("report", "surgeist-fixture::streaming"));
    assert_eq!(report.recorded_runs.len(), 1);
    assert!(report.recorded_runs[0].counts().evaluated() > 0);
    assert!(report.recorded_runs[0].counts().evaluated() <= 1000);
    assert_eq!(report.recorded_runs[0].counts().generation_rejected(), 0);
    assert_eq!(
        report.recorded_runs[0].outcome(),
        surgeist_fuzz::RunOutcome::Sufficient
    );
}

#[test]
#[ignore = "compiles an isolated instrumented owner and cancels after observed coverage rounds"]
fn continuous_coverage_cancels_after_multiple_rounds_and_restarts_with_recorded_provenance() {
    let snapshot = Snapshot::new();
    let cancel = snapshot.root.join("cancel");
    let mut args = streaming_args(&snapshot);
    for option in ["--cases", "--wall-ms"] {
        let index = args.iter().position(|arg| arg == option).unwrap();
        args.drain(index..index + 2);
    }
    args.extend([
        "--continuous".into(),
        "--round-cases".into(),
        "32".into(),
        "--round-ms".into(),
        "5000".into(),
        "--progress-ms".into(),
        "1".into(),
        "--cancel-file".into(),
        cancel.as_os_str().to_owned(),
    ]);
    let continuous_args = args.clone();
    let running = std::thread::spawn(move || run_cli(&args, &AtomicBool::new(false)));
    let start = Instant::now();
    let observed = await_round_checkpoint(&snapshot, &running, 3, &BTreeSet::new());
    // Request cancellation even on a timeout before awaiting the owned thread.
    fs::write(&cancel, b"cancel after observable completed rounds").unwrap();
    let summary = running.join().unwrap().unwrap();
    let observed = observed.expect("no progress checkpoint observed before bounded wait ended");
    assert!(
        observed["completed_rounds"].as_u64().unwrap_or(0) >= 3,
        "wait ended after {:?}; last checkpoint {observed}; summary {summary:?}",
        start.elapsed()
    );
    assert_eq!(summary.status, CampaignStatus::Cancelled, "{summary:?}");
    assert_eq!(summary.cleanup, "complete");
    assert!(summary.evaluated >= 96);
    assert_eq!(summary.generation_rejected, 0);
    assert_eq!(summary.builds.len(), 1);
    assert_eq!(summary.runs.len(), 1);
    let terminal = checkpoint(&summary);
    assert_eq!(terminal["mode"], "continuous");
    assert_eq!(terminal["terminal"], "cancelled");
    assert_eq!(terminal["round_cases"], 32);
    assert_eq!(terminal["round_millis"], 5000);
    assert_eq!(terminal["evaluated"], summary.evaluated);
    assert_eq!(terminal["generation_rejected"], summary.generation_rejected);
    eprintln!(
        "continuous coverage: {} evaluations across {} rounds, {} engine campaign ms; build/setup time excluded",
        summary.evaluated, terminal["completed_rounds"], terminal["elapsed_millis"]
    );
    assert_eq!(
        terminal["checked_properties"],
        serde_json::json!(["surgeist-fixture::streaming-byte-count"])
    );
    assert_eq!(
        terminal["build"],
        serde_json::to_value(&summary.builds[0]).unwrap()
    );
    assert!(terminal["retained_entries"].as_u64().unwrap() <= 2);
    assert_eq!(
        regular_file_usage(&snapshot.work.join("artifacts/corpus")),
        (0, 0)
    );
    let report = snapshot.invoke(snapshot.args("report", "surgeist-fixture::streaming"));
    assert_eq!(report.recorded_runs.len(), 1);
    assert_eq!(
        report.recorded_runs[0].outcome(),
        surgeist_fuzz::RunOutcome::Cancelled
    );
    let source_corpus = PathBuf::from(&summary.corpora[0]);
    let originals: BTreeMap<_, _> = fs::read_dir(&source_corpus)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name(), fs::read(entry.path()).unwrap())
        })
        .collect();
    let mut restart = streaming_args(&snapshot);
    set_option(&mut restart, "--cases", "64");
    set_option(&mut restart, "--wall-ms", "30000");
    restart.extend([
        "--restart-corpus".into(),
        source_corpus.as_os_str().to_owned(),
    ]);
    let restarted = snapshot.invoke(restart);
    assert_eq!(
        restarted.status,
        CampaignStatus::Sufficient,
        "{restarted:?}"
    );
    assert_eq!(restarted.evaluated, 64);
    assert_eq!(restarted.cleanup, "complete");
    assert_eq!(restarted.builds, summary.builds);
    assert_ne!(restarted.runs, summary.runs);
    assert_ne!(restarted.corpora, summary.corpora);
    assert_eq!(checkpoint(&restarted)["build"], terminal["build"]);
    for (name, bytes) in originals {
        assert_eq!(fs::read(source_corpus.join(name)).unwrap(), bytes);
    }
    let previous_sessions: BTreeSet<_> = fs::read_dir(snapshot.work.join("sessions"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    let changed_cancel = snapshot.root.join("cancel-source-change");
    let mut args = continuous_args;
    let index = args.iter().position(|arg| arg == "--cancel-file").unwrap();
    args[index + 1] = changed_cancel.as_os_str().to_owned();
    let running = std::thread::spawn(move || run_cli(&args, &AtomicBool::new(false)));
    let observed = await_round_checkpoint(&snapshot, &running, 1, &previous_sessions);
    let observed_round = observed
        .as_ref()
        .is_some_and(|progress| progress["completed_rounds"].as_u64().unwrap_or(0) >= 1);
    if observed_round {
        fs::write(
            snapshot.source.join("unexpected-source-change.txt"),
            b"changed after engine round",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        while !running.is_finished() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    let stopped_on_source_change = running.is_finished();
    if !stopped_on_source_change {
        fs::write(&changed_cancel, b"bounded source-change wait expired").unwrap();
    }
    let changed = running.join().unwrap().unwrap();
    assert!(
        observed_round,
        "no fresh source-change round observed; last checkpoint {observed:?}; result {changed:?}"
    );
    assert!(
        stopped_on_source_change,
        "source-change stop deadline expired; result {changed:?}"
    );
    assert_eq!(
        changed.status,
        CampaignStatus::InfrastructureFailure,
        "{changed:?}"
    );
    assert_eq!(changed.cleanup, "complete");
    assert_eq!(changed.builds, summary.builds);
    assert_eq!(checkpoint(&changed)["terminal"], "infrastructure-failure");
}

#[test]
#[ignore = "compiles isolated owner source bases and exercises campaign processes"]
fn isolated_campaigns_capture_replay_reduce_and_compare_two_source_builds() {
    let snapshot = Snapshot::new();
    let input = snapshot.input(b"padding BUG! tail ALT!");
    let mut args = snapshot.args("check", "surgeist-fixture::bytes");
    args.extend(["--input".into(), input.as_os_str().to_owned()]);
    let finding = snapshot.invoke(args);
    assert_eq!(finding.status, CampaignStatus::Finding);
    assert!(!finding.artifacts.is_empty());
    assert_eq!(finding.cleanup, "complete");
    let report = snapshot.invoke(snapshot.args("report", "surgeist-fixture::bytes"));
    assert_eq!(report.recorded_runs.len(), 1);
    assert_eq!(
        report.recorded_runs[0].outcome(),
        surgeist_fuzz::RunOutcome::Finding
    );
    assert_eq!(
        report.recorded_runs[0].counts().evaluated(),
        finding.evaluated
    );
    let original = finding
        .artifacts
        .iter()
        .find(|p| p.starts_with("failures/"))
        .unwrap()
        .clone();
    let mut args = snapshot.args("replay", "surgeist-fixture::bytes");
    args.extend(["--artifact".into(), original.clone().into()]);
    let replay = snapshot.invoke(args);
    assert_eq!(replay.status, CampaignStatus::Finding);
    let mut args = snapshot.args("minimize", "surgeist-fixture::bytes");
    args.extend([
        "--artifact".into(),
        original.clone().into(),
        "--attempts".into(),
        "100".into(),
    ]);
    let minimized = snapshot.invoke(args);
    assert_eq!(minimized.status, CampaignStatus::Finding);
    assert!(minimized.artifacts.iter().any(|p| p != &original));
    let store = ArtifactStore::open(
        snapshot.work.join("artifacts"),
        StorageLimits::new(1024, 64 * 1024 * 1024, 4096, 1024 * 1024).unwrap(),
    )
    .unwrap();
    let before = store
        .load_failure(&RelativePath::new(&original).unwrap())
        .unwrap();
    assert_eq!(before.input(), b"padding BUG! tail ALT!");
    let smallest = minimized
        .artifacts
        .iter()
        .filter(|p| p.starts_with("failures/") && *p != &original)
        .map(|p| store.load_failure(&RelativePath::new(p).unwrap()).unwrap())
        .min_by_key(|a| a.input().len())
        .unwrap();
    assert_eq!(smallest.input(), b"BUG!");
    assert_eq!(
        smallest.metadata().outcome().failure_key(),
        before.metadata().outcome().failure_key()
    );
    fs::write(
        snapshot.source.join("fixture-revision.txt"),
        b"new source basis",
    )
    .unwrap();
    tool("git", &snapshot.source, &["add", "fixture-revision.txt"]);
    tool(
        "git",
        &snapshot.source,
        &[
            "commit",
            "-q",
            "-m",
            "test(fuzz): record second isolated source basis",
        ],
    );
    let mut exact = snapshot.args("replay", "surgeist-fixture::bytes");
    exact.extend(["--artifact".into(), original.clone().into()]);
    assert!(run_cli(&exact, &AtomicBool::new(false)).is_err());
    exact.push("--comparison".into());
    let comparison = snapshot.invoke(exact);
    assert_eq!(comparison.status, CampaignStatus::Finding);
    assert_ne!(finding.builds[0].source(), comparison.builds[0].source());
    assert_eq!(
        store
            .load_failure(&RelativePath::new(original).unwrap())
            .unwrap()
            .input(),
        before.input()
    );
}

#[test]
#[ignore = "compiles an isolated owner and exercises real sanitizer-guided targets"]
fn raw_and_structured_coverage_findings_replay_the_materialized_owner_case() {
    let snapshot = Snapshot::new();
    for (target, entropy) in [
        ("surgeist-fixture::bytes", b"padding BUG! after".as_slice()),
        ("surgeist-fixture::structured", &[8, 0, 0xd8][..]),
    ] {
        let input = snapshot.input(entropy);
        let mut args = snapshot.args("run", target);
        let cases = args.iter().position(|arg| arg == "--cases").unwrap();
        args[cases + 1] = "2".into();
        let case = args.iter().position(|arg| arg == "--case-ms").unwrap();
        args[case + 1] = "1000".into();
        args.extend([
            "--toolchain".into(),
            "nightly-2026-09-01".into(),
            "--input".into(),
            input.as_os_str().to_owned(),
        ]);
        let finding = snapshot.invoke(args);
        assert_eq!(finding.status, CampaignStatus::Finding, "{finding:?}");
        assert!(finding.evaluated >= 1);
        assert_eq!(
            serde_json::to_value(&finding.builds[0]).unwrap()["engine"],
            "libfuzzer-sys 0.4.13; canonical replay via isolated owner protocol"
        );
        assert_eq!(finding.cleanup, "complete");
        let path = finding
            .artifacts
            .iter()
            .find(|p| p.starts_with("failures/"))
            .unwrap();
        let store = ArtifactStore::open(
            snapshot.work.join("artifacts"),
            StorageLimits::new(1024, 64 * 1024 * 1024, 4096, 1024 * 1024).unwrap(),
        )
        .unwrap();
        let artifact = store
            .load_failure(&RelativePath::new(path).unwrap())
            .unwrap();
        if target.ends_with("::bytes") {
            assert_eq!(artifact.input(), entropy);
        } else {
            assert_ne!(artifact.input(), entropy);
            let canonical: serde_json::Value = serde_json::from_slice(artifact.input()).unwrap();
            assert_eq!(canonical["text"][0], 0xd800);
            assert_eq!(canonical["node_count"], 8);
        }
        let mut replay = snapshot.args("replay", target);
        replay.extend([
            "--toolchain".into(),
            "nightly-2026-09-01".into(),
            "--artifact".into(),
            path.into(),
        ]);
        let replayed = snapshot.invoke(replay);
        assert_eq!(replayed.status, CampaignStatus::Finding, "{replayed:?}");
        assert_eq!(replayed.evaluated, 1);
        assert_eq!(replayed.cleanup, "complete");
        assert_eq!(replayed.detail, "original failure key reproduced");
        assert_eq!(
            store
                .load_failure(&RelativePath::new(path).unwrap())
                .unwrap()
                .input(),
            artifact.input()
        );
        if target.ends_with("::bytes") {
            let mut minimize = snapshot.args("minimize", target);
            let case = minimize.iter().position(|arg| arg == "--case-ms").unwrap();
            minimize[case + 1] = "1000".into();
            let cases = minimize.iter().position(|arg| arg == "--cases").unwrap();
            minimize[cases + 1] = "100".into();
            minimize.extend([
                "--toolchain".into(),
                "nightly-2026-09-01".into(),
                "--artifact".into(),
                path.into(),
                "--engine-proposal".into(),
            ]);
            let reduced = snapshot.invoke(minimize);
            assert!(
                matches!(
                    reduced.status,
                    CampaignStatus::Finding | CampaignStatus::ResourceStop
                ),
                "{reduced:?}"
            );
            assert!(
                reduced
                    .artifacts
                    .iter()
                    .filter(|p| p.starts_with("failures/"))
                    .any(|p| store
                        .load_failure(&RelativePath::new(p).unwrap())
                        .unwrap()
                        .input()
                        == b"BUG!")
            );
            let safe = snapshot.input(&[0]);
            let mut run = snapshot.args("run", target);
            let case = run.iter().position(|arg| arg == "--case-ms").unwrap();
            run[case + 1] = "1000".into();
            let cases = run.iter().position(|arg| arg == "--cases").unwrap();
            run[cases + 1] = "8".into();
            run.extend([
                "--toolchain".into(),
                "nightly-2026-09-01".into(),
                "--input".into(),
                safe.into_os_string(),
            ]);
            let safe_run = snapshot.invoke(run);
            assert_eq!(safe_run.status, CampaignStatus::Sufficient, "{safe_run:?}");
            let source_corpus = PathBuf::from(&safe_run.corpora[0]);
            let originals: BTreeMap<_, _> = fs::read_dir(&source_corpus)
                .unwrap()
                .map(|entry| {
                    let entry = entry.unwrap();
                    (entry.file_name(), fs::read(entry.path()).unwrap())
                })
                .collect();
            let mut minimize = snapshot.args("minimize", target);
            let case = minimize.iter().position(|arg| arg == "--case-ms").unwrap();
            minimize[case + 1] = "1000".into();
            let cases = minimize.iter().position(|arg| arg == "--cases").unwrap();
            minimize[cases + 1] = "100".into();
            minimize.extend([
                "--toolchain".into(),
                "nightly-2026-09-01".into(),
                "--corpus".into(),
                source_corpus.as_os_str().to_owned(),
            ]);
            let reduced_corpus = snapshot.invoke(minimize);
            assert_eq!(
                reduced_corpus.status,
                CampaignStatus::Sufficient,
                "{reduced_corpus:?}"
            );
            assert_eq!(reduced_corpus.corpora.len(), 1);
            let mut restart = snapshot.args("run", target);
            let case = restart.iter().position(|arg| arg == "--case-ms").unwrap();
            restart[case + 1] = "1000".into();
            let cases = restart.iter().position(|arg| arg == "--cases").unwrap();
            restart[cases + 1] = "8".into();
            restart.extend([
                "--toolchain".into(),
                "nightly-2026-09-01".into(),
                "--restart-corpus".into(),
                reduced_corpus.corpora[0].clone().into(),
            ]);
            let restarted = snapshot.invoke(restart);
            assert_eq!(
                restarted.status,
                CampaignStatus::Sufficient,
                "{restarted:?}"
            );
            assert_ne!(restarted.corpora, reduced_corpus.corpora);
            assert!(restarted.evaluated > 0 && restarted.evaluated <= 8);
            for (name, bytes) in originals {
                assert_eq!(fs::read(source_corpus.join(name)).unwrap(), bytes);
            }
            assert_eq!(
                store
                    .load_failure(&RelativePath::new(path).unwrap())
                    .unwrap()
                    .input(),
                entropy
            );
        }
    }
}

#[test]
#[ignore = "compiles an isolated instrumented owner and checks its exact evaluation limit"]
fn coverage_initial_corpus_cannot_exceed_the_requested_case_budget() {
    let snapshot = Snapshot::new();
    let input = snapshot.input(b"BUG!");
    let mut args = snapshot.args("run", "surgeist-fixture::bytes");
    let case = args.iter().position(|arg| arg == "--case-ms").unwrap();
    args[case + 1] = "1000".into();
    args.extend([
        "--toolchain".into(),
        "nightly-2026-09-01".into(),
        "--input".into(),
        input.as_os_str().to_owned(),
    ]);
    let summary = snapshot.invoke(args);
    assert!(summary.evaluated <= 1, "{summary:?}");
    assert_eq!(summary.status, CampaignStatus::Sufficient, "{summary:?}");
    assert_eq!(summary.cleanup, "complete");
}

#[test]
#[ignore = "compiles an isolated owner and exercises finite process termination"]
fn incomplete_owner_evaluations_preserve_inputs_without_inventing_property_keys() {
    let snapshot = Snapshot::new();
    for (input, status) in [
        (b"PANIC! exact case".as_slice(), CampaignStatus::Finding),
        (b"HANG! exact case".as_slice(), CampaignStatus::ResourceStop),
    ] {
        let path = snapshot.input(input);
        let mut args = snapshot.args("check", "surgeist-fixture::bytes");
        args.extend(["--input".into(), path.as_os_str().to_owned()]);
        let summary = snapshot.invoke(args);
        assert_eq!(summary.status, status, "{summary:?}");
        assert_eq!(summary.evaluated, 0);
        assert_eq!(summary.cleanup, "complete");
        assert!(!summary.artifacts.iter().any(|p| p.starts_with("failures/")));
        let captured = summary
            .artifacts
            .iter()
            .find(|p| p.starts_with("captures/"))
            .unwrap();
        let store = ArtifactStore::open(
            snapshot.work.join("artifacts"),
            StorageLimits::new(1024, 64 * 1024 * 1024, 4096, 1024 * 1024).unwrap(),
        )
        .unwrap();
        let original = store
            .load_capture(&RelativePath::new(captured).unwrap())
            .unwrap();
        assert_eq!(original.input(), input);
        let mut args = snapshot.args("replay", "surgeist-fixture::bytes");
        args.extend(["--artifact".into(), captured.into()]);
        let replayed = snapshot.invoke(args);
        assert_eq!(replayed.status, status, "{replayed:?}");
        assert_eq!(replayed.evaluated, 0);
        assert_eq!(replayed.cleanup, "complete");
        assert!(
            replayed
                .artifacts
                .iter()
                .any(|p| p.starts_with("capture-replays/"))
        );
        assert_eq!(
            store
                .load_capture(&RelativePath::new(captured).unwrap())
                .unwrap()
                .input(),
            original.input()
        );
    }
}

#[test]
#[ignore = "builds an isolated owner and verifies command selection and publication limits"]
fn regression_selection_parallel_counts_and_preflight_limits_are_observable() {
    let snapshot = Snapshot::new();
    let fixture = snapshot
        .source
        .join("crates/surgeist-fuzz/tests/fixtures/subject/fuzz/regressions");
    let empty = fixture.join("empty");
    let selected = fixture.join("selected");
    fs::create_dir(&empty).unwrap();
    fs::create_dir(&selected).unwrap();
    for (name, bytes) in [
        ("a.case", b"ordinary".as_slice()),
        ("b.case", b"BUG!".as_slice()),
        ("c.case", b"ALT!".as_slice()),
    ] {
        fs::write(selected.join(name), bytes).unwrap();
    }
    tool("git", &snapshot.source, &["add", "."]);
    tool(
        "git",
        &snapshot.source,
        &[
            "commit",
            "-q",
            "-m",
            "test(fuzz): record explicit regression selection",
        ],
    );
    let mut args = snapshot.args("check", "surgeist-fixture::bytes");
    args.extend(["--regressions".into(), empty.into_os_string()]);
    let result = snapshot.invoke(args);
    assert_eq!(result.status, CampaignStatus::Empty);
    assert_eq!(result.evaluated, 0);
    assert!(!snapshot.work.exists());
    let mut args = snapshot.args("check", "surgeist-fixture::bytes");
    args.extend([
        "--cargo".into(),
        snapshot.root.join("missing-cargo").into_os_string(),
    ]);
    assert!(run_cli(&args, &AtomicBool::new(false)).is_err());
    assert!(!snapshot.work.exists());
    fs::write(snapshot.source.join("uncommitted.txt"), b"dirty source").unwrap();
    assert!(
        run_cli(
            &snapshot.args("check", "surgeist-fixture::bytes"),
            &AtomicBool::new(false)
        )
        .is_err()
    );
    assert!(!snapshot.work.exists());
    fs::remove_file(snapshot.source.join("uncommitted.txt")).unwrap();
    let mut args = snapshot.args("check", "surgeist-fixture::bytes");
    let cases = args.iter().position(|arg| arg == "--cases").unwrap();
    args[cases + 1] = "3".into();
    args.extend(["--regressions".into(), selected.into_os_string()]);
    let result = snapshot.invoke(args);
    assert_eq!(result.status, CampaignStatus::Finding, "{result:?}");
    assert_eq!(result.evaluated, 3);
    assert_eq!(
        result
            .artifacts
            .iter()
            .filter(|p| p.starts_with("failures/"))
            .count(),
        2
    );
    let pass = snapshot.input(b"ordinary");
    let mut args = snapshot.args("check", "surgeist-fixture::bytes");
    let cases = args.iter().position(|arg| arg == "--cases").unwrap();
    args[cases + 1] = "4".into();
    args.extend([
        "--workers".into(),
        "2".into(),
        "--input".into(),
        pass.into_os_string(),
    ]);
    let result = snapshot.invoke(args);
    assert_eq!(result.status, CampaignStatus::Sufficient, "{result:?}");
    assert_eq!(result.evaluated, 4);
    assert_eq!(result.cleanup, "complete");
    let cancelled = run_cli(
        &snapshot.args("check", "surgeist-fixture::bytes"),
        &AtomicBool::new(true),
    )
    .unwrap();
    assert_eq!(cancelled.status, CampaignStatus::Cancelled);
    let input = snapshot.input(b"new BUG! finding");
    let mut args = snapshot.args("check", "surgeist-fixture::bytes");
    args.extend([
        "--input".into(),
        input.into_os_string(),
        "--storage-entries".into(),
        "1".into(),
    ]);
    assert_eq!(
        run_cli(&args, &AtomicBool::new(false)).unwrap_err().kind(),
        surgeist_fuzz::FuzzErrorKind::Resource
    );
}

#[test]
#[ignore = "resolves an isolated owner configuration before admitting campaign output"]
fn coverage_requires_a_resolved_engine_before_campaign_writes() {
    let snapshot = Snapshot::new();
    let mut args = snapshot.args("run", "surgeist-fixture::bytes");
    let index = args.iter().position(|arg| arg == "--case-ms").unwrap();
    args[index + 1] = "1000".into();
    args.extend([
        "--toolchain".into(),
        "nightly-2026-09-01".into(),
        "--coverage-feature".into(),
        "property".into(),
    ]);
    let result = run_cli(&args, &AtomicBool::new(false));
    assert!(
        !snapshot.work.exists(),
        "unsupported engine selection must fail before campaign writes"
    );
    let error = result.expect_err("property-only owner has no coverage engine");
    assert_eq!(
        error.kind(),
        surgeist_fuzz::FuzzErrorKind::MissingCapability
    );
}

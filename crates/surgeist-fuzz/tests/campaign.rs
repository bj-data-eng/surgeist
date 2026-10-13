#![forbid(unsafe_code)]
#![cfg(all(feature = "execution", target_os = "macos", target_arch = "aarch64"))]

use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
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
        let _ = fs::remove_dir_all(&self.root);
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
        assert_eq!(result.targets.len(), 2);
        assert!(result.builds.is_empty());
        assert!(result.runs.is_empty());
    }
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

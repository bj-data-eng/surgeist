#![forbid(unsafe_code)]

use super::*;
use crate::{
    BuildRequirements, Capability, CaseFormat, FailureClass, FailureKey, GenerationRejection,
    GeneratorIdentity, InputEncoding, InputLimits, PropertyId, TargetDescriptorSpec, TargetId,
};
use std::sync::atomic::AtomicU64;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let root = fs::canonicalize(std::env::temp_dir()).unwrap();
        let path = root.join(format!(
            "surgeist-coverage-observation-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

struct Fixture {
    options: Options,
    target: TargetDescriptor,
    build: BuildIdentity,
    config: RunConfig,
    store: ArtifactStore,
    cancellation: AtomicBool,
    directory: Directory,
}
impl Fixture {
    fn new(cases: u64, entries: u64) -> Self {
        let directory = Directory::new();
        let mut options = parse_cli(&["run".into()]).unwrap().command.options();
        options.input_bytes = 1024;
        options.cases = cases;
        let target = TargetDescriptor::new(TargetDescriptorSpec {
            id: TargetId::new("surgeist-test::coverage-observations").unwrap(),
            owner: "surgeist-test".into(),
            binary: "coverage-observations".into(),
            properties: vec![property("first"), property("second")],
            source_references: vec!["controlled observation fixture".into()],
            format: CaseFormat::new("surgeist-test::bytes", 1).unwrap(),
            generator_identity: GeneratorIdentity::new("surgeist-test::bytes-generator", 1)
                .unwrap(),
            encoding: InputEncoding::RawBytes,
            limits: InputLimits::new(1024, 0, 0, 0, 0).unwrap(),
            requirements: BuildRequirements::new(vec![], vec![Capability::Replay]).unwrap(),
            min_evaluated: 1,
            max_generation_rejections: 10_000,
        })
        .unwrap();
        let build = BuildIdentity::new(BuildIdentitySpec {
            source: SourceRevision::new("a".repeat(40)).unwrap(),
            lockfile: crate::Sha256Digest::from_bytes(b"fixture lock"),
            executable: crate::Sha256Digest::from_bytes(b"fixture executable"),
            toolchain: "rustc fixture".into(),
            engine: "fixture".into(),
            sanitizer: "none".into(),
            platform: "test-platform".into(),
            features: vec![],
            providers: vec![],
        })
        .unwrap();
        let config = RunConfig::new(RunConfigSpec {
            cases,
            wall_millis: 100,
            case_millis: 10,
            memory_bytes: 1024,
            seed: Some(12),
        })
        .unwrap();
        let store = ArtifactStore::open(
            &directory.0,
            StorageLimits::new(entries, 1024 * 1024, 1024, 8192).unwrap(),
        )
        .unwrap();
        Self {
            options,
            target,
            build,
            config,
            store,
            cancellation: AtomicBool::new(false),
            directory,
        }
    }
    fn context(&self) -> CampaignContext<'_> {
        CampaignContext {
            options: &self.options,
            target: &self.target,
            executable: Path::new("/unused-fixture-executable"),
            cwd: &self.directory.0,
            build: &self.build,
            config: &self.config,
            store: &self.store,
            cancellation: &self.cancellation,
            regression_files: None,
        }
    }
}

fn property(name: &str) -> PropertyId {
    PropertyId::new(format!("surgeist-test::{name}")).unwrap()
}
fn failure() -> CaseOutcome {
    CaseOutcome::failure(
        FailureKey::new(property("first"), FailureClass::PropertyViolation, None).unwrap(),
        "controlled owner failure",
    )
    .unwrap()
}
fn capture(observed: &mut Observations<'_>, bytes: &[u8]) {
    observed
        .accept(WorkerResponse::Captured {
            canonical: bytes.to_vec(),
        })
        .unwrap();
}
fn evaluated(outcome: CaseOutcome, evaluated: u64, rejected: u64) -> WorkerResponse {
    WorkerResponse::Evaluated {
        outcome,
        counts: ExecutionCounts::new(evaluated, rejected),
    }
}

#[test]
fn invalid_owner_outcome_preserves_exact_pending_case_and_prior_counts() {
    let fixture = Fixture::new(10, 8);
    let context = fixture.context();
    let mut observed = Observations::new(&context);
    let current = [0, 255, 0, 128];
    capture(&mut observed, &current);
    let error = observed
        .accept(evaluated(
            CaseOutcome::evaluated(vec![property("undeclared")]).unwrap(),
            1,
            0,
        ))
        .unwrap_err();
    assert_eq!(error.kind(), FuzzErrorKind::InvalidObservation);
    assert_eq!(observed.pending.as_deref(), Some(current.as_slice()));
    assert_eq!(observed.counts, ExecutionCounts::default());
    assert!(observed.checked.is_empty() && observed.failures.is_empty());
}

#[test]
fn invalid_evaluation_counters_preserve_current_case_without_acknowledgement() {
    let fixture = Fixture::new(10, 8);
    let context = fixture.context();
    for counts in [
        ExecutionCounts::new(0, 0),
        ExecutionCounts::new(2, 0),
        ExecutionCounts::new(1, 1),
    ] {
        let mut observed = Observations::new(&context);
        capture(&mut observed, &[0, 255, 1]);
        assert!(
            observed
                .accept(WorkerResponse::Evaluated {
                    outcome: CaseOutcome::evaluated(vec![property("first")]).unwrap(),
                    counts,
                })
                .is_err()
        );
        assert_eq!(observed.pending.as_deref(), Some([0, 255, 1].as_slice()));
        assert_eq!(observed.counts, ExecutionCounts::default());
        assert!(observed.checked.is_empty());
    }
}

#[test]
fn publication_quota_failure_preserves_current_case_and_existing_finding() {
    let fixture = Fixture::new(10, 1);
    let prior = CaseArtifact::new(
        fixture.target.clone(),
        fixture.build.clone(),
        fixture.config.clone(),
        b"prior finding".to_vec(),
        failure(),
    )
    .unwrap();
    let reference = fixture.store.publish_failure(&prior).unwrap();
    let context = fixture.context();
    let mut observed = Observations::new(&context);
    let current = [0, 255, 0, 128];
    capture(&mut observed, &current);
    let error = observed.accept(evaluated(failure(), 1, 0)).unwrap_err();
    assert_eq!(error.kind(), FuzzErrorKind::Resource);
    assert_eq!(observed.pending.as_deref(), Some(current.as_slice()));
    assert_eq!(observed.counts, ExecutionCounts::default());
    assert!(observed.failures.is_empty() && !observed.known_failure);
    assert_eq!(fixture.store.load_failure(&reference).unwrap(), prior);
    assert_eq!(
        fixture.store.recovery_report().unwrap().published,
        [reference]
    );
}

#[test]
fn thousands_of_successes_keep_exact_counts_properties_and_no_success_artifacts() {
    let fixture = Fixture::new(6000, 1);
    let before = fixture.store.recovery_report().unwrap();
    let context = fixture.context();
    let mut observed = Observations::new(&context);
    for index in 0..5000_u64 {
        capture(&mut observed, &index.to_be_bytes());
        observed
            .accept(evaluated(
                CaseOutcome::evaluated(vec![property(if index % 2 == 0 {
                    "first"
                } else {
                    "second"
                })])
                .unwrap(),
                index + 1,
                0,
            ))
            .unwrap();
        assert!(observed.pending.is_none());
    }
    for rejected in 1..=500 {
        observed
            .accept(WorkerResponse::GenerationRejected {
                rejection: GenerationRejection::new("bounded generation discard").unwrap(),
                counts: ExecutionCounts::new(5000, rejected),
            })
            .unwrap();
    }
    assert_eq!(observed.counts, ExecutionCounts::new(5000, 500));
    assert_eq!(
        observed.checked,
        BTreeSet::from([property("first"), property("second")])
    );
    assert!(observed.failures.is_empty() && !observed.known_failure);
    assert_eq!(fixture.store.recovery_report().unwrap(), before);
    for entry in fs::read_dir(&fixture.directory.0).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            assert_eq!(
                fs::read_dir(entry.path()).unwrap().count(),
                0,
                "success populated {}",
                entry.path().display()
            );
        }
    }
}

#[test]
fn malformed_or_truncated_following_frames_leave_current_capture_available() {
    let fixture = Fixture::new(10, 8);
    let context = fixture.context();
    for truncated in [false, true] {
        let mut observed = Observations::new(&context);
        let mut decoder = worker::ResponseDecoder::new(1024).unwrap();
        let current = vec![0, 255, 0, 128];
        let mut frames = Vec::new();
        worker::write_frame(
            &mut frames,
            &WorkerResponse::Captured {
                canonical: current.clone(),
            },
            1024,
        )
        .unwrap();
        if truncated {
            frames.extend_from_slice(&10_u32.to_be_bytes());
            frames.push(b'{');
            decoder
                .push(&frames, &mut |response| observed.accept(response))
                .unwrap();
            assert!(decoder.finish().is_err());
        } else {
            frames.extend_from_slice(&1_u32.to_be_bytes());
            frames.push(b'{');
            assert!(
                decoder
                    .push(&frames, &mut |response| observed.accept(response))
                    .is_err()
            );
        }
        assert_eq!(observed.pending, Some(current));
        assert_eq!(observed.counts, ExecutionCounts::default());
        assert!(observed.failures.is_empty());
    }
}

#[test]
fn round_attempt_budget_rejects_additional_evaluation_and_rejection() {
    let fixture = Fixture::new(1, 8);
    let context = fixture.context();
    let mut observed = Observations::new(&context);
    capture(&mut observed, b"first");
    observed
        .accept(evaluated(
            CaseOutcome::evaluated(vec![property("first")]).unwrap(),
            1,
            0,
        ))
        .unwrap();
    assert!(
        observed
            .accept(WorkerResponse::GenerationRejected {
                rejection: GenerationRejection::new("discard").unwrap(),
                counts: ExecutionCounts::new(1, 1),
            })
            .is_err()
    );
    assert_eq!(observed.counts, ExecutionCounts::new(1, 0));
    capture(&mut observed, b"over budget");
    assert!(
        observed
            .accept(evaluated(
                CaseOutcome::evaluated(vec![property("second")]).unwrap(),
                2,
                0
            ))
            .is_err()
    );
    assert_eq!(observed.pending.as_deref(), Some(b"over budget".as_slice()));
    assert_eq!(observed.counts, ExecutionCounts::new(1, 0));
    assert_eq!(observed.checked, BTreeSet::from([property("first")]));
}

#[test]
fn owner_finding_publishes_exact_binary_case_before_clearing_capture() {
    let fixture = Fixture::new(10, 8);
    let context = fixture.context();
    let mut observed = Observations::new(&context);
    let current = [0, 255, 0, 128, 0xd8, 0, 0xdc];
    capture(&mut observed, &current);
    observed.accept(evaluated(failure(), 1, 0)).unwrap();
    assert!(observed.pending.is_none() && observed.known_failure);
    assert_eq!(observed.counts, ExecutionCounts::new(1, 0));
    let [EvidenceStatus::Available { artifact }] = observed.failures.as_slice() else {
        panic!("finding did not retain one available artifact")
    };
    let loaded = fixture.store.load_failure(artifact).unwrap();
    assert_eq!(loaded.input(), current);
    assert_eq!(loaded.metadata().outcome(), &failure());
    assert_eq!(loaded.metadata().build(), &fixture.build);
    assert_eq!(loaded.metadata().config(), &fixture.config);
}

#[test]
fn capture_evidence_keeps_newest_diagnostic_tail_and_reports_discarded_bytes() {
    use crate::execution::{
        CapturedOutput, CleanupStatus, ProcessExit, ProcessOutcome, ProcessStop,
    };

    let terminal = b"terminal sanitizer message";
    let mut diagnostics = vec![b'o'; 32 * 1024];
    diagnostics.extend(vec![b'n'; 32 * 1024 - terminal.len()]);
    diagnostics.extend_from_slice(terminal);
    let process = ProcessOutcome {
        stop: ProcessStop::Cancelled,
        exit: ProcessExit::Signal(15),
        stdout: CapturedOutput {
            bytes: Vec::new(),
            truncated: false,
            eof: true,
        },
        stderr: CapturedOutput {
            bytes: diagnostics,
            truncated: false,
            eof: true,
        },
        input_bytes_written: 0,
        duration: Duration::from_millis(10),
        cleanup: CleanupStatus::Complete,
        process_group: 42,
        sampled_peak_rss_bytes: None,
    };
    let evidence = capture_evidence(&process).unwrap();
    assert_eq!(evidence.diagnostics().len(), 32 * 1024);
    assert!(evidence.diagnostics().as_bytes().ends_with(terminal));
    assert!(
        evidence.diagnostics().as_bytes()[..32 * 1024 - terminal.len()]
            .iter()
            .all(|byte| *byte == b'n')
    );
    assert!(evidence.diagnostics_truncated());
    assert_eq!(process.stderr.bytes.len(), 64 * 1024);
    assert!(!process.stderr.truncated);
}

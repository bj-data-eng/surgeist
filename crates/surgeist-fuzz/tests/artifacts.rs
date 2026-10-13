#![forbid(unsafe_code)]
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use surgeist_fuzz::{artifact::*, *};

static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        // /tmp is a symlink on macOS; callers resolve their root before opening.
        let parent = fs::canonicalize(std::env::temp_dir()).unwrap();
        let path = parent.join(format!(
            "surgeist-fuzz-artifact-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn descriptor() -> TargetDescriptor {
    TargetDescriptor::new(TargetDescriptorSpec {
        id: TargetId::new("surgeist-test::artifact-test").unwrap(),
        owner: "surgeist-test".into(),
        binary: "artifact-test".into(),
        properties: vec![PropertyId::new("surgeist-test::input-preserved").unwrap()],
        source_references: vec!["fixture codec".into()],
        format: CaseFormat::new("surgeist-test::bytes", 1).unwrap(),
        generator_identity: GeneratorIdentity::new("surgeist-test::bytes-generator", 1).unwrap(),
        encoding: InputEncoding::RawBytes,
        limits: InputLimits::new(1024, 0, 0, 0, 0).unwrap(),
        requirements: BuildRequirements::new(vec![], vec![Capability::Replay]).unwrap(),
        min_evaluated: 1,
        max_generation_rejections: 10,
    })
    .unwrap()
}
fn build(source: char) -> BuildIdentity {
    BuildIdentity::new(BuildIdentitySpec {
        source: SourceRevision::new(source.to_string().repeat(40)).unwrap(),
        lockfile: Sha256Digest::from_bytes(b"lock"),
        executable: Sha256Digest::from_bytes(source.to_string()),
        toolchain: "rustc fixture".into(),
        engine: "fixture".into(),
        sanitizer: "none".into(),
        platform: "test-platform".into(),
        features: vec![],
        providers: vec![],
    })
    .unwrap()
}
fn config() -> RunConfig {
    RunConfig::new(RunConfigSpec {
        cases: 10,
        wall_millis: 100,
        case_millis: 10,
        memory_bytes: 1024,
        seed: Some(12),
    })
    .unwrap()
}
fn artifact(input: &[u8]) -> CaseArtifact {
    CaseArtifact::new(
        descriptor(),
        build('a'),
        config(),
        input.to_vec(),
        CaseOutcome::failure(
            FailureKey::new(
                PropertyId::new("surgeist-test::input-preserved").unwrap(),
                FailureClass::PropertyViolation,
                None,
            )
            .unwrap(),
            "controlled failure",
        )
        .unwrap(),
    )
    .unwrap()
}
fn limits() -> StorageLimits {
    StorageLimits::new(100, 1024 * 1024, 1024, 8192).unwrap()
}

#[test]
fn exact_binary_bytes_publish_roundtrip_and_export_without_extra_files() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let original = artifact(&[0, 255, 0, 128, 0xd8, 0, 0xdc, 255]);
    let reference = store.publish_failure(&original).unwrap();
    assert_eq!(store.load_failure(&reference).unwrap(), original);
    assert_eq!(store.publish_failure(&original).unwrap(), reference);
    let export_dir = Directory::new();
    let destination = ArtifactStore::open(export_dir.path(), limits()).unwrap();
    let exported = store.export_failure(&reference, &destination).unwrap();
    assert_eq!(
        destination.load_failure(&exported).unwrap().input(),
        original.input()
    );
    let mut files = fs::read_dir(reference.join(export_dir.path()))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files, ["input.bin", "metadata.json", "published"]);
}

#[test]
fn schema_rejects_unknown_missing_duplicate_and_invalid_metadata() {
    let original = artifact(b"case");
    let json = serde_json::to_string(original.metadata()).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["extra"] = true.into();
    assert!(serde_json::from_value::<CaseMetadata>(value).is_err());
    let missing = json.replace("\"seed\":12,", "").replace(",\"seed\":12", "");
    assert!(serde_json::from_str::<CaseMetadata>(&missing).is_err());
    let duplicate = json.replacen("\"schema\":1", "\"schema\":1,\"schema\":1", 1);
    assert!(serde_json::from_str::<CaseMetadata>(&duplicate).is_err());
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["schema"] = 2.into();
    assert!(serde_json::from_value::<CaseMetadata>(value).is_err());
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["config"]["cases"] = 0.into();
    assert!(serde_json::from_value::<CaseMetadata>(value).is_err());
    assert!(CaseArtifact::from_parts(original.metadata().clone(), b"changed".to_vec()).is_err());
}

#[test]
fn interrupted_publication_is_reported_and_cannot_be_loaded_or_overwritten() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let original = artifact(b"current input");
    let reference = store.publish_failure(&original).unwrap();
    fs::remove_file(reference.join(directory.path()).join("published")).unwrap();
    let report = store.recovery_report().unwrap();
    assert_eq!(report.partial.as_slice(), std::slice::from_ref(&reference));
    assert!(report.published.is_empty());
    assert!(store.load_failure(&reference).is_err());
    assert!(store.publish_failure(&original).is_err());
    assert_eq!(
        fs::read(reference.join(directory.path()).join("input.bin")).unwrap(),
        original.input()
    );
}

#[test]
fn quota_never_discards_the_original_failure() {
    let directory = Directory::new();
    let store = ArtifactStore::open(
        directory.path(),
        StorageLimits::new(1, 16384, 1024, 8192).unwrap(),
    )
    .unwrap();
    let original = artifact(b"first");
    let reference = store.publish_failure(&original).unwrap();
    assert!(store.publish_failure(&artifact(b"second")).is_err());
    assert_eq!(store.load_failure(&reference).unwrap(), original);
    assert_eq!(store.publish_failure(&original).unwrap(), reference);
}

#[test]
fn changed_content_under_duplicate_identity_is_rejected() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let original = artifact(b"first");
    let reference = store.publish_failure(&original).unwrap();
    fs::write(reference.join(directory.path()).join("input.bin"), b"other").unwrap();
    assert!(store.publish_failure(&original).is_err());
    assert!(store.load_failure(&reference).is_err());
}

#[test]
fn rooted_storage_rejects_path_escape_and_symlink_components() {
    use std::os::unix::fs::symlink;
    let directory = Directory::new();
    let outside = Directory::new();
    symlink(outside.path(), directory.path().join("escape")).unwrap();
    assert!(ArtifactStore::open(directory.path().join("escape"), limits()).is_err());
    assert!(ArtifactStore::open(directory.path().join("../escape"), limits()).is_err());
    fs::remove_file(directory.path().join("escape")).unwrap();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    fs::remove_dir(directory.path().join("failures")).unwrap();
    symlink(outside.path(), directory.path().join("failures")).unwrap();
    assert!(store.publish_failure(&artifact(b"safe")).is_err());
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    assert!(RelativePath::new("../input.bin").is_err());
}

#[test]
fn corpus_deduplicates_bytes_with_compatible_generation_identity() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let original = artifact(b"corpus");
    let reference = store.add_corpus(&original).unwrap();
    let other_observation = CaseArtifact::new(
        descriptor(),
        build('b'),
        config(),
        b"corpus".to_vec(),
        CaseOutcome::evaluated(descriptor().properties().to_vec()).unwrap(),
    )
    .unwrap();
    assert_eq!(store.add_corpus(&other_observation).unwrap(), reference);
    assert_eq!(
        store.load_corpus(&reference, &descriptor()).unwrap(),
        b"corpus"
    );
    let mut spec: serde_json::Value = serde_json::to_value(descriptor()).unwrap();
    spec["generator_identity"]["revision"] = 2.into();
    let new_descriptor: TargetDescriptor = serde_json::from_value(spec).unwrap();
    assert!(store.load_corpus(&reference, &new_descriptor).is_err());
    let changed = CaseArtifact::new(
        new_descriptor,
        build('a'),
        config(),
        b"corpus".to_vec(),
        original.metadata().outcome().clone(),
    )
    .unwrap();
    assert_ne!(store.add_corpus(&changed).unwrap(), reference);
}

struct Target {
    descriptor: TargetDescriptor,
}
impl CaseTarget for Target {
    type Case = Vec<u8>;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        Ok(input.to_vec())
    }
    fn encode(&self, case: &Vec<u8>) -> Result<Vec<u8>> {
        Ok(case.clone())
    }
    fn generate(&self, _input: &[u8]) -> Result<Generation<Vec<u8>>> {
        panic!("exact replay must never generate")
    }
    fn evaluate(&self, case: &Vec<u8>) -> Result<CaseOutcome> {
        if case == b"fails" {
            CaseOutcome::failure(
                FailureKey::new(
                    self.descriptor.properties()[0].clone(),
                    FailureClass::PropertyViolation,
                    None,
                )?,
                "controlled failure",
            )
        } else {
            CaseOutcome::evaluated(self.descriptor.properties().to_vec())
        }
    }
}
#[test]
fn replay_uses_exact_materialized_bytes_and_requires_recorded_build() {
    let original = artifact(b"fails");
    let before = original.clone();
    let target = Target {
        descriptor: descriptor(),
    };
    let replay = replay_case(&target, &original, &build('a'), ReplayMode::Exact).unwrap();
    assert!(replay.observed().failure_key().is_some());
    assert!(!replay.is_comparison());
    assert!(replay_case(&target, &original, &build('b'), ReplayMode::Exact).is_err());
    let comparison = replay_case(&target, &original, &build('b'), ReplayMode::Comparison).unwrap();
    assert!(comparison.is_comparison());
    assert_eq!(original, before);
    let mut descriptor_json = serde_json::to_value(descriptor()).unwrap();
    descriptor_json["generator_identity"]["revision"] = 2.into();
    let revised_strategy = Target {
        descriptor: serde_json::from_value(descriptor_json).unwrap(),
    };
    assert!(replay_case(&revised_strategy, &original, &build('a'), ReplayMode::Exact).is_ok());
}

#[test]
fn replay_rejects_fixture_and_codec_mismatches_even_for_comparison() {
    let original = artifact(b"fails");
    let target = Target {
        descriptor: descriptor(),
    };
    let mut build_json = serde_json::to_value(build('b')).unwrap();
    build_json["providers"] = serde_json::json!([{ "role": "fixture", "name": "fixture", "digest": Sha256Digest::from_bytes(b"changed") }]);
    let changed_build: BuildIdentity = serde_json::from_value(build_json).unwrap();
    assert!(replay_case(&target, &original, &changed_build, ReplayMode::Comparison).is_err());
    let mut descriptor_json = serde_json::to_value(descriptor()).unwrap();
    descriptor_json["format"]["revision"] = 2.into();
    let changed_target = Target {
        descriptor: serde_json::from_value(descriptor_json).unwrap(),
    };
    assert!(
        replay_case(
            &changed_target,
            &original,
            &build('b'),
            ReplayMode::Comparison
        )
        .is_err()
    );
}

#[test]
fn run_observation_validates_counts_and_reports_unavailable_evidence() {
    let record = RunRecord::new(RunRecordSpec {
        schema: 1,
        target: descriptor(),
        build: build('a'),
        config: config(),
        counts: ExecutionCounts::new(2, 3),
        checked_properties: descriptor().properties().to_vec(),
        exclusions: vec!["browser unavailable".into()],
        failures: vec![EvidenceStatus::Unavailable {
            reason: EvidenceUnavailable::InterruptedCapture,
        }],
        cleanup: "worker reaped".into(),
    })
    .unwrap();
    let report = record.readable_report();
    assert!(report.contains("Executed: 2"));
    assert!(report.contains("Generation rejected: 3"));
    assert!(report.contains("InterruptedCapture"));
    assert!(report.contains("worker reaped"));
    let mut json = serde_json::to_value(record).unwrap();
    json["counts"]["evaluated"] = 20.into();
    assert!(serde_json::from_value::<RunRecord>(json).is_err());
}

#[test]
fn metadata_and_byte_quotas_bound_decoding_and_failed_publication() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let reference = store.publish_failure(&artifact(b"case")).unwrap();
    let metadata = reference.join(directory.path()).join("metadata.json");
    fs::write(metadata, vec![b' '; 8193]).unwrap();
    assert!(store.load_failure(&reference).is_err());
    assert!(store.recovery_report().is_err());
    let other = Directory::new();
    let store = ArtifactStore::open(
        other.path(),
        StorageLimits::new(3, 8192, 1024, 128).unwrap(),
    )
    .unwrap();
    assert!(store.publish_failure(&artifact(b"case")).is_err());
    assert!(store.recovery_report().unwrap().published.is_empty());
    let other = Directory::new();
    let store = ArtifactStore::open(
        other.path(),
        StorageLimits::new(3, 9000, 1024, 8192).unwrap(),
    )
    .unwrap();
    store.publish_failure(&artifact(b"first")).unwrap();
    // Account for the first exact metadata size, then choose a root budget unable
    // to hold both bundles. No count quota causes this rejection.
    let occupied = store.recovery_report().unwrap().bytes;
    let tight = ArtifactStore::open(
        other.path(),
        StorageLimits::new(3, occupied + 1, 1024, occupied.min(8192)).unwrap(),
    )
    .unwrap();
    assert!(tight.publish_failure(&artifact(b"second")).is_err());
    assert_eq!(tight.recovery_report().unwrap().published.len(), 1);
}

#[test]
fn comparison_is_a_new_validated_record_and_original_failure_is_immutable() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let original = artifact(b"fails");
    let original_ref = store.publish_failure(&original).unwrap();
    let original_metadata =
        fs::read(original_ref.join(directory.path()).join("metadata.json")).unwrap();
    let comparison = replay_case(
        &Target {
            descriptor: descriptor(),
        },
        &original,
        &build('b'),
        ReplayMode::Comparison,
    )
    .unwrap();
    let comparison_ref = store.publish_replay(&comparison).unwrap();
    assert_eq!(store.load_replay(&comparison_ref).unwrap(), comparison);
    assert!(comparison_ref.as_str().starts_with("comparisons/"));
    assert_eq!(
        fs::read(original_ref.join(directory.path()).join("metadata.json")).unwrap(),
        original_metadata
    );
    assert_eq!(store.load_failure(&original_ref).unwrap(), original);
    let mut json = serde_json::to_value(comparison).unwrap();
    json["comparison"] = false.into();
    assert!(serde_json::from_value::<ReplayRecord>(json).is_err());
}

// This test owner codec deliberately encodes UTF-16 code units directly. Storage
// must never route this construction data through a Rust String or lossy JSON.
struct Utf16Target {
    descriptor: TargetDescriptor,
}
impl CaseTarget for Utf16Target {
    type Case = Vec<u16>;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, bytes: &[u8]) -> Result<Self::Case> {
        if bytes.len() < 4
            || bytes.len() > 1024
            || !bytes.len().is_multiple_of(2)
            || bytes[..4] != [1, 0, 3, 0]
        {
            return Err(FuzzError::new(
                FuzzErrorKind::Codec,
                "UTF-16 fixture",
                "wrong context or length",
            ));
        }
        Ok(bytes[4..]
            .chunks_exact(2)
            .map(|p| u16::from_le_bytes([p[0], p[1]]))
            .collect())
    }
    fn encode(&self, units: &Self::Case) -> Result<Vec<u8>> {
        let mut bytes = vec![1, 0, 3, 0];
        for unit in units {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        Ok(bytes)
    }
    fn generate(&self, _bytes: &[u8]) -> Result<Generation<Self::Case>> {
        panic!("replay cannot generate")
    }
    fn evaluate(&self, units: &Self::Case) -> Result<CaseOutcome> {
        assert_eq!(units, &[0xd800, 0x0061, 0xdcff]);
        CaseOutcome::failure(
            FailureKey::new(
                self.descriptor.properties()[0].clone(),
                FailureClass::PropertyViolation,
                None,
            )?,
            "unpaired fixture",
        )
    }
}
#[test]
fn structured_owner_codec_preserves_unpaired_utf16_context_and_history_bytes() {
    let mut spec = serde_json::to_value(descriptor()).unwrap();
    spec["encoding"] = "structured".into();
    spec["limits"]["text_units"] = 3.into();
    spec["limits"]["operations"] = 1.into();
    let target = Utf16Target {
        descriptor: serde_json::from_value(spec).unwrap(),
    };
    let input = target.encode(&vec![0xd800, 0x0061, 0xdcff]).unwrap();
    let case = CaseArtifact::new(
        target.descriptor.clone(),
        build('a'),
        config(),
        input.clone(),
        target.evaluate(&vec![0xd800, 0x0061, 0xdcff]).unwrap(),
    )
    .unwrap();
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let reference = store.publish_failure(&case).unwrap();
    let loaded = store.load_failure(&reference).unwrap();
    assert_eq!(loaded.input(), input);
    assert_eq!(
        target.decode(loaded.input()).unwrap(),
        [0xd800, 0x0061, 0xdcff]
    );
    assert!(
        replay_case(&target, &loaded, &build('a'), ReplayMode::Exact)
            .unwrap()
            .observed()
            .failure_key()
            .is_some()
    );
}

#[test]
fn publication_marker_cannot_claim_missing_or_symlinked_evidence() {
    use std::os::unix::fs::symlink;
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let reference = store.publish_failure(&artifact(b"case")).unwrap();
    let input = reference.join(directory.path()).join("input.bin");
    fs::remove_file(&input).unwrap();
    assert!(store.recovery_report().is_err());
    let outside = Directory::new();
    let outside_file = outside.path().join("input.bin");
    fs::write(&outside_file, b"case").unwrap();
    symlink(outside_file, input).unwrap();
    assert!(store.load_failure(&reference).is_err());
    assert!(store.recovery_report().is_err());
}

fn process(stop: ProcessStopReason, exit: ProcessExitStatus) -> ProcessEvidence {
    ProcessEvidence::new(ProcessEvidenceSpec {
        stop,
        exit,
        diagnostics: "worker diagnostics\n".into(),
        diagnostics_truncated: false,
        cleanup: ProcessCleanup::Complete,
    })
    .unwrap()
}
#[test]
fn abrupt_process_capture_has_exact_input_and_no_invented_semantic_failure_key() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    for evidence in [
        process(
            ProcessStopReason::Exited,
            ProcessExitStatus::Code { code: 101 },
        ),
        process(
            ProcessStopReason::Exited,
            ProcessExitStatus::Signal { signal: 6 },
        ),
        process(
            ProcessStopReason::Deadline,
            ProcessExitStatus::Signal { signal: 15 },
        ),
        process(ProcessStopReason::Cancelled, ProcessExitStatus::Unavailable),
    ] {
        let original = CaptureArtifact::new(
            descriptor(),
            build('a'),
            config(),
            vec![0, 255, 128, 0],
            evidence.clone(),
        )
        .unwrap();
        let reference = store.publish_capture(&original).unwrap();
        let loaded = store.load_capture(&reference).unwrap();
        assert_eq!(loaded, original);
        assert_eq!(loaded.metadata().process(), &evidence);
        let json = serde_json::to_value(loaded.metadata()).unwrap();
        assert!(json.get("outcome").is_none());
        assert!(json.get("failure_key").is_none());
        assert_eq!(store.publish_capture(&original).unwrap(), reference);
        assert!(
            original
                .validate_replay(&descriptor(), &build('b'), ReplayMode::Exact)
                .is_err()
        );
        let comparison = replay_capture_observation(
            &original,
            &descriptor(),
            &build('b'),
            ReplayMode::Comparison,
            evidence,
        )
        .unwrap();
        let comparison_ref = store.publish_capture_replay(&comparison).unwrap();
        assert_eq!(
            store.load_capture_replay(&comparison_ref).unwrap(),
            comparison
        );
        assert!(comparison.is_comparison());
        assert_eq!(store.load_capture(&reference).unwrap(), original);
    }
}
#[test]
fn capture_schemas_reject_inconsistent_exit_missing_fields_and_corrupt_input() {
    let evidence = process(ProcessStopReason::Deadline, ProcessExitStatus::Unavailable);
    let original = CaptureArtifact::new(
        descriptor(),
        build('a'),
        config(),
        b"capture".to_vec(),
        evidence.clone(),
    )
    .unwrap();
    let mut json = serde_json::to_value(original.metadata()).unwrap();
    json["process"]["exit"]["kind"] = "invented-panic".into();
    assert!(serde_json::from_value::<CaptureMetadata>(json).is_err());
    let mut json = serde_json::to_value(original.metadata()).unwrap();
    json["process"].as_object_mut().unwrap().remove("cleanup");
    assert!(serde_json::from_value::<CaptureMetadata>(json).is_err());
    let mut json = serde_json::to_value(evidence).unwrap();
    json["stop"]["kind"] = "exited".into();
    assert!(serde_json::from_value::<ProcessEvidence>(json).is_err());
    assert!(
        CaptureArtifact::from_parts(original.metadata().clone(), b"not captured".to_vec()).is_err()
    );
}
#[cfg(feature = "execution")]
#[test]
fn supervisor_mapping_preserves_deadline_signal_truncation_and_cleanup_facts() {
    use std::time::Duration;
    use surgeist_fuzz::execution::*;
    let observation = ProcessOutcome {
        stop: ProcessStop::Deadline,
        exit: ProcessExit::Signal(15),
        stdout: CapturedOutput {
            bytes: vec![],
            truncated: false,
            eof: true,
        },
        stderr: CapturedOutput {
            bytes: b"actual stderr\n".to_vec(),
            truncated: true,
            eof: false,
        },
        input_bytes_written: 4,
        duration: Duration::from_millis(10),
        cleanup: CleanupStatus::Failed {
            child_reaped: true,
            group_gone: false,
            errors: vec!["group still exists".into()],
        },
        process_group: 123,
        sampled_peak_rss_bytes: None,
    };
    let evidence = ProcessEvidence::from_outcome(&observation).unwrap();
    assert_eq!(evidence.stop(), &ProcessStopReason::Deadline);
    assert_eq!(evidence.exit(), ProcessExitStatus::Signal { signal: 15 });
    assert!(evidence.diagnostics_truncated());
    assert_eq!(evidence.diagnostics(), "actual stderr\n");
    assert_eq!(
        evidence.cleanup(),
        &ProcessCleanup::Incomplete {
            child_reaped: true,
            group_gone: false,
            errors: vec!["group still exists".into()]
        }
    );
}

#[test]
fn corpus_compatibility_ignores_campaign_thresholds_and_requires_exact_fixtures() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let original = artifact(b"corpus");
    let reference = store.add_corpus(&original).unwrap();
    let mut json = serde_json::to_value(descriptor()).unwrap();
    json["min_evaluated"] = 5.into();
    json["source_references"] = serde_json::json!(["updated evidence reference"]);
    let threshold: TargetDescriptor = serde_json::from_value(json).unwrap();
    let compatible = CaseArtifact::new(
        threshold.clone(),
        build('a'),
        config(),
        b"corpus".to_vec(),
        original.metadata().outcome().clone(),
    )
    .unwrap();
    assert_eq!(store.add_corpus(&compatible).unwrap(), reference);
    assert_eq!(
        store.load_corpus(&reference, &threshold).unwrap(),
        b"corpus"
    );
    let mut json = serde_json::to_value(build('a')).unwrap();
    json["providers"] = serde_json::json!([{ "role": "fixture", "name": "fixture", "digest": Sha256Digest::from_bytes(b"fixture") }]);
    let fixture_build: BuildIdentity = serde_json::from_value(json).unwrap();
    let fixture = CaseArtifact::new(
        descriptor(),
        fixture_build.clone(),
        config(),
        b"corpus".to_vec(),
        original.metadata().outcome().clone(),
    )
    .unwrap();
    let fixture_ref = store.add_corpus(&fixture).unwrap();
    assert_ne!(fixture_ref, reference);
    assert!(store.load_corpus(&fixture_ref, &descriptor()).is_err());
    assert_eq!(
        store
            .load_corpus_for_build(&fixture_ref, &descriptor(), &fixture_build)
            .unwrap(),
        b"corpus"
    );
}
#[test]
fn opening_seed_directory_does_not_mutate_its_files_or_create_campaign_namespaces() {
    let directory = Directory::new();
    fs::write(directory.path().join("seed.bin"), b"checked-in seed").unwrap();
    assert!(ArtifactStore::open(directory.path(), limits()).is_err());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    assert_eq!(
        fs::read(directory.path().join("seed.bin")).unwrap(),
        b"checked-in seed"
    );
}

#[test]
fn run_publication_links_only_available_evidence_from_its_recorded_build() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let failure = store.publish_failure(&artifact(b"failure")).unwrap();
    let config = config();
    let spec = |build| RunRecordSpec {
        schema: 1,
        target: descriptor(),
        build,
        config: config.clone(),
        counts: ExecutionCounts::new(1, 0),
        checked_properties: descriptor().properties().to_vec(),
        exclusions: vec![],
        failures: vec![EvidenceStatus::Available {
            artifact: failure.clone(),
        }],
        cleanup: "complete".into(),
    };
    let run = RunRecord::new(spec(build('a'))).unwrap();
    let reference = store.publish_run(&run).unwrap();
    assert_eq!(store.load_run(&reference).unwrap(), run);
    assert!(
        store
            .publish_run(&RunRecord::new(spec(build('b'))).unwrap())
            .is_err()
    );
}

#[test]
fn closed_process_unit_tags_reject_unknown_fields() {
    for tag in ["exited", "deadline", "cancelled"] {
        let value = serde_json::json!({"kind": tag, "ignored": true});
        assert!(
            serde_json::from_value::<ProcessStopReason>(value).is_err(),
            "accepted unknown field for {tag}"
        );
    }
    assert!(
        serde_json::from_value::<ProcessExitStatus>(
            serde_json::json!({"kind": "unavailable", "ignored": true})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<ProcessCleanup>(
            serde_json::json!({"state": "complete", "ignored": true})
        )
        .is_err()
    );
}

#[cfg(feature = "execution")]
#[test]
fn lossy_diagnostics_are_bounded_after_utf8_encoding_and_capture_is_retained() {
    use std::time::Duration;
    use surgeist_fuzz::execution::*;
    let observation = ProcessOutcome {
        stop: ProcessStop::Deadline,
        exit: ProcessExit::Signal(15),
        stdout: CapturedOutput {
            bytes: vec![],
            truncated: false,
            eof: true,
        },
        stderr: CapturedOutput {
            bytes: vec![0xff; 32 * 1024],
            truncated: false,
            eof: true,
        },
        input_bytes_written: 4,
        duration: Duration::from_millis(10),
        cleanup: CleanupStatus::Complete,
        process_group: 123,
        sampled_peak_rss_bytes: None,
    };
    let evidence = ProcessEvidence::from_outcome(&observation)
        .expect("bounded unreadable diagnostics must not discard current case");
    assert!(evidence.diagnostics().len() <= 64 * 1024);
    assert!(evidence.diagnostics_truncated());
    assert!(evidence.diagnostics().chars().all(|c| c == '\u{fffd}'));
    let capture =
        CaptureArtifact::new(descriptor(), build('a'), config(), vec![0, 255], evidence).unwrap();
    let directory = Directory::new();
    let store = ArtifactStore::open(
        directory.path(),
        StorageLimits::new(100, 1024 * 1024, 1024, 128 * 1024).unwrap(),
    )
    .unwrap();
    let reference = store.publish_capture(&capture).unwrap();
    assert_eq!(store.load_capture(&reference).unwrap(), capture);
}

fn role_build(source: char, tool: &[u8], engine: &[u8], fixture: &[u8]) -> BuildIdentity {
    let mut spec: BuildIdentitySpec =
        serde_json::from_value(serde_json::to_value(build(source)).unwrap()).unwrap();
    spec.providers = vec![
        ContentIdentity::new(
            ContentRole::BuildTool,
            "compiler",
            Sha256Digest::from_bytes(tool),
        )
        .unwrap(),
        ContentIdentity::new(
            ContentRole::Engine,
            "coverage-driver",
            Sha256Digest::from_bytes(engine),
        )
        .unwrap(),
        ContentIdentity::new(
            ContentRole::Fixture,
            "fixture",
            Sha256Digest::from_bytes(fixture),
        )
        .unwrap(),
    ];
    BuildIdentity::new(spec).unwrap()
}
#[test]
fn comparison_allows_changed_tools_and_engines_but_requires_identical_fixtures() {
    let original = CaseArtifact::new(
        descriptor(),
        role_build('a', b"old tool", b"old engine", b"same fixture"),
        config(),
        b"fails".to_vec(),
        artifact(b"fails").metadata().outcome().clone(),
    )
    .unwrap();
    let selected = role_build('b', b"new tool", b"new engine", b"same fixture");
    let target = Target {
        descriptor: descriptor(),
    };
    assert!(
        original
            .validate_replay(&descriptor(), &selected, ReplayMode::Exact)
            .is_err()
    );
    let comparison = replay_case(&target, &original, &selected, ReplayMode::Comparison).unwrap();
    assert!(comparison.is_comparison());
    assert_eq!(
        serde_json::from_value::<ReplayRecord>(serde_json::to_value(&comparison).unwrap()).unwrap(),
        comparison
    );
    let changed_fixture = role_build('b', b"new tool", b"new engine", b"changed fixture");
    assert!(
        original
            .validate_replay(&descriptor(), &changed_fixture, ReplayMode::Comparison)
            .is_err()
    );
    let mut record = serde_json::to_value(comparison).unwrap();
    record["selected_build"] = serde_json::to_value(changed_fixture).unwrap();
    assert!(serde_json::from_value::<ReplayRecord>(record).is_err());
    let capture = CaptureArtifact::new(
        descriptor(),
        original.metadata().build().clone(),
        config(),
        b"fails".to_vec(),
        process(ProcessStopReason::Deadline, ProcessExitStatus::Unavailable),
    )
    .unwrap();
    let observed = process(
        ProcessStopReason::Exited,
        ProcessExitStatus::Code { code: 101 },
    );
    assert!(
        replay_capture_observation(
            &capture,
            &descriptor(),
            &selected,
            ReplayMode::Comparison,
            observed.clone()
        )
        .is_ok()
    );
    assert!(
        replay_capture_observation(
            &capture,
            &descriptor(),
            &selected,
            ReplayMode::Exact,
            observed
        )
        .is_err()
    );
}
#[test]
fn corpus_identity_tracks_fixture_contents_but_not_build_tools_or_execution_engines() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let original = CaseArtifact::new(
        descriptor(),
        role_build('a', b"old tool", b"old engine", b"same fixture"),
        config(),
        b"corpus".to_vec(),
        artifact(b"corpus").metadata().outcome().clone(),
    )
    .unwrap();
    let reference = store.add_corpus(&original).unwrap();
    let selected = role_build('b', b"new tool", b"new engine", b"same fixture");
    let other = CaseArtifact::new(
        descriptor(),
        selected.clone(),
        config(),
        b"corpus".to_vec(),
        original.metadata().outcome().clone(),
    )
    .unwrap();
    assert_eq!(store.add_corpus(&other).unwrap(), reference);
    assert_eq!(
        store
            .load_corpus_for_build(&reference, &descriptor(), &selected)
            .unwrap(),
        b"corpus"
    );
    let changed_fixture = role_build('b', b"new tool", b"new engine", b"changed fixture");
    assert!(
        store
            .load_corpus_for_build(&reference, &descriptor(), &changed_fixture)
            .is_err()
    );
}
#[test]
fn content_role_is_mandatory_and_closed_in_persisted_identity() {
    let content = ContentIdentity::new(
        ContentRole::BuildTool,
        "compiler",
        Sha256Digest::from_bytes(b"compiler"),
    )
    .unwrap();
    assert_eq!(content.role(), ContentRole::BuildTool);
    let json = serde_json::to_value(&content).unwrap();
    assert_eq!(json["role"], "build-tool");
    assert_eq!(
        serde_json::from_value::<ContentIdentity>(json.clone()).unwrap(),
        content
    );
    let mut missing = json.clone();
    missing.as_object_mut().unwrap().remove("role");
    assert!(serde_json::from_value::<ContentIdentity>(missing).is_err());
    let mut unknown = json;
    unknown["role"] = "inferred-fixture".into();
    assert!(serde_json::from_value::<ContentIdentity>(unknown).is_err());
}

struct ReductionTarget {
    descriptor: TargetDescriptor,
}
impl CaseTarget for ReductionTarget {
    type Case = Vec<u8>;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<Self::Case> {
        Ok(input.to_vec())
    }
    fn encode(&self, case: &Self::Case) -> Result<Vec<u8>> {
        Ok(case.clone())
    }
    fn generate(&self, _: &[u8]) -> Result<Generation<Self::Case>> {
        panic!("reduction does not generate")
    }
    fn evaluate(&self, input: &Self::Case) -> Result<CaseOutcome> {
        if input.contains(&b'x') {
            CaseOutcome::failure(
                FailureKey::new(
                    self.descriptor.properties()[0].clone(),
                    FailureClass::PropertyViolation,
                    None,
                )?,
                "selected failure",
            )
        } else {
            CaseOutcome::evaluated(self.descriptor.properties().to_vec())
        }
    }
}
fn reduction_report(input: &[u8]) -> surgeist_fuzz::reduction::ReductionReport {
    use surgeist_fuzz::reduction::*;
    let target = ReductionTarget {
        descriptor: descriptor(),
    };
    let limits = ReductionLimits::new(128, std::time::Duration::from_secs(5), 65536, 2).unwrap();
    reduce_direct(
        &target,
        input,
        artifact(input).metadata().outcome().failure_key().unwrap(),
        limits,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap()
}
#[test]
fn reduction_publication_preserves_links_actual_budgets_and_confirmed_chain() {
    let directory = Directory::new();
    let store = ArtifactStore::open(
        directory.path(),
        StorageLimits::new(100, 1024 * 1024, 1024, 128 * 1024).unwrap(),
    )
    .unwrap();
    let original = artifact(b"prefixx");
    let original_ref = store.publish_failure(&original).unwrap();
    let original_metadata =
        fs::read(original_ref.join(directory.path()).join("metadata.json")).unwrap();
    let report = reduction_report(original.input());
    assert_eq!(report.best_input(), b"x");
    assert!(!report.chain().is_empty());
    let selected_config = RunConfig::new(RunConfigSpec {
        cases: 20,
        wall_millis: 200,
        case_millis: 20,
        memory_bytes: 2048,
        seed: None,
    })
    .unwrap();
    let reduced = CaseArtifact::new(
        descriptor(),
        build('a'),
        selected_config.clone(),
        report.best_input().to_vec(),
        original.metadata().outcome().clone(),
    )
    .unwrap();
    let reduced_ref = store.publish_failure(&reduced).unwrap();
    let reference = store
        .publish_reduction(&original_ref, Some(&reduced_ref), &report)
        .unwrap();
    let loaded = store.load_reduction(&reference).unwrap();
    assert_eq!(loaded.original(), &original_ref);
    assert_eq!(loaded.reduced(), Some(&reduced_ref));
    assert_eq!(loaded.original_config(), original.metadata().config());
    assert_eq!(loaded.selected_config(), Some(&selected_config));
    assert_eq!(loaded.limits().max_attempts(), 128);
    assert_eq!(loaded.limits().confirmations(), 2);
    assert_eq!(
        loaded.limits().duration(),
        std::time::Duration::from_secs(5)
    );
    assert_eq!(loaded.replay_count(), report.replay_count());
    assert!(loaded.original_confirmed());
    assert!(loaded.final_confirmed());
    assert_eq!(
        loaded.chain().last().unwrap().to(),
        &Sha256Digest::from_bytes(b"x")
    );
    assert_eq!(loaded.observations().len(), report.observations().len());
    assert_eq!(
        store
            .publish_reduction(&original_ref, Some(&reduced_ref), &report)
            .unwrap(),
        reference
    );
    assert_eq!(
        fs::read(original_ref.join(directory.path()).join("metadata.json")).unwrap(),
        original_metadata
    );
    let metadata: serde_json::Value = serde_json::from_slice(
        &fs::read(reference.join(directory.path()).join("metadata.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(metadata["proposal_output_bytes"], 0);
    let mut proposal_metadata = metadata.clone();
    proposal_metadata["proposal_output_bytes"] = 128.into();
    assert!(serde_json::from_value::<ReductionRecord>(proposal_metadata.clone()).is_err());
    proposal_metadata["output_bytes"] = (metadata["output_bytes"].as_u64().unwrap() + 128).into();
    assert!(serde_json::from_value::<ReductionRecord>(proposal_metadata).is_ok());
    let mut missing_proposal_count = metadata.clone();
    missing_proposal_count
        .as_object_mut()
        .unwrap()
        .remove("proposal_output_bytes");
    assert!(serde_json::from_value::<ReductionRecord>(missing_proposal_count).is_err());
    assert!(metadata.get("original_input").is_none());
    assert!(metadata.get("best_input").is_none());
    let mut malformed = metadata.clone();
    malformed["chain"][0]["from"] =
        serde_json::to_value(Sha256Digest::from_bytes(b"broken")).unwrap();
    assert!(serde_json::from_value::<ReductionRecord>(malformed).is_err());
    let mut malformed = metadata.clone();
    malformed["limits"]["duration_nanoseconds"] = 1_000_000_000.into();
    assert!(serde_json::from_value::<ReductionRecord>(malformed).is_err());
    let mut malformed = metadata;
    malformed["status"] = "unknown".into();
    assert!(serde_json::from_value::<ReductionRecord>(malformed).is_err());
}
#[test]
fn reduction_publication_rejects_wrong_case_links_and_retains_original_on_quota_failure() {
    let directory = Directory::new();
    let store = ArtifactStore::open(
        directory.path(),
        StorageLimits::new(1, 1024 * 1024, 1024, 128 * 1024).unwrap(),
    )
    .unwrap();
    let original = artifact(b"prefixx");
    let reference = store.publish_failure(&original).unwrap();
    let report = reduction_report(original.input());
    assert!(store.publish_reduction(&reference, None, &report).is_err());
    assert_eq!(store.load_failure(&reference).unwrap(), original);
    let directory = Directory::new();
    let store = ArtifactStore::open(
        directory.path(),
        StorageLimits::new(100, 1024 * 1024, 1024, 128 * 1024).unwrap(),
    )
    .unwrap();
    let original_ref = store.publish_failure(&original).unwrap();
    let wrong = store.publish_failure(&artifact(b"wrongx")).unwrap();
    assert!(store.publish_reduction(&wrong, None, &report).is_err());
    assert!(
        store
            .publish_reduction(&original_ref, Some(&wrong), &report)
            .is_err()
    );
    let record_ref = store
        .publish_reduction(&original_ref, None, &report)
        .unwrap();
    assert!(
        store
            .load_reduction(&record_ref)
            .unwrap()
            .reduced()
            .is_none()
    );
}

#[test]
fn publication_waits_for_another_writer_then_preserves_both_cases() {
    use rustix::fs::{FlockOperation, flock};
    use std::{sync::mpsc, time::Duration};
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let first = store.publish_failure(&artifact(b"first")).unwrap();
    let lock = fs::File::open(directory.path().join("publication.lock")).unwrap();
    flock(&lock, FlockOperation::LockExclusive).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::scope(|scope| {
        scope.spawn(|| {
            started_tx.send(()).unwrap();
            done_tx
                .send(store.publish_failure(&artifact(b"second")))
                .unwrap();
        });
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        // Lock contention is the timed behavior: a writer must wait for the
        // existing publication, not report partial I/O while the lock is held.
        let early = done_rx.recv_timeout(Duration::from_millis(100));
        drop(lock);
        let second = match early {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                done_rx.recv_timeout(Duration::from_secs(5)).unwrap()
            }
            Err(error) => panic!("writer disconnected: {error}"),
        }
        .unwrap();
        assert_eq!(store.load_failure(&first).unwrap().input(), b"first");
        assert_eq!(store.load_failure(&second).unwrap().input(), b"second");
        assert!(store.recovery_report().unwrap().partial.is_empty());
    });
}

#[test]
fn smaller_store_quotas_report_resource_limits_and_preserve_evidence() {
    let directory = Directory::new();
    let store = ArtifactStore::open(directory.path(), limits()).unwrap();
    let first = store.publish_failure(&artifact(b"original")).unwrap();
    store.publish_failure(&artifact(b"another")).unwrap();
    for limit in [
        StorageLimits::new(1, 1024 * 1024, 1024, 8192).unwrap(),
        StorageLimits::new(100, 64, 32, 64).unwrap(),
    ] {
        let error = match ArtifactStore::open(directory.path(), limit) {
            Ok(_) => panic!("smaller quota admitted existing evidence"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), FuzzErrorKind::Resource, "{error}");
    }
    assert_eq!(store.load_failure(&first).unwrap().input(), b"original");
    assert_eq!(store.recovery_report().unwrap().published.len(), 2);
}

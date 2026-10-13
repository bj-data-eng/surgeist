#![forbid(unsafe_code)]

use std::{cell::Cell, io::Cursor};
use surgeist_fuzz::engine::{GeneratedEvaluation, evaluate_generated, replay};
use surgeist_fuzz::worker::{
    WorkerLimits, WorkerRequest, WorkerResponse, read_frame, serve_io, write_frame,
};
use surgeist_fuzz::{
    BuildRequirements, CaseFormat, CaseOutcome, CaseTarget, ExecutionCounts, FailureClass,
    FailureKey, FuzzError, FuzzErrorKind, Generation, GenerationRejection, GeneratorIdentity,
    InputEncoding, InputLimits, PropertyId, Result, TargetDescriptor, TargetDescriptorSpec,
    TargetId,
};

fn property() -> PropertyId {
    PropertyId::new("surgeist-test::even-value").unwrap()
}
fn key(discriminator: &str) -> FailureKey {
    FailureKey::new(
        property(),
        FailureClass::PropertyViolation,
        Some(discriminator.into()),
    )
    .unwrap()
}

struct Target {
    descriptor: TargetDescriptor,
    evaluated: Cell<u64>,
}
impl Target {
    fn new() -> Self {
        Self {
            descriptor: TargetDescriptor::new(TargetDescriptorSpec {
                id: TargetId::new("surgeist-test::byte").unwrap(),
                owner: "surgeist-test".into(),
                binary: "byte-worker".into(),
                properties: vec![property()],
                source_references: vec!["https://example.test/even-value".into()],
                format: CaseFormat::new("surgeist-test::byte", 1).unwrap(),
                generator_identity: GeneratorIdentity::new("surgeist-test::byte-generation", 1)
                    .unwrap(),
                encoding: InputEncoding::RawBytes,
                limits: InputLimits::new(16, 0, 0, 0, 0).unwrap(),
                requirements: BuildRequirements::new(vec![], vec![]).unwrap(),
                min_evaluated: 2,
                max_generation_rejections: 1,
            })
            .unwrap(),
            evaluated: Cell::new(0),
        }
    }
}
impl CaseTarget for Target {
    type Case = u8;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<u8> {
        match input {
            [value] => Ok(*value),
            _ => Err(FuzzError::new(
                FuzzErrorKind::InvalidCase,
                "decode byte",
                "exactly one byte required",
            )),
        }
    }
    fn encode(&self, case: &u8) -> Result<Vec<u8>> {
        Ok(vec![*case])
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<u8>> {
        if input.is_empty() || input == [255] {
            return Ok(Generation::Rejected(
                GenerationRejection::new("no valid generation recipe").unwrap(),
            ));
        }
        self.decode(input).map(Generation::Generated)
    }
    fn evaluate(&self, case: &u8) -> Result<CaseOutcome> {
        self.evaluated.set(self.evaluated.get() + 1);
        match case {
            254 => panic!("unknown owner panic"),
            4 => CaseOutcome::expected_rejection(
                vec![],
                "product rejected value without checking a property",
            ),
            5 => CaseOutcome::failure(key("five"), "five is forbidden"),
            value if *value >= 8 => CaseOutcome::failure(key("large"), "value exceeds safe range"),
            value if value % 2 == 1 => CaseOutcome::failure(key("odd"), "value is odd"),
            _ => CaseOutcome::evaluated(vec![property()]),
        }
    }
    fn reduction_candidates(&self, _: &u8, emit: &mut dyn FnMut(u8) -> bool) -> Result<()> {
        for value in [3, 2, 1] {
            if !emit(value) {
                break;
            }
        }
        Ok(())
    }
}

#[test]
fn generated_failure_and_concrete_replay_share_the_owner_evaluator() {
    let target = Target::new();
    let mut captured = vec![];
    let generated = evaluate_generated(&target, &[3], &mut |bytes| {
        assert_eq!(target.evaluated.get(), 0);
        captured.extend_from_slice(bytes);
        Ok(())
    })
    .unwrap();
    let GeneratedEvaluation::Evaluated(generated) = generated else {
        panic!("valid recipe rejected")
    };
    assert_eq!(captured, [3]);
    assert_eq!(generated.outcome.failure_key(), Some(&key("odd")));
    let replayed = replay(&target, &captured, &mut |_| Ok(())).unwrap();
    assert_eq!(generated, replayed);
    assert_eq!(target.evaluated.get(), 2);
}

#[test]
fn capture_error_and_oversized_input_prevent_owner_evaluation() {
    let target = Target::new();
    let cause = evaluate_generated(&target, &[2], &mut |_| {
        Err(FuzzError::new(FuzzErrorKind::Io, "capture", "disk full"))
    })
    .unwrap_err();
    assert_eq!(cause.kind(), FuzzErrorKind::Io);
    assert_eq!(target.evaluated.get(), 0);
    assert_eq!(
        evaluate_generated(&target, &[0; 17], &mut |_| Ok(()))
            .unwrap_err()
            .kind(),
        FuzzErrorKind::InvalidLimits
    );
    assert_eq!(target.evaluated.get(), 0);
}

fn exchange(target: &Target, request: &WorkerRequest) -> Vec<WorkerResponse> {
    let mut request_bytes = vec![];
    write_frame(&mut request_bytes, request, 4096).unwrap();
    let mut output = vec![];
    serve_io(
        target,
        &mut Cursor::new(request_bytes),
        &mut output,
        WorkerLimits::new(4096, 4096, 16384, 4).unwrap(),
    )
    .unwrap();
    let mut output = Cursor::new(output);
    let mut responses = vec![];
    while let Some(response) = read_frame(&mut output, 4096).unwrap() {
        responses.push(response);
    }
    responses
}

#[test]
fn worker_capture_precedes_failure_and_generation_rejection_is_separate() {
    let target = Target::new();
    let response = exchange(&target, &WorkerRequest::Evaluate { canonical: vec![3] });
    assert_eq!(response[0], WorkerResponse::Captured { canonical: vec![3] });
    assert!(
        matches!(&response[1], WorkerResponse::Evaluated { outcome, counts } if outcome.failure_key() == Some(&key("odd")) && counts.evaluated() == 1)
    );
    assert!(
        matches!(&exchange(&target, &WorkerRequest::Generate { entropy: vec![255] })[..], [WorkerResponse::GenerationRejected { counts, .. }] if *counts == ExecutionCounts::new(0, 1))
    );
    assert_eq!(
        exchange(&target, &WorkerRequest::Generate { entropy: vec![2] }),
        vec![WorkerResponse::Generated { canonical: vec![2] }]
    );
    assert_eq!(target.evaluated.get(), 1);
}

#[test]
fn unknown_owner_panic_leaves_only_the_flushed_concrete_capture() {
    let target = Target::new();
    let mut request = vec![];
    write_frame(
        &mut request,
        &WorkerRequest::Evaluate {
            canonical: vec![254],
        },
        4096,
    )
    .unwrap();
    let mut output = vec![];
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        serve_io(
            &target,
            &mut Cursor::new(request),
            &mut output,
            WorkerLimits::default(),
        )
    }));
    assert!(panic.is_err());
    let mut output = Cursor::new(output);
    assert_eq!(
        read_frame::<_, WorkerResponse>(&mut output, 4096).unwrap(),
        Some(WorkerResponse::Captured {
            canonical: vec![254]
        })
    );
    assert!(
        read_frame::<_, WorkerResponse>(&mut output, 4096)
            .unwrap()
            .is_none()
    );
}

#[test]
fn reduction_stops_at_explicit_candidate_budget_without_evaluating() {
    let target = Target::new();
    assert_eq!(
        exchange(
            &target,
            &WorkerRequest::Reduce {
                canonical: vec![9],
                max_candidates: 2
            }
        ),
        vec![
            WorkerResponse::Candidate { canonical: vec![3] },
            WorkerResponse::Candidate { canonical: vec![2] },
            WorkerResponse::Reduced { emitted: 2 }
        ]
    );
    assert_eq!(target.evaluated.get(), 0);
}

fn json_frame(json: &str) -> Vec<u8> {
    let mut bytes = (json.len() as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(json.as_bytes());
    bytes
}

#[test]
fn request_frames_reject_unknown_duplicate_missing_and_malformed_fields() {
    for json in [
        r#"{"mode":"descriptor","extra":true}"#,
        r#"{"mode":"evaluate","canonical":[1],"canonical":[2]}"#,
        r#"{"mode":"descriptor","mode":"generate","entropy":[1]}"#,
        r#"{"mode":"evaluate"}"#,
        r#"{"mode":"generate","entropy":[256]}"#,
        r#"{"mode":"reduce","canonical":[1]}"#,
        r#"{"mode":"evaluate","canonical":[1]} null"#,
        "{",
    ] {
        assert!(
            read_frame::<_, WorkerRequest>(&mut Cursor::new(json_frame(json)), 4096).is_err(),
            "accepted {json}"
        );
    }
    for bytes in [
        vec![0],
        vec![0, 0, 0, 10, b'{'],
        4097_u32.to_be_bytes().to_vec(),
        vec![0; 4],
    ] {
        assert!(read_frame::<_, WorkerRequest>(&mut Cursor::new(bytes), 4096).is_err());
    }
}

#[test]
fn worker_refuses_second_request_and_enforces_output_quota_before_writing() {
    let target = Target::new();
    let mut input = json_frame(r#"{"mode":"descriptor"}"#);
    input.extend(json_frame(r#"{"mode":"descriptor"}"#));
    let mut output = vec![];
    assert_eq!(
        serve_io(
            &target,
            &mut Cursor::new(input),
            &mut output,
            WorkerLimits::default()
        )
        .unwrap_err()
        .kind(),
        FuzzErrorKind::Codec
    );
    assert!(output.is_empty());
    assert_eq!(
        write_frame(
            &mut output,
            &WorkerResponse::Captured {
                canonical: vec![1, 2, 3]
            },
            4
        )
        .unwrap_err()
        .kind(),
        FuzzErrorKind::Resource
    );
    assert!(output.is_empty());
}

#[test]
fn worker_total_output_quota_retains_capture_and_rejects_later_observation() {
    let target = Target::new();
    let mut input = vec![];
    write_frame(
        &mut input,
        &WorkerRequest::Evaluate { canonical: vec![2] },
        4096,
    )
    .unwrap();
    let mut output = vec![];
    let error = serve_io(
        &target,
        &mut Cursor::new(input),
        &mut output,
        WorkerLimits::new(4096, 64, 68, 1).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), FuzzErrorKind::Resource);
    let mut output = Cursor::new(output);
    assert_eq!(
        read_frame::<_, WorkerResponse>(&mut output, 4096).unwrap(),
        Some(WorkerResponse::Captured { canonical: vec![2] })
    );
    assert!(
        read_frame::<_, WorkerResponse>(&mut output, 4096)
            .unwrap()
            .is_none()
    );
    assert_eq!(target.evaluated.get(), 1);
}

#[cfg(feature = "coverage")]
#[test]
fn framed_coverage_reports_failure_identity_before_escalating() {
    use surgeist_fuzz::engine::CoverageDriver;
    let target = Target::new();
    let mut driver = CoverageDriver::new(&target, None).unwrap();
    let mut output = vec![];
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || driver.libfuzzer_entry_framed(&[3], &mut output, 4096)
        ))
        .is_err()
    );
    let mut output = Cursor::new(output);
    assert_eq!(
        read_frame::<_, WorkerResponse>(&mut output, 4096).unwrap(),
        Some(WorkerResponse::Captured { canonical: vec![3] })
    );
    assert!(
        matches!(read_frame::<_, WorkerResponse>(&mut output, 4096).unwrap(), Some(WorkerResponse::Evaluated { outcome, counts }) if outcome.failure_key() == Some(&key("odd")) && counts == ExecutionCounts::new(1, 0))
    );
    assert!(
        read_frame::<_, WorkerResponse>(&mut output, 4096)
            .unwrap()
            .is_none()
    );
}

#[cfg(feature = "coverage")]
#[test]
fn safe_arbitrary_adapter_uses_real_arbitrary_integer_contract() {
    assert_eq!(
        surgeist_fuzz::engine::arbitrary_recipe::<u16>(&[0x34, 0x12]).unwrap(),
        0x1234
    );
    // The upstream integer contract accepts exhausted entropy as zero.
    assert_eq!(
        surgeist_fuzz::engine::arbitrary_recipe::<u16>(&[]).unwrap(),
        0
    );
}

#[cfg(feature = "coverage")]
#[test]
fn coverage_counts_discards_and_checked_properties_and_filters_other_failures() {
    use surgeist_fuzz::{CoverageAssessment, engine::CoverageDriver};
    let target = Target::new();
    let mut driver = CoverageDriver::new(&target, Some(key("odd"))).unwrap();
    driver.test_one(&[255], &mut |_| Ok(())).unwrap();
    driver.test_one(&[4], &mut |_| Ok(())).unwrap();
    driver.test_one(&[2], &mut |_| Ok(())).unwrap();
    assert_eq!(driver.counts(), ExecutionCounts::new(1, 1));
    assert_eq!(driver.assessment(), CoverageAssessment::Insufficient);
    let other = driver.libfuzzer_entry(&[5], &mut |_| Ok(())).unwrap();
    assert!(
        matches!(other, GeneratedEvaluation::Evaluated(evaluation) if evaluation.outcome.failure_key() == Some(&key("five")))
    );
    assert_eq!(driver.assessment(), CoverageAssessment::Sufficient);
    let mut captured = vec![];
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| driver.libfuzzer_entry(
            &[3],
            &mut |bytes| {
                captured.extend_from_slice(bytes);
                Ok(())
            }
        )))
        .is_err()
    );
    assert_eq!(captured, [3]);
    assert_eq!(driver.counts().evaluated(), 3);
}

#[cfg(feature = "property")]
fn config() -> surgeist_fuzz::engine::PropertyConfig {
    surgeist_fuzz::engine::PropertyConfig {
        cases: 4,
        max_rejections: 2,
        max_shrink_iterations: 16,
        seed: 42,
    }
}

#[cfg(feature = "property")]
#[test]
fn real_proptest_checks_semantic_properties_and_returns_concrete_failure() {
    use proptest::prelude::*;
    use surgeist_fuzz::{CoverageAssessment, engine::run_property};
    let target = Target::new();
    let success = run_property(&target, &Just(vec![2]), config(), &mut |_| Ok(())).unwrap();
    assert_eq!(success.assessment, CoverageAssessment::Sufficient);
    assert!(success.counts.evaluated() >= 4);
    assert!(success.failure.is_none());
    let failure = run_property(
        &target,
        &(8_u8..=200).prop_map(|value| vec![value]),
        config(),
        &mut |_| Ok(()),
    )
    .unwrap()
    .failure
    .unwrap();
    assert_eq!(failure.canonical, [8]);
    assert_eq!(failure.outcome.failure_key(), Some(&key("large")));
    assert_eq!(
        replay(&target, &failure.canonical, &mut |_| Ok(())).unwrap(),
        failure
    );
}

#[cfg(feature = "property")]
#[test]
fn property_rejection_only_and_unknown_panic_cannot_be_success_or_stable_failure() {
    use proptest::prelude::*;
    use surgeist_fuzz::engine::run_property;
    let target = Target::new();
    for input in [vec![255], vec![4]] {
        assert_eq!(
            run_property(&target, &Just(input), config(), &mut |_| Ok(()))
                .unwrap_err()
                .kind(),
            FuzzErrorKind::InsufficientCoverage
        );
    }
    assert_eq!(
        run_property(&target, &Just(vec![254]), config(), &mut |_| Ok(()))
            .unwrap_err()
            .kind(),
        FuzzErrorKind::Harness
    );
}

#[cfg(feature = "property")]
#[test]
fn real_state_machine_strategy_feeds_the_same_owner_property_driver() {
    use proptest::prelude::*;
    use surgeist_fuzz::engine::{run_property, state_machine::Sequential};
    let strategy = Sequential::new(
        (1..=3).into(),
        || Just(false),
        |state: &bool, transition: &bool| state != transition,
        |state: &bool| Just(!*state),
        |_: bool, transition: &bool| *transition,
    )
    .prop_map(|(_, transitions, _)| vec![if *transitions.last().unwrap() { 2 } else { 6 }]);
    let target = Target::new();
    let report = run_property(&target, &strategy, config(), &mut |_| Ok(())).unwrap();
    assert!(report.failure.is_none());
    assert_eq!(report.counts.evaluated(), 4);
    assert_eq!(target.evaluated.get(), 4);
}

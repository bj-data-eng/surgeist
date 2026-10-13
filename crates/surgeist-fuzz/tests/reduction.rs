#![forbid(unsafe_code)]
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use surgeist_fuzz::reduction::{
    ReductionEvaluation, ReductionLimits, ReductionPhase, ReductionStatus, reduce_direct,
    reduce_with,
};
use surgeist_fuzz::{
    BuildRequirements, CaseFormat, CaseOutcome, CaseTarget, FailureClass, FailureKey, FuzzError,
    FuzzErrorKind, Generation, GeneratorIdentity, InputEncoding, InputLimits, PropertyId, Result,
    TargetDescriptor, TargetDescriptorSpec,
};

fn property() -> PropertyId {
    PropertyId::new("surgeist-test::selected-failure").unwrap()
}
fn key(discriminator: &str) -> FailureKey {
    FailureKey::new(
        property(),
        FailureClass::PropertyViolation,
        Some(discriminator.into()),
    )
    .unwrap()
}
fn fail(discriminator: &str) -> CaseOutcome {
    CaseOutcome::failure(key(discriminator), "controlled violation").unwrap()
}
fn pass() -> CaseOutcome {
    CaseOutcome::evaluated(vec![property()]).unwrap()
}
fn limits() -> ReductionLimits {
    ReductionLimits::new(2000, Duration::from_secs(5), 100_000, 2).unwrap()
}
fn target(encoding: InputEncoding) -> Target {
    Target {
        descriptor: TargetDescriptor::new(TargetDescriptorSpec {
            id: surgeist_fuzz::TargetId::new("surgeist-test::reducer").unwrap(),
            owner: "surgeist-test".into(),
            binary: "reducer".into(),
            properties: vec![property()],
            source_references: vec!["https://example.test/reduction-contract".into()],
            format: CaseFormat::new("surgeist-test::bytes", 1).unwrap(),
            generator_identity: GeneratorIdentity::new("surgeist-test::bytes", 1).unwrap(),
            encoding,
            limits: InputLimits::new(128, 0, 0, 0, 0).unwrap(),
            requirements: BuildRequirements::new(vec![], vec![]).unwrap(),
            min_evaluated: 1,
            max_generation_rejections: 0,
        })
        .unwrap(),
        canonicalize: false,
        duplicate: false,
    }
}
struct Target {
    descriptor: TargetDescriptor,
    canonicalize: bool,
    duplicate: bool,
}
impl CaseTarget for Target {
    type Case = Vec<u8>;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<Self::Case> {
        if input.len() > 128 {
            return Err(FuzzError::new(
                FuzzErrorKind::InvalidCase,
                "decode",
                "too large",
            ));
        }
        Ok(input.to_vec())
    }
    fn encode(&self, case: &Self::Case) -> Result<Vec<u8>> {
        Ok(if self.canonicalize {
            case.iter().map(u8::to_ascii_uppercase).collect()
        } else {
            case.clone()
        })
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<Self::Case>> {
        Ok(Generation::Generated(self.decode(input)?))
    }
    fn evaluate(&self, case: &Self::Case) -> Result<CaseOutcome> {
        Ok(if case.windows(4).any(|part| part == b"BUG!") {
            fail("selected")
        } else if case.windows(4).any(|part| part == b"ALT!") {
            fail("alternative")
        } else {
            pass()
        })
    }
    fn reduction_candidates(
        &self,
        case: &Self::Case,
        emit: &mut dyn FnMut(Self::Case) -> bool,
    ) -> Result<()> {
        if self.duplicate {
            while emit(case.clone()) {}
            return Ok(());
        }
        for candidate in [b"ALT!".to_vec(), b"BUG!".to_vec(), case.clone()] {
            if !emit(candidate) {
                break;
            }
        }
        Ok(())
    }
}

#[test]
fn raw_reduction_keeps_exact_selected_key_and_immutable_original() {
    let target = target(InputEncoding::RawBytes);
    let original = b"noiseBUG!ALT!suffix".to_vec();
    let report = reduce_direct(
        &target,
        &original,
        &key("selected"),
        limits(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::Complete);
    assert_eq!(report.original_input(), original);
    assert_eq!(original, b"noiseBUG!ALT!suffix");
    assert_eq!(report.best_input(), b"BUG!");
    assert!(report.original_confirmed() && report.final_confirmed());
    assert!(!report.chain().is_empty());
    assert!(
        report
            .observations()
            .iter()
            .any(|observation| observation.outcome.failure_key() == Some(&key("alternative")))
    );
    assert!(
        report
            .observations()
            .iter()
            .filter(|observation| observation.phase == ReductionPhase::Final)
            .all(|observation| observation.outcome.failure_key() == Some(&key("selected")))
    );
}

#[test]
fn structured_candidates_reject_alternative_bug_and_preserve_owner_codec() {
    let target = target(InputEncoding::Structured);
    let report = reduce_direct(
        &target,
        b"BUG!duplicated",
        &key("selected"),
        limits(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.best_input(), b"BUG!");
    assert_eq!(report.canonical_input(), Some(b"BUG!".as_slice()));
    assert_eq!(
        target
            .encode(&target.decode(report.best_input()).unwrap())
            .unwrap(),
        b"BUG!"
    );
    assert!(
        report
            .observations()
            .iter()
            .any(|observation| observation.outcome.failure_key() == Some(&key("alternative")))
    );
}

#[test]
fn nonfailing_or_flaky_original_never_starts_reduction() {
    let target = target(InputEncoding::RawBytes);
    let report = reduce_direct(
        &target,
        b"ordinary",
        &key("selected"),
        limits(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::OriginalNotReproduced);
    assert!(!report.original_confirmed());
    let mut calls = 0;
    let report = reduce_with(
        &target,
        b"BUG!",
        &key("selected"),
        limits(),
        &AtomicBool::new(false),
        |_, _| {
            calls += 1;
            Ok(ReductionEvaluation::new(
                if calls == 1 { fail("selected") } else { pass() },
                0,
            ))
        },
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::FlakyOriginal);
    assert_eq!(report.best_input(), b"BUG!");
    assert!(report.chain().is_empty());
}

#[test]
fn flaky_candidate_never_replaces_verified_best() {
    let target = target(InputEncoding::Structured);
    let mut bug_calls = 0;
    let report = reduce_with(
        &target,
        b"BUG!original",
        &key("selected"),
        limits(),
        &AtomicBool::new(false),
        |input, _| {
            let outcome = if input == b"BUG!" {
                bug_calls += 1;
                if bug_calls == 1 {
                    fail("selected")
                } else {
                    pass()
                }
            } else {
                target.evaluate(&input.to_vec())?
            };
            Ok(ReductionEvaluation::new(outcome, 0))
        },
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::FlakyCandidate);
    assert!(report.final_confirmed());
    assert_eq!(report.best_input(), b"BUG!original");
    assert!(report.chain().is_empty());
}

#[test]
fn final_reconfirmation_reports_flakiness_separately() {
    let target = target(InputEncoding::Structured);
    let mut calls = 0;
    let report = reduce_with(
        &target,
        b"BUG!",
        &key("selected"),
        limits(),
        &AtomicBool::new(false),
        |_, _| {
            calls += 1;
            Ok(ReductionEvaluation::new(
                if calls < 3 { fail("selected") } else { pass() },
                0,
            ))
        },
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::FlakyFinal);
    assert!(!report.final_confirmed());
}

#[test]
fn cancellation_retains_best_previously_verified_case() {
    let target = target(InputEncoding::Structured);
    let cancellation = AtomicBool::new(false);
    let mut bug_calls = 0;
    let report = reduce_with(
        &target,
        b"BUG!noise",
        &key("selected"),
        limits(),
        &cancellation,
        |input, _| {
            if input == b"BUG!" {
                bug_calls += 1;
                if bug_calls == 3 {
                    cancellation.store(true, Ordering::Release);
                }
            }
            Ok(ReductionEvaluation::new(
                target.evaluate(&input.to_vec())?,
                0,
            ))
        },
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::Cancelled);
    assert_eq!(report.best_input(), b"BUG!");
    assert_eq!(report.chain().len(), 1);
}

#[test]
fn attempt_and_output_budgets_do_not_accept_unconfirmed_candidates() {
    let target = target(InputEncoding::RawBytes);
    let report = reduce_direct(
        &target,
        b"BUG!noise",
        &key("selected"),
        ReductionLimits::new(4, Duration::from_secs(5), 1000, 2).unwrap(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::AttemptLimit);
    assert_eq!(report.replay_count(), 4);
    assert!(report.final_confirmed());
    assert_eq!(report.best_input(), b"BUG!noise");
    let report = reduce_with(
        &target,
        b"BUG!noise",
        &key("selected"),
        ReductionLimits::new(20, Duration::from_secs(5), 5, 2).unwrap(),
        &AtomicBool::new(false),
        |_, budget| {
            assert!(budget.output_bytes <= 5);
            Ok(ReductionEvaluation::new(fail("selected"), 6))
        },
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::OutputLimit);
    assert!(!report.original_confirmed());
}

#[test]
fn duplicate_owner_candidates_are_bounded_without_replay_or_progress() {
    let mut target = target(InputEncoding::Structured);
    target.duplicate = true;
    let report = reduce_direct(
        &target,
        b"BUG!",
        &key("selected"),
        ReductionLimits::new(8, Duration::from_secs(5), 1000, 2).unwrap(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::AttemptLimit);
    assert_eq!(report.candidate_count(), 8);
    assert_eq!(report.replay_count(), 4);
    assert!(report.final_confirmed());
    assert!(report.chain().is_empty());
}

#[test]
fn canonical_serialization_cannot_replace_exact_input_without_same_failure() {
    let mut target = target(InputEncoding::Structured);
    target.canonicalize = true;
    target.duplicate = false;
    let report = reduce_with(
        &target,
        b"bug!",
        &key("selected"),
        limits(),
        &AtomicBool::new(false),
        |input, _| {
            Ok(ReductionEvaluation::new(
                if input == b"bug!" {
                    fail("selected")
                } else {
                    fail("alternative")
                },
                0,
            ))
        },
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::Complete);
    assert_eq!(report.original_input(), b"bug!");
    assert_eq!(report.best_input(), b"bug!");
    assert_eq!(report.canonical_input(), None);
}

#[test]
fn ambiguous_panics_and_crashes_are_unsupported_without_replaying() {
    let target = target(InputEncoding::RawBytes);
    for class in [FailureClass::Panic, FailureClass::Crash] {
        let expected = FailureKey::new(property(), class, None).unwrap();
        let report = reduce_with(
            &target,
            b"BUG!",
            &expected,
            limits(),
            &AtomicBool::new(false),
            |_, _| panic!("ambiguous identity must not be evaluated"),
        )
        .unwrap();
        assert_eq!(report.status(), ReductionStatus::UnsupportedFailure);
        assert_eq!(report.replay_count(), 0);
        assert!(!report.original_confirmed());
    }
}

#[test]
fn expected_product_rejection_never_replaces_selected_failure() {
    let target = target(InputEncoding::Structured);
    let report = reduce_with(
        &target,
        b"BUG!noise",
        &key("selected"),
        limits(),
        &AtomicBool::new(false),
        |input, _| {
            let outcome = if input == b"BUG!noise" {
                fail("selected")
            } else {
                CaseOutcome::expected_rejection(vec![], "candidate rejected by product")?
            };
            Ok(ReductionEvaluation::new(outcome, 0))
        },
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::Complete);
    assert_eq!(report.best_input(), b"BUG!noise");
    assert!(report.chain().is_empty());
    assert!(report.final_confirmed());
}

#[test]
fn elapsed_replay_deadline_discards_overdue_confirmation() {
    let target = target(InputEncoding::RawBytes);
    let report = reduce_with(
        &target,
        b"BUG!",
        &key("selected"),
        ReductionLimits::new(20, Duration::from_millis(1), 1000, 2).unwrap(),
        &AtomicBool::new(false),
        |_, budget| {
            // Elapsed time is the behavior under test: deliberately exceed the exact
            // deadline supplied to this replay. No completion is inferred from sleep.
            std::thread::sleep(budget.remaining_time + Duration::from_millis(1));
            Ok(ReductionEvaluation::new(fail("selected"), 0))
        },
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::TimeLimit);
    assert!(!report.original_confirmed());
    assert!(report.replay_count() <= 1);
    assert_eq!(report.best_input(), b"BUG!");
}

struct IdentityTarget {
    descriptor: TargetDescriptor,
}
impl CaseTarget for IdentityTarget {
    // Each node identity must be unique and each following reference names it.
    type Case = Vec<(u8, u8)>;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<Self::Case> {
        if input.len() > 128 || !input.len().is_multiple_of(2) {
            return Err(FuzzError::new(
                FuzzErrorKind::InvalidCase,
                "decode identities",
                "invalid pair encoding",
            ));
        }
        let pairs: Vec<_> = input
            .chunks_exact(2)
            .map(|pair| (pair[0], pair[1]))
            .collect();
        let mut ids = std::collections::BTreeSet::new();
        if pairs
            .iter()
            .any(|(id, reference)| *id == 0 || id != reference || !ids.insert(*id))
        {
            return Err(FuzzError::new(
                FuzzErrorKind::InvalidCase,
                "decode identities",
                "invalid identity or reference",
            ));
        }
        Ok(pairs)
    }
    fn encode(&self, case: &Self::Case) -> Result<Vec<u8>> {
        Ok(case
            .iter()
            .flat_map(|(id, reference)| [*id, *reference])
            .collect())
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<Self::Case>> {
        Ok(Generation::Generated(self.decode(input)?))
    }
    fn evaluate(&self, case: &Self::Case) -> Result<CaseOutcome> {
        Ok(if case.iter().any(|(id, _)| *id == 7) {
            fail("selected")
        } else {
            pass()
        })
    }
    fn reduction_candidates(
        &self,
        case: &Self::Case,
        emit: &mut dyn FnMut(Self::Case) -> bool,
    ) -> Result<()> {
        for index in 0..case.len() {
            let mut candidate = case.clone();
            candidate.remove(index);
            if !emit(candidate) {
                break;
            }
        }
        Ok(())
    }
}

#[test]
fn owner_structural_reduction_keeps_unique_ids_and_valid_references() {
    let target = IdentityTarget {
        descriptor: target(InputEncoding::Structured).descriptor,
    };
    let original = [7, 7, 9, 9, 11, 11];
    let report = reduce_direct(
        &target,
        &original,
        &key("selected"),
        limits(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.status(), ReductionStatus::Complete);
    assert_eq!(report.original_input(), original);
    assert_eq!(report.best_input(), [7, 7]);
    assert_eq!(target.decode(report.best_input()).unwrap(), [(7, 7)]);
    assert_eq!(report.chain().len(), 2);
    assert!(report.original_confirmed() && report.final_confirmed());
}

#[test]
fn operational_interruptions_preserve_the_verified_best_without_final_confirmation() {
    use surgeist_fuzz::reduction::ReductionInterruption;
    for (reason, status) in [
        (ReductionInterruption::Cancelled, ReductionStatus::Cancelled),
        (ReductionInterruption::TimeLimit, ReductionStatus::TimeLimit),
        (
            ReductionInterruption::OutputLimit,
            ReductionStatus::OutputLimit,
        ),
    ] {
        let target = target(InputEncoding::Structured);
        let mut best_calls = 0;
        let report = reduce_with(
            &target,
            b"BUG!noise",
            &key("selected"),
            limits(),
            &AtomicBool::new(false),
            |input, _| {
                if input == b"BUG!" {
                    best_calls += 1;
                    if best_calls == 3 {
                        return ReductionEvaluation::interrupted(reason, 7);
                    }
                }
                Ok(ReductionEvaluation::new(
                    target.evaluate(&input.to_vec())?,
                    3,
                ))
            },
        )
        .unwrap();
        assert_eq!(report.status(), status);
        assert_eq!(report.original_input(), b"BUG!noise");
        assert_eq!(report.best_input(), b"BUG!");
        assert!(report.original_confirmed());
        assert!(!report.final_confirmed());
        assert_eq!(report.chain().len(), 1);
        assert_eq!(report.replay_count() as usize, report.observations().len());
        assert_eq!(report.output_bytes(), (report.replay_count() - 1) * 3 + 7);
        assert!(
            report
                .observations()
                .last()
                .unwrap()
                .outcome
                .failure_key()
                .is_none()
        );
    }
}

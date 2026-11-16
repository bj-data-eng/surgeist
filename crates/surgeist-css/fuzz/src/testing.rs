//! Test-only support shared by every target's controlled-failure tests.
use std::{sync::atomic::AtomicBool, time::Duration};
use surgeist_fuzz::{
    CaseTarget, FailureClass, FailureKey, OutcomeKind, PropertyId, engine,
    reduction::{ReductionLimits, ReductionStatus, reduce_direct},
};

/// The exact failure a deliberately broken expectation must produce.
pub(crate) fn key(property: &str, discriminator: &str) -> FailureKey {
    FailureKey::new(
        PropertyId::new(property).unwrap(),
        FailureClass::PropertyViolation,
        Some(discriminator.to_owned()),
    )
    .unwrap()
}

/// A planted failure is detected, captured exactly, replays to the same
/// failure from its canonical bytes and reduces to a smaller input that still
/// fails with the same key. Returns the reduced canonical bytes.
pub(crate) fn assert_detected_replayed_and_reduced<T: CaseTarget>(
    broken: &T,
    sound: &T,
    case: &T::Case,
    expected: &FailureKey,
) -> Vec<u8> {
    let mut captured = Vec::new();
    let observed = engine::evaluate_case(broken, case, &mut |bytes| {
        captured = bytes.to_vec();
        Ok(())
    })
    .unwrap();
    assert_eq!(observed.outcome.kind(), OutcomeKind::Failure);
    assert_eq!(observed.outcome.failure_key(), Some(expected));
    assert_eq!(captured, observed.canonical);

    // The sound expectations accept the same input, so the failure is the
    // planted expectation's and not the input's.
    let sound_outcome = engine::replay(sound, &captured, &mut |_| Ok(())).unwrap();
    assert_ne!(
        sound_outcome.outcome.kind(),
        OutcomeKind::Failure,
        "{:?}",
        sound_outcome.outcome.detail()
    );

    // Replay from the canonical bytes alone reproduces the identical observation.
    let replayed = engine::replay(broken, &captured, &mut |_| Ok(())).unwrap();
    assert_eq!(replayed, observed);

    let limits = ReductionLimits::new(400, Duration::from_secs(60), 1 << 20, 2).unwrap();
    let report =
        reduce_direct(broken, &captured, expected, limits, &AtomicBool::new(false)).unwrap();
    assert_eq!(report.status(), ReductionStatus::Complete);
    assert!(report.original_confirmed() && report.final_confirmed());
    let best = report
        .canonical_input()
        .unwrap_or(report.best_input())
        .to_vec();
    assert!(best.len() <= captured.len());
    let reduced = engine::replay(broken, &best, &mut |_| Ok(())).unwrap();
    assert_eq!(reduced.outcome.failure_key(), Some(expected));
    best
}

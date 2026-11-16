#![forbid(unsafe_code)]
//! Every target through the shared framework: capture, exact replay, generation,
//! optional property and coverage engines, and seed freshness.
use surgeist_css_fuzz::{ByteTarget, SourceTarget, ValueTarget};
use surgeist_fuzz::{CaseTarget, Generation, OutcomeKind, engine};

fn check_literals<T: CaseTarget>(target: &T, count: u8, literal: impl Fn(u8) -> T::Case) {
    for selector in 0..count {
        let case = literal(selector);
        let mut captured = Vec::new();
        let observed = engine::evaluate_case(target, &case, &mut |bytes| {
            captured = bytes.to_vec();
            Ok(())
        })
        .unwrap();
        assert!(
            !observed.outcome.checked_properties().is_empty(),
            "selector {selector}"
        );
        assert!(
            matches!(
                observed.outcome.kind(),
                OutcomeKind::Evaluated | OutcomeKind::ExpectedRejection
            ),
            "selector {selector}: {:?}",
            observed.outcome.detail()
        );
        assert_eq!(captured, observed.canonical);
        let decoded = target.decode(&captured).unwrap();
        assert_eq!(target.encode(&decoded).unwrap(), captured);
        let replayed = engine::replay(target, &captured, &mut |_| Ok(())).unwrap();
        assert_eq!(observed, replayed, "selector {selector}");
    }
}

#[test]
fn literals_capture_exact_inputs_and_replay_the_same_checked_properties() {
    let bytes = ByteTarget::new();
    check_literals(&bytes, ByteTarget::LITERALS, |id| {
        bytes.literal(id).unwrap()
    });
    let source = SourceTarget::new();
    check_literals(&source, SourceTarget::LITERALS, |id| {
        source.literal(id).unwrap()
    });
    let values = ValueTarget::new();
    check_literals(&values, ValueTarget::LITERALS, |id| {
        values.literal(id).unwrap()
    });
}

fn entropy(seed: u8, length: u8) -> Vec<u8> {
    (0..length)
        .map(|slot| seed.wrapping_mul(17).wrapping_add(slot).wrapping_mul(31))
        .collect()
}

fn generated<T: CaseTarget>(target: &T) {
    let mut evaluated = 0;
    for seed in 0_u8..64 {
        let entropy = entropy(seed, 192);
        let first = target.generate(&entropy).unwrap();
        let second = target.generate(&entropy).unwrap();
        match (first, second) {
            (Generation::Generated(first), Generation::Generated(second)) => {
                let bytes = target.encode(&first).unwrap();
                assert_eq!(bytes, target.encode(&second).unwrap());
                let captured = engine::evaluate_case(target, &first, &mut |_| Ok(())).unwrap();
                assert!(captured.outcome.is_evaluated());
                assert_ne!(
                    captured.outcome.kind(),
                    OutcomeKind::Failure,
                    "seed {seed}: {:?}",
                    captured.outcome.detail()
                );
                assert_eq!(
                    captured,
                    engine::replay(target, &bytes, &mut |_| Ok(())).unwrap()
                );
                evaluated += 1;
            }
            (Generation::Rejected(first), Generation::Rejected(second)) => {
                assert_eq!(first, second)
            }
            _ => panic!("same entropy changed generation disposition"),
        }
    }
    assert!(evaluated > 0, "generation must check real properties");
}

#[test]
fn bounded_generation_is_deterministic_and_uses_the_replay_evaluator() {
    generated(&ByteTarget::new());
    generated(&SourceTarget::new());
    generated(&ValueTarget::new());
}

#[cfg(feature = "property")]
fn property<T: CaseTarget>(target: &T, seed: u64) {
    use proptest::prelude::*;
    let strategy = proptest::collection::vec(any::<u8>(), 0..256);
    let mut captures = 0;
    let report = engine::run_property(
        target,
        &strategy,
        engine::PropertyConfig {
            cases: 64,
            max_rejections: 64,
            max_shrink_iterations: 16,
            seed,
        },
        &mut |bytes| {
            let case = target.decode(bytes)?;
            assert_eq!(target.encode(&case)?, bytes);
            captures += 1;
            Ok(())
        },
    )
    .unwrap();
    assert!(report.failure.is_none(), "{:?}", report.failure);
    assert_eq!(report.counts.evaluated(), 64);
    assert_eq!(captures, 64);
}

#[cfg(feature = "property")]
#[test]
fn finite_property_runs_capture_nonvacuous_public_observations() {
    property(&ByteTarget::new(), 1147);
    property(&SourceTarget::new(), 1147);
    property(&ValueTarget::new(), 1147);
}

#[cfg(feature = "coverage")]
fn coverage<T: CaseTarget>(target: &T) {
    let mut driver = engine::CoverageDriver::new(target, None).unwrap();
    for seed in 0_u8..32 {
        let mut captured = Vec::new();
        let entropy = entropy(seed, 192);
        let observation = driver
            .test_one(&entropy, &mut |bytes| {
                captured = bytes.to_vec();
                Ok(())
            })
            .unwrap();
        if let engine::GeneratedEvaluation::Evaluated(observation) = observation {
            assert_eq!(
                observation,
                engine::replay(target, &captured, &mut |_| Ok(())).unwrap()
            );
            assert_ne!(observation.outcome.kind(), OutcomeKind::Failure);
        }
    }
    assert!(driver.counts().evaluated() > 0);
}

#[cfg(feature = "coverage")]
#[test]
fn finite_coverage_callbacks_capture_the_same_case_as_direct_replay() {
    coverage(&ByteTarget::new());
    coverage(&SourceTarget::new());
    coverage(&ValueTarget::new());
}

fn seed_freshness<T: CaseTarget>(
    target: &T,
    directory: &str,
    count: u8,
    literal: impl Fn(u8) -> T::Case,
) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("seeds")
        .join(directory);
    assert_eq!(
        std::fs::read_dir(&root).unwrap().count(),
        usize::from(count)
    );
    for selector in 0..count {
        let actual = std::fs::read(root.join(format!("{selector:02}.case"))).unwrap();
        let expected = target.encode(&literal(selector)).unwrap();
        assert_eq!(actual, expected, "generated {directory} seed {selector}");
        assert_eq!(
            target.encode(&target.decode(&actual).unwrap()).unwrap(),
            actual
        );
    }
}

#[test]
fn generated_seed_artifacts_match_the_authoritative_literal_factories_and_codec() {
    let bytes = ByteTarget::new();
    seed_freshness(&bytes, "bytes", ByteTarget::LITERALS, |id| {
        bytes.literal(id).unwrap()
    });
    let source = SourceTarget::new();
    seed_freshness(&source, "source", SourceTarget::LITERALS, |id| {
        source.literal(id).unwrap()
    });
    let values = ValueTarget::new();
    seed_freshness(&values, "values", ValueTarget::LITERALS, |id| {
        values.literal(id).unwrap()
    });
    seed_freshness(
        &values,
        "known-failures",
        ValueTarget::KNOWN_FAILURES,
        |id| values.known_failure(id).unwrap(),
    );
}

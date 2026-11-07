#![forbid(unsafe_code)]
use surgeist_fuzz::{CaseTarget, Generation, OutcomeKind, engine};
use surgeist_render_fuzz::{ResourceTarget, SceneTarget};

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
        assert!(matches!(
            observed.outcome.kind(),
            OutcomeKind::Evaluated | OutcomeKind::ExpectedRejection
        ));
        assert_eq!(captured, observed.canonical);
        let decoded = target.decode(&captured).unwrap();
        assert_eq!(target.encode(&decoded).unwrap(), captured);
        let replayed = engine::replay(target, &captured, &mut |_| Ok(())).unwrap();
        assert_eq!(observed, replayed, "selector {selector}");
    }
}

#[test]
fn resource_literals_capture_exact_inputs_and_replay_the_same_checked_properties() {
    let target = ResourceTarget::new();
    check_literals(&target, 24, |selector| target.literal(selector).unwrap());
    assert!(target.literal(24).is_err());
}
#[test]
fn scene_literals_capture_exact_inputs_and_replay_the_same_checked_properties() {
    let target = SceneTarget::new();
    check_literals(&target, 12, |selector| target.literal(selector).unwrap());
    assert!(target.literal(12).is_err());
}

fn generated<T: CaseTarget>(target: &T) {
    let mut evaluated = 0;
    for seed in 0_u8..32 {
        let entropy: Vec<_> = (0_u8..128)
            .map(|slot| seed.wrapping_mul(17).wrapping_add(slot))
            .collect();
        let first = target.generate(&entropy).unwrap();
        let second = target.generate(&entropy).unwrap();
        match (first, second) {
            (Generation::Generated(first), Generation::Generated(second)) => {
                let bytes = target.encode(&first).unwrap();
                assert_eq!(bytes, target.encode(&second).unwrap());
                let captured = engine::evaluate_case(target, &first, &mut |_| Ok(())).unwrap();
                assert!(captured.outcome.is_evaluated());
                assert_ne!(captured.outcome.kind(), OutcomeKind::Failure);
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
    generated(&ResourceTarget::new());
    generated(&SceneTarget::new());
}

#[cfg(feature = "property")]
fn property<T: CaseTarget>(target: &T) {
    use proptest::prelude::*;
    let strategy = proptest::collection::vec(any::<u8>(), 0..256);
    let mut captures = 0;
    let report = engine::run_property(
        target,
        &strategy,
        engine::PropertyConfig {
            cases: 32,
            max_rejections: 64,
            max_shrink_iterations: 16,
            seed: 1155,
        },
        &mut |bytes| {
            let case = target.decode(bytes)?;
            assert_eq!(target.encode(&case)?, bytes);
            captures += 1;
            Ok(())
        },
    )
    .unwrap();
    assert!(report.failure.is_none());
    assert_eq!(report.counts.evaluated(), 32);
    assert_eq!(captures, 32);
}
#[cfg(feature = "property")]
#[test]
fn finite_property_runs_capture_nonvacuous_public_observations() {
    property(&ResourceTarget::new());
    property(&SceneTarget::new());
}

#[cfg(feature = "coverage")]
fn coverage<T: CaseTarget>(target: &T) {
    let mut driver = engine::CoverageDriver::new(target, None).unwrap();
    for seed in 0_u8..16 {
        let mut captured = Vec::new();
        let entropy: Vec<_> = (0_u8..128).map(|slot| seed.wrapping_add(slot)).collect();
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
    coverage(&ResourceTarget::new());
    coverage(&SceneTarget::new());
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
    let resources = ResourceTarget::new();
    seed_freshness(&resources, "resources", 24, |selector| {
        resources.literal(selector).unwrap()
    });
    let scenes = SceneTarget::new();
    seed_freshness(&scenes, "scenes", 12, |selector| {
        scenes.literal(selector).unwrap()
    });
}

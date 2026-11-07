use super::*;
#[cfg(feature = "property")]
use surgeist_fuzz::FailureClass;
use surgeist_fuzz::engine;

fn inspection(case: SceneCase) -> Inspection {
    let Payload::Inspect(recipe) = case.payload else {
        panic!("expected inspection fixture")
    };
    recipe
}

#[test]
fn literal_recipes_round_trip_and_check_their_actual_properties() {
    let target = SceneTarget::new();
    for which in 0..12 {
        let case = target.literal(which).unwrap();
        let bytes = target.encode(&case).unwrap();
        assert_eq!(&bytes[..5], b"SRFS\x01");
        assert_eq!(
            target.encode(&target.decode(&bytes).unwrap()).unwrap(),
            bytes
        );
        let evaluation = engine::replay(&target, &bytes, &mut |captured| {
            assert_eq!(captured, bytes);
            Ok(())
        })
        .unwrap();
        assert!(
            evaluation.outcome.failure_key().is_none(),
            "literal {which}: {:?}",
            evaluation.outcome
        );
        assert!(evaluation.outcome.is_evaluated(), "literal {which}");
        target
            .descriptor()
            .validate_outcome(&evaluation.outcome)
            .unwrap();
    }
    assert!(target.literal(13).is_err());
}

#[test]
fn nested_separately_allocated_equal_and_dimension_distinct_images_have_exact_counts() {
    let target = SceneTarget::new();
    let recipe = inspection(target.literal(0).unwrap());
    let actual = recipe
        .build()
        .unwrap()
        .inspect(&mut render::RenderAccountingContext::default())
        .unwrap();
    // Hand-derived from the authored recipe, without consulting the comparator.
    assert_eq!(
        actual,
        SceneSummary {
            commands: 10,
            fills: 2,
            strokes: 1,
            shadows: 1,
            images: 6,
            glyphs: 3,
            layers: 2,
            image_content_hits: 3,
            image_content_misses: 3,
            first_observed_image_bytes: 20
        }
    );
    assert!(recipe.evaluate().unwrap().failure_key().is_none());
}

#[test]
fn command_probe_and_identity_byte_refusals_release_scratch_preserve_scene_and_retry() {
    let target = SceneTarget::new();
    let pixel = ImageRecipe {
        width: 1,
        height: 1,
        rgba: vec![1, 2, 3, 255],
    };
    for refusal in [
        Refusal::Commands {
            limit: 1,
            retry_limit: 64,
        },
        Refusal::ResourceProbe { retry_limit: 64 },
        Refusal::IdentityByte {
            retry_limit: 262144,
        },
    ] {
        let case = SceneCase {
            payload: Payload::Inspect(Inspection {
                images: vec![pixel.clone(), pixel.clone()],
                commands: vec![
                    Node {
                        parent: ROOT,
                        draw: Draw::Image(0),
                    },
                    Node {
                        parent: ROOT,
                        draw: Draw::Fill(1),
                    },
                ],
                refusal,
            }),
        };
        let outcome = target.evaluate(&case).unwrap();
        assert!(outcome.failure_key().is_none(), "{outcome:?}");
        assert!(
            outcome
                .checked_properties()
                .iter()
                .any(|property| property.as_str() == RETRY)
        );
    }
}

#[test]
fn codec_rejects_target_revision_unknown_tags_truncation_trailing_and_collection_bounds() {
    let target = SceneTarget::new();
    let canonical = target.encode(&target.literal(0).unwrap()).unwrap();
    for length in 0..canonical.len() {
        assert!(
            target.decode(&canonical[..length]).is_err(),
            "prefix length {length}"
        );
    }
    for (offset, value) in [(0, b'X'), (4, 2), (5, 255)] {
        let mut malformed = canonical.clone();
        malformed[offset] = value;
        assert!(target.decode(&malformed).is_err());
    }
    let mut trailing = canonical.clone();
    trailing.push(0);
    assert!(target.decode(&trailing).is_err());
    let mut writer = Writer::new(MAGIC).unwrap();
    writer.u8(0).unwrap();
    writer.u32(5).unwrap();
    assert!(target.decode(&writer.finish()).is_err());
    let mut writer = Writer::new(MAGIC).unwrap();
    writer.u8(0).unwrap();
    writer.u32(0).unwrap();
    writer.u32(65).unwrap();
    assert!(target.decode(&writer.finish()).is_err());
    let mut oversized = vec![0; crate::codec::MAX_CASE_BYTES + 1];
    oversized[..5].copy_from_slice(b"SRFS\x01");
    assert!(target.decode(&oversized).is_err());
}

#[test]
fn forward_self_nonlayer_image_references_depth_and_aggregate_payloads_are_refused() {
    let target = SceneTarget::new();
    for commands in [
        vec![Node {
            parent: 0,
            draw: Draw::Layer,
        }],
        vec![
            Node {
                parent: ROOT,
                draw: Draw::Fill(ROOT),
            },
            Node {
                parent: 0,
                draw: Draw::Fill(ROOT),
            },
        ],
        vec![Node {
            parent: ROOT,
            draw: Draw::Image(0),
        }],
    ] {
        assert!(
            target
                .encode(&SceneCase {
                    payload: Payload::Inspect(Inspection {
                        images: vec![],
                        commands,
                        refusal: Refusal::None
                    })
                })
                .is_err()
        );
    }
    let commands = (0..9)
        .map(|index| Node {
            parent: if index == 0 { ROOT } else { index - 1 },
            draw: Draw::Layer,
        })
        .collect();
    assert!(
        target
            .encode(&SceneCase {
                payload: Payload::Inspect(Inspection {
                    images: vec![],
                    commands,
                    refusal: Refusal::None
                })
            })
            .is_err()
    );
    let image = ImageRecipe {
        width: 16,
        height: 16,
        rgba: vec![255; 1024],
    };
    let mut writer = Writer::new(MAGIC).unwrap();
    writer.u8(0).unwrap();
    writer.count(4, 4).unwrap();
    // A declared byte length beyond each exact dimension is rejected before copy.
    writer.u8(1).unwrap();
    writer.u8(1).unwrap();
    writer.u32(4096).unwrap();
    assert!(target.decode(&writer.finish()).is_err());
    let accepted = SceneCase {
        payload: Payload::Inspect(Inspection {
            images: vec![image; 4],
            commands: vec![],
            refusal: Refusal::None,
        }),
    };
    let bytes = target.encode(&accepted).unwrap();
    assert!(target.decode(&bytes).is_ok());
}

#[test]
fn exceptional_and_signed_zero_coefficient_bits_remain_canonical() {
    let target = SceneTarget::new();
    let bits = [
        0,
        0x8000_0000_0000_0000,
        0x7ff8_0000_0000_1234,
        0xfff0_0000_0000_0000,
        1,
        0x7fef_ffff_ffff_ffff,
    ];
    let case = SceneCase {
        payload: Payload::Coefficients(bits.map(f64::from_bits)),
    };
    let bytes = target.encode(&case).unwrap();
    let decoded = target.decode(&bytes).unwrap();
    let Payload::Coefficients(actual) = decoded.payload else {
        panic!("coefficient fixture")
    };
    assert_eq!(actual.map(f64::to_bits), bits);
    assert_eq!(
        target
            .encode(&SceneCase {
                payload: Payload::Coefficients(actual)
            })
            .unwrap(),
        bytes
    );
    assert!(target.evaluate(&case).unwrap().failure_key().is_none());
}

#[test]
fn canonical_refusal_retains_exact_low_and_raised_work_limits() {
    let target = SceneTarget::new();
    let case = target.literal(2).unwrap();
    let canonical = target.encode(&case).unwrap();
    let mut recipe = inspection(target.decode(&canonical).unwrap());
    assert!(matches!(
        recipe.refusal,
        Refusal::Commands {
            limit: 1,
            retry_limit: 64
        }
    ));
    recipe.refusal = Refusal::Commands {
        limit: 1,
        retry_limit: 128,
    };
    let changed = target
        .encode(&SceneCase {
            payload: Payload::Inspect(recipe.clone()),
        })
        .unwrap();
    assert_ne!(changed, canonical);
    recipe.refusal = Refusal::Commands {
        limit: 1,
        retry_limit: 1,
    };
    assert!(
        target
            .encode(&SceneCase {
                payload: Payload::Inspect(recipe)
            })
            .is_err()
    );
}

#[test]
fn lattice_transform_order_and_origin_are_independently_known() {
    let first = render::Transform::try_new([2., 1., -1., 3., 5., -7.]).unwrap();
    let next = render::Transform::try_new([1., -2., 3., 1., -4., 8.]).unwrap();
    assert_eq!(
        first.then(next).unwrap().as_array(),
        [5., -3., 8., 5., -20., -9.]
    );
    assert_eq!(
        first
            .around(render::Point::try_new(3., -5.).unwrap())
            .unwrap()
            .as_array(),
        [2., 1., -1., 3., -3., 0.]
    );
    assert!(
        geometry::compose(first.as_array(), next.as_array(), [3., -5.])
            .unwrap()
            .failure_key()
            .is_none()
    );
    assert!(geometry::overflow().unwrap().failure_key().is_none());
}

#[test]
fn concrete_clip_retains_original_geometry_space_and_accounts_path_walks() {
    let target = SceneTarget::new();
    for which in [6, 7, 8] {
        assert!(
            target
                .evaluate(&target.literal(which).unwrap())
                .unwrap()
                .failure_key()
                .is_none()
        );
    }
    let overflow = SceneCase {
        payload: Payload::Clip(geometry::ClipRecipe {
            geometry: geometry::ClipSource::Rect([-f64::MAX, 0., f64::MAX, 1.]),
            transform: [2., 0., 0., 1., 0., 0.],
        }),
    };
    assert!(target.evaluate(&overflow).unwrap().failure_key().is_none());
}

#[test]
fn concrete_mask_rejects_zero_negative_nonfinite_and_overflowing_bounds() {
    let target = SceneTarget::new();
    let pixel = ImageRecipe {
        width: 1,
        height: 1,
        rgba: vec![1, 2, 3, 255],
    };
    for bounds in [
        [0., 0., 0., 1.],
        [0., 0., -1., 1.],
        [f64::NAN, 0., 1., 1.],
        [f64::MAX, 0., f64::MAX, 1.],
        [-3., 2., 4., 5.],
    ] {
        let case = SceneCase {
            payload: Payload::Mask {
                image: pixel.clone(),
                bounds,
            },
        };
        assert!(target.evaluate(&case).unwrap().failure_key().is_none());
    }
}

#[test]
fn entropy_exhaustion_is_a_generation_rejection_and_bounded_recipes_are_concrete() {
    let target = SceneTarget::new();
    assert!(matches!(
        target.generate(&[]).unwrap(),
        Generation::Rejected(_)
    ));
    for selector in 0_u64..16 {
        let mut entropy = vec![0; 1024];
        entropy[..8].copy_from_slice(&selector.to_le_bytes());
        let Generation::Generated(case) = target.generate(&entropy).unwrap() else {
            panic!("bounded recipe rejected")
        };
        let encoded = target.encode(&case).unwrap();
        assert!(encoded.len() < crate::codec::MAX_CASE_BYTES);
        assert!(target.evaluate(&case).unwrap().failure_key().is_none());
    }
}

#[test]
fn empty_forest_and_image_free_commands_keep_zero_node_depth_credit_consistent() {
    let target = SceneTarget::new();
    for commands in [
        vec![],
        vec![Node {
            parent: ROOT,
            draw: Draw::Fill(ROOT),
        }],
    ] {
        let case = SceneCase {
            payload: Payload::Inspect(Inspection {
                images: vec![],
                commands,
                refusal: Refusal::None,
            }),
        };
        let canonical = target.encode(&case).unwrap();
        let decoded = target.decode(&canonical).unwrap();
        assert!(target.evaluate(&decoded).unwrap().failure_key().is_none());
    }
}

// Separate rejection-observation control. A real public inspection refusal is
// observed under a zero command cap, then presented to the unchanged positive
// comparator for the exact normal two-command case. The intentional observation
// difference belongs only to this explicitly identified test target.
#[cfg(feature = "property")]
struct InspectionRefusalObservationControl {
    normal: SceneTarget,
    descriptor: TargetDescriptor,
}
#[cfg(feature = "property")]
impl InspectionRefusalObservationControl {
    const ID: &str = "surgeist-render::scene-refusal-observation-control";
    fn new() -> Self {
        let document =
            include_str!("../../fuzz.toml").replace("surgeist-render::scene-inspection", Self::ID);
        let manifest: surgeist_fuzz::registry::FuzzManifest = toml::from_str(&document).unwrap();
        let descriptor = manifest
            .targets
            .into_iter()
            .find(|descriptor| descriptor.id().as_str() == Self::ID)
            .unwrap();
        Self {
            normal: SceneTarget::new(),
            descriptor,
        }
    }
}
#[cfg(feature = "property")]
impl CaseTarget for InspectionRefusalObservationControl {
    type Case = SceneCase;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<SceneCase> {
        self.normal.decode(input)
    }
    fn encode(&self, case: &SceneCase) -> Result<Vec<u8>> {
        self.normal.encode(case)
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<SceneCase>> {
        self.normal.generate(input)
    }
    fn evaluate(&self, case: &SceneCase) -> Result<CaseOutcome> {
        if self.normal.encode(case)? != self.normal.encode(&self.normal.literal(1)?)? {
            return Err(error(
                "refusal observation control requires the exact normal two-command case",
            ));
        }
        let Payload::Inspect(recipe) = &case.payload else {
            return Err(error("refusal observation control requires inspection"));
        };
        let scene = match product_observation(
            recipe.build(),
            STRUCTURE,
            "authored-scene-construction-rejected",
        )? {
            Ok(scene) => scene,
            Err(failure) => return Ok(failure),
        };
        let limits = render::RenderLimits::new()
            .with_work_limit(render::RenderWork::AuthoredCommands, 0)
            .map_err(|cause| error(cause.to_string()))?;
        let mut context = render::RenderAccountingContext::new(
            render::RenderAccountingOptions::new().with_limits(limits),
        );
        let observation = scene.inspect(&mut context);
        // Establish the injected observation really came from the public
        // inspection boundary's accounting refusal, rather than a made-up error.
        assert!(
            matches!(&observation, Err(cause) if matches!(cause.accounting_diagnostic(), Some(render::RenderAccountingError::Refused(_))))
        );
        compare_inspection(
            recipe.expected(),
            observation,
            STRUCTURE,
            "authored-inspection-rejected",
        )?
        .ok_or_else(|| error("real refusal observation escaped positive comparator"))
    }
}

#[cfg(feature = "property")]
#[test]
fn real_refusal_observation_has_semantic_identity_through_capture_and_replay() {
    use proptest::strategy::Just;
    let target = InspectionRefusalObservationControl::new();
    assert_eq!(
        target.descriptor().id().as_str(),
        InspectionRefusalObservationControl::ID
    );
    let canonical = target
        .normal
        .encode(&target.normal.literal(1).unwrap())
        .unwrap();
    let mut captures = Vec::new();
    let report = engine::run_property(
        &target,
        &Just(1_u64.to_le_bytes().to_vec()),
        engine::PropertyConfig {
            cases: 1,
            max_rejections: 1,
            max_shrink_iterations: 0,
            seed: 19,
        },
        &mut |bytes| {
            captures.push(bytes.to_vec());
            Ok(())
        },
    )
    .unwrap();
    let failure = report
        .failure
        .expect("real refusal control must be semantic");
    let key = failure.outcome.failure_key().unwrap();
    assert_eq!(key.property().as_str(), STRUCTURE);
    assert_eq!(key.class(), FailureClass::PropertyViolation);
    assert_eq!(key.discriminator(), Some("authored-inspection-rejected"));
    assert_eq!(failure.canonical, canonical);
    assert!(!captures.is_empty());
    assert!(captures.iter().all(|bytes| bytes == &canonical));
    let mut replay_capture = Vec::new();
    let replay = engine::replay(&target, &canonical, &mut |bytes| {
        replay_capture = bytes.to_vec();
        Ok(())
    })
    .unwrap();
    assert_eq!(replay.outcome.failure_key(), Some(key));
    assert_eq!(replay.canonical, canonical);
    assert_eq!(replay_capture, canonical);
    assert!(
        target
            .normal
            .evaluate(&target.normal.literal(1).unwrap())
            .unwrap()
            .failure_key()
            .is_none()
    );
}

// Explicit synthetic observer control: the real public Scene::inspect result is
// perturbed by exactly one command count. The normal comparator and the exact
// two-command canonical fixture are unchanged. This is a harness detection
// control, not a product regression or an override encoded in ordinary cases.
#[cfg(feature = "property")]
struct CommandCountObservationControl {
    normal: SceneTarget,
    descriptor: TargetDescriptor,
}
#[cfg(feature = "property")]
impl CommandCountObservationControl {
    const ID: &str = "surgeist-render::scene-command-count-observation-control";
    fn new() -> Self {
        let document =
            include_str!("../../fuzz.toml").replace("surgeist-render::scene-inspection", Self::ID);
        let manifest: surgeist_fuzz::registry::FuzzManifest = toml::from_str(&document).unwrap();
        let descriptor = manifest
            .targets
            .into_iter()
            .find(|descriptor| descriptor.id().as_str() == Self::ID)
            .unwrap();
        Self {
            normal: SceneTarget::new(),
            descriptor,
        }
    }
}
#[cfg(feature = "property")]
impl CaseTarget for CommandCountObservationControl {
    type Case = SceneCase;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<SceneCase> {
        self.normal.decode(input)
    }
    fn encode(&self, case: &SceneCase) -> Result<Vec<u8>> {
        self.normal.encode(case)
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<SceneCase>> {
        self.normal.generate(input)
    }
    fn evaluate(&self, case: &SceneCase) -> Result<CaseOutcome> {
        if self.normal.encode(case)? != self.normal.encode(&self.normal.literal(1)?)? {
            return Err(error(
                "observation control only admits the exact two-command case",
            ));
        }
        let Payload::Inspect(recipe) = &case.payload else {
            return Err(error("observation control needs inspection"));
        };
        let scene = match product_observation(
            recipe.build(),
            STRUCTURE,
            "authored-scene-construction-rejected",
        )? {
            Ok(scene) => scene,
            Err(failure) => return Ok(failure),
        };
        let mut observed = match product_observation(
            scene.inspect(&mut render::RenderAccountingContext::default()),
            STRUCTURE,
            "authored-inspection-rejected",
        )? {
            Ok(observed) => observed,
            Err(failure) => return Ok(failure),
        };
        observed.commands += 1;
        compare_summary(recipe.expected(), observed)?
            .ok_or_else(|| error("command observation perturbation escaped comparator"))
    }
}

#[cfg(feature = "property")]
#[test]
fn observer_control_survives_property_capture_and_exact_canonical_replay() {
    use proptest::strategy::Just;
    let target = CommandCountObservationControl::new();
    assert_eq!(
        target.descriptor().id().as_str(),
        CommandCountObservationControl::ID
    );
    let canonical = target
        .normal
        .encode(&target.normal.literal(1).unwrap())
        .unwrap();
    let strategy = Just(1_u64.to_le_bytes().to_vec());
    let mut captures = Vec::new();
    let report = engine::run_property(
        &target,
        &strategy,
        engine::PropertyConfig {
            cases: 1,
            max_rejections: 1,
            max_shrink_iterations: 0,
            seed: 7,
        },
        &mut |bytes| {
            captures.push(bytes.to_vec());
            Ok(())
        },
    )
    .unwrap();
    let failure = report.failure.expect("control must be detected");
    let key = failure.outcome.failure_key().unwrap();
    assert_eq!(key.property().as_str(), STRUCTURE);
    assert_eq!(key.class(), FailureClass::PropertyViolation);
    assert_eq!(key.discriminator(), Some("authored-summary-counts"));
    assert_eq!(failure.canonical, canonical);
    assert!(!captures.is_empty());
    assert!(captures.iter().all(|bytes| bytes == &canonical));
    let mut replay_capture = Vec::new();
    let replay = engine::replay(&target, &canonical, &mut |bytes| {
        replay_capture = bytes.to_vec();
        Ok(())
    })
    .unwrap();
    assert_eq!(replay.outcome.failure_key(), Some(key));
    assert_eq!(replay.canonical, canonical);
    assert_eq!(replay_capture, canonical);
    assert!(
        target
            .normal
            .evaluate(&target.normal.literal(1).unwrap())
            .unwrap()
            .failure_key()
            .is_none()
    );
}

#[test]
fn generation_classifier_propagates_a_real_capacity_refusal_with_its_source() {
    use std::error::Error as _;
    use surgeist_fuzz::{FuzzError, FuzzErrorKind};
    // Capacity overflow is a deterministic fallible-allocation control. This
    // exercises disposition, without claiming an actual bounded request hit OOM.
    let allocation = Vec::<u8>::new().try_reserve_exact(usize::MAX).unwrap_err();
    let cause = FuzzError::with_source(
        FuzzErrorKind::Resource,
        "allocate scene recipe",
        "capacity admission refused",
        allocation,
    );
    let result = generation_outcome(Err(cause));
    let Err(cause) = result else {
        panic!("allocation failure became a generation rejection: {result:?}");
    };
    assert_eq!(cause.kind(), FuzzErrorKind::Resource);
    assert_eq!(cause.operation(), "allocate scene recipe");
    assert_eq!(cause.detail(), "capacity admission refused");
    assert!(
        cause
            .source()
            .unwrap()
            .is::<std::collections::TryReserveError>()
    );
}

#[test]
fn generation_classifier_propagates_a_real_budget_contract_failure() {
    use surgeist_fuzz::{FuzzErrorKind, InputLimits};
    let mut budget = Budget::new(InputLimits::new(1024, 1, 1, 0, 1).unwrap());
    let cause = budget
        .credit(BudgetCharge {
            depth: 1,
            ..BudgetCharge::default()
        })
        .unwrap_err();
    assert_eq!(cause.kind(), FuzzErrorKind::InvalidLimits);
    let expected_operation = cause.operation().to_owned();
    let expected_detail = cause.detail().to_owned();
    let result = generation_outcome(Err(cause));
    let Err(cause) = result else {
        panic!("budget contract failure became a generation rejection: {result:?}");
    };
    assert_eq!(cause.kind(), FuzzErrorKind::InvalidLimits);
    assert_eq!(cause.operation(), expected_operation);
    assert_eq!(cause.detail(), expected_detail);
}

#[test]
fn generation_classifier_success_preserves_the_exact_canonical_case() {
    let target = SceneTarget::new();
    let case = target.literal(1).unwrap();
    let expected = target.encode(&case).unwrap();
    let Generation::Generated(actual) = generation_outcome(Ok(case)).unwrap() else {
        panic!("valid concrete case rejected");
    };
    assert_eq!(target.encode(&actual).unwrap(), expected);
}

#[test]
fn scene_entropy_only_discards_missing_input_and_preserves_admitted_helper_errors() {
    use surgeist_fuzz::{FuzzErrorKind, InputLimits};
    let mut empty = GenerationEntropy::new(&[]);
    assert!(matches!(empty.range(7, 7), Ok(7)));
    assert!(matches!(
        empty.range(0, 1),
        Err(GenerationFailure::EntropyExhausted)
    ));
    match empty.range(9, 1) {
        Err(GenerationFailure::Harness(cause)) => {
            assert_eq!(cause.kind(), FuzzErrorKind::InvalidCase)
        }
        _ => panic!("a malformed range was treated as ordinary entropy exhaustion"),
    }
    let mut budget = Budget::new(InputLimits::new(1, 0, 0, 0, 0).unwrap());
    budget
        .credit(BudgetCharge {
            bytes: 1,
            ..BudgetCharge::default()
        })
        .unwrap();
    let mut available = GenerationEntropy::new(&[23]);
    assert!(matches!(
        available.bytes(1, &mut budget),
        Err(GenerationFailure::Harness(_))
    ));
    assert_eq!(available.0.remaining(), 1);
    assert_eq!(budget.used().bytes, 1);

    let target = SceneTarget::new();
    for length in 0..8 {
        assert!(matches!(
            target.generate(&vec![0; length]).unwrap(),
            Generation::Rejected(_)
        ));
    }
    let Generation::Generated(case) = target.generate(&[0; 8]).unwrap() else {
        panic!("complete selector entropy rejected");
    };
    assert_eq!(
        target.encode(&case).unwrap(),
        target.encode(&target.literal(0).unwrap()).unwrap()
    );
}

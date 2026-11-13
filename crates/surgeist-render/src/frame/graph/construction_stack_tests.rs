use super::*;
use crate::frame::{
    ForcedVelloCaptureMappingForTest, forced_base_graph_with_capture_mapping_for_test,
};
use crate::vello_engine::scene::{
    VelloDrawObservationForTest, VelloFillRuleObservationForTest, VelloPathDrawObservationForTest,
    VelloPathElementObservationForTest,
};
use crate::{
    Capabilities, Color, Layer, RenderAccountingContext, RenderAccountingError,
    RenderAccountingOptions, RenderAttemptKind, RenderAttemptStatus, RenderLimits, RenderPhase,
    RenderQuantity, RenderResource, Scene, Size,
};

fn normalized_scene(depth: u64) -> RenderCommands {
    let mut scene = Scene::new();
    scene.layer(Layer::new().isolated(), |scene| {
        scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    });
    for _ in 0..depth {
        let child = scene;
        scene = Scene::new();
        scene.layer(Layer::new(), |scene| *scene = child);
    }
    scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap()
}

fn frame_context() -> FrameContext {
    FrameContext::try_new(
        Size::new(2.0, 2.0),
        1.0,
        Antialiasing::Area,
        Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

fn normalized_groups(depth: usize) -> RenderCommands {
    let mut scene = Scene::new();
    scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    for index in 0..depth {
        let child = scene;
        scene = Scene::new();
        scene.layer(
            Layer::new()
                .isolated()
                .try_opacity(if index % 2 == 0 { 0.5 } else { 0.75 })
                .unwrap(),
            |scene| *scene = child,
        );
    }
    scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap()
}

fn on_construction_stack(name: &str, stack_size: usize, run: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name(name.into())
        .stack_size(stack_size)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

fn assert_released(context: &RenderAccountingContext) {
    let report = context.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    let usage = report.attempt().unwrap().usage();
    for resource in [
        RenderResource::RetainedCpuBytes,
        RenderResource::ScratchCpuBytes,
    ] {
        assert_eq!(usage.live(resource), 0);
    }
}

fn assert_isolated_rectangle(
    graph: &GpuRenderGraph,
    expected_opacities: &[f32],
    expected_outer_clips: usize,
    account: &Account,
) -> Result<()> {
    let root = graph
        .resources
        .iter()
        .find(|resource| resource.id == graph.root_working_image)
        .unwrap();
    assert_eq!(root.descriptor.role, SemanticResourceRole::RootWorkingImage);
    assert_eq!(
        root.descriptor.logical_bounds.rect(),
        Rect::new(0.0, 0.0, 2.0, 2.0),
    );
    assert_eq!(
        (
            root.descriptor.device_extent.width,
            root.descriptor.device_extent.height,
        ),
        (2, 2),
    );
    assert!(graph.passes.iter().any(|pass| {
        pass.result == SemanticPassResult::Resource(graph.root_working_image)
            && pass.intent
                == SemanticPassIntent::ClearRoot {
                    initialization: WorkingImageInitialization::SurfaceBaseColor(
                        Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
                    ),
                }
    }));

    let [span] = graph.vello_spans.as_slice() else {
        panic!("the isolated rectangle must have one captured source");
    };
    assert_eq!(span.scope, SemanticVelloSpanScope::LayerSource);
    assert_eq!(span.source_space, CompositeSourceSpace::LayerLocal);
    assert_eq!(span.capture_transform, Transform::identity());
    assert_eq!(span.parent_to_surface, Transform::identity());
    let Some((source_over, isolated_groups)) = graph.composites.split_first() else {
        panic!("the isolated source must be composed into its transparent parent");
    };
    assert_eq!(source_over.kind, SemanticCompositeKind::SpanSourceOver);
    assert_eq!(isolated_groups.len(), expected_opacities.len());
    for (isolated, expected_opacity) in isolated_groups.iter().zip(expected_opacities) {
        let SemanticCompositeKind::Layer {
            transform,
            opacity,
            composition,
            clip,
            outer_clips,
            alpha_mask,
            ..
        } = &isolated.kind
        else {
            panic!("the captured source must retain its isolated outer composition");
        };
        assert_eq!(*transform, Transform::identity());
        assert_eq!(opacity, expected_opacity);
        assert!(composition.is_source_over_normal());
        assert!(clip.is_none() && alpha_mask.is_none());
        assert_eq!(outer_clips.len(), expected_outer_clips);
        assert!(
            outer_clips
                .iter()
                .all(|clip| clip.transform == Transform::identity())
        );
    }

    let encoded = crate::encode::encode_vello_scene_with_initial_transform(
        &span.commands,
        Transform::identity(),
        PhysicalSize::new(2, 2),
        account,
    )?;
    use VelloPathElementObservationForTest::{Close, LineTo, MoveTo};
    assert_eq!(
        encoded
            .observation_for_test()
            .solid_path_draws_for_test()
            .unwrap(),
        [VelloPathDrawObservationForTest {
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            fill_rule: VelloFillRuleObservationForTest::NonZero,
            geometry: vec![
                MoveTo([0.0, 0.0]),
                LineTo([2.0, 0.0]),
                LineTo([2.0, 2.0]),
                LineTo([0.0, 2.0]),
                Close,
            ],
            draw: VelloDrawObservationForTest::SolidColor { rgba: 0xff00_0000 },
        }],
    );
    Ok(())
}

fn construct(commands: RenderCommands, expected_opacities: &[f32]) {
    let mut context = RenderAccountingContext::default();
    context
        .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
            // This private ingress constructs and validates the real graph
            // without the outer semantic contribution/selection walks.
            let graph = forced_base_graph_with_capture_mapping_for_test(
                commands,
                frame_context(),
                ForcedVelloCaptureMappingForTest::identity(),
                account,
            )?;
            assert_isolated_rectangle(&graph, expected_opacities, 0, account)
        })
        .unwrap();
    assert_released(&context);
    assert_eq!(
        context.last_attempt().unwrap().status(),
        RenderAttemptStatus::Succeeded,
    );
}

#[test]
fn shallow_wrappers_preserve_the_isolated_graph_source() {
    on_construction_stack("construct wrapped isolated graph", 2 * 1024 * 1024, || {
        construct(normalized_scene(2), &[1.0]);
    });
}

#[test]
fn deep_wrappers_preserve_the_isolated_graph_source() {
    on_construction_stack("construct wrapped isolated graph", 2 * 1024 * 1024, || {
        construct(normalized_scene(8_192), &[1.0]);
    });
}

fn refuse_first_command(commands: RenderCommands) {
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CommandVisits, 0)
        .unwrap();
    let mut context =
        RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
    let error = context
        .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
            forced_base_graph_with_capture_mapping_for_test(
                commands,
                frame_context(),
                ForcedVelloCaptureMappingForTest::identity(),
                account,
            )
            .map(drop)
        })
        .unwrap_err();
    let report = context.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    let attempt = report.attempt().unwrap();
    let refusal = attempt.refusal().unwrap();
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(refusal)),
    );
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::CommandVisits),
    );
    assert_eq!((refusal.current(), refusal.amount()), (0, 1));
    assert_eq!(attempt.usage().work(RenderWork::CommandVisits), 0);
    assert_eq!(attempt.usage().work(RenderWork::PaintVisits), 0);
    assert!(attempt.usage().work(RenderWork::GraphNodeVisits) > 0);
    assert_released(&context);
}

#[test]
fn deep_first_command_refusal_releases_graph_setup() {
    on_construction_stack("refuse wrapped isolated graph", 2 * 1024 * 1024, || {
        refuse_first_command(normalized_scene(8_192));
    });
}

#[test]
fn shallow_isolated_groups_preserve_inner_to_outer_opacity_order() {
    on_construction_stack("construct nested isolated groups", 512 * 1024, || {
        construct(normalized_groups(2), &[0.5, 0.75]);
    });
}

#[test]
fn deep_isolated_groups_preserve_inner_to_outer_opacity_order() {
    on_construction_stack("construct nested isolated groups", 512 * 1024, || {
        construct(normalized_groups(128), &[0.5, 0.75].repeat(64));
    });
}

#[test]
fn deep_group_first_command_refusal_releases_graph_setup() {
    on_construction_stack("refuse nested isolated groups", 512 * 1024, || {
        refuse_first_command(normalized_groups(128));
    });
}

#[test]
fn wrapper_and_clip_returns_restore_sibling_coordinates_clips_and_parent_locality() {
    let wrapper_transform = Transform::translation(0.25, 0.0).unwrap();
    let clip_transform = Transform::translation(0.0, 0.5).unwrap();
    let combined = Transform::translation(0.25, 0.5).unwrap();
    let rect = Rect::new(0.0, 0.0, 1.0, 1.0);
    let mut scene = Scene::new();
    scene.layer(
        Layer::new().isolated().try_opacity(0.25).unwrap(),
        |scene| {
            scene.layer(Layer::new().with_transform(wrapper_transform), |scene| {
                scene.layer(
                    Layer::new()
                        .with_transform(clip_transform)
                        .try_clip(rect.into())
                        .unwrap(),
                    |scene| {
                        scene.fill(rect, Color::BLACK);
                        scene.layer(Layer::new().isolated().try_opacity(0.5).unwrap(), |scene| {
                            scene.fill(rect, Color::BLACK);
                        });
                        scene.fill(rect, Color::BLACK);
                    },
                );
                scene.layer(Layer::new().blend(crate::BlendMode::Multiply), |scene| {
                    scene.fill(rect, Color::BLACK);
                });
            });
            scene.layer(Layer::new().blend(crate::BlendMode::Multiply), |scene| {
                scene.fill(rect, Color::BLACK);
            });
        },
    );
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let mut context = RenderAccountingContext::default();
    context
        .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
            let graph = forced_base_graph_with_capture_mapping_for_test(
                commands,
                frame_context(),
                ForcedVelloCaptureMappingForTest::identity(),
                account,
            )?;
            // The two ordinary spans around the inner isolated group remain under
            // the clip. Each separately captured source starts in its own coordinates.
            let expected_captures = [
                combined,
                Transform::identity(),
                combined,
                Transform::identity(),
                Transform::identity(),
            ];
            let expected_parents = [
                Transform::identity(),
                combined,
                Transform::identity(),
                wrapper_transform,
                Transform::identity(),
            ];
            assert_eq!(graph.vello_spans.len(), expected_captures.len());
            for ((span, capture), parent) in graph
                .vello_spans
                .iter()
                .zip(expected_captures)
                .zip(expected_parents)
            {
                assert_eq!(span.scope, SemanticVelloSpanScope::LayerSource);
                assert_eq!(span.capture_transform, capture);
                assert_eq!(span.parent_to_surface, parent);
            }
            // Returning from either wrapper leaves the actual parent external:
            // both subsequent multiply operations require separate group capture.
            let expected = [
                (1.0, Transform::identity(), true, false),
                (1.0, Transform::identity(), false, false),
                (0.5, combined, true, false),
                (1.0, Transform::identity(), true, false),
                (1.0, Transform::identity(), false, false),
                (1.0, wrapper_transform, false, true),
                (1.0, Transform::identity(), false, false),
                (1.0, Transform::identity(), false, true),
                (0.25, Transform::identity(), false, false),
            ];
            assert_eq!(graph.composites.len(), expected.len());
            for (composite, (expected_opacity, expected_transform, clipped, multiply)) in
                graph.composites.iter().zip(expected)
            {
                assert!(composite.source_captured_before_outer_semantics);
                assert_eq!(
                    matches!(composite.kind, SemanticCompositeKind::SpanSourceOver),
                    expected_opacity == 1.0 && !clipped && !multiply,
                );
                match &composite.kind {
                    SemanticCompositeKind::SpanSourceOver => {
                        assert_eq!(
                            (expected_opacity, expected_transform, clipped, multiply),
                            (1.0, Transform::identity(), false, false)
                        );
                    }
                    SemanticCompositeKind::Layer {
                        transform,
                        opacity,
                        composition,
                        clip,
                        outer_clips,
                        alpha_mask,
                        ..
                    } => {
                        assert_eq!(*transform, expected_transform);
                        assert_eq!(*opacity, expected_opacity);
                        assert_eq!(
                            *composition,
                            crate::layer::LayerComposition::from_blend(if multiply {
                                crate::BlendMode::Multiply
                            } else {
                                crate::BlendMode::Normal
                            })
                        );
                        assert!(clip.is_none() && alpha_mask.is_none());
                        assert_eq!(outer_clips.len(), usize::from(clipped));
                        if clipped {
                            assert_eq!(outer_clips[0].transform, combined);
                            assert_eq!(
                                outer_clips[0].clip.geometry(),
                                &crate::command::RenderClipGeometry::Rect(rect)
                            );
                        }
                    }
                    SemanticCompositeKind::DropShadow => panic!("no shadow was authored"),
                }
            }
            Ok(())
        })
        .unwrap();
    assert_released(&context);
}

fn run_for_retry(context: &mut RenderAccountingContext, commands: &RenderCommands) -> Result<()> {
    context.run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
        forced_base_graph_with_capture_mapping_for_test(
            commands.clone(),
            frame_context(),
            ForcedVelloCaptureMappingForTest::identity(),
            account,
        )
        .map(drop)
    })
}

#[test]
fn pending_wrapper_and_group_work_refusal_releases_storage_and_retries() {
    for commands in [normalized_scene(32), normalized_groups(12)] {
        let mut context = RenderAccountingContext::default();
        run_for_retry(&mut context, &commands).unwrap();
        assert_released(&context);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        let command_visits = usage.work(RenderWork::CommandVisits);
        let coordinate_visits = usage.work(RenderWork::CoordinateVisits);
        let scratch_peak = usage.peak(RenderResource::ScratchCpuBytes);
        assert!(scratch_peak > 0);
        for (work, total) in [
            (RenderWork::CommandVisits, command_visits),
            (RenderWork::CoordinateVisits, coordinate_visits),
        ] {
            let limit = total - 1;
            context.set_options(
                RenderAccountingOptions::new()
                    .with_limits(RenderLimits::new().with_work_limit(work, limit).unwrap()),
            );
            let error = run_for_retry(&mut context, &commands).unwrap_err();
            let report = context.last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            let attempt = report.attempt().unwrap();
            let refusal = attempt.refusal().unwrap();
            assert_eq!(
                error.accounting_diagnostic(),
                Some(RenderAccountingError::Refused(refusal))
            );
            assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
            if work == RenderWork::CommandVisits {
                assert_eq!((refusal.current(), refusal.amount()), (limit, 1));
            } else {
                assert!(refusal.current() <= limit);
                assert!(refusal.current() + refusal.amount() > limit);
            }
            // These last command/coordinate visits occur while inner source planning
            // still has its ancestor wrapper/group returns pending.
            assert_eq!(
                attempt.usage().peak(RenderResource::ScratchCpuBytes),
                scratch_peak
            );
            assert_released(&context);
        }
        context.set_options(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_scratch_bytes(scratch_peak - 1)),
        );
        let error = run_for_retry(&mut context, &commands).unwrap_err();
        let report = context.last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        let refusal = report.attempt().unwrap().refusal().unwrap();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
        );
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Resource(RenderResource::ScratchCpuBytes)
        );
        assert!(refusal.current() > 0 && refusal.amount() > 0);
        assert_released(&context);

        let limits = RenderLimits::new()
            .with_scratch_bytes(scratch_peak)
            .with_work_limit(RenderWork::CommandVisits, command_visits)
            .unwrap()
            .with_work_limit(RenderWork::CoordinateVisits, coordinate_visits)
            .unwrap();
        context.set_options(RenderAccountingOptions::new().with_limits(limits));
        run_for_retry(&mut context, &commands).unwrap();
        assert_released(&context);
        assert_eq!(
            context.last_attempt().unwrap().status(),
            RenderAttemptStatus::Succeeded
        );
    }
}

#[test]
fn deep_clip_wrappers_preserve_all_outer_clips_on_the_isolated_group() {
    on_construction_stack(
        "construct deeply clipped isolated graph",
        512 * 1024,
        || {
            let mut scene = Scene::new();
            let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
            scene.layer(Layer::new().isolated(), |scene| {
                scene.fill(rect, Color::BLACK);
            });
            for _ in 0..512 {
                let child = scene;
                scene = Scene::new();
                scene.layer(Layer::new().try_clip(rect.into()).unwrap(), |scene| {
                    *scene = child
                });
            }
            let commands = scene
                .normalize(Capabilities::CURRENT, &Account::for_test())
                .unwrap();
            let mut context = RenderAccountingContext::default();
            context
                .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                    let graph = forced_base_graph_with_capture_mapping_for_test(
                        commands,
                        frame_context(),
                        ForcedVelloCaptureMappingForTest::identity(),
                        account,
                    )?;
                    assert_isolated_rectangle(&graph, &[1.0], 512, account)
                })
                .unwrap();
            assert_released(&context);
        },
    );
}

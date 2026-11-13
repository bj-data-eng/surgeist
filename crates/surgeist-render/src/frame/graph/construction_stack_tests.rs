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
        assert!(clip.is_none() && outer_clips.is_empty() && alpha_mask.is_none());
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
            assert_isolated_rectangle(&graph, expected_opacities, account)
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

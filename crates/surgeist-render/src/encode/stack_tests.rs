use super::*;
use crate::vello_engine::scene::{
    VelloDrawObservationForTest, VelloFillRuleObservationForTest, VelloPathDrawObservationForTest,
    VelloPathElementObservationForTest,
};

fn normalized_rectangle(depth: u64) -> RenderCommands {
    let mut scene = Scene::new();
    scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    for _ in 0..depth {
        let child = scene;
        scene = Scene::new();
        scene.layer(Layer::new(), |scene| *scene = child);
    }
    scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap()
}

fn on_encoder_stack(name: &str, run: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name(name.into())
        .stack_size(2 * 1024 * 1024)
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

fn rectangle_geometry(width: f32, height: f32) -> Vec<VelloPathElementObservationForTest> {
    use VelloPathElementObservationForTest::{Close, LineTo, MoveTo};
    vec![
        MoveTo([0.0, 0.0]),
        LineTo([width, 0.0]),
        LineTo([width, height]),
        LineTo([0.0, height]),
        Close,
    ]
}

fn expected_rectangle(transform: [f32; 6]) -> VelloPathDrawObservationForTest {
    VelloPathDrawObservationForTest {
        transform,
        fill_rule: VelloFillRuleObservationForTest::NonZero,
        geometry: rectangle_geometry(2.0, 2.0),
        // The native draw stream packs premultiplied RGBA with alpha in the
        // most significant byte.
        draw: VelloDrawObservationForTest::SolidColor { rgba: 0xff00_0000 },
    }
}

fn encode_and_check(
    context: &mut RenderAccountingContext,
    commands: &RenderCommands,
    check: impl FnOnce(&[VelloPathDrawObservationForTest]),
) -> Result<()> {
    context.run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
        let encoded = encode_vello_scene_with_initial_transform(
            commands,
            Transform::identity(),
            PhysicalSize::new(16, 16),
            account,
        )?;
        let draws = encoded
            .observation_for_test()
            .solid_path_draws_for_test()
            .unwrap();
        check(&draws);
        Ok(())
    })
}

fn encode_rectangle(depth: u64) {
    on_encoder_stack("encode wrapped rectangle", move || {
        let commands = normalized_rectangle(depth);
        let leaf = normalized_rectangle(0);
        let mut context = RenderAccountingContext::default();
        context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                encode_vello_scene_with_initial_transform(
                    &leaf,
                    Transform::identity(),
                    PhysicalSize::new(2, 2),
                    account,
                )
                .map(drop)
            })
            .unwrap();
        assert_released(&context);
        let leaf_scratch = context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .peak(RenderResource::ScratchCpuBytes);
        // The same leaf's existing native geometry scratch suffices at every
        // wrapper depth; a chain has no pending sibling or layer exit.
        context.set_options(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_scratch_bytes(leaf_scratch)),
        );
        context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                let encoded = encode_vello_scene_with_initial_transform(
                    &commands,
                    Transform::identity(),
                    PhysicalSize::new(2, 2),
                    account,
                )?;
                assert_eq!(
                    encoded
                        .observation_for_test()
                        .solid_path_draws_for_test()
                        .unwrap(),
                    [expected_rectangle([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])],
                );
                Ok(())
            })
            .unwrap();
        assert_released(&context);
        let report = context.last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        let usage = report.attempt().unwrap().usage();
        // Each wrapper and the one leaf are encoded exactly once.
        assert_eq!(usage.work(RenderWork::CommandVisits), depth + 1);
        assert!(usage.peak(RenderResource::RetainedCpuBytes) > 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), leaf_scratch);
    });
}

fn branching_commands() -> RenderCommands {
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut scene = Scene::new();
    scene.fill(rect, Color::BLACK);
    scene.layer(
        Layer::new().with_transform(Transform::translation(3.0, 4.0).unwrap()),
        |scene| {
            scene.fill(rect, Color::BLACK);
            scene.layer(
                Layer::new().with_transform(Transform::scale(2.0, 3.0).unwrap()),
                |scene| {
                    scene.fill(rect, Color::BLACK);
                },
            );
            scene.fill(rect, Color::BLACK);
        },
    );
    scene.fill(rect, Color::BLACK);
    scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap()
}

fn assert_branching_draws(draws: &[VelloPathDrawObservationForTest]) {
    assert_eq!(
        draws,
        [
            expected_rectangle([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            expected_rectangle([1.0, 0.0, 0.0, 1.0, 3.0, 4.0]),
            expected_rectangle([2.0, 0.0, 0.0, 3.0, 3.0, 4.0]),
            expected_rectangle([1.0, 0.0, 0.0, 1.0, 3.0, 4.0]),
            expected_rectangle([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        ],
    );
}

#[test]
fn branching_wrappers_restore_each_parent_transform_in_authored_order() {
    let commands = branching_commands();
    let mut context = RenderAccountingContext::default();
    encode_and_check(&mut context, &commands, assert_branching_draws).unwrap();
    assert_released(&context);
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CommandVisits),
        7,
    );
}

fn nested_layer_commands() -> RenderCommands {
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut scene = Scene::new();
    scene.fill(rect, Color::BLACK);
    scene.layer(
        Layer::new()
            .with_transform(Transform::translation(2.0, 3.0).unwrap())
            .try_clip(rect.into())
            .unwrap()
            .try_opacity(0.5)
            .unwrap()
            .with_composition(LayerComposition::from_blend(BlendMode::Multiply)),
        |scene| {
            scene.fill(rect, Color::BLACK);
            scene.layer(
                Layer::new()
                    .with_transform(Transform::scale(2.0, 2.0).unwrap())
                    .try_clip(Rect::new(0.0, 0.0, 1.0, 1.0).into())
                    .unwrap(),
                |scene| {
                    scene.fill(rect, Color::BLACK);
                },
            );
            scene.fill(rect, Color::BLACK);
        },
    );
    scene.fill(rect, Color::BLACK);
    scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap()
}

fn assert_nested_layer_draws(draws: &[VelloPathDrawObservationForTest]) {
    use VelloDrawObservationForTest::{BeginClip, EndClip, SolidColor};
    let multiply = vello_encoding::DrawBeginClip::new(peniko::Mix::Multiply.into(), 0.5);
    assert_eq!(
        draws.iter().map(|draw| draw.draw).collect::<Vec<_>>(),
        [
            SolidColor { rgba: 0xff00_0000 },
            BeginClip {
                blend_mode: multiply.blend_mode,
                alpha: 0.5,
            },
            SolidColor { rgba: 0xff00_0000 },
            BeginClip {
                blend_mode: vello_encoding::DrawBeginClip::CLIP_BLEND_MODE,
                alpha: 1.0,
            },
            SolidColor { rgba: 0xff00_0000 },
            EndClip,
            SolidColor { rgba: 0xff00_0000 },
            EndClip,
            SolidColor { rgba: 0xff00_0000 },
        ],
    );
    for (index, transform) in [
        (0, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        (2, [1.0, 0.0, 0.0, 1.0, 2.0, 3.0]),
        (4, [2.0, 0.0, 0.0, 2.0, 2.0, 3.0]),
        (6, [1.0, 0.0, 0.0, 1.0, 2.0, 3.0]),
        (8, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
    ] {
        assert_eq!(draws[index], expected_rectangle(transform));
    }
    for (index, side, transform) in [
        (1, 2.0, [1.0, 0.0, 0.0, 1.0, 2.0, 3.0]),
        (3, 1.0, [2.0, 0.0, 0.0, 2.0, 2.0, 3.0]),
    ] {
        assert_eq!(draws[index].geometry, rectangle_geometry(side, side));
        assert_eq!(draws[index].transform, transform);
        assert_eq!(
            draws[index].fill_rule,
            VelloFillRuleObservationForTest::NonZero
        );
    }
    for index in [5, 7] {
        assert!(draws[index].geometry.is_empty());
    }
}

#[test]
fn nested_clip_and_opacity_blend_layers_exit_before_following_siblings() {
    let commands = nested_layer_commands();
    let mut context = RenderAccountingContext::default();
    encode_and_check(&mut context, &commands, assert_nested_layer_draws).unwrap();
    assert_released(&context);
}

#[test]
fn empty_layers_finish_before_the_following_parent_fill() {
    let mut scene = Scene::new();
    scene.layer(
        Layer::new().with_transform(Transform::translation(3.0, 4.0).unwrap()),
        |_| {},
    );
    scene.layer(
        Layer::new()
            .with_transform(Transform::translation(1.0, 2.0).unwrap())
            .try_clip(Rect::new(0.0, 0.0, 2.0, 2.0).into())
            .unwrap(),
        |_| {},
    );
    scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let mut context = RenderAccountingContext::default();
    encode_and_check(&mut context, &commands, |draws| {
        assert_eq!(draws.len(), 3);
        assert_eq!(
            draws[0],
            VelloPathDrawObservationForTest {
                transform: [1.0, 0.0, 0.0, 1.0, 1.0, 2.0],
                fill_rule: VelloFillRuleObservationForTest::NonZero,
                geometry: rectangle_geometry(2.0, 2.0),
                draw: VelloDrawObservationForTest::BeginClip {
                    blend_mode: vello_encoding::DrawBeginClip::CLIP_BLEND_MODE,
                    alpha: 1.0,
                },
            },
        );
        assert_eq!(draws[1].draw, VelloDrawObservationForTest::EndClip);
        assert!(draws[1].geometry.is_empty());
        assert_eq!(draws[2], expected_rectangle([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),);
    })
    .unwrap();
    assert_released(&context);
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CommandVisits),
        3,
    );
}

#[test]
fn suspended_layers_release_after_work_and_scratch_refusals_and_retry() {
    let commands = nested_layer_commands();
    let mut context = RenderAccountingContext::default();
    encode_and_check(&mut context, &commands, assert_nested_layer_draws).unwrap();
    assert_released(&context);
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    // Two layers and five fills; the two matching exits are not commands.
    assert_eq!(usage.work(RenderWork::CommandVisits), 7);
    let scratch = usage.peak(RenderResource::ScratchCpuBytes);
    assert!(scratch > 0);

    // Four visits reach the inner child with both layer exits pending; six
    // visits reach the last root fill after both exits have completed.
    for cap in [4, 6] {
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CommandVisits, cap)
            .unwrap();
        context.set_options(RenderAccountingOptions::new().with_limits(limits));
        let error = encode_and_check(&mut context, &commands, |_| {}).unwrap_err();
        let report = context.last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        let refusal = report.attempt().unwrap().refusal().unwrap();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal)),
        );
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::CommandVisits)
        );
        assert_eq!((refusal.current(), refusal.amount()), (cap, 1));
        assert_released(&context);
    }

    context.set_options(
        RenderAccountingOptions::new()
            .with_limits(RenderLimits::new().with_scratch_bytes(scratch - 1)),
    );
    let error = encode_and_check(&mut context, &commands, |_| {}).unwrap_err();
    let report = context.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    let refusal = report.attempt().unwrap().refusal().unwrap();
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(refusal)),
    );
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
    );
    assert_released(&context);

    let limits = RenderLimits::new()
        .with_scratch_bytes(scratch)
        .with_work_limit(RenderWork::CommandVisits, 7)
        .unwrap();
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    encode_and_check(&mut context, &commands, assert_nested_layer_draws).unwrap();
    assert_released(&context);
}

#[test]
fn pending_sibling_growth_refuses_before_leaf_encoding_and_retries() {
    const DEPTH: u64 = 128;
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut scene = Scene::new();
    scene.fill(rect, Color::BLACK);
    for _ in 0..DEPTH {
        let child = scene;
        scene = Scene::new();
        scene.layer(Layer::new(), |scene| *scene = child);
        scene.fill(rect, Color::BLACK);
    }
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let check = |draws: &[VelloPathDrawObservationForTest]| {
        assert_eq!(draws.len(), 129);
        for draw in draws {
            assert_eq!(*draw, expected_rectangle([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]));
        }
    };
    let mut context = RenderAccountingContext::default();
    encode_and_check(&mut context, &commands, check).unwrap();
    assert_released(&context);
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    // Every layer has one following fill, in addition to the deepest fill.
    let visits = 2 * DEPTH + 1;
    assert_eq!(usage.work(RenderWork::CommandVisits), visits);
    let scratch = usage.peak(RenderResource::ScratchCpuBytes);

    context.set_options(
        RenderAccountingOptions::new()
            .with_limits(RenderLimits::new().with_scratch_bytes(scratch - 1)),
    );
    let error = encode_and_check(&mut context, &commands, |_| {}).unwrap_err();
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
        RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
    );
    // Pending parent storage has already relocated, but its next growth is
    // refused before the first native path or paint can allocate scratch.
    let usage = attempt.usage();
    assert!(usage.work(RenderWork::StorageRelocations) > 0);
    assert_eq!(usage.work(RenderWork::PathElementVisits), 0);
    assert_eq!(usage.work(RenderWork::PaintVisits), 0);
    assert_released(&context);

    let limits = RenderLimits::new()
        .with_scratch_bytes(scratch)
        .with_work_limit(RenderWork::CommandVisits, visits)
        .unwrap();
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    encode_and_check(&mut context, &commands, check).unwrap();
    assert_released(&context);
}

#[test]
fn shallow_default_wrappers_preserve_encoded_rectangle() {
    encode_rectangle(2);
}

#[test]
fn deep_default_wrappers_preserve_encoded_rectangle() {
    encode_rectangle(8_192);
}

#[test]
fn deep_zero_command_budget_refuses_before_encoding() {
    on_encoder_stack("refuse wrapped rectangle encoding", || {
        let commands = normalized_rectangle(8_192);
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CommandVisits, 0)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                encode_vello_scene_with_initial_transform(
                    &commands,
                    Transform::identity(),
                    PhysicalSize::new(2, 2),
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
        for work in [
            RenderWork::CommandVisits,
            RenderWork::CoordinateVisits,
            RenderWork::PathElementVisits,
            RenderWork::PaintVisits,
        ] {
            assert_eq!(attempt.usage().work(work), 0);
        }
        assert_released(&context);
    });
}

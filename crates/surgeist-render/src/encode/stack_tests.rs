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

fn encode_rectangle(depth: u64) {
    on_encoder_stack("encode wrapped rectangle", move || {
        let commands = normalized_rectangle(depth);
        let mut context = RenderAccountingContext::default();
        context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                let encoded = encode_vello_scene_with_initial_transform(
                    &commands,
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
                        // The native draw stream packs premultiplied RGBA
                        // with alpha in the most significant byte.
                        draw: VelloDrawObservationForTest::SolidColor { rgba: 0xff00_0000 },
                    }],
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
    });
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

use super::*;
use crate::{
    BlendMode, Capabilities, Layer, LayerComposition, PorterDuffOperator, RenderAccountingContext,
    RenderAccountingError, RenderAccountingOptions, RenderAttemptKind, RenderAttemptStatus,
    RenderLimits, RenderPhase, RenderQuantity, RenderResource, Scene,
};

fn scan(context: &mut RenderAccountingContext, commands: &RenderCommands) -> Result<bool> {
    context.run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
        commands_have_bounded_operation(commands, account)
    })
}

fn assert_clean(context: &RenderAccountingContext) {
    let report = context.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
}

#[test]
fn branching_scan_stops_at_first_bounded_operation_and_retries_after_refusal() {
    let mut scene = Scene::new();
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    scene.layer(Layer::new(), |scene| {
        scene.fill(rect, Color::BLACK);
        scene.layer(Layer::new(), |scene| {
            scene.fill(rect, Color::BLACK);
            scene.layer(
                Layer::new().with_composition(
                    LayerComposition::try_porter_duff(
                        PorterDuffOperator::SourceIn,
                        BlendMode::Normal,
                    )
                    .unwrap(),
                ),
                |scene| {
                    scene.fill(rect, Color::BLACK);
                },
            );
            scene.fill(rect, Color::BLACK);
        });
        scene.fill(rect, Color::BLACK);
    });
    scene.fill(rect, Color::BLACK);
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let mut context = RenderAccountingContext::default();
    assert!(scan(&mut context, &commands).unwrap());
    assert_clean(&context);
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    // Outer wrapper, first fill, inner wrapper, inner fill, SourceIn.
    // The SourceIn child and all following siblings are not inspected.
    assert_eq!(usage.work(RenderWork::CommandVisits), 5);
    let scratch = usage.peak(RenderResource::ScratchCpuBytes);
    assert!(scratch > 0);

    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CommandVisits, 4)
        .unwrap();
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    let error = scan(&mut context, &commands).unwrap_err();
    let report = context.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    let attempt = report.attempt().unwrap();
    let refusal = attempt.refusal().unwrap();
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(refusal))
    );
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::CommandVisits)
    );
    assert_eq!(refusal.current(), 4);
    assert_eq!(refusal.amount(), 1);
    assert_clean(&context);

    context.set_options(
        RenderAccountingOptions::new()
            .with_limits(RenderLimits::new().with_scratch_bytes(scratch - 1)),
    );
    let error = scan(&mut context, &commands).unwrap_err();
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
    assert_clean(&context);

    let limits = RenderLimits::new()
        .with_scratch_bytes(scratch)
        .with_work_limit(RenderWork::CommandVisits, 5)
        .unwrap();
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    assert!(scan(&mut context, &commands).unwrap());
    assert_clean(&context);
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn deep_unbounded_chain_scans_every_layer_without_pending_sibling_backing() {
    std::thread::Builder::new()
        .name("scan normalized layer chain".into())
        .stack_size(512 * 1024)
        .spawn(|| {
            const DEPTH: u64 = 8_192;
            let mut scene = Scene::new();
            for _ in 0..DEPTH {
                let child = scene;
                scene = Scene::new();
                scene.layer(Layer::new(), |scene| *scene = child);
            }
            let commands = scene
                .normalize(Capabilities::CURRENT, &Account::for_test())
                .unwrap();
            let mut context = RenderAccountingContext::default();
            assert!(!scan(&mut context, &commands).unwrap());
            let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::CommandVisits), DEPTH);
            assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
            assert_clean(&context);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn branching_unbounded_scan_resumes_all_pending_siblings() {
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut scene = Scene::new();
    scene.layer(Layer::new(), |scene| {
        scene.fill(rect, Color::BLACK);
        scene.layer(Layer::new(), |scene| {
            scene.fill(rect, Color::BLACK);
        });
        scene.fill(rect, Color::BLACK);
    });
    scene.fill(rect, Color::BLACK);
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let mut context = RenderAccountingContext::default();
    assert!(!scan(&mut context, &commands).unwrap());
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::CommandVisits), 6);
    assert!(usage.peak(RenderResource::ScratchCpuBytes) > 0);
    assert_clean(&context);
}

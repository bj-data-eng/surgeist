use super::*;
use crate::{
    BlendMode, Capabilities, Color, Image, ImageRendering, Layer, RenderAccountingContext,
    RenderAccountingError, RenderAccountingOptions, RenderAttemptKind, RenderLimits, RenderPhase,
    RenderQuantity, RenderResource, Scene,
};

#[derive(Clone, Copy)]
enum Query {
    SurfaceGrid,
    CompositionDomain,
    CaptureLocality,
}

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

fn query(commands: &RenderCommands, query: Query, account: &Account) -> Result<bool> {
    match query {
        Query::SurfaceGrid => commands_require_surface_grid(commands, account),
        Query::CompositionDomain => commands_require_composition_domain(commands, account),
        Query::CaptureLocality => {
            let [command] = commands.as_ref() else {
                panic!("expected one outer wrapper");
            };
            command_is_local_to_capture(command, CaptureParentLocality::External, account)
        }
    }
}

fn check(depth: u64, selected: Query) {
    std::thread::Builder::new()
        .name("query graph capture descendants".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let commands = normalized_scene(depth);
            let mut context = RenderAccountingContext::default();
            let result = context
                .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                    query(&commands, selected, account)
                })
                .unwrap();
            assert_eq!(result, !matches!(selected, Query::CaptureLocality));
            let report = context.last_attempt().unwrap();
            assert_eq!(report.cleanup_fault(), None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::CommandVisits), depth + 1);
            assert_eq!(usage.work(RenderWork::PaintVisits), 0);
            assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
            for resource in [
                RenderResource::ScratchCpuBytes,
                RenderResource::RetainedCpuBytes,
            ] {
                assert_eq!(usage.live(resource), 0);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn shallow_surface_grid_query_finds_isolated_descendant() {
    check(2, Query::SurfaceGrid);
}

#[test]
fn deep_surface_grid_query_finds_isolated_descendant() {
    check(8_192, Query::SurfaceGrid);
}

#[test]
fn shallow_composition_domain_query_finds_isolated_descendant() {
    check(2, Query::CompositionDomain);
}

#[test]
fn deep_composition_domain_query_finds_isolated_descendant() {
    check(8_192, Query::CompositionDomain);
}

#[test]
fn shallow_capture_locality_query_finds_external_descendant() {
    check(2, Query::CaptureLocality);
}

#[test]
fn deep_capture_locality_query_finds_external_descendant() {
    check(8_192, Query::CaptureLocality);
}

#[test]
fn deep_capture_queries_admit_the_first_visit_before_descent() {
    let commands = normalized_scene(8_192);
    for selected in [
        Query::SurfaceGrid,
        Query::CompositionDomain,
        Query::CaptureLocality,
    ] {
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CommandVisits, 0)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                query(&commands, selected, account)
            })
            .unwrap_err();
        let attempt = context.last_attempt().unwrap().attempt().unwrap();
        let refusal = attempt.refusal().unwrap();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
        );
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::CommandVisits)
        );
        assert_eq!(refusal.current(), 0);
        assert_eq!(refusal.amount(), 1);
    }
}

fn assert_released(context: &RenderAccountingContext) {
    let report = context.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
}

fn run(
    context: &mut RenderAccountingContext,
    commands: &RenderCommands,
    selected: Query,
) -> Result<bool> {
    context.run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
        query(commands, selected, account)
    })
}

#[test]
fn branching_capture_queries_complete_and_retry_after_refusal() {
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut scene = Scene::new();
    scene.layer(Layer::new(), |scene| {
        for _ in 0..2 {
            scene.layer(Layer::new(), |scene| {
                scene.fill(rect, Color::BLACK);
            });
        }
    });
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    for selected in [
        Query::SurfaceGrid,
        Query::CompositionDomain,
        Query::CaptureLocality,
    ] {
        let expected = matches!(selected, Query::CaptureLocality);
        let mut context = RenderAccountingContext::default();
        assert_eq!(run(&mut context, &commands, selected).unwrap(), expected);
        assert_released(&context);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CommandVisits), 5);
        let paints = if matches!(selected, Query::SurfaceGrid) {
            2
        } else {
            0
        };
        assert_eq!(usage.work(RenderWork::PaintVisits), paints);
        let scratch = usage.peak(RenderResource::ScratchCpuBytes);
        assert!(scratch > 0);

        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CommandVisits, 4)
            .unwrap();
        context.set_options(RenderAccountingOptions::new().with_limits(limits));
        let error = run(&mut context, &commands, selected).unwrap_err();
        let refusal = context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .refusal()
            .unwrap();
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
        assert_released(&context);

        context.set_options(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_scratch_bytes(scratch - 1)),
        );
        let error = run(&mut context, &commands, selected).unwrap_err();
        let refusal = context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .refusal()
            .unwrap();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
        );
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Resource(RenderResource::ScratchCpuBytes)
        );
        assert_released(&context);

        let limits = RenderLimits::new()
            .with_scratch_bytes(scratch)
            .with_work_limit(RenderWork::CommandVisits, 5)
            .unwrap()
            .with_work_limit(RenderWork::PaintVisits, paints)
            .unwrap();
        context.set_options(RenderAccountingOptions::new().with_limits(limits));
        assert_eq!(run(&mut context, &commands, selected).unwrap(), expected);
        assert_released(&context);
    }
}

#[test]
fn capture_locality_restores_the_actual_parent_for_following_siblings() {
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut scene = Scene::new();
    scene.layer(Layer::new(), |scene| {
        scene.layer(Layer::new().try_opacity(0.5).unwrap(), |scene| {
            scene.layer(Layer::new().blend(BlendMode::Multiply), |scene| {
                scene.fill(rect, Color::BLACK);
            });
        });
        scene.layer(Layer::new().blend(BlendMode::Multiply), |scene| {
            scene.fill(rect, Color::BLACK);
        });
    });
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let [command] = commands.as_ref() else {
        panic!("expected one outer wrapper");
    };
    for (locality, expected, visits) in [
        (CaptureParentLocality::External, false, 5),
        (CaptureParentLocality::CaptureLocal, true, 6),
    ] {
        let mut context = RenderAccountingContext::default();
        let result = context
            .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                command_is_local_to_capture(command, locality, account)
            })
            .unwrap();
        assert_eq!(result, expected);
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CommandVisits),
            visits
        );
        assert_released(&context);
    }
}

#[test]
fn surface_grid_query_preserves_the_first_sensitive_paint_and_paint_budget() {
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    let image = Image::from_rgba(crate::Size::new(1.0, 1.0), vec![255, 0, 0, 255])
        .unwrap()
        .rendering(ImageRendering::Pixelated);
    let mut scene = Scene::new();
    scene.layer(Layer::new(), |scene| {
        scene.fill(rect, Color::BLACK);
        scene.layer(Layer::new(), |scene| {
            scene.fill(rect, image);
        });
        scene.fill(rect, Color::BLACK);
    });
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let mut context = RenderAccountingContext::default();
    assert!(run(&mut context, &commands, Query::SurfaceGrid).unwrap());
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::CommandVisits), 4);
    assert_eq!(usage.work(RenderWork::PaintVisits), 2);
    assert_released(&context);
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::PaintVisits, 1)
        .unwrap();
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    let error = run(&mut context, &commands, Query::SurfaceGrid).unwrap_err();
    let report = context.last_attempt().unwrap();
    let attempt = report.attempt().unwrap();
    let refusal = attempt.refusal().unwrap();
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(refusal))
    );
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::PaintVisits)
    );
    assert_eq!(refusal.current(), 1);
    assert_eq!(refusal.amount(), 1);
    assert_eq!(attempt.usage().work(RenderWork::CommandVisits), 4);
    assert_released(&context);
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::PaintVisits, 2)
        .unwrap()
        .with_work_limit(RenderWork::CommandVisits, 4)
        .unwrap();
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    assert!(run(&mut context, &commands, Query::SurfaceGrid).unwrap());
    assert_released(&context);
}

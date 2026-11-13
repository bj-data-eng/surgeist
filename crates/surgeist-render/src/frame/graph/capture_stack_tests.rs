use super::*;
use crate::{
    Capabilities, Color, Layer, RenderAccountingContext, RenderAccountingError,
    RenderAccountingOptions, RenderAttemptKind, RenderLimits, RenderPhase, RenderQuantity,
    RenderResource, Scene,
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

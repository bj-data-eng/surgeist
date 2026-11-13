use super::*;
use crate::{
    Capabilities, Layer, RenderAccountingContext, RenderAttemptKind, RenderResource, Scene,
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

fn check_walk(depth: u64, collect: bool) {
    std::thread::Builder::new()
        .name(
            if collect {
                "collect graph requirements"
            } else {
                "validate graph text bounds"
            }
            .into(),
        )
        .stack_size(512 * 1024)
        .spawn(move || {
            let commands = normalized_scene(depth);
            let mut context = RenderAccountingContext::default();
            context
                .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                    if collect {
                        let requirements = graph_selection_requirements(&commands, account)?;
                        assert_eq!(
                            requirements.as_slice(),
                            &[GraphSelectionRequirement::ExplicitIsolation]
                        );
                    } else {
                        let mut path = account.temporary(0)?;
                        require_graph_text_bounds(&commands, &mut path, account)?;
                        assert!(path.is_empty());
                    }
                    Ok(())
                })
                .unwrap();
            let report = context.last_attempt().unwrap();
            assert_eq!(report.cleanup_fault(), None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::CommandVisits), depth + 2);
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
fn shallow_graph_requirements_preserve_the_isolated_descendant() {
    check_walk(2, true);
}

#[test]
fn deep_graph_requirements_preserve_the_isolated_descendant() {
    check_walk(8_192, true);
}

#[test]
fn shallow_graph_text_validation_restores_the_path() {
    check_walk(2, false);
}

#[test]
fn deep_graph_text_validation_restores_the_path() {
    check_walk(8_192, false);
}

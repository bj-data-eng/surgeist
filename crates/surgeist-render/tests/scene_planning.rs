#![forbid(unsafe_code)]
#![cfg(not(target_family = "wasm"))]

use surgeist_render::{
    Color, Layer, Options, Parameters, RenderAccountingError, RenderAccountingOptions,
    RenderAttemptStatus, RenderCpuScope, RenderLimits, RenderPhase, RenderQuantity, RenderResource,
    RenderWork, Renderer, Scene, Size,
};

fn planning_refusal(depth: u64) {
    std::thread::Builder::new()
        .name("plan nested empty layers".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            // Normalization visits each layer and reads each nonroot child bound.
            // Planning visits the root, then scans its descendants before
            // qualifying its first child: (2*depth-1) + depth visits.
            let normalized_visits = 2 * depth - 1;
            let cap = normalized_visits + depth;
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::CommandVisits, cap)
                .unwrap();
            let options = Options::default()
                .with_accounting(RenderAccountingOptions::new().with_limits(limits));
            let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::try_new(2.0, 2.0).unwrap(), 1.0))
                    .unwrap();
            let published = Scene::new();
            pollster::block_on(renderer.render(
                &mut surface,
                &published,
                Parameters {
                    base_color: Color::BLACK,
                    debug: false,
                },
            ))
            .unwrap();
            let prior_stats = renderer.stats();
            let prior_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let mut scene = Scene::new();
            for _ in 0..depth {
                let child = scene;
                scene = Scene::new();
                scene.layer(Layer::new(), |target| *target = child);
            }
            let error =
                pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                    .unwrap_err();
            let report = renderer.last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(report.cleanup_fault(), None);
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
            assert_eq!(refusal.current(), cap);
            assert_eq!(refusal.amount(), 1);
            assert_eq!(
                attempt
                    .phase_usage(RenderPhase::Normalize)
                    .work(RenderWork::CommandVisits),
                normalized_visits
            );
            assert_eq!(
                attempt
                    .phase_usage(RenderPhase::Plan)
                    .work(RenderWork::CommandVisits),
                depth
            );
            assert_eq!(attempt.usage().work(RenderWork::Submissions), 0);
            let cpu = report
                .renderer_cpu()
                .unwrap()
                .scope_usage(RenderCpuScope::FrameBacking);
            for resource in [
                RenderResource::ScratchCpuBytes,
                RenderResource::RetainedCpuBytes,
            ] {
                assert_eq!(cpu.live(resource), 0);
            }
            assert_eq!(renderer.stats(), prior_stats);
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                prior_pixels
            );
            scene.clear();
            pollster::block_on(renderer.render(
                &mut surface,
                &scene,
                Parameters {
                    base_color: Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
                    debug: false,
                },
            ))
            .unwrap();
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface))
                    .unwrap()
                    .rgba(),
                [255, 0, 0, 255].repeat(4)
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn shallow_planning_refusal_preserves_publication_and_retry() {
    planning_refusal(2);
}

#[test]
fn deep_planning_refusal_preserves_publication_and_retry() {
    planning_refusal(8_192);
}

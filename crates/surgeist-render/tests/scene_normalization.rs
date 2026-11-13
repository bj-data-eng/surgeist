#![forbid(unsafe_code)]
#![cfg(not(target_family = "wasm"))]

use surgeist_render::{
    Color, ErrorCode, Image, Layer, Options, Parameters, Rect, RenderAttemptStatus, RenderPhase,
    RenderResource, RenderWork, Renderer, Scene, Size,
};

#[test]
fn deep_invalid_image_source_preserves_publication_and_can_retry() {
    std::thread::Builder::new()
        .name("normalize deep authored scene".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            const DEPTH: u64 = 8_192;
            let size = Size::try_new(2.0, 2.0).unwrap();
            let target = Rect::try_new(0.0, 0.0, 2.0, 2.0).unwrap();
            let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
            let mut surface = pollster::block_on(renderer.create_headless(size, 1.0)).unwrap();
            let mut published = Scene::new();
            published.fill(target, Color::BLACK);
            pollster::block_on(renderer.render(&mut surface, &published, Parameters::default()))
                .unwrap();
            let prior_stats = renderer.stats();
            let prior_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();

            let image =
                Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 0, 0, 255]).unwrap();
            let mut scene = Scene::new();
            scene.image_region(image, Rect::try_new(2.0, 0.0, 1.0, 1.0).unwrap(), target);
            for _ in 0..DEPTH {
                let child = scene;
                scene = Scene::new();
                scene.layer(Layer::new(), |target| *target = child);
            }
            let error =
                pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                    .unwrap_err();
            assert_eq!(error.code(), ErrorCode::InvalidInput);
            assert_eq!(
                error.invalid_value_diagnostic().map(|value| value.field()),
                Some("image source region x")
            );
            let report = renderer.last_attempt().unwrap();
            assert_eq!(
                report.status(),
                RenderAttemptStatus::Failed(ErrorCode::InvalidInput)
            );
            assert_eq!(report.cleanup_fault(), None);
            let attempt = report.attempt().unwrap();
            let normalized = attempt.phase_usage(RenderPhase::Normalize);
            assert_eq!(normalized.work(RenderWork::CommandVisits), DEPTH + 1);
            assert_eq!(normalized.work(RenderWork::CommandsEmitted), 0);
            assert_eq!(attempt.usage().work(RenderWork::Submissions), 0);
            let cpu = report.renderer_cpu().unwrap().usage();
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
            scene.fill(target, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                .unwrap();
            let retried = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            assert_eq!(retried.rgba(), [255, 0, 0, 255].repeat(4));
        })
        .unwrap()
        .join()
        .unwrap();
}

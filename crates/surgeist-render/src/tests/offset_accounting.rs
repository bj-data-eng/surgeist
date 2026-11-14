use super::convolution::{assert_pixel, bounds, image};
use super::offset::{image_scene, operation, render};
use crate::{
    ImageRendering, OffsetInput, Options, Parameters, RenderAccountingOptions, RenderAttemptStatus,
    RenderLimits, RenderPhaseFilter, RenderProfile, RenderQuantity, RenderWork, Renderer,
    ResourceCacheBudget, Scene, Size,
};

fn fixture(dx: f64, hint: ImageRendering) -> Scene {
    let domain = bounds(4, 1);
    image_scene(
        image(4, 1, [64, 128, 192, 255].repeat(4)),
        domain,
        operation(
            OffsetInput::new().with_dx(dx).with_rendering(hint),
            domain,
            domain,
            domain,
        ),
    )
}

#[test]
fn public_offset_profiles_preserve_pixels_and_exact_sample_footprints() {
    for (dx, hint, samples) in [
        (0.0, ImageRendering::HighQuality, 4),
        (0.5, ImageRendering::CrispEdges, 4),
        (0.5, ImageRendering::Auto, 16),
        (0.5, ImageRendering::HighQuality, 64),
    ] {
        let mut observations = Vec::new();
        for profile in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 128,
                filter: RenderPhaseFilter::All,
            },
        ] {
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
            ))
            .unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
            let output = render(&mut renderer, &mut surface, &fixture(dx, hint));
            let report = renderer.last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            let usage = report.attempt().unwrap().usage();
            let counts = [
                RenderWork::ScheduledOffsetSamples,
                RenderWork::ScheduledFilterTexels,
                RenderWork::KernelSampleVisits,
                RenderWork::ScheduledConvolutionTaps,
                RenderWork::SerializedBytes,
                RenderWork::UploadBytes,
            ]
            .map(|work| usage.work(work));
            // One actual output pass covers4 cells; its selected fixed footprint
            // is1/1/4/16. Transparent exterior taps remain scheduled opportunities.
            assert_eq!(counts[..4], [samples, 4, 0, 0]);
            observations.push((output, counts));
        }
        assert_eq!(observations[0], observations[1]);
        assert_eq!(observations[1], observations[2]);
    }
}

#[test]
fn public_offset_exact_sample_budget_and_one_under_preserve_previous_publication() {
    let scene = fixture(0.5, ImageRendering::Auto);
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 128,
            filter: RenderPhaseFilter::All,
        },
    ] {
        for cache in [ResourceCacheBudget::DEFAULT, ResourceCacheBudget::DISABLED] {
            for cap in [15, 16] {
                let limits = RenderLimits::new()
                    .with_work_limit(RenderWork::ScheduledOffsetSamples, cap)
                    .unwrap();
                let mut renderer = pollster::block_on(Renderer::new(
                    Options::default()
                        .with_resource_cache_budget(cache)
                        .with_accounting(
                            RenderAccountingOptions::new()
                                .with_profile(profile)
                                .with_limits(limits),
                        ),
                ))
                .unwrap();
                let mut surface =
                    pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
                let baseline = pollster::block_on(renderer.render(
                    &mut surface,
                    &Scene::new(),
                    Parameters::default(),
                ))
                .unwrap();
                let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                let result = pollster::block_on(renderer.render(
                    &mut surface,
                    &scene,
                    Parameters::default(),
                ));
                if cap == 15 {
                    let error = result.unwrap_err();
                    let report = renderer.last_attempt().unwrap();
                    assert_eq!(report.status(), RenderAttemptStatus::Refused);
                    assert_eq!(report.cleanup_fault(), None);
                    let refusal = report.attempt().unwrap().refusal().unwrap();
                    assert_eq!(
                        refusal.quantity(),
                        RenderQuantity::Work(RenderWork::ScheduledOffsetSamples)
                    );
                    assert_eq!((refusal.current(), refusal.amount()), (0, 16));
                    assert!(error.accounting_diagnostic().is_some());
                    assert_eq!(renderer.stats(), baseline);
                    assert_eq!(
                        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                        before
                    );
                } else {
                    result.unwrap();
                    let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                    assert_pixel(&output, 2, 0, [64, 128, 192, 255], 1);
                }
                let resources = renderer
                    .default_ready_device_state_borrow_for_test()
                    .unwrap()
                    .internal_resource_manager_observation_for_test();
                assert_eq!(resources.active_frame_count, 0);
                assert_eq!(resources.accounting_fault_for_test(), None);
            }
        }
    }
}

#[test]
fn public_offset_actual_uniform_upload_allocation_and_serialization_caps_refuse_cleanly() {
    let scene = fixture(0.5, ImageRendering::HighQuality);
    for work in [
        RenderWork::UploadBytes,
        RenderWork::SerializedBytes,
        RenderWork::ResourceAllocationRequests,
    ] {
        let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
        pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
        render(&mut renderer, &mut surface, &scene);
        let measured = renderer
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(work);
        assert!(measured > 0);
        drop((renderer, surface));
        for cap in [measured - 1, measured] {
            let limits = RenderLimits::new().with_work_limit(work, cap).unwrap();
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_accounting(RenderAccountingOptions::new().with_limits(limits)),
            ))
            .unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
            let baseline = pollster::block_on(renderer.render(
                &mut surface,
                &Scene::new(),
                Parameters::default(),
            ))
            .unwrap();
            let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let result =
                pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()));
            if cap < measured {
                let error = result.unwrap_err();
                assert!(error.accounting_diagnostic().is_some());
                let report = renderer.last_attempt().unwrap();
                assert_eq!(
                    report.attempt().unwrap().refusal().unwrap().quantity(),
                    RenderQuantity::Work(work)
                );
                assert_eq!(report.cleanup_fault(), None);
                assert_eq!(renderer.stats(), baseline);
                assert_eq!(
                    pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                    before
                );
            } else {
                result.unwrap();
            }
            assert_eq!(
                renderer
                    .default_ready_device_state_borrow_for_test()
                    .unwrap()
                    .internal_resource_manager_observation_for_test()
                    .active_frame_count,
                0
            );
        }
    }
}

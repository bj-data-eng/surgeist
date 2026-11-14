//! Actual planned Gaussian cell/sample footprints, with atomic budget refusal.
use super::convolution::{assert_pixel, bounds, image};
use super::gaussian::{finite_scene, operation, render, sigma, source_scene};
use crate::{
    FilterColorSpace, FilterDomains, FilterEdgeMode, Layer, Options, Parameters,
    RenderAccountingOptions, RenderAttemptStatus, RenderLimits, RenderPhaseFilter, RenderProfile,
    RenderQuantity, RenderWork, Renderer, ResourceCacheBudget, Scene, Size,
};

fn fixture(x: f64, y: f64) -> Scene {
    let domain = bounds(3, 1);
    finite_scene(
        source_scene(image(3, 1, [64, 128, 192, 255].repeat(3)), domain),
        FilterDomains::try_new(domain, domain, domain).unwrap(),
        FilterEdgeMode::Duplicate,
        operation(x, y, FilterColorSpace::Srgb),
        Layer::new(),
    )
}
fn profiles() -> [RenderProfile; 3] {
    [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 256,
            filter: RenderPhaseFilter::All,
        },
    ]
}

#[test]
fn public_gaussian_profiles_match_real_halo_batch_and_sample_footprints() {
    for (x, y, texels, samples) in [
        (sigma(), 0.0, 9, 33),
        (sigma(), sigma(), 36, 276),
        (4.0, 0.0, 12, 108),
    ] {
        let mut observations = Vec::new();
        for profile in profiles() {
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
            ))
            .unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::new(3.0, 1.0), 1.0)).unwrap();
            let output = render(&mut renderer, &mut surface, &fixture(x, y));
            assert_pixel(&output, 1, 0, [64, 128, 192, 255], 2);
            let report = renderer.last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            let usage = report.attempt().unwrap().usage();
            let counts = [
                RenderWork::ScheduledFilterTexels,
                RenderWork::ScheduledGaussianSamples,
                RenderWork::KernelSampleVisits,
                RenderWork::SerializedBytes,
                RenderWork::UploadBytes,
            ]
            .map(|w| usage.work(w));
            assert_eq!(counts[..2], [texels, samples]);
            observations.push((output, counts));
        }
        assert_eq!(observations[0], observations[1]);
        assert_eq!(observations[1], observations[2]);
    }
}

#[test]
fn public_gaussian_exact_sample_and_halo_budgets_preserve_publication_on_one_under() {
    let scene = fixture(sigma(), sigma());
    for profile in profiles() {
        for cache in [ResourceCacheBudget::DEFAULT, ResourceCacheBudget::DISABLED] {
            for (work, exact) in [
                (RenderWork::ScheduledGaussianSamples, 276),
                (RenderWork::ScheduledFilterTexels, 36),
            ] {
                for cap in [exact - 1, exact] {
                    let options = RenderAccountingOptions::new()
                        .with_profile(profile)
                        .with_limits(RenderLimits::new().with_work_limit(work, cap).unwrap());
                    let mut renderer = pollster::block_on(Renderer::new(
                        Options::default()
                            .with_resource_cache_budget(cache)
                            .with_accounting(options),
                    ))
                    .unwrap();
                    let mut surface =
                        pollster::block_on(renderer.create_headless(Size::new(3.0, 1.0), 1.0))
                            .unwrap();
                    let baseline = pollster::block_on(renderer.render(
                        &mut surface,
                        &Scene::new(),
                        Parameters::default(),
                    ))
                    .unwrap();
                    let old = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                    let outcome = pollster::block_on(renderer.render(
                        &mut surface,
                        &scene,
                        Parameters::default(),
                    ));
                    if cap < exact {
                        let error = outcome.unwrap_err();
                        let report = renderer.last_attempt().unwrap();
                        assert_eq!(
                            report.attempt().unwrap().refusal().unwrap().quantity(),
                            RenderQuantity::Work(work)
                        );
                        assert!(error.accounting_diagnostic().is_some());
                        assert_eq!(report.status(), RenderAttemptStatus::Refused);
                        assert_eq!(report.cleanup_fault(), None);
                        assert_eq!(renderer.stats(), baseline);
                        assert_eq!(
                            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                            old
                        );
                    } else {
                        outcome.unwrap();
                        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                        assert_pixel(&pixels, 1, 0, [64, 128, 192, 255], 2);
                        assert_eq!(
                            renderer
                                .last_attempt()
                                .unwrap()
                                .attempt()
                                .unwrap()
                                .usage()
                                .work(work),
                            exact
                        );
                    }
                    let ready = renderer
                        .default_ready_device_state_borrow_for_test()
                        .unwrap();
                    let resources = ready.internal_resource_manager_observation_for_test();
                    assert_eq!(resources.leased_count, 0);
                    assert_eq!(resources.active_frame_count, 0);
                    assert_eq!(resources.accounting_fault_for_test(), None);
                }
            }
        }
    }
}

#[test]
fn public_gaussian_actual_uniform_upload_allocation_and_serialization_caps_refuse_cleanly() {
    let scene = fixture(4.0, 1.0);
    for work in [
        RenderWork::UploadBytes,
        RenderWork::SerializedBytes,
        RenderWork::ResourceAllocationRequests,
    ] {
        let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(3.0, 1.0), 1.0)).unwrap();
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
                pollster::block_on(renderer.create_headless(Size::new(3.0, 1.0), 1.0)).unwrap();
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

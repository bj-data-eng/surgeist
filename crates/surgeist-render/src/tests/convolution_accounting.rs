use super::convolution::*;
use crate::{
    ConvolutionInput, FilterColorSpace, Options, Parameters, RenderAccountingOptions,
    RenderAttemptStatus, RenderLimits, RenderPhaseFilter, RenderProfile, RenderQuantity,
    RenderWork, Renderer, ResourceCacheBudget, Scene, Size,
};

fn fixture() -> Scene {
    let domain = bounds(2, 1);
    image_scene(
        image(2, 1, vec![64, 128, 192, 255, 192, 128, 64, 255]),
        domain,
        operation(
            ConvolutionInput::new(&[1.0]).with_order(1.0, None),
            domain,
            FilterColorSpace::Srgb,
        ),
    )
}

#[test]
fn accounting_profiles_preserve_gpu_pixels_and_actual_halo_work() {
    let scene = fixture();
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
        let options = Options::default()
            .with_accounting(RenderAccountingOptions::new().with_profile(profile));
        let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(2.0, 1.0), 1.0)).unwrap();
        let output = render(&mut renderer, &mut surface, &scene);
        let report = renderer.last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(report.cleanup_fault(), None);
        let usage = report.attempt().unwrap().usage();
        let counts = [
            RenderWork::ScheduledConvolutionTaps,
            RenderWork::ScheduledFilterTexels,
            RenderWork::KernelSampleVisits,
            RenderWork::SerializedBytes,
            RenderWork::UploadBytes,
        ]
        .map(|work| usage.work(work));
        // P is2x1, I is2x1. Bilinear result reconstruction requires J[-1..3)x[-1..2),
        // twelve cells. One tap executes at all12, not only two visible pixels.
        // Prepare2+accumulate12+finalize12+resolve2 =28 actual scheduled texels.
        assert_eq!(counts[0], 12);
        assert_eq!(counts[1], 28);
        observations.push((output, counts));
    }
    assert_eq!(observations[0], observations[1]);
    assert_eq!(observations[1], observations[2]);
}

#[test]
fn exact_tap_cap_one_under_preserves_publication_and_higher_budget_retries() {
    let scene = fixture();
    for cache in [ResourceCacheBudget::DEFAULT, ResourceCacheBudget::DISABLED] {
        for cap in [11, 12] {
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::ScheduledConvolutionTaps, cap)
                .unwrap();
            let options = Options::default()
                .with_resource_cache_budget(cache)
                .with_accounting(RenderAccountingOptions::new().with_limits(limits));
            let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::new(2.0, 1.0), 1.0)).unwrap();
            let baseline = pollster::block_on(renderer.render(
                &mut surface,
                &Scene::new(),
                Parameters::default(),
            ))
            .unwrap();
            let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let result =
                pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()));
            if cap == 11 {
                let error = result.unwrap_err();
                let report = renderer.last_attempt().unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(report.cleanup_fault(), None);
                let refusal = report.attempt().unwrap().refusal().unwrap();
                assert_eq!(
                    refusal.quantity(),
                    RenderQuantity::Work(RenderWork::ScheduledConvolutionTaps)
                );
                assert_eq!((refusal.current(), refusal.amount()), (0, 12));
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(crate::RenderAccountingError::Refused(refusal))
                );
                assert_eq!(renderer.stats(), baseline);
                assert_eq!(
                    pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                    before
                );
                assert!(renderer.default_device_has_no_terminal_signal_for_test());
            } else {
                result.unwrap();
                let after = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                assert_pixel(&after, 0, 0, [64, 128, 192, 255], 1);
                assert_pixel(&after, 1, 0, [192, 128, 64, 255], 1);
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

#[test]
fn upload_serialization_and_allocation_caps_refuse_at_real_gpu_boundaries() {
    let scene = fixture();
    let mut measured = [0; 3];
    let dimensions = [
        RenderWork::UploadBytes,
        RenderWork::SerializedBytes,
        RenderWork::ResourceAllocationRequests,
    ];
    for (position, work) in dimensions.into_iter().enumerate() {
        // Each calibration and limited attempt starts from the same successful
        // empty-frame state. The report measures real platform allocation work;
        // it is not used as a numerical pixel oracle.
        let mut measure = pollster::block_on(Renderer::new(Options::default())).unwrap();
        let mut surface =
            pollster::block_on(measure.create_headless(Size::new(2.0, 1.0), 1.0)).unwrap();
        pollster::block_on(measure.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
        render(&mut measure, &mut surface, &scene);
        measured[position] = measure
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(work);
        assert!(measured[position] > 0);
        drop((measure, surface));
        for cap in [measured[position] - 1, measured[position]] {
            let limits = RenderLimits::new().with_work_limit(work, cap).unwrap();
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_accounting(RenderAccountingOptions::new().with_limits(limits)),
            ))
            .unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::new(2.0, 1.0), 1.0)).unwrap();
            let baseline = pollster::block_on(renderer.render(
                &mut surface,
                &Scene::new(),
                Parameters::default(),
            ))
            .unwrap();
            let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let result =
                pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()));
            if cap < measured[position] {
                let error = result.unwrap_err();
                let report = renderer.last_attempt().unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(report.cleanup_fault(), None);
                assert_eq!(
                    report.attempt().unwrap().refusal().unwrap().quantity(),
                    RenderQuantity::Work(work)
                );
                assert!(error.accounting_diagnostic().is_some());
                assert_eq!(renderer.stats(), baseline);
                assert_eq!(
                    pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                    before
                );
            } else {
                result.unwrap();
                let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                assert_pixel(&output, 0, 0, [64, 128, 192, 255], 1);
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

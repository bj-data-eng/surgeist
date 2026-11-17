//! Independent scheduled opportunities and omission rules for native displacement.
use super::convolution::{assert_pixel, bounds, image};
use super::displacement::{active, finite_scene, native_map, source_scene};
use crate::{
    accounting::RenderAccountingContext, accounting::RenderAccountingOptions,
    accounting::RenderAttemptStatus, accounting::RenderLimits, accounting::RenderPhaseFilter,
    accounting::RenderProfile, accounting::RenderQuantity, accounting::RenderWork,
    displacement::DisplacementChannel as Channel, displacement::DisplacementInput,
    displacement::PreparedDisplacementMap, displacement::prepare_displacement,
    displacement::prepare_native_displacement_map, filter_domains::FilterDomains,
    filter_model::FilterColorSpace, geometry::Rect, geometry::Size, layer::Layer, paint::Color,
    renderer::Options, renderer::Renderer, renderer::ResourceCacheBudget, scene::Scene,
    surface::Parameters,
};
use std::sync::Arc;

const GPU: [RenderWork; 4] = [
    RenderWork::ScheduledDisplacementMapRecordReads,
    RenderWork::ScheduledDisplacementAddressReads,
    RenderWork::ScheduledDisplacementMapSamples,
    RenderWork::ScheduledDisplacementSourceSamples,
];
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
fn fixture(scale: f64, map: PreparedDisplacementMap) -> Scene {
    let full = bounds(4, 1);
    finite_scene(
        source_scene(
            image(
                4,
                1,
                vec![
                    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
                ],
            ),
            full,
        ),
        FilterDomains::try_new(full, full, Rect::new(1.0, 0.0, 2.0, 1.0)).unwrap(),
        active(scale, Channel::A, Channel::R),
        map,
        Layer::new(),
    )
}
fn map() -> PreparedDisplacementMap {
    // A255 selects +1, R128 selects0 for scale2. O is2x1, independent of W.
    native_map(1, 1, vec![128, 0, 0, 255], bounds(4, 1))
}
fn zero_map() -> PreparedDisplacementMap {
    native_map(0, 1, vec![], bounds(4, 1))
}
fn new_renderer(options: RenderAccountingOptions) -> (Renderer, crate::surface::Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
            .with_accounting(options),
    ))
    .unwrap();
    let surface = pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
    (renderer, surface)
}
fn counts(renderer: &Renderer) -> [u64; 7] {
    let usage = renderer.last_attempt().unwrap().attempt().unwrap().usage();
    [
        GPU[0],
        GPU[1],
        GPU[2],
        GPU[3],
        RenderWork::DisplacementTableVisits,
        RenderWork::DisplacementMapRecordVisits,
        RenderWork::DisplacementMapByteVisits,
    ]
    .map(|work| usage.work(work))
}

#[test]
fn public_displacement_profiles_observe_raw_zero_and_bypass_opportunities_independently() {
    for (scale, map, expected, expected_pixel) in [
        (2.0, map(), [4, 4, 2, 2, 512, 3, 0], [0, 0, 255, 255]),
        (2.0, zero_map(), [0, 4, 0, 2, 512, 0, 0], [0, 255, 0, 255]),
        (0.0, map(), [0; 7], [0, 255, 0, 255]),
        (
            2.0,
            PreparedDisplacementMap::source_graphic(),
            [0; 7],
            [0, 255, 0, 255],
        ),
        (
            2.0,
            PreparedDisplacementMap::source_alpha(),
            [0; 7],
            [0, 255, 0, 255],
        ),
        (
            2.0,
            PreparedDisplacementMap::suppressed(),
            [0; 7],
            [0, 255, 0, 255],
        ),
    ] {
        let scene = fixture(scale, map);
        let mut observations = Vec::new();
        for profile in profiles() {
            let (mut renderer, mut surface) =
                new_renderer(RenderAccountingOptions::new().with_profile(profile));
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                .unwrap();
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            assert_pixel(&pixels, 1, 0, expected_pixel, 1);
            assert_pixel(&pixels, 0, 0, [0; 4], 0);
            assert_eq!(counts(&renderer), expected);
            let report = renderer.last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            observations.push((pixels, counts(&renderer)));
        }
        assert_eq!(observations[0], observations[1]);
        assert_eq!(observations[1], observations[2]);
    }
}

#[test]
fn public_displacement_each_scheduled_counter_and_metadata_has_exact_cap_and_one_under() {
    // N=2 actual O cells: two axis records and addresses per cell, one map and
    // one source opportunity per cell. Records are Ow+Oh=3, tables2*256=512.
    for (work, exact) in [
        (GPU[0], 4),
        (GPU[1], 4),
        (GPU[2], 2),
        (GPU[3], 2),
        (RenderWork::DisplacementTableVisits, 512),
        (RenderWork::DisplacementMapRecordVisits, 3),
    ] {
        let scene = fixture(2.0, map());
        for cap in [exact - 1, exact] {
            let options = RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_work_limit(work, cap).unwrap());
            let (mut renderer, mut surface) = new_renderer(options);
            let old_stats = pollster::block_on(renderer.render(
                &mut surface,
                &Scene::new(),
                Parameters::default(),
            ))
            .unwrap();
            let old_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let outcome =
                pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()));
            if cap < exact {
                let error = outcome.unwrap_err();
                let report = renderer.last_attempt().unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(report.cleanup_fault(), None);
                let refusal = report.attempt().unwrap().refusal().unwrap();
                assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(crate::accounting::RenderAccountingError::Refused(refusal))
                );
                assert_eq!(renderer.stats(), old_stats);
                assert_eq!(
                    pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                    old_pixels
                );
                // Same input succeeds with precisely the complete allowance.
                let (mut retry_renderer, mut retry_surface) = new_renderer(
                    options.with_limits(RenderLimits::new().with_work_limit(work, exact).unwrap()),
                );
                pollster::block_on(retry_renderer.render(
                    &mut retry_surface,
                    &scene,
                    Parameters::default(),
                ))
                .unwrap();
                assert_eq!(
                    retry_renderer
                        .last_attempt()
                        .unwrap()
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(work),
                    exact
                );
            } else {
                outcome.unwrap();
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
            let resources = renderer
                .default_ready_device_state_borrow_for_test()
                .unwrap()
                .internal_resource_manager_observation_for_test();
            assert_eq!(resources.leased_count, 0);
            assert_eq!(resources.active_frame_count, 0);
            assert_eq!(resources.accounting_fault_for_test(), None);
        }
    }
}

#[test]
fn public_displacement_constructor_byte_work_is_separate_from_later_unused_map_frame() {
    let bytes: Arc<[u8]> = Arc::from([255, 128, 64, 1]);
    let mut context = RenderAccountingContext::default();
    let map =
        prepare_native_displacement_map(Size::new(1.0, 1.0), bytes, bounds(4, 1), &mut context)
            .unwrap();
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::DisplacementMapByteVisits),
        4
    );
    // The supplied map remains admitted; only this frame's scale-zero route
    // omits import/caches/tables/records and map-color work.
    let effect = prepare_displacement(
        DisplacementInput::new(),
        FilterColorSpace::LinearSrgb,
        &mut context,
    )
    .unwrap();
    let mut scene = Scene::new();
    scene
        .displacement_layer(
            Layer::new(),
            effect,
            crate::displacement::DisplacementSource::constant(
                Color::BLACK,
                bounds(4, 1),
                bounds(4, 1),
            )
            .unwrap(),
            map,
        )
        .unwrap();
    let omitted = [
        RenderWork::DisplacementMapByteVisits,
        RenderWork::DisplacementMapRecordVisits,
        RenderWork::DisplacementTableVisits,
        RenderWork::ImageIdentityCopies,
        GPU[0],
        GPU[1],
        GPU[2],
        GPU[3],
    ];
    let mut limits = RenderLimits::new();
    for work in omitted {
        limits = limits.with_work_limit(work, 0).unwrap();
    }
    let (mut renderer, mut surface) =
        new_renderer(RenderAccountingOptions::new().with_limits(limits));
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    for work in omitted {
        assert_eq!(
            renderer
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(work),
            0
        );
    }
}

#[test]
fn public_displacement_raw_map_capability_refusal_preserves_zero_and_bypass_routes_then_retries() {
    use crate::{error::RuntimeCapabilityUnavailableReason, error::RuntimeOperation};
    let (mut renderer, mut surface) = new_renderer(RenderAccountingOptions::new());
    let zero = fixture(2.0, zero_map());
    let raw = fixture(2.0, map());
    // Withhold only raw-map upload eligibility. The actual source raster and
    // working-format facts remain available to zero-map and bypass execution.
    assert!(renderer.override_default_device_displacement_map_upload_for_test(false));
    pollster::block_on(renderer.render(&mut surface, &zero, Parameters::default())).unwrap();
    pollster::block_on(renderer.render(&mut surface, &fixture(0.0, map()), Parameters::default()))
        .unwrap();
    let old_stats = renderer.stats();
    let old_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let error =
        pollster::block_on(renderer.render(&mut surface, &raw, Parameters::default())).unwrap_err();
    let diagnostic = error.runtime_capability_unavailable_diagnostic().unwrap();
    assert_eq!(
        diagnostic.operation(),
        RuntimeOperation::DisplacementRendering
    );
    assert_eq!(
        diagnostic.reason(),
        RuntimeCapabilityUnavailableReason::DisplacementMapFormatUnavailable
    );
    assert_eq!(renderer.stats(), old_stats);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        old_pixels
    );
    assert!(renderer.override_default_device_displacement_map_upload_for_test(true));
    pollster::block_on(renderer.render(&mut surface, &raw, Parameters::default())).unwrap();
    assert_pixel(
        &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        1,
        0,
        [0, 0, 255, 255],
        1,
    );
}

#[test]
fn public_displacement_shared_native_bytes_have_one_renderer_import_and_actual_last_owner() {
    use crate::{
        accounting::RenderCpuScope, accounting::RenderResource, displacement::DisplacementSource,
    };
    let bytes: Arc<[u8]> = Arc::from([128, 0, 0, 255]);
    let weak = Arc::downgrade(&bytes);
    let map = prepare_native_displacement_map(
        Size::new(1.0, 1.0),
        bytes,
        bounds(4, 1),
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    let build = |layers| {
        let mut scene = Scene::new();
        for _ in 0..layers {
            scene
                .displacement_layer(
                    Layer::new(),
                    active(2.0, Channel::A, Channel::R),
                    DisplacementSource::constant(Color::BLACK, bounds(4, 1), bounds(4, 1)).unwrap(),
                    map.clone(),
                )
                .unwrap();
        }
        scene
    };
    let single = build(1);
    let scene = build(2);
    drop(map);
    let mut renderer = pollster::block_on(Renderer::new(Options::default().with_accounting(
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
    )))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
    pollster::block_on(renderer.render(&mut surface, &single, Parameters::default())).unwrap();
    let one = renderer
        .cpu_accounting_snapshot()
        .unwrap()
        .scope_usage(RenderCpuScope::ImagePayloads)
        .unwrap()
        .live(RenderResource::RetainedCpuBytes);
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    // The existing registry capacity and cached raw descriptor are unchanged.
    // A second actual layer sharing that Arc adds zero pixel/registry backing.
    assert_eq!(
        renderer
            .cpu_accounting_snapshot()
            .unwrap()
            .scope_usage(RenderCpuScope::ImagePayloads)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes),
        one
    );
    drop(single);
    drop(scene);
    assert!(
        weak.upgrade().is_some(),
        "the actual retained raw-map cache owns its immutable bytes"
    );
    drop((surface, renderer));
    assert!(weak.upgrade().is_none());
}

#[test]
fn public_displacement_linear_rgb_builds_one_256_entry_color_table_before_two_address_tables() {
    let full = bounds(9, 9);
    let mut source = Scene::new();
    source.fill(full, Color::BLACK);
    let effect = prepare_displacement(
        DisplacementInput::new()
            .with_scale(8.0)
            .with_channels(Channel::R, Channel::R),
        FilterColorSpace::LinearSrgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    let scene = finite_scene(
        source,
        FilterDomains::try_new(full, full, Rect::new(4.0, 4.0, 1.0, 1.0)).unwrap(),
        effect,
        native_map(1, 1, vec![128, 0, 0, 255], full),
        Layer::new(),
    );
    // Exactly one map-only color LUT256 plus two256-entry final address tables.
    // There is no per-axis duplicate color LUT and alpha/source color do not transfer.
    for cap in [767, 768] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default().with_accounting(
                RenderAccountingOptions::new().with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::DisplacementTableVisits, cap)
                        .unwrap(),
                ),
            ),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(9.0, 9.0), 1.0)).unwrap();
        let old =
            pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
                .unwrap();
        let result =
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()));
        if cap == 767 {
            let error = result.unwrap_err();
            let refusal = renderer
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .refusal()
                .unwrap();
            assert_eq!(
                refusal.quantity(),
                RenderQuantity::Work(RenderWork::DisplacementTableVisits)
            );
            assert_eq!(
                error.accounting_diagnostic(),
                Some(crate::accounting::RenderAccountingError::Refused(refusal))
            );
            assert_eq!(renderer.stats(), old);
        } else {
            result.unwrap();
            assert_eq!(
                renderer
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::DisplacementTableVisits),
                768
            );
            assert_pixel(
                &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                4,
                4,
                [0, 0, 0, 255],
                1,
            );
        }
    }
}

#[test]
fn public_displacement_impossible_storage_limits_refuse_before_any_native_backing_and_preserve_route_omissions()
 {
    use crate::{
        displacement::DisplacementNumericPhase, displacement::DisplacementPreparationError,
    };
    let (mut renderer, mut surface) = new_renderer(RenderAccountingOptions::new());
    pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
        .unwrap();
    let actual = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap()
        .device_for_test()
        .limits();
    let raw = fixture(2.0, map());
    for boundary in 0..4 {
        let mut limits = actual.clone();
        match boundary {
            0 => limits.max_buffer_size = 4095,
            1 => limits.max_storage_buffer_binding_size = 4095,
            2 => limits.max_storage_buffers_per_shader_stage = 0,
            _ => limits.max_storage_buffers_per_shader_stage = 1,
        }
        assert!(renderer.override_default_device_displacement_buffer_limits_for_test(&limits));
        let opening = renderer
            .device_accounting_snapshot(Some(&surface))
            .unwrap()
            .usage()
            .work(RenderWork::ResourceAllocations);
        let error = pollster::block_on(renderer.render(&mut surface, &raw, Parameters::default()))
            .unwrap_err();
        assert!(
            matches!(
                error.displacement_diagnostic(),
                Some(DisplacementPreparationError::Numeric {
                    phase: DisplacementNumericPhase::TableLayout,
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(
            renderer
                .device_accounting_snapshot(Some(&surface))
                .unwrap()
                .usage()
                .work(RenderWork::ResourceAllocations),
            opening
        );
        // Bypass does not require a storage buffer. A zero native map requires
        // only one, so withholding the second binding must still allow it.
        pollster::block_on(renderer.render(
            &mut surface,
            &fixture(0.0, map()),
            Parameters::default(),
        ))
        .unwrap();
        if boundary == 3 {
            pollster::block_on(renderer.render(
                &mut surface,
                &fixture(2.0, zero_map()),
                Parameters::default(),
            ))
            .unwrap();
        }
    }
    assert!(renderer.override_default_device_displacement_buffer_limits_for_test(&actual));
    pollster::block_on(renderer.render(&mut surface, &raw, Parameters::default())).unwrap();
    assert_pixel(
        &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        1,
        0,
        [0, 0, 255, 255],
        1,
    );
}

#[test]
fn public_displacement_actual_upload_serialization_and_native_creation_caps_are_atomic() {
    let mut scene = Scene::new();
    scene
        .displacement_layer(
            Layer::new(),
            active(2.0, Channel::A, Channel::R),
            crate::displacement::DisplacementSource::constant(
                Color::BLACK,
                bounds(4, 1),
                bounds(4, 1),
            )
            .unwrap(),
            map(),
        )
        .unwrap();
    for work in [
        RenderWork::UploadBytes,
        RenderWork::SerializedBytes,
        RenderWork::ResourceAllocationRequests,
    ] {
        let (mut renderer, mut surface) = new_renderer(RenderAccountingOptions::new());
        pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
        pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
        // This is a measured cold-descriptor admission boundary, distinct from
        // the independently derived scheduled opportunity and physical-byte oracles.
        let measured = renderer
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(work);
        assert!(measured > 0);
        drop((surface, renderer));
        for cap in [measured - 1, measured] {
            let (mut renderer, mut surface) = new_renderer(
                RenderAccountingOptions::new()
                    .with_limits(RenderLimits::new().with_work_limit(work, cap).unwrap()),
            );
            let stats = pollster::block_on(renderer.render(
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
                let report = renderer.last_attempt().unwrap();
                let refusal = report.attempt().unwrap().refusal().unwrap();
                assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(crate::accounting::RenderAccountingError::Refused(refusal))
                );
                assert_eq!(report.cleanup_fault(), None);
                assert_eq!(renderer.stats(), stats);
                assert_eq!(
                    pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                    before
                );
            } else {
                result.unwrap();
                assert_eq!(
                    renderer
                        .last_attempt()
                        .unwrap()
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(work),
                    measured
                );
                assert_pixel(
                    &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                    1,
                    0,
                    [0, 0, 0, 255],
                    1,
                );
            }
            let resources = renderer
                .default_ready_device_state_borrow_for_test()
                .unwrap()
                .internal_resource_manager_observation_for_test();
            assert_eq!(
                (resources.active_frame_count, resources.leased_count),
                (0, 0)
            );
            assert_eq!(resources.accounting_fault_for_test(), None);
        }
    }
}

use super::*;
use crate::{
    ErrorCode,
    accounting::{
        RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus, RenderLimits,
        RenderQuantity, RenderResource,
    },
    capability::Capabilities,
    command::{NormalizedForegroundEffect, RenderCommand},
    error::{PrimitiveFamily, PrimitiveOperation, UnsupportedPrimitive},
    geometry::{Rect, Transform},
    scene::{Filter, Layer, LayerComposition, PorterDuffOperator},
};

fn admit(input: DiffuseLightingInput, light: LightingLightInput) -> PreparedLighting {
    prepare_lighting(
        input,
        light,
        FilterColorSpace::Srgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap()
}

#[test]
fn diffuse_defaults_and_explicit_pairs_admit_the_selected_grid() {
    let default = admit(DiffuseLightingInput::default(), LightingLightInput::point());
    assert_eq!(default.surface_scale(), 1.0);
    assert_eq!(default.constant(), 1.0);
    assert_eq!(default.material_exponent(), None);
    assert_eq!(default.kernel_unit_lengths(), None);
    assert_eq!(
        default.lighting_color(),
        Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap()
    );
    assert_eq!(default.light(), PreparedLight::Point { position: [0.0; 3] });

    let input = DiffuseLightingInput::new()
        .with_surface_scale(-3.0)
        .with_diffuse_constant(f64::MAX)
        .with_kernel_unit_lengths(1.5, 2.25);
    let prepared = admit(input, LightingLightInput::point());
    assert_eq!(prepared.surface_scale(), -3.0);
    assert_eq!(prepared.constant(), f64::MAX);
    assert_eq!(prepared.kernel_unit_lengths(), Some([1.5, 2.25]));

    let uniform = admit(
        input.with_kernel_unit_length(1.25),
        LightingLightInput::point(),
    );
    assert_eq!(uniform.kernel_unit_lengths(), Some([1.25, 1.25]));

    for [x, y] in [[-1.0, 5.0], [5.0, -1.0], [-0.0, 5.0]] {
        let prepared = admit(
            input.with_kernel_unit_lengths(x, y),
            LightingLightInput::point(),
        );
        assert_eq!(prepared.kernel_unit_lengths(), None);
    }
}

#[test]
fn specular_defaults_and_explicit_requests_admit_the_shared_surface_parameters() {
    let mut accounting = RenderAccountingContext::default();
    let mut prepare = |input| {
        prepare_specular_lighting(
            input,
            LightingLightInput::point(),
            FilterColorSpace::LinearSrgb,
            &mut accounting,
        )
        .unwrap()
    };
    let default = prepare(SpecularLightingInput::default());
    assert_eq!(default.surface_scale(), 1.0);
    assert_eq!(default.constant(), 1.0);
    assert_eq!(default.material_exponent(), Some(1.0));
    assert_eq!(default.kernel_unit_lengths(), None);
    assert_eq!(
        default.lighting_color(),
        Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap()
    );
    assert_eq!(default.color_space(), FilterColorSpace::LinearSrgb);

    let input = SpecularLightingInput::new()
        .with_surface_scale(-3.5)
        .with_specular_constant(f64::MAX)
        .with_kernel_unit_length(2.5)
        .with_lighting_color(Color::try_rgba(0.25, -0.0, 0.75, 0.0).unwrap());
    let prepared = prepare(input);
    assert_eq!(prepared.surface_scale(), -3.5);
    assert_eq!(prepared.constant(), f64::MAX);
    assert_eq!(prepared.kernel_unit_lengths(), Some([2.5, 2.5]));
    let color = prepared.lighting_color();
    assert_eq!((color.r(), color.b(), color.a()), (0.25, 0.75, 1.0));
    assert_eq!(color.g().to_bits(), 0);

    let prepared = prepare(input.with_kernel_unit_lengths(2.5, 1.25));
    assert_eq!(prepared.kernel_unit_lengths(), Some([2.5, 1.25]));
    let prepared = prepare(
        input
            .with_specular_constant(-0.0)
            .with_kernel_unit_lengths(2.5, -0.0),
    );
    assert_eq!(prepared.kernel_unit_lengths(), None);
    assert_eq!(prepared.constant().to_bits(), 0);
}

#[test]
fn specular_material_exponent_and_spot_focus_clamp_independently() {
    let mut accounting = RenderAccountingContext::default();
    for (material, spot, expected_material, expected_spot) in [
        (f64::MIN, 3.25, 1.0, 3.25),
        (-0.0, 128.0, 1.0, 128.0),
        (1.0, 1.0, 1.0, 1.0),
        (2.75, -7.0, 2.75, 1.0),
        (128.0, f64::MAX, 128.0, 128.0),
        (f64::MAX, 1.5, 128.0, 1.5),
    ] {
        let prepared = prepare_specular_lighting(
            SpecularLightingInput::new().with_specular_exponent(material),
            LightingLightInput::Spot(
                SpotLightInput::new()
                    .with_position([0.0, 0.0, 1.0])
                    .with_specular_exponent(spot),
            ),
            FilterColorSpace::Srgb,
            &mut accounting,
        )
        .unwrap();
        assert_eq!(prepared.material_exponent(), Some(expected_material));
        let PreparedLight::Spot { exponent, .. } = prepared.light() else {
            panic!("expected a spot light");
        };
        assert_eq!(exponent, expected_spot);
    }
}

#[test]
fn admission_uses_straight_light_rgb_and_canonicalizes_signed_zero() {
    let light = LightingLightInput::Point {
        position: [-0.0, 0.0, -0.0],
    };
    let input = DiffuseLightingInput::new()
        .with_surface_scale(-0.0)
        .with_diffuse_constant(-0.0)
        .with_lighting_color(Color::try_rgba(-0.0, 0.25, 0.75, 0.0).unwrap());
    let prepared = prepare_lighting(
        input,
        light,
        FilterColorSpace::LinearSrgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();

    assert_eq!(prepared.surface_scale().to_bits(), 0_u64);
    assert_eq!(prepared.constant().to_bits(), 0_u64);
    assert_eq!(prepared.lighting_color().r().to_bits(), 0_u32);
    assert_eq!(prepared.lighting_color().g(), 0.25);
    assert_eq!(prepared.lighting_color().b(), 0.75);
    assert_eq!(prepared.lighting_color().a(), 1.0);
    assert_eq!(prepared.color_space(), FilterColorSpace::LinearSrgb);
    let PreparedLight::Point { position } = prepared.light() else {
        panic!("expected a point light");
    };
    assert_eq!(position.map(f64::to_bits), [0; 3]);
}

#[test]
fn distant_angles_reduce_before_trigonometry_and_preserve_axis_directions() {
    for (azimuth, elevation, expected) in [
        (0.0, 0.0, [1.0, 0.0, 0.0]),
        (450.0, -0.0, [0.0, 1.0, 0.0]),
        (-180.0, 0.0, [-1.0, 0.0, 0.0]),
        (f64::MAX, 450.0, [0.0, 0.0, 1.0]),
        (f64::MIN, -450.0, [0.0, 0.0, -1.0]),
    ] {
        let prepared = admit(
            DiffuseLightingInput::default(),
            LightingLightInput::Distant { azimuth, elevation },
        );
        assert_eq!(
            prepared.light(),
            PreparedLight::Distant {
                direction: expected
            }
        );
    }
    let angled = admit(
        DiffuseLightingInput::default(),
        LightingLightInput::Distant {
            azimuth: 30.0,
            elevation: 30.0,
        },
    );
    let PreparedLight::Distant { direction } = angled.light() else {
        panic!("expected a distant light");
    };
    assert!((direction[0] - 0.75).abs() < 1e-15);
    assert!((direction[1] - 3.0_f64.sqrt() / 4.0).abs() < 1e-15);
    assert!((direction[2] - 0.5).abs() < 1e-15);
    assert_eq!(
        angled,
        admit(
            DiffuseLightingInput::default(),
            LightingLightInput::Distant {
                azimuth: -330.0,
                elevation: 390.0,
            },
        ),
    );
}

#[test]
fn spot_direction_is_stable_and_cone_presence_survives_canonicalization() {
    let no_axis = admit(
        DiffuseLightingInput::new(),
        LightingLightInput::Spot(SpotLightInput::new()),
    );
    assert_eq!(
        no_axis.light(),
        PreparedLight::Spot {
            position: [0.0; 3],
            direction: None,
            exponent: 1.0,
            cone_angle: None,
        }
    );

    for (raw_exponent, expected_exponent, raw_cone, expected_cone) in [
        (-10.0, 1.0, -0.0, 0.0),
        (1.0, 1.0, -30.0, 30.0),
        (13.25, 13.25, 59.0, 59.0),
        (128.0, 128.0, -110.0, 90.0),
        (f64::MAX, 128.0, f64::MAX, 90.0),
    ] {
        let prepared = admit(
            DiffuseLightingInput::new(),
            LightingLightInput::Spot(
                SpotLightInput::new()
                    .with_position([0.0, 3.25, f64::MAX])
                    .with_points_at([0.0, 3.25, 0.0])
                    .with_specular_exponent(raw_exponent)
                    .with_limiting_cone_angle(raw_cone),
            ),
        );
        assert_eq!(
            prepared.light(),
            PreparedLight::Spot {
                position: [0.0, 3.25, f64::MAX],
                direction: Some([
                    0.0,
                    0.0,
                    if expected_cone == 0.0 {
                        -2.0_f64.next_down()
                    } else {
                        -1.0
                    },
                ]),
                exponent: expected_exponent,
                cone_angle: Some(expected_cone),
            },
        );
    }

    let mut accounting = RenderAccountingContext::default();
    let error = prepare_lighting(
        DiffuseLightingInput::new(),
        LightingLightInput::Spot(
            SpotLightInput::new()
                .with_position([-f64::MAX, 0.0, 0.0])
                .with_points_at([f64::MAX, 0.0, 0.0]),
        ),
        FilterColorSpace::Srgb,
        &mut accounting,
    )
    .unwrap_err();
    assert_eq!(
        error.lighting_diagnostic(),
        Some(&LightingPreparationError::Numeric {
            phase: LightingNumericPhase::LightVector,
            value: f64::INFINITY,
        }),
    );
}

#[test]
fn invalid_parameters_preserve_the_reached_scalar_and_work() {
    use LightingParameter::*;

    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let input = DiffuseLightingInput::new();
        let distant = LightingLightInput::distant();
        let spot = SpotLightInput::new();
        let position = |x, y, z| LightingLightInput::Point {
            position: [x, y, z],
        };
        let spot_target = |x, y, z| LightingLightInput::Spot(spot.with_points_at([x, y, z]));
        for (case, light, parameter, visits) in [
            (input.with_surface_scale(value), distant, SurfaceScale, 1),
            (
                input.with_diffuse_constant(value),
                distant,
                DiffuseConstant,
                2,
            ),
            (
                input.with_kernel_unit_lengths(value, 0.0),
                distant,
                KernelUnitLengthX,
                3,
            ),
            (
                input.with_kernel_unit_lengths(0.0, value),
                distant,
                KernelUnitLengthY,
                4,
            ),
            (
                input,
                LightingLightInput::Distant {
                    azimuth: value,
                    elevation: 0.0,
                },
                Azimuth,
                3,
            ),
            (
                input,
                LightingLightInput::Distant {
                    azimuth: 0.0,
                    elevation: value,
                },
                Elevation,
                4,
            ),
            (input, position(value, 0.0, 0.0), LightX, 3),
            (input, position(0.0, value, 0.0), LightY, 4),
            (input, position(0.0, 0.0, value), LightZ, 5),
            (input, spot_target(value, 0.0, 0.0), PointsAtX, 6),
            (input, spot_target(0.0, value, 0.0), PointsAtY, 7),
            (input, spot_target(0.0, 0.0, value), PointsAtZ, 8),
            (
                input,
                LightingLightInput::Spot(spot.with_specular_exponent(value)),
                SpotExponent,
                9,
            ),
            (
                input,
                LightingLightInput::Spot(spot.with_limiting_cone_angle(value)),
                Cone,
                10,
            ),
        ] {
            assert_parameter_refusal(case, light, parameter, value, visits);
        }
    }

    assert_parameter_refusal(
        DiffuseLightingInput::new().with_diffuse_constant(-0.5),
        LightingLightInput::distant(),
        DiffuseConstant,
        -0.5,
        2,
    );
}

fn assert_parameter_refusal(
    input: DiffuseLightingInput,
    light: LightingLightInput,
    parameter: LightingParameter,
    value: f64,
    visits: u64,
) {
    assert_reached_parameter(
        |account| prepare_lighting(input, light, FilterColorSpace::Srgb, account),
        parameter,
        value,
        visits,
    );
}

fn assert_reached_parameter(
    prepare: impl FnOnce(&mut RenderAccountingContext) -> Result<PreparedLighting>,
    parameter: LightingParameter,
    value: f64,
    visits: u64,
) {
    let mut accounting = RenderAccountingContext::default();
    let error = prepare(&mut accounting).unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidInput);
    let Some(LightingPreparationError::InvalidParameter {
        parameter: actual,
        value: actual_value,
    }) = error.lighting_diagnostic()
    else {
        panic!("expected a lighting parameter diagnostic");
    };
    assert_eq!(*actual, parameter);
    assert_eq!(actual_value.to_bits(), value.to_bits());
    let report = accounting.last_attempt().unwrap();
    assert_eq!(report.kind(), RenderAttemptKind::Lighting);
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(ErrorCode::InvalidInput)
    );
    let usage = report
        .attempt()
        .unwrap()
        .phase_usage(RenderPhase::Preparation);
    assert_eq!(usage.work(RenderWork::CoordinateVisits), visits);
    assert_eq!(usage.work(RenderWork::FilterOperationVisits), 1);
}

#[test]
fn specular_invalid_material_inputs_precede_grid_and_light_and_report_their_own_parameters() {
    let default = SpecularLightingInput::new();
    let spot = SpotLightInput::new().with_specular_exponent(f64::INFINITY);
    let invalid_spot = LightingLightInput::Spot(spot);
    let invalid_y = default.with_kernel_unit_lengths(0.0, f64::NEG_INFINITY);
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_reached_parameter(
            |account| {
                prepare_specular_lighting(
                    invalid_y
                        .with_specular_constant(value)
                        .with_specular_exponent(value),
                    invalid_spot,
                    FilterColorSpace::Srgb,
                    account,
                )
            },
            LightingParameter::SpecularConstant,
            value,
            2,
        );
        assert_reached_parameter(
            |account| {
                prepare_specular_lighting(
                    invalid_y.with_specular_exponent(value),
                    invalid_spot,
                    FilterColorSpace::Srgb,
                    account,
                )
            },
            LightingParameter::MaterialExponent,
            value,
            3,
        );
    }
    for (input, parameter, value, visits) in [
        (
            invalid_y.with_specular_constant(-0.5),
            LightingParameter::SpecularConstant,
            -0.5,
            2,
        ),
        (
            invalid_y,
            LightingParameter::KernelUnitLengthY,
            f64::NEG_INFINITY,
            5,
        ),
        (default, LightingParameter::SpotExponent, f64::INFINITY, 10),
    ] {
        assert_reached_parameter(
            |account| {
                prepare_specular_lighting(input, invalid_spot, FilterColorSpace::Srgb, account)
            },
            parameter,
            value,
            visits,
        );
    }
}

#[test]
fn preparation_limits_refuse_before_reached_validation_and_allow_retry() {
    let input = DiffuseLightingInput::new().with_diffuse_constant(f64::NAN);
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CoordinateVisits, 1)
        .unwrap()
        .with_scratch_bytes(0)
        .with_retained_bytes(0);
    let options = RenderAccountingOptions::new();
    let mut accounting = RenderAccountingContext::new(options.with_limits(limits));
    let prepare = |input, accounting: &mut RenderAccountingContext| {
        prepare_lighting(
            input,
            LightingLightInput::distant(),
            FilterColorSpace::Srgb,
            accounting,
        )
    };
    let refused = prepare(input, &mut accounting).unwrap_err();
    assert!(matches!(
        refused.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(refusal))
            if refusal.quantity() == RenderQuantity::Work(RenderWork::CoordinateVisits)
    ));
    let report = accounting.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CoordinateVisits),
        1
    );
    accounting.set_options(
        options.with_limits(
            limits
                .with_work_limit(RenderWork::CoordinateVisits, 7)
                .unwrap(),
        ),
    );

    let prepared = prepare(input.with_diffuse_constant(1.5), &mut accounting).unwrap();
    assert_eq!(prepared.constant(), 1.5);
    let report = accounting.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::CoordinateVisits), 7);
    assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
}

#[test]
fn scene_lighting_owns_the_real_children_and_keeps_empty_planes_through_normalization() {
    let mut accounting = RenderAccountingContext::default();
    let diffuse = admit(DiffuseLightingInput::new(), LightingLightInput::distant());
    let specular = prepare_specular_lighting(
        SpecularLightingInput::new().with_specular_exponent(7.0),
        LightingLightInput::distant(),
        FilterColorSpace::Srgb,
        &mut accounting,
    )
    .unwrap();
    let domains = FilterDomains::try_new(
        Rect::try_new(0.0, 0.0, 0.0, 0.0).unwrap(),
        Rect::try_new(-1.0, -2.0, 8.0, 7.0).unwrap(),
        Rect::try_new(0.5, 0.25, 2.0, 3.0).unwrap(),
    )
    .unwrap();
    let transform = Transform::translation(3.0, -4.0).unwrap();
    let composition = LayerComposition::normal_porter_duff(PorterDuffOperator::Copy);
    let layer = Layer::new()
        .isolated()
        .with_transform(transform)
        .try_opacity(0.375)
        .unwrap()
        .with_composition(composition)
        .try_filter(Filter::try_blur(2.0).unwrap())
        .unwrap();
    let mut child = Scene::new();
    child.fill(domains.result(), Color::BLACK);
    let mut scene = Scene::new();
    scene.lighting_layer(
        layer.clone(),
        diffuse,
        LightingSource::finite(child, domains),
    );
    scene.lighting_layer(
        layer.clone(),
        diffuse,
        LightingSource::finite(Scene::new(), domains),
    );
    scene.lighting_layer(
        layer,
        specular,
        LightingSource::finite(Scene::new(), domains),
    );

    let summary = scene.clone().inspect(&mut accounting).unwrap();
    assert_eq!((summary.commands, summary.layers, summary.fills), (4, 3, 1));
    accounting
        .run(
            RenderAttemptKind::Frame,
            RenderPhase::Normalize,
            |account| {
                let normalized = scene.normalize(Capabilities::CURRENT, account)?;
                assert_eq!(normalized.commands.len(), 3);
                let expected_layers = [
                    (
                        1,
                        diffuse,
                        PrimitiveOperation::GpuDiffuseLightingFilterExecution,
                    ),
                    (
                        0,
                        diffuse,
                        PrimitiveOperation::GpuDiffuseLightingFilterExecution,
                    ),
                    (
                        0,
                        specular,
                        PrimitiveOperation::GpuSpecularLightingFilterExecution,
                    ),
                ];
                for (command, (child_count, operation, primitive)) in
                    normalized.commands.iter().zip(expected_layers)
                {
                    let RenderCommand::Layer { layer, children } = command else {
                        panic!("expected a normalized lighting layer");
                    };
                    assert_eq!(children.len(), child_count);
                    assert_eq!(layer.transform, transform);
                    assert_eq!(layer.opacity, 0.375);
                    assert_eq!(layer.composition, composition);
                    assert!(layer.explicit_isolation);
                    let Some(NormalizedForegroundEffect::Lighting(lighting)) =
                        &layer.foreground_filters
                    else {
                        panic!("expected the admitted lighting foreground");
                    };
                    assert_eq!(lighting.operation, operation);
                    assert_eq!(lighting.operation.primitive_operation(), primitive);
                    assert_eq!(lighting.source, domains);
                }
                Ok(())
            },
        )
        .unwrap();
}

#[test]
fn capability_and_work_surface_recognize_only_supported_attempts() {
    let caps = Capabilities::CURRENT;
    assert!(
        caps.filters()
            .supports_gpu_diffuse_lighting_filter_execution()
    );
    assert!(
        caps.filters()
            .supports_gpu_specular_lighting_filter_execution()
    );
    assert!(
        caps.ensure_supported(UnsupportedPrimitive::new(
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuDiffuseLightingFilterExecution,
        ))
        .is_ok()
    );
    assert!(
        caps.ensure_supported(UnsupportedPrimitive::new(
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuSpecularLightingFilterExecution,
        ))
        .is_ok()
    );
    for work in [
        RenderWork::ScheduledLightingAlphaSamples,
        RenderWork::ScheduledLightingEvaluations,
        RenderWork::ScheduledLightingResultSamples,
    ] {
        let limits = RenderLimits::new().with_work_limit(work, 10).unwrap();
        assert!(limits.validate(RenderAttemptKind::Frame).is_ok());
        assert!(limits.validate(RenderAttemptKind::Lighting).is_err());
    }
}

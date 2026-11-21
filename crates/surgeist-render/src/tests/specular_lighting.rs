//! Public specular admission, real GPU shading, and published pixels.

use super::convolution::{assert_pixel, bounds, image, renderer_and_surface};
use super::displacement::FORMATS;
use super::lighting::VERTICAL;
use super::native_shadow::{render, source_scene};
use crate::{
    accounting::{
        RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
        RenderAttemptStatus, RenderLimits, RenderPhase, RenderProfile, RenderQuantity,
        RenderRefusalReason, RenderWork,
    },
    effect::{FilterColorSpace, FilterDomains},
    geometry::{Rect, Size, Transform},
    layer::{BlendMode, LayerComposition, PorterDuffOperator},
    lighting::{
        LightingLightInput, LightingSource, SpecularLightingInput, SpotLightInput,
        prepare_specular_lighting,
    },
    paint::Color,
    renderer::{Options, Renderer},
    scene::{Layer, Scene},
    stats::RenderRoute,
    surface::Parameters,
};
use std::f64::consts::FRAC_1_SQRT_2;

fn specular(
    bump: Scene,
    domains: FilterDomains,
    material: SpecularLightingInput,
    light: LightingLightInput,
    working: FilterColorSpace,
    layer: Layer,
) -> Scene {
    let mut scene = Scene::new();
    let operation = prepare_specular_lighting(
        material,
        light,
        working,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    scene.lighting_layer(layer, operation, LightingSource::finite(bump, domains));
    scene
}

fn full_domains(width: u32, height: u32) -> FilterDomains {
    let full = bounds(width, height);
    FilterDomains::try_new(full, full, full).unwrap()
}

fn white_highlight(alpha: u8) -> [u8; 4] {
    if alpha == 0 {
        [0; 4]
    } else {
        [255, 255, 255, alpha]
    }
}

#[test]
fn flat_specular_highlights_use_halfway_angle_and_independent_spot_focus() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let material = SpecularLightingInput::new().with_specular_exponent(4.0);
    let position = [0.5, 0.5, 1.0];
    let spot = SpotLightInput::new()
        .with_position(position)
        .with_points_at([0.5, 0.5, 0.0])
        .with_specular_exponent(3.0);
    // At the second pixel the point and spot have a 45° incident angle.
    // Specular alpha is cos⁴(22.5°), with an extra cos³(45°) for spot
    // focus. The chosen tapered cone cuts halfway through the 0.016 cosine
    // ramp there. A horizontal distant light gives cos⁴(45°) = 1/4.
    let half_taper_at_45 = (FRAC_1_SQRT_2 - 0.008).acos().to_degrees();
    for (light, alpha) in [
        (LightingLightInput::distant(), [64, 64]),
        (LightingLightInput::Point { position }, [255, 186]),
        (LightingLightInput::Spot(spot), [255, 66]),
        (
            LightingLightInput::Spot(spot.with_limiting_cone_angle(half_taper_at_45)),
            [255, 33],
        ),
        (
            LightingLightInput::Spot(spot.with_limiting_cone_angle(30.0)),
            [255, 0],
        ),
        (
            LightingLightInput::Spot(spot.with_limiting_cone_angle(0.0)),
            [255, 0],
        ),
    ] {
        let scene = specular(
            Scene::new(),
            full_domains(2, 1),
            material,
            light,
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        for (x, expected) in (0..).zip(alpha) {
            assert_pixel(&pixels, x, 0, white_highlight(expected), 2);
        }
    }
}

#[test]
fn specular_clips_working_rgb_and_uses_its_maximum_as_alpha() {
    let (mut renderer, mut surface) = renderer_and_surface(1, 1);
    let color = Color::try_rgba(0.25, 0.75, 0.5, 0.0).unwrap();
    // With gain 3/4, sRGB premul is (3/16, 9/16, 3/8), so straight
    // RGB is (1/3, 1, 2/3) and alpha is 9/16. In linear sRGB the decoded
    // channels are about (.05088, .52252, .21404). The green-channel maximum
    // yields alpha .39189 and straight RGB is encoded after division by alpha.
    // Green stays full in straight readback even when alpha is fractional.
    // With gain 2, sRGB clips both green and blue, and linear clips only green.
    for format in FORMATS {
        for (space, gain, expected) in [
            (FilterColorSpace::Srgb, 0.75, [85, 255, 170, 143]),
            (FilterColorSpace::LinearSrgb, 0.75, [88, 255, 171, 100]),
            (FilterColorSpace::Srgb, 2.0, [128, 255, 255, 255]),
            (FilterColorSpace::LinearSrgb, 2.0, [90, 255, 175, 255]),
        ] {
            let scene = specular(
                Scene::new(),
                full_domains(1, 1),
                SpecularLightingInput::new()
                    .with_specular_constant(gain)
                    .with_lighting_color(color),
                VERTICAL,
                space,
                Layer::new(),
            );
            let stats =
                pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                    &mut surface,
                    &scene,
                    Parameters::default(),
                    format,
                ))
                .unwrap();
            assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            assert_pixel(&pixels, 0, 0, expected, 3);
        }
    }
}

#[test]
fn zero_incident_and_antipodal_light_are_transparent_but_a_grazing_halfway_is_finite() {
    let (mut renderer, mut surface) = renderer_and_surface(1, 1);
    let material = SpecularLightingInput::new().with_specular_constant(1e8);
    let min_normal = f64::from(f32::MIN_POSITIVE);
    // Exact antipodes have no halfway direction. In the last case the
    // incident ray is 10⁻⁸ radians from the antipode. The flat normal sees
    // sin(0.5 × 10⁻⁸), which gives alpha 1/2 at this gain. Rounding the
    // unit incident z to -1 before adding the eye vector loses the highlight.
    for (light, alpha) in [
        (
            LightingLightInput::Distant {
                azimuth: 0.0,
                elevation: -90.0,
            },
            0,
        ),
        (
            LightingLightInput::Point {
                position: [0.5, 0.5, -1.0],
            },
            0,
        ),
        (
            LightingLightInput::Point {
                position: [0.5, 0.5, 0.0],
            },
            0,
        ),
        (
            LightingLightInput::Point {
                position: [0.5, 0.5, min_normal * 0.5],
            },
            0,
        ),
        (
            LightingLightInput::Point {
                position: [0.5, 0.5, min_normal * 2.0],
            },
            255,
        ),
        (LightingLightInput::Spot(SpotLightInput::new()), 0),
        (
            LightingLightInput::Point {
                position: [0.5 + 1e-8, 0.5, -1.0],
            },
            128,
        ),
    ] {
        let scene = specular(
            Scene::new(),
            full_domains(1, 1),
            material,
            light,
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&pixels, 0, 0, white_highlight(alpha), 1);
    }
}

#[test]
fn a_normal_grazing_alignment_survives_when_its_square_would_underflow() {
    let (mut renderer, mut surface) = renderer_and_surface(1, 1);
    let filter = Rect::new(-0.5, 0.0, 2.0, 1.0);
    let output = bounds(1, 1);
    let offset = 2.0_f64.powi(-80);
    let scene = specular(
        Scene::new(),
        FilterDomains::try_new(output, filter, output).unwrap(),
        SpecularLightingInput::new().with_specular_constant(offset.recip()),
        LightingLightInput::Point {
            position: [offset, 0.5, -1.0],
        },
        FilterColorSpace::Srgb,
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // At grid node x=0 the light is about 2⁻⁸⁰ radians from the antipode,
    // so the halfway alignment is about 2⁻⁸¹ and gain lifts it to 1/2.
    // At node x=1 the highlight saturates. Their midpoint has alpha 3/4.
    // Squaring the first horizontal component prematurely loses 2⁻¹⁶⁰ in f32.
    assert_pixel(&pixels, 0, 0, white_highlight(191), 1);
}

#[test]
fn an_asymmetric_bump_tilts_the_normal_toward_one_of_two_opposite_lights() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 3);
    let full = bounds(3, 3);
    let pixels = [255, 0, 0, 0, 0, 255, 0, 0, 0]
        .into_iter()
        .flat_map(|alpha| [0, 0, 0, alpha])
        .collect();
    let bump = source_scene(image(3, 3, pixels), full);
    // The center Sobel slopes are (+1/4, -1/4), giving N=(-1,1,4)/sqrt(18).
    // The two horizontal lights give H=(0,±1,1)/sqrt(2). Squaring the dot
    // products gives alpha 25/36 and 1/4, respectively.
    for (azimuth, alpha) in [(90.0, 177), (270.0, 64)] {
        let scene = specular(
            bump.clone(),
            full_domains(3, 3),
            SpecularLightingInput::new().with_specular_exponent(2.0),
            LightingLightInput::Distant {
                azimuth,
                elevation: 0.0,
            },
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&pixels, 1, 1, white_highlight(alpha), 2);
    }
}

#[test]
fn explicit_grid_reconstructs_premul_highlights_before_placing_the_bounded_result() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 7);
    let full = bounds(5, 1);
    let result = Rect::new(1.25, 0.0, 2.5, 1.0);
    let turn = Transform::try_new([0.0, 1.0, -1.0, 0.0, 2.0, 1.0]).unwrap();
    let row = [0, 0, 255, 255, 255]
        .into_iter()
        .flat_map(|alpha| [255, 0, 0, alpha])
        .collect();
    let scene = specular(
        source_scene(image(5, 1, row), full),
        FilterDomains::try_new(full, full, result).unwrap(),
        SpecularLightingInput::new()
            .with_kernel_unit_lengths(2.0, 1.0)
            .with_specular_exponent(2.0),
        VERTICAL,
        FilterColorSpace::Srgb,
        Layer::new().with_transform(turn),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // At x=1,3,5 the grid samples alpha 0,1,1/2. With slopes 1,1/4,-1/2,
    // the highlight alpha at those nodes is N_z² = 1/2,16/17,4/5.
    // Interpolate at x=1.5,2.5,3.5, then apply coverage 3/4,1,3/4.
    // The quarter turn maps these three cells to column 1, rows 2–4.
    for (y, alpha) in (0..).zip([0, 0, 117, 212, 173, 0, 0]) {
        assert_pixel(&pixels, 1, y, white_highlight(alpha), 2);
        for x in [0, 2] {
            assert_pixel(&pixels, x, y, [0; 4], 0);
        }
    }
}

#[test]
fn copying_a_transparent_highlight_clears_only_the_bounded_output() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let output = Rect::new(1.0, 0.0, 1.0, 1.0);
    let copy =
        LayerComposition::try_porter_duff(PorterDuffOperator::Copy, BlendMode::Normal).unwrap();
    let operation = prepare_specular_lighting(
        SpecularLightingInput::new().with_specular_constant(0.0),
        VERTICAL,
        FilterColorSpace::Srgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.fill(full, Color::BLACK);
    scene.lighting_layer(
        Layer::new().with_composition(copy),
        operation,
        LightingSource::finite(
            Scene::new(),
            FilterDomains::try_new(full, full, output).unwrap(),
        ),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&pixels, 0, 0, [0, 0, 0, 255], 0);
    assert_pixel(&pixels, 1, 0, [0; 4], 0);
    assert_pixel(&pixels, 2, 0, [0, 0, 0, 255], 0);
}

#[test]
fn specular_ledger_charges_encoded_work_and_refuses_one_under_without_publication() {
    let evaluations = RenderWork::ScheduledLightingEvaluations;
    let work = [
        RenderWork::ScheduledLightingAlphaSamples,
        evaluations,
        RenderWork::ScheduledLightingResultSamples,
    ];
    let full = bounds(2, 1);
    let mut original = Scene::new();
    original.fill(full, Color::BLACK);

    // Each pass has two cells. Resampling admits 2×4 alpha loads, shade
    // admits 2×9 more and two evaluations together, and resolve admits 2×4
    // result samples. The absent bump omits all alpha traffic.
    for (textured, cap, expected) in [
        (false, 2, [0, 2, 8]),
        (true, 2, [26, 2, 8]),
        (true, 1, [8, 0, 0]),
    ] {
        let options = RenderAccountingOptions::new()
            .with_limits(
                RenderLimits::new()
                    .with_work_limit(evaluations, cap)
                    .unwrap(),
            )
            .with_profile(RenderProfile::Summary { timing: true });
        let mut renderer =
            pollster::block_on(Renderer::new(Options::default().with_accounting(options))).unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(2.0, 1.0), 1.0)).unwrap();
        let old_stats =
            pollster::block_on(renderer.render(&mut surface, &original, Parameters::default()))
                .unwrap();
        let old_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();

        let mut bump = Scene::new();
        if textured {
            bump.fill(full, Color::BLACK);
        }
        let scene = specular(
            bump,
            full_domains(2, 1),
            SpecularLightingInput::new().with_specular_constant(0.5),
            VERTICAL,
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        let result =
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()));
        let report = renderer.last_attempt().unwrap();
        let attempt = report.attempt().unwrap();
        assert_eq!(report.cleanup_fault(), None);
        assert_eq!(work.map(|kind| attempt.usage().work(kind)), expected);
        for phase in [
            RenderPhase::Plan,
            RenderPhase::GpuPrepare,
            RenderPhase::Encode,
        ] {
            assert!(attempt.phase_count(phase) > 0, "{phase:?}");
            assert!(attempt.phase_elapsed(phase).is_some(), "{phase:?}");
        }
        let encoded = attempt.phase_usage(RenderPhase::Encode);
        assert_eq!(work.map(|kind| encoded.work(kind)), expected);

        if cap == 1 {
            let error = result.unwrap_err();
            let refusal = attempt.refusal().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(refusal.phase(), RenderPhase::Attempt);
            assert_eq!(refusal.quantity(), RenderQuantity::Work(evaluations));
            assert_eq!(refusal.current(), 0);
            assert_eq!(refusal.amount(), 2);
            assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: cap });
            assert_eq!(
                error.accounting_diagnostic(),
                Some(RenderAccountingError::Refused(refusal))
            );
            assert_eq!(attempt.phase_count(RenderPhase::Submit), 0);
            assert_eq!(attempt.phase_count(RenderPhase::Publish), 0);
            assert_eq!(renderer.stats(), old_stats);
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                old_pixels
            );
        } else {
            let stats = result.unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
            assert_eq!(renderer.stats(), stats);
            for phase in [RenderPhase::Submit, RenderPhase::Publish] {
                assert!(attempt.phase_count(phase) > 0);
                assert!(attempt.phase_elapsed(phase).is_some());
            }
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for x in 0..2 {
                assert_pixel(&pixels, x, 0, white_highlight(128), 1);
            }
        }
    }
}

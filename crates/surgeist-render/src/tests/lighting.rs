//! Public native diffusion through the real GPU graph and readback.

use super::convolution::{assert_pixel, bounds, image, renderer_and_surface};
use super::native_shadow::{render, source_scene};
use crate::{
    accounting::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptStatus, RenderLimits,
        RenderPhase, RenderProfile, RenderQuantity, RenderWork,
    },
    effect::{FilterColorSpace, FilterDomains},
    geometry::{Rect, Size, Transform},
    lighting::{
        DiffuseLightingInput, LightingLightInput, LightingSource, SpotLightInput, prepare_lighting,
    },
    paint::Color,
    renderer::{Options, Renderer},
    scene::{Layer, Scene},
    surface::Parameters,
};

pub(super) fn diffuse(
    bump: Scene,
    domains: FilterDomains,
    material: DiffuseLightingInput,
    light: LightingLightInput,
    working: FilterColorSpace,
    layer: Layer,
) -> Scene {
    let mut scene = Scene::new();
    let mut account = RenderAccountingContext::default();
    let operation = prepare_lighting(material, light, working, &mut account).unwrap();
    scene.lighting_layer(layer, operation, LightingSource::finite(bump, domains));
    scene
}

pub(super) const VERTICAL: LightingLightInput = LightingLightInput::Distant {
    azimuth: 0.0,
    elevation: 90.0,
};

#[test]
fn empty_bump_shades_an_opaque_plane_with_distant_point_and_spot_equations() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let full = bounds(2, 1);
    let domains = FilterDomains::try_new(full, full, full).unwrap();
    let white = DiffuseLightingInput::new();
    let spot = SpotLightInput::new()
        .with_position([0.5, 0.5, 1.0])
        .with_points_at([0.5, 0.5, 0.0])
        .with_specular_exponent(2.0);
    for (light, expected) in [
        (VERTICAL, [255, 255]),
        (
            LightingLightInput::Point {
                position: [0.5, 0.5, 1.0],
            },
            [255, 180],
        ),
        (
            LightingLightInput::Point {
                position: [0.5, 0.5, 0.0],
            },
            [0, 0],
        ),
        (LightingLightInput::Spot(spot), [255, 90]),
        (
            LightingLightInput::Spot(spot.with_limiting_cone_angle(30.0)),
            [255, 0],
        ),
        (
            LightingLightInput::Spot(spot.with_limiting_cone_angle(0.0)),
            [255, 0],
        ),
        (LightingLightInput::Spot(SpotLightInput::new()), [0, 0]),
    ] {
        let scene = diffuse(
            Scene::new(),
            domains,
            white,
            light,
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        // The off-axis point sees cos 45°, and spot focus 2 adds cos² 45°.
        // Both finite cones exclude that sample. Point coincidence and a zero
        // spot axis contribute no light, while diffuse alpha stays opaque.
        for (x, value) in (0..).zip(expected) {
            assert_pixel(&pixels, x, 0, [value, value, value, 255], 2);
        }
    }
}

#[test]
fn finite_bump_edges_and_singleton_axes_use_the_selected_one_sided_sobel() {
    for (width, height, columns, expected) in [
        (1, 1, vec![128], vec![255]),
        (2, 1, vec![0, 255], vec![114, 114]),
        (3, 3, vec![0, 128, 255], vec![180, 180, 181]),
    ] {
        let (mut renderer, mut surface) = renderer_and_surface(width, height);
        let full = bounds(width, height);
        let row: Vec<u8> = columns
            .into_iter()
            .flat_map(|alpha| [0, 0, 255, alpha])
            .collect();
        let bump = source_scene(image(width, height, row.repeat(height as usize)), full);
        let rendered = render(
            &mut renderer,
            &mut surface,
            &diffuse(
                bump,
                FilterDomains::try_new(full, full, full).unwrap(),
                DiffuseLightingInput::new(),
                VERTICAL,
                FilterColorSpace::Srgb,
                Layer::new(),
            ),
        );
        // Alpha is replicated along Y so its Y derivative is zero. The
        // midpoint gain is A(+1)-A(-1); ends use twice the one-step change.
        for y in 0..height {
            for (x, value) in (0..).zip(expected.iter().copied()) {
                assert_pixel(&rendered, x, y, [value, value, value, 255], 2);
            }
        }
    }
}

#[test]
fn asymmetric_bump_uses_two_dimensional_sobel_weights() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 3);
    let full = bounds(3, 3);
    let pixels = [255, 0, 0, 0, 0, 255, 0, 0, 0]
        .into_iter()
        .flat_map(|a| [0, 0, 0, a])
        .collect();
    let bump = source_scene(image(3, 3, pixels), full);
    let scene = diffuse(
        bump,
        FilterDomains::try_new(full, full, full).unwrap(),
        DiffuseLightingInput::new(),
        LightingLightInput::Distant {
            azimuth: 90.0,
            elevation: 0.0,
        },
        FilterColorSpace::Srgb,
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // At the center Gx=+1/4 and Gy=-1/4. The top-left corner has
    // Gx=Gy=-4/3. The light points along +Y, so it sees the normal's Y.
    assert_pixel(&pixels, 1, 1, [60, 60, 60, 255], 2);
    assert_pixel(&pixels, 0, 0, [159, 159, 159, 255], 2);
}

#[test]
fn explicit_spacing_reconstructs_lit_nodes_on_the_filter_anchored_grid() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 1);
    let full = bounds(5, 1);
    let bump = source_scene(
        image(
            5,
            1,
            [0, 0, 255, 255, 255]
                .into_iter()
                .flat_map(|a| [255, 0, 0, a])
                .collect(),
        ),
        full,
    );
    let material = DiffuseLightingInput::new().with_kernel_unit_lengths(2.0, 1.0);
    for output in [full, Rect::new(2.0, 0.0, 1.0, 1.0)] {
        let scene = diffuse(
            bump.clone(),
            FilterDomains::try_new(full, full, output).unwrap(),
            material,
            VERTICAL,
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        // Kernel nodes x=1,3,5 sample A=0,1,1/2. Their signed slopes
        // are 1,1/4,-1/2. Reconstruct their normalized Z components.
        for (x, value) in (0..).zip([180, 197, 231, 243, 233]) {
            let expected = if output == full || x == 2 {
                [value, value, value, 255]
            } else {
                [0; 4]
            };
            assert_pixel(&pixels, x, 0, expected, 2);
        }
    }
}

#[test]
fn diffuse_clips_each_working_component_and_applies_output_coverage_once() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let output = Rect::new(0.25, 0.0, 1.5, 1.0);
    let material = DiffuseLightingInput::new()
        .with_diffuse_constant(2.0)
        .with_lighting_color(Color::try_rgba(0.25, 0.5, 1.0, 0.0).unwrap());
    let domains = FilterDomains::try_new(full, full, output).unwrap();
    for (working, expected) in [
        (FilterColorSpace::Srgb, [128, 255, 255, 191]),
        (FilterColorSpace::LinearSrgb, [90, 176, 255, 191]),
    ] {
        let pixels = render(
            &mut renderer,
            &mut surface,
            &diffuse(
                Scene::new(),
                domains,
                material,
                VERTICAL,
                working,
                Layer::new(),
            ),
        );
        // Coverage 3/4 applies to premul after shading; readback returns straight.
        assert_pixel(&pixels, 0, 0, expected, 3);
        assert_pixel(&pixels, 1, 0, expected, 3);
        assert_pixel(&pixels, 2, 0, [0; 4], 0);
    }
}

#[test]
fn coverage_uses_one_intersection_of_the_fractional_filter_and_result() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let full = bounds(2, 1);
    let filter = Rect::new(0.25, 0.0, 1.5, 1.0);
    let result = Rect::new(0.75, 0.0, 0.5, 1.0);
    let scene = diffuse(
        Scene::new(),
        FilterDomains::try_new(full, filter, result).unwrap(),
        DiffuseLightingInput::new(),
        VERTICAL,
        FilterColorSpace::Srgb,
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // Each cell intersects O by 1/4. Multiplying independent F and P
    // coverage would incorrectly reduce that to 3/16.
    for x in 0..2 {
        assert_pixel(&pixels, x, 0, [255, 255, 255, 64], 1);
    }
}

#[test]
fn lighting_color_subnormals_are_effectively_zero_before_strength_amplifies_rgb() {
    let (mut renderer, mut surface) = renderer_and_surface(1, 1);
    let full = bounds(1, 1);
    let domains = FilterDomains::try_new(full, full, full).unwrap();
    let material = DiffuseLightingInput::new()
        .with_lighting_color(
            Color::try_rgba(f32::MIN_POSITIVE / 2.0, f32::MIN_POSITIVE * 2.0, 1.0, 1.0).unwrap(),
        )
        .with_diffuse_constant(2.0_f64.powi(126));
    for (space, expected) in [
        (FilterColorSpace::Srgb, [0, 255, 255, 255]),
        (FilterColorSpace::LinearSrgb, [0, 0, 255, 255]),
    ] {
        let scene = diffuse(
            Scene::new(),
            domains,
            material,
            VERTICAL,
            space,
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&pixels, 0, 0, expected, 1);
    }
}

#[test]
fn ledger_counts_scheduled_lighting_opportunities_without_sampling_an_absent_bump() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 2);
    let full = bounds(3, 2);
    let domains = FilterDomains::try_new(full, full, full).unwrap();
    for (textured, expected_alpha_loads) in [(false, 0), (true, 78)] {
        let mut bump = Scene::new();
        if textured {
            bump.fill(full, Color::BLACK);
        }
        let scene = diffuse(
            bump,
            domains,
            DiffuseLightingInput::new(),
            VERTICAL,
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        render(&mut renderer, &mut surface, &scene);
        let usage = renderer.last_attempt().unwrap().attempt().unwrap().usage();
        // Each of the three stage windows has six cells. A textured bump
        // needs 6×4 resampling plus 6×9 shading loads. Resolve needs 6×4.
        assert_eq!(
            usage.work(RenderWork::ScheduledLightingAlphaSamples),
            expected_alpha_loads
        );
        assert_eq!(usage.work(RenderWork::ScheduledLightingEvaluations), 6);
        assert_eq!(usage.work(RenderWork::ScheduledLightingResultSamples), 24);
    }
}

#[test]
fn a_rotated_layer_places_the_lit_plane_with_its_local_light_unchanged() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 3);
    let bounds = bounds(2, 1);
    let quarter_turn = Transform::try_new([0.0, 1.0, -1.0, 0.0, 2.0, 1.0]).unwrap();
    let scene = diffuse(
        Scene::new(),
        FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
        DiffuseLightingInput::new(),
        LightingLightInput::Point {
            position: [0.5, 0.5, 1.0],
        },
        FilterColorSpace::Srgb,
        Layer::new().with_transform(quarter_turn),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // The two local centers rotate to surface (1.5, 1.5) and (1.5, 2.5).
    assert_pixel(&pixels, 1, 1, [255; 4], 1);
    assert_pixel(&pixels, 1, 2, [180, 180, 180, 255], 2);
    assert_pixel(&pixels, 2, 1, [0; 4], 0);
    assert_pixel(&pixels, 1, 0, [0; 4], 0);
}

#[test]
fn point_lighting_rebases_a_small_reached_window_of_a_large_filter_canvas() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let output = bounds(2, 1);
    let filter = Rect::new(-5e15, 0.0, 5e15 + 2.0, 1.0);
    let scene = diffuse(
        Scene::new(),
        FilterDomains::try_new(output, filter, output).unwrap(),
        DiffuseLightingInput::new().with_kernel_unit_length(1.0),
        LightingLightInput::Point {
            position: [0.25, 0.5, 0.25],
        },
        FilterColorSpace::Srgb,
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // The reached centers stay at 0.5 and 1.5 despite the large full-canvas
    // indices. The light's horizontal offsets are therefore 0.25 and 1.25.
    assert_pixel(&pixels, 0, 0, [180, 180, 180, 255], 2);
    assert_pixel(&pixels, 1, 0, [50, 50, 50, 255], 2);
}

#[test]
fn lighting_precision_point_result_is_independent_of_the_reached_window_origin() {
    let (mut renderer, mut surface) = renderer_and_surface(1026, 1);
    let full = bounds(1026, 1);
    let light = LightingLightInput::Point {
        position: [1024.50001, 0.5, 0.000001],
    };
    for output in [full, Rect::new(1024.0, 0.0, 1.0, 1.0)] {
        let scene = diffuse(
            Scene::new(),
            FilterDomains::try_new(full, full, output).unwrap(),
            DiffuseLightingInput::new(),
            light,
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        // dx=10^-5 and dz=10^-6, so cos incidence = 1/sqrt(101).
        assert_pixel(&pixels, 1024, 0, [25, 25, 25, 255], 1);
    }
}

#[test]
fn lighting_precision_point_uses_the_default_raster_phase_at_nonbinary_density() {
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(1.0, 1.0), 3.0)).unwrap();
    let output = bounds(1, 1);
    let filter = Rect::new(-2e15, 0.0, 2e15 + 1.0, 1.0);
    let scene = diffuse(
        Scene::new(),
        FilterDomains::try_new(output, filter, output).unwrap(),
        DiffuseLightingInput::new(),
        LightingLightInput::Point {
            position: [1.0 / 6.0, 1.0 / 6.0, 0.1],
        },
        FilterColorSpace::Srgb,
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // At density 3 the default grid exactly follows raster texel centers.
    assert_pixel(&pixels, 0, 0, [255; 4], 1);
}

#[test]
fn lighting_precision_default_grid_recognizes_the_rounded_local_center() {
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(1.0, 1.0), 3.0)).unwrap();
    let full = bounds(1, 1);
    let cropped = Rect::new(2.0 / 3.0, 0.0, 1.0 / 3.0, 1.0);
    let scene = diffuse(
        Scene::new(),
        FilterDomains::try_new(full, full, cropped).unwrap(),
        DiffuseLightingInput::new(),
        LightingLightInput::Point {
            position: [5.0 / 6.0, 1.0 / 6.0, 1e-19],
        },
        FilterColorSpace::Srgb,
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // The represented local center agrees with the light's X and Y. A
    // round-trip to raster coordinates must not invent horizontal offset.
    assert_pixel(&pixels, 2, 0, [255; 4], 1);
}

#[test]
fn lighting_precision_near_height_coincidence_preserves_incident_direction() {
    let (mut renderer, mut surface) = renderer_and_surface(1, 1);
    let full = bounds(1, 1);
    let mut bump = Scene::new();
    bump.fill(full, Color::BLACK);
    for (height, value) in [(1.00000001, 255), (1.0, 0), (0.99999999, 0)] {
        let scene = diffuse(
            bump.clone(),
            FilterDomains::try_new(full, full, full).unwrap(),
            DiffuseLightingInput::new(),
            LightingLightInput::Point {
                position: [0.5, 0.5, height],
            },
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        // The opaque bump has height 1 and the light has identical XY.
        // Any positive vertical offset lights the flat plane fully.
        let pixels = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&pixels, 0, 0, [value, value, value, 255], 1);
    }
}

#[test]
fn lighting_precision_zero_cone_excludes_a_near_axis_sample_and_accepts_an_oblique_axis() {
    let (mut renderer, mut surface) = renderer_and_surface(1, 1);
    let full = bounds(1, 1);
    for (position, target, value) in [
        ([0.5001, 0.5, 1.0], [0.5001, 0.5, 0.0], 0),
        ([-0.5, -0.5, 1.0], [1.5, 1.5, -1.0], 147),
        ([0.1, 0.1, 0.6], [0.5, 0.5, 0.0], 186),
        ([-0.46, -0.41, 0.98], [0.5, 0.5, 0.0], 152),
    ] {
        let spot = SpotLightInput::new()
            .with_position(position)
            .with_points_at(target)
            .with_limiting_cone_angle(0.0);
        let scene = diffuse(
            Scene::new(),
            FilterDomains::try_new(full, full, full).unwrap(),
            DiffuseLightingInput::new(),
            LightingLightInput::Spot(spot),
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        // The oblique light axes contain the center (0.5,0.5,0).
        let pixels = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&pixels, 0, 0, [value, value, value, 255], 1);
    }
}

#[test]
fn unrepresentable_reached_lighting_arithmetic_preserves_publication() {
    use crate::lighting::{LightingNumericPhase, LightingPreparationError};
    let (mut renderer, mut surface) = renderer_and_surface(1, 1);
    let full = bounds(1, 1);
    let mut original = Scene::new();
    original.fill(full, Color::BLACK);
    let stats = pollster::block_on(renderer.render(&mut surface, &original, Parameters::default()))
        .unwrap();
    let scene = diffuse(
        Scene::new(),
        FilterDomains::try_new(full, full, full).unwrap(),
        DiffuseLightingInput::new().with_diffuse_constant(f64::MAX),
        VERTICAL,
        FilterColorSpace::Srgb,
        Layer::new(),
    );
    let failure = pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
        .unwrap_err();
    assert_eq!(
        failure.lighting_diagnostic(),
        Some(&LightingPreparationError::Numeric {
            phase: LightingNumericPhase::ColorProduct,
            value: f64::MAX,
        }),
    );
    assert_eq!(renderer.stats(), stats);
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_pixel(&pixels, 0, 0, [0, 0, 0, 255], 0);
}

#[test]
fn lighting_precision_collapsed_filter_extent_refuses_before_scene_culling() {
    use crate::lighting::{LightingNumericPhase, LightingPreparationError};
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let mut original = Scene::new();
    original.fill(full, Color::BLACK);
    let stats = pollster::block_on(renderer.render(&mut surface, &original, Parameters::default()))
        .unwrap();
    let collapsed = Rect::try_new(1.0, 0.0, 2.0_f64.powi(-54), 1.0).unwrap();
    let scene = diffuse(
        Scene::new(),
        FilterDomains::try_new(full, collapsed, collapsed).unwrap(),
        DiffuseLightingInput::new(),
        VERTICAL,
        FilterColorSpace::Srgb,
        Layer::new(),
    );
    let failure = pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
        .unwrap_err();
    assert!(matches!(
        failure.lighting_diagnostic(),
        Some(LightingPreparationError::Numeric {
            phase: LightingNumericPhase::GridExtent,
            ..
        })
    ));
    assert_eq!(renderer.stats(), stats);
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_pixel(&pixels, 1, 0, [0, 0, 0, 255], 0);

    for touches in [Rect::new(0.0, 1.0, 3.0, 1.0), bounds(1, 1)] {
        let inactive = diffuse(
            Scene::new(),
            FilterDomains::try_new(full, collapsed, touches).unwrap(),
            DiffuseLightingInput::new(),
            VERTICAL,
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        pollster::block_on(renderer.render(&mut surface, &inactive, Parameters::default()))
            .unwrap();
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_pixel(&pixels, 1, 0, [0; 4], 0);
    }
}

#[test]
fn bounded_lighting_copy_preserves_exterior_support_for_a_later_backdrop() {
    use crate::{
        layer::{BlendMode, LayerComposition, PorterDuffOperator},
        style::filter::{
            BackdropCaptureBounds, BackdropFilterInput, FilterAmount, FilterList, FilterOp,
        },
    };
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let white = Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap();
    let mut scene = Scene::new();
    scene.fill(full, white);
    let copy =
        LayerComposition::try_porter_duff(PorterDuffOperator::Copy, BlendMode::Normal).unwrap();
    let lighting = prepare_lighting(
        DiffuseLightingInput::new().with_diffuse_constant(0.0),
        VERTICAL,
        FilterColorSpace::Srgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    scene.lighting_layer(
        Layer::new().with_composition(copy),
        lighting,
        LightingSource::finite(
            Scene::new(),
            FilterDomains::try_new(full, full, bounds(1, 1)).unwrap(),
        ),
    );
    let backdrop = BackdropFilterInput::try_new(
        FilterList::try_ops(vec![FilterOp::brightness(
            FilterAmount::try_new(0.0).unwrap(),
        )])
        .unwrap(),
        BackdropCaptureBounds::try_new(Rect::new(2.0, 0.0, 1.0, 1.0)).unwrap(),
        None,
    )
    .unwrap();
    scene.layer(Layer::new().try_backdrop_filter(backdrop).unwrap(), |_| {});
    let pixels = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&pixels, 0, 0, [0, 0, 0, 255], 0);
    assert_pixel(&pixels, 1, 0, [255; 4], 0);
    assert_pixel(&pixels, 2, 0, [0, 0, 0, 255], 0);
}

#[test]
fn same_layer_backdrop_supplies_bump_alpha_only_inside_the_source_domain() {
    use crate::style::filter::{
        BackdropCaptureBounds, BackdropFilterInput, FilterBlur, FilterList, FilterOp,
    };
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let unit = bounds(1, 1);
    let backdrop = BackdropFilterInput::try_new(
        FilterList::try_ops(vec![FilterOp::blur(FilterBlur::try_new(0.0).unwrap())]).unwrap(),
        BackdropCaptureBounds::try_new(full).unwrap(),
        None,
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.fill(full, Color::BLACK);
    let operation = prepare_lighting(
        DiffuseLightingInput::new(),
        VERTICAL,
        FilterColorSpace::Srgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    scene.lighting_layer(
        Layer::new().try_backdrop_filter(backdrop).unwrap(),
        operation,
        LightingSource::finite(
            Scene::new(),
            FilterDomains::try_new(unit, full, full).unwrap(),
        ),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // Captured alpha reaches the stencil as [1,0,0]. The first derivative
    // is -2, the interior one is -1, and the last one is zero.
    for (x, value) in (0..).zip([114, 180, 255]) {
        assert_pixel(&pixels, x, 0, [value, value, value, 255], 2);
    }
}

#[test]
fn lighting_work_cap_refusal_preserves_the_published_frame_and_stats() {
    let work = RenderWork::ScheduledLightingEvaluations;
    for cap in [1, 2] {
        let options = RenderAccountingOptions::new()
            .with_limits(RenderLimits::new().with_work_limit(work, cap).unwrap())
            .with_profile(RenderProfile::Summary { timing: true });
        let mut renderer =
            pollster::block_on(Renderer::new(Options::default().with_accounting(options))).unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(2.0, 1.0), 1.0)).unwrap();
        let full = bounds(2, 1);
        let mut original = Scene::new();
        original.fill(full, Color::BLACK);
        let stats =
            pollster::block_on(renderer.render(&mut surface, &original, Parameters::default()))
                .unwrap();
        let scene = diffuse(
            Scene::new(),
            FilterDomains::try_new(full, full, full).unwrap(),
            DiffuseLightingInput::new(),
            VERTICAL,
            FilterColorSpace::Srgb,
            Layer::new(),
        );
        let outcome =
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()));
        let report = renderer.last_attempt().unwrap();
        let attempt = report.attempt().unwrap();
        for phase in [
            RenderPhase::Plan,
            RenderPhase::GpuPrepare,
            RenderPhase::Encode,
        ] {
            assert!(attempt.phase_elapsed(phase).is_some(), "{phase:?}");
        }
        let expected = if cap == 1 {
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(
                attempt.refusal().unwrap().quantity(),
                RenderQuantity::Work(work)
            );
            assert_eq!(report.cleanup_fault(), None);
            assert!(outcome.unwrap_err().accounting_diagnostic().is_some());
            assert_eq!(renderer.stats(), stats);
            [0, 0, 0, 255]
        } else {
            outcome.unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(attempt.usage().work(work), 2);
            [255; 4]
        };
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for x in 0..2 {
            assert_pixel(&pixels, x, 0, expected, 1);
        }
    }
}

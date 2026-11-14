//! Public composition oracles keep bounded operation coverage independent of
//! filtered alpha, outer attenuation and authored clip coverage.
use super::convolution::{assert_pixel, bounds, image, pixel, renderer_and_surface};
use super::native_shadow::{operation, render};
use crate::{
    BlendMode, Color, DropShadowInput, DropShadowSource, FilterColorSpace, FilterDomains,
    ImageBuffer, Layer, LayerComposition, Parameters, PorterDuffOperator as Pd, Rect,
    ResolvedLayerAlphaMask, Scene, Shape, Transform, resource::WorkingFormat,
};

fn zero_blur() -> DropShadowInput {
    DropShadowInput::new()
        .with_dx(0.0)
        .with_dy(0.0)
        .with_standard_deviation(0.0)
        .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap())
}

fn append_constant(scene: &mut Scene, layer: Layer, filter: Rect, output: Rect) {
    scene
        .drop_shadow_layer(
            layer,
            operation(zero_blur(), FilterColorSpace::Srgb),
            DropShadowSource::constant(
                Color::try_rgba(0.0, 0.0, 1.0, 0.5).unwrap(),
                filter,
                output,
            )
            .unwrap(),
        )
        .unwrap();
}

fn assert_premul(output: &ImageBuffer, x: u32, y: u32, expected: [f64; 4], tolerance: f64) {
    let actual = pixel(output, x, y);
    let alpha = f64::from(actual[3]) / 255.0;
    assert!(
        (alpha - expected[3]).abs() * 255.0 <= tolerance,
        "alpha at ({x},{y}): {actual:?}, expected premul {expected:?}"
    );
    for channel in 0..3 {
        let value = f64::from(actual[channel]) / 255.0 * alpha;
        assert!(
            (value - expected[channel]).abs() * 255.0 <= tolerance,
            "premul channel {channel} at ({x},{y}): {actual:?}, expected {expected:?}"
        );
    }
}

#[test]
fn public_native_shadow_full_outer_affines_and_large_origin_preserve_local_merge() {
    let (mut renderer, mut surface) = renderer_and_surface(14, 12);
    let domain = bounds(6, 6);
    for (affine, inside, outside) in [
        ([-1.0, 0.0, 0.0, 1.0, 10.0, 0.0], (7, 3), (2, 2)),
        ([0.0, 1.0, -1.0, 0.0, 10.0, 0.0], (6, 3), (2, 2)),
        ([1.0, 0.5, 0.5, 1.0, 1.0, 1.0], (5, 5), (0, 0)),
        ([2.0, 0.0, 0.0, 0.5, 1.0, 1.0], (6, 2), (0, 0)),
    ] {
        let mut scene = Scene::new();
        append_constant(
            &mut scene,
            Layer::new().with_transform(Transform::try_new(affine).unwrap()),
            domain,
            domain,
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        // I blue,a=.5; red shadow h=.25; M=(.125,0,.5,.625).
        assert_premul(&pixels, inside.0, inside.1, [0.125, 0.0, 0.5, 0.625], 2.0);
        assert_pixel(&pixels, outside.0, outside.1, [0; 4], 1);
    }
    // An exactly representable large origin cancels on the host. The shader
    // still receives only the small local map, including both skew terms.
    let origin = 100_000_000.0;
    let large = Rect::new(origin, origin, 6.0, 6.0);
    let mut scene = Scene::new();
    append_constant(
        &mut scene,
        Layer::new().with_transform(
            Transform::try_new([1.0, 0.5, 0.5, 1.0, 1.0 - 1.5 * origin, 1.0 - 1.5 * origin])
                .unwrap(),
        ),
        large,
        large,
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    assert_premul(&pixels, 5, 5, [0.125, 0.0, 0.5, 0.625], 2.0);
    assert_pixel(&pixels, 0, 0, [0; 4], 1);
}

#[test]
fn public_native_shadow_author_clip_mask_and_opacity_attenuate_completed_merge() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
    let mask = ResolvedLayerAlphaMask::try_new(
        image(
            6,
            1,
            [255_u8, 255, 128, 0, 255, 255]
                .into_iter()
                .flat_map(|a| [255, 255, 255, a])
                .collect(),
        ),
        full,
    )
    .unwrap();
    let layer = Layer::new()
        .try_clip(Shape::rect(Rect::new(2.0, 0.0, 2.0, 1.0)))
        .unwrap()
        .with_resolved_alpha_mask(mask)
        .try_opacity(0.5)
        .unwrap();
    let mut scene = Scene::new();
    append_constant(&mut scene, layer, full, full);
    let output = render(&mut renderer, &mut surface, &scene);
    let attenuation = 0.5 * 128.0 / 255.0;
    assert_premul(
        &output,
        2,
        0,
        [
            0.125 * attenuation,
            0.0,
            0.5 * attenuation,
            0.625 * attenuation,
        ],
        2.0,
    );
    for x in [0, 1, 3, 4, 5] {
        assert_pixel(&output, x, 0, [0; 4], 1);
    }
}

#[test]
fn public_native_shadow_fractional_output_bounds_all_six_destructive_operations() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    let parent = [0.0, 0.3, 0.0, 0.75];
    for format in [
        WorkingFormat::HighPrecision,
        WorkingFormat::ReducedPrecision,
    ] {
        for operator in [
            Pd::Clear,
            Pd::Copy,
            Pd::SourceIn,
            Pd::SourceOut,
            Pd::DestinationIn,
            Pd::DestinationAtop,
            Pd::SourceOver,
        ] {
            let mask = ResolvedLayerAlphaMask::try_new(
                image(
                    4,
                    1,
                    [255_u8, 128, 64, 255]
                        .into_iter()
                        .flat_map(|a| [255, 255, 255, a])
                        .collect(),
                ),
                full,
            )
            .unwrap();
            let layer = Layer::new()
                .with_composition(
                    LayerComposition::try_porter_duff(operator, BlendMode::Normal).unwrap(),
                )
                .try_opacity(0.5)
                .unwrap()
                .with_resolved_alpha_mask(mask)
                .try_clip(Shape::rect(Rect::new(0.0, 0.0, 2.25, 1.0)))
                .unwrap();
            let mut scene = Scene::new();
            scene.fill(full, Color::try_rgba(0.0, 0.4, 0.0, 0.75).unwrap());
            append_constant(&mut scene, layer, full, output);
            pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                &mut surface,
                &scene,
                Parameters::default(),
                format,
            ))
            .unwrap();
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for x in 0..4 {
                let c = [0.5, 1.0, 0.5, 0.0][x as usize];
                let mask = [1.0, 128.0 / 255.0, 64.0 / 255.0, 1.0][x as usize];
                // Complete native merge before any O/outer attenuation:
                // I=(0,0,.5,.5), H=(.25,0,0,.25), M=(.125,0,.5,.625).
                let attenuation = 0.5 * mask;
                let source = [
                    0.125 * attenuation,
                    0.0,
                    0.5 * attenuation,
                    0.625 * attenuation,
                ];
                let (fs, fd) = match operator {
                    Pd::Clear => (0.0, 0.0),
                    Pd::Copy => (1.0, 0.0),
                    Pd::SourceIn => (parent[3], 0.0),
                    Pd::SourceOut => (1.0 - parent[3], 0.0),
                    Pd::DestinationIn => (0.0, source[3]),
                    Pd::DestinationAtop => (1.0 - parent[3], source[3]),
                    Pd::SourceOver => (1.0, 1.0 - source[3]),
                    _ => unreachable!(),
                };
                let author = [1.0, 1.0, 0.25, 0.0][x as usize];
                // Evaluate the uncovered full operation independently, mix
                // by geometric O membership, then mix by the authored clip.
                let complete = std::array::from_fn::<_, 4, _>(|i| fs * source[i] + fd * parent[i]);
                let expected = std::array::from_fn(|i| {
                    author * (c * complete[i] + (1.0 - c) * parent[i]) + (1.0 - author) * parent[i]
                });
                // Intermediate working/readback quantization is bounded in
                // premultiplied channels and alpha, including low-alpha Out.
                assert_premul(&pixels, x, 0, expected, 2.5);
            }
        }
    }
}

#[test]
fn public_native_shadow_plus_overrides_copy_without_destructive_coverage_correction() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    for format in [
        WorkingFormat::HighPrecision,
        WorkingFormat::ReducedPrecision,
    ] {
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(0.0, 0.25, 0.0, 1.0).unwrap());
        let layer = Layer::new()
            .with_composition(
                LayerComposition::try_porter_duff(Pd::Copy, BlendMode::Normal).unwrap(),
            )
            .blend(BlendMode::Plus)
            .try_opacity(0.5)
            .unwrap();
        append_constant(&mut scene, layer, full, output);
        pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            format,
        ))
        .unwrap();
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for (x, c) in [0.5, 1.0, 0.5, 0.0].into_iter().enumerate() {
            // Plus adds the already O-covered and opacity-attenuated merge.
            assert_premul(&pixels, x as u32, 0, [0.0625 * c, 0.25, 0.25 * c, 1.0], 2.0);
        }
    }
}

#[test]
fn public_native_shadow_empty_fractional_copy_preserves_exterior_and_singular_copy_is_empty() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    for format in [
        WorkingFormat::HighPrecision,
        WorkingFormat::ReducedPrecision,
    ] {
        for singular in [false, true] {
            let mut layer = Layer::new().with_composition(
                LayerComposition::try_porter_duff(Pd::Copy, BlendMode::Normal).unwrap(),
            );
            if singular {
                layer = layer
                    .with_transform(Transform::try_new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0]).unwrap());
            }
            let mut scene = Scene::new();
            scene.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
            scene
                .drop_shadow_layer(
                    layer,
                    operation(zero_blur(), FilterColorSpace::Srgb),
                    DropShadowSource::finite(
                        Scene::new(),
                        FilterDomains::try_new(full, full, output).unwrap(),
                    ),
                )
                .unwrap();
            pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                &mut surface,
                &scene,
                Parameters::default(),
                format,
            ))
            .unwrap();
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for x in 0..4 {
                let alpha = if singular {
                    1.0
                } else {
                    [0.5, 0.0, 0.5, 1.0][x as usize]
                };
                assert_premul(&pixels, x, 0, [0.0, alpha, 0.0, alpha], 2.0);
            }
        }
    }
}

#[test]
fn public_native_shadow_fractional_copy_reconstructs_geometric_coverage_through_full_affine() {
    let (mut renderer, mut surface) = renderer_and_surface(10, 7);
    let full = bounds(10, 7);
    let result = Rect::new(0.25, 0.25, 3.0, 2.0);
    let affine = [1.0, 0.25, 0.5, 1.0, 2.0, 1.0];
    // Density is the maximum singular value of the full 2x2 outer matrix.
    let determinant: f64 = 0.875;
    let trace: f64 = 2.3125;
    let rho = ((trace + (trace * trace - 4.0 * determinant * determinant).sqrt()) / 2.0).sqrt();
    let extents = [(3.25 * rho).ceil() as i32, (2.25 * rho).ceil() as i32];
    let cell = |x: i32, y: i32| {
        let x = f64::from(x.clamp(0, extents[0] - 1));
        let y = f64::from(y.clamp(0, extents[1] - 1));
        ((x + 1.0).min(3.25 * rho) - x.max(0.25 * rho)).clamp(0.0, 1.0)
            * ((y + 1.0).min(2.25 * rho) - y.max(0.25 * rho)).clamp(0.0, 1.0)
    };
    let mut scene = Scene::new();
    scene.fill(full, Color::try_rgba(0.0, 0.5, 0.0, 1.0).unwrap());
    append_constant(
        &mut scene,
        Layer::new()
            .with_transform(Transform::try_new(affine).unwrap())
            .with_composition(
                LayerComposition::try_porter_duff(Pd::Copy, BlendMode::Normal).unwrap(),
            ),
        Rect::new(-4.0, -4.0, 12.0, 12.0),
        result,
    );
    for format in [
        WorkingFormat::HighPrecision,
        WorkingFormat::ReducedPrecision,
    ] {
        pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            format,
        ))
        .unwrap();
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for y in 0..7 {
            for x in 0..10 {
                let px = f64::from(x) + 0.5 - 2.0;
                let py = f64::from(y) + 0.5 - 1.0;
                let texel = [
                    (px - 0.5 * py) / determinant * rho,
                    (-0.25 * px + py) / determinant * rho,
                ];
                let c = if texel[0] < 0.0
                    || texel[1] < 0.0
                    || texel[0] >= f64::from(extents[0])
                    || texel[1] >= f64::from(extents[1])
                {
                    0.0
                } else {
                    let q = [texel[0] - 0.5, texel[1] - 0.5];
                    let base = [q[0].floor() as i32, q[1].floor() as i32];
                    let fraction = [q[0] - q[0].floor(), q[1] - q[1].floor()];
                    let top = cell(base[0], base[1]) * (1.0 - fraction[0])
                        + cell(base[0] + 1, base[1]) * fraction[0];
                    let bottom = cell(base[0], base[1] + 1) * (1.0 - fraction[0])
                        + cell(base[0] + 1, base[1] + 1) * fraction[0];
                    top * (1.0 - fraction[1]) + bottom * fraction[1]
                };
                // Copy applies the completed native merge, then restores the
                // uncovered parent. c comes solely from O and the full affine.
                assert_premul(
                    &pixels,
                    x,
                    y,
                    [0.125 * c, 0.5 * (1.0 - c), 0.5 * c, 0.625 * c + (1.0 - c)],
                    2.5,
                );
            }
        }
    }
}

#[test]
fn public_native_shadow_unrepresentable_existing_capture_lattice_refuses_before_runtime() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 4);
    let origin = 1_099_511_627_776.0;
    let large = Rect::new(origin, origin, 6.0, 6.0);
    let mut scene = Scene::new();
    append_constant(
        &mut scene,
        Layer::new().with_transform(Transform::translation(-origin, -origin).unwrap()),
        large,
        large,
    );
    let error = pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
        .unwrap_err();
    // Local filters use the existing signed i32 device-origin lattice before
    // rebasing shader coordinates. 2^40 cannot fit even though final A cancels.
    assert!(matches!(
        error.gaussian_diagnostic(),
        Some(crate::GaussianPreparationError::Numeric {
            phase: crate::GaussianNumericPhase::GridMapping,
            ..
        })
    ));
}

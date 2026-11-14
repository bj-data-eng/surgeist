//! Public GPU witnesses for authentic displacement sources and bounded outer
//! composition. Expectations below are literal geometry or premultiplied
//! Porter-Duff equations, independent of production grid/table/shader helpers.
use super::convolution::{assert_pixel, bounds, image, renderer_and_surface};
use super::displacement::{
    FORMATS, active, assert_premul, finite_scene, native_map, render, source_scene,
};
use crate::{
    BackdropCaptureBounds, BackdropFilterInput, BlendMode, Color, DisplacementChannel as Channel,
    DisplacementPreparationError, DisplacementSource, FilterBlur, FilterDomains, FilterList,
    FilterOp, ImageFit, Layer, LayerComposition, PorterDuffOperator as Pd, PreparedDisplacementMap,
    Rect, ResolvedLayerAlphaMask, Scene, Shape, Transform,
};

fn white(alpha: u8) -> [u8; 4] {
    if alpha == 0 {
        [0; 4]
    } else {
        [255, 255, 255, alpha]
    }
}

#[test]
fn public_displacement_constant_and_periodic_inputs_are_clipped_to_fractional_filter_before_lookup()
{
    let (mut renderer, mut surface) = renderer_and_surface(8, 2);
    let filter = Rect::new(1.5, 0.0, 4.0, 1.0);
    let result = bounds(8, 1);
    for format in FORMATS {
        for periodic in [false, true] {
            let source = if periodic {
                let period = Rect::new(-1.0, -2.0, 2.0, 3.0);
                let mut tile = Scene::new();
                tile.fill(period, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
                DisplacementSource::periodic(tile, period, filter, result).unwrap()
            } else {
                DisplacementSource::constant(
                    Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
                    filter,
                    result,
                )
                .unwrap()
            };
            let mut scene = Scene::new();
            scene
                .displacement_layer(
                    Layer::new(),
                    active(4.0, Channel::R, Channel::G),
                    source,
                    native_map(1, 1, vec![255, 128, 0, 255], result),
                )
                .unwrap();
            let pixels = render(&mut renderer, &mut surface, &scene, format);
            // O=F. x1 has O coverage.5 and reads full source cell3.
            // x3 reads source cell5 with D coverage.5. x4 reads outside F.
            assert_premul(&pixels, 1, 0, [0.5, 0.0, 0.0, 0.5], 1.5);
            assert_pixel(&pixels, 2, 0, [255, 0, 0, 255], 1);
            assert_premul(&pixels, 3, 0, [0.5, 0.0, 0.0, 0.5], 1.5);
            for x in [0, 4, 5, 6, 7] {
                assert_pixel(&pixels, x, 0, [0; 4], 1);
            }
        }
    }
}

#[test]
fn public_displacement_non_square_period_seams_and_finite_sibling_preserve_their_own_context() {
    let (mut renderer, mut surface) = renderer_and_surface(12, 6);
    let period = Rect::new(-1.0, -2.0, 2.0, 3.0);
    let filter = Rect::new(0.0, 0.0, 7.0, 5.0);
    let colors = [
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 0, 255],
        [255, 0, 255, 255],
        [0, 255, 255, 255],
    ];
    for format in FORMATS {
        let mut scene = Scene::new();
        scene
            .displacement_layer(
                Layer::new(),
                active(4.0, Channel::R, Channel::G),
                DisplacementSource::periodic(
                    source_scene(image(2, 3, colors.concat()), period),
                    period,
                    filter,
                    filter,
                )
                .unwrap(),
                native_map(1, 1, vec![0, 128, 0, 255], filter),
            )
            .unwrap();
        let sibling = Rect::new(9.0, 0.0, 2.0, 1.0);
        scene
            .displacement_layer(
                Layer::new(),
                active(4.0, Channel::R, Channel::G),
                DisplacementSource::finite(
                    source_scene(image(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255]), sibling),
                    FilterDomains::try_new(sibling, sibling, sibling).unwrap(),
                ),
                native_map(1, 1, vec![128, 128, 0, 255], sibling),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene, format);
        for y in 0..5 {
            assert_pixel(&pixels, 0, y, [0; 4], 0); // x-1 is outside F.
            for x in 1..7 {
                // Displaced source x'=x-1. Tile origin(-1,-2) gives
                // column x mod2 and row(y+2) mod3, including both seams.
                let index = (((y + 2) % 3) * 2 + x % 2) as usize;
                assert_pixel(&pixels, x, y, colors[index], 1);
            }
        }
        assert_pixel(&pixels, 9, 0, [255, 0, 0, 255], 1);
        assert_pixel(&pixels, 10, 0, [0, 255, 0, 255], 1);
        assert_pixel(&pixels, 8, 0, [0; 4], 0);
    }
}

#[test]
fn public_displacement_fractional_period_seam_reconstructs_tile_before_integer_copy() {
    let (mut renderer, mut surface) = renderer_and_surface(12, 2);
    let period = Rect::new(0.0, 0.0, 2.5, 1.25);
    let full = bounds(8, 1);
    for format in FORMATS {
        // The authentic tile is captured as3x2. Its three columns are white
        // alpha51,153,255; both rows are equal. Tile realization at x=.5 gives
        // q=.5*(3/2.5)-.5=.1, hence .9*51+.1*153=61.2.
        // At x=2.5 q=2.5, seam reconstruction mixes columns2 and0 equally.
        let tile = source_scene(
            image(
                3,
                2,
                [51_u8, 153, 255, 51, 153, 255]
                    .into_iter()
                    .flat_map(white)
                    .collect(),
            ),
            period,
        );
        let mut scene = Scene::new();
        scene
            .displacement_layer(
                Layer::new(),
                active(4.0, Channel::R, Channel::G),
                DisplacementSource::periodic(tile, period, full, full).unwrap(),
                native_map(1, 1, vec![128, 128, 0, 255], full),
            )
            .unwrap();
        let sibling = Rect::new(9.0, 0.0, 2.0, 1.0);
        scene
            .displacement_layer(
                Layer::new(),
                active(4.0, Channel::R, Channel::G),
                DisplacementSource::finite(
                    source_scene(image(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255]), sibling),
                    FilterDomains::try_new(sibling, sibling, sibling).unwrap(),
                ),
                native_map(1, 1, vec![0, 128, 0, 255], sibling),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene, format);
        for (x, a) in [
            (0, 61.2 / 255.0),
            (2, 153.0 / 255.0),
            (5, 61.2 / 255.0),
            (7, 153.0 / 255.0),
        ] {
            assert_premul(&pixels, x, 0, [a; 4], 2.0);
        }
        // The finite sibling restores rho1 after tile densities6/5 and8/5:
        // R0 at scale4 moves one root cell, so x10 reads the red source x9.
        assert_pixel(&pixels, 9, 0, [0; 4], 0);
        assert_pixel(&pixels, 10, 0, [255, 0, 0, 255], 1);
    }
}

fn nested_vectors(origin: f64) -> Scene {
    let mut source = Scene::new();
    source.layer(
        Layer::new().with_transform(Transform::translation(origin, origin).unwrap()),
        |child| {
            child.fill(bounds(6, 6), Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
            child.layer(Layer::new(), |nested| {
                nested.image(
                    image(1, 1, vec![0, 0, 255, 128]),
                    bounds(6, 6),
                    ImageFit::Stretch,
                );
            });
        },
    );
    source
}

#[test]
fn public_displacement_full_affines_and_large_rebased_origins_preserve_nested_source_order() {
    let (mut renderer, mut surface) = renderer_and_surface(14, 12);
    let domain = bounds(6, 6);
    for format in FORMATS {
        for (affine, inside, outside) in [
            ([-1.0, 0.0, 0.0, 1.0, 10.0, 0.0], (7, 3), (2, 2)),
            ([0.0, 1.0, -1.0, 0.0, 10.0, 0.0], (6, 3), (2, 2)),
            ([1.0, 0.5, 0.5, 1.0, 1.0, 1.0], (5, 5), (0, 0)),
            ([2.0, 0.0, 0.0, 0.5, 1.0, 1.0], (6, 2), (0, 0)),
        ] {
            let scene = finite_scene(
                nested_vectors(0.0),
                FilterDomains::try_new(domain, domain, domain).unwrap(),
                active(4.0, Channel::R, Channel::G),
                native_map(1, 1, vec![128, 128, 0, 255], domain),
                Layer::new().with_transform(Transform::try_new(affine).unwrap()),
            );
            let pixels = render(&mut renderer, &mut surface, &scene, format);
            // Opaque red, then blue alpha128/255: premul(127,0,128)/255.
            assert_premul(
                &pixels,
                inside.0,
                inside.1,
                [127.0 / 255.0, 0.0, 128.0 / 255.0, 1.0],
                2.0,
            );
            assert_pixel(&pixels, outside.0, outside.1, [0; 4], 1);
        }
        let origin = 100_000_000.0;
        let large = Rect::new(origin, origin, 6.0, 6.0);
        let scene = finite_scene(
            nested_vectors(origin),
            FilterDomains::try_new(large, large, large).unwrap(),
            active(4.0, Channel::R, Channel::G),
            native_map(1, 1, vec![128, 128, 0, 255], large),
            Layer::new().with_transform(
                Transform::try_new([1.0, 0.5, 0.5, 1.0, 1.0 - 1.5 * origin, 1.0 - 1.5 * origin])
                    .unwrap(),
            ),
        );
        let pixels = render(&mut renderer, &mut surface, &scene, format);
        assert_premul(&pixels, 5, 5, [127.0 / 255.0, 0.0, 128.0 / 255.0, 1.0], 2.0);
        assert_pixel(&pixels, 0, 0, [0; 4], 1);
    }
}

#[test]
fn public_displacement_nested_active_effects_sample_each_completed_source() {
    let (mut renderer, mut surface) = renderer_and_surface(8, 1);
    let full = bounds(8, 1);
    for format in FORMATS {
        let bytes = [
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255, 255, 0, 255, 255, 0,
            255, 255, 255, 80, 80, 80, 255, 160, 160, 160, 255,
        ];
        let inner = finite_scene(
            source_scene(image(8, 1, bytes.to_vec()), full),
            FilterDomains::try_new(full, full, full).unwrap(),
            active(4.0, Channel::R, Channel::G),
            native_map(1, 1, vec![255, 128, 0, 255], full),
            Layer::new(),
        );
        let scene = finite_scene(
            inner,
            FilterDomains::try_new(full, full, full).unwrap(),
            active(4.0, Channel::R, Channel::G),
            native_map(1, 1, vec![0, 128, 0, 255], full),
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene, format);
        assert_pixel(&pixels, 0, 0, [0; 4], 0); // outer source address -1.
        for x in 1..7 {
            let at = ((x + 1) * 4) as usize;
            assert_pixel(&pixels, x, 0, bytes[at..at + 4].try_into().unwrap(), 1);
        }
        assert_pixel(&pixels, 7, 0, [0; 4], 0); // inner address8 is transparent.
    }
}

#[test]
fn public_displacement_finite_same_layer_backdrop_completes_once_before_displaced_lookup() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
    let source_domain = Rect::new(1.0, 0.0, 2.0, 1.0);
    let result = Rect::new(2.0, 0.0, 2.0, 1.0);
    for format in FORMATS {
        let backdrop = BackdropFilterInput::try_new(
            FilterList::try_ops(vec![FilterOp::blur(FilterBlur::try_new(0.0).unwrap())]).unwrap(),
            BackdropCaptureBounds::try_new(full).unwrap(),
            None,
        )
        .unwrap();
        let mut scene = Scene::new();
        scene.fill(
            Rect::new(1.0, 0.0, 1.0, 1.0),
            Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
        );
        scene
            .displacement_layer(
                Layer::new().try_backdrop_filter(backdrop).unwrap(),
                active(4.0, Channel::R, Channel::G),
                DisplacementSource::finite(
                    source_scene(
                        image(1, 1, vec![0, 0, 255, 128]),
                        Rect::new(2.0, 0.0, 1.0, 1.0),
                    ),
                    FilterDomains::try_new(source_domain, full, result).unwrap(),
                ),
                native_map(1, 1, vec![0, 128, 0, 255], full),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene, format);
        assert_pixel(&pixels, 1, 0, [255, 0, 0, 255], 1);
        assert_pixel(&pixels, 2, 0, [255, 0, 0, 255], 1);
        assert_premul(&pixels, 3, 0, [0.0, 0.0, 128.0 / 255.0, 128.0 / 255.0], 1.5);
        assert_pixel(&pixels, 4, 0, [0; 4], 0);
    }
}

#[test]
fn public_displacement_infinite_backdrop_rejection_preserves_existing_receiver_scene() {
    let full = bounds(4, 1);
    let backdrop = Layer::new()
        .try_backdrop_filter(
            BackdropFilterInput::try_new(
                FilterList::try_ops(vec![FilterOp::blur(FilterBlur::try_new(0.0).unwrap())])
                    .unwrap(),
                BackdropCaptureBounds::try_new(full).unwrap(),
                None,
            )
            .unwrap(),
        )
        .unwrap();
    let mut scene = Scene::new();
    scene.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
    let previous = scene.clone();
    for source in [
        DisplacementSource::constant(Color::BLACK, full, full).unwrap(),
        DisplacementSource::periodic(
            source_scene(image(1, 1, vec![255; 4]), bounds(1, 1)),
            bounds(1, 1),
            full,
            full,
        )
        .unwrap(),
    ] {
        let error = scene
            .displacement_layer(
                backdrop.clone(),
                active(4.0, Channel::R, Channel::G),
                source,
                native_map(1, 1, vec![128, 128, 0, 255], full),
            )
            .unwrap_err();
        assert_eq!(
            error.displacement_diagnostic(),
            Some(&DisplacementPreparationError::InfiniteSourceBackdrop)
        );
        assert_eq!(scene, previous);
    }
}

#[test]
fn public_displacement_outer_clip_mask_and_opacity_apply_after_lookup() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
    for format in FORMATS {
        let mask = ResolvedLayerAlphaMask::try_new(
            image(
                6,
                1,
                [255_u8, 255, 128, 0, 255, 255]
                    .into_iter()
                    .flat_map(white)
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
        let source = source_scene(
            image(
                6,
                1,
                [0_u8, 255, 0, 0, 0, 0]
                    .into_iter()
                    .flat_map(|a| if a == 0 { [0; 4] } else { [255, 0, 0, a] })
                    .collect(),
            ),
            full,
        );
        let scene = finite_scene(
            source,
            FilterDomains::try_new(full, full, full).unwrap(),
            active(4.0, Channel::R, Channel::G),
            native_map(1, 1, vec![0, 128, 0, 255], full),
            layer,
        );
        let pixels = render(&mut renderer, &mut surface, &scene, format);
        let a = 0.5 * 128.0 / 255.0;
        assert_premul(&pixels, 2, 0, [a, 0.0, 0.0, a], 1.5);
        for x in [0, 1, 3, 4, 5] {
            assert_pixel(&pixels, x, 0, [0; 4], 1);
        }
    }
}

fn append_uniform(scene: &mut Scene, layer: Layer, full: Rect, output: Rect) {
    scene
        .displacement_layer(
            layer,
            active(4.0, Channel::R, Channel::G),
            DisplacementSource::finite(
                source_scene(image(4, 1, [255, 0, 128, 128].repeat(4)), full),
                FilterDomains::try_new(full, full, output).unwrap(),
            ),
            native_map(1, 1, vec![128, 128, 0, 255], full),
        )
        .unwrap();
}

#[test]
fn public_displacement_fractional_output_bounds_all_six_destructive_operations() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    let parent = [0.0, 0.3, 0.0, 0.75];
    for format in FORMATS {
        for operator in [
            Pd::Clear,
            Pd::Copy,
            Pd::SourceIn,
            Pd::SourceOut,
            Pd::DestinationIn,
            Pd::DestinationAtop,
        ] {
            let mask = ResolvedLayerAlphaMask::try_new(
                image(
                    4,
                    1,
                    [255_u8, 128, 64, 255].into_iter().flat_map(white).collect(),
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
            append_uniform(&mut scene, layer, full, output);
            let pixels = render(&mut renderer, &mut surface, &scene, format);
            for x in 0..4 {
                let coverage = [0.5, 1.0, 0.5, 0.0][x as usize];
                let mask = [1.0, 128.0 / 255.0, 64.0 / 255.0, 1.0][x as usize];
                let a = 128.0 / 255.0 * 0.5 * mask;
                let source = [a, 0.0, a * 128.0 / 255.0, a];
                let (fs, fd) = match operator {
                    Pd::Clear => (0.0, 0.0),
                    Pd::Copy => (1.0, 0.0),
                    Pd::SourceIn => (parent[3], 0.0),
                    Pd::SourceOut => (1.0 - parent[3], 0.0),
                    Pd::DestinationIn => (0.0, source[3]),
                    Pd::DestinationAtop => (1.0 - parent[3], source[3]),
                    _ => unreachable!(),
                };
                let author = [1.0, 1.0, 0.25, 0.0][x as usize];
                let expected = std::array::from_fn(|i| {
                    let composed = fs * source[i] + fd * parent[i];
                    author * (coverage * composed + (1.0 - coverage) * parent[i])
                        + (1.0 - author) * parent[i]
                });
                // One RGBA8 source/readback quantization and half-float working
                // rounding fit2.5 premul byte units, even for low-alpha Out.
                assert_premul(&pixels, x, 0, expected, 2.5);
            }
        }
    }
}

#[test]
fn public_displacement_plus_overrides_copy_and_preserves_fractional_bounded_exterior() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    for format in FORMATS {
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(0.0, 0.25, 0.0, 1.0).unwrap());
        append_uniform(
            &mut scene,
            Layer::new()
                .with_composition(
                    LayerComposition::try_porter_duff(Pd::Copy, BlendMode::Normal).unwrap(),
                )
                .blend(BlendMode::Plus)
                .try_opacity(0.5)
                .unwrap(),
            full,
            output,
        );
        let pixels = render(&mut renderer, &mut surface, &scene, format);
        for (x, c) in [0.5, 1.0, 0.5, 0.0].into_iter().enumerate() {
            let a = 128.0 / 255.0 * 0.5 * c;
            assert_premul(&pixels, x as u32, 0, [a, 0.25, a * 128.0 / 255.0, 1.0], 2.0);
        }
    }
}

#[test]
fn public_displacement_transparent_bounded_copy_changes_only_output_and_singular_transform_is_empty()
 {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    for format in FORMATS {
        for (singular, empty_source) in [(false, true), (false, false), (true, false)] {
            let mut layer = Layer::new().with_composition(
                LayerComposition::try_porter_duff(Pd::Copy, BlendMode::Normal).unwrap(),
            );
            if singular {
                layer = layer
                    .with_transform(Transform::try_new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0]).unwrap());
            }
            let mut scene = Scene::new();
            scene.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
            let source = if empty_source {
                Scene::new()
            } else {
                source_scene(image(4, 1, [255, 0, 0, 255].repeat(4)), full)
            };
            let map = if empty_source {
                PreparedDisplacementMap::source_graphic()
            } else {
                // Active raw-map execution moves Y by+2, beyond this one-row
                // source. Transparent samples still participate in bounded Copy.
                native_map(1, 1, vec![255; 4], full)
            };
            scene
                .displacement_layer(
                    layer,
                    active(4.0, Channel::R, Channel::G),
                    DisplacementSource::finite(
                        source,
                        FilterDomains::try_new(full, full, output).unwrap(),
                    ),
                    map,
                )
                .unwrap();
            let pixels = render(&mut renderer, &mut surface, &scene, format);
            for x in 0..4 {
                let a = if singular {
                    1.0
                } else {
                    [0.5, 0.0, 0.5, 1.0][x as usize]
                };
                assert_premul(&pixels, x, 0, [0.0, a, 0.0, a], 1.5);
            }
        }
    }
}

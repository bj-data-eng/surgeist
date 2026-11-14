//! Independent public morphology source, placement and bounded-composition witnesses.
use super::convolution::{assert_pixel, bounds, image, renderer_and_surface};
use super::morphology::{assert_premul, finite_scene, operation, render, source_scene};
use crate::{
    BackdropCaptureBounds, BackdropFilterInput, BlendMode, Color, FilterBlur, FilterColorSpace,
    FilterDomains, FilterList, FilterOp, ImageFit, Layer, LayerComposition, MorphologyInput,
    MorphologyOperator as Op, MorphologySource, Parameters, PorterDuffOperator as Pd, Rect,
    ResolvedLayerAlphaMask, Scene, Shape, Transform, UnitFilterAmount, resource::WorkingFormat,
};

fn active(op: Op, x: f64, y: f64) -> crate::PreparedMorphology {
    operation(
        MorphologyInput::new().with_operator(op).with_radii(x, y),
        FilterColorSpace::Srgb,
    )
}

fn white(a: u8) -> [u8; 4] {
    if a == 0 { [0; 4] } else { [255, 255, 255, a] }
}

#[test]
fn public_morphology_fractional_period_uses_actual_unequal_axis_densities_and_restores_context() {
    let (mut renderer, mut surface) = renderer_and_surface(20, 6);
    let period = Rect::new(0.0, 0.0, 2.5, 1.25);
    let output = Rect::new(-5.0, 0.0, 13.0, 1.25);
    for format in [
        WorkingFormat::HighPrecision,
        WorkingFormat::ReducedPrecision,
    ] {
        for op in [Op::Erode, Op::Dilate] {
            // Authentic canvas is 3x2, with densities 6/5 and 8/5.
            // Radius .7 selects Rx0,Ry1, not a shared scalar-density radius.
            let input = image(
                3,
                2,
                [26, 230, 77, 153, 51, 204]
                    .into_iter()
                    .flat_map(white)
                    .collect(),
            );
            let mut tile = Scene::new();
            tile.layer(Layer::new(), |child| {
                child.image(input, period, ImageFit::Stretch);
            });
            let mut scene = Scene::new();
            scene
                .morphology_layer(
                    Layer::new().with_transform(Transform::translation(5.0, 0.0).unwrap()),
                    active(op, 0.7, 0.7),
                    MorphologySource::periodic(tile, period, output, output).unwrap(),
                )
                .unwrap();
            // A finite sibling must use the restored unit-density root context.
            let sibling = Rect::new(15.0, 0.0, 2.0, 1.0);
            scene
                .morphology_layer(
                    Layer::new(),
                    active(Op::Dilate, 0.25, 0.25),
                    MorphologySource::finite(
                        source_scene(image(2, 1, [0, 255, 0, 255].repeat(2)), sibling),
                        FilterDomains::try_new(sibling, sibling, sibling).unwrap(),
                    ),
                )
                .unwrap();
            let second_period = Rect::new(0.0, 0.0, 1.25, 2.5);
            let second_output = Rect::new(0.0, 0.0, 2.5, 3.0);
            scene
                .morphology_layer(
                    Layer::new().with_transform(Transform::translation(15.0, 2.0).unwrap()),
                    active(op, 0.7, 0.7),
                    MorphologySource::periodic(
                        source_scene(
                            image(
                                2,
                                3,
                                [51, 204, 102, 153, 77, 230]
                                    .into_iter()
                                    .flat_map(white)
                                    .collect(),
                            ),
                            second_period,
                        ),
                        second_period,
                        second_output,
                        second_output,
                    )
                    .unwrap(),
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
            let cells = if op == Op::Erode {
                [26.0, 51.0, 77.0]
            } else {
                [153.0, 230.0, 204.0]
            };
            for x in 0..13 {
                let q = (f64::from(x) + 0.5 - 5.0) * 6.0 / 5.0 - 0.5;
                let base = q.floor() as i64;
                let t = q - q.floor();
                let a = (cells[base.rem_euclid(3) as usize] * (1.0 - t)
                    + cells[(base + 1).rem_euclid(3) as usize] * t)
                    / 255.0;
                for (y, coverage) in [(0, 1.0), (1, 0.25)] {
                    assert_premul(&pixels, x, y, [a * coverage; 4], 2.5);
                }
            }
            assert_pixel(&pixels, 15, 0, [0, 255, 0, 255], 1);
            assert_pixel(&pixels, 16, 0, [0, 255, 0, 255], 1);
            // Second canvas is 2x3: Rx1,Ry0. Its rows are their X extrema;
            // surface y2.5 maps to qy=.1, so rows0 and1 mix 9:1.
            let rows = if op == Op::Erode {
                [51.0, 102.0]
            } else {
                [204.0, 153.0]
            };
            let a = (0.9 * rows[0] + 0.1 * rows[1]) / 255.0;
            assert_premul(&pixels, 15, 2, [a; 4], 2.5);
            assert_pixel(&pixels, 19, 5, [0; 4], 1);
        }
    }
}

#[test]
fn public_morphology_short_wrapped_windows_and_one_cell_period_axes_match_literal_extrema() {
    let (mut renderer, mut surface) = renderer_and_surface(10, 3);
    let values = [26_u8, 230, 51, 179, 102];
    for format in [
        WorkingFormat::HighPrecision,
        WorkingFormat::ReducedPrecision,
    ] {
        for op in [Op::Erode, Op::Dilate] {
            for vertical in [false, true] {
                let period = if vertical { bounds(1, 5) } else { bounds(5, 1) };
                let out = bounds(10, 3);
                let (w, h) = if vertical { (1, 5) } else { (5, 1) };
                let mut scene = Scene::new();
                scene
                    .morphology_layer(
                        Layer::new(),
                        active(
                            op,
                            if vertical { 1e300 } else { 1.0 },
                            if vertical { 1.0 } else { 1e300 },
                        ),
                        MorphologySource::periodic(
                            source_scene(
                                image(w, h, values.into_iter().flat_map(white).collect()),
                                period,
                            ),
                            period,
                            out,
                            out,
                        )
                        .unwrap(),
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
                for y in 0..3 {
                    for x in 0..10 {
                        let p = if vertical { y } else { x };
                        let mut a = if op == Op::Erode { 255 } else { 0 };
                        for d in -1_i64..=1 {
                            let v = values[(i64::from(p) + d).rem_euclid(5) as usize];
                            a = if op == Op::Erode { a.min(v) } else { a.max(v) };
                        }
                        assert_pixel(&pixels, x, y, white(a), 2);
                    }
                }
            }
        }
    }
}

fn nested_vector_and_image(origin: f64) -> Scene {
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
fn public_morphology_finite_vector_and_image_keep_full_affines_and_large_rebased_origins() {
    let (mut renderer, mut surface) = renderer_and_surface(14, 12);
    let domain = bounds(6, 6);
    for (affine, inside, outside) in [
        ([-1.0, 0.0, 0.0, 1.0, 10.0, 0.0], (7, 3), (2, 2)),
        ([0.0, 1.0, -1.0, 0.0, 10.0, 0.0], (6, 3), (2, 2)),
        ([1.0, 0.5, 0.5, 1.0, 1.0, 1.0], (5, 5), (0, 0)),
        ([2.0, 0.0, 0.0, 0.5, 1.0, 1.0], (6, 2), (0, 0)),
    ] {
        let scene = finite_scene(
            nested_vector_and_image(0.0),
            FilterDomains::try_new(domain, domain, domain).unwrap(),
            active(Op::Dilate, 1.0, 1.0),
            Layer::new().with_transform(Transform::try_new(affine).unwrap()),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        // Opaque red under a half-alpha blue image is premul(127/255,0,128/255,1).
        assert_premul(
            &pixels,
            inside.0,
            inside.1,
            [127.0 / 255.0, 0.0, 128.0 / 255.0, 1.0],
            2.5,
        );
        assert_pixel(&pixels, outside.0, outside.1, [0; 4], 1);
    }
    let origin = 100_000_000.0;
    let large = Rect::new(origin, origin, 6.0, 6.0);
    let scene = finite_scene(
        nested_vector_and_image(origin),
        FilterDomains::try_new(large, large, large).unwrap(),
        active(Op::Dilate, 1.0, 1.0),
        Layer::new().with_transform(
            Transform::try_new([1.0, 0.5, 0.5, 1.0, 1.0 - 1.5 * origin, 1.0 - 1.5 * origin])
                .unwrap(),
        ),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    assert_premul(&pixels, 5, 5, [127.0 / 255.0, 0.0, 128.0 / 255.0, 1.0], 2.5);
    assert_pixel(&pixels, 0, 0, [0; 4], 1);
}

#[test]
fn public_morphology_finite_backdrop_and_backdrop_only_complete_before_extrema() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
    let domain = Rect::new(1.0, 0.0, 2.0, 1.0);
    for has_children in [false, true] {
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
        let mut source = Scene::new();
        if has_children {
            source.image(
                image(1, 1, vec![0, 0, 255, 128]),
                Rect::new(2.0, 0.0, 1.0, 1.0),
                ImageFit::Stretch,
            );
        }
        scene
            .morphology_layer(
                Layer::new().try_backdrop_filter(backdrop).unwrap(),
                active(Op::Dilate, 1.0, 0.25),
                MorphologySource::finite(
                    source,
                    FilterDomains::try_new(domain, full, full).unwrap(),
                ),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        let blue = if has_children { 128.0 / 255.0 } else { 0.0 };
        // D remains origin1, width2, regardless of the larger backdrop capture.
        assert_premul(&pixels, 2, 0, [1.0, 0.0, blue, 1.0], 2.5);
        assert_premul(&pixels, 3, 0, [0.0, 0.0, blue, blue], 2.5);
        assert_pixel(&pixels, 5, 0, [0; 4], 1);
    }
}

#[test]
fn public_morphology_empty_domains_omit_huge_unused_backdrop_and_radius_work() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let empty = Rect::new(0.0, 0.0, 0.0, 1e300);
    for (source, output) in [(empty, full), (full, empty)] {
        let backdrop = BackdropFilterInput::try_new(
            FilterList::try_ops(vec![FilterOp::invert(
                UnitFilterAmount::try_new(1.0).unwrap(),
            )])
            .unwrap(),
            BackdropCaptureBounds::try_new(Rect::new(0.0, 0.0, 1e12, 1e12)).unwrap(),
            None,
        )
        .unwrap();
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
        scene
            .morphology_layer(
                Layer::new().try_backdrop_filter(backdrop).unwrap(),
                active(Op::Dilate, 1e300, 1e300),
                MorphologySource::finite(
                    source_scene(image(3, 1, [255; 4].repeat(3)), full),
                    FilterDomains::try_new(source, full, output).unwrap(),
                ),
            )
            .unwrap();
        let stats =
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                .unwrap();
        assert_eq!(stats, renderer.stats());
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for x in 0..3 {
            assert_pixel(&pixels, x, 0, [0, 255, 0, 255], 1);
        }
        assert_eq!(
            renderer
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(crate::RenderWork::KernelSampleVisits),
            0
        );
    }
}

#[test]
fn public_morphology_outer_clip_mask_and_opacity_attenuate_after_active_extrema() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
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
    // Only x1 is red. It must reach x2 before the outer clip removes x1.
    let scene = finite_scene(
        source_scene(
            image(
                6,
                1,
                [0, 255, 0, 0, 0, 0]
                    .into_iter()
                    .flat_map(|a| if a == 0 { [0; 4] } else { [255, 0, 0, a] })
                    .collect(),
            ),
            full,
        ),
        FilterDomains::try_new(full, full, full).unwrap(),
        active(Op::Dilate, 1.0, 0.25),
        layer,
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    let a = 0.5 * 128.0 / 255.0;
    assert_premul(&pixels, 2, 0, [a, 0.0, 0.0, a], 2.0);
    for x in [0, 1, 3, 4, 5] {
        assert_pixel(&pixels, x, 0, [0; 4], 1);
    }
}

fn append_uniform_finite(scene: &mut Scene, layer: Layer, full: Rect, output: Rect) {
    scene
        .morphology_layer(
            layer,
            active(Op::Dilate, 1.0, 0.25),
            MorphologySource::finite(
                source_scene(image(4, 1, [255, 0, 128, 128].repeat(4)), full),
                FilterDomains::try_new(full, full, output).unwrap(),
            ),
        )
        .unwrap();
}

#[test]
fn public_morphology_fractional_output_bounds_all_six_destructive_operations_in_both_formats() {
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
            append_uniform_finite(&mut scene, layer, full, output);
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
                    let complete = fs * source[i] + fd * parent[i];
                    author * (c * complete + (1.0 - c) * parent[i]) + (1.0 - author) * parent[i]
                });
                // Bound alpha and premul color directly, including low-alpha Out.
                assert_premul(&pixels, x, 0, expected, 2.5);
            }
        }
    }
}

#[test]
fn public_morphology_plus_overrides_copy_without_destructive_coverage_correction() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    for format in [
        WorkingFormat::HighPrecision,
        WorkingFormat::ReducedPrecision,
    ] {
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(0.0, 0.25, 0.0, 1.0).unwrap());
        append_uniform_finite(
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
        pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            format,
        ))
        .unwrap();
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for (x, c) in [0.5, 1.0, 0.5, 0.0].into_iter().enumerate() {
            let a = 128.0 / 255.0 * 0.5 * c;
            assert_premul(&pixels, x as u32, 0, [a, 0.25, a * 128.0 / 255.0, 1.0], 2.0);
        }
    }
}

#[test]
fn public_morphology_empty_fractional_copy_preserves_exterior_and_singular_copy_is_empty() {
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
                .morphology_layer(
                    layer,
                    active(Op::Dilate, 1.0, 0.25),
                    MorphologySource::finite(
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
                let a = if singular {
                    1.0
                } else {
                    [0.5, 0.0, 0.5, 1.0][x as usize]
                };
                assert_premul(&pixels, x, 0, [0.0, a, 0.0, a], 2.0);
            }
        }
    }
}

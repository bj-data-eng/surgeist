use super::*;
use crate::tests::{
    icc_png,
    support::{default_graph_working_format_for_test, pixel_rgba},
};
use crate::{renderer::Options, renderer::Renderer, stats::RenderRoute, surface::Parameters};

struct ColorCase {
    chromaticities: Option<[u32; 8]>,
    source: [u8; 3],
    expected: [u8; 3],
}

// Independent standard-math expectations: power-one source samples, exact
// rational primary calibration/Bradford adaptation and high-precision sRGB
// transfer/rounding. No production converter supplies these expected bytes.
const CASES: [ColorCase; 5] = [
    ColorCase {
        chromaticities: None,
        source: [128; 3],
        expected: [188; 3],
    },
    ColorCase {
        chromaticities: Some([31270, 32900, 15000, 6000, 30000, 60000, 64000, 33000]),
        source: [255, 0, 0],
        expected: [0, 0, 255],
    },
    ColorCase {
        chromaticities: Some([34570, 35850, 64000, 33000, 30000, 60000, 15000, 6000]),
        source: [128, 64, 192],
        expected: [191, 137, 223],
    },
    ColorCase {
        chromaticities: Some([33333, 33333, 64000, 33000, 30000, 60000, 15000, 6000]),
        source: [128, 64, 192],
        expected: [192, 138, 224],
    },
    ColorCase {
        chromaticities: Some([33333, 33333, 64000, 33000, 30000, 60000, 15000, 6000]),
        source: [255, 0, 0],
        expected: [255, 21, 11],
    },
];

fn source_font(case: &ColorCase, alpha: u8) -> Vec<u8> {
    let mut png = Vec::new();
    let mut encoder = png::Encoder::new(&mut png, 1, 1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(100000));
    if let Some(values) = case.chromaticities {
        let [wx, wy, rx, ry, gx, gy, bx, by] = values.map(png::ScaledFloat::from_scaled);
        encoder.set_source_chromaticities(png::SourceChromaticities {
            white: (wx, wy),
            red: (rx, ry),
            green: (gx, gy),
            blue: (bx, by),
        });
    }
    let mut writer = encoder.write_header().unwrap();
    writer
        .write_image_data(&[case.source[0], case.source[1], case.source[2], alpha])
        .unwrap();
    drop(writer);
    positioned_sbix(&png)
}

fn positioned_sbix(png: &[u8]) -> Vec<u8> {
    let mut sbix = sbix_table(png, &[SbixStrikeFixture::new(20, true)]);
    let strike = read_be_u32(&sbix, 8) as usize;
    let record = strike + 4 + (ahem_num_glyphs() + 1) * 4;
    write_be_u16(&mut sbix, record, 2);
    write_be_u16(&mut sbix, record + 2, 3);
    ahem_with_tables(vec![(*b"sbix", sbix)])
}

fn prepare(bytes: Vec<u8>) -> crate::text::PreparedGlyphRun {
    FontData::try_from_bytes(bytes, 0, surgeist_test::fixtures::FONT_LIMITS)
        .unwrap()
        .prepare_glyph_run(
            &instance(20.0),
            &one_glyph(),
            limits(),
            &mut crate::accounting::RenderAccountingContext::default(),
        )
        .unwrap()
}

#[test]
fn selected_png_colors_retain_converted_pixels_original_font_and_strike_placement() {
    for case in &CASES {
        for alpha in [128, 255] {
            let original = source_font(case, alpha);
            let ready = prepare(original.clone());
            assert_eq!(ready.font().bytes(), original);
            assert_eq!(ready.font().index(), 0);
            assert_eq!(ready.instance(), &instance(20.0));
            assert_eq!(ready.glyphs(), one_glyph());
            let expected_rect = Rect::new(5.0, 11.0, 1.0, 1.0);
            assert_eq!(ready.bounds().ink_rect(), Some(expected_rect));
            let [crate::text::GlyphPaint::Bitmap { image, rect, .. }] = ready.paints() else {
                panic!("the selected sbix glyph must retain its bitmap");
            };
            assert_eq!(*rect, expected_rect);
            assert_eq!(image.size(), Size::new(1.0, 1.0));
            assert_eq!(
                image.content_identity().source_bytes().as_ref(),
                [case.expected[0], case.expected[1], case.expected[2], alpha]
            );
        }
    }
}

#[test]
fn real_gpu_selected_png_colors_and_straight_alpha_reach_direct_and_graph_pixels() {
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(12.0, 16.0), 1.0)).unwrap();
    let backdrop = [20u8, 40, 60];
    let background = Color::try_rgba(20.0 / 255.0, 40.0 / 255.0, 60.0 / 255.0, 1.0).unwrap();
    for case in &CASES {
        for alpha in [128u8, 255] {
            let ready = prepare(source_font(case, alpha));
            let mut scene = Scene::new();
            scene.fill(Rect::new(0.0, 0.0, 12.0, 16.0), background);
            scene.prepared_glyph_run(
                ready,
                Transform::identity(),
                TextPaint::try_fill(Color::BLACK.into()).unwrap(),
            );
            // The existing numeric-sRGB receiver composites converted straight
            // source channels with the original alpha over this opaque backdrop.
            let expected = std::array::from_fn::<_, 3, _>(|index| {
                let a = u32::from(alpha);
                ((u32::from(case.expected[index]) * a
                    + u32::from(backdrop[index]) * (255 - a)
                    + 127)
                    / 255) as u8
            });
            for graph in [false, true] {
                let stats = if graph {
                    pollster::block_on(renderer.render_forced_base_graph_for_test(
                        &mut surface,
                        &scene,
                        Parameters::default(),
                        format,
                    ))
                    .unwrap()
                    .stats
                } else {
                    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                        .unwrap()
                };
                assert_eq!(
                    stats.route,
                    Some(if graph {
                        RenderRoute::GpuGraph
                    } else {
                        RenderRoute::DirectVello
                    })
                );
                assert_eq!(stats.glyphs, 1);
                let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                for (position, expected) in [
                    ((5, 11), [expected[0], expected[1], expected[2], 255]),
                    ((4, 11), [20, 40, 60, 255]),
                ] {
                    let actual = pixel_rgba(&image, position.0, position.1);
                    assert!(
                        actual.iter().zip(expected).all(|(a, e)| a.abs_diff(e) <= 2),
                        "source={:?},alpha={alpha},graph={graph},position={position:?}: {actual:?} != {expected:?}",
                        case.source
                    );
                }
            }
        }
    }
}

#[test]
fn real_gpu_selected_icc_png_preserves_converted_ink_on_both_routes() {
    let png = icc_png::image(
        &[128, 128, 192, 128],
        Some(&icc_png::matrix_profile(false)),
        &[(png::chunk::sRGB, &[0])],
        &[],
    );
    // The fixture's per-channel TRCs, P3-to-sRGB matrix and sRGB transfer
    // give [196.976, 134.791, 216.167] after scaling to byte range.
    assert_icc_readback(&png, [197, 135, 216, 128], [109, 88, 138, 255]);
}

#[test]
fn real_gpu_selected_icc_lut_png_preserves_converted_ink_on_both_routes() {
    let profile = icc_png::colorimetric_lut_profile(icc_png::lut16_transform(), b"XYZ ", 4);
    let png = icc_png::image(&[128, 128, 128, 128], Some(&profile), &[], &[]);
    // The neutral P3 corner sum gives a D50 neutral at linear 128/255.
    // The sRGB transfer maps it to byte 188; source-over rounds the result
    // on the original [20, 40, 60] destination with the original 128 alpha.
    assert_icc_readback(&png, [188, 188, 188, 128], [104, 114, 124, 255]);
}

#[test]
fn real_gpu_selected_icc_multi_process_png_preserves_ink_on_both_routes() {
    let profile = icc_png::mpe_matrix_profile();
    let png = icc_png::image(&[128, 128, 128, 128], Some(&profile), &[], &[]);
    // The float Display-P3 matrix maps equal linear samples to D50 neutral.
    // sRGB encodes 128/255 linear to 188. The original 128/255 alpha then
    // composites over the test backdrop, preserving the retained placement.
    assert_icc_readback(&png, [188, 188, 188, 128], [104, 114, 124, 255]);
}

fn assert_icc_readback(png: &[u8], converted: [u8; 4], composited: [u8; 4]) {
    let original = positioned_sbix(png);
    let ready = prepare(original.clone());
    assert_eq!(ready.font().bytes(), original);
    assert_eq!(
        ready.bounds().ink_rect(),
        Some(Rect::new(5.0, 11.0, 1.0, 1.0))
    );
    let [crate::text::GlyphPaint::Bitmap { image, .. }] = ready.paints() else {
        panic!("the selected sbix glyph must retain its bitmap");
    };
    assert_eq!(image.content_identity().source_bytes().as_ref(), converted);
    let mut scene = Scene::new();
    scene.fill(
        Rect::new(0.0, 0.0, 12.0, 16.0),
        Color::try_rgba(20.0 / 255.0, 40.0 / 255.0, 60.0 / 255.0, 1.0).unwrap(),
    );
    scene.prepared_glyph_run(
        ready,
        Transform::identity(),
        TextPaint::try_fill(Color::BLACK.into()).unwrap(),
    );
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(12.0, 16.0), 1.0)).unwrap();
    for route in [RenderRoute::DirectVello, RenderRoute::GpuGraph] {
        let parameters = Parameters::default();
        let stats = if route == RenderRoute::DirectVello {
            pollster::block_on(renderer.render(&mut surface, &scene, parameters)).unwrap()
        } else {
            pollster::block_on(renderer.render_forced_base_graph_for_test(
                &mut surface,
                &scene,
                parameters,
                format,
            ))
            .unwrap()
            .stats
        };
        assert_eq!(stats.route, Some(route));
        assert_eq!(stats.glyphs, 1);
        let readback = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for ((x, y), expected) in [((5, 11), composited), ((4, 11), [20, 40, 60, 255])] {
            let actual = pixel_rgba(&readback, x, y);
            assert!(
                actual.iter().zip(expected).all(|(a, e)| a.abs_diff(e) <= 2),
                "route={route:?},point=({x},{y}),actual={actual:?},expected={expected:?}"
            );
        }
    }
}

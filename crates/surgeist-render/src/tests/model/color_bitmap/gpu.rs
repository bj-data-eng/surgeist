use super::*;
use crate::tests::support::{default_graph_working_format_for_test, pixel_rgba};
use crate::{Options, Parameters, RenderRoute, Renderer};

fn registered_layout(
    bytes: Vec<u8>,
    size: f32,
    source: &str,
    spacing: f64,
    alpha: f32,
) -> surgeist_text::Layout {
    use surgeist_text as text;
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
        .layout(
            text::Source::new(source),
            text::Style {
                font: text::Font::new().family("Ahem"),
                size,
                line_height: text::LineHeight::Absolute(f64::from(size)),
                letter_spacing: spacing,
                brush: text::Brush::color(0.0, 0.0, 1.0, alpha),
                ..text::Style::default()
            },
            text::Options {
                quantize: false,
                ..text::Options::default()
            },
        )
        .unwrap()
}

#[test]
fn real_gpu_v0_v1_and_sbix_use_retained_intrinsic_red_in_direct_and_graph() {
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(24.0, 24.0), 1.0)).unwrap();
    for (bytes, size, inside, outside) in [
        (ahem_color_font(valid_cpal()), 10.0, (2, 2), (12, 2)),
        (translated_clipped_v1(), 10.0, (4, 4), (2, 4)),
        (offset_sbix(), 20.0, (3, 16), (4, 16)),
    ] {
        let input = registered_layout(bytes, size, "X", 0.0, 1.0);
        let ready = crate::prepare_text_layout(
            &input,
            Transform::translation(1.0, 0.0).unwrap(),
            crate::TextLayoutLimits::new(limits(), 4, 4, 4),
        )
        .unwrap();
        assert!(std::ptr::eq(ready.layout(), &input));
        for graph in [false, true] {
            let stats = if graph {
                pollster::block_on(renderer.render_forced_base_graph_for_test(
                    &mut surface,
                    ready.scene(),
                    Parameters::default(),
                    format,
                ))
                .unwrap()
                .stats
            } else {
                pollster::block_on(renderer.render(
                    &mut surface,
                    ready.scene(),
                    Parameters::default(),
                ))
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
            let actual = pixel_rgba(&image, inside.0, inside.1);
            assert!(
                actual
                    .iter()
                    .zip([255u8, 0, 0, 255])
                    .all(|(actual, expected)| actual.abs_diff(expected) <= 2),
                "graph={graph},size={size},actual={actual:?}"
            );
            assert!(
                pixel_rgba(&image, outside.0, outside.1)[3] <= 2,
                "graph={graph},size={size}: outside selected representation must stay transparent"
            );
        }
    }
}

#[test]
fn real_gpu_mixed_overlap_keeps_foreground_after_intrinsic_color() {
    let input = registered_layout(ahem_color_font(valid_cpal()), 10.0, "AXA", -10.0, 0.5);
    let run = input.run_views().next().unwrap();
    assert_eq!(
        run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>(),
        [35, 58, 35]
    );
    assert_eq!(
        run.glyphs()
            .map(|glyph| glyph.pen() + f64::from(glyph.x()))
            .collect::<Vec<_>>(),
        [0.0, 0.0, 0.0]
    );
    let ready = crate::prepare_text_layout(
        &input,
        Transform::identity(),
        crate::TextLayoutLimits::new(limits(), 4, 4, 4),
    )
    .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(12.0, 12.0), 1.0)).unwrap();
    for graph in [false, true] {
        if graph {
            pollster::block_on(renderer.render_forced_base_graph_for_test(
                &mut surface,
                ready.scene(),
                Parameters::default(),
                format,
            ))
            .unwrap();
        } else {
            pollster::block_on(renderer.render(&mut surface, ready.scene(), Parameters::default()))
                .unwrap();
        }
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        let actual = pixel_rgba(&image, 3, 3);
        // Opaque intrinsic red covers the first half-blue foreground; the final
        // half-blue foreground then produces numeric-sRGB purple. Merging all
        // foreground contours before the intrinsic operation would yield red.
        assert!(
            actual
                .iter()
                .zip([128u8, 0, 128, 255])
                .all(|(actual, expected)| actual.abs_diff(expected) <= 2),
            "graph={graph}: {actual:?}"
        );
    }
}

#[test]
fn real_gpu_one_text_layout_retains_outline_color_bitmap_and_image_resources() {
    let input = registered_layout(mixed_representation_font(), 16.0, "AXB", 0.0, 1.0);
    let ready = crate::prepare_text_layout(
        &input,
        Transform::identity(),
        crate::TextLayoutLimits::new(limits(), 4, 4, 4),
    )
    .unwrap();
    assert!(std::ptr::eq(ready.layout(), &input));
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(56.0, 24.0), 1.0)).unwrap();
    for graph in [false, true] {
        let stats = if graph {
            pollster::block_on(renderer.render_forced_base_graph_for_test(
                &mut surface,
                ready.scene(),
                Parameters::default(),
                format,
            ))
            .unwrap()
            .stats
        } else {
            pollster::block_on(renderer.render(&mut surface, ready.scene(), Parameters::default()))
                .unwrap()
        };
        assert_eq!(stats.glyphs, 3);
        assert_eq!(stats.images, 1);
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for (point, expected) in [
            ((3, 3), [0u8, 0, 255, 255]),
            ((19, 3), [255, 0, 0, 255]),
            ((32, 15), [255, 0, 0, 255]),
            ((36, 5), [0, 0, 0, 0]),
        ] {
            let actual = pixel_rgba(&pixels, point.0, point.1);
            assert!(
                actual.iter().zip(expected).all(|(a, e)| a.abs_diff(e) <= 2),
                "graph={graph},point={point:?},actual={actual:?}"
            );
        }
    }
}

fn empty_intrinsic_keeps_two_foreground_layers(bytes: Vec<u8>) {
    let empty = FontData::try_from_bytes(bytes.clone(), 0)
        .unwrap()
        .prepare_glyph_run(&instance(10.0), &one_glyph(), limits())
        .unwrap();
    assert_eq!(empty.bounds().kind(), TextRunBoundsKind::Empty);
    let input = registered_layout(bytes, 10.0, "AXA", -10.0, 0.5);
    let ready = crate::prepare_text_layout(
        &input,
        Transform::identity(),
        crate::TextLayoutLimits::new(limits(), 4, 4, 4),
    )
    .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(12.0, 12.0), 1.0)).unwrap();
    let mut failures = Vec::new();
    for graph in [false, true] {
        if graph {
            pollster::block_on(renderer.render_forced_base_graph_for_test(
                &mut surface,
                ready.scene(),
                Parameters::default(),
                format,
            ))
            .unwrap();
        } else {
            pollster::block_on(renderer.render(&mut surface, ready.scene(), Parameters::default()))
                .unwrap();
        }
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        let actual = pixel_rgba(&image, 3, 3);
        // Two distinct half-alpha blue operations compose to alpha .75, even
        // when the intervening intrinsic representation contributes no pixels.
        if !actual
            .iter()
            .zip([0u8, 0, 255, 191])
            .all(|(a, e)| a.abs_diff(e) <= 2)
        {
            failures.push((graph, actual));
        }
    }
    assert!(
        failures.is_empty(),
        "empty intrinsic selection must retain both foreground layers: {failures:?}"
    );
}

#[test]
fn empty_intrinsic_transparent_colr_preserves_foreground_alpha_direct_and_graph() {
    let mut palette = valid_cpal();
    *palette.last_mut().unwrap() = 0;
    empty_intrinsic_keeps_two_foreground_layers(ahem_color_font(palette));
}

#[test]
fn empty_intrinsic_transparent_sbix_preserves_foreground_alpha_direct_and_graph() {
    empty_intrinsic_keeps_two_foreground_layers(ahem_sbix_font(encoded_png(
        png::ColorType::Rgba,
        &[255, 0, 0, 0],
    )));
}

#[test]
fn empty_intrinsic_empty_colr_outline_preserves_foreground_alpha_direct_and_graph() {
    let bytes = ahem_color_font(valid_cpal());
    let mut colr = table_bytes(&bytes, b"COLR");
    write_be_u16(&mut colr, 20, 3); // Ahem space is a genuine zero-contour glyph.
    empty_intrinsic_keeps_two_foreground_layers(font_with_tables(&bytes, vec![(*b"COLR", colr)]));
}

#[test]
fn empty_intrinsic_clipped_colr_preserves_foreground_alpha_direct_and_graph() {
    let bytes = translated_clipped_v1();
    let mut colr = table_bytes(&bytes, b"COLR");
    let at = colr.len() - 8;
    for (i, value) in [2000, 2000, 2100, 2100].into_iter().enumerate() {
        write_be_u16(&mut colr, at + i * 2, value);
    }
    empty_intrinsic_keeps_two_foreground_layers(font_with_tables(&bytes, vec![(*b"COLR", colr)]));
}

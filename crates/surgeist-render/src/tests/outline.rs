use super::support::{
    AHEM_FONT_BYTES, AHEM_GLYPH_X, default_graph_working_format_for_test, pixel_rgba,
};
use crate::{
    Capabilities, Color, FontData, Gradient, GradientStop, Image, ImageQuality, Options,
    OutlineLimits, OutlineSynthesis, Parameters, Point, PreparedOutline, Rect, RenderRoute,
    Renderer, Scene, Shape, Size, TextGlyph, TextInstance, TextPaint, Transform,
    command::RenderCommand,
    encode::encode_vello_scene_with_initial_transform,
    vello_engine::scene::{VelloFillRuleObservationForTest, VelloPathElementObservationForTest},
};

fn overlapping_outline() -> PreparedOutline {
    let font = FontData::try_from_bytes(AHEM_FONT_BYTES.to_vec(), 0).unwrap();
    let instance = TextInstance::try_new(1000.0, vec![], OutlineSynthesis::none()).unwrap();
    let glyph = TextGlyph::try_new(AHEM_GLYPH_X, 0.0, 800.0, -10.0).unwrap();
    font.prepare_outline(
        &instance,
        &[glyph, glyph],
        OutlineLimits::new(2, 16, 1 << 20, 1 << 20),
    )
    .unwrap()
}

// An actual static composite X repeats the original A contour twice. No source
// point flattening: the production qualifier/provider must traverse both records.
fn overlapping_composite_outline() -> PreparedOutline {
    fn u16_at(bytes: &[u8], at: usize) -> u16 {
        u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
    }
    fn table(tag: &[u8; 4]) -> &[u8] {
        for i in 0..usize::from(u16_at(AHEM_FONT_BYTES, 4)) {
            let at = 12 + i * 16;
            if &AHEM_FONT_BYTES[at..at + 4] == tag {
                let start = u32::from_be_bytes(AHEM_FONT_BYTES[at + 8..at + 12].try_into().unwrap())
                    as usize;
                let len = u32::from_be_bytes(AHEM_FONT_BYTES[at + 12..at + 16].try_into().unwrap())
                    as usize;
                return &AHEM_FONT_BYTES[start..start + len];
            }
        }
        panic!("missing fixture table");
    }
    let composite: Vec<u8> = [-1i16, 0, -200, 1000, 800, 35, 35, 0, 0, 3, 35, 0, 0]
        .into_iter()
        .flat_map(i16::to_be_bytes)
        .collect();
    let old_loca = table(b"loca");
    let old_glyf = table(b"glyf");
    let mut loca = Vec::new();
    let mut glyf = Vec::new();
    for id in 0..usize::from(u16_at(table(b"maxp"), 4)) {
        loca.extend(u16::try_from(glyf.len() / 2).unwrap().to_be_bytes());
        if id == AHEM_GLYPH_X as usize {
            glyf.extend_from_slice(&composite);
        } else {
            glyf.extend_from_slice(
                &old_glyf[usize::from(u16_at(old_loca, id * 2)) * 2
                    ..usize::from(u16_at(old_loca, id * 2 + 2)) * 2],
            );
        }
        if !glyf.len().is_multiple_of(2) {
            glyf.push(0);
        }
    }
    loca.extend(u16::try_from(glyf.len() / 2).unwrap().to_be_bytes());
    let count = usize::from(u16_at(AHEM_FONT_BYTES, 4));
    let mut bytes = AHEM_FONT_BYTES[..12 + count * 16].to_vec();
    for i in 0..count {
        let at = 12 + i * 16;
        let tag: &[u8; 4] = AHEM_FONT_BYTES[at..at + 4].try_into().unwrap();
        let data = match tag {
            b"loca" => &loca[..],
            b"glyf" => &glyf[..],
            _ => table(tag),
        };
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let start = u32::try_from(bytes.len()).unwrap();
        bytes[at + 8..at + 12].copy_from_slice(&start.to_be_bytes());
        bytes[at + 12..at + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        bytes.extend_from_slice(data);
    }
    let font = FontData::try_from_bytes(bytes, 0).unwrap();
    let instance = TextInstance::try_new(1000.0, vec![], OutlineSynthesis::none()).unwrap();
    let glyph = TextGlyph::try_new(AHEM_GLYPH_X, 0.0, 800.0, -10.0).unwrap();
    let result = font
        .prepare_outline(
            &instance,
            &[glyph],
            OutlineLimits::new(1, 16, 1 << 20, 1 << 20),
        )
        .unwrap();
    assert_eq!(result.glyphs(), [glyph]);
    assert_eq!(result.path_elements().count(), 10);
    result
}

#[test]
fn prepared_outline_normalization_and_encoder_share_one_real_run_wide_nonzero_draw() {
    let outline = overlapping_outline();
    let transform = Transform::translation(7.0, -3.0).unwrap();
    let mut scene = Scene::new();
    scene.prepared_outline(
        outline.clone(),
        transform,
        TextPaint::try_fill(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap().into()).unwrap(),
    );
    let normalized = scene.normalize(Capabilities::CURRENT).unwrap();
    let RenderCommand::PreparedOutline {
        outline: actual,
        transform: actual_transform,
        ..
    } = &normalized.commands[0]
    else {
        panic!("prepared command must preserve its owner");
    };
    assert!(actual.is_same_preparation(&outline));
    assert_eq!(*actual_transform, transform);
    let encoded =
        encode_vello_scene_with_initial_transform(&normalized, Transform::identity()).unwrap();
    let observation = encoded.observation_for_test();
    assert_eq!(observation.glyph_run_count_for_test(), 0);
    let draws = observation.solid_path_draws_for_test().unwrap();
    assert_eq!(draws.len(), 1);
    assert_eq!(draws[0].fill_rule, VelloFillRuleObservationForTest::NonZero);
    assert_eq!(draws[0].transform, [1.0, 0.0, 0.0, 1.0, 7.0, -3.0]);
    assert_eq!(
        draws[0]
            .geometry
            .iter()
            .filter(|p| matches!(p, VelloPathElementObservationForTest::MoveTo(_)))
            .count(),
        2
    );
}

#[test]
fn prepared_outline_backend_placement_refuses_before_any_frame_output() {
    for transform in [
        Transform::scale(f64::MAX, 1.0).unwrap(),
        Transform::translation(1e30, 0.0).unwrap(),
    ] {
        let mut scene = Scene::new();
        scene.prepared_outline(
            overlapping_outline(),
            transform,
            TextPaint::try_fill(Color::BLACK.into()).unwrap(),
        );
        let normalized = scene.normalize(Capabilities::CURRENT).unwrap();
        let error =
            match encode_vello_scene_with_initial_transform(&normalized, Transform::identity()) {
                Ok(_) => panic!("unrepresentable composed affine cannot encode"),
                Err(error) => error,
            };
        assert!(matches!(
            error.outline_preparation_diagnostic(),
            Some(crate::OutlinePreparationError::Numeric {
                phase: crate::OutlineNumericPhase::Placement,
                ..
            })
        ));
    }
    let mut singular = Scene::new();
    singular.prepared_outline(
        overlapping_outline(),
        Transform::scale(0.0, 0.0).unwrap(),
        TextPaint::try_fill(Color::BLACK.into()).unwrap(),
    );
    assert!(
        encode_vello_scene_with_initial_transform(
            &singular.normalize(Capabilities::CURRENT).unwrap(),
            Transform::identity()
        )
        .is_ok()
    );
}

#[test]
fn prepared_outline_real_gpu_direct_and_graph_keep_overlapping_translucent_glyphs_one_fill() {
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let working_format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
    for (outline, graph) in [overlapping_outline(), overlapping_composite_outline()]
        .into_iter()
        .flat_map(|outline| [false, true].map(|graph| (outline.clone(), graph)))
    {
        let mut scene = Scene::new();
        scene.fill(
            Shape::rect(Rect::new(0.0, 0.0, 2.0, 2.0)),
            Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap(),
        );
        let paint =
            TextPaint::try_fill(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap().into()).unwrap();
        let draw = |scene: &mut Scene| {
            scene.prepared_outline(
                outline.clone(),
                Transform::scale(0.002, 0.002).unwrap(),
                paint,
            );
        };
        draw(&mut scene);
        let stats = if graph {
            pollster::block_on(renderer.render_forced_base_graph_for_test(
                &mut surface,
                &scene,
                Parameters::default(),
                working_format,
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
        let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        let [r, g, b, a] = pixel_rgba(&output, 0, 0);
        assert!(
            r >= 253 && (125..=130).contains(&g) && (125..=130).contains(&b) && a == 255,
            "one half-opacity run over white: {:?}",
            [r, g, b, a]
        );
    }
}

#[test]
fn prepared_outline_real_gpu_color_gradient_and_image_keep_run_local_paint_space() {
    let font = FontData::try_from_bytes(AHEM_FONT_BYTES.to_vec(), 0).unwrap();
    let outline = font
        .prepare_outline(
            &TextInstance::try_new(1.0, vec![], OutlineSynthesis::none()).unwrap(),
            &[
                TextGlyph::try_new(AHEM_GLYPH_X, 0.0, 0.8, 1.0).unwrap(),
                TextGlyph::try_new(AHEM_GLYPH_X, 2.0, 0.8, 1.0).unwrap(),
            ],
            OutlineLimits::new(2, 16, 1 << 20, 1 << 20),
        )
        .unwrap();
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let gradient = Gradient::try_linear(
        Point::try_new(0.0, 0.0).unwrap(),
        Point::try_new(3.0, 0.0).unwrap(),
        vec![
            GradientStop::try_new(0.0, red).unwrap(),
            GradientStop::try_new(1.0, blue).unwrap(),
        ],
    )
    .unwrap();
    let image = Image::from_rgba(
        Size::new(3.0, 1.0),
        vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255],
    )
    .unwrap()
    .quality(ImageQuality::Low);
    let paints = [
        TextPaint::try_fill(red.into()).unwrap(),
        TextPaint::try_fill(gradient.into()).unwrap(),
        TextPaint::try_fill(image.into()).unwrap(),
    ];
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let working_format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(8.0, 1.0), 1.0)).unwrap();
    for (kind, paint) in paints.into_iter().enumerate() {
        for graph in [false, true] {
            let mut scene = Scene::new();
            let draw = |scene: &mut Scene| {
                scene.prepared_outline(
                    outline.clone(),
                    Transform::try_new([2.0, 0.0, 0.0, 1.0, 1.0, 0.0]).unwrap(),
                    paint.clone(),
                );
            };
            draw(&mut scene);
            let stats = if graph {
                pollster::block_on(renderer.render_forced_base_graph_for_test(
                    &mut surface,
                    &scene,
                    Parameters::default(),
                    working_format,
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
            let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            // Literal glyph squares occupy [1,3] and [5,7] after placement.
            // Missing/double placement changes these occupied pixels.
            for x in [0, 3, 4, 7] {
                assert!(
                    pixel_rgba(&output, x, 0)[3] <= 2,
                    "kind{kind}/graph{graph}: unexpected ink at{x}"
                );
            }
            let first = pixel_rgba(&output, 1, 0);
            let second = pixel_rgba(&output, 5, 0);
            assert!(
                first[3] >= 250 && second[3] >= 250,
                "kind{kind}/graph{graph}: lost ink {first:?}/{second:?}"
            );
            if kind == 0 {
                assert!(first[0] >= 250 && first[2] <= 2 && second[0] >= 250 && second[2] <= 2);
            } else {
                // Run-local samples are x.25 and2.25. A brush restarted at
                // each glyph would make both red; missing/double brush
                // placement also fails these distinct red/blue dominances.
                assert!(
                    i16::from(first[0]) - i16::from(first[2]) > 100
                        && i16::from(second[2]) - i16::from(second[0]) > 50,
                    "kind{kind}/graph{graph}: brush space {first:?}/{second:?}"
                );
            }
        }
    }
}

#[test]
fn bold_contour_copies_are_one_nonzero_fill_on_direct_and_base_graph_routes() {
    let outlines: Vec<_> = [overlapping_outline(), overlapping_composite_outline()]
        .into_iter()
        .map(|plain| {
            let instance = TextInstance::try_new(
                plain.instance().size(),
                plain.instance().coordinates().to_vec(),
                OutlineSynthesis::try_new(0.0, plain.instance().size() / 36.0_f32).unwrap(),
            )
            .unwrap();
            let prepared = plain
                .font()
                .prepare_outline(
                    &instance,
                    plain.glyphs(),
                    OutlineLimits::new(2, 32, 1 << 20, 1 << 20),
                )
                .unwrap();
            assert_eq!(prepared.path_elements().count(), 20);
            assert_eq!(prepared.glyphs(), plain.glyphs());
            assert_eq!(prepared.instance(), &instance);
            prepared
        })
        .collect();
    // The original preparations, font inputs and instance temporaries have
    // dropped. Retained preparations supply both encoder and GPU geometry.
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let working_format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
    for outline in outlines {
        for graph in [false, true] {
            let mut scene = Scene::new();
            scene.fill(
                Shape::rect(Rect::new(0.0, 0.0, 2.0, 2.0)),
                Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap(),
            );
            scene.prepared_outline(
                outline.clone(),
                Transform::scale(0.002, 0.002).unwrap(),
                TextPaint::try_fill(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap().into()).unwrap(),
            );
            let normalized = scene.normalize(Capabilities::CURRENT).unwrap();
            let encoded =
                encode_vello_scene_with_initial_transform(&normalized, Transform::identity())
                    .unwrap();
            let observation = encoded.observation_for_test();
            let draws = observation.solid_path_draws_for_test().unwrap();
            assert_eq!(draws.len(), 2); // white background, then one text path
            assert_eq!(draws[1].fill_rule, VelloFillRuleObservationForTest::NonZero);
            assert_eq!(
                draws[1]
                    .geometry
                    .iter()
                    .filter(|point| matches!(point, VelloPathElementObservationForTest::MoveTo(_)))
                    .count(),
                4
            );
            let stats = if graph {
                pollster::block_on(renderer.render_forced_base_graph_for_test(
                    &mut surface,
                    &scene,
                    Parameters::default(),
                    working_format,
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
            let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let [r, g, b, a] = pixel_rgba(&output, 0, 0);
            assert!(
                r >= 253 && (125..=130).contains(&g) && (125..=130).contains(&b) && a == 255,
                "graph={graph}, one half-opacity path at shared original/copy ink: {:?}",
                [r, g, b, a]
            );
        }
    }
}

#[test]
fn any_absorbed_bold_point_refuses_even_when_other_copy_points_move() {
    let font = FontData::try_from_bytes(AHEM_FONT_BYTES.to_vec(), 0).unwrap();
    let instance =
        TextInstance::try_new(36.0, vec![], OutlineSynthesis::try_new(0.0, 1.0).unwrap()).unwrap();
    // Exact IEEE ties-to-even facts isolate partial copy loss: the left point
    // moves, but the right original/copy both round to33554464.
    assert_ne!(33_554_426.0_f64 as f32, 33_554_427.0_f64 as f32);
    assert_eq!(33_554_462.0_f64 as f32, 33_554_463.0_f64 as f32);
    let glyphs = [
        TextGlyph::try_new(AHEM_GLYPH_X, 20.0, 30.0, -10.0).unwrap(),
        TextGlyph::try_new(AHEM_GLYPH_X, 33_554_426.0, 0.0, 0.0).unwrap(),
    ];
    let error = font
        .prepare_outline(
            &instance,
            &glyphs,
            OutlineLimits::new(2, 20, 1 << 20, 1 << 20),
        )
        .unwrap_err();
    assert!(matches!(
        error.outline_preparation_diagnostic(),
        Some(crate::OutlinePreparationError::Numeric {
            phase: crate::OutlineNumericPhase::Synthesis,
            value: 1.0,
        })
    ));
    assert_eq!(glyphs[1].x(), 33_554_426.0);
}

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
    let outline = overlapping_outline();
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let working_format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
    for graph in [false, true] {
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

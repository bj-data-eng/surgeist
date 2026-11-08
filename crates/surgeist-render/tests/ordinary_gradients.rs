#![forbid(unsafe_code)]

use surgeist_render::{Color, Gradient, GradientStop, Point};

#[derive(Clone, Copy, Debug)]
enum Field {
    Linear,
    Radial,
    Sweep,
}

const FIELDS: [Field; 3] = [Field::Linear, Field::Radial, Field::Sweep];

fn red(alpha: f32) -> Color {
    Color::try_rgba(1.0, 0.0, 0.0, alpha).unwrap()
}

fn blue() -> Color {
    Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap()
}

fn point(x: f64, y: f64) -> Point {
    Point::try_new(x, y).unwrap()
}

fn gradient(field: Field, first: Color) -> Gradient {
    let stops = vec![
        GradientStop::try_new(0.5, first).unwrap(),
        GradientStop::try_new(1.0, blue()).unwrap(),
    ];
    match field {
        Field::Linear => Gradient::try_linear(point(0.0, 0.0), point(8.0, 0.0), stops),
        Field::Radial => Gradient::try_radial(point(0.0, 4.0), 8.0, stops),
        Field::Sweep => Gradient::try_sweep(point(12.0, 4.0), stops),
    }
    .unwrap()
}

fn assert_rgba(actual: [u8; 4], expected: [u8; 4], tolerance: u8, context: &str) {
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance),
        "{context}: actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
    );
}

#[test]
fn owned_native_conversion_preserves_the_leading_color_in_the_real_resolved_ramp() {
    for field in FIELDS {
        for (alpha, leading, midpoint) in [
            (1.0, [255, 0, 0, 255], [128, 0, 128, 255]),
            (0.5, [128, 0, 0, 128], [64, 0, 128, 191]),
        ] {
            let native: peniko::Gradient = gradient(field, red(alpha)).into();
            let mut encoding = vello_encoding::Encoding::new();
            encoding.encode_transform(vello_encoding::Transform::IDENTITY);
            encoding.encode_fill_style(peniko::Fill::NonZero);
            assert!(encoding.encode_shape(&peniko::kurbo::Rect::new(0.0, 0.0, 32.0, 8.0), true));
            encoding.encode_brush(&native, 1.0);
            let mut resolver = vello_encoding::Resolver::new();
            let mut packed = Vec::new();
            let (_, ramps, _) = resolver.resolve(&encoding, &mut packed);
            let width = usize::try_from(ramps.width).unwrap();
            assert!(
                width > 1,
                "the real backend must resolve a sampleable ramp for {field:?}"
            );
            let sample =
                |index| peniko::color::PremulRgba8::from_u32(ramps.data[index]).to_u8_array();
            // Images3 3.4.2 requires the first color before offset .5.
            // Native samples are premultiplied bytes: alpha .5 red is 128 red.
            assert_rgba(
                sample((width - 1) / 4),
                leading,
                0,
                &format!("{field:?} leading interval"),
            );
            // At t=.75 the supplied [.5,1] interval is halfway traversed.
            // The nearest sample of the declared 512-sample ramp can differ
            // by one byte from the exact midpoint after RGBA8 quantization.
            assert_rgba(
                sample(3 * (width - 1) / 4),
                midpoint,
                1,
                &format!("{field:?} interval midpoint"),
            );
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod gpu {
    use super::*;
    use surgeist_render::{
        EffectQualityPolicy, Image, ImageBuffer, Layer, Options, Parameters, Rect, RenderRoute,
        Renderer, ResolvedLayerAlphaMask, Scene, Size, Surface,
    };

    fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect::try_new(x, y, width, height).unwrap()
    }

    fn pixel(image: &ImageBuffer, x: u32, y: u32) -> [u8; 4] {
        let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
        image.rgba()[index..index + 4].try_into().unwrap()
    }

    fn read(
        renderer: &mut Renderer,
        surface: &mut Surface,
        scene: &Scene,
        graph: bool,
    ) -> ImageBuffer {
        let stats =
            pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
        assert_eq!(
            stats.route,
            Some(if graph {
                RenderRoute::GpuGraph
            } else {
                RenderRoute::DirectVello
            })
        );
        pollster::block_on(renderer.read_headless(surface)).unwrap()
    }

    fn scene(field: Field, first: Color, graph: bool) -> Scene {
        let mut scene = Scene::new();
        let draw = |scene: &mut Scene| {
            scene.fill(rect(0.0, 0.0, 24.0, 8.0), gradient(field, first));
            let zero_first = Gradient::try_linear(
                point(24.0, 0.0),
                point(28.0, 0.0),
                vec![
                    GradientStop::try_new(0.0, red(1.0)).unwrap(),
                    GradientStop::try_new(1.0, blue()).unwrap(),
                ],
            )
            .unwrap();
            scene.fill(rect(24.0, 0.0, 4.0, 8.0), zero_first);
            // One stop is already solid. Coincident endpoints deliberately
            // make any newly introduced, unused gradient geometry observable.
            let one_stop = Gradient::try_linear(
                point(30.0, 4.0),
                point(30.0, 4.0),
                vec![GradientStop::try_new(0.5, first).unwrap()],
            )
            .unwrap();
            scene.fill(rect(28.0, 0.0, 4.0, 8.0), one_stop);
        };
        if graph {
            let mask = Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128])
                .unwrap();
            scene.layer(
                Layer::new().with_resolved_alpha_mask(
                    ResolvedLayerAlphaMask::try_new(mask, rect(0.0, 0.0, 32.0, 8.0)).unwrap(),
                ),
                draw,
            );
        } else {
            draw(&mut scene);
        }
        scene
    }

    #[test]
    fn ordinary_fields_keep_the_first_color_until_its_positive_offset_on_both_gpu_routes() {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::try_new(32.0, 8.0).unwrap(), 1.0))
                .unwrap();
        // Twelve frames share one Renderer and one 256-pixel surface. In each
        // frame, the two control strips exercise the existing zero-first and
        // positive-offset one-stop/coincident-line behavior independently.
        for (alpha, leading, midpoint, masked_leading, masked_midpoint) in [
            (
                1.0,
                [255, 0, 0, 255],
                [128, 0, 128, 255],
                [255, 0, 0, 128],
                [128, 0, 128, 128],
            ),
            (
                0.5,
                [255, 0, 0, 128],
                [85, 0, 170, 191],
                [255, 0, 0, 64],
                [85, 0, 170, 96],
            ),
        ] {
            for field in FIELDS {
                for graph in [false, true] {
                    let image = read(
                        &mut renderer,
                        &mut surface,
                        &scene(field, red(alpha), graph),
                        graph,
                    );
                    // Integer physical indices determine color. Linear x/8,
                    // radial distance/8, and sweep quarter/three-quarter turns
                    // give t=.25 and .75 at these interior coverage points.
                    let (before, interior) = match field {
                        Field::Linear => ((2, 2), (6, 2)),
                        Field::Radial => ((2, 4), (6, 4)),
                        Field::Sweep => ((12, 6), (12, 2)),
                    };
                    let expected_leading = if graph { masked_leading } else { leading };
                    let expected_midpoint = if graph { masked_midpoint } else { midpoint };
                    let context = format!("{field:?}, alpha={alpha}, graph={graph}");
                    // Flat leading colors are exact; no blue contamination is
                    // allowed before the first supplied red stop.
                    assert_rgba(
                        pixel(&image, before.0, before.1),
                        expected_leading,
                        0,
                        &format!("{context}, leading"),
                    );
                    // Halfway interpolation is premultiplied: translucent red
                    // and opaque blue give (.25,0,.5,.75), hence straight
                    // (1/3,0,2/3,.75). Ramp quantization and mask conversion
                    // account for at most these small interior-byte bounds.
                    let tolerance = if graph { 3 } else { 2 };
                    assert_rgba(
                        pixel(&image, interior.0, interior.1),
                        expected_midpoint,
                        tolerance,
                        &format!("{context}, midpoint"),
                    );
                    let control_alpha = if graph { 128 } else { 255 };
                    assert_rgba(
                        pixel(&image, 25, 2),
                        [191, 0, 64, control_alpha],
                        tolerance,
                        &format!("{context}, zero-first quarter"),
                    );
                    assert_rgba(
                        pixel(&image, 27, 2),
                        [64, 0, 191, control_alpha],
                        tolerance,
                        &format!("{context}, zero-first three-quarter"),
                    );
                    assert_rgba(
                        pixel(&image, 30, 2),
                        expected_leading,
                        0,
                        &format!("{context}, one-stop solid"),
                    );
                    assert_rgba(
                        pixel(&image, 30, 6),
                        expected_leading,
                        0,
                        &format!("{context}, one-stop solid second row"),
                    );
                }
            }
        }
    }

    #[test]
    fn leading_native_record_work_refusal_preserves_publication_and_allows_zero_first_retry() {
        use surgeist_render::{
            RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus, RenderLimits,
            RenderPhase, RenderProfile, RenderQuantity, RenderRefusalReason, RenderWork,
        };

        let bounds = rect(0.0, 0.0, 8.0, 4.0);
        let mut positive_first = Scene::new();
        positive_first.fill(bounds, gradient(Field::Linear, red(1.0)));
        let accounting =
            RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
        let mut unrestricted = pollster::block_on(Renderer::new(
            Options::default().with_accounting(accounting),
        ))
        .unwrap();
        let mut unrestricted_surface =
            pollster::block_on(unrestricted.create_headless(Size::try_new(8.0, 4.0).unwrap(), 1.0))
                .unwrap();
        pollster::block_on(unrestricted.render(
            &mut unrestricted_surface,
            &positive_first,
            Parameters::default(),
        ))
        .unwrap();
        let full_report = unrestricted.take_last_attempt().unwrap();
        assert_eq!(full_report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(full_report.cleanup_fault(), None);
        let full_attempt = full_report.attempt().unwrap();
        let prepared_visits = full_attempt
            .phase_usage(RenderPhase::GradientPrepare)
            .work(RenderWork::GradientStopVisits);
        // Two supplied records need one first-record preflight and three
        // actual native records: the leading endpoint plus both supplied stops.
        assert_eq!(prepared_visits, 1 + 3);
        let total_visits = full_attempt.usage().work(RenderWork::GradientStopVisits);
        let cap = total_visits - 1;
        drop(unrestricted_surface);
        drop(unrestricted);

        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::GradientStopVisits, cap)
            .unwrap();
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default().with_accounting(accounting.with_limits(limits)),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::try_new(8.0, 4.0).unwrap(), 1.0))
                .unwrap();
        let mut solid = Scene::new();
        solid.fill(bounds, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
        let successful_stats =
            pollster::block_on(renderer.render(&mut surface, &solid, Parameters::default()))
                .unwrap();
        let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_rgba(
            pixel(&before, 2, 2),
            [0, 255, 0, 255],
            0,
            "solid publication before refusal",
        );

        let error = pollster::block_on(renderer.render(
            &mut surface,
            &positive_first,
            Parameters::default(),
        ))
        .unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("expected a named gradient-stop work refusal")
        };
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::GradientStopVisits)
        );
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: cap });
        assert_eq!(renderer.stats(), successful_stats);
        let refused_report = renderer.take_last_attempt().unwrap();
        assert_eq!(refused_report.status(), RenderAttemptStatus::Refused);
        assert_eq!(refused_report.cleanup_fault(), None);
        let refused_attempt = refused_report.attempt().unwrap();
        assert_eq!(refused_attempt.refusal(), Some(refusal));
        let accepted = refused_attempt.usage().work(RenderWork::GradientStopVisits);
        assert!(accepted > 0 && accepted <= cap);
        assert_eq!(refused_attempt.usage().work(RenderWork::Submissions), 0);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            before
        );
        assert_eq!(renderer.stats(), successful_stats);

        // Starting at zero needs no leading native endpoint. With all other
        // scene inputs held equal, that removes one record from materialization
        // and the encoder's separate visit, permitting a fresh publication.
        let zero_first = Gradient::try_linear(
            point(0.0, 0.0),
            point(8.0, 0.0),
            vec![
                GradientStop::try_new(0.0, red(1.0)).unwrap(),
                GradientStop::try_new(1.0, blue()).unwrap(),
            ],
        )
        .unwrap();
        let mut retry = Scene::new();
        retry.fill(bounds, zero_first);
        pollster::block_on(renderer.render(&mut surface, &retry, Parameters::default())).unwrap();
        let retry_report = renderer.take_last_attempt().unwrap();
        assert_eq!(retry_report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(retry_report.cleanup_fault(), None);
        let retry_attempt = retry_report.attempt().unwrap();
        assert_eq!(
            retry_attempt.usage().work(RenderWork::GradientStopVisits),
            total_visits - 2
        );
        assert_eq!(
            retry_attempt
                .phase_usage(RenderPhase::GradientPrepare)
                .work(RenderWork::GradientStopVisits),
            prepared_visits - 1
        );
        // Encode includes GradientPrepare. Removing that child's work exposes
        // the native encoder's independently charged three/two-record walk.
        assert_eq!(
            full_attempt
                .phase_usage(RenderPhase::Encode)
                .work(RenderWork::GradientStopVisits)
                - prepared_visits,
            3
        );
        assert_eq!(
            retry_attempt
                .phase_usage(RenderPhase::Encode)
                .work(RenderWork::GradientStopVisits)
                - (prepared_visits - 1),
            2
        );
        let after = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_ne!(after, before);
        assert_rgba(
            pixel(&after, 2, 2),
            [191, 0, 64, 255],
            2,
            "zero-first retry quarter",
        );
        assert_rgba(
            pixel(&after, 6, 2),
            [64, 0, 191, 255],
            2,
            "zero-first retry three-quarter",
        );
    }
}

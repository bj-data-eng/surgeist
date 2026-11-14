//! Independent fixtures through the public admitted-convolution receiver and real GPU readback.
//! Readback pixels are straight RGBA8; comments distinguish premultiplied equations.
use crate::{
    Color, ConvolutionInput, FilterAmount, FilterColorSpace, FilterDomains, FilterEdgeMode,
    FilterList, FilterOp, Image, ImageBuffer, ImageFit, ImageQuality, Layer, Options, Parameters,
    PreparedConvolution, Rect, RenderAccountingContext, RenderRoute, Renderer,
    ResolvedLayerAlphaMask, Scene, Size, Surface, prepare_convolution,
};

pub(super) fn bounds(width: u32, height: u32) -> Rect {
    Rect::try_new(0.0, 0.0, f64::from(width), f64::from(height)).unwrap()
}

pub(super) fn operation(
    input: ConvolutionInput<'_>,
    domain: Rect,
    color_space: FilterColorSpace,
) -> PreparedConvolution {
    prepare_convolution(
        input,
        FilterDomains::try_new(domain, domain, domain).unwrap(),
        color_space,
        &mut RenderAccountingContext::default(),
    )
    .unwrap()
}

pub(super) fn image(width: u32, height: u32, pixels: Vec<u8>) -> Image {
    Image::from_rgba(
        Size::try_new(f64::from(width), f64::from(height)).unwrap(),
        pixels,
    )
    .unwrap()
    .quality(ImageQuality::Low)
}

pub(super) fn image_scene(image: Image, domain: Rect, convolution: PreparedConvolution) -> Scene {
    let mut scene = Scene::new();
    scene.layer(Layer::new().with_convolution(convolution), |children| {
        children.image(image, domain, ImageFit::Stretch);
    });
    scene
}

pub(super) fn renderer_and_surface(width: u32, height: u32) -> (Renderer, Surface) {
    let mut renderer = pollster::block_on(Renderer::new(Options::default()))
        .expect("convolution fixtures require a real compatible GPU adapter");
    let surface = pollster::block_on(renderer.create_headless(
        Size::try_new(f64::from(width), f64::from(height)).unwrap(),
        1.0,
    ))
    .expect("convolution fixtures require a real headless GPU surface");
    (renderer, surface)
}

pub(super) fn render(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default()))
        .expect("the public convolution Layer must execute successfully");
    assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
    assert_eq!(stats, renderer.stats());
    pollster::block_on(renderer.read_headless(surface))
        .expect("explicit headless readback must observe the published frame")
}

pub(super) fn pixel(output: &ImageBuffer, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * output.size().width() + x) * 4) as usize;
    output.rgba()[offset..offset + 4].try_into().unwrap()
}

pub(super) fn assert_pixel(output: &ImageBuffer, x: u32, y: u32, expected: [u8; 4], tolerance: u8) {
    let actual = pixel(output, x, y);
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(a, e)| a.abs_diff(e) <= tolerance),
        "pixel ({x},{y}): {actual:?}, expected {expected:?} within {tolerance}"
    );
}

#[test]
fn translated_nested_opacity_capture_uses_a_finite_native_layer_clip() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 6);
    let source = Rect::new(1.0, 1.0, 3.0, 3.0);
    let convolution = operation(
        ConvolutionInput::new(&[1.0]).with_order(1.0, None),
        source,
        FilterColorSpace::Srgb,
    );
    let mut scene = Scene::new();
    scene.layer(Layer::new().with_convolution(convolution), |children| {
        children.layer(Layer::new().try_opacity(0.5).unwrap(), |nested| {
            nested.fill(source, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
        });
    });
    let output = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&output, 2, 2, [255, 0, 0, 128], 2);
    assert_pixel(&output, 0, 0, [0; 4], 1);
}

#[test]
fn translated_nested_opacity_capture_preserves_an_authored_local_clip() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 6);
    let source = Rect::new(1.0, 1.0, 3.0, 3.0);
    let convolution = operation(
        ConvolutionInput::new(&[1.0]).with_order(1.0, None),
        source,
        FilterColorSpace::Srgb,
    );
    let mut scene = Scene::new();
    scene.layer(Layer::new().with_convolution(convolution), |children| {
        children.layer(
            Layer::new()
                .try_opacity(0.5)
                .unwrap()
                .try_clip(crate::Shape::rect(Rect::new(2.0, 2.0, 1.0, 1.0)))
                .unwrap(),
            |nested| {
                nested.fill(source, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
            },
        );
    });
    let output = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&output, 2, 2, [255, 0, 0, 128], 2);
    assert_pixel(&output, 1, 2, [0; 4], 1);
    assert_pixel(&output, 2, 1, [0; 4], 1);
}

#[test]
fn public_convolution_gpu_rotates_asymmetric_kernel_on_actual_image() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 5);
    let domain = bounds(5, 5);
    let source: Vec<u8> = (0..25_u8)
        .flat_map(|value| [value * 10, value * 10, value * 10, 255])
        .collect();
    let convolution = operation(
        ConvolutionInput::new(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]),
        domain,
        FilterColorSpace::Srgb,
    );
    let output = render(
        &mut renderer,
        &mut surface,
        &image_scene(image(5, 5, source), domain, convolution),
    );
    // Center neighborhood is [60,70,80;110,120,130;160,170,180].
    // The 180-degree rotated weights give
    // 9*60+8*70+7*80+6*110+5*120+4*130+3*160+2*170+180 = 4440.
    // Dividing by45 gives98.666..., while unrotated weights give141.333....
    assert_pixel(&output, 2, 2, [99, 99, 99, 255], 1);
}

#[test]
fn public_convolution_gpu_addresses_all_edges_and_preserves_alpha() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let domain = bounds(3, 1);
    let source = image(
        3,
        1,
        vec![64, 64, 64, 255, 128, 128, 128, 255, 192, 192, 192, 255],
    );
    // A seven-column kernel with only its first coefficient set samples x+3.
    // Expected addressing below is handwritten for source indices3,4,5.
    for (edge, expected) in [
        (FilterEdgeMode::Duplicate, [192, 192, 192]),
        (FilterEdgeMode::Wrap, [64, 128, 192]),
        (FilterEdgeMode::Mirror, [192, 128, 64]),
        (FilterEdgeMode::None, [0, 0, 0]),
    ] {
        let convolution = operation(
            ConvolutionInput::new(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
                .with_order(7.0, Some(1.0))
                .with_target(Some(3.0), Some(0.0))
                .with_divisor(1.0)
                .with_edge_mode(edge)
                .with_preserve_alpha(true),
            domain,
            FilterColorSpace::Srgb,
        );
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(source.clone(), domain, convolution),
        );
        for (x, value) in (0..3).zip(expected) {
            assert_pixel(&output, x, 0, [value, value, value, 255], 1);
        }
    }
    let convolution = operation(
        ConvolutionInput::new(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            .with_order(7.0, Some(1.0))
            .with_divisor(1.0)
            .with_edge_mode(FilterEdgeMode::None),
        domain,
        FilterColorSpace::Srgb,
    );
    let output = render(
        &mut renderer,
        &mut surface,
        &image_scene(source, domain, convolution),
    );
    for x in 0..3 {
        assert_pixel(&output, x, 0, [0; 4], 0);
    }
}

#[test]
fn public_convolution_gpu_bias_empty_input_multibatch_and_normalization() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let domain = bounds(2, 1);
    let black = image(2, 1, vec![0, 0, 0, 255, 0, 0, 0, 255]);
    for (preserve_alpha, expected) in [(false, [143, 143, 143, 255]), (true, [64, 64, 64, 255])] {
        let convolution = operation(
            ConvolutionInput::new(&[2.0])
                .with_order(1.0, None)
                .with_divisor(1.0)
                .with_bias(0.25)
                .with_preserve_alpha(preserve_alpha),
            domain,
            FilterColorSpace::Srgb,
        );
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(black.clone(), domain, convolution),
        );
        // Without preserveAlpha: rawA=2+.25=2.25, rawRGB=.25*2.25=.5625.
        // PreserveAlpha uses straight RGB bias.25 and original alpha1.
        assert_pixel(&output, 0, 0, expected, 1);
    }
    for (preserve_alpha, expected) in [(false, [128, 128, 128, 128]), (true, [0; 4])] {
        let convolution = operation(
            ConvolutionInput::new(&[1.0])
                .with_order(1.0, None)
                .with_bias(0.5)
                .with_preserve_alpha(preserve_alpha),
            domain,
            FilterColorSpace::Srgb,
        );
        let mut scene = Scene::new();
        scene.layer(Layer::new().with_convolution(convolution), |_| {});
        let output = render(&mut renderer, &mut surface, &scene);
        // Empty input still executes: rawA=.5, premultipliedRGB=.5*.5=.25;
        // explicit straight readback is RGB.5 alpha.5. Preserved source alpha is0.
        assert_pixel(&output, 0, 0, expected, 1);
    }
    let original = [64, 128, 192, 255];
    let source = image(2, 1, [original, original].concat());
    for scale in [2.0_f64.powi(-130), 2.0_f64.powi(130)] {
        let convolution = operation(
            ConvolutionInput::new(&[scale])
                .with_order(1.0, None)
                .with_divisor(scale),
            domain,
            FilterColorSpace::Srgb,
        );
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(source.clone(), domain, convolution),
        );
        assert_pixel(&output, 0, 0, original, 1);
    }
    let mut kernel = [0.0; 33];
    kernel[0] = 65536.0;
    kernel[32] = -65536.0;
    let convolution = operation(
        ConvolutionInput::new(&kernel)
            .with_order(33.0, Some(1.0))
            .with_divisor(1.0)
            .with_bias(0.25)
            .with_preserve_alpha(true),
        domain,
        FilterColorSpace::Srgb,
    );
    let cancellation_scene = image_scene(image(2, 1, vec![255; 8]), domain, convolution);
    let output = render(&mut renderer, &mut surface, &cancellation_scene);
    // The first32-tap batch reaches65536, above half-float's65504 maximum.
    // The last tap cancels it exactly; only bias.25 remains. Partial clamping
    // or half-float saturation cannot produce this opaque result.
    for x in 0..2 {
        assert_pixel(&output, x, 0, [64, 64, 64, 255], 1);
    }
    // Select the real reduced ordinary working format through the existing
    // private format qualification front door, retaining the same public Layer
    // receiver and actual graph. Its signed accumulation still requires float32.
    let stats = pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
        &mut surface,
        &cancellation_scene,
        Parameters::default(),
        crate::resource::WorkingFormat::ReducedPrecision,
    ))
    .unwrap();
    assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
    let reduced = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for x in 0..2 {
        assert_pixel(&reduced, x, 0, [64, 64, 64, 255], 1);
    }
    let resources = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap()
        .internal_resource_manager_observation_for_test();
    assert!(
        resources.filter_float32_entries_for_test() > 0,
        "the actual GPU resource payload must include Rgba32Float accumulation storage"
    );
    assert!(resources.convolution_weight_entries_for_test() > 0);
    assert!(
        renderer
            .default_device_capabilities_for_test()
            .effect_precisions()
            .supports_filter_float32()
    );
    assert!(renderer.default_device_has_no_terminal_signal_for_test());
}

#[test]
fn public_convolution_gpu_selects_numeric_or_linear_srgb_working_color() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let domain = bounds(2, 1);
    let source = image(2, 1, vec![0, 0, 0, 255, 255, 255, 255, 255]);
    for (space, expected) in [
        (FilterColorSpace::Srgb, 128),
        (FilterColorSpace::LinearSrgb, 188),
    ] {
        let convolution = operation(
            ConvolutionInput::new(&[1.0, 1.0])
                .with_order(2.0, Some(1.0))
                .with_target(Some(0.0), Some(0.0))
                .with_divisor(2.0),
            domain,
            space,
        );
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(source.clone(), domain, convolution),
        );
        // Average encoded black/white is.5; average linear light is.5 and
        // encodes as1.055*.5^(1/2.4)-.055=.73535698..., or188 in RGBA8.
        assert_pixel(&output, 0, 0, [expected, expected, expected, 255], 1);
    }
    let fractional = image(2, 1, vec![128, 128, 128, 128, 255, 0, 255, 0]);
    let identity = operation(
        ConvolutionInput::new(&[1.0]).with_order(1.0, None),
        domain,
        FilterColorSpace::LinearSrgb,
    );
    let output = render(
        &mut renderer,
        &mut surface,
        &image_scene(fractional, domain, identity),
    );
    // Decoding and encoding RGB must leave fractional alpha unchanged, and
    // alpha-zero hidden color cannot become a visible sample through conversion.
    assert_pixel(&output, 0, 0, [128, 128, 128, 128], 2);
    assert_pixel(&output, 1, 0, [0; 4], 1);
}

#[test]
fn public_convolution_gpu_composes_real_nested_vectors_css_mask_and_opacity() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let domain = bounds(2, 1);
    let convolution = operation(
        ConvolutionInput::new(&[1.0]).with_order(1.0, None),
        domain,
        FilterColorSpace::Srgb,
    );
    let mask = ResolvedLayerAlphaMask::try_new(
        image(2, 1, vec![255, 255, 255, 255, 255, 255, 255, 128]),
        domain,
    )
    .unwrap();
    let filters = FilterList::try_ops(vec![FilterOp::brightness(
        FilterAmount::try_new(0.5).unwrap(),
    )])
    .unwrap();
    let outer = Layer::new()
        .with_filter_list(filters)
        .with_resolved_alpha_mask(mask)
        .try_opacity(0.5)
        .unwrap();
    let mut scene = Scene::new();
    scene.layer(outer, |css| {
        css.layer(Layer::new().with_convolution(convolution), |convolution| {
            convolution.layer(Layer::new().try_opacity(0.5).unwrap(), |vectors| {
                vectors.fill(domain, Color::try_rgba(0.8, 0.4, 0.2, 1.0).unwrap());
            });
        });
    });
    let output = render(&mut renderer, &mut surface, &scene);
    // The real nested vector capture has alpha.5. CSS brightness halves straight
    // RGB; the post-effect mask and outer opacity give alpha.25 and.12549....
    assert_pixel(&output, 0, 0, [102, 51, 26, 64], 2);
    assert_pixel(&output, 1, 0, [102, 51, 26, 32], 2);
    assert!(renderer.default_device_has_no_terminal_signal_for_test());
}

#[test]
fn public_convolution_gpu_signed_defaults_and_negative_alpha_are_canonical() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let domain = bounds(3, 1);
    let source = image(
        3,
        1,
        vec![64, 64, 64, 255, 128, 128, 128, 255, 192, 192, 192, 255],
    );
    for explicit_zero in [false, true] {
        let request = ConvolutionInput::new(&[-1.0, 1.0])
            .with_order(2.0, Some(1.0))
            .with_target(Some(0.0), Some(0.0))
            .with_bias(0.5)
            .with_preserve_alpha(true);
        let request = if explicit_zero {
            request.with_divisor(0.0)
        } else {
            request
        };
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(
                source.clone(),
                domain,
                operation(request, domain, FilterColorSpace::Srgb),
            ),
        );
        // Default divisor is1 for this zero-sum kernel. Rotated taps compute
        // source[x]-source[x+1]+.5; duplicate at the final cell gives .5.
        for (x, red) in [64, 64, 128].into_iter().enumerate() {
            assert_pixel(&output, x as u32, 0, [red, red, red, 255], 1);
        }
    }
    let output = render(
        &mut renderer,
        &mut surface,
        &image_scene(
            source.clone(),
            domain,
            operation(
                ConvolutionInput::new(&[-1.0, 1.0])
                    .with_order(2.0, Some(1.0))
                    .with_target(Some(0.0), Some(0.0))
                    .with_divisor(-1.0)
                    .with_preserve_alpha(true),
                domain,
                FilterColorSpace::Srgb,
            ),
        ),
    );
    for (x, red) in [64, 64, 0].into_iter().enumerate() {
        assert_pixel(&output, x as u32, 0, [red, red, red, 255], 1);
    }
    let output = render(
        &mut renderer,
        &mut surface,
        &image_scene(
            source.clone(),
            domain,
            operation(
                ConvolutionInput::new(&[-1.0])
                    .with_order(1.0, None)
                    .with_divisor(1.0),
                domain,
                FilterColorSpace::Srgb,
            ),
        ),
    );
    for x in 0..3 {
        assert_pixel(&output, x, 0, [0; 4], 1);
    }
    for invalid in [-1.0, 0.0, f64::NAN] {
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(
                source.clone(),
                domain,
                operation(
                    ConvolutionInput::new(&[1.0])
                        .with_order(1.0, None)
                        .with_unit_length(invalid, Some(0.75)),
                    domain,
                    FilterColorSpace::Srgb,
                ),
            ),
        );
        for (x, red) in [64, 128, 192].into_iter().enumerate() {
            assert_pixel(&output, x as u32, 0, [red, red, red, 255], 1);
        }
    }
}

#[test]
fn public_convolution_gpu_negative_multiple_periods_and_empty_domains_are_defined() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let domain = bounds(3, 1);
    let source = image(
        3,
        1,
        vec![64, 64, 64, 255, 128, 128, 128, 255, 192, 192, 192, 255],
    );
    let mut weights = [0.0; 15];
    weights[14] = 1.0;
    for (edge, reds) in [
        (FilterEdgeMode::Duplicate, [64, 64, 64]),
        (FilterEdgeMode::Wrap, [128, 192, 64]),
        (FilterEdgeMode::Mirror, [128, 64, 64]),
        (FilterEdgeMode::None, [0, 0, 0]),
    ] {
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(
                source.clone(),
                domain,
                operation(
                    ConvolutionInput::new(&weights)
                        .with_order(15.0, Some(1.0))
                        .with_target(Some(14.0), Some(0.0))
                        .with_divisor(1.0)
                        .with_preserve_alpha(true)
                        .with_edge_mode(edge),
                    domain,
                    FilterColorSpace::Srgb,
                ),
            ),
        );
        // The only nonzero tap addresses x-14: [-14,-13,-12]. These literal
        // expected cells cross several wrap/mirror periods with negative indices.
        for (x, red) in reds.into_iter().enumerate() {
            assert_pixel(&output, x as u32, 0, [red, red, red, 255], 1);
        }
        let singleton = bounds(1, 1);
        let op = prepare_convolution(
            ConvolutionInput::new(&weights)
                .with_order(15.0, Some(1.0))
                .with_target(Some(14.0), Some(0.0))
                .with_divisor(1.0)
                .with_preserve_alpha(true)
                .with_edge_mode(edge),
            FilterDomains::try_new(singleton, domain, singleton).unwrap(),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let singleton_output = render(
            &mut renderer,
            &mut surface,
            &image_scene(image(1, 1, vec![128, 128, 128, 255]), singleton, op),
        );
        let red = if edge == FilterEdgeMode::None { 0 } else { 128 };
        assert_pixel(&singleton_output, 0, 0, [red, red, red, 255], 1);
        assert_pixel(&singleton_output, 1, 0, [0; 4], 1);
        let op = prepare_convolution(
            ConvolutionInput::new(&[1.0])
                .with_order(1.0, None)
                .with_bias(0.5)
                .with_edge_mode(edge),
            FilterDomains::try_new(Rect::new(0.0, 0.0, 0.0, 1.0), domain, domain).unwrap(),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let mut empty = Scene::new();
        empty.layer(Layer::new().with_convolution(op), |_| {});
        let output = render(&mut renderer, &mut surface, &empty);
        for x in 0..3 {
            assert_pixel(&output, x, 0, [128, 128, 128, 128], 2);
        }
    }
}

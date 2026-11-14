//! Hand-derived image-domain and two-stage lattice witnesses.
use super::convolution::*;
use crate::{
    Color, ConvolutionEdgeMode, ConvolutionInput, FilterColorSpace, FilterDomains, Layer,
    Parameters, Rect, RenderAccountingContext, Scene, Transform, prepare_convolution,
};

#[test]
fn overflowing_capture_resampling_refuses_before_publication() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 2);
    let domain = bounds(2, 1);
    let baseline =
        pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let convolution = operation(
        ConvolutionInput::new(&[1.0])
            .with_order(1.0, None)
            .with_unit_length(1.0e38, None),
        domain,
        FilterColorSpace::Srgb,
    );
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_convolution(convolution)
            .with_transform(Transform::try_new([16.0, 0.0, 0.0, 16.0, 0.0, 0.0]).unwrap()),
        |child| {
            child.image(
                image(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]),
                domain,
                crate::ImageFit::Stretch,
            );
        },
    );
    // The sole input-grid center is finite (5e37), but its actual capture
    // coordinate multiplies by density16 and overflows f32 before floor/i32.
    // This tests typed admission, not a claim of wrong pixels on this GPU.
    assert!(((1.0e38_f32 * 0.5) * 16.0).is_infinite());
    let error = pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
        .expect_err("unrepresentable capture arithmetic must refuse before submission");
    assert!(matches!(
        error.convolution_diagnostic(),
        Some(crate::ConvolutionPreparationError::Numeric {
            phase: crate::ConvolutionNumericPhase::GridMapping,
            ..
        })
    ));
    assert_eq!(renderer.stats(), baseline);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
}

#[test]
fn explicit_kernel_grid_materializes_finite_input_before_edge_extension() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let domain = bounds(2, 1);
    let source = image(2, 1, vec![255, 255, 255, 255, 255, 0, 255, 0]);
    for (edge, alpha) in [
        (ConvolutionEdgeMode::Duplicate, 128),
        (ConvolutionEdgeMode::Wrap, 128),
        (ConvolutionEdgeMode::Mirror, 128),
        (ConvolutionEdgeMode::None, 96),
    ] {
        let op = operation(
            ConvolutionInput::new(&[1.0])
                .with_order(1.0, None)
                .with_unit_length(2.0, Some(1.0))
                .with_preserve_alpha(true)
                .with_edge_mode(edge),
            domain,
            FilterColorSpace::Srgb,
        );
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(source.clone(), domain, op),
        );
        // I has one center at1: bilinear([alpha1,alpha0])=.5. Duplicate/wrap/mirror
        // extend I's .5 alpha. None reconstructs .75*.5=.375 at both output centers.
        for x in 0..2 {
            assert_pixel(&output, x, 0, [255, 255, 255, alpha], 2);
        }
    }
}

#[test]
fn noninteger_anisotropic_units_execute_two_resampling_stages() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let domain = bounds(3, 1);
    let source = image(3, 1, vec![255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0]);
    let op = operation(
        ConvolutionInput::new(&[1.0])
            .with_order(1.0, None)
            .with_unit_length(1.5, Some(0.75))
            .with_preserve_alpha(true),
        domain,
        FilterColorSpace::Srgb,
    );
    let output = render(
        &mut renderer,
        &mut surface,
        &image_scene(source, domain, op),
    );
    // Source kernel centers .75 and2.25 yield alpha .75 and0. Output centers
    // .5,1.5,2.5 reconstruct alpha .75,.375,0. Direct fractional identity taps
    // would leave the original [1,0,0], making this an independent discriminator.
    assert_pixel(&output, 0, 0, [255, 255, 255, 191], 2);
    assert_pixel(&output, 1, 0, [255, 255, 255, 96], 2);
    assert_pixel(&output, 2, 0, [0, 0, 0, 0], 1);
}

#[test]
fn source_filter_and_result_domains_clip_different_image_cells() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let source = image(3, 1, vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255]);
    let full = bounds(3, 1);
    let middle = Rect::new(1.0, 0.0, 1.0, 1.0);
    let request = ConvolutionInput::new(&[0.0, 0.0, 1.0])
        .with_order(3.0, Some(1.0))
        .with_divisor(1.0)
        .with_edge_mode(ConvolutionEdgeMode::None);
    for (filter, expected) in [
        (full, [255, 0, 0, 255]),
        (Rect::new(1.0, 0.0, 2.0, 1.0), [0, 0, 0, 0]),
    ] {
        let op = prepare_convolution(
            request,
            FilterDomains::try_new(full, filter, middle).unwrap(),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(source.clone(), full, op),
        );
        // Centerx1 samples sourcex0: P must not remove this contributing source;
        // F removes it before convolution. Allocation pixels are never edge extent.
        assert_pixel(&output, 1, 0, expected, 1);
        assert_pixel(&output, 0, 0, [0; 4], 1);
        assert_pixel(&output, 2, 0, [0; 4], 1);
    }
    let op = prepare_convolution(
        ConvolutionInput::new(&[f64::NAN])
            .with_bias(f64::NAN)
            .with_unit_length(f64::MIN_POSITIVE, None),
        FilterDomains::try_new(full, full, middle).unwrap(),
        FilterColorSpace::LinearSrgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    let output = render(&mut renderer, &mut surface, &image_scene(source, full, op));
    assert_pixel(&output, 1, 0, [0, 255, 0, 255], 1);
    assert_pixel(&output, 0, 0, [0; 4], 1);
}

#[test]
fn fractional_source_cells_receive_coverage_once_before_sampling() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let full = bounds(2, 1);
    let op = prepare_convolution(
        ConvolutionInput::new(&[1.0]).with_order(1.0, None),
        FilterDomains::try_new(Rect::new(0.0, 0.0, 1.5, 1.0), full, full).unwrap(),
        FilterColorSpace::Srgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    let output = render(
        &mut renderer,
        &mut surface,
        &image_scene(image(2, 1, vec![255; 8]), full, op),
    );
    assert_pixel(&output, 0, 0, [255; 4], 1);
    assert_pixel(&output, 1, 0, [255, 255, 255, 128], 2);
}

#[test]
fn anisotropic_convolution_keeps_full_outer_affine_placement() {
    let (mut renderer, mut surface) = renderer_and_surface(16, 16);
    let domain = bounds(4, 4);
    let op = operation(
        ConvolutionInput::new(&[1.0])
            .with_order(1.0, None)
            .with_unit_length(0.75, Some(1.25)),
        domain,
        FilterColorSpace::Srgb,
    );
    let affine = Transform::try_new([2.0, 1.0, -0.5, 1.5, 4.0, 3.0]).unwrap();
    let mut scene = Scene::new();
    scene.layer(
        Layer::new().with_convolution(op).with_transform(affine),
        |child| {
            child.fill(domain, Color::try_rgba(0.25, 0.5, 0.75, 1.0).unwrap());
        },
    );
    let output = render(&mut renderer, &mut surface, &scene);
    // A^-1 maps surface center(8.5,8.5) to(19/7,13/7), safely inside the
    // local4x4 result. Center(1.5,1.5) lies outside its transformed polygon.
    assert_pixel(&output, 8, 8, [64, 128, 191, 255], 3);
    assert_pixel(&output, 1, 1, [0; 4], 1);
    // Center(11.5,4.5) maps to(24/7,-9/7), outside even though scalar-density
    // placement would put it inside a large axis-aligned bounding image.
    assert_pixel(&output, 11, 4, [0; 4], 1);
}

#[test]
fn unequal_kernel_axes_under_affine_follow_local_sampling_equations() {
    use crate::{ImageFit, ImageQuality};
    let (mut renderer, mut surface) = renderer_and_surface(24, 24);
    let domain = bounds(8, 8);
    let op = operation(
        ConvolutionInput::new(&[1.0, 0.0, 0.0, 0.0])
            .with_order(2.0, Some(2.0))
            .with_target(Some(0.0), Some(0.0))
            .with_divisor(1.0)
            .with_unit_length(0.75, Some(1.25)),
        domain,
        FilterColorSpace::Srgb,
    );
    let pixels = (0..8)
        .flat_map(|y| {
            (0..8).flat_map(move |x| {
                let red = (((f64::from(x) + 0.5 + 2.0 * (f64::from(y) + 0.5)) / 20.0).min(1.0)
                    * 255.0)
                    .round() as u8;
                [red, 0, 0, 255]
            })
        })
        .collect();
    // Author the source image at its texel centers explicitly. Medium image
    // reconstruction preserves this interior linear field; vector-gradient
    // sampling has its own existing native raster phase and is not this oracle.
    let source = image(8, 8, pixels).quality(ImageQuality::Medium);
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_convolution(op)
            .with_transform(Transform::try_new([2.0, 1.0, -0.5, 1.5, 4.0, 3.0]).unwrap()),
        |child| {
            child.image(source, domain, ImageFit::Stretch);
        },
    );
    let output = render(&mut renderer, &mut surface, &scene);
    // Inverting the complete A gives p=(33/7,13/7) at surface(12.5,10.5).
    // Rotated first tap samples p+(.75,1.25). The linear field is(x+2y)/20,
    // so the red result is327/560, or148.9018 byte levels. Every resampling
    // stage preserves this interior linear field apart from source quantization.
    assert_pixel(&output, 12, 10, [149, 0, 0, 255], 3);
}

#[test]
fn vector_gradient_capture_phase_is_preserved_through_affine_convolution() {
    use crate::{Gradient, GradientStop, Paint, Point};
    let (mut renderer, mut surface) = renderer_and_surface(24, 24);
    let domain = bounds(8, 8);
    let gradient = Gradient::try_linear(
        Point::new(0.0, 0.0),
        Point::new(4.0, 8.0),
        vec![
            GradientStop::try_new(0.0, Color::BLACK).unwrap(),
            GradientStop::try_new(1.0, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()).unwrap(),
        ],
    )
    .unwrap();
    let affine = Transform::try_new([2.0, 1.0, -0.5, 1.5, 4.0, 3.0]).unwrap();
    let mut baseline = Scene::new();
    baseline.layer(Layer::new().with_transform(affine), |child| {
        child.fill(domain, Paint::gradient(gradient.clone()));
    });
    pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default())).unwrap();
    let direct = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    // Pinned vello_shaders0.9.0 fine.wgsl1076,1227,1229 evaluates linear
    // fields at integer raster positions. Render's shaders.rs replacement
    // preserves my_d. A^-1(12,10)=(31/7,12/7), hence red55/140*255≈100.18.
    assert_pixel(&direct, 12, 10, [100, 0, 0, 255], 3);
    let op = operation(
        ConvolutionInput::new(&[1.0, 0.0, 0.0, 0.0])
            .with_order(2.0, Some(2.0))
            .with_target(Some(0.0), Some(0.0))
            .with_divisor(1.0)
            .with_unit_length(0.75, Some(1.25)),
        domain,
        FilterColorSpace::Srgb,
    );
    let mut scene = Scene::new();
    scene.layer(
        Layer::new().with_transform(affine).with_convolution(op),
        |child| {
            child.fill(domain, Paint::gradient(gradient));
        },
    );
    let output = render(&mut renderer, &mut surface, &scene);
    // Kernel/capture mappings still refer to texel centers. This producer's
    // field values at centerq were evaluated at q-(.5/rho,.5/rho). Largest
    // singular value rho follows independently from eigenvalues of A^T A.
    let rho = ((7.5_f64 + 7.25_f64.sqrt()) / 2.0).sqrt();
    let expected = ((327.0 / 560.0 - 3.0 / (40.0 * rho)) * 255.0).round() as u8;
    assert_pixel(&output, 12, 10, [expected, 0, 0, 255], 3);
}

#[test]
fn empty_result_region_publishes_transparent_frame() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let full = bounds(2, 1);
    let op = prepare_convolution(
        ConvolutionInput::new(&[1.0])
            .with_order(1.0, None)
            .with_bias(0.5),
        FilterDomains::try_new(full, full, Rect::new(0.0, 0.0, 0.0, 1.0)).unwrap(),
        FilterColorSpace::Srgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    let scene = image_scene(image(2, 1, vec![255; 8]), full, op);
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for x in 0..2 {
        assert_pixel(&output, x, 0, [0; 4], 1);
    }
}

#[test]
fn same_layer_backdrop_outside_source_does_not_shift_explicit_kernel_origin() {
    use crate::{BackdropCaptureBounds, BackdropFilterInput, FilterBlur, FilterList, FilterOp};
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
    let finite = Rect::new(1.0, 0.0, 2.0, 1.0);
    let op = operation(
        ConvolutionInput::new(&[1.0])
            .with_order(1.0, None)
            .with_unit_length(2.0, Some(1.0)),
        finite,
        FilterColorSpace::Srgb,
    );
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
    scene.layer(
        Layer::new()
            .with_convolution(op)
            .try_backdrop_filter(backdrop.clone())
            .unwrap(),
        |_| {},
    );
    let output = render(&mut renderer, &mut surface, &scene);
    // Captured D has red-alpha[1,0], origin1. Its dx2 input grid has one
    // center at2, samplepremul(.5,0,0,.5). Duplicate extension keeps that
    // alpha.5 at transparentx2. Using backdrop's larger origin0 instead
    // creates two kernel cells and incorrectly yields alpha.25 atx2.
    assert_pixel(&output, 2, 0, [255, 0, 0, 128], 2);
    assert_pixel(&output, 1, 0, [255, 0, 0, 255], 1);
    assert_pixel(&output, 0, 0, [0; 4], 1);
    let mismatch = prepare_convolution(
        ConvolutionInput::new(&[f64::NAN])
            .with_bias(f64::NAN)
            .with_unit_length(f64::MIN_POSITIVE, None),
        FilterDomains::try_new(finite, finite, finite).unwrap(),
        FilterColorSpace::LinearSrgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    let mut passthrough = Scene::new();
    passthrough.fill(
        Rect::new(1.0, 0.0, 1.0, 1.0),
        Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
    );
    passthrough.layer(
        Layer::new()
            .with_convolution(mismatch)
            .try_backdrop_filter(backdrop)
            .unwrap(),
        |_| {},
    );
    let output = render(&mut renderer, &mut surface, &passthrough);
    assert_pixel(&output, 1, 0, [255, 0, 0, 255], 1);
    assert_pixel(&output, 2, 0, [0; 4], 1);
    let report = renderer.last_attempt().unwrap();
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(crate::RenderWork::ScheduledConvolutionTaps), 0);
    assert_eq!(usage.work(crate::RenderWork::KernelSampleVisits), 0);
}

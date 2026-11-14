use super::convolution::*;
use crate::{
    Color, ConvolutionDomains, ConvolutionEdgeMode, ConvolutionInput, FilterColorSpace, Layer,
    Rect, RenderAccountingContext, Scene, prepare_convolution,
};

#[test]
fn zero_axis_source_does_not_qualify_the_unused_large_axis() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let result = bounds(2, 1);
    for extent in [1.0e12, 1.0e300] {
        let filter = Rect::new(0.0, 0.0, extent, extent);
        for source in [
            Rect::new(0.0, 0.0, 0.0, extent),
            Rect::new(0.0, 0.0, extent, 0.0),
        ] {
            for edge in [
                ConvolutionEdgeMode::Duplicate,
                ConvolutionEdgeMode::Wrap,
                ConvolutionEdgeMode::Mirror,
                ConvolutionEdgeMode::None,
            ] {
                let op = prepare_convolution(
                    ConvolutionInput::new(&[1.0])
                        .with_order(1.0, None)
                        .with_bias(0.5)
                        .with_edge_mode(edge),
                    ConvolutionDomains::try_new(source, filter, result).unwrap(),
                    FilterColorSpace::Srgb,
                    &mut RenderAccountingContext::default(),
                )
                .unwrap();
                let mut scene = Scene::new();
                scene.layer(Layer::new().with_convolution(op), |child| {
                    child.fill(result, Color::BLACK);
                });
                let output = render(&mut renderer, &mut surface, &scene);
                for x in 0..2 {
                    assert_pixel(&output, x, 0, [128, 128, 128, 128], 2);
                }
            }
        }
    }
}

#[test]
fn empty_source_excludes_backdrop_before_bias_generates_the_result() {
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, FilterList, FilterOp, UnitFilterAmount,
    };
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let result = bounds(2, 1);
    let op = prepare_convolution(
        ConvolutionInput::new(&[1.0])
            .with_order(1.0, None)
            .with_bias(0.5),
        ConvolutionDomains::try_new(
            Rect::new(0.0, 0.0, 0.0, 1.0e12),
            Rect::new(0.0, 0.0, 1.0e12, 1.0e12),
            result,
        )
        .unwrap(),
        FilterColorSpace::Srgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    let backdrop = BackdropFilterInput::try_new(
        FilterList::try_ops(vec![FilterOp::invert(
            UnitFilterAmount::try_new(1.0).unwrap(),
        )])
        .unwrap(),
        BackdropCaptureBounds::try_new(result).unwrap(),
        None,
    )
    .unwrap();
    let layer = Layer::new()
        .with_convolution(op)
        .try_backdrop_filter(backdrop)
        .unwrap();
    let mut scene = Scene::new();
    scene.fill(result, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
    scene.layer(layer, |child| {
        child.fill(result, Color::BLACK);
    });
    let output = render(&mut renderer, &mut surface, &scene);
    // Empty input removes both children and captured backdrop before sampling.
    // Bias creates premul(.25,.25,.25,.5), over opaque green =>(.25,.75,.25,1).
    for x in 0..2 {
        assert_pixel(&output, x, 0, [64, 191, 64, 255], 2);
    }
}

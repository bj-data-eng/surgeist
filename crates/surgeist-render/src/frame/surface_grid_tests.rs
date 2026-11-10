use super::*;
use crate::{
    Capabilities, Color, Image, ImageFit, Layer, Paint, ResolvedLayerAlphaMask, Scene, Size,
};

fn white_image() -> Image {
    Image::from_rgba(Size::new(1.0, 1.0), vec![255; 4]).unwrap()
}

fn opaque_mask(bounds: Rect) -> ResolvedLayerAlphaMask {
    ResolvedLayerAlphaMask::try_new(white_image(), bounds).unwrap()
}

fn plan(scene: &Scene, dpr: f64) -> crate::accounting::OwnedBox<GpuRenderGraph> {
    let account = crate::accounting::Account::for_test();
    let commands = scene.normalize(Capabilities::CURRENT, &account).unwrap();
    let context = FrameContext::try_new(
        Size::new(32.0, 32.0),
        dpr,
        Antialiasing::Area,
        Color::TRANSPARENT,
    )
    .unwrap();
    let super::super::FramePlan::GpuGraph(graph) = commands.plan_for(context, &account).unwrap()
    else {
        panic!("the resolved mask requires graph execution");
    };
    graph
}

#[test]
fn sensitive_image_selects_surface_density_for_all_foreground_siblings() {
    for hint in [ImageRendering::Pixelated, ImageRendering::CrispEdges] {
        let mut scene = Scene::new();
        let bounds = Rect::new(0.0, 0.0, 2.0, 3.0);
        scene.layer(
            Layer::new()
                .with_transform(Transform::scale(2.5, 2.0).unwrap())
                .with_resolved_alpha_mask(opaque_mask(bounds)),
            |scene| {
                scene.image(white_image().rendering(hint), bounds, ImageFit::Fill);
            },
        );
        scene.layer(
            Layer::new()
                .with_transform(Transform::scale(3.0, 1.5).unwrap())
                .with_resolved_alpha_mask(opaque_mask(bounds)),
            |scene| {
                scene.fill(bounds, Color::BLACK);
            },
        );
        let graph = plan(&scene, 1.5);
        assert_eq!(graph.vello_spans.len(), 2);
        for span in &graph.vello_spans {
            assert_eq!(span.source_space, CompositeSourceSpace::SurfaceGrid);
            assert_eq!(span.parent_to_surface, Transform::identity());
        }
        for resource in &graph.resources {
            if resource.descriptor.role != SemanticResourceRole::ImportedImage {
                assert_eq!(
                    resource.descriptor.texel_center_mapping.raster_scale.get(),
                    1.5
                );
            }
        }
        let world_transforms = graph
            .vello_spans
            .iter()
            .map(|span| span.capture_transform)
            .collect::<Vec<_>>();
        assert_eq!(
            world_transforms,
            [
                Transform::scale(2.5, 2.0).unwrap(),
                Transform::scale(3.0, 1.5).unwrap()
            ]
        );
    }
}

#[test]
fn surface_capture_maps_signed_fractional_source_bounds_once_before_rounding() {
    let bounds = Rect::new(-1.0, 0.5, 2.0, 3.0);
    let world = Transform::try_new([2.5, 0.0, 0.0, 2.0, -0.25, 1.25]).unwrap();
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_transform(world)
            .with_resolved_alpha_mask(opaque_mask(bounds)),
        |scene| {
            scene.fill(
                bounds,
                Paint::image(white_image().rendering(ImageRendering::Pixelated)),
            );
        },
    );
    let graph = plan(&scene, 1.5);
    for resource in graph.resources.iter().filter(|resource| {
        matches!(
            resource.descriptor.role,
            SemanticResourceRole::CaptureWorkingImage | SemanticResourceRole::IsolationWorkingImage
        )
    }) {
        let spatial = resource.descriptor;
        assert_eq!(
            spatial.logical_bounds.rect(),
            Rect::new(-2.75, 2.25, 5.0, 6.0)
        );
        assert_eq!(spatial.device_origin, SignedDeviceOrigin::new(-5, 3));
        assert_eq!(
            (spatial.device_extent.width, spatial.device_extent.height),
            (9, 10)
        );
        assert_eq!(spatial.texel_center_mapping.raster_scale.get(), 1.5);
    }
}

#[test]
fn surface_foreground_keeps_nested_world_clips_and_independent_mask_inverse() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 3.0);
    let outer = Transform::scale(2.0, 3.0).unwrap();
    let inner = Transform::scale(2.5, 2.0).unwrap();
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_transform(outer)
            .try_clip(bounds.into())
            .unwrap(),
        |scene| {
            scene.layer(
                Layer::new()
                    .with_transform(inner)
                    .with_resolved_alpha_mask(opaque_mask(bounds)),
                |scene| {
                    scene.fill(
                        bounds,
                        Paint::image(white_image().rendering(ImageRendering::Pixelated)),
                    );
                },
            );
        },
    );
    let graph = plan(&scene, 1.0);
    assert_eq!(
        graph.vello_spans[0].capture_transform,
        Transform::scale(5.0, 6.0).unwrap()
    );
    let masked = graph
        .composites
        .iter()
        .find_map(|composite| match &composite.kind {
            SemanticCompositeKind::Layer {
                destination_to_layer_local,
                source_space,
                alpha_mask: Some(_),
                ..
            } => Some((destination_to_layer_local, source_space)),
            _ => None,
        })
        .unwrap();
    assert_eq!(*masked.1, CompositeSourceSpace::SurfaceGrid);
    let inverse = kurbo::Affine::from(masked.0.affine);
    let local = inverse * kurbo::Point::new(5.0, 6.0);
    assert!((local.x - 1.0).abs() < 1e-12 && (local.y - 1.0).abs() < 1e-12);
    assert!(
        graph
            .clip_coverages
            .iter()
            .flat_map(|coverage| coverage.elements.iter())
            .any(|element| element.transform == outer)
    );
}

#[test]
fn ordinary_image_keeps_existing_local_capture_density_and_coordinates() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 3.0);
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_transform(Transform::scale(2.5, 2.0).unwrap())
            .with_resolved_alpha_mask(opaque_mask(bounds)),
        |scene| {
            scene.image(
                white_image().rendering(ImageRendering::Smooth),
                bounds,
                ImageFit::Fill,
            );
        },
    );
    let graph = plan(&scene, 1.5);
    assert_eq!(
        graph.vello_spans[0].source_space,
        CompositeSourceSpace::LayerLocal
    );
    let isolation = graph
        .resources
        .iter()
        .find(|resource| resource.descriptor.role == SemanticResourceRole::IsolationWorkingImage)
        .unwrap();
    assert_eq!(isolation.descriptor.logical_bounds.rect(), bounds);
    assert_eq!(
        isolation.descriptor.texel_center_mapping.raster_scale.get(),
        3.75
    );
}

#[test]
fn sensitive_resolved_mask_selects_surface_grid_for_an_ordinary_foreground() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 3.0);
    let mask = ResolvedLayerAlphaMask::try_new(
        white_image().rendering(ImageRendering::CrispEdges),
        bounds,
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_transform(Transform::scale(2.5, 2.0).unwrap())
            .with_resolved_alpha_mask(mask),
        |scene| {
            scene.fill(bounds, Color::BLACK);
        },
    );
    let graph = plan(&scene, 1.0);
    assert_eq!(
        graph.vello_spans[0].source_space,
        CompositeSourceSpace::SurfaceGrid
    );
    let capture = graph
        .resources
        .iter()
        .find(|resource| resource.descriptor.role == SemanticResourceRole::CaptureWorkingImage)
        .unwrap();
    assert_eq!(
        capture.descriptor.texel_center_mapping.raster_scale.get(),
        1.0
    );
}

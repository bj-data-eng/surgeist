use super::{
    command::{
        LayerIsolation, NormalizedLayer, RenderClip, RenderClipGeometry, RenderCommand,
        RenderCommands, RenderPaint, RenderShadow, RenderShape, RenderStroke, RenderStrokeShape,
        ShadowShape, kurbo_rounded_rect,
    },
    geometry::{expand_rect, offset_radii},
    paint::PaintKind,
    vello_engine::scene::{RoundedRectBlur, SceneDraws, VelloScene},
    *,
};
use crate::accounting::{Account, AccountedVec, DisposalCompletion, Hold, RenderPhase, RenderWork};
use crate::vello_engine::geometry::{action as coordinate_action, affine_product};
use crate::vello_engine::gradient_endpoints::{
    GradientProgram, GradientSampling, PadEndpoints, SceneBrush,
};

mod gradient_field;
mod hint_program;
mod ordinary_gradient;
mod repeating_gradient;

pub(crate) fn encode_vello_scene(
    commands: &RenderCommands,
    scale: f64,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<VelloScene> {
    encode_vello_scene_with_initial_transform(
        commands,
        Transform::scale(scale, scale)?,
        target_extent,
        account,
    )
}

pub(crate) fn encode_vello_scene_with_initial_transform(
    commands: &RenderCommands,
    initial_transform: Transform,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<VelloScene> {
    let account = account.at(RenderPhase::Encode);
    account.phase(RenderPhase::Encode, || {
        let mut encoded = VelloScene::default();
        encode_vello_commands(
            &commands.commands,
            &mut encoded,
            kurbo::Affine::from(initial_transform),
            target_extent,
            &account,
        )?;
        Ok(encoded)
    })
}

pub(crate) fn encode_vello_clip_coverage_scene(
    elements: &[(RenderClip, Transform)],
    initial_transform: Transform,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<VelloScene> {
    let account = account.at(RenderPhase::Encode);
    account.phase(RenderPhase::Encode, || {
        if elements.is_empty() {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "Vello clip coverage requires at least one ordered RenderClip",
            ));
        }
        if target_extent.width() == 0 || target_extent.height() == 0 {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "Vello clip coverage requires a positive bounded extent",
            ));
        }

        let mut scene = VelloScene::default();
        for (clip, transform) in elements {
            push_vello_clip(
                &mut scene,
                clip,
                clip_coverage_transform(clip, *transform, initial_transform, &account)?,
                &account,
            )?;
        }
        scene.fill(
            peniko::Fill::NonZero,
            kurbo::Affine::IDENTITY,
            peniko::Color::WHITE,
            None,
            &kurbo::Rect::new(
                0.0,
                0.0,
                f64::from(target_extent.width()),
                f64::from(target_extent.height()),
            ),
            &account,
        )?;
        for _ in elements {
            scene.pop_layer(&account)?;
        }
        Ok(scene)
    })
}

fn clip_coverage_transform(
    clip: &RenderClip,
    transform: Transform,
    initial_transform: Transform,
    account: &Account,
) -> Result<Transform> {
    clip.coordinate_space()
        .map_or(Transform::IDENTITY, CoordinateSpaceTag::transform)
        .then_in(transform, account)?
        .then_in(initial_transform, account)
}

fn exact_fit_clip_circle(
    rect: Rect,
    radii: Radii,
    account: &Account,
) -> Result<Option<kurbo::Circle>> {
    let side = coordinate_action(account, || rect.width())?;
    let height = coordinate_action(account, || rect.height())?;
    if coordinate_action(account, || side != height)? {
        return Ok(None);
    }
    let radius = coordinate_action(account, || side * 0.5)?;
    if coordinate_action(account, || radius <= 0.0)? {
        return Ok(None);
    }

    // The rounded-rectangle encoder clamps each corner to half the shorter
    // side. Test that effective radius using the stored logical dimensions.
    let corners: [fn(Radii) -> f64; 4] = [
        Radii::top_left,
        Radii::top_right,
        Radii::bottom_right,
        Radii::bottom_left,
    ];
    for read_radius in corners {
        let corner = coordinate_action(account, || read_radius(radii))?;
        if coordinate_action(account, || corner < radius)? {
            return Ok(None);
        }
    }
    // The four corner arcs share a center. A circle has no straight joins
    // whose generated endpoints can differ only by trigonometric roundoff.
    let origin_x = coordinate_action(account, || rect.x())?;
    let origin_y = coordinate_action(account, || rect.y())?;
    let x = coordinate_action(account, || origin_x + radius)?;
    let y = coordinate_action(account, || origin_y + radius)?;
    Ok(Some(kurbo::Circle::new((x, y), radius)))
}

fn push_vello_clip(
    scene: &mut VelloScene,
    clip: &RenderClip,
    transform: Transform,
    account: &Account,
) -> Result<()> {
    let transform = kurbo::Affine::from(transform);
    match clip.geometry() {
        RenderClipGeometry::Rect(rect) => scene.push_clip_layer(
            peniko::Fill::NonZero,
            transform,
            &kurbo::Rect::from(*rect),
            account,
        ),
        RenderClipGeometry::RoundedRect { rect, radii } => {
            if let Some(circle) = exact_fit_clip_circle(*rect, *radii, account)? {
                scene.push_clip_layer(peniko::Fill::NonZero, transform, &circle, account)
            } else {
                scene.push_clip_layer(
                    peniko::Fill::NonZero,
                    transform,
                    &kurbo_rounded_rect(*rect, *radii),
                    account,
                )
            }
        }
        RenderClipGeometry::Circle { center, radius } => scene.push_clip_layer(
            peniko::Fill::NonZero,
            transform,
            &kurbo::Circle::new((center.x(), center.y()), *radius),
            account,
        ),
        RenderClipGeometry::Ellipse { center, radii } => scene.push_clip_layer(
            peniko::Fill::NonZero,
            transform,
            &kurbo::Ellipse::new(
                (center.x(), center.y()),
                (radii.width(), radii.height()),
                0.0,
            ),
            account,
        ),
        RenderClipGeometry::Path { path, fill_rule } => scene.push_clip_layer(
            vello_fill_rule(*fill_rule),
            transform,
            &*path.kurbo_shape_in(account)?,
            account,
        ),
    }
}

fn encode_vello_commands(
    commands: &[RenderCommand],
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<()> {
    for command in commands {
        account.work(RenderWork::CommandVisits, 1)?;
        match command {
            RenderCommand::Fill { shape, paint } => {
                encode_fill(scene, transform, shape, paint, target_extent, account)?
            }
            RenderCommand::Stroke {
                shape,
                stroke,
                paint,
            } => encode_stroke(
                scene,
                transform,
                shape,
                stroke,
                paint,
                target_extent,
                account,
            )?,
            RenderCommand::Shadow { shape, shadow } => {
                encode_shadow(scene, transform, shape, shadow, account)?
            }
            RenderCommand::Image { image, rect, fit } => {
                encode_image(scene, transform, image, *rect, *fit, account)?
            }
            RenderCommand::ImageRegion {
                image,
                rect,
                region,
            } => {
                let local = image_transform(region.local().size(), *rect, ImageFit::Fill, account)?;
                let source_to_device = affine_product(transform, local, account)?;
                scene.draw_image_region(image, *region, source_to_device, account)?;
            }
            RenderCommand::PreparedGlyphRun {
                run: outline,
                transform: run_transform,
                paint,
            } => {
                let transform = crate::vello_engine::geometry::affine_product(
                    transform,
                    kurbo::Affine::from(*run_transform),
                    account,
                )?;
                crate::text::qualify_receiving_placement(transform, outline, account)?;
                for operation in outline.paints() {
                    account.work(RenderWork::PaintVisits, 1)?;
                    match operation {
                        crate::text::GlyphPaint::Fill {
                            elements,
                            color,
                            clip,
                        } => {
                            let brush = match color {
                                Some(color) => FrameBrush::solid((*color).into()),
                                None => glyph_paint_brush(
                                    paint.fill(),
                                    transform,
                                    target_extent,
                                    account,
                                )?,
                            };
                            if let Some(clip) = clip {
                                scene.push_clip_layer(
                                    peniko::Fill::NonZero,
                                    transform,
                                    &kurbo::Rect::from(*clip),
                                    account,
                                )?;
                            }
                            let elements = &outline.path().elements()[elements.clone()];
                            let sampled_image = if color.is_none() {
                                account.work(RenderWork::PaintVisits, 1)?;
                                if let PaintKind::Image(image) = paint.fill().kind() {
                                    Some(image)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            if let Some(image) = sampled_image {
                                with_sampled_image(
                                    scene,
                                    image,
                                    transform,
                                    |scene, brush, brush_transform| {
                                        scene.fill_elements(
                                            transform,
                                            brush,
                                            brush_transform,
                                            elements,
                                            account,
                                        )?;
                                        Ok(())
                                    },
                                    account,
                                )?;
                            } else {
                                scene.fill_elements(
                                    transform,
                                    brush.scene_brush(target_extent),
                                    brush.transform(),
                                    elements,
                                    account,
                                )?;
                            }
                            if clip.is_some() {
                                scene.pop_layer(account)?;
                            }
                        }
                        crate::text::GlyphPaint::Bitmap {
                            image,
                            rect,
                            brush_transform,
                        } => encode_prepared_bitmap(
                            scene,
                            transform,
                            image,
                            *rect,
                            *brush_transform,
                            account,
                        )?,
                    }
                }
            }
            RenderCommand::TextRun {
                font,
                size,
                transform: run_transform,
                paint,
                glyphs,
                ..
            } => encode_text_run(
                scene,
                transform,
                TextRunSource {
                    font,
                    size: *size,
                    transform: *run_transform,
                    paint,
                    glyphs,
                },
                target_extent,
                account,
            )?,
            RenderCommand::Layer { layer, children } => {
                let layer_transform = affine_product(transform, layer.transform.into(), account)?;
                if layer.isolation != LayerIsolation::None {
                    encode_layer_start(scene, layer, layer_transform, account)?;
                    encode_vello_commands(
                        children,
                        scene,
                        layer_transform,
                        target_extent,
                        account,
                    )?;
                    scene.pop_layer(account)?;
                } else {
                    encode_vello_commands(
                        children,
                        scene,
                        layer_transform,
                        target_extent,
                        account,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn encode_fill(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    shape: &RenderShape,
    paint: &RenderPaint,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<()> {
    account.work(RenderWork::PaintVisits, 1)?;
    if let RenderPaint::Image(image) = paint {
        return with_sampled_image(
            scene,
            image,
            transform,
            |scene, brush, brush_transform| {
                encode_solid_fill(scene, transform, shape, brush, brush_transform, account)?;
                Ok(())
            },
            account,
        );
    }
    let brush = render_paint_brush(paint, transform, target_extent, account)?;
    encode_solid_fill(
        &mut scene.draws(),
        transform,
        shape,
        brush.scene_brush(target_extent),
        brush.transform(),
        account,
    )?;
    Ok(())
}

fn encode_solid_fill<'a>(
    scene: &mut SceneDraws<'_>,
    transform: kurbo::Affine,
    shape: &RenderShape,
    brush: impl Into<SceneBrush<'a>>,
    brush_transform: Option<kurbo::Affine>,
    account: &Account,
) -> Result<()> {
    let brush = brush.into();
    match shape {
        RenderShape::Rect(rect) => scene.fill(
            peniko::Fill::NonZero,
            transform,
            brush,
            brush_transform,
            &kurbo::Rect::from(*rect),
            account,
        ),
        RenderShape::RoundedRect { rect, radii } => scene.fill(
            peniko::Fill::NonZero,
            transform,
            brush,
            brush_transform,
            &kurbo_rounded_rect(*rect, *radii),
            account,
        ),
        RenderShape::Circle { center, radius } => scene.fill(
            peniko::Fill::NonZero,
            transform,
            brush,
            brush_transform,
            &kurbo::Circle::new((center.x(), center.y()), *radius),
            account,
        ),
        RenderShape::Ellipse { center, radii } => scene.fill(
            peniko::Fill::NonZero,
            transform,
            brush,
            brush_transform,
            &kurbo::Ellipse::new(
                (center.x(), center.y()),
                (radii.width(), radii.height()),
                0.0,
            ),
            account,
        ),
        RenderShape::Path(path) => {
            let elements = path.kurbo_elements(account)?;
            scene.fill_elements(transform, brush, brush_transform, &elements, account)
        }
    }
}

fn encode_stroke(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    shape: &RenderStrokeShape,
    stroke: &RenderStroke,
    paint: &RenderPaint,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<()> {
    account.work(RenderWork::PaintVisits, 1)?;
    if let RenderPaint::Image(image) = paint {
        return with_sampled_image(
            scene,
            image,
            transform,
            |scene, brush, brush_transform| {
                encode_brush_stroke(
                    scene,
                    transform,
                    shape,
                    stroke,
                    brush,
                    brush_transform,
                    account,
                )?;
                Ok(())
            },
            account,
        );
    }
    let brush = render_paint_brush(paint, transform, target_extent, account)?;
    encode_brush_stroke(
        &mut scene.draws(),
        transform,
        shape,
        stroke,
        brush.scene_brush(target_extent),
        brush.transform(),
        account,
    )?;
    Ok(())
}

fn encode_brush_stroke<'a>(
    scene: &mut SceneDraws<'_>,
    transform: kurbo::Affine,
    shape: &RenderStrokeShape,
    stroke: &RenderStroke,
    brush: impl Into<SceneBrush<'a>>,
    brush_transform: Option<kurbo::Affine>,
    account: &Account,
) -> Result<()> {
    let brush = brush.into();
    let vello_stroke = vello_stroke(stroke, account)?;
    match shape {
        RenderStrokeShape::Rect(rect) => scene.stroke(
            &vello_stroke,
            transform,
            brush,
            brush_transform,
            rect,
            account,
        ),
        RenderStrokeShape::RoundedRect(rect) => scene.stroke(
            &vello_stroke,
            transform,
            brush,
            brush_transform,
            rect,
            account,
        ),
        RenderStrokeShape::Circle(circle) => scene.stroke(
            &vello_stroke,
            transform,
            brush,
            brush_transform,
            circle,
            account,
        ),
        RenderStrokeShape::Ellipse(ellipse) => scene.stroke(
            &vello_stroke,
            transform,
            brush,
            brush_transform,
            ellipse,
            account,
        ),
        RenderStrokeShape::Path(path) => scene.stroke(
            &vello_stroke,
            transform,
            brush,
            brush_transform,
            &**path,
            account,
        ),
    }
}

fn encode_shadow(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    shape: &ShadowShape,
    shadow: &RenderShadow,
    account: &Account,
) -> Result<()> {
    let color = peniko::Color::from(shadow.color);
    let std_dev = shadow.blur * 0.5;
    match shape {
        ShadowShape::Rect(rect) => {
            let rect = offset_rect(expand_rect(*rect, shadow.spread), shadow.offset);
            scene.draw_blurred_rounded_rect(
                transform,
                rect.into(),
                color,
                0.0,
                std_dev,
                account,
            )?;
        }
        ShadowShape::RoundedRect { rect, radii } => {
            let rect = offset_rect(expand_rect(*rect, shadow.spread), shadow.offset);
            if let Some(radius) = radii.uniform() {
                scene.draw_blurred_rounded_rect(
                    transform,
                    rect.into(),
                    color,
                    (radius + shadow.spread).max(0.0),
                    std_dev,
                    account,
                )?;
            } else {
                encode_non_uniform_rounded_shadow(
                    scene,
                    transform,
                    rect,
                    offset_radii(*radii, shadow.spread),
                    shadow.blur,
                    shadow.color,
                    account,
                )?;
            }
        }
        ShadowShape::Circle { center, radius } => {
            let radius = (radius + shadow.spread).max(0.0);
            let rect = Rect::new(
                center.x() - radius + shadow.offset.x(),
                center.y() - radius + shadow.offset.y(),
                radius * 2.0,
                radius * 2.0,
            );
            scene.draw_blurred_rounded_rect(
                transform,
                rect.into(),
                color,
                radius,
                std_dev,
                account,
            )?;
        }
    }
    Ok(())
}

fn encode_non_uniform_rounded_shadow(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    rect: Rect,
    radii: Radii,
    blur: f64,
    color: Color,
    account: &Account,
) -> Result<()> {
    if blur == 0.0 {
        encode_solid_fill(
            &mut scene.draws(),
            transform,
            &RenderShape::RoundedRect { rect, radii },
            peniko::Color::from(color),
            None,
            account,
        )?;
        return Ok(());
    }

    let std_dev = blur * 0.5;
    let kernel = 2.5 * std_dev;
    let support = kurbo::Rect::from(rect).inflate(kernel, kernel);
    let mid_x = rect.x() + rect.width() * 0.5;
    let mid_y = rect.y() + rect.height() * 0.5;
    let shadow_rect = kurbo::Rect::from(rect);
    let color = peniko::Color::from(color);

    let regions = [
        (
            kurbo::Rect::new(support.x0, support.y0, mid_x, mid_y),
            radii.top_left(),
        ),
        (
            kurbo::Rect::new(mid_x, support.y0, support.x1, mid_y),
            radii.top_right(),
        ),
        (
            kurbo::Rect::new(mid_x, mid_y, support.x1, support.y1),
            radii.bottom_right(),
        ),
        (
            kurbo::Rect::new(support.x0, mid_y, mid_x, support.y1),
            radii.bottom_left(),
        ),
    ];

    for (clip, radius) in regions {
        scene.draw_blurred_rounded_rect_in(
            &clip,
            transform,
            RoundedRectBlur {
                rect: shadow_rect,
                color,
                radius: radius.max(0.0),
                std_dev,
            },
            account,
        )?;
    }

    Ok(())
}

fn encode_prepared_bitmap(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    image: &Image,
    rect: Rect,
    brush_transform: kurbo::Affine,
    account: &Account,
) -> Result<()> {
    // Prepared sbix bitmaps use ordinary sampling and keep the admitted endpoint quad.
    scene.import_image(image, account)?;
    let brush = image_brush(image);
    let brush_transform = affine_product(brush_transform, kurbo::Affine::IDENTITY, account)?;
    scene.fill(
        peniko::Fill::NonZero,
        transform,
        &brush,
        Some(brush_transform),
        &kurbo::Rect::from(rect),
        account,
    )
}

fn encode_image(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    image: &Image,
    rect: Rect,
    fit: ImageFit,
    account: &Account,
) -> Result<()> {
    let local = image_transform(image.size, rect, fit, account)?;
    let image_transform = affine_product(transform, local, account)?;
    let clip_to_target = matches!(fit, ImageFit::Contain | ImageFit::Cover);
    if clip_to_target {
        scene.push_clip_layer(
            peniko::Fill::NonZero,
            transform,
            &kurbo::Rect::from(rect),
            account,
        )?;
    }
    with_sampled_image(
        scene,
        image,
        image_transform,
        |scene, brush, brush_transform| {
            if let Some(brush_transform) = brush_transform {
                scene.fill(
                    peniko::Fill::NonZero,
                    image_transform,
                    brush,
                    Some(brush_transform),
                    &kurbo::Rect::new(0.0, 0.0, image.size.width(), image.size.height()),
                    account,
                )?;
            } else {
                scene.draw_image(brush, image_transform, account)?;
            }
            Ok(())
        },
        account,
    )?;
    if clip_to_target {
        scene.pop_layer(account)?;
    }
    Ok(())
}

fn with_sampled_image<T>(
    scene: &mut VelloScene,
    image: &Image,
    source_to_device: kurbo::Affine,
    encode: impl FnOnce(&mut SceneDraws<'_>, &peniko::ImageBrush, Option<kurbo::Affine>) -> Result<T>,
    account: &Account,
) -> Result<T> {
    scene.import_image(image, account)?;
    let brush = image_brush(image);
    if image.rendering_hint() == Some(ImageRendering::Pixelated) {
        scene.with_pixelated_image(
            image,
            source_to_device,
            |scene, compensation| encode(scene, &brush, Some(compensation)),
            account,
        )
    } else {
        encode(&mut scene.draws(), &brush, None)
    }
}

pub(crate) fn image_brush(image: &Image) -> peniko::ImageBrush {
    peniko::ImageBrush::new(image_data(image))
        .with_quality(image.sampling_quality().into())
        .with_extend(image.extend.into())
}

pub(crate) fn image_data(image: &Image) -> peniko::ImageData {
    image.data.clone()
}

/// The already normalized borrowed facts of one text run.
struct TextRunSource<'a> {
    font: &'a FontRef<'static>,
    size: f32,
    transform: Transform,
    paint: &'a TextPaint,
    glyphs: &'a [TextGlyph],
}

fn encode_text_run(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    source: TextRunSource<'_>,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<()> {
    let TextRunSource {
        font,
        size,
        transform: run_transform,
        paint,
        glyphs,
    } = source;
    let mut run_completion = DisposalCompletion::default();
    let mut font_completion = DisposalCompletion::default();
    let (copied_font, font_name_hold) = font.try_to_owned_in(account)?;
    let font_name_hold = font_name_hold;
    let font = copied_font;
    let font_start = font_completion.start([font_name_hold.as_ref(), None], None);
    let (paint, paint_hold) = paint.try_clone_in(account)?;
    let paint_hold = paint_hold;
    // A failed paint copy disposes the still-armed font. The complete run then
    // borrows both original backings through encoding and actual destruction.
    font_start.disarm();
    let run = TextRun::from_validated_parts(
        font,
        size,
        run_transform,
        paint,
        glyphs,
        TextRunBounds::unspecified(),
    );
    let _run_start = run_completion.start([font_name_hold.as_ref(), paint_hold.as_ref()], None);
    scene.encode_text_run_with_transform(&run, transform, target_extent, account)
}

fn encode_layer_start(
    scene: &mut VelloScene,
    layer: &NormalizedLayer,
    transform: kurbo::Affine,
    account: &Account,
) -> Result<()> {
    let default_clip = RenderClipGeometry::Rect(Rect::new(-1.0e9, -1.0e9, 2.0e9, 2.0e9));
    let (clip, clip_transform) = match &layer.clip {
        Some(clip) => {
            let clip_transform = match clip.coordinate_space() {
                Some(space) => affine_product(transform, space.transform().into(), account)?,
                None => transform,
            };
            (clip.geometry(), clip_transform)
        }
        None => (&default_clip, transform),
    };
    match clip {
        RenderClipGeometry::Rect(rect) => push_vello_layer(
            scene,
            layer,
            peniko::Fill::NonZero,
            clip_transform,
            &kurbo::Rect::from(*rect),
            account,
        ),
        RenderClipGeometry::RoundedRect { rect, radii } => {
            if let Some(circle) = exact_fit_clip_circle(*rect, *radii, account)? {
                push_vello_layer(
                    scene,
                    layer,
                    peniko::Fill::NonZero,
                    clip_transform,
                    &circle,
                    account,
                )
            } else {
                push_vello_layer(
                    scene,
                    layer,
                    peniko::Fill::NonZero,
                    clip_transform,
                    &kurbo_rounded_rect(*rect, *radii),
                    account,
                )
            }
        }
        RenderClipGeometry::Circle { center, radius } => push_vello_layer(
            scene,
            layer,
            peniko::Fill::NonZero,
            clip_transform,
            &kurbo::Circle::new((center.x(), center.y()), *radius),
            account,
        ),
        RenderClipGeometry::Ellipse { center, radii } => push_vello_layer(
            scene,
            layer,
            peniko::Fill::NonZero,
            clip_transform,
            &kurbo::Ellipse::new(
                (center.x(), center.y()),
                (radii.width(), radii.height()),
                0.0,
            ),
            account,
        ),
        RenderClipGeometry::Path { path, fill_rule } => push_vello_layer(
            scene,
            layer,
            vello_fill_rule(*fill_rule),
            clip_transform,
            &*path.kurbo_shape_in(account)?,
            account,
        ),
    }
}

fn push_vello_layer(
    scene: &mut VelloScene,
    layer: &NormalizedLayer,
    fill: peniko::Fill,
    transform: kurbo::Affine,
    shape: &impl kurbo::Shape,
    account: &Account,
) -> Result<()> {
    if layer.isolation == LayerIsolation::ClipOnly {
        scene.push_clip_layer(fill, transform, shape, account)
    } else {
        scene.push_layer(
            fill,
            vello_composition(layer.composition)?,
            layer.opacity.clamp(0.0, 1.0),
            transform,
            shape,
            account,
        )
    }
}

// Keep the solved local map and its intended device field in one state.
// Recomposition can itself lose phase before native narrowing.
#[derive(Clone, Copy)]
struct BrushTransform {
    local: kurbo::Affine,
    reference_device: kurbo::Affine,
    radial: Option<gradient_field::RadialReference>,
    comparison_only: bool,
}

enum PreparedSampling {
    PadRamp(PadEndpoints),
    HorizontalDistanceStep(PadEndpoints),
    Program(hint_program::PreparedProgram),
}

pub(crate) struct FrameBrush {
    brush: peniko::Brush,
    transform: Option<BrushTransform>,
    sampling: Option<PreparedSampling>,
    _hold: Option<Hold>,
    _payload: Option<crate::accounting::CpuImport>,
}
impl Drop for FrameBrush {
    fn drop(&mut self) {
        if let Some(hold) = self._hold.take() {
            let brush = std::mem::replace(
                &mut self.brush,
                peniko::Brush::Solid(peniko::Color::TRANSPARENT),
            );
            hold.dispose(|| drop(brush));
        }
    }
}
impl FrameBrush {
    fn solid(color: peniko::Color) -> Self {
        Self {
            brush: peniko::Brush::Solid(color),
            transform: None,
            sampling: None,
            _hold: None,
            _payload: None,
        }
    }
    fn image(image: &Image, account: &Account) -> Result<Self> {
        let payload = account.import_image(image)?;
        Ok(Self {
            brush: peniko::Brush::Image(image_brush(image)),
            transform: None,
            sampling: None,
            _hold: None,
            _payload: payload,
        })
    }
    pub(crate) const fn transform(&self) -> Option<kurbo::Affine> {
        match self.transform {
            Some(transform) => Some(transform.local),
            None => None,
        }
    }
    pub(crate) fn native_glyph_transform(
        &self,
        transform: kurbo::Affine,
        target_extent: PhysicalSize,
        account: &Account,
    ) -> Result<Option<vello_encoding::Transform>> {
        use gradient_field::NativeGlyphField;

        let Some(brush_transform) = self.transform else {
            return Ok(None);
        };
        let account = account.at(RenderPhase::GradientPrepare);
        account.phase(RenderPhase::GradientPrepare, || {
            let radial_radii = match &self.brush {
                peniko::Brush::Gradient(gradient) => match gradient.kind {
                    peniko::GradientKind::Radial(radial) => {
                        Some((radial.start_radius, radial.end_radius))
                    }
                    _ => None,
                },
                _ => None,
            };
            let sampling = self.linear_sampling(&account)?;
            let field = match (radial_radii, brush_transform.radial) {
                (Some(radii), Some(reference)) => NativeGlyphField::Radial { radii, reference },
                (None, None) => NativeGlyphField::Linear(sampling),
                (Some(_), None) | (None, Some(_)) => {
                    return Err(Error::new(
                        BackendErrorCode::RenderFailed,
                        "prepared native glyph radial field has mismatched radii and reference",
                    ));
                }
            };
            gradient_field::native_glyph_transform(
                brush_transform.local,
                brush_transform.reference_device,
                brush_transform.comparison_only,
                transform,
                target_extent,
                field,
                &account,
            )
            .map_err(|error| {
                if self.sampling.is_some() {
                    ordinary_gradient::capacity_error(error)
                } else {
                    error
                }
            })
            .map(Some)
        })
    }
    fn linear_sampling(&self, account: &Account) -> Result<gradient_field::LinearSampling> {
        use crate::vello_engine::gradient_endpoints::ProgramCoordinates;
        match &self.sampling {
            Some(PreparedSampling::Program(program)) => match program.coordinates {
                ProgramCoordinates::HorizontalDistancePad => {
                    let first_bits = gradient_field::scalar(account, || {
                        program.endpoints.first_position_bits()
                    })?;
                    let last_bits =
                        gradient_field::scalar(account, || program.endpoints.last_position_bits())?;
                    let first = gradient_field::scalar(account, || f32::from_bits(first_bits))?;
                    let last = gradient_field::scalar(account, || f32::from_bits(last_bits))?;
                    let first = gradient_field::scalar(account, || f64::from(first))?;
                    let last = gradient_field::scalar(account, || f64::from(last))?;
                    let span = gradient_field::scalar(account, || last - first)?;
                    Ok(gradient_field::LinearSampling::HorizontalDistancePad { span })
                }
                ProgramCoordinates::HorizontalDistanceRepeat { phase } => {
                    account.work(RenderWork::CoordinateVisits, 1)?;
                    Ok(gradient_field::LinearSampling::HorizontalDistanceRepeat { phase })
                }
                ProgramCoordinates::Pad | ProgramCoordinates::Repeat => {
                    Ok(gradient_field::LinearSampling::Direct)
                }
            },
            _ => Ok(gradient_field::LinearSampling::Direct),
        }
    }
    pub(crate) fn as_ref(&self) -> peniko::BrushRef<'_> {
        (&self.brush).into()
    }
    pub(crate) fn scene_brush(&self, target_extent: PhysicalSize) -> SceneBrush<'_> {
        let sampling = self.sampling.as_ref().map(|sampling| match sampling {
            PreparedSampling::PadRamp(endpoints) => GradientSampling::PadRamp(*endpoints),
            PreparedSampling::HorizontalDistanceStep(endpoints) => {
                GradientSampling::HorizontalDistanceStep(*endpoints)
            }
            PreparedSampling::Program(program) => GradientSampling::Program(GradientProgram {
                records: program.records.as_slice(),
                endpoints: program.endpoints,
                coordinates: program.coordinates,
            }),
        });
        SceneBrush::new(self.as_ref(), sampling, target_extent)
    }
}
impl std::ops::Deref for FrameBrush {
    type Target = peniko::Brush;
    fn deref(&self) -> &Self::Target {
        &self.brush
    }
}

pub(crate) fn glyph_paint_brush(
    paint: &Paint,
    transform: kurbo::Affine,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<FrameBrush> {
    account.work(RenderWork::PaintVisits, 1)?;
    match paint.kind() {
        PaintKind::Color(color) => Ok(FrameBrush::solid((*color).into())),
        PaintKind::Gradient(gradient) => {
            gradient_brush(gradient, transform, target_extent, account)
        }
        PaintKind::RepeatingGradient(gradient) => {
            repeating_gradient::prepare(gradient, transform, target_extent, account)
        }
        PaintKind::Image(image) => FrameBrush::image(image, account),
    }
}
fn render_paint_brush(
    paint: &RenderPaint,
    transform: kurbo::Affine,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<FrameBrush> {
    account.work(RenderWork::PaintVisits, 1)?;
    match paint {
        RenderPaint::Color(color) => Ok(FrameBrush::solid((*color).into())),
        RenderPaint::Gradient(gradient) => {
            gradient_brush(gradient, transform, target_extent, account)
        }
        RenderPaint::RepeatingGradient(gradient) => {
            repeating_gradient::prepare(gradient, transform, target_extent, account)
        }
        RenderPaint::Image(image) => FrameBrush::image(image, account),
    }
}
fn gradient_brush(
    gradient: &Gradient,
    transform: kurbo::Affine,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<FrameBrush> {
    let account = account.at(RenderPhase::GradientPrepare);
    account.phase(RenderPhase::GradientPrepare, || {
        match gradient.preparation() {
            crate::paint::GradientPreparation::Sweep(sweep) => {
                prepare_sweep_gradient(sweep, &account)
            }
            crate::paint::GradientPreparation::Ordinary(field) => {
                ordinary_gradient::prepare(field, transform, target_extent, &account)
            }
        }
    })
}

fn prepare_sweep_gradient(
    gradient: crate::paint::SweepGradient<'_>,
    account: &Account,
) -> Result<FrameBrush> {
    let supplied = gradient.stops();
    if supplied.len() == 1 {
        account.work(RenderWork::GradientStopVisits, 1)?;
        return Ok(FrameBrush::solid(supplied[0].color().into()));
    }
    let mut step = false;
    let endpoints = if supplied.len() >= 2 {
        account.work(RenderWork::GradientStopVisits, 1)?;
        let first = supplied[0];
        let first = (
            gradient_field::scalar(account, || first.position())?,
            first.color(),
        );
        account.work(RenderWork::GradientStopVisits, 1)?;
        let last = supplied[supplied.len() - 1];
        let last = (
            gradient_field::scalar(account, || last.position())?,
            last.color(),
        );
        let (first, last) = if gradient_field::scalar(account, || first.0 == last.0)? {
            step = true;
            for stop in supplied {
                account.work(RenderWork::GradientStopVisits, 1)?;
                let position = gradient_field::scalar(account, || stop.position())?;
                step &= gradient_field::scalar(account, || position == first.0)?;
            }
            let cutoff = ordinary_gradient::cutoff(first.0, 0, account)?;
            ((cutoff, first.1), (cutoff, last.1))
        } else {
            (
                (gradient_field::scalar(account, || first.0 as f32)?, first.1),
                (gradient_field::scalar(account, || last.0 as f32)?, last.1),
            )
        };
        Some(PadEndpoints::new(first, last, account)?)
    } else {
        None
    };
    let leading = gradient.native_leading_stop(account)?;
    let count = gradient
        .stops()
        .len()
        .checked_add(usize::from(leading.is_some()))
        .ok_or_else(|| {
            account
                .storage()
                .capacity_error(RenderResource::ScratchCpuBytes)
        })?;
    let mut stops = account.temporary(count)?;
    // The preflight above inspects one existing record only for multistop
    // fields. This walk materializes every actual native record, including the
    // leading endpoint, into backing whose actual capacity is already admitted.
    if let Some(stop) = leading {
        account.work(RenderWork::GradientStopVisits, 1)?;
        let position = gradient_field::scalar(account, || stop.position())?;
        let offset = gradient_field::scalar(account, || position as f32)?;
        account.work(RenderWork::CoordinateVisits, 4)?;
        stops.append_slot()?.write(peniko::ColorStop::from((
            offset,
            peniko::Color::from(stop.color()),
        )));
    }
    let mut previous = None;
    let mut ordered = true;
    for (index, stop) in supplied.iter().enumerate() {
        account.work(RenderWork::GradientStopVisits, 1)?;
        let position = gradient_field::scalar(account, || stop.position())?;
        let offset = if step {
            0.0
        } else {
            gradient_field::scalar(account, || position as f32)?
        };
        if !step
            && gradient_field::scalar(account, || position != 0.0)?
            && gradient_field::scalar(account, || offset == 0.0)?
        {
            return Err(GradientRealizationError::UnrepresentableStop { stop_index: index }.into());
        }
        if let Some((previous_position, previous_offset)) = previous {
            let source_order =
                gradient_field::scalar(account, || position.partial_cmp(&previous_position))?;
            let native_order =
                gradient_field::scalar(account, || offset.partial_cmp(&previous_offset))?;
            if source_order != native_order {
                return Err(
                    GradientRealizationError::UnrepresentableStop { stop_index: index }.into(),
                );
            }
            ordered &= gradient_field::scalar(account, || position >= previous_position)?;
        }
        previous = Some((position, offset));
        account.work(RenderWork::CoordinateVisits, 4)?;
        stops.append_slot()?.write(peniko::ColorStop::from((
            offset,
            peniko::Color::from(stop.color()),
        )));
    }
    account.work(RenderWork::CoordinateVisits, 2)?;
    let center = gradient.center();
    let (x, y) = (center.x(), center.y());
    let x = gradient_field::scalar(account, || x as f32)?;
    let y = gradient_field::scalar(account, || y as f32)?;
    let output = peniko::Gradient::new_sweep((x, y), 0.0, std::f32::consts::TAU);
    let mut brush = gradient_with_stops(output, stops, None)?;
    brush.sampling = endpoints.filter(|_| ordered).map(PreparedSampling::PadRamp);
    Ok(brush)
}

pub(crate) fn qualify_native_sweep(
    native: vello_encoding::Transform,
    sweep: peniko::SweepGradientPosition,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<()> {
    let account = account.at(RenderPhase::GradientPrepare);
    account.phase(RenderPhase::GradientPrepare, || {
        // This boundary contains only the field's scalar proof. Other receiving
        // geometry, resources, font admission and device work stay outside it.
        gradient_field::qualify_sweep(native, sweep, target_extent, &account).map_err(|error| {
            if error.code() == ErrorCode::InvalidInput {
                GradientRealizationError::InsufficientNativeCapacity.into()
            } else {
                error
            }
        })
    })
}

fn gradient_with_stops(
    mut output: peniko::Gradient,
    stops: AccountedVec<peniko::ColorStop>,
    transform: Option<BrushTransform>,
) -> Result<FrameBrush> {
    let (stops, hold) = stops.into_parts()?;
    // SmallVec::from_vec moves an existing heap allocation; an inline result
    // releases it. Never call with_stops, which clones into an unadmitted backing.
    let hold = if stops.capacity() <= output.stops.capacity() {
        output.stops = hold.dispose(|| peniko::ColorStops(stops.into()));
        None
    } else {
        output.stops = peniko::ColorStops(stops.into());
        Some(hold)
    };
    Ok(FrameBrush {
        brush: peniko::Brush::Gradient(output),
        transform,
        sampling: None,
        _hold: hold,
        _payload: None,
    })
}
struct FrameStroke {
    stroke: kurbo::Stroke,
    backing: Option<Hold>,
}
impl std::ops::Deref for FrameStroke {
    type Target = kurbo::Stroke;
    fn deref(&self) -> &kurbo::Stroke {
        &self.stroke
    }
}
impl Drop for FrameStroke {
    fn drop(&mut self) {
        if let Some(hold) = self.backing.take() {
            let dash = std::mem::take(&mut self.stroke.dash_pattern);
            hold.dispose(|| drop(dash));
        }
    }
}
fn vello_stroke(stroke: &RenderStroke, account: &Account) -> Result<FrameStroke> {
    let mut vello = kurbo::Stroke::new(stroke.width)
        .with_join(stroke.join.into())
        .with_start_cap(stroke.start_cap.into())
        .with_end_cap(stroke.end_cap.into())
        .with_miter_limit(stroke.miter_limit);
    let mut hold = None;
    if let Some(dash) = stroke.dash {
        vello.dash_offset = dash.offset();
        let mut intervals = account.temporary(dash.intervals().len())?;
        for interval in dash.intervals() {
            account.work(RenderWork::PathElementVisits, 1)?;
            intervals.push(*interval)?;
        }
        let (intervals, backing) = intervals.into_parts()?;
        if intervals.capacity() <= vello.dash_pattern.capacity() {
            vello.dash_pattern = backing.dispose(|| intervals.into());
        } else {
            vello.dash_pattern = intervals.into();
            hold = Some(backing);
        }
    }
    Ok(FrameStroke {
        stroke: vello,
        backing: hold,
    })
}

fn vello_composition(composition: LayerComposition) -> Result<peniko::BlendMode> {
    if !composition.is_source_over() && !composition.is_plus() {
        return Err(Error::new(
            BackendErrorCode::RenderFailed,
            "a selected non-source-over operation reached the direct raster encoder",
        ));
    }
    Ok(vello_blend(composition.blend_mode()))
}

fn vello_blend(blend: BlendMode) -> peniko::BlendMode {
    match blend {
        BlendMode::Normal => peniko::Mix::Normal.into(),
        BlendMode::Multiply => peniko::Mix::Multiply.into(),
        BlendMode::Screen => peniko::Mix::Screen.into(),
        BlendMode::Overlay => peniko::Mix::Overlay.into(),
        BlendMode::Darken => peniko::Mix::Darken.into(),
        BlendMode::Lighten => peniko::Mix::Lighten.into(),
        BlendMode::ColorDodge => peniko::Mix::ColorDodge.into(),
        BlendMode::ColorBurn => peniko::Mix::ColorBurn.into(),
        BlendMode::HardLight => peniko::Mix::HardLight.into(),
        BlendMode::SoftLight => peniko::Mix::SoftLight.into(),
        BlendMode::Difference => peniko::Mix::Difference.into(),
        BlendMode::Exclusion => peniko::Mix::Exclusion.into(),
        BlendMode::Hue => peniko::Mix::Hue.into(),
        BlendMode::Saturation => peniko::Mix::Saturation.into(),
        BlendMode::Color => peniko::Mix::Color.into(),
        BlendMode::Luminosity => peniko::Mix::Luminosity.into(),
        BlendMode::Plus => peniko::Compose::Plus.into(),
    }
}

fn vello_fill_rule(fill_rule: FillRule) -> peniko::Fill {
    match fill_rule {
        FillRule::NonZero => peniko::Fill::NonZero,
        FillRule::EvenOdd => peniko::Fill::EvenOdd,
    }
}

fn offset_rect(rect: Rect, offset: Point) -> Rect {
    Rect::new(
        rect.x() + offset.x(),
        rect.y() + offset.y(),
        rect.width(),
        rect.height(),
    )
}

pub(crate) fn image_transform(
    size: Size,
    rect: Rect,
    fit: ImageFit,
    account: &Account,
) -> Result<kurbo::Affine> {
    let positive_source = |size: f64| -> Result<f64> {
        if coordinate_action(account, || size <= 0.0)? {
            return Err(Error::new(
                BackendErrorCode::ImageUploadFailed,
                "image size must be positive",
            ));
        }
        Ok(size)
    };
    let width = positive_source(coordinate_action(account, || size.width())?)?;
    let height = positive_source(coordinate_action(account, || size.height())?)?;
    let target_width = coordinate_action(account, || rect.width())?;
    let scale_x = coordinate_action(account, || target_width / width)?;
    let target_height = coordinate_action(account, || rect.height())?;
    let scale_y = coordinate_action(account, || target_height / height)?;
    let (fit_scale_x, fit_scale_y, tx, ty) = match fit {
        ImageFit::Fill | ImageFit::Stretch | ImageFit::None => (
            scale_x,
            scale_y,
            coordinate_action(account, || rect.x())?,
            coordinate_action(account, || rect.y())?,
        ),
        ImageFit::Contain | ImageFit::Cover => {
            let scale = coordinate_action(account, || match fit {
                ImageFit::Contain => scale_x.min(scale_y),
                _ => scale_x.max(scale_y),
            })?;
            let center = |origin: f64, target: f64, source: f64| -> Result<f64> {
                let fitted = coordinate_action(account, || source * scale)?;
                let remaining = coordinate_action(account, || target - fitted)?;
                let offset = coordinate_action(account, || remaining * 0.5)?;
                coordinate_action(account, || origin + offset)
            };
            let tx = center(
                coordinate_action(account, || rect.x())?,
                target_width,
                width,
            )?;
            let ty = center(
                coordinate_action(account, || rect.y())?,
                target_height,
                height,
            )?;
            (scale, scale, tx, ty)
        }
    };
    affine_product(
        kurbo::Affine::translate((tx, ty)),
        kurbo::Affine::scale_non_uniform(fit_scale_x, fit_scale_y),
        account,
    )
}

#[cfg(test)]
mod image_fit_accounting_tests {
    use super::*;
    use crate::accounting::{FrameFixture, RenderAttemptKind, RendererCpu};

    fn positive_image() -> Image {
        Image::from_rgba(Size::new(4.0, 2.0), [255, 0, 0, 255].repeat(8)).unwrap()
    }

    fn assert_fit_refuses_before_downstream_work(image: &Image) {
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CoordinateVisits, 0)
            .unwrap()
            .with_work_limit(RenderWork::PathElementVisits, 0)
            .unwrap();
        let options = RenderAccountingOptions::new()
            .with_limits(limits)
            .with_profile(RenderProfile::Detailed {
                timing: false,
                event_limit: 16,
                filter: RenderPhaseFilter::Phase(RenderPhase::Encode),
            });
        let cpu = RendererCpu::new(options).unwrap();
        let (result, _, report, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|attempt| {
                let account = attempt.account().at(RenderPhase::Encode);
                account.phase(RenderPhase::Encode, || {
                    let mut scene = VelloScene::default();
                    encode_image(
                        &mut scene,
                        kurbo::Affine::translate((8.0, -3.0)),
                        image,
                        Rect::try_new(1.0, 0.0, 2.0, 2.0).unwrap(),
                        ImageFit::Fill,
                        &account,
                    )
                })
            })
            .unwrap()
            .into_parts();
        assert_eq!(fault, None);
        let attempt = report.attempt().unwrap();
        assert_eq!(attempt.status(), RenderAttemptStatus::Refused);
        let refusal = attempt.refusal().unwrap();
        assert_eq!(
            result.unwrap_err().accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
        );
        assert_eq!(refusal.phase(), RenderPhase::Attempt);
        assert_eq!((refusal.current(), refusal.amount()), (0, 1));
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 0 });
        assert!(attempt.events().any(|event| matches!(
            event,
            RenderEvent::Refused {
                phase: RenderPhase::Encode,
                ..
            }
        )));
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::CoordinateVisits),
            "image fit must admit its first coordinate before downstream work"
        );

        let encode = attempt.phase_usage(RenderPhase::Encode);
        for work in [
            RenderWork::CoordinateVisits,
            RenderWork::PathElementVisits,
            RenderWork::PaintVisits,
            RenderWork::ResourceProbes,
            RenderWork::ImageSourceBytes,
            RenderWork::AtlasTexels,
        ] {
            assert_eq!(encode.work(work), 0, "{work:?}");
        }
        let cpu = report.renderer_cpu().unwrap();
        for scope in [RenderCpuScope::ImagePayloads, RenderCpuScope::FrameBacking] {
            let usage = cpu.scope_usage(scope);
            for resource in [
                RenderResource::RetainedCpuBytes,
                RenderResource::ScratchCpuBytes,
            ] {
                assert_eq!(
                    usage.peak(resource),
                    usage.opening_live(resource),
                    "{scope:?} {resource:?} peak"
                );
                assert_eq!(
                    usage.live(resource),
                    usage.opening_live(resource),
                    "{scope:?} {resource:?} live"
                );
            }
        }
    }

    #[test]
    fn ordinary_image_fit_refuses_coordinates_before_path_preparation() {
        assert_fit_refuses_before_downstream_work(&positive_image());
    }

    #[test]
    fn pixelated_image_fit_refuses_coordinates_before_path_preparation() {
        let image = positive_image().rendering(ImageRendering::Pixelated);
        assert_fit_refuses_before_downstream_work(&image);
    }

    #[test]
    fn image_fit_places_literal_bounds_and_counts_local_work() {
        let size = Size::new(4.0, 2.0);
        let rect = Rect::new(1.0, 0.0, 2.0, 2.0);
        for (fit, visits, expected) in [
            (ImageFit::Fill, 42, [0.5, 0.0, 0.0, 1.0, 1.0, 0.0]),
            (ImageFit::Contain, 51, [0.5, 0.0, 0.0, 0.5, 1.0, 0.5]),
            (ImageFit::Cover, 51, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        ] {
            let mut context = RenderAccountingContext::default();
            let fitted = context
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                    image_transform(size, rect, fit, account)
                })
                .unwrap();
            assert_eq!(fitted.as_coeffs(), expected, "{fit:?}");
            let attempt = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(
                attempt
                    .phase_usage(RenderPhase::Encode)
                    .work(RenderWork::CoordinateVisits),
                visits,
                "{fit:?}"
            );
        }
    }

    #[test]
    fn image_fit_admits_the_device_product_before_entering_the_draw() {
        let rect = Rect::new(1.0, 0.0, 2.0, 2.0);
        for (fit, visits) in [
            (ImageFit::Fill, 74),
            (ImageFit::Contain, 83),
            (ImageFit::Cover, 83),
        ] {
            for (cap, next_work, current) in [
                (visits - 1, RenderWork::CoordinateVisits, visits - 1),
                (visits, RenderWork::PathElementVisits, 0),
            ] {
                assert_placement_boundary(cap, next_work, current, |account| {
                    encode_image(
                        &mut VelloScene::default(),
                        kurbo::Affine::translate((8.0, -3.0)),
                        &positive_image(),
                        rect,
                        fit,
                        account,
                    )
                });
            }
        }
    }

    #[test]
    fn prepared_bitmap_replay_admits_one_affine_product_before_the_draw() {
        for (cap, next_work, current) in [
            (31, RenderWork::CoordinateVisits, 31),
            (32, RenderWork::PathElementVisits, 0),
        ] {
            assert_placement_boundary(cap, next_work, current, |account| {
                encode_prepared_bitmap(
                    &mut VelloScene::default(),
                    kurbo::Affine::translate((8.0, -3.0)),
                    &positive_image(),
                    Rect::new(1.0, 0.0, 2.0, 2.0),
                    kurbo::Affine::new([0.5, 0.0, 0.0, 1.0, 1.0, 0.0]),
                    account,
                )
            });
        }
    }

    fn assert_placement_boundary(
        cap: u64,
        next_work: RenderWork,
        current: u64,
        draw: impl FnOnce(&Account) -> Result<()>,
    ) {
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CoordinateVisits, cap)
            .unwrap()
            .with_work_limit(RenderWork::PathElementVisits, 0)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, draw)
            .unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("expected a work refusal at cap {cap}: {error:?}");
        };
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(next_work),
            "cap {cap}"
        );
        assert_eq!((refusal.current(), refusal.amount()), (current, 1));
        let usage = context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .phase_usage(RenderPhase::Encode);
        assert_eq!(usage.work(RenderWork::CoordinateVisits), cap);
        assert_eq!(usage.work(RenderWork::PathElementVisits), 0);
    }
}

#[cfg(test)]
mod layer_transform_accounting_tests {
    use super::*;
    use crate::accounting::{FrameFixture, RendererCpu};

    fn tagged_layer_commands() -> (RenderCommands, Transform, Transform) {
        let clip = ClipInput::try_shape(Shape::rect(Rect::new(0.0, 0.0, 1.0, 1.0)))
            .unwrap()
            .with_coordinate_space(
                CoordinateSpaceTag::surface(Transform::translation(1.0, 1.0).unwrap()).unwrap(),
            );
        let layer_transform = Transform::try_new([2.0, 0.0, 0.0, 1.0, -1.0, -1.0]).unwrap();
        let initial_transform = Transform::translation(3.0, 1.0).unwrap();
        let layer = Layer::new()
            .with_transform(layer_transform)
            .try_clip_input(clip)
            .unwrap();
        let mut scene = Scene::new();
        scene.layer(layer, |scene| {
            scene.fill(Rect::new(0.0, 0.0, 4.0, 4.0), Color::BLACK);
        });
        let commands = scene
            .normalize(Capabilities::CURRENT, &Account::for_test())
            .unwrap();
        (commands, layer_transform, initial_transform)
    }

    fn assert_refusal_before_native_path(
        cap: u64,
        amount: u64,
        encode: impl FnOnce(&Account) -> Result<VelloScene>,
    ) {
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CoordinateVisits, cap)
            .unwrap()
            .with_work_limit(RenderWork::PathElementVisits, 0)
            .unwrap();
        let options = RenderAccountingOptions::new()
            .with_limits(limits)
            .with_profile(RenderProfile::Detailed {
                timing: false,
                event_limit: 128,
                filter: RenderPhaseFilter::Phase(RenderPhase::Encode),
            });
        let cpu = RendererCpu::new(options).unwrap();
        let (result, _, report, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|attempt| encode(attempt.account()).map(drop))
            .unwrap()
            .into_parts();
        assert_eq!(fault, None);
        let error = result.unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("expected a transform coordinate refusal at cap {cap}: {error:?}");
        };
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::CoordinateVisits),
            "transform composition must refuse before native geometry at cap {cap}",
        );
        assert_eq!(
            (refusal.current(), refusal.amount()),
            (cap, amount),
            "wrong admission boundary at cap {cap}",
        );
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: cap });
        let report = report.attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(
            report.dropped_events(),
            0,
            "incomplete profile at cap {cap}"
        );
        assert!(report.events().any(|event| matches!(
            event,
            RenderEvent::Refused {
                phase: RenderPhase::Encode,
                ..
            }
        )));
        let usage = report.phase_usage(RenderPhase::Encode);
        assert_eq!(usage.work(RenderWork::CoordinateVisits), cap);
        assert_eq!(usage.work(RenderWork::PathElementVisits), 0);
        assert_eq!(usage.work(RenderWork::PaintVisits), 0);
    }

    #[test]
    fn direct_layer_and_tagged_clip_refuse_before_native_geometry() {
        let (commands, _, initial_transform) = tagged_layer_commands();
        let extent = PhysicalSize::new(8, 4);
        // Each product reads twelve input coefficients and performs twenty
        // scalar operations. A separate charge starts each product.
        for (cap, amount) in [(0, 12), (31, 1), (32, 12), (63, 1)] {
            assert_refusal_before_native_path(cap, amount, |account| {
                encode_vello_scene_with_initial_transform(
                    &commands,
                    initial_transform,
                    extent,
                    account,
                )
            });
        }
        encode_vello_scene_with_initial_transform(
            &commands,
            initial_transform,
            extent,
            &Account::for_test(),
        )
        .unwrap();
    }

    #[test]
    fn clip_capture_refuses_during_composition_and_finite_validation() {
        let (commands, layer_transform, initial_transform) = tagged_layer_commands();
        let Some(RenderCommand::Layer { layer, .. }) = commands.iter().next() else {
            panic!("expected one normalized layer");
        };
        let elements = [(layer.clip.as_ref().unwrap().clone(), layer_transform)];
        let extent = PhysicalSize::new(8, 4);
        // Each product is followed by six finite checks before its result
        // can be used as the next product's input.
        for (cap, amount) in [(0, 12), (31, 1), (32, 1), (37, 1), (38, 12), (75, 1)] {
            assert_refusal_before_native_path(cap, amount, |account| {
                encode_vello_clip_coverage_scene(&elements, initial_transform, extent, account)
            });
        }
        encode_vello_clip_coverage_scene(
            &elements,
            initial_transform,
            extent,
            &Account::for_test(),
        )
        .unwrap();
    }
    #[test]
    fn clip_capture_keeps_the_first_nonfinite_coefficient_error() {
        use crate::accounting::RenderAttemptKind;

        let (commands, layer_transform, _) = tagged_layer_commands();
        let Some(RenderCommand::Layer { layer, .. }) = commands.iter().next() else {
            panic!("expected one normalized layer");
        };
        let elements = [(layer.clip.as_ref().unwrap().clone(), layer_transform)];
        let initial_transform = Transform::scale(f64::MAX, 1.0).unwrap();
        let mut context = RenderAccountingContext::new(RenderAccountingOptions::new());
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                encode_vello_clip_coverage_scene(
                    &elements,
                    initial_transform,
                    PhysicalSize::new(8, 4),
                    account,
                )
                .map(drop)
            })
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        let diagnostic = error.invalid_value_diagnostic().unwrap();
        assert_eq!(diagnostic.field(), "transform");
        assert_eq!(diagnostic.value(), "inf");
        let usage = context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .phase_usage(RenderPhase::Encode);
        // The first composition and its checks complete. The second
        // composition refuses at its first finite check.
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 38 + 32 + 1);
        assert_eq!(usage.work(RenderWork::PathElementVisits), 0);
    }
}

#[cfg(test)]
mod ordinary_gradient_accounting_tests {
    use super::*;
    use crate::accounting::RenderAttemptKind;

    fn gradient(positions: &[f64]) -> Gradient {
        Gradient::try_linear(
            Point::new(0.0, 0.0),
            Point::new(8.0, 0.0),
            positions
                .iter()
                .map(|p| GradientStop::try_new(*p, Color::BLACK).unwrap())
                .collect(),
        )
        .unwrap()
    }
    fn prepared(gradient: &Gradient, account: &Account) -> Result<FrameBrush> {
        gradient_brush(
            gradient,
            kurbo::Affine::IDENTITY,
            PhysicalSize::new(16, 8),
            account,
        )
    }
    #[test]
    fn native_ramps_preserve_premultiplied_intervals_in_each_field_coordinate() {
        for (alpha, leading, midpoint) in [
            (1.0, [255_u8, 0, 0, 255], [128_u8, 0, 128, 255]),
            (0.5, [128, 0, 0, 128], [64, 0, 128, 191]),
        ] {
            for field in 0..3 {
                let red = Color::try_rgba(1.0, 0.0, 0.0, alpha).unwrap();
                let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
                let stops = vec![
                    GradientStop::try_new(0.5, red).unwrap(),
                    GradientStop::try_new(1.0, blue).unwrap(),
                ];
                let gradient = match field {
                    0 => Gradient::try_linear(Point::new(0., 0.), Point::new(8., 0.), stops),
                    1 => Gradient::try_radial(Point::new(0., 4.), 8., 8., stops),
                    _ => Gradient::try_sweep(Point::new(12., 4.), stops),
                }
                .unwrap();
                let brush = prepared(&gradient, &Account::for_test()).unwrap();
                let mut encoding = vello_encoding::Encoding::new();
                encoding.encode_brush(brush.as_ref(), 1.0);
                let mut resolver = vello_encoding::Resolver::default();
                let mut packed = Vec::new();
                let (_, ramps, _) = resolver.resolve(&encoding, &mut packed);
                // Ordinary nonzero fields map their first/last source stops to
                // 0/1. Sweep retains its normalized leading anchor.
                let indices = if field == 2 { [128, 384] } else { [0, 256] };
                for (index, expected) in indices.into_iter().zip([leading, midpoint]) {
                    let actual = ramps.data[index].to_le_bytes();
                    assert!(
                        actual
                            .into_iter()
                            .zip(expected)
                            .all(|(a, b)| a.abs_diff(b) <= 1),
                        "field{field} alpha{alpha}: {actual:?} expected{expected:?}"
                    );
                }
            }
        }
    }
    #[test]
    fn unordered_sweep_retains_its_supplied_native_order() {
        let gradient = Gradient::try_sweep(
            Point::new(0., 0.),
            vec![
                GradientStop::try_new(0.75, Color::BLACK).unwrap(),
                GradientStop::try_new(0.25, Color::BLACK).unwrap(),
            ],
        )
        .unwrap();
        let brush = prepared(&gradient, &Account::for_test()).unwrap();
        assert!(brush.sampling.is_none());
        let peniko::BrushRef::Gradient(native) = brush.as_ref() else {
            panic!("gradient");
        };
        assert_eq!(
            native.stops.iter().map(|s| s.offset).collect::<Vec<_>>(),
            [0., 0.75, 0.25]
        );
    }
    #[test]
    fn ordinary_one_stop_is_solid_without_unused_geometry_or_native_storage() {
        let mut context = RenderAccountingContext::default();
        context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    let brush = prepared(&gradient(&[0.5]), account)?;
                    assert!(matches!(brush.as_ref(), peniko::BrushRef::Solid(_)));
                    Ok(())
                },
            )
            .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::GradientStopVisits), 1);
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
    }
    #[test]
    fn one_stop_sweep_is_solid_even_when_its_unused_position_would_underflow() {
        let color = Color::try_rgba(0.25, 0.5, 0.75, 0.5).unwrap();
        let gradient = Gradient::try_sweep(
            Point::new(0., 0.),
            vec![GradientStop::try_new(f64::from_bits(1), color).unwrap()],
        )
        .unwrap();
        let mut context = RenderAccountingContext::default();
        context.run(RenderAttemptKind::Frame,RenderPhase::GradientPrepare,|account|{
            let brush=prepared(&gradient,account)?;
            assert!(matches!(brush.as_ref(),peniko::BrushRef::Solid(actual) if actual==peniko::Color::from(color)));
            Ok(())
        }).unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::GradientStopVisits), 1);
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
    }
    #[test]
    fn canonical_record_backing_stays_guarded_through_heap_handoff() {
        let gradient = gradient(&[0.5, 0.6, 0.7, 0.8, 1.0]);
        let mut context = RenderAccountingContext::default();
        let bytes = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    let brush = prepared(&gradient, account)?;
                    let peniko::BrushRef::Gradient(native) = brush.as_ref() else {
                        panic!("gradient");
                    };
                    assert_eq!(native.stops.len(), 5);
                    Ok((native.stops.capacity() * std::mem::size_of::<peniko::ColorStop>()) as u64)
                },
            )
            .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), bytes);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        // Two endpoint records and exactly one conversion visit per source stop.
        assert_eq!(usage.work(RenderWork::GradientStopVisits), 7);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
    }
    #[test]
    fn endpoint_admission_refusals_preserve_the_actual_prefix_without_allocation() {
        for (limits, stops) in [
            (
                RenderLimits::new()
                    .with_work_limit(RenderWork::GradientStopVisits, 0)
                    .unwrap(),
                0,
            ),
            (
                RenderLimits::new()
                    .with_work_limit(RenderWork::GradientStopVisits, 1)
                    .unwrap(),
                1,
            ),
            (
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 0)
                    .unwrap(),
                2,
            ),
        ] {
            let mut context =
                RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
            let error = context
                .run(
                    RenderAttemptKind::Frame,
                    RenderPhase::GradientPrepare,
                    |account| prepared(&gradient(&[0.5, 1.0]), account).map(drop),
                )
                .unwrap_err();
            assert!(error.accounting_diagnostic().is_some());
            let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::GradientStopVisits), stops);
            assert_eq!(usage.work(RenderWork::CoordinateVisits), 0);
            assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        }
    }
    #[test]
    fn native_storage_refusal_releases_all_accepted_scalar_work_and_can_retry() {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(RenderLimits::new().with_scratch_bytes(0)),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| prepared(&gradient(&[0.5, 1.0]), account).map(drop),
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::GradientStopVisits), 2);
        assert!(usage.work(RenderWork::CoordinateVisits) > 0);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        assert!(prepared(&gradient(&[0.5, 1.0]), &Account::for_test()).is_ok());
    }
}

#[cfg(test)]
mod composition_admission_tests {
    use super::*;

    #[test]
    fn direct_raster_admits_source_over_and_plus_but_rejects_selected_operators() {
        let source_over =
            LayerComposition::try_porter_duff(PorterDuffOperator::SourceOver, BlendMode::Color)
                .unwrap();
        assert_eq!(
            vello_composition(source_over).unwrap(),
            peniko::Mix::Color.into()
        );
        assert_eq!(
            vello_composition(LayerComposition::from_blend(BlendMode::Plus)).unwrap(),
            peniko::Compose::Plus.into(),
        );
        let copy =
            LayerComposition::try_porter_duff(PorterDuffOperator::Copy, BlendMode::Normal).unwrap();
        assert_eq!(
            vello_composition(copy).unwrap_err().code(),
            ErrorCode::RenderFailed
        );
    }
}

#[cfg(test)]
mod paint_visit_tests {
    use super::*;

    fn context(limit: u64) -> RenderAccountingContext {
        RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::PaintVisits, limit)
                    .unwrap(),
            ),
        )
    }

    fn paints() -> [Paint; 3] {
        [
            Paint::color(Color::BLACK),
            Paint::gradient(
                Gradient::try_linear(
                    Point::new(0.0, 0.0),
                    Point::new(2.0, 0.0),
                    vec![GradientStop::try_new(0.0, Color::BLACK).unwrap()],
                )
                .unwrap(),
            ),
            Paint::image(Image::from_rgba(Size::new(1.0, 1.0), vec![0, 0, 0, 255]).unwrap()),
        ]
    }

    fn commands(paint: Paint, stroke: bool) -> RenderCommands {
        let mut scene = Scene::new();
        let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
        if stroke {
            scene.stroke(rect, Stroke::try_new(1.0).unwrap(), paint);
        } else {
            scene.fill(rect, paint);
        }
        // Authored normalization is an earlier, independently measured walk.
        scene
            .normalize(Capabilities::CURRENT, &Account::for_test())
            .unwrap()
    }

    fn refusal(error: &Error, current: u64, limit: u64) {
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("expected a paint-visit refusal, got {error:?}");
        };
        // The configured cap belongs to the enclosing attempt. The origin of
        // this inspection remains Encode, but the refusal names the cap owner.
        assert_eq!(refusal.phase(), RenderPhase::Attempt);
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::PaintVisits)
        );
        assert_eq!(refusal.current(), current);
        assert_eq!(refusal.amount(), 1);
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
    }

    #[test]
    fn brush_source_dispatch_counts_each_authored_and_normalized_paint_inspection() {
        for paint in paints() {
            let normalized = commands(paint.clone(), false);
            let RenderCommand::Fill {
                paint: render_paint,
                ..
            } = &normalized.commands[0]
            else {
                panic!("the fixture must normalize to one fill");
            };
            for authored in [true, false] {
                let mut context = context(1);
                context
                    .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                        if authored {
                            glyph_paint_brush(
                                &paint,
                                kurbo::Affine::IDENTITY,
                                PhysicalSize::new(2, 2),
                                account,
                            )
                            .map(drop)
                        } else {
                            render_paint_brush(
                                render_paint,
                                kurbo::Affine::IDENTITY,
                                PhysicalSize::new(2, 2),
                                account,
                            )
                            .map(drop)
                        }
                    })
                    .unwrap();
                assert_eq!(
                    context
                        .last_attempt()
                        .unwrap()
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::PaintVisits),
                    1
                );
            }
        }
    }

    #[test]
    fn brush_source_dispatch_refuses_before_materializing_a_paint() {
        for paint in paints() {
            let normalized = commands(paint.clone(), false);
            let RenderCommand::Fill {
                paint: render_paint,
                ..
            } = &normalized.commands[0]
            else {
                panic!("the fixture must normalize to one fill");
            };
            for authored in [true, false] {
                let mut context = context(0);
                let error = context
                    .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                        if authored {
                            glyph_paint_brush(
                                &paint,
                                kurbo::Affine::IDENTITY,
                                PhysicalSize::new(2, 2),
                                account,
                            )
                            .map(drop)
                        } else {
                            render_paint_brush(
                                render_paint,
                                kurbo::Affine::IDENTITY,
                                PhysicalSize::new(2, 2),
                                account,
                            )
                            .map(drop)
                        }
                    })
                    .unwrap_err();
                refusal(&error, 0, 0);
                let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
                assert_eq!(usage.work(RenderWork::PaintVisits), 0);
                assert_eq!(usage.work(RenderWork::GradientStopVisits), 0);
            }
        }
    }

    #[test]
    fn fill_and_stroke_count_routing_and_brush_inspections_separately() {
        for stroke in [false, true] {
            for (paint, expected) in paints().into_iter().zip([3, 3, 2]) {
                let normalized = commands(paint, stroke);
                let mut context = context(expected);
                context
                    .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                        encode_vello_scene_with_initial_transform(
                            &normalized,
                            Transform::identity(),
                            PhysicalSize::new(2, 2),
                            account,
                        )
                        .map(drop)
                    })
                    .unwrap();
                // Color/gradient: image routing, source dispatch, native brush.
                // Image: routing takes the sampled-image branch directly, then native brush.
                assert_eq!(
                    context
                        .last_attempt()
                        .unwrap()
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::PaintVisits),
                    expected
                );
            }
        }
    }

    #[test]
    fn fill_and_stroke_refuse_paint_routing_before_geometry_encoding() {
        for stroke in [false, true] {
            for paint in paints() {
                let normalized = commands(paint, stroke);
                let mut context = context(0);
                let error = context
                    .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                        encode_vello_scene_with_initial_transform(
                            &normalized,
                            Transform::identity(),
                            PhysicalSize::new(2, 2),
                            account,
                        )
                        .map(drop)
                    })
                    .unwrap_err();
                refusal(&error, 0, 0);
                assert_eq!(
                    context
                        .last_attempt()
                        .unwrap()
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::PathElementVisits),
                    0
                );
            }
        }
    }

    fn prepared_commands(paint: Paint) -> RenderCommands {
        let font = FontData::try_from_bytes(
            include_bytes!("../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let run = font
            .prepare_glyph_run(
                &TextInstance::try_new(20.0, vec![], GlyphSynthesis::none()).unwrap(),
                &[TextGlyph::try_new(58, 0.0, 16.0, 20.0).unwrap()],
                GlyphRunLimits::new(1, 16, 1 << 20, 1 << 20),
                &mut RenderAccountingContext::default(),
            )
            .unwrap();
        assert!(matches!(
            run.paints(),
            [crate::text::GlyphPaint::Fill { color: None, .. }]
        ));
        let mut scene = Scene::new();
        scene.prepared_glyph_run(
            run,
            Transform::identity(),
            TextPaint::try_fill(paint).unwrap(),
        );
        scene
            .normalize(Capabilities::CURRENT, &Account::for_test())
            .unwrap()
    }

    #[test]
    fn prepared_fill_counts_qualification_dispatch_routing_and_native_brush_inspections() {
        for paint in paints() {
            let normalized = prepared_commands(paint);
            let mut context = context(5);
            context
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                    encode_vello_scene_with_initial_transform(
                        &normalized,
                        Transform::identity(),
                        PhysicalSize::new(2, 2),
                        account,
                    )
                    .map(drop)
                })
                .unwrap();
            // Five distinct walks/reads of existing records: placement qualification,
            // retained GlyphPaint dispatch, authored brush dispatch, image routing,
            // and consumption of the newly materialized native brush.
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::PaintVisits),
                5
            );
        }
    }

    #[test]
    fn prepared_fill_refuses_each_inspection_after_its_exact_accepted_prefix() {
        let normalized = prepared_commands(Paint::color(Color::BLACK));
        for limit in 1..=4 {
            let mut context = context(limit);
            let error = context
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                    encode_vello_scene_with_initial_transform(
                        &normalized,
                        Transform::identity(),
                        PhysicalSize::new(2, 2),
                        account,
                    )
                    .map(drop)
                })
                .unwrap_err();
            refusal(&error, limit, limit);
            let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::PaintVisits), limit);
            if limit < 4 {
                assert_eq!(usage.work(RenderWork::PathElementVisits), 0);
            } else {
                assert!(usage.work(RenderWork::PathElementVisits) > 0);
            }
        }
    }

    mod text_value_tests {
        use super::*;
        use crate::accounting::{FrameFixture, RendererCpu};
        #[test]
        fn text_value_paint_copy_refusal_releases_the_previously_copied_font_name() {
            let options = RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::PaintVisits, 0)
                    .unwrap(),
            );
            let cpu = RendererCpu::new(options).unwrap();
            let opening = cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes);
            let (result, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    let account = attempt.account().at(RenderPhase::Encode);
                    let font = FontRef::new(1).named("owned intermediate font name");
                    let paint = TextPaint::try_fill(paints()[1].clone()).unwrap();
                    let mut encoded = VelloScene::default();
                    let error = encode_text_run(
                        &mut encoded,
                        kurbo::Affine::IDENTITY,
                        TextRunSource {
                            font: &font,
                            size: 20.0,
                            transform: Transform::identity(),
                            paint: &paint,
                            glyphs: &[],
                        },
                        PhysicalSize::new(2, 2),
                        &account,
                    )
                    .unwrap_err();
                    refusal(&error, 0, 0);
                    let snapshot = cpu.snapshot().unwrap();
                    assert!(
                        snapshot.usage().peak(RenderResource::RetainedCpuBytes)
                            >= opening + font.name.as_ref().unwrap().len() as u64
                    );
                    assert_eq!(
                        snapshot.usage().live(RenderResource::RetainedCpuBytes),
                        opening
                    );
                    assert_eq!(encoded.observation_for_test().glyph_run_count_for_test(), 0);

                    Err::<(), _>(error)
                })
                .unwrap()
                .into_parts();
            assert!(result.is_err());
            assert_eq!(report.status(), crate::RenderAttemptStatus::Refused);
            assert_eq!(fault, None);
        }

        #[test]
        fn complete_text_staging_releases_both_independent_copies_after_success_or_font_error() {
            for with_data in [false, true] {
                let options = RenderAccountingOptions::new();
                let cpu = RendererCpu::new(options).unwrap();
                let opening = cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes);
                let (result, _, report, fault) = FrameFixture::new(&cpu, options)
                    .run_sync(|attempt| {
                        let name = "independently copied display name";
                        let mut font = FontRef::new(1).named(name);
                        if with_data {
                            font = font.with_data(
                                crate::FontData::try_from_bytes(
                                    include_bytes!("../tests/fixtures/fonts/ahem/Ahem.ttf")
                                        .to_vec(),
                                    0,
                                    surgeist_test::fixtures::FONT_LIMITS,
                                )
                                .unwrap(),
                            );
                        }
                        let paint = TextPaint::try_fill(paints()[1].clone()).unwrap();
                        let mut encoded = VelloScene::default();
                        let result = encode_text_run(
                            &mut encoded,
                            kurbo::Affine::IDENTITY,
                            TextRunSource {
                                font: &font,
                                size: 20.0,
                                transform: Transform::identity(),
                                paint: &paint,
                                glyphs: &[],
                            },
                            PhysicalSize::new(2, 2),
                            attempt.account(),
                        );
                        if with_data {
                            result.as_ref().unwrap();
                        } else {
                            let error = result.as_ref().unwrap_err();
                            assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
                            assert!(error.message().contains("font data"));
                        }
                        let snapshot = cpu.snapshot().unwrap();
                        assert!(
                            snapshot.usage().peak(RenderResource::RetainedCpuBytes)
                                >= opening
                                    + name.len() as u64
                                    + std::mem::size_of::<GradientStop>() as u64
                        );
                        assert_eq!(
                            snapshot.usage().live(RenderResource::RetainedCpuBytes),
                            opening
                        );
                        assert_eq!(encoded.observation_for_test().glyph_run_count_for_test(), 0);
                        assert_eq!(font.name.as_deref(), Some(name));
                        assert_eq!(paint, TextPaint::try_fill(paints()[1].clone()).unwrap());

                        result
                    })
                    .unwrap()
                    .into_parts();
                assert_eq!(result.is_ok(), with_data);
                assert_eq!(
                    report.status(),
                    if with_data {
                        crate::RenderAttemptStatus::Succeeded
                    } else {
                        crate::RenderAttemptStatus::Failed(crate::ErrorCode::InvalidInput)
                    }
                );
                assert_eq!(fault, None);
            }
        }
    }
}

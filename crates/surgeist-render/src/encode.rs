use super::{
    command::{
        LayerIsolation, NormalizedLayer, RenderClip, RenderClipGeometry, RenderCommand,
        RenderCommands, RenderPaint, RenderShadow, RenderShape, RenderStroke, RenderStrokeShape,
        ShadowShape, kurbo_rounded_rect,
    },
    geometry::{expand_rect, offset_radii},
    paint::PaintKind,
    vello_engine::scene::{RoundedRectBlur, VelloScene},
    *,
};
use crate::accounting::{Account, Hold, RenderPhase, RenderWork};

pub(crate) fn encode_vello_scene(
    commands: &RenderCommands,
    scale: f64,
    account: &Account,
) -> Result<VelloScene> {
    encode_vello_scene_with_initial_transform(commands, Transform::scale(scale, scale)?, account)
}

pub(crate) fn encode_vello_scene_with_initial_transform(
    commands: &RenderCommands,
    initial_transform: Transform,
    account: &Account,
) -> Result<VelloScene> {
    let account = account.at(RenderPhase::Encode);
    account.phase(RenderPhase::Encode, || {
        let mut encoded = VelloScene::default();
        encode_vello_commands(
            &commands.commands,
            &mut encoded,
            kurbo::Affine::from(initial_transform),
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
                clip_coverage_transform(clip, *transform, initial_transform)?,
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
) -> Result<Transform> {
    clip.coordinate_space()
        .map(|coordinate_space| coordinate_space.transform())
        .unwrap_or_else(Transform::identity)
        .then(transform)?
        .then(initial_transform)
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
        RenderClipGeometry::RoundedRect { rect, radii } => scene.push_clip_layer(
            peniko::Fill::NonZero,
            transform,
            &kurbo_rounded_rect(*rect, *radii),
            account,
        ),
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
    account: &Account,
) -> Result<()> {
    for command in commands {
        account.work(RenderWork::CommandVisits, 1)?;
        match command {
            RenderCommand::Fill { shape, paint } => {
                encode_fill(scene, transform, shape, paint, account)?
            }
            RenderCommand::Stroke {
                shape,
                stroke,
                paint,
            } => encode_stroke(scene, transform, shape, stroke, paint, account)?,
            RenderCommand::Shadow { shape, shadow } => {
                encode_shadow(scene, transform, shape, shadow, account)?
            }
            RenderCommand::Image { image, rect, fit } => {
                encode_image(scene, transform, image, *rect, *fit, account)?
            }
            RenderCommand::PreparedGlyphRun {
                run: outline,
                transform: run_transform,
                paint,
            } => {
                let transform = transform * kurbo::Affine::from(*run_transform);
                crate::text::qualify_glyph_placement(transform, outline, Some(account))?;
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
                                None => glyph_paint_brush(paint.fill(), account)?,
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
                                    brush.as_ref(),
                                    None,
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
                        } => {
                            // The admitted endpoint quad is the geometry. A
                            // rounded sampling matrix cannot rebuild/expand it.
                            with_sampled_image(
                                scene,
                                image,
                                transform * *brush_transform,
                                |scene, brush, compensation| {
                                    scene.fill(
                                        peniko::Fill::NonZero,
                                        transform,
                                        brush,
                                        Some(
                                            *brush_transform
                                                * compensation.unwrap_or(kurbo::Affine::IDENTITY),
                                        ),
                                        &kurbo::Rect::from(*rect),
                                        account,
                                    )?;
                                    Ok(())
                                },
                                account,
                            )?;
                        }
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
                account,
            )?,
            RenderCommand::Layer { layer, children } => {
                let layer_transform = transform * kurbo::Affine::from(layer.transform);
                if layer.isolation != LayerIsolation::None {
                    encode_layer_start(scene, layer, layer_transform, account)?;
                    encode_vello_commands(children, scene, layer_transform, account)?;
                    scene.pop_layer(account)?;
                } else {
                    encode_vello_commands(children, scene, layer_transform, account)?;
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
    account: &Account,
) -> Result<()> {
    account.work(RenderWork::PaintVisits, 1)?;
    if let RenderPaint::Image(image) = paint {
        return with_sampled_image(
            scene,
            image,
            transform,
            |scene, brush, brush_transform| {
                encode_solid_fill(
                    scene,
                    transform,
                    shape,
                    &peniko::Brush::Image(brush.clone()),
                    brush_transform,
                    account,
                )?;
                Ok(())
            },
            account,
        );
    }
    let brush = render_paint_brush(paint, account)?;
    encode_solid_fill(scene, transform, shape, &brush, None, account)?;
    Ok(())
}

fn encode_solid_fill(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    shape: &RenderShape,
    brush: &peniko::Brush,
    brush_transform: Option<kurbo::Affine>,
    account: &Account,
) -> Result<()> {
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
                    &peniko::Brush::Image(brush.clone()),
                    brush_transform,
                    account,
                )?;
                Ok(())
            },
            account,
        );
    }
    let brush = render_paint_brush(paint, account)?;
    encode_brush_stroke(scene, transform, shape, stroke, &brush, None, account)?;
    Ok(())
}

fn encode_brush_stroke(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    shape: &RenderStrokeShape,
    stroke: &RenderStroke,
    brush: &peniko::Brush,
    brush_transform: Option<kurbo::Affine>,
    account: &Account,
) -> Result<()> {
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
        let brush = peniko::Brush::Solid(color.into());
        encode_solid_fill(
            scene,
            transform,
            &RenderShape::RoundedRect { rect, radii },
            &brush,
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

fn encode_image(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    image: &Image,
    rect: Rect,
    fit: ImageFit,
    account: &Account,
) -> Result<()> {
    let image_transform = transform * image_transform(image.size, rect, fit)?;
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
    encode: impl FnOnce(&mut VelloScene, &peniko::ImageBrush, Option<kurbo::Affine>) -> Result<T>,
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
        encode(scene, &brush, None)
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

/// Stack custody for copied text values and their independently admitted backing.
/// The font exists before paint cloning can fail; the completed run subsequently
/// owns both allocations. One disposal phase surrounds the actual value drop.
struct FrameTextValue<T> {
    value: Option<T>,
    backing: [Option<Hold>; 2],
}

impl<T> FrameTextValue<T> {
    fn new(value: T, backing: [Option<Hold>; 2]) -> Self {
        Self {
            value: Some(value),
            backing,
        }
    }

    fn into_parts(mut self) -> (T, [Option<Hold>; 2]) {
        (
            self.value.take().expect("owned text value"),
            std::mem::take(&mut self.backing),
        )
    }
}

impl<T> std::ops::Deref for FrameTextValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().expect("owned text value")
    }
}

impl<T> Drop for FrameTextValue<T> {
    fn drop(&mut self) {
        let value = self.value.take();
        let [first, second] = std::mem::take(&mut self.backing);
        if let Some(first) = first {
            first.dispose(|| {
                drop(value);
                drop(second);
            });
        } else if let Some(second) = second {
            second.dispose(|| drop(value));
        } else {
            drop(value);
        }
    }
}

fn encode_text_run(
    scene: &mut VelloScene,
    transform: kurbo::Affine,
    source: TextRunSource<'_>,
    account: &Account,
) -> Result<()> {
    let TextRunSource {
        font,
        size,
        transform: run_transform,
        paint,
        glyphs,
    } = source;
    let (font, font_name_hold) = font.try_to_owned_in(account)?;
    let font = FrameTextValue::new(font, [font_name_hold, None]);
    let (paint, paint_hold) = paint.try_clone_in(account)?;
    // Only infallible moves remain between disarming the font guard and arming
    // the complete run guard; a failed paint copy still drops the guarded font.
    let (font, [font_name_hold, _]) = font.into_parts();
    let run = FrameTextValue::new(
        TextRun::from_validated_parts(
            font,
            size,
            run_transform,
            paint,
            glyphs,
            TextRunBounds::unspecified(),
        ),
        [font_name_hold, paint_hold],
    );
    scene.encode_text_run_with_transform(&run, transform, account)
}

fn encode_layer_start(
    scene: &mut VelloScene,
    layer: &NormalizedLayer,
    transform: kurbo::Affine,
    account: &Account,
) -> Result<()> {
    let default_clip = RenderClipGeometry::Rect(Rect::new(-1.0e9, -1.0e9, 2.0e9, 2.0e9));
    let (clip, clip_transform) = layer
        .clip
        .as_ref()
        .map(|clip| {
            let clip_transform = clip
                .coordinate_space()
                .map(|coordinate_space| {
                    transform * kurbo::Affine::from(coordinate_space.transform())
                })
                .unwrap_or(transform);
            (clip.geometry(), clip_transform)
        })
        .unwrap_or((&default_clip, transform));
    match clip {
        RenderClipGeometry::Rect(rect) => push_vello_layer(
            scene,
            layer,
            peniko::Fill::NonZero,
            clip_transform,
            &kurbo::Rect::from(*rect),
            account,
        ),
        RenderClipGeometry::RoundedRect { rect, radii } => push_vello_layer(
            scene,
            layer,
            peniko::Fill::NonZero,
            clip_transform,
            &kurbo_rounded_rect(*rect, *radii),
            account,
        ),
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
            vello_blend(layer.blend),
            layer.opacity.clamp(0.0, 1.0),
            transform,
            shape,
            account,
        )
    }
}

pub(crate) struct FrameBrush {
    brush: peniko::Brush,
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
            _hold: None,
            _payload: None,
        }
    }
    fn image(image: &Image, account: &Account) -> Result<Self> {
        let payload = account.import_image(image)?;
        Ok(Self {
            brush: peniko::Brush::Image(image_brush(image)),
            _hold: None,
            _payload: payload,
        })
    }
    pub(crate) fn as_ref(&self) -> peniko::BrushRef<'_> {
        (&self.brush).into()
    }
}
impl std::ops::Deref for FrameBrush {
    type Target = peniko::Brush;
    fn deref(&self) -> &Self::Target {
        &self.brush
    }
}

pub(crate) fn glyph_paint_brush(paint: &Paint, account: &Account) -> Result<FrameBrush> {
    account.work(RenderWork::PaintVisits, 1)?;
    match paint.kind() {
        PaintKind::Color(color) => Ok(FrameBrush::solid((*color).into())),
        PaintKind::Gradient(gradient) => gradient_brush(gradient, account),
        PaintKind::Image(image) => FrameBrush::image(image, account),
    }
}
fn render_paint_brush(paint: &RenderPaint, account: &Account) -> Result<FrameBrush> {
    account.work(RenderWork::PaintVisits, 1)?;
    match paint {
        RenderPaint::Color(color) => Ok(FrameBrush::solid((*color).into())),
        RenderPaint::Gradient(gradient) => gradient_brush(gradient, account),
        RenderPaint::Image(image) => FrameBrush::image(image, account),
    }
}
fn gradient_brush(gradient: &Gradient, account: &Account) -> Result<FrameBrush> {
    let mut stops = account.temporary(gradient.stops().len())?;
    for stop in gradient.stops() {
        account.work(RenderWork::GradientStopVisits, 1)?;
        stops.push(peniko::ColorStop::from((
            stop.offset(),
            peniko::Color::from(stop.color()),
        )))?;
    }
    let (stops, hold) = stops.into_parts();
    let mut output = match gradient.kind() {
        crate::paint::GradientKind::Linear { start, end, .. } => peniko::Gradient::new_linear(
            (start.x() as f32, start.y() as f32),
            (end.x() as f32, end.y() as f32),
        ),
        crate::paint::GradientKind::Radial { center, radius, .. } => {
            peniko::Gradient::new_radial((center.x() as f32, center.y() as f32), *radius as f32)
        }
        crate::paint::GradientKind::Sweep { center, .. } => peniko::Gradient::new_sweep(
            (center.x() as f32, center.y() as f32),
            0.0,
            std::f32::consts::TAU,
        ),
    };
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
        let (intervals, backing) = intervals.into_parts();
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

fn vello_blend(blend: BlendMode) -> peniko::BlendMode {
    match blend {
        BlendMode::Normal => peniko::Mix::Normal.into(),
        BlendMode::Multiply => peniko::Mix::Multiply.into(),
        BlendMode::Screen => peniko::Mix::Screen.into(),
        BlendMode::Overlay => peniko::Mix::Overlay.into(),
        BlendMode::Darken => peniko::Mix::Darken.into(),
        BlendMode::Lighten => peniko::Mix::Lighten.into(),
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

pub(crate) fn image_transform(size: Size, rect: Rect, fit: ImageFit) -> Result<kurbo::Affine> {
    if size.width() <= 0.0 || size.height() <= 0.0 {
        return Err(Error::new(
            BackendErrorCode::ImageUploadFailed,
            "image size must be positive",
        ));
    }
    let scale_x = rect.width() / size.width();
    let scale_y = rect.height() / size.height();
    let (fit_scale_x, fit_scale_y, tx, ty) = match fit {
        ImageFit::Fill | ImageFit::Stretch | ImageFit::None => {
            (scale_x, scale_y, rect.x(), rect.y())
        }
        ImageFit::Contain => {
            let scale = scale_x.min(scale_y);
            (
                scale,
                scale,
                rect.x() + (rect.width() - size.width() * scale) * 0.5,
                rect.y() + (rect.height() - size.height() * scale) * 0.5,
            )
        }
        ImageFit::Cover => {
            let scale = scale_x.max(scale_y);
            (
                scale,
                scale,
                rect.x() + (rect.width() - size.width() * scale) * 0.5,
                rect.y() + (rect.height() - size.height() * scale) * 0.5,
            )
        }
    };
    Ok(kurbo::Affine::translate((tx, ty))
        * kurbo::Affine::scale_non_uniform(fit_scale_x, fit_scale_y))
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
                            glyph_paint_brush(&paint, account).map(drop)
                        } else {
                            render_paint_brush(render_paint, account).map(drop)
                        }
                    })
                    .unwrap();
                assert_eq!(
                    context
                        .last_attempt()
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
                            glyph_paint_brush(&paint, account).map(drop)
                        } else {
                            render_paint_brush(render_paint, account).map(drop)
                        }
                    })
                    .unwrap_err();
                refusal(&error, 0, 0);
                let usage = context.last_attempt().unwrap().usage();
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
                        account,
                    )
                    .map(drop)
                })
                .unwrap_err();
            refusal(&error, limit, limit);
            let usage = context.last_attempt().unwrap().usage();
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
        use crate::accounting::{FrameAttempt, FrameReportSlot, RendererCpu};
        use std::{
            sync::{
                Arc,
                atomic::{AtomicU64, AtomicUsize, Ordering},
            },
            time::Duration,
        };

        static DISPOSAL_TIME: AtomicU64 = AtomicU64::new(0);

        fn clock() -> Duration {
            Duration::from_millis(DISPOSAL_TIME.load(Ordering::SeqCst))
        }

        fn forbidden_clock() -> Duration {
            panic!("untimed text-value disposal read the clock");
        }

        struct DropProbe {
            buffers: [Vec<u8>; 2],
            cpu: RendererCpu,
            drops: Arc<AtomicUsize>,
            admitted: u64,
        }

        impl Drop for DropProbe {
            fn drop(&mut self) {
                assert_eq!(
                    self.cpu
                        .snapshot()
                        .unwrap()
                        .usage()
                        .live(RenderResource::RetainedCpuBytes),
                    self.admitted,
                );
                drop(std::mem::take(&mut self.buffers));
                // Both real allocations have been destroyed, but their paired
                // admissions must still cover the complete value destructor.
                assert_eq!(
                    self.cpu
                        .snapshot()
                        .unwrap()
                        .usage()
                        .live(RenderResource::RetainedCpuBytes),
                    self.admitted,
                );
                DISPOSAL_TIME.fetch_add(7, Ordering::SeqCst);
                self.drops.fetch_add(1, Ordering::SeqCst);
            }
        }

        fn probe(
            cpu: &RendererCpu,
            drops: &Arc<AtomicUsize>,
            count: usize,
        ) -> FrameTextValue<DropProbe> {
            let opening = cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes);
            let buffers = [
                if count > 0 { vec![1; 3] } else { Vec::new() },
                if count > 1 { vec![2; 5] } else { Vec::new() },
            ];
            let account = cpu.at(RenderCpuScope::FrameBacking);
            let backing = std::array::from_fn(|index| {
                (index < count).then(|| {
                    Hold::Renderer(
                        account
                            .acquire(RenderResource::RetainedCpuBytes, buffers[index].capacity())
                            .unwrap(),
                    )
                })
            });
            let admitted = opening + buffers.iter().map(Vec::capacity).sum::<usize>() as u64;
            FrameTextValue::new(
                DropProbe {
                    buffers,
                    cpu: cpu.clone(),
                    drops: drops.clone(),
                    admitted,
                },
                backing,
            )
        }

        #[derive(Clone, Copy, Debug)]
        enum Exit {
            Drop,
            Error,
            Unwind,
            Transfer,
        }

        #[test]
        fn text_value_disposal_keeps_all_backing_live_through_one_actual_destructor() {
            for profile in [
                RenderProfile::Off,
                RenderProfile::Summary { timing: false },
                RenderProfile::Summary { timing: true },
            ] {
                for (count, exit) in [
                    (0, Exit::Drop),
                    (1, Exit::Drop),
                    (1, Exit::Error),
                    (2, Exit::Drop),
                    (2, Exit::Unwind),
                    (2, Exit::Transfer),
                ] {
                    DISPOSAL_TIME.store(0, Ordering::SeqCst);
                    let timed = matches!(profile, RenderProfile::Summary { timing: true });
                    let options = RenderAccountingOptions::new()
                        .with_profile(profile)
                        .with_clock(RenderClock::Host(if timed {
                            clock
                        } else {
                            forbidden_clock
                        }));
                    let cpu = RendererCpu::new(options).unwrap();
                    let checkpoint = cpu.begin_interval().unwrap();
                    let drops = Arc::new(AtomicUsize::new(0));
                    match exit {
                        Exit::Drop => drop(probe(&cpu, &drops, count)),
                        Exit::Error => {
                            let result = (|| -> Result<()> {
                                let _value = probe(&cpu, &drops, count);
                                Err(Error::invalid_value(
                                    "text-value fixture",
                                    "refused",
                                    "must return the original error",
                                ))
                            })();
                            assert_eq!(result.unwrap_err().code(), ErrorCode::InvalidInput);
                        }
                        Exit::Unwind => {
                            assert!(
                                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                    let _value = probe(&cpu, &drops, count);
                                    panic!("text-value fixture unwind");
                                }))
                                .is_err()
                            );
                        }
                        Exit::Transfer => {
                            let (value, backing) = probe(&cpu, &drops, count).into_parts();
                            assert_eq!(drops.load(Ordering::SeqCst), 0);
                            drop(FrameTextValue::new((value,), backing));
                        }
                    }
                    assert_eq!(
                        drops.load(Ordering::SeqCst),
                        1,
                        "{count} backings, {exit:?}"
                    );
                    let interval = checkpoint.finish().unwrap();
                    assert_eq!(
                        interval.usage().live(RenderResource::RetainedCpuBytes),
                        interval
                            .usage()
                            .opening_live(RenderResource::RetainedCpuBytes),
                    );
                    if timed {
                        // The destructor advances the clock by seven milliseconds.
                        // Two nested disposal phases around it would report fourteen.
                        assert_eq!(
                            interval
                                .scope_elapsed(RenderCpuScope::FrameBacking)
                                .unwrap_or_default(),
                            Duration::from_millis(if count == 0 { 0 } else { 7 }),
                            "{count} backings, {exit:?}",
                        );
                    } else {
                        assert_eq!(interval.scope_elapsed(RenderCpuScope::FrameBacking), None);
                    }
                }
            }
        }

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
            let attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
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
            drop(attempt);
        }
    }
}

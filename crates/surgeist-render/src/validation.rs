use super::*;
use super::{geometry::validate_rect_maxima, paint::PaintKind, shape::ShapeKind};
use crate::accounting::{Account, RenderWork};

pub(crate) fn validate_path_in(path: &Path, account: &Account) -> Result<()> {
    for element in path.elements() {
        account.work(RenderWork::PathElementVisits, 1)?;
        match *element {
            PathElement::MoveTo(point) | PathElement::LineTo(point) => {
                validate_point(point, "path point")?
            }
            PathElement::QuadTo(control, point) => {
                validate_point(control, "path control point")?;
                validate_point(point, "path point")?;
            }
            PathElement::CubicTo(a, b, point) => {
                validate_point(a, "path control point")?;
                validate_point(b, "path control point")?;
                validate_point(point, "path point")?;
            }
            PathElement::Close => {}
        }
    }
    Ok(())
}
pub(crate) fn validate_shape_in(shape: &Shape, account: &Account) -> Result<()> {
    match shape.kind() {
        ShapeKind::Path(path) => validate_path_in(path, account),
        _ => validate_shape(shape),
    }
}
pub(crate) fn validate_paint_in(paint: &Paint, account: &Account) -> Result<()> {
    account.work(RenderWork::PaintVisits, 1)?;
    validate_paint(paint)
}

pub(crate) fn validate_shadow_in(shadow: &Shadow, account: &Account) -> Result<()> {
    validate_point(shadow.offset(), "shadow offset")?;
    validate_non_negative_f64(shadow.blur(), "shadow blur")?;
    validate_finite_f64(shadow.spread(), "shadow spread")?;
    validate_paint_in(shadow.paint(), account)
}
pub(crate) fn validate_text_run_in(
    size: f32,
    transform: Transform,
    glyphs: &[TextGlyph],
    account: &Account,
) -> Result<()> {
    validate_text_run(size, transform, &[])?;
    for glyph in glyphs {
        account.work(RenderWork::GlyphVisits, 1)?;
        if !glyph.x().is_finite() || !glyph.y().is_finite() || !glyph.advance().is_finite() {
            return Err(invalid_input(
                "text glyph positions and advances must be finite",
            ));
        }
    }
    Ok(())
}
pub(crate) fn validate_layer_in(layer: &Layer, account: &Account) -> Result<()> {
    validate_transform(layer.transform(), "layer transform")?;
    if !layer.opacity().is_finite() {
        return Err(invalid_input("layer opacity must be finite"));
    }
    if let Some(clip) = layer.clip_input() {
        clip.validate_in(account)?;
    }
    if let Some(mask) = layer.mask() {
        validate_shape_in(mask, account)?;
    }
    if let Some(filter) = layer.filter() {
        validate_filter(filter)?;
    }
    if let Some(backdrop) = layer.backdrop_filter() {
        backdrop.ensure_supported_for_planning_in(Capabilities::CURRENT, account)?;
    }
    Ok(())
}

pub(crate) fn validate_surface_options(options: SurfaceOptions) -> Result<()> {
    validate_size(options.size, "surface size")?;
    validate_positive_f64(options.scale, "surface scale")
}

pub(crate) fn validate_shape(shape: &Shape) -> Result<()> {
    match shape.kind() {
        ShapeKind::Rect(rect) => validate_rect(*rect, "rectangle"),
        ShapeKind::RoundedRect { rect, radii } => {
            validate_rect(*rect, "rounded rectangle")?;
            validate_radii(*radii, "rounded rectangle radii")
        }
        ShapeKind::Circle { center, radius } => {
            validate_point(*center, "circle center")?;
            validate_non_negative_f64(*radius, "circle radius")
        }
        ShapeKind::Ellipse { center, radii } => {
            validate_point(*center, "ellipse center")?;
            validate_size(*radii, "ellipse radii")
        }
        ShapeKind::Path(path) => validate_path(path),
    }
}

pub(crate) fn validate_path(path: &Path) -> Result<()> {
    for element in path.elements() {
        match *element {
            PathElement::MoveTo(point) | PathElement::LineTo(point) => {
                validate_point(point, "path point")?;
            }
            PathElement::QuadTo(control, point) => {
                validate_point(control, "path control point")?;
                validate_point(point, "path point")?;
            }
            PathElement::CubicTo(a, b, point) => {
                validate_point(a, "path control point")?;
                validate_point(b, "path control point")?;
                validate_point(point, "path point")?;
            }
            PathElement::Close => {}
        }
    }
    Ok(())
}

pub(crate) fn validate_stroke(stroke: Stroke) -> Result<()> {
    validate_positive_f64(stroke.width(), "stroke width")?;
    validate_positive_f64(stroke.miter_limit(), "stroke miter limit")?;
    // The immutable Dash constructor owns authored interval validity. The
    // actual native receiver qualifies its separate algorithm capacity.
    Ok(())
}

pub(crate) fn validate_paint(paint: &Paint) -> Result<()> {
    match paint.kind() {
        PaintKind::Color(color) => validate_color(*color, "color paint"),
        // Canonical constructors establish the immutable gradient invariants.
        PaintKind::Gradient(_) | PaintKind::RepeatingGradient(_) => Ok(()),
        PaintKind::Image(image) => validate_size(image.size(), "image size"),
    }
}

pub(crate) fn validate_shadow(shadow: &Shadow) -> Result<()> {
    validate_point(shadow.offset(), "shadow offset")?;
    validate_non_negative_f64(shadow.blur(), "shadow blur")?;
    validate_finite_f64(shadow.spread(), "shadow spread")?;
    validate_paint(shadow.paint())
}

pub(crate) fn validate_filter(filter: Filter) -> Result<()> {
    validate_non_negative_f64(filter.blur_radius(), "layer blur radius")
}

pub(crate) fn validate_text_run(
    size: f32,
    transform: Transform,
    glyphs: &[TextGlyph],
) -> Result<()> {
    if !size.is_finite() || size <= 0.0 {
        return Err(invalid_input(
            "text run size must be finite and greater than 0",
        ));
    }
    validate_transform(transform, "text transform")?;
    for glyph in glyphs {
        if !glyph.x().is_finite() || !glyph.y().is_finite() || !glyph.advance().is_finite() {
            return Err(invalid_input(
                "text glyph positions and advances must be finite",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_rect(rect: Rect, name: &str) -> Result<()> {
    validate_point(rect.origin(), name)?;
    validate_size(rect.size(), name)?;
    validate_rect_maxima(rect, name)
}

pub(crate) fn validate_size(size: Size, name: &str) -> Result<()> {
    validate_non_negative_f64(size.width(), &format!("{name} width"))?;
    validate_non_negative_f64(size.height(), &format!("{name} height"))
}

pub(crate) fn validate_radii(radii: Radii, name: &str) -> Result<()> {
    validate_non_negative_f64(radii.top_left(), &format!("{name} top-left"))?;
    validate_non_negative_f64(radii.top_right(), &format!("{name} top-right"))?;
    validate_non_negative_f64(radii.bottom_right(), &format!("{name} bottom-right"))?;
    validate_non_negative_f64(radii.bottom_left(), &format!("{name} bottom-left"))
}

pub(crate) fn validate_point(point: Point, name: &str) -> Result<()> {
    validate_finite_f64(point.x(), &format!("{name} x"))?;
    validate_finite_f64(point.y(), &format!("{name} y"))
}

pub(crate) fn validate_transform(transform: Transform, name: &str) -> Result<()> {
    for value in transform.as_array() {
        validate_finite_f64(value, name)?;
    }
    Ok(())
}

pub(crate) fn validate_color(color: Color, name: &str) -> Result<()> {
    for (channel, value) in [
        ("red", color.r()),
        ("green", color.g()),
        ("blue", color.b()),
        ("alpha", color.a()),
    ] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(Error::invalid_value(
                format!("{name} {channel} channel"),
                value,
                "must be finite and between 0 and 1",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_positive_f64(value: f64, name: &str) -> Result<()> {
    if !value.is_finite() || value <= 0.0 {
        return Err(Error::invalid_value(
            name,
            value,
            "must be finite and greater than 0",
        ));
    }
    Ok(())
}

pub(crate) fn validate_non_negative_f64(value: f64, name: &str) -> Result<()> {
    if !value.is_finite() || value < 0.0 {
        return Err(Error::invalid_value(
            name,
            value,
            "must be finite and non-negative",
        ));
    }
    Ok(())
}

pub(crate) fn validate_finite_f64(value: f64, name: &str) -> Result<()> {
    if !value.is_finite() {
        return Err(Error::invalid_value(name, value, "must be finite"));
    }
    Ok(())
}

pub(crate) fn invalid_input(message: impl Into<String>) -> Error {
    Error::invalid_input_message(message)
}

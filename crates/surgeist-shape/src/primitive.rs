use super::*;
use kurbo::Shape as KurboShape;

#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    kind: ShapeKind,
}

#[derive(Clone, Debug, PartialEq)]
enum ShapeKind {
    Rect(Rect),
    RoundedRect { rect: Rect, radii: Radii },
    Circle { center: Point, radius: NonNegative },
    Ellipse { center: Point, radii: Size },
    Path { path: Path, fill_rule: FillRule },
}

impl Shape {
    pub fn try_rect(rect: Rect) -> Result<Self> {
        rect.validate("rect")?;
        Ok(Self {
            kind: ShapeKind::Rect(rect),
        })
    }

    #[must_use]
    pub const fn rect(rect: Rect) -> Self {
        Self {
            kind: ShapeKind::Rect(rect),
        }
    }

    pub fn try_rounded_rect(rect: Rect, radii: Radii) -> Result<Self> {
        rect.validate("rounded rect")?;
        radii.normalized_for(rect)?;
        Ok(Self {
            kind: ShapeKind::RoundedRect { rect, radii },
        })
    }

    pub fn try_circle(center: Point, radius: f64) -> Result<Self> {
        center.validate("circle center")?;
        let radius = NonNegative::try_new(radius, NumericKind::Radius)?;
        Ok(Self {
            kind: ShapeKind::Circle { center, radius },
        })
    }

    pub fn try_ellipse(center: Point, radii: Size) -> Result<Self> {
        center.validate("ellipse center")?;
        radii.validate("ellipse radii")?;
        Ok(Self {
            kind: ShapeKind::Ellipse { center, radii },
        })
    }

    pub fn try_path(path: Path, fill_rule: FillRule) -> Result<Self> {
        path.validate()?;
        Ok(Self {
            kind: ShapeKind::Path { path, fill_rule },
        })
    }

    pub fn validate(&self) -> Result<()> {
        match &self.kind {
            ShapeKind::Rect(rect) => rect.validate("rect"),
            ShapeKind::RoundedRect { rect, radii } => {
                rect.validate("rounded rect")?;
                radii.normalized_for(*rect).map(|_| ())
            }
            ShapeKind::Circle { center, radius } => {
                center.validate("circle center")?;
                NonNegative::try_new(radius.get(), NumericKind::Radius).map(|_| ())
            }
            ShapeKind::Ellipse { center, radii } => {
                center.validate("ellipse center")?;
                radii.validate("ellipse radii")
            }
            ShapeKind::Path { path, .. } => path.validate(),
        }
    }

    pub fn bounds(&self) -> Rect {
        match &self.kind {
            ShapeKind::Rect(rect) | ShapeKind::RoundedRect { rect, .. } => *rect,
            ShapeKind::Circle { center, radius } => Rect::new(
                center.x() - radius.get(),
                center.y() - radius.get(),
                radius.get() * 2.0,
                radius.get() * 2.0,
            ),
            ShapeKind::Ellipse { center, radii } => Rect::new(
                center.x() - radii.width(),
                center.y() - radii.height(),
                radii.width() * 2.0,
                radii.height() * 2.0,
            ),
            ShapeKind::Path { path, .. } => path.bounds(),
        }
    }

    pub fn visual_bounds(&self, stroke: Option<Stroke>) -> Result<Rect> {
        let Some(stroke) = stroke else {
            return Ok(self.bounds());
        };
        stroke.validate()?;
        let outset = match stroke.align() {
            StrokeAlign::Center => stroke.width() * 0.5,
            StrokeAlign::Inside => 0.0,
            StrokeAlign::Outside => stroke.width(),
        };
        if matches!(&self.kind, ShapeKind::Path { .. }) && stroke.align() != StrokeAlign::Center {
            return Err(Error::new(
                ErrorCode::UnsupportedStrokeBounds,
                "inside and outside stroke bounds for arbitrary paths are not supported",
            ));
        }
        Ok(self.bounds().outset(Insets::all(outset)))
    }

    pub fn support_bounds(&self, outset: Insets) -> Result<Rect> {
        outset.validate("support outset")?;
        Ok(self.bounds().outset(outset))
    }

    pub fn transformed_bounds(&self, transform: Transform) -> Result<Rect> {
        transform.try_apply_rect(self.bounds())
    }

    pub fn contains(&self, point: Point) -> bool {
        match &self.kind {
            ShapeKind::Rect(rect) => rect.contains(point),
            ShapeKind::RoundedRect { rect, radii } => rounded_rect_contains(*rect, *radii, point),
            ShapeKind::Circle { center, radius } => center.distance_to(point) <= radius.get(),
            ShapeKind::Ellipse { center, radii } => {
                if radii.width() <= 0.0 || radii.height() <= 0.0 {
                    return false;
                }
                let x = (point.x() - center.x()) / radii.width();
                let y = (point.y() - center.y()) / radii.height();
                x * x + y * y <= 1.0
            }
            ShapeKind::Path { path, fill_rule } => path.contains(point, *fill_rule),
        }
    }

    pub fn inflate(&self, amount: f64) -> Result<Self> {
        validate_finite(amount, "inflate amount")?;
        match &self.kind {
            ShapeKind::Rect(rect) => Self::try_rect(outset_rect_clamped(*rect, amount)?),
            ShapeKind::RoundedRect { rect, radii } => {
                Self::try_rounded_rect(outset_rect_clamped(*rect, amount)?, radii.outset(amount))
            }
            ShapeKind::Circle { center, radius } => {
                Self::try_circle(*center, (radius.get() + amount).max(0.0))
            }
            ShapeKind::Ellipse { center, radii } => Self::try_ellipse(
                *center,
                Size::try_new(
                    (radii.width() + amount).max(0.0),
                    (radii.height() + amount).max(0.0),
                )?,
            ),
            ShapeKind::Path { .. } => Err(Error::new(
                ErrorCode::UnsupportedStrokeBounds,
                "inflating arbitrary paths is not supported",
            )),
        }
    }

    pub fn deflate(&self, amount: f64) -> Result<Self> {
        self.inflate(-amount)
    }

    pub fn to_path(&self) -> Result<Path> {
        Ok(Path::from_kurbo(self.to_kurbo_path()?))
    }

    pub fn to_kurbo_path(&self) -> Result<kurbo::BezPath> {
        self.validate()?;
        Ok(match &self.kind {
            ShapeKind::Rect(rect) => rect.to_kurbo().to_path(PATH_TOLERANCE),
            ShapeKind::RoundedRect { rect, radii } => {
                let radii = radii.normalized_for(*rect)?;
                kurbo::RoundedRect::from_rect(rect.to_kurbo(), radii.to_kurbo())
                    .to_path(PATH_TOLERANCE)
            }
            ShapeKind::Circle { center, radius } => {
                kurbo::Circle::new(center.to_kurbo(), radius.get()).to_path(PATH_TOLERANCE)
            }
            ShapeKind::Ellipse { center, radii } => {
                kurbo::Ellipse::new(center.to_kurbo(), (radii.width(), radii.height()), 0.0)
                    .to_path(PATH_TOLERANCE)
            }
            ShapeKind::Path { path, .. } => path.to_kurbo(),
        })
    }

    #[must_use]
    pub fn to_kurbo_rect(&self) -> Option<kurbo::Rect> {
        match &self.kind {
            ShapeKind::Rect(rect) => Some(rect.to_kurbo()),
            _ => None,
        }
    }

    pub fn to_kurbo_rounded_rect(&self) -> Option<kurbo::RoundedRect> {
        match &self.kind {
            ShapeKind::RoundedRect { rect, radii } => Some(kurbo::RoundedRect::from_rect(
                rect.to_kurbo(),
                radii.normalized_for(*rect).ok()?.to_kurbo(),
            )),
            _ => None,
        }
    }

    #[must_use]
    pub fn key(&self) -> Key {
        let mut state = StableHasher::new(0xcbf2_9ce4_8422_2325);
        match &self.kind {
            ShapeKind::Rect(rect) => {
                state.write_u8(1);
                hash_rect(&mut state, *rect);
            }
            ShapeKind::RoundedRect { rect, radii } => {
                state.write_u8(2);
                hash_rect(&mut state, *rect);
                hash_radii(
                    &mut state,
                    radii
                        .normalized_for(*rect)
                        .expect("shape rounded rect was validated at construction"),
                );
            }
            ShapeKind::Circle { center, radius } => {
                state.write_u8(3);
                hash_point(&mut state, *center);
                state.write_f64(radius.get());
            }
            ShapeKind::Ellipse { center, radii } => {
                state.write_u8(4);
                hash_point(&mut state, *center);
                state.write_f64(radii.width());
                state.write_f64(radii.height());
            }
            ShapeKind::Path { path, fill_rule } => {
                state.write_u8(5);
                path.hash_with_rule(&mut state, *fill_rule);
            }
        }
        Key::from_parts(
            state.finish(),
            state.finish_with_seed(0x9e37_79b9_7f4a_7c15),
        )
    }

    pub fn dashed_stroke(&self, stroke: Stroke) -> Result<DashGeometry> {
        stroke.validate()?;
        let dash = stroke
            .dash()
            .ok_or_else(|| Error::new(ErrorCode::InvalidDash, "stroke has no dash geometry"))?;
        dash.validate()?;
        match &self.kind {
            ShapeKind::Rect(rect) => dash_rect(*rect, Radii::zero(), stroke),
            ShapeKind::RoundedRect { rect, radii } => {
                dash_rect(*rect, radii.normalized_for(*rect)?, stroke)
            }
            ShapeKind::Circle { center, radius } => {
                dash_ellipse(*center, Size::new(radius.get(), radius.get()), stroke)
            }
            ShapeKind::Ellipse { center, radii } => dash_ellipse(*center, *radii, stroke),
            ShapeKind::Path { .. } => Err(Error::new(
                ErrorCode::InvalidDash,
                "dash geometry for arbitrary paths is not part of the first implementation",
            )),
        }
    }
}

fn outset_rect_clamped(rect: Rect, amount: f64) -> Result<Rect> {
    Rect::try_new(
        rect.origin().x() - amount,
        rect.origin().y() - amount,
        (rect.width() + amount * 2.0).max(0.0),
        (rect.height() + amount * 2.0).max(0.0),
    )
}

fn rounded_rect_contains(rect: Rect, radii: Radii, point: Point) -> bool {
    if !rect.contains(point) {
        return false;
    }
    let radii = radii.normalized_for(rect).unwrap_or(radii);
    let max = rect.max();
    for (corner, radius, cx, cy) in [
        (
            Corner::TopLeft,
            radii.top_left(),
            rect.origin().x() + radii.top_left(),
            rect.origin().y() + radii.top_left(),
        ),
        (
            Corner::TopRight,
            radii.top_right(),
            max.x() - radii.top_right(),
            rect.origin().y() + radii.top_right(),
        ),
        (
            Corner::BottomRight,
            radii.bottom_right(),
            max.x() - radii.bottom_right(),
            max.y() - radii.bottom_right(),
        ),
        (
            Corner::BottomLeft,
            radii.bottom_left(),
            rect.origin().x() + radii.bottom_left(),
            max.y() - radii.bottom_left(),
        ),
    ] {
        if radius <= 0.0 {
            continue;
        }
        let in_corner = match corner {
            Corner::TopLeft => point.x() < cx && point.y() < cy,
            Corner::TopRight => point.x() > cx && point.y() < cy,
            Corner::BottomRight => point.x() > cx && point.y() > cy,
            Corner::BottomLeft => point.x() < cx && point.y() > cy,
        };
        if in_corner && Point::new(cx, cy).distance_to(point) > radius {
            return false;
        }
    }
    true
}

use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    width: f64,
    join: LineJoin,
    start_cap: LineCap,
    end_cap: LineCap,
    miter_limit: f64,
    dash: Option<Dash>,
    align: StrokeAlign,
}

impl Stroke {
    pub fn try_new(width: f64) -> Result<Self> {
        validate_non_negative_kind(width, "stroke width", NumericKind::Stroke)?;
        Ok(Self {
            width,
            ..Self::default()
        })
    }

    #[must_use]
    pub fn hairline() -> Self {
        Self::default()
    }

    pub fn try_centered(width: f64) -> Result<Self> {
        Ok(Self {
            align: StrokeAlign::Center,
            ..Self::try_new(width)?
        })
    }

    pub fn try_inside(width: f64) -> Result<Self> {
        Ok(Self {
            align: StrokeAlign::Inside,
            ..Self::try_new(width)?
        })
    }

    pub fn try_outside(width: f64) -> Result<Self> {
        Ok(Self {
            align: StrokeAlign::Outside,
            ..Self::try_new(width)?
        })
    }

    #[must_use]
    pub const fn with_join(mut self, join: LineJoin) -> Self {
        self.join = join;
        self
    }

    #[must_use]
    pub const fn with_caps(mut self, start_cap: LineCap, end_cap: LineCap) -> Self {
        self.start_cap = start_cap;
        self.end_cap = end_cap;
        self
    }

    pub fn try_with_miter_limit(mut self, miter_limit: f64) -> Result<Self> {
        validate_non_negative_kind(miter_limit, "stroke miter limit", NumericKind::Stroke)?;
        self.miter_limit = miter_limit;
        Ok(self)
    }

    #[must_use]
    pub const fn with_dash(mut self, dash: Dash) -> Self {
        self.dash = Some(dash);
        self
    }

    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }

    #[must_use]
    pub const fn join(self) -> LineJoin {
        self.join
    }

    #[must_use]
    pub const fn start_cap(self) -> LineCap {
        self.start_cap
    }

    #[must_use]
    pub const fn end_cap(self) -> LineCap {
        self.end_cap
    }

    #[must_use]
    pub const fn miter_limit(self) -> f64 {
        self.miter_limit
    }

    #[must_use]
    pub const fn align(self) -> StrokeAlign {
        self.align
    }

    #[must_use]
    pub const fn dash(self) -> Option<Dash> {
        self.dash
    }

    pub fn validate(self) -> Result<()> {
        validate_non_negative_kind(self.width, "stroke width", NumericKind::Stroke)?;
        validate_non_negative_kind(self.miter_limit, "stroke miter limit", NumericKind::Stroke)?;
        if let Some(dash) = self.dash {
            dash.validate()?;
        }
        Ok(())
    }

    pub fn bounds_for(self, shape: &Shape) -> Result<Rect> {
        shape.visual_bounds(Some(self))
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            join: LineJoin::Miter,
            start_cap: LineCap::Butt,
            end_cap: LineCap::Butt,
            miter_limit: 4.0,
            dash: None,
            align: StrokeAlign::Center,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StrokeAlign {
    #[default]
    Center,
    Inside,
    Outside,
}

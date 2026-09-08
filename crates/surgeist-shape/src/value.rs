use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericKind {
    Coordinate,
    Size,
    Radius,
    Stroke,
    Dash,
    Transform,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Finite(f64);

impl Finite {
    pub const ZERO: Self = Self(0.0);

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    pub fn try_new(value: f64, name: &str) -> Result<Self> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(Error::new(
                ErrorCode::NonFinite,
                format!("{name} must be finite"),
            ))
        }
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct NonNegative(Finite);

impl NonNegative {
    pub const ZERO: Self = Self(Finite::ZERO);

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    pub fn try_new(value: f64, kind: NumericKind) -> Result<Self> {
        let finite = Finite::try_new(value, numeric_name(kind))?;
        if value >= 0.0 {
            Ok(Self(finite))
        } else {
            Err(Error::new(
                negative_code(kind),
                format!("{} must be non-negative", numeric_name(kind)),
            ))
        }
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0.get()
    }
}

pub(crate) fn numeric_name(kind: NumericKind) -> &'static str {
    match kind {
        NumericKind::Coordinate => "coordinate",
        NumericKind::Size => "size",
        NumericKind::Radius => "radius",
        NumericKind::Stroke => "stroke",
        NumericKind::Dash => "dash",
        NumericKind::Transform => "transform",
    }
}

pub(crate) fn negative_code(kind: NumericKind) -> ErrorCode {
    match kind {
        NumericKind::Radius => ErrorCode::NegativeRadius,
        NumericKind::Stroke => ErrorCode::InvalidStroke,
        NumericKind::Dash => ErrorCode::InvalidDash,
        _ => ErrorCode::NegativeSize,
    }
}

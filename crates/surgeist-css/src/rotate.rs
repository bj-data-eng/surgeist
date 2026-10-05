//! Checked authored individual rotation, before contextual transform execution.

use crate::{CssAngleValue, CssSpecifiedNumber, CssValueOrigin};

/// The authored individual rotation; `none` is distinct from an identity rotation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssRotate {
    None,
    Value(CssRotateValues),
}

/// An optional authored rotation axis, retaining exact symbolic vector operands.
/// A zero vector is valid syntax; contextual interpretation belongs downstream.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssRotateAxis {
    X,
    Y,
    Z,
    Vector([CssSpecifiedNumber; 3]),
}

impl PartialEq for CssRotateAxis {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::X, Self::X) | (Self::Y, Self::Y) | (Self::Z, Self::Z) => true,
            (Self::Vector(left), Self::Vector(right)) => left
                .iter()
                .zip(right)
                .all(|(left, right)| left.structural_eq(right)),
            _ => false,
        }
    }
}

/// One checked angle and optional axis for the individual `rotate` property.
/// Unlike transform-function angles, this property's angle does not admit bare zero.
#[derive(Clone, Debug)]
pub struct CssRotateValues {
    angle: CssAngleValue,
    axis: Option<CssRotateAxis>,
    keyword_axis_origin: Option<CssValueOrigin>,
}

impl PartialEq for CssRotateValues {
    fn eq(&self, other: &Self) -> bool {
        self.angle.structural_eq(&other.angle) && self.axis == other.axis
    }
}

impl CssRotateValues {
    /// Retains checked operands without normalizing or resolving the axis.
    /// Keyword axes receive programmatic provenance; vector operands keep their origins.
    pub fn new(angle: CssAngleValue, axis: Option<CssRotateAxis>) -> Self {
        let keyword_axis_origin = match &axis {
            Some(CssRotateAxis::X | CssRotateAxis::Y | CssRotateAxis::Z) => {
                Some(CssValueOrigin::Programmatic)
            }
            Some(CssRotateAxis::Vector(_)) | None => None,
        };
        Self {
            angle,
            axis,
            keyword_axis_origin,
        }
    }

    pub(crate) fn from_keyword_axis(
        angle: CssAngleValue,
        axis: CssRotateAxis,
        origin: CssValueOrigin,
    ) -> Self {
        assert!(matches!(
            axis,
            CssRotateAxis::X | CssRotateAxis::Y | CssRotateAxis::Z
        ));
        Self {
            angle,
            axis: Some(axis),
            keyword_axis_origin: Some(origin),
        }
    }

    /// Borrows the original checked angle, including literal or math provenance.
    #[must_use]
    pub const fn angle(&self) -> &CssAngleValue {
        &self.angle
    }

    /// Borrows the authored axis; absence denotes the implicit positive z axis.
    #[must_use]
    pub const fn axis(&self) -> Option<&CssRotateAxis> {
        self.axis.as_ref()
    }

    /// Borrows keyword-axis provenance; omitted and vector axes return `None`.
    /// Vector origins remain available on each exact numeric operand.
    #[must_use]
    pub const fn keyword_axis_origin(&self) -> Option<&CssValueOrigin> {
        self.keyword_axis_origin.as_ref()
    }
}

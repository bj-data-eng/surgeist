//! Authored clipping, scrolling, and ellipsis controls from CSS Overflow 3.

use crate::specified_serialization::{SpecifiedSerializationContext, serialize_keyword_sequence};
use crate::{
    CssBoxEdgeKeyword, CssComponentValue, CssSpecifiedNonNegativeLength,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits, CssTextOverflow,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// The authored `<visual-box> || <length [0,∞]>` clip-margin value.
#[derive(Clone, Debug)]
pub struct CssOverflowClipMargin {
    authored_box_edge: Option<CssBoxEdgeKeyword>,
    authored_offset: Option<CssSpecifiedNonNegativeLength>,
}

impl PartialEq for CssOverflowClipMargin {
    fn eq(&self, other: &Self) -> bool {
        self.authored_box_edge == other.authored_box_edge
            && match (&self.authored_offset, &other.authored_offset) {
                (None, None) => true,
                (Some(left), Some(right)) => left.structural_eq(right),
                _ => false,
            }
    }
}

impl Eq for CssOverflowClipMargin {}

impl CssOverflowClipMargin {
    /// Checks that at least one component is authored and the edge is visual.
    #[must_use]
    pub fn try_new(
        authored_box_edge: Option<CssBoxEdgeKeyword>,
        authored_offset: Option<CssSpecifiedNonNegativeLength>,
    ) -> Option<Self> {
        if authored_box_edge.is_none() && authored_offset.is_none() {
            return None;
        }
        if authored_box_edge.is_some_and(|edge| {
            !matches!(
                edge,
                CssBoxEdgeKeyword::ContentBox
                    | CssBoxEdgeKeyword::PaddingBox
                    | CssBoxEdgeKeyword::BorderBox
            )
        }) {
            return None;
        }
        Some(Self {
            authored_box_edge,
            authored_offset,
        })
    }

    /// Constructs the specified initial `0px`, with no authored box edge.
    #[must_use]
    pub fn initial() -> Self {
        Self::try_new(None, Some(Self::zero_px())).expect("valid clip-margin initial")
    }

    fn zero_px() -> CssSpecifiedNonNegativeLength {
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("0", "px").expect("valid dimension"),
        )
        .expect("valid nonnegative length")
    }

    /// Borrows the authored visual box, if present.
    pub const fn authored_box_edge(&self) -> Option<CssBoxEdgeKeyword> {
        self.authored_box_edge
    }

    /// Borrows the authored offset, if present.
    pub fn authored_offset(&self) -> Option<&CssSpecifiedNonNegativeLength> {
        self.authored_offset.as_ref()
    }

    /// Returns the effective box edge, defaulting to `padding-box`.
    pub const fn box_edge(&self) -> CssBoxEdgeKeyword {
        match self.authored_box_edge {
            Some(edge) => edge,
            None => CssBoxEdgeKeyword::PaddingBox,
        }
    }

    /// Returns the effective offset, defaulting to exact `0px`.
    pub fn offset(&self) -> CssSpecifiedNonNegativeLength {
        self.authored_offset.clone().unwrap_or_else(Self::zero_px)
    }

    /// Serializes only the authored components in canonical box-first order.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the complete authored value under one resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        if let Some(edge) = self.authored_box_edge {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            context.append(&mut output, edge.as_css_str())?;
        }
        if let Some(offset) = &self.authored_offset {
            if self.authored_box_edge.is_some() {
                context.append(&mut output, " ")?;
            }
            let captured = offset.capture_specified(&mut context)?;
            context.append(&mut output, &captured)?;
        }
        Ok(output)
    }
}

/// The authored scroll-behavior keyword, before scroll execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollBehavior {
    Auto,
    Smooth,
}

impl CssScrollBehavior {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize_keyword_sequence(self.as_css(), limits)
    }

    const fn as_css(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Smooth => "smooth",
        }
    }
}

/// The valid authored scrollbar-gutter keyword combinations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollbarGutter {
    Auto,
    Stable,
    StableBothEdges,
}

impl CssScrollbarGutter {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize_keyword_sequence(self.as_css(), limits)
    }

    const fn as_css(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Stable => "stable",
            Self::StableBothEdges => "stable both-edges",
        }
    }
}

impl CssTextOverflow {
    /// Serializes the canonical Overflow 3 ellipsis keyword.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one keyword under exact resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize_keyword_sequence(
            match self {
                Self::Clip => "clip",
                Self::Ellipsis => "ellipsis",
            },
            limits,
        )
    }
}

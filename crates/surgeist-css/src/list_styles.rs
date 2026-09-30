//! Checked, symbolic values for the Lists 3 marker-style properties.

use crate::{
    CssContentString, CssCounterStyleValue, CssIdent, CssImageValue, CssListStylePosition,
};

/// Symbolic marker-side selection for a list item.
///
/// Selecting the element's or parent's directionality and placing a marker occur downstream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMarkerSide {
    MatchSelf,
    MatchParent,
}

/// A checked authored `list-style-type` value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssListStyleTypeValue {
    None,
    String(CssContentString),
    CounterStyle(CssCounterStyleValue),
}

impl CssListStyleTypeValue {
    /// The inherited `disc` initial, still symbolic until marker generation.
    #[must_use]
    pub fn initial() -> Self {
        Self::CounterStyle(
            CssCounterStyleValue::try_named(CssIdent::try_new("disc").expect("disc identifier"))
                .expect("disc counter style"),
        )
    }
}

/// Checked authored shorthand components. Missing members reset to their initials.
#[derive(Clone, Debug, PartialEq)]
pub struct CssListStyleValue {
    style_type: Option<CssListStyleTypeValue>,
    position: Option<CssListStylePosition>,
    image: Option<CssImageValue>,
}

impl CssListStyleValue {
    /// Rejects an empty shorthand while retaining each authored component's presence.
    #[must_use]
    pub fn try_new(
        style_type: Option<CssListStyleTypeValue>,
        position: Option<CssListStylePosition>,
        image: Option<CssImageValue>,
    ) -> Option<Self> {
        (style_type.is_some() || position.is_some() || image.is_some()).then_some(Self {
            style_type,
            position,
            image,
        })
    }

    #[must_use]
    pub const fn style_type(&self) -> Option<&CssListStyleTypeValue> {
        self.style_type.as_ref()
    }
    #[must_use]
    pub const fn position(&self) -> Option<CssListStylePosition> {
        self.position
    }
    #[must_use]
    pub const fn image(&self) -> Option<&CssImageValue> {
        self.image.as_ref()
    }
}

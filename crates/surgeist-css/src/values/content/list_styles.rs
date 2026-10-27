//! Checked, symbolic values for the Lists 3 marker-style properties.

use crate::values::common::CssIdent;
use crate::values::content::{CssContentString, CssCounterStyleValue, CssListStylePosition};
use crate::values::image::CssImageValue;

/// Symbolic marker-side selection for a list item.
///
/// Selecting the element's or parent's directionality and placing a marker occur downstream.
///
/// # Specified serialization
///
/// Serializes the specified keyword without resolving directionality.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMarkerSide {
    /// `match-self`: the marker box is positioned using the directionality of the `::marker`'s originating element, the list item itself.
    MatchSelf,
    /// `match-parent`: the marker box is positioned using the directionality of the parent of the `::marker`'s originating element.
    MatchParent,
}

/// A checked authored `list-style-type` value.
///
/// # Specified serialization
///
/// Serializes an authored counter style, string, or `none` without resolution.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssListStyleTypeValue {
    /// `none`: the element has no marker string.
    None,
    /// A `<string>` that is the element's marker string.
    String(CssContentString),
    /// A named or `symbols()` counter style.
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
///
/// # Specified serialization
///
/// Serializes effective marker values in position, image, type order. Initial values are omitted
/// only where parsing still preserves the same meaning. An `outside` position is inserted when a
/// named `inside` or `outside` counter style would otherwise be parsed as the position.
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

    /// Returns the authored `<'list-style-type'>` component, or `None` when it was omitted.
    #[must_use]
    pub const fn style_type(&self) -> Option<&CssListStyleTypeValue> {
        self.style_type.as_ref()
    }
    /// Returns the authored `<'list-style-position'>` component, or `None` when it was omitted.
    #[must_use]
    pub const fn position(&self) -> Option<CssListStylePosition> {
        self.position
    }
    /// Returns the authored `<'list-style-image'>` component, or `None` when it was omitted.
    #[must_use]
    pub const fn image(&self) -> Option<&CssImageValue> {
        self.image.as_ref()
    }
}

//! Checked, symbolic values for the Lists 3 marker-style properties.

use crate::{
    CssContentString, CssCounterStyleValue, CssIdent, CssImageValue, CssListStyle,
    CssListStyleImage, CssListStylePosition, CssListStyleType,
};

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

#[must_use]
pub(crate) fn type_i01(value: &CssListStyleTypeValue) -> Option<CssListStyleType> {
    Some(match value {
        CssListStyleTypeValue::None => CssListStyleType::None,
        CssListStyleTypeValue::String(value) => CssListStyleType::String(value.clone()),
        CssListStyleTypeValue::CounterStyle(value) => {
            CssListStyleType::CounterStyle(crate::content_values::legacy_style(value)?)
        }
    })
}

#[must_use]
pub(crate) fn image_i01(value: &CssImageValue) -> Option<CssListStyleImage> {
    match value {
        CssImageValue::None => Some(CssListStyleImage::None),
        CssImageValue::Url(value) => Some(CssListStyleImage::Url(value.clone())),
        CssImageValue::Gradient(_) => None,
    }
}

#[must_use]
pub(crate) fn shorthand_i01(value: &CssListStyleValue) -> Option<CssListStyle> {
    let style_type = match value.style_type.as_ref() {
        Some(value) => Some(type_i01(value)?),
        None => None,
    };
    let image = match value.image.as_ref() {
        Some(value) => Some(image_i01(value)?),
        None => None,
    };
    CssListStyle::try_new(style_type, value.position, image)
}

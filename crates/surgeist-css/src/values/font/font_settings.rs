//! Authored OpenType feature and variation settings, before font selection.

use crate::numeric::{CssInteger, CssNonNegative, CssNumber, CssNumeric};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;

type SerializationResult<T> = Result<T, CssSerializationError>;

/// A case-sensitive, four-character printable ASCII OpenType tag.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssOpenTypeTag {
    value: String,
}

impl CssOpenTypeTag {
    /// Checks an OpenType tag.
    ///
    /// Returns `None` unless `value` is exactly four bytes, each printable ASCII (U+0020 through U+007E).
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (value.len() == 4 && value.bytes().all(|byte| (0x20..=0x7e).contains(&byte)))
            .then_some(Self { value })
    }

    /// Returns the four-character tag text, with its case preserved.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// The authored value following an OpenType feature tag.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredFontFeatureValue {
    /// No value was written after the tag; the feature takes the default value `1`.
    Omitted,
    /// `on`: enables the feature, equivalent to `1`.
    On,
    /// `off`: disables the feature, equivalent to `0`.
    Off,
    /// A non-negative `<integer>` feature value; it may be a deferred calculation.
    Index(CssNumeric<CssInteger, CssNonNegative>),
}

/// One checked authored OpenType feature setting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAuthoredFontFeature {
    tag: CssOpenTypeTag,
    value: CssAuthoredFontFeatureValue,
}

impl CssAuthoredFontFeature {
    /// Creates a feature setting from a checked tag and its authored value.
    #[must_use]
    pub const fn new(tag: CssOpenTypeTag, value: CssAuthoredFontFeatureValue) -> Self {
        Self { tag, value }
    }
    /// Returns the feature tag.
    #[must_use]
    pub const fn tag(&self) -> &CssOpenTypeTag {
        &self.tag
    }
    /// Returns the authored value written after the tag.
    #[must_use]
    pub const fn value(&self) -> &CssAuthoredFontFeatureValue {
        &self.value
    }
}

/// A nonempty ordered list of feature settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAuthoredFontFeatureList {
    features: Vec<CssAuthoredFontFeature>,
}

impl CssAuthoredFontFeatureList {
    /// Checks a feature list.
    ///
    /// Returns `None` when `features` is empty.
    #[must_use]
    pub fn try_new(features: Vec<CssAuthoredFontFeature>) -> Option<Self> {
        (!features.is_empty()).then_some(Self { features })
    }
    /// Returns the feature settings in authored order; the slice is never empty.
    #[must_use]
    pub fn features(&self) -> &[CssAuthoredFontFeature] {
        &self.features
    }
}

/// Current authored `font-feature-settings` property and descriptor value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredFontFeatureSettings {
    /// `normal`: no change in glyph selection or positioning occurs due to this property.
    Normal,
    /// A nonempty comma-separated list of feature settings.
    Features(CssAuthoredFontFeatureList),
}

/// One authored OpenType variation axis and signed number.
#[derive(Clone, Debug)]
pub struct CssFontVariation {
    tag: CssOpenTypeTag,
    value: CssNumeric<CssNumber>,
}

impl CssFontVariation {
    /// Creates a variation setting from a checked axis tag and a number.
    #[must_use]
    pub const fn new(tag: CssOpenTypeTag, value: CssNumeric<CssNumber>) -> Self {
        Self { tag, value }
    }
    /// Returns the variation axis tag.
    #[must_use]
    pub const fn tag(&self) -> &CssOpenTypeTag {
        &self.tag
    }
    /// Returns the authored axis value; it may be a deferred calculation.
    #[must_use]
    pub const fn value(&self) -> &CssNumeric<CssNumber> {
        &self.value
    }
}

impl PartialEq for CssFontVariation {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.value == other.value
    }
}
impl Eq for CssFontVariation {}

/// Nonempty ordered variation settings, retaining duplicate axis names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontVariationList {
    variations: Vec<CssFontVariation>,
}

impl CssFontVariationList {
    /// Checks a variation list.
    ///
    /// Returns `None` when `variations` is empty.
    #[must_use]
    pub fn try_new(variations: Vec<CssFontVariation>) -> Option<Self> {
        (!variations.is_empty()).then_some(Self { variations })
    }
    /// Returns the variation settings in authored order, including repeated axis tags; the slice is never empty.
    #[must_use]
    pub fn variations(&self) -> &[CssFontVariation] {
        &self.variations
    }
}

/// Current authored `font-variation-settings` property and descriptor value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariationSettings {
    /// `normal`: no change in glyph shape, matching, or positioning occurs due to this property.
    Normal,
    /// A nonempty comma-separated list of variation settings.
    Variations(CssFontVariationList),
}

impl CssAuthoredFontFeatureSettings {
    pub(crate) fn append_specified(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal => writer.keyword("normal")?,
            Self::Features(list) => {
                for (index, feature) in list.features().iter().enumerate() {
                    if index != 0 {
                        writer.append(", ")?;
                    }
                    writer.node()?;
                    writer.append_string(feature.tag().as_str())?;
                    match feature.value() {
                        CssAuthoredFontFeatureValue::Omitted => {}
                        CssAuthoredFontFeatureValue::On => {
                            writer.append(" ")?;
                            writer.keyword("on")?;
                        }
                        CssAuthoredFontFeatureValue::Off => {
                            writer.append(" ")?;
                            writer.keyword("off")?;
                        }
                        CssAuthoredFontFeatureValue::Index(value) => {
                            writer.append(" ")?;
                            value.append_specified(&mut writer.context, &mut writer.css)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssAuthoredFontFeatureSettings
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    CssAuthoredFontFeatureSettings
);

impl CssFontVariationSettings {
    pub(crate) fn append_specified(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal => writer.keyword("normal")?,
            Self::Variations(list) => {
                for (index, variation) in list.variations().iter().enumerate() {
                    if index != 0 {
                        writer.append(", ")?;
                    }
                    writer.node()?;
                    writer.append_string(variation.tag().as_str())?;
                    writer.append(" ")?;
                    let captured = variation.value().capture_specified(&mut writer.context)?;
                    writer.append(&captured)?;
                }
            }
        }
        Ok(())
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssFontVariationSettings
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    CssFontVariationSettings
);

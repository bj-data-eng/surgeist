//! Authored OpenType feature and variation settings, before font selection.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssComponentValue, CssComponentValueRef, CssIntegerCalculation, CssIntegerLiteral,
    CssNumericConstructionError, CssNumericConstructionErrorKind, CssNumericTokenKind,
    CssSpecifiedNumber, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    CssValueOrigin, CssValueTokenRef,
};

type ConstructionResult<T> = Result<T, CssNumericConstructionError>;
type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// A case-sensitive, four-character printable ASCII OpenType tag.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssOpenTypeTag {
    value: String,
}

impl CssOpenTypeTag {
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (value.len() == 4 && value.bytes().all(|byte| (0x20..=0x7e).contains(&byte)))
            .then_some(Self { value })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Exact nonnegative authored OpenType feature index or deferred integer math.
#[derive(Clone, Debug)]
pub struct CssFontFeatureIndex {
    value: FeatureIndexValue,
}

#[derive(Clone, Debug)]
enum FeatureIndexValue {
    Literal(CssIntegerLiteral),
    Calculation(CssIntegerCalculation),
}

impl PartialEq for CssFontFeatureIndex {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (FeatureIndexValue::Literal(left), FeatureIndexValue::Literal(right)) => left
                .component()
                .structural_eq_ignoring_origin(right.component()),
            (FeatureIndexValue::Calculation(left), FeatureIndexValue::Calculation(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}
impl Eq for CssFontFeatureIndex {}

impl CssFontFeatureIndex {
    #[must_use]
    pub fn try_new(value: i32) -> Option<Self> {
        (value >= 0).then(|| {
            Self::try_from_component(
                CssComponentValue::try_number(&value.to_string()).expect("valid i32 number"),
            )
            .expect("nonnegative i32 integer")
        })
    }

    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        };
        if number.kind() != CssNumericTokenKind::Integer {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        }
        if crate::exact_decimal::LexicalDecimal::new(number.representation()).negative {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        let literal = CssIntegerLiteral::try_from_component(component).expect("checked integer");
        Ok(Self {
            value: FeatureIndexValue::Literal(literal),
        })
    }

    pub fn try_from_calculation(calculation: CssIntegerCalculation) -> ConstructionResult<Self> {
        let root = crate::specified_numeric::significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: FeatureIndexValue::Calculation(calculation),
        })
    }

    #[must_use]
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            FeatureIndexValue::Literal(value) => Some(value.component()),
            FeatureIndexValue::Calculation(_) => None,
        }
    }

    #[must_use]
    pub fn calculation(&self) -> Option<&CssIntegerCalculation> {
        match &self.value {
            FeatureIndexValue::Literal(_) => None,
            FeatureIndexValue::Calculation(value) => Some(value),
        }
    }

    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            FeatureIndexValue::Literal(value) => value.origin(),
            FeatureIndexValue::Calculation(value) => value.origin(),
        }
    }

    #[must_use]
    pub fn i32_value(&self) -> Option<i32> {
        self.literal_component().and_then(|component| {
            let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view()
            else {
                return None;
            };
            crate::integer_value::exact_i32(number.representation())
        })
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match &self.value {
            FeatureIndexValue::Literal(value) => value.append_specified(context, output),
            FeatureIndexValue::Calculation(value) => {
                value.serialize_specified_into(context, output)
            }
        }
    }
}

/// The authored value following an OpenType feature tag.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredFontFeatureValue {
    Omitted,
    On,
    Off,
    Index(CssFontFeatureIndex),
}

/// One checked authored OpenType feature setting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAuthoredFontFeature {
    tag: CssOpenTypeTag,
    value: CssAuthoredFontFeatureValue,
}

impl CssAuthoredFontFeature {
    #[must_use]
    pub const fn new(tag: CssOpenTypeTag, value: CssAuthoredFontFeatureValue) -> Self {
        Self { tag, value }
    }
    #[must_use]
    pub const fn tag(&self) -> &CssOpenTypeTag {
        &self.tag
    }
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
    #[must_use]
    pub fn try_new(features: Vec<CssAuthoredFontFeature>) -> Option<Self> {
        (!features.is_empty()).then_some(Self { features })
    }
    #[must_use]
    pub fn features(&self) -> &[CssAuthoredFontFeature] {
        &self.features
    }
}

/// Current authored `font-feature-settings` property and descriptor value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredFontFeatureSettings {
    Normal,
    Features(CssAuthoredFontFeatureList),
}

/// One authored OpenType variation axis and signed number.
#[derive(Clone, Debug)]
pub struct CssFontVariation {
    tag: CssOpenTypeTag,
    value: CssSpecifiedNumber,
}

impl CssFontVariation {
    #[must_use]
    pub const fn new(tag: CssOpenTypeTag, value: CssSpecifiedNumber) -> Self {
        Self { tag, value }
    }
    #[must_use]
    pub const fn tag(&self) -> &CssOpenTypeTag {
        &self.tag
    }
    #[must_use]
    pub const fn value(&self) -> &CssSpecifiedNumber {
        &self.value
    }
}

impl PartialEq for CssFontVariation {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.value.structural_eq(&other.value)
    }
}
impl Eq for CssFontVariation {}

/// Nonempty ordered variation settings, retaining duplicate axis names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontVariationList {
    variations: Vec<CssFontVariation>,
}

impl CssFontVariationList {
    #[must_use]
    pub fn try_new(variations: Vec<CssFontVariation>) -> Option<Self> {
        (!variations.is_empty()).then_some(Self { variations })
    }
    #[must_use]
    pub fn variations(&self) -> &[CssFontVariation] {
        &self.variations
    }
}

/// Current authored `font-variation-settings` property and descriptor value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariationSettings {
    Normal,
    Variations(CssFontVariationList),
}

fn keyword(writer: &mut SpecifiedRuleWriter, value: &str) -> SerializationResult<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    writer.append(value)
}

impl CssAuthoredFontFeatureSettings {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        match self {
            Self::Normal => keyword(&mut writer, "normal")?,
            Self::Features(list) => {
                for (index, feature) in list.features().iter().enumerate() {
                    if index != 0 {
                        writer.append(", ")?;
                    }
                    writer.context.charge_input(1)?;
                    writer.context.charge_projection(1)?;
                    writer.append_string(feature.tag().as_str())?;
                    match feature.value() {
                        CssAuthoredFontFeatureValue::Omitted => {}
                        CssAuthoredFontFeatureValue::On => {
                            writer.append(" ")?;
                            keyword(&mut writer, "on")?;
                        }
                        CssAuthoredFontFeatureValue::Off => {
                            writer.append(" ")?;
                            keyword(&mut writer, "off")?;
                        }
                        CssAuthoredFontFeatureValue::Index(value) => {
                            writer.append(" ")?;
                            value.append_specified(&mut writer.context, &mut writer.css)?;
                        }
                    }
                }
            }
        }
        Ok(writer.css)
    }
}

impl CssFontVariationSettings {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        match self {
            Self::Normal => keyword(&mut writer, "normal")?,
            Self::Variations(list) => {
                for (index, variation) in list.variations().iter().enumerate() {
                    if index != 0 {
                        writer.append(", ")?;
                    }
                    writer.context.charge_input(1)?;
                    writer.context.charge_projection(1)?;
                    writer.append_string(variation.tag().as_str())?;
                    writer.append(" ")?;
                    let captured = variation.value().capture_specified(&mut writer.context)?;
                    writer.append(&captured)?;
                }
            }
        }
        Ok(writer.css)
    }
}

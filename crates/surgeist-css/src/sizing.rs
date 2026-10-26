//! Checked authored box sizes before layout and writing-mode resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssBoxCalcSize, CssLengthPercentage, CssNonNegative, CssNumeric, CssNumericConstructionError,
    CssSerializationError,
};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// One shared `<size-keyword> | <length-percentage> | fit-content() | calc-size()` value.
///
/// # Specified serialization
///
/// Serializes the retained specified value without resolving percentages or intrinsic sizes. Finite
/// calculation children use the shared six-place specified number text policy.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBoxSize {
    LengthPercentage(CssNumeric<CssLengthPercentage, CssNonNegative>),
    Stretch,
    Contain,
    MinContent,
    MaxContent,
    FitContent,
    FitContentFunction(CssNumeric<CssLengthPercentage, CssNonNegative>),
    CalcSize(CssBoxCalcSize),
}

// A specified sizing value is comparable across separately parsed or checked
// declarations: numeric payloads compare by value, without source origins.
impl PartialEq for CssBoxSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LengthPercentage(left), Self::LengthPercentage(right))
            | (Self::FitContentFunction(left), Self::FitContentFunction(right)) => left == right,
            (Self::Stretch, Self::Stretch)
            | (Self::Contain, Self::Contain)
            | (Self::MinContent, Self::MinContent)
            | (Self::MaxContent, Self::MaxContent)
            | (Self::FitContent, Self::FitContent) => true,
            (Self::CalcSize(left), Self::CalcSize(right)) => {
                left.as_calc_size() == right.as_calc_size()
            }
            _ => false,
        }
    }
}

impl CssBoxSize {
    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::LengthPercentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)
            }
            Self::Stretch => context.keyword(output, "stretch"),
            Self::Contain => context.keyword(output, "contain"),
            Self::MinContent => context.keyword(output, "min-content"),
            Self::MaxContent => context.keyword(output, "max-content"),
            Self::FitContent => context.keyword(output, "fit-content"),
            Self::FitContentFunction(value) => {
                context.charge_nodes(1)?;
                context.append(output, "fit-content(")?;
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)?;
                context.append(output, ")")
            }
            Self::CalcSize(value) => value.serialize_specified_into(context, output),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssBoxSize {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBoxSize);

/// A preferred or minimum size. `auto` is distinct from every shared box size.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssSizeValue {
    Auto,
    BoxSize(CssBoxSize),
}

impl CssSizeValue {
    pub fn box_size(&self) -> Option<&CssBoxSize> {
        match self {
            Self::Auto => None,
            Self::BoxSize(value) => Some(value),
        }
    }

    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Auto => context.keyword(output, "auto"),
            Self::BoxSize(value) => value.serialize_specified_into(context, output),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssSizeValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssSizeValue);

/// A maximum size, constructed only after rejecting `auto` at every nested basis.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMaxSizeValue(CssMaxSizeKind);

#[derive(Clone, Debug, PartialEq)]
enum CssMaxSizeKind {
    None,
    BoxSize(CssBoxSize),
}

impl CssMaxSizeValue {
    pub const NONE: Self = Self(CssMaxSizeKind::None);

    pub fn try_box_size(value: CssBoxSize) -> Result<Self, CssNumericConstructionError> {
        if let CssBoxSize::CalcSize(calc) = &value {
            calc.validate_maximum_context()?;
        }
        Ok(Self(CssMaxSizeKind::BoxSize(value)))
    }

    pub fn box_size(&self) -> Option<&CssBoxSize> {
        match &self.0 {
            CssMaxSizeKind::None => None,
            CssMaxSizeKind::BoxSize(value) => Some(value),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self.0, CssMaxSizeKind::None)
    }

    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match &self.0 {
            CssMaxSizeKind::None => context.keyword(output, "none"),
            CssMaxSizeKind::BoxSize(value) => value.serialize_specified_into(context, output),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssMaxSizeValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssMaxSizeValue);

//! Checked authored box sizes before layout and writing-mode resolution.

use crate::numeric::{
    CssLengthPercentage, CssNonNegative, CssNumeric, CssNumericConstructionError,
};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::values::box_model::CssBoxCalcSize;

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
    /// A non-negative `<length-percentage>` size.
    LengthPercentage(CssNumeric<CssLengthPercentage, CssNonNegative>),
    /// `stretch`: stretch-fit sizing, which matches the box's margin box to the size of its containing block.
    Stretch,
    /// `contain`: contain-fit sizing when the box has a preferred aspect ratio, otherwise stretch-fit sizing.
    Contain,
    /// `min-content`: the box's min-content size.
    MinContent,
    /// `max-content`: the box's max-content size.
    MaxContent,
    /// `fit-content`: the box's fit-content size, `min(max-content, max(min-content, stretch))`.
    FitContent,
    /// The `fit-content(<length-percentage>)` function with its non-negative limit argument.
    FitContentFunction(CssNumeric<CssLengthPercentage, CssNonNegative>),
    /// A `calc-size()` function checked for box-sizing use.
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
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssBoxSize {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssBoxSize);

/// A preferred or minimum size. `auto` is distinct from every shared box size.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssSizeValue {
    /// `auto`: the box's automatic size, which the relevant layout module determines (for minimum sizes, an automatic minimum size).
    Auto,
    /// A shared box size.
    BoxSize(CssBoxSize),
}

impl CssSizeValue {
    /// Returns the shared box size, or `None` for `auto`.
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
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssSizeValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssSizeValue);

/// A maximum size, constructed only after rejecting `auto` at every nested basis.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMaxSizeValue(CssMaxSizeKind);

#[derive(Clone, Debug, PartialEq)]
enum CssMaxSizeKind {
    None,
    BoxSize(CssBoxSize),
}

impl CssMaxSizeValue {
    /// The `none` maximum size, which imposes no limit.
    pub const NONE: Self = Self(CssMaxSizeKind::None);

    /// Wraps a shared box size as a maximum size.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is a `calc-size()` whose basis chain contains `auto`, which a maximum size rejects.
    pub fn try_box_size(value: CssBoxSize) -> Result<Self, CssNumericConstructionError> {
        if let CssBoxSize::CalcSize(calc) = &value {
            calc.validate_maximum_context()?;
        }
        Ok(Self(CssMaxSizeKind::BoxSize(value)))
    }

    /// Returns the shared box size, or `None` for the `none` maximum size.
    pub fn box_size(&self) -> Option<&CssBoxSize> {
        match &self.0 {
            CssMaxSizeKind::None => None,
            CssMaxSizeKind::BoxSize(value) => Some(value),
        }
    }

    /// Returns whether this is the `none` maximum size.
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
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssMaxSizeValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssMaxSizeValue);

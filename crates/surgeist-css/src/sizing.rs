//! Checked authored box sizes before layout and writing-mode resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssCalcSize, CssNumericConstructionError, CssSpecifiedNonNegativeLengthPercentage,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// One shared `<size-keyword> | <length-percentage> | fit-content() | calc-size()` value.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBoxSize {
    LengthPercentage(CssSpecifiedNonNegativeLengthPercentage),
    Stretch,
    Contain,
    MinContent,
    MaxContent,
    FitContent,
    FitContentFunction(CssSpecifiedNonNegativeLengthPercentage),
    CalcSize(CssCalcSize),
}

// A specified sizing value is comparable across separately parsed or checked
// declarations. The exact authored token/math structure matters; diagnostic
// source snapshots and offsets do not.
impl PartialEq for CssBoxSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LengthPercentage(left), Self::LengthPercentage(right))
            | (Self::FitContentFunction(left), Self::FitContentFunction(right)) => {
                match (left.literal_component(), right.literal_component()) {
                    (Some(left), Some(right)) => left.structural_eq_ignoring_origin(right),
                    (None, None) => left
                        .calculation()
                        .expect("checked math")
                        .structural_eq(right.calculation().expect("checked math")),
                    _ => false,
                }
            }
            (Self::Stretch, Self::Stretch)
            | (Self::Contain, Self::Contain)
            | (Self::MinContent, Self::MinContent)
            | (Self::MaxContent, Self::MaxContent)
            | (Self::FitContent, Self::FitContent) => true,
            (Self::CalcSize(left), Self::CalcSize(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

impl CssBoxSize {
    /// Serializes the retained specified value without resolving percentages or intrinsic sizes.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under one shared resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.serialize_specified_into(&mut context, &mut output)?;
        Ok(output)
    }

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
            Self::Stretch => append_keyword(context, output, "stretch"),
            Self::Contain => append_keyword(context, output, "contain"),
            Self::MinContent => append_keyword(context, output, "min-content"),
            Self::MaxContent => append_keyword(context, output, "max-content"),
            Self::FitContent => append_keyword(context, output, "fit-content"),
            Self::FitContentFunction(value) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "fit-content(")?;
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)?;
                context.append(output, ")")
            }
            Self::CalcSize(value) => value.serialize_specified_into(context, output),
        }
    }
}

fn append_keyword(
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    keyword: &'static str,
) -> SerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, keyword)
}

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

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        match self {
            Self::Auto => {
                let mut context = SpecifiedSerializationContext::new(limits);
                let mut output = String::new();
                append_keyword(&mut context, &mut output, "auto")?;
                Ok(output)
            }
            Self::BoxSize(value) => value.serialize_specified_with_limits(limits),
        }
    }
}

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

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        match &self.0 {
            CssMaxSizeKind::None => {
                let mut context = SpecifiedSerializationContext::new(limits);
                let mut output = String::new();
                append_keyword(&mut context, &mut output, "none")?;
                Ok(output)
            }
            CssMaxSizeKind::BoxSize(value) => value.serialize_specified_with_limits(limits),
        }
    }
}

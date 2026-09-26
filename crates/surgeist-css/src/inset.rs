//! Exact authored positioned offsets before writing-mode and layout resolution.

use crate::specified_serialization::{SpecifiedSerializationContext, serialize_keyword_sequence};
use crate::{
    CssBoxSideKind, CssSpecifiedLengthPercentage, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssValueOrigin,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// One physical or logical inset longhand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssInsetValue {
    /// The unresolved automatic offset.
    Auto,
    /// An exact signed length-percentage, including deferred math.
    LengthPercentage(CssSpecifiedLengthPercentage),
}

impl CssInsetValue {
    /// Borrows the checked numeric value, if present.
    pub fn length_percentage(&self) -> Option<&CssSpecifiedLengthPercentage> {
        match self {
            Self::Auto => None,
            Self::LengthPercentage(value) => Some(value),
        }
    }

    /// Borrows a retained numeric origin; `auto` has no numeric origin.
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.length_percentage()
            .map(CssSpecifiedLengthPercentage::origin)
    }

    /// Serializes the canonical specified value.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under an explicit resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.serialize_into(&mut context, &mut output)?;
        Ok(output)
    }

    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Auto => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "auto")
            }
            Self::LengthPercentage(value) => {
                let text = value.capture_specified(context)?;
                context.append(output, &text)
            }
        }
    }
}

impl PartialEq for CssInsetValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}
impl Eq for CssInsetValue {}

/// A one- or two-value logical inset-axis shorthand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssInsetPair {
    start: CssInsetValue,
    authored_end: Option<CssInsetValue>,
}

impl CssInsetPair {
    /// Retains the authored start and optional end.
    #[must_use]
    pub fn new(start: CssInsetValue, authored_end: Option<CssInsetValue>) -> Self {
        Self {
            start,
            authored_end,
        }
    }

    /// Borrows the start assignment.
    pub fn start(&self) -> &CssInsetValue {
        &self.start
    }
    /// Borrows the explicitly authored end, if any.
    pub fn authored_end(&self) -> Option<&CssInsetValue> {
        self.authored_end.as_ref()
    }
    /// Borrows the effective end, repeating start when omitted.
    pub fn end(&self) -> &CssInsetValue {
        self.authored_end.as_ref().unwrap_or(&self.start)
    }

    /// Serializes only authored components.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes both authored values under one resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.start.serialize_into(&mut context, &mut output)?;
        if let Some(end) = &self.authored_end {
            context.append(&mut output, " ")?;
            end.serialize_into(&mut context, &mut output)?;
        }
        Ok(output)
    }
}

/// A one- to four-value physical or logical inset shorthand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssInsetShorthand {
    kind: CssBoxSideKind,
    authored_values: Box<[CssInsetValue]>,
}

impl CssInsetShorthand {
    /// Checks the shorthand cardinality and preserves its side kind.
    pub fn try_new(kind: CssBoxSideKind, authored_values: Vec<CssInsetValue>) -> Option<Self> {
        if !(1..=4).contains(&authored_values.len()) {
            return None;
        }
        Some(Self {
            kind,
            authored_values: authored_values.into_boxed_slice(),
        })
    }

    /// Returns the physical or logical side kind.
    pub fn kind(&self) -> CssBoxSideKind {
        self.kind
    }
    /// Borrows the exact authored values in source order.
    pub fn authored_values(&self) -> &[CssInsetValue] {
        &self.authored_values
    }
    /// Returns top/right/bottom/left or block-start/inline-start/block-end/inline-end assignments.
    /// This does not determine the shorthand's full reset membership.
    pub fn assigned_values(&self) -> [&CssInsetValue; 4] {
        let values = &self.authored_values;
        match values.len() {
            1 => [&values[0], &values[0], &values[0], &values[0]],
            2 => [&values[0], &values[1], &values[0], &values[1]],
            3 => [&values[0], &values[1], &values[2], &values[1]],
            4 => [&values[0], &values[1], &values[2], &values[3]],
            _ => unreachable!("checked shorthand cardinality"),
        }
    }

    /// Serializes only the authored switch and components.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes all authored components under one resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        if self.kind == CssBoxSideKind::Logical {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            context.append(&mut output, "logical ")?;
        }
        for (index, value) in self.authored_values.iter().enumerate() {
            if index > 0 {
                context.append(&mut output, " ")?;
            }
            value.serialize_into(&mut context, &mut output)?;
        }
        Ok(output)
    }
}

impl crate::CssLayoutPosition {
    /// Serializes the canonical authored position keyword.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical keyword under explicit limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let text = match self {
            Self::Static => "static",
            Self::Relative => "relative",
            Self::Absolute => "absolute",
            Self::Fixed => "fixed",
            Self::Sticky => "sticky",
        };
        serialize_keyword_sequence(text, limits)
    }
}

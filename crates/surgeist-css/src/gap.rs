//! Exact authored gap values before layout and percentage resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssSpecifiedNonNegativeLengthPercentage, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// One `row-gap` or `column-gap` value, retaining symbolic normal or exact numeric syntax.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CssGapValue {
    Normal,
    LengthPercentage(CssSpecifiedNonNegativeLengthPercentage),
}

impl CssGapValue {
    /// Serializes this specified value without resolving layout context.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under explicit input, projection, and byte limits.
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
            Self::Normal => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "normal")
            }
            Self::LengthPercentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    }
}

impl PartialEq for CssGapValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Normal, Self::Normal) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}

impl Eq for CssGapValue {}

/// An authored `gap` shorthand with one row value and an optional column value.
///
/// A missing column repeats the row without manufacturing a second numeric origin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGapShorthand {
    row: CssGapValue,
    authored_column: Option<CssGapValue>,
}

impl CssGapShorthand {
    /// Retains already checked values and whether the column was authored.
    #[must_use]
    pub fn new(row: CssGapValue, authored_column: Option<CssGapValue>) -> Self {
        Self {
            row,
            authored_column,
        }
    }

    /// Borrows the row value.
    #[must_use]
    pub const fn row(&self) -> &CssGapValue {
        &self.row
    }

    /// Borrows an explicitly authored column value.
    #[must_use]
    pub const fn authored_column(&self) -> Option<&CssGapValue> {
        self.authored_column.as_ref()
    }

    /// Borrows the effective column value, repeating the row when omitted.
    #[must_use]
    pub fn column(&self) -> &CssGapValue {
        self.authored_column.as_ref().unwrap_or(&self.row)
    }

    /// Serializes exactly the authored one or two specified components.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under one cumulative input, projection, and byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        self.row
            .serialize_specified_into(&mut context, &mut output)?;
        if let Some(column) = &self.authored_column {
            context.append(&mut output, " ")?;
            column.serialize_specified_into(&mut context, &mut output)?;
        }
        Ok(output)
    }
}

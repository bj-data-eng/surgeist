//! Exact authored gap values before layout and percentage resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssLengthPercentage, CssNonNegative, CssNumeric, CssSerializationError};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// One `row-gap` or `column-gap` value, retaining symbolic normal or exact numeric syntax.
///
/// # Specified serialization
///
/// Serializes this specified value without resolving layout context.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CssGapValue {
    Normal,
    LengthPercentage(CssNumeric<CssLengthPercentage, CssNonNegative>),
}

impl CssGapValue {
    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal => {
                context.charge_nodes(1)?;
                context.append(output, "normal")
            }
            Self::LengthPercentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssGapValue {
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
crate::specified_rule_serialization::specified_serialization!(CssGapValue);

impl PartialEq for CssGapValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Normal, Self::Normal) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for CssGapValue {}

/// An authored `gap` shorthand with one row value and an optional column value.
///
/// A missing column repeats the row without manufacturing a second numeric origin.
///
/// # Specified serialization
///
/// Serializes exactly the authored one or two specified components.
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
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssGapShorthand {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        context.charge_nodes(1)?;
        writer.source_member(0, |writer| {
            self.row
                .serialize_specified_into(&mut writer.context, &mut writer.css)
        })?;
        if let Some(column) = &self.authored_column {
            writer.source_member(1, |writer| {
                writer.append(" ")?;
                column.serialize_specified_into(&mut writer.context, &mut writer.css)
            })?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssGapShorthand);

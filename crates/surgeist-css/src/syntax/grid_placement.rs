//! Authored Grid placement values.

use super::{CssIdent, CssIntegerValue, CssPositiveIntegerValue};
use crate::{
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

/// A decoded Grid placement line name with Values 4 and Grid reserved words excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridLineName(CssIdent);

impl CssGridLineName {
    #[must_use]
    pub fn try_new(value: CssIdent) -> Option<Self> {
        (!matches!(
            value.as_str().to_ascii_lowercase().as_str(),
            "inherit"
                | "initial"
                | "unset"
                | "revert"
                | "revert-layer"
                | "default"
                | "auto"
                | "span"
        ))
        .then_some(Self(value))
    }

    #[must_use]
    pub fn ident(&self) -> &CssIdent {
        &self.0
    }
}

/// A nonzero ordinary line index or symbolic integer-root calculation.
/// Equality keeps integer variant and token spelling while ignoring diagnostic origins.
#[derive(Clone, Debug)]
pub struct CssGridLineIndex {
    value: CssIntegerValue,
    name: Option<CssGridLineName>,
}

impl PartialEq for CssGridLineIndex {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && match (&self.value, &other.value) {
                (CssIntegerValue::Literal(left), CssIntegerValue::Literal(right)) => left == right,
                (CssIntegerValue::ExactLiteral(left), CssIntegerValue::ExactLiteral(right)) => left
                    .component()
                    .structural_eq_ignoring_origin(right.component()),
                (CssIntegerValue::Calculation(left), CssIntegerValue::Calculation(right)) => {
                    left.structural_eq(right)
                }
                _ => false,
            }
    }
}

impl CssGridLineIndex {
    #[must_use]
    pub fn try_new(value: CssIntegerValue, name: Option<CssGridLineName>) -> Option<Self> {
        let valid = match &value {
            CssIntegerValue::Literal(number) => *number != 0,
            CssIntegerValue::ExactLiteral(number) => number
                .numeric()
                .representation()
                .strip_prefix(['+', '-'])
                .unwrap_or(number.numeric().representation())
                .bytes()
                .any(|digit| digit != b'0'),
            CssIntegerValue::Calculation(_) => true,
        };
        valid.then_some(Self { value, name })
    }

    #[must_use]
    pub fn value(&self) -> &CssIntegerValue {
        &self.value
    }

    #[must_use]
    pub fn name(&self) -> Option<&CssGridLineName> {
        self.name.as_ref()
    }
}

/// A Grid span containing a positive ordinary index, a name, or both.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGridLineSpan {
    integer: Option<CssPositiveIntegerValue>,
    name: Option<CssGridLineName>,
}

impl CssGridLineSpan {
    #[must_use]
    pub fn try_new(
        integer: Option<CssPositiveIntegerValue>,
        name: Option<CssGridLineName>,
    ) -> Option<Self> {
        (integer.is_some() || name.is_some()).then_some(Self { integer, name })
    }

    #[must_use]
    pub fn integer(&self) -> Option<&CssPositiveIntegerValue> {
        self.integer.as_ref()
    }

    #[must_use]
    pub fn name(&self) -> Option<&CssGridLineName> {
        self.name.as_ref()
    }
}

/// One authored Grid line, before placement conflict resolution.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridLine {
    Auto,
    Name(CssGridLineName),
    Indexed(CssGridLineIndex),
    Span(CssGridLineSpan),
}

impl CssGridLine {
    #[must_use]
    pub fn try_indexed(value: CssIntegerValue, name: Option<CssGridLineName>) -> Option<Self> {
        CssGridLineIndex::try_new(value, name).map(Self::Indexed)
    }

    #[must_use]
    pub fn try_span(
        integer: Option<CssPositiveIntegerValue>,
        name: Option<CssGridLineName>,
    ) -> Option<Self> {
        CssGridLineSpan::try_new(integer, name).map(Self::Span)
    }

    fn omitted_partner(&self) -> Self {
        match self {
            Self::Name(name) => Self::Name(name.clone()),
            _ => Self::Auto,
        }
    }

    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_specified(&mut writer)?;
        Ok(writer.css)
    }

    fn append_specified(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        match self {
            Self::Auto => {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append("auto")
            }
            Self::Name(name) => append_name(name, writer),
            Self::Indexed(index) => {
                index
                    .value
                    .append_specified(&mut writer.context, &mut writer.css)?;
                if let Some(name) = &index.name {
                    writer.append(" ")?;
                    append_name(name, writer)?;
                }
                Ok(())
            }
            Self::Span(span) => {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append("span")?;
                if let Some(integer) = &span.integer {
                    writer.append(" ")?;
                    match integer {
                        CssPositiveIntegerValue::Literal(value) => {
                            CssIntegerValue::Literal(value.value())
                                .append_specified(&mut writer.context, &mut writer.css)?
                        }
                        CssPositiveIntegerValue::ExactLiteral(value) => value
                            .integer()
                            .append_specified(&mut writer.context, &mut writer.css)?,
                        CssPositiveIntegerValue::Calculation(value) => {
                            value.serialize_specified_into(&mut writer.context, &mut writer.css)?
                        }
                    }
                }
                if let Some(name) = &span.name {
                    writer.append(" ")?;
                    append_name(name, writer)?;
                }
                Ok(())
            }
        }
    }
}

fn append_name(
    name: &CssGridLineName,
    writer: &mut SpecifiedRuleWriter,
) -> Result<(), CssSpecifiedValueSerializationError> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    writer.append_identifier(name.ident().as_str())
}

/// An authored row or column shorthand; an absent end remains distinguishable.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGridLineRange {
    start: CssGridLine,
    end: Option<CssGridLine>,
}

impl CssGridLineRange {
    #[must_use]
    pub const fn new(start: CssGridLine, end: Option<CssGridLine>) -> Self {
        Self { start, end }
    }
    #[must_use]
    pub const fn start(&self) -> &CssGridLine {
        &self.start
    }
    #[must_use]
    pub const fn authored_end(&self) -> Option<&CssGridLine> {
        self.end.as_ref()
    }
    #[must_use]
    pub fn effective_end(&self) -> CssGridLine {
        self.end
            .clone()
            .unwrap_or_else(|| self.start.omitted_partner())
    }
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.start.append_specified(&mut writer)?;
        if let Some(end) = &self.end {
            writer.append(" / ")?;
            end.append_specified(&mut writer)?;
        }
        Ok(writer.css)
    }
}

/// An authored area shorthand in row-start, column-start, row-end, column-end order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGridArea {
    row_start: CssGridLine,
    column_start: Option<CssGridLine>,
    row_end: Option<CssGridLine>,
    column_end: Option<CssGridLine>,
}

impl CssGridArea {
    #[must_use]
    pub fn try_new(
        row_start: CssGridLine,
        column_start: Option<CssGridLine>,
        row_end: Option<CssGridLine>,
        column_end: Option<CssGridLine>,
    ) -> Option<Self> {
        if (row_end.is_some() && column_start.is_none())
            || (column_end.is_some() && row_end.is_none())
        {
            return None;
        }
        Some(Self {
            row_start,
            column_start,
            row_end,
            column_end,
        })
    }
    #[must_use]
    pub const fn row_start(&self) -> &CssGridLine {
        &self.row_start
    }
    #[must_use]
    pub const fn authored_column_start(&self) -> Option<&CssGridLine> {
        self.column_start.as_ref()
    }
    #[must_use]
    pub const fn authored_row_end(&self) -> Option<&CssGridLine> {
        self.row_end.as_ref()
    }
    #[must_use]
    pub const fn authored_column_end(&self) -> Option<&CssGridLine> {
        self.column_end.as_ref()
    }
    #[must_use]
    pub fn effective_column_start(&self) -> CssGridLine {
        self.column_start
            .clone()
            .unwrap_or_else(|| self.row_start.omitted_partner())
    }
    #[must_use]
    pub fn effective_row_end(&self) -> CssGridLine {
        self.row_end
            .clone()
            .unwrap_or_else(|| self.row_start.omitted_partner())
    }
    #[must_use]
    pub fn effective_column_end(&self) -> CssGridLine {
        self.column_end
            .clone()
            .unwrap_or_else(|| self.effective_column_start().omitted_partner())
    }
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.row_start.append_specified(&mut writer)?;
        for line in [&self.column_start, &self.row_end, &self.column_end]
            .into_iter()
            .flatten()
        {
            writer.append(" / ")?;
            line.append_specified(&mut writer)?;
        }
        Ok(writer.css)
    }
}

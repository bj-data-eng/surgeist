//! Authored Grid placement values.

use crate::numeric::CssNumericLiteral;
use crate::numeric::{CssInteger, CssNumeric, CssPositive};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::common::CssIdent;

/// A decoded Grid line name with Values 4 and Grid reserved words excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridLineName(CssIdent);

impl CssGridLineName {
    /// Checks a decoded identifier as a grid line name.
    ///
    /// Returns `None` when the identifier is a reserved custom identifier, `auto` or `span`, in any ASCII case.
    #[must_use]
    pub fn try_new(value: CssIdent) -> Option<Self> {
        (!crate::values::common::syntax::is_reserved_custom_ident(value.as_str())
            && !value.as_str().eq_ignore_ascii_case("auto")
            && !value.as_str().eq_ignore_ascii_case("span"))
        .then_some(Self(value))
    }

    /// Returns the decoded identifier.
    #[must_use]
    pub fn ident(&self) -> &CssIdent {
        &self.0
    }
}

/// A nonzero ordinary line index or symbolic integer-root calculation.
/// Equality compares the integer value and name, ignoring spelling and diagnostic origins.
#[derive(Clone, Debug)]
pub struct CssGridLineIndex {
    value: CssNumeric<CssInteger>,
    name: Option<CssGridLineName>,
}

impl PartialEq for CssGridLineIndex {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}

impl CssGridLineIndex {
    /// Builds a line index.
    ///
    /// Returns `None` when `value` is a literal zero. A calculation is accepted without evaluation.
    #[must_use]
    pub fn try_new(value: CssNumeric<CssInteger>, name: Option<CssGridLineName>) -> Option<Self> {
        let valid = !value.literal().is_some_and(CssNumericLiteral::is_zero);
        valid.then_some(Self { value, name })
    }

    /// Returns the authored integer index.
    #[must_use]
    pub fn value(&self) -> &CssNumeric<CssInteger> {
        &self.value
    }

    /// Returns the authored line name, or `None` when it was omitted.
    #[must_use]
    pub fn name(&self) -> Option<&CssGridLineName> {
        self.name.as_ref()
    }
}

/// A Grid span containing a positive ordinary index, a name, or both.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGridLineSpan {
    integer: Option<CssNumeric<CssInteger, CssPositive>>,
    name: Option<CssGridLineName>,
}

impl CssGridLineSpan {
    /// Builds a span from its optional parts.
    ///
    /// Returns `None` when both the integer and the name are omitted.
    #[must_use]
    pub fn try_new(
        integer: Option<CssNumeric<CssInteger, CssPositive>>,
        name: Option<CssGridLineName>,
    ) -> Option<Self> {
        (integer.is_some() || name.is_some()).then_some(Self { integer, name })
    }

    /// Returns the authored positive span count, or `None` when it was omitted.
    #[must_use]
    pub fn integer(&self) -> Option<&CssNumeric<CssInteger, CssPositive>> {
        self.integer.as_ref()
    }

    /// Returns the authored line name, or `None` when it was omitted.
    #[must_use]
    pub fn name(&self) -> Option<&CssGridLineName> {
        self.name.as_ref()
    }
}

/// One authored Grid line, before placement conflict resolution.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridLine {
    /// `auto`: contributes nothing to placement, indicating auto-placement or a default span of one.
    Auto,
    /// A `<custom-ident>` that names a grid area or grid line.
    Name(CssGridLineName),
    /// A nonzero integer index, optionally with a line name.
    Indexed(CssGridLineIndex),
    /// A `span` with a positive count, a line name, or both.
    Span(CssGridLineSpan),
}

impl CssGridLine {
    /// Builds an indexed line.
    ///
    /// Returns `None` when `value` is a literal zero.
    #[must_use]
    pub fn try_indexed(
        value: CssNumeric<CssInteger>,
        name: Option<CssGridLineName>,
    ) -> Option<Self> {
        CssGridLineIndex::try_new(value, name).map(Self::Indexed)
    }

    /// Builds a span line.
    ///
    /// Returns `None` when both the integer and the name are omitted.
    #[must_use]
    pub fn try_span(
        integer: Option<CssNumeric<CssInteger, CssPositive>>,
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

    fn append_specified(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        match self {
            Self::Auto => {
                writer.node()?;
                writer.append("auto")
            }
            Self::Name(name) => writer.identifier(name.ident().as_str()),
            Self::Indexed(index) => {
                index
                    .value
                    .append_specified(&mut writer.context, &mut writer.css)?;
                if let Some(name) = &index.name {
                    writer.append(" ")?;
                    writer.identifier(name.ident().as_str())?;
                }
                Ok(())
            }
            Self::Span(span) => {
                writer.node()?;
                writer.append("span")?;
                if let Some(integer) = &span.integer {
                    writer.append(" ")?;
                    integer.append_specified(&mut writer.context, &mut writer.css)?;
                }
                if let Some(name) = &span.name {
                    writer.append(" ")?;
                    writer.identifier(name.ident().as_str())?;
                }
                Ok(())
            }
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssGridLine {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssGridLine);

/// An authored row or column shorthand; an absent end remains distinguishable.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGridLineRange {
    start: CssGridLine,
    end: Option<CssGridLine>,
}

impl CssGridLineRange {
    pub(crate) fn from_specified(start: CssGridLine, end: CssGridLine) -> Self {
        let end = (end != start.omitted_partner()).then_some(end);
        Self::new(start, end)
    }

    /// Builds a range from a start line and an optional end line; `None` records that the end was omitted.
    #[must_use]
    pub const fn new(start: CssGridLine, end: Option<CssGridLine>) -> Self {
        Self { start, end }
    }
    /// Returns the start line.
    #[must_use]
    pub const fn start(&self) -> &CssGridLine {
        &self.start
    }
    /// Returns the authored end line, or `None` when it was omitted.
    #[must_use]
    pub const fn authored_end(&self) -> Option<&CssGridLine> {
        self.end.as_ref()
    }
    /// Returns the authored end line, or the line that an omitted end implies: the start's name when the start is a name, otherwise `auto`.
    #[must_use]
    pub fn effective_end(&self) -> CssGridLine {
        self.end
            .clone()
            .unwrap_or_else(|| self.start.omitted_partner())
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssGridLineRange {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.source_member(0, |writer| self.start.append_specified(writer))?;
        if let Some(end) = &self.end {
            writer.source_member(1, |writer| {
                writer.append(" / ")?;
                end.append_specified(writer)
            })?;
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssGridLineRange);

/// An authored area shorthand in row-start, column-start, row-end, column-end order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGridArea {
    row_start: CssGridLine,
    column_start: Option<CssGridLine>,
    row_end: Option<CssGridLine>,
    column_end: Option<CssGridLine>,
}

impl CssGridArea {
    pub(crate) fn from_specified<E>(
        row_start: CssGridLine,
        column_start: CssGridLine,
        row_end: CssGridLine,
        column_end: CssGridLine,
        equal: &mut impl FnMut(&CssGridLine, &CssGridLine, usize) -> Result<bool, E>,
    ) -> Result<Self, E> {
        fn partner<'a>(line: &'a CssGridLine, auto: &'a CssGridLine) -> &'a CssGridLine {
            if matches!(line, CssGridLine::Name(_)) {
                line
            } else {
                auto
            }
        }
        let auto = CssGridLine::Auto;
        for count in 0..=3 {
            let effective_column_start = if count >= 1 {
                &column_start
            } else {
                partner(&row_start, &auto)
            };
            let effective_row_end = if count >= 2 {
                &row_end
            } else {
                partner(&row_start, &auto)
            };
            let effective_column_end = if count >= 3 {
                &column_end
            } else {
                partner(effective_column_start, &auto)
            };
            if equal(effective_column_start, &column_start, 1)?
                && equal(effective_row_end, &row_end, 2)?
                && equal(effective_column_end, &column_end, 3)?
            {
                return Ok(Self {
                    row_start,
                    column_start: (count >= 1).then_some(column_start),
                    row_end: (count >= 2).then_some(row_end),
                    column_end: (count >= 3).then_some(column_end),
                });
            }
        }
        unreachable!("full explicit area represents its four values")
    }

    /// Builds an area from its authored lines.
    ///
    /// Returns `None` when a later line is authored without the line before it in the order row-start, column-start, row-end, column-end.
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
    /// Returns the row-start line.
    #[must_use]
    pub const fn row_start(&self) -> &CssGridLine {
        &self.row_start
    }
    /// Returns the authored column-start line, or `None` when it was omitted.
    #[must_use]
    pub const fn authored_column_start(&self) -> Option<&CssGridLine> {
        self.column_start.as_ref()
    }
    /// Returns the authored row-end line, or `None` when it was omitted.
    #[must_use]
    pub const fn authored_row_end(&self) -> Option<&CssGridLine> {
        self.row_end.as_ref()
    }
    /// Returns the authored column-end line, or `None` when it was omitted.
    #[must_use]
    pub const fn authored_column_end(&self) -> Option<&CssGridLine> {
        self.column_end.as_ref()
    }
    /// Returns the authored column-start line, or the line that its omission implies from the row-start line.
    #[must_use]
    pub fn effective_column_start(&self) -> CssGridLine {
        self.column_start
            .clone()
            .unwrap_or_else(|| self.row_start.omitted_partner())
    }
    /// Returns the authored row-end line, or the line that its omission implies from the row-start line.
    #[must_use]
    pub fn effective_row_end(&self) -> CssGridLine {
        self.row_end
            .clone()
            .unwrap_or_else(|| self.row_start.omitted_partner())
    }
    /// Returns the authored column-end line, or the line that its omission implies from the effective column-start line.
    #[must_use]
    pub fn effective_column_end(&self) -> CssGridLine {
        self.column_end
            .clone()
            .unwrap_or_else(|| self.effective_column_start().omitted_partner())
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssGridArea {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.source_member(0, |writer| self.row_start.append_specified(writer))?;
        for (index, line) in [&self.column_start, &self.row_end, &self.column_end]
            .into_iter()
            .enumerate()
        {
            if let Some(line) = line {
                writer.source_member(index + 1, |writer| {
                    writer.append(" / ")?;
                    line.append_specified(writer)
                })?;
            }
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssGridArea);

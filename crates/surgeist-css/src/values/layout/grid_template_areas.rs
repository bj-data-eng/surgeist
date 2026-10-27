//! Decoded Grid template-area cells and intrinsic validity.

use std::collections::BTreeMap;

use crate::serialization::CssSerializationError;

/// A semantic failure in a decoded template-area name or row matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridTemplateAreaError {
    kind: GridTemplateAreaKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GridTemplateAreaKind {
    InvalidName,
    TrashCharacter(char),
    MissingRows,
    EmptyRow,
    InconsistentWidths,
    NonRectangular(String),
}
impl From<GridTemplateAreaKind> for CssGridTemplateAreaError {
    fn from(kind: GridTemplateAreaKind) -> Self {
        Self { kind }
    }
}
impl CssGridTemplateAreaError {
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether an area name contains a code point outside CSS ident code points.
        is_invalid_name => GridTemplateAreaKind::InvalidName,
        /// Returns whether the template has no rows.
        is_missing_rows => GridTemplateAreaKind::MissingRows,
        /// Returns whether a row has no cells.
        is_empty_row => GridTemplateAreaKind::EmptyRow,
        /// Returns whether the rows have different numbers of columns.
        is_inconsistent_widths => GridTemplateAreaKind::InconsistentWidths,
    }
    /// Returns the code point that is neither a name, a null cell nor whitespace.
    #[must_use]
    pub const fn trash_character(&self) -> Option<char> {
        match self.kind {
            GridTemplateAreaKind::TrashCharacter(character) => Some(character),
            _ => None,
        }
    }
    /// Returns the name of an area whose cells do not form one rectangle.
    #[must_use]
    pub fn non_rectangular_area(&self) -> Option<&str> {
        match &self.kind {
            GridTemplateAreaKind::NonRectangular(name) => Some(name),
            _ => None,
        }
    }
}
impl std::fmt::Display for CssGridTemplateAreaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            GridTemplateAreaKind::InvalidName => {
                f.write_str("grid area name contains a non-ident code point")
            }
            GridTemplateAreaKind::TrashCharacter(character) => write!(
                f,
                "grid template areas contain the invalid code point U+{:04X}",
                u32::from(*character)
            ),
            GridTemplateAreaKind::MissingRows => f.write_str("grid template areas have no rows"),
            GridTemplateAreaKind::EmptyRow => f.write_str("a grid template area row is empty"),
            GridTemplateAreaKind::InconsistentWidths => {
                f.write_str("grid template area rows have different numbers of columns")
            }
            GridTemplateAreaKind::NonRectangular(name) => {
                write!(f, "grid area `{name}` is not rectangular")
            }
        }
    }
}
impl std::error::Error for CssGridTemplateAreaError {}

fn is_ident_code_point(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(character, '-' | '_')
        || (character as u32) >= 0x80
}

fn is_css_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n')
}

/// A nonempty run of CSS ident code points from a decoded template string.
/// Digits and CSS-wide keywords are ordinary area names here.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CssGridTemplateAreaName {
    value: String,
}

impl CssGridTemplateAreaName {
    /// Checks the decoded string's code points; a leading digit or CSS-wide
    /// keyword is allowed because this grammar is not a custom-ident.
    pub fn try_new(value: impl Into<String>) -> Result<Self, CssGridTemplateAreaError> {
        let value = value.into();
        if value.is_empty() || !value.chars().all(is_ident_code_point) {
            return Err(CssGridTemplateAreaError::from(
                GridTemplateAreaKind::InvalidName,
            ));
        }
        Ok(Self { value })
    }

    /// Returns the decoded area name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// One decoded template cell.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplateAreaCell {
    /// A null cell token (`.`) that belongs to no area.
    Empty,
    /// A cell that belongs to the named area.
    Named(CssGridTemplateAreaName),
}

/// A nonempty sequence of decoded template cells.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridTemplateAreaRow {
    cells: Vec<CssGridTemplateAreaCell>,
}

impl CssGridTemplateAreaRow {
    /// Builds a row from its cells.
    ///
    /// # Errors
    ///
    /// Returns an error when `cells` is empty.
    pub fn try_new(cells: Vec<CssGridTemplateAreaCell>) -> Result<Self, CssGridTemplateAreaError> {
        if cells.is_empty() {
            Err(CssGridTemplateAreaError::from(
                GridTemplateAreaKind::EmptyRow,
            ))
        } else {
            Ok(Self { cells })
        }
    }

    /// Returns the cells in order; the row is never empty.
    #[must_use]
    pub fn cells(&self) -> &[CssGridTemplateAreaCell] {
        &self.cells
    }
}

/// A checked nonempty matrix with equal row widths and filled rectangular names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridTemplateAreaRows {
    rows: Vec<CssGridTemplateAreaRow>,
}

impl CssGridTemplateAreaRows {
    /// Builds the matrix from its rows.
    ///
    /// # Errors
    ///
    /// Returns an error when there are no rows, the rows are empty, the rows have different widths, or a named area does not form one rectangle.
    pub fn try_new(rows: Vec<CssGridTemplateAreaRow>) -> Result<Self, CssGridTemplateAreaError> {
        validate_area_matrix(&rows, CssGridTemplateAreaRow::cells, |cell| match cell {
            CssGridTemplateAreaCell::Empty => None,
            CssGridTemplateAreaCell::Named(name) => Some(name.as_str()),
        })?;
        Ok(Self { rows })
    }

    /// Returns the rows in order; the matrix is never empty.
    #[must_use]
    pub fn rows(&self) -> &[CssGridTemplateAreaRow] {
        &self.rows
    }
}

/// An authored `grid-template-areas` value.
///
/// # Specified serialization
///
/// Emits one quoted row per string, with one dot per null cell and one space between cells.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplateAreas {
    /// `none`: no areas are defined.
    None,
    /// A rectangular matrix of area rows.
    Rows(CssGridTemplateAreaRows),
}

impl CssGridTemplateAreas {
    /// Builds a `grid-template-areas` value from rows.
    ///
    /// # Errors
    ///
    /// Returns an error when there are no rows, the rows are empty, the rows have different widths, or a named area does not form one rectangle.
    pub fn try_rows(rows: Vec<CssGridTemplateAreaRow>) -> Result<Self, CssGridTemplateAreaError> {
        CssGridTemplateAreaRows::try_new(rows).map(Self::Rows)
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssGridTemplateAreas
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        context.charge_nodes(1)?;
        let output = &mut writer.css;
        match self {
            Self::None => context.append(output, "none")?,
            Self::Rows(rows) => {
                for (row_index, row) in rows.rows().iter().enumerate() {
                    write_area_row(row, context, output, row_index > 0)?;
                }
            }
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssGridTemplateAreas);

// Shared with authored area-template rows; no fresh budget or aggregate charge.
pub(crate) fn write_area_row(
    row: &CssGridTemplateAreaRow,
    context: &mut crate::serialization::specified_serialization::SpecifiedSerializationContext,
    output: &mut String,
    leading_space: bool,
) -> Result<(), CssSerializationError> {
    context.charge_nodes(1)?;
    // Matrix row charges precede its separator, preserving failure precedence
    // when both a node budget and the next separator byte are exhausted.
    if leading_space {
        context.append(output, " ")?;
    }
    context.append(output, "\"")?;
    for (cell_index, cell) in row.cells().iter().enumerate() {
        context.charge_nodes(1)?;
        if cell_index > 0 {
            context.append(output, " ")?;
        }
        match cell {
            CssGridTemplateAreaCell::Empty => context.append(output, ".")?,
            // Checked names contain only ident code points. Non-ASCII space-like
            // code points remain data; ASCII quoting/escape delimiters cannot occur.
            CssGridTemplateAreaCell::Named(name) => context.append(output, name.as_str())?,
        }
    }
    context.append(output, "\"")
}

/// Tokenizes a decoded CSS string with Grid 2 longest-run semantics.
pub(crate) fn parse_decoded_row(
    source: &str,
) -> Result<CssGridTemplateAreaRow, CssGridTemplateAreaError> {
    let mut characters = source.chars().peekable();
    let mut cells = Vec::new();
    while let Some(&next) = characters.peek() {
        if is_css_whitespace(next) {
            while characters
                .next_if(|character| is_css_whitespace(*character))
                .is_some()
            {}
        } else if next == '.' {
            while characters.next_if_eq(&'.').is_some() {}
            cells.push(CssGridTemplateAreaCell::Empty);
        } else if is_ident_code_point(next) {
            let mut name = String::new();
            while let Some(character) =
                characters.next_if(|character| is_ident_code_point(*character))
            {
                name.push(character);
            }
            cells.push(CssGridTemplateAreaCell::Named(
                CssGridTemplateAreaName::try_new(name)?,
            ));
        } else {
            return Err(CssGridTemplateAreaError::from(
                GridTemplateAreaKind::TrashCharacter(next),
            ));
        }
    }
    CssGridTemplateAreaRow::try_new(cells)
}

#[derive(Clone, Copy)]
struct AreaBounds {
    min_row: usize,
    max_row: usize,
    min_column: usize,
    max_column: usize,
    count: usize,
}

/// Intrinsic validation for decoded area rows.
pub(crate) fn validate_area_matrix<R, C>(
    rows: &[R],
    cells: impl Fn(&R) -> &[C],
    name: impl Fn(&C) -> Option<&str>,
) -> Result<(), CssGridTemplateAreaError> {
    let Some(first) = rows.first() else {
        return Err(CssGridTemplateAreaError::from(
            GridTemplateAreaKind::MissingRows,
        ));
    };
    let width = cells(first).len();
    if width == 0 {
        return Err(CssGridTemplateAreaError::from(
            GridTemplateAreaKind::EmptyRow,
        ));
    }
    if rows.iter().any(|row| cells(row).len() != width) {
        return Err(CssGridTemplateAreaError::from(
            GridTemplateAreaKind::InconsistentWidths,
        ));
    }
    let mut bounds = BTreeMap::<String, AreaBounds>::new();
    for (row_index, row) in rows.iter().enumerate() {
        for (column_index, cell) in cells(row).iter().enumerate() {
            let Some(name) = name(cell) else { continue };
            bounds
                .entry(name.to_owned())
                .and_modify(|bounds| {
                    bounds.min_row = bounds.min_row.min(row_index);
                    bounds.max_row = bounds.max_row.max(row_index);
                    bounds.min_column = bounds.min_column.min(column_index);
                    bounds.max_column = bounds.max_column.max(column_index);
                    bounds.count += 1;
                })
                .or_insert(AreaBounds {
                    min_row: row_index,
                    max_row: row_index,
                    min_column: column_index,
                    max_column: column_index,
                    count: 1,
                });
        }
    }
    for (name, bounds) in bounds {
        if (bounds.max_row - bounds.min_row + 1)
            .checked_mul(bounds.max_column - bounds.min_column + 1)
            != Some(bounds.count)
        {
            return Err(CssGridTemplateAreaError::from(
                GridTemplateAreaKind::NonRectangular(name),
            ));
        }
    }
    Ok(())
}

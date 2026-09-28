//! Decoded Grid template-area cells and intrinsic validity.

use std::collections::BTreeMap;

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits};

/// A semantic failure in a decoded template-area name or row matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplateAreaError {
    InvalidName,
    TrashCharacter(char),
    MissingRows,
    EmptyRow,
    InconsistentWidths,
    NonRectangular(String),
}

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
            return Err(CssGridTemplateAreaError::InvalidName);
        }
        Ok(Self { value })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// One decoded template cell.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplateAreaCell {
    Empty,
    Named(CssGridTemplateAreaName),
}

/// A nonempty sequence of decoded template cells.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridTemplateAreaRow {
    cells: Vec<CssGridTemplateAreaCell>,
}

impl CssGridTemplateAreaRow {
    pub fn try_new(cells: Vec<CssGridTemplateAreaCell>) -> Result<Self, CssGridTemplateAreaError> {
        if cells.is_empty() {
            Err(CssGridTemplateAreaError::EmptyRow)
        } else {
            Ok(Self { cells })
        }
    }

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
    pub fn try_new(rows: Vec<CssGridTemplateAreaRow>) -> Result<Self, CssGridTemplateAreaError> {
        validate_area_matrix(&rows, CssGridTemplateAreaRow::cells, |cell| match cell {
            CssGridTemplateAreaCell::Empty => None,
            CssGridTemplateAreaCell::Named(name) => Some(name.as_str()),
        })?;
        Ok(Self { rows })
    }

    #[must_use]
    pub fn rows(&self) -> &[CssGridTemplateAreaRow] {
        &self.rows
    }
}

/// An authored `grid-template-areas` value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplateAreas {
    None,
    Rows(CssGridTemplateAreaRows),
}

impl CssGridTemplateAreas {
    pub fn try_rows(rows: Vec<CssGridTemplateAreaRow>) -> Result<Self, CssGridTemplateAreaError> {
        CssGridTemplateAreaRows::try_new(rows).map(Self::Rows)
    }

    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Emits one quoted row per string, with one dot per null cell and one space
    /// between cells. The value, each row and each cell charge both node budgets.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut context = SpecifiedSerializationContext::new(limits);
        context.charge_input(1)?;
        context.charge_projection(1)?;
        let mut output = String::new();
        match self {
            Self::None => context.append(&mut output, "none")?,
            Self::Rows(rows) => {
                for (row_index, row) in rows.rows().iter().enumerate() {
                    context.charge_input(1)?;
                    context.charge_projection(1)?;
                    if row_index > 0 {
                        context.append(&mut output, " ")?;
                    }
                    context.append(&mut output, "\"")?;
                    for (cell_index, cell) in row.cells().iter().enumerate() {
                        context.charge_input(1)?;
                        context.charge_projection(1)?;
                        if cell_index > 0 {
                            context.append(&mut output, " ")?;
                        }
                        match cell {
                            CssGridTemplateAreaCell::Empty => context.append(&mut output, ".")?,
                            // Checked names contain only ident code points: no ASCII
                            // quote, backslash, line break, or CSS whitespace can
                            // disturb the enclosing CSS string token. Non-ASCII
                            // code points, including U+0085 and NBSP, remain data.
                            CssGridTemplateAreaCell::Named(name) => {
                                context.append(&mut output, name.as_str())?
                            }
                        }
                    }
                    context.append(&mut output, "\"")?;
                }
            }
        }
        Ok(output)
    }
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
            return Err(CssGridTemplateAreaError::TrashCharacter(next));
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
        return Err(CssGridTemplateAreaError::MissingRows);
    };
    let width = cells(first).len();
    if width == 0 {
        return Err(CssGridTemplateAreaError::EmptyRow);
    }
    if rows.iter().any(|row| cells(row).len() != width) {
        return Err(CssGridTemplateAreaError::InconsistentWidths);
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
            return Err(CssGridTemplateAreaError::NonRectangular(name));
        }
    }
    Ok(())
}

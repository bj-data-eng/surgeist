use super::{CssCustomIdent, CssLength, CssNonNegativeNumber};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridTrackBreadth {
    Length(CssLength),
    Fraction(CssNonNegativeNumber),
    MinContent,
    MaxContent,
    Auto,
}

impl CssGridTrackBreadth {
    #[must_use]
    pub const fn length(length: CssLength) -> Self {
        Self::Length(length)
    }

    #[must_use]
    pub fn try_fraction(value: f32) -> Option<Self> {
        CssNonNegativeNumber::try_new(value).map(Self::Fraction)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridTrackSize {
    Breadth(CssGridTrackBreadth),
    MinMax {
        min: CssGridTrackBreadth,
        max: CssGridTrackBreadth,
    },
    FitContent(CssLength),
}

impl CssGridTrackSize {
    #[must_use]
    pub const fn breadth(breadth: CssGridTrackBreadth) -> Self {
        Self::Breadth(breadth)
    }

    #[must_use]
    pub const fn minmax(min: CssGridTrackBreadth, max: CssGridTrackBreadth) -> Self {
        Self::MinMax { min, max }
    }

    #[must_use]
    pub const fn fit_content(limit: CssLength) -> Self {
        Self::FitContent(limit)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridLineNames {
    names: Vec<CssCustomIdent>,
}

impl CssGridLineNames {
    /// Creates an authored line-name group, including the valid empty group `[]`.
    #[must_use]
    pub fn try_new(names: Vec<CssCustomIdent>) -> Option<Self> {
        Some(Self::new(names))
    }

    #[must_use]
    pub(crate) fn new(names: Vec<CssCustomIdent>) -> Self {
        Self { names }
    }

    #[must_use]
    pub fn names(&self) -> &[CssCustomIdent] {
        &self.names
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridTrackComponent {
    LineNames(CssGridLineNames),
    TrackSize(CssGridTrackSize),
    Repeat(CssGridRepeat),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssGridTrackList {
    components: Vec<CssGridTrackComponent>,
}

impl CssGridTrackList {
    #[must_use]
    pub fn try_new(components: Vec<CssGridTrackComponent>) -> Option<Self> {
        if components.is_empty() {
            None
        } else {
            Some(Self::new(components))
        }
    }

    #[must_use]
    pub(crate) fn new(components: Vec<CssGridTrackComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssGridTrackComponent] {
        &self.components
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridRepeatCount {
    Integer(CssGridRepeatInteger),
    AutoFill,
    AutoFit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssGridRepeatInteger {
    value: i32,
}

impl CssGridRepeatInteger {
    #[must_use]
    pub const fn try_new(value: i32) -> Option<Self> {
        if value > 0 {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

impl CssGridRepeatCount {
    #[must_use]
    pub const fn try_integer(value: i32) -> Option<Self> {
        match CssGridRepeatInteger::try_new(value) {
            Some(value) => Some(Self::Integer(value)),
            None => None,
        }
    }

    #[must_use]
    pub(crate) const fn integer(value: i32) -> Self {
        match Self::try_integer(value) {
            Some(value) => value,
            None => panic!("grid repeat integer must be positive"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssGridRepeat {
    count: CssGridRepeatCount,
    tracks: CssGridTrackList,
}

impl CssGridRepeat {
    #[must_use]
    pub fn try_new(count: CssGridRepeatCount, tracks: CssGridTrackList) -> Option<Self> {
        if tracks.components().is_empty() {
            None
        } else {
            Some(Self::new(count, tracks))
        }
    }

    #[must_use]
    pub(crate) const fn new(count: CssGridRepeatCount, tracks: CssGridTrackList) -> Self {
        Self { count, tracks }
    }

    #[must_use]
    pub const fn count(&self) -> CssGridRepeatCount {
        self.count
    }

    #[must_use]
    pub const fn tracks(&self) -> &CssGridTrackList {
        &self.tracks
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplateAreaCell {
    Empty,
    Named(CssCustomIdent),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridTemplateAreaRow {
    cells: Vec<CssGridTemplateAreaCell>,
}

impl CssGridTemplateAreaRow {
    #[must_use]
    pub fn try_new(cells: Vec<CssGridTemplateAreaCell>) -> Option<Self> {
        if cells.is_empty() {
            None
        } else {
            Some(Self::new(cells))
        }
    }

    #[must_use]
    pub(crate) fn new(cells: Vec<CssGridTemplateAreaCell>) -> Self {
        Self { cells }
    }

    #[must_use]
    pub fn cells(&self) -> &[CssGridTemplateAreaCell] {
        &self.cells
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplateAreas {
    None,
    Rows(CssGridTemplateAreaRows),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridTemplateAreaRows {
    rows: Vec<CssGridTemplateAreaRow>,
}

impl CssGridTemplateAreaRows {
    #[must_use]
    pub fn try_new(rows: Vec<CssGridTemplateAreaRow>) -> Option<Self> {
        if validate_grid_template_area_rows(&rows).is_ok() {
            Some(Self { rows })
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new_unchecked(rows: Vec<CssGridTemplateAreaRow>) -> Self {
        Self { rows }
    }

    #[must_use]
    pub fn rows(&self) -> &[CssGridTemplateAreaRow] {
        &self.rows
    }
}

impl CssGridTemplateAreas {
    #[must_use]
    pub fn try_rows(rows: Vec<CssGridTemplateAreaRow>) -> Option<Self> {
        CssGridTemplateAreaRows::try_new(rows).map(Self::Rows)
    }

    #[must_use]
    pub(crate) fn rows(rows: Vec<CssGridTemplateAreaRow>) -> Self {
        Self::Rows(CssGridTemplateAreaRows::new_unchecked(rows))
    }
}

pub(crate) enum GridAreaValidationError {
    MissingRows,
    EmptyRow,
    InconsistentWidths,
    NonRectangular(String),
}

pub(crate) fn validate_grid_template_area_rows(
    rows: &[CssGridTemplateAreaRow],
) -> Result<(), GridAreaValidationError> {
    if rows.is_empty() {
        return Err(GridAreaValidationError::MissingRows);
    }
    let width = rows[0].cells().len();
    if width == 0 {
        return Err(GridAreaValidationError::EmptyRow);
    }
    if rows.iter().any(|row| row.cells().len() != width) {
        return Err(GridAreaValidationError::InconsistentWidths);
    }

    let mut bounds = HashMap::<String, GridAreaBounds>::new();
    for (row_index, row) in rows.iter().enumerate() {
        for (col_index, cell) in row.cells().iter().enumerate() {
            let CssGridTemplateAreaCell::Named(name) = cell else {
                continue;
            };
            bounds
                .entry(name.as_str().to_owned())
                .and_modify(|bounds| {
                    bounds.min_row = bounds.min_row.min(row_index);
                    bounds.max_row = bounds.max_row.max(row_index);
                    bounds.min_col = bounds.min_col.min(col_index);
                    bounds.max_col = bounds.max_col.max(col_index);
                    bounds.count += 1;
                })
                .or_insert(GridAreaBounds {
                    min_row: row_index,
                    max_row: row_index,
                    min_col: col_index,
                    max_col: col_index,
                    count: 1,
                });
        }
    }

    for (name, bounds) in bounds {
        let rectangle_area =
            (bounds.max_row - bounds.min_row + 1) * (bounds.max_col - bounds.min_col + 1);
        if rectangle_area != bounds.count {
            return Err(GridAreaValidationError::NonRectangular(name));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct GridAreaBounds {
    min_row: usize,
    max_row: usize,
    min_col: usize,
    max_col: usize,
    count: usize,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplate {
    None,
    RowsColumns {
        rows: CssGridTrackList,
        columns: Option<CssGridTrackList>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridAutoFlowAxis {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssGridAutoFlow {
    axis: CssGridAutoFlowAxis,
    dense: bool,
}

impl CssGridAutoFlow {
    #[must_use]
    pub const fn new(axis: CssGridAutoFlowAxis, dense: bool) -> Self {
        Self { axis, dense }
    }

    #[must_use]
    pub const fn axis(self) -> CssGridAutoFlowAxis {
        self.axis
    }

    #[must_use]
    pub const fn dense(self) -> bool {
        self.dense
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridLine {
    Auto,
    Integer(CssGridLineInteger),
    CustomIdent(CssCustomIdent),
    Span(CssGridLineSpan),
}

impl CssGridLine {
    #[must_use]
    pub fn try_integer(value: i32) -> Option<Self> {
        CssGridLineInteger::try_new(value).map(Self::Integer)
    }

    #[must_use]
    pub(crate) fn integer(value: i32) -> Self {
        match Self::try_integer(value) {
            Some(value) => value,
            None => panic!("grid line integer must be non-zero"),
        }
    }

    #[must_use]
    pub fn try_span(integer: Option<i32>, name: Option<CssCustomIdent>) -> Option<Self> {
        CssGridLineSpan::try_new(integer, name).map(Self::Span)
    }

    #[must_use]
    pub(crate) fn span(integer: Option<i32>, name: Option<CssCustomIdent>) -> Self {
        Self::Span(CssGridLineSpan::new(integer, name))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssGridLineInteger {
    value: i32,
}

impl CssGridLineInteger {
    #[must_use]
    pub const fn try_new(value: i32) -> Option<Self> {
        if value != 0 {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridLineSpan {
    integer: Option<CssGridSpanInteger>,
    name: Option<CssCustomIdent>,
}

impl CssGridLineSpan {
    #[must_use]
    pub fn try_new(integer: Option<i32>, name: Option<CssCustomIdent>) -> Option<Self> {
        let integer = match integer {
            Some(value) => Some(CssGridSpanInteger::try_new(value)?),
            None => None,
        };
        if integer.is_none() && name.is_none() {
            None
        } else {
            Some(Self { integer, name })
        }
    }

    #[must_use]
    pub(crate) fn new(integer: Option<i32>, name: Option<CssCustomIdent>) -> Self {
        match Self::try_new(integer, name) {
            Some(value) => value,
            None => panic!("grid span must include a positive integer or name"),
        }
    }

    #[must_use]
    pub const fn integer(&self) -> Option<i32> {
        match self.integer {
            Some(value) => Some(value.value()),
            None => None,
        }
    }

    #[must_use]
    pub const fn name(&self) -> Option<&CssCustomIdent> {
        self.name.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssGridSpanInteger {
    value: i32,
}

impl CssGridSpanInteger {
    #[must_use]
    pub const fn try_new(value: i32) -> Option<Self> {
        if value > 0 {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
    pub const fn end(&self) -> Option<&CssGridLine> {
        self.end.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridArea {
    row_start: CssGridLine,
    column_start: Option<CssGridLine>,
    row_end: Option<CssGridLine>,
    column_end: Option<CssGridLine>,
}

impl CssGridArea {
    #[must_use]
    pub const fn new(
        row_start: CssGridLine,
        column_start: Option<CssGridLine>,
        row_end: Option<CssGridLine>,
        column_end: Option<CssGridLine>,
    ) -> Self {
        Self {
            row_start,
            column_start,
            row_end,
            column_end,
        }
    }

    #[must_use]
    pub const fn row_start(&self) -> &CssGridLine {
        &self.row_start
    }

    #[must_use]
    pub const fn column_start(&self) -> Option<&CssGridLine> {
        self.column_start.as_ref()
    }

    #[must_use]
    pub const fn row_end(&self) -> Option<&CssGridLine> {
        self.row_end.as_ref()
    }

    #[must_use]
    pub const fn column_end(&self) -> Option<&CssGridLine> {
        self.column_end.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGrid {
    Template(CssGridTemplate),
    AutoFlow {
        flow: CssGridAutoFlow,
        auto_tracks: Option<CssGridTrackList>,
        explicit_tracks: CssGridTrackList,
    },
}

/// The semantic branch of a parser-owned current Grid track breadth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridTrackBreadthKind {
    Length,
    Fraction,
    MinContent,
    MaxContent,
    Auto,
}

/// A parser-owned current authored Grid track breadth.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackBreadth {
    representation: CssAuthoredGridTrackBreadthRepresentation,
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridTrackBreadthRepresentation {
    Length(CssLength),
    Fraction(CssNonNegativeNumber),
    MinContent,
    MaxContent,
    Auto,
}

impl CssAuthoredGridTrackBreadth {
    pub(crate) const fn from_length(value: CssLength) -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::Length(value),
        }
    }

    pub(crate) const fn from_fraction(value: CssNonNegativeNumber) -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::Fraction(value),
        }
    }

    pub(crate) const fn min_content() -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::MinContent,
        }
    }

    pub(crate) const fn max_content() -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::MaxContent,
        }
    }

    pub(crate) const fn auto() -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::Auto,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssAuthoredGridTrackBreadthKind {
        match self.representation {
            CssAuthoredGridTrackBreadthRepresentation::Length(_) => {
                CssAuthoredGridTrackBreadthKind::Length
            }
            CssAuthoredGridTrackBreadthRepresentation::Fraction(_) => {
                CssAuthoredGridTrackBreadthKind::Fraction
            }
            CssAuthoredGridTrackBreadthRepresentation::MinContent => {
                CssAuthoredGridTrackBreadthKind::MinContent
            }
            CssAuthoredGridTrackBreadthRepresentation::MaxContent => {
                CssAuthoredGridTrackBreadthKind::MaxContent
            }
            CssAuthoredGridTrackBreadthRepresentation::Auto => {
                CssAuthoredGridTrackBreadthKind::Auto
            }
        }
    }

    #[must_use]
    pub const fn length(&self) -> Option<&CssLength> {
        match &self.representation {
            CssAuthoredGridTrackBreadthRepresentation::Length(value) => Some(value),
            CssAuthoredGridTrackBreadthRepresentation::Fraction(_)
            | CssAuthoredGridTrackBreadthRepresentation::MinContent
            | CssAuthoredGridTrackBreadthRepresentation::MaxContent
            | CssAuthoredGridTrackBreadthRepresentation::Auto => None,
        }
    }

    #[must_use]
    pub const fn fraction(&self) -> Option<CssNonNegativeNumber> {
        match self.representation {
            CssAuthoredGridTrackBreadthRepresentation::Fraction(value) => Some(value),
            CssAuthoredGridTrackBreadthRepresentation::Length(_)
            | CssAuthoredGridTrackBreadthRepresentation::MinContent
            | CssAuthoredGridTrackBreadthRepresentation::MaxContent
            | CssAuthoredGridTrackBreadthRepresentation::Auto => None,
        }
    }

    pub(crate) fn i01_projection(&self) -> Option<CssGridTrackBreadth> {
        Some(match &self.representation {
            CssAuthoredGridTrackBreadthRepresentation::Length(value) => {
                if matches!(value, CssLength::Calc(_)) {
                    return None;
                }
                CssGridTrackBreadth::length(value.clone())
            }
            CssAuthoredGridTrackBreadthRepresentation::Fraction(value) => {
                CssGridTrackBreadth::Fraction(*value)
            }
            CssAuthoredGridTrackBreadthRepresentation::MinContent => {
                CssGridTrackBreadth::MinContent
            }
            CssAuthoredGridTrackBreadthRepresentation::MaxContent => {
                CssGridTrackBreadth::MaxContent
            }
            CssAuthoredGridTrackBreadthRepresentation::Auto => CssGridTrackBreadth::Auto,
        })
    }

    pub(crate) const fn is_fixed(&self) -> bool {
        matches!(
            self.representation,
            CssAuthoredGridTrackBreadthRepresentation::Length(_)
        )
    }

    pub(crate) const fn is_inflexible(&self) -> bool {
        !matches!(
            self.representation,
            CssAuthoredGridTrackBreadthRepresentation::Fraction(_)
        )
    }
}

/// The semantic branch of a current authored Grid track size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridTrackSizeKind {
    Breadth,
    MinMax,
    FitContent,
}

/// A parser-owned current authored Grid track size.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackSize {
    representation: CssAuthoredGridTrackSizeRepresentation,
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridTrackSizeRepresentation {
    Breadth(CssAuthoredGridTrackBreadth),
    MinMax {
        min: CssAuthoredGridTrackBreadth,
        max: CssAuthoredGridTrackBreadth,
    },
    FitContent(CssLength),
}

impl CssAuthoredGridTrackSize {
    pub(crate) const fn from_breadth(value: CssAuthoredGridTrackBreadth) -> Self {
        Self {
            representation: CssAuthoredGridTrackSizeRepresentation::Breadth(value),
        }
    }

    pub(crate) const fn from_minmax(
        min: CssAuthoredGridTrackBreadth,
        max: CssAuthoredGridTrackBreadth,
    ) -> Self {
        Self {
            representation: CssAuthoredGridTrackSizeRepresentation::MinMax { min, max },
        }
    }

    pub(crate) const fn from_fit_content(value: CssLength) -> Self {
        Self {
            representation: CssAuthoredGridTrackSizeRepresentation::FitContent(value),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssAuthoredGridTrackSizeKind {
        match self.representation {
            CssAuthoredGridTrackSizeRepresentation::Breadth(_) => {
                CssAuthoredGridTrackSizeKind::Breadth
            }
            CssAuthoredGridTrackSizeRepresentation::MinMax { .. } => {
                CssAuthoredGridTrackSizeKind::MinMax
            }
            CssAuthoredGridTrackSizeRepresentation::FitContent(_) => {
                CssAuthoredGridTrackSizeKind::FitContent
            }
        }
    }

    #[must_use]
    pub const fn breadth(&self) -> Option<&CssAuthoredGridTrackBreadth> {
        match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::Breadth(value) => Some(value),
            CssAuthoredGridTrackSizeRepresentation::MinMax { .. }
            | CssAuthoredGridTrackSizeRepresentation::FitContent(_) => None,
        }
    }

    #[must_use]
    pub const fn minmax(
        &self,
    ) -> Option<(&CssAuthoredGridTrackBreadth, &CssAuthoredGridTrackBreadth)> {
        match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::MinMax { min, max } => Some((min, max)),
            CssAuthoredGridTrackSizeRepresentation::Breadth(_)
            | CssAuthoredGridTrackSizeRepresentation::FitContent(_) => None,
        }
    }

    #[must_use]
    pub const fn fit_content(&self) -> Option<&CssLength> {
        match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::FitContent(value) => Some(value),
            CssAuthoredGridTrackSizeRepresentation::Breadth(_)
            | CssAuthoredGridTrackSizeRepresentation::MinMax { .. } => None,
        }
    }

    pub(crate) fn i01_projection(&self) -> Option<CssGridTrackSize> {
        Some(match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::Breadth(value) => {
                CssGridTrackSize::breadth(value.i01_projection()?)
            }
            CssAuthoredGridTrackSizeRepresentation::MinMax { min, max } => {
                CssGridTrackSize::minmax(min.i01_projection()?, max.i01_projection()?)
            }
            CssAuthoredGridTrackSizeRepresentation::FitContent(value) => {
                if matches!(value, CssLength::Calc(_)) {
                    return None;
                }
                CssGridTrackSize::fit_content(value.clone())
            }
        })
    }

    pub(crate) const fn is_fixed(&self) -> bool {
        match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::Breadth(value) => value.is_fixed(),
            CssAuthoredGridTrackSizeRepresentation::MinMax { min, max } => {
                min.is_fixed() || (min.is_inflexible() && max.is_fixed())
            }
            CssAuthoredGridTrackSizeRepresentation::FitContent(_) => false,
        }
    }
}

/// A fixed size admitted around automatic repetition or in an integer fixed-repeat.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridFixedSize {
    size: CssAuthoredGridTrackSize,
}

impl CssAuthoredGridFixedSize {
    pub(crate) const fn new(size: CssAuthoredGridTrackSize) -> Self {
        Self { size }
    }

    #[must_use]
    pub const fn size(&self) -> &CssAuthoredGridTrackSize {
        &self.size
    }
}

/// One non-recursive member of integer or automatic track-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridTrackRepeatComponent {
    LineNames(CssGridLineNames),
    TrackSize(CssAuthoredGridTrackSize),
}

/// Non-empty, non-recursive integer or automatic track-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackRepeatContent {
    components: Vec<CssAuthoredGridTrackRepeatComponent>,
}

impl CssAuthoredGridTrackRepeatContent {
    pub(crate) const fn new(components: Vec<CssAuthoredGridTrackRepeatComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssAuthoredGridTrackRepeatComponent] {
        &self.components
    }
}

/// One non-recursive member of fixed-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridFixedRepeatComponent {
    LineNames(CssGridLineNames),
    FixedSize(CssAuthoredGridFixedSize),
}

/// Non-empty, non-recursive fixed-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridFixedRepeatContent {
    components: Vec<CssAuthoredGridFixedRepeatComponent>,
}

impl CssAuthoredGridFixedRepeatContent {
    pub(crate) const fn new(components: Vec<CssAuthoredGridFixedRepeatComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssAuthoredGridFixedRepeatComponent] {
        &self.components
    }
}

/// A positive-integer repetition whose content may use any track size.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridIntegerTrackRepeat {
    count: CssGridRepeatInteger,
    content: CssAuthoredGridTrackRepeatContent,
}

impl CssAuthoredGridIntegerTrackRepeat {
    pub(crate) const fn new(
        count: CssGridRepeatInteger,
        content: CssAuthoredGridTrackRepeatContent,
    ) -> Self {
        Self { count, content }
    }

    #[must_use]
    pub const fn count(&self) -> CssGridRepeatInteger {
        self.count
    }

    #[must_use]
    pub const fn content(&self) -> &CssAuthoredGridTrackRepeatContent {
        &self.content
    }
}

/// A positive-integer repetition constrained to fixed-size content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridIntegerFixedRepeat {
    count: CssGridRepeatInteger,
    content: CssAuthoredGridFixedRepeatContent,
}

impl CssAuthoredGridIntegerFixedRepeat {
    pub(crate) const fn new(
        count: CssGridRepeatInteger,
        content: CssAuthoredGridFixedRepeatContent,
    ) -> Self {
        Self { count, content }
    }

    #[must_use]
    pub const fn count(&self) -> CssGridRepeatInteger {
        self.count
    }

    #[must_use]
    pub const fn content(&self) -> &CssAuthoredGridFixedRepeatContent {
        &self.content
    }
}

/// The automatic repetition mode in an authored Grid track list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridAutoRepeatKind {
    AutoFill,
    AutoFit,
}

/// The single automatic repetition admitted by an auto track list.
///
/// Its non-recursive body admits general track sizes under CSS Grid 3. Surrounding
/// tracks and integer repetitions in the same list remain constrained to fixed sizes.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridAutoRepeat {
    kind: CssAuthoredGridAutoRepeatKind,
    content: CssAuthoredGridTrackRepeatContent,
}

impl CssAuthoredGridAutoRepeat {
    pub(crate) const fn new(
        kind: CssAuthoredGridAutoRepeatKind,
        content: CssAuthoredGridTrackRepeatContent,
    ) -> Self {
        Self { kind, content }
    }

    #[must_use]
    pub const fn kind(&self) -> CssAuthoredGridAutoRepeatKind {
        self.kind
    }

    #[must_use]
    pub const fn content(&self) -> &CssAuthoredGridTrackRepeatContent {
        &self.content
    }
}

/// One component of a general track list, which never contains automatic repetition.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridGeneralTrackComponent {
    LineNames(CssGridLineNames),
    TrackSize(CssAuthoredGridTrackSize),
    Repeat(CssAuthoredGridIntegerTrackRepeat),
}

/// A non-empty general Grid track list.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridGeneralTrackList {
    components: Vec<CssAuthoredGridGeneralTrackComponent>,
}

impl CssAuthoredGridGeneralTrackList {
    pub(crate) const fn new(components: Vec<CssAuthoredGridGeneralTrackComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssAuthoredGridGeneralTrackComponent] {
        &self.components
    }
}

/// One component of an auto track list.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridAutoTrackComponent {
    LineNames(CssGridLineNames),
    FixedSize(CssAuthoredGridFixedSize),
    Repeat(CssAuthoredGridIntegerFixedRepeat),
    AutoRepeat(CssAuthoredGridAutoRepeat),
}

/// A Grid track list containing exactly one automatic repetition.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridAutoTrackList {
    components: Vec<CssAuthoredGridAutoTrackComponent>,
}

impl CssAuthoredGridAutoTrackList {
    pub(crate) const fn new(components: Vec<CssAuthoredGridAutoTrackComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssAuthoredGridAutoTrackComponent] {
        &self.components
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridTrackListRepresentation {
    General(CssAuthoredGridGeneralTrackList),
    Auto(CssAuthoredGridAutoTrackList),
}

/// A parser-owned current Grid track list, classified as general or automatic.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackList {
    representation: CssAuthoredGridTrackListRepresentation,
}

impl CssAuthoredGridTrackList {
    pub(crate) const fn general(value: CssAuthoredGridGeneralTrackList) -> Self {
        Self {
            representation: CssAuthoredGridTrackListRepresentation::General(value),
        }
    }

    pub(crate) const fn auto(value: CssAuthoredGridAutoTrackList) -> Self {
        Self {
            representation: CssAuthoredGridTrackListRepresentation::Auto(value),
        }
    }

    #[must_use]
    pub const fn general_list(&self) -> Option<&CssAuthoredGridGeneralTrackList> {
        match &self.representation {
            CssAuthoredGridTrackListRepresentation::General(value) => Some(value),
            CssAuthoredGridTrackListRepresentation::Auto(_) => None,
        }
    }

    #[must_use]
    pub const fn auto_list(&self) -> Option<&CssAuthoredGridAutoTrackList> {
        match &self.representation {
            CssAuthoredGridTrackListRepresentation::Auto(value) => Some(value),
            CssAuthoredGridTrackListRepresentation::General(_) => None,
        }
    }
}

/// A non-empty authored list for `grid-auto-rows` or `grid-auto-columns`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackSizeList {
    sizes: Vec<CssAuthoredGridTrackSize>,
}

impl CssAuthoredGridTrackSizeList {
    pub(crate) const fn new(sizes: Vec<CssAuthoredGridTrackSize>) -> Self {
        Self { sizes }
    }

    #[must_use]
    pub fn sizes(&self) -> &[CssAuthoredGridTrackSize] {
        &self.sizes
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridTemplateRepresentation {
    None,
    RowsColumns {
        rows: CssAuthoredGridTrackList,
        columns: Option<CssAuthoredGridTrackList>,
    },
}

/// The parser-owned current authored `grid-template` aggregate.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTemplateValue {
    representation: CssAuthoredGridTemplateRepresentation,
}

impl CssAuthoredGridTemplateValue {
    pub(crate) const fn none() -> Self {
        Self {
            representation: CssAuthoredGridTemplateRepresentation::None,
        }
    }

    pub(crate) const fn rows_columns(
        rows: CssAuthoredGridTrackList,
        columns: Option<CssAuthoredGridTrackList>,
    ) -> Self {
        Self {
            representation: CssAuthoredGridTemplateRepresentation::RowsColumns { rows, columns },
        }
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(
            self.representation,
            CssAuthoredGridTemplateRepresentation::None
        )
    }

    #[must_use]
    pub const fn rows(&self) -> Option<&CssAuthoredGridTrackList> {
        match &self.representation {
            CssAuthoredGridTemplateRepresentation::RowsColumns { rows, .. } => Some(rows),
            CssAuthoredGridTemplateRepresentation::None => None,
        }
    }

    #[must_use]
    pub const fn columns(&self) -> Option<&CssAuthoredGridTrackList> {
        match &self.representation {
            CssAuthoredGridTemplateRepresentation::RowsColumns { columns, .. } => columns.as_ref(),
            CssAuthoredGridTemplateRepresentation::None => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridRepresentation {
    Template(CssAuthoredGridTemplateValue),
    AutoFlow {
        flow: CssGridAutoFlow,
        auto_tracks: Option<CssAuthoredGridTrackSizeList>,
        explicit_tracks: CssAuthoredGridTrackList,
    },
}

/// The parser-owned current authored `grid` aggregate.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridValue {
    representation: CssAuthoredGridRepresentation,
}

impl CssAuthoredGridValue {
    pub(crate) const fn template(value: CssAuthoredGridTemplateValue) -> Self {
        Self {
            representation: CssAuthoredGridRepresentation::Template(value),
        }
    }

    pub(crate) const fn from_auto_flow(
        flow: CssGridAutoFlow,
        auto_tracks: Option<CssAuthoredGridTrackSizeList>,
        explicit_tracks: CssAuthoredGridTrackList,
    ) -> Self {
        Self {
            representation: CssAuthoredGridRepresentation::AutoFlow {
                flow,
                auto_tracks,
                explicit_tracks,
            },
        }
    }

    #[must_use]
    pub const fn template_value(&self) -> Option<&CssAuthoredGridTemplateValue> {
        match &self.representation {
            CssAuthoredGridRepresentation::Template(value) => Some(value),
            CssAuthoredGridRepresentation::AutoFlow { .. } => None,
        }
    }

    #[must_use]
    pub const fn auto_flow(&self) -> Option<CssGridAutoFlow> {
        match self.representation {
            CssAuthoredGridRepresentation::AutoFlow { flow, .. } => Some(flow),
            CssAuthoredGridRepresentation::Template(_) => None,
        }
    }

    #[must_use]
    pub const fn auto_tracks(&self) -> Option<&CssAuthoredGridTrackSizeList> {
        match &self.representation {
            CssAuthoredGridRepresentation::AutoFlow { auto_tracks, .. } => auto_tracks.as_ref(),
            CssAuthoredGridRepresentation::Template(_) => None,
        }
    }

    #[must_use]
    pub const fn explicit_tracks(&self) -> Option<&CssAuthoredGridTrackList> {
        match &self.representation {
            CssAuthoredGridRepresentation::AutoFlow {
                explicit_tracks, ..
            } => Some(explicit_tracks),
            CssAuthoredGridRepresentation::Template(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedGridTrackList {
    current: CssAuthoredGridTrackList,
    i01_subset: Option<CssGridTrackList>,
}

impl CssParsedGridTrackList {
    pub(crate) const fn new(
        current: CssAuthoredGridTrackList,
        i01_subset: Option<CssGridTrackList>,
    ) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredGridTrackList, Option<CssGridTrackList>) {
        (self.current, self.i01_subset)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedGridTrackSizeList {
    current: CssAuthoredGridTrackSizeList,
    i01_subset: Option<CssGridTrackList>,
}

impl CssParsedGridTrackSizeList {
    pub(crate) const fn new(
        current: CssAuthoredGridTrackSizeList,
        i01_subset: Option<CssGridTrackList>,
    ) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredGridTrackSizeList, Option<CssGridTrackList>) {
        (self.current, self.i01_subset)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedGridTemplate {
    current: CssAuthoredGridTemplateValue,
    i01_subset: Option<CssGridTemplate>,
}

impl CssParsedGridTemplate {
    pub(crate) const fn new(
        current: CssAuthoredGridTemplateValue,
        i01_subset: Option<CssGridTemplate>,
    ) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredGridTemplateValue, Option<CssGridTemplate>) {
        (self.current, self.i01_subset)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedGrid {
    current: CssAuthoredGridValue,
    i01_subset: Option<CssGrid>,
}

impl CssParsedGrid {
    pub(crate) const fn new(current: CssAuthoredGridValue, i01_subset: Option<CssGrid>) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredGridValue, Option<CssGrid>) {
        (self.current, self.i01_subset)
    }
}

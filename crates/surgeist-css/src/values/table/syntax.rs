/// The authored `border-collapse` keyword.
///
/// # Specified serialization
///
/// Serializes the specified table border model keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBorderCollapse {
    /// `collapse`: selects the collapsing borders model.
    Collapse,
    /// `separate`: selects the separated borders model, in which each cell has an individual border and `border-spacing` sets the distance between the borders of adjoining cells.
    Separate,
}

/// The authored `caption-side` keyword.
///
/// # Specified serialization
///
/// Serializes the specified table caption side keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCaptionSide {
    /// `top`: positions the caption box above the table box.
    Top,
    /// `bottom`: positions the caption box below the table box.
    Bottom,
}

/// The authored `empty-cells` keyword.
///
/// # Specified serialization
///
/// Serializes the specified empty-cell visibility keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssEmptyCells {
    /// `show`: in the separated borders model, borders and backgrounds are drawn around and behind empty cells like normal cells.
    Show,
    /// `hide`: in the separated borders model, no borders or backgrounds are drawn around or behind empty cells.
    Hide,
}

/// The authored `table-layout` keyword.
///
/// # Specified serialization
///
/// Serializes the specified table layout algorithm keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTableLayout {
    /// `auto`: the user agent uses any automatic table layout algorithm.
    Auto,
    /// `fixed`: the fixed table layout algorithm, in which column widths come from column elements and first-row cells and cells in later rows do not affect them.
    Fixed,
}

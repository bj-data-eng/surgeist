/// The authored `border-collapse` keyword.
///
/// # Specified serialization
///
/// Serializes the specified table border model keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBorderCollapse {
    Collapse,
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
    Top,
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
    Show,
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
    Auto,
    Fixed,
}

/// Authored scrollbar thickness preference, initially `auto` and noninherited.
///
/// CSS Scrollbars 1 applies this keyword to scroll containers; its computed
/// value is the specified keyword and animation is by computed value. Actual
/// scrollbar thickness and root-to-viewport application belong downstream.
///
/// # Specified serialization
///
/// Writes the specified keyword without resolving scrollbar geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollbarWidth {
    Auto,
    Thin,
    None,
}

/// The authored `caret-color` value.
///
/// This value preserves color syntax only; it does not choose a rendered caret
/// color or apply contrast adjustments.
///
/// # Specified serialization
///
/// Emits `auto` or the child color's specified CSS without resolving the caret color.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCaretColor {
    Auto,
    Color(Box<CssColor>),
}

/// The authored `resize` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssResize {
    None,
    Both,
    Horizontal,
    Vertical,
    /// Allows resizing along the block axis, resolved by the downstream writing mode.
    Block,
    /// Allows resizing along the inline axis, resolved by the downstream writing mode.
    Inline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursorKeyword {
    Auto,
    Default,
    None,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    AllScroll,
    ColResize,
    RowResize,
    NResize,
    EResize,
    SResize,
    WResize,
    NeResize,
    NwResize,
    SeResize,
    SwResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ZoomIn,
    ZoomOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPointerEvents {
    Auto,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssUserSelect {
    Auto,
    Text,
    None,
    All,
    Contain,
}
use crate::color::CssColor;

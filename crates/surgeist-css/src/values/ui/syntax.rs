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
    /// `auto`: the user agent shows its default scrollbar thickness.
    Auto,
    /// `thin`: implementations should use thinner scrollbars than `auto`, while keeping them wide enough to be usable.
    Thin,
    /// `none`: implementations must not display any scrollbar; the element's scrollability by other means is unaffected.
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
    /// `auto`: the user agent chooses the caret color, usually `currentcolor`.
    Auto,
    /// An authored `<color>` for the caret.
    Color(Box<CssColor>),
}

/// The authored `resize` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssResize {
    /// `none`: the user agent presents no resizing mechanism and gives the user no direct manipulation mechanism to resize the element.
    None,
    /// `both`: the user agent presents a bidirectional resizing mechanism that adjusts both the height and the width.
    Both,
    /// `horizontal`: the user agent presents a unidirectional horizontal resizing mechanism that adjusts only the width.
    Horizontal,
    /// `vertical`: the user agent presents a unidirectional vertical resizing mechanism that adjusts only the height.
    Vertical,
    /// `block`: the user agent presents a unidirectional block-axis resizing mechanism that adjusts only the block size; the axis is resolved by the downstream writing mode.
    Block,
    /// `inline`: the user agent presents a unidirectional inline-axis resizing mechanism that adjusts only the inline size; the axis is resolved by the downstream writing mode.
    Inline,
}

/// The authored cursor keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursorKeyword {
    /// `auto`: the user agent determines the cursor from the current context; it behaves as `text` over selectable text or editable elements and as `default` otherwise.
    Auto,
    /// `default`: the platform-dependent default cursor, often rendered as an arrow.
    Default,
    /// `none`: no cursor is rendered for the element.
    None,
    /// `context-menu`: a context menu is available for the object under the cursor.
    ContextMenu,
    /// `help`: help is available for the object under the cursor.
    Help,
    /// `pointer`: the cursor is a pointer that indicates a link.
    Pointer,
    /// `progress`: the program is performing some processing, but unlike `wait` the user may still interact with it.
    Progress,
    /// `wait`: the program is busy and the user should wait.
    Wait,
    /// `cell`: a cell or set of cells may be selected.
    Cell,
    /// `crosshair`: a simple crosshair, often used to indicate a two-dimensional bitmap selection mode.
    Crosshair,
    /// `text`: text that may be selected.
    Text,
    /// `vertical-text`: vertical text that may be selected.
    VerticalText,
    /// `alias`: an alias of or shortcut to something is to be created.
    Alias,
    /// `copy`: something is to be copied.
    Copy,
    /// `move`: something is to be moved.
    Move,
    /// `no-drop`: the dragged item cannot be dropped at the current cursor location.
    NoDrop,
    /// `not-allowed`: the requested action will not be carried out.
    NotAllowed,
    /// `grab`: something can be grabbed (dragged to be moved).
    Grab,
    /// `grabbing`: something is being grabbed (dragged to be moved).
    Grabbing,
    /// `all-scroll`: something can be scrolled in any direction.
    AllScroll,
    /// `col-resize`: the item or column can be resized horizontally.
    ColResize,
    /// `row-resize`: the item or row can be resized vertically.
    RowResize,
    /// `n-resize`: some edge is to be moved, here the north edge.
    NResize,
    /// `e-resize`: some edge is to be moved, here the east edge.
    EResize,
    /// `s-resize`: some edge is to be moved, here the south edge.
    SResize,
    /// `w-resize`: some edge is to be moved, here the west edge.
    WResize,
    /// `ne-resize`: some edge is to be moved, with the movement starting from the north-east corner.
    NeResize,
    /// `nw-resize`: some edge is to be moved, with the movement starting from the north-west corner.
    NwResize,
    /// `se-resize`: some edge is to be moved, with the movement starting from the south-east corner.
    SeResize,
    /// `sw-resize`: some edge is to be moved, with the movement starting from the south-west corner.
    SwResize,
    /// `ew-resize`: a bidirectional resize cursor.
    EwResize,
    /// `ns-resize`: a bidirectional resize cursor.
    NsResize,
    /// `nesw-resize`: a bidirectional resize cursor.
    NeswResize,
    /// `nwse-resize`: a bidirectional resize cursor.
    NwseResize,
    /// `zoom-in`: something can be zoomed (magnified) in.
    ZoomIn,
    /// `zoom-out`: something can be zoomed (magnified) out.
    ZoomOut,
}

/// The authored `pointer-events` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPointerEvents {
    /// `auto`: hit-testing occurs normally.
    Auto,
    /// `none`: hit-testing acts as if the boxes generated by the element were not there; a descendant with `pointer-events: auto` still takes part in hit-testing.
    None,
}

/// The authored `user-select` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssUserSelect {
    /// `auto`: the used value is derived from the element and its context.
    Auto,
    /// `text`: the element imposes no constraint on the selection.
    Text,
    /// `none`: the user agent must not allow selections to be started in the element, and a selection started outside it must not end in it; selections across the element exclude it, except descendants whose used value is not `none`.
    None,
    /// `all`: the element's content is selected atomically: a selection containing part of the element must contain the entire element.
    All,
    /// `contain`: a selection started inside the element must not extend outside it, and a selection started outside must not end inside it.
    Contain,
}
use crate::color::CssColor;

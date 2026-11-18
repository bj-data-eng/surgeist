//! Shared semantic whitespace-edge rule, independent of glyph advances.
pub(crate) fn whitespace_edge(
    space: bool,
    ogham: bool,
    mode: crate::style::WhiteSpace,
    native: bool,
) -> crate::supplied_line::InlineWhitespaceEdge {
    if space && mode.collapses_spaces() {
        crate::supplied_line::InlineWhitespaceEdge::DiscardAtBoth
    } else if native && ogham && mode.collapses_spaces() {
        crate::supplied_line::InlineWhitespaceEdge::DiscardAtLineEnd
    } else {
        crate::supplied_line::InlineWhitespaceEdge::Preserve
    }
}

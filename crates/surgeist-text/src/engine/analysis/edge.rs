//! Shared semantic whitespace-edge rule, independent of glyph advances.
pub(crate) fn whitespace_edge(
    space: bool,
    ogham: bool,
    mode: crate::WhiteSpace,
    native: bool,
) -> crate::InlineWhitespaceEdge {
    if space && mode.collapses_spaces() {
        crate::InlineWhitespaceEdge::DiscardAtBoth
    } else if native && ogham && mode.collapses_spaces() {
        crate::InlineWhitespaceEdge::DiscardAtLineEnd
    } else {
        crate::InlineWhitespaceEdge::Preserve
    }
}

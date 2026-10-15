//! HTML877d generic relevant-value clamping preserves independent direction.
use surgeist_html::{
    HtmlInputEmptySelectionSupport, HtmlInputSelection, HtmlInputSelectionDirection,
};

#[test]
fn range_collapse_keeps_opposite_directions_distinct_at_the_cursor() {
    let forward = HtmlInputSelection::selection(1, 2, HtmlInputSelectionDirection::Forward)
        .unwrap()
        .clamp(0, HtmlInputEmptySelectionSupport::Unsupported);
    let backward = HtmlInputSelection::selection(1, 2, HtmlInputSelectionDirection::Backward)
        .unwrap()
        .clamp(0, HtmlInputEmptySelectionSupport::Unsupported);
    assert_eq!(forward.cursor_position(), Some(0));
    assert_eq!(backward.cursor_position(), Some(0));
    assert_ne!(forward, backward);
}

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

#[test]
fn all_four_platform_combinations_keep_empty_and_none_support_independent() {
    use HtmlInputSelectionDirection::{Backward, Forward, None};
    use surgeist_html::HtmlInputEmptySelectionSupport as Empty;
    use surgeist_html::HtmlInputNoneDirectionSupport as NoneSupport;
    // Literal independent source outcomes. No product profile is selected here.
    for (empty, none, initial_direction, keeps_range) in [
        (Empty::Supported, NoneSupport::Supported, None, true),
        (Empty::Supported, NoneSupport::Unsupported, Forward, true),
        (Empty::Unsupported, NoneSupport::Supported, None, false),
        (Empty::Unsupported, NoneSupport::Unsupported, Forward, false),
    ] {
        let initial = HtmlInputSelection::initial(none);
        assert_eq!(initial.cursor_position(), Some(0));
        assert_eq!(initial.selection_range(), Option::None);
        assert_eq!(initial.direction(), initial_direction);
        let collapsed = HtmlInputSelection::selection(3, 5, Backward)
            .unwrap()
            .clamp(2, empty);
        assert_eq!(collapsed.direction(), Backward);
        if keeps_range {
            assert_eq!(collapsed.selection_range(), Some((2, 2, Backward)));
            assert_eq!(collapsed.cursor_position(), Option::None);
        } else {
            assert_eq!(collapsed.selection_range(), Option::None);
            assert_eq!(collapsed.cursor_position(), Some(2));
        }
        let explicit_none = collapsed.set_direction(None, none);
        assert_eq!(explicit_none.direction(), initial_direction);
        assert_eq!(explicit_none.cursor_position(), collapsed.cursor_position());
        assert_eq!(
            explicit_none
                .selection_range()
                .map(|(start, end, _)| (start, end)),
            collapsed
                .selection_range()
                .map(|(start, end, _)| (start, end))
        );
    }
}

#[test]
fn cursor_clamp_retains_actual_direction_until_a_source_direction_write() {
    use HtmlInputSelectionDirection::{Backward, Forward, None};
    use surgeist_html::{
        HtmlInputEmptySelectionSupport as Empty, HtmlInputNoneDirectionSupport as Support,
    };
    let retained = HtmlInputSelection::cursor(9, None);
    let clamped = retained.clamp(3, Empty::Unsupported);
    assert_eq!(clamped.cursor_position(), Some(3));
    assert_eq!(clamped.direction(), None);
    assert_eq!(retained.direction(), None);
    // A different current support fact affects an explicit direction write,
    // not preservation of already-established state (e.g. across adoption).
    assert_eq!(
        clamped
            .set_direction(None, Support::Unsupported)
            .direction(),
        Forward
    );
    assert_eq!(
        clamped
            .set_direction(Backward, Support::Unsupported)
            .direction(),
        Backward
    );
    assert_eq!(
        clamped
            .set_direction(Forward, Support::Supported)
            .direction(),
        Forward
    );
    assert_eq!(
        clamped.set_direction(None, Support::Supported).direction(),
        None
    );
}

#[test]
fn range_validation_and_nonempty_clamping_preserve_bounds_and_direction() {
    use HtmlInputSelectionDirection::Backward;
    use surgeist_html::HtmlInputEmptySelectionSupport as Empty;
    let invalid = HtmlInputSelection::selection(4, 3, Backward).unwrap_err();
    assert_eq!((invalid.start(), invalid.end()), (4, 3));
    for support in [Empty::Supported, Empty::Unsupported] {
        let range = HtmlInputSelection::selection(1, 5, Backward)
            .unwrap()
            .clamp(3, support);
        assert_eq!(range.selection_range(), Some((1, 3, Backward)));
        assert_eq!(range.direction(), Backward);
        assert_eq!(range.cursor_position(), Option::None);
        let cursor = HtmlInputSelection::cursor(1, Backward).clamp(3, support);
        assert_eq!(cursor.cursor_position(), Some(1));
        assert_eq!(cursor.direction(), Backward);
    }
}

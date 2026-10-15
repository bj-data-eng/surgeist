//! Literal setter/selection outcomes from HTML877d41486–41494 and44332–44352.
use surgeist_html::{
    HtmlInputEmptySelectionSupport as EmptySupport, HtmlInputNoneDirectionSupport as NoneSupport,
    HtmlInputSelection as Selection, HtmlInputSelectionDirection as Direction, HtmlInputType,
    HtmlInputValueComparison, HtmlInputValueComparisonCursor, HtmlNewlineNormalizer,
    HtmlTextControlValueSetterFinish as Finish, HtmlTextareaFlags, input_value_setter_finish,
    textarea_value_setter_finish,
};

fn api_value(raw: &[u16]) -> Vec<u16> {
    let mut normalizer = HtmlNewlineNormalizer::new();
    raw.iter()
        .filter_map(|&unit| normalizer.push(unit))
        .collect()
}

fn compare(old: &[u16], new: &[u16]) -> HtmlInputValueComparison {
    let mut cursor = HtmlInputValueComparisonCursor::new(old, new);
    while cursor.required_work().is_some() {
        cursor.advance();
    }
    cursor.result().unwrap()
}

// Execute only the returned effect through the existing real selection values.
// The producer owns no raw storage, host direction support or live transaction.
fn apply_finish(
    finish: Finish,
    post_clamp: Selection,
    actual_api_len: usize,
    none_support: NoneSupport,
) -> Selection {
    match finish {
        Finish::NoAdditionalEffect => post_clamp,
        Finish::SetEndCursorUnselectAndNoneDirection => {
            Selection::cursor(actual_api_len, Direction::None.resolve_none(none_support))
        }
    }
}

#[test]
fn changed_textarea_api_value_unselects_a_surviving_range_and_resets_direction() {
    let comparison = compare(&[97, 98, 99, 100], &[119, 120, 121, 122]);
    let post_clamp = Selection::selection(1, 3, Direction::Backward)
        .unwrap()
        .clamp(4, EmptySupport::Supported);
    assert_eq!(
        post_clamp.selection_range(),
        Some((1, 3, Direction::Backward))
    );
    let finish = textarea_value_setter_finish(comparison);
    assert_eq!(finish, Finish::SetEndCursorUnselectAndNoneDirection);
    let result = apply_finish(finish, post_clamp, 4, NoneSupport::Supported);
    assert_eq!(result.cursor_position(), Some(4));
    assert_eq!(result.selection_range(), None);
    assert_eq!(result.direction(), Direction::None);
    // Input's separate source predicate still retains a surviving selection.
    assert_eq!(
        input_value_setter_finish(HtmlInputType::Text, comparison, post_clamp),
        Finish::NoAdditionalEffect
    );
}

#[test]
fn changed_short_or_empty_api_value_finishes_after_either_actual_clamp_outcome() {
    for (new_api, expected_end) in [(&[120_u16][..], 1), (&[][..], 0)] {
        let comparison = compare(&[97, 98, 99, 100], new_api);
        for empty_support in [EmptySupport::Supported, EmptySupport::Unsupported] {
            let post_clamp = Selection::selection(3, 4, Direction::Backward)
                .unwrap()
                .clamp(new_api.len(), empty_support);
            match empty_support {
                EmptySupport::Supported => assert_eq!(
                    post_clamp.selection_range(),
                    Some((expected_end, expected_end, Direction::Backward))
                ),
                EmptySupport::Unsupported => {
                    assert_eq!(post_clamp.cursor_position(), Some(expected_end));
                    assert_eq!(post_clamp.selection_range(), None);
                }
            }
            let finish = textarea_value_setter_finish(comparison);
            assert_eq!(finish, Finish::SetEndCursorUnselectAndNoneDirection);
            for (none_support, expected_direction) in [
                (NoneSupport::Supported, Direction::None),
                (NoneSupport::Unsupported, Direction::Forward),
            ] {
                let result = apply_finish(finish, post_clamp, new_api.len(), none_support);
                assert_eq!(result.cursor_position(), Some(expected_end));
                assert_eq!(result.selection_range(), None);
                assert_eq!(result.direction(), expected_direction);
            }
        }
    }
}

#[test]
fn different_raw_newlines_with_equal_api_value_keep_selection_while_marking_dirty() {
    for (old_raw, new_raw) in [
        (&[65, 10, 66][..], &[65, 13, 10, 66][..]),
        (&[65, 13, 66][..], &[65, 10, 66][..]),
        (&[65, 13, 10, 66][..], &[65, 13, 66][..]),
    ] {
        assert_ne!(old_raw, new_raw);
        let old_api = api_value(old_raw);
        let mut flags = HtmlTextareaFlags::new(false, true);
        flags.mark_dirty_value();
        let new_api = api_value(new_raw);
        assert_eq!(old_api, [65, 10, 66]);
        assert_eq!(new_api, [65, 10, 66]);
        let post_clamp = Selection::selection(0, 2, Direction::Backward)
            .unwrap()
            .clamp(new_api.len(), EmptySupport::Supported);
        let finish = textarea_value_setter_finish(compare(&old_api, &new_api));
        assert_eq!(finish, Finish::NoAdditionalEffect);
        let result = apply_finish(finish, post_clamp, new_api.len(), NoneSupport::Supported);
        assert_eq!(result.selection_range(), Some((0, 2, Direction::Backward)));
        assert_eq!(result.cursor_position(), None);
        assert_eq!(result.direction(), Direction::Backward);
        assert!(flags.dirty_value());
        assert!(flags.user_validity());
    }
}

#[test]
fn actual_normalized_utf16_comparison_and_end_preserve_surrogates_and_nul() {
    let old_api = api_value(&[0xd83d, 0xde00, 13, 10, 0]);
    let new_api = api_value(&[0xd800, 13, 0]);
    assert_eq!(old_api, [0xd83d, 0xde00, 10, 0]);
    assert_eq!(new_api, [0xd800, 10, 0]);
    let finish = textarea_value_setter_finish(compare(&old_api, &new_api));
    assert_eq!(finish, Finish::SetEndCursorUnselectAndNoneDirection);
    let post_clamp =
        Selection::cursor(8, Direction::Backward).clamp(new_api.len(), EmptySupport::Supported);
    let result = apply_finish(finish, post_clamp, new_api.len(), NoneSupport::Supported);
    assert_eq!(result.cursor_position(), Some(3));
    assert_eq!(result.selection_range(), None);
    assert_eq!(result.direction(), Direction::None);
}

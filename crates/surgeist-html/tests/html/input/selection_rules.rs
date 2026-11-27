//! HTML877d generic relevant-value clamping preserves independent direction.
use surgeist_html::elements::input::{
    HtmlInputEmptySelectionSupport, HtmlInputSelection, HtmlInputSelectionDirection,
};

#[test]
fn positional_and_select_all_applicability_are_distinct_input_rules() {
    use surgeist_html::{
        attributes::HtmlInputType as Type,
        elements::input::{HtmlInputFlags, HtmlInputSelectMethod as Applies},
    };
    for (kind, select, positional) in [
        (Type::Text, Applies::CurrentValue, true),
        (Type::Url, Applies::CurrentValue, true),
        (Type::Password, Applies::CurrentValue, true),
        (Type::Email, Applies::ConditionalControlText, false),
        (Type::Number, Applies::ConditionalControlText, false),
        (Type::Time, Applies::ConditionalControlText, false),
        (Type::Color, Applies::ConditionalControlText, false),
        (Type::File, Applies::ConditionalControlText, false),
        (Type::Range, Applies::Inapplicable, false),
        (Type::Hidden, Applies::Inapplicable, false),
    ] {
        assert_eq!(kind.select_method(), select);
        assert_eq!(kind.is_selectable(), positional);
        let flags = HtmlInputFlags::fresh().type_changed(kind);
        assert!(!flags.dirty_value());
        assert_eq!(
            flags.range_text_started().map(HtmlInputFlags::dirty_value),
            positional.then_some(true),
        );
    }
}

#[test]
fn range_edit_handles_nullable_offsets_and_collapsed_ranges_inside_the_value() {
    use HtmlInputSelectionDirection::{Backward, Forward, None};
    use surgeist_html::elements::input::{
        HtmlInputEmptySelectionSupport as Empty, HtmlInputNoneDirectionSupport as Direction,
        HtmlTextSelectionEdit as Edit,
    };
    let original = HtmlInputSelection::selection(1, 3, Backward).unwrap();
    for (edit, expected) in [
        (Edit::Start(Some(4)), (4, 4, Backward)),
        (Edit::End(Option::None), (0, 0, Backward)),
        (Edit::All, (0, 6, None)),
        (Edit::Direction(Forward), (1, 3, Forward)),
        (
            Edit::Range {
                start: 10,
                end: 5,
                direction: Forward,
            },
            (5, 5, Forward),
        ),
    ] {
        let edited = original.edit(edit, 6, Empty::Supported, Direction::Supported);
        assert_eq!(edited.selection_range(), Some(expected));
    }
    let edited = original.edit(
        Edit::Range {
            start: 3,
            end: 3,
            direction: None,
        },
        6,
        Empty::Unsupported,
        Direction::Unsupported,
    );
    assert_eq!(edited.cursor_position(), Some(3));
    assert_eq!(edited.offsets(), (3, 3));
    assert_eq!(edited.direction(), Forward);
}

#[test]
fn range_text_modes_use_the_original_selection_and_replacement_extent() {
    use surgeist_html::elements::input::{
        HtmlRangeTextMode as Mode, HtmlRangeTextTarget as Target, HtmlTextSelectionEdit as Edit,
    };
    let selection =
        HtmlInputSelection::selection(4, 8, HtmlInputSelectionDirection::Backward).unwrap();
    for (mode, start, end) in [
        (Mode::Select, 3, 4),
        (Mode::Start, 3, 3),
        (Mode::End, 4, 4),
        (Mode::Preserve, 3, 6),
    ] {
        let plan = Target::Offsets {
            start: 3,
            end: 6,
            mode,
        }
        .resolve(selection)
        .unwrap()
        .plan(10, 1)
        .unwrap();
        assert_eq!(plan.replaced(), 3..6);
        assert_eq!(plan.output_len(), 8);
        assert_eq!(
            plan.selection(),
            Edit::Range {
                start,
                end,
                direction: HtmlInputSelectionDirection::None
            },
        );
    }
    let omitted = Target::CurrentSelection
        .resolve(selection)
        .unwrap()
        .plan(10, 2)
        .unwrap();
    let explicit = Target::Offsets {
        start: 4,
        end: 8,
        mode: Mode::default(),
    }
    .resolve(selection)
    .unwrap()
    .plan(10, 2)
    .unwrap();
    assert_eq!(omitted, explicit);
    assert_eq!(omitted.replaced(), 4..8);
    assert_eq!(omitted.output_len(), 8);
    assert_eq!(
        omitted.selection(),
        Edit::Range {
            start: 4,
            end: 6,
            direction: HtmlInputSelectionDirection::None
        },
    );
}

#[test]
fn range_text_rejects_reversed_explicit_offsets_before_clamping() {
    use surgeist_html::elements::input::{
        HtmlRangeTextError as Error, HtmlRangeTextMode as Mode, HtmlRangeTextTarget as Target,
    };
    let selection = HtmlInputSelection::cursor(0, HtmlInputSelectionDirection::None);
    let offsets = |start, end| Target::Offsets {
        start,
        end,
        mode: Mode::Select,
    };
    assert_eq!(offsets(13, 12).resolve(selection), Err(Error::Reversed));
    let clipped = offsets(11, usize::MAX)
        .resolve(selection)
        .unwrap()
        .plan(10, 3)
        .unwrap();
    assert_eq!(clipped.replaced(), 10..10);
    assert_eq!(clipped.output_len(), 13);
    assert_eq!(
        offsets(0, 0)
            .resolve(selection)
            .unwrap()
            .plan(usize::MAX, 1),
        Err(Error::ExtentOverflow),
    );
}

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
    use surgeist_html::elements::input::HtmlInputEmptySelectionSupport as Empty;
    use surgeist_html::elements::input::HtmlInputNoneDirectionSupport as NoneSupport;
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
    use surgeist_html::elements::input::{
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
    use surgeist_html::elements::input::HtmlInputEmptySelectionSupport as Empty;
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

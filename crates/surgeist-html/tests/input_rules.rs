use surgeist_html::*;

fn recognize(raw: Option<&DocumentString>) -> HtmlInputType {
    let mut cursor = HtmlInputTypeCursor::new(raw);
    while cursor.required_work().is_some() {
        cursor.advance();
    }
    cursor.result().unwrap()
}
fn compare(old: &[u16], new: &[u16]) -> HtmlInputValueComparison {
    let mut cursor = HtmlInputValueComparisonCursor::new(old, new);
    while cursor.required_work().is_some() {
        cursor.advance();
    }
    cursor.result().unwrap()
}

#[test]
fn authoritative_type_recognition_preserves_all_states_and_text_defaults() {
    let states = [
        ("hidden", HtmlInputType::Hidden),
        ("text", HtmlInputType::Text),
        ("search", HtmlInputType::Search),
        ("tel", HtmlInputType::Telephone),
        ("url", HtmlInputType::Url),
        ("email", HtmlInputType::Email),
        ("password", HtmlInputType::Password),
        ("date", HtmlInputType::Date),
        ("month", HtmlInputType::Month),
        ("week", HtmlInputType::Week),
        ("time", HtmlInputType::Time),
        ("datetime-local", HtmlInputType::LocalDateTime),
        ("number", HtmlInputType::Number),
        ("range", HtmlInputType::Range),
        ("color", HtmlInputType::Color),
        ("checkbox", HtmlInputType::Checkbox),
        ("radio", HtmlInputType::Radio),
        ("file", HtmlInputType::File),
        ("submit", HtmlInputType::Submit),
        ("image", HtmlInputType::Image),
        ("reset", HtmlInputType::Reset),
        ("button", HtmlInputType::Button),
    ];
    for (keyword, state) in states {
        assert_eq!(
            recognize(Some(&DocumentString::from(keyword.to_ascii_uppercase()))),
            state
        );
        if matches!(
            state,
            HtmlInputType::Hidden
                | HtmlInputType::Text
                | HtmlInputType::Search
                | HtmlInputType::Telephone
                | HtmlInputType::Password
                | HtmlInputType::Checkbox
                | HtmlInputType::Radio
                | HtmlInputType::Email
                | HtmlInputType::Url
                | HtmlInputType::Submit
                | HtmlInputType::Reset
                | HtmlInputType::Button
                | HtmlInputType::Date
                | HtmlInputType::Month
                | HtmlInputType::Week
                | HtmlInputType::Time
                | HtmlInputType::LocalDateTime
                | HtmlInputType::Number
                | HtmlInputType::File
                | HtmlInputType::Image
        ) {
            assert_eq!(HtmlInputKind::try_from(state).unwrap().input_type(), state);
        } else {
            assert_eq!(
                HtmlInputKind::try_from(state).unwrap_err().input_type(),
                state
            );
        }
    }
    assert_eq!(recognize(None), HtmlInputType::Text);
    for invalid in ["", " text", "text ", "bogus", "tеxt", "\0text"] {
        assert_eq!(
            recognize(Some(&DocumentString::from(invalid))),
            HtmlInputType::Text
        );
    }
    assert_eq!(
        recognize(Some(&DocumentString::from_utf16(vec![0xd800]))),
        HtmlInputType::Text
    );
}

#[test]
fn default_button_states_preserve_units_and_have_no_selection_or_display_fallback() {
    let units = [0, 9, 10, 13, 0x85, 0xa0, 0xd800, 0xdc00, 0xdfff, 88];
    for (state, kind) in [
        (HtmlInputType::Submit, HtmlInputKind::Submit),
        (HtmlInputType::Reset, HtmlInputKind::Reset),
        (HtmlInputType::Button, HtmlInputKind::Button),
    ] {
        assert_eq!(HtmlInputKind::try_from(state).unwrap(), kind);
        assert_eq!(kind.input_type(), state);
        assert_eq!(kind.value_mode(), HtmlInputValueMode::Default);
        assert_eq!(kind.value_mode().readback(&[], None, None), Some(&[][..]));
        assert_eq!(kind.sanitization(), HtmlInputSanitization::Preserve);
        assert!(!kind.is_selectable());
        let kept: Vec<_> = units
            .into_iter()
            .filter(|&u| kind.sanitization().retain_unit(u))
            .collect();
        assert_eq!(kept, units);
        let fresh = HtmlInputFlags::fresh().type_changed(kind);
        assert_eq!(
            fresh.value_setter_begin(),
            HtmlInputValueSetterStart::SetContentValue
        );
        assert_eq!(
            fresh.value_attribute_changed(),
            HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
                HtmlInputSanitization::Preserve
            )
        );
        let dirty = HtmlInputFlags::from_retained(kind, true, true, true, true, true);
        assert_eq!(
            dirty.value_attribute_changed(),
            HtmlInputValueAttributeEffect::KeepCurrent
        );
        let mut reset = HtmlInputReset::new(dirty);
        let Some(HtmlInputResetEffect::ClearDirtyAndUserValidity(cleared)) = reset.next_effect()
        else {
            panic!("clear flags first")
        };
        assert!(!cleared.dirty_value() && !cleared.dirty_checkedness() && !cleared.user_validity());
        assert!(cleared.checkedness() && cleared.indeterminate());
        assert_eq!(
            reset.next_effect(),
            Some(HtmlInputResetEffect::ReplaceCurrentFromDefault)
        );
        assert_eq!(
            reset.next_effect(),
            Some(HtmlInputResetEffect::SetCheckedFromPresence)
        );
        assert_eq!(
            reset.next_effect(),
            Some(HtmlInputResetEffect::EmptySelectedFiles)
        );
        assert_eq!(
            reset.next_effect(),
            Some(HtmlInputResetEffect::Sanitize(
                HtmlInputSanitization::Preserve
            ))
        );
        assert_eq!(reset.next_effect(), None);
    }
}

#[test]
fn default_button_type_changes_write_back_only_nonempty_value_and_select_only_on_exit() {
    use HtmlInputTypeEffect::*;
    let effects = |old, new, len| {
        let mut change = HtmlInputTypeChange::new(old, new, len);
        std::iter::from_fn(|| change.next_effect()).collect::<Vec<_>>()
    };
    for kind in [
        HtmlInputKind::Submit,
        HtmlInputKind::Reset,
        HtmlInputKind::Button,
    ] {
        assert_eq!(
            effects(HtmlInputKind::Text, kind, 1),
            [
                SetContentValueFromCurrent,
                SetRendering,
                SignalTypeChange,
                Sanitize(HtmlInputSanitization::Preserve)
            ]
        );
        assert_eq!(
            effects(HtmlInputKind::Text, kind, 0),
            [
                SetRendering,
                SignalTypeChange,
                Sanitize(HtmlInputSanitization::Preserve)
            ]
        );
        assert_eq!(
            effects(kind, HtmlInputKind::Text, 1),
            [
                SetCurrentFromDefaultAndClearDirty,
                SetRendering,
                SignalTypeChange,
                Sanitize(HtmlInputSanitization::StripCrLf),
                SetInitialCursorAndNoneDirection
            ]
        );
        assert!(effects(kind, kind, 1).is_empty());
    }
    assert_eq!(
        effects(HtmlInputKind::Submit, HtmlInputKind::Reset, 1),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::Preserve)
        ]
    );
}

#[test]
fn five_state_sanitization_filters_only_cr_lf_units() {
    let original = [0, 9, 10, 13, 0x85, 0xa0, 0xd800, 0xdc00, 0xdfff, 0x41];
    for kind in [
        HtmlInputKind::Hidden,
        HtmlInputKind::Text,
        HtmlInputKind::Search,
        HtmlInputKind::Telephone,
        HtmlInputKind::Password,
    ] {
        let kept: Vec<_> = original
            .into_iter()
            .filter(|&unit| kind.sanitization().retain_unit(unit))
            .collect();
        if kind == HtmlInputKind::Hidden {
            assert_eq!(kind.value_mode(), HtmlInputValueMode::Default);
            assert_eq!(kept, original);
        } else {
            assert_eq!(kind.value_mode(), HtmlInputValueMode::Value);
            assert_eq!(kept, [0, 9, 0x85, 0xa0, 0xd800, 0xdc00, 0xdfff, 0x41]);
        }
    }
}

#[test]
fn dirty_value_and_checkedness_gate_real_attribute_triggers_independently() {
    let fresh = HtmlInputFlags::fresh();
    assert_eq!(fresh.kind(), HtmlInputKind::Text);
    assert_eq!(
        fresh.value_attribute_changed(),
        HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
            HtmlInputSanitization::StripCrLf
        )
    );
    let HtmlInputValueSetterStart::SetCurrentValue(dirty) = fresh.value_setter_begin() else {
        panic!("Value mode")
    };
    assert!(dirty.dirty_value());
    assert_eq!(
        dirty.value_attribute_changed(),
        HtmlInputValueAttributeEffect::KeepCurrent
    );
    let checked = fresh
        .checked_attribute_changed(HtmlInputCheckedAttributeChange::Added)
        .unwrap();
    assert!(checked.checkedness());
    assert!(
        !checked
            .checked_attribute_changed(HtmlInputCheckedAttributeChange::Removed)
            .unwrap()
            .checkedness()
    );
    let retained =
        HtmlInputFlags::from_retained(HtmlInputKind::Text, false, true, true, true, true);
    assert!(
        retained
            .checked_attribute_changed(HtmlInputCheckedAttributeChange::Removed)
            .is_none()
    );
    assert!(retained.dirty_checkedness() && retained.indeterminate() && retained.user_validity());
    let hidden = fresh.type_changed(HtmlInputKind::Hidden);
    assert_eq!(
        hidden.value_setter_begin(),
        HtmlInputValueSetterStart::SetContentValue
    );
    assert!(!hidden.dirty_value());
    assert_eq!(
        hidden.value_attribute_changed(),
        HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
            HtmlInputSanitization::Preserve
        )
    );
}

#[test]
fn reset_orders_flags_default_checked_files_then_sanitization() {
    let flags =
        HtmlInputFlags::from_retained(HtmlInputKind::Password, true, true, true, true, true);
    let mut reset = HtmlInputReset::new(flags);
    let Some(HtmlInputResetEffect::ClearDirtyAndUserValidity(cleared)) = reset.next_effect() else {
        panic!("first reset step")
    };
    assert!(!cleared.dirty_value() && !cleared.dirty_checkedness() && !cleared.user_validity());
    assert!(cleared.checkedness() && cleared.indeterminate());
    assert_eq!(
        reset.next_effect(),
        Some(HtmlInputResetEffect::ReplaceCurrentFromDefault)
    );
    assert_eq!(
        reset.next_effect(),
        Some(HtmlInputResetEffect::SetCheckedFromPresence)
    );
    assert!(!cleared.reset_checkedness(false).checkedness());
    assert_eq!(
        reset.next_effect(),
        Some(HtmlInputResetEffect::EmptySelectedFiles)
    );
    assert_eq!(
        reset.next_effect(),
        Some(HtmlInputResetEffect::Sanitize(
            HtmlInputSanitization::StripCrLf
        ))
    );
    assert_eq!(reset.next_effect(), None);
    assert_eq!(reset.next_effect(), None);
}

#[test]
fn state_changes_have_source_order_and_same_state_has_no_type_effects() {
    use HtmlInputTypeEffect::*;
    let effects = |old, new, len| {
        let mut change = HtmlInputTypeChange::new(old, new, len);
        std::iter::from_fn(|| change.next_effect()).collect::<Vec<_>>()
    };
    assert_eq!(
        effects(HtmlInputKind::Text, HtmlInputKind::Hidden, 3),
        [
            SetContentValueFromCurrent,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::Preserve)
        ]
    );
    assert_eq!(
        effects(HtmlInputKind::Text, HtmlInputKind::Hidden, 0),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::Preserve)
        ]
    );
    assert_eq!(
        effects(HtmlInputKind::Hidden, HtmlInputKind::Search, 3),
        [
            SetCurrentFromDefaultAndClearDirty,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::StripCrLf),
            SetInitialCursorAndNoneDirection
        ]
    );
    assert_eq!(
        effects(HtmlInputKind::Search, HtmlInputKind::Telephone, 3),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::StripCrLf)
        ]
    );
    assert!(effects(HtmlInputKind::Text, HtmlInputKind::Text, 3).is_empty());
    let dirty = HtmlInputFlags::from_retained(HtmlInputKind::Hidden, true, true, true, true, true);
    let next = dirty
        .type_changed(HtmlInputKind::Text)
        .type_default_installed();
    assert!(!next.dirty_value());
    assert!(
        next.dirty_checkedness()
            && next.checkedness()
            && next.indeterminate()
            && next.user_validity()
    );
}

#[test]
fn clone_copies_only_specified_flags_without_resetting_current_or_default() {
    let original =
        HtmlInputFlags::from_retained(HtmlInputKind::Search, true, true, true, true, true);
    let destination = HtmlInputFlags::fresh().type_changed(HtmlInputKind::Search);
    let clone = destination.clone_flags_from(original);
    assert!(
        clone.dirty_value()
            && clone.dirty_checkedness()
            && clone.checkedness()
            && clone.indeterminate()
    );
    assert!(!clone.user_validity());
    assert_eq!(clone.kind(), HtmlInputKind::Search);
    assert!(original.user_validity());
}

#[test]
fn value_comparison_is_exact_and_has_no_hidden_full_scan() {
    let old = [0x41, 0xd800, 0xdc00, 0xdfff];
    assert!(compare(&old, &old).is_equal());
    assert!(!compare(&old, &[0x41, 0xd800, 0xdc00, 0xdffe]).is_equal());
    assert!(!compare(&[1], &[]).is_equal());
    assert!(compare(&[], &[]).is_equal());
    let mut cursor = HtmlInputValueComparisonCursor::new(&old, &old);
    assert_eq!(
        cursor.required_work(),
        Some(HtmlInputRuleWork::ReadValueLengths)
    );
    cursor.advance();
    for _ in 0..4 {
        assert_eq!(cursor.result(), None);
        assert_eq!(
            cursor.required_work(),
            Some(HtmlInputRuleWork::CompareValueUnits)
        );
        cursor.advance();
    }
    assert!(cursor.result().unwrap().is_equal());
    assert_eq!(cursor.required_work(), None);
}

#[test]
fn native_setter_requires_different_value_and_actual_post_clamp_cursor() {
    use HtmlInputEmptySelectionSupport::{Supported, Unsupported};
    use HtmlInputSelectionDirection::Forward;
    use HtmlInputValueSetterFinish::{NoAdditionalEffect, SetEndCursorUnselectAndNoneDirection};
    let different = compare(&[97, 98, 99, 100], &[119, 120, 121, 122]);
    assert_eq!(
        input_value_setter_finish(
            HtmlInputKind::Hidden,
            different,
            HtmlInputSelection::cursor(1)
        ),
        NoAdditionalEffect
    );

    let selection = HtmlInputSelection::selection(1, 2, Forward)
        .unwrap()
        .clamp(4, Unsupported);
    assert_eq!(selection.selection_range(), Some((1, 2, Forward)));
    assert_eq!(
        input_value_setter_finish(HtmlInputKind::Text, different, selection),
        NoAdditionalEffect
    );
    assert_eq!(
        input_value_setter_finish(
            HtmlInputKind::Text,
            different,
            HtmlInputSelection::cursor(1).clamp(4, Supported)
        ),
        SetEndCursorUnselectAndNoneDirection
    );
    let equal = compare(&[97, 98, 99, 100], &[97, 98, 99, 100]);
    assert_eq!(
        input_value_setter_finish(HtmlInputKind::Text, equal, selection),
        NoAdditionalEffect
    );
    assert_eq!(
        input_value_setter_finish(HtmlInputKind::Text, equal, HtmlInputSelection::cursor(1)),
        NoAdditionalEffect
    );
    let shortened = compare(&[97, 98, 99, 100], &[120]);
    let range = HtmlInputSelection::selection(3, 4, Forward).unwrap();
    let cursor = range.clamp(1, Unsupported);
    assert_eq!(cursor.cursor_position(), Some(1));
    assert_eq!(
        input_value_setter_finish(HtmlInputKind::Text, shortened, cursor),
        SetEndCursorUnselectAndNoneDirection
    );
    let retained = range.clamp(1, Supported);
    assert_eq!(retained.selection_range(), Some((1, 1, Forward)));
    assert_eq!(
        input_value_setter_finish(HtmlInputKind::Text, shortened, retained),
        NoAdditionalEffect
    );
}

#[test]
fn native_sanitized_equal_value_still_sets_dirty_without_replacing_default_or_cursor() {
    let default = [65, 13, 66, 10, 67];
    let old_current = [65, 66, 67];
    let supplied = [65, 13, 66, 10, 67];
    let flags = HtmlInputFlags::fresh();
    let HtmlInputValueSetterStart::SetCurrentValue(dirty) = flags.value_setter_begin() else {
        panic!("Text Value mode")
    };
    let current: Vec<_> = supplied
        .into_iter()
        .filter(|&unit| dirty.kind().sanitization().retain_unit(unit))
        .collect();
    assert_eq!(current, old_current);
    assert!(dirty.dirty_value());
    assert_eq!(default, [65, 13, 66, 10, 67]);
    let equal = compare(&old_current, &current);
    assert!(equal.is_equal());
    assert_eq!(
        input_value_setter_finish(
            HtmlInputKind::Text,
            equal,
            HtmlInputSelection::cursor(1)
                .clamp(current.len(), HtmlInputEmptySelectionSupport::Supported)
        ),
        HtmlInputValueSetterFinish::NoAdditionalEffect
    );
}

#[test]
fn selection_range_validation_clamping_and_direction_use_actual_capabilities() {
    use HtmlInputSelectionDirection::{Backward, Forward, None};
    let invalid = HtmlInputSelection::selection(2, 1, Forward).unwrap_err();
    assert_eq!((invalid.start(), invalid.end()), (2, 1));
    assert_eq!(
        HtmlInputSelection::cursor(9)
            .clamp(3, HtmlInputEmptySelectionSupport::Supported)
            .cursor_position(),
        Some(3)
    );
    assert_eq!(
        HtmlInputSelection::cursor(1)
            .clamp(3, HtmlInputEmptySelectionSupport::Supported)
            .cursor_position(),
        Some(1)
    );
    assert_eq!(
        None.resolve_none(HtmlInputNoneDirectionSupport::Unsupported),
        Forward
    );
    assert_eq!(
        None.resolve_none(HtmlInputNoneDirectionSupport::Supported),
        None
    );
    assert_eq!(
        Backward.resolve_none(HtmlInputNoneDirectionSupport::Unsupported),
        Backward
    );
}

#[test]
fn number_shared_rules_preserve_complete_grammar_without_binary64_conversion() {
    let kind = HtmlInputKind::try_from(HtmlInputType::Number).unwrap();
    assert_eq!(kind.value_mode(), HtmlInputValueMode::Value);
    assert_eq!(kind.sanitization(), HtmlInputSanitization::Number);
    assert!(!kind.is_selectable());
    for (raw, expected) in [
        (".5", ".5"),
        ("-0", "-0"),
        ("1e+2", "1e+2"),
        ("1e999", "1e999"),
        ("001.00", "001.00"),
        ("+1", ""),
        ("1.", ""),
        ("1e", ""),
        ("1e+", ""),
        ("12junk", ""),
        (" 1", ""),
        ("1 ", ""),
        ("1\r\n2", ""),
        ("NaN", ""),
        ("Infinity", ""),
        ("", ""),
    ] {
        let units = raw.encode_utf16().collect::<Vec<_>>();
        let mut cursor = HtmlInputSanitizationCursor::new(
            kind.sanitization(),
            &units,
            HtmlInputEmailMode::Single,
        );
        let mut ranges = Vec::new();
        let mut steps = 0;
        while cursor.required_work().is_some() {
            if let Some(range) = cursor.advance() {
                ranges.push(range);
            }
            steps += 1;
            assert!(steps <= units.len() * 4 + 14, "bounded complete grammar");
        }
        assert_eq!(ranges.len(), 1);
        let range = ranges[0];
        assert_eq!(
            &units[range.start()..range.end()],
            expected.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(cursor.email_value_count(), None);
        assert!((0..units.len()).all(|i| cursor.replacement_unit(i).is_none()));
    }
    let units = [0x31, 0xd800, 0x32];
    let mut cursor =
        HtmlInputSanitizationCursor::new(kind.sanitization(), &units, HtmlInputEmailMode::Multiple);
    let mut range = None;
    while cursor.required_work().is_some() {
        range = cursor.advance().or(range);
    }
    assert_eq!(range.unwrap().end(), 0);
}

#[test]
fn number_sanitizer_inspection_actions_are_explicit_and_source_ordered() {
    let units = [0x31, 0x65, 0x39, 0x39, 0x39];
    let mut cursor = HtmlInputSanitizationCursor::new(
        HtmlInputSanitization::Number,
        &units,
        HtmlInputEmailMode::Single,
    );
    let mut inspections = 0;
    let mut fixed = 0;
    let mut result = None;
    while let Some(work) = cursor.required_work() {
        match work {
            HtmlInputRuleWork::InspectValueUnit => inspections += 1,
            HtmlInputRuleWork::ReadField => fixed += 1,
            _ => panic!("no unrelated rule operation"),
        }
        result = cursor.advance().or(result);
    }
    // Sign; digit loop (digit plus terminator); decimal-point decision;
    // exponent marker/sign decisions; three exponent digits = nine reads.
    // Initial state, EOF exponent decision, finish, and interval emission = four.
    assert_eq!((inspections, fixed), (9, 4));
    assert_eq!((result.unwrap().start(), result.unwrap().end()), (0, 5));
    assert_eq!(cursor.required_work(), None);
    assert_eq!(cursor.advance(), None);
}

#[test]
fn number_reset_type_and_clone_use_value_mode_without_selection_effects() {
    use HtmlInputTypeEffect::*;
    let number = HtmlInputKind::Number;
    let old = HtmlInputFlags::from_retained(number, true, true, true, true, true);
    let reset = old.reset_begin().reset_checkedness(false);
    assert_eq!(reset.kind(), number);
    assert!(!reset.dirty_value() && !reset.dirty_checkedness() && !reset.checkedness());
    let clone = HtmlInputFlags::fresh()
        .type_changed(number)
        .clone_flags_from(old);
    assert!(clone.dirty_value() && clone.dirty_checkedness() && clone.checkedness());
    assert!(!clone.user_validity());
    let mut change = HtmlInputTypeChange::new(HtmlInputKind::Hidden, number, 3);
    let effects = std::iter::from_fn(|| change.next_effect()).collect::<Vec<_>>();
    assert_eq!(
        effects,
        [
            SetCurrentFromDefaultAndClearDirty,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::Number),
        ]
    );
    let mut change = HtmlInputTypeChange::new(number, HtmlInputKind::Text, 3);
    assert_eq!(
        std::iter::from_fn(|| change.next_effect()).collect::<Vec<_>>(),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::StripCrLf),
            SetInitialCursorAndNoneDirection,
        ]
    );
}

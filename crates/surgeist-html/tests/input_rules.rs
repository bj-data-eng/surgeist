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
        ) {
            assert_eq!(
                HtmlInputStringKind::try_from(state).unwrap().input_type(),
                state
            );
        } else {
            assert_eq!(
                HtmlInputStringKind::try_from(state)
                    .unwrap_err()
                    .input_type(),
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
fn five_state_sanitization_filters_only_cr_lf_units() {
    let original = [0, 9, 10, 13, 0x85, 0xa0, 0xd800, 0xdc00, 0xdfff, 0x41];
    for kind in [
        HtmlInputStringKind::Hidden,
        HtmlInputStringKind::Text,
        HtmlInputStringKind::Search,
        HtmlInputStringKind::Telephone,
        HtmlInputStringKind::Password,
    ] {
        let kept: Vec<_> = original
            .into_iter()
            .filter(|&unit| kind.sanitization().retain_unit(unit))
            .collect();
        if kind == HtmlInputStringKind::Hidden {
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
    let fresh = HtmlInputStringFlags::fresh();
    assert_eq!(fresh.kind(), HtmlInputStringKind::Text);
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
    let retained = HtmlInputStringFlags::from_retained(
        HtmlInputStringKind::Text,
        false,
        true,
        true,
        true,
        true,
    );
    assert!(
        retained
            .checked_attribute_changed(HtmlInputCheckedAttributeChange::Removed)
            .is_none()
    );
    assert!(retained.dirty_checkedness() && retained.indeterminate() && retained.user_validity());
    let hidden = fresh.type_changed(HtmlInputStringKind::Hidden);
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
    let flags = HtmlInputStringFlags::from_retained(
        HtmlInputStringKind::Password,
        true,
        true,
        true,
        true,
        true,
    );
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
        effects(HtmlInputStringKind::Text, HtmlInputStringKind::Hidden, 3),
        [
            SetContentValueFromCurrent,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::Preserve)
        ]
    );
    assert_eq!(
        effects(HtmlInputStringKind::Text, HtmlInputStringKind::Hidden, 0),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::Preserve)
        ]
    );
    assert_eq!(
        effects(HtmlInputStringKind::Hidden, HtmlInputStringKind::Search, 3),
        [
            SetCurrentFromDefaultAndClearDirty,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::StripCrLf),
            SetInitialCursorAndNoneDirection
        ]
    );
    assert_eq!(
        effects(
            HtmlInputStringKind::Search,
            HtmlInputStringKind::Telephone,
            3
        ),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputSanitization::StripCrLf)
        ]
    );
    assert!(effects(HtmlInputStringKind::Text, HtmlInputStringKind::Text, 3).is_empty());
    let dirty = HtmlInputStringFlags::from_retained(
        HtmlInputStringKind::Hidden,
        true,
        true,
        true,
        true,
        true,
    );
    let next = dirty
        .type_changed(HtmlInputStringKind::Text)
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
    let original = HtmlInputStringFlags::from_retained(
        HtmlInputStringKind::Search,
        true,
        true,
        true,
        true,
        true,
    );
    let destination = HtmlInputStringFlags::fresh().type_changed(HtmlInputStringKind::Search);
    let clone = destination.clone_flags_from(original);
    assert!(
        clone.dirty_value()
            && clone.dirty_checkedness()
            && clone.checkedness()
            && clone.indeterminate()
    );
    assert!(!clone.user_validity());
    assert_eq!(clone.kind(), HtmlInputStringKind::Search);
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
            HtmlInputStringKind::Hidden,
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
        input_value_setter_finish(HtmlInputStringKind::Text, different, selection),
        NoAdditionalEffect
    );
    assert_eq!(
        input_value_setter_finish(
            HtmlInputStringKind::Text,
            different,
            HtmlInputSelection::cursor(1).clamp(4, Supported)
        ),
        SetEndCursorUnselectAndNoneDirection
    );
    let equal = compare(&[97, 98, 99, 100], &[97, 98, 99, 100]);
    assert_eq!(
        input_value_setter_finish(HtmlInputStringKind::Text, equal, selection),
        NoAdditionalEffect
    );
    assert_eq!(
        input_value_setter_finish(
            HtmlInputStringKind::Text,
            equal,
            HtmlInputSelection::cursor(1)
        ),
        NoAdditionalEffect
    );
    let shortened = compare(&[97, 98, 99, 100], &[120]);
    let range = HtmlInputSelection::selection(3, 4, Forward).unwrap();
    let cursor = range.clamp(1, Unsupported);
    assert_eq!(cursor.cursor_position(), Some(1));
    assert_eq!(
        input_value_setter_finish(HtmlInputStringKind::Text, shortened, cursor),
        SetEndCursorUnselectAndNoneDirection
    );
    let retained = range.clamp(1, Supported);
    assert_eq!(retained.selection_range(), Some((1, 1, Forward)));
    assert_eq!(
        input_value_setter_finish(HtmlInputStringKind::Text, shortened, retained),
        NoAdditionalEffect
    );
}

#[test]
fn native_sanitized_equal_value_still_sets_dirty_without_replacing_default_or_cursor() {
    let default = [65, 13, 66, 10, 67];
    let old_current = [65, 66, 67];
    let supplied = [65, 13, 66, 10, 67];
    let flags = HtmlInputStringFlags::fresh();
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
            HtmlInputStringKind::Text,
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

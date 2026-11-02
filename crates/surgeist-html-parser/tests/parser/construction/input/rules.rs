use surgeist_html::*;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseAssociationRelation, HtmlParseHistoryFrame,
    HtmlParseHistoryStep, HtmlParseLimits, HtmlParseMetadataStep, HtmlParseRangeBranch,
    HtmlParseRangeOperandKind, HtmlParseRangeOperandRole, HtmlSource,
};

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
        assert_eq!(recognize(Some(&DocumentString::from(keyword))), state);
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
        (HtmlInputType::Submit, HtmlInputType::Submit),
        (HtmlInputType::Reset, HtmlInputType::Reset),
        (HtmlInputType::Button, HtmlInputType::Button),
    ] {
        assert_eq!(state, kind);
        assert_eq!(kind, state);
        assert_eq!(kind.value_mode(), HtmlInputValueMode::Default);
        assert_eq!(kind.value_mode().readback(&[], None, None), Some(&[][..]));
        assert_eq!(
            string_policy(kind.sanitization()),
            HtmlInputSanitization::Preserve
        );
        assert!(!kind.is_selectable());
        let kept: Vec<_> = units
            .into_iter()
            .filter(|&u| string_policy(kind.sanitization()).retain_unit(u))
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
                HtmlInputValueSanitization::String(HtmlInputSanitization::Preserve)
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
                HtmlInputValueSanitization::String(HtmlInputSanitization::Preserve)
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
        HtmlInputType::Submit,
        HtmlInputType::Reset,
        HtmlInputType::Button,
    ] {
        assert_eq!(
            effects(HtmlInputType::Text, kind, 1),
            [
                SetContentValueFromCurrent,
                SetRendering,
                SignalTypeChange,
                Sanitize(HtmlInputValueSanitization::String(
                    HtmlInputSanitization::Preserve
                ))
            ]
        );
        assert_eq!(
            effects(HtmlInputType::Text, kind, 0),
            [
                SetRendering,
                SignalTypeChange,
                Sanitize(HtmlInputValueSanitization::String(
                    HtmlInputSanitization::Preserve
                ))
            ]
        );
        assert_eq!(
            effects(kind, HtmlInputType::Text, 1),
            [
                SetCurrentFromDefaultAndClearDirty,
                SetRendering,
                SignalTypeChange,
                Sanitize(HtmlInputValueSanitization::String(
                    HtmlInputSanitization::StripCrLf
                )),
                SetInitialCursorAndNoneDirection
            ]
        );
        assert!(effects(kind, kind, 1).is_empty());
    }
    assert_eq!(
        effects(HtmlInputType::Submit, HtmlInputType::Reset, 1),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
}

#[test]
fn five_state_sanitization_filters_only_cr_lf_units() {
    let original = [0, 9, 10, 13, 0x85, 0xa0, 0xd800, 0xdc00, 0xdfff, 0x41];
    for kind in [
        HtmlInputType::Hidden,
        HtmlInputType::Text,
        HtmlInputType::Search,
        HtmlInputType::Telephone,
        HtmlInputType::Password,
    ] {
        let kept: Vec<_> = original
            .into_iter()
            .filter(|&unit| string_policy(kind.sanitization()).retain_unit(unit))
            .collect();
        if kind == HtmlInputType::Hidden {
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
    assert_eq!(fresh.input_type(), HtmlInputType::Text);
    assert_eq!(
        fresh.value_attribute_changed(),
        HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
            HtmlInputValueSanitization::String(HtmlInputSanitization::StripCrLf)
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
        HtmlInputFlags::from_retained(HtmlInputType::Text, false, true, true, true, true);
    assert!(
        retained
            .checked_attribute_changed(HtmlInputCheckedAttributeChange::Removed)
            .is_none()
    );
    assert!(retained.dirty_checkedness() && retained.indeterminate() && retained.user_validity());
    let hidden = fresh.type_changed(HtmlInputType::Hidden);
    assert_eq!(
        hidden.value_setter_begin(),
        HtmlInputValueSetterStart::SetContentValue
    );
    assert!(!hidden.dirty_value());
    assert_eq!(
        hidden.value_attribute_changed(),
        HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
            HtmlInputValueSanitization::String(HtmlInputSanitization::Preserve)
        )
    );
}

#[test]
fn reset_orders_flags_default_checked_files_then_sanitization() {
    let flags =
        HtmlInputFlags::from_retained(HtmlInputType::Password, true, true, true, true, true);
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
            HtmlInputValueSanitization::String(HtmlInputSanitization::StripCrLf)
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
        effects(HtmlInputType::Text, HtmlInputType::Hidden, 3),
        [
            SetContentValueFromCurrent,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
    assert_eq!(
        effects(HtmlInputType::Text, HtmlInputType::Hidden, 0),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
    assert_eq!(
        effects(HtmlInputType::Hidden, HtmlInputType::Search, 3),
        [
            SetCurrentFromDefaultAndClearDirty,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::StripCrLf
            )),
            SetInitialCursorAndNoneDirection
        ]
    );
    assert_eq!(
        effects(HtmlInputType::Search, HtmlInputType::Telephone, 3),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::StripCrLf
            ))
        ]
    );
    assert!(effects(HtmlInputType::Text, HtmlInputType::Text, 3).is_empty());
    let dirty = HtmlInputFlags::from_retained(HtmlInputType::Hidden, true, true, true, true, true);
    let next = dirty
        .type_changed(HtmlInputType::Text)
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
        HtmlInputFlags::from_retained(HtmlInputType::Search, true, true, true, true, true);
    let destination = HtmlInputFlags::fresh().type_changed(HtmlInputType::Search);
    let clone = destination.clone_flags_from(original);
    assert!(
        clone.dirty_value()
            && clone.dirty_checkedness()
            && clone.checkedness()
            && clone.indeterminate()
    );
    assert!(!clone.user_validity());
    assert_eq!(clone.input_type(), HtmlInputType::Search);
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
    use HtmlTextControlValueSetterFinish::{
        NoAdditionalEffect, SetEndCursorUnselectAndNoneDirection,
    };
    let different = compare(&[97, 98, 99, 100], &[119, 120, 121, 122]);
    assert_eq!(
        input_value_setter_finish(
            HtmlInputType::Hidden,
            different,
            HtmlInputSelection::cursor(1, HtmlInputSelectionDirection::Forward)
        ),
        NoAdditionalEffect
    );

    let selection = HtmlInputSelection::selection(1, 2, Forward)
        .unwrap()
        .clamp(4, Unsupported);
    assert_eq!(selection.selection_range(), Some((1, 2, Forward)));
    assert_eq!(
        input_value_setter_finish(HtmlInputType::Text, different, selection),
        NoAdditionalEffect
    );
    assert_eq!(
        input_value_setter_finish(
            HtmlInputType::Text,
            different,
            HtmlInputSelection::cursor(1, HtmlInputSelectionDirection::Forward).clamp(4, Supported)
        ),
        SetEndCursorUnselectAndNoneDirection
    );
    let equal = compare(&[97, 98, 99, 100], &[97, 98, 99, 100]);
    assert_eq!(
        input_value_setter_finish(HtmlInputType::Text, equal, selection),
        NoAdditionalEffect
    );
    assert_eq!(
        input_value_setter_finish(
            HtmlInputType::Text,
            equal,
            HtmlInputSelection::cursor(1, HtmlInputSelectionDirection::Forward)
        ),
        NoAdditionalEffect
    );
    let shortened = compare(&[97, 98, 99, 100], &[120]);
    let range = HtmlInputSelection::selection(3, 4, Forward).unwrap();
    let cursor = range.clamp(1, Unsupported);
    assert_eq!(cursor.cursor_position(), Some(1));
    assert_eq!(
        input_value_setter_finish(HtmlInputType::Text, shortened, cursor),
        SetEndCursorUnselectAndNoneDirection
    );
    let retained = range.clamp(1, Supported);
    assert_eq!(retained.selection_range(), Some((1, 1, Forward)));
    assert_eq!(
        input_value_setter_finish(HtmlInputType::Text, shortened, retained),
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
        .filter(|&unit| string_policy(dirty.input_type().sanitization()).retain_unit(unit))
        .collect();
    assert_eq!(current, old_current);
    assert!(dirty.dirty_value());
    assert_eq!(default, [65, 13, 66, 10, 67]);
    let equal = compare(&old_current, &current);
    assert!(equal.is_equal());
    assert_eq!(
        input_value_setter_finish(
            HtmlInputType::Text,
            equal,
            HtmlInputSelection::cursor(1, HtmlInputSelectionDirection::Forward)
                .clamp(current.len(), HtmlInputEmptySelectionSupport::Supported)
        ),
        HtmlTextControlValueSetterFinish::NoAdditionalEffect
    );
}

#[test]
fn selection_range_validation_clamping_and_direction_use_actual_capabilities() {
    use HtmlInputSelectionDirection::{Backward, Forward, None};
    let invalid = HtmlInputSelection::selection(2, 1, Forward).unwrap_err();
    assert_eq!((invalid.start(), invalid.end()), (2, 1));
    assert_eq!(
        HtmlInputSelection::cursor(9, HtmlInputSelectionDirection::Forward)
            .clamp(3, HtmlInputEmptySelectionSupport::Supported)
            .cursor_position(),
        Some(3)
    );
    assert_eq!(
        HtmlInputSelection::cursor(1, HtmlInputSelectionDirection::Forward)
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
    let kind = HtmlInputType::Number;
    assert_eq!(kind.value_mode(), HtmlInputValueMode::Value);
    assert_eq!(
        string_policy(kind.sanitization()),
        HtmlInputSanitization::Number
    );
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
            string_policy(kind.sanitization()),
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
    let mut cursor = HtmlInputSanitizationCursor::new(
        string_policy(kind.sanitization()),
        &units,
        HtmlInputEmailMode::Multiple,
    );
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
    let number = HtmlInputType::Number;
    let old = HtmlInputFlags::from_retained(number, true, true, true, true, true);
    let reset = old.reset_begin().reset_checkedness(false);
    assert_eq!(reset.input_type(), number);
    assert!(!reset.dirty_value() && !reset.dirty_checkedness() && !reset.checkedness());
    let clone = HtmlInputFlags::fresh()
        .type_changed(number)
        .clone_flags_from(old);
    assert!(clone.dirty_value() && clone.dirty_checkedness() && clone.checkedness());
    assert!(!clone.user_validity());
    let mut change = HtmlInputTypeChange::new(HtmlInputType::Hidden, number, 3);
    let effects = std::iter::from_fn(|| change.next_effect()).collect::<Vec<_>>();
    assert_eq!(
        effects,
        [
            SetCurrentFromDefaultAndClearDirty,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Number
            )),
        ]
    );
    let mut change = HtmlInputTypeChange::new(number, HtmlInputType::Text, 3);
    assert_eq!(
        std::iter::from_fn(|| change.next_effect()).collect::<Vec<_>>(),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::StripCrLf
            )),
            SetInitialCursorAndNoneDirection,
        ]
    );
}

fn string_policy(policy: HtmlInputValueSanitization) -> HtmlInputSanitization {
    match policy {
        HtmlInputValueSanitization::String(policy) => policy,
        HtmlInputValueSanitization::Range | HtmlInputValueSanitization::Color => {
            panic!("these literal cases require the string operation")
        }
    }
}

#[test]
fn range_requests_constraint_sanitization_without_selection_or_string_cursor_bypass() {
    let kind = HtmlInputType::Range;
    assert_eq!(kind.sanitization(), HtmlInputValueSanitization::Range);
    assert_eq!(kind.value_mode(), HtmlInputValueMode::Value);
    assert!(!kind.is_selectable());
    let flags = HtmlInputFlags::fresh().type_changed(kind);
    assert_eq!(
        flags.value_attribute_changed(),
        HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
            HtmlInputValueSanitization::Range
        ),
    );
}

#[test]
fn range_public_parse_lends_native_default_current_and_qualified_generated_dependencies() {
    let literal =
        "<!doctype html><form id=f><input type=range min=0 max=100 step=20 value=50>Z</form>";
    let limits = HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap();
    let source = HtmlSource::utf8(literal);
    let identity = source.identity().clone();
    let report = HtmlDocumentParseRequest::offline(source, limits)
        .parse()
        .unwrap();
    assert_eq!(report.source().identity(), &identity);
    assert_eq!(report.source().as_utf8(), Some(literal));
    let document = report.document().unwrap();
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let html_children: Vec<_> = document.node(&html).unwrap().children().collect();
    let body = document.node(&html_children[1]).unwrap();
    let form = body.children().next().unwrap();
    let form_children: Vec<_> = document.node(&form).unwrap().children().collect();
    assert_eq!(form_children.len(), 2);
    let input = document.node(&form_children[0]).unwrap();
    assert!(input.children().next().is_none());
    let HtmlNodeKind::Element(element) = input.kind() else {
        panic!("input")
    };
    assert_eq!(element.name().local_name(), &DocumentString::from("input"));
    assert_eq!(
        element
            .attributes()
            .as_slice()
            .iter()
            .find(|a| a.name().local_name() == &DocumentString::from("value"))
            .unwrap()
            .value(),
        &DocumentString::from("50")
    );
    assert!(
        matches!(document.node(&form_children[1]).unwrap().kind(), HtmlNodeKind::Text(text) if text==&DocumentString::from("Z"))
    );
    let mut metadata = report.metadata();
    let mut found_current = false;
    let mut found_owner = false;
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) if record.node().id() == input.id() => {
                if let Some(flags) = record.input_flags() {
                    assert_eq!(flags.input_type(), HtmlInputType::Range);
                    assert!(
                        !flags.dirty_value() && !flags.dirty_checkedness() && !flags.checkedness()
                    );
                    assert_eq!(record.input_raw_value(), Some(&DocumentString::from("60")));
                    found_current = true;
                }
                if let Some(owner) = record.form_owner() {
                    assert_eq!(owner.unwrap().id(), form);
                    found_owner = true;
                }
            }
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            _ => {}
        }
    }
    assert!(found_current && found_owner);
    let history = report.history_view();
    let rows = report.associations();
    let mut selected = None;
    for index in 0..history.len() {
        let group = history.group(index).unwrap();
        let header = history.header(group).unwrap();
        if let Some(computation) = header.cause.range_computation()
            && computation.branch() == HtmlParseRangeBranch::StepCorrection
        {
            selected = Some((group, computation));
        }
    }
    let (group, computation) = selected.expect("50 is equally far from40 and60: source selects60");
    assert!(!computation.original_is_fresh_empty());
    assert!(
        computation.original_group().unwrap().is_some()
            && computation.original_cause().unwrap().is_some()
    );
    for role in [
        HtmlParseRangeOperandRole::Minimum,
        HtmlParseRangeOperandRole::Maximum,
        HtmlParseRangeOperandRole::Step,
    ] {
        let operand = computation.operand(role);
        assert_eq!(operand.kind(), HtmlParseRangeOperandKind::Finite);
        assert!(operand.group().unwrap().is_some() && operand.cause().unwrap().is_some());
        assert!(
            (0..rows.len()).any(|index| rows.row(index).unwrap().unwrap().relation()
                == HtmlParseAssociationRelation::RangeDependency(role))
        );
    }
    assert_eq!(
        computation
            .operand(HtmlParseRangeOperandRole::ValueContent)
            .kind(),
        HtmlParseRangeOperandKind::NotConsulted
    );
    let mut scratch = vec![HtmlParseHistoryFrame::default(); history.len()];
    let mut cursor = history.cursor(group, &mut scratch, None).unwrap();
    assert!(
        matches!(cursor.advance(), HtmlParseHistoryStep::Leaf(atom) if atom.units()==[54,48] && atom.origins().is_empty())
    );
    assert!(matches!(cursor.advance(), HtmlParseHistoryStep::Complete));
}

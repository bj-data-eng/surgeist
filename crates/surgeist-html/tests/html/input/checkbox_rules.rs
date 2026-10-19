use surgeist_html::*;

// HTML877d Checkbox/default-on, common reset/type-change/clone/checked rules.
#[test]
fn checkbox_has_exact_default_on_content_fallback_and_no_sanitizer() {
    let kind = HtmlInputType::Checkbox;
    assert_eq!(kind, HtmlInputType::Checkbox);
    assert_eq!(kind, HtmlInputType::Checkbox);
    assert_eq!(kind.value_mode(), HtmlInputValueMode::DefaultOn);
    assert_eq!(
        kind.value_mode().readback(&[], None, None),
        Some(&[111, 110][..])
    );
    assert_eq!(
        HtmlInputValueMode::Default.readback(&[], None, None),
        Some(&[][..])
    );
    assert_eq!(
        HtmlInputValueMode::Value.readback(&[65], None, None),
        Some(&[65][..])
    );
    let units = [0, 9, 10, 13, 32, 0x85, 0xa0, 0xd800, 0xdc00, 0xdfff];
    assert!(
        units
            .into_iter()
            .all(|unit| string_policy(kind.sanitization()).retain_unit(unit))
    );
    assert_eq!(
        string_policy(kind.sanitization()),
        HtmlInputSanitization::Preserve
    );
    assert_eq!(
        HtmlInputType::Color.sanitization(),
        HtmlInputValueSanitization::Color
    );
}

#[test]
fn default_on_setter_changes_content_and_does_not_dirty_current_value() {
    let flags = HtmlInputFlags::fresh().type_changed(HtmlInputType::Checkbox);
    assert_eq!(
        flags.value_setter_begin(),
        HtmlInputValueSetterStart::SetContentValue
    );
    assert!(!flags.dirty_value());
    assert_eq!(
        flags.value_attribute_changed(),
        HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
            HtmlInputValueSanitization::String(HtmlInputSanitization::Preserve)
        )
    );
    let dirty =
        HtmlInputFlags::from_retained(HtmlInputType::Checkbox, true, false, false, false, false);
    assert_eq!(
        dirty.value_attribute_changed(),
        HtmlInputValueAttributeEffect::KeepCurrent
    );
}

#[test]
fn checked_setter_is_dirty_and_indeterminate_is_independent() {
    let initial = HtmlInputFlags::fresh().type_changed(HtmlInputType::Checkbox);
    let checked = initial
        .checked_attribute_changed(HtmlInputCheckedAttributeChange::Added)
        .unwrap();
    assert!(checked.checkedness());
    assert!(!checked.dirty_checkedness());
    assert!(
        !checked
            .checked_attribute_changed(HtmlInputCheckedAttributeChange::Removed)
            .unwrap()
            .checkedness()
    );
    let set = checked.checked_setter(false).indeterminate_setter(true);
    assert!(!set.checkedness());
    assert!(set.dirty_checkedness() && set.indeterminate());
    assert!(!set.dirty_value() && !set.user_validity());
    assert_eq!(
        set.checked_attribute_changed(HtmlInputCheckedAttributeChange::Added),
        None
    );
    assert_eq!(
        set.indeterminate_setter(false).checkedness(),
        set.checkedness()
    );
    assert_eq!(
        set.indeterminate_setter(false).dirty_checkedness(),
        set.dirty_checkedness()
    );
}

#[test]
fn checkbox_reset_clears_only_specified_flags_and_retains_indeterminate() {
    let initial =
        HtmlInputFlags::from_retained(HtmlInputType::Checkbox, true, true, true, true, true);
    let mut reset = HtmlInputReset::new(initial);
    let Some(HtmlInputResetEffect::ClearDirtyAndUserValidity(next)) = reset.next_effect() else {
        panic!("first flags");
    };
    assert!(!next.dirty_value() && !next.dirty_checkedness() && !next.user_validity());
    assert!(next.checkedness() && next.indeterminate());
    assert_eq!(
        reset.next_effect(),
        Some(HtmlInputResetEffect::ReplaceCurrentFromDefault)
    );
    assert_eq!(
        reset.next_effect(),
        Some(HtmlInputResetEffect::SetCheckedFromPresence)
    );
    assert!(!next.reset_checkedness(false).checkedness());
    assert!(next.reset_checkedness(false).indeterminate());
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

fn effects(old: HtmlInputType, new: HtmlInputType, len: usize) -> Vec<HtmlInputTypeEffect> {
    let mut cursor = HtmlInputTypeChange::new(old, new, len);
    std::iter::from_fn(|| cursor.next_effect()).collect()
}
#[test]
fn checkbox_type_changes_preserve_mode_order_and_selection_applicability() {
    use HtmlInputType::{Checkbox, Hidden, Text};
    use HtmlInputTypeEffect::*;
    assert_eq!(
        effects(Text, Checkbox, 3),
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
        effects(Text, Checkbox, 0),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
    assert_eq!(
        effects(Checkbox, Text, 3),
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
        effects(Hidden, Checkbox, 3),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
    assert_eq!(
        effects(Checkbox, Hidden, 3),
        [
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
    assert!(effects(Checkbox, Checkbox, 3).is_empty());
    let original = HtmlInputFlags::from_retained(Checkbox, true, true, true, true, true);
    let reload = original.type_changed(Text).type_default_installed();
    assert!(!reload.dirty_value());
    assert!(
        reload.dirty_checkedness()
            && reload.checkedness()
            && reload.indeterminate()
            && reload.user_validity()
    );
}

#[test]
fn checkbox_cloning_copies_four_flags_without_user_validity_or_kind_overwrite() {
    let original =
        HtmlInputFlags::from_retained(HtmlInputType::Checkbox, true, true, true, true, true);
    let destination = HtmlInputFlags::fresh().type_changed(HtmlInputType::Checkbox);
    let copy = destination.clone_flags_from(original);
    assert_eq!(copy.input_type(), HtmlInputType::Checkbox);
    assert!(
        copy.dirty_value()
            && copy.dirty_checkedness()
            && copy.checkedness()
            && copy.indeterminate()
    );
    assert!(!copy.user_validity());
    assert!(original.user_validity());
}

#[test]
fn checkbox_value_finish_never_requests_text_cursor_effects() {
    let mut comparison = HtmlInputValueComparisonCursor::new(&[65], &[66]);
    while comparison.required_work().is_some() {
        comparison.advance();
    }
    assert_eq!(
        input_value_setter_finish(
            HtmlInputType::Checkbox,
            comparison.result().unwrap(),
            HtmlInputSelection::cursor(0, HtmlInputSelectionDirection::Forward)
        ),
        HtmlTextControlValueSetterFinish::NoAdditionalEffect
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

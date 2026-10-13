use surgeist_html::*;

#[test]
fn radio_canonical_kind_default_on_raw_policy_and_no_selection_are_source_defined() {
    let kind = HtmlInputStringKind::try_from(HtmlInputType::Radio).unwrap();
    assert_eq!(kind, HtmlInputStringKind::Radio);
    assert_eq!(kind.input_type(), HtmlInputType::Radio);
    assert_eq!(kind.value_mode(), HtmlInputValueMode::DefaultOn);
    assert_eq!(kind.sanitization(), HtmlInputSanitization::Preserve);
    let flags = HtmlInputStringFlags::fresh().type_changed(kind);
    assert!(matches!(
        flags.value_attribute_changed(),
        HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
            HtmlInputSanitization::Preserve
        )
    ));
}
#[test]
fn radio_five_trigger_requirements_depend_on_actual_radio_current_checkedness() {
    for trigger in [
        HtmlRadioTrigger::Checkedness,
        HtmlRadioTrigger::Name,
        HtmlRadioTrigger::FormOwner,
        HtmlRadioTrigger::Type,
        HtmlRadioTrigger::Connected,
    ] {
        for kind in [
            HtmlInputStringKind::Radio,
            HtmlInputStringKind::Checkbox,
            HtmlInputStringKind::Text,
        ] {
            for checked in [false, true] {
                let flags =
                    HtmlInputStringFlags::from_retained(kind, true, true, checked, true, true);
                assert_eq!(
                    flags.radio_normalization_required(trigger),
                    kind == HtmlInputStringKind::Radio && checked
                );
                let peer = flags.radio_peer_unchecked();
                assert!(!peer.checkedness());
                assert_eq!(peer.kind(), kind);
                assert_eq!(peer.dirty_checkedness(), flags.dirty_checkedness());
                assert_eq!(peer.dirty_value(), flags.dirty_value());
                assert_eq!(peer.indeterminate(), flags.indeterminate());
            }
        }
    }
}
#[test]
fn checkbox_to_radio_type_change_has_real_state_signal_and_preserving_sanitization() {
    let mut change =
        HtmlInputTypeChange::new(HtmlInputStringKind::Checkbox, HtmlInputStringKind::Radio, 4);
    let mut effects = Vec::new();
    while let Some(effect) = change.next_effect() {
        effects.push(effect);
    }
    assert_eq!(
        effects,
        vec![
            HtmlInputTypeEffect::SetRendering,
            HtmlInputTypeEffect::SignalTypeChange,
            HtmlInputTypeEffect::Sanitize(HtmlInputSanitization::Preserve)
        ]
    );
    let mut same =
        HtmlInputTypeChange::new(HtmlInputStringKind::Radio, HtmlInputStringKind::Radio, 4);
    assert_eq!(same.next_effect(), None);
}

use crate::html::support::string_sanitization as string_policy;
use surgeist_html::*;

#[test]
fn radio_canonical_kind_default_on_raw_policy_and_no_selection_are_source_defined() {
    let kind = HtmlInputType::Radio;
    assert_eq!(kind, HtmlInputType::Radio);
    assert_eq!(kind, HtmlInputType::Radio);
    assert_eq!(kind.value_mode(), HtmlInputValueMode::DefaultOn);
    assert_eq!(
        string_policy(kind.sanitization()),
        HtmlInputSanitization::Preserve
    );
    let flags = HtmlInputFlags::fresh().type_changed(kind);
    assert!(matches!(
        flags.value_attribute_changed(),
        HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
            HtmlInputValueSanitization::String(HtmlInputSanitization::Preserve)
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
            HtmlInputType::Radio,
            HtmlInputType::Checkbox,
            HtmlInputType::Text,
        ] {
            for checked in [false, true] {
                let flags = HtmlInputFlags::from_retained(kind, true, true, checked, true, true);
                assert_eq!(
                    flags.radio_normalization_required(trigger),
                    kind == HtmlInputType::Radio && checked
                );
                let peer = flags.radio_peer_unchecked();
                assert!(!peer.checkedness());
                assert_eq!(peer.input_type(), kind);
                assert_eq!(peer.dirty_checkedness(), flags.dirty_checkedness());
                assert_eq!(peer.dirty_value(), flags.dirty_value());
                assert_eq!(peer.indeterminate(), flags.indeterminate());
            }
        }
    }
}
#[test]
fn checkbox_to_radio_type_change_has_real_state_signal_and_preserving_sanitization() {
    let mut change = HtmlInputTypeChange::new(HtmlInputType::Checkbox, HtmlInputType::Radio, 4);
    let mut effects = Vec::new();
    while let Some(effect) = change.next_effect() {
        effects.push(effect);
    }
    assert_eq!(
        effects,
        vec![
            HtmlInputTypeEffect::SetRendering,
            HtmlInputTypeEffect::SignalTypeChange,
            HtmlInputTypeEffect::Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
    let mut same = HtmlInputTypeChange::new(HtmlInputType::Radio, HtmlInputType::Radio, 4);
    assert_eq!(same.next_effect(), None);
}

use surgeist_bindings_js::declarations::{
    DocumentReadyState, DocumentVisibilityState, EventInit, EventListenerOptions,
    GetRootNodeOptions, ShadowRootMode, SlotAssignmentMode,
};

#[test]
fn enumeration_membership_uses_exact_code_units() {
    assert_eq!(
        ShadowRootMode::from_code_units(&[111, 112, 101, 110]),
        Some(ShadowRootMode::Open)
    );
    assert_eq!(
        ShadowRootMode::from_code_units(&[99, 108, 111, 115, 101, 100]),
        Some(ShadowRootMode::Closed)
    );
    for invalid in [
        &[][..],
        &[79, 112, 101, 110],
        &[111, 112, 101, 110, 0],
        &[0xd800],
    ] {
        assert_eq!(ShadowRootMode::from_code_units(invalid), None);
    }
    assert_eq!(ShadowRootMode::Open.as_code_units(), [111, 112, 101, 110]);
    assert_eq!(
        SlotAssignmentMode::Named.as_code_units(),
        [110, 97, 109, 101, 100]
    );
    assert_eq!(
        SlotAssignmentMode::from_code_units(&[109, 97, 110, 117, 97, 108]),
        Some(SlotAssignmentMode::Manual)
    );
    assert_eq!(
        DocumentReadyState::from_code_units(&[108, 111, 97, 100, 105, 110, 103]),
        Some(DocumentReadyState::Loading)
    );
    assert_eq!(
        DocumentReadyState::Interactive.as_code_units(),
        [105, 110, 116, 101, 114, 97, 99, 116, 105, 118, 101]
    );
    assert_eq!(
        DocumentReadyState::Complete.as_code_units(),
        [99, 111, 109, 112, 108, 101, 116, 101]
    );
    assert_eq!(
        DocumentVisibilityState::Visible.as_code_units(),
        [118, 105, 115, 105, 98, 108, 101]
    );
    assert_eq!(
        DocumentVisibilityState::from_code_units(&[104, 105, 100, 100, 101, 110]),
        Some(DocumentVisibilityState::Hidden)
    );
}

#[test]
fn dictionary_defaults_are_typed_values_and_do_not_prevent_other_valid_values() {
    let init = EventInit::default();
    assert!(!init.bubbles);
    assert!(!init.cancelable);
    assert!(!init.composed);
    let changed = EventInit {
        bubbles: true,
        composed: true,
        ..init
    };
    assert!(changed.bubbles && changed.composed);
    assert!(!changed.cancelable);
    assert!(!EventListenerOptions::default().capture);
    assert!(EventListenerOptions { capture: true }.capture);
    assert!(!GetRootNodeOptions::default().composed);
    assert!(GetRootNodeOptions { composed: true }.composed);
}

//! CSS Syntax preprocessing and original-unit provenance for String notifications.
#![forbid(unsafe_code)]

use surgeist_css::*;
use surgeist_cssom::*;

fn selected_notification_source() -> CssSourceSnapshot {
    let owner = CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "style-owner".into(),
        revision: 1,
    };
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut creation = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let ticket = creation
        .create_declarations(
            parse_declaration_block_contents("").into_parts().0,
            false,
            false,
            Some(owner.clone()),
        )
        .unwrap();
    let id = store
        .commit(creation)
        .unwrap()
        .block(&ticket)
        .unwrap()
        .clone();

    let mut change = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let prepared = change
        .prepare_style_attribute_change(
            &id,
            &CssomStyleAttributeChange {
                owner,
                local_name: "style".into(),
                namespace: None,
                value: Some("--x:a\r\nb".into()),
                echo: None,
            },
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap()
        .unwrap();
    assert!(prepared.diagnostics().is_empty());
    assert!(prepared.candidates().is_empty());
    change.apply_declaration(prepared, &[]).unwrap();
    let commit = store.commit(change).unwrap();
    assert!(commit.owner_effects().is_empty());

    let snapshot = store.snapshot();
    let CssomBlockData::Properties {
        selected: CssomProjection::Available(selected),
        ..
    } = snapshot.block(&id).unwrap().data()
    else {
        panic!("ordinary selected declarations")
    };
    let [entry] = selected.entries() else {
        panic!("one selected custom declaration")
    };
    assert_eq!(
        entry.property_name(),
        CssPropertyNameRef::Custom(&CssCustomPropertyName::try_from_decoded("--x").unwrap())
    );
    entry.source().parsed_value().unwrap().source().clone()
}

#[test]
fn string_style_notification_parses_the_css_normalized_input() {
    let source = selected_notification_source();
    // CSS Syntax input preprocessing collapses CRLF to a single LF. These byte
    // coordinates describe admitted CSS, independently of the original units.
    assert_eq!(source.as_str(), "--x:a\nb");
}

#[test]
fn string_style_notification_preserves_original_utf16_and_exact_crlf_mapping() {
    let source = selected_notification_source();
    let original = source
        .original_utf16()
        .expect("String convenience retains the exact original UTF16 input");
    assert_eq!(original.as_slice(), [45, 45, 120, 58, 97, 13, 10, 98]);
    assert_eq!(source.original_utf16_range(0..4), Some(0..4));
    assert_eq!(source.original_utf16_range(4..5), Some(4..5));
    assert_eq!(source.original_utf16_range(5..6), Some(5..7));
    assert_eq!(source.original_utf16_range(6..7), Some(7..8));
    assert_eq!(source.original_utf16_range(7..7), Some(8..8));
}

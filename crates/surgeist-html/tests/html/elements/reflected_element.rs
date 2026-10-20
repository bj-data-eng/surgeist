#![forbid(unsafe_code)]

use surgeist_html::{
    DocumentString, HtmlReflectedElementSetter, HtmlReflectedElementSetterEffect,
    KnownHtmlAttribute, Namespace, QualifiedName, reflected_element_attribute_change,
};

// HTML 877d raw 5416–5424: exact local name and null namespace clear the
// corresponding explicit target. Old/new values do not participate.
#[test]
fn matching_attribute_changes_clear_only_the_corresponding_explicit_target() {
    let command = QualifiedName::new(Namespace::none(), None, "commandfor");
    let popover = QualifiedName::new(Namespace::none(), None, "popovertarget");
    assert!(reflected_element_attribute_change(
        KnownHtmlAttribute::Commandfor,
        &command
    ));
    assert!(!reflected_element_attribute_change(
        KnownHtmlAttribute::Popovertarget,
        &command
    ));
    assert!(reflected_element_attribute_change(
        KnownHtmlAttribute::Popovertarget,
        &popover
    ));
    assert!(!reflected_element_attribute_change(
        KnownHtmlAttribute::Commandfor,
        &popover
    ));
}

#[test]
fn redundant_matching_changes_still_request_a_clear_every_time() {
    // Each invocation represents actual change steps, including repeated sets
    // of the same empty value. No old/new value equality can bypass the rule.
    let changed = QualifiedName::new(Namespace::none(), None, "commandfor");
    for _ in 0..3 {
        assert!(reflected_element_attribute_change(
            KnownHtmlAttribute::Commandfor,
            &changed
        ));
    }
}

#[test]
fn null_namespace_matches_regardless_of_preserved_prefix_metadata() {
    for prefix in [None, Some("".into()), Some("raw-prefix".into())] {
        let changed = QualifiedName::new(Namespace::from_uri(""), prefix, "popovertarget");
        assert!(reflected_element_attribute_change(
            KnownHtmlAttribute::Popovertarget,
            &changed
        ));
    }
}

#[test]
fn any_non_null_namespace_keeps_the_explicit_target() {
    for namespace in [Namespace::html(), Namespace::from_uri("urn:arbitrary")] {
        for (reflected, local) in [
            (KnownHtmlAttribute::Commandfor, "commandfor"),
            (KnownHtmlAttribute::Popovertarget, "popovertarget"),
        ] {
            let changed = QualifiedName::new(namespace.clone(), None, local);
            assert!(!reflected_element_attribute_change(reflected, &changed));
        }
    }
}

#[test]
fn local_name_matching_is_exact_without_case_folding_or_utf16_repair() {
    for local in [
        DocumentString::from("Commandfor"),
        DocumentString::from("COMMANDFor"),
        DocumentString::from("commandforx"),
        DocumentString::from("commandfo"),
        DocumentString::from(""),
        DocumentString::from_utf16([
            0x0063, 0x006f, 0x006d, 0x006d, 0x0061, 0x006e, 0x0064, 0x0066, 0x006f, 0xd800,
        ]),
        DocumentString::from_utf16([
            0x0063, 0x006f, 0x006d, 0x006d, 0x0061, 0x006e, 0x0064, 0x0066, 0x006f, 0x0000,
        ]),
    ] {
        let changed = QualifiedName::new(Namespace::none(), None, local);
        assert!(!reflected_element_attribute_change(
            KnownHtmlAttribute::Commandfor,
            &changed
        ));
    }
    let uppercase = QualifiedName::new(Namespace::none(), None, "POPOVERTARGET");
    assert!(!reflected_element_attribute_change(
        KnownHtmlAttribute::Popovertarget,
        &uppercase
    ));
}

// Independent literal setter traces from HTML 877d raw 5404–5412.
#[test]
fn null_setter_clears_the_explicit_target_before_removing_content() {
    let mut setter = HtmlReflectedElementSetter::null();
    assert_eq!(
        setter.next_effect(),
        Some(HtmlReflectedElementSetterEffect::ClearExplicitTarget)
    );
    assert_eq!(
        setter.next_effect(),
        Some(HtmlReflectedElementSetterEffect::RemoveContentAttribute)
    );
    assert_eq!(setter.next_effect(), None);
    assert_eq!(setter.next_effect(), None);
}

#[test]
fn non_null_setter_writes_empty_content_before_installing_supplied_target() {
    let mut setter = HtmlReflectedElementSetter::element();
    assert_eq!(
        setter.next_effect(),
        Some(HtmlReflectedElementSetterEffect::SetEmptyContentAttribute)
    );
    assert_eq!(
        setter.next_effect(),
        Some(HtmlReflectedElementSetterEffect::SetExplicitTargetFromSuppliedElement)
    );
    assert_eq!(setter.next_effect(), None);
    assert_eq!(setter.next_effect(), None);
}

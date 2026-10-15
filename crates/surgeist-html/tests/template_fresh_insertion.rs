#![forbid(unsafe_code)]

use std::convert::Infallible;
use surgeist_html::bounded::native_element_initialization_with;
use surgeist_html::{
    HtmlNativeInitialization, HtmlNativeInitializationRequirement, HtmlTemplateFreshInsertionState,
    KnownHtmlElement, Namespace, QualifiedName,
};

fn initialize(name: &QualifiedName) -> HtmlNativeInitialization {
    native_element_initialization_with(name, |_| Ok::<_, Infallible>(())).unwrap()
}

// HTML877d raw48464–48470: the actual template's insertion target, start marker
// and end marker are initially null. This does not create its contents object.
#[test]
fn exact_html_template_reports_null_initial_target_and_both_markers() {
    let name = QualifiedName::new(Namespace::html(), None, "template");
    let initialization = initialize(&name);
    assert_eq!(
        initialization.requirement(),
        HtmlNativeInitializationRequirement::KnownHtml(KnownHtmlElement::Template)
    );
    assert_eq!(
        initialization.template_insertion(),
        Some(HtmlTemplateFreshInsertionState::NullTargetAndMarkers)
    );
}

#[test]
fn prefix_metadata_does_not_change_fresh_template_insertion_state() {
    for prefix in [Some("h".into()), Some("".into())] {
        let name = QualifiedName::new(Namespace::html(), prefix, "template");
        assert_eq!(
            initialize(&name).template_insertion(),
            Some(HtmlTemplateFreshInsertionState::NullTargetAndMarkers)
        );
    }
}

#[test]
fn foreign_case_variant_and_other_html_names_have_no_template_fresh_token() {
    for (namespace, local) in [
        (Namespace::none(), "template"),
        (Namespace::svg(), "template"),
        (Namespace::mathml(), "template"),
        (Namespace::from_uri("urn:other"), "template"),
        (Namespace::html(), "Template"),
        (Namespace::html(), "TEMPLATE"),
        (Namespace::html(), "templatex"),
        (Namespace::html(), "div"),
        (Namespace::html(), "slot"),
        (Namespace::html(), "form"),
        (Namespace::html(), "script"),
    ] {
        let name = QualifiedName::new(namespace, None, local);
        assert_eq!(initialize(&name).template_insertion(), None);
    }
}

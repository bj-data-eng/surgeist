//! HTML877d: every original Document begins with design mode disabled.
use surgeist_html::{
    HtmlConstructionLimits, HtmlDocument, HtmlDocumentMode, HtmlFragment, Namespace,
    OrderedAttributes, QualifiedName,
};

#[test]
fn original_owned_document_lends_disabled_state_without_granting_non_document_state() {
    let limits = HtmlConstructionLimits::new(16, 16, 16, 4096, 16);
    let mut builder = HtmlDocument::builder(limits, HtmlDocumentMode::NoQuirks).unwrap();
    let root = builder.root();
    builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let document = builder.finish();
    assert_eq!(document.root().document_design_mode_enabled(), Some(false));
    let child = document.root().children().next().unwrap();
    assert_eq!(
        document
            .node(&child)
            .unwrap()
            .document_design_mode_enabled(),
        None
    );
    let independent = HtmlDocument::builder(limits, HtmlDocumentMode::Quirks)
        .unwrap()
        .finish();
    assert_eq!(
        independent.root().document_design_mode_enabled(),
        Some(false)
    );
    assert_ne!(independent.root().id(), document.root().id());
    assert!(independent.node(&document.root().id()).is_err());
    let fragment = HtmlFragment::builder(limits).unwrap().finish();
    assert_eq!(fragment.root().document_design_mode_enabled(), None);
}

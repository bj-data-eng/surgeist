//! Constructor facts survive the actual original controlled owner and its publication.
use super::*;
use surgeist_html::construction::{
    HtmlConstructionLimits, HtmlDocument, HtmlFocusedAreaDesignation, HtmlFragment,
};

#[test]
fn native_constructor_and_non_document_readbacks_have_distinct_meanings() {
    let mut builder = HtmlDocument::builder(
        HtmlConstructionLimits::new(20, 20, 20, 1000, 20),
        HtmlDocumentMode::NoQuirks,
    )
    .unwrap();
    let root = builder.root();
    builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::html(), None, "input"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let document = builder.finish().unwrap();
    assert_eq!(
        document.root().document_focused_area_designation(),
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
    let child = document.root().children().next().unwrap();
    assert_eq!(
        document
            .node(&child)
            .unwrap()
            .document_focused_area_designation(),
        None
    );
    let fragment = HtmlFragment::builder(HtmlConstructionLimits::new(2, 1, 0, 10, 2))
        .unwrap()
        .finish()
        .unwrap();
    assert_eq!(fragment.root().document_focused_area_designation(), None);
}

#[test]
fn controlled_constructor_fact_survives_mode_edit_detachment_and_publication() {
    let owner = storage();
    let (owner, document) = create(
        owner,
        Make::Document {
            mode: HtmlDocumentMode::NoQuirks,
        },
    );
    let id = owner.result_id(&document).unwrap();
    assert_eq!(
        owner
            .node(&document)
            .unwrap()
            .as_node()
            .document_focused_area_designation(),
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
    let (owner, child) = create(owner, element("div"));
    let (owner, _) = change(
        owner,
        Op::Insert {
            node: child,
            parent: document,
            before: None,
        },
    );
    let (owner, _) = change(
        owner,
        Op::SetDocumentMode {
            document,
            mode: HtmlDocumentMode::LimitedQuirks,
        },
    );
    let (owner, _) = change(owner, Op::Detach { node: child });
    let (owner, _) = change(owner, Op::ReserveRecords { additional: 32 });
    let value = publish(owner, document);
    let original = value.node(&id).unwrap();
    assert_eq!(original.id(), id);
    assert_eq!(
        original.kind(),
        HtmlNodeKind::Document(HtmlDocumentMode::LimitedQuirks)
    );
    assert_eq!(original.children().len(), 0);
    assert_eq!(
        original.document_focused_area_designation(),
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
}

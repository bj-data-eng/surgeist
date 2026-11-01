//! Constructor facts survive the actual original controlled owner and recovery.
use super::*;
use surgeist_html::{
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
    let document = builder.finish();
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
        .finish();
    assert_eq!(fragment.root().document_focused_area_designation(), None);
}

#[test]
fn controlled_constructor_fact_survives_alias_refusal_mode_edit_and_original_reseal() {
    let owner = refuse(
        storage(),
        HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Document(HtmlDocumentMode::Quirks),
        },
        HtmlConstructionError::WrongNodeKind,
    );
    let (owner, document) = create(
        owner,
        HtmlConstructionCommand::CreateDocument {
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
        HtmlConstructionCommand::Insert {
            node: child.clone(),
            parent: document.clone(),
            before: None,
        },
    );
    let (value, lease) = publish(owner, document);
    let alias = value.clone();
    let mut value = Some(value);
    let mut lease = Some(lease);
    assert_eq!(
        HtmlConstructionLease::try_recover(&mut value, &mut lease).err(),
        Some(HtmlConstructionError::SharedAllocationAliased)
    );
    assert_eq!(
        alias.node(&id).unwrap().document_focused_area_designation(),
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
    drop(alias);
    let (owner, retired_wrapper) = HtmlConstructionLease::try_recover(&mut value, &mut lease)
        .unwrap()
        .into_parts();
    drop(retired_wrapper);
    let document = owner.node_by_result_id(&id).unwrap().handle();
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::SetDocumentMode {
            document: document.clone(),
            mode: HtmlDocumentMode::LimitedQuirks,
        },
    );
    let (owner, _) = change(owner, HtmlConstructionCommand::Detach { node: child });
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::ReserveRecords { additional: 32 },
    );
    let (value, lease) = publish(owner, document);
    let original = value.node(&id).unwrap();
    assert_eq!(original.id(), id);
    assert_eq!(
        original.kind(),
        &HtmlNodeKind::Document(HtmlDocumentMode::LimitedQuirks)
    );
    assert_eq!(original.children().len(), 0);
    assert_eq!(
        original.document_focused_area_designation(),
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
    drop((value, lease));
}

#[test]
fn aborted_constructor_does_not_change_existing_owner_and_retirement_still_completes() {
    let (owner, document) = create(
        storage(),
        HtmlConstructionCommand::CreateDocument {
            mode: HtmlDocumentMode::Quirks,
        },
    );
    let before = owner.usage();
    let abandoned = ready(
        owner,
        HtmlConstructionCommand::CreateDocument {
            mode: HtmlDocumentMode::NoQuirks,
        },
    );
    let (owner, _, _) = abandoned.abort().into_parts();
    assert_eq!(owner.usage(), before);
    assert_eq!(
        owner
            .node(&document)
            .unwrap()
            .as_node()
            .document_focused_area_designation(),
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
    let mut retirement = owner.retire();
    let mut payloads = 0;
    while let Some(action) = retirement.next_action().unwrap() {
        payloads += usize::from(matches!(
            action.description(),
            HtmlConstructionRetirementDescription::Payload { .. }
        ));
        action.perform();
    }
    assert_eq!(payloads, 1, "only the committed actual Document exists");
}

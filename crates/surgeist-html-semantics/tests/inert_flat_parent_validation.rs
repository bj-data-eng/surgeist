use surgeist_html::{
    HtmlDocumentMode, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    HtmlValueInterpreter, Namespace, OrderedAttributes, QualifiedName,
};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlDocumentFlatTreeRelation, HtmlDocumentModalBlocking,
    HtmlDocumentModalFact, HtmlFlatTreeProducerId, HtmlFlatTreeRevision, HtmlHostFactsRevision,
    HtmlInertFactError, HtmlInertFactLimits, HtmlInertFacts, HtmlInertNodeFact,
    HtmlScriptingStatus, HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput,
    HtmlSemanticFact, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticRequest, HtmlSemanticSession, HtmlSemanticValue,
};

fn builder() -> HtmlCaptureBuilder {
    HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(8, 128, 0, 256, 16, 0, 0, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    )
}
fn document(builder: &mut HtmlCaptureBuilder) -> HtmlCaptureBuilderSubject {
    builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap()
}
fn element(
    builder: &mut HtmlCaptureBuilder,
    document: &HtmlCaptureBuilderSubject,
    name: &str,
) -> HtmlCaptureBuilderSubject {
    builder
        .add_element(
            document,
            QualifiedName::new(Namespace::html(), None, name),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap()
}
fn revision() -> HtmlFlatTreeRevision {
    HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 1, 1)
}

struct Split {
    capture: HtmlCapture,
    host_document: HtmlCaptureSubjectId,
    root_document: HtmlCaptureSubjectId,
    child: HtmlCaptureSubjectId,
    other: HtmlCaptureSubjectId,
}
fn split(child_name: &str) -> Split {
    let mut builder = builder();
    let host_document = document(&mut builder);
    let root_document = document(&mut builder);
    let host = element(&mut builder, &host_document, "div");
    let root = builder
        .add_shadow_root(
            &root_document,
            &host,
            HtmlCapturedShadowProperties::new(
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Open,
                    HtmlSlotAssignment::Named,
                    false,
                    false,
                    false,
                ),
                false,
                false,
                false,
            ),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let child = element(&mut builder, &root_document, child_name);
    let other = element(&mut builder, &root_document, "section");
    builder.establish_children(&host, &[]).unwrap();
    builder.establish_children(&child, &[]).unwrap();
    builder.establish_children(&other, &[]).unwrap();
    builder
        .establish_children(&host_document, std::slice::from_ref(&host))
        .unwrap();
    builder
        .establish_children(&root_document, std::slice::from_ref(&other))
        .unwrap();
    builder
        .establish_children(&root, std::slice::from_ref(&child))
        .unwrap();
    let capture = builder.finish().unwrap();
    Split {
        host_document: capture.subject_id(&host_document).unwrap(),
        root_document: capture.subject_id(&root_document).unwrap(),
        child: capture.subject_id(&child).unwrap(),
        other: capture.subject_id(&other).unwrap(),
        capture,
    }
}

#[test]
fn equal_node_documents_do_not_allow_flat_parent_from_another_connected_document() {
    // Child is connected through the D2 ShadowRoot's D1 host. Other is the
    // ordinary document element of D2. CSS flat trees cannot join those roots.
    let split = split("span");
    assert_eq!(
        split.capture.node(&split.child).unwrap().node_document(),
        split.capture.node(&split.other).unwrap().node_document()
    );
    let result = HtmlInertFacts::new(
        &split.capture,
        split.capture.host_facts_revision().clone(),
        revision(),
        vec![HtmlInertNodeFact::new(
            split.child.clone(),
            Some(true),
            HtmlDocumentFlatTreeRelation::Parent(split.other),
        )],
        vec![],
        HtmlInertFactLimits::new(1, 10000),
    );
    assert!(matches!(result, Err(HtmlInertFactError::DocumentMismatch(id)) if id == split.child));
}

#[test]
fn modal_blocker_still_qualifies_by_its_own_node_document() {
    let split = split("dialog");
    assert_eq!(
        split
            .capture
            .node(&split.child)
            .unwrap()
            .node_document()
            .subject(),
        &split.root_document
    );
    assert!(matches!(
        HtmlInertFacts::new(
            &split.capture,
            split.capture.host_facts_revision().clone(),
            revision(),
            vec![],
            vec![HtmlDocumentModalFact::new(
                split.host_document,
                HtmlDocumentModalBlocking::BlockedBy(split.child.clone()),
            )],
            HtmlInertFactLimits::new(1, 10000),
        ),
        Err(HtmlInertFactError::DocumentMismatch(id)) if id == split.child
    ));
}

#[test]
fn unavailable_ordinary_ancestry_does_not_invent_a_document_mismatch() {
    let mut builder = builder();
    let first_document = document(&mut builder);
    let second_document = document(&mut builder);
    let child = element(&mut builder, &first_document, "span");
    let parent = element(&mut builder, &second_document, "div");
    // Ordinary ancestry is deliberately unavailable for both, unlike the
    // complete contradiction above. Supplied flat coverage also stays partial.
    let capture = builder.finish().unwrap();
    let child = capture.subject_id(&child).unwrap();
    let parent = capture.subject_id(&parent).unwrap();
    let key = revision();
    let facts = HtmlInertFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        key.clone(),
        vec![
            HtmlInertNodeFact::new(
                child.clone(),
                None,
                HtmlDocumentFlatTreeRelation::Parent(parent.clone()),
            ),
            HtmlInertNodeFact::new(
                parent.clone(),
                None,
                HtmlDocumentFlatTreeRelation::Unavailable,
            ),
        ],
        vec![HtmlDocumentModalFact::new(
            capture.subject_id(&first_document).unwrap(),
            HtmlDocumentModalBlocking::Unblocked,
        )],
        HtmlInertFactLimits::new(3, 10000),
    )
    .unwrap();
    assert!(facts.nodes().iter().all(|fact| fact.connected().is_none()));
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_inert(key, Some(facts)),
        0,
    )
    .unwrap();
    let report = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(128),
        HtmlSemanticLimits::new(10000, 1, 64, 16, 10000, 128),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        child,
        HtmlSemanticQuery::Inertness,
    )])
    .unwrap();
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(id))) if id == &parent
    ));
}

//! DOM3071 replacement step 6 can adopt a ShadowRoot without its retained host.
//! A subsequent child belongs to the root's actual document. CSS Shadow §4.1
//! nevertheless places that child under the host in the document flat tree.
use surgeist_html::{
    Attribute, HtmlDocumentMode, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    HtmlValueInterpreter, Namespace, OrderedAttributes, QualifiedName,
};
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlDocumentFlatTreeRelation, HtmlDocumentModalBlocking,
    HtmlDocumentModalFact, HtmlFlatTreeProducerId, HtmlFlatTreeRevision, HtmlHostFactsRevision,
    HtmlInertFactLimits, HtmlInertFacts, HtmlInertNodeFact, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFact,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticRequest,
    HtmlSemanticSession, HtmlSemanticValue,
};

#[test]
fn actual_cross_document_shadow_child_inherits_inert_from_its_flat_host() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(5, 64, 1, 128, 8, 0, 0, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let host_document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let root_document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let host = builder
        .add_element(
            &host_document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "inert"),
                "",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let root = builder
        .add_shadow_root(
            &root_document,
            &host,
            HtmlCapturedShadowProperties::new(
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Closed,
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
    let child = builder
        .add_element(
            &root_document,
            QualifiedName::new(Namespace::html(), None, "span"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_no_shadow_root(&child).unwrap();
    builder.establish_children(&child, &[]).unwrap();
    builder.establish_children(&host, &[]).unwrap();
    builder
        .establish_children(&root, std::slice::from_ref(&child))
        .unwrap();
    builder
        .establish_children(&host_document, std::slice::from_ref(&host))
        .unwrap();
    builder.establish_children(&root_document, &[]).unwrap();
    let capture = builder.finish().unwrap();
    let host_document = capture.subject_id(&host_document).unwrap();
    let root_document = capture.subject_id(&root_document).unwrap();
    let host = capture.subject_id(&host).unwrap();
    let root = capture.subject_id(&root).unwrap();
    let child = capture.subject_id(&child).unwrap();
    assert_eq!(
        capture.node(&host).unwrap().node_document().subject(),
        &host_document
    );
    assert_eq!(
        capture.node(&root).unwrap().node_document().subject(),
        &root_document
    );
    assert_eq!(
        capture.node(&child).unwrap().node_document().subject(),
        &root_document
    );
    let revision = HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 1, 1);
    let facts = HtmlInertFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        vec![
            HtmlInertNodeFact::new(host.clone(), Some(true), HtmlDocumentFlatTreeRelation::Root),
            HtmlInertNodeFact::new(
                child.clone(),
                Some(true),
                HtmlDocumentFlatTreeRelation::Parent(host),
            ),
        ],
        vec![
            HtmlDocumentModalFact::new(
                host_document.clone(),
                HtmlDocumentModalBlocking::Unavailable,
            ),
            HtmlDocumentModalFact::new(root_document.clone(), HtmlDocumentModalBlocking::Unblocked),
        ],
        HtmlInertFactLimits::new(4, 10000),
    )
    .expect("actual shadow flat-parent edge preserves the distinct node documents");
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_inert(revision, Some(facts)),
        0,
    )
    .unwrap();
    let report = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(128),
        HtmlSemanticLimits::new(10000, 3, 64, 16, 10000, 128),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[
        HtmlSemanticRequest::new(child, HtmlSemanticQuery::Inertness),
        HtmlSemanticRequest::new(host_document.clone(), HtmlSemanticQuery::Inertness),
        HtmlSemanticRequest::new(root_document, HtmlSemanticQuery::Inertness),
    ])
    .unwrap();
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    ));
    assert!(
        matches!(report.results()[1].fact(), HtmlSemanticFact::Inertness(HtmlSemanticValue::Missing(HtmlSemanticMissing::ModalBlocking(document))) if document == &host_document)
    );
    assert!(matches!(
        report.results()[2].fact(),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(false))
    ));
}

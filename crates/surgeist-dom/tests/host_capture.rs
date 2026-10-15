use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomError, DomHostChange, DomHostChangeCoverage, DomHostDomain,
    DomHostDomains, DomHostError, DomHostObservations, DomHostPublisher, DomHostRecomputeReason,
    DomLimits, NodeId, Observed,
};

fn fixture() -> (Dom, DocumentId, [NodeId; 2]) {
    let mut dom = Dom::new(DomLimits::new(64, 4096, 128, 0, 0, 16, 65536).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let a = batch.create_element(&document, "A");
    let b = batch.create_element(&document, "B");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&document).unwrap();
    let nodes = [
        commit.created(&a).unwrap().unwrap(),
        commit.created(&b).unwrap().unwrap(),
    ];
    (dom, document, nodes)
}
fn both(focus: Option<NodeId>, hover: Option<NodeId>) -> DomHostObservations {
    DomHostObservations::FocusAndHover {
        focus: Observed::Supported(focus),
        hover: Observed::Supported(hover),
    }
}
fn complete(coverage: &DomHostChangeCoverage) -> Vec<DomHostChange> {
    let DomHostChangeCoverage::Complete(changes) = coverage else {
        panic!("expected complete coverage: {coverage:?}")
    };
    changes.changes().copied().collect()
}

#[test]
fn fixed_variants_preserve_availability_and_reject_different_configuration_atomically() {
    let (dom, document, _) = fixture();
    let snapshot = dom.snapshot();
    let document = snapshot.document(document).unwrap();
    for (domains, observations) in [
        (
            DomHostDomains::Focus,
            DomHostObservations::Focus(Observed::Unsupported),
        ),
        (
            DomHostDomains::Hover,
            DomHostObservations::Hover(Observed::NotApplicable),
        ),
        (DomHostDomains::FocusAndHover, both(None, None)),
    ] {
        let mut publisher = DomHostPublisher::new(domains);
        assert_eq!(
            publisher.changes_since(None),
            Err(DomHostError::Unpublished)
        );
        let first = publisher
            .publish(document, document.stamp(), observations)
            .unwrap();
        assert_eq!(first.capture().observations(), &observations);
        assert_eq!(first.capture().stamp().host_revision().get(), 0);
        assert_eq!(first.capture().stamp().document(), document.stamp());
        assert_eq!(first.capture().validate_for(document), Ok(()));
        let wrong = if domains == DomHostDomains::Focus {
            DomHostObservations::Hover(Observed::Supported(None))
        } else {
            DomHostObservations::Focus(Observed::Supported(None))
        };
        assert_eq!(
            publisher.publish(document, document.stamp(), wrong),
            Err(DomHostError::DomainMismatch {
                expected: domains,
                actual: wrong.domains()
            })
        );
        assert_eq!(publisher.latest(), Some(first.capture()));
    }
}

#[test]
fn exact_selections_have_complete_ordered_deltas_and_noops_preserve_predecessor_history() {
    let (dom, document, [a, b]) = fixture();
    let snapshot = dom.snapshot();
    let document = snapshot.document(document).unwrap();
    assert!(!document.node(a).unwrap().is_connected());
    let mut publisher = DomHostPublisher::new(DomHostDomains::FocusAndHover);
    let first = publisher
        .publish(document, document.stamp(), both(None, None))
        .unwrap();
    let second = publisher
        .publish(document, document.stamp(), both(Some(a), Some(b)))
        .unwrap();
    assert_eq!(second.capture().stamp().host_revision().get(), 1);
    assert_eq!(
        complete(second.changes()),
        [
            DomHostChange::Target {
                domain: DomHostDomain::ElementFocusSelection,
                previous: None,
                current: Some(a)
            },
            DomHostChange::Target {
                domain: DomHostDomain::ElementHoverSelection,
                previous: None,
                current: Some(b)
            },
        ]
    );
    let noop = publisher
        .publish(document, document.stamp(), both(Some(a), Some(b)))
        .unwrap();
    assert!(complete(noop.changes()).is_empty());
    assert_eq!(noop.capture(), second.capture());
    assert_eq!(
        publisher
            .changes_since(Some(first.capture().stamp()))
            .unwrap(),
        *second.changes()
    );
    assert!(
        complete(
            &publisher
                .changes_since(Some(second.capture().stamp()))
                .unwrap()
        )
        .is_empty()
    );
    publisher
        .publish(document, document.stamp(), both(Some(b), None))
        .unwrap();
    assert!(matches!(
        publisher
            .changes_since(Some(first.capture().stamp()))
            .unwrap(),
        DomHostChangeCoverage::FullRecomputeRequired {
            domains: DomHostDomains::FocusAndHover,
            reason: DomHostRecomputeReason::HistoryUnavailable,
            ..
        }
    ));
    assert_eq!(first.capture().observations(), &both(None, None));
}

#[test]
fn availability_transitions_invalidate_the_whole_domain_even_without_any_node() {
    let (dom, document, _) = fixture();
    let snapshot = dom.snapshot();
    let document = snapshot.document(document).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::Focus);
    for (index, value) in [
        Observed::Unsupported,
        Observed::NotApplicable,
        Observed::Supported(None),
    ]
    .into_iter()
    .enumerate()
    {
        let publication = publisher
            .publish(
                document,
                document.stamp(),
                DomHostObservations::Focus(value),
            )
            .unwrap();
        assert_eq!(
            publication.capture().stamp().host_revision().get(),
            index as u64
        );
        if index != 0 {
            assert_eq!(
                complete(publication.changes()),
                [DomHostChange::Availability {
                    domain: DomHostDomain::ElementFocusSelection
                }]
            );
        }
    }
}

#[test]
fn canonical_revision_changes_require_recomputation_without_advancing_equal_host_facts() {
    let (mut dom, id, [a, _]) = fixture();
    let old = dom.snapshot();
    let old_document = old.document(id).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::Focus);
    let facts = DomHostObservations::Focus(Observed::Supported(Some(a)));
    let first = publisher
        .publish(old_document, old_document.stamp(), facts)
        .unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "key", "value");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let new = dom.snapshot();
    let document = new.document(id).unwrap();
    assert_eq!(
        publisher.publish(document, old_document.stamp(), facts),
        Err(DomHostError::DomRevisionMismatch {
            expected: old.revision(),
            actual: new.revision()
        })
    );
    assert_eq!(publisher.latest(), Some(first.capture()));
    let second = publisher
        .publish(document, document.stamp(), facts)
        .unwrap();
    assert_eq!(second.capture().stamp().host_revision().get(), 0);
    assert_ne!(first.capture().stamp(), second.capture().stamp());
    assert!(matches!(
        second.changes(),
        DomHostChangeCoverage::FullRecomputeRequired {
            reason: DomHostRecomputeReason::DomBasisChanged,
            ..
        }
    ));
    assert_eq!(
        publisher.publish(old_document, old_document.stamp(), facts),
        Err(DomHostError::StaleDomRevision {
            current: new.revision(),
            actual: old.revision()
        })
    );
    assert_eq!(
        first.capture().validate_for(document),
        Err(DomHostError::DomRevisionMismatch {
            expected: old.revision(),
            actual: new.revision()
        })
    );
    assert_eq!(first.capture().validate_for(old_document), Ok(()));
}

#[test]
fn subjects_preserve_node_errors_and_focus_is_validated_before_hover() {
    let (mut dom, id, [a, _]) = fixture();
    let (foreign, foreign_id, [foreign_a, _]) = fixture();
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let other_a = batch.create_element(&other, "Other");
    let text = batch.create_text(id, "text");
    let comment = batch.create_comment(id, "comment");
    let fragment = batch.create_fragment(id);
    let pi = batch.create_processing_instruction(id, "pi", "");
    let doctype = batch.create_document_type(id, "doc", "", "");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let other = commit.document(&other).unwrap();
    let other_a = commit.created(&other_a).unwrap().unwrap();
    let mut non_elements = vec![id.node()];
    non_elements.extend(
        [text, comment, fragment, pi, doctype]
            .iter()
            .map(|ticket| commit.created(ticket).unwrap().unwrap()),
    );
    let mut batch = DomBatch::new();
    batch.destroy(a);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::FocusAndHover);
    let first = publisher
        .publish(document, document.stamp(), both(None, None))
        .unwrap();
    for (node, error) in non_elements
        .into_iter()
        .map(|node| (node, DomError::WrongNodeKind))
        .chain([
            (foreign_a, DomError::ForeignOwner),
            (other_a, DomError::WrongDocument),
            (a, DomError::StaleNode),
        ])
    {
        assert_eq!(
            publisher.publish(
                document,
                document.stamp(),
                both(Some(node), Some(foreign_a))
            ),
            Err(DomHostError::Subject { node, error })
        );
        assert_eq!(publisher.latest(), Some(first.capture()));
    }
    let foreign_snapshot = foreign.snapshot();
    let foreign_document = foreign_snapshot.document(foreign_id).unwrap();
    assert_eq!(
        publisher.publish(foreign_document, foreign_document.stamp(), both(None, None)),
        Err(DomHostError::ForeignOwner)
    );
    let other_document = snapshot.document(other).unwrap();
    assert_eq!(
        publisher.publish(other_document, other_document.stamp(), both(None, None)),
        Err(DomHostError::WrongDocument)
    );
}

#[test]
fn source_provenance_and_old_capture_retention_are_independent_of_publisher_lifetime() {
    let (dom, id, [a, _]) = fixture();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let mut first = DomHostPublisher::new(DomHostDomains::Focus);
    let old = first
        .publish(
            document,
            document.stamp(),
            DomHostObservations::Focus(Observed::Supported(Some(a))),
        )
        .unwrap();
    let mut second = DomHostPublisher::new(DomHostDomains::Focus);
    let current = second
        .publish(
            document,
            document.stamp(),
            DomHostObservations::Focus(Observed::Supported(Some(a))),
        )
        .unwrap();
    assert_ne!(
        old.capture().stamp().source(),
        current.capture().stamp().source()
    );
    assert_eq!(
        second.changes_since(Some(old.capture().stamp())),
        Err(DomHostError::ForeignSource)
    );
    first
        .publish(
            document,
            document.stamp(),
            DomHostObservations::Focus(Observed::Unsupported),
        )
        .unwrap();
    drop(first);
    assert_eq!(
        old.capture().observations(),
        &DomHostObservations::Focus(Observed::Supported(Some(a)))
    );
    assert_eq!(old.capture().validate_for(document), Ok(()));
}

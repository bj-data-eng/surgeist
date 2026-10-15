#![forbid(unsafe_code)]

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomError, DomHostChange, DomHostChangeCoverage, DomHostDomain,
    DomHostDomains, DomHostError, DomHostObservations, DomHostPublisher, DomHostRecomputeReason,
    DomLimits, NodeId, Observed,
};

fn fixture() -> (Dom, DocumentId, [NodeId; 2]) {
    let mut dom = Dom::new(DomLimits::new(64, 4096, 128, 16, 65536).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    // Neither spelling nor an id attribute determines the supplied selection.
    let a = batch.create_element(&document, "A");
    let b = batch.create_element(&document, "B");
    batch.set_attribute(&a, "id", "target");
    batch.set_attribute(&b, "id", "target");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        [
            commit.created(&a).unwrap().unwrap(),
            commit.created(&b).unwrap().unwrap(),
        ],
    )
}

fn target(value: Observed<Option<NodeId>>) -> DomHostObservations {
    DomHostObservations::DocumentTarget(value)
}

fn complete(coverage: &DomHostChangeCoverage) -> Vec<DomHostChange> {
    let DomHostChangeCoverage::Complete(changes) = coverage else {
        panic!("expected complete coverage, got {coverage:?}");
    };
    changes.changes().copied().collect()
}

#[test]
fn supplied_target_absence_and_changes_preserve_exact_captures_and_adjacent_history() {
    let (dom, id, [a, b]) = fixture();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::DocumentTarget);
    assert_eq!(publisher.domains(), DomHostDomains::DocumentTarget);
    assert!(publisher.latest().is_none());
    assert_eq!(
        publisher.changes_since(None),
        Err(DomHostError::Unpublished)
    );
    let first = publisher
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(Some(a))),
        )
        .unwrap();
    assert_eq!(first.capture().stamp().document(), document.stamp());
    assert_eq!(first.capture().stamp().host_revision().get(), 0);
    assert!(matches!(
        first.changes(),
        DomHostChangeCoverage::FullRecomputeRequired {
            from: None,
            domains: DomHostDomains::DocumentTarget,
            reason: DomHostRecomputeReason::InitialCapture,
            ..
        }
    ));

    let absent = publisher
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(None)),
        )
        .unwrap();
    assert_eq!(absent.capture().stamp().host_revision().get(), 1);
    assert_eq!(
        complete(absent.changes()),
        [DomHostChange::Target {
            domain: DomHostDomain::DocumentTargetElementSelection,
            previous: Some(a),
            current: None,
        }]
    );
    let selected = publisher
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(Some(b))),
        )
        .unwrap();
    assert_eq!(selected.capture().stamp().host_revision().get(), 2);
    assert_eq!(
        complete(selected.changes()),
        [DomHostChange::Target {
            domain: DomHostDomain::DocumentTargetElementSelection,
            previous: None,
            current: Some(b),
        }]
    );
    assert_eq!(
        selected.capture().stamp().source(),
        first.capture().stamp().source()
    );
    assert_eq!(selected.capture().stamp().document(), document.stamp());
    let noop = publisher
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(Some(b))),
        )
        .unwrap();
    assert_eq!(noop.capture(), selected.capture());
    assert!(complete(noop.changes()).is_empty());
    assert_eq!(
        publisher
            .changes_since(Some(absent.capture().stamp()))
            .unwrap(),
        *selected.changes()
    );
    assert!(matches!(
        publisher
            .changes_since(Some(first.capture().stamp()))
            .unwrap(),
        DomHostChangeCoverage::FullRecomputeRequired {
            domains: DomHostDomains::DocumentTarget,
            reason: DomHostRecomputeReason::HistoryUnavailable,
            ..
        }
    ));
    drop(publisher);
    assert_eq!(
        first.capture().observations(),
        &target(Observed::Supported(Some(a)))
    );
    assert_eq!(
        absent.capture().observations(),
        &target(Observed::Supported(None))
    );
    assert_eq!(
        selected.capture().observations(),
        &target(Observed::Supported(Some(b)))
    );
    assert_eq!(first.capture().validate_for(document), Ok(()));
}

#[test]
fn supported_absence_unsupported_and_inapplicable_have_distinct_domain_coverage() {
    let (dom, id, _) = fixture();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::DocumentTarget);
    let known_none = publisher
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(None)),
        )
        .unwrap();
    for (revision, observation) in [
        (1, Observed::Unsupported),
        (2, Observed::NotApplicable),
        (3, Observed::Supported(None)),
    ] {
        let publication = publisher
            .publish(document, document.stamp(), target(observation))
            .unwrap();
        assert_eq!(publication.capture().observations(), &target(observation));
        assert_eq!(
            publication.capture().stamp().host_revision().get(),
            revision
        );
        assert_eq!(
            complete(publication.changes()),
            [DomHostChange::Availability {
                domain: DomHostDomain::DocumentTargetElementSelection,
            }]
        );
    }
    assert_eq!(
        known_none.capture().observations(),
        &target(Observed::Supported(None))
    );
    assert_ne!(
        publisher.latest().unwrap().stamp(),
        known_none.capture().stamp()
    );
}

#[test]
fn mismatching_domains_do_not_publish_an_absent_target_or_replace_existing_history() {
    let (dom, id, [a, _]) = fixture();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::DocumentTarget);
    let first = publisher
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(Some(a))),
        )
        .unwrap();
    let before = publisher.changes_since(None).unwrap();
    for observations in [
        DomHostObservations::Focus(Observed::Supported(None)),
        DomHostObservations::Hover(Observed::Unsupported),
        DomHostObservations::FocusAndHover {
            focus: Observed::NotApplicable,
            hover: Observed::Supported(None),
        },
    ] {
        assert_eq!(
            publisher.publish(document, document.stamp(), observations),
            Err(DomHostError::DomainMismatch {
                expected: DomHostDomains::DocumentTarget,
                actual: observations.domains(),
            })
        );
        assert_eq!(publisher.latest(), Some(first.capture()));
        assert_eq!(publisher.changes_since(None).unwrap(), before);
    }
    let mut focus = DomHostPublisher::new(DomHostDomains::Focus);
    assert_eq!(
        focus.publish(document, document.stamp(), target(Observed::Unsupported)),
        Err(DomHostError::DomainMismatch {
            expected: DomHostDomains::Focus,
            actual: DomHostDomains::DocumentTarget,
        })
    );
    assert_eq!(focus.latest(), None);
    assert_eq!(focus.changes_since(None), Err(DomHostError::Unpublished));
}

#[test]
fn subject_rejections_preserve_target_publisher_and_do_not_resurrect_destroyed_ids() {
    let (mut dom, id, [retired, b]) = fixture();
    let (foreign, foreign_id, [foreign_a, _]) = fixture();
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let other_a = batch.create_element(&other, "other");
    let text = batch.create_text(id, "target");
    batch.destroy(retired);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let other = commit.document(&other).unwrap();
    let other_a = commit.created(&other_a).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let mut batch = DomBatch::new();
    let replacement = batch.create_element(id, "replacement");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let replacement = commit.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, retired);
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::DocumentTarget);
    let first = publisher
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(Some(b))),
        )
        .unwrap();
    let before = publisher.changes_since(None).unwrap();
    for (node, error) in [
        (text, DomError::WrongNodeKind),
        (id.node(), DomError::WrongNodeKind),
        (other_a, DomError::WrongDocument),
        (foreign_a, DomError::ForeignOwner),
        (retired, DomError::StaleNode),
    ] {
        assert_eq!(
            publisher.publish(
                document,
                document.stamp(),
                target(Observed::Supported(Some(node)))
            ),
            Err(DomHostError::Subject { node, error })
        );
        assert_eq!(publisher.latest(), Some(first.capture()));
        assert_eq!(publisher.changes_since(None).unwrap(), before);
    }
    let other_document = snapshot.document(other).unwrap();
    assert_eq!(
        publisher.publish(
            other_document,
            other_document.stamp(),
            target(Observed::Supported(None))
        ),
        Err(DomHostError::WrongDocument)
    );
    let foreign_snapshot = foreign.snapshot();
    let foreign_document = foreign_snapshot.document(foreign_id).unwrap();
    assert_eq!(
        publisher.publish(
            foreign_document,
            foreign_document.stamp(),
            target(Observed::Supported(None))
        ),
        Err(DomHostError::ForeignOwner)
    );
    assert_eq!(publisher.latest(), Some(first.capture()));
    assert_eq!(publisher.changes_since(None).unwrap(), before);
    let current = publisher
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(Some(replacement))),
        )
        .unwrap();
    assert_eq!(
        current.capture().observations(),
        &target(Observed::Supported(Some(replacement)))
    );
    assert_eq!(
        first.capture().observations(),
        &target(Observed::Supported(Some(b)))
    );
    assert!(foreign_snapshot.node(foreign_a).is_ok());
}

#[test]
fn exact_dom_basis_rebase_and_source_identity_keep_host_revision_and_retained_values_separate() {
    let (mut dom, id, [a, b]) = fixture();
    let old = dom.snapshot();
    let old_document = old.document(id).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::DocumentTarget);
    let first = publisher
        .publish(
            old_document,
            old_document.stamp(),
            target(Observed::Supported(Some(a))),
        )
        .unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(b, "id", "changed");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    assert_eq!(
        publisher.publish(
            document,
            old_document.stamp(),
            target(Observed::Supported(Some(a)))
        ),
        Err(DomHostError::DomRevisionMismatch {
            expected: old.revision(),
            actual: snapshot.revision(),
        })
    );
    assert_eq!(publisher.latest(), Some(first.capture()));
    let rebased = publisher
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(Some(a))),
        )
        .unwrap();
    assert_eq!(
        rebased.capture().stamp().host_revision(),
        first.capture().stamp().host_revision()
    );
    assert!(matches!(
        rebased.changes(),
        DomHostChangeCoverage::FullRecomputeRequired {
            domains: DomHostDomains::DocumentTarget,
            reason: DomHostRecomputeReason::DomBasisChanged,
            ..
        }
    ));
    assert_eq!(
        publisher.publish(
            old_document,
            old_document.stamp(),
            target(Observed::Supported(None))
        ),
        Err(DomHostError::StaleDomRevision {
            current: snapshot.revision(),
            actual: old.revision(),
        })
    );
    assert_eq!(publisher.latest(), Some(rebased.capture()));
    assert_eq!(first.capture().validate_for(old_document), Ok(()));
    assert!(matches!(
        first.capture().validate_for(document),
        Err(DomHostError::DomRevisionMismatch { .. })
    ));
    let mut independent = DomHostPublisher::new(DomHostDomains::DocumentTarget);
    let other = independent
        .publish(
            document,
            document.stamp(),
            target(Observed::Supported(Some(a))),
        )
        .unwrap();
    assert_ne!(
        other.capture().stamp().source(),
        rebased.capture().stamp().source()
    );
    assert_eq!(
        publisher.changes_since(Some(other.capture().stamp())),
        Err(DomHostError::ForeignSource)
    );
    assert_eq!(
        first.capture().observations(),
        &target(Observed::Supported(Some(a)))
    );
}

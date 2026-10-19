use super::*;

fn configured(scopes: usize, references: usize) -> DomLimits {
    DomLimits::new(
        32,
        4096,
        64,
        0,
        0,
        0,
        0,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_dispatch_custody(
        DomDispatchCustodyLimits::new(scopes, references, 65536, 65536, 10000).unwrap(),
    )
}
fn fixture(limits: DomLimits) -> (Dom, DocumentId, DocumentId, NodeId) {
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    let target = batch.create_comment(&first, "target");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let first = commit.document(&first).unwrap();
    let second = commit.document(&second).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    (dom, first, second, target)
}

#[test]
fn custody_is_send_and_scope_footprint_matches_producer_maximum() {
    fn send<T: Send>() {}
    send::<Dom>();
    send::<DomDispatchCustody>();
    let (mut dom, _, _, target) = fixture(configured(2, 2));
    let scope = dom
        .register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
    assert_eq!(
        scope.retained_bytes(),
        DomDispatchCustody::maximum_retained_bytes(1).unwrap()
    );
}

#[test]
fn wrong_owner_row_and_target_refuse_without_retirement_inference() {
    let (mut dom, document, _, target) = fixture(configured(2, 2));
    let (foreign, _, _, foreign_target) = fixture(configured(2, 2));
    let scope = dom
        .register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
    assert_eq!(
        dom.dispatch_custody_association(&scope, 0, target),
        Ok(DomDispatchAssociation::Current(document))
    );
    assert_eq!(
        dom.dispatch_custody_association(&scope, 1, target),
        Err(DomError::InvalidDispatchCustody)
    );
    assert_eq!(
        dom.dispatch_custody_association(&scope, 0, foreign_target),
        Err(DomError::InvalidDispatchCustody)
    );
    assert_eq!(
        foreign.dispatch_custody_association(&scope, 0, target),
        Err(DomError::ForeignOwner)
    );
    assert_eq!(
        dom.register_dispatch_custody(&[foreign_target], &AtomicBool::new(false))
            .unwrap_err(),
        DomError::ForeignOwner
    );
}

#[test]
fn aggregate_admission_reclaims_abandoned_scope_before_reusing_capacity() {
    let (mut dom, _, _, target) = fixture(configured(1, 1));
    let scope = dom
        .register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
    assert!(matches!(
        dom.register_dispatch_custody(&[target], &AtomicBool::new(false)),
        Err(DomError::ResourceLimit {
            resource: DomResource::DispatchCustodyScopes,
            limit: 1
        })
    ));
    drop(scope);
    let second = dom
        .register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
    assert_eq!(second.targets(), 1);
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&AtomicBool::new(false))
            .unwrap(),
        super::incremental_tests::enumerated(&dom.dispatch_custody)
    );
}

#[test]
fn same_edit_adoption_and_retirement_records_actual_staged_document_without_history() {
    let (mut dom, _, destination, target) = fixture(configured(2, 2));
    let scope = dom
        .register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
    let mut batch = DomBatch::new();
    batch.adopt(target, destination.node());
    batch.destroy(target);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let DomDispatchAssociation::Retired(stamp) =
        dom.dispatch_custody_association(&scope, 0, target).unwrap()
    else {
        panic!("successful canonical retirement proof")
    };
    assert_eq!(stamp.document(), destination);
    assert_eq!(stamp.store().revision(), dom.revision());
    assert_eq!(dom.usage().history_entries, 0);
}

#[test]
fn failed_atomic_retirement_publishes_no_cell_or_association_change() {
    let (mut dom, document, destination, target) = fixture(configured(2, 2));
    let scope = dom
        .register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
    let revision = dom.revision();
    let mut batch = DomBatch::new();
    batch.adopt(target, destination.node());
    batch.destroy(target);
    batch.destroy(document.node()); // Documents are not ordinary Destroy targets.
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidDestruction
    );
    assert_eq!(dom.revision(), revision);
    assert_eq!(
        dom.dispatch_custody_association(&scope, 0, target),
        Ok(DomDispatchAssociation::Current(document))
    );
    let mut retry = DomBatch::new();
    retry.destroy(target);
    dom.apply(retry, &AtomicBool::new(false)).unwrap();
    let DomDispatchAssociation::Retired(stamp) =
        dom.dispatch_custody_association(&scope, 0, target).unwrap()
    else {
        panic!("actual successful retry")
    };
    assert_eq!(stamp.document(), document);
}

#[test]
fn disabled_custody_does_not_disable_ordinary_destruction() {
    let (mut dom, _, _, target) = fixture(
        DomLimits::new(
            32,
            4096,
            64,
            0,
            0,
            0,
            0,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    );
    assert!(matches!(
        dom.register_dispatch_custody(&[target], &AtomicBool::new(false)),
        Err(DomError::ResourceLimit {
            resource: DomResource::DispatchCustodyScopes,
            limit: 0
        })
    ));
    let mut batch = DomBatch::new();
    batch.destroy(target);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        dom.snapshot().node(target).unwrap_err(),
        DomError::StaleNode
    );
}

#[test]
fn canceled_registration_publishes_no_scope_and_same_capacity_can_retry() {
    let (mut dom, _, _, target) = fixture(configured(1, 1));
    assert_eq!(
        dom.register_dispatch_custody(&[target], &AtomicBool::new(true))
            .unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&AtomicBool::new(false))
            .unwrap(),
        0
    );
    dom.register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
}

#[test]
fn retired_generation_cannot_register_again_or_qualify_a_fresh_identity() {
    let (mut dom, document, _, target) = fixture(configured(2, 2));
    let scope = dom
        .register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(target);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        dom.register_dispatch_custody(&[target], &AtomicBool::new(false))
            .unwrap_err(),
        DomError::StaleNode
    );
    let mut batch = DomBatch::new();
    let fresh = batch.create_comment(document.node(), "fresh");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let fresh = commit.created(&fresh).unwrap().unwrap();
    assert_ne!(fresh, target);
    assert_eq!(
        dom.dispatch_custody_association(&scope, 0, fresh),
        Err(DomError::InvalidDispatchCustody)
    );
    assert!(matches!(
        dom.dispatch_custody_association(&scope, 0, target),
        Ok(DomDispatchAssociation::Retired(_))
    ));
}

#[test]
fn retirement_after_adoption_to_new_surviving_document_uses_its_real_issued_identity() {
    let (mut dom, _, _, target) = fixture(configured(1, 1));
    let scope = dom
        .register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
    let mut batch = DomBatch::new();
    let destination = batch.create_document();
    batch.adopt(target, &destination);
    batch.destroy(target);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let destination = commit.document(&destination).unwrap();
    let DomDispatchAssociation::Retired(stamp) =
        dom.dispatch_custody_association(&scope, 0, target).unwrap()
    else {
        panic!("successful retirement has actual Document identity")
    };
    assert_eq!(stamp.document(), destination);
    assert_eq!(stamp.store().revision(), dom.revision());
}

#[test]
fn attempting_to_destroy_new_document_refuses_before_retirement_publication() {
    let (mut dom, document, _, target) = fixture(configured(1, 1));
    let scope = dom
        .register_dispatch_custody(&[target], &AtomicBool::new(false))
        .unwrap();
    let revision = dom.revision();
    let mut batch = DomBatch::new();
    let destination = batch.create_document();
    batch.adopt(target, &destination);
    batch.destroy(target);
    batch.destroy(&destination);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidDestruction
    );
    assert_eq!(dom.revision(), revision);
    assert_eq!(
        dom.dispatch_custody_association(&scope, 0, target),
        Ok(DomDispatchAssociation::Current(document))
    );
}

#[test]
fn completed_source_retirement_records_once_and_later_publication_preserves_its_stamp() {
    use crate::{DomSourceEdit, DomSourceLimits, DomSourceProgress};

    let cancel = AtomicBool::new(false);
    let (mut dom, document, _, target) = fixture(configured(1, 1));
    let scope = dom.register_dispatch_custody(&[target], &cancel).unwrap();
    let limits = DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096, 1);
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::destroy(target), limits, &cancel)
        .unwrap();
    let original = lease.status().operation();
    let DomSourceProgress::Complete(completion) = lease.advance(&cancel).unwrap() else {
        panic!("comment destruction requires no external service");
    };
    assert_eq!(completion.operation(), original);
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    let DomDispatchAssociation::Retired(stamp) =
        dom.dispatch_custody_association(&scope, 0, target).unwrap()
    else {
        panic!("completed original Source publishes actual retirement proof");
    };
    assert_eq!(stamp.document(), document);
    assert_eq!(stamp.store().revision(), dom.revision());
    assert_eq!(
        dom.dispatch_custody_association(&scope, 0, target),
        Ok(DomDispatchAssociation::Retired(stamp))
    );

    let mut batch = DomBatch::new();
    batch.create_comment(document.node(), "later publication");
    dom.apply(batch, &cancel).unwrap();
    assert!(dom.revision() > stamp.store().revision());
    assert_eq!(
        dom.dispatch_custody_association(&scope, 0, target),
        Ok(DomDispatchAssociation::Retired(stamp))
    );
    assert_eq!(dom.usage().history_entries(), 0);
}

#[test]
fn owner_destruction_records_shadow_and_template_descendants_in_their_actual_documents() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, _, _) = fixture(configured(1, 4));
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document.node(), crate::Namespace::html(), "div");
    batch.insert_before(&host, document.node(), None);
    let shadow = batch.attach_shadow(
        &host,
        crate::HtmlShadowRootProperties::new(
            crate::HtmlShadowRootMode::Open,
            crate::HtmlSlotAssignment::Manual,
            false,
            false,
            false,
        ),
    );
    let template = batch.create_element_ns(document.node(), crate::Namespace::html(), "template");
    batch.insert_before(&template, &shadow, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let shadow = commit.created(&shadow).unwrap().unwrap();
    let template = commit.created(&template).unwrap().unwrap();
    let contents = dom
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert_document = dom
        .snapshot()
        .node(contents)
        .unwrap()
        .owner_document()
        .unwrap();
    assert_ne!(inert_document, document);
    let mut batch = DomBatch::new();
    let descendant = batch.create_comment(inert_document.node(), "owned contents");
    batch.insert_before(&descendant, contents, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let descendant = commit.created(&descendant).unwrap().unwrap();
    let targets = [shadow, template, contents, descendant];
    let scope = dom.register_dispatch_custody(&targets, &cancel).unwrap();
    for (index, target) in targets.into_iter().enumerate() {
        let actual = dom
            .snapshot()
            .node(target)
            .unwrap()
            .owner_document()
            .unwrap();
        assert_eq!(
            dom.dispatch_custody_association(&scope, index, target),
            Ok(DomDispatchAssociation::Current(actual))
        );
    }
    let mut batch = DomBatch::new();
    batch.detach(host);
    batch.destroy(host);
    dom.apply(batch, &cancel).unwrap();
    for (index, (target, expected_document)) in targets
        .into_iter()
        .zip([document, document, inert_document, inert_document])
        .enumerate()
    {
        assert_eq!(
            dom.snapshot().node(target).unwrap_err(),
            DomError::StaleNode
        );
        let DomDispatchAssociation::Retired(stamp) = dom
            .dispatch_custody_association(&scope, index, target)
            .unwrap()
        else {
            panic!("actual owned destruction publishes descendant retirement");
        };
        assert_eq!(stamp.document(), expected_document);
        assert_eq!(stamp.store().revision(), dom.revision());
    }
    assert!(dom.snapshot().document(inert_document).is_ok());
    assert_eq!(dom.usage().history_entries(), 0);
}

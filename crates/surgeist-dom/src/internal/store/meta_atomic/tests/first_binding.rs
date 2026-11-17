//! First-owner resource custody uses the actual canonical bank and atomic fence.
use super::*;

#[test]
fn first_owner_staging_is_private_until_commit_and_drop_discards_it() {
    let mut f = fixture_binding(u64::MAX, false);
    let before = snapshots(&f);
    let mut batch = DomBatch::new();
    insert(&mut batch, f.docs[0], f.heads[0], "B".into());
    let caps = [&f.owners[0]];
    let prepared = prepare(&mut f.dom, batch, &caps, None, limits()).unwrap();
    assert!(
        prepared
            .dom
            .style_cssom_roots
            .get(f.docs[0].node())
            .is_none()
    );
    assert!(
        prepared
            .publication
            .style_custody
            .as_ref()
            .unwrap()
            .get(f.docs[0].node())
            .unwrap()
            .same_owner(&f.owners[0])
    );
    drop(prepared);
    unchanged(&f, &before);
    assert!(f.dom.style_cssom_roots.get(f.docs[0].node()).is_none());
}

#[test]
fn first_binding_publishes_with_unchanged_tree_without_issuing_destroyed_meta_identity() {
    let mut f = fixture_binding(u64::MAX, false);
    let before = snapshots(&f);
    let mut batch = DomBatch::new();
    let meta = insert(&mut batch, f.docs[0], f.heads[0], "B".into());
    batch.detach(&meta);
    batch.destroy(&meta);
    // Repeated references to the same actual owner reuse a single lock.
    let caps = [&f.owners[0], &f.owners[0]];
    let prepared = prepare(&mut f.dom, batch, &caps, None, limits()).unwrap();
    assert_eq!(prepared.occurrences().len(), 1);
    let (commit, dead) = prepared.commit(&flag()).unwrap().into_parts();
    assert_eq!(commit.dom().created(&meta).unwrap(), None);
    assert_eq!(commit.dom().changes().from(), before.0);
    assert_eq!(commit.dom().changes().to(), before.0);
    assert_eq!(f.dom.snapshot().stamp(), before.0);
    assert!(
        matches!(commit.roots()[0].cssom().publication(), CssomPublication::Unchanged {revision} if revision == before.1[0].revision())
    );
    assert!(
        f.dom
            .style_cssom_roots
            .get(f.docs[0].node())
            .unwrap()
            .same_owner(&f.owners[0])
    );
    drop(dead);
}

#[test]
fn later_actual_producer_refusal_does_not_install_an_earlier_first_binding() {
    let mut f = fixture_binding(1, false);
    let before = snapshots(&f);
    let mut batch = DomBatch::new();
    for i in 0..2 {
        insert(&mut batch, f.docs[i], f.heads[i], "B".into());
    }
    let caps = f.owners.iter().collect::<Vec<_>>();
    let mut prepared = prepare(&mut f.dom, batch, &caps, None, limits()).unwrap();
    for i in 0..2 {
        prepared
            .set_disabled(f.docs[i].node(), &f.sheets[i], true, &flag())
            .unwrap();
    }
    assert!(matches!(
        prepared.commit(&flag()),
        Err(DomMetaAtomicError::Cssom {
            error: CssomError::RevisionExhausted,
            ..
        })
    ));
    unchanged(&f, &before);
    for document in f.docs {
        assert!(f.dom.style_cssom_roots.get(document.node()).is_none());
    }
}

#[test]
fn document_destruction_refusal_cannot_publish_first_owner_or_earlier_meta() {
    let mut f = fixture_binding(u64::MAX, false);
    let before = snapshots(&f);
    let mut batch = DomBatch::new();
    insert(&mut batch, f.docs[1], f.heads[1], "B".into());
    // Documents are not destructible through the ordinary domain. This actual
    // refusal must abandon the earlier reached Meta, rather than faking a
    // retirement stimulus or retaining its private candidate owner.
    batch.destroy(f.docs[1].node());
    let caps = [&f.owners[1]];
    assert!(matches!(
        prepare(&mut f.dom, batch, &caps, None, limits()),
        Err(DomMetaAtomicError::Dom(DomError::InvalidDestruction))
    ));
    unchanged(&f, &before);
    assert!(f.dom.snapshot().document(f.docs[1]).is_ok());
    assert!(f.dom.style_cssom_roots.get(f.docs[1].node()).is_none());
}

#[test]
fn conflicting_provided_stores_cannot_choose_first_binding_or_override_canonical_owner_by_order() {
    for bound in [false, true] {
        for reversed in [false, true] {
            let mut f = fixture_binding(u64::MAX, bound);
            let before = snapshots(&f);
            let wrong = DomStyleCssomRoot::new(
                f.docs[0].node(),
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            );
            let caps = if reversed {
                [&wrong, &f.owners[0]]
            } else {
                [&f.owners[0], &wrong]
            };
            let mut batch = DomBatch::new();
            insert(&mut batch, f.docs[0], f.heads[0], "B".into());
            assert!(
                matches!(prepare(&mut f.dom, batch, &caps, None, limits()), Err(DomMetaAtomicError::MismatchedOwner {root}) if root == f.docs[0].node())
            );
            unchanged(&f, &before);
            assert_eq!(
                f.dom.style_cssom_roots.get(f.docs[0].node()).is_some(),
                bound
            );
            if bound {
                assert!(
                    f.dom
                        .style_cssom_roots
                        .get(f.docs[0].node())
                        .unwrap()
                        .same_owner(&f.owners[0])
                );
            }
        }
    }
}

#[test]
fn real_first_binding_registry_allocation_requires_extra_native_admission_before_canonical_retention()
 {
    let f = fixture_binding(u64::MAX, false);
    let before = snapshots(&f);
    f.dom.custody_registry.reconcile_releases();
    let basis = f.dom.custody_registry.ledger().total();
    let envelope = Envelope::new(
        DomMetaAtomicLimits::new(0, 0, 100_000, 0).unwrap(),
        DomMetaRuntimeUsage::new(0, 0, 0).unwrap(),
        0,
    )
    .unwrap();
    let result = f.dom.style_cssom_roots.prepare_insert(
        &f.owners[0],
        f.docs[0],
        |usage| envelope.controlled_extra(usage, basis, &flag()),
        || envelope.step(&flag()),
    );
    assert!(matches!(
        result,
        Err(DomError::ResourceLimit {
            resource: DomResource::MetaAtomicRetainedBytes,
            limit: 0
        })
    ));
    unchanged(&f, &before);
    assert!(f.dom.style_cssom_roots.get(f.docs[0].node()).is_none());
    f.dom.custody_registry.reconcile_releases();
    assert_eq!(f.dom.custody_registry.ledger().total(), basis);
}

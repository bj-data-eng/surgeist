//! CSSOM §6.2 caller-selected order with exact batch-local creation ownership.
use surgeist_css::{parse_media_query_list, parse_sheet};
use surgeist_cssom::*;

fn store(limits: CssomLimits) -> CssomStore {
    CssomStore::new(limits, CssomContext::default()).unwrap()
}
fn batch(owner: &CssomStore) -> CssomBatch {
    owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot()))
        .unwrap()
}
fn inputs(name: &str) -> CssomSheetInputs {
    let mut inputs = CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: name.into(),
            revision: 1,
        },
        true,
    );
    inputs.owner = Some(name.into());
    inputs
}
fn existing(owner: &mut CssomStore, name: &str) -> CssomSheetId {
    let mut edit = batch(owner);
    let ticket = edit.create_parsed_sheet("a {}", inputs(name)).unwrap();
    owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn create(edit: &mut CssomBatch, name: &str, source: &str, media: &str) -> CssomSheetConstruction {
    edit.create_parsed_sheet_with_media_reports(
        parse_sheet(source),
        parse_media_query_list(media),
        inputs(name),
    )
    .unwrap()
}

#[test]
fn parsed_creation_and_existing_sheets_publish_one_explicit_order_with_stable_identities() {
    let mut owner = store(CssomLimits::default());
    let a = existing(&mut owner, "a");
    let b = existing(&mut owner, "b");
    let before = owner.snapshot();
    let a_sheet = before.sheet(&a).unwrap();
    let a_rules = before.rules(a_sheet.rules()).unwrap().to_vec();
    let live_list = owner.style_sheet_list(before.sheet_list()).unwrap();
    let captured_list = live_list.capture(&before).unwrap();
    let mut edit = batch(&owner);
    let new = create(&mut edit, "new-style", "new { color: blue }", "screen");
    assert!(new.media_diagnostics().is_empty());
    assert!(
        edit.set_sheet_list_membership(
            before.sheet_list(),
            &[
                CssomSheetListTarget::Existing(&b),
                CssomSheetListTarget::Created(new.ticket()),
                CssomSheetListTarget::Existing(&a),
            ],
        )
        .unwrap()
    );
    // Staging never exposes the new list or a tentative identity to the store.
    assert_eq!(owner.snapshot().revision(), before.revision());
    assert_eq!(owner.snapshot().sheets(), &[a.clone(), b.clone()]);
    let committed = owner.commit(edit).unwrap();
    let new_id = committed.sheet(new.ticket()).unwrap();
    let after = owner.snapshot();
    assert_eq!(after.revision().value(), before.revision().value() + 1);
    assert_eq!(after.sheet_list(), before.sheet_list());
    assert_eq!(after.sheets(), &[b.clone(), new_id.clone(), a.clone()]);
    assert_eq!(live_list.length(&owner).unwrap(), 3);
    assert_eq!(captured_list.length().unwrap(), 2);
    assert_eq!(before.sheets(), &[a.clone(), b]);
    let retained = after.sheet(&a).unwrap();
    assert_eq!(retained.rules(), a_sheet.rules());
    assert_eq!(retained.media(), a_sheet.media());
    assert_eq!(retained.inputs(), a_sheet.inputs());
    assert_eq!(after.rules(retained.rules()).unwrap(), a_rules);
    let created = after.sheet(new_id).unwrap();
    assert_eq!(created.inputs().owner.as_deref(), Some("new-style"));
    assert_eq!(
        created.authored().parse_source().unwrap().source().as_str(),
        "new { color: blue }"
    );
    assert_eq!(
        after
            .media(created.media())
            .unwrap()
            .parse_source()
            .unwrap()
            .source()
            .as_str(),
        "screen"
    );
    let CssomPublication::Changed(summary) = committed.publication() else {
        panic!("creation and ordering are one real publication");
    };
    assert_eq!(summary.from(), before.revision());
    assert_eq!(summary.to(), after.revision());
    assert!(summary.categories().contains(&CssomChange::SheetMembership));
    assert!(summary.categories().contains(&CssomChange::Order));
    assert!(
        summary
            .affected()
            .contains(&CssomObjectId::SheetList(before.sheet_list().clone()))
    );
}

#[test]
fn multiple_created_tickets_resolve_in_supplied_order_in_the_same_commit() {
    let mut owner = store(CssomLimits::default());
    let old = existing(&mut owner, "old");
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    let first = create(&mut edit, "first", "first {}", "");
    let second = create(&mut edit, "second", "second {}", "");
    edit.set_sheet_list_membership(
        before.sheet_list(),
        &[
            CssomSheetListTarget::Created(second.ticket()),
            CssomSheetListTarget::Existing(&old),
            CssomSheetListTarget::Created(first.ticket()),
        ],
    )
    .unwrap();
    let committed = owner.commit(edit).unwrap();
    let first = committed.sheet(first.ticket()).unwrap();
    let second = committed.sheet(second.ticket()).unwrap();
    assert_ne!(first, second);
    assert_eq!(
        owner.snapshot().sheets(),
        &[second.clone(), old, first.clone()]
    );
    assert_eq!(
        owner.snapshot().revision().value(),
        before.revision().value() + 1
    );
    assert_eq!(before.sheets().len(), 1);
}

#[test]
fn other_batch_foreign_store_and_prior_commit_tickets_poison_without_partial_publication() {
    let mut owner = store(CssomLimits::default());
    let old = existing(&mut owner, "old");
    let mut sibling = batch(&owner);
    let sibling_ticket = create(&mut sibling, "sibling", "", "");
    let foreign = store(CssomLimits::default());
    let mut foreign_batch = batch(&foreign);
    let foreign_ticket = create(&mut foreign_batch, "foreign", "", "");
    let mut previous = batch(&owner);
    let previous_ticket = create(&mut previous, "previous", "", "");
    owner.commit(previous).unwrap();
    let before = owner.snapshot();
    for invalid in [
        sibling_ticket.ticket(),
        foreign_ticket.ticket(),
        previous_ticket.ticket(),
    ] {
        let mut edit = batch(&owner);
        let fresh = create(&mut edit, "unpublished", "", "");
        assert!(matches!(
            edit.set_sheet_list_membership(
                before.sheet_list(),
                &[
                    CssomSheetListTarget::Created(fresh.ticket()),
                    CssomSheetListTarget::Existing(&old),
                    CssomSheetListTarget::Created(invalid),
                ],
            ),
            Err(CssomError::ForeignTicket)
        ));
        assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(owner.snapshot().revision(), before.revision());
        assert_eq!(owner.snapshot().sheets(), before.sheets());
        assert_eq!(owner.snapshot().inputs(), before.inputs());
    }
}

#[test]
fn foreign_list_and_existing_sheet_reject_the_whole_creation_batch() {
    let mut owner = store(CssomLimits::default());
    let old = existing(&mut owner, "old");
    let mut foreign = store(CssomLimits::default());
    let foreign_sheet = existing(&mut foreign, "foreign");
    let before = owner.snapshot();
    let foreign_snapshot = foreign.snapshot();
    let mut edit = batch(&owner);
    let fresh = create(&mut edit, "new", "", "");
    assert!(matches!(
        edit.set_sheet_list_membership(
            foreign_snapshot.sheet_list(),
            &[CssomSheetListTarget::Created(fresh.ticket())],
        ),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    let mut edit = batch(&owner);
    let fresh = create(&mut edit, "new", "", "");
    assert!(matches!(
        edit.set_sheet_list_membership(
            before.sheet_list(),
            &[
                CssomSheetListTarget::Existing(&old),
                CssomSheetListTarget::Created(fresh.ticket()),
                CssomSheetListTarget::Existing(&foreign_sheet),
            ],
        ),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
    assert_eq!(owner.snapshot().sheets(), before.sheets());
}

#[test]
fn stale_commit_cannot_publish_created_sheet_or_selected_order() {
    let mut owner = store(CssomLimits::default());
    let old = existing(&mut owner, "old");
    let before = owner.snapshot();
    let mut stale = batch(&owner);
    let fresh = create(&mut stale, "stale", "stale {}", "");
    stale
        .set_sheet_list_membership(
            before.sheet_list(),
            &[
                CssomSheetListTarget::Created(fresh.ticket()),
                CssomSheetListTarget::Existing(&old),
            ],
        )
        .unwrap();
    let concurrent = existing(&mut owner, "concurrent");
    let current = owner.snapshot();
    assert!(matches!(
        owner.commit(stale),
        Err(CssomError::StaleRevision)
    ));
    assert_eq!(owner.snapshot().revision(), current.revision());
    assert_eq!(owner.snapshot().sheets(), &[old.clone(), concurrent]);
    assert_eq!(before.sheets(), &[old]);
}

#[test]
fn full_entry_limit_counts_retained_media_beside_mixed_selected_membership() {
    // Empty sheets/rule lists add zero entries; one screen media query plus
    // two selected memberships requires exactly three logical entries.
    for cap in [2, 3] {
        let mut owner = store(CssomLimits {
            max_entries: cap,
            ..CssomLimits::default()
        });
        let mut initial = batch(&owner);
        let old = initial
            .construct_sheet(
                CssomConstructorDocument {
                    version: CssomInputVersion {
                        role: CssomInputRole::Document,
                        identity: "document".into(),
                        revision: 1,
                    },
                    identity: "document".into(),
                    base_location: String::new(),
                },
                CssomSheetInit::default(),
            )
            .unwrap();
        let old = owner
            .commit(initial)
            .unwrap()
            .sheet(old.ticket())
            .unwrap()
            .clone();
        let before = owner.snapshot();
        assert!(before.sheets().is_empty());
        let mut edit = batch(&owner);
        let fresh = create(&mut edit, "new", "", "screen");
        let result = edit.set_sheet_list_membership(
            before.sheet_list(),
            &[
                CssomSheetListTarget::Created(fresh.ticket()),
                CssomSheetListTarget::Existing(&old),
            ],
        );
        if cap == 2 {
            assert!(matches!(
                result,
                Err(CssomError::Limit {
                    resource: "entries",
                    maximum: 2,
                })
            ));
            assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
            assert_eq!(owner.snapshot().revision(), before.revision());
            assert!(owner.snapshot().sheets().is_empty());
        } else {
            assert!(result.unwrap());
            let committed = owner.commit(edit).unwrap();
            let fresh = committed.sheet(fresh.ticket()).unwrap();
            assert_eq!(owner.snapshot().sheets(), &[fresh.clone(), old]);
            assert_eq!(
                owner.snapshot().revision().value(),
                before.revision().value() + 1
            );
        }
    }
}

#[test]
fn identical_and_restored_existing_order_preserve_revision_and_historical_list() {
    let mut owner = store(CssomLimits::default());
    let a = existing(&mut owner, "a");
    let b = existing(&mut owner, "b");
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    assert!(
        !edit
            .set_sheet_list_membership(
                before.sheet_list(),
                &[
                    CssomSheetListTarget::Existing(&a),
                    CssomSheetListTarget::Existing(&b),
                ],
            )
            .unwrap()
    );
    assert!(matches!(
        owner.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { revision } if revision == before.revision()
    ));
    let mut edit = batch(&owner);
    edit.set_sheet_list_membership(
        before.sheet_list(),
        &[
            CssomSheetListTarget::Existing(&b),
            CssomSheetListTarget::Existing(&a),
        ],
    )
    .unwrap();
    edit.set_sheet_list_membership(
        before.sheet_list(),
        &[
            CssomSheetListTarget::Existing(&a),
            CssomSheetListTarget::Existing(&b),
        ],
    )
    .unwrap();
    assert!(matches!(
        owner.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { revision } if revision == before.revision()
    ));
    assert_eq!(owner.snapshot().sheets(), &[a, b]);
    assert_eq!(owner.snapshot().revision(), before.revision());
}

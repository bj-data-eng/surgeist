//! CSSOM #remove-a-css-style-sheet, with retained-object lifetime selected by #840.
use surgeist_cssom::*;

fn version(role: CssomInputRole, name: &str) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: name.into(),
        revision: 1,
    }
}
fn store(limits: CssomLimits) -> CssomStore {
    CssomStore::new(limits, CssomContext::default()).unwrap()
}
fn guard(store: &CssomStore) -> CssomEditGuard {
    CssomEditGuard::from_snapshot(&store.snapshot())
}
fn batch(store: &CssomStore) -> CssomBatch {
    store.batch(guard(store)).unwrap()
}
fn external(store: &mut CssomStore, source: &str, owner: Option<&str>) -> CssomSheetId {
    let mut inputs = CssomSheetInputs::external(version(CssomInputRole::Owner, "sheet"), true);
    inputs.owner = owner.map(str::to_owned);
    inputs.location = Some("https://example.test/sheet.css".into());
    let mut edit = batch(store);
    let ticket = edit.create_parsed_sheet(source, inputs).unwrap();
    store.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn rules(snapshot: &CssomSnapshot, sheet: &CssomSheetId) -> Vec<CssomRuleId> {
    snapshot
        .rules(snapshot.sheet(sheet).unwrap().rules())
        .unwrap()
        .to_vec()
}

#[test]
fn removal_keeps_sheet_rules_media_live_handles_and_historical_captures() {
    let mut owner = store(CssomLimits::default());
    let a = external(&mut owner, "a {}", None);
    let removed = external(&mut owner, "b { color: red; }", Some("style-node"));
    let c = external(&mut owner, "c {}", None);
    let before = owner.snapshot();
    let handle = owner.css_style_sheet(&removed).unwrap();
    let captured = handle.capture(&before).unwrap();
    let list = handle.css_rules(&owner).unwrap();
    let rule = list.item(&owner, 0).unwrap().unwrap();
    let old_text = rule
        .css_text(&owner, &CssomRuleFormatLimits::default())
        .unwrap();
    let media = handle.media(&owner).unwrap();
    let inputs = before.sheet(&removed).unwrap().inputs().clone();
    let mut edit = batch(&owner);
    assert!(edit.remove_sheet(&removed).unwrap());
    let committed = owner.commit(edit).unwrap();
    let after = owner.snapshot();
    assert_eq!(after.sheets(), &[a, c]);
    assert_eq!(handle.owner_node(&owner).unwrap(), None);
    assert_eq!(handle.parent_style_sheet(&owner).unwrap(), None);
    assert_eq!(handle.owner_rule(&owner).unwrap(), None);
    assert_eq!(handle.css_rules(&owner).unwrap(), list);
    assert_eq!(list.length(&owner).unwrap(), 1);
    assert_eq!(list.item(&owner, 0).unwrap(), Some(rule.clone()));
    assert_eq!(
        rule.css_text(&owner, &CssomRuleFormatLimits::default())
            .unwrap(),
        old_text
    );
    assert_eq!(
        rule.parent_style_sheet(&owner).unwrap(),
        Some(handle.clone())
    );
    assert_eq!(handle.media(&owner).unwrap(), media);
    assert_eq!(
        after.sheet(&removed).unwrap().inputs().version,
        inputs.version
    );
    assert_eq!(
        after.sheet(&removed).unwrap().inputs().location,
        inputs.location
    );
    assert_eq!(captured.owner_node().unwrap(), Some("style-node"));
    assert_eq!(captured.rules().unwrap().length().unwrap(), 1);
    assert_eq!(before.sheets().len(), 3);
    let CssomPublication::Changed(summary) = committed.publication() else {
        panic!("actual removal must publish");
    };
    for change in [
        CssomChange::SheetMembership,
        CssomChange::Order,
        CssomChange::Owner,
    ] {
        assert!(summary.categories().contains(&change));
    }
    assert!(
        summary
            .affected()
            .contains(&CssomObjectId::Sheet(removed.clone()))
    );
    assert!(
        summary
            .affected()
            .contains(&CssomObjectId::SheetList(after.sheet_list().clone()))
    );
    assert!(!summary.categories().contains(&CssomChange::RuleMembership));
    assert!(matches!(
        owner.changes_since(&before),
        CssomChanges::Incremental(_)
    ));
    let revision = after.revision().clone();
    let mut edit = batch(&owner);
    assert!(!edit.remove_sheet(&removed).unwrap());
    assert!(
        matches!(owner.commit(edit).unwrap().publication(), CssomPublication::Unchanged { revision: current } if current == &revision)
    );
    let mut edit = batch(&owner);
    edit.insert_sheet_rule(&removed, "d {}", 1).unwrap();
    owner.commit(edit).unwrap();
    assert_eq!(handle.css_rules(&owner).unwrap(), list);
    assert_eq!(list.length(&owner).unwrap(), 2);
    assert_eq!(list.item(&owner, 0).unwrap(), Some(rule));
    assert_eq!(captured.rules().unwrap().length().unwrap(), 1);
    drop(owner);
    assert_eq!(captured.owner_node().unwrap(), Some("style-node"));
    assert_eq!(captured.rules().unwrap().length().unwrap(), 1);
}

#[test]
fn imported_removal_clears_parent_but_retains_forward_import_and_same_input_reattaches() {
    let mut owner = store(CssomLimits::default());
    let parent = external(&mut owner, "@import 'child.css';", None);
    let child = external(&mut owner, "@import 'grandchild.css'; b {}", Some("host"));
    let grandchild = external(&mut owner, "c {}", None);
    let snapshot = owner.snapshot();
    let import = rules(&snapshot, &parent)[0].clone();
    let child_import = rules(&snapshot, &child)[0].clone();
    let input = version(CssomInputRole::Import, "child-fetch");
    let location = Some("https://example.test/child.css".to_owned());
    let mut edit = batch(&owner);
    edit.associate_import(&import, &child, input.clone(), location.clone())
        .unwrap();
    edit.associate_import(
        &child_import,
        &grandchild,
        version(CssomInputRole::Import, "grandchild-fetch"),
        None,
    )
    .unwrap();
    owner.commit(edit).unwrap();
    let before = owner.snapshot();
    let import_handle = owner.css_import_rule(&import).unwrap();
    let mut edit = batch(&owner);
    assert!(edit.remove_sheet(&child).unwrap());
    let commit = owner.commit(edit).unwrap();
    let after = owner.snapshot();
    assert_eq!(after.parent_sheet(&child).unwrap(), None);
    assert_eq!(after.sheet(&child).unwrap().owner_rule(), None);
    assert_eq!(after.sheet(&child).unwrap().inputs().owner, None);
    assert_eq!(
        import_handle.style_sheet(&owner).unwrap().unwrap().id(),
        &child
    );
    assert_eq!(after.parent_sheet(&grandchild).unwrap(), Some(&child));
    assert_eq!(
        after.sheet(&grandchild).unwrap().owner_rule(),
        Some(&child_import)
    );
    assert_eq!(rules(&after, &child), rules(&before, &child));
    assert_eq!(before.parent_sheet(&child).unwrap(), Some(&parent));
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("removal must publish");
    };
    for change in [
        CssomChange::Owner,
        CssomChange::Ancestry,
        CssomChange::ImportLinks,
    ] {
        assert!(summary.categories().contains(&change));
    }
    assert!(matches!(
        owner.changes_since(&before),
        CssomChanges::FullRecompute(CssomRecomputeReason::ConservativeChange)
    ));

    let mut edit = batch(&owner);
    assert!(
        edit.associate_import(&import, &child, input.clone(), location.clone())
            .unwrap()
    );
    owner.commit(edit).unwrap();
    assert_eq!(
        owner.snapshot().parent_sheet(&child).unwrap(),
        Some(&parent)
    );
    assert_eq!(
        owner.snapshot().sheet(&child).unwrap().owner_rule(),
        Some(&import)
    );
    assert_eq!(owner.snapshot().sheet(&child).unwrap().inputs().owner, None);
    let current = owner.snapshot();
    let mut edit = batch(&owner);
    assert!(
        !edit
            .associate_import(&import, &child, input, location)
            .unwrap()
    );
    assert!(
        matches!(owner.commit(edit).unwrap().publication(), CssomPublication::Unchanged { revision } if revision == current.revision())
    );
    // Reattachment restores only the import relation, not host-list membership.
    // Removing that relation alone must still be an observable publication.
    assert!(!current.sheets().contains(&child));
    let mut edit = batch(&owner);
    assert!(edit.remove_sheet(&child).unwrap());
    let commit = owner.commit(edit).unwrap();
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("owner-rule-only clearing must publish");
    };
    assert!(!summary.categories().contains(&CssomChange::SheetMembership));
    assert_eq!(owner.snapshot().parent_sheet(&child).unwrap(), None);
    assert_eq!(
        import_handle.style_sheet(&owner).unwrap().unwrap().id(),
        &child
    );
}

#[test]
fn association_only_removal_publishes_without_selected_membership() {
    let mut owner = store(CssomLimits::default());
    let sheet = external(&mut owner, "a {}", Some("style"));
    let mut edit = batch(&owner);
    edit.set_sheet_list_membership(owner.snapshot().sheet_list(), &[])
        .unwrap();
    owner.commit(edit).unwrap();
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    assert!(edit.remove_sheet(&sheet).unwrap());
    let commit = owner.commit(edit).unwrap();
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("owner-only clearing must publish");
    };
    assert!(summary.categories().contains(&CssomChange::Owner));
    assert!(!summary.categories().contains(&CssomChange::SheetMembership));
    assert!(owner.snapshot().sheets().is_empty());
    assert!(
        owner
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    assert_eq!(
        before.sheet(&sheet).unwrap().inputs().owner.as_deref(),
        Some("style")
    );
}

#[test]
fn stale_commit_and_foreign_operation_leave_membership_and_owner_unchanged() {
    let mut owner = store(CssomLimits::default());
    let sheet = external(&mut owner, "a {}", Some("style"));
    let mut stale = batch(&owner);
    stale.remove_sheet(&sheet).unwrap();
    let _other = external(&mut owner, "b {}", None);
    let before = owner.snapshot();
    assert!(matches!(
        owner.commit(stale),
        Err(CssomError::StaleRevision)
    ));
    assert_eq!(owner.snapshot().revision(), before.revision());
    assert_eq!(
        owner
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .owner
            .as_deref(),
        Some("style")
    );
    let mut foreign = store(CssomLimits::default());
    let foreign_sheet = external(&mut foreign, "", None);
    let mut edit = batch(&owner);
    assert!(edit.remove_sheet(&sheet).unwrap());
    assert!(matches!(
        edit.remove_sheet(&foreign_sheet),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        edit.remove_sheet(&sheet),
        Err(CssomError::BatchAborted)
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
    assert_eq!(owner.snapshot().sheets(), before.sheets());
}

#[test]
fn removal_retains_object_quota_and_revision_failure_is_atomic() {
    let mut owner = store(CssomLimits {
        max_objects: 7,
        ..Default::default()
    });
    let first = external(&mut owner, "", None);
    let second = external(&mut owner, "", None);
    let mut edit = batch(&owner);
    edit.remove_sheet(&first).unwrap();
    owner.commit(edit).unwrap();
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.create_parsed_sheet(
            "",
            CssomSheetInputs::external(version(CssomInputRole::Owner, "third"), true)
        ),
        Err(CssomError::Limit {
            resource: "objects",
            maximum: 7
        })
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
    assert!(owner.snapshot().sheet(&first).is_ok());
    assert_eq!(owner.snapshot().sheets(), &[second]);

    let mut exhausted = store(CssomLimits {
        max_revision: 1,
        ..Default::default()
    });
    let sheet = external(&mut exhausted, "a {}", Some("style"));
    let before = exhausted.snapshot();
    let mut edit = batch(&exhausted);
    edit.remove_sheet(&sheet).unwrap();
    assert!(matches!(
        exhausted.commit(edit),
        Err(CssomError::RevisionExhausted)
    ));
    assert_eq!(exhausted.snapshot().revision(), before.revision());
    assert_eq!(exhausted.snapshot().sheets(), std::slice::from_ref(&sheet));
    assert_eq!(
        exhausted
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .owner
            .as_deref(),
        Some("style")
    );
}

#[test]
fn removal_does_not_inherit_css_rules_security_or_modification_guards() {
    let mut owner = store(CssomLimits::default());
    let mut inputs =
        CssomSheetInputs::external(version(CssomInputRole::Owner, "cross-origin"), false);
    inputs.owner = Some("link".into());
    inputs.disallow_modification = true;
    let mut edit = batch(&owner);
    let ticket = edit.create_parsed_sheet("a {}", inputs).unwrap();
    let sheet = owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let mut edit = batch(&owner);
    assert!(edit.remove_sheet(&sheet).unwrap());
    owner.commit(edit).unwrap();
    let current = owner.snapshot();
    assert!(
        current
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    assert!(!current.sheet(&sheet).unwrap().inputs().origin_clean);
    assert!(matches!(
        current.sheet_rules(&sheet),
        Err(CssomError::Source(CssomException::Security))
    ));
    assert!(current.sheets().is_empty());
    assert_eq!(rules(&current, &sheet).len(), 1);
}

#[test]
fn removal_keeps_pending_replacement_and_its_terminal_work_on_the_same_sheet() {
    let mut owner = store(CssomLimits::default());
    let mut edit = batch(&owner);
    let construction = edit
        .construct_sheet(
            CssomConstructorDocument {
                version: version(CssomInputRole::Document, "doc"),
                identity: "doc".into(),
                base_location: "https://example.test/".into(),
            },
            CssomSheetInit::default(),
        )
        .unwrap();
    let sheet = owner
        .commit(edit)
        .unwrap()
        .sheet(construction.ticket())
        .unwrap()
        .clone();
    let mut edit = batch(&owner);
    edit.set_sheet_list_membership(
        owner.snapshot().sheet_list(),
        &[CssomSheetListTarget::Existing(&sheet)],
    )
    .unwrap();
    owner.commit(edit).unwrap();
    let job = owner
        .begin_replace_sheet(guard(&owner), &sheet, "a { color: red; }")
        .unwrap();
    let token = job.token().clone();
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    assert!(edit.remove_sheet(&sheet).unwrap());
    owner.commit(edit).unwrap();
    assert_eq!(
        owner.pending_sheet_replacement(&sheet).unwrap(),
        Some(&token)
    );
    assert!(
        owner
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    assert!(owner.snapshot().sheets().is_empty());
    let completion = owner
        .finish_replace_sheet(guard(&owner), job.parse())
        .unwrap();
    assert!(matches!(
        completion.outcome(),
        CssomReplaceOutcome::Replaced
    ));
    assert!(owner.pending_sheet_replacement(&sheet).unwrap().is_none());
    assert!(owner.snapshot().sheets().is_empty());
    assert!(
        !owner
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    assert_eq!(rules(&owner.snapshot(), &sheet).len(), 1);
    assert_eq!(rules(&before, &sheet).len(), 0);
    assert_eq!(before.sheets(), &[sheet]);
}

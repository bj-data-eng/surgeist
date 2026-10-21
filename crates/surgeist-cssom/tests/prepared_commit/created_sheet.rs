use super::*;

fn revision(commit: &CssomCommit) -> &CssomRevision {
    match commit.publication() {
        CssomPublication::Changed(summary) => summary.to(),
        CssomPublication::Unchanged { revision } => revision,
    }
}

#[test]
fn prepared_sheet_metadata_matches_actual_publication_and_rejects_other_batch_tickets() {
    let mut owner = store(Default::default());
    let mut other = store(Default::default());
    let mut other_edit = batch(&other);
    let foreign = other_edit
        .create_typed_sheet(CssSheet::new(), inputs())
        .unwrap();
    drop(other.prepare_commit(other_edit).unwrap());
    let mut sibling = batch(&owner);
    let wrong = sibling
        .create_typed_sheet(CssSheet::new(), inputs())
        .unwrap();
    let mut edit = batch(&owner);
    let ticket = edit.create_parsed_sheet("a {color:red}", inputs()).unwrap();
    let before = owner.snapshot();
    let prepared = owner.prepare_commit(edit).unwrap();
    for invalid in [&foreign, &wrong] {
        assert!(matches!(
            prepared.sheet(invalid),
            Err(CssomError::ForeignTicket)
        ));
    }
    let metadata = prepared.sheet(&ticket).unwrap();
    let id = metadata.id().clone();
    let proposed = metadata.revision().clone();
    assert_eq!(id.owner(), before.owner());
    assert!(matches!(before.sheet(&id), Err(CssomError::MissingObject)));
    let (commit, retirement) = prepared.publish();
    assert_eq!(commit.sheet(&ticket).unwrap(), &id);
    assert_eq!(revision(&commit), &proposed);
    assert_eq!(owner.snapshot().revision(), &proposed);
    assert!(owner.snapshot().sheet(&id).is_ok());
    assert!(commit.owner_effects().is_empty());
    drop((sibling, retirement));
}

#[test]
fn abandoned_metadata_never_aliases_any_later_sheet_ingress_but_revision_can_repeat() {
    let mut owner = store(Default::default());
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    let ticket = edit.create_typed_sheet(CssSheet::new(), inputs()).unwrap();
    let prepared = owner.prepare_commit(edit).unwrap();
    let abandoned = prepared.sheet(&ticket).unwrap().id().clone();
    let proposed = prepared.sheet(&ticket).unwrap().revision().clone();
    drop(prepared);
    assert_unchanged(&owner, &before);
    for kind in 0..4 {
        let mut edit = batch(&owner);
        let ticket = match kind {
            0 => edit.create_typed_sheet(CssSheet::new(), inputs()).unwrap(),
            1 => edit.create_parsed_sheet("a{}", inputs()).unwrap(),
            2 => {
                let report = CssParserContext::default().parse_sheet("b{}");
                edit.create_parsed_sheet_report(report, inputs()).unwrap()
            }
            _ => {
                let context = CssParserContext::default();
                let sheet = context.parse_sheet("c{}");
                let media = parse_media_query_list("screen");
                edit.create_parsed_sheet_with_media_reports(sheet, media, inputs())
                    .unwrap()
                    .ticket()
                    .clone()
            }
        };
        let prepared = owner.prepare_commit(edit).unwrap();
        let actual = prepared.sheet(&ticket).unwrap().id().clone();
        assert_ne!(actual, abandoned);
        if kind == 0 {
            assert_eq!(prepared.sheet(&ticket).unwrap().revision(), &proposed);
        }
        let (commit, retirement) = prepared.publish();
        assert_eq!(commit.sheet(&ticket).unwrap(), &actual);
        assert!(matches!(
            owner.snapshot().sheet(&abandoned),
            Err(CssomError::MissingObject)
        ));
        drop(retirement);
    }
}

#[test]
fn detached_competing_batches_issue_distinct_ids_before_either_is_prepared() {
    let mut owner = store(Default::default());
    let before = owner.snapshot();
    let mut a = batch(&owner);
    let mut b = batch(&owner);
    let (a_ticket, b_ticket) = std::thread::scope(|scope| {
        let a = scope.spawn(|| a.create_typed_sheet(CssSheet::new(), inputs()).unwrap());
        let b = scope.spawn(|| b.create_typed_sheet(CssSheet::new(), inputs()).unwrap());
        (a.join().unwrap(), b.join().unwrap())
    });
    let prepared = owner.prepare_commit(a).unwrap();
    let abandoned = prepared.sheet(&a_ticket).unwrap().id().clone();
    drop(prepared);
    assert_unchanged(&owner, &before);
    let prepared = owner.prepare_commit(b).unwrap();
    let published = prepared.sheet(&b_ticket).unwrap().id().clone();
    assert_ne!(published, abandoned);
    let (commit, retirement) = prepared.publish();
    assert_eq!(commit.sheet(&b_ticket).unwrap(), &published);
    assert!(matches!(
        owner.snapshot().sheet(&abandoned),
        Err(CssomError::MissingObject)
    ));
    drop(retirement);
}

#[test]
fn stale_loser_burns_finite_headroom_without_changing_error_precedence_or_state() {
    // Initial sheet list uses one identity; two empty sheets use three each.
    let mut owner = store(CssomLimits {
        max_identity: 7,
        ..Default::default()
    });
    let mut winner = batch(&owner);
    let ticket = winner
        .create_typed_sheet(CssSheet::new(), inputs())
        .unwrap();
    let mut loser = batch(&owner);
    loser.create_typed_sheet(CssSheet::new(), inputs()).unwrap();
    let commit = owner.commit(winner).unwrap();
    let before = owner.snapshot();
    assert!(matches!(
        owner.prepare_commit(loser),
        Err(CssomError::StaleRevision)
    ));
    let mut retry = batch(&owner);
    assert!(matches!(
        retry.create_typed_sheet(CssSheet::new(), inputs()),
        Err(CssomError::IdentityExhausted)
    ));
    assert!(matches!(
        owner.prepare_commit(retry),
        Err(CssomError::BatchAborted)
    ));
    assert_unchanged(&owner, &before);
    assert_eq!(
        owner.snapshot().sheets(),
        std::slice::from_ref(commit.sheet(&ticket).unwrap())
    );
}

#[test]
fn abandoning_a_preparation_burns_exact_identity_capacity_without_publishing_its_sheet() {
    let mut owner = store(CssomLimits {
        max_identity: 7,
        ..Default::default()
    });
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    let ticket = edit.create_typed_sheet(CssSheet::new(), inputs()).unwrap();
    let prepared = owner.prepare_commit(edit).unwrap();
    let abandoned = prepared.sheet(&ticket).unwrap().id().clone();
    drop(prepared);
    assert_unchanged(&owner, &before);
    let mut edit = batch(&owner);
    let real = edit.create_typed_sheet(CssSheet::new(), inputs()).unwrap();
    owner.commit(edit).unwrap().sheet(&real).unwrap();
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.create_typed_sheet(CssSheet::new(), inputs()),
        Err(CssomError::IdentityExhausted)
    ));
    assert!(matches!(
        owner.snapshot().sheet(&abandoned),
        Err(CssomError::MissingObject)
    ));
    let noop = batch(&owner);
    let prepared = owner.prepare_commit(noop).unwrap();
    assert!(matches!(
        prepared.sheet(&ticket),
        Err(CssomError::ForeignTicket)
    ));
    let (commit, retirement) = prepared.publish();
    assert!(matches!(
        commit.publication(),
        CssomPublication::Unchanged { .. }
    ));
    drop(retirement);
}

#[test]
fn sibling_burns_do_not_turn_empty_or_restored_batches_into_changed_publications() {
    let mut owner = store(Default::default());
    let id = sheet(&mut owner);
    let before = owner.snapshot();
    for restore in [false, true] {
        let mut noop = batch(&owner);
        if restore {
            noop.set_disabled(&id, true).unwrap();
            noop.set_disabled(&id, false).unwrap();
        }
        let mut sibling = batch(&owner);
        sibling
            .create_typed_sheet(CssSheet::new(), inputs())
            .unwrap();
        drop(sibling);
        let (commit, retirement) = owner.prepare_commit(noop).unwrap().publish();
        assert_eq!(revision(&commit), before.revision());
        assert!(matches!(
            commit.publication(),
            CssomPublication::Unchanged { .. }
        ));
        assert_unchanged(&owner, &before);
        drop(retirement);
    }
}

#[test]
fn replacement_tokens_and_detached_creation_share_one_nonrollback_identity_domain() {
    // Root list1, constructed empty sheet3, detached sheet3, replacement token1.
    let mut owner = store(CssomLimits {
        max_identity: 8,
        ..Default::default()
    });
    let mut edit = batch(&owner);
    let ticket = edit.create_typed_sheet(CssSheet::new(), inputs()).unwrap();
    let id = owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let mut detached = batch(&owner);
    detached
        .create_typed_sheet(CssSheet::new(), inputs())
        .unwrap();
    let job = owner.begin_replace_sheet(guard(&owner), &id, "").unwrap();
    assert!(matches!(
        owner.prepare_commit(detached),
        Err(CssomError::StaleRevision)
    ));
    assert!(matches!(
        owner.cancel_replace_sheet(job.token()).unwrap().outcome(),
        CssomReplaceOutcome::Cancelled
    ));
    let mut exhausted = batch(&owner);
    assert!(matches!(
        exhausted.create_typed_sheet(CssSheet::new(), inputs()),
        Err(CssomError::IdentityExhausted)
    ));
    assert!(owner.pending_sheet_replacement(&id).unwrap().is_none());
    assert!(
        !owner
            .snapshot()
            .sheet(&id)
            .unwrap()
            .inputs()
            .disallow_modification
    );
}

use super::*;

fn packet(name: &str) -> CssomInlineDeclarationContext {
    let support = CssomDeclarationSupport::try_new(vec![(
        CssPropertyGrammar::from_name("width").unwrap(),
        CssomUsabilityRequirement::AllCheckedValues,
    )])
    .unwrap();
    CssomInlineDeclarationContext::try_new(CssomContext {
        inputs: vec![
            CssomInput {
                version: version(CssomInputRole::Owner, name, 1),
                data: CssomInputData::Owner {
                    identity: name.into(),
                    attribute: Some("style".into()),
                },
            },
            CssomInput {
                version: version(CssomInputRole::ParserMode, "parser", 1),
                data: CssomInputData::ParserMode(CssParserContext::default()),
            },
            CssomInput {
                version: version(CssomInputRole::Support, "support", 1),
                data: CssomInputData::Support(support),
            },
        ],
        linked: Vec::new(),
    })
    .unwrap()
}
fn block(owner: &mut CssomStore) -> CssomBlockId {
    let mut edit = batch(owner);
    let ticket = edit.create_inline_declarations(packet("element")).unwrap();
    owner.commit(edit).unwrap().block(&ticket).unwrap().clone()
}
fn set(edit: &mut CssomBatch, block: &CssomBlockId) {
    let preparation = edit
        .prepare_set_property(block, "width", "1px", "", Default::default())
        .unwrap();
    edit.apply_declaration(preparation, &[]).unwrap();
}
fn sync(owner: &mut CssomStore, block: &CssomBlockId) -> CssomOwnerEffect {
    let mut edit = batch(owner);
    edit.request_owner_synchronization(block, Default::default())
        .unwrap();
    owner
        .commit(edit)
        .unwrap()
        .take_owner_effects()
        .pop()
        .unwrap()
}

#[test]
fn effect_only_publication_preserves_revision_and_returns_genuine_current_effect() {
    let mut owner = store(CssomLimits::default());
    let id = block(&mut owner);
    let mut edit = batch(&owner);
    set(&mut edit, &id);
    let (mut commit, retirement) = owner.prepare_commit(edit).unwrap().publish();
    assert_eq!(commit.owner_effects().len(), 1);
    let after = owner.snapshot();
    assert_eq!(commit.owner_effects()[0].value(), "width: 1px;");
    assert_eq!(commit.owner_effects()[0].block(), &id);
    assert_eq!(commit.owner_effects()[0].revision(), after.revision());
    assert_eq!(
        commit.owner_effects()[0].owner(),
        after.block(&id).unwrap().owner().unwrap()
    );
    owner
        .validate_pending_owner_effect(&commit.owner_effects()[0])
        .unwrap();
    let mut first = commit.take_owner_effects().pop().unwrap();
    first.cancel().unwrap();
    drop(retirement);
    let mut edit = batch(&owner);
    edit.request_owner_synchronization(&id, Default::default())
        .unwrap();
    let (mut commit, retirement) = owner.prepare_commit(edit).unwrap().publish();
    assert!(
        matches!(commit.publication(), CssomPublication::Unchanged { revision } if revision == after.revision())
    );
    let mut actual = commit.take_owner_effects().pop().unwrap();
    assert_ne!(actual.id(), first.id());
    assert_eq!(actual.value(), "width: 1px;");
    owner.validate_pending_owner_effect(&actual).unwrap();
    owner
        .begin_owner_application(&mut actual)
        .unwrap()
        .finish_success();
    assert_eq!(actual.status(), CssomOwnerEffectStatus::Applied);
    assert_unchanged(&owner, &after);
    drop(retirement);
}

#[test]
fn old_effect_release_during_prepared_reservation_then_abort_restores_exact_capacity_not_old_counter()
 {
    let bytes = "width: 1px;".len();
    let mut owner = store(CssomLimits {
        max_pending_owner_effects: 2,
        max_pending_owner_effect_bytes: bytes * 2,
        max_owner_effect_identity: 4,
        ..Default::default()
    });
    let id = block(&mut owner);
    let mut edit = batch(&owner);
    set(&mut edit, &id);
    let mut initial = owner
        .commit(edit)
        .unwrap()
        .take_owner_effects()
        .pop()
        .unwrap();
    initial.cancel().unwrap(); // published identity 1
    let mut old = sync(&mut owner, &id); // published identity 2
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    edit.request_owner_synchronization(&id, Default::default())
        .unwrap();
    let prepared = owner.prepare_commit(edit).unwrap(); // privately reserves prospective identity 3
    old.cancel().unwrap(); // atomic quota release needs no Store access
    drop(prepared); // only this private effect's exact count/bytes are released
    assert_unchanged(&owner, &before);
    let mut a = sync(&mut owner, &id); // identity 3, not spent by abandoned preparation
    let mut b = sync(&mut owner, &id); // identity 4; both fit exact two-effect byte/count quotas
    assert_ne!(a.id(), old.id());
    assert_ne!(a.id(), b.id());
    assert_eq!(a.value().len() + b.value().len(), bytes * 2);
    owner.validate_pending_owner_effect(&a).unwrap();
    owner.validate_pending_owner_effect(&b).unwrap();
    a.cancel().unwrap();
    b.cancel().unwrap();
    let mut edit = batch(&owner);
    edit.request_owner_synchronization(&id, Default::default())
        .unwrap();
    assert!(matches!(
        owner.prepare_commit(edit),
        Err(CssomError::EffectIdentityExhausted)
    ));
    assert_unchanged(&owner, &before);
}

#[test]
fn later_effect_limit_failure_releases_earlier_private_effect_without_spending_identity() {
    let mut owner = store(CssomLimits {
        max_pending_owner_effects: 1,
        max_owner_effect_identity: 2,
        ..Default::default()
    });
    let a = block(&mut owner);
    let b = block(&mut owner);
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    edit.request_owner_synchronization(&a, Default::default())
        .unwrap();
    edit.request_owner_synchronization(&b, Default::default())
        .unwrap();
    assert!(matches!(
        owner.prepare_commit(edit),
        Err(CssomError::Limit {
            resource: "pending owner effects",
            maximum: 1
        })
    ));
    assert_unchanged(&owner, &before);
    let mut actual_a = sync(&mut owner, &a);
    actual_a.cancel().unwrap();
    let mut actual_b = sync(&mut owner, &b);
    actual_b.cancel().unwrap();
    assert_ne!(actual_a.id(), actual_b.id());
}

#[test]
fn qualification_unresolved_and_poisoned_error_precedence_remains_consuming_and_atomic() {
    let mut owner = store(CssomLimits::default());
    let id = block(&mut owner);
    let own_sheet = sheet(&mut owner);
    let mut foreign = store(CssomLimits::default());
    let foreign_sheet = sheet(&mut foreign);
    let before = owner.snapshot();

    let mut edit = batch(&owner);
    let pending = edit
        .prepare_set_property(&id, "width", "1px", "", Default::default())
        .unwrap();
    assert!(matches!(
        edit.set_disabled(&foreign_sheet, true),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        owner.prepare_commit(edit),
        Err(CssomError::UnresolvedDeclarationPreparation)
    ));
    drop(pending);
    assert_unchanged(&owner, &before);

    let mut poisoned = batch(&owner);
    assert!(poisoned.set_disabled(&foreign_sheet, true).is_err());
    assert!(matches!(
        owner.prepare_commit(poisoned),
        Err(CssomError::BatchAborted)
    ));
    let mut foreign_batch = batch(&owner);
    let pending = foreign_batch
        .prepare_set_property(&id, "width", "1px", "", Default::default())
        .unwrap();
    assert!(foreign_batch.set_disabled(&foreign_sheet, true).is_err());
    assert!(matches!(
        foreign.prepare_commit(foreign_batch),
        Err(CssomError::ForeignOwner)
    ));
    drop(pending);

    let mut stale = batch(&owner);
    let pending = stale
        .prepare_set_property(&id, "width", "1px", "", Default::default())
        .unwrap();
    assert!(stale.set_disabled(&foreign_sheet, true).is_err());
    let mut fresh = batch(&owner);
    fresh.set_disabled(&own_sheet, true).unwrap();
    owner.commit(fresh).unwrap();
    let current = owner.snapshot();
    assert!(matches!(
        owner.prepare_commit(stale),
        Err(CssomError::StaleRevision)
    ));
    drop(pending);
    assert_unchanged(&owner, &current);

    let wrong_context = CssomContext {
        inputs: vec![CssomInput {
            version: version(CssomInputRole::Origin, "other", 2),
            data: CssomInputData::Origin {
                origin: "other".into(),
                profile: "author".into(),
            },
        }],
        linked: Vec::new(),
    };
    assert!(matches!(
        owner.batch(CssomEditGuard::for_inputs(&current, wrong_context)),
        Err(CssomError::StaleContext)
    ));
    assert_unchanged(&owner, &current);
}

use surgeist_css::*;
use surgeist_cssom::*;

fn version(role: CssomInputRole, identity: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: identity.into(),
        revision,
    }
}
fn support_context(conditional: bool) -> CssomContext {
    let profile = CssomDeclarationSupport::try_new(
        [
            "margin",
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left",
            "margin-inline-start",
            "width",
            "height",
            "color",
            "background",
            "float",
            "opacity",
        ]
        .iter()
        .map(|name| {
            (
                CssPropertyGrammar::from_name(name).unwrap(),
                if conditional && *name == "background" {
                    CssomUsabilityRequirement::WholeDeclarationDecision
                } else {
                    CssomUsabilityRequirement::AllCheckedValues
                },
            )
        })
        .collect(),
    )
    .unwrap();
    CssomContext {
        inputs: vec![CssomInput {
            version: version(CssomInputRole::Support, "renderer", 1),
            data: CssomInputData::Support(profile),
        }],
        ..CssomContext::default()
    }
}
fn store() -> CssomStore {
    CssomStore::new(CssomLimits::default(), support_context(false)).unwrap()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(
    store: &mut CssomStore,
    text: &str,
    computed: bool,
    readonly: bool,
    owner: bool,
) -> CssomBlockId {
    let mut edit = batch(store);
    let values = parse_declaration_block_contents(text).into_parts().0;
    let ticket = edit
        .create_declarations(
            values,
            computed,
            readonly,
            owner.then(|| version(CssomInputRole::Owner, "element", 1)),
        )
        .unwrap();
    store.commit(edit).unwrap().block(&ticket).unwrap().clone()
}
fn view<'a>(snapshot: &'a CssomSnapshot, id: &CssomBlockId) -> CssomDeclarationView<'a> {
    snapshot
        .declarations(id, CssomDeclarationRequestLimits::default())
        .unwrap()
}
fn set(
    edit: &mut CssomBatch,
    id: &CssomBlockId,
    name: &str,
    value: &str,
    priority: &str,
) -> CssomDeclarationEditResult {
    let prepared = edit
        .prepare_set_property(
            id,
            name,
            value,
            priority,
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap()
}
fn replace(edit: &mut CssomBatch, id: &CssomBlockId, text: &str) {
    let prepared = edit
        .prepare_css_text(id, text, CssomDeclarationRequestLimits::default())
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
}
fn selected<'a>(
    snapshot: &'a CssomSnapshot,
    id: &CssomBlockId,
) -> &'a [CssSpecifiedDeclarationEntry] {
    match snapshot.block(id).unwrap().data() {
        CssomBlockData::Properties {
            selected: CssomProjection::Available(v),
            ..
        } => v.entries(),
        other => panic!("properties: {other:?}"),
    }
}

#[test]
fn pending_replacement_reserves_cleanup_against_owner_declaration_publication() {
    let mut store = CssomStore::new(
        CssomLimits {
            max_revision: 4,
            summary_history: 1,
            ..Default::default()
        },
        support_context(false),
    )
    .unwrap();
    let block = create(&mut store, "width:1px", false, false, true);
    let mut edit = batch(&store);
    let construction = edit
        .construct_sheet(
            CssomConstructorDocument {
                version: version(CssomInputRole::Document, "document", 1),
                identity: "document".into(),
                base_location: String::new(),
            },
            CssomSheetInit::default(),
        )
        .unwrap();
    let sheet = store
        .commit(edit)
        .unwrap()
        .sheet(construction.ticket())
        .unwrap()
        .clone();
    let job = store
        .begin_replace_sheet(
            CssomEditGuard::from_snapshot(&store.snapshot()),
            &sheet,
            "a {}",
        )
        .unwrap();
    let pending = store.snapshot();

    let mut edit = batch(&store);
    set(&mut edit, &block, "unknown-property", "2px", "");
    let unchanged = store.commit(edit).unwrap();
    assert!(matches!(
        unchanged.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert!(unchanged.owner_effects().is_empty());

    let mut edit = batch(&store);
    set(&mut edit, &block, "width", "2px", "");
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::RevisionExhausted)
    ));
    assert_eq!(store.snapshot().revision(), pending.revision());
    assert_eq!(
        view(&store.snapshot(), &block)
            .get_property_value("width")
            .unwrap(),
        "1px"
    );
    assert_eq!(
        store.pending_sheet_replacement(&sheet).unwrap(),
        Some(job.token())
    );

    assert!(matches!(
        store.cancel_replace_sheet(job.token()).unwrap().outcome(),
        CssomReplaceOutcome::Cancelled
    ));
    assert_eq!(store.snapshot().revision().value(), 4);
    assert!(
        !store
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    assert_eq!(
        view(&pending, &block).get_property_value("width").unwrap(),
        "1px"
    );
}

#[test]
fn canonical_getter_physical_priority_and_remove_ignore_logical_only_members() {
    let mut store = store();
    let id = create(
        &mut store,
        "margin:1px 2px 3px 4px!important;margin-inline-start:9px;",
        false,
        false,
        false,
    );
    let before = store.snapshot();
    let read = view(&before, &id);
    assert_eq!(
        read.get_property_value("MARGIN").unwrap(),
        "1px 2px 3px 4px"
    );
    assert_eq!(read.get_property_priority("margin").unwrap(), "important");
    assert_eq!(read.length().unwrap(), 5);
    assert_eq!(read.item(0).unwrap(), "margin-top");
    assert_eq!(read.item(99).unwrap(), "");
    assert_eq!(read.parent_rule(), None);
    let mut edit = batch(&store);
    assert_eq!(
        edit.remove_property(&id, "margin", CssomDeclarationRequestLimits::default())
            .unwrap(),
        "1px 2px 3px 4px"
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        view(&after, &id).css_text().unwrap(),
        "margin-inline-start: 9px;"
    );
    assert_eq!(view(&after, &id).get_property_value("margin").unwrap(), "");
    assert_eq!(
        view(&after, &id).get_property_priority("margin").unwrap(),
        ""
    );
    let mut edit = batch(&store);
    assert_eq!(
        edit.remove_property(&id, "margin", CssomDeclarationRequestLimits::default())
            .unwrap(),
        ""
    );
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
}
#[test]
fn mixed_priority_shorthand_is_empty_but_removal_removes_survivors() {
    let mut store = store();
    let id = create(
        &mut store,
        "margin-top:1px!important;margin-right:2px;--X:var(--a);",
        false,
        false,
        false,
    );
    let before = store.snapshot();
    assert_eq!(view(&before, &id).get_property_value("margin").unwrap(), "");
    assert_eq!(
        view(&before, &id).get_property_priority("margin").unwrap(),
        ""
    );
    let custom = selected(&before, &id)[2].source().clone();
    let mut edit = batch(&store);
    assert_eq!(
        edit.remove_property(&id, "margin", CssomDeclarationRequestLimits::default())
            .unwrap(),
        ""
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert!(selected(&after, &id)[0].source().same_occurrence(&custom));
    assert_eq!(view(&after, &id).css_text().unwrap(), "--X: var(--a);");
}
#[test]
fn setting_existing_physical_target_moves_after_logical_conflicts_only() {
    let mut store = store();
    let id = create(
        &mut store,
        "margin-left:1px;color:red;margin-inline-start:2px;width:3px",
        false,
        false,
        false,
    );
    let before = store.snapshot();
    let color = selected(&before, &id)[1].source().clone();
    let width = selected(&before, &id)[3].source().clone();
    let mut edit = batch(&store);
    assert_eq!(
        set(&mut edit, &id, "margin-left", "8px", ""),
        CssomDeclarationEditResult::Set { updated: true }
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    let read = view(&after, &id);
    assert_eq!(read.item(0).unwrap(), "color");
    assert_eq!(read.item(1).unwrap(), "margin-inline-start");
    assert_eq!(read.item(2).unwrap(), "margin-left");
    assert_eq!(read.item(3).unwrap(), "width");
    assert!(selected(&after, &id)[0].source().same_occurrence(&color));
    assert!(selected(&after, &id)[3].source().same_occurrence(&width));
}
#[test]
fn invalid_priority_and_complete_value_are_noops_with_no_revision_or_effect() {
    let mut store = store();
    let id = create(&mut store, "width:1px", false, false, true);
    let before = store.snapshot();
    let source = selected(&before, &id)[0].source().clone();
    let mut edit = batch(&store);
    for (name, value, priority) in [
        ("width", "2px", " important"),
        ("width", "2px!important", ""),
        ("width", "2px; height:4px", ""),
        ("unknown", "", "garbage"),
    ] {
        assert_eq!(
            set(&mut edit, &id, name, value, priority),
            CssomDeclarationEditResult::Noop
        );
    }
    let commit = store.commit(edit).unwrap();
    assert!(matches!(
        commit.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert!(commit.owner_effects().is_empty());
    assert_eq!(store.snapshot().revision(), before.revision());
    assert!(
        selected(&store.snapshot(), &id)[0]
            .source()
            .same_occurrence(&source)
    );
}
#[test]
fn empty_set_preparation_is_abandonable_and_precedes_invalid_priority() {
    let mut store = store();
    let id = create(&mut store, "width:1px", false, false, false);
    let before = store.snapshot();
    let mut edit = batch(&store);
    drop(
        edit.prepare_set_property(
            &id,
            "width",
            "",
            "garbage",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap(),
    );
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut edit = batch(&store);
    set(&mut edit, &id, "width", "", "garbage");
    store.commit(edit).unwrap();
    assert_eq!(view(&store.snapshot(), &id).length().unwrap(), 0);
}
#[test]
fn equal_value_set_has_new_honest_provenance_without_owner_update() {
    let mut store = store();
    let id = create(&mut store, "width:1px", false, false, true);
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert_eq!(
        set(&mut edit, &id, "width", "1px", ""),
        CssomDeclarationEditResult::Set { updated: false }
    );
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    assert!(matches!(commit.publication(), CssomPublication::Changed(_)));
    let after = store.snapshot();
    let source = selected(&after, &id)[0].source();
    assert!(!source.same_occurrence(selected(&before, &id)[0].source()));
    assert!(source.parsed_name().is_none());
    assert_eq!(source.parsed_value().unwrap().source().as_str(), "1px");
}
#[test]
fn recovery_replaces_and_custom_names_bypass_capability_membership() {
    let mut store = store();
    let id = create(&mut store, "height:9px", false, false, false);
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_css_text(
            &id,
            "width:2px; nope:garbage; --X:var(--x); --x:; } color:red",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    assert!(!prepared.diagnostics().is_empty());
    edit.apply_declaration(prepared, &[]).unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    let read = view(&after, &id);
    assert_eq!(read.get_property_value("width").unwrap(), "2px");
    assert_eq!(read.get_property_value("height").unwrap(), "");
    assert_eq!(read.get_property_value("--X").unwrap(), "var(--x)");
    assert_eq!(read.get_property_value("--x").unwrap(), "");
    assert_eq!(read.get_property_value("color").unwrap(), "");
    assert_eq!(
        selected(&after, &id)[0]
            .source()
            .parsed_name()
            .unwrap()
            .source()
            .as_str(),
        "width:2px; nope:garbage; --X:var(--x); --x:; } color:red"
    );
}
#[test]
fn whole_declaration_decision_drops_compound_as_one_and_retains_input_versions() {
    let mut store = CssomStore::new(CssomLimits::default(), support_context(true)).unwrap();
    let id = create(&mut store, "width:1px", false, false, false);
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_css_text(
            &id,
            "background:linear-gradient(red,blue);width:2px;--x:linear-gradient(red,blue)",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    assert_eq!(prepared.candidates().len(), 1);
    assert_eq!(
        prepared.candidates()[0]
            .declaration()
            .unwrap()
            .known()
            .unwrap()
            .grammar(),
        CssPropertyGrammar::from_name("background").unwrap()
    );
    let decision_version = version(CssomInputRole::Support, "gradient-owner", 3);
    let decision = prepared.candidates()[0]
        .decide(
            decision_version.clone(),
            CssomDeclarationUsability::Unusable,
        )
        .unwrap();
    edit.apply_declaration_with_inputs(
        prepared,
        &[decision],
        std::slice::from_ref(&decision_version),
    )
    .unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        view(&after, &id).get_property_value("background").unwrap(),
        ""
    );
    assert_eq!(
        view(&after, &id).get_property_value("width").unwrap(),
        "2px"
    );
    assert_eq!(
        view(&after, &id).get_property_value("--x").unwrap(),
        "linear-gradient(red,blue)"
    );
    assert!(
        after
            .block(&id)
            .unwrap()
            .admission_inputs()
            .contains(&decision_version)
    );
}
#[test]
fn decision_wrong_revision_and_unresolved_preparation_publish_nothing() {
    let mut store = CssomStore::new(CssomLimits::default(), support_context(true)).unwrap();
    let id = create(&mut store, "width:1px", false, false, false);
    let before = store.snapshot();
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(
            &id,
            "background",
            "linear-gradient(red,blue)",
            "",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::UnresolvedDeclarationPreparation)
    ));
    drop(prepared);
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(
            &id,
            "background",
            "linear-gradient(red,blue)",
            "",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    let decision = prepared.candidates()[0]
        .decide(
            version(CssomInputRole::Support, "gradient-owner", 3),
            CssomDeclarationUsability::Usable,
        )
        .unwrap();
    assert!(matches!(
        edit.apply_declaration_with_inputs(
            prepared,
            &[decision],
            &[version(CssomInputRole::Support, "gradient-owner", 4)]
        ),
        Err(CssomError::StaleDeclarationDecision)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
}
#[test]
fn computed_and_readonly_flags_have_source_branch_precedence() {
    for computed in [false, true] {
        for readonly in [false, true] {
            let mut store = store();
            let id = create(&mut store, "width:1px", computed, readonly, false);
            let before = store.snapshot();
            assert_eq!(
                view(&before, &id).css_text().unwrap(),
                if computed { "" } else { "width: 1px;" }
            );
            assert_eq!(
                view(&before, &id).get_property_value("width").unwrap(),
                "1px"
            );
            let mut edit = batch(&store);
            let result = edit.prepare_set_property(
                &id,
                "width",
                "invalid",
                "",
                CssomDeclarationRequestLimits::default(),
            );
            if readonly {
                assert!(matches!(
                    result,
                    Err(CssomError::Source(CssomException::NoModificationAllowed))
                ));
            } else {
                assert_eq!(
                    edit.apply_declaration(result.unwrap(), &[]).unwrap(),
                    CssomDeclarationEditResult::Noop
                );
            }
            let mut edit = batch(&store);
            let result =
                edit.remove_property(&id, "width", CssomDeclarationRequestLimits::default());
            if readonly {
                assert!(matches!(
                    result,
                    Err(CssomError::Source(CssomException::NoModificationAllowed))
                ));
            } else if computed {
                assert!(matches!(
                    result,
                    Err(CssomError::ComputedStyleUpdatePrecondition)
                ));
                assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
                assert_eq!(store.snapshot().revision(), before.revision());
            } else {
                assert_eq!(result.unwrap(), "1px");
            }
        }
    }
}
#[test]
fn css_float_internal_set_get_and_request_resource_failure_are_atomic() {
    let mut store = store();
    let id = create(&mut store, "width:1px", false, false, false);
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_css_float(&id, "left", CssomDeclarationRequestLimits::default())
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
    store.commit(edit).unwrap();
    assert_eq!(view(&store.snapshot(), &id).css_float().unwrap(), "left");
    let before = store.snapshot();
    let mut edit = batch(&store);
    set(&mut edit, &id, "width", "3px", "");
    let limits = CssomDeclarationRequestLimits {
        max_input_bytes: 1,
        ..Default::default()
    };
    assert!(matches!(
        edit.prepare_css_text(&id, "width:4px", limits),
        Err(CssomError::Limit { .. })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(
        view(&store.snapshot(), &id)
            .get_property_value("width")
            .unwrap(),
        "1px"
    );
}
#[test]
fn effect_lease_marks_updating_before_write_and_exact_echo_only() {
    let mut store = store();
    let id = create(&mut store, "width:1px", false, false, true);
    let mut edit = batch(&store);
    set(&mut edit, &id, "width", "2px", "");
    let mut effects = store.commit(edit).unwrap().take_owner_effects();
    assert_eq!(effects.len(), 1);
    let mut effect = effects.pop().unwrap();
    let revision = store.snapshot().revision().clone();
    let before = store.snapshot();
    {
        let lease = store.begin_owner_application(&mut effect).unwrap();
        assert!(lease.flags().updating);
        assert!(!before.block(&id).unwrap().flags().updating);
        let matching = CssomStyleAttributeChange {
            owner: lease.effect().owner().clone(),
            local_name: "style".into(),
            namespace: None,
            value: Some(lease.effect().value().into()),
            echo: Some(lease.effect().id().clone()),
        };
        assert_eq!(
            lease.classify(&matching),
            CssomOwnerNotification::MatchingEcho
        );
        let mut unrelated = matching.clone();
        unrelated.owner.revision += 1;
        assert_eq!(
            lease.classify(&unrelated),
            CssomOwnerNotification::Unrelated
        );
        unrelated = matching;
        unrelated.echo = None;
        assert_eq!(
            lease.classify(&unrelated),
            CssomOwnerNotification::Unrelated
        );
        lease.finish_success();
    }
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Applied);
    assert!(!store.declaration_flags(&id).unwrap().updating);
    assert_eq!(store.snapshot().revision(), &revision);
    assert!(matches!(
        store.begin_owner_application(&mut effect),
        Err(CssomError::EffectAlreadyConsumed)
    ));
}
#[test]
fn host_failure_unwind_and_abandonment_clear_updating_without_rollback() {
    for failure in 0..3 {
        let mut store = store();
        let id = create(&mut store, "", false, false, true);
        let mut edit = batch(&store);
        replace(&mut edit, &id, "width:2px");
        let mut effect = store
            .commit(edit)
            .unwrap()
            .take_owner_effects()
            .pop()
            .unwrap();
        let revision = store.snapshot().revision().clone();
        if failure == 0 {
            store
                .begin_owner_application(&mut effect)
                .unwrap()
                .finish_host_failure();
            assert_eq!(effect.status(), CssomOwnerEffectStatus::HostFailed);
        } else if failure == 1 {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _lease = store.begin_owner_application(&mut effect).unwrap();
                panic!("host unwind");
            }));
            assert!(result.is_err());
            assert_eq!(effect.status(), CssomOwnerEffectStatus::Abandoned);
        } else {
            drop(store.begin_owner_application(&mut effect).unwrap());
            assert_eq!(effect.status(), CssomOwnerEffectStatus::Abandoned);
        }
        assert!(!store.declaration_flags(&id).unwrap().updating);
        assert_eq!(store.snapshot().revision(), &revision);
        assert_eq!(
            view(&store.snapshot(), &id)
                .get_property_value("width")
                .unwrap(),
            "2px"
        );
    }
}
#[test]
fn equal_empty_css_text_requires_effect_without_css_revision_after_admission() {
    let mut store = store();
    let id = create(&mut store, "", false, false, true);
    let mut edit = batch(&store);
    replace(&mut edit, &id, "");
    drop(store.commit(edit).unwrap().take_owner_effects());
    let before = store.snapshot();
    let mut edit = batch(&store);
    replace(&mut edit, &id, "");
    let mut commit = store.commit(edit).unwrap();
    assert!(matches!(
        commit.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(commit.owner_effects().len(), 1);
    let mut effect = commit.take_owner_effects().pop().unwrap();
    effect.cancel().unwrap();
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Canceled);
    assert_eq!(store.snapshot().revision(), before.revision());
}
#[test]
fn stale_effect_and_pending_quota_failure_preserve_committed_css() {
    let limits = CssomLimits {
        max_pending_owner_effects: 1,
        ..Default::default()
    };
    let mut store = CssomStore::new(limits, support_context(false)).unwrap();
    let id = create(&mut store, "", false, false, true);
    let mut edit = batch(&store);
    replace(&mut edit, &id, "width:1px");
    let mut effect = store
        .commit(edit)
        .unwrap()
        .take_owner_effects()
        .pop()
        .unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    replace(&mut edit, &id, "width:2px");
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::Limit {
            resource: "pending owner effects",
            ..
        })
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    effect.cancel().unwrap();
    let mut edit = batch(&store);
    replace(&mut edit, &id, "width:3px");
    let mut stale = store
        .commit(edit)
        .unwrap()
        .take_owner_effects()
        .pop()
        .unwrap();
    let mut edit = batch(&store);
    set(&mut edit, &id, "width", "3px", "");
    drop(store.commit(edit).unwrap());
    assert!(matches!(
        store.begin_owner_application(&mut stale),
        Err(CssomError::StaleOwnerEffect)
    ));
}

fn sheet(store: &mut CssomStore, text: &str) -> CssomSheetId {
    let mut edit = batch(store);
    let inputs = CssomSheetInputs::constructed(
        version(CssomInputRole::Document, "document", 1),
        "document".into(),
        None,
    );
    let ticket = edit.create_parsed_sheet(text, inputs).unwrap();
    store.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn rule_block(snapshot: &CssomSnapshot, rule: &CssomRuleId) -> CssomBlockId {
    snapshot.rule(rule).unwrap().data().block().unwrap().clone()
}
#[test]
fn keyframe_and_margin_source_domains_reject_invalid_members_preserve_ids_and_parent() {
    let mut store = store();
    let sheet = sheet(
        &mut store,
        "@keyframes demo {from {opacity:0}} @page {margin:1px;@top-left {width:2px}} ",
    );
    let before = store.snapshot();
    let roots = before.sheet_rules(&sheet).unwrap();
    let frames = before
        .rules(before.rule(&roots[0]).unwrap().data().children().unwrap())
        .unwrap()
        .to_vec();
    let margins = before
        .rules(before.rule(&roots[1]).unwrap().data().children().unwrap())
        .unwrap()
        .to_vec();
    let frame = rule_block(&before, &frames[0]);
    let margin = rule_block(&before, &margins[0]);
    assert_eq!(view(&before, &frame).parent_rule(), Some(&frames[0]));
    assert_eq!(view(&before, &margin).parent_rule(), Some(&margins[0]));
    let mut edit = batch(&store);
    assert_eq!(
        set(&mut edit, &frame, "opacity", ".5", "important"),
        CssomDeclarationEditResult::Noop
    );
    replace(
        &mut edit,
        &frame,
        "opacity:.5!important;width:4px;opacity:.25;",
    );
    replace(&mut edit, &margin, "width:5px;opacity:.5;--x:var(--a)");
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        view(&after, &frame).css_text().unwrap(),
        "width: 4px; opacity: 0.25;"
    );
    assert_eq!(
        view(&after, &margin).get_property_value("width").unwrap(),
        "5px"
    );
    assert_eq!(
        view(&after, &margin).get_property_value("opacity").unwrap(),
        ""
    );
    assert_eq!(
        view(&after, &margin).get_property_value("--x").unwrap(),
        "var(--a)"
    );
    assert_eq!(
        after
            .rules(after.rule(&roots[1]).unwrap().data().children().unwrap())
            .unwrap(),
        margins
    );
    assert_eq!(
        after
            .rules(after.rule(&roots[0]).unwrap().data().children().unwrap())
            .unwrap(),
        frames
    );
}
#[test]
fn page_property_edits_preserve_interspersed_descriptor_slots_and_margin_children() {
    let mut store = store();
    let sheet = sheet(
        &mut store,
        "@page {width:1px;size:a4;color:red;marks:crop;height:2px;@top-left {width:3px}} ",
    );
    let before = store.snapshot();
    let page = before.sheet_rules(&sheet).unwrap()[0].clone();
    let id = rule_block(&before, &page);
    let children = before
        .rules(before.rule(&page).unwrap().data().children().unwrap())
        .unwrap()
        .to_vec();
    let mut edit = batch(&store);
    set(&mut edit, &id, "width", "8px", "");
    edit.remove_property(&id, "color", CssomDeclarationRequestLimits::default())
        .unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    let read = view(&after, &id);
    assert_eq!(
        (0..read.length().unwrap())
            .map(|i| read.item(i).unwrap())
            .collect::<Vec<_>>(),
        ["width", "size", "marks", "height"]
    );
    assert_eq!(read.get_property_value("width").unwrap(), "8px");
    assert_eq!(read.get_property_value("size").unwrap(), "a4");
    assert_eq!(
        after
            .rules(after.rule(&page).unwrap().data().children().unwrap())
            .unwrap(),
        children
    );
}
#[test]
fn inline_external_attribute_changes_bypass_readonly_without_echoing_owner() {
    let mut store = store();
    let id = create(&mut store, "width:1px", false, true, true);
    let mut edit = batch(&store);
    let change = CssomStyleAttributeChange {
        owner: version(CssomInputRole::Owner, "element", 1),
        local_name: "style".into(),
        namespace: None,
        value: Some("width:4px;nope:bad".into()),
        echo: None,
    };
    let prepared = edit
        .prepare_style_attribute_change(&id, &change, CssomDeclarationRequestLimits::default())
        .unwrap()
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    assert_eq!(
        view(&store.snapshot(), &id)
            .get_property_value("width")
            .unwrap(),
        "4px"
    );
    let mut edit = batch(&store);
    let mut null = change.clone();
    null.value = None;
    let prepared = edit
        .prepare_style_attribute_change(&id, &null, CssomDeclarationRequestLimits::default())
        .unwrap()
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
    store.commit(edit).unwrap();
    assert_eq!(view(&store.snapshot(), &id).length().unwrap(), 0);
}
#[test]
fn nonqualifying_inline_notifications_do_not_parse_and_stale_owner_is_atomic() {
    let mut store = store();
    let id = create(&mut store, "width:1px", false, false, true);
    let before = store.snapshot();
    for (local, namespace) in [("STYLE", None), ("style", Some("ns"))] {
        let mut edit = batch(&store);
        let change = CssomStyleAttributeChange {
            owner: version(CssomInputRole::Owner, "element", 99),
            local_name: local.into(),
            namespace: namespace.map(str::to_owned),
            value: Some("enormous-invalid".into()),
            echo: None,
        };
        assert!(
            edit.prepare_style_attribute_change(
                &id,
                &change,
                CssomDeclarationRequestLimits {
                    max_input_bytes: 0,
                    ..Default::default()
                }
            )
            .unwrap()
            .is_none()
        );
        assert!(matches!(
            store.commit(edit).unwrap().publication(),
            CssomPublication::Unchanged { .. }
        ));
    }
    let mut edit = batch(&store);
    let change = CssomStyleAttributeChange {
        owner: version(CssomInputRole::Owner, "element", 2),
        local_name: "style".into(),
        namespace: None,
        value: None,
        echo: None,
    };
    assert!(matches!(
        edit.prepare_style_attribute_change(&id, &change, CssomDeclarationRequestLimits::default()),
        Err(CssomError::StaleOwnerNotification)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
}
#[test]
fn final_effects_use_last_operation_slot_and_final_text() {
    let mut store = store();
    let a = create(&mut store, "", false, false, true);
    let b = create(&mut store, "", false, false, true);
    let mut edit = batch(&store);
    set(&mut edit, &a, "width", "1px", "");
    set(&mut edit, &b, "height", "2px", "");
    set(&mut edit, &a, "width", "3px", "");
    let effects = store.commit(edit).unwrap().take_owner_effects();
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[0].block(), &b);
    assert_eq!(effects[1].block(), &a);
    assert_eq!(effects[0].value(), "height: 2px;");
    assert_eq!(effects[1].value(), "width: 3px;");
}
#[test]
fn effect_identity_and_byte_exhaustion_have_real_limits_and_no_publication() {
    for limits in [
        CssomLimits {
            max_owner_effect_identity: 0,
            ..Default::default()
        },
        CssomLimits {
            max_pending_owner_effect_bytes: 1,
            ..Default::default()
        },
    ] {
        let mut store = CssomStore::new(limits, support_context(false)).unwrap();
        let id = create(&mut store, "", false, false, true);
        let before = store.snapshot();
        let mut edit = batch(&store);
        replace(&mut edit, &id, "width:1px");
        let result = store.commit(edit);
        assert!(matches!(
            result,
            Err(CssomError::EffectIdentityExhausted)
                | Err(CssomError::Limit {
                    resource: "pending owner effect bytes",
                    ..
                })
        ));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(view(&store.snapshot(), &id).length().unwrap(), 0);
    }
}
#[test]
fn read_request_limits_and_parser_resources_remain_typed() {
    let mut store = store();
    let id = create(&mut store, "width:1px", false, false, false);
    let before = store.snapshot();
    let read = before
        .declarations(
            &id,
            CssomDeclarationRequestLimits {
                max_entries: 0,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(matches!(read.length(), Err(CssomError::Limit { .. })));
    assert!(matches!(read.item(0), Err(CssomError::Limit { .. })));
    let mut edit = batch(&store);
    let limits = CssomDeclarationRequestLimits {
        max_input_components: 0,
        ..Default::default()
    };
    assert!(matches!(
        edit.prepare_css_text(&id, "width:2px", limits),
        Err(CssomError::ParseResource(_))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
}

#[test]
fn custom_queries_compare_semantic_supplied_names_without_authored_escape_aliases() {
    let mut store = store();
    let id = create(
        &mut store,
        "--\\78:one;--a\\ b:two;--X:three",
        false,
        false,
        false,
    );
    let before = store.snapshot();
    let read = view(&before, &id);
    assert_eq!(read.get_property_value("--x").unwrap(), "one");
    assert_eq!(read.get_property_value("--\\78").unwrap(), "");
    assert_eq!(read.get_property_value("--a b").unwrap(), "two");
    assert_eq!(read.get_property_value("--X").unwrap(), "three");
    let mut edit = batch(&store);
    assert_eq!(
        edit.remove_property(&id, "--\\78", CssomDeclarationRequestLimits::default())
            .unwrap(),
        ""
    );
    assert_eq!(
        edit.remove_property(&id, "--a b", CssomDeclarationRequestLimits::default())
            .unwrap(),
        "two"
    );
    set(&mut edit, &id, "--x", "four", "");
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(view(&after, &id).get_property_value("--x").unwrap(), "four");
    assert_eq!(view(&after, &id).get_property_value("--a b").unwrap(), "");
    assert_eq!(
        view(&after, &id).get_property_value("--X").unwrap(),
        "three"
    );
}

#[test]
fn independent_svg_terminal_has_supplied_support_and_uses_existing_metadata_identity() {
    let parser = CssParserContext::default().with_svg_glyph_orientation_vertical();
    let profile = CssomDeclarationSupport::try_new(vec![])
        .unwrap()
        .with_svg_glyph_orientation_vertical(CssomUsabilityRequirement::AllCheckedValues);
    let context = CssomContext {
        inputs: vec![
            CssomInput {
                version: version(CssomInputRole::Support, "svg-renderer", 1),
                data: CssomInputData::Support(profile),
            },
            CssomInput {
                version: version(CssomInputRole::ParserMode, "definition", 1),
                data: CssomInputData::ParserMode(parser),
            },
        ],
        ..Default::default()
    };
    let mut store = CssomStore::new(CssomLimits::default(), context).unwrap();
    let mut edit = batch(&store);
    let declarations = parser
        .parse_declaration_block_contents("glyph-orientation-vertical:45deg")
        .into_parts()
        .0;
    let ticket = edit
        .create_declarations(declarations, false, false, None)
        .unwrap();
    let id = store.commit(edit).unwrap().block(&ticket).unwrap().clone();
    let before = store.snapshot();
    assert_eq!(
        view(&before, &id)
            .get_property_value("GLYPH-ORIENTATION-VERTICAL")
            .unwrap(),
        "45deg"
    );
    assert_eq!(
        selected(&before, &id)[0].property_name(),
        CssSvgGlyphOrientationVerticalDeclaration::metadata().property()
    );
    let mut edit = batch(&store);
    assert_eq!(
        set(
            &mut edit,
            &id,
            "glyph-orientation-vertical",
            "60deg",
            "important"
        ),
        CssomDeclarationEditResult::Set { updated: true }
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        view(&after, &id)
            .get_property_priority("glyph-orientation-vertical")
            .unwrap(),
        "important"
    );
    assert_eq!(
        view(&after, &id)
            .get_property_value("glyph-orientation-vertical")
            .unwrap(),
        "60deg"
    );
    let mut edit = batch(&store);
    assert_eq!(
        edit.remove_property(
            &id,
            "glyph-orientation-vertical",
            CssomDeclarationRequestLimits::default()
        )
        .unwrap(),
        "60deg"
    );
    store.commit(edit).unwrap();
    assert_eq!(view(&store.snapshot(), &id).length().unwrap(), 0);
}

fn descriptor_store(conditional: bool) -> CssomStore {
    let mut context = support_context(false);
    let CssomInputData::Support(profile) = &mut context.inputs[0].data else {
        unreachable!()
    };
    *profile = profile
        .clone()
        .with_page_descriptors(
            ["size", "marks", "bleed", "page-orientation"]
                .map(|name| {
                    (
                        CssPageDescriptorKind::from_name(name).unwrap(),
                        if conditional && name == "size" {
                            CssomUsabilityRequirement::WholeDeclarationDecision
                        } else {
                            CssomUsabilityRequirement::AllCheckedValues
                        },
                    )
                })
                .to_vec(),
        )
        .unwrap()
        .with_font_face_descriptors(
            [
                "font-family",
                "src",
                "font-weight",
                "font-width",
                "font-display",
            ]
            .map(|name| {
                (
                    CssFontFaceDescriptorKind::from_css_name(name).unwrap(),
                    if conditional && name == "src" {
                        CssomUsabilityRequirement::WholeDeclarationDecision
                    } else {
                        CssomUsabilityRequirement::AllCheckedValues
                    },
                )
            })
            .to_vec(),
        )
        .unwrap();
    CssomStore::new(CssomLimits::default(), context).unwrap()
}
#[test]
fn page_descriptor_names_use_page_owner_and_keep_children_and_properties() {
    let mut store = descriptor_store(false);
    let sheet = sheet(
        &mut store,
        "@page {width:2px;height:3px;size:A4!important;marks:crop;@top-left{width:1px}} ",
    );
    let before = store.snapshot();
    let rule = before.sheet_rules(&sheet).unwrap()[0].clone();
    let id = rule_block(&before, &rule);
    let children = before
        .rules(before.rule(&rule).unwrap().data().children().unwrap())
        .unwrap()
        .to_vec();
    assert_eq!(view(&before, &id).get_property_value("SIZE").unwrap(), "a4");
    assert_eq!(
        view(&before, &id).get_property_priority("size").unwrap(),
        "important"
    );
    let mut edit = batch(&store);
    assert_eq!(
        edit.remove_property(&id, "SiZe", Default::default())
            .unwrap(),
        "a4"
    );
    store.commit(edit).unwrap();
    let removed = store.snapshot();
    // Ordinary width/height cannot masquerade as this missing Page size descriptor.
    assert_eq!(view(&removed, &id).get_property_value("size").unwrap(), "");
    assert_eq!(
        view(&removed, &id).get_property_value("width").unwrap(),
        "2px"
    );
    let mut edit = batch(&store);
    assert_eq!(
        set(&mut edit, &id, "size", "letter landscape", "important"),
        CssomDeclarationEditResult::Set { updated: true }
    );
    assert_eq!(
        set(&mut edit, &id, "bleed", "1px !important", ""),
        CssomDeclarationEditResult::Noop
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        view(&after, &id).get_property_value("size").unwrap(),
        "letter landscape"
    );
    assert_eq!(
        view(&after, &id).get_property_priority("size").unwrap(),
        "important"
    );
    assert_eq!(
        after
            .rules(after.rule(&rule).unwrap().data().children().unwrap())
            .unwrap(),
        children
    );
    assert_eq!(view(&before, &id).get_property_value("size").unwrap(), "a4");
}
#[test]
fn font_face_current_selection_retains_raw_occurrences_aliases_and_domain_noops() {
    let mut store = descriptor_store(false);
    let sheet = sheet(
        &mut store,
        "@font-face{font-weight:100;src:local(A);font-weight:200;font-stretch:condensed}",
    );
    let before = store.snapshot();
    let rule = before.sheet_rules(&sheet).unwrap()[0].clone();
    let id = rule_block(&before, &rule);
    assert_eq!(view(&before, &id).length().unwrap(), 3);
    assert_eq!(view(&before, &id).item(0).unwrap(), "src");
    assert_eq!(view(&before, &id).item(1).unwrap(), "font-weight");
    assert_eq!(
        view(&before, &id)
            .get_property_value("FONT-STRETCH")
            .unwrap(),
        "condensed"
    );
    let CssomBlockData::FontFace(raw) = before.block(&id).unwrap().data() else {
        panic!("FontFace")
    };
    assert_eq!(raw.occurrences().len(), 4);
    assert!(raw.occurrences().all(|value| value.position().is_some()));
    let mut edit = batch(&store);
    assert_eq!(
        set(&mut edit, &id, "font-weight", "300", "important"),
        CssomDeclarationEditResult::Noop
    );
    assert_eq!(
        set(&mut edit, &id, "width", "2px", ""),
        CssomDeclarationEditResult::Noop
    );
    assert_eq!(
        set(&mut edit, &id, "font-weight", "300", ""),
        CssomDeclarationEditResult::Set { updated: true }
    );
    assert_eq!(
        edit.remove_property(&id, "font-stretch", Default::default())
            .unwrap(),
        "condensed"
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        view(&after, &id).css_text().unwrap(),
        "src: local(\"A\"); font-weight: 300;"
    );
    assert_eq!(
        view(&after, &id)
            .get_property_priority("font-weight")
            .unwrap(),
        ""
    );
    let selected = after.block(&id).unwrap().selected_font_face().unwrap();
    let CssomProjection::Available(selected) = selected else {
        panic!("selected")
    };
    assert!(selected.entries()[0].position().is_some());
    assert!(selected.entries()[1].position().is_none());
    assert_eq!(
        view(&before, &id)
            .get_property_value("font-weight")
            .unwrap(),
        "200"
    );
}
#[test]
fn descriptor_whole_decisions_are_correlated_and_recovery_keeps_real_raw_input() {
    let mut store = descriptor_store(true);
    let sheet = sheet(
        &mut store,
        "@font-face{font-family:A;src:local(A)} @page{size:A4}",
    );
    let snapshot = store.snapshot();
    let rules = snapshot.sheet_rules(&sheet).unwrap();
    let font = rule_block(&snapshot, &rules[0]);
    let page = rule_block(&snapshot, &rules[1]);
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_css_text(
            &font,
            "font-family:B;src:local(B), invalid(), url(font.woff);font-weight:400!important;",
            Default::default(),
        )
        .unwrap();
    assert_eq!(prepared.candidates().len(), 1);
    assert!(matches!(
        prepared.candidates()[0].value(),
        CssomDeclarationValue::FontFaceDescriptor(_)
    ));
    assert!(!prepared.diagnostics().is_empty());
    let decision = prepared.candidates()[0]
        .decide(
            prepared.profile_version().unwrap().clone(),
            CssomDeclarationUsability::Unusable,
        )
        .unwrap();
    edit.apply_declaration(prepared, &[decision]).unwrap();
    let prepared = edit
        .prepare_set_property(&page, "size", "letter", "", Default::default())
        .unwrap();
    assert!(matches!(
        prepared.candidates()[0].value(),
        CssomDeclarationValue::PageDescriptor(_)
    ));
    let decision = prepared.candidates()[0]
        .decide(
            prepared.profile_version().unwrap().clone(),
            CssomDeclarationUsability::Usable,
        )
        .unwrap();
    edit.apply_declaration(prepared, &[decision]).unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(view(&after, &font).css_text().unwrap(), "font-family: B;");
    assert_eq!(
        view(&after, &page).get_property_value("size").unwrap(),
        "letter"
    );
    let input = after.block(&font).unwrap().replacement_input().unwrap();
    assert!(input.source.contains("invalid()"));
    assert!(!input.diagnostics.is_empty());
    assert_eq!(input.parser_context, CssParserContext::default());
    let mut edit = batch(&store);
    replace(&mut edit, &font, "font-family:;width:2px;}");
    store.commit(edit).unwrap();
    let empty = store.snapshot();
    drop(store);
    assert_eq!(view(&empty, &font).css_text().unwrap(), "");
    assert_eq!(
        empty
            .block(&font)
            .unwrap()
            .replacement_input()
            .unwrap()
            .source,
        "font-family:;width:2px;}"
    );
    assert!(
        !empty
            .block(&font)
            .unwrap()
            .replacement_input()
            .unwrap()
            .diagnostics
            .is_empty()
    );
    assert_eq!(view(&after, &font).css_text().unwrap(), "font-family: B;");
}

#[test]
fn supplied_custom_names_keep_literal_characters_and_nul_identity_across_edits() {
    let mut store = store();
    let id = create(&mut store, "--x:old", false, false, false);
    let mut edit = batch(&store);
    for name in ["--bad name", r"--\78", "--a:b;c{}", "--a\0Z", "--a�Z"] {
        assert_eq!(
            set(&mut edit, &id, name, "MiXeD", "important"),
            CssomDeclarationEditResult::Set { updated: true }
        );
    }
    store.commit(edit).unwrap();
    let before = store.snapshot();
    for name in ["--bad name", r"--\78", "--a:b;c{}", "--a\0Z", "--a�Z"] {
        assert_eq!(
            view(&before, &id).get_property_value(name).unwrap(),
            "MiXeD"
        );
        assert_eq!(
            view(&before, &id).get_property_priority(name).unwrap(),
            "important"
        );
    }
    assert_eq!(view(&before, &id).get_property_value("--x").unwrap(), "old");
    assert_eq!(view(&before, &id).item(4).unwrap().as_bytes(), b"--a\0Z");
    let mut edit = batch(&store);
    assert_eq!(
        edit.remove_property(&id, "--a\0Z", Default::default())
            .unwrap(),
        "MiXeD"
    );
    assert_eq!(
        edit.remove_property(&id, r"--\78", Default::default())
            .unwrap(),
        "MiXeD"
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    drop(store);
    assert_eq!(view(&after, &id).get_property_value("--a\0Z").unwrap(), "");
    assert_eq!(
        view(&after, &id).get_property_value("--a�Z").unwrap(),
        "MiXeD"
    );
    assert_eq!(view(&after, &id).get_property_value("--x").unwrap(), "old");
    assert_eq!(
        view(&before, &id).get_property_value(r"--\78").unwrap(),
        "MiXeD"
    );
}
#[test]
fn descriptor_support_is_supplied_not_inferred_and_resource_failures_abort_whole_batch() {
    let mut unsupported = store();
    let unsupported_sheet = sheet(
        &mut unsupported,
        "@page{size:A4} @font-face{font-family:A;src:local(A)}",
    );
    let before = unsupported.snapshot();
    let rules = before.sheet_rules(&unsupported_sheet).unwrap();
    let page = rule_block(&before, &rules[0]);
    let font = rule_block(&before, &rules[1]);
    let mut edit = batch(&unsupported);
    assert_eq!(
        set(&mut edit, &page, "size", "", "invalid"),
        CssomDeclarationEditResult::Noop
    );
    assert_eq!(
        set(&mut edit, &font, "font-family", "B", ""),
        CssomDeclarationEditResult::Noop
    );
    assert!(matches!(
        unsupported.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    let mut edit = batch(&unsupported);
    replace(&mut edit, &page, "size:letter;width:2px;--custom:kept");
    replace(&mut edit, &font, "font-family:B;src:local(B)");
    unsupported.commit(edit).unwrap();
    let after = unsupported.snapshot();
    assert_eq!(view(&after, &page).get_property_value("size").unwrap(), "");
    assert_eq!(
        view(&after, &page).get_property_value("--custom").unwrap(),
        "kept"
    );
    assert_eq!(view(&after, &font).css_text().unwrap(), "");
    let mut supported = descriptor_store(false);
    let sheet = sheet(&mut supported, "@page{size:A4} @font-face{font-family:A}");
    let before = supported.snapshot();
    let rules = before.sheet_rules(&sheet).unwrap();
    let page = rule_block(&before, &rules[0]);
    let font = rule_block(&before, &rules[1]);
    let mut edit = batch(&supported);
    set(&mut edit, &page, "size", "letter", "");
    let prepared = edit
        .prepare_set_property(
            &font,
            "font-family",
            "B",
            "",
            CssomDeclarationRequestLimits {
                provider: CssSpecifiedValueSerializationLimits::new(1000, 1000, 0),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(matches!(
        edit.apply_declaration(prepared, &[]),
        Err(CssomError::FontFace(
            CssFontFaceDeclarationBlockError::Serialization(_)
        ))
    ));
    assert!(matches!(
        supported.commit(edit),
        Err(CssomError::BatchAborted)
    ));
    let after = supported.snapshot();
    assert_eq!(after.revision(), before.revision());
    assert_eq!(
        view(&after, &page).get_property_value("size").unwrap(),
        "a4"
    );
    assert_eq!(
        view(&after, &font)
            .get_property_value("font-family")
            .unwrap(),
        "A"
    );
}

#[test]
fn independent_decision_input_cannot_relabel_a_stale_membership_owner() {
    let mut store = CssomStore::new(CssomLimits::default(), support_context(true)).unwrap();
    let id = create(&mut store, "width:1px", false, false, false);
    let before = store.snapshot();
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(
            &id,
            "background",
            "linear-gradient(red,blue)",
            "",
            Default::default(),
        )
        .unwrap();
    let wrong = version(CssomInputRole::Support, "renderer", 2);
    let decision = prepared.candidates()[0]
        .decide(wrong.clone(), CssomDeclarationUsability::Usable)
        .unwrap();
    assert!(matches!(
        edit.apply_declaration_with_inputs(prepared, &[decision], &[wrong]),
        Err(CssomError::StaleDeclarationDecision)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
}

fn malformed_complete_value_is_noop(value: &str, domain: usize) {
    let mut live = descriptor_store(false);
    let ordinary = create(&mut live, "width:1px", false, false, true);
    let sheet = sheet(&mut live, "@page{size:A4} @font-face{font-family:A}");
    let before = live.snapshot();
    let rules = before.sheet_rules(&sheet).unwrap().to_vec();
    let page = rule_block(&before, &rules[0]);
    let font = rule_block(&before, &rules[1]);
    for (id, name, valid, expected) in [
        (&ordinary, "width", "2px", "2px"),
        (&page, "size", "letter", "letter"),
        (&font, "font-family", "B", "B"),
    ]
    .into_iter()
    .skip(domain)
    .take(1)
    {
        let original = live.snapshot();
        let mut edit = batch(&live);
        set(&mut edit, id, name, valid, "");
        let prepared = edit
            .prepare_set_property(id, name, value, "", Default::default())
            .unwrap_or_else(|error| {
                panic!("{name}={value:?} must be a successful null-parse no-op: {error:?}")
            });
        assert!(prepared.candidates().is_empty());
        assert_eq!(
            edit.apply_declaration(prepared, &[]).unwrap(),
            CssomDeclarationEditResult::Noop
        );
        let commit = live.commit(edit).unwrap();
        let after = live.snapshot();
        assert_eq!(view(&after, id).get_property_value(name).unwrap(), expected);
        assert_eq!(after.sheet_rules(&sheet).unwrap(), rules);
        assert_ne!(after.revision(), original.revision());
        assert_eq!(commit.owner_effects().len(), usize::from(id == &ordinary));
        assert_eq!(
            view(&original, id).get_property_value(name).unwrap(),
            if id == &ordinary {
                "1px"
            } else if id == &page {
                "a4"
            } else {
                "A"
            }
        );
    }
}
#[test]
fn unmatched_complete_value_is_noop_across_property_and_descriptor_domains() {
    malformed_complete_value_is_noop("]", 0);
}
#[test]
fn bad_string_complete_value_is_noop_across_property_and_descriptor_domains() {
    malformed_complete_value_is_noop("\"bad\nstring\"", 0);
}
#[test]
fn bad_url_complete_value_is_noop_across_property_and_descriptor_domains() {
    malformed_complete_value_is_noop("url(bad font)", 0);
}
#[test]
fn complete_value_component_resource_failure_remains_atomic_error() {
    let mut live = descriptor_store(false);
    let ordinary = create(&mut live, "width:1px", false, false, false);
    let sheet = sheet(&mut live, "@page{size:A4} @font-face{font-family:A}");
    let before = live.snapshot();
    let rules = before.sheet_rules(&sheet).unwrap();
    let page = rule_block(&before, &rules[0]);
    let font = rule_block(&before, &rules[1]);
    for (id, name, value) in [
        (&ordinary, "width", "2px"),
        (&page, "size", "letter"),
        (&font, "font-family", "B"),
    ] {
        let mut edit = batch(&live);
        set(&mut edit, &ordinary, "width", "9px", "");
        let result = edit.prepare_set_property(
            id,
            name,
            value,
            "",
            CssomDeclarationRequestLimits {
                max_input_components: 0,
                ..Default::default()
            },
        );
        assert!(
            matches!(result, Err(CssomError::Component(ref error)) if error.kind() == CssComponentValueErrorKind::ComponentLimit)
        );
        assert!(matches!(live.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(live.snapshot().revision(), before.revision());
        assert_eq!(
            view(&live.snapshot(), &ordinary)
                .get_property_value("width")
                .unwrap(),
            "1px"
        );
    }
}

#[test]
fn page_unmatched_complete_value_is_noop() {
    malformed_complete_value_is_noop("]", 1);
}

#[test]
fn page_bad_string_complete_value_is_noop() {
    malformed_complete_value_is_noop("\"bad\nstring\"", 1);
}

#[test]
fn page_bad_url_complete_value_is_noop() {
    malformed_complete_value_is_noop("url(bad font)", 1);
}

#[test]
fn font_face_unmatched_complete_value_is_noop() {
    malformed_complete_value_is_noop("]", 2);
}

#[test]
fn font_face_bad_string_complete_value_is_noop() {
    malformed_complete_value_is_noop("\"bad\nstring\"", 2);
}

#[test]
fn font_face_bad_url_complete_value_is_noop() {
    malformed_complete_value_is_noop("url(bad font)", 2);
}

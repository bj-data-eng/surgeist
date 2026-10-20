use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use surgeist_css::{
    CssErrorCode, CssPropertyGrammar, CssRecoveryAction, parse_declaration_block_contents,
};
use surgeist_cssom::*;

fn version(identity: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Support,
        identity: identity.into(),
        revision,
    }
}
fn context(conditional: bool) -> CssomContext {
    let profile = CssomDeclarationSupport::try_new(vec![
        (
            CssPropertyGrammar::from_name("width").unwrap(),
            if conditional {
                CssomUsabilityRequirement::WholeDeclarationDecision
            } else {
                CssomUsabilityRequirement::AllCheckedValues
            },
        ),
        (
            CssPropertyGrammar::from_name("height").unwrap(),
            CssomUsabilityRequirement::AllCheckedValues,
        ),
    ])
    .unwrap();
    CssomContext {
        inputs: vec![CssomInput {
            version: version("renderer", 1),
            data: CssomInputData::Support(profile),
        }],
        ..Default::default()
    }
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(store: &mut CssomStore, text: &str, owner: bool) -> CssStyleDeclaration {
    let mut edit = batch(store);
    let ticket = edit
        .create_declarations(
            parse_declaration_block_contents(text).into_parts().0,
            false,
            false,
            owner.then(|| CssomInputVersion {
                role: CssomInputRole::Owner,
                identity: "element".into(),
                revision: 1,
            }),
        )
        .unwrap();
    let commit = store.commit(edit).unwrap();
    store
        .css_style_declaration(commit.block(&ticket).unwrap())
        .unwrap()
}
fn text(snapshot: &CssomSnapshot, object: &CssStyleDeclaration) -> String {
    match snapshot.block(object.id()).unwrap().data() {
        CssomBlockData::Properties {
            selected: CssomProjection::Available(value),
            ..
        } => value.serialize_cssom().unwrap(),
        other => panic!("current selected properties: {other:?}"),
    }
}
fn admission(
    request: &CssomAdmissionRequest<'_>,
    usability: CssomDeclarationUsability,
) -> CssomAdmission {
    let decision_version = version("values-owner", 1);
    let decisions = request
        .candidates()
        .iter()
        .map(|candidate| {
            candidate
                .decide(decision_version.clone(), usability)
                .unwrap()
        })
        .collect();
    let mut current = request
        .context()
        .inputs
        .iter()
        .map(|input| input.version.clone())
        .collect::<Vec<_>>();
    current.push(decision_version);
    CssomAdmission::new(decisions, current)
}

#[test]
fn each_object_call_publishes_once_and_remove_returns_the_standard_old_string() {
    let mut store = CssomStore::new(CssomLimits::default(), context(false)).unwrap();
    let object = create(&mut store, "width:1px;height:2px", false);
    let before = store.snapshot();
    let capture = object.capture(&before).unwrap();
    let result = object
        .remove_property(
            &mut store,
            "width",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    assert_eq!(result.value(), "1px");
    assert!(matches!(result.publication(), CssomPublication::Changed(_)));
    assert_eq!(
        store.snapshot().revision().value(),
        before.revision().value() + 1
    );
    assert_eq!(text(&store.snapshot(), &object), "height: 2px;");
    assert_eq!(text(capture.snapshot(), capture.handle()), "size: 1px 2px;");
    let after = store.snapshot();
    let noop = object
        .remove_property(
            &mut store,
            "width",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    assert_eq!(noop.value(), "");
    assert!(matches!(
        noop.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), after.revision());
    assert!(noop.owner_effects().is_empty());
}

#[test]
fn resolver_is_skipped_for_checked_custom_and_source_noop_values() {
    let mut store = CssomStore::new(CssomLimits::default(), context(false)).unwrap();
    let object = create(&mut store, "width:1px", false);
    for (name, value, priority) in [
        ("width", "2px", None),
        ("--Case", "RAW", None),
        ("missing-property", "3px", None),
        ("width", "4px", Some("invalid")),
    ] {
        let result = object
            .set_property(
                &mut store,
                name,
                value,
                priority,
                CssomDeclarationRequestLimits::default(),
                |_| panic!("unneeded resolver must not run"),
            )
            .unwrap();
        assert!(result.diagnostics().is_empty());
    }
    assert_eq!(text(&store.snapshot(), &object), "width: 2px; --Case: RAW;");
}

#[test]
fn recovered_setter_preserves_each_native_zero_width_eof_diagnostic() {
    let mut store = CssomStore::new(CssomLimits::default(), context(false)).unwrap();
    let object = create(&mut store, "", false);
    let outcome = object
        .set_property(
            &mut store,
            "--x",
            "f([",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| panic!("custom values need no resolver"),
        )
        .unwrap();
    assert_eq!(outcome.diagnostics().len(), 2);
    for diagnostic in outcome.diagnostics() {
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(diagnostic.error().position().byte_offset().value(), 3);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 3);
        assert_eq!(diagnostic.span().end().byte_offset().value(), 3);
    }
    assert_eq!(text(&store.snapshot(), &object), "--x: f([]);");
}

#[test]
fn commit_effect_exhaustion_is_prepublication_and_drop_allows_retry() {
    let mut store = CssomStore::new(
        CssomLimits {
            max_pending_owner_effects: 1,
            ..Default::default()
        },
        context(false),
    )
    .unwrap();
    let object = create(&mut store, "width:1px", true);
    let first = object
        .set_property(
            &mut store,
            "width",
            "2px",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| panic!("no decisions"),
        )
        .unwrap();
    let before = store.snapshot();
    let rejected = object.set_property(
        &mut store,
        "width",
        "3px",
        None,
        CssomDeclarationRequestLimits::default(),
        |_| panic!("no decisions"),
    );
    assert!(matches!(
        rejected,
        Err(CssomError::Limit {
            resource: "pending owner effects",
            maximum: 1
        })
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(text(&store.snapshot(), &object), "width: 2px;");
    drop(first);
    let retry = object
        .set_property(
            &mut store,
            "width",
            "3px",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| panic!("no decisions"),
        )
        .unwrap();
    assert_eq!(retry.owner_effects().len(), 1);
    assert_eq!(text(&store.snapshot(), &object), "width: 3px;");
}

#[test]
fn resolver_receives_actual_candidates_and_captured_support_then_runs_once() {
    let mut store = CssomStore::new(CssomLimits::default(), context(true)).unwrap();
    let object = create(&mut store, "height:1px", false);
    let calls = Cell::new(0);
    let result = object
        .set_property(
            &mut store,
            "width",
            "2px",
            Some("important"),
            CssomDeclarationRequestLimits::default(),
            |request| {
                calls.set(calls.get() + 1);
                assert_eq!(request.context(), &context(true));
                assert_eq!(request.candidates().len(), 1);
                let declaration = request.candidates()[0].declaration().unwrap();
                assert_eq!(
                    declaration.known().unwrap().grammar(),
                    CssPropertyGrammar::from_name("width").unwrap()
                );
                assert_eq!(
                    declaration.value_components().serialize().unwrap().as_css(),
                    "2px"
                );
                Ok(admission(request, CssomDeclarationUsability::Usable))
            },
        )
        .unwrap();
    assert_eq!(calls.get(), 1);
    let _: &() = result.value();
    assert_eq!(
        text(&store.snapshot(), &object),
        "height: 1px; width: 2px !important;"
    );
    assert!(
        store
            .snapshot()
            .block(object.id())
            .unwrap()
            .admission_inputs()
            .contains(&version("values-owner", 1))
    );
}

#[test]
fn unusable_decision_is_successful_noop_and_changed_inventory_is_rejected() {
    let mut store = CssomStore::new(CssomLimits::default(), context(true)).unwrap();
    let object = create(&mut store, "width:1px", false);
    let before = store.snapshot();
    let noop = object
        .set_property(
            &mut store,
            "width",
            "2px",
            None,
            CssomDeclarationRequestLimits::default(),
            |request| Ok(admission(request, CssomDeclarationUsability::Unusable)),
        )
        .unwrap();
    assert!(matches!(
        noop.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    for changed_identity in ["values-owner", "renderer"] {
        let result = object.set_property(
            &mut store,
            "width",
            "3px",
            None,
            CssomDeclarationRequestLimits::default(),
            |request| {
                let packet = admission(request, CssomDeclarationUsability::Usable);
                // Decision work observed revision1; independently current facts
                // now report revision2, rather than reusing the earlier inventory.
                let current = packet
                    .current_inputs()
                    .iter()
                    .map(|input| {
                        if input.identity == changed_identity {
                            version(changed_identity, 2)
                        } else {
                            input.clone()
                        }
                    })
                    .collect();
                Ok(CssomAdmission::new(packet.decisions().to_vec(), current))
            },
        );
        assert!(matches!(result, Err(CssomError::StaleDeclarationDecision)));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(text(&store.snapshot(), &object), "width: 1px;");
    }
}

#[test]
fn incomplete_and_foreign_decisions_publish_nothing() {
    let mut store = CssomStore::new(CssomLimits::default(), context(true)).unwrap();
    let object = create(&mut store, "width:1px", false);
    let before = store.snapshot();
    let incomplete = object.set_property(
        &mut store,
        "width",
        "2px",
        None,
        CssomDeclarationRequestLimits::default(),
        |_| Ok(CssomAdmission::new(Vec::new(), Vec::new())),
    );
    assert!(matches!(
        incomplete,
        Err(CssomError::UnresolvedDeclarationPreparation)
    ));
    let mut foreign_batch = batch(&store);
    let foreign_preparation = foreign_batch
        .prepare_set_property(
            object.id(),
            "width",
            "7px",
            "",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    let foreign_decision = foreign_preparation.candidates()[0]
        .decide(
            version("values-owner", 1),
            CssomDeclarationUsability::Usable,
        )
        .unwrap();
    let foreign = object.set_property(
        &mut store,
        "width",
        "2px",
        None,
        CssomDeclarationRequestLimits::default(),
        |_| {
            Ok(CssomAdmission::new(
                vec![foreign_decision],
                vec![version("values-owner", 1)],
            ))
        },
    );
    assert!(matches!(foreign, Err(CssomError::StaleDeclarationDecision)));
    drop(foreign_preparation);
    drop(foreign_batch);
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(text(&store.snapshot(), &object), "width: 1px;");
}

#[test]
fn resolver_error_and_unwind_preserve_store_and_allow_fresh_retry() {
    let mut store = CssomStore::new(
        CssomLimits {
            max_pending_owner_effects: 1,
            ..Default::default()
        },
        context(true),
    )
    .unwrap();
    let object = create(&mut store, "width:1px", true);
    let before = store.snapshot();
    let error = object.set_property(
        &mut store,
        "width",
        "2px",
        None,
        CssomDeclarationRequestLimits::default(),
        |_| Err(CssomError::InvalidInput("decision owner failed")),
    );
    assert!(matches!(
        error,
        Err(CssomError::InvalidInput("decision owner failed"))
    ));
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _ = object.set_property(
                &mut store,
                "width",
                "3px",
                None,
                CssomDeclarationRequestLimits::default(),
                |_| panic!("decision owner unwind"),
            );
        }))
        .is_err()
    );
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(text(&store.snapshot(), &object), "width: 1px;");
    let retry = object
        .set_property(
            &mut store,
            "width",
            "4px",
            None,
            CssomDeclarationRequestLimits::default(),
            |request| Ok(admission(request, CssomDeclarationUsability::Usable)),
        )
        .unwrap();
    assert_eq!(retry.owner_effects().len(), 1);
    assert_eq!(retry.owner_effects()[0].value(), "width: 4px;");
}

#[test]
fn typed_staging_shares_operations_and_caught_resolver_unwind_aborts_prior_edit() {
    let mut store = CssomStore::new(CssomLimits::default(), context(true)).unwrap();
    let object = create(&mut store, "width:1px;height:2px", false);
    let before = store.snapshot();
    let mut pending = batch(&store);
    {
        let mut edit = object.edit(&mut pending).unwrap();
        let removed = edit
            .remove_property("height", CssomDeclarationRequestLimits::default())
            .unwrap();
        assert_eq!(removed.value(), "2px");
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _ = edit.set_property(
                    "width",
                    "3px",
                    None,
                    CssomDeclarationRequestLimits::default(),
                    |_| panic!("staged resolver unwind"),
                );
            }))
            .is_err()
        );
    }
    assert!(matches!(
        store.commit(pending),
        Err(CssomError::BatchAborted)
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(text(&store.snapshot(), &object), "size: 1px 2px;");
    let mut pending = batch(&store);
    {
        let mut edit = object.edit(&mut pending).unwrap();
        let removed = edit
            .remove_property("height", CssomDeclarationRequestLimits::default())
            .unwrap();
        assert_eq!(removed.into_parts().0, "2px");
        let staged = edit
            .set_property(
                "width",
                "4px",
                None,
                CssomDeclarationRequestLimits::default(),
                |request| Ok(admission(request, CssomDeclarationUsability::Usable)),
            )
            .unwrap();
        let _: &() = staged.value();
    }
    assert_eq!(store.snapshot().revision(), before.revision());
    store.commit(pending).unwrap();
    assert_eq!(
        store.snapshot().revision().value(),
        before.revision().value() + 1
    );
    assert_eq!(text(&store.snapshot(), &object), "width: 4px;");
}

#[test]
fn outstanding_preparation_prevents_typed_staged_publication() {
    let mut store = CssomStore::new(CssomLimits::default(), context(true)).unwrap();
    let object = create(&mut store, "width:1px;height:2px", false);
    let before = store.snapshot();
    let mut pending = batch(&store);
    let outstanding = pending
        .prepare_set_property(
            object.id(),
            "width",
            "3px",
            "",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    let staged = object
        .edit(&mut pending)
        .unwrap()
        .remove_property("height", CssomDeclarationRequestLimits::default())
        .unwrap();
    assert_eq!(staged.value(), "2px");
    assert!(matches!(
        store.commit(pending),
        Err(CssomError::UnresolvedDeclarationPreparation)
    ));
    drop(outstanding);
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(text(&store.snapshot(), &object), "size: 1px 2px;");
    let _ = object
        .remove_property(
            &mut store,
            "height",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
}

#[test]
fn equal_value_provenance_changes_and_abandoned_or_canceled_effects_keep_native_distinctions() {
    let mut store = CssomStore::new(
        CssomLimits {
            max_pending_owner_effects: 1,
            ..Default::default()
        },
        context(false),
    )
    .unwrap();
    let object = create(&mut store, "width:1px", true);
    let before = store.snapshot();
    let old_source = match before.block(object.id()).unwrap().data() {
        CssomBlockData::Properties {
            selected: CssomProjection::Available(value),
            ..
        } => value.entries()[0].source().clone(),
        other => panic!("selected original occurrence: {other:?}"),
    };
    let equal = object
        .set_property(
            &mut store,
            "width",
            "1px",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| panic!("no decision needed"),
        )
        .unwrap();
    assert!(matches!(equal.publication(), CssomPublication::Changed(_)));
    assert!(equal.owner_effects().is_empty());
    assert_eq!(text(&store.snapshot(), &object), text(&before, &object));
    match store.snapshot().block(object.id()).unwrap().data() {
        CssomBlockData::Properties {
            selected: CssomProjection::Available(value),
            ..
        } => assert!(!value.entries()[0].source().same_occurrence(&old_source)),
        other => panic!("selected replacement occurrence: {other:?}"),
    }
    let changed = object
        .set_property(
            &mut store,
            "width",
            "2px",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| panic!("no decision needed"),
        )
        .unwrap();
    assert_eq!(changed.owner_effects().len(), 1);
    drop(changed);
    let removed = object
        .remove_property(
            &mut store,
            "width",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    let (old, publication, diagnostics, mut effects) = removed.into_parts();
    assert_eq!(old, "2px");
    assert!(matches!(publication, CssomPublication::Changed(_)));
    assert!(diagnostics.is_empty());
    assert_eq!(effects[0].value(), "");
    effects[0].cancel().unwrap();
    assert_eq!(effects[0].status(), CssomOwnerEffectStatus::Canceled);
    let next = object
        .set_property(
            &mut store,
            "width",
            "3px",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| panic!("no decision needed"),
        )
        .unwrap();
    assert_eq!(next.owner_effects().len(), 1);
}

#[test]
fn binding_applies_exact_owner_write_and_host_failure_is_postpublication_status() {
    let mut store = CssomStore::new(
        CssomLimits {
            max_pending_owner_effects: 1,
            ..Default::default()
        },
        context(false),
    )
    .unwrap();
    let object = create(&mut store, "width:1px", true);
    let outcome = object
        .set_property(
            &mut store,
            "width",
            "2px",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| panic!("no decision needed"),
        )
        .unwrap();
    let (_, publication, _, mut effects) = outcome.into_parts();
    let committed = store.snapshot();
    assert!(matches!(publication, CssomPublication::Changed(_)));
    let effect = &mut effects[0];
    let application = store.begin_owner_application(effect).unwrap();
    assert!(application.flags().updating);
    // This concrete binding-owned synchronous write occurs after publication,
    // with the lease already active; CSSOM itself invokes no host callback.
    let host_attribute = application.effect().value().to_owned();
    let echo = CssomStyleAttributeChange {
        owner: application.effect().owner().clone(),
        local_name: "style".into(),
        namespace: None,
        value: Some(host_attribute.as_str().into()),
        echo: Some(application.effect().id().clone()),
    };
    assert_eq!(host_attribute, "width: 2px;");
    assert_eq!(
        application.classify(&echo),
        CssomOwnerNotification::MatchingEcho
    );
    application.finish_success();
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Applied);
    assert!(!store.declaration_flags(object.id()).unwrap().updating);
    assert_eq!(store.snapshot().revision(), committed.revision());
    let outcome = object
        .set_property(
            &mut store,
            "width",
            "3px",
            None,
            CssomDeclarationRequestLimits::default(),
            |_| panic!("no decision needed"),
        )
        .unwrap();
    let (_, _, _, mut effects) = outcome.into_parts();
    let published = store.snapshot();
    store
        .begin_owner_application(&mut effects[0])
        .unwrap()
        .finish_host_failure();
    assert_eq!(effects[0].status(), CssomOwnerEffectStatus::HostFailed);
    assert_eq!(store.snapshot().revision(), published.revision());
    assert_eq!(text(&store.snapshot(), &object), "width: 3px;");
}

#[test]
fn inherited_rule_css_text_setter_is_a_checked_noop_for_base_and_derived_handle() {
    let mut store = CssomStore::new(CssomLimits::default(), context(false)).unwrap();
    let mut edit = batch(&store);
    let sheet = edit
        .create_parsed_sheet(
            "a{width:1px}",
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "document".into(),
                    revision: 1,
                },
                "document".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&sheet).unwrap().clone();
    let before = store.snapshot();
    let rule = before.sheet_rules(&sheet).unwrap()[0].clone();
    let style = store.css_style_rule(&rule).unwrap();
    let base = style.as_rule();
    let derived_result = style
        .set_css_text(&mut store, "not even valid css")
        .unwrap();
    assert!(matches!(
        derived_result.publication(),
        CssomPublication::Unchanged { .. }
    ));
    let base_result = base.set_css_text(&mut store, "b{height:3px}").unwrap();
    assert!(matches!(
        base_result.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut foreign = CssomStore::new(CssomLimits::default(), context(false)).unwrap();
    assert!(matches!(
        style.set_css_text(&mut foreign, ""),
        Err(CssomError::ForeignOwner)
    ));
}

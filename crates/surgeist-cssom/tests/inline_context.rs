//! Per-block context is current association evidence, not a reparse of retained values.
use std::cell::Cell;
use surgeist_css::*;
use surgeist_cssom::*;

fn version(role: CssomInputRole, identity: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: identity.into(),
        revision,
    }
}
fn context(owner: &str, revision: u64, svg: bool) -> CssomContext {
    let mut support = CssomDeclarationSupport::try_new(
        ["width", "height"]
            .map(|name| {
                (
                    CssPropertyGrammar::from_name(name).unwrap(),
                    CssomUsabilityRequirement::AllCheckedValues,
                )
            })
            .to_vec(),
    )
    .unwrap();
    let mut parser = CssParserContext::default();
    if svg {
        support = support
            .with_svg_glyph_orientation_vertical(CssomUsabilityRequirement::AllCheckedValues);
        parser = parser.with_svg_glyph_orientation_vertical();
    }
    CssomContext {
        inputs: vec![
            CssomInput {
                version: version(CssomInputRole::Owner, owner, revision),
                data: CssomInputData::Owner {
                    identity: owner.into(),
                    attribute: Some("style".into()),
                },
            },
            CssomInput {
                version: version(
                    CssomInputRole::ParserMode,
                    &format!("{owner}:parser"),
                    revision,
                ),
                data: CssomInputData::ParserMode(parser),
            },
            CssomInput {
                version: version(
                    CssomInputRole::Support,
                    &format!("{owner}:support"),
                    revision,
                ),
                data: CssomInputData::Support(support),
            },
            CssomInput {
                version: version(
                    CssomInputRole::Document,
                    &format!("{owner}:document"),
                    revision,
                ),
                data: CssomInputData::Document {
                    identity: format!("document-{owner}"),
                },
            },
            CssomInput {
                version: version(CssomInputRole::Base, &format!("{owner}:base"), revision),
                data: CssomInputData::Base {
                    url: Some(format!("https://example.test/{owner}/{revision}/")),
                    location: None,
                },
            },
        ],
        linked: Vec::new(),
    }
}
fn packet(owner: &str, revision: u64, svg: bool) -> CssomInlineDeclarationContext {
    CssomInlineDeclarationContext::try_new(context(owner, revision, svg)).unwrap()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(store: &mut CssomStore, packet: CssomInlineDeclarationContext) -> CssomBlockId {
    let mut edit = batch(store);
    let ticket = edit.create_inline_declarations(packet).unwrap();
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    commit.block(&ticket).unwrap().clone()
}
fn commit(store: &mut CssomStore, edit: CssomBatch) -> CssomCommit {
    let mut commit = store.commit(edit).unwrap();
    // These producer tests explicitly abandon host writes, without claiming a
    // DOM operation or composed association has completed.
    for mut effect in commit.take_owner_effects() {
        effect.cancel().unwrap();
    }
    commit
}
fn set(edit: &mut CssomBatch, id: &CssomBlockId, name: &str, value: &str) {
    let prepared = edit
        .prepare_set_property(
            id,
            name,
            value,
            "",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
}
fn read(store: &CssomStore, id: &CssomBlockId, name: &str) -> String {
    store
        .css_style_properties(id)
        .unwrap()
        .get_property_value(store, name, CssomDeclarationRequestLimits::default())
        .unwrap()
}
fn versions(context: &CssomContext) -> Vec<CssomInputVersion> {
    context
        .inputs
        .iter()
        .map(|input| input.version.clone())
        .chain(context.linked.iter().map(|linked| linked.version.clone()))
        .collect()
}

#[test]
fn packet_requires_explicit_unique_roles_and_consistent_data() {
    for role in [
        CssomInputRole::Owner,
        CssomInputRole::ParserMode,
        CssomInputRole::Support,
    ] {
        let mut missing = context("element", 1, false);
        missing.inputs.retain(|input| input.version.role != role);
        assert!(matches!(
            CssomInlineDeclarationContext::try_new(missing),
            Err(CssomError::InvalidInput(_))
        ));
        let mut duplicate = context("element", 1, false);
        let mut input = duplicate
            .inputs
            .iter()
            .find(|input| input.version.role == role)
            .unwrap()
            .clone();
        input.version.identity.push_str("-second");
        duplicate.inputs.push(input);
        assert!(matches!(
            CssomInlineDeclarationContext::try_new(duplicate),
            Err(CssomError::InvalidInput(_))
        ));
    }
    let mut mismatched = context("element", 1, false);
    mismatched.inputs[1].data = CssomInputData::Document {
        identity: "wrong role".into(),
    };
    assert!(matches!(
        CssomInlineDeclarationContext::try_new(mismatched),
        Err(CssomError::InvalidInput(_))
    ));
    let mut duplicate = context("element", 1, false);
    duplicate.inputs.push(duplicate.inputs[4].clone());
    assert!(matches!(
        CssomInlineDeclarationContext::try_new(duplicate),
        Err(CssomError::InvalidInput(_))
    ));
}

#[test]
fn two_inline_blocks_keep_parser_and_support_independent_of_the_store_context() {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let first = create(&mut store, packet("first", 1, true));
    let second = create(&mut store, packet("second", 1, false));
    assert_ne!(first, second);
    let snapshot = store.snapshot();
    for (id, owner) in [(&first, "first"), (&second, "second")] {
        let block = snapshot.block(id).unwrap();
        assert_eq!(
            block.owner(),
            Some(&version(CssomInputRole::Owner, owner, 1))
        );
        assert_eq!(block.parent(), None);
        assert_eq!(block.flags(), CssomBlockFlags::default());
    }
    let mut edit = batch(&store);
    set(&mut edit, &first, "glyph-orientation-vertical", "45deg");
    set(&mut edit, &second, "glyph-orientation-vertical", "60deg");
    set(&mut edit, &first, "width", "2px");
    set(&mut edit, &second, "width", "3px");
    commit(&mut store, edit);
    assert_eq!(read(&store, &first, "glyph-orientation-vertical"), "45deg");
    assert_eq!(read(&store, &second, "glyph-orientation-vertical"), "");
    assert_eq!(read(&store, &first, "width"), "2px");
    assert_eq!(read(&store, &second, "width"), "3px");
    let mut edit = batch(&store);
    edit.update_context(context("store-global", 3, true))
        .unwrap();
    set(&mut edit, &second, "glyph-orientation-vertical", "90deg");
    commit(&mut store, edit);
    assert_eq!(read(&store, &second, "glyph-orientation-vertical"), "");
    assert_eq!(
        store.snapshot().block(&first).unwrap().inline_context(),
        Some(&packet("first", 1, true))
    );
}

#[test]
fn context_update_preserves_identity_selected_values_and_parse_time_capture() {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let original = packet("element", 1, true);
    let id = create(&mut store, original.clone());
    let object = store.css_style_properties(&id).unwrap();
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_css_text(
            &id,
            "width:4px;glyph-orientation-vertical:45deg",
            Default::default(),
        )
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
    commit(&mut store, edit);
    let before = store.snapshot();
    let captured = object.capture(&before).unwrap();
    let old_input = before
        .block(&id)
        .unwrap()
        .replacement_input()
        .unwrap()
        .clone();
    let old_admissions = before.block(&id).unwrap().admission_inputs().to_vec();
    let mut current = context("element", 2, false);
    current
        .inputs
        .iter_mut()
        .find(|input| input.version.role == CssomInputRole::Support)
        .unwrap()
        .data = CssomInputData::Support(CssomDeclarationSupport::default());
    let current = CssomInlineDeclarationContext::try_new(current).unwrap();
    let mut edit = batch(&store);
    assert!(
        edit.update_inline_declaration_context(&id, current.clone())
            .unwrap()
    );
    let publication = store.commit(edit).unwrap();
    assert!(publication.owner_effects().is_empty());
    let CssomPublication::Changed(change) = publication.publication() else {
        panic!("current context is observable");
    };
    assert!(
        change
            .affected()
            .contains(&CssomObjectId::Block(id.clone()))
    );
    for category in [
        CssomChange::Parser,
        CssomChange::Support,
        CssomChange::Owner,
        CssomChange::Base,
        CssomChange::Provenance,
    ] {
        assert!(change.categories().contains(&category));
    }
    let after = store.snapshot();
    assert!(change.supports_incremental(&before, &after));
    assert_eq!(object, store.css_style_properties(&id).unwrap());
    assert_eq!(after.block(&id).unwrap().inline_context(), Some(&current));
    assert_eq!(
        after.block(&id).unwrap().replacement_input(),
        Some(&old_input)
    );
    assert_eq!(after.block(&id).unwrap().admission_inputs(), old_admissions);
    assert_eq!(before.block(&id).unwrap().inline_context(), Some(&original));
    assert_eq!(read(&store, &id, "width"), "4px");
    assert_eq!(read(&store, &id, "glyph-orientation-vertical"), "45deg");
    assert_eq!(
        captured
            .get_property_value("width", Default::default())
            .unwrap(),
        "4px"
    );
    assert_eq!(
        after.inputs().inline_declarations,
        vec![(id.clone(), current)]
    );
    assert_eq!(after.context(), &CssomContext::default());
    let mut edit = batch(&store);
    set(&mut edit, &id, "width", "9px");
    commit(&mut store, edit);
    assert_eq!(read(&store, &id, "width"), "4px");
}

#[test]
fn equal_packet_preserves_revision_and_emits_no_effect() {
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, packet("element", 1, false));
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(
        !edit
            .update_inline_declaration_context(&id, packet("element", 1, false))
            .unwrap()
    );
    let commit = store.commit(edit).unwrap();
    assert!(matches!(
        commit.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert!(commit.owner_effects().is_empty());
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(store.snapshot().inputs(), before.inputs());
}

#[test]
fn updating_another_block_does_not_stale_prepared_work_in_the_same_batch() {
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let first = create(&mut store, packet("first", 1, false));
    let second = create(&mut store, packet("second", 1, false));
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(&second, "width", "7px", "", Default::default())
        .unwrap();
    edit.update_inline_declaration_context(&first, packet("first", 2, true))
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
    commit(&mut store, edit);
    assert_eq!(read(&store, &second, "width"), "7px");
    assert_eq!(
        store.snapshot().block(&first).unwrap().owner(),
        Some(&version(CssomInputRole::Owner, "first", 2))
    );
}

#[test]
fn changing_the_prepared_blocks_packet_rejects_work_without_publication() {
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, packet("element", 1, false));
    let before = store.snapshot();
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(&id, "width", "7px", "", Default::default())
        .unwrap();
    edit.update_inline_declaration_context(&id, packet("element", 2, false))
        .unwrap();
    assert!(matches!(
        edit.apply_declaration(prepared, &[]),
        Err(CssomError::StaleDeclarationDecision)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(store.snapshot().inputs(), before.inputs());
    assert_eq!(read(&store, &id, "width"), "");
}

#[test]
fn independently_current_versions_must_cover_the_complete_inline_packet() {
    for case in 0..9 {
        let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
        let packet = packet("element", 1, false);
        let id = create(&mut store, packet.clone());
        let before = store.snapshot();
        let mut current = versions(packet.context());
        match case {
            0..5 => {
                current.remove(case);
            }
            5 => current[4].revision = 2,
            6 => current.push(version(CssomInputRole::Document, "unrelated", 1)),
            7 => current.push(current[0].clone()),
            8 => {
                let mut conflicting = current[2].clone();
                conflicting.revision = 9;
                current.push(conflicting);
            }
            _ => unreachable!(),
        }
        let mut edit = batch(&store);
        let prepared = edit
            .prepare_set_property(&id, "width", "7px", "", Default::default())
            .unwrap();
        let error = edit
            .apply_declaration_with_inputs(prepared, &[], &current)
            .unwrap_err();
        if case >= 7 {
            assert!(matches!(error, CssomError::InvalidInput(_)));
        } else {
            assert!(matches!(error, CssomError::StaleDeclarationDecision));
        }
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(store.snapshot().inputs(), before.inputs());
    }
}

#[test]
fn resolver_receives_the_block_packet_and_current_decision_versions_remain_independent() {
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let mut context = context("element", 1, false);
    context
        .inputs
        .iter_mut()
        .find(|input| input.version.role == CssomInputRole::Support)
        .unwrap()
        .data = CssomInputData::Support(
        CssomDeclarationSupport::try_new(vec![(
            CssPropertyGrammar::from_name("background").unwrap(),
            CssomUsabilityRequirement::WholeDeclarationDecision,
        )])
        .unwrap(),
    );
    let packet = CssomInlineDeclarationContext::try_new(context).unwrap();
    let id = create(&mut store, packet.clone());
    let object = store.css_style_properties(&id).unwrap();
    let calls = Cell::new(0);
    let decision_version = version(CssomInputRole::Support, "independent-value-owner", 4);
    let outcome = object
        .set_property(
            &mut store,
            "background",
            "red",
            None,
            Default::default(),
            |request| {
                calls.set(calls.get() + 1);
                assert_eq!(request.context(), packet.context());
                let decisions = request
                    .candidates()
                    .iter()
                    .map(|candidate| {
                        candidate
                            .decide(decision_version.clone(), CssomDeclarationUsability::Usable)
                            .unwrap()
                    })
                    .collect();
                let mut current = versions(packet.context());
                current.push(decision_version.clone());
                Ok(CssomAdmission::new(decisions, current))
            },
        )
        .unwrap();
    assert_eq!(calls.get(), 1);
    let (_, _, _, effects) = outcome.into_parts();
    assert_eq!(effects.len(), 1);
    for mut effect in effects {
        assert_eq!(effect.owner(), packet.owner());
        effect.cancel().unwrap();
    }
    assert!(!read(&store, &id, "background").is_empty());
    let before = store.snapshot();
    let error = object
        .set_property(
            &mut store,
            "background",
            "blue",
            None,
            Default::default(),
            |request| {
                calls.set(calls.get() + 1);
                let decisions = request
                    .candidates()
                    .iter()
                    .map(|candidate| {
                        candidate
                            .decide(decision_version.clone(), CssomDeclarationUsability::Usable)
                            .unwrap()
                    })
                    .collect();
                let mut current = versions(packet.context());
                current[3].revision = 2;
                current.push(decision_version.clone());
                Ok(CssomAdmission::new(decisions, current))
            },
        )
        .unwrap_err();
    assert!(matches!(error, CssomError::StaleDeclarationDecision));
    assert_eq!(calls.get(), 2);
    assert_eq!(store.snapshot().revision(), before.revision());
}

#[test]
fn explicit_empty_support_and_nonqualifying_notifications_do_not_resolve_or_parse() {
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let mut input = context("element", 1, false);
    input.inputs[2].data = CssomInputData::Support(CssomDeclarationSupport::default());
    let packet = CssomInlineDeclarationContext::try_new(input).unwrap();
    let id = create(&mut store, packet.clone());
    let before = store.snapshot();
    let object = store.css_style_properties(&id).unwrap();
    let outcome = object
        .set_property(&mut store, "width", "2px", None, Default::default(), |_| {
            panic!("empty support has no usability candidate")
        })
        .unwrap();
    assert!(outcome.owner_effects().is_empty());
    assert_eq!(store.snapshot().revision(), before.revision());
    for (local, namespace) in [("STYLE", None), ("style", Some("other".into()))] {
        let mut edit = batch(&store);
        let change = CssomStyleAttributeChange {
            owner: version(CssomInputRole::Owner, "wrong", 90),
            local_name: local.into(),
            namespace,
            value: Some("unparsed".into()),
            echo: None,
        };
        assert!(
            edit.prepare_style_attribute_change(
                &id,
                &change,
                surgeist_css::CssUtf16InputLimits::default(),
                CssomDeclarationRequestLimits {
                    max_input_bytes: 0,
                    ..Default::default()
                },
                || Ok::<_, ()>(())
            )
            .unwrap()
            .is_none()
        );
        assert!(store.commit(edit).unwrap().owner_effects().is_empty());
    }
    assert_eq!(store.snapshot().revision(), before.revision());
}

#[test]
fn context_updates_reject_other_owners_foreign_blocks_and_store_context_blocks() {
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, packet("element", 1, false));
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.update_inline_declaration_context(&id, packet("different-element", 2, false)),
        Err(CssomError::InvalidInput(_))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    let mut other = CssomStore::new(Default::default(), Default::default()).unwrap();
    let foreign = create(&mut other, packet("element", 1, false));
    let mut edit = batch(&store);
    assert!(matches!(
        edit.update_inline_declaration_context(&foreign, packet("element", 2, false)),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut edit = batch(&store);
    let ticket = edit
        .create_declarations(
            CssParser::default().parse_block_contents("").into_parts().0,
            false,
            false,
            None,
        )
        .unwrap();
    let ordinary = store.commit(edit).unwrap().block(&ticket).unwrap().clone();
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.update_inline_declaration_context(&ordinary, packet("element", 1, false)),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
}

fn minimal_context() -> CssomContext {
    CssomContext {
        inputs: vec![
            CssomInput {
                version: version(CssomInputRole::Owner, "e", 1),
                data: CssomInputData::Owner {
                    identity: "e".into(),
                    attribute: None,
                },
            },
            CssomInput {
                version: version(CssomInputRole::ParserMode, "p", 1),
                data: CssomInputData::ParserMode(CssParserContext::default()),
            },
            CssomInput {
                version: version(CssomInputRole::Support, "s", 1),
                data: CssomInputData::Support(CssomDeclarationSupport::default()),
            },
        ],
        linked: Vec::new(),
    }
}

#[test]
fn packet_entry_and_string_boundaries_are_admitted_once_and_refuse_atomically() {
    // Three input entries; four bytes: version identities e/p/s and Owner data e.
    for (entries, bytes, succeeds) in [(3, 4, true), (2, 4, false), (3, 3, false)] {
        let mut store = CssomStore::new(
            CssomLimits {
                max_entries: entries,
                max_string_bytes: bytes,
                ..Default::default()
            },
            Default::default(),
        )
        .unwrap();
        let before = store.snapshot();
        let mut edit = batch(&store);
        let result = edit.create_inline_declarations(
            CssomInlineDeclarationContext::try_new(minimal_context()).unwrap(),
        );
        if succeeds {
            let ticket = result.unwrap();
            let commit = store.commit(edit).unwrap();
            assert_eq!(
                store
                    .snapshot()
                    .block(commit.block(&ticket).unwrap())
                    .unwrap()
                    .owner(),
                Some(&version(CssomInputRole::Owner, "e", 1))
            );
        } else {
            assert!(matches!(result, Err(CssomError::Limit { .. })));
            assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
            assert_eq!(store.snapshot().revision(), before.revision());
            assert_eq!(store.snapshot().inputs(), before.inputs());
        }
    }
    let mut store = CssomStore::new(
        CssomLimits {
            max_entries: 3,
            max_string_bytes: 4,
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap();
    let id = create(
        &mut store,
        CssomInlineDeclarationContext::try_new(minimal_context()).unwrap(),
    );
    let before = store.snapshot();
    let mut larger = minimal_context();
    larger.inputs[1].version.identity = "pp".into();
    larger.inputs[1].version.revision = 2;
    let mut edit = batch(&store);
    assert!(matches!(
        edit.update_inline_declaration_context(
            &id,
            CssomInlineDeclarationContext::try_new(larger).unwrap()
        ),
        Err(CssomError::Limit {
            resource: "host and live string bytes",
            maximum: 4
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().inputs(), before.inputs());
    assert_eq!(store.snapshot().revision(), before.revision());
}

#[test]
fn linked_inline_inputs_participate_in_actual_retained_source_admission() {
    let mut source = CssomStore::new(Default::default(), Default::default()).unwrap();
    let mut edit = batch(&source);
    let ticket = edit
        .create_parsed_sheet(
            "/* actual retained producer source */",
            CssomSheetInputs::external(version(CssomInputRole::Owner, "sheet", 1), true),
        )
        .unwrap();
    let sheet = source.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let source_snapshot = source.snapshot();
    assert!(source_snapshot.sheet_rules(&sheet).unwrap().is_empty());
    let mut target = CssomStore::new(
        CssomLimits {
            max_retained_source_bytes: 0,
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap();
    let id = create(&mut target, packet("element", 1, false));
    let before = target.snapshot();
    let mut linked = context("element", 2, false);
    linked.linked.push(CssomLinkedInput {
        version: version(CssomInputRole::Import, "actual-source", 1),
        snapshot: source_snapshot,
        sheet,
    });
    let mut edit = batch(&target);
    assert!(matches!(
        edit.update_inline_declaration_context(
            &id,
            CssomInlineDeclarationContext::try_new(linked).unwrap()
        ),
        Err(CssomError::Limit {
            resource: "retained parse source bytes",
            maximum: 0
        })
    ));
    assert!(matches!(target.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(target.snapshot().revision(), before.revision());
    assert_eq!(target.snapshot().inputs(), before.inputs());
}

#[test]
fn a_packet_update_keeps_the_existing_whole_store_owner_effect_staleness_guard() {
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let first = create(&mut store, packet("first", 1, false));
    let second = create(&mut store, packet("second", 1, false));
    let mut edit = batch(&store);
    set(&mut edit, &first, "width", "2px");
    let mut effect = store
        .commit(edit)
        .unwrap()
        .take_owner_effects()
        .pop()
        .unwrap();
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Pending);
    let before = store.snapshot();
    let mut edit = batch(&store);
    edit.update_inline_declaration_context(&second, packet("second", 2, false))
        .unwrap();
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    assert_ne!(before.inputs(), store.snapshot().inputs());
    assert!(matches!(
        store.begin_owner_application(&mut effect),
        Err(CssomError::StaleOwnerEffect)
    ));
    assert_eq!(effect.status(), CssomOwnerEffectStatus::Pending);
    effect.cancel().unwrap();
    assert_eq!(read(&store, &first, "width"), "2px");
}

#[test]
fn legacy_declarations_continue_to_follow_the_store_context() {
    let mut store = CssomStore::new(Default::default(), context("global", 1, false)).unwrap();
    let inline = create(&mut store, packet("element", 1, false));
    let mut edit = batch(&store);
    let ticket = edit
        .create_declarations(
            CssParser::default().parse_block_contents("").into_parts().0,
            false,
            false,
            None,
        )
        .unwrap();
    let legacy = store.commit(edit).unwrap().block(&ticket).unwrap().clone();
    let mut edit = batch(&store);
    edit.update_inline_declaration_context(&inline, packet("element", 2, true))
        .unwrap();
    set(&mut edit, &legacy, "width", "8px");
    commit(&mut store, edit);
    assert_eq!(read(&store, &legacy, "width"), "8px");
    assert!(
        store
            .snapshot()
            .block(&legacy)
            .unwrap()
            .inline_context()
            .is_none()
    );
    let mut edit = batch(&store);
    edit.update_context(CssomContext::default()).unwrap();
    set(&mut edit, &legacy, "width", "9px");
    set(&mut edit, &inline, "width", "7px");
    commit(&mut store, edit);
    assert_eq!(read(&store, &legacy, "width"), "8px");
    assert_eq!(read(&store, &inline, "width"), "7px");
}

#[test]
fn linked_packet_versions_are_required_and_duplicate_input_qualifications_are_rejected() {
    let mut source = CssomStore::new(Default::default(), Default::default()).unwrap();
    let mut edit = batch(&source);
    let ticket = edit
        .create_parsed_sheet(
            "/* retained input */",
            CssomSheetInputs::external(version(CssomInputRole::Owner, "sheet", 1), true),
        )
        .unwrap();
    let sheet = source.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let linked = CssomLinkedInput {
        version: version(CssomInputRole::Import, "linked-source", 4),
        snapshot: source.snapshot(),
        sheet,
    };
    let mut inputs = context("element", 1, false);
    inputs.linked.push(linked.clone());
    let packet = CssomInlineDeclarationContext::try_new(inputs.clone()).unwrap();
    let mut duplicate = inputs.clone();
    duplicate.linked.push(linked.clone());
    assert!(matches!(
        CssomInlineDeclarationContext::try_new(duplicate),
        Err(CssomError::InvalidInput(_))
    ));
    let mut duplicate = inputs;
    duplicate.linked[0].version = duplicate.inputs[0].version.clone();
    assert!(matches!(
        CssomInlineDeclarationContext::try_new(duplicate),
        Err(CssomError::InvalidInput(_))
    ));
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, packet.clone());
    let before = store.snapshot();
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(&id, "width", "2px", "", Default::default())
        .unwrap();
    let mut current = versions(packet.context());
    assert_eq!(current.pop(), Some(linked.version));
    assert!(matches!(
        edit.apply_declaration_with_inputs(prepared, &[], &current),
        Err(CssomError::StaleDeclarationDecision)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(&id, "width", "2px", "", Default::default())
        .unwrap();
    edit.apply_declaration_with_inputs(prepared, &[], &versions(packet.context()))
        .unwrap();
    commit(&mut store, edit);
    assert_eq!(read(&store, &id, "width"), "2px");
}

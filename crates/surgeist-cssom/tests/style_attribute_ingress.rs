//! Native owner input, source custody, finite admission and exact feedback echoes.
#![forbid(unsafe_code)]
use std::{cell::Cell, sync::Arc};
use surgeist_css::*;
use surgeist_cssom::*;

fn version(role: CssomInputRole, name: &str) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: name.into(),
        revision: 1,
    }
}
fn packet(name: &str, mode: CssParserMode) -> CssomInlineDeclarationContext {
    let profile = CssomDeclarationSupport::try_new(
        ["color", "width", "height", "margin", "margin-left"]
            .map(|name| {
                (
                    CssPropertyGrammar::from_name(name).unwrap(),
                    CssomUsabilityRequirement::AllCheckedValues,
                )
            })
            .to_vec(),
    )
    .unwrap();
    CssomInlineDeclarationContext::try_new(CssomContext {
        inputs: vec![
            CssomInput {
                version: version(CssomInputRole::Owner, name),
                data: CssomInputData::Owner {
                    identity: name.into(),
                    attribute: Some("style".into()),
                },
            },
            CssomInput {
                version: version(CssomInputRole::ParserMode, &format!("{name}:parser")),
                data: CssomInputData::ParserMode(CssParserContext::new(mode)),
            },
            CssomInput {
                version: version(CssomInputRole::Support, &format!("{name}:support")),
                data: CssomInputData::Support(profile),
            },
        ],
        linked: Vec::new(),
    })
    .unwrap()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(store: &mut CssomStore, packet: &CssomInlineDeclarationContext) -> CssomBlockId {
    let mut edit = batch(store);
    let ticket = edit.create_inline_declarations(packet.clone()).unwrap();
    store.commit(edit).unwrap().block(&ticket).unwrap().clone()
}
fn change<'a>(
    packet: &CssomInlineDeclarationContext,
    value: Option<CssomStyleAttributeInput<'a>>,
) -> CssomStyleAttributeChange<'a> {
    CssomStyleAttributeChange {
        owner: packet.owner().clone(),
        local_name: "style".into(),
        namespace: None,
        value,
        echo: None,
    }
}
fn apply(edit: &mut CssomBatch, id: &CssomBlockId, change: &CssomStyleAttributeChange<'_>) {
    let prepared = edit
        .prepare_style_attribute_change(
            id,
            change,
            CssUtf16InputLimits::default(),
            CssomDeclarationRequestLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap()
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
}
fn read(store: &CssomStore, id: &CssomBlockId, name: &str) -> String {
    store
        .css_style_properties(id)
        .unwrap()
        .get_property_value(store, name, Default::default())
        .unwrap()
}

#[test]
fn native_units_and_shared_origins_survive_preprocessing_and_report_replacement() {
    let packet = packet("element", CssParserMode::Standards);
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, &packet);
    let mut units: Vec<u16> = "--x:".encode_utf16().collect();
    units.extend([13, 10, 13, 12, 0, 0xd83d, 0xde00, 0xd800, 97, 0xdc00]);
    units.extend("; mystery:bad; width:2px".encode_utf16());
    let original = Arc::new(units);
    let mut edit = batch(&store);
    apply(
        &mut edit,
        &id,
        &change(
            &packet,
            Some(CssomStyleAttributeInput::Utf16(original.clone())),
        ),
    );
    assert!(store.commit(edit).unwrap().owner_effects().is_empty());
    let before = store.snapshot();
    let input = before.block(&id).unwrap().replacement_input().unwrap();
    let root = input.parse_source().unwrap().source();
    assert_eq!(
        input.source(),
        "--x:\n\n\n\u{fffd}😀\u{fffd}a\u{fffd}; mystery:bad; width:2px"
    );
    assert!(Arc::ptr_eq(root.original_utf16().unwrap(), &original));
    for (bytes, units) in [
        (4..5, 4..6),
        (5..6, 6..7),
        (6..7, 7..8),
        (7..10, 8..9),
        (10..14, 9..11),
        (14..17, 11..12),
        (17..18, 12..13),
        (18..21, 13..14),
    ] {
        assert_eq!(root.original_utf16_range(bytes), Some(units));
    }
    let CssomBlockData::Properties {
        selected: CssomProjection::Available(selected),
        ..
    } = before.block(&id).unwrap().data()
    else {
        panic!()
    };
    for entry in selected.entries() {
        assert!(
            entry
                .source()
                .parsed_name()
                .unwrap()
                .source()
                .same_snapshot(root)
        );
        assert!(
            entry
                .source()
                .parsed_value()
                .unwrap()
                .source()
                .same_snapshot(root)
        );
    }
    assert_eq!(input.diagnostics().len(), 1);
    assert!(input.diagnostics()[0].source().unwrap().same_snapshot(root));
    let old = root.clone();
    let mut edit = batch(&store);
    apply(&mut edit, &id, &change(&packet, Some("width:3px".into())));
    store.commit(edit).unwrap();
    assert_eq!(read(&store, &id, "width"), "3px");
    assert_eq!(
        old.original_utf16().unwrap().as_slice(),
        original.as_slice()
    );
    assert!(
        before
            .block(&id)
            .unwrap()
            .replacement_input()
            .unwrap()
            .parse_source()
            .unwrap()
            .source()
            .same_snapshot(&old)
    );
}

#[test]
fn empty_and_invalid_inputs_charge_exact_root_storage_before_publication() {
    let packet = packet("element", CssParserMode::Standards);
    for text in ["", "/* nothing */", "mystery:bad"] {
        let original: Arc<Vec<u16>> = Arc::new(text.encode_utf16().collect());
        let parsed = CssParser::new(CssParserContext::default(), Default::default())
            .parse_block_contents_utf16(original.clone(), Default::default(), || Ok::<_, ()>(()))
            .unwrap();
        let bytes = parsed.source().source().storage_usage().retained_bytes();
        assert!(bytes > 0);
        for maximum in [bytes - 1, bytes] {
            let mut store = CssomStore::new(
                CssomLimits {
                    max_retained_source_bytes: maximum,
                    ..Default::default()
                },
                Default::default(),
            )
            .unwrap();
            let id = create(&mut store, &packet);
            let before = store.snapshot();
            let mut edit = batch(&store);
            let prepared = edit
                .prepare_style_attribute_change(
                    &id,
                    &change(
                        &packet,
                        Some(CssomStyleAttributeInput::Utf16(original.clone())),
                    ),
                    Default::default(),
                    Default::default(),
                    || Ok::<_, ()>(()),
                )
                .unwrap()
                .unwrap();
            let result = edit.apply_declaration(prepared, &[]);
            if maximum < bytes {
                assert!(
                    matches!(
                        result,
                        Err(CssomError::Limit {
                            resource: "retained parse source bytes",
                            ..
                        })
                    ),
                    "{result:?}"
                );
                assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
                assert_eq!(store.snapshot().revision(), before.revision());
            } else {
                result.unwrap();
                store.commit(edit).unwrap();
                let snapshot = store.snapshot();
                let source = snapshot
                    .block(&id)
                    .unwrap()
                    .replacement_input()
                    .unwrap()
                    .parse_source()
                    .unwrap()
                    .source();
                assert!(Arc::ptr_eq(source.original_utf16().unwrap(), &original));
                assert_eq!(read(&store, &id, "width"), "");
            }
        }
    }
}

#[test]
fn created_tickets_initialize_two_different_contexts_in_one_batch() {
    let standards = packet("standards", CssParserMode::Standards);
    let quirks = packet("quirks", CssParserMode::Quirks);
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    let a = edit.create_inline_declarations(standards.clone()).unwrap();
    let b = edit.create_inline_declarations(quirks.clone()).unwrap();
    for (ticket, context) in [(&a, &standards), (&b, &quirks)] {
        let prepared = edit
            .prepare_created_style_attribute_change(
                ticket,
                &change(context, Some("color:ABC; width:2px".into())),
                Default::default(),
                Default::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap()
            .unwrap();
        edit.apply_declaration(prepared, &[]).unwrap();
    }
    assert_eq!(store.snapshot().revision(), before.revision());
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    let a = commit.block(&a).unwrap();
    let b = commit.block(&b).unwrap();
    assert_ne!(a, b);
    assert_eq!(read(&store, a, "color"), "");
    assert_eq!(read(&store, b, "color"), "rgb(170, 187, 204)");
    for (id, packet, mode) in [
        (a, &standards, CssParserMode::Standards),
        (b, &quirks, CssParserMode::Quirks),
    ] {
        let snapshot = store.snapshot();
        let block = snapshot.block(id).unwrap();
        assert_eq!(
            block.replacement_input().unwrap().parser_context(),
            CssParserContext::new(mode)
        );
        assert_eq!(block.owner(), Some(packet.owner()));
        assert!(block.parent().is_none());
    }
}

#[test]
fn foreign_creation_tickets_and_late_refusal_abort_the_entire_batch() {
    let packet = packet("element", CssParserMode::Standards);
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let before = store.snapshot();
    let mut first = batch(&store);
    let ticket = first.create_inline_declarations(packet.clone()).unwrap();
    let mut second = batch(&store);
    let result = second.prepare_created_style_attribute_change(
        &ticket,
        &change(&packet, Some("width:2px".into())),
        Default::default(),
        Default::default(),
        || Ok::<_, ()>(()),
    );
    assert!(matches!(
        result,
        Err(CssomStyleAttributeError::Cssom(CssomError::ForeignTicket))
    ));
    assert!(matches!(
        store.commit(second),
        Err(CssomError::BatchAborted)
    ));
    let prepared = first
        .prepare_created_style_attribute_change(
            &ticket,
            &change(&packet, Some("width:2px".into())),
            Default::default(),
            Default::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap()
        .unwrap();
    first.apply_declaration(prepared, &[]).unwrap();
    let result = first.prepare_created_style_attribute_change(
        &ticket,
        &change(&packet, Some("width:3px".into())),
        CssUtf16InputLimits {
            max_input_units: 0,
            ..Default::default()
        },
        Default::default(),
        || Ok::<_, ()>(()),
    );
    assert!(matches!(
        result,
        Err(CssomStyleAttributeError::Input(error)) if error.exhausted_limit() == Some(CssUtf16Resource::InputUnits)
    ));
    assert!(matches!(store.commit(first), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
}

#[test]
fn interruption_and_component_refusal_preserve_the_cause_and_poison_the_batch() {
    let packet = packet("element", CssParserMode::Standards);
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, &packet);
    let before = store.snapshot();
    let raw = Arc::new("width:2px".encode_utf16().collect());
    for input in [
        CssomStyleAttributeInput::Utf8("width:2px"),
        CssomStyleAttributeInput::Utf16(raw),
    ] {
        let mut count = 0;
        let mut probe = batch(&store);
        let prepared = probe
            .prepare_style_attribute_change(
                &id,
                &change(&packet, Some(input.clone())),
                Default::default(),
                Default::default(),
                || {
                    count += 1;
                    Ok::<_, usize>(())
                },
            )
            .unwrap()
            .unwrap();
        drop(prepared);
        drop(probe);
        for stop in [1, 3, count] {
            let mut calls = 0;
            let mut edit = batch(&store);
            let result = edit.prepare_style_attribute_change(
                &id,
                &change(&packet, Some(input.clone())),
                Default::default(),
                Default::default(),
                || {
                    calls += 1;
                    if calls == stop { Err(stop) } else { Ok(()) }
                },
            );
            assert!(
                matches!(result, Err(CssomStyleAttributeError::Input(ref error)) if error.interruption() == Some(&stop))
            );
            assert_eq!(calls, stop);
            assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
            assert_eq!(store.snapshot().revision(), before.revision());
        }
    }
    let mut edit = batch(&store);
    let result = edit.prepare_style_attribute_change(
        &id,
        &change(&packet, Some("width:2px} a b c d".into())),
        Default::default(),
        CssomDeclarationRequestLimits {
            max_input_components: 4,
            ..Default::default()
        },
        || Ok::<_, ()>(()),
    );
    let Err(CssomStyleAttributeError::Cssom(CssomError::ParseResource(diagnostic))) = result else {
        panic!("component allowance must be a resource refusal")
    };
    assert!(
        diagnostic
            .error()
            .component_error()
            .is_some_and(CssComponentValueError::is_component_limit)
    );
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
}

#[test]
fn unrelated_notifications_do_no_input_work_and_absence_still_parses_empty() {
    let packet = packet("element", CssParserMode::Standards);
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, &packet);
    let before = store.snapshot();
    for (name, namespace) in [("STYLE", None), ("style", Some("other".into()))] {
        let mut notification = change(&packet, Some("arbitrarily-invalid".into()));
        notification.local_name = name.into();
        notification.namespace = namespace;
        notification.owner.revision += 1;
        let mut edit = batch(&store);
        assert!(
            edit.prepare_style_attribute_change(
                &id,
                &notification,
                CssUtf16InputLimits {
                    max_input_units: 0,
                    ..Default::default()
                },
                CssomDeclarationRequestLimits {
                    max_input_bytes: 0,
                    ..Default::default()
                },
                || -> Result<(), ()> { panic!("unrelated input must not poll") },
            )
            .unwrap()
            .is_none()
        );
        store.commit(edit).unwrap();
    }
    assert_eq!(store.snapshot().revision(), before.revision());
    let calls = Cell::new(0);
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_style_attribute_change(
            &id,
            &change(&packet, None),
            Default::default(),
            Default::default(),
            || {
                calls.set(calls.get() + 1);
                Ok::<_, ()>(())
            },
        )
        .unwrap()
        .unwrap();
    assert!(calls.get() > 0);
    edit.apply_declaration(prepared, &[]).unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    let input = snapshot.block(&id).unwrap().replacement_input().unwrap();
    assert_eq!(input.source(), "");
    assert!(
        input
            .parse_source()
            .unwrap()
            .source()
            .original_utf16()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn raw_utf16_echoes_match_exact_units_without_css_normalization() {
    let packet = packet("element", CssParserMode::Standards);
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, &packet);
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(&id, "--x", "\u{fffd}", "", Default::default())
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
    let mut effects = store.commit(edit).unwrap().take_owner_effects();
    let effect = &mut effects[0];
    assert_eq!(effect.value(), "--x: \u{fffd};");
    let exact: Arc<Vec<u16>> = Arc::new(effect.value().encode_utf16().collect());
    let lease = store.begin_owner_application(effect).unwrap();
    let mut notification = change(
        &packet,
        Some(CssomStyleAttributeInput::Utf16(exact.clone())),
    );
    notification.echo = Some(lease.effect().id().clone());
    assert_eq!(
        lease.classify(&notification),
        CssomOwnerNotification::MatchingEcho
    );
    // These would preprocess to the serialized U+FFFD, but are not its raw units.
    for raw in [0, 0xd800] {
        let mut different = exact.as_ref().clone();
        let replacement = different.iter_mut().find(|unit| **unit == 0xfffd).unwrap();
        *replacement = raw;
        notification.value = Some(CssomStyleAttributeInput::Utf16(Arc::new(different)));
        assert_eq!(
            lease.classify(&notification),
            CssomOwnerNotification::Unrelated
        );
    }
    let mut different = exact.as_ref().clone();
    different.push(0xd800);
    notification.value = Some(CssomStyleAttributeInput::Utf16(Arc::new(different)));
    assert_eq!(
        lease.classify(&notification),
        CssomOwnerNotification::Unrelated
    );
    notification.value = None;
    assert_eq!(
        lease.classify(&notification),
        CssomOwnerNotification::Unrelated
    );
    notification.value = Some(CssomStyleAttributeInput::Utf16(exact));
    notification.echo = None;
    assert_eq!(
        lease.classify(&notification),
        CssomOwnerNotification::Unrelated
    );
    lease.finish_success();
    assert!(!store.declaration_flags(&id).unwrap().updating);
}

#[test]
fn selected_edits_keep_the_original_notification_without_replaying_shorthand() {
    let packet = packet("element", CssParserMode::Standards);
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, &packet);
    let mut edit = batch(&store);
    apply(
        &mut edit,
        &id,
        &change(&packet, Some("margin:1px 2px; width:3px".into())),
    );
    store.commit(edit).unwrap();
    let old = store.snapshot();
    let root = old
        .block(&id)
        .unwrap()
        .replacement_input()
        .unwrap()
        .parse_source()
        .unwrap()
        .source()
        .clone();
    let mut edit = batch(&store);
    edit.remove_property(&id, "margin-left", Default::default())
        .unwrap();
    let mut effects = store.commit(edit).unwrap().take_owner_effects();
    assert_eq!(read(&store, &id, "margin-left"), "");
    assert_eq!(read(&store, &id, "margin-top"), "1px");
    assert!(!effects[0].value().contains("margin:"));
    effects[0].cancel().unwrap();
    let snapshot = store.snapshot();
    assert!(
        snapshot
            .block(&id)
            .unwrap()
            .replacement_input()
            .unwrap()
            .parse_source()
            .unwrap()
            .source()
            .same_snapshot(&root)
    );
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_css_text(&id, "--x:a\r\nb", Default::default())
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
    for mut effect in store.commit(edit).unwrap().take_owner_effects() {
        effect.cancel().unwrap();
    }
    let snapshot = store.snapshot();
    let setter = snapshot.block(&id).unwrap().replacement_input().unwrap();
    assert_eq!(setter.source(), "--x:a\r\nb");
    assert!(setter.parse_source().is_none());
}

#[test]
fn normalized_equal_empty_projections_keep_distinct_original_replacement_provenance() {
    let packet = packet("element", CssParserMode::Standards);
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, &packet);
    let mut old = None;
    for text in ["/*a\r\nb*/", "/*a\nb*/"] {
        let mut edit = batch(&store);
        apply(&mut edit, &id, &change(&packet, Some(text.into())));
        store.commit(edit).unwrap();
        let snapshot = store.snapshot();
        let input = snapshot.block(&id).unwrap().replacement_input().unwrap();
        assert_eq!(input.source(), "/*a\nb*/");
        let root = input.parse_source().unwrap().source();
        assert_eq!(
            root.original_utf16().unwrap().as_slice(),
            text.encode_utf16().collect::<Vec<_>>()
        );
        if let Some(previous) = &old {
            assert!(!root.same_snapshot(previous));
        }
        old = Some(root.clone());
    }
}

#[test]
fn preparation_request_and_state_limits_keep_their_own_atomic_refusals() {
    let packet = packet("element", CssParserMode::Standards);
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let id = create(&mut store, &packet);
    let before = store.snapshot();
    let mut edit = batch(&store);
    let result = edit.prepare_style_attribute_change(
        &id,
        &change(&packet, Some("width:2px".into())),
        CssUtf16InputLimits {
            max_work: 0,
            ..Default::default()
        },
        Default::default(),
        || Ok::<_, ()>(()),
    );
    assert!(matches!(
        result,
        Err(CssomStyleAttributeError::Input(error)) if error.exhausted_limit() == Some(CssUtf16Resource::Work)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    for (limits, resource) in [
        (
            CssomDeclarationRequestLimits {
                max_input_bytes: 2,
                ..Default::default()
            },
            "declaration input bytes",
        ),
        (
            CssomDeclarationRequestLimits {
                max_entries: 0,
                ..Default::default()
            },
            "declaration request entries",
        ),
    ] {
        let mut edit = batch(&store);
        let result = edit.prepare_style_attribute_change(
            &id,
            &change(&packet, Some("width:2px".into())),
            Default::default(),
            limits,
            || Ok::<_, ()>(()),
        );
        assert!(
            matches!(result, Err(CssomStyleAttributeError::Cssom(CssomError::Limit { resource: actual, .. })) if actual == resource)
        );
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(store.snapshot().revision(), before.revision());
    }
    let mut store = CssomStore::new(
        CssomLimits {
            max_entries: 64,
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap();
    let id = create(&mut store, &packet);
    let before = store.snapshot();
    let text = (0..40).map(|i| format!("--x{i}:a;")).collect::<String>();
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_style_attribute_change(
            &id,
            &change(&packet, Some(text.as_str().into())),
            Default::default(),
            Default::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap()
        .unwrap();
    assert!(matches!(
        edit.apply_declaration(prepared, &[]),
        Err(CssomError::Limit {
            resource: "entries",
            ..
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
}

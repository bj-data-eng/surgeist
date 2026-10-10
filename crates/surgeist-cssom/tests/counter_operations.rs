use surgeist_css::*;
use surgeist_cssom::*;

fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(store: &mut CssomStore, source: &str) -> (CssomSheetId, CssomRuleId) {
    let mut edit = batch(store);
    let ticket = edit
        .create_parsed_sheet(
            source,
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "d".into(),
                    revision: 1,
                },
                "d".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let rule = store.snapshot().sheet_rules(&sheet).unwrap()[0].clone();
    (sheet, rule)
}
fn store(source: &str) -> (CssomStore, CssomSheetId, CssomRuleId) {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let (sheet, rule) = create(&mut store, source);
    (store, sheet, rule)
}
fn read(
    snapshot: &CssomSnapshot,
    rule: &CssomRuleId,
    kind: CssCounterStyleDescriptorKind,
) -> String {
    snapshot
        .counter_descriptor(rule, kind, CssSpecifiedValueSerializationLimits::default())
        .unwrap()
}
fn set(
    edit: &mut CssomBatch,
    rule: &CssomRuleId,
    kind: CssCounterStyleDescriptorKind,
    value: &str,
) -> bool {
    edit.set_counter_descriptor(rule, kind, value, CssomDeclarationRequestLimits::default())
        .unwrap()
}

#[test]
fn ten_literal_attributes_use_specified_values_without_defaults_or_whole_wrapper() {
    use CssCounterStyleDescriptorKind::*;
    let (mut store, _, rule) = store("@counter-style custom { system: fixed; symbols: 'a'; }");
    assert_eq!(read(&store.snapshot(), &rule, Prefix), "");
    let cases = [
        (System, "fixed 3", "fixed 3"),
        (Symbols, "'x' 'y'", "\"x\" \"y\""),
        (AdditiveSymbols, "10 'X', 1 'I'", "10 \"X\", 1 \"I\""),
        (Negative, "'-' ')'", "\"-\" \")\""),
        (Prefix, "'('", "\"(\""),
        (Suffix, "'. '", "\". \""),
        (Range, "1 10, 20 infinite", "1 10, 20 infinite"),
        (Pad, "3 '0'", "3 \"0\""),
        (SpeakAs, "words", "words"),
        (Fallback, "DeCiMaL", "decimal"),
    ];
    let before = store.snapshot();
    let block = before.rule(&rule).unwrap().data().block().unwrap().clone();
    let mut edit = batch(&store);
    for (kind, source, _) in cases {
        assert!(set(&mut edit, &rule, kind, source));
    }
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    let after = store.snapshot();
    for (kind, _, expected) in cases {
        assert_eq!(read(&after, &rule, kind), expected);
    }
    assert_eq!(after.rule(&rule).unwrap().data().block(), Some(&block));
    assert_eq!(read(&before, &rule, System), "fixed");
    assert_eq!(read(&before, &rule, Symbols), "\"a\"");
    assert_eq!(read(&before, &rule, AdditiveSymbols), "");
    assert!(
        matches!(after.block(&block).unwrap().data(), CssomBlockData::CounterStyle(original)
        if original.occurrences().len() == 2)
    );
}

#[test]
fn undefined_rules_remain_readable_and_can_be_repaired() {
    use CssCounterStyleDescriptorKind::*;
    for (source, kind, repair, expected) in [
        (
            "@counter-style custom { system: numeric; symbols:'a'; }",
            Symbols,
            "'0' '1'",
            "\"0\" \"1\"",
        ),
        (
            "@counter-style custom { system: additive; }",
            AdditiveSymbols,
            "1 'I', 0 'Z'",
            "1 \"I\", 0 \"Z\"",
        ),
        ("@counter-style custom {}", Symbols, "'a'", "\"a\""),
    ] {
        let (mut store, _, rule) = store(source);
        let before = store.snapshot();
        let block = before.rule(&rule).unwrap().data().block().unwrap();
        assert!(matches!(
            before
                .block(block)
                .unwrap()
                .counter_descriptors(Default::default())
                .unwrap()
                .definition_status(),
            CssCounterStyleDefinitionStatus::Undefined(_)
        ));
        assert_eq!(read(&before, &rule, Prefix), "");
        let mut edit = batch(&store);
        assert!(set(&mut edit, &rule, kind, repair));
        store.commit(edit).unwrap();
        assert_eq!(read(&store.snapshot(), &rule, kind), expected);
    }
}

#[test]
fn noops_preserve_prior_changes_and_algorithm_identity_including_default_system() {
    use CssCounterStyleDescriptorKind::*;
    let (mut store, _, rule) = store("@counter-style custom { system:numeric; symbols:'0' '1'; }");
    let before = store.snapshot();
    let mut edit = batch(&store);
    for (kind, value) in [
        (System, "fixed"),
        (Symbols, "'x'"),
        (Prefix, "]"),
        (Prefix, "'x' !important"),
        (Range, "10 1"),
    ] {
        assert!(!set(&mut edit, &rule, kind, value));
    }
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Prefix, "'P'"));
    assert!(!set(&mut edit, &rule, Symbols, "'x'"));
    store.commit(edit).unwrap();
    assert_eq!(read(&store.snapshot(), &rule, Prefix), "\"P\"");
    assert_eq!(read(&store.snapshot(), &rule, Symbols), "\"0\" \"1\"");

    let (mut store, _, rule) = self::store("@counter-style custom { symbols:'a' 'b'; }");
    assert_eq!(read(&store.snapshot(), &rule, System), "");
    let mut edit = batch(&store);
    assert!(!set(&mut edit, &rule, System, "numeric"));
    assert!(set(&mut edit, &rule, System, "symbolic"));
    store.commit(edit).unwrap();
    assert_eq!(read(&store.snapshot(), &rule, System), "symbolic");
}

#[test]
fn native_symbolic_status_is_typed_unavailability_and_aborts_prior_staged_work() {
    use CssCounterStyleDescriptorKind::*;
    for (source, kind, value, status) in [
        (
            "@counter-style custom { system:fixed; symbols:'x'; }",
            Symbols,
            "env(counter-symbols)",
            CssCounterStyleDefinitionStatus::SubstitutionDependent,
        ),
        (
            "@counter-style custom { system:fixed; symbols:'x'; }",
            System,
            "env(counter-system)",
            CssCounterStyleDefinitionStatus::SubstitutionDependent,
        ),
        (
            "@counter-style custom { system:additive; additive-symbols:1 'I'; }",
            AdditiveSymbols,
            "calc(1) 'I'",
            CssCounterStyleDefinitionStatus::RequiresResolution,
        ),
    ] {
        let (mut store, _, rule) = store(source);
        let before = store.snapshot();
        let mut edit = batch(&store);
        assert!(set(&mut edit, &rule, Prefix, "'P'"));
        let result = edit.set_counter_descriptor(&rule, kind, value, Default::default());
        assert!(
            matches!(result,
            Err(CssomError::CounterStylePreparationUnavailable(s)) if s == status),
            "{kind:?} {value}: {result:?}"
        );
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(read(&store.snapshot(), &rule, Prefix), "");
    }
}

#[test]
fn original_duplicates_keep_actual_named_origins_and_edits_keep_only_raw_value_origins() {
    use CssCounterStyleDescriptorKind::*;
    let text = "@counter-style custom { symbols:'a'; prefix:'old'; prefix:'winner'; }";
    let (mut store, _, rule) = store(text);
    let before = store.snapshot();
    let block = before.rule(&rule).unwrap().data().block().unwrap().clone();
    let original = before
        .block(&block)
        .unwrap()
        .counter_descriptors(Default::default())
        .unwrap();
    assert_eq!(original.entries().len(), 3);
    let symbols = original.effective(Symbols).unwrap();
    let named = symbols.occurrence().unwrap();
    assert_eq!(named.parsed_value().unwrap().source().as_str(), text);
    assert_eq!(read(&before, &rule, Prefix), "\"winner\"");
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Prefix, " /* raw */ 'é' "));
    store.commit(edit).unwrap();
    let after = store.snapshot();
    let current = after
        .block(&block)
        .unwrap()
        .counter_descriptors(Default::default())
        .unwrap();
    assert_eq!(current.entries().len(), 2);
    assert!(
        current
            .effective(Symbols)
            .unwrap()
            .occurrence()
            .unwrap()
            .same_occurrence(named)
    );
    let edited = current.effective(Prefix).unwrap();
    assert!(edited.occurrence().is_none());
    assert_eq!(
        edited.value().origin().unwrap().source().as_str(),
        " /* raw */ 'é' "
    );
    assert!(
        !edited
            .value()
            .origin()
            .unwrap()
            .source()
            .same_snapshot(named.parsed_value().unwrap().source())
    );
    drop(store);
    assert_eq!(read(&after, &rule, Prefix), "\"é\"");
    assert_eq!(read(&before, &rule, Prefix), "\"winner\"");
}

#[test]
fn live_name_readout_escapes_raw_content_and_existing_reserved_guards_remain_noops() {
    let (mut store, _, rule) = store("@counter-style custom { symbols:'x'; }");
    for name in [
        "NoNe",
        "DECIMAL",
        "Disc",
        "SQUARE",
        "Circle",
        "DISCLOSURE-OPEN",
        "disclosure-closed",
    ] {
        let mut edit = batch(&store);
        assert!(!edit.set_counter_name(&rule, name).unwrap());
        store.commit(edit).unwrap();
    }
    let mut edit = batch(&store);
    assert!(edit.set_counter_name(&rule, "UpPeR-RoMaN").unwrap());
    store.commit(edit).unwrap();
    assert_eq!(
        store
            .snapshot()
            .counter_name(&rule, Default::default())
            .unwrap(),
        "upper-roman"
    );
    let mut edit = batch(&store);
    assert!(edit.set_counter_name(&rule, "1!?\0").unwrap());
    store.commit(edit).unwrap();
    assert_eq!(
        store
            .snapshot()
            .counter_name(&rule, Default::default())
            .unwrap(),
        "\\31 \\!\\?�"
    );
    let mut edit = batch(&store);
    assert!(edit.set_counter_name(&rule, "").unwrap());
    store.commit(edit).unwrap();
    assert_eq!(
        store
            .snapshot()
            .counter_name(&rule, Default::default())
            .unwrap(),
        ""
    );
}

#[test]
fn detached_rule_and_block_remain_editable_and_old_captures_keep_liveness() {
    use CssCounterStyleDescriptorKind::*;
    let (mut store, sheet, rule) = store("@counter-style custom { symbols:'x'; }");
    let before = store.snapshot();
    let block = before.rule(&rule).unwrap().data().block().unwrap().clone();
    let mut edit = batch(&store);
    edit.delete_sheet_rule(&sheet, 0).unwrap();
    store.commit(edit).unwrap();
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Prefix, "'detached'"));
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert!(after.parent_style_sheet(&rule).unwrap().is_none());
    assert_eq!(before.parent_style_sheet(&rule).unwrap(), Some(&sheet));
    assert_eq!(after.block(&block).unwrap().parent(), Some(&rule));
    assert_eq!(read(&after, &rule, Prefix), "\"detached\"");
    assert_eq!(read(&before, &rule, Prefix), "");
}

#[test]
fn request_resources_fail_atomically_and_retry_uses_same_current_state() {
    use CssCounterStyleDescriptorKind::*;
    let (mut store, _, rule) = store("@counter-style custom { symbols:'x'; }");
    let before = store.snapshot();
    for limits in [
        CssomDeclarationRequestLimits {
            max_input_components: 0,
            ..Default::default()
        },
        CssomDeclarationRequestLimits {
            max_input_bytes: 0,
            ..Default::default()
        },
        CssomDeclarationRequestLimits {
            max_entries: 0,
            ..Default::default()
        },
        CssomDeclarationRequestLimits {
            provider: CssSpecifiedValueSerializationLimits::new(0, 1000, 1000),
            ..Default::default()
        },
        CssomDeclarationRequestLimits {
            provider: CssSpecifiedValueSerializationLimits::new(1000, 0, 1000),
            ..Default::default()
        },
        CssomDeclarationRequestLimits {
            provider: CssSpecifiedValueSerializationLimits::new(1000, 1000, 0),
            ..Default::default()
        },
    ] {
        let mut edit = batch(&store);
        assert!(set(&mut edit, &rule, Suffix, "'staged'"));
        assert!(matches!(
            edit.set_counter_descriptor(&rule, Prefix, "'P'", limits),
            Err(CssomError::Component(_) | CssomError::Limit { .. } | CssomError::Value(_))
        ));
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(read(&store.snapshot(), &rule, Suffix), "");
    }
    assert!(matches!(
        before.counter_descriptor(
            &rule,
            Symbols,
            CssSpecifiedValueSerializationLimits::new(1000, 1000, 1)
        ),
        Err(CssomError::Value(_))
    ));
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Prefix, "'P'"));
    store.commit(edit).unwrap();
    assert_eq!(read(&store.snapshot(), &rule, Prefix), "\"P\"");
}

#[test]
fn foreign_owner_wrong_kind_and_stale_context_never_publish_descriptor_changes() {
    use CssCounterStyleDescriptorKind::*;
    let (mut store, _, rule) = store("@counter-style custom { symbols:'x'; }");
    let (other, _, foreign) = self::store("@counter-style custom { symbols:'x'; }");
    assert!(matches!(
        store
            .snapshot()
            .counter_descriptor(&foreign, Prefix, Default::default()),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        other.snapshot().counter_name(&rule, Default::default()),
        Err(CssomError::ForeignOwner)
    ));
    let (_, style) = create(&mut store, "a {}");
    assert!(matches!(
        store.snapshot().counter_name(&style, Default::default()),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(
        store
            .snapshot()
            .counter_descriptor(&style, Prefix, Default::default()),
        Err(CssomError::WrongKind)
    ));
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_counter_descriptor(&style, Prefix, "'x'", Default::default()),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Prefix, "'x'"));
    let mut context_edit = batch(&store);
    context_edit
        .update_context(CssomContext {
            inputs: vec![CssomInput {
                version: CssomInputVersion {
                    role: CssomInputRole::Owner,
                    identity: "element".into(),
                    revision: 1,
                },
                data: CssomInputData::Owner {
                    identity: "element".into(),
                    attribute: None,
                },
            }],
            ..Default::default()
        })
        .unwrap();
    store.commit(context_edit).unwrap();
    assert!(matches!(store.commit(edit), Err(CssomError::StaleRevision)));
    assert_eq!(read(&store.snapshot(), &rule, Prefix), "");
}

#[test]
fn only_definition_relevant_pending_values_require_resolution() {
    use CssCounterStyleDescriptorKind::*;
    let (mut store, _, rule) = store("@counter-style custom { symbols:env(counter-symbols); }");
    assert_eq!(
        read(&store.snapshot(), &rule, Symbols),
        "env(counter-symbols)"
    );
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Symbols, "'repaired'"));
    assert!(set(&mut edit, &rule, Prefix, "env(counter-prefix)"));
    store.commit(edit).unwrap();
    assert_eq!(
        read(&store.snapshot(), &rule, Prefix),
        "env(counter-prefix)"
    );
    let (mut store, _, rule) =
        self::store("@counter-style custom { system:env(counter-system); symbols:'x'; }");
    assert_eq!(
        read(&store.snapshot(), &rule, System),
        "env(counter-system)"
    );
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_counter_descriptor(&rule, System, "symbolic", Default::default()),
        Err(CssomError::CounterStylePreparationUnavailable(
            CssCounterStyleDefinitionStatus::SubstitutionDependent
        ))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
}

#[test]
fn edited_utf8_storage_is_global_retains_detached_rules_and_excludes_old_captures() {
    use CssCounterStyleDescriptorKind::*;
    // Two six/five-byte names + one-byte version/document identities = 13.
    // The actual raw source `'é'` owns four UTF-8 bytes. Original syntax is
    // retained under its existing occurrence quota, not charged again here.
    let mut store = CssomStore::new(
        CssomLimits {
            max_string_bytes: 17,
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap();
    let (sheet, rule) = create(
        &mut store,
        "@counter-style custom { symbols:'x'; } @counter-style other { symbols:'y'; }",
    );
    let second = store.snapshot().sheet_rules(&sheet).unwrap()[1].clone();
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Prefix, "'é'"));
    store.commit(edit).unwrap();
    let old = store.snapshot();
    let mut edit = batch(&store);
    edit.delete_sheet_rule(&sheet, 0).unwrap();
    store.commit(edit).unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_counter_descriptor(&second, Prefix, "''", Default::default()),
        Err(CssomError::Limit {
            resource: "host and live string bytes",
            maximum: 17
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Prefix, "''"));
    assert!(set(&mut edit, &second, Prefix, "''"));
    store.commit(edit).unwrap();
    assert_eq!(read(&store.snapshot(), &rule, Prefix), "\"\"");
    assert_eq!(read(&old, &rule, Prefix), "\"é\"");
}

#[test]
fn edited_entries_are_bounded_and_replacement_reuses_the_current_slot() {
    use CssCounterStyleDescriptorKind::*;
    // Sheet membership + original root + live root + original symbols = four.
    let mut store = CssomStore::new(
        CssomLimits {
            max_entries: 5,
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap();
    let (_, rule) = create(&mut store, "@counter-style custom { symbols:'x'; }");
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Prefix, "'first'"));
    store.commit(edit).unwrap();
    let old = store.snapshot();
    let mut edit = batch(&store);
    assert!(set(&mut edit, &rule, Prefix, "'replacement'"));
    store.commit(edit).unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_counter_descriptor(&rule, Suffix, "'extra'", Default::default()),
        Err(CssomError::Limit {
            resource: "entries",
            maximum: 5
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(read(&before, &rule, Prefix), "\"replacement\"");
    assert_eq!(read(&old, &rule, Prefix), "\"first\"");
}

#[test]
fn each_descriptor_uses_its_native_complete_grammar_and_undefined_old_algorithm_is_guarded() {
    use CssCounterStyleDescriptorKind::*;
    let (mut store, _, rule) = store("@counter-style custom { system:numeric; symbols:'0' '1'; }");
    let before = store.snapshot();
    let mut edit = batch(&store);
    for (kind, invalid) in [
        (System, "fixed 1px"),
        (Symbols, "1"),
        (AdditiveSymbols, "1 'I', 2 'II'"),
        (Negative, "'a' 'b' 'c'"),
        (Prefix, "'a' 'b'"),
        (Suffix, "'a' 'b'"),
        (Range, "1"),
        (Pad, "-1 'x'"),
        (SpeakAs, "1"),
        (Fallback, "none"),
    ] {
        assert!(!set(&mut edit, &rule, kind, invalid), "{kind:?}: {invalid}");
    }
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    let block = before.rule(&rule).unwrap().data().block().unwrap();
    assert!(matches!(
        before
            .declarations(block, Default::default())
            .unwrap()
            .length(),
        Err(CssomError::WrongKind)
    ));
    let (mut store, _, rule) =
        self::store("@counter-style custom { system:numeric; symbols:'x'; }");
    let mut edit = batch(&store);
    // Prospective cyclic would be defined, but the original numeric algorithm
    // is real even though its original one-symbol definition is undefined.
    assert!(!set(&mut edit, &rule, System, "cyclic"));
    assert!(set(&mut edit, &rule, Symbols, "'0' '1'"));
    store.commit(edit).unwrap();
    assert_eq!(read(&store.snapshot(), &rule, System), "numeric");
}

#[test]
fn name_and_current_collection_preserve_native_resource_errors() {
    let (store, _, rule) = store("@counter-style custom { symbols:'x'; }");
    let snapshot = store.snapshot();
    assert_eq!(
        snapshot
            .counter_name(&rule, CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "custom"
    );
    for limits in [
        CssSpecifiedValueSerializationLimits::new(0, 1, 6),
        CssSpecifiedValueSerializationLimits::new(1, 0, 6),
        CssSpecifiedValueSerializationLimits::new(1, 1, 5),
    ] {
        assert!(matches!(
            snapshot.counter_name(&rule, limits),
            Err(CssomError::Value(_))
        ));
    }
    let block = snapshot.rule(&rule).unwrap().data().block().unwrap();
    assert!(matches!(
        snapshot
            .block(block)
            .unwrap()
            .counter_descriptors(CssSpecifiedValueSerializationLimits::new(0, 1000, 1000)),
        Err(CssomError::Value(_))
    ));
}

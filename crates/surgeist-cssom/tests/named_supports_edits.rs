//! Public tests for Surgeist's explicitly selected inert named-support policy.
use surgeist_css::*;
use surgeist_cssom::*;

fn captured_rule(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    id: &CssomRuleId,
) -> CssomCapture<surgeist_cssom::CssRule> {
    store.css_rule(id).unwrap().capture(snapshot).unwrap()
}

fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn setup(
    source: &str,
    limits: CssomLimits,
    inputs: CssomSheetInputs,
) -> (CssomStore, CssomSheetId, CssomRuleId) {
    let mut store = CssomStore::new(limits, CssomContext::default()).unwrap();
    let mut edit = batch(&store);
    let ticket = edit.create_parsed_sheet(source, inputs).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let id = store.snapshot().sheet_rules(&sheet).unwrap()[0].clone();
    (store, sheet, id)
}
fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "inert-tests".into(),
            revision: 1,
        },
        true,
    )
}
fn definition(body: &str) -> (CssomStore, CssomSheetId, CssomRuleId) {
    setup(
        &format!("@supports-condition --x {{ {body} }}"),
        CssomLimits::default(),
        inputs(),
    )
}
fn text(store: &CssomStore, snapshot: &CssomSnapshot, id: &CssomRuleId) -> String {
    captured_rule(store, snapshot, id)
        .css_text(&CssomRuleFormatLimits::default())
        .unwrap()
}
fn insert(store: &mut CssomStore, id: &CssomRuleId, source: &str, index: usize) -> CssomRuleId {
    let mut edit = batch(store);
    let ticket = edit.insert_group_rule(id, source, index).unwrap();
    store.commit(edit).unwrap().rule(&ticket).unwrap().clone()
}

#[test]
fn direct_payload_variants_preserve_unknown_candidates_and_immutable_nested_contents() {
    let (store, _, id) =
        definition("unknown: v!important; ??? { z: q; @future nested { a: b; } } @future value;");
    let snap = store.snapshot();
    let children = snap
        .rules(
            snap.rule(&id)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap();
    assert_eq!(children.len(), 3);
    let CssSupportsTestChild::Declaration(declaration) =
        snap.named_supports_test_child(&children[0]).unwrap()
    else {
        panic!("declaration")
    };
    assert_eq!(declaration.property(), "unknown");
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssSupportsTestChild::QualifiedRule(candidate) =
        snap.named_supports_test_child(&children[1]).unwrap()
    else {
        panic!("qualified")
    };
    assert_eq!(candidate.body().items().len(), 2);
    assert!(matches!(
        candidate.body().items()[1],
        CssSupportsTestItem::AtRule(_)
    ));
    assert!(matches!(
        snap.rule(&children[1])
            .and_then(|record| record.data().children().ok_or(CssomError::WrongKind)),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(
        snap.named_supports_test_child(&children[2]).unwrap(),
        CssSupportsTestChild::AtRule(_)
    ));
    assert_eq!(
        text(&store, &snap, &children[1]),
        "??? { z: q; @future nested { a: b; } }"
    );
}

#[test]
fn splitting_and_merging_css_runs_preserves_declaration_ids_and_provenance() {
    let (mut store, _, id) = definition("a: one; a: two;");
    let before = store.snapshot();
    let list = before
        .rule(&id)
        .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    let original = before.rules(&list).unwrap().to_vec();
    let origins: Vec<_> = original
        .iter()
        .map(|id| match before.named_supports_test_child(id).unwrap() {
            CssSupportsTestChild::Declaration(v) => v.origin().clone(),
            _ => panic!(),
        })
        .collect();
    let mut edit = batch(&store);
    let ticket = edit.insert_group_rule(&id, "& {}", 1).unwrap();
    assert_eq!(edit.named_supports_body(&id).unwrap().items().len(), 3);
    assert_eq!(
        edit.rule_css_text(&id, &CssomRuleFormatLimits::default())
            .unwrap(),
        "@supports-condition --x { a: one; & { } a: two; }"
    );
    assert_eq!(
        text(&store, &before, &id),
        "@supports-condition --x { a: one; a: two; }"
    );
    let commit = store.commit(edit).unwrap();
    let candidate = commit.rule(&ticket).unwrap().clone();
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("changed")
    };
    assert!(summary.categories().contains(&CssomChange::Modification));
    assert!(
        summary
            .affected()
            .contains(&CssomObjectId::Rule(id.clone()))
    );
    let mut edit = batch(&store);
    assert_eq!(edit.delete_group_rule(&id, 1).unwrap(), candidate);
    assert_eq!(edit.named_supports_body(&id).unwrap().items().len(), 1);
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        after
            .rule(&id)
            .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
            .unwrap(),
        &list
    );
    assert_eq!(after.rules(&list).unwrap(), original);
    assert_eq!(after.parent_rule(&candidate).unwrap(), None);
    assert_eq!(after.parent_style_sheet(&candidate).unwrap(), None);
    assert_eq!(text(&store, &after, &candidate), "& { }");
    assert_eq!(before.rules(&list).unwrap(), original);
    for (id, origin) in original.iter().zip(origins) {
        let CssSupportsTestChild::Declaration(value) = after.named_supports_test_child(id).unwrap()
        else {
            panic!()
        };
        assert_eq!(value.origin(), &origin);
    }
}

#[test]
fn accepted_single_child_forms_keep_complete_real_input_provenance() {
    for (source, expected) in [
        (";color:red;", "color: red;"),
        ("color:red;;", "color: red;"),
        ("/*a*/ color:red /*b*/", "color: red;"),
        ("& { ; }", "& { }"),
        ("@future", "@future;"),
        ("{color:red}", "{ color: red; }"),
    ] {
        let (mut store, _, id) = definition("");
        let child = insert(&mut store, &id, source, 0);
        let snap = store.snapshot();
        assert_eq!(text(&store, &snap, &child), expected);
        let origin = match snap.named_supports_test_child(&child).unwrap() {
            CssSupportsTestChild::Declaration(v) => v.origin(),
            CssSupportsTestChild::QualifiedRule(v) => v.origin(),
            CssSupportsTestChild::AtRule(v) => v.origin(),
        };
        let CssValueOrigin::Parsed(origin) = origin else {
            panic!("actual source")
        };
        assert_eq!(origin.source().as_str(), source);
    }
}

#[test]
fn empty_multiple_recovered_and_discarded_inputs_reject_and_roll_back_earlier_edits() {
    for source in [
        "",
        " /*x*/ ",
        ";;;",
        "a:b; c:d;",
        "& {",
        "@future {",
        "a:url(a b);",
        "& { bad:url(a b); }",
        "a:b; bad:url(a b);",
    ] {
        let (mut store, sheet, id) = definition("original: x;");
        let before = store.snapshot();
        let mut edit = batch(&store);
        edit.set_disabled(&sheet, true).unwrap();
        assert!(
            matches!(
                edit.insert_group_rule(&id, source, 0),
                Err(CssomError::Source(CssomException::Syntax))
            ),
            "{source:?}"
        );
        assert!(matches!(
            edit.rule_css_text(&id, &CssomRuleFormatLimits::default()),
            Err(CssomError::BatchAborted)
        ));
        assert!(matches!(
            edit.named_supports_body(&id),
            Err(CssomError::BatchAborted)
        ));
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert!(!store.snapshot().sheet(&sheet).unwrap().inputs().disabled);
        assert_eq!(
            text(&store, &store.snapshot(), &id),
            text(&store, &before, &id)
        );
    }
}

#[test]
fn insertion_start_and_end_and_atomic_replacement_preserve_sibling_ids() {
    let (mut store, _, id) = definition("old:x;");
    let before = store.snapshot();
    let list = before
        .rule(&id)
        .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    let old = before.rules(&list).unwrap()[0].clone();
    let first = insert(&mut store, &id, "first:a", 0);
    let last = insert(&mut store, &id, "last:z", 2);
    let mut edit = batch(&store);
    assert_eq!(edit.delete_group_rule(&id, 1).unwrap(), old);
    let ticket = edit.insert_group_rule(&id, "@future {}", 1).unwrap();
    let replacement = store.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    assert_ne!(replacement, old);
    let snap = store.snapshot();
    assert_eq!(snap.rules(&list).unwrap(), &[first, replacement, last]);
    assert_eq!(snap.parent_rule(&old).unwrap(), None);
    assert_eq!(before.parent_rule(&old).unwrap(), Some(&id));
    assert_eq!(
        text(&store, &snap, &id),
        "@supports-condition --x { first: a; @future  { } last: z; }"
    );
}

#[test]
fn recognized_at_rule_tests_remain_inert_opaque_leaves_without_namespace_or_host_effects() {
    let (mut store, sheet, id) = definition("");
    let before = store.snapshot();
    let mut edit = batch(&store);
    for (index, source) in [
        "@namespace n 'urn:n';",
        "@import 'x';",
        "@font-face {font-family:x;src:url(y)}",
        "@supports-condition --nested { z: q; }",
    ]
    .iter()
    .enumerate()
    {
        edit.insert_group_rule(&id, source, index).unwrap();
    }
    let commit = store.commit(edit).unwrap();
    assert!(commit.owner_effects().is_empty());
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("changed")
    };
    assert!(!summary.categories().contains(&CssomChange::Namespaces));
    assert!(!summary.categories().contains(&CssomChange::ImportLinks));
    let after = store.snapshot();
    assert_eq!(
        after.sheet(&sheet).unwrap().namespaces(),
        before.sheet(&sheet).unwrap().namespaces()
    );
    assert_eq!(after.sheets(), before.sheets());
    for child in after
        .rules(
            after
                .rule(&id)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap()
    {
        assert_eq!(captured_rule(&store, &after, child).r#type().unwrap(), 0);
        assert!(matches!(
            after
                .rule(child)
                .and_then(|record| record.data().block().ok_or(CssomError::WrongKind)),
            Err(CssomError::WrongKind)
        ));
        assert!(matches!(
            after
                .rule(child)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind)),
            Err(CssomError::WrongKind)
        ));
        assert!(matches!(
            after.named_supports_test_child(child).unwrap(),
            CssSupportsTestChild::AtRule(_)
        ));
        assert!(matches!(
            store
                .css_import_rule(child)
                .and_then(|handle| handle.capture(&after))
                .and_then(|capture| capture.href()),
            Err(CssomError::WrongKind)
        ));
    }
}

#[test]
fn escaped_names_and_test_tokens_use_css_owned_canonical_identifier_serialization() {
    let (store, _, id) = setup(
        r"@supports-condition --\78 { \63 olor: \72 ed; @\66 uture x; }",
        CssomLimits::default(),
        inputs(),
    );
    let snap = store.snapshot();
    assert_eq!(
        store
            .css_supports_condition_rule(&id)
            .and_then(|handle| handle.capture(&snap))
            .and_then(|capture| capture.name())
            .unwrap(),
        "--x"
    );
    assert_eq!(
        text(&store, &snap, &id),
        r"@supports-condition --x { color: \72 ed; @\66 uture x; }"
    );
    let children = snap
        .rules(
            snap.rule(&id)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap();
    let CssSupportsTestChild::Declaration(v) =
        snap.named_supports_test_child(&children[0]).unwrap()
    else {
        panic!("declaration")
    };
    assert_eq!(v.property(), "color");
}

#[test]
fn recovery_owned_by_removed_candidate_disappears_but_historical_diagnostics_remain() {
    let (mut store, sheet, id) = definition("& { bad:url(a b); }");
    let before = store.snapshot();
    assert!(
        before
            .named_supports_body(&id)
            .unwrap()
            .local_recovery_origins()
            .is_empty()
    );
    assert!(
        before
            .named_supports_body(&id)
            .unwrap()
            .recovery_origin()
            .is_some()
    );
    assert!(!before.sheet(&sheet).unwrap().diagnostics().is_empty());
    let historical = before.sheet(&sheet).unwrap().diagnostics().to_vec();
    let mut edit = batch(&store);
    let removed = edit.delete_group_rule(&id, 0).unwrap();
    assert!(
        edit.named_supports_body(&id)
            .unwrap()
            .recovery_origin()
            .is_none()
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert!(
        after
            .named_supports_body(&id)
            .unwrap()
            .recovery_origin()
            .is_none()
    );
    assert_eq!(after.sheet(&sheet).unwrap().diagnostics(), historical);
    assert!(
        before
            .named_supports_body(&id)
            .unwrap()
            .recovery_origin()
            .is_some()
    );
    assert_eq!(text(&store, &after, &id), "@supports-condition --x { }");
    assert_eq!(text(&store, &after, &removed), "& { }");
}

#[test]
fn outer_local_recovery_survives_deletion_of_a_recovered_candidate() {
    let (mut store, sheet, id) = definition("& { bad:url(a b); } bad:url(a b);");
    let before = store.snapshot();
    let local = before
        .named_supports_body(&id)
        .unwrap()
        .local_recovery_origins()
        .to_vec();
    assert!(!local.is_empty());
    let mut edit = batch(&store);
    edit.delete_group_rule(&id, 0).unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        after
            .named_supports_body(&id)
            .unwrap()
            .local_recovery_origins(),
        local
    );
    assert_eq!(
        after.named_supports_body(&id).unwrap().recovery_origin(),
        local.first()
    );
    assert_eq!(
        after.sheet(&sheet).unwrap().diagnostics(),
        before.sheet(&sheet).unwrap().diagnostics()
    );
    assert_eq!(text(&store, &after, &id), "@supports-condition --x { }");
}

#[test]
fn child_css_text_assignment_is_noop_even_for_malformed_text() {
    let (mut store, _, id) = definition("x:y;");
    let before = store.snapshot();
    let child = before
        .rules(
            before
                .rule(&id)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap()[0]
        .clone();
    let mut edit = batch(&store);
    edit.set_rule_css_text(&child, "@bad {").unwrap();
    let commit = store.commit(edit).unwrap();
    assert!(matches!(
        commit.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(text(&store, &store.snapshot(), &child), "x: y;");
}

#[test]
fn index_errors_precede_child_syntax_and_failed_delete_keeps_state() {
    let (mut store, _, id) = definition("x:y;");
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.insert_group_rule(&id, "& {", 2),
        Err(CssomError::Source(CssomException::IndexSize))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    let mut edit = batch(&store);
    assert!(matches!(
        edit.delete_group_rule(&id, 1),
        Err(CssomError::Source(CssomException::IndexSize))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(
        text(&store, &store.snapshot(), &id),
        text(&store, &before, &id)
    );
}

#[test]
fn security_and_modification_guards_precede_index_and_test_parsing() {
    for (origin_clean, locked, exception) in [
        (false, true, CssomException::Security),
        (true, true, CssomException::NotAllowed),
    ] {
        let mut facts = inputs();
        facts.origin_clean = origin_clean;
        facts.disallow_modification = locked;
        let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
        let mut edit = batch(&store);
        let sheet = edit
            .create_parsed_sheet("@supports-condition --x {a:b;}", facts)
            .unwrap();
        let sheet = store.commit(edit).unwrap().sheet(&sheet).unwrap().clone();
        let id = store
            .snapshot()
            .rules(store.snapshot().sheet(&sheet).unwrap().rules())
            .unwrap()[0]
            .clone();
        let mut edit = batch(&store);
        assert!(
            matches!(edit.insert_group_rule(&id, "&{", usize::MAX), Err(CssomError::Source(e)) if e == exception)
        );
        let mut edit = batch(&store);
        assert!(
            matches!(edit.delete_group_rule(&id, usize::MAX), Err(CssomError::Source(e)) if e == exception)
        );
    }
}

#[test]
fn canonical_text_reparse_preserves_payload_projection_not_original_trivia() {
    let (store, _, id) = definition("a:; a: x !IMPORTANT; & { ; @future; } @unknown /*x*/ ;");
    let canonical = text(&store, &store.snapshot(), &id);
    assert_eq!(
        canonical,
        "@supports-condition --x { a: ; a: x !important; & { @future; } @unknown /*x*/; }"
    );
    let (other, _, other_id) = setup(&canonical, CssomLimits::default(), inputs());
    assert_eq!(text(&other, &other.snapshot(), &other_id), canonical);
    assert_eq!(
        other
            .css_supports_condition_rule(&other_id)
            .unwrap()
            .css_rules(&other)
            .unwrap()
            .length(&other)
            .unwrap(),
        4
    );
}

#[test]
fn current_named_body_serializes_inside_ordinary_ancestor_and_staged_reads() {
    let (mut store, _, media) = setup(
        "@media all { @supports-condition --x {a:old;} }",
        CssomLimits::default(),
        inputs(),
    );
    let before = store.snapshot();
    let named = before
        .rules(
            before
                .rule(&media)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap()[0]
        .clone();
    let mut edit = batch(&store);
    edit.delete_group_rule(&named, 0).unwrap();
    edit.insert_group_rule(&named, "x:new", 0).unwrap();
    assert!(
        edit.rule_css_text(&media, &CssomRuleFormatLimits::default())
            .unwrap()
            .contains("@supports-condition --x { x: new; }")
    );
    assert!(text(&store, &before, &media).contains("a: old;"));
    store.commit(edit).unwrap();
    assert!(text(&store, &store.snapshot(), &media).contains("x: new;"));
    assert!(!text(&store, &store.snapshot(), &media).contains("a: old;"));
}

#[test]
fn nested_leaf_depth_limits_cover_components_and_ancestor_allowance_with_retry() {
    let (store, _, media) = setup(
        "@media all { @supports-condition --x { & { x:f(f(v)); } } }",
        CssomLimits::default(),
        inputs(),
    );
    let snap = store.snapshot();
    let named = snap
        .rules(
            snap.rule(&media)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap()[0]
        .clone();
    let child = snap
        .rules(
            snap.rule(&named)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap()[0]
        .clone();
    let expected = text(&store, &snap, &media);
    for target in [&media, &named, &child] {
        let mut low = CssomRuleFormatLimits {
            max_depth: 4,
            ..Default::default()
        };
        assert!(captured_rule(&store, &snap, target).css_text(&low).is_err());
        low.max_depth = 6;
        assert!(captured_rule(&store, &snap, target).css_text(&low).is_ok());
    }
    assert_eq!(text(&store, &snap, &media), expected);
    assert_eq!(store.snapshot().revision(), snap.revision());
}

#[test]
fn depth_zero_rejects_live_root_and_exact_allowance_accounts_for_actual_ancestors() {
    let (mut store, _, id) = definition("x:y;");
    let snap = store.snapshot();
    let child = snap
        .rules(
            snap.rule(&id)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap()[0]
        .clone();
    let mut limits = CssomRuleFormatLimits {
        max_depth: 0,
        ..Default::default()
    };
    assert!(captured_rule(&store, &snap, &id).css_text(&limits).is_err());
    limits.max_depth = 1;
    assert!(captured_rule(&store, &snap, &id).css_text(&limits).is_err());
    assert!(
        captured_rule(&store, &snap, &child)
            .css_text(&limits)
            .is_err()
    );
    limits.max_depth = 2;
    assert_eq!(
        captured_rule(&store, &snap, &id).css_text(&limits).unwrap(),
        "@supports-condition --x { x: y; }"
    );
    assert_eq!(
        captured_rule(&store, &snap, &child)
            .css_text(&limits)
            .unwrap(),
        "x: y;"
    );
    let mut edit = batch(&store);
    edit.delete_group_rule(&id, 0).unwrap();
    store.commit(edit).unwrap();
    limits.max_depth = 1;
    assert_eq!(
        captured_rule(&store, &store.snapshot(), &id)
            .css_text(&limits)
            .unwrap(),
        "@supports-condition --x { }"
    );
    assert_eq!(
        captured_rule(&store, &store.snapshot(), &child)
            .css_text(&limits)
            .unwrap(),
        "x: y;"
    );
    assert!(
        captured_rule(&store, &snap, &child)
            .css_text(&limits)
            .is_err()
    );
}

#[test]
fn read_work_output_and_live_node_budgets_fail_without_partial_output_or_state_change() {
    let (store, _, id) = definition("x:y; & {z:q;}");
    let snap = store.snapshot();
    for limits in [
        CssomRuleFormatLimits {
            max_nodes: 2,
            ..Default::default()
        },
        CssomRuleFormatLimits {
            max_preparation_steps: 1,
            ..Default::default()
        },
        CssomRuleFormatLimits {
            css: CssSpecifiedValueSerializationLimits::new(1000, 1000, 3),
            ..Default::default()
        },
    ] {
        assert!(captured_rule(&store, &snap, &id).css_text(&limits).is_err());
    }
    assert_eq!(store.snapshot().revision(), snap.revision());
    assert_eq!(
        text(&store, &snap, &id),
        "@supports-condition --x { x: y; & { z: q; } }"
    );
}

#[test]
fn failed_publication_preserves_current_recovery_and_old_identity_order() {
    let limits = CssomLimits {
        max_revision: 1,
        ..Default::default()
    };
    let (mut store, sheet, id) = setup(
        "@supports-condition --x { & {bad:url(a b);} bad:url(a b); }",
        limits,
        inputs(),
    );
    let before = store.snapshot();
    let list = before
        .rule(&id)
        .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    let child = before.rules(&list).unwrap()[0].clone();
    let recovery = before.named_supports_body(&id).unwrap().clone();
    let mut edit = batch(&store);
    edit.delete_group_rule(&id, 0).unwrap();
    assert_ne!(edit.named_supports_body(&id).unwrap(), &recovery);
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::RevisionExhausted)
    ));
    let after = store.snapshot();
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.named_supports_body(&id).unwrap(), &recovery);
    assert_eq!(after.rules(&list).unwrap(), std::slice::from_ref(&child));
    assert_eq!(after.parent_rule(&child).unwrap(), Some(&id));
    assert_eq!(
        after.sheet(&sheet).unwrap().diagnostics(),
        before.sheet(&sheet).unwrap().diagnostics()
    );
}

#[test]
fn deep_opaque_payload_admission_failure_rolls_back_earlier_sheet_edit() {
    let limits = CssomLimits {
        max_depth: 3,
        ..Default::default()
    };
    let (mut store, sheet, id) = setup("@supports-condition --x {}", limits, inputs());
    let before = store.snapshot();
    let mut edit = batch(&store);
    edit.set_disabled(&sheet, true).unwrap();
    let error = edit
        .insert_group_rule(&id, "& { & { x:f(f(v)); } }", 0)
        .unwrap_err();
    assert!(!matches!(error, CssomError::Source(CssomException::Syntax)));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    let after = store.snapshot();
    assert_eq!(after.revision(), before.revision());
    assert_eq!(
        store
            .css_rule_list(
                after
                    .rule(&id)
                    .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                    .unwrap()
            )
            .and_then(|list| list.capture(&after))
            .and_then(|list| list.length())
            .unwrap(),
        0
    );
    assert!(!after.sheet(&sheet).unwrap().inputs().disabled);
    assert_eq!(text(&store, &after, &id), text(&store, &before, &id));
}

#[test]
fn detached_outer_keeps_stable_children_and_accepts_inert_edits() {
    let (mut store, sheet, id) = definition("x:y;");
    let before = store.snapshot();
    let list = before
        .rule(&id)
        .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    let original = before.rules(&list).unwrap()[0].clone();
    let mut edit = batch(&store);
    assert_eq!(edit.delete_sheet_rule(&sheet, 0).unwrap(), id);
    store.commit(edit).unwrap();
    let added = insert(&mut store, &id, "@import 'x';", 1);
    let after = store.snapshot();
    assert_eq!(
        after
            .rule(&id)
            .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
            .unwrap(),
        &list
    );
    assert_eq!(
        after.rules(&list).unwrap(),
        &[original.clone(), added.clone()]
    );
    assert_eq!(after.parent_rule(&original).unwrap(), Some(&id));
    assert_eq!(after.parent_style_sheet(&original).unwrap(), None);
    assert_eq!(after.parent_style_sheet(&added).unwrap(), None);
    assert_eq!(before.parent_style_sheet(&original).unwrap(), Some(&sheet));
}

#[test]
fn typed_empty_definition_cannot_bypass_retained_input_byte_limit() {
    let source = format!("/*{}*/ @supports-condition --x {{}}", "padding".repeat(100));
    let sheet = parse_sheet(&source).into_parts().0;
    let mut store = CssomStore::new(
        CssomLimits {
            max_input_bytes: 100,
            ..Default::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.create_typed_sheet(sheet, inputs()),
        Err(CssomError::Limit {
            resource: "retained test input bytes",
            ..
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert!(store.snapshot().sheets().is_empty());
}

#[test]
fn cumulative_retained_entry_limit_aborts_multiple_small_insertions() {
    let (mut store, _, id) = setup(
        "@supports-condition --x {}",
        CssomLimits {
            max_entries: 20,
            ..Default::default()
        },
        inputs(),
    );
    let before = store.snapshot();
    let mut edit = batch(&store);
    edit.insert_group_rule(&id, "x:y", 0).unwrap();
    assert!(matches!(
        edit.insert_group_rule(&id, "z:q", 1),
        Err(CssomError::Limit { .. })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(
        store
            .css_supports_condition_rule(&id)
            .unwrap()
            .css_rules(&store)
            .unwrap()
            .length(&store)
            .unwrap(),
        0
    );
}

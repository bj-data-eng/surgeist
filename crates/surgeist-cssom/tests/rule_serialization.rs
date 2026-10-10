use surgeist_css::*;
use surgeist_cssom::*;

#[test]
fn edited_font_families_and_counter_name_keep_native_wrapper_limits() {
    let (mut store, _, roots) = fixture(
        "@font-feature-values Original { @styleset { fancy: 1; } } @counter-style original { system: cyclic; symbols: a; }",
    );
    let before = store.snapshot();
    let mut edit = batch(&store);
    edit.set_font_feature_family_text(&roots[0], "Current")
        .unwrap();
    edit.set_counter_name(&roots[1], "current").unwrap();
    store.commit(edit).unwrap();
    let current = store.snapshot();
    assert_eq!(
        current
            .font_feature_values(&roots[0])
            .unwrap()
            .font_family(CssSpecifiedValueSerializationLimits::default())
            .unwrap(),
        "Current"
    );
    assert_eq!(
        before
            .font_feature_values(&roots[0])
            .unwrap()
            .font_family(CssSpecifiedValueSerializationLimits::default())
            .unwrap(),
        "Original"
    );
    assert_eq!(
        current
            .counter_name(&roots[1], CssSpecifiedValueSerializationLimits::default())
            .unwrap(),
        "current"
    );
    assert_eq!(
        before
            .counter_name(&roots[1], CssSpecifiedValueSerializationLimits::default())
            .unwrap(),
        "original"
    );
    for snapshot in [&before, &current] {
        for (rule, kind) in roots.iter().zip([
            CssRuleCssomFormat::FontFeatureValues,
            CssRuleCssomFormat::CounterStyle,
        ]) {
            assert!(matches!(
                snapshot.rule_css_text(rule, &CssomRuleFormatLimits::default()),
                Err(CssomError::Format(ref error)) if error.kind() == CssRuleCssomSerializationErrorKind::FormatUnavailable(kind)
            ));
        }
    }
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn fixture(source: &str) -> (CssomStore, CssomSheetId, Vec<CssomRuleId>) {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut inputs = CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "document".into(),
            revision: 1,
        },
        "document".into(),
        None,
    );
    inputs.constructed = false;
    let mut edit = batch(&store);
    let ticket = edit.create_parsed_sheet(source, inputs).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let roots = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()
        .to_vec();
    (store, sheet, roots)
}
fn children(snapshot: &CssomSnapshot, id: &CssomRuleId) -> Vec<CssomRuleId> {
    snapshot
        .rules(snapshot.child_rules(id).unwrap())
        .unwrap()
        .to_vec()
}

#[test]
fn recursive_text_uses_current_selectors_selected_survivors_order_and_media() {
    let (mut store, _, roots) = fixture(
        "@media screen { a { margin: 1px 2px; & b { color: red; } color: blue; & c { width: 1px; } } }",
    );
    let before = store.snapshot();
    let style = children(&before, &roots[0])[0].clone();
    let old = children(&before, &style);
    let nested = old[1].clone();
    let block = before.rule_style(&style).unwrap().clone();
    let nested_block = before.rule_style(&nested).unwrap().clone();
    let media = before.group_media(&roots[0]).unwrap().clone();
    let mut edit = batch(&store);
    edit.set_style_selector_text(&style, "x").unwrap();
    edit.remove_selected_terminal(
        &block,
        CssPropertyNameRef::Known(CssKnownProperty::MarginLeft),
    )
    .unwrap();
    edit.remove_selected_terminal(
        &nested_block,
        CssPropertyNameRef::Known(CssKnownProperty::Color),
    )
    .unwrap();
    edit.delete_group_rule(&style, 0).unwrap();
    let ticket = edit
        .insert_group_rule(&style, "& d { height: 2px; }", 2)
        .unwrap();
    edit.set_media(&media, parse_media_query_list("print").into_parts().0)
        .unwrap();
    let inserted = store.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    let current = store.snapshot();
    assert_eq!(
        children(&current, &style),
        vec![nested.clone(), old[2].clone(), inserted]
    );
    assert_eq!(current.rule_style(&nested).unwrap(), &nested_block);
    assert_eq!(
        current
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        // CSSOM appends each child's serialized text after its initial indent;
        // it does not reindent every line of an already serialized child.
        "@media print {\n  x {\n  margin-top: 1px; margin-right: 2px; margin-bottom: 1px;\n  & c { width: 1px; }\n  & d { height: 2px; }\n}\n}"
    );
    assert!(
        before
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap()
            .contains("a {")
    );
    assert_eq!(current.parent_rule(&old[0]).unwrap(), None);
    assert_eq!(current.parent_style_sheet(&old[0]).unwrap(), None);
}

#[test]
fn empty_nested_declaration_identity_is_kept_while_its_text_is_filtered() {
    let (mut store, _, roots) = fixture("a { @media screen { color: red; } }");
    let before = store.snapshot();
    let media = children(&before, &roots[0])[0].clone();
    let nested = children(&before, &media)[0].clone();
    let block = before.rule_style(&nested).unwrap().clone();
    let mut edit = batch(&store);
    edit.remove_selected_terminal(&block, CssPropertyNameRef::Known(CssKnownProperty::Color))
        .unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    assert_eq!(
        snapshot
            .rule_css_text(&nested, &CssomRuleFormatLimits::default())
            .unwrap(),
        ""
    );
    assert_eq!(
        snapshot
            .rule_css_text(&media, &CssomRuleFormatLimits::default())
            .unwrap(),
        "@media screen {\n\n}"
    );
    assert_eq!(children(&snapshot, &media), vec![nested]);
    assert_eq!(
        snapshot
            .rule_style(&children(&snapshot, &media)[0])
            .unwrap(),
        &block
    );
}

#[test]
fn current_import_media_without_child_sheet_is_serialized_with_genuine_clauses() {
    let (mut store, _, roots) = fixture("@import 'child.css' screen;");
    let before = store.snapshot();
    let media = before.import_media(&roots[0]).unwrap().clone();
    let mut edit = batch(&store);
    edit.set_media(&media, parse_media_query_list("print").into_parts().0)
        .unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    assert_eq!(snapshot.import_style_sheet(&roots[0]).unwrap(), None);
    assert_eq!(snapshot.import_href(&roots[0]).unwrap(), "child.css");
    assert_eq!(
        snapshot
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        "@import url(\"child.css\") print;"
    );
    assert_eq!(
        before
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        "@import url(\"child.css\") screen;"
    );
}

#[test]
fn independent_work_and_native_output_failures_preserve_capture_and_retry() {
    let (store, _, roots) = fixture(
        "@supports (display: grid) { @media screen { a { color: red; } b { width: 1px; } } }",
    );
    let snapshot = store.snapshot();
    let normal = snapshot
        .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
        .unwrap();
    let tiny = CssomRuleFormatLimits {
        max_preparation_steps: 1,
        ..CssomRuleFormatLimits::default()
    };
    assert!(matches!(
        snapshot.rule_css_text(&roots[0], &tiny),
        Err(CssomError::Limit {
            resource: "rule format preparation work",
            ..
        })
    ));
    let native = CssomRuleFormatLimits {
        css: CssSpecifiedValueSerializationLimits::new(1, 1, normal.len()),
        ..CssomRuleFormatLimits::default()
    };
    assert!(matches!(
        snapshot.rule_css_text(&roots[0], &native),
        Err(CssomError::Format(_))
    ));
    let short = CssomRuleFormatLimits {
        css: CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, normal.len() - 1),
        ..CssomRuleFormatLimits::default()
    };
    assert!(matches!(
        snapshot.rule_css_text(&roots[0], &short),
        Err(CssomError::Format(_))
    ));
    assert_eq!(
        snapshot
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        normal
    );
    assert_eq!(snapshot.revision(), store.snapshot().revision());
}

#[test]
fn preparation_bounds_queued_siblings_and_actual_ancestor_context_without_changing_state() {
    let (store, _, roots) = fixture("@media screen { a {} b {} c {} }");
    let snapshot = store.snapshot();
    let limits = CssomRuleFormatLimits {
        max_nodes: 1,
        ..CssomRuleFormatLimits::default()
    };
    assert!(matches!(
        snapshot.rule_css_text(&roots[0], &limits),
        Err(CssomError::Limit {
            resource: "rule format nodes",
            maximum: 1
        })
    ));
    let children = children(&snapshot, &roots[0]);
    let shallow = CssomRuleFormatLimits {
        max_depth: 1,
        ..CssomRuleFormatLimits::default()
    };
    assert!(matches!(
        snapshot.rule_css_text(&children[0], &shallow),
        Err(CssomError::Limit {
            resource: "rule format preparation depth",
            maximum: 1
        })
    ));
    assert_eq!(
        snapshot
            .rule_css_text(&children[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        "a { }"
    );
    assert_eq!(snapshot.revision(), store.snapshot().revision());
    assert_eq!(
        snapshot
            .rules(snapshot.child_rules(&roots[0]).unwrap())
            .unwrap(),
        children
    );
}

#[test]
fn unavailable_wrapper_is_lazy_and_reports_actual_path_without_preparing_descendants() {
    let (store, _, roots) =
        fixture("@media screen { @scope (.outer) { a { color: red; } b { width: 1px; } } }");
    let snapshot = store.snapshot();
    let scope = children(&snapshot, &roots[0])[0].clone();
    let limits = CssomRuleFormatLimits {
        max_nodes: 1,
        ..CssomRuleFormatLimits::default()
    };
    assert!(
        matches!(snapshot.rule_css_text(&scope, &limits), Err(CssomError::Format(ref error)) if error.kind() == CssRuleCssomSerializationErrorKind::FormatUnavailable(CssRuleCssomFormat::Scope))
    );
    let Err(CssomError::Format(error)) =
        snapshot.rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
    else {
        panic!("typed unavailable");
    };
    assert_eq!(
        error.kind(),
        CssRuleCssomSerializationErrorKind::FormatUnavailable(CssRuleCssomFormat::Scope)
    );
    assert_eq!(error.rule_path(), &[0]);
    assert_eq!(
        snapshot
            .scope_start(&scope, CssSpecifiedValueSerializationLimits::default())
            .unwrap()
            .as_deref(),
        Some(".outer")
    );
}

#[test]
fn current_keyframes_name_key_text_membership_and_declarations_are_independent_of_authored_rule() {
    let (mut store, _, roots) =
        fixture("@keyframes old { from { opacity: 0; width: 1px; } to { opacity: 1; } }");
    let before = store.snapshot();
    let old = children(&before, &roots[0]);
    let block = before.rule_style(&old[0]).unwrap().clone();
    let mut edit = batch(&store);
    edit.set_keyframes_name(&roots[0], "new name").unwrap();
    edit.set_keyframe_key_text(&old[0], "50%").unwrap();
    edit.remove_property(&block, "width", CssomDeclarationRequestLimits::default())
        .unwrap();
    let appended = edit
        .append_keyframe_rule(&roots[0], "25% { opacity: 0.25; }")
        .unwrap()
        .unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    assert_eq!(
        snapshot
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        // The selected algorithm serializes ordinary names as identifiers and
        // puts the first indented rule after the literal " { " prefix.
        "@keyframes new\\ name {   50% { opacity: 0; }\n  100% { opacity: 1; }\n  25% { opacity: 0.25; }\n}"
    );
    assert_eq!(
        snapshot
            .rule_css_text(&old[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        "50% { opacity: 0; }"
    );
    assert_eq!(snapshot.rule_style(&old[0]).unwrap(), &block);
    // A ticket remains opaque after publication; actual current membership owns IDs.
    drop(appended);
    assert_eq!(children(&snapshot, &roots[0]).len(), 3);
    assert!(
        before
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap()
            .starts_with("@keyframes old")
    );
}

#[test]
fn current_page_selection_and_margin_child_edits_preserve_descriptor_and_block_identity() {
    let (mut store, _, roots) = fixture(
        "@page old:left { size: A4; color: red; @top-left { color: blue; width: 1px; } @bottom-center { color: green; } }",
    );
    let before = store.snapshot();
    let page_block = before.rule_style(&roots[0]).unwrap().clone();
    let old = children(&before, &roots[0]);
    let margin_block = before.rule_style(&old[0]).unwrap().clone();
    let mut edit = batch(&store);
    edit.set_page_selector_text(&roots[0], "new:right").unwrap();
    edit.remove_property(
        &page_block,
        "color",
        CssomDeclarationRequestLimits::default(),
    )
    .unwrap();
    edit.remove_property(
        &margin_block,
        "width",
        CssomDeclarationRequestLimits::default(),
    )
    .unwrap();
    edit.delete_group_rule(&roots[0], 1).unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    assert_eq!(
        snapshot
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        "@page new:right { size: a4; @top-left { color: blue; } }"
    );
    assert_eq!(
        snapshot
            .rule_css_text(&old[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        "@top-left { color: blue; }"
    );
    assert_eq!(snapshot.rule_style(&roots[0]).unwrap(), &page_block);
    assert_eq!(snapshot.rule_style(&old[0]).unwrap(), &margin_block);
    assert_eq!(snapshot.parent_rule(&old[1]).unwrap(), None);
    assert!(
        before
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap()
            .contains("@bottom-center")
    );
}

#[test]
fn font_face_current_selected_descriptor_removal_does_not_expand_original_occurrences() {
    let (mut store, _, roots) =
        fixture("@font-face { font-family: Example; font-weight: 300; font-weight: 700; }");
    let before = store.snapshot();
    let block = before.rule_style(&roots[0]).unwrap().clone();
    let mut edit = batch(&store);
    assert_eq!(
        edit.remove_property(
            &block,
            "font-weight",
            CssomDeclarationRequestLimits::default()
        )
        .unwrap(),
        "700"
    );
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    assert_eq!(
        snapshot
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        "@font-face { font-family: Example; }"
    );
    assert_eq!(snapshot.rule_style(&roots[0]).unwrap(), &block);
    assert!(
        before
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap()
            .contains("font-weight: 700;")
    );
    let CssomBlockData::FontFace(raw) = snapshot.block(&block).unwrap().data() else {
        panic!("raw facet");
    };
    assert_eq!(raw.occurrences().len(), 3);
}

#[test]
fn custom_media_current_query_keeps_native_unavailable_wrapper_boundary() {
    let (mut store, _, roots) =
        fixture("@custom-media --narrow screen; @custom-media --always true;");
    let before = store.snapshot();
    let CssomCustomMediaQuery::MediaList(media) = before.custom_media_query(&roots[0]).unwrap()
    else {
        panic!("current query list");
    };
    let media = media.clone();
    let mut edit = batch(&store);
    edit.set_media_text(&media, "print").unwrap();
    store.commit(edit).unwrap();
    let current = store.snapshot();
    assert_eq!(
        current
            .media_text(&media, CssSpecifiedValueSerializationLimits::default())
            .unwrap(),
        "print"
    );
    assert_eq!(
        before
            .media_text(&media, CssSpecifiedValueSerializationLimits::default())
            .unwrap(),
        "screen"
    );
    for snapshot in [&before, &current] {
        for rule in &roots {
            let error = snapshot
                .rule_css_text(rule, &CssomRuleFormatLimits::default())
                .unwrap_err();
            assert!(
                matches!(error, CssomError::Format(ref error) if error.kind() == CssRuleCssomSerializationErrorKind::FormatUnavailable(CssRuleCssomFormat::CustomMedia))
            );
        }
    }
    let CssomAuthoredRule::Ordinary(CssRule::CustomMedia(original)) =
        current.rule(&roots[0]).unwrap().authored()
    else {
        panic!("genuine original");
    };
    let CssCustomMediaBody::Media(original_query) = original.body() else {
        panic!("original query");
    };
    assert_eq!(original_query.serialize_cssom().unwrap().as_css(), "screen");
}

#[test]
fn supports_writer_consumes_current_selectors_declarations_and_child_membership() {
    let (mut store, _, roots) = fixture("@supports (display:grid) { a { color: red; } }");
    let before = store.snapshot();
    let child = children(&before, &roots[0])[0].clone();
    let block = before.rule_style(&child).unwrap().clone();
    let mut edit = batch(&store);
    edit.set_style_selector_text(&child, "x").unwrap();
    edit.remove_property(&block, "color", CssomDeclarationRequestLimits::default())
        .unwrap();
    let ticket = edit
        .insert_group_rule(&roots[0], "b { width: 1px; }", 1)
        .unwrap();
    let inserted = store.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    let current = store.snapshot();
    assert_eq!(children(&current, &roots[0]), [child.clone(), inserted]);
    assert_eq!(current.rule_style(&child).unwrap(), &block);
    assert_eq!(
        current
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        "@supports (display:grid) {\n  x { }\n  b { width: 1px; }\n}"
    );
    assert_eq!(
        before
            .rule_css_text(&roots[0], &CssomRuleFormatLimits::default())
            .unwrap(),
        "@supports (display:grid) {\n  a { color: red; }\n}"
    );
    let CssomAuthoredRule::Ordinary(original) = current.rule(&roots[0]).unwrap().authored() else {
        panic!("genuine original");
    };
    assert_eq!(
        original.serialize_cssom().unwrap(),
        "@supports (display:grid) {\n  a { color: red; }\n}"
    );
}

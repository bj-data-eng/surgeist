use surgeist_css::*;
use surgeist_cssom::*;
fn store() -> CssomStore {
    CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn sheet(store: &mut CssomStore, source: &str, constructed: bool) -> CssomSheetId {
    let mut inputs = CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "document".into(),
            revision: 1,
        },
        "document".into(),
        None,
    );
    inputs.constructed = constructed;
    let mut edit = batch(store);
    let ticket = edit.create_parsed_sheet(source, inputs).unwrap();
    store.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn roots(snapshot: &CssomSnapshot, sheet: &CssomSheetId) -> Vec<CssomRuleId> {
    snapshot
        .rules(snapshot.sheet(sheet).unwrap().rules())
        .unwrap()
        .to_vec()
}
fn children(snapshot: &CssomSnapshot, rule: &CssomRuleId) -> Vec<CssomRuleId> {
    snapshot
        .rules(snapshot.rule(rule).unwrap().data().children().unwrap())
        .unwrap()
        .to_vec()
}
fn source_error<T: std::fmt::Debug>(result: Result<T, CssomError>, expected: CssomException) {
    assert!(
        matches!(&result, Err(CssomError::Source(actual)) if *actual == expected),
        "expected {expected:?}, actual {result:?}"
    );
}

#[test]
fn live_list_resolves_same_identity_while_capture_and_detached_links_stay_fixed() {
    let mut owner = store();
    let sheet = sheet(&mut owner, "@media screen { a { color: red } }", true);
    let before = owner.snapshot();
    let root = roots(&before, &sheet)[0].clone();
    let list = before
        .rule(&root)
        .unwrap()
        .data()
        .children()
        .unwrap()
        .clone();
    let old = children(&before, &root)[0].clone();
    let mut edit = batch(&owner);
    let ticket = edit
        .insert_group_rule(&root, "b { width: 1px }", 1)
        .unwrap();
    let new = owner.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    assert_eq!(
        owner
            .css_rule_list(&list)
            .and_then(|list| list.length(&owner))
            .unwrap(),
        2
    );
    assert_eq!(
        owner
            .css_rule_list(&list)
            .and_then(|list| list.item(&owner, 1))
            .map(|item| item.map(|rule| rule.id().clone()))
            .unwrap(),
        Some(new.clone())
    );
    assert_eq!(
        owner
            .css_rule_list(&list)
            .and_then(|list| list.item(&owner, usize::MAX))
            .map(|item| item.map(|rule| rule.id().clone()))
            .unwrap(),
        None
    );
    assert_eq!(
        owner
            .css_rule_list(&list)
            .and_then(|list| list.capture(&before))
            .and_then(|list| list.length())
            .unwrap(),
        1
    );
    assert_eq!(
        owner
            .css_rule_list(&list)
            .and_then(|list| list.capture(&before))
            .and_then(|list| list.item(1))
            .map(|item| item.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        None
    );
    let mut edit = batch(&owner);
    edit.delete_sheet_rule(&sheet, 0).unwrap();
    owner.commit(edit).unwrap();
    let detached = owner.snapshot();
    assert_eq!(detached.parent_rule(&old).unwrap(), Some(&root));
    assert_eq!(detached.parent_style_sheet(&old).unwrap(), None);
    assert_eq!(detached.parent_rule(&new).unwrap(), Some(&root));
    assert_eq!(
        owner
            .css_rule_list(&list)
            .and_then(|list| list.length(&owner))
            .unwrap(),
        2
    );
}
#[test]
fn sheet_preliminary_checks_and_group_index_have_distinct_source_precedence() {
    let mut owner = store();
    let sheet = sheet(&mut owner, "a {}", true);
    let root = roots(&owner.snapshot(), &sheet)[0].clone();
    let mut edit = batch(&owner);
    source_error(
        edit.insert_sheet_rule(&sheet, "garbage", usize::MAX),
        CssomException::Syntax,
    );
    let mut edit = batch(&owner);
    source_error(
        edit.insert_sheet_rule(&sheet, "@import 'x';", usize::MAX),
        CssomException::Syntax,
    );
    let mut edit = batch(&owner);
    source_error(
        edit.insert_sheet_rule(&sheet, "b {}", usize::MAX),
        CssomException::IndexSize,
    );
    let mut edit = batch(&owner);
    source_error(
        edit.insert_group_rule(&root, "garbage", usize::MAX),
        CssomException::IndexSize,
    );
    let mut edit = batch(&owner);
    source_error(
        edit.insert_group_rule(&root, "@namespace p 'urn:p';", 0),
        CssomException::HierarchyRequest,
    );
    let mut edit = batch(&owner);
    source_error(
        edit.insert_group_rule(&root, "@namespace p;", 0),
        CssomException::Syntax,
    );
}
#[test]
fn initial_layer_and_whole_list_namespace_state_are_separate() {
    let mut owner = store();
    let sheet = sheet(&mut owner, "", false);
    let mut edit = batch(&owner);
    edit.insert_sheet_rule(&sheet, "@layer seed;", 0).unwrap();
    edit.insert_sheet_rule(&sheet, "@import 'x';", 1).unwrap();
    owner.commit(edit).unwrap();
    let mut edit = batch(&owner);
    source_error(
        edit.insert_sheet_rule(&sheet, "@namespace p 'urn:p';", 2),
        CssomException::InvalidState,
    );
    let mut edit = batch(&owner);
    let (_, ticket) = edit.insert_sheet_rule(&sheet, "@import 'y';", 1).unwrap();
    let added = owner.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    assert_eq!(roots(&owner.snapshot(), &sheet)[1], added);
    let mut edit = batch(&owner);
    edit.insert_sheet_rule(&sheet, "b {}", 3).unwrap();
    owner.commit(edit).unwrap();
    let mut edit = batch(&owner);
    source_error(
        edit.insert_sheet_rule(&sheet, "@import 'z';", 4),
        CssomException::HierarchyRequest,
    );
}
#[test]
fn nested_raw_fallback_keeps_occurrences_block_identity_and_flags() {
    let mut owner = store();
    let sheet = sheet(&mut owner, "a {}", true);
    let root = roots(&owner.snapshot(), &sheet)[0].clone();
    let text = "/* actual */ color: red; color: blue !important;";
    let mut edit = batch(&owner);
    let ticket = edit.insert_group_rule(&root, text, 0).unwrap();
    let id = owner.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    let snapshot = owner.snapshot();
    let rule = snapshot.rule(&id).unwrap();
    let CssomAuthoredRule::RawNestedDeclarations(authored) = rule.authored() else {
        panic!("raw occurrence");
    };
    assert_eq!(authored.len(), 2);
    assert_eq!(authored[0].parsed_value().unwrap().source().as_str(), text);
    let block = rule.data().block().unwrap();
    assert_eq!(snapshot.block(block).unwrap().parent(), Some(&id));
    assert!(!snapshot.block(block).unwrap().flags().computed);
    assert!(!snapshot.block(block).unwrap().flags().readonly);
    assert_eq!(snapshot.block(block).unwrap().owner(), None);
    assert_eq!(
        snapshot
            .specified_property_css_text(block, CssSerializationLimits::default())
            .unwrap(),
        "color: blue !important;"
    );
    let mut edit = batch(&owner);
    source_error(
        edit.insert_group_rule(&root, "bogus: value;", 1),
        CssomException::Syntax,
    );
}
#[test]
fn page_insertion_uses_same_detached_candidate_and_rejects_nonmargin_hierarchy() {
    let mut owner = store();
    let sheet = sheet(&mut owner, "@page :left { size: A4; }", true);
    let page = roots(&owner.snapshot(), &sheet)[0].clone();
    let text = " /* prefix */ @top-left { content: 'hello'; } /* suffix */ ";
    let mut edit = batch(&owner);
    let ticket = edit.insert_group_rule(&page, text, 0).unwrap();
    let id = owner.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    let snapshot = owner.snapshot();
    assert_eq!(
        owner
            .css_rule(&id)
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.r#type())
            .unwrap(),
        9
    );
    let CssomAuthoredRule::Margin(authored) = snapshot.rule(&id).unwrap().authored() else {
        panic!("margin");
    };
    assert_eq!(authored.detached_origin().unwrap().source().as_str(), text);
    assert_eq!(snapshot.parent_rule(&id).unwrap(), Some(&page));
    let mut edit = batch(&owner);
    source_error(
        edit.insert_group_rule(&page, "a {}", 1),
        CssomException::HierarchyRequest,
    );
}
#[test]
fn selector_success_keeps_block_and_children_and_records_new_occurrence_even_same_text() {
    let mut owner = store();
    let sheet = sheet(&mut owner, "a { color: red; & b {} }", true);
    let before = owner.snapshot();
    let root = roots(&before, &sheet)[0].clone();
    let block = before.rule(&root).unwrap().data().block().unwrap().clone();
    let child = children(&before, &root)[0].clone();
    let text = " /* original */ .changed ";
    let mut edit = batch(&owner);
    assert!(edit.set_style_selector_text(&root, text).unwrap());
    owner.commit(edit).unwrap();
    let first = owner.snapshot();
    assert_eq!(
        owner
            .css_style_rule(&root)
            .unwrap()
            .capture(&first)
            .unwrap()
            .selector_text(CssSerializationLimits::default())
            .unwrap(),
        ".changed"
    );
    assert_eq!(first.rule(&root).unwrap().data().block(), Some(&block));
    assert_eq!(children(&first, &root), [child]);
    assert_eq!(
        first
            .rule(&root)
            .unwrap()
            .selector_input()
            .unwrap()
            .source()
            .as_str(),
        text
    );
    let mut edit = batch(&owner);
    assert!(edit.set_style_selector_text(&root, text).unwrap());
    let commit = owner.commit(edit).unwrap();
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("fresh successful parse publishes provenance");
    };
    assert!(summary.categories().contains(&CssomChange::Selectors));
    assert!(summary.categories().contains(&CssomChange::Provenance));
    let mut edit = batch(&owner);
    assert!(!edit.set_style_selector_text(&root, ".bad,").unwrap());
    assert!(matches!(
        owner.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
}
#[test]
fn public_keyframe_offset_probe_distinguishes_exact_literal_data_and_symbolic_provenance() {
    let parse = |text| {
        parse_keyframe_selector_list(text)
            .into_parts()
            .0
            .unwrap()
            .selectors()
            .selectors()[0]
            .offset()
    };
    assert_eq!(parse("from"), parse("0%"));
    assert_eq!(parse("to"), parse("100%"));
    let a = parse("10.00000000001%");
    let b = parse("10.00000000002%");
    assert_ne!(a.literal_value(), b.literal_value());
    let a = parse("calc(10%)");
    let b = parse("calc( 10% )");
    assert!(a.literal_value().is_none());
    assert!(a.calculation().is_some());
    assert_eq!(
        a, b,
        "calculation equality compares numeric values, not source inputs or spacing"
    );
}
#[test]
fn partial_getters_remain_independent_of_unavailable_wrappers_and_host_links() {
    let mut owner = store();
    let sheet = sheet(
        &mut owner,
        "@import 'x' layer(base) supports(display: grid) screen; @namespace 'urn:default'; @scope (.start) to (.end) { a {} } @media print {} @supports (display: grid) {} @page { @bottom-right { content: 'x'; } }",
        false,
    );
    let snapshot = owner.snapshot();
    let roots = roots(&snapshot, &sheet);
    assert_eq!(roots.len(), 6);
    assert_eq!(
        owner
            .css_import_rule(&roots[0])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.href())
            .unwrap(),
        "x"
    );
    assert_eq!(
        owner
            .css_import_rule(&roots[0])
            .unwrap()
            .capture(&snapshot)
            .unwrap()
            .style_sheet()
            .unwrap()
            .map(|sheet| sheet.handle().id().clone()),
        None
    );
    let media = owner
        .css_import_rule(&roots[0])
        .unwrap()
        .capture(&snapshot)
        .unwrap()
        .media()
        .unwrap();
    assert_eq!(
        snapshot
            .media(media.handle().id())
            .unwrap()
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        "screen"
    );
    assert_eq!(
        owner
            .css_import_rule(&roots[0])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.supports_text(1024))
            .unwrap()
            .as_deref(),
        Some("(display: grid)")
    );
    assert_eq!(
        owner
            .css_namespace_rule(&roots[1])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.namespace_uri())
            .unwrap(),
        "urn:default"
    );
    assert_eq!(
        owner
            .css_namespace_rule(&roots[1])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.prefix())
            .unwrap(),
        ""
    );
    let limits = CssSerializationLimits::default();
    assert_eq!(
        owner
            .css_scope_rule(&roots[2])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.start(limits))
            .unwrap()
            .as_deref(),
        Some(".start")
    );
    assert_eq!(
        owner
            .css_scope_rule(&roots[2])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.end(limits))
            .unwrap()
            .as_deref(),
        Some(".end")
    );
    assert_eq!(
        owner
            .css_rule(&roots[2])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.r#type())
            .unwrap(),
        0
    );
    assert_eq!(
        owner
            .css_condition_rule(&roots[3])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.condition_text(limits))
            .unwrap(),
        "print"
    );
    assert_eq!(
        owner
            .css_rule(&roots[3])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.r#type())
            .unwrap(),
        4
    );
    assert_eq!(
        owner
            .css_condition_rule(&roots[4])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.condition_text(limits))
            .unwrap(),
        " (display: grid) "
    );
    assert_eq!(
        owner
            .css_rule(&roots[4])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.r#type())
            .unwrap(),
        12
    );
    let margin = children(&snapshot, &roots[5])[0].clone();
    assert_eq!(
        owner
            .css_margin_rule(&margin)
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.name())
            .unwrap(),
        "bottom-right"
    );
    let block = snapshot
        .rule(&margin)
        .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
        .unwrap();
    assert_eq!(snapshot.block(block).unwrap().parent(), Some(&margin));
    assert!(
        owner
            .css_import_rule(&roots[2])
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.href())
            .is_err()
    );
}
#[test]
fn keyframe_append_invalid_key_edit_and_raw_name_have_atomic_publication() {
    let mut owner = store();
    let sheet = sheet(&mut owner, "@keyframes first { from { opacity: 0 } }", true);
    let before = owner.snapshot();
    let keys = roots(&before, &sheet)[0].clone();
    let first = children(&before, &keys)[0].clone();
    let list = before
        .rule(&keys)
        .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    let mut edit = batch(&owner);
    let ticket = edit
        .append_keyframe_rule(&keys, "0% { opacity: 1 }")
        .unwrap()
        .unwrap();
    let appended = owner.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    assert_ne!(first, appended);
    let after = owner.snapshot();
    assert_eq!(
        owner
            .css_keyframes_rule(&keys)
            .and_then(|handle| handle.capture(&after))
            .and_then(|capture| capture.length())
            .unwrap(),
        2
    );
    assert_eq!(
        after
            .rule(&keys)
            .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
            .unwrap(),
        &list
    );
    assert_eq!(
        owner
            .css_keyframes_rule(&keys)
            .unwrap()
            .indexed(&owner, 2)
            .unwrap(),
        CssomKeyframeIndex::Undefined
    );
    assert_eq!(
        owner
            .css_keyframes_rule(&keys)
            .unwrap()
            .indexed(&owner, 1)
            .unwrap(),
        CssomKeyframeIndex::Rule(owner.css_keyframe_rule(&appended).unwrap())
    );
    let mut edit = batch(&owner);
    assert!(
        edit.append_keyframe_rule(&keys, "10% {} 20% {}")
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        owner.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    let block = after
        .rule(&first)
        .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    let mut edit = batch(&owner);
    edit.set_keyframes_name(&keys, "temporary").unwrap();
    source_error(
        edit.set_keyframe_key_text(&first, "bad"),
        CssomException::Syntax,
    );
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    let unchanged = owner.snapshot();
    assert_eq!(unchanged.revision(), after.revision());
    assert_eq!(
        owner
            .css_keyframes_rule(&keys)
            .and_then(|handle| handle.capture(&unchanged))
            .and_then(|capture| capture.name())
            .unwrap(),
        "first"
    );
    assert_eq!(
        unchanged
            .rule(&first)
            .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
            .unwrap(),
        &block
    );
    assert_eq!(
        owner
            .css_keyframe_rule(&first)
            .and_then(|handle| handle.capture(&unchanged))
            .and_then(|capture| capture.key_text(CssSerializationLimits::default()))
            .unwrap(),
        "0%"
    );
    let mut edit = batch(&owner);
    edit.set_keyframes_name(&keys, "raw name with spaces")
        .unwrap();
    edit.set_keyframe_key_text(&first, "from, 10%, to").unwrap();
    owner.commit(edit).unwrap();
    let after = owner.snapshot();
    assert_eq!(
        owner
            .css_keyframes_rule(&keys)
            .and_then(|handle| handle.capture(&after))
            .and_then(|capture| capture.name())
            .unwrap(),
        "raw name with spaces"
    );
    assert_eq!(
        owner
            .css_keyframe_rule(&first)
            .and_then(|handle| handle.capture(&after))
            .and_then(|capture| capture.key_text(CssSerializationLimits::default()))
            .unwrap(),
        "0%, 10%, 100%"
    );
    assert_eq!(
        owner
            .css_keyframes_rule(&keys)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.name())
            .unwrap(),
        "first"
    );
    assert_eq!(
        owner
            .css_keyframes_rule(&keys)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.length())
            .unwrap(),
        1
    );
}
#[test]
fn insertion_uses_actual_parent_depth_and_failed_batch_cannot_publish_earlier_edits() {
    let limits = CssomLimits {
        max_depth: 2,
        ..Default::default()
    };
    let mut owner = CssomStore::new(limits, CssomContext::default()).unwrap();
    let sheet = sheet(&mut owner, "a { & b {} }", true);
    let before = owner.snapshot();
    let root = roots(&before, &sheet)[0].clone();
    let nested = children(&before, &root)[0].clone();
    let mut edit = batch(&owner);
    edit.set_disabled(&sheet, true).unwrap();
    assert!(matches!(
        edit.insert_group_rule(&nested, "& c {}", 0),
        Err(CssomError::Limit {
            resource: "rule depth",
            ..
        })
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    let unchanged = owner.snapshot();
    assert_eq!(unchanged.revision(), before.revision());
    assert!(!unchanged.sheet(&sheet).unwrap().inputs().disabled);
    assert!(children(&unchanged, &nested).is_empty());
    let mut edit = batch(&owner);
    let ticket = edit.insert_group_rule(&root, "& c {}", 1).unwrap();
    let created = owner.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    assert_eq!(owner.snapshot().parent_rule(&created).unwrap(), Some(&root));
}
#[test]
fn nested_and_scoped_selector_contexts_follow_real_ancestry_without_synthetic_sources() {
    let mut owner = store();
    let sheet = sheet(
        &mut owner,
        "a { @media screen { & b {} } } @scope (.root) { @media print { .scoped {} } }",
        true,
    );
    let before = owner.snapshot();
    let roots = roots(&before, &sheet);
    let nested = children(&before, &children(&before, &roots[0])[0])[0].clone();
    let scoped = children(&before, &children(&before, &roots[1])[0])[0].clone();
    let mut edit = batch(&owner);
    assert!(edit.set_style_selector_text(&nested, "> .changed").unwrap());
    assert!(
        edit.set_style_selector_text(&scoped, ".scoped-changed")
            .unwrap()
    );
    owner.commit(edit).unwrap();
    let after = owner.snapshot();
    assert_eq!(
        owner
            .css_style_rule(&nested)
            .and_then(|handle| handle.capture(&after))
            .and_then(|capture| capture.selector_text(CssSerializationLimits::default()))
            .unwrap(),
        "& > .changed"
    );
    assert!(matches!(
        after.rule(&scoped).unwrap().data(),
        CssomRuleData::Style {
            selectors: CssomSelectors::Scoped(_),
            ..
        }
    ));
    assert_eq!(
        after
            .rule(&scoped)
            .unwrap()
            .selector_input()
            .unwrap()
            .source()
            .as_str(),
        ".scoped-changed"
    );
    let mut edit = batch(&owner);
    edit.delete_sheet_rule(&sheet, 1).unwrap();
    owner.commit(edit).unwrap();
    let detached = owner.snapshot();
    assert_eq!(detached.parent_style_sheet(&scoped).unwrap(), None);
    let mut edit = batch(&owner);
    assert!(
        edit.set_style_selector_text(&scoped, ".still-scoped")
            .unwrap()
    );
    owner.commit(edit).unwrap();
    assert!(matches!(
        owner.snapshot().rule(&scoped).unwrap().data(),
        CssomRuleData::Style {
            selectors: CssomSelectors::Scoped(_),
            ..
        }
    ));
}
#[test]
fn page_selector_success_and_syntax_noop_preserve_current_descriptor_and_margin_identity() {
    let mut owner = store();
    let sheet = sheet(
        &mut owner,
        "@page :left { size: A4; @top-left { content: 'x' } }",
        true,
    );
    let before = owner.snapshot();
    let page = roots(&before, &sheet)[0].clone();
    let block = before
        .rule(&page)
        .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    let margin = children(&before, &page);
    let mut edit = batch(&owner);
    assert!(
        edit.set_page_selector_text(&page, "chapter:recto, appendix:verso")
            .unwrap()
    );
    owner.commit(edit).unwrap();
    let after = owner.snapshot();
    assert_eq!(
        owner
            .css_page_rule(&page)
            .and_then(|handle| handle.capture(&after))
            .and_then(|capture| capture.selector_text(CssSerializationLimits::default()))
            .unwrap(),
        "chapter:recto, appendix:verso"
    );
    assert_eq!(
        after
            .rule(&page)
            .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
            .unwrap(),
        &block
    );
    assert_eq!(children(&after, &page), margin);
    let mut edit = batch(&owner);
    assert!(!edit.set_page_selector_text(&page, "a > b").unwrap());
    assert!(matches!(
        owner.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    let mut edit = batch(&owner);
    assert!(edit.set_page_selector_text(&page, "").unwrap());
    assert!(matches!(
        owner.commit(edit).unwrap().publication(),
        CssomPublication::Changed(_)
    ));
    assert_eq!(
        owner
            .css_page_rule(&page)
            .unwrap()
            .selector_text(&owner, CssSerializationLimits::default())
            .unwrap(),
        ""
    );
}
#[test]
fn input_and_live_name_limits_abort_without_identity_or_revision_changes_and_retry_is_fresh() {
    let limits = CssomLimits {
        max_input_bytes: 64,
        max_string_bytes: 64,
        ..Default::default()
    };
    let mut owner = CssomStore::new(limits, CssomContext::default()).unwrap();
    let sheet = sheet(&mut owner, "@keyframes short { from {} } a {}", true);
    let before = owner.snapshot();
    let roots = roots(&before, &sheet);
    let mut edit = batch(&owner);
    edit.set_disabled(&sheet, true).unwrap();
    assert!(matches!(
        edit.set_keyframes_name(&roots[0], &"x".repeat(128)),
        Err(CssomError::Limit { .. })
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    let unchanged = owner.snapshot();
    assert_eq!(unchanged.revision(), before.revision());
    assert_eq!(
        owner
            .css_keyframes_rule(&roots[0])
            .and_then(|handle| handle.capture(&unchanged))
            .and_then(|capture| capture.name())
            .unwrap(),
        "short"
    );
    assert!(!unchanged.sheet(&sheet).unwrap().inputs().disabled);
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.set_style_selector_text(&roots[1], &"a".repeat(128)),
        Err(CssomError::ParseResource(_))
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    let mut edit = batch(&owner);
    assert!(edit.set_style_selector_text(&roots[1], ".ok").unwrap());
    owner.commit(edit).unwrap();
    assert_eq!(
        owner
            .css_style_rule(&roots[1])
            .unwrap()
            .selector_text(&owner, CssSerializationLimits::default())
            .unwrap(),
        ".ok"
    );
}
#[test]
fn raw_nested_fallback_uses_actual_quirks_mode_and_retains_recovered_occurrences() {
    let context = CssomContext {
        inputs: vec![CssomInput {
            version: CssomInputVersion {
                role: CssomInputRole::ParserMode,
                identity: "quirks".into(),
                revision: 3,
            },
            data: CssomInputData::ParserMode(CssParserContext::new(CssParserMode::Quirks)),
        }],
        linked: vec![],
    };
    let mut owner = CssomStore::new(CssomLimits::default(), context).unwrap();
    let sheet = sheet(&mut owner, "a {}", true);
    let root = roots(&owner.snapshot(), &sheet)[0].clone();
    let mut edit = batch(&owner);
    let ticket = edit
        .insert_group_rule(&root, "width: 12; color: blue; broken: x;", 0)
        .unwrap();
    let id = owner.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    let after = owner.snapshot();
    let CssomAuthoredRule::RawNestedDeclarations(values) = after.rule(&id).unwrap().authored()
    else {
        panic!("raw");
    };
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].parser_context().mode(), CssParserMode::Quirks);
    assert_eq!(
        after
            .specified_property_css_text(
                after
                    .rule(&id)
                    .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
                    .unwrap(),
                CssSerializationLimits::default()
            )
            .unwrap(),
        "width: 12px; color: blue;"
    );
}

#[test]
fn successful_recovery_keeps_owning_diagnostics_and_invalid_noop_keeps_previous_record() {
    let mut owner = store();
    let sheet = sheet(&mut owner, "a {}", true);
    let root = roots(&owner.snapshot(), &sheet)[0].clone();
    let mut edit = batch(&owner);
    let ticket = edit.insert_group_rule(&root, "b { width: 1px", 0).unwrap();
    let child = owner.commit(edit).unwrap().rule(&ticket).unwrap().clone();
    let before = owner.snapshot();
    assert!(
        before
            .rule(&child)
            .unwrap()
            .admission_diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    let mut edit = batch(&owner);
    assert!(edit.set_style_selector_text(&child, ":is(.ok").unwrap());
    owner.commit(edit).unwrap();
    let recovered = owner.snapshot();
    assert!(
        recovered
            .rule(&child)
            .unwrap()
            .admission_diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    let diagnostics = recovered
        .rule(&child)
        .unwrap()
        .admission_diagnostics()
        .to_vec();
    let mut edit = batch(&owner);
    assert!(!edit.set_style_selector_text(&child, ".bad,").unwrap());
    owner.commit(edit).unwrap();
    let unchanged = owner.snapshot();
    assert_eq!(unchanged.revision(), recovered.revision());
    assert_eq!(
        unchanged.rule(&child).unwrap().admission_diagnostics(),
        diagnostics
    );
    assert_eq!(
        unchanged
            .rule(&child)
            .unwrap()
            .selector_input()
            .unwrap()
            .source()
            .as_str(),
        ":is(.ok"
    );
    assert!(before.rule(&child).unwrap().selector_input().is_none());
}

#[test]
fn deprecated_type_numbers_and_associated_style_objects_follow_checked_kinds() {
    let mut owner = store();
    let sheet = sheet(
        &mut owner,
        "@import 'x'; @namespace 'urn:test'; a {} @media print {} \
         @font-face { font-family: Demo; src: url('demo.woff'); } \
         @page { @top-left {} } @keyframes demo { from {} } \
         @counter-style marker { system: cyclic; symbols: 'x'; } \
         @supports (display: grid) {} \
         @font-feature-values Demo { @styleset { normal: 1; } } \
         @layer last; @container (width > 1px) {}",
        false,
    );
    let snapshot = owner.snapshot();
    let roots = roots(&snapshot, &sheet);
    assert_eq!(roots.len(), 12);
    let expected = [3, 10, 1, 4, 5, 6, 7, 11, 12, 14, 0, 0];
    for (id, expected) in roots.iter().zip(expected) {
        assert_eq!(
            owner
                .css_rule(id)
                .and_then(|handle| handle.capture(&snapshot))
                .and_then(|capture| capture.r#type())
                .unwrap(),
            expected
        );
        assert_eq!(snapshot.parent_rule(id).unwrap(), None);
        assert_eq!(snapshot.parent_style_sheet(id).unwrap(), Some(&sheet));
    }
    let margin = children(&snapshot, &roots[5])[0].clone();
    let keyframe = children(&snapshot, &roots[6])[0].clone();
    assert_eq!(
        owner
            .css_rule(&margin)
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.r#type())
            .unwrap(),
        9
    );
    assert_eq!(
        owner
            .css_rule(&keyframe)
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.r#type())
            .unwrap(),
        8
    );
    for id in [&roots[2], &roots[4], &roots[5], &margin, &keyframe] {
        let block = snapshot
            .rule(id)
            .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
            .unwrap();
        assert_eq!(snapshot.block(block).unwrap().parent(), Some(id));
        assert_eq!(snapshot.block(block).unwrap().owner(), None);
        assert_eq!(
            snapshot.block(block).unwrap().flags(),
            CssomBlockFlags::default()
        );
    }
    assert!(matches!(
        snapshot
            .rule(&roots[0])
            .and_then(|record| record.data().block().ok_or(CssomError::WrongKind)),
        Err(CssomError::WrongKind)
    ));
}

#[test]
fn component_syntax_uses_source_noop_or_syntax_without_poisoning_successful_noops() {
    let mut owner = store();
    let sheet = sheet(&mut owner, "@page old {} @keyframes demo { from {} }", true);
    let before = owner.snapshot();
    let roots = roots(&before, &sheet);
    let keyframe = children(&before, &roots[1])[0].clone();
    let mut edit = batch(&owner);
    edit.set_disabled(&sheet, true).unwrap();
    assert!(!edit.set_page_selector_text(&roots[0], ")").unwrap());
    assert!(edit.append_keyframe_rule(&roots[1], ")").unwrap().is_none());
    owner.commit(edit).unwrap();
    let after = owner.snapshot();
    assert!(after.sheet(&sheet).unwrap().inputs().disabled);
    assert_eq!(
        owner
            .css_page_rule(&roots[0])
            .and_then(|handle| handle.capture(&after))
            .and_then(|capture| capture.selector_text(CssSerializationLimits::default()))
            .unwrap(),
        "old"
    );
    assert_eq!(
        owner
            .css_keyframes_rule(&roots[1])
            .and_then(|handle| handle.capture(&after))
            .and_then(|capture| capture.length())
            .unwrap(),
        1
    );
    let mut edit = batch(&owner);
    edit.set_disabled(&sheet, false).unwrap();
    source_error(
        edit.set_keyframe_key_text(&keyframe, ")"),
        CssomException::Syntax,
    );
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), after.revision());
    assert!(owner.snapshot().sheet(&sheet).unwrap().inputs().disabled);
}

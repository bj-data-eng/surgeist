//! Surgeist's selected inert named-support grouping and canonical text contract.
//! These public operations exist before its dedicated live payload API.
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

fn definition(source: &str) -> (CssomStore, CssomSheetId, CssomRuleId) {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = batch(&store);
    let ticket = edit
        .create_parsed_sheet(
            source,
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "named-support-document".into(),
                    revision: 1,
                },
                true,
            ),
        )
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let roots = snapshot.sheet_rules(&sheet).unwrap();
    assert_eq!(roots.len(), 1);
    let rule = roots[0].clone();
    assert_eq!(
        store
            .css_supports_condition_rule(&rule)
            .and_then(|handle| handle.capture(&snapshot))
            .and_then(|capture| capture.name())
            .unwrap(),
        "--x"
    );
    (store, sheet, rule)
}

#[test]
fn empty_named_definition_has_one_stable_empty_child_list() {
    let (mut store, sheet, rule) = definition("@supports-condition --x {}");
    let before = store.snapshot();
    let list = before
        .rule(&rule)
        .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    assert_eq!(
        store
            .css_rule_list(&list)
            .and_then(|list| list.capture(&before))
            .and_then(|list| list.length())
            .unwrap(),
        0
    );
    assert_eq!(
        store
            .css_rule_list(&list)
            .and_then(|list| list.capture(&before))
            .and_then(|list| list.item(0))
            .map(|item| item.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        None
    );
    let mut edit = batch(&store);
    edit.set_disabled(&sheet, true).unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_ne!(after.revision(), before.revision());
    assert_eq!(
        after
            .rule(&rule)
            .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
            .unwrap(),
        &list
    );
    assert_eq!(
        store
            .css_rule_list(&list)
            .and_then(|list| list.capture(&after))
            .and_then(|list| list.length())
            .unwrap(),
        0
    );
    assert_eq!(
        before
            .rule(&rule)
            .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
            .unwrap(),
        &list
    );
    assert_eq!(
        store
            .css_rule_list(&list)
            .and_then(|list| list.capture(&before))
            .and_then(|list| list.length())
            .unwrap(),
        0
    );
}

#[test]
fn direct_declarations_and_candidates_are_individual_inert_leaf_rules() {
    let (store, sheet, rule) =
        definition("@supports-condition --x { color:red; color:blue!important; & { } @future x; }");
    let snapshot = store.snapshot();
    let list = snapshot
        .rule(&rule)
        .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
        .unwrap();
    let children = snapshot.rules(list).unwrap();
    assert_eq!(
        store
            .css_rule_list(list)
            .and_then(|list| list.capture(&snapshot))
            .and_then(|list| list.length())
            .unwrap(),
        4
    );
    let expected = [
        "color: red;",
        "color: blue !important;",
        "& { }",
        "@future x;",
    ];
    for (index, (child, text)) in children.iter().zip(expected).enumerate() {
        assert_eq!(
            store
                .css_rule_list(list)
                .and_then(|list| list.capture(&snapshot))
                .and_then(|list| list.item(index))
                .map(|item| item.map(|rule| rule.handle().id().clone()))
                .unwrap(),
            Some(child.clone())
        );
        assert_eq!(captured_rule(&store, &snapshot, child).r#type().unwrap(), 0);
        assert_eq!(snapshot.parent_rule(child).unwrap(), Some(&rule));
        assert_eq!(snapshot.parent_style_sheet(child).unwrap(), Some(&sheet));
        assert!(matches!(
            snapshot
                .rule(child)
                .and_then(|record| record.data().block().ok_or(CssomError::WrongKind)),
            Err(CssomError::WrongKind)
        ));
        assert!(matches!(
            snapshot
                .rule(child)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind)),
            Err(CssomError::WrongKind)
        ));
        assert_eq!(
            captured_rule(&store, &snapshot, child)
                .css_text(&CssomRuleFormatLimits::default())
                .unwrap(),
            text
        );
    }
    assert_eq!(children.len(), 4);
    assert_eq!(
        store
            .css_rule_list(list)
            .and_then(|list| list.capture(&snapshot))
            .and_then(|list| list.item(4))
            .map(|item| item.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        None
    );
    assert!(
        children
            .iter()
            .enumerate()
            .all(|(index, id)| !children[..index].contains(id))
    );
}

#[test]
fn empty_named_definition_has_selected_canonical_outer_text() {
    let (store, _, rule) = definition("@supports-condition --x{}");
    assert_eq!(
        captured_rule(&store, &store.snapshot(), &rule)
            .css_text(&CssomRuleFormatLimits::default())
            .unwrap(),
        "@supports-condition --x { }"
    );
}

#[test]
fn named_definition_outer_text_preserves_duplicate_tests_and_candidate_order() {
    let (store, _, rule) =
        definition("@supports-condition --x { color:red; color:blue!important; & { } @future x; }");
    let snapshot = store.snapshot();
    let revision = snapshot.revision().clone();
    assert_eq!(
        captured_rule(&store, &snapshot, &rule)
            .css_text(&CssomRuleFormatLimits::default())
            .unwrap(),
        "@supports-condition --x { color: red; color: blue !important; & { } @future x; }"
    );
    assert_eq!(store.snapshot().revision(), &revision);
}

#[test]
fn assigning_outer_css_text_keeps_original_definition_and_publication_unchanged() {
    let (mut store, sheet, rule) = definition("@supports-condition --x { color:red; }");
    let before = store.snapshot();
    let mut edit = batch(&store);
    edit.set_rule_css_text(&rule, "@supports-condition --changed { unknown:value; }")
        .unwrap();
    let commit = store.commit(edit).unwrap();
    assert!(matches!(
        commit.publication(),
        CssomPublication::Unchanged { .. }
    ));
    let after = store.snapshot();
    assert_eq!(after.revision(), before.revision());
    assert_eq!(
        after.sheet_rules(&sheet).unwrap(),
        before.sheet_rules(&sheet).unwrap()
    );
    assert_eq!(
        store
            .css_supports_condition_rule(&rule)
            .and_then(|handle| handle.capture(&after))
            .and_then(|capture| capture.name())
            .unwrap(),
        "--x"
    );
    let CssomAuthoredRule::Ordinary(after_original) = after.rule(&rule).unwrap().authored() else {
        panic!("original ordinary occurrence");
    };
    let CssomAuthoredRule::Ordinary(before_original) = before.rule(&rule).unwrap().authored()
    else {
        panic!("original ordinary occurrence");
    };
    assert_eq!(after_original, before_original);
}

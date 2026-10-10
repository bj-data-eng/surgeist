//! Surgeist's selected inert named-support grouping and canonical text contract.
//! These public operations exist before its dedicated live payload API.
use surgeist_cssom::*;

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
    assert_eq!(snapshot.named_support_condition_name(&rule).unwrap(), "--x");
    (store, sheet, rule)
}

#[test]
fn empty_named_definition_has_one_stable_empty_child_list() {
    let (mut store, sheet, rule) = definition("@supports-condition --x {}");
    let before = store.snapshot();
    let list = before.child_rules(&rule).unwrap().clone();
    assert_eq!(before.rule_list_length(&list).unwrap(), 0);
    assert_eq!(before.rule_list_item(&list, 0).unwrap(), None);
    let mut edit = batch(&store);
    edit.set_disabled(&sheet, true).unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_ne!(after.revision(), before.revision());
    assert_eq!(after.child_rules(&rule).unwrap(), &list);
    assert_eq!(after.rule_list_length(&list).unwrap(), 0);
    assert_eq!(before.child_rules(&rule).unwrap(), &list);
    assert_eq!(before.rule_list_length(&list).unwrap(), 0);
}

#[test]
fn direct_declarations_and_candidates_are_individual_inert_leaf_rules() {
    let (store, sheet, rule) =
        definition("@supports-condition --x { color:red; color:blue!important; & { } @future x; }");
    let snapshot = store.snapshot();
    let list = snapshot.child_rules(&rule).unwrap();
    let children = snapshot.rules(list).unwrap();
    assert_eq!(snapshot.rule_list_length(list).unwrap(), 4);
    let expected = [
        "color: red;",
        "color: blue !important;",
        "& { }",
        "@future x;",
    ];
    for (index, (child, text)) in children.iter().zip(expected).enumerate() {
        assert_eq!(snapshot.rule_list_item(list, index).unwrap(), Some(child));
        assert_eq!(snapshot.rule_type(child).unwrap(), 0);
        assert_eq!(snapshot.parent_rule(child).unwrap(), Some(&rule));
        assert_eq!(snapshot.parent_style_sheet(child).unwrap(), Some(&sheet));
        assert!(matches!(
            snapshot.rule_style(child),
            Err(CssomError::WrongKind)
        ));
        assert!(matches!(
            snapshot.child_rules(child),
            Err(CssomError::WrongKind)
        ));
        assert_eq!(
            snapshot
                .rule_css_text(child, &CssomRuleFormatLimits::default())
                .unwrap(),
            text
        );
    }
    assert_eq!(children.len(), 4);
    assert_eq!(snapshot.rule_list_item(list, 4).unwrap(), None);
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
        store
            .snapshot()
            .rule_css_text(&rule, &CssomRuleFormatLimits::default())
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
        snapshot
            .rule_css_text(&rule, &CssomRuleFormatLimits::default())
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
    assert_eq!(after.named_support_condition_name(&rule).unwrap(), "--x");
    let CssomAuthoredRule::Ordinary(after_original) = after.rule(&rule).unwrap().authored() else {
        panic!("original ordinary occurrence");
    };
    let CssomAuthoredRule::Ordinary(before_original) = before.rule(&rule).unwrap().authored()
    else {
        panic!("original ordinary occurrence");
    };
    assert_eq!(after_original, before_original);
}

//! Conditional5 §8 permits named supports definitions before imports/namespaces,
//! including before a later prefix member. CSSOM insert step7 remains separate.
use surgeist_css::CssRule;
use surgeist_cssom::*;

const NAMED: &str = "@supports-condition --probe { display: grid; }";

fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}

fn fixture(source: &str) -> (CssomStore, CssomSheetId) {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = batch(&store);
    let ticket = edit
        .create_parsed_sheet(
            source,
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "ordering-document".into(),
                    revision: 1,
                },
                true,
            ),
        )
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    (store, sheet)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Named,
    Import,
    Namespace,
}

fn kinds(snapshot: &CssomSnapshot, rules: &[CssomRuleId]) -> Vec<Kind> {
    rules
        .iter()
        .map(|id| match snapshot.rule(id).unwrap().authored() {
            CssomAuthoredRule::Ordinary(CssRule::SupportsCondition(_)) => Kind::Named,
            CssomAuthoredRule::Ordinary(CssRule::Import(_)) => Kind::Import,
            CssomAuthoredRule::Ordinary(CssRule::Namespace(_)) => Kind::Namespace,
            other => panic!("unexpected fixture kind: {other:?}"),
        })
        .collect()
}

fn successful_insertion(source: &str, initial: &[Kind], incoming: &str, index: usize, kind: Kind) {
    let (mut store, sheet) = fixture(source);
    let before = store.snapshot();
    let list = before.sheet(&sheet).unwrap().rules().clone();
    let original = before.rules(&list).unwrap().to_vec();
    assert_eq!(kinds(&before, &original), initial);
    let mut edit = batch(&store);
    let (returned_index, ticket) = edit.insert_sheet_rule(&sheet, incoming, index).unwrap();
    assert_eq!(returned_index, index);
    let committed = store.commit(edit).unwrap();
    let inserted = committed.rule(&ticket).unwrap().clone();
    assert!(!original.contains(&inserted));
    let after = store.snapshot();
    let CssomPublication::Changed(summary) = committed.publication() else {
        panic!("insertion must publish");
    };
    assert_ne!(before.revision(), after.revision());
    assert_eq!(summary.from(), before.revision());
    assert_eq!(summary.to(), after.revision());
    assert_eq!(after.sheet(&sheet).unwrap().rules(), &list);
    let mut expected = original.clone();
    expected.insert(index, inserted.clone());
    assert_eq!(after.rules(&list).unwrap(), expected);
    let mut expected_kinds = initial.to_vec();
    expected_kinds.insert(index, kind);
    assert_eq!(kinds(&after, &expected), expected_kinds);
    assert_eq!(after.parent_style_sheet(&inserted).unwrap(), Some(&sheet));
    assert_eq!(before.rules(&list).unwrap(), original);
    for id in original {
        assert_eq!(before.parent_style_sheet(&id).unwrap(), Some(&sheet));
        assert_eq!(after.parent_style_sheet(&id).unwrap(), Some(&sheet));
    }
}

#[test]
fn import_after_named_support_definition_is_allowed() {
    successful_insertion(
        NAMED,
        &[Kind::Named],
        "@import 'later.css';",
        1,
        Kind::Import,
    );
}

#[test]
fn named_support_definition_before_existing_import_is_allowed() {
    successful_insertion(
        "@import 'existing.css';",
        &[Kind::Import],
        NAMED,
        0,
        Kind::Named,
    );
}

#[test]
fn named_support_definition_before_existing_namespace_is_allowed() {
    successful_insertion(
        "@namespace p 'urn:p';",
        &[Kind::Namespace],
        NAMED,
        0,
        Kind::Named,
    );
}

#[test]
fn named_support_definition_between_existing_imports_is_allowed() {
    successful_insertion(
        "@import 'first.css'; @import 'second.css';",
        &[Kind::Import, Kind::Import],
        NAMED,
        1,
        Kind::Named,
    );
}

#[test]
fn named_support_definition_between_existing_import_and_namespace_is_allowed() {
    successful_insertion(
        "@import 'first.css'; @namespace p 'urn:p';",
        &[Kind::Import, Kind::Namespace],
        NAMED,
        1,
        Kind::Named,
    );
}

#[test]
fn new_namespace_after_named_definition_reaches_whole_list_invalid_state() {
    let (mut store, sheet) = fixture(NAMED);
    let before = store.snapshot();
    let list = before.sheet(&sheet).unwrap().rules().clone();
    let original = before.rules(&list).unwrap().to_vec();
    assert_eq!(kinds(&before, &original), [Kind::Named]);
    let mut edit = batch(&store);
    edit.set_disabled(&sheet, true).unwrap();
    let result = edit.insert_sheet_rule(&sheet, "@namespace p 'urn:p';", 1);
    assert!(
        matches!(
            result,
            Err(CssomError::Source(CssomException::InvalidState))
        ),
        "whole-list namespace error must follow permitted prefix ordering: {result:?}"
    );
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    let after = store.snapshot();
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.rules(&list).unwrap(), original);
    assert!(!after.sheet(&sheet).unwrap().inputs().disabled);
    assert_eq!(after.sheet(&sheet).unwrap().rules(), &list);
    assert_eq!(before.rules(&list).unwrap(), original);
}

#[test]
fn ordinary_body_rules_still_block_a_later_import() {
    for source in ["a {}", "@supports (display: grid) {}"] {
        let (mut store, sheet) = fixture(source);
        let before = store.snapshot();
        let list = before.sheet(&sheet).unwrap().rules().clone();
        let original = before.rules(&list).unwrap().to_vec();
        assert_eq!(original.len(), 1);
        let mut edit = batch(&store);
        assert!(matches!(
            edit.insert_sheet_rule(&sheet, "@import 'blocked.css';", 1),
            Err(CssomError::Source(CssomException::HierarchyRequest))
        ));
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(store.snapshot().rules(&list).unwrap(), original);
        assert_eq!(before.rules(&list).unwrap(), original);
    }
}

//! Detached live families remain part of the global storage allowance.
use surgeist_cssom::*;

fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "doc".into(),
            revision: 1,
        },
        "doc".into(),
        None,
    )
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
#[test]
fn detached_family_bytes_cannot_be_reused_by_another_sheet() {
    // Each sheet contributes six host bytes; C consumes the thirteenth byte.
    let mut store = CssomStore::new(
        CssomLimits {
            max_string_bytes: 13,
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap();
    let mut edit = batch(&store);
    let first = edit
        .create_parsed_sheet("@font-feature-values C {}", inputs())
        .unwrap();
    let empty = edit.create_parsed_sheet("", inputs()).unwrap();
    let commit = store.commit(edit).unwrap();
    let first = commit.sheet(&first).unwrap().clone();
    let empty = commit.sheet(&empty).unwrap().clone();
    let before = store.snapshot();
    let rule = before.rules(before.sheet(&first).unwrap().rules()).unwrap()[0].clone();
    let mut edit = batch(&store);
    assert_eq!(edit.delete_sheet_rule(&first, 0).unwrap(), rule);
    store.commit(edit).unwrap();
    let detached = store.snapshot();
    assert_eq!(
        detached.rule(&rule).unwrap().parent(),
        &CssomParent::Detached
    );
    let CssomRuleData::FontFeatureValues { families, .. } = detached.rule(&rule).unwrap().data()
    else {
        panic!("retained feature rule expected");
    };
    assert_eq!(families[0].as_str(), "C");
    let mut edit = batch(&store);
    assert!(matches!(
        edit.replace_sheet_text(&empty, "@font-feature-values D {}"),
        Err(CssomError::Limit {
            resource: "host and live string bytes",
            maximum: 13
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), detached.revision());
    assert!(
        store
            .snapshot()
            .rules(store.snapshot().sheet(&empty).unwrap().rules())
            .unwrap()
            .is_empty()
    );
    assert!(before.rule(&rule).is_ok());
}

//! Regression for the published owning foundation's current family storage.
use surgeist_css::*;
use surgeist_cssom::*;

fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "sheet".into(),
            revision: 1,
        },
        true,
    )
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn assert_family_quota(typed: bool, bytes: bool) {
    let limits = CssomLimits {
        // External version identity consumes all five admitted host string bytes.
        // Three admitted non-family entries are sheet-order, original rule and
        // current sheet child membership. The fourth is the live family entry.
        max_string_bytes: if bytes { 5 } else { 1_000 },
        max_entries: if bytes { 100 } else { 3 },
        ..Default::default()
    };
    let mut control = CssomStore::new(limits.clone(), Default::default()).unwrap();
    let mut edit = batch(&control);
    edit.create_typed_sheet(CssSheet::new(), inputs()).unwrap();
    control.commit(edit).unwrap();
    let mut store = CssomStore::new(limits, Default::default()).unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    let source = "@font-feature-values A {}";
    let result = if typed {
        let report = parse_sheet(source);
        assert!(report.is_clean());
        edit.create_typed_sheet(report.into_parts().0, inputs())
    } else {
        edit.create_parsed_sheet(source, inputs())
    };
    let resource = if bytes {
        "host and live string bytes"
    } else {
        "entries"
    };
    assert!(
        matches!(result, Err(CssomError::Limit { resource: actual, .. }) if actual == resource),
        "current live family must consume the global {resource} allowance; actual {result:?}"
    );
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert!(store.snapshot().sheets().is_empty());
}
#[test]
fn parsed_family_bytes_respect_global_live_storage() {
    assert_family_quota(false, true);
}
#[test]
fn typed_family_bytes_respect_global_live_storage() {
    assert_family_quota(true, true);
}
#[test]
fn parsed_family_entries_respect_global_live_storage() {
    assert_family_quota(false, false);
}
#[test]
fn typed_family_entries_respect_global_live_storage() {
    assert_family_quota(true, false);
}

//! Structural equality ignores where CSS values were parsed from, so the store compares
//! provenance explicitly: syntax that is equal but came from other text is a change.
use surgeist_cssom::*;

fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}

fn is_changed(commit: &CssomCommit) -> bool {
    matches!(commit.publication(), CssomPublication::Changed(_))
}

#[test]
fn replacing_a_sheet_by_text_whose_diagnostics_differ_only_in_position_is_a_change() {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = batch(&store);
    let ticket = edit
        .create_parsed_sheet(
            "",
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "document".into(),
                    revision: 1,
                },
                "document".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    // Neither text retains a rule, so no identity is allocated: only the recovery diagnostics
    // decide whether the replacement changed anything.
    for (text, expected) in [("}", true), ("}", false), (" }", true), (" }", false)] {
        let mut edit = batch(&store);
        edit.replace_sheet_text(&sheet, text).unwrap();
        let commit = store.commit(edit).unwrap();
        assert_eq!(is_changed(&commit), expected, "{text:?}");
    }
}

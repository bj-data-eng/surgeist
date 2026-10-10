use surgeist_css::*;
use surgeist_cssom::*;

fn context(revision: u64) -> CssomContext {
    CssomContext {
        inputs: vec![CssomInput {
            version: CssomInputVersion {
                role: CssomInputRole::ParserMode,
                identity: "selector-parser".into(),
                revision,
            },
            data: CssomInputData::ParserMode(CssParserContext::default()),
        }],
        ..Default::default()
    }
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}

#[test]
fn retained_selector_endpoints_change_the_manifest_when_current_context_is_restored() {
    let mut store = CssomStore::new(CssomLimits::default(), context(1)).unwrap();
    let mut edit = batch(&store);
    let ticket = edit
        .create_parsed_sheet(
            "a { color: red; }",
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
    let before = store.snapshot();
    let rule = before.sheet_rules(&sheet).unwrap()[0].clone();
    let mut edit = batch(&store);
    edit.update_context(context(2)).unwrap();
    edit.set_style_selector_text(&rule, ".changed").unwrap();
    edit.update_context(context(1)).unwrap();
    let commit = store.commit(edit).unwrap();
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("selector admission publishes");
    };
    let after = store.snapshot();
    assert_eq!(after.context(), before.context());
    assert_eq!(
        after.rule(&rule).unwrap().selector_inputs(),
        Some(&context(2))
    );
    assert_eq!(
        summary.before_inputs().context,
        summary.after_inputs().context
    );
    assert_eq!(
        summary.before_inputs().sheets,
        summary.after_inputs().sheets
    );
    assert_eq!(
        summary.before_inputs().blocks,
        summary.after_inputs().blocks
    );
    assert_eq!(
        summary.before_inputs().imports,
        summary.after_inputs().imports
    );
    assert_eq!(
        summary.before_inputs().declaration_admissions,
        summary.after_inputs().declaration_admissions
    );
    assert_eq!(
        summary.before_inputs().font_family_admissions,
        summary.after_inputs().font_family_admissions
    );
    // The restored current context cannot stand in for the independently revised
    // endpoint retained by the successful selector admission.
    assert_ne!(
        summary.before_inputs(),
        summary.after_inputs(),
        "selector-admission endpoints must appear in the correlated manifest"
    );
    assert_eq!(before.rule(&rule).unwrap().selector_inputs(), None);
    assert!(summary.before_inputs().selector_admissions.is_empty());
    assert_eq!(
        summary.after_inputs().selector_admissions,
        vec![(rule.clone(), vec![context(2).inputs[0].version.clone()])]
    );
    assert_eq!(
        after
            .rule(&rule)
            .unwrap()
            .selector_input()
            .unwrap()
            .source()
            .as_str(),
        ".changed"
    );
    let retained = after.inputs().selector_admissions;
    let mut edit = batch(&store);
    assert!(!edit.set_style_selector_text(&rule, ".bad,").unwrap());
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().inputs().selector_admissions, retained);

    let mut edit = batch(&store);
    assert_eq!(edit.delete_sheet_rule(&sheet, 0).unwrap(), rule);
    let commit = store.commit(edit).unwrap();
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("detach publishes");
    };
    assert_eq!(summary.before_inputs().selector_admissions, retained);
    assert_eq!(summary.after_inputs().selector_admissions, retained);
    let detached = store.snapshot();
    assert_eq!(detached.parent_style_sheet(&rule).unwrap(), None);
    assert_eq!(detached.inputs().selector_admissions, retained);
    assert_eq!(after.parent_style_sheet(&rule).unwrap(), Some(&sheet));
    assert!(summary.supports_incremental(&after, &detached));
}

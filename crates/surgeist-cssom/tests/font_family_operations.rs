//! Public Fonts4 family attribute behavior through the actual CSS prelude owner.
use surgeist_css::*;
use surgeist_cssom::*;

fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(limits: CssomLimits) -> (CssomStore, CssomRuleId) {
    let mut store = CssomStore::new(limits, Default::default()).unwrap();
    let mut edit = batch(&store);
    let ticket = edit
        .create_parsed_sheet(
            "@font-feature-values A {}",
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "doc".into(),
                    revision: 1,
                },
                true,
            ),
        )
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let rule = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()[0]
        .clone();
    (store, rule)
}
fn parsed_source(input: &CssomFontFamilyInput) -> &str {
    let CssValueOrigin::Parsed(origin) = input.value().origin() else {
        panic!("real raw parsed origin");
    };
    origin.source().as_str()
}
#[test]
fn family_text_uses_current_mode_and_retains_actual_member_origins_and_input_versions() {
    let (mut store, rule) = create(Default::default());
    let before = store.snapshot();
    assert!(
        before
            .font_feature_values(&rule)
            .unwrap()
            .family_input()
            .is_none()
    );
    let version = CssomInputVersion {
        role: CssomInputRole::ParserMode,
        identity: "font-mode".into(),
        revision: 7,
    };
    let mode = CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical();
    let mut edit = batch(&store);
    edit.update_context(CssomContext {
        inputs: vec![CssomInput {
            version: version.clone(),
            data: CssomInputData::ParserMode(mode),
        }],
        ..Default::default()
    })
    .unwrap();
    let source = String::from("/*head*/ 'serif', A\\ B, '1Face' /*tail*/");
    assert!(edit.set_font_feature_family_text(&rule, &source).unwrap());
    let expected = source.clone();
    drop(source);
    let commit = store.commit(edit).unwrap();
    let after = store.snapshot();
    let view = after.font_feature_values(&rule).unwrap();
    assert_eq!(
        view.font_family(Default::default()).unwrap(),
        "\"serif\", A B, \"1Face\""
    );
    let input = view.family_input().unwrap();
    assert_eq!(parsed_source(input), expected);
    assert_eq!(input.parser_context(), mode);
    assert_eq!(input.inputs(), std::slice::from_ref(&version));
    assert!(input.diagnostics().is_empty());
    let origin = input.value().family_origin(1).unwrap();
    assert_eq!(origin.source().as_str(), expected);
    let range =
        origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value();
    assert_eq!(&expected[range], "A\\ B");
    assert_eq!(
        after.inputs().font_family_admissions,
        [(rule.clone(), vec![version])]
    );
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("family publication");
    };
    assert!(summary.categories().contains(&CssomChange::Provenance));
    let mut edit = batch(&store);
    edit.update_context(Default::default()).unwrap();
    store.commit(edit).unwrap();
    drop(store);
    assert_eq!(
        after
            .font_feature_values(&rule)
            .unwrap()
            .family_input()
            .unwrap()
            .parser_context(),
        mode
    );
    assert_eq!(
        before
            .font_feature_values(&rule)
            .unwrap()
            .font_family(Default::default())
            .unwrap(),
        "A"
    );
}
#[test]
fn syntax_noops_preserve_staged_map_edits_and_valid_eof_recovery_is_retained() {
    let (mut store, rule) = create(Default::default());
    let before = store.snapshot();
    let map = before
        .font_feature_values(&rule)
        .unwrap()
        .annotation()
        .clone();
    let mut edit = batch(&store);
    edit.set_feature(&map, "kept", &[3]).unwrap();
    for source in [
        "",
        "serif",
        "A,",
        "A;B",
        "A{}",
        "A !important",
        "]",
        "'bad\nname'",
        "url(bad url)",
    ] {
        assert!(
            !edit.set_font_feature_family_text(&rule, source).unwrap(),
            "{source}"
        );
    }
    assert!(
        edit.set_font_feature_family_text(&rule, "'Recovered")
            .unwrap()
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        after.feature_map(&map).unwrap().get("kept").unwrap(),
        Some([3].as_slice())
    );
    let view = after.font_feature_values(&rule).unwrap();
    assert_eq!(view.font_family(Default::default()).unwrap(), "Recovered");
    assert!(!view.family_input().unwrap().diagnostics().is_empty());
    assert_eq!(parsed_source(view.family_input().unwrap()), "'Recovered");
    let mut edit = batch(&store);
    assert!(
        !edit
            .set_font_feature_family_text(&rule, "\"Recovered\"")
            .unwrap()
    );
    store.commit(edit).unwrap();
    let unchanged = store.snapshot();
    assert_eq!(unchanged.revision(), after.revision());
    assert_eq!(
        unchanged.font_feature_values(&rule).unwrap().family_input(),
        view.family_input()
    );
}
#[test]
fn complete_family_resource_failure_aborts_earlier_edits_while_typed_ingress_keeps_its_origin() {
    let (mut store, rule) = create(CssomLimits {
        max_input_bytes: 40,
        ..Default::default()
    });
    let before = store.snapshot();
    let map = before
        .font_feature_values(&rule)
        .unwrap()
        .styleset()
        .clone();
    let mut edit = batch(&store);
    edit.set_feature(&map, "earlier", &[4]).unwrap();
    assert!(matches!(
        edit.set_font_feature_family_text(&rule, &format!("{}B", " ".repeat(40))),
        Err(CssomError::ParseResource(_))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert!(!store.snapshot().feature_map(&map).unwrap().has("earlier"));
    let value = parse_font_feature_values_family_list("/*actual*/ B")
        .into_parts()
        .0
        .unwrap();
    let mut edit = batch(&store);
    assert!(edit.set_font_feature_families(&rule, value).unwrap());
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    let view = snapshot.font_feature_values(&rule).unwrap();
    assert_eq!(parsed_source(view.family_input().unwrap()), "/*actual*/ B");
    let value =
        CssFontFeatureValuesFamilyList::try_new(vec![CssFontFaceFamily::try_new("C").unwrap()])
            .unwrap();
    let mut edit = batch(&store);
    assert!(edit.set_font_feature_families(&rule, value).unwrap());
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    let view = snapshot.font_feature_values(&rule).unwrap();
    let input = view.family_input().unwrap();
    assert_eq!(input.value().origin(), &CssValueOrigin::Programmatic);
    assert!(input.value().family_origin(0).is_none());
    let oversized = parse_font_feature_values_family_list(&format!("{}D", " ".repeat(40)))
        .into_parts()
        .0
        .unwrap();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_font_feature_families(&rule, oversized),
        Err(CssomError::Limit {
            resource: "retained parse input bytes",
            maximum: 40
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), snapshot.revision());
}
#[test]
fn family_output_budget_is_cumulative_and_raw_input_storage_joins_the_global_allowance() {
    let (mut store, rule) = create(Default::default());
    let mut edit = batch(&store);
    edit.set_font_feature_family_text(&rule, "A, B").unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    let view = snapshot.font_feature_values(&rule).unwrap();
    assert_eq!(
        view.font_family(CssSpecifiedValueSerializationLimits::new(2, 2, 4))
            .unwrap(),
        "A, B"
    );
    for limits in [
        CssSpecifiedValueSerializationLimits::new(1, 2, 4),
        CssSpecifiedValueSerializationLimits::new(2, 1, 4),
        CssSpecifiedValueSerializationLimits::new(2, 2, 3),
    ] {
        assert!(matches!(
            view.font_family(limits),
            Err(CssomError::Value(_))
        ));
    }
    // Document identity three bytes + current A one byte exhaust the allowance.
    let (mut store, rule) = create(CssomLimits {
        max_string_bytes: 4,
        ..Default::default()
    });
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_font_feature_family_text(&rule, "B"),
        Err(CssomError::Limit {
            resource: "host and live string bytes",
            maximum: 4
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut edit = batch(&store);
    assert!(
        edit.set_font_feature_families(
            &rule,
            CssFontFeatureValuesFamilyList::try_new(vec![CssFontFaceFamily::try_new("B").unwrap()])
                .unwrap()
        )
        .unwrap()
    );
    store.commit(edit).unwrap();
}

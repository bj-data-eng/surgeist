use std::sync::Arc;
use surgeist_css::*;
use surgeist_cssom::*;

fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "inline".into(),
            revision: 1,
        },
        true,
    )
}
fn context(parser: CssParserContext) -> CssomContext {
    CssomContext {
        inputs: vec![CssomInput {
            version: CssomInputVersion {
                role: CssomInputRole::ParserMode,
                identity: "parser".into(),
                revision: 1,
            },
            data: CssomInputData::ParserMode(parser),
        }],
        linked: vec![],
    }
}
fn batch(owner: &CssomStore) -> CssomBatch {
    owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot()))
        .unwrap()
}

#[test]
fn report_ingress_preserves_recovery_and_shared_original_after_owner_drop() {
    let raw = Arc::new("@unknown x;\r\n.a { color: red }".encode_utf16().collect());
    let report = CssParser::new(CssParserContext::default(), Default::default())
        .parse_sheet_utf16(raw, CssUtf16InputLimits::default(), || Ok::<_, ()>(()))
        .unwrap();
    let source = report.syntax().parse_source().unwrap().source().clone();
    let diagnostics = report.diagnostics().to_vec();
    let mut owner = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = batch(&owner);
    let ticket = edit.create_parsed_sheet_report(report, inputs()).unwrap();
    let commit = owner.commit(edit).unwrap();
    let id = commit.sheet(&ticket).unwrap();
    let snapshot = owner.snapshot();
    let sheet = snapshot.sheet(id).unwrap();
    assert_eq!(sheet.diagnostics(), diagnostics);
    assert!(
        sheet.diagnostics()[0]
            .source()
            .unwrap()
            .same_snapshot(&source)
    );
    assert!(
        sheet
            .authored()
            .parse_source()
            .unwrap()
            .source()
            .same_snapshot(&source)
    );
    drop(owner);
    assert!(
        sheet.diagnostics()[0]
            .source()
            .unwrap()
            .original_utf16()
            .is_some()
    );
}

#[test]
fn empty_and_fully_recovered_reports_check_full_context_including_glyph_choice() {
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let svg = CssParserContext::default().with_svg_glyph_orientation_vertical();
    for actual in [quirks, svg] {
        for text in ["", "@unknown value;"] {
            let mut owner =
                CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
            let before = owner.snapshot();
            let mut edit = batch(&owner);
            assert!(matches!(
                edit.create_parsed_sheet_report(
                    CssParser::new(actual, Default::default()).parse_sheet(text),
                    inputs()
                ),
                Err(CssomError::InvalidInput(
                    "stylesheet parser context mismatch"
                ))
            ));
            assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
            assert_eq!(owner.snapshot().revision(), before.revision());
            let mut owner = CssomStore::new(CssomLimits::default(), context(actual)).unwrap();
            let mut edit = batch(&owner);
            edit.create_parsed_sheet_report(
                CssParser::new(actual, Default::default()).parse_sheet(text),
                inputs(),
            )
            .unwrap();
            owner.commit(edit).unwrap();
        }
    }
}

#[test]
fn complete_input_limit_does_not_depend_on_surviving_rules() {
    for text in ["/*12345*/", "@unknown;"] {
        let mut owner = CssomStore::new(
            CssomLimits {
                max_input_bytes: text.len() - 1,
                ..Default::default()
            },
            CssomContext::default(),
        )
        .unwrap();
        let mut edit = batch(&owner);
        assert!(matches!(
            edit.create_parsed_sheet_report(CssParser::default().parse_sheet(text), inputs()),
            Err(CssomError::Limit {
                resource: "parse input bytes",
                ..
            })
        ));
        assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    }
}

#[test]
fn report_import_rejects_actual_resource_recovery_atomically() {
    let text = format!("{}x{}", "@media all{".repeat(258), "}".repeat(258));
    let report = CssParser::default().parse_sheet(&text);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|value| matches!(value.action(), CssRecoveryAction::StopAtNestingLimit))
    );
    let mut owner = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.create_parsed_sheet_report(report, inputs()),
        Err(CssomError::ParseResource(_))
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
}

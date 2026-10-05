#![forbid(unsafe_code)]
//! Authored rule composition, cumulative resource and immutable graph contracts.
//! CSS Syntax 3 §9.3/10 and the specified-output provider contracts.
use std::error::Error;
use surgeist_css::{
    CssMediaRule, CssNamespaceContext, CssRecoveryAction, CssRule, CssSheet,
    CssSpecifiedRuleSerializationErrorKind as RuleError,
    CssSpecifiedValueSerializationError as ValueError,
    CssSpecifiedValueSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationLimits as Limits, parse_sheet,
};

fn sheet(source: &str) -> CssSheet {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

fn exact(source: &str, expected: &str) {
    let input = sheet(source);
    let before = input.clone();
    assert_eq!(input.to_specified_css().unwrap(), expected);
    assert_eq!(input, before);
    // Supplemental semantic stability; the independent text above is the oracle.
    assert_eq!(sheet(expected).to_specified_css().unwrap(), expected);
}

#[test]
fn existing_namespace_and_font_providers_remain_callable_controls() {
    exact(
        "@namespace n 'u';@font-face{}",
        "@namespace n url(\"u\");\n@font-face { }",
    );
}

#[test]
fn immutable_style_output_preserves_duplicate_importance_and_child_run_order() {
    exact(
        ".parent{opacity:.25!important;opacity:.5;& .child{opacity:1}opacity:.75!important;opacity:0}",
        ".parent { opacity: 0.25 !important; opacity: 0.5; & .child { opacity: 1; } opacity: 0.75 !important; opacity: 0; }",
    );
}

#[test]
fn groups_compose_existing_typed_providers_without_losing_sibling_order() {
    exact(
        "@media PRINT,ONLY SCREEN{@supports (display: grid){@layer ink{@font-face{font-weight:500}@color-profile --x{}}}@font-face{font-weight:600}}",
        "@media print, only screen { @supports (display: grid) { @layer ink { @font-face { font-weight: 500; } @color-profile --x { } } } @font-face { font-weight: 600; } }",
    );
}

#[test]
fn recovered_media_members_emit_not_all_in_place_and_keep_report_origins() {
    let source = "/* 😀 */\r\n@media SCREEN,???,PRINT,&test,speech{.child{opacity:.5}}";
    let report = parse_sheet(source);
    let before = report.clone();
    assert_eq!(report.diagnostics().len(), 2);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|item| item.action() == CssRecoveryAction::ReplaceMediaQueryWithNever)
    );
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("media")
    };
    let origins = media
        .query()
        .queries()
        .iter()
        .map(|query| query.origin().clone())
        .collect::<Vec<_>>();
    let expected = "@media screen, not all, print, not all, speech { .child { opacity: 0.5; } }";
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    let reparsed = sheet(expected);
    let [CssRule::Media(emitted)] = reparsed.rules() else {
        panic!("media")
    };
    assert_eq!(
        emitted.query().serialize_cssom().unwrap().as_css(),
        "screen, not all, print, not all, speech"
    );
    assert_eq!(
        media
            .query()
            .queries()
            .iter()
            .map(|query| query.origin().clone())
            .collect::<Vec<_>>(),
        origins
    );
    // Recovered authored members are not graph-equal to clean canonical false members.
    assert_ne!(emitted.query(), media.query());
}

#[test]
fn valid_symbolic_media_conditions_keep_grouping_case_and_member_order() {
    exact(
        "@media NOT FuTuRe,(FuTuRe:ACTIVE),Future(\"A\"),NOT ((COLOR) OR (MONOCHROME)){}",
        "@media not future, (future: ACTIVE), Future(\"A\"), not ((color) or (monochrome)) { }",
    );
}

#[test]
fn invalid_declaration_recovery_drops_only_the_invalid_occurrence() {
    let report = parse_sheet(".x{opacity:.25;opacity:bad;opacity:.5!important}");
    let before = report.clone();
    assert!(!report.is_clean());
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        ".x { opacity: 0.25; opacity: 0.5 !important; }"
    );
    assert_eq!(report, before);
}

#[test]
fn checked_media_and_sheet_reuse_parsed_children_without_inventing_provenance() {
    let parsed = sheet("/* 😀 */\r\n@media PRINT{@font-face{font-weight:500}}");
    let [CssRule::Media(original)] = parsed.rules() else {
        panic!("media")
    };
    let checked = CssMediaRule::try_new(
        original.query().clone(),
        original.rules().to_vec(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert!(checked.position().is_none());
    assert_eq!(checked.rules(), original.rules());
    let input = CssSheet::try_from_rules(vec![CssRule::Media(checked)]).unwrap();
    let before = input.clone();
    assert_eq!(
        input.to_specified_css().unwrap(),
        "@media print { @font-face { font-weight: 500; } }"
    );
    assert_eq!(input, before);
}

#[test]
fn logical_utf8_output_omits_encoding_metadata_and_preserves_the_input_record() {
    for source in [
        "@charset \"windows-1252\";@namespace n '日本';",
        "@charset \"UTF-8\";",
    ] {
        let parsed = sheet(source);
        let before = parsed.clone();
        assert!(parsed.encoding().is_some());
        let checked = CssSheet::try_from_rules(parsed.rules().to_vec()).unwrap();
        assert!(checked.encoding().is_none());
        let expected = if parsed.rules().is_empty() {
            ""
        } else {
            "@namespace n url(\"日本\");"
        };
        assert_eq!(parsed.to_specified_css().unwrap(), expected);
        assert_eq!(checked.to_specified_css().unwrap(), expected);
        assert_eq!(parsed, before);
    }
}

#[test]
fn utf8_byte_budget_counts_multibyte_output_and_sheet_separator_exactly_once() {
    let input = sheet("@namespace n '日本';@media SCREEN{}");
    let before = input.clone();
    let expected = "@namespace n url(\"日本\");\n@media screen { }";
    assert!(expected.len() > expected.chars().count());
    assert_eq!(
        input
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len()))
            .unwrap(),
        expected
    );
    let error = input
        .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len() - 1))
        .unwrap_err();
    assert_eq!(error.kind(), RuleError::Resource(Kind::ByteLimit));
    assert_eq!(error.rule_index(), Some(1));
    assert!(error.source().is_some());
    assert_eq!(input, before);
    assert_eq!(input.to_specified_css().unwrap(), expected);
}

#[test]
fn namespace_node_control_proves_shared_sheet_aggregate_and_rule_accounting() {
    let input = sheet("@namespace a 'x';@namespace b 'y';");
    let before = input.clone();
    let expected = "@namespace a url(\"x\");\n@namespace b url(\"y\");";
    assert_eq!(
        input
            .to_specified_css_with_limits(Limits::new(7, 7, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(6, 7, expected.len()), Kind::InputNodeLimit),
        (Limits::new(7, 6, expected.len()), Kind::ProjectionNodeLimit),
    ] {
        assert!(
            input
                .rules()
                .iter()
                .all(|rule| rule.to_specified_css_with_limits(limits).is_ok())
        );
        let error = input.to_specified_css_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), RuleError::Resource(kind));
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<ValueError>()
                .unwrap()
                .kind(),
            kind
        );
        assert_eq!(input, before);
    }
}

#[test]
fn media_provider_work_shares_nodes_with_preceding_sibling_rules() {
    let input = sheet("@namespace a 'x';@media screen{}");
    let before = input.clone();
    // Namespace alone consumes three nodes. The sheet adds one; the second rule
    // must fail with four total nodes, regardless of additional provider work.
    for (limits, kind) in [
        (Limits::new(4, usize::MAX, 256), Kind::InputNodeLimit),
        (Limits::new(usize::MAX, 4, 256), Kind::ProjectionNodeLimit),
    ] {
        assert!(
            input.rules()[0]
                .to_specified_css_with_limits(limits)
                .is_ok()
        );
        let error = input.to_specified_css_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), RuleError::Resource(kind));
        assert_eq!(error.rule_index(), Some(1));
        assert!(error.source().is_some());
        assert_eq!(input, before);
    }
}

#[test]
fn finite_deep_grouping_is_emitted_and_stops_atomically_at_shared_work_limits() {
    for depth in [63, 64, 65, 128, 255] {
        let source = format!(
            "{}@font-face{{}}{}",
            "@media screen{".repeat(depth),
            "}".repeat(depth)
        );
        let input = sheet(&source);
        let before = input.clone();
        let expected = format!(
            "{}@font-face {{ }}{}",
            "@media screen { ".repeat(depth),
            " }".repeat(depth)
        );
        assert_eq!(input.to_specified_css().unwrap(), expected);
        for (limits, kind) in [
            (
                Limits::new(1, usize::MAX, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(usize::MAX, 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
        ] {
            let error = input.rules()[0]
                .to_specified_css_with_limits(limits)
                .unwrap_err();
            assert_eq!(error.kind(), RuleError::Resource(kind));
            assert_eq!(error.rule_index(), None);
            assert_eq!(input, before);
        }
    }
}

#[test]
fn nested_provider_failure_points_to_the_enclosing_top_level_rule() {
    let input = sheet(
        "@namespace n 'u';@media screen{@supports (display: grid){@font-face{font-weight:500}}}",
    );
    let before = input.clone();
    let expected = "@namespace n url(\"u\");\n@media screen { @supports (display: grid) { @font-face { font-weight: 500; } } }";
    let error = input
        .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len() - 1))
        .unwrap_err();
    assert_eq!(error.kind(), RuleError::Resource(Kind::ByteLimit));
    assert_eq!(error.rule_index(), Some(1));
    assert!(error.source().is_some());
    assert_eq!(input, before);
    assert_eq!(input.to_specified_css().unwrap(), expected);
}

#[test]
fn condition_providers_keep_authored_grouping_and_comment_boundaries() {
    let supports = sheet("@supports ((display:/*keep*/grid)){}");
    let [CssRule::Supports(rule)] = supports.rules() else {
        panic!("supports")
    };
    assert_eq!(
        rule.condition().serialize().unwrap().as_css(),
        " ((display:/*keep*/grid))"
    );
    assert_eq!(
        supports.to_specified_css().unwrap(),
        "@supports ((display:/*keep*/grid)) { }"
    );
    let container = sheet("@container (width/**/>/**/1px){}");
    let [CssRule::Container(rule)] = container.rules() else {
        panic!("container")
    };
    assert_eq!(
        rule.prelude().serialize().unwrap().as_css(),
        " (width/**/>/**/1px)"
    );
    assert_eq!(
        container.to_specified_css().unwrap(),
        "@container (width/**/>/**/1px) { }"
    );
}

#[test]
fn recovered_media_projection_cost_preserves_its_typed_provider_origin() {
    let report = parse_sheet("@media screen,???,print{}");
    let before = report.clone();
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("media")
    };
    // Provider-only control: aggregate 1 + screen 2 + recovered query 1 + print 2.
    // Recovered query generates two additional keyword projection nodes.
    let expected = "screen, not all, print";
    assert_eq!(
        rule.query()
            .serialize_cssom_with_limits(Limits::new(6, 8, expected.len()))
            .unwrap()
            .as_css(),
        expected
    );
    let provider = rule
        .query()
        .serialize_cssom_with_limits(Limits::new(6, 7, expected.len()))
        .unwrap_err();
    assert!(matches!(
        provider,
        surgeist_css::CssMediaCssomSerializationError::Resource { .. }
    ));
    // A sheet and enclosing media rule add work; the provider may not reset it.
    let error = report
        .syntax()
        .to_specified_css_with_limits(Limits::new(usize::MAX, 6, 256))
        .unwrap_err();
    assert_eq!(error.kind(), RuleError::Resource(Kind::ProjectionNodeLimit));
    assert_eq!(error.rule_index(), Some(0));
    let mut source = error.source();
    let mut media_error = None;
    while let Some(cause) = source {
        if let Some(value) = cause.downcast_ref::<surgeist_css::CssMediaCssomSerializationError>() {
            media_error = Some(value);
            break;
        }
        source = cause.source();
    }
    let media_error = media_error.expect("preserved typed media provider source");
    assert_eq!(media_error.origin(), rule.query().queries()[1].origin());
    assert_eq!(report, before);
}

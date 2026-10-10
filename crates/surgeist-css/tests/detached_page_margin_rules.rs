#![forbid(unsafe_code)]
//! Complete original-input Page margin insertion provider (#1087).
use surgeist_css::*;

fn names(rule: &CssMarginRule) -> Vec<&str> {
    rule.declarations()
        .properties()
        .iter()
        .map(|d| match d.property_name() {
            CssPropertyNameRef::Known(p) => p.canonical_name(),
            CssPropertyNameRef::Custom(n) => n.as_str(),
            _ => panic!(),
        })
        .collect()
}

#[test]
fn detached_margin_accepts_actual_decoded_names_and_composes_checked_selected_body() {
    for (source, name) in [
        ("@top-left { content:'X' }", CssMarginBox::TopLeft),
        ("@BOTTOM-RIGHT{content:'X'}", CssMarginBox::BottomRight),
        ("@\\74 op-left{content:'X'}", CssMarginBox::TopLeft),
        ("/**/ @top-left-corner {} /**/", CssMarginBox::TopLeftCorner),
    ] {
        let report = parse_page_margin_rule(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let rule = report.syntax().as_ref().unwrap();
        assert_eq!(rule.name(), name);
    }
    let report = parse_page_margin_rule("@top-left {content:'X'; color:red !important}");
    let rule = report.syntax().as_ref().unwrap();
    let selected = rule.declarations().try_specified_properties().unwrap();
    assert_eq!(
        CssMarginRuleView::try_new(rule.name(), &selected)
            .unwrap()
            .serialize_cssom()
            .unwrap(),
        "@top-left { content: \"X\"; color: red !important; }"
    );
}

#[test]
fn local_invalid_neighbors_descriptors_and_structural_units_recover_under_margin_grammar() {
    let report = parse_page_margin_rule(
        "@top-left { content:'X'; size:A4; display:block; color:bogus; @unknown {} margin-left:2px; --X: RAW; color:blue !important }",
    );
    let rule = report.syntax().as_ref().unwrap();
    assert!(!report.is_clean());
    assert_eq!(names(rule), ["content", "margin-left", "--X", "color"]);
    assert_eq!(
        rule.declarations().properties()[3].importance(),
        CssImportance::Important
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropDeclaration)
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropAtRule)
    );
    assert_eq!(
        rule.declarations().serialize_cssom().unwrap(),
        "content: \"X\"; margin-left: 2px; --X: RAW; color: blue !important;"
    );
}

#[test]
fn complete_rule_framing_and_empty_prelude_reject_extra_input_without_hierarchy_relabeling() {
    for source in [
        "",
        " /**/ ",
        "{content:'x'}",
        "@top-left;",
        "@top-left",
        "@top-left bad {content:'x'}",
        "@top-left(){}",
        "@top-left{};",
        "@top-left{} @bottom-left{}",
        "@top-left{} junk",
        "@top-left{} }",
    ] {
        let report = parse_page_margin_rule(source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|d| !matches!(d.error().kind(), ErrorKind::InvalidAtRulePlacement(_))),
            "{source}: {report:?}"
        );
    }
}

#[test]
fn preliminary_syntax_and_typed_page_placement_or_unsupported_name_remain_distinct() {
    let candidate = classify_rule_syntax("@page {}");
    assert!(candidate.is_clean());
    let placed = candidate
        .syntax()
        .as_ref()
        .unwrap()
        .admit_page_margin_rule();
    assert!(placed.syntax().is_none());
    assert!(
        placed
            .diagnostics()
            .iter()
            .any(|d| matches!(d.error().kind(), ErrorKind::InvalidAtRulePlacement(_)))
    );
    let unknown = classify_rule_syntax("@unknown {}");
    assert!(unknown.is_clean());
    let unsupported = unknown.syntax().as_ref().unwrap().admit_page_margin_rule();
    assert!(unsupported.syntax().is_none());
    assert!(
        unsupported
            .diagnostics()
            .iter()
            .any(|d| matches!(d.error().kind(), ErrorKind::UnknownAtRule(_)))
    );
    let raw = "@top-left {content:'x'}";
    let candidate = classify_rule_syntax(raw).into_parts().0.unwrap();
    assert!(candidate.admit_page_margin_rule().syntax().is_some());
    // Outside-Page destinations stay with the established generic/domain owners.
    assert!(
        candidate
            .admit(
                &CssNamespaceContext::default(),
                CssRuleAdmissionContext::Stylesheet
            )
            .syntax()
            .is_none()
    );
    assert!(
        candidate
            .admit(
                &CssNamespaceContext::default(),
                CssRuleAdmissionContext::Group
            )
            .syntax()
            .is_none()
    );
    assert!(
        parse_page_margin_rule("@page{} @top-left{}")
            .syntax()
            .is_none()
    );
}

#[test]
fn original_rule_and_declaration_origins_outlive_the_input_without_coordinate_shift() {
    let source = "/*😀*/\r\n@\\74 op-left { content:'X'; bogus:bad; color:red } /**/";
    let candidate = {
        let input = source.to_owned();
        classify_rule_syntax(&input).into_parts().0.unwrap()
    };
    assert_eq!(candidate.origin().source().as_str(), source);
    assert_eq!(candidate.origin().span().start().byte_offset().value(), 10);
    let report = candidate.admit_page_margin_rule();
    let rule = report.syntax().as_ref().unwrap();
    assert_eq!(rule.position().unwrap().byte_offset().value(), 10);
    assert_eq!(rule.position().unwrap().line().value(), 1);
    assert_eq!(rule.detached_origin().unwrap(), candidate.origin());
    let name = rule.declarations().properties()[0].parsed_name().unwrap();
    assert_eq!(name.source().as_str(), source);
    assert_eq!(name.span().start().byte_offset().value(), 25);
    assert_eq!(
        report.diagnostics()[0].span().start().byte_offset().value(),
        38
    );
    assert_eq!(
        rule.declarations().properties()[1]
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        source
    );
    assert_eq!(candidate.admit_page_margin_rule(), report);
}

#[test]
fn empty_detached_child_retains_its_complete_occurrence_after_candidate_drop() {
    let source = " /*😀*/ @top-left{} /**/ ";
    let rule = {
        let input = source.to_owned();
        parse_page_margin_rule(&input).into_parts().0.unwrap()
    };
    assert!(rule.declarations().is_empty());
    let origin = rule.detached_origin().unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 10);
    assert_eq!(origin.span().end().byte_offset().value(), 21);
    let constructed = CssMarginRule::new(rule.name(), rule.declarations().clone());
    assert!(constructed.detached_origin().is_none());
    assert!(constructed.position().is_none());
}

#[test]
fn actual_document_context_and_eof_recovery_are_retained() {
    let context = CssParserContext::new(CssParserMode::Quirks);
    let source = "@top-left { margin:1px; --x:func(1";
    let report = context.parse_page_margin_rule(source);
    let rule = report.syntax().as_ref().unwrap();
    assert_eq!(names(rule), ["margin", "--x"]);
    assert_eq!(
        rule.declarations().properties()[0].parser_context(),
        context
    );
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
            .count(),
        2
    );
    assert_eq!(
        report,
        context.parse_page_margin_rule_with_limits(source, CssComponentValueLimits::default())
    );
    let candidate = classify_rule_syntax(source).into_parts().0.unwrap();
    assert_eq!(
        report,
        candidate.admit_page_margin_rule_with_context(context)
    );
}

#[test]
fn whole_input_limits_fail_atomically_and_candidate_admission_does_not_reset_charges() {
    let source = "@top-left{color:red}";
    // At-keyword, block, ident, colon, ident; actual block contributes depth one.
    let adequate = CssComponentValueLimits::try_new(1, 5, source.len()).unwrap();
    let candidate = classify_rule_syntax_with_limits(source, adequate)
        .into_parts()
        .0
        .unwrap();
    assert_eq!(candidate.component_count(), 5);
    assert!(candidate.admit_page_margin_rule().is_clean());
    for (limits, cause) in [
        (
            CssComponentValueLimits::try_new(1, 4, source.len()).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(0, 5, source.len()).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 5, source.len() - 1).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let report = parse_page_margin_rule_with_limits(source, limits);
        assert!(report.syntax().is_none());
        assert!(report.diagnostics().iter().any(|d| matches!(d.error().kind(), ErrorKind::InvalidComponentValue(error) if error.kind() == cause)), "{report:?}");
        let retry = parse_page_margin_rule_with_limits(source, adequate);
        assert!(retry.is_clean());
        assert_eq!(names(retry.syntax().as_ref().unwrap()), ["color"]);
        assert_eq!(candidate.component_count(), 5);
    }
    let trailing = "@top-left{color:red} /*charged trailing trivia*/";
    let failed = parse_page_margin_rule_with_limits(trailing, adequate);
    assert!(failed.syntax().is_none());
    assert!(failed.diagnostics().iter().any(|d| matches!(d.error().kind(), ErrorKind::InvalidComponentValue(error) if error.kind() == CssComponentValueErrorKind::ByteLimit)));
    assert!(parse_page_margin_rule(trailing).is_clean());
}

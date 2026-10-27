//! Complete original-input Page margin insertion provider (#1087).
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::component_failure;
use surgeist_css::component::CssComponentValueLimits;
use surgeist_css::conformance::{feature_metadata, property_support_metadata};
use surgeist_css::declaration::CssImportance;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::CssPropertyNameRef;
use surgeist_css::rule::page::{CssMarginBox, CssMarginRule, CssMarginRuleView};
use surgeist_css::rule::{CssRuleAdmissionContext, CssRuleSyntax};
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::{CssParser, CssParserContext, CssParserMode};

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
        let report = CssMarginRule::parse(&CssParser::default(), source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let rule = report.syntax().as_ref().unwrap();
        assert_eq!(rule.name(), name);
    }
    let report = CssMarginRule::parse(
        &CssParser::default(),
        "@top-left {content:'X'; color:red !important}",
    );
    let rule = report.syntax().as_ref().unwrap();
    let selected = rule
        .declarations()
        .try_specified_properties(Default::default())
        .unwrap();
    assert_eq!(
        CssMarginRuleView::try_new(rule.name(), &selected)
            .unwrap()
            .serialize_cssom(Default::default())
            .unwrap(),
        "@top-left { content: \"X\"; color: red !important; }"
    );
}

#[test]
fn local_invalid_neighbors_descriptors_and_structural_units_recover_under_margin_grammar() {
    let report = CssMarginRule::parse(
        &CssParser::default(),
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
        rule.declarations()
            .serialize_cssom(Default::default())
            .unwrap(),
        "content: \"X\"; margin-left: 2px; --X: RAW; color: blue !important;"
    );
}

/// A recognized property refused by the margin grammar names its own
/// support-catalog record, as `CssUnsupportedPropertyError::feature` documents.
#[test]
fn recognized_page_descriptor_property_in_margin_names_its_catalog_identity() {
    for spelling in ["size", "SIZE"] {
        let report = CssMarginRule::parse(
            &CssParser::default(),
            format!("@top-left {{ {spelling}: A4 }}"),
        );
        let errors = report
            .diagnostics()
            .iter()
            .filter_map(|diagnostic| match diagnostic.error().code() {
                CssErrorCode::UnsupportedProperty => {
                    let error = diagnostic.error();
                    Some(error)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let [error] = errors.as_slice() else {
            panic!("{spelling}: one unsupported property: {report:?}")
        };
        assert_eq!(error.property_name().unwrap().as_str(), spelling);
        let catalog = property_support_metadata("size").expect("size catalog record");
        assert_eq!(error.feature(), Some(catalog.feature().id()));
        assert!(feature_metadata(error.feature().unwrap().as_str()).is_some());
    }
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
        let report = CssMarginRule::parse(&CssParser::default(), source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|d| d.error().code() != CssErrorCode::InvalidAtRulePlacement),
            "{source}: {report:?}"
        );
    }
}

#[test]
fn preliminary_syntax_and_typed_page_placement_or_unsupported_name_remain_distinct() {
    let candidate = CssRuleSyntax::classify(&CssParser::default(), "@page {}");
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
            .any(|d| d.error().code() == CssErrorCode::InvalidAtRulePlacement)
    );
    let unknown = CssRuleSyntax::classify(&CssParser::default(), "@unknown {}");
    assert!(unknown.is_clean());
    let unsupported = unknown.syntax().as_ref().unwrap().admit_page_margin_rule();
    assert!(unsupported.syntax().is_none());
    assert!(
        unsupported
            .diagnostics()
            .iter()
            .any(|d| d.error().code() == CssErrorCode::UnknownAtRule)
    );
    let raw = "@top-left {content:'x'}";
    let candidate = CssRuleSyntax::classify(&CssParser::default(), raw)
        .into_parts()
        .0
        .unwrap();
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
        CssMarginRule::parse(&CssParser::default(), "@page{} @top-left{}")
            .syntax()
            .is_none()
    );
}

#[test]
fn original_rule_and_declaration_origins_outlive_the_input_without_coordinate_shift() {
    let source = "/*😀*/\r\n@\\74 op-left { content:'X'; bogus:bad; color:red } /**/";
    let candidate = {
        let input = source.to_owned();
        CssRuleSyntax::classify(&CssParser::default(), &input)
            .into_parts()
            .0
            .unwrap()
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
        CssMarginRule::parse(&CssParser::default(), &input)
            .into_parts()
            .0
            .unwrap()
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
    let report = CssMarginRule::parse(&CssParser::new(context, Default::default()), source);
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
        CssMarginRule::parse(
            &CssParser::new(context, CssComponentValueLimits::default()),
            source
        )
    );
    let candidate = CssRuleSyntax::classify(&CssParser::default(), source)
        .into_parts()
        .0
        .unwrap();
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
    let candidate = CssRuleSyntax::classify(
        &CssParser::new(CssParserContext::default(), adequate),
        source,
    )
    .into_parts()
    .0
    .unwrap();
    assert_eq!(candidate.component_count(), 5);
    assert!(candidate.admit_page_margin_rule().is_clean());
    for (limits, cause) in [
        (
            CssComponentValueLimits::try_new(1, 4, source.len()).unwrap(),
            ComponentFailure::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(0, 5, source.len()).unwrap(),
            ComponentFailure::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 5, source.len() - 1).unwrap(),
            ComponentFailure::ByteLimit,
        ),
    ] {
        let report =
            CssMarginRule::parse(&CssParser::new(CssParserContext::default(), limits), source);
        assert!(report.syntax().is_none());
        assert!(
            report.diagnostics().iter().any(|d| d
                .error()
                .component_error()
                .is_some_and(|error| component_failure(error) == cause)),
            "{report:?}"
        );
        let retry = CssMarginRule::parse(
            &CssParser::new(CssParserContext::default(), adequate),
            source,
        );
        assert!(retry.is_clean());
        assert_eq!(names(retry.syntax().as_ref().unwrap()), ["color"]);
        assert_eq!(candidate.component_count(), 5);
    }
    let trailing = "@top-left{color:red} /*charged trailing trivia*/";
    let failed = CssMarginRule::parse(
        &CssParser::new(CssParserContext::default(), adequate),
        trailing,
    );
    assert!(failed.syntax().is_none());
    assert!(failed.diagnostics().iter().any(|d| {
        d.error()
            .component_error()
            .is_some_and(|error| error.is_byte_limit())
    }));
    assert!(CssMarginRule::parse(&CssParser::default(), trailing).is_clean());
}

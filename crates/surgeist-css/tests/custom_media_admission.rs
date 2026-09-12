//! Pinned MQ5 #custom-mq, #mq-syntax and #error-handling; the profile's
//! imported Extensions 1 name grammar and adopted feature-candidate precedence.
//! These tests deliberately use only the public surface preceding custom media.
use surgeist_css::{
    CssMediaConditionKind, CssMediaQuery, CssMediaType, CssNormalizedItem, CssRecoveryAction,
    CssRule, CssUnknownMediaFeatureReason, normalize_sheet, parse_media_query,
    parse_media_query_list, parse_sheet, validate_sheet,
};

#[test]
fn root_definitions_retain_boolean_query_and_empty_bodies() {
    for body in ["true", "FALSE", "screen", "(color), (hover)", ""] {
        let source = format!("@custom-media --mode {body}; .after {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{body}: {:?}", report.diagnostics());
        assert_eq!(validate_sheet(&source).unwrap(), *report.syntax());
        assert_eq!(report.syntax().rules().len(), 2, "{body}");
        assert!(matches!(
            report.syntax().rules().last(),
            Some(CssRule::Style(_))
        ));
    }
}

#[test]
fn extension_names_include_bare_hyphens_and_escaped_identifiers() {
    for name in ["--", "------", "--Case", r"--\6d ode"] {
        let source = format!("@custom-media {name} true;");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{name}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), 1, "{name}");
    }
}

#[test]
fn duplicates_and_cycles_survive_normalization_in_authored_order() {
    let source = concat!(
        "@custom-media --a (--b);\n",
        "@custom-media --b (--a);\n",
        "@custom-media --a false;\n",
        "@custom-media --self (--self);"
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 4);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let expected: Vec<_> = source
        .match_indices("@custom-media")
        .map(|(at, _)| at)
        .collect();
    let positions: Vec<_> = normalized
        .items()
        .iter()
        .map(|item| {
            let CssNormalizedItem::Rule(rule) = item else {
                panic!("retained definition")
            };
            rule.position()
                .expect("original definition position")
                .byte_offset()
                .value()
        })
        .collect();
    assert_eq!(positions, expected);
}

#[test]
fn definitions_are_allowed_after_ordinary_root_style_rules() {
    let report = parse_sheet(".before { color: red; } @custom-media --x true;");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 2);
    assert!(matches!(
        report.syntax().rules().first(),
        Some(CssRule::Style(_))
    ));
}

#[test]
fn unrestricted_root_group_contexts_retain_definitions() {
    for group in [
        "@media screen",
        "@supports (display: grid)",
        "@container (width > 1px)",
        "@layer theme",
        "@scope (.root)",
    ] {
        let source = format!("{group} {{ @custom-media --x true; }} .after {{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{group}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), 2);
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let [
            CssNormalizedItem::Rule(group_context),
            CssNormalizedItem::Rule(definition),
            CssNormalizedItem::Rule(style),
        ] = normalized.items()
        else {
            panic!("group, definition and following empty style: {group}");
        };
        assert!(
            definition
                .parent()
                .expect("group parent")
                .same_context(group_context)
        );
        assert!(style.parent().is_none());
        assert_eq!(
            style.position().unwrap().byte_offset().value(),
            source.find(".after").unwrap()
        );
        assert_eq!(
            definition.position().unwrap().byte_offset().value(),
            source.find("@custom-media").unwrap()
        );
    }
}

#[test]
fn style_ancestors_reject_definitions_without_losing_parent_or_sibling() {
    for group in [
        "",
        "@media screen",
        "@supports (display: grid)",
        "@container (width > 1px)",
        "@layer theme",
        "@scope (.root)",
    ] {
        let nested = if group.is_empty() {
            "@custom-media --x true;".to_owned()
        } else {
            format!("{group} {{ @custom-media --x true; }}")
        };
        let source = format!(".parent {{ color: red; {nested} }} .after {{ color: blue; }}");
        let report = parse_sheet(&source);
        assert!(
            matches!(
                report.syntax().rules(),
                [CssRule::Style(_), CssRule::Style(_)]
            ),
            "{group}"
        );
        assert_eq!(
            report.diagnostics().len(),
            1,
            "{group}: {:?}",
            report.diagnostics()
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule,
            "{group}"
        );
    }
}

#[test]
fn retained_definition_closes_the_import_prefix() {
    let source = "@custom-media --x true; @import 'late.css'; .after { color: red; }";
    let report = parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 2);
    assert!(matches!(
        report.syntax().rules().last(),
        Some(CssRule::Style(_))
    ));
    assert!(
        !report
            .syntax()
            .rules()
            .iter()
            .any(|rule| matches!(rule, CssRule::Import(_)))
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
    assert_eq!(
        report.diagnostics()[0].span().start().byte_offset().value(),
        source.find("@import").unwrap()
    );
}

#[test]
fn malformed_definition_rules_leave_following_style_rules_eligible() {
    for invalid in [
        "@custom-media;",
        "@custom-media ordinary true;",
        "@custom-media 2px true;",
        "@custom-media --x true {}",
    ] {
        let source = format!("{invalid} .after {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(
            matches!(report.syntax().rules(), [CssRule::Style(_)]),
            "{invalid}"
        );
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule,
            "{invalid}"
        );
    }
}

#[test]
fn boolean_custom_references_are_not_unknown_ordinary_features() {
    for source in ["(--x)", "(--)", r"(--\78)"] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}");
        let CssMediaQuery::Condition(condition) = report.syntax() else {
            panic!("condition")
        };
        assert!(
            !matches!(
                condition.kind(),
                CssMediaConditionKind::UnknownFeature(_)
                    | CssMediaConditionKind::GeneralEnclosed(_)
            ),
            "custom reference: {source}"
        );
        assert_eq!(condition.position().unwrap().byte_offset().value(), 0);
    }
}

#[test]
fn matched_nonboolean_custom_features_recover_only_their_comma_member() {
    for invalid in [
        "(--x: 1)",
        "(--x > 1)",
        "(1 < --x)",
        "(1 < --x <= 2)",
        "(--x < future)",
    ] {
        let source = format!("{invalid}, screen");
        let report = parse_media_query_list(&source);
        let [CssMediaQuery::Never(_), CssMediaQuery::Typed(screen)] = report.syntax().queries()
        else {
            panic!("member recovery: {source}")
        };
        assert_eq!(screen.media_type(), CssMediaType::Screen);
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
    }
}

#[test]
fn committed_custom_context_errors_survive_nested_speculation() {
    for invalid in ["not ((--x: 1))", "screen and ((1 < --x))"] {
        let report = parse_media_query_list(&format!("{invalid}, screen"));
        assert!(
            matches!(report.syntax().queries(), [CssMediaQuery::Never(_), CssMediaQuery::Typed(screen)] if screen.media_type() == CssMediaType::Screen),
            "{invalid}"
        );
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
    }
}

#[test]
fn feature_first_width_rejects_custom_looking_values_with_unknown_truth() {
    for source in ["(width > --x)", "(width: --x)"] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssMediaQuery::Condition(condition) = report.syntax() else {
            panic!("condition")
        };
        let CssMediaConditionKind::UnknownFeature(feature) = condition.kind() else {
            panic!("ordinary width domain")
        };
        assert_eq!(feature.name(), "width");
        assert_eq!(feature.reason(), CssUnknownMediaFeatureReason::InvalidValue);
    }
}

#[test]
fn ambiguous_operands_keep_existing_feature_candidate_precedence() {
    let source = "(--x < width)";
    let report = parse_media_query(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssMediaQuery::Condition(condition) = report.syntax() else {
        panic!("condition")
    };
    let CssMediaConditionKind::UnknownFeature(feature) = condition.kind() else {
        panic!("ordinary width domain")
    };
    assert_eq!(feature.name(), "width");
    assert_eq!(feature.reason(), CssUnknownMediaFeatureReason::InvalidValue);
    let report = parse_media_query("(future < --x)");
    assert!(report.is_clean());
    let CssMediaQuery::Condition(condition) = report.syntax() else {
        panic!("condition")
    };
    let CssMediaConditionKind::UnknownFeature(feature) = condition.kind() else {
        panic!("ordinary future feature")
    };
    assert_eq!(feature.name(), "future");
    assert_eq!(feature.reason(), CssUnknownMediaFeatureReason::UnknownName);
    for source in ["(--x:)", "(--x: arbitrary())"] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(
            matches!(report.syntax(), CssMediaQuery::Condition(condition) if matches!(condition.kind(), CssMediaConditionKind::GeneralEnclosed(_))),
            "{source}"
        );
    }
}

#[test]
fn eof_closed_query_body_retains_definition_and_closure_diagnostic() {
    let report = parse_sheet("@custom-media --x (color");
    assert_eq!(report.syntax().rules().len(), 1);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
}

#[test]
fn invalid_definition_media_member_preserves_definition_and_sibling() {
    let source = "@custom-media --x (color), ???, screen; .after { color: red; }";
    let report = parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 2);
    assert!(matches!(
        report.syntax().rules().last(),
        Some(CssRule::Style(_))
    ));
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
}

#[test]
fn eof_statement_termination_does_not_invent_an_enclosure_diagnostic() {
    let report = parse_sheet("@custom-media --x true");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 1);
}

use surgeist_css::{
    CssContainerConditionKind, CssContainerFeatureQuery, CssErrorCode, CssRecoveryAction, CssRule,
    parse_sheet, validate_sheet,
};

// Pinned Conditional Rules 5, section 5.4:
// https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule
// A query-in-parens can contain another complete container query. `not`
// prefixes one query-in-parens; it is not an ungrouped boolean operand.

#[test]
fn grouped_size_queries_retain_their_boolean_structure() {
    let source = "@container ((width > 1px) or (height > 2px)) and (inline-size > 3px) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(validate_sheet(source).is_ok());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("one accepted container rule must survive its empty body");
    };
    let CssContainerConditionKind::And(and) = rule.prelude().entries()[0].query().unwrap().kind()
    else {
        panic!("the outer condition must retain conjunction");
    };
    let [group, inline] = and.conditions() else {
        panic!("two operands")
    };
    let CssContainerConditionKind::Parenthesized(disjunction) = group.kind() else {
        panic!("explicit outer grouping")
    };
    let CssContainerConditionKind::Or(or) = disjunction.kind() else {
        panic!("grouped disjunction")
    };
    assert!(matches!(
        inline.kind(),
        CssContainerConditionKind::Feature(CssContainerFeatureQuery::InlineSize(_))
    ));
    let [width, height] = or.conditions() else {
        panic!("two disjuncts")
    };
    assert!(matches!(
        width.kind(),
        CssContainerConditionKind::Feature(CssContainerFeatureQuery::Width(_))
    ));
    assert!(matches!(
        height.kind(),
        CssContainerConditionKind::Feature(CssContainerFeatureQuery::Height(_))
    ));
}

#[test]
fn negation_can_target_a_grouped_query() {
    let source = "@container not ((width > 1px) and (height > 2px)) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("one container rule expected");
    };
    let CssContainerConditionKind::Not(operand) =
        rule.prelude().entries()[0].query().unwrap().kind()
    else {
        panic!("expected negation");
    };
    let CssContainerConditionKind::Parenthesized(grouped) = operand.kind() else {
        panic!("negated grouping")
    };
    assert!(
        matches!(grouped.kind(), CssContainerConditionKind::And(list) if list.conditions().len() == 2)
    );
}

#[test]
fn ungrouped_negations_drop_only_the_invalid_outer_rule() {
    for query in [
        "not not (width > 1px)",
        "not (width > 1px) and (height > 2px)",
        "(width > 1px) and not (height > 2px)",
        "(width > 1px) or not (height > 2px)",
    ] {
        let source = format!(".before {{}} @container {query} {{ .child {{}} }} .after {{}}");
        let report = parse_sheet(&source);
        assert!(validate_sheet(&source).is_err(), "{query}");
        assert!(
            matches!(
                report.syntax().rules(),
                [CssRule::Style(_), CssRule::Style(_)]
            ),
            "{query}: {:?}",
            report.syntax().rules()
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one outer-rule diagnostic expected for {query}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePrelude
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    }
}

#[test]
fn simple_queries_and_consistent_operator_lists_remain_accepted() {
    for query in [
        "(width > 1px)",
        "not (width > 1px)",
        "(width > 1px) and (height > 2px) and (inline-size > 3px)",
        "(width > 1px) or (height > 2px) or (inline-size > 3px)",
        "style(--theme)",
    ] {
        let source = format!("@container {query} {{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{query}: {:?}", report.diagnostics());
        assert!(matches!(report.syntax().rules(), [CssRule::Container(_)]));
    }
}

#[test]
fn mixed_operators_without_grouping_remain_invalid() {
    let source = "@container (width > 1px) and (height > 2px) or (inline-size > 3px) {}";
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    assert!(report.syntax().rules().is_empty());
    assert_eq!(report.diagnostics().len(), 1);
}

#[test]
fn grouped_queries_observe_the_existing_component_nesting_limit() {
    let accepted = format!(
        "@container {}width > 1px{} {{}}",
        "(".repeat(256),
        ")".repeat(256),
    );
    let report = parse_sheet(&accepted);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(report.syntax().rules(), [CssRule::Container(_)]));

    let rejected = format!(
        "@container {}width > 1px{} {{}} .after {{}}",
        "(".repeat(257),
        ")".repeat(257),
    );
    let report = parse_sheet(&rejected);
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.error().code() == CssErrorCode::NestingLimit)
        .expect("the first component beyond the shared depth limit must be reported");
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        "@container ".len() + 256,
    );
}

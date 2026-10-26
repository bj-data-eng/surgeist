use crate::css::support::authored_syntax::assert_strict_rejection;
use surgeist_css::{
    CssConditionKind, CssContainerFeatureQuery, CssContainerLeaf, CssErrorCode, CssParser,
    CssRecoveryAction, CssRule,
};

// Pinned Conditional Rules 5, section 5.4:
// https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule
// A query-in-parens can contain another complete container query. `not`
// prefixes one query-in-parens; it is not an ungrouped boolean operand.

#[test]
fn grouped_size_queries_retain_their_boolean_structure() {
    let source = "@container ((width > 1px) or (height > 2px)) and (inline-size > 3px) {}";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        CssParser::default()
            .parse_sheet(source)
            .into_validation_result()
            .unwrap(),
        *report.syntax()
    );
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("one accepted container rule must survive its empty body");
    };
    let CssConditionKind::And(and) = rule.prelude().entries()[0].query().unwrap().kind() else {
        panic!("the outer condition must retain conjunction");
    };
    let [group, inline] = and.conditions() else {
        panic!("two operands")
    };
    let CssConditionKind::Parenthesized(disjunction) = group.kind() else {
        panic!("explicit outer grouping")
    };
    let CssConditionKind::Or(or) = disjunction.kind() else {
        panic!("grouped disjunction")
    };
    assert!(matches!(
        inline.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::Feature(
            CssContainerFeatureQuery::InlineSize(_)
        ))
    ));
    let [width, height] = or.conditions() else {
        panic!("two disjuncts")
    };
    assert!(matches!(
        width.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::Feature(CssContainerFeatureQuery::Width(
            _
        )))
    ));
    assert!(matches!(
        height.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::Feature(CssContainerFeatureQuery::Height(
            _
        )))
    ));
}

#[test]
fn negation_can_target_a_grouped_query() {
    let source = "@container not ((width > 1px) and (height > 2px)) {}";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("one container rule expected");
    };
    let CssConditionKind::Not(operand) = rule.prelude().entries()[0].query().unwrap().kind() else {
        panic!("expected negation");
    };
    let CssConditionKind::Parenthesized(grouped) = operand.kind() else {
        panic!("negated grouping")
    };
    assert!(matches!(grouped.kind(), CssConditionKind::And(list) if list.conditions().len() == 2));
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
        let report = CssParser::default().parse_sheet(&source);
        assert_strict_rejection(
            CssParser::default()
                .parse_sheet(&source)
                .into_validation_result(),
            report.diagnostics(),
        );
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
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.is_clean(), "{query}: {:?}", report.diagnostics());
        assert!(matches!(report.syntax().rules(), [CssRule::Container(_)]));
    }
}

#[test]
fn mixed_operators_without_grouping_remain_invalid() {
    let source = "@container (width > 1px) and (height > 2px) or (inline-size > 3px) {}";
    let report = CssParser::default().parse_sheet(source);
    assert!(!report.is_clean());
    assert!(report.syntax().rules().is_empty());
    assert_eq!(report.diagnostics().len(), 1);
}

#[test]
fn grouped_queries_observe_the_component_nesting_limit() {
    let accepted = format!(
        "@container {}width > 1px{} {{}}",
        "(".repeat(256),
        ")".repeat(256),
    );
    let report = CssParser::default().parse_sheet(&accepted);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(report.syntax().rules(), [CssRule::Container(_)]));

    let rejected = format!(
        "@container {}width > 1px{} {{}} .after {{}}",
        "(".repeat(257),
        ")".repeat(257),
    );
    let report = CssParser::default().parse_sheet(&rejected);
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

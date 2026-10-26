//! Conditional Rules 5 section 9.1 forbids logical simplifications, including
//! removing unnecessary parentheses from the authored container query.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#the-csscontainerrule-interface
//! Section 5.4 permits explicit grouped operands and homogeneous operator lists.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule
use crate::css::support::authored_syntax::{component_list as components, component_token};
use surgeist_css::{
    CssBlockKind, CssComponentValue, CssComponentValueRef, CssCondition, CssContainerLeaf,
    CssErrorCode, CssGeneralEnclosed, CssParser, CssRecoveryAction, CssRule, CssValueOrigin,
};

fn parsed_condition(query: &str) -> CssCondition<CssContainerLeaf> {
    let source = format!("@container {query} {{}}");
    let report = CssParser::default().parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {report:?}");
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("one retained container rule: {source}: {report:?}");
    };
    rule.prelude().entries()[0].query().unwrap().clone()
}

fn group(items: Vec<CssComponentValue>) -> CssComponentValue {
    CssComponentValue::try_block(CssBlockKind::Parenthesis, components(items)).unwrap()
}

fn feature(name: &str, value: &str) -> CssComponentValue {
    group(vec![
        component_token(name),
        component_token(" "),
        component_token(">"),
        component_token(" "),
        component_token(value),
    ])
}

fn assert_programmatic(component: &CssComponentValue) {
    assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
    let children = match component.view() {
        CssComponentValueRef::Block(block) => Some(block.values()),
        CssComponentValueRef::Function(function) => Some(function.values()),
        _ => None,
    };
    if let Some(children) = children {
        for child in children.items() {
            assert_programmatic(child);
        }
    }
}

fn checked(component: CssComponentValue) -> CssCondition<CssContainerLeaf> {
    // No parsed-source components are supplied. Every opener, token and
    // synthesized delimiter has the same Programmatic provenance, so different
    // source snapshots or offsets cannot supply the required distinction.
    assert_programmatic(&component);
    CssCondition::<CssContainerLeaf>::try_from_enclosed(
        CssGeneralEnclosed::try_from_component(component).unwrap(),
    )
    .unwrap()
}

#[test]
fn parsed_redundant_size_groups_are_not_logically_simplified() {
    let simple = parsed_condition("(width > 1px)");
    let grouped = parsed_condition("((width > 1px))");
    let twice_grouped = parsed_condition("(((width > 1px)))");
    assert_ne!(simple, grouped, "one explicit grouping layer must survive");
    assert_ne!(grouped, twice_grouped, "each additional group must survive");
}

#[test]
fn parsed_redundant_boolean_groups_are_not_logically_simplified() {
    let simple = parsed_condition("(width > 1px) and (height > 2px)");
    let grouped = parsed_condition("((width > 1px) and (height > 2px))");
    assert_ne!(
        simple, grouped,
        "an explicit outer boolean group must survive"
    );
}

#[test]
fn checked_size_groups_remain_distinct_with_identical_programmatic_origins() {
    let size = feature("width", "1px");
    let simple = checked(size.clone());
    let grouped = checked(group(vec![size.clone()]));
    let twice_grouped = checked(group(vec![group(vec![size])]));
    assert_ne!(
        simple, grouped,
        "grouping cannot disappear during classification"
    );
    assert_ne!(grouped, twice_grouped, "every explicit group must survive");
}

#[test]
fn checked_style_groups_remain_distinct_with_identical_programmatic_origins() {
    let style =
        CssComponentValue::try_function("style", components(vec![component_token("--theme")]))
            .unwrap();
    let simple = checked(style.clone());
    let grouped = checked(group(vec![style]));
    assert_ne!(
        simple, grouped,
        "parentheses around a recognized function must survive"
    );
}

#[test]
fn checked_boolean_groups_remain_distinct_with_identical_programmatic_origins() {
    let conjunction = group(vec![
        feature("width", "1px"),
        component_token(" "),
        component_token("and"),
        component_token(" "),
        feature("height", "2px"),
    ]);
    let grouped = checked(conjunction.clone());
    let twice_grouped = checked(group(vec![conjunction]));
    assert_ne!(
        grouped, twice_grouped,
        "a redundant boolean group remains authored syntax"
    );
}

#[test]
fn checked_repeated_construction_preserves_equality_without_reordering_operands() {
    let width = feature("width", "1px");
    let height = feature("height", "2px");
    let ordered = group(vec![
        width.clone(),
        component_token(" "),
        component_token("and"),
        component_token(" "),
        height.clone(),
    ]);
    let reordered = group(vec![
        height,
        component_token(" "),
        component_token("and"),
        component_token(" "),
        width,
    ]);
    assert_eq!(checked(ordered.clone()), checked(ordered.clone()));
    assert_ne!(
        checked(ordered),
        checked(reordered),
        "authored operand order is not commuted"
    );
}

#[test]
fn explicit_grouping_allows_boolean_combinations_without_ungrouped_negation() {
    for query in [
        "not ((width > 1px) and (height > 2px))",
        "((width > 1px) or (height > 2px)) and (inline-size > 3px)",
        "(width > 1px) and (not (height > 2px))",
        "(width > 1px) and ((height > 2px) and (inline-size > 3px))",
        "((width > 1px) and (height > 2px)) and (inline-size > 3px)",
    ] {
        parsed_condition(query);
        let source = format!("@container {query} {{}}");
        assert_eq!(
            CssParser::default()
                .parse_sheet(&source)
                .into_validation_result()
                .unwrap(),
            *CssParser::default().parse_sheet(&source).syntax(),
            "{query}"
        );
    }
}

#[test]
fn ungrouped_operator_mixtures_reject_only_the_container_rule() {
    for query in [
        "not not (width > 1px)",
        "not (width > 1px) and (height > 2px)",
        "(width > 1px) and not (height > 2px)",
        "(width > 1px) and (height > 2px) or (inline-size > 3px)",
    ] {
        let source = format!(".before {{}} @container {query} {{ .inside {{}} }} .after {{}}");
        let report = CssParser::default().parse_sheet(&source);
        assert!(
            matches!(
                report.syntax().rules(),
                [CssRule::Style(_), CssRule::Style(_)]
            ),
            "{query}: {report:?}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected container prelude: {query}: {report:?}");
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropAtRule,
            "{query}"
        );
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePrelude,
            "{query}"
        );
        assert!(
            CssParser::default()
                .parse_sheet(&source)
                .into_validation_result()
                .is_err(),
            "{query}"
        );
    }
}

#![forbid(unsafe_code)]
//! Authored scroll-state grammar: Conditional5 (2025-10-30) sections 5.4 and 6.3.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#scroll-state-container
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule
//! Pending values apply the container-query var() extension in section 6.1 with
//! Variables1 whole-value deferral. Compound-value admission is the combined
//! interpretation of these sources, not a separate scroll-specific production.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#size-container
//! https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#using-variables
use surgeist_css::*;

fn assert_query(source: &str, recognized: bool) {
    let checked =
        CssContainerCondition::try_from_components(parse_component_values(source).unwrap())
            .unwrap();
    assert_eq!(
        !matches!(
            checked.kind(),
            CssContainerConditionKind::GeneralEnclosed(_)
        ),
        recognized,
        "checked {source}",
    );
    assert_eq!(checked.serialize().unwrap().as_css(), source);
    let input = format!("@container {source}{{}}");
    let report = parse_sheet(&input);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("retained container rule")
    };
    let parsed = rule.prelude().entries()[0].query().unwrap();
    assert_eq!(
        !matches!(parsed.kind(), CssContainerConditionKind::GeneralEnclosed(_)),
        recognized,
        "parsed {source}",
    );
    assert_eq!(parsed.serialize().unwrap().as_css(), source);
}

#[test]
fn four_scroll_features_admit_boolean_and_representative_domain_keywords() {
    for query in [
        "stuck",
        "snapped",
        "scrollable",
        "scrolled",
        "stuck: inline-start",
        "snapped: both",
        "scrollable: x",
        "scrolled: bottom",
        "STUCK: NONE",
        r"s\74 uck: t\6f p",
    ] {
        assert_query(&format!("scroll-state({query})"), true);
    }
}

#[test]
fn recursive_scroll_queries_retain_groups_logic_and_inner_unknowns() {
    for query in [
        "not (stuck)",
        "(stuck: top) and (snapped: x)",
        "(scrollable) or (scrolled)",
        "((stuck))",
        "(stuck) and future()",
        "(future: top)",
    ] {
        assert_query(&format!("scroll-state({query})"), true);
    }
}

#[test]
fn scroll_values_admit_variables_without_eager_fallback_domain_checks() {
    for query in [
        "stuck: var(--edge, banana)",
        "snapped: var(--axis, both)",
        "scrollable: var(--direction,)",
        "scrolled: var(--direction, var(--fallback, inline))",
    ] {
        assert_query(&format!("scroll-state({query})"), true);
    }
}

#[test]
fn compound_variable_values_are_retained_as_complete_pending_operands() {
    for query in ["stuck: top var(--empty)", "snapped: var(--a) var(--b)"] {
        assert_query(&format!("scroll-state({query})"), true);
    }
}

#[test]
fn fully_programmatic_scroll_features_use_the_same_checked_boundary() {
    let pending = CssComponentValue::try_function(
        "var",
        CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("--edge").unwrap(),
            CssComponentValue::try_token(",").unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    for (value, expected) in [
        (
            CssComponentValue::try_ident("top").unwrap(),
            "scroll-state(stuck:top)",
        ),
        (pending, "scroll-state(stuck:var(--edge,))"),
    ] {
        let values = CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("stuck").unwrap(),
            CssComponentValue::try_token(":").unwrap(),
            value,
        ])
        .unwrap();
        let function = CssComponentValue::try_function("scroll-state", values).unwrap();
        let condition = CssContainerCondition::try_from_components(
            CssComponentValues::try_new(vec![function]).unwrap(),
        )
        .unwrap();
        assert!(!matches!(
            condition.kind(),
            CssContainerConditionKind::GeneralEnclosed(_)
        ));
        assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(condition.serialize().unwrap().as_css(), expected);
        assert_query(expected, true);
    }
}

#[test]
fn invalid_discrete_domains_names_variables_and_logic_remain_opaque() {
    for query in [
        "stuck: both",
        "snapped: top",
        "scrollable: both",
        "scrolled: both",
        "stuck = top",
        "stuck > var(--edge)",
        "min-stuck: top",
        "future: var(--edge)",
        "var(--feature): top",
        "stuck: var(edge)",
        "stuck: var(--edge extra)",
        "snapped: var(--axis, var(fallback))",
        "(stuck) and not (snapped)",
        "(stuck) and (snapped) or (scrolled)",
    ] {
        assert_query(&format!("scroll-state({query})"), false);
    }
    assert_query("future-scroll(stuck: top)", false);
}

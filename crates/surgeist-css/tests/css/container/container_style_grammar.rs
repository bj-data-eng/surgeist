//! Authored style-query recognition follows Conditional Rules 5 (2025-10-30)
//! sections 5.4 and 6.2. Values remain syntax, without computed-value evaluation.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#style-container
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#typedef-declaration-value
use crate::css::support::authored_syntax::checked_container_condition as checked;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::query::container::CssContainerLeaf;
use surgeist_css::query::{CssCondition, CssConditionKind};
use surgeist_css::rule::CssRule;

fn assert_style(source: &str) {
    let condition = checked(source);
    assert!(
        matches!(
            condition.kind(),
            CssConditionKind::Leaf(CssContainerLeaf::Style(_))
        ),
        "recognized authored style query: {source}"
    );
    assert_eq!(
        condition
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );

    let sheet_source = format!("@container {source} {{ .x {{ color: red }} }}");
    let report = CssParser::default().parse_sheet(&sheet_source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("container rule")
    };
    let parsed = rule.prelude().entries()[0].query().unwrap();
    assert!(matches!(
        parsed.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::Style(_))
    ));
}

fn assert_opaque(source: &str) {
    let condition = checked(source);
    assert!(
        matches!(
            condition.kind(),
            CssConditionKind::Leaf(CssContainerLeaf::GeneralEnclosed(_))
        ),
        "unrecognized style enclosure: {source}"
    );
    assert_eq!(
        condition
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    let report = CssParser::default().parse_sheet(format!("@container {source} {{}}"));
    assert!(report.is_clean());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("retained container with opaque style operand")
    };
    assert!(matches!(
        rule.prelude().entries()[0].query().unwrap().kind(),
        CssConditionKind::Leaf(CssContainerLeaf::GeneralEnclosed(_))
    ));
}

#[test]
fn custom_features_and_unknown_enclosures_keep_their_classification() {
    for source in [
        "style(--theme)",
        "style(--theme: dark)",
        "style(--x: var(--fallback,))",
    ] {
        assert_style(source);
    }
    for source in [
        "style(future-property)",
        "style(1px < --size > 2px)",
        "style(--x: var(color))",
    ] {
        assert_opaque(source);
    }
}

#[test]
fn ordinary_property_identity_does_not_require_a_computed_value() {
    for source in [
        "style(COLOR)",
        "style(color: red)",
        "style(color: banana)",
        "style(glyph-orientation-vertical: 0deg)",
    ] {
        assert_style(source);
    }
}

#[test]
fn css_wide_values_are_retained_before_query_evaluation() {
    for source in [
        "style(color: initial)",
        "style(color: inherit)",
        "style(color: unset)",
        "style(color: revert)",
        "style(color: revert-layer)",
    ] {
        assert_style(source);
    }
}

#[test]
fn style_boolean_grammar_admits_groups_negation_and_inner_unknowns() {
    for source in [
        "style(not (--theme))",
        "style((--a) and (--b))",
        "style((--a) or (--b))",
        "style(((--a)))",
        "style((--a) and future(x))",
    ] {
        assert_style(source);
    }
}

#[test]
fn style_ranges_preserve_broad_operands_and_both_chain_directions() {
    for source in [
        "style(--size > 10px)",
        "style(1px < --size <= 10px)",
        "style(10px >= --size > 1px)",
        "style(1px < red)",
        "style(var(--size,) = 0)",
        "style(--x: a > b)",
    ] {
        assert_style(source);
    }
}

#[test]
fn explicit_declaration_value_production_counts_whitespace_but_not_comments() {
    // Syntax 3 sections 5.4.1 and 7.2 match component tokens directly here;
    // consume-a-declaration's whitespace trimming is not this production.
    for source in ["style(--x: )", "style(--x:/**/ )", "style(1px< )"] {
        assert_style(source);
    }
    for source in ["style(--x:)", "style(--x:/**/)", "style(1px<)"] {
        assert_opaque(source);
    }
}

#[test]
fn style_value_comparison_exclusions_reach_nested_tokens() {
    for source in [
        "style(--x: a >)",
        "style(--x: f(>))",
        "style(--x: var(--other, >))",
        "style(--x: red!important)",
    ] {
        assert_opaque(source);
    }
    assert_style("style(--x: f(\">\"))");
}

#[test]
fn programmatic_style_function_uses_the_same_admission_boundary() {
    let arguments = CssComponentValues::try_new(
        vec![
            CssComponentValue::try_token("color").unwrap(),
            CssComponentValue::try_token(":").unwrap(),
            CssComponentValue::try_token("red").unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    let function = CssComponentValue::try_function("style", arguments).unwrap();
    let values = CssComponentValues::try_new(vec![function], Default::default()).unwrap();
    let condition =
        CssCondition::<CssContainerLeaf>::try_from_components(values, Default::default()).unwrap();
    assert!(matches!(
        condition.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::Style(_))
    ));
    assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        condition
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "style(color:red)"
    );
}

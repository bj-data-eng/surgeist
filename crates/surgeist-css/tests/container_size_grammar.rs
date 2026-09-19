#![forbid(unsafe_code)]
//! Authored recognition, independently specified by Conditional 5 §§5.4/6.1,
//! MQ5 §§2.4/3, Values4 §§5.7/6/10, Variables1 §3, and the narrowly imported
//! Values5 §9 tree-counting definitions. Opaque fallback is valid query syntax.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#size-container
//! https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/#mq-syntax
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#ratios
//! https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#using-variables
//! https://www.w3.org/TR/2024/WD-css-values-5-20241111/#tree-counting
use surgeist_css::{
    CssComponentValue, CssComponentValues, CssContainerCondition, CssContainerConditionKind,
    CssGeneralEnclosed, CssRule, CssValueOrigin, parse_component_values, parse_sheet,
};

fn assert_classification(query: &str, recognized: bool) {
    let source = format!("@container {query} {{}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{query}: {:?}", report.diagnostics());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("retained container for {query}")
    };
    let parsed = rule.prelude().entries()[0].query().unwrap();
    let components = parse_component_values(query).unwrap();
    let checked = CssContainerCondition::try_from_components(components.clone()).unwrap();
    let enclosed = CssContainerCondition::try_from_enclosed(
        CssGeneralEnclosed::try_from_component(components.items()[0].clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(checked, enclosed, "checked entry points agree for {query}");
    for (route, condition) in [
        ("parsed", parsed),
        ("checked", &checked),
        ("enclosed", &enclosed),
    ] {
        let actual = match condition.kind() {
            CssContainerConditionKind::Feature(_) => true,
            CssContainerConditionKind::GeneralEnclosed(_) => false,
            other => panic!("unexpected {route} leaf for {query}: {other:?}"),
        };
        assert_eq!(actual, recognized, "{route} recognition for {query}");
        assert_eq!(condition.serialize().unwrap().as_css().trim(), query);
        let CssValueOrigin::Parsed(origin) = condition.origin() else {
            panic!("retained parsed origin for {route}: {query}")
        };
        let expected_source = if route == "parsed" { &source } else { query };
        assert_eq!(origin.source().as_str(), expected_source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            if route == "parsed" {
                "@container ".len()
            } else {
                0
            }
        );
    }
}

#[test]
fn scalar_plain_and_prefixed_features_remain_recognized() {
    for query in [
        "(width: 1px)",
        "(height > 2em)",
        "(inline-size: 0)",
        "(block-size <= 3rem)",
        "(min-width: 4px)",
        "(max-height: 5px)",
        "(min-inline-size: 6px)",
        "(max-block-size: 7px)",
        "(aspect-ratio: 16 / 9)",
        "(min-aspect-ratio: 4 / 3)",
        "(orientation: portrait)",
        "(ORIENTATION: LANDSCAPE)",
        r"(w\69 dth: 1px)",
    ] {
        assert_classification(query, true);
    }
}

#[test]
fn all_six_size_features_admit_boolean_context() {
    for name in [
        "width",
        "height",
        "inline-size",
        "block-size",
        "aspect-ratio",
        "orientation",
    ] {
        assert_classification(&format!("({name})"), true);
    }
}

#[test]
fn signed_and_exact_lengths_are_recognized_without_float_projection() {
    for query in [
        "(width: -2px)",
        "(height > -0.5em)",
        "(min-inline-size: -1e-999px)",
        "(max-block-size: 1e999px)",
        "(width: 123456789012345678901234567890.123456789px)",
        "(width: -0)",
    ] {
        assert_classification(query, true);
    }
}

#[test]
fn each_range_feature_admits_value_first_and_both_chain_directions() {
    for name in ["width", "height", "inline-size", "block-size"] {
        for query in [
            format!("(1px < {name})"),
            format!("(1px = {name})"),
            format!("(1px < {name} <= 2px)"),
            format!("(2px >= {name} > 1px)"),
            format!("(2px < {name} < 1px)"),
        ] {
            assert_classification(&query, true);
        }
    }
    for query in [
        "(1 / 2 < aspect-ratio)",
        "(1 / 2 < aspect-ratio <= 16 / 9)",
        "(16 / 9 >= aspect-ratio > 1 / 2)",
    ] {
        assert_classification(query, true);
    }
}

#[test]
fn ratios_admit_implicit_denominators_degenerate_zeros_and_exact_numbers() {
    for query in [
        "(aspect-ratio: 1.5)",
        "(aspect-ratio: 0 / 0)",
        "(aspect-ratio: 1 / 0)",
        "(aspect-ratio: -0 / 2)",
        "(aspect-ratio: 1e999 / 1e-999)",
        "(max-aspect-ratio: 2)",
    ] {
        assert_classification(query, true);
    }
}

#[test]
fn dimensionally_typed_math_remains_symbolic_in_size_operands() {
    for query in [
        "(width: calc(2em + 3px))",
        "(height > min(2rem, 10px))",
        "(inline-size: clamp(1px, 2em, 30px))",
        "(calc(1px * 2) < block-size <= calc(3px + 1em))",
        "(aspect-ratio: calc(4 / 2) / calc(3))",
        "(aspect-ratio: calc(-1) / 2)",
        "(width: calc(infinity * 1px))",
    ] {
        assert_classification(query, true);
    }
}

#[test]
fn relative_length_units_do_not_require_a_query_container_at_parse_time() {
    for unit in [
        "em", "rem", "ex", "ch", "cap", "ic", "lh", "rlh", "vw", "vh", "vi", "vb", "vmin", "vmax",
        "svw", "lvh", "dvi", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax",
    ] {
        assert_classification(&format!("(width: calc(1{unit} + 1px))"), true);
    }
}

#[test]
fn valid_custom_property_operands_remain_recognized_pending_values() {
    for query in [
        "(width > var(--limit))",
        "(width > var(--limit,))",
        "(width > var(--limit, red))",
        "(height: calc(var(--count) * 1px))",
        "(inline-size: var(--limit, var(--fallback, 2em)))",
        "(var(--lower) < block-size <= var(--upper))",
        "(aspect-ratio: var(--pair))",
        "(aspect-ratio: var(--numerator) / 2)",
        "(orientation: var(--Orientation, portrait))",
    ] {
        assert_classification(query, true);
    }
}

#[test]
fn scoped_tree_counting_functions_admit_numeric_container_contexts() {
    for query in [
        "(width: calc(sibling-count() * 1px))",
        "(height > calc(sibling-index() * 2em))",
        "(aspect-ratio: sibling-count())",
        "(aspect-ratio: sibling-index() / 2)",
        "(calc(sibling-count() * 1px) < inline-size)",
    ] {
        assert_classification(query, true);
    }
}

#[test]
fn comments_between_comparator_delimiters_preserve_inclusive_recognition() {
    for query in ["(width </**/= 1px)", "(height >/**/= 2px)"] {
        assert_classification(query, true);
    }
}

#[test]
fn whitespace_between_comparator_delimiters_prevents_feature_recognition() {
    for query in [
        "(width < = 1px)",
        "(height >\t= 2px)",
        "(width </**/ = 1px)",
    ] {
        assert_classification(query, false);
    }
}

#[test]
fn invalid_feature_domains_and_range_shapes_use_opaque_fallback() {
    for query in [
        "(min-width)",
        "(min-width > 1px)",
        "(1px < min-width)",
        "(1px < width > 2px)",
        "(1px = width = 2px)",
        "(width: 2)",
        "(width: 10%)",
        "(width: calc(0))",
        "(width: calc(1px + 1s))",
        "(aspect-ratio: -1 / 2)",
        "(aspect-ratio: 1 / -2)",
        "(aspect-ratio: 1px / 2)",
        "(orientation = landscape)",
        "(min-orientation: portrait)",
        "(orientation: square)",
        "(width: sibling-index())",
        "(width: calc(sibling-count(2) * 1px))",
        "(aspect-ratio: sibling-index(1))",
    ] {
        assert_classification(query, false);
    }
}

#[test]
fn malformed_variable_references_use_opaque_fallback() {
    for query in [
        "(width: var())",
        "(width: var(bad-name))",
        "(width: var(--))",
        "(width: var(--limit 1px))",
        "(width: var(--limit, var(bad-name)))",
        "(width: var(--limit, !))",
        "(width: var(--limit, ;))",
    ] {
        assert_classification(query, false);
    }
}

#[test]
fn unknown_feature_names_and_future_functions_remain_clean_opaque_syntax() {
    for query in ["(future-size: 1px)", "future(1px)", "()"] {
        assert_classification(query, false);
    }
}

#[test]
fn programmatic_signed_length_keeps_programmatic_origins_and_exact_spelling() {
    let values = CssComponentValues::try_new(
        ["width", ":", "-12345678901234567890.123456789px"]
            .into_iter()
            .map(|token| CssComponentValue::try_token(token).unwrap())
            .collect(),
    )
    .unwrap();
    let condition = CssContainerCondition::try_from_enclosed(
        CssGeneralEnclosed::try_parenthesized(values).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        condition.kind(),
        CssContainerConditionKind::Feature(_)
    ));
    assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(condition.position(), None);
    assert_eq!(
        condition.serialize().unwrap().as_css(),
        "(width:-12345678901234567890.123456789px)"
    );
}

#![forbid(unsafe_code)]
//! Known Conditional 5 support predicates have an inspectable feature
//! classification; balanced unknown functions and invalid leaf arguments keep
//! the general-enclosed fallback. This observes authored syntax, not host truth.
//!
//! Conditional 5 §2: https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#at-supports-ext
//! Fonts 4 vocabularies: https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-format-values
//! and https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-tech-values

use surgeist_css::{
    CssNamespaceContext, CssRule, CssSupportsCondition, CssSupportsConditionKind,
    parse_component_values, parse_sheet,
};

fn assert_opaque_classification(condition: &str, expected_opaque: bool) {
    let checked = CssSupportsCondition::try_from_components(
        parse_component_values(condition).expect("valid component syntax"),
        &CssNamespaceContext::default(),
    )
    .expect("valid authored supports condition");

    let source = format!("@supports {condition} {{}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Supports(parsed)] = report.syntax().rules() else {
        panic!("{source}: expected one retained supports group");
    };

    // Both real fronts run before checking the shared classification contract.
    let checked_opaque = matches!(checked.kind(), CssSupportsConditionKind::GeneralEnclosed(_));
    let parsed_opaque = matches!(
        parsed.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(_)
    );
    assert_eq!(
        (checked_opaque, parsed_opaque),
        (expected_opaque, expected_opaque),
        "{condition}: checked={:?}; parsed={:?}",
        checked.kind(),
        parsed.condition().kind(),
    );
}

#[test]
fn known_font_technology_has_a_feature_classification_on_both_fronts() {
    assert_opaque_classification("font-tech(variations)", false);
}

#[test]
fn known_font_format_has_a_feature_classification_on_both_fronts() {
    assert_opaque_classification("font-format(woff2)", false);
}

#[test]
fn at_keyword_predicate_has_a_feature_classification_on_both_fronts() {
    assert_opaque_classification("at-rule(@media)", false);
}

#[test]
fn unknown_functions_and_wrong_leaf_arity_remain_general_enclosed() {
    for condition in [
        "future(variations)",
        "font-tech(variations, palettes)",
        "font-format(woff2, opentype)",
        "at-rule(@media @supports)",
    ] {
        assert_opaque_classification(condition, true);
    }
}

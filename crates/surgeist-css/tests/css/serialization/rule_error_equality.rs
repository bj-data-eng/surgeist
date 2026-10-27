//! A literal rule serialization failure compares by what it reports.
use surgeist_css::CssParser;
use surgeist_css::rule::CssStyleAncestor;
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::serialization::{
    CssRuleGraphInput, CssRuleSerializationError, CssRuleSerializationLimits,
};

fn refused_at_depth(source: &str, max_depth: usize) -> CssRuleSerializationError {
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{source}");
    CssRuleGraphInput::from_rule(&report.syntax().rules()[0])
        .serialize_cssom(
            &CssNamespaceContext::default(),
            CssStyleAncestor::Absent,
            CssRuleSerializationLimits {
                max_depth,
                ..Default::default()
            },
        )
        .expect_err("the depth bound refuses the rule graph")
}

#[test]
fn the_same_refusal_of_the_same_rule_compares_equal() {
    let first = refused_at_depth("@media all { a {} }", 0);
    let second = refused_at_depth("@media all { a {} }", 0);
    assert!(first.is_resource_limit());
    assert_eq!(first, second);
    assert_eq!(first, first.clone());
}

#[test]
fn refusals_at_different_rule_paths_compare_unequal() {
    let shallow = refused_at_depth("@media all { a {} }", 0);
    let nested = refused_at_depth("@media all { @media all { a {} } }", 1);
    assert_ne!(shallow.rule_path(), nested.rule_path());
    assert_ne!(shallow, nested);
}

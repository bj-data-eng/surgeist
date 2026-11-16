//! A rule-graph serialization failure names the output form that failed: compact
//! specified CSS or literal CSSOM text.
use surgeist_css::CssParser;
use surgeist_css::rule::{CssRule, CssStyleAncestor};
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::serialization::{
    CssRuleGraphInput, CssRuleSerializationError, CssRuleSerializationLimits,
    CssRuleSerializationOutput, CssSerializationLimits,
};

const SPECIFIED_RESOURCE: &str = "specified rule serialization exceeded a resource limit";
const CSSOM_RESOURCE: &str = "literal CSSOM rule serialization exceeded a resource limit";

fn rule(source: &str) -> CssRule {
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{source}");
    report.syntax().rules()[0].clone()
}

fn specified(rule: &CssRule, limits: CssRuleSerializationLimits) -> CssRuleSerializationError {
    CssRuleGraphInput::from_rule(rule)
        .serialize_specified(limits)
        .expect_err("the limits refuse the specified output")
}

fn cssom(rule: &CssRule, limits: CssRuleSerializationLimits) -> CssRuleSerializationError {
    CssRuleGraphInput::from_rule(rule)
        .serialize_cssom(
            &CssNamespaceContext::default(),
            CssStyleAncestor::Absent,
            limits,
        )
        .expect_err("the limits refuse the CSSOM output")
}

fn no_bytes() -> CssRuleSerializationLimits {
    CssRuleSerializationLimits {
        values: CssSerializationLimits::new(usize::MAX, usize::MAX, 0),
        ..Default::default()
    }
}

fn no_depth() -> CssRuleSerializationLimits {
    CssRuleSerializationLimits {
        max_depth: 0,
        ..Default::default()
    }
}

#[test]
fn a_specified_byte_limit_names_the_specified_output() {
    let error = specified(&rule(".a { color: red }"), no_bytes());
    assert!(error.is_resource_limit(), "{error:?}");
    assert_eq!(error.output(), CssRuleSerializationOutput::Specified);
    assert_eq!(error.to_string(), SPECIFIED_RESOURCE);
}

#[test]
fn a_specified_depth_limit_names_the_specified_output() {
    let error = specified(&rule("@media all { .a {} }"), no_depth());
    assert!(error.is_resource_limit(), "{error:?}");
    assert_eq!(error.output(), CssRuleSerializationOutput::Specified);
    assert_eq!(error.to_string(), SPECIFIED_RESOURCE);
}

#[test]
fn a_cssom_byte_limit_names_the_cssom_output() {
    let error = cssom(&rule(".a { color: red }"), no_bytes());
    assert!(error.is_resource_limit(), "{error:?}");
    assert_eq!(error.output(), CssRuleSerializationOutput::Cssom);
    assert_eq!(error.to_string(), CSSOM_RESOURCE);
}

#[test]
fn a_cssom_depth_limit_names_the_cssom_output() {
    let error = cssom(&rule("@media all { .a {} }"), no_depth());
    assert!(error.is_resource_limit(), "{error:?}");
    assert_eq!(error.output(), CssRuleSerializationOutput::Cssom);
    assert_eq!(error.to_string(), CSSOM_RESOURCE);
}

#[test]
fn the_same_limit_in_the_two_forms_is_two_different_failures() {
    let rule = rule(".a { color: red }");
    assert_ne!(specified(&rule, no_bytes()), cssom(&rule, no_bytes()));
}

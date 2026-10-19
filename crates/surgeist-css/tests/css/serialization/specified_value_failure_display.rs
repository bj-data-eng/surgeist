#![forbid(unsafe_code)]
use surgeist_css::{
    CssSelector, CssSpecifiedValueSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationLimits as Limits,
};

const VALUE_FAILURE: &str = "specified value cannot be represented by the selected serializer";

#[test]
fn empty_selector_identity_reports_a_general_value_failure() {
    let selector = CssSelector::Class(String::new());
    let before = selector.clone();
    let error = selector.to_specified_css().unwrap_err();
    assert_eq!(error.kind(), Kind::UnrepresentableValue);
    assert_eq!(error.to_string(), VALUE_FAILURE);
    assert_eq!(selector, before);
}

#[test]
fn nul_selector_identity_reports_a_general_value_failure() {
    let selector = CssSelector::Key("a\0b".into());
    let before = selector.clone();
    let error = selector.to_specified_css().unwrap_err();
    assert_eq!(error.kind(), Kind::UnrepresentableValue);
    assert_eq!(error.to_string(), VALUE_FAILURE);
    assert_eq!(selector, before);
}

#[test]
fn byte_exhaustion_retains_its_distinct_resource_message() {
    let error = CssSelector::Class("x".into())
        .to_specified_css_with_limits(Limits::new(1, 1, 0))
        .unwrap_err();
    assert_eq!(error.kind(), Kind::ByteLimit);
    assert_eq!(error.to_string(), "specified-value CSS byte limit exceeded");
}

#[test]
fn valid_escaped_identity_retains_its_output() {
    assert_eq!(
        CssSelector::Class("a b".into()).to_specified_css().unwrap(),
        ".a\\ b"
    );
}

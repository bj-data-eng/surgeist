use surgeist_css::{CssSelector, CssSerializationLimits as Limits};

const VALUE_FAILURE: &str = "specified value cannot be represented by the selected serializer";

#[test]
fn empty_selector_identity_reports_a_general_value_failure() {
    let selector = CssSelector::Class(String::new());
    let before = selector.clone();
    let error = selector
        .serialize_specified(Default::default())
        .unwrap_err();
    assert!(error.is_unrepresentable_value(), "{error:?}");
    assert_eq!(error.to_string(), VALUE_FAILURE);
    assert_eq!(selector, before);
}

#[test]
fn nul_selector_identity_reports_a_general_value_failure() {
    let selector = CssSelector::Key("a\0b".into());
    let before = selector.clone();
    let error = selector
        .serialize_specified(Default::default())
        .unwrap_err();
    assert!(error.is_unrepresentable_value(), "{error:?}");
    assert_eq!(error.to_string(), VALUE_FAILURE);
    assert_eq!(selector, before);
}

#[test]
fn byte_exhaustion_retains_its_distinct_resource_message() {
    let error = CssSelector::Class("x".into())
        .serialize_specified(Limits::new(1, 1, 0))
        .unwrap_err();
    assert!(error.is_byte_limit(), "{error:?}");
    assert_eq!(error.to_string(), "specified-value CSS byte limit exceeded");
}

#[test]
fn valid_escaped_identity_retains_its_output() {
    assert_eq!(
        CssSelector::Class("a b".into())
            .serialize_specified(Default::default())
            .unwrap(),
        ".a\\ b"
    );
}

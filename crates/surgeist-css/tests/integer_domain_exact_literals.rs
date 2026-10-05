#![forbid(unsafe_code)]

//! Grid repeat counts have range [1, infinity], without a machine-sized maximum:
//! https://www.w3.org/TR/2025/CRD-css-grid-1-20250326/#funcdef-repeat
//! Counter additive weights are nonnegative integers in strictly descending order:
//! https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/#descdef-counter-style-additive-symbols
//! CSS retains exact authored bounds before downstream implementation-range policy;
//! the counter-style standard separately permits supported-range clamping.

use surgeist_css::{CssKnownPropertyValueRef, CssRule, parse_sheet, parse_style_attribute};

fn serialized_columns(value: &str) -> String {
    let source = format!("grid-template-columns: {value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::GridTemplateColumns(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected checked grid-template-columns");
    };
    wrapper.value().serialize_specified().unwrap()
}

#[test]
fn integer_track_repeat_preserves_count_above_machine_integer_maximum() {
    assert_eq!(
        serialized_columns("repeat(2147483648, 1fr)"),
        "repeat(2147483648, 1fr)"
    );
}

#[test]
fn integer_fixed_repeat_preserves_count_above_machine_integer_maximum() {
    assert_eq!(
        serialized_columns("repeat(2147483648, 1px) repeat(auto-fill, 2px)"),
        "repeat(2147483648, 1px) repeat(auto-fill, 2px)"
    );
}

#[test]
fn ordinary_repeat_counts_and_literal_only_grammar_remain_validated() {
    assert_eq!(serialized_columns("repeat(2, 1fr)"), "repeat(2, 1fr)");
    for count in ["0", "-0", "-1", "2.0", "2e0", "calc(2)"] {
        let source = format!("grid-template-columns: repeat({count}, 1px); color: red");
        let report = parse_style_attribute(&source);
        assert!(!report.is_clean(), "{source}");
        assert_eq!(report.syntax().len(), 1, "{source}");
    }
}

#[test]
fn huge_strictly_descending_additive_weights_are_distinct() {
    let source = concat!(
        "@counter-style exact { system: additive; ",
        "additive-symbols: 2147483649 \"a\", 2147483648 \"b\"; }"
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("valid additive definition must survive");
    };
    assert!(rule.descriptors().additive_symbols().is_some());
}

#[test]
fn equal_additive_weights_with_different_spelling_remain_invalid() {
    let source = concat!(
        "@counter-style exact { system: additive; ",
        "additive-symbols: +0007 \"a\", 7 \"b\"; } .after { color: red; }"
    );
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("invalid additive descriptor is dropped while the valid rule and sibling survive");
    };
    assert!(rule.descriptors().additive_symbols().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("one wholly invalid additive descriptor");
    };
    assert_eq!(
        diagnostic.error().code(),
        surgeist_css::CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(
        diagnostic.action(),
        surgeist_css::CssRecoveryAction::DropDescriptor
    );
}

#[test]
fn reversed_huge_range_is_rejected_before_implementation_range_policy() {
    let source = concat!(
        "@counter-style exact { system: cyclic; symbols: \"a\"; ",
        "range: 2147483649 2147483648; } .after { color: red; }"
    );
    let report = parse_sheet(source);
    assert!(
        !report.is_clean(),
        "exact reversed bounds must be diagnosed"
    );
    let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("recoverable range error preserves the rule and sibling");
    };
    assert!(rule.descriptors().range().is_none());
}

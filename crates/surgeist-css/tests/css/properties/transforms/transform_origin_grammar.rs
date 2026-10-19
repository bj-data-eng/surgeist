#![forbid(unsafe_code)]
//! The selected WebKit transform-origin parser rejects a vertical keyword
//! followed by a length. Values 4 §<position> gives a conflicting example;
//! this test records the selected browser behavior for the authored grammar.
use surgeist_css::{CssKnownProperty, CssRecoveryAction, parse_style_attribute};

fn accepted(value: &str) {
    let source = format!("transform-origin: {value}; color: red");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2, "{source}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::TransformOrigin
    );
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color
    );
}

fn rejected(value: &str) {
    let source = format!("transform-origin: {value}; color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one transform-origin diagnostic: {source}: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
}

#[test]
fn vertical_keyword_followed_by_length_is_not_one_planar_value_plus_z() {
    rejected("top 50px");
    rejected("bottom 50px");
}

#[test]
fn vertical_keyword_followed_by_length_calculation_is_not_z() {
    rejected("top calc(1px * 2)");
    rejected("bottom calc(1px * 2)");
}

#[test]
fn vertical_keyword_followed_by_zero_is_not_z() {
    rejected("top 0");
    rejected("bottom 0");
}

#[test]
fn one_and_two_planar_values_and_two_planar_values_plus_z_remain_valid() {
    accepted("top");
    accepted("bottom");
    accepted("left 50px");
    accepted("center 50px");
    accepted("left calc(1px * 2)");
    accepted("left top 50px");
    accepted("top left calc(1px * 2)");
}

#[test]
fn percentage_is_not_a_valid_third_z_value() {
    rejected("left top 50%");
}

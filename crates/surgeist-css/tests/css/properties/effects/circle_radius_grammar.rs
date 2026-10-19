#![forbid(unsafe_code)]
//! CSS Shapes `circle()` has one nonnegative length-percentage radius.
//! The selected WebKit Shapes consumer admits the same domain before `at`.
use surgeist_css::{CssKnownProperty, CssRecoveryAction, parse_style_attribute};

fn accepted(radius: &str) {
    let source = format!("clip-path: {radius}; color: red");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2, "{source}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::ClipPath,
        "{source}"
    );
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color,
        "{source}"
    );
}

fn rejected(radius: &str) {
    let source = format!("clip-path: {radius}; color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color,
        "{source}"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one radius diagnostic: {source}: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
}

#[test]
fn one_literal_percentage_radius_is_accepted_without_position() {
    accepted("circle(25%)");
    accepted("circle(50%)");
}

#[test]
fn percentage_radius_is_accepted_before_position() {
    accepted("circle(75% at left top)");
}

#[test]
fn mixed_length_percentage_calculation_is_accepted_as_one_radius() {
    accepted("circle(calc(10px + 20%))");
}

#[test]
fn length_omission_and_existing_percentage_position_remain_accepted() {
    accepted("circle(10px)");
    accepted("circle()");
    accepted("circle(50% at center)");
}

#[test]
fn negative_literal_and_two_radii_are_rejected_with_sibling_recovery() {
    rejected("circle(-1px)");
    rejected("circle(-1%)");
    rejected("circle(10px 20px)");
    rejected("circle(25% 50%)");
}

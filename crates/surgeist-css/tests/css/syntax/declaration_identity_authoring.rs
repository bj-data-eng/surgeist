//! Parsed style rules and declarations keep their authored property identity.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

#[test]
fn parsed_style_rule_is_explicit_rule_variant() {
    let sheet = parse_sheet(".panel { width: 10px; }").expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("style sheet should parse exactly one rule");
    };
    let style_rule = style_rule(rule);

    assert_eq!(
        style_rule.selectors().selectors()[0].selector(),
        &CssSelector::Class("panel".to_owned())
    );
    assert_eq!(style_rule.declarations().len(), 1);
}

#[test]
fn background_color_preserves_authored_property_identity() {
    let declaration = single_declaration(".panel { background-color: black; }");
    assert_eq!(
        declaration.known().map(|known| known.property()),
        Some(CssKnownProperty::BackgroundColor)
    );
    let CssKnownPropertyValueRef::BackgroundColor(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("background color");
    };
    assert_eq!(value.value().named().unwrap().name(), "black");
}

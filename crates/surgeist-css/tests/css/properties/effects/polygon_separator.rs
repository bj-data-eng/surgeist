//! CSS Shapes 1 (2025-06-12) §3.1 and Values 4 §2.1 require the comma
//! to be omitted when every preceding optional polygon modifier is absent.
use surgeist_css::CssParser;
use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef};

#[test]
fn polygon_without_modifiers_starts_with_its_first_point() {
    let report = CssParser::default()
        .parse_block_contents("clip-path: polygon(1px 0px, 1px 1px); opacity: 0.5");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
}

#[test]
fn polygon_without_modifiers_rejects_a_leading_comma_and_retains_its_sibling() {
    let report = CssParser::default()
        .parse_block_contents("clip-path: polygon(, 1px 0px, 1px 1px); opacity: 0.5");
    assert!(!report.is_clean());
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].property_name(),
        CssPropertyNameRef::Known(CssKnownProperty::Opacity)
    );
}

#[test]
fn polygon_fill_rule_requires_its_separator() {
    let valid =
        CssParser::default().parse_block_contents("clip-path: polygon(evenodd, 1px 0px, 1px 1px)");
    assert!(valid.is_clean(), "{:?}", valid.diagnostics());
    assert_eq!(valid.syntax().len(), 1);
    let invalid =
        CssParser::default().parse_block_contents("clip-path: polygon(evenodd 1px 0px, 1px 1px)");
    assert!(!invalid.is_clean());
    assert!(invalid.syntax().is_empty());
}

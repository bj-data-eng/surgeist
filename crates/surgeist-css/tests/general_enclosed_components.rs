#![forbid(unsafe_code)]

use surgeist_css::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssRule, CssSerializedOrigin,
    CssSupportsConditionKind, CssValueOrigin, parse_component_values, parse_sheet,
};

// Authored syntax preserves the complete opaque enclosure, including escapes,
// comments and exact numeric spelling. Coordinates address the original sheet.
#[test]
fn supports_opaque_enclosures_preserve_spelling_and_original_positions() {
    for enclosure in [r"f\75 ture(/*🦊*/ +001.2300e+02 [x])", "(/*🦊*/ 2px [x])"] {
        let prefix = "/*🦊*/\n@supports ";
        let source = format!("{prefix}{enclosure} {{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let [CssRule::Supports(rule)] = report.syntax().rules() else {
            panic!("expected one supports rule");
        };
        let CssSupportsConditionKind::GeneralEnclosed(value) = rule.condition().kind() else {
            panic!("expected an opaque enclosure");
        };
        assert_eq!(value.authored(), enclosure);
        assert_eq!(value.position().byte_offset().value(), prefix.len());
        assert_eq!(value.position().line().value(), 1);
        assert_eq!(value.position().column().value(), 10);
    }
}

// CSS Syntax closes a function at EOF. The emitted delimiter must retain that
// recovery origin rather than pretending that a closing token was authored.
#[test]
fn eof_function_serialization_retains_the_implicit_closing_origin() {
    let source = "future(/*🦊*/ +001.2300e+02";
    let values = parse_component_values(source).unwrap();
    let [component] = values.items() else {
        panic!("expected one component");
    };
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("expected a function");
    };
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("expected an EOF-implied close");
    };
    assert_eq!(opening.span().start().byte_offset().value(), 0);
    assert_eq!(at.span().start().byte_offset().value(), source.len());
    let serialized = values.serialize().unwrap();
    assert_eq!(serialized.as_css(), format!("{source})"));
    assert_eq!(
        serialized.origin_at(source.len()),
        Some(&CssSerializedOrigin::Token(
            function.closing_origin().clone()
        ))
    );
}

// A supplied wrapper around parsed children owns only its delimiters. Child
// provenance remains attached to the actual parsed input.
#[test]
fn programmatic_function_preserves_parsed_child_and_delimiter_origins() {
    let children = parse_component_values("+001.2300e+02").unwrap();
    let child_origin = children.items()[0].origin().clone();
    let component = CssComponentValue::try_function("future", children).unwrap();
    assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("expected a function");
    };
    assert_eq!(function.values().items()[0].origin(), &child_origin);
    assert_eq!(function.closing_origin(), &CssValueOrigin::Programmatic);
    let values = CssComponentValues::try_new(vec![component]).unwrap();
    let serialized = values.serialize().unwrap();
    assert_eq!(serialized.as_css(), "future(+001.2300e+02)");
    assert_eq!(
        serialized.origin_at(7),
        Some(&CssSerializedOrigin::Token(child_origin))
    );
}

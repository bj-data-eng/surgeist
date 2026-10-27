//! Custom properties and `var()` references (CSS Variables 1) keep authored
//! syntax, names and fallbacks through the public parser.

use surgeist_css::declaration::{
    CssAuthoredDeclarationValue, CssCustomPropertyName, CssCustomPropertyValue, CssDeclaration,
    CssGlobalKeyword,
};
use surgeist_css::diagnostic::{CssErrorCode, CssTokenKind};
use surgeist_css::property::CssKnownProperty;

use crate::css::support::sheet_cases::*;

fn custom_property_value(declaration: &CssDeclaration) -> &CssCustomPropertyValue {
    declaration
        .custom()
        .expect("custom declaration")
        .value()
        .value()
        .expect("authored custom property value")
}

#[test]
fn custom_property_name_constructor_preserves_case_and_rejects_non_custom_names() {
    let name = CssCustomPropertyName::try_new("--BrandColor").unwrap();
    assert_eq!(name.as_str(), "--BrandColor");
    assert_eq!(
        CssCustomPropertyName::try_new("--brand_color-1")
            .unwrap()
            .as_str(),
        "--brand_color-1",
    );
    assert_eq!(CssCustomPropertyName::try_new("color"), None);
    assert_eq!(CssCustomPropertyName::try_new("-gap"), None);
    assert_eq!(CssCustomPropertyName::try_new("--"), None);
    assert_eq!(CssCustomPropertyName::try_new("-- bad"), None);
    assert_eq!(CssCustomPropertyName::try_new("--;"), None);
    assert_eq!(CssCustomPropertyName::try_new("--gap;"), None);
    assert_eq!(CssCustomPropertyName::try_new("--gap\n"), None);
    assert_eq!(CssCustomPropertyName::try_new("--gap\u{7f}"), None);
}

#[test]
fn authored_declaration_value_constructor_rejects_empty_css() {
    let value = CssAuthoredDeclarationValue::try_new("  8px  ").unwrap();
    assert_eq!(value.as_css(), "  8px  ");
    assert_eq!(CssAuthoredDeclarationValue::try_new(""), None);
    assert_eq!(CssAuthoredDeclarationValue::try_new(" \t\n "), None);
}

#[test]
fn parses_custom_property_declarations_as_authored_syntax() {
    let declaration = single_declaration(".theme { --BrandColor: #fff; }");
    assert_eq!(
        declaration.custom().map(|custom| custom.name()),
        Some(&CssCustomPropertyName::try_new("--BrandColor").unwrap())
    );
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "#fff");
    assert!(!value.is_empty());
}

#[test]
fn custom_value_preserves_nested_variable_fallback_authored_css() {
    let declaration =
        single_declaration(".theme { --gap: var(--space, calc(1px + var(--fallback))); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "var(--space, calc(1px + var(--fallback)))");
}

#[test]
fn custom_value_preserves_variable_fallback_authored_css() {
    let declaration = single_declaration(".theme { --gap: var(--space, 8px); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "var(--space, 8px)");
}

#[test]
fn custom_values_accept_plain_and_empty_variable_fallback_forms() {
    let declaration = single_declaration(".theme { --gap: var(--space); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "var(--space)");

    let declaration = single_declaration(".theme { --gap: var(--empty,); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "var(--empty,)");
}

#[test]
fn supported_properties_accept_variable_dependent_values_symbolically() {
    let declaration = single_declaration(".panel { gap: var(--space, 8px); }");
    assert_eq!(
        declaration.known().map(|known| known.property()),
        Some(CssKnownProperty::Gap)
    );
    assert!(declaration.known().unwrap().property_value().is_none());
    let value = declaration
        .known()
        .unwrap()
        .substitution_dependent()
        .expect("expected variable dependent value");
    assert_eq!(value.as_css(), "var(--space, 8px)");
}

#[test]
fn supported_properties_accept_embedded_variable_dependent_values_symbolically() {
    let declaration = single_declaration(".panel { width: calc(var(--w) + 1px); }");
    assert_eq!(
        declaration.known().map(|known| known.property()),
        Some(CssKnownProperty::Width)
    );
    assert!(declaration.known().unwrap().property_value().is_none());
    let value = declaration
        .known()
        .unwrap()
        .substitution_dependent()
        .expect("expected variable dependent value");
    assert_eq!(value.as_css(), "calc(var(--w) + 1px)");
}

#[test]
fn variable_dependent_values_skip_post_substitution_validation() {
    let declaration = single_declaration(".panel { color: var(--brand, 8px); }");
    assert_eq!(
        declaration.known().map(|known| known.property()),
        Some(CssKnownProperty::Color)
    );
    assert!(declaration.known().unwrap().property_value().is_none());
    let value = declaration
        .known()
        .unwrap()
        .substitution_dependent()
        .expect("expected variable dependent value");
    assert_eq!(value.as_css(), "var(--brand, 8px)");
}

#[test]
fn malformed_var_in_supported_property_drops_the_declaration() {
    // Variables 1 (CR 2022-06-16) §3: var( <custom-property-name> ,
    // <declaration-value>? ). `color` is not a custom property name, and a
    // fallback needs its comma, so neither reference is syntactically valid
    // and the declaration is invalid at parse time.
    for (input, property) in [
        (".panel { gap: var(color); }", CssKnownProperty::Gap),
        (
            ".panel { color: var(--brand red); }",
            CssKnownProperty::Color,
        ),
    ] {
        // The declaration's own value grammar rejects it, for its property.
        let error = assert_drops_sole_declaration(input);
        let detail = &error;
        assert_eq!(
            detail.code(),
            CssErrorCode::InvalidPropertyValue,
            "{input}: property value rejection: {error:?}"
        );
        assert_eq!(detail.known_property().unwrap(), property, "{input}");
    }
}

#[test]
fn invalid_supported_value_without_var_drops_the_declaration() {
    // Box Alignment 3 (WD 2026-01-30) §8.2 and §8.1: gap is <'row-gap'> <'column-gap'>?,
    // each normal | <length-percentage [0,∞]>; `auto` is neither.
    let error = assert_drops_sole_declaration(".panel { gap: auto; }");
    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "gap value rejection: {error:?}"
    );
    assert_eq!(detail.known_property().unwrap(), CssKnownProperty::Gap);
    let token = detail.encountered().expect("responsible keyword");
    assert_eq!(
        (token.kind(), token.authored()),
        (CssTokenKind::Ident, "auto")
    );
}

#[test]
fn custom_property_with_var_remains_custom_property_value() {
    let declaration = single_declaration(".theme { --gap: var(--space, 8px); }");
    assert!(matches!(
        custom_property_value(&declaration),
        value
            if value.as_css() == "var(--space, 8px)"
    ));
}

#[test]
fn unknown_property_with_var_rejects() {
    assert_rejection(
        ".panel { made-up-property: var(--space); }",
        Rejection::unknown_property("made-up-property"),
    );
}

#[test]
fn custom_property_values_validate_nested_variable_syntax() {
    let declaration = single_declaration(".theme { --gap: calc(1px + var(--space)); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "calc(1px + var(--space))");
}

#[test]
fn rejects_malformed_variable_references() {
    assert_rejection(
        ".theme { --gap: var(); }",
        Rejection::new(CssErrorCode::UnexpectedEnd, None, None),
    );
    assert_rejection(
        ".theme { --gap: var(color); }",
        Rejection::new(CssErrorCode::InvalidQualifiedRule, None, Some("color")),
    );
    assert_rejection(
        ".theme { --gap: var(--gap --other); }",
        Rejection::new(CssErrorCode::UnexpectedToken, None, Some("--other")),
    );
    assert_rejection(
        ".theme { --gap: var(--gap, }",
        Rejection::new(CssErrorCode::UnexpectedToken, None, Some("}")),
    );
}

#[test]
fn custom_property_global_keyword_must_be_whole_value() {
    assert_eq!(
        single_declaration(".theme { --gap: inherit; }")
            .custom()
            .unwrap()
            .value()
            .global(),
        Some(CssGlobalKeyword::Inherit)
    );
    let declaration = single_declaration(".theme { --gap: inherit 1px; }");
    let value = declaration.custom().unwrap().value();
    assert_eq!(value.global(), None);
    assert_eq!(value.value().unwrap().as_css(), "inherit 1px");
}

#[test]
fn custom_property_names_are_case_sensitive_when_parsed() {
    let declaration = single_declaration(".theme { --BrandColor: 1px; }");
    assert_eq!(
        declaration.custom().map(|custom| custom.name()),
        Some(&CssCustomPropertyName::try_new("--BrandColor").unwrap())
    );
    assert_ne!(
        declaration.custom().map(|custom| custom.name()),
        Some(&CssCustomPropertyName::try_new("--brandcolor").unwrap())
    );
}

#[test]
fn parser_accepts_escaped_custom_property_names_from_cssparser_ident_tokens() {
    assert_eq!(CssCustomPropertyName::try_new("--bad name"), None);

    let declaration = single_declaration(".theme { --bad\\ name: 1px; }");
    let Some(name) = declaration.custom().map(|custom| custom.name()) else {
        panic!("expected custom property");
    };
    assert_eq!(name.as_str(), "--bad name");
}

#[test]
fn rejects_malformed_custom_property_names() {
    assert_rejection(".theme { --: 1px; }", Rejection::unknown_property("--"));
    assert_rejection(
        ".theme { --bad name: 1px; }",
        Rejection::new(CssErrorCode::UnexpectedToken, None, Some("name")),
    );
}

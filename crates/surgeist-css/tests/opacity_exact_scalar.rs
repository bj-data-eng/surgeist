#![forbid(unsafe_code)]
//! Finite authored opacity scalars retain lexical tokens and provenance.
//! CSS Color 4 section 3.3 admits numbers and percentages; the exact scalar
//! contract preserves finite decimals beyond tokenizer float range.
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#transparency

use surgeist_css::*;

fn parsed(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("opacity:{text}!important"));
    assert!(
        report.is_clean(),
        "finite opacity {text}: {:?}",
        report.diagnostics()
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained ordinary opacity declaration")
    };
    declaration.clone()
}

fn wrapper(declaration: &CssDeclaration) -> &CssOpacityPropertyValue {
    let CssKnownPropertyValueRef::Opacity(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary opacity wrapper")
    };
    value
}

fn assert_token(declaration: &CssDeclaration, text: &str, percentage: bool) {
    assert_eq!(wrapper(declaration).as_css(), text);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        text
    );
    let [component] = declaration.value_components().items() else {
        panic!("one authored numeric token")
    };
    let representation = match (percentage, component.view()) {
        (false, CssComponentValueRef::Token(CssValueTokenRef::Number(number))) => {
            number.representation()
        }
        (true, CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))) => {
            number.representation()
        }
        _ => panic!("authored token kind must remain unchanged"),
    };
    assert_eq!(representation, text.strip_suffix('%').unwrap_or(text));
}

fn assert_parsed(text: &str, percentage: bool) {
    let declaration = parsed(text);
    assert_token(&declaration, text, percentage);
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin() else {
        panic!("parsed token origin")
    };
    assert!(
        origin
            .source()
            .same_snapshot(declaration.parsed_value().unwrap().source())
    );
}

fn assert_constructed(text: &str, percentage: bool) {
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()]).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Opacity),
        components.clone(),
        CssImportance::Normal,
    )
    .expect("one finite lexical opacity value must admit through strict property construction");
    assert_token(&declaration, text, percentage);
    assert_eq!(declaration.value_components(), &components);
    assert_eq!(declaration.importance(), CssImportance::Normal);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert!(matches!(
        components.items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
}

#[test]
fn parsed_large_finite_numbers_are_ordinary_opacity() {
    for text in ["1e100", "-1e100"] {
        assert_parsed(text, false);
    }
}

#[test]
fn parsed_large_finite_percentages_are_ordinary_opacity() {
    for text in ["1e100%", "-1e100%"] {
        assert_parsed(text, true);
    }
}

#[test]
fn constructed_large_finite_numbers_are_ordinary_opacity() {
    for text in ["1e100", "-1e100"] {
        assert_constructed(text, false);
    }
}

#[test]
fn constructed_large_finite_percentages_are_ordinary_opacity() {
    for text in ["1e100%", "-1e100%"] {
        assert_constructed(text, true);
    }
}

#[test]
fn parsed_tiny_scalars_keep_exact_tokens_and_parsed_origins() {
    for (text, percentage) in [
        ("1e-47", false),
        ("-1e-47", false),
        ("1e-47%", true),
        ("-1e-47%", true),
    ] {
        assert_parsed(text, percentage);
    }
}

#[test]
fn constructed_tiny_scalars_keep_exact_tokens_and_programmatic_origins() {
    for (text, percentage) in [
        ("1e-47", false),
        ("-1e-47", false),
        ("1e-47%", true),
        ("-1e-47%", true),
    ] {
        assert_constructed(text, percentage);
    }
}

#[test]
fn opacity_scalar_admission_keeps_grammar_and_symbolic_boundaries() {
    for text in ["1px", "1 2", "infinity", "NaN"] {
        let components = parse_component_values(text).unwrap();
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Opacity),
                components,
                CssImportance::Normal,
            )
            .is_err(),
            "invalid opacity scalar: {text}"
        );
        let report = parse_style_attribute(&format!("opacity:{text}"));
        assert!(!report.is_clean(), "invalid opacity scalar: {text}");
        assert!(report.syntax().is_empty());
    }
    let report = parse_style_attribute("opacity:inherit;opacity:var(--x)");
    assert!(report.is_clean());
    assert_eq!(report.syntax().len(), 2);
    assert!(matches!(
        report.syntax()[0].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));
    assert!(matches!(
        report.syntax()[1].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
}

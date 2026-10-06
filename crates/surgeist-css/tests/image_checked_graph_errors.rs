#![forbid(unsafe_code)]
//! Checked Image graph failures retain their resource category and authored
//! opener through image-only consumers, speculative Content alternatives and
//! reusable pending substitution. The image wrapper contributes zero depth.

use surgeist_css::*;

fn image_source(light_dark_functions: usize) -> String {
    // An unquoted URL is one raw token but one retained image function.
    let mut value = "url(image.svg)".to_owned();
    for _ in 0..light_dark_functions {
        value = format!("light-dark({value}, none)");
    }
    format!("/*😀*/ {value}")
}

fn checked(
    property: CssKnownProperty,
    values: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    if grammar {
        parse_property_value_for_grammar(property.grammar(), values, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            values,
            CssImportance::Important,
        )
    }
}

#[test]
fn image_only_consumers_preserve_resource_origin_and_pending_retry() {
    // 255 LightDark functions plus a URL fit exactly; 256 plus URL exceed 256.
    let valid = parse_component_values(&image_source(255)).unwrap();
    let invalid = parse_component_values(&image_source(256)).unwrap();
    assert_eq!(valid.nesting_depth(), 255);
    assert_eq!(invalid.nesting_depth(), 256);
    let valid_before = valid.clone();
    let invalid_before = invalid.clone();
    let opener = invalid
        .items()
        .iter()
        .find(|item| matches!(item.view(), CssComponentValueRef::Function(_)))
        .unwrap()
        .origin()
        .clone();
    let CssValueOrigin::Parsed(parsed) = &opener else {
        panic!("authored image opener")
    };
    assert_eq!(parsed.span().start().byte_offset().value(), "/*😀*/ ".len());

    for name in ["content", "shape-outside"] {
        let property = CssKnownProperty::from_name(name).unwrap();
        let pending = checked(
            property,
            parse_component_values("var(--image)").unwrap(),
            false,
        )
        .unwrap();
        let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
            panic!("pending image consumer")
        };
        for _ in 0..2 {
            for grammar in [false, true] {
                assert!(checked(property, valid.clone(), grammar).is_ok());
                let error = checked(property, invalid.clone(), grammar).unwrap_err();
                assert_eq!(
                    error.kind(),
                    &CssPropertyValueErrorKind::Component(CssComponentValueErrorKind::NestingLimit),
                    "{name} image resource failure survives grammar alternatives"
                );
                assert_eq!(error.origin(), &CssSerializedOrigin::Token(opener.clone()));
            }
            let error = handle.reenter(invalid.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("typed image replacement failure")
            };
            assert_eq!(
                error.kind(),
                &CssPropertyValueErrorKind::Component(CssComponentValueErrorKind::NestingLimit)
            );
            assert_eq!(error.origin(), &CssSerializedOrigin::Token(opener.clone()));
            assert!(handle.reenter(valid.clone()).is_ok());
            assert!(handle.source().same_occurrence(&pending));
            assert_eq!(handle.source().importance(), CssImportance::Important);
            assert_eq!(valid, valid_before);
            assert_eq!(invalid, invalid_before);
        }
    }
}

#[test]
fn typed_none_failure_preserves_consumer_keyword_and_symbol_ownership() {
    let error = CssImage::try_new(CssImageValue::None).unwrap_err();
    assert_eq!(error, CssImageConstructionError::NotImage);
    assert_eq!(error.to_string(), "none is not an image");
    let semantic: &dyn std::error::Error = &error;
    assert!(semantic.source().is_none());

    // None remains a property keyword, and a counter-style custom identifier.
    for name in ["content", "shape-outside"] {
        let property = CssKnownProperty::from_name(name).unwrap();
        assert!(checked(property, parse_component_values("none").unwrap(), false).is_ok());
    }
    let report = parse_sheet("@counter-style ImageCase { symbols:none; }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    // symbols() accepts images or strings; it has no bare None image branch.
    let property = CssKnownProperty::from_name("list-style-type").unwrap();
    let invalid = parse_component_values("symbols(none)").unwrap();
    let before = invalid.clone();
    for grammar in [false, true] {
        let error = checked(property, invalid.clone(), grammar).unwrap_err();
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
        assert_eq!(invalid, before);
        assert!(
            checked(
                property,
                parse_component_values("symbols(light-dark(none, none))").unwrap(),
                grammar,
            )
            .is_ok()
        );
    }
}

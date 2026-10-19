#![forbid(unsafe_code)]
//! Ordinary authored scalar transport preserves checked tokens at every magnitude.
use surgeist_css::*;

fn integer(declaration: &CssDeclaration) -> &CssIntegerLiteral {
    let CssKnownPropertyValueRef::Order(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary order")
    };
    let CssIntegerValue::Literal(literal) = value.value() else {
        panic!("ordinary integer")
    };
    literal
}

fn opacity(declaration: &CssDeclaration) -> &CssOpacityScalar {
    let CssKnownPropertyValueRef::Opacity(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary opacity")
    };
    let CssOpacityValue::Scalar(scalar) = value.value() else {
        panic!("ordinary scalar")
    };
    scalar
}

#[test]
fn checked_integer_tokens_keep_sign_digits_kind_and_origins_at_every_magnitude() {
    for text in [
        "7",
        "+0007",
        "-0000",
        "2147483647",
        "-2147483648",
        "2147483648",
        "-2147483649",
        "99999999999999999999999999999999999",
    ] {
        for component in [
            CssComponentValue::try_number(text).unwrap(),
            parse_component_values(text).unwrap().items()[0].clone(),
        ] {
            let components = CssComponentValues::try_new(vec![component.clone()]).unwrap();
            let declaration = parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Order),
                components,
                CssImportance::Normal,
            )
            .unwrap();
            let literal = integer(&declaration);
            assert_eq!(literal.component(), &component);
            assert_eq!(literal.origin(), component.origin());
            assert_eq!(literal.numeric().representation(), text);
            assert_eq!(literal.numeric().kind(), CssNumericTokenKind::Integer);
            let retained = literal.clone();
            drop(declaration);
            assert_eq!(retained.component(), &component);
        }
    }
}

#[test]
fn stylesheet_scalar_tokens_keep_actual_spans_and_snapshots_in_semantic_payloads() {
    for (property, text) in [
        ("order", "+0007"),
        ("order", "2147483648"),
        ("opacity", "+.5000"),
        ("opacity", "1e999%"),
        ("opacity", "-0%"),
        ("opacity", "1e-999"),
    ] {
        let input = format!("{property}: {text} !important");
        let report = parse_style_attribute(&input);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let declaration = &report.syntax()[0];
        let component = if property == "order" {
            integer(declaration).component()
        } else {
            opacity(declaration).component()
        };
        assert_eq!(
            component,
            declaration
                .value_components()
                .items()
                .iter()
                .find(|component| matches!(
                    component.view(),
                    CssComponentValueRef::Token(
                        CssValueTokenRef::Number(_) | CssValueTokenRef::Percentage(_)
                    )
                ))
                .unwrap()
        );
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("parsed token")
        };
        let start = property.len() + 2;
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            start + text.len()
        );
        assert!(
            origin
                .source()
                .same_snapshot(declaration.parsed_value().unwrap().source())
        );
        assert_eq!(declaration.importance(), CssImportance::Important);
    }
}

#[test]
fn checked_opacity_tokens_keep_number_or_percentage_identity_without_rounding_or_clamping() {
    for text in [
        ".5", "+.5000", "2", "-1", "0.1", "50%", "-000%", "1e999", "1e-999%",
    ] {
        for component in [
            CssComponentValue::try_token(text).unwrap(),
            parse_component_values(text).unwrap().items()[0].clone(),
        ] {
            let components = CssComponentValues::try_new(vec![component.clone()]).unwrap();
            let declaration = parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Opacity),
                components,
                CssImportance::Normal,
            )
            .unwrap();
            let scalar = opacity(&declaration);
            assert_eq!(scalar.component(), &component);
            assert_eq!(scalar.origin(), component.origin());
            assert_eq!(
                scalar.numeric().representation(),
                text.trim_end_matches('%')
            );
            assert_eq!(
                scalar.kind(),
                if text.ends_with('%') {
                    CssOpacityScalarKind::Percentage
                } else {
                    CssOpacityScalarKind::Number
                }
            );
        }
    }
}

#[test]
fn signed_zero_has_nonnegative_semantics_and_positive_counts_reject_every_zero_spelling() {
    for text in ["0", "+000", "-000"] {
        let literal =
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
                .unwrap();
        assert!(literal.is_zero());
        assert!(!literal.is_negative());
        assert!(CssFontPaletteIndex::try_new(CssIntegerValue::Literal(literal.clone())).is_ok());
        assert!(
            CssGridLine::try_indexed(CssIntegerValue::Literal(literal.clone()), None).is_none()
        );
        assert!(CssPositiveIntegerLiteral::try_new(literal).is_none());
    }
    let negative = CssIntegerLiteral::from_i32(-1);
    assert!(negative.is_negative());
    assert!(!negative.is_zero());
    assert!(CssFontPaletteIndex::try_new(CssIntegerValue::Literal(negative)).is_err());
    for text in ["+0002", "2147483648", "9999999999999999999999999"] {
        let literal =
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
                .unwrap();
        let positive = CssPositiveIntegerLiteral::try_new(literal).unwrap();
        assert_eq!(positive.integer().numeric().representation(), text);
        assert_eq!(positive.integer().origin(), &CssValueOrigin::Programmatic);
    }
}

#[test]
fn environment_indices_and_digit_counts_apply_their_actual_grammar_ranges() {
    for text in ["0", "+0000", "-0000", "2147483648"] {
        let report = parse_style_attribute(&format!("width:env(viewport-segment-width {text} 0)"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    }
    for text in ["-1", "-2147483649"] {
        let report = parse_style_attribute(&format!("width:env(viewport-segment-width {text} 0)"));
        assert!(!report.is_clean(), "negative environment index {text}");
    }
    for text in ["2", "+0003", "4"] {
        assert!(parse_style_attribute(&format!("text-combine-upright:digits {text}")).is_clean());
    }
    for text in ["1", "5", "-0000", "2147483648"] {
        assert!(!parse_style_attribute(&format!("text-combine-upright:digits {text}")).is_clean());
    }
}

#[test]
fn positive_column_counts_keep_exact_token_origins_at_small_and_large_magnitudes() {
    for text in ["+0002", "2147483648", "9999999999999999999999"] {
        let report = parse_style_attribute(&format!("column-count:{text}"));
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let declaration = &report.syntax()[0];
        let CssKnownPropertyValueRef::ColumnCount(count) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("column count")
        };
        let CssColumnCount::Count(CssPositiveIntegerValue::Literal(positive)) = count.count()
        else {
            panic!("positive literal")
        };
        assert_eq!(positive.integer().numeric().representation(), text);
        assert_eq!(
            positive.integer().component(),
            &declaration.value_components().items()[0]
        );
        assert_eq!(
            positive.integer().origin(),
            declaration.value_components().items()[0].origin()
        );
    }
}

#[test]
fn bounded_scalar_serialization_normalizes_output_without_changing_retained_tokens() {
    for (text, expected) in [("+.5000", "0.5"), ("50%", "0.5"), ("-000%", "0")] {
        let component = CssComponentValue::try_token(text).unwrap();
        let scalar = CssOpacityScalar::try_from_component(component.clone()).unwrap();
        let value = CssOpacityValue::Scalar(scalar);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, 10),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, 10),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        let CssOpacityValue::Scalar(scalar) = value else {
            unreachable!()
        };
        assert_eq!(scalar.component(), &component);
    }
}

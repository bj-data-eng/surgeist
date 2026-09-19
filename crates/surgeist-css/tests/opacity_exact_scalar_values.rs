#![forbid(unsafe_code)]
//! Exact decimal transport is independent of the tokenizer's binary32 cache.
//! Dyadic goldens below follow directly from powers of two, not parser output.

use surgeist_css::*;

fn declaration(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("opacity: {text} !important"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

fn wrapper(declaration: &CssDeclaration) -> &CssOpacityPropertyValue {
    let CssKnownPropertyValueRef::Opacity(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary opacity")
    };
    value
}

fn exact(value: &CssOpacityValue) -> &CssOpacityScalar {
    let CssOpacityValue::ExactScalar(value) = value else {
        panic!("exact authored decimal: {value:?}")
    };
    value
}

#[test]
fn parsed_exact_scalars_preserve_kind_spelling_and_original_span() {
    for text in [
        ".1",
        ".9",
        "1e-47",
        "-1e-47",
        "1e100",
        "-1e100",
        "1e-47%",
        "-1e-47%",
        "1e100%",
        "-1e100%",
        "1e1000000000",
    ] {
        let source = declaration(text);
        let value = exact(wrapper(&source).value());
        assert_eq!(value.numeric().representation(), text.trim_end_matches('%'));
        assert_eq!(
            value.kind(),
            if text.ends_with('%') {
                CssOpacityScalarKind::Percentage
            } else {
                CssOpacityScalarKind::Number
            }
        );
        assert!(wrapper(&source).i01_subset().is_none());
        let CssValueOrigin::Parsed(origin) = value.origin() else {
            panic!("original parsed origin")
        };
        assert_eq!(origin.span().start().byte_offset().value(), 9);
        assert_eq!(origin.span().end().byte_offset().value(), 9 + text.len());
        assert!(
            origin
                .source()
                .same_snapshot(source.parsed_value().unwrap().source())
        );
        assert_eq!(value.component().origin(), value.origin());
        let cloned = value.clone();
        drop(source);
        assert_eq!(
            cloned.numeric().representation(),
            text.trim_end_matches('%')
        );
    }
}

#[test]
fn checked_scalar_constructor_retains_numeric_components_without_rounding() {
    for text in [".5", ".1", "-1e-47", "1e100", "1e-47%", "-1e100%"] {
        let component = CssComponentValue::try_token(text).unwrap();
        let scalar = CssOpacityScalar::try_from_component(component.clone()).unwrap();
        assert_eq!(scalar.component(), &component);
        assert_eq!(
            scalar.numeric().representation(),
            text.trim_end_matches('%')
        );
        assert_eq!(scalar.origin(), &CssValueOrigin::Programmatic);
        let values = CssComponentValues::try_new(vec![component]).unwrap();
        let source = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Opacity),
            values.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(source.value_components(), &values);
        if text != ".5" {
            assert_eq!(exact(wrapper(&source).value()), &scalar);
        }
    }
}

#[test]
fn checked_scalar_rejects_other_component_kinds_with_their_actual_origins() {
    for text in [
        "name",
        "1px",
        "\"text\"",
        "url(path)",
        "#abc",
        ",",
        " ",
        "calc(1)",
        "(1)",
        "[1]",
        "{1}",
    ] {
        let parsed = parse_component_values(text).unwrap();
        let [component] = parsed.items() else {
            panic!("one component for {text}")
        };
        let error = CssOpacityScalar::try_from_component(component.clone()).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
        assert_eq!(error.origin(), component.origin());
    }
    let component = CssComponentValue::try_token("name").unwrap();
    let error = CssOpacityScalar::try_from_component(component).unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
    assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
}

#[test]
fn exact_dyadics_keep_legacy_branches_and_nearby_decimals_do_not() {
    for (text, expected) in [
        (".5", 0.5),
        ("5e-1", 0.5),
        ("0.5000", 0.5),
        ("0.100000001490116119384765625", f32::from_bits(0x3dcc_cccd)),
        (
            "0.00000000000000000000000000000000000000000000140129846432481707092372958328991613128026194187651577175706828388979108268586060148663818836212158203125",
            f32::from_bits(1),
        ),
    ] {
        let source = declaration(text);
        let CssOpacityValue::Literal(value) = wrapper(&source).value() else {
            panic!("exact binary32 dyadic {text}")
        };
        assert_eq!(value.value().to_bits(), expected.to_bits());
        assert!(wrapper(&source).i01_subset().is_some());
    }
    for (text, expected) in [
        ("-0.5", -0.5_f32),
        ("1.5", 1.5),
        ("340282346638528859811704183484516925440", f32::MAX),
    ] {
        let source = declaration(text);
        let CssOpacityValue::Number(value) = wrapper(&source).value() else {
            panic!("exact out-of-range number")
        };
        assert_eq!(value.value().to_bits(), expected.to_bits());
    }
    for text in [
        "0.100000001490116119384765624",
        "0.100000001490116119384765626",
        "340282346638528859811704183484516925441",
    ] {
        let source = declaration(text);
        assert_eq!(
            exact(wrapper(&source).value()).numeric().representation(),
            text
        );
    }
}

#[test]
fn percentage_classification_uses_coefficient_and_true_zero_is_positive() {
    for coefficient in 0..=100 {
        let source = declaration(&format!("{coefficient}%"));
        let CssOpacityValue::Percentage(value) = wrapper(&source).value() else {
            panic!("integer percentage coefficient")
        };
        assert_eq!(value.value(), coefficient as f32);
    }
    for text in ["-0", "0e99999999999999999999", "-0e-99999999999999999999"] {
        let source = declaration(text);
        let CssOpacityValue::Literal(value) = wrapper(&source).value() else {
            panic!("true zero")
        };
        assert_eq!(value.value().to_bits(), 0);
    }
    for text in ["-0%", "-0e99999999999999999999%"] {
        let source = declaration(text);
        let CssOpacityValue::Percentage(value) = wrapper(&source).value() else {
            panic!("true percentage zero")
        };
        assert_eq!(value.value().to_bits(), 0);
    }
}

fn contribution_scalar(value: &CssLonghandContribution) -> &CssOpacityScalar {
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::Opacity(value)) = value.value()
    else {
        panic!("ordinary opacity contribution")
    };
    exact(value)
}

#[test]
fn expansion_and_strict_reentry_transport_the_same_exact_payload() {
    let source = declaration("-1e-47%");
    let expected = exact(wrapper(&source).value());
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("one longhand")
    };
    assert_eq!(values.items().len(), 1);
    assert_eq!(contribution_scalar(&values.items()[0]), expected);
    assert!(values.items()[0].source().same_occurrence(&source));
    assert_eq!(
        values.items()[0].source().importance(),
        CssImportance::Important
    );
    let pending = declaration("var(--fade)");
    let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
        panic!("pending value")
    };
    let replacement = parse_component_values("1e100%").unwrap();
    let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("completed reentry")
    };
    assert_eq!(
        contribution_scalar(&values.items()[0]).component(),
        &replacement.items()[0]
    );
    assert_eq!(
        values.items()[0].replacement_components(),
        Some(&replacement)
    );
    assert!(values.items()[0].source().same_occurrence(&pending));
    assert_eq!(
        values.items()[0].source().importance(),
        CssImportance::Important
    );
    assert!(
        handle
            .reenter(parse_component_values("1px").unwrap())
            .is_err()
    );
}

#[test]
fn normalization_preserves_exact_values_and_declaration_order() {
    let report = parse_sheet(".a { opacity: .1; opacity: 1e100% !important; }");
    assert!(report.is_clean());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 2);
    for (index, expected) in [".1", "1e100"].into_iter().enumerate() {
        assert_eq!(values[index].order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            values[index].expansion()
        else {
            panic!("normalized exact value")
        };
        let scalar = contribution_scalar(&items.items()[0]);
        assert_eq!(scalar.numeric().representation(), expected);
        assert_eq!(scalar, exact(wrapper(values[index].source()).value()));
        assert_eq!(
            values[index].source().importance(),
            if index == 0 {
                CssImportance::Normal
            } else {
                CssImportance::Important
            }
        );
    }
}

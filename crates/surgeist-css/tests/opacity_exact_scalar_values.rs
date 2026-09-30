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
    let CssOpacityValue::Scalar(value) = value else {
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
        assert_eq!(exact(wrapper(&source).value()), &scalar);
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
fn all_scalar_magnitudes_retain_lexemes_and_kinds() {
    for text in [
        ".5",
        "5e-1",
        "0.5000",
        "-0.5",
        "1.5",
        "0.00000000000000000000000000000000000000000000140129846432481707092372958328991613128026194187651577175706828388979108268586060148663818836212158203125",
        "0.100000001490116119384765625",
        "340282346638528859811704183484516925440",
        "0.100000001490116119384765624",
        "0.100000001490116119384765626",
        "340282346638528859811704183484516925441",
        "-0",
        "0e99999999999999999999",
        "-0e-99999999999999999999",
        "-0%",
        "-0e99999999999999999999%",
    ] {
        let source = declaration(text);
        let scalar = exact(wrapper(&source).value());
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
        assert_eq!(
            scalar.component(),
            source
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
    }
    for coefficient in 0..=100 {
        let text = format!("{coefficient}%");
        let source = declaration(&text);
        let scalar = exact(wrapper(&source).value());
        assert_eq!(scalar.numeric().representation(), coefficient.to_string());
        assert_eq!(scalar.kind(), CssOpacityScalarKind::Percentage);
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

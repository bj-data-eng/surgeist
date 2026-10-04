#![forbid(unsafe_code)]
//! Contextual Number matching retains its unresolved percentage hint.
//! Color 5 WD 2026-09-08 §§4–5 admits Number or Percentage in non-hue
//! relative slots, custom-profile slots, and alpha; §6 legacy CMYK is Number-only.
//! Color 4 CRD 2026-09-08 defines Hue as Number or Angle, without Percentage.
//! https://www.w3.org/TR/2026/WD-css-color-5-20260908/#relative-colors
//! https://www.w3.org/TR/2026/WD-css-color-5-20260908/#color-function
//! https://www.w3.org/TR/2026/WD-css-color-5-20260908/#device-cmyk
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#hue-syntax
//! https://www.w3.org/TR/2024/WD-css-typed-om-1-20240321/#cssnumericvalue-match

use surgeist_css::*;

const HINTED: &str = "calc((1px + 1%) / 1px)";

fn components(text: &str) -> CssComponentValues {
    parse_component_values(text).unwrap()
}

fn assert_hinted_number(ty: CssNumericType) {
    for axis in [
        CssNumericDimension::Length,
        CssNumericDimension::Angle,
        CssNumericDimension::Time,
        CssNumericDimension::Frequency,
        CssNumericDimension::Resolution,
        CssNumericDimension::Flex,
        CssNumericDimension::Percentage,
    ] {
        assert_eq!(ty.exponent(axis), 0);
    }
    assert_eq!(ty.percent_hint(), Some(CssNumericDimension::Length));
}

fn assert_profile(expression: &CssProfileColorExpression) {
    let CssProfileColorExpressionRef::Calculation(calculation) = expression.view() else {
        panic!("retained profile calculation")
    };
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    assert_hinted_number(calculation.expression().numeric_type());
    assert!(calculation.references().is_empty());
    assert_eq!(
        calculation.components().serialize().unwrap().as_css(),
        HINTED
    );
}

fn assert_relative(expression: &CssRelativeColorExpression) {
    let CssRelativeColorExpressionValue::Calculation(calculation) = expression.value() else {
        panic!("retained relative calculation")
    };
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    assert_hinted_number(calculation.expression().numeric_type());
    assert!(calculation.references().is_empty());
    assert_eq!(calculation.authored().as_css(), HINTED);
}

fn parsed(text: &str) -> CssDeclaration {
    let source = format!("color:{text}!important;height:1px");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Height
    );
    assert!(validate_style_attribute(&source).is_ok());
    let declaration = report.syntax()[0].clone();
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        text
    );
    assert_eq!(declaration.importance(), CssImportance::Important);
    declaration
}

fn checked(text: &str) -> CssDeclaration {
    let original = components(text);
    let result = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        original.clone(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {text}: {error:?}"));
    assert_eq!(result.value_components(), &original);
    assert!(result.position().is_none());
    result
}

fn color(source: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("color property")
    };
    value.value()
}

fn assert_reentry(text: &str) {
    let source = checked("var(--paint)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending color")
    };
    let replacement = components(text);
    let CssContributions::Longhands(actual) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("color replacement")
    };
    let [item] = actual.items() else {
        panic!("one contribution")
    };
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(&replacement));
    let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
        expand_declaration(&checked(text)).unwrap()
    else {
        panic!("checked color")
    };
    assert_eq!(item.value(), expected.items()[0].value());
}

#[test]
fn profile_expression_constructor_matches_number_with_retained_hint() {
    let original = components(HINTED);
    let value = CssProfileColorExpression::try_from_components(original.clone()).unwrap();
    assert_profile(&value);
    assert_eq!(value.components(), &original);
    assert_eq!(value.origin(), original.items()[0].origin());
}

#[test]
fn relative_custom_profile_channels_and_alpha_admit_hinted_numbers_through_frontdoors() {
    let text = format!("color(from red --P {HINTED} / {HINTED})");
    for source in [parsed(&text), checked(&text)] {
        let value = color(&source).relative_custom_value().unwrap();
        assert_eq!(value.profile().as_str(), "--P");
        assert_eq!(value.channels().len(), 1);
        assert_profile(&value.channels()[0]);
        assert_profile(value.alpha().unwrap());
    }
    assert_reentry(&text);
}

#[test]
fn relative_expression_constructors_admit_hinted_number_only_in_percentage_permitting_slots() {
    use CssRelativeColorEnvironment as E;
    use CssRelativeColorResultDomain as D;
    // Every listed channel grammar explicitly permits Number or Percentage.
    for environment in [
        E::Rgb,
        E::Hsl,
        E::Hwb,
        E::Lab,
        E::Lch,
        E::Oklab,
        E::Oklch,
        E::PredefinedRgb(CssPredefinedColorSpace::Srgb),
        E::Xyz(CssPredefinedColorSpace::XyzD65),
    ] {
        for domain in [D::NumberPercentage, D::Alpha] {
            let original = components(HINTED);
            let value = CssRelativeColorExpression::try_from_components(
                original.clone(),
                environment,
                domain,
            )
            .unwrap();
            assert_relative(&value);
            assert_eq!(value.environment(), environment);
            assert_eq!(value.result_domain(), domain);
            assert_eq!(value.origin(), original.items()[0].origin());
        }
    }
    let value =
        CssRelativeColorExpression::try_from_components(components(HINTED), E::Alpha, D::Alpha)
            .unwrap();
    assert_relative(&value);
}

#[test]
fn relative_non_hue_channels_and_alpha_admit_hinted_numbers_through_frontdoors() {
    for (prefix, before, after, slot) in [
        ("rgb", "", " g b", 0),
        ("hsl", "h ", " l", 1),
        ("hwb", "h ", " b", 1),
        ("lab", "", " a b", 0),
        ("lch", "", " c h", 0),
        ("oklab", "", " a b", 0),
        ("oklch", "", " c h", 0),
        ("color", "srgb ", " g b", 0),
        ("color", "xyz-d65 ", " y z", 0),
    ] {
        let text = format!("{prefix}(from red {before}{HINTED}{after} / {HINTED})");
        for source in [parsed(&text), checked(&text)] {
            let value = color(&source).relative_value().unwrap();
            assert_relative(&value.channels()[slot]);
            assert_relative(value.alpha().unwrap());
        }
        assert_reentry(&text);
    }
    let text = format!("alpha(from red / {HINTED})");
    for source in [parsed(&text), checked(&text)] {
        assert_relative(color(&source).alpha_value().unwrap().alpha().unwrap());
    }
    assert_reentry(&text);
}

fn rejects(text: &str) {
    let source = format!("color:{text};height:1px");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 1, "{text}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Height
    );
    assert_eq!(report.diagnostics().len(), 1, "{text}");
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            components(text),
            CssImportance::Normal
        )
        .is_err()
    );
    let pending = checked("var(--paint)");
    let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
        panic!("pending")
    };
    assert!(matches!(
        handle.reenter(components(text)).unwrap_err().kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
}

#[test]
fn hue_does_not_gain_percentage_permission_from_neighboring_channels() {
    use CssRelativeColorEnvironment as E;
    use CssRelativeColorResultDomain as D;
    for environment in [E::Hsl, E::Hwb, E::Lch, E::Oklch] {
        let error = CssRelativeColorExpression::try_from_components(
            components(HINTED),
            environment,
            D::Hue,
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert!(error.origin().is_some());
        assert!(
            CssRelativeColorExpression::try_from_components(
                components("calc(1deg / 1deg)"),
                environment,
                D::Hue
            )
            .is_ok()
        );
    }
    for text in [
        format!("hsl(from red {HINTED} s l)"),
        format!("hwb(from red {HINTED} w b)"),
        format!("lch(from red l c {HINTED})"),
        format!("oklch(from red l c {HINTED})"),
    ] {
        rejects(&text);
    }
}

#[test]
fn legacy_number_only_device_cmyk_keeps_its_percentage_hint_rejection() {
    rejects(&format!("device-cmyk({HINTED}, 0, 0, 0)"));
    rejects("device-cmyk(1%, 0, 0, 0)");
    for source in [
        parsed("device-cmyk(calc(1px / 1px), 0, 0, 0)"),
        checked("device-cmyk(calc(1px / 1px), 0, 0, 0)"),
    ] {
        assert!(color(&source).device_cmyk_value().is_some());
    }
}

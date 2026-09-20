#![forbid(unsafe_code)]
//! Authored custom color and alpha grammar from the selected Color5 definition.
//! Profile binding and computed colors belong to later owners.
use surgeist_css::*;

fn parsed(text: &str) -> CssDeclaration {
    let source = format!("color:{text}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one color declaration")
    };
    let declaration = declaration.clone();
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        text
    );
    let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin() else {
        panic!("original function origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 6);
    declaration
}

fn checked(text: &str) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(declaration.value_components(), &components);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_value().is_none());
    declaration
}

fn wrapper(declaration: &CssDeclaration) -> &CssColorPropertyValue {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary color property")
    };
    value
}

fn admits_current_only(text: &str) {
    for declaration in [parsed(text), checked(text)] {
        let value = wrapper(&declaration);
        assert_eq!(value.as_css(), text);
        assert!(value.i01_subset().is_none(), "{text}");
    }
}

fn rejects(text: &str) {
    let report = parse_style_attribute(&format!("color:{text};opacity:.5"));
    assert_eq!(report.diagnostics().len(), 1, "{text}");
    assert_eq!(report.syntax().len(), 1, "{text}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            parse_component_values(text).unwrap(),
            CssImportance::Normal,
        )
        .is_err(),
        "{text}"
    );
}

#[test]
fn ordinary_custom_colors_accept_nonempty_variable_component_lists() {
    for text in ["color(--P 1)", "color(--P 1 2)", "color(--P 1 2 3 4 5)"] {
        admits_current_only(text);
    }
}

#[test]
fn ordinary_custom_colors_preserve_exact_literals_missing_values_and_math() {
    for text in [
        "color(--P 1e100 none 20% -2)",
        "color(--P 1e-100 0.1% / none)",
        "color(--P calc(2) calc(120%) / calc(-5%))",
    ] {
        admits_current_only(text);
    }
}

#[test]
fn custom_profile_names_accept_case_escapes_and_bare_dashes() {
    for text in [
        "color(--Profile 1)",
        "color(--profile 1)",
        "color(--Pr\\6f file 1)",
        "color(--a\\ b 1)",
        "color(-- 1)",
    ] {
        admits_current_only(text);
    }
}

#[test]
fn relative_custom_colors_admit_unbound_profile_names_in_channels_and_alpha() {
    for text in [
        "color(from red --P cyan calc(magenta * 2) / alpha)",
        "color(from red --P alpha none default initial --channel / cyan)",
        "color(from red --P c\\79 an / calc(alpha / 2))",
    ] {
        admits_current_only(text);
    }
}

#[test]
fn relative_custom_colors_admit_direct_constant_names_and_constant_math() {
    // A direct pi may name a profile channel; inside math it is a constant.
    // Typed reference-versus-constant inspection belongs to the expression API.
    for text in [
        "color(from red --P pi calc(pi) calc(pi + cyan))",
        "color(from red --P e calc(e * cyan))",
        "color(from red --P infinity calc(infinity) NaN calc(NaN))",
    ] {
        admits_current_only(text);
    }
}

#[test]
fn relative_alpha_accepts_an_omitted_override() {
    admits_current_only("alpha(from red)");
}

#[test]
fn relative_alpha_accepts_none_numeric_percentage_and_channel_math_overrides() {
    for text in [
        "alpha(from red / none)",
        "alpha(from red / 0.1)",
        "alpha(from red / 25%)",
        "alpha(from red / alpha)",
        "alpha(from red / calc(alpha / 2))",
        "alpha(from red / calc(120%))",
    ] {
        admits_current_only(text);
    }
}

#[test]
fn aggregate_border_color_transports_custom_and_alpha_colors_without_frozen_projection() {
    for text in [
        "color(--P 1e100 none 20% -2)",
        "alpha(from red / calc(alpha / 2))",
    ] {
        let source = format!("border-color:{text}!important");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1);
        let components = parse_component_values(text).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::BorderColor),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &components);
        for declaration in [report.syntax()[0].clone(), checked] {
            assert_eq!(declaration.importance(), CssImportance::Important);
            assert_eq!(
                declaration.value_components().serialize().unwrap().as_css(),
                text
            );
            let CssKnownPropertyValueRef::BorderColor(value) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("border color")
            };
            let colors = value.current();
            assert_eq!(colors.top(), colors.right());
            assert_eq!(colors.top(), colors.bottom());
            assert_eq!(colors.top(), colors.left());
            assert!(value.i01_subset().is_none());
        }
    }
}

#[test]
fn nested_custom_and_alpha_descendants_keep_enclosing_projection_conservative() {
    for text in [
        "color-mix(in srgb, color(--P 1), blue)",
        "rgb(from alpha(from red / none) r g b)",
        "alpha(from color(from red --P cyan) / alpha)",
    ] {
        admits_current_only(text);
    }
}

#[test]
fn substitution_reentry_accepts_expanded_custom_and_alpha_colors() {
    for declaration in [parsed("var(--paint)"), checked("var(--paint)")] {
        let CssExpansion::Pending(pending) = expand_declaration(&declaration).unwrap() else {
            panic!("pending color")
        };
        for text in [
            "color(--P 1e100 none 20% -2)",
            "color(from red --P cyan calc(magenta * 2) / alpha)",
            "alpha(from red / calc(alpha / 2))",
        ] {
            let replacement = parse_component_values(text).unwrap();
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("color contribution")
            };
            let [value] = values.items() else {
                panic!("one longhand")
            };
            assert_eq!(value.property(), CssKnownProperty::Color);
            assert!(value.ordinary_value().is_some());
            assert!(value.source().same_occurrence(&declaration));
            assert_eq!(value.source().importance(), CssImportance::Important);
            assert_eq!(value.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn substitutions_remain_pending_and_strict_reentry_rejects_residuals() {
    for text in [
        "color(--P var(--channels))",
        "color(from var(--source) --P cyan)",
        "alpha(from red / env(opacity))",
    ] {
        for declaration in [parsed(text), checked(text)] {
            let known = declaration.known().unwrap();
            assert!(known.property_value().is_none());
            assert_eq!(known.substitution_dependent().unwrap().as_css(), text);
            let CssExpansion::Pending(pending) = expand_declaration(&declaration).unwrap() else {
                panic!("pending color")
            };
            for residual in [
                "color(--P var(--channels))",
                "alpha(from red / env(opacity))",
            ] {
                assert_eq!(
                    pending
                        .reenter(parse_component_values(residual).unwrap())
                        .unwrap_err()
                        .kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
            }
            assert!(
                pending
                    .reenter(parse_component_values("color(--P 1px)").unwrap())
                    .is_err()
            );
        }
    }
}

#[test]
fn malformed_custom_lists_and_wrong_dimensions_stay_invalid() {
    for text in [
        "color(--P)",
        "color(--P / .5)",
        "color(--P 1, 2)",
        "color(--P 1 /)",
        "color(--P 1 / .5 / .2)",
        "color(--P 1px)",
        "color(--P calc(1deg))",
        "color(from red --P)",
        "color(from red --P calc(cyan + 1%))",
        "color(from red --P calc(none))",
        "color(from red --P cyan / calc(1px))",
    ] {
        rejects(text);
    }
}

#[test]
fn alpha_requires_from_and_rejects_foreign_channels_dimensions_and_extra_slashes() {
    for text in [
        "alpha()",
        "alpha(red)",
        "alpha(/ .5)",
        "alpha(from red /)",
        "alpha(from red / .5 / .2)",
        "alpha(from red / r)",
        "alpha(from red / cyan)",
        "alpha(from red / calc(h / 2))",
        "alpha(from red / 1px)",
        "alpha(from red / calc(1deg))",
    ] {
        rejects(text);
    }
}

#[test]
fn profile_channel_names_do_not_leak_into_ordinary_or_predefined_numeric_roots() {
    for text in [
        "color(--P cyan)",
        "color(--P pi)",
        "color(--P calc(cyan))",
        "color(--P 1 / alpha)",
        "rgb(from red cyan g b)",
        "color(from red srgb cyan g b)",
    ] {
        rejects(text);
    }
    assert!(
        CssNumberCalculation::try_from_components(parse_component_values("calc(cyan)").unwrap())
            .is_err()
    );
    assert!(
        CssPercentageCalculation::try_from_components(
            parse_component_values("calc(cyan)").unwrap()
        )
        .is_err()
    );
    assert!(
        CssNumberCalculation::try_from_components(parse_component_values("calc(pi)").unwrap())
            .is_ok()
    );
}

#[test]
fn existing_predefined_and_two_color_frozen_values_remain_available() {
    for text in [
        "color(srgb 1 0 0)",
        "rgb(from red r g b)",
        "color-mix(in srgb, red 25%, blue 50%)",
    ] {
        for declaration in [parsed(text), checked(text)] {
            assert!(wrapper(&declaration).i01_subset().is_some(), "{text}");
        }
    }
}

// Each witness is a separate test so an expected parsed failure cannot hide
// checked admission, alpha math, aggregate transport, or substitution reentry.
fn checked_current_only(text: &str) {
    let declaration = checked(text);
    let value = wrapper(&declaration);
    assert_eq!(value.as_css(), text);
    assert!(value.i01_subset().is_none());
}

#[test]
fn checked_custom_color_accepts_exact_variable_channels() {
    checked_current_only("color(--P 1e100 none 20% -2)");
}

#[test]
fn checked_relative_custom_color_accepts_symbolic_channel_math() {
    checked_current_only("color(from red --P cyan calc(magenta * 2) / alpha)");
}

#[test]
fn checked_relative_alpha_accepts_transparency_math() {
    checked_current_only("alpha(from red / calc(alpha / 2))");
}

#[test]
fn parsed_relative_alpha_accepts_transparency_math() {
    let declaration = parsed("alpha(from red / calc(alpha / 2))");
    assert!(wrapper(&declaration).i01_subset().is_none());
}

fn reenters_current_color(text: &str) {
    let declaration = checked("var(--paint)");
    let CssExpansion::Pending(pending) = expand_declaration(&declaration).unwrap() else {
        panic!("pending color")
    };
    let replacement = parse_component_values(text).unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("color contribution")
    };
    let [value] = values.items() else {
        panic!("one color contribution")
    };
    assert_eq!(value.property(), CssKnownProperty::Color);
    assert!(value.ordinary_value().is_some());
    assert!(value.source().same_occurrence(&declaration));
    assert_eq!(value.source().importance(), CssImportance::Important);
    assert_eq!(value.replacement_components(), Some(&replacement));
}

#[test]
fn substitution_reentry_accepts_relative_custom_channel_math() {
    reenters_current_color("color(from red --P cyan calc(magenta * 2) / alpha)");
}

#[test]
fn substitution_reentry_accepts_relative_alpha_math() {
    reenters_current_color("alpha(from red / calc(alpha / 2))");
}

fn checked_border_current_only(text: &str) {
    let components = parse_component_values(text).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BorderColor),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &components);
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssKnownPropertyValueRef::BorderColor(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("border color")
    };
    let colors = value.current();
    assert_eq!(colors.top(), colors.right());
    assert_eq!(colors.top(), colors.bottom());
    assert_eq!(colors.top(), colors.left());
    assert!(value.i01_subset().is_none());
}

#[test]
fn checked_border_color_transports_custom_channels() {
    checked_border_current_only("color(--P 1e100 none 20% -2)");
}

#[test]
fn checked_border_color_transports_relative_alpha_math() {
    checked_border_current_only("alpha(from red / calc(alpha / 2))");
}

#[test]
fn parsed_border_color_transports_relative_alpha_math() {
    let text = "alpha(from red / calc(alpha / 2))";
    let report = parse_style_attribute(&format!("border-color:{text}!important"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        text
    );
    let CssKnownPropertyValueRef::BorderColor(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("border color")
    };
    assert!(value.i01_subset().is_none());
}

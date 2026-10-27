//! Authored custom color and alpha grammar from the selected Color5 definition.
//! Profile binding and computed colors belong to later owners.
use crate::css::support::color_and_selector_fixtures::{
    assert_color_value_rejected_at, parsed_important_declaration,
};
use crate::css::support::color_property::color_property_value as wrapper;
use crate::css::support::detached_checked_declaration;
use surgeist_css::CssParser;
use surgeist_css::color::{
    CssColorComponent, CssProfileColorExpressionRef, CssRelativeColorChannel,
    CssRelativeColorExpressionValue,
};
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::numeric::{CssCalculation, CssNumber, CssPercentage};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef,
    CssPropertyTarget,
};
use surgeist_css::values::common::CssBoxSideKind;

fn admits_authored_color(text: &str) {
    for declaration in [
        parsed_important_declaration("color", text),
        detached_checked_declaration("color", text, false),
    ] {
        let value = wrapper(&declaration);
        assert_eq!(value.as_css(), text);
        let color = value.value();
        if text.starts_with("color(from") {
            assert!(color.relative_custom_value().is_some(), "{text}");
        } else if text.starts_with("color(--") {
            assert!(color.custom_value().is_some(), "{text}");
        } else if text.starts_with("alpha(from") {
            assert!(color.alpha_value().is_some(), "{text}");
        } else if text.starts_with("color-mix(") {
            assert!(color.color_mix_value().is_some(), "{text}");
        } else if text.starts_with("rgb(from") {
            assert!(color.relative_value().is_some(), "{text}");
        } else {
            panic!("unexpected color branch: {text}");
        }
    }
}

/// The color grammar rejects `text` through the style attribute, dropping
/// only its declaration, and through checked components. `responsible` is
/// the first token that violates the grammar with its byte offset in `text`;
/// a truncated argument list or a calculation names no single token, so only
/// the color grammar kind is checked.
fn rejects(text: &str, responsible: Option<(&str, usize)>) {
    let report = CssParser::default().parse_block_contents(format!("color:{text};opacity:.5"));
    let [diagnostic] = report.diagnostics() else {
        panic!("{text}: one diagnostic: {report:?}")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidColorSyntax,
        "{text}"
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    if let Some((_, at)) = responsible {
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            "color:".len() + at,
            "{text}"
        );
    }
    assert_eq!(report.syntax().len(), 1, "{text}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    let checked = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
        CssComponentValues::parse(&CssParser::default(), text).unwrap(),
        CssImportance::Normal,
    );
    match responsible {
        Some((token, at)) => {
            assert_color_value_rejected_at(checked, CssKnownProperty::Color, text, token, at)
        }
        None => assert!(
            checked.unwrap_err().code() == Some(CssErrorCode::InvalidColorSyntax),
            "{text}"
        ),
    }
}

#[test]
fn ordinary_custom_colors_accept_nonempty_variable_component_lists() {
    for text in ["color(--P 1)", "color(--P 1 2)", "color(--P 1 2 3 4 5)"] {
        admits_authored_color(text);
    }
}

#[test]
fn ordinary_custom_colors_preserve_exact_literals_missing_values_and_math() {
    for text in [
        "color(--P 1e100 none 20% -2)",
        "color(--P 1e-100 0.1% / none)",
        "color(--P calc(2) calc(120%) / calc(-5%))",
    ] {
        admits_authored_color(text);
    }
    for declaration in [
        parsed_important_declaration("color", "color(--P 1e100 none 20% -2)"),
        detached_checked_declaration("color", "color(--P 1e100 none 20% -2)", false),
    ] {
        let custom = wrapper(&declaration).value().custom_value().unwrap();
        assert_eq!(custom.profile().as_str(), "--P");
        assert_eq!(custom.channels().len(), 4);
        let CssColorComponent::Number(large) = &custom.channels()[0] else {
            panic!("large number")
        };
        assert_eq!(large.numeric().representation(), "1e100");
        assert_eq!(large.component().origin(), large.origin());
        assert!(matches!(custom.channels()[1], CssColorComponent::None));
        let CssColorComponent::Percentage(pct) = &custom.channels()[2] else {
            panic!("percentage")
        };
        assert_eq!(pct.numeric().representation(), "20");
        let CssColorComponent::Number(negative) = &custom.channels()[3] else {
            panic!("negative number")
        };
        assert_eq!(negative.numeric().representation(), "-2");
        assert!(custom.alpha().is_none());
    }
    for declaration in [
        parsed_important_declaration("color", "color(--P 1e-100 0.1% / none)"),
        detached_checked_declaration("color", "color(--P 1e-100 0.1% / none)", false),
    ] {
        let custom = wrapper(&declaration).value().custom_value().unwrap();
        let CssColorComponent::Number(tiny) = &custom.channels()[0] else {
            panic!("tiny number")
        };
        assert_eq!(tiny.numeric().representation(), "1e-100");
        let CssColorComponent::Percentage(pct) = &custom.channels()[1] else {
            panic!("small percentage")
        };
        assert_eq!(pct.numeric().representation(), "0.1");
        assert!(matches!(custom.alpha(), Some(CssColorComponent::None)));
    }
}

#[test]
fn custom_profile_names_accept_case_escapes_and_bare_dashes() {
    for (text, decoded) in [
        ("color(--Profile 1)", "--Profile"),
        ("color(--profile 1)", "--profile"),
        ("color(--Pr\\6f file 1)", "--Profile"),
        ("color(--a\\ b 1)", "--a b"),
        ("color(-- 1)", "--"),
    ] {
        admits_authored_color(text);
        for declaration in [
            parsed_important_declaration("color", text),
            detached_checked_declaration("color", text, false),
        ] {
            assert_eq!(
                wrapper(&declaration)
                    .value()
                    .custom_value()
                    .unwrap()
                    .profile()
                    .as_str(),
                decoded
            );
        }
    }
}

#[test]
fn relative_custom_colors_admit_unbound_profile_names_in_channels_and_alpha() {
    for text in [
        "color(from red --P cyan calc(magenta * 2) / alpha)",
        "color(from red --P alpha none default initial --channel / cyan)",
        "color(from red --P c\\79 an / calc(alpha / 2))",
    ] {
        admits_authored_color(text);
    }
    for declaration in [
        parsed_important_declaration(
            "color",
            "color(from red --P cyan calc(magenta * 2) / alpha)",
        ),
        detached_checked_declaration(
            "color",
            "color(from red --P cyan calc(magenta * 2) / alpha)",
            false,
        ),
    ] {
        let relative = wrapper(&declaration)
            .value()
            .relative_custom_value()
            .unwrap();
        assert_eq!(relative.source().named().unwrap().name(), "red");
        assert_eq!(relative.profile().as_str(), "--P");
        assert_eq!(relative.channels().len(), 2);
        assert!(
            matches!(relative.channels()[0].view(), CssProfileColorExpressionRef::Reference(name) if name.as_str() == "cyan")
        );
        assert!(
            matches!(relative.channels()[1].view(), CssProfileColorExpressionRef::Calculation(calc) if calc.references().iter().map(|name| name.as_str()).collect::<Vec<_>>() == ["magenta"])
        );
        assert!(
            matches!(relative.alpha().unwrap().view(), CssProfileColorExpressionRef::Reference(name) if name.as_str() == "alpha")
        );
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
        admits_authored_color(text);
    }
}

#[test]
fn relative_alpha_accepts_an_omitted_override() {
    admits_authored_color("alpha(from red)");
    for declaration in [
        parsed_important_declaration("color", "alpha(from red)"),
        detached_checked_declaration("color", "alpha(from red)", false),
    ] {
        let alpha = wrapper(&declaration).value().alpha_value().unwrap();
        assert_eq!(alpha.source().named().unwrap().name(), "red");
        assert!(alpha.alpha().is_none());
    }
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
        admits_authored_color(text);
    }
    for (text, expected) in [
        ("alpha(from red / none)", "none"),
        ("alpha(from red / 0.1)", "number"),
        ("alpha(from red / 25%)", "percentage"),
        ("alpha(from red / alpha)", "channel"),
        ("alpha(from red / calc(alpha / 2))", "calculation"),
    ] {
        for declaration in [
            parsed_important_declaration("color", text),
            detached_checked_declaration("color", text, false),
        ] {
            let alpha = wrapper(&declaration)
                .value()
                .alpha_value()
                .unwrap()
                .alpha()
                .unwrap();
            match (expected, alpha.value()) {
                ("none", CssRelativeColorExpressionValue::None) => {}
                ("number", CssRelativeColorExpressionValue::Number(v)) => {
                    assert_eq!(v.numeric().representation(), "0.1")
                }
                ("percentage", CssRelativeColorExpressionValue::Percentage(v)) => {
                    assert_eq!(v.numeric().representation(), "25")
                }
                (
                    "channel",
                    CssRelativeColorExpressionValue::Channel(CssRelativeColorChannel::Alpha),
                ) => {}
                ("calculation", CssRelativeColorExpressionValue::Calculation(calc)) => {
                    assert_eq!(calc.references(), [CssRelativeColorChannel::Alpha])
                }
                _ => panic!("wrong alpha branch: {text}"),
            }
        }
    }
}

#[test]
fn aggregate_border_color_transports_custom_and_alpha_colors() {
    for text in [
        "color(--P 1e100 none 20% -2)",
        "alpha(from red / calc(alpha / 2))",
    ] {
        let source = format!("border-color:{text}!important");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1);
        let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        let checked = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::BorderColor)),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &components);
        for declaration in [report.syntax()[0].clone(), checked] {
            assert_eq!(declaration.importance(), CssImportance::Important);
            assert_eq!(
                declaration
                    .value_components()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                text
            );
            let CssKnownPropertyValueRef::BorderColor(value) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("border color")
            };
            let colors = value.value();
            assert_eq!(colors.kind(), CssBoxSideKind::Physical);
            assert_eq!(colors.authored().len(), 1);
            let [top, right, bottom, left] = colors.assigned();
            assert_eq!(top, right);
            assert_eq!(top, bottom);
            assert_eq!(top, left);
            if text.starts_with("alpha(") {
                assert!(top.alpha_value().is_some());
            } else {
                assert!(top.custom_value().is_some());
            }
        }
    }
}

#[test]
fn nested_custom_and_alpha_descendants_keep_typed_branches() {
    for text in [
        "color-mix(in srgb, color(--P 1), blue)",
        "rgb(from alpha(from red / none) r g b)",
        "alpha(from color(from red --P cyan) / alpha)",
    ] {
        admits_authored_color(text);
    }
}

#[test]
fn substitution_reentry_accepts_expanded_custom_and_alpha_colors() {
    for declaration in [
        parsed_important_declaration("color", "var(--paint)"),
        detached_checked_declaration("color", "var(--paint)", false),
    ] {
        let CssExpansion::Pending(pending) = declaration.expand().unwrap() else {
            panic!("pending color")
        };
        for text in [
            "color(--P 1e100 none 20% -2)",
            "color(from red --P cyan calc(magenta * 2) / alpha)",
            "alpha(from red / calc(alpha / 2))",
        ] {
            let replacement = CssComponentValues::parse(&CssParser::default(), text).unwrap();
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
        for declaration in [
            parsed_important_declaration("color", text),
            detached_checked_declaration("color", text, false),
        ] {
            let known = declaration.known().unwrap();
            assert!(known.property_value().is_none());
            assert_eq!(known.substitution_dependent().unwrap().as_css(), text);
            let CssExpansion::Pending(pending) = declaration.expand().unwrap() else {
                panic!("pending color")
            };
            for residual in [
                "color(--P var(--channels))",
                "alpha(from red / env(opacity))",
            ] {
                assert!(
                    pending
                        .reenter(
                            CssComponentValues::parse(&CssParser::default(), residual).unwrap()
                        )
                        .unwrap_err()
                        .is_residual_substitution()
                );
            }
            assert!(
                pending
                    .reenter(
                        CssComponentValues::parse(&CssParser::default(), "color(--P 1px)").unwrap()
                    )
                    .is_err()
            );
        }
    }
}

#[test]
fn malformed_custom_lists_and_wrong_dimensions_stay_invalid() {
    for (text, responsible) in [
        ("color(--P)", None),
        ("color(--P / .5)", Some(("/", 10))),
        ("color(--P 1, 2)", Some((",", 11))),
        ("color(--P 1 /)", None),
        ("color(--P 1 / .5 / .2)", Some(("/", 17))),
        ("color(--P 1px)", Some(("1px", 10))),
        ("color(--P calc(1deg))", None),
        ("color(from red --P)", None),
        ("color(from red --P calc(cyan + 1%))", None),
        ("color(from red --P calc(none))", None),
        ("color(from red --P cyan / calc(1px))", None),
    ] {
        rejects(text, responsible);
    }
}

#[test]
fn alpha_requires_from_and_rejects_foreign_channels_dimensions_and_extra_slashes() {
    for (text, responsible) in [
        ("alpha()", None),
        ("alpha(red)", Some(("red", 6))),
        ("alpha(/ .5)", Some(("/", 6))),
        ("alpha(from red /)", None),
        ("alpha(from red / .5 / .2)", Some(("/", 20))),
        ("alpha(from red / r)", Some(("r", 17))),
        ("alpha(from red / cyan)", Some(("cyan", 17))),
        ("alpha(from red / calc(h / 2))", None),
        ("alpha(from red / 1px)", Some(("1px", 17))),
        ("alpha(from red / calc(1deg))", None),
    ] {
        rejects(text, responsible);
    }
}

#[test]
fn profile_channel_names_do_not_leak_into_ordinary_or_predefined_numeric_roots() {
    for (text, responsible) in [
        ("color(--P cyan)", Some(("cyan", 10))),
        ("color(--P pi)", Some(("pi", 10))),
        ("color(--P calc(cyan))", None),
        ("color(--P 1 / alpha)", Some(("alpha", 14))),
        ("rgb(from red cyan g b)", Some(("cyan", 13))),
        ("color(from red srgb cyan g b)", Some(("cyan", 20))),
    ] {
        rejects(text, responsible);
    }
    assert!(
        CssCalculation::<CssNumber>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), "calc(cyan)").unwrap(),
            Default::default()
        )
        .is_err()
    );
    assert!(
        CssCalculation::<CssPercentage>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), "calc(cyan)").unwrap(),
            Default::default()
        )
        .is_err()
    );
    assert!(
        CssCalculation::<CssNumber>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), "calc(pi)").unwrap(),
            Default::default()
        )
        .is_ok()
    );
}

#[test]
fn predefined_relative_and_two_color_values_remain_available() {
    for text in [
        "color(srgb 1 0 0)",
        "rgb(from red r g b)",
        "color-mix(in srgb, red 25%, blue 50%)",
    ] {
        for declaration in [
            parsed_important_declaration("color", text),
            detached_checked_declaration("color", text, false),
        ] {
            let color = wrapper(&declaration).value();
            if text.starts_with("color(srgb") {
                assert!(color.predefined_value().is_some());
            } else if text.starts_with("rgb(from") {
                assert!(color.relative_value().is_some());
            } else {
                assert_eq!(color.color_mix_value().unwrap().components().len(), 2);
            }
        }
    }
}

// Each witness is a separate test so an expected parsed failure cannot hide
// checked admission, alpha math, aggregate transport, or substitution reentry.
fn assert_checked_color_value(text: &str) {
    let declaration = detached_checked_declaration("color", text, false);
    let value = wrapper(&declaration);
    assert_eq!(value.as_css(), text);
    let color = value.value();
    if text.starts_with("color(from") {
        assert!(color.relative_custom_value().is_some());
    } else if text.starts_with("color(--") {
        assert!(color.custom_value().is_some());
    } else {
        assert!(color.alpha_value().is_some());
    }
}

#[test]
fn checked_custom_color_accepts_exact_variable_channels() {
    assert_checked_color_value("color(--P 1e100 none 20% -2)");
}

#[test]
fn checked_relative_custom_color_accepts_symbolic_channel_math() {
    assert_checked_color_value("color(from red --P cyan calc(magenta * 2) / alpha)");
}

#[test]
fn checked_relative_alpha_accepts_transparency_math() {
    assert_checked_color_value("alpha(from red / calc(alpha / 2))");
}

#[test]
fn parsed_relative_alpha_accepts_transparency_math() {
    let declaration = parsed_important_declaration("color", "alpha(from red / calc(alpha / 2))");
    assert!(wrapper(&declaration).value().alpha_value().is_some());
}

fn reenters_color(text: &str) {
    let declaration = detached_checked_declaration("color", "var(--paint)", false);
    let CssExpansion::Pending(pending) = declaration.expand().unwrap() else {
        panic!("pending color")
    };
    let replacement = CssComponentValues::parse(&CssParser::default(), text).unwrap();
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
    reenters_color("color(from red --P cyan calc(magenta * 2) / alpha)");
}

#[test]
fn substitution_reentry_accepts_relative_alpha_math() {
    reenters_color("alpha(from red / calc(alpha / 2))");
}

fn assert_checked_border_color_value(text: &str) {
    let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
    let declaration = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::BorderColor)),
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
    let colors = value.value();
    assert_eq!(colors.kind(), CssBoxSideKind::Physical);
    assert_eq!(colors.authored().len(), 1);
    let [top, right, bottom, left] = colors.assigned();
    assert_eq!(top, right);
    assert_eq!(top, bottom);
    assert_eq!(top, left);
    if text.starts_with("alpha(") {
        assert!(top.alpha_value().is_some());
    } else {
        assert!(top.custom_value().is_some());
    }
}

#[test]
fn checked_border_color_transports_custom_channels() {
    assert_checked_border_color_value("color(--P 1e100 none 20% -2)");
}

#[test]
fn checked_border_color_transports_relative_alpha_math() {
    assert_checked_border_color_value("alpha(from red / calc(alpha / 2))");
}

#[test]
fn parsed_border_color_transports_relative_alpha_math() {
    let text = "alpha(from red / calc(alpha / 2))";
    let report =
        CssParser::default().parse_block_contents(format!("border-color:{text}!important"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        declaration
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        text
    );
    let CssKnownPropertyValueRef::BorderColor(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("border color")
    };
    assert!(value.value().assigned()[0].alpha_value().is_some());
}

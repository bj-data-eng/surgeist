mod common;
#[path = "common/property_expectations.rs"]
mod property_expectations;

use common::CssParseReportTestExt;
use surgeist_css::*;

#[test]
fn direct_color_wrappers_keep_system_named_global_and_substitution_branches_distinct() {
    let report = parse_style_attribute(concat!(
        "color: ActiveBorder; ",
        "background-color: red; ",
        "outline-color: inherit; ",
        "text-decoration-color: var(--decoration)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::Color(deprecated) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected color wrapper");
    };
    assert_eq!(
        deprecated.value().system(),
        Some(CssSystemColor::ActiveBorder)
    );

    let CssKnownPropertyValueRef::BackgroundColor(named) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected background-color wrapper");
    };
    assert_eq!(named.value().named().unwrap().name(), "red");

    assert!(matches!(
        report.syntax()[2].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));
    assert!(matches!(
        report.syntax()[3].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
}

#[test]
fn core_font_wrappers_keep_current_global_and_substitution_branches_distinct() {
    let report = parse_style_attribute(concat!(
        "font-size: 16px; line-height: normal; ",
        "font-family: Arial, serif; font: italic 16px/normal Arial; ",
        "font-feature-settings: \"kern\"; ",
        "font-feature-settings: var(--features); font: inherit",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    for index in 0..5 {
        let value = report.syntax()[index]
            .known()
            .unwrap()
            .property_value()
            .unwrap();
        match value {
            CssKnownPropertyValueRef::FontSize(value) => {
                let _ = value.size();
            }
            CssKnownPropertyValueRef::LineHeight(value) => {
                let _ = value.line_height();
            }
            CssKnownPropertyValueRef::FontFamily(value) => {
                assert_eq!(value.families().families()[0].as_str(), "Arial");
                assert_eq!(
                    value.families().families()[1].generic_family(),
                    Some(surgeist_css::CssGenericFontFamily::Serif)
                );
            }
            CssKnownPropertyValueRef::Font(value) => {
                let surgeist_css::CssFontValue::Explicit(font) = value.font() else {
                    panic!("expected explicit font");
                };
                assert_eq!(font.families().families()[0].as_str(), "Arial");
            }
            CssKnownPropertyValueRef::FontFeatureSettings(value) => {
                assert!(matches!(
                    value.settings(),
                    surgeist_css::CssAuthoredFontFeatureSettings::Features(_)
                ));
            }
            _ => panic!("expected core font wrapper"),
        }
    }
    assert!(matches!(
        report.syntax()[5].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
    assert!(matches!(
        report.syntax()[6].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));
}

#[test]
fn font_variant_wrappers_keep_current_global_and_substitution_branches_distinct() {
    let report = parse_style_attribute(concat!(
        "font-variant: small-caps; font-variant: all-small-caps; ",
        "font-variant-caps: petite-caps; font-variant-east-asian: ruby; ",
        "font-variant-ligatures: none; font-variant-numeric: ordinal; ",
        "font-variant-position: sub; font-variant-caps: inherit; ",
        "font-variant-numeric: var(--numeric)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::FontVariant(small_caps) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-variant");
    };
    let CssFontVariantValue::Values(small_caps_values) = small_caps.variant() else {
        panic!("expected small-caps shorthand value")
    };
    assert_eq!(
        small_caps_values.caps(),
        Some(CssFontVariantCaps::SmallCaps)
    );
    assert!(small_caps_values.ligatures().is_none());
    assert!(small_caps_values.alternates().is_none());
    assert!(small_caps_values.numeric().is_none());
    assert!(small_caps_values.east_asian().is_none());
    assert!(small_caps_values.position().is_none());
    assert!(small_caps_values.emoji().is_none());
    let CssKnownPropertyValueRef::FontVariant(current) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-variant");
    };
    let CssFontVariantValue::Values(current_values) = current.variant() else {
        panic!("expected all-small-caps shorthand value")
    };
    assert_eq!(
        current_values.caps(),
        Some(CssFontVariantCaps::AllSmallCaps)
    );

    assert!(matches!(
        report.syntax()[2].known().unwrap().property_value(),
        Some(CssKnownPropertyValueRef::FontVariantCaps(value))
            if value.as_css() == "petite-caps"
    ));
    assert!(matches!(
        report.syntax()[3].known().unwrap().property_value(),
        Some(CssKnownPropertyValueRef::FontVariantEastAsian(value))
            if value.as_css() == "ruby"
    ));
    assert!(matches!(
        report.syntax()[4].known().unwrap().property_value(),
        Some(CssKnownPropertyValueRef::FontVariantLigatures(value))
            if value.as_css() == "none"
    ));
    assert!(matches!(
        report.syntax()[5].known().unwrap().property_value(),
        Some(CssKnownPropertyValueRef::FontVariantNumeric(value))
            if value.as_css() == "ordinal"
    ));
    assert!(matches!(
        report.syntax()[6].known().unwrap().property_value(),
        Some(CssKnownPropertyValueRef::FontVariantPosition(value))
            if value.as_css() == "sub"
    ));
    assert!(matches!(
        report.syntax()[7].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));
    assert!(matches!(
        report.syntax()[8].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
}

#[test]
fn direct_color_wrappers_expose_perceptual_and_predefined_channels() {
    let report = parse_style_attribute(concat!(
        "color: lab(calc(40% + 10%) 20 30); ",
        "background-color: color(xyz none calc(2 + 1) -3 / 120%)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::Color(lab) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected color wrapper");
    };
    let lab = lab.value().lab_value().expect("lab color");
    assert!(matches!(
        lab.lightness(),
        CssColorComponent::PercentageCalculation(_)
    ));
    let CssColorComponent::Number(a) = lab.a() else {
        panic!("expected lab a number");
    };
    assert_eq!(a.numeric().representation(), "20");
    let CssColorComponent::Number(b) = lab.b() else {
        panic!("expected lab b number");
    };
    assert_eq!(b.numeric().representation(), "30");
    assert!(lab.alpha().is_none());

    let CssKnownPropertyValueRef::BackgroundColor(predefined) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected background-color wrapper");
    };
    let predefined = predefined
        .value()
        .predefined_value()
        .expect("predefined color");
    assert_eq!(predefined.color_space(), CssPredefinedColorSpace::XyzD65);
    assert!(matches!(
        predefined.channels(),
        [
            CssColorComponent::None,
            CssColorComponent::NumberCalculation(_),
            CssColorComponent::Number(_),
        ]
    ));
    let CssColorComponent::Number(third) = &predefined.channels()[2] else {
        panic!("expected third channel number");
    };
    assert_eq!(third.numeric().representation(), "-3");
    let Some(CssColorComponent::Percentage(alpha)) = predefined.alpha() else {
        panic!("expected percentage alpha");
    };
    assert_eq!(alpha.numeric().representation(), "120");
}

#[test]
fn direct_color_wrappers_preserve_typed_relative_channels() {
    let report = parse_style_attribute(concat!(
        "color: rgb(from red r g b); ",
        "background-color: color(from red xyz x y z / alpha)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::Color(color) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected color wrapper");
    };
    let relative = color.value().relative_value().expect("relative rgb color");
    assert_eq!(relative.function(), &CssRelativeColorFunction::Rgb);
    assert_eq!(relative.environment(), CssRelativeColorEnvironment::Rgb);
    assert_eq!(relative.source().named().unwrap().name(), "red");
    assert_eq!(relative.alpha(), None);
    for (channel, expected) in relative.channels().iter().zip([
        CssRelativeColorChannel::R,
        CssRelativeColorChannel::G,
        CssRelativeColorChannel::B,
    ]) {
        assert_eq!(
            channel.value(),
            &CssRelativeColorExpressionValue::Channel(expected)
        );
    }

    let CssKnownPropertyValueRef::BackgroundColor(background) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected background-color wrapper");
    };
    let relative = background
        .value()
        .relative_value()
        .expect("relative xyz color");
    assert_eq!(
        relative.function(),
        &CssRelativeColorFunction::Color(CssPredefinedColorSpace::XyzD65)
    );
    assert_eq!(
        relative.environment(),
        CssRelativeColorEnvironment::Xyz(CssPredefinedColorSpace::XyzD65)
    );
    assert_eq!(relative.source().named().unwrap().name(), "red");
    for (channel, expected) in relative.channels().iter().zip([
        CssRelativeColorChannel::X,
        CssRelativeColorChannel::Y,
        CssRelativeColorChannel::Z,
    ]) {
        assert_eq!(
            channel.value(),
            &CssRelativeColorExpressionValue::Channel(expected)
        );
    }
    assert_eq!(
        relative.alpha().unwrap().value(),
        &CssRelativeColorExpressionValue::Channel(CssRelativeColorChannel::Alpha)
    );
}

#[test]
fn property_schema_dispatch_exposes_property_specific_authored_wrappers() {
    for case in property_expectations::CASES {
        let (Some(dispatch), Some(_)) = (&case.dispatch, case.wrapper) else {
            continue;
        };
        let (name, authored) = (case.name, dispatch.ordinary);
        let source = format!("{}: {}", name, authored);
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let declaration = report.syntax()[0].known().expect("known declaration");
        let value = declaration
            .property_value()
            .expect("ordinary value has a property wrapper");
        assert_eq!(declaration.property().canonical_name(), name);
        case.assert_wrapper(declaration, value, authored);
    }
}

#[test]
fn explicit_property_dispatch_cases_preserve_ordinary_and_important_behavior() {
    for case in property_expectations::CASES {
        let Some(dispatch) = &case.dispatch else {
            continue;
        };
        let name = case.name;
        assert!(
            !["inherit", "initial", "unset", "revert", "revert-layer"]
                .contains(&dispatch.ordinary.trim().to_ascii_lowercase().as_str()),
            "`{name}` dispatch vector must not use a CSS-wide global keyword"
        );
        assert!(
            !dispatch.ordinary.to_ascii_lowercase().contains("var("),
            "`{name}` dispatch vector must not use substitution-dependent parsing"
        );

        let source = format!(
            ".test {{ {}: {}; }}",
            name.to_ascii_uppercase(),
            dispatch.ordinary
        );

        let important_value = dispatch.important;
        let important_source = format!(
            ".test {{ {}: {} !important; }}",
            name.to_ascii_uppercase(),
            important_value
        );
        let important_sheet = parse_sheet(&important_source).unwrap_or_else(|error| {
            panic!("`{name}: {important_value} !important` must parse: {error}")
        });
        let [CssRule::Style(important_rule)] = important_sheet.rules() else {
            panic!("`{name}` importance path should produce one style rule");
        };
        let [important_declaration] = important_rule.declarations().as_slice() else {
            panic!("`{name}` importance path should produce one declaration");
        };
        assert_eq!(important_declaration.importance(), CssImportance::Important);
        assert_eq!(
            important_declaration.known().map(|known| known.property()),
            Some(case.property),
            "`{name}` importance path diverged from schema identity",
        );

        if name == "all" {
            assert_eq!(
                important_declaration
                    .known()
                    .and_then(|known| known.global()),
                Some(surgeist_css::CssGlobalKeyword::Inherit),
            );
            let error = parse_sheet(&source).expect_err("`all` ordinary value must be rejected");
            assert_eq!(error.code(), CssErrorCode::InvalidPropertyValue);
            let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
                panic!(
                    "`all` returned the wrong structured error: {:?}",
                    error.kind()
                );
            };
            assert_eq!(detail.property(), CssKnownProperty::All);
            continue;
        }

        let sheet = parse_sheet(&source).unwrap_or_else(|error| {
            panic!(
                "`{name}: {}` should parse through generated dispatch: {error}",
                dispatch.ordinary
            )
        });
        let [CssRule::Style(rule)] = sheet.rules() else {
            panic!("`{name}` should produce one style rule");
        };
        let [declaration] = rule.declarations().as_slice() else {
            panic!("`{name}` should produce one declaration");
        };
        assert_eq!(
            declaration.known().map(|known| known.property()),
            Some(case.property),
            "`{name}` parser identity diverged from schema lookup",
        );

        let declaration = declaration.known().expect("known ordinary declaration");
        let value = declaration
            .property_value()
            .expect("ordinary value has a property wrapper");
        case.assert_wrapper(declaration, value, dispatch.ordinary);

        let important_declaration = important_declaration
            .known()
            .expect("known important declaration");
        let important_value = important_declaration
            .property_value()
            .expect("important ordinary value has a property wrapper");
        case.assert_wrapper(important_declaration, important_value, dispatch.ordinary);
    }
}

#[test]
fn layered_position_wrappers_keep_current_global_and_substitution_branches_distinct() {
    let report = parse_style_attribute(concat!(
        "background-position: left 10px top, center; ",
        "mask-position: right bottom; ",
        "background-position: inherit; ",
        "mask-position: var(--position)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let background = report.syntax()[0].known().expect("background declaration");
    let CssKnownPropertyValueRef::BackgroundPosition(value) = background
        .property_value()
        .expect("ordinary background position")
    else {
        panic!("expected background-position value");
    };
    assert_eq!(value.positions().positions().len(), 2);
    assert!(background.global().is_none());
    assert!(background.substitution_dependent().is_none());

    let mask = report.syntax()[1].known().expect("mask declaration");
    let CssKnownPropertyValueRef::MaskPosition(value) =
        mask.property_value().expect("ordinary mask position")
    else {
        panic!("expected mask-position value");
    };
    assert_eq!(value.positions().positions().len(), 1);
    assert!(mask.global().is_none());
    assert!(mask.substitution_dependent().is_none());

    let global = report.syntax()[2].known().expect("global declaration");
    assert!(global.property_value().is_none());
    assert!(global.global().is_some());
    assert!(global.substitution_dependent().is_none());

    let substitution = report.syntax()[3]
        .known()
        .expect("substitution-dependent declaration");
    assert!(substitution.property_value().is_none());
    assert!(substitution.global().is_none());
    assert!(substitution.substitution_dependent().is_some());
}

#[test]
fn object_and_transform_origin_wrappers_keep_ordinary_global_and_substitution_branches_distinct() {
    let report = parse_style_attribute(concat!(
        "object-position: left 10px bottom 20%; ",
        "transform-origin: left top 5px; ",
        "object-position: inherit; ",
        "object-position: var(--position); ",
        "transform-origin: var(--origin)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let object = report.syntax()[0].known().expect("object declaration");
    assert_eq!(object.property(), CssKnownProperty::ObjectPosition);
    let CssKnownPropertyValueRef::ObjectPosition(value) =
        object.property_value().expect("ordinary object position")
    else {
        panic!("expected object-position value");
    };
    assert!(matches!(
        value.position().horizontal(),
        surgeist_css::CssHorizontalPosition::LeftOffset(_)
    ));
    assert!(object.global().is_none());
    assert!(object.substitution_dependent().is_none());

    let transform = report.syntax()[1]
        .known()
        .expect("transform-origin declaration");
    let CssKnownPropertyValueRef::TransformOrigin(value) = transform
        .property_value()
        .expect("ordinary transform origin")
    else {
        panic!("expected transform-origin value");
    };
    assert!(value.origin().z().is_some());
    assert!(transform.global().is_none());
    assert!(transform.substitution_dependent().is_none());

    let global = report.syntax()[2].known().expect("global declaration");
    assert_eq!(global.property(), CssKnownProperty::ObjectPosition);
    assert!(global.property_value().is_none());
    assert_eq!(
        global.global(),
        Some(surgeist_css::CssGlobalKeyword::Inherit)
    );
    assert!(global.substitution_dependent().is_none());

    let object_substitution = report.syntax()[3]
        .known()
        .expect("object substitution-dependent declaration");
    assert_eq!(
        object_substitution.property(),
        CssKnownProperty::ObjectPosition
    );
    assert!(object_substitution.property_value().is_none());
    assert!(object_substitution.global().is_none());
    assert_eq!(
        object_substitution
            .substitution_dependent()
            .expect("object-position substitution branch")
            .as_css(),
        "var(--position)",
    );

    let substitution = report.syntax()[4]
        .known()
        .expect("substitution-dependent declaration");
    assert_eq!(substitution.property(), CssKnownProperty::TransformOrigin);
    assert!(substitution.property_value().is_none());
    assert!(substitution.global().is_none());
    assert!(substitution.substitution_dependent().is_some());
}

#[test]
fn timing_wrappers_expose_exact_property_specific_accessors() {
    use surgeist_css::{CssAnimationIterationCount, CssTimeUnit, CssTransitionProperty};
    let report = parse_style_attribute(concat!(
        "transition-duration: calc(1s + 2s); transition-delay: -1s; ",
        "animation-duration: calc(3ms * 2); animation-delay: -4ms; ",
        "animation-iteration-count: calc(1 + 2); ",
        "transition: opacity 1s -2s; animation: fade 3s -4s 2"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    for declaration in report.syntax().as_slice() {
        match declaration.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::TransitionDuration(value) => {
                assert!(
                    ({
                        let values = value.durations().values();
                        if values.len() == 1 {
                            values[0].time().calculation()
                        } else {
                            None
                        }
                    })
                    .is_some()
                );
            }
            CssKnownPropertyValueRef::TransitionDelay(value) => {
                assert!(
                    matches!({ let values = value.delays().values(); if values.len() == 1 { values[0].literal() } else { None } }, Some(t)
                    if t.numeric().representation() == "-1" && t.unit() == CssTimeUnit::Seconds)
                );
            }
            CssKnownPropertyValueRef::AnimationDuration(value) => {
                assert!(
                    ({
                        let values = value.durations().values();
                        if values.len() == 1 {
                            values[0].time().calculation()
                        } else {
                            None
                        }
                    })
                    .is_some()
                );
            }
            CssKnownPropertyValueRef::AnimationDelay(value) => {
                assert!(
                    matches!({ let values = value.delays().values(); if values.len() == 1 { values[0].literal() } else { None } }, Some(t)
                    if t.numeric().representation() == "-4" && t.unit() == CssTimeUnit::Milliseconds)
                );
            }
            CssKnownPropertyValueRef::AnimationIterationCount(value) => {
                assert!(matches!(
                    value.iteration_counts().values(),
                    [CssAnimationIterationCount::Number(number)] if number.calculation().is_some()
                ));
            }
            CssKnownPropertyValueRef::Transition(value) => {
                let [item] = value.transitions().values() else {
                    panic!("one transition");
                };
                assert!(
                    matches!(item.property(), Some(CssTransitionProperty::Custom(name)) if name.as_str() == "opacity")
                );
                assert!(
                    matches!(item.duration().and_then(|value| value.time().literal()), Some(t) if t.numeric().representation() == "1")
                );
                assert!(
                    matches!(item.delay().and_then(|value| value.literal()), Some(t) if t.numeric().representation() == "-2")
                );
            }
            CssKnownPropertyValueRef::Animation(value) => {
                let [item] = value.animations().values() else {
                    panic!("one animation");
                };
                assert!(
                    matches!(item.duration().and_then(|value| value.time().literal()), Some(t) if t.numeric().representation() == "3")
                );
                assert!(
                    matches!(item.delay().and_then(|value| value.literal()), Some(t) if t.numeric().representation() == "-4")
                );
                assert!(
                    matches!(item.iteration_count(), Some(CssAnimationIterationCount::Number(n)) if n.serialize_specified().unwrap() == "2")
                );
            }
            _ => panic!("unexpected property wrapper"),
        }
    }
}

#[test]
fn typed_length_calculations_are_accepted_by_the_exact_current_consumer_set() {
    for (property, value) in [
        ("width", "calc((1px + 2%) * 3)"),
        ("height", "calc((1px + 2%) * 3)"),
        ("min-width", "calc((1px + 2%) * 3)"),
        ("min-height", "calc((1px + 2%) * 3)"),
        ("max-width", "calc((1px + 2%) * 3)"),
        ("max-height", "calc((1px + 2%) * 3)"),
        ("flex-basis", "calc((1px + 2%) * 3)"),
        ("gap", "calc((1px + 2%) * 3)"),
        ("row-gap", "calc((1px + 2%) * 3)"),
        ("column-gap", "calc((1px + 2%) * 3)"),
        ("grid-template-rows", "calc((1px + 2%) * 3)"),
        ("grid-template-columns", "calc((1px + 2%) * 3)"),
        ("grid-auto-rows", "calc((1px + 2%) * 3)"),
        ("grid-auto-columns", "calc((1px + 2%) * 3)"),
        ("grid-template", "calc((1px + 2%) * 3) / 1fr"),
        ("grid", "calc((1px + 2%) * 3) / 1fr"),
        ("font-size", "calc((1px + 2%) * 3)"),
        ("line-height", "calc((1px + 2%) * 3)"),
        (
            "font",
            "italic calc((10px + 2%) * 2)/calc((1px + 2%) * 3) serif",
        ),
        ("text-indent", "calc((1px + 2%) * 3) hanging"),
        ("vertical-align", "calc((1px + 2%) * 3)"),
        ("letter-spacing", "calc((1px + 2em) * 3)"),
        ("text-decoration", "underline calc((1px + 2%) * 3)"),
        ("text-decoration-thickness", "calc((1px + 2%) * 3)"),
        ("inset", "calc((1px + 2%) * 3)"),
        ("top", "calc((1px + 2%) * 3)"),
        ("right", "calc((1px + 2%) * 3)"),
        ("bottom", "calc((1px + 2%) * 3)"),
        ("left", "calc((1px + 2%) * 3)"),
        ("margin", "calc((1px + 2%) * 3)"),
        ("margin-top", "calc((1px + 2%) * 3)"),
        ("margin-right", "calc((1px + 2%) * 3)"),
        ("margin-bottom", "calc((1px + 2%) * 3)"),
        ("margin-left", "calc((1px + 2%) * 3)"),
        ("padding", "calc((-1px + 2%) * 3)"),
        ("padding-top", "calc((-1px + 2%) * 3)"),
        ("padding-right", "calc((-1px + 2%) * 3)"),
        ("padding-bottom", "calc((-1px + 2%) * 3)"),
        ("padding-left", "calc((-1px + 2%) * 3)"),
        ("border", "solid calc((-1px + 2px) * 3)"),
        ("border-top", "solid calc((-1px + 2px) * 3)"),
        ("border-right", "solid calc((-1px + 2px) * 3)"),
        ("border-bottom", "solid calc((-1px + 2px) * 3)"),
        ("border-left", "solid calc((-1px + 2px) * 3)"),
        ("border-width", "calc((-1px + 2px) * 3)"),
        ("border-top-width", "calc((-1px + 2px) * 3)"),
        ("border-right-width", "calc((-1px + 2px) * 3)"),
        ("border-bottom-width", "calc((-1px + 2px) * 3)"),
        ("border-left-width", "calc((-1px + 2px) * 3)"),
        ("border-radius", "calc((-1px + 2%) * 3)"),
        ("border-top-left-radius", "calc((-1px + 2%) * 3)"),
        ("border-top-right-radius", "calc((-1px + 2%) * 3)"),
        ("border-bottom-right-radius", "calc((-1px + 2%) * 3)"),
        ("border-bottom-left-radius", "calc((-1px + 2%) * 3)"),
        ("box-shadow", "calc((1px + 2px) * 3) 2px"),
        ("outline", "solid calc((-1px + 2px) * 3)"),
        ("outline-width", "calc((-1px + 2px) * 3)"),
        ("background-position", "calc((1px + 2%) * 3) top"),
        ("background-size", "calc((-1px + 2%) * 3) auto"),
        ("mask-position", "calc((1px + 2%) * 3) top"),
        ("mask-size", "calc((-1px + 2%) * 3) auto"),
        ("transform-origin", "calc((1px + 2%) * 3) top"),
        ("translate", "calc((1px + 2%) * 3)"),
        ("transform", "translate(calc((1px + 2%) * 3))"),
    ] {
        let source = format!("{property}: {value}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::from_name(property).unwrap(),
            "{source}",
        );
    }

    let source = "clip-path: polygon(, calc((1px + 2%) * 3) 0px, 1px 1px)";
    let report = parse_style_attribute(source);
    assert!(
        !report.is_clean(),
        "later function grammar changed: {source}"
    );
    assert!(report.syntax().is_empty(), "{source}");
}

#[test]
fn transform_wrapper_keeps_value_global_and_substitution_branches_distinct() {
    let report = parse_style_attribute(concat!(
        "transform: translate(calc((1px + 2%) * 3), 4px) rotate(0); ",
        "transform: none; transform: inherit; transform: var(--transform)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let ordinary = report.syntax()[0]
        .known()
        .expect("ordinary transform declaration");
    let CssKnownPropertyValueRef::Transform(value) = ordinary
        .property_value()
        .expect("ordinary transform property value")
    else {
        panic!("expected transform property value");
    };
    let CssTransform::Functions(functions) = value.value() else {
        panic!("expected transform function list");
    };
    assert!(matches!(
        functions.functions()[0],
        CssTransformFunction::Translate(_)
    ));
    assert!(matches!(
        functions.functions()[1],
        CssTransformFunction::Rotate(_)
    ));
    assert!(ordinary.global().is_none());
    assert!(ordinary.substitution_dependent().is_none());

    let none = report.syntax()[1].known().expect("none declaration");
    let CssKnownPropertyValueRef::Transform(value) =
        none.property_value().expect("ordinary none value")
    else {
        panic!("expected transform none value");
    };
    assert!(matches!(value.value(), CssTransform::None));

    let global = report.syntax()[2].known().expect("global declaration");
    assert!(global.property_value().is_none());
    assert!(global.global().is_some());
    assert!(global.substitution_dependent().is_none());

    let substitution = report.syntax()[3]
        .known()
        .expect("substitution-dependent declaration");
    assert!(substitution.property_value().is_none());
    assert!(substitution.global().is_none());
    assert!(substitution.substitution_dependent().is_some());
}

#[test]
fn timing_function_wrappers_keep_typed_global_and_substitution_branches_distinct() {
    let report = parse_style_attribute(concat!(
        "transition-timing-function: cubic-bezier(0.1, -2, 0.9, 3); ",
        "animation-timing-function: steps(2, jump-none); ",
        "transition-timing-function: inherit; ",
        "animation-timing-function: var(--timing)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let transition = report.syntax()[0]
        .known()
        .expect("ordinary transition timing");
    let CssKnownPropertyValueRef::TransitionTimingFunction(value) = transition
        .property_value()
        .expect("ordinary transition timing value")
    else {
        panic!("expected transition timing wrapper");
    };
    assert!(matches!(
        value.timing_functions().values(),
        [CssEasing::CubicBezier(_)]
    ));

    let animation = report.syntax()[1]
        .known()
        .expect("ordinary animation timing");
    let CssKnownPropertyValueRef::AnimationTimingFunction(value) = animation
        .property_value()
        .expect("ordinary animation timing value")
    else {
        panic!("expected animation timing wrapper");
    };
    assert!(matches!(
        value.timing_functions().values(),
        [CssEasing::Steps(_)]
    ));

    let global = report.syntax()[2]
        .known()
        .expect("global timing declaration");
    assert!(global.property_value().is_none());
    assert!(global.global().is_some());
    assert!(global.substitution_dependent().is_none());

    let substitution = report.syntax()[3]
        .known()
        .expect("substitution-dependent timing declaration");
    assert!(substitution.property_value().is_none());
    assert!(substitution.global().is_none());
    assert!(substitution.substitution_dependent().is_some());
}

#[test]
fn shadow_and_filter_wrappers_expose_ordered_typed_values() {
    let report = parse_style_attribute(concat!(
        "box-shadow: inset 1px 2px red; ",
        "filter: blur(4px) opacity(50%); ",
        "backdrop-filter: brightness()"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::BoxShadow(shadow) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected box-shadow wrapper");
    };
    let CssBoxShadow::Shadows(shadows) = shadow.value() else {
        panic!("expected box-shadow list");
    };
    let [shadow] = shadows.shadows() else {
        panic!("expected one box shadow");
    };
    assert!(shadow.inset());
    assert!(exact_dimension(
        shadow.offset_x().literal_component(),
        "1",
        "px"
    ));
    assert!(exact_dimension(
        shadow.offset_y().literal_component(),
        "2",
        "px"
    ));
    assert!(shadow.blur_radius().is_none());
    assert!(shadow.spread_radius().is_none());
    assert_eq!(shadow.color().unwrap().named().unwrap().name(), "red");

    let CssKnownPropertyValueRef::Filter(filter) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected filter wrapper");
    };
    let CssFilter::Functions(functions) = filter.value() else {
        panic!("expected filter functions");
    };
    let [
        CssFilterFunction::Blur(blur),
        CssFilterFunction::Opacity(amount),
    ] = functions.functions()
    else {
        panic!("expected ordered blur and opacity functions");
    };
    assert!(exact_dimension(
        blur.length().literal_component(),
        "4",
        "px"
    ));
    assert!(matches!(
        amount,
        CssFilterAmount::Percentage(value)
            if exact_percentage(value.literal_component(), "50")
    ));

    let CssKnownPropertyValueRef::BackdropFilter(backdrop) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected backdrop-filter wrapper");
    };
    let CssFilter::Functions(functions) = backdrop.value() else {
        panic!("expected backdrop filter functions");
    };
    assert!(matches!(
        functions.functions(),
        [CssFilterFunction::Brightness(CssFilterAmount::Default)]
    ));
}

fn assert_srgb_mix_with_lab_and_blue(color: &CssColor) {
    let mix = color.color_mix_value().expect("color-mix value");
    let interpolation = mix.interpolation().expect("explicit interpolation");
    let method = interpolation
        .predefined()
        .expect("predefined interpolation");
    assert_eq!(
        method.space(),
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb)
    );
    assert_eq!(method.hue(), None);
    let [lab, blue] = mix.components() else {
        panic!("expected two ordered color-mix components");
    };
    assert!(lab.weight().is_none());
    let lab = lab.color().lab_value().expect("lab first component");
    assert!(matches!(
        lab.lightness(),
        CssColorComponent::PercentageCalculation(_)
    ));
    let CssColorComponent::Number(a) = lab.a() else {
        panic!("expected lab a number");
    };
    assert_eq!(a.numeric().representation(), "20");
    let CssColorComponent::Number(b) = lab.b() else {
        panic!("expected lab b number");
    };
    assert_eq!(b.numeric().representation(), "30");
    assert!(lab.alpha().is_none());
    assert!(blue.weight().is_none());
    assert_eq!(blue.color().named().unwrap().name(), "blue");
}

#[test]
fn every_aggregate_color_consumer_retains_typed_color_mix() {
    let report = parse_style_attribute(concat!(
        "border: 1px solid color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue); ",
        "border-top: solid color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue); ",
        "border-right: solid color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue); ",
        "border-bottom: solid color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue); ",
        "border-left: solid color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue); ",
        "text-decoration: underline color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue); ",
        "outline: solid color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue); ",
        "box-shadow: 1px 2px color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue); ",
        "filter: drop-shadow(1px 2px color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue)); ",
        "backdrop-filter: drop-shadow(1px 2px color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue))",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    for declaration in &report.syntax()[..5] {
        let known = declaration.known().unwrap();
        let border = match known.property_value().unwrap() {
            CssKnownPropertyValueRef::Border(value) => value.value(),
            CssKnownPropertyValueRef::BorderTop(value) => value.value(),
            CssKnownPropertyValueRef::BorderRight(value) => value.value(),
            CssKnownPropertyValueRef::BorderBottom(value) => value.value(),
            CssKnownPropertyValueRef::BorderLeft(value) => value.value(),
            _ => panic!("expected border shorthand wrapper"),
        };
        assert_eq!(border.style(), Some(CssBorderStyle::Solid));
        assert_srgb_mix_with_lab_and_blue(border.color().expect("border color"));
    }
    let border = match report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    {
        CssKnownPropertyValueRef::Border(value) => value.value(),
        _ => panic!("expected border shorthand"),
    };
    assert_eq!(
        border.width().unwrap().serialize_specified().unwrap(),
        "1px"
    );
    for declaration in &report.syntax()[1..5] {
        let border = match declaration.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::BorderTop(value) => value.value(),
            CssKnownPropertyValueRef::BorderRight(value) => value.value(),
            CssKnownPropertyValueRef::BorderBottom(value) => value.value(),
            CssKnownPropertyValueRef::BorderLeft(value) => value.value(),
            _ => panic!("expected side border"),
        };
        assert!(border.width().is_none());
    }

    let CssKnownPropertyValueRef::TextDecoration(decoration) = report.syntax()[5]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected text-decoration wrapper");
    };
    assert_srgb_mix_with_lab_and_blue(decoration.value().color().expect("decoration color"));
    assert!(matches!(
        decoration.value().line().unwrap().components(),
        [CssTextDecorationLineComponent::Underline]
    ));
    assert!(decoration.value().style().is_none());
    assert!(decoration.value().thickness().is_none());

    let CssKnownPropertyValueRef::Outline(outline) = report.syntax()[6]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected outline wrapper");
    };
    assert_srgb_mix_with_lab_and_blue(
        outline
            .value()
            .color()
            .expect("outline color")
            .color()
            .expect("explicit color"),
    );
    assert_eq!(outline.value().style(), Some(CssOutlineStyle::Solid));
    assert!(outline.value().width().is_none());

    let CssKnownPropertyValueRef::BoxShadow(shadow) = report.syntax()[7]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected box-shadow wrapper");
    };
    let CssBoxShadow::Shadows(shadows) = shadow.value() else {
        panic!("expected box-shadow list");
    };
    let [shadow] = shadows.shadows() else {
        panic!("expected one box shadow");
    };
    assert_srgb_mix_with_lab_and_blue(shadow.color().expect("box-shadow color"));
    assert!(!shadow.inset());
    assert!(exact_dimension(
        shadow.offset_x().literal_component(),
        "1",
        "px"
    ));
    assert!(exact_dimension(
        shadow.offset_y().literal_component(),
        "2",
        "px"
    ));
    assert!(shadow.blur_radius().is_none());
    assert!(shadow.spread_radius().is_none());

    for declaration in &report.syntax()[8..] {
        let value = match declaration.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::Filter(value) => value.value(),
            CssKnownPropertyValueRef::BackdropFilter(value) => value.value(),
            _ => panic!("expected filter wrapper"),
        };
        let CssFilter::Functions(functions) = value else {
            panic!("expected filter function list");
        };
        let [CssFilterFunction::DropShadow(shadow)] = functions.functions() else {
            panic!("expected one drop-shadow");
        };
        assert_srgb_mix_with_lab_and_blue(shadow.color().expect("drop-shadow color"));
        assert!(exact_dimension(
            shadow.offset_x().literal_component(),
            "1",
            "px"
        ));
        assert!(exact_dimension(
            shadow.offset_y().literal_component(),
            "2",
            "px"
        ));
        assert!(shadow.standard_deviation().is_none());
    }
}

#[test]
fn clip_path_wrapper_separates_authored_global_and_substitution_values() {
    let report = parse_style_attribute(concat!(
        "clip-path: inset(1px round 2px); ",
        "clip-path: circle(50% at center); ",
        "clip-path: inherit; ",
        "clip-path: var(--clip)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::ClipPath(inset) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected inset clip-path wrapper");
    };
    assert!(matches!(
        inset.value(),
        CssClipPath::BasicShape(shape) if matches!(shape.shape(), CssBasicShape::Inset(_))
    ));

    let CssKnownPropertyValueRef::ClipPath(circle) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected circle clip-path wrapper");
    };
    let CssClipPath::BasicShape(clip_shape) = circle.value() else {
        panic!("typed percentage circle");
    };
    let CssBasicShape::Circle(shape) = clip_shape.shape() else {
        panic!("typed percentage circle");
    };
    assert!(
        matches!(shape.radius(), CssCircleRadius::LengthPercentage(radius)
        if exact_percentage(radius.literal_component(), "50"))
    );
    assert!(shape.position().is_some());

    assert!(matches!(
        report.syntax()[2].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));
    assert!(matches!(
        report.syntax()[3].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
}

#[test]
fn deferred_basic_shape_math_drops_only_the_invalid_declaration() {
    for shape in [
        "circle(calc((1px + 2%)) 1px * 2)",
        "ellipse(calc((1px + 2%)) 1px * 2)",
        "inset(calc((1px + 2%)) 1px * 2)",
    ] {
        let invalid = format!("clip-path: {shape};");
        let source = format!("{invalid} color: red");
        let report = parse_style_attribute(&source);

        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one recovered declaration");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropDeclaration,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            0,
            "{source}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            invalid.len(),
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-value detail");
        };
        assert_eq!(detail.property(), CssKnownProperty::ClipPath, "{source}");

        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("recovered basic-shape math must fail strict validation");
            assert_eq!(failure.diagnostics(), report.diagnostics(), "{source}");
        }
    }
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
fn exact_percentage(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Percentage(number))) if number.representation() == representation)
}

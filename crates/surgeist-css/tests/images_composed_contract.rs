#![forbid(unsafe_code)]
//! CSS Images 3 §§3.1.1, 3.2.1, 3.3, 3.4.1 and 7.
//! https://www.w3.org/TR/2023/CRD-css-images-3-20231218/
//! Values 4 §4.5 and selected Color 5 §7 retain URL and image-pair identity.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#urls
//! https://www.w3.org/TR/2026/WD-css-color-5-20260908/#light-dark

use surgeist_css::*;

fn fronts(property: CssKnownProperty, text: &str) -> Vec<(CssDeclaration, String)> {
    let css = format!("/*😀*/{}:{text}!important", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(validate_style_attribute(&css).unwrap(), *report.syntax());
    let [parsed] = report.syntax().as_slice() else {
        panic!("one image declaration")
    };
    let mut declarations = vec![(parsed.clone(), css)];
    let components = parse_component_values(text).unwrap();
    let original = components.clone();
    for grammar in [false, true] {
        let declaration = if grammar {
            parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Important,
            )
        } else {
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Important,
            )
        }
        .unwrap();
        assert_eq!(declaration.value_components(), &original);
        declarations.push((declaration, text.to_owned()));
    }
    assert_eq!(components, original);
    for (declaration, _) in &declarations {
        assert_eq!(declaration.known().unwrap().property(), property);
        assert_eq!(declaration.importance(), CssImportance::Important);
    }
    declarations
}

fn image(declaration: &CssDeclaration) -> &CssImageValue {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::BackgroundImage(value) => {
            let [image] = value.images().images() else {
                panic!("one background image")
            };
            image
        }
        CssKnownPropertyValueRef::ListStyleImage(value) => value.value(),
        CssKnownPropertyValueRef::BorderImageSource(value) => value.source(),
        CssKnownPropertyValueRef::Content(value) => {
            let CssContentValue::Generated(value) = value.value() else {
                panic!("generated replacement image")
            };
            let CssGeneratedContentBodyRef::Replacement(value) = value.body() else {
                panic!("one content replacement image")
            };
            value.value()
        }
        _ => panic!("selected image consumer"),
    }
}

fn gradient(declaration: &CssDeclaration) -> &CssGradient {
    let CssImageValue::Gradient(value) = image(declaration) else {
        panic!("gradient image")
    };
    value
}

fn linear(value: &CssGradient) -> &CssLinearGradient {
    match value {
        CssGradient::Linear(value) | CssGradient::RepeatingLinear(value) => value,
        _ => panic!("linear family"),
    }
}

fn radial(value: &CssGradient) -> &CssRadialGradient {
    match value {
        CssGradient::Radial(value) | CssGradient::RepeatingRadial(value) => value,
        _ => panic!("radial family"),
    }
}

fn parsed_origin(origin: &CssValueOrigin, source: &str, token: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original parsed numeric token")
    };
    assert_eq!(origin.source().as_str(), source);
    let start = source.find(token).unwrap();
    for (position, offset) in [
        (origin.span().start(), start),
        (origin.span().end(), start + token.len()),
    ] {
        assert_eq!(position.byte_offset().value(), offset);
        assert_eq!(position.line().value(), 0);
        assert_eq!(
            position.column().value() as usize,
            source[..offset].encode_utf16().count(),
        );
    }
}

fn percentage(value: &CssSpecifiedLengthPercentage, source: &str, spelling: &str) {
    let component = value
        .literal_component()
        .expect("literal authored percentage");
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) = component.view() else {
        panic!("percentage branch")
    };
    assert_eq!(number.representation(), spelling.strip_suffix('%').unwrap());
    parsed_origin(component.origin(), source, spelling);
}

fn rejects_background(text: &str) {
    let prefix = "/*😀*/color:red;";
    let unit = format!("background-image:{text};");
    let source = format!("{prefix}{unit}color:blue!important");
    let report = parse_style_attribute(&source);
    let [first, last] = report.syntax().as_slice() else {
        panic!("only the two valid neighbors remain: {source}")
    };
    assert_eq!(first.to_specified_css().unwrap(), "color: red;");
    assert_eq!(last.to_specified_css().unwrap(), "color: blue !important;");
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejected image declaration: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        prefix.len()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        prefix.len() + unit.len(),
    );
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics(),
    );
    let values = parse_component_values(text).unwrap();
    let original = values.clone();
    for grammar in [false, true] {
        let result = if grammar {
            parse_property_value_for_grammar(
                CssKnownProperty::BackgroundImage.grammar(),
                values.clone(),
                CssImportance::Normal,
            )
        } else {
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
                values.clone(),
                CssImportance::Normal,
            )
        };
        assert!(result.is_err(), "whole invalid image: {text}");
    }
    assert_eq!(values, original);
}

#[test]
fn all_four_gradients_keep_signed_positions_hint_origins_and_function_identity() {
    for (function, family) in [
        ("linear-gradient", 0),
        ("repeating-linear-gradient", 1),
        ("radial-gradient", 2),
        ("repeating-radial-gradient", 3),
    ] {
        let text = format!("{function}(red -10%, 40%, blue 120%)");
        for (declaration, source) in fronts(CssKnownProperty::BackgroundImage, &text) {
            let value = gradient(&declaration);
            let stops = match (family, value) {
                (0, CssGradient::Linear(value)) => value.stops(),
                (1, CssGradient::RepeatingLinear(value)) => value.stops(),
                (2, CssGradient::Radial(value)) => value.stops(),
                (3, CssGradient::RepeatingRadial(value)) => value.stops(),
                _ => panic!("authored repeating/nonrepeating identity"),
            };
            let [
                CssColorStopListItem::Stop(first),
                CssColorStopListItem::Hint(hint),
                CssColorStopListItem::Stop(last),
            ] = stops.items()
            else {
                panic!("exactly two stops with one intervening hint")
            };
            assert_eq!(first.color().to_specified_css().unwrap(), "red");
            assert_eq!(last.color().to_specified_css().unwrap(), "blue");
            percentage(first.position().unwrap(), &source, "-10%");
            percentage(hint, &source, "40%");
            percentage(last.position().unwrap(), &source, "120%");
            assert_eq!(value.serialize_specified().unwrap(), text);
            let checked = CssImage::try_new(image(&declaration).clone()).unwrap();
            assert_eq!(checked.serialize_specified().unwrap(), text);
            assert_eq!(
                declaration.to_specified_css().unwrap(),
                format!("background-image: {text} !important;"),
            );
        }
    }
}

#[test]
fn reversed_corner_order_and_unitless_zero_have_the_same_linear_sibling_grammar() {
    for function in ["linear-gradient", "repeating-linear-gradient"] {
        for direction in ["to top right", "to right top"] {
            let text = format!("{function}({direction}, red, blue)");
            for (declaration, _) in fronts(CssKnownProperty::BackgroundImage, &text) {
                let Some(CssLinearGradientDirection::SideOrCorner(direction)) =
                    linear(gradient(&declaration)).direction()
                else {
                    panic!("keyword corner")
                };
                assert_eq!(
                    direction.horizontal(),
                    Some(CssHorizontalGradientSide::Right)
                );
                assert_eq!(direction.vertical(), Some(CssVerticalGradientSide::Top));
                assert_eq!(
                    gradient(&declaration).serialize_specified().unwrap(),
                    format!("{function}(to right top, red, blue)"),
                );
            }
        }
        let text = format!("{function}(0, red, blue)");
        for (declaration, source) in fronts(CssKnownProperty::BackgroundImage, &text) {
            let Some(CssLinearGradientDirection::Angle(CssAngleOrZero::Zero(zero))) =
                linear(gradient(&declaration)).direction()
            else {
                panic!("literal unitless zero")
            };
            assert_eq!(zero.numeric().representation(), "0");
            parsed_origin(zero.origin(), &source, "0");
        }
        for invalid in ["1", "to left right", "to top bottom", "to"] {
            rejects_background(&format!("{function}({invalid}, red, blue)"));
        }
    }
}

#[test]
fn radial_siblings_keep_unordered_shape_size_and_omitted_authored_defaults() {
    for function in ["radial-gradient", "repeating-radial-gradient"] {
        for prelude in ["5em circle at top left", "circle 5em at top left"] {
            let text = format!("{function}({prelude}, red, blue)");
            for (declaration, _) in fronts(CssKnownProperty::BackgroundImage, &text) {
                let value = radial(gradient(&declaration));
                assert_eq!(value.shape(), Some(CssRadialShape::Circle));
                assert!(matches!(value.size(), Some(CssRadialSize::Circle(_))));
                assert!(value.position().is_some());
                assert_eq!(
                    gradient(&declaration).serialize_specified().unwrap(),
                    format!("{function}(5em at left top, red, blue)"),
                );
            }
        }
        for prelude in ["ellipse 20% 30%", "20% 30% ellipse"] {
            let text = format!("{function}({prelude}, red, blue)");
            for (declaration, source) in fronts(CssKnownProperty::BackgroundImage, &text) {
                let value = radial(gradient(&declaration));
                assert_eq!(value.shape(), Some(CssRadialShape::Ellipse));
                let Some(CssRadialSize::Ellipse(size)) = value.size() else {
                    panic!("two nonnegative ellipse radii")
                };
                let horizontal = size.horizontal().literal_component().unwrap();
                let vertical = size.vertical().literal_component().unwrap();
                parsed_origin(horizontal.origin(), &source, "20%");
                parsed_origin(vertical.origin(), &source, "30%");
                assert_eq!(
                    gradient(&declaration).serialize_specified().unwrap(),
                    format!("{function}(20% 30%, red, blue)"),
                );
            }
        }
        let text = format!("{function}(red, blue)");
        for (declaration, _) in fronts(CssKnownProperty::BackgroundImage, &text) {
            let value = radial(gradient(&declaration));
            assert_eq!(value.shape(), None);
            assert_eq!(value.size(), None);
            assert_eq!(value.position(), None);
            assert_eq!(gradient(&declaration).serialize_specified().unwrap(), text);
        }
        for prelude in [
            "circle 10%",
            "circle 10px 20px",
            "ellipse 10px",
            "circle -1px",
            "ellipse 10px -2px",
            "circle circle",
        ] {
            rejects_background(&format!("{function}({prelude}, red, blue)"));
        }
    }
}

#[test]
fn every_gradient_sibling_rejects_bad_stop_hint_order_and_excess_positions() {
    for function in [
        "linear-gradient",
        "repeating-linear-gradient",
        "radial-gradient",
        "repeating-radial-gradient",
    ] {
        for stops in [
            "red,",
            "20%, red, blue",
            "red, 20%",
            "red, 20%, 30%, blue",
            "red,, blue",
            "red 10% 20% 30%, blue",
            "red 20% blue",
        ] {
            rejects_background(&format!("{function}({stops})"));
        }
    }
}

#[test]
fn nested_src_and_repeating_gradient_pairs_cross_real_image_consumers_without_branch_selection() {
    let text = "light-dark(src(\"#art\"), repeating-linear-gradient(red -10%, 40%, blue 120%))";
    for property in [
        CssKnownProperty::BackgroundImage,
        CssKnownProperty::BorderImageSource,
        CssKnownProperty::ListStyleImage,
        CssKnownProperty::Content,
    ] {
        for (declaration, source) in fronts(property, text) {
            let CssImageValue::LightDark(pair) = image(&declaration) else {
                panic!("image pair remains unresolved")
            };
            let CssImageValue::Url(url) = pair.light() else {
                panic!("light branch is original Src URL")
            };
            assert_eq!(url.function(), CssUrlFunction::Src);
            assert_eq!(url.as_str(), "#art");
            assert!(url.is_local_url());
            let CssImageValue::Gradient(CssGradient::RepeatingLinear(value)) = pair.dark() else {
                panic!("dark branch is a repeating gradient")
            };
            let [
                CssColorStopListItem::Stop(first),
                CssColorStopListItem::Hint(hint),
                CssColorStopListItem::Stop(last),
            ] = value.stops().items()
            else {
                panic!("stop/hint/stop order inside the actual pair")
            };
            percentage(first.position().unwrap(), &source, "-10%");
            percentage(hint, &source, "40%");
            percentage(last.position().unwrap(), &source, "120%");
            let expected = format!("{}: {text} !important;", property.canonical_name());
            assert_eq!(declaration.to_specified_css().unwrap(), expected);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&declaration).unwrap()
            else {
                panic!("ordinary image longhand")
            };
            let [contribution] = values.items() else {
                panic!("one actual image longhand")
            };
            assert_eq!(contribution.property(), property);
            assert!(contribution.source().same_occurrence(&declaration));
            assert_eq!(contribution.source().importance(), CssImportance::Important);
            assert!(contribution.ordinary_value().is_some());
        }
    }
}

#[test]
fn image_reentry_keeps_nested_numeric_origins_and_retries_after_invalid_whole_replacement() {
    let report = parse_style_attribute("/*😀*/list-style-image:var(--art)!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssExpansion::Pending(pending) = expand_declaration(source).unwrap() else {
        panic!("whole symbolic image replacement")
    };
    let text = "light-dark(src(\"#art\"), repeating-radial-gradient(red -10%, 40%, blue 120%))";
    let replacement = parse_component_values(text).unwrap();
    let before = replacement.clone();
    for invalid in [
        "light-dark(src(\"#art\"), repeating-radial-gradient(red,))",
        "light-dark(src(\"#art\"), repeating-radial-gradient(red, blue)) trailing",
    ] {
        assert!(matches!(
            pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("valid retry produces one terminal")
        };
        let [contribution] = values.items() else {
            panic!("one list-style-image contribution")
        };
        assert_eq!(contribution.property(), CssKnownProperty::ListStyleImage);
        assert!(contribution.source().same_occurrence(source));
        assert_eq!(contribution.source().importance(), CssImportance::Important);
        assert_eq!(contribution.replacement_components(), Some(&replacement));
        let CssLonghandValueRef::ListStyleImage(CssImageValue::LightDark(pair)) =
            contribution.ordinary_value().unwrap().view()
        else {
            panic!("typed image pair")
        };
        let CssImageValue::Gradient(CssGradient::RepeatingRadial(value)) = pair.dark() else {
            panic!("repeating radial child remains symbolic")
        };
        let [
            CssColorStopListItem::Stop(first),
            CssColorStopListItem::Hint(hint),
            CssColorStopListItem::Stop(last),
        ] = value.stops().items()
        else {
            panic!("retained numeric children")
        };
        percentage(first.position().unwrap(), text, "-10%");
        percentage(hint, text, "40%");
        percentage(last.position().unwrap(), text, "120%");
        assert_eq!(
            pair.light(),
            &CssImageValue::Url(CssUrl::from_parts(CssUrlFunction::Src, "#art", Vec::new(),))
        );
    }
    assert_eq!(replacement, before);
}

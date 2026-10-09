#![forbid(unsafe_code)]
//! Complete retained Image graph admission through the public checked constructor.
//! #328 requires complete composed image structural admission, not only None rejection.
//! Typed Result admission preserves the published callable graph-rejection expectations.
//! Color5 checked LightDark and Values4 math/URL providers remain independent controls.
//! Independent function-depth expectations and matched provider controls.
use surgeist_css::*;

fn named(name: &str) -> CssColor {
    CssColor::from_named(CssNamedColor::try_new(name).unwrap())
}

fn color_chain(functions: usize) -> CssColor {
    let mut value = named("red");
    for _ in 0..functions {
        value =
            CssColor::from_light_dark(CssLightDarkColor::try_new(value, named("blue")).unwrap());
    }
    value
}

fn math(functions: usize, leaf: &str) -> CssComponentValues {
    let mut text = leaf.to_owned();
    for _ in 0..functions {
        text = format!("calc({text})");
    }
    let values = parse_component_values(&text).unwrap();
    assert_eq!(values.nesting_depth() as usize, functions);
    values
}

fn lp(functions: usize) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_calculation(
        CssLengthPercentageCalculation::try_from_components(math(functions, "1px")).unwrap(),
    )
    .unwrap()
}

fn nnlp(functions: usize) -> CssSpecifiedNonNegativeLengthPercentage {
    CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
        CssLengthPercentageCalculation::try_from_components(math(functions, "1px")).unwrap(),
    )
    .unwrap()
}

fn stops(color: CssColor, position: Option<CssSpecifiedLengthPercentage>) -> CssColorStopList {
    CssColorStopList::try_new(vec![
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(color, position))),
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
            named("blue"),
            None,
        ))),
    ])
    .unwrap()
}

fn color_gradient(functions: usize, family: usize) -> CssImageValue {
    let stops = stops(color_chain(functions), None);
    let gradient = match family {
        0 => CssGradient::Linear(CssLinearGradient::new(None, stops)),
        1 => CssGradient::RepeatingLinear(CssLinearGradient::new(None, stops)),
        2 => CssGradient::Radial(CssRadialGradient::try_new(None, None, None, stops).unwrap()),
        3 => CssGradient::RepeatingRadial(
            CssRadialGradient::try_new(None, None, None, stops).unwrap(),
        ),
        _ => panic!("four selected gradient functions"),
    };
    CssImageValue::Gradient(gradient)
}

fn preserves_acceptance(value: &CssImageValue) {
    let original = value.clone();
    for _ in 0..2 {
        let image =
            CssImage::try_new(value.clone()).expect("complete retained graph at or below 256");
        assert_eq!(image.value(), value);
        assert_eq!(value, &original);
    }
}

fn preserves_rejection_and_retry(invalid: CssImageValue, valid: CssImageValue) {
    let invalid_before = invalid.clone();
    let valid_before = valid.clone();
    preserves_acceptance(&valid);
    for _ in 0..2 {
        // The graph-rejection expectation is unchanged from the published RED.
        // Typed admission now distinguishes structural failure from non-image None.
        assert_eq!(
            CssImage::try_new(invalid.clone()).unwrap_err(),
            CssImageConstructionError::NestingLimit,
            "whole graph exceeds 256 retained function levels"
        );
        assert_eq!(invalid, invalid_before);
        assert_eq!(valid, valid_before);
        preserves_acceptance(&valid);
    }
}

macro_rules! color_boundary {
    ($name:ident, $family:literal) => {
        #[test]
        fn $name() {
            // 255 checked color functions + one gradient = 256; the next is 257.
            preserves_rejection_and_retry(
                color_gradient(256, $family),
                color_gradient(255, $family),
            );
        }
    };
}
color_boundary!(linear_image_rejects_a_257_level_complete_color_graph, 0);
color_boundary!(
    repeating_linear_image_rejects_a_257_level_complete_color_graph,
    1
);
color_boundary!(radial_image_rejects_a_257_level_complete_color_graph, 2);
color_boundary!(
    repeating_radial_image_rejects_a_257_level_complete_color_graph,
    3
);

#[derive(Clone, Copy)]
enum NumericRole {
    Direction,
    Stop,
    Hint,
    Circle,
    EllipseX,
    EllipseY,
    PositionX,
    PositionY,
}

fn numeric_gradient(functions: usize, role: NumericRole, repeating: bool) -> CssImageValue {
    let plain_stops = || stops(named("red"), None);
    let gradient = match role {
        NumericRole::Direction => {
            let angle = CssAngleValue::try_from_calculation(
                CssAngleCalculation::try_from_components(math(functions, "1deg")).unwrap(),
            )
            .unwrap();
            CssGradient::Linear(CssLinearGradient::new(
                Some(CssLinearGradientDirection::Angle(CssAngleOrZero::Angle(
                    angle,
                ))),
                plain_stops(),
            ))
        }
        NumericRole::Stop => CssGradient::Linear(CssLinearGradient::new(
            None,
            stops(named("red"), Some(lp(functions))),
        )),
        NumericRole::Hint => {
            let stops = CssColorStopList::try_new(vec![
                CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
                    named("red"),
                    None,
                ))),
                CssColorStopListItem::Hint(lp(functions)),
                CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
                    named("blue"),
                    None,
                ))),
            ])
            .unwrap();
            CssGradient::Linear(CssLinearGradient::new(None, stops))
        }
        NumericRole::Circle => {
            let radius = CssSpecifiedNonNegativeLength::try_from_calculation(
                CssLengthCalculation::try_from_components(math(functions, "1px")).unwrap(),
            )
            .unwrap();
            CssGradient::Radial(
                CssRadialGradient::try_new(
                    None,
                    Some(CssRadialSize::Circle(radius)),
                    None,
                    plain_stops(),
                )
                .unwrap(),
            )
        }
        NumericRole::EllipseX | NumericRole::EllipseY => {
            let pair = if matches!(role, NumericRole::EllipseX) {
                CssRadialEllipseSize::new(
                    nnlp(functions),
                    CssSpecifiedNonNegativeLengthPercentage::zero(),
                )
            } else {
                CssRadialEllipseSize::new(
                    CssSpecifiedNonNegativeLengthPercentage::zero(),
                    nnlp(functions),
                )
            };
            CssGradient::Radial(
                CssRadialGradient::try_new(
                    None,
                    Some(CssRadialSize::Ellipse(pair)),
                    None,
                    plain_stops(),
                )
                .unwrap(),
            )
        }
        NumericRole::PositionX | NumericRole::PositionY => {
            let (x, y) = if matches!(role, NumericRole::PositionX) {
                (
                    CssHorizontalPosition::Offset(lp(functions)),
                    CssVerticalPosition::Center,
                )
            } else {
                (
                    CssHorizontalPosition::Center,
                    CssVerticalPosition::Offset(lp(functions)),
                )
            };
            let position = CssPhysicalPosition::try_new(x, y).unwrap();
            CssGradient::Radial(
                CssRadialGradient::try_new(None, None, Some(position.into()), plain_stops())
                    .unwrap(),
            )
        }
    };
    let gradient = if repeating {
        match gradient {
            CssGradient::Linear(value) => CssGradient::RepeatingLinear(value),
            CssGradient::Radial(value) => CssGradient::RepeatingRadial(value),
            _ => panic!("unrepeated fixture"),
        }
    } else {
        gradient
    };
    CssImageValue::Gradient(gradient)
}

macro_rules! numeric_boundary {
    ($name:ident, $role:ident) => {
        #[test]
        fn $name() {
            for repeating in [false, true] {
                // Original calc graph255 + gradient1 is admitted; 256+1 is not.
                preserves_rejection_and_retry(
                    numeric_gradient(256, NumericRole::$role, repeating),
                    numeric_gradient(255, NumericRole::$role, repeating),
                );
            }
        }
    };
}
numeric_boundary!(image_root_checks_direction_math, Direction);
numeric_boundary!(image_root_checks_stop_math, Stop);
numeric_boundary!(image_root_checks_hint_math, Hint);
numeric_boundary!(image_root_checks_circle_radius_math, Circle);
numeric_boundary!(image_root_checks_first_ellipse_radius_math, EllipseX);
numeric_boundary!(image_root_checks_second_ellipse_radius_math, EllipseY);
numeric_boundary!(image_root_checks_horizontal_position_math, PositionX);
numeric_boundary!(image_root_checks_vertical_position_math, PositionY);

fn modified_url(argument_functions: usize) -> CssImageValue {
    let arguments = math(argument_functions, "1");
    let before = arguments.clone();
    let modifier =
        CssUrlModifierFunction::try_new(CssIdent::try_new("m").unwrap(), arguments.clone())
            .unwrap();
    assert_eq!(modifier.argument_components(), &before);
    assert_eq!(arguments, before);
    CssImageValue::Url(CssUrl::from_parts(
        CssUrlFunction::Src,
        "a.svg",
        vec![CssUrlModifier::Function(modifier)],
    ))
}

#[test]
fn image_root_checks_url_modifier_and_retained_argument_functions() {
    // URL1 + modifier function1 + argument functions254 = 256.
    // Existing modifier construction admits255+1; the containing URL makes257.
    preserves_rejection_and_retry(modified_url(255), modified_url(254));
}

#[test]
fn color_provider_control_admits_256_and_refuses_257_before_image_composition() {
    let value = color_chain(256);
    let before = value.clone();
    for _ in 0..2 {
        assert_eq!(
            CssLightDarkColor::try_new(value.clone(), named("blue")).unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
        assert_eq!(
            CssLightDarkColor::try_new(named("blue"), value.clone()).unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
        assert_eq!(value, before);
        assert!(CssLightDarkColor::try_new(color_chain(255), named("blue")).is_ok());
    }
}

#[test]
fn numeric_provider_control_retains_the_complete_256_level_original_components() {
    for leaf in ["1px", "1deg"] {
        let components = math(256, leaf);
        let before = components.clone();
        if leaf == "1px" {
            let length =
                CssLengthPercentageCalculation::try_from_components(components.clone()).unwrap();
            assert_eq!(length.components(), &before);
            assert_eq!(
                CssSpecifiedLengthPercentage::try_from_calculation(length)
                    .unwrap()
                    .calculation()
                    .unwrap()
                    .components(),
                &before
            );
        } else {
            let angle = CssAngleCalculation::try_from_components(components.clone()).unwrap();
            assert_eq!(angle.components(), &before);
            assert_eq!(
                CssAngleValue::try_from_calculation(angle)
                    .unwrap()
                    .calculation()
                    .unwrap()
                    .components(),
                &before
            );
        }
        assert_eq!(components, before);
    }
}

#[test]
fn image_pair_provider_control_counts_its_own_enclosing_function_and_cached_children() {
    let mut image = CssImageValue::None;
    for _ in 0..256 {
        image = CssImageValue::LightDark(Box::new(
            CssLightDarkImage::try_new(image, CssImageValue::None).unwrap(),
        ));
    }
    let before = image.clone();
    preserves_acceptance(&image); // CssImage itself adds no CSS function level.
    for _ in 0..2 {
        assert_eq!(
            CssLightDarkImage::try_new(image.clone(), CssImageValue::None).unwrap_err(),
            CssImageConstructionError::NestingLimit
        );
        assert_eq!(
            CssLightDarkImage::try_new(CssImageValue::None, image.clone()).unwrap_err(),
            CssImageConstructionError::NestingLimit
        );
        assert_eq!(image, before);
    }
    // Gradient1 + color254 + pair1 = 256. Increasing color by one rejects.
    assert!(CssLightDarkImage::try_new(color_gradient(254, 0), CssImageValue::None).is_ok());
    assert_eq!(
        CssLightDarkImage::try_new(color_gradient(255, 0), CssImageValue::None).unwrap_err(),
        CssImageConstructionError::NestingLimit
    );
}

#[test]
fn none_and_low_depth_image_controls_preserve_every_existing_family_and_payload() {
    for _ in 0..2 {
        assert_eq!(
            CssImage::try_new(CssImageValue::None).unwrap_err(),
            CssImageConstructionError::NotImage
        );
    }
    preserves_acceptance(&CssImageValue::Url(CssUrl::new("a.svg")));
    for family in 0..4 {
        preserves_acceptance(&color_gradient(0, family));
    }
    let pair = CssLightDarkImage::try_new(CssImageValue::None, CssImageValue::None).unwrap();
    assert_eq!(pair.light(), &CssImageValue::None);
    assert_eq!(pair.dark(), &CssImageValue::None);
    preserves_acceptance(&CssImageValue::LightDark(Box::new(pair)));
}

#[test]
fn mixed_source_numeric_components_survive_admission_and_failed_enclosing_composition() {
    let parsed = parse_component_values("/*😀*/1px").unwrap();
    let parsed_before = parsed.clone();
    let mut components = parsed.clone();
    for _ in 0..256 {
        components = CssComponentValues::try_new(vec![
            CssComponentValue::try_function("calc", components).unwrap(),
        ])
        .unwrap();
    }
    assert_eq!(components.nesting_depth(), 256);
    let before = components.clone();
    let calculation =
        CssLengthPercentageCalculation::try_from_components(components.clone()).unwrap();
    assert_eq!(calculation.components(), &before);
    let position = CssSpecifiedLengthPercentage::try_from_calculation(calculation).unwrap();
    let invalid = CssImageValue::Gradient(CssGradient::Linear(CssLinearGradient::new(
        None,
        stops(named("red"), Some(position.clone())),
    )));
    assert_eq!(
        CssImage::try_new(invalid).unwrap_err(),
        CssImageConstructionError::NestingLimit
    );
    assert_eq!(position.calculation().unwrap().components(), &before);
    assert_eq!(components, before);
    assert_eq!(parsed, parsed_before);
}

fn cursor_fronts(text: &str, admitted: bool) {
    let p = CssKnownProperty::Cursor;
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    for grammar in [false, true] {
        let result = if grammar {
            parse_property_value_for_grammar(p.grammar(), values.clone(), CssImportance::Important)
        } else {
            parse_property_value(
                CssPropertyNameRef::Known(p),
                values.clone(),
                CssImportance::Important,
            )
        };
        if admitted {
            let source = result.unwrap();
            assert_eq!(source.value_components(), &before);
            assert_eq!(source.importance(), CssImportance::Important);
        } else {
            assert!(matches!(
                result.unwrap_err().kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
        }
    }
    for report in [
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Important),
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Important),
    ] {
        assert_eq!(report.is_clean(), admitted);
        assert_eq!(report.syntax().is_some(), admitted);
    }
    let css = format!("color:red;cursor:{text}!important;color:blue");
    let report = parse_style_attribute(&css);
    if admitted {
        assert!(report.is_clean());
        assert_eq!(report.syntax().len(), 3);
    } else {
        assert_eq!(report.syntax().len(), 2);
        let [d] = report.diagnostics() else {
            panic!("one atomic rejection")
        };
        assert_eq!(d.action(), CssRecoveryAction::DropDeclaration);
    }
    assert_eq!(values, before);
}

#[test]
fn selected_ui4_cursor_control_preserves_url_and_limited_url_set_admission() {
    for text in [
        "url(a.cur),pointer",
        "src('a.cur') -1 2,pointer",
        "image-set(url(a.cur) 1x,src('b.cur') 2x),auto",
        "image-set('a.cur' type('image/png') 2x) calc(1 + 2) calc(-1),move",
        "-webkit-image-set('a.cur' 1x),none",
    ] {
        cursor_fronts(text, true);
    }
}

#[test]
fn selected_narrow_cursor_owner_control_does_not_require_optional_full_image_support() {
    // UI4 §5.1.1 makes broader Image optional. Accepted #426 owns the narrow
    // grammar; these retain that owner and are controls, not widening RED.
    for text in [
        "linear-gradient(red,blue),pointer",
        "radial-gradient(red,blue),pointer",
        "light-dark(url(a),url(b)),pointer",
        "light-dark(none,none),pointer",
        "filter(url(a),blur()),pointer",
        "image-set(linear-gradient(red,blue) 1x),pointer",
        "image-set(filter(url(a),blur()) 1x),pointer",
        "'a.cur',pointer",
        "url(a.cur) 1,pointer",
        "url(a.cur)",
    ] {
        cursor_fronts(text, false);
    }
}

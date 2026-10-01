#![forbid(unsafe_code)]
//! Functional evidence for new specified serializers and the omission wrapper.
//! Grammar/defaults: Filter Effects 1 WD 2018-12-18 §§5, 6.1, 6.3.
//! Canonical child projection and exact resource charges follow the adopted
//! Surgeist authored contract, without computed defaults or filter execution.
//! These new APIs have no executable preimplementation behavioral RED.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn declaration(property: CssKnownProperty, css: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{}:{css}", property.canonical_name()));
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    value.clone()
}

fn filter(property: CssKnownProperty, css: &str) -> CssFilter {
    let source = declaration(property, css);
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Filter(value) => value.value().clone(),
        CssKnownPropertyValueRef::BackdropFilter(value) => value.value().clone(),
        _ => panic!("filter property"),
    }
}

fn function(css: &str) -> CssFilterFunction {
    let CssFilter::Functions(list) = filter(CssKnownProperty::Filter, css) else {
        panic!("function list")
    };
    let [value] = list.functions() else {
        panic!("one function")
    };
    value.clone()
}

fn angle(value: f32, unit: CssAngleUnit) -> CssAngleValue {
    CssAngleValue::Literal(CssAngleLiteral::try_new(value, unit).unwrap())
}

fn angle_math(css: &str) -> CssAngleValue {
    CssAngleValue::Calculation(
        CssAngleCalculation::try_from_components(parse_component_values(css).unwrap()).unwrap(),
    )
}

fn length(css: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_component(CssComponentValue::try_token(css).unwrap()).unwrap()
}

fn nonnegative_length(css: &str) -> CssSpecifiedNonNegativeLength {
    CssSpecifiedNonNegativeLength::try_from_component(CssComponentValue::try_token(css).unwrap())
        .unwrap()
}

fn number(css: &str) -> CssFilterAmount {
    CssFilterAmount::Number(
        CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number(css).unwrap(),
        )
        .unwrap(),
    )
}

fn percentage(css: &str) -> CssFilterAmount {
    CssFilterAmount::Percentage(
        CssSpecifiedNonNegativePercentage::try_from_component(
            CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap(),
    )
}

fn color(css: &str) -> CssColor {
    let source = declaration(CssKnownProperty::Color, css);
    let CssKnownPropertyValueRef::Color(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    value.value().clone()
}

fn budget(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        serialize(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            L::new(input - 1, projection, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), K::ByteLimit),
        (L::new(0, projection, expected.len()), K::InputNodeLimit),
        (L::new(input, 0, expected.len()), K::ProjectionNodeLimit),
        (L::new(input, projection, 0), K::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind, "{expected}");
    }
    assert_eq!(serialize(L::default()).unwrap(), expected);
}

#[test]
fn hue_construction_keeps_omission_zero_units_and_origin_sensitive_math() {
    let omitted = CssFilterHueRotate::omitted();
    assert!(omitted.authored_angle().is_none());
    assert_eq!(omitted.angle(), &angle(0.0, CssAngleUnit::Degrees));
    let zero = CssFilterHueRotate::new(CssAngleValue::Zero);
    let degrees = CssFilterHueRotate::new(angle(0.0, CssAngleUnit::Degrees));
    assert_eq!(zero.authored_angle(), Some(&CssAngleValue::Zero));
    assert_eq!(degrees.authored_angle(), Some(degrees.angle()));
    assert_ne!(omitted, zero);
    assert_ne!(omitted, degrees);
    assert_ne!(zero, degrees);
    for (unit, expected) in [
        (CssAngleUnit::Degrees, "hue-rotate(450deg)"),
        (CssAngleUnit::Gradians, "hue-rotate(450grad)"),
        (CssAngleUnit::Radians, "hue-rotate(450rad)"),
        (CssAngleUnit::Turns, "hue-rotate(450turn)"),
    ] {
        let hue = CssFilterHueRotate::new(angle(450.0, unit));
        assert_eq!(hue.angle(), &angle(450.0, unit));
        assert_eq!(
            CssFilterFunction::HueRotate(hue)
                .serialize_specified()
                .unwrap(),
            expected
        );
    }
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(CssAngleLiteral::try_new(invalid, CssAngleUnit::Degrees).is_none());
    }
    let first = CssFilterHueRotate::new(angle_math("calc(30deg + 60deg)"));
    let second = CssFilterHueRotate::new(angle_math("  calc(30deg + 60deg)"));
    assert_ne!(first, second);
    let CssAngleValue::Calculation(math) = first.angle() else {
        panic!("math")
    };
    let origin = math.origin().clone();
    let function = CssFilterFunction::HueRotate(first.clone());
    budget("hue-rotate(calc(90deg))", 5, 4, |limits| {
        function.serialize_specified_with_limits(limits)
    });
    let CssFilterFunction::HueRotate(after) = function else {
        panic!("hue")
    };
    assert_eq!(after, first);
    let CssAngleValue::Calculation(math) = after.angle() else {
        panic!("math")
    };
    assert_eq!(math.origin(), &origin);
}

#[test]
fn parsed_empty_and_comment_hue_keep_omission_and_authored_declaration_text() {
    for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
        for css in ["hue-rotate()", "hue-rotate(/**/)"] {
            let source = declaration(property, css);
            let CssFilter::Functions(list) = filter(property, css) else {
                panic!("list")
            };
            let [CssFilterFunction::HueRotate(hue)] = list.functions() else {
                panic!("hue")
            };
            assert!(hue.authored_angle().is_none());
            assert_eq!(hue.angle(), &angle(0.0, CssAngleUnit::Degrees));
            assert_eq!(list.serialize_specified().unwrap(), "hue-rotate()");
            let span = source.parsed_value().unwrap().span();
            assert_eq!(
                &source.parsed_value().unwrap().source().as_str()
                    [span.start().byte_offset().value()..span.end().byte_offset().value()],
                css
            );
        }
    }
}

#[test]
fn every_function_has_independent_literal_text_and_exact_budgets() {
    for (css, input, projection) in [
        ("blur()", 1, 1),
        ("blur(2em)", 2, 2),
        ("brightness()", 1, 1),
        ("brightness(2)", 2, 2),
        ("contrast(150%)", 2, 2),
        ("grayscale(2)", 2, 2),
        ("hue-rotate()", 1, 1),
        ("hue-rotate(0)", 2, 2),
        ("hue-rotate(0deg)", 2, 2),
        ("hue-rotate(450deg)", 2, 2),
        ("hue-rotate(-0.25turn)", 2, 2),
        ("invert(2)", 2, 2),
        ("opacity(125%)", 2, 2),
        ("saturate(3)", 2, 2),
        ("sepia(200%)", 2, 2),
        ("drop-shadow(1px 2px)", 3, 3),
        ("drop-shadow(red 1px 2px 3px)", 5, 5),
        ("url(\"#f\")", 2, 2),
        ("url(\"x\" cors m(a b))", 7, 7),
    ] {
        let value = function(css);
        let before = value.clone();
        budget(css, input, projection, |limits| {
            value.serialize_specified_with_limits(limits)
        });
        assert_eq!(value, before);
        assert_eq!(value.serialize_specified().unwrap(), css);
    }
    for name in [
        "brightness",
        "contrast",
        "grayscale",
        "invert",
        "opacity",
        "saturate",
        "sepia",
    ] {
        let css = format!("{name}()");
        let value = function(&css);
        budget(&css, 1, 1, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }
    budget("none", 1, 1, |limits| {
        CssFilter::None.serialize_specified_with_limits(limits)
    });
}

#[test]
fn checked_all_function_construction_preserves_order_and_literal_domains() {
    let drop = CssDropShadow::new(
        length("1px"),
        length("2px"),
        Some(nonnegative_length("3px")),
        Some(color("red")),
    );
    assert_eq!(drop.offset_x().origin(), &CssValueOrigin::Programmatic);
    let functions = vec![
        CssFilterFunction::Blur(CssFilterBlur::new(nonnegative_length("2em"))),
        CssFilterFunction::Brightness(number("2")),
        CssFilterFunction::Contrast(percentage("150%")),
        CssFilterFunction::DropShadow(drop),
        CssFilterFunction::Grayscale(number("2")),
        CssFilterFunction::HueRotate(CssFilterHueRotate::new(angle(450.0, CssAngleUnit::Degrees))),
        CssFilterFunction::Invert(number("2")),
        CssFilterFunction::Opacity(percentage("125%")),
        CssFilterFunction::Saturate(number("3")),
        CssFilterFunction::Sepia(percentage("200%")),
    ];
    assert!(CssFilterFunctionList::try_new(Vec::new()).is_none());
    let list = CssFilterFunctionList::try_new(functions.clone()).unwrap();
    assert_eq!(list.functions(), functions);
    let expected = "blur(2em) brightness(2) contrast(150%) drop-shadow(red 1px 2px 3px) grayscale(2) hue-rotate(450deg) invert(2) opacity(125%) saturate(3) sepia(200%)";
    budget(expected, 24, 24, |limits| {
        list.serialize_specified_with_limits(limits)
    });
    let value = CssFilter::Functions(list);
    budget(expected, 24, 24, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    assert_eq!(value.serialize_specified().unwrap(), expected);
    let defaults = CssFilterFunctionList::try_new(vec![
        CssFilterFunction::Blur(CssFilterBlur::omitted()),
        CssFilterFunction::HueRotate(CssFilterHueRotate::omitted()),
        CssFilterFunction::Brightness(CssFilterAmount::Default),
        CssFilterFunction::Contrast(CssFilterAmount::Default),
        CssFilterFunction::Grayscale(CssFilterAmount::Default),
        CssFilterFunction::Invert(CssFilterAmount::Default),
        CssFilterFunction::Opacity(CssFilterAmount::Default),
        CssFilterFunction::Saturate(CssFilterAmount::Default),
        CssFilterFunction::Sepia(CssFilterAmount::Default),
    ])
    .unwrap();
    budget(
        "blur() hue-rotate() brightness() contrast() grayscale() invert() opacity() saturate() sepia()",
        10,
        10,
        |limits| defaults.serialize_specified_with_limits(limits),
    );
}

#[test]
fn list_enum_and_shared_children_have_cumulative_resources() {
    for (css, expected, input, projection) in [
        (
            "blur() hue-rotate() drop-shadow(1px 2px)",
            "blur() hue-rotate() drop-shadow(1px 2px)",
            6,
            6,
        ),
        (
            "blur(2em) hue-rotate(450deg) url(\"#f\") drop-shadow(red 1px 2px 3px)",
            "blur(2em) hue-rotate(450deg) url(\"#f\") drop-shadow(red 1px 2px 3px)",
            12,
            12,
        ),
        (
            "blur(calc(1px + 2em)) blur(calc(1px + 2em))",
            "blur(calc(2em + 1px)) blur(calc(2em + 1px))",
            11,
            13,
        ),
        (
            "url(\"x\" cors m(a b)) hue-rotate()",
            "url(\"x\" cors m(a b)) hue-rotate()",
            9,
            9,
        ),
        (
            "drop-shadow(1px 2px) drop-shadow(3px 4px)",
            "drop-shadow(1px 2px) drop-shadow(3px 4px)",
            7,
            7,
        ),
        (
            "drop-shadow(red calc(1px + 2em) 2px) drop-shadow(red calc(1px + 2em) 2px)",
            "drop-shadow(red calc(2em + 1px) 2px) drop-shadow(red calc(2em + 1px) 2px)",
            15,
            17,
        ),
        (
            "blur() hue-rotate() brightness() contrast() grayscale() invert() opacity() saturate() sepia()",
            "blur() hue-rotate() brightness() contrast() grayscale() invert() opacity() saturate() sepia()",
            10,
            10,
        ),
    ] {
        for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
            let value = filter(property, css);
            let before = value.clone();
            budget(expected, input, projection, |limits| {
                value.serialize_specified_with_limits(limits)
            });
            let CssFilter::Functions(list) = &value else {
                panic!("list")
            };
            budget(expected, input, projection, |limits| {
                list.serialize_specified_with_limits(limits)
            });
            assert_eq!(list.serialize_specified().unwrap(), expected);
            assert_eq!(
                filter(property, expected).serialize_specified().unwrap(),
                expected
            );
            assert_eq!(value, before);
        }
    }
}

#[test]
fn exact_amount_literals_keep_branch_spelling_and_origins_without_clamping() {
    let huge = "1234567890123456789012345678901234567890";
    let parsed = function(&format!("brightness({huge})"));
    let CssFilterFunction::Brightness(CssFilterAmount::Number(value)) = &parsed else {
        panic!("number")
    };
    let origin = value.origin().clone();
    assert!(matches!(origin, CssValueOrigin::Parsed(_)));
    budget(&format!("brightness({huge})"), 2, 2, |limits| {
        parsed.serialize_specified_with_limits(limits)
    });
    assert_eq!(value.origin(), &origin);
    let constructed = CssFilterFunction::Brightness(number(huge));
    assert_eq!(constructed, parsed);
    let CssFilterFunction::Brightness(CssFilterAmount::Number(value)) = constructed else {
        panic!("number")
    };
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    for (css, expected) in [
        ("contrast(1.50e2%)", "contrast(150%)"),
        ("grayscale(2)", "grayscale(2)"),
        ("invert(-0)", "invert(0)"),
        ("opacity(calc(50% + 75%))", "opacity(calc(125%))"),
    ] {
        assert_eq!(function(css).serialize_specified().unwrap(), expected);
    }
    assert_ne!(function("brightness()"), function("brightness(1)"));
    assert_ne!(function("brightness(1)"), function("brightness(100%)"));
}

#[test]
fn symbolic_drop_children_share_existing_providers_without_mutating_origins() {
    let css = "drop-shadow(calc(1px + 2em) -2px color-mix(in srgb, currentcolor, blue))";
    let value = function(css);
    let CssFilterFunction::DropShadow(drop) = &value else {
        panic!("drop")
    };
    let origin = drop.offset_x().origin().clone();
    assert!(drop.color().unwrap().color_mix_value().is_some());
    assert!(drop.standard_deviation().is_none());
    let expected = "drop-shadow(color-mix(in srgb, currentcolor, blue) calc(2em + 1px) -2px)";
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(drop.offset_x().origin(), &origin);
    assert_eq!(
        filter(CssKnownProperty::Filter, expected)
            .serialize_specified()
            .unwrap(),
        expected
    );
    let explicit = function("drop-shadow(currentcolor 0 0 0)");
    assert_eq!(
        explicit.serialize_specified().unwrap(),
        "drop-shadow(currentcolor 0 0 0)"
    );
}

#[test]
fn url_escaping_modifiers_and_adjacent_tokens_compose_with_sibling_functions() {
    let modifier = CssUrlModifier::Function(
        CssUrlModifierFunction::try_new(
            CssIdent::try_new("m").unwrap(),
            CssComponentValues::try_new(vec![
                CssComponentValue::try_ident("a").unwrap(),
                CssComponentValue::try_number("1").unwrap(),
            ])
            .unwrap(),
        )
        .unwrap(),
    );
    let url = CssUrl::from_parts(CssUrlFunction::Src, "é\"\\\n", vec![modifier]);
    let value = CssFilter::Functions(
        CssFilterFunctionList::try_new(vec![
            CssFilterFunction::Url(url.clone()),
            CssFilterFunction::HueRotate(CssFilterHueRotate::omitted()),
        ])
        .unwrap(),
    );
    let expected = "src(\"é\\\"\\\\\\a \" m(a/**/1)) hue-rotate()";
    // List1, URL1, target1, modifier1, two original argument tokens, hue1.
    budget(expected, 7, 7, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(url.as_str(), "é\"\\\n");
    assert_eq!(
        filter(CssKnownProperty::Filter, expected)
            .serialize_specified()
            .unwrap(),
        expected
    );
}

#[test]
fn borrowed_filter_terminals_and_all_reset_keep_schema_identity() {
    for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("terminal")
        };
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary initial")
        };
        assert!(matches!(
            (property, initial.view()),
            (
                CssKnownProperty::Filter,
                CssLonghandValueRef::Filter(CssFilter::None)
            ) | (
                CssKnownProperty::BackdropFilter,
                CssLonghandValueRef::BackdropFilter(CssFilter::None)
            )
        ));
        let source = declaration(property, "hue-rotate()");
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("terminal")
        };
        let [value] = values.items() else {
            panic!("one contribution")
        };
        let borrowed = value.ordinary_value().unwrap().view();
        assert!(matches!(
            (property, borrowed),
            (
                CssKnownProperty::Filter,
                CssLonghandValueRef::Filter(CssFilter::Functions(_))
            ) | (
                CssKnownProperty::BackdropFilter,
                CssLonghandValueRef::BackdropFilter(CssFilter::Functions(_))
            )
        ));
        let CssPropertyKindRef::UniversalReset(all) =
            CssKnownProperty::All.metadata().unwrap().kind()
        else {
            panic!("all metadata")
        };
        assert!(!all.excludes(CssPropertyNameRef::Known(property)));
        let source = declaration(CssKnownProperty::All, "unset");
        let CssExpansion::Contributions(CssContributions::UniversalReset(all)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("all reset")
        };
        assert!(!all.excludes(CssPropertyNameRef::Known(property)));
    }
}

#[test]
fn repeated_pending_reentry_preserves_replacement_child_origins_after_failure() {
    for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
        let source = declaration(property, "var(--effect)!important");
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        assert!(
            pending
                .reenter(parse_component_values("blur(-1px)").unwrap())
                .is_err()
        );
        for css in [
            "blur(2em) hue-rotate(calc(30deg + 60deg))",
            "blur(3px) hue-rotate()",
        ] {
            let replacement = parse_component_values(css).unwrap();
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("longhands")
            };
            let [value] = values.items() else {
                panic!("one contribution")
            };
            assert!(value.source().same_occurrence(&source));
            assert_eq!(value.source().importance(), CssImportance::Important);
            assert_eq!(value.replacement_components(), Some(&replacement));
            let borrowed = value.ordinary_value().unwrap();
            let filter = match borrowed.view() {
                CssLonghandValueRef::Filter(filter)
                | CssLonghandValueRef::BackdropFilter(filter) => filter,
                _ => panic!("filter"),
            };
            let CssFilter::Functions(list) = filter else {
                panic!("list")
            };
            let [
                CssFilterFunction::Blur(blur),
                CssFilterFunction::HueRotate(hue),
            ] = list.functions()
            else {
                panic!("blur and hue")
            };
            let CssValueOrigin::Parsed(origin) = blur.authored_length().unwrap().origin() else {
                panic!("parsed replacement child")
            };
            assert_eq!(origin.source().as_str(), css);
            let before = filter.clone();
            assert!(
                filter
                    .serialize_specified_with_limits(L::new(100, 100, 1))
                    .is_err()
            );
            assert_eq!(filter, &before);
            if let Some(CssAngleValue::Calculation(math)) = hue.authored_angle() {
                let CssValueOrigin::Parsed(origin) = math.origin() else {
                    panic!("parsed angle")
                };
                assert_eq!(origin.source().as_str(), css);
            }
        }
        assert!(pending.source().same_occurrence(&source));
    }
}

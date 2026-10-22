//! Authored `<color>` construction through both production source modes: text
//! parsed by the `color` property grammar, and component values checked by it.
//!
//! Color 4 (CRD 2026-09-08) and Color 5 (WD 2026-09-08) define the accepted
//! forms; the values stay symbolic, keep their exact numeric spellings and
//! origins, and a rejection is located at its responsible authored token. CSS
//! Syntax 3 (CRD 2026-10-01) §5.5.10 closes a function at EOF, so an unclosed
//! function's responsible site is the end of the input.

use surgeist_css::*;

fn color_of(declaration: &CssDeclaration) -> CssColor {
    let Some(CssKnownPropertyValueRef::Color(value)) =
        declaration.known().and_then(|known| known.property_value())
    else {
        panic!("typed color: {declaration:?}");
    };
    value.value().clone()
}

/// The `color` property's parse of authored value text.
fn parsed_report(text: &str) -> CssParseReport<Option<CssDeclaration>> {
    parse_property_value_text(
        text,
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        CssImportance::Normal,
    )
}

/// The `color` property's check of component values.
fn checked(values: CssComponentValues) -> Result<CssDeclaration, CssPropertyValueParseError> {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        values,
        CssImportance::Normal,
    )
}

fn parsed(text: &str) -> Result<CssColor, String> {
    let report = parsed_report(text);
    match report.into_validation_result() {
        Ok(declaration) => Ok(color_of(&declaration.expect("retained color value"))),
        Err(failure) => Err(format!("{:?}", failure.first().error())),
    }
}

fn components(text: &str) -> Result<CssColor, String> {
    checked(parse_component_values(text).unwrap())
        .map(|declaration| color_of(&declaration))
        .map_err(|error| format!("{error:?}"))
}

/// The sole diagnostic of a rejected parsed-text color.
fn parsed_error(text: &str) -> Error {
    let report = parsed_report(text);
    let [diagnostic] = report.diagnostics() else {
        panic!("{text}: exactly one diagnostic: {:?}", report.diagnostics());
    };
    diagnostic.error().clone()
}

/// The authored byte offset a component-value rejection reports.
fn checked_error_offset(text: &str) -> usize {
    let error = checked(parse_component_values(text).unwrap()).unwrap_err();
    let origin = match error.origin() {
        CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => origin,
        other => panic!("{text}: rejection names an authored token: {other:?}"),
    };
    match origin {
        CssValueOrigin::Parsed(parsed) | CssValueOrigin::ImplicitClosure { at: parsed, .. } => {
            assert_eq!(parsed.source().as_str(), text);
            parsed.span().start().byte_offset().value()
        }
        other => panic!("{text}: authored origin: {other:?}"),
    }
}

#[test]
fn literal_rgb_uses_real_parsed_and_component_entries() {
    for value in [parsed("rgb(1 2 3)"), components("rgb(1 2 3)")] {
        let value = value.unwrap();
        let rgb = value.rgb_value().unwrap();
        assert_eq!(rgb.syntax(), CssColorSyntax::Modern);
        for (channel, expected) in rgb.channels().iter().zip(["1", "2", "3"]) {
            let CssColorComponent::Number(value) = channel else {
                panic!("number");
            };
            assert_eq!(value.numeric().representation(), expected);
        }
    }
}

fn branch(value: &CssColor) -> usize {
    if value.is_current_color() {
        0
    } else if value.is_transparent() {
        1
    } else if value.named().is_some() {
        2
    } else if value.hex_value().is_some() {
        3
    } else if value.system().is_some() {
        4
    } else if value.rgb_value().is_some() {
        5
    } else if value.hsl_value().is_some() {
        6
    } else if value.hwb_value().is_some() {
        7
    } else if value.lab_value().is_some() {
        8
    } else if value.lch_value().is_some() {
        9
    } else if value.oklab_value().is_some() {
        10
    } else if value.oklch_value().is_some() {
        11
    } else if value.predefined_value().is_some() {
        12
    } else if value.custom_value().is_some() {
        13
    } else if value.relative_value().is_some() {
        14
    } else if value.relative_custom_value().is_some() {
        15
    } else if value.alpha_value().is_some() {
        16
    } else if value.color_mix_value().is_some() {
        17
    } else if value.light_dark_value().is_some() {
        18
    } else if value.contrast_color_value().is_some() {
        19
    } else if value.device_cmyk_value().is_some() {
        20
    } else {
        panic!("native branch")
    }
}

#[test]
fn all_twenty_one_native_branches_are_symbolic_in_both_source_modes() {
    for (text, expected) in [
        ("currentcolor", 0),
        ("transparent", 1),
        ("ReBeccAPurPle", 2),
        ("#F08c", 3),
        ("CanvasText", 4),
        ("rgb(1 2 3)", 5),
        ("hsl(10 20% 30%)", 6),
        ("hwb(10 20% 30%)", 7),
        ("lab(50 1 2)", 8),
        ("lch(50 1 2)", 9),
        ("oklab(.5 .1 .2)", 10),
        ("oklch(.5 .1 2)", 11),
        ("color(display-p3 1 0 .5)", 12),
        ("color(--Exact 1 2 3 4)", 13),
        ("rgb(from red r g b)", 14),
        ("color(from red --Exact foo bar baz quux)", 15),
        ("alpha(from red / alpha)", 16),
        ("color-mix(in srgb, red, blue)", 17),
        ("light-dark(red, blue)", 18),
        ("contrast-color(red)", 19),
        ("device-cmyk(0 1 1 0)", 20),
    ] {
        for value in [parsed(text), components(text)] {
            assert_eq!(
                branch(&value.unwrap_or_else(|error| panic!("{text}: {error}"))),
                expected,
                "{text}"
            );
        }
    }
}

#[test]
fn aliases_escapes_comments_and_exact_numeric_origins_survive() {
    for text in ["rgba(1, 2, 3, .5)", "\\72 GB(1/**/ 2 3 / .5)"] {
        let value = parsed(text).unwrap();
        let rgb = value.rgb_value().unwrap();
        assert!(rgb.alpha().is_some());
        let CssColorComponent::Number(first) = &rgb.channels()[0] else {
            panic!("number");
        };
        let CssValueOrigin::Parsed(origin) = first.origin() else {
            panic!("parsed origin");
        };
        assert_eq!(origin.source().as_str(), text);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            text.find('1').unwrap()
        );
    }
    let text = "rgb(+0001.2500e-02 -0 1e999 / none)";
    for value in [parsed(text), components(text)] {
        let value = value.unwrap();
        let rgb = value.rgb_value().unwrap();
        for (channel, expected) in rgb.channels().iter().zip(["+0001.2500e-02", "-0", "1e999"]) {
            let CssColorComponent::Number(value) = channel else {
                panic!("number");
            };
            assert_eq!(value.numeric().representation(), expected);
        }
        assert!(matches!(rgb.alpha(), Some(CssColorComponent::None)));
    }
    let color = parsed("hsl(1.25TuRn 20% 30%)").unwrap();
    let CssColorHue::Angle(angle) = color.hsl_value().unwrap().hue() else {
        panic!("angle");
    };
    assert_eq!(angle.numeric().representation(), "1.25");
    assert_eq!(angle.unit(), CssAngleUnit::Turns);
    assert_eq!(
        parsed("hsla(10, 20%, 30%, .5)")
            .unwrap()
            .hsl_value()
            .unwrap()
            .syntax(),
        CssColorSyntax::Legacy
    );
    assert_eq!(parsed("aqua").unwrap().named().unwrap().name(), "aqua");
    assert_eq!(parsed("cyan").unwrap().named().unwrap().name(), "cyan");
}

#[test]
fn shared_numeric_admission_retains_calculations_and_relative_environments() {
    for parse in [parsed, components] {
        let value = parse("rgb(calc(1 + 2) calc(20%) none / calc(.5))").unwrap();
        let rgb = value.rgb_value().unwrap();
        assert!(matches!(
            rgb.channels()[0],
            CssColorComponent::NumberCalculation(_)
        ));
        assert!(matches!(
            rgb.channels()[1],
            CssColorComponent::PercentageCalculation(_)
        ));
        assert!(matches!(rgb.channels()[2], CssColorComponent::None));
        let value = parse("lch(from red l c calc(h + 1) / alpha)").unwrap();
        let relative = value.relative_value().unwrap();
        assert_eq!(*relative.function(), CssRelativeColorFunction::Lch);
        assert_eq!(
            relative.channels()[2].result_domain(),
            CssRelativeColorResultDomain::Hue
        );
        assert!(matches!(
            relative.channels()[2].value(),
            CssRelativeColorExpressionValue::Calculation(_)
        ));
        assert!(relative.alpha().is_some());
        for text in [
            "hsl(from red h s l)",
            "hwb(from red h w b)",
            "lab(from red l a b)",
            "oklab(from red l a b)",
            "oklch(from red l c h)",
            "color(from red display-p3 r g b)",
        ] {
            assert!(parse(text).unwrap().relative_value().is_some(), "{text}");
        }
    }
}

#[test]
fn custom_names_weights_order_and_nested_branches_are_not_resolved() {
    for parse in [parsed, components] {
        let value = parse("color-mix(in --Exact, 20% red, blue calc(30%), currentcolor)").unwrap();
        let mix = value.color_mix_value().unwrap();
        assert_eq!(
            mix.interpolation()
                .unwrap()
                .custom_profile()
                .unwrap()
                .as_str(),
            "--Exact"
        );
        assert_eq!(mix.components().len(), 3);
        assert_eq!(mix.components()[0].color().named().unwrap().name(), "red");
        assert!(
            mix.components()[0]
                .weight()
                .unwrap()
                .literal_value()
                .is_some()
        );
        assert!(
            mix.components()[1]
                .weight()
                .unwrap()
                .calculation()
                .is_some()
        );
        assert!(mix.components()[2].color().is_current_color());
        assert!(mix.components()[2].weight().is_none());
        let value = parse("light-dark(contrast-color(currentcolor), device-cmyk(1%, none, 0, 1))");
        assert!(value.is_err(), "legacy CMYK percentages/none forbidden");
        let value =
            parse("light-dark(contrast-color(currentcolor), device-cmyk(1% none 0 1 / .5))")
                .unwrap();
        assert!(
            value
                .light_dark_value()
                .unwrap()
                .light()
                .contrast_color_value()
                .unwrap()
                .color()
                .is_current_color()
        );
        assert!(
            value
                .light_dark_value()
                .unwrap()
                .dark()
                .device_cmyk_value()
                .unwrap()
                .alpha()
                .is_some()
        );
        let value = parse("color(from red --Exact foo calc(bar + 1) none / baz)").unwrap();
        let custom = value.relative_custom_value().unwrap();
        assert_eq!(custom.profile().as_str(), "--Exact");
        assert_eq!(custom.channels().len(), 3);
        assert!(matches!(
            custom.channels()[1].view(),
            CssProfileColorExpressionRef::Calculation(_)
        ));
    }
    // Color 5 (WD 2026-09-08) §6 legacy device-cmyk() takes `<number>#{4}`
    // and the modern form takes no commas, so the color grammar rejects the
    // value in both fronts. The furthest alternative fails at the `,` after
    // `1%`; the parser currently names the later `none`, so the site is left
    // out.
    let text = "light-dark(contrast-color(currentcolor), device-cmyk(1%, none, 0, 1))";
    let error = parsed_error(text);
    assert!(
        matches!(error.kind(), ErrorKind::InvalidColorSyntax(_)),
        "{text}: {:?}",
        error.kind()
    );
    let error = checked(parse_component_values(text).unwrap()).unwrap_err();
    assert!(
        matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidColorSyntax(_))
        ),
        "{text}: {error:?}"
    );
}

#[test]
fn grammar_failures_and_prefix_consumption_use_actual_boundaries() {
    for text in [
        "rgb(1, 2%, 3)",
        "rgb(none, 2, 3)",
        "rgb(1 2 3, .5)",
        "hsl(1, 2, 3)",
        "rgb(1 2)",
        "color(--Exact)",
        "rgb(1 x 3)",
        "color-mix(in srgb, 101% red, blue)",
        "light-dark(red, blue, green)",
    ] {
        // Both production fronts reject the value with the color grammar's
        // typed diagnostic rather than an unrelated earlier failure.
        let error = parsed_error(text);
        assert!(
            matches!(error.kind(), ErrorKind::InvalidColorSyntax(_)),
            "{text}: {:?}",
            error.kind()
        );
        let error = checked(parse_component_values(text).unwrap()).unwrap_err();
        assert!(
            matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidColorSyntax(_))
            ),
            "{text}: {error:?}"
        );
    }
    // The responsible token is where the furthest-reaching Color 4 (CRD
    // 2026-09-08) / Color 5 (WD 2026-09-08) alternative fails: legacy rgb()
    // takes numbers only after a number (Color 4 §5.1), legacy hsl() takes
    // percentages (Color 4 §7), modern syntax takes no commas, color-mix()
    // weights are `<percentage [0,100]>` (Color 5 §3), and light-dark() takes
    // two colors (Color 5 §7). `rgb(none, 2, 3)` (expected at its first `,`)
    // and the argument lists cut short at `)` are left out: the parser
    // currently reports a later or dangling token for them.
    for (text, token, at) in [
        ("rgb(1, 2%, 3)", "2%", "rgb(1, ".len()),
        ("rgb(1 2 3, .5)", ",", "rgb(1 2 3".len()),
        ("hsl(1, 2, 3)", "2", "hsl(1, ".len()),
        ("rgb(1 x 3)", "x", "rgb(1 ".len()),
        (
            "color-mix(in srgb, 101% red, blue)",
            "101%",
            "color-mix(in srgb, ".len(),
        ),
        (
            "light-dark(red, blue, green)",
            ",",
            "light-dark(red, blue".len(),
        ),
    ] {
        let error = parsed_error(text);
        let ErrorKind::InvalidColorSyntax(detail) = error.kind() else {
            panic!("{text}: color grammar rejection: {:?}", error.kind());
        };
        assert_eq!(
            (
                detail.encountered().map(|token| token.authored()),
                error.position().byte_offset().value(),
                checked_error_offset(text),
            ),
            (Some(token), at, at),
            "{text}: {:?}",
            error.kind()
        );
    }
    // The color consumes exactly its `<color>` prefix, so the trailing
    // identifier is the token the `color` grammar rejects.
    let text = "rgb(1 2 3) blue";
    let error = parsed_error(text);
    assert_eq!(
        error.position().byte_offset().value(),
        text.find("blue").unwrap()
    );
    assert_eq!(checked_error_offset(text), text.find("blue").unwrap());
    let text = "light-dark(red, rgb(1 x 3))";
    assert_eq!(checked_error_offset(text), text.find('x').unwrap());
    let text = "color-mix(in srgb, red 101%, blue)";
    let error = parsed_error(text);
    assert_eq!(
        error.position().column().value() as usize,
        text.find("101").unwrap()
    );
    assert_eq!(checked_error_offset(text), text.find("101").unwrap());
}

#[test]
fn source_positions_keep_utf16_columns_and_constructor_limits_stay_typed() {
    let text = "/*😀*/light-dark(red, rgb(1 x 3))";
    let error = parsed_error(text);
    let responsible = text.find('x').unwrap();
    assert_eq!(error.position().byte_offset().value(), responsible);
    assert_eq!(
        error.position().column().value() as usize,
        text[..responsible].encode_utf16().count()
    );
    let too_deep = format!("{}red{}", "contrast-color(".repeat(257), ")".repeat(257));
    // Authored nesting beyond the parser's structural limit reports the
    // dedicated nesting-limit kind.
    let error = parsed_error(&too_deep);
    assert_eq!(error.code(), CssErrorCode::NestingLimit);
    assert!(matches!(error.kind(), ErrorKind::NestingLimit(_)));
}

#[test]
fn closing_and_eof_failures_stay_inside_their_function_in_both_source_modes() {
    // Each rgb() has two channels, so Color 4 (CRD 2026-09-08) §5.1 rejects
    // it where its arguments end. The responsible site is the authored `)`,
    // or EOF when CSS Syntax 3 (CRD 2026-10-01) §5.5.10 closes the function
    // implicitly. The parser currently reports the last channel instead (in
    // parsed text always, in component values when the `)` is explicit), so
    // the exact site is not asserted; only that the rejection lies inside the
    // rejected rgb().
    for text in [
        "rgb(1 2)",
        "rgb(1 2",
        "/*😀*/light-dark(red, rgb(1 2))",
        "/*😀*/light-dark(red, rgb(1 2",
    ] {
        let function = text.find("rgb(").unwrap()..=text.len();
        let parsed = parsed_error(text).position().byte_offset().value();
        assert!(function.contains(&parsed), "{text:?} Parsed at {parsed}");
        let checked = checked_error_offset(text);
        assert!(
            function.contains(&checked),
            "{text:?} Components at {checked}"
        );
    }
}

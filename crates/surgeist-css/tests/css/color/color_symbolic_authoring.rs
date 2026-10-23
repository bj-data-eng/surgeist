//! Authored `<color>` forms (Color 4 CRD 2026-09-08, Color 5 WD 2026-09-08) stay
//! symbolic through the public stylesheet parser, and invalid forms are
//! rejected by the color grammar at their responsible token.

use surgeist_color::{ColorSpace, HueInterpolation};
use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

/// The authored token a color rejection names.
#[derive(Clone, Copy, Debug)]
enum ResponsibleToken {
    /// The first token the grammar cannot accept.
    Token(CssTokenKind, &'static str),
    /// The function's arguments end before its grammar completes. The
    /// specifications name no responsible token for a truncated argument
    /// list, so only the diagnostic's placement inside the value is required.
    Truncated,
}

/// Typed evidence that `color` rejected `value` by the color grammar: the
/// declaration alone is dropped with an `InvalidColorSyntax` diagnostic (the
/// documented kind for invalid authored color syntax) that names the
/// responsible authored token at that token's source offset.
fn assert_color_rejected(value: &str, responsible: ResponsibleToken) {
    let input = format!(".panel {{ color: {value}; }}");
    let error = assert_drops_sole_declaration(&input);
    let ErrorKind::InvalidColorSyntax(detail) = error.kind() else {
        panic!("{input}: not a color syntax rejection: {error:?}");
    };
    let offset = error.position().byte_offset().value();
    match responsible {
        ResponsibleToken::Token(kind, spelling) => {
            let token = detail
                .encountered()
                .unwrap_or_else(|| panic!("{input}: responsible token"));
            assert_eq!(
                (token.kind(), token.authored()),
                (kind, spelling),
                "{input}"
            );
            assert_eq!(
                offset,
                input.find(spelling).unwrap(),
                "{input}: diagnostic offset is the responsible token"
            );
        }
        ResponsibleToken::Truncated => {
            let start = input.find(value).unwrap();
            assert!(
                (start..start + value.len()).contains(&offset),
                "{input}: diagnostic at {offset} lies inside the color value"
            );
        }
    }
}

#[test]
fn parses_authored_absolute_color_forms() {
    let cases = [
        ("red", "named"),
        ("rebeccapurple", "named"),
        ("transparent", "transparent"),
        ("currentcolor", "currentcolor"),
        ("#abcd", "hex"),
        ("#11223344", "hex"),
        ("rgb(255 0 0 / 50%)", "rgb"),
        ("rgba(255, 0, 0, 0.5)", "rgb"),
        ("hsl(120deg 100% 25% / 0.75)", "hsl"),
        ("hwb(90 10% 20% / 0.8)", "hwb"),
        ("lab(50% 20 -30 / 0.9)", "lab"),
        ("lch(60% 40 120deg)", "lch"),
        ("oklab(0.6 0.1 -0.1)", "oklab"),
        ("oklch(0.7 0.2 240deg / 80%)", "oklch"),
        ("color(display-p3 0.8 0.2 0.1 / 0.9)", "color"),
        ("color(display-p3-linear 0.8 0.2 0.1 / 0.9)", "color"),
    ];

    for (value, expected_kind) in cases {
        let css = format!(".panel {{ color: {value}; }}");
        let declaration = declaration(&css, CssKnownProperty::Color);
        let CssKnownPropertyValueRef::Color(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("color property");
        };
        let color = value.value();
        assert_eq!(color.kind_name(), expected_kind, "{css}");
    }
}

#[test]
fn parses_css_system_colors_symbolically() {
    let color = property_payload!(
        declaration(".panel { color: CanvasText; }", CssKnownProperty::Color),
        Color,
        value
    );
    assert_eq!(color.system(), Some(CssSystemColor::CanvasText));

    let color = property_payload!(
        declaration(
            ".panel { background-color: Canvas; }",
            CssKnownProperty::BackgroundColor
        ),
        BackgroundColor,
        value
    );
    assert_eq!(color.system(), Some(CssSystemColor::Canvas));

    let color = property_payload!(
        declaration(
            ".panel { border-color: AccentColor; }",
            CssKnownProperty::BorderColor
        ),
        BorderColor,
        value
    );
    assert_eq!(
        color.authored_values()[0].system(),
        Some(CssSystemColor::AccentColor)
    );

    let color = property_payload!(
        declaration(
            ".panel { outline-color: HighlightText; }",
            CssKnownProperty::OutlineColor
        ),
        OutlineColor,
        value
    );
    assert_eq!(
        color.color().expect("explicit outline color").system(),
        Some(CssSystemColor::HighlightText)
    );
}

#[test]
fn rejects_unknown_system_color_like_identifiers() {
    // Color 4 (CRD 2026-09-08) §6.2 enumerates <system-color>; neither name
    // is a system color nor a <named-color>.
    for name in ["MadeUpSystemColor", "PlatformAccent"] {
        assert_color_rejected(name, ResponsibleToken::Token(CssTokenKind::Ident, name));
    }
}

#[test]
fn parses_color_mix_symbolically() {
    let color = property_payload!(
        declaration(
            ".panel { color: color-mix(in oklch, red 40%, blue); }",
            CssKnownProperty::Color
        ),
        Color,
        value
    );
    let mix = color.color_mix_value().expect("color-mix");

    assert_eq!(
        mix.interpolation().unwrap().predefined().unwrap().space(),
        ColorSpace::Oklch
    );
    assert_eq!(
        mix.interpolation().unwrap().predefined().unwrap().hue(),
        None
    );
    assert_eq!(mix.components().len(), 2);
    assert_eq!(
        mix.components()[0]
            .weight()
            .unwrap()
            .literal_value()
            .unwrap()
            .literal()
            .numeric()
            .representation(),
        "40"
    );
    assert_eq!(mix.components()[0].color().named().unwrap().name(), "red");
    assert!(mix.components()[1].weight().is_none());
    assert_eq!(mix.components()[1].color().named().unwrap().name(), "blue");
}

#[test]
fn parses_color_mix_with_hue_interpolation() {
    let color = property_payload!(
        declaration(
            ".panel { color: color-mix(in lch longer hue, red, blue 25%); }",
            CssKnownProperty::Color
        ),
        Color,
        value
    );
    let mix = color.color_mix_value().expect("color-mix");

    assert_eq!(
        mix.interpolation().unwrap().predefined().unwrap().space(),
        ColorSpace::Lch
    );
    assert_eq!(
        mix.interpolation().unwrap().predefined().unwrap().hue(),
        Some(HueInterpolation::Longer)
    );
    assert_eq!(
        mix.components()[1]
            .weight()
            .unwrap()
            .literal_value()
            .unwrap()
            .literal()
            .numeric()
            .representation(),
        "25"
    );
}

#[test]
fn rejects_invalid_color_mix_forms_strictly() {
    // Color 5 (WD 2026-09-08) §3: color-mix( <color-interpolation-method>? ,
    // [ <color> && <percentage [0,100]>? ]# ). The list needs one <color>,
    // `in` takes a color space, and each item takes at most one percentage.
    for (value, responsible) in [
        ("color-mix()", ResponsibleToken::Truncated),
        (
            "color-mix(in unknown, red, blue)",
            ResponsibleToken::Token(CssTokenKind::Ident, "unknown"),
        ),
        (
            "color-mix(in oklch, red 10% 20%, blue)",
            ResponsibleToken::Token(CssTokenKind::Percentage, "20%"),
        ),
    ] {
        assert_color_rejected(value, responsible);
    }
}

#[test]
fn parses_relative_colors_symbolically() {
    let color = property_payload!(
        declaration(
            ".panel { color: rgb(from red r g b / alpha); }",
            CssKnownProperty::Color
        ),
        Color,
        value
    );
    let relative = color.relative_value().expect("relative color");

    assert_eq!(relative.function(), &CssRelativeColorFunction::Rgb);
    assert_eq!(relative.source().named().unwrap().name(), "red");
    assert_eq!(relative.channels().len(), 3);
    assert!(matches!(
        relative.channels()[0].value(),
        CssRelativeColorExpressionValue::Channel(CssRelativeColorChannel::R)
    ));
    assert_eq!(
        relative.channels()[0].result_domain(),
        CssRelativeColorResultDomain::NumberPercentage
    );
    assert!(matches!(
        relative.alpha().unwrap().value(),
        CssRelativeColorExpressionValue::Channel(CssRelativeColorChannel::Alpha)
    ));
    assert_eq!(
        relative.alpha().unwrap().result_domain(),
        CssRelativeColorResultDomain::Alpha
    );
}

#[test]
fn parses_relative_oklch_with_component_expressions() {
    let color = property_payload!(
        declaration(
            ".panel { color: oklch(from red l c calc(h + 20) / 80%); }",
            CssKnownProperty::Color
        ),
        Color,
        value
    );
    let relative = color.relative_value().expect("relative color");

    assert_eq!(relative.function(), &CssRelativeColorFunction::Oklch);
    assert_eq!(relative.channels().len(), 3);
    let CssRelativeColorExpressionValue::Calculation(calculation) = relative.channels()[2].value()
    else {
        panic!("expected symbolic hue expression");
    };
    assert_eq!(calculation.authored().as_css(), "calc(h + 20)");
}

#[test]
fn rejects_invalid_relative_color_forms_strictly() {
    // Color 5 (WD 2026-09-08) §4.2 and §5.1: a relative rgb() or hsl() takes
    // exactly three channel values and an optional `/ <alpha-value>`; a
    // channel is a number, percentage, `none`, a channel keyword of that
    // function, or a math function over those keywords (Values 4 §10).
    use ResponsibleToken::{Token, Truncated};
    for (value, responsible) in [
        ("rgb(from red r g)", Truncated),
        ("rgb(from red r g b /)", Truncated),
        (
            "rgb(from red r g b extra)",
            Token(CssTokenKind::Ident, "extra"),
        ),
        ("hsl(from red h s)", Truncated),
        (
            "rgb(from red foo(r) g b)",
            Token(CssTokenKind::Function, "foo("),
        ),
        // calc() needs a <calc-sum> (Values 4 §10.1), so its arguments are truncated.
        ("rgb(from red calc() g b)", Truncated),
        // `h` is not an rgb() channel keyword.
        (
            "rgb(from red calc(h +) g b)",
            Token(CssTokenKind::Ident, "h"),
        ),
    ] {
        assert_color_rejected(value, responsible);
    }
}

#[test]
fn rgba_hex_alpha_preserves_channels() {
    let color = property_payload!(
        declaration(".panel { color: #11223344; }", CssKnownProperty::Color),
        Color,
        value
    );
    assert_eq!(color.hex_value().unwrap().digits(), "11223344");
}

#[test]
fn preserves_finite_css_color_coefficients_beyond_machine_range() {
    for css in [
        ".panel { color: hsl(1e999 100% 50%); }",
        ".panel { color: hwb(1e999 10% 20%); }",
        ".panel { color: lab(50% 1e999 0); }",
        ".panel { color: lch(50% 1e999 0); }",
        ".panel { color: oklab(0.5 1e999 0); }",
        ".panel { color: oklch(0.5 1e999 0); }",
        ".panel { color: color(display-p3 1e999 0 0 / 1); }",
    ] {
        let declaration = declaration(css, CssKnownProperty::Color);
        let CssKnownPropertyValueRef::Color(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("color property");
        };
        let color = value.value();
        let literal = if let Some(hsl) = color.hsl_value() {
            let CssColorHue::Number(literal) = hsl.hue() else {
                panic!("exact HSL hue");
            };
            literal
        } else if let Some(hwb) = color.hwb_value() {
            let CssColorHue::Number(literal) = hwb.hue() else {
                panic!("exact HWB hue");
            };
            literal
        } else {
            let channel = if let Some(lab) = color.lab_value().or(color.oklab_value()) {
                lab.a()
            } else if let Some(lch) = color.lch_value().or(color.oklch_value()) {
                lch.chroma()
            } else {
                &color.predefined_value().unwrap().channels()[0]
            };
            let CssColorComponent::Number(literal) = channel else {
                panic!("exact color channel");
            };
            literal
        };
        assert_eq!(literal.numeric().representation(), "1e999", "{css}");
    }
}

#[test]
fn color_surface_accepts_expanded_strict_forms() {
    for css in [
        ".panel { color: color-mix(in oklch, red); }",
        ".panel { color: red; }",
        ".panel { background-color: rebeccapurple; }",
        ".panel { border-color: #11223344; }",
        ".panel { outline-color: rgb(255 0 0 / 50%); }",
        ".panel { text-decoration-color: oklch(0.7 0.2 240deg / 80%); }",
        ".panel { box-shadow: 0 1px 2px color(display-p3 0.8 0.2 0.1 / 0.9); }",
        ".panel { color: CanvasText; }",
        ".panel { color: color-mix(in oklch, red 40%, blue); }",
        ".panel { color: rgb(from red r g b / alpha); }",
    ] {
        parse_sheet(css).unwrap_or_else(|error| panic!("{css} should parse: {error:?}"));
    }
}

#[test]
fn color_surface_rejects_invalid_forms_without_recovery() {
    // Color 4 (CRD 2026-09-08) §5.1: legacy rgb() separates every channel
    // with a comma; §10.1: color() takes a predefined color space or a
    // dashed-ident profile. The remaining forms are covered above.
    use ResponsibleToken::{Token, Truncated};
    for (value, responsible) in [
        ("rgb(1, 2 3)", Token(CssTokenKind::Number, "3")),
        (
            "color(unknown-space 1 0 0)",
            Token(CssTokenKind::Ident, "unknown-space"),
        ),
        (
            "MadeUpSystemColor",
            Token(CssTokenKind::Ident, "MadeUpSystemColor"),
        ),
        ("color-mix()", Truncated),
        ("rgb(from red r g)", Truncated),
        ("rgb(from red r g b /)", Truncated),
    ] {
        assert_color_rejected(value, responsible);
    }
}

#[test]
fn color_model_preserves_authored_hex_and_currentcolor() {
    let hex = CssHexColor::try_new("ff800080").expect("eight hex digits");
    let color = CssColor::from_hex(hex);
    assert_eq!(color.hex_value().unwrap().digits(), "ff800080");
    assert_eq!(
        color.serialize_specified(Default::default()).unwrap(),
        "rgba(255, 128, 0, 0.5)"
    );
    assert!(CssColor::current_color().is_current_color());
    assert_eq!(
        CssColor::current_color()
            .serialize_specified(Default::default())
            .unwrap(),
        "currentcolor"
    );
}

#[test]
fn color_model_rejects_invalid_hex_and_named_tokens() {
    for invalid in ["ff", "fffff", "ggg", "#fff"] {
        assert!(CssHexColor::try_new(invalid).is_none());
    }
    assert_eq!(CssNamedColor::try_new("ReD").unwrap().name(), "red");
    assert!(CssNamedColor::try_new("MadeUpSystemColor").is_none());
}

#[test]
fn symbolic_color_model_rejects_invalid_mix_weights_and_component_counts() {
    // Color 5 (WD 2026-09-08) §3: each mix weight is <percentage [0,100]>.
    for invalid in ["-0.1%", "100.1%", "1e999%"] {
        assert_eq!(
            CssColorMixPercentage::try_from_component(
                CssComponentValue::try_token(invalid).unwrap()
            )
            .unwrap_err()
            .kind(),
            CssColorScalarErrorKind::OutOfRange,
            "{invalid}"
        );
    }
    assert_eq!(
        CssColorMix::try_new(None, Vec::new()),
        Err(CssColorMixConstructionError::EmptyComponents)
    );
    assert_eq!(
        CssCustomColor::try_new(
            CssColorProfileName::try_new("--brand").unwrap(),
            Vec::new(),
            None
        ),
        Err(CssColorConstructionError::EmptyComponents)
    );
}

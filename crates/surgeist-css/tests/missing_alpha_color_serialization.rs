#![forbid(unsafe_code)]
//! Color 4 CRD 2026-09-08 §16.2.2 chooses a missing-preserving sRGB form
//! for any missing component, including alpha. Explicit values below use exact
//! no-tie scales; grammar reentry is additional evidence, not the text oracle.
//! Color 5 WD 2026-09-08 keeps direct origins in their authored domains, while
//! ordinary mix children still use ordinary specified serialization.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn components(source: &str) -> CssComponentValues {
    parse_component_values(source).unwrap()
}

fn declaration(source: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        components(source),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("{source}: {error:?}"))
}

fn declared_color(value: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) = value.known().unwrap().property_value().unwrap()
    else {
        panic!("checked color property")
    };
    value.value()
}

fn color(source: &str) -> CssColor {
    declared_color(&declaration(source)).clone()
}

fn assert_value(value: &CssColor, expected: &str) {
    let before = value.clone();
    let text = value.to_specified_css();
    assert_eq!(text, value.to_specified_css());
    assert_eq!(value, &before);
    assert_eq!(text.unwrap(), expected);
    // A independently required canonical value must also be admitted by the
    // real property grammar; this does not assert equality across color spaces.
    let reparsed = declaration(expected);
    assert_eq!(
        declared_color(&reparsed).to_specified_css().unwrap(),
        expected
    );
}

fn assert_text(source: &str, expected: &str) {
    let checked = declaration(source);
    let report = parse_style_attribute(&format!("color:{source}!important"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    for value in [&checked, &report.syntax()[0]] {
        let before = value.clone();
        assert_eq!(value.importance(), CssImportance::Important);
        assert_eq!(
            value.value_components().serialize().unwrap().as_css(),
            source
        );
        assert_value(declared_color(value), expected);
        assert_eq!(value, &before);
    }
}

macro_rules! text_case {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            assert_text($source, $expected);
        }
    };
}

text_case!(
    rgb_alpha_none,
    "rgb(255 0 0 / none)",
    "color(srgb 1 0 0 / none)"
);
text_case!(
    rgba_modern_alpha_none,
    "rgba(255 0 0 / none)",
    "color(srgb 1 0 0 / none)"
);
text_case!(
    rgb_percentage_alpha_none,
    "rgb(50% 0 0 / none)",
    "color(srgb 0.5 0 0 / none)"
);
text_case!(
    hsl_percentage_alpha_none,
    "hsl(0 100% 50% / none)",
    "hsl(0 100% 50% / none)"
);
text_case!(
    hsla_number_alpha_none,
    "hsla(0 100 50 / none)",
    "hsl(0 100% 50% / none)"
);
text_case!(
    hsl_turn_alpha_none,
    "hsl(1turn 100 50 / none)",
    "hsl(0 100% 50% / none)"
);
text_case!(
    hwb_percentage_alpha_none,
    "hwb(0 0% 0% / none)",
    "hwb(0 0% 0% / none)"
);
text_case!(
    hwb_turn_number_alpha_none,
    "hwb(.25turn 20 30 / none)",
    "hwb(90 20% 30% / none)"
);
text_case!(
    uppercase_rgba_alpha_none,
    "RGBA(255 0 0 / NoNe)",
    "color(srgb 1 0 0 / none)"
);
text_case!(
    uppercase_hsla_alpha_none,
    "HSLA(0 100 50 / NONE)",
    "hsl(0 100% 50% / none)"
);
text_case!(
    rgb_exact_percentage_channels,
    "rgb(50% 25% 100% / none)",
    "color(srgb 0.5 0.25 1 / none)"
);
text_case!(
    rgb_exact_number_channels,
    "rgb(127.5 63.75 255 / none)",
    "color(srgb 0.5 0.25 1 / none)"
);
text_case!(
    hsl_negative_hue_alpha_none,
    "hsl(-90 20 30 / none)",
    "hsl(270 20% 30% / none)"
);
text_case!(
    hwb_negative_hue_alpha_none,
    "hwb(-90 20 30 / none)",
    "hwb(270 20% 30% / none)"
);

fn number(source: &str, programmatic: bool) -> CssColorComponent {
    let token = if programmatic {
        CssComponentValue::try_number(source).unwrap()
    } else {
        components(source).items()[0].clone()
    };
    CssColorComponent::Number(CssColorNumberLiteral::try_from_component(token).unwrap())
}

fn hue(source: &str, programmatic: bool) -> CssColorHue {
    let CssColorComponent::Number(value) = number(source, programmatic) else {
        unreachable!()
    };
    CssColorHue::Number(value)
}

fn checked_rgb(programmatic: bool, alpha: Option<CssColorComponent>) -> CssColor {
    CssColor::from_rgb(
        CssRgbColor::try_new(
            CssColorSyntax::Modern,
            [
                number("255", programmatic),
                number("0", programmatic),
                number("0", programmatic),
            ],
            alpha,
        )
        .unwrap(),
    )
}

#[test]
fn checked_rgb_missing_alpha_retains_parsed_channel_provenance() {
    let value = checked_rgb(false, Some(CssColorComponent::None));
    let rgb = value.rgb_value().unwrap();
    assert_eq!(rgb.syntax(), CssColorSyntax::Modern);
    assert_eq!(rgb.alpha(), Some(&CssColorComponent::None));
    let CssColorComponent::Number(red) = &rgb.channels()[0] else {
        panic!("number");
    };
    assert_eq!(red.numeric().representation(), "255");
    let CssValueOrigin::Parsed(origin) = red.origin() else {
        panic!("parsed channel");
    };
    assert_eq!(origin.source().as_str(), "255");
    assert_value(&value, "color(srgb 1 0 0 / none)");
}

#[test]
fn programmatic_rgb_missing_alpha_keeps_programmatic_channels() {
    let value = checked_rgb(true, Some(CssColorComponent::None));
    let CssColorComponent::Number(red) = &value.rgb_value().unwrap().channels()[0] else {
        panic!("number");
    };
    assert_eq!(red.origin(), &CssValueOrigin::Programmatic);
    assert_value(&value, "color(srgb 1 0 0 / none)");
}

#[test]
fn checked_hsl_number_channels_gain_percentage_suffixes() {
    let value = CssColor::from_hsl(
        CssHslColor::try_new(
            CssColorSyntax::Modern,
            hue("0", false),
            number("100", false),
            number("50", false),
            Some(CssColorComponent::None),
        )
        .unwrap(),
    );
    assert_eq!(
        value.hsl_value().unwrap().alpha(),
        Some(&CssColorComponent::None)
    );
    assert_value(&value, "hsl(0 100% 50% / none)");
}

#[test]
fn programmatic_hsl_number_channels_gain_percentage_suffixes() {
    let value = CssColor::from_hsl(
        CssHslColor::try_new(
            CssColorSyntax::Modern,
            hue("0", true),
            number("100", true),
            number("50", true),
            Some(CssColorComponent::None),
        )
        .unwrap(),
    );
    assert_value(&value, "hsl(0 100% 50% / none)");
}

#[test]
fn checked_hwb_number_channels_gain_percentage_suffixes() {
    let value = CssColor::from_hwb(
        CssHwbColor::try_new(
            hue("90", false),
            number("20", false),
            number("30", false),
            Some(CssColorComponent::None),
        )
        .unwrap(),
    );
    assert_eq!(
        value.hwb_value().unwrap().alpha(),
        Some(&CssColorComponent::None)
    );
    assert_value(&value, "hwb(90 20% 30% / none)");
}

#[test]
fn programmatic_hwb_number_channels_gain_percentage_suffixes() {
    let value = CssColor::from_hwb(
        CssHwbColor::try_new(
            hue("90", true),
            number("20", true),
            number("30", true),
            Some(CssColorComponent::None),
        )
        .unwrap(),
    );
    assert_value(&value, "hwb(90 20% 30% / none)");
}

#[test]
fn parsed_none_alpha_kind_and_raw_tokens_survive_repeated_serialization() {
    let source = "RGB(255 0 0 / NoNe)";
    let declared = declaration(source);
    let before = declared.clone();
    let rgb = declared_color(&declared).rgb_value().unwrap();
    assert_eq!(rgb.alpha(), Some(&CssColorComponent::None));
    assert_eq!(declared.value_components(), &components(source));
    let CssValueOrigin::Parsed(origin) = declared.value_components().items()[0].origin() else {
        panic!("parsed function");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_value(declared_color(&declared), "color(srgb 1 0 0 / none)");
    assert_eq!(declared, before);
    assert_eq!(
        declared.value_components().serialize().unwrap().as_css(),
        source
    );
}

text_case!(
    ordinary_mix_rgb_child,
    "color-mix(rgb(255 0 0 / none), blue)",
    "color-mix(color(srgb 1 0 0 / none), blue)"
);
text_case!(
    ordinary_mix_hsl_child,
    "color-mix(hsla(0 100 50 / none), blue)",
    "color-mix(hsl(0 100% 50% / none), blue)"
);
text_case!(
    ordinary_mix_hwb_child,
    "color-mix(hwb(0 0 0 / none), blue)",
    "color-mix(hwb(0 0% 0% / none), blue)"
);
text_case!(
    nested_mix_missing_alpha_children,
    "color-mix(color-mix(rgb(255 0 0 / none), blue), hwb(0 0 0 / none))",
    "color-mix(color-mix(color(srgb 1 0 0 / none), blue), hwb(0 0% 0% / none))"
);
text_case!(
    origin_mix_rgb_child,
    "alpha(from color-mix(rgb(255 0 0 / none), blue))",
    "alpha(from color-mix(color(srgb 1 0 0 / none), blue))"
);
text_case!(
    origin_mix_hsl_child,
    "alpha(from color-mix(hsl(0 100 50 / none), blue))",
    "alpha(from color-mix(hsl(0 100% 50% / none), blue))"
);
text_case!(
    origin_mix_hwb_child,
    "alpha(from color-mix(hwb(0 0 0 / none), blue))",
    "alpha(from color-mix(hwb(0 0% 0% / none), blue))"
);
text_case!(
    missing_alpha_child_with_calculated_weight,
    "color-mix(rgb(255 0 0 / none) calc(.0078125%), blue)",
    "color-mix(color(srgb 1 0 0 / none) calc(0.007813%), blue)"
);

#[test]
fn important_checked_declaration_and_validation_keep_sibling_properties() {
    let source = "color:hsla(0 100 50 / none)!important;opacity:.5";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(&validate_style_attribute(source).unwrap(), report.syntax());
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(report.syntax()[0].importance(), CssImportance::Important);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    let before = report.syntax().clone();
    assert_value(
        declared_color(&report.syntax()[0]),
        "hsl(0 100% 50% / none)",
    );
    assert_eq!(report.syntax(), &before);
}

#[test]
fn omitted_direct_unity_and_calculated_alpha_are_distinct_from_none() {
    let omitted = checked_rgb(true, None);
    let missing = checked_rgb(true, Some(CssColorComponent::None));
    let calculated = CssColorComponent::NumberCalculation(
        CssNumberCalculation::try_from_components(components("calc(1 / 128)")).unwrap(),
    );
    let retained = checked_rgb(true, Some(calculated.clone()));
    assert_eq!(omitted.rgb_value().unwrap().alpha(), None);
    assert_eq!(
        missing.rgb_value().unwrap().alpha(),
        Some(&CssColorComponent::None)
    );
    assert_eq!(retained.rgb_value().unwrap().alpha(), Some(&calculated));
    assert_ne!(omitted, missing);
    assert_value(&omitted, "rgb(255, 0, 0)");
    assert_value(
        &checked_rgb(true, Some(number("1", true))),
        "rgb(255, 0, 0)",
    );
    assert_value(&retained, "rgba(255, 0, 0, 0.007813)");
}

#[test]
fn nonmissing_ordinary_colors_keep_legacy_conversion_and_alpha_policy() {
    for (source, expected) in [
        ("rgb(255 0 0)", "rgb(255, 0, 0)"),
        ("rgba(255, 0, 0, .5)", "rgba(255, 0, 0, 0.5)"),
        ("hsl(0 100 50 / 1)", "rgb(255, 0, 0)"),
        ("hwb(0 0 0 / .5)", "rgba(255, 0, 0, 0.5)"),
        ("rgb(255 0 0 / 200%)", "rgb(255, 0, 0)"),
        ("rgb(255 0 0 / calc(2))", "rgb(255, 0, 0)"),
        ("hsl(0 100 50 / calc(1 / 128))", "rgba(255, 0, 0, 0.007813)"),
        ("hwb(0 0 0 / calc(.78125%))", "rgba(255, 0, 0, 0.007813)"),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn existing_missing_nonalpha_forms_keep_none_and_percentage_domains() {
    for (source, expected) in [
        ("rgb(none 0 255)", "color(srgb none 0 1)"),
        ("rgb(none 0 255 / none)", "color(srgb none 0 1 / none)"),
        ("hsl(none 100 50)", "hsl(none 100% 50%)"),
        ("hsl(none 100 50 / none)", "hsl(none 100% 50% / none)"),
        ("hwb(none 20 30)", "hwb(none 20% 30%)"),
        ("hwb(none 20 30 / none)", "hwb(none 20% 30% / none)"),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn direct_origins_relative_and_custom_domains_keep_separate_serialization() {
    for (source, expected) in [
        (
            "alpha(from rgb(255 0 0 / none))",
            "alpha(from rgb(255 0 0 / none))",
        ),
        (
            "alpha(from hsl(1turn 100 50 / none))",
            "alpha(from hsl(360deg 100 50 / none))",
        ),
        (
            "alpha(from hwb(.25turn 20 30 / none))",
            "alpha(from hwb(90deg 20 30 / none))",
        ),
        ("rgb(from red r g b / none)", "rgb(from red r g b / none)"),
        ("hsl(from red h s l / none)", "hsl(from red h s l / none)"),
        ("hwb(from red h w b / none)", "hwb(from red h w b / none)"),
        ("color(--P 0 / none)", "color(--P 0 / none)"),
        ("color(srgb 1 0 0 / none)", "color(srgb 1 0 0 / none)"),
        (
            "color-mix(red calc(.0078125%), blue)",
            "color-mix(red calc(0.007813%), blue)",
        ),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn serialized_missing_alpha_reenters_real_property_grammar() {
    for source in [
        "rgb(255 0 0 / none)",
        "hsl(0 100 50 / none)",
        "hwb(0 0 0 / none)",
    ] {
        let value = color(source);
        let text = value.to_specified_css().unwrap();
        let result = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            components(&text),
            CssImportance::Normal,
        );
        assert!(result.is_ok(), "{source} serialized as {text}: {result:?}");
    }
}

fn assert_error(value: &CssColor, limits: Limits, kind: Kind) {
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, &before);
    assert_eq!(result.unwrap_err().kind(), kind);
}

#[test]
fn zero_input_projection_and_byte_limits_keep_root_precedence() {
    for source in [
        "rgb(255 0 0 / none)",
        "hsl(0 100 50 / none)",
        "hwb(0 0 0 / none)",
    ] {
        let value = color(source);
        // The color root charges input, then projection before dispatch.
        for (limits, kind) in [
            (Limits::new(0, usize::MAX, usize::MAX), Kind::InputNodeLimit),
            (Limits::new(1, 0, usize::MAX), Kind::ProjectionNodeLimit),
            (Limits::new(0, 0, 0), Kind::InputNodeLimit),
            (Limits::new(1, 0, 0), Kind::ProjectionNodeLimit),
            (Limits::new(usize::MAX, usize::MAX, 0), Kind::ByteLimit),
        ] {
            assert_error(&value, limits, kind);
        }
    }
}

macro_rules! byte_case {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            let value = color($source);
            let before = value.clone();
            let limits = Limits::new(usize::MAX, usize::MAX, $expected.len());
            let result = value.to_specified_css_with_limits(limits);
            assert_eq!(result, value.to_specified_css_with_limits(limits));
            assert_eq!(value, before);
            assert_eq!(result.unwrap(), $expected);
            assert_error(
                &value,
                Limits::new(usize::MAX, usize::MAX, $expected.len() - 1),
                Kind::ByteLimit,
            );
        }
    };
}

byte_case!(
    rgb_exact_utf8_output_bytes,
    "rgb(255 0 0 / none)",
    "color(srgb 1 0 0 / none)"
);
byte_case!(
    hsl_exact_utf8_output_bytes,
    "hsl(0 100 50 / none)",
    "hsl(0 100% 50% / none)"
);
byte_case!(
    hwb_exact_utf8_output_bytes,
    "hwb(0 0 0 / none)",
    "hwb(0 0% 0% / none)"
);

#[test]
fn border_pair_siblings_share_final_bytes_and_spent_prefix() {
    let pair = CssBorderColorPair::new(
        color("rgb(255 0 0 / none)"),
        Some(color("hsl(0 100 50 / none)")),
    );
    let expected = "color(srgb 1 0 0 / none) hsl(0 100% 50% / none)";
    let before = pair.clone();
    let limits = Limits::new(usize::MAX, usize::MAX, expected.len());
    let result = pair.serialize_specified_with_limits(limits);
    assert_eq!(result, pair.serialize_specified_with_limits(limits));
    assert_eq!(pair, before);
    assert_eq!(result.unwrap(), expected);
    let error = pair
        .serialize_specified_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len() - 1))
        .unwrap_err();
    assert_eq!(error.kind(), Kind::ByteLimit);
    assert_eq!(pair, before);
}

#[test]
fn border_pair_input_limit_is_cumulative_across_literal_colors() {
    let pair = CssBorderColorPair::new(
        color("rgb(255 0 0 / none)"),
        Some(color("hsl(0 100 50 / none)")),
    );
    let before = pair.clone();
    // Pair root: one input. Each color: root, three literal channels, missing
    // alpha = five inputs. Literal exact-rational work adds no input visits.
    assert_eq!(
        pair.serialize_specified_with_limits(Limits::new(10, usize::MAX, usize::MAX))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(pair, before);
    assert_eq!(
        pair.serialize_specified_with_limits(Limits::new(11, usize::MAX, usize::MAX))
            .unwrap(),
        "color(srgb 1 0 0 / none) hsl(0 100% 50% / none)"
    );
    assert_eq!(pair, before);
}

#[test]
fn rgb_one_below_required_preserving_form_bytes_is_atomic() {
    assert_error(
        &color("rgb(255 0 0 / none)"),
        Limits::new(usize::MAX, usize::MAX, "color(srgb 1 0 0 / none)".len() - 1),
        Kind::ByteLimit,
    );
}

#[test]
fn hsl_one_below_required_preserving_form_bytes_is_atomic() {
    assert_error(
        &color("hsl(0 100 50 / none)"),
        Limits::new(usize::MAX, usize::MAX, "hsl(0 100% 50% / none)".len() - 1),
        Kind::ByteLimit,
    );
}

#[test]
fn hwb_one_below_required_preserving_form_bytes_is_atomic() {
    assert_error(
        &color("hwb(0 0 0 / none)"),
        Limits::new(usize::MAX, usize::MAX, "hwb(0 0% 0% / none)".len() - 1),
        Kind::ByteLimit,
    );
}

#[test]
fn utf8_caller_prefix_and_missing_alpha_share_exact_final_bytes() {
    // The non-ASCII custom profile is a valid CSS identifier. Its UTF-8 bytes,
    // the separator, and the RGB child's preserving form share one byte budget.
    let pair = CssBorderColorPair::new(color("color(--é 0)"), Some(color("rgb(255 0 0 / none)")));
    let expected = "color(--é 0) color(srgb 1 0 0 / none)";
    let before = pair.clone();
    let limits = Limits::new(usize::MAX, usize::MAX, expected.len());
    let result = pair.serialize_specified_with_limits(limits);
    assert_eq!(result, pair.serialize_specified_with_limits(limits));
    assert_eq!(pair, before);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn utf8_caller_prefix_leaves_one_below_missing_alpha_final_bytes() {
    let pair = CssBorderColorPair::new(color("color(--é 0)"), Some(color("rgb(255 0 0 / none)")));
    let before = pair.clone();
    let limits = Limits::new(
        usize::MAX,
        usize::MAX,
        "color(--é 0) color(srgb 1 0 0 / none)".len() - 1,
    );
    let result = pair.serialize_specified_with_limits(limits);
    assert_eq!(result, pair.serialize_specified_with_limits(limits));
    assert_eq!(pair, before);
    assert_eq!(result.unwrap_err().kind(), Kind::ByteLimit);
}

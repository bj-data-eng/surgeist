//! Public standalone used-value phases, independent of the authored grammar join.

use surgeist_color::{
    Color, ColorSpace, PredefinedColorSpace, SerializationRequest, Transformation,
};
use surgeist_css::{
    CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInput,
    CssColorOperationError, CssColorOrdinaryError, CssColorOrdinaryLimits, CssColorOrdinaryOutcome,
    CssColorOrdinaryResource, CssColorPaletteEntry, CssColorPropertyFacts, CssColorSchemeFact,
    CssColorStandardFailureKind, CssColorUnavailable, CssDeviceColorFact, CssSystemColor,
    parse_used_color,
};

fn limits() -> CssColorOrdinaryLimits {
    // Explicit ordinary logical allowances for these short literal fixtures.
    // They are neither a physical allocator bound nor an HTML work ledger.
    CssColorOrdinaryLimits {
        input_units: 1024,
        algorithm_records: 16_384,
        arithmetic_records: 4096,
        evaluation_records: 4096,
        actions: 4_000_000,
    }
}
fn numerical_limits() -> surgeist_color::SerializationLimits {
    surgeist_color::SerializationLimits {
        actions: 4_000_000,
        digits: 4096,
        output_bytes: 256,
    }
}
fn absent_facts() -> CssColorFacts<'static> {
    CssColorFacts::new(
        CssColorFactIdentity::new(7, 3),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    )
}
fn value(input: &str, facts: &CssColorFacts<'_>) -> Color {
    let result = parse_used_color(
        CssColorInput::Utf8(input.as_bytes()),
        CssColorGrammarRequest::WholeColor,
        facts,
        facts.identity(),
        limits(),
    )
    .unwrap();
    match result.outcome() {
        CssColorOrdinaryOutcome::Used(value) => value,
        other => panic!("expected intrinsic color for {input}: {other:?}"),
    }
}

#[test]
fn equal_absolute_relative_and_mixed_channels_preserve_selected_serialization() {
    // Color4 absolute sRGB and Color5 relative/mix serialization have different
    // forms even for exactly equal finite channels; no byte proof is inferred.
    let facts = absent_facts();
    for (input, expected) in [
        ("rgb(255 0 0)", "rgb(255, 0, 0)"),
        ("hsl(0 100% 50%)", "rgb(255, 0, 0)"),
        ("hwb(0 0% 0%)", "rgb(255, 0, 0)"),
        ("rgb(from red r g b)", "color(srgb 1 0 0)"),
        ("hsl(from red h s l)", "color(srgb 1 0 0)"),
        ("hwb(from red h w b)", "color(srgb 1 0 0)"),
        ("color-mix(in hsl, red, red)", "color(srgb 1 0 0)"),
        ("color-mix(in hwb, red, red)", "color(srgb 1 0 0)"),
        ("color-mix(in srgb, red, blue)", "color(srgb 0.5 0 0.5)"),
    ] {
        let actual = value(input, &facts);
        assert_eq!(actual.srgb_bytes(), None, "{input}");
        assert_eq!(
            surgeist_color::serialize(actual, SerializationRequest::Css, numerical_limits())
                .unwrap(),
            expected,
            "{input}"
        );
    }
}

#[test]
fn equal_floating_and_byte_alpha_keep_their_distinct_serialization_policies() {
    let facts = absent_facts();
    for (byte, floating, expected_byte, expected_float) in [
        (
            236,
            "rgb(0 0 0 / 0.9254901960784314)",
            "rgba(0, 0, 0, 0.925)",
            "rgba(0, 0, 0, 0.92549)",
        ),
        (
            237,
            "rgb(0 0 0 / 0.9294117647058824)",
            "rgba(0, 0, 0, 0.93)",
            "rgba(0, 0, 0, 0.929412)",
        ),
    ] {
        let byte = Color::from_srgba8([0, 0, 0, byte]);
        let floating = value(floating, &facts);
        assert_eq!(byte.alpha(), floating.alpha());
        assert_eq!(
            surgeist_color::serialize(byte, SerializationRequest::Css, numerical_limits()).unwrap(),
            expected_byte,
        );
        assert_eq!(
            surgeist_color::serialize(floating, SerializationRequest::Css, numerical_limits())
                .unwrap(),
            expected_float,
        );
    }
}

#[test]
fn initial_current_color_reaches_canvas_text_and_only_the_selected_scheme_child() {
    let id = CssColorFactIdentity::new(7, 3);
    let resource = CssColorFactIdentity::new(8, 4);
    let white = Color::from_srgba8([255; 4]);
    let entries = [CssColorPaletteEntry::new(
        CssSystemColor::CanvasText,
        resource,
        white,
    )];
    let facts = CssColorFacts::new(
        id,
        CssColorPropertyFacts::InitialProperties,
        CssColorSchemeFact::Light,
        &entries,
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    assert_eq!(value("currentcolor", &facts), white);
    assert_eq!(
        value("light-dark(red, currentcolor)", &facts).srgb_bytes(),
        Some([255, 0, 0])
    );
    let missing = CssColorFacts::new(
        id,
        CssColorPropertyFacts::InitialProperties,
        CssColorSchemeFact::Light,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    let error = parse_used_color(
        CssColorInput::Utf8(b"currentcolor"),
        CssColorGrammarRequest::WholeColor,
        &missing,
        id,
        limits(),
    )
    .err()
    .unwrap();
    assert_eq!(
        error,
        CssColorOrdinaryError::Operation(CssColorOperationError::Unavailable(
            CssColorUnavailable::Palette(CssSystemColor::CanvasText)
        ))
    );
    assert_eq!(
        value("light-dark(red, currentcolor)", &missing).srgb_bytes(),
        Some([255, 0, 0])
    );
}

#[test]
fn missing_components_and_alpha_are_preserved_until_explicit_transform() {
    let facts = absent_facts();
    let missing = value("hsl(from red none s l / none)", &facts);
    assert_eq!(missing.channels(), [None, Some(100.0), Some(50.0)]);
    assert_eq!(missing.alpha(), None);
    assert_eq!(
        surgeist_color::serialize(missing, SerializationRequest::Css, numerical_limits()).unwrap(),
        "hsl(none 100% 50% / none)"
    );
    let present = value("rgb(100% 0 0 / .5)", &facts);
    let opaque = present
        .transformed(Transformation::ForceOpaqueAlpha)
        .unwrap();
    assert_eq!(present.alpha(), Some(0.5));
    assert_eq!(opaque.alpha(), Some(1.0));
    assert_eq!(opaque.srgb_bytes(), None);
    let bytes = opaque.transformed(Transformation::QuantizeSrgb).unwrap();
    assert_eq!(bytes.srgb_bytes(), Some([255, 0, 0]));
    assert_eq!(
        surgeist_color::serialize(
            bytes,
            SerializationRequest::HtmlCompatible,
            numerical_limits()
        )
        .unwrap(),
        "#ff0000"
    );
}

#[test]
fn standard_invalidity_cannot_be_confused_with_logical_admission_or_reached_facts() {
    let facts = absent_facts();
    let invalid = parse_used_color(
        CssColorInput::Utf8(b"rgb(1deg 0 0)"),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        limits(),
    )
    .unwrap();
    assert!(
        matches!(invalid.outcome(), CssColorOrdinaryOutcome::StandardFailure(failure)
        if failure.kind == CssColorStandardFailureKind::ComponentGrammar)
    );
    let mut denied = limits();
    denied.algorithm_records = 0;
    assert_eq!(
        parse_used_color(
            CssColorInput::Utf8(b"red"),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
            denied
        )
        .err()
        .unwrap(),
        CssColorOrdinaryError::Limit {
            resource: CssColorOrdinaryResource::AlgorithmRecords,
            required: 1,
            allowed: 0
        }
    );
    let error = parse_used_color(
        CssColorInput::Utf8(b"currentcolor"),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        limits(),
    )
    .err()
    .unwrap();
    assert_eq!(
        error,
        CssColorOrdinaryError::Operation(CssColorOperationError::Unavailable(
            CssColorUnavailable::CurrentColor
        ))
    );
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn utf16_prefix_input_and_native_conversion_retain_actual_units_and_phase_meaning() {
    let facts = absent_facts();
    let input = [114, 101, 100, 32, 98, 108, 117, 101];
    let result = parse_used_color(
        CssColorInput::Utf16(&input),
        CssColorGrammarRequest::PrefixColor,
        &facts,
        facts.identity(),
        limits(),
    )
    .unwrap();
    let CssColorInput::Utf16(retained) = result.input() else {
        panic!("original UTF16")
    };
    assert!(std::ptr::eq(retained, input.as_slice()));
    assert_eq!(result.source().consumed.start, 0);
    assert_eq!(result.source().consumed.end, 3);
    assert_eq!(
        result.outcome(),
        CssColorOrdinaryOutcome::Used(Color::from_srgba8([255, 0, 0, 255]))
    );
    let modern = Color::try_new(
        ColorSpace::Predefined(PredefinedColorSpace::Srgb),
        [Some(1.0), Some(0.0), Some(0.0)],
        Some(1.0),
    )
    .unwrap();
    let cylindrical = modern
        .transformed(Transformation::Convert(ColorSpace::Hwb))
        .unwrap();
    assert_eq!(cylindrical.channels(), [Some(0.0); 3]);
    assert_eq!(
        surgeist_color::serialize(cylindrical, SerializationRequest::Css, numerical_limits())
            .unwrap(),
        "color(srgb 1 0 0)"
    );
}

#[test]
fn complete_cylindrical_parse_only_results_publish_srgb_before_output() {
    // Color4 15.1; Color5 11.1/11.3. Red is independent of any serializer.
    let facts = absent_facts();
    for (text, modern) in [
        ("hsl(0 100% 50%)", false),
        ("hwb(0 0% 0%)", false),
        ("hsl(from red h s l)", true),
        ("hwb(from red h w b)", true),
        ("color-mix(in hsl, red, red)", true),
        ("color-mix(in hwb, red, red)", true),
    ] {
        let result = parse_used_color(
            CssColorInput::Utf8(text.as_bytes()),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
            limits(),
        )
        .unwrap();
        let CssColorOrdinaryOutcome::Used(color) = result.outcome() else {
            panic!("used color")
        };
        assert!(std::ptr::eq(result.facts(), &facts));
        assert_eq!(result.source().consumed.start, 0);
        assert_eq!(result.source().consumed.end, text.len());
        assert_eq!(color.alpha(), Some(1.0));
        assert_eq!(
            color.srgb_bytes(),
            None,
            "floating conversion is not byte provenance"
        );
        assert_eq!(
            color.space(),
            ColorSpace::Predefined(PredefinedColorSpace::Srgb),
            "{text}"
        );
        assert_eq!(
            color.channels(),
            [Some(1.0), Some(0.0), Some(0.0)],
            "{text}"
        );
        assert_eq!(
            (color.serialization_family() == surgeist_color::SerializationFamily::ColorFunction),
            modern,
            "{text}"
        );
    }
}

#[test]
fn missing_cylindrical_channels_or_alpha_retain_their_natural_used_space() {
    // Color4 16.2.2 and Color5 11.3 preserve every none, including alpha.
    let facts = absent_facts();
    for (text, space, channels, alpha) in [
        (
            "hsl(none 100% 50%)",
            ColorSpace::Hsl,
            [None, Some(100.0), Some(50.0)],
            Some(1.0),
        ),
        (
            "hsl(0 100% 50% / none)",
            ColorSpace::Hsl,
            [Some(0.0), Some(100.0), Some(50.0)],
            None,
        ),
        (
            "hwb(0 none 0%)",
            ColorSpace::Hwb,
            [Some(0.0), None, Some(0.0)],
            Some(1.0),
        ),
        ("hwb(0 0% 0% / none)", ColorSpace::Hwb, [Some(0.0); 3], None),
        (
            "hsl(from red none s l)",
            ColorSpace::Hsl,
            [None, Some(100.0), Some(50.0)],
            Some(1.0),
        ),
        (
            "hsl(from red h s l / none)",
            ColorSpace::Hsl,
            [Some(0.0), Some(100.0), Some(50.0)],
            None,
        ),
        (
            "hwb(from red h none b)",
            ColorSpace::Hwb,
            [Some(0.0), None, Some(0.0)],
            Some(1.0),
        ),
        (
            "hwb(from red h w b / none)",
            ColorSpace::Hwb,
            [Some(0.0); 3],
            None,
        ),
        (
            "color-mix(in hsl, hsl(0 100% 50% / none), hsl(0 100% 50% / none))",
            ColorSpace::Hsl,
            [Some(0.0), Some(100.0), Some(50.0)],
            None,
        ),
        (
            "color-mix(in hwb, hwb(0 0% 0% / none), hwb(0 0% 0% / none))",
            ColorSpace::Hwb,
            [Some(0.0); 3],
            None,
        ),
    ] {
        let color = value(text, &facts);
        assert_eq!(color.space(), space, "{text}");
        assert_eq!(color.channels(), channels, "{text}");
        assert_eq!(color.alpha(), alpha, "{text}");
        assert_eq!(color.srgb_bytes(), None, "{text}");
    }
}

#[test]
fn nested_cylindrical_origins_keep_hue_and_signed_channels_until_root_completion() {
    let facts = absent_facts();
    // An achromatic inner color still supplies the authored 120-degree hue.
    // The signed inner saturation must be read before any cyan round trip.
    for text in [
        "hsl(from hsl(120 0% 50%) h 100% l)",
        "hsl(from hsl(from red 0 -100 50) 120 calc(s * -1) l)",
    ] {
        let color = value(text, &facts);
        assert_eq!(
            color.space(),
            ColorSpace::Predefined(PredefinedColorSpace::Srgb)
        );
        assert_eq!(
            color.channels(),
            [Some(0.0), Some(1.0), Some(0.0)],
            "{text}"
        );
        assert!(
            (color.serialization_family() == surgeist_color::SerializationFamily::ColorFunction)
        );
    }
    // Color4's HSL conversion explicitly permits out-of-gamut results.
    for (text, modern) in [("hsl(0 200% 50%)", false), ("hsl(from red 0 200 50)", true)] {
        let color = value(text, &facts);
        assert_eq!(
            color.space(),
            ColorSpace::Predefined(PredefinedColorSpace::Srgb)
        );
        assert_eq!(color.channels(), [Some(1.5), Some(-0.5), Some(-0.5)]);
        assert_eq!(
            (color.serialization_family() == surgeist_color::SerializationFamily::ColorFunction),
            modern
        );
        assert_eq!(color.srgb_bytes(), None);
    }
}

#[test]
fn wrapper_and_utf16_prefix_results_complete_only_the_final_used_color() {
    let id = CssColorFactIdentity::new(7, 3);
    let facts = CssColorFacts::new(
        id,
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Light,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    for (text, alpha) in [
        ("light-dark(hsl(0 100% 50%), blue)", 1.0),
        ("alpha(from hwb(0 0% 0%) / .5)", 0.5),
    ] {
        let color = value(text, &facts);
        assert_eq!(
            color.space(),
            ColorSpace::Predefined(PredefinedColorSpace::Srgb)
        );
        assert_eq!(color.channels(), [Some(1.0), Some(0.0), Some(0.0)]);
        assert_eq!(color.alpha(), Some(alpha));
        assert_eq!(color.srgb_bytes(), None);
    }
    let input = [
        104, 115, 108, 40, 48, 32, 49, 48, 48, 37, 32, 53, 48, 37, 41, 32, 98, 108, 117, 101,
    ];
    let result = parse_used_color(
        CssColorInput::Utf16(&input),
        CssColorGrammarRequest::PrefixColor,
        &facts,
        id,
        limits(),
    )
    .unwrap();
    let CssColorInput::Utf16(original) = result.input() else {
        panic!("UTF16 source")
    };
    assert!(std::ptr::eq(original, input.as_slice()));
    assert_eq!(result.source().consumed.start, 0);
    assert_eq!(result.source().consumed.end, 15);
    let CssColorOrdinaryOutcome::Used(color) = result.outcome() else {
        panic!("used prefix")
    };
    assert_eq!(
        color.space(),
        ColorSpace::Predefined(PredefinedColorSpace::Srgb)
    );
    assert_eq!(color.channels(), [Some(1.0), Some(0.0), Some(0.0)]);
}

#[test]
fn numerical_failure_reports_reached_nested_recipe_in_original_utf16_units() {
    // The impossible finite conversion occurs while resolving the inner mix.
    // A leading supplementary character in a comment distinguishes original
    // UTF-16 units from UTF-8 bytes without changing the CSS recipe.
    let facts = absent_facts();
    let inner = "color-mix(in srgb, lab(0 1e308 0), red)";
    let prefix = "/*🦀*/color-mix(in srgb, ";
    let text = format!("{prefix}{inner}, blue)");
    let input: Vec<u16> = text.encode_utf16().collect();
    let mut allowance = limits();
    // Exact conversion of a large authored exponent runs through the original
    // incremental decimal owner before the fixed numerical failure is reached.
    allowance.actions = 100_000_000;
    let error = parse_used_color(
        CssColorInput::Utf16(&input),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        allowance,
    )
    .err()
    .expect("inner conversion cannot produce finite sRGB");
    assert_eq!(
        error,
        CssColorOrdinaryError::Operation(CssColorOperationError::Numerical {
            error: surgeist_color::ColorError::Unrepresentable,
            site: surgeist_css::CssColorResolutionSite::Source(surgeist_css::CssColorInputRange {
                start: prefix.encode_utf16().count(),
                end: prefix.encode_utf16().count() + inner.encode_utf16().count(),
            }),
        })
    );
}

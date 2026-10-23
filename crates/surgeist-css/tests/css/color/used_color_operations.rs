//! Public standalone used-value phases, independent of the authored grammar join.

use surgeist_color::{
    Color, ColorSpace, PredefinedColorSpace, SerializationRequest, Transformation,
};
use surgeist_css::{
    CssColorError, CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInput,
    CssColorOrdinaryLimits, CssColorOrdinaryOutcome, CssColorOrdinaryResource,
    CssColorPaletteEntry, CssColorPropertyFacts, CssColorSchemeFact, CssColorStandardFailureKind,
    CssColorUnavailable, CssDeviceColorFact, CssSystemColor, parse_used_color,
};

/// A finite numerical color from plain test coordinates.
fn numerical_color(space: ColorSpace, channels: [Option<f64>; 3], alpha: Option<f64>) -> Color {
    Color::new(
        space,
        channels.map(|channel| channel.map(|value| surgeist_color::Finite::new(value).unwrap())),
        alpha.map(|value| surgeist_color::UnitInterval::new(value).unwrap()),
    )
}
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
fn authored_negative_zero_is_the_unsigned_zero_in_color_calculations() {
    // Values 4 (WD-20240312 §10.9): "0, +0 and -0 all produce the standard
    // 'unsigned' zero, which is considered positive (0⁺)". So 1 / sign(-0) is
    // +∞, which top-level range checking clamps to the channel maximum. A
    // genuinely negative operand keeps its sign and clamps to the minimum.
    let facts = absent_facts();
    for (input, expected) in [
        ("rgb(0 0 calc(1 / sign(-0)))", "rgb(0, 0, 255)"),
        ("rgb(0 0 calc(1 / sign(+0)))", "rgb(0, 0, 255)"),
        ("rgb(0 0 calc(1 / sign(0)))", "rgb(0, 0, 255)"),
        ("rgb(0 0 calc(1 / sign(-0.000)))", "rgb(0, 0, 255)"),
        ("rgb(0 0 calc(1 / sign(-0e999999)))", "rgb(0, 0, 255)"),
        ("rgb(calc(1 / -0) 0 0)", "rgb(255, 0, 0)"),
        ("rgb(0 0 calc(1 / sign(-1)))", "rgb(0, 0, 0)"),
    ] {
        let actual = value(input, &facts);
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
        error.color_error().and_then(CssColorError::unavailable),
        Some(CssColorUnavailable::Palette(CssSystemColor::CanvasText))
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
    let opaque = present.transformed(Transformation::ForceOpaqueAlpha);
    assert_eq!(present.alpha(), Some(0.5));
    assert_eq!(opaque.alpha(), Some(1.0));
    assert_eq!(opaque.srgb_bytes(), None);
    let bytes = opaque.transformed(Transformation::QuantizeSrgb);
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
    assert!(
        parse_used_color(
            CssColorInput::Utf8(b"red"),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
            denied
        )
        .err()
        .unwrap()
        .exhausted_limit()
            == Some((CssColorOrdinaryResource::AlgorithmRecords, 0, 1))
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
        error.color_error().and_then(CssColorError::unavailable),
        Some(CssColorUnavailable::CurrentColor)
    );
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn malformed_utf8_input_is_a_semantic_color_error_at_its_original_unit() {
    // Encoding §4.1 UTF-8 decoder: 0xFF is never a valid byte, and an input
    // ending inside a two-byte sequence is malformed. The declared UTF-8
    // source is the caller's data, not misuse of the controlled protocol.
    // The offset is the unit whose observation proved the input malformed, or
    // the input length when the input ends inside a sequence. The decoder
    // bounds the byte after 0xE0, 0xED, 0xF0 and 0xF4 (A0..BF, 80..9F, 90..BF,
    // 80..8F), so an overlong form, a surrogate or a code point above
    // U+10FFFF is already malformed at that second byte.
    let facts = absent_facts();
    let cases = [
        (b"r\xffd".as_slice(), 1),
        (b"r\xc3".as_slice(), 2),
        (b"\xe0\x80\x80".as_slice(), 1),
        (b"\xed\xa0\x80".as_slice(), 1),
        (b"\xf0\x80\x80\x80".as_slice(), 1),
        (b"\xf4\x90\x80\x80".as_slice(), 1),
    ];
    let actual = cases.map(|(input, _)| {
        let error = parse_used_color(
            CssColorInput::Utf8(input),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
            limits(),
        )
        .err()
        .expect("malformed UTF-8 has no used color");
        (
            input,
            error
                .color_error()
                .and_then(CssColorError::malformed_input_offset),
        )
    });
    let expected = cases.map(|(input, offset)| (input, Some(offset)));
    assert_eq!(actual, expected);
}

#[test]
fn utf8_sequences_just_inside_each_second_byte_bound_decode() {
    // Encoding §4.1 UTF-8 decoder: A0 after E0, 9F after ED, 90 after F0 and
    // 8F after F4 are the edge bytes the bounds admit (U+0800, U+D7FF,
    // U+10000 and U+10FFFF). Each decodes to one non-ASCII identifier code
    // point, which is not a color: a standard grammar failure, never input
    // malformation.
    let facts = absent_facts();
    for input in [
        b"\xe0\xa0\x80".as_slice(),
        b"\xed\x9f\xbf".as_slice(),
        b"\xf0\x90\x80\x80".as_slice(),
        b"\xf4\x8f\xbf\xbf".as_slice(),
    ] {
        let result = parse_used_color(
            CssColorInput::Utf8(input),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
            limits(),
        )
        .unwrap_or_else(|error| panic!("{input:?}: {error:?}"));
        assert!(
            matches!(
                result.outcome(),
                CssColorOrdinaryOutcome::StandardFailure(failure)
                    if failure.kind == CssColorStandardFailureKind::ColorGrammar
            ),
            "{input:?}: {:?}",
            result.outcome()
        );
        assert_eq!(result.source().consumed.end, input.len(), "{input:?}");
    }
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
    let modern = numerical_color(
        ColorSpace::Predefined(PredefinedColorSpace::Srgb),
        [Some(1.0), Some(0.0), Some(0.0)],
        Some(1.0),
    );
    let cylindrical = modern.transformed(Transformation::Convert(ColorSpace::Hwb));
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
fn nested_overflowing_mix_resolves_to_closest_finite_srgb_in_original_utf16_units() {
    // lab(0 1e308 0) has X far beyond binary64 and Y = Z = 0; its D50-adapted
    // linear sRGB column is (+, -, +), so Color 4 conversion saturates it to
    // (MAX, -MAX, MAX). Even mixes with red and then blue halve and quarter
    // those channels exactly. A leading supplementary character in a comment
    // distinguishes original UTF-16 units from UTF-8 bytes.
    let facts = absent_facts();
    let inner = "color-mix(in srgb, lab(0 1e308 0), red)";
    let prefix = "/*🦀*/color-mix(in srgb, ";
    let text = format!("{prefix}{inner}, blue)");
    let input: Vec<u16> = text.encode_utf16().collect();
    let mut allowance = limits();
    // Exact conversion of a large authored exponent runs through the original
    // incremental decimal owner.
    allowance.actions = 100_000_000;
    let result = parse_used_color(
        CssColorInput::Utf16(&input),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        allowance,
    )
    .unwrap();
    assert_eq!(result.source().consumed.start, 0);
    assert_eq!(result.source().consumed.end, input.len());
    let CssColorOrdinaryOutcome::Used(value) = result.outcome() else {
        panic!("a finite used color")
    };
    assert_eq!(
        value.space(),
        ColorSpace::Predefined(PredefinedColorSpace::Srgb)
    );
    assert_eq!(
        value.channels(),
        [
            Some(f64::MAX / 4.0),
            Some(-f64::MAX / 4.0),
            Some(f64::MAX / 4.0)
        ]
    );
    assert_eq!(value.alpha(), Some(1.0));
}

#[test]
fn used_hues_are_normalized_into_the_half_open_degree_range() {
    // Color 4 CRD 2026-09-08 §4.3: a hue "is normalized to the range [0,360)";
    // hsl(360 0 0) and hsl(calc(±infinity) 0 0) normalize to 0 degrees. Values 4
    // makes -0 the unsigned zero. A negative hue smaller in magnitude than half
    // an ulp of 360 therefore normalizes to 0, never to the excluded 360.
    let facts = absent_facts();
    for hue in [
        "-1e-15",
        "calc(-1e-15)",
        "-0",
        "0",
        "360",
        "-360",
        "720",
        "360deg",
        "1turn",
        "-400grad",
        "calc(infinity)",
        "calc(-infinity)",
    ] {
        for (input, space, index) in [
            (format!("hsl({hue} none 50%)"), ColorSpace::Hsl, 0),
            (format!("hwb({hue} none 50%)"), ColorSpace::Hwb, 0),
            (format!("lch(50 none {hue})"), ColorSpace::Lch, 2),
            (format!("oklch(0.5 0.1 {hue})"), ColorSpace::Oklch, 2),
            (format!("lch(from red l c {hue})"), ColorSpace::Lch, 2),
            (
                format!("oklch(from red l none {hue})"),
                ColorSpace::Oklch,
                2,
            ),
            (format!("hsl(from red {hue} none l)"), ColorSpace::Hsl, 0),
            (format!("hwb(from red {hue} w none)"), ColorSpace::Hwb, 0),
        ] {
            let actual = value(&input, &facts);
            assert_eq!(actual.space(), space, "{input}");
            let hue = actual.channels()[index].unwrap();
            assert!(hue == 0.0 && hue.is_sign_positive(), "{input}: {hue:?}");
        }
    }
}

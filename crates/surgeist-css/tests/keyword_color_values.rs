#![forbid(unsafe_code)]
//! Fixed keyword expectations from Color 4 CRD 2026-09-08 §§6.1/6.3.
//!
//! Table source: https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#named-colors
//! Source SHA-256: bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba
//! All 148 normative decimal RGB rows were independently extracted and checked
//! against their hex column; the 139 distinct triples preserve keyword aliases.
//! Expected values are independent of the parser, serializer, and cssparser table.

use surgeist_css::{
    CssColor, CssHexColor, CssKnownProperty, CssKnownPropertyValueRef, CssNamedColor,
    CssRecoveryAction, CssSystemColor, parse_style_attribute, validate_style_attribute,
};

const NAMED_COLORS: [(&str, [u8; 3]); 148] = [
    ("aliceblue", [240, 248, 255]),
    ("antiquewhite", [250, 235, 215]),
    ("aqua", [0, 255, 255]),
    ("aquamarine", [127, 255, 212]),
    ("azure", [240, 255, 255]),
    ("beige", [245, 245, 220]),
    ("bisque", [255, 228, 196]),
    ("black", [0, 0, 0]),
    ("blanchedalmond", [255, 235, 205]),
    ("blue", [0, 0, 255]),
    ("blueviolet", [138, 43, 226]),
    ("brown", [165, 42, 42]),
    ("burlywood", [222, 184, 135]),
    ("cadetblue", [95, 158, 160]),
    ("chartreuse", [127, 255, 0]),
    ("chocolate", [210, 105, 30]),
    ("coral", [255, 127, 80]),
    ("cornflowerblue", [100, 149, 237]),
    ("cornsilk", [255, 248, 220]),
    ("crimson", [220, 20, 60]),
    ("cyan", [0, 255, 255]),
    ("darkblue", [0, 0, 139]),
    ("darkcyan", [0, 139, 139]),
    ("darkgoldenrod", [184, 134, 11]),
    ("darkgray", [169, 169, 169]),
    ("darkgreen", [0, 100, 0]),
    ("darkgrey", [169, 169, 169]),
    ("darkkhaki", [189, 183, 107]),
    ("darkmagenta", [139, 0, 139]),
    ("darkolivegreen", [85, 107, 47]),
    ("darkorange", [255, 140, 0]),
    ("darkorchid", [153, 50, 204]),
    ("darkred", [139, 0, 0]),
    ("darksalmon", [233, 150, 122]),
    ("darkseagreen", [143, 188, 143]),
    ("darkslateblue", [72, 61, 139]),
    ("darkslategray", [47, 79, 79]),
    ("darkslategrey", [47, 79, 79]),
    ("darkturquoise", [0, 206, 209]),
    ("darkviolet", [148, 0, 211]),
    ("deeppink", [255, 20, 147]),
    ("deepskyblue", [0, 191, 255]),
    ("dimgray", [105, 105, 105]),
    ("dimgrey", [105, 105, 105]),
    ("dodgerblue", [30, 144, 255]),
    ("firebrick", [178, 34, 34]),
    ("floralwhite", [255, 250, 240]),
    ("forestgreen", [34, 139, 34]),
    ("fuchsia", [255, 0, 255]),
    ("gainsboro", [220, 220, 220]),
    ("ghostwhite", [248, 248, 255]),
    ("gold", [255, 215, 0]),
    ("goldenrod", [218, 165, 32]),
    ("gray", [128, 128, 128]),
    ("green", [0, 128, 0]),
    ("greenyellow", [173, 255, 47]),
    ("grey", [128, 128, 128]),
    ("honeydew", [240, 255, 240]),
    ("hotpink", [255, 105, 180]),
    ("indianred", [205, 92, 92]),
    ("indigo", [75, 0, 130]),
    ("ivory", [255, 255, 240]),
    ("khaki", [240, 230, 140]),
    ("lavender", [230, 230, 250]),
    ("lavenderblush", [255, 240, 245]),
    ("lawngreen", [124, 252, 0]),
    ("lemonchiffon", [255, 250, 205]),
    ("lightblue", [173, 216, 230]),
    ("lightcoral", [240, 128, 128]),
    ("lightcyan", [224, 255, 255]),
    ("lightgoldenrodyellow", [250, 250, 210]),
    ("lightgray", [211, 211, 211]),
    ("lightgreen", [144, 238, 144]),
    ("lightgrey", [211, 211, 211]),
    ("lightpink", [255, 182, 193]),
    ("lightsalmon", [255, 160, 122]),
    ("lightseagreen", [32, 178, 170]),
    ("lightskyblue", [135, 206, 250]),
    ("lightslategray", [119, 136, 153]),
    ("lightslategrey", [119, 136, 153]),
    ("lightsteelblue", [176, 196, 222]),
    ("lightyellow", [255, 255, 224]),
    ("lime", [0, 255, 0]),
    ("limegreen", [50, 205, 50]),
    ("linen", [250, 240, 230]),
    ("magenta", [255, 0, 255]),
    ("maroon", [128, 0, 0]),
    ("mediumaquamarine", [102, 205, 170]),
    ("mediumblue", [0, 0, 205]),
    ("mediumorchid", [186, 85, 211]),
    ("mediumpurple", [147, 112, 219]),
    ("mediumseagreen", [60, 179, 113]),
    ("mediumslateblue", [123, 104, 238]),
    ("mediumspringgreen", [0, 250, 154]),
    ("mediumturquoise", [72, 209, 204]),
    ("mediumvioletred", [199, 21, 133]),
    ("midnightblue", [25, 25, 112]),
    ("mintcream", [245, 255, 250]),
    ("mistyrose", [255, 228, 225]),
    ("moccasin", [255, 228, 181]),
    ("navajowhite", [255, 222, 173]),
    ("navy", [0, 0, 128]),
    ("oldlace", [253, 245, 230]),
    ("olive", [128, 128, 0]),
    ("olivedrab", [107, 142, 35]),
    ("orange", [255, 165, 0]),
    ("orangered", [255, 69, 0]),
    ("orchid", [218, 112, 214]),
    ("palegoldenrod", [238, 232, 170]),
    ("palegreen", [152, 251, 152]),
    ("paleturquoise", [175, 238, 238]),
    ("palevioletred", [219, 112, 147]),
    ("papayawhip", [255, 239, 213]),
    ("peachpuff", [255, 218, 185]),
    ("peru", [205, 133, 63]),
    ("pink", [255, 192, 203]),
    ("plum", [221, 160, 221]),
    ("powderblue", [176, 224, 230]),
    ("purple", [128, 0, 128]),
    ("rebeccapurple", [102, 51, 153]),
    ("red", [255, 0, 0]),
    ("rosybrown", [188, 143, 143]),
    ("royalblue", [65, 105, 225]),
    ("saddlebrown", [139, 69, 19]),
    ("salmon", [250, 128, 114]),
    ("sandybrown", [244, 164, 96]),
    ("seagreen", [46, 139, 87]),
    ("seashell", [255, 245, 238]),
    ("sienna", [160, 82, 45]),
    ("silver", [192, 192, 192]),
    ("skyblue", [135, 206, 235]),
    ("slateblue", [106, 90, 205]),
    ("slategray", [112, 128, 144]),
    ("slategrey", [112, 128, 144]),
    ("snow", [255, 250, 250]),
    ("springgreen", [0, 255, 127]),
    ("steelblue", [70, 130, 180]),
    ("tan", [210, 180, 140]),
    ("teal", [0, 128, 128]),
    ("thistle", [216, 191, 216]),
    ("tomato", [255, 99, 71]),
    ("turquoise", [64, 224, 208]),
    ("violet", [238, 130, 238]),
    ("wheat", [245, 222, 179]),
    ("white", [255, 255, 255]),
    ("whitesmoke", [245, 245, 245]),
    ("yellow", [255, 255, 0]),
    ("yellowgreen", [154, 205, 50]),
];

fn parsed_color(value: &str) -> CssColor {
    let source = format!("color: {value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    let CssKnownPropertyValueRef::Color(color) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected a typed color: {source}");
    };
    let color = color.value().clone();
    assert_eq!(validate_style_attribute(&source).unwrap(), *report.syntax());
    color
}

fn assert_named(color: &CssColor, name: &str, [red, green, blue]: [u8; 3]) {
    // Observe intrinsic bytes before serialization or any round trip.
    assert_eq!(
        color.keyword_srgba8(),
        Some([red, green, blue, 255]),
        "{name}"
    );
    assert_eq!(color.named().unwrap().name(), name);
    assert!(!color.is_transparent(), "{name}");
    assert!(color.rgb_value().is_none(), "{name}");
    assert!(color.system().is_none(), "{name}");
    assert_eq!(color.to_specified_css().unwrap(), name);
}

fn mixed_case(name: &str) -> String {
    name.chars()
        .enumerate()
        .map(|(index, character)| {
            if index % 2 == 0 {
                character.to_ascii_uppercase()
            } else {
                character
            }
        })
        .collect()
}

#[test]
fn every_named_constructor_exposes_the_normative_opaque_rgb_and_canonical_keyword() {
    for (name, expected) in NAMED_COLORS {
        for spelling in [name.to_owned(), name.to_ascii_uppercase(), mixed_case(name)] {
            let named = CssNamedColor::try_new(&spelling).unwrap();
            assert_eq!(named.name(), name);
            assert_eq!(named, CssNamedColor::try_new(name).unwrap());
            assert_named(&CssColor::from_named(named), name, expected);
        }
    }
}

#[test]
fn every_named_keyword_case_and_escape_has_clean_validation_and_exact_numeric_meaning() {
    for (name, expected) in NAMED_COLORS {
        let escaped = format!("\\{:x} {}", name.as_bytes()[0], &name[1..]);
        for spelling in [
            name.to_owned(),
            name.to_ascii_uppercase(),
            mixed_case(name),
            escaped,
        ] {
            let color = parsed_color(&spelling);
            assert_named(&color, name, expected);
            // Reparse only after independently observing the initial projection.
            let canonical = color.to_specified_css().unwrap();
            assert_eq!(parsed_color(&canonical), color);
        }
    }
}

#[test]
fn equal_rgb_aliases_preserve_distinct_declared_names_and_equality() {
    for (left, right, expected) in [
        ("aqua", "cyan", [0, 255, 255]),
        ("fuchsia", "magenta", [255, 0, 255]),
        ("gray", "grey", [128, 128, 128]),
        ("darkgray", "darkgrey", [169, 169, 169]),
        ("dimgray", "dimgrey", [105, 105, 105]),
        ("lightgray", "lightgrey", [211, 211, 211]),
        ("slategray", "slategrey", [112, 128, 144]),
        ("lightslategray", "lightslategrey", [119, 136, 153]),
        ("darkslategray", "darkslategrey", [47, 79, 79]),
    ] {
        let left_color = CssColor::from_named(CssNamedColor::try_new(left).unwrap());
        let right_color = CssColor::from_named(CssNamedColor::try_new(right).unwrap());
        assert_named(&left_color, left, expected);
        assert_named(&right_color, right, expected);
        assert_ne!(left_color.named(), right_color.named());
        assert_ne!(left_color, right_color);
        assert_eq!(parsed_color(left), left_color);
        assert_eq!(parsed_color(right), right_color);
    }
}

#[test]
fn transparent_is_distinct_transparent_black_before_serialization_in_every_spelling() {
    let direct = CssColor::transparent();
    let opaque_black = CssColor::from_named(CssNamedColor::try_new("black").unwrap());
    for color in [
        direct.clone(),
        parsed_color("transparent"),
        parsed_color("TRANSPARENT"),
        parsed_color("TrAnSpArEnT"),
        parsed_color("\\74 ransparent"),
    ] {
        assert_eq!(color.keyword_srgba8(), Some([0, 0, 0, 0]));
        assert!(color.is_transparent());
        assert!(color.named().is_none());
        assert!(color.rgb_value().is_none());
        assert_eq!(color, direct);
        assert_ne!(color, opaque_black);
        assert_eq!(color.to_specified_css().unwrap(), "transparent");
        assert_eq!(parsed_color(&color.to_specified_css().unwrap()), direct);
    }
    assert_eq!(opaque_black.keyword_srgba8(), Some([0, 0, 0, 255]));
    assert!(CssNamedColor::try_new("transparent").is_none());
}

#[test]
fn unknown_keyword_recovery_keeps_valid_siblings_without_a_numeric_keyword_value() {
    for invalid in [
        "",
        "notacolor",
        "rebeccapurples",
        "alice-blue",
        "gray1",
        "red blue",
    ] {
        assert!(CssNamedColor::try_new(invalid).is_none(), "{invalid}");
        let source = format!("color: {invalid}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
        assert!(validate_style_attribute(&source).is_err(), "{source}");
    }
    for symbolic in ["currentcolor", "Canvas", "ActiveBorder", "transparent"] {
        assert!(CssNamedColor::try_new(symbolic).is_none(), "{symbolic}");
    }
}

#[test]
fn valid_nonkeyword_branches_stay_outside_the_fixed_projection_even_when_numeric() {
    for spelling in [
        "currentcolor",
        "Canvas",
        "ActiveBorder",
        "#ff0000",
        "#0000",
        "rgb(255 0 0)",
        "rgb(0 0 0 / 0)",
        "hsl(0 100% 50%)",
        "hwb(0 0% 0%)",
        "lab(50 3 4)",
        "lch(50 5 30)",
        "oklab(.5 .03 .04)",
        "oklch(.5 .05 30)",
        "color(srgb 1 0 0)",
        "color(--profile 1 0 0)",
        "rgb(from red r g b)",
        "color-mix(in srgb, red, blue)",
        "light-dark(red, blue)",
        "contrast-color(red)",
        "device-cmyk(0 1 1 0)",
        "alpha(from red)",
    ] {
        let color = parsed_color(spelling);
        assert_eq!(color.keyword_srgba8(), None, "{spelling}");
        assert!(color.named().is_none(), "{spelling}");
        assert!(!color.is_transparent(), "{spelling}");
    }
    for direct in [
        CssColor::current_color(),
        CssColor::from_system(CssSystemColor::Canvas),
        CssColor::from_system(CssSystemColor::ActiveBorder),
        CssColor::from_hex(CssHexColor::try_new("ff0000").unwrap()),
    ] {
        assert_eq!(direct.keyword_srgba8(), None);
    }
}

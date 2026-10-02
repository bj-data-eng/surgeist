#![forbid(unsafe_code)]
//! CSS Color 4 CRD 2026-09-08 sections 5.2 and 16.2.2 require implicit unit
//! alpha and the rgb() name for opaque hexadecimal colors. Color 5 origin
//! serialization retains the separate modern punctuation contract.

use surgeist_css::{
    CssColor, CssHexColor, CssKnownPropertyValueRef, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, parse_style_attribute,
};

fn parsed_color(value: &str) -> CssColor {
    let report = parse_style_attribute(&format!("color: {value}"));
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Color(color) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected a color");
    };
    color.value().clone()
}

#[test]
fn parsed_opaque_hex_alpha_uses_rgb_with_an_implicit_alpha() {
    // #123 repeats each nibble: 0x11 = 17, 0x22 = 34, 0x33 = 51.
    // Both the repeated F nibble and the FF byte are exactly opaque.
    for source in ["#123f", "#123F", "#112233ff", "#112233FF"] {
        let color = parsed_color(source);
        assert_eq!(color.hex_value().unwrap().digits(), &source[1..]);
        assert_eq!(color.to_specified_css().unwrap(), "rgb(17, 34, 51)");
    }
}

#[test]
fn constructed_opaque_hex_alpha_uses_the_same_rgb_form() {
    for digits in ["123f", "123F", "112233ff", "112233FF"] {
        let color = CssColor::from_hex(CssHexColor::try_new(digits).unwrap());
        assert_eq!(color.hex_value().unwrap().digits(), digits);
        assert_eq!(color.to_specified_css().unwrap(), "rgb(17, 34, 51)");
    }
}

#[test]
fn opaque_hex_mix_children_use_rgb_without_resolving_the_mix() {
    assert_eq!(
        parsed_color("color-mix(in srgb, #123f, #112233ff)")
            .to_specified_css()
            .unwrap(),
        "color-mix(in srgb, rgb(17, 34, 51), rgb(17, 34, 51))",
    );
}

#[test]
fn opaque_hex_fits_the_exact_rgb_byte_budget_without_mutation() {
    let color = parsed_color("#112233ff");
    let before = color.clone();
    let expected = "rgb(17, 34, 51)";
    assert_eq!(
        color
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                1,
                1,
                expected.len() - 1,
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit,
    );
    assert_eq!(color, before);
    assert_eq!(
        color
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                1,
                1,
                expected.len(),
            ))
            .unwrap(),
        expected,
    );
    assert_eq!(color, before);
}

#[test]
fn absent_and_zero_hex_alpha_keep_their_existing_forms() {
    for (source, expected) in [
        ("#123", "rgb(17, 34, 51)"),
        ("#112233", "rgb(17, 34, 51)"),
        ("#1230", "rgba(17, 34, 51, 0)"),
        ("#11223300", "rgba(17, 34, 51, 0)"),
    ] {
        let parsed = parsed_color(source);
        let constructed = CssColor::from_hex(CssHexColor::try_new(&source[1..]).unwrap());
        assert_eq!(parsed.to_specified_css().unwrap(), expected);
        assert_eq!(constructed.to_specified_css().unwrap(), expected);
    }
}

#[test]
fn hexadecimal_origins_keep_modern_punctuation_and_alpha() {
    for (source, expected) in [
        ("alpha(from #123f)", "alpha(from rgb(17 34 51))"),
        ("alpha(from #112233ff)", "alpha(from rgb(17 34 51))"),
        ("alpha(from #1230)", "alpha(from rgb(17 34 51 / 0))"),
        ("rgb(from #112233ff r g b)", "rgb(from rgb(17 34 51) r g b)"),
    ] {
        assert_eq!(parsed_color(source).to_specified_css().unwrap(), expected);
    }
}

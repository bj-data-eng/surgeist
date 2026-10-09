#![forbid(unsafe_code)]
//! Selected Color4 CRD2026-09-08 §13.2 requires gamma-sRGB for legacy colors
//! by default; an explicit interpolation method opts into a different space.
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#interpolation-space
//! This qualifies Images4 WD2025-09-30 §§3.5.2 and8: dropping explicit Oklab
//! with legacy stop colors changes meaning despite the general default prose.
use surgeist_css::*;

const FUNCTIONS: [&str; 5] = [
    "linear-gradient",
    "radial-gradient",
    "repeating-linear-gradient",
    "repeating-radial-gradient",
    "conic-gradient",
];

fn image_css(function: &str, method: &str, colors: &str) -> String {
    let css = format!("{function}(in {method}, {colors})");
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
        parse_component_values(&css).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::BackgroundImage(images) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("background image")
    };
    let [image] = images.images().images() else {
        panic!("one gradient")
    };
    image.serialize_specified().unwrap()
}

#[test]
fn explicit_oklab_with_legacy_colors_survives_image_and_declaration_serialization() {
    for function in FUNCTIONS {
        for colors in [
            "red, blue",
            "#f00, #00f",
            "rgb(255 0 0), rgb(0 0 255)",
            "rgba(255,0,0,.5), rgba(0,0,255,.5)",
            "hsl(0 100% 50%), hsl(240 100% 50%)",
            "hwb(0 0% 0%), hwb(240 0% 0%)",
        ] {
            let css = format!("{function}(in oklab, {colors})");
            let declaration = parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
                parse_component_values(&css).unwrap(),
                CssImportance::Important,
            )
            .unwrap();
            let before = declaration.clone();
            let CssKnownPropertyValueRef::BackgroundImage(images) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("background image")
            };
            let [image] = images.images().images() else {
                panic!("one gradient")
            };
            let specified = image.serialize_specified().unwrap();
            assert!(
                specified.starts_with(&format!("{function}(in oklab, ")),
                "{specified}"
            );
            let checked = CssImage::try_new(image.clone()).unwrap();
            assert_eq!(checked.serialize_specified().unwrap(), specified);
            let output = declaration.to_specified_css().unwrap();
            assert!(output.contains("in oklab"), "{output}");
            let report = parse_style_attribute(&output);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            assert_eq!(report.syntax()[0].to_specified_css().unwrap(), output);
            assert_eq!(declaration, before);
        }
    }
}

#[test]
fn matching_oklab_is_omitted_when_a_stable_nonlegacy_stop_proves_the_default() {
    // Color4 §13.2: at least one nonlegacy stop opts into the Oklab default.
    // Images4 §8 mandates omission without changing that meaning. Mixed
    // legacy/nonlegacy and separately unresolved siblings use the same proof.
    for function in FUNCTIONS {
        for modern in [
            "lab(50% 20 30)",
            "lch(50% 20 30)",
            "oklab(50% .2 .3)",
            "oklch(50% .2 30)",
            "color(srgb 1 0 0)",
            "color(display-p3 1 0 0)",
        ] {
            for colors in [
                format!("{modern}, blue"),
                format!("red, {modern}"),
                format!("red, {modern}, currentcolor"),
            ] {
                let specified = image_css(function, "oklab", &colors);
                assert!(!specified.contains("in oklab"), "{specified}");
                let report = parse_style_attribute(&format!("background-image:{specified}"));
                assert!(report.is_clean(), "{:?}", report.diagnostics());
            }
        }
    }
}

#[test]
fn matching_srgb_is_omitted_for_stable_ordinary_legacy_stops() {
    for function in FUNCTIONS {
        for colors in [
            "red, blue",
            "#f00, #00f",
            "rgb(255 0 0), rgb(0 0 255)",
            "hsl(0 100% 50%), hwb(240 0% 0%)",
        ] {
            let specified = image_css(function, "srgb", colors);
            assert!(!specified.contains("in srgb"), "{specified}");
            let report = parse_style_attribute(&format!("background-image:{specified}"));
            assert!(report.is_clean(), "{:?}", report.diagnostics());
        }
    }
}

#[test]
fn explicit_oklab_remains_when_only_legacy_or_unproved_unresolved_stops_are_present() {
    for function in FUNCTIONS {
        for colors in [
            "currentcolor, blue",
            "CanvasText, red",
            "rgb(from currentcolor r g b), blue",
            "color-mix(in srgb, currentcolor, blue), red",
        ] {
            let specified = image_css(function, "oklab", colors);
            assert!(
                specified.starts_with(&format!("{function}(in oklab, ")),
                "{specified}"
            );
        }
    }
}

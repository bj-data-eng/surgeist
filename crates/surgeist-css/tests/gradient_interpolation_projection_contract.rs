#![forbid(unsafe_code)]
//! Color4 CRD2026-09-08 §13.2 enumerates RGB/HSL/HWB legacy formats,
//! independently of comma-vs-modern syntax (§4.1.2); it has no missing-channel
//! exemption. §16.2.2 serializes missing RGB as color(srgb), a nonlegacy format.
//! Images4 WD2025-09-30 §8 therefore requires retaining the original gradient
//! interpolation meaning across that shared Color projection.
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#interpolation-space
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#serializing-sRGB-values
//! https://www.w3.org/TR/2025/WD-css-images-4-20250930/#serialization
use surgeist_css::*;

const FUNCTIONS: [&str; 5] = [
    "linear-gradient",
    "radial-gradient",
    "repeating-linear-gradient",
    "repeating-radial-gradient",
    "conic-gradient",
];

fn parsed(css: &str) -> (CssDeclaration, CssImageValue) {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
        parse_component_values(css).unwrap(),
        CssImportance::Important,
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
    let image = image.clone();
    (declaration, image)
}

fn verify(css: &str, expected: &str) {
    let (declaration, image) = parsed(css);
    let before = declaration.clone();
    assert_eq!(image.serialize_specified().unwrap(), expected);
    let checked = CssImage::try_new(image.clone()).unwrap();
    assert_eq!(checked.serialize_specified().unwrap(), expected);
    // Exact output capacity succeeds; every failure is reusable and returns no
    // partial string. This exercises the existing public cumulative provider.
    assert_eq!(
        image
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                1000,
                1000,
                expected.len(),
            ))
            .unwrap(),
        expected,
    );
    for (input, projection, bytes, cause) in [
        (
            0,
            1000,
            1000,
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            1000,
            0,
            1000,
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            1000,
            1000,
            expected.len() - 1,
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = image
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                input, projection, bytes,
            ))
            .unwrap_err();
        assert_eq!(error.kind(), cause);
        assert_eq!(image.serialize_specified().unwrap(), expected);
    }
    let output = declaration.to_specified_css().unwrap();
    assert_eq!(output, format!("background-image: {expected} !important;"));
    let report = parse_style_attribute(&output);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax()[0].to_specified_css().unwrap(), output);
    assert_eq!(declaration, before);
}

#[test]
fn implicit_legacy_default_survives_missing_rgb_projection() {
    for function in FUNCTIONS {
        for (rgb, emitted) in [
            ("rgb(none 0 255)", "color(srgb none 0 1)"),
            ("rgb(255 0 none)", "color(srgb 1 0 none)"),
            ("rgb(255 0 0 / none)", "color(srgb 1 0 0 / none)"),
        ] {
            verify(
                &format!("{function}({rgb}, blue)"),
                &format!("{function}(in srgb, {emitted}, blue)"),
            );
        }
    }
}

#[test]
fn explicit_srgb_survives_missing_rgb_projection() {
    for function in FUNCTIONS {
        verify(
            &format!("{function}(in srgb, rgb(none 0 255), blue)"),
            &format!("{function}(in srgb, color(srgb none 0 1), blue)"),
        );
    }
}

#[test]
fn explicit_oklab_matches_the_emitted_nonlegacy_default() {
    for function in FUNCTIONS {
        verify(
            &format!("{function}(in oklab, rgb(none 0 255), blue)"),
            &format!("{function}(color(srgb none 0 1), blue)"),
        );
    }
}

#[test]
fn nonlegacy_sibling_already_proves_implicit_oklab() {
    for function in FUNCTIONS {
        for method in ["", "in oklab, "] {
            verify(
                &format!("{function}({method}rgb(none 0 255), color(srgb 1 0 0))"),
                &format!("{function}(color(srgb none 0 1), color(srgb 1 0 0))"),
            );
        }
    }
}

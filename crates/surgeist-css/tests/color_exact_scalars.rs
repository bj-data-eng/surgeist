#![forbid(unsafe_code)]
//! Existing-boundary RED for exact ordinary color scalar transport.
//! Selected Color4/Color5 grammar plus adopted exact-fidelity product contract.
//! No future exact-payload API; raw serialization is not canonical output.
use surgeist_css::*;

fn wrapper(declaration: &CssDeclaration) -> &CssColorPropertyValue {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary color")
    };
    value
}
fn parsed(text: &str) -> CssDeclaration {
    let source = format!("color:{text}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    let declaration = report.syntax()[0].clone();
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(wrapper(&declaration).as_css(), text);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        text
    );
    let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin() else {
        panic!("original parsed function")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 6);
    declaration
}
fn checked(text: &str) -> CssDeclaration {
    let values = parse_component_values(text).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        values.clone(),
        CssImportance::Normal,
    )
    .expect("finite ordinary authored color must admit via checked property grammar");
    assert_eq!(declaration.value_components(), &values);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_value().is_none());
    declaration
}
fn admission(text: &str) {
    let parsed = parsed(text);
    let checked = checked(text);
    assert!(
        wrapper(&parsed).i01_subset().is_none(),
        "out-of-cache literal cannot project: {text}"
    );
    assert!(
        wrapper(&checked).i01_subset().is_none(),
        "checked out-of-cache literal cannot project: {text}"
    );
}
fn no_projection(text: &str) {
    let parsed = parsed(text);
    let checked = checked(text);
    assert!(
        wrapper(&parsed).i01_subset().is_none(),
        "unproved frozen projection: {text}"
    );
    assert!(
        wrapper(&checked).i01_subset().is_none(),
        "unproved checked frozen projection: {text}"
    );
}

#[test]
fn ordinary_number_channels_admit_large_finite_decimals() {
    admission("rgb(1e100 0 0)");
}
#[test]
fn ordinary_percentage_channels_admit_large_finite_decimals() {
    admission("lab(1e100% 0 0)");
}
#[test]
fn hue_dimensions_admit_large_finite_coefficients() {
    admission("hsl(1e100deg 25% 50%)");
}
#[test]
fn relative_direct_channels_admit_large_finite_decimals() {
    admission("rgb(from red 1e100 g b)");
}
#[test]
fn display_p3_linear_uses_current_none_math_and_exact_channel_admission() {
    admission("color(display-p3-linear none calc(0.5) 1e100)");
}

#[test]
fn finite_current_mix_coefficient_does_not_justify_a_rounded_frozen_weight() {
    // The coefficient is exactly binary32. Dividing by 100 into the tokenizer's
    // normalized cache and multiplying back changes it; finite membership alone
    // cannot prove the frozen weight preserves the authored coefficient.
    no_projection("color-mix(in srgb, red 50.000003814697265625%, blue)");
}
#[test]
fn root_mix_rejects_projection_of_nonbinary32_literal_weight() {
    no_projection("color-mix(in srgb, red 0.1%, blue)");
}
#[test]
fn root_relative_rejects_projection_of_nonbinary32_direct_channel() {
    no_projection("rgb(from red 0.1 g b)");
}
#[test]
fn nested_relative_and_mix_projection_gate_checks_descendants() {
    // Test both root dispatch routes before asserting, so one root's bad pairing
    // does not prevent the other route from being evaluated.
    let texts = [
        "rgb(from color-mix(in srgb, red 0.1%, blue) r g b)",
        "color-mix(in srgb, rgb(from red 0.1 g b), blue)",
    ];
    let declarations = texts.map(parsed);
    assert!(
        declarations
            .iter()
            .all(|declaration| wrapper(declaration).i01_subset().is_none())
    );
    let declarations = texts.map(checked);
    assert!(
        declarations
            .iter()
            .all(|declaration| wrapper(declaration).i01_subset().is_none())
    );
}

fn range_rejected(text: &str) {
    let report = parse_style_attribute(&format!("color:{text};opacity:.5"));
    assert_eq!(report.diagnostics().len(), 1, "{text}");
    assert_eq!(report.syntax().len(), 1, "{text}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            parse_component_values(text).unwrap(),
            CssImportance::Normal
        )
        .is_err(),
        "{text}"
    );
}
#[test]
fn negative_nonzero_mix_weight_is_not_zero_when_cache_underflows() {
    range_rejected("color-mix(in srgb, red -1e-100%, blue)");
}
#[test]
fn mix_weight_above_100_is_not_in_range_when_cache_rounds_down() {
    range_rejected("color-mix(in srgb, red 100.0000000000000000000001%, blue)");
}

#[test]
fn exact_finite_projection_controls_keep_supported_meaning() {
    for text in [
        "hsl(0 25% 50%)",
        "color(srgb 0.5 0 1)",
        "color-mix(in srgb, red 25%, blue 50%)",
    ] {
        for declaration in [parsed(text), checked(text)] {
            assert!(
                wrapper(&declaration).i01_subset().is_some(),
                "safe finite subset: {text}"
            );
        }
    }
    for text in [
        "color-mix(in srgb, red -0%, blue)",
        "color-mix(in srgb, red 100%, blue)",
    ] {
        parsed(text);
        checked(text);
    }
}
#[test]
fn underflow_admission_and_raw_retention_are_controls_not_exact_payload_proof() {
    for text in ["rgb(1e-47 0 0)", "color-mix(in srgb, red 1e-47%, blue)"] {
        parsed(text);
        checked(text);
    }
}

#![forbid(unsafe_code)]

//! Grid track-size admission through the public property boundary.
//! Grid 2 (2025-03-26): https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#typedef-track-size
//! Grid 2 SHA256: 05aa64853c4428146973943b35caf121e44c1076bdf5b8c29f8896dba9b778e2
//! Grid 3 (2026-01-21) intrinsic auto-repeat bodies:
//! https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#intrinsic-auto-repeat
//! Grid 3 SHA256: ab3a5d476f748764136b3ad89f2b4c9c9b47d6d50304aaa5d0a02319b9f5f46b
//! Values 4 (2024-03-12): https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-ranges
//! Values 4 SHA256: 7b6b68c34d00d7f6945e66e4e2efa913266299f597d5825393cae85dac1cc78c
//! Surgeist's exact authored-numeric contract requires range checks before float narrowing.

use surgeist_css::{
    CssErrorCode, CssImportance, CssKnownProperty as Property, CssPropertyNameRef,
    CssRecoveryAction, parse_component_values, parse_property_value, parse_style_attribute,
    validate_style_attribute,
};

const AXES: [Property; 2] = [Property::GridTemplateRows, Property::GridTemplateColumns];
const IMPLICIT: [Property; 2] = [Property::GridAutoRows, Property::GridAutoColumns];

#[test]
fn finite_decimal_track_literals_survive_without_float_narrowing() {
    for property in AXES.into_iter().chain(IMPLICIT) {
        for authored in ["1e50fr", "1e-50fr", "1e50px", "1e-50px", "1e50%", "1e-50%"] {
            assert_accepted(property, authored);
        }
    }

    for (property, authored) in [
        (Property::GridTemplateColumns, "minmax(auto, 1e-50fr)"),
        (Property::GridTemplateRows, "fit-content(1e50px)"),
        (Property::GridAutoRows, "minmax(1e-50%, 1fr)"),
        (Property::GridAutoColumns, "fit-content(1e-50%)"),
        (Property::GridTemplateColumns, "repeat(2, 1e-50fr)"),
        (Property::GridTemplateRows, "repeat(auto-fill, 1e50px)"),
        (Property::GridTemplate, "1e-50fr / 10px"),
        (Property::Grid, "auto-flow 1e50fr / 10px"),
    ] {
        assert_accepted(property, authored);
    }
}

#[test]
fn ordinary_track_range_checks_use_authored_decimal_not_rounded_float() {
    for property in AXES.into_iter().chain(IMPLICIT) {
        for authored in ["-1e-50fr", "-1e-50px", "-1e-50%", "1e-50"] {
            assert_rejected(property, authored);
        }
    }

    for (property, authored) in [
        (Property::GridTemplateColumns, "minmax(auto, -1e-50fr)"),
        (Property::GridTemplateColumns, "minmax(-1e-50px, 1fr)"),
        (Property::GridAutoRows, "fit-content(-1e-50%)"),
        (Property::GridTemplateRows, "repeat(2, -1e-50fr)"),
        (Property::Grid, "auto-flow -1e-50fr / 10px"),
    ] {
        assert_rejected(property, authored);
    }
}

#[test]
fn lexical_zero_and_length_percentage_math_remain_admitted_controls() {
    for property in AXES.into_iter().chain(IMPLICIT) {
        for authored in ["-0fr", "-0px", "-0%", "0"] {
            assert_accepted(property, authored);
        }
        // The length-percentage branch retains its percentage hint.
        assert_accepted(property, "calc(10px + 5%)");
    }
}

#[test]
fn flex_result_math_is_selected_across_grid_track_consumers() {
    for property in AXES.into_iter().chain(IMPLICIT) {
        for authored in [
            "calc(2 * 1fr)",
            "min(1fr, 2fr)",
            "calc(-1fr)",
            "calc(1fr * (1% / 1%))",
            "minmax(auto, max(1fr, 2fr))",
        ] {
            assert_accepted(property, authored);
        }
    }

    for (property, authored) in [
        (Property::GridTemplateColumns, "repeat(2, calc(1fr + 1fr))"),
        (
            Property::GridTemplateRows,
            "repeat(auto-fit, calc(2 * 1fr))",
        ),
        (Property::GridTemplate, "calc(2 * 1fr) / 10px"),
        (Property::Grid, "auto-flow calc(2 * 1fr) / 10px"),
    ] {
        assert_accepted(property, authored);
    }
}

#[test]
fn flex_math_does_not_enter_inflexible_or_length_percentage_slots() {
    for property in AXES.into_iter().chain(IMPLICIT) {
        for authored in [
            "calc(1fr + 10px)",
            "calc(1fr + 1%)",
            "round(1fr)",
            "minmax(calc(1fr), 10px)",
            "fit-content(calc(1fr))",
        ] {
            assert_rejected(property, authored);
        }
    }
    for property in IMPLICIT {
        assert_rejected(property, "repeat(2, calc(1fr))");
    }
}

fn assert_accepted(property: Property, authored: &str) {
    let source = format!(
        "color:red;{}:{authored};width:3px",
        property.canonical_name()
    );
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 3, "{source}");
    let declaration = &report.syntax()[1];
    assert_eq!(
        declaration.known().unwrap().property(),
        property,
        "{source}"
    );
    assert!(
        declaration.known().unwrap().property_value().is_some(),
        "{source}"
    );
    assert!(validate_style_attribute(&source).is_ok(), "{source}");
    let checked = parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(authored).expect("balanced Grid value"),
        CssImportance::Normal,
    )
    .unwrap_or_else(|error| panic!("checked Grid admission of {source}: {error:?}"));
    assert_eq!(checked.known().unwrap().property(), property);
}

fn assert_rejected(property: Property, authored: &str) {
    let source = format!(
        "color:red;{}:{authored};width:3px",
        property.canonical_name()
    );
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 2, "{source}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid Grid declaration: {source}: {report:?}");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert!(validate_style_attribute(&source).is_err(), "{source}");
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(property),
            parse_component_values(authored).expect("balanced Grid value"),
            CssImportance::Normal,
        )
        .is_err(),
        "checked Grid construction rejects {source}"
    );
}

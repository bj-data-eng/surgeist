//! `Error::to_string()` of every public constructor, with the combinations the constructors fix.
//!
//! The expected strings are written from each constructor's documented arguments: the field, the
//! rejected value and the rule for `invalid_value`; the family and operation labels for an
//! unsupported primitive; the category label and identifier for an unresolved resource; the quality
//! label and value for a degraded quality. `Error` prints its message and nothing else, so the text
//! never contains the classification. The constructor that takes a runtime-capability payload has
//! no public way to build the payload and is covered inside the crate.

use core::error::Error as StdError;
use surgeist_render::{
    DegradedQuality, DegradedQualityKind, Error, ErrorCode, InvalidValue, PrimitiveFamily, Rect, Size,
    PrimitiveOperation, UnresolvedResource, UnresolvedResourceKind, UnsupportedPrimitive,
};

#[test]
fn an_invalid_value_prints_its_field_value_and_rule_without_the_classification() {
    let error = Error::invalid_value("blur radius", -2.5, "must be finite and non-negative");
    assert_eq!(error.code(), ErrorCode::InvalidInput);
    assert_eq!(
        error.to_string(),
        "blur radius value -2.5 is invalid: must be finite and non-negative"
    );
    assert_eq!(error.message(), error.to_string());
    assert!(error.source().is_none());
    let value = error.invalid_value_diagnostic().unwrap();
    assert_eq!(value.field(), "blur radius");
    assert_eq!(value.value(), "-2.5");
}

#[test]
fn an_invalid_value_built_from_its_payload_prints_the_same_text() {
    let payload = InvalidValue::new("layer opacity", f64::NAN, "must be finite");
    let error = Error::from_invalid_value(payload);
    assert_eq!(error.code(), ErrorCode::InvalidInput);
    assert_eq!(
        error.to_string(),
        "layer opacity value NaN is invalid: must be finite"
    );
}

#[test]
fn an_unsupported_primitive_prints_its_family_and_operation() {
    let error = Error::unsupported_render_primitive(UnsupportedPrimitive::new(
        PrimitiveFamily::Filters,
        PrimitiveOperation::LayerFilter,
    ));
    assert_eq!(error.code(), ErrorCode::UnsupportedPrimitive);
    assert_eq!(
        error.to_string(),
        "render primitive is unsupported: filters / layer filter"
    );
    assert!(error.source().is_none());
}

#[test]
fn an_unresolved_resource_prints_its_category_and_identifier() {
    for (kind, text) in [
        (
            UnresolvedResourceKind::Image,
            "image resource hero.png could not be resolved",
        ),
        (
            UnresolvedResourceKind::Mask,
            "mask resource hero.png could not be resolved",
        ),
        (
            UnresolvedResourceKind::Filter,
            "filter resource hero.png could not be resolved",
        ),
        (
            UnresolvedResourceKind::Clip,
            "clip resource hero.png could not be resolved",
        ),
        (
            UnresolvedResourceKind::TextRunInkBounds,
            "text run ink bounds resource hero.png could not be resolved",
        ),
    ] {
        let error = Error::unresolved_resource(UnresolvedResource::new(kind, "hero.png"));
        assert_eq!(error.code(), ErrorCode::UnresolvedResource);
        assert_eq!(error.to_string(), text);
    }
}

#[test]
fn a_degraded_quality_prints_its_limitation_and_value() {
    for (kind, text) in [
        (
            DegradedQualityKind::ReducedIntermediatePrecision,
            "render quality degraded: reduced intermediate precision (rgba8)",
        ),
        (
            DegradedQualityKind::UnsupportedPaintSpaceConversion,
            "render quality degraded: unsupported paint-space conversion (rgba8)",
        ),
    ] {
        let error = Error::degraded_quality(DegradedQuality::new(kind, "rgba8"));
        assert_eq!(error.code(), ErrorCode::DegradedQuality);
        assert_eq!(error.to_string(), text);
    }
}

#[test]
fn a_value_passed_to_the_public_constructor_keeps_its_text_exactly() {
    // The constructor receives only the `Display` text of a value, so it never rewrites it:
    // trailing zeros, leading zeros, long digit strings and tiny or huge numbers stay as given.
    for text in [
        "2.10",
        "007.5",
        "0.000001",
        "0.12345678901234567890123456789",
        "123456789012345678901234567890.5",
        "1.7976931348623157e308",
        "NaN",
        "-inf",
        "hello",
        "",
    ] {
        let error = Error::invalid_value("x", text, "r");
        assert_eq!(error.to_string(), format!("x value {text} is invalid: r"));
        assert_eq!(error.invalid_value_diagnostic().unwrap().value(), text);
    }
    for (text, error) in [
        ("0.5", Error::invalid_value("x", 0.5, "r")),
        ("-12.25", Error::invalid_value("x", -12.25, "r")),
        ("0.00001", Error::invalid_value("x", 0.00001, "r")),
        ("0.000001", Error::invalid_value("x", 0.000001, "r")),
        ("18446744073709551615", Error::invalid_value("x", u64::MAX, "r")),
        ("-9223372036854775808", Error::invalid_value("x", i64::MIN, "r")),
        (
            "340282366920938463463374607431768211455",
            Error::invalid_value("x", u128::MAX, "r"),
        ),
        ("NaN", Error::invalid_value("x", f64::NAN, "r")),
    ] {
        assert_eq!(error.to_string(), format!("x value {text} is invalid: r"));
    }
}

#[test]
fn an_extreme_float_at_a_crate_call_site_prints_in_exponent_form() {
    // These crate call sites pass the rejected float, so they print it through the number helper.
    for (value, text) in [
        (-f64::MAX, "-1.7976931348623157e308"),
        (-f64::MIN_POSITIVE, "-2.2250738585072014e-308"),
        (-1e300, "-1e300"),
        (-2.5, "-2.5"),
    ] {
        let error = Rect::try_new(0.0, 0.0, value, 1.0).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("rectangle width value {text} is invalid: must be finite and non-negative")
        );
        let error = Size::try_new(1.0, value).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("size height value {text} is invalid: must be finite and non-negative")
        );
        assert!(error.to_string().len() < 100);
    }
}

//! Values 4 §5 converts finite Angle results outside supported range to a
//! supported multiple of 360deg. This applies to Angle-valued functions,
//! preserving their specified nonfinite argument exceptions.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#round-infinities
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#exponent-infinities
//! §10.10 Product step 9.2 merges Number children before typed evaluation:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification
//! Frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d likewise merges
//! Number children in CSSCalcTree+Simplification.cpp lines 654–745.
//! Already projected Number/compound values retain NaN and genuine infinities;
//! their finite overflow converts to the closest finite Number (#1297). An
//! Angle ancestor does not reconstruct their earlier magnitudes.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::value_assertions::assert_calculated_number as assert_number;
use crate::css::support::value_assertions::calculated_number as number;
use surgeist_css::*;

type Limits = CssSerializationLimits;
type Kind = SerializationFailure;

#[test]
fn finite_hypot_angle_results_obey_the_supported_angle_range() {
    // sqrt(2)*1.3e308 exceeds MAX, with entirely finite Angle arguments.
    for (source, expected) in [
        ("cos(hypot(1.3e308deg,1.3e308deg))", "calc(1)"),
        ("sin(hypot(-1.3e308deg,1.3e308deg))", "calc(0)"),
        ("tan(hypot(1.3e308deg,-1.3e308deg))", "calc(0)"),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn finite_round_angle_results_obey_the_supported_angle_range() {
    // Finite nonzero steps select ±2e308deg. Infinite-step and zero-step
    // argument exceptions remain separate controls below.
    for (source, expected) in [
        ("cos(round(up,1.5e308deg,1e308deg))", "calc(1)"),
        ("cos(round(down,-1.5e308deg,1e308deg))", "calc(1)"),
        ("sin(round(up,1.5e308deg,1e308deg))", "calc(0)"),
        ("tan(round(down,-1.5e308deg,1e308deg))", "calc(0)"),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn flat_products_merge_number_children_before_typed_angle_evaluation() {
    for (source, expected) in [
        ("cos(1e308deg * 2 * 0)", "calc(1)"),
        ("cos(2 * 1e308deg * 0)", "calc(1)"),
        ("cos(2 * 0 * 1e308deg)", "calc(1)"),
        ("sin(-1e308deg * 2 * 0)", "calc(0)"),
        ("cos(1e308deg * 2 * 1e-308)", "calc(0.999391)"),
        ("cos(1e308deg * 1e-308 * 2)", "calc(0.999391)"),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn grouped_angle_results_and_ordinary_finite_functions_keep_their_behavior() {
    for (source, expected) in [
        ("cos((1e308deg * 2) * 0)", "calc(1)"),
        ("sin((-1e308deg * 2) * 0)", "calc(0)"),
        ("cos(hypot(3deg,4deg))", "calc(0.996195)"),
        ("cos(round(up,1.5deg,1deg))", "calc(0.999391)"),
    ] {
        assert_number(source, expected);
    }
}

/// The cosine of a finite supported angle: a number in [-1, 1], never NaN.
/// Its digits depend on binary64 radian conversion and are not prescribed.
fn assert_finite_cosine(source: &str) {
    let value = number(source);
    let result = value.serialize_specified(Default::default()).unwrap();
    let coefficient: f64 = result
        .strip_prefix("calc(")
        .and_then(|rest| rest.strip_suffix(')'))
        .and_then(|text| text.parse().ok())
        .unwrap_or_else(|| panic!("{source}: {result}"));
    assert!((-1.0..=1.0).contains(&coefficient), "{source}: {result}");
}

#[test]
fn materialized_numbers_and_retained_products_follow_the_selected_policy() {
    // A Number subtree becomes a scalar before its Angle ancestor sees it.
    // Its finite overflow converts to the closest finite Number, f64::MAX
    // (#1297); MAX degrees is a supported angle, so its cosine is finite.
    // A compound Length*Angle subtree instead remains a Product: step 9.4
    // cannot materialize its compound type, and the ancestor flattens it under
    // step 9.1. Its finite leaves then participate in one final Angle Product.
    let max = format!("calc({:.0})", f64::MAX);
    for (source, expected) in [
        ("calc(1e308 * 2)", max.as_str()),
        ("calc((1e308 * 2) / (1e308 * 2))", "calc(1)"),
        ("calc((1e308 * 2) * 0)", "calc(0)"),
        ("calc(1 / (0 * -1))", "calc(-infinity)"),
        ("cos((1e308 * 2) * 0deg)", "calc(1)"),
        // The overflowing prefix 1e308px * 2deg has a compound Length*Angle
        // type, so it converts to f64::MAX, not to the Angle endpoint; the
        // final quotient is then MAX / binary64(1e308) = 1.7976931348623157deg.
        ("cos((1e308px * 2deg) / 1e308px)", "calc(0.999508)"),
    ] {
        assert_number(source, expected);
    }
    for source in [
        // MAX degrees is a supported angle.
        "cos((1e308px * 2deg) / 1px)",
        "cos((1e308 * 2) * 1deg)",
        "cos(calc(1e308 * 2) * 1deg)",
        "cos(hypot(1.3e308,1.3e308) * 1deg)",
        "cos(exp(710) * 1deg)",
        "cos(pow(1e308,2) * 1deg)",
    ] {
        assert_finite_cosine(source);
    }
}

#[test]
fn finite_authored_number_coefficients_keep_the_selected_projection_range() {
    // A finite Number coefficient beyond binary64 converts to f64::MAX (#1297).
    let huge = "1e170141183460469231731687303715884105728";
    for source in [
        "cos(1e400 * 1deg)".to_owned(),
        "cos(-1e400 * 1deg)".to_owned(),
        format!("cos({huge} * 1deg)"),
    ] {
        assert_finite_cosine(&source);
    }
    // MAX / MAX is one up to binary64 rounding of the subnormal reciprocal.
    assert_number(&format!("cos(({huge} / {huge}) * 1deg)"), "calc(0.999848)");
    for source in [
        "cos(0e170141183460469231731687303715884105728 * 1deg)",
        "cos(1e-170141183460469231731687303715884105728 * 1deg)",
    ] {
        assert_number(source, "calc(1)");
    }
}

#[test]
fn defined_infinity_and_nan_exceptions_remain_infectious() {
    for source in [
        "cos(infinity * 1deg * 0)",
        "cos(pow(0,-1) * 1deg * 0)",
        "cos((1 / 0) * 0deg)",
        "cos((infinity * 1px * 2deg) / 1px)",
        "cos(hypot(infinity * 1deg,1deg))",
        "cos(round(up,1deg,infinity * 1deg))",
        "cos(round(down,-1deg,infinity * 1deg))",
        "cos(hypot(NaN * 1deg,1.3e308deg))",
        "cos(round(up,1.5e308deg,0deg))",
        "cos(((infinity * 2) + (-infinity * 2)) * 1deg)",
    ] {
        assert_number(source, "calc(NaN)");
    }
}

#[test]
fn merged_number_product_replacements_obey_exact_limits_and_atomic_failures() {
    let value = number("cos(1e308deg * 2 * 0)");
    let before = value.clone();
    let expected = "calc(1)";
    // Cos, Product and three leaves: five inputs. The three leaf projections,
    // merged Number replacement, typed Product replacement and Cos replacement
    // consume six projection nodes, including the temporary Number coefficient.
    for (limits, kind) in [
        (Limits::new(4, 6, expected.len()), Kind::InputNodeLimit),
        (Limits::new(5, 5, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(5, 6, expected.len() - 1), Kind::ByteLimit),
    ] {
        let error = value.serialize_specified(limits).unwrap_err();
        assert_eq!(serialization_failure(&error), kind, "{limits:?}");
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
    }
    let result = value.serialize_specified(Limits::new(5, 6, expected.len()));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn function_and_product_siblings_keep_one_cumulative_budget() {
    let value = number("calc(cos(1e308deg * 2 * 0) + cos(hypot(1.3e308deg,1.3e308deg)))");
    let before = value.clone();
    let expected = "calc(2)";
    // Outer Calc/Sum plus five-node Product/Cos and four-node Hypot/Cos:
    // eleven inputs. Product/Cos emits six, Hypot/Cos four, and Sum one.
    for (limits, kind) in [
        (Limits::new(10, 11, expected.len()), Kind::InputNodeLimit),
        (
            Limits::new(11, 10, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(11, 11, expected.len() - 1), Kind::ByteLimit),
    ] {
        let error = value.serialize_specified(limits).unwrap_err();
        assert_eq!(serialization_failure(&error), kind, "{limits:?}");
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
    }
    let result = value.serialize_specified(Limits::new(11, 11, expected.len()));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}

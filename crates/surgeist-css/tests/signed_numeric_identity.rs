#![forbid(unsafe_code)]
//! The adopted Surgeist aggregate contract compares exact numeric syntax while
//! ignoring numeric provenance only. This is an authored-value product contract,
//! not a CSSOM semantic-equivalence claim or an evaluation of symbolic math.
//! Parsing, public aggregate getters, and PartialEq already express the behavior.

use surgeist_css::*;

macro_rules! aggregate_parser {
    ($name:ident, $ty:ty, $variant:ident, $getter:ident) => {
        fn $name(source: &str) -> $ty {
            let report = parse_style_attribute(source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().len(), 1, "{source}");
            let CssKnownPropertyValueRef::$variant(value) = report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap()
            else {
                panic!("typed property: {source}")
            };
            value.$getter().clone()
        }
    };
}
aggregate_parser!(transform, CssTransform, Transform, value);
aggregate_parser!(scale, CssScale, Scale, value);
aggregate_parser!(
    easing,
    CssEasingList,
    TransitionTimingFunction,
    timing_functions
);
aggregate_parser!(animation, CssAnimationList, Animation, animations);
aggregate_parser!(transition, CssTransitionList, Transition, transitions);

#[test]
fn matrix_equality_keeps_exact_ordinary_number_spelling() {
    assert!(
        transform("transform: matrix(1, 0, 0, 1, 10, 20)")
            != transform("transform: matrix(1.0, 0, 0, 1, 10, 20)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: matrix(1, 0, 0, 1, 10, 20)\") versus transform(\"transform: matrix(1.0, 0, 0, 1, 10, 20)\")"
    );
}
#[test]
fn matrix3d_equality_keeps_exact_ordinary_number_spelling() {
    assert!(
        transform("transform: matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)")
            != transform(
                "transform: matrix3d(1.0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)"
            ),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)\") versus transform(\"transform: matrix3d(1.0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)\")"
    );
}
#[test]
fn rotate3d_equality_keeps_exact_axis_number_spelling() {
    assert!(
        transform("transform: rotate3d(1, 0, -1, 45deg)")
            != transform("transform: rotate3d(1.0, 0, -1, 45deg)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: rotate3d(1, 0, -1, 45deg)\") versus transform(\"transform: rotate3d(1.0, 0, -1, 45deg)\")"
    );
}
#[test]
fn scale_and_axis_equality_keep_exact_number_spelling() {
    for name in ["scale", "scaleX", "scaleY", "scaleZ"] {
        assert!(
            transform(&format!("transform: {name}(1)"))
                != transform(&format!("transform: {name}(1.0)")),
            "Numeric identity must retain distinct authored structures or provenance; inputs: transform(&format!(\"transform: {{name}}(1)\")) versus transform(&format!(\"transform: {{name}}(1.0)\")); case: {name}"
        );
    }
}
#[test]
fn scale3d_equality_keeps_exact_number_spelling() {
    assert!(
        transform("transform: scale3d(1, 2, 3)") != transform("transform: scale3d(1.0, 2, 3)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scale3d(1, 2, 3)\") versus transform(\"transform: scale3d(1.0, 2, 3)\")"
    );
}
#[test]
fn scale3d_and_scale_z_equality_keep_exact_percentage_spelling() {
    assert!(
        transform("transform: scale3d(1, 25%, 3)") != transform("transform: scale3d(1, 25.0%, 3)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scale3d(1, 25%, 3)\") versus transform(\"transform: scale3d(1, 25.0%, 3)\")"
    );
    assert!(
        transform("transform: scaleZ(25%)") != transform("transform: scaleZ(25.0%)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scaleZ(25%)\") versus transform(\"transform: scaleZ(25.0%)\")"
    );
}
#[test]
fn independent_scale_equality_keeps_exact_number_spelling() {
    assert!(
        scale("scale: 1 2 3") != scale("scale: 1.0 2 3"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: scale(\"scale: 1 2 3\") versus scale(\"scale: 1.0 2 3\")"
    );
}
#[test]
fn cubic_bezier_equality_keeps_exact_x_and_y_number_spelling() {
    assert!(
        easing("transition-timing-function: cubic-bezier(0, 2, 1, 3)")
            != easing("transition-timing-function: cubic-bezier(0.0, 2, 1, 3)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: easing(\"transition-timing-function: cubic-bezier(0, 2, 1, 3)\") versus easing(\"transition-timing-function: cubic-bezier(0.0, 2, 1, 3)\")"
    );
    assert!(
        easing("transition-timing-function: cubic-bezier(0, 2, 1, 3)")
            != easing("transition-timing-function: cubic-bezier(0, 2.0, 1, 3)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: easing(\"transition-timing-function: cubic-bezier(0, 2, 1, 3)\") versus easing(\"transition-timing-function: cubic-bezier(0, 2.0, 1, 3)\")"
    );
}
#[test]
fn transition_and_animation_equality_keep_exact_cubic_bezier_numbers() {
    assert!(
        transition("transition: opacity 1s cubic-bezier(0, 2, 1, 3)")
            != transition("transition: opacity 1s cubic-bezier(0, 2.0, 1, 3)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transition(\"transition: opacity 1s cubic-bezier(0, 2, 1, 3)\") versus transition(\"transition: opacity 1s cubic-bezier(0, 2.0, 1, 3)\")"
    );
    assert!(
        animation("animation: fade 1s cubic-bezier(0, 2, 1, 3)")
            != animation("animation: fade 1s cubic-bezier(0, 2.0, 1, 3)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: animation(\"animation: fade 1s cubic-bezier(0, 2, 1, 3)\") versus animation(\"animation: fade 1s cubic-bezier(0, 2.0, 1, 3)\")"
    );
}
#[test]
fn signed_numeric_aggregate_equality_keeps_zero_sign_and_exponent_spelling() {
    assert!(
        transform("transform: scale(-0)") != transform("transform: scale(0)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scale(-0)\") versus transform(\"transform: scale(0)\")"
    );
    assert!(
        transform("transform: scaleZ(-0%)") != transform("transform: scaleZ(0%)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scaleZ(-0%)\") versus transform(\"transform: scaleZ(0%)\")"
    );
    assert!(
        scale("scale: 1e0") != scale("scale: 1"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: scale(\"scale: 1e0\") versus scale(\"scale: 1\")"
    );
}

macro_rules! transform_math_origin_case {
    ($name:ident, $value:literal) => {
        #[test]
        fn $name() {
            assert!(transform(concat!("transform: ", $value)) == transform(concat!("  transform: ", $value)), "Exact numeric structure must compare equal across changed source origins; transform input: {}", $value);
        }
    };
}
transform_math_origin_case!(
    matrix_equality_ignores_number_math_origin,
    "matrix(calc(1 + 2), 0, 0, 1, 10, 20)"
);
transform_math_origin_case!(
    matrix3d_equality_ignores_number_math_origin,
    "matrix3d(calc(1 + 2), 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)"
);
transform_math_origin_case!(
    rotate3d_equality_ignores_axis_number_math_origin,
    "rotate3d(calc(1 + 2), 0, -1, 45deg)"
);
transform_math_origin_case!(
    scale_equality_ignores_number_math_origin,
    "scale(calc(1 + 2), 2)"
);
transform_math_origin_case!(
    scale_x_equality_ignores_number_math_origin,
    "scaleX(calc(1 + 2))"
);
transform_math_origin_case!(
    scale_y_equality_ignores_number_math_origin,
    "scaleY(calc(1 + 2))"
);
transform_math_origin_case!(
    scale_z_equality_ignores_number_math_origin,
    "scaleZ(calc(1 + 2))"
);
transform_math_origin_case!(
    scale3d_equality_ignores_number_math_origin,
    "scale3d(calc(1 + 2), 2, 3)"
);
transform_math_origin_case!(
    scale3d_equality_ignores_percentage_math_origin,
    "scale3d(1, calc(25% + 5%), 3)"
);
transform_math_origin_case!(
    scale_z_equality_ignores_percentage_math_origin,
    "scaleZ(calc(25% + 5%))"
);

#[test]
fn cubic_bezier_equality_ignores_number_math_origins() {
    assert!(
        easing("transition-timing-function: cubic-bezier(calc(0), calc(1 + 2), 1, 3)")
            == easing("  transition-timing-function: cubic-bezier(calc(0), calc(1 + 2), 1, 3)"),
        "Numeric identity must compare equal when only numeric origin differs; inputs: easing(\"transition-timing-function: cubic-bezier(calc(0), calc(1 + 2), 1, 3)\") versus easing(\"  transition-timing-function: cubic-bezier(calc(0), calc(1 + 2), 1, 3)\")"
    );
}

#[test]
fn transform_equality_controls_keep_order_omission_branch_and_symbolic_structure() {
    assert!(
        transform("transform: scale(1, 2)") == transform("  transform: scale(1, 2)"),
        "Numeric identity must compare equal when only numeric origin differs; inputs: transform(\"transform: scale(1, 2)\") versus transform(\"  transform: scale(1, 2)\")"
    );
    assert!(
        transform("transform: scale(1)") != transform("transform: scale(1, 1)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scale(1)\") versus transform(\"transform: scale(1, 1)\")"
    );
    assert!(
        transform("transform: scale(1, 2)") != transform("transform: scale(2, 1)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scale(1, 2)\") versus transform(\"transform: scale(2, 1)\")"
    );
    assert!(
        transform("transform: scaleX(1)") != transform("transform: scaleY(1)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scaleX(1)\") versus transform(\"transform: scaleY(1)\")"
    );
    assert!(
        transform("transform: scaleZ(1)") != transform("transform: scaleZ(100%)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scaleZ(1)\") versus transform(\"transform: scaleZ(100%)\")"
    );
    assert!(
        transform("transform: scale(calc(1 + 2))") != transform("transform: scale(calc(2 + 1))"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scale(calc(1 + 2))\") versus transform(\"transform: scale(calc(2 + 1))\")"
    );
    assert!(
        transform("transform: scale(1)") != transform("transform: scale(calc(1))"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: scale(1)\") versus transform(\"transform: scale(calc(1))\")"
    );
    assert!(
        scale("scale: 1") != scale("scale: 1 1"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: scale(\"scale: 1\") versus scale(\"scale: 1 1\")"
    );
    assert!(
        scale("scale: 1 2") != scale("scale: 2 1"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: scale(\"scale: 1 2\") versus scale(\"scale: 2 1\")"
    );
    assert!(
        easing("transition-timing-function: cubic-bezier(0, 2, 1, 3)")
            != easing("transition-timing-function: cubic-bezier(0, 3, 1, 2)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: easing(\"transition-timing-function: cubic-bezier(0, 2, 1, 3)\") versus easing(\"transition-timing-function: cubic-bezier(0, 3, 1, 2)\")"
    );
    assert!(
        transform("transform: rotate(calc(45deg))")
            != transform("  transform: rotate(calc(45deg))"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: transform(\"transform: rotate(calc(45deg))\") versus transform(\"  transform: rotate(calc(45deg))\")"
    );
}

//! Authored transform grammar behind the SVG transform source questions (#562).
//!
//! Characterization of existing behavior, with no production change and so no
//! preimplementation RED. The expectations come from the selected sources:
//! Transforms 1 §9.1 (the `skewX()` and `skewY()` functions), §7.2 (the SVG
//! `transform` attribute syntax, which differs from the CSS property) and
//! Transforms 2 §§11 and 12.2 (3D functions in SVG, and the 3D function list).
//! The crate owns authored CSS property syntax only; it has no SVG attribute
//! front, no `animate` element and no interpolation, so the attribute-only
//! productions must stay rejected here.

use crate::css::support::authored_property::invalid;
use crate::css::support::sole_declaration as declaration;
use surgeist_css::numeric::{
    CssAngle, CssAngleOrZero, CssAngleUnit, CssNumeric, CssNumericLiteral,
};
use surgeist_css::property::{CssKnownProperty, CssKnownPropertyValueRef};
use surgeist_css::values::transform::{
    CssTransform, CssTransformFunction, CssTransformFunctionKind,
};

fn transform_value(css: &str) -> CssTransform {
    let declaration = declaration(css);
    let CssKnownPropertyValueRef::Transform(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected the transform property: {css}")
    };
    value.value().clone()
}

fn transform_functions(css: &str) -> Vec<CssTransformFunction> {
    let CssTransform::Functions(list) = transform_value(css) else {
        panic!("expected a function list: {css}")
    };
    list.functions().to_vec()
}

fn specified(css: &str) -> String {
    transform_value(css)
        .serialize_specified(Default::default())
        .unwrap()
}

fn angle(text: &str, unit: CssAngleUnit) -> CssAngleOrZero {
    CssAngleOrZero::Angle(
        CssNumeric::<CssAngle>::try_from_literal(
            CssNumericLiteral::<CssAngle>::try_new(text, unit).unwrap(),
        )
        .unwrap(),
    )
}

#[test]
fn skew_x_and_skew_y_keep_their_own_identity_spelling_and_angle() {
    // Transforms 1 §9.1 defines `skewX() = skewX( [ <angle> | <zero> ] )` ("2D
    // skew transformation along the X axis") and `skewY()` likewise along the
    // Y axis. The `"skewY"` literal printed in the §7.2 `skewX` BNF does not
    // make one name stand for the other: the two names are distinct functions
    // with distinct operands.
    let functions = transform_functions("transform:skewX(10deg) skewY(20deg)");
    assert_eq!(
        functions,
        [
            CssTransformFunction::SkewX(angle("10", CssAngleUnit::Degrees)),
            CssTransformFunction::SkewY(angle("20", CssAngleUnit::Degrees)),
        ]
    );
    assert_eq!(functions[0].kind(), CssTransformFunctionKind::SkewX);
    assert_eq!(functions[1].kind(), CssTransformFunctionKind::SkewY);
    assert_ne!(functions[0].kind(), functions[1].kind());
    assert_eq!(
        specified("transform:skewX(10deg) skewY(20deg)"),
        "skewX(10deg) skewY(20deg)"
    );
    // Reversed order keeps each name attached to its own angle.
    assert_eq!(
        transform_functions("transform:skewY(20deg) skewX(10deg)"),
        [
            CssTransformFunction::SkewY(angle("20", CssAngleUnit::Degrees)),
            CssTransformFunction::SkewX(angle("10", CssAngleUnit::Degrees)),
        ]
    );
    assert_eq!(
        specified("transform:skewY(20deg) skewX(10deg)"),
        "skewY(20deg) skewX(10deg)"
    );
    // Each name also stands alone; neither is accepted only as the other.
    assert_eq!(specified("transform:skewX(10deg)"), "skewX(10deg)");
    assert_eq!(specified("transform:skewY(10deg)"), "skewY(10deg)");
}

#[test]
fn skew_x_and_skew_y_take_exactly_one_angle_or_zero() {
    // Transforms 1 §9.1: one `<angle> | <zero>` operand. The SVG attribute
    // productions of §7.2 take a unitless `number` instead, interpreted as
    // degrees by §7.3; that attribute-only form is not a property production.
    for name in ["skewX", "skewY"] {
        for text in [
            format!("{name}(0)"),
            format!("{name}(0deg)"),
            format!("{name}(-10deg)"),
            format!("{name}(0.5turn)"),
            format!("{name}(100grad)"),
            format!("{name}(1rad)"),
        ] {
            let css = format!("transform:{text}");
            assert_eq!(transform_functions(&css).len(), 1, "{css}");
        }
        for text in [
            format!("{name}(10)"),
            format!("{name}(-10)"),
            format!("{name}(0.5)"),
            format!("{name}()"),
            format!("{name}(10deg 20deg)"),
            format!("{name}(10deg, 20deg)"),
            format!("{name}(10px)"),
            format!("{name}(10%)"),
            format!("{name}( , 10deg)"),
        ] {
            invalid(CssKnownProperty::Transform, &text);
        }
    }
}

#[test]
fn svg_attribute_only_productions_are_not_property_syntax() {
    // Transforms 1 §7.2 states that the attribute syntax differs from the CSS
    // property: unitless numbers, optional commas between numbers and a
    // `rotate` with a center. None of these is a `<transform-function>`
    // argument list, so each is rejected whole. Comma-separated function lists
    // (the `transforms` production) are not a property list either.
    for text in [
        "translate(10 20)",
        "translate(10,20)",
        "scale(2 3)",
        "rotate(30)",
        "rotate(30 10 10)",
        "rotate(30deg 10px 10px)",
        "rotate(30deg, 10px, 10px)",
        "matrix(1 0 0 1 0 0)",
        "matrix(1,0,0,1,0)",
        "skewX(30)",
        "skewY(30)",
        "skewX(10deg),skewY(10deg)",
        "skewX(10deg),",
        ",skewX(10deg)",
    ] {
        invalid(CssKnownProperty::Transform, text);
    }
    // The comma-separated property arguments and the unitless zero stay valid.
    assert_eq!(specified("transform:translate(0,0)"), "translate(0)");
}

#[test]
fn three_dimensional_functions_are_property_syntax_and_stay_in_the_list() {
    // Transforms 2 §12.2 lists the 3D transform functions; §11 gives no
    // separate SVG attribute grammar for them (its open Issue asks to describe
    // it). The property grammar accepts each with the property argument types,
    // and a list that mixes 2D and 3D functions keeps every function in
    // authored order. Where SVG forbids 3D functions (§11: for clipPath,
    // linearGradient, radialGradient and pattern "the complete transform list
    // must be ignored") is a per-element decision of style resolution, which
    // this crate does not make.
    for (input, kind) in [
        (
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1)",
            CssTransformFunctionKind::Matrix3d,
        ),
        (
            "translate3d(1px,2px,3px)",
            CssTransformFunctionKind::Translate3d,
        ),
        ("translateZ(3px)", CssTransformFunctionKind::TranslateZ),
        ("scale3d(1,2,3)", CssTransformFunctionKind::Scale3d),
        ("scaleZ(3)", CssTransformFunctionKind::ScaleZ),
        ("rotate3d(1,2,3,30deg)", CssTransformFunctionKind::Rotate3d),
        ("rotateX(30deg)", CssTransformFunctionKind::RotateX),
        ("rotateY(30deg)", CssTransformFunctionKind::RotateY),
        ("rotateZ(30deg)", CssTransformFunctionKind::RotateZ),
        ("perspective(100px)", CssTransformFunctionKind::Perspective),
    ] {
        let alone = transform_functions(&format!("transform:{input}"));
        assert_eq!(alone.len(), 1, "{input}");
        assert_eq!(alone[0].kind(), kind, "{input}");

        let mixed = transform_functions(&format!("transform:translate(1px) {input} skewX(5deg)"));
        let kinds: Vec<_> = mixed.iter().map(CssTransformFunction::kind).collect();
        assert_eq!(
            kinds,
            [
                CssTransformFunctionKind::Translate,
                kind,
                CssTransformFunctionKind::SkewX
            ],
            "{input}: mixed list keeps every function in order"
        );
    }
    // The property argument types apply; SVG-style unitless numbers do not.
    for text in [
        "translateZ(3)",
        "rotateX(30)",
        "rotate3d(1 2 3 30deg)",
        "rotate3d(1,2,3,30)",
        "scale3d(1 2 3)",
        "translate3d(1px 2px 3px)",
        "perspective(100)",
        "matrix3d(1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1)",
    ] {
        invalid(CssKnownProperty::Transform, text);
    }
}

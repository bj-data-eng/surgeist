#![forbid(unsafe_code)]

use super::authored::construct_component;
use crate::component::{CssComponentValueRef, CssValueOrigin};
use crate::numeric::expression::NumericInputContext;

#[test]
fn portable_failures_name_original_children_and_closing_boundaries() {
    for (text, path, closing) in [
        ("light-dark(red,rgb(1 x 3))", &[2, 2][..], false),
        ("light-dark(red,rgb(1 2))", &[2][..], true),
        ("light-dark(red,rgb(1 2", &[2][..], true),
        ("rgb(1 2)", &[][..], true),
    ] {
        let values = crate::component::values::parse_component_values(text).unwrap();
        let root = &values.items()[0];
        let CssValueOrigin::Parsed(origin) = root.origin() else {
            panic!("actual source root");
        };
        let failure =
            construct_component(root, &NumericInputContext::Parsed(origin.source())).unwrap_err();
        assert!(matches!(
            failure.kind,
            crate::diagnostic::error::ErrorKind::InvalidColorSyntax(_)
        ));
        assert_eq!(failure.path.as_ref(), path, "{text}");
        assert_eq!(failure.closing, closing, "{text}");
    }
}

#[test]
fn mixed_origin_failure_path_reaches_the_original_programmatic_leaf() {
    let children = crate::component::CssComponentValues::try_new(
        vec![
            crate::component::values::parse_component_values("1")
                .unwrap()
                .items()[0]
                .clone(),
            crate::component::CssComponentValue::try_ident("x").unwrap(),
            crate::component::CssComponentValue::try_number("3").unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    let root = crate::component::CssComponentValue::try_function("rgb", children).unwrap();
    let values =
        crate::component::CssComponentValues::try_new(vec![root], Default::default()).unwrap();
    let serialized = values.serialize_tokens(Default::default()).unwrap();
    let failure = construct_component(
        &values.items()[0],
        &NumericInputContext::Components(&values, &serialized),
    )
    .unwrap_err();
    assert_eq!(failure.path.as_ref(), &[1]);
    assert!(!failure.closing);
    let CssComponentValueRef::Function(root) = values.items()[0].view() else {
        panic!("actual root");
    };
    let leaf = &root.values().items()[failure.path[0]];
    assert!(matches!(
        root.values().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(leaf.origin(), &CssValueOrigin::Programmatic);
    assert!(matches!(
        leaf.view(),
        CssComponentValueRef::Token(crate::component::CssValueTokenRef::Ident("x"))
    ));
}

#[test]
fn borrowed_calculation_admission_preserves_exact_and_one_short_limits() {
    let values = crate::component::values::parse_component_values("calc(1 + max(2,3))").unwrap();
    let serialized = values.serialize_tokens(Default::default()).unwrap();
    let context = NumericInputContext::Components(&values, &serialized);
    let root = &values.items()[0];
    let expression = context
        .admit_with_limits(
            root,
            crate::numeric::expression::CalculationRoot::Number,
            crate::component::CssComponentValueLimits::try_new(2, 9, 19).unwrap(),
        )
        .unwrap();
    assert_eq!(
        expression.result_type(),
        crate::numeric::CssCalculationType::Number
    );
    let value =
        crate::numeric::CssCalculation::<crate::numeric::CssNumber>::from_expression(expression);
    let retained = value.components();
    assert_eq!(retained.items()[0], *root);
    assert!(
        !std::ptr::eq(&retained.items()[0], root),
        "completed public storage remains owned"
    );
    for (limits, kind) in [
        (
            (1, 9, 19),
            crate::component::values::CssComponentValueErrorKind::NestingLimit,
        ),
        (
            (2, 8, 19),
            crate::component::values::CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            (2, 9, 18),
            crate::component::values::CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let error = context
            .admit_with_limits(
                root,
                crate::numeric::expression::CalculationRoot::Number,
                crate::component::CssComponentValueLimits::try_new(limits.0, limits.1, limits.2)
                    .unwrap(),
            )
            .unwrap_err();
        assert_eq!(error.component_error().unwrap().kind(), kind);
    }
}

#[test]
fn relative_numeric_failure_excludes_only_the_synthetic_numeric_root() {
    let values =
        crate::component::values::parse_component_values("rgb(from red calc(r + 1px) g b)")
            .unwrap();
    let serialized = values.serialize_tokens(Default::default()).unwrap();
    let failure = construct_component(
        &values.items()[0],
        &NumericInputContext::Components(&values, &serialized),
    )
    .unwrap_err();
    assert_eq!(failure.path.as_ref(), &[4, 4]);
    assert!(!failure.closing);
    assert!(matches!(
        failure.kind,
        crate::diagnostic::error::ErrorKind::InvalidColorSyntax(_)
    ));
}

#[test]
fn completed_failure_keeps_its_immutable_path_after_the_input_drops() {
    let failure = {
        let values =
            crate::component::values::parse_component_values("light-dark(red,rgb(1 2").unwrap();
        let CssValueOrigin::Parsed(origin) = values.items()[0].origin() else {
            panic!("actual source");
        };
        construct_component(
            &values.items()[0],
            &NumericInputContext::Parsed(origin.source()),
        )
        .unwrap_err()
    };
    assert_eq!(failure.path.as_ref(), &[2]);
    assert!(failure.closing);
    assert!(matches!(
        failure.kind,
        crate::diagnostic::error::ErrorKind::InvalidColorSyntax(_)
    ));
    eprintln!(
        "completed color failure layout: kind={}, path={}, total={}",
        std::mem::size_of::<crate::diagnostic::error::ErrorKind>(),
        std::mem::size_of::<Box<[usize]>>(),
        std::mem::size_of::<super::authored::ColorInputFailure>()
    );
}

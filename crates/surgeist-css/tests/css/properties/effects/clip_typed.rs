#![forbid(unsafe_code)]

//! Checked `clip` construction and specified serialization per CSS Masking 1
//! Appendix A, selected 2021-08-05 Candidate Recommendation Draft:
//! https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#propdef-clip
//! CSS Values 4 §10.13 defines canonical calculation serialization:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-serialize

use surgeist_css::*;

fn edge(value: CssSpecifiedLength) -> CssClipEdge {
    CssClipEdge::Length(value)
}

fn parsed(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn parsed_clip(declaration: &CssDeclaration) -> &CssClip {
    let CssKnownPropertyValueRef::Clip(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed clip")
    };
    value.clip()
}

#[test]
fn checked_edge_construction_accepts_signed_pure_lengths_and_rejects_other_domains() {
    for css in ["0", "-1px", "2em"] {
        let value = signed_length(css);
        assert_eq!(value.serialize_specified().unwrap(), css);
        assert_eq!(
            CssClipEdge::Length(value).serialize_specified().unwrap(),
            css
        );
    }
    for css in ["1%", "auto", "min-content", "max-content"] {
        assert!(
            CssSpecifiedLength::try_from_component(CssComponentValue::try_token(css).unwrap())
                .is_err()
        );
    }
    assert!(matches!(
        signed_length("2em").origin(),
        CssValueOrigin::Programmatic
    ));
    assert_eq!(CssClipEdge::Auto.serialize_specified().unwrap(), "auto");
    assert_eq!(CssClip::Auto.serialize_specified().unwrap(), "auto");
}

#[test]
fn constructed_rectangle_preserves_four_edge_order_and_canonical_commas() {
    let rect = CssClipRect::new(
        CssClipEdge::Auto,
        edge(signed_length("-1px")),
        edge(signed_length("2em")),
        edge(signed_length("0")),
    );
    assert!(matches!(rect.top(), CssClipEdge::Auto));
    assert!(
        matches!(rect.right(), CssClipEdge::Length(length) if exact_literal(length.literal_component(), "-1px"))
    );
    assert!(
        matches!(rect.bottom(), CssClipEdge::Length(length) if exact_literal(length.literal_component(), "2em"))
    );
    assert!(
        matches!(rect.left(), CssClipEdge::Length(length) if exact_literal(length.literal_component(), "0"))
    );
    assert_eq!(
        rect.serialize_specified().unwrap(),
        "rect(auto, -1px, 2em, 0)"
    );
    let clip = CssClip::Rect(rect);
    assert_eq!(
        clip.serialize_specified().unwrap(),
        "rect(auto, -1px, 2em, 0)"
    );

    let from_space = parsed("clip:rect(auto -1px 2em 0)");
    assert_eq!(
        parsed_clip(&from_space).serialize_specified().unwrap(),
        "rect(auto, -1px, 2em, 0)"
    );
    let reparsed = parsed("clip:rect(auto, -1px, 2em, 0)");
    let CssClip::Rect(reparsed_rect) = parsed_clip(&reparsed) else {
        panic!("canonical rectangle reparses")
    };
    assert!(matches!(reparsed_rect.top(), CssClipEdge::Auto));
    assert!(
        matches!(reparsed_rect.right(), CssClipEdge::Length(length) if exact_dimension(length.literal_component(), "-1", "px"))
    );
    assert!(
        matches!(reparsed_rect.bottom(), CssClipEdge::Length(length) if exact_dimension(length.literal_component(), "2", "em"))
    );
    assert!(
        matches!(reparsed_rect.left(), CssClipEdge::Length(length) if exact_literal(length.literal_component(), "0"))
    );
    for edge in [
        reparsed_rect.right(),
        reparsed_rect.bottom(),
        reparsed_rect.left(),
    ] {
        let CssClipEdge::Length(length) = edge else {
            panic!("parsed numeric edge")
        };
        let CssValueOrigin::Parsed(origin) = length.origin() else {
            panic!("parsed numeric origin")
        };
        assert_eq!(origin.source().as_str(), "clip:rect(auto, -1px, 2em, 0)");
        assert!(
            origin.span().end().byte_offset().value() > origin.span().start().byte_offset().value()
        );
    }
}

#[test]
fn checked_calculations_remain_wrapped_and_canonical() {
    let literal_math = signed_length("calc(-2px)");
    assert!(literal_math.literal_component().is_none());
    let calculation = literal_math.calculation().expect("checked math root");
    assert_eq!(calculation.result_type(), CssCalculationType::Length);
    let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
        panic!("authored calc wrapper")
    };
    let CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(value)) = root.operand()
    else {
        panic!("signed length operand")
    };
    assert_eq!(value.representation(), "-2");
    assert_eq!(value.unit(), Some("px"));
    assert_eq!(literal_math.serialize_specified().unwrap(), "calc(-2px)");

    let folded = signed_length("calc(1px - 2px)");
    let mixed = signed_length("calc(1px + 2em)");
    for (value, operator, second_unit) in [
        (&folded, CssCalculationSumOperator::Subtract, "px"),
        (&mixed, CssCalculationSumOperator::Add, "em"),
    ] {
        assert!(value.literal_component().is_none());
        let calculation = value.calculation().expect("checked length calculation");
        assert_eq!(calculation.result_type(), CssCalculationType::Length);
        let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
            panic!("authored calc wrapper")
        };
        let CssCalculationExpressionRef::Sum(sum) = root.operand() else {
            panic!("ordered authored arithmetic")
        };
        assert_eq!(sum.len(), 2);
        assert!(sum.term(2).is_none());
        for (index, expected_operator, representation, unit) in
            [(0, None, "1", "px"), (1, Some(operator), "2", second_unit)]
        {
            let term = sum.term(index).unwrap();
            assert_eq!(term.operator(), expected_operator);
            let CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(operand)) =
                term.expression()
            else {
                panic!("exact authored length operand")
            };
            assert_eq!(operand.representation(), representation);
            assert_eq!(operand.unit(), Some(unit));
        }
    }
    let clip = CssClip::Rect(CssClipRect::new(
        edge(literal_math),
        edge(folded),
        edge(mixed),
        CssClipEdge::Auto,
    ));
    assert_eq!(
        clip.serialize_specified().unwrap(),
        "rect(calc(-2px), calc(-1px), calc(2em + 1px), auto)"
    );
    let parsed = parsed(&format!("clip:{}", clip.serialize_specified().unwrap()));
    let CssClip::Rect(rect) = parsed_clip(&parsed) else {
        panic!("math rectangle")
    };
    assert!(matches!(rect.top(), CssClipEdge::Length(length) if length.calculation().is_some()));
    assert!(matches!(rect.right(), CssClipEdge::Length(length) if length.calculation().is_some()));
    assert!(matches!(rect.bottom(), CssClipEdge::Length(length) if length.calculation().is_some()));
}

#[test]
fn initial_and_authored_expansion_have_typed_auto_and_rectangle_values() {
    let CssPropertyKindRef::Longhand(metadata) = CssKnownProperty::Clip.metadata().unwrap().kind()
    else {
        panic!("clip longhand")
    };
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("ordinary initial")
    };
    let CssLonghandValueRef::Clip(initial_clip) = value.view() else {
        panic!("typed initial")
    };
    assert_eq!(initial_clip.serialize_specified().unwrap(), "auto");

    let authored = parsed("clip:rect(auto -1px 2em 0)!important");
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(&authored).unwrap()
    else {
        panic!("clip longhand contribution")
    };
    let [item] = items.items() else {
        panic!("one contribution")
    };
    let CssLonghandValueRef::Clip(clip) = item.ordinary_value().unwrap().view() else {
        panic!("typed clip contribution")
    };
    assert_eq!(
        clip.serialize_specified().unwrap(),
        "rect(auto, -1px, 2em, 0)"
    );
    assert!(item.source().same_occurrence(&authored));
    assert_eq!(item.source().importance(), CssImportance::Important);
}

#[test]
fn edge_and_rectangle_limits_share_exact_cumulative_budgets() {
    let one = CssClipEdge::Auto;
    assert_eq!(
        one.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 4))
            .unwrap(),
        "auto"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 4),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 4),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 3),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            one.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    let rect = CssClipRect::new(
        CssClipEdge::Auto,
        edge(signed_length("-1px")),
        edge(signed_length("2em")),
        edge(signed_length("0")),
    );
    let expected = "rect(auto, -1px, 2em, 0)";
    assert_eq!(expected.len(), 24);
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(5, 5, 24))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(4, 5, 24),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 4, 24),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 5, 23),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            rect.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(
        rect.serialize_specified().unwrap(),
        expected,
        "failed serialization leaves value unchanged"
    );
}

#[test]
fn symbolic_math_edges_obey_one_shared_rectangle_byte_budget() {
    let mixed = edge(signed_length("calc(1px + 2em)"));
    assert_eq!(mixed.serialize_specified().unwrap(), "calc(2em + 1px)");
    assert_eq!(
        mixed
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                1_000, 1_000, 15
            ))
            .unwrap(),
        "calc(2em + 1px)"
    );
    let rect = CssClipRect::new(mixed.clone(), mixed, CssClipEdge::Auto, CssClipEdge::Auto);
    let expected = "rect(calc(2em + 1px), calc(2em + 1px), auto, auto)";
    assert_eq!(rect.serialize_specified().unwrap(), expected);
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1_000,
            1_000,
            expected.len()
        ))
        .unwrap(),
        expected
    );
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1_000, 1_000, 15
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            0, 1_000, 1_000
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1_000, 0, 1_000
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
fn signed_length(css: &str) -> surgeist_css::CssSpecifiedLength {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedLength::try_from_calculation(
            surgeist_css::CssLengthCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedLength::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}

fn exact_literal(component: Option<&surgeist_css::CssComponentValue>, css: &str) -> bool {
    use surgeist_css::{CssComponentValueRef as Component, CssValueTokenRef as Token};
    let expected = surgeist_css::CssComponentValue::try_token(css).unwrap();
    match (
        component.map(surgeist_css::CssComponentValue::view),
        expected.view(),
    ) {
        (
            Some(Component::Token(Token::Number(actual))),
            Component::Token(Token::Number(expected)),
        )
        | (
            Some(Component::Token(Token::Percentage(actual))),
            Component::Token(Token::Percentage(expected)),
        ) => actual.representation() == expected.representation(),
        (
            Some(Component::Token(Token::Dimension {
                number: actual,
                unit: actual_unit,
            })),
            Component::Token(Token::Dimension {
                number: expected,
                unit: expected_unit,
            }),
        ) => actual.representation() == expected.representation() && actual_unit == expected_unit,
        _ => false,
    }
}

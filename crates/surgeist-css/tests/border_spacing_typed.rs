#![forbid(unsafe_code)]

//! Functional checked construction and specified serialization of CSS2 §17.6.1
//! `border-spacing` (selected 2011-06-07 Recommendation).
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#propdef-border-spacing

use surgeist_css::*;

fn px(value: f32) -> CssLength {
    CssLength::try_px(value).unwrap()
}

fn em(value: f32) -> CssLength {
    CssLength::try_dimension(value, CssLengthUnit::Em).unwrap()
}

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("border-spacing expands to one terminal")
    };
    let [item] = items.items() else {
        panic!("one border-spacing terminal")
    };
    item.clone()
}

fn typed(value: &CssLonghandValue) -> &CssBorderSpacing {
    let CssLonghandValueRef::BorderSpacing(spacing) = value.view() else {
        panic!("typed border-spacing longhand")
    };
    spacing
}

#[test]
fn checked_pair_construction_accepts_zero_absolute_relative_and_symbolic_math() {
    for (horizontal, vertical, expected) in [
        (CssLength::Zero, CssLength::Zero, "0 0"),
        (px(2.0), em(3.0), "2px 3em"),
        (em(1.5), px(4.0), "1.5em 4px"),
    ] {
        let pair = CssBorderSpacing::try_new(horizontal.clone(), vertical.clone()).unwrap();
        assert_eq!(pair.horizontal().value(), &horizontal);
        assert_eq!(pair.vertical().value(), &vertical);
        assert_eq!(pair.serialize_specified().unwrap(), expected);
    }
    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc(1px + 2em)").unwrap(),
    )
    .unwrap();
    let symbolic = CssLength::Calc(CssCalcLength::Typed(calculation));
    let pair = CssBorderSpacing::try_new(symbolic.clone(), px(3.0)).unwrap();
    assert_eq!(pair.horizontal().value(), &symbolic);
    assert!(matches!(
        pair.horizontal().value(),
        CssLength::Calc(CssCalcLength::Typed(_))
    ));
    // CSS Values 4 §10.13 sorts dimension terms by unit when serializing calc().
    assert_eq!(pair.serialize_specified().unwrap(), "calc(2em + 1px) 3px");
    let negative_math = CssLength::Calc(CssCalcLength::Typed(
        CssLengthPercentageCalculation::try_from_components(
            parse_component_values("calc(1px - 2px)").unwrap(),
        )
        .unwrap(),
    ));
    let pair = CssBorderSpacing::try_new(negative_math.clone(), CssLength::Zero).unwrap();
    assert_eq!(pair.horizontal().value(), &negative_math);
    assert_eq!(pair.serialize_specified().unwrap(), "calc(-1px) 0");
}

#[test]
fn checked_single_length_rejects_bare_negative_percent_and_keyword_values() {
    for valid in [CssLength::Zero, px(2.0), em(3.0)] {
        let length = CssBorderSpacingLength::try_new(valid.clone()).unwrap();
        assert_eq!(length.value(), &valid);
    }
    for invalid in [
        px(-1.0),
        em(-2.0),
        CssLength::try_percent(1.0).unwrap(),
        CssLength::Auto,
    ] {
        assert!(CssBorderSpacingLength::try_new(invalid.clone()).is_none());
        assert!(CssBorderSpacing::try_new(invalid.clone(), CssLength::Zero).is_none());
        assert!(CssBorderSpacing::try_new(CssLength::Zero, invalid).is_none());
    }
    assert_eq!(
        CssBorderSpacingLength::try_new(px(2.0))
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "2px"
    );
}

#[test]
fn legacy_calc_wrappers_remain_symbolic_even_when_their_math_is_negative() {
    let negative = CssLength::Calc(CssCalcLength::try_px(-2.0).unwrap());
    let relative = CssLength::Calc(CssCalcLength::try_dimension(3.0, CssLengthUnit::Em).unwrap());
    let pair = CssBorderSpacing::try_new(negative.clone(), relative.clone()).unwrap();
    assert_eq!(pair.horizontal().value(), &negative);
    assert_eq!(pair.vertical().value(), &relative);
    assert_eq!(pair.serialize_specified().unwrap(), "calc(-2px) calc(3em)");
    assert_eq!(
        CssBorderSpacingLength::try_new(negative)
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "calc(-2px)"
    );
}

#[test]
fn initial_and_authored_contributions_expose_exact_numeric_axes() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::BorderSpacing.metadata().unwrap().kind()
    else {
        panic!("border-spacing is one longhand")
    };
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("fixed initial")
    };
    assert_eq!(typed(initial).horizontal().value(), &CssLength::Zero);
    assert_eq!(typed(initial).vertical().value(), &CssLength::Zero);
    assert_eq!(typed(initial).serialize_specified().unwrap(), "0 0");

    for (source, horizontal, vertical) in [
        ("border-spacing:2px", "2px", "2px"),
        ("border-spacing:2px 3em", "2px", "3em"),
        (
            "border-spacing:calc(1px + 2em) 3px",
            "calc(2em + 1px)",
            "3px",
        ),
    ] {
        let declaration = declaration(source);
        let expanded = contribution(&declaration);
        let typed = typed(expanded.ordinary_value().unwrap());
        assert_eq!(
            typed.horizontal().serialize_specified().unwrap(),
            horizontal
        );
        assert_eq!(typed.vertical().serialize_specified().unwrap(), vertical);
        assert_eq!(
            typed.serialize_specified().unwrap(),
            format!("{horizontal} {vertical}")
        );
    }
}

#[test]
fn pending_reentry_exposes_effective_axes_with_original_source() {
    let source = declaration("border-spacing:var(--gap)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending substitution")
    };
    let replacement = parse_component_values("2px 3em").unwrap();
    let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("reentered terminal")
    };
    let [item] = items.items() else {
        panic!("one border-spacing contribution")
    };
    let value = typed(item.ordinary_value().unwrap());
    assert_eq!(value.horizontal().serialize_specified().unwrap(), "2px");
    assert_eq!(value.vertical().serialize_specified().unwrap(), "3em");
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(&replacement));
}

#[test]
fn single_and_pair_serializers_enforce_exact_shared_resource_budgets() {
    let single = CssBorderSpacingLength::try_new(px(2.0)).unwrap();
    assert_eq!(
        single
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 3))
            .unwrap(),
        "2px"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 3),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 3),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 2),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            single
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }

    let pair = CssBorderSpacing::try_new(px(2.0), em(3.0)).unwrap();
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 7))
            .unwrap(),
        "2px 3em"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 7),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 7),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 6),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            pair.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}

#[test]
fn two_typed_calculation_axes_share_one_streaming_serialization_budget() {
    let first = CssLength::Calc(CssCalcLength::Typed(
        CssLengthPercentageCalculation::try_from_components(
            parse_component_values("calc(1px + 2em)").unwrap(),
        )
        .unwrap(),
    ));
    let second = CssLength::Calc(CssCalcLength::Typed(
        CssLengthPercentageCalculation::try_from_components(
            parse_component_values("calc(3px + 4em)").unwrap(),
        )
        .unwrap(),
    ));
    for axis in [&first, &second] {
        let axis = CssBorderSpacingLength::try_new(axis.clone()).unwrap();
        assert!(
            axis.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                1_000, 1_000, 15
            ))
            .is_ok()
        );
        assert_eq!(
            axis.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                0, 1_000, 1_000
            ))
            .unwrap_err()
            .kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );
        assert_eq!(
            axis.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                1_000, 0, 1_000
            ))
            .unwrap_err()
            .kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
    }
    let pair = CssBorderSpacing::try_new(first, second).unwrap();
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1_000, 1_000, 31
        ))
        .unwrap(),
        "calc(2em + 1px) calc(4em + 3px)"
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1_000, 1_000, 15
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}

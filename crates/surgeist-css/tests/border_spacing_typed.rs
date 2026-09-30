#![forbid(unsafe_code)]

//! Functional checked construction and specified serialization of CSS2 §17.6.1
//! `border-spacing` (selected 2011-06-07 Recommendation).
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#propdef-border-spacing

use surgeist_css::*;

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
        (nonnegative_length("0"), nonnegative_length("0"), "0 0"),
        (
            nonnegative_length("2px"),
            nonnegative_length("3em"),
            "2px 3em",
        ),
        (
            nonnegative_length("1.5em"),
            nonnegative_length("4px"),
            "1.5em 4px",
        ),
    ] {
        let pair = CssBorderSpacing::new(horizontal.clone(), vertical.clone());
        assert_eq!(pair.horizontal(), &horizontal);
        assert_eq!(pair.vertical(), &vertical);
        assert_eq!(pair.serialize_specified().unwrap(), expected);
    }
    let calculation = CssLengthCalculation::try_from_components(
        parse_component_values("calc(1px + 2em)").unwrap(),
    )
    .unwrap();
    let symbolic = CssSpecifiedNonNegativeLength::try_from_calculation(calculation).unwrap();
    let pair = CssBorderSpacing::new(symbolic.clone(), nonnegative_length("3px"));
    assert_eq!(pair.horizontal(), &symbolic);
    assert!(pair.horizontal().calculation().is_some());
    // CSS Values 4 §10.13 sorts dimension terms by unit when serializing calc().
    assert_eq!(pair.serialize_specified().unwrap(), "calc(2em + 1px) 3px");
    let negative_math = nonnegative_length("calc(1px - 2px)");
    let pair = CssBorderSpacing::new(negative_math.clone(), nonnegative_length("0"));
    assert_eq!(pair.horizontal(), &negative_math);
    assert_eq!(pair.serialize_specified().unwrap(), "calc(-1px) 0");
}

#[test]
fn checked_single_length_rejects_bare_negative_percent_and_keyword_values() {
    for css in ["0", "2px", "3em"] {
        let value = nonnegative_length(css);
        assert_eq!(value.serialize_specified().unwrap(), css);
    }
    for css in ["-1px", "-2em", "1%", "auto"] {
        assert!(
            CssSpecifiedNonNegativeLength::try_from_component(
                CssComponentValue::try_token(css).unwrap()
            )
            .is_err(),
            "{css}"
        );
        assert!(!parse_style_attribute(&format!("border-spacing:{css} 0")).is_clean());
        assert!(!parse_style_attribute(&format!("border-spacing:0 {css}")).is_clean());
    }
}

#[test]
fn checked_calculations_remain_symbolic_even_when_their_math_is_negative() {
    let negative = nonnegative_length("calc(-2px)");
    let relative = nonnegative_length("calc(3em)");
    let pair = CssBorderSpacing::new(negative.clone(), relative.clone());
    assert_eq!(pair.horizontal(), &negative);
    assert_eq!(pair.vertical(), &relative);
    assert_eq!(pair.serialize_specified().unwrap(), "calc(-2px) calc(3em)");
    assert_eq!(
        negative.clone().serialize_specified().unwrap(),
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
    assert_eq!(typed(initial).horizontal(), &nonnegative_length("0"));
    assert_eq!(typed(initial).vertical(), &nonnegative_length("0"));
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
    let single = nonnegative_length("2px");
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

    let pair = CssBorderSpacing::new(nonnegative_length("2px"), nonnegative_length("3em"));
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
    let first = nonnegative_length("calc(1px + 2em)");
    let second = nonnegative_length("calc(3px + 4em)");
    for axis in [&first, &second] {
        let axis = axis.clone();
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
    let pair = CssBorderSpacing::new(first, second);
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

fn nonnegative_length(css: &str) -> surgeist_css::CssSpecifiedNonNegativeLength {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedNonNegativeLength::try_from_calculation(
            surgeist_css::CssLengthCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedNonNegativeLength::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}

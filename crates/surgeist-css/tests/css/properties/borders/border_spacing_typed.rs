//! Functional checked construction and specified serialization of CSS2 §17.6.1
//! `border-spacing` (selected 2011-06-07 Recommendation).
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#propdef-border-spacing

use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::property_admission::{
    assert_component_rejected, assert_style_attribute_rejected,
};
use crate::css::support::sole_declaration as declaration;
use surgeist_css::*;

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) = source.expand().unwrap()
    else {
        panic!("border-spacing expands to one terminal")
    };
    let [item] = items.items() else {
        panic!("one border-spacing terminal")
    };
    item.clone()
}

fn typed(value: &CssLonghandValue) -> &CssSidePair<CssNumeric<CssLength, CssNonNegative>> {
    let CssLonghandValueRef::BorderSpacing(spacing) = value.view() else {
        panic!("typed border-spacing longhand")
    };
    spacing
}

#[test]
fn checked_pair_construction_accepts_zero_absolute_relative_and_symbolic_math() {
    for (horizontal, vertical, expected) in [
        (nonnegative_length("0"), nonnegative_length("0"), "0"),
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
        let pair = CssSidePair::new(horizontal.clone(), Some(vertical.clone()));
        assert_eq!(pair.first(), &horizontal);
        assert_eq!(pair.second(), &vertical);
        assert_eq!(
            pair.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
    let calculation = CssCalculation::<CssLength>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "calc(1px + 2em)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let symbolic =
        CssNumeric::<CssLength, CssNonNegative>::try_from_calculation(calculation).unwrap();
    let pair = CssSidePair::new(symbolic.clone(), Some(nonnegative_length("3px")));
    assert_eq!(pair.first(), &symbolic);
    assert!(pair.first().calculation().is_some());
    // CSS Values 4 §10.13 sorts dimension terms by unit when serializing calc().
    assert_eq!(
        pair.serialize_specified(Default::default()).unwrap(),
        "calc(2em + 1px) 3px"
    );
    let negative_math = nonnegative_length("calc(1px - 2px)");
    let pair = CssSidePair::new(negative_math.clone(), Some(nonnegative_length("0")));
    assert_eq!(pair.first(), &negative_math);
    assert_eq!(
        pair.serialize_specified(Default::default()).unwrap(),
        "calc(-1px) 0"
    );
}

#[test]
fn checked_single_length_rejects_bare_negative_percent_and_keyword_values() {
    for css in ["0", "2px", "3em"] {
        let value = nonnegative_length(css);
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), css);
    }
    for (css, kind) in [
        ("-1px", NumericFailure::OutOfRange),
        ("-2em", NumericFailure::OutOfRange),
        ("1%", NumericFailure::RootDomainMismatch),
        ("auto", NumericFailure::RootDomainMismatch),
    ] {
        assert_component_rejected(
            CssNumeric::<CssLength, CssNonNegative>::try_from_component,
            CssComponentValue::try_token(css).unwrap(),
            kind,
        );
        for pair in [
            format!("border-spacing:{css} 0"),
            format!("border-spacing:0 {css}"),
        ] {
            assert_style_attribute_rejected(&pair, CssKnownProperty::BorderSpacing, Some(css));
        }
    }
}

#[test]
fn checked_calculations_remain_symbolic_even_when_their_math_is_negative() {
    let negative = nonnegative_length("calc(-2px)");
    let relative = nonnegative_length("calc(3em)");
    let pair = CssSidePair::new(negative.clone(), Some(relative.clone()));
    assert_eq!(pair.first(), &negative);
    assert_eq!(pair.second(), &relative);
    assert_eq!(
        pair.serialize_specified(Default::default()).unwrap(),
        "calc(-2px) calc(3em)"
    );
    assert_eq!(
        negative
            .clone()
            .serialize_specified(Default::default())
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
    assert_eq!(typed(initial).first(), &nonnegative_length("0"));
    assert_eq!(typed(initial).second(), &nonnegative_length("0"));
    assert_eq!(
        typed(initial)
            .serialize_specified(Default::default())
            .unwrap(),
        "0"
    );

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
            typed
                .first()
                .serialize_specified(Default::default())
                .unwrap(),
            horizontal
        );
        assert_eq!(
            typed
                .second()
                .serialize_specified(Default::default())
                .unwrap(),
            vertical
        );
        // Equal axes write one length.
        let expected = if horizontal == vertical {
            horizontal.to_owned()
        } else {
            format!("{horizontal} {vertical}")
        };
        assert_eq!(
            typed.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
}

#[test]
fn pending_reentry_exposes_effective_axes_with_original_source() {
    let source = declaration("border-spacing:var(--gap)!important");
    let CssExpansion::Pending(pending) = source.expand().unwrap() else {
        panic!("pending substitution")
    };
    let replacement = CssComponentValues::parse(&CssParser::default(), "2px 3em").unwrap();
    let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("reentered terminal")
    };
    let [item] = items.items() else {
        panic!("one border-spacing contribution")
    };
    let value = typed(item.ordinary_value().unwrap());
    assert_eq!(
        value
            .first()
            .serialize_specified(Default::default())
            .unwrap(),
        "2px"
    );
    assert_eq!(
        value
            .second()
            .serialize_specified(Default::default())
            .unwrap(),
        "3em"
    );
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(&replacement));
}

#[test]
fn single_and_pair_serializers_enforce_exact_shared_resource_budgets() {
    let single = nonnegative_length("2px");
    assert_eq!(
        single
            .serialize_specified(CssSerializationLimits::new(1, 1, 3))
            .unwrap(),
        "2px"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(0, 1, 3),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 0, 3),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 1, 2),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&single.serialize_specified(limits).unwrap_err()),
            kind
        );
    }

    let pair = CssSidePair::new(nonnegative_length("2px"), Some(nonnegative_length("3em")));
    assert_eq!(
        pair.serialize_specified(CssSerializationLimits::new(3, 3, 7))
            .unwrap(),
        "2px 3em"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(2, 3, 7),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 7),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 6),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&pair.serialize_specified(limits).unwrap_err()),
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
            axis.serialize_specified(CssSerializationLimits::new(1_000, 1_000, 15))
                .is_ok()
        );
        assert!(
            axis.serialize_specified(CssSerializationLimits::new(0, 1_000, 1_000))
                .unwrap_err()
                .is_input_node_limit()
        );
        assert!(
            axis.serialize_specified(CssSerializationLimits::new(1_000, 0, 1_000))
                .unwrap_err()
                .is_projection_node_limit()
        );
    }
    let pair = CssSidePair::new(first, Some(second));
    assert_eq!(
        pair.serialize_specified(CssSerializationLimits::new(1_000, 1_000, 31))
            .unwrap(),
        "calc(2em + 1px) calc(4em + 3px)"
    );
    assert!(
        pair.serialize_specified(CssSerializationLimits::new(1_000, 1_000, 15))
            .unwrap_err()
            .is_byte_limit()
    );
}

fn nonnegative_length(
    css: &str,
) -> surgeist_css::CssNumeric<surgeist_css::CssLength, surgeist_css::CssNonNegative> {
    let components =
        surgeist_css::CssComponentValues::parse(&surgeist_css::CssParser::default(), css).unwrap();
    if css.contains('(') {
        surgeist_css::CssNumeric::<surgeist_css::CssLength, surgeist_css::CssNonNegative>::try_from_calculation(
            surgeist_css::CssCalculation::<surgeist_css::CssLength>::try_from_components(components, Default::default())
                .unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssNumeric::<surgeist_css::CssLength, surgeist_css::CssNonNegative>::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}

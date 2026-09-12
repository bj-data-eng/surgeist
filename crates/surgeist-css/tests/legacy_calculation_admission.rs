#![forbid(unsafe_code)]
//! Values 4 (2024-03-12), section 10.8, requires a calc-sum to begin
//! with an operand. In the public legacy representation that is an Add term,
//! whose operand may itself be signed; every nested sum has the same invariant.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-syntax
//! CssFlowTolerance::try_length_percentage already documents this distinction.
use surgeist_css::{
    CssCalcLength, CssCalcLengthTerm, CssLength, CssLengthPercentageCalculation,
    CssNonNegativeLength, CssTransformLength, CssTransformLengthPercentage,
    CssTransformNonNegativeLength, parse_component_values,
};

fn px(value: f32) -> CssCalcLength {
    CssCalcLength::try_px(value).unwrap()
}

fn empty() -> CssCalcLength {
    CssCalcLength::Sum(vec![])
}

fn leading_subtraction() -> CssCalcLength {
    CssCalcLength::sum(CssCalcLengthTerm::sub(px(1.0)), [])
}

// Each boundary gets a separately executed witness. All APIs already exist;
// failures report real admission of malformed public values, not compilation.
macro_rules! malformed_sums {
    ($consumer:ty, $empty:ident, $leading:ident, $nested_first:ident, $nested_later:ident) => {
        #[test]
        fn $empty() {
            assert!(<$consumer>::try_new(CssLength::Calc(empty())).is_none());
        }

        #[test]
        fn $leading() {
            assert!(<$consumer>::try_new(CssLength::Calc(leading_subtraction())).is_none());
        }

        #[test]
        fn $nested_first() {
            for malformed in [empty(), leading_subtraction()] {
                let sum = CssCalcLength::sum(
                    CssCalcLengthTerm::add(malformed),
                    [CssCalcLengthTerm::sub(px(2.0))],
                );
                assert!(<$consumer>::try_new(CssLength::Calc(sum)).is_none());
            }
        }

        #[test]
        fn $nested_later() {
            for malformed in [empty(), leading_subtraction()] {
                let sum = CssCalcLength::sum(
                    CssCalcLengthTerm::add(px(2.0)),
                    [CssCalcLengthTerm::sub(malformed)],
                );
                assert!(<$consumer>::try_new(CssLength::Calc(sum)).is_none());
            }
        }
    };
}

malformed_sums!(
    CssTransformLengthPercentage,
    transform_length_percentage_rejects_empty_sum,
    transform_length_percentage_rejects_leading_subtraction,
    transform_length_percentage_rejects_malformed_first_descendant,
    transform_length_percentage_rejects_malformed_subtracted_descendant
);
malformed_sums!(
    CssTransformLength,
    transform_length_rejects_empty_sum,
    transform_length_rejects_leading_subtraction,
    transform_length_rejects_malformed_first_descendant,
    transform_length_rejects_malformed_subtracted_descendant
);
malformed_sums!(
    CssTransformNonNegativeLength,
    transform_nonnegative_length_rejects_empty_sum,
    transform_nonnegative_length_rejects_leading_subtraction,
    transform_nonnegative_length_rejects_malformed_first_descendant,
    transform_nonnegative_length_rejects_malformed_subtracted_descendant
);
malformed_sums!(
    CssNonNegativeLength,
    nonnegative_length_rejects_empty_sum,
    nonnegative_length_rejects_leading_subtraction,
    nonnegative_length_rejects_malformed_first_descendant,
    nonnegative_length_rejects_malformed_subtracted_descendant
);

fn assert_admitted_unchanged(value: CssCalcLength, expected: &str) {
    let value = CssLength::Calc(value);
    let values = [
        CssTransformLengthPercentage::try_new(value.clone())
            .expect("valid authored length-percentage calculation")
            .value()
            .clone(),
        CssTransformLength::try_new(value.clone())
            .expect("valid authored pure-length calculation")
            .value()
            .clone(),
        CssTransformNonNegativeLength::try_new(value.clone())
            .expect("symbolic transform calculation is not range-evaluated here")
            .value()
            .clone(),
        CssNonNegativeLength::try_new(value)
            .expect("symbolic calculation is not range-evaluated here")
            .value()
            .clone(),
    ];
    for value in values {
        let CssLength::Calc(calc) = value else {
            panic!("calculation remains symbolic")
        };
        assert_eq!(calc.to_css_string(), expected);
    }
}

#[test]
fn a_signed_first_add_operand_remains_valid_without_range_evaluation() {
    assert_admitted_unchanged(
        CssCalcLength::sum(CssCalcLengthTerm::add(px(-1.0)), []),
        "calc(-1px)",
    );
}

#[test]
fn later_subtraction_and_signed_nested_first_operands_remain_valid() {
    let child = CssCalcLength::sum(CssCalcLengthTerm::add(px(-1.0)), []);
    let sum = CssCalcLength::sum(
        CssCalcLengthTerm::add(child),
        [CssCalcLengthTerm::sub(px(2.0))],
    );
    assert_admitted_unchanged(sum, "calc(calc(-1px) - 2px)");
}

#[test]
fn checked_typed_descendants_remain_valid_and_symbolic_inside_a_legacy_sum() {
    let typed = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc(1em + 2px)").unwrap(),
    )
    .unwrap();
    let sum = CssCalcLength::sum(
        CssCalcLengthTerm::add(CssCalcLength::Typed(typed)),
        [CssCalcLengthTerm::sub(px(3.0))],
    );
    assert_admitted_unchanged(sum, "calc(calc(1em + 2px) - 3px)");
}

#[test]
fn pure_readmission_preserves_typed_descendant_components_and_original_snapshots() {
    use surgeist_css::CssValueOrigin;
    let components = parse_component_values("calc(/*😀*/1em + 2px)").unwrap();
    let typed = CssLengthPercentageCalculation::try_from_components(components.clone()).unwrap();
    let sum = CssLength::Calc(CssCalcLength::sum(
        CssCalcLengthTerm::add(CssCalcLength::Typed(typed)),
        [],
    ));
    let admitted = [
        CssTransformLength::try_new(sum.clone())
            .unwrap()
            .value()
            .clone(),
        CssTransformNonNegativeLength::try_new(sum.clone())
            .unwrap()
            .value()
            .clone(),
        CssNonNegativeLength::try_new(sum).unwrap().value().clone(),
    ];
    for value in admitted {
        let CssLength::Calc(CssCalcLength::Sum(terms)) = value else {
            panic!("retained sum")
        };
        let CssCalcLength::Typed(typed) = terms[0].value() else {
            panic!("retained typed child")
        };
        assert_eq!(typed.components(), &components);
        let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) =
            (typed.origin(), components.items().last().unwrap().origin())
        else {
            panic!("original parsed calc")
        };
        assert!(actual.source().same_snapshot(expected.source()));
        assert_eq!(actual.span(), expected.span());
    }
}

// Each entry injects only the calculation under test. Other arguments satisfy
// the owner's existing structure, cardinality, and range requirements.
type CheckedOwner = (&'static str, fn(CssLength) -> bool);

fn checked_owners() -> Vec<CheckedOwner> {
    use surgeist_css::*;
    vec![
        ("outline offset", |v| CssOutlineOffset::try_new(v).is_some()),
        ("border spacing length", |v| {
            CssBorderSpacingLength::try_new(v).is_some()
        }),
        ("clip length", |v| CssClipLength::try_new(v).is_some()),
        ("word spacing", |v| {
            CssWordSpacingLength::try_new(v).is_some()
        }),
        ("letter spacing", |v| {
            CssLetterSpacingLength::try_new(v).is_some()
        }),
        ("border image outset", |v| {
            CssBorderImageOutsetLength::try_new(v).is_some()
        }),
        ("radial circle", |v| {
            CssRadialCircleSize::try_new(v).is_some()
        }),
        ("transform origin z", |v| {
            CssTransformOriginZ::try_new(v).is_some()
        }),
        ("filter blur", |v| CssFilterBlur::try_new(v).is_some()),
        ("shape length", |v| CssShapeLength::try_new(v).is_some()),
        ("border", |v| {
            CssBorder::try_new(Some(v), None, None).is_some()
        }),
        ("shadow x", |v| {
            CssShadow::try_new(false, v, CssLength::Zero, None, None, None).is_some()
        }),
        ("shadow y", |v| {
            CssShadow::try_new(false, CssLength::Zero, v, None, None, None).is_some()
        }),
        ("shadow blur", |v| {
            CssShadow::try_new(false, CssLength::Zero, CssLength::Zero, Some(v), None, None)
                .is_some()
        }),
        ("shadow spread", |v| {
            CssShadow::try_new(
                false,
                CssLength::Zero,
                CssLength::Zero,
                Some(CssLength::Zero),
                Some(v),
                None,
            )
            .is_some()
        }),
        ("drop shadow x", |v| {
            CssDropShadow::try_new(v, CssLength::Zero, None, None).is_some()
        }),
        ("drop shadow y", |v| {
            CssDropShadow::try_new(CssLength::Zero, v, None, None).is_some()
        }),
        ("drop shadow blur", |v| {
            CssDropShadow::try_new(CssLength::Zero, CssLength::Zero, Some(v), None).is_some()
        }),
        ("text indent", |v| {
            CssTextIndent::try_new(v, false, false).is_some()
        }),
        ("vertical align", |v| {
            CssVerticalAlignLength::try_new(v).is_some()
        }),
        ("font size", |v| {
            CssFontSizeLengthPercentage::try_new(v).is_some()
        }),
        ("line height", |v| {
            CssLineHeightLengthPercentage::try_new(v).is_some()
        }),
        ("decoration thickness", |v| {
            CssTextDecorationThicknessLength::try_new(v).is_some()
        }),
        ("border image width", |v| {
            CssBorderImageWidthLengthPercentage::try_new(v).is_some()
        }),
        ("gradient position", |v| {
            CssGradientLinePosition::try_new(v).is_some()
        }),
        ("position offset", |v| {
            CssPositionOffset::try_new(v).is_some()
        }),
        ("shape length percentage", |v| {
            CssShapeLengthPercentage::try_new(v).is_some()
        }),
        ("corner horizontal", |v| {
            CssCornerRadius::try_new(v, CssLength::Zero).is_some()
        }),
        ("corner vertical", |v| {
            CssCornerRadius::try_new(CssLength::Zero, v).is_some()
        }),
        ("radial ellipse horizontal", |v| {
            CssRadialEllipseSize::try_new(v, CssLength::Zero).is_some()
        }),
        ("radial ellipse vertical", |v| {
            CssRadialEllipseSize::try_new(CssLength::Zero, v).is_some()
        }),
        ("inset offsets", |v| {
            CssInsetShapeOffsets::try_new(vec![v]).is_some()
        }),
        ("polygon x", |v| {
            CssPolygonPoint::try_new(v, CssLength::Zero).is_some()
        }),
        ("polygon y", |v| {
            CssPolygonPoint::try_new(CssLength::Zero, v).is_some()
        }),
        ("border spacing horizontal", |v| {
            CssBorderSpacing::try_new(v, CssLength::Zero).is_some()
        }),
        ("border spacing vertical", |v| {
            CssBorderSpacing::try_new(CssLength::Zero, v).is_some()
        }),
        ("ellipse horizontal", |v| {
            CssEllipseRadii::try_new(v, CssLength::Zero).is_some()
        }),
        ("ellipse vertical", |v| {
            CssEllipseRadii::try_new(CssLength::Zero, v).is_some()
        }),
        ("translate values", |v| {
            CssTranslateValues::try_new(vec![v]).is_some()
        }),
        ("position", |v| {
            CssPosition::try_new(vec![CssPositionComponent::Length(v)]).is_some()
        }),
        ("outline", |v| {
            CssOutline::try_new(Some(CssOutlineWidth::Length(v)), None, None).is_some()
        }),
        ("background width", |v| {
            CssBackgroundSizeList::try_new(vec![CssBackgroundSize::Explicit {
                width: CssBackgroundSizeComponent::Length(v),
                height: None,
            }])
            .is_some()
        }),
        ("background height", |v| {
            CssBackgroundSizeList::try_new(vec![CssBackgroundSize::Explicit {
                width: CssBackgroundSizeComponent::Auto,
                height: Some(CssBackgroundSizeComponent::Length(v)),
            }])
            .is_some()
        }),
        ("grid fit content", |v| {
            CssGridTrackList::try_new(vec![CssGridTrackComponent::TrackSize(
                CssGridTrackSize::FitContent(v),
            )])
            .is_some()
        }),
        ("grid breadth", |v| {
            CssGridTrackList::try_new(vec![CssGridTrackComponent::TrackSize(
                CssGridTrackSize::Breadth(CssGridTrackBreadth::Length(v)),
            )])
            .is_some()
        }),
        ("grid minimum", |v| {
            CssGridTrackList::try_new(vec![CssGridTrackComponent::TrackSize(
                CssGridTrackSize::MinMax {
                    min: CssGridTrackBreadth::Length(v),
                    max: CssGridTrackBreadth::Auto,
                },
            )])
            .is_some()
        }),
        ("grid maximum", |v| {
            CssGridTrackList::try_new(vec![CssGridTrackComponent::TrackSize(
                CssGridTrackSize::MinMax {
                    min: CssGridTrackBreadth::Auto,
                    max: CssGridTrackBreadth::Length(v),
                },
            )])
            .is_some()
        }),
    ]
}

fn assert_all_owners_reject(calc: CssCalcLength) {
    let admitted: Vec<_> = checked_owners()
        .into_iter()
        .filter_map(|(name, admit)| admit(CssLength::Calc(calc.clone())).then_some(name))
        .collect();
    assert!(
        admitted.is_empty(),
        "malformed calculation admitted by {admitted:?}"
    );
}

#[test]
fn checked_length_owners_reject_empty_sum() {
    assert_all_owners_reject(empty());
}

#[test]
fn checked_length_owners_reject_leading_subtraction() {
    assert_all_owners_reject(leading_subtraction());
}

#[test]
fn checked_length_owners_reject_empty_first_descendant() {
    assert_all_owners_reject(CssCalcLength::sum(
        CssCalcLengthTerm::add(empty()),
        [CssCalcLengthTerm::sub(px(1.0))],
    ));
}

#[test]
fn checked_length_owners_reject_subtracting_first_descendant() {
    assert_all_owners_reject(CssCalcLength::sum(
        CssCalcLengthTerm::add(leading_subtraction()),
        [CssCalcLengthTerm::sub(px(1.0))],
    ));
}

#[test]
fn checked_length_owners_reject_empty_later_descendant() {
    assert_all_owners_reject(CssCalcLength::sum(
        CssCalcLengthTerm::add(px(2.0)),
        [CssCalcLengthTerm::sub(empty())],
    ));
}

#[test]
fn checked_length_owners_reject_subtracting_later_descendant() {
    assert_all_owners_reject(CssCalcLength::sum(
        CssCalcLengthTerm::add(px(2.0)),
        [CssCalcLengthTerm::sub(leading_subtraction())],
    ));
}

#[test]
fn checked_length_owner_controls_admit_valid_symbolic_subtraction() {
    let sum = CssCalcLength::sum(
        CssCalcLengthTerm::add(px(2.0)),
        [CssCalcLengthTerm::sub(px(1.0))],
    );
    let rejected: Vec<_> = checked_owners()
        .into_iter()
        .filter_map(|(name, admit)| (!admit(CssLength::Calc(sum.clone()))).then_some(name))
        .collect();
    assert!(
        rejected.is_empty(),
        "valid calculation rejected by {rejected:?}"
    );
}

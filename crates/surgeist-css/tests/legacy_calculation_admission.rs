#![forbid(unsafe_code)]
//! Checked typed calculations retain symbolic values across authored consumers.
//! Malformed raw sums are no longer constructible after their API retirement;
//! checked assembly rejection is covered by typed_sum_construction.
use surgeist_css::{
    CssCalcLength, CssCalculationSumOperator as Op, CssComponentValueRef, CssLength,
    CssLengthPercentageCalculation as Calculation, CssNonNegativeLength, CssTransformLength,
    CssTransformLengthPercentage, CssTransformNonNegativeLength, CssValueOrigin,
    parse_component_values,
};

fn operand(source: &str) -> Calculation {
    Calculation::try_from_components(parse_component_values(source).unwrap()).unwrap()
}
fn admitted(value: Calculation) -> Vec<CssLength> {
    let value = CssLength::Calc(CssCalcLength::Typed(value));
    vec![
        CssTransformLengthPercentage::try_new(value.clone())
            .unwrap()
            .value()
            .clone(),
        CssTransformLength::try_new(value.clone())
            .unwrap()
            .value()
            .clone(),
        CssTransformNonNegativeLength::try_new(value.clone())
            .unwrap()
            .value()
            .clone(),
        CssNonNegativeLength::try_new(value)
            .unwrap()
            .value()
            .clone(),
    ]
}
#[test]
fn signed_first_operands_and_later_subtraction_remain_symbolic() {
    let first = Calculation::try_sum(operand("-1px"), []).unwrap();
    let sum = Calculation::try_sum(first, [(Op::Subtract, operand("2px"))]).unwrap();
    for value in admitted(sum) {
        let CssLength::Calc(CssCalcLength::Typed(value)) = value else {
            panic!("typed calculation")
        };
        assert_eq!(
            value.serialize().unwrap().as_css(),
            "calc(calc(-1px) - 2px)"
        );
    }
}
#[test]
fn pure_readmission_preserves_typed_child_components_and_original_snapshots() {
    let child = operand("calc(10% / 10% * 1px)");
    let original = child.components().clone();
    let sum = Calculation::try_sum(child, []).unwrap();
    for value in admitted(sum).into_iter().skip(1) {
        let CssLength::Calc(CssCalcLength::Typed(typed)) = value else {
            panic!("typed calculation")
        };
        let CssComponentValueRef::Function(outer) = typed.components().items()[0].view() else {
            panic!("outer calc")
        };
        assert_eq!(outer.values(), &original);
        let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) = (
            outer.values().items()[0].origin(),
            original.items()[0].origin(),
        ) else {
            panic!("original child")
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

#[test]
fn checked_length_owners_admit_valid_typed_symbolic_subtraction() {
    let sum = Calculation::try_sum(operand("2px"), [(Op::Subtract, operand("1px"))]).unwrap();
    let rejected: Vec<_> = checked_owners()
        .into_iter()
        .filter_map(|(name, admit)| {
            (!admit(CssLength::Calc(CssCalcLength::Typed(sum.clone())))).then_some(name)
        })
        .collect();
    assert!(
        rejected.is_empty(),
        "valid calculation rejected by {rejected:?}"
    );
}

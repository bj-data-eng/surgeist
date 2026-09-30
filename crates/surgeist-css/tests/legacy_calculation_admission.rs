#![forbid(unsafe_code)]
//! Checked typed calculations retain symbolic values across authored consumers.
//! Checked assembly rejection is covered by typed_sum_construction.
use surgeist_css::CssCalculationSumOperator as Op;
use surgeist_css::*;
type Calculation = CssLengthPercentageCalculation;
fn operand(source: &str) -> Calculation {
    Calculation::try_from_components(parse_component_values(source).unwrap()).unwrap()
}
fn pure(value: Calculation) -> Result<CssSpecifiedLength, CssNumericConstructionError> {
    CssSpecifiedLength::try_from_calculation(CssLengthCalculation::try_from_components(
        value.components().clone(),
    )?)
}
fn nonnegative_pure(
    value: Calculation,
) -> Result<CssSpecifiedNonNegativeLength, CssNumericConstructionError> {
    CssSpecifiedNonNegativeLength::try_from_calculation(CssLengthCalculation::try_from_components(
        value.components().clone(),
    )?)
}
fn lp(value: Calculation) -> Result<CssSpecifiedLengthPercentage, CssNumericConstructionError> {
    CssSpecifiedLengthPercentage::try_from_calculation(value)
}
fn nonnegative_lp(
    value: Calculation,
) -> Result<CssSpecifiedNonNegativeLengthPercentage, CssNumericConstructionError> {
    CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(value)
}
fn zero_length() -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_component(CssComponentValue::try_number("0").unwrap()).unwrap()
}
fn zero_nonnegative_length() -> CssSpecifiedNonNegativeLength {
    CssSpecifiedNonNegativeLength::try_from_component(CssComponentValue::try_number("0").unwrap())
        .unwrap()
}
fn zero_lp() -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_component(CssComponentValue::try_number("0").unwrap())
        .unwrap()
}
fn zero_nonnegative_lp() -> CssSpecifiedNonNegativeLengthPercentage {
    CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_number("0").unwrap(),
    )
    .unwrap()
}
fn admitted(value: Calculation) -> Vec<CssComponentValues> {
    vec![
        lp(value.clone())
            .unwrap()
            .calculation()
            .unwrap()
            .components()
            .clone(),
        pure(value.clone())
            .unwrap()
            .calculation()
            .unwrap()
            .components()
            .clone(),
        nonnegative_pure(value)
            .unwrap()
            .calculation()
            .unwrap()
            .components()
            .clone(),
    ]
}
#[test]
fn signed_first_operands_and_later_subtraction_remain_symbolic() {
    let first = Calculation::try_sum(operand("-1px"), []).unwrap();
    let sum = Calculation::try_sum(first, [(Op::Subtract, operand("2px"))]).unwrap();
    for components in admitted(sum) {
        assert_eq!(
            components.serialize().unwrap().as_css(),
            "calc(calc(-1px) - 2px)"
        );
    }
}
#[test]
fn pure_readmission_preserves_typed_child_components_and_original_snapshots() {
    let child = operand("calc(10% / 10% * 1px)");
    let original = child.components().clone();
    let sum = Calculation::try_sum(child, []).unwrap();
    for components in admitted(sum).into_iter().skip(1) {
        let CssComponentValueRef::Function(outer) = components.items()[0].view() else {
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
// Each entry injects only the calculation under test; other components are valid.
type CheckedOwner = (&'static str, fn(Calculation) -> bool);
fn checked_owners() -> Vec<CheckedOwner> {
    vec![
        ("outline offset", |v| pure(v).is_ok()),
        ("border spacing length", |v| nonnegative_pure(v).is_ok()),
        ("clip length", |v| pure(v).is_ok()),
        ("border image outset", |v| nonnegative_pure(v).is_ok()),
        ("radial circle", |v| nonnegative_pure(v).is_ok()),
        ("transform origin z", |v| pure(v).is_ok()),
        ("filter blur", |v| {
            nonnegative_pure(v).map(CssFilterBlur::new).is_ok()
        }),
        ("shape length", |v| nonnegative_pure(v).is_ok()),
        ("border", |v| {
            nonnegative_pure(v)
                .ok()
                .and_then(|v| CssBorder::try_new(Some(CssBorderWidth::Length(v)), None, None))
                .is_some()
        }),
        ("shadow x", |v| {
            pure(v)
                .ok()
                .and_then(|v| CssShadow::try_new(false, v, zero_length(), None, None, None))
                .is_some()
        }),
        ("shadow y", |v| {
            pure(v)
                .ok()
                .and_then(|v| CssShadow::try_new(false, zero_length(), v, None, None, None))
                .is_some()
        }),
        ("shadow blur", |v| {
            nonnegative_pure(v)
                .ok()
                .and_then(|v| {
                    CssShadow::try_new(false, zero_length(), zero_length(), Some(v), None, None)
                })
                .is_some()
        }),
        ("shadow spread", |v| {
            pure(v)
                .ok()
                .and_then(|v| {
                    CssShadow::try_new(
                        false,
                        zero_length(),
                        zero_length(),
                        Some(zero_nonnegative_length()),
                        Some(v),
                        None,
                    )
                })
                .is_some()
        }),
        ("drop shadow x", |v| {
            pure(v)
                .map(|v| CssDropShadow::new(v, zero_length(), None, None))
                .is_ok()
        }),
        ("drop shadow y", |v| {
            pure(v)
                .map(|v| CssDropShadow::new(zero_length(), v, None, None))
                .is_ok()
        }),
        ("drop shadow blur", |v| {
            nonnegative_pure(v)
                .map(|v| CssDropShadow::new(zero_length(), zero_length(), Some(v), None))
                .is_ok()
        }),
        ("text indent", |v| {
            lp(v).map(|v| CssTextIndent::new(v, false, false)).is_ok()
        }),
        ("vertical align", |v| lp(v).is_ok()),
        ("decoration thickness", |v| nonnegative_lp(v).is_ok()),
        ("border image width", |v| nonnegative_lp(v).is_ok()),
        ("gradient position", |v| lp(v).is_ok()),
        ("position offset", |v| lp(v).is_ok()),
        ("shape length percentage", |v| nonnegative_lp(v).is_ok()),
        ("corner horizontal", |v| {
            nonnegative_lp(v)
                .map(|v| CssCornerRadiusValue::new(v, Some(zero_nonnegative_lp())))
                .is_ok()
        }),
        ("corner vertical", |v| {
            nonnegative_lp(v)
                .map(|v| CssCornerRadiusValue::new(zero_nonnegative_lp(), Some(v)))
                .is_ok()
        }),
        ("radial ellipse horizontal", |v| {
            nonnegative_lp(v)
                .map(|v| CssRadialEllipseSize::new(v, zero_nonnegative_lp()))
                .is_ok()
        }),
        ("radial ellipse vertical", |v| {
            nonnegative_lp(v)
                .map(|v| CssRadialEllipseSize::new(zero_nonnegative_lp(), v))
                .is_ok()
        }),
        ("inset offsets", |v| {
            lp(v)
                .ok()
                .and_then(|v| CssInsetShapeOffsets::try_new(vec![v]))
                .is_some()
        }),
        ("polygon x", |v| {
            lp(v).map(|v| CssPolygonPoint::new(v, zero_lp())).is_ok()
        }),
        ("polygon y", |v| {
            lp(v).map(|v| CssPolygonPoint::new(zero_lp(), v)).is_ok()
        }),
        ("border spacing horizontal", |v| {
            nonnegative_pure(v)
                .map(|v| CssBorderSpacing::new(v, zero_nonnegative_length()))
                .is_ok()
        }),
        ("border spacing vertical", |v| {
            nonnegative_pure(v)
                .map(|v| CssBorderSpacing::new(zero_nonnegative_length(), v))
                .is_ok()
        }),
        ("ellipse horizontal", |v| {
            nonnegative_lp(v)
                .map(|v| CssEllipseRadii::new(v, zero_nonnegative_lp()))
                .is_ok()
        }),
        ("ellipse vertical", |v| {
            nonnegative_lp(v)
                .map(|v| CssEllipseRadii::new(zero_nonnegative_lp(), v))
                .is_ok()
        }),
        ("translate values", |v| {
            lp(v)
                .ok()
                .and_then(|v| CssTranslateValues::try_new(v, None, None))
                .is_some()
        }),
        ("position", |v| {
            lp(v)
                .ok()
                .and_then(|v| {
                    CssPosition::try_new(
                        CssHorizontalPosition::Offset(v),
                        CssVerticalPosition::Center,
                    )
                })
                .is_some()
        }),
        ("outline", |v| {
            nonnegative_pure(v)
                .ok()
                .and_then(|v| CssOutline::try_new(Some(CssOutlineWidth::Length(v)), None, None))
                .is_some()
        }),
        ("background width", |v| {
            nonnegative_lp(v)
                .ok()
                .and_then(|v| {
                    CssBackgroundSizeList::try_new(vec![CssBackgroundSize::Explicit {
                        width: CssBackgroundSizeComponent::Length(v),
                        height: None,
                    }])
                })
                .is_some()
        }),
        ("background height", |v| {
            nonnegative_lp(v)
                .ok()
                .and_then(|v| {
                    CssBackgroundSizeList::try_new(vec![CssBackgroundSize::Explicit {
                        width: CssBackgroundSizeComponent::Auto,
                        height: Some(CssBackgroundSizeComponent::Length(v)),
                    }])
                })
                .is_some()
        }),
        ("grid fit content", |v| grid_track_accepts(v, 0)),
        ("grid breadth", |v| grid_track_accepts(v, 1)),
        ("grid minimum", |v| grid_track_accepts(v, 2)),
        ("grid maximum", |v| grid_track_accepts(v, 3)),
    ]
}
fn grid_track_accepts(calculation: Calculation, mode: u8) -> bool {
    let Ok(scalar) = nonnegative_lp(calculation) else {
        return false;
    };
    let breadth = CssGridTrackBreadth::from_length_percentage(scalar);
    let size = match mode {
        0 => CssGridTrackSize::from_fit_content(breadth.length_percentage().unwrap().clone()),
        1 => CssGridTrackSize::from_breadth(breadth),
        2 => CssGridTrackSize::try_minmax(breadth, CssGridTrackBreadth::auto()).unwrap(),
        _ => CssGridTrackSize::try_minmax(CssGridTrackBreadth::auto(), breadth).unwrap(),
    };
    size.serialize_specified().is_ok()
}
#[test]
fn checked_length_owners_admit_valid_typed_symbolic_subtraction() {
    let sum = Calculation::try_sum(operand("2px"), [(Op::Subtract, operand("1px"))]).unwrap();
    // Both spacing properties now consume the checked length-percentage owner
    // directly; preserve the same symbolic subtraction admission at that boundary.
    for name in ["word-spacing", "letter-spacing"] {
        use surgeist_css::*;
        let declaration = parse_property_value_for_grammar(
            CssPropertyGrammar::from_name(name).unwrap(),
            sum.components().clone(),
            CssImportance::Normal,
        )
        .unwrap();
        let spacing = match declaration.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::WordSpacing(value) => value.spacing(),
            CssKnownPropertyValueRef::LetterSpacing(value) => value.value(),
            _ => panic!("spacing owner"),
        };
        assert_eq!(
            spacing
                .length_percentage()
                .unwrap()
                .calculation()
                .unwrap()
                .components(),
            sum.components()
        );
        assert_eq!(spacing.serialize_specified().unwrap(), "calc(1px)");
    }
    let rejected: Vec<_> = checked_owners()
        .into_iter()
        .filter_map(|(name, admit)| (!admit(sum.clone())).then_some(name))
        .collect();
    assert!(
        rejected.is_empty(),
        "valid calculation rejected by {rejected:?}"
    );
}

#![forbid(unsafe_code)]

use surgeist_css::{
    CssCalculationExpressionRef, CssCalculationType, CssCalculationValueRef, CssComponentValue,
    CssFlexCalculation, CssGridAutoRepeat, CssGridAutoRepeatKind, CssGridAutoTrackComponent,
    CssGridAutoTrackList, CssGridFixedRepeatComponent, CssGridFixedRepeatContent, CssGridFixedSize,
    CssGridGeneralTrackComponent, CssGridGeneralTrackList, CssGridIntegerTrackRepeat,
    CssGridLineNames, CssGridTrackBreadth, CssGridTrackList, CssGridTrackRepeatComponent,
    CssGridTrackRepeatContent, CssGridTrackSize, CssGridTrackSizeList, CssKnownPropertyValueRef,
    CssLengthPercentageCalculation, CssNumericConstructionErrorKind, CssNumericDimension,
    CssPositiveIntegerLiteral, CssSpecifiedNonNegativeFlex,
    CssSpecifiedNonNegativeLengthPercentage, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, parse_component_values,
    parse_style_attribute,
};

fn flex(text: &str) -> CssSpecifiedNonNegativeFlex {
    CssSpecifiedNonNegativeFlex::try_from_component(
        CssComponentValue::try_dimension(text, "fr").unwrap(),
    )
    .unwrap()
}

fn length(text: &str) -> CssSpecifiedNonNegativeLengthPercentage {
    CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_dimension(text, "px").unwrap(),
    )
    .unwrap()
}

fn canonical_large(unit: &str) -> String {
    format!("1{}{unit}", "0".repeat(50))
}

fn canonical_tiny(unit: &str) -> String {
    format!("0.{}1{unit}", "0".repeat(49))
}

#[test]
fn exact_flex_scalar_has_checked_programmatic_origin() {
    for text in ["1e50", "1e-50"] {
        let scalar = flex(text);
        assert!(matches!(scalar.origin(), CssValueOrigin::Programmatic));
        let canonical = if text == "1e50" {
            canonical_large("fr")
        } else {
            canonical_tiny("fr")
        };
        assert_eq!(scalar.serialize_specified().unwrap(), canonical);
        let breadth = CssGridTrackBreadth::from_flex(scalar);
        assert_eq!(breadth.serialize_specified().unwrap(), canonical);
        assert!(breadth.flex().is_some());
    }
    let error = CssSpecifiedNonNegativeFlex::try_from_component(
        CssComponentValue::try_dimension("-1e-50", "fr").unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
    assert!(matches!(error.origin(), Some(CssValueOrigin::Programmatic)));

    let bare_negative =
        CssFlexCalculation::try_from_components(parse_component_values("-1fr").unwrap()).unwrap();
    assert_eq!(
        CssSpecifiedNonNegativeFlex::try_from_calculation(bare_negative)
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::OutOfRange,
    );
    let deferred_negative = CssSpecifiedNonNegativeFlex::try_from_calculation(
        CssFlexCalculation::try_from_components(parse_component_values("calc(-1fr)").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(deferred_negative.calculation().is_some());
    assert_eq!(
        deferred_negative.serialize_specified().unwrap(),
        "calc(-1fr)"
    );

    let zero = CssGridTrackBreadth::from_flex(flex("-0"));
    assert_eq!(zero.serialize_specified().unwrap(), "0fr");

    let scalar = flex("2");
    assert_eq!(
        scalar
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 3))
            .unwrap(),
        "2fr",
    );
    assert_eq!(
        scalar
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 0, 3))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
    );
}

#[test]
fn flex_calculation_keeps_its_unhinted_type_and_function_tree() {
    let calculation = CssFlexCalculation::try_from_components(
        parse_component_values("calc(1fr * (1% / 1%))").unwrap(),
    )
    .unwrap();
    assert_eq!(calculation.result_type(), CssCalculationType::Flex);
    assert_eq!(
        calculation
            .numeric_type()
            .exponent(CssNumericDimension::Flex),
        1
    );
    assert_eq!(calculation.numeric_type().percent_hint(), None);
    assert!(matches!(calculation.origin(), CssValueOrigin::Parsed(_)));
    let scalar = CssSpecifiedNonNegativeFlex::try_from_calculation(calculation).unwrap();
    assert!(scalar.literal_component().is_none());
    assert!(scalar.calculation().is_some());
    let breadth = CssGridTrackBreadth::from_flex(scalar);
    let canonical = breadth.serialize_specified().unwrap();
    assert_eq!(canonical, "calc(1fr)");
    let reparsed =
        CssFlexCalculation::try_from_components(parse_component_values(&canonical).unwrap())
            .unwrap();
    assert_eq!(reparsed.result_type(), CssCalculationType::Flex);
    assert_eq!(reparsed.numeric_type().percent_hint(), None);
}

#[test]
fn length_percentage_math_retains_its_percentage_child_and_hint() {
    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc(10px + 5%)").unwrap(),
    )
    .unwrap();
    assert_eq!(
        calculation.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    let CssCalculationExpressionRef::NestedCalc(calc) = calculation.expression() else {
        panic!("outer calc");
    };
    let CssCalculationExpressionRef::Sum(sum) = calc.operand() else {
        panic!("calc sum");
    };
    let CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(percent)) =
        sum.term(1).unwrap().expression()
    else {
        panic!("percentage child remains a percentage");
    };
    assert_eq!(
        percent.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    assert_eq!(
        percent.numeric_type().exponent(CssNumericDimension::Length),
        1
    );
    assert_eq!(
        percent
            .numeric_type()
            .exponent(CssNumericDimension::Percentage),
        0
    );
}

#[test]
fn checked_track_sizes_reject_flexible_minima_and_preserve_exact_lengths() {
    let flex = CssGridTrackBreadth::from_flex(flex("1"));
    let short_length = CssGridTrackBreadth::from_length_percentage(length("1e-50"));
    assert_eq!(
        short_length.serialize_specified().unwrap(),
        canonical_tiny("px")
    );
    assert!(CssGridTrackSize::try_minmax(flex.clone(), short_length.clone()).is_none());
    let size = CssGridTrackSize::try_minmax(short_length, flex).unwrap();
    assert_eq!(
        size.serialize_specified().unwrap(),
        format!("minmax({}, 1fr)", canonical_tiny("px"))
    );
    assert!(CssGridFixedSize::try_new(size).is_some());
    let fit = CssGridTrackSize::from_fit_content(length("1e50"));
    assert_eq!(
        fit.serialize_specified().unwrap(),
        format!("fit-content({})", canonical_large("px"))
    );
    assert!(CssGridFixedSize::try_new(fit).is_none());
}

#[test]
fn checked_lists_reject_empty_tracks_and_repeat_cross_products() {
    let names = CssGridLineNames::new(vec![]);
    assert!(
        CssGridTrackRepeatContent::try_new(vec![CssGridTrackRepeatComponent::LineNames(
            names.clone()
        ),])
        .is_none()
    );
    assert!(
        CssGridGeneralTrackList::try_new(vec![CssGridGeneralTrackComponent::LineNames(
            names.clone()
        ),])
        .is_none()
    );
    assert!(CssGridTrackSizeList::try_new(vec![]).is_none());
    let size = CssGridTrackSize::from_breadth(CssGridTrackBreadth::from_flex(flex("2")));
    let content = CssGridTrackRepeatContent::try_new(vec![CssGridTrackRepeatComponent::TrackSize(
        size.clone(),
    )])
    .unwrap();
    let repeat = CssGridIntegerTrackRepeat::new(
        CssPositiveIntegerLiteral::try_new(surgeist_css::CssIntegerLiteral::from_i32(2)).unwrap(),
        content.clone(),
    );
    let general = CssGridTrackList::general(
        CssGridGeneralTrackList::try_new(vec![CssGridGeneralTrackComponent::Repeat(repeat)])
            .unwrap(),
    );
    assert_eq!(general.serialize_specified().unwrap(), "repeat(2, 2fr)");
    let auto = CssGridAutoRepeat::new(CssGridAutoRepeatKind::AutoFit, content);
    assert!(
        CssGridAutoTrackList::try_new(vec![
            CssGridAutoTrackComponent::AutoRepeat(auto.clone()),
            CssGridAutoTrackComponent::AutoRepeat(auto.clone()),
        ])
        .is_none()
    );
    let automatic = CssGridTrackList::auto(
        CssGridAutoTrackList::try_new(vec![CssGridAutoTrackComponent::AutoRepeat(auto)]).unwrap(),
    );
    assert_eq!(
        automatic.serialize_specified().unwrap(),
        "repeat(auto-fit, 2fr)"
    );
    assert!(CssGridFixedSize::try_new(size).is_none());
}

#[test]
fn checked_track_graph_rejects_adjacent_line_name_blocks() {
    let empty = CssGridLineNames::new(vec![]);
    let size =
        CssGridTrackSize::from_breadth(CssGridTrackBreadth::from_length_percentage(length("10")));
    assert!(
        CssGridGeneralTrackList::try_new(vec![
            CssGridGeneralTrackComponent::LineNames(empty.clone()),
            CssGridGeneralTrackComponent::LineNames(empty.clone()),
            CssGridGeneralTrackComponent::TrackSize(size.clone()),
        ])
        .is_none()
    );
    assert!(
        CssGridTrackRepeatContent::try_new(vec![
            CssGridTrackRepeatComponent::TrackSize(size.clone()),
            CssGridTrackRepeatComponent::LineNames(empty.clone()),
            CssGridTrackRepeatComponent::LineNames(empty.clone()),
        ])
        .is_none()
    );
    assert!(
        CssGridFixedRepeatContent::try_new(vec![
            CssGridFixedRepeatComponent::FixedSize(
                CssGridFixedSize::try_new(size.clone()).unwrap()
            ),
            CssGridFixedRepeatComponent::LineNames(empty.clone()),
            CssGridFixedRepeatComponent::LineNames(empty.clone()),
        ])
        .is_none()
    );
    let repeat = CssGridAutoRepeat::new(
        CssGridAutoRepeatKind::AutoFit,
        CssGridTrackRepeatContent::try_new(vec![CssGridTrackRepeatComponent::TrackSize(size)])
            .unwrap(),
    );
    assert!(
        CssGridAutoTrackList::try_new(vec![
            CssGridAutoTrackComponent::LineNames(empty.clone()),
            CssGridAutoTrackComponent::LineNames(empty),
            CssGridAutoTrackComponent::AutoRepeat(repeat),
        ])
        .is_none()
    );
}

#[test]
fn parsed_track_graph_serializes_with_one_cumulative_budget() {
    let report = parse_style_attribute("grid-auto-rows: 1e50fr fit-content(1e-50px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::GridAutoRows(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed implicit track list");
    };
    let list = wrapper.value();
    assert_eq!(
        list.serialize_specified().unwrap(),
        format!(
            "{} fit-content({})",
            canonical_large("fr"),
            canonical_tiny("px")
        )
    );
    assert_eq!(
        list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1, 100, 100
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
    );
    assert_eq!(
        list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            100, 100, 6
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit,
    );
}

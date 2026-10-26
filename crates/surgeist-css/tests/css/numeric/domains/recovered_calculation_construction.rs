//! Checked specified numeric constructors reject math recovered by implicit closure.
//!
//! Parsing retains `calc(` closed by end of input with a diagnostic, but every
//! public checked constructor admits only closed calculations, whatever its
//! root type or range.

use surgeist_css::{
    CssAbsoluteFontWeight, CssAngle, CssAspectRatioValue, CssAuthoredFontFeatureSettings,
    CssAuthoredFontFeatureValue, CssCalculation, CssComponentValueRef, CssFlex, CssFontStyle,
    CssFontWeight, CssFontWeightRange, CssFontWidth, CssInteger, CssKnownPropertyValueRef,
    CssLength, CssLengthPercentage, CssLineHeight, CssNonNegative, CssNumber, CssNumeric,
    CssNumericConstructionError, CssObliqueAngleRange, CssPercentage, CssPositive,
    CssSpecifiedRatio, CssStripeThickness, CssValueOrigin, parse_component_values,
    parse_style_attribute,
};

fn property(source: &str) -> surgeist_css::CssParseReport<surgeist_css::CssDeclarationList> {
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1, "{source}");
    // Syntax 3 §5.4.9: the implicit closure at EOF is a parse error.
    crate::css::support::value_assertions::assert_first_code(
        report.diagnostics(),
        surgeist_css::CssErrorCode::UnexpectedEnd,
    );
    report
}

macro_rules! known {
    ($report:expr, $variant:ident) => {{
        let CssKnownPropertyValueRef::$variant(value) = $report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!(stringify!($variant))
        };
        value
    }};
}

fn recovered_number() -> CssCalculation<CssNumber> {
    let report = property("line-height:calc(1 + 2");
    let CssLineHeight::Number(value) = known!(report, LineHeight).line_height() else {
        panic!("number")
    };
    value.calculation().expect("retained number math").clone()
}

fn recovered_length_percentage() -> CssCalculation<CssLengthPercentage> {
    let report = property("line-height:calc(1px + 2%");
    let CssLineHeight::LengthPercentage(value) = known!(report, LineHeight).line_height() else {
        panic!("length-percentage")
    };
    value.calculation().expect("retained length math").clone()
}

fn recovered_length() -> CssCalculation<CssLength> {
    let report = property("outline-offset:calc(1px + 2px");
    known!(report, OutlineOffset)
        .offset()
        .calculation()
        .expect("retained length math")
        .clone()
}

fn recovered_percentage() -> CssCalculation<CssPercentage> {
    let report = property("font-width:calc(50% + 10%");
    let CssFontWidth::Percentage(value) = known!(report, FontWidth).value() else {
        panic!("percentage")
    };
    value
        .calculation()
        .expect("retained percentage math")
        .clone()
}

fn recovered_flex() -> CssCalculation<CssFlex> {
    let report = property("outline-color:stripes(red calc(2fr + 3fr");
    let Some(CssStripeThickness::Flex(flex)) = known!(report, OutlineColor)
        .value()
        .image_1d()
        .unwrap()
        .stripes()
        .stripes()[0]
        .thickness()
    else {
        panic!("flex")
    };
    flex.calculation().expect("retained flex math").clone()
}

/// The rejection reports the implicit closure of the recovered root function.
fn assert_recovered(
    error: CssNumericConstructionError,
    components: &surgeist_css::CssComponentValues,
) {
    assert!(error.is_recovered_component(), "{error:?}");
    let [root] = components.items() else {
        panic!("one math root")
    };
    let CssComponentValueRef::Function(function) = root.view() else {
        panic!("math root")
    };
    assert_eq!(error.origin(), Some(function.closing_origin()));
    assert!(matches!(
        error.origin(),
        Some(CssValueOrigin::ImplicitClosure { .. })
    ));
}

#[test]
fn number_constructors_reject_recovered_math_and_accept_closed_math() {
    let recovered = recovered_number();
    assert_recovered(
        CssNumeric::<CssNumber>::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssNumeric::<CssNumber, CssNonNegative>::try_from_calculation(recovered.clone())
            .unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssCalculation::<CssNumber>::try_from_components(
            parse_component_values("calc(1 + 2)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssNumeric::<CssNumber>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(3)"
    );
    let nonnegative =
        CssNumeric::<CssNumber, CssNonNegative>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(3)"
    );
}

#[test]
fn percentage_constructors_reject_recovered_math_and_accept_closed_math() {
    let recovered = recovered_percentage();
    assert_recovered(
        CssNumeric::<CssPercentage>::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssNumeric::<CssPercentage, CssNonNegative>::try_from_calculation(recovered.clone())
            .unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssCalculation::<CssPercentage>::try_from_components(
            parse_component_values("calc(50% + 10%)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssNumeric::<CssPercentage>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(60%)"
    );
    let nonnegative =
        CssNumeric::<CssPercentage, CssNonNegative>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(60%)"
    );
}

#[test]
fn length_constructors_reject_recovered_math_and_accept_closed_math() {
    let recovered = recovered_length();
    assert_recovered(
        CssNumeric::<CssLength>::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssNumeric::<CssLength, CssNonNegative>::try_from_calculation(recovered.clone())
            .unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssCalculation::<CssLength>::try_from_components(
            parse_component_values("calc(1px + 2px)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssNumeric::<CssLength>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(3px)"
    );
    let nonnegative =
        CssNumeric::<CssLength, CssNonNegative>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(3px)"
    );
}

#[test]
fn length_percentage_constructors_reject_recovered_math_and_accept_closed_math() {
    let recovered = recovered_length_percentage();
    assert_recovered(
        CssNumeric::<CssLengthPercentage>::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(recovered.clone())
            .unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssCalculation::<CssLengthPercentage>::try_from_components(
            parse_component_values("calc(1px + 2%)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssNumeric::<CssLengthPercentage>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(2% + 1px)"
    );
    let nonnegative =
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(2% + 1px)"
    );
}

fn closed<C>(
    text: &str,
    construct: fn(
        surgeist_css::CssComponentValues,
        surgeist_css::CssComponentValueLimits,
    ) -> Result<C, CssNumericConstructionError>,
) -> C {
    construct(parse_component_values(text).unwrap(), Default::default()).unwrap()
}

// The five ranged domains of #1390 share the #1374 checked/recovering split.

#[test]
fn font_weight_number_constructor_rejects_recovered_math_and_accepts_closed_math() {
    let report = property("font-weight:calc(400 + 1");
    let CssFontWeight::Absolute(CssAbsoluteFontWeight::Number(value)) =
        known!(report, FontWeight).value()
    else {
        panic!("number weight")
    };
    let recovered = value.calculation().expect("retained number math").clone();
    assert_recovered(
        CssNumeric::<CssNumber, CssFontWeightRange>::try_from_calculation(recovered.clone())
            .unwrap_err(),
        recovered.components(),
    );
    let weight = CssNumeric::<CssNumber, CssFontWeightRange>::try_from_calculation(closed(
        "calc(400 + 1)",
        CssCalculation::<CssNumber>::try_from_components,
    ))
    .unwrap();
    assert_eq!(
        weight.serialize_specified(Default::default()).unwrap(),
        "calc(401)"
    );
}

#[test]
fn font_feature_index_constructor_rejects_recovered_math_and_accepts_closed_math() {
    let report = property("font-feature-settings:\"liga\" calc(1 + 1");
    let CssAuthoredFontFeatureSettings::Features(features) =
        known!(report, FontFeatureSettings).settings()
    else {
        panic!("feature list")
    };
    let CssAuthoredFontFeatureValue::Index(index) = features.features()[0].value() else {
        panic!("feature index")
    };
    let recovered = index.calculation().expect("retained integer math").clone();
    assert_recovered(
        CssNumeric::<CssInteger, CssNonNegative>::try_from_calculation(recovered.clone())
            .unwrap_err(),
        recovered.components(),
    );
    let index = CssNumeric::<CssInteger, CssNonNegative>::try_from_calculation(closed(
        "calc(1 + 1)",
        CssCalculation::<CssInteger>::try_from_components,
    ))
    .unwrap();
    assert_eq!(
        index.serialize_specified(Default::default()).unwrap(),
        "calc(2)"
    );
}

#[test]
fn ratio_operand_constructor_rejects_recovered_math_and_accepts_closed_math() {
    let report = property("aspect-ratio:1 / calc(1 + 1");
    let CssAspectRatioValue::Ratio(ratio) = known!(report, AspectRatio).ratio() else {
        panic!("ratio")
    };
    let recovered = ratio
        .denominator()
        .and_then(|operand| operand.calculation())
        .expect("retained number math")
        .clone();
    assert_recovered(
        CssNumeric::<CssNumber, CssNonNegative>::try_from_calculation(recovered.clone())
            .unwrap_err(),
        recovered.components(),
    );
    let operand = CssNumeric::<CssNumber, CssNonNegative>::try_from_calculation(closed(
        "calc(1 + 1)",
        CssCalculation::<CssNumber>::try_from_components,
    ))
    .unwrap();
    assert_eq!(
        CssSpecifiedRatio::new(operand, None)
            .serialize_specified(Default::default())
            .unwrap(),
        "calc(2) / 1"
    );
}

#[test]
fn page_line_minimum_constructor_rejects_recovered_math_and_accepts_closed_math() {
    let report = property("orphans:calc(1 + 1");
    let recovered = known!(report, Orphans)
        .minimum()
        .calculation()
        .expect("retained integer math")
        .clone();
    assert_recovered(
        CssNumeric::<CssInteger, CssPositive>::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    let minimum = CssNumeric::<CssInteger, CssPositive>::try_from_calculation(closed(
        "calc(1 + 1)",
        CssCalculation::<CssInteger>::try_from_components,
    ))
    .unwrap();
    assert_eq!(
        minimum.serialize_specified(Default::default()).unwrap(),
        "calc(2)"
    );
}

#[test]
fn oblique_angle_constructor_rejects_recovered_math_and_accepts_closed_math() {
    let report = property("font-style:oblique calc(10deg + 1deg");
    let CssFontStyle::Oblique { angle: Some(angle) } = known!(report, FontStyle).value() else {
        panic!("oblique angle")
    };
    let recovered = angle.calculation().expect("retained angle math").clone();
    assert_recovered(
        CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_calculation(recovered.clone())
            .unwrap_err(),
        recovered.components(),
    );
    let angle = CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_calculation(closed(
        "calc(10deg + 1deg)",
        CssCalculation::<CssAngle>::try_from_components,
    ))
    .unwrap();
    assert_eq!(
        angle.serialize_specified(Default::default()).unwrap(),
        "calc(11deg)"
    );
}

#[test]
fn flex_constructors_reject_recovered_math_and_accept_closed_math() {
    let recovered = recovered_flex();
    assert_recovered(
        CssNumeric::<CssFlex>::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssNumeric::<CssFlex, CssNonNegative>::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssCalculation::<CssFlex>::try_from_components(
            parse_component_values("calc(2fr + 3fr)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssNumeric::<CssFlex>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(5fr)"
    );
    let nonnegative =
        CssNumeric::<CssFlex, CssNonNegative>::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(5fr)"
    );
}

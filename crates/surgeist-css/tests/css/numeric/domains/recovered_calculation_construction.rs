//! Checked specified numeric constructors reject math recovered by implicit closure.
//!
//! Parsing retains `calc(` closed by end of input with a diagnostic, but every
//! public checked constructor admits only closed calculations, whatever its
//! root type or range.

use surgeist_css::{
    CssComponentValueRef, CssFlexCalculation, CssFontWidth, CssKnownPropertyValueRef,
    CssLengthCalculation, CssLengthPercentageCalculation, CssLineHeight, CssNumberCalculation,
    CssNumericConstructionError, CssPercentageCalculation, CssSpecifiedFlex, CssSpecifiedLength,
    CssSpecifiedLengthPercentage, CssSpecifiedNonNegativeFlex, CssSpecifiedNonNegativeLength,
    CssSpecifiedNonNegativeLengthPercentage, CssSpecifiedNonNegativeNumber,
    CssSpecifiedNonNegativePercentage, CssSpecifiedNumber, CssSpecifiedPercentage,
    CssStripeThickness, CssValueOrigin, parse_component_values, parse_style_attribute,
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

fn recovered_number() -> CssNumberCalculation {
    let report = property("line-height:calc(1 + 2");
    let CssLineHeight::Number(value) = known!(report, LineHeight).line_height() else {
        panic!("number")
    };
    value.calculation().expect("retained number math").clone()
}

fn recovered_length_percentage() -> CssLengthPercentageCalculation {
    let report = property("line-height:calc(1px + 2%");
    let CssLineHeight::LengthPercentage(value) = known!(report, LineHeight).line_height() else {
        panic!("length-percentage")
    };
    value.calculation().expect("retained length math").clone()
}

fn recovered_length() -> CssLengthCalculation {
    let report = property("outline-offset:calc(1px + 2px");
    known!(report, OutlineOffset)
        .offset()
        .calculation()
        .expect("retained length math")
        .clone()
}

fn recovered_percentage() -> CssPercentageCalculation {
    let report = property("font-width:calc(50% + 10%");
    let CssFontWidth::Percentage(value) = known!(report, FontWidth).value() else {
        panic!("percentage")
    };
    value
        .calculation()
        .expect("retained percentage math")
        .clone()
}

fn recovered_flex() -> CssFlexCalculation {
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
        CssSpecifiedNumber::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssSpecifiedNonNegativeNumber::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssNumberCalculation::try_from_components(
            parse_component_values("calc(1 + 2)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssSpecifiedNumber::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(3)"
    );
    let nonnegative = CssSpecifiedNonNegativeNumber::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(3)"
    );
}

#[test]
fn percentage_constructors_reject_recovered_math_and_accept_closed_math() {
    let recovered = recovered_percentage();
    assert_recovered(
        CssSpecifiedPercentage::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssSpecifiedNonNegativePercentage::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssPercentageCalculation::try_from_components(
            parse_component_values("calc(50% + 10%)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssSpecifiedPercentage::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(60%)"
    );
    let nonnegative = CssSpecifiedNonNegativePercentage::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(60%)"
    );
}

#[test]
fn length_constructors_reject_recovered_math_and_accept_closed_math() {
    let recovered = recovered_length();
    assert_recovered(
        CssSpecifiedLength::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssSpecifiedNonNegativeLength::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssLengthCalculation::try_from_components(
            parse_component_values("calc(1px + 2px)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssSpecifiedLength::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(3px)"
    );
    let nonnegative = CssSpecifiedNonNegativeLength::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(3px)"
    );
}

#[test]
fn length_percentage_constructors_reject_recovered_math_and_accept_closed_math() {
    let recovered = recovered_length_percentage();
    assert_recovered(
        CssSpecifiedLengthPercentage::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(recovered.clone())
            .unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssLengthPercentageCalculation::try_from_components(
            parse_component_values("calc(1px + 2%)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssSpecifiedLengthPercentage::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(2% + 1px)"
    );
    let nonnegative =
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(2% + 1px)"
    );
}

#[test]
fn flex_constructors_reject_recovered_math_and_accept_closed_math() {
    let recovered = recovered_flex();
    assert_recovered(
        CssSpecifiedFlex::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    assert_recovered(
        CssSpecifiedNonNegativeFlex::try_from_calculation(recovered.clone()).unwrap_err(),
        recovered.components(),
    );
    let closed = || {
        CssFlexCalculation::try_from_components(
            parse_component_values("calc(2fr + 3fr)").unwrap(),
            Default::default(),
        )
        .unwrap()
    };
    let signed = CssSpecifiedFlex::try_from_calculation(closed()).unwrap();
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(5fr)"
    );
    let nonnegative = CssSpecifiedNonNegativeFlex::try_from_calculation(closed()).unwrap();
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(5fr)"
    );
}

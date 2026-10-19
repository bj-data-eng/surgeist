#![forbid(unsafe_code)]
//! Functional typed API evidence added with authored Transforms implementation.
//! Transforms 1 §§4–6/9, Transforms 2 §§5/7–10/12, and Values 4 §10.13
//! select these operands and canonical defaults. Generic inventories remain in
//! common/property_expectations/records.rs. No contextual matrix is evaluated.
use surgeist_css::*;
type L = CssSpecifiedValueSerializationLimits;
type K = CssSpecifiedValueSerializationErrorKind;

fn number(text: &str) -> CssTransformScaleComponent {
    CssTransformScaleComponent::Number(
        CssSpecifiedNumber::try_from_component(CssComponentValue::try_number(text).unwrap())
            .unwrap(),
    )
}
fn percentage(text: &str) -> CssTransformScaleComponent {
    CssTransformScaleComponent::Percentage(
        CssSpecifiedPercentage::try_from_component(CssComponentValue::try_token(text).unwrap())
            .unwrap(),
    )
}
fn number_math(text: &str) -> CssTransformScaleComponent {
    CssTransformScaleComponent::Number(
        CssSpecifiedNumber::try_from_calculation(
            CssNumberCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap(),
        )
        .unwrap(),
    )
}
fn percentage_math(text: &str) -> CssTransformScaleComponent {
    CssTransformScaleComponent::Percentage(
        CssSpecifiedPercentage::try_from_calculation(
            CssPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap(),
        )
        .unwrap(),
    )
}
fn hinted(text: &str) -> CssTransformScaleComponent {
    CssTransformScaleComponent::HintedNumberCalculation(
        CssHintedNumberCalculation::try_from_components(parse_component_values(text).unwrap())
            .unwrap(),
    )
}
fn scale(values: Vec<CssTransformScaleComponent>) -> CssScale {
    CssScale::Values(CssScaleValues::try_new(values).unwrap())
}
fn transform(value: CssTransformFunction) -> CssTransform {
    CssTransform::Functions(CssTransformFunctionList::try_new(vec![value]).unwrap())
}
fn declaration(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}
fn budget(
    expected: &str,
    input: usize,
    projection: usize,
    emit: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        emit(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            L::new(input - 1, projection, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(emit(limits).unwrap_err().kind(), kind);
    }
    assert_eq!(
        emit(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
}
fn snapshot<T: Clone>(value: &T) -> T {
    value.clone()
}
macro_rules! check {
    ($value:expr, $input:expr, $projection:expr, $expected:expr) => {{
        let value = $value;
        let before = snapshot(&value);
        assert_eq!(value.serialize_specified().unwrap(), $expected);
        budget($expected, $input, $projection, |limits| {
            value.serialize_specified_with_limits(limits)
        });
        assert_eq!(value, before);
    }};
}

#[test]
fn new_3d_types_have_closed_domains_and_public_cumulative_providers() {
    check!(CssTransformStyle::Flat, 1, 1, "flat");
    check!(CssTransformStyle::Preserve3d, 1, 1, "preserve-3d");
    check!(CssBackfaceVisibility::Visible, 1, 1, "visible");
    check!(CssBackfaceVisibility::Hidden, 1, 1, "hidden");
    check!(CssPerspective::None, 1, 1, "none");
    check!(
        CssPerspective::Length(CssSpecifiedNonNegativeLength::zero()),
        2,
        2,
        "0"
    );
    let length = CssSpecifiedNonNegativeLength::try_from_component(
        CssComponentValue::try_token(".25px").unwrap(),
    )
    .unwrap();
    assert!(matches!(length.origin(), CssValueOrigin::Programmatic));
    check!(CssPerspective::Length(length), 2, 2, "0.25px");
    for invalid in ["-1e-999px", "1%", "1"] {
        assert!(
            CssSpecifiedNonNegativeLength::try_from_component(
                CssComponentValue::try_token(invalid).unwrap()
            )
            .is_err()
        );
    }
    let math = CssLengthCalculation::try_from_components(
        parse_component_values("calc(1px - 2px)").unwrap(),
    )
    .unwrap();
    let original = math.components().clone();
    let perspective =
        CssPerspective::Length(CssSpecifiedNonNegativeLength::try_from_calculation(math).unwrap());
    check!(&perspective, 5, 5, "calc(-1px)");
    let CssPerspective::Length(value) = &perspective else {
        panic!("length");
    };
    assert_eq!(value.calculation().unwrap().components(), &original);
    let parsed = declaration("perspective:calc(1px - 2px)");
    let CssKnownPropertyValueRef::Perspective(wrapper) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("perspective wrapper");
    };
    assert_eq!(wrapper.value(), &perspective);
    let parsed = declaration("transform-style:PRESERVE-3D");
    let CssKnownPropertyValueRef::TransformStyle(wrapper) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("style wrapper");
    };
    assert_eq!(wrapper.value(), &CssTransformStyle::Preserve3d);
    let parsed = declaration("backface-visibility:HIDDEN");
    let CssKnownPropertyValueRef::BackfaceVisibility(wrapper) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("visibility wrapper");
    };
    assert_eq!(wrapper.value(), &CssBackfaceVisibility::Hidden);
}

#[test]
fn perspective_origin_uses_the_existing_physical_position_owner_and_origin_fields() {
    let h = CssSpecifiedLengthPercentage::try_from_component(
        CssComponentValue::try_token("2px").unwrap(),
    )
    .unwrap();
    let v = CssSpecifiedLengthPercentage::try_from_component(
        CssComponentValue::try_token("3%").unwrap(),
    )
    .unwrap();
    let position = CssPhysicalPosition::try_new(
        CssHorizontalPosition::RightOffset(h),
        CssVerticalPosition::BottomOffset(v),
    )
    .unwrap();
    check!(&position, 5, 5, "right 2px bottom 3%");
    let text = "/*😀*/\r\nperspective-origin:right 2px bottom 3%!important";
    let source = declaration(text);
    let CssKnownPropertyValueRef::PerspectiveOrigin(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("position wrapper");
    };
    assert_eq!(wrapper.as_css(), "right 2px bottom 3%");
    assert_eq!(wrapper.position(), &position);
    let CssHorizontalPosition::RightOffset(h) = wrapper.position().horizontal() else {
        panic!("right offset");
    };
    let CssValueOrigin::Parsed(origin) = h.origin() else {
        panic!("parsed length");
    };
    assert_eq!(origin.source().as_str(), text);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        text.find("2px").unwrap()
    );
    budget(
        "perspective-origin: right 2px bottom 3% !important;",
        7,
        7,
        |limits| source.to_specified_css_with_limits(limits),
    );
}

#[test]
fn expanded_scale_constructors_retain_percentage_and_symbolic_operands() {
    let values =
        CssScaleValues::try_new(vec![percentage("50%"), number(".5"), percentage("100%")]).unwrap();
    assert!(
        matches!(&values.values()[0], CssTransformScaleComponent::Percentage(v)
        if matches!(v.origin(), CssValueOrigin::Programmatic))
    );
    assert!(CssScaleValues::try_new(Vec::new()).is_none());
    assert!(CssScaleValues::try_new(vec![number("1"); 4]).is_none());
    check!(CssScale::Values(values), 4, 4, "0.5");
    let pair = CssTransformScale::new(percentage("50%"), Some(number(".5")));
    assert!(matches!(
        pair.x(),
        CssTransformScaleComponent::Percentage(_)
    ));
    assert!(matches!(
        pair.y(),
        Some(CssTransformScaleComponent::Number(_))
    ));
    check!(
        transform(CssTransformFunction::Scale(pair)),
        4,
        4,
        "scale(0.5)"
    );
    check!(
        transform(CssTransformFunction::ScaleX(percentage("25%"))),
        3,
        3,
        "scaleX(0.25)"
    );
    check!(
        transform(CssTransformFunction::ScaleY(percentage("125%"))),
        3,
        3,
        "scaleY(1.25)"
    );
    check!(scale(vec![number_math("calc(1 + 2)")]), 5, 4, "calc(3)");
    // The selected specified percentage projection adds a divisor, inverse and
    // product; the Scale aggregate borrows that same numeric owner's budget.
    check!(
        scale(vec![percentage_math("calc(50% + 50%)")]),
        5,
        7,
        "calc(1)"
    );
    check!(
        transform(CssTransformFunction::ScaleX(percentage_math(
            "calc(50% + 50%)"
        ))),
        6,
        8,
        "scaleX(calc(1))"
    );
}

#[test]
fn scale_defaults_compare_exact_cross_unit_values_before_six_place_formatting() {
    for (x, y, z, output) in [
        (percentage("50%"), number(".5"), percentage("100%"), "0.5"),
        (number(".5"), percentage("50%"), number("1e0"), "0.5"),
        (
            percentage("50%"),
            number(".50000004"),
            percentage("100%"),
            "0.5 0.5",
        ),
        (
            percentage("50%"),
            number(".5"),
            percentage("100.000004%"),
            "0.5 0.5 1",
        ),
        (percentage("1e-999%"), number("1e-1001"), number("1"), "0"),
        (percentage("1e-999%"), number("0"), number("1"), "0 0"),
        (number("-0"), percentage("0%"), percentage("100%"), "0"),
    ] {
        check!(scale(vec![x, y, z]), 4, 4, output);
    }
    check!(
        transform(CssTransformFunction::Scale(CssTransformScale::new(
            percentage("50%"),
            Some(number(".50000004"))
        ))),
        4,
        4,
        "scale(0.5, 0.5)"
    );
}

#[test]
fn structural_symbolic_equality_ignores_origins_but_preserves_distinct_authored_calculations() {
    let parsed = number_math("calc(1 + 2)");
    let programmatic = CssTransformScaleComponent::Number(
        CssSpecifiedNumber::try_from_calculation(
            CssNumberCalculation::try_from_components(
                CssComponentValues::try_new(vec![
                    CssComponentValue::try_function(
                        "calc",
                        parse_component_values("1 + 2").unwrap(),
                    )
                    .unwrap(),
                ])
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    assert_eq!(parsed, programmatic);
    // Both equal symbolic factors and the suppressed exact ordinary Z are visited.
    check!(
        scale(vec![parsed, programmatic, number("1")]),
        10,
        8,
        "calc(3)"
    );
    check!(
        scale(vec![number_math("calc(1 + 2)"), number_math("calc(2 + 1)")]),
        9,
        7,
        "calc(3) calc(3)"
    );
    // A calculated one is retained; deciding its default would need a separate
    // symbolic equivalence policy. It never becomes an ordinary literal field.
    check!(
        scale(vec![number("2"), number("2"), number_math("calc(1)")]),
        5,
        4,
        "2 2 calc(1)"
    );
}

#[test]
fn hinted_number_factors_keep_the_unresolved_basis_and_original_source_after_failures() {
    let text = "/*😀*/\r\nscale:calc((1px + 1%) / 1px)";
    let source = declaration(text);
    let before = source.clone();
    let CssKnownPropertyValueRef::Scale(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("scale");
    };
    let CssScale::Values(values) = wrapper.value() else {
        panic!("factors");
    };
    let CssTransformScaleComponent::HintedNumberCalculation(value) = &values.values()[0] else {
        panic!("hinted number factor");
    };
    assert_eq!(
        value.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    assert_eq!(
        value.components().serialize().unwrap().as_css(),
        "calc((1px + 1%) / 1px)"
    );
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed math root");
    };
    assert_eq!(origin.source().as_str(), text);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        text.find("calc").unwrap()
    );
    check!(wrapper.value(), 8, 9, "calc((1% + 1px) / 1px)");
    budget("scale: calc((1% + 1px) / 1px);", 10, 11, |limits| {
        source.to_specified_css_with_limits(limits)
    });
    assert_eq!(source, before);
    let same = hinted("calc((1px + 1%) / 1px)");
    check!(
        scale(vec![same.clone(), same, number("1")]),
        16,
        18,
        "calc((1% + 1px) / 1px)"
    );
    check!(
        transform(CssTransformFunction::ScaleY(hinted(
            "calc((1px + 1%) / 1px)"
        ))),
        9,
        10,
        "scaleY(calc((1% + 1px) / 1px))"
    );
}

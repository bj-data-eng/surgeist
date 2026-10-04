#![forbid(unsafe_code)]
//! Same-unit abs/hypot coefficient identities require a nonnegative unit basis.
//! Values 4 defines line-height and viewport-relative lengths by sizes, and
//! Containment 3 defines container-relative lengths by container dimensions:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#font-relative-lengths
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#viewport-relative-lengths
//! https://www.w3.org/TR/2022/WD-css-contain-3-20220818/#container-lengths
//! CSS2 disallows negative line-height values:
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/visudet.html#propdef-line-height
//! The projection policy conservatively retains glyph-metric and flex bases;
//! these tests characterize that boundary rather than assuming metric signs.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn length(source: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_calculation(
        CssLengthCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap(),
    )
    .unwrap()
}

fn length_percentage(source: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_calculation(
        CssLengthPercentageCalculation::try_from_components(
            parse_component_values(source).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn assert_length(source: &str, expected: &str) {
    let value = length(source);
    let before = value.clone();
    let calculation = value.calculation().unwrap();
    let components = calculation.components().clone();
    let ty = calculation.numeric_type();
    assert_eq!(components.serialize().unwrap().as_css(), source);
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(value.calculation().unwrap().numeric_type(), ty);
    assert_eq!(result.unwrap(), expected, "{source}");
}

#[test]
fn line_height_units_preserve_same_unit_magnitude_identities() {
    for unit in ["lh", "rlh"] {
        for (expression, expected) in [
            (format!("abs(-2{unit})"), format!("calc(2{unit})")),
            (format!("abs(0{unit})"), format!("calc(0{unit})")),
            (format!("hypot(3{unit},4{unit})"), format!("calc(5{unit})")),
        ] {
            assert_length(&expression, &expected);
        }
    }
}

#[test]
fn viewport_families_preserve_same_unit_magnitude_identities() {
    // Default, small, large and dynamic viewports; physical and logical axes,
    // plus minimum/maximum dimensions, each denote a nonnegative size.
    for unit in ["vw", "vh", "vi", "vb", "vmin", "vmax", "svh", "lvi", "dvb"] {
        assert_length(&format!("abs(-2{unit})"), &format!("calc(2{unit})"));
        assert_length(
            &format!("hypot(-3{unit},4{unit})"),
            &format!("calc(5{unit})"),
        );
    }
}

#[test]
fn container_dimensions_preserve_same_unit_magnitude_identities() {
    for unit in ["cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax"] {
        assert_length(&format!("abs(-2{unit})"), &format!("calc(2{unit})"));
        assert_length(
            &format!("hypot(3{unit},4{unit})"),
            &format!("calc(5{unit})"),
        );
    }
}

#[test]
fn glyph_metric_units_keep_magnitude_functions_symbolic() {
    for unit in ["ex", "rex", "cap", "rcap", "ch", "rch", "ic", "ric"] {
        let abs = format!("abs(-2{unit})");
        assert_length(&abs, &abs);
        assert_length(
            &format!("hypot(3{unit},4{unit})"),
            &format!("hypot(3{unit}, 4{unit})"),
        );
    }
}

#[test]
fn flex_fraction_basis_keeps_magnitude_functions_symbolic() {
    for (source, expected) in [
        ("abs(-2fr)", "abs(-2fr)"),
        ("hypot(3fr,4fr)", "hypot(3fr, 4fr)"),
    ] {
        let value = CssSpecifiedNonNegativeFlex::try_from_calculation(
            CssFlexCalculation::try_from_components(parse_component_values(source).unwrap())
                .unwrap(),
        )
        .unwrap();
        let before = value.clone();
        let result = value.serialize_specified();
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn different_size_bases_and_hinted_percentages_remain_symbolic() {
    for (source, expected) in [
        ("abs(1lh - 1px)", "abs(1lh - 1px)"),
        ("hypot(3lh,4rlh)", "hypot(3lh, 4rlh)"),
        ("hypot(3cqw,4cqh)", "hypot(3cqw, 4cqh)"),
    ] {
        assert_length(source, expected);
    }
    for (source, expected) in [("abs(-2%)", "abs(-2%)"), ("hypot(3%,4%)", "hypot(3%, 4%)")] {
        let value = length_percentage(source);
        let before = value.clone();
        assert_eq!(
            value.calculation().unwrap().numeric_type().percent_hint(),
            Some(CssNumericDimension::Length)
        );
        let result = value.serialize_specified();
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn newly_folded_siblings_share_one_cumulative_enclosing_budget() {
    let shape = CssInsetShape::new(
        CssInsetShapeOffsets::try_new(vec![
            length_percentage("abs(-2lh)"),
            length_percentage("hypot(3cqw,4cqw)"),
        ])
        .unwrap(),
        None,
    );
    let before = shape.clone();
    let expected = "inset(calc(2lh) calc(5cqw))";
    // Shape and offset-list aggregates, two abs visits and three hypot visits.
    // Each folded function replacement spends a projection node after leaves.
    for (limits, kind) in [
        (Limits::new(6, 7, expected.len()), Kind::InputNodeLimit),
        (Limits::new(7, 6, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(7, 7, expected.len() - 1), Kind::ByteLimit),
    ] {
        let error = shape.serialize_specified_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), kind, "{limits:?}");
        assert_eq!(shape, before);
        for (value, original) in shape
            .offsets()
            .values()
            .iter()
            .zip(before.offsets().values())
        {
            assert_eq!(value.origin(), original.origin());
            assert_eq!(
                value.calculation().unwrap().components(),
                original.calculation().unwrap().components()
            );
        }
    }
    let result = shape.serialize_specified_with_limits(Limits::new(7, 7, expected.len()));
    assert_eq!(shape, before);
    assert_eq!(result.unwrap(), expected);
}

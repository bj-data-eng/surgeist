#![forbid(unsafe_code)]
//! The adopted Surgeist aggregate contract compares exact numeric syntax while
//! ignoring numeric provenance only. This is an authored-value product contract,
//! not a CSSOM semantic-equivalence claim or an evaluation of symbolic math.
//! Parsing, public aggregate getters, and PartialEq already express the behavior.

use surgeist_css::*;

macro_rules! aggregate_parser {
    ($name:ident, $ty:ty, $variant:ident, $getter:ident) => {
        fn $name(source: &str) -> $ty {
            let report = parse_style_attribute(source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().len(), 1, "{source}");
            let CssKnownPropertyValueRef::$variant(value) = report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap()
            else {
                panic!("typed property: {source}")
            };
            value.$getter().clone()
        }
    };
}
aggregate_parser!(slice, CssBorderImageSlice, BorderImageSlice, slice);
aggregate_parser!(width, CssBorderImageWidth, BorderImageWidth, widths);
aggregate_parser!(outset, CssBorderImageOutset, BorderImageOutset, outsets);
aggregate_parser!(border, CssBorderImage, BorderImage, border_image);
aggregate_parser!(
    iterations,
    CssAnimationIterationCountList,
    AnimationIterationCount,
    iteration_counts
);
aggregate_parser!(animation, CssAnimationList, Animation, animations);

fn filter(source: &str) -> CssFilter {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    match report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    {
        CssKnownPropertyValueRef::Filter(value) => value.value().clone(),
        CssKnownPropertyValueRef::BackdropFilter(value) => value.value().clone(),
        _ => panic!("typed filter property"),
    }
}

#[test]
fn filter_and_backdrop_equality_keep_exact_number_and_percentage_spelling() {
    for property in ["filter", "backdrop-filter"] {
        for name in [
            "brightness",
            "contrast",
            "grayscale",
            "invert",
            "opacity",
            "saturate",
            "sepia",
        ] {
            for (left, right) in [("1", "1.0"), ("25%", "25.0%")] {
                assert!(
                    filter(&format!("{property}: {name}({left})"))
                        != filter(&format!("{property}: {name}({right})")),
                    "Numeric identity must retain distinct authored structures or provenance; inputs: filter(&format!(\"{{property}}: {{name}}({{left}})\")) versus filter(&format!(\"{{property}}: {{name}}({{right}})\")); case: {property} {name} {left}/{right}"
                );
            }
        }
    }
}
#[test]
fn filter_and_backdrop_equality_ignore_number_and_percentage_math_origins() {
    for property in ["filter", "backdrop-filter"] {
        for name in [
            "brightness",
            "contrast",
            "grayscale",
            "invert",
            "opacity",
            "saturate",
            "sepia",
        ] {
            for value in ["calc(1 + 2)", "calc(25% + 5%)"] {
                assert!(
                    filter(&format!("{property}: {name}({value})"))
                        == filter(&format!("  {property}: {name}({value})")),
                    "Numeric identity must compare equal when only numeric origin differs; inputs: filter(&format!(\"{{property}}: {{name}}({{value}})\")) versus filter(&format!(\"  {{property}}: {{name}}({{value}})\")); case: {property} {name} {value}"
                );
            }
        }
    }
}
#[test]
fn border_slice_equality_keeps_exact_number_and_percentage_spelling() {
    assert!(
        slice("border-image-slice: 1 fill") != slice("border-image-slice: 1.0 fill"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: slice(\"border-image-slice: 1 fill\") versus slice(\"border-image-slice: 1.0 fill\")"
    );
    assert!(
        slice("border-image-slice: 25% fill") != slice("border-image-slice: 25.0% fill"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: slice(\"border-image-slice: 25% fill\") versus slice(\"border-image-slice: 25.0% fill\")"
    );
}
#[test]
fn border_width_equality_keeps_exact_number_spelling() {
    assert!(
        width("border-image-width: 1 auto") != width("border-image-width: 1.0 auto"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: width(\"border-image-width: 1 auto\") versus width(\"border-image-width: 1.0 auto\")"
    );
}
#[test]
fn border_outset_equality_keeps_exact_number_spelling() {
    assert!(
        outset("border-image-outset: 1 2px") != outset("border-image-outset: 1.0 2px"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: outset(\"border-image-outset: 1 2px\") versus outset(\"border-image-outset: 1.0 2px\")"
    );
}
#[test]
fn border_shorthand_equality_keeps_exact_numeric_spellings() {
    assert!(
        border("border-image: 1 / 2 / 3") != border("border-image: 1.0 / 2 / 3"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: border(\"border-image: 1 / 2 / 3\") versus border(\"border-image: 1.0 / 2 / 3\")"
    );
}
#[test]
fn border_slice_equality_ignores_number_and_percentage_math_origins() {
    for value in ["calc(1 + 2)", "calc(25% + 5%)"] {
        assert!(
            slice(&format!("border-image-slice: {value} fill"))
                == slice(&format!("  border-image-slice: {value} fill")),
            "Numeric identity must compare equal when only numeric origin differs; inputs: slice(&format!(\"border-image-slice: {{value}} fill\")) versus slice(&format!(\"  border-image-slice: {{value}} fill\"))"
        );
    }
}
#[test]
fn border_width_equality_ignores_number_math_origins() {
    assert!(
        width("border-image-width: calc(1 + 2) auto")
            == width("  border-image-width: calc(1 + 2) auto"),
        "Numeric identity must compare equal when only numeric origin differs; inputs: width(\"border-image-width: calc(1 + 2) auto\") versus width(\"  border-image-width: calc(1 + 2) auto\")"
    );
}
#[test]
fn border_outset_equality_ignores_number_math_origins() {
    assert!(
        outset("border-image-outset: calc(1 + 2) 2px")
            == outset("  border-image-outset: calc(1 + 2) 2px"),
        "Numeric identity must compare equal when only numeric origin differs; inputs: outset(\"border-image-outset: calc(1 + 2) 2px\") versus outset(\"  border-image-outset: calc(1 + 2) 2px\")"
    );
}
#[test]
fn border_shorthand_equality_ignores_number_math_origins() {
    assert!(
        border("border-image: calc(1 + 2) / 2 / 3")
            == border("  border-image: calc(1 + 2) / 2 / 3"),
        "Numeric identity must compare equal when only numeric origin differs; inputs: border(\"border-image: calc(1 + 2) / 2 / 3\") versus border(\"  border-image: calc(1 + 2) / 2 / 3\")"
    );
}
#[test]
fn iteration_equality_keeps_exact_ordinary_number_spelling() {
    assert!(
        iterations("animation-iteration-count: .5, infinite")
            != iterations("animation-iteration-count: 0.5, infinite"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: iterations(\"animation-iteration-count: .5, infinite\") versus iterations(\"animation-iteration-count: 0.5, infinite\")"
    );
}
#[test]
fn iteration_equality_ignores_number_math_origins() {
    assert!(
        iterations("animation-iteration-count: calc(1 + 2), infinite")
            == iterations("  animation-iteration-count: calc(1 + 2), infinite"),
        "Numeric identity must compare equal when only numeric origin differs; inputs: iterations(\"animation-iteration-count: calc(1 + 2), infinite\") versus iterations(\"  animation-iteration-count: calc(1 + 2), infinite\")"
    );
}
#[test]
fn animation_shorthand_equality_keeps_exact_iteration_number_spelling() {
    assert!(
        animation("animation: fade 1s .5") != animation("animation: fade 1s 0.5"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: animation(\"animation: fade 1s .5\") versus animation(\"animation: fade 1s 0.5\")"
    );
}
#[test]
fn animation_shorthand_equality_ignores_iteration_number_math_origins() {
    assert!(
        animation("animation: fade 1s calc(1 + 2)")
            == animation("  animation: fade 1s calc(1 + 2)"),
        "Numeric identity must compare equal when only numeric origin differs; inputs: animation(\"animation: fade 1s calc(1 + 2)\") versus animation(\"  animation: fade 1s calc(1 + 2)\")"
    );
}
#[test]
fn nonnegative_numeric_aggregate_equality_keeps_zero_sign_and_exponent_spelling() {
    assert!(
        filter("filter: contrast(-0)") != filter("filter: contrast(0)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: filter(\"filter: contrast(-0)\") versus filter(\"filter: contrast(0)\")"
    );
    assert!(
        slice("border-image-slice: -0%") != slice("border-image-slice: 0%"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: slice(\"border-image-slice: -0%\") versus slice(\"border-image-slice: 0%\")"
    );
    assert!(
        iterations("animation-iteration-count: 1e0") != iterations("animation-iteration-count: 1"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: iterations(\"animation-iteration-count: 1e0\") versus iterations(\"animation-iteration-count: 1\")"
    );
}

#[test]
fn nonnegative_equality_controls_keep_defaults_branches_fill_order_and_omission() {
    assert!(
        filter("filter: contrast(1)") == filter("  filter: contrast(1)"),
        "Numeric identity must compare equal when only numeric origin differs; inputs: filter(\"filter: contrast(1)\") versus filter(\"  filter: contrast(1)\")"
    );
    assert!(
        filter("filter: contrast()") != filter("filter: contrast(1)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: filter(\"filter: contrast()\") versus filter(\"filter: contrast(1)\")"
    );
    assert!(
        filter("filter: contrast(1)") != filter("filter: contrast(100%)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: filter(\"filter: contrast(1)\") versus filter(\"filter: contrast(100%)\")"
    );
    assert!(
        filter("filter: contrast(1) brightness(2)") != filter("filter: brightness(2) contrast(1)"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: filter(\"filter: contrast(1) brightness(2)\") versus filter(\"filter: brightness(2) contrast(1)\")"
    );
    assert!(
        slice("border-image-slice: 1") == slice("border-image-slice: 1 1 1 1"),
        "Numeric identity must compare equal when only numeric origin differs; inputs: slice(\"border-image-slice: 1\") versus slice(\"border-image-slice: 1 1 1 1\")"
    );
    assert!(
        slice("border-image-slice: 1") != slice("border-image-slice: 1 fill"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: slice(\"border-image-slice: 1\") versus slice(\"border-image-slice: 1 fill\")"
    );
    assert!(
        slice("border-image-slice: 1") != slice("border-image-slice: 1%"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: slice(\"border-image-slice: 1\") versus slice(\"border-image-slice: 1%\")"
    );
    assert!(
        slice("border-image-slice: 1 2") != slice("border-image-slice: 2 1"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: slice(\"border-image-slice: 1 2\") versus slice(\"border-image-slice: 2 1\")"
    );
    assert!(
        width("border-image-width: auto") != width("border-image-width: 1"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: width(\"border-image-width: auto\") versus width(\"border-image-width: 1\")"
    );
    assert!(
        outset("border-image-outset: 2") != outset("border-image-outset: 2px"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: outset(\"border-image-outset: 2\") versus outset(\"border-image-outset: 2px\")"
    );
    assert!(
        border("border-image: 1") != border("border-image: 1 / 1 / 0"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: border(\"border-image: 1\") versus border(\"border-image: 1 / 1 / 0\")"
    );
    assert!(
        iterations("animation-iteration-count: 1, infinite")
            != iterations("animation-iteration-count: infinite, 1"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: iterations(\"animation-iteration-count: 1, infinite\") versus iterations(\"animation-iteration-count: infinite, 1\")"
    );
    assert!(
        animation("animation: fade 1s") != animation("animation: fade 1s 1"),
        "Numeric identity must retain distinct authored structures or provenance; inputs: animation(\"animation: fade 1s\") versus animation(\"animation: fade 1s 1\")"
    );
}
#[test]
fn raw_nonnegative_scalar_and_calculation_equality_keep_provenance_policy() {
    let n = CssSpecifiedNonNegativeNumber::try_from_component(
        parse_component_values("1").unwrap().items()[0].clone(),
    )
    .unwrap();
    let built = CssSpecifiedNonNegativeNumber::try_from_component(
        CssComponentValue::try_number("1").unwrap(),
    )
    .unwrap();
    assert!(
        n != built,
        "Direct shared nonnegative Number equality retains parsed versus programmatic provenance for 1"
    );
    let p = CssSpecifiedNonNegativePercentage::try_from_component(
        parse_component_values("25%").unwrap().items()[0].clone(),
    )
    .unwrap();
    let built = CssSpecifiedNonNegativePercentage::try_from_component(
        CssComponentValue::try_token("25%").unwrap(),
    )
    .unwrap();
    assert!(
        p == built,
        "Direct shared nonnegative Percentage equality ignores parsed versus programmatic provenance for 25%"
    );
    let a =
        CssNumberCalculation::try_from_components(parse_component_values("calc(1 + 2)").unwrap())
            .unwrap();
    let b =
        CssNumberCalculation::try_from_components(parse_component_values("  calc(1 + 2)").unwrap())
            .unwrap();
    assert!(
        a != b,
        "Raw Number calculation equality retains source origins for calc(1 + 2) versus its leading-whitespace source"
    );
}

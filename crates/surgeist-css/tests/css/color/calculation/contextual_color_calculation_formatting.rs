#![forbid(unsafe_code)]
//! Values 4 WD 2024-03-12 retained calculations and CSSOM WD 2021-08-26
//! component numbers: generic six-place text for context-dependent ordinary components.
//! Halfway away from zero is the adopted operational policy on binary64 bits.

use surgeist_css::*;
type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;
fn components(source: &str) -> CssComponentValues {
    parse_component_values(source).unwrap()
}
fn declaration(source: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        components(source),
        CssImportance::Important,
    )
    .unwrap_or_else(|e| panic!("{source}: {e:?}"))
}
fn declaration_color(d: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(c) = d.known().unwrap().property_value().unwrap() else {
        panic!("color")
    };
    c.value()
}
fn color(source: &str) -> CssColor {
    declaration_color(&declaration(source)).clone()
}
fn assert_text(source: &str, expected: &str) {
    let d = declaration(source);
    let before = d.clone();
    assert_eq!(d.value_components(), &components(source));
    let c = declaration_color(&d);
    let first = c.to_specified_css();
    assert_eq!(first, c.to_specified_css());
    assert_eq!(d, before);
    assert_eq!(first.unwrap(), expected, "{source}");
}
macro_rules! text_case {
    ($name:ident,$source:literal,$expected:literal) => {
        #[test]
        fn $name() {
            assert_text($source, $expected);
        }
    };
}

// Exact binary64 1/128 has scaled midpoint 7812.5. Contextual unit ratios
// remain symbolic; this contract changes coefficient text, not their arithmetic.
text_case!(
    rgb_contextual_number,
    "rgb(calc(.0078125 * 1em / 1px) 0 0)",
    "rgb(calc(0.007813 * 1em / 1px) 0 0)"
);
text_case!(
    hsl_contextual_number,
    "hsl(calc(.0078125 * 1em / 1px) 50% 50%)",
    "hsl(calc(0.007813 * 1em / 1px) 50% 50%)"
);
text_case!(
    hwb_contextual_number,
    "hwb(calc(.0078125 * 1em / 1px) 20% 30%)",
    "hwb(calc(0.007813 * 1em / 1px) 20% 30%)"
);
text_case!(
    lab_contextual_number,
    "lab(calc(.0078125 * 1em / 1px) 0 0)",
    "lab(calc(0.007813 * 1em / 1px) 0 0)"
);
text_case!(
    lch_contextual_number,
    "lch(calc(.0078125 * 1em / 1px) 0 0)",
    "lch(calc(0.007813 * 1em / 1px) 0 0)"
);
text_case!(
    oklab_contextual_number,
    "oklab(calc(.0078125 * 1em / 1px) 0 0)",
    "oklab(calc(0.007813 * 1em / 1px) 0 0)"
);
text_case!(
    oklch_contextual_number,
    "oklch(calc(.0078125 * 1em / 1px) 0 0)",
    "oklch(calc(0.007813 * 1em / 1px) 0 0)"
);
text_case!(
    predefined_contextual_number,
    "color(srgb calc(.0078125 * 1em / 1px) 0 0)",
    "color(srgb calc(0.007813 * 1em / 1px) 0 0)"
);
text_case!(
    custom_contextual_number,
    "color(--P calc(.0078125 * 1em / 1px) 0 0)",
    "color(--P calc(0.007813 * 1em / 1px) 0 0)"
);
text_case!(
    hsl_number_hue,
    "hsl(calc(1em / 1px + 1 / 128) 50% 50%)",
    "hsl(calc(0.007813 + (1em / 1px)) 50% 50%)"
);
text_case!(
    hsl_angle_hue,
    "hsl(calc(1em / 1px * 1rad) 50% 50%)",
    "hsl(calc(57.29578deg * 1em / 1px) 50% 50%)"
);
text_case!(
    hsl_angle_tiny_turn_hue,
    "hsl(calc(1em / 1px * .0000001turn) 50% 50%)",
    "hsl(calc(0.000036deg * 1em / 1px) 50% 50%)"
);
text_case!(
    hwb_number_hue,
    "hwb(calc(1em / 1px + 1 / 128) 20% 30%)",
    "hwb(calc(0.007813 + (1em / 1px)) 20% 30%)"
);
text_case!(
    hwb_angle_hue,
    "hwb(calc(1em / 1px * 1rad) 20% 30%)",
    "hwb(calc(57.29578deg * 1em / 1px) 20% 30%)"
);
text_case!(
    hwb_angle_tiny_turn_hue,
    "hwb(calc(1em / 1px * .0000001turn) 20% 30%)",
    "hwb(calc(0.000036deg * 1em / 1px) 20% 30%)"
);
text_case!(
    lch_number_hue,
    "lch(20 30 calc(1em / 1px + 1 / 128))",
    "lch(20 30 calc(0.007813 + (1em / 1px)))"
);
text_case!(
    lch_angle_hue,
    "lch(20 30 calc(1em / 1px * 1rad))",
    "lch(20 30 calc(57.29578deg * 1em / 1px))"
);
text_case!(
    lch_angle_tiny_turn_hue,
    "lch(20 30 calc(1em / 1px * .0000001turn))",
    "lch(20 30 calc(0.000036deg * 1em / 1px))"
);
text_case!(
    oklch_number_hue,
    "oklch(.2 .3 calc(1em / 1px + 1 / 128))",
    "oklch(0.2 0.3 calc(0.007813 + (1em / 1px)))"
);
text_case!(
    oklch_angle_hue,
    "oklch(.2 .3 calc(1em / 1px * 1rad))",
    "oklch(0.2 0.3 calc(57.29578deg * 1em / 1px))"
);
text_case!(
    oklch_angle_tiny_turn_hue,
    "oklch(.2 .3 calc(1em / 1px * .0000001turn))",
    "oklch(0.2 0.3 calc(0.000036deg * 1em / 1px))"
);
text_case!(
    positive_midpoint,
    "hsl(calc(1 / 128 + 1em / 1px) 50% 50%)",
    "hsl(calc(0.007813 + (1em / 1px)) 50% 50%)"
);
text_case!(
    negative_midpoint,
    "hsl(calc(-1 / 128 + 1em / 1px) 50% 50%)",
    "hsl(calc(-0.007813 + (1em / 1px)) 50% 50%)"
);
text_case!(
    below_midpoint,
    "hsl(calc(.007812499999999999 + 1em / 1px) 50% 50%)",
    "hsl(calc(0.007812 + (1em / 1px)) 50% 50%)"
);
text_case!(
    above_midpoint,
    "hsl(calc(.007812500000000002 + 1em / 1px) 50% 50%)",
    "hsl(calc(0.007813 + (1em / 1px)) 50% 50%)"
);
text_case!(
    binary_micro_below,
    "hsl(calc(5e-7 + 1em / 1px) 50% 50%)",
    "hsl(calc(0 + (1em / 1px)) 50% 50%)"
);
text_case!(
    carry,
    "hsl(calc(.9999996 + 1em / 1px) 50% 50%)",
    "hsl(calc(1 + (1em / 1px)) 50% 50%)"
);
text_case!(
    tiny,
    "hsl(calc(5e-324 + 1em / 1px) 50% 50%)",
    "hsl(calc(0 + (1em / 1px)) 50% 50%)"
);
text_case!(
    negative_tiny,
    "hsl(calc(-5e-324 + 1em / 1px) 50% 50%)",
    "hsl(calc(0 + (1em / 1px)) 50% 50%)"
);
text_case!(
    integer_digits,
    "hsl(calc(7812500000000001 * 128 + 1em / 1px) 50% 50%)",
    "hsl(calc(1000000000000000128 + (1em / 1px)) 50% 50%)"
);
text_case!(
    rgb_percentage_slot_0,
    "rgb(calc(.0078125% * 1em / 1px) 0 0)",
    "rgb(calc(0.007813% * 1em / 1px / 0.392157%) 0 0)"
);
text_case!(
    rgb_percentage_slot_1,
    "rgb(0 calc(.0078125% * 1em / 1px) 0)",
    "rgb(0 calc(0.007813% * 1em / 1px / 0.392157%) 0)"
);
text_case!(
    rgb_percentage_slot_2,
    "rgb(0 0 calc(.0078125% * 1em / 1px))",
    "rgb(0 0 calc(0.007813% * 1em / 1px / 0.392157%))"
);
text_case!(
    lab_percentage_slot_0,
    "lab(calc(.0078125% * 1em / 1px) 0 0)",
    "lab(calc(0.007813% * 1em / 1px / 1%) 0 0)"
);
text_case!(
    lab_percentage_slot_1,
    "lab(0 calc(.0078125% * 1em / 1px) 0)",
    "lab(0 calc(0.007813% * 1em / 1px / 0.8%) 0)"
);
text_case!(
    lab_percentage_slot_2,
    "lab(0 0 calc(.0078125% * 1em / 1px))",
    "lab(0 0 calc(0.007813% * 1em / 1px / 0.8%))"
);
text_case!(
    lch_percentage_slot_0,
    "lch(calc(.0078125% * 1em / 1px) 0 0)",
    "lch(calc(0.007813% * 1em / 1px / 1%) 0 0)"
);
text_case!(
    lch_percentage_slot_1,
    "lch(0 calc(.0078125% * 1em / 1px) 0)",
    "lch(0 calc(0.007813% * 1em / 1px / 0.666667%) 0)"
);
text_case!(
    oklab_percentage_slot_0,
    "oklab(calc(.0078125% * 1em / 1px) 0 0)",
    "oklab(calc(0.007813% * 1em / 1px / 100%) 0 0)"
);
text_case!(
    oklab_percentage_slot_1,
    "oklab(0 calc(.0078125% * 1em / 1px) 0)",
    "oklab(0 calc(0.007813% * 1em / 1px / 250%) 0)"
);
text_case!(
    oklab_percentage_slot_2,
    "oklab(0 0 calc(.0078125% * 1em / 1px))",
    "oklab(0 0 calc(0.007813% * 1em / 1px / 250%))"
);
text_case!(
    oklch_percentage_slot_0,
    "oklch(calc(.0078125% * 1em / 1px) 0 0)",
    "oklch(calc(0.007813% * 1em / 1px / 100%) 0 0)"
);
text_case!(
    oklch_percentage_slot_1,
    "oklch(0 calc(.0078125% * 1em / 1px) 0)",
    "oklch(0 calc(0.007813% * 1em / 1px / 250%) 0)"
);
text_case!(
    predefined_percentage_slot_0,
    "color(srgb calc(.0078125% * 1em / 1px) 0 0)",
    "color(srgb calc(0.007813% * 1em / 1px / 100%) 0 0)"
);
text_case!(
    predefined_percentage_slot_1,
    "color(srgb 0 calc(.0078125% * 1em / 1px) 0)",
    "color(srgb 0 calc(0.007813% * 1em / 1px / 100%) 0)"
);
text_case!(
    predefined_percentage_slot_2,
    "color(srgb 0 0 calc(.0078125% * 1em / 1px))",
    "color(srgb 0 0 calc(0.007813% * 1em / 1px / 100%))"
);
text_case!(
    custom_percentage_slot_0,
    "color(--P calc(.0078125% * 1em / 1px) 0 0)",
    "color(--P calc(0.007813% * 1em / 1px / 100%) 0 0)"
);
text_case!(
    custom_percentage_slot_1,
    "color(--P 0 calc(.0078125% * 1em / 1px) 0)",
    "color(--P 0 calc(0.007813% * 1em / 1px / 100%) 0)"
);
text_case!(
    custom_percentage_slot_2,
    "color(--P 0 0 calc(.0078125% * 1em / 1px))",
    "color(--P 0 0 calc(0.007813% * 1em / 1px / 100%))"
);
text_case!(
    hsl_percentage_identity_1,
    "hsl(0 calc(.0078125% * 1em / 1px) 30%)",
    "hsl(0 calc(0.007813% * 1em / 1px) 30%)"
);
text_case!(
    hsl_percentage_identity_2,
    "hsl(0 20% calc(.0078125% * 1em / 1px))",
    "hsl(0 20% calc(0.007813% * 1em / 1px))"
);
text_case!(
    hwb_percentage_identity_1,
    "hwb(0 calc(.0078125% * 1em / 1px) 30%)",
    "hwb(0 calc(0.007813% * 1em / 1px) 30%)"
);
text_case!(
    hwb_percentage_identity_2,
    "hwb(0 20% calc(.0078125% * 1em / 1px))",
    "hwb(0 20% calc(0.007813% * 1em / 1px))"
);
text_case!(
    hsl_number_to_percentage,
    "hsl(none calc(.0078125 * 1em / 1px) 50%)",
    "hsl(none calc(0.007813 * 1% * 1em / 1px) 50%)"
);
text_case!(
    hwb_number_to_percentage,
    "hwb(none calc(.0078125 * 1em / 1px) 50%)",
    "hwb(none calc(0.007813 * 1% * 1em / 1px) 50%)"
);
text_case!(
    same_contextual_unit_ratio,
    "hsl(calc(1 / 128 + 1em / 1em) 50% 50%)",
    "hsl(calc(0.007813 + (1em / 1em)) 50% 50%)"
);
text_case!(
    absolute_ratio_numeric_control,
    "color(--P calc(1 / 128 + 1in / 96px))",
    "color(--P calc(1.0078125))"
);
text_case!(
    infectious_nan_numeric_control,
    "color(--P calc(NaN + 1em / 1px))",
    "color(--P calc(NaN))"
);
text_case!(
    nonfinite_contextual_control,
    "hsl(calc(infinity + 1em / 1px) 50% 50%)",
    "hsl(calc(infinity + (1em / 1px)) 50% 50%)"
);
text_case!(
    negative_zero_operand_control,
    "hsl(calc(0 * -1 + 1em / 1px) 50% 50%)",
    "hsl(calc((0 * -1) + (1em / 1px)) 50% 50%)"
);
text_case!(
    angle_nonfinite_wrapper_control,
    "hsl(calc(infinity * 1deg + 1em / 1px * 1deg) 50% 50%)",
    "hsl(calc((infinity * 1deg) + (1deg * 1em / 1px)) 50% 50%)"
);
text_case!(
    nested_mix_contextual,
    "color-mix(color(--P calc(.0078125 * 1em / 1px)), blue)",
    "color-mix(color(--P calc(0.007813 * 1em / 1px)), blue)"
);
text_case!(
    nested_mix_mix_contextual,
    "color-mix(color-mix(color(--P calc(.0078125 * 1em / 1px)), blue), red)",
    "color-mix(color-mix(color(--P calc(0.007813 * 1em / 1px)), blue), red)"
);
text_case!(
    numeric_retained_control,
    "color(srgb calc(1 / 128) 0 0)",
    "color(srgb calc(0.0078125) 0 0)"
);
text_case!(
    numeric_missing_control,
    "hsl(none calc(.0078125) 50%)",
    "hsl(none 0.007813% 50%)"
);
text_case!(rgb_resolved_control, "rgb(calc(300) 0 0)", "rgb(255, 0, 0)");
text_case!(
    hsl_resolved_control,
    "hsl(calc(0) 100% 50%)",
    "rgb(255, 0, 0)"
);
text_case!(hwb_resolved_control, "hwb(calc(0) 0% 0%)", "rgb(255, 0, 0)");
text_case!(
    standalone_alpha_control,
    "color(--P 0 / calc(1 / 128))",
    "color(--P 0 / calc(0.007813))"
);
text_case!(
    contextual_alpha_control,
    "color(--P 0 / calc(.0078125 * 1em / 1px))",
    "color(--P 0 / calc(0.007813 * 1em / 1px))"
);
text_case!(
    weight_control,
    "color-mix(in srgb, red calc(.78125%), blue)",
    "color-mix(in srgb, red calc(0.78125%), blue)"
);
text_case!(
    origin_control,
    "alpha(from color(--P calc(.0078125 * 1em / 1px)))",
    "alpha(from color(--P calc(0.007813 * 1em / 1px)))"
);
text_case!(
    relative_control,
    "rgb(from red calc(r + 1 / 128) g b)",
    "rgb(from red calc(0.007813 + r) g b)"
);
text_case!(
    literal_precision_control,
    "color(--P .1234567)",
    "color(--P 0.1234567)"
);
text_case!(literal_modulo_control, "lch(20 30 720)", "lch(20 30 0)");

// calc(.0078125 * 1em / 1px): wrapper/product/three leaves = five inputs.
// Leaves (3), inverse (1), product (1) = five projections. A single Number
// reuses its existing scalar instead of creating a merged replacement.
// Non-origin Number(1/1) adds scale leaf, combined Number and Product (3).
// With the Color root (1), one scaled contextual component costs nine.
// A percentage product has no number-combination node (five projections),
// then percentage-to-number adds percentage leaf/inverse/product (3).
fn assert_limits(source: &str, expected: &str, inputs: usize, projections: usize) {
    let c = color(source);
    let before = c.clone();
    for (limits, kind) in [
        (
            Limits::new(inputs - 1, usize::MAX, usize::MAX),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(usize::MAX, projections - 1, usize::MAX),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(inputs, projections, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_eq!(
            c.to_specified_css_with_limits(limits).unwrap_err().kind(),
            kind
        );
        assert_eq!(c, before);
    }
    let limits = Limits::new(inputs, projections, expected.len());
    let result = c.to_specified_css_with_limits(limits);
    assert_eq!(result, c.to_specified_css_with_limits(limits));
    assert_eq!(c, before);
    assert_eq!(result.unwrap(), expected);
}
#[test]
fn number_exact_counts_and_final_bytes() {
    assert_limits(
        "color(--P calc(.0078125 * 1em / 1px))",
        "color(--P calc(0.007813 * 1em / 1px))",
        6,
        9,
    );
}
#[test]
fn percent_scale_suffix_counts_and_final_bytes() {
    assert_limits(
        "color(--P calc(.0078125% * 1em / 1px))",
        "color(--P calc(0.007813% * 1em / 1px / 100%))",
        6,
        9,
    );
}
#[test]
fn cumulative_sibling_counts_and_bytes() {
    // One Color root + two captures of (first pass 5 + scale 3) = seventeen.
    assert_limits(
        "color(--P calc(.0078125 * 1em / 1px) calc(.0078125 * 1em / 1px))",
        "color(--P calc(0.007813 * 1em / 1px) calc(0.007813 * 1em / 1px))",
        11,
        17,
    );
}
#[test]
fn mix_remaining_bytes_and_cumulative_counts() {
    // An explicit calc(50%) selects the retained weight path, avoiding generated
    // exact-rational weights. Three color roots + five component inputs + two
    // weight inputs = ten. Roots (3) + component (5 + 3) + weight leaf (1) = twelve.
    assert_limits(
        "color-mix(color(--P calc(.0078125 * 1em / 1px)) calc(50%), blue)",
        "color-mix(color(--P calc(0.007813 * 1em / 1px)) calc(50%), blue)",
        10,
        12,
    );
}

#[test]
fn contextual_scratch_reaches_sibling_input_limit() {
    let c = color("color(--P calc(5e-324 * 1em / 1px) none)");
    let before = c.clone();
    // Rounded calc(0 * 1em / 1px) needs 19 bytes. A 24-byte scratch bound
    // allows it, then the seventh input (none) exceeds the six-slot budget.
    assert_eq!(
        c.to_specified_css_with_limits(Limits::new(6, usize::MAX, 24))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(c, before);
}
#[test]
fn contextual_scratch_reaches_sibling_projection_limit() {
    let c = color("color(--P calc(5e-324 * 1em / 1px) none)");
    // Color root + first pass 5 + scale 3 spend nine before the none sibling.
    assert_eq!(
        c.to_specified_css_with_limits(Limits::new(usize::MAX, 9, 24))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
}
#[test]
fn numeric_discarded_scratch_control() {
    let c = color("rgb(calc(10000000000) 0 0)");
    let before = c.clone();
    assert_eq!(
        c.to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, 17))
            .unwrap(),
        "rgb(255, 0, 0)"
    );
    assert_eq!(
        c.to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, 16))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(c, before);
}
macro_rules! error_case {
    ($name:ident,$limits:expr,$kind:expr) => {
        #[test]
        fn $name() {
            let c = color("color(--P calc(.0078125 * 1em / 1px))");
            let before = c.clone();
            assert_eq!(
                c.to_specified_css_with_limits($limits).unwrap_err().kind(),
                $kind
            );
            assert_eq!(c, before);
        }
    };
}
error_case!(
    zero_input_control,
    Limits::new(0, usize::MAX, usize::MAX),
    Kind::InputNodeLimit
);
error_case!(
    zero_projection_control,
    Limits::new(usize::MAX, 0, usize::MAX),
    Kind::ProjectionNodeLimit
);
error_case!(
    zero_bytes_control,
    Limits::new(usize::MAX, usize::MAX, 0),
    Kind::ByteLimit
);
error_case!(
    input_precedes_other_zero_limits_control,
    Limits::new(0, 0, 0),
    Kind::InputNodeLimit
);
error_case!(
    projection_precedes_zero_bytes_control,
    Limits::new(1, 0, 0),
    Kind::ProjectionNodeLimit
);
error_case!(
    calculation_input_precedes_zero_scratch_control,
    Limits::new(1, usize::MAX, 0),
    Kind::InputNodeLimit
);
#[test]
fn checked_graph_preserves_parsed_components_and_provenance() {
    let raw = components("calc(.0078125 * 1em / 1px)");
    let calc = CssNumberCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(calc.components(), &raw);
    assert_eq!(calc.result_type(), CssCalculationType::Number);
    assert_eq!(calc.expression().origin(), raw.items()[0].origin());
    let CssValueOrigin::Parsed(p) = calc.expression().origin() else {
        panic!("parsed")
    };
    assert_eq!(p.source().as_str(), "calc(.0078125 * 1em / 1px)");
    let source = CssColor::from_rgb(
        CssRgbColor::try_new(
            CssColorSyntax::Modern,
            [
                CssColorComponent::NumberCalculation(calc.clone()),
                number("0"),
                number("0"),
            ],
            None,
        )
        .unwrap(),
    );
    assert_eq!(
        source.rgb_value().unwrap().channels()[0],
        CssColorComponent::NumberCalculation(calc)
    );
    let before = source.clone();
    let result = source.to_specified_css();
    assert_eq!(result, source.to_specified_css());
    assert_eq!(source, before);
    assert_eq!(raw, components("calc(.0078125 * 1em / 1px)"));
    assert_eq!(result.unwrap(), "rgb(calc(0.007813 * 1em / 1px) 0 0)");
}
fn number(text: &str) -> CssColorComponent {
    CssColorComponent::Number(
        CssColorNumberLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
            .unwrap(),
    )
}
#[test]
fn clean_validation_retains_graph_and_siblings() {
    let source = "color: hsl(calc(1 / 128 + 1em / 1px) 50% 50%) !important; opacity: .5";
    let r = parse_style_attribute(source);
    assert!(r.is_clean(), "{:?}", r.diagnostics());
    assert_eq!(&validate_style_attribute(source).unwrap(), r.syntax());
    assert_eq!(r.syntax().len(), 2);
    let before = r.syntax().clone();
    let d = &r.syntax()[0];
    assert_eq!(d.importance(), CssImportance::Important);
    assert_eq!(
        d.value_components().serialize().unwrap().as_css(),
        " hsl(calc(1 / 128 + 1em / 1px) 50% 50%) "
    );
    let result = declaration_color(d).to_specified_css();
    assert_eq!(r.syntax(), &before);
    assert_eq!(result.unwrap(), "hsl(calc(0.007813 + (1em / 1px)) 50% 50%)");
}
#[test]
fn invalid_contextual_dimension_recovery_control() {
    let r = parse_style_attribute("color: rgb(calc(1em) 0 0); opacity: .5");
    assert_eq!(r.diagnostics().len(), 1);
    assert_eq!(r.syntax().len(), 1);
    assert_eq!(
        r.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
}
#[test]
fn substitution_stays_pending_control() {
    let src = "rgb(calc(var(--n) + 1em / 1px) 0 0)";
    let d = declaration(src);
    let known = d.known().unwrap();
    assert!(known.property_value().is_none());
    assert_eq!(known.substitution_dependent().unwrap().as_css(), src);
    assert!(matches!(
        expand_declaration(&d).unwrap(),
        CssExpansion::Pending(_)
    ));
    let e = CssNumberCalculation::try_from_components(components("calc(var(--n) + 1em / 1px)"))
        .unwrap_err();
    assert_eq!(
        e.kind(),
        &CssNumericConstructionErrorKind::SubstitutionRequired
    );
    assert!(e.origin().is_some());
}

text_case!(
    pinned_existing_contextual_hue_stimulus,
    "hsl(calc(1em / 1px + 0.0078125deg / 1deg) 50% 50%)",
    "hsl(calc(0.007813 + (1em / 1px)) 50% 50%)"
);
text_case!(
    percentage_nonfinite_operand_wrapper_control,
    "hsl(0 calc(infinity * 1% * 1em / 1px) 50%)",
    "hsl(0 calc(infinity * 1% * 1em / 1px) 50%)"
);
#[test]
fn checked_percentage_graph_retains_scale_inputs_and_provenance() {
    let raw = components("calc(.0078125% * 1em / 1px)");
    let calc = CssPercentageCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(calc.result_type(), CssCalculationType::Percentage);
    assert_eq!(calc.components(), &raw);
    assert_eq!(calc.expression().origin(), raw.items()[0].origin());
    let value = CssColor::from_rgb(
        CssRgbColor::try_new(
            CssColorSyntax::Modern,
            [
                CssColorComponent::PercentageCalculation(calc.clone()),
                number("0"),
                number("0"),
            ],
            None,
        )
        .unwrap(),
    );
    assert_eq!(
        value.rgb_value().unwrap().channels()[0],
        CssColorComponent::PercentageCalculation(calc)
    );
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(result, value.to_specified_css());
    assert_eq!(value, before);
    assert_eq!(
        raw.serialize().unwrap().as_css(),
        "calc(.0078125% * 1em / 1px)"
    );
    assert_eq!(
        result.unwrap(),
        "rgb(calc(0.007813% * 1em / 1px / 0.392157%) 0 0)"
    );
}
#[test]
fn mixed_programmatic_and_parsed_graph_preserves_each_actual_origin() {
    let args = components(".0078125 * 1em / 1px");
    let leaf = CssComponentValue::try_number(".0078125").unwrap();
    let mut items = args.items().to_vec();
    items[0] = leaf.clone();
    let mixed = CssComponentValues::try_new(items).unwrap();
    let function = CssComponentValue::try_function("calc", mixed.clone()).unwrap();
    let raw = CssComponentValues::try_new(vec![function]).unwrap();
    let calc = CssNumberCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(calc.expression().origin(), &CssValueOrigin::Programmatic);
    assert_eq!(calc.components(), &raw);
    assert_eq!(mixed.items()[0].origin(), &CssValueOrigin::Programmatic);
    assert_eq!(mixed.items()[4].origin(), args.items()[4].origin());
    assert!(matches!(
        mixed.items()[4].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let value = CssColor::from_rgb(
        CssRgbColor::try_new(
            CssColorSyntax::Modern,
            [
                CssColorComponent::NumberCalculation(calc),
                number("0"),
                number("0"),
            ],
            None,
        )
        .unwrap(),
    );
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(result, value.to_specified_css());
    assert_eq!(value, before);
    assert_eq!(
        raw.serialize().unwrap().as_css(),
        "calc(.0078125 * 1em / 1px)"
    );
    assert_eq!(mixed.items()[0], leaf);
    assert_eq!(result.unwrap(), "rgb(calc(0.007813 * 1em / 1px) 0 0)");
}
#[test]
fn discarded_numeric_capture_error_precedence_control() {
    let value = color("rgb(calc(10000000000) 0 0)");
    let before = value.clone();
    // Root + wrapper/leaf = three inputs before the first capture. Root plus
    // leaf/scale/resolved product = four projections before its 17-byte scratch.
    for (limits, kind) in [
        (Limits::new(2, usize::MAX, 16), Kind::InputNodeLimit),
        (Limits::new(3, 3, 16), Kind::ProjectionNodeLimit),
        (Limits::new(3, 4, 16), Kind::ByteLimit),
    ] {
        assert_eq!(
            value
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
    }
}
text_case!(
    rgb_ordinary_mix_component,
    "color-mix(rgb(calc(.0078125 * 1em / 1px) 0 0), blue)",
    "color-mix(rgb(calc(0.007813 * 1em / 1px) 0 0), blue)"
);
text_case!(
    hsl_ordinary_mix_component,
    "color-mix(hsl(calc(.0078125 * 1em / 1px) 50% 50%), blue)",
    "color-mix(hsl(calc(0.007813 * 1em / 1px) 50% 50%), blue)"
);
text_case!(
    hwb_ordinary_mix_component,
    "color-mix(hwb(calc(.0078125 * 1em / 1px) 20% 30%), blue)",
    "color-mix(hwb(calc(0.007813 * 1em / 1px) 20% 30%), blue)"
);
text_case!(
    lab_ordinary_mix_component,
    "color-mix(lab(calc(.0078125 * 1em / 1px) 0 0), blue)",
    "color-mix(lab(calc(0.007813 * 1em / 1px) 0 0), blue)"
);
text_case!(
    lch_ordinary_mix_component,
    "color-mix(lch(calc(.0078125 * 1em / 1px) 0 0), blue)",
    "color-mix(lch(calc(0.007813 * 1em / 1px) 0 0), blue)"
);
text_case!(
    oklab_ordinary_mix_component,
    "color-mix(oklab(calc(.0078125 * 1em / 1px) 0 0), blue)",
    "color-mix(oklab(calc(0.007813 * 1em / 1px) 0 0), blue)"
);
text_case!(
    oklch_ordinary_mix_component,
    "color-mix(oklch(calc(.0078125 * 1em / 1px) 0 0), blue)",
    "color-mix(oklch(calc(0.007813 * 1em / 1px) 0 0), blue)"
);
text_case!(
    predefined_ordinary_mix_component,
    "color-mix(color(srgb calc(.0078125 * 1em / 1px) 0 0), blue)",
    "color-mix(color(srgb calc(0.007813 * 1em / 1px) 0 0), blue)"
);
text_case!(
    custom_ordinary_mix_component,
    "color-mix(color(--P calc(.0078125 * 1em / 1px) 0 0), blue)",
    "color-mix(color(--P calc(0.007813 * 1em / 1px) 0 0), blue)"
);

#[test]
fn contextual_angle_suffix_and_wrapper_final_bytes() {
    // Root + wrapper/product/three leaves + two none slots = eight inputs.
    // Three leaves/inverse/product plus root/two none = eight projections.
    assert_limits(
        "hsl(calc(.0078125deg * 1em / 1px) none none)",
        "hsl(calc(0.007813deg * 1em / 1px) none none)",
        8,
        8,
    );
}
#[test]
fn contextual_percentage_identity_final_bytes() {
    assert_limits(
        "hsl(none calc(.0078125% * 1em / 1px) none)",
        "hsl(none calc(0.007813% * 1em / 1px) none)",
        8,
        8,
    );
}
text_case!(
    numeric_sibling_capture_control_with_selected_context,
    "color(--P calc(1 / 128) calc(.0078125 * 1em / 1px))",
    "color(--P calc(0.0078125) calc(0.007813 * 1em / 1px))"
);

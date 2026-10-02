#![forbid(unsafe_code)]
//! Color 5 WD 2026-09-08 origin serialization and CSSOM WD 2021-08-26
//! component numbers: generic six-place text for calculated ordinary origins.
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

// 1/128 * 10^6 is exactly 7812.5; the finite halfway oracle is independent
// of the serializer. Percentages preserve Identity scale in an ordinary origin.
text_case!(
    rgb_number,
    "alpha(from rgb(calc(1 / 128) none none))",
    "alpha(from rgb(calc(0.007813) none none))"
);
text_case!(
    rgb_percentage_identity,
    "alpha(from rgb(none calc(1% / 128) none))",
    "alpha(from rgb(none calc(0.007813%) none))"
);
text_case!(
    hsl_number,
    "alpha(from hsl(calc(1 / 128) none none))",
    "alpha(from hsl(calc(0.007813) none none))"
);
text_case!(
    hsl_percentage_identity,
    "alpha(from hsl(none calc(1% / 128) none))",
    "alpha(from hsl(none calc(0.007813%) none))"
);
text_case!(
    hwb_number,
    "alpha(from hwb(calc(1 / 128) none none))",
    "alpha(from hwb(calc(0.007813) none none))"
);
text_case!(
    hwb_percentage_identity,
    "alpha(from hwb(none calc(1% / 128) none))",
    "alpha(from hwb(none calc(0.007813%) none))"
);
text_case!(
    lab_number,
    "alpha(from lab(calc(1 / 128) none none))",
    "alpha(from lab(calc(0.007813) none none))"
);
text_case!(
    lab_percentage_identity,
    "alpha(from lab(none calc(1% / 128) none))",
    "alpha(from lab(none calc(0.007813%) none))"
);
text_case!(
    lch_number,
    "alpha(from lch(calc(1 / 128) none none))",
    "alpha(from lch(calc(0.007813) none none))"
);
text_case!(
    lch_percentage_identity,
    "alpha(from lch(none calc(1% / 128) none))",
    "alpha(from lch(none calc(0.007813%) none))"
);
text_case!(
    oklab_number,
    "alpha(from oklab(calc(1 / 128) none none))",
    "alpha(from oklab(calc(0.007813) none none))"
);
text_case!(
    oklab_percentage_identity,
    "alpha(from oklab(none calc(1% / 128) none))",
    "alpha(from oklab(none calc(0.007813%) none))"
);
text_case!(
    oklch_number,
    "alpha(from oklch(calc(1 / 128) none none))",
    "alpha(from oklch(calc(0.007813) none none))"
);
text_case!(
    oklch_percentage_identity,
    "alpha(from oklch(none calc(1% / 128) none))",
    "alpha(from oklch(none calc(0.007813%) none))"
);
text_case!(
    predefined_number,
    "alpha(from color(srgb calc(1 / 128) none none))",
    "alpha(from color(srgb calc(0.007813) none none))"
);
text_case!(
    predefined_percentage_identity,
    "alpha(from color(srgb none calc(1% / 128) none))",
    "alpha(from color(srgb none calc(0.007813%) none))"
);
text_case!(
    custom_number,
    "alpha(from color(--P calc(1 / 128) none none))",
    "alpha(from color(--P calc(0.007813) none none))"
);
text_case!(
    custom_percentage_identity,
    "alpha(from color(--P none calc(1% / 128) none))",
    "alpha(from color(--P none calc(0.007813%) none))"
);
text_case!(
    negative_halfway,
    "alpha(from rgb(calc(-1 / 128) none none))",
    "alpha(from rgb(calc(-0.007813) none none))"
);
text_case!(
    below_halfway,
    "alpha(from rgb(calc(0.007812499999999999) none none))",
    "alpha(from rgb(calc(0.007812) none none))"
);
text_case!(
    above_halfway,
    "alpha(from rgb(calc(0.007812500000000002) none none))",
    "alpha(from rgb(calc(0.007813) none none))"
);
text_case!(
    binary_micro_below,
    "alpha(from rgb(calc(5e-7) none none))",
    "alpha(from rgb(calc(0) none none))"
);
text_case!(
    tiny,
    "alpha(from rgb(calc(5e-324) none none))",
    "alpha(from rgb(calc(0) none none))"
);
text_case!(
    negative_tiny,
    "alpha(from rgb(calc(-5e-324) none none))",
    "alpha(from rgb(calc(0) none none))"
);
text_case!(
    carry,
    "alpha(from rgb(calc(0.9999996) none none))",
    "alpha(from rgb(calc(1) none none))"
);
text_case!(
    integer_digits,
    "alpha(from rgb(calc(7812500000000001 * 128) none none))",
    "alpha(from rgb(calc(1000000000000000128) none none))"
);
text_case!(
    hsl_number_hue,
    "alpha(from hsl(calc(1 / 128) none none))",
    "alpha(from hsl(calc(0.007813) none none))"
);
text_case!(
    hsl_angle_hue,
    "alpha(from hsl(calc(1rad) none none))",
    "alpha(from hsl(calc(57.29578deg) none none))"
);
text_case!(
    hsl_tiny_turn,
    "alpha(from hsl(calc(0.0000001turn) none none))",
    "alpha(from hsl(calc(0.000036deg) none none))"
);
text_case!(
    hwb_number_hue,
    "alpha(from hwb(calc(1 / 128) none none))",
    "alpha(from hwb(calc(0.007813) none none))"
);
text_case!(
    hwb_angle_hue,
    "alpha(from hwb(calc(1rad) none none))",
    "alpha(from hwb(calc(57.29578deg) none none))"
);
text_case!(
    hwb_tiny_turn,
    "alpha(from hwb(calc(0.0000001turn) none none))",
    "alpha(from hwb(calc(0.000036deg) none none))"
);
text_case!(
    lch_number_hue,
    "alpha(from lch(none none calc(1 / 128)))",
    "alpha(from lch(none none calc(0.007813)))"
);
text_case!(
    lch_angle_hue,
    "alpha(from lch(none none calc(1rad)))",
    "alpha(from lch(none none calc(57.29578deg)))"
);
text_case!(
    lch_tiny_turn,
    "alpha(from lch(none none calc(0.0000001turn)))",
    "alpha(from lch(none none calc(0.000036deg)))"
);
text_case!(
    oklch_number_hue,
    "alpha(from oklch(none none calc(1 / 128)))",
    "alpha(from oklch(none none calc(0.007813)))"
);
text_case!(
    oklch_angle_hue,
    "alpha(from oklch(none none calc(1rad)))",
    "alpha(from oklch(none none calc(57.29578deg)))"
);
text_case!(
    oklch_tiny_turn,
    "alpha(from oklch(none none calc(0.0000001turn)))",
    "alpha(from oklch(none none calc(0.000036deg)))"
);
text_case!(
    alpha_number,
    "alpha(from rgb(none none none / calc(1 / 128)))",
    "alpha(from rgb(none none none / calc(0.007813)))"
);
text_case!(
    alpha_percentage,
    "alpha(from rgb(none none none / calc(1% / 128)))",
    "alpha(from rgb(none none none / calc(0.007813%)))"
);
text_case!(
    alpha_identity,
    "alpha(from rgb(none none none / calc(.78125%)))",
    "alpha(from rgb(none none none / calc(0.78125%)))"
);
text_case!(
    alpha_above_one,
    "alpha(from rgb(none none none / calc(2.1234567)))",
    "alpha(from rgb(none none none / calc(2.123457)))"
);
text_case!(
    alpha_above_hundred,
    "alpha(from rgb(none none none / calc(120.1234567%)))",
    "alpha(from rgb(none none none / calc(120.123457%)))"
);
text_case!(
    alpha_negative,
    "alpha(from rgb(none none none / calc(-1.1234567)))",
    "alpha(from rgb(none none none / calc(-1.123457)))"
);
text_case!(
    alpha_explicit_one,
    "alpha(from rgb(none none none / calc(1)))",
    "alpha(from rgb(none none none / calc(1)))"
);
text_case!(
    relative_origin,
    "rgb(from rgb(calc(1 / 128) none none) r g b)",
    "rgb(from rgb(calc(0.007813) none none) r g b)"
);
text_case!(
    custom_relative_origin,
    "color(from color(--P calc(1 / 128)) --Q Cyan)",
    "color(from color(--P calc(0.007813)) --Q Cyan)"
);
text_case!(
    nested_alpha_origin,
    "alpha(from alpha(from rgb(calc(1 / 128) none none)))",
    "alpha(from alpha(from rgb(calc(0.007813) none none)))"
);
text_case!(
    mix_nested_origin,
    "color-mix(alpha(from rgb(calc(1 / 128) none none)), blue)",
    "color-mix(alpha(from rgb(calc(0.007813) none none)), blue)"
);
text_case!(
    origin_mix_relative,
    "alpha(from color-mix(rgb(from color(srgb calc(1 / 128) none none) r g b), blue))",
    "alpha(from color-mix(rgb(from color(srgb calc(0.007813) none none) r g b), blue))"
);
text_case!(
    legacy_origin,
    "alpha(from rgb(calc(1 / 128), 0, 0))",
    "alpha(from rgb(calc(0.007813) 0 0))"
);
text_case!(
    symbolic_origin,
    "alpha(from rgb(calc(1em / 1px + 1 / 128) none none))",
    "alpha(from rgb(calc(0.007813 + (1em / 1px)) none none))"
);
text_case!(
    nonfinite_nan_control,
    "alpha(from rgb(calc(0 / 0) none none))",
    "alpha(from rgb(calc(NaN) none none))"
);
text_case!(
    nonfinite_infinity_control,
    "alpha(from rgb(calc(infinity) none none))",
    "alpha(from rgb(calc(infinity) none none))"
);
text_case!(
    negative_zero_inverse_control,
    "alpha(from rgb(calc(1 / (0 * -1)) none none))",
    "alpha(from rgb(calc(-infinity) none none))"
);
text_case!(
    negative_zero_root_control,
    "alpha(from rgb(calc(0 * -1) none none))",
    "alpha(from rgb(calc(0) none none))"
);
text_case!(
    negative_zero_symbolic_control,
    "alpha(from rgb(calc(0 * -1 + 1em / 1px) none none))",
    "alpha(from rgb(calc((0 * -1) + (1em / 1px)) none none))"
);
text_case!(
    standalone_retained_control,
    "color(srgb calc(1 / 128) 0 0)",
    "color(srgb calc(0.0078125) 0 0)"
);
text_case!(
    standalone_conversion_control,
    "rgb(calc(300) 0 0)",
    "rgb(255, 0, 0)"
);
text_case!(
    relative_calculation_control,
    "rgb(from red calc(1 / 128) g b)",
    "rgb(from red calc(0.007813) g b)"
);
text_case!(
    literal_control,
    "alpha(from rgb(.1234567 0 0))",
    "alpha(from rgb(0.123457 0 0))"
);
text_case!(
    mix_weight_control,
    "color-mix(in srgb, red calc(.78125%), blue)",
    "color-mix(in srgb, red calc(0.78125%), blue)"
);

// Two color roots cost 2/2. None costs 1/1. A divide calculation visits
// wrapper/product/two leaves (4) and projects leaves/inverse/product (4).
// A single-leaf calculation visits wrapper/leaf (2) and projects one leaf.
// Identity percentages add no scale nodes. Counts are cumulative across siblings.
fn assert_limits(source: &str, text: &str, inputs: usize, projections: usize) {
    let value = color(source);
    let before = value.clone();
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
            Limits::new(inputs, projections, text.len() - 1),
            Kind::ByteLimit,
        ),
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
    let limits = Limits::new(inputs, projections, text.len());
    let first = value.to_specified_css_with_limits(limits);
    assert_eq!(first, value.to_specified_css_with_limits(limits));
    assert_eq!(value, before);
    assert_eq!(first.unwrap(), text);
}
#[test]
fn number_final_bytes_and_counts() {
    assert_limits(
        "alpha(from rgb(calc(1 / 128) none none))",
        "alpha(from rgb(calc(0.007813) none none))",
        8,
        8,
    );
}
#[test]
fn percentage_suffix_final_bytes_and_counts() {
    assert_limits(
        "alpha(from rgb(calc(1% / 128) none none))",
        "alpha(from rgb(calc(0.007813%) none none))",
        8,
        8,
    );
}
#[test]
fn angle_suffix_final_bytes_and_counts() {
    assert_limits(
        "alpha(from hsl(calc(1rad) none none))",
        "alpha(from hsl(calc(57.29578deg) none none))",
        6,
        5,
    );
}
#[test]
fn sibling_work_and_final_bytes_are_cumulative() {
    assert_limits(
        "alpha(from color(--P calc(1 / 128) calc(1 / 128)))",
        "alpha(from color(--P calc(0.007813) calc(0.007813)))",
        10,
        10,
    );
}
#[test]
fn alpha_percentage_identity_counts() {
    assert_limits(
        "alpha(from rgb(none none none / calc(.78125%)))",
        "alpha(from rgb(none none none / calc(0.78125%)))",
        7,
        6,
    );
}
macro_rules! error_case {
    ($name:ident,$limits:expr,$kind:expr) => {
        #[test]
        fn $name() {
            let c = color("alpha(from rgb(calc(1 / 128) none none))");
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
    root_input_precedes_projection_and_bytes_control,
    Limits::new(0, 0, 0),
    Kind::InputNodeLimit
);
error_case!(
    root_projection_precedes_bytes_control,
    Limits::new(1, 0, 0),
    Kind::ProjectionNodeLimit
);
error_case!(
    outer_bytes_precede_origin_input_control,
    Limits::new(1, 1, 0),
    Kind::ByteLimit
);

#[test]
fn rounded_origin_scratch_reaches_sibling_input_error() {
    let c = color("rgb(from color(srgb calc(1 / 128) none none) r g b)");
    let before = c.clone();
    // Four outer root/reference visits; origin root/divide bring totals to 9/9.
    // After 'rgb(from ', 14 bytes remain. Rounded calc needs exactly 14;
    // the old 15-byte capture fails before visiting either none sibling.
    assert_eq!(
        c.to_specified_css_with_limits(Limits::new(9, usize::MAX, 23))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(c, before);
}
#[test]
fn rounded_origin_scratch_reaches_sibling_projection_error() {
    let c = color("rgb(from color(srgb calc(1 / 128) none none) r g b)");
    assert_eq!(
        c.to_specified_css_with_limits(Limits::new(usize::MAX, 9, 23))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
}
#[test]
fn standalone_discarded_capture_control() {
    let c = color("rgb(calc(10000000000) 0 0)");
    let before = c.clone();
    assert_eq!(
        c.to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, 16))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(
        c.to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, 17))
            .unwrap(),
        "rgb(255, 0, 0)"
    );
    assert_eq!(c, before);
}
#[test]
fn checked_graph_retains_calculation_components_and_parsed_provenance() {
    let raw = components("calc(1 / 128)");
    let calc = CssNumberCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(calc.components(), &raw);
    assert_eq!(calc.result_type(), CssCalculationType::Number);
    assert_eq!(calc.expression().origin(), raw.items()[0].origin());
    let CssValueOrigin::Parsed(origin) = calc.expression().origin() else {
        panic!("parsed provenance")
    };
    assert_eq!(origin.source().as_str(), "calc(1 / 128)");
    let rgb = CssRgbColor::try_new(
        CssColorSyntax::Modern,
        [
            CssColorComponent::NumberCalculation(calc.clone()),
            CssColorComponent::None,
            CssColorComponent::None,
        ],
        None,
    )
    .unwrap();
    let source = CssColor::from_rgb(rgb);
    assert_eq!(
        source.rgb_value().unwrap().channels()[0],
        CssColorComponent::NumberCalculation(calc)
    );
    let c = CssColor::from_alpha(CssAlphaColor::try_new(source, None).unwrap());
    let before = c.clone();
    let first = c.to_specified_css();
    assert_eq!(first, c.to_specified_css());
    assert_eq!(c, before);
    assert_eq!(raw, components("calc(1 / 128)"));
    assert_eq!(first.unwrap(), "alpha(from rgb(calc(0.007813) none none))");
}
#[test]
fn clean_validation_preserves_raw_graph_and_siblings() {
    let source = "color: alpha(from rgb(calc(1 / 128) none none)) !important; opacity: .5";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(&validate_style_attribute(source).unwrap(), report.syntax());
    assert_eq!(report.syntax().len(), 2);
    let before = report.syntax().clone();
    let d = &report.syntax()[0];
    assert_eq!(d.importance(), CssImportance::Important);
    assert_eq!(
        d.value_components().serialize().unwrap().as_css(),
        " alpha(from rgb(calc(1 / 128) none none)) "
    );
    let text = declaration_color(d).to_specified_css();
    assert_eq!(report.syntax(), &before);
    assert_eq!(text.unwrap(), "alpha(from rgb(calc(0.007813) none none))");
}
#[test]
fn invalid_origin_recovery_retains_sibling_control() {
    let report = parse_style_attribute("color: alpha(from rgb(calc(1px) none none)); opacity: .5");
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
}
#[test]
fn origin_substitution_stays_pending_control() {
    let src = "alpha(from rgb(calc(var(--n)) none none))";
    let d = declaration(src);
    let known = d.known().unwrap();
    assert!(known.property_value().is_none());
    assert_eq!(known.substitution_dependent().unwrap().as_css(), src);
    assert!(matches!(
        expand_declaration(&d).unwrap(),
        CssExpansion::Pending(_)
    ));
    let error =
        CssNumberCalculation::try_from_components(components("calc(var(--n))")).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::SubstitutionRequired
    );
    assert!(error.origin().is_some());
}

text_case!(
    rgb_percentage_slot_0,
    "alpha(from rgb(calc(1% / 128) none none))",
    "alpha(from rgb(calc(0.007813%) none none))"
);

text_case!(
    rgb_percentage_slot_2,
    "alpha(from rgb(none none calc(1% / 128)))",
    "alpha(from rgb(none none calc(0.007813%)))"
);

text_case!(
    hsl_percentage_slot_2,
    "alpha(from hsl(none none calc(1% / 128)))",
    "alpha(from hsl(none none calc(0.007813%)))"
);

text_case!(
    hwb_percentage_slot_2,
    "alpha(from hwb(none none calc(1% / 128)))",
    "alpha(from hwb(none none calc(0.007813%)))"
);

text_case!(
    lab_percentage_slot_0,
    "alpha(from lab(calc(1% / 128) none none))",
    "alpha(from lab(calc(0.007813%) none none))"
);

text_case!(
    lab_percentage_slot_2,
    "alpha(from lab(none none calc(1% / 128)))",
    "alpha(from lab(none none calc(0.007813%)))"
);

text_case!(
    lch_percentage_slot_0,
    "alpha(from lch(calc(1% / 128) none none))",
    "alpha(from lch(calc(0.007813%) none none))"
);

text_case!(
    oklab_percentage_slot_0,
    "alpha(from oklab(calc(1% / 128) none none))",
    "alpha(from oklab(calc(0.007813%) none none))"
);

text_case!(
    oklab_percentage_slot_2,
    "alpha(from oklab(none none calc(1% / 128)))",
    "alpha(from oklab(none none calc(0.007813%)))"
);

text_case!(
    oklch_percentage_slot_0,
    "alpha(from oklch(calc(1% / 128) none none))",
    "alpha(from oklch(calc(0.007813%) none none))"
);

text_case!(
    predefined_percentage_slot_0,
    "alpha(from color(srgb calc(1% / 128) none none))",
    "alpha(from color(srgb calc(0.007813%) none none))"
);

text_case!(
    predefined_percentage_slot_2,
    "alpha(from color(srgb none none calc(1% / 128)))",
    "alpha(from color(srgb none none calc(0.007813%)))"
);

text_case!(
    custom_percentage_slot_0,
    "alpha(from color(--P calc(1% / 128) none none))",
    "alpha(from color(--P calc(0.007813%) none none))"
);

text_case!(
    custom_percentage_slot_2,
    "alpha(from color(--P none none calc(1% / 128)))",
    "alpha(from color(--P none none calc(0.007813%)))"
);

#[test]
fn checked_percentage_origin_preserves_identity_dimension_and_provenance() {
    let raw = components("calc(1% / 128)");
    let calc = CssPercentageCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(calc.result_type(), CssCalculationType::Percentage);
    assert_eq!(calc.components(), &raw);
    assert_eq!(calc.expression().origin(), raw.items()[0].origin());
    let rgb = CssRgbColor::try_new(
        CssColorSyntax::Modern,
        [
            CssColorComponent::PercentageCalculation(calc.clone()),
            CssColorComponent::None,
            CssColorComponent::None,
        ],
        None,
    )
    .unwrap();
    let source = CssColor::from_rgb(rgb);
    assert_eq!(
        source.rgb_value().unwrap().channels()[0],
        CssColorComponent::PercentageCalculation(calc)
    );
    let value = CssColor::from_alpha(CssAlphaColor::try_new(source, None).unwrap());
    let before = value.clone();
    let first = value.to_specified_css();
    assert_eq!(first, value.to_specified_css());
    assert_eq!(value, before);
    assert_eq!(raw, components("calc(1% / 128)"));
    assert_eq!(first.unwrap(), "alpha(from rgb(calc(0.007813%) none none))");
}
// A subnormal needs zero coefficient text after rounding, even though the
// existing color capture requires hundreds of scratch digits.
#[test]
fn rounded_tiny_capture_has_final_zero_text_budget() {
    assert_limits(
        "alpha(from rgb(calc(5e-324) none none))",
        "alpha(from rgb(calc(0) none none))",
        6,
        5,
    );
}

text_case!(
    hsl_explicit_unclamped_alpha_number,
    "alpha(from hsl(none none none / calc(120.1234567)))",
    "alpha(from hsl(none none none / calc(120.123457)))"
);

text_case!(
    hsl_explicit_unclamped_alpha_percentage,
    "alpha(from hsl(none none none / calc(120.1234567%)))",
    "alpha(from hsl(none none none / calc(120.123457%)))"
);

text_case!(
    hwb_explicit_unclamped_alpha_number,
    "alpha(from hwb(none none none / calc(120.1234567)))",
    "alpha(from hwb(none none none / calc(120.123457)))"
);

text_case!(
    hwb_explicit_unclamped_alpha_percentage,
    "alpha(from hwb(none none none / calc(120.1234567%)))",
    "alpha(from hwb(none none none / calc(120.123457%)))"
);

text_case!(
    lab_explicit_unclamped_alpha_number,
    "alpha(from lab(none none none / calc(120.1234567)))",
    "alpha(from lab(none none none / calc(120.123457)))"
);

text_case!(
    lab_explicit_unclamped_alpha_percentage,
    "alpha(from lab(none none none / calc(120.1234567%)))",
    "alpha(from lab(none none none / calc(120.123457%)))"
);

text_case!(
    lch_explicit_unclamped_alpha_number,
    "alpha(from lch(none none none / calc(120.1234567)))",
    "alpha(from lch(none none none / calc(120.123457)))"
);

text_case!(
    lch_explicit_unclamped_alpha_percentage,
    "alpha(from lch(none none none / calc(120.1234567%)))",
    "alpha(from lch(none none none / calc(120.123457%)))"
);

text_case!(
    oklab_explicit_unclamped_alpha_number,
    "alpha(from oklab(none none none / calc(120.1234567)))",
    "alpha(from oklab(none none none / calc(120.123457)))"
);

text_case!(
    oklab_explicit_unclamped_alpha_percentage,
    "alpha(from oklab(none none none / calc(120.1234567%)))",
    "alpha(from oklab(none none none / calc(120.123457%)))"
);

text_case!(
    oklch_explicit_unclamped_alpha_number,
    "alpha(from oklch(none none none / calc(120.1234567)))",
    "alpha(from oklch(none none none / calc(120.123457)))"
);

text_case!(
    oklch_explicit_unclamped_alpha_percentage,
    "alpha(from oklch(none none none / calc(120.1234567%)))",
    "alpha(from oklch(none none none / calc(120.123457%)))"
);

text_case!(
    predefined_explicit_unclamped_alpha_number,
    "alpha(from color(srgb none none none / calc(120.1234567)))",
    "alpha(from color(srgb none none none / calc(120.123457)))"
);

text_case!(
    predefined_explicit_unclamped_alpha_percentage,
    "alpha(from color(srgb none none none / calc(120.1234567%)))",
    "alpha(from color(srgb none none none / calc(120.123457%)))"
);

text_case!(
    custom_explicit_unclamped_alpha_number,
    "alpha(from color(--P none none none / calc(120.1234567)))",
    "alpha(from color(--P none none none / calc(120.123457)))"
);

text_case!(
    custom_explicit_unclamped_alpha_percentage,
    "alpha(from color(--P none none none / calc(120.1234567%)))",
    "alpha(from color(--P none none none / calc(120.123457%)))"
);

text_case!(
    nonfinite_percentage_wrapper_control,
    "alpha(from rgb(calc(infinity * 1%) none none))",
    "alpha(from rgb(calc(infinity * 1%) none none))"
);
text_case!(
    nonfinite_angle_wrapper_control,
    "alpha(from hsl(calc(infinity * 1deg) none none))",
    "alpha(from hsl(calc(infinity * 1deg) none none))"
);
#[test]
fn programmatic_calculation_preserves_exact_leaf_spelling_and_origins() {
    let leaf = CssComponentValue::try_number(".0078125").unwrap();
    assert_eq!(leaf.origin(), &CssValueOrigin::Programmatic);
    let function = CssComponentValue::try_function(
        "calc",
        CssComponentValues::try_new(vec![leaf.clone()]).unwrap(),
    )
    .unwrap();
    let raw = CssComponentValues::try_new(vec![function]).unwrap();
    let calculation = CssNumberCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(calculation.components(), &raw);
    assert_eq!(
        calculation.expression().origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(raw.serialize().unwrap().as_css(), "calc(.0078125)");
    let source = CssColor::from_rgb(
        CssRgbColor::try_new(
            CssColorSyntax::Modern,
            [
                CssColorComponent::NumberCalculation(calculation.clone()),
                CssColorComponent::None,
                CssColorComponent::None,
            ],
            None,
        )
        .unwrap(),
    );
    assert_eq!(
        source.rgb_value().unwrap().channels()[0],
        CssColorComponent::NumberCalculation(calculation)
    );
    let value = CssColor::from_alpha(CssAlphaColor::try_new(source, None).unwrap());
    let before = value.clone();
    let first = value.to_specified_css();
    assert_eq!(first, value.to_specified_css());
    assert_eq!(value, before);
    assert_eq!(leaf.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(raw.serialize().unwrap().as_css(), "calc(.0078125)");
    assert_eq!(first.unwrap(), "alpha(from rgb(calc(0.007813) none none))");
}

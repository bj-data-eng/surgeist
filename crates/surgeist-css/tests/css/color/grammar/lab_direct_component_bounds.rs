//! Color4 CRD 2026-09-08 §§9.3/9.4 parsed-value L/C bounds. Color5 WD
//! 2026-09-08 §§11.1–11.3 preserves distinct Mix/Origin/relative roles.
//! Direct finite decimal tokens remain exact even beyond binary64 and i128.
//! Expected text follows exact percentage scales and the selected standalone
//! domains; resource oracles count the adopted logical classifier operation.
//! These public tests do not inspect implementation source or private helpers.

use crate::css::support::lab_direct_component_cases::lab_direct_component_cases;
use crate::css::support::{
    checked_color as color, checked_color_declaration as declaration,
    component_values as components, declaration_color,
};
use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn assert_text(source: &str, expected: &str) {
    let checked = declaration(source);
    let source_attribute = format!("color:{source}!important;opacity:.5");
    let report = parse_style_attribute(&source_attribute);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let parsed = &report.syntax()[0];
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert_eq!(checked.importance(), CssImportance::Important);
    assert_eq!(parsed.importance(), CssImportance::Important);
    assert_eq!(checked.value_components(), &components(source));
    assert_eq!(
        parsed.value_components().serialize().unwrap().as_css(),
        source
    );
    let CssValueOrigin::Parsed(origin) = parsed.value_components().items()[0].origin() else {
        panic!("real parsed source origin")
    };
    assert_eq!(origin.source().as_str(), source_attribute);
    assert_eq!(origin.span().start().byte_offset().value(), 6);
    // A function component's public origin is its opening token, including
    // the authored name and '('; its child and closing tokens own their spans.
    assert_eq!(
        origin.span().end().byte_offset().value(),
        6 + source.find('(').unwrap() + 1
    );
    let checked_before = checked.clone();
    let parsed_before = parsed.clone();
    // Execute both actual entry routes before asserting either output, so RED
    // in one route does not prevent the other route's writer from executing.
    let checked_result = declaration_color(&checked).to_specified_css();
    let parsed_result = declaration_color(parsed).to_specified_css();
    assert_eq!(
        checked_result,
        declaration_color(&checked).to_specified_css()
    );
    assert_eq!(parsed_result, declaration_color(parsed).to_specified_css());
    assert_eq!(checked, checked_before);
    assert_eq!(parsed, &parsed_before);
    assert_eq!(checked_result.unwrap(), expected, "{source}: checked entry");
    assert_eq!(parsed_result.unwrap(), expected, "{source}: parsed entry");
    // Reentry complements the explicit independent literal oracle.
    assert_eq!(color(expected).to_specified_css().unwrap(), expected);
}

macro_rules! text_case {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            assert_text($source, $expected);
        }
    };
}

lab_direct_component_cases!(text_case);

text_case!(
    mix_direct_child_lightness_clips_to_parsed_domain,
    "color-mix(lab(125 0 0), blue)",
    "color-mix(lab(100 0 0), blue)"
);
text_case!(
    mix_direct_lightness_clips_beside_calculated_axis,
    "color-mix(lab(125 calc(0) 0), blue)",
    "color-mix(lab(100 calc(0) 0), blue)"
);
text_case!(
    origin_mix_preserves_individual_child_parsed_bounds,
    "alpha(from color-mix(lab(125 0 0), blue))",
    "alpha(from color-mix(lab(100 0 0), blue))"
);

fn assert_limited(value: &CssColor, limits: Limits, expected: &str) {
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, &before);
    assert_eq!(result.unwrap(), expected);
    assert_eq!(color(expected).to_specified_css().unwrap(), expected);
}

fn assert_error(value: &CssColor, limits: Limits, kind: Kind) {
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, &before);
    assert_eq!(result.unwrap_err().kind(), kind);
}

macro_rules! limit_success {
    ($name:ident, $source:literal, $inputs:expr, $projections:expr, $bytes:expr, $expected:literal) => {
        #[test]
        fn $name() {
            assert_limited(
                &color($source),
                Limits::new($inputs, $projections, $bytes),
                $expected,
            );
        }
    };
}
macro_rules! limit_error {
    ($name:ident, $source:literal, $inputs:expr, $projections:expr, $bytes:expr, $kind:ident) => {
        #[test]
        fn $name() {
            assert_error(
                &color($source),
                Limits::new($inputs, $projections, $bytes),
                Kind::$kind,
            );
        }
    };
}

// Root1 + one selected classifier1. Each of three scalar slots is an input;
// zero a/b rational paths have no nonzero coefficient limbs. Equality uses
// normalized metadata; > uses the existing inclusive comparison once.
limit_success!(
    greater_endpoint_exact_resources,
    "lab(101 0 0)",
    4,
    2,
    12,
    "lab(100 0 0)"
);
limit_success!(
    equal_endpoint_exact_resources,
    "lab(100 0 0)",
    4,
    2,
    12,
    "lab(100 0 0)"
);
limit_success!(
    huge_endpoint_exact_resources,
    "lab(1e400 0 0)",
    4,
    2,
    12,
    "lab(100 0 0)"
);
limit_success!(
    beyond_i128_endpoint_exact_resources,
    "lab(1e999999999999999999999999999999999999999999 0 0)",
    4,
    2,
    12,
    "lab(100 0 0)"
);
limit_success!(
    negative_endpoint_exact_resources,
    "lab(-1e-400 0 0)",
    4,
    2,
    10,
    "lab(0 0 0)"
);
limit_error!(
    greater_endpoint_one_short_input,
    "lab(101 0 0)",
    3,
    usize::MAX,
    usize::MAX,
    InputNodeLimit
);
limit_error!(
    equal_endpoint_one_short_input,
    "lab(100 0 0)",
    3,
    usize::MAX,
    usize::MAX,
    InputNodeLimit
);
limit_error!(
    greater_endpoint_one_short_projection,
    "lab(101 0 0)",
    usize::MAX,
    1,
    usize::MAX,
    ProjectionNodeLimit
);
limit_error!(
    equal_endpoint_one_short_projection,
    "lab(100 0 0)",
    usize::MAX,
    1,
    usize::MAX,
    ProjectionNodeLimit
);
limit_error!(
    greater_endpoint_one_short_final_byte,
    "lab(101 0 0)",
    usize::MAX,
    usize::MAX,
    11,
    ByteLimit
);
limit_error!(
    equal_endpoint_one_short_final_byte,
    "lab(100 0 0)",
    usize::MAX,
    usize::MAX,
    11,
    ByteLimit
);
limit_error!(
    negative_endpoint_one_short_final_byte,
    "lab(-1e-400 0 0)",
    usize::MAX,
    usize::MAX,
    9,
    ByteLimit
);

// For50, root1 + classifier1 + one lexical coefficient limb1 +
// multiplication by1 limb1 + existing emission clone1 = five projections.
limit_success!(
    strictly_smaller_exact_resources,
    "lab(50 0 0)",
    4,
    5,
    11,
    "lab(50 0 0)"
);
limit_error!(
    strictly_smaller_one_short_input,
    "lab(50 0 0)",
    3,
    usize::MAX,
    usize::MAX,
    InputNodeLimit
);
limit_error!(
    strictly_smaller_one_short_projection,
    "lab(50 0 0)",
    usize::MAX,
    4,
    usize::MAX,
    ProjectionNodeLimit
);
limit_error!(
    strictly_smaller_one_short_final_byte,
    "lab(50 0 0)",
    usize::MAX,
    usize::MAX,
    10,
    ByteLimit
);
limit_error!(
    zero_input_precedes_other_limits,
    "lab(125 0 0)",
    0,
    0,
    0,
    InputNodeLimit
);
limit_error!(
    zero_projection_precedes_bytes_after_root_input,
    "lab(125 0 0)",
    1,
    0,
    0,
    ProjectionNodeLimit
);
limit_error!(
    zero_output_budget_is_typed,
    "lab(125 0 0)",
    usize::MAX,
    usize::MAX,
    0,
    ByteLimit
);
limit_error!(
    positive_beyond_i128_inrange_exact_text_is_bounded,
    "lab(1e-999999999999999999999999999999999999999999 0 0)",
    usize::MAX,
    usize::MAX,
    12,
    ByteLimit
);
limit_error!(
    unbounded_positive_chroma_keeps_output_limit,
    "lch(50 1e2000000 30)",
    usize::MAX,
    usize::MAX,
    64,
    ByteLimit
);
limit_error!(
    unbounded_axis_keeps_output_limit,
    "lab(50 1e2000000 0)",
    usize::MAX,
    usize::MAX,
    64,
    ByteLimit
);
limit_error!(
    unbounded_axis_beyond_i128_keeps_typed_failure,
    "lab(50 1e999999999999999999999999999999999999999999 0)",
    usize::MAX,
    usize::MAX,
    64,
    ByteLimit
);
limit_error!(
    retained_calculation_text_cannot_be_replaced_by_endpoint,
    "lab(calc(10000000000) 0 0)",
    usize::MAX,
    usize::MAX,
    12,
    ByteLimit
);
limit_error!(
    retained_contextual_text_cannot_be_replaced_by_endpoint,
    "lab(calc(1em / 1px) 0 0)",
    usize::MAX,
    usize::MAX,
    12,
    ByteLimit
);

// Retained calc text is17 bytes; the final function is4 +3 +1 +17 +1 +1 +1
// =28 bytes. The calculation must still capture its17-byte scratch before
// composition. Direct1e400 needs no401-byte pre-clamp scratch text.
limit_success!(
    clipped_huge_lightness_and_retained_calculation_exact_bytes,
    "lab(1e400 calc(10000000000) 0)",
    usize::MAX,
    usize::MAX,
    28,
    "lab(100 calc(10000000000) 0)"
);
limit_error!(
    clipped_huge_lightness_and_retained_calculation_one_short_byte,
    "lab(1e400 calc(10000000000) 0)",
    usize::MAX,
    usize::MAX,
    27,
    ByteLimit
);

fn pair() -> CssBorderColorPair {
    let value = color("lab(1e400 0 0)");
    CssBorderColorPair::new(value.clone(), Some(value))
}

fn assert_pair_result(value: &CssBorderColorPair, limits: Limits, expected: Result<&str, Kind>) {
    let before = value.clone();
    let result = value.serialize_specified_with_limits(limits);
    assert_eq!(result, value.serialize_specified_with_limits(limits));
    assert_eq!(value, &before);
    match expected {
        Ok(expected) => assert_eq!(result.unwrap(), expected),
        Err(kind) => assert_eq!(result.unwrap_err().kind(), kind),
    }
}

// Pair root1 + two color inputs4 =9; pair root1 + two projections2 =5.
// The independently counted final12 + separator1 + final12 =25 bytes.
#[test]
fn sibling_endpoint_colors_share_exact_resources() {
    assert_pair_result(
        &pair(),
        Limits::new(9, 5, 25),
        Ok("lab(100 0 0) lab(100 0 0)"),
    );
}
#[test]
fn sibling_endpoint_colors_one_short_input_is_typed() {
    assert_pair_result(
        &pair(),
        Limits::new(8, usize::MAX, usize::MAX),
        Err(Kind::InputNodeLimit),
    );
}
#[test]
fn sibling_endpoint_colors_one_short_projection_is_typed() {
    assert_pair_result(
        &pair(),
        Limits::new(usize::MAX, 4, usize::MAX),
        Err(Kind::ProjectionNodeLimit),
    );
}
#[test]
fn sibling_endpoint_colors_one_short_byte_is_typed() {
    assert_pair_result(
        &pair(),
        Limits::new(usize::MAX, usize::MAX, 24),
        Err(Kind::ByteLimit),
    );
}
#[test]
fn utf8_prefix_leaves_exact_endpoint_output_budget() {
    let value = CssBorderColorPair::new(color("color(--é 0)"), Some(color("lab(1e400 0 0)")));
    // The actual first sibling + separating space uses14 UTF-8 bytes;12 remain.
    assert_pair_result(
        &value,
        Limits::new(usize::MAX, usize::MAX, 26),
        Ok("color(--é 0) lab(100 0 0)"),
    );
}
#[test]
fn utf8_prefix_one_short_endpoint_output_budget_is_typed() {
    let value = CssBorderColorPair::new(color("color(--é 0)"), Some(color("lab(1e400 0 0)")));
    assert_pair_result(
        &value,
        Limits::new(usize::MAX, usize::MAX, 25),
        Err(Kind::ByteLimit),
    );
}
#[test]
fn unbounded_second_sibling_failure_keeps_entire_pair_unchanged() {
    let value =
        CssBorderColorPair::new(color("lab(1e400 0 0)"), Some(color("lab(50 1e2000000 0)")));
    assert_pair_result(
        &value,
        Limits::new(usize::MAX, usize::MAX, 64),
        Err(Kind::ByteLimit),
    );
}
#[test]
fn contextual_second_sibling_failure_keeps_entire_pair_unchanged() {
    let value = CssBorderColorPair::new(
        color("color(--é 0)"),
        Some(color("lab(calc(1em / 1px) 0 0)")),
    );
    assert_pair_result(
        &value,
        Limits::new(usize::MAX, usize::MAX, 26),
        Err(Kind::ByteLimit),
    );
}

fn number_literal(spelling: &str, programmatic: bool) -> CssColorComponent {
    let component = if programmatic {
        CssComponentValue::try_number(spelling).unwrap()
    } else {
        components(spelling).items()[0].clone()
    };
    let literal = CssColorNumberLiteral::try_from_component(component).unwrap();
    assert_eq!(literal.numeric().representation(), spelling);
    if programmatic {
        assert_eq!(literal.origin(), &CssValueOrigin::Programmatic);
    } else {
        let CssValueOrigin::Parsed(origin) = literal.origin() else {
            panic!("parsed literal origin")
        };
        assert_eq!(origin.source().as_str(), spelling);
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(origin.span().end().byte_offset().value(), spelling.len());
    }
    CssColorComponent::Number(literal)
}

fn checked_lab(programmatic: bool, oklab: bool) {
    let lightness = number_literal("1e999999999999999999999999999999999999999999", programmatic);
    let before_lightness = lightness.clone();
    let model = CssLabColor::try_new(
        lightness,
        number_literal("0", programmatic),
        number_literal("0", programmatic),
        None,
    )
    .unwrap();
    let value = if oklab {
        CssColor::from_oklab(model)
    } else {
        CssColor::from_lab(model)
    };
    let expected = if oklab {
        "oklab(1 0 0)"
    } else {
        "lab(100 0 0)"
    };
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(result, value.to_specified_css());
    assert_eq!(value, before);
    let retained = if oklab {
        value.oklab_value().unwrap()
    } else {
        value.lab_value().unwrap()
    };
    assert_eq!(retained.lightness(), &before_lightness);
    assert_eq!(result.unwrap(), expected);
}
#[test]
fn checked_lab_programmatic_extreme_literal_clips_without_mutation() {
    checked_lab(true, false);
}
#[test]
fn checked_lab_parsed_literal_origin_survives_clipping() {
    checked_lab(false, false);
}
#[test]
fn checked_oklab_programmatic_extreme_literal_clips_without_mutation() {
    checked_lab(true, true);
}
#[test]
fn checked_oklab_parsed_literal_origin_survives_clipping() {
    checked_lab(false, true);
}

#[test]
fn checked_lch_percentage_literal_keeps_coefficient_kind_and_origin() {
    let supplied = components("-20%").items()[0].clone();
    let literal = CssColorPercentageLiteral::try_from_component(supplied.clone()).unwrap();
    assert_eq!(literal.numeric().representation(), "-20");
    assert_eq!(literal.origin(), supplied.origin());
    let chroma = CssColorComponent::Percentage(literal);
    let before_chroma = chroma.clone();
    let model = CssLchColor::try_new(
        number_literal("50", false),
        chroma,
        CssColorHue::Number(
            CssColorNumberLiteral::try_from_component(components("30").items()[0].clone()).unwrap(),
        ),
        None,
    )
    .unwrap();
    let value = CssColor::from_lch(model);
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(result, value.to_specified_css());
    assert_eq!(value, before);
    assert_eq!(value.lch_value().unwrap().chroma(), &before_chroma);
    assert_eq!(result.unwrap(), "lch(50 0 30)");
}

fn percentage_literal(spelling: &str, programmatic: bool) -> CssColorComponent {
    let supplied = if programmatic {
        CssComponentValue::try_token(spelling).unwrap()
    } else {
        components(spelling).items()[0].clone()
    };
    let literal = CssColorPercentageLiteral::try_from_component(supplied.clone()).unwrap();
    assert_eq!(
        literal.numeric().representation(),
        spelling.strip_suffix('%').unwrap()
    );
    assert_eq!(literal.origin(), supplied.origin());
    if programmatic {
        assert_eq!(literal.origin(), &CssValueOrigin::Programmatic);
    } else {
        let CssValueOrigin::Parsed(origin) = literal.origin() else {
            panic!("parsed percentage literal origin")
        };
        assert_eq!(origin.source().as_str(), spelling);
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(origin.span().end().byte_offset().value(), spelling.len());
    }
    CssColorComponent::Percentage(literal)
}

fn checked_oklch_percentages(programmatic: bool) {
    let lightness = percentage_literal("120%", programmatic);
    let chroma = percentage_literal("-20%", programmatic);
    let before_lightness = lightness.clone();
    let before_chroma = chroma.clone();
    let model = CssLchColor::try_new(
        lightness,
        chroma,
        CssColorHue::Number(
            CssColorNumberLiteral::try_from_component(if programmatic {
                CssComponentValue::try_number("30").unwrap()
            } else {
                components("30").items()[0].clone()
            })
            .unwrap(),
        ),
        None,
    )
    .unwrap();
    let value = CssColor::from_oklch(model);
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(result, value.to_specified_css());
    assert_eq!(value, before);
    assert_eq!(value.oklch_value().unwrap().lightness(), &before_lightness);
    assert_eq!(value.oklch_value().unwrap().chroma(), &before_chroma);
    assert_eq!(result.unwrap(), "oklch(1 0 30)");
}

#[test]
fn checked_oklch_parsed_percentage_kinds_and_origins_survive_clipping() {
    checked_oklch_percentages(false);
}

#[test]
fn checked_oklch_programmatic_percentage_kinds_and_origins_survive_clipping() {
    checked_oklch_percentages(true);
}

#[test]
fn long_coefficient_upper_neighbor_clips_from_actual_public_token() {
    // 100 plus a positive finite decimal tail is mathematically greater100.
    let source = format!("lab(100.{}1 0 0)", "0".repeat(4096));
    assert_text(&source, "lab(100 0 0)");
}

#[test]
fn long_exponent_clips_without_expanding_decimal_output() {
    let source = format!("lab(1e{} 0 0)", "9".repeat(4096));
    assert_text(&source, "lab(100 0 0)");
}

#[test]
fn long_negative_exponent_chroma_clips_from_actual_public_token() {
    let source = format!("lch(50 -1e-{} 30)", "9".repeat(4096));
    assert_text(&source, "lch(50 0 30)");
}

#[test]
fn positive_inrange_binary_underflow_keeps_exact_decimal_text() {
    // 10^-400 is positive and in range, never the clipped lower endpoint.
    let expected = format!("lab(0.{}1 0 0)", "0".repeat(399));
    assert_text("lab(1e-400 0 0)", &expected);
}

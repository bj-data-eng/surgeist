//! Declared relative calculations: Color 5 WD 2026-09-08 §11.3, CSSOM WD
//! 2021-08-26 component numbers, and Values 4 WD 2024-03-12 calculation trees.
//! Six fractional places apply to actual binary64 coefficients, retaining
//! references. Halfway ties away from zero are the adopted operational WebKit
//! FIXED policy, not a claim that CSSOM normatively resolves halfway direction.

use crate::css::support::color_and_selector_fixtures::{
    assert_color_limit_error, assert_color_limits,
};
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::{
    checked_color as color, checked_color_declaration as declaration,
    component_values as components, declaration_color,
};
use surgeist_css::*;

type Limits = CssSerializationLimits;
type Kind = SerializationFailure;

fn assert_text(source: &str, expected: &str) {
    let declaration = declaration(source);
    let before = declaration.clone();
    assert_eq!(declaration.value_components(), &components(source));
    let value = declaration_color(&declaration);
    // Repeated successful or failed serialization must never rewrite the graph.
    let first = value.serialize_specified(Default::default());
    let second = value.serialize_specified(Default::default());
    assert_eq!(declaration, before);
    assert_eq!(first, second);
    assert_eq!(first.unwrap(), expected, "{source}");
}

// Each fixture is a test: an early failure cannot hide a different owning route.
macro_rules! text_case {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            assert_text($source, $expected);
        }
    };
}

// Exact imported WPT assertion at frozen WebKit commit
// 73aa6c89e2cb77c46184a81aec944e4ab99d114d,
// LayoutTests/imported/w3c/web-platform-tests/css/css-color/parsing/
// color-valid-relative-color.html:329, blob 5f05a55eaaf11263115e594fe67a4f79735caaaf.
text_case!(
    frozen_wpt_lab_keeps_references_and_six_place_third,
    "lab(from lab(50 -30 40) l calc(a / 3) calc(b / 2))",
    "lab(from lab(50 -30 40) l calc(0.333333 * a) calc(0.5 * b))"
);
text_case!(
    predefined_reference_third_remains_symbolic,
    "rgb(from red calc(r / 3) g b)",
    "rgb(from red calc(0.333333 * r) g b)"
);
text_case!(
    custom_reference_third_remains_symbolic_and_case_sensitive,
    "color(from red --P calc(Cyan / 3))",
    "color(from red --P calc(0.333333 * Cyan))"
);

// ±1/128 * 10^6 = ±7812.5 exactly; operational ties go away from zero.
text_case!(
    predefined_positive_dyadic_halfway,
    "rgb(from red calc(1 / 128) g b)",
    "rgb(from red calc(0.007813) g b)"
);
text_case!(
    predefined_negative_dyadic_halfway,
    "rgb(from red calc(-1 / 128) g b)",
    "rgb(from red calc(-0.007813) g b)"
);
text_case!(
    custom_positive_dyadic_halfway,
    "color(from red --P calc(1 / 128))",
    "color(from red --P calc(0.007813))"
);
text_case!(
    custom_negative_dyadic_halfway,
    "color(from red --P calc(-1 / 128))",
    "color(from red --P calc(-0.007813))"
);

// Binary64 5e-7 = 4722366482869645 / 9444732965739290427392,
// strictly below 1/2000000. Rounding its shortest decimal would give the wrong 1.
text_case!(
    predefined_actual_binary_below_half_micro_unit,
    "rgb(from red calc(5e-7 * 1) g b)",
    "rgb(from red calc(0) g b)"
);
text_case!(
    custom_actual_binary_below_half_micro_unit,
    "color(from red --P calc(5e-7 * 1))",
    "color(from red --P calc(0))"
);
text_case!(
    predefined_tiny_finite_coefficient,
    "rgb(from red calc(5e-324) g b)",
    "rgb(from red calc(0) g b)"
);
text_case!(
    custom_tiny_finite_coefficient,
    "color(from red --P calc(5e-324))",
    "color(from red --P calc(0))"
);
text_case!(
    predefined_rounding_carries_to_integer,
    "rgb(from red calc(0.9999996) g b)",
    "rgb(from red calc(1) g b)"
);
text_case!(
    custom_rounding_carries_to_integer,
    "color(from red --P calc(0.9999996))",
    "color(from red --P calc(1))"
);
// The nearest binary64 integer is 7812500000000001 * 128 = ...0128.
text_case!(
    predefined_large_integral_coefficient_has_full_digits,
    "rgb(from red calc(1000000000000000100 * 1) g b)",
    "rgb(from red calc(1000000000000000128) g b)"
);
text_case!(
    custom_large_integral_coefficient_has_full_digits,
    "color(from red --P calc(1000000000000000100 * 1))",
    "color(from red --P calc(1000000000000000128))"
);
// An operation-free calculation leaf keeps its exact authored coefficient (#1297).
text_case!(
    predefined_operation_free_leaf_keeps_its_exact_coefficient,
    "rgb(from red calc(5e-7) g b)",
    "rgb(from red calc(0.000001) g b)"
);
text_case!(
    custom_operation_free_leaf_keeps_its_exact_coefficient,
    "color(from red --P calc(1000000000000000100))",
    "color(from red --P calc(1000000000000000100))"
);

// .78125% becomes .78125/100 = 1/128 exactly. Calculated alpha remains
// in the currently unclamped phase; neither explicit unity nor 2 is omitted.
text_case!(
    predefined_calculated_alpha_number,
    "rgb(from red r g b / calc(1 / 128))",
    "rgb(from red r g b / calc(0.007813))"
);
text_case!(
    predefined_calculated_alpha_percentage,
    "rgb(from red r g b / calc(.78125%))",
    "rgb(from red r g b / calc(0.007813))"
);
text_case!(
    alpha_override_number,
    "alpha(from red / calc(1 / 128))",
    "alpha(from red / calc(0.007813))"
);
text_case!(
    alpha_override_percentage,
    "alpha(from red / calc(.78125%))",
    "alpha(from red / calc(0.007813))"
);
text_case!(
    custom_calculated_alpha_number,
    "color(from red --P Cyan / calc(1 / 128))",
    "color(from red --P Cyan / calc(0.007813))"
);
text_case!(
    custom_calculated_alpha_percentage,
    "color(from red --P Cyan / calc(.78125%))",
    "color(from red --P Cyan / calc(0.007813))"
);
text_case!(
    custom_calculated_channel_percentage,
    "color(from red --P calc(.78125%))",
    "color(from red --P calc(0.007813))"
);
text_case!(
    calculated_alpha_preserves_unclamped_value,
    "alpha(from red / calc(2))",
    "alpha(from red / calc(2))"
);
text_case!(
    calculated_alpha_preserves_explicit_unity,
    "alpha(from red / calc(1))",
    "alpha(from red / calc(1))"
);

text_case!(
    nested_predefined_relative_selected_calculation,
    "rgb(from rgb(from red calc(r + 1 / 128) g b) r g b)",
    "rgb(from rgb(from red calc(0.007813 + r) g b) r g b)"
);
text_case!(
    nested_custom_relative_selected_calculation,
    "rgb(from color(from red --P calc(Cyan + 1 / 128)) r g b)",
    "rgb(from color(from red --P calc(0.007813 + Cyan)) r g b)"
);
text_case!(
    nested_alpha_selected_calculation,
    "color(from alpha(from red / calc(.78125%)) --P Cyan)",
    "color(from alpha(from red / calc(0.007813)) --P Cyan)"
);
text_case!(
    mix_predefined_relative_selected_calculation,
    "color-mix(rgb(from red calc(r / 3) g b), blue)",
    "color-mix(rgb(from red calc(0.333333 * r) g b), blue)"
);
text_case!(
    mix_custom_relative_selected_calculation,
    "color-mix(color(from red --P calc(Cyan / 3)), blue)",
    "color-mix(color(from red --P calc(0.333333 * Cyan)), blue)"
);
text_case!(
    mix_alpha_selected_calculation,
    "color-mix(alpha(from red / calc(.78125%)), blue)",
    "color-mix(alpha(from red / calc(0.007813)), blue)"
);

text_case!(
    ordinary_calculated_origin_rounds_coefficient,
    "rgb(from color(srgb calc(1 / 128) 0 0) r g b)",
    "rgb(from color(srgb calc(0.007813) 0 0) r g b)"
);
text_case!(
    ordinary_alpha_origin_rounds_coefficient,
    "alpha(from rgb(1 2 3 / calc(1 / 128)))",
    "alpha(from rgb(1 2 3 / calc(0.007813)))"
);
text_case!(
    ordinary_mix_weight_retains_calculation_and_percentage,
    "color-mix(in srgb, red calc(.78125%), blue)",
    "color-mix(in srgb, red calc(0.78125%), blue)"
);

text_case!(
    predefined_contextual_product_retains_units_and_grouping,
    "hsl(from red calc(h + 1em / 1px + 1 / 128) s l)",
    "hsl(from red calc(0.007813 + h + (1em / 1px)) s l)"
);
text_case!(
    custom_contextual_product_retains_units_and_grouping,
    "color(from red --P calc(Cyan + 1em / 1px + 1 / 128))",
    "color(from red --P calc(0.007813 + Cyan + (1em / 1px)))"
);
text_case!(
    predefined_nested_sum_retains_symbolic_product,
    "rgb(from red calc((r + 1 / 128) * (g + 1)) g b)",
    "rgb(from red calc((0.007813 + r) * (1 + g)) g b)"
);
text_case!(
    custom_nested_sum_retains_symbolic_product,
    "color(from red --P calc((Cyan + 1 / 128) * (Magenta + 1)))",
    "color(from red --P calc((0.007813 + Cyan) * (1 + Magenta)))"
);

#[test]
fn nonfinite_and_signed_zero_predefined_roots_and_operands_keep_math_syntax() {
    for (expression, expected) in [
        ("calc(0 * -1)", "calc(0)"),
        ("calc(1 / (0 * -1))", "calc(-infinity)"),
        ("calc(0 / 0)", "calc(NaN)"),
        ("calc(infinity)", "calc(infinity)"),
        ("calc(r + 0 * -1)", "calc((0 * -1) + r)"),
        ("calc(r * infinity)", "calc(infinity * r)"),
    ] {
        assert_text(
            &format!("rgb(from red {expression} g b)"),
            &format!("rgb(from red {expected} g b)"),
        );
    }
}

#[test]
fn nonfinite_and_signed_zero_custom_roots_and_operands_keep_math_syntax() {
    for (expression, expected) in [
        ("calc(0 * -1)", "calc(0)"),
        ("calc(1 / (0 * -1))", "calc(-infinity)"),
        ("calc(0 / 0)", "calc(NaN)"),
        ("calc(infinity)", "calc(infinity)"),
        ("calc(Cyan + 0 * -1)", "calc((0 * -1) + Cyan)"),
        ("calc(Cyan * infinity)", "calc(infinity * Cyan)"),
    ] {
        assert_text(
            &format!("color(from red --P {expression})"),
            &format!("color(from red --P {expected})"),
        );
    }
}

#[test]
fn checked_rust_relative_graph_preserves_domains_references_and_origins() {
    use CssRelativeColorEnvironment as E;
    use CssRelativeColorResultDomain as D;
    let raw = components("calc(r + 1 / 128)");
    let expression = CssRelativeColorExpression::try_from_components(
        raw.clone(),
        E::Rgb,
        D::NumberPercentage,
        Default::default(),
    )
    .unwrap();
    assert_eq!(expression.environment(), E::Rgb);
    assert_eq!(expression.result_domain(), D::NumberPercentage);
    assert_eq!(expression.origin(), raw.items()[0].origin());
    let CssValueOrigin::Parsed(origin) = expression.origin() else {
        panic!("parsed origin")
    };
    assert_eq!(origin.source().as_str(), "calc(r + 1 / 128)");
    let CssRelativeColorExpressionValue::Calculation(calculation) = expression.value() else {
        panic!("retained calculation")
    };
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    assert_eq!(calculation.references(), &[CssRelativeColorChannel::R]);
    assert_eq!(calculation.authored().as_css(), "calc(r + 1 / 128)");
    let channel = |name| {
        CssRelativeColorExpression::try_from_components(
            CssComponentValues::try_new(
                vec![CssComponentValue::try_ident(name).unwrap()],
                Default::default(),
            )
            .unwrap(),
            E::Rgb,
            D::NumberPercentage,
            Default::default(),
        )
        .unwrap()
    };
    let source = CssColor::from_named(CssNamedColor::try_new("red").unwrap());
    let relative = CssRelativeColor::try_new(
        CssRelativeColorFunction::Rgb,
        source.clone(),
        [expression.clone(), channel("g"), channel("b")],
        None,
    )
    .unwrap();
    assert_eq!(relative.source(), &source);
    assert_eq!(relative.channels()[0], expression);
    assert_eq!(
        relative.channels()[1].origin(),
        &CssValueOrigin::Programmatic
    );
    let value = CssColor::from_relative(relative);
    let before = value.clone();
    // The color root charges its input node, then its projection node,
    // before any output byte.
    for (limits, kind) in [
        (Limits::new(0, 0, 0), Kind::InputNodeLimit),
        (Limits::new(100, 0, 100), Kind::ProjectionNodeLimit),
        (Limits::new(100, 100, 0), Kind::ByteLimit),
    ] {
        assert_color_limit_error(&value, limits, kind);
    }
    let first = value.serialize_specified(Default::default());
    assert_eq!(first, value.serialize_specified(Default::default()));
    assert_eq!(value, before);
    assert_eq!(raw, components("calc(r + 1 / 128)"));
    assert_eq!(first.unwrap(), "rgb(from red calc(0.007813 + r) g b)");
}

// Independent charging: each color root costs 1/1; a channel reference costs
// 1/1. Calc(divide) visits wrapper/product/two leaves (4 inputs), creates two
// leaves/inverse/product (4 projections). Calc(percent) visits wrapper/leaf
// (2 inputs), projects leaf plus scale/inverse/product (4 projections).
// Profile identifiers add no visits. Captures charge scratch bounds, then final
// output once. The two-calculation fixture therefore charges 2+4+4 = 10/10.
#[test]
fn predefined_exact_visits_and_final_bytes_are_atomic() {
    const TEXT: &str = "rgb(from red calc(0.007813) g b)";
    assert_eq!(TEXT.len(), 32);
    assert_color_limits(&color("rgb(from red calc(1 / 128) g b)"), TEXT, 8, 8);
}
#[test]
fn alpha_number_exact_visits_and_final_bytes_are_atomic() {
    const TEXT: &str = "alpha(from red / calc(0.007813))";
    assert_eq!(TEXT.len(), 32);
    assert_color_limits(&color("alpha(from red / calc(1 / 128))"), TEXT, 6, 6);
}
#[test]
fn alpha_percentage_exact_scaled_visits_and_final_bytes_are_atomic() {
    assert_color_limits(
        &color("alpha(from red / calc(.78125%))"),
        "alpha(from red / calc(0.007813))",
        4,
        6,
    );
}
#[test]
fn profile_number_exact_visits_and_final_bytes_are_atomic() {
    const TEXT: &str = "color(from red --P calc(0.007813))";
    assert_eq!(TEXT.len(), 34);
    assert_color_limits(&color("color(from red --P calc(1 / 128))"), TEXT, 6, 6);
}
#[test]
fn profile_percentage_exact_scaled_visits_and_final_bytes_are_atomic() {
    assert_color_limits(
        &color("color(from red --P calc(.78125%))"),
        "color(from red --P calc(0.007813))",
        4,
        6,
    );
}
#[test]
fn profile_sibling_calculations_share_cumulative_visits_and_final_bytes() {
    const TEXT: &str = "color(from red --P calc(0.007813) calc(0.007813))";
    assert_eq!(TEXT.len(), 49);
    assert_color_limits(
        &color("color(from red --P calc(1 / 128) calc(1 / 128))"),
        TEXT,
        10,
        10,
    );
}

#[test]
fn discarded_ordinary_rgb_capture_preserves_larger_scratch_requirement() {
    let value = color("rgb(calc(10000000000) 0 0)");
    let before = value.clone();
    // Capture occurs before final RGB output. calc(10000000000) needs seventeen
    // scratch bytes although the clamped final rgb(255, 0, 0) needs fourteen.
    assert_eq!(
        value
            .serialize_specified(Limits::new(usize::MAX, usize::MAX, 17))
            .unwrap(),
        "rgb(255, 0, 0)"
    );
    assert!(
        value
            .serialize_specified(Limits::new(usize::MAX, usize::MAX, 16))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(value, before);
}

#[test]
fn rounded_nested_origin_capture_controls_scratch_and_source_visit_precedence() {
    let value = color("rgb(from color(srgb calc(1 / 128) 0 0) r g b)");
    let before = value.clone();
    // Outer root and b/g/r consume four inputs. Origin root and the first
    // channel's Calc(divide) consume five more. After "rgb(from ", a
    // 22-byte limit leaves 13 bytes, below the rounded calc(0.007813)'s
    // 14-byte scratch. At 23 bytes that capture fits and the zero siblings
    // require more inputs. Eight input slots fail within the calculation.
    for (inputs, bytes, expected) in [
        (9, 22, Kind::ByteLimit),
        (8, 22, Kind::InputNodeLimit),
        (9, 23, Kind::InputNodeLimit),
    ] {
        assert_eq!(
            serialization_failure(
                &value
                    .serialize_specified(Limits::new(inputs, usize::MAX, bytes))
                    .unwrap_err()
            ),
            expected
        );
    }
    assert_eq!(value, before);
}

#[test]
fn declarations_retain_raw_components_importance_and_recovery_siblings() {
    let source = "color: rgb(from red calc(r / 3) g b) !important; opacity: .5";
    let report = CssParser::default().parse_block_contents(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let declaration = &report.syntax()[0];
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        declaration
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        " rgb(from red calc(r / 3) g b) "
    );
    let before = report.syntax().clone();
    let text = declaration_color(declaration).serialize_specified(Default::default());
    assert_eq!(report.syntax(), &before);
    assert_eq!(text.unwrap(), "rgb(from red calc(0.333333 * r) g b)");
}

#[test]
fn invalid_grammars_recover_and_substitutions_remain_pending() {
    for invalid in [
        "rgb(from red calc(r + 1px) g b)",
        "color(from red --P calc(Cyan + 1px))",
        "alpha(from red / calc(alpha + 1px))",
    ] {
        let report =
            CssParser::default().parse_block_contents(format!("color: {invalid}; opacity: .5"));
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
    }
    for pending in [
        "rgb(from red calc(r + var(--n)) g b)",
        "color(from red --P calc(Cyan + var(--n)))",
        "alpha(from red / var(--a))",
    ] {
        let declaration = declaration(pending);
        let known = declaration.known().unwrap();
        assert!(known.property_value().is_none());
        assert_eq!(known.substitution_dependent().unwrap().as_css(), pending);
        assert!(matches!(
            declaration.expand().unwrap(),
            CssExpansion::Pending(_)
        ));
    }
    let original = components("calc(r + var(--n))");
    let before = original.clone();
    let error = CssRelativeColorExpression::try_from_components(
        original,
        CssRelativeColorEnvironment::Rgb,
        CssRelativeColorResultDomain::NumberPercentage,
        Default::default(),
    )
    .unwrap_err();
    assert!(error.is_substitution_required(), "{error:?}");
    assert!(error.origin().is_some());
    assert_eq!(before, components("calc(r + var(--n))"));
}

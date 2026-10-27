//! Authored integer identity is visible through existing parsed aggregate APIs.
//! Grid, Steps and counter integer owners compare integers as CSS Typed OM 1
//! "equal numeric values": the exact value, ignoring spelling and numeric
//! provenance. These tests use no newly added integer type, constructor or
//! accessor.

use crate::css::support::descriptor_phases;
use crate::css::support::sole_declaration as declaration;
use descriptor_phases::*;

use std::fmt::Debug;
use surgeist_css::CssParser;
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::rule::CssRule;
use surgeist_css::rule::counter_style::CssCounterStyleRule;

fn counter(source: &str) -> CssCounterStyleRule {
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("one retained counter definition");
    };
    rule.clone()
}

fn lexical_identity<T: PartialEq + Debug>(
    parse: impl Fn(&str) -> T,
    source: &str,
    alternate_spelling: &str,
) {
    assert_eq!(
        parse(source),
        parse(source),
        "same spelling and coordinates"
    );
    assert_eq!(
        parse(source),
        parse(alternate_spelling),
        "equal values across authored spellings"
    );
}

#[test]
fn parsed_grid_repeat_spelling_and_shifted_origin_are_ignored() {
    let parse = |source: &str| {
        let declaration = declaration(source);
        let Some(CssKnownPropertyValueRef::GridTemplateColumns(value)) =
            declaration.known().unwrap().property_value()
        else {
            panic!("grid columns");
        };
        value.value().clone()
    };
    let source = "grid-template-columns: repeat(1, auto)";
    lexical_identity(parse, source, "grid-template-columns: repeat(+0001, auto)");
    assert_eq!(
        parse(source),
        parse("/* shift */ grid-template-columns: repeat(1, auto)")
    );
}

#[test]
fn parsed_steps_spelling_and_shifted_origin_are_ignored() {
    let parse = |source: &str| {
        let declaration = declaration(source);
        let Some(CssKnownPropertyValueRef::TransitionTimingFunction(value)) =
            declaration.known().unwrap().property_value()
        else {
            panic!("easing list");
        };
        value.value().values()[0].clone()
    };
    let source = "transition-timing-function: steps(1)";
    lexical_identity(parse, source, "transition-timing-function: steps(+0001)");
    assert_eq!(
        parse(source),
        parse("/* shift */ transition-timing-function: steps(1)")
    );
}

#[test]
fn parsed_fixed_start_identity_ignores_spelling_and_numeric_origin() {
    let parse = |source: &str| {
        counter(source)
            .descriptors()
            .system()
            .unwrap()
            .ordinary_system()
            .clone()
    };
    let source = "@counter-style eq { system: fixed 1; symbols: \"x\"; }";
    lexical_identity(
        parse,
        source,
        "@counter-style eq { system: fixed +0001; symbols: \"x\"; }",
    );
    assert_eq!(
        parse(source),
        parse("/* shift */ @counter-style eq { system: fixed 1; symbols: \"x\"; }")
    );
}

#[test]
fn parsed_finite_range_identity_ignores_spelling_and_numeric_origins() {
    let parse = |source: &str| {
        counter(source)
            .descriptors()
            .range()
            .unwrap()
            .ordinary_range()
            .clone()
    };
    let source = "@counter-style eq { symbols: \"x\"; range: 7 9; }";
    lexical_identity(
        parse,
        source,
        "@counter-style eq { symbols: \"x\"; range: +0007 9; }",
    );
    assert_eq!(
        parse(source),
        parse("/* shift */ @counter-style eq { symbols: \"x\"; range: 7 9; }")
    );
}

#[test]
fn parsed_pad_minimum_identity_ignores_spelling_and_numeric_origin() {
    let parse = |source: &str| {
        counter(source)
            .descriptors()
            .pad()
            .unwrap()
            .ordinary_pad()
            .clone()
    };
    let source = "@counter-style eq { symbols: \"x\"; pad: 7 \"_\"; }";
    lexical_identity(
        parse,
        source,
        "@counter-style eq { symbols: \"x\"; pad: +0007 \"_\"; }",
    );
    assert_eq!(
        parse(source),
        parse("/* shift */ @counter-style eq { symbols: \"x\"; pad: 7 \"_\"; }")
    );
}

#[test]
fn parsed_additive_weights_identity_ignores_spelling_and_numeric_origins() {
    let parse = |source: &str| {
        counter(source)
            .descriptors()
            .additive_symbols()
            .unwrap()
            .ordinary_additive_symbols()
            .clone()
    };
    let source = "@counter-style eq { system: additive; additive-symbols: 7 \"x\", 0 \"z\"; }";
    lexical_identity(
        parse,
        source,
        "@counter-style eq { system: additive; additive-symbols: +0007 \"x\", 0 \"z\"; }",
    );
    assert_eq!(
        parse(source),
        parse(
            "/* shift */ @counter-style eq { system: additive; additive-symbols: 7 \"x\", 0 \"z\"; }"
        )
    );
}

#[test]
fn parsed_fixed_default_and_steps_position_omission_remain_authored_distinctions() {
    let fixed = |source: &str| {
        counter(source)
            .descriptors()
            .system()
            .unwrap()
            .ordinary_system()
            .clone()
    };
    assert_ne!(
        fixed("@counter-style eq { system: fixed; symbols: \"x\"; }"),
        fixed("@counter-style eq { system: fixed 1; symbols: \"x\"; }"),
    );
    let steps = |source: &str| {
        let declaration = declaration(source);
        let Some(CssKnownPropertyValueRef::TransitionTimingFunction(value)) =
            declaration.known().unwrap().property_value()
        else {
            panic!("easing list");
        };
        value.value().values()[0].clone()
    };
    assert_ne!(
        steps("transition-timing-function: steps(2)"),
        steps("transition-timing-function: steps(2, end)")
    );
}

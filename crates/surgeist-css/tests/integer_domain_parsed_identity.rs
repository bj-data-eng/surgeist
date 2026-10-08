#![forbid(unsafe_code)]

//! Authored integer identity is visible through existing parsed aggregate APIs.
//! Grid and Steps preserve spelling while ignoring numeric provenance in Eq;
//! counter integer owners preserve both spelling and provenance. Mathematical
//! equivalence does not erase these authored distinctions. These tests use no
//! newly added integer type, constructor or accessor.

#[path = "support/descriptor_phases.rs"]
mod descriptor_phases;
use descriptor_phases::*;

use std::fmt::Debug;
use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}

fn counter(source: &str) -> CssCounterStyleRule {
    let report = parse_sheet(source);
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
    assert_ne!(
        parse(source),
        parse(alternate_spelling),
        "distinct authored spelling"
    );
}

#[test]
fn parsed_grid_repeat_spelling_is_distinct_while_shifted_origin_is_ignored() {
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
fn parsed_steps_spelling_is_distinct_while_shifted_origin_is_ignored() {
    let parse = |source: &str| {
        let declaration = declaration(source);
        let Some(CssKnownPropertyValueRef::TransitionTimingFunction(value)) =
            declaration.known().unwrap().property_value()
        else {
            panic!("easing list");
        };
        value.timing_functions().values()[0].clone()
    };
    let source = "transition-timing-function: steps(1)";
    lexical_identity(parse, source, "transition-timing-function: steps(+0001)");
    assert_eq!(
        parse(source),
        parse("/* shift */ transition-timing-function: steps(1)")
    );
}

#[test]
fn parsed_fixed_start_preserves_spelling_and_numeric_origin_in_identity() {
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
    assert_ne!(
        parse(source),
        parse("/* shift */ @counter-style eq { system: fixed 1; symbols: \"x\"; }")
    );
}

#[test]
fn parsed_finite_range_preserves_spelling_and_numeric_origins_in_identity() {
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
    assert_ne!(
        parse(source),
        parse("/* shift */ @counter-style eq { symbols: \"x\"; range: 7 9; }")
    );
}

#[test]
fn parsed_pad_minimum_preserves_spelling_and_numeric_origin_in_identity() {
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
    assert_ne!(
        parse(source),
        parse("/* shift */ @counter-style eq { symbols: \"x\"; pad: 7 \"_\"; }")
    );
}

#[test]
fn parsed_additive_weights_preserve_spelling_and_numeric_origins_in_identity() {
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
    assert_ne!(
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
        value.timing_functions().values()[0].clone()
    };
    assert_ne!(
        steps("transition-timing-function: steps(2)"),
        steps("transition-timing-function: steps(2, end)")
    );
}

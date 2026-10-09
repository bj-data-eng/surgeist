#[path = "support/descriptor_phases.rs"]
mod descriptor_phases;
use descriptor_phases::*;

use surgeist_css::{
    CssCounterStyleDescriptorKind as DescriptorKind, CssCounterStyleDescriptorRef,
    CssCounterStyleRule, CssCounterStyleSystem, CssRule,
    CssSpecifiedValueSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationLimits as Limits, parse_sheet,
};

fn rule(source: &str) -> CssCounterStyleRule {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::CounterStyle(value)] = report.syntax().rules() else {
        panic!("expected one counter-style rule")
    };
    value.clone()
}

#[test]
fn descriptor_getters_emit_each_associated_value_without_wrappers() {
    let value = rule(concat!(
        "@counter-style x { system: cyclic; negative: '(' ')'; prefix: '['; suffix: ']'; ",
        "range: infinite -1, 1 infinite; pad: '_' 3; fallback: Missing; ",
        "symbols: '0' One url(mark.svg); additive-symbols: X 10, N 0; speak-as: Spoken; }",
    ));
    for (kind, expected) in [
        (DescriptorKind::System, "cyclic"),
        (DescriptorKind::Negative, "\"(\" \")\""),
        (DescriptorKind::Prefix, "\"[\""),
        (DescriptorKind::Suffix, "\"]\""),
        (DescriptorKind::Range, "infinite -1, 1 infinite"),
        (DescriptorKind::Pad, "3 \"_\""),
        (DescriptorKind::Fallback, "Missing"),
        (DescriptorKind::Symbols, "\"0\" One url(\"mark.svg\")"),
        (DescriptorKind::AdditiveSymbols, "10 X, 0 N"),
        (DescriptorKind::SpeakAs, "Spoken"),
    ] {
        assert_eq!(value.descriptor_specified_css(kind).unwrap(), expected);
        assert_eq!(
            value
                .descriptor_specified_css_with_limits(kind, Limits::new(100, 100, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .descriptor_specified_css_with_limits(
                    kind,
                    Limits::new(100, 100, expected.len() - 1)
                )
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
    }
    // Rule/name + ten occurrences + typed values: 42 input/projection nodes.
    let expected = concat!(
        "@counter-style x { system: cyclic; negative: \"(\" \")\"; prefix: \"[\"; ",
        "suffix: \"]\"; range: infinite -1, 1 infinite; pad: 3 \"_\"; fallback: Missing; ",
        "symbols: \"0\" One url(\"mark.svg\"); additive-symbols: 10 X, 0 N; speak-as: Spoken; }",
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(42, 42, expected.len()))
            .unwrap(),
        expected,
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(41, 42, expected.len()))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit,
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(42, 41, expected.len()))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit,
    );
}

#[test]
fn every_omitted_descriptor_getter_returns_empty_without_implicit_initials() {
    let extends = rule("@counter-style x { system: extends Other; }");
    for kind in [
        DescriptorKind::Negative,
        DescriptorKind::Prefix,
        DescriptorKind::Suffix,
        DescriptorKind::Range,
        DescriptorKind::Pad,
        DescriptorKind::Fallback,
        DescriptorKind::Symbols,
        DescriptorKind::AdditiveSymbols,
        DescriptorKind::SpeakAs,
    ] {
        assert_eq!(extends.descriptor_specified_css(kind).unwrap(), "");
        assert_eq!(
            extends
                .descriptor_specified_css_with_limits(kind, Limits::new(0, 0, 0))
                .unwrap(),
            ""
        );
    }
    let implicit = rule("@counter-style x { symbols: a; }");
    assert_eq!(
        implicit
            .descriptor_specified_css(DescriptorKind::System)
            .unwrap(),
        ""
    );
    assert_eq!(
        implicit
            .descriptor_specified_css_with_limits(DescriptorKind::System, Limits::new(0, 0, 0))
            .unwrap(),
        ""
    );
}

#[test]
fn field_projection_selects_last_valid_value_and_accounts_for_earlier_occurrences() {
    let value = rule("@counter-style x { symbols: 'discarded long string'; symbols: b; }");
    let retained = value.clone();
    // Only this field: two occurrences, two symbol lists, two string/ident leaves.
    assert_eq!(
        value
            .descriptor_specified_css_with_limits(DescriptorKind::Symbols, Limits::new(6, 6, 1))
            .unwrap(),
        "b"
    );
    assert_eq!(
        value
            .descriptor_specified_css_with_limits(DescriptorKind::Symbols, Limits::new(5, 6, 1))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(
        value
            .descriptor_specified_css_with_limits(DescriptorKind::Symbols, Limits::new(6, 5, 1))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
    assert_eq!(
        value
            .descriptor_specified_css(DescriptorKind::Symbols)
            .unwrap(),
        "b"
    );
    assert_eq!(value, retained);
}

#[test]
fn name_getter_escapes_the_parsed_literal_name_under_exact_limits() {
    let value = rule(r#"@counter-style C\61se { symbols: a; }"#);
    let expected = "Case";
    assert_eq!(value.name().as_str(), "Case");
    assert_eq!(value.name_specified_css().unwrap(), expected);
    assert_eq!(
        value
            .name_specified_css_with_limits(Limits::new(1, 1, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(0, 1, expected.len()), Kind::InputNodeLimit),
        (Limits::new(1, 0, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(1, 1, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            value
                .name_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(value.name().as_str(), "Case");
}

fn assert_css(source: &str, expected: &str) {
    let value = rule(source);
    let retained = value.clone();
    let css = value.to_specified_css().unwrap();
    assert_eq!(css, expected);
    assert_eq!(value, retained);
    // Parsing canonical text is an independent grammar check. Reprojection is
    // stable despite authored spelling and occurrence identity being different.
    assert_eq!(rule(&css).to_specified_css().unwrap(), css);
}

// Counter Styles 3 §3 chooses the last valid occurrence. Complete rule order
// uses the explicit project policy of descriptor section order, not IDL order.
#[test]
fn all_ten_descriptor_kinds_use_effective_values_in_section_order() {
    assert_css(
        concat!(
            "@counter-style Rich { speak-as: Spoken; additive-symbols: X +0010, N -000; ",
            "symbols: '0' One url(mark.svg); fallback: Missing; pad: '_' +0003; ",
            "range: infinite -0001, +0001 infinite; suffix: '. '; prefix: '['; ",
            "negative: '(' ')'; system: CYCLIC; }",
        ),
        concat!(
            "@counter-style Rich { system: cyclic; negative: \"(\" \")\"; ",
            "prefix: \"[\"; suffix: \". \"; range: infinite -1, 1 infinite; ",
            "pad: 3 \"_\"; fallback: Missing; symbols: \"0\" One url(\"mark.svg\"); ",
            "additive-symbols: 10 X, 0 N; speak-as: Spoken; }",
        ),
    );
}

#[test]
fn each_system_branch_serializes_without_resolving_external_names() {
    for (body, expected) in [
        ("system: cyclic; symbols: a;", "system: cyclic; symbols: a;"),
        (
            "system: symbolic; symbols: a;",
            "system: symbolic; symbols: a;",
        ),
        (
            "system: numeric; symbols: a b;",
            "system: numeric; symbols: a b;",
        ),
        (
            "system: alphabetic; symbols: a b;",
            "system: alphabetic; symbols: a b;",
        ),
        (
            "system: additive; additive-symbols: I 1;",
            "system: additive; additive-symbols: 1 I;",
        ),
        ("system: fixed; symbols: a;", "system: fixed; symbols: a;"),
        (
            "system: fixed -0002; symbols: a;",
            "system: fixed -2; symbols: a;",
        ),
        ("system: extends External;", "system: extends External;"),
    ] {
        assert_css(
            &format!("@counter-style Local {{ {body} }}"),
            &format!("@counter-style Local {{ {expected} }}"),
        );
    }
}

#[test]
fn absent_descriptors_do_not_materialize_implicit_defaults() {
    assert_css(
        "@counter-style x { symbols: a; }",
        "@counter-style x { symbols: a; }",
    );
    assert_css(
        "@counter-style x { system: extends Other; }",
        "@counter-style x { system: extends Other; }",
    );
}

// §3.1.2 gives fixed's optional starting integer the value 1; CSSOM §6.7.2
// removes an optional component only when its omission preserves meaning.
#[test]
fn fixed_default_is_omitted_but_its_authored_spelling_remains() {
    let value = rule("@counter-style x { system: fixed +0001; symbols: a; }");
    assert_eq!(
        value.to_specified_css().unwrap(),
        "@counter-style x { system: fixed; symbols: a; }"
    );
    let CssCounterStyleSystem::Fixed(fixed) =
        value.descriptors().system().unwrap().ordinary_system()
    else {
        panic!("expected fixed")
    };
    assert_eq!(
        fixed
            .first_symbol_value()
            .unwrap()
            .literal()
            .numeric()
            .representation(),
        "+0001"
    );
}

#[test]
fn negative_empty_string_suffix_is_omitted_and_nonempty_suffix_is_preserved() {
    assert_css(
        "@counter-style x { symbols: a; negative: '-' ''; }",
        "@counter-style x { negative: \"-\"; symbols: a; }",
    );
    assert_css(
        "@counter-style x { symbols: a; negative: '(' ')'; }",
        "@counter-style x { negative: \"(\" \")\"; symbols: a; }",
    );
    assert_css(
        "@counter-style x { symbols: a; negative: '-'; }",
        "@counter-style x { negative: \"-\"; symbols: a; }",
    );
}

#[test]
fn equal_range_bounds_remain_two_required_components() {
    assert_css(
        "@counter-style x { symbols: a; range: +0007 0007, infinite infinite; }",
        "@counter-style x { range: 7 7, infinite infinite; symbols: a; }",
    );
    assert_css(
        "@counter-style x { symbols: a; range: AUTO; }",
        "@counter-style x { range: auto; symbols: a; }",
    );
}

#[test]
fn unbounded_integer_magnitudes_are_not_rounded_or_clamped() {
    let huge = "9999999999999999999999999999999999999999999999999999999999999999";
    assert_css(
        &format!(
            "@counter-style x {{ symbols: a; system: fixed +000{huge}; range: -{huge} +000{huge}; pad: +000{huge} '0'; additive-symbols: +000{huge} X, -000 N; }}"
        ),
        &format!(
            "@counter-style x {{ system: fixed {huge}; range: -{huge} {huge}; pad: {huge} \"0\"; symbols: a; additive-symbols: {huge} X, 0 N; }}"
        ),
    );
}

#[test]
fn identifiers_strings_and_url_symbols_use_their_own_escaping() {
    assert_css(
        r#"@counter-style N\61me { system: extends \45xternal; negative: url("minus.svg"); prefix: "a\a b\"c\\d"; suffix: \45nd; fallback: \4fther; speak-as: \53poken; }"#,
        r#"@counter-style Name { system: extends External; negative: url("minus.svg"); prefix: "a\a b\"c\\d"; suffix: End; fallback: Other; speak-as: Spoken; }"#,
    );
    assert_css(
        r#"@counter-style x { symbols: url(icon.svg) src("other.svg" Hint mode(a)); }"#,
        r#"@counter-style x { symbols: url("icon.svg") src("other.svg" Hint mode(a)); }"#,
    );
}

#[test]
fn every_speech_keyword_and_symbolic_reference_remains_unresolved() {
    for (authored, expected) in [
        ("AUTO", "auto"),
        ("BULLETS", "bullets"),
        ("NUMBERS", "numbers"),
        ("WORDS", "words"),
        ("SPELL-OUT", "spell-out"),
        ("Unknown", "Unknown"),
    ] {
        assert_css(
            &format!("@counter-style x {{ symbols: a; speak-as: {authored}; }}"),
            &format!("@counter-style x {{ symbols: a; speak-as: {expected}; }}"),
        );
    }
}

#[test]
fn discarded_duplicates_keep_occurrence_positions_and_numeric_origins() {
    let source = "@counter-style x { symbols: a; pad: +0002 '0'; symbols: b; pad: '_' 3; }";
    let value = rule(source);
    let before = value.clone();
    assert_eq!(
        value.to_specified_css().unwrap(),
        "@counter-style x { pad: 3 \"_\"; symbols: b; }"
    );
    assert_eq!(value, before);
    let occurrences = value.descriptors().occurrences().collect::<Vec<_>>();
    assert_eq!(occurrences.len(), 4);
    let CssCounterStyleDescriptorRef::Pad(first) = occurrences[1] else {
        panic!("expected first pad")
    };
    assert_eq!(
        first.position().byte_offset().value(),
        source.find("pad:").unwrap()
    );
    assert_eq!(
        first
            .ordinary_pad()
            .minimum_length()
            .literal()
            .numeric()
            .representation(),
        "+0002"
    );
    assert_eq!(
        first.ordinary_pad().minimum_length().literal().origin(),
        before
            .descriptors()
            .occurrences()
            .find_map(|value| match value {
                CssCounterStyleDescriptorRef::Pad(value) =>
                    Some(value.ordinary_pad().minimum_length().literal().origin()),
                _ => None,
            })
            .unwrap()
    );
}

#[test]
fn exact_rule_node_and_byte_limits_include_the_complete_wrapper() {
    let value = rule("@counter-style x { symbols: a; }");
    let expected = "@counter-style x { symbols: a; }";
    // Rule + name + occurrence + symbol list + identifier = five nodes.
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(5, 5, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(4, 5, expected.len()), Kind::InputNodeLimit),
        (Limits::new(5, 4, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(5, 5, expected.len() - 1), Kind::ByteLimit),
        (Limits::new(0, 5, expected.len()), Kind::InputNodeLimit),
        (Limits::new(5, 0, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(5, 5, 0), Kind::ByteLimit),
    ] {
        assert_eq!(
            value
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(value.to_specified_css().unwrap(), expected);
}

#[test]
fn suppressed_duplicates_consume_nodes_without_spending_final_bytes() {
    let value = rule("@counter-style x { symbols: 'a very long discarded value'; symbols: b; }");
    let expected = "@counter-style x { symbols: b; }";
    // Rule/name + two occurrences + two lists + two leaves = eight nodes.
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(8, 8, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(7, 8, expected.len()))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(8, 7, expected.len()))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
}

#[test]
fn omitted_optional_values_retain_exact_cumulative_node_cost() {
    let value = rule("@counter-style x { system: fixed +0001; negative: '-' ''; symbols: a; }");
    let expected = "@counter-style x { system: fixed; negative: \"-\"; symbols: a; }";
    // Rule/name + three occurrences + fixed/1 + negative/prefix/suffix + list/symbol.
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(12, 12, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(11, 12, expected.len()))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(12, 11, expected.len()))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
    for (kind, expected, nodes) in [
        (DescriptorKind::System, "fixed", 3),
        (DescriptorKind::Negative, "\"-\"", 4),
    ] {
        assert_eq!(
            value
                .descriptor_specified_css_with_limits(
                    kind,
                    Limits::new(nodes, nodes, expected.len())
                )
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .descriptor_specified_css_with_limits(
                    kind,
                    Limits::new(nodes - 1, nodes, expected.len())
                )
                .unwrap_err()
                .kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            value
                .descriptor_specified_css_with_limits(
                    kind,
                    Limits::new(nodes, nodes - 1, expected.len())
                )
                .unwrap_err()
                .kind(),
            Kind::ProjectionNodeLimit
        );
    }
}

#[test]
fn suppressed_url_children_keep_the_url_owners_nested_node_accounting() {
    let value = rule(
        r#"@counter-style x { symbols: src("discarded-long.svg" Hint mode(a)); symbols: b; }"#,
    );
    let expected = "@counter-style x { symbols: b; }";
    // Rule/name + two occurrences/lists + effective identifier + URL aggregate,
    // target, two modifiers, and retained argument component = twelve nodes.
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(12, 12, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(11, 12, expected.len()))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(12, 11, expected.len()))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
}

#[test]
fn late_descriptor_failure_is_atomic_and_a_fresh_retry_succeeds() {
    let value = rule("@counter-style x { symbols: a; speak-as: Unknown; }");
    let retained = value.clone();
    // Six nodes are consumed before the final speech reference.
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(6, 7, 100))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(value, retained);
    assert_eq!(
        value.to_specified_css().unwrap(),
        "@counter-style x { symbols: a; speak-as: Unknown; }"
    );
}

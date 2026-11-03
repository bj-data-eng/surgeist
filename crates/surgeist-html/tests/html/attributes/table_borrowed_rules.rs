//! The allocation-free span and scope entry points through the public
//! `bounded` front door.
//!
//! Expected values come from the pinned HTML Standard's table sections and
//! its rules for parsing non-negative integers (877d §2.3.4.2, §4.9.12 forming
//! a table, §4.9.10 the th `scope` attribute), never from the implementation.

use surgeist_html::bounded::attributes::{HtmlTableSpanRule, th_scope_state, th_scope_work_bound};
use surgeist_html::{
    attributes::table_header_scope::{HtmlTableHeaderScope, HtmlTableHeaderScopeState},
    values::{DocumentString, HtmlValueInterpreter},
    vocabulary::{KnownHtmlAttribute, KnownHtmlElement},
};

fn value(rule: HtmlTableSpanRule, raw: &str) -> u16 {
    rule.content_value(Some(&DocumentString::from(raw)))
}

#[test]
fn colspan_and_both_span_rules_default_zero_and_failure_to_one_and_cap_at_one_thousand() {
    for rule in [
        HtmlTableSpanRule::CellColspan,
        HtmlTableSpanRule::ColSpan,
        HtmlTableSpanRule::ColgroupSpan,
    ] {
        assert_eq!(rule.content_value(None), 1);
        for (raw, expected) in [
            ("0", 1),
            ("1", 1),
            ("2", 2),
            ("999", 999),
            ("1000", 1000),
            ("1001", 1000),
            ("0001001", 1000),
            ("+1001", 1000),
            ("1000tail", 1000),
            (" \t\n\x0c\r7", 7),
            ("", 1),
            ("many", 1),
            ("-1", 1),
            ("-0", 1),
            ("\u{a0}7", 1),
            ("99999999999999999999999999999999999999999999", 1000),
        ] {
            assert_eq!(value(rule, raw), expected, "{rule:?} {raw:?}");
        }
    }
}

#[test]
fn rowspan_keeps_zero_and_caps_at_sixty_five_thousand_five_hundred_thirty_four() {
    let rule = HtmlTableSpanRule::CellRowspan;
    assert_eq!(rule.content_value(None), 1);
    for (raw, expected) in [
        ("0", 0),
        ("-0", 0),
        ("1", 1),
        ("65534", 65534),
        ("65535", 65534),
        ("0065536", 65534),
        ("99999999999999999999999999999999999999999999", 65534),
        ("", 1),
        ("many", 1),
        ("-1", 1),
    ] {
        assert_eq!(value(rule, raw), expected, "{raw:?}");
    }
}

#[test]
fn borrowed_spans_agree_with_the_report_readers() {
    let interpreter = HtmlValueInterpreter::new(4096);
    let td = KnownHtmlElement::Td.qualified_name();
    for raw in ["", "0", "7", "1001", "70000", "-0", "x"] {
        let colspan = surgeist_html::values::Attribute::new(
            KnownHtmlAttribute::Colspan.qualified_name(),
            raw,
        );
        let rowspan = surgeist_html::values::Attribute::new(
            KnownHtmlAttribute::Rowspan.qualified_name(),
            raw,
        );
        assert_eq!(
            interpreter
                .cell_colspan(&td, Some(&colspan))
                .unwrap()
                .intrinsic()
                .copied(),
            Some(value(HtmlTableSpanRule::CellColspan, raw))
        );
        assert_eq!(
            interpreter
                .cell_rowspan(&td, Some(&rowspan))
                .unwrap()
                .intrinsic()
                .copied(),
            Some(value(HtmlTableSpanRule::CellRowspan, raw))
        );
    }
}

#[test]
fn span_bounds_grow_linearly_and_refuse_unrepresentable_inputs() {
    for rule in [
        HtmlTableSpanRule::CellColspan,
        HtmlTableSpanRule::CellRowspan,
        HtmlTableSpanRule::ColSpan,
        HtmlTableSpanRule::ColgroupSpan,
    ] {
        assert_eq!(rule.work_bound(0), Some(2));
        assert_eq!(rule.work_bound(10), Some(32));
        assert_eq!(rule.work_bound(usize::MAX), None);
    }
}

#[test]
fn scope_keywords_select_their_states_and_everything_else_is_auto() {
    use HtmlTableHeaderScope::{Column, ColumnGroup, Row, RowGroup};
    let state = |raw: &str| th_scope_state(Some(&DocumentString::from(raw)));
    assert_eq!(th_scope_state(None), HtmlTableHeaderScopeState::Auto);
    for (raw, expected) in [
        ("row", Row),
        ("ROW", Row),
        ("rOw", Row),
        ("col", Column),
        ("CoL", Column),
        ("rowgroup", RowGroup),
        ("RowGroup", RowGroup),
        ("colgroup", ColumnGroup),
        ("COLGROUP", ColumnGroup),
    ] {
        assert_eq!(
            state(raw),
            HtmlTableHeaderScopeState::Keyword(expected),
            "{raw:?}"
        );
    }
    for raw in [
        "",
        "auto",
        "AUTO",
        " row",
        "row ",
        "rows",
        "ro",
        "colgrou",
        "colgroups",
        "r\u{ff4f}w",
        "\u{212a}",
        "ro\u{17f}",
    ] {
        assert_eq!(state(raw), HtmlTableHeaderScopeState::Auto, "{raw:?}");
    }
}

#[test]
fn scope_bound_is_the_hand_derived_worst_case_for_each_input_length() {
    // The value header and the four keyword headers cost five for any input. A
    // unit pair is read only against a keyword of the input's own length: `row`
    // and `col` at 3, `rowgroup` and `colgroup` at 8. Each such pair differs at
    // its first unit, so the worst input matches one keyword but for its last
    // unit (n pairs) and costs one pair against the other: 5 + 3 + 1 and
    // 5 + 8 + 1. Every other length, however long, costs five.
    for (units, expected) in [
        (0, 5),
        (1, 5),
        (2, 5),
        (3, 9),
        (4, 5),
        (5, 5),
        (6, 5),
        (7, 5),
        (8, 14),
        (9, 5),
        (10, 5),
        (4096, 5),
        (100_000, 5),
        (usize::MAX, 5),
    ] {
        assert_eq!(th_scope_work_bound(units), Some(expected), "{units} units");
    }
}

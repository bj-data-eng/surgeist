//! Work a table query admits for span and scope reads is the HTML owner's
//! published bound for the raw value, not a local estimate.
//!
//! Varying only the length of one value that the query reads once changes
//! nothing else the query charges, so the change in admitted work must equal
//! the change in the owner's bound for that rule. The owner's bounds are
//! derived from the HTML 877d rules for parsing non-negative integers and the
//! `scope` enumerated attribute; see `surgeist_html::bounded`.

use super::scope_work::{cell, element, fixture, limits, query, usage};
use crate::semantics::support::wide_builder_without_registries;
use surgeist_html::bounded::attributes::{HtmlTableSpanRule, th_scope_work_bound};
use surgeist_html_semantics::{HtmlCapture, HtmlCaptureBuilderSubject, HtmlSemanticUsage};

const SHORT: usize = 3;
const LONG: usize = 2003;

/// `units` ASCII digits with the value one: zeros, then a final one.
fn one_padded(units: usize) -> String {
    format!("{}1", "0".repeat(units - 1))
}

/// A table whose only read of `rule`'s attribute carries a raw value of `units`
/// code units whose span value is one; every other span read is absent.
fn span_fixture(rule: HtmlTableSpanRule, units: usize) -> (HtmlCapture, HtmlCaptureBuilderSubject) {
    let (mut b, d) = wide_builder_without_registries();
    let table = element(&mut b, &d, "table", &[]);
    let raw = one_padded(units);
    match rule {
        HtmlTableSpanRule::CellColspan | HtmlTableSpanRule::CellRowspan => {
            let name = if rule == HtmlTableSpanRule::CellColspan {
                "colspan"
            } else {
                "rowspan"
            };
            let c = cell(&mut b, &d, "td", &[(name, &raw)]);
            let row = element(&mut b, &d, "tr", &[]);
            b.establish_children(&row, &[c]).unwrap();
            b.establish_children(&table, &[row]).unwrap();
        }
        HtmlTableSpanRule::ColSpan => {
            let group = element(&mut b, &d, "colgroup", &[]);
            let col = element(&mut b, &d, "col", &[("span", &raw)]);
            b.establish_children(&group, &[col]).unwrap();
            b.establish_children(&table, &[group]).unwrap();
        }
        HtmlTableSpanRule::ColgroupSpan => {
            let group = element(&mut b, &d, "colgroup", &[("span", &raw)]);
            b.establish_children(&group, &[]).unwrap();
            b.establish_children(&table, &[group]).unwrap();
        }
    }
    (b.finish().unwrap(), table)
}

fn span_usage(rule: HtmlTableSpanRule, units: usize) -> HtmlSemanticUsage {
    let (capture, table) = span_fixture(rule, units);
    query(&capture, &table, limits()).0.unwrap().usage()
}

#[test]
fn span_reads_admit_the_owner_published_bound_for_each_rule() {
    for rule in [
        HtmlTableSpanRule::CellColspan,
        HtmlTableSpanRule::CellRowspan,
        HtmlTableSpanRule::ColSpan,
        HtmlTableSpanRule::ColgroupSpan,
    ] {
        let admitted = span_usage(rule, LONG).work() - span_usage(rule, SHORT).work();
        let published = rule.work_bound(LONG).unwrap() - rule.work_bound(SHORT).unwrap();
        assert_eq!(admitted, published, "{rule:?}");
    }
}

#[test]
fn scope_reads_admit_the_owner_published_bound_once_per_consulted_header() {
    // Every one of the 2n - 1 headers is reached by another cell's scan. Lengths
    // 3 and 8 are the keyword lengths, where the owner's bound counts unit
    // comparisons: 9 and 14 admitted reads, a difference of five per header.
    let n = 4;
    let consulted = 2 * n - 1;
    let admitted = usage(n, 8).work() - usage(n, 3).work();
    assert_eq!(
        admitted,
        consulted * (th_scope_work_bound(8).unwrap() - th_scope_work_bound(3).unwrap())
    );
    assert_eq!(admitted, consulted * 5);
}

#[test]
fn scope_reads_of_values_longer_than_any_keyword_cost_the_constant() {
    // No keyword is that long, so no unit is ever compared: the owner admits
    // five reads however long the value, and so does the query.
    assert_eq!(th_scope_work_bound(LONG), th_scope_work_bound(4));
    assert_eq!(usage(4, LONG).work(), usage(4, 4).work());
    assert_eq!(usage(4, 100).work(), usage(4, 4).work());
}

#[test]
fn raw_value_length_changes_neither_allocated_bytes_nor_retained_records() {
    // Reading a value is allocation-free for the session: the same table with
    // a long and a short value of the same span retains exactly the same bytes
    // and records, which are those of a table that carries no attribute.
    let (capture, table) = fixture(4, 0);
    let baseline = query(&capture, &table, limits()).0.unwrap().usage();
    for rule in [
        HtmlTableSpanRule::CellColspan,
        HtmlTableSpanRule::CellRowspan,
        HtmlTableSpanRule::ColSpan,
        HtmlTableSpanRule::ColgroupSpan,
    ] {
        let short = span_usage(rule, SHORT);
        let long = span_usage(rule, LONG);
        assert_eq!(long.allocated_bytes(), short.allocated_bytes(), "{rule:?}");
        assert_eq!(long.scratch_records(), short.scratch_records(), "{rule:?}");
        assert_eq!(
            long.nested_output_records(),
            short.nested_output_records(),
            "{rule:?}"
        );
    }
    let scoped_short = usage(4, SHORT);
    let scoped_long = usage(4, LONG);
    assert_eq!(
        scoped_long.allocated_bytes(),
        scoped_short.allocated_bytes()
    );
    assert_eq!(scoped_long.allocated_bytes(), baseline.allocated_bytes());
}

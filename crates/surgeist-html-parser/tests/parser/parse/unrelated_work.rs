//! Per-element parse work depends on what the element touches, not on how many
//! unrelated elements the document holds.
//!
//! The authority is the work model's own contract: every record visit charges
//! one credit, and an element's creation, insertion and association touch the
//! element, its ancestors and the state it keeps, so the visits one more
//! element adds cannot depend on a sibling subtree it has no relation to.
//! Every measure is the visits one extra element adds to a document whose
//! unrelated filler holds `filler` spans: that marginal cost is the same for
//! any filler size, whatever the filler's fanout adds to both documents.

use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseWorkCategory, HtmlSource};

use super::linear_work::roomy_limits;

/// One unrelated element with an attribute and a text node.
const SPAN: &str = "<span class=b>t</span>";

fn record_visits(raw: String) -> u64 {
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), roomy_limits());
    let report = request
        .parse()
        .unwrap_or_else(|_| panic!("the document parses within roomy limits"));
    report.usage().work(HtmlParseWorkCategory::RecordVisit)
}

/// The visits `element` adds to a document whose other section holds `filler`
/// copies of `unit` in an unrelated subtree.
fn visits_added_after(unit: &str, filler: usize, element: &str) -> i64 {
    let page = |inner: &str| {
        format!(
            "<!doctype html><body><div>{}</div><section>{inner}</section>",
            unit.repeat(filler)
        )
    };
    record_visits(page(element)) as i64 - record_visits(page("")) as i64
}

/// Requires the visits `element` adds to be at most 5% (and eight visits) more
/// with 400 unrelated `unit`s than with 50.
fn assert_independent_of_unrelated(unit: &str, element: &str) {
    let few = visits_added_after(unit, 50, element);
    let many = visits_added_after(unit, 400, element);
    assert!(
        many <= few + few / 20 + 8,
        "{element} added {few} record visits after 50 unrelated {unit} but {many} after 400"
    );
}

#[test]
fn an_option_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated(SPAN, "<select><option>x</option></select>");
}

#[test]
fn an_input_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated(SPAN, "<input name=a>");
}

#[test]
fn a_details_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated(SPAN, "<details><summary>x</summary></details>");
}

#[test]
fn an_anchor_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated(SPAN, "<a href=x>t</a>");
}

#[test]
fn an_image_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated(SPAN, "<img src=x>");
}

#[test]
fn a_dialog_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated(SPAN, "<dialog></dialog>");
}

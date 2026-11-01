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

/// The visits the last of `options` copies of `option` adds inside `container`.
fn visits_added_by_last_option(container: &str, option: &str, options: usize) -> i64 {
    let page = |count: usize| {
        format!(
            "<!doctype html><body><{container}>{}</{container}>",
            option.repeat(count)
        )
    };
    record_visits(page(options)) as i64 - record_visits(page(options - 1)) as i64
}

/// Requires a select to add at most 5% (and eight visits) more per option than
/// the same options in a div add, with 400 options as with 50. The div pays the
/// same insertion, creation and registry costs but runs no selectedness
/// setting, so the difference is the select algorithm's own work.
fn assert_selectedness_work_independent_of_other_options(option: &str) {
    let excess = |options: usize| {
        visits_added_by_last_option("select", option, options)
            - visits_added_by_last_option("div", option, options)
    };
    let (few, many) = (excess(50), excess(400));
    assert!(
        many <= few + few.abs() / 20 + 8,
        "{option} in a select added {few} visits over a div after 50 options but {many} after 400"
    );
}

#[test]
fn an_option_adds_the_same_selectedness_visits_whatever_other_options_the_select_holds() {
    assert_selectedness_work_independent_of_other_options("<option>x</option>");
}

#[test]
fn a_selected_option_adds_the_same_selectedness_visits_whatever_other_options_the_select_holds() {
    assert_selectedness_work_independent_of_other_options("<option selected>x</option>");
}

#[test]
fn a_disabled_option_adds_the_same_selectedness_visits_whatever_other_options_the_select_holds() {
    assert_selectedness_work_independent_of_other_options("<option disabled>x</option>");
}

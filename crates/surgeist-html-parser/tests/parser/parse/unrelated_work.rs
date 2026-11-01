//! Per-element parse work depends on what the element touches, not on how many
//! unrelated elements the document holds.
//!
//! The authority is the work model's own contract: every record visit charges
//! one credit, and an element's creation, insertion and association touch the
//! element, its ancestors and the state it keeps, so the visits one more
//! element adds cannot depend on a sibling subtree it has no relation to.
//! Every measure is the visits one extra element adds to a document whose
//! unrelated filler holds `filler` elements: that marginal cost is the same for
//! any filler size, whatever the filler's fanout adds to both documents. It is
//! exactly equal for most elements; `POOL_PROBE_SLACK` explains the rest.

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

/// The filler sizes every measure is taken at.
const SIZES: [usize; 3] = [50, 200, 800];

/// Requires the measures taken at `SIZES` to differ by at most `slack`.
fn assert_flat(what: &str, unit: &str, measures: [i64; 3], slack: i64) {
    let (least, most) = (
        measures.iter().min().copied().unwrap_or(0),
        measures.iter().max().copied().unwrap_or(0),
    );
    assert!(
        most - least <= slack,
        "{what} added {measures:?} record visits after {SIZES:?} unrelated {unit}"
    );
}

/// Requires the visits `element` adds to be the same with 50, 200 and 800
/// unrelated `unit`s, within `slack` visits.
fn assert_independent_of_unrelated_within(unit: &str, element: &str, slack: i64) {
    let measures = SIZES.map(|filler| visits_added_after(unit, filler, element));
    assert_flat(element, unit, measures, slack);
}

/// Slack for an element whose strings are new to the value pool the final
/// sharing pass probes. The pool is a fixed table (surgeist-html
/// `values/sharing/policy.rs`), so a probe's visits are bounded, but they
/// depend on the occupancy the filler's pooled values leave in its bucket:
/// how many of the four candidates it inspects, whether a matching hash is
/// upgraded and compared, how many hot slots it scans before a free one, and
/// whether that slot's page is already allocated. This value is not derived
/// from those terms: the largest spread seen for an anchor or a style element
/// over filler sizes 1 to 130, 190 to 210 and 790 to 810 was 29 visits.
const POOL_PROBE_SLACK: i64 = 32;

fn assert_independent_of_unrelated(unit: &str, element: &str) {
    assert_independent_of_unrelated_within(unit, element, 0);
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
    assert_independent_of_unrelated_within(SPAN, "<a href=x>t</a>", POOL_PROBE_SLACK);
}

#[test]
fn an_image_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated(SPAN, "<img src=x>");
}

#[test]
fn a_dialog_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated(SPAN, "<dialog></dialog>");
}

#[test]
fn a_style_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated_within(SPAN, "<style>a{}</style>", POOL_PROBE_SLACK);
}

#[test]
fn a_style_adds_the_same_visits_whatever_other_style_elements_precede_it() {
    // The sheet order the style records names the earlier style elements, but
    // one more entry is all that a later style adds to it.
    assert_independent_of_unrelated_within(
        "<style>b{}</style>",
        "<style>a{}</style>",
        POOL_PROBE_SLACK,
    );
}

/// One form control that registers a form association.
const CONTROL: &str = "<input name=a>";

#[test]
fn a_paragraph_adds_the_same_visits_whatever_form_controls_precede_it() {
    assert_independent_of_unrelated(CONTROL, "<p>x</p>");
}

#[test]
fn a_division_adds_the_same_visits_whatever_form_controls_precede_it() {
    assert_independent_of_unrelated(CONTROL, "<div class=a><span>x</span></div>");
}

#[test]
fn an_input_adds_the_same_visits_whatever_form_controls_precede_it() {
    assert_independent_of_unrelated(CONTROL, "<input name=b>");
}

#[test]
fn a_form_adds_the_same_visits_whatever_form_controls_precede_it() {
    assert_independent_of_unrelated(CONTROL, "<form><input name=b></form>");
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

/// Requires a select to add the same visits per option as the same options in
/// a div add, with 50, 200 and 800 options before it. The div pays the same
/// insertion, creation and registry costs but runs no selectedness setting, so
/// the difference is the select algorithm's own work.
fn assert_selectedness_work_independent_of_other_options(option: &str) {
    let excess = |options: usize| {
        visits_added_by_last_option("select", option, options)
            - visits_added_by_last_option("div", option, options)
    };
    assert_flat(
        &format!("{option} in a select over a div"),
        "options",
        SIZES.map(excess),
        0,
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

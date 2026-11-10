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

use surgeist_html::{
    construction::HtmlDocumentMode,
    parsing::HtmlParserScriptingMode,
    security::nonce::{
        HtmlBrowsingContextPresence, HtmlHeaderDeliveredCsp, HtmlNonceDocumentEnvironment,
    },
    values::{Namespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseWorkCategory, HtmlSource,
};
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCaptureTemplateCacheInput,
    HtmlCapturedIsValue, HtmlDocumentClass, HtmlFragmentContext, HtmlHostFactsRevision,
    HtmlScriptingStatus, HtmlTemplateDocumentRole,
};

use super::linear_work::roomy_limits;

/// One unrelated element with an attribute and a text node.
const SPAN: &str = "<span class=b>t</span>";

fn work(raw: String, category: HtmlParseWorkCategory) -> u64 {
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), roomy_limits());
    let report = request
        .parse()
        .unwrap_or_else(|_| panic!("the document parses within roomy limits"));
    report.usage().work(category)
}

fn record_visits(raw: String) -> u64 {
    work(raw, HtmlParseWorkCategory::RecordVisit)
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
    assert_flat_of("record visits", what, unit, measures, slack);
}

/// As `assert_flat`, naming the `measured` quantity in the failure.
fn assert_flat_of(measured: &str, what: &str, unit: &str, measures: [i64; 3], slack: i64) {
    let (least, most) = (
        measures.iter().min().copied().unwrap_or(0),
        measures.iter().max().copied().unwrap_or(0),
    );
    assert!(
        most - least <= slack,
        "{what} added {measures:?} {measured} after {SIZES:?} unrelated {unit}"
    );
}

/// Requires the visits `element` adds to be the same with 50, 200 and 800
/// unrelated `unit`s, within `slack` visits.
fn assert_independent_of_unrelated_within(unit: &str, element: &str, slack: i64) {
    let measures = SIZES.map(|filler| visits_added_after(unit, filler, element));
    assert_flat(element, unit, measures, slack);
}

/// Slack, in visits, for an element whose strings are new to the parse. The value is not derived
/// from the terms of those lookups.
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

#[test]
fn a_meta_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    // Only a meta named theme-color or color-scheme can change which one the
    // document selects, so a charset declaration changes nothing to recompute.
    assert_independent_of_unrelated(SPAN, "<meta charset=x>");
}

#[test]
fn a_viewport_meta_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    // Its name and content attributes are each written to the element before
    // it is inserted anywhere, which no recomputation of the selection sees.
    assert_independent_of_unrelated(SPAN, "<meta name=viewport content=x>");
}

#[test]
fn a_theme_color_meta_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    // It is a candidate for the theme color, so its insertion settles the
    // selection, and that takes reading this meta, not walking the tree.
    assert_independent_of_unrelated_within(
        SPAN,
        "<meta name=theme-color content=red>",
        POOL_PROBE_SLACK,
    );
}

#[test]
fn a_color_scheme_meta_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    assert_independent_of_unrelated_within(
        SPAN,
        "<meta name=color-scheme content=dark>",
        POOL_PROBE_SLACK,
    );
}

/// One form control that registers a form association.
const CONTROL: &str = "<input name=a>";

#[test]
fn an_element_with_an_id_adds_the_same_visits_whatever_form_controls_precede_it() {
    // An id can resolve the form attribute of a control, so inserting one runs
    // the id pass; none of these controls has a form attribute to resolve.
    assert_independent_of_unrelated(CONTROL, "<div id=a>x</div>");
}

#[test]
fn an_input_with_an_id_adds_the_same_visits_whatever_form_controls_precede_it() {
    assert_independent_of_unrelated(CONTROL, "<input id=b name=b>");
}

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

/// The work `category` adds when `element` follows `depth` copies of `open`,
/// all left open: the element's parent has `depth` ancestors that are open
/// elements and nothing else between.
fn work_added_at_depth(
    category: HtmlParseWorkCategory,
    open: &str,
    depth: usize,
    element: &str,
) -> i64 {
    let page = |inner: &str| format!("<!doctype html><body>{}{inner}", open.repeat(depth));
    work(page(element), category) as i64 - work(page(""), category) as i64
}

/// Requires `element` to add the same state steps after 50, 200 and 800 open
/// copies of `open`. A state step is what the tree builder charges for each
/// stack entry it examines, so a flat measure means the element's insertion
/// examines no entry beyond the top of the stack of open elements. The check
/// is exact: a start tag that has no scope or duplicate check to run adds a
/// fixed number of steps.
fn assert_state_steps_independent_of_depth(open: &str, element: &str) {
    let measures = SIZES
        .map(|depth| work_added_at_depth(HtmlParseWorkCategory::StateStep, open, depth, element));
    assert_flat_of("state steps", element, open, measures, 0);
}

#[test]
fn a_foreign_element_adds_the_same_state_steps_whatever_elements_are_open_around_it() {
    assert_state_steps_independent_of_depth("<svg>", "<g>");
}

#[test]
fn a_template_adds_the_same_state_steps_whatever_elements_are_open_around_it() {
    // Closing the template leaves the stack as it was, whatever lies below: the
    // template that was last before it is read, not searched for.
    assert_state_steps_independent_of_depth("<div>", "<template></template>");
}

#[test]
fn a_template_adds_the_same_state_steps_whatever_separates_it_from_an_open_template() {
    let measures = SIZES.map(|depth| {
        let page = |inner: &str| {
            format!(
                "<!doctype html><body><template>{}{inner}",
                "<div>".repeat(depth)
            )
        };
        work(
            page("<template></template>"),
            HtmlParseWorkCategory::StateStep,
        ) as i64
            - work(page(""), HtmlParseWorkCategory::StateStep) as i64
    });
    assert_flat_of(
        "state steps",
        "<template></template>",
        "<div> in an open template",
        measures,
        0,
    );
}

#[test]
fn an_html_element_adds_the_same_state_steps_whatever_elements_are_open_around_it() {
    assert_state_steps_independent_of_depth("<span>", "<span>");
}

/// Requires `element` to add the same record visits over what `control` adds,
/// after 50, 200 and 800 open copies of `open`. Both are created and inserted
/// below the same open elements, so every cost they share, including the
/// walks the construction owner makes from the new node's parent toward the
/// root, cancels; what remains is the work only `element`'s creation does.
fn assert_creation_excess_independent_of_depth(open: &str, element: &str, control: &str) {
    let excess = |depth| {
        let visits =
            |element| work_added_at_depth(HtmlParseWorkCategory::RecordVisit, open, depth, element);
        visits(element) - visits(control)
    };
    assert_flat_of(
        &format!("record visits over {control}"),
        element,
        open,
        SIZES.map(excess),
        0,
    );
}

#[test]
fn an_html_element_adds_no_more_visits_than_a_foreign_one_whatever_is_open_around_it() {
    // An HTML element's creation asks whether a template is open to decide
    // whether the form pointer applies; a foreign element's does not.
    assert_creation_excess_independent_of_depth("<span>", "<span>", "<svg>");
}

/// Requires `element` to add the same record visits after 50, 200 and 800 open
/// copies of `open`. The pinned algorithms ask nothing of the ancestors beyond
/// the element's parent for the elements and nodes this is used with (a start
/// tag with no scope check, text, a comment), so a flat measure means creating
/// and inserting the node never climbs the ancestors toward the root.
fn assert_visits_independent_of_depth(open: &str, element: &str) {
    let measures = SIZES
        .map(|depth| work_added_at_depth(HtmlParseWorkCategory::RecordVisit, open, depth, element));
    assert_flat_of("record visits", element, open, measures, 0);
}

#[test]
fn an_html_element_adds_the_same_visits_whatever_elements_are_open_around_it() {
    assert_visits_independent_of_depth("<span>", "<span>");
}

#[test]
fn a_foreign_element_adds_the_same_visits_whatever_elements_are_open_around_it() {
    assert_visits_independent_of_depth("<svg>", "<g>");
    assert_visits_independent_of_depth("<math>", "<mi>");
}

#[test]
fn text_adds_the_same_visits_whatever_elements_are_open_around_it() {
    assert_visits_independent_of_depth("<span>", "x");
}

#[test]
fn a_comment_adds_the_same_visits_whatever_elements_are_open_around_it() {
    assert_visits_independent_of_depth("<span>", "<!--c-->");
}

/// The record visits `element` adds inside a table cell that follows `depth`
/// open spans. The table cell is a scope boundary, so a check the pinned text
/// makes with "in scope" ends at the cell and every walk that remains reaches
/// beyond it only because the algorithm asks about an ancestor.
///
/// Outside a cell the per-level cost of the start tags below is the scope
/// check the pinned text makes, which no cached state replaces:
/// "has a `p` element in button scope" for `details`, `summary` and `form`
/// (`references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md`
/// lines 4368-4370 and 4392-4398), and "has a `select` element in scope" for
/// `input`, `select` and `option` (lines 4664, 4750 and 4768). An `input` or a
/// `select` also resets its form owner, which asks for its nearest `form`
/// ancestor (lines 10831 and 10841 of
/// `references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md`),
/// so a walk up the ancestors is the algorithm's own.
fn visits_added_in_a_cell_at_depth(depth: usize, prefix: &str, element: &str) -> i64 {
    let page = |inner: &str| {
        format!(
            "<!doctype html><body>{}{prefix}<table><tr><td>{inner}",
            "<span>".repeat(depth)
        )
    };
    work(page(element), HtmlParseWorkCategory::RecordVisit) as i64
        - work(page(""), HtmlParseWorkCategory::RecordVisit) as i64
}

/// Requires `element` to add the same record visits in a cell after 50, 200
/// and 800 open spans, with `prefix` opened between the spans and the cell.
fn assert_cell_work_independent_of_depth(prefix: &str, element: &str) {
    let measures = SIZES.map(|depth| visits_added_in_a_cell_at_depth(depth, prefix, element));
    assert_flat_of(
        "record visits in a table cell",
        element,
        "<span>",
        measures,
        0,
    );
}

#[test]
fn a_form_adds_the_same_visits_whatever_elements_are_open_around_its_cell() {
    // Its end tag removes the form from the stack of open elements, which is
    // the current node: finding it needs no look at the entries below.
    assert_cell_work_independent_of_depth("", "<form></form>");
}

#[test]
fn an_input_adds_the_same_visits_whatever_elements_are_open_around_its_cell() {
    // "Has a select element in scope" ends at the cell; the nearest form
    // ancestor is the form opened inside the spans.
    assert_cell_work_independent_of_depth("<form>", "<input name=a>");
}

#[test]
fn a_select_with_an_option_adds_the_same_visits_whatever_is_open_around_its_cell() {
    assert_cell_work_independent_of_depth("<form>", "<select><option>x</option></select>");
}

#[test]
fn a_details_adds_the_same_visits_whatever_elements_are_open_around_its_cell() {
    assert_cell_work_independent_of_depth("", "<details><summary>x</summary></details>");
}

/// The visits `element` adds to a document in which `filler` copies of `unit`
/// precede it, so that the difference is the work of the one element.
fn visits_added_by_selected_option_after(unit: &str, count: usize) -> i64 {
    let page = |options: usize| {
        format!(
            "<!doctype html><body><div>{}</div><select><selectedcontent></selectedcontent>{}</select>",
            unit.repeat(count),
            "<option selected>x</option>".repeat(options)
        )
    };
    record_visits(page(2)) as i64 - record_visits(page(1)) as i64
}

#[test]
fn a_selected_option_adds_the_same_selectedcontent_visits_whatever_unrelated_elements_precede_it() {
    // Updating the select's selectedcontent reads the select's descendants and
    // its options; nothing outside the select is read.
    let measures = SIZES.map(|count| visits_added_by_selected_option_after(SPAN, count));
    assert_flat_of(
        "record visits",
        "a second selected option",
        SPAN,
        measures,
        POOL_PROBE_SLACK,
    );
}

#[test]
fn a_style_inside_a_fostered_element_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    // The element is fostered before its table, so the style is not the last
    // node of the tree; it still follows every style before it.
    assert_independent_of_unrelated_within(
        SPAN,
        "<table><b><style>a{}</style></b></table>",
        POOL_PROBE_SLACK,
    );
}

/// A `div` in a connected document that has a header-delivered policy, the
/// context in which a nonce is hidden from the tree once its element is inserted.
fn connected_div_context() -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    builder
        .establish_document_nonce_environment(
            &document,
            HtmlNonceDocumentEnvironment::new(
                HtmlBrowsingContextPresence::Present,
                HtmlHeaderDeliveredCsp::Present,
            ),
        )
        .unwrap();
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_children(&document, std::slice::from_ref(&div))
        .unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap()
}

fn connected_fragment_visits(raw: String) -> i64 {
    let request = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8(raw),
        connected_div_context(),
        roomy_limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap();
    let report = request
        .parse()
        .unwrap_or_else(|_| panic!("the fragment parses within roomy limits"));
    report.usage().work(HtmlParseWorkCategory::RecordVisit) as i64
}

#[test]
fn a_hidden_nonce_adds_the_same_visits_whatever_unrelated_elements_precede_it() {
    // The nonce of an element in a shadow root of a connected host is hidden
    // when the element is inserted. Both elements are the same but for that
    // attribute, so the difference between them is the work of hiding it.
    const HIDDEN: &str = "<div><template shadowrootmode=open><p nonce=A></p></template></div>";
    const PLAIN: &str = "<div><template shadowrootmode=open><p></p></template></div>";
    let excess = |filler: usize| {
        let page = |element: &str| format!("{}{element}", SPAN.repeat(filler));
        let added = |element: &str| {
            connected_fragment_visits(page(element)) - connected_fragment_visits(page(""))
        };
        added(HIDDEN) - added(PLAIN)
    };
    assert_flat_of(
        "record visits over an element without a nonce",
        "a hidden nonce",
        SPAN,
        SIZES.map(excess),
        POOL_PROBE_SLACK,
    );
}

/// The record visits `inner` adds to a document whose head already holds a
/// deciding element, after `depth` open spans in the body.
fn visits_added_below_a_first(first: &str, depth: usize, cell: &str, inner: &str) -> i64 {
    let page = |inner: &str| {
        format!(
            "<!doctype html><head>{first}</head><body>{}{cell}{inner}",
            "<span>".repeat(depth)
        )
    };
    record_visits(page(inner)) as i64 - record_visits(page("")) as i64
}

/// Requires the visits `inner` adds to grow with the open elements around it by
/// at most `PER_LEVEL` for each one, exactly as far as the order comparison needs.
///
/// Placing a meta that may precede the deciding meta, or a style that may follow
/// the last listed style, asks which of two elements comes first in tree order.
/// The pinned algorithms define that order: the selection of a theme color and
/// of a supported color scheme take their candidates "in tree order" (lines
/// 1834 and 1876 of
/// `references/html-language-elements-attributes--877d6146487f--2ca83b5ac984.md`),
/// and two elements in different branches are ordered by their lowest common
/// ancestor, so every level between the deeper element and that ancestor is
/// looked at. The walk the pinned text describes looks at all of them and more;
/// this comparison looks at each once.
fn assert_order_comparison_costs_one_visit_per_level(first: &str, cell: &str, inner: &str) {
    const PER_LEVEL: i64 = 1;
    let depths = [50usize, 200, 800];
    let measures = depths.map(|depth| visits_added_below_a_first(first, depth, cell, inner));
    for (pair, window) in depths.windows(2).zip(measures.windows(2)) {
        let levels = (pair[1] - pair[0]) as i64;
        let grew = window[1] - window[0];
        assert!(
            grew <= PER_LEVEL * levels + POOL_PROBE_SLACK,
            "{inner} added {measures:?} record visits after {depths:?} open spans: {grew} more \
             for {levels} more levels"
        );
    }
}

#[test]
fn a_second_theme_color_meta_costs_one_visit_per_open_level_to_order() {
    assert_order_comparison_costs_one_visit_per_level(
        "<meta name=theme-color content=red>",
        "",
        "<meta name=theme-color content=blue>",
    );
}

#[test]
fn a_second_style_costs_one_visit_per_open_level_to_order() {
    assert_order_comparison_costs_one_visit_per_level(
        "<style>a{}</style>",
        "",
        "<style>b{}</style>",
    );
}

#[test]
fn a_second_fostered_style_costs_one_visit_per_open_level_to_order() {
    assert_order_comparison_costs_one_visit_per_level(
        "<style>a{}</style>",
        // A table cell ends the scope checks of the table start tag and of the
        // fostered element, which would otherwise add their own level costs.
        "<table><tr><td>",
        "<table><b><style>b{}</style></b></table>",
    );
}

#[test]
fn a_second_meta_in_a_nearby_branch_costs_nothing_for_the_open_levels_above_it() {
    // The deciding meta and the second one lie in sibling branches of the last
    // of `depth` open divs. Their lowest common ancestor is two levels above
    // each, so ordering them looks at those levels and no other, however deep
    // the divs are. (The `p` and `section` start tags make their own scope
    // checks, which both documents share.)
    let depths = [50usize, 200, 800];
    let visits = |depth: usize| {
        let page = |inner: &str| {
            format!(
                "<!doctype html><body>{}<p><meta name=theme-color content=red></p><section>{inner}",
                "<div>".repeat(depth)
            )
        };
        record_visits(page("<meta name=theme-color content=blue>")) as i64
            - record_visits(page("")) as i64
    };
    let measures = depths.map(visits);
    assert!(
        measures[2] - measures[0] <= POOL_PROBE_SLACK,
        "a second meta added {measures:?} record visits after {depths:?} open divs"
    );
}

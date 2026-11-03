//! Parse work is linear in document size.
//!
//! The authority is the work model's own contract: every unit charges one
//! credit and a document of N repeated elements must cost O(N) credits, so an
//! exact (non-geometric) reservation that charges the whole old plus new vector
//! on every element violates it.

use surgeist_html::{
    construction::HtmlDocumentMode,
    parsing::HtmlParserScriptingMode,
    values::{Namespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseLimits, HtmlParseResource,
    HtmlParseWorkCategory, HtmlSource,
};
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCaptureTemplateCacheInput,
    HtmlCapturedIsValue, HtmlDocumentClass, HtmlFragmentContext, HtmlHostFactsRevision,
    HtmlScriptingStatus, HtmlTemplateDocumentRole,
};

/// One element subtree with attributes, nested elements and two text nodes.
const SUBTREE: &str = "<div id=a class=b><p>t</p><span>x</span></div>";

/// `Work` is charged to one parse: total credits, bytes reserved, bytes copied,
/// bytes inspected and records visited.
#[derive(Clone, Copy)]
struct Work {
    total: u64,
    reserved: u64,
    copied: u64,
    inspected: u64,
    visited: u64,
}

/// A named measure of one parse's work.
type Measure = (&'static str, fn(&Work) -> u64);

/// Limits that no measured document reaches, so only the work model differs
/// between sizes.
pub(super) fn roomy_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(1_000_000)
        .buffered_utf16_units(1_000_000)
        .token_attributes(1_000_000)
        .auxiliary_entries(100_000_000)
        .managed_nodes(1_000_000)
        .managed_edges(1_000_000)
        .managed_attributes(1_000_000)
        .managed_utf16_units(1_000_000)
        .source_map_entries(1_000_000)
        .source_map_payload_units(1_000_000)
        .diagnostics(1_000_000)
        .diagnostic_payload_units(1_000_000)
        .context_effects(1_000_000)
        .open_elements(1_000)
        .tree_depth(1_000_000)
        .managed_capacity_bytes(4_000_000_000)
        .peak_transient_bytes(4_000_000_000)
        .atomic_layout_bytes(4_000_000_000)
        .work_credits(100_000_000_000)
        .finish()
        .unwrap()
}

/// `roomy_limits` with room for the stack of open elements that nested shadow
/// roots fill: each layer keeps its host and its template open.
fn deep_limits() -> HtmlParseLimits {
    roomy_limits()
        .into_builder()
        .open_elements(5_000)
        .finish()
        .unwrap()
}

fn parse_work(raw: String) -> Work {
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), roomy_limits());
    let report = request
        .parse()
        .unwrap_or_else(|_| panic!("the document parses within roomy limits"));
    work_of(report.usage())
}

/// A `div` captured as the context element of a fragment parse.
fn div_context() -> HtmlFragmentContext {
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
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&div).unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap()
}

/// Parses `raw` as a fragment that allows declarative shadow roots.
fn parse_fragment_work(raw: String) -> Work {
    let request = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8(raw),
        div_context(),
        deep_limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap();
    let report = request
        .parse()
        .unwrap_or_else(|_| panic!("the fragment parses within roomy limits"));
    work_of(report.usage())
}

fn work_of(usage: &surgeist_html_parser::HtmlParseUsage) -> Work {
    Work {
        total: usage.resource(HtmlParseResource::WorkCredits).current(),
        reserved: usage.work(HtmlParseWorkCategory::ReserveByte),
        copied: usage.work(HtmlParseWorkCategory::CopyByte),
        inspected: usage.work(HtmlParseWorkCategory::InspectByte),
        visited: usage.work(HtmlParseWorkCategory::RecordVisit),
    }
}

/// Requires the work each doubling of `sizes` adds to be at most `BOUND` times
/// what the previous doubling added, for every measure in `measures`.
///
/// Let D(n) = W(2n) - W(n) be the work a doubling adds. Linear work
/// W(n) = a*n + b gives D(n) = a*n, so D(2n) = 2*D(n) whatever the constant b
/// is; quadratic work W(n) = c*n^2 gives D(2n) = 4*D(n). Three is the
/// midpoint, so a quadratic term is rejected once it dominates the growth
/// while geometric capacity steps, which vary the ratio around two, and the
/// child-order walks that grow with one parent's fanout, a small share of the
/// work, are tolerated.
fn assert_doublings_add_near_double(
    sizes: [usize; 4],
    document: impl Fn(usize) -> String,
    measures: &[Measure],
) {
    assert_parses_add_near_double(sizes, document, parse_work, measures);
}

/// As `assert_doublings_add_near_double`, with the parse that measures each size.
fn assert_parses_add_near_double(
    sizes: [usize; 4],
    document: impl Fn(usize) -> String,
    parse: impl Fn(String) -> Work,
    measures: &[Measure],
) {
    const BOUND: u64 = 3;
    let measured: Vec<Work> = sizes.iter().map(|&n| parse(document(n))).collect();
    for (name, pick) in measures {
        let work: Vec<u64> = measured.iter().map(pick).collect();
        for (pair, window) in sizes.windows(3).zip(work.windows(3)) {
            let (first, second) = (window[1] - window[0], window[2] - window[1]);
            assert!(
                second <= BOUND * first,
                "{name} grew faster than linearly: doubling {} -> {} repeats added {first}, \
                 doubling {} -> {} repeats added {second}",
                pair[0],
                pair[1],
                pair[1],
                pair[2],
            );
        }
    }
}

#[test]
fn work_added_by_each_doubling_of_the_document_is_near_double_the_previous() {
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |repeats| format!("<!doctype html>{}", SUBTREE.repeat(repeats)),
        &[
            ("total work", |work| work.total),
            ("reservation work", |work| work.reserved),
        ],
    );
}

#[test]
fn reservation_work_added_by_each_doubling_of_one_elements_attributes_is_near_double() {
    // Larger counts than the other documents: the attribute vector's own share
    // of the reserved bytes is small until the count is in the hundreds.
    assert_doublings_add_near_double(
        [200, 400, 800, 1600],
        |attributes| {
            let names: String = (0..attributes).map(|i| format!(" a{i}=v")).collect();
            format!("<!doctype html><div{names}></div>")
        },
        &[("reservation work", |work| work.reserved)],
    );
}

#[test]
fn reservation_work_added_by_each_doubling_of_merged_attributes_beyond_creating_them_is_near_double()
 {
    // A second `html` start tag merges its attributes into the first element one candidate at a
    // time ("in body" insertion mode, `html` start tag, retained HTML Standard capture
    // `html-language-syntax-parser--877d6146487f--37a9ba51eed8.md`, line 4306). The plan that
    // records each candidate's decision grows by one slot per candidate unless it grows
    // geometrically, so its reservation work is quadratic. The same attributes on a new element
    // are tokenized and installed without a plan, so the reservation work that differs from that
    // baseline, `E(n)`, is the merge's own (the two paths also keep tables of the names, which
    // are linear in `n` and may reserve more or less than each other, so the gap is measured
    // without its sign).
    //
    // As in `assert_doublings_add_near_double`, let D(n) = E(2n) - E(n). Linear work
    // E(n) = a*n + b gives D(n) = a*n, so D(2n) = 2*D(n) whatever the constant b is; quadratic
    // work gives D(2n) = 4*D(n). Three is the midpoint. Differences, unlike ratios of E, are not
    // moved by the constant term, which is as large as the linear term at these sizes.
    let extra = |attributes: usize| {
        let names: String = (0..attributes).map(|i| format!(" a{i}=v")).collect();
        let merged = parse_work(format!("<!doctype html><html><body><html{names}>")).reserved;
        let created = parse_work(format!("<!doctype html><html><body><div{names}>")).reserved;
        // Each path keeps its own table of the attribute names, so neither reserves a bound on the
        // other; the plan's growth is the part of the gap that is not linear.
        merged.abs_diff(created)
    };
    let sizes = [100_usize, 200, 400, 800];
    let measured: Vec<u64> = sizes.iter().map(|&n| extra(n)).collect();
    for (pair, window) in sizes.windows(3).zip(measured.windows(3)) {
        let (first, second) = (window[1] - window[0], window[2] - window[1]);
        assert!(
            second <= 3 * first,
            "merge reservation work grew faster than linearly: doubling {} -> {} attributes \
             added {first} bytes beyond creating them, doubling {} -> {} added {second}",
            pair[0],
            pair[1],
            pair[1],
            pair[2],
        );
    }
}

#[test]
fn reservation_work_added_by_each_doubling_of_one_selects_options_is_near_double() {
    // An option inserted last settles from the select's summary of its list, so
    // the list is not collected per insertion and total work is linear too.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |options| {
            format!(
                "<!doctype html><select>{}</select>",
                "<option>x</option>".repeat(options)
            )
        },
        &[
            ("total work", |work| work.total),
            ("reservation work", |work| work.reserved),
        ],
    );
}

#[test]
fn reservation_work_added_by_each_doubling_of_a_documents_style_elements_is_near_double() {
    // The style elements of the tree share one order row that each style at the
    // end of the tree extends by one entry, so total work is linear and the
    // row must grow geometrically.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |styles| {
            format!(
                "<!doctype html><head>{}",
                "<style>a{}</style>".repeat(styles)
            )
        },
        &[
            ("total work", |work| work.total),
            ("reservation work", |work| work.reserved),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_one_parents_children_is_near_double_the_previous() {
    // Every comment is appended under the same body, so appending a child must
    // cost the same however many children the body already has.
    assert_doublings_add_near_double(
        [400, 800, 1600, 3200],
        |children| format!("<!doctype html><html><body>{}", "<!--c-->".repeat(children)),
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
            ("reservation work", |work| work.reserved),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_repeated_adoption_agency_misnesting_is_near_double() {
    // Each `</b>` finds a formatting element with a furthest block, so the
    // adoption agency algorithm moves a subtree. Once any subtree has moved,
    // a later insertion must not recompute every record's depth.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |repeats| format!("<!doctype html>{}", "<b><p><i>x</b>".repeat(repeats)),
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_sibling_formatting_elements_is_near_double() {
    // The control for the misnesting document: well-nested formatting elements
    // never move a subtree.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |repeats| format!("<!doctype html>{}", "<b>x</b>".repeat(repeats)),
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_repeated_foster_parented_text_is_near_double() {
    // Each text inside a table is foster parented: it is inserted before the
    // table, which sits last under the growing body, so inserting before a
    // sibling must cost what shifting the entries behind it costs, never the
    // body's whole child order.
    assert_doublings_add_near_double(
        [200, 400, 800, 1600],
        |repeats| {
            format!(
                "<!doctype html><body>{}",
                "<table>x</table>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
            ("reservation work", |work| work.reserved),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_repeated_tables_with_cells_is_near_double() {
    // The control for foster parenting: nothing is inserted before a sibling.
    assert_doublings_add_near_double(
        [200, 400, 800, 1600],
        |repeats| {
            format!(
                "<!doctype html><body>{}",
                "<table><tr><td>x</td></tr></table>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
            ("reservation work", |work| work.reserved),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_declarative_shadow_roots_is_near_double() {
    // Each template becomes the shadow root of its parent div: the parser gives
    // the host a shadow root and releases the template's contents.
    assert_parses_add_near_double(
        [50, 100, 200, 400],
        |repeats| "<div><template shadowrootmode=open><p>x</p></template></div>".repeat(repeats),
        parse_fragment_work,
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_inputs_with_ids_is_near_double() {
    // Each input with an id runs the id pass over the form controls; none has
    // a form attribute, so the pass must find nothing to visit.
    assert_doublings_add_near_double(
        [200, 400, 800, 1600],
        |repeats| {
            format!(
                "<!doctype html><body>{}",
                "<input id=a name=a>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_meta_elements_is_near_double() {
    // None of these is a theme-color or color-scheme candidate, so inserting
    // one never recomputes the document's selected theme color or scheme.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |repeats| {
            format!(
                "<!doctype html><head>{}",
                "<meta name=viewport content=x>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_unselectable_theme_color_metas_is_near_double() {
    // Each of these is a candidate for the theme color: the exact name and a
    // content attribute. None selects anything, since `bogus` is no color, so
    // the selection is "none" after every one. Placing a candidate must cost
    // what it takes to read that candidate, whatever candidates precede it.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |repeats| {
            format!(
                "<!doctype html><head>{}",
                "<meta name=theme-color content=bogus>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_unselectable_color_scheme_metas_is_near_double() {
    // `1` is no color-scheme value, so every candidate is read and none selected.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |repeats| {
            format!(
                "<!doctype html><head>{}",
                "<meta name=color-scheme content=1>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_metas_behind_a_selected_theme_color_is_near_double() {
    // The first candidate selects red. Every later one follows it in tree
    // order, so it cannot change the selection and needs no reading.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |repeats| {
            format!(
                "<!doctype html><head><meta name=theme-color content=red>{}",
                "<meta name=theme-color content=blue>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_candidate_metas_in_a_moved_subtree_is_near_double() {
    // `</b>` finds the formatting element `b` with the furthest block `p`, and
    // the adoption agency algorithm moves `p` with every meta it holds. Each
    // meta is a candidate for the theme color that selects nothing, and the
    // one move decides the selection once, not once for each candidate.
    assert_doublings_add_near_double(
        [10, 20, 40, 80],
        |repeats| {
            format!(
                "<!doctype html><b><p>{}</b>",
                "<meta name=theme-color content=bogus>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_added_by_each_doubling_of_candidate_metas_in_a_nested_moved_subtree_is_near_double() {
    // The same move after a second formatting element and a block, so that the
    // subtree moved is not the first child of its new parent.
    assert_doublings_add_near_double(
        [10, 20, 40, 80],
        |repeats| {
            format!(
                "<!doctype html><a><b><div>{}</a>",
                "<meta name=theme-color content=bogus>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn work_the_adoption_agency_adds_by_moving_children_at_most_doubles_with_their_number() {
    // HTML 877d 13.2.6.4.7, adoption agency step 18 (line 4952 of
    // `references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md`):
    // "take all of the child nodes of furthestBlock and append them to the
    // element created in the last step". `</b>` finds the formatting element
    // `b` with the furthest block `p`, which holds the n comments, the cheapest
    // nodes, so their number dominates. The same document without `</b>` moves
    // nothing, so the work the move adds is the difference of the two.
    //
    // Moving n children is n moves of constant work each, so the work added is
    // a*n + b with b the fixed cost of the algorithm and no negative term:
    // E(2n) = 2*a*n + b <= 2*E(n). A tenth more is left for the geometric
    // capacity steps of the new parent's child order. Taking each child from
    // the front of an order that shifts every later entry, after finding it by
    // a walk from the end, adds a term quadratic in n: E(2n) approaches
    // 4*E(n).
    const BOUND_TENTHS: u64 = 22;
    let sizes = [100usize, 200, 400, 800];
    let added: Vec<u64> = sizes
        .iter()
        .map(|&n| {
            let comments = "<!--c-->".repeat(n);
            let moved = parse_work(format!("<!doctype html><b><p>{comments}</b>")).visited;
            let unmoved = parse_work(format!("<!doctype html><b><p>{comments}")).visited;
            moved - unmoved
        })
        .collect();
    for (pair, window) in sizes.windows(2).zip(added.windows(2)) {
        assert!(
            window[1] * 10 <= BOUND_TENTHS * window[0],
            "moving {} children added {} record visits and moving {} added {}: the work added \
             grew faster than linearly (all measured: {added:?})",
            pair[0],
            window[0],
            pair[1],
            window[1],
        );
    }
}

#[test]
fn work_added_by_each_doubling_of_one_selects_selected_options_is_near_four_times() {
    // Each selected option updates the select's selectedcontent. The pinned
    // algorithm "update a select's descendant selectedcontent elements" lists
    // the select's descendants in tree order and, for each selectedcontent,
    // takes "the first option in select's list of options whose selectedness
    // is true", a walk of the select's descendants (lines 10645-10660 of
    // `references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md`).
    // It is run by the steps that settle a select (lines 7838, 7930, 7970,
    // 7988, 8258 and 8282 of the same file), so n selected options cost work
    // proportional to the select's size each: quadratically in all. Reading the
    // children of one parent must itself be linear in them, so the quadratic
    // total is the whole growth: a ratio of at most four per doubling.
    const BOUND: u64 = 4;
    let sizes = [50usize, 100, 200, 400];
    let measured: Vec<u64> = sizes
        .iter()
        .map(|&n| {
            parse_work(format!(
                "<!doctype html><select><selectedcontent></selectedcontent>{}",
                "<option selected>x</option>".repeat(n)
            ))
            .visited
        })
        .collect();
    for (pair, window) in sizes.windows(3).zip(measured.windows(3)) {
        let (first, second) = (window[1] - window[0], window[2] - window[1]);
        assert!(
            second <= BOUND * first,
            "record visits grew faster than quadratically: doubling {} -> {} options added \
             {first}, doubling {} -> {} added {second}",
            pair[0],
            pair[1],
            pair[1],
            pair[2],
        );
    }
}

#[test]
fn work_added_by_each_doubling_of_styles_fostered_before_their_table_is_near_double() {
    // Each style sits in an element fostered before the table, so none of them
    // is the last node of the tree, but each follows the styles before it.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |repeats| {
            format!(
                "<!doctype html><body><table>{}",
                "<b><style>a{}</style></b>".repeat(repeats)
            )
        },
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

/// One layer of nested declarative shadow roots: a host whose template becomes
/// its shadow root, and the template and host still open.
const SHADOW_LAYER: &str = "<div><template shadowrootmode=open><p>x</p>";
const SHADOW_LAYER_END: &str = "</template></div>";

fn nested_shadow_roots(layers: usize, innermost: &str) -> String {
    format!(
        "{}{innermost}{}",
        SHADOW_LAYER.repeat(layers),
        SHADOW_LAYER_END.repeat(layers)
    )
}

#[test]
fn work_added_by_each_doubling_of_nested_declarative_shadow_roots_is_near_double() {
    // Each host sits inside the shadow root of the one before it, so the tree
    // and the stack of open elements deepen with every repeat.
    assert_parses_add_near_double(
        [50, 100, 200, 400],
        |repeats| nested_shadow_roots(repeats, ""),
        parse_fragment_work,
        &[
            ("record visits", |work| work.visited),
            ("total work", |work| work.total),
        ],
    );
}

#[test]
fn a_declarative_shadow_root_adds_the_same_visits_whatever_shadow_roots_enclose_it() {
    // The layers deepen both the tree and the stack, and the scope checks the
    // added layer makes end at its own template, so nothing the layer does
    // needs to look beyond the layers around it.
    let added = |layers: usize| {
        let layer = format!("{SHADOW_LAYER}{SHADOW_LAYER_END}");
        parse_fragment_work(nested_shadow_roots(layers, &layer)).visited as i64
            - parse_fragment_work(nested_shadow_roots(layers, "")).visited as i64
    };
    let measures = [25, 100, 400].map(added);
    assert!(
        measures.iter().all(|visits| *visits == measures[0]),
        "one more shadow root added {measures:?} record visits inside 25, 100 and 400"
    );
}

/// `roomy_limits` with room for the stacks, trees and credits that the longest
/// single-token and single-run documents below reach.
fn long_input_limits() -> HtmlParseLimits {
    roomy_limits()
        .into_builder()
        .source_units(10_000_000)
        .buffered_utf16_units(10_000_000)
        .open_elements(100_000)
        .tree_depth(10_000_000)
        .work_credits(500_000_000_000)
        .finish()
        .unwrap()
}

fn parse_long_input_work(raw: String) -> Work {
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), long_input_limits());
    let report = request
        .parse()
        .unwrap_or_else(|_| panic!("the document parses within the long-input limits"));
    work_of(report.usage())
}

/// Requires every measure of the work at a size to be at most 2.2 times its
/// work at the size before, for each of the three doublings of `sizes`.
///
/// A buffer that grows by a constant factor copies each element a constant
/// number of times, so n elements cost `a*n + b` credits with `b` the fixed
/// cost of the document: doubling n at most doubles the credits, and the
/// tenth more allowed here is for capacity steps and the share of the work
/// that is not linear in the size of one run. A buffer that grows by one slot
/// per element charges the whole old and new vector for each: credits `c*n^2`,
/// which quadruple with each doubling.
fn assert_each_doubling_costs_at_most_two_point_two_times(
    sizes: [usize; 4],
    document: impl Fn(usize) -> String,
) {
    assert_measures_at_most_two_point_two_times(sizes, document, &[TOTAL_WORK, RESERVATION_WORK]);
}

const TOTAL_WORK: Measure = ("total work", |work| work.total);
const RESERVATION_WORK: Measure = ("reservation work", |work| work.reserved);
/// Work other than reservation. Reserved bytes step up with each capacity doubling of the many
/// geometric lists a document keeps, which moves the ratio of two sizes by a factor other than two
/// whatever the algorithm; the reservation of attribute lists has its own tests.
const WORK_OUTSIDE_RESERVATION: Measure = ("work outside reservation", |work| {
    work.total.saturating_sub(work.reserved)
});
const COPY_WORK: Measure = ("copy work", |work| work.copied);
const INSPECTION_WORK: Measure = ("inspection work", |work| work.inspected);
const VISIT_WORK: Measure = ("record visits", |work| work.visited);

/// As `assert_each_doubling_costs_at_most_two_point_two_times`, for the given measures only.
fn assert_measures_at_most_two_point_two_times(
    sizes: [usize; 4],
    document: impl Fn(usize) -> String,
    measures: &[Measure],
) {
    let measured: Vec<Work> = sizes
        .iter()
        .map(|&n| parse_long_input_work(document(n)))
        .collect();
    for (name, pick) in measures {
        let work: Vec<u64> = measured.iter().map(pick).collect();
        for (pair, window) in sizes.windows(2).zip(work.windows(2)) {
            assert!(
                window[1].saturating_mul(10) <= window[0].saturating_mul(22),
                "{name} grew faster than linearly: {} repeats cost {}, {} repeats cost {} \
                 (all measured: {work:?})",
                pair[0],
                window[0],
                pair[1],
                window[1],
            );
        }
    }
}

const LONG_RUN_SIZES: [usize; 4] = [1_000, 2_000, 4_000, 8_000];

#[test]
fn work_of_one_long_comment_at_most_doubles_with_its_length() {
    assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
        format!("<!doctype html><!--{}-->", "x".repeat(n))
    });
}

#[test]
fn work_of_one_long_attribute_value_at_most_doubles_with_its_length() {
    assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
        format!("<!doctype html><div a='{}'>", "x".repeat(n))
    });
}

#[test]
fn work_of_one_long_attribute_name_at_most_doubles_with_its_length() {
    assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
        format!("<!doctype html><div {}>", "x".repeat(n))
    });
}

#[test]
fn work_of_one_long_doctype_identifier_at_most_doubles_with_its_length() {
    assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
        format!("<!DOCTYPE html PUBLIC \"{}\">", "x".repeat(n))
    });
}

#[test]
fn work_of_one_long_processing_instruction_at_most_doubles_with_its_length() {
    assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
        format!("<!doctype html><?{}>", "a".repeat(n))
    });
}

#[test]
fn work_of_one_long_ambiguous_ampersand_literal_at_most_doubles_with_its_length() {
    assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
        format!("<!doctype html>&{}", "a".repeat(n))
    });
}

#[test]
fn work_of_long_whitespace_runs_before_the_html_element_at_most_doubles_with_their_length() {
    for unit in [" ", "\n", "\r\n", "\0"] {
        assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
            format!("<!doctype html>{}", unit.repeat(n))
        });
    }
}

#[test]
fn work_of_whitespace_runs_kept_between_comments_before_the_html_element_at_most_doubles() {
    // Each run is a separate pending text that waits for the `html` element.
    assert_each_doubling_costs_at_most_two_point_two_times([500, 1_000, 2_000, 4_000], |n| {
        format!("<!doctype html>{}", "<!--c--> ".repeat(n))
    });
}

#[test]
fn work_of_long_whitespace_and_null_runs_inside_the_body_at_most_doubles_with_their_length() {
    for unit in [" ", "\r\n", "\0", "a\0"] {
        assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
            format!("<!doctype html><body>{}", unit.repeat(n))
        });
    }
}

#[test]
fn work_of_a_long_text_run_in_a_table_at_most_doubles_with_its_length() {
    // The characters wait as pending table text until a token ends the run.
    assert_each_doubling_costs_at_most_two_point_two_times([250, 500, 1_000, 2_000], |n| {
        format!("<!doctype html><table>{}", "a b ".repeat(n))
    });
}

#[test]
fn work_of_nested_template_elements_at_most_doubles_with_their_number() {
    assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
        format!("<!doctype html>{}", "<template>".repeat(n))
    });
}

#[test]
fn work_of_repeated_formatting_elements_with_an_attribute_at_most_doubles_with_their_number() {
    assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
        format!("<!doctype html><p>{}", "<b a=i>".repeat(n))
    });
}

#[test]
fn work_of_reconstructing_a_formatting_element_with_many_attributes_at_most_doubles() {
    // `</p>` closes the `b` it contains, which stays in the list of active formatting elements;
    // the `x` that follows reconstructs it, copying every attribute to the new element. Every
    // measure of the work is linear in the number of attributes: the replay compares no name
    // pairwise.
    assert_measures_at_most_two_point_two_times(
        [250, 500, 1_000, 2_000],
        |n| {
            let attributes: String = (0..n).map(|i| format!(" a{i}=v")).collect();
            format!("<!doctype html><p><b{attributes}></p>x")
        },
        &[
            TOTAL_WORK,
            RESERVATION_WORK,
            COPY_WORK,
            INSPECTION_WORK,
            VISIT_WORK,
        ],
    );
}

/// One start tag of `tag` with `n` distinct attributes after `prefix`. The names have one length,
/// so that the total length of the names, which every algorithm reads, is linear in `n`.
fn start_tag_with_attributes(prefix: &str, tag: &str, n: usize) -> String {
    let attributes: String = (0..n).map(|i| format!(" a{i:05}=v")).collect();
    format!("<!doctype html>{prefix}<{tag}{attributes}>")
}

#[test]
fn work_of_an_input_start_tag_with_many_attributes_at_most_doubles() {
    // `input` installs its attributes one at a time, so that each can run its element-specific
    // effects. The installation must not compare each against every earlier one.
    assert_measures_at_most_two_point_two_times(
        [250, 500, 1_000, 2_000],
        |n| start_tag_with_attributes("", "input", n),
        &[
            WORK_OUTSIDE_RESERVATION,
            COPY_WORK,
            INSPECTION_WORK,
            VISIT_WORK,
        ],
    );
}

#[test]
fn work_of_a_details_start_tag_with_many_attributes_at_most_doubles() {
    assert_measures_at_most_two_point_two_times(
        [250, 500, 1_000, 2_000],
        |n| start_tag_with_attributes("", "details", n),
        &[
            WORK_OUTSIDE_RESERVATION,
            COPY_WORK,
            INSPECTION_WORK,
            VISIT_WORK,
        ],
    );
}

#[test]
fn work_of_a_select_start_tag_with_many_attributes_at_most_doubles() {
    assert_measures_at_most_two_point_two_times(
        [250, 500, 1_000, 2_000],
        |n| start_tag_with_attributes("", "select", n),
        &[
            WORK_OUTSIDE_RESERVATION,
            COPY_WORK,
            INSPECTION_WORK,
            VISIT_WORK,
        ],
    );
}

#[test]
fn work_of_a_div_start_tag_with_many_attributes_at_most_doubles() {
    // The bulk installation path every other element takes.
    assert_measures_at_most_two_point_two_times(
        [250, 500, 1_000, 2_000],
        |n| start_tag_with_attributes("", "div", n),
        &[
            WORK_OUTSIDE_RESERVATION,
            COPY_WORK,
            INSPECTION_WORK,
            VISIT_WORK,
        ],
    );
}

#[test]
fn work_of_merging_the_attributes_of_a_second_html_start_tag_at_most_doubles() {
    // A second `html` start tag merges its attributes into the first element ("in body"
    // insertion mode, `html` start tag, retained HTML Standard capture
    // `html-language-syntax-parser--877d6146487f--37a9ba51eed8.md`, line 4306); the tokenizer
    // already guarantees the candidates' names are distinct.
    assert_measures_at_most_two_point_two_times(
        [250, 500, 1_000, 2_000],
        |n| start_tag_with_attributes("<html><body>", "html", n),
        &[
            WORK_OUTSIDE_RESERVATION,
            COPY_WORK,
            INSPECTION_WORK,
            VISIT_WORK,
        ],
    );
}

#[test]
fn work_of_one_long_input_value_at_most_doubles_with_its_length() {
    // The value is sanitized into a copy that keeps the origin of every code point.
    assert_each_doubling_costs_at_most_two_point_two_times(LONG_RUN_SIZES, |n| {
        format!("<!doctype html><input value='{}'>", "x".repeat(n))
    });
}

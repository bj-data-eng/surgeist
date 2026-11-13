//! Parse work is linear in document size.
//!
//! The authority is the work model's own contract: every unit charges one
//! credit and a document of N repeated elements must cost O(N) credits, so an
//! exact (non-geometric) reservation that charges the whole old plus new vector
//! on every element violates it.

use surgeist_html::{
    construction::HtmlDocumentMode,
    parsing::HtmlParserScriptingMode,
    security::nonce::{
        HtmlBrowsingContextPresence, HtmlHeaderDeliveredCsp, HtmlNonceDocumentEnvironment,
    },
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
        .active_formatting_entries(100_000_000)
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
    // the list is not collected per insertion and total work is linear too. The retained
    // rows of the parse hold about ten per option and their capacity doubles at 1024 rows, so
    // the sizes are chosen where no size sits on a capacity step: each doubling then crosses
    // one step.
    assert_doublings_add_near_double(
        [60, 120, 240, 480],
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
    // sibling must cost the same links as inserting last, never a search along
    // the body's children.
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

/// The three lowercase letters of `n` in base 26, least significant first, so that names
/// for consecutive `n` differ in their first unit.
fn letters(n: usize) -> String {
    [n % 26, n / 26 % 26, n / 676 % 26]
        .iter()
        .map(|&digit| char::from(b'a' + digit as u8))
        .collect()
}

/// The bytes inspected beyond those of a processing instruction whose `pairs` distinct names
/// differ in their first unit, when the names share a 20-unit prefix instead. The two documents
/// have the same length and structure, so every other charge, including the walk over the
/// instruction's source-map origins that grows with the instruction on its own, is identical;
/// only comparing names inspects units to a depth that depends on a shared prefix.
fn shared_prefix_name_bytes(pairs: usize) -> u64 {
    let document = |name: &dyn Fn(usize) -> String| {
        let names: Vec<String> = (0..pairs).map(|n| format!("{}=''", name(n))).collect();
        format!("<?marker {}?>", names.join(" "))
    };
    let prefix = "x".repeat(20);
    let shared = parse_fragment_work(document(&|n| format!("{prefix}{}", letters(n))));
    let early = parse_fragment_work(document(&|n| format!("{}{prefix}", letters(n))));
    shared.inspected.saturating_sub(early.inspected)
}

#[test]
fn bytes_inspected_comparing_a_processing_instructions_names_grow_no_faster_than_its_pairs() {
    // XML-stylesheet 20101028 section 3 makes a repeated name an error, so each name is checked
    // against the earlier ones; that check must not compare every pair of names. Each
    // doubling of the pairs may at most triple the previous extra, with a little slack for the
    // random slot collisions of the keyed name table; comparing every pair quadruples it.
    let extra: Vec<u64> = [25, 50, 100, 200].map(shared_prefix_name_bytes).to_vec();
    for window in extra.windows(2) {
        assert!(
            window[1] <= 3 * window[0] + 4_000,
            "doubling the pairs grew the extra inspected bytes from {} to {}: {extra:?}",
            window[0],
            window[1]
        );
    }
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
    // capacity steps of the row banks. Finding each child by a search along
    // the parent's children adds a term quadratic in n: E(2n) approaches
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
    // needs to look beyond the layers around it. The sizes are not capacities of a
    // bank: the layer that fills a bank also takes its growth, which charges visits of
    // its own (the layer after 25 adds five more than after 26, 100 or 400), and the
    // doubling tests above bound that growth.
    let added = |layers: usize| {
        let layer = format!("{SHADOW_LAYER}{SHADOW_LAYER_END}");
        parse_fragment_work(nested_shadow_roots(layers, &layer)).visited as i64
            - parse_fragment_work(nested_shadow_roots(layers, "")).visited as i64
    };
    let measures = [26, 100, 400].map(added);
    assert!(
        measures.iter().all(|visits| *visits == measures[0]),
        "one more shadow root added {measures:?} record visits inside 26, 100 and 400"
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
/// Sizes for the runs whose growth shows only past the first few thousand units.
const LONGER_RUN_SIZES: [usize; 4] = [2_000, 4_000, 8_000, 16_000];

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
fn work_of_a_long_text_run_in_a_closed_textarea_at_most_doubles_with_its_length() {
    // Each character appends to the one text child, and the textarea's value is that child's text.
    assert_measures_at_most_two_point_two_times(
        LONGER_RUN_SIZES,
        |n| format!("<!doctype html><textarea>{}</textarea>", "a".repeat(n)),
        &[TOTAL_WORK, RESERVATION_WORK, COPY_WORK],
    );
}

#[test]
fn work_of_a_long_text_run_with_lines_in_a_closed_textarea_at_most_doubles_with_its_length() {
    // Three units for each repeat, and the largest size is the most the source map limit of the
    // long-input limits holds.
    assert_measures_at_most_two_point_two_times(
        [1_500, 3_000, 6_000, 12_000],
        |n| format!("<!doctype html><textarea>{}</textarea>", "a\nb".repeat(n)),
        &[TOTAL_WORK, RESERVATION_WORK, COPY_WORK],
    );
}

#[test]
fn work_of_a_long_text_run_in_an_unclosed_textarea_at_most_doubles_with_its_length() {
    assert_measures_at_most_two_point_two_times(
        LONGER_RUN_SIZES,
        |n| format!("<!doctype html><textarea>{}", "a".repeat(n)),
        &[TOTAL_WORK, RESERVATION_WORK, COPY_WORK],
    );
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
fn work_of_nested_divs_with_text_at_most_doubles_with_their_number() {
    // Pinned HTML 877d6146487f section 13.2.6.4.7, "A start tag whose tag name is one of: address,
    // ..., div, ...": "If the stack of open elements has a p element in button scope, then close
    // a p element." No `p` is open and no scope boundary is, so the answer is no for every
    // `div`. Deciding it must not examine the whole stack of open elements each time.
    assert_measures_at_most_two_point_two_times(
        [250, 500, 1_000, 2_000],
        |n| format!("<!doctype html>{}", "<div>y".repeat(n)),
        &[WORK_OUTSIDE_RESERVATION, VISIT_WORK],
    );
}

#[test]
fn work_of_nested_divs_without_text_at_most_doubles_with_their_number() {
    assert_measures_at_most_two_point_two_times(
        [250, 500, 1_000, 2_000],
        |n| format!("<!doctype html>{}", "<div>".repeat(n)),
        &[WORK_OUTSIDE_RESERVATION, VISIT_WORK],
    );
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

/// What one parse holds at its high water in `resource`.
fn high_water(raw: String, resource: HtmlParseResource) -> u64 {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), long_input_limits())
        .parse()
        .unwrap_or_else(|_| panic!("the document parses within the long-input limits"))
        .usage()
        .resource(resource)
        .high_water()
}

#[test]
fn auxiliary_entries_of_one_long_attribute_name_grow_by_less_than_an_entry_per_eight_units() {
    // HTML 877d6146487f section 13.2.5.33 (attribute name state): a name equal to one already on
    // the token is dropped as a duplicate, so the tokenizer must recognise the names it kept. That
    // takes a record for the name, not sixteen for each of its units.
    let entries = |units: usize| {
        high_water(
            format!("<!doctype html><div {}>", "a".repeat(units)),
            HtmlParseResource::AuxiliaryEntries,
        )
    };
    let (small, large) = (entries(4_000), entries(8_000));
    assert!(
        (large - small) * 8 <= 4_000,
        "4,000 more units of one name hold {} more auxiliary entries ({small} at 4,000 units, \
         {large} at 8,000)",
        large - small,
    );
}

#[test]
fn capacity_of_one_long_attribute_name_grows_by_under_four_hundred_bytes_per_unit() {
    // The name is stored once as text and once for each origin of its units, which is under 400
    // bytes of capacity for each unit; recognising it as a duplicate adds nothing for each unit.
    let capacity = |units: usize| {
        high_water(
            format!("<!doctype html><div {}>", "a".repeat(units)),
            HtmlParseResource::ManagedCapacityBytes,
        )
    };
    let (small, large) = (capacity(4_000), capacity(8_000));
    assert!(
        (large - small) <= 4_000 * 400,
        "4,000 more units of one name hold {} more bytes of capacity ({small} at 4,000 units, \
         {large} at 8,000)",
        large - small,
    );
}

#[test]
fn auxiliary_entries_of_distinct_attribute_names_grow_by_at_most_eight_for_each_name() {
    // Names with no shared prefix, so a trie finds nothing to reuse: eight hexadecimal digits of a
    // multiplicative hash of the index. A bitwise trie keeps sixteen records for each of the
    // five or so units a name does not share, about eighty; eight is a tenth of that. The names'
    // own records are a leaf and a branch, and each attribute holds a few more auxiliary entries
    // of the token (three are held by a repeated name, which has no record of its own).
    let entries = |names: usize| {
        let attributes: String = (0..names)
            .map(|i| format!(" {:08x}", (i as u32).wrapping_mul(2_654_435_761)))
            .collect();
        high_water(
            format!("<!doctype html><div{attributes}>"),
            HtmlParseResource::AuxiliaryEntries,
        )
    };
    let (small, large) = (entries(1_000), entries(2_000));
    assert!(
        (large - small) <= 8 * 1_000,
        "1,000 more names hold {} more auxiliary entries ({small} at 1,000 names, {large} at \
         2,000)",
        large - small,
    );
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

/// What one parse of a document cost, for the three shapes below that repeat a trigger that once
/// rescanned its tree or its registry: records visited, and the high-water mark of the auxiliary
/// entries the parse retained.
#[derive(Clone, Copy)]
struct Rescan {
    visited: u64,
    entries: u64,
}

/// A named measure of a `Rescan`.
type RescanMeasure = (&'static str, fn(&Rescan) -> u64);

fn rescan_of(usage: &surgeist_html_parser::HtmlParseUsage) -> Rescan {
    Rescan {
        visited: usage.work(HtmlParseWorkCategory::RecordVisit),
        entries: usage
            .resource(HtmlParseResource::AuxiliaryEntries)
            .high_water(),
    }
}

fn parse_rescan(raw: String) -> Rescan {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), long_input_limits())
        .parse()
        .unwrap_or_else(|_| panic!("the document parses within the long-input limits"));
    rescan_of(report.usage())
}

/// Requires each measure at a size to be at most 2.2 times the measure at the size before, for
/// each of the three doublings of `sizes`. Linear work costs at most double with each doubling
/// (the fixed cost of the document only lowers the ratio), while a rescan per trigger costs
/// four times as much and already exceeds 2.2 at the sizes below.
fn assert_rescans_at_most_two_point_two_times(
    sizes: [usize; 4],
    measured: &[Rescan],
    measures: &[RescanMeasure],
) {
    for (name, pick) in measures {
        let values: Vec<u64> = measured.iter().map(pick).collect();
        for (pair, window) in sizes.windows(2).zip(values.windows(2)) {
            assert!(
                window[1].saturating_mul(10) <= window[0].saturating_mul(22),
                "{name} grew faster than linearly: {} repeats cost {}, {} repeats cost {} \
                 (all measured: {values:?})",
                pair[0],
                window[0],
                pair[1],
                window[1],
            );
        }
    }
}

/// `<?start name='x'?>`, `siblings` empty elements, `<?end?>`, then a `template for=x` whose
/// content patching moves the siblings between the markers.
fn template_patching_document(siblings: usize, with_template: bool) -> String {
    format!(
        "<!doctype html><body><?start name='x'?>{}<?end?>{}",
        "<i></i>".repeat(siblings),
        if with_template {
            "<template for=x><p>y</template>"
        } else {
            ""
        },
    )
}

#[test]
fn work_of_patching_a_template_from_a_marker_range_at_most_doubles_with_the_siblings() {
    // "Prepare content patching" and "find markers" (HTML Standard, the template element) walk
    // the siblings between the start and end markers once each. The control without the
    // `template for` is measured at the smallest size to show that the patching ran.
    let sizes = [1_000, 2_000, 4_000, 8_000];
    let control = parse_rescan(template_patching_document(sizes[0], false));
    let measured: Vec<Rescan> = sizes
        .iter()
        .map(|&n| parse_rescan(template_patching_document(n, true)))
        .collect();
    assert!(
        measured[0].visited > control.visited,
        "the template patching visited nothing: {} with it, {} without",
        measured[0].visited,
        control.visited,
    );
    assert_rescans_at_most_two_point_two_times(
        sizes,
        &measured,
        &[("record visits", |work| work.visited)],
    );
}

#[test]
fn work_of_styles_in_template_contents_at_most_doubles_with_their_number() {
    // The contents of a template are not the main tree, so the order of their styles was walked
    // and retained again for each style: the visits and the retained entries both grew with the
    // square of the number of styles.
    let sizes = [250, 500, 1_000, 2_000];
    let measured: Vec<Rescan> = sizes
        .iter()
        .map(|&n| {
            parse_rescan(format!(
                "<!doctype html><body><template>{}",
                "<style>a{}</style>".repeat(n)
            ))
        })
        .collect();
    assert_rescans_at_most_two_point_two_times(
        sizes,
        &measured,
        &[
            ("record visits", |work| work.visited),
            ("auxiliary entries retained", |work| work.entries),
        ],
    );
}

#[test]
fn work_of_reached_meta_csp_elements_at_most_doubles_with_their_number() {
    // The controlled processor reports every policy unchanged, so nothing it returns grows with
    // the number of metas; only the parser's own bookkeeping around each reached meta can.
    use crate::parser::construction::meta_csp::Processor;
    let sizes = [250, 500, 1_000, 2_000];
    let measured: Vec<Rescan> = sizes
        .iter()
        .map(|&n| {
            let mut processor = Processor::new();
            let raw = format!(
                "<!doctype html><head>{}",
                "<meta http-equiv=content-security-policy content='='>".repeat(n)
            );
            let report =
                HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), long_input_limits())
                    .parse_with_meta_csp(&mut processor)
                    .unwrap_or_else(|_| panic!("the document parses within the long-input limits"));
            assert_eq!(processor.calls, n, "every meta is reached once");
            rescan_of(report.usage())
        })
        .collect();
    assert_rescans_at_most_two_point_two_times(
        sizes,
        &measured,
        &[("record visits", |work| work.visited)],
    );
}

/// `<b><p>`, `styles` style elements, then `</b>`: the adoption agency moves the `p` and all its
/// styles at once, so each style sees one removal and one insertion with the styles after it
/// unlisted. `inside` wraps the markup in the given opening tags.
fn moved_styles_document(inside: &str, styles: usize) -> String {
    format!(
        "<!doctype html><body>{inside}<b><p>{}</b>",
        "<style>a{}</style>".repeat(styles)
    )
}

fn assert_moved_styles_at_most_double(inside: &str) {
    let sizes = [250, 500, 1_000, 2_000];
    let measured: Vec<Rescan> = sizes
        .iter()
        .map(|&n| parse_rescan(moved_styles_document(inside, n)))
        .collect();
    assert_rescans_at_most_two_point_two_times(
        sizes,
        &measured,
        &[
            ("record visits", |work| work.visited),
            ("auxiliary entries retained", |work| work.entries),
        ],
    );
}

#[test]
fn work_of_styles_moved_together_by_the_adoption_agency_at_most_doubles_with_their_number() {
    assert_moved_styles_at_most_double("");
}

#[test]
fn work_of_styles_moved_together_by_the_adoption_agency_in_a_template_at_most_doubles() {
    assert_moved_styles_at_most_double("<template>");
}

/// Requires `visits(n)` to be at most 2.2 times `visits(m)` for each pair of
/// consecutive `sizes`: a lookup that scans the payload registry for each of n
/// repeated nodes makes the record visits quadruple with each doubling.
fn assert_visits_each_doubling_at_most_two_point_two_times(
    sizes: [usize; 4],
    what: &str,
    visits: impl Fn(usize) -> u64,
) {
    let measured: Vec<u64> = sizes.iter().map(|&n| visits(n)).collect();
    for (pair, window) in sizes.windows(2).zip(measured.windows(2)) {
        assert!(
            window[1].saturating_mul(10) <= window[0].saturating_mul(22),
            "{what}: {} repeats cost {} record visits, {} repeats cost {} \
             (all measured: {measured:?})",
            pair[0],
            window[0],
            pair[1],
            window[1],
        );
    }
}

const REPEAT_SIZES: [usize; 4] = [500, 1_000, 2_000, 4_000];

#[test]
fn record_visits_of_one_processing_instruction_with_many_pairs_at_most_double_with_the_pairs() {
    // XML-stylesheet 20101028 section 3: each pseudo-attribute name and value is
    // read once, and the origins of the instruction's data that cover them are
    // consumed in source order, so no unit of a name or value may walk the
    // origins of all the others again.
    assert_visits_each_doubling_at_most_two_point_two_times(
        [50, 100, 200, 400],
        "one processing instruction with many pairs",
        |pairs| {
            let attributes: Vec<String> = (0..pairs).map(|n| format!("n{n}=''")).collect();
            parse_fragment_work(format!("<?marker {}?>", attributes.join(" "))).visited
        },
    );
}

#[test]
fn record_visits_of_repeated_processing_instructions_with_a_pair_at_most_double_with_their_number()
{
    assert_visits_each_doubling_at_most_two_point_two_times(
        REPEAT_SIZES,
        "repeated processing instructions",
        |n| {
            parse_work(format!(
                "<!doctype html><body>{}",
                "<?a name='q'?>".repeat(n)
            ))
            .visited
        },
    );
}

#[test]
fn record_visits_a_template_for_adds_over_repeated_processing_instructions_at_most_double() {
    // `template for` looks for a marker among the instructions that precede it;
    // none is named `x`, so the lookup reads the `name` pair of each one.
    assert_visits_each_doubling_at_most_two_point_two_times(
        REPEAT_SIZES,
        "a template for over repeated processing instructions",
        |n| {
            let instructions = "<?a name='q'?>".repeat(n);
            let with = parse_work(format!(
                "<!doctype html><body>{instructions}<template for=x></template>"
            ));
            let without = parse_work(format!("<!doctype html><body>{instructions}"));
            with.visited.saturating_sub(without.visited)
        },
    );
}

#[test]
fn record_visits_of_repeated_svg_scripts_at_most_double_with_their_number() {
    assert_visits_each_doubling_at_most_two_point_two_times(
        REPEAT_SIZES,
        "repeated svg scripts",
        |n| {
            parse_work(format!(
                "<!doctype html><body>{}",
                "<svg><script></script></svg>".repeat(n)
            ))
            .visited
        },
    );
}

#[test]
fn record_visits_of_repeated_open_details_at_most_double_with_their_number() {
    assert_visits_each_doubling_at_most_two_point_two_times(
        REPEAT_SIZES,
        "repeated open details",
        |n| {
            parse_work(format!(
                "<!doctype html><body>{}",
                "<details open></details>".repeat(n)
            ))
            .visited
        },
    );
}

/// A `div` in a document with a browsing context and a header-delivered CSP
/// policy, captured as the context element of a fragment parse.
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
    let host = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_children(&document, std::slice::from_ref(&host))
        .unwrap();
    builder.establish_no_shadow_root(&host).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&host).unwrap()).unwrap()
}

#[test]
fn record_visits_of_nonce_elements_hidden_by_a_header_policy_at_most_double_with_their_number() {
    // Content Security Policy (nonce hiding): an inserted element with a nonce in a connected
    // shadow tree under a header-delivered policy has its `nonce` attribute emptied, which
    // replaces one attribute of each such element.
    assert_visits_each_doubling_at_most_two_point_two_times(
        REPEAT_SIZES,
        "repeated nonce elements hidden by a header policy",
        |n| {
            let request = HtmlFragmentParseRequest::with_options(
                HtmlSource::utf8(format!(
                    "<template shadowrootmode=open>{}</template>",
                    "<p nonce=A></p>".repeat(n)
                )),
                connected_div_context(),
                deep_limits(),
                HtmlParserScriptingMode::Inert,
                true,
            )
            .unwrap();
            work_of(
                request
                    .parse()
                    .unwrap_or_else(|_| panic!("the fragment parses"))
                    .usage(),
            )
            .visited
        },
    );
}

#[test]
fn record_visits_of_repeated_open_details_of_one_group_at_most_double_with_their_number() {
    // Opening a details element closes the others of its name group, which removes the `open`
    // attribute of the one before: one removal for each element after the first.
    assert_visits_each_doubling_at_most_two_point_two_times(
        REPEAT_SIZES,
        "repeated open details of one name group",
        |n| {
            parse_work(format!(
                "<!doctype html><body>{}",
                "<details name=a open></details>".repeat(n)
            ))
            .visited
        },
    );
}

#[test]
fn record_visits_of_repeated_bases_without_an_href_at_most_double_with_their_number() {
    // HTML 877d6146487f section 4.2.3 (the base element): the document base URL follows the first
    // base element with an href attribute in tree order, so a base without one cannot change it
    // and its insertion needs no search of the document.
    assert_visits_each_doubling_at_most_two_point_two_times(
        REPEAT_SIZES,
        "repeated bases without an href",
        |n| parse_work(format!("<!doctype html><body>{}", "<base>".repeat(n))).visited,
    );
}

#[test]
fn record_visits_the_adoption_agency_adds_by_moving_bases_without_an_href_at_most_double() {
    // `</b>` makes the adoption agency move the children of the `p`, one removal and one insertion
    // for each base. With no base that has an href, none of them changes the first.
    assert_visits_each_doubling_at_most_two_point_two_times(
        [512, 1_024, 2_048, 4_096],
        "bases moved by the adoption agency",
        |n| {
            let bases = "<base>".repeat(n);
            let moved = parse_work(format!("<!doctype html><body><b>1<p>{bases}</b>"));
            let kept = parse_work(format!("<!doctype html><body><b>1<p>{bases}"));
            moved.visited.saturating_sub(kept.visited)
        },
    );
}

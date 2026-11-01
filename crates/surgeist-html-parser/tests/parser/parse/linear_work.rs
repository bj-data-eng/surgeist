//! Parse work is linear in document size.
//!
//! The authority is the work model's own contract: every unit charges one
//! credit and a document of N repeated elements must cost O(N) credits, so an
//! exact (non-geometric) reservation that charges the whole old plus new vector
//! on every element violates it.

use surgeist_html::{
    HtmlDocumentMode, HtmlParserScriptingMode, Namespace, OrderedAttributes, QualifiedName,
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

/// `Work` is charged to one parse: total credits, bytes reserved and records
/// visited.
#[derive(Clone, Copy)]
struct Work {
    total: u64,
    reserved: u64,
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
        roomy_limits(),
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

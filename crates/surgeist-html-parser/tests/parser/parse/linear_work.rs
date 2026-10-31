//! Parse work is linear in document size.
//!
//! The authority is the work model's own contract: every unit charges one
//! credit and a document of N repeated elements must cost O(N) credits, so an
//! exact (non-geometric) reservation that charges the whole old plus new vector
//! on every element violates it.

use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseResource, HtmlParseWorkCategory, HtmlSource,
};

/// One element subtree with attributes, nested elements and two text nodes.
const SUBTREE: &str = "<div id=a class=b><p>t</p><span>x</span></div>";

/// `Work` is charged to one parse: total credits and bytes reserved.
#[derive(Clone, Copy)]
struct Work {
    total: u64,
    reserved: u64,
}

/// A named measure of one parse's work.
type Measure = (&'static str, fn(&Work) -> u64);

/// Limits that no measured document reaches, so only the work model differs
/// between sizes.
fn roomy_limits() -> HtmlParseLimits {
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
    let usage = report.usage();
    Work {
        total: usage.resource(HtmlParseResource::WorkCredits).current(),
        reserved: usage.work(HtmlParseWorkCategory::ReserveByte),
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
    const BOUND: u64 = 3;
    let measured: Vec<Work> = sizes.iter().map(|&n| parse_work(document(n))).collect();
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
    // The selectedness algorithm walks every option on each insertion, so total
    // work is not linear; the list it collects must still grow geometrically.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |options| {
            format!(
                "<!doctype html><select>{}</select>",
                "<option>x</option>".repeat(options)
            )
        },
        &[("reservation work", |work| work.reserved)],
    );
}

#[test]
fn reservation_work_added_by_each_doubling_of_a_documents_style_elements_is_near_double() {
    // Each style element's association collects every style element of its
    // tree in order, so total work is not linear; that list must still grow
    // geometrically.
    assert_doublings_add_near_double(
        [50, 100, 200, 400],
        |styles| {
            format!(
                "<!doctype html><head>{}",
                "<style>a{}</style>".repeat(styles)
            )
        },
        &[("reservation work", |work| work.reserved)],
    );
}

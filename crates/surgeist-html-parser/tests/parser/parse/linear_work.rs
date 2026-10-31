//! Parse work is linear in document size.
//!
//! The authority is the work model's own contract: every unit charges one
//! credit and a document of N repeated elements must cost O(N) credits, so an
//! exact (non-geometric) reservation that charges the whole old plus new vector
//! on every element violates it.

use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseResource, HtmlParseWorkCategory,
    HtmlSource,
};

/// One element subtree with attributes, nested elements and two text nodes.
const SUBTREE: &str = "<div id=a class=b><p>t</p><span>x</span></div>";

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

/// Total credits and reserved bytes charged to parse `repeats` subtrees.
fn parse_work(repeats: usize) -> (u64, u64) {
    let raw = format!("<!doctype html>{}", SUBTREE.repeat(repeats));
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), roomy_limits());
    let report = request
        .parse()
        .unwrap_or_else(|_| panic!("{repeats} repeats parse within roomy limits"));
    let usage = report.usage();
    (
        usage.resource(HtmlParseResource::WorkCredits).current(),
        usage.work(HtmlParseWorkCategory::ReserveByte),
    )
}

#[test]
fn doubling_the_document_at_most_doubles_total_and_reservation_work() {
    // Linear work W(n) = a*n + b with b >= 0 satisfies W(2n) <= 2*W(n). Geometric
    // vector growth keeps that bound: each vector's capacity sequence doubles
    // from a fixed base, so its capacity for a document twice the size is at
    // most twice as large, and the old-plus-new bytes charged per growth step
    // are a fixed multiple of that capacity. A term growing as n^2 instead
    // approaches a ratio of four.
    let sizes = [50_usize, 100, 200, 400];
    let measured: Vec<(u64, u64)> = sizes.iter().map(|&n| parse_work(n)).collect();
    for (pair, window) in sizes.windows(2).zip(measured.windows(2)) {
        let (small, large) = (window[0], window[1]);
        assert!(
            large.0 <= 2 * small.0,
            "total work grew faster than linearly from {} to {} repeats: {} -> {}",
            pair[0],
            pair[1],
            small.0,
            large.0
        );
        assert!(
            large.1 <= 2 * small.1,
            "reservation work grew faster than linearly from {} to {} repeats: {} -> {}",
            pair[0],
            pair[1],
            small.1,
            large.1
        );
    }
}

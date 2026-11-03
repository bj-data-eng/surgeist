//! Success is monotone in each byte limit.
//!
//! Buffers grow to a target that depends only on the buffer, never on a limit, and a limit
//! that cannot hold the target refuses the reservation. A parse therefore follows one
//! allocation path under every limit, needs exactly the peak that a run with generous limits
//! reports, and succeeds under a limit exactly when the limit holds that peak.

use super::linear_work::roomy_limits;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseResource, HtmlSource,
};

/// Text, a comment, attributes, misnested formatting, a template and a table, so that the
/// tokenizer's buffers, the open-element stack, the formatting list and the pending text of a
/// table all grow.
const DOCUMENT: &str = "<!doctype html>  \n<html><head><title>t &amp; u</title></head>\
    <body><p class=a id=b data-x='long value'>text &amp; more<b>x<i>y</b></i></p>\
    <template><div><span>s</span></div></template>\
    <table><tr><td>c</td></tr>a b c</table><!--a comment--></body></html>";

fn parses(limits: HtmlParseLimits) -> bool {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(DOCUMENT), limits)
        .parse()
        .is_ok()
}

#[test]
fn a_parse_succeeds_under_a_byte_limit_exactly_when_the_limit_holds_its_measured_peak() {
    type Select = fn(HtmlParseLimits, u64) -> HtmlParseLimits;
    let limited: [(HtmlParseResource, Select); 3] = [
        (HtmlParseResource::ManagedCapacityBytes, |limits, cap| {
            limits
                .into_builder()
                .managed_capacity_bytes(cap)
                .finish()
                .unwrap()
        }),
        (HtmlParseResource::PeakTransientBytes, |limits, cap| {
            limits
                .into_builder()
                .peak_transient_bytes(cap)
                .finish()
                .unwrap()
        }),
        (HtmlParseResource::AtomicLayoutBytes, |limits, cap| {
            limits
                .into_builder()
                .atomic_layout_bytes(cap)
                .finish()
                .unwrap()
        }),
    ];
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(DOCUMENT), roomy_limits())
        .parse()
        .expect("the document parses within roomy limits");
    for (resource, select) in limited {
        // The peak is what the run under generous limits reports, not a formula.
        let peak = report.usage().resource(resource).high_water();
        assert!(peak > 0, "{resource:?} was never used");
        // Every limit within a window around the peak, and a spread of limits far below it.
        let window = peak.saturating_sub(256)..=peak + 256;
        let spread = (0..24).map(|step| peak >> step).chain([0, 1]);
        let mut limits: Vec<u64> = window.chain(spread).collect();
        limits.sort_unstable();
        limits.dedup();
        for limit in limits {
            assert_eq!(
                parses(select(roomy_limits(), limit)),
                limit >= peak,
                "{resource:?} limit {limit} against a measured peak of {peak}"
            );
        }
    }
}

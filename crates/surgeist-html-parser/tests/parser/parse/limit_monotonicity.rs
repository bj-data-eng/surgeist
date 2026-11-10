//! Success is monotone in each byte limit.
//!
//! Buffers grow to a target that depends only on the buffer, never on a limit, and a limit
//! that cannot hold the target refuses the reservation. A parse therefore follows one
//! allocation path under every limit, needs exactly the peak that a run with generous limits
//! reports, and succeeds under a limit exactly when the limit holds that peak.

use super::linear_work::roomy_limits;
use surgeist_html_parser::{HtmlParseLimits, HtmlParseResource, HtmlSource};

/// Documents that between them grow the tokenizer's buffers, the open-element stack, the
/// formatting list and the pending text of a table; take the meta elements whose values the
/// CSS-delegated budgets parse; and sanitize input values of the color, range and number
/// types through the same budgets.
const DOCUMENTS: [(&str, &str); 4] = [
    (
        "text, formatting, template and table",
        "<!doctype html>  \n<html><head><title>t &amp; u</title></head>\
         <body><p class=a id=b data-x='long value'>text &amp; more<b>x<i>y</b></i></p>\
         <template><div><span>s</span></div></template>\
         <table><tr><td>c</td></tr>a b c</table><!--a comment--></body></html>",
    ),
    (
        "color-scheme and theme-color metas",
        "<!doctype html><html><head>\
         <meta name=color-scheme content='light dark'>\
         <meta name=theme-color content='#336699'>\
         <meta name=theme-color media='(prefers-color-scheme: dark)' content=rebeccapurple>\
         <meta name=theme-color content=bogus><title>t</title></head><body>x</body></html>",
    ),
    (
        "color, range and number inputs",
        "<!doctype html><body><form>\
         <input type=color value='#AABBCC'><input type=color value=bogus>\
         <input type=range min=0 max=10 step=2 value=5><input type=range value=99>\
         <input type=number min=1 max=9 value=4><input type=number value=abc>\
         </form>",
    ),
    (
        "textarea text that grows the raw value's backing",
        "<!doctype html><body><form><textarea name=t>abcdefghijklmnopqrstuvwxyz0123456789</textarea></form>",
    ),
];

fn parses(document: &str, limits: HtmlParseLimits) -> bool {
    crate::parser::support::offline_all(HtmlSource::utf8(document), limits)
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
    for (name, document) in DOCUMENTS {
        let report =
            crate::parser::support::offline_all(HtmlSource::utf8(document), roomy_limits())
                .parse()
                .unwrap_or_else(|_| panic!("{name} parses within roomy limits"));
        for (resource, select) in limited {
            // The peak is what the run under generous limits reports, not a formula.
            let peak = report.usage().resource(resource).high_water();
            assert!(peak > 0, "{name}: {resource:?} was never used");
            // Every limit within a window around the peak, and a spread far below it.
            let window = peak.saturating_sub(256)..=peak + 256;
            let spread = (0..24).map(|step| peak >> step).chain([0, 1]);
            let mut limits: Vec<u64> = window.chain(spread).collect();
            limits.sort_unstable();
            limits.dedup();
            for limit in limits {
                assert_eq!(
                    parses(document, select(roomy_limits(), limit)),
                    limit >= peak,
                    "{name}: {resource:?} limit {limit} against a measured peak of {peak}"
                );
            }
        }
    }
}

/// Parses a textarea whose raw value is extended by each of its thirty-seven characters, under
/// limits of the logical resources the extension charges or admits. The parse succeeds exactly
/// when the limit holds what the run under generous limits reports, and a refused parse hands
/// every owner back.
#[test]
fn a_textarea_parse_succeeds_under_a_logical_limit_exactly_when_the_limit_holds_its_peak() {
    type Select = fn(HtmlParseLimits, u64) -> HtmlParseLimits;
    let limited: [(HtmlParseResource, Select); 4] = [
        (HtmlParseResource::ManagedUtf16Units, |limits, cap| {
            limits
                .into_builder()
                .managed_utf16_units(cap)
                .finish()
                .unwrap()
        }),
        (HtmlParseResource::SourceMapEntries, |limits, cap| {
            limits
                .into_builder()
                .source_map_entries(cap)
                .finish()
                .unwrap()
        }),
        (HtmlParseResource::AuxiliaryEntries, |limits, cap| {
            limits
                .into_builder()
                .auxiliary_entries(cap)
                .finish()
                .unwrap()
        }),
        (HtmlParseResource::WorkCredits, |limits, cap| {
            limits.into_builder().work_credits(cap).finish().unwrap()
        }),
    ];
    let (_, document) = DOCUMENTS[3];
    let report = crate::parser::support::offline_all(HtmlSource::utf8(document), roomy_limits())
        .parse()
        .unwrap();
    for (resource, select) in limited {
        let peak = report.usage().resource(resource).high_water();
        assert!(peak > 0, "{resource:?} was never used");
        let spread = (0..24).map(|step| peak >> step).chain([0, 1]);
        let mut limits: Vec<u64> = (peak.saturating_sub(300)..=peak + 3)
            .chain(spread)
            .collect();
        limits.sort_unstable();
        limits.dedup();
        for limit in limits {
            let result = crate::parser::support::offline_all(
                HtmlSource::utf8(document),
                select(roomy_limits(), limit),
            )
            .parse();
            assert_eq!(
                result.is_ok(),
                limit >= peak,
                "{resource:?} limit {limit} against a measured peak of {peak}"
            );
            if let Err(failure) = result {
                assert_eq!(failure.cleanup_error(), None, "{resource:?} limit {limit}");
            }
        }
    }
}

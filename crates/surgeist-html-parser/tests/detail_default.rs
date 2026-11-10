//! What a parse that selects no detail retains.
//!
//! A request selects optional parse detail, and the default selection is none.
//! A parse of a document with parse errors, CRLF newlines, markup of every
//! lexical shape, a `style` element and a `meta` default-style name then keeps no
//! source map, lexical, omission, history or diagnostic storage: none of the
//! resources that hold them is ever used, which the usage of the report states.

use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseResource, HtmlSource,
};

fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .active_formatting_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(1_000)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(u64::MAX)
        .finish()
        .unwrap()
}

const DOCUMENT: &str = "<p class=a>one<b>two</b></p>\r\n< <!-- c --><table><tr><td>x</table>\
<style title=A>a{}</style><meta http-equiv=default-style content=B>";

#[test]
fn a_parse_that_selects_no_detail_retains_none_of_it() {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(DOCUMENT), limits())
        .parse()
        .unwrap();
    for resource in [
        HtmlParseResource::SourceMapEntries,
        HtmlParseResource::SourceMapPayloadUnits,
        HtmlParseResource::Diagnostics,
        HtmlParseResource::DiagnosticPayloadUnits,
    ] {
        let usage = report.usage().resource(resource);
        assert_eq!(
            (usage.current(), usage.high_water()),
            (0, 0),
            "{resource:?} is neither retained nor reached"
        );
    }
}

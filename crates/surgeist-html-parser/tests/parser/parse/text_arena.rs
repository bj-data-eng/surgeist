//! The text of one parse: every text and comment of a document is a range of one block, and the
//! text the parse retains stays within two bytes per unit plus a bounded slack, however the
//! document merges its text.

use crate::parser::support::limits_builder;
use std::collections::BTreeSet;
use surgeist_html::{
    construction::{HtmlNode, HtmlNodeKind},
    values::DocumentString,
};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseResource, HtmlParseWorkCategory, HtmlSource,
};

/// Limits that no document here reaches.
fn large() -> HtmlParseLimits {
    limits_builder()
        .source_units(1 << 22)
        .buffered_utf16_units(1 << 22)
        .managed_nodes(1 << 22)
        .managed_edges(1 << 22)
        .managed_attributes(1 << 22)
        .managed_utf16_units(1 << 24)
        .source_map_entries(1 << 22)
        .source_map_payload_units(1 << 22)
        .diagnostics(1 << 22)
        .diagnostic_payload_units(1 << 22)
        .auxiliary_entries(1 << 24)
        .active_formatting_entries(1 << 22)
        .token_attributes(1 << 22)
        .tree_depth(1 << 22)
        .managed_capacity_bytes(1 << 36)
        .peak_transient_bytes(1 << 36)
        .atomic_layout_bytes(1 << 36)
        .work_credits(u64::MAX)
        .finish()
        .unwrap()
}

/// The strings of every text and comment node of the parsed document, in document order.
fn texts(report: &surgeist_html_parser::HtmlParseReport) -> Vec<DocumentString> {
    let document = report.document().unwrap();
    let mut found = Vec::new();
    let mut pending: Vec<_> = document.root().children().collect();
    pending.reverse();
    while let Some(id) = pending.pop() {
        let node: HtmlNode<'_> = document.node(&id).unwrap();
        match node.kind() {
            HtmlNodeKind::Text(value) | HtmlNodeKind::Comment(value) => found.push(value),
            _ => {}
        }
        let mut children: Vec<_> = node.children().collect();
        children.reverse();
        pending.extend(children);
        if let Some(contents) = node.template_contents() {
            pending.push(contents);
        }
    }
    found
}

fn parse(html: String) -> surgeist_html_parser::HtmlParseReport {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(html), large())
        .parse()
        .unwrap_or_else(|failure| panic!("the document parses: {failure:?}"))
}

/// The bytes the distinct blocks of `texts` keep alive: capacity, and one vector header each.
fn retained_text_bytes(texts: &[DocumentString]) -> usize {
    let mut seen = BTreeSet::new();
    let mut bytes = 0usize;
    for text in texts {
        let block = text.block();
        if seen.insert(block.address()) {
            bytes += block.retained_bytes();
        }
    }
    bytes
}

/// `pairs` foster-parented merges: the text before a table meets the text of the next `style`.
fn foster_merges(pairs: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body><table>a");
    for _ in 0..pairs {
        html.push_str("<style>x</style>b");
    }
    html.push_str("</table></body></html>");
    html
}

/// `count` paragraphs of one short text each, and a comment between them.
fn many_texts(count: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for index in 0..count {
        html.push_str("<p>t");
        html.push_str(&(index % 10).to_string());
        html.push_str("</p><!--c-->");
    }
    html.push_str("</body></html>");
    html
}

#[test]
fn every_text_and_comment_of_a_parse_borrows_the_one_block_of_the_parse() {
    let report = parse(many_texts(50));
    let texts = texts(&report);
    assert_eq!(texts.len(), 100);
    let blocks: BTreeSet<usize> = texts
        .iter()
        .map(|text| text.borrowed_block().map(|block| block.address()))
        .collect::<Option<_>>()
        .expect("every text of the parse borrows its block");
    assert_eq!(blocks.len(), 1, "one block for the parse");
}

#[test]
fn the_text_a_parse_retains_is_at_most_two_bytes_per_unit_plus_a_bounded_slack() {
    for count in [100_usize, 1000, 4000] {
        let report = parse(many_texts(count));
        let texts = texts(&report);
        let units: usize = texts.iter().map(DocumentString::len).sum();
        let bytes = retained_text_bytes(&texts);
        // The block is packed when more than a quarter of it is spare, so at most a third of the
        // units are slack; the slack that is constant is one vector header and a small reserve.
        let bound = 2 * units + (2 * units).div_ceil(3) + 1024;
        assert!(
            bytes <= bound,
            "{count} texts of {units} units retain {bytes} bytes, more than {bound}"
        );
    }
}

#[test]
fn the_text_merged_around_a_table_keeps_the_same_bound() {
    for pairs in [1000_usize, 2000, 4000, 8000] {
        let report = parse(foster_merges(pairs));
        let texts = texts(&report);
        let units: usize = texts.iter().map(DocumentString::len).sum();
        assert_eq!(
            units,
            pairs + 1 + pairs,
            "the merged text and the style texts"
        );
        // The text before the table is `a` and the `b` of each merge, in the order the merges
        // appended them, and each style holds `x`.
        let units_of = |text: &DocumentString| String::from_utf16(text.code_units()).unwrap();
        let (merged, styles) = texts.split_first().unwrap();
        assert_eq!(units_of(merged), format!("a{}", "b".repeat(pairs)));
        assert_eq!(styles.len(), pairs);
        assert!(styles.iter().all(|style| units_of(style) == "x"));
        let bytes = retained_text_bytes(&texts);
        let bound = 2 * units + (2 * units).div_ceil(3) + 1024;
        assert!(
            bytes <= bound,
            "{pairs} merges retain {bytes} bytes of {units} units, more than {bound}"
        );
    }
}

/// The text copy work of a parse: the bytes the work copied beyond those of the same document
/// without the merged text.
fn merge_copy_bytes(pairs: usize) -> u64 {
    let copied = |html: String| parse(html).usage().work(HtmlParseWorkCategory::CopyByte);
    let with = copied(foster_merges(pairs));
    let mut without = String::from("<!DOCTYPE html><html><body><table>");
    for _ in 0..pairs {
        without.push_str("<style>x</style>");
    }
    without.push_str("</table></body></html>");
    with.saturating_sub(copied(without))
}

#[test]
fn the_copy_work_of_merged_text_at_most_doubles_with_the_merges() {
    // One merged unit costs a bounded number of copied bytes however long the text has grown:
    // an append copies two bytes a unit, and a text that is relocated copies at most four units
    // for each unit it holds at the end, so the copied bytes grow by at most 10 percent faster
    // than the merges.
    let copied: Vec<u64> = [1000_usize, 2000, 4000, 8000]
        .into_iter()
        .map(merge_copy_bytes)
        .collect();
    for pair in copied.windows(2) {
        assert!(
            u128::from(pair[1]) * 10 <= u128::from(pair[0]) * 22,
            "copied {} bytes then {} for twice the merges",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn the_allocations_of_a_parse_per_node_do_not_grow_with_the_document() {
    let per_node = |count: usize| {
        let report = parse(many_texts(count));
        let usage = report.usage();
        let nodes = usage.resource(HtmlParseResource::ManagedNodes).current();
        let allocations = usage.work(HtmlParseWorkCategory::ProviderCall);
        (allocations as f64) / (nodes as f64)
    };
    let small = per_node(1000);
    for count in [2000_usize, 4000, 8000] {
        let large = per_node(count);
        assert!(
            large <= small * 1.1,
            "{count} texts: {large} allocations per node against {small}"
        );
    }
}

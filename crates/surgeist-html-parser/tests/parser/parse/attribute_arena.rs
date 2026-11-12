//! The attribute values of one parse: every value is a range of the one block of the parse that
//! also holds its text and comments, whatever the shape of the document.

use crate::parser::support::limits_builder;
use std::collections::BTreeSet;
use surgeist_html::{
    construction::{HtmlNode, HtmlNodeKind},
    values::DocumentString,
};
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};

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

fn parse(html: String) -> surgeist_html_parser::HtmlParseReport {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(html), large())
        .parse()
        .unwrap_or_else(|failure| panic!("the document parses: {failure:?}"))
}

/// The attribute values and the text and comment strings of the parsed document, in document
/// order.
fn strings(
    report: &surgeist_html_parser::HtmlParseReport,
) -> (Vec<DocumentString>, Vec<DocumentString>) {
    let document = report.document().unwrap();
    let (mut values, mut texts) = (Vec::new(), Vec::new());
    let mut pending: Vec<_> = document.root().children().collect();
    pending.reverse();
    while let Some(id) = pending.pop() {
        let node: HtmlNode<'_> = document.node(&id).unwrap();
        match node.kind() {
            HtmlNodeKind::Element(element) => {
                values.extend(element.attributes().iter().map(|a| a.value().clone()));
            }
            HtmlNodeKind::Text(value) | HtmlNodeKind::Comment(value) => texts.push(value),
            _ => {}
        }
        let mut children: Vec<_> = node.children().collect();
        children.reverse();
        pending.extend(children);
        if let Some(contents) = node.template_contents() {
            pending.push(contents);
        }
    }
    (values, texts)
}

/// The addresses of the blocks that `strings` borrow, or `None` when one of them is an owner.
fn blocks(strings: &[DocumentString]) -> Option<BTreeSet<usize>> {
    strings
        .iter()
        .map(|string| string.borrowed_block().map(|block| block.address()))
        .collect()
}

/// `elements` elements of 25 attributes each and a text in each.
fn many_attributes(elements: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for element in 0..elements {
        html.push_str(&format!("<div id=\"e{element}\""));
        for attribute in 0..24_usize {
            html.push_str(&format!(
                " data-a{attribute}=\"value {element} {attribute}\""
            ));
        }
        html.push_str(">content</div>");
    }
    html.push_str("</body></html>");
    html
}

fn attributes_of_one_element(attributes: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body><div");
    for attribute in 0..attributes {
        html.push_str(&format!(" data-a{attribute}=\"value {attribute}\""));
    }
    html.push_str(">content</div><!--c--></body></html>");
    html
}

#[test]
fn every_attribute_value_of_a_parse_borrows_the_one_block_that_holds_its_text() {
    let documents = [
        (many_attributes(40), 40 * 25),
        (attributes_of_one_element(800), 800),
        (
            "<!DOCTYPE html><html a=\"1\"><body b=\"2\"><p c=\"3\">t</p><svg viewBox=\"0 0 1 1\"></svg>\
             <template><i d=\"4\"></i></template><!--x--></body></html>"
                .to_owned(),
            6,
        ),
    ];
    for (html, expected) in documents {
        let report = parse(html);
        let (values, texts) = strings(&report);
        assert_eq!(values.len(), expected);
        let value_blocks = blocks(&values).expect("every attribute value borrows its block");
        let text_blocks = blocks(&texts).expect("every text borrows its block");
        assert_eq!(value_blocks.len(), 1, "one block for the attribute values");
        assert_eq!(value_blocks, text_blocks, "the block of the text");
    }
}

#[test]
fn the_values_of_a_merged_and_a_repeated_attribute_keep_their_exact_units() {
    // The root merges the attributes it lacks from later `html` start tags and keeps its own value
    // of one it has; a duplicate attribute on an element is dropped.
    let report = parse(
        "<!DOCTYPE html><html a=\"1\"><html a=\"2\" b=\"3\"><body c=\"4\" c=\"5\">x</body></html>"
            .to_owned(),
    );
    let (values, _) = strings(&report);
    let units: Vec<String> = values
        .iter()
        .map(|value| value.to_utf8().unwrap())
        .collect();
    assert_eq!(units, ["1", "3", "4"]);
    assert_eq!(blocks(&values).map(|blocks| blocks.len()), Some(1));
}

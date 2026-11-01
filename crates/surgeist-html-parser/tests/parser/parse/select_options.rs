//! Option selectedness after one insertion connects several options.
//!
//! The authority is HTML 877d's selectedness setting algorithm
//! (`references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md`,
//! lines 7932-7964): the last selected option stays selected, and a single
//! select with a display size of one and no selected option selects its first
//! enabled option. The adoption agency moves a `div` out of a `datalist` into
//! the select, connecting its options in one insertion.

use crate::parser::support::limits;
use surgeist_html::*;
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseMetadataStep, HtmlSource};

/// The selectedness of each option, in tree order, keyed by its text.
fn option_selectedness(body: &str) -> Vec<(String, bool)> {
    let literal = format!("<!doctype html>{body}");
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits())
        .parse()
        .unwrap();
    let document = report.document().unwrap();
    let mut found = Vec::new();
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(flags) = record.option_flags() {
                    let text = record
                        .node()
                        .children()
                        .filter_map(|id| match document.node(&id).unwrap().kind() {
                            HtmlNodeKind::Text(text) => text.to_utf8().ok(),
                            _ => None,
                        })
                        .collect::<String>();
                    found.push((text, flags.selectedness()));
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    // Metadata is in registry order and the text is unique per option.
    found.sort();
    found
}

fn expect(body: &str, expected: &[(&str, bool)]) {
    let expected: Vec<(String, bool)> = expected
        .iter()
        .map(|&(text, selected)| (text.to_string(), selected))
        .collect();
    assert_eq!(option_selectedness(body), expected, "{body}");
}

#[test]
fn a_selected_option_connected_with_an_earlier_one_stays_selected() {
    expect(
        "<select><b><datalist><div><option>A<option selected>B</b></select>",
        &[("A", false), ("B", true)],
    );
}

#[test]
fn a_first_enabled_option_is_selected_when_a_subtree_connects_a_later_enabled_one() {
    expect(
        "<select size=1><b><datalist><div><option disabled>A<option>B</b></select>",
        &[("A", false), ("B", true)],
    );
}

#[test]
fn the_last_selected_of_three_connected_options_stays_selected() {
    expect(
        "<select><b><datalist><div><option>A<option>B<option selected>C</b></select>",
        &[("A", false), ("B", false), ("C", true)],
    );
}

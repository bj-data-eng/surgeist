//! How a parse's accounted work, retained bytes, allocations and accounting
//! calls grow with the document.
//!
//! The authority is the linear-cost bound of the parse model: growing the
//! document by a factor grows each measure by at most that factor plus 10
//! percent, which is 2.2 times for a doubling. The expectations
//! come from that bound, not from what the parser reports today. Each family
//! is the shape of one benchmark document, scaled over three doublings from one
//! sixteenth of the benchmark document.
//!
//! A measure that exceeds the bound today is a pending test that names the stage
//! of the construction data model whose change satisfies it.

use super::accounts::last_parse_counts;
use crate::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseResource, HtmlParseWorkCategory, HtmlSource,
};
use std::fmt::Write as _;

/// Limits that no measured document reaches.
fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(1 << 30)
        .buffered_utf16_units(1 << 30)
        .token_attributes(1 << 30)
        .auxiliary_entries(1 << 30)
        .active_formatting_entries(1 << 30)
        .managed_nodes(1 << 30)
        .managed_edges(1 << 30)
        .managed_attributes(1 << 30)
        .managed_utf16_units(1 << 30)
        .source_map_entries(1 << 30)
        .source_map_payload_units(1 << 30)
        .diagnostics(1 << 30)
        .diagnostic_payload_units(1 << 30)
        .context_effects(1 << 30)
        .open_elements(1 << 30)
        .tree_depth(1 << 30)
        .managed_capacity_bytes(1 << 40)
        .peak_transient_bytes(1 << 40)
        .atomic_layout_bytes(1 << 40)
        .work_credits(u64::MAX)
        .finish()
        .unwrap()
}

/// What one parse of a document cost.
#[derive(Clone, Copy, Debug)]
struct Cost {
    /// The size of the document in bytes.
    bytes: u64,
    /// The work credit total of the parse.
    credits: u64,
    /// The managed capacity bytes the report retains.
    retained_bytes: u64,
    /// The provider requests the parse charged, one per allocation it made.
    allocations: u64,
    /// The bytes the parse's work copied.
    copied_bytes: u64,
    /// The nodes the parse retains.
    nodes: u64,
    /// The text code units the parse retains.
    units: u64,
    /// The batches the parse admitted through its accounts.
    batches: u64,
    /// The calls the parse made into a ledger operation.
    ledger_calls: u64,
}

fn cost(html: String) -> Cost {
    cost_with_detail(html, crate::DetailSet::NONE)
}

/// What one parse of a document cost when it records `detail`.
fn cost_with_detail(html: String, detail: crate::DetailSet) -> Cost {
    let bytes = html.len() as u64;
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(html), limits())
        .with_detail(detail)
        .unwrap_or_else(|unavailable| panic!("the detail is available: {unavailable:?}"))
        .parse()
        .unwrap_or_else(|failure| panic!("the document parses within the limits: {failure:?}"));
    let usage = report.usage().clone();
    drop(report);
    let counts = last_parse_counts();
    Cost {
        bytes,
        credits: usage.resource(HtmlParseResource::WorkCredits).current(),
        retained_bytes: usage.actual_capacity_bytes,
        allocations: usage.work(HtmlParseWorkCategory::ProviderCall),
        copied_bytes: usage.work(HtmlParseWorkCategory::CopyByte),
        nodes: usage.resource(HtmlParseResource::ManagedNodes).current(),
        units: usage
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current(),
        batches: counts.batches,
        ledger_calls: counts.ledger_calls,
    }
}

/// The costs of a family at each scale, from the smallest, each scale double
/// the one before.
fn costs(scales: [usize; 4], document: fn(usize) -> String) -> Vec<Cost> {
    scales.iter().map(|&scale| cost(document(scale))).collect()
}

/// Asserts that `measure` grows at most 10 percent faster than the document
/// from each scale of a family to the next, which is at most 2.2 times for a
/// doubling.
fn assert_at_most_doubles(family: &str, name: &str, costs: &[Cost], measure: fn(&Cost) -> u64) {
    for pair in costs.windows(2) {
        let (small, large) = (measure(&pair[0]), measure(&pair[1]));
        assert!(
            u128::from(large) * u128::from(pair[0].bytes) * 10
                <= u128::from(small) * u128::from(pair[1].bytes) * 11,
            "{family}: {name} grew from {small} at {} bytes to {large} at {} bytes, \
             more than 10 percent faster than the document",
            pair[0].bytes,
            pair[1].bytes,
        );
    }
}

fn assert_family(family: &str, costs: &[Cost]) {
    eprintln!("{family}: {costs:#?}");
    assert_at_most_doubles(family, "work credits", costs, |cost| cost.credits);
    assert_at_most_doubles(family, "retained bytes", costs, |cost| cost.retained_bytes);
    assert_at_most_doubles(family, "allocations", costs, |cost| cost.allocations);
    assert_at_most_doubles(family, "accounting batches", costs, |cost| cost.batches);
}

fn ordinary(sections: usize) -> String {
    let mut html = String::from(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <title>Ordinary</title><style>body{margin:0}</style></head><body>",
    );
    for section in 0..sections {
        let _ = write!(
            html,
            "<section id=\"s{section}\" class=\"part\"><h2>Section {section}</h2>\
             <p>Paragraph with <b>bold</b>, <i>italic</i> and <a href=\"/p/{section}\">a link</a>. \
             Entities &amp; &lt;markup&gt; &copy; stay text.</p>\
             <ul>"
        );
        for item in 0..6_usize {
            let _ = write!(
                html,
                "<li class=\"item\"><span>Item {item}</span> of section {section}</li>"
            );
        }
        html.push_str("</ul><p>Closing paragraph of the section.</p></section>");
    }
    html.push_str("</body></html>");
    html
}

fn deeply_nested(depth: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for _ in 0..depth {
        html.push_str("<div>");
    }
    html.push_str("<p>Leaf</p>");
    for _ in 0..depth {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");
    html
}

fn many_attributes(elements: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for element in 0..elements {
        let _ = write!(html, "<div id=\"e{element}\"");
        for attribute in 0..24_usize {
            let _ = write!(html, " data-a{attribute}=\"value {element} {attribute}\"");
        }
        html.push_str(">content</div>");
    }
    html.push_str("</body></html>");
    html
}

fn attributes_of_one_element(attributes: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body><div");
    for attribute in 0..attributes {
        let _ = write!(html, " data-a{attribute}=\"value {attribute}\"");
    }
    html.push_str(">content</div></body></html>");
    html
}

/// `merges` repeated `html` start tags, each carrying one attribute the root does not have yet.
fn attributes_merged_into_the_root(merges: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html a=\"v\">");
    for merge in 0..merges {
        let _ = write!(html, "<html m{merge}=\"v\">");
    }
    html.push_str("<body>content</body></html>");
    html
}

/// `elements` elements of eight attributes each, then as many `html` start tags, each carrying
/// one attribute the root does not have yet. The origins of the root's attributes are recorded
/// with those of every element before it in the document.
fn attributes_merged_into_the_root_after_many_elements(elements: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html a=\"v\"><body>");
    for element in 0..elements {
        let _ = write!(html, "<div id=\"e{element}\"");
        for attribute in 0..8_usize {
            let _ = write!(html, " data-a{attribute}=\"value {element} {attribute}\"");
        }
        html.push_str(">content</div>");
    }
    for merge in 0..elements {
        let _ = write!(html, "<html m{merge}=\"v\">");
    }
    html.push_str("</body></html>");
    html
}

fn table_row(html: &mut String, row: usize) {
    html.push_str("<tr>");
    for column in 0..16_usize {
        if column == 3 && row.is_multiple_of(7) {
            let _ = write!(html, "<td colspan=\"2\">{row}:{column}</td>");
        } else {
            let _ = write!(html, "<td>{row}:{column}</td>");
        }
    }
    html.push_str("</tr>");
}

/// `tables` tables of 60 rows.
fn table_heavy(tables: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for table in 0..tables {
        let _ = write!(
            html,
            "<table id=\"t{table}\"><caption>Table {table}</caption><thead><tr>"
        );
        for column in 0..16_usize {
            let _ = write!(html, "<th scope=\"col\">C{column}</th>");
        }
        html.push_str("</tr></thead><tbody>");
        for row in 0..60_usize {
            table_row(&mut html, row);
        }
        html.push_str("</tbody></table>");
    }
    html.push_str("</body></html>");
    html
}

/// One table of `rows` rows.
fn table_rows(rows: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body><table><tbody>");
    for row in 0..rows {
        table_row(&mut html, row);
    }
    html.push_str("</tbody></table></body></html>");
    html
}

fn many_diagnostics(repeats: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for index in 0..repeats {
        let _ = write!(
            html,
            "</span><p class=\"a\" class=\"b\">text &bogus; &#x110000; {index}</i>\
             <table>stray {index}<tr><td>cell</table></b>"
        );
    }
    html.push_str("</body></html>");
    html
}

/// `items` siblings of one parent.
fn wide(items: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body><ul>");
    for item in 0..items {
        let _ = write!(html, "<li>{item}</li>");
    }
    html.push_str("</ul></body></html>");
    html
}

/// One text node of `units` repeated words and character references.
fn long_text(units: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body><p>");
    for _ in 0..units {
        html.push_str("word &amp; word &lt; ");
    }
    html.push_str("</p></body></html>");
    html
}

/// Formatting elements left open by a paragraph that ends, so the next
/// paragraph reconstructs them. The elements repeat exactly, so the list of
/// active formatting elements keeps at most three of each.
fn unclosed_formatting(repeats: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for _ in 0..repeats {
        html.push_str("<p><b>x<i>y</p>");
    }
    html.push_str("</body></html>");
    html
}

/// Misnested formatting that the adoption agency algorithm repairs.
fn misnested_formatting(repeats: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for _ in 0..repeats {
        html.push_str("<b><i>x</b>y</i>z");
    }
    html.push_str("</body></html>");
    html
}

/// One attribute value of `units` code units.
fn long_attribute_value(units: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body><div title=\"");
    for _ in 0..units {
        html.push_str("value ");
    }
    html.push_str("\">x</div></body></html>");
    html
}

#[test]
#[ignore = "retained bytes of a wide parent grow faster than the document; the node table stage (S4) of #2228 satisfies it"]
fn documents_of_wide_parents_grow_linearly() {
    assert_family("wide parent", &costs([1_000, 2_000, 4_000, 8_000], wide));
}

#[test]
fn long_text_grows_linearly() {
    assert_family("long text", &costs([500, 1_000, 2_000, 4_000], long_text));
}

#[test]
fn reconstructed_formatting_grows_linearly() {
    assert_family(
        "reconstructed formatting",
        &costs([250, 500, 1_000, 2_000], unclosed_formatting),
    );
}

#[test]
fn adopted_formatting_grows_linearly() {
    assert_family(
        "adopted formatting",
        &costs([250, 500, 1_000, 2_000], misnested_formatting),
    );
}

#[test]
fn a_long_attribute_value_grows_linearly() {
    assert_family(
        "long attribute value",
        &costs([500, 1_000, 2_000, 4_000], long_attribute_value),
    );
}

#[test]
fn ordinary_documents_grow_linearly() {
    assert_family("ordinary", &costs([15, 30, 60, 120], ordinary));
}

#[test]
fn nested_documents_grow_linearly() {
    assert_family(
        "deeply nested",
        &costs([250, 500, 1_000, 2_000], deeply_nested),
    );
}

#[test]
fn documents_of_many_attributed_elements_grow_linearly() {
    assert_family(
        "many attributes",
        &costs([63, 125, 250, 500], many_attributes),
    );
}

#[test]
fn an_element_with_many_attributes_grows_linearly() {
    assert_family(
        "attributes of one element",
        &costs([100, 200, 400, 800], attributes_of_one_element),
    );
}

/// Each repeated `html` start tag appends one attribute to the same element. The copy that grows
/// the list must not depend on how many attributes the element already has.
#[test]
fn attributes_merged_into_the_root_one_at_a_time_grow_linearly() {
    assert_family(
        "attributes merged into the root",
        &costs([100, 200, 400, 800], attributes_merged_into_the_root),
    );
}

/// With the origin graph recorded, merging an attribute into an element moves the origin
/// records of that element alone, however many elements follow it in the document.
#[cfg(feature = "detail")]
#[test]
fn attributes_merged_into_the_root_grow_linearly_when_origins_are_recorded() {
    let costs: Vec<Cost> = [100, 200, 400, 800]
        .iter()
        .map(|&elements| {
            cost_with_detail(
                attributes_merged_into_the_root_after_many_elements(elements),
                crate::DetailSet::ALL,
            )
        })
        .collect();
    assert_family("attributes merged into the root, origins recorded", &costs);
    assert_at_most_doubles(
        "attributes merged into the root, origins recorded",
        "copied bytes",
        &costs,
        |cost| cost.copied_bytes,
    );
}

#[test]
fn documents_of_many_tables_grow_linearly() {
    assert_family("table-heavy", &costs([1, 2, 4, 8], table_heavy));
}

#[test]
fn a_table_of_many_rows_grows_linearly() {
    assert_family("table rows", &costs([60, 120, 240, 480], table_rows));
}

#[test]
fn documents_with_many_diagnostics_grow_linearly() {
    assert_family(
        "many diagnostics",
        &costs([150, 300, 600, 1_200], many_diagnostics),
    );
}

/// A published tree retains at most 28 bytes per node and 2 bytes per retained
/// text code unit, the bound the model sets for documents without attributes.
#[test]
#[ignore = "retained bytes are far above the bound per node; the node table stage (S4) of #2228 satisfies it"]
fn retained_bytes_stay_within_the_bytes_per_node_bound() {
    for items in [1_000, 4_000] {
        let cost = cost(wide(items));
        let bound = 28 * cost.nodes + 2 * cost.units;
        assert!(
            cost.retained_bytes <= bound,
            "{items} items: {} retained bytes for {} nodes and {} units, bound {bound}",
            cost.retained_bytes,
            cost.nodes,
            cost.units,
        );
    }
}

/// An owned parse charges plain counters and calls no ledger operation after its
/// setup. The counter records calls on a shared operation, so this test guards
/// the owned parse's choice of the meter, not calls made inside the meter.
#[test]
fn an_owned_parse_makes_no_ledger_call() {
    let cost = cost(ordinary(8));
    assert!(cost.batches > 0, "the parse admitted batches");
    assert_eq!(cost.ledger_calls, 0);
}

/// A parse in a host's operation charges that operation, one call per batch at
/// least.
#[test]
fn a_hosted_parse_charges_the_host_operation() {
    use surgeist_html::accounting::{HtmlHostScope, HtmlLimits};
    let ledger = surgeist_ledger::Operation::new("host", HtmlLimits::unlimited(), 8)
        .unwrap()
        .share();
    let root = ledger.root();
    let accounting = crate::HtmlParseAccounting::host(HtmlHostScope::new(ledger, root));
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(ordinary(2)), limits())
        .with_accounting(accounting)
        .parse()
        .unwrap_or_else(|failure| panic!("the document parses: {failure:?}"));
    drop(report);
    let counts = last_parse_counts();
    assert!(counts.batches > 0, "the parse admitted batches");
    assert!(counts.ledger_calls >= counts.batches);
}

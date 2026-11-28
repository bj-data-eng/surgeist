//! What a parse retains for the cached tree roots of its records.
//!
//! The cache of a record's root is build-only state of the questions the parse asks of a tree.
//! A node's join cell does not carry it: the cells of the roots are a bank beside the node-join
//! bank, and a published parse retains none of it.
use super::plain_elements::with_plain_elements;
use super::*;
use std::mem::size_of;
use std::num::NonZeroU32;

/// The cell of a node's joins holds one slot of a registry row for each kind of row a node joins,
/// and the slot of its `textarea` state: a registry slot plus one in 32 bits each. The oracle is
/// that count, from the kinds the joins name, and the width of a slot cell.
#[test]
fn a_node_join_cell_holds_only_the_slots_of_the_rows_it_joins() {
    let slot = size_of::<Option<NonZeroU32>>();
    assert_eq!(slot, 4);
    let kinds = Join::Details as usize + 1;
    assert_eq!(size_of::<NodeJoins>(), (kinds + 1) * slot);
    assert_eq!(size_of::<NodeJoins>(), 60);
}

/// A document of 1,000 list items, as the growth test of the retained bytes parses it.
fn wide(items: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><body><ul>");
    for item in 0..items {
        html.push_str(&format!("<li>{item}</li>"));
    }
    html.push_str("</ul></body></html>");
    html
}

/// A published parse retains the node-join bank and no bank of cached roots: the roots a parse
/// asked for are build-only, so the retained cells of a node are its joins. The oracle is the
/// doubling rule of the bank, which holds the smallest power of two at least the records, and the
/// 60 bytes of a join cell.
#[test]
fn a_published_parse_retains_only_the_node_join_bank_for_its_records() {
    let source = wide(1000);
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(source.as_str()),
        g2_d2::corpus_limits(),
    );
    let report = request.parse().unwrap();
    let nodes = report
        .usage()
        .resource(HtmlParseResource::ManagedNodes)
        .current();
    assert_eq!(nodes, 2006);
    let capacity = report.history().storage().payloads.node_joins.capacity();
    assert_eq!(capacity as u64, nodes.next_power_of_two());
    assert_eq!(
        capacity * size_of::<NodeJoins>(),
        60 * nodes.next_power_of_two() as usize
    );
}

/// While a parse builds, the bank of cached roots holds one cell for each record and has the
/// capacity of the node-join bank, whatever number of records the parse has created: the two
/// banks are made together and grow at the same creation, to the same capacity. The oracle is the
/// records the session holds, and the node-join bank, which is a bank of one cell for each.
#[test]
fn the_root_bank_has_a_cell_for_each_record_and_the_capacity_of_the_node_join_bank() {
    for elements in [0, 1, 2, 3, 4, 5, 8, 9, 63, 64, 65] {
        with_plain_elements(elements, |_, session| {
            let storage = session.active().unwrap();
            let records = storage.construction.len();
            // The session holds the records its fixture made and one for each element.
            assert!(records > elements, "{elements} elements");
            let payloads = &storage.payloads;
            assert_eq!(payloads.node_joins.len(), records, "{elements} elements");
            assert_eq!(payloads.tree_roots.len(), records, "{elements} elements");
            assert_eq!(
                payloads.tree_roots.capacity(),
                payloads.node_joins.capacity(),
                "{elements} elements"
            );
        });
    }
}

/// The bank is freed when the parse is published: a published parse retains no cell and no
/// capacity for the roots it asked for.
#[test]
fn a_published_parse_holds_no_root_bank() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<p><b>a</b></p><div><i>b</i></div>"),
        g2_d2::corpus_limits(),
    );
    let report = request.parse().unwrap();
    let payloads = &report.history().storage().payloads;
    assert!(!payloads.node_joins.is_empty());
    assert_eq!(payloads.tree_roots.len(), 0);
    assert_eq!(payloads.tree_roots.capacity(), 0);
}

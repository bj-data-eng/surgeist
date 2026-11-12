//! The charge of a name of the old and new union that the three meta name sites make, with a name
//! that borrows a block. No public input reaches these sites with a borrower name today, because
//! the names they see are the DOM's decoded owners.
use super::*;
use surgeist_html::values::TextBlock;

fn backing() -> Backing {
    Backing {
        bytes: 0,
        units: 0,
        dom_shared_bytes: 0,
        dom_shared_units: 0,
        outputs: 0,
        current: 0,
        runtime_limit: usize::MAX,
        limits: BackingLimits {
            bytes: usize::MAX,
            units: usize::MAX,
            outputs: usize::MAX,
            resources: [
                surgeist_dom::DomResource::MetaAtomicRetainedBytes,
                surgeist_dom::DomResource::MetaAtomicRetainedUnits,
                surgeist_dom::DomResource::MetaAtomicOutputs,
            ],
        },
    }
}

fn block() -> TextBlock {
    let mut units = Vec::with_capacity(512);
    units.extend([97_u16, 98, 99, 100, 101, 102]);
    TextBlock::from_utf16(units)
}

/// The bytes of one name occurrence of `len` units, and of one block.
fn occurrence(len: usize) -> usize {
    size_of::<Vec<u16>>() + len * size_of::<u16>()
}

#[test]
fn a_borrower_is_charged_its_length_and_its_block_once_and_dom_sharing_follows_the_dom_occurrence()
{
    let block = block();
    let name = block.string(0..3).unwrap();
    let block_bytes = block.retained_bytes();
    // Not held by any name of the union, and no DOM occurrence holds the name or its block.
    let mut charged = backing();
    charged
        .retain_union_name(&name, false, false, false)
        .unwrap();
    assert_eq!(charged.bytes, occurrence(3) + block_bytes);
    assert_eq!(charged.units, 3 + block.capacity());
    assert_eq!((charged.dom_shared_bytes, charged.dom_shared_units), (0, 0));
    // A name of the union holds the block already: the occurrence alone.
    let mut held = backing();
    held.retain_union_name(&name, false, true, false).unwrap();
    assert_eq!((held.bytes, held.units), (occurrence(3), 3));
    // A DOM occurrence holds the name and the block: both are DOM-shared.
    let mut dom = backing();
    dom.retain_union_name(&name, true, false, true).unwrap();
    assert_eq!(dom.bytes, occurrence(3) + block_bytes);
    assert_eq!(dom.units, 3 + block.capacity());
    assert_eq!(
        (dom.dom_shared_bytes, dom.dom_shared_units),
        (dom.bytes, dom.units)
    );
    // A DOM occurrence holds the name and not its block: only the occurrence is DOM-shared.
    let mut mixed = backing();
    mixed.retain_union_name(&name, true, false, false).unwrap();
    assert_eq!(
        (mixed.dom_shared_bytes, mixed.dom_shared_units),
        (occurrence(3), 3)
    );
}

#[test]
fn an_owner_is_charged_its_capacity_and_no_block() {
    let owner = DocumentString::from_utf16(Vec::with_capacity(40));
    let mut charged = backing();
    charged
        .retain_union_name(&owner, false, false, false)
        .unwrap();
    assert_eq!(charged.bytes, occurrence(owner.retained_units()));
    assert_eq!(charged.units, owner.retained_units());
}

/// The `content` values of the default-style `meta` elements of a real parse of `source`, in
/// document order.
fn parsed_contents(source: &str) -> Vec<DocumentString> {
    use surgeist_html::construction::HtmlNodeKind;
    use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(source),
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
            .open_elements(100)
            .tree_depth(100_000)
            .managed_capacity_bytes(100_000_000)
            .peak_transient_bytes(100_000_000)
            .atomic_layout_bytes(100_000_000)
            .work_credits(100_000_000)
            .finish()
            .unwrap(),
    )
    .parse()
    .unwrap();
    let document = report.document().unwrap();
    let mut pending = vec![document.root().id()];
    let mut contents = Vec::new();
    while let Some(id) = pending.pop() {
        let node = document.node(&id).unwrap();
        if let HtmlNodeKind::Element(element) = node.kind()
            && element.name().local_name() == &DocumentString::from("meta")
            && let Some(content) = element
                .attributes()
                .iter()
                .find(|attribute| attribute.name().local_name() == &DocumentString::from("content"))
        {
            contents.push(content.value().clone());
        }
        pending.extend(node.children().rev());
    }
    contents
}

#[test]
fn two_default_style_names_of_one_parse_are_charged_their_lengths_and_the_one_block() {
    let contents = parsed_contents(
        "<!doctype html><meta http-equiv=default-style content=chosen>\
         <meta http-equiv=default-style content=later-set>",
    );
    let [first, second] = contents.as_slice() else {
        panic!("two meta elements")
    };
    assert_eq!(
        (first.len(), second.len()),
        ("chosen".len(), "later-set".len())
    );
    // The site retains each name of the union with the block it borrows counted once: the first
    // name finds no name of the union holding its block, the second finds the first.
    let mut backing = backing();
    let mut retained: Vec<&DocumentString> = Vec::new();
    for name in [first, second] {
        let block_held = retained
            .iter()
            .any(|old| sheet_sets::shares_block(name, old));
        backing
            .retain_union_name(name, false, block_held, false)
            .unwrap();
        retained.push(name);
    }
    // The charge is the two lengths and one block of the parse, which is the block of either name.
    let block = first.block();
    assert_eq!(
        backing.units,
        first.len() + second.len() + block.capacity(),
        "the values borrow the block of the parse and the charge counts it once"
    );
    assert_eq!(
        backing.bytes,
        occurrence(first.len()) + occurrence(second.len()) + block.retained_bytes()
    );
    assert!(first.block().shares_allocation_with(second.block()));
}

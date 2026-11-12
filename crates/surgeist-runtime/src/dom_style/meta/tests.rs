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

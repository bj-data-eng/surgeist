//! The custody a meta occurrence charges for a name of its output that borrows a block. No public
//! input reaches the loop with a borrower name today, because the names it sees are the DOM's
//! decoded owners.
use super::*;
use surgeist_dom::DocumentString;
use surgeist_html::values::TextBlock;

fn new_custody() -> Custody {
    Custody {
        bytes: 0,
        shared: 0,
        units: 0,
        current: 0,
        maximum: usize::MAX,
        outputs: 1,
    }
}

fn new_surviving() -> Surviving {
    Surviving { bytes: 0, units: 0 }
}

fn block() -> TextBlock {
    let mut units = Vec::with_capacity(512);
    units.extend([97_u16, 98, 99, 100, 101, 102]);
    TextBlock::from_utf16(units)
}

fn occurrence(len: usize) -> usize {
    size_of::<Vec<u16>>() + len * size_of::<u16>()
}

const FRESH: NameHeld = NameHeld {
    existing: false,
    block: false,
    in_content: false,
};

#[test]
fn a_name_the_output_keeps_alone_is_charged_by_length_and_its_block_to_the_custody_and_the_survivors()
 {
    let block = block();
    let name = block.string(0..3).unwrap();
    let (mut custody, mut surviving) = (new_custody(), new_surviving());
    charge_name(&mut custody, &mut surviving, &name, FRESH).unwrap();
    let both = occurrence(3) + block.retained_bytes();
    // The runtime union and the output's closing custody each hold the name and its block.
    assert_eq!(custody.bytes, 2 * both);
    assert_eq!(custody.units, 3 + block.capacity());
    assert_eq!(custody.shared, 0);
    assert_eq!(
        (surviving.bytes, surviving.units),
        (both, 3 + block.capacity())
    );
}

#[test]
fn a_name_that_is_the_content_of_the_occurrence_is_dom_shared_with_its_block() {
    let block = block();
    let name = block.string(0..3).unwrap();
    let (mut custody, mut surviving) = (new_custody(), new_surviving());
    let held = NameHeld {
        in_content: true,
        ..FRESH
    };
    charge_name(&mut custody, &mut surviving, &name, held).unwrap();
    let both = occurrence(3) + block.retained_bytes();
    assert_eq!(
        (custody.bytes, custody.shared, custody.units),
        (both, both, 0)
    );
    assert_eq!((surviving.bytes, surviving.units), (0, 0));
}

#[test]
fn a_name_the_old_state_aliases_or_whose_block_is_held_is_charged_less() {
    let block = block();
    let name = block.string(0..3).unwrap();
    // The block is held: the occurrence alone, to the custody twice and to the survivors once.
    let (mut custody, mut surviving) = (new_custody(), new_surviving());
    let held = NameHeld {
        block: true,
        ..FRESH
    };
    charge_name(&mut custody, &mut surviving, &name, held).unwrap();
    assert_eq!(custody.bytes, 2 * occurrence(3));
    assert_eq!(custody.units, 3);
    assert_eq!((surviving.bytes, surviving.units), (occurrence(3), 3));
    // An old name aliases it, and it is the occurrence's content: nothing is charged but the block.
    let (mut custody, mut surviving) = (new_custody(), new_surviving());
    let held = NameHeld {
        existing: true,
        block: false,
        in_content: true,
    };
    charge_name(&mut custody, &mut surviving, &name, held).unwrap();
    assert_eq!(custody.bytes, block.retained_bytes());
    assert_eq!(custody.shared, block.retained_bytes());
}

#[test]
fn an_owner_name_is_charged_its_capacity_and_no_block() {
    let owner = DocumentString::from_utf16(Vec::with_capacity(40));
    let capacity = owner.retained_units();
    let (mut custody, mut surviving) = (new_custody(), new_surviving());
    charge_name(&mut custody, &mut surviving, &owner, FRESH).unwrap();
    assert_eq!(custody.bytes, 2 * occurrence(capacity));
    assert_eq!(custody.units, capacity);
    assert_eq!(
        (surviving.bytes, surviving.units),
        (occurrence(capacity), capacity)
    );
}

//! The table of blocks an owner has accounted for: lookup by block, release when no string holds
//! the block, and an address that no other block can take while its entry exists.
use surgeist_html::values::{DocumentString, TextBlock, TextBlockTable, TextBlockTableError};

fn block(text: &str) -> TextBlock {
    TextBlock::from_utf16(text.encode_utf16().collect::<Vec<_>>())
}

#[test]
fn a_listed_block_is_found_through_every_string_that_borrows_it() {
    let listed = block("abcdef");
    let other = block("abcdef");
    let mut table = TextBlockTable::new();
    assert!(table.is_empty());
    assert_eq!(table.insert(&listed, 7_u32), Ok(()));
    assert_eq!(table.len(), 1);
    let first = listed.string(0..3).unwrap();
    let second = listed.string(3..6).unwrap();
    for string in [&first, &second] {
        assert_eq!(table.find(string.borrowed_block().unwrap()), Some(&7));
    }
    assert_eq!(table.find(&other), None);
    assert_eq!(
        table.insert(first.borrowed_block().unwrap(), 8),
        Err(TextBlockTableError::Present)
    );
    assert_eq!(table.find(&listed), Some(&7));
    assert_eq!(table.remove(&listed), Some(7));
    assert!(table.is_empty());
    assert_eq!(table.remove(&listed), None);
}

#[test]
fn a_swept_table_releases_only_the_blocks_no_string_holds() {
    let kept = block("kept");
    let dropped = block("dropped");
    let borrower = dropped.string(1..3).unwrap();
    let mut table = TextBlockTable::new();
    table.insert(&kept, "kept").unwrap();
    table.insert(&dropped, "dropped").unwrap();
    let mut released = Vec::new();
    table.sweep(|value| released.push(value));
    assert_eq!(table.len(), 2);
    drop(dropped);
    table.sweep(|value| released.push(value));
    assert_eq!(table.len(), 2, "a borrower holds it");
    drop(borrower);
    table.sweep(|value| released.push(value));
    assert_eq!(released, ["dropped"]);
    assert_eq!(table.len(), 1);
    assert_eq!(table.find(&kept), Some(&"kept"));
}

#[test]
fn an_entry_keeps_the_address_of_a_freed_block_from_every_block_made_after_it() {
    let listed = block("freed");
    let address = listed.address();
    let mut table = TextBlockTable::new();
    table.insert(&listed, ()).unwrap();
    drop(listed);
    // The allocator reuses a freed slot of one size within a few allocations, so this many blocks
    // of the layout of the freed one would meet its address were it not held.
    let later: Vec<TextBlock> = (0..4096).map(|_| block("freed")).collect();
    assert!(later.iter().all(|block| block.address() != address));
    for block in &later {
        assert!(table.find(block).is_none());
    }
    table.sweep(drop);
    assert!(table.is_empty());
}

#[test]
fn an_owner_string_has_no_borrowed_block_and_an_entry_costs_its_size() {
    assert!(DocumentString::from("abc").borrowed_block().is_none());
    assert_eq!(
        TextBlockTable::<u64>::ENTRY_BYTES,
        size_of::<usize>() + size_of::<std::sync::Weak<Vec<u16>>>() + size_of::<u64>()
    );
}

#[test]
fn an_insertion_moves_the_entries_after_it_in_address_order() {
    let blocks: Vec<TextBlock> = (0..5).map(|_| block("x")).collect();
    let mut sorted: Vec<&TextBlock> = blocks.iter().collect();
    sorted.sort_by_key(|block| block.address());
    let mut table = TextBlockTable::new();
    // Listed from the highest address to the lowest, each insertion moves every entry listed.
    for (listed, block) in sorted.iter().rev().enumerate() {
        assert_eq!(table.shifted_by_insert(block), listed);
        table.insert(block, ()).unwrap();
    }
    assert_eq!(
        table.shifted_by_insert(sorted[0]),
        5,
        "a listed block counts itself"
    );
}

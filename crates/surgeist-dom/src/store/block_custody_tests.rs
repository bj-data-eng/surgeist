//! A block that several borrowed strings index is charged to the store once.
use super::*;
use crate::DomNameLimits;
use surgeist_html::values::TextBlock;

fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            32,
            65_536,
            128,
            0,
            0,
            64,
            1_000_000,
            DomNameLimits::new(16_777_216, 4096, 1_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn total(dom: &Dom) -> custody::Usage {
    dom.custody_registry.ledger().total()
}

fn document(dom: &mut Dom) -> crate::DocumentId {
    let mut batch = crate::DomBatch::new();
    let document = batch.create_document();
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    done.document(&document).unwrap()
}

/// The custody total of a store whose document holds one text node for each of `values`, made
/// in one batch so that the banks of two such stores grow alike.
fn total_with_texts(values: &[DocumentString]) -> custody::Usage {
    let mut dom = dom();
    let document = document(&mut dom);
    let mut batch = crate::DomBatch::new();
    for value in values {
        batch.create_text(document, value.clone());
    }
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    total(&dom)
}

fn spare_block() -> TextBlock {
    let mut units = Vec::with_capacity(512);
    units.extend([97_u16, 98, 99, 100, 101, 102, 103, 104]);
    TextBlock::from_utf16(units)
}

fn exact_owner() -> DocumentString {
    let mut units = Vec::new();
    units.try_reserve_exact(4).unwrap();
    units.extend([97_u16, 98, 99, 100]);
    assert_eq!(units.capacity(), 4);
    DocumentString::from_utf16(units)
}

#[test]
fn borrowers_of_one_block_are_charged_by_length_and_the_block_once() {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let borrowed = [
        block.string(0..4).unwrap(),
        block.string(4..8).unwrap(),
        block.string(2..6).unwrap(),
    ];
    let owners = [exact_owner(), exact_owner(), exact_owner()];
    let (shared, owned) = (total_with_texts(&borrowed), total_with_texts(&owners));
    // The block adds its capacity in units once, whatever the number of borrowers.
    assert_eq!(shared.units(), owned.units() + block.capacity());
    // Its bytes are charged once, not at the capacity of the block for each occurrence.
    let block_bytes = size_of::<Vec<u16>>() + block.capacity() * size_of::<u16>();
    assert!(
        shared.bytes() < owned.bytes() + 2 * block_bytes,
        "three borrowers charged {} bytes more than three owners for a block of {block_bytes}",
        shared.bytes() - owned.bytes()
    );
}

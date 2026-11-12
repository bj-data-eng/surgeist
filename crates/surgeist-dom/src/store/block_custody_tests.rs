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

/// The custody total of a store whose document holds one text node for each of `values`, and the
/// number of blocks it charges. The bytes of two stores built alike differ by the sizes of tables
/// that depend on the process hash key, so only units are compared between stores.
fn total_with_texts(values: &[DocumentString]) -> (custody::Usage, usize) {
    let mut dom = dom();
    let document = document(&mut dom);
    let mut batch = crate::DomBatch::new();
    for value in values {
        batch.create_text(document, value.clone());
    }
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (total(&dom), dom.custody_registry.listed_blocks())
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
    let ((shared, listed), (owned, none)) =
        (total_with_texts(&borrowed), total_with_texts(&owners));
    // The block adds its capacity in units once, whatever the number of borrowers.
    assert_eq!(shared.units(), owned.units() + block.capacity());
    assert_eq!((listed, none), (1, 0));
}

/// The bytes the debit of `block` holds in the registry.
fn block_usage(block: &TextBlock) -> custody::Usage {
    custody::Usage::new(block.retained_bytes(), block.capacity())
}

/// The bytes the registry charges for the capacity of its table of blocks.
fn table_usage(dom: &Dom) -> custody::Usage {
    custody::Usage::new(
        dom.custody_registry.listed_capacity() * bank::CustodyRegistry::TABLE_ENTRY_BYTES,
        0,
    )
}

const NO_STEP: fn() -> Result<(), crate::DomError> = || Ok(());

#[test]
fn the_debit_of_a_block_holds_the_block_and_the_table_the_room_of_its_entry() {
    let dom = dom();
    let transaction = block_custody::BlockAdoptions::new(dom.custody_registry.clone());
    let block = spare_block();
    let before = total(&dom);
    transaction
        .adopt(&block.string(0..4).unwrap(), |_| Ok(()), NO_STEP)
        .unwrap();
    assert_eq!(
        total(&dom).checked_sub(before).unwrap(),
        block_usage(&block).checked_add(table_usage(&dom)).unwrap()
    );
    // A second borrower of the block, and an owner of its own block, charge no block.
    let after = total(&dom);
    for value in [block.string(4..8).unwrap(), exact_owner()] {
        transaction.adopt(&value, |_| Ok(()), NO_STEP).unwrap();
    }
    assert_eq!(total(&dom), after);
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
}

#[test]
fn the_table_is_charged_by_its_capacity_and_its_insertions_are_metered() {
    let dom = dom();
    let transaction = block_custody::BlockAdoptions::new(dom.custody_registry.clone());
    let before = total(&dom);
    let blocks: Vec<TextBlock> = (0..9).map(|_| spare_block()).collect();
    let steps = std::cell::Cell::new(0_usize);
    for (listed, block) in blocks.iter().enumerate() {
        transaction
            .adopt(
                &block.string(0..4).unwrap(),
                |_| Ok(()),
                || {
                    steps.set(steps.get() + 1);
                    Ok(())
                },
            )
            .unwrap();
        // The table reserves its room in doublings from four entries, and the ledger holds
        // every block and the whole capacity.
        let capacity = [4, 8, 16][usize::from(listed >= 4) + usize::from(listed >= 8)];
        assert_eq!(dom.custody_registry.listed_capacity(), capacity);
    }
    // Every insertion charged one step for each entry it moved, in the registry's table and in
    // the transaction's own table: in each, none moved more than the entries listed before it, so
    // twice the sum of the entries listed before is the most they can have moved.
    assert!(steps.get() <= 2 * (0..9).sum::<usize>());
    let listed: custody::Usage = blocks
        .iter()
        .try_fold(custody::Usage::ZERO, |sum, block| {
            sum.checked_add(block_usage(block))
        })
        .unwrap();
    assert_eq!(
        total(&dom).checked_sub(before).unwrap(),
        listed.checked_add(table_usage(&dom)).unwrap()
    );
}

#[test]
fn a_cancelled_insertion_lists_nothing_and_keeps_the_ledger() {
    let dom = dom();
    let transaction = block_custody::BlockAdoptions::new(dom.custody_registry.clone());
    let mut blocks: Vec<TextBlock> = (0..8).map(|_| spare_block()).collect();
    blocks.sort_by_key(|block| block.address());
    // The block of the highest address is listed first, so every other block goes before it and
    // moves it.
    let first = blocks.pop().unwrap();
    transaction
        .adopt(&first.string(0..4).unwrap(), |_| Ok(()), NO_STEP)
        .unwrap();
    let before = total(&dom);
    for block in &blocks {
        let outcome = transaction.adopt(
            &block.string(0..4).unwrap(),
            |_| Ok(()),
            || Err(crate::DomError::Cancelled),
        );
        assert_eq!(outcome, Err(crate::DomError::Cancelled));
        assert!(!dom.custody_registry.block_listed(block));
        assert_eq!(total(&dom), before);
    }
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
}

#[test]
fn a_refused_batch_leaves_no_block_charged() {
    let mut dom = dom();
    let document = document(&mut dom);
    let block = spare_block();
    let before = total(&dom);
    let mut batch = crate::DomBatch::new();
    let text = batch.create_text(document, block.string(0..4).unwrap());
    // A text node is not a parent, so the batch is refused after the text is charged.
    batch.insert_before(&text, &text, None);
    assert!(dom.apply(batch, &AtomicBool::new(false)).is_err());
    // The units are exact; the bytes of the banks a refused batch reserved are not compared.
    assert_eq!(total(&dom).units(), before.units());
    assert_eq!(dom.custody_registry.listed_blocks(), 0);
    assert!(!dom.custody_registry.block_listed(&block));
}

#[test]
fn a_block_is_released_once_no_string_holds_it() {
    let mut dom = dom();
    let document = document(&mut dom);
    let empty = total(&dom);
    let block = spare_block();
    let mut batch = crate::DomBatch::new();
    let first = batch.create_text(document, block.string(0..4).unwrap());
    let second = batch.create_text(document, block.string(4..8).unwrap());
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert!(dom.custody_registry.block_listed(&block));
    let nodes = [
        done.created(&first).unwrap().unwrap(),
        done.created(&second).unwrap().unwrap(),
    ];
    drop(done);
    drop(block);
    let mut batch = crate::DomBatch::new();
    for node in nodes {
        batch.destroy(node);
    }
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.custody_registry.listed_blocks(), 0);
    assert_eq!(total(&dom).units(), empty.units());
}

#[test]
fn a_block_whose_last_holder_dropped_between_applies_is_released_before_the_next_transaction() {
    let mut dom = dom();
    let registry = dom.custody_registry.clone();
    let document = document(&mut dom);
    let block = spare_block();
    let limit = total(&dom).units() + block.capacity();
    let mut batch = crate::DomBatch::new();
    let text = batch.create_text(document, block.string(0..4).unwrap());
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let node = done.created(&text).unwrap().unwrap();
    // A snapshot holds the text, and so the block, past the apply that destroys the node.
    let snapshot = dom.snapshot();
    drop(done);
    drop(block);
    let mut batch = crate::DomBatch::new();
    batch.destroy(node);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(registry.listed_blocks(), 1);
    // The last holder drops between applies, so no apply has swept the entry yet.
    drop(snapshot);
    assert_eq!(registry.listed_blocks(), 1);
    let cancel = AtomicBool::new(false);
    let draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    assert_eq!(registry.listed_blocks(), 0);
    // A block that fits only when the stale entry is gone is admitted.
    let fresh = spare_block();
    let outcome = draft.blocks.adopt(
        &fresh.string(0..4).unwrap(),
        |usage| {
            if usage.units() > limit {
                Err(crate::DomError::ResourceLimit {
                    resource: crate::DomResource::SourceRetainedUnits,
                    limit,
                })
            } else {
                Ok(())
            }
        },
        NO_STEP,
    );
    assert!(outcome.is_ok(), "{outcome:?}");
}

#[test]
fn a_freed_block_is_not_mistaken_for_a_block_made_after_it() {
    let dom = dom();
    let registry = dom.custody_registry.clone();
    let transaction = block_custody::BlockAdoptions::new(registry.clone());
    let freed = spare_block();
    let freed_address = freed.address();
    transaction
        .adopt(&freed.string(0..4).unwrap(), |_| Ok(()), NO_STEP)
        .unwrap();
    transaction.publish();
    let baseline = total(&dom)
        .checked_sub(block_usage(&freed))
        .unwrap()
        .checked_sub(table_usage(&dom))
        .unwrap();
    drop(freed);
    // The entry is not swept yet, and holds the address of the freed block against every block
    // made now: each of these is charged on its own, and none is taken for the freed block.
    let mut later = Vec::new();
    for _ in 0..64 {
        let block = spare_block();
        assert_ne!(block.address(), freed_address);
        assert!(!registry.block_listed(&block));
        transaction
            .adopt(&block.string(0..4).unwrap(), |_| Ok(()), NO_STEP)
            .unwrap();
        later.push(block);
    }
    assert_eq!(registry.listed_blocks(), 65);
    transaction.publish();
    registry.sweep_blocks();
    // Only the freed block's charge is released.
    assert_eq!(registry.listed_blocks(), 64);
    let blocks: custody::Usage = later
        .iter()
        .try_fold(custody::Usage::ZERO, |sum, block| {
            sum.checked_add(block_usage(block))
        })
        .unwrap();
    assert_eq!(
        total(&dom).checked_sub(baseline).unwrap(),
        blocks.checked_add(table_usage(&dom)).unwrap()
    );
}

/// The payload a formation holds after it shares `value`, and the number of blocks listed, by the
/// listing way and by the way that lists nothing.
fn shared_through_formation(value: &DocumentString, listed: bool) -> (custody::Usage, usize) {
    let dom = dom();
    let cancel = AtomicBool::new(false);
    let batch = crate::DomBatch::new();
    let limits = crate::DomSourceLimits::new(16_000_000, 1_000_000, 1_000_000, 32, 4096, 4096, 1);
    let (draft, _) =
        Draft::new_source(&dom, batch.owner.clone(), &batch.commands, limits, &cancel).unwrap();
    let formation = source::formation::Formation::new(&draft, &cancel)
        .unwrap()
        .unwrap();
    let shared = if listed {
        source::formation::share_listed(value, Some(&formation), &cancel)
    } else {
        source::formation::share(value, Some(&formation), &cancel)
    }
    .unwrap();
    assert!(shared.shares_storage_with(value));
    let payload = formation.owned_payload();
    (
        custody::Usage::new(payload.bytes, payload.units),
        dom.custody_registry.listed_blocks(),
    )
}

#[test]
fn a_formation_that_lists_prices_a_borrower_by_length_and_one_that_does_not_by_capacity() {
    let block = spare_block();
    let borrower = block.string(0..4).unwrap();
    let (listed, blocks) = shared_through_formation(&borrower, true);
    assert_eq!((listed.units(), blocks), (4, 1));
    let (unlisted, blocks) = shared_through_formation(&borrower, false);
    assert_eq!((unlisted.units(), blocks), (block.capacity(), 0));
    // An owner is priced by its capacity either way, and lists nothing.
    let owner = exact_owner();
    assert_eq!(
        shared_through_formation(&owner, true),
        (unlisted_exact(), 0)
    );
    assert_eq!(
        shared_through_formation(&owner, false),
        (unlisted_exact(), 0)
    );
}

fn unlisted_exact() -> custody::Usage {
    custody::Usage::new(size_of::<Vec<u16>>() + 4 * size_of::<u16>(), 4)
}

#[test]
fn a_comment_node_lists_the_block_its_string_borrows_and_is_charged_by_length() {
    let build = |value: DocumentString| {
        let mut dom = dom();
        let document = document(&mut dom);
        let mut batch = crate::DomBatch::new();
        batch.create_comment(document, value);
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        (total(&dom), dom.custody_registry.listed_blocks())
    };
    let block = spare_block();
    let ((shared, listed), (owned, none)) =
        (build(block.string(0..4).unwrap()), build(exact_owner()));
    assert_eq!((listed, none), (1, 0));
    assert_eq!(shared.units(), owned.units() + block.capacity());
}

#[test]
fn a_transaction_relies_only_on_a_listing_that_is_published_or_its_own() {
    let dom = dom();
    let registry = dom.custody_registry.clone();
    let block = spare_block();
    let borrower = block.string(0..4).unwrap();
    let before = total(&dom);
    // Two transactions are alive at once: the first lists the block and has not published, and
    // the second needs it too.
    let first = block_custody::BlockAdoptions::new(registry.clone());
    let second = block_custody::BlockAdoptions::new(registry.clone());
    first.adopt(&borrower, |_| Ok(()), NO_STEP).unwrap();
    let charged = total(&dom);
    second.adopt(&borrower, |_| Ok(()), NO_STEP).unwrap();
    // It claims the listing, and does not charge the block again.
    assert_eq!(total(&dom), charged);
    assert_eq!(registry.listed_blocks(), 1);
    // The first gives up without publishing: the listing stays, because the second relies on it.
    drop(first);
    assert!(registry.block_listed(&block));
    assert_eq!(total(&dom), charged);
    // The second publishes, and the listing is published for every later transaction.
    second.publish();
    drop(second);
    assert!(registry.block_listed(&block));
    let third = block_custody::BlockAdoptions::new(registry.clone());
    third.adopt(&borrower, |_| Ok(()), NO_STEP).unwrap();
    drop(third);
    assert!(registry.block_listed(&block));
    // No transaction relies on it once the last string is gone, and the sweep releases it.
    drop(borrower);
    drop(block);
    registry.sweep_blocks();
    assert_eq!(registry.listed_blocks(), 0);
    assert_eq!(total(&dom).units(), before.units());
}

#[test]
fn a_listing_every_transaction_gave_up_is_unlisted() {
    let dom = dom();
    let registry = dom.custody_registry.clone();
    let block = spare_block();
    let borrower = block.string(0..4).unwrap();
    let first = block_custody::BlockAdoptions::new(registry.clone());
    let second = block_custody::BlockAdoptions::new(registry.clone());
    first.adopt(&borrower, |_| Ok(()), NO_STEP).unwrap();
    second.adopt(&borrower, |_| Ok(()), NO_STEP).unwrap();
    drop(first);
    assert!(registry.block_listed(&block));
    drop(second);
    assert!(!registry.block_listed(&block));
}

#[test]
fn a_publication_that_does_not_pass_through_apply_sweeps_dead_blocks() {
    let mut dom = dom();
    let document = document(&mut dom);
    let block = spare_block();
    let mut batch = crate::DomBatch::new();
    let text = batch.create_text(document, block.string(0..4).unwrap());
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let node = done.created(&text).unwrap().unwrap();
    drop(done);
    drop(block);
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
    let cancel = AtomicBool::new(false);
    let mut batch = crate::DomBatch::new();
    batch.destroy(node);
    // The route of the details toggle and the select and fullscreen steps: prepare, then publish.
    let prepared = dom.prepare(batch, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    assert_eq!(dom.custody_registry.listed_blocks(), 0);
}

#[test]
fn the_registry_table_entry_holds_an_address_a_weak_and_the_entry_record() {
    // Address 8, weak reference 8, and the record: a debit of 24 (a ledger handle and a usage),
    // a claim count of 8 and a published flag padded to 8, which is 40 for the record.
    assert_eq!(bank::CustodyRegistry::TABLE_ENTRY_BYTES, 8 + 8 + 24 + 8 + 8);
    eprintln!(
        "TABLE_ENTRY_BYTES = {}",
        bank::CustodyRegistry::TABLE_ENTRY_BYTES
    );
}

/// The shifts a table of two sorted address lists charges while `blocks` are listed in order: each
/// insertion moves the entries of a higher address, in the registry's table and in the
/// transaction's own, which hold the same addresses.
fn expected_steps(blocks: &[TextBlock]) -> usize {
    let mut listed: Vec<usize> = Vec::new();
    let mut steps = 0;
    for block in blocks {
        let address = block.address();
        let shifted = listed.iter().filter(|held| **held > address).count();
        steps += 2 * shifted;
        listed.push(address);
    }
    steps
}

#[test]
fn the_work_of_adopting_n_distinct_blocks_is_metered_and_grows_as_the_shifts_do() {
    let mut measured = Vec::new();
    for n in [1000_usize, 2000, 4000, 8000] {
        let dom = dom();
        let transaction = block_custody::BlockAdoptions::new(dom.custody_registry.clone());
        // Listed from the highest address to the lowest, so that every insertion moves every entry
        // listed before it: the worst case, which does not depend on the allocator.
        let mut blocks: Vec<TextBlock> = (0..n).map(|_| spare_block()).collect();
        blocks.sort_by_key(|block| std::cmp::Reverse(block.address()));
        let steps = std::cell::Cell::new(0_usize);
        for block in &blocks {
            transaction
                .adopt(
                    &block.string(0..4).unwrap(),
                    |_| Ok(()),
                    || {
                        steps.set(steps.get() + 1);
                        Ok(())
                    },
                )
                .unwrap();
        }
        // Every entry an insertion moves, in both tables, is one step before the move; the search
        // for a listed block is a binary search of n log n comparisons that moves nothing.
        assert_eq!(steps.get(), expected_steps(&blocks), "n = {n}");
        // In this order the shifts of the two tables are n * (n - 1) in all.
        assert_eq!(steps.get(), n * (n - 1), "n = {n}");
        measured.push(steps.get());
    }
    // The shifts are quadratic in n, so the steps grow by 4 for each doubling, and by at most 4.4.
    for pair in measured.windows(2) {
        assert!(
            pair[1] as f64 <= 4.4 * pair[0] as f64,
            "{} steps then {}",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn a_parked_installation_draft_and_another_batch_charge_a_shared_block_once() {
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let document = document(&mut dom);
    let mut batch = crate::DomBatch::new();
    let parent = batch.create_element_ns(document, surgeist_html::values::Namespace::html(), "div");
    let done = dom.apply(batch, &cancel).unwrap();
    let parent = done.created(&parent).unwrap().unwrap();
    drop(done);
    // The fragment copies its text into the one block of the fragment, which the installation
    // lists and another batch borrows.
    let mut fragment = surgeist_html::construction::HtmlFragment::builder(
        surgeist_html::construction::HtmlConstructionLimits::new(16, 16, 16, 4096, 16),
    )
    .unwrap();
    fragment
        .append_text(&fragment.root(), "a".repeat(600))
        .unwrap();
    let source = fragment.finish().unwrap();
    let surgeist_html::construction::HtmlNodeKind::Text(text) = source
        .node(&source.root().children().next().unwrap())
        .unwrap()
        .kind()
    else {
        panic!("the fragment holds one text")
    };
    let block = text.block().clone();
    assert!(block.capacity() >= 600);
    let stamp = dom.snapshot().document(document).unwrap().stamp();
    // The installation stages a draft that lists the block and does not publish yet.
    let staged = dom
        .stage_native_html_installation(
            stamp,
            parent,
            None,
            &source,
            installation::Budgets {
                native: crate::DomNativeHtmlInstallLimits::new(16, 1_000_000, 65536),
                source: None,
            },
            &cancel,
        )
        .map_err(|_| ())
        .unwrap();
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
    let listed = total(&dom);
    // Another batch borrows the same block and publishes while the staged draft is parked.
    let mut batch = crate::DomBatch::new();
    batch.create_text(document, block.string(4..8).unwrap());
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
    // The block is not charged a second time: only the node and its own length were added.
    assert!(total(&dom).units() < listed.units() + block.capacity());
    // The parked draft is dropped unpublished, and the block stays charged for the published node.
    drop(staged);
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
    assert!(dom.custody_registry.block_listed(&block));
    assert!(total(&dom).units() >= block.capacity());
}

/// The custody total of a store whose document holds one element with an attribute for each of
/// `values`, and the number of blocks it charges.
fn total_with_attributes(values: &[DocumentString]) -> (custody::Usage, usize) {
    let mut dom = dom();
    let document = document(&mut dom);
    let mut batch = crate::DomBatch::new();
    let element = batch.create_element(document, "div");
    for (index, value) in values.iter().enumerate() {
        batch.set_attribute(&element, format!("data-{index}"), value.clone());
    }
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (total(&dom), dom.custody_registry.listed_blocks())
}

#[test]
fn attributes_that_borrow_one_block_are_charged_by_length_and_the_block_once() {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let borrowed = [
        block.string(0..4).unwrap(),
        block.string(4..8).unwrap(),
        block.string(2..6).unwrap(),
    ];
    let owners = [exact_owner(), exact_owner(), exact_owner()];
    let ((shared, listed), (owned, none)) = (
        total_with_attributes(&borrowed),
        total_with_attributes(&owners),
    );
    // The block adds its capacity in units once, whatever the number of attributes.
    assert_eq!(shared.units(), owned.units() + block.capacity());
    assert_eq!((listed, none), (1, 0));
    // A fourth attribute of the block adds its own length and no block.
    let four = [
        borrowed[0].clone(),
        borrowed[1].clone(),
        borrowed[2].clone(),
        block.string(1..5).unwrap(),
    ];
    let more = [
        owners[0].clone(),
        owners[1].clone(),
        owners[2].clone(),
        exact_owner(),
    ];
    let ((shared, listed), (owned, _)) =
        (total_with_attributes(&four), total_with_attributes(&more));
    assert_eq!(shared.units(), owned.units() + block.capacity());
    assert_eq!(listed, 1);
}

#[test]
fn a_block_is_released_once_no_attribute_holds_it() {
    let mut dom = dom();
    let document = document(&mut dom);
    let block = spare_block();
    let mut batch = crate::DomBatch::new();
    let element = batch.create_element(document, "div");
    batch.set_attribute(&element, "data-a", block.string(0..4).unwrap());
    batch.set_attribute(&element, "data-b", block.string(4..8).unwrap());
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let node = done.created(&element).unwrap().unwrap();
    assert!(dom.custody_registry.block_listed(&block));
    drop(done);
    drop(block);
    // Removing one attribute leaves the block with the other.
    let mut batch = crate::DomBatch::new();
    batch.remove_attribute(node, "data-a");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
    let mut batch = crate::DomBatch::new();
    batch.remove_attribute(node, "data-b");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.custody_registry.listed_blocks(), 0);
}

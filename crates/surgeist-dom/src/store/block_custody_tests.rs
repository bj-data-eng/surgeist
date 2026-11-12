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
    let [first, second] = ["data-a", "data-b"].map(|name| {
        dom.snapshot()
            .node(node)
            .unwrap()
            .get_attribute_node(&name.into())
            .unwrap()
            .id()
    });
    drop(done);
    drop(block);
    // A detached Attr still owns its value.
    let mut batch = crate::DomBatch::new();
    batch.remove_attribute(node, "data-a");
    batch.remove_attribute(node, "data-b");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
    for (attr, blocks) in [(first, 1), (second, 0)] {
        let mut batch = crate::DomBatch::new();
        batch.destroy(attr.node());
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        assert_eq!(dom.custody_registry.listed_blocks(), blocks);
    }
}

/// A store whose element and group limits admit the families the attribute tests build.
fn element_dom() -> Dom {
    Dom::new(
        DomLimits::new(
            2048,
            1_000_000,
            4096,
            128,
            128,
            32,
            32_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_details_toggle_intentions(64)
        .unwrap()
        .with_input_processing(1_000_000, 1_000_000)
        .unwrap(),
    )
    .unwrap()
}

/// The custody total of a store whose document holds one HTML `tag` element with the given
/// attributes, and the number of blocks it charges.
fn total_with_element(
    tag: &str,
    attributes: &[(&str, &DocumentString)],
) -> (custody::Usage, usize) {
    let mut dom = element_dom();
    let document = document(&mut dom);
    let mut batch = crate::DomBatch::new();
    let element = batch.create_element_ns(document, surgeist_html::values::Namespace::html(), tag);
    for (name, value) in attributes {
        batch.set_attribute(&element, *name, (*value).clone());
    }
    batch.insert_before(&element, document, None);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (total(&dom), dom.custody_registry.listed_blocks())
}

/// A borrower of a block with spare capacity and an exact owner of the same four units, which
/// the elements of the tests below retain through the attribute `name`.
fn assert_charged_by_length_and_the_block_once(
    tag: &str,
    name: &str,
    others: &[(&str, &DocumentString)],
) {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let borrowed = block.string(0..4).unwrap();
    let owner = exact_owner();
    let with = |value: &DocumentString| {
        let mut attributes = others.to_vec();
        attributes.push((name, value));
        total_with_element(tag, &attributes)
    };
    let (shared, listed) = with(&borrowed);
    let (owned, none) = with(&owner);
    // The element retains the value again beyond the attribute, and every retention of a
    // borrower is its length: the block adds its capacity once, whatever the retentions.
    assert_eq!(shared.units(), owned.units() + block.capacity());
    assert_eq!((listed, none), (1, 0));
}

#[test]
fn a_nonce_that_borrows_a_block_is_charged_by_length_and_the_block_once() {
    assert_charged_by_length_and_the_block_once("script", "nonce", &[]);
}

#[test]
fn the_name_of_an_input_that_borrows_a_block_is_charged_by_length_and_the_block_once() {
    assert_charged_by_length_and_the_block_once("input", "name", &[]);
}

#[test]
fn the_name_of_a_details_that_borrows_a_block_is_charged_by_length_and_the_block_once() {
    assert_charged_by_length_and_the_block_once("details", "name", &[]);
}

#[test]
fn the_observed_source_of_an_image_input_that_borrows_a_block_is_charged_by_length_and_the_block_once()
 {
    assert_charged_by_length_and_the_block_once(
        "input",
        "src",
        &[("type", &DocumentString::from("image"))],
    );
}

#[test]
fn a_focus_report_of_an_attribute_that_borrows_a_block_is_charged_by_length() {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let mut dom = element_dom();
    let document = document(&mut dom);
    let mut batch = crate::DomBatch::new();
    let element =
        batch.create_element_ns(document, surgeist_html::values::Namespace::html(), "dialog");
    batch.set_attribute(&element, "inert", block.string(0..4).unwrap());
    batch.insert_before(&element, document, None);
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let node = done.created(&element).unwrap().unwrap();
    // The source operation retains the units the store holds and at most 100 more: the report of
    // the attribute fits when it is priced by its four units, and cannot when it is priced by the
    // capacity of its block.
    let cancel = AtomicBool::new(false);
    let units = total(&dom).units() + 100;
    let limits = crate::DomSourceLimits::new(16_000_000, units, 1_000_000, 32, 4096, 4096, 1);
    let batch = crate::DomBatch::new();
    let (draft, _) =
        Draft::new_source(&dom, batch.owner.clone(), &batch.commands, limits, &cancel).unwrap();
    let key = draft.resolve(node.into()).unwrap();
    assert_eq!(
        draft.focus_boolean(
            key,
            surgeist_html::attributes::HtmlBooleanAttribute::Inert,
            surgeist_html::vocabulary::KnownHtmlAttribute::Inert,
            &cancel,
        ),
        Ok(true)
    );
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
}

/// Builds a store with one HTML `tag` element whose `name` attribute is `value`, and gives back
/// the store and the element.
fn store_with_attribute(tag: &str, name: &str, value: DocumentString) -> (Dom, crate::NodeId) {
    let mut dom = element_dom();
    let document = document(&mut dom);
    let mut batch = crate::DomBatch::new();
    let element = batch.create_element_ns(document, surgeist_html::values::Namespace::html(), tag);
    batch.set_attribute(&element, name, value);
    batch.insert_before(&element, document, None);
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let node = done.created(&element).unwrap().unwrap();
    (dom, node)
}

#[test]
fn destroying_the_detached_attr_of_each_retention_releases_the_block() {
    for (tag, name) in [
        ("script", "nonce"),
        ("input", "name"),
        ("input", "src"),
        ("details", "name"),
    ] {
        let block = spare_block();
        let weak = block.downgrade();
        let (mut dom, node) = store_with_attribute(tag, name, block.string(0..4).unwrap());
        let attr = dom
            .snapshot()
            .node(node)
            .unwrap()
            .get_attribute_node(&name.into())
            .unwrap()
            .id();
        assert!(dom.custody_registry.block_listed(&block), "{tag} {name}");
        drop(block);
        let mut batch = crate::DomBatch::new();
        batch.remove_attribute(node, name);
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        assert!(weak.holders() > 0, "{tag} {name}");
        let mut batch = crate::DomBatch::new();
        batch.destroy(attr.node());
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        // The attribute, the nonce slot, the control facts and the input value no longer hold
        // the block, so the next sweep gives its charge back.
        assert_eq!(weak.holders(), 0, "{tag} {name}");
        assert_eq!(dom.custody_registry.listed_blocks(), 0, "{tag} {name}");
    }
}

#[test]
fn replacing_an_attribute_value_releases_the_block_of_the_old_value() {
    let first = spare_block();
    let second = spare_block();
    let (first_weak, second_weak) = (first.downgrade(), second.downgrade());
    let (mut dom, node) = store_with_attribute("div", "data-a", first.string(0..4).unwrap());
    drop(first);
    let mut batch = crate::DomBatch::new();
    batch.set_attribute(node, "data-a", second.string(4..8).unwrap());
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(first_weak.holders(), 0);
    assert_eq!(second_weak.holders(), 2);
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
    drop(second);
    assert_eq!(second_weak.holders(), 1);
}

#[test]
fn replacing_an_attribute_value_by_an_equal_one_keeps_the_published_value_and_its_block() {
    // The edited element compares equal to the published one, so the published element and its
    // value stay; the block of the equal replacement is not held.
    let first = spare_block();
    let second = spare_block();
    let (first_weak, second_weak) = (first.downgrade(), second.downgrade());
    let (mut dom, node) = store_with_attribute("div", "data-a", first.string(0..4).unwrap());
    drop(first);
    let mut batch = crate::DomBatch::new();
    batch.set_attribute(node, "data-a", second.string(0..4).unwrap());
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(first_weak.holders(), 1);
    assert_eq!(second_weak.holders(), 1);
    drop(second);
    assert_eq!(second_weak.holders(), 0);
    // The next transaction sweeps the entry of the block nothing holds.
    dom.apply(crate::DomBatch::new(), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(dom.custody_registry.listed_blocks(), 1);
}

/// Removes the `name` attribute, whose value is `value`, from an HTML `tag` element in a source
/// operation that retains at most `margin` units beyond the store, and gives whether the
/// operation completed.
fn removal_completes_within(tag: &str, name: &str, value: &DocumentString, margin: usize) -> bool {
    let (mut dom, node) = store_with_attribute(tag, name, value.clone());
    let cancel = AtomicBool::new(false);
    let mut batch = crate::DomBatch::new();
    batch.remove_attribute(node, name);
    let units = total(&dom).units() + margin;
    let limits = crate::DomSourceLimits::new(16_000_000, units, 1_000_000, 32, 4096, 4096, 1);
    match dom
        .begin_source_commands(batch, limits, &cancel)
        .and_then(|mut lease| lease.advance(&cancel))
    {
        Ok(crate::DomSourceProgress::Complete(_)) => true,
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedUnits,
            ..
        }) => false,
        other => panic!("unexpected end of the source operation: {other:?}"),
    }
}

/// The least margin of units beyond the store in which the removal completes.
fn least_margin(tag: &str, name: &str, value: &DocumentString) -> usize {
    (0..2048)
        .find(|margin| removal_completes_within(tag, name, value, *margin))
        .expect("the removal completes within the limit of the scan")
}

/// The removal of an attribute that borrows a block needs the margin of the removal of an
/// attribute that is an exact owner of the same text, which is below the capacity of the block.
fn assert_removal_is_charged_by_length(tag: &str, name: &str) {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let owner = least_margin(tag, name, &exact_owner());
    let borrower = least_margin(tag, name, &block.string(0..4).unwrap());
    assert!(owner < block.capacity());
    assert_eq!(borrower, owner);
}

#[test]
fn the_transient_loading_report_of_an_image_that_borrows_a_block_is_charged_by_length() {
    assert_removal_is_charged_by_length("img", "loading");
}

#[test]
fn the_loading_report_of_an_iframe_that_borrows_a_block_is_charged_by_length() {
    assert_removal_is_charged_by_length("iframe", "loading");
}

/// A store whose shadow trees and elements admit the named slots the tests below build.
fn slot_dom() -> Dom {
    Dom::new(
        DomLimits::new(
            4096,
            1_000_000,
            16_384,
            64,
            1,
            0,
            0,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

/// The custody total of a store whose host has an open shadow root with named slot assignment
/// holding one `slot` element named `slot_name` and one light child slotted by `child_name`, and
/// the number of blocks it charges.
fn total_with_named_slots(
    slot_name: Option<&DocumentString>,
    child_name: Option<&DocumentString>,
) -> (custody::Usage, usize) {
    use surgeist_html::construction::{
        HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    };
    let mut dom = slot_dom();
    let document = document(&mut dom);
    let html = surgeist_html::values::Namespace::html();
    let mut batch = crate::DomBatch::new();
    let host = batch.create_element_ns(document, html.clone(), "div");
    batch.insert_before(&host, document, None);
    let shadow = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Named,
            false,
            true,
            false,
        ),
    );
    let slot = batch.create_element_ns(document, html.clone(), "slot");
    if let Some(name) = slot_name {
        batch.set_attribute(&slot, "name", name.clone());
    }
    batch.insert_before(&slot, &shadow, None);
    let child = batch.create_element_ns(document, html, "span");
    if let Some(name) = child_name {
        batch.set_attribute(&child, "slot", name.clone());
    }
    batch.insert_before(&child, &host, None);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (total(&dom), dom.custody_registry.listed_blocks())
}

#[test]
fn the_name_of_a_slot_that_borrows_a_block_is_charged_by_length_and_the_block_once() {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let (shared, listed) = total_with_named_slots(Some(&block.string(0..4).unwrap()), None);
    let (owned, none) = total_with_named_slots(Some(&exact_owner()), None);
    assert_eq!(shared.units(), owned.units() + block.capacity());
    assert_eq!((listed, none), (1, 0));
}

#[test]
fn the_slot_name_of_a_slottable_that_borrows_a_block_is_charged_by_length_and_the_block_once() {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let (shared, listed) = total_with_named_slots(None, Some(&block.string(0..4).unwrap()));
    let (owned, none) = total_with_named_slots(None, Some(&exact_owner()));
    assert_eq!(shared.units(), owned.units() + block.capacity());
    assert_eq!((listed, none), (1, 0));
}

/// The custody total of a store whose element holds a saved `is` value of `value`, which a second
/// transaction keeps while it binds the element to a registry, and the number of blocks charged.
fn total_after_binding_a_registry(value: &DocumentString) -> (custody::Usage, usize) {
    total_after_binding_a_registry_without(value, None)
}

/// [`total_after_binding_a_registry`], with the listing of `unlist` dropped between the two
/// transactions, so that the binding is the only path that lists the block of the saved `is`.
fn total_after_binding_a_registry_without(
    value: &DocumentString,
    unlist: Option<&TextBlock>,
) -> (custody::Usage, usize) {
    let cancel = AtomicBool::new(false);
    let mut dom = element_dom();
    let document = document(&mut dom);
    let mut batch = crate::DomBatch::new();
    let element =
        batch.create_element_ns(document, surgeist_html::values::Namespace::html(), "div");
    batch.insert_before(&element, document, None);
    let done = dom.apply(batch, &cancel).unwrap();
    let node = done.created(&element).unwrap().unwrap();
    // The first transaction forms the saved `is` of the element, as element creation does.
    let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    let key = draft.resolve(node.into()).unwrap();
    let registry = draft.custody_registry().clone();
    let mut admit = |usage| draft.admit_custody_usage(usage, &cancel);
    let mut formation =
        crate::element::ElementFormation::new(&registry, &draft.blocks, None, &cancel, &mut admit);
    let custom = formation
        .custom(
            crate::element::CustomAssociation {
                registry: crate::DomRegistryBinding::Null,
                definition: crate::DomElementDefinition::Null,
                saved_is: Some(value.clone()),
            },
            source::storage::Payload::default(),
        )
        .unwrap();
    let mut saved = draft
        .node(key)
        .unwrap()
        .element()
        .unwrap()
        .language()
        .clone();
    saved.set_custom_payload(Some(custom));
    let formed = formation.finish(saved);
    draft
        .replace_element_formed(key, formed, None, &cancel)
        .unwrap();
    let prepared = draft.finish(&dom, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    if let Some(block) = unlist {
        dom.custody_registry.unlist_block(block);
        assert_eq!(dom.custody_registry.listed_blocks(), 0);
    }
    // The second transaction binds the element to a registry and re-forms the association.
    let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    let key = draft.resolve(node.into()).unwrap();
    let binding = crate::DomRegistryBinding::Registry(crate::registry::DomRegistryId {
        owner: dom.owner,
        slot: 0,
        generation: 0,
    });
    draft
        .establish_element_registry(key, binding, &cancel)
        .unwrap();
    let prepared = draft.finish(&dom, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    (total(&dom), dom.custody_registry.listed_blocks())
}

#[test]
fn the_saved_is_that_borrows_a_block_is_charged_by_length_when_a_registry_binds_the_element() {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let (shared, listed) = total_after_binding_a_registry(&block.string(0..4).unwrap());
    let (owned, none) = total_after_binding_a_registry(&exact_owner());
    assert_eq!(shared.units(), owned.units() + block.capacity());
    assert_eq!((listed, none), (1, 0));
}

/// Copies `value` into an exact owner under a formation of an operation that retains at most
/// `units` units and `bytes` bytes beyond the store, and gives the copy, the custody total before
/// and after, and the number of blocks listed.
fn exact_owner_within(
    value: &DocumentString,
    margin: (usize, usize),
) -> (
    Result<DocumentString, DomError>,
    custody::Usage,
    custody::Usage,
    usize,
) {
    let dom = dom();
    let cancel = AtomicBool::new(false);
    let batch = crate::DomBatch::new();
    let roomy = crate::DomSourceLimits::new(16_000_000, 1_000_000, 1_000_000, 32, 4096, 4096, 1);
    let (mut draft, _) =
        Draft::new_source(&dom, batch.owner.clone(), &batch.commands, roomy, &cancel).unwrap();
    // The operation admits against the custody it controls, the draft's own included, and the
    // bytes and units outside the store, so the margins count from there.
    let held = (
        draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes,
        draft.source_storage_units(&cancel).unwrap() + draft.source_other_units,
    );
    draft.source_limits = Some(crate::DomSourceLimits::new(
        held.0 + margin.1,
        held.1 + margin.0,
        1_000_000,
        32,
        4096,
        4096,
        1,
    ));
    let formation = source::formation::Formation::new(&draft, &cancel)
        .unwrap()
        .unwrap();
    let before = total(&dom);
    let copy = source::formation::exact_owner(value, Some(&formation), &cancel);
    (
        copy,
        before,
        total(&dom),
        dom.custody_registry.listed_blocks(),
    )
}

#[test]
fn a_borrower_is_copied_into_an_exact_owner_whose_charge_ends_with_the_copy() {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let borrowed = block.string(2..6).unwrap();
    let (copy, before, after, listed) = exact_owner_within(&borrowed, (64, 1_000_000));
    let copy = copy.unwrap();
    assert_eq!(copy, borrowed);
    assert!(copy.borrowed_block().is_none());
    assert!(!copy.shares_storage_with(&borrowed));
    // The owner holds its four units and nothing of the block, and the copy leaves the ledger as
    // it was: the holder of the owner prices it.
    assert_eq!(copy.retained_units(), 4);
    assert_eq!(after, before);
    assert_eq!(listed, 0);
    // An owner is shared and charged nothing.
    let owner = exact_owner();
    let (shared, before, after, _) = exact_owner_within(&owner, (0, 1_000_000));
    assert!(shared.unwrap().shares_storage_with(&owner));
    assert_eq!(after, before);
}

#[test]
fn the_copy_of_a_borrower_is_charged_its_header_length_and_spare_before_and_after_it_allocates() {
    let block = spare_block();
    let borrowed = block.string(2..6).unwrap();
    let (copy, ..) = exact_owner_within(&borrowed, (1_000, 1_000_000));
    let held = copy.unwrap().retained_units();
    // The units are the length and the spare room the allocator left; the bytes are the header and
    // two bytes for each of those units.
    let units = borrowed.len() + (held - borrowed.len());
    let bytes = size_of::<Vec<u16>>() + units * size_of::<u16>();
    for (margin, accepted) in [
        ((units - 1, 1_000_000), false),
        ((units, 1_000_000), true),
        ((1_000, bytes - 1), false),
        ((1_000, bytes), true),
    ] {
        let (outcome, before, after, _) = exact_owner_within(&borrowed, margin);
        assert_eq!(outcome.is_ok(), accepted, "{margin:?}: {outcome:?}");
        if !accepted {
            let resource = if margin.0 < 1_000 {
                DomResource::SourceRetainedUnits
            } else {
                DomResource::SourceRetainedBytes
            };
            assert!(
                matches!(&outcome, Err(DomError::ResourceLimit { resource: r, .. }) if *r == resource),
                "{margin:?}: {outcome:?}"
            );
        }
        // A refusal and an accepted copy alike leave the ledger as it was.
        assert_eq!(after, before);
    }
}

/// The slot names that a store holds for a slot named by a range of `slot_block` and a light child
/// slotted by a range of `child_block`: the nodes the second transaction of a test moves.
struct NamedSlots {
    dom: Dom,
    shadow: crate::NodeId,
    slot: crate::NodeId,
    host: crate::NodeId,
    child: crate::NodeId,
}

fn named_slots(slot_block: &TextBlock, child_block: &TextBlock) -> NamedSlots {
    use surgeist_html::construction::{
        HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    };
    let mut dom = slot_dom();
    let document = document(&mut dom);
    let html = surgeist_html::values::Namespace::html();
    let mut batch = crate::DomBatch::new();
    let host = batch.create_element_ns(document, html.clone(), "div");
    batch.insert_before(&host, document, None);
    let shadow = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Named,
            false,
            true,
            false,
        ),
    );
    let slot = batch.create_element_ns(document, html.clone(), "slot");
    batch.set_attribute(&slot, "name", slot_block.string(0..4).unwrap());
    batch.insert_before(&slot, &shadow, None);
    let child = batch.create_element_ns(document, html, "span");
    batch.set_attribute(&child, "slot", child_block.string(0..4).unwrap());
    batch.insert_before(&child, &host, None);
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    NamedSlots {
        shadow: done.attached_shadow_root(&shadow).unwrap().unwrap(),
        slot: done.created(&slot).unwrap().unwrap(),
        host: done.created(&host).unwrap().unwrap(),
        child: done.created(&child).unwrap().unwrap(),
        dom,
    }
}

#[test]
fn a_cached_slot_fact_that_borrows_a_block_lists_it_when_the_nodes_are_moved() {
    let (slot_block, child_block) = (spare_block(), spare_block());
    let mut store = named_slots(&slot_block, &child_block);
    let registry = store.dom.custody_registry.clone();
    let charged = total(&store.dom);
    assert_eq!(registry.listed_blocks(), 2);
    // No path has listed the blocks when the second transaction moves the slot and the child,
    // which keeps their cached facts: the facts list the blocks the names borrow.
    registry.unlist_block(&slot_block);
    registry.unlist_block(&child_block);
    assert_eq!(registry.listed_blocks(), 0);
    let mut batch = crate::DomBatch::new();
    batch.insert_before(store.slot, store.shadow, None);
    batch.insert_before(store.child, store.host, None);
    store.dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert!(registry.block_listed(&slot_block));
    assert!(registry.block_listed(&child_block));
    assert_eq!(registry.listed_blocks(), 2);
    // Each block is charged once again, and the store holds what it held before.
    assert_eq!(total(&store.dom).units(), charged.units());
}

#[test]
fn a_registry_binding_lists_the_block_of_the_saved_is_it_reforms() {
    let block = spare_block();
    let saved = block.string(0..4).unwrap();
    let (listed, count) = total_after_binding_a_registry_without(&saved, Some(&block));
    let (kept, kept_count) = total_after_binding_a_registry(&saved);
    // The block is charged once again, as when the element formed the saved `is`.
    assert_eq!((count, kept_count), (1, 1));
    assert_eq!(listed.units(), kept.units());
}

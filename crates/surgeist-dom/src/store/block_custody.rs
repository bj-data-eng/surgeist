//! Custody of the text blocks that borrowed strings index.
//!
//! A string that is a range of a block accounts for its own length where it is retained through a
//! path that lists its block, and the block is charged once for the store, by one debit listed in
//! the custody registry beside the block's address. The registry charges its table of blocks by
//! the table's capacity, in one debit that grows with it. It keeps each block weakly, so no other
//! block can take the address of a listed block, and drops the debit of a block no string holds
//! any more when it sweeps blocks, at the start of every transaction and at the end of every
//! publication. A path that does not list the block charges its strings by the capacity of their
//! blocks, as for any other string.
//!
//! A transaction relies only on a listing that it made, or that it claimed, or that a published
//! transaction made. A listing made by a transaction that has not published yet stays while any
//! transaction claims it, and is dropped with the last claim unless one of them publishes.

use std::sync::{Arc, Mutex, PoisonError};

use surgeist_html::values::{DocumentString, TextBlock, TextBlockTable};

use super::{DomError, bank::CustodyRegistry, custody::Usage};

/// The blocks one transaction listed or claimed in the custody registry, which it gives up on drop
/// unless it published them first.
///
/// The transaction's own record is a table keyed by block address, with one strong handle for each
/// block it relies on. It is bounded by the entries the registry's table already charges by
/// capacity, finds a block by search, and meters the entries an insertion moves like the registry's
/// table does. What is charged for the transaction is its `Arc` target, which is this structure,
/// beside the `Arc` handle the draft's own header covers; the table's entries are the handles
/// counted above.
pub(crate) struct BlockAdoptions {
    registry: CustodyRegistry,
    listed: Mutex<TextBlockTable<TextBlock>>,
}

impl BlockAdoptions {
    /// The bytes of the transaction's own allocation, which the draft charges with its header.
    pub(crate) const BYTES: usize = size_of::<Self>();

    pub(crate) fn new(registry: CustodyRegistry) -> Arc<Self> {
        Arc::new(Self {
            registry,
            listed: Mutex::new(TextBlockTable::new()),
        })
    }

    /// Relies on a debit that charges the block `value` borrows: one that is published, one this
    /// transaction made or claimed, or a new one. `admit` checks the prospective total as for any
    /// retained custody, and `step` meters the work of the insertions: one step for each entry an
    /// insertion moves in the registry's table and in the transaction's own. A refusal lists and
    /// claims nothing.
    pub(crate) fn adopt(
        &self,
        value: &DocumentString,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let Some(block) = value.borrowed_block() else {
            return Ok(());
        };
        // The table is plain data that no panic leaves half changed, so a poisoned lock is usable.
        let mut listed = self.listed.lock().unwrap_or_else(PoisonError::into_inner);
        if listed.find(block).is_some() {
            return Ok(());
        }
        for _ in 0..listed.shifted_by_insert(block) {
            step()?;
        }
        listed.reserve(1).map_err(|_| DomError::AllocationFailed)?;
        if self.registry.adopt_block(block, admit, step)? {
            listed
                .insert(block, block.clone())
                .map_err(|_| DomError::AllocationFailed)?;
        }
        Ok(())
    }

    /// Keeps the blocks this transaction relies on, because it published.
    pub(crate) fn publish(&self) {
        let mut listed = self.listed.lock().unwrap_or_else(PoisonError::into_inner);
        self.registry.publish_blocks(listed.values());
        *listed = TextBlockTable::new();
    }
}

impl Drop for BlockAdoptions {
    fn drop(&mut self) {
        let listed = self
            .listed
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner);
        self.registry.release_claims(listed.values());
        *listed = TextBlockTable::new();
    }
}

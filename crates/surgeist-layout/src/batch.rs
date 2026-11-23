//! Completed layout batches, their entries and atomic publication to a sink.

pub use crate::block_fragmentation::input::{
    LayoutPublicationTargets, LayoutSourceBasis, LayoutTargetBasis,
};
pub use crate::block_fragmentation::publication::{
    LayoutBatchApplyError, LayoutBatchCommit, LayoutBatchPreparation,
};
pub use crate::output::{
    CompletedLayoutBatch, CompletedLayoutBatchOf, InlineFragmentOutputEntry,
    InlineFragmentOutputEntryOf, LayoutCacheClearEntry, LayoutCacheStoreEntry,
    LayoutCacheStoreEntryOf, LayoutOutputEntry, LayoutOutputEntryOf,
};

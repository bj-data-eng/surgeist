//! Completed layout batches, their entries, and their atomic publication to a sink.
//!
//! A successful run returns a [`CompletedLayoutBatchOf`]. Its entries are a [`LayoutOutputEntryOf`] for each node
//! (geometry with its private rounding custody), [`LayoutCacheStoreEntryOf`] and [`LayoutCacheClearEntry`] for the
//! host's cache, and [`InlineFragmentOutputEntryOf`] for each retained Text fragment.
//! [`CompletedLayoutBatchOf::apply_to`] publishes the batch to a [`LayoutBatchSink`](crate::tree::LayoutBatchSink):
//! the sink receives a call-scoped [`LayoutBatchPreparation`] and then a [`LayoutBatchCommit`], and a failed
//! publication reports a [`LayoutBatchApplyError`]. The sink keeps its [`LayoutPublicationTargets`] with its visible
//! state; [`LayoutSourceBasis`] and [`LayoutTargetBasis`] are the host's promise about the immutable source and the
//! authorization for one source-bound root request.

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

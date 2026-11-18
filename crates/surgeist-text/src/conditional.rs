//! Conditional searches and phrase pieces.
//!
//! [`PreparedConditionalSearch`], [`PreparedConditionalBatch`] and [`PreparedConditionalOpportunity`] describe a
//! same-preparation query, one issued tier of candidate variants and an eligible cut; [`ConditionalPurpose`],
//! [`ConditionalStage`], [`ConditionalDisposition`] and [`ConditionalLineContext`] describe the query, its
//! relaxation tiers, the dispositions of variants and the line context. [`PhraseOverflowDecision`] is the
//! consumer-owned fit decision, and [`PreparedPhrase`], [`PreparedPhrasePiece`], [`PreparedPhrasePieceOwner`],
//! [`PreparedPhrasePortion`] and [`PreparedPhraseProvenance`] describe a final-analysis phrase and its pieces.

pub use crate::protocol::{
    ConditionalDisposition, ConditionalLineContext, ConditionalPurpose, ConditionalStage,
    PhraseOverflowDecision, PreparedConditionalBatch, PreparedConditionalOpportunity,
    PreparedConditionalSearch, PreparedPhrase, PreparedPhrasePiece, PreparedPhrasePieceOwner,
    PreparedPhrasePortion, PreparedPhraseProvenance,
};

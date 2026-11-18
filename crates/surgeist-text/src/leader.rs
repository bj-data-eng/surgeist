//! Isolated leader-pattern preparation and readback.
//!
//! [`LeaderPatternInput`] is the used Text context for one isolated pattern and [`LeaderPattern`] its decoded
//! source spelling. [`FinalizedLeaderPattern`] is the finalized repetition unit, read through
//! [`LeaderRunView`], [`LeaderClusterView`] and [`LeaderPaintGroup`]; [`LeaderCombinationContext`] is the
//! generated inline context for horizontal-in-vertical composition.

pub use crate::engine::leader::{
    FinalizedLeaderPattern, LeaderClusterView, LeaderCombinationContext, LeaderPaintGroup,
    LeaderPattern, LeaderPatternInput, LeaderRunView,
};

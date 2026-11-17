//! Stacking-context membership and child context order from resolved box facts.
//!
//! [`plan_stacking`] takes an immutable slice of [`BoxFacts`], one per
//! rendering box, and returns a [`StackingPlan`]: which actual stacking context
//! each box belongs to and the order of each context's immediate child
//! contexts. A box is identified by a [`BoxId`], its slot in that slice; the
//! slot is not a DOM identity.
//!
//! The facts are explicit caller input ([`ChildOrder`], [`Participation`],
//! [`Position`], [`ZIndex`], [`ContextEffect`]); nothing is inferred from
//! layout geometry. The plan stops at contexts: it does not order primitives.
//! Use [`crate::paint_order`] for the supported non-table primitive sequence,
//! which reuses this algorithm inside one operation.
//!
//! Every call runs under the limits and profiling of a
//! [`PaintOptions`](crate::accounting::PaintOptions) and returns a
//! [`StackingOutcome`] holding the complete plan or a
//! [`PaintError`](crate::PaintError), with the accounting report of the work
//! that was admitted.

pub use crate::algorithm::{ContextView, StackingOutcome, StackingPlan, plan_stacking};
pub use crate::model::{
    BoxFacts, BoxId, ChildOrder, ContextEffect, Participation, Position, ZIndex,
};

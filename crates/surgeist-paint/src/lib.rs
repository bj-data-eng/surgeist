#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod accounting;
mod algorithm;
mod model;

pub use accounting::{
    AccountingFailure, BudgetRefusal, PaintChange, PaintEvent, PaintLimits, PaintOptions,
    PaintPhase, PaintReport, PhaseReport, RefusalReason, ResourceKind, WorkKind,
};
pub use algorithm::{PaintOutcome, plan_stacking};
pub use model::{
    BoxFacts, BoxId, ChildOrder, ContextEffect, ContextView, PaintError, Participation, Position,
    StackingPlan, ZIndex,
};
pub use surgeist_ledger::{EventFilter, ProfileMode};

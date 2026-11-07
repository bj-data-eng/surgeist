#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod accounting;
mod algorithm;
mod model;
mod paint_order;

pub use accounting::{
    AccountingFailure, BudgetRefusal, PaintChange, PaintEvent, PaintLimits, PaintOptions,
    PaintPhase, PaintReport, PhaseReport, RefusalReason, ResourceKind, WorkKind,
};
pub use algorithm::{PaintOutcome, plan_stacking};
pub use model::{
    BoxFacts, BoxId, ChildOrder, ContextEffect, ContextView, PaintError, Participation, Position,
    StackingPlan, ZIndex,
};
pub use paint_order::{
    BlockInInlinePlacement, BoxCategoryError, BoxPaint, BoxPaintKind, BoxView, CanvasPaintKind,
    ContentEntry, DecorationKind, FloatParticipation, FragmentId, FragmentOwner, FragmentPaintKind,
    FragmentView, LineId, LineView, PaintFragment, PaintInputError, PaintLine, PaintOrder,
    PaintOrderEvent, PaintOrderOutcome, PaintPrimitive, PaintRequest, PrimitiveId, PrimitiveKind,
    PrimitiveView, SourceChild, TextId, TextSource, plan_paint_order,
};
pub use surgeist_ledger::{EventFilter, ProfileMode};

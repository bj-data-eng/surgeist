//! Validated, ordered paint primitives inside actual stacking contexts.
//!
//! [`plan_paint_order`] orders already-resolved, non-table CSS paint records.
//! A [`PaintRequest`] borrows six tables: the semantic
//! [`BoxFacts`](crate::stacking::BoxFacts), same-slot [`BoxPaint`] categories
//! and float facts, logical [`TextSource`] records, [`PaintLine`]
//! associations, [`PaintFragment`] occurrences and [`PaintPrimitive`]
//! appearance records. Records refer to each other by the request-local slots
//! [`TextId`], [`LineId`], [`FragmentId`] and [`PrimitiveId`] (and
//! [`BoxId`](crate::stacking::BoxId)); a slot copied from another request
//! cannot be told apart from a local one, so callers keep their own maps from
//! slots to geometry and source identity. Constructors preserve raw facts; the
//! operation validates the relations between the tables and returns a typed
//! [`PaintInputError`] for the first it finds.
//!
//! The result is a [`PaintOrder`]: borrowed [`PaintOrderEvent`]s that open and
//! close each actual stacking context and draw each primitive once, plus
//! [`BoxView`], [`FragmentView`], [`LineView`] and [`PrimitiveView`]
//! observations of the exact request records. Table and inline-table boxes are
//! refused with [`PaintInputError::UnsupportedBoxKind`]; the order is not a
//! backend command stream or a complete transform and clip tree, and it does not
//! authenticate computed style.
//!
//! One Ledger operation, limited and profiled by the
//! [`PaintOptions`] passed in, covers context
//! planning, input validation, sequencing, publication and cleanup.

pub use crate::algorithm::paint_model::{
    BlockInInlinePlacement, BoxCategoryError, BoxPaint, BoxPaintKind, BoxView, CanvasPaintKind,
    ContentEntry, DecorationKind, FloatParticipation, FragmentId, FragmentOwner, FragmentPaintKind,
    FragmentView, LineId, LineView, PaintFragment, PaintInputError, PaintLine, PaintOrder,
    PaintOrderEvent, PaintPrimitive, PaintRequest, PrimitiveId, PrimitiveKind, PrimitiveView,
    SourceChild, TextId, TextSource,
};

use crate::PaintError;
use crate::accounting::{OperationKind, PaintOptions, PaintPhase, PaintReport, finish};
use crate::algorithm;
use surgeist_ledger::{Limits, Tariffs};

/// One complete order or a typed refusal, with the operation's accepted prefix.
#[derive(Debug)]
pub struct PaintOrderOutcome<'a> {
    pub result: Result<PaintOrder<'a>, PaintError>,
    pub report: Option<PaintReport>,
}

/// Orders resolved non-table paint records while preserving source associations.
/// One Ledger operation owns topology, actual contexts, incidence, sequencing,
/// publication and cleanup. Caller records are never backend commands or an
/// authenticated computed-style capture.
pub fn plan_paint_order(request: PaintRequest<'_>, options: PaintOptions) -> PaintOrderOutcome<'_> {
    let mut meter = match options.operation(OperationKind::PaintOrder) {
        Ok(meter) => meter,
        Err(error) => {
            return PaintOrderOutcome {
                result: Err(error),
                report: None,
            };
        }
    };
    let root = meter.root();
    let scopes = (|| {
        let core = algorithm::setup_scopes(&mut meter)?;
        let validation = meter.child(
            root,
            PaintPhase::PaintValidation,
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
        let sequence = meter.child(
            root,
            PaintPhase::PaintSequence,
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
        Ok::<_, PaintError>((core, validation, sequence))
    })();
    let (scopes, validate, sequence) = match scopes {
        Ok(scopes) => scopes,
        Err(error) => {
            let (result, report) = finish(meter, Err(error));
            return PaintOrderOutcome { result, report };
        }
    };
    let result = algorithm::plan_order(request, &mut meter, root, &scopes, validate, sequence);
    let (result, report) = finish(meter, result);
    PaintOrderOutcome { result, report }
}

impl From<PaintInputError> for crate::PaintError {
    fn from(error: PaintInputError) -> Self {
        Self::Input(error)
    }
}

#[cfg(test)]
mod accounting_tests;
#[cfg(test)]
mod tests;

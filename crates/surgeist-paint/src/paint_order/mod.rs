mod model;
mod sequencing;
mod validation;

pub use model::*;

use crate::{
    PaintError, PaintOptions, PaintPhase, PaintReport, ResourceKind, StackingPlan,
    accounting::{OperationKind, apply, finish, setup_error},
    algorithm,
};
use std::mem::{size_of, take};
use surgeist_ledger::{Change, Limits, Tariffs};

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
    let mut core = algorithm::State::default();
    let mut index = validation::PaintIndex::default();
    let mut output = sequencing::SequenceState::default();
    let result = (|| {
        algorithm::prepare(
            &mut meter,
            root,
            &scopes,
            &mut core,
            request.boxes(),
            request.root(),
        )?;
        index = meter
            .with_phase(validate, |p| {
                validation::prepare(request, &core, p, root, validate)
            })
            .map_err(setup_error)??;
        meter
            .with_phase(sequence, |p| {
                sequencing::run(request, &core, &index, &mut output, p, root, sequence)
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[5], |p| {
                let plan = PaintOrder {
                    request,
                    stacking: StackingPlan {
                        facts: request.boxes(),
                        membership: take(&mut core.membership),
                        contexts: take(&mut core.contexts),
                        children: take(&mut core.children),
                    },
                    events: take(&mut output.events),
                };
                let amount = u64::try_from(size_of::<PaintOrder<'_>>())
                    .map_err(|_| PaintError::CapacityOverflow)?;
                match apply(
                    p,
                    root,
                    Change::Acquire(ResourceKind::RetainedResultBytes, amount),
                ) {
                    Ok(()) => Ok(plan),
                    Err(error) => {
                        core.membership = plan.stacking.membership;
                        core.contexts = plan.stacking.contexts;
                        core.children = plan.stacking.children;
                        output.events = plan.events;
                        Err(error)
                    }
                }
            })
            .map_err(setup_error)?
    })();
    let cleanup = meter
        .with_phase(scopes[6], |p| {
            core.scratch_cleanup(p, root)?;
            index.cleanup(p, root)?;
            output.scratch_cleanup(p, root)?;
            if result.is_err() {
                core.result_cleanup(p, root)?;
                output.result_cleanup(p, root)?;
            }
            Ok(())
        })
        .map_err(setup_error)
        .and_then(|done| done);
    let result = result.and_then(|value| cleanup.map(|()| value));
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

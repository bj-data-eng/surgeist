pub use crate::algorithm::paint_model::*;

use crate::{
    PaintError, PaintOptions, PaintPhase, PaintReport,
    accounting::{OperationKind, finish},
    algorithm,
};
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

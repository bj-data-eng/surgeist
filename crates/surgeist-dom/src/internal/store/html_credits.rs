//! The DOM's charge of `surgeist-html` work into its own work owners.
//!
//! The DOM counts the work of HTML operations in credits: each work dimension weighs
//! the schema's credit rate, and a batch is the sum of its dimensions. A work owner
//! takes that sum.

use super::DomError;
use surgeist_html::accounting::{
    HtmlChange, HtmlChargeOverflow, HtmlChargedError, HtmlResource, HtmlWork,
};
use surgeist_ledger::{Change, Charge};

/// A [`Charge`] that hands the credits of each batch to `spend`, which stops with the caller's
/// error type `E`. A change that is not work, or a work dimension with no credit rate, is
/// [`DomError::HtmlChargeProtocol`]. A sum that does not fit `u64` is
/// [`DomError::RepresentationExhausted`], and no credit is spent.
pub(super) struct Credits<F>(pub(super) F);

impl<E: From<DomError>, F: FnMut(u64) -> Result<(), E>> Charge<HtmlWork, HtmlResource>
    for Credits<F>
{
    type Stop = E;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), E> {
        let mut credits = 0_u64;
        for change in batch {
            let Change::Work(dimension, amount) = *change else {
                return Err(DomError::HtmlChargeProtocol.into());
            };
            let Some(rate) = dimension.credits_per_unit() else {
                return Err(DomError::HtmlChargeProtocol.into());
            };
            credits = amount
                .checked_mul(rate)
                .and_then(|weighed| credits.checked_add(weighed))
                .ok_or(DomError::RepresentationExhausted)?;
        }
        (self.0)(credits)
    }

    fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), E> {
        Err(DomError::HtmlChargeProtocol.into())
    }
}

/// The caller's error for a charge that can also fault on an amount the ledger cannot hold.
pub(super) fn charged<E: From<DomError>>(error: HtmlChargedError<E, HtmlChargeOverflow>) -> E {
    match error {
        HtmlChargedError::Stopped(stop) => stop,
        HtmlChargedError::Faulted(HtmlChargeOverflow) => DomError::RepresentationExhausted.into(),
    }
}

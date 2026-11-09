//! The accounting schema that the HTML crates charge through `surgeist-ledger`.
//!
//! The schema declares the closed work and resource dimension sets with their
//! units, custody policy and conversion rates, and names the scope that carries
//! each cap. The bounded operations of this crate charge these dimensions through
//! a caller's [`surgeist_ledger::Charge`]. See [`HtmlWork`] and [`HtmlResource`]
//! for the dimensions, [`charging_tariffs`] and [`aggregate_tariffs`] for the
//! conversion rates, [`HtmlCapScope`] for cap placement and [`HtmlChargedError`]
//! for an operation that can also fault by itself. [`HtmlAccountingFault`] is the one
//! name a ledger condition has in every HTML crate, [`classify`] derives it from a
//! ledger error, and [`HtmlAuxiliaryScope`] with [`HtmlCapScope`] names every scope the
//! HTML crates create.

pub use crate::internal::accounting::{
    HtmlAccountingFault, HtmlAuxiliaryScope, HtmlCapRefusal, HtmlCapScope, HtmlChange,
    HtmlChargeOverflow, HtmlChargedError, HtmlCustody, HtmlHostScope, HtmlLedgerOutcome,
    HtmlLimits, HtmlRefusalFigures, HtmlResource, HtmlResourceTable, HtmlTariffs, HtmlUnit,
    HtmlWork, HtmlWorkTable, aggregate_tariffs, charging_tariffs, classify, fault_of,
    record_largest, retained_provider_storage,
};

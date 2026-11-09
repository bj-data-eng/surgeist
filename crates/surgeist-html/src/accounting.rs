//! The accounting schema that the HTML crates charge through `surgeist-ledger`.
//!
//! The schema declares the closed work and resource dimension sets with their
//! units, custody policy and conversion rates, names the scope that carries each
//! cap, and converts the work, layout and limits shapes of this crate into
//! ledger changes and limits. See [`HtmlWork`] and [`HtmlResource`] for the
//! dimensions, [`charging_tariffs`] and [`aggregate_tariffs`] for the conversion
//! rates, and [`HtmlCapScope`] for cap placement.

pub use crate::internal::accounting::{
    HtmlCapScope, HtmlChange, HtmlChargeOverflow, HtmlChargedError, HtmlCustody, HtmlLimits,
    HtmlResource, HtmlResourceTable, HtmlTariffs, HtmlUnit, HtmlWork, HtmlWorkTable,
    aggregate_tariffs, attribute_edit_entries, attribute_edit_work, autocomplete_limits,
    charging_tariffs, construction_limits, csp_storage, csp_work, record_largest,
    retained_provider_storage, text_append_work, value_pool_usage,
};

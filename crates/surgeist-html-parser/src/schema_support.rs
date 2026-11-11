//! Helpers for the tests that compare the parser's own charges with the HTML
//! accounting schema.

use crate::{HtmlParseWorkCategory, parser::control::ParseControl};
use surgeist_html::accounting::{HtmlLimits, HtmlResource, HtmlWork, charging_tariffs};
use surgeist_ledger::{Error, Operation, ScopeCharge};

/// Each parser work category with the schema dimension that carries it.
pub(crate) const CATEGORIES: [(HtmlParseWorkCategory, HtmlWork); 8] = [
    (
        HtmlParseWorkCategory::InputUnitRead,
        HtmlWork::InputUnitRead,
    ),
    (HtmlParseWorkCategory::InspectByte, HtmlWork::InspectByte),
    (HtmlParseWorkCategory::CopyByte, HtmlWork::CopyByte),
    (HtmlParseWorkCategory::StateStep, HtmlWork::StateStep),
    (HtmlParseWorkCategory::RecordVisit, HtmlWork::RecordVisit),
    (HtmlParseWorkCategory::RecordWrite, HtmlWork::RecordWrite),
    (HtmlParseWorkCategory::ReserveByte, HtmlWork::ReserveByte),
    (HtmlParseWorkCategory::ProviderCall, HtmlWork::ProviderCall),
];

/// Work per category and the credit total.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Amounts {
    pub(crate) categories: [u64; 8],
    pub(crate) credits: u64,
}

/// The amounts an operation of `surgeist-html` records when it charges a scope
/// of a real ledger operation, the stock implementation of the charging trait.
pub(crate) fn through_ledger(
    charge: impl FnOnce(
        &mut ScopeCharge<'_, HtmlWork, HtmlResource>,
    ) -> Result<(), Error<HtmlWork, HtmlResource>>,
) -> Amounts {
    let mut operation = Operation::new("cap", HtmlLimits::unlimited(), 2).unwrap();
    let cap = operation.root();
    let charging = operation
        .child(cap, "charging", HtmlLimits::unlimited(), charging_tariffs())
        .unwrap();
    charge(&mut operation.charge_at(charging)).unwrap();
    let mut categories = [0; 8];
    for (slot, (_, dimension)) in categories.iter_mut().zip(CATEGORIES) {
        *slot = operation.usage(charging).unwrap().work(dimension);
    }
    Amounts {
        categories,
        credits: operation.usage(cap).unwrap().work(HtmlWork::Credits),
    }
}

/// The amounts a controller has charged so far.
pub(crate) fn charged(control: &ParseControl) -> Amounts {
    let mut categories = [0; 8];
    for (slot, (category, _)) in categories.iter_mut().zip(CATEGORIES) {
        *slot = control.usage().work(category);
    }
    Amounts {
        categories,
        credits: control
            .usage()
            .resource(crate::HtmlParseResource::WorkCredits)
            .current(),
    }
}

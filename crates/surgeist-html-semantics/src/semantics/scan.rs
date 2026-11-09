//! The metered scan kit shared by the semantic session and the fact constructors.
//!
//! Every fact constructor and the session charge the same three things the same
//! way: the vocabulary-recognition envelope of an element name, the comparison
//! of an element's namespace with an expected URI, and the lookup of a node.
//! They differ only in the error they report and in what they do when a charge
//! is refused. The helpers here take that error from the caller and preserve
//! each caller's charge order and first reached failure.

use super::*;
use crate::accounting::wide;
use crate::{HtmlCapture, HtmlCaptureError, HtmlCapturedElement, HtmlCapturedNode};
use std::{fmt, marker::PhantomData};
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlAuxiliaryScope, HtmlCapScope, HtmlLedgerOutcome, HtmlLimits,
    HtmlResource, HtmlWork, classify,
};
use surgeist_html::vocabulary::KnownHtmlElement;
use surgeist_ledger::{Change, Error, Operation, Scope, Tariff, Tariffs};

/// The namespace URI of HTML elements.
pub(super) const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

/// Units that admit one recognition of an element name in the HTML vocabulary:
/// `(local-name units + 1) * KnownHtmlElement::ALL.len() + namespace units`.
///
/// The envelope covers every candidate spelling and all supplied-name units
/// before the vocabulary owner performs its bounded recognition. `None` is an
/// overflow of the arithmetic, which every caller reports as its own
/// representation-exhausted failure.
pub(super) fn recognition_envelope(element: &HtmlCapturedElement) -> Option<usize> {
    let name = element.name();
    name.local_name()
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
        .and_then(|n| n.checked_add(name.namespace().uri().map_or(0, |uri| uri.len())))
}

/// Compares an element's namespace with `expected`, reporting each unit of work
/// through `charge` before it is done: 1 for the test, then 1 per compared
/// UTF-16 unit when the lengths are equal, stopping at the first mismatch. The
/// 28-unit HTML namespace therefore costs 29 on a match.
pub(super) fn namespace_is_with<E>(
    element: &HtmlCapturedElement,
    expected: &str,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, E> {
    charge(1)?;
    let Some(uri) = element.name().namespace().uri() else {
        return Ok(false);
    };
    if uri.len() != expected.len() {
        return Ok(false);
    }
    for (unit, byte) in uri.code_units().iter().zip(expected.bytes()) {
        charge(1)?;
        if *unit != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The refusals a fact constructor's validation scan can report. Each
/// constructor keeps its own error type and its own spelling of the work refusal.
pub(crate) trait ScanError: Sized {
    /// A work total or envelope that does not fit in `usize`.
    fn representation_exhausted() -> Self;
    /// The validation work limit was exceeded: `required` is the total that was refused.
    fn work_exceeded(limit: usize, required: usize) -> Self;
    /// The capture refused a node lookup.
    fn capture(error: HtmlCaptureError) -> Self;
    /// The ledger that enforces the limits failed in a way that is not a refused limit.
    fn accounting(fault: HtmlAccountingFault) -> Self;
}

/// The caps of one constructor's ledger scope. A cap left unset is unlimited.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Caps {
    work: usize,
    records: usize,
    bytes: usize,
    code_units: usize,
    assigned_nodes: usize,
}
impl Caps {
    /// Caps the validation work and nothing else.
    pub(crate) const fn work(work: usize) -> Self {
        Self {
            work,
            records: usize::MAX,
            bytes: usize::MAX,
            code_units: usize::MAX,
            assigned_nodes: usize::MAX,
        }
    }
    pub(crate) const fn records(mut self, records: usize) -> Self {
        self.records = records;
        self
    }
    pub(crate) const fn bytes(mut self, bytes: usize) -> Self {
        self.bytes = bytes;
        self
    }
    pub(crate) const fn code_units(mut self, code_units: usize) -> Self {
        self.code_units = code_units;
        self
    }
    pub(crate) const fn assigned_nodes(mut self, assigned_nodes: usize) -> Self {
        self.assigned_nodes = assigned_nodes;
        self
    }
    fn limits(self) -> HtmlLimits {
        HtmlLimits::from_fn(
            |work| match work {
                HtmlWork::SemanticStep => wide(self.work),
                _ => u64::MAX,
            },
            |resource| match resource {
                HtmlResource::FactRecords => wide(self.records),
                HtmlResource::FactBytes => wide(self.bytes),
                HtmlResource::FactCodeUnits => wide(self.code_units),
                HtmlResource::AssignedNodes => wide(self.assigned_nodes),
                _ => u64::MAX,
            },
        )
    }
}

/// A validation-work total and the fact counts of one constructor, charged to a ledger scope.
///
/// The scope belongs to an operation of the meter's own and carries the constructor's caps. The
/// meter reports refusals as the constructor's own error `E`. A refused charge leaves the total
/// unchanged, so a refusal reports exactly the work that was admitted before it.
pub(crate) struct ScanMeter<E> {
    operation: Operation<HtmlWork, HtmlResource>,
    /// The scope that carries the caps.
    scope: Scope,
    /// The scope that takes every fact amount first, whether or not the caps admit it yet. It
    /// projects nothing into the capped scope.
    grown: Scope,
    error: PhantomData<fn() -> E>,
}
impl<E> fmt::Debug for ScanMeter<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScanMeter").finish_non_exhaustive()
    }
}
impl<E: ScanError> ScanMeter<E> {
    /// A meter in a scope named for `kind` that carries `caps`.
    pub(crate) fn new(kind: HtmlCapScope, caps: Caps) -> Result<Self, E> {
        let mut operation =
            Operation::new(kind.name(), caps.limits(), 2).map_err(Self::refusal_of)?;
        let scope = operation.root();
        let grown = operation
            .child(
                scope,
                HtmlAuxiliaryScope::FactGrown.name(),
                HtmlLimits::unlimited(),
                Tariffs::from_fn(
                    |work| Tariff::new(work, 0),
                    |resource| Tariff::new(resource, 0),
                ),
            )
            .map_err(Self::refusal_of)?;
        Ok(Self {
            operation,
            scope,
            grown,
            error: PhantomData,
        })
    }
    /// The typed refusal of a ledger error. A refused work cap is the constructor's work refusal,
    /// an overflow its representation exhaustion, and anything else an accounting fault.
    fn refusal_of(error: Error<HtmlWork, HtmlResource>) -> E {
        match classify(error) {
            HtmlLedgerOutcome::Fault(fault) => E::accounting(fault),
            HtmlLedgerOutcome::Exhausted(_) => E::representation_exhausted(),
            HtmlLedgerOutcome::Cap(refusal) => match (refusal.change(), refusal.figures()) {
                (Change::Work(HtmlWork::SemanticStep, _), Ok(figures)) => {
                    E::work_exceeded(figures.limit, figures.required)
                }
                (Change::Work(HtmlWork::SemanticStep, _), Err(_)) => E::representation_exhausted(),
                _ => E::accounting(HtmlAccountingFault::UnexpectedChange),
            },
        }
    }
    /// The typed refusal of a ledger error at the cap of a fact `resource`: the refused total goes
    /// through `refuse`.
    fn resource_refusal(
        error: Error<HtmlWork, HtmlResource>,
        resource: HtmlResource,
        refuse: impl FnOnce(usize, usize) -> E,
    ) -> E {
        match classify(error) {
            HtmlLedgerOutcome::Fault(fault) => E::accounting(fault),
            HtmlLedgerOutcome::Exhausted(_) => E::representation_exhausted(),
            HtmlLedgerOutcome::Cap(refusal) => match (refusal.change(), refusal.figures()) {
                (Change::Acquire(refused, _), Ok(figures)) if refused == resource => {
                    refuse(figures.limit, figures.required)
                }
                (Change::Acquire(refused, _), Err(_)) if refused == resource => {
                    E::representation_exhausted()
                }
                _ => E::accounting(HtmlAccountingFault::UnexpectedChange),
            },
        }
    }
    /// The work admitted so far.
    pub(crate) fn work(&self) -> Result<usize, E> {
        let work = self
            .operation
            .usage(self.scope)
            .map_err(Self::refusal_of)?
            .work(HtmlWork::SemanticStep);
        usize::try_from(work).map_err(|_| E::representation_exhausted())
    }
    pub(crate) fn charge(&mut self, amount: usize) -> Result<(), E> {
        self.operation
            .apply(
                self.scope,
                Change::Work(HtmlWork::SemanticStep, wide(amount)),
            )
            .map_err(Self::refusal_of)
    }
    /// Takes `amount` of a fact resource without checking its cap, which `check` does where the
    /// constructor's own order of refusals requires.
    pub(crate) fn grow(&mut self, resource: HtmlResource, amount: usize) -> Result<(), E> {
        self.operation
            .apply(self.grown, Change::Acquire(resource, wide(amount)))
            .map_err(Self::refusal_of)
    }
    /// Refuses a total of a fact resource beyond its cap. A refusal is reported through `refuse`
    /// with the limit and the total taken.
    pub(crate) fn check(
        &mut self,
        resource: HtmlResource,
        refuse: impl FnOnce(usize, usize) -> E,
    ) -> Result<(), E> {
        let live = |meter: &Self, scope| {
            meter
                .operation
                .usage(scope)
                .map(|usage| usage.live(resource))
                .map_err(Self::refusal_of)
        };
        let owed = live(self, self.grown)?.saturating_sub(live(self, self.scope)?);
        if owed == 0 {
            return Ok(());
        }
        self.operation
            .apply(self.scope, Change::Acquire(resource, owed))
            .map_err(|error| Self::resource_refusal(error, resource, refuse))
    }
    /// Admits `amount` of a fact resource: takes it, then checks the cap.
    pub(crate) fn acquire(
        &mut self,
        resource: HtmlResource,
        amount: usize,
        refuse: impl FnOnce(usize, usize) -> E,
    ) -> Result<(), E> {
        self.grow(resource, amount)?;
        self.check(resource, refuse)
    }
    /// Refuses `amount` of a fact resource that would not fit its cap, taking none of it.
    pub(crate) fn fits(
        &mut self,
        resource: HtmlResource,
        amount: usize,
        refuse: impl FnOnce(usize, usize) -> E,
    ) -> Result<(), E> {
        let amount = wide(amount);
        self.operation
            .check_batch(self.scope, &[Change::Acquire(resource, amount)])
            .map_err(|error| Self::resource_refusal(error, resource, refuse))
    }
    /// The total of a fact resource taken so far, checked or not.
    pub(crate) fn taken(&self, resource: HtmlResource) -> Result<usize, E> {
        let live = self
            .operation
            .usage(self.grown)
            .map_err(Self::refusal_of)?
            .live(resource);
        usize::try_from(live).map_err(|_| E::representation_exhausted())
    }
    /// Looks up a node, charging 1 before the lookup.
    pub(crate) fn node<'a>(
        &mut self,
        capture: &'a HtmlCapture,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCapturedNode<'a>, E> {
        self.charge(1)?;
        capture.node(id).map_err(E::capture)
    }
    /// Recognizes an element name in the HTML vocabulary, charging its envelope first.
    pub(crate) fn known(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<Option<KnownHtmlElement>, E> {
        let envelope = recognition_envelope(element).ok_or_else(E::representation_exhausted)?;
        self.charge(envelope)?;
        Ok(KnownHtmlElement::from_name(element.name()))
    }
    /// Whether the element is in the HTML namespace, charged as `namespace_is_with`.
    pub(crate) fn html_namespace(&mut self, element: &HtmlCapturedElement) -> Result<bool, E> {
        namespace_is_with(element, HTML_NAMESPACE, |amount| self.charge(amount))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCapturedIsValue,
        HtmlCapturedNodeKind, HtmlScriptingStatus,
    };
    use surgeist_html::{
        construction::HtmlDocumentMode,
        values::{Namespace, OrderedAttributes, QualifiedName},
    };

    #[derive(Debug, Eq, PartialEq)]
    enum Refusal {
        Exhausted,
        Work { limit: usize, required: usize },
        Capture,
        Accounting(HtmlAccountingFault),
    }
    impl ScanError for Refusal {
        fn representation_exhausted() -> Self {
            Self::Exhausted
        }
        fn work_exceeded(limit: usize, required: usize) -> Self {
            Self::Work { limit, required }
        }
        fn capture(_: HtmlCaptureError) -> Self {
            Self::Capture
        }
        fn accounting(fault: HtmlAccountingFault) -> Self {
            Self::Accounting(fault)
        }
    }

    /// A capture with an HTML `div` and an SVG `circle`, in that order.
    fn capture() -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(10, 100, 10, 1000, 10, 0, 0, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let mut element = |name: QualifiedName| {
            builder
                .add_element(
                    &document,
                    name,
                    OrderedAttributes::new(vec![]).unwrap(),
                    HtmlCapturedIsValue::Unavailable,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap()
        };
        let div = element(KnownHtmlElement::Div.qualified_name());
        let circle = element(QualifiedName::new(Namespace::svg(), None, "circle"));
        builder
            .establish_children(&document, &[div.clone(), circle.clone()])
            .unwrap();
        builder.establish_children(&div, &[]).unwrap();
        builder.establish_children(&circle, &[]).unwrap();
        let capture = builder.finish().unwrap();
        let div = capture.subject_id(&div).unwrap();
        let circle = capture.subject_id(&circle).unwrap();
        (capture, div, circle)
    }
    fn element<'a>(capture: &'a HtmlCapture, id: &HtmlCaptureSubjectId) -> &'a HtmlCapturedElement {
        match capture.node(id).unwrap().kind() {
            HtmlCapturedNodeKind::Element(element) => element,
            _ => panic!("expected an element"),
        }
    }

    #[test]
    fn envelope_covers_every_candidate_spelling_and_the_namespace_units() {
        let (capture, div, circle) = capture();
        // "div" is 3 units and the HTML namespace 28; "circle" is 6 and the SVG namespace 26.
        assert_eq!(
            recognition_envelope(element(&capture, &div)),
            Some(4 * KnownHtmlElement::ALL.len() + 28)
        );
        assert_eq!(
            recognition_envelope(element(&capture, &circle)),
            Some(7 * KnownHtmlElement::ALL.len() + 26)
        );
    }

    #[test]
    fn namespace_compare_charges_one_then_one_per_compared_unit() {
        let (capture, div, circle) = capture();
        let run = |element, expected: &str| {
            let mut charges = Vec::new();
            let result = namespace_is_with::<()>(element, expected, |n| {
                charges.push(n);
                Ok(())
            });
            (result, charges)
        };
        let (result, charges) = run(element(&capture, &div), HTML_NAMESPACE);
        assert_eq!(result, Ok(true));
        assert_eq!(charges, vec![1; 29], "the 28-unit HTML namespace costs 29");
        // A different length stops after the test itself.
        let (result, charges) = run(element(&capture, &circle), HTML_NAMESPACE);
        assert_eq!((result, charges), (Ok(false), vec![1]));
        // The same length stops at the first mismatching unit.
        let (result, charges) = run(element(&capture, &div), "xttp://www.w3.org/1999/xhtml");
        assert_eq!((result, charges), (Ok(false), vec![1, 1]));
        let (result, charges) = run(element(&capture, &div), "http://www.w3.org/1999/xhtmx");
        assert_eq!((result, charges), (Ok(false), vec![1; 29]));
    }

    #[test]
    fn the_first_refused_charge_ends_the_compare() {
        let (capture, div, _) = capture();
        let mut asked = 0;
        let result = namespace_is_with(element(&capture, &div), HTML_NAMESPACE, |_| {
            asked += 1;
            if asked == 3 { Err("stopped") } else { Ok(()) }
        });
        assert_eq!((result, asked), (Err("stopped"), 3));
    }

    fn meter(caps: Caps) -> ScanMeter<Refusal> {
        ScanMeter::new(HtmlCapScope::ControlFacts, caps).unwrap()
    }

    #[test]
    fn a_meter_names_its_scopes_by_the_schema() {
        let meter = meter(Caps::work(10));
        assert_eq!(
            meter.operation.scope_name(meter.scope),
            Ok(HtmlCapScope::ControlFacts.name())
        );
        assert_eq!(
            meter.operation.scope_name(meter.grown),
            Ok(HtmlAuxiliaryScope::FactGrown.name())
        );
    }

    #[test]
    fn meter_refusal_reports_the_refused_total_and_keeps_the_admitted_one() {
        let mut meter = meter(Caps::work(10));
        meter.charge(6).unwrap();
        assert_eq!(
            meter.charge(5),
            Err(Refusal::Work {
                limit: 10,
                required: 11
            })
        );
        assert_eq!(meter.work(), Ok(6), "a refusal does not consume the work");
        meter.charge(4).unwrap();
        assert_eq!(meter.work(), Ok(10));
        let mut exhausted = self::meter(Caps::work(usize::MAX));
        exhausted.charge(usize::MAX).unwrap();
        assert_eq!(exhausted.charge(1), Err(Refusal::Exhausted));
    }

    #[test]
    fn meter_lookups_charge_before_they_read() {
        let (capture, div, circle) = capture();
        let mut meter = meter(Caps::work(1_000_000));
        meter.node(&capture, &div).unwrap();
        assert_eq!(meter.work(), Ok(1));
        let known = meter.known(element(&capture, &div)).unwrap();
        assert_eq!(known, Some(KnownHtmlElement::Div));
        assert_eq!(meter.work(), Ok(1 + 4 * KnownHtmlElement::ALL.len() + 28));
        let before = meter.work().unwrap();
        assert!(meter.html_namespace(element(&capture, &div)).unwrap());
        assert_eq!(meter.work(), Ok(before + 29));
        assert!(!meter.html_namespace(element(&capture, &circle)).unwrap());
        assert_eq!(meter.work(), Ok(before + 30));
        // A refused envelope reads nothing: no vocabulary result without its charge.
        let mut tight = self::meter(Caps::work(3));
        assert_eq!(
            tight.known(element(&capture, &div)),
            Err(Refusal::Work {
                limit: 3,
                required: 4 * KnownHtmlElement::ALL.len() + 28
            })
        );
        assert_eq!(tight.work(), Ok(0));
    }

    #[test]
    fn a_fact_resource_cap_reports_its_limit_and_the_total_it_would_have_reached() {
        let mut meter = meter(Caps::work(10).records(5).bytes(100));
        let refuse = |limit, required| Refusal::Work { limit, required };
        meter.acquire(HtmlResource::FactRecords, 3, refuse).unwrap();
        assert_eq!(
            meter.acquire(HtmlResource::FactRecords, 3, refuse),
            Err(Refusal::Work {
                limit: 5,
                required: 6
            })
        );
        // The refused amount was taken before the cap refused it, as a grown total is; the cap
        // still holds only what it admitted.
        assert_eq!(meter.taken(HtmlResource::FactRecords), Ok(6));
        // A cap left unset is unlimited, and a sum beyond the ledger's range is exhaustion.
        meter
            .acquire(HtmlResource::FactCodeUnits, 1_000_000, refuse)
            .unwrap();
        assert_eq!(
            meter.acquire(HtmlResource::FactCodeUnits, usize::MAX, refuse),
            Err(Refusal::Exhausted)
        );
    }

    #[test]
    fn growing_checks_nothing_until_the_constructor_checks_and_checking_charges_the_difference() {
        let mut meter = meter(Caps::work(10).records(5));
        let refuse = |limit, required| Refusal::Work { limit, required };
        meter.grow(HtmlResource::FactRecords, 4).unwrap();
        meter.grow(HtmlResource::FactRecords, 3).unwrap();
        assert_eq!(meter.taken(HtmlResource::FactRecords), Ok(7));
        assert_eq!(
            meter.check(HtmlResource::FactRecords, refuse),
            Err(Refusal::Work {
                limit: 5,
                required: 7
            })
        );
        // `fits` checks an amount against the cap without taking it.
        let mut fits = self::meter(Caps::work(10).records(5));
        fits.fits(HtmlResource::FactRecords, 5, refuse).unwrap();
        assert_eq!(
            fits.fits(HtmlResource::FactRecords, 6, refuse),
            Err(Refusal::Work {
                limit: 5,
                required: 6
            })
        );
        assert_eq!(fits.taken(HtmlResource::FactRecords), Ok(0));
    }

    #[test]
    fn a_ledger_failure_that_is_not_a_refused_cap_is_the_constructors_accounting_fault() {
        let mut meter = meter(Caps::work(10));
        // A release that matches no custody is a state the constructors' own sequencing rules out;
        // an acquisition of another dimension than the one asked for cannot occur either.
        assert_eq!(
            ScanMeter::<Refusal>::refusal_of(Error::ForeignScope),
            Refusal::Accounting(HtmlAccountingFault::ForeignScope)
        );
        assert_eq!(
            ScanMeter::<Refusal>::refusal_of(Error::Refused(surgeist_ledger::Refusal {
                scope: meter.scope,
                item: 0,
                change: Change::Release(HtmlResource::FactRecords, 1),
                current: 0,
                reason: surgeist_ledger::RefusalReason::Underflow,
            })),
            Refusal::Accounting(HtmlAccountingFault::ReleaseUnderflow)
        );
        assert_eq!(
            ScanMeter::<Refusal>::refusal_of(Error::ScopeLimit { limit: 2 }),
            Refusal::Accounting(HtmlAccountingFault::ScopeLimit { limit: 2 })
        );
        meter.charge(1).unwrap();
    }

    #[test]
    fn a_failed_ledger_allocation_is_the_constructors_accounting_fault() {
        assert_eq!(
            ScanMeter::<Refusal>::refusal_of(Error::Allocation),
            Refusal::Accounting(HtmlAccountingFault::Allocation)
        );
    }
}

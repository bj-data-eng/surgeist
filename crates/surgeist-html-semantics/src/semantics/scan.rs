//! The metered scan kit shared by the semantic session and the fact constructors.
//!
//! Every fact constructor and the session charge the same three things the same
//! way: the vocabulary-recognition envelope of an element name, the comparison
//! of an element's namespace with an expected URI, and the lookup of a node (or
//! of a fact by subject). They differ only in the error they report and in what
//! they do when a charge is refused. The helpers here take that error from the
//! caller and preserve each caller's charge order and first reached failure.

use super::*;
use crate::{HtmlCapture, HtmlCaptureError, HtmlCapturedElement, HtmlCapturedNode};
use std::marker::PhantomData;
use surgeist_html::vocabulary::KnownHtmlElement;

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
pub(super) trait ScanError: Sized {
    /// A work total or envelope that does not fit in `usize`.
    fn representation_exhausted() -> Self;
    /// The validation work limit was exceeded: `required` is the total that was refused.
    fn work_exceeded(limit: usize, required: usize) -> Self;
    /// The capture refused a node lookup.
    fn capture(error: HtmlCaptureError) -> Self;
}

/// A validation-work total against its limit, with the lookups that charge it.
///
/// The meter reports refusals as the constructor's own error `E`. A refused
/// charge leaves the total unchanged, so a refusal reports exactly the work that
/// was admitted before it.
pub(super) struct ScanMeter<E> {
    work: usize,
    limit: usize,
    error: PhantomData<fn() -> E>,
}
impl<E: ScanError> ScanMeter<E> {
    pub(super) fn new(work: usize, limit: usize) -> Self {
        Self {
            work,
            limit,
            error: PhantomData,
        }
    }
    /// The work admitted so far.
    pub(super) fn work(&self) -> usize {
        self.work
    }
    /// The work limit this meter refuses beyond.
    pub(super) fn limit(&self) -> usize {
        self.limit
    }
    pub(super) fn charge(&mut self, amount: usize) -> Result<(), E> {
        let required = self
            .work
            .checked_add(amount)
            .ok_or_else(E::representation_exhausted)?;
        if required > self.limit {
            return Err(E::work_exceeded(self.limit, required));
        }
        self.work = required;
        Ok(())
    }
    /// Looks up a node, charging 1 before the lookup.
    pub(super) fn node<'a>(
        &mut self,
        capture: &'a HtmlCapture,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCapturedNode<'a>, E> {
        self.charge(1)?;
        capture.node(id).map_err(E::capture)
    }
    /// Recognizes an element name in the HTML vocabulary, charging its envelope first.
    pub(super) fn known(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<Option<KnownHtmlElement>, E> {
        let envelope = recognition_envelope(element).ok_or_else(E::representation_exhausted)?;
        self.charge(envelope)?;
        Ok(KnownHtmlElement::from_name(element.name()))
    }
    /// Whether the element is in the HTML namespace, charged as `namespace_is_with`.
    pub(super) fn html_namespace(&mut self, element: &HtmlCapturedElement) -> Result<bool, E> {
        namespace_is_with(element, HTML_NAMESPACE, |amount| self.charge(amount))
    }
    /// Finds the first fact of `facts` bound to `id`, charging 1 per fact examined.
    pub(super) fn fact<'a, T>(
        &mut self,
        facts: &'a [T],
        subject: impl Fn(&T) -> &HtmlCaptureSubjectId,
        id: &HtmlCaptureSubjectId,
    ) -> Result<Option<&'a T>, E> {
        for fact in facts {
            self.charge(1)?;
            if subject(fact) == id {
                return Ok(Some(fact));
            }
        }
        Ok(None)
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

    #[test]
    fn meter_refusal_reports_the_refused_total_and_keeps_the_admitted_one() {
        let mut meter = ScanMeter::<Refusal>::new(0, 10);
        meter.charge(6).unwrap();
        assert_eq!(
            meter.charge(5),
            Err(Refusal::Work {
                limit: 10,
                required: 11
            })
        );
        assert_eq!(meter.work(), 6, "a refusal does not consume the work");
        meter.charge(4).unwrap();
        assert_eq!(meter.work(), 10);
        assert_eq!(
            ScanMeter::<Refusal>::new(usize::MAX, usize::MAX).charge(1),
            Err(Refusal::Exhausted)
        );
    }

    #[test]
    fn meter_lookups_charge_before_they_read() {
        let (capture, div, circle) = capture();
        let mut meter = ScanMeter::<Refusal>::new(0, 1_000_000);
        meter.node(&capture, &div).unwrap();
        assert_eq!(meter.work(), 1);
        let known = meter.known(element(&capture, &div)).unwrap();
        assert_eq!(known, Some(KnownHtmlElement::Div));
        assert_eq!(meter.work(), 1 + 4 * KnownHtmlElement::ALL.len() + 28);
        let before = meter.work();
        assert!(meter.html_namespace(element(&capture, &div)).unwrap());
        assert_eq!(meter.work(), before + 29);
        assert!(!meter.html_namespace(element(&capture, &circle)).unwrap());
        assert_eq!(meter.work(), before + 30);
        // A refused envelope reads nothing: no vocabulary result without its charge.
        let mut tight = ScanMeter::<Refusal>::new(0, 3);
        assert_eq!(
            tight.known(element(&capture, &div)),
            Err(Refusal::Work {
                limit: 3,
                required: 4 * KnownHtmlElement::ALL.len() + 28
            })
        );
        assert_eq!(tight.work(), 0);
    }

    #[test]
    fn fact_lookup_charges_one_per_fact_examined() {
        let (_, div, circle) = capture();
        let facts = [div.clone(), circle.clone(), div.clone()];
        let mut meter = ScanMeter::<Refusal>::new(0, 100);
        let found = meter.fact(&facts, |fact| fact, &circle).unwrap();
        assert!(std::ptr::eq(found.unwrap(), &facts[1]));
        assert_eq!(meter.work(), 2);
        // No match examines every fact.
        let (_, other, _) = capture();
        assert!(meter.fact(&facts, |fact| fact, &other).unwrap().is_none());
        assert_eq!(meter.work(), 5);
        let mut tight = ScanMeter::<Refusal>::new(0, 1);
        assert_eq!(
            tight.fact(&facts, |fact| fact, &circle),
            Err(Refusal::Work {
                limit: 1,
                required: 2
            })
        );
    }
}

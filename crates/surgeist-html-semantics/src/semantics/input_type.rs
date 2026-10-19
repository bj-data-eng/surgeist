//! Finite semantic admission for HTML-owned input-type reports.

use super::*;
use crate::{HtmlAttributeReport, HtmlInputType};

impl HtmlSemanticSession<'_> {
    pub(super) fn admitted_input_type_report(
        &mut self,
        element: &HtmlCapturedElement,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlInputType>, HtmlSemanticFailure> {
        // INPUT_TYPE has 22 authored keyword rows. The former 23-row envelope
        // in control/direction was conservative; use the same 22-row allowance
        // as autocomplete, form settings and command targets. All other name
        // and namespace allowances remain in the existing intrinsic envelope.
        self.admit_intrinsic(element, raw, 22)?;
        // The HTML owner supplies recognition, author state, defaults and raw
        // provenance in one scan. HtmlInputTypeCursor supplies only a state;
        // using it here would require a second scan to obtain the full report.
        // Callers retain their existing raw-read order and post-operation polls
        // so consolidation introduces no separate cancellation-order change.
        self.interpreter
            .input_type(element.name(), raw)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{
        HtmlAttributeAuthorState, HtmlAttributeRecognition, HtmlCaptureBuilder, HtmlCaptureLimits,
        HtmlCaptureRegistryInput, HtmlCapturedIsValue, HtmlDocumentMode, HtmlInputType,
        HtmlScriptingStatus, OrderedAttributes,
    };

    fn capture() -> (HtmlCapture, HtmlCaptureSubjectId) {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(2, 1, 1, 100, 0, 0, 0, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let input = builder
            .add_element(
                &document,
                KnownHtmlElement::Input.qualified_name(),
                OrderedAttributes::new([Attribute::new(
                    KnownHtmlAttribute::Type.qualified_name(),
                    "submit",
                )])
                .unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let capture = builder.finish().unwrap();
        let subject = capture.subject_id(&input).unwrap();
        (capture, subject)
    }

    fn session<'a>(
        capture: &'a HtmlCapture,
        context: &'a HtmlSemanticContext,
        work: usize,
        cancellation: HtmlSemanticCancellation,
    ) -> HtmlSemanticSession<'a> {
        HtmlSemanticSession::new(
            capture,
            context,
            HtmlValueInterpreter::new(6),
            HtmlSemanticLimits::new(work, 0, 0, 0, 0, 0),
            cancellation,
        )
        .unwrap()
    }

    #[test]
    fn submit_report_admits_exact_twenty_two_keyword_envelope_and_rejects_one_short() {
        let (capture, subject) = capture();
        let context = HtmlSemanticContext::new(
            &capture,
            capture.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]),
            0,
        )
        .unwrap();
        let node = capture.node(&subject).unwrap();
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            panic!("expected captured input");
        };
        // The retained wrapper envelope reserves three element-vocabulary
        // passes over input's six positions, five attribute-name positions,
        // three namespace passes over 29 positions, and six value units for
        // each of HTML's 22 authored type keywords. Finding type adds one
        // attribute visit and its four local-name comparisons.
        let work =
            18 * KnownHtmlElement::ALL.len() + 5 * KnownHtmlAttribute::ALL.len() + 87 + 6 * 22 + 5;
        let mut exact = session(&capture, &context, work, HtmlSemanticCancellation::new());
        let report = exact.control_input_type_report(element).unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlInputType::Submit));
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report.recognition(),
            HtmlAttributeRecognition::Keyword {
                matched: "submit",
                canonical: "submit",
            }
        );
        assert_eq!(
            report.raw().unwrap().code_units(),
            [115, 117, 98, 109, 105, 116]
        );
        assert_eq!(exact.usage().work(), work);

        let mut short = session(
            &capture,
            &context,
            work - 1,
            HtmlSemanticCancellation::new(),
        );
        let failure = short.control_input_type_report(element).unwrap_err();
        assert_eq!(
            failure.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: work - 1,
                required: work,
            }
        );
        assert_eq!(short.query(&[]).unwrap_err(), failure);

        let cancellation = HtmlSemanticCancellation::new();
        let mut cancelled = session(&capture, &context, work, cancellation.clone());
        cancellation.cancel();
        let failure = cancelled.control_input_type_report(element).unwrap_err();
        assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
        assert_eq!(cancelled.usage().work(), 0);
        assert_eq!(cancelled.query(&[]).unwrap_err(), failure);
        assert_eq!(report.intrinsic(), Some(&HtmlInputType::Submit));
    }
}

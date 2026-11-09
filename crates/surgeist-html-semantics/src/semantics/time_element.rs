//! Captured time datetime selection and source-ordered exact interpretation.
//! HTML owns the temporal syntax and numeric primitives; this consumer owns
//! admission through the existing semantic session. No host zone is inferred.
//! Recognition is bounded by admitted UTF-16 length; numeric operations are
//! bounded by admitted digit length. The session polls between these atomic
//! operations, not during each instruction or at the parser's fixed quantum.
//! Observed variable-buffer capacity is cumulative, including dropped buffers
//! and later failures. Shared immutable input backing and fixed wrappers are
//! outside that accounting.

use super::*;
use surgeist_html::accounting::{HtmlChange, HtmlResource, HtmlWork};
use surgeist_html::bounded::{
    numeric::{HtmlIntegerUnitStepError, HtmlNumericMaterializationError},
    temporal::{HtmlTimeElementInterpretation, HtmlTimeElementInterpretationError},
};
use surgeist_html::microsyntax::temporal::{HtmlMachineReadableTime, HtmlTimeElementValue};
use surgeist_html::{
    values::{Attribute, DocumentString, HtmlValueReport},
    vocabulary::KnownHtmlElement,
};
use surgeist_ledger::{Change, Charge};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTimeElementValueOrigin {
    DatetimeAttribute,
    ChildText,
}

/// The independent author restriction on absent-datetime element descendants.
/// This does not certify that the datetime value conveys human-readable meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTimeElementDescendantAuthorState {
    NotRequired,
    Conforming,
    Nonconforming(HtmlCaptureSubjectId),
}

/// Actual datetime selection, lexical attempts and descendant author observation.
/// Borrowed accessors retain exact raw spelling and selecting capture evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlTimeElementReadback {
    // Exactly one fallibly admitted record, with no successful empty/multi value.
    data: Vec<HtmlTimeElementReadbackData>,
}
#[derive(Clone, Debug, PartialEq)]
struct HtmlTimeElementReadbackData {
    report: HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>,
    datetime_attribute: Option<Attribute>,
    origin: HtmlTimeElementValueOrigin,
    descendants: HtmlTimeElementDescendantAuthorState,
}
impl HtmlTimeElementReadback {
    pub fn value_report(&self) -> &HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime> {
        &self.data[0].report
    }
    pub fn datetime_attribute(&self) -> Option<&Attribute> {
        self.data[0].datetime_attribute.as_ref()
    }
    pub fn origin(&self) -> HtmlTimeElementValueOrigin {
        self.data[0].origin
    }
    pub fn descendant_author_state(&self) -> &HtmlTimeElementDescendantAuthorState {
        &self.data[0].descendants
    }
}

impl HtmlSemanticSession<'_> {
    pub(super) fn time_element(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTimeElementReadback>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Time) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let datetime_attribute = self.attribute(element, "datetime")?;
        let (raw, origin, descendants) = if let Some(attribute) = datetime_attribute {
            self.charge(1)?;
            (
                attribute.value().clone(),
                HtmlTimeElementValueOrigin::DatetimeAttribute,
                HtmlTimeElementDescendantAuthorState::NotRequired,
            )
        } else {
            let raw =
                match self.control_text_or_missing(subject, HtmlControlTextOrigin::ChildText)? {
                    Ok(raw) => raw,
                    Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                };
            let node = self.node(subject)?;
            self.charge(1)?;
            // The text selection above read these children as complete.
            let HtmlCapturedChildren::Complete(mut children) = node.children() else {
                return Err(self.stop(HtmlSemanticFailureKind::ProtocolViolated));
            };
            let mut descendants = HtmlTimeElementDescendantAuthorState::Conforming;
            loop {
                self.charge(1)?;
                let Some(child) = children.next() else {
                    break;
                };
                if matches!(self.node(&child)?.kind(), HtmlCapturedNodeKind::Element(_)) {
                    // Checked capture edges forbid Document/Fragment children;
                    // permitted non-elements cannot contain ordinary children.
                    descendants = HtmlTimeElementDescendantAuthorState::Nonconforming(child);
                    break;
                }
            }
            (raw, HtmlTimeElementValueOrigin::ChildText, descendants)
        };
        self.charge(1)?;
        let report = self.interpret_time_element(&raw)?;
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(HtmlTimeElementReadbackData {
            report,
            datetime_attribute: datetime_attribute.cloned(),
            origin,
            descendants,
        });
        Ok(HtmlSemanticValue::Known(HtmlTimeElementReadback { data }))
    }

    fn interpret_time_element(
        &mut self,
        raw: &DocumentString,
    ) -> Result<HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>, HtmlSemanticFailure>
    {
        let mut owner = HtmlTimeElementInterpretation::new(raw, self.interpreter)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        loop {
            let action = owner.next_action().map_err(|error| {
                self.time_interpretation_error(error, self.usage.allocated_bytes)
            })?;
            let Some(action) = action else {
                break;
            };
            let baseline = self.usage.allocated_bytes;
            let (digits, remaining) =
                (self.limits.integer_digits, self.remaining_allocated_bytes());
            // The action charges its own semantic work before it runs. Fixed continuations after a
            // numeric poll belong to the already admitted operation and charge nothing.
            let step = action.advance(&mut StepCharge(self), digits, remaining)?;
            // Every numeric attempt's actual capacity is observed exactly once,
            // including allocations preceding a later failure in that attempt.
            self.record_allocated_bytes(step.capacity_bytes())?;
            step.outcome()
                .map_err(|error| self.time_interpretation_error(error, baseline))?;
            if step.poll_after() {
                self.poll()?;
            }
        }
        owner.finish().map_err(|failure| {
            self.time_interpretation_error(*failure.cause(), self.usage.allocated_bytes)
        })
    }

    fn time_interpretation_error(
        &mut self,
        error: HtmlTimeElementInterpretationError,
        baseline: usize,
    ) -> HtmlSemanticFailure {
        let kind = if error.is_representation_exhausted() {
            HtmlSemanticFailureKind::RepresentationExhausted
        } else if let Some(error) = error.processing_failure() {
            match error {
                HtmlNumericMaterializationError::Digits(_) => {
                    HtmlSemanticFailureKind::IntrinsicUnavailable
                }
                HtmlNumericMaterializationError::Boundary(error) => match error {
                    HtmlIntegerUnitStepError::RepresentationExhausted => {
                        HtmlSemanticFailureKind::RepresentationExhausted
                    }
                    HtmlIntegerUnitStepError::AllocationFailed { .. } => {
                        HtmlSemanticFailureKind::AllocationFailed
                    }
                    HtmlIntegerUnitStepError::DigitLimitExceeded { limit, required } => {
                        HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::IntegerDigits,
                            limit,
                            required,
                        }
                    }
                    HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                        required, ..
                    } => match baseline.checked_add(required) {
                        Some(required) => HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::AllocatedBytes,
                            limit: self.limits.allocated_bytes,
                            required,
                        },
                        None => HtmlSemanticFailureKind::RepresentationExhausted,
                    },
                    HtmlIntegerUnitStepError::ActualCapacityLimitExceeded { .. } => {
                        HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::AllocatedBytes,
                            limit: self.limits.allocated_bytes,
                            required: self.usage.allocated_bytes,
                        }
                    }
                },
            }
        } else {
            HtmlSemanticFailureKind::IntrinsicUnavailable
        };
        self.stop(kind)
    }
}

/// Charges the semantic steps of `surgeist-html` operations to the session's work.
struct StepCharge<'s, 'a>(&'s mut HtmlSemanticSession<'a>);

impl Charge<HtmlWork, HtmlResource> for StepCharge<'_, '_> {
    type Stop = HtmlSemanticFailure;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), HtmlSemanticFailure> {
        let mut steps = 0_usize;
        for change in batch {
            let Change::Work(HtmlWork::SemanticStep, amount) = *change else {
                return Err(self.0.stop(HtmlSemanticFailureKind::ProtocolViolated));
            };
            steps = usize::try_from(amount)
                .ok()
                .and_then(|amount| steps.checked_add(amount))
                .ok_or_else(|| {
                    self.0
                        .stop(HtmlSemanticFailureKind::RepresentationExhausted)
                })?;
        }
        self.0.charge(steps)
    }

    fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), HtmlSemanticFailure> {
        Err(self.0.stop(HtmlSemanticFailureKind::ProtocolViolated))
    }
}

#[cfg(test)]
mod schema_tests {
    use super::super::*;
    use crate::{
        HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlScriptingStatus,
    };
    use surgeist_html::{
        accounting::{HtmlLimits, HtmlWork},
        bounded::temporal::HtmlTimeElementInterpretation,
        construction::HtmlDocumentMode,
    };
    use surgeist_ledger::Operation;

    /// The session charges each interpretation action's semantic work. The same actions run through
    /// the stock ledger charger record the same total.
    #[test]
    fn time_interpretation_layouts_convert_to_the_work_the_session_charges() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(1, 0, 0, 0, 0, 0, 0, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let capture = builder.finish().unwrap();
        let context = HtmlSemanticContext::new(
            &capture,
            capture.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]),
            0,
        )
        .unwrap();
        for text in [
            "2011-11-18T14:54:39.929Z",
            "2011-11-18",
            "2011-W47",
            "PT4H18M3S",
            "3d 4h",
            "2011-11-18 25:00",
            "bad",
        ] {
            let raw = DocumentString::from(text);
            let mut session = HtmlSemanticSession::new(
                &capture,
                &context,
                HtmlValueInterpreter::new(4096),
                HtmlSemanticLimits::new(1 << 40, 0, 0, 64, 1 << 30, 0),
                HtmlSemanticCancellation::new(),
            )
            .unwrap();
            let _ = session.interpret_time_element(&raw);
            let mut owner =
                HtmlTimeElementInterpretation::new(&raw, HtmlValueInterpreter::new(4096)).unwrap();
            let mut ledger = Operation::new("steps", HtmlLimits::unlimited(), 1).unwrap();
            let root = ledger.root();
            while let Ok(Some(action)) = owner.next_action() {
                let step = action
                    .advance(&mut ledger.charge_at(root), 64, 1 << 30)
                    .unwrap();
                if step.outcome().is_err() {
                    break;
                }
            }
            let expected = ledger.usage(root).unwrap().work(HtmlWork::SemanticStep);
            assert_eq!(session.usage().work() as u64, expected, "{text:?}");
            assert_ne!(expected, 0, "{text:?}");
        }
    }
}

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
use crate::bounded::{
    HtmlTimeElementInterpretation, HtmlTimeElementInterpretationActionKind,
    HtmlTimeElementInterpretationError, IntegerUnitStepError, NumericMaterializationError,
};
use crate::{HtmlMachineReadableTime, HtmlTimeElementValue, HtmlValueReport};

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
            let raw = match self.control_text_content(subject, HtmlControlTextOrigin::ChildText)? {
                HtmlSemanticValue::Known(raw) => raw,
                HtmlSemanticValue::Missing(missing) => {
                    return Ok(HtmlSemanticValue::Missing(missing));
                }
                HtmlSemanticValue::Inapplicable => return Ok(HtmlSemanticValue::Inapplicable),
                HtmlSemanticValue::Unsupported(reason) => {
                    return Ok(HtmlSemanticValue::Unsupported(reason));
                }
            };
            let node = self.node(subject)?;
            self.charge(1)?;
            let HtmlCapturedChildren::Complete(mut children) = node.children() else {
                unreachable!("the same immutable child sequence was complete for text selection")
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
        let mut owner = Some(
            HtmlTimeElementInterpretation::new(raw, self.interpreter)
                .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?,
        );
        loop {
            let action = owner
                .as_mut()
                .expect("retained time interpretation")
                .next_action()
                .map_err(|error| {
                    self.time_interpretation_error(error, self.usage.allocated_bytes)
                })?;
            let Some(action) = action else {
                break;
            };
            let layout = action.layout();
            // Fixed continuations after a numeric poll belong to the already
            // admitted operation; the original path added no fee or poll here.
            if layout.work() != 0
                || layout.kind() != HtmlTimeElementInterpretationActionKind::Assembly
            {
                self.charge(layout.work())?;
            }
            let baseline = self.usage.allocated_bytes;
            let step = action.advance(self.limits.integer_digits, self.remaining_allocated_bytes());
            // Every numeric attempt's actual capacity is observed exactly once,
            // including allocations preceding a later failure in that attempt.
            self.record_allocated_bytes(step.capacity_bytes())?;
            step.outcome()
                .map_err(|error| self.time_interpretation_error(error, baseline))?;
            if step.poll_after() {
                self.poll()?;
            }
        }
        HtmlTimeElementInterpretation::try_finish(&mut owner)
            .map_err(|error| self.time_interpretation_error(error, self.usage.allocated_bytes))
    }

    fn time_interpretation_error(
        &mut self,
        error: HtmlTimeElementInterpretationError,
        baseline: usize,
    ) -> HtmlSemanticFailure {
        let kind = match error {
            HtmlTimeElementInterpretationError::RepresentationExhausted => {
                HtmlSemanticFailureKind::RepresentationExhausted
            }
            HtmlTimeElementInterpretationError::Processing(error) => match error {
                NumericMaterializationError::Digits(_) => {
                    HtmlSemanticFailureKind::IntrinsicUnavailable
                }
                NumericMaterializationError::Boundary(error) => match error {
                    IntegerUnitStepError::RepresentationExhausted => {
                        HtmlSemanticFailureKind::RepresentationExhausted
                    }
                    IntegerUnitStepError::AllocationFailed { .. } => {
                        HtmlSemanticFailureKind::AllocationFailed
                    }
                    IntegerUnitStepError::DigitLimitExceeded { limit, required } => {
                        HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::IntegerDigits,
                            limit,
                            required,
                        }
                    }
                    IntegerUnitStepError::RequestedCapacityLimitExceeded { required, .. } => {
                        match baseline.checked_add(required) {
                            Some(required) => HtmlSemanticFailureKind::LimitExceeded {
                                resource: HtmlSemanticResource::AllocatedBytes,
                                limit: self.limits.allocated_bytes,
                                required,
                            },
                            None => HtmlSemanticFailureKind::RepresentationExhausted,
                        }
                    }
                    IntegerUnitStepError::ActualCapacityLimitExceeded { .. } => {
                        HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::AllocatedBytes,
                            limit: self.limits.allocated_bytes,
                            required: self.usage.allocated_bytes,
                        }
                    }
                },
            },
            _ => HtmlSemanticFailureKind::IntrinsicUnavailable,
        };
        self.stop(kind)
    }
}

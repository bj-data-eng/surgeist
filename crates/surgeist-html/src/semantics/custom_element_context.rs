//! Independent author obligations over actual immutable custom-element slots.
//! HTML 877d6146487f #attr-is, #autonomous-custom-element and definition lookup.
//! No registry execution, upgrade, inference from hyphens, or content-to-saved
//! slot conversion occurs. Inline output retains only shared captured storage;
//! the session admits the actual observation layout and lookup actions. Atomic
//! local validation uses its checked bounded envelope, not a parser quantum or
//! hardware-preemption/RSS guarantee.

use super::*;
use crate::capture::DefinitionLookupStep;
use crate::{
    HtmlAttributeApplicability, HtmlAttributeReport, HtmlCaptureDefinitionId,
    HtmlCaptureRegistryId, HtmlCapturedCustomElementDefinition, HtmlCapturedCustomElementState,
    HtmlCustomElementName, HtmlCustomElementState,
};

/// An influencing unavailable fact, retained independently per obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIsContextUnavailable {
    Registry(HtmlCaptureSubjectId),
    Definitions(HtmlCaptureRegistryId),
    CustomElementState(HtmlCaptureSubjectId),
    AssociatedDefinition(HtmlCaptureSubjectId),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIsDefinitionRequirement {
    NotRequired,
    BlockedByLocalSyntax,
    Matched(HtmlCaptureDefinitionId),
    NoMatchingCustomizedDefinition {
        selected: Option<HtmlCaptureDefinitionId>,
    },
    Unavailable(HtmlIsContextUnavailable),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIsAutonomousRequirement {
    NotRequired,
    Allowed,
    Forbidden(HtmlCaptureDefinitionId),
    Unavailable(HtmlIsContextUnavailable),
}
/// A complete observation, including independently unavailable obligations.
/// `Known` does not mean author-conforming; no overall conformance bool is supplied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCustomElementIsAuthorContext {
    local: HtmlAttributeReport<HtmlCustomElementName>,
    definition_requirement: HtmlIsDefinitionRequirement,
    autonomous_requirement: HtmlIsAutonomousRequirement,
}
impl HtmlCustomElementIsAuthorContext {
    pub fn local(&self) -> &HtmlAttributeReport<HtmlCustomElementName> {
        &self.local
    }
    pub fn definition_requirement(&self) -> &HtmlIsDefinitionRequirement {
        &self.definition_requirement
    }
    pub fn autonomous_requirement(&self) -> &HtmlIsAutonomousRequirement {
        &self.autonomous_requirement
    }
    pub fn source_clause(&self) -> &'static str {
        HtmlSemanticQuery::IsAttributeAuthorContext.source_clause()
    }
}
impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn is_attribute_author_context(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlCustomElementIsAuthorContext>, HtmlSemanticFailure> {
        let raw = self.attribute(element, "is")?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        let bound = HtmlValueInterpreter::is_attribute_report_work_bound(element.name(), raw)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(bound)?;
        let local = self
            .interpreter
            .is_attribute(element.name(), raw)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        self.poll()?;
        if local.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let Some(raw) = raw else {
            self.charge(1)?;
            return Ok(HtmlSemanticValue::Known(HtmlCustomElementIsAuthorContext {
                local,
                definition_requirement: HtmlIsDefinitionRequirement::NotRequired,
                autonomous_requirement: HtmlIsAutonomousRequirement::NotRequired,
            }));
        };
        let definition_requirement = if local.intrinsic().is_none() {
            HtmlIsDefinitionRequirement::BlockedByLocalSyntax
        } else {
            self.charge(2)?;
            let registry = self
                .capture
                .node(subject)
                .map_err(|error| self.stop(HtmlSemanticFailureKind::Capture(error)))?
                .lookup_registry();
            match self.capture.definition_lookup_cursor(
                &registry,
                element.name().local_name(),
                Some(raw.value()),
            ) {
                Err(HtmlCaptureError::MissingFact { .. }) => {
                    HtmlIsDefinitionRequirement::Unavailable(HtmlIsContextUnavailable::Registry(
                        subject.clone(),
                    ))
                }
                Err(HtmlCaptureError::UnavailableDefinitions(id)) => {
                    HtmlIsDefinitionRequirement::Unavailable(HtmlIsContextUnavailable::Definitions(
                        id,
                    ))
                }
                Err(error) => return Err(self.stop(HtmlSemanticFailureKind::Capture(error))),
                Ok(mut cursor) => {
                    let selected = loop {
                        self.charge(cursor.next_cost())?;
                        if let DefinitionLookupStep::Ready(selected) = cursor.step() {
                            break selected;
                        }
                    };
                    if let Some(id) = &selected {
                        self.charge(1)?;
                        let d = self
                            .capture
                            .definition(id)
                            .map_err(|error| self.stop(HtmlSemanticFailureKind::Capture(error)))?;
                        if self.control_equal(d.name(), raw.value())?
                            && self.control_equal(d.local_name(), element.name().local_name())?
                            && !self.control_equal(d.name(), d.local_name())?
                        {
                            HtmlIsDefinitionRequirement::Matched(
                                selected.expect("checked selected identity"),
                            )
                        } else {
                            HtmlIsDefinitionRequirement::NoMatchingCustomizedDefinition { selected }
                        }
                    } else {
                        HtmlIsDefinitionRequirement::NoMatchingCustomizedDefinition { selected }
                    }
                }
            }
        };
        self.charge(1)?;
        let autonomous_requirement = match element.custom_element_state() {
            HtmlCapturedCustomElementState::Unavailable => {
                HtmlIsAutonomousRequirement::Unavailable(
                    HtmlIsContextUnavailable::CustomElementState(subject.clone()),
                )
            }
            HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom) => {
                self.charge(1)?;
                match element.custom_element_definition() {
                    HtmlCapturedCustomElementDefinition::Unavailable => {
                        HtmlIsAutonomousRequirement::Unavailable(
                            HtmlIsContextUnavailable::AssociatedDefinition(subject.clone()),
                        )
                    }
                    HtmlCapturedCustomElementDefinition::Definition(id) => {
                        self.charge(1)?;
                        let d = self
                            .capture
                            .definition(id)
                            .map_err(|error| self.stop(HtmlSemanticFailureKind::Capture(error)))?;
                        if self.control_equal(d.name(), d.local_name())? {
                            self.charge(1)?;
                            HtmlIsAutonomousRequirement::Forbidden(id.clone())
                        } else {
                            HtmlIsAutonomousRequirement::Allowed
                        }
                    }
                    HtmlCapturedCustomElementDefinition::Null => {
                        unreachable!("capture rejects Custom with known null")
                    }
                }
            }
            HtmlCapturedCustomElementState::Known(_) => HtmlIsAutonomousRequirement::Allowed,
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlCustomElementIsAuthorContext {
            local,
            definition_requirement,
            autonomous_requirement,
        }))
    }
}

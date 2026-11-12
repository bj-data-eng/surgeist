//! The original standard setter's registered verification and retained reply.
use super::*;
use crate::hosted::ScriptRegistration;
use crate::store::attributes::PreparedAttributeName;
use crate::{DomDocumentScriptRegistration, DomHostedError};

pub(in crate::store) struct Pending {
    request: DomSourceAttributeVerificationRequest,
    name: PreparedAttributeName,
    outcome: Option<RetainedOutcome>,
}

struct RetainedOutcome {
    outcome: DomSourceAttributeVerificationOutcome,
    _backing: Option<formation::Formation>,
}

impl Pending {
    pub(super) fn has_outcome(&self) -> bool {
        self.outcome.is_some()
    }

    pub(in crate::store) fn apply(
        &self,
        draft: &mut Draft,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use DomSourceAttributeVerificationOutcome::{Abrupt, Rejected, Value};
        let retained = self
            .outcome
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let error = match &retained.outcome {
            Value(value) => {
                let element = draft.resolve(self.request.element.into())?;
                return draft
                    .apply_verified_attribute_name(element, &self.name, value, limits, cancel);
            }
            Rejected => DomError::SourceAttributeVerificationRejected,
            Abrupt => DomError::SourceAttributeVerificationAbrupt,
        };
        draft.source_set_method(
            Some(crate::store::method::Frame::CompoundFailure(error)),
            cancel,
        )
    }
}

impl Operation {
    pub(super) fn waiting_attribute_verification(
        &self,
    ) -> Option<DomSourceAttributeVerificationRequest> {
        self.attribute_verification
            .as_ref()
            .filter(|pending| !pending.has_outcome())
            .map(|pending| pending.request)
    }

    pub(super) fn apply_reached_command(
        &mut self,
        dom: &Dom,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(Command::SetAttribute {
            node,
            namespace,
            name,
            value,
            mode: AttributeSetterMode::Standard,
        }) = self.commands.get(self.index)
        else {
            return self.apply_command(dom.limits, cancel);
        };
        self.draft.source_step(cancel)?;
        if let Some(pending) = &self.attribute_verification {
            return pending.apply(&mut self.draft, dom.limits, cancel);
        }
        let element = self.draft.resolve(node.clone())?;
        let (name, sink) = self.draft.prepare_standard_attribute_name(
            element,
            namespace.as_ref(),
            name,
            cancel,
        )?;
        let Some(sink) = sink else {
            return self
                .draft
                .apply_attribute_name(element, name, value, dom.limits, cancel);
        };
        let document = self.draft.node(element)?.document();
        let global = self
            .draft
            .node(document)?
            .relevant_global()
            .ok_or(DomError::SourceAttributeGlobalUnavailable)?;
        let registration = self
            .draft
            .script_registration
            .as_ref()
            .ok_or(DomError::SourceAttributeExecutorUnavailable)?;
        self.qualify_attribute_registration(dom, global, registration, cancel)?;
        let element = self
            .draft
            .published_id(element)
            .ok_or(DomError::InvalidSourceOperation)?;
        self.attribute_verification = Some(Pending {
            request: DomSourceAttributeVerificationRequest {
                operation: self.id,
                element,
                global,
                sink,
            },
            name,
            outcome: None,
        });
        Ok(())
    }

    fn qualify_attribute_registration(
        &self,
        dom: &Dom,
        global: crate::DomHostedGlobalId,
        registration: &Arc<ScriptRegistration>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if registration.global != global
            || dom.active_script_registration() != Some(registration.key)
            || !self
                .draft
                .script_registration
                .as_ref()
                .is_some_and(|original| Arc::ptr_eq(original, registration))
        {
            return Err(DomError::SourceAttributeExecutorUnavailable);
        }
        match dom.dispatch_script_registration_current_with(registration, || {
            self.draft.source_step(cancel).map_err(DomHostedError::from)
        }) {
            Ok(true) => Ok(()),
            Err(DomHostedError::Dom(error)) => Err(error),
            Err(DomHostedError::SourceExpired | DomHostedError::SourcePoisoned) => {
                Err(DomError::HostedSourceUnavailable)
            }
            Ok(false) | Err(_) => Err(DomError::SourceAttributeExecutorUnavailable),
        }
    }
}

impl DomSourceLease<'_> {
    fn qualify_attribute_verification(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<&Operation, DomError> {
        check_cancel(cancel)?;
        let operation = self
            .dom
            .source_operation
            .as_deref()
            .filter(|operation| {
                operation.id == self.id
                    && operation.waiting_attribute_verification() == Some(request)
            })
            .ok_or(DomError::InvalidSourceOperation)?;
        if operation.cancelled {
            return Err(DomError::Cancelled);
        }
        operation.qualify_attribute_registration(
            self.dom,
            request.global,
            &registration.0,
            cancel,
        )?;
        Ok(operation)
    }

    /// Read the original, exact UTF-16 input for this registered verifier.
    pub fn attribute_verification_input(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<&DocumentString, DomError> {
        let operation = self.qualify_attribute_verification(request, registration, cancel)?;
        let Some(Command::SetAttribute { value, .. }) = operation.commands.get(operation.index)
        else {
            return Err(DomError::InvalidSourceOperation);
        };
        Ok(value)
    }

    /// Admit the registered verifier's reply once. On refusal the caller still
    /// owns the reply, including any rooted abrupt outcome it represents.
    pub fn accept_attribute_verification(
        &mut self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        outcome: &mut Option<DomSourceAttributeVerificationOutcome>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let operation = self.qualify_attribute_verification(request, registration, cancel)?;
        let candidate = outcome.as_ref().ok_or(DomError::InvalidSourceOperation)?;
        let backing = if let DomSourceAttributeVerificationOutcome::Value(value) = candidate {
            let formation = formation::Formation::new(&operation.draft, cancel)?;
            if let Some(formation) = &formation {
                let mut payload = storage::Payload::default();
                payload.string(value)?;
                formation.retain_payload(payload, cancel)?;
            }
            formation
        } else {
            None
        };
        let pending = self
            .dom
            .source_operation
            .as_mut()
            .and_then(|operation| operation.attribute_verification.as_mut())
            .ok_or(DomError::InvalidSourceOperation)?;
        let outcome = outcome.take().ok_or(DomError::InvalidSourceOperation)?;
        pending.outcome = Some(RetainedOutcome {
            outcome,
            _backing: backing,
        });
        Ok(())
    }
}

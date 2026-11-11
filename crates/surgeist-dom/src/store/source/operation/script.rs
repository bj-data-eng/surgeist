//! The original private Script effect waits for a read-only host observation.

use super::*;
use crate::store::script::{Boundary, Frame};
use crate::store::select::Effect;
use crate::{DomDocumentScriptBinding, DomDocumentScriptingFacts, DomHostedError, DomHostedSource};
use surgeist_html::bounded::script::{HtmlScriptPreparationError, HtmlScriptPreparationReply};

impl Operation {
    fn script_request(
        &self,
        frame: Frame,
        ordinal: u64,
    ) -> Result<DomSourceScriptRequest, DomError> {
        let node = self.draft.node(frame.node)?;
        let step = match frame.boundary {
            Some(Boundary::Settings(_)) => DomSourceScriptStep::ReadScripting,
            Some(Boundary::Executable(_)) => DomSourceScriptStep::ExecutablePreparation,
            None => return Err(DomError::InvalidSourceOperation),
        };
        Ok(DomSourceScriptRequest {
            operation: self.id,
            occurrence: crate::source::watcher::HookOccurrence {
                command: self.index,
                ordinal,
            },
            basis: self.draft.basis,
            subject: self.draft.published_id(frame.node),
            node_document: self
                .draft
                .published_id(node.document())
                .map(crate::DocumentId),
            relevant_global: node.relevant_global(),
            step,
        })
    }

    pub(super) fn issue_script_request(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<DomSourceScriptRequest, DomError> {
        self.draft.source_step(cancel)?;
        let frame = self
            .draft
            .pending_script()
            .ok_or(DomError::InvalidSourceOperation)?;
        if let Some(Boundary::Settings(Some(ordinal)) | Boundary::Executable(ordinal)) =
            frame.boundary
        {
            return self.script_request(frame, ordinal);
        }
        let ordinal = self.draft.source_invocation;
        let next = ordinal
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let identities = self
            .identities
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if identities > self.limits.identities {
            return Err(DomError::SourceIdentityLimit);
        }
        let request = self.script_request(frame, ordinal)?;
        let Some(Effect::Script(frame)) = self
            .draft
            .effects
            .last_mut()
            .map(|effect| &mut effect.effect)
        else {
            return Err(DomError::InvalidSourceOperation);
        };
        frame.boundary = Some(Boundary::Settings(Some(ordinal)));
        self.draft.source_invocation = next;
        self.identities = identities;
        Ok(request)
    }
}

impl Operation {
    fn require_script_request(&self, request: DomSourceScriptRequest) -> Result<Frame, DomError> {
        if self.id != request.operation() {
            return Err(DomError::InvalidSourceOperation);
        }
        let dialog::OperationDraft::Segment(draft) = &self.draft else {
            return Err(DomError::InvalidSourceOperation);
        };
        let frame = draft
            .pending_script()
            .ok_or(DomError::InvalidSourceOperation)?;
        let Some(Boundary::Settings(Some(ordinal)) | Boundary::Executable(ordinal)) =
            frame.boundary
        else {
            return Err(DomError::InvalidSourceOperation);
        };
        if self.script_request(frame, ordinal)? != request {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok(frame)
    }
}

impl DomSourceLease<'_> {
    /// Charge the exact private Script request's original Source work before
    /// consulting the trusted settings producer. No mutation or reentry is lent.
    pub fn source_script_step(
        &self,
        request: DomSourceScriptRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        operation.require_script_request(request)?;
        if operation.cancelled && !operation.published && !operation.event_exposed {
            return Err(DomError::Cancelled);
        }
        if self.dom.snapshot().stamp() != request.basis()
            || request.step() != DomSourceScriptStep::ReadScripting
        {
            return Err(DomError::InvalidSourceOperation);
        }
        operation.draft.source_step(cancel)
    }

    /// Read live settings for the Script's original relevant Global. The
    /// current node Document's browsing context was already qualified by DOM
    /// at the reached Script action. This is a read of the original owner.
    pub fn observe_script_settings(
        &self,
        request: DomSourceScriptRequest,
        source: &DomHostedSource,
        binding: &DomDocumentScriptBinding,
        facts: DomDocumentScriptingFacts,
        cancel: &AtomicBool,
    ) -> Result<DomSourceScriptSettings, DomHostedError> {
        if request.operation() != self.id {
            return Err(DomError::InvalidSourceOperation.into());
        }
        self.source_script_step(request, cancel)?;
        let global = request
            .relevant_global()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let enabled = source.script_window_settings(self.dom, binding, global, facts, || {
            operation
                .draft
                .source_step(cancel)
                .map_err(DomHostedError::from)
        })?;
        Ok(DomSourceScriptSettings { request, enabled })
    }

    /// Apply this exact live settings observation once. An enabled answer
    /// parks the next executable-preparation requirement on the same frame.
    pub fn accept_script_settings(
        &mut self,
        observation: DomSourceScriptSettings,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let request = observation.request();
        if request.operation() != self.id || self.dom.snapshot().stamp() != request.basis() {
            return Err(DomError::InvalidSourceOperation);
        }
        let operation = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        let mut frame = operation.require_script_request(request)?;
        if operation.cancelled && !operation.published && !operation.event_exposed {
            return Err(DomError::Cancelled);
        }
        let Some(Boundary::Settings(Some(ordinal))) = frame.boundary else {
            return Err(DomError::InvalidSourceOperation);
        };
        operation.draft.source_step(cancel)?;
        let reply = if observation.enabled() {
            HtmlScriptPreparationReply::Enabled
        } else {
            HtmlScriptPreparationReply::Disabled
        };
        frame.boundary = match frame.cursor.reply(reply) {
            Ok(()) => None,
            Err(HtmlScriptPreparationError::ScriptingEnabled) => {
                Some(Boundary::Executable(ordinal))
            }
            Err(_) => return Err(DomError::InvalidSourceOperation),
        };
        let Some(Effect::Script(pending)) = operation
            .draft
            .effects
            .last_mut()
            .map(|effect| &mut effect.effect)
        else {
            return Err(DomError::InvalidSourceOperation);
        };
        *pending = frame;
        Ok(())
    }

    /// Request explicit cancellation of the original Script preparation. A
    /// published Source still owns its mandatory continuation.
    pub fn cancel_script_preparation(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        if !matches!(&operation.draft, dialog::OperationDraft::Segment(draft)
            if draft.pending_script().is_some())
        {
            return Err(DomError::InvalidSourceOperation);
        }
        check_cancel(cancel)?;
        self.request_cancel();
        Ok(())
    }
}

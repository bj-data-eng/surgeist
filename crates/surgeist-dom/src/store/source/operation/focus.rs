//! Remembered absence joins the original Show cursor, receipt and checkpoint.
use super::*;
use crate::{
    DomHostedSource, DomRememberedFocusPreparation, DomRememberedFocusPublication,
    DomSourceExecutionEntryScope,
};
use surgeist_html::{HtmlDialogAction, HtmlDialogReply};
impl Operation {
    fn remembered_frame(
        &self,
        request: &DomSourceDialogRequest,
    ) -> Result<crate::store::dialog::ShowFrame, DomError> {
        let occurrence = request.show().ok_or(DomError::InvalidSourceOperation)?;
        let frame = self
            .draft
            .dialog_show
            .ok_or(DomError::InvalidSourceOperation)?;
        if request.operation != self.id
            || request.action != HtmlDialogAction::RememberFocusedElement
            || occurrence.command != self.index
            || !matches!(frame.invocation, Invocation::Exposed(actual) if actual == occurrence.ordinal)
            || frame.cursor.action() != request.action
            || self.draft.published_id(frame.subject) != Some(request.subject)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        self.draft.dialog_state(frame.subject)?;
        Ok(frame)
    }
    pub(super) fn apply_remembered_focus(
        &mut self,
        request: &DomSourceDialogRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut frame = self.remembered_frame(request)?;
        self.draft.source_step(cancel)?;
        // Actual common-state write; an already-null value stays a semantic
        // no-op and cannot manufacture a DOM change or revision.
        self.draft
            .element_mut(frame.subject)
            .remember_no_focused_element();
        self.draft.source_step(cancel)?;
        frame
            .cursor
            .reply(HtmlDialogReply::Applied)
            .map_err(DomError::DialogProtocol)?;
        self.draft.dialog_show = Some(frame);
        Ok(())
    }
}
impl DomSourceLease<'_> {
    /// Prepare only the actual original remembered-focus occurrence. Missing
    /// original Host scope remains unavailable, never evidence of absence.
    pub fn prepare_remembered_focus(
        &self,
        request: &DomSourceDialogRequest,
        source: &DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<Option<DomRememberedFocusPreparation>, DomError> {
        self.dom
            .validate_hosted_source(source)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        let op = self
            .dom
            .source_operation
            .as_ref()
            .filter(|op| op.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        op.remembered_frame(request)?;
        if op.receipt.is_some() {
            return Err(DomError::InvalidSourceOperation);
        }
        op.draft.source_step(cancel)?;
        let scope = op.entry_scope.observation();
        let host = match scope {
            DomSourceExecutionEntryScope::ExplicitHost(host)
            | DomSourceExecutionEntryScope::Hosted {
                original_host: Some(host),
                ..
            } => host,
            _ => return Ok(None),
        };
        if host.source != source.id() {
            return Err(DomError::HostedSourceUnavailable);
        }
        source
            .observe()
            .validate(
                source.id(),
                crate::hosted::Role::Host,
                host.slot,
                host.generation,
            )
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        if op
            .phases
            .checked_add(1)
            .is_none_or(|n| n > op.limits.phases)
        {
            return Err(DomError::SourcePublicationLimit);
        }
        if op
            .outputs
            .checked_add(1)
            .is_none_or(|n| n > op.limits.outputs)
        {
            return Err(DomError::SourceOutputLimit);
        }
        op.draft.source_admit(0, cancel)?;
        Ok(Some(DomRememberedFocusPreparation {
            request: request.clone(),
            host,
            scope,
            basis: self.dom.snapshot().stamp(),
        }))
    }
    /// Move the exact authentic observation into the original receipt before
    /// subsequent fallible cursor application/publication.
    pub fn accept_remembered_focus(
        &mut self,
        request: &DomSourceDialogRequest,
        source: &DomHostedSource,
        publication: &mut Option<DomRememberedFocusPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.dom
            .validate_hosted_source(source)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        let basis = self.dom.snapshot().stamp();
        let op = self
            .dom
            .source_operation
            .as_ref()
            .filter(|op| op.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        op.remembered_frame(request)?;
        let actual = publication
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        if &actual.request != request
            || actual.basis != basis
            || actual.scope != op.entry_scope.observation()
            || actual.host.source != source.id()
            || op.receipt.is_some()
        {
            return Err(DomError::InvalidSourceOperation);
        }
        // The sealed complete-Host absence is lifetime-qualified. Unrelated
        // issuer progress cannot replace this Host role/slot/generation.
        source
            .observe()
            .validate(
                source.id(),
                crate::hosted::Role::Host,
                actual.host.slot,
                actual.host.generation,
            )
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        // Relative producer formation may have released its real temporary
        // backing before transfer. Refresh the same owner/ancestor projection
        // only after qualifying the exact retained observation.
        self.dom.admit_source_owner_payload(cancel)?;
        let op = self
            .dom
            .source_operation
            .as_mut()
            .expect("qualified original operation");
        op.draft.source_step(cancel)?;
        op.refresh_admission(cancel)?;
        op.receipt = Some(Receipt::RememberedFocus {
            publication: publication.take().expect("qualified authentic observation"),
            applied: false,
        });
        Ok(())
    }
}

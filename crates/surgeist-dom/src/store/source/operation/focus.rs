//! Qualified negative focus observations join the original Show receipt.
use super::*;
use crate::{
    DomDialogFocusPreparation, DomDialogFocusPublication, DomDialogFocusRequest, DomHostedSource,
    DomSourceExecutionEntryScope, source::FocusQualification,
};
use surgeist_html::{HtmlDialogAction, HtmlDialogReply};
impl Operation {
    fn focus_frame(
        &self,
        request: &DomDialogFocusRequest,
    ) -> Result<crate::store::dialog::MethodFrame, DomError> {
        let frame = self
            .draft
            .dialog_method
            .ok_or(DomError::InvalidSourceOperation)?;
        if request.operation() != self.id
            || self.draft.published_id(frame.subject) != Some(request.subject())
        {
            return Err(DomError::InvalidSourceOperation);
        }
        match request {
            DomDialogFocusRequest::Remember(request) => {
                let occurrence = request.method().ok_or(DomError::InvalidSourceOperation)?;
                if request.action() != HtmlDialogAction::RememberFocusedElement
                    || frame.cursor.action() != request.action()
                    || occurrence.command != self.index
                    || !matches!(frame.invocation,Invocation::Exposed(actual) if actual==occurrence.ordinal)
                {
                    return Err(DomError::InvalidSourceOperation);
                }
            }
            DomDialogFocusRequest::Allow(request) => {
                let child = frame.focusing.ok_or(DomError::InvalidSourceOperation)?;
                if request.action() != surgeist_html::HtmlDialogFocusingAction::ReadAllowFocus
                    || frame.cursor.action() != HtmlDialogAction::RunDialogFocusingSteps
                    || child.cursor.action() != request.action()
                    || request.occurrence.command != self.index
                    || !matches!(child.invocation,Invocation::Exposed(actual) if actual==request.occurrence.ordinal)
                {
                    return Err(DomError::InvalidSourceOperation);
                }
            }
        }
        self.draft.dialog_state(frame.subject)?;
        Ok(frame)
    }
    fn focus_qualification(
        &self,
        request: &DomDialogFocusRequest,
        cancel: &AtomicBool,
    ) -> Result<Option<FocusQualification>, DomError> {
        let frame = self.focus_frame(request)?;
        if matches!(request, DomDialogFocusRequest::Remember(_)) {
            let scope = self.entry_scope.observation();
            return Ok(match scope {
                DomSourceExecutionEntryScope::ExplicitHost(host)
                | DomSourceExecutionEntryScope::Hosted {
                    original_host: Some(host),
                    ..
                } => Some(FocusQualification::Remembered { host, scope }),
                _ => None,
            });
        }
        Ok(
            match self
                .draft
                .dialog_allow_focus_permission(frame.subject, cancel)?
            {
                crate::store::dialog::AllowFocusPermission::NeedsWindow(qualification) => {
                    Some(qualification)
                }
                _ => None,
            },
        )
    }
    fn validate_focus_producer(
        &self,
        qualification: FocusQualification,
        source: &DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let host = qualification.host();
        if host.source != source.id() {
            return Err(DomError::HostedSourceUnavailable);
        }
        let observation = source.observe();
        observation
            .validate(
                source.id(),
                crate::hosted::Role::Host,
                host.slot,
                host.generation,
            )
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        if let FocusQualification::DeniedDocument { global, .. } = qualification {
            self.draft.source_step(cancel)?;
            if global.source != source.id() {
                return Err(DomError::HostedSourceUnavailable);
            }
            observation
                .validate(
                    source.id(),
                    crate::hosted::Role::Global,
                    global.slot,
                    global.generation,
                )
                .map_err(|_| DomError::HostedSourceUnavailable)?;
        }
        Ok(())
    }
    pub(super) fn apply_dialog_focus(
        &mut self,
        request: &DomDialogFocusRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut frame = self.focus_frame(request)?;
        self.draft.source_step(cancel)?;
        if matches!(request, DomDialogFocusRequest::Remember(_)) {
            // An already-null slot remains a semantic no-op.
            self.draft
                .element_mut(frame.subject)
                .remember_no_focused_element();
            frame
                .cursor
                .reply(HtmlDialogReply::Applied)
                .map_err(DomError::DialogProtocol)?;
            frame.invocation = Invocation::Unissued;
        } else {
            let called = frame
                .focusing
                .as_mut()
                .ok_or(DomError::InvalidSourceOperation)?;
            called
                .cursor
                .reply(surgeist_html::HtmlDialogFocusingReply::AllowFocus(false))
                .map_err(DomError::DialogFocusingProtocol)?;
            called.invocation = Invocation::Unissued;
        }
        // Denied allow-focus returns before all positive focusing effects.
        self.draft.source_step(cancel)?;
        self.draft.dialog_method = Some(frame);
        Ok(())
    }
}
impl DomSourceLease<'_> {
    /// Prepare the actual original Show occurrence for its qualified negative
    /// focus producer. Missing evidence remains unavailable.
    pub fn prepare_dialog_focus(
        &self,
        request: &DomDialogFocusRequest,
        source: &DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<Option<DomDialogFocusPreparation>, DomError> {
        self.dom
            .validate_hosted_source(source)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        let op = self
            .dom
            .source_operation
            .as_ref()
            .filter(|op| op.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        op.focus_frame(request)?;
        if op.receipt.is_some() {
            return Err(DomError::InvalidSourceOperation);
        }
        op.draft.source_step(cancel)?;
        let Some(qualification) = op.focus_qualification(request, cancel)? else {
            return Ok(None);
        };
        op.validate_focus_producer(qualification, source, cancel)?;
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
        Ok(Some(DomDialogFocusPreparation {
            request: request.clone(),
            qualification,
            basis: self.dom.snapshot().stamp(),
        }))
    }
    /// Transfer the exact authentic observation into the original receipt before
    /// fallible cursor application/publication. Rejection preserves the result.
    pub fn accept_dialog_focus(
        &mut self,
        request: &DomDialogFocusRequest,
        source: &DomHostedSource,
        publication: &mut Option<DomDialogFocusPublication>,
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
        op.focus_frame(request)?;
        let actual = publication
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        if &actual.request != request
            || actual.basis != basis
            || op.receipt.is_some()
            || actual.qualification.host().source != source.id()
            || Some(actual.qualification) != op.focus_qualification(request, cancel)?
        {
            return Err(DomError::InvalidSourceOperation);
        }
        op.validate_focus_producer(actual.qualification, source, cancel)?;
        self.dom.admit_source_owner_payload(cancel)?;
        let op = self
            .dom
            .source_operation
            .as_mut()
            .expect("qualified original operation");
        op.draft.source_step(cancel)?;
        op.refresh_admission(cancel)?;
        op.receipt = Some(Receipt::DialogFocus {
            publication: publication.take().expect("qualified authentic observation"),
            applied: false,
        });
        Ok(())
    }
}

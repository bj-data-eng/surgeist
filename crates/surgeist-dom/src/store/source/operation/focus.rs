//! Qualified negative focus observations join the original Show receipt.
use super::*;
use crate::{
    DomDialogFocusPreparation, DomDialogFocusPublication, DomHostedSource,
    DomSourceExecutionEntryScope, source::FocusQualification,
};
use surgeist_html::{HtmlDialogAction, HtmlDialogReply};
impl Operation {
    fn focus_frame(
        &self,
        request: &DomSourceDialogRequest,
    ) -> Result<crate::store::dialog::MethodFrame, DomError> {
        let occurrence = request.method().ok_or(DomError::InvalidSourceOperation)?;
        let frame = self
            .draft
            .dialog_method
            .ok_or(DomError::InvalidSourceOperation)?;
        if request.operation != self.id
            || !matches!(
                request.action,
                HtmlDialogAction::RememberFocusedElement | HtmlDialogAction::RunDialogFocusingSteps
            )
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
    fn focus_qualification(
        &self,
        request: &DomSourceDialogRequest,
        cancel: &AtomicBool,
    ) -> Result<Option<FocusQualification>, DomError> {
        let frame = self.focus_frame(request)?;
        if request.action == HtmlDialogAction::RememberFocusedElement {
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
        // allow focus reads the receiver's THEN-current node document. Neither
        // the entry association nor the earlier popover document substitutes.
        self.draft.source_step(cancel)?;
        let document = self.draft.node(frame.subject)?.document;
        self.draft.source_step(cancel)?;
        let environment = self
            .draft
            .document_metadata(document)?
            .document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(None);
        };
        let retired = matches!(
            association.phase,
            crate::hosted::DocumentPhase::Retired { .. }
        );
        // The exact allowed-to-use order is BC, fully active, then inherited
        // permission. A constructed/incomplete association supplies no activity
        // answer; unknown state cannot establish denial.
        if environment.browsing_context() == surgeist_html::HtmlBrowsingContextPresence::Present {
            if !matches!(
                association.phase,
                crate::hosted::DocumentPhase::InitialEntry { .. }
            ) {
                return Ok(None);
            }
            if self.draft.document_fully_active(document, cancel)? {
                self.draft.source_step(cancel)?;
                if association.policy.inherited_focus_without_user_activation()
                    == crate::DomPermissionsPolicyValue::Enabled
                {
                    // Permission grants allow-focus before activation. The
                    // positive focusing algorithm remains a genuine wait.
                    return Ok(None);
                }
            }
        } else if !retired {
            return Ok(None);
        }
        // Explicit retirement names a historical Window; current denial names
        // the live association. Producer validation preserves that distinction.
        let document = self
            .draft
            .published_id(document)
            .map(DocumentId)
            .ok_or(DomError::UnavailableCreation)?;
        Ok(Some(if retired {
            FocusQualification::RetiredDocument {
                document,
                global: association.global,
                host: association.host,
            }
        } else {
            FocusQualification::DeniedDocument {
                document,
                global: association.global,
                host: association.host,
            }
        }))
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
        request: &DomSourceDialogRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut frame = self.focus_frame(request)?;
        self.draft.source_step(cancel)?;
        if request.action == HtmlDialogAction::RememberFocusedElement {
            // An already-null slot remains a semantic no-op.
            self.draft
                .element_mut(frame.subject)
                .remember_no_focused_element();
        }
        // Denied allow-focus returns before all positive focusing effects.
        self.draft.source_step(cancel)?;
        frame
            .cursor
            .reply(HtmlDialogReply::Applied)
            .map_err(DomError::DialogProtocol)?;
        frame.invocation = Invocation::Unissued;
        self.draft.dialog_method = Some(frame);
        Ok(())
    }
}
impl DomSourceLease<'_> {
    /// Prepare the actual original Show occurrence for its qualified negative
    /// focus producer. Missing evidence remains unavailable.
    pub fn prepare_dialog_focus(
        &self,
        request: &DomSourceDialogRequest,
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
        request: &DomSourceDialogRequest,
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

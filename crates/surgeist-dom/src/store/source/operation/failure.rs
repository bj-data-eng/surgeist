//! First native failure custody belongs to the original Source operation.
use super::*;

impl DomSourceCleanupCause {
    pub(super) fn into_error(self, operation: DomSourceOperationId) -> DomError {
        match self {
            Self::InvariantFailed(failure) => {
                DomError::SourceInvariantFailed { operation, failure }
            }
            Self::SvgScriptProcessingUnavailable => DomError::SvgScriptProcessingUnavailable,
            Self::ReceiverRetired {
                subject,
                action: DomSourceStoppedAction::Dialog(action),
            } => DomError::SourceReceiverRetired {
                operation,
                subject,
                action,
            },
            Self::ReceiverRetired {
                subject,
                action: DomSourceStoppedAction::CloseWatcher(action),
            } => DomError::SourceCloseWatcherReceiverRetired {
                operation,
                subject,
                action,
            },
        }
    }
}
impl Operation {
    pub(super) fn can_close_svg_failure(&self, dom: &Dom) -> bool {
        self.published
            && dom.state.incomplete_source == Some(self.id)
            && matches!(self.commands, OperationRoot::Edit(_))
            && !self.event_exposed
            && self.event_custody.is_none()
            && self.child.is_none()
            && self.hosted_global.is_none()
            && self.request.is_none()
            && self.receipt.is_none()
            && self.owner_binding.is_none()
            && self.request_close.is_none()
            && self.inline_source.is_none()
            && self.meta_policy.is_none()
            && self.pending_meta.is_none()
            && self.meta_receipt.is_none()
            && self.draft.method.is_none()
            && self.draft.compound_failure().is_none()
    }

    pub(super) fn remember_failure(&mut self, cause: DomSourceCleanupCause) {
        if self.failure.is_none() {
            self.failure = Some(cause);
        }
    }
}
impl DomSourceLease<'_> {
    pub(super) fn finish_source_failure(
        &mut self,
        op: &mut Operation,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        let cause = op.failure.ok_or(DomError::InvalidSourceOperation)?;
        if op.request_close.as_ref().is_some_and(|call| !call.settled) {
            return Err(DomError::InvalidSourceOperation);
        }
        // Only the already-settled original operation releases its own marker.
        // This publication never resolves a dead receiver or runs its method.
        if cause == DomSourceCleanupCause::SvgScriptProcessingUnavailable {
            // This failure abandons the unpublished suffix. Form the closing
            // marker from canonical state under the original Source account.
            let clean = Draft::new_source_failure_close(self.dom, &op.draft, cancel)?;
            let rejected = std::mem::replace(&mut op.draft, dialog::OperationDraft::Segment(clean));
            if let Err(error) =
                op.publication(self.dom, false, PublicationTarget::None, None, cancel)
            {
                op.draft = rejected;
                return Err(error);
            }
        } else {
            op.draft.source_set_method(None, cancel)?;
            op.publication(self.dom, false, PublicationTarget::None, None, cancel)?;
        }
        op.receipt = None;
        self.complete = true;
        Err(cause.into_error(op.id))
    }
}

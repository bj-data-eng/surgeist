//! First native failure custody belongs to the original Source operation.
use super::*;

impl DomSourceCleanupCause {
    pub(super) fn into_error(self, operation: DomSourceOperationId) -> DomError {
        match self {
            Self::InvariantFailed(failure) => {
                DomError::SourceInvariantFailed { operation, failure }
            }
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
        op.draft.method = None;
        op.publication(self.dom, false, PublicationTarget::None, cancel)?;
        op.receipt = None;
        self.complete = true;
        Err(cause.into_error(op.id))
    }
}

//! Affine results from the original Window manager, consumed by Source receipts.
use crate::{DomCloseWatcherId, DomSourceDialogRequest};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HookOccurrence {
    pub(crate) command: usize,
    pub(crate) ordinal: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DialogOccurrence {
    Hook(HookOccurrence),
    Method(HookOccurrence),
}
/// Exact source-qualified manager effect. The trusted owner performs the real
/// callback-free registration/removal before publishing this capability.
#[derive(Debug)]
pub struct DomDialogWatcherPreparation {
    pub(crate) request: DomSourceDialogRequest,
    pub(crate) watcher: DomCloseWatcherId,
}
impl DomDialogWatcherPreparation {
    pub const fn watcher(&self) -> DomCloseWatcherId {
        self.watcher
    }
    pub fn request(&self) -> &DomSourceDialogRequest {
        &self.request
    }
    /// Attest the prepared actual Window-manager effect, without a DOM fence.
    pub fn publish(self) -> DomDialogWatcherPublication {
        DomDialogWatcherPublication {
            request: self.request,
            watcher: self.watcher,
        }
    }
}
/// Move-only actual manager effect retained until exact Source acceptance.
#[derive(Debug)]
pub struct DomDialogWatcherPublication {
    pub(crate) request: DomSourceDialogRequest,
    pub(crate) watcher: DomCloseWatcherId,
}

/// One original standard requestClose call, distinct from each issued action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceCloseWatcherInvocationId {
    pub(crate) operation: crate::DomSourceOperationId,
    pub(crate) method: u64,
}
impl DomSourceCloseWatcherInvocationId {
    pub const fn operation(self) -> crate::DomSourceOperationId {
        self.operation
    }
    pub const fn method_occurrence(self) -> u64 {
        self.method
    }
}
/// Exact original watcher service, or release of its started invocation custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceCloseWatcherStep {
    Action(surgeist_html::HtmlCloseWatcherRequestAction),
    SettleInvocation,
}
/// Immutable original-operation qualification, without execution authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomSourceCloseWatcherRequest {
    pub(crate) invocation: DomSourceCloseWatcherInvocationId,
    pub(crate) watcher: DomCloseWatcherId,
    pub(crate) step: DomSourceCloseWatcherStep,
    pub(crate) occurrence: u64,
    pub(crate) cause: Option<DomSourceCleanupCause>,
}
impl DomSourceCloseWatcherRequest {
    pub const fn operation(&self) -> crate::DomSourceOperationId {
        self.invocation.operation
    }
    pub const fn invocation(&self) -> DomSourceCloseWatcherInvocationId {
        self.invocation
    }
    pub const fn watcher(&self) -> DomCloseWatcherId {
        self.watcher
    }
    pub const fn step(&self) -> DomSourceCloseWatcherStep {
        self.step
    }
    pub const fn current_occurrence(&self) -> u64 {
        self.occurrence
    }
    /// Original stopped native action while actual invocation cleanup remains.
    pub const fn cleanup_cause(&self) -> Option<DomSourceCleanupCause> {
        self.cause
    }
}
/// Exact externally stopped native action retained while cleanup is pending.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceStoppedAction {
    Dialog(surgeist_html::HtmlDialogAction),
    CloseWatcher(surgeist_html::HtmlCloseWatcherRequestAction),
}
/// Original native failure whose real invocation cleanup must still settle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceCleanupCause {
    ReceiverRetired {
        subject: crate::NodeId,
        action: DomSourceStoppedAction,
    },
}
/// Trusted original-owner attestation capability for one reached service.
#[derive(Debug)]
pub struct DomCloseWatcherRequestPreparation {
    pub(crate) request: DomSourceCloseWatcherRequest,
    pub(crate) cursor: surgeist_html::HtmlCloseWatcherRequestCursor,
}
impl DomCloseWatcherRequestPreparation {
    pub fn request(&self) -> &DomSourceCloseWatcherRequest {
        &self.request
    }
    pub fn publish_reply(
        self,
        reply: surgeist_html::HtmlCloseWatcherRequestReply,
    ) -> Result<DomCloseWatcherRequestPublication, crate::DomError> {
        use surgeist_html::HtmlCloseWatcherRequestAction as A;
        if !matches!(self.request.step, DomSourceCloseWatcherStep::Action(action) if !matches!(action, A::CaptureWindow | A::RunCancelAction { .. } | A::RunCloseAction | A::Complete(_)))
        {
            return Err(crate::DomError::InvalidSourceOperation);
        }
        let mut cursor = self.cursor;
        cursor
            .reply(reply)
            .map_err(crate::DomError::CloseWatcherProtocol)?;
        Ok(DomCloseWatcherRequestPublication {
            request: self.request,
            reply: Some(reply),
        })
    }
    pub fn publish_settled(self) -> Result<DomCloseWatcherRequestPublication, crate::DomError> {
        if self.request.step != DomSourceCloseWatcherStep::SettleInvocation {
            return Err(crate::DomError::InvalidSourceOperation);
        }
        Ok(DomCloseWatcherRequestPublication {
            request: self.request,
            reply: None,
        })
    }
}
/// Move-only effect result; admission refusal preserves the caller's publication.
#[derive(Debug)]
pub struct DomCloseWatcherRequestPublication {
    pub(crate) request: DomSourceCloseWatcherRequest,
    pub(crate) reply: Option<surgeist_html::HtmlCloseWatcherRequestReply>,
}

/// Admitted original-Window history-consumption targets. The original hosted
/// retention keeps its actual backing charged until the capture is dropped.
pub struct DomCloseWatcherHistoryWindows {
    pub(crate) windows: Vec<crate::DomHostedGlobalId>,
    pub(crate) navigable: Option<crate::DomHostedNavigableId>,
    pub(crate) _retained: crate::DomHostedRuntimeRetention,
}
impl DomCloseWatcherHistoryWindows {
    pub fn windows(&self) -> &[crate::DomHostedGlobalId] {
        &self.windows
    }
    pub const fn window_navigable(&self) -> Option<crate::DomHostedNavigableId> {
        self.navigable
    }
}

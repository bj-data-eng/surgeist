//! Qualified execution-resource withdrawal, preserving the original root history.
use super::*;
use crate::dom_lifecycle::{DomHostedRuntimeError, HostedRuntime};
use surgeist_dom::{
    DocumentId, DocumentString, DomHostedOperationId, DomHostedStyleRootWithdrawal,
};

/// Refusal retains both actual owners and the same destruction task.
#[derive(Debug)]
pub enum DomStyleRootWithdrawalError {
    /// Actual Style owner, execution phase or finite admission refused.
    Style(DomStyleRuntimeError),
    /// Actual hosted task, original cleanup or canonical producer refused.
    Hosted(DomHostedRuntimeError),
}
impl From<DomStyleRuntimeError> for DomStyleRootWithdrawalError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Style(error)
    }
}
impl From<DomHostedRuntimeError> for DomStyleRootWithdrawalError {
    fn from(error: DomHostedRuntimeError) -> Self {
        Self::Hosted(error)
    }
}
impl std::fmt::Display for DomStyleRootWithdrawalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Style(error) => error.fmt(f),
            Self::Hosted(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for DomStyleRootWithdrawalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Style(error) => Some(error),
            Self::Hosted(error) => Some(error),
        }
    }
}

/// Historical observations of one actual withdrawal. This grants neither a
/// current writer nor destruction completion; independently escaped mutable
/// producer capabilities retain their own lifetime.
pub struct DomStyleRootWithdrawal {
    // Dispose all upper name/header backing before the lower and upper receipts.
    history: Option<sheet_sets::State>,
    invocation: DomHostedOperationId,
    lower: DomHostedStyleRootWithdrawal,
    _retention: surgeist_dom::DomHostedRuntimeRetention,
}
impl std::fmt::Debug for DomStyleRootWithdrawal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomStyleRootWithdrawal")
            .field("root", &self.root())
            .field("invocation", &self.invocation)
            .field("cleanup", &self.cleanup_operation())
            .finish_non_exhaustive()
    }
}
/// Borrowed withdrawal-time preference history, never a second mutable owner.
pub struct DomStyleWithdrawnSheetSetHistory<'a> {
    state: &'a sheet_sets::State,
}
impl DomStyleWithdrawnSheetSetHistory<'_> {
    /// Actual preferred name retained at withdrawal.
    pub fn preferred(&self) -> &DocumentString {
        self.state.preferred()
    }
    /// Actual last-selected name retained at withdrawal.
    pub fn last_selected(&self) -> Option<&DocumentString> {
        self.state.last_selected()
    }
    /// The sole preference owner's revision at withdrawal.
    pub fn revision(&self) -> u64 {
        self.state.revision()
    }
}
impl DomStyleRootWithdrawal {
    /// Exact canonical root whose execution custody was withdrawn.
    pub fn root(&self) -> NodeId {
        self.lower.root()
    }
    /// Actual Document of the authenticated original cleanup.
    pub fn document(&self) -> DocumentId {
        self.lower.document()
    }
    /// Original invocation selected through its actual retained permission.
    pub fn invocation(&self) -> DomHostedOperationId {
        self.invocation
    }
    /// Same original cleanup operation which the ordinary pump must resume.
    pub fn cleanup_operation(&self) -> DomHostedOperationId {
        self.lower.cleanup_operation()
    }
    /// Actual capture under the withdrawal lock; it is not a current snapshot.
    pub fn withdrawal_cssom_snapshot(&self) -> &CssomSnapshot {
        self.lower.withdrawal_cssom_snapshot()
    }
    /// Present only for an actual Document root; ShadowRoot has no preference owner.
    pub fn document_sheet_sets(&self) -> Option<DomStyleWithdrawnSheetSetHistory<'_>> {
        self.history
            .as_ref()
            .map(|state| DomStyleWithdrawnSheetSetHistory { state })
    }
}
impl DomStyleRuntime {
    /// Withdraw this original owner's idle execution resource only at its
    /// reached, retained hosted destruction Cleanup task. All canonical and
    /// Runtime obligations must already be settled. Refusal keeps the same
    /// owners, original publications and cumulative budgets for a raised retry.
    ///
    /// Success preserves the original label, serial and preference owner in a
    /// closed released phase. Resume the SAME destruction using its normal pump;
    /// this supplies no acknowledgement, resource outcome or cap revocation.
    pub fn withdraw_root_for_hosted_destruction(
        &self,
        dom: &mut DomEventDom<'_>,
        hosted: &HostedRuntime,
        invocation: DomHostedOperationId,
        root: NodeId,
        cancel: &AtomicBool,
    ) -> Result<DomStyleRootWithdrawal, DomStyleRootWithdrawalError> {
        // Fixed lock order: original Hosted owner, original Style owner, actual
        // native task, then lower producer. None is held across callbacks.
        let hosted_owner = hosted.style_withdrawal_owner()?;
        let mut hosted_state = hosted_owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let selection = hosted_state.select_style_cleanup(dom, invocation, cancel)?;
        let mut cleanup = hosted_state.style_cleanup_context(&selection, dom, cancel)?;
        let owner = self.qualified(dom)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        let task = selection.hold(&mut cleanup, cancel)?;
        state.step(cancel)?;
        if state.staged.is_some() || state.publication.is_some() {
            return Err(DomStyleRuntimeError::SheetSetPublicationPending.into());
        }
        let mut selected = None;
        for (index, bound) in state.roots.iter().enumerate() {
            state.step(cancel)?;
            if bound.node == root {
                selected = Some(index);
                break;
            }
        }
        let index = selected.ok_or(DomStyleRuntimeError::SheetSetRootUnavailable { root })?;
        state.step(cancel)?;
        let capture = dom.snapshot();
        let node = capture.node(root).map_err(DomStyleRuntimeError::from)?;
        if node
            .as_document()
            .map(|document| document.id())
            .or_else(|| node.owner_document())
            != Some(cleanup.document)
        {
            return Err(DomHostedRuntimeError::Binding.into());
        }
        drop(capture);
        let bound = state.roots[index].bound()?;
        for row in &state.completions {
            state.step(cancel)?;
            if row.owner().root() == root || std::rc::Rc::ptr_eq(row.owner(), bound) {
                return Err(DomStyleRuntimeError::RootWorkPending { root }.into());
            }
        }
        // Actual output header/name aliases, external cap loan and old/new
        // resource retirement frame are prepaid before cloning/moving them.
        let upper_header = size_of::<DomStyleRootWithdrawal>()
            .checked_sub(size_of::<DomHostedStyleRootWithdrawal>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        let frame = upper_header
            .checked_add(size_of::<std::rc::Rc<DomStyleCssomRoot>>())
            .and_then(|n| n.checked_add(2 * size_of::<RootResource>()))
            .ok_or(DomStyleRuntimeError::Admission)?;
        state.admit(frame, cancel)?;
        cleanup.step(cancel)?;
        // The same immutable names remain in this original Document RootSets
        // until Session disposal (Document retirement is unsupported). Output
        // aliases add headers, not another name allocation/projection. Shadow
        // roots have no such history. Lower already owns its embedded header.
        let output_bytes = upper_header;
        let retention = cleanup
            .source
            .retain_runtime_payload(dom.raw_mut(), output_bytes, cancel)
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        state.step(cancel)?;
        let history = state.roots[index].sheet_sets.state().cloned();
        let cap = state.roots[index].bound()?.clone();
        cleanup.step(cancel)?;
        let prep = cleanup
            .lease
            .prepare_style_root_withdrawal(
                dom.raw_mut(),
                cleanup.source,
                cleanup.publication,
                &cap,
                cancel,
            )
            .map_err(DomHostedRuntimeError::Dom)?;
        let next = RootResource::ReleasedForHostedDestruction {
            cleanup: cleanup.lease.operation(),
            at_withdrawal: prep.withdrawal_snapshot().revision().clone(),
        };
        state.step(cancel)?;
        cleanup.step(cancel)?;
        // Exclusive infallible joint fence: install both semantic states before
        // unlocking or disposing any displaced mutable capability/custody.
        let resource = &mut state.roots[index].resource;
        let published = prep.publish();
        let retired = std::mem::replace(resource, next);
        let lower = published.finish();
        drop(retired);
        drop(cap);
        drop(task);
        Ok(DomStyleRootWithdrawal {
            history,
            invocation: cleanup.invocation,
            lower,
            _retention: retention,
        })
    }
}

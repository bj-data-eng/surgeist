//! Source-issued child lifecycle phases. These observations do not carry the
//! unique issuer or move the original canonical edit/controller out of its owner.
use super::*;
use crate::{DomDocumentStamp, DomSourceOperationId, DomStamp, NodeId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomHostedChildRequest {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) occurrence: u64,
    pub(crate) basis: DomStamp,
    pub(crate) container: NodeId,
    pub(crate) document: DomDocumentStamp,
    pub(crate) parent: DomHostedNavigableId,
    pub(crate) parent_global: DomHostedGlobalId,
    pub(crate) traversable: DomHostedNavigableId,
    pub(crate) host: DomHostedHostId,
}
impl DomHostedChildRequest {
    pub fn operation(&self) -> DomSourceOperationId {
        self.operation
    }
    pub fn occurrence(&self) -> u64 {
        self.occurrence
    }
    pub fn basis(&self) -> DomStamp {
        self.basis
    }
    pub fn container(&self) -> NodeId {
        self.container
    }
    pub fn parent_document(&self) -> DomDocumentStamp {
        self.document
    }
    pub fn parent_navigable(&self) -> DomHostedNavigableId {
        self.parent
    }
    pub fn parent_global(&self) -> DomHostedGlobalId {
        self.parent_global
    }
    pub fn traversable(&self) -> DomHostedNavigableId {
        self.traversable
    }
    pub fn host(&self) -> DomHostedHostId {
        self.host
    }
}
/// Original BC-prefix ownership, before the real external object is attested.
#[derive(Debug)]
pub struct DomHostedChildPreparation {
    pub(crate) request: DomHostedChildRequest,
    pub(crate) host: DomHostedHostId,
    pub(crate) context: DomHostedBrowsingContextId,
    pub(crate) work: Arc<Work>,
}
impl DomHostedChildPreparation {
    pub fn browsing_context(&self) -> DomHostedBrowsingContextId {
        self.context
    }
    pub fn host(&self) -> DomHostedHostId {
        self.host
    }
}
/// Actual producer acknowledgement of that exact retained original prefix.
#[derive(Debug)]
pub struct DomHostedPreparedChild {
    pub(crate) preparation: DomHostedChildPreparation,
}
impl DomHostedSource {
    pub(crate) fn prepare_child_prefix(
        &mut self,
        request: &DomHostedChildRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedChildPreparation, DomHostedError> {
        if self.active_work.upgrade().is_some() {
            return Err(DomHostedError::IncompleteOperation);
        }
        self.observe().validate(
            request.host.source,
            Role::Host,
            request.host.slot,
            request.host.generation,
        )?;
        let work = self.work(cancel)?;
        let (slot, generation) =
            self.issue(Role::BrowsingContext, &work, false, None, 0, cancel)?;
        self.active_work = Arc::downgrade(&work);
        Ok(DomHostedChildPreparation {
            request: request.clone(),
            host: request.host,
            context: DomHostedBrowsingContextId {
                source: self.id(),
                slot,
                generation,
            },
            work,
        })
    }
    /// Seal only after the original producer created its real BC and retained
    /// the actual raw creation sample. No clock or external object is faked here.
    pub fn seal_child_preparation(
        &self,
        request: &DomHostedChildRequest,
        preparation: &mut Option<DomHostedChildPreparation>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedPreparedChild, DomHostedError> {
        let current = preparation
            .as_ref()
            .ok_or(DomHostedError::MissingPublication)?;
        if &current.request != request {
            return Err(DomHostedError::WrongPhase);
        }
        current.work.step(cancel)?;
        self.observe().validate(
            current.context.source,
            Role::BrowsingContext,
            current.context.slot,
            current.context.generation,
        )?;
        Ok(DomHostedPreparedChild {
            preparation: preparation.take().expect("validated exact BC preparation"),
        })
    }
}

/// The exact native phase reached inside the original Source controller.
#[derive(Debug)]
pub enum DomHostedChildStep<'a> {
    Prepare(&'a DomHostedChildRequest),
    PrepareRemoval(&'a DomHostedChildRequest),
    PermissionsPolicy(&'a DomHostedPolicyRequest),
    NativeExecution(&'a DomHostedExecutionRequest),
    MakeActive(&'a DomHostedMakeActiveRequest),
    FinishLoading(&'a DomHostedFinishLoadingRequest),
    InitialEntry(&'a DomHostedEntryRequest),
    QueueHistory(&'a DomHostedChildHistoryRequest),
    Removal(&'a DomHostedChildRemovalRequest),
    NeedsUrl(crate::DomSourceUrlStage),
    Returned,
}
/// Actual queued creation C input. It preserves the original parent/traversable
/// and E; the parent active DocumentState is selected when C executes.
#[derive(Debug)]
pub struct DomHostedChildHistoryRequest {
    pub(crate) request: Request,
    pub(crate) parent: DomHostedNavigableId,
    pub(crate) traversable: DomHostedNavigableId,
    pub(crate) creation: DomHostedCreation,
}
impl DomHostedChildHistoryRequest {
    pub fn operation(&self) -> DomHostedOperationId {
        self.request.key.operation
    }
    pub fn parent_navigable(&self) -> DomHostedNavigableId {
        self.parent
    }
    pub fn traversable(&self) -> DomHostedNavigableId {
        self.traversable
    }
    pub fn creation(&self) -> DomHostedCreation {
        self.creation
    }
}
/// Destruction starts after canonical live-edge unlinking. The former relation
/// remains real cleanup/history custody until actual generation completion.
#[derive(Debug)]
pub struct DomHostedChildRemovalRequest {
    pub(crate) request: Request,
    pub(crate) parent: DomHostedNavigableId,
    pub(crate) traversable: DomHostedNavigableId,
    pub(crate) creation: DomHostedCreation,
}
impl DomHostedChildRemovalRequest {
    pub fn operation(&self) -> DomHostedOperationId {
        self.request.key.operation
    }
    pub fn parent_navigable(&self) -> DomHostedNavigableId {
        self.parent
    }
    pub fn traversable(&self) -> DomHostedNavigableId {
        self.traversable
    }
    pub fn creation(&self) -> DomHostedCreation {
        self.creation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostedEmbedderKind {
    Frame,
    Iframe,
}
/// Retained source-qualified raw facts, not a final Permissions Policy answer.
#[derive(Debug)]
pub struct DomHostedEmbedderPolicyInputs {
    pub(crate) kind: DomHostedEmbedderKind,
    pub(crate) parent_document: DomDocumentStamp,
    pub(crate) parent_origin: crate::DomOpaqueOrigin,
    pub(crate) parent_fullscreen: DomFullscreenInherited,
    pub(crate) parent_sandbox: surgeist_html::HtmlSandboxFlags,
    pub(crate) embedder_sandbox: surgeist_html::HtmlSandboxFlags,
    pub(crate) sandbox: Option<crate::DocumentString>,
    pub(crate) srcdoc: Option<crate::DocumentString>,
    pub(crate) src: Option<crate::DocumentString>,
    pub(crate) allow: Option<crate::DocumentString>,
    pub(crate) allow_fullscreen: bool,
    pub(crate) _retained: Retained,
}
impl DomHostedEmbedderPolicyInputs {
    pub fn kind(&self) -> DomHostedEmbedderKind {
        self.kind
    }
    pub fn parent_document(&self) -> DomDocumentStamp {
        self.parent_document
    }
    pub fn parent_origin(&self) -> &crate::DomOpaqueOrigin {
        &self.parent_origin
    }
    pub fn parent_fullscreen(&self) -> DomFullscreenInherited {
        self.parent_fullscreen
    }
    pub fn parent_sandbox_flags(&self) -> surgeist_html::HtmlSandboxFlags {
        self.parent_sandbox
    }
    pub fn embedder_sandbox_flags(&self) -> surgeist_html::HtmlSandboxFlags {
        self.embedder_sandbox
    }
    /// The actual raw attribute used by Permissions Policy's declared-origin
    /// step. It is independent of the retained creation sandbox flag set.
    pub fn sandbox(&self) -> Option<&crate::DocumentString> {
        self.sandbox.as_ref()
    }
    pub fn srcdoc(&self) -> Option<&crate::DocumentString> {
        self.srcdoc.as_ref()
    }
    pub fn src(&self) -> Option<&crate::DocumentString> {
        self.src.as_ref()
    }
    pub fn allow(&self) -> Option<&crate::DocumentString> {
        self.allow.as_ref()
    }
    pub fn allow_fullscreen(&self) -> bool {
        self.allow_fullscreen
    }
}

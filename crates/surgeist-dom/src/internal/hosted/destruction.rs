//! Permissions over the original parallel destruction invocation and its exact
//! captured obligations. Mutable continuations remain in the canonical owner.
use super::*;

#[derive(Debug)]
pub struct DomHostedDestructionInvocation {
    pub(crate) operation: DomHostedOperationId,
    pub(crate) navigable: DomHostedNavigableId,
}
impl DomHostedDestructionInvocation {
    pub fn operation(&self) -> DomHostedOperationId {
        self.operation
    }
    pub fn navigable(&self) -> DomHostedNavigableId {
        self.navigable
    }
}
#[derive(Debug)]
pub struct DomHostedDestructionInvocationRequest {
    pub(crate) request: Request,
    pub(crate) creation: DomHostedCreation,
    pub(crate) traversable: DomHostedNavigableId,
    pub(crate) cleanup: Option<DomHostedOperationId>,
    pub(crate) children: Arc<CapturedChildren>,
}
impl DomHostedDestructionInvocationRequest {
    pub fn operation(&self) -> DomHostedOperationId {
        self.request.key.operation
    }
    pub fn cleanup_operation(&self) -> DomHostedOperationId {
        self.cleanup
            .expect("only a prepared invocation is publicly lent")
    }
    pub fn document(&self) -> crate::DocumentId {
        self.creation.document
    }
    pub fn global(&self) -> DomHostedGlobalId {
        self.creation.global
    }
    pub fn navigable(&self) -> DomHostedNavigableId {
        self.creation.navigable
    }
    pub fn traversable(&self) -> DomHostedNavigableId {
        self.traversable
    }
    pub fn child_documents(&self) -> &[DomHostedCreation] {
        &self.children.values
    }
}
#[derive(Debug)]
pub struct DomHostedChildRemovalPublication {
    pub(crate) key: RequestKey,
    pub(crate) revision: DomHostedSourceRevision,
}
/// Both permissions leave their original continuations retained in canonical
/// rows. Runtime preadmits their two execution destinations before acceptance.
#[derive(Debug)]
pub struct DomHostedChildRemovalHandoff {
    pub(crate) invocation: DomHostedDestructionInvocation,
    pub(crate) history: DomHostedQueuedHistory,
}
impl DomHostedChildRemovalHandoff {
    pub fn into_parts(self) -> (DomHostedDestructionInvocation, DomHostedQueuedHistory) {
        (self.invocation, self.history)
    }
}
impl DomHostedSource {
    pub fn seal_child_removal(
        &self,
        request: &DomHostedChildRemovalRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedChildRemovalPublication, DomHostedError> {
        let revision = self.validate_request(&request.request, cancel)?;
        Ok(DomHostedChildRemovalPublication {
            key: request.request.key,
            revision,
        })
    }
}

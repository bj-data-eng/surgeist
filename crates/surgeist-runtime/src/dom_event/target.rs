use super::*;
use surgeist_dom::{DocumentId, DomEventRecipientTarget, DomHostedGlobalId};

/// One actual endpoint in the native EventTarget registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomEventTarget {
    /// An exact canonical Node identity.
    Node(NodeId),
    /// An actual native Global identity.
    Global(DomHostedGlobalId),
    /// An authored EventTarget with its own listener list and null parent.
    Standalone(DomStandaloneEventTarget),
}

impl DomEventTarget {
    /// The exact Node when this endpoint is a Node.
    pub const fn node(&self) -> Option<NodeId> {
        match self {
            Self::Node(node) => Some(*node),
            Self::Global(_) | Self::Standalone(_) => None,
        }
    }

    /// The exact Global when this endpoint is a Global.
    pub const fn global(&self) -> Option<DomHostedGlobalId> {
        match self {
            Self::Global(global) => Some(*global),
            Self::Node(_) | Self::Standalone(_) => None,
        }
    }

    pub(super) fn recipient(&self) -> Option<DomEventRecipientTarget> {
        match self {
            Self::Node(node) => Some((*node).into()),
            Self::Global(global) => Some((*global).into()),
            Self::Standalone(_) => None,
        }
    }
}

impl From<NodeId> for DomEventTarget {
    fn from(node: NodeId) -> Self {
        Self::Node(node)
    }
}

impl From<DomHostedGlobalId> for DomEventTarget {
    fn from(global: DomHostedGlobalId) -> Self {
        Self::Global(global)
    }
}

impl From<DomStandaloneEventTarget> for DomEventTarget {
    fn from(target: DomStandaloneEventTarget) -> Self {
        Self::Standalone(target)
    }
}

struct StandaloneIdentity {
    registry: u64,
    global: DomHostedGlobalId,
    document: DocumentId,
}

/// Shared identity of an authored EventTarget and its original relevant Global.
/// A clone keeps the identity alive independently of registry authority.
#[derive(Clone)]
pub struct DomStandaloneEventTarget {
    identity: Rc<StandaloneIdentity>,
    // The fixed identity allocation is released before its receipt.
    _backing: Rc<EventAllocation>,
}

impl PartialEq for DomStandaloneEventTarget {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.identity, &other.identity)
    }
}

impl Eq for DomStandaloneEventTarget {}

impl std::fmt::Debug for DomStandaloneEventTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomStandaloneEventTarget")
            .field("global", &self.identity.global)
            .finish_non_exhaustive()
    }
}

impl DomStandaloneEventTarget {
    /// Actual relevant Global authenticated when this EventTarget was created.
    pub fn relevant_global(&self) -> DomHostedGlobalId {
        self.identity.global
    }

    pub(super) fn document(&self) -> DocumentId {
        self.identity.document
    }

    #[cfg(test)]
    pub(super) fn inspect_allocations(&self) -> (usize, &EventAllocation) {
        (
            size_of_val(self.identity.as_ref()) + size_of_val(self._backing.as_ref()),
            self._backing.as_ref(),
        )
    }
}

impl<C> DomEventDispatcher<C> {
    pub(crate) fn create_standalone_target(
        &self,
        dom: &mut DomEventDom<'_>,
        association: crate::dom_lifecycle::EventTargetAssociation,
        cancel: &AtomicBool,
    ) -> Result<DomStandaloneEventTarget, DomEventDispatchError> {
        self.association.validate(dom.raw())?;
        let (global, document) = association.into_parts();
        if document.node().owner() != dom.owner() {
            return Err(DomError::ForeignOwner.into());
        }
        self.storage.step(1, cancel)?;
        let bytes = size_of::<StandaloneIdentity>() + size_of::<EventAllocation>();
        let backing = self.storage.reserve(dom, bytes, bytes, 0, cancel)?;
        Ok(DomStandaloneEventTarget {
            identity: Rc::new(StandaloneIdentity {
                registry: self.owner.id,
                global,
                document,
            }),
            _backing: Rc::new(backing),
        })
    }

    pub(super) fn validate_target(
        &self,
        dom: &Dom,
        target: &DomEventTarget,
        cancel: &AtomicBool,
    ) -> Result<(), DomEventDispatchError> {
        match target {
            DomEventTarget::Node(node) => {
                dom.snapshot().node(*node)?;
            }
            DomEventTarget::Global(global) => {
                dom.dispatch_global_document(*global, cancel)?;
            }
            DomEventTarget::Standalone(target) => {
                if target.document().node().owner() != dom.owner() {
                    return Err(DomError::ForeignOwner.into());
                }
                if target.identity.registry != self.owner.id {
                    return Err(DomEventDispatchError::WrongTargetRegistry);
                }
            }
        }
        Ok(())
    }
}

//! Positive native event recipients and shared event-parent observations.
use crate::{DomError, DomHostedGlobalId, DomResource, NodeId};

/// Affine association of the one live listener-registry owner with a canonical
/// DOM. This capability owns no callbacks or tree and cannot be cloned. Dropping
/// it permits a fresh owner; it transfers no registrations or dispatch progress.
pub struct DomEventRegistryAssociation {
    pub(crate) binding: std::sync::Arc<RegistryBinding>,
    pub(crate) body: crate::store::custody::SharedDebit,
    pub(crate) payload: crate::store::custody::Debit,
}
pub(crate) struct RegistryBinding {
    pub(crate) owner: crate::DomOwnerId,
}
/// Admitted, unpublished registry storage. Drop discards preparation without
/// claiming the canonical listener list. Keep represented buffers alive with
/// this funding capability until they are freed or publication succeeds.
#[derive(Debug)]
pub struct DomEventRegistryPreparation(pub(crate) DomEventRegistryAssociation);
pub(crate) struct RegistryObservation {
    pub(crate) binding: std::sync::Weak<RegistryBinding>,
    pub(crate) _body: crate::store::custody::SharedDebit,
}
impl std::fmt::Debug for DomEventRegistryAssociation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomEventRegistryAssociation")
            .field("owner", &self.binding.owner)
            .finish_non_exhaustive()
    }
}
impl DomEventRegistryAssociation {
    /// Exact canonical owner; this scalar observation grants no execution.
    pub fn owner(&self) -> crate::DomOwnerId {
        self.binding.owner
    }
    /// Fixed producer-owned payload, excluding opaque allocator metadata.
    pub const fn binding_payload_bytes() -> usize {
        std::mem::size_of::<RegistryBinding>()
            + crate::store::custody::SharedDebit::administration_usage().bytes()
    }
    /// Actual binding body and admitted registry control backing.
    pub fn retained_bytes(&self) -> usize {
        self.body.usage().bytes() + self.payload.usage().bytes()
    }
}

/// One actual Node or Global event endpoint. An identity retains no tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomEventTarget {
    /// Exact owner-qualified Node identity.
    Node(NodeId),
    /// Actual issuer-qualified Global identity.
    Global(DomHostedGlobalId),
}
impl DomEventTarget {
    /// The Node endpoint, when selected.
    pub const fn node(self) -> Option<NodeId> {
        match self {
            Self::Node(node) => Some(node),
            Self::Global(_) => None,
        }
    }
    /// The Global endpoint, when selected.
    pub const fn global(self) -> Option<DomHostedGlobalId> {
        match self {
            Self::Global(global) => Some(global),
            Self::Node(_) => None,
        }
    }
}
impl From<NodeId> for DomEventTarget {
    fn from(value: NodeId) -> Self {
        Self::Node(value)
    }
}
impl From<DomHostedGlobalId> for DomEventTarget {
    fn from(value: DomHostedGlobalId) -> Self {
        Self::Global(value)
    }
}

/// The native event classes currently admitted by the DOM/Runtime integration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomEventType {
    /// Native load notification.
    Load,
    /// Native error notification.
    Error,
    /// Queued Details/Dialog state change.
    Toggle,
    /// Synchronous Dialog state-change observation.
    BeforeToggle,
    /// Queued Dialog close notification.
    Close,
    /// Synchronous close-watcher cancellation observation.
    Cancel,
}
impl DomEventType {
    /// Standard event type spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Error => "error",
            Self::Toggle => "toggle",
            Self::BeforeToggle => "beforetoggle",
            Self::Close => "close",
            Self::Cancel => "cancel",
        }
    }
    pub(crate) const fn notification(self) -> Option<usize> {
        match self {
            Self::Load => Some(0),
            Self::Error => Some(1),
            _ => None,
        }
    }
}

/// Independent finite ownership of positive Load/Error recipient capabilities.
/// These allowances do not represent already-started dispatch custody.
/// Releasing a lease frees its live occurrence. Endpoint index backing may
/// remain charged until the DOM owner is dropped and is reusable for the same
/// exact endpoint. An unallocated owner or zero live count needs no route work.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DomEventRecipientLimits {
    registrations: usize,
    retained_bytes: usize,
    scratch_bytes: usize,
    work: u64,
}
impl DomEventRecipientLimits {
    /// Construct explicit aggregate live, retained, temporary and work bounds.
    pub fn new(
        registrations: usize,
        retained_bytes: usize,
        scratch_bytes: usize,
        work: u64,
    ) -> Result<Self, DomError> {
        for (amount, resource) in [
            (registrations, DomResource::EventRecipients),
            (retained_bytes, DomResource::EventRecipientRetainedBytes),
            (scratch_bytes, DomResource::EventRecipientScratch),
        ] {
            if amount > isize::MAX as usize {
                return Err(DomError::InvalidLimit(resource));
            }
        }
        Ok(Self {
            registrations,
            retained_bytes,
            scratch_bytes,
            work,
        })
    }
    /// Maximum simultaneous positive recipient occurrences across registries.
    pub const fn registrations(self) -> usize {
        self.registrations
    }
    /// Actual owner, index, bucket and affine-lease storage allowance.
    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
    /// Temporary index reservations and topology-query storage allowance.
    pub const fn scratch_bytes(self) -> usize {
        self.scratch_bytes
    }
    /// Per-admission or reached closure-query work; Source is also cumulative.
    pub const fn work(self) -> u64 {
        self.work
    }
}

/// Affine positive recipient capability. Keep this beside the actual callback
/// registration; removal or owner destruction releases it. It owns no callback
/// and cannot retain canonical tree state or grant event delivery.
#[must_use]
pub struct DomEventRecipient {
    pub(crate) _lease: crate::store::event::Lease,
}
impl std::fmt::Debug for DomEventRecipient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomEventRecipient").finish_non_exhaustive()
    }
}

/// Inputs to one current canonical parent/ancestry observation. These are query
/// operands, not a caller-supplied no-recipient qualification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomEventParentQuery {
    pub(crate) current: NodeId,
    pub(crate) first_root: NodeId,
    pub(crate) adjusted_target: NodeId,
    pub(crate) event_type: DomEventType,
}
impl DomEventParentQuery {
    /// Query operands must name this issuing DOM's current snapshot. They do
    /// not grant permission to execute a dispatch or change processing history.
    pub const fn new(
        current: NodeId,
        first_root: NodeId,
        adjusted_target: NodeId,
        event_type: DomEventType,
    ) -> Self {
        Self {
            current,
            first_root,
            adjusted_target,
            event_type,
        }
    }
}

/// Current event-parent topology. Runtime retains path construction, target
/// adjustment, invocation and closed-tree visibility rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomEventParent {
    pub(crate) parent: Option<DomEventTarget>,
    pub(crate) assigned: bool,
    pub(crate) in_target_tree: Option<bool>,
}
impl DomEventParent {
    /// Actual parent endpoint; None terminates this path.
    pub const fn parent(self) -> Option<DomEventTarget> {
        self.parent
    }
    /// Whether the parent step follows stored assigned-slot state.
    pub const fn via_assigned_slot(self) -> bool {
        self.assigned
    }
    /// Present only for a Node parent. Its negation is Runtime's existing
    /// at-target condition; a Global parent is never at-target.
    pub const fn parent_in_adjusted_target_tree(self) -> Option<bool> {
        self.in_target_tree
    }
}

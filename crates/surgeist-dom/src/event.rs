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
pub enum DomEventRecipientTarget {
    /// Exact owner-qualified Node identity.
    Node(NodeId),
    /// Actual issuer-qualified Global identity.
    Global(DomHostedGlobalId),
}
impl DomEventRecipientTarget {
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
impl From<NodeId> for DomEventRecipientTarget {
    fn from(value: NodeId) -> Self {
        Self::Node(value)
    }
}
impl From<DomHostedGlobalId> for DomEventRecipientTarget {
    fn from(value: DomHostedGlobalId) -> Self {
        Self::Global(value)
    }
}

pub(crate) const LOAD_EVENT: &[u16] = &[108, 111, 97, 100];
pub(crate) const ERROR_EVENT: &[u16] = &[101, 114, 114, 111, 114];

#[derive(Clone, Copy)]
pub(crate) enum Notification {
    Load,
    Error,
}
impl Notification {
    pub(crate) fn from_type(event_type: &[u16]) -> Option<Self> {
        if event_type == LOAD_EVENT {
            Some(Self::Load)
        } else if event_type == ERROR_EVENT {
            Some(Self::Error)
        } else {
            None
        }
    }
    pub(crate) const fn event_type(self) -> &'static [u16] {
        match self {
            Self::Load => LOAD_EVENT,
            Self::Error => ERROR_EVENT,
        }
    }
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Load => 0,
            Self::Error => 1,
        }
    }
}

/// Affine backing custody for actual Runtime event values and dispatch storage.
/// This accounting receipt retains no tree, callback, registry or execution scope.
#[derive(Debug)]
pub struct DomEventPayloadRetention {
    pub(crate) debit: crate::store::custody::Debit,
}
impl DomEventPayloadRetention {
    /// Actual retained bytes, including any represented fixed targets.
    pub fn retained_bytes(&self) -> usize {
        self.debit.usage().bytes()
    }
    /// Actual retained UTF-16 capacity.
    pub fn retained_units(&self) -> usize {
        self.debit.usage().units()
    }
    /// Transfer part of this already-admitted backing into its actual holder.
    pub fn split(&mut self, bytes: usize, units: usize) -> Result<Self, DomError> {
        Ok(Self {
            debit: self
                .debit
                .split(crate::store::custody::Usage::new(bytes, units))?,
        })
    }
    /// Merge same-owner extents. A foreign extent is returned intact.
    pub fn absorb(&mut self, other: Self) -> Result<(), Self> {
        self.debit
            .absorb(other.debit)
            .map_err(|debit| Self { debit })
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
pub struct DomEventParentQuery<'event> {
    pub(crate) current: NodeId,
    pub(crate) first_root: NodeId,
    pub(crate) adjusted_target: NodeId,
    pub(crate) event: EventParentFacts<'event>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EventParentFacts<'event> {
    pub(crate) event_type: &'event [u16],
    pub(crate) composed: bool,
}
impl<'event> DomEventParentQuery<'event> {
    /// Query operands must name this issuing DOM's current snapshot. They do
    /// not grant permission to execute a dispatch or change processing history.
    pub const fn new(
        current: NodeId,
        first_root: NodeId,
        adjusted_target: NodeId,
        event_type: &'event [u16],
        composed: bool,
    ) -> Self {
        Self {
            current,
            first_root,
            adjusted_target,
            event: EventParentFacts {
                event_type,
                composed,
            },
        }
    }
}

/// Current event-parent topology. Runtime retains path construction, target
/// adjustment, invocation and closed-tree visibility rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomEventParent {
    /// The path ends here.
    None,
    /// A parent reached within the canonical node topology.
    Node {
        /// Exact current Node endpoint.
        parent: NodeId,
        /// Whether the step follows stored assigned-slot state.
        via_assigned_slot: bool,
        /// Whether the adjusted target's root is an inclusive shadow ancestor.
        in_adjusted_target_tree: bool,
    },
    /// The live Global at the end of a qualifying Document path.
    Global(DomHostedGlobalId),
}

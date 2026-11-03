//! The sole mutable owner of shared HTML construction records.

use super::*;
use std::mem::{align_of, size_of};

mod attribute_edit;
pub use attribute_edit::*;

mod operations;
pub use operations::*;

mod text_append;
pub use text_append::*;

/// One hidden allocation's role in a construction owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlConstructionAllocationRole {
    /// The shared identity allocation. Builder node handles and allocation ids refer to it
    /// weakly, while published node ids and the arena hold it strongly, so it distinguishes one
    /// construction owner from another.
    Identity,
    /// The vector of retained node records.
    Records,
    /// One record's child-order vector. `HtmlConstructionAllocationSnapshot::child_position` gives
    /// that record's ordinal.
    Children,
    /// The temporary decision vector that an attribute edit builds for one target element before
    /// it commits.
    AttributePlan,
    /// The shared arena allocation that publication creates. The report taken before it is
    /// created shows no initialized payload bytes. The snapshot after publication, and the
    /// lease's, reports the size of the arena itself.
    ArenaWrapper,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AllocationRole {
    Identity,
    Records,
    Children(usize),
    AttributePlan(usize),
    ArenaWrapper,
}

/// Identity of an actual owner, bank and allocation generation.
#[derive(Clone, Debug)]
pub struct HtmlConstructionAllocationId {
    identity: Weak<Identity>,
    role: AllocationRole,
    generation: u64,
}
impl PartialEq for HtmlConstructionAllocationId {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.identity, &other.identity)
            && self.role == other.role
            && self.generation == other.generation
    }
}
impl Eq for HtmlConstructionAllocationId {}
impl HtmlConstructionAllocationId {
    /// Compares only issuing storage identity and role, ignoring generation.
    /// This observation cannot authorize retirement or identify an incarnation.
    pub fn same_owner_role(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.identity, &other.identity) && self.role == other.role
    }
}

/// The actual private payload layout, excluding Arc headers and allocator metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlConstructionAllocationSnapshot {
    id: HtmlConstructionAllocationId,
    initialized_len: usize,
    capacity: usize,
    element_size: usize,
    element_alignment: usize,
    payload_bytes: usize,
    wrapper_payload_bytes: usize,
}
impl HtmlConstructionAllocationSnapshot {
    /// The allocation's identity. Two snapshots of the same owner, role and generation compare
    /// equal.
    pub fn id(&self) -> &HtmlConstructionAllocationId {
        &self.id
    }
    /// The allocation's role, without the record ordinal that `child_position` carries.
    pub fn role(&self) -> HtmlConstructionAllocationRole {
        match self.id.role {
            AllocationRole::Identity => HtmlConstructionAllocationRole::Identity,
            AllocationRole::Records => HtmlConstructionAllocationRole::Records,
            AllocationRole::Children(_) => HtmlConstructionAllocationRole::Children,
            AllocationRole::AttributePlan(_) => HtmlConstructionAllocationRole::AttributePlan,
            AllocationRole::ArenaWrapper => HtmlConstructionAllocationRole::ArenaWrapper,
        }
    }
    /// The Children bank's owner-local dense record ordinal, including a
    /// pending uncommitted bank. It grants no node or retirement authority.
    pub fn child_position(&self) -> Option<usize> {
        match self.id.role {
            AllocationRole::Children(position) => Some(position),
            _ => None,
        }
    }
    /// The number of initialized elements in the allocation's vector. The identity allocation, a
    /// single value, reports 1.
    pub fn initialized_len(&self) -> usize {
        self.initialized_len
    }
    /// The element capacity that the allocation reserved. It is at least `initialized_len` and may
    /// exceed it.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// The size in bytes of one element of the allocation.
    pub fn element_size(&self) -> usize {
        self.element_size
    }
    /// The alignment in bytes of one element of the allocation.
    pub fn element_alignment(&self) -> usize {
        self.element_alignment
    }
    /// The allocation's payload in bytes: capacity times element size. It excludes `Arc` headers
    /// and allocator metadata.
    pub fn payload_bytes(&self) -> usize {
        self.payload_bytes
    }
    /// The bytes of the inline wrapper that holds the payload, such as a `Vec` header. It is 0 for
    /// a shared single-value allocation.
    pub fn wrapper_payload_bytes(&self) -> usize {
        self.wrapper_payload_bytes
    }

    fn vector<T>(
        identity: &Arc<Identity>,
        role: AllocationRole,
        generation: u64,
        values: &Vec<T>,
    ) -> Result<Self, HtmlConstructionError> {
        Self::buffer::<T>(
            identity,
            role,
            generation,
            values.len(),
            values.capacity(),
            size_of::<Vec<T>>(),
        )
    }
    /// The allocation of a child-order deque, whose slots are the same words a vector holds.
    fn deque<T>(
        identity: &Arc<Identity>,
        role: AllocationRole,
        generation: u64,
        values: &VecDeque<T>,
    ) -> Result<Self, HtmlConstructionError> {
        Self::buffer::<T>(
            identity,
            role,
            generation,
            values.len(),
            values.capacity(),
            size_of::<VecDeque<T>>(),
        )
    }
    fn buffer<T>(
        identity: &Arc<Identity>,
        role: AllocationRole,
        generation: u64,
        initialized_len: usize,
        capacity: usize,
        wrapper_payload_bytes: usize,
    ) -> Result<Self, HtmlConstructionError> {
        let payload_bytes = capacity
            .checked_mul(size_of::<T>())
            .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
        Ok(Self {
            id: HtmlConstructionAllocationId {
                identity: Arc::downgrade(identity),
                role,
                generation,
            },
            initialized_len,
            capacity,
            element_size: size_of::<T>(),
            element_alignment: align_of::<T>(),
            payload_bytes,
            wrapper_payload_bytes,
        })
    }
    fn shared<T>(identity: &Arc<Identity>, role: AllocationRole, generation: u64) -> Self {
        Self {
            id: HtmlConstructionAllocationId {
                identity: Arc::downgrade(identity),
                role,
                generation,
            },
            initialized_len: 1,
            capacity: 1,
            element_size: size_of::<T>(),
            element_alignment: align_of::<T>(),
            payload_bytes: size_of::<T>(),
            wrapper_payload_bytes: 0,
        }
    }
}

/// Shared immutable output with the same identity and records as its storage.
#[derive(Clone, Debug)]
pub enum HtmlConstructionResult {
    /// A published Document result.
    Document(HtmlDocument),
    /// A published fragment result.
    Fragment(HtmlFragment),
}
impl HtmlConstructionResult {
    /// The Document when this result is one; `None` for a fragment result.
    pub fn document(&self) -> Option<&HtmlDocument> {
        match self {
            Self::Document(v) => Some(v),
            _ => None,
        }
    }
    /// The fragment when this result is one; `None` for a Document result.
    pub fn fragment(&self) -> Option<&HtmlFragment> {
        match self {
            Self::Fragment(v) => Some(v),
            _ => None,
        }
    }
    /// Resolves a result id into this result's node view. Returns `ForeignHtmlResult` when the id
    /// belongs to another construction result or is out of range.
    pub fn node(&self, id: &HtmlNodeId) -> Result<HtmlNode<'_>, ForeignHtmlResult> {
        self.arena().lookup(id)
    }
    /// Reads a retained ordinal; it never interprets an integer as a result identity.
    pub fn node_at(&self, position: usize) -> Option<HtmlConstructionNode<'_>> {
        (position < self.arena().records.len()).then_some(HtmlConstructionNode {
            arena: self.arena(),
            index: position,
        })
    }
    /// The number of retained records, including roots, template contents and shadow contents
    /// fragments.
    pub fn len(&self) -> usize {
        self.arena().records.len()
    }
    /// Whether the result retains no records. It is the exact counterpart of `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.arena().records.is_empty()
    }
    /// Resolves a builder handle issued by this result's construction owner to its retained
    /// ordinal. Returns the foreign-builder error for another owner's handle and the unknown-node
    /// error for an ordinal past the retained records.
    pub fn position(&self, handle: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        if handle.identity.as_ptr() != Arc::as_ptr(&self.arena().identity) {
            return Err(HtmlConstructionErrorKind::ForeignBuilder.into());
        }
        if handle.index >= self.arena().records.len() {
            return Err(HtmlConstructionErrorKind::UnknownNode.into());
        }
        Ok(handle.index)
    }
    /// Resolves a result id into a retained-record view. Returns `ForeignHtmlResult` for an id of
    /// another construction result or one past the retained records.
    pub fn node_by_result_id(
        &self,
        id: &HtmlNodeId,
    ) -> Result<HtmlConstructionNode<'_>, ForeignHtmlResult> {
        if !Arc::ptr_eq(&self.arena().identity, &id.identity)
            || id.index >= self.arena().records.len()
        {
            return Err(ForeignHtmlResult);
        }
        Ok(HtmlConstructionNode {
            arena: self.arena(),
            index: id.index,
        })
    }
    /// The usage that construction admitted for this result.
    pub fn usage(&self) -> HtmlConstructionUsage {
        self.arena().usage
    }
    pub(super) fn arena(&self) -> &Arc<Arena> {
        match self {
            Self::Document(v) => &v.arena,
            Self::Fragment(v) => &v.arena,
        }
    }
    pub(super) fn into_arena(self) -> (Arc<Arena>, Option<usize>) {
        match self {
            Self::Document(v) => (v.arena, None),
            Self::Fragment(v) => (v.arena, Some(v.root)),
        }
    }
}

/// A read-only view qualified by one live storage borrow.
#[derive(Clone, Copy, Debug)]
pub struct HtmlConstructionNode<'a> {
    arena: &'a Arena,
    index: usize,
}
impl<'a> HtmlConstructionNode<'a> {
    /// The retained record ordinal that this view reads, in creation order.
    pub fn position(self) -> usize {
        self.index
    }
    /// Converts to the ordinary node view over the same record.
    pub fn as_node(self) -> HtmlNode<'a> {
        HtmlNode {
            arena: self.arena,
            index: self.index,
        }
    }
    /// Checks only this retained record's generic container invariant.
    pub fn check_container(self) -> Result<(), HtmlConstructionError> {
        if self.arena.records[self.index].template_contents.is_some() {
            return Err(HtmlConstructionErrorKind::TemplateContentsRequired.into());
        }
        if !matches!(
            self.kind(),
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
        ) {
            return Err(HtmlConstructionErrorKind::ParentCannotHaveChildren.into());
        }
        Ok(())
    }
    /// Checks the fixed movable-node invariant, without scanning ancestry or order.
    pub fn check_detachment(self) -> Result<(), HtmlConstructionError> {
        if matches!(self.kind(), HtmlNodeKind::Document(_)) {
            return Err(HtmlConstructionErrorKind::RootMove.into());
        }
        if self.fragment_owner().is_some() {
            return Err(HtmlConstructionErrorKind::AssociationMove.into());
        }
        if matches!(self.kind(), HtmlNodeKind::Fragment) {
            return Err(HtmlConstructionErrorKind::RootMove.into());
        }
        Ok(())
    }
    /// Qualifies an actual retained reference child against the supplied parent.
    pub fn check_reference_child_of(self, parent: Self) -> Result<(), HtmlConstructionError> {
        if !std::ptr::eq(self.arena, parent.arena) {
            return Err(HtmlConstructionErrorKind::ForeignBuilder.into());
        }
        if self.arena.records[self.index].parent != Some(parent.index) {
            return Err(HtmlConstructionErrorKind::WrongSibling.into());
        }
        Ok(())
    }
    /// Issues a builder handle for this record from the owner's identity. The handle is weak and
    /// does not keep the owner alive.
    pub fn handle(self) -> HtmlBuilderNode {
        HtmlBuilderNode {
            identity: Arc::downgrade(&self.arena.identity),
            index: self.index,
        }
    }
    /// The record's node kind.
    pub fn kind(self) -> &'a HtmlNodeKind {
        &self.arena.records[self.index].kind
    }
    /// The number of ordinary children, excluding template contents and shadow contents.
    pub fn child_count(self) -> usize {
        self.arena.records[self.index].children.len()
    }
    /// The child at `ordinal` in document order; `None` at or past `child_count`.
    pub fn child_at(self, ordinal: usize) -> Option<Self> {
        self.arena.records[self.index]
            .children
            .get(ordinal)
            .copied()
            .map(|index| Self {
                arena: self.arena,
                index,
            })
    }
    /// The ordinary parent. Records without one, such as roots and association fragments, return
    /// `None`.
    pub fn parent(self) -> Option<Self> {
        self.arena.records[self.index].parent.map(|index| Self {
            arena: self.arena,
            index,
        })
    }
    /// The ordinary children in document order. The iterator knows its length and runs from both
    /// ends.
    pub fn children(self) -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator + 'a {
        self.arena.records[self.index]
            .children
            .iter()
            .map(move |&index| Self {
                arena: self.arena,
                index,
            })
    }
    /// The contents fragment of a `template` element, whether owned or a shadow-contents
    /// reference. `None` for any other record.
    pub fn template_contents(self) -> Option<Self> {
        self.arena.records[self.index]
            .template_contents
            .map(|value| Self {
                arena: self.arena,
                index: value.position(),
            })
    }
    /// Whether the template's contents fragment is owned or a shadow reference; `None` for a
    /// record without template contents.
    pub fn template_contents_kind(self) -> Option<HtmlTemplateContentsKind> {
        self.arena.records[self.index]
            .template_contents
            .map(TemplateContents::kind)
    }
    /// The declarative shadow root attached to this host element, if any.
    pub fn shadow_root(self) -> Option<HtmlDeclarativeShadowRoot<'a>> {
        HtmlNode {
            arena: self.arena,
            index: self.index,
        }
        .declarative_shadow_root()
    }
    /// The template or shadow host that owns this fragment root. `None` for ordinary records.
    pub fn fragment_owner(self) -> Option<&'a HtmlFragmentOwner> {
        self.arena.records[self.index].owner.as_ref()
    }
    /// Whether this fragment was explicitly constructed as shadow contents.
    pub fn is_shadow_contents(self) -> bool {
        self.shadow_properties().is_some()
    }
    /// The shadow properties of a shadow contents fragment; `None` otherwise.
    pub fn shadow_properties(self) -> Option<HtmlShadowRootProperties> {
        self.arena.records[self.index].shadow_properties
    }
}

/// Exclusive ownership of the actual construction arena, before publication.
#[derive(Debug)]
pub struct HtmlConstructionStorage {
    arena: Arena,
}
impl HtmlConstructionStorage {
    /// Starts allocation-free initialization of a construction owner. Drive the returned value
    /// with `next_action`; no identity exists until its first action runs.
    pub fn begin() -> HtmlConstructionInitialization {
        HtmlConstructionInitialization {
            identity: None,
            phase: InitializationPhase::Pending,
        }
    }
    /// The admitted usage; it is zero for a new owner.
    pub fn usage(&self) -> HtmlConstructionUsage {
        self.arena.usage
    }
    /// The number of records created so far.
    pub fn len(&self) -> usize {
        self.arena.records.len()
    }
    /// Whether no record has been created.
    pub fn is_empty(&self) -> bool {
        self.arena.records.is_empty()
    }
    /// The record at `position`, or `None` past the last created record.
    pub fn node_at(&self, position: usize) -> Option<HtmlConstructionNode<'_>> {
        (position < self.arena.records.len()).then_some(HtmlConstructionNode {
            arena: &self.arena,
            index: position,
        })
    }
    /// Resolves a builder handle to its record view. Returns the foreign-builder error for another
    /// owner's handle and the unknown-node error for an ordinal past the created records.
    pub fn node(
        &self,
        handle: &HtmlBuilderNode,
    ) -> Result<HtmlConstructionNode<'_>, HtmlConstructionError> {
        let index = self.position(handle)?;
        Ok(HtmlConstructionNode {
            arena: &self.arena,
            index,
        })
    }
    /// Resolves a result id to its record view. Returns `ForeignHtmlResult` when the id's identity
    /// is not this owner's or it is past the created records.
    pub fn node_by_result_id(
        &self,
        id: &HtmlNodeId,
    ) -> Result<HtmlConstructionNode<'_>, ForeignHtmlResult> {
        if !Arc::ptr_eq(&self.arena.identity, &id.identity) || id.index >= self.arena.records.len()
        {
            return Err(ForeignHtmlResult);
        }
        Ok(HtmlConstructionNode {
            arena: &self.arena,
            index: id.index,
        })
    }
    /// Resolves a builder handle to its record ordinal. Errors as `node` does.
    pub fn position(&self, handle: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        if handle.identity.as_ptr() != Arc::as_ptr(&self.arena.identity) {
            return Err(HtmlConstructionErrorKind::ForeignBuilder.into());
        }
        if handle.index >= self.arena.records.len() {
            return Err(HtmlConstructionErrorKind::UnknownNode.into());
        }
        Ok(handle.index)
    }
    /// Resolves a builder handle to the result id that the published result will use for the same
    /// record. Errors as `node` does.
    pub fn result_id(&self, handle: &HtmlBuilderNode) -> Result<HtmlNodeId, HtmlConstructionError> {
        Ok(self.arena.id(self.position(handle)?))
    }
    /// Starts an operation for `command` and takes the owner. Operand qualification happens in the
    /// operation's own actions; a rejection hands the owner and command back intact.
    #[expect(clippy::result_large_err)] // Refusal returns the complete unallocated owner and operand.
    pub fn prepare(
        self,
        command: HtmlConstructionCommand,
    ) -> Result<HtmlConstructionOperation, HtmlConstructionRejection> {
        Ok(HtmlConstructionOperation::new(self, command))
    }
    /// Starts the publication of the tree rooted at `root`; equivalent to preparing
    /// `HtmlConstructionCommand::Publish`.
    #[expect(clippy::result_large_err)] // Refusal retains actual storage without allocating a box.
    pub fn prepare_finish(
        self,
        root: HtmlBuilderNode,
    ) -> Result<HtmlConstructionOperation, HtmlConstructionRejection> {
        self.prepare(HtmlConstructionCommand::Publish { root })
    }
    /// Starts deterministic cleanup of the owner's allocations. Cleanup never resumes
    /// construction.
    pub fn retire(self) -> HtmlConstructionRetirement {
        HtmlConstructionRetirement::new(self)
    }
    pub(super) fn native_depth(&self, position: usize) -> usize {
        self.arena.records[position].depth
    }
    // Native append composes creation and insertion before its fresh handle escapes.
    // Only an uncommitted insertion may reach this rollback; previous topology is intact.
    pub(super) fn rollback_native_append(&mut self, len: usize, usage: HtmlConstructionUsage) {
        for index in (len..self.arena.records.len()).rev() {
            if let Some(depth) = self.arena.records.get(index).map(|record| record.depth) {
                self.arena.unpopulate(depth);
            }
        }
        self.arena.records.truncate(len);
        self.arena.usage = usage;
    }
    fn records_snapshot(
        &self,
    ) -> Result<HtmlConstructionAllocationSnapshot, HtmlConstructionError> {
        HtmlConstructionAllocationSnapshot::vector(
            &self.arena.identity,
            AllocationRole::Records,
            self.arena.records_generation,
            &self.arena.records,
        )
    }
    fn children_snapshot(
        &self,
        index: usize,
    ) -> Result<HtmlConstructionAllocationSnapshot, HtmlConstructionError> {
        let record = &self.arena.records[index];
        HtmlConstructionAllocationSnapshot::deque(
            &self.arena.identity,
            AllocationRole::Children(index),
            record.children_generation,
            &record.children,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InitializationPhase {
    Pending,
    Observing,
    Ready,
}
/// Allocation-free initialization; the actual shared identity is created by its action.
#[derive(Debug)]
pub struct HtmlConstructionInitialization {
    identity: Option<Arc<Identity>>,
    phase: InitializationPhase,
}
/// One pending initialization action. It borrows the initialization exclusively; `perform`
/// consumes it.
#[derive(Debug)]
pub struct HtmlConstructionInitializationAction<'a> {
    owner: &'a mut HtmlConstructionInitialization,
}
/// The observation after an initialization action. It borrows the initialization exclusively until
/// it is accepted.
#[derive(Debug)]
pub struct HtmlConstructionInitializationObservation<'a> {
    owner: &'a mut HtmlConstructionInitialization,
    snapshot: HtmlConstructionAllocationSnapshot,
}
/// An initialization that completed identity allocation. `commit` turns it into storage; `abort`
/// retires the identity.
#[derive(Debug)]
pub struct HtmlConstructionReadyInitialization {
    identity: Arc<Identity>,
}
impl HtmlConstructionInitialization {
    /// The next action to run, `Ok(None)` once initialization is ready, or the not-ready error
    /// while an observation awaits `accept`.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionInitializationAction<'_>>, HtmlConstructionError> {
        match self.phase {
            InitializationPhase::Pending => {
                Ok(Some(HtmlConstructionInitializationAction { owner: self }))
            }
            InitializationPhase::Ready => Ok(None),
            InitializationPhase::Observing => {
                Err(HtmlConstructionErrorKind::OperationNotReady.into())
            }
        }
    }
    /// Takes a ready initialization out of `owner`. Returns the consumed error for an empty owner
    /// and the not-ready error when initialization has not completed; on error the owner is left
    /// in place.
    pub fn try_ready(
        owner: &mut Option<Self>,
    ) -> Result<HtmlConstructionReadyInitialization, HtmlConstructionError> {
        let value = owner
            .as_ref()
            .ok_or(HtmlConstructionErrorKind::OperationConsumed)?;
        if value.phase != InitializationPhase::Ready {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
        Ok(HtmlConstructionReadyInitialization {
            identity: owner
                .take()
                .expect("occupied initialization")
                .identity
                .expect("observed real identity"),
        })
    }
    /// Abandons initialization. Returns the retirement evidence for the identity allocation if one
    /// had been made.
    pub fn abort(self) -> Option<HtmlConstructionRetired> {
        self.identity.map(|identity| {
            let snapshot = HtmlConstructionAllocationSnapshot::shared::<Identity>(
                &identity,
                AllocationRole::Identity,
                0,
            );
            drop(identity);
            HtmlConstructionRetired { snapshot }
        })
    }
}
impl<'a> HtmlConstructionInitializationAction<'a> {
    /// Describes the pending action's work for the consumer to admit.
    pub fn description(&self) -> HtmlConstructionActionDescription<'_> {
        HtmlConstructionActionDescription::InitializeIdentity {
            payload_bytes: size_of::<Identity>(),
            initialized_bytes: size_of::<Arena>(),
            provider_calls: 2,
        }
    }
    /// Allocates the shared identity. The observation reports that allocation and must be accepted
    /// before the next action.
    pub fn perform(self) -> HtmlConstructionInitializationObservation<'a> {
        let identity = Arc::new(Identity);
        let snapshot = HtmlConstructionAllocationSnapshot::shared::<Identity>(
            &identity,
            AllocationRole::Identity,
            0,
        );
        self.owner.identity = Some(identity);
        self.owner.phase = InitializationPhase::Observing;
        HtmlConstructionInitializationObservation {
            owner: self.owner,
            snapshot,
        }
    }
}
impl HtmlConstructionInitializationObservation<'_> {
    /// The allocation that the performed action created.
    pub fn allocation(&self) -> &HtmlConstructionAllocationSnapshot {
        &self.snapshot
    }
    /// Accepts the observation, which makes the initialization ready.
    pub fn accept(self) {
        self.owner.phase = InitializationPhase::Ready;
    }
}
impl HtmlConstructionReadyInitialization {
    /// Completes initialization and returns empty construction storage with zero usage.
    pub fn commit(self) -> HtmlConstructionStorage {
        HtmlConstructionStorage {
            arena: Arena {
                identity: self.identity,
                records: Vec::new(),
                usage: HtmlConstructionUsage::default(),
                records_generation: 0,
                wrapper_generation: 0,
                temporary_generation: 0,
            },
        }
    }
    /// Abandons a ready initialization and returns the retirement evidence for its identity
    /// allocation.
    pub fn abort(self) -> Option<HtmlConstructionRetired> {
        HtmlConstructionInitialization {
            identity: Some(self.identity),
            phase: InitializationPhase::Ready,
        }
        .abort()
    }
}

/// Non-cloneable evidence that the indicated actual allocation was retired.
#[derive(Debug)]
pub struct HtmlConstructionRetired {
    snapshot: HtmlConstructionAllocationSnapshot,
}
impl HtmlConstructionRetired {
    /// The allocation snapshot that was retired.
    pub fn allocation(&self) -> &HtmlConstructionAllocationSnapshot {
        &self.snapshot
    }
}

/// Recovery capability for the actual published shared arena.
#[derive(Debug)]
pub struct HtmlConstructionLease {
    arena: Weak<Arena>,
    snapshot: HtmlConstructionAllocationSnapshot,
}
/// Recovered mutable storage together with the retirement evidence for the published allocation.
/// `HtmlConstructionLease::try_recover` produces it.
#[derive(Debug)]
pub struct HtmlConstructionRecovery {
    storage: HtmlConstructionStorage,
    retired: HtmlConstructionRetired,
}
impl HtmlConstructionRecovery {
    /// Splits into the recovered mutable storage and the retirement evidence.
    pub fn into_parts(self) -> (HtmlConstructionStorage, HtmlConstructionRetired) {
        (self.storage, self.retired)
    }
}
impl HtmlConstructionLease {
    /// The snapshot of the published arena allocation, recorded when the lease was issued.
    pub fn allocation(&self) -> &HtmlConstructionAllocationSnapshot {
        &self.snapshot
    }
    /// Takes published storage back into mutable ownership. The result must be the sole strong
    /// owner of its arena. Returns the consumed error if `result` or `lease` is empty, the
    /// foreign-allocation error if the lease belongs to another result, and the shared-allocation
    /// error while any clone of the result lives. On error neither argument is taken.
    pub fn try_recover(
        result: &mut Option<HtmlConstructionResult>,
        lease: &mut Option<Self>,
    ) -> Result<HtmlConstructionRecovery, HtmlConstructionError> {
        let value = result
            .as_ref()
            .ok_or(HtmlConstructionErrorKind::OperationConsumed)?;
        let capability = lease
            .as_ref()
            .ok_or(HtmlConstructionErrorKind::OperationConsumed)?;
        if capability.arena.as_ptr() != Arc::as_ptr(value.arena()) {
            return Err(HtmlConstructionErrorKind::ForeignAllocation.into());
        }
        if Arc::strong_count(value.arena()) != 1 {
            return Err(HtmlConstructionErrorKind::SharedAllocationAliased.into());
        }
        let (arena, _) = result.take().expect("checked result owner").into_arena();
        let capability = lease.take().expect("checked lease owner");
        let arena = Arc::try_unwrap(arena).expect("preflight proved sole strong arena owner");
        Ok(HtmlConstructionRecovery {
            storage: HtmlConstructionStorage { arena },
            retired: HtmlConstructionRetired {
                snapshot: capability.snapshot,
            },
        })
    }
}

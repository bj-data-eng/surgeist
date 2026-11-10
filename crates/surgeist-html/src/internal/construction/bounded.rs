//! The sole mutable owner of shared HTML construction records.

use super::*;
use std::mem::{align_of, size_of};
use std::sync::Weak;

/// A fault of charging a construction step: the step and what the caller says about it disagree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlConstructionChargeFault {
    /// The caller declared more prepaid provider calls than the step describes. The caller's own
    /// charge for the step and the step's plan disagree.
    PrepaidCallsExceedPlan,
    /// The caller asked a description to charge a part it does not have.
    WrongStep,
}
impl fmt::Display for HtmlConstructionChargeFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrepaidCallsExceedPlan => {
                f.write_str("the prepaid provider calls exceed the step's described calls")
            }
            Self::WrongStep => f.write_str("the description has no such part to charge"),
        }
    }
}
impl std::error::Error for HtmlConstructionChargeFault {}

pub(crate) mod attribute_edit;

pub(crate) mod operations;
use operations::{
    HtmlConstructionActionDescription, HtmlConstructionCommand, HtmlConstructionOperation,
    HtmlConstructionRetirement, HtmlConstructionStepWork,
};

pub(crate) mod text_append;

/// Test adapters that keep a protocol owner in an `Option` slot, as a driver that stores it in a
/// field does, and put the owner back when its consuming step refuses it.
#[cfg(test)]
mod slots {
    use super::attribute_edit::*;
    use super::operations::HtmlConstructionReadyOperation;
    use super::text_append::*;
    use super::*;

    fn step<T, R>(
        slot: &mut Option<T>,
        consume: impl FnOnce(T) -> Result<R, HtmlConstructionRefusal<T>>,
    ) -> Result<R, HtmlConstructionError> {
        let owner = slot.take().expect("slot holds its owner");
        match consume(owner) {
            Ok(value) => Ok(value),
            Err(refusal) => {
                let (owner, reason) = refusal.into_parts();
                *slot = Some(owner);
                Err(reason)
            }
        }
    }
    pub(super) fn initialization(
        slot: &mut Option<HtmlConstructionInitialization>,
    ) -> Result<HtmlConstructionReadyInitialization, HtmlConstructionError> {
        step(slot, HtmlConstructionInitialization::try_ready)
    }
    pub(super) fn operation(
        slot: &mut Option<HtmlConstructionOperation>,
    ) -> Result<HtmlConstructionReadyOperation, HtmlConstructionError> {
        step(slot, HtmlConstructionOperation::try_ready)
    }
    pub(super) fn text_append(
        slot: &mut Option<HtmlConstructionTextAppend>,
    ) -> Result<HtmlConstructionTextAppendReady, HtmlConstructionError> {
        step(slot, HtmlConstructionTextAppend::try_ready)
    }
    pub(super) fn attribute_slot<'e>(
        slot: &mut Option<HtmlConstructionAttributeSlot<'e>>,
    ) -> Result<HtmlConstructionReadyAttributeSlot<'e>, HtmlConstructionError> {
        step(slot, HtmlConstructionAttributeSlot::try_ready)
    }
    pub(super) fn candidate(
        slot: &mut Option<HtmlConstructionAttributeCandidate<'_, '_>>,
    ) -> Result<bool, HtmlConstructionError> {
        step(slot, HtmlConstructionAttributeCandidate::try_finish)
    }
    pub(super) fn edit(
        slot: &mut Option<HtmlConstructionAttributeEdit>,
    ) -> Result<HtmlConstructionAttributeCompletion, HtmlConstructionError> {
        step(slot, HtmlConstructionAttributeEdit::try_finish)
    }
    pub(super) fn recover(
        result: &mut Option<HtmlConstructionResult>,
        lease: &mut Option<HtmlConstructionLease>,
    ) -> Result<HtmlConstructionRecovery, HtmlConstructionError> {
        let owners = (
            result.take().expect("slot holds its result"),
            lease.take().expect("slot holds its lease"),
        );
        match HtmlConstructionLease::try_recover(owners.0, owners.1) {
            Ok(value) => Ok(value),
            Err(refusal) => {
                let ((returned_result, returned_lease), reason) = refusal.into_parts();
                *result = Some(returned_result);
                *lease = Some(returned_lease);
                Err(reason)
            }
        }
    }
}

/// One hidden allocation's role in a construction owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlConstructionAllocationRole {
    /// The shared identity allocation. Builder node handles and allocation ids refer to it
    /// weakly, while published node ids and the arena hold it strongly, so it distinguishes one
    /// construction owner from another.
    Identity,
    /// The vector of retained node records.
    Records,
    /// One record's child-order deque. `HtmlConstructionAllocationSnapshot::child_position` gives
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
    /// The bytes of the inline wrapper that holds the payload, such as a `Vec` header, or a
    /// `VecDeque` header for a child-order bank. It is 0 for a shared single-value allocation.
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
            .ok_or(HtmlConstructionCause::RepresentationExhausted)?;
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
        self.arena().record_at(position)
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
        Ok(self.index(handle)?.position())
    }
    /// The record index of a builder handle, with the errors of `position`.
    pub(super) fn index(&self, handle: &HtmlBuilderNode) -> Result<NodeIx, HtmlConstructionError> {
        self.arena().index(handle)
    }
    /// Resolves a result id into a retained-record view. Returns `ForeignHtmlResult` for an id of
    /// another construction result or one past the retained records.
    pub fn node_by_result_id(
        &self,
        id: &HtmlNodeId,
    ) -> Result<HtmlConstructionNode<'_>, ForeignHtmlResult> {
        self.arena().construction_node(id)
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
    /// The sole owner's arena, or the unchanged result while another strong owner lives.
    fn try_unwrap_arena(self) -> Result<Arena, Self> {
        match self {
            Self::Document(v) => Arc::try_unwrap(v.arena).map_err(|arena| {
                Self::Document(HtmlDocument {
                    arena,
                    root: v.root,
                    mode: v.mode,
                })
            }),
            Self::Fragment(v) => Arc::try_unwrap(v.arena).map_err(|arena| {
                Self::Fragment(HtmlFragment {
                    arena,
                    root: v.root,
                })
            }),
        }
    }
}

/// A read-only view qualified by one live storage borrow.
#[derive(Clone, Copy, Debug)]
pub struct HtmlConstructionNode<'a> {
    pub(super) arena: &'a Arena,
    pub(super) index: NodeIx,
}
impl<'a> HtmlConstructionNode<'a> {
    /// The retained record ordinal that this view reads, in creation order.
    pub fn position(self) -> usize {
        self.index.position()
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
        if self.arena.records[self.index.position()]
            .template_contents
            .is_some()
        {
            return Err(HtmlConstructionCause::TemplateContentsRequired.into());
        }
        if !matches!(
            self.kind(),
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
        ) {
            return Err(HtmlConstructionCause::ParentCannotHaveChildren.into());
        }
        Ok(())
    }
    /// Checks the fixed movable-node invariant, without scanning ancestry or order.
    pub fn check_detachment(self) -> Result<(), HtmlConstructionError> {
        if matches!(self.kind(), HtmlNodeKind::Document(_)) {
            return Err(HtmlConstructionCause::RootMove.into());
        }
        if self.fragment_owner().is_some() {
            return Err(HtmlConstructionCause::AssociationMove.into());
        }
        if matches!(self.kind(), HtmlNodeKind::Fragment) {
            return Err(HtmlConstructionCause::RootMove.into());
        }
        Ok(())
    }
    /// Qualifies an actual retained reference child against the supplied parent.
    pub fn check_reference_child_of(self, parent: Self) -> Result<(), HtmlConstructionError> {
        if !std::ptr::eq(self.arena, parent.arena) {
            return Err(HtmlConstructionCause::ForeignBuilder.into());
        }
        if self.arena.records[self.index.position()].parent != Some(parent.index) {
            return Err(HtmlConstructionCause::WrongSibling.into());
        }
        Ok(())
    }
    /// Issues a builder handle for this record from the owner's serial. The handle holds no
    /// reference to the owner.
    pub fn handle(self) -> HtmlBuilderNode {
        HtmlBuilderNode {
            tree: self.arena.serial,
            index: self.index,
        }
    }
    /// The record's node kind.
    pub fn kind(self) -> &'a HtmlNodeKind {
        &self.arena.records[self.index.position()].kind
    }
    /// The number of ordinary children, excluding template contents and shadow contents.
    pub fn child_count(self) -> usize {
        self.arena.records[self.index.position()].children.len()
    }
    /// The child at `ordinal` in document order; `None` at or past `child_count`.
    pub fn child_at(self, ordinal: usize) -> Option<Self> {
        self.arena.records[self.index.position()]
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
        self.arena.records[self.index.position()]
            .parent
            .map(|index| Self {
                arena: self.arena,
                index,
            })
    }
    /// The ordinary children in document order. The iterator knows its length and runs from both
    /// ends.
    pub fn children(self) -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator + 'a {
        self.arena.records[self.index.position()]
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
        self.arena.records[self.index.position()]
            .template_contents
            .map(|value| Self {
                arena: self.arena,
                index: value.position(),
            })
    }
    /// Whether the template's contents fragment is owned or a shadow reference; `None` for a
    /// record without template contents.
    pub fn template_contents_kind(self) -> Option<HtmlTemplateContentsKind> {
        self.arena.records[self.index.position()]
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
        self.arena.records[self.index.position()].owner.as_ref()
    }
    /// Whether this fragment was explicitly constructed as shadow contents.
    pub fn is_shadow_contents(self) -> bool {
        self.shadow_properties().is_some()
    }
    /// The shadow properties of a shadow contents fragment; `None` otherwise.
    pub fn shadow_properties(self) -> Option<HtmlShadowRootProperties> {
        self.arena.records[self.index.position()].shadow_properties
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
        HtmlConstructionInitialization { created: None }
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
        self.arena.record_at(position)
    }
    /// Resolves a builder handle to its record view. Returns the foreign-builder error for another
    /// owner's handle and the unknown-node error for an ordinal past the created records.
    pub fn node(
        &self,
        handle: &HtmlBuilderNode,
    ) -> Result<HtmlConstructionNode<'_>, HtmlConstructionError> {
        let index = self.index(handle)?;
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
        self.arena.construction_node(id)
    }
    /// Resolves a builder handle to its record ordinal. Errors as `node` does.
    pub fn position(&self, handle: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        Ok(self.index(handle)?.position())
    }
    /// Resolves a builder handle to its record index. Errors as `node` does.
    pub(super) fn index(&self, handle: &HtmlBuilderNode) -> Result<NodeIx, HtmlConstructionError> {
        self.arena.index(handle)
    }
    /// Resolves a builder handle to the result id that the published result will use for the same
    /// record. Errors as `node` does.
    pub fn result_id(&self, handle: &HtmlBuilderNode) -> Result<HtmlNodeId, HtmlConstructionError> {
        Ok(self.arena.id(self.index(handle)?))
    }
    /// Starts an operation for `command` and takes the owner. Preparation allocates and inspects
    /// nothing, so it cannot fail: operand qualification happens in the operation's own actions,
    /// and `HtmlConstructionOperation::abort` hands the owner and command back intact.
    pub fn prepare(self, command: HtmlConstructionCommand) -> HtmlConstructionOperation {
        HtmlConstructionOperation::new(self, command)
    }
    /// Starts the publication of the tree rooted at `root`; equivalent to preparing
    /// `HtmlConstructionCommand::Publish`.
    pub fn prepare_finish(self, root: HtmlBuilderNode) -> HtmlConstructionOperation {
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
        // A later record may take an ordinal given up here, so a table of attribute names that
        // described one of them must not be trusted.
        self.arena.attribute_revision = self.arena.attribute_revision.wrapping_add(1);
        self.arena.usage = usage;
    }
    /// The capacity of the record bank.
    pub(super) fn records_capacity(&self) -> usize {
        self.arena.records.capacity()
    }
    /// The capacity of the child order of the record at `index`.
    #[cfg(test)]
    pub(super) fn children_capacity(&self, index: usize) -> Option<usize> {
        self.arena
            .records
            .get(index)
            .map(|record| record.children.capacity())
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

/// The identity allocation once an initialization action has created it, and whether its
/// observation was accepted.
#[derive(Debug)]
struct CreatedIdentity {
    identity: Arc<Identity>,
    accepted: bool,
}
/// Allocation-free initialization; the actual shared identity is created by its action.
#[derive(Debug)]
pub struct HtmlConstructionInitialization {
    /// `None` until the action runs: no identity exists before it.
    created: Option<CreatedIdentity>,
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
    accepted: &'a mut bool,
    snapshot: HtmlConstructionAllocationSnapshot,
}
/// An initialization that completed identity allocation. `commit` turns it into storage; `abort`
/// retires the identity.
#[derive(Debug)]
pub struct HtmlConstructionReadyInitialization {
    identity: Arc<Identity>,
    serial: u64,
}
impl HtmlConstructionInitialization {
    /// The next action to run, `Ok(None)` once initialization is ready, or the not-ready error
    /// while an observation awaits `accept`.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionInitializationAction<'_>>, HtmlConstructionError> {
        match self.created {
            None => Ok(Some(HtmlConstructionInitializationAction { owner: self })),
            Some(CreatedIdentity { accepted: true, .. }) => Ok(None),
            Some(CreatedIdentity {
                accepted: false, ..
            }) => Err(HtmlConstructionCause::OperationNotReady.into()),
        }
    }
    /// Consumes a ready initialization and issues the storage's process-wide serial. Returns the
    /// not-ready error, with the initialization handed back unchanged, when initialization has not
    /// completed, and `RepresentationExhausted` when no serial is left.
    pub fn try_ready(
        self,
    ) -> Result<HtmlConstructionReadyInitialization, HtmlConstructionRefusal<Self>> {
        self.try_ready_from(&NEXT_STORAGE_SERIAL)
    }
    /// `try_ready` drawing the serial from `counter`.
    fn try_ready_from(
        self,
        counter: &std::sync::atomic::AtomicU64,
    ) -> Result<HtmlConstructionReadyInitialization, HtmlConstructionRefusal<Self>> {
        match self.created {
            Some(CreatedIdentity {
                identity,
                accepted: true,
            }) => match issue_storage_serial(counter) {
                Ok(serial) => Ok(HtmlConstructionReadyInitialization { identity, serial }),
                Err(cause) => Err(HtmlConstructionRefusal::new(
                    Self {
                        created: Some(CreatedIdentity {
                            identity,
                            accepted: true,
                        }),
                    },
                    cause,
                )),
            },
            created => Err(HtmlConstructionRefusal::new(
                Self { created },
                HtmlConstructionCause::OperationNotReady,
            )),
        }
    }
    /// Abandons initialization. Returns the retirement evidence for the identity allocation if one
    /// had been made.
    pub fn abort(self) -> Option<HtmlConstructionRetired> {
        self.created
            .map(|created| retire_identity(created.identity))
    }
}
fn retire_identity(identity: Arc<Identity>) -> HtmlConstructionRetired {
    let snapshot = HtmlConstructionAllocationSnapshot::shared::<Identity>(
        &identity,
        AllocationRole::Identity,
        0,
    );
    drop(identity);
    HtmlConstructionRetired { snapshot }
}
impl<'a> HtmlConstructionInitializationAction<'a> {
    /// Describes the pending action's work for the consumer to admit.
    pub fn description(&self) -> HtmlConstructionActionDescription<'_> {
        HtmlConstructionActionDescription::InitializeIdentity {
            payload_bytes: size_of::<Identity>(),
            work: HtmlConstructionStepWork {
                initialized_bytes: size_of::<Arena>(),
                provider_calls: 2,
                ..HtmlConstructionStepWork::default()
            },
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
        let created = self.owner.created.insert(CreatedIdentity {
            identity,
            accepted: false,
        });
        HtmlConstructionInitializationObservation {
            accepted: &mut created.accepted,
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
        *self.accepted = true;
    }
}
/// The serials issued to construction storages. A serial names its storage among all storages in
/// the process: node ids and builder handles carry it, and no two storages share one.
static NEXT_STORAGE_SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
/// Takes the next serial from `counter`, or `RepresentationExhausted` when none is left. The
/// counter keeps its value on refusal, so a serial is never reissued.
fn issue_storage_serial(
    counter: &std::sync::atomic::AtomicU64,
) -> Result<u64, HtmlConstructionCause> {
    use std::sync::atomic::Ordering;
    counter
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
            next.checked_add(1)
        })
        .map_err(|_| HtmlConstructionCause::RepresentationExhausted)
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
                serial: self.serial,
                attribute_revision: 0,
            },
        }
    }
    /// Abandons a ready initialization and returns the retirement evidence for its identity
    /// allocation.
    pub fn abort(self) -> HtmlConstructionRetired {
        retire_identity(self.identity)
    }
}

/// A consuming step that its owner refused. The refusal hands the owner back unchanged together
/// with the cause, so the caller can abort, retry after the missing action, or recover.
#[derive(Debug)]
pub struct HtmlConstructionRefusal<T> {
    owner: T,
    reason: HtmlConstructionError,
}
impl<T> HtmlConstructionRefusal<T> {
    pub(super) fn new(owner: T, reason: impl Into<HtmlConstructionError>) -> Self {
        Self {
            owner,
            reason: reason.into(),
        }
    }
    /// The cause the step was refused for.
    pub fn reason(&self) -> &HtmlConstructionError {
        &self.reason
    }
    /// The owner, unchanged.
    pub fn owner(&self) -> &T {
        &self.owner
    }
    /// Returns the unchanged owner and the cause. Nothing the caller supplied is lost.
    pub fn into_parts(self) -> (T, HtmlConstructionError) {
        (self.owner, self.reason)
    }
}
impl<T> fmt::Display for HtmlConstructionRefusal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "construction step refused: {}", self.reason)
    }
}
impl<T: fmt::Debug> std::error::Error for HtmlConstructionRefusal<T> {}

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
    /// owner of its arena. Returns the foreign-allocation error if the lease belongs to another
    /// result and the shared-allocation error while any clone of the result lives. On error both
    /// arguments are handed back unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn try_recover(
        result: HtmlConstructionResult,
        lease: Self,
    ) -> Result<HtmlConstructionRecovery, HtmlConstructionRefusal<(HtmlConstructionResult, Self)>>
    {
        if lease.arena.as_ptr() != Arc::as_ptr(result.arena()) {
            return Err(HtmlConstructionRefusal::new(
                (result, lease),
                HtmlConstructionCause::ForeignAllocation,
            ));
        }
        match result.try_unwrap_arena() {
            Ok(arena) => Ok(HtmlConstructionRecovery {
                storage: HtmlConstructionStorage { arena },
                retired: HtmlConstructionRetired {
                    snapshot: lease.snapshot,
                },
            }),
            Err(result) => Err(HtmlConstructionRefusal::new(
                (result, lease),
                HtmlConstructionCause::SharedAllocationAliased,
            )),
        }
    }
}

#[cfg(test)]
mod serial_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn storage_serials_are_issued_in_order_and_exhaustion_is_a_typed_refusal() {
        let counter = AtomicU64::new(u64::MAX - 2);
        assert_eq!(issue_storage_serial(&counter), Ok(u64::MAX - 2));
        assert_eq!(issue_storage_serial(&counter), Ok(u64::MAX - 1));
        // The last value cannot be followed by another, so it is never issued: the counter would
        // have to wrap onto a serial an earlier storage may still hold.
        let refusal = issue_storage_serial(&counter);
        assert_eq!(refusal, Err(HtmlConstructionCause::RepresentationExhausted));
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
        // The refusal repeats and the counter keeps its value.
        assert_eq!(
            issue_storage_serial(&counter),
            Err(HtmlConstructionCause::RepresentationExhausted)
        );
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }

    /// An initialization that has performed and accepted its identity action.
    fn performed_initialization() -> HtmlConstructionInitialization {
        let mut initialization = HtmlConstructionStorage::begin();
        while let Ok(Some(action)) = initialization.next_action() {
            action.perform().accept();
        }
        initialization
    }

    #[test]
    fn a_refused_ready_transition_hands_back_the_initialization_whose_identity_retires() {
        let counter = AtomicU64::new(u64::MAX);
        let Err(refusal) = performed_initialization().try_ready_from(&counter) else {
            panic!("an exhausted counter issues no serial");
        };
        assert_eq!(
            refusal.reason().cause(),
            HtmlConstructionCause::RepresentationExhausted
        );
        let (mut initialization, _) = refusal.into_parts();
        // The initialization is as it was: complete, holding its identity allocation.
        assert!(matches!(initialization.next_action(), Ok(None)));
        let retired = initialization
            .abort()
            .expect("the refused initialization still owns its identity allocation");
        assert_eq!(
            retired.allocation().role(),
            HtmlConstructionAllocationRole::Identity
        );
        // A counter with a value left readies the same kind of initialization.
        let counter = AtomicU64::new(7);
        let ready = performed_initialization()
            .try_ready_from(&counter)
            .expect("a counter with a value left issues a serial");
        assert_eq!(ready.commit().arena.serial, 7);
    }

    #[test]
    fn two_storages_never_share_a_serial() {
        let storages: Vec<_> = (0..4)
            .map(|_| {
                performed_initialization()
                    .try_ready()
                    .expect("a performed initialization is ready")
                    .commit()
            })
            .collect();
        for (index, storage) in storages.iter().enumerate() {
            for other in &storages[index + 1..] {
                assert_ne!(storage.arena.serial, other.arena.serial);
            }
        }
    }
}

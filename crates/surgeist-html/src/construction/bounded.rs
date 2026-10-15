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
    Identity,
    Records,
    Children,
    PendingChildOrder { slot: usize, parent_position: usize },
    AttributePlan,
    ArenaWrapper,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AllocationRole {
    Identity,
    Records,
    Children(usize),
    PendingChildOrder { slot: usize, parent: usize },
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
    pub fn id(&self) -> &HtmlConstructionAllocationId {
        &self.id
    }
    pub fn role(&self) -> HtmlConstructionAllocationRole {
        match self.id.role {
            AllocationRole::Identity => HtmlConstructionAllocationRole::Identity,
            AllocationRole::Records => HtmlConstructionAllocationRole::Records,
            AllocationRole::Children(_) => HtmlConstructionAllocationRole::Children,
            AllocationRole::PendingChildOrder { slot, parent } => {
                HtmlConstructionAllocationRole::PendingChildOrder {
                    slot,
                    parent_position: parent,
                }
            }
            AllocationRole::AttributePlan(_) => HtmlConstructionAllocationRole::AttributePlan,
            AllocationRole::ArenaWrapper => HtmlConstructionAllocationRole::ArenaWrapper,
        }
    }
    /// Actual bounded pending-order bank slot; it grants no release authority.
    pub fn pending_child_order_slot(&self) -> Option<usize> {
        match self.id.role {
            AllocationRole::PendingChildOrder { slot, .. } => Some(slot),
            _ => None,
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
    pub fn initialized_len(&self) -> usize {
        self.initialized_len
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn element_size(&self) -> usize {
        self.element_size
    }
    pub fn element_alignment(&self) -> usize {
        self.element_alignment
    }
    pub fn payload_bytes(&self) -> usize {
        self.payload_bytes
    }
    pub fn wrapper_payload_bytes(&self) -> usize {
        self.wrapper_payload_bytes
    }

    fn vector<T>(
        identity: &Arc<Identity>,
        role: AllocationRole,
        generation: u64,
        values: &Vec<T>,
    ) -> Result<Self, HtmlConstructionError> {
        let payload_bytes = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(HtmlConstructionError::RepresentationExhausted)?;
        Ok(Self {
            id: HtmlConstructionAllocationId {
                identity: Arc::downgrade(identity),
                role,
                generation,
            },
            initialized_len: values.len(),
            capacity: values.capacity(),
            element_size: size_of::<T>(),
            element_alignment: align_of::<T>(),
            payload_bytes,
            wrapper_payload_bytes: size_of::<Vec<T>>(),
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
    Document(HtmlDocument),
    Fragment(HtmlFragment),
}
impl HtmlConstructionResult {
    pub fn document(&self) -> Option<&HtmlDocument> {
        match self {
            Self::Document(v) => Some(v),
            _ => None,
        }
    }
    pub fn fragment(&self) -> Option<&HtmlFragment> {
        match self {
            Self::Fragment(v) => Some(v),
            _ => None,
        }
    }
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
    pub fn len(&self) -> usize {
        self.arena().records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.arena().records.is_empty()
    }
    pub fn position(&self, handle: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        if handle.identity.as_ptr() != Arc::as_ptr(&self.arena().identity) {
            return Err(HtmlConstructionError::ForeignBuilder);
        }
        if handle.index >= self.arena().records.len() {
            return Err(HtmlConstructionError::UnknownNode);
        }
        Ok(handle.index)
    }
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
    pub fn position(self) -> usize {
        self.index
    }
    pub fn as_node(self) -> HtmlNode<'a> {
        HtmlNode {
            arena: self.arena,
            index: self.index,
        }
    }
    /// Checks only this retained record's generic container invariant.
    pub fn check_container(self) -> Result<(), HtmlConstructionError> {
        if self.arena.records[self.index].template_contents.is_some() {
            return Err(HtmlConstructionError::TemplateContentsRequired);
        }
        if !matches!(
            self.kind(),
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
        ) {
            return Err(HtmlConstructionError::ParentCannotHaveChildren);
        }
        Ok(())
    }
    /// Checks the fixed movable-node invariant, without scanning ancestry or order.
    pub fn check_detachment(self) -> Result<(), HtmlConstructionError> {
        if matches!(self.kind(), HtmlNodeKind::Document(_)) {
            return Err(HtmlConstructionError::RootMove);
        }
        if self.fragment_owner().is_some() {
            return Err(HtmlConstructionError::AssociationMove);
        }
        if matches!(self.kind(), HtmlNodeKind::Fragment) {
            return Err(HtmlConstructionError::RootMove);
        }
        Ok(())
    }
    /// Qualifies an actual retained reference child against the supplied parent.
    pub fn check_reference_child_of(self, parent: Self) -> Result<(), HtmlConstructionError> {
        if !std::ptr::eq(self.arena, parent.arena) {
            return Err(HtmlConstructionError::ForeignBuilder);
        }
        if self.arena.records[self.index].parent != Some(parent.index) {
            return Err(HtmlConstructionError::WrongSibling);
        }
        Ok(())
    }
    pub fn handle(self) -> HtmlBuilderNode {
        HtmlBuilderNode {
            identity: Arc::downgrade(&self.arena.identity),
            index: self.index,
        }
    }
    pub fn kind(self) -> &'a HtmlNodeKind {
        &self.arena.records[self.index].kind
    }
    pub fn child_count(self) -> usize {
        self.arena.records[self.index].children.len()
    }
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
    pub fn parent(self) -> Option<Self> {
        self.arena.records[self.index].parent.map(|index| Self {
            arena: self.arena,
            index,
        })
    }
    pub fn children(self) -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator + 'a {
        self.arena.records[self.index]
            .children
            .iter()
            .map(move |&index| Self {
                arena: self.arena,
                index,
            })
    }
    pub fn template_contents(self) -> Option<Self> {
        self.arena.records[self.index]
            .template_contents
            .map(|value| Self {
                arena: self.arena,
                index: value.position(),
            })
    }
    pub fn template_contents_kind(self) -> Option<HtmlTemplateContentsKind> {
        self.arena.records[self.index]
            .template_contents
            .map(TemplateContents::kind)
    }
    pub fn shadow_root(self) -> Option<HtmlDeclarativeShadowRoot<'a>> {
        HtmlNode {
            arena: self.arena,
            index: self.index,
        }
        .declarative_shadow_root()
    }
    pub fn fragment_owner(self) -> Option<&'a HtmlFragmentOwner> {
        self.arena.records[self.index].owner.as_ref()
    }
    pub fn is_shadow_contents(self) -> bool {
        self.shadow_properties().is_some()
    }
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
    pub fn begin() -> HtmlConstructionInitialization {
        HtmlConstructionInitialization {
            identity: None,
            phase: InitializationPhase::Pending,
        }
    }
    pub fn usage(&self) -> HtmlConstructionUsage {
        self.arena.usage
    }
    pub fn len(&self) -> usize {
        self.arena.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.arena.records.is_empty()
    }
    pub fn node_at(&self, position: usize) -> Option<HtmlConstructionNode<'_>> {
        (position < self.arena.records.len()).then_some(HtmlConstructionNode {
            arena: &self.arena,
            index: position,
        })
    }
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
    pub fn position(&self, handle: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        if handle.identity.as_ptr() != Arc::as_ptr(&self.arena.identity) {
            return Err(HtmlConstructionError::ForeignBuilder);
        }
        if handle.index >= self.arena.records.len() {
            return Err(HtmlConstructionError::UnknownNode);
        }
        Ok(handle.index)
    }
    pub fn result_id(&self, handle: &HtmlBuilderNode) -> Result<HtmlNodeId, HtmlConstructionError> {
        Ok(self.arena.id(self.position(handle)?))
    }
    #[expect(clippy::result_large_err)] // Refusal returns the complete unallocated owner and operand.
    pub fn prepare(
        self,
        command: HtmlConstructionCommand,
    ) -> Result<HtmlConstructionOperation, HtmlConstructionRejection> {
        Ok(HtmlConstructionOperation::new(self, command))
    }
    #[expect(clippy::result_large_err)] // Refusal retains actual storage without allocating a box.
    pub fn prepare_finish(
        self,
        root: HtmlBuilderNode,
    ) -> Result<HtmlConstructionOperation, HtmlConstructionRejection> {
        self.prepare(HtmlConstructionCommand::Publish { root })
    }
    pub fn retire(self) -> HtmlConstructionRetirement {
        HtmlConstructionRetirement::new(self)
    }
    pub(super) fn native_depth(&self, position: usize) -> usize {
        assert!(
            self.arena.depths_valid,
            "native operations preserve their depth cache"
        );
        self.arena.records[position].depth
    }
    // Native append composes creation and insertion before its fresh handle escapes.
    // Only an uncommitted insertion may reach this rollback; previous topology is intact.
    pub(super) fn rollback_native_append(&mut self, len: usize, usage: HtmlConstructionUsage) {
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
        HtmlConstructionAllocationSnapshot::vector(
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
pub struct HtmlConstructionInitializationAction<'a> {
    owner: &'a mut HtmlConstructionInitialization,
}
pub struct HtmlConstructionInitializationObservation<'a> {
    owner: &'a mut HtmlConstructionInitialization,
    snapshot: HtmlConstructionAllocationSnapshot,
}
#[derive(Debug)]
pub struct HtmlConstructionReadyInitialization {
    identity: Arc<Identity>,
}
impl HtmlConstructionInitialization {
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionInitializationAction<'_>>, HtmlConstructionError> {
        match self.phase {
            InitializationPhase::Pending => {
                Ok(Some(HtmlConstructionInitializationAction { owner: self }))
            }
            InitializationPhase::Ready => Ok(None),
            InitializationPhase::Observing => Err(HtmlConstructionError::OperationNotReady),
        }
    }
    pub fn try_ready(
        owner: &mut Option<Self>,
    ) -> Result<HtmlConstructionReadyInitialization, HtmlConstructionError> {
        let value = owner
            .as_ref()
            .ok_or(HtmlConstructionError::OperationConsumed)?;
        if value.phase != InitializationPhase::Ready {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok(HtmlConstructionReadyInitialization {
            identity: owner
                .take()
                .expect("occupied initialization")
                .identity
                .expect("observed real identity"),
        })
    }
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
    pub fn description(&self) -> HtmlConstructionActionDescription<'_> {
        HtmlConstructionActionDescription::InitializeIdentity {
            payload_bytes: size_of::<Identity>(),
            initialized_bytes: size_of::<Arena>(),
            provider_calls: 2,
        }
    }
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
    pub fn allocation(&self) -> &HtmlConstructionAllocationSnapshot {
        &self.snapshot
    }
    pub fn accept(self) {
        self.owner.phase = InitializationPhase::Ready;
    }
}
impl HtmlConstructionReadyInitialization {
    pub fn commit(self) -> HtmlConstructionStorage {
        HtmlConstructionStorage {
            arena: Arena {
                identity: self.identity,
                records: Vec::new(),
                usage: HtmlConstructionUsage::default(),
                records_generation: 0,
                wrapper_generation: 0,
                depths_valid: true,
                temporary_generation: 0,
            },
        }
    }
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
#[derive(Debug)]
pub struct HtmlConstructionRecovery {
    storage: HtmlConstructionStorage,
    retired: HtmlConstructionRetired,
}
impl HtmlConstructionRecovery {
    pub fn into_parts(self) -> (HtmlConstructionStorage, HtmlConstructionRetired) {
        (self.storage, self.retired)
    }
}
impl HtmlConstructionLease {
    pub fn allocation(&self) -> &HtmlConstructionAllocationSnapshot {
        &self.snapshot
    }
    pub fn try_recover(
        result: &mut Option<HtmlConstructionResult>,
        lease: &mut Option<Self>,
    ) -> Result<HtmlConstructionRecovery, HtmlConstructionError> {
        let value = result
            .as_ref()
            .ok_or(HtmlConstructionError::OperationConsumed)?;
        let capability = lease
            .as_ref()
            .ok_or(HtmlConstructionError::OperationConsumed)?;
        if capability.arena.as_ptr() != Arc::as_ptr(value.arena()) {
            return Err(HtmlConstructionError::ForeignAllocation);
        }
        if Arc::strong_count(value.arena()) != 1 {
            return Err(HtmlConstructionError::SharedAllocationAliased);
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

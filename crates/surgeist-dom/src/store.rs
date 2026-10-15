use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::batch::{BatchIdentity, Command, Creation};
use crate::{
    CreatedNode, DocumentId, DocumentString, DomBatch, DomNodeKind, DomOwnerId, DomRevision,
    DomSnapshot, DomStamp, NodeId, NodeOperand,
};

/// Configured capacity whose admission would be exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomResource {
    ManagedNodes,
    StringCodeUnits,
    Commands,
    HistoryEntries,
    HistoryBytes,
}

/// Typed rejection; no rejected batch publishes any state or identity.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomError {
    ForeignOwner,
    StaleNode,
    ForeignBatch,
    UnavailableCreation,
    WrongNodeKind,
    WrongDocument,
    HierarchyRequest,
    NotFound,
    InvalidDestruction,
    CrossDocumentInsertionUnsupported,
    IndexSize,
    Cancelled,
    ResourceLimit { resource: DomResource, limit: usize },
    InvalidLimit(DomResource),
    AllocationFailed,
    RepresentationExhausted,
    OwnerIdentityExhausted,
    RevisionExhausted,
}
impl std::fmt::Display for DomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DOM operation rejected: {self:?}")
    }
}
impl std::error::Error for DomError {}

/// Explicit finite admission policy. No hidden or unlimited default is selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomLimits {
    nodes: usize,
    units: usize,
    commands: usize,
    history_entries: usize,
    history_bytes: usize,
}
impl DomLimits {
    pub fn new(
        nodes: usize,
        units: usize,
        commands: usize,
        history_entries: usize,
        history_bytes: usize,
    ) -> Result<Self, DomError> {
        for (value, resource, width) in [
            (nodes, DomResource::ManagedNodes, size_of::<Slot>()),
            (units, DomResource::StringCodeUnits, size_of::<u16>()),
            (commands, DomResource::Commands, size_of::<Command>()),
            (
                history_entries,
                DomResource::HistoryEntries,
                size_of::<HistoryEntry>(),
            ),
            (history_bytes, DomResource::HistoryBytes, 1),
        ] {
            if value > isize::MAX as usize / width {
                return Err(DomError::InvalidLimit(resource));
            }
        }
        Ok(Self {
            nodes,
            units,
            commands,
            history_entries,
            history_bytes,
        })
    }
    pub fn managed_nodes(self) -> usize {
        self.nodes
    }
    pub fn string_code_units(self) -> usize {
        self.units
    }
    pub fn commands_per_batch(self) -> usize {
        self.commands
    }
    pub fn history_entries(self) -> usize {
        self.history_entries
    }
    /// Retained entry records plus exact change-array payload bytes. Allocator
    /// bookkeeping and caller-held reports/snapshots are outside this budget.
    pub fn history_bytes(self) -> usize {
        self.history_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomUsage {
    nodes: usize,
    units: usize,
    history_entries: usize,
    history_bytes: usize,
}
impl DomUsage {
    pub fn managed_nodes(self) -> usize {
        self.nodes
    }
    pub fn string_code_units(self) -> usize {
        self.units
    }
    pub fn history_entries(self) -> usize {
        self.history_entries
    }
    pub fn history_bytes(self) -> usize {
        self.history_bytes
    }
}

/// Complete changed domains for this non-element surface. Created/destroyed
/// nodes cover their entire state; surviving nodes report data and relations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomChange {
    Created(NodeId),
    Destroyed(NodeId),
    Data(NodeId),
    Parent(NodeId),
    Children(NodeId),
}

#[derive(Clone, Debug)]
pub struct DomChanges {
    from: DomStamp,
    to: DomStamp,
    changes: Arc<[DomChange]>,
}
impl DomChanges {
    pub fn from(&self) -> DomStamp {
        self.from
    }
    pub fn to(&self) -> DomStamp {
        self.to
    }
    pub fn changes(&self) -> &[DomChange] {
        &self.changes
    }
}

/// Complete interval unavailable; a consumer must recompute from a snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomHistoryError {
    ForeignOwner,
    FutureRevision,
    FullRecomputeRequired { available_from: DomRevision },
    AllocationFailed,
}
impl std::fmt::Display for DomHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DOM change coverage unavailable: {self:?}")
    }
}
impl std::error::Error for DomHistoryError {}

#[derive(Debug)]
pub struct DomCommit {
    owner: Arc<BatchIdentity>,
    created: Vec<Option<NodeId>>,
    documents: Vec<bool>,
    changes: DomChanges,
}
impl DomCommit {
    pub fn changes(&self) -> &DomChanges {
        &self.changes
    }
    /// None means the batch destroyed its own unpublished creation.
    pub fn created(&self, ticket: &CreatedNode) -> Result<Option<NodeId>, DomError> {
        if !Arc::ptr_eq(&ticket.owner, &self.owner) {
            return Err(DomError::ForeignBatch);
        }
        self.created
            .get(ticket.index)
            .copied()
            .ok_or(DomError::UnavailableCreation)
    }
    pub fn document(&self, ticket: &CreatedNode) -> Result<DocumentId, DomError> {
        let id = self.created(ticket)?.ok_or(DomError::UnavailableCreation)?;
        if !self.documents[ticket.index] {
            return Err(DomError::WrongNodeKind);
        }
        Ok(DocumentId(id))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Node {
    pub(crate) document: DocumentId,
    pub(crate) kind: DomNodeKind,
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
}
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Slot {
    pub(crate) generation: u64,
    retired: bool,
    pub(crate) node: Option<Node>,
}
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct State {
    pub(crate) slots: Vec<Slot>,
    nodes: usize,
    units: usize,
}
impl State {
    pub(crate) fn node(&self, owner: DomOwnerId, id: NodeId) -> Result<&Node, DomError> {
        if id.owner != owner {
            return Err(DomError::ForeignOwner);
        }
        self.slots
            .get(id.slot)
            .filter(|s| s.generation == id.generation)
            .and_then(|s| s.node.as_ref())
            .ok_or(DomError::StaleNode)
    }
}

struct HistoryEntry {
    from: DomRevision,
    changes: Arc<[DomChange]>,
    bytes: usize,
    older: Option<Box<HistoryEntry>>,
}
fn discard_history(mut history: Option<Box<HistoryEntry>>) {
    while let Some(mut entry) = history {
        history = entry.older.take();
    }
}

/// Exclusive canonical mutation owner. Snapshots never borrow its mutable state.
pub struct Dom {
    owner: DomOwnerId,
    revision: DomRevision,
    limits: DomLimits,
    state: Arc<State>,
    history: Option<Box<HistoryEntry>>,
    history_entries: usize,
    history_bytes: usize,
}
impl Drop for Dom {
    fn drop(&mut self) {
        discard_history(self.history.take());
    }
}
impl Dom {
    pub fn new(limits: DomLimits) -> Result<Self, DomError> {
        Ok(Self {
            owner: DomOwnerId::allocate()?,
            revision: DomRevision(0),
            limits,
            state: Arc::new(State {
                slots: Vec::new(),
                nodes: 0,
                units: 0,
            }),
            history: None,
            history_entries: 0,
            history_bytes: 0,
        })
    }
    pub fn owner(&self) -> DomOwnerId {
        self.owner
    }
    pub fn revision(&self) -> DomRevision {
        self.revision
    }
    pub fn limits(&self) -> DomLimits {
        self.limits
    }
    pub fn usage(&self) -> DomUsage {
        DomUsage {
            nodes: self.state.nodes,
            units: self.state.units,
            history_entries: self.history_entries,
            history_bytes: self.history_bytes,
        }
    }
    pub fn snapshot(&self) -> DomSnapshot {
        DomSnapshot {
            owner: self.owner,
            revision: self.revision,
            state: self.state.clone(),
        }
    }
    /// Live bounds cannot fall below admitted state. Smaller history bounds
    /// discard coverage explicitly, without changing semantic revision.
    pub fn set_limits(&mut self, limits: DomLimits) -> Result<(), DomError> {
        admit(self.state.nodes, limits.nodes, DomResource::ManagedNodes)?;
        admit(self.state.units, limits.units, DomResource::StringCodeUnits)?;
        self.limits = limits;
        self.trim_history();
        Ok(())
    }
    pub fn apply(
        &mut self,
        batch: DomBatch,
        cancelled: &AtomicBool,
    ) -> Result<DomCommit, DomError> {
        let prepared = self.prepare(batch, cancelled)?;
        self.publish(prepared, cancelled)
    }
    fn prepare(&self, batch: DomBatch, cancelled: &AtomicBool) -> Result<Prepared, DomError> {
        check_cancel(cancelled)?;
        admit(
            batch.commands.len(),
            self.limits.commands,
            DomResource::Commands,
        )?;
        let mut draft = Draft::new(self, batch.owner)?;
        for command in batch.commands {
            check_cancel(cancelled)?;
            draft.apply(command, self.limits)?;
        }
        check_cancel(cancelled)?;
        draft.finish(self)
    }
    fn publish(
        &mut self,
        prepared: Prepared,
        cancelled: &AtomicBool,
    ) -> Result<DomCommit, DomError> {
        check_cancel(cancelled)?;
        let Prepared {
            state,
            owner,
            created,
            documents,
            changes,
        } = prepared;
        let changed = state != *self.state;
        let revision = if changed {
            DomRevision(
                self.revision
                    .0
                    .checked_add(1)
                    .ok_or(DomError::RevisionExhausted)?,
            )
        } else {
            self.revision
        };
        let from = DomStamp {
            owner: self.owner,
            revision: self.revision,
        };
        let to = DomStamp {
            owner: self.owner,
            revision,
        };
        let changes: Arc<[DomChange]> = changes.into();
        let bytes = size_of::<HistoryEntry>()
            .checked_add(
                changes
                    .len()
                    .checked_mul(size_of::<DomChange>())
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .ok_or(DomError::RepresentationExhausted)?;
        let entry =
            if changed && self.limits.history_entries != 0 && bytes <= self.limits.history_bytes {
                Some(Box::new(HistoryEntry {
                    from: self.revision,
                    changes: changes.clone(),
                    bytes,
                    older: None,
                }))
            } else {
                None
            };
        let state = Arc::new(state);
        // Last cancellation observation is the publication boundary. No user
        // callbacks or external effects occur while prepared state is private.
        check_cancel(cancelled)?;
        if changed {
            self.state = state;
            self.revision = revision;
            if let Some(mut entry) = entry {
                entry.older = self.history.take();
                self.history = Some(entry);
                self.trim_history();
            } else {
                discard_history(self.history.take());
                self.history_entries = 0;
                self.history_bytes = 0;
            }
        }
        Ok(DomCommit {
            owner,
            created,
            documents,
            changes: DomChanges { from, to, changes },
        })
    }
    fn trim_history(&mut self) {
        let mut entries = 0usize;
        let mut bytes = 0usize;
        let mut cursor = &mut self.history;
        loop {
            let keep = cursor.as_ref().is_some_and(|entry| {
                entries < self.limits.history_entries
                    && entry.bytes <= self.limits.history_bytes - bytes
            });
            if !keep {
                discard_history(cursor.take());
                break;
            }
            let entry = cursor.as_mut().expect("retained entry");
            entries += 1;
            bytes += entry.bytes;
            cursor = &mut entry.older;
        }
        self.history_entries = entries;
        self.history_bytes = bytes;
    }
    /// Complete chronological change records, or an explicit recompute result.
    pub fn changes_since(&self, baseline: DomStamp) -> Result<DomChanges, DomHistoryError> {
        if baseline.owner != self.owner {
            return Err(DomHistoryError::ForeignOwner);
        }
        if baseline.revision > self.revision {
            return Err(DomHistoryError::FutureRevision);
        }
        let mut entries = Vec::new();
        let mut cursor = self.history.as_deref();
        let mut available_from = self.revision;
        while let Some(entry) = cursor {
            available_from = entry.from;
            if entry.from >= baseline.revision {
                entries
                    .try_reserve(1)
                    .map_err(|_| DomHistoryError::AllocationFailed)?;
                entries.push(entry);
            }
            cursor = entry.older.as_deref();
        }
        if baseline.revision < available_from {
            return Err(DomHistoryError::FullRecomputeRequired { available_from });
        }
        let mut changes = Vec::new();
        for entry in entries.into_iter().rev() {
            changes
                .try_reserve(entry.changes.len())
                .map_err(|_| DomHistoryError::AllocationFailed)?;
            changes.extend_from_slice(&entry.changes);
        }
        Ok(DomChanges {
            from: baseline,
            to: DomStamp {
                owner: self.owner,
                revision: self.revision,
            },
            changes: changes.into(),
        })
    }
}

fn admit(value: usize, limit: usize, resource: DomResource) -> Result<(), DomError> {
    if value > limit {
        Err(DomError::ResourceLimit { resource, limit })
    } else {
        Ok(())
    }
}
fn check_cancel(cancelled: &AtomicBool) -> Result<(), DomError> {
    if cancelled.load(Ordering::Acquire) {
        Err(DomError::Cancelled)
    } else {
        Ok(())
    }
}
fn reserve<T>(values: &mut Vec<T>, additional: usize) -> Result<(), DomError> {
    values
        .try_reserve(additional)
        .map_err(|_| DomError::AllocationFailed)
}
fn copy<T: Clone>(values: &[T]) -> Result<Vec<T>, DomError> {
    let mut result = Vec::new();
    reserve(&mut result, values.len())?;
    result.extend_from_slice(values);
    Ok(result)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Key {
    Existing(usize),
    Created(usize),
}
struct DraftNode {
    document: Key,
    kind: DomNodeKind,
    parent: Option<Key>,
    children: Vec<Key>,
}
struct Draft {
    owner: Arc<BatchIdentity>,
    store: DomOwnerId,
    existing: Vec<Option<DraftNode>>,
    generations: Vec<u64>,
    created: Vec<Option<DraftNode>>,
    nodes: usize,
    units: usize,
}
struct Prepared {
    state: State,
    owner: Arc<BatchIdentity>,
    created: Vec<Option<NodeId>>,
    documents: Vec<bool>,
    changes: Vec<DomChange>,
}

impl Draft {
    fn new(dom: &Dom, owner: Arc<BatchIdentity>) -> Result<Self, DomError> {
        let mut existing = Vec::new();
        let mut generations = Vec::new();
        reserve(&mut existing, dom.state.slots.len())?;
        reserve(&mut generations, dom.state.slots.len())?;
        for slot in &dom.state.slots {
            let node = if let Some(node) = &slot.node {
                let mut children = Vec::new();
                reserve(&mut children, node.children.len())?;
                children.extend(node.children.iter().map(|id| Key::Existing(id.slot)));
                Some(DraftNode {
                    document: Key::Existing(node.document.0.slot),
                    kind: node.kind.clone(),
                    parent: node.parent.map(|id| Key::Existing(id.slot)),
                    children,
                })
            } else {
                None
            };
            existing.push(node);
            generations.push(slot.generation);
        }
        Ok(Self {
            owner,
            store: dom.owner,
            existing,
            generations,
            created: Vec::new(),
            nodes: dom.state.nodes,
            units: dom.state.units,
        })
    }
    fn node(&self, key: Key) -> Result<&DraftNode, DomError> {
        match key {
            Key::Existing(i) => self.existing.get(i),
            Key::Created(i) => self.created.get(i),
        }
        .and_then(Option::as_ref)
        .ok_or(DomError::StaleNode)
    }
    fn node_mut(&mut self, key: Key) -> &mut DraftNode {
        match key {
            Key::Existing(i) => &mut self.existing[i],
            Key::Created(i) => &mut self.created[i],
        }
        .as_mut()
        .expect("validated staged node")
    }
    fn resolve(&self, operand: NodeOperand) -> Result<Key, DomError> {
        let key = match operand {
            NodeOperand::Existing(id) => {
                if id.owner != self.store {
                    return Err(DomError::ForeignOwner);
                }
                if self.generations.get(id.slot) != Some(&id.generation) {
                    return Err(DomError::StaleNode);
                }
                Key::Existing(id.slot)
            }
            NodeOperand::Created(ticket) => {
                if !Arc::ptr_eq(&self.owner, &ticket.owner) {
                    return Err(DomError::ForeignBatch);
                }
                if self
                    .created
                    .get(ticket.index)
                    .and_then(Option::as_ref)
                    .is_none()
                {
                    return Err(DomError::UnavailableCreation);
                }
                Key::Created(ticket.index)
            }
        };
        self.node(key)?;
        Ok(key)
    }
    fn detach(&mut self, node: Key) {
        if let Some(parent) = self.node(node).expect("staged node").parent {
            self.node_mut(parent)
                .children
                .retain(|child| *child != node);
            self.node_mut(node).parent = None;
        }
    }
    fn apply(&mut self, command: Command, limits: DomLimits) -> Result<(), DomError> {
        match command {
            Command::Create { kind, document } => {
                let key = Key::Created(self.created.len());
                let document = if let Some(document) = document {
                    let document = self.resolve(document)?;
                    if self.node(document)?.kind != DomNodeKind::Document {
                        return Err(DomError::WrongNodeKind);
                    }
                    document
                } else {
                    key
                };
                let kind = match kind {
                    Creation::Document => DomNodeKind::Document,
                    Creation::Fragment => DomNodeKind::DocumentFragment,
                    Creation::Text(data) => DomNodeKind::Text(data),
                    Creation::Comment(data) => DomNodeKind::Comment(data),
                };
                let units = self
                    .units
                    .checked_add(kind_units(&kind))
                    .ok_or(DomError::RepresentationExhausted)?;
                let nodes = self
                    .nodes
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(nodes, limits.nodes, DomResource::ManagedNodes)?;
                admit(units, limits.units, DomResource::StringCodeUnits)?;
                reserve(&mut self.created, 1)?;
                self.created.push(Some(DraftNode {
                    document,
                    kind,
                    parent: None,
                    children: Vec::new(),
                }));
                self.nodes = nodes;
                self.units = units;
            }
            Command::Detach(node) => {
                let node = self.resolve(node)?;
                self.detach(node);
            }
            Command::Destroy(node) => {
                let node = self.resolve(node)?;
                if self.node(node)?.parent.is_some()
                    || self.node(node)?.kind == DomNodeKind::Document
                {
                    return Err(DomError::InvalidDestruction);
                }
                let mut pending = Vec::new();
                reserve(&mut pending, self.nodes)?;
                pending.push(node);
                while let Some(key) = pending.pop() {
                    let node = match key {
                        Key::Existing(i) => self.existing[i].take(),
                        Key::Created(i) => self.created[i].take(),
                    }
                    .expect("owned subtree");
                    pending.extend(node.children);
                    self.nodes -= 1;
                    self.units -= kind_units(&node.kind);
                }
            }
            Command::ReplaceData {
                node,
                offset,
                count,
                data,
            } => {
                let node = self.resolve(node)?;
                let previous = match &self.node(node)?.kind {
                    DomNodeKind::Text(data) | DomNodeKind::Comment(data) => data,
                    _ => return Err(DomError::WrongNodeKind),
                };
                if offset > previous.len() {
                    return Err(DomError::IndexSize);
                }
                let removed = count.min(previous.len() - offset);
                let length = (previous.len() - removed)
                    .checked_add(data.len())
                    .ok_or(DomError::RepresentationExhausted)?;
                let units = (self.units - previous.len())
                    .checked_add(length)
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(units, limits.units, DomResource::StringCodeUnits)?;
                let mut value = Vec::new();
                reserve(&mut value, length)?;
                value.extend_from_slice(&previous.code_units()[..offset]);
                value.extend_from_slice(data.code_units());
                value.extend_from_slice(&previous.code_units()[offset + removed..]);
                let value = DocumentString::from_utf16(value);
                let target = self.node_mut(node);
                target.kind = if matches!(target.kind, DomNodeKind::Text(_)) {
                    DomNodeKind::Text(value)
                } else {
                    DomNodeKind::Comment(value)
                };
                self.units = units;
            }
            Command::Insert {
                node,
                parent,
                before,
            } => {
                let node = self.resolve(node)?;
                let parent = self.resolve(parent)?;
                if !matches!(
                    self.node(parent)?.kind,
                    DomNodeKind::Document | DomNodeKind::DocumentFragment
                ) {
                    return Err(DomError::HierarchyRequest);
                }
                let mut ancestor = Some(parent);
                while let Some(key) = ancestor {
                    if key == node {
                        return Err(DomError::HierarchyRequest);
                    }
                    ancestor = self.node(key)?.parent;
                }
                let mut before = before.map(|id| self.resolve(id)).transpose()?;
                if before.is_some_and(|child| {
                    self.node(child).expect("resolved child").parent != Some(parent)
                }) {
                    return Err(DomError::NotFound);
                }
                if self.node(node)?.kind == DomNodeKind::Document {
                    return Err(DomError::HierarchyRequest);
                }
                if self.node(node)?.document != self.node(parent)?.document {
                    return Err(DomError::CrossDocumentInsertionUnsupported);
                }
                if before == Some(node) {
                    let children = &self.node(parent)?.children;
                    let position = children
                        .iter()
                        .position(|child| *child == node)
                        .expect("reference membership");
                    before = children.get(position + 1).copied();
                }
                let moving = if self.node(node)?.kind == DomNodeKind::DocumentFragment {
                    copy(&self.node(node)?.children)?
                } else {
                    vec![node]
                };
                if self.node(parent)?.kind == DomNodeKind::Document
                    && moving.iter().any(|key| {
                        matches!(
                            self.node(*key).expect("staged child").kind,
                            DomNodeKind::Text(_)
                        )
                    })
                {
                    return Err(DomError::HierarchyRequest);
                }
                reserve(&mut self.node_mut(parent).children, moving.len())?;
                for child in moving {
                    self.detach(child);
                    let target = self.node_mut(parent);
                    let index = before.map_or(target.children.len(), |reference| {
                        target
                            .children
                            .iter()
                            .position(|child| *child == reference)
                            .expect("retained reference")
                    });
                    target.children.insert(index, child);
                    self.node_mut(child).parent = Some(parent);
                }
            }
        }
        Ok(())
    }
    fn finish(self, dom: &Dom) -> Result<Prepared, DomError> {
        let mut slots = Vec::new();
        let mut occupants = Vec::new();
        reserve(&mut slots, dom.state.slots.len())?;
        reserve(&mut occupants, dom.state.slots.len())?;
        for (index, old) in dom.state.slots.iter().enumerate() {
            let destroyed = old.node.is_some() && self.existing[index].is_none();
            let generation = if destroyed {
                old.generation.checked_add(1).unwrap_or(old.generation)
            } else {
                old.generation
            };
            slots.push(Slot {
                generation,
                retired: old.retired || (destroyed && old.generation == u64::MAX),
                node: None,
            });
            occupants.push(self.existing[index].as_ref().map(|_| Key::Existing(index)));
        }
        let mut created = Vec::new();
        let mut documents = Vec::new();
        reserve(&mut created, self.created.len())?;
        reserve(&mut documents, self.created.len())?;
        for (index, node) in self.created.iter().enumerate() {
            documents.push(
                node.as_ref()
                    .is_some_and(|node| node.kind == DomNodeKind::Document),
            );
            let Some(_) = node else {
                created.push(None);
                continue;
            };
            let slot = occupants
                .iter()
                .enumerate()
                .position(|(i, key)| key.is_none() && !slots[i].retired);
            let slot = if let Some(slot) = slot {
                slot
            } else {
                reserve(&mut slots, 1)?;
                reserve(&mut occupants, 1)?;
                slots.push(Slot {
                    generation: 0,
                    retired: false,
                    node: None,
                });
                occupants.push(None);
                slots.len() - 1
            };
            occupants[slot] = Some(Key::Created(index));
            created.push(Some(NodeId {
                owner: dom.owner,
                slot,
                generation: slots[slot].generation,
            }));
        }
        let id = |key| match key {
            Key::Existing(slot) => NodeId {
                owner: dom.owner,
                slot,
                generation: dom.state.slots[slot].generation,
            },
            Key::Created(index) => created[index].expect("surviving creation reference"),
        };
        for (slot, key) in occupants.into_iter().enumerate() {
            let Some(key) = key else {
                continue;
            };
            let source = self.node(key)?;
            let mut children = Vec::new();
            reserve(&mut children, source.children.len())?;
            children.extend(source.children.iter().copied().map(id));
            slots[slot].node = Some(Node {
                document: DocumentId(id(source.document)),
                kind: source.kind.clone(),
                parent: source.parent.map(id),
                children,
            });
        }
        let state = State {
            slots,
            nodes: self.nodes,
            units: self.units,
        };
        let changes = difference(dom.owner, &dom.state, &state)?;
        Ok(Prepared {
            state,
            owner: self.owner,
            created,
            documents,
            changes,
        })
    }
}
fn kind_units(kind: &DomNodeKind) -> usize {
    match kind {
        DomNodeKind::Text(data) | DomNodeKind::Comment(data) => data.len(),
        _ => 0,
    }
}
fn difference(owner: DomOwnerId, old: &State, new: &State) -> Result<Vec<DomChange>, DomError> {
    let mut changes = Vec::new();
    for index in 0..old.slots.len().max(new.slots.len()) {
        let before = old
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let after = new
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let same = before.zip(after).is_some_and(|(a, b)| a.0 == b.0);
        if let Some((generation, node)) = before {
            let id = NodeId {
                owner,
                slot: index,
                generation,
            };
            if same {
                let current = after.expect("same live identity").1;
                for change in [
                    (node.kind != current.kind).then_some(DomChange::Data(id)),
                    (node.parent != current.parent).then_some(DomChange::Parent(id)),
                    (node.children != current.children).then_some(DomChange::Children(id)),
                ]
                .into_iter()
                .flatten()
                {
                    reserve(&mut changes, 1)?;
                    changes.push(change);
                }
            } else {
                reserve(&mut changes, 1)?;
                changes.push(DomChange::Destroyed(id));
            }
        }
        if let Some((generation, _)) = after
            && !same
        {
            reserve(&mut changes, 1)?;
            changes.push(DomChange::Created(NodeId {
                owner,
                slot: index,
                generation,
            }));
        }
    }
    Ok(changes)
}

#[cfg(test)]
mod tests;

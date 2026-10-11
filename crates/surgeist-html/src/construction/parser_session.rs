//! Functional structural checkpoint over the actual construction records.
//!
//! Element payloads are preexisting immutable inputs: stored slots and inspection
//! work count, but their backing allocations are outside this session's physical
//! ledger. Token-created strings/attributes must gain an ownership transfer
//! producer before this module can be consumed by an ingested tree builder.
//! No text merging, lexical metadata, origin report, or public parser is supplied.

use super::*;
use crate::parser::control::{
    ControlIdentity, ParseControl, ParseStop, Resource, SharedAllocation, VectorCapacity,
    WorkCategory,
};
use std::{mem, sync::Weak};

#[derive(Clone, Debug)]
struct ParserNode {
    owner: Weak<Identity>,
    index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SessionError {
    Stop(ParseStop),
    ForeignNode,
    RootMove,
    AssociationMove,
    NotContainer,
    TemplateContentsRequired,
    WrongSibling,
    Cycle,
    Consumed,
}
impl From<ParseStop> for SessionError {
    fn from(value: ParseStop) -> Self {
        Self::Stop(value)
    }
}

enum RootKind {
    Document(HtmlDocumentMode),
    Fragment,
}

struct ParserArenaSession {
    storage: Option<SessionStorage>,
}

struct SessionStorage {
    control: ControlIdentity,
    identity: Arc<Identity>,
    identity_allocation: SharedAllocation<Identity>,
    records: Vec<Record>,
    records_allocation: VectorCapacity<Record>,
    children_allocations: Vec<VectorCapacity<usize>>,
    tickets_allocation: VectorCapacity<VectorCapacity<usize>>,
    usage: HtmlConstructionUsage,
}

enum FrozenView {
    Document(HtmlDocument),
    Fragment(HtmlFragment),
}
struct FrozenArena {
    storage: Option<FrozenStorage>,
}

struct FrozenStorage {
    view: FrozenView,
    control: ControlIdentity,
    arena_allocation: SharedAllocation<Arena>,
    identity_allocation: SharedAllocation<Identity>,
    records_allocation: VectorCapacity<Record>,
    children_allocations: Vec<VectorCapacity<usize>>,
    tickets_allocation: VectorCapacity<VectorCapacity<usize>>,
}
impl SessionStorage {
    fn new(control: &mut ParseControl, root: RootKind) -> Result<Self, SessionError> {
        let (records, records_allocation) = control.new_vector()?;
        let (children_allocations, tickets_allocation) = control.new_vector()?;
        let (identity, identity_allocation) = control
            .share_owned(Identity)
            .map_err(|(stop, _)| SessionError::Stop(stop))?;
        let mut session = Self {
            control: control.identity(),
            identity,
            identity_allocation,
            records,
            records_allocation,
            children_allocations,
            tickets_allocation,
            usage: HtmlConstructionUsage::default(),
        };
        let kind = match root {
            RootKind::Document(mode) => HtmlNodeKind::Document(mode),
            RootKind::Fragment => HtmlNodeKind::Fragment,
        };
        if let Err(error) = session.allocate(control, kind, false, 0, 0) {
            session.release(control);
            return Err(error);
        }
        Ok(session)
    }

    fn require(&self, control: &mut ParseControl) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        Ok(())
    }
    fn node(&self, control: &mut ParseControl, index: usize) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::CopyByte, mem::size_of::<ParserNode>() as u64)?;
        Ok(ParserNode {
            owner: Arc::downgrade(&self.identity),
            index,
        })
    }
    fn root(&self, control: &mut ParseControl) -> Result<ParserNode, SessionError> {
        self.node(control, 0)
    }
    fn index(&self, control: &mut ParseControl, node: &ParserNode) -> Result<usize, SessionError> {
        self.require(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if node.owner.as_ptr() != Arc::as_ptr(&self.identity) || node.index >= self.records.len() {
            return Err(SessionError::ForeignNode);
        }
        Ok(node.index)
    }

    /// Consumes preexisting immutable payloads, not tokenizer-created backing.
    fn allocate_element(
        &mut self,
        control: &mut ParseControl,
        element: HtmlElement,
    ) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        let name = element.name();
        let template = exact_ascii(
            control,
            name.namespace().uri(),
            "http://www.w3.org/1999/xhtml",
        )? && exact_ascii(control, Some(name.local_name()), "template")?;
        let mut units = name_units(control, name)?;
        let attributes = element.attributes().len();
        for index in 0..attributes {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let attribute = &element.attributes().as_slice()[index];
            let Some(next) = units
                .checked_add(name_units(control, attribute.name())?)
                .and_then(|sum| sum.checked_add(attribute.value().len()))
            else {
                return control
                    .representation(Resource::ManagedUtf16Units)
                    .map_err(Into::into);
            };
            units = next;
        }
        self.allocate(
            control,
            HtmlNodeKind::Element(element),
            template,
            attributes,
            units,
        )
    }

    fn allocate(
        &mut self,
        control: &mut ParseControl,
        kind: HtmlNodeKind,
        template: bool,
        attributes: usize,
        units: usize,
    ) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        let count = 1 + usize::from(template);
        let requests = [
            (Resource::ManagedNodes, count as u64),
            (Resource::ManagedEdges, u64::from(template)),
            (Resource::ManagedAttributes, attributes as u64),
            (Resource::ManagedUtf16Units, units as u64),
            (Resource::AuxiliaryEntries, count as u64),
        ];
        control.check_acquire_many(&requests)?;
        control.reserve_vector(&mut self.records, &mut self.records_allocation, count)?;
        control.reserve_vector(
            &mut self.children_allocations,
            &mut self.tickets_allocation,
            count,
        )?;
        let (children, children_allocation) = control.new_vector()?;
        let associated = if template {
            Some(control.new_vector::<usize>()?)
        } else {
            None
        };
        let index = self.records.len();
        let handle = self.node(control, index)?;
        let owner = if template {
            control.charge(WorkCategory::ProviderCall, 1)?;
            Some(HtmlFragmentOwner::Template(HtmlNodeId {
                identity: self.identity.clone(),
                index,
            }))
        } else {
            None
        };
        control.observe_tree_depth(self.usage.depth.max(usize::from(template)) as u64)?;
        control.admit_atomic_layout(mem::size_of::<Record>() * count)?;
        control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<Record>() as u64 * count as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, (count * 2) as u64)?;
        // All allocations/checks precede this final logical commit. The bounded
        // append writes at most two records and two tickets, without further polls.
        control.acquire_many(&requests)?;
        self.records.push(Record {
            kind,
            parent: None,
            children,
            depth: 0,
            template_contents: template.then_some(index + 1),
            shadow: None,
            owner: None,
        });
        self.children_allocations.push(children_allocation);
        if let Some((children, allocation)) = associated {
            self.records.push(Record {
                kind: HtmlNodeKind::Fragment,
                parent: None,
                children,
                depth: 1,
                template_contents: None,
                shadow: None,
                owner,
            });
            self.children_allocations.push(allocation);
        }
        self.usage.nodes += count;
        self.usage.edges += usize::from(template);
        self.usage.attributes += attributes;
        self.usage.utf16_units += units;
        self.usage.depth = self.usage.depth.max(usize::from(template));
        Ok(handle)
    }

    fn template_contents(
        &self,
        control: &mut ParseControl,
        element: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        let index = self.index(control, element)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.records[index]
            .template_contents
            .map(|index| self.node(control, index))
            .transpose()
    }

    fn container(&self, control: &mut ParseControl, index: usize) -> Result<(), SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.records[index].template_contents.is_some() {
            return Err(SessionError::TemplateContentsRequired);
        }
        match self.records[index].kind {
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_) => Ok(()),
            _ => Err(SessionError::NotContainer),
        }
    }

    /// Follow ordinary or association parents, with one proposed ordinary move.
    /// No recursive tree or temporary ancestry allocation is needed.
    fn depth(
        &self,
        control: &mut ParseControl,
        mut index: usize,
        change: Option<(usize, Option<usize>)>,
    ) -> Result<usize, SessionError> {
        let mut depth = 0;
        loop {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let record = &self.records[index];
            let parent = if let Some((_, parent)) = change.filter(|(node, _)| *node == index) {
                parent
            } else if let Some(owner) = &record.owner {
                let (HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) =
                    owner;
                Some(owner.index)
            } else {
                record.parent
            };
            let Some(parent) = parent else {
                return Ok(depth);
            };
            depth += 1;
            if depth > self.records.len() {
                return Err(SessionError::Cycle);
            }
            index = parent;
        }
    }
    fn maximum_depth(
        &self,
        control: &mut ParseControl,
        change: Option<(usize, Option<usize>)>,
    ) -> Result<usize, SessionError> {
        let mut maximum = 0;
        for index in 0..self.records.len() {
            maximum = maximum.max(self.depth(control, index, change)?);
        }
        Ok(maximum)
    }

    /// Prepared child order. The real replacement vector is admitted and every
    /// entry is read/copied before use. Commit later consists only of vector swaps.
    fn child_order(
        &self,
        control: &mut ParseControl,
        parent: usize,
        remove: usize,
        insert: Option<(usize, Option<usize>)>,
    ) -> Result<(Vec<usize>, VectorCapacity<usize>), SessionError> {
        let (mut order, mut allocation) = control.new_vector()?;
        let prepare = (|| {
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.reserve_vector(
                &mut order,
                &mut allocation,
                self.records[parent].children.len() + 1,
            )?;
            let mut inserted = false;
            for index in 0..self.records[parent].children.len() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
                let child = self.records[parent].children[index];
                if child == remove {
                    continue;
                }
                if let Some((node, Some(before))) = insert
                    && child == before
                {
                    control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
                    control.charge(WorkCategory::RecordWrite, 1)?;
                    control.acquire(Resource::AuxiliaryEntries, 1)?;
                    order.push(node);
                    inserted = true;
                }
                control.charge(WorkCategory::RecordWrite, 1)?;
                control.acquire(Resource::AuxiliaryEntries, 1)?;
                order.push(child);
            }
            if let Some((node, before)) = insert
                && !inserted
            {
                if before.is_some() {
                    return Err(SessionError::WrongSibling);
                }
                control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
                control.charge(WorkCategory::RecordWrite, 1)?;
                control.acquire(Resource::AuxiliaryEntries, 1)?;
                order.push(node);
            }
            Ok(())
        })();
        if let Err(error) = prepare {
            release_order(control, &self.control, order, allocation)?;
            return Err(error);
        }
        Ok((order, allocation))
    }

    fn insert(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
    ) -> Result<(), SessionError> {
        let parent = self.index(control, parent)?;
        let node = self.index(control, node)?;
        let before = before.map(|node| self.index(control, node)).transpose()?;
        self.container(control, parent)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if node == 0 {
            return Err(SessionError::RootMove);
        }
        if self.records[node].owner.is_some() {
            return Err(SessionError::AssociationMove);
        }
        if let Some(before) = before {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if self.records[before].parent != Some(parent) {
                return Err(SessionError::WrongSibling);
            }
            if before == node {
                return Ok(());
            }
        }
        let old_parent = self.records[node].parent;
        let depth = self.maximum_depth(control, Some((node, Some(parent))))?;
        control.observe_tree_depth(depth as u64)?;
        let requests = [(Resource::ManagedEdges, u64::from(old_parent.is_none()))];
        control.check_acquire_many(&requests)?;
        let (order, allocation) = self.child_order(control, parent, node, Some((node, before)))?;
        let old_order = if let Some(old) = old_parent.filter(|old| *old != parent) {
            match self.child_order(control, old, node, None) {
                Ok(order) => Some((old, order)),
                Err(error) => {
                    release_order(control, &self.control, order, allocation)?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        let admission = (|| {
            control.charge(WorkCategory::RecordWrite, 8)?;
            control.charge(
                WorkCategory::CopyByte,
                (mem::size_of::<Vec<usize>>() * 4) as u64,
            )?;
            let scratch_entries = order.len() as u64
                + old_order
                    .as_ref()
                    .map_or(0, |(_, (order, _))| order.len() as u64);
            control.commit_child_orders(&self.control, scratch_entries, requests[0].1, 0)?;
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = admission {
            release_order(control, &self.control, order, allocation)?;
            if let Some((_, (order, allocation))) = old_order {
                release_order(control, &self.control, order, allocation)?;
            }
            return Err(error);
        }
        let mut retired = self.swap_order(parent, order, allocation);
        let mut retired_old =
            old_order.map(|(old, (order, allocation))| self.swap_order(old, order, allocation));
        self.records[node].parent = Some(parent);
        self.usage.edges += usize::from(old_parent.is_none());
        self.usage.depth = depth;
        // Complete the same fixed commit by retiring the old initialized
        // references. usize has no destructor, so clear is a bounded length
        // metadata write, not a useful-work traversal or fallible provider call.
        // No initialized retired scratch survives to the capacity cleanup below.
        retired.0.clear();
        if let Some((order, _)) = &mut retired_old {
            order.clear();
        }
        release_vector(control, retired.0, retired.1)?;
        if let Some((order, allocation)) = retired_old {
            release_vector(control, order, allocation)?;
        }
        Ok(())
    }

    fn swap_order(
        &mut self,
        index: usize,
        order: Vec<usize>,
        allocation: VectorCapacity<usize>,
    ) -> (Vec<usize>, VectorCapacity<usize>) {
        (
            mem::replace(&mut self.records[index].children, order),
            mem::replace(&mut self.children_allocations[index], allocation),
        )
    }

    fn remove(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(), SessionError> {
        let node = self.index(control, node)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if node == 0 {
            return Err(SessionError::RootMove);
        }
        if self.records[node].owner.is_some() {
            return Err(SessionError::AssociationMove);
        }
        let Some(parent) = self.records[node].parent else {
            return Ok(());
        };
        let depth = self.maximum_depth(control, Some((node, None)))?;
        control.observe_tree_depth(depth as u64)?;
        let (order, allocation) = self.child_order(control, parent, node, None)?;
        let admission = (|| {
            control.charge(WorkCategory::RecordWrite, 4)?;
            control.charge(
                WorkCategory::CopyByte,
                (mem::size_of::<Vec<usize>>() * 2) as u64,
            )?;
            control.commit_child_orders(&self.control, order.len() as u64, 0, 1)?;
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = admission {
            release_order(control, &self.control, order, allocation)?;
            return Err(error);
        }
        let (mut order, allocation) = self.swap_order(parent, order, allocation);
        self.records[node].parent = None;
        self.usage.edges -= 1;
        self.usage.depth = depth;
        order.clear();
        release_vector(control, order, allocation)
    }

    fn prepare_freeze(&mut self, control: &mut ParseControl) -> Result<bool, SessionError> {
        self.require(control)?;
        let depth = self.maximum_depth(control, None)?;
        control.observe_tree_depth(depth as u64)?;
        for index in 0..self.records.len() {
            let depth = self.depth(control, index, None)?;
            control.charge(WorkCategory::RecordWrite, 1)?;
            self.records[index].depth = depth;
            control.charge(WorkCategory::RecordVisit, 1)?;
            for position in 0..self.records[index].children.len() {
                control.charge(WorkCategory::RecordVisit, 2)?;
                let child = self.records[index].children[position];
                if self.records[child].parent != Some(index) {
                    return Err(SessionError::WrongSibling);
                }
            }
        }
        self.usage.depth = depth;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let document = matches!(self.records[0].kind, HtmlNodeKind::Document(_));
        control.charge(WorkCategory::CopyByte, mem::size_of::<Arena>() as u64)?;
        Ok(document)
    }

    /// Called only after the owning transition verifies cleanup identity. This
    /// consumes actual storage; exact private tickets prove the release invariant.
    fn release(self, control: &mut ParseControl) {
        let Self {
            control: owner,
            identity,
            identity_allocation,
            records,
            records_allocation,
            children_allocations,
            tickets_allocation,
            usage,
        } = self;
        release_parts(
            control,
            &owner,
            Arena {
                identity,
                records,
                usage,
            },
            records_allocation,
            children_allocations,
            tickets_allocation,
            identity_allocation,
        );
    }
}

/// Borrowed transitions retain the owner on every failure, including terminal
/// stops. Storage is absent only after successful freeze or explicit cleanup.
impl ParserArenaSession {
    fn new(control: &mut ParseControl, root: RootKind) -> Result<Self, SessionError> {
        Ok(Self {
            storage: Some(SessionStorage::new(control, root)?),
        })
    }
    fn active(&self) -> Result<&SessionStorage, SessionError> {
        self.storage.as_ref().ok_or(SessionError::Consumed)
    }
    fn active_mut(&mut self) -> Result<&mut SessionStorage, SessionError> {
        self.storage.as_mut().ok_or(SessionError::Consumed)
    }
    fn root(&self, control: &mut ParseControl) -> Result<ParserNode, SessionError> {
        self.active()?.root(control)
    }
    fn allocate_element(
        &mut self,
        control: &mut ParseControl,
        element: HtmlElement,
    ) -> Result<ParserNode, SessionError> {
        self.active_mut()?.allocate_element(control, element)
    }
    fn template_contents(
        &self,
        control: &mut ParseControl,
        element: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.active()?.template_contents(control, element)
    }
    fn insert(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
    ) -> Result<(), SessionError> {
        self.active_mut()?.insert(control, parent, node, before)
    }
    fn remove(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(), SessionError> {
        self.active_mut()?.remove(control, node)
    }
    fn freeze(&mut self, control: &mut ParseControl) -> Result<FrozenArena, SessionError> {
        let document = self.active_mut()?.prepare_freeze(control)?;
        let SessionStorage {
            control: owner,
            identity,
            identity_allocation,
            records,
            records_allocation,
            children_allocations,
            tickets_allocation,
            usage,
        } = self
            .storage
            .take()
            .expect("successful preparation retains active storage");
        let arena = Arena {
            identity,
            records,
            usage,
        };
        let (arena, arena_allocation) = match control.share_owned(arena) {
            Ok(result) => result,
            Err((stop, arena)) => {
                self.storage = Some(SessionStorage {
                    control: owner,
                    identity: arena.identity,
                    identity_allocation,
                    records: arena.records,
                    records_allocation,
                    children_allocations,
                    tickets_allocation,
                    usage: arena.usage,
                });
                return Err(stop.into());
            }
        };
        let view = if document {
            FrozenView::Document(HtmlDocument { arena })
        } else {
            FrozenView::Fragment(HtmlFragment { arena, root: 0 })
        };
        Ok(FrozenArena {
            storage: Some(FrozenStorage {
                view,
                control: owner,
                arena_allocation,
                identity_allocation,
                records_allocation,
                children_allocations,
                tickets_allocation,
            }),
        })
    }
    fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        cleanup_owner(control, &self.active()?.control)?;
        self.storage
            .take()
            .expect("verified active owner retains storage")
            .release(control);
        Ok(())
    }
}

impl FrozenArena {
    fn document(&self) -> Option<&HtmlDocument> {
        match &self.storage.as_ref()?.view {
            FrozenView::Document(view) => Some(view),
            _ => None,
        }
    }
    fn fragment(&self) -> Option<&HtmlFragment> {
        match &self.storage.as_ref()?.view {
            FrozenView::Fragment(view) => Some(view),
            _ => None,
        }
    }
    fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        cleanup_owner(control, &storage.control)?;
        let FrozenStorage {
            view,
            control: owner,
            arena_allocation,
            identity_allocation,
            records_allocation,
            children_allocations,
            tickets_allocation,
        } = self
            .storage
            .take()
            .expect("verified frozen owner retains storage");
        let document = matches!(view, FrozenView::Document(_));
        let arena = match view {
            FrozenView::Document(view) => view.arena,
            FrozenView::Fragment(view) => view.arena,
        };
        let arena = match control.release_shared(arena, arena_allocation) {
            Ok(arena) => arena,
            Err(error) => {
                let (stop, arena, arena_allocation) = error.into_parts();
                let view = if document {
                    FrozenView::Document(HtmlDocument { arena })
                } else {
                    FrozenView::Fragment(HtmlFragment { arena, root: 0 })
                };
                self.storage = Some(FrozenStorage {
                    view,
                    control: owner,
                    arena_allocation,
                    identity_allocation,
                    records_allocation,
                    children_allocations,
                    tickets_allocation,
                });
                return Err(stop.into());
            }
        };
        release_parts(
            control,
            &owner,
            arena,
            records_allocation,
            children_allocations,
            tickets_allocation,
            identity_allocation,
        );
        Ok(())
    }
}

fn release_vector<T>(
    control: &mut ParseControl,
    vector: Vec<T>,
    allocation: VectorCapacity<T>,
) -> Result<(), SessionError> {
    control.release_vector(vector, allocation).map_err(|error| {
        let (stop, _, _) = error.into_parts();
        SessionError::Stop(stop)
    })
}

/// Disposal consumes the actual initialized scratch entries, independently of
/// their reserved capacity. A terminal stop permits only this ownership cleanup.
fn release_order(
    control: &mut ParseControl,
    owner: &ControlIdentity,
    order: Vec<usize>,
    allocation: VectorCapacity<usize>,
) -> Result<(), SessionError> {
    let entries = order.len() as u64;
    release_vector(control, order, allocation)?;
    if control.stop().is_none() && control.release(Resource::AuxiliaryEntries, entries).is_ok() {
        return Ok(());
    }
    control.cleanup_release(owner, Resource::AuxiliaryEntries, entries)?;
    Ok(())
}
fn release_parts(
    control: &mut ParseControl,
    owner: &ControlIdentity,
    mut arena: Arena,
    records_allocation: VectorCapacity<Record>,
    mut children_allocations: Vec<VectorCapacity<usize>>,
    tickets_allocation: VectorCapacity<VectorCapacity<usize>>,
    identity_allocation: SharedAllocation<Identity>,
) {
    for record in arena.records.iter_mut().rev() {
        let allocation = children_allocations
            .pop()
            .expect("each actual record owns a child-vector ticket");
        release_vector(control, mem::take(&mut record.children), allocation)
            .expect("private record retains its exact child-vector capacity ticket and control");
    }
    // Release drops association IDs before reclaiming the identity shared payload.
    release_vector(control, arena.records, records_allocation)
        .expect("private session retains its exact record-vector capacity ticket and control");
    release_vector(control, children_allocations, tickets_allocation)
        .expect("private session retains its exact ticket-vector capacity ticket and control");
    for (resource, units) in [
        (Resource::ManagedEdges, arena.usage.edges as u64),
        (Resource::ManagedAttributes, arena.usage.attributes as u64),
        (Resource::ManagedUtf16Units, arena.usage.utf16_units as u64),
        (Resource::AuxiliaryEntries, arena.usage.nodes as u64),
    ] {
        if control.stop().is_none() && control.release(resource, units).is_ok() {
            continue;
        }
        control
            .cleanup_release(owner, resource, units)
            .expect("private session's admitted logical storage has not been released elsewhere");
    }
    // Identity has a zero-sized known payload. Public IDs may retain its actual
    // Arc header, which is explicitly excluded from measured payload capacity.
    // The controller checks the exact ticket; this makes no deallocation claim.
    if control
        .release_zero_sized_shared(arena.identity, identity_allocation)
        .is_err()
    {
        panic!("private session retains the exact zero-payload identity ticket and control");
    }
}

fn cleanup_owner(control: &mut ParseControl, owner: &ControlIdentity) -> Result<(), SessionError> {
    if control.stop().is_some() {
        control.cleanup_release(owner, Resource::AuxiliaryEntries, 0)?;
    } else if let Err(stop) = control.require_identity(owner) {
        // A cancellation can latch during the normal identity checkpoint. The
        // terminal cleanup check still verifies ownership before any consumption.
        control
            .cleanup_release(owner, Resource::AuxiliaryEntries, 0)
            .map_err(|_| SessionError::Stop(stop))?;
    }
    Ok(())
}

fn exact_ascii(
    control: &mut ParseControl,
    value: Option<&DocumentString>,
    expected: &str,
) -> Result<bool, SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    let Some(value) = value else { return Ok(false) };
    if value.len() != expected.len() {
        return Ok(false);
    }
    for (index, byte) in expected.bytes().enumerate() {
        control.charge(WorkCategory::InspectByte, 3)?;
        if value.code_units()[index] != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn name_units(control: &mut ParseControl, name: &QualifiedName) -> Result<usize, SessionError> {
    control.charge(WorkCategory::RecordVisit, 4)?;
    let units = name
        .local_name()
        .len()
        .checked_add(name.namespace().uri().map_or(0, DocumentString::len))
        .and_then(|sum| sum.checked_add(name.prefix().map_or(0, DocumentString::len)));
    match units {
        Some(units) => Ok(units),
        None => control
            .representation(Resource::ManagedUtf16Units)
            .map_err(Into::into),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Attribute;
    use crate::parser::control::{Cancellation, ParseLimits};

    fn limits() -> ParseLimits {
        ParseLimits {
            source_units: 1_000,
            buffered_utf16_units: 1_000,
            token_attributes: 1_000,
            auxiliary_entries: 1_000,
            managed_nodes: 1_000,
            managed_edges: 1_000,
            managed_attributes: 1_000,
            managed_utf16_units: 100_000,
            source_map_entries: 1_000,
            source_map_payload_units: 1_000,
            diagnostics: 1_000,
            diagnostic_payload_units: 1_000,
            context_effects: 1_000,
            open_elements: 1_000,
            tree_depth: 1_000,
            managed_capacity_bytes: 1_000_000,
            peak_transient_bytes: 2_000_000,
            atomic_layout_bytes: 1_000_000,
            work_credits: 10_000_000,
        }
    }
    fn control() -> ParseControl {
        ParseControl::new(limits(), Cancellation::new()).unwrap()
    }
    fn element(namespace: Namespace, local: &str) -> HtmlElement {
        HtmlElement {
            name: QualifiedName::new(namespace, None, local),
            attributes: OrderedAttributes::new(Vec::new()).unwrap(),
        }
    }
    fn html(local: &str) -> HtmlElement {
        element(Namespace::html(), local)
    }
    fn frozen(mut session: ParserArenaSession, control: &mut ParseControl) -> FrozenArena {
        match session.freeze(control) {
            Ok(value) => value,
            Err(failure) => panic!("freeze failed: {:?}", failure),
        }
    }
    fn locals(fragment: &HtmlFragment) -> Vec<String> {
        fragment
            .root()
            .children()
            .map(|id| {
                let HtmlNodeKind::Element(element) = fragment.node(&id).unwrap().kind() else {
                    panic!("expected element");
                };
                element.name().local_name().to_utf8().unwrap()
            })
            .collect()
    }

    #[test]
    fn detached_allocations_and_ordered_moves_keep_permanent_nodes_and_exact_views() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let a = session.allocate_element(&mut control, html("a")).unwrap();
        let b = session.allocate_element(&mut control, html("b")).unwrap();
        let c = session.allocate_element(&mut control, html("c")).unwrap();
        for node in [&a, &b, &c] {
            session.insert(&mut control, &root, node, None).unwrap();
        }
        session.insert(&mut control, &root, &a, Some(&c)).unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [b.index, a.index, c.index]
        );
        session.insert(&mut control, &root, &a, Some(&a)).unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [b.index, a.index, c.index]
        );
        session.insert(&mut control, &root, &c, Some(&b)).unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [c.index, b.index, a.index]
        );
        session.remove(&mut control, &b).unwrap();
        assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 4);
        assert_eq!(control.usage().resource(Resource::ManagedEdges).current, 2);
        assert_eq!(
            control.usage().resource(Resource::ManagedEdges).high_water,
            3
        );
        let mut output = frozen(session, &mut control);
        let fragment = output.fragment().unwrap();
        assert_eq!(locals(fragment), ["c", "a"]);
        assert_eq!(fragment.usage().nodes(), 4);
        assert_eq!(fragment.usage().edges(), 2);
        let detached_id = fragment.arena.id(b.index);
        assert!(fragment.node(&detached_id).unwrap().parent().is_none());
        assert_eq!(fragment.arena.records[b.index].depth, 0);
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 4);
        assert_eq!(control.usage().resource(Resource::ManagedEdges).current, 0);
        assert!(
            control
                .usage()
                .resource(Resource::ManagedCapacityBytes)
                .high_water
                > 0
        );
    }

    #[test]
    fn paired_template_contents_are_separate_and_association_cycles_are_rejected() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let template = session
            .allocate_element(&mut control, html("template"))
            .unwrap();
        let contents = session
            .template_contents(&mut control, &template)
            .unwrap()
            .unwrap();
        let child = session.allocate_element(&mut control, html("x")).unwrap();
        session
            .insert(&mut control, &root, &template, None)
            .unwrap();
        session
            .insert(&mut control, &contents, &child, None)
            .unwrap();
        assert_eq!(
            session.insert(&mut control, &child, &template, None),
            Err(SessionError::Cycle)
        );
        assert_eq!(
            session.insert(&mut control, &template, &child, None),
            Err(SessionError::TemplateContentsRequired)
        );
        assert_eq!(
            session.insert(&mut control, &root, &contents, None),
            Err(SessionError::AssociationMove)
        );
        let mut output = frozen(session, &mut control);
        let fragment = output.fragment().unwrap();
        assert_eq!(fragment.usage().nodes(), 4);
        assert_eq!(fragment.usage().edges(), 3);
        assert_eq!(fragment.usage().depth(), 3);
        let template_id = fragment.root().children().next().unwrap();
        let template_node = fragment.node(&template_id).unwrap();
        assert_eq!(template_node.children().len(), 0);
        let contents_id = template_node.template_contents().unwrap();
        assert_eq!(
            fragment.node(&contents_id).unwrap().fragment_owner(),
            Some(&HtmlFragmentOwner::Template(template_id))
        );
        assert!(fragment.node(&contents_id).unwrap().parent().is_none());
        assert_eq!(
            fragment
                .fragment(&contents_id)
                .unwrap()
                .unwrap()
                .root()
                .children()
                .len(),
            1
        );
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn foreign_template_and_namespaced_attributes_keep_preexisting_exact_slots() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let payload = HtmlElement {
            name: QualifiedName::new(
                Namespace::none(),
                Some(DocumentString::from("p")),
                DocumentString::from_utf16(vec![0xD800, 32]),
            ),
            attributes: OrderedAttributes::new(vec![Attribute::new(
                QualifiedName::new(Namespace::none(), None, "v"),
                DocumentString::from_utf16(vec![0xDC00]),
            )])
            .unwrap(),
        };
        let backing = payload.name().local_name().code_units().as_ptr();
        let raw = session.allocate_element(&mut control, payload).unwrap();
        let foreign = session
            .allocate_element(&mut control, element(Namespace::svg(), "template"))
            .unwrap();
        assert!(
            session
                .template_contents(&mut control, &foreign)
                .unwrap()
                .is_none()
        );
        session.insert(&mut control, &root, &raw, None).unwrap();
        session.insert(&mut control, &root, &foreign, None).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(Resource::ManagedAttributes)
                .current,
            1
        );
        // 2 local + 1 prefix + 1 attribute local + 1 attribute value; SVG URI
        // http://www.w3.org/2000/svg (26) plus template (8).
        assert_eq!(
            control
                .usage()
                .resource(Resource::ManagedUtf16Units)
                .current,
            39
        );
        let mut output = frozen(session, &mut control);
        let fragment = output.fragment().unwrap();
        let id = fragment.root().children().next().unwrap();
        let HtmlNodeKind::Element(element) = fragment.node(&id).unwrap().kind() else {
            panic!("expected element")
        };
        assert_eq!(element.name().local_name().code_units(), [0xD800, 32]);
        assert_eq!(element.name().local_name().code_units().as_ptr(), backing);
        assert_eq!(
            element.attributes().as_slice()[0].value().code_units(),
            [0xDC00]
        );
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn node_and_depth_cap_failures_leave_topology_unmodified_and_cleanup_releases_capacity() {
        let mut selected = limits();
        selected.managed_nodes = 2;
        let mut control = ParseControl::new(selected, Cancellation::new()).unwrap();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        assert!(matches!(
            session.allocate_element(&mut control, html("template")),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: Resource::ManagedNodes,
                ..
            }))
        ));
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(session.active().unwrap().records[0].children, []);
        assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 1);
        assert!(session.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);

        let mut selected = limits();
        selected.tree_depth = 2;
        let mut control = ParseControl::new(selected, Cancellation::new()).unwrap();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let template = session
            .allocate_element(&mut control, html("template"))
            .unwrap();
        let contents = session
            .template_contents(&mut control, &template)
            .unwrap()
            .unwrap();
        let child = session.allocate_element(&mut control, html("x")).unwrap();
        session
            .insert(&mut control, &contents, &child, None)
            .unwrap();
        assert!(matches!(
            session.insert(&mut control, &root, &template, None),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: Resource::TreeDepth,
                ..
            }))
        ));
        assert!(session.active().unwrap().records[0].children.is_empty());
        assert!(
            session.active().unwrap().records[template.index]
                .parent
                .is_none()
        );
        assert_eq!(
            session.active().unwrap().records[contents.index].children,
            [child.index]
        );
        assert!(session.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn foreign_handles_controls_roots_and_wrong_siblings_reject_before_topology_change() {
        let mut owner = control();
        let mut foreign = control();
        let mut session = ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap();
        let mut other = ParserArenaSession::new(&mut foreign, RootKind::Fragment).unwrap();
        let root = session.root(&mut owner).unwrap();
        let foreign_root = other.root(&mut foreign).unwrap();
        let a = session.allocate_element(&mut owner, html("a")).unwrap();
        let b = session.allocate_element(&mut owner, html("b")).unwrap();
        assert_eq!(
            session.insert(&mut owner, &root, &foreign_root, None),
            Err(SessionError::ForeignNode)
        );
        assert_eq!(
            session.insert(&mut owner, &a, &root, None),
            Err(SessionError::RootMove)
        );
        assert_eq!(
            session.insert(&mut owner, &root, &a, Some(&b)),
            Err(SessionError::WrongSibling)
        );
        assert!(session.active().unwrap().records[0].children.is_empty());
        let before = foreign.usage().clone();
        assert!(matches!(
            session.insert(&mut foreign, &root, &a, None),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(
            foreign.usage().actual_capacity_bytes,
            before.actual_capacity_bytes
        );
        assert_eq!(
            foreign.usage().resource(Resource::WorkCredits),
            before.resource(Resource::WorkCredits)
        );
        assert!(other.release(&mut foreign).is_ok());
        assert!(session.release(&mut owner).is_ok());
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
        assert_eq!(foreign.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn failed_freeze_and_foreign_cleanup_retain_owned_tickets_for_recovery() {
        let mut owner = control();
        let mut foreign = control();
        let mut session = ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap();
        assert!(matches!(
            session.release(&mut foreign),
            Err(SessionError::Stop(_))
        ));
        let cancellation = Cancellation::new();
        let mut stopped = ParseControl::new(limits(), cancellation.clone()).unwrap();
        let mut candidate = ParserArenaSession::new(&mut stopped, RootKind::Fragment).unwrap();
        candidate.allocate_element(&mut stopped, html("x")).unwrap();
        cancellation.cancel();
        assert!(matches!(
            candidate.freeze(&mut stopped),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        assert!(candidate.release(&mut stopped).is_ok());
        assert_eq!(stopped.usage().actual_capacity_bytes, 0);
        assert_eq!(stopped.usage().resource(Resource::ManagedNodes).current, 2);
        assert!(session.release(&mut owner).is_ok());
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn frozen_document_uses_existing_mode_and_old_arena_survives_later_session() {
        let mut first = control();
        let mut session = ParserArenaSession::new(
            &mut first,
            RootKind::Document(HtmlDocumentMode::LimitedQuirks),
        )
        .unwrap();
        let root = session.root(&mut first).unwrap();
        let node = session.allocate_element(&mut first, html("html")).unwrap();
        session.insert(&mut first, &root, &node, None).unwrap();
        let mut output = frozen(session, &mut first);
        let old = output.document().unwrap().clone();
        let old_id = old.root().children().next().unwrap();
        assert_eq!(old.mode(), HtmlDocumentMode::LimitedQuirks);
        let mut second = control();
        let mut later = ParserArenaSession::new(&mut second, RootKind::Fragment).unwrap();
        later.allocate_element(&mut second, html("new")).unwrap();
        let mut later = frozen(later, &mut second);
        assert!(later.fragment().unwrap().node(&old_id).is_err());
        assert_eq!(old.root().children().next().unwrap(), old_id);
        assert!(matches!(
            output.release(&mut first),
            Err(SessionError::Stop(_))
        ));
        drop(old);
        assert!(output.release(&mut first).is_ok());
        assert!(later.release(&mut second).is_ok());
        assert_eq!(first.usage().actual_capacity_bytes, 0);
        assert_eq!(second.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn deep_topology_freezes_without_recursive_tree_or_child_depth_cache() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let mut parent = session.root(&mut control).unwrap();
        for _ in 0..48 {
            let child = session.allocate_element(&mut control, html("x")).unwrap();
            session.insert(&mut control, &parent, &child, None).unwrap();
            parent = child;
        }
        let mut output = frozen(session, &mut control);
        assert_eq!(output.fragment().unwrap().usage().depth(), 48);
        assert_eq!(
            output.fragment().unwrap().arena.records[parent.index].depth,
            48
        );
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn cross_parent_move_changes_both_orders_and_backlinks_without_new_nodes_or_edges() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let a = session.allocate_element(&mut control, html("a")).unwrap();
        let b = session.allocate_element(&mut control, html("b")).unwrap();
        let c = session.allocate_element(&mut control, html("c")).unwrap();
        let d = session.allocate_element(&mut control, html("d")).unwrap();
        session.insert(&mut control, &root, &a, None).unwrap();
        session.insert(&mut control, &root, &b, None).unwrap();
        session.insert(&mut control, &a, &c, None).unwrap();
        session.insert(&mut control, &b, &d, None).unwrap();
        session.insert(&mut control, &b, &c, Some(&d)).unwrap();
        assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 5);
        assert_eq!(control.usage().resource(Resource::ManagedEdges).current, 4);
        assert_eq!(
            control.usage().resource(Resource::ManagedEdges).high_water,
            4
        );
        let mut output = frozen(session, &mut control);
        let fragment = output.fragment().unwrap();
        let a_id = fragment.arena.id(a.index);
        let b_id = fragment.arena.id(b.index);
        let c_id = fragment.arena.id(c.index);
        let d_id = fragment.arena.id(d.index);
        assert_eq!(fragment.node(&a_id).unwrap().children().len(), 0);
        assert_eq!(
            fragment.node(&b_id).unwrap().children().collect::<Vec<_>>(),
            [c_id.clone(), d_id]
        );
        assert_eq!(fragment.node(&c_id).unwrap().parent(), Some(b_id));
        assert_eq!(fragment.usage().depth(), 2);
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn edge_preflight_and_zero_physical_capacity_reject_before_actual_record_admission() {
        let mut selected = limits();
        selected.managed_edges = 0;
        let mut control = ParseControl::new(selected, Cancellation::new()).unwrap();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let bytes_before = control.usage().actual_capacity_bytes;
        assert!(matches!(
            session.allocate_element(&mut control, html("template")),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: Resource::ManagedEdges,
                ..
            }))
        ));
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 1);
        assert_eq!(control.usage().actual_capacity_bytes, bytes_before);
        assert!(session.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        let mut selected = limits();
        selected.managed_capacity_bytes = 0;
        let mut control = ParseControl::new(selected, Cancellation::new()).unwrap();
        assert!(matches!(
            ParserArenaSession::new(&mut control, RootKind::Fragment),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: Resource::ManagedCapacityBytes,
                ..
            }))
        ));
        assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 0);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn successful_transfers_consume_each_owner_once_without_failure_time_allocation() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let records_pointer = session.active().unwrap().records.as_ptr();
        let mut output = session
            .freeze(&mut control)
            .unwrap_or_else(|error| panic!("freeze failed: {error:?}"));
        assert_eq!(
            output.fragment().unwrap().arena.records.as_ptr(),
            records_pointer
        );
        let before = control.usage().clone();
        assert!(matches!(
            session.root(&mut control),
            Err(SessionError::Consumed)
        ));
        assert!(matches!(
            session.freeze(&mut control),
            Err(SessionError::Consumed)
        ));
        assert_eq!(session.release(&mut control), Err(SessionError::Consumed));
        assert_eq!(control.usage(), &before);
        output.release(&mut control).unwrap();
        assert!(output.fragment().is_none());
        assert!(output.document().is_none());
        let before = control.usage().clone();
        assert_eq!(output.release(&mut control), Err(SessionError::Consumed));
        assert_eq!(control.usage(), &before);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn foreign_freeze_and_frozen_cleanup_preserve_the_borrowed_original_owner() {
        let mut owner = control();
        let mut foreign = control();
        let mut session = ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap();
        let root = session.root(&mut owner).unwrap();
        let node = session.allocate_element(&mut owner, html("x")).unwrap();
        session.insert(&mut owner, &root, &node, None).unwrap();
        let before = owner.usage().clone();
        assert!(matches!(
            session.freeze(&mut foreign),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(session.active().unwrap().records[0].children, [node.index]);
        assert_eq!(owner.usage(), &before);
        let mut output = session
            .freeze(&mut owner)
            .unwrap_or_else(|error| panic!("freeze failed: {error:?}"));
        let before = owner.usage().clone();
        assert!(matches!(
            output.release(&mut foreign),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(locals(output.fragment().unwrap()), ["x"]);
        assert_eq!(owner.usage(), &before);
        output.release(&mut owner).unwrap();
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn final_shared_payload_cap_stop_restores_actual_vectors_and_all_tickets() {
        let mut selected = limits();
        // A root owns one actual record slot and one actual child-vector ticket;
        // its empty child vector and zero-sized identity add no known payload.
        let root_capacity = mem::size_of::<Record>() + mem::size_of::<VectorCapacity<usize>>();
        selected.managed_capacity_bytes = root_capacity as u64;
        let mut control = ParseControl::new(selected, Cancellation::new()).unwrap();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let records_pointer = session.active().unwrap().records.as_ptr();
        assert!(
            matches!(session.freeze(&mut control), Err(SessionError::Stop(ParseStop::LimitExceeded {
            resource: Resource::ManagedCapacityBytes,
            current,
            requested_delta,
            ..
        })) if current == root_capacity as u64 && requested_delta == mem::size_of::<Arena>() as u64)
        );
        assert_eq!(session.active().unwrap().records.as_ptr(), records_pointer);
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(control.usage().actual_capacity_bytes, root_capacity as u64);
        session.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 1);
    }

    #[test]
    fn empty_end_and_before_sibling_preparations_charge_each_inserted_index_copy() {
        let word = mem::size_of::<usize>() as u64;
        let vector_records =
            (mem::size_of::<Vec<usize>>() + mem::size_of::<VectorCapacity<usize>>()) as u64;
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let a = session
            .allocate_element(&mut control, element(Namespace::none(), "a"))
            .unwrap();
        let before = control.usage().work(WorkCategory::CopyByte);
        let storage = session.active().unwrap();
        let (order, allocation) = storage
            .child_order(&mut control, 0, usize::MAX, Some((a.index, None)))
            .unwrap();
        // Actual Vec/ticket record initialization, capacity field, inserted index.
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte) - before,
            vector_records + 2 * word
        );
        assert_eq!(order, [a.index]);
        assert_eq!(
            control.usage().resource(Resource::AuxiliaryEntries).current,
            3
        );
        release_order(&mut control, &storage.control, order, allocation).unwrap();
        assert_eq!(
            control.usage().resource(Resource::AuxiliaryEntries).current,
            2
        );
        session.insert(&mut control, &root, &a, None).unwrap();
        let b = session
            .allocate_element(&mut control, element(Namespace::none(), "b"))
            .unwrap();
        let before = control.usage().work(WorkCategory::CopyByte);
        let storage = session.active().unwrap();
        let (order, allocation) = storage
            .child_order(&mut control, 0, usize::MAX, Some((b.index, Some(a.index))))
            .unwrap();
        // The existing child contributes a separate word; insertion contributes
        // another word even though that existing-child loop is already charged.
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte) - before,
            vector_records + 3 * word
        );
        assert_eq!(order, [b.index, a.index]);
        assert_eq!(
            control.usage().resource(Resource::AuxiliaryEntries).current,
            5
        );
        release_order(&mut control, &storage.control, order, allocation).unwrap();
        assert_eq!(
            control.usage().resource(Resource::AuxiliaryEntries).current,
            3
        );
        session.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn inserted_index_copy_work_boundary_stops_before_any_scratch_slot_is_initialized() {
        let mut selected = limits();
        let work_limit = selected.work_credits;
        selected.auxiliary_entries = 3;
        let mut control = ParseControl::new(selected, Cancellation::new()).unwrap();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let a = session
            .allocate_element(&mut control, element(Namespace::none(), "a"))
            .unwrap();
        let word = mem::size_of::<usize>() as u64;
        let records =
            (mem::size_of::<Vec<usize>>() + mem::size_of::<VectorCapacity<usize>>()) as u64;
        // Before the inserted copy: new Vec/ticket has 2 writes + record bytes;
        // parent visit costs 1; reserve has 2 visits, one word ReserveByte,
        // provider call, capacity write and one word CopyByte. Total: records+7+2*word.
        let prefix = records + 7 + 2 * word;
        let before = control.usage().resource(Resource::WorkCredits).current;
        control
            .charge(
                WorkCategory::StateStep,
                work_limit - before - prefix - (word - 1),
            )
            .unwrap();
        let copy_before = control.usage().work(WorkCategory::CopyByte);
        let capacity_before = control.usage().actual_capacity_bytes;
        let storage = session.active().unwrap();
        assert!(
            matches!(storage.child_order(&mut control, 0, usize::MAX, Some((a.index, None))),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: Resource::WorkCredits,
                requested_delta,
                ..
            })) if requested_delta == word)
        );
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte) - copy_before,
            records + word
        );
        assert_eq!(
            control.usage().resource(Resource::AuxiliaryEntries).current,
            2
        );
        assert_eq!(
            control
                .usage()
                .resource(Resource::AuxiliaryEntries)
                .high_water,
            2
        );
        assert_eq!(control.usage().actual_capacity_bytes, capacity_before);
        assert!(control.usage().actual_capacity_high_water > capacity_before);
        assert!(storage.records[0].children.is_empty());
        let stop = control.stop();
        let work = control.usage().resource(Resource::WorkCredits);
        session.release(&mut control).unwrap();
        assert_eq!(control.stop(), stop);
        assert_eq!(control.usage().resource(Resource::WorkCredits), work);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn one_scratch_slot_exact_limit_succeeds_and_one_short_preserves_topology() {
        for (auxiliary_limit, succeeds) in [(3, true), (2, false)] {
            let mut selected = limits();
            selected.auxiliary_entries = auxiliary_limit;
            let mut control = ParseControl::new(selected, Cancellation::new()).unwrap();
            let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
            let root = session.root(&mut control).unwrap();
            let child = session
                .allocate_element(&mut control, element(Namespace::none(), "x"))
                .unwrap();
            let result = session.insert(&mut control, &root, &child, None);
            if succeeds {
                result.unwrap();
                assert_eq!(session.active().unwrap().records[0].children, [child.index]);
                assert_eq!(control.usage().resource(Resource::ManagedEdges).current, 1);
                assert_eq!(
                    control
                        .usage()
                        .resource(Resource::AuxiliaryEntries)
                        .high_water,
                    3
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: Resource::AuxiliaryEntries,
                        current: 2,
                        requested_delta: 1,
                        ..
                    }))
                ));
                assert!(session.active().unwrap().records[0].children.is_empty());
                assert!(
                    session.active().unwrap().records[child.index]
                        .parent
                        .is_none()
                );
                assert_eq!(control.usage().resource(Resource::ManagedEdges).current, 0);
            }
            assert_eq!(
                control.usage().resource(Resource::AuxiliaryEntries).current,
                2
            );
            session.release(&mut control).unwrap();
            assert_eq!(
                control.usage().resource(Resource::AuxiliaryEntries).current,
                0
            );
            assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 2);
            assert_eq!(control.usage().actual_capacity_bytes, 0);
        }
    }

    #[test]
    fn simultaneous_cross_parent_orders_require_both_live_scratch_sequences() {
        for (auxiliary_limit, succeeds) in [(9, true), (8, false)] {
            let mut selected = limits();
            selected.auxiliary_entries = auxiliary_limit;
            let mut control = ParseControl::new(selected, Cancellation::new()).unwrap();
            let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
            let root = session.root(&mut control).unwrap();
            let mut nodes = Vec::new();
            for name in ["a", "b", "c", "d", "e"] {
                nodes.push(
                    session
                        .allocate_element(&mut control, element(Namespace::none(), name))
                        .unwrap(),
                );
            }
            let [a, b, c, d, e] = nodes.as_slice() else {
                panic!("expected five fixture handles")
            };
            session.insert(&mut control, &root, a, None).unwrap();
            session.insert(&mut control, &root, b, None).unwrap();
            session.insert(&mut control, a, c, None).unwrap();
            session.insert(&mut control, a, e, None).unwrap();
            session.insert(&mut control, b, d, None).unwrap();
            let capacity_before = control.usage().actual_capacity_bytes;
            let result = session.insert(&mut control, b, c, Some(d));
            if succeeds {
                result.unwrap();
                assert_eq!(
                    session.active().unwrap().records[a.index].children,
                    [e.index]
                );
                assert_eq!(
                    session.active().unwrap().records[b.index].children,
                    [c.index, d.index]
                );
                assert_eq!(
                    control
                        .usage()
                        .resource(Resource::AuxiliaryEntries)
                        .high_water,
                    9
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: Resource::AuxiliaryEntries,
                        current: 8,
                        requested_delta: 1,
                        ..
                    }))
                ));
                assert_eq!(
                    session.active().unwrap().records[a.index].children,
                    [c.index, e.index]
                );
                assert_eq!(
                    session.active().unwrap().records[b.index].children,
                    [d.index]
                );
                assert_eq!(control.usage().actual_capacity_bytes, capacity_before);
                assert!(control.usage().actual_capacity_high_water > capacity_before);
            }
            assert_eq!(
                control.usage().resource(Resource::AuxiliaryEntries).current,
                6
            );
            assert_eq!(control.usage().resource(Resource::ManagedEdges).current, 5);
            assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 6);
            let stop = control.stop();
            let work = control.usage().resource(Resource::WorkCredits);
            session.release(&mut control).unwrap();
            assert_eq!(control.stop(), stop);
            assert_eq!(control.usage().resource(Resource::WorkCredits), work);
            assert_eq!(
                control.usage().resource(Resource::AuxiliaryEntries).current,
                0
            );
            assert_eq!(control.usage().actual_capacity_bytes, 0);
        }
    }

    #[test]
    fn partially_initialized_order_cap_stop_releases_only_its_one_admitted_entry() {
        let mut selected = limits();
        selected.auxiliary_entries = 5;
        let mut control = ParseControl::new(selected, Cancellation::new()).unwrap();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let a = session
            .allocate_element(&mut control, element(Namespace::none(), "a"))
            .unwrap();
        session.insert(&mut control, &root, &a, None).unwrap();
        let b = session
            .allocate_element(&mut control, element(Namespace::none(), "b"))
            .unwrap();
        session.insert(&mut control, &root, &b, None).unwrap();
        let c = session
            .allocate_element(&mut control, element(Namespace::none(), "c"))
            .unwrap();
        let capacity_before = control.usage().actual_capacity_bytes;
        assert!(matches!(
            session.active().unwrap().child_order(
                &mut control,
                0,
                usize::MAX,
                Some((c.index, None))
            ),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: Resource::AuxiliaryEntries,
                current: 5,
                requested_delta: 1,
                ..
            }))
        ));
        assert_eq!(
            control.usage().resource(Resource::AuxiliaryEntries).current,
            4
        );
        assert_eq!(
            control
                .usage()
                .resource(Resource::AuxiliaryEntries)
                .high_water,
            5
        );
        assert_eq!(control.usage().actual_capacity_bytes, capacity_before);
        assert_eq!(control.usage().resource(Resource::ManagedEdges).current, 2);
        assert_eq!(
            session.active().unwrap().records[0].children,
            [a.index, b.index]
        );
        assert!(session.active().unwrap().records[c.index].parent.is_none());
        let stop = control.stop();
        let work = control.usage().resource(Resource::WorkCredits);
        session.release(&mut control).unwrap();
        assert_eq!(control.stop(), stop);
        assert_eq!(control.usage().resource(Resource::WorkCredits), work);
        assert_eq!(control.usage().resource(Resource::ManagedNodes).current, 4);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
}

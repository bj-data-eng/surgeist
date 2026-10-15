//! DOM3071 slot algorithms over the canonical immutable/staged owner.
use crate::store::{check_cancel, reserve};
use crate::{
    DocumentId, DocumentString, DomError, DomNode, DomNodeKind, DomResource, DomSnapshot, DomStamp,
    NodeId,
};
use std::sync::{Arc, atomic::AtomicBool};

pub(crate) struct TreeNode<'a, K> {
    pub kind: &'a DomNodeKind,
    pub parent: Option<K>,
    pub children: &'a [K],
    pub shadow_root: Option<K>,
    pub host: Option<K>,
    pub manual: &'a [K],
}
/// Two real phases of the same owner use one selected DOM query algorithm.
pub(crate) trait Tree {
    type Key: Copy + Eq;
    fn get(&self, key: Self::Key) -> Result<TreeNode<'_, Self::Key>, DomError>;
}
impl Tree for DomSnapshot {
    type Key = NodeId;
    fn get(&self, key: NodeId) -> Result<TreeNode<'_, NodeId>, DomError> {
        let n = self.node(key)?.node;
        Ok(TreeNode {
            kind: &n.kind,
            parent: n.parent,
            children: &n.children,
            shadow_root: n.shadow_root,
            host: n.host,
            manual: &n.manual_nodes,
        })
    }
}
pub(crate) fn is_slot(kind: &DomNodeKind) -> bool {
    matches!(kind, DomNodeKind::Element(e) if e.is_slot())
}
pub(crate) fn is_slottable(kind: &DomNodeKind) -> bool {
    matches!(kind, DomNodeKind::Element(_) | DomNodeKind::Text(_))
}
pub(crate) fn name<'a>(kind: &'a DomNodeKind, attr: &str) -> Option<&'a DocumentString> {
    match kind {
        DomNodeKind::Element(e) => e
            .attributes()
            .iter()
            .find(|a| {
                a.name().namespace().uri().is_none()
                    && a.name()
                        .local_name()
                        .code_units()
                        .iter()
                        .copied()
                        .eq(attr.encode_utf16())
            })
            .map(|a| a.value()),
        _ => None,
    }
}
fn same_name(a: Option<&DocumentString>, b: Option<&DocumentString>) -> bool {
    a.map_or(&[][..], DocumentString::code_units) == b.map_or(&[][..], DocumentString::code_units)
}
pub(crate) fn root<T: Tree>(
    tree: &T,
    mut key: T::Key,
    cancel: &AtomicBool,
) -> Result<T::Key, DomError> {
    loop {
        check_cancel(cancel)?;
        match tree.get(key)?.parent {
            Some(p) => key = p,
            None => return Ok(key),
        }
    }
}
/// Stackless ordinary inclusive tree order; hosted roots are separate trees.
pub(crate) fn next<T: Tree>(
    tree: &T,
    boundary: T::Key,
    key: T::Key,
    cancel: &AtomicBool,
) -> Result<Option<T::Key>, DomError> {
    next_admitted(tree, boundary, key, &mut || check_cancel(cancel))
}
pub(crate) fn next_admitted<T: Tree>(
    tree: &T,
    boundary: T::Key,
    mut key: T::Key,
    admit: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<Option<T::Key>, DomError> {
    admit()?;
    if let Some(child) = tree.get(key)?.children.first() {
        return Ok(Some(*child));
    }
    loop {
        admit()?;
        if key == boundary {
            return Ok(None);
        }
        let parent = tree.get(key)?.parent.expect("ordinary descendant parent");
        let children = tree.get(parent)?.children;
        for (index, child) in children.iter().enumerate() {
            admit()?;
            if *child == key {
                if let Some(next) = children.get(index + 1) {
                    return Ok(Some(*next));
                }
                break;
            }
        }
        key = parent;
    }
}
pub(crate) fn find_slot<T: Tree>(
    tree: &T,
    node: T::Key,
    open: bool,
    cancel: &AtomicBool,
) -> Result<Option<T::Key>, DomError> {
    check_cancel(cancel)?;
    let n = tree.get(node)?;
    if !is_slottable(n.kind) {
        return Err(DomError::WrongNodeKind);
    }
    let Some(parent) = n.parent else {
        return Ok(None);
    };
    let Some(root) = tree.get(parent)?.shadow_root else {
        return Ok(None);
    };
    let DomNodeKind::ShadowRoot(state) = tree.get(root)?.kind else {
        unreachable!("hosted root")
    };
    if open && state.mode() == crate::HtmlShadowRootMode::Closed {
        return Ok(None);
    }
    let manual = state.slot_assignment() == crate::HtmlSlotAssignment::Manual;
    let mut current = Some(root);
    while let Some(key) = current {
        check_cancel(cancel)?;
        let candidate = tree.get(key)?;
        if is_slot(candidate.kind) {
            let matches = if manual {
                let mut found = false;
                for value in candidate.manual {
                    check_cancel(cancel)?;
                    if *value == node {
                        found = true;
                        break;
                    }
                }
                found
            } else {
                same_name(name(candidate.kind, "name"), name(n.kind, "slot"))
            };
            if matches {
                return Ok(Some(key));
            }
        }
        current = next(tree, root, key, cancel)?;
    }
    Ok(None)
}
pub(crate) fn push<T>(
    out: &mut Vec<T>,
    value: T,
    limit: usize,
    resource: DomResource,
) -> Result<(), DomError> {
    if out.len() >= limit {
        return Err(DomError::ResourceLimit { resource, limit });
    }
    reserve(out, 1)?;
    out.push(value);
    Ok(())
}
pub(crate) fn slottables<T: Tree>(
    tree: &T,
    slot: T::Key,
    limit: usize,
    cancel: &AtomicBool,
) -> Result<Vec<T::Key>, DomError> {
    check_cancel(cancel)?;
    if !is_slot(tree.get(slot)?.kind) {
        return Err(DomError::SlotInapplicable);
    }
    let r = root(tree, slot, cancel)?;
    let rn = tree.get(r)?;
    let DomNodeKind::ShadowRoot(state) = rn.kind else {
        return Ok(Vec::new());
    };
    let host = rn.host.expect("shadow host");
    let mut out = Vec::new();
    if state.slot_assignment() == crate::HtmlSlotAssignment::Manual {
        for &node in tree.get(slot)?.manual {
            check_cancel(cancel)?;
            if tree.get(node)?.parent == Some(host) {
                push(&mut out, node, limit, DomResource::SlotQueryOutput)?;
            }
        }
    } else {
        for &node in tree.get(host)?.children {
            check_cancel(cancel)?;
            if is_slottable(tree.get(node)?.kind)
                && find_slot(tree, node, false, cancel)? == Some(slot)
            {
                push(&mut out, node, limit, DomResource::SlotQueryOutput)?;
            }
        }
    }
    Ok(out)
}

/// Logical result/scratch reference admission, independent of allocator RSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSlotQueryLimits {
    output: usize,
    scratch: usize,
}
impl DomSlotQueryLimits {
    pub fn new(output_nodes: usize, scratch_references: usize) -> Result<Self, DomError> {
        for (n, r, width) in [
            (
                output_nodes,
                DomResource::SlotQueryOutput,
                std::mem::size_of::<(NodeId, Option<NodeId>)>(),
            ),
            (
                scratch_references,
                DomResource::SlotQueryScratch,
                std::mem::size_of::<(std::vec::IntoIter<NodeId>, usize)>(),
            ),
        ] {
            if n > isize::MAX as usize / width {
                return Err(DomError::InvalidLimit(r));
            }
        }
        Ok(Self {
            output: output_nodes,
            scratch: scratch_references,
        })
    }
    pub fn output_nodes(self) -> usize {
        self.output
    }
    pub fn scratch_references(self) -> usize {
        self.scratch
    }
}

/// Checked captured HTMLSlotElement; its stored lists can differ from queries.
#[derive(Clone, Copy, Debug)]
pub struct DomSlot<'a> {
    node: DomNode<'a>,
}
impl<'a> DomSlot<'a> {
    pub fn node(self) -> DomNode<'a> {
        self.node
    }
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn manual_assignments(self) -> &'a [NodeId] {
        &self.node.node.manual_nodes
    }
    /// Source assignedNodes(false), not fresh find-slottables.
    pub fn assigned_nodes(self) -> &'a [NodeId] {
        &self.node.node.assigned_nodes
    }
    pub fn assigned_elements(self) -> impl Iterator<Item = NodeId> + 'a {
        self.assigned_nodes().iter().copied().filter(move |id| {
            matches!(
                self.node
                    .snapshot
                    .node(*id)
                    .expect("captured stored assignment")
                    .kind(),
                DomNodeKind::Element(_)
            )
        })
    }
    pub fn find_slottables(
        self,
        limits: DomSlotQueryLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSlotNodes, DomError> {
        let nodes = slottables(self.node.snapshot, self.id(), limits.output, cancel)?;
        Ok(DomSlotNodes {
            source: self.node.snapshot.clone(),
            nodes: nodes.into(),
        })
    }
    /// Source flattened slottables. Each frame and its unconsumed candidate
    /// list is charged; raw manual references are never recursively followed.
    pub fn flattened_slottables(
        self,
        limits: DomSlotQueryLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSlotNodes, DomError> {
        if !matches!(
            self.node
                .snapshot
                .get(root(self.node.snapshot, self.id(), cancel)?)?
                .kind,
            DomNodeKind::ShadowRoot(_)
        ) {
            check_cancel(cancel)?;
            return Ok(DomSlotNodes {
                source: self.node.snapshot.clone(),
                nodes: Vec::new().into(),
            });
        }
        let mut frames: Vec<(std::vec::IntoIter<NodeId>, usize)> = Vec::new();
        let mut pending = Some(self.id());
        let mut retained = 0usize;
        let mut result = Vec::new();
        loop {
            check_cancel(cancel)?;
            if let Some(slot) = pending.take() {
                let available =
                    limits
                        .scratch
                        .checked_sub(retained)
                        .ok_or(DomError::ResourceLimit {
                            resource: DomResource::SlotQueryScratch,
                            limit: limits.scratch,
                        })?;
                let mut nodes = slottables(self.node.snapshot, slot, available, cancel)
                    .map_err(|e| scratch_error(e, limits.scratch))?;
                if nodes.is_empty() {
                    for &child in self.node.snapshot.get(slot)?.children {
                        check_cancel(cancel)?;
                        if is_slottable(self.node.snapshot.get(child)?.kind) {
                            push(&mut nodes, child, available, DomResource::SlotQueryScratch)?;
                        }
                    }
                }
                // A frame itself retains one logical traversal reference.
                let required = retained
                    .checked_add(nodes.len())
                    .and_then(|n| n.checked_add(1))
                    .ok_or(DomError::RepresentationExhausted)?;
                if required > limits.scratch {
                    return Err(DomError::ResourceLimit {
                        resource: DomResource::SlotQueryScratch,
                        limit: limits.scratch,
                    });
                }
                let charge = nodes.len() + 1;
                retained = required;
                reserve(&mut frames, 1)?;
                frames.push((nodes.into_iter(), charge));
            }
            let Some(frame) = frames.last_mut() else {
                break;
            };
            if let Some(node) = frame.0.next() {
                // The frame's allocation remains retained until its pop.
                if is_slot(self.node.snapshot.get(node)?.kind)
                    && matches!(
                        self.node
                            .snapshot
                            .get(root(self.node.snapshot, node, cancel)?)?
                            .kind,
                        DomNodeKind::ShadowRoot(_)
                    )
                {
                    pending = Some(node);
                } else {
                    push(
                        &mut result,
                        node,
                        limits.output,
                        DomResource::SlotQueryOutput,
                    )?;
                }
            } else {
                let (_, charge) = frames.pop().expect("present frame");
                retained -= charge;
            }
        }
        Ok(DomSlotNodes {
            source: self.node.snapshot.clone(),
            nodes: result.into(),
        })
    }
}
fn scratch_error(e: DomError, limit: usize) -> DomError {
    match e {
        DomError::ResourceLimit {
            resource: DomResource::SlotQueryOutput,
            ..
        } => DomError::ResourceLimit {
            resource: DomResource::SlotQueryScratch,
            limit,
        },
        e => e,
    }
}
/// Owned query result retains its exact source even after edits/destruction.
#[derive(Clone, Debug)]
pub struct DomSlotNodes {
    source: DomSnapshot,
    nodes: Arc<[NodeId]>,
}
impl DomSlotNodes {
    pub fn source(&self) -> &DomSnapshot {
        &self.source
    }
    pub fn stamp(&self) -> DomStamp {
        self.source.stamp()
    }
    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }
}
impl<'a> DomNode<'a> {
    pub fn as_slot(self) -> Option<DomSlot<'a>> {
        is_slot(self.kind()).then_some(DomSlot { node: self })
    }
    pub fn manual_assignment(self) -> Result<Option<NodeId>, DomError> {
        if !is_slottable(self.kind()) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(self.node.manual_slot)
    }
    pub fn stored_assigned_slot(self) -> Result<Option<NodeId>, DomError> {
        if !is_slottable(self.kind()) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(self.node.assigned_slot)
    }
    pub fn assigned_slot(self, cancel: &AtomicBool) -> Result<Option<NodeId>, DomError> {
        find_slot(self.snapshot, self.id(), true, cancel)
    }
    pub fn find_slot(self, open: bool, cancel: &AtomicBool) -> Result<Option<NodeId>, DomError> {
        find_slot(self.snapshot, self.id(), open, cancel)
    }
}
impl DomSnapshot {
    pub fn slot(&self, id: NodeId) -> Result<DomSlot<'_>, DomError> {
        self.node(id)?.as_slot().ok_or(DomError::SlotInapplicable)
    }
    pub fn pending_slot_signals(&self) -> &[NodeId] {
        &self.state.signal_slots
    }
    pub fn slot_notification_queued(&self) -> bool {
        self.state.notification_queued
    }
}

/// Exact captured CSS document flat membership. Excluded is known absence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomFlatRelation {
    Excluded,
    Root,
    Parent(NodeId),
}
#[derive(Clone, Debug)]
pub struct DomFlatTree {
    source: DomSnapshot,
    document: DocumentId,
    nodes: Arc<[(NodeId, Option<NodeId>)]>,
}
impl DomFlatTree {
    pub fn source(&self) -> &DomSnapshot {
        &self.source
    }
    pub fn document(&self) -> DocumentId {
        self.document
    }
    /// CSS construction order is breadth-first, not tree preorder.
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().map(|(n, _)| *n)
    }
    pub fn relation(&self, node: NodeId) -> Result<DomFlatRelation, DomError> {
        self.source.node(node)?;
        Ok(match self.nodes.iter().find(|(n, _)| *n == node) {
            Some((_, Some(p))) => DomFlatRelation::Parent(*p),
            Some((_, None)) => DomFlatRelation::Root,
            None => DomFlatRelation::Excluded,
        })
    }
    pub fn children(&self, node: NodeId) -> Result<impl Iterator<Item = NodeId> + '_, DomError> {
        self.source.node(node)?;
        Ok(self
            .nodes
            .iter()
            .filter_map(move |(n, p)| (*p == Some(node)).then_some(*n)))
    }
}
impl DomSnapshot {
    pub fn flat_tree(
        &self,
        document: DocumentId,
        limits: DomSlotQueryLimits,
        cancel: &AtomicBool,
    ) -> Result<DomFlatTree, DomError> {
        check_cancel(cancel)?;
        let mut nodes = Vec::new();
        if let Some(root) = self.document(document)?.document_element() {
            push(
                &mut nodes,
                (root, None),
                limits.output,
                DomResource::SlotQueryOutput,
            )?;
        }
        let mut index = 0;
        while index < nodes.len() {
            check_cancel(cancel)?;
            let key = nodes[index].0;
            index += 1;
            let n = self.get(key)?;
            if let Some(root) = n.shadow_root {
                for &child in self.get(root)?.children {
                    check_cancel(cancel)?;
                    push(
                        &mut nodes,
                        (child, Some(key)),
                        limits.output,
                        DomResource::SlotQueryOutput,
                    )?;
                }
            } else if is_slot(n.kind) {
                let assigned = slottables(self, key, limits.scratch, cancel)
                    .map_err(|e| scratch_error(e, limits.scratch))?;
                let children = if assigned.is_empty() {
                    n.children
                } else {
                    &assigned
                };
                for &child in children {
                    check_cancel(cancel)?;
                    push(
                        &mut nodes,
                        (child, Some(key)),
                        limits.output,
                        DomResource::SlotQueryOutput,
                    )?;
                }
            } else {
                for &child in n.children {
                    check_cancel(cancel)?;
                    push(
                        &mut nodes,
                        (child, Some(key)),
                        limits.output,
                        DomResource::SlotQueryOutput,
                    )?;
                }
            }
        }
        Ok(DomFlatTree {
            source: self.clone(),
            document,
            nodes: nodes.into(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSlotSignalCancellationReason {
    Destroyed,
}
/// Historical target batch from an atomic intrinsic notification checkpoint.
#[derive(Clone, Debug)]
pub struct DomSlotNotifications {
    source: DomSnapshot,
    targets: Arc<[NodeId]>,
    changes: crate::DomChanges,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSlotNotificationTarget {
    Live(NodeId),
    SkippedDestroyedTarget(NodeId),
}
impl DomSlotNotifications {
    pub(crate) fn new(
        source: DomSnapshot,
        targets: Arc<[NodeId]>,
        changes: crate::DomChanges,
    ) -> Self {
        Self {
            source,
            targets,
            changes,
        }
    }
    pub fn changes(&self) -> &crate::DomChanges {
        &self.changes
    }
    pub fn source(&self) -> &DomSnapshot {
        &self.source
    }
    /// Event name slotchange, bubbles=true, composed=false, cancelable=false.
    pub fn targets(&self) -> &[NodeId] {
        &self.targets
    }
    pub fn revalidate(
        &self,
        current: &crate::Dom,
        limits: DomSlotQueryLimits,
        cancel: &AtomicBool,
    ) -> Result<Vec<DomSlotNotificationTarget>, DomError> {
        check_cancel(cancel)?;
        let current = current.snapshot();
        if current.owner() != self.source.owner() {
            return Err(DomError::ForeignOwner);
        }
        let mut result = Vec::new();
        for &id in self.targets.iter() {
            check_cancel(cancel)?;
            let value = match current.node(id) {
                Ok(_) => DomSlotNotificationTarget::Live(id),
                Err(DomError::StaleNode) => DomSlotNotificationTarget::SkippedDestroyedTarget(id),
                Err(e) => return Err(e),
            };
            push(
                &mut result,
                value,
                limits.output,
                DomResource::SlotQueryOutput,
            )?;
        }
        Ok(result)
    }
}

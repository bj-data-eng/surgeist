//! DOM3071 slot algorithms over the canonical immutable/staged owner.
use crate::store::{check_cancel, reserve};
use crate::{
    DocumentId, DocumentString, DomError, DomNode, DomNodeKind, DomResource, DomSnapshot, DomStamp,
    NodeId,
};
use std::sync::{Arc, atomic::AtomicBool};

/// Language facts actually needed by the shared slot queries.
#[derive(Clone, Copy)]
pub(crate) enum TreeKind<'a> {
    Element(&'a crate::DomElement),
    Text,
    ShadowRoot(&'a crate::DomShadowRootState),
    Other,
}
impl<'a> From<DomNodeKind<'a>> for TreeKind<'a> {
    fn from(kind: DomNodeKind<'a>) -> Self {
        match kind {
            DomNodeKind::Element(element) => Self::Element(element),
            DomNodeKind::Text(_) => Self::Text,
            DomNodeKind::ShadowRoot(state) => Self::ShadowRoot(state),
            _ => Self::Other,
        }
    }
}
pub(crate) struct TreeNode<'a, K> {
    pub kind: TreeKind<'a>,
    pub parent: Option<K>,
    pub ordinal: Option<usize>,
    pub children: TreeRelations<'a, K>,
    pub shadow_root: Option<K>,
    pub host: Option<K>,
    pub manual: TreeRelations<'a, K>,
}
/// Borrow the actual relation backing; canonical identities map only on read.
#[derive(Clone, Copy)]
pub(crate) enum TreeRelations<'a, K> {
    Direct(&'a [K]),
    Canonical(&'a [NodeId], fn(NodeId) -> K),
    AliasedCanonical(
        &'a [NodeId],
        fn(NodeId) -> K,
        &'a crate::store::KeyAliases,
        fn(K, &crate::store::KeyAliases) -> K,
    ),
    AliasedStaged(
        &'a [K],
        &'a crate::store::KeyAliases,
        fn(K, &crate::store::KeyAliases) -> K,
    ),
}
impl<K: Copy> TreeRelations<'_, K> {
    pub fn len(self) -> usize {
        match self {
            Self::Direct(values) => values.len(),
            Self::Canonical(values, _) | Self::AliasedCanonical(values, ..) => values.len(),
            Self::AliasedStaged(values, ..) => values.len(),
        }
    }
    pub fn get(self, index: usize) -> Option<K> {
        match self {
            Self::Direct(values) => values.get(index).copied(),
            Self::Canonical(values, map) => values.get(index).copied().map(map),
            Self::AliasedCanonical(values, map, aliases, alias) => values
                .get(index)
                .copied()
                .map(map)
                .map(|key| alias(key, aliases)),
            Self::AliasedStaged(values, aliases, alias) => {
                values.get(index).copied().map(|key| alias(key, aliases))
            }
        }
    }
    pub fn first(self) -> Option<K> {
        self.get(0)
    }
    pub fn iter(self) -> impl DoubleEndedIterator<Item = K> + ExactSizeIterator {
        (0..self.len()).map(move |index| self.get(index).expect("relation extent"))
    }
}
/// Two real phases of the same owner use one selected DOM query algorithm.
pub(crate) trait Tree {
    type Key: Copy + Eq;
    fn get(&self, key: Self::Key) -> Result<TreeNode<'_, Self::Key>, DomError>;
    fn slot_after(
        &self,
        root: Self::Key,
        after: Option<Self::Key>,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Self::Key>, DomError>;
    fn named_slot(
        &self,
        root: Self::Key,
        node: Self::Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Self::Key>, DomError>;
    fn named_light(
        &self,
        slot: Self::Key,
        host: Self::Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
        visit: &mut dyn FnMut(Self::Key) -> Result<(), DomError>,
    ) -> Result<(), DomError>;
    #[cfg(test)]
    fn ordinal_successor_read(&self);
    fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)
    }
}
impl Tree for DomSnapshot {
    type Key = NodeId;
    fn named_slot(
        &self,
        root: NodeId,
        node: NodeId,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<NodeId>, DomError> {
        self.state.slot_named_slot(root, node, work)
    }
    fn named_light(
        &self,
        slot: NodeId,
        host: NodeId,
        work: &mut dyn FnMut() -> Result<(), DomError>,
        visit: &mut dyn FnMut(NodeId) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.state
            .slot_named_light(self.owner(), slot, host, work, visit)
    }
    #[cfg(test)]
    fn ordinal_successor_read(&self) {
        self.state.slot_ordinal_successor_read();
    }
    fn get(&self, key: NodeId) -> Result<TreeNode<'_, NodeId>, DomError> {
        let n = self.node(key)?.node;
        Ok(TreeNode {
            kind: n.kind().into(),
            parent: n.parent(),
            ordinal: n
                .parent()
                .and_then(|parent| self.state.slot_ordinal(key, parent)),
            children: TreeRelations::Direct(n.children()),
            shadow_root: n.shadow_root(),
            host: n.host(),
            manual: TreeRelations::Direct(n.manual_nodes()),
        })
    }
    fn slot_after(
        &self,
        root: NodeId,
        after: Option<NodeId>,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<NodeId>, DomError> {
        self.state.slot_after(root, after, work)
    }
}
pub(crate) fn is_slot(kind: TreeKind<'_>) -> bool {
    matches!(kind, TreeKind::Element(e) if e.is_slot())
}
pub(crate) fn is_slottable(kind: TreeKind<'_>) -> bool {
    matches!(kind, TreeKind::Element(_) | TreeKind::Text)
}
pub(crate) fn name_admitted<'a, E>(
    kind: TreeKind<'a>,
    attr: &str,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<Option<&'a DocumentString>, E> {
    match kind {
        TreeKind::Element(e) => {
            for a in e.attributes().iter() {
                step()?;
                if a.name().namespace_uri().is_some() {
                    continue;
                }
                let units = a.name().local_name().code_units();
                if units.len() != attr.len() {
                    continue;
                }
                let mut equal = true;
                for (actual, expected) in units.iter().zip(attr.bytes()) {
                    step()?;
                    if *actual != u16::from(expected) {
                        equal = false;
                        break;
                    }
                }
                if equal {
                    return Ok(Some(a.value()));
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}
pub(crate) fn same_name(
    a: Option<&DocumentString>,
    b: Option<&DocumentString>,
    step: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    let a = a.map_or(&[][..], DocumentString::code_units);
    let b = b.map_or(&[][..], DocumentString::code_units);
    if a.len() != b.len() {
        return Ok(false);
    }
    for (a, b) in a.iter().zip(b) {
        step()?;
        if a != b {
            return Ok(false);
        }
    }
    Ok(true)
}
pub(crate) fn root<T: Tree>(
    tree: &T,
    key: T::Key,
    cancel: &AtomicBool,
) -> Result<T::Key, DomError> {
    root_admitted(tree, key, &mut || tree.step(cancel))
}
pub(crate) fn root_admitted<T: Tree>(
    tree: &T,
    mut key: T::Key,
    admit: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<T::Key, DomError> {
    loop {
        admit()?;
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
    next_admitted(tree, boundary, key, &mut || tree.step(cancel))
}
pub(crate) fn next_admitted<T: Tree>(
    tree: &T,
    boundary: T::Key,
    mut key: T::Key,
    admit: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<Option<T::Key>, DomError> {
    admit()?;
    if let Some(child) = tree.get(key)?.children.first() {
        return Ok(Some(child));
    }
    loop {
        admit()?;
        if key == boundary {
            return Ok(None);
        }
        let node = tree.get(key)?;
        let parent = node.parent.expect("ordinary descendant parent");
        let children = tree.get(parent)?.children;
        let ordinal = node.ordinal.ok_or(DomError::StaleNode)?;
        #[cfg(test)]
        tree.ordinal_successor_read();
        if children.get(ordinal) != Some(key) {
            return Err(DomError::StaleNode);
        }
        if let Some(next) = ordinal
            .checked_add(1)
            .and_then(|ordinal| children.get(ordinal))
        {
            return Ok(Some(next));
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
    tree.step(cancel)?;
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
    let TreeKind::ShadowRoot(state) = tree.get(root)?.kind else {
        unreachable!("hosted root")
    };
    if open && state.mode() == crate::HtmlShadowRootMode::Closed {
        return Ok(None);
    }
    if state.slot_assignment() != crate::HtmlSlotAssignment::Manual {
        return tree.named_slot(root, node, &mut || tree.step(cancel));
    }
    let mut current = tree.slot_after(root, None, &mut || tree.step(cancel))?;
    while let Some(key) = current {
        tree.step(cancel)?;
        let candidate = tree.get(key)?;
        if is_slot(candidate.kind) {
            for value in candidate.manual.iter() {
                tree.step(cancel)?;
                if value == node {
                    return Ok(Some(key));
                }
            }
        }
        current = tree.slot_after(root, Some(key), &mut || tree.step(cancel))?;
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
    let mut out = Vec::new();
    visit_slottables(tree, slot, cancel, &mut |node| {
        push(&mut out, node, limit, DomResource::SlotQueryOutput)
    })?;
    Ok(out)
}
pub(crate) fn visit_slottables<T: Tree>(
    tree: &T,
    slot: T::Key,
    cancel: &AtomicBool,
    visit: &mut impl FnMut(T::Key) -> Result<(), DomError>,
) -> Result<(), DomError> {
    tree.step(cancel)?;
    if !is_slot(tree.get(slot)?.kind) {
        return Err(DomError::SlotInapplicable);
    }
    let r = root(tree, slot, cancel)?;
    let rn = tree.get(r)?;
    let TreeKind::ShadowRoot(state) = rn.kind else {
        return Ok(());
    };
    let host = rn.host.expect("shadow host");
    if state.slot_assignment() == crate::HtmlSlotAssignment::Manual {
        for node in tree.get(slot)?.manual.iter() {
            tree.step(cancel)?;
            if tree.get(node)?.parent == Some(host) {
                visit(node)?;
            }
        }
    } else {
        tree.named_light(slot, host, &mut || tree.step(cancel), visit)?;
    }
    Ok(())
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
        self.node.node.manual_nodes()
    }
    /// Source assignedNodes(false), not fresh find-slottables.
    pub fn assigned_nodes(self) -> &'a [NodeId] {
        self.node.node.assigned_nodes()
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
            TreeKind::ShadowRoot(_)
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
                    for child in self.node.snapshot.get(slot)?.children.iter() {
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
                        TreeKind::ShadowRoot(_)
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
        is_slot(self.kind().into()).then_some(DomSlot { node: self })
    }
    pub fn manual_assignment(self) -> Result<Option<NodeId>, DomError> {
        if !is_slottable(self.kind().into()) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(self.node.manual_slot())
    }
    pub fn stored_assigned_slot(self) -> Result<Option<NodeId>, DomError> {
        if !is_slottable(self.kind().into()) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(self.node.assigned_slot())
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
                for child in self.get(root)?.children.iter() {
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
                    TreeRelations::Direct(&assigned)
                };
                for child in children.iter() {
                    check_cancel(cancel)?;
                    push(
                        &mut nodes,
                        (child, Some(key)),
                        limits.output,
                        DomResource::SlotQueryOutput,
                    )?;
                }
            } else {
                for child in n.children.iter() {
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

#[cfg(test)]
pub(crate) mod discovery_observation {
    pub(super) use crate::store::scale_metrics::ENABLED;
    use std::{cell::RefCell, marker::PhantomData, rc::Rc};

    #[derive(Clone, Copy, Debug, Default)]
    pub(super) struct Counts {
        pub named_lookups: usize,
        pub host_lookups: usize,
        pub named_candidates: usize,
        pub host_candidates: usize,
        pub max_named_candidates: usize,
        pub max_host_candidates: usize,
    }
    thread_local! {
        static COUNTS: RefCell<Option<Counts>> = const { RefCell::new(None) };
    }
    pub(super) struct Observation(PhantomData<Rc<()>>);
    impl Observation {
        pub fn begin() -> Self {
            crate::store::scale_metrics::require_enabled()
                .expect("requires SURGEIST_DOM_STORAGE_COUNTERS=1");
            COUNTS.with(|counts| {
                let mut counts = counts.borrow_mut();
                assert!(counts.is_none(), "nested Slot discovery observation");
                *counts = Some(Counts::default());
            });
            Self(PhantomData)
        }
        pub fn finish(self) -> Counts {
            COUNTS.with(|counts| counts.borrow_mut().take().expect("active observation"))
        }
    }
    impl Drop for Observation {
        fn drop(&mut self) {
            if ENABLED {
                COUNTS.with(|counts| counts.borrow_mut().take());
            }
        }
    }
    enum Kind {
        Named,
        Host,
    }
    pub(crate) struct Lookup {
        kind: Kind,
        candidates: usize,
    }
    impl Lookup {
        pub fn named() -> Self {
            Self {
                kind: Kind::Named,
                candidates: 0,
            }
        }
        pub fn host() -> Self {
            Self {
                kind: Kind::Host,
                candidates: 0,
            }
        }
        /// A reached candidate from the real producer, independently of the
        /// eventual name match or assignment result.
        pub fn read_candidate(&mut self) {
            if ENABLED {
                self.candidates = self.candidates.checked_add(1).expect("candidate count");
            }
        }
    }
    impl Drop for Lookup {
        fn drop(&mut self) {
            if !ENABLED {
                return;
            }
            COUNTS.with(|counts| {
                let mut counts = counts.borrow_mut();
                let Some(counts) = &mut *counts else { return };
                let (calls, total, maximum) = match self.kind {
                    Kind::Named => (
                        &mut counts.named_lookups,
                        &mut counts.named_candidates,
                        &mut counts.max_named_candidates,
                    ),
                    Kind::Host => (
                        &mut counts.host_lookups,
                        &mut counts.host_candidates,
                        &mut counts.max_host_candidates,
                    ),
                };
                *calls = calls.checked_add(1).expect("lookup count");
                *total = total.checked_add(self.candidates).expect("candidate count");
                *maximum = (*maximum).max(self.candidates);
            });
        }
    }
}

#[cfg(test)]
#[path = "slot/named_discovery_tests.rs"]
mod named_discovery_tests;

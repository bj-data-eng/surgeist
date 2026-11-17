//! Caller-admitted observations using the canonical Slot algorithms.
use super::*;
use std::cell::RefCell;

/// Private invocation adapter for the native owner's DomError callbacks. The
/// first caller error stops the native algorithm immediately and is restored
/// unchanged at the front door. The sentinel never escapes as cancellation,
/// and subsequent callbacks cannot run after a refusal.
struct Caller<E, Step> {
    step: RefCell<Step>,
    failure: RefCell<Option<E>>,
}
impl<E: From<DomError>, Step: FnMut() -> Result<(), E>> Caller<E, Step> {
    fn new(step: Step) -> Self {
        Self {
            step: RefCell::new(step),
            failure: RefCell::new(None),
        }
    }
    fn capture<T>(&self, action: impl FnOnce() -> Result<T, E>) -> Result<T, DomError> {
        if self
            .failure
            .try_borrow()
            .map_err(|_| DomError::StaleNode)?
            .is_some()
        {
            return Err(DomError::Cancelled);
        }
        match action() {
            Ok(value) => Ok(value),
            Err(error) => {
                *self
                    .failure
                    .try_borrow_mut()
                    .map_err(|_| DomError::StaleNode)? = Some(error);
                Err(DomError::Cancelled)
            }
        }
    }
    fn step(&self) -> Result<(), DomError> {
        self.capture(|| {
            let mut step = self
                .step
                .try_borrow_mut()
                .map_err(|_| DomError::StaleNode)?;
            step()
        })
    }
    fn finish<T>(self, result: Result<T, DomError>) -> Result<T, E> {
        match self.failure.into_inner() {
            Some(error) => Err(error),
            None => result.map_err(E::from),
        }
    }
}
struct Read<'a, 'c, E, Step> {
    snapshot: &'a DomSnapshot,
    caller: &'c Caller<E, Step>,
}
impl<E: From<DomError>, Step: FnMut() -> Result<(), E>> Tree for Read<'_, '_, E, Step> {
    type Key = NodeId;
    fn get(&self, key: NodeId) -> Result<TreeNode<'_, NodeId>, DomError> {
        let node = self
            .snapshot
            .state
            .node_with(self.snapshot.owner(), key, &mut || self.caller.step())?;
        let position = self
            .snapshot
            .state
            .ordinary_child_position_with(key, &mut || self.caller.step())?;
        Ok(TreeNode {
            kind: node.kind().into(),
            parent: node.parent(),
            ordinal: position
                .filter(|(parent, _)| Some(*parent) == node.parent())
                .map(|(_, ordinal)| ordinal),
            children: TreeRelations::Direct(node.children()),
            shadow_root: node.shadow_root(),
            host: node.host(),
            manual: TreeRelations::Direct(node.manual_nodes()),
        })
    }
    fn slot_after(
        &self,
        root: NodeId,
        after: Option<NodeId>,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<NodeId>, DomError> {
        self.snapshot.state.slot_after(root, after, work)
    }
    fn named_slot(
        &self,
        root: NodeId,
        node: NodeId,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<NodeId>, DomError> {
        self.snapshot.state.slot_named_slot(root, node, work)
    }
    fn named_light(
        &self,
        slot: NodeId,
        host: NodeId,
        work: &mut dyn FnMut() -> Result<(), DomError>,
        visit: &mut dyn FnMut(NodeId) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.snapshot
            .state
            .slot_named_light(self.snapshot.owner(), slot, host, work, visit)
    }
    fn step(&self, _: &AtomicBool) -> Result<(), DomError> {
        self.caller.step()
    }
    #[cfg(test)]
    fn ordinal_successor_read(&self) {
        self.snapshot.state.slot_ordinal_successor_read();
    }
}

// Allocation precedes its admission in field order, including on early return.
struct Funded<T, Backing> {
    values: Vec<T>,
    _backing: Option<Backing>,
}
impl<T, Backing> Funded<T, Backing> {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            _backing: None,
        }
    }
    fn push<E: From<DomError>, Step: FnMut() -> Result<(), E>>(
        &mut self,
        value: T,
        caller: &Caller<E, Step>,
        reserve: &mut impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<(), DomError> {
        caller.step()?;
        if self.values.len() == self.values.capacity() {
            let capacity = self
                .values
                .capacity()
                .checked_mul(2)
                .ok_or(DomError::RepresentationExhausted)?
                .max(1);
            let bytes = capacity
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?;
            let backing = caller.capture(|| reserve(bytes))?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(capacity)
                .map_err(|_| DomError::AllocationFailed)?;
            if replacement.capacity() != capacity {
                return Err(DomError::AllocationFailed);
            }
            for item in self.values.drain(..) {
                caller.step()?;
                replacement.push(item);
            }
            *self = Self {
                values: replacement,
                _backing: Some(backing),
            };
        }
        self.values.push(value);
        Ok(())
    }
}
struct Frame<Backing> {
    nodes: Funded<NodeId, Backing>,
    cursor: usize,
    references: usize,
}
impl<'a> DomSlot<'a> {
    /// Visit the stored assigned-node sequence, not fresh find-slottables.
    /// Each reference and canonical lookup is admitted before inspection. The
    /// caller owns output and filtering work; its first refusal ends the visit.
    pub fn visit_assigned_nodes_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
        mut visit: impl FnMut(DomNode<'a>) -> Result<(), E>,
    ) -> Result<(), E> {
        let caller = Caller::new(step);
        let result = (|| {
            caller.step()?;
            for &id in self.assigned_nodes() {
                caller.step()?;
                let node = self.node.snapshot.node_with(id, || caller.step())?;
                caller.capture(|| visit(node))?;
            }
            Ok(())
        })();
        caller.finish(result)
    }
    /// Fresh flattening with slottable fallback and nested Slot substitution.
    /// `reserve` admits each complete new scratch allocation while all old
    /// allocations remain funded. Every copied candidate/frame is metered.
    /// Scratch drops before its returned backing. Output allocation belongs to
    /// `visit`; logical output and scratch-reference limits remain enforced here.
    /// The first work, lookup, reservation or visitor refusal returns unchanged;
    /// successful visits form the accepted prefix. `step` owns cancellation.
    pub fn visit_flattened_slottables_with<E: From<DomError>, Backing>(
        self,
        limits: DomSlotQueryLimits,
        step: impl FnMut() -> Result<(), E>,
        mut reserve: impl FnMut(usize) -> Result<Backing, E>,
        mut visit: impl FnMut(DomNode<'a>) -> Result<(), E>,
    ) -> Result<(), E> {
        let caller = Caller::new(step);
        let result = (|| {
            let tree = Read {
                snapshot: self.node.snapshot,
                caller: &caller,
            };
            let cancel = AtomicBool::new(false);
            if !matches!(
                tree.get(root(&tree, self.id(), &cancel)?)?.kind,
                TreeKind::ShadowRoot(_)
            ) {
                return Ok(());
            }
            let mut frames = Funded::<Frame<Backing>, Backing>::new();
            let mut pending = Some(self.id());
            let mut retained = 0usize;
            let mut output = 0usize;
            loop {
                caller.step()?;
                if let Some(slot) = pending.take() {
                    let available = limits
                        .scratch
                        .checked_sub(retained)
                        .and_then(|n| n.checked_sub(1))
                        .ok_or(DomError::ResourceLimit {
                            resource: DomResource::SlotQueryScratch,
                            limit: limits.scratch,
                        })?;
                    let mut nodes = Funded::new();
                    let mut append = |node| {
                        if nodes.values.len() >= available {
                            return Err(DomError::ResourceLimit {
                                resource: DomResource::SlotQueryScratch,
                                limit: limits.scratch,
                            });
                        }
                        nodes.push(node, &caller, &mut reserve)
                    };
                    visit_slottables(&tree, slot, &cancel, &mut append)?;
                    if nodes.values.is_empty() {
                        for child in tree.get(slot)?.children.iter() {
                            caller.step()?;
                            if is_slottable(tree.get(child)?.kind) {
                                if nodes.values.len() >= available {
                                    return Err(DomError::ResourceLimit {
                                        resource: DomResource::SlotQueryScratch,
                                        limit: limits.scratch,
                                    });
                                }
                                nodes.push(child, &caller, &mut reserve)?;
                            }
                        }
                    }
                    let references = nodes
                        .values
                        .len()
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    retained = retained
                        .checked_add(references)
                        .ok_or(DomError::RepresentationExhausted)?;
                    frames.push(
                        Frame {
                            nodes,
                            cursor: 0,
                            references,
                        },
                        &caller,
                        &mut reserve,
                    )?;
                }
                let Some(frame) = frames.values.last_mut() else {
                    return Ok(());
                };
                if let Some(&id) = frame.nodes.values.get(frame.cursor) {
                    caller.step()?;
                    frame.cursor = frame
                        .cursor
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    if is_slot(tree.get(id)?.kind)
                        && matches!(
                            tree.get(root(&tree, id, &cancel)?)?.kind,
                            TreeKind::ShadowRoot(_)
                        )
                    {
                        pending = Some(id);
                    } else {
                        if output >= limits.output {
                            return Err(DomError::ResourceLimit {
                                resource: DomResource::SlotQueryOutput,
                                limit: limits.output,
                            });
                        }
                        let node = self.node.snapshot.node_with(id, || caller.step())?;
                        caller.capture(|| visit(node))?;
                        output = output
                            .checked_add(1)
                            .ok_or(DomError::RepresentationExhausted)?;
                    }
                } else {
                    let frame = frames.values.pop().ok_or(DomError::StaleNode)?;
                    retained = retained
                        .checked_sub(frame.references)
                        .ok_or(DomError::RepresentationExhausted)?;
                }
            }
        })();
        caller.finish(result)
    }
}
impl DomNode<'_> {
    /// Find the fresh assigned Slot. `open` hides assignment through closed
    /// roots. Each reached lookup, association and name comparison is admitted;
    /// `step` owns cancellation and its original first error is preserved.
    pub fn find_slot_with<E: From<DomError>>(
        self,
        open: bool,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<NodeId>, E> {
        let caller = Caller::new(step);
        let tree = Read {
            snapshot: self.snapshot,
            caller: &caller,
        };
        let result = find_slot(&tree, self.id(), open, &AtomicBool::new(false));
        caller.finish(result)
    }
    /// Fresh assignedSlot presentation, excluding closed ShadowRoots.
    pub fn assigned_slot_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<NodeId>, E> {
        self.find_slot_with(true, step)
    }
}

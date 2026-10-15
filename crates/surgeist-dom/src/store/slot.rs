use super::*;
use crate::slot::{self, Tree};

impl Tree for Draft {
    type Key = Key;
    fn get(&self, key: Key) -> Result<slot::TreeNode<'_, Key>, DomError> {
        let n = self.node(key)?;
        Ok(slot::TreeNode {
            kind: &n.kind,
            parent: n.parent,
            children: &n.children,
            shadow_root: n.shadow_root,
            host: n.host,
            manual: &n.manual_nodes,
        })
    }
}
impl State {
    pub(super) fn slot_reference_count(&self) -> usize {
        self.slots
            .iter()
            .filter_map(|s| s.node.as_ref())
            .fold(self.signal_slots.len(), |sum, n| {
                sum.checked_add(n.manual_nodes.len())
                    .and_then(|sum| sum.checked_add(n.assigned_nodes.len()))
                    .expect("admitted reference count")
            })
    }
}
impl Draft {
    fn slot_reference_count(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        self.existing
            .iter()
            .chain(&self.created)
            .filter_map(Option::as_ref)
            .try_fold(self.signal_slots.len(), |sum, n| {
                check_cancel(cancel)?;
                sum.checked_add(n.manual_nodes.len())
                    .and_then(|sum| sum.checked_add(n.assigned_nodes.len()))
                    .ok_or(DomError::RepresentationExhausted)
            })
    }
    fn signal_slot(
        &mut self,
        slot: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if !self.signal_slots.contains(&slot) {
            let count = self
                .slot_reference_count(cancel)?
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            admit(count, limits.slot_references, DomResource::SlotReferences)?;
            reserve(&mut self.signal_slots, 1)?;
            self.signal_slots.push(slot);
        }
        self.queue_slot_marker()?;
        Ok(())
    }
    fn assign_slottables(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let nodes = slot::slottables(self, key, limits.nodes, cancel)?;
        let changed = self.node(key)?.assigned_nodes != nodes;
        let new_signal = changed && !self.signal_slots.contains(&key);
        let required = self
            .slot_reference_count(cancel)?
            .checked_sub(self.node(key)?.assigned_nodes.len())
            .and_then(|n| n.checked_add(nodes.len()))
            .and_then(|n| n.checked_add(usize::from(new_signal)))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            required,
            limits.slot_references,
            DomResource::SlotReferences,
        )?;
        // Admission uses the installed replacement, allowing exact-capacity
        // shrinking changes even though source signals before replacing.
        if changed {
            if new_signal {
                reserve(&mut self.signal_slots, 1)?;
                self.signal_slots.push(key);
            }
            self.queue_slot_marker()?;
        }
        self.node_mut(key).assigned_nodes = nodes;
        for index in 0..self.node(key)?.assigned_nodes.len() {
            check_cancel(cancel)?;
            let node = self.node(key)?.assigned_nodes[index];
            self.node_mut(node).assigned_slot = Some(key);
        }
        Ok(())
    }
    fn assign_tree(
        &mut self,
        root: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut current = Some(root);
        while let Some(key) = current {
            check_cancel(cancel)?;
            if slot::is_slot(&self.node(key)?.kind) {
                self.assign_slottables(key, limits, cancel)?;
            }
            current = slot::next(self, root, key, cancel)?;
        }
        Ok(())
    }
    fn assign_a_slot(
        &mut self,
        node: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(key) = slot::find_slot(self, node, false, cancel)? {
            self.assign_slottables(key, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn assign_manual(
        &mut self,
        target: Key,
        incoming: Vec<NodeOperand>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.assign_manual_admitted(target, incoming, limits, cancel, &mut || {
            check_cancel(cancel)
        })
    }
    fn assign_manual_admitted(
        &mut self,
        target: Key,
        incoming: Vec<NodeOperand>,
        limits: DomLimits,
        cancel: &AtomicBool,
        admit_action: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        use surgeist_html::bounded::{
            HtmlSlotAssignAction as A, HtmlSlotAssignCursor, HtmlSlotAssignReply as R,
        };
        if !slot::is_slot(&self.node(target)?.kind) {
            return Err(DomError::SlotInapplicable);
        }
        let mut nodes = Vec::new();
        reserve(&mut nodes, incoming.len())?;
        for operand in incoming {
            check_cancel(cancel)?;
            let key = self.resolve(operand)?;
            if !slot::is_slottable(&self.node(key)?.kind) {
                return Err(DomError::WrongNodeKind);
            }
            nodes.push(key);
        }
        let mut cursor =
            HtmlSlotAssignCursor::new(self.node(target)?.manual_nodes.len(), nodes.len());
        let mut pending = Vec::new();
        let mut referred = None;
        loop {
            admit_action()?;
            let reply = match cursor.action() {
                A::ClearPreviousManualInverse { previous_ordinal } => {
                    let key = self.node(target)?.manual_nodes[previous_ordinal];
                    self.node_mut(key).manual_slot = None;
                    R::Applied
                }
                A::BeginPendingManualSet => R::Applied,
                A::ReadIncomingManualAssignment { ordinal } => {
                    referred = self.node(nodes[ordinal])?.manual_slot;
                    R::ManualAssignmentPresent(referred.is_some())
                }
                A::RemoveIncomingFromReferredManualSet { ordinal } => {
                    let key = referred.expect("actual preceding read");
                    remove_reference(&mut self.node_mut(key).manual_nodes, nodes[ordinal], cancel)?;
                    R::Applied
                }
                A::SetIncomingManualAssignmentToTarget { ordinal } => {
                    self.node_mut(nodes[ordinal]).manual_slot = Some(target);
                    R::Applied
                }
                A::CompareIncoming {
                    ordinal,
                    previous_ordinal,
                } => R::SameNode(nodes[ordinal] == nodes[previous_ordinal]),
                A::AppendIncomingToPendingSet { ordinal } => {
                    reserve(&mut pending, 1)?;
                    pending.push(nodes[ordinal]);
                    R::Applied
                }
                A::ReplaceTargetManualSet => {
                    let required = self
                        .slot_reference_count(cancel)?
                        .checked_sub(self.node(target)?.manual_nodes.len())
                        .and_then(|n| n.checked_add(pending.len()))
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(
                        required,
                        limits.slot_references,
                        DomResource::SlotReferences,
                    )?;
                    self.node_mut(target).manual_nodes = std::mem::take(&mut pending);
                    R::Applied
                }
                A::AssignSlottablesForTargetRoot => {
                    let root = slot::root(self, target, cancel)?;
                    self.assign_tree(root, limits, cancel)?;
                    R::Applied
                }
                A::Complete => break,
            };
            cursor
                .reply(reply)
                .expect("reply for the actual performed source action");
        }
        Ok(())
    }
    pub(super) fn slot_insertion(
        &mut self,
        node: Key,
        parent: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(root) = self.node(parent)?.shadow_root
            && matches!(&self.node(root)?.kind,DomNodeKind::ShadowRoot(state) if state.slot_assignment()==crate::HtmlSlotAssignment::Named)
            && slot::is_slottable(&self.node(node)?.kind)
        {
            self.assign_a_slot(node, limits, cancel)?;
        }
        self.signal_fallback_parent(parent, limits, cancel)?;
        self.assign_tree(slot::root(self, node, cancel)?, limits, cancel)
    }
    fn signal_fallback_parent(
        &mut self,
        parent: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if slot::is_slot(&self.node(parent)?.kind)
            && self.node(parent)?.assigned_nodes.is_empty()
            && matches!(
                self.node(slot::root(self, parent, cancel)?)?.kind,
                DomNodeKind::ShadowRoot(_)
            )
        {
            self.signal_slot(parent, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn slot_removal(
        &mut self,
        node: Key,
        parent: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(slot) = self.node(node)?.assigned_slot {
            self.assign_slottables(slot, limits, cancel)?;
        }
        self.signal_fallback_parent(parent, limits, cancel)?;
        let mut current = Some(node);
        let mut has_slot = false;
        while let Some(key) = current {
            check_cancel(cancel)?;
            if slot::is_slot(&self.node(key)?.kind) {
                has_slot = true;
                break;
            }
            current = slot::next(self, node, key, cancel)?;
        }
        if has_slot {
            self.assign_tree(slot::root(self, parent, cancel)?, limits, cancel)?;
            self.assign_tree(node, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn slot_attribute(
        &mut self,
        node: Key,
        changed: &crate::QualifiedName,
        old: Option<DocumentString>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if changed.namespace().uri().is_some() {
            return Ok(());
        }
        let attribute = changed.local_name();
        let is_name = attribute
            .code_units()
            .iter()
            .copied()
            .eq("name".encode_utf16());
        let is_slot = attribute
            .code_units()
            .iter()
            .copied()
            .eq("slot".encode_utf16());
        if !is_name && !is_slot {
            return Ok(());
        }
        let new = slot::name(
            &self.node(node)?.kind,
            if is_name { "name" } else { "slot" },
        );
        if old.as_ref().map_or(&[][..], DocumentString::code_units)
            == new.map_or(&[][..], DocumentString::code_units)
        {
            return Ok(());
        }
        if is_name && slot::is_slot(&self.node(node)?.kind) {
            self.assign_tree(slot::root(self, node, cancel)?, limits, cancel)?;
        }
        if is_slot {
            if let Some(slot) = self.node(node)?.assigned_slot {
                self.assign_slottables(slot, limits, cancel)?;
            }
            self.assign_a_slot(node, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn slot_retirement(
        &mut self,
        key: Key,
        _limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // Explicit Rust retirement, not source remove/adopt: clean exact Key
        // relations before publication can reuse an existing arena slot.
        for node in self
            .existing
            .iter_mut()
            .chain(&mut self.created)
            .filter_map(Option::as_mut)
        {
            check_cancel(cancel)?;
            remove_reference(&mut node.manual_nodes, key, cancel)?;
            if node.manual_slot == Some(key) {
                node.manual_slot = None;
            }
            if node.assigned_slot == Some(key) {
                node.assigned_slot = None;
            }
        }
        if self.signal_slots.contains(&key) {
            remove_reference(&mut self.signal_slots, key, cancel)?;
            if let Key::Existing(slot) = key {
                reserve(&mut self.cancelled_signals, 1)?;
                self.cancelled_signals.push(NodeId {
                    owner: self.store,
                    slot,
                    generation: self.generations[slot],
                });
            }
        }
        Ok(())
    }
}
impl Dom {
    pub(super) fn prepare_slot_checkpoint(
        &self,
        limits: crate::DomSlotQueryLimits,
        cancel: &AtomicBool,
    ) -> Result<(Prepared, Arc<[NodeId]>), DomError> {
        check_cancel(cancel)?;
        admit(
            self.state.signal_slots.len(),
            limits.output_nodes(),
            DomResource::SlotQueryOutput,
        )?;
        let mut targets = Vec::new();
        reserve(&mut targets, self.state.signal_slots.len())?;
        for &target in &self.state.signal_slots {
            check_cancel(cancel)?;
            targets.push(target);
        }
        let mut draft = Draft::new(self, DomBatch::new().owner, cancel)?;
        assert!(matches!(
            draft.native_tasks.first(),
            Some(select::Pending::Slots)
        ));
        draft.native_tasks.remove(0);
        draft.signal_slots.clear();
        draft.notification_queued = false;
        let prepared = draft.finish(self, cancel)?;
        // Vec-to-Arc conversion allocates. Complete it while the drained
        // state is private, before publish's final cancellation boundary.
        Ok((prepared, targets.into()))
    }
}

fn remove_reference<T: Copy + Eq>(
    values: &mut Vec<T>,
    target: T,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let mut kept = 0;
    for index in 0..values.len() {
        check_cancel(cancel)?;
        if values[index] != target {
            values[kept] = values[index];
            kept += 1;
        }
    }
    values.truncate(kept);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    fn fixture() -> (Dom, NodeId, NodeId) {
        let mut dom = Dom::new(DomLimits::new(16, 4096, 32, 32, 32, 8, 65536).unwrap()).unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let s = b.create_element_ns(&d, crate::Namespace::html(), "slot");
        let x = b.create_text(&d, "X");
        let c = dom.apply(b, &AtomicBool::new(false)).unwrap();
        let s = c.created(&s).unwrap().unwrap();
        let x = c.created(&x).unwrap().unwrap();
        (dom, s, x)
    }
    #[test]
    fn actual_cursor_cancellation_after_partial_inverse_writes_never_publishes() {
        let (dom, s, x) = fixture();
        let before = dom.snapshot();
        let usage = dom.usage();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        let target = draft.resolve(s.into()).unwrap();
        let mut actions = 0;
        let result = draft.assign_manual_admitted(
            target,
            vec![x.into()],
            dom.limits(),
            &cancel,
            &mut || {
                actions += 1;
                if actions == 5 {
                    cancel.store(true, Ordering::Release);
                }
                check_cancel(&cancel)
            },
        );
        assert_eq!(result, Err(DomError::Cancelled));
        assert_eq!(actions, 5);
        // Read/Set actions have run on the real staged owner before refusal.
        assert_eq!(
            draft
                .node(draft.resolve(x.into()).unwrap())
                .unwrap()
                .manual_slot,
            Some(target)
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    #[test]
    fn prepared_slot_publication_cancellation_and_revision_exhaustion_roll_back_every_relation() {
        let (mut dom, s, x) = fixture();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.assign_slot(s, vec![x.into()]);
        let prepared = dom.prepare(b, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            dom.publish(prepared, &AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        dom.revision = DomRevision(u64::MAX);
        let mut b = DomBatch::new();
        b.assign_slot(s, vec![x.into()]);
        assert_eq!(
            dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
    }
    #[test]
    fn prepared_signal_drain_cancel_preserves_the_exact_pending_queue_and_history() {
        let (mut dom, s, _) = fixture();
        let cancel = AtomicBool::new(false);
        let history_from = dom.snapshot().stamp();
        let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        let key = draft.resolve(s.into()).unwrap();
        draft.signal_slot(key, dom.limits(), &cancel).unwrap();
        let prepared = draft.finish(&dom, &cancel).unwrap();
        dom.publish(prepared, &cancel).unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let history = dom.changes_since(history_from).unwrap();
        let (prepared, targets) = dom
            .prepare_slot_checkpoint(crate::DomSlotQueryLimits::new(1, 0).unwrap(), &cancel)
            .unwrap();
        // The actual checkpoint has finished owning its immutable output,
        // including the Arc allocation, while the queue remains published.
        assert_eq!(&*targets, &[s]);
        assert_eq!(dom.snapshot().pending_slot_signals(), &[s]);
        assert!(dom.snapshot().slot_notification_queued());
        assert_eq!(*dom.state, *before.state);
        cancel.store(true, Ordering::Release);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(&*targets, &[s]);
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        let current_history = dom.changes_since(history_from).unwrap();
        assert_eq!(current_history.from(), history.from());
        assert_eq!(current_history.to(), history.to());
        assert_eq!(current_history.changes(), history.changes());
    }
}

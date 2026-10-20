use super::*;
use crate::slot::{self, Tree};
use transaction::RelationField;

impl<'a> From<transaction::KindRead<'a>> for slot::TreeKind<'a> {
    fn from(kind: transaction::KindRead<'a>) -> Self {
        match kind {
            transaction::KindRead::Element(element) => Self::Element(element.language()),
            transaction::KindRead::Text(_) => Self::Text,
            transaction::KindRead::ShadowRoot(state) => Self::ShadowRoot(state),
            _ => Self::Other,
        }
    }
}

fn tree_relations(
    view: transaction::SequenceView<'_, NodeId, Key>,
) -> slot::TreeRelations<'_, Key> {
    match view {
        transaction::SequenceView::Empty => slot::TreeRelations::Direct(&[]),
        transaction::SequenceView::Canonical(values, map) => {
            slot::TreeRelations::Canonical(values, map)
        }
        transaction::SequenceView::Staged(values) => slot::TreeRelations::Direct(values),
        transaction::SequenceView::AliasedCanonical(values, map, aliases, alias) => {
            slot::TreeRelations::AliasedCanonical(values, map, aliases, alias)
        }
        transaction::SequenceView::AliasedStaged(values, aliases, alias) => {
            slot::TreeRelations::AliasedStaged(values, aliases, alias)
        }
    }
}

impl Tree for Draft {
    type Key = Key;
    fn named_slot(
        &self,
        root: Key,
        node: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.slot_names
            .named_slot(self.live_key(root), self.live_key(node), work)
    }
    fn named_light(
        &self,
        slot: Key,
        host: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
        visit: &mut dyn FnMut(Key) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.slot_names.visit_light(
            self.live_key(slot),
            self.live_key(host),
            self.order_view(),
            work,
            visit,
        )
    }
    #[cfg(test)]
    fn ordinal_successor_read(&self) {
        ordinal_successor_read();
    }
    fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.source_step(cancel)
    }
    fn get(&self, key: Key) -> Result<slot::TreeNode<'_, Key>, DomError> {
        let n = self.node(key)?;
        Ok(slot::TreeNode {
            kind: n.kind().into(),
            parent: n.parent(),
            ordinal: n.parent().and_then(|parent| {
                self.slot_index
                    .position(self.live_key(key))
                    .filter(|position| position.parent == self.live_key(parent))
                    .map(|position| position.ordinal)
            }),
            children: tree_relations(n.children()),
            shadow_root: n.shadow_root(),
            host: n.host(),
            manual: tree_relations(n.manual_nodes()),
        })
    }
    fn slot_after(
        &self,
        root: Key,
        after: Option<Key>,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        let root = self.live_key(root);
        match after {
            Some(key) => self.slot_index.next(root, self.live_key(key), work),
            None => self.slot_index.first(root, work),
        }
    }
}
impl State {
    pub(crate) fn slot_named_slot(
        &self,
        root: NodeId,
        node: NodeId,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<NodeId>, DomError> {
        self.slot_index
            .named
            .named_slot(Key::Existing(root), Key::Existing(node), work)?
            .map(|key| match key {
                Key::Existing(id) => Ok(id),
                Key::Created(_) => Err(DomError::UnavailableCreation),
            })
            .transpose()
    }
    pub(crate) fn slot_named_light(
        &self,
        owner: DomOwnerId,
        slot: NodeId,
        host: NodeId,
        work: &mut dyn FnMut() -> Result<(), DomError>,
        visit: &mut dyn FnMut(NodeId) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.slot_index.named.visit_light(
            Key::Existing(slot),
            Key::Existing(host),
            transaction::ordered_members::OrderView::state(self, owner),
            work,
            &mut |key| match key {
                Key::Existing(id) => visit(id),
                Key::Created(_) => Err(DomError::UnavailableCreation),
            },
        )
    }
    #[cfg(test)]
    pub(crate) fn slot_ordinal_successor_read(&self) {
        ordinal_successor_read();
    }
    pub(crate) fn slot_ordinal(&self, key: NodeId, parent: NodeId) -> Option<usize> {
        self.slot_index
            .position(Key::Existing(key))
            .filter(|position| position.parent == Key::Existing(parent))
            .map(|position| position.ordinal)
    }
    pub(crate) fn slot_after(
        &self,
        root: NodeId,
        after: Option<NodeId>,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<NodeId>, DomError> {
        let next = match after {
            Some(key) => self
                .slot_index
                .next(Key::Existing(root), Key::Existing(key), work)?,
            None => self.slot_index.first(Key::Existing(root), work)?,
        };
        Ok(next.map(|key| match key {
            Key::Existing(id) => id,
            Key::Created(_) => unreachable!("published Slot identity"),
        }))
    }
    pub(super) fn slot_reference_count(&self) -> usize {
        self.counts
            .slot_references
            .checked_add(self.signal_slots.len())
            .expect("admitted reference count")
    }
}

#[cfg(test)]
fn ordinal_successor_read() {
    if scale_metrics::ENABLED {
        scale_metrics::record(scale_metrics::Event::SlotOrdinalSuccessorRead);
    }
}
impl Draft {
    fn slot_signal_contains(&self, key: Key, cancel: &AtomicBool) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        Ok(self.signal_slots.target_count(self.live_key(key)) != 0)
    }
    fn slot_signal_push(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        self.queue_signals_insert(self.signal_slots.len(), &[self.live_key(key)], cancel)
    }
    fn slot_reference_count(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        self.source_step(cancel)?;
        self.counts
            .slot_references
            .checked_add(self.signal_slots.len())
            .ok_or(DomError::RepresentationExhausted)
    }
    fn signal_slot(
        &mut self,
        slot: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !self.slot_signal_contains(slot, cancel)? {
            let count = self
                .slot_reference_count(cancel)?
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            admit(count, limits.slot_references, DomResource::SlotReferences)?;
            self.slot_signal_push(slot, cancel)?;
        }
        self.queue_slot_marker(cancel)?;
        Ok(())
    }
    fn assign_slottables(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::SlotAssignmentInvoked);
        }
        #[cfg(test)]
        assignment_observation::record(key);
        let mut nodes = self.effect_vec();
        slot::visit_slottables(self, key, cancel, &mut |node| {
            admit(
                nodes
                    .len()
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?,
                limits.nodes,
                DomResource::SlotQueryOutput,
            )?;
            self.select_reserve(&mut nodes, 1, cancel)?;
            nodes.push(node);
            Ok(())
        })?;
        for _ in &*nodes {
            self.source_step(cancel)?;
        }
        for _ in self.node(key)?.assigned_nodes().iter() {
            self.source_step(cancel)?;
        }
        let changed = !self
            .node(key)?
            .assigned_nodes()
            .iter()
            .eq(nodes.iter().copied());
        let new_signal = changed && !self.slot_signal_contains(key, cancel)?;
        let required = self
            .slot_reference_count(cancel)?
            .checked_sub(self.node(key)?.assigned_nodes().len())
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
                self.slot_signal_push(key, cancel)?;
            }
            self.queue_slot_marker(cancel)?;
        }
        self.relations_replace_owned(
            key,
            RelationField::AssignedNodes,
            nodes.into_values(),
            cancel,
        )?;
        for index in 0..self.node(key)?.assigned_nodes().len() {
            self.source_step(cancel)?;
            let node = self
                .node(key)?
                .assigned_nodes()
                .get(index)
                .expect("assigned node");
            let mut slottable = self.slottable_fields_mut(node, cancel)?;
            slottable.assigned_slot = Some(key);
            slottable.commit(cancel)?;
        }
        Ok(())
    }
    pub(super) fn assign_tree(
        &mut self,
        root: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut current = self.slot_after(root, None, &mut || self.source_step(cancel))?;
        while let Some(key) = current {
            self.source_step(cancel)?;
            if slot::is_slot(self.node(key)?.kind().into()) {
                self.assign_slottables(key, limits, cancel)?;
            }
            current = self.slot_after(root, Some(key), &mut || self.source_step(cancel))?;
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
        incoming: &[NodeOperand],
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
        incoming: &[NodeOperand],
        limits: DomLimits,
        cancel: &AtomicBool,
        admit_action: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        use surgeist_html::bounded::{
            HtmlSlotAssignAction as A, HtmlSlotAssignCursor, HtmlSlotAssignReply as R,
        };
        self.source_step(cancel)?;
        if !slot::is_slot(self.node(target)?.kind().into()) {
            return Err(DomError::SlotInapplicable);
        }
        let mut nodes = self.effect_vec();
        self.select_reserve(&mut nodes, incoming.len(), cancel)?;
        for operand in incoming {
            self.source_step(cancel)?;
            let key = self.resolve(operand.clone())?;
            if !slot::is_slottable(self.node(key)?.kind().into()) {
                return Err(DomError::WrongNodeKind);
            }
            nodes.push(key);
        }
        let mut cursor =
            HtmlSlotAssignCursor::new(self.node(target)?.manual_nodes().len(), nodes.len());
        let mut pending = self.effect_vec();
        let mut referred = None;
        loop {
            admit_action()?;
            self.source_step(cancel)?;
            let reply = match cursor.action() {
                A::ClearPreviousManualInverse { previous_ordinal } => {
                    let key = self
                        .node(target)?
                        .manual_nodes()
                        .get(previous_ordinal)
                        .expect("previous manual node");
                    let mut slottable = self.slottable_fields_mut(key, cancel)?;
                    slottable.manual_slot = None;
                    slottable.commit(cancel)?;
                    R::Applied
                }
                A::BeginPendingManualSet => R::Applied,
                A::ReadIncomingManualAssignment { ordinal } => {
                    referred = self.node(nodes[ordinal])?.manual_slot();
                    R::ManualAssignmentPresent(referred.is_some())
                }
                A::RemoveIncomingFromReferredManualSet { ordinal } => {
                    let key = referred.expect("actual preceding read");
                    for _ in self.node(key)?.manual_nodes().iter() {
                        self.source_step(cancel)?;
                    }
                    self.relations_retain(
                        key,
                        RelationField::ManualNodes,
                        |value| value != nodes[ordinal],
                        cancel,
                    )?;
                    R::Applied
                }
                A::SetIncomingManualAssignmentToTarget { ordinal } => {
                    let mut slottable = self.slottable_fields_mut(nodes[ordinal], cancel)?;
                    slottable.manual_slot = Some(target);
                    slottable.commit(cancel)?;
                    R::Applied
                }
                A::CompareIncoming {
                    ordinal,
                    previous_ordinal,
                } => R::SameNode(nodes[ordinal] == nodes[previous_ordinal]),
                A::AppendIncomingToPendingSet { ordinal } => {
                    self.select_reserve(&mut pending, 1, cancel)?;
                    pending.push(nodes[ordinal]);
                    R::Applied
                }
                A::ReplaceTargetManualSet => {
                    let required = self
                        .slot_reference_count(cancel)?
                        .checked_sub(self.node(target)?.manual_nodes().len())
                        .and_then(|n| n.checked_add(pending.len()))
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(
                        required,
                        limits.slot_references,
                        DomResource::SlotReferences,
                    )?;
                    self.relations_replace_owned(
                        target,
                        RelationField::ManualNodes,
                        pending.into_values(),
                        cancel,
                    )?;
                    pending = self.effect_vec();
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
        if let Some(root) = self.node(parent)?.shadow_root()
            && matches!(self.node(root)?.kind(),transaction::KindRead::ShadowRoot(state) if state.slot_assignment()==crate::HtmlSlotAssignment::Named)
            && slot::is_slottable(self.node(node)?.kind().into())
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
        if slot::is_slot(self.node(parent)?.kind().into())
            && self.node(parent)?.assigned_nodes().is_empty()
            && matches!(
                self.node(slot::root(self, parent, cancel)?)?.kind(),
                transaction::KindRead::ShadowRoot(_)
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
        if let Some(slot) = self.node(node)?.assigned_slot() {
            self.assign_slottables(slot, limits, cancel)?;
        }
        self.signal_fallback_parent(parent, limits, cancel)?;
        if self
            .slot_after(node, None, &mut || self.source_step(cancel))?
            .is_some()
        {
            self.assign_tree(slot::root(self, parent, cancel)?, limits, cancel)?;
            self.assign_tree(node, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn slot_attribute(
        &mut self,
        node: Key,
        changed: &crate::DomQualifiedName,
        old: Option<&DocumentString>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if changed.namespace_uri().is_some() {
            return Ok(());
        }
        let attribute = changed.local_name();
        let is_name = self.source_ascii(attribute, "name", cancel)?;
        let is_slot = self.source_ascii(attribute, "slot", cancel)?;
        if !is_name && !is_slot {
            return Ok(());
        }
        let new = slot::name_admitted(
            self.node(node)?.kind().into(),
            if is_name { "name" } else { "slot" },
            &mut || self.source_step(cancel),
        )?;
        if slot::same_name(old, new, &mut || self.source_step(cancel))? {
            return Ok(());
        }
        if is_name && slot::is_slot(self.node(node)?.kind().into()) {
            self.assign_tree(slot::root(self, node, cancel)?, limits, cancel)?;
        }
        if is_slot {
            if let Some(slot) = self.node(node)?.assigned_slot() {
                self.assign_slottables(slot, limits, cancel)?;
            }
            self.assign_a_slot(node, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn slot_retirement(
        &mut self,
        key: Key,
        document: Key,
        _limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // Explicit Rust retirement, not source remove/adopt: clean exact Key
        // relations before publication can reuse an existing arena slot.
        let affected = self.inbound_keys(key, cancel)?;
        for owner in affected.iter().copied() {
            self.source_step(cancel)?;
            let Ok(node) = self.node(owner) else { continue };
            let mut manual = false;
            for value in node.manual_nodes().iter() {
                self.source_step(cancel)?;
                if value == key {
                    manual = true;
                    break;
                }
            }
            let clear_manual = node.manual_slot() == Some(key);
            let clear_assigned = node.assigned_slot() == Some(key);
            if manual {
                self.relations_retain(
                    owner,
                    RelationField::ManualNodes,
                    |value| value != key,
                    cancel,
                )?;
            }
            if clear_manual || clear_assigned {
                let mut slottable = self.slottable_fields_mut(owner, cancel)?;
                if clear_manual {
                    slottable.manual_slot = None;
                }
                if clear_assigned {
                    slottable.assigned_slot = None;
                }
                slottable.commit(cancel)?;
            }
        }
        if self.slot_signal_contains(key, cancel)? {
            let signal_key = self.live_key(key);
            for _ in self.signal_slots.iter() {
                self.source_step(cancel)?;
            }
            let index = self
                .signal_slots
                .iter()
                .position(|value| value == signal_key)
                .expect("queued slot signal");
            self.queue_retired_signal_remove(index, signal_key, document, cancel)?;
            self.source_step(cancel)?;
            if let Some(id) = self.published_id(key) {
                self.queue_cancelled_signals_insert(self.cancelled_signals.len(), &[id], cancel)?;
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
        draft.queue_native_remove(0, 1, cancel)?;
        draft.queue_signals_clear(cancel)?;
        draft.notification_queued = false;
        let prepared = draft.finish(self, cancel)?;
        // Vec-to-Arc conversion allocates. Complete it while the drained
        // state is private, before publish's final cancellation boundary.
        Ok((prepared, targets.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    fn fixture() -> (Dom, NodeId, NodeId) {
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                4096,
                32,
                32,
                32,
                8,
                65536,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
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
    fn resolved_source_slot_bank_is_charged_before_the_cursor_reads_it() {
        let (dom, s, x) = fixture();
        let cancel = AtomicBool::new(false);
        let batch = DomBatch::new();
        let (mut draft, _) = Draft::new_source(
            &dom,
            batch.owner,
            &batch.commands,
            crate::DomSourceLimits::new(1_000_000, 100_000, 1_000_000, 32, 128, 128, 1),
            &cancel,
        )
        .unwrap();
        let target = draft.resolve(s.into()).unwrap();
        let live = draft.select_temporary_bytes.clone();
        let mut entered = false;
        draft
            .assign_manual_admitted(target, &[x.into()], dom_limits(), &cancel, &mut || {
                if !entered {
                    entered = true;
                    assert!(
                        live.load(Ordering::Relaxed) >= size_of::<Key>(),
                        "actual resolved input bank is already allocated at first cursor entry"
                    );
                }
                Ok(())
            })
            .unwrap();
        assert!(entered);
        assert_eq!(live.load(Ordering::Relaxed), 0);
    }
    fn dom_limits() -> DomLimits {
        DomLimits::new(
            16,
            4096,
            32,
            32,
            32,
            8,
            65536,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn source_slot_resolution_cannot_run_past_its_spent_work_allowance() {
        let (dom, s, x) = fixture();
        let cancel = AtomicBool::new(false);
        let limits = |work| crate::DomSourceLimits::new(1_000_000, 100_000, work, 32, 128, 128, 1);
        let batch = DomBatch::new();
        let (mut draft, _) = Draft::new_source(
            &dom,
            batch.owner,
            &batch.commands,
            limits(1_000_000),
            &cancel,
        )
        .unwrap();
        let target = draft.resolve(s.into()).unwrap();
        let spent = draft.source_work.load(Ordering::Relaxed);
        draft.source_limits.as_mut().unwrap().work = spent;
        assert_eq!(
            draft.assign_manual(target, &[x.into()], dom_limits(), &cancel),
            Err(DomError::SourceWorkLimit { limit: spent })
        );
        assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
        assert!(draft.node(target).unwrap().manual_nodes().is_empty());
        assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
        assert!(
            dom.state
                .node(dom.owner, x)
                .unwrap()
                .manual_slot()
                .is_none()
        );
    }
    #[test]
    fn source_slot_attribute_hook_admits_work_before_reading_its_name() {
        let (mut dom, s, _) = fixture();
        let mut setup = DomBatch::new();
        setup.set_attribute(s, "name", "retained");
        dom.apply(setup, &AtomicBool::new(false)).unwrap();
        let name = dom
            .snapshot()
            .node(s)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()[0]
            .name()
            .clone();
        let cancel = AtomicBool::new(false);
        let limits = |work| crate::DomSourceLimits::new(1_000_000, 100_000, work, 32, 128, 128, 1);
        let batch = DomBatch::new();
        let (mut draft, _) = Draft::new_source(
            &dom,
            batch.owner,
            &batch.commands,
            limits(1_000_000),
            &cancel,
        )
        .unwrap();
        let key = draft.resolve(s.into()).unwrap();
        let spent = draft.source_work.load(Ordering::Relaxed);
        draft.source_limits.as_mut().unwrap().work = spent;
        assert_eq!(
            draft.slot_attribute(key, &name, None, dom.limits(), &cancel),
            Err(DomError::SourceWorkLimit { limit: spent })
        );
        assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    }
    #[test]
    fn source_slot_retirement_admits_its_actual_cancellation_bank_before_allocating() {
        let (mut dom, s, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut setup = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        let key = setup.resolve(s.into()).unwrap();
        setup.signal_slot(key, dom.limits(), &cancel).unwrap();
        let prepared = setup.finish(&dom, &cancel).unwrap();
        dom.publish(prepared, &cancel).unwrap();
        assert_eq!(dom.snapshot().pending_slot_signals(), &[s]);
        let limits =
            |bytes| crate::DomSourceLimits::new(bytes, 100_000, 1_000_000, 32, 128, 128, 1);
        let batch = DomBatch::new();
        let (mut draft, _) = Draft::new_source(
            &dom,
            batch.owner,
            &batch.commands,
            limits(1_000_000),
            &cancel,
        )
        .unwrap();
        let retained = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
        draft.source_limits.as_mut().unwrap().retained_bytes = retained;
        let key = draft.resolve(s.into()).unwrap();
        let document = draft.node(key).unwrap().document();
        assert_eq!(
            draft.slot_retirement(key, document, dom.limits(), &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: retained
            })
        );
        assert!(draft.cancelled_signals.is_empty());
        assert_eq!(dom.snapshot().pending_slot_signals(), &[s]);
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
        let result =
            draft.assign_manual_admitted(target, &[x.into()], dom.limits(), &cancel, &mut || {
                actions += 1;
                if actions == 5 {
                    cancel.store(true, Ordering::Release);
                }
                check_cancel(&cancel)
            });
        assert_eq!(result, Err(DomError::Cancelled));
        assert_eq!(actions, 5);
        // Read/Set actions have run on the real staged owner before refusal.
        assert_eq!(
            draft
                .node(draft.resolve(x.into()).unwrap())
                .unwrap()
                .manual_slot(),
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

#[cfg(test)]
#[path = "slot/slot_discovery_work_tests.rs"]
mod slot_discovery_work_tests;

#[cfg(test)]
pub(in crate::store) mod assignment_observation {
    use super::Key;
    use std::cell::RefCell;
    thread_local! { static ASSIGNMENTS: RefCell<Option<Vec<Key>>> = const { RefCell::new(None) }; }
    pub(in crate::store) struct Observation;
    impl Observation {
        pub(in crate::store) fn begin() -> Self {
            ASSIGNMENTS.with(|values| assert!(values.replace(Some(Vec::new())).is_none()));
            Self
        }
        pub(in crate::store) fn finish(self) -> Vec<Key> {
            ASSIGNMENTS.with(|values| {
                values
                    .borrow_mut()
                    .take()
                    .expect("active assignment observation")
            })
        }
    }
    impl Drop for Observation {
        fn drop(&mut self) {
            ASSIGNMENTS.with(|values| {
                values.borrow_mut().take();
            });
        }
    }
    pub(super) fn record(key: Key) {
        ASSIGNMENTS.with(|values| {
            if let Some(values) = values.borrow_mut().as_mut() {
                values.push(key);
            }
        });
    }
}

//! Exact reverse references. Semantic banks never carry admission leases; their
//! parallel nested custody banks remain with the controlled frontier.
use super::super::*;
use super::counted_relations::{Change, Custody as MapCustody, Delta, Index as Map};
use super::names::NamesBacking;
use super::records::NamesRead;
use super::{NodeFields, NodeRead};

/// Lease-free canonical roots. Created-owner edges are indexed separately so
/// publication never visits unrelated canonical targets to find those edges.
#[derive(Clone, Default)]
pub(in crate::store) struct InboundIndex {
    targets: Map,
    created_owners: Map,
    usage: custody::Usage,
}
impl std::fmt::Debug for InboundIndex {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InboundIndex")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Default)]
pub(in crate::store) struct InboundCustody {
    targets: MapCustody,
    created_owners: MapCustody,
}
#[derive(Clone, Default)]
pub(in crate::store) struct Inbound {
    index: InboundIndex,
    custody: InboundCustody,
    private: PrivateInbound,
}
#[derive(Clone, Default)]
struct PrivateInbound {
    index: InboundIndex,
    custody: InboundCustody,
}
pub(in crate::store) type InboundUndo = Inbound;
pub(in crate::store) struct InboundPlan {
    next: Inbound,
    retain_undo: bool,
}
impl InboundIndex {
    #[cfg(test)]
    pub fn retained_usage(&self) -> custody::Usage {
        self.usage
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<InboundCustody, DomError> {
        Ok(InboundCustody {
            targets: self.targets.adopt_custody(registry, admit, step)?,
            created_owners: self.created_owners.adopt_custody(registry, admit, step)?,
        })
    }
}

#[cfg(test)]
impl InboundIndex {
    /// Raw semantic allocations, independent of the cached receipt and ledger.
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        let mut graph = self.targets.inspect_graph()?;
        graph.merge(&self.created_owners.inspect_graph()?)?;
        Ok(graph)
    }
    /// Decode the actual count leaves without the production reference walker.
    /// Created identities remain visible for the canonical-state oracle.
    pub fn inspect_references(&self) -> Vec<(Key, Key, usize)> {
        self.targets.inspect_relations()
    }
}

impl Inbound {
    pub fn capture(index: &InboundIndex, custody: &InboundCustody) -> Self {
        Self {
            index: index.clone(),
            custody: custody.clone(),
            private: PrivateInbound::default(),
        }
    }
    pub fn mark_published(&mut self, index: &InboundIndex, custody: &InboundCustody) {
        self.index = index.clone();
        self.custody = custody.clone();
    }
    fn adjust(
        &mut self,
        target: Key,
        owner: Key,
        delta: Delta,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        Self::adjust_pair(
            &mut self.index,
            &mut self.custody,
            Change {
                group: target,
                member: owner,
                delta,
            },
            registry,
            admit,
            step,
        )
    }
    fn adjust_owner(
        &mut self,
        private: bool,
        change: Change,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        if private {
            Self::adjust_pair(
                &mut self.private.index,
                &mut self.private.custody,
                change,
                registry,
                admit,
                step,
            )
        } else {
            self.adjust(
                change.group,
                change.member,
                change.delta,
                registry,
                admit,
                step,
            )
        }
    }
    fn adjust_pair(
        index: &mut InboundIndex,
        custody: &mut InboundCustody,
        change: Change,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let Change {
            group: target,
            member: owner,
            delta,
        } = change;
        let changed =
            index
                .targets
                .change(&custody.targets, change, registry, &mut *admit, &mut *step)?;
        let (targets_custody, targets) = (changed.custody, changed.value);
        let (created_custody, created) = if matches!(owner, Key::Created(_)) {
            let changed = index.created_owners.change(
                &custody.created_owners,
                Change {
                    group: owner,
                    member: target,
                    delta,
                },
                registry,
                &mut *admit,
                &mut *step,
            )?;
            (changed.custody, changed.value)
        } else {
            (custody.created_owners.clone(), index.created_owners.clone())
        };
        let usage = targets
            .retained_usage()
            .checked_add(created.retained_usage())?;
        index.targets = targets;
        custody.targets = targets_custody;
        index.created_owners = created;
        custody.created_owners = created_custody;
        index.usage = usage;
        Ok(())
    }
    pub fn finish(
        &self,
        map: impl Fn(Key) -> Option<NodeId>,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<(InboundIndex, InboundCustody), DomError> {
        let mut next = self.clone();
        let steps = std::cell::RefCell::new(step);
        let mut rewrite = |target, owner, count| -> Result<(), DomError> {
            next.adjust(
                target,
                owner,
                Delta::Remove(count),
                registry,
                &mut admit,
                &mut || (*steps.borrow_mut())(),
            )?;
            if let (Some(target), Some(owner)) = (map(target), map(owner)) {
                next.adjust(
                    Key::Existing(target),
                    Key::Existing(owner),
                    Delta::Add(count),
                    registry,
                    &mut admit,
                    &mut || (*steps.borrow_mut())(),
                )?;
            }
            Ok(())
        };
        self.index.targets.visit_created(
            &mut |target, owner, count| rewrite(target, owner, count),
            &mut || (*steps.borrow_mut())(),
        )?;
        // Created targets were handled above; only Existing targets remain.
        self.index.created_owners.visit(
            &mut |owner, target, count| {
                if matches!(target, Key::Existing(_)) {
                    rewrite(target, owner, count)?;
                }
                Ok(())
            },
            &mut || (*steps.borrow_mut())(),
        )?;
        Ok((next.index, next.custody))
    }
}

impl Draft {
    pub(in crate::store) fn canonical_key(&self, key: Key) -> Key {
        self.published_id(key).map_or(key, Key::Existing)
    }
    pub(in crate::store) fn live_key(&self, key: Key) -> Key {
        match key {
            Key::Existing(id) => self
                .phase_map
                .as_ref()
                .and_then(|phase| phase.ids.original(id))
                .map_or(key, Key::Created),
            Key::Created(_) => key,
        }
    }
    pub(in crate::store) fn prepare_inbound_delta(
        &self,
        owner: Key,
        removed: impl IntoIterator<Item = Key>,
        added: impl IntoIterator<Item = Key>,
        cancel: &AtomicBool,
    ) -> Result<InboundPlan, DomError> {
        let retain_undo = self.prepare_undo(cancel)?;
        let mut next = self.inbound.clone();
        let private = self.is_private(self.live_key(owner));
        let normalize = |key| {
            if private {
                self.live_key(key)
            } else {
                self.canonical_key(key)
            }
        };
        let owner = normalize(owner);
        for target in removed {
            next.adjust_owner(
                private,
                Change {
                    group: normalize(target),
                    member: owner,
                    delta: Delta::Remove(1),
                },
                self.custody_registry(),
                &mut |usage| self.admit_custody_usage(usage, cancel),
                &mut || self.source_step(cancel),
            )?;
        }
        for target in added {
            next.adjust_owner(
                private,
                Change {
                    group: normalize(target),
                    member: owner,
                    delta: Delta::Add(1),
                },
                self.custody_registry(),
                &mut |usage| self.admit_custody_usage(usage, cancel),
                &mut || self.source_step(cancel),
            )?;
        }
        Ok(InboundPlan { next, retain_undo })
    }
    pub(in crate::store) fn commit_inbound(&mut self, plan: InboundPlan) {
        let old = std::mem::replace(&mut self.inbound, plan.next);
        if plan.retain_undo {
            self.record_inbound_undo_prepared(old);
        }
    }
    pub(in crate::store) fn inbound_keys(
        &self,
        target: Key,
        cancel: &AtomicBool,
    ) -> Result<select::ScratchVec<Key>, DomError> {
        self.source_step(cancel)?;
        let mut out = self.effect_vec();
        for (index, target) in [
            (&self.inbound.index.targets, self.canonical_key(target)),
            (&self.inbound.private.index.targets, self.live_key(target)),
        ] {
            index.visit_group(
                target,
                &mut |owner, _| {
                    self.select_reserve(&mut out, 1, cancel)?;
                    out.push(self.live_key(owner));
                    Ok(())
                },
                &mut || self.source_step(cancel),
            )?;
        }
        Ok(out)
    }
    pub(in crate::store) fn prepare_inbound_visibility(
        &self,
        owner: Key,
        was_private: bool,
        is_private: bool,
        cancel: &AtomicBool,
    ) -> Result<InboundPlan, DomError> {
        let retain_undo = self.prepare_undo(cancel)?;
        let mut next = self.inbound.clone();
        if was_private != is_private {
            visit_node(self.node(owner)?, |target| {
                for (private, delta) in
                    [(was_private, Delta::Remove(1)), (is_private, Delta::Add(1))]
                {
                    let normalize = |key| {
                        if private {
                            self.live_key(key)
                        } else {
                            self.canonical_key(key)
                        }
                    };
                    next.adjust_owner(
                        private,
                        Change {
                            group: normalize(target),
                            member: normalize(owner),
                            delta,
                        },
                        self.custody_registry(),
                        &mut |usage| self.admit_custody_usage(usage, cancel),
                        &mut || self.source_step(cancel),
                    )?;
                }
                Ok(())
            })?;
        }
        Ok(InboundPlan { next, retain_undo })
    }
}

/// One fixed position per scalar reference occurrence. Duplicates in different
/// fields deliberately remain distinct, so clearing one cannot lose another.
fn scalar_refs(node: &NodeRead<'_>) -> [Option<Key>; 24] {
    use crate::reflected::ReflectedTarget;
    let patch = node.template_patch;
    [
        Some(node.document),
        node.parent,
        node.shadow_root,
        node.host,
        node.template_contents,
        patch.and_then(|value| value.target),
        patch.and_then(|value| value.start),
        patch.and_then(|value| value.end),
        node.template_document
            .and_then(crate::template::TemplateDocument::inert),
        node.manual_slot,
        node.assigned_slot,
        node.reflected_targets
            .and_then(|value| value.get(ReflectedTarget::CommandFor))
            .flatten(),
        node.reflected_targets
            .and_then(|value| value.get(ReflectedTarget::PopoverTarget))
            .flatten(),
        node.control_owner,
        node.option_select,
        node.script.and_then(|value| value.parser_document),
        node.script.and_then(|value| value.preparation_document),
        node.style_parser_document,
        node.base
            .as_ref()
            .and_then(|value| value.frozen.as_ref())
            .map(|value| value.document),
        node.base_document.as_ref().and_then(|value| value.first),
        node.link.as_ref().and_then(|value| value.parser_document),
        node.image.as_ref().map(|value| value.dimension_source),
        node.hyperlink
            .as_ref()
            .and_then(|value| value.parsed.as_ref())
            .map(|value| value.document),
        match node.kind {
            DomNodeKind::Element(element) => element
                .common_state()
                .html()
                .and_then(|state| state.previously_focused_element().copied())
                .map(Key::Existing),
            _ => None,
        },
    ]
}
fn same_names(old: Option<NamesRead<'_>>, next: Option<&super::Names>) -> bool {
    match (old, next.map(|value| &value.backing)) {
        (None, None) => true,
        (
            Some(NamesRead::Canonical(old) | NamesRead::AliasedCanonical(old, _)),
            Some(NamesBacking::Canonical(next)),
        ) => old.same_storage(next),
        (
            Some(NamesRead::Staged(old) | NamesRead::AliasedStaged(old, _)),
            Some(NamesBacking::Staged(next)),
        ) => old.same_storage(next),
        _ => false,
    }
}

fn visit_node(
    node: NodeRead<'_>,
    mut visit: impl FnMut(Key) -> Result<(), DomError>,
) -> Result<(), DomError> {
    for target in scalar_refs(&node).into_iter().flatten() {
        visit(target)?;
    }
    if let Some(names) = node.form_past_names {
        for name in names.iter() {
            visit(name.target)?;
        }
    }
    for values in [node.children, node.manual_nodes, node.assigned_nodes] {
        for target in values.iter() {
            visit(target)?;
        }
    }
    if let Some(state) = node.base_document {
        for target in state.render_blocking.iter() {
            visit(target)?;
        }
    }
    if let Some(super::AutofocusRead::Known { candidates, .. }) = node.autofocus_document {
        for target in candidates.iter() {
            visit(target)?;
        }
    }
    if let Some(super::InteractiveRead::Known {
        open_dialogs,
        top_layer,
        pending_top_layer_removals,
    }) = node.interactive_document
    {
        for values in [open_dialogs, top_layer, pending_top_layer_removals] {
            for target in values.iter() {
                visit(target)?;
            }
        }
    }
    Ok(())
}

impl Draft {
    pub(in crate::store) fn prepare_inbound_fields(
        &self,
        owner: Key,
        fields: &NodeFields,
        cancel: &AtomicBool,
    ) -> Result<InboundPlan, DomError> {
        let retain_undo = self.prepare_undo(cancel)?;
        let mut next = self.inbound.clone();
        let current = self.node(owner)?;
        let candidate = NodeRead::staged_fields(self.records_for(owner), fields);
        let private = self.is_private(self.live_key(owner));
        let normalize = |key| {
            if private {
                self.live_key(key)
            } else {
                self.canonical_key(key)
            }
        };
        let normalized_owner = normalize(owner);
        for (old, new) in scalar_refs(&current)
            .into_iter()
            .zip(scalar_refs(&candidate))
        {
            let old = old.map(normalize);
            let new = new.map(normalize);
            if old != new {
                if let Some(target) = old {
                    next.adjust_owner(
                        private,
                        Change {
                            group: target,
                            member: normalized_owner,
                            delta: Delta::Remove(1),
                        },
                        self.custody_registry(),
                        &mut |usage| self.admit_custody_usage(usage, cancel),
                        &mut || self.source_step(cancel),
                    )?;
                }
                if let Some(target) = new {
                    next.adjust_owner(
                        private,
                        Change {
                            group: target,
                            member: normalized_owner,
                            delta: Delta::Add(1),
                        },
                        self.custody_registry(),
                        &mut |usage| self.admit_custody_usage(usage, cancel),
                        &mut || self.source_step(cancel),
                    )?;
                }
            }
        }
        if !same_names(current.form_past_names, fields.form_past_names.as_ref()) {
            if let Some(names) = current.form_past_names {
                for name in names.iter() {
                    next.adjust_owner(
                        private,
                        Change {
                            group: normalize(name.target),
                            member: normalized_owner,
                            delta: Delta::Remove(1),
                        },
                        self.custody_registry(),
                        &mut |usage| self.admit_custody_usage(usage, cancel),
                        &mut || self.source_step(cancel),
                    )?;
                }
            }
            if let Some(names) = &fields.form_past_names {
                for name in names.view().iter() {
                    next.adjust_owner(
                        private,
                        Change {
                            group: normalize(name.target),
                            member: normalized_owner,
                            delta: Delta::Add(1),
                        },
                        self.custody_registry(),
                        &mut |usage| self.admit_custody_usage(usage, cancel),
                        &mut || self.source_step(cancel),
                    )?;
                }
            }
        }
        Ok(InboundPlan { next, retain_undo })
    }
    pub(in crate::store) fn prepare_inbound_node(
        &self,
        owner: Key,
        node: NodeRead<'_>,
        added: bool,
        cancel: &AtomicBool,
    ) -> Result<InboundPlan, DomError> {
        let retain_undo = self.prepare_undo(cancel)?;
        let mut next = self.inbound.clone();
        let private = self.is_private(self.live_key(owner));
        let normalize = |key| {
            if private {
                self.live_key(key)
            } else {
                self.canonical_key(key)
            }
        };
        let owner = normalize(owner);
        let delta = if added {
            Delta::Add(1)
        } else {
            Delta::Remove(1)
        };
        visit_node(node, |target| {
            next.adjust_owner(
                private,
                Change {
                    group: normalize(target),
                    member: owner,
                    delta,
                },
                self.custody_registry(),
                &mut |usage| self.admit_custody_usage(usage, cancel),
                &mut || self.source_step(cancel),
            )
        })?;
        Ok(InboundPlan { next, retain_undo })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(owner: DomOwnerId, slot: usize, generation: u64) -> Key {
        Key::Existing(NodeId {
            owner,
            slot,
            generation,
        })
    }
    fn registry() -> (custody::Ledger, bank::CustodyRegistry) {
        let ledger = custody::Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
        (ledger, registry)
    }
    fn change(
        index: &mut Inbound,
        target: Key,
        owner: Key,
        delta: Delta,
        registry: &bank::CustodyRegistry,
    ) -> usize {
        let mut work = 0;
        index
            .adjust(target, owner, delta, registry, &mut |_| Ok(()), &mut || {
                work += 1;
                Ok(())
            })
            .unwrap();
        work
    }
    fn publish_phase(draft: &mut Draft, dom: &mut Dom, cancel: &AtomicBool) {
        let prepared = draft.finish(dom, cancel).unwrap();
        let mut publication = dom.prepare_publication(prepared, cancel).unwrap();
        let rebase = draft.prepare_rebase(&mut publication, cancel).unwrap();
        let publication = dom.publish_prepared_with_proof(publication);
        draft.commit_publication(rebase, &publication.committed);
        crate::store::oracle_tests::assert_full_canonical(dom);
    }
    #[test]
    fn private_references_survive_real_publication_rollback_and_exposure_without_backlog_scan() {
        let cancel = AtomicBool::new(false);
        let limits = DomLimits::new(
            256,
            65536,
            256,
            0,
            0,
            8,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut setup = DomBatch::new();
        let first = setup.create_document();
        let second = setup.create_document();
        let commit = dom.apply(setup, &cancel).unwrap();
        let first = Key::Existing(commit.document(&first).unwrap().node());
        let second = Key::Existing(commit.document(&second).unwrap().node());
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        draft.phase_map = Some(source::PhaseMap {
            ids: source::AllocationIds::default(),
            private: source::Privacy::default(),
        });
        // This Document acquires a public identity at the first phase. The
        // private edges must retain its original continuation alias.
        let kind = draft.node(first).unwrap().kind.clone();
        let first = Key::Created(draft.created.len());
        assert_eq!(draft.allocate(kind, first, limits, &cancel).unwrap(), first);
        let mut selected = None;
        for _ in 0..64 {
            let key = draft
                .allocate(DomNodeKind::Text("private".into()), first, limits, &cancel)
                .unwrap();
            draft.mark_source_private(key, &cancel).unwrap();
            selected.get_or_insert(key);
        }
        let selected = selected.unwrap();
        publish_phase(&mut draft, &mut dom, &cancel);
        assert!(
            draft
                .inbound_keys(first, &cancel)
                .unwrap()
                .contains(&selected)
        );
        let mark = draft.journal_mark(0, 0, &cancel).unwrap();
        draft
            .with_node_fields(selected, &cancel, |fields| fields.document = second)
            .unwrap();
        assert!(
            !draft
                .inbound_keys(first, &cancel)
                .unwrap()
                .contains(&selected)
        );
        assert!(
            draft
                .inbound_keys(second, &cancel)
                .unwrap()
                .contains(&selected)
        );
        draft.rollback_journal(mark);
        assert!(
            draft
                .inbound_keys(first, &cancel)
                .unwrap()
                .contains(&selected)
        );
        assert!(
            !draft
                .inbound_keys(second, &cancel)
                .unwrap()
                .contains(&selected)
        );
        for _ in 0..3 {
            let visits = std::cell::Cell::new(0);
            let mapped = std::cell::Cell::new(0);
            draft
                .inbound
                .finish(
                    |_| {
                        mapped.set(mapped.get() + 1);
                        panic!("unchanged private backlog must not map");
                    },
                    draft.custody_registry(),
                    |_| Ok(()),
                    || {
                        visits.set(visits.get() + 1);
                        Ok(())
                    },
                )
                .unwrap();
            assert_eq!(mapped.get(), 0);
            assert_eq!(visits.get(), 0);
            publish_phase(&mut draft, &mut dom, &cancel);
        }
        draft.expose_source_subtree(selected, &cancel).unwrap();
        publish_phase(&mut draft, &mut dom, &cancel);
        let published = draft.published_id(selected).unwrap();
        assert_eq!(
            count(
                &dom.state.inbound,
                draft.canonical_key(first),
                Key::Existing(published)
            ),
            1
        );
        assert!(
            draft
                .inbound_keys(first, &cancel)
                .unwrap()
                .contains(&selected)
        );
    }
    fn count(index: &InboundIndex, target: Key, owner: Key) -> usize {
        index.targets.count(target, owner)
    }
    fn enumerate(map: &Map) -> usize {
        let graph = map.inspect_graph().unwrap();
        graph.index_charge().bytes + graph.record_totals().bytes
    }
    #[test]
    fn exact_generations_and_owner_multiplicities_survive_partial_reference_removal() {
        let (_, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let mut inbound = Inbound::default();
        let target = key(owner, 7, 1);
        let reused = key(owner, 7, u64::MAX);
        let first = key(owner, 9, 2);
        let second = key(owner, 10, 2);
        change(&mut inbound, target, first, Delta::Add(2), &registry);
        change(&mut inbound, target, second, Delta::Add(1), &registry);
        change(&mut inbound, reused, first, Delta::Add(3), &registry);
        change(&mut inbound, target, first, Delta::Remove(1), &registry);
        assert_eq!(count(&inbound.index, target, first), 1);
        assert_eq!(count(&inbound.index, target, second), 1);
        assert_eq!(count(&inbound.index, reused, first), 3);
        change(&mut inbound, target, first, Delta::Remove(1), &registry);
        assert_eq!(count(&inbound.index, target, first), 0);
        assert_eq!(count(&inbound.index, target, second), 1);
        change(&mut inbound, target, second, Delta::Remove(1), &registry);
        assert_eq!(inbound.index.targets.count(target, second), 0);
        assert_eq!(count(&inbound.index, reused, first), 3);
    }
    #[test]
    fn adding_one_owner_has_fixed_path_work_despite_many_other_inbound_owners() {
        let (_, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let target = key(owner, 7, 1);
        let mut inbound = Inbound::default();
        let first = change(
            &mut inbound,
            target,
            key(owner, 1000, 1),
            Delta::Add(1),
            &registry,
        );
        for slot in 0..256 {
            change(
                &mut inbound,
                target,
                key(owner, slot, 1),
                Delta::Add(1),
                &registry,
            );
        }
        let last = change(
            &mut inbound,
            target,
            key(owner, 1001, 1),
            Delta::Add(1),
            &registry,
        );
        assert_eq!(first, 2 * super::super::counted_relations::KEY_PARTS);
        assert_eq!(last, first);
        let mut owners = 0;
        inbound
            .index
            .targets
            .visit_group(
                target,
                &mut |_, _| {
                    owners += 1;
                    Ok(())
                },
                &mut || Ok(()),
            )
            .unwrap();
        assert_eq!(owners, 258);
    }
    #[test]
    fn refused_nested_control_admission_keeps_the_original_root_and_custody() {
        let (ledger, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let target = key(owner, 7, 1);
        let mut inbound = Inbound::default();
        change(
            &mut inbound,
            target,
            Key::Created(0),
            Delta::Add(2),
            &registry,
        );
        registry.reconcile_releases();
        let before = ledger.total();
        let mut admissions = 0;
        let result = inbound.adjust(
            target,
            Key::Created(1),
            Delta::Add(1),
            &registry,
            &mut |_| {
                admissions += 1;
                if admissions == 8 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
            &mut || Ok(()),
        );
        assert_eq!(result, Err(DomError::Cancelled));
        assert_eq!(admissions, 8);
        assert_eq!(count(&inbound.index, target, Key::Created(0)), 2);
        assert_eq!(count(&inbound.index, target, Key::Created(1)), 0);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), before);
    }
    #[test]
    fn publication_rewrites_created_edges_and_excludes_private_owners() {
        let (_, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let existing = key(owner, 7, 1);
        let published_owner = key(owner, 1000, 4);
        let published_target = key(owner, 1001, 9);
        let mut inbound = Inbound::default();
        change(
            &mut inbound,
            existing,
            Key::Created(0),
            Delta::Add(2),
            &registry,
        );
        change(
            &mut inbound,
            Key::Created(1),
            Key::Created(0),
            Delta::Add(3),
            &registry,
        );
        change(
            &mut inbound,
            Key::Created(1),
            existing,
            Delta::Add(1),
            &registry,
        );
        change(
            &mut inbound,
            existing,
            Key::Created(2),
            Delta::Add(1),
            &registry,
        );
        for slot in 20..148 {
            change(
                &mut inbound,
                key(owner, slot, 1),
                existing,
                Delta::Add(1),
                &registry,
            );
        }
        let mapped = std::cell::Cell::new(0);
        let map = |key| {
            mapped.set(mapped.get() + 1);
            match key {
                Key::Existing(id) => Some(id),
                Key::Created(0) => {
                    if let Key::Existing(id) = published_owner {
                        Some(id)
                    } else {
                        unreachable!()
                    }
                }
                Key::Created(1) => {
                    if let Key::Existing(id) = published_target {
                        Some(id)
                    } else {
                        unreachable!()
                    }
                }
                Key::Created(_) => None,
            }
        };
        let mut work = 0;
        let (index, custody) = inbound
            .finish(
                map,
                &registry,
                |_| Ok(()),
                || {
                    work += 1;
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(mapped.get(), 8);
        assert!(
            work <= 40 * super::super::counted_relations::KEY_PARTS,
            "publication visited {work} selected path nodes"
        );
        assert_eq!(count(&index, existing, published_owner), 2);
        assert_eq!(count(&index, published_target, published_owner), 3);
        assert_eq!(count(&index, published_target, existing), 1);
        assert_eq!(count(&index, existing, Key::Created(2)), 0);
        assert!(index.created_owners.is_empty());
        assert!(
            index
                .targets
                .inspect_relations()
                .iter()
                .all(|(target, _, _)| matches!(target, Key::Existing(_)))
        );
        assert_eq!(count(&index, key(owner, 20, 1), existing), 1);
        let captured = Inbound::capture(&index, &custody);
        assert_eq!(count(&captured.index, published_target, published_owner), 3);
    }
    #[test]
    fn controlled_aliases_share_custody_and_external_semantics_do_not_retain_it() {
        let (ledger, registry) = registry();
        let baseline = ledger.total();
        let owner = DomOwnerId::allocate().unwrap();
        let target = key(owner, 7, 1);
        let source = key(owner, 8, 1);
        let mut inbound = Inbound::default();
        change(&mut inbound, target, source, Delta::Add(2), &registry);
        let live = ledger.total();
        let alias = Inbound::capture(&inbound.index, &inbound.custody);
        let external = inbound.index.clone();
        assert_eq!(ledger.total(), live);
        drop(inbound);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), live);
        drop(alias);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
        assert_eq!(count(&external, target, source), 2);
    }
    #[test]
    fn cached_semantic_receipt_matches_independent_full_nested_enumeration() {
        let (_, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let mut inbound = Inbound::default();
        for slot in 0..32 {
            change(
                &mut inbound,
                key(owner, slot, 1),
                Key::Created(slot),
                Delta::Add(2),
                &registry,
            );
        }
        let expected = enumerate(&inbound.index.targets) + enumerate(&inbound.index.created_owners);
        assert_eq!(
            inbound.index.retained_usage(),
            custody::Usage::new(expected, 0)
        );
        change(
            &mut inbound,
            key(owner, 4, 1),
            Key::Created(4),
            Delta::Remove(2),
            &registry,
        );
        let expected = enumerate(&inbound.index.targets) + enumerate(&inbound.index.created_owners);
        assert_eq!(
            inbound.index.retained_usage(),
            custody::Usage::new(expected, 0)
        );
    }
    #[test]
    fn active_capture_and_historical_adoption_prune_unchanged_nested_frontiers() {
        let (ledger, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let source = key(owner, 1000, 1);
        let mut inbound = Inbound::default();
        for slot in 0..256 {
            change(
                &mut inbound,
                key(owner, slot, 1),
                source,
                Delta::Add(2),
                &registry,
            );
        }
        registry.reconcile_releases();
        let before = ledger.total();
        let mut described = 0;
        let active = inbound
            .index
            .adopt_custody(&registry, &mut |_| Ok(()), &mut || {
                described += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(described, 0);
        assert_eq!(ledger.total(), before);
        drop(active);
        let historical = inbound.index.clone();
        change(
            &mut inbound,
            key(owner, 4, 1),
            source,
            Delta::Add(1),
            &registry,
        );
        let mut described = 0;
        let control = historical
            .adopt_custody(&registry, &mut |_| Ok(()), &mut || {
                described += 1;
                Ok(())
            })
            .unwrap();
        assert!(described > 0);
        assert!(
            described <= 4 * super::super::counted_relations::KEY_PARTS,
            "adoption described {described} materialized entries"
        );
        let old = Inbound::capture(&historical, &control);
        assert_eq!(count(&old.index, key(owner, 4, 1), source), 2);
        assert_eq!(count(&inbound.index, key(owner, 4, 1), source), 3);
        assert_eq!(count(&old.index, key(owner, 255, 1), source), 2);
    }
    #[test]
    fn uncontrolled_historical_adoption_rebuilds_dependencies_without_semantic_leases() {
        let (old_ledger, old_registry) = registry();
        let baseline = old_ledger.total();
        let owner = DomOwnerId::allocate().unwrap();
        let target = key(owner, 7, u64::MAX);
        let source = key(owner, 8, 1);
        let mut inbound = Inbound::default();
        change(&mut inbound, target, source, Delta::Add(2), &old_registry);
        let historical = inbound.index.clone();
        drop(inbound);
        old_registry.reconcile_releases();
        assert_eq!(old_ledger.total(), baseline);
        let (ledger, registry) = registry();
        let baseline = ledger.total();
        let mut described = 0;
        let control = historical
            .adopt_custody(&registry, &mut |_| Ok(()), &mut || {
                described += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(described, 2 * super::super::counted_relations::KEY_PARTS);
        assert_eq!(count(&historical, target, source), 2);
        assert!(ledger.total().bytes() > baseline.bytes());
        let captured = Inbound::capture(&historical, &control);
        let before = ledger.total();
        assert_eq!(count(&captured.index, target, source), 2);
        assert_eq!(ledger.total(), before);
        drop(captured);
        drop(control);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
        assert_eq!(count(&historical, target, source), 2);
    }
}

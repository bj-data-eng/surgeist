//! Shared canonical source execution and actual temporary custody.
use super::style::Frame;
use super::*;
pub(super) mod checkpoint;
mod control;
pub(in crate::store) use control::{PreparedFullscreenControl, Undo as ControlUndo};
pub(in crate::store) mod effects;
pub(crate) mod formation;
pub(super) mod headroom;
mod identities;
pub(super) mod initial;
pub(in crate::store) mod input;
pub(crate) mod operation;
pub(in crate::store) mod scratch;
pub(crate) mod storage;
pub(in crate::store) use identities::Privacy;
pub(crate) use identities::{AllocationIds, alias_id, alias_key};

/// Original Draft keys retain their first published canonical generation.
#[derive(Clone)]
pub(super) struct PhaseMap {
    pub ids: AllocationIds,
    pub private: Privacy,
}
impl Draft {
    pub(super) fn source_ascii(
        &self,
        value: &DocumentString,
        expected: &str,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        if value.len() != expected.len() {
            return Ok(false);
        }
        for (actual, expected) in value.code_units().iter().zip(expected.bytes()) {
            self.source_step(cancel)?;
            if *actual != u16::from(expected) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub(super) fn source_admit_units(
        &self,
        extra: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(limits) = self.source_limits else {
            return Ok(());
        };
        let mut units = self
            .source_other_units
            .checked_add(extra)
            .ok_or(DomError::RepresentationExhausted)?;
        units = units
            .checked_add(self.source_storage_units(cancel)?)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            units,
            limits.retained_units,
            DomResource::SourceRetainedUnits,
        )
    }
    pub(super) fn source_storage_units(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        Ok(self.controlled_storage_usage(cancel)?.units())
    }
    pub(super) fn source_admit(
        &self,
        additional_bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(limits) = self.source_limits else {
            return Ok(());
        };
        let total = self
            .source_storage_bytes(cancel)?
            .checked_add(self.source_other_bytes)
            .and_then(|n| n.checked_add(additional_bytes))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            total,
            limits.retained_bytes,
            DomResource::SourceRetainedBytes,
        )
    }
    pub(super) fn source_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if let Some(limits) = self.source_limits {
            let next = self
                .source_work
                .load(Ordering::Relaxed)
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            if next > limits.work {
                return Err(DomError::SourceWorkLimit { limit: limits.work });
            }
            self.source_work.store(next, Ordering::Relaxed);
            source::formation::SourceStorage::record_step();
            crate::source::event::step_ancestors(self.source_ancestors.as_ref(), cancel)?;
        }
        Ok(())
    }
    pub(super) fn published_id(&self, key: Key) -> Option<NodeId> {
        match key {
            Key::Existing(id) => Some(id),
            Key::Created(i) => self.phase_map.as_ref().and_then(|m| m.ids.get(i)),
        }
    }
    pub(super) fn is_private(&self, key: Key) -> bool {
        match key {
            Key::Existing(_) => false,
            Key::Created(i) => self.phase_map.as_ref().is_some_and(|m| m.private.get(i)),
        }
    }
    pub(super) fn mark_source_private(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.set_source_privacy(key, true, cancel)
    }
    fn set_source_privacy(
        &mut self,
        key: Key,
        value: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Key::Created(index) = key else {
            return Ok(());
        };
        let Some(map) = &self.phase_map else {
            return Ok(());
        };
        if map.private.get(index) == value {
            return Ok(());
        }
        if value {
            debug_assert!(
                map.ids.get(index).is_none(),
                "published nodes cannot become private"
            );
        }
        let mut next = map.private.clone();
        let registry = self.custody_registry().clone();
        next.set(index, value, &registry, |usage| {
            self.admit_custody_usage(usage, cancel)
        })?;
        let units = self.node(key)?.logical_units()?;
        self.set_created_dirty(index, !value, cancel)?;
        let nodes = if value {
            self.private_nodes.checked_add(1)
        } else {
            self.private_nodes.checked_sub(1)
        }
        .ok_or(DomError::RepresentationExhausted)?;
        let nextunits = if value {
            self.private_units.checked_add(units)
        } else {
            self.private_units.checked_sub(units)
        }
        .ok_or(DomError::RepresentationExhausted)?;
        let contribution = transaction::Counts::node(self.node(key)?)?;
        let nextcounts = if value {
            self.private_counts.checked_add(contribution)?
        } else {
            self.private_counts
                .checked_replace(contribution, transaction::Counts::default())?
        };
        let undo = self.prepare_undo_slots(9, cancel)?;
        let visibility = self.prepare_queue_visibility_change(key, !value, value, cancel)?;
        let inbound_visibility = self.prepare_inbound_visibility(key, !value, value, cancel)?;
        let controls = self.prepare_control_visibility(key, !value, value, cancel)?;
        let slots = self.prepare_slot_visibility(key, !value, value, cancel)?;
        let discovery = self.prepare_discovery_visibility(key, !value, value, cancel)?;
        let oldcounts = std::mem::replace(&mut self.private_counts, nextcounts);
        let before = std::mem::replace(
            &mut self.phase_map.as_mut().expect("Source phase map").private,
            next,
        );
        let oldnodes = std::mem::replace(&mut self.private_nodes, nodes);
        let oldunits = std::mem::replace(&mut self.private_units, nextunits);
        if undo {
            self.journal.push_prepared(transaction::Undo::Privacy {
                before,
                nodes: oldnodes,
                units: oldunits,
                counts: oldcounts,
            });
        }
        self.commit_queue_visibility_change(visibility);
        self.commit_inbound(inbound_visibility);
        if let Some(plan) = controls {
            self.commit_control_plan(plan);
        }
        if let Some(plan) = slots {
            self.commit_slot_plan(plan);
        }
        self.commit_discovery(discovery, undo);
        Ok(())
    }
    pub(super) fn expose_source_subtree(
        &mut self,
        root: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // Includes actual hosted/auxiliary descendants, preserving one storage
        // owner and its actual document/cache closure.
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, 1, cancel)?;
        pending.push(root);
        while !pending.is_empty() {
            self.source_step(cancel)?;
            let key = pending.pop().expect("admitted public subtree node");
            self.effect_step(cancel)?;
            self.set_source_privacy(key, false, cancel)?;
            let n = self.node(key)?;
            self.select_reserve(&mut pending, n.children().len() + 2, cancel)?;
            for child in n.children().iter() {
                self.source_step(cancel)?;
                pending.push(child);
            }
            pending.extend(n.shadow_root());
            pending.extend(self.owned_template_contents(key, cancel)?);
        }
        Ok(())
    }
    pub(super) fn public_nodes(&self) -> usize {
        self.nodes - self.private_nodes
    }
    pub(super) fn public_units(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        self.source_step(cancel)?;
        self.units
            .checked_sub(self.private_units)
            .ok_or(DomError::RepresentationExhausted)
    }
}

#[derive(Clone, Copy)]
pub(in crate::store) enum Invocation {
    Unissued,
    Exposed(u64),
}

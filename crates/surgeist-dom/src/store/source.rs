//! Shared canonical source execution and actual temporary custody.
use super::style::Frame;
use super::*;
pub(super) mod checkpoint;
pub(crate) mod formation;
pub(super) mod headroom;
pub(super) mod initial;
pub(crate) mod operation;
pub(crate) mod storage;

pub(super) struct PublicationMap {
    pub slots: Vec<Slot>,
    pub occupants: Vec<Option<Key>>,
    pub ids: Vec<Option<NodeId>>,
}
/// Original Draft keys retain their first published canonical generation.
pub(super) struct PhaseMap {
    pub ids: Vec<Option<NodeId>>,
    pub private: Vec<bool>,
}
/// Apply the admitted Source growth plan; ordinary edits keep their allocator policy.
pub(super) fn reserve_bank<T>(
    bank: &mut Vec<T>,
    additional: usize,
    admitted: Option<usize>,
) -> Result<(), DomError> {
    match admitted {
        Some(amount) => bank
            .try_reserve_exact(amount)
            .map_err(|_| DomError::AllocationFailed),
        None => reserve(bank, additional),
    }
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
            .and_then(|n| n.checked_add(self.source_temporary_units.load(Ordering::Relaxed)))
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
        let mut units = self
            .hosted_history
            .as_ref()
            .map(|g| g.payload(|| self.source_step(cancel)).map(|p| p.units))
            .transpose()?
            .unwrap_or(0);
        for node in self.existing.iter().chain(&self.created).flatten() {
            units = units
                .checked_add(storage::node(node, || self.source_step(cancel))?.units)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(units)
    }
    /// Preadmit the replacement bank while its old backing remains live. The
    /// caller applies this exact growth policy and then checks actual storage.
    pub(super) fn source_reserve_amount<T>(
        &self,
        length: usize,
        capacity: usize,
        additional: usize,
        cancel: &AtomicBool,
    ) -> Result<Option<usize>, DomError> {
        if self.source_limits.is_none() {
            return Ok(None);
        }
        self.source_step(cancel)?;
        let required = length
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if required <= capacity {
            self.source_admit(0, cancel)?;
            return Ok(Some(0));
        }
        let maximum = isize::MAX as usize / size_of::<T>().max(1);
        let target = capacity
            .checked_mul(2)
            .filter(|n| *n <= maximum)
            .unwrap_or(required)
            .max(required);
        let bytes = target
            .checked_mul(size_of::<T>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        self.source_admit(bytes, cancel)?;
        Ok(Some(target - length))
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
            .and_then(|n| n.checked_add(self.source_formation_bytes.load(Ordering::Relaxed)))
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
            crate::source::event::step_ancestors(self.source_ancestors.as_ref(), cancel)?;
        }
        Ok(())
    }
    pub(super) fn published_id(&self, key: Key) -> Option<NodeId> {
        match key {
            Key::Existing(slot) => Some(NodeId {
                owner: self.store,
                slot,
                generation: self.generations[slot],
            }),
            Key::Created(i) => self.phase_map.as_ref().and_then(|m| m.ids[i]),
        }
    }
    pub(super) fn is_private(&self, key: Key) -> bool {
        match key {
            Key::Existing(_) => false,
            Key::Created(i) => self.phase_map.as_ref().is_some_and(|m| m.private[i]),
        }
    }
    pub(super) fn mark_source_private(&mut self, key: Key) {
        if let (Some(m), Key::Created(i)) = (&mut self.phase_map, key) {
            debug_assert!(m.ids[i].is_none(), "published nodes cannot become private");
            m.private[i] = true;
        }
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
            if let (Some(m), Key::Created(i)) = (&mut self.phase_map, key) {
                m.private[i] = false;
            }
            let n = self.node(key)?;
            self.select_reserve(&mut pending, n.children.len() + 2, cancel)?;
            for child in &n.children {
                self.source_step(cancel)?;
                pending.push(*child);
            }
            pending.extend(n.shadow_root);
            pending.extend(self.owned_template_contents(key, cancel)?);
        }
        Ok(())
    }
    pub(super) fn public_nodes(&self) -> usize {
        self.existing.iter().flatten().count()
            + self
                .created
                .iter()
                .enumerate()
                .filter(|(i, n)| n.is_some() && !self.is_private(Key::Created(*i)))
                .count()
    }
    pub(super) fn public_units(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        let mut units = self.units;
        for (i, n) in self.created.iter().enumerate() {
            if self.is_private(Key::Created(i))
                && let Some(n) = n
            {
                self.source_step(cancel)?;
                let names = form::name_units(n.form_past_names.as_ref(), cancel)?;
                let hidden = kind_units(&n.kind)?
                    .checked_add(n.base.as_ref().map_or(0, crate::base::BaseState::units))
                    .and_then(|n0| {
                        n0.checked_add(
                            n.hyperlink
                                .as_ref()
                                .map_or(0, crate::hyperlink::HyperlinkState::units),
                        )
                    })
                    .and_then(|n0| {
                        n0.checked_add(
                            n.encoding_url_document
                                .as_ref()
                                .map_or(0, crate::hyperlink::EncodingUrlDocument::units),
                        )
                    })
                    .and_then(|n0| {
                        n0.checked_add(
                            n.base_document
                                .as_ref()
                                .map_or(0, crate::base::BaseDocument::units),
                        )
                    })
                    .and_then(|n0| n0.checked_add(names))
                    .ok_or(DomError::RepresentationExhausted)?;
                units = units
                    .checked_sub(hidden)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        Ok(units)
    }
    pub(super) fn publication_map(
        &self,
        dom: &Dom,
        formation: Option<&formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<PublicationMap, DomError> {
        let mut occupants = Vec::new();
        formation::reserve(&mut occupants, dom.state.slots.len(), formation, cancel)?;
        occupants.resize(dom.state.slots.len(), None);
        for (i, n) in self.existing.iter().enumerate() {
            self.source_step(cancel)?;
            if n.is_some() {
                occupants[i] = Some(Key::Existing(i));
            }
        }
        if let Some(m) = &self.phase_map {
            for (i, id) in m.ids.iter().enumerate() {
                self.source_step(cancel)?;
                if let Some(id) = id
                    && self.created[i].is_some()
                {
                    debug_assert!(!m.private[i]);
                    occupants[id.slot] = Some(Key::Created(i));
                }
            }
        }
        let mut slots = Vec::new();
        formation::reserve(&mut slots, occupants.len(), formation, cancel)?;
        for (i, old) in dom.state.slots.iter().enumerate() {
            self.source_step(cancel)?;
            let destroyed = old.node.is_some() && occupants[i].is_none();
            slots.push(Slot {
                generation: if destroyed {
                    old.generation.checked_add(1).unwrap_or(old.generation)
                } else {
                    old.generation
                },
                retired: old.retired || (destroyed && old.generation == u64::MAX),
                node: None,
            });
        }
        let mut allocated = Vec::new();
        formation::reserve(&mut allocated, self.created.len(), formation, cancel)?;
        for (i, n) in self.created.iter().enumerate() {
            self.source_step(cancel)?;
            if n.is_none() || self.is_private(Key::Created(i)) {
                allocated.push(None);
                continue;
            }
            if let Some(id) = self.phase_map.as_ref().and_then(|m| m.ids[i]) {
                allocated.push(Some(id));
                continue;
            }
            let slot = if let Some(i) = occupants
                .iter()
                .enumerate()
                .position(|(i, k)| k.is_none() && !slots[i].retired)
            {
                i
            } else {
                formation::reserve(&mut slots, 1, formation, cancel)?;
                formation::reserve(&mut occupants, 1, formation, cancel)?;
                slots.push(Slot {
                    generation: 0,
                    retired: false,
                    node: None,
                });
                occupants.push(None);
                slots.len() - 1
            };
            occupants[slot] = Some(Key::Created(i));
            allocated.push(Some(NodeId {
                owner: dom.owner,
                slot,
                generation: slots[slot].generation,
            }));
        }
        Ok(PublicationMap {
            slots,
            occupants,
            ids: allocated,
        })
    }
    pub(super) fn map_public_ids(
        &self,
        keys: &[Key],
        id: &impl Fn(Key) -> NodeId,
        budget: Option<&formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Vec<NodeId>, DomError> {
        let mut out = Vec::new();
        formation::reserve(
            &mut out,
            keys.iter().filter(|k| !self.is_private(**k)).count(),
            budget,
            cancel,
        )?;
        for key in keys {
            self.source_step(cancel)?;
            if !self.is_private(*key) {
                out.push(id(*key));
            }
        }
        Ok(out)
    }
}

// Shared Source invocation issuance; only successful exposure consumes an ordinal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum Invocation {
    Unissued,
    Exposed(u64),
}

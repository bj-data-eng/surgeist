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
    pub(super) fn reuse_equal_node_backings(
        &self,
        candidate: &mut Node,
        original: &Node,
        budget: Option<&formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        reuse_sequence(&mut candidate.children, &original.children, budget, cancel)?;
        reuse_sequence(
            &mut candidate.manual_nodes,
            &original.manual_nodes,
            budget,
            cancel,
        )?;
        reuse_sequence(
            &mut candidate.assigned_nodes,
            &original.assigned_nodes,
            budget,
            cancel,
        )?;
        reuse_sequence(
            &mut candidate.fullscreen_events,
            &original.fullscreen_events,
            budget,
            cancel,
        )?;
        if let (
            Some(canonical::Autofocus::Known {
                candidates: current,
                ..
            }),
            Some(canonical::Autofocus::Known {
                candidates: old, ..
            }),
        ) = (
            &mut candidate.autofocus_document,
            &original.autofocus_document,
        ) {
            reuse_sequence(current, old, budget, cancel)?;
        }
        if let (
            Some(canonical::Interactive::Known {
                open_dialogs,
                top_layer,
                pending_top_layer_removals,
            }),
            Some(canonical::Interactive::Known {
                open_dialogs: old_dialogs,
                top_layer: old_top,
                pending_top_layer_removals: old_pending,
            }),
        ) = (
            &mut candidate.interactive_document,
            &original.interactive_document,
        ) {
            reuse_sequence(open_dialogs, old_dialogs, budget, cancel)?;
            reuse_sequence(top_layer, old_top, budget, cancel)?;
            reuse_sequence(pending_top_layer_removals, old_pending, budget, cancel)?;
        }
        if let (Some(current), Some(old)) = (&mut candidate.base_document, &original.base_document)
        {
            reuse_sequence(
                &mut current.render_blocking,
                &old.render_blocking,
                budget,
                cancel,
            )?;
        }
        Ok(())
    }

    /// Transitional dense formation seals into the same persistent root. The
    /// sparse transaction owner replaces formation, not this canonical bank.
    pub(super) fn seal_bank<T: PartialEq>(
        &self,
        records: Vec<T>,
        original: &bank::Bank<T>,
        budget: Option<&formation::Formation>,
        cancel: &AtomicBool,
        mut describe: impl FnMut(&T) -> Result<(bool, storage::Payload), DomError>,
    ) -> Result<bank::Bank<T>, DomError> {
        // Borrow the original until a replacement actually needs a candidate
        // root. Equal temporary records can drop before the same canonical
        // backing becomes the final candidate occurrence.
        let retain_original = || {
            if let Some(budget) = budget {
                budget.retain_payload(
                    storage::Payload {
                        bytes: original
                            .index_charge()
                            .bytes
                            .checked_add(original.totals().bytes)
                            .ok_or(DomError::RepresentationExhausted)?,
                        units: original.totals().units,
                    },
                    cancel,
                )?;
            }
            Ok::<_, DomError>(original.clone())
        };
        let mut candidate: Option<bank::Bank<T>> = None;
        let vector_bytes = records
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        for (index, record) in records.into_iter().enumerate() {
            self.source_step(cancel)?;
            let (reusable, payload) = describe(&record)?;
            if candidate.as_ref().unwrap_or(original).get(index) == Some(&record) {
                // The temporary Vec header/capacity remains until iteration
                // ends; only this discarded record's backing occurrence ends.
                if let Some(budget) = budget {
                    budget.release_payload(storage::Payload {
                        bytes: payload
                            .bytes
                            .checked_sub(size_of::<T>())
                            .ok_or(DomError::RepresentationExhausted)?,
                        units: payload.units,
                    });
                }
                continue;
            }
            if candidate.is_none() {
                candidate = Some(retain_original()?);
            }
            let bank = candidate
                .as_mut()
                .expect("changed record has a candidate root");
            let old_record = bank.totals_at(index).unwrap_or_default();
            let old_index = bank.index_charge().bytes;
            if let Some(budget) = budget {
                budget.retain_bytes(size_of::<T>(), cancel)?;
            }
            let charge = bank.update(
                index,
                |_| {
                    Ok((
                        Arc::new(record),
                        reusable,
                        bank::Totals {
                            bytes: payload.bytes,
                            units: payload.units,
                            occurrences: 1,
                        },
                    ))
                },
                |charge| {
                    if let Some(budget) = budget {
                        budget.retain_bytes(charge.bytes, cancel)?;
                    }
                    Ok(())
                },
            )?;
            if let Some(budget) = budget {
                let released_index = old_index
                    .checked_add(charge.bytes)
                    .and_then(|bytes| bytes.checked_sub(bank.index_charge().bytes))
                    .ok_or(DomError::RepresentationExhausted)?;
                budget.release_bytes(released_index);
                budget.release_payload(storage::Payload {
                    bytes: old_record.bytes,
                    units: old_record.units,
                });
            }
        }
        if let Some(budget) = budget {
            budget.release_bytes(vector_bytes);
        }
        match candidate {
            Some(bank) => Ok(bank),
            None => retain_original(),
        }
    }

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
        let mut vacancy_cursor = dom
            .state
            .slots
            .lowest_vacant()
            .unwrap_or(dom.state.slots.len());
        formation::reserve(&mut slots, occupants.len(), formation, cancel)?;
        for (i, old) in dom.state.slots.values().enumerate() {
            self.source_step(cancel)?;
            let destroyed = old.node.is_some() && occupants[i].is_none();
            if destroyed && !old.retired && old.generation != u64::MAX {
                vacancy_cursor = vacancy_cursor.min(i);
            }
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
            while vacancy_cursor < occupants.len()
                && (occupants[vacancy_cursor].is_some() || slots[vacancy_cursor].retired)
            {
                self.source_step(cancel)?;
                vacancy_cursor += 1;
            }
            let slot = if vacancy_cursor < occupants.len() {
                let selected = vacancy_cursor;
                vacancy_cursor += 1;
                selected
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

/// Transitional dense formation can rebuild equal sequences. Canonical
/// records retain their original backing after exact comparison; charge both
/// actual occurrences until the temporary candidate backing is dropped.
pub(super) fn reuse_sequence<T: PartialEq>(
    candidate: &mut SharedVec<T>,
    original: &SharedVec<T>,
    budget: Option<&formation::Formation>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    if candidate.shares_backing(original) || candidate.len() != original.len() {
        return Ok(());
    }
    if let Some(budget) = budget {
        for _ in 0..candidate.len() {
            budget.retain_bytes(0, cancel)?;
        }
    }
    if candidate != original {
        return Ok(());
    }
    let charge = |value: &SharedVec<T>| {
        value
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<T>>()))
            .ok_or(DomError::RepresentationExhausted)
    };
    let discard = charge(candidate)?;
    if let Some(budget) = budget {
        budget.retain_bytes(charge(original)?, cancel)?;
    }
    *candidate = original.clone();
    if let Some(budget) = budget {
        budget.release_bytes(discard);
    }
    Ok(())
}

// Shared Source invocation issuance; only successful exposure consumes an ordinal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum Invocation {
    Unissued,
    Exposed(u64),
}

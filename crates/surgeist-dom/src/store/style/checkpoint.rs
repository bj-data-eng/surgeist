//! Unpublished source-segment rollback storage. The canonical owner retains the
//! original external receipt separately, so restoring a segment cannot replay it.
use super::super::*;
use std::cell::Cell;

pub(in crate::store) struct Charge {
    live: Arc<AtomicUsize>,
    bytes: usize,
    live_units: Arc<AtomicUsize>,
    units: usize,
}
impl Charge {
    pub(super) fn payload(
        live: Arc<AtomicUsize>,
        bytes: usize,
        live_units: Arc<AtomicUsize>,
        units: usize,
    ) -> Self {
        Self {
            live,
            bytes,
            live_units,
            units,
        }
    }
    pub(in crate::store) fn merge(&mut self, mut other: Self) {
        assert!(
            Arc::ptr_eq(&self.live, &other.live)
                && Arc::ptr_eq(&self.live_units, &other.live_units),
            "same Source payload owner"
        );
        self.bytes = self
            .bytes
            .checked_add(other.bytes)
            .expect("already admitted live bytes");
        self.units = self
            .units
            .checked_add(other.units)
            .expect("already admitted live units");
        other.bytes = 0;
        other.units = 0;
    }
    /// Reclassify the exact current graph of an original controlled producer.
    /// Its call must already have admitted all allocation/overlap against an
    /// envelope derived from this same ledger, with this charge excluded once.
    /// Nothing may allocate or change other custody between that call and sync.
    pub(in crate::store) fn replace_admitted_payload(&mut self, payload: super::storage::Payload) {
        let bytes = self
            .live
            .load(Ordering::Relaxed)
            .checked_sub(self.bytes)
            .and_then(|n| n.checked_add(payload.bytes))
            .expect("producer-preadmitted live bytes");
        let units = self
            .live_units
            .load(Ordering::Relaxed)
            .checked_sub(self.units)
            .and_then(|n| n.checked_add(payload.units))
            .expect("producer-preadmitted live units");
        self.live.store(bytes, Ordering::Relaxed);
        self.live_units.store(units, Ordering::Relaxed);
        self.bytes = payload.bytes;
        self.units = payload.units;
    }
}
impl Drop for Charge {
    fn drop(&mut self) {
        self.live.store(
            self.live.load(Ordering::Relaxed) - self.bytes,
            Ordering::Relaxed,
        );
        self.live_units.store(
            self.live_units.load(Ordering::Relaxed) - self.units,
            Ordering::Relaxed,
        );
    }
}
pub(in crate::store) struct Admission<'a> {
    original: &'a Draft,
    maximum: usize,
    total: usize,
    charge: Charge,
}
impl<'a> Admission<'a> {
    pub(in crate::store) fn new(original: &'a Draft, live_bytes: usize) -> Result<Self, DomError> {
        let limits = original
            .source_limits
            .expect("source checkpoint has explicit admission");
        admit(
            live_bytes,
            limits.retained_bytes,
            DomResource::StyleRetainedBytes,
        )?;
        Ok(Self {
            original,
            maximum: limits.retained_bytes,
            total: live_bytes,
            charge: Charge {
                live: original.select_temporary_bytes.clone(),
                bytes: 0,
                live_units: original.source_temporary_units.clone(),
                units: 0,
            },
        })
    }
    pub(in crate::store) fn step(&self) -> Result<(), DomError> {
        // The enclosing driver admits cancellation before checkpoint formation;
        // copies also use its exact cancellation token through copy methods.
        let limits = self
            .original
            .source_limits
            .expect("source checkpoint admission");
        let next = self
            .original
            .source_work
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > limits.work {
            return Err(DomError::StyleWorkLimit { limit: limits.work });
        }
        self.original.source_work.store(next, Ordering::Relaxed);
        Ok(())
    }
    pub(in crate::store) fn before(&self, bytes: usize) -> Result<(), DomError> {
        let next = self
            .total
            .checked_add(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(next, self.maximum, DomResource::StyleRetainedBytes)?;
        if self.original.select_work.get() != 0 {
            self.original.select_scratch(bytes)?;
        }
        Ok(())
    }
    pub(in crate::store) fn scratch_allocated(&mut self, bytes: usize) -> Result<(), DomError> {
        self.before(bytes)?;
        self.total += bytes;
        Ok(())
    }
    fn retain_payload(
        &mut self,
        payload: super::storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.before(payload.bytes)?;
        self.original.source_admit_units(payload.units, cancel)?;
        let bytes = self
            .charge
            .bytes
            .checked_add(payload.bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let live = self
            .charge
            .live
            .load(Ordering::Relaxed)
            .checked_add(payload.bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let units = self
            .charge
            .units
            .checked_add(payload.units)
            .ok_or(DomError::RepresentationExhausted)?;
        let live_units = self
            .charge
            .live_units
            .load(Ordering::Relaxed)
            .checked_add(payload.units)
            .ok_or(DomError::RepresentationExhausted)?;
        self.total += payload.bytes;
        self.charge.bytes = bytes;
        self.charge.live.store(live, Ordering::Relaxed);
        self.charge.units = units;
        self.charge.live_units.store(live_units, Ordering::Relaxed);
        Ok(())
    }
    pub(in crate::store) fn reserve<T>(
        &mut self,
        values: &mut Vec<T>,
        count: usize,
    ) -> Result<(), DomError> {
        debug_assert!(values.is_empty());
        let bytes = count
            .checked_mul(size_of::<T>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        self.before(bytes)?;
        if self.original.select_work.get() != 0 {
            self.original.select_scratch(bytes)?;
        }
        values
            .try_reserve_exact(count)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        self.before(actual)?;
        self.total += actual;
        self.charge.bytes = self
            .charge
            .bytes
            .checked_add(actual)
            .ok_or(DomError::RepresentationExhausted)?;
        self.charge.live.store(
            self.charge
                .live
                .load(Ordering::Relaxed)
                .checked_add(actual)
                .ok_or(DomError::RepresentationExhausted)?,
            Ordering::Relaxed,
        );
        if self.original.select_work.get() != 0 {
            self.original.select_scratch(0)?;
        }
        Ok(())
    }
    pub(in crate::store) fn effect_payload(
        &mut self,
        payload: super::storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<Option<Charge>, DomError> {
        if payload.bytes == 0 && payload.units == 0 {
            return Ok(None);
        }
        let budget =
            super::formation::Formation::new(self.original, cancel)?.expect("source checkpoint");
        budget.retain_payload(payload, cancel)?;
        self.total = self
            .total
            .checked_add(payload.bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(Some(budget.into_charge()))
    }
    pub(in crate::store) fn copy<T: Clone>(
        &mut self,
        original: &[T],
        cancel: &AtomicBool,
    ) -> Result<Vec<T>, DomError> {
        let mut values = Vec::new();
        self.reserve(&mut values, original.len())?;
        for value in original {
            check_cancel(cancel)?;
            self.step()?;
            values.push(value.clone());
        }
        Ok(values)
    }
    pub(in crate::store) fn finish(self) -> Charge {
        self.charge
    }
}

pub(in crate::store) struct Checkpoint {
    draft: Draft,
    charge: Charge,
}
impl Checkpoint {
    pub(in crate::store) fn restore(self, failed: &mut Draft) {
        let Self {
            mut draft, charge, ..
        } = self;
        // Every independent cumulative domain retains actual spent work. Only
        // private semantic writes/banks roll back; the owner-held CSSOM receipt
        // and phase identity are not part of this checkpoint.
        draft.source_work.store(
            draft
                .source_work
                .load(Ordering::Relaxed)
                .max(failed.source_work.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
        draft
            .base_work
            .set(draft.base_work.get().max(failed.base_work.get()));
        draft.encoding_url_work.set(
            draft
                .encoding_url_work
                .get()
                .max(failed.encoding_url_work.get()),
        );
        draft
            .meta_work
            .set(draft.meta_work.get().max(failed.meta_work.get()));
        draft
            .select_work
            .set(draft.select_work.get().max(failed.select_work.get()));
        draft.select_semantic_work.set(
            draft
                .select_semantic_work
                .get()
                .max(failed.select_semantic_work.get()),
        );
        draft.select_semantic_bytes.set(
            draft
                .select_semantic_bytes
                .get()
                .max(failed.select_semantic_bytes.get()),
        );
        draft
            .input_processing
            .preserve_spent(&failed.input_processing);
        draft.source_invocation = draft.source_invocation.max(failed.source_invocation);
        draft.source_limits = failed.source_limits;
        draft.source_other_bytes = failed.source_other_bytes;
        draft.source_other_units = failed.source_other_units;
        *failed = draft;
        // These banks now are the active Draft's ordinary structural storage.
        // Its effect bank and ScratchVec lifetimes keep their normal accounting.
        drop(charge);
    }
}
impl DraftNode {
    fn checkpoint(&self, a: &mut Admission<'_>, cancel: &AtomicBool) -> Result<Self, DomError> {
        check_cancel(cancel)?;
        a.step()?;
        let base_document = if let Some(d) = &self.base_document {
            Some(crate::base::BaseDocument {
                facts: d.facts.clone(),
                first: d.first,
                render_blocking: a.copy(&d.render_blocking, cancel)?,
            })
        } else {
            None
        };
        Ok(Self {
            relevant_global: self.relevant_global,
            document: self.document,
            kind: self.kind.clone(),
            parent: self.parent,
            children: a.copy(&self.children, cancel)?,
            shadow_root: self.shadow_root,
            host: self.host,
            template_contents: self.template_contents,
            template_patch: self.template_patch,
            template_document: self.template_document,
            manual_nodes: a.copy(&self.manual_nodes, cancel)?,
            assigned_nodes: a.copy(&self.assigned_nodes, cancel)?,
            manual_slot: self.manual_slot,
            assigned_slot: self.assigned_slot,
            reflected_targets: self.reflected_targets,
            control_owner: self.control_owner,
            option_select: self.option_select,
            form_past_names: self.form_past_names.clone(),
            script: self.script,
            style_parser_document: self.style_parser_document,
            base: self.base.clone(),
            base_document,
            link: self.link.clone(),
            image: self.image.clone(),
            hyperlink: self.hyperlink.clone(),
            encoding_url_document: self.encoding_url_document.clone(),
            interactive_document: self
                .interactive_document
                .as_ref()
                .map(|state| {
                    use crate::dialog::DocumentInteractiveState as Interactive;
                    Ok::<_, DomError>(match state {
                        Interactive::Known {
                            open_dialogs,
                            top_layer,
                        } => Interactive::Known {
                            open_dialogs: a.copy(open_dialogs, cancel)?,
                            top_layer: a.copy(top_layer, cancel)?,
                        },
                        Interactive::Unavailable => Interactive::Unavailable,
                    })
                })
                .transpose()?,
        })
    }
}
impl Draft {
    pub(in crate::store) fn source_checkpoint(
        &self,
        other_live_bytes: usize,
        other_retained_units: usize,
        cancel: &AtomicBool,
    ) -> Result<Checkpoint, DomError> {
        check_cancel(cancel)?;
        let live = self
            .source_storage_bytes(cancel)?
            .checked_add(self.source_formation_bytes.load(Ordering::Relaxed))
            .and_then(|bytes| bytes.checked_add(other_live_bytes))
            .ok_or(DomError::RepresentationExhausted)?;
        let mut a = Admission::new(self, live)?;
        let mut payload = super::storage::Payload {
            bytes: size_of::<Draft>() + size_of::<AtomicU64>(),
            units: 0,
        };
        for node in self.existing.iter().chain(&self.created).flatten() {
            payload.append(super::storage::node(node, || self.source_step(cancel))?)?;
        }
        payload.append(
            self.view_transitions
                .retained_payload(|| self.source_step(cancel))?,
        )?;
        payload.add_units(other_retained_units)?;
        a.retain_payload(payload, cancel)?;
        let mut existing = Vec::new();
        a.reserve(&mut existing, self.existing.len())?;
        for node in &self.existing {
            existing.push(
                node.as_ref()
                    .map(|n| n.checkpoint(&mut a, cancel))
                    .transpose()?,
            );
        }
        let mut created = Vec::new();
        a.reserve(&mut created, self.created.len())?;
        for node in &self.created {
            created.push(
                node.as_ref()
                    .map(|n| n.checkpoint(&mut a, cancel))
                    .transpose()?,
            );
        }
        let mut effects = Vec::new();
        a.reserve(&mut effects, self.effects.len())?;
        for frame in &self.effects {
            effects.push(frame.checkpoint(&mut a, cancel)?);
        }
        let effect_bytes = effects
            .capacity()
            .checked_mul(size_of::<select::EffectFrame>())
            .ok_or(DomError::RepresentationExhausted)?;
        let draft = Self {
            owner: self.owner.clone(),
            store: self.store,
            registries: self.registries.clone(),
            hosted_globals: self.hosted_globals.clone(),
            hosted_owner: self.hosted_owner,
            basis: self.basis,
            source_limits: self.source_limits,
            source_other_bytes: self.source_other_bytes,
            source_other_units: self.source_other_units,
            source_invocation: self.source_invocation,
            source_headroom: None,
            meta_work: Cell::new(self.meta_work.get()),
            meta_work_limit: self.meta_work_limit,
            phase_map: self
                .phase_map
                .as_ref()
                .map(|m| {
                    Ok::<_, DomError>(super::PhaseMap {
                        ids: a.copy(&m.ids, cancel)?,
                        private: a.copy(&m.private, cancel)?,
                    })
                })
                .transpose()?,
            base_work: Cell::new(self.base_work.get()),
            base_work_limit: self.base_work_limit,
            encoding_url_work: Cell::new(self.encoding_url_work.get()),
            encoding_url_work_limit: self.encoding_url_work_limit,
            view_transitions: self.view_transitions.checkpoint(&mut a, cancel)?,
            signal_slots: a.copy(&self.signal_slots, cancel)?,
            notification_queued: self.notification_queued,
            cancelled_signals: a.copy(&self.cancelled_signals, cancel)?,
            native_tasks: a.copy(&self.native_tasks, cancel)?,
            cancelled_selects: a.copy(&self.cancelled_selects, cancel)?,
            details_intentions: a.copy(&self.details_intentions, cancel)?,
            cancelled_details: a.copy(&self.cancelled_details, cancel)?,
            cancelled_images: self
                .cancelled_images
                .as_ref()
                .map(|v| v.checkpoint(&mut a, cancel))
                .transpose()?,
            effects,
            effects_running: false,
            effect_bytes,
            effect_scoped: false,
            select_work: Cell::new(self.select_work.get()),
            select_work_limit: self.select_work_limit,
            select_scratch_limit: self.select_scratch_limit,
            select_temporary_bytes: self.select_temporary_bytes.clone(),
            select_semantic_work: Cell::new(self.select_semantic_work.get()),
            select_semantic_bytes: Cell::new(self.select_semantic_bytes.get()),
            existing,
            generations: a.copy(&self.generations, cancel)?,
            created,
            tickets: a.copy(&self.tickets, cancel)?,
            nodes: self.nodes,
            units: self.units,
            document_environment: self.document_environment.clone(),
            input_processing: self.input_processing.checkpoint(),
            #[cfg(test)]
            insertion_trace: self.insertion_trace,
            #[cfg(test)]
            svg_processing_trace: self
                .svg_processing_trace
                .as_ref()
                .map(|v| v.checkpoint(&mut a, cancel))
                .transpose()?,
            source_work: Arc::new(AtomicU64::new(self.source_work.load(Ordering::Relaxed))),
            source_formation_bytes: self.source_formation_bytes.clone(),
            source_temporary_units: self.source_temporary_units.clone(),
        };
        Ok(Checkpoint {
            draft,
            charge: a.finish(),
        })
    }
    pub(in crate::store) fn source_storage_bytes(
        &self,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        let mut bytes = size_of::<Self>() + size_of::<AtomicU64>() + 3 * size_of::<AtomicUsize>();
        let mut payload = super::storage::Payload::default();
        if let Some(index) = &self.hosted_globals {
            payload.add_bytes(index.retained_payload_bytes()?)?;
        }
        if let Some(registries) = &self.registries {
            payload.add_bytes(registries.retained_payload_bytes(|| self.source_step(cancel))?)?;
        }
        let mut add = |capacity: usize, width: usize| -> Result<(), DomError> {
            bytes = bytes
                .checked_add(
                    capacity
                        .checked_mul(width)
                        .ok_or(DomError::RepresentationExhausted)?,
                )
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(())
        };
        add(self.existing.capacity(), size_of::<Option<DraftNode>>())?;
        add(self.created.capacity(), size_of::<Option<DraftNode>>())?;
        add(self.generations.capacity(), size_of::<u64>())?;
        add(self.tickets.capacity(), size_of::<Key>())?;
        if let Some(map) = &self.phase_map {
            add(map.ids.capacity(), size_of::<Option<NodeId>>())?;
            add(map.private.capacity(), size_of::<bool>())?;
        }
        add(self.signal_slots.capacity(), size_of::<Key>())?;
        add(self.cancelled_signals.capacity(), size_of::<NodeId>())?;
        add(self.native_tasks.capacity(), size_of::<select::Pending>())?;
        add(
            self.cancelled_selects.capacity(),
            size_of::<(Option<u64>, NodeId)>(),
        )?;
        add(
            self.details_intentions.capacity(),
            size_of::<details::Pending<Key>>(),
        )?;
        add(
            self.cancelled_details.capacity(),
            size_of::<details::Pending<NodeId>>(),
        )?;
        add(self.effects.capacity(), size_of::<select::EffectFrame>())?;
        add(self.select_temporary_bytes.load(Ordering::Relaxed), 1)?;
        add(
            self.view_transitions
                .source_storage_bytes(|| self.source_step(cancel))?,
            1,
        )?;
        for node in self.existing.iter().chain(&self.created).flatten() {
            check_cancel(cancel)?;
            self.source_step(cancel)?;
            payload.append(super::storage::node(node, || self.source_step(cancel))?)?;
            add(node.children.capacity(), size_of::<Key>())?;
            add(node.manual_nodes.capacity(), size_of::<Key>())?;
            add(node.assigned_nodes.capacity(), size_of::<Key>())?;
            if let Some(crate::dialog::DocumentInteractiveState::Known {
                open_dialogs,
                top_layer,
            }) = &node.interactive_document
            {
                add(open_dialogs.capacity(), size_of::<Key>())?;
                add(top_layer.capacity(), size_of::<Key>())?;
            }
            if let Some(d) = &node.base_document {
                add(d.render_blocking.capacity(), size_of::<Key>())?;
            }
        }
        if let Some(limits) = self.source_limits {
            admit(
                payload
                    .units
                    .checked_add(self.source_other_units)
                    .and_then(|n| {
                        n.checked_add(self.source_temporary_units.load(Ordering::Relaxed))
                    })
                    .ok_or(DomError::RepresentationExhausted)?,
                limits.retained_units,
                DomResource::StyleRetainedUnits,
            )?;
        }
        bytes
            .checked_add(payload.bytes)
            .ok_or(DomError::RepresentationExhausted)
    }
}

//! Actual temporary publication banks while the private Draft/checkpoint remain
//! live. Their charge ends only at discard or canonical ownership transfer.
use super::*;
use std::cell::Cell;

pub(crate) struct Formation {
    base: usize,
    limit: usize,
    live: Arc<AtomicUsize>,
    own: Cell<usize>,
    base_units: usize,
    unit_limit: usize,
    live_units: Arc<AtomicUsize>,
    own_units: Cell<usize>,
    work: Arc<AtomicU64>,
    ancestors: Option<Arc<crate::source::event::EventWork>>,
    work_limit: u64,
}
impl Drop for Formation {
    fn drop(&mut self) {
        self.live.fetch_sub(self.own.get(), Ordering::Relaxed);
        self.live_units
            .fetch_sub(self.own_units.get(), Ordering::Relaxed);
    }
}
impl Formation {
    pub(in crate::store) fn new(
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<Option<Self>, DomError> {
        let Some(limits) = draft.source_limits else {
            return Ok(None);
        };
        draft.source_admit(0, cancel)?;
        Ok(Some(Self {
            base: draft
                .source_storage_bytes(cancel)?
                .checked_add(draft.source_other_bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            limit: limits.retained_bytes,
            live: draft.source_formation_bytes.clone(),
            own: Cell::new(0),
            base_units: draft
                .source_storage_units(cancel)?
                .checked_add(draft.source_other_units)
                .ok_or(DomError::RepresentationExhausted)?,
            unit_limit: limits.retained_units,
            live_units: draft.source_temporary_units.clone(),
            own_units: Cell::new(0),
            work: draft.source_work.clone(),
            ancestors: draft.source_ancestors.clone(),
            work_limit: limits.work,
        }))
    }
    pub(in crate::store) fn from_event(
        work: &Arc<crate::source::event::EventWork>,
        base: storage::Payload,
        limits: crate::DomSourceLimits,
    ) -> Self {
        Self {
            base: base.bytes,
            base_units: base.units,
            limit: limits.retained_bytes,
            unit_limit: limits.retained_units,
            live: work.bytes.clone(),
            live_units: work.units.clone(),
            own: Cell::new(0),
            own_units: Cell::new(0),
            work: work.work.clone(),
            work_limit: limits.work,
            ancestors: work.ancestors.clone(),
        }
    }
    /// A distinct temporary custody guard over these same actual live atoms.
    /// No budget or spent-work counter is renewed.
    pub(in crate::store) fn fork(&self) -> Self {
        Self {
            base: self.base,
            limit: self.limit,
            live: self.live.clone(),
            own: Cell::new(0),
            base_units: self.base_units,
            unit_limit: self.unit_limit,
            live_units: self.live_units.clone(),
            own_units: Cell::new(0),
            work: self.work.clone(),
            ancestors: self.ancestors.clone(),
            work_limit: self.work_limit,
        }
    }
    /// Transfer an already admitted synchronous candidate into its publication
    /// owner without releasing its live debit between the two owners.
    pub(in crate::store) fn absorb(&self, other: Self) {
        assert!(
            Arc::ptr_eq(&self.live, &other.live)
                && Arc::ptr_eq(&self.live_units, &other.live_units),
            "same original Source formation"
        );
        self.own.set(
            self.own
                .get()
                .checked_add(other.own.replace(0))
                .expect("already admitted simultaneous payload"),
        );
        self.own_units.set(
            self.own_units
                .get()
                .checked_add(other.own_units.replace(0))
                .expect("already admitted simultaneous units"),
        );
    }
    pub(crate) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let next = self
            .work
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.work_limit {
            return Err(DomError::SourceWorkLimit {
                limit: self.work_limit,
            });
        }
        self.work.store(next, Ordering::Relaxed);
        crate::source::event::step_ancestors(self.ancestors.as_ref(), cancel)?;
        Ok(())
    }
    pub(in crate::store) fn retained_byte_limit(&self) -> usize {
        self.limit
    }
    pub(in crate::store) fn remaining_bytes(&self) -> Result<usize, DomError> {
        self.limit
            .checked_sub(self.base)
            .and_then(|bytes| bytes.checked_sub(self.live.load(Ordering::Relaxed)))
            .ok_or(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: self.limit,
            })
    }
    pub(crate) fn retain_bytes(&self, bytes: usize, cancel: &AtomicBool) -> Result<(), DomError> {
        self.retain_payload(storage::Payload { bytes, units: 0 }, cancel)
    }
    pub(crate) fn retain_payload(
        &self,
        payload: storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        self.before(payload.bytes)?;
        let units = self
            .live_units
            .load(Ordering::Relaxed)
            .checked_add(payload.units)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            self.base_units
                .checked_add(units)
                .ok_or(DomError::RepresentationExhausted)?,
            self.unit_limit,
            DomResource::SourceRetainedUnits,
        )?;
        let own_units = self
            .own_units
            .get()
            .checked_add(payload.units)
            .ok_or(DomError::RepresentationExhausted)?;
        let own = self
            .own
            .get()
            .checked_add(payload.bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        // Only this exclusive canonical execution adds; independent Send job
        // guards may release concurrently. Admission used the larger observed
        // extent, and relative atomics must preserve those releases.
        self.live.fetch_add(payload.bytes, Ordering::Relaxed);
        self.own.set(own);
        self.live_units.fetch_add(payload.units, Ordering::Relaxed);
        self.own_units.set(own_units);
        Ok(())
    }
    fn before(&self, new: usize) -> Result<(), DomError> {
        admit(
            self.base
                .checked_add(self.live.load(Ordering::Relaxed))
                .and_then(|v| v.checked_add(new))
                .ok_or(DomError::RepresentationExhausted)?,
            self.limit,
            DomResource::SourceRetainedBytes,
        )
    }
    fn before_units(&self, new: usize) -> Result<(), DomError> {
        admit(
            self.base_units
                .checked_add(self.live_units.load(Ordering::Relaxed))
                .and_then(|n| n.checked_add(new))
                .ok_or(DomError::RepresentationExhausted)?,
            self.unit_limit,
            DomResource::SourceRetainedUnits,
        )
    }
    fn reserve<T>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
        utf16: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        let required = values
            .len()
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if required <= values.capacity() {
            return self.before(0);
        }
        let maximum = isize::MAX as usize / size_of::<T>().max(1);
        let target = values
            .capacity()
            .checked_mul(2)
            .filter(|n| *n <= maximum)
            .unwrap_or(required)
            .max(required);
        let new = target
            .checked_mul(size_of::<T>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        let old_capacity = values.capacity();
        let old = old_capacity
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        self.before(new)?;
        if utf16 {
            self.before_units(target)?;
        }
        values
            .try_reserve_exact(target - values.len())
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        // Exact growth owns the requested policy; checking actual capacity also
        // protects implementations with greater retained capacity.
        self.before(actual)?;
        if utf16 {
            self.before_units(values.capacity())?;
        }
        let growth = actual
            .checked_sub(old)
            .ok_or(DomError::RepresentationExhausted)?;
        self.live.fetch_add(growth, Ordering::Relaxed);
        self.own.set(
            self.own
                .get()
                .checked_add(growth)
                .ok_or(DomError::RepresentationExhausted)?,
        );
        if utf16 {
            let growth = values.capacity() - old_capacity;
            self.live_units.fetch_add(growth, Ordering::Relaxed);
            self.own_units.set(
                self.own_units
                    .get()
                    .checked_add(growth)
                    .ok_or(DomError::RepresentationExhausted)?,
            );
        }
        self.step(cancel)
    }
    pub(in crate::store) fn into_charge(self) -> super::checkpoint::Charge {
        let bytes = self.own.replace(0);
        let units = self.own_units.replace(0);
        super::checkpoint::Charge::payload(self.live.clone(), bytes, self.live_units.clone(), units)
    }
    pub(in crate::store) fn owned_payload(&self) -> storage::Payload {
        storage::Payload {
            bytes: self.own.get(),
            units: self.own_units.get(),
        }
    }
    /// Transfer custody of an admitted part without changing the shared live
    /// debit. The actual payload must move with the returned charge.
    pub(in crate::store) fn split_payload(
        &self,
        payload: storage::Payload,
    ) -> super::checkpoint::Charge {
        self.own.set(
            self.own
                .get()
                .checked_sub(payload.bytes)
                .expect("owned payload bytes"),
        );
        self.own_units.set(
            self.own_units
                .get()
                .checked_sub(payload.units)
                .expect("owned payload units"),
        );
        super::checkpoint::Charge::payload(
            self.live.clone(),
            payload.bytes,
            self.live_units.clone(),
            payload.units,
        )
    }
    pub(crate) fn release_payload(&self, payload: storage::Payload) {
        self.own.set(
            self.own
                .get()
                .checked_sub(payload.bytes)
                .expect("owned payload bytes"),
        );
        self.own_units.set(
            self.own_units
                .get()
                .checked_sub(payload.units)
                .expect("owned payload units"),
        );
        self.live.fetch_sub(payload.bytes, Ordering::Relaxed);
        self.live_units.fetch_sub(payload.units, Ordering::Relaxed);
    }
    pub(in crate::store) fn release_bytes(&self, bytes: usize) {
        self.own.set(self.own.get() - bytes);
        self.live.fetch_sub(bytes, Ordering::Relaxed);
    }
    #[cfg(test)]
    pub(in crate::store) fn observe_candidate(
        &self,
        slots: &Vec<Slot>,
        occupants: usize,
        ids: usize,
        mapped: usize,
    ) {
        candidate_observation::observe(
            self.base,
            self.base + self.live.load(Ordering::Relaxed),
            self.limit,
            slots,
            occupants,
            ids,
            mapped,
        );
    }
}
pub(crate) fn reserve<T>(
    values: &mut Vec<T>,
    additional: usize,
    formation: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    match formation {
        Some(f) => f.reserve(values, additional, false, cancel),
        None => super::super::reserve(values, additional),
    }
}

/// Grow an actual UTF16 bank with the same byte policy and an explicit unit
/// extent. Other two-byte element types do not acquire UTF16 classification.
pub(crate) fn reserve_utf16(
    values: &mut Vec<u16>,
    additional: usize,
    formation: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    match formation {
        Some(f) => f.reserve(values, additional, true, cancel),
        None => super::super::reserve(values, additional),
    }
}

/// Materialize the original native factory's exact text under its existing
/// Source formation custody. Ordinary factories retain their prior path.
pub(crate) fn string(
    text: &str,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    let Some(budget) = budget else {
        return Ok(text.into());
    };
    budget.step(cancel)?;
    // Admit the UTF8 scan before deriving its exact UTF16 output length.
    for _ in 0..text.len() {
        budget.step(cancel)?;
    }
    let length = text.encode_utf16().count();
    budget.retain_payload(
        storage::Payload {
            bytes: size_of::<Vec<u16>>(),
            units: length,
        },
        cancel,
    )?;
    let mut units = Vec::new();
    reserve(&mut units, length, Some(budget), cancel)?;
    budget.retain_payload(
        storage::Payload {
            bytes: 0,
            units: units
                .capacity()
                .checked_sub(length)
                .ok_or(DomError::RepresentationExhausted)?,
        },
        cancel,
    )?;
    for unit in text.encode_utf16() {
        budget.step(cancel)?;
        units.push(unit);
    }
    Ok(DocumentString::from_utf16(units))
}

/// Fresh exact UTF16 output under its actual private owner. Its Vec header and
/// capacity remain owned by the caller's formation through immutable transfer.
pub(crate) fn utf16_buffer(
    length: usize,
    limit: usize,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<Vec<u16>, DomError> {
    admit(length, limit, DomResource::StringCodeUnits)?;
    length
        .checked_mul(size_of::<u16>())
        .filter(|n| *n <= isize::MAX as usize)
        .ok_or(DomError::RepresentationExhausted)?;
    if let Some(budget) = budget {
        budget.retain_payload(
            storage::Payload {
                bytes: size_of::<Vec<u16>>(),
                units: length,
            },
            cancel,
        )?;
    }
    let mut output = Vec::new();
    if budget.is_some() {
        reserve(&mut output, length, budget, cancel)?;
    } else {
        output
            .try_reserve_exact(length)
            .map_err(|_| DomError::AllocationFailed)?;
    }
    admit(output.capacity(), limit, DomResource::StringCodeUnits)?;
    if let Some(budget) = budget {
        budget.retain_payload(
            storage::Payload {
                bytes: 0,
                units: output
                    .capacity()
                    .checked_sub(length)
                    .ok_or(DomError::RepresentationExhausted)?,
            },
            cancel,
        )?;
    }
    Ok(output)
}

/// Copies an actual borrowed UTF16 operand under the same private formation
/// lease. The returned string remains covered until its caller transfers it.
pub(crate) fn units(
    input: &[u16],
    lowercase: bool,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    if let Some(budget) = budget {
        budget.retain_payload(
            storage::Payload {
                bytes: size_of::<Vec<u16>>(),
                units: input.len(),
            },
            cancel,
        )?;
    }
    let mut output = Vec::new();
    reserve(&mut output, input.len(), budget, cancel)?;
    if let Some(budget) = budget {
        budget.retain_payload(
            storage::Payload {
                bytes: 0,
                units: output
                    .capacity()
                    .checked_sub(input.len())
                    .ok_or(DomError::RepresentationExhausted)?,
            },
            cancel,
        )?;
    }
    for unit in input {
        step(budget, cancel)?;
        output.push(if lowercase {
            crate::names::ascii_lower(*unit)
        } else {
            *unit
        });
    }
    Ok(DocumentString::from_utf16(output))
}
pub(crate) fn step(budget: Option<&Formation>, cancel: &AtomicBool) -> Result<(), DomError> {
    match budget {
        Some(budget) => budget.step(cancel),
        None => check_cancel(cancel),
    }
}

pub(in crate::store) fn merge_charge(
    retained: &mut Option<super::checkpoint::Charge>,
    incoming: Option<super::checkpoint::Charge>,
) {
    if let Some(incoming) = incoming {
        if let Some(retained) = retained {
            retained.merge(incoming);
        } else {
            *retained = Some(incoming);
        }
    }
}
pub(crate) fn share(
    value: &DocumentString,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    if let Some(budget) = budget {
        let mut payload = storage::Payload::default();
        payload.string(value)?;
        budget.retain_payload(payload, cancel)?;
    }
    Ok(value.clone())
}
pub(in crate::store) fn shared_string(
    draft: &Draft,
    value: &DocumentString,
    cancel: &AtomicBool,
) -> Result<(DocumentString, Option<super::checkpoint::Charge>), DomError> {
    let formation = Formation::new(draft, cancel)?;
    let value = share(value, formation.as_ref(), cancel)?;
    Ok((value, formation.map(Formation::into_charge)))
}

/// Scalar-only observation at the actual populated Slot/occupants overlap.
/// Enabled only by the dedicated Document/Text fixture; no model allocation or
/// surrogate candidate bank participates in the subject operation.
#[cfg(test)]
pub(in crate::store) mod candidate_observation {
    use super::*;
    #[derive(Clone, Copy, Debug)]
    pub struct Boundary {
        pub required: usize,
        pub admitted: usize,
        pub limit: usize,
        pub occupants: usize,
    }
    thread_local! {
        static ENABLED: Cell<bool> = const { Cell::new(false) };
        static LAST: Cell<Option<Boundary>> = const { Cell::new(None) };
    }
    pub fn start() {
        LAST.set(None);
        ENABLED.set(true);
    }
    pub fn finish() -> Option<Boundary> {
        ENABLED.set(false);
        LAST.take()
    }
    pub(super) fn observe(
        base: usize,
        admitted: usize,
        limit: usize,
        slots: &Vec<Slot>,
        occupants: usize,
        ids: usize,
        mapped: usize,
    ) {
        if !ENABLED.get() {
            return;
        }
        let mut bytes = slots.capacity() * size_of::<Slot>()
            + occupants
            + ids * size_of::<Option<NodeId>>()
            + mapped * size_of::<NodeId>();
        let mut text = false;
        for node in slots.iter().filter_map(|slot| slot.node.as_ref()) {
            bytes += 4 * size_of::<Vec<NodeId>>();
            if let Some(canonical::Autofocus::Known { candidates, .. }) = &node.autofocus_document {
                bytes += size_of::<Vec<NodeId>>() + candidates.capacity() * size_of::<NodeId>();
            }
            if let Some(canonical::Interactive::Known {
                open_dialogs,
                top_layer,
                pending_top_layer_removals,
            }) = &node.interactive_document
            {
                bytes += 3 * size_of::<Vec<NodeId>>()
                    + (open_dialogs.capacity()
                        + top_layer.capacity()
                        + pending_top_layer_removals.capacity())
                        * size_of::<NodeId>();
            }

            bytes += node.children.capacity() * size_of::<NodeId>()
                + node.manual_nodes.capacity() * size_of::<NodeId>()
                + node.assigned_nodes.capacity() * size_of::<NodeId>();
            match &node.kind {
                DomNodeKind::Document(metadata) => {
                    bytes += metadata.retained_payload_bytes().unwrap()
                }
                DomNodeKind::Text(value) => {
                    bytes += size_of::<Vec<u16>>()
                        + value.shared_code_units().capacity() * size_of::<u16>();
                    text = true;
                }
                _ => panic!("candidate observer is enabled only for the Document/Text fixture"),
            }
            if let Some(document) = &node.base_document {
                assert!(document.facts.is_none());
                bytes += size_of::<Vec<NodeId>>()
                    + document.render_blocking.capacity() * size_of::<NodeId>();
            }
            assert!(
                node.image.is_none()
                    && node.hyperlink.is_none()
                    && node
                        .encoding_url_document
                        .as_ref()
                        .is_none_or(|document| document.facts.is_none())
                    && node.form_past_names.is_none()
                    && node.base.is_none()
            );
        }
        if text {
            LAST.set(Some(Boundary {
                required: base + bytes,
                admitted,
                limit,
                occupants,
            }));
        }
    }
}

#[cfg(test)]
pub(in crate::store) mod mutation_observation {
    use super::*;
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Site {
        Children,
        SelectTasks,
        Metadata,
        Family,
        Output,
        TextareaApi,
        InputString,
        CustomValidity,
        ReplaceData,
        InputAlias,
        InputRadioName,
        HyperlinkSnapshot,
        EncodingFacts,
        BaseFacts,
    }
    #[derive(Clone, Copy, Debug)]
    pub struct Growth {
        pub required: usize,
        pub limit: usize,
        pub required_units: usize,
        pub unit_limit: usize,
    }
    thread_local! {
        static ACTIVE: Cell<Option<Site>> = const { Cell::new(None) };
        static GROWTH: Cell<Option<Growth>> = const { Cell::new(None) };
    }
    pub fn start(site: Site) {
        ACTIVE.set(Some(site));
        GROWTH.set(None);
    }
    pub fn finish() -> Option<Growth> {
        ACTIVE.set(None);
        GROWTH.take()
    }
    pub fn before(
        draft: &Draft,
        site: Site,
        cancel: &AtomicBool,
    ) -> Result<Option<Growth>, DomError> {
        if ACTIVE.get() != Some(site) || GROWTH.get().is_some() {
            return Ok(None);
        }
        let Some(limits) = draft.source_limits else {
            return Ok(None);
        };
        Ok(Some(Growth {
            required: draft
                .source_storage_bytes(cancel)?
                .checked_add(draft.source_other_bytes)
                .and_then(|n| n.checked_add(draft.source_formation_bytes.load(Ordering::Relaxed)))
                .ok_or(DomError::RepresentationExhausted)?,
            limit: limits.retained_bytes,
            required_units: draft
                .source_storage_units(cancel)?
                .checked_add(draft.source_other_units)
                .and_then(|n| n.checked_add(draft.source_temporary_units.load(Ordering::Relaxed)))
                .ok_or(DomError::RepresentationExhausted)?,
            unit_limit: limits.retained_units,
        }))
    }
    pub fn grew(before: Option<Growth>, new_bank: usize) {
        grew_payload(before, new_bank, 0);
    }
    pub fn grew_payload(before: Option<Growth>, bytes: usize, units: usize) {
        if let Some(mut before) = before {
            before.required = before.required.checked_add(bytes).unwrap();
            before.required_units = before.required_units.checked_add(units).unwrap();
            GROWTH.set(Some(before));
        }
    }
}

//! Admission before the initial private Draft copies any canonical bank.
use super::super::*;
use super::storage::Payload;
use std::cell::Cell;

pub(in crate::store) struct Initial {
    limits: crate::DomSourceLimits,
    bytes: Cell<usize>,
    units: Cell<usize>,
    work: Work,
    ancestors: Option<Arc<crate::source::event::EventWork>>,
}
enum Work {
    Local(Cell<u64>),
    Shared(Arc<AtomicU64>),
}
/// Borrowed commands and their actual allocated backing, which can be zero
/// for an inline command even when the slice is nonempty.
struct CommandStorage<'a> {
    commands: &'a [Command],
    capacity: usize,
}
impl Work {
    fn get(&self) -> u64 {
        match self {
            Self::Local(work) => work.get(),
            Self::Shared(work) => work.load(Ordering::Relaxed),
        }
    }
    fn set(&self, value: u64) {
        match self {
            Self::Local(work) => work.set(value),
            Self::Shared(work) => work.store(value, Ordering::Relaxed),
        }
    }
}
impl Initial {
    pub(in crate::store) fn new(limits: crate::DomSourceLimits) -> Self {
        Self {
            limits,
            bytes: Cell::new(0),
            units: Cell::new(0),
            work: Work::Local(Cell::new(0)),
            ancestors: None,
        }
    }
    pub(in crate::store) fn with_work(
        limits: crate::DomSourceLimits,
        work: Arc<AtomicU64>,
    ) -> Self {
        Self {
            limits,
            bytes: Cell::new(0),
            units: Cell::new(0),
            work: Work::Shared(work),
            ancestors: None,
        }
    }
    pub(in crate::store) fn ancestors(&self) -> Option<&Arc<crate::source::event::EventWork>> {
        self.ancestors.as_ref()
    }
    fn with_ancestors(mut self, ancestors: Option<Arc<crate::source::event::EventWork>>) -> Self {
        self.ancestors = ancestors;
        self
    }
    pub(in crate::store) fn limits(&self) -> crate::DomSourceLimits {
        self.limits
    }
    pub(in crate::store) fn work(&self) -> u64 {
        self.work.get()
    }
    pub(in crate::store) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let next = self
            .work
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.limits.work {
            return Err(DomError::SourceWorkLimit {
                limit: self.limits.work,
            });
        }
        self.work.set(next);
        crate::source::event::step_ancestors(self.ancestors.as_ref(), cancel)?;
        Ok(())
    }
    pub(in crate::store) fn retain(
        &self,
        payload: Payload,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        let bytes = self
            .bytes
            .get()
            .checked_add(payload.bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let units = self
            .units
            .get()
            .checked_add(payload.units)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            bytes,
            self.limits.retained_bytes,
            DomResource::SourceRetainedBytes,
        )?;
        admit(
            units,
            self.limits.retained_units,
            DomResource::SourceRetainedUnits,
        )?;
        self.bytes.set(bytes);
        self.units.set(units);
        Ok(())
    }
    pub(in crate::store) fn reserve<T>(
        &self,
        values: &mut Vec<T>,
        count: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        debug_assert!(values.is_empty());
        self.step(cancel)?;
        let requested = count
            .checked_mul(size_of::<T>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            self.bytes
                .get()
                .checked_add(requested)
                .ok_or(DomError::RepresentationExhausted)?,
            self.limits.retained_bytes,
            DomResource::SourceRetainedBytes,
        )?;
        values
            .try_reserve_exact(count)
            .map_err(|_| DomError::AllocationFailed)?;
        self.retain(
            Payload {
                bytes: values
                    .capacity()
                    .checked_mul(size_of::<T>())
                    .ok_or(DomError::RepresentationExhausted)?,
                units: 0,
            },
            cancel,
        )
    }
}
pub(in crate::store) fn reserve<T>(
    values: &mut Vec<T>,
    count: usize,
    initial: Option<&Initial>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    match initial {
        Some(initial) => initial.reserve(values, count, cancel),
        None => super::super::reserve(values, count),
    }
}
pub(in crate::store) fn step(
    initial: Option<&Initial>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    match initial {
        Some(initial) => initial.step(cancel),
        None => check_cancel(cancel),
    }
}
pub(in crate::store) fn map_keys(
    source: &[NodeId],
    initial: Option<&Initial>,
    cancel: &AtomicBool,
) -> Result<Vec<Key>, DomError> {
    let mut out = Vec::new();
    reserve(&mut out, source.len(), initial, cancel)?;
    for id in source {
        step(initial, cancel)?;
        out.push(Key::Existing(id.slot));
    }
    Ok(out)
}
pub(in crate::store) fn copy<T: Copy>(
    source: &[T],
    initial: Option<&Initial>,
    cancel: &AtomicBool,
) -> Result<Vec<T>, DomError> {
    let mut out = Vec::new();
    reserve(&mut out, source.len(), initial, cancel)?;
    for value in source {
        step(initial, cancel)?;
        out.push(*value);
    }
    Ok(out)
}
pub(in crate::store) fn base_document(
    source: &crate::base::BaseDocument<NodeId>,
    initial: Option<&Initial>,
    cancel: &AtomicBool,
) -> Result<crate::base::BaseDocument<Key>, DomError> {
    if initial.is_none() {
        return source.map(|id| Key::Existing(id.slot));
    }
    Ok(crate::base::BaseDocument {
        facts: source.facts.clone(),
        first: source.first.map(|id| Key::Existing(id.slot)),
        render_blocking: map_keys(&source.render_blocking, initial, cancel)?,
    })
}
pub(in crate::store) fn form_names(
    source: &crate::form::FormPastNames<NodeId>,
    initial: Option<&Initial>,
    cancel: &AtomicBool,
) -> Result<crate::form::FormPastNames<Key>, DomError> {
    let mut entries = Vec::new();
    reserve(&mut entries, source.entries.len(), initial, cancel)?;
    for entry in source.entries.iter() {
        step(initial, cancel)?;
        entries.push(crate::form::FormPastName {
            name: entry.name.clone(),
            target: Key::Existing(entry.target.slot),
        });
    }
    Ok(crate::form::FormPastNames {
        entries: Arc::new(entries),
    })
}

impl Draft {
    pub(in crate::store) fn new_source(
        dom: &Dom,
        owner: Arc<BatchIdentity>,
        commands: &Vec<Command>,
        limits: crate::DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(Self, Payload, Payload), DomError> {
        Self::new_source_root(dom, owner, commands, commands.capacity(), limits, cancel)
    }
    pub(in crate::store) fn new_source_root(
        dom: &Dom,
        owner: Arc<BatchIdentity>,
        commands: &[Command],
        command_capacity: usize,
        limits: crate::DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(Self, Payload, Payload), DomError> {
        let initial = Initial::new(limits).with_ancestors(dom.source_work_ancestors(None));
        Self::new_source_initial(
            dom,
            owner,
            CommandStorage {
                commands,
                capacity: command_capacity,
            },
            initial,
            size_of::<super::operation::Operation>()
                .checked_sub(size_of::<Draft>())
                .ok_or(DomError::RepresentationExhausted)?,
            None,
            cancel,
        )
    }
    pub(in crate::store) fn new_source_root_with_work(
        dom: &Dom,
        owner: Arc<BatchIdentity>,
        commands: &[Command],
        command_capacity: usize,
        limits: crate::DomSourceLimits,
        work: Arc<AtomicU64>,
        cancel: &AtomicBool,
    ) -> Result<(Self, Payload, Payload), DomError> {
        let initial = Initial::with_work(limits, work.clone())
            .with_ancestors(dom.source_work_ancestors(Some(&work)));
        let (mut draft, canonical, command) = Self::new_source_initial(
            dom,
            owner,
            CommandStorage {
                commands,
                capacity: command_capacity,
            },
            initial,
            0,
            Some(work.clone()),
            cancel,
        )?;
        draft.source_work = work;
        Ok((draft, canonical, command))
    }
    pub(in crate::store) fn new_hosted_source(
        dom: &Dom,
        owner: Arc<BatchIdentity>,
        limits: crate::DomSourceLimits,
        work: Arc<AtomicU64>,
        operation: crate::hosted::DomHostedOperationId,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let initial = Initial::with_work(limits, work.clone())
            .with_ancestors(dom.source_work_ancestors(Some(&work)));
        let (mut draft, _, _) = Self::new_source_initial(
            dom,
            owner,
            CommandStorage {
                commands: &[],
                capacity: 0,
            },
            initial,
            0,
            None,
            cancel,
        )?;
        draft.source_work = work;
        draft.hosted_owner = Some(operation);
        Ok(draft)
    }
    fn new_source_initial(
        dom: &Dom,
        owner: Arc<BatchIdentity>,
        commands: CommandStorage<'_>,
        initial: Initial,
        owner_bytes: usize,
        shared_work: Option<Arc<AtomicU64>>,
        cancel: &AtomicBool,
    ) -> Result<(Self, Payload, Payload), DomError> {
        let CommandStorage {
            commands,
            capacity: command_capacity,
        } = commands;
        // The source front retains the ordinary owner's whole-batch canonical
        // quota policy. Meter the actual scan before any Draft retention or ID
        // issuance; later execution cannot reduce raw duplicate input counts.
        dom.admit_batch_inputs(commands, || initial.step(cancel))?;
        let canonical = super::storage::state(&dom.state, || initial.step(cancel))?;
        let command_payload = super::storage::commands(commands, || initial.step(cancel))?;
        let mut retained = canonical;
        retained.append(command_payload)?;
        retained.add_bytes(dom.history_bytes)?;
        retained.add_bytes(dom.source_retained_stack_bytes()?)?;
        retained.add_bytes(dom.source_owner_custody_bytes()?)?;
        retained.units = retained
            .units
            .checked_add(dom.fullscreen_custody_units()?)
            .ok_or(DomError::RepresentationExhausted)?;
        retained.add_bytes(dom.source_dispatch_custody_bytes(|| initial.step(cancel))?)?;
        retained.add_bytes(owner_bytes)?;
        retained.bank::<Command>(command_capacity)?;
        for command in commands {
            initial.step(cancel)?;
            if let Command::AssignSlot { incoming, .. } = command {
                retained.bank::<NodeOperand>(incoming.capacity())?;
            }
        }
        initial.retain(retained, cancel)?;
        // The canonical owner stays live while the initial Draft is formed.
        // Shared immutable payloads are charged per retained occurrence.
        initial.retain(
            super::storage::state_payload(&dom.state, || initial.step(cancel))?,
            cancel,
        )?;
        initial.retain(
            Payload {
                bytes: size_of::<Draft>() + size_of::<AtomicU64>() + 3 * size_of::<AtomicUsize>(),
                units: 0,
            },
            cancel,
        )?;
        let mut draft = Self::new_formed(dom, owner, Some(&initial), cancel)?;
        if let Some(work) = shared_work {
            draft.fullscreen_work_custody =
                dom.fullscreen_work_custody(&work, || initial.step(cancel))?;
            draft.source_work = work;
        }
        draft.source_other_bytes = retained.bytes;
        draft.source_other_units = retained.units;
        draft.phase_map = Some(super::PhaseMap {
            ids: Vec::new(),
            private: Vec::new(),
        });
        draft.source_admit(0, cancel)?;
        Ok((draft, canonical, command_payload))
    }
}

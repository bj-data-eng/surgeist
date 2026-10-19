//! Admission before the initial private Draft copies any canonical bank.
use super::super::*;
use super::{formation, storage::Payload};
use std::cell::Cell;

pub(in crate::store) struct Initial {
    limits: crate::DomSourceLimits,
    bytes: Cell<usize>,
    units: Cell<usize>,
    work: Work,
    ancestors: Option<Arc<crate::source::event::EventWork>>,
    ancestor_custody: Option<Arc<formation::EventWorkCustody>>,
    names: Option<name_registry::Operation>,
    #[cfg(test)]
    measurement_identity: Cell<Option<usize>>,
}
/// Original counters and name reservations of one resumed Source segment.
pub(in crate::store) struct Replay {
    pub work: Arc<AtomicU64>,
    pub identity: formation::SourceWorkIdentity,
    pub names: Option<name_registry::Operation>,
}
enum Work {
    Local(Cell<u64>),
    Shared {
        work: Arc<AtomicU64>,
        identity: formation::SourceWorkIdentity,
    },
}
/// Borrowed commands and their actual allocated backing, which can be zero
/// for an inline command even when the slice is nonempty.
pub(in crate::store) struct CommandStorage<'a> {
    pub(in crate::store) commands: &'a [Command],
    pub(in crate::store) capacity: usize,
}
impl Work {
    fn get(&self) -> u64 {
        match self {
            Self::Local(work) => work.get(),
            Self::Shared { work, .. } => work.load(Ordering::Relaxed),
        }
    }
    fn set(&self, value: u64) {
        match self {
            Self::Local(work) => work.set(value),
            Self::Shared { work, .. } => work.store(value, Ordering::Relaxed),
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
            ancestor_custody: None,
            names: None,
            #[cfg(test)]
            measurement_identity: Cell::new(None),
        }
    }
    pub(in crate::store) fn with_work(
        limits: crate::DomSourceLimits,
        work: Arc<AtomicU64>,
        identity: formation::SourceWorkIdentity,
    ) -> Self {
        Self {
            limits,
            bytes: Cell::new(0),
            units: Cell::new(0),
            work: Work::Shared { work, identity },
            ancestors: None,
            ancestor_custody: None,
            names: None,
            #[cfg(test)]
            measurement_identity: Cell::new(None),
        }
    }
    pub(in crate::store) fn names(&self) -> Option<&name_registry::Operation> {
        self.names.as_ref()
    }
    fn with_names(mut self, names: Option<name_registry::Operation>) -> Self {
        self.names = names;
        self
    }
    pub(in crate::store) fn ancestors(&self) -> Option<&Arc<crate::source::event::EventWork>> {
        self.ancestors.as_ref()
    }
    fn with_ancestors(
        mut self,
        ancestors: Option<Arc<crate::source::event::EventWork>>,
        custody: Option<Arc<formation::EventWorkCustody>>,
    ) -> Self {
        self.ancestors = ancestors;
        self.ancestor_custody = custody;
        self
    }
    pub(in crate::store) fn ancestor_custody(&self) -> Option<Arc<formation::EventWorkCustody>> {
        self.ancestor_custody.clone()
    }
    pub(in crate::store) fn source_work(
        &self,
        registry: &bank::CustodyRegistry,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<(Arc<AtomicU64>, formation::SourceWorkCustody), DomError> {
        match &self.work {
            Work::Local(_) => {
                let (work, custody) = formation::SourceWorkCustody::allocate(0, registry, admit)?;
                work.store(self.work.get(), Ordering::Relaxed);
                Ok((work, custody))
            }
            Work::Shared { work, identity } => Ok((
                work.clone(),
                formation::SourceWorkCustody::adopt_existing(*identity, work, registry, admit)?,
            )),
        }
    }
    /// Obligations outside the paired controlled-state ledger. The sparse
    /// constructor combines these with its proposed union before retaining any
    /// canonical capture or newly allocated transaction header.
    pub(in crate::store) fn external_usage(&self) -> custody::Usage {
        custody::Usage::new(self.bytes.get(), self.units.get())
    }
    pub(in crate::store) fn limits(&self) -> crate::DomSourceLimits {
        self.limits
    }

    fn finish_work(&self, work: &Arc<AtomicU64>) {
        if matches!(&self.work, Work::Local(_)) {
            // Constructor admission may spend after creating the target. The
            // original local meter joins that same atom before execution starts.
            work.store(self.work.get(), Ordering::Relaxed);
        }
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
        formation::SourceStorage::record_step();
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
        #[cfg(test)]
        if scale_metrics::ENABLED
            && let Some(identity) = self.measurement_identity.get()
        {
            scale_metrics::source_external_changed(identity, custody::Usage::new(bytes, units));
        }
        Ok(())
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
impl Draft {
    pub(in crate::store) fn new_source(
        dom: &Dom,
        owner: Arc<BatchIdentity>,
        commands: &Vec<Command>,
        limits: crate::DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(Self, Payload), DomError> {
        Self::new_source_root(dom, owner, commands, commands.capacity(), limits, cancel)
    }
    pub(in crate::store) fn new_source_root(
        dom: &Dom,
        owner: Arc<BatchIdentity>,
        commands: &[Command],
        command_capacity: usize,
        limits: crate::DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(Self, Payload), DomError> {
        let initial = Initial::new(limits).with_ancestors(
            dom.source_work_ancestors(None),
            dom.source_work_ancestor_custody(None),
        );
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
        commands: CommandStorage<'_>,
        limits: crate::DomSourceLimits,
        replay: Replay,
        cancel: &AtomicBool,
    ) -> Result<(Self, Payload), DomError> {
        let Replay {
            work,
            identity,
            names,
        } = replay;
        let initial = Initial::with_work(limits, work.clone(), identity)
            .with_names(names)
            .with_ancestors(
                dom.source_work_ancestors(Some(&work)),
                dom.source_work_ancestor_custody(Some(&work)),
            );
        let (mut draft, command) =
            Self::new_source_initial(dom, owner, commands, initial, 0, Some(work.clone()), cancel)?;
        draft.source_work = work;
        Ok((draft, command))
    }
    pub(in crate::store) fn new_hosted_source(
        dom: &Dom,
        owner: Arc<BatchIdentity>,
        limits: crate::DomSourceLimits,
        work: Arc<AtomicU64>,
        work_identity: formation::SourceWorkIdentity,
        operation: crate::hosted::DomHostedOperationId,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let initial = Initial::with_work(limits, work.clone(), work_identity).with_ancestors(
            dom.source_work_ancestors(Some(&work)),
            dom.source_work_ancestor_custody(Some(&work)),
        );
        let (mut draft, _) = Self::new_source_initial(
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
    ) -> Result<(Self, Payload), DomError> {
        formation::SourceStorage::record_entry();
        #[cfg(test)]
        if scale_metrics::ENABLED {
            let ledger = dom.custody_registry.ledger();
            let identity = ledger.measurement_identity();
            initial.measurement_identity.set(Some(identity));
            scale_metrics::source_owner_changed(
                identity,
                ledger.total(),
                initial.external_usage(),
                dom.source_measurement_ancestors(),
                None,
            );
        }
        let CommandStorage {
            commands,
            capacity: command_capacity,
        } = commands;
        // The source front retains the ordinary owner's whole-batch canonical
        // quota policy. Meter the actual scan before any Draft retention or ID
        // issuance; later execution cannot reduce raw duplicate input counts.
        dom.admit_batch_inputs(commands, || initial.step(cancel))?;
        let command_payload = super::storage::commands(commands, || initial.step(cancel))?;
        // Canonical capture and transaction storage are controlled by the
        // paired ledger. This separate envelope contains only external inputs
        // and owner obligations, so a root alias cannot duplicate its payload.
        let mut retained = Payload::default();
        retained.append(command_payload)?;
        retained.add_bytes(dom.source_retained_stack_bytes()?)?;
        retained.add_bytes(dom.source_owner_custody_bytes()?)?;
        retained.units = retained
            .units
            .checked_add(dom.fullscreen_custody_units()?)
            .ok_or(DomError::RepresentationExhausted)?;
        retained.add_bytes(dom.source_dispatch_custody_bytes(|| initial.step(cancel))?)?;
        retained.add_bytes(owner_bytes)?;
        retained.bank::<Command>(command_capacity)?;
        initial.retain(retained, cancel)?;
        let mut draft = Self::new_formed(dom, owner, Some(&initial), cancel)?;
        if let Some(work) = shared_work {
            draft.fullscreen_work_custody =
                dom.fullscreen_work_custody(&work, || initial.step(cancel))?;
            draft.source_work = work;
        }
        draft.source_other_bytes = retained.bytes;
        draft.source_other_units = retained.units;
        initial.finish_work(&draft.source_work);
        // Whole-snapshot import semantics retain the original input frontier
        // once for this Source. Called frames alias its owner-side capsule.
        for command in commands {
            draft.source_step(cancel)?;
            if let Command::Import { snapshot, .. } = command {
                draft.retain_input_capture(&snapshot.state, cancel)?;
            }
        }
        draft.phase_map = Some(super::PhaseMap {
            ids: super::AllocationIds::default(),
            private: super::Privacy::default(),
        });
        draft.source_admit(0, cancel)?;
        Ok((draft, command_payload))
    }
}

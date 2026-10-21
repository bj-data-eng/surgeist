//! Borrow the actual retained destruction row and native Cleanup task.
use super::*;
use std::cell::RefMut;

pub(crate) struct StyleCleanupSelection {
    index: usize,
    info: Info,
    owner: Rc<RefCell<records::WindowLoop>>,
    _retention: surgeist_dom::DomHostedRuntimeRetention,
}
pub(crate) struct StyleCleanupTask<'a> {
    _loop: RefMut<'a, records::WindowLoop>,
}
impl StyleCleanupSelection {
    pub(crate) fn hold(
        &self,
        context: &mut StyleCleanupContext<'_>,
        cancel: &AtomicBool,
    ) -> Result<StyleCleanupTask<'_>, DomHostedRuntimeError> {
        let event_loop = self
            .owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let current = records::CurrentTask::Destroy {
            operation: self.info.cleanup,
            document: self.info.document,
        };
        if event_loop.executing || event_loop.current != Some(current) {
            return Err(DomHostedRuntimeError::OperationPending);
        }
        let mut actual = None;
        for task in &event_loop.navigation {
            context.step(cancel)?;
            if let Some(task) = task.destroy()
                && task.operation == self.info.cleanup
            {
                actual = Some(task);
                break;
            }
        }
        let task = actual.ok_or(DomHostedRuntimeError::Protocol)?;
        if task.operation != self.info.cleanup
            || task.document != self.info.document
            || task.global != self.info.global
            || task.phase != records::DestroyPhase::Cleanup
        {
            return Err(DomHostedRuntimeError::OperationPending);
        }
        Ok(StyleCleanupTask { _loop: event_loop })
    }
}

pub(crate) struct StyleCleanupContext<'a> {
    pub(crate) source: &'a DomHostedSource,
    pub(crate) lease: &'a DomHostedDestructionLease,
    pub(crate) publication: &'a DomHostedDestructionPublication,
    work: &'a mut u64,
    limit: u64,
    pub(crate) invocation: DomHostedOperationId,
    pub(crate) document: surgeist_dom::DocumentId,
}
impl StyleCleanupContext<'_> {
    pub(crate) fn step(&mut self, cancel: &AtomicBool) -> Result<(), DomHostedRuntimeError> {
        spend(self.work, self.limit, cancel)
    }
}
impl HostedState {
    pub(crate) fn select_style_cleanup(
        &mut self,
        dom: &mut DomEventDom<'_>,
        invocation: DomHostedOperationId,
        cancel: &AtomicBool,
    ) -> Result<StyleCleanupSelection, DomHostedRuntimeError> {
        self.qualify(dom)?;
        if self.creation.is_some() {
            return Err(DomHostedRuntimeError::OperationPending);
        }
        let mut actual = None;
        for row in &self.destruction.invocations {
            spend(&mut self.work, self.limits.work, cancel)?;
            if row.permission.operation() == invocation {
                let info = row.info.ok_or(DomHostedRuntimeError::OperationPending)?;
                let request = row
                    .permission
                    .request(dom.raw())
                    .map_err(DomHostedRuntimeError::Dom)?;
                if request.cleanup_operation() != info.cleanup
                    || request.document() != info.document
                    || request.global() != info.global
                {
                    return Err(DomHostedRuntimeError::Binding);
                }
                actual = Some(info);
                break;
            }
        }
        let info = actual.ok_or(DomHostedRuntimeError::TargetNotFound)?;
        let mut selected = None;
        for (index, row) in self.destruction.cleanups.iter().enumerate() {
            spend(&mut self.work, self.limits.work, cancel)?;
            if row.info.operation == invocation && row.info.cleanup == info.cleanup {
                if row.info.document != info.document
                    || row.info.global != info.global
                    || row.lease.operation() != info.cleanup
                {
                    return Err(DomHostedRuntimeError::Binding);
                }
                if !matches!(
                    row.lease
                        .step(dom.raw())
                        .map_err(DomHostedRuntimeError::Dom)?,
                    DomHostedDestructionStep::Cleanup(_)
                ) || row.publication.is_none()
                {
                    return Err(DomHostedRuntimeError::OperationPending);
                }
                selected = Some(index);
                break;
            }
        }
        let index = selected.ok_or(DomHostedRuntimeError::OperationPending)?;
        let (h, t, c, g) = self.find_global(info.global, cancel, || Ok(()))?;
        let frame = size_of::<StyleCleanupSelection>()
            + size_of::<StyleCleanupContext<'_>>()
            + size_of::<StyleCleanupTask<'_>>();
        let retained = self.retained_bytes(cancel)?;
        if frame > self.limits.scratch_bytes
            || retained
                .checked_add(frame)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        let retention = self
            .source
            .retain_runtime_payload(dom.raw_mut(), frame, cancel)
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        self.step(cancel)?;
        let cluster = &self.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters[c];
        if cluster.agent.globals[g].settings.document != Some(info.document) {
            return Err(DomHostedRuntimeError::Binding);
        }
        Ok(StyleCleanupSelection {
            index,
            info,
            owner: cluster.agent.event_loop.clone(),
            _retention: retention,
        })
    }
    pub(crate) fn style_cleanup_context<'a>(
        &'a mut self,
        selected: &StyleCleanupSelection,
        dom: &DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<StyleCleanupContext<'a>, DomHostedRuntimeError> {
        self.qualify(dom)?;
        self.step(cancel)?;
        let row = self
            .destruction
            .cleanups
            .get(selected.index)
            .ok_or(DomHostedRuntimeError::TargetNotFound)?;
        if row.info.operation != selected.info.operation
            || row.info.cleanup != selected.info.cleanup
            || row.info.document != selected.info.document
            || row.info.global != selected.info.global
        {
            return Err(DomHostedRuntimeError::Binding);
        }
        Ok(StyleCleanupContext {
            source: &self.source,
            lease: &row.lease,
            publication: row
                .publication
                .as_ref()
                .ok_or(DomHostedRuntimeError::OperationPending)?,
            work: &mut self.work,
            limit: self.limits.work,
            invocation: row.info.operation,
            document: row.info.document,
        })
    }
}
impl HostedRuntime {
    pub(crate) fn style_withdrawal_owner(
        &self,
    ) -> Result<Rc<RefCell<HostedState>>, DomHostedRuntimeError> {
        self.state()
    }
}

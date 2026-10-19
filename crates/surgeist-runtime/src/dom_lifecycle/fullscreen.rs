//! Scheduling custody for the canonical Fullscreen exit invocation.
//! The DOM job owns the algorithm. This bank owns only its original route and
//! whether the scheduler has offered its execution permission since handoff.
use super::*;
use surgeist_dom::{
    DomFullscreenExitId, DomFullscreenExitJob, DomHostedRuntimeRetention,
    DomSourceFullscreenExitHandoffRequest, DomSourceLease,
};

pub(super) struct Jobs {
    pub rows: Vec<Row>,
    // Drop represented backing before releasing its original Source debit.
    backing: Option<DomHostedRuntimeRetention>,
}
impl Jobs {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            backing: None,
        }
    }
    pub fn retained_bytes(&self) -> Result<usize, DomHostedRuntimeError> {
        self.rows
            .capacity()
            .checked_mul(size_of::<Row>())
            .ok_or(DomHostedRuntimeError::Admission)
    }
}

pub(super) struct Row {
    invocation: DomFullscreenExitId,
    global: DomHostedGlobalId,
    // None is a real reserved slot, never runnable until canonical publication.
    permission: Option<DomFullscreenExitJob>,
    ready: bool,
}

fn source_work(
    lease: &DomSourceLease<'_>,
    cancel: &AtomicBool,
) -> Result<(), DomHostedRuntimeError> {
    lease
        .admit_hosted_runtime_work(1, cancel)
        .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
}

impl HostedState {
    fn reserve_fullscreen_row(
        &mut self,
        h: usize,
        t: usize,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceFullscreenExitHandoffRequest,
        cancel: &AtomicBool,
    ) -> Result<usize, DomHostedRuntimeError> {
        let length = self.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .fullscreen
            .rows
            .len();
        for index in 0..length {
            self.step(cancel)?;
            source_work(lease, cancel)?;
            let row = &self.hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .fullscreen
                .rows[index];
            if row.invocation == request.invocation() {
                // A committed request advances in its original DOM owner. A
                // copied old request cannot create or replace that job.
                return Err(DomHostedRuntimeError::Protocol);
            }
        }
        let count = native_task_count_admitted(
            &self.hosts,
            &mut self.work,
            self.limits.work,
            cancel,
            || source_work(lease, cancel),
        )?;
        if count >= self.limits.tasks {
            return Err(DomHostedRuntimeError::Admission);
        }
        let capacity = self.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .fullscreen
            .rows
            .capacity();
        if length == capacity {
            let required = length
                .checked_add(1)
                .ok_or(DomHostedRuntimeError::Admission)?;
            let retained = self.source_retained_bytes(lease, cancel)?;
            let requested = required
                .checked_mul(size_of::<Row>())
                .ok_or(DomHostedRuntimeError::Admission)?;
            if requested > self.limits.scratch_bytes
                || retained
                    .checked_add(requested)
                    .is_none_or(|bytes| bytes > self.limits.storage_bytes)
            {
                return Err(DomHostedRuntimeError::Admission);
            }
            let mut backing = lease
                .retain_hosted_runtime_payload(&self.source, requested, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            // Admit every move and the eventual reserved-slot write before
            // allocation. The old bank and its debit remain live throughout.
            for _ in 0..required {
                self.step(cancel)?;
                source_work(lease, cancel)?;
            }
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(required)
                .map_err(|_| DomHostedRuntimeError::Admission)?;
            let actual = replacement
                .capacity()
                .checked_mul(size_of::<Row>())
                .ok_or(DomHostedRuntimeError::Admission)?;
            if actual > self.limits.scratch_bytes
                || retained
                    .checked_add(actual)
                    .is_none_or(|bytes| bytes > self.limits.storage_bytes)
            {
                return Err(DomHostedRuntimeError::Admission);
            }
            if actual > requested {
                backing
                    .absorb(
                        lease
                            .retain_hosted_runtime_payload(&self.source, actual - requested, cancel)
                            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?,
                    )
                    .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            }
            let jobs = &mut self.hosts[h].top_levels[t]
                .group
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .fullscreen;
            replacement.append(&mut jobs.rows);
            jobs.rows = replacement;
            jobs.backing = Some(backing);
        }
        self.step(cancel)?;
        source_work(lease, cancel)?;
        let jobs = &mut self.hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .fullscreen;
        jobs.rows.push(Row {
            invocation: request.invocation(),
            global: request.global(),
            permission: None,
            ready: false,
        });
        Ok(length)
    }

    /// Offer one actual parallel permission when ordinary queued work is idle.
    /// A passive service wait has no WindowLoop task or retained canonical loan.
    pub(super) fn pump_fullscreen(
        &mut self,
        dom: &mut DomEventDom<'_>,
        h: usize,
        t: usize,
        cancel: &AtomicBool,
    ) -> Result<Option<DomHostedTaskProgress>, DomHostedRuntimeError> {
        let length = self.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .fullscreen
            .rows
            .len();
        if length == 0 {
            return Ok(None);
        }
        let clusters = self.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters
            .len();
        for cluster in 0..clusters {
            self.step(cancel)?;
            let event_loop = self.hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .clusters[cluster]
                .agent
                .event_loop
                .try_borrow()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            if event_loop.executing || event_loop.current.is_some() {
                return Err(DomHostedRuntimeError::Busy);
            }
        }
        for index in 0..length {
            self.step(cancel)?;
            let row = &self.hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .fullscreen
                .rows[index];
            if !row.ready {
                continue;
            }
            let global = row.global;
            let (route_h, route_t, _, _) = self.find_global(global, cancel, || Ok(()))?;
            if (route_h, route_t) != (h, t) {
                return Err(DomHostedRuntimeError::Binding);
            }
            // The row stays in its original owner on every refusal. DOM alone
            // chooses the source action and retains its selected Document/resize.
            let Self { source, hosts, .. } = self;
            let row = &mut hosts[h].top_levels[t]
                .group
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .fullscreen
                .rows[index];
            let permission = row
                .permission
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            let status = dom
                .raw_mut()
                .resume_fullscreen_exit(source, permission, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            row.ready = false;
            return Ok(Some(DomHostedTaskProgress::FullscreenExitPending {
                invocation: status.id(),
                step: status.step(),
            }));
        }
        Ok(Some(DomHostedTaskProgress::Quiescent))
    }
}

// The unissued row is affine scheduling custody. A refusal or unwind releases
// only this reservation; admitted spare backing remains with the actual bank.
struct ReservedRow<'a> {
    jobs: &'a mut Jobs,
    index: usize,
    installed: bool,
}
impl ReservedRow<'_> {
    fn install(mut self, permission: DomFullscreenExitJob) {
        let row = &mut self.jobs.rows[self.index];
        row.permission = Some(permission);
        row.ready = true;
        self.installed = true;
    }
}
impl Drop for ReservedRow<'_> {
    fn drop(&mut self) {
        if !self.installed {
            debug_assert_eq!(self.index + 1, self.jobs.rows.len());
            self.jobs.rows.truncate(self.index);
        }
    }
}

impl HostedRuntime {
    pub(crate) fn service_fullscreen_exit_handoff(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceFullscreenExitHandoffRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        self.service_fullscreen_handoff(lease, request, cancel, |lease, source| {
            let permission = lease
                .prepare_fullscreen_exit_handoff(request, source, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?
                .publish();
            Ok((permission, ()))
        })
    }

    pub(crate) fn service_fullscreen_exit_handoff_with_inline_styles(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceFullscreenExitHandoffRequest,
        resolver: &mut crate::dom_source::InlineResolver<'_>,
        cancel: &AtomicBool,
    ) -> Result<
        Option<surgeist_dom::DomInlineStyleSourcePublication>,
        crate::dom_source::DomSourceRuntimeError,
    > {
        self.service_fullscreen_handoff(lease, request, cancel, |lease, source| {
            lease
                .commit_fullscreen_exit_handoff_with_inline_styles(
                    request, source, resolver, cancel,
                )
                .map_err(crate::dom_source::DomSourceRuntimeError::Inline)
        })
    }

    fn service_fullscreen_handoff<T, E>(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceFullscreenExitHandoffRequest,
        cancel: &AtomicBool,
        commit: impl FnOnce(
            &mut DomSourceLease<'_>,
            &surgeist_dom::DomHostedSource,
        ) -> Result<(DomFullscreenExitJob, T), E>,
    ) -> Result<T, E>
    where
        E: From<DomHostedRuntimeError>,
    {
        let owner = self.state().map_err(E::from)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| E::from(DomHostedRuntimeError::Busy))?;
        if state.dom != lease.snapshot().owner() {
            return Err(DomHostedRuntimeError::Binding.into());
        }
        let (h, t, c, g) = state
            .find_global(request.global(), cancel, || source_work(lease, cancel))
            .map_err(E::from)?;
        let group = state.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)
            .map_err(E::from)?;
        let global = &group.clusters[c].agent.globals[g];
        if state.hosts[h].id != request.host()
            || group.traversal.traversable != Some(request.traversable())
            || global.settings.document != Some(request.document())
            || !global.settings.ready
        {
            return Err(DomHostedRuntimeError::Binding.into());
        }
        state.step(cancel).map_err(E::from)?;
        source_work(lease, cancel).map_err(E::from)?;
        let index = state
            .reserve_fullscreen_row(h, t, lease, request, cancel)
            .map_err(E::from)?;
        let HostedState { source, hosts, .. } = &mut *state;
        let jobs = &mut hosts[h].top_levels[t]
            .group
            .as_mut()
            .expect("qualified original group")
            .fullscreen;
        let reservation = ReservedRow {
            jobs,
            index,
            installed: false,
        };
        let (permission, output) = commit(lease, source)?;
        // All three owners now join without allocation, callbacks or cancellation.
        reservation.install(permission);
        Ok(output)
    }
}

#[cfg(test)]
mod tests;

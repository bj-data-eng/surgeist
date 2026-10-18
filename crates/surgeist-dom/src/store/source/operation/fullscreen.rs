//! Prepared Runtime scheduling join of the original Fullscreen method/hook.
use super::*;
use crate::store::fullscreen::exit::{Frame as ExitFrame, Invocation as ExitInvocation};
use crate::{DomFullscreenExitId, DomFullscreenExitStatus};
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(in crate::store) fn native_fixture(
    limits: DomLimits,
) -> (Dom, crate::DomHostedSource, DocumentId, NodeId) {
    tests::fixture_with_limits(limits)
}

/// One immutable requirement. Copies confer neither queue nor execution authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomSourceFullscreenExitHandoffRequest {
    operation: DomSourceOperationId,
    invocation: DomFullscreenExitId,
    document: DocumentId,
    occurrence: crate::source::watcher::HookOccurrence,
    traversable: crate::DomHostedNavigableId,
    global: crate::DomHostedGlobalId,
    host: crate::DomHostedHostId,
}
impl DomSourceFullscreenExitHandoffRequest {
    pub const fn operation(&self) -> DomSourceOperationId {
        self.operation
    }
    pub const fn invocation(&self) -> DomFullscreenExitId {
        self.invocation
    }
    pub const fn document(&self) -> DocumentId {
        self.document
    }
    pub const fn command(&self) -> usize {
        self.occurrence.command
    }
    pub const fn current_occurrence(&self) -> u64 {
        self.occurrence.ordinal
    }
    pub const fn traversable(&self) -> crate::DomHostedNavigableId {
        self.traversable
    }
    pub const fn global(&self) -> crate::DomHostedGlobalId {
        self.global
    }
    pub const fn host(&self) -> crate::DomHostedHostId {
        self.host
    }
}
/// Affine scheduling permission for the same canonical invocation. Drop parks it.
#[derive(Debug)]
pub struct DomFullscreenExitJob {
    id: DomFullscreenExitId,
    traversable: crate::DomHostedNavigableId,
    global: crate::DomHostedGlobalId,
    host: crate::DomHostedHostId,
}
impl DomFullscreenExitJob {
    pub const fn id(&self) -> DomFullscreenExitId {
        self.id
    }
    pub const fn traversable(&self) -> crate::DomHostedNavigableId {
        self.traversable
    }
    pub const fn global(&self) -> crate::DomHostedGlobalId {
        self.global
    }
    pub const fn host(&self) -> crate::DomHostedHostId {
        self.host
    }
}
/// No-callback fence loan. All tree, continuation, result and overlap admission
/// has finished; the Runtime owner must already hold its real reserved row.
pub struct DomFullscreenExitHandoffCommit<'a, 'dom> {
    lease: &'a mut DomSourceLease<'dom>,
    segment: Option<SegmentPublication>,
    bank: Option<crate::store::fullscreen::custody::Invocations>,
    job: Option<DomFullscreenExitJob>,
    control: Option<source::PreparedFullscreenControl>,
    custody_units: usize,
    checkpoint: Option<checkpoint::Checkpoint>,
}
impl DomFullscreenExitHandoffCommit<'_, '_> {
    pub fn publish(mut self) -> DomFullscreenExitJob {
        let job = self.job.take().expect("prepared invocation job");
        let bank = self.bank.take().expect("prepared invocation bank");
        let control = self.control.take().expect("prepared continuation control");
        let mut operation = self
            .lease
            .dom
            .source_operation
            .take()
            .expect("exclusive original Source");
        let published = self
            .segment
            .take()
            .expect("prepared original publication")
            .publish(&mut operation, self.lease.dom);
        // No allocation, callback or cancellation can interrupt this fence.
        self.lease.dom.fullscreen_exits = bank;
        operation
            .draft
            .commit_fullscreen_handoff(control, &published.committed);
        drop(published);
        // Transfer the named mandatory credit, leaving the outer guard intact.
        operation
            .headroom
            .as_mut()
            .expect("admitted original command")
            .remaining_phases -= 1;
        operation.receipt = None;
        operation.owner_binding = None;
        operation.custody_units = self.custody_units;
        self.lease.dom.source_operation = Some(operation);
        self.checkpoint.take();
        job
    }
}
impl Drop for DomFullscreenExitHandoffCommit<'_, '_> {
    fn drop(&mut self) {
        if let Some(checkpoint) = self.checkpoint.take() {
            checkpoint.restore(
                &mut self
                    .lease
                    .dom
                    .source_operation
                    .as_mut()
                    .expect("exclusive original Source")
                    .draft,
            );
        }
    }
}
impl Operation {
    pub(super) fn fullscreen_handoff_request(
        &mut self,
        dom: &mut Dom,
        cancel: &AtomicBool,
    ) -> Result<DomSourceFullscreenExitHandoffRequest, DomError> {
        self.draft.source_step(cancel)?;
        let frame = self
            .draft
            .fullscreen_exit
            .ok_or(DomError::InvalidSourceOperation)?;
        let id = frame.id.unwrap_or(DomFullscreenExitId {
            owner: dom.owner,
            slot: dom.fullscreen_exits.len(),
            generation: dom.next_fullscreen_exit,
        });
        admit(
            id.slot
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
            dom.limits.fullscreen_exits,
            DomResource::FullscreenExits,
        )?;
        let ordinal = frame.ordinal.unwrap_or(self.draft.source_invocation);
        let next = ordinal
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let document = DocumentId(
            self.draft
                .published_id(frame.document)
                .ok_or(DomError::UnavailableCreation)?,
        );
        let next_identity = dom
            .next_fullscreen_exit
            .checked_add(u64::from(frame.id.is_none()))
            .ok_or(DomError::RepresentationExhausted)?;
        let identities = self
            .identities
            .checked_add(u64::from(frame.id.is_none()))
            .ok_or(DomError::RepresentationExhausted)?;
        if identities > self.limits.identities {
            return Err(DomError::SourceIdentityLimit);
        }
        self.draft.fullscreen_exit = Some(ExitFrame {
            id: Some(id),
            ordinal: Some(ordinal),
            ..frame
        });
        // Exposed identities never roll back into a later invocation.
        dom.next_fullscreen_exit = next_identity;
        self.identities = identities;
        if frame.ordinal.is_none() {
            self.draft.source_invocation = next;
        }
        Ok(DomSourceFullscreenExitHandoffRequest {
            operation: self.id,
            invocation: id,
            document,
            occurrence: crate::source::watcher::HookOccurrence {
                command: self.index,
                ordinal,
            },
            traversable: frame.traversable,
            global: frame.global,
            host: frame.host,
        })
    }
}
impl<'dom> DomSourceLease<'dom> {
    pub fn prepare_fullscreen_exit_handoff<'a>(
        &'a mut self,
        request: &DomSourceFullscreenExitHandoffRequest,
        source: &crate::DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<DomFullscreenExitHandoffCommit<'a, 'dom>, DomError> {
        self.dom
            .validate_hosted_source(source)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        if request.operation != self.id
            || !self
                .dom
                .source_operation
                .as_ref()
                .is_some_and(|op| op.id == self.id)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let mut op = self
            .dom
            .source_operation
            .take()
            .expect("qualified original Source");
        let qualified = op.fullscreen_handoff_request(self.dom, cancel);
        if qualified.as_ref() != Ok(request)
            || request.invocation.slot != self.dom.fullscreen_exits.len()
            || request.host.source() != source.id()
        {
            self.dom.source_operation = Some(op);
            return Err(qualified.err().unwrap_or(DomError::InvalidSourceOperation));
        }
        let checkpoint = match op
            .input_bytes()
            .and_then(|bytes| op.draft.source_checkpoint(bytes, 0, cancel))
        {
            Ok(checkpoint) => checkpoint,
            Err(error) => {
                self.dom.source_operation = Some(op);
                return Err(error);
            }
        };
        let result = (|| {
            op.refresh_admission(cancel)?;
            let control = op
                .draft
                .prepare_fullscreen_handoff_control(request.invocation, cancel)?;
            let formation = formation::Formation::new(&op.draft, cancel)?;
            admit(
                self.dom
                    .fullscreen_exits
                    .len()
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?,
                self.dom.limits.fullscreen_exits,
                DomResource::FullscreenExits,
            )?;
            if let Some(formation) = &formation {
                formation.retain_bytes(
                    size_of::<DomFullscreenExitHandoffCommit<'_, '_>>()
                        .checked_sub(size_of::<checkpoint::Checkpoint>())
                        .ok_or(DomError::RepresentationExhausted)?,
                    cancel,
                )?;
            }
            let frame = op.draft.fullscreen_exit.expect("qualified original prefix");
            let invocation = ExitInvocation {
                id: request.invocation,
                document: request.document,
                resize: frame.resize,
                operation: op.id,
                occurrence: request.occurrence,
                traversable: frame.traversable,
                global: frame.global,
                host: frame.host,
                work: op.draft.source_work.clone(),
                ancestors: op.draft.source_ancestors.clone(),
                work_custody: op.draft.source_work_custody.clone(),
                retained_ancestors: op.draft.source_ancestor_custody.clone(),
                limits: op.draft.source_limits.expect("original limits"),
                closing_credit: 1,
            };
            let job = DomFullscreenExitJob {
                id: invocation.id,
                traversable: invocation.traversable,
                global: invocation.global,
                host: invocation.host,
            };
            let bank = self.dom.fullscreen_exits.prepare_append(
                invocation,
                |usage| op.draft.admit_custody_usage(usage, cancel),
                || op.draft.source_step(cancel),
            )?;
            let custody_bytes = self.dom.source_owner_custody_bytes()?;
            let custody_units = self.dom.fullscreen_custody_units()?;
            op.draft.fullscreen_exit_prefix(self.dom.limits, cancel)?;
            let remaining = op
                .headroom
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?
                .remaining_phases;
            if remaining < 2 {
                return Err(DomError::SourcePublicationLimit);
            }
            let mut segment = op
                .prepare_segment(self.dom, true, PublicationTarget::None, None, cancel)
                .map_err(|error| match error {
                    crate::DomHostedError::Dom(error) => error,
                    _ => DomError::HostedSourceUnavailable,
                })?;
            if let (Some(destination), Some(formation)) =
                (segment.prepared._formation.as_ref(), formation)
            {
                destination.absorb(formation);
            }
            // The prepared roots own their actual union debit in the shared
            // ledger. The remaining owner envelope contains only headers.
            segment.owner_custody = custody_bytes;
            Ok((segment, bank, job, custody_units, control))
        })();
        self.dom.source_operation = Some(op);
        match result {
            Ok((segment, bank, job, custody_units, control)) => {
                Ok(DomFullscreenExitHandoffCommit {
                    lease: self,
                    segment: Some(segment),
                    bank: Some(bank),
                    job: Some(job),
                    control: Some(control),
                    custody_units,
                    checkpoint: Some(checkpoint),
                })
            }
            Err(error) => {
                checkpoint.restore(
                    &mut self
                        .dom
                        .source_operation
                        .as_mut()
                        .expect("restored original")
                        .draft,
                );
                Err(error)
            }
        }
    }
}
impl Dom {
    pub fn resume_fullscreen_exit(
        &mut self,
        source: &crate::DomHostedSource,
        job: &mut DomFullscreenExitJob,
        cancel: &AtomicBool,
    ) -> Result<DomFullscreenExitStatus, DomError> {
        self.require_sync_source_idle()?;
        if job.id.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        self.validate_hosted_source(source)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        let invocation = self
            .fullscreen_exits
            .get(job.id.slot)
            .filter(|invocation| invocation.id == job.id)
            .ok_or(DomError::StaleFullscreenExit)?;
        invocation.step(cancel)?;
        if invocation.traversable != job.traversable
            || invocation.global != job.global
            || invocation.host != job.host
            || job.host.source() != source.id()
        {
            return Err(DomError::InvalidSourceOperation);
        }
        match self.state.node(self.owner, invocation.document.node()) {
            Err(DomError::StaleNode) => {
                return Err(DomError::FullscreenExitDocumentRetired {
                    invocation: job.id,
                    document: invocation.document,
                    step: crate::DomFullscreenExitStep::FullyUnlockScreenOrientation,
                });
            }
            Err(error) => return Err(error),
            Ok(_) => {}
        }
        // No real orientation owner exists. Hosted retirement/inactivity is not
        // canonical identity retirement and cannot complete or cancel this call.
        Ok(invocation.status())
    }
}

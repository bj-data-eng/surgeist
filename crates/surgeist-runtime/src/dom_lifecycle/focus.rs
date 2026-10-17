//! Actual complete-Host absence producer; no external focus adapter is implied.
use super::*;
use surgeist_dom::{
    DomHostedRuntimeRetention, DomRememberedFocusPreparation, DomRememberedFocusPublication,
    DomSourceDialogRequest, DomSourceLease,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HostFocusDisposition {
    ExternalUnobserved,
    HeadlessNoContentSystemFocus,
}
/// Weak permission naming one actual Host in the original durable Session.
/// It owns no Window, DOM, Host lifetime, focus observation or Source cursor.
#[derive(Clone)]
pub struct DomSourceExecutionScope {
    state: Weak<RefCell<HostedState>>,
    host: DomHostedHostId,
}
impl fmt::Debug for DomSourceExecutionScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomSourceExecutionScope")
            .field("host", &self.host)
            .finish_non_exhaustive()
    }
}
pub(super) struct FocusState {
    pub(super) pending: Option<DomRememberedFocusPublication>,
    pub(super) host_backing: Option<DomHostedRuntimeRetention>,
    _header: DomHostedRuntimeRetention,
}
impl FocusState {
    pub(super) const fn header_bytes() -> usize {
        size_of::<Self>() + size_of::<Vec<records::Host>>()
    }
    pub(super) fn new(header: DomHostedRuntimeRetention) -> Self {
        Self {
            pending: None,
            host_backing: None,
            _header: header,
        }
    }
}
// Actual temporary producer header; its result moves into the pre-admitted
// fixed slot before this separate relative formation extent is released.
struct RememberedFormation {
    preparation: DomRememberedFocusPreparation,
    payload: DomHostedRuntimeRetention,
}
impl HostedRuntime {
    pub(crate) fn execution_scope(
        &self,
        dom: &Dom,
        host: DomHostedHostId,
        cancel: &AtomicBool,
    ) -> Result<DomSourceExecutionScope, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        if state.dom != dom.owner() {
            return Err(DomHostedRuntimeError::Binding);
        }
        for index in 0..state.hosts.len() {
            state.step(cancel)?;
            state
                .source
                .admit_runtime_work(dom, 1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            if state.hosts[index].id == host {
                return Ok(DomSourceExecutionScope {
                    state: Rc::downgrade(&owner),
                    host,
                });
            }
        }
        Err(DomHostedRuntimeError::TargetNotFound)
    }
    pub(crate) fn qualify_execution_scope(
        &self,
        scope: &DomSourceExecutionScope,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let scoped = scope
            .state
            .upgrade()
            .ok_or(DomHostedRuntimeError::OwnerDisposed)?;
        if !Rc::ptr_eq(&owner, &scoped) {
            return Err(DomHostedRuntimeError::Binding);
        }
        Ok(())
    }
    pub(crate) fn bind_execution_scope(
        &self,
        lease: &mut DomSourceLease<'_>,
        scope: &DomSourceExecutionScope,
        fresh: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        self.qualify_execution_scope(scope)?;
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        if state.dom != lease.snapshot().owner() {
            return Err(DomHostedRuntimeError::Binding);
        }
        for index in 0..state.hosts.len() {
            state.step(cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            if state.hosts[index].id == scope.host {
                return if fresh {
                    lease.bind_execution_host(&state.source, scope.host, cancel)
                } else {
                    lease.validate_execution_host(&state.source, scope.host, cancel)
                }
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()));
            }
        }
        Err(DomHostedRuntimeError::TargetNotFound)
    }
    pub(crate) fn service_remembered_focus(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceDialogRequest,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        if state.dom != lease.snapshot().owner() {
            return Err(DomHostedRuntimeError::Binding);
        }
        if let Some(publication) = &state.focus.pending {
            if publication.request() != request {
                return Err(DomHostedRuntimeError::OperationPending);
            }
        } else {
            // Read only the retained original scope and immutable Host mode.
            // Missing/unobserved scope needs no owned observation formation.
            let host = match lease
                .execution_entry_scope()
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?
            {
                surgeist_dom::DomSourceExecutionEntryScope::ExplicitHost(host)
                | surgeist_dom::DomSourceExecutionEntryScope::Hosted {
                    original_host: Some(host),
                    ..
                } => host,
                _ => return Ok(false),
            };
            let mut disposition = None;
            for index in 0..state.hosts.len() {
                state.step(cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
                if state.hosts[index].id == host {
                    disposition = Some(state.hosts[index].focus);
                    break;
                }
            }
            let disposition = disposition.ok_or(DomHostedRuntimeError::TargetNotFound)?;
            if disposition != HostFocusDisposition::HeadlessNoContentSystemFocus {
                return Ok(false);
            }
            let retained = state.source_retained_bytes(lease, cancel)?;
            let temporary = size_of::<RememberedFormation>();
            if retained
                .checked_add(temporary)
                .is_none_or(|bytes| bytes > state.limits.storage_bytes)
                || temporary > state.limits.scratch_bytes
            {
                return Err(DomHostedRuntimeError::Admission);
            }
            let payload = lease
                .retain_hosted_runtime_payload(&state.source, temporary, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            // Admit the entire owned temporary before DOM clones the exact
            // request into its affine preparation. Public DOM qualification
            // keeps its original validation order; this private service is
            // reached only for the drive loop's actual remembered occurrence.
            let Some(prepared) = lease
                .prepare_remembered_focus(request, &state.source, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?
            else {
                return Ok(false);
            };
            let formation = RememberedFormation {
                preparation: prepared,
                payload,
            };
            state.step(cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            // No callback, attachment or conversion can invalidate this Host's
            // immutable complete-lifetime restriction. The fixed actual result
            // slot is admitted from construction, before the observation.
            let RememberedFormation {
                preparation,
                payload,
            } = formation;
            state.focus.pending = Some(preparation.publish_known_none());
            drop(payload);
        }
        let HostedState { source, focus, .. } = &mut *state;
        lease
            .accept_remembered_focus(request, source, &mut focus.pending, cancel)
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests;

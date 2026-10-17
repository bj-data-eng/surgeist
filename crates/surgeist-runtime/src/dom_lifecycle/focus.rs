//! Qualified negative focus observations from actual Host/Window lifetime state.
use super::*;
use surgeist_dom::{
    DomDialogFocusObservation, DomDialogFocusPreparation, DomDialogFocusPublication,
    DomDialogFocusTarget, DomHostedRuntimeRetention, DomSourceDialogRequest, DomSourceLease,
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
    pub(super) pending: Option<DomDialogFocusPublication>,
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
struct FocusFormation {
    preparation: DomDialogFocusPreparation,
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
    pub(crate) fn service_dialog_focus(
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
            let remembering =
                request.action() == surgeist_html::HtmlDialogAction::RememberFocusedElement;
            if remembering {
                // Missing/unobserved original scope needs no owned formation.
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
                if disposition.ok_or(DomHostedRuntimeError::TargetNotFound)?
                    != HostFocusDisposition::HeadlessNoContentSystemFocus
                {
                    return Ok(false);
                }
            }
            let retained = state.source_retained_bytes(lease, cancel)?;
            let temporary = size_of::<FocusFormation>();
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
            // reached only for the drive loop's actual focus occurrence.
            let Some(prepared) = lease
                .prepare_dialog_focus(request, &state.source, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?
            else {
                return Ok(false);
            };
            let formation = FocusFormation {
                preparation: prepared,
                payload,
            };
            let observation = match formation.preparation.target() {
                DomDialogFocusTarget::RememberedHost(_) => DomDialogFocusObservation::KnownNone,
                DomDialogFocusTarget::DeniedDocumentWindow {
                    document,
                    global,
                    host,
                } => {
                    // This qualification belongs to the current live Document,
                    // independently of the queue Global or remembered Host.
                    let (gh, gt, gc, g) = match state.find_global(global, cancel, || {
                        lease
                            .admit_hosted_runtime_work(1, cancel)
                            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
                    }) {
                        Ok(location) => location,
                        Err(DomHostedRuntimeError::TargetNotFound) => return Ok(false),
                        Err(error) => return Err(error),
                    };
                    let live = &state.hosts[gh].top_levels[gt]
                        .group
                        .as_ref()
                        .expect("actual live group")
                        .clusters[gc]
                        .agent
                        .globals[g];
                    if state.hosts[gh].id != host
                        || live.settings.document != Some(document)
                        || live.id != global
                    {
                        return Err(DomHostedRuntimeError::Binding);
                    }
                    let (h, t, c, w) = match state.find_window(global, cancel, || {
                        lease
                            .admit_hosted_runtime_work(1, cancel)
                            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
                    }) {
                        Ok(location) => location,
                        Err(DomHostedRuntimeError::Binding) => return Ok(false),
                        Err(error) => return Err(error),
                    };
                    if (h, t, c) != (gh, gt, gc) {
                        return Err(DomHostedRuntimeError::Binding);
                    }
                    let window = &state.hosts[h].top_levels[t]
                        .group
                        .as_ref()
                        .expect("actual live group")
                        .clusters[c]
                        .agent
                        .windows
                        .rows[w];
                    if window.id != global || window.retired {
                        return Ok(false);
                    }
                    if surgeist_html::allow_focus_decision(
                        Some(false),
                        Some(window.transient_activation()),
                    ) != surgeist_html::HtmlAllowFocusDecision::Denied
                    {
                        return Ok(false);
                    }
                    DomDialogFocusObservation::Denied
                }
                DomDialogFocusTarget::RetiredDocumentWindow { global, host, .. } => {
                    let location = match state.find_window(global, cancel, || {
                        lease
                            .admit_hosted_runtime_work(1, cancel)
                            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
                    }) {
                        Ok(location) => location,
                        Err(DomHostedRuntimeError::Binding) => return Ok(false),
                        Err(error) => return Err(error),
                    };
                    let (h, t, c, w) = location;
                    let window = &state.hosts[h].top_levels[t]
                        .group
                        .as_ref()
                        .expect("actual retained group")
                        .clusters[c]
                        .agent
                        .windows
                        .rows[w];
                    if state.hosts[h].id != host
                        || window.id != global
                        || !window.retired
                        || surgeist_html::allow_focus_decision(
                            Some(false),
                            Some(window.transient_activation()),
                        ) != surgeist_html::HtmlAllowFocusDecision::Denied
                    {
                        return Ok(false);
                    }
                    DomDialogFocusObservation::Denied
                }
            };
            state.step(cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            // No callback or reentry occurs during the qualified observation.
            // The original Source tail and watcher own retained Window custody;
            // the immutable complete-Host restriction owns absence. The result
            // slot is admitted from construction, before the observation.
            let FocusFormation {
                preparation,
                payload,
            } = formation;
            state.focus.pending = Some(
                preparation
                    .publish(observation)
                    .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?,
            );
            drop(payload);
        }
        let HostedState { source, focus, .. } = &mut *state;
        lease
            .accept_dialog_focus(request, source, &mut focus.pending, cancel)
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests;

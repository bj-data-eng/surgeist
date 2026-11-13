//! Deferred delivery for the attached Window's native CSP violation queue.

use super::*;
use crate::dom_lifecycle::{CspTaskCustody, EventCreation};

/// Progress of one attached Window's deferred CSP violation task.
#[derive(Debug)]
pub enum DocumentCspViolationProgress {
    /// The original queue has no pending report.
    Empty,
    /// The original event finished dispatch and its native task settled.
    Complete(DomEventDispatchReceipt),
    /// Native dispatch stopped at its original cursor.
    Retained(DomEventDispatchOutcome),
}

pub(super) struct ActiveViolationTask {
    number: u64,
    custody: Option<CspTaskCustody>,
    creation: Option<Result<EventCreation, DomEventTimestampUnavailable>>,
    dispatch: Option<DomEventDispatch>,
    receipt: Option<DomEventDispatchReceipt>,
}

impl ActiveViolationTask {
    pub(super) fn new(number: u64) -> Self {
        Self {
            number,
            custody: None,
            creation: None,
            dispatch: None,
            receipt: None,
        }
    }
}

impl DocumentScriptHost {
    /// Service one queued CSP violation for the attached Window. An interrupted
    /// native dispatch resumes the same Event and listener position.
    /// This entry runs no guest microtask checkpoint.
    pub fn run_attached_csp_violation_task(
        &mut self,
        permit: &DocumentScriptPermit,
        limits: DomEventDispatchLimits,
    ) -> Result<DocumentCspViolationProgress, DocumentScriptEventError> {
        self.check_permit(permit)?;
        self.attachment()?.realm.check_owner_active()?;
        let events = self
            .guest_events()
            .map_err(DocumentScriptEventError::Guest)?;
        let owner = self.native.clone();
        let cancel = &owner.cancellation;
        let running = self.with_native(|dom, hosted, _| {
            let mut queue = owner
                .trusted_types_reports
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let Some(queue) = queue.as_mut().filter(|queue| queue.head().is_some()) else {
                return Ok::<_, DocumentScriptEventError>(None);
            };
            let origin = queue.origin;
            let active = queue
                .start_head()
                .map_err(DocumentScriptEventError::Guest)?;
            let running = hosted
                .enter_csp_violation_task(
                    dom,
                    origin.global,
                    origin.document,
                    active.number,
                    &mut active.custody,
                    cancel,
                )
                .map_err(DocumentScriptNativeError::from)?;
            Ok(Some(running))
        })??;
        let Some(mut running) = running else {
            return Ok(DocumentCspViolationProgress::Empty);
        };

        let mut usage = crate::dom_event::DispatchUsage::default();
        loop {
            let progress = self
                .with_native(|dom, _, _| {
                    let mut queue = owner
                        .trusted_types_reports
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    let queue = queue
                        .as_mut()
                        .ok_or(DocumentGuestEventError::RegistryState)?;
                    let report = queue.head().ok_or(DocumentGuestEventError::RegistryState)?;
                    let document = queue.origin.document;
                    let active = queue.start_head()?;
                    if active.receipt.is_none() {
                        let mut registry = events
                            .registry
                            .try_borrow_mut()
                            .map_err(|_| DocumentScriptNativeError::Busy)?;
                        if active.dispatch.is_none() {
                            if active.creation.is_none() {
                                active.creation = Some(
                                    running
                                        .sample_event_creation(dom, cancel)
                                        .map_err(DocumentScriptNativeError::from)?,
                                );
                            }
                            active.dispatch = Some(registry.prepare_csp_violation(
                                dom,
                                document,
                                report,
                                &mut active.creation,
                                cancel,
                            )?);
                        }
                        let dispatch = active
                            .dispatch
                            .as_mut()
                            .ok_or(DocumentGuestEventError::RegistryState)?;
                        match dispatch.advance_with_host_usage(
                            dom,
                            &mut registry,
                            &mut (),
                            (limits, &mut usage),
                            cancel,
                            DispatchHost::AttachedNative,
                        )? {
                            DomEventDispatchOutcome::Complete(receipt) => {
                                active.receipt = Some(receipt)
                            }
                            progress => return Ok::<_, DocumentGuestEventError>(progress),
                        }
                    }
                    running
                        .checkpoint(dom, cancel)
                        .map_err(DocumentScriptNativeError::from)?;
                    let receipt = active
                        .receipt
                        .take()
                        .ok_or(DocumentGuestEventError::RegistryState)?;
                    running.complete();
                    queue.finish_head();
                    Ok(DomEventDispatchOutcome::Complete(receipt))
                })?
                .map_err(DocumentScriptEventError::Guest)?;
            match progress {
                DomEventDispatchOutcome::Complete(receipt) => {
                    return Ok(DocumentCspViolationProgress::Complete(receipt));
                }
                DomEventDispatchOutcome::HostInvocation(mut selected) => {
                    self.invoke_attached_listener(&events, None, &mut selected)?;
                }
                retained => return Ok(DocumentCspViolationProgress::Retained(retained)),
            }
        }
    }
}

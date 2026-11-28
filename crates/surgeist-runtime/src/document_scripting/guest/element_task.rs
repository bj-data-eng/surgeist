//! Guest delivery through the original native element task bank.

use super::*;
use crate::internal::dom_source::{
    DomSourceDriveProgress, DomSourceServices, DomSourceUnavailable, DomSourceUrlService,
};
use crate::internal::dom_style::DomStyleUnavailable;
use crate::internal::element_tasks::{
    AttachedTaskRequest, DomElementTaskRuntimeError, ElementTaskRunOutcome,
};
use crate::{
    internal::element_tasks::DomElementTaskProgress,
    internal::element_tasks::DomElementTaskRunLimits,
};
use surgeist_dom::{
    source::{DomSourceOperationId, DomSourceRequest},
    style::DomStyleInlineCspObservation,
    task::DomElementTaskSourceKind,
    url::{DomSourceUrlObservation, DomSourceUrlRequest},
};

impl DocumentScriptHost {
    /// Execute or resume the attached Window's original task at the selected
    /// source's head. The native task bank owns event delivery and completion.
    ///
    /// This entry runs no guest microtask checkpoint.
    pub fn run_attached_element_task(
        &mut self,
        permit: &DocumentScriptPermit,
        source: DomElementTaskSourceKind,
        limits: DomElementTaskRunLimits,
    ) -> Result<DomElementTaskProgress, DocumentScriptEventError> {
        self.run_attached_element_task_with_source_services(
            permit,
            source,
            limits,
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
        )
    }

    /// Execute or resume the attached Window's FIFO head with actual Source
    /// services for a reached selectedcontent request. Service callbacks and
    /// guest listeners run after the previous Session and registry loans end.
    pub fn run_attached_element_task_with_source_services(
        &mut self,
        permit: &DocumentScriptPermit,
        source: DomElementTaskSourceKind,
        limits: DomElementTaskRunLimits,
        services: &mut DomSourceServices<'_>,
    ) -> Result<DomElementTaskProgress, DocumentScriptEventError> {
        self.check_permit(permit)?;
        let attachment = self.attachment()?;
        attachment.realm.check_owner_active()?;
        let target = attachment.target;
        let events = self
            .guest_events()
            .map_err(DocumentScriptEventError::Guest)?;
        let owner = self.native.clone();
        let tasks = owner
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .element_task_execution();
        let mut expected = None;
        let mut select_backup_ready = None;
        let mut usage = crate::internal::dom_event::DispatchUsage::default();
        let mut style_observation: Option<(
            (DomSourceOperationId, u64),
            DomStyleInlineCspObservation,
        )> = None;
        let mut url_observation: Option<(DomSourceUrlRequest, DomSourceUrlObservation)> = None;
        loop {
            let mut issued_url = None;
            let result = self.with_native(|dom, _, _| {
                let mut registry = events
                    .registry
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let mut transfer_url = |request: &DomSourceUrlRequest, _: &AtomicBool| {
                    if url_observation.as_ref().is_some_and(|(issued, _)| {
                        issued.operation() == request.operation()
                            && issued.occurrence() == request.occurrence()
                            && issued.stage() == request.stage()
                    }) {
                        return Ok(url_observation.take().map(|(_, observation)| observation));
                    }
                    issued_url = Some(request.clone());
                    Ok(None)
                };
                tasks
                    .run_attached(
                        dom,
                        &mut registry,
                        (source, limits),
                        AttachedTaskRequest {
                            target,
                            expected,
                            select_backup_ready,
                            usage: &mut usage,
                        },
                        &mut DomSourceServices {
                            meta: services.meta,
                            execution_scope: services.execution_scope,
                            style_policy: &mut |request| {
                                if style_observation.as_ref().is_some_and(|(issued, _)| {
                                    *issued == (request.operation(), request.ordinal())
                                }) {
                                    return style_observation
                                        .take()
                                        .map(|(_, observation)| observation);
                                }
                                None
                            },
                            url: services.url.as_ref().map(|service| DomSourceUrlService {
                                revision: service.revision,
                                observe: &mut transfer_url,
                            }),
                        },
                        &owner.cancellation,
                    )
                    .map_err(DocumentScriptEventError::Task)
            })?;
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    // A refused Source advance can have published a prefix.
                    // Keep its original refusal even if the backup job must
                    // retry scheduling as well.
                    if matches!(
                        error,
                        DocumentScriptEventError::Task(
                            DomElementTaskRuntimeError::SelectNotificationSource(_)
                        )
                    ) {
                        let _ = self.schedule_select_backup_reactions(&events);
                    }
                    return Err(error);
                }
            };
            match result {
                ElementTaskRunOutcome::SelectNotificationBackupReactions(id) => {
                    self.schedule_select_backup_reactions(&events)?;
                    expected = Some(id);
                    select_backup_ready = Some(id);
                }
                ElementTaskRunOutcome::Native(progress) => {
                    let DomElementTaskProgress::SelectNotificationSourcePending {
                        task,
                        source,
                        ..
                    } = &progress
                    else {
                        return Ok(progress);
                    };
                    self.schedule_select_backup_reactions(&events)?;
                    match source.as_ref() {
                        DomSourceDriveProgress::Pending {
                            request: DomSourceRequest::Style(request),
                            reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
                        } => {
                            let Some(observation) = (services.style_policy)(request.source())
                            else {
                                return Ok(progress);
                            };
                            style_observation = Some((
                                (request.source().operation(), request.source().ordinal()),
                                observation,
                            ));
                        }
                        DomSourceDriveProgress::Pending {
                            request: DomSourceRequest::HostedChild(_),
                            reason: DomSourceUnavailable::Url(_),
                        } => {
                            let (Some(issued), Some(service)) = (issued_url, services.url.as_mut())
                            else {
                                return Ok(progress);
                            };
                            let Some(observation) = (service.observe)(&issued, &owner.cancellation)
                                .map_err(DocumentScriptNativeError::Hosted)?
                            else {
                                return Ok(progress);
                            };
                            url_observation = Some((issued, observation));
                        }
                        _ => return Ok(progress),
                    }
                    expected = Some(task.id());
                }
                ElementTaskRunOutcome::Guest(mut selected) => {
                    self.invoke_attached_listener(
                        &events,
                        selected.request.as_ref(),
                        &mut selected.invocation,
                    )?;
                    expected = Some(selected.id);
                }
            }
        }
    }

    fn schedule_select_backup_reactions(
        &mut self,
        events: &GuestEvents,
    ) -> Result<(), DocumentScriptEventError> {
        let Some(public) = events.public.as_ref() else {
            return Ok(());
        };
        let attachment = self
            .native
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let registration = attachment
            .registration
            .upgrade()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        if !public
            .has_unscheduled_backup_reactions(events, &registration)
            .map_err(DocumentScriptEventError::Guest)?
        {
            return Ok(());
        }
        let engine = self
            .engine
            .as_mut()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let result = engine.process.execution().in_realm(&engine.realm, |scope| {
            public.schedule_backup_reactions(scope)
        });
        engine.settle_engine_cleanup(&self.native, &attachment.realm);
        Ok(result?)
    }
}

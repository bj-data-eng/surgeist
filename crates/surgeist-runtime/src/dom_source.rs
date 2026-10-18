//! One requested Source operation through the original durable session owners.
//!
//! The canonical owner retains the command, hook cursor and acknowledged tail.
//! This weak permission routes actual yielded requests; it owns neither tree nor
//! resources and cannot accept an arbitrary sequence of commands.
use crate::dom_dialog::{
    DomSourceEventRuntimeError, DomSourceEventServiceLimits, DomSourceEventServiceProgress,
    SourceEventRuntime,
};
use crate::dom_event::DomEventDispatcher;
use crate::dom_event::DomEventDom;
use crate::dom_lifecycle::{DomHostedRuntimeError, HostedRuntime, HostedSourceProgress};
use crate::dom_style::{
    DomStyleRuntime, DomStyleRuntimeError, DomStyleServiceProgress, DomStyleUnavailable,
    StyleSourceGuard,
};
use crate::element_tasks::{DomElementTaskRuntime, DomElementTaskRuntimeError};
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DomError, DomNativeCheckpointLimits, DomNativeHtmlInstallError, DomNativeHtmlInstallTarget,
    DomSelectCompletionStart, DomSelectTask, DomSelectTaskCompletion, DomSourceCompletion,
    DomSourceEdit, DomSourceLease, DomSourceLimits, DomSourceNativeHtmlInstallLimits,
    DomSourceOperationId, DomSourceProgress, DomSourceRequest, DomStyleInlineCspObservation,
    DomStyleSource,
};

/// Borrowed services for the reached requests of one actual Source operation.
/// These references own no resource, response slot, issuer or operation cursor.
pub struct DomSourceServices<'a> {
    /// Optional exact Session/Host permission, bound only at fresh Source entry.
    pub execution_scope: Option<&'a crate::DomSourceExecutionScope>,
    /// Actual CSP observation supplier; absence remains an unavailable policy.
    pub style_policy: &'a mut dyn FnMut(&DomStyleSource) -> Option<DomStyleInlineCspObservation>,
    /// Independently configured URL/origin observation service, if available.
    pub url: Option<DomSourceUrlService<'a>>,
}

/// A reached URL response: `Ok(None)` is unavailable; `Err` refuses the step.
pub type DomSourceUrlObservationResult =
    Result<Option<surgeist_dom::DomSourceUrlObservation>, DomHostedRuntimeError>;

/// A URL observer and its independently selected influencing configuration.
/// The driver releases all canonical and Runtime borrows before observation.
pub struct DomSourceUrlService<'a> {
    /// Expected parse configuration, selected before receiving the observation.
    pub revision: &'a surgeist_html_semantics::HtmlBaseFactsRevision,
    /// Observes the issued inputs; None is unavailable, never URL parse failure.
    pub observe: &'a mut dyn FnMut(
        &surgeist_dom::DomSourceUrlRequest,
        &AtomicBool,
    ) -> DomSourceUrlObservationResult,
}

/// A missing service at the actual next request, without acknowledging it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceUnavailable {
    /// An exact beforetoggle request requires explicit event dispatch.
    Event,
    /// This actual Dialog service has no implementation yet.
    Dialog(surgeist_html::HtmlDialogAction),
    /// The exact child focusing action still lacks a qualified service.
    DialogFocusing(surgeist_html::HtmlDialogFocusingAction),
    /// Exact unsupported fact in an original called focus-delegate traversal.
    FocusDelegate(surgeist_dom::DomSourceFocusDelegateStep),
    /// The original designMode invocation needs this exact qualified service.
    DesignMode(surgeist_dom::DomSourceDesignModeStep),
    /// The exact autofocus insertion action still lacks a qualified service.
    AutofocusInsertion(surgeist_html::HtmlAutofocusInsertionAction),
    /// The original Style owner needs this precise external input or resource.
    Style(DomStyleUnavailable),
    /// A reached URL stage has no qualified observation yet.
    Url(surgeist_dom::DomSourceUrlStage),
}

/// The requested operation returned, or its exact next request remains parked.
#[expect(
    clippy::large_enum_variant,
    reason = "Move the original Source completion inline without allocating after canonical publication"
)]
#[derive(Debug)]
pub enum DomSourceDriveProgress {
    /// The one command or named feature operation completed its mandatory hooks.
    Complete(DomSourceCompletion),
    /// Original canonical custody remains live for this unacknowledged request.
    Pending {
        /// Exact yielded request, not execution or publication authority.
        request: DomSourceRequest,
        /// Actual missing service.
        reason: DomSourceUnavailable,
    },
}

/// Canonical retirement or live Source progress for one authorized Select task.
#[expect(
    clippy::large_enum_variant,
    reason = "Return the original Source progress inline without a new fallible allocation after canonical publication"
)]
#[derive(Debug)]
pub enum DomSelectCompletionProgress {
    /// The canonical owner consumed the exact historical retirement outcome.
    Skipped(DomSelectTaskCompletion),
    /// The live task entered its original Source continuation.
    Source(DomSourceDriveProgress),
}

/// Refusal preserves the original canonical operation and family-owned results.
#[derive(Debug)]
pub enum DomSourceRuntimeError {
    /// The durable Source event adapter refused admission or execution.
    SourceEvent(DomSourceEventRuntimeError),
    /// Canonical Source validation or finite admission refused the operation.
    Dom(DomError),
    /// The original Style resource or processing owner refused its step.
    Style(DomStyleRuntimeError),
    /// The original bound Details owner is unavailable or foreign.
    ElementTasks(DomElementTaskRuntimeError),
    /// The original hosted owner is unavailable or foreign.
    Hosted(DomHostedRuntimeError),
    /// The named native installation refused its actual input or target.
    NativeInstallation(DomNativeHtmlInstallError),
}
impl std::fmt::Display for DomSourceRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source operation refused: {self:?}")
    }
}
impl std::error::Error for DomSourceRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SourceEvent(error) => Some(error),
            Self::Dom(error) => Some(error),
            Self::Style(error) => Some(error),
            Self::ElementTasks(error) => Some(error),
            Self::Hosted(error) => Some(error),
            Self::NativeInstallation(error) => Some(error),
        }
    }
}
impl From<DomError> for DomSourceRuntimeError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}
impl From<DomStyleRuntimeError> for DomSourceRuntimeError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Style(error)
    }
}

/// Weak permission to route one operation through the same session's owners.
/// Only the durable session constructs it. Replacing or dropping this value
/// neither extracts the canonical owner nor releases its resources or cursor.
pub struct DomSourceRuntime {
    styles: DomStyleRuntime,
    element_tasks: DomElementTaskRuntime,
    hosted: HostedRuntime,
    events: SourceEventRuntime,
}
impl std::fmt::Debug for DomSourceRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomSourceRuntime").finish_non_exhaustive()
    }
}
impl DomSourceRuntime {
    pub(crate) fn new(
        styles: DomStyleRuntime,
        element_tasks: DomElementTaskRuntime,
        hosted: HostedRuntime,
        events: SourceEventRuntime,
    ) -> Self {
        Self {
            styles,
            element_tasks,
            hosted,
            events,
        }
    }
    /// Dispatch exactly the issued event and accept its actual receipt. This
    /// does not advance the remaining Source tail; use ordinary `resume` next.
    pub fn service_event<C>(
        &self,
        dom: &mut DomEventDom<'_>,
        request: &surgeist_dom::DomSourceEventRequest,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        limits: DomSourceEventServiceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceEventServiceProgress, DomSourceRuntimeError> {
        self.events
            .service_event(dom, request, dispatcher, context, limits, cancel)
            .map_err(DomSourceRuntimeError::SourceEvent)
    }
    /// Monotonically raise the original event operation's policy limits, even
    /// while a retained child is topmost. This performs no dispatch, resume or
    /// Source-tail execution and preserves the original cumulative counters.
    pub fn raise_event_limits(
        &self,
        dom: &mut DomEventDom<'_>,
        request: &surgeist_dom::DomSourceEventRequest,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomSourceRuntimeError> {
        self.events
            .raise_event_limits(dom, request, limits, cancel)
            .map_err(DomSourceRuntimeError::SourceEvent)
    }
    /// Monotonically raise this Session's independent event-adapter limits.
    /// Retained jobs, actual receipts and cumulative work stay with the same
    /// owner. Validation consumes actual work under the proposed work cap;
    /// failed policy changes preserve that work and the original limits.
    pub fn raise_event_runtime_limits(
        &self,
        dom: &DomEventDom<'_>,
        limits: crate::dom_dialog::DomSourceEventRuntimeLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomSourceRuntimeError> {
        self.events
            .raise_event_runtime_limits(dom, limits, cancel)
            .map_err(DomSourceRuntimeError::SourceEvent)
    }
    fn prepare(
        &self,
        dom: &DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<StyleSourceGuard, DomSourceRuntimeError> {
        let guard = self.qualify_source_owners(dom)?;
        guard.reserve_cleanup(cancel)?;
        Ok(guard)
    }
    fn qualify_source_owners(
        &self,
        dom: &DomEventDom<'_>,
    ) -> Result<StyleSourceGuard, DomSourceRuntimeError> {
        self.element_tasks
            .validate_view(dom)
            .map_err(DomSourceRuntimeError::ElementTasks)?;
        self.hosted
            .validate_view(dom)
            .map_err(DomSourceRuntimeError::Hosted)?;
        Ok(self.styles.qualify_source(dom)?)
    }
    /// Execute exactly one requested edit through the original Source engine.
    pub fn apply(
        &mut self,
        dom: &mut DomEventDom<'_>,
        edit: DomSourceEdit,
        limits: DomSourceLimits,
        services: &mut DomSourceServices<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let guard = self.prepare(dom, cancel)?;
        if let Some(scope) = services.execution_scope {
            self.hosted
                .qualify_execution_scope(scope)
                .map_err(DomSourceRuntimeError::Hosted)?;
        }
        let progress = {
            let mut lease = dom.begin_source_edit(edit, limits, cancel)?;
            if let Some(scope) = services.execution_scope {
                self.hosted
                    .bind_execution_scope(&mut lease, scope, true, cancel)
                    .map_err(DomSourceRuntimeError::Hosted)?;
            }
            self.drive_lease(
                &guard,
                &mut lease,
                services.url.as_ref().map(|service| service.revision),
                cancel,
            )?
        };
        self.finish_or_supply(
            dom,
            DriveRoot {
                limits,
                select: None,
            },
            guard,
            progress,
            services,
            cancel,
        )
    }
    /// Resume the same operation without rebuilding acknowledged external work.
    pub fn resume(
        &mut self,
        dom: &mut DomEventDom<'_>,
        operation: DomSourceOperationId,
        limits: DomSourceLimits,
        services: &mut DomSourceServices<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let guard = self.prepare(dom, cancel)?;
        if let Some(scope) = services.execution_scope {
            self.hosted
                .qualify_execution_scope(scope)
                .map_err(DomSourceRuntimeError::Hosted)?;
        }
        let progress = {
            let mut lease = dom.resume_source_edit(operation, limits, cancel)?;
            if let Some(scope) = services.execution_scope {
                self.hosted
                    .bind_execution_scope(&mut lease, scope, false, cancel)
                    .map_err(DomSourceRuntimeError::Hosted)?;
            }
            self.drive_lease(
                &guard,
                &mut lease,
                services.url.as_ref().map(|service| service.revision),
                cancel,
            )?
        };
        self.finish_or_supply(
            dom,
            DriveRoot {
                limits,
                select: None,
            },
            guard,
            progress,
            services,
            cancel,
        )
    }
    /// Install one immutable native fragment using its original feature root.
    pub fn install_native_html(
        &mut self,
        dom: &mut DomEventDom<'_>,
        target: DomNativeHtmlInstallTarget,
        fragment: &surgeist_html::HtmlFragment,
        budgets: DomSourceNativeHtmlInstallLimits,
        services: &mut DomSourceServices<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let guard = self.prepare(dom, cancel)?;
        if let Some(scope) = services.execution_scope {
            self.hosted
                .qualify_execution_scope(scope)
                .map_err(DomSourceRuntimeError::Hosted)?;
        }
        let limits = budgets.source();
        let progress = {
            let mut lease = dom
                .raw_mut()
                .begin_native_installation(target, fragment, budgets, cancel)
                .map_err(DomSourceRuntimeError::NativeInstallation)?;
            if let Some(scope) = services.execution_scope {
                self.hosted
                    .bind_execution_scope(&mut lease, scope, true, cancel)
                    .map_err(DomSourceRuntimeError::Hosted)?;
            }
            self.drive_lease(
                &guard,
                &mut lease,
                services.url.as_ref().map(|service| service.revision),
                cancel,
            )?
        };
        self.finish_or_supply(
            dom,
            DriveRoot {
                limits,
                select: None,
            },
            guard,
            progress,
            services,
            cancel,
        )
    }
    /// Complete the actual once-authorized Select task through its feature root.
    pub fn complete_select(
        &mut self,
        dom: &mut DomEventDom<'_>,
        task: &DomSelectTask,
        checkpoint: DomNativeCheckpointLimits,
        limits: DomSourceLimits,
        services: &mut DomSourceServices<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSelectCompletionProgress, DomSourceRuntimeError> {
        // Validate every original Session binding before canonical consumption.
        // Cleanup admission and optional services belong only to a live Source.
        let guard = self.qualify_source_owners(dom)?;
        let progress = {
            let mut lease = match dom
                .raw_mut()
                .begin_select_completion(task, checkpoint, limits, cancel)?
            {
                DomSelectCompletionStart::Skipped(completion) => {
                    return Ok(DomSelectCompletionProgress::Skipped(completion));
                }
                DomSelectCompletionStart::Source(lease) => lease,
            };
            guard.reserve_cleanup(cancel)?;
            if let Some(scope) = services.execution_scope {
                self.hosted
                    .qualify_execution_scope(scope)
                    .map_err(DomSourceRuntimeError::Hosted)?;
            }
            if let Some(scope) = services.execution_scope {
                self.hosted
                    .bind_execution_scope(&mut lease, scope, true, cancel)
                    .map_err(DomSourceRuntimeError::Hosted)?;
            }
            self.drive_lease(
                &guard,
                &mut lease,
                services.url.as_ref().map(|service| service.revision),
                cancel,
            )?
        };
        self.finish_or_supply(
            dom,
            DriveRoot {
                limits,
                select: Some(task),
            },
            guard,
            progress,
            services,
            cancel,
        )
        .map(DomSelectCompletionProgress::Source)
    }
    /// Resume Select processing with the same actual task authorization.
    pub fn resume_select(
        &mut self,
        dom: &mut DomEventDom<'_>,
        operation: DomSourceOperationId,
        task: &DomSelectTask,
        limits: DomSourceLimits,
        services: &mut DomSourceServices<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let guard = self.prepare(dom, cancel)?;
        if let Some(scope) = services.execution_scope {
            self.hosted
                .qualify_execution_scope(scope)
                .map_err(DomSourceRuntimeError::Hosted)?;
        }
        let progress = {
            let mut lease = dom
                .raw_mut()
                .resume_select_completion(operation, task, limits, cancel)?;
            if let Some(scope) = services.execution_scope {
                self.hosted
                    .bind_execution_scope(&mut lease, scope, false, cancel)
                    .map_err(DomSourceRuntimeError::Hosted)?;
            }
            self.drive_lease(
                &guard,
                &mut lease,
                services.url.as_ref().map(|service| service.revision),
                cancel,
            )?
        };
        self.finish_or_supply(
            dom,
            DriveRoot {
                limits,
                select: Some(task),
            },
            guard,
            progress,
            services,
            cancel,
        )
    }
}

impl DomSourceRuntime {
    fn drive_lease(
        &self,
        guard: &StyleSourceGuard,
        lease: &mut DomSourceLease<'_>,
        url_revision: Option<&surgeist_html_semantics::HtmlBaseFactsRevision>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        loop {
            guard.flush(lease, cancel)?;
            let settlement = self
                .hosted
                .prepare_watcher_settlement(lease, cancel)
                .map_err(DomSourceRuntimeError::Hosted)?;
            let advanced = lease.advance(cancel);
            settlement.finish(&lease.snapshot());
            match advanced? {
                DomSourceProgress::Request(DomSourceRequest::Event(request)) => {
                    return Ok(DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::Event(request),
                        reason: DomSourceUnavailable::Event,
                    });
                }
                DomSourceProgress::Request(DomSourceRequest::CloseWatcher(request)) => {
                    self.hosted
                        .service_close_watcher_request(lease, &request, cancel)
                        .map_err(DomSourceRuntimeError::Hosted)?;
                }
                DomSourceProgress::Request(DomSourceRequest::Dialog(request)) => {
                    if matches!(
                        request.action(),
                        surgeist_html::HtmlDialogAction::QueueToggle {
                            source: surgeist_html::HtmlDialogEventSource::Null
                                | surgeist_html::HtmlDialogEventSource::OperationSource,
                            ..
                        } | surgeist_html::HtmlDialogAction::QueueClose
                    ) {
                        self.element_tasks
                            .queue_dialog(lease, &request, cancel)
                            .map_err(DomSourceRuntimeError::ElementTasks)?;
                        continue;
                    }
                    if matches!(
                        request.action(),
                        surgeist_html::HtmlDialogAction::EstablishCloseWatcher
                            | surgeist_html::HtmlDialogAction::DestroyCloseWatcher
                    ) && self
                        .hosted
                        .service_dialog_watcher(lease, &request, cancel)
                        .map_err(DomSourceRuntimeError::Hosted)?
                    {
                        continue;
                    }
                    if matches!(
                        request.action(),
                        surgeist_html::HtmlDialogAction::RememberFocusedElement
                    ) && self
                        .hosted
                        .service_focus(
                            lease,
                            &surgeist_dom::DomFocusRequest::RememberDialog(request.clone()),
                            cancel,
                        )
                        .map_err(DomSourceRuntimeError::Hosted)?
                    {
                        continue;
                    }
                    let reason = DomSourceUnavailable::Dialog(request.action());
                    return Ok(DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::Dialog(request),
                        reason,
                    });
                }
                DomSourceProgress::Request(DomSourceRequest::DialogFocusing(request)) => {
                    if request.action() == surgeist_html::HtmlDialogFocusingAction::ReadAllowFocus
                        && self
                            .hosted
                            .service_focus(
                                lease,
                                &surgeist_dom::DomFocusRequest::AllowDialog(request.clone()),
                                cancel,
                            )
                            .map_err(DomSourceRuntimeError::Hosted)?
                    {
                        continue;
                    }
                    let reason = DomSourceUnavailable::DialogFocusing(request.action());
                    return Ok(DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::DialogFocusing(request),
                        reason,
                    });
                }
                DomSourceProgress::Request(DomSourceRequest::FocusDelegate(request)) => {
                    let reason = DomSourceUnavailable::FocusDelegate(request.step());
                    return Ok(DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::FocusDelegate(request),
                        reason,
                    });
                }
                DomSourceProgress::Request(DomSourceRequest::DesignMode(request)) => {
                    let reason = DomSourceUnavailable::DesignMode(request.step());
                    return Ok(DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::DesignMode(request),
                        reason,
                    });
                }
                DomSourceProgress::Request(DomSourceRequest::AutofocusInsertion(request)) => {
                    if request.action()
                        == surgeist_html::HtmlAutofocusInsertionAction::ReadAllowFocus
                        && self
                            .hosted
                            .service_focus(
                                lease,
                                &surgeist_dom::DomFocusRequest::AllowAutofocusInsertion(
                                    request.clone(),
                                ),
                                cancel,
                            )
                            .map_err(DomSourceRuntimeError::Hosted)?
                    {
                        continue;
                    }
                    let reason = DomSourceUnavailable::AutofocusInsertion(request.action());
                    return Ok(DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::AutofocusInsertion(request),
                        reason,
                    });
                }
                DomSourceProgress::Complete(done) => {
                    return Ok(DomSourceDriveProgress::Complete(done));
                }
                DomSourceProgress::Request(DomSourceRequest::HostedChild(request)) => {
                    match self
                        .hosted
                        .service_source_child(lease, &request, url_revision, cancel)
                        .map_err(DomSourceRuntimeError::Hosted)?
                    {
                        HostedSourceProgress::Serviced => {}
                        HostedSourceProgress::Url(stage) => {
                            return Ok(DomSourceDriveProgress::Pending {
                                request: DomSourceRequest::HostedChild(request),
                                reason: DomSourceUnavailable::Url(stage),
                            });
                        }
                    }
                }
                DomSourceProgress::Request(DomSourceRequest::Style(request)) => {
                    match guard.service(lease, request, cancel)? {
                        DomStyleServiceProgress::Serviced => {}
                        DomStyleServiceProgress::Pending { request, reason } => {
                            return Ok(DomSourceDriveProgress::Pending {
                                request: DomSourceRequest::Style(request),
                                reason: DomSourceUnavailable::Style(reason),
                            });
                        }
                    }
                }
            }
        }
    }
}

struct DriveRoot<'a> {
    limits: DomSourceLimits,
    select: Option<&'a DomSelectTask>,
}

impl DomSourceRuntime {
    fn finish_or_supply(
        &self,
        dom: &mut DomEventDom<'_>,
        root: DriveRoot<'_>,
        guard: StyleSourceGuard,
        mut progress: DomSourceDriveProgress,
        services: &mut DomSourceServices<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let DriveRoot { limits, select } = root;
        loop {
            match progress {
                DomSourceDriveProgress::Complete(_) => {
                    guard.finish(&dom.snapshot());
                    return Ok(progress);
                }
                DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Style(request),
                    reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
                } => {
                    // The prior lease and every RefCell/CSSOM borrow ended before
                    // this actual external observation supplier is entered.
                    let Some(observation) = (services.style_policy)(request.source()) else {
                        return Ok(DomSourceDriveProgress::Pending {
                            request: DomSourceRequest::Style(request),
                            reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
                        });
                    };
                    let operation = request.source().operation();
                    let mut lease = if let Some(task) = select {
                        dom.raw_mut()
                            .resume_select_completion(operation, task, limits, cancel)?
                    } else {
                        dom.resume_source_edit(operation, limits, cancel)?
                    };
                    lease.accept_inline_csp(&request, &observation, cancel)?;
                    progress = self.drive_lease(
                        &guard,
                        &mut lease,
                        services.url.as_ref().map(|service| service.revision),
                        cancel,
                    )?;
                }
                DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::HostedChild(child),
                    reason: DomSourceUnavailable::Url(stage),
                } => {
                    let Some(service) = services.url.as_mut() else {
                        return Ok(DomSourceDriveProgress::Pending {
                            request: DomSourceRequest::HostedChild(child),
                            reason: DomSourceUnavailable::Url(stage),
                        });
                    };
                    let operation = child.operation();
                    let issued = {
                        let mut lease = if let Some(task) = select {
                            dom.raw_mut()
                                .resume_select_completion(operation, task, limits, cancel)?
                        } else {
                            dom.resume_source_edit(operation, limits, cancel)?
                        };
                        lease.source_url_request(&child, stage, service.revision, cancel)?
                    };
                    // Both exclusive owners and all issuer/store/RefCell guards are
                    // released before calling the external producer. A rejected
                    // not-yet-admitted candidate may drop; an accepted candidate
                    // belongs to the canonical operation's charged response slot.
                    self.hosted
                        .admit_url_observation(&issued, cancel)
                        .map_err(DomSourceRuntimeError::Hosted)?;
                    let Some(observation) = (service.observe)(&issued, cancel)
                        .map_err(DomSourceRuntimeError::Hosted)?
                    else {
                        return Ok(DomSourceDriveProgress::Pending {
                            request: DomSourceRequest::HostedChild(child),
                            reason: DomSourceUnavailable::Url(stage),
                        });
                    };
                    let mut lease = if let Some(task) = select {
                        dom.raw_mut()
                            .resume_select_completion(operation, task, limits, cancel)?
                    } else {
                        dom.resume_source_edit(operation, limits, cancel)?
                    };
                    lease.accept_source_url_observation(
                        &child,
                        &issued,
                        &mut Some(observation),
                        cancel,
                    )?;
                    progress = self.drive_lease(
                        &guard,
                        &mut lease,
                        services.url.as_ref().map(|service| service.revision),
                        cancel,
                    )?;
                }
                other => return Ok(other),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomSession, DomSessionLimits, DomStyleRuntimeLimits};
    use surgeist_dom::{Dom, DomHostedLimits, DomHostedRecordLimits, DomLimits, DomSourceOutput};

    #[test]
    fn each_requested_creation_returns_its_actual_scalar_without_cross_operation_tickets() {
        let cancel = AtomicBool::new(false);
        let dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut session = DomSession::new(
            dom,
            DomSessionLimits {
                source_events: crate::dom_dialog::DomSourceEventRuntimeLimits {
                    jobs: 0,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                },
                style: DomStyleRuntimeLimits {
                    roots: 4,
                    completions: 8,
                    reports: 8,
                    storage_bytes: 1_000_000,
                    work: 100_000,
                    css_input: surgeist_css::CssUtf16InputLimits::default(),
                },
                element_tasks: crate::DomElementTaskRuntimeLimits {
                    tasks: 8,
                    storage_bytes: 1_000_000,
                    work: 100_000,
                },
                hosted: crate::DomHostedRuntimeLimits {
                    canonical: DomHostedLimits::new(
                        DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                        1_000_000,
                        1_000_000,
                        100_000,
                    ),
                    tasks: 8,
                    storage_bytes: 1_000_000,
                    scratch_bytes: 1_000_000,
                    work: 100_000,
                },
            },
        )
        .unwrap();
        let mut source = session.source_execution();
        let limits = DomSourceLimits::new(1_000_000, 4096, 100_000, 16, 64, 64, 1);
        let DomSourceDriveProgress::Complete(document) = source
            .apply(
                &mut session.dom(),
                DomSourceEdit::create_document(),
                limits,
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| panic!("Document creation has no inline CSS request"),
                    url: None,
                },
                &cancel,
            )
            .unwrap()
        else {
            panic!("actual Document creation must finish");
        };
        let Some(DomSourceOutput::Document(id)) = document.output() else {
            panic!("Document operation must return its actual typed identity");
        };
        assert_eq!(document.published_node(), Some(id.node()));
        assert_eq!(session.snapshot().document(id).unwrap().id(), id);
        let DomSourceDriveProgress::Complete(text) = source
            .apply(
                &mut session.dom(),
                DomSourceEdit::create_text(id.node(), "exact text"),
                limits,
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| panic!("Text creation has no inline CSS request"),
                    url: None,
                },
                &cancel,
            )
            .unwrap()
        else {
            panic!("actual Text creation must finish");
        };
        let Some(DomSourceOutput::Node(node)) = text.output() else {
            panic!("Text operation must return its actual typed identity");
        };
        assert_eq!(text.published_node(), Some(node));
        assert_ne!(document.operation(), text.operation());
        assert_eq!(
            session.snapshot().node(node).unwrap().data().unwrap(),
            &surgeist_html::DocumentString::from("exact text")
        );
        assert_eq!(
            session.snapshot().node(node).unwrap().owner_document(),
            Some(id)
        );
    }
}

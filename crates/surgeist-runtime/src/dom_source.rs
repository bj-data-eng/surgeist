//! One requested Source operation through the original durable session owners.
//!
//! The canonical owner retains the command, hook cursor and acknowledged tail.
//! This weak permission routes actual yielded requests; it owns neither tree nor
//! resources and cannot accept an arbitrary sequence of commands.
use crate::dom_details::{DomDetailsRuntime, DomDetailsRuntimeError};
use crate::dom_event::DomEventDom;
use crate::dom_lifecycle::{DomHostedRuntimeError, HostedRuntime};
use crate::dom_style::{
    DomStyleRuntime, DomStyleRuntimeError, DomStyleServiceProgress, DomStyleUnavailable,
    StyleSourceGuard,
};
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DomError, DomNativeCheckpointLimits, DomNativeHtmlInstallError, DomNativeHtmlInstallTarget,
    DomSelectTask, DomSourceCompletion, DomSourceEdit, DomSourceLease, DomSourceLimits,
    DomSourceNativeHtmlInstallLimits, DomSourceOperationId, DomSourceProgress, DomSourceRequest,
    DomStyleInlineCspObservation, DomStyleSource,
};

/// A missing service at the actual next request, without acknowledging it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceUnavailable {
    /// The original Style owner needs this precise external input or resource.
    Style(DomStyleUnavailable),
}

/// The requested operation returned, or its exact next request remains parked.
#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "Transfer the already admitted Source completion after irreversible publication without another allocation or fallible custody handoff"
)]
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

/// Refusal preserves the original canonical operation and family-owned results.
#[derive(Debug)]
pub enum DomSourceRuntimeError {
    /// Canonical Source validation or finite admission refused the operation.
    Dom(DomError),
    /// The original Style resource or processing owner refused its step.
    Style(DomStyleRuntimeError),
    /// The original bound Details owner is unavailable or foreign.
    Details(DomDetailsRuntimeError),
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
            Self::Dom(error) => Some(error),
            Self::Style(error) => Some(error),
            Self::Details(error) => Some(error),
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
    details: DomDetailsRuntime,
    hosted: HostedRuntime,
}
impl std::fmt::Debug for DomSourceRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomSourceRuntime").finish_non_exhaustive()
    }
}
impl DomSourceRuntime {
    pub(crate) fn new(
        styles: DomStyleRuntime,
        details: DomDetailsRuntime,
        hosted: HostedRuntime,
    ) -> Self {
        Self {
            styles,
            details,
            hosted,
        }
    }
    fn prepare(
        &self,
        dom: &DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<StyleSourceGuard, DomSourceRuntimeError> {
        self.details
            .validate_view(dom)
            .map_err(DomSourceRuntimeError::Details)?;
        self.hosted
            .validate_view(dom)
            .map_err(DomSourceRuntimeError::Hosted)?;
        Ok(self.styles.prepare_source(dom, cancel)?)
    }
    /// Execute exactly one requested edit through the original Source engine.
    pub fn apply(
        &mut self,
        dom: &mut DomEventDom<'_>,
        edit: DomSourceEdit,
        limits: DomSourceLimits,
        policy: impl FnMut(&DomStyleSource) -> Option<DomStyleInlineCspObservation>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let guard = self.prepare(dom, cancel)?;
        let progress = {
            let mut lease = dom.begin_source_edit(edit, limits, cancel)?;
            drive_lease(&guard, &mut lease, cancel)?
        };
        finish_or_supply(dom, limits, None, guard, progress, policy, cancel)
    }
    /// Resume the same operation without rebuilding acknowledged external work.
    pub fn resume(
        &mut self,
        dom: &mut DomEventDom<'_>,
        operation: DomSourceOperationId,
        limits: DomSourceLimits,
        policy: impl FnMut(&DomStyleSource) -> Option<DomStyleInlineCspObservation>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let guard = self.prepare(dom, cancel)?;
        let progress = {
            let mut lease = dom.resume_source_edit(operation, limits, cancel)?;
            drive_lease(&guard, &mut lease, cancel)?
        };
        finish_or_supply(dom, limits, None, guard, progress, policy, cancel)
    }
    /// Install one immutable native fragment using its original feature root.
    pub fn install_native_html(
        &mut self,
        dom: &mut DomEventDom<'_>,
        target: DomNativeHtmlInstallTarget,
        fragment: &surgeist_html::HtmlFragment,
        budgets: DomSourceNativeHtmlInstallLimits,
        policy: impl FnMut(&DomStyleSource) -> Option<DomStyleInlineCspObservation>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let guard = self.prepare(dom, cancel)?;
        let limits = budgets.source();
        let progress = {
            let mut lease = dom
                .raw_mut()
                .begin_native_installation(target, fragment, budgets, cancel)
                .map_err(DomSourceRuntimeError::NativeInstallation)?;
            drive_lease(&guard, &mut lease, cancel)?
        };
        finish_or_supply(dom, limits, None, guard, progress, policy, cancel)
    }
    /// Complete the actual once-authorized Select task through its feature root.
    pub fn complete_select(
        &mut self,
        dom: &mut DomEventDom<'_>,
        task: &DomSelectTask,
        checkpoint: DomNativeCheckpointLimits,
        limits: DomSourceLimits,
        policy: impl FnMut(&DomStyleSource) -> Option<DomStyleInlineCspObservation>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let guard = self.prepare(dom, cancel)?;
        let progress = {
            let mut lease = dom
                .raw_mut()
                .begin_select_completion(task, checkpoint, limits, cancel)?;
            drive_lease(&guard, &mut lease, cancel)?
        };
        finish_or_supply(dom, limits, Some(task), guard, progress, policy, cancel)
    }
    /// Resume Select processing with the same actual task authorization.
    pub fn resume_select(
        &mut self,
        dom: &mut DomEventDom<'_>,
        operation: DomSourceOperationId,
        task: &DomSelectTask,
        limits: DomSourceLimits,
        policy: impl FnMut(&DomStyleSource) -> Option<DomStyleInlineCspObservation>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
        let guard = self.prepare(dom, cancel)?;
        let progress = {
            let mut lease = dom
                .raw_mut()
                .resume_select_completion(operation, task, limits, cancel)?;
            drive_lease(&guard, &mut lease, cancel)?
        };
        finish_or_supply(dom, limits, Some(task), guard, progress, policy, cancel)
    }
}

fn drive_lease(
    guard: &StyleSourceGuard,
    lease: &mut DomSourceLease<'_>,
    cancel: &AtomicBool,
) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
    loop {
        guard.flush(lease, cancel)?;
        match lease.advance(cancel)? {
            DomSourceProgress::Complete(done) => return Ok(DomSourceDriveProgress::Complete(done)),
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

fn finish_or_supply(
    dom: &mut DomEventDom<'_>,
    limits: DomSourceLimits,
    select: Option<&DomSelectTask>,
    guard: StyleSourceGuard,
    mut progress: DomSourceDriveProgress,
    mut policy: impl FnMut(&DomStyleSource) -> Option<DomStyleInlineCspObservation>,
    cancel: &AtomicBool,
) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
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
                let Some(observation) = policy(request.source()) else {
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
                progress = drive_lease(&guard, &mut lease, cancel)?;
            }
            other => return Ok(other),
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
                style: DomStyleRuntimeLimits {
                    roots: 4,
                    completions: 8,
                    reports: 8,
                    storage_bytes: 1_000_000,
                    work: 100_000,
                    css_input: surgeist_css::CssUtf16InputLimits::default(),
                },
                details: crate::DomDetailsRuntimeLimits {
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
        let limits = DomSourceLimits::new(1_000_000, 4096, 100_000, 16, 64, 64);
        let DomSourceDriveProgress::Complete(document) = source
            .apply(
                &mut session.dom(),
                DomSourceEdit::create_document(),
                limits,
                |_| panic!("Document creation has no inline CSS request"),
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
                |_| panic!("Text creation has no inline CSS request"),
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

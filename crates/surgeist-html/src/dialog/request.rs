//! Complete HTML877d watcher request and called Close language protocol.
use std::num::NonZeroUsize;

/// Rejected transitions leave the original watcher state unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCloseWatcherStateError {
    kind: HtmlCloseWatcherStateErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlCloseWatcherStateErrorKind {
    AlreadyRunning,
    NotRunning,
}

impl HtmlCloseWatcherStateErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlCloseWatcherStateError {
        HtmlCloseWatcherStateError { kind: self }
    }
}

impl From<HtmlCloseWatcherStateErrorKind> for HtmlCloseWatcherStateError {
    fn from(kind: HtmlCloseWatcherStateErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlCloseWatcherStateError {
    /// Whether the cause is: close watcher cancel action is already running.
    pub fn is_already_running(&self) -> bool {
        matches!(self.kind, HtmlCloseWatcherStateErrorKind::AlreadyRunning)
    }
    /// Whether the cause is: close watcher cancel action is not running.
    pub fn is_not_running(&self) -> bool {
        matches!(self.kind, HtmlCloseWatcherStateErrorKind::NotRunning)
    }
}

impl std::fmt::Display for HtmlCloseWatcherStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlCloseWatcherStateErrorKind::AlreadyRunning => {
                f.write_str("close watcher cancel action is already running")
            }
            HtmlCloseWatcherStateErrorKind::NotRunning => {
                f.write_str("close watcher cancel action is not running")
            }
        }
    }
}

impl std::error::Error for HtmlCloseWatcherStateError {}

/// Reads and effects on the actual original watcher and Window. The adapter
/// retains identities and admits/executes each effect before acknowledging it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCloseWatcherRequestAction {
    /// Read whether the close watcher is active, that is, whether its Window's close
    /// watcher manager contains it in some group.
    ReadActive,
    /// Read the result of the close watcher's get enabled state.
    ReadEnabled,
    /// Read the close watcher's is running cancel action flag.
    ReadRunningCancelAction,
    /// Retain the request's actual Window for group/history reads and consumption.
    CaptureWindow,
    /// Request uses its captured Window; called Close reads the watcher's Window
    /// afresh. Neither observation can reuse a pre-dispatch activity result.
    ReadWindowFullyActive,
    /// Read the size of the Window's close watcher manager groups and its allowed
    /// number of groups.
    ReadManagerGroups,
    /// Read whether the captured Window has history-action activation.
    ReadHistoryActionActivation,
    /// Use checked begin/finish on the original invocation's retained state,
    /// even when its manager membership was removed during cancel dispatch.
    SetRunningCancelAction(bool),
    /// Run the actual cancel action and report its actual shouldContinue value.
    RunCancelAction {
        /// The `canPreventClose` argument: whether the cancel action's result can stop
        /// the close.
        can_prevent_close: bool,
    },
    /// Consume history-action user activation for the captured Window, as the
    /// request does when the cancel action prevents the close.
    ConsumeHistoryActionActivation,
    /// Remove this watcher from all actual manager groups and remove empty groups.
    DestroyMembership,
    /// Run the watcher's close action after it has been destroyed.
    RunCloseAction,
    /// True is the source return, not evidence that RunCloseAction was reached.
    Complete(bool),
}

/// Known source observations. Allowed groups is intrinsically at least one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCloseWatcherRequestReply {
    /// Whether the close watcher is active.
    Active(bool),
    /// Whether the close watcher's get enabled state returned true.
    Enabled(bool),
    /// Whether the close watcher's is running cancel action flag is set.
    RunningCancelAction(bool),
    /// Whether the Window's associated Document is fully active.
    WindowFullyActive(bool),
    /// The size of the close watcher manager's groups and its allowed number of
    /// groups.
    ManagerGroups {
        /// The number of groups in the Window's close watcher manager.
        groups: usize,
        /// The manager's allowed number of groups, which is at least 1.
        allowed_groups: NonZeroUsize,
    },
    /// Whether the Window has history-action activation.
    HistoryActionActivation(bool),
    /// The cancel action's `shouldContinue` result. `false` is only valid when
    /// `canPreventClose` was `true`.
    CancelActionResult(bool),
    /// The pending effect was fulfilled.
    Applied,
}
/// Why a close-watcher request cursor refused a reply, queried through `is_*`
/// predicates. The cursor stays unchanged after a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCloseWatcherRequestError {
    kind: HtmlCloseWatcherRequestErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlCloseWatcherRequestErrorKind {
    WrongReply,
    Terminal,
    /// Selected correction to the contradictory source cancel-action prose:
    /// a noncancelable action must return true. It cannot prevent Close.
    NoncancelableActionPreventedClose,
}

impl HtmlCloseWatcherRequestErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlCloseWatcherRequestError {
        HtmlCloseWatcherRequestError { kind: self }
    }
}

impl From<HtmlCloseWatcherRequestErrorKind> for HtmlCloseWatcherRequestError {
    fn from(kind: HtmlCloseWatcherRequestErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlCloseWatcherRequestError {
    /// Whether the cause is: reply does not match the pending close watcher action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlCloseWatcherRequestErrorKind::WrongReply)
    }
    /// Whether the cause is: close watcher request is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlCloseWatcherRequestErrorKind::Terminal)
    }
    /// Whether the cause is: noncancelable action cannot prevent close.
    pub fn is_noncancelable_action_prevented_close(&self) -> bool {
        matches!(
            self.kind,
            HtmlCloseWatcherRequestErrorKind::NoncancelableActionPreventedClose
        )
    }
}

impl std::fmt::Display for HtmlCloseWatcherRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlCloseWatcherRequestErrorKind::WrongReply => {
                f.write_str("reply does not match the pending close watcher action")
            }
            HtmlCloseWatcherRequestErrorKind::Terminal => {
                f.write_str("close watcher request is already complete")
            }
            HtmlCloseWatcherRequestErrorKind::NoncancelableActionPreventedClose => {
                f.write_str("noncancelable action cannot prevent close")
            }
        }
    }
}

impl std::error::Error for HtmlCloseWatcherRequestError {}

/// Allocation-free language cursor with no identity registry, event dispatcher,
/// manager or invocation custody. Every read is fresh at its source position.
/// Dropping or copying this value performs no guard reset or external effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCloseWatcherRequestCursor {
    pending: HtmlCloseWatcherRequestAction,
    require_history_action_activation: bool,
    closing: bool,
    can_prevent_close: bool,
    should_continue: bool,
}
impl HtmlCloseWatcherRequestCursor {
    /// Starts a request to close with the given `requireHistoryActionActivation`
    /// argument. The first pending action is the active read.
    pub const fn new(require_history_action_activation: bool) -> Self {
        Self {
            pending: HtmlCloseWatcherRequestAction::ReadActive,
            require_history_action_activation,
            closing: false,
            can_prevent_close: false,
            should_continue: false,
        }
    }
    /// Returns the read or effect that must be fulfilled before the next reply.
    /// After `Complete`, it returns the final result.
    pub const fn action(&self) -> HtmlCloseWatcherRequestAction {
        self.pending
    }
    /// Wrong, terminal and impossible replies preserve the entire cursor.
    pub fn reply(
        &mut self,
        reply: HtmlCloseWatcherRequestReply,
    ) -> Result<(), HtmlCloseWatcherRequestError> {
        if matches!(self.pending, HtmlCloseWatcherRequestAction::Complete(_)) {
            return Err(HtmlCloseWatcherRequestErrorKind::Terminal.error());
        }
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }
    fn advance(
        &mut self,
        reply: HtmlCloseWatcherRequestReply,
    ) -> Result<(), HtmlCloseWatcherRequestError> {
        use HtmlCloseWatcherRequestAction as A;
        use HtmlCloseWatcherRequestReply as R;
        self.pending = match (self.pending, reply) {
            (A::ReadActive, R::Active(false))
            | (A::ReadEnabled, R::Enabled(false))
            | (A::ReadRunningCancelAction, R::RunningCancelAction(true))
            | (A::ReadWindowFullyActive, R::WindowFullyActive(false)) => A::Complete(true),
            (A::ReadActive, R::Active(true)) => A::ReadEnabled,
            (A::ReadEnabled, R::Enabled(true)) => {
                if self.closing {
                    A::ReadWindowFullyActive
                } else {
                    A::ReadRunningCancelAction
                }
            }
            (A::ReadRunningCancelAction, R::RunningCancelAction(false)) => A::CaptureWindow,
            (A::CaptureWindow, R::Applied) => A::ReadWindowFullyActive,
            (A::ReadWindowFullyActive, R::WindowFullyActive(true)) => {
                if self.closing {
                    A::DestroyMembership
                } else if self.require_history_action_activation {
                    A::ReadManagerGroups
                } else {
                    self.can_prevent_close = true;
                    A::SetRunningCancelAction(true)
                }
            }
            (
                A::ReadManagerGroups,
                R::ManagerGroups {
                    groups,
                    allowed_groups,
                },
            ) => {
                if groups < allowed_groups.get() {
                    A::ReadHistoryActionActivation
                } else {
                    self.can_prevent_close = false;
                    A::SetRunningCancelAction(true)
                }
            }
            (A::ReadHistoryActionActivation, R::HistoryActionActivation(active)) => {
                self.can_prevent_close = active;
                A::SetRunningCancelAction(true)
            }
            (A::SetRunningCancelAction(true), R::Applied) => A::RunCancelAction {
                can_prevent_close: self.can_prevent_close,
            },
            (A::RunCancelAction { can_prevent_close }, R::CancelActionResult(should_continue)) => {
                if !can_prevent_close && !should_continue {
                    return Err(
                        HtmlCloseWatcherRequestErrorKind::NoncancelableActionPreventedClose.error(),
                    );
                }
                self.should_continue = should_continue;
                A::SetRunningCancelAction(false)
            }
            (A::SetRunningCancelAction(false), R::Applied) => {
                if self.should_continue {
                    self.closing = true;
                    A::ReadActive
                } else {
                    A::ConsumeHistoryActionActivation
                }
            }
            (A::ConsumeHistoryActionActivation, R::Applied) => A::Complete(false),
            (A::DestroyMembership, R::Applied) => A::RunCloseAction,
            (A::RunCloseAction, R::Applied) => A::Complete(true),
            _ => return Err(HtmlCloseWatcherRequestErrorKind::WrongReply.error()),
        };
        Ok(())
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_close_watcher_request_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlCloseWatcherRequestError) -> bool;
        let cases: Vec<(HtmlCloseWatcherRequestError, &str, Predicate)> = vec![
            (
                HtmlCloseWatcherRequestErrorKind::WrongReply.error(),
                "reply does not match the pending close watcher action",
                HtmlCloseWatcherRequestError::is_wrong_reply,
            ),
            (
                HtmlCloseWatcherRequestErrorKind::Terminal.error(),
                "close watcher request is already complete",
                HtmlCloseWatcherRequestError::is_terminal,
            ),
            (
                HtmlCloseWatcherRequestErrorKind::NoncancelableActionPreventedClose.error(),
                "noncancelable action cannot prevent close",
                HtmlCloseWatcherRequestError::is_noncancelable_action_prevented_close,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }

    #[test]
    fn html_close_watcher_state_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlCloseWatcherStateError) -> bool;
        let cases: Vec<(HtmlCloseWatcherStateError, &str, Predicate)> = vec![
            (
                HtmlCloseWatcherStateErrorKind::AlreadyRunning.error(),
                "close watcher cancel action is already running",
                HtmlCloseWatcherStateError::is_already_running,
            ),
            (
                HtmlCloseWatcherStateErrorKind::NotRunning.error(),
                "close watcher cancel action is not running",
                HtmlCloseWatcherStateError::is_not_running,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}

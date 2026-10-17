//! Complete HTML877d watcher request and called Close language protocol.
use std::num::NonZeroUsize;

/// Rejected transitions leave the original watcher state unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCloseWatcherStateError {
    AlreadyRunning,
    NotRunning,
}
impl std::fmt::Display for HtmlCloseWatcherStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::AlreadyRunning => "close watcher cancel action is already running",
            Self::NotRunning => "close watcher cancel action is not running",
        })
    }
}
impl std::error::Error for HtmlCloseWatcherStateError {}

/// Reads and effects on the actual original watcher and Window. The adapter
/// retains identities and admits/executes each effect before acknowledging it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCloseWatcherRequestAction {
    ReadActive,
    ReadEnabled,
    ReadRunningCancelAction,
    /// Retain the request's actual Window for group/history reads and consumption.
    CaptureWindow,
    /// Request uses its captured Window; called Close reads the watcher's Window
    /// afresh. Neither observation can reuse a pre-dispatch activity result.
    ReadWindowFullyActive,
    ReadManagerGroups,
    ReadHistoryActionActivation,
    /// Use checked begin/finish on the original invocation's retained state,
    /// even when its manager membership was removed during cancel dispatch.
    SetRunningCancelAction(bool),
    /// Run the actual cancel action and report its actual shouldContinue value.
    RunCancelAction {
        can_prevent_close: bool,
    },
    ConsumeHistoryActionActivation,
    /// Remove this watcher from all actual manager groups and remove empty groups.
    DestroyMembership,
    RunCloseAction,
    /// True is the source return, not evidence that RunCloseAction was reached.
    Complete(bool),
}

/// Known source observations. Allowed groups is intrinsically at least one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCloseWatcherRequestReply {
    Active(bool),
    Enabled(bool),
    RunningCancelAction(bool),
    WindowFullyActive(bool),
    ManagerGroups {
        groups: usize,
        allowed_groups: NonZeroUsize,
    },
    HistoryActionActivation(bool),
    CancelActionResult(bool),
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCloseWatcherRequestError {
    WrongReply,
    Terminal,
    /// Selected correction to the contradictory source cancel-action prose:
    /// a noncancelable action must return true. It cannot prevent Close.
    NoncancelableActionPreventedClose,
}
impl std::fmt::Display for HtmlCloseWatcherRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending close watcher action",
            Self::Terminal => "close watcher request is already complete",
            Self::NoncancelableActionPreventedClose => "noncancelable action cannot prevent close",
        })
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
    pub const fn new(require_history_action_activation: bool) -> Self {
        Self {
            pending: HtmlCloseWatcherRequestAction::ReadActive,
            require_history_action_activation,
            closing: false,
            can_prevent_close: false,
            should_continue: false,
        }
    }
    pub const fn action(&self) -> HtmlCloseWatcherRequestAction {
        self.pending
    }
    /// Wrong, terminal and impossible replies preserve the entire cursor.
    pub fn reply(
        &mut self,
        reply: HtmlCloseWatcherRequestReply,
    ) -> Result<(), HtmlCloseWatcherRequestError> {
        if matches!(self.pending, HtmlCloseWatcherRequestAction::Complete(_)) {
            return Err(HtmlCloseWatcherRequestError::Terminal);
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
                    return Err(HtmlCloseWatcherRequestError::NoncancelableActionPreventedClose);
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
            _ => return Err(HtmlCloseWatcherRequestError::WrongReply),
        };
        Ok(())
    }
}

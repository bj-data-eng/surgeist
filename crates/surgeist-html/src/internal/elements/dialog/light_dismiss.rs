//! HTML877d's light-dismiss sequence for open Dialogs.
use super::request::{
    HtmlCloseWatcherRequestAction, HtmlCloseWatcherRequestCursor, HtmlCloseWatcherRequestError,
    HtmlCloseWatcherRequestReply,
};
use crate::internal::attributes::HtmlClosedBy;

/// Pointer event types that can change a Document's Dialog pointerdown target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogLightDismissKind {
    /// Remember the nearest clicked Dialog for the original pointerdown.
    PointerDown,
    /// Compare the original pointerup with the stored pointerdown target.
    PointerUp,
}

/// The terminal state of the Dialog portion of light dismiss.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogLightDismissOutcome {
    /// The actual event failed the source's trusted-event assertion.
    UntrustedEvent,
    /// The source sequence returned before a watcher request.
    Finished,
    /// The observed topmost Dialog had no close watcher at the source assertion.
    MissingCloseWatcher,
    /// The original close-watcher request returned this Boolean. A true result
    /// also includes the watcher's early returns.
    WatcherFinished {
        /// The original request's result before returning from light dismiss.
        should_continue: bool,
    },
}

/// A read or effect on the original event's Document and retained operands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogLightDismissAction {
    /// Observe the original event's trusted flag.
    ReadEventTrusted,
    /// Capture the actual trusted event target's node Document.
    CaptureTargetDocument,
    /// Check the captured Document's actual open Dialog list.
    ReadOpenDialogsEmpty,
    /// Run nearest clicked Dialog and retain its exact nullable result.
    CaptureNearestClickedDialog,
    /// Store that exact nullable result as the Document's pointerdown target.
    StorePointerDownTarget,
    /// Compare the retained result to the Document's actual pointerdown target.
    ComparePointerDownTarget,
    /// Set the captured Document's pointerdown target to null.
    ClearPointerDownTarget,
    /// Retain the last entry of this Document's open Dialog list and compare it
    /// with the original nearest clicked Dialog result.
    CompareLastOpenDialog,
    /// Read the computed closed-by state of the retained last open Dialog.
    ReadClosedBy,
    /// Capture the retained last open Dialog's current close watcher.
    CaptureCloseWatcher,
    /// Fulfill one action of the original request against the captured watcher.
    CloseWatcher(HtmlCloseWatcherRequestAction),
    /// The original sequence has reached its terminal outcome.
    Complete(HtmlDialogLightDismissOutcome),
}

/// Qualified observations and acknowledgements. An unavailable observation
/// leaves the exact action pending until the host can supply it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogLightDismissReply {
    /// The original event's actual trusted flag.
    EventTrusted(bool),
    /// The event target's node Document is retained.
    TargetDocumentCaptured,
    /// Whether the retained Document's open Dialog list is empty.
    OpenDialogsEmpty(bool),
    /// The nearest clicked Dialog operation completed with a retained nullable result.
    NearestClickedDialogCaptured,
    /// The Document's pointerdown target now holds that exact result.
    PointerDownTargetStored,
    /// Whether the stored target equals the result, including two actual nulls.
    PointerDownTargetMatches(bool),
    /// The Document's pointerdown target was set to null.
    PointerDownTargetCleared,
    /// Whether the retained last open Dialog is the nearest clicked Dialog.
    LastOpenDialogMatches(bool),
    /// The retained Dialog's computed state.
    ClosedBy(HtmlClosedBy),
    /// Whether the retained Dialog's close watcher is nonnull and captured.
    CloseWatcherPresent(bool),
    /// The observation or effect result for the exact watcher request action.
    CloseWatcher(HtmlCloseWatcherRequestReply),
}

/// A reply rejected without advancing the pending sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDialogLightDismissReplyError {
    kind: ReplyErrorKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReplyErrorKind {
    WrongReply,
    Terminal,
    CloseWatcher(HtmlCloseWatcherRequestError),
}
impl HtmlDialogLightDismissReplyError {
    /// Whether the reply did not match the pending light-dismiss action.
    pub const fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, ReplyErrorKind::WrongReply)
    }
    /// Whether the light-dismiss sequence had already ended.
    pub const fn is_terminal(&self) -> bool {
        matches!(self.kind, ReplyErrorKind::Terminal)
    }
    /// Return the original protocol error from a reached watcher action.
    pub const fn close_watcher_error(&self) -> Option<&HtmlCloseWatcherRequestError> {
        match &self.kind {
            ReplyErrorKind::CloseWatcher(error) => Some(error),
            ReplyErrorKind::WrongReply | ReplyErrorKind::Terminal => None,
        }
    }
}
impl std::fmt::Display for HtmlDialogLightDismissReplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ReplyErrorKind::WrongReply => {
                f.write_str("reply does not match the pending Dialog light-dismiss action")
            }
            ReplyErrorKind::Terminal => f.write_str("Dialog light dismiss is already complete"),
            ReplyErrorKind::CloseWatcher(error) => std::fmt::Display::fmt(&error, f),
        }
    }
}
impl std::error::Error for HtmlDialogLightDismissReplyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.close_watcher_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

#[derive(Clone, Copy, Debug)]
enum Step {
    ReadEventTrusted,
    CaptureTargetDocument,
    ReadOpenDialogsEmpty,
    CaptureNearestClickedDialog,
    StorePointerDownTarget,
    ComparePointerDownTarget,
    ClearPointerDownTarget { same_target: bool },
    CompareLastOpenDialog,
    ReadClosedBy,
    CaptureCloseWatcher,
    RequestClose(HtmlCloseWatcherRequestCursor),
    Complete(HtmlDialogLightDismissOutcome),
}

/// Bounded Dialog light-dismiss control flow. The caller retains the actual
/// event, Document, nearest clicked Dialog, last open Dialog and watcher as the
/// ordered actions capture them.
#[derive(Debug)]
pub struct HtmlDialogLightDismissCursor {
    kind: HtmlDialogLightDismissKind,
    step: Step,
}
impl HtmlDialogLightDismissCursor {
    /// Start the Dialog portion after the popover portion has run for the
    /// original pointer event. The first action checks the event's trust.
    pub const fn new(kind: HtmlDialogLightDismissKind) -> Self {
        Self {
            kind,
            step: Step::ReadEventTrusted,
        }
    }

    /// Return the next source observation or effect.
    pub const fn action(&self) -> HtmlDialogLightDismissAction {
        use HtmlDialogLightDismissAction as A;
        match self.step {
            Step::ReadEventTrusted => A::ReadEventTrusted,
            Step::CaptureTargetDocument => A::CaptureTargetDocument,
            Step::ReadOpenDialogsEmpty => A::ReadOpenDialogsEmpty,
            Step::CaptureNearestClickedDialog => A::CaptureNearestClickedDialog,
            Step::StorePointerDownTarget => A::StorePointerDownTarget,
            Step::ComparePointerDownTarget => A::ComparePointerDownTarget,
            Step::ClearPointerDownTarget { .. } => A::ClearPointerDownTarget,
            Step::CompareLastOpenDialog => A::CompareLastOpenDialog,
            Step::ReadClosedBy => A::ReadClosedBy,
            Step::CaptureCloseWatcher => A::CaptureCloseWatcher,
            Step::RequestClose(request) => A::CloseWatcher(request.action()),
            Step::Complete(outcome) => A::Complete(outcome),
        }
    }

    /// Advance after the caller fulfills the current action.
    pub fn reply(
        &mut self,
        reply: HtmlDialogLightDismissReply,
    ) -> Result<(), HtmlDialogLightDismissReplyError> {
        use HtmlDialogLightDismissOutcome as O;
        use HtmlDialogLightDismissReply as R;
        self.step = match (self.step, reply) {
            (Step::ReadEventTrusted, R::EventTrusted(true)) => Step::CaptureTargetDocument,
            (Step::ReadEventTrusted, R::EventTrusted(false)) => Step::Complete(O::UntrustedEvent),
            (Step::CaptureTargetDocument, R::TargetDocumentCaptured) => Step::ReadOpenDialogsEmpty,
            (Step::ReadOpenDialogsEmpty, R::OpenDialogsEmpty(true))
            | (Step::StorePointerDownTarget, R::PointerDownTargetStored)
            | (Step::CompareLastOpenDialog, R::LastOpenDialogMatches(true))
            | (Step::ReadClosedBy, R::ClosedBy(HtmlClosedBy::None | HtmlClosedBy::CloseRequest))
            | (Step::ClearPointerDownTarget { same_target: false }, R::PointerDownTargetCleared) => {
                Step::Complete(O::Finished)
            }
            (Step::ReadOpenDialogsEmpty, R::OpenDialogsEmpty(false)) => {
                Step::CaptureNearestClickedDialog
            }
            (Step::CaptureNearestClickedDialog, R::NearestClickedDialogCaptured) => match self.kind
            {
                HtmlDialogLightDismissKind::PointerDown => Step::StorePointerDownTarget,
                HtmlDialogLightDismissKind::PointerUp => Step::ComparePointerDownTarget,
            },
            (Step::ComparePointerDownTarget, R::PointerDownTargetMatches(same_target)) => {
                Step::ClearPointerDownTarget { same_target }
            }
            (Step::ClearPointerDownTarget { same_target: true }, R::PointerDownTargetCleared) => {
                Step::CompareLastOpenDialog
            }
            (Step::CompareLastOpenDialog, R::LastOpenDialogMatches(false)) => Step::ReadClosedBy,
            (Step::ReadClosedBy, R::ClosedBy(HtmlClosedBy::Any)) => Step::CaptureCloseWatcher,
            (Step::CaptureCloseWatcher, R::CloseWatcherPresent(false)) => {
                Step::Complete(O::MissingCloseWatcher)
            }
            (Step::CaptureCloseWatcher, R::CloseWatcherPresent(true)) => {
                Step::RequestClose(HtmlCloseWatcherRequestCursor::new(false))
            }
            (Step::RequestClose(mut request), R::CloseWatcher(reply)) => {
                request
                    .reply(reply)
                    .map_err(|error| HtmlDialogLightDismissReplyError {
                        kind: ReplyErrorKind::CloseWatcher(error),
                    })?;
                match request.action() {
                    HtmlCloseWatcherRequestAction::Complete(should_continue) => {
                        Step::Complete(O::WatcherFinished { should_continue })
                    }
                    _ => Step::RequestClose(request),
                }
            }
            (Step::Complete(_), _) => {
                return Err(HtmlDialogLightDismissReplyError {
                    kind: ReplyErrorKind::Terminal,
                });
            }
            _ => {
                return Err(HtmlDialogLightDismissReplyError {
                    kind: ReplyErrorKind::WrongReply,
                });
            }
        };
        Ok(())
    }
}

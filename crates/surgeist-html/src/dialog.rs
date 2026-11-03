//! HTML877d Dialog language transitions, without a tree or service executor.
//!
//! The adapter binds actual operands/identities, admits each read/effect and
//! acknowledges only fulfilled actions. Missing evidence is not a false reply.
//! Synchronous dispatch, focus, watchers, popover algorithms and layer services
//! remain real requirements. This producer grants no hosted Document capability.
//! The bounded operations are nonmodal show, close, insertion/removal and open
//! changes, and standard null-source requestClose/showModal. Light-dismiss
//! execution remains separate.
mod focusing;
mod request;
use crate::{HtmlClosedBy, HtmlClosedByState, KnownHtmlAttribute, Namespace};
pub use focusing::*;
pub use request::*;

/// Ordered partial observations for HTML's allow-focus steps. Unknown feature
/// permission cannot be replaced by a known activation state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAllowFocusDecision {
    /// The document's feature permission is not yet known, so neither it nor the
    /// activation state decides the result. Nothing is concluded from unknown input.
    NeedsFeaturePermission,
    /// The feature permission is known to be negative and the Window's transient
    /// activation is not yet known.
    NeedsTransientActivation,
    /// Focusing is permitted, either by feature permission or by transient activation.
    Allowed,
    /// The feature permission and transient activation are both known to be negative.
    Denied,
}
/// First test the document's actual feature permission; only a negative result
/// consults the relevant Window's transient activation. This decides permission,
/// never whether any focusing effects were performed.
pub const fn allow_focus_decision(
    feature_permission: Option<bool>,
    transient_activation: Option<bool>,
) -> HtmlAllowFocusDecision {
    match feature_permission {
        None => HtmlAllowFocusDecision::NeedsFeaturePermission,
        Some(true) => HtmlAllowFocusDecision::Allowed,
        Some(false) => match transient_activation {
            None => HtmlAllowFocusDecision::NeedsTransientActivation,
            Some(true) => HtmlAllowFocusDecision::Allowed,
            Some(false) => HtmlAllowFocusDecision::Denied,
        },
    }
}

/// Exact creation values, not a getter for an advanced live Dialog.
/// Common HTML previous-focus initialization remains its existing owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDialogFreshState {
    _private: (),
}
impl HtmlDialogFreshState {
    /// Returns the creation values of a new Dialog. Each accessor reports the
    /// initial value that HTML 4.11.4 (The dialog element) gives its field.
    pub const fn fresh() -> Self {
        Self { _private: () }
    }
    /// Returns the initial `returnValue`, the empty string.
    pub const fn return_value(&self) -> &'static str {
        ""
    }
    /// Returns the initial value of the dialog's is modal flag, always `false`.
    pub const fn is_modal(&self) -> bool {
        false
    }
    /// Returns whether a new dialog has a toggle task tracker, always `false`.
    pub const fn has_toggle_task_tracker(&self) -> bool {
        false
    }
    /// Returns whether a new dialog has a close watcher, always `false`.
    pub const fn has_close_watcher(&self) -> bool {
        false
    }
    /// Returns whether a new dialog has a request close return value, always `false`
    /// because the value is initially null.
    pub const fn has_request_close_return_value(&self) -> bool {
        false
    }
    /// Returns whether a new dialog has a request close source element, always
    /// `false` because the element is initially null.
    pub const fn has_request_close_source(&self) -> bool {
        false
    }
    /// Returns the initial value of the enable close watcher for request close flag,
    /// always `false`.
    pub const fn force_close_watcher_enabled(&self) -> bool {
        false
    }
}

/// One exact recognized open-attribute change, including equal presence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDialogOpenChange {
    old_present: bool,
    new_present: bool,
}
impl HtmlDialogOpenChange {
    /// Returns whether the `open` attribute was present before the change.
    pub const fn old_present(self) -> bool {
        self.old_present
    }
    /// Returns whether the `open` attribute is present after the change.
    pub const fn new_present(self) -> bool {
        self.new_present
    }
    const fn added(self) -> bool {
        !self.old_present && self.new_present
    }
    const fn removed(self) -> bool {
        self.old_present && !self.new_present
    }
}

/// The selected source algorithm. Actual close text/source remain with the
/// adapter for their original lifetime; supplied empty text is still present.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogOperation {
    /// The `show()` method steps of HTML 4.11.4 (The dialog element).
    Show,
    /// Standard showModal method with null source.
    ShowModal,
    /// Standard requestClose method; the adapter retains its actual nullable result.
    RequestClose,
    /// The close algorithm for a dialog to be closed with a null or string result.
    Close {
        /// Whether a non-null result was supplied. An empty string is still supplied and
        /// sets `returnValue`.
        result_present: bool,
    },
    /// The dialog HTML element insertion steps.
    Inserted,
    /// The dialog HTML element removing steps.
    Removed,
    /// The attribute change steps for the `open` attribute, with the exact presence
    /// change.
    OpenChanged(HtmlDialogOpenChange),
}

/// Recognizes only exact null-namespace Open, without scanning any strings.
pub fn dialog_attribute_operation(
    namespace: &Namespace,
    attribute: KnownHtmlAttribute,
    old_present: bool,
    new_present: bool,
) -> Option<HtmlDialogOperation> {
    if namespace.uri().is_some() || attribute != KnownHtmlAttribute::Open {
        return None;
    }
    Some(HtmlDialogOperation::OpenChanged(HtmlDialogOpenChange {
        old_present,
        new_present,
    }))
}

/// Exact ToggleEvent state spelling, independent of current attributes at delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogState {
    /// The ToggleEvent state `"closed"`.
    Closed,
    /// The ToggleEvent state `"open"`.
    Open,
}
impl HtmlDialogState {
    /// Returns the exact ToggleEvent state string, `"closed"` or `"open"`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
        }
    }
}
/// A source operand role; this contains no actual Element identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogEventSource {
    /// The source is null, as for the `show()`, `showModal()` and `requestClose()`
    /// methods.
    Null,
    /// Use the original nullable close source retained by the adapter.
    OperationSource,
}
/// Normal source return versus a show()/showModal() InvalidState error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogOutcome {
    /// The operation returned normally.
    Completed,
    /// The operation throws an `"InvalidStateError"` DOMException, which only
    /// `show()` and `showModal()` can reach.
    InvalidState,
}

/// Next source read/effect. Each read uses actual state at this source position.
/// Effects run through the existing ordinary mutation/service owner before ack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogAction {
    /// Read whether the dialog has an `open` attribute.
    ReadOpen,
    /// Read the dialog's is modal flag.
    ReadModal,
    /// Read whether the dialog's node document is fully active.
    ReadFullyActive,
    /// Read whether the dialog is connected.
    ReadConnected,
    /// Read actual popover showing state, not its authored popover attribute.
    ReadPopoverShowing,
    /// Read whether the dialog's close watcher is nonnull.
    ReadWatcherPresent,
    /// Capture the asserted nonnull watcher. Absence violates the source
    /// invariant; it cannot be acknowledged or treated as an early return.
    CaptureCloseWatcher,
    /// Set the dialog's enable close watcher for request close flag to the supplied
    /// value.
    SetForceCloseWatcherEnabled(bool),
    /// Write the actual nullable method argument, including null or empty text.
    WriteRequestCloseReturnValue,
    /// Write null for the standard method source.
    WriteNullRequestCloseSource,
    /// Run the captured watcher request through its actual owner to completion.
    RequestWatcherClose {
        /// The `requireHistoryActionActivation` argument of the close-watcher request to
        /// close. The `requestClose()` steps pass `false`.
        require_history_action_activation: bool,
    },
    /// Read the receiver's then-current node Document's actual top layer.
    ReadTopLayerContains,
    /// Retain the actual previous-focus identity locally when present.
    ReadPreviousFocusPresent,
    /// Use the then-current node Document's focused-area DOM anchor and actual
    /// shadow-including inclusive descendant relation to this subject.
    ReadFocusedAnchorInside,
    /// Fire a `beforetoggle` ToggleEvent at the dialog and reply whether it was
    /// canceled. A noncancelable event is acknowledged with `Applied` instead.
    FireBeforeToggle {
        /// The `oldState` attribute of the event.
        old: HtmlDialogState,
        /// The `newState` attribute of the event.
        new: HtmlDialogState,
        /// The `cancelable` attribute of the event: `true` for `show()` and
        /// `showModal()`, `false` for closing.
        cancelable: bool,
        /// The source attribute of the event: null, or the operation's original source.
        source: HtmlDialogEventSource,
    },
    /// Run the source Dialog toggle task operation on the DOM-manipulation task
    /// source, using the actual original nullable source. Not event delivery.
    QueueToggle {
        /// The `oldState` that the queued `toggle` task reports.
        old: HtmlDialogState,
        /// The `newState` that the queued `toggle` task reports.
        new: HtmlDialogState,
        /// The source attribute of the queued `toggle` task.
        source: HtmlDialogEventSource,
    },
    /// Ordinary empty open addition, including its complete nested attribute hook.
    AddOpen,
    /// Assert the actual watcher is nonnull after AddOpen and its complete
    /// nested hook. This neither establishes nor captures an invocation.
    AssertCloseWatcherPresent,
    /// Set the receiver's modal state to literal true.
    SetModal,
    /// Block the receiver's then-current node Document by this receiver.
    BlockNodeDocument,
    /// Execute actual top-layer addition for the receiver.
    AddTopLayer,
    /// Ordinary open removal, including its complete nested attribute hook.
    RemoveOpen,
    /// Store the actually focused Element (or known null), not a desired target.
    RememberFocusedElement,
    /// Retain the current node Document as show's local `document`.
    CaptureNodeDocument,
    /// Find topmost popover ancestor of subject with source null/isPopover=false;
    /// retain its actual endpoint for the next action.
    FindTopmostPopoverAncestor,
    /// Use the captured show Document and endpoint, even after nested reentry.
    HidePopoversUntil {
        /// Whether the hide step focuses the previously focused element. The dialog
        /// algorithms pass `false`.
        focus_previous_element: bool,
        /// Whether the hide step fires events. The dialog algorithms pass `true`.
        fire_events: bool,
    },
    /// Fulfill the pinned Dialog focusing algorithm, including actual allow-focus,
    /// autofocus/delegate, focus, navigable/origin and autofocus-state requirements.
    /// An absent service cannot acknowledge this action as complete.
    RunDialogFocusingSteps,
    /// Add the dialog to its node document's open dialogs list.
    AddOpenDialog,
    /// Remove the dialog from its node document's open dialogs list.
    RemoveOpenDialog,
    /// Establish the actual watcher with source cancelAction firing cancel with
    /// canPreventClose, closeAction using stored request result/source, and
    /// getEnabledState from dialog_close_watcher_enabled. Requires actual null
    /// watcher/open/fully-active setup preconditions, never a fabricated handle.
    EstablishCloseWatcher,
    /// Destroy the dialog's close watcher.
    DestroyCloseWatcher,
    /// Set the dialog's close watcher to null.
    ClearCloseWatcher,
    /// Request that the dialog be removed from the top layer, as `close` does for a
    /// modal dialog.
    RequestTopLayerRemoval,
    /// Remove the dialog from the top layer immediately, as the removing steps do.
    RemoveTopLayerImmediately,
    /// Set the dialog's is modal flag to `false`.
    ClearModal,
    /// Write the original supplied result, including supplied empty text.
    WriteReturnValue,
    /// Set the dialog's request close return value to null.
    ClearRequestCloseReturnValue,
    /// Set the dialog's request close source element to null.
    ClearRequestCloseSource,
    /// Set the dialog's previously focused element to null.
    ClearPreviouslyFocusedElement,
    /// Focus the previously retained actual Element without scrolling viewport.
    RestorePreviousFocusWithoutScrolling,
    /// Queue the close event on the user-interaction task source.
    QueueClose,
    /// The transition is finished with this outcome. No reply is accepted afterward.
    Complete(HtmlDialogOutcome),
}

/// A reply for exactly the pending read or fulfilled effect. False is known
/// false, never unavailable. Only cancelable beforetoggle consumes EventCanceled;
/// its noncancelable close counterpart consumes Applied after actual dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogReply {
    /// Whether the dialog has an `open` attribute.
    Open(bool),
    /// Whether the dialog's is modal flag is true.
    Modal(bool),
    /// Whether the dialog's node document is fully active.
    FullyActive(bool),
    /// Whether the dialog is connected.
    Connected(bool),
    /// Whether the dialog is in the popover showing state.
    PopoverShowing(bool),
    /// Whether the dialog's close watcher is nonnull.
    WatcherPresent(bool),
    /// Whether the node document's top layer contains the dialog.
    TopLayerContains(bool),
    /// Whether the dialog's previously focused element is nonnull.
    PreviousFocusPresent(bool),
    /// Whether the focused area's DOM anchor is a shadow-including inclusive
    /// descendant of the dialog.
    FocusedAnchorInside(bool),
    /// Whether the fired `beforetoggle` event was canceled.
    EventCanceled(bool),
    /// Normal return of the called watcher request, distinct from event dispatch.
    WatcherRequestResult(bool),
    /// The pending effect was fulfilled.
    Applied,
}
/// Why a Dialog cursor refused a reply, queried through `is_*` predicates. The
/// cursor keeps its pending action unchanged after a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDialogCursorError {
    kind: HtmlDialogCursorErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlDialogCursorErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlDialogCursorErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlDialogCursorError {
        HtmlDialogCursorError { kind: self }
    }
}

impl From<HtmlDialogCursorErrorKind> for HtmlDialogCursorError {
    fn from(kind: HtmlDialogCursorErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlDialogCursorError {
    /// Whether the cause is: reply does not match the pending Dialog action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlDialogCursorErrorKind::WrongReply)
    }
    /// Whether the cause is: Dialog transition is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlDialogCursorErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlDialogCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlDialogCursorErrorKind::WrongReply => {
                f.write_str("reply does not match the pending Dialog action")
            }
            HtmlDialogCursorErrorKind::Terminal => {
                f.write_str("Dialog transition is already complete")
            }
        }
    }
}

impl std::error::Error for HtmlDialogCursorError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModalRead {
    Show,
    ShowModal,
    Removal,
    WasModal,
}

/// Constant-state, allocation-free language protocol. It is not a DOM operation
/// token, service receipt or continuation owner. Cloning gives no mutation or
/// replay authority. Caller admission covers every actual action and nested
/// iterative frame; there is no hidden allowance or internal unbounded loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDialogCursor {
    operation: HtmlDialogOperation,
    pending: HtmlDialogAction,
    after_dispatch: bool,
    modal_read: ModalRead,
    was_modal: bool,
}
impl HtmlDialogCursor {
    /// Starts the transition for `operation`. The pending action is the operation's
    /// first read or effect, which the adapter fulfills and acknowledges with
    /// [`reply`](Self::reply).
    pub const fn new(operation: HtmlDialogOperation) -> Self {
        use HtmlDialogAction as A;
        let pending = match operation {
            HtmlDialogOperation::Show
            | HtmlDialogOperation::ShowModal
            | HtmlDialogOperation::RequestClose
            | HtmlDialogOperation::Close { .. }
            | HtmlDialogOperation::Removed => A::ReadOpen,
            HtmlDialogOperation::Inserted => A::ReadFullyActive,
            HtmlDialogOperation::OpenChanged(change) => {
                if change.removed() {
                    A::RemoveOpenDialog
                } else {
                    A::ReadFullyActive
                }
            }
        };
        Self {
            operation,
            pending,
            after_dispatch: false,
            modal_read: ModalRead::Show,
            was_modal: false,
        }
    }
    /// Returns the read or effect that must be fulfilled before the next reply.
    /// After `Complete`, it returns the final outcome.
    pub const fn action(&self) -> HtmlDialogAction {
        self.pending
    }

    /// Wrong/terminal replies preserve all fields and the exact pending action.
    /// Missing facts, cancellation or unfinished services make no reply. The
    /// actual adapter retains its source obligations under the owning protocol.
    pub fn reply(&mut self, reply: HtmlDialogReply) -> Result<(), HtmlDialogCursorError> {
        if matches!(self.pending, HtmlDialogAction::Complete(_)) {
            return Err(HtmlDialogCursorErrorKind::Terminal.error());
        }
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }
    const fn completed() -> HtmlDialogAction {
        HtmlDialogAction::Complete(HtmlDialogOutcome::Completed)
    }
    const fn after_cleanup(&self) -> HtmlDialogAction {
        if matches!(self.operation, HtmlDialogOperation::Removed) {
            HtmlDialogAction::ReadTopLayerContains
        } else {
            HtmlDialogAction::ReadFullyActive
        }
    }
    fn read_open(&mut self, open: bool) -> Result<HtmlDialogAction, HtmlDialogCursorError> {
        use HtmlDialogAction as A;
        use HtmlDialogOperation as O;
        Ok(match self.operation {
            O::Show if open && !self.after_dispatch => {
                self.modal_read = ModalRead::Show;
                A::ReadModal
            }
            O::Show if open => Self::completed(),
            O::Show if self.after_dispatch => A::QueueToggle {
                old: HtmlDialogState::Closed,
                new: HtmlDialogState::Open,
                source: HtmlDialogEventSource::Null,
            },
            O::Show => A::FireBeforeToggle {
                old: HtmlDialogState::Closed,
                new: HtmlDialogState::Open,
                cancelable: true,
                source: HtmlDialogEventSource::Null,
            },
            O::ShowModal if open && !self.after_dispatch => {
                self.modal_read = ModalRead::ShowModal;
                A::ReadModal
            }
            O::ShowModal if open => Self::completed(),
            O::ShowModal if self.after_dispatch => A::ReadConnected,
            O::ShowModal => A::ReadFullyActive,
            O::RequestClose => {
                if open {
                    A::ReadConnected
                } else {
                    Self::completed()
                }
            }
            O::Close { .. } if !open => Self::completed(),
            O::Close { .. } if self.after_dispatch => A::QueueToggle {
                old: HtmlDialogState::Open,
                new: HtmlDialogState::Closed,
                source: HtmlDialogEventSource::OperationSource,
            },
            O::Close { .. } => A::FireBeforeToggle {
                old: HtmlDialogState::Open,
                new: HtmlDialogState::Closed,
                cancelable: false,
                source: HtmlDialogEventSource::OperationSource,
            },
            O::Inserted => {
                if open {
                    A::ReadConnected
                } else {
                    Self::completed()
                }
            }
            O::Removed => {
                if open {
                    A::RemoveOpenDialog
                } else {
                    A::ReadTopLayerContains
                }
            }
            O::OpenChanged(_) => return Err(HtmlDialogCursorErrorKind::WrongReply.error()),
        })
    }
    fn advance(&mut self, reply: HtmlDialogReply) -> Result<(), HtmlDialogCursorError> {
        use HtmlDialogAction as A;
        use HtmlDialogOperation as O;
        use HtmlDialogReply as R;
        self.pending = match (self.pending, reply) {
            (A::ReadOpen, R::Open(open)) => self.read_open(open)?,
            (A::ReadModal, R::Modal(modal)) => match self.modal_read {
                ModalRead::Show => A::Complete(if modal {
                    HtmlDialogOutcome::InvalidState
                } else {
                    HtmlDialogOutcome::Completed
                }),
                ModalRead::ShowModal => A::Complete(if modal {
                    HtmlDialogOutcome::Completed
                } else {
                    HtmlDialogOutcome::InvalidState
                }),
                ModalRead::Removal => {
                    self.modal_read = ModalRead::WasModal;
                    if modal {
                        A::RequestTopLayerRemoval
                    } else {
                        A::ReadModal
                    }
                }
                ModalRead::WasModal => {
                    self.was_modal = modal;
                    A::ClearModal
                }
            },
            (
                A::FireBeforeToggle {
                    cancelable: true, ..
                },
                R::EventCanceled(canceled),
            ) => {
                if canceled {
                    Self::completed()
                } else {
                    self.after_dispatch = true;
                    A::ReadOpen
                }
            }
            (
                A::FireBeforeToggle {
                    cancelable: false, ..
                },
                R::Applied,
            ) => {
                self.after_dispatch = true;
                A::ReadOpen
            }
            (A::QueueToggle { .. }, R::Applied) => {
                if matches!(self.operation, O::Show | O::ShowModal) {
                    A::AddOpen
                } else {
                    A::RemoveOpen
                }
            }
            (A::AddOpen, R::Applied) => {
                if matches!(self.operation, O::ShowModal) {
                    A::AssertCloseWatcherPresent
                } else {
                    A::RememberFocusedElement
                }
            }
            (A::AssertCloseWatcherPresent, R::Applied) => A::SetModal,
            (A::SetModal, R::Applied) => A::BlockNodeDocument,
            (A::BlockNodeDocument, R::Applied) => A::ReadTopLayerContains,
            (A::AddTopLayer, R::Applied) => A::RememberFocusedElement,
            (A::RememberFocusedElement, R::Applied) => A::CaptureNodeDocument,
            (A::CaptureNodeDocument, R::Applied) => A::FindTopmostPopoverAncestor,
            (A::FindTopmostPopoverAncestor, R::Applied) => A::HidePopoversUntil {
                focus_previous_element: false,
                fire_events: true,
            },
            (A::HidePopoversUntil { .. }, R::Applied) => A::RunDialogFocusingSteps,
            (A::RunDialogFocusingSteps | A::EstablishCloseWatcher | A::QueueClose, R::Applied) => {
                Self::completed()
            }
            (A::RemoveOpen, R::Applied) => {
                self.modal_read = ModalRead::Removal;
                A::ReadModal
            }
            (A::RequestTopLayerRemoval, R::Applied) => A::ReadModal,
            (A::ClearModal, R::Applied) => match self.operation {
                O::Close {
                    result_present: true,
                } => A::WriteReturnValue,
                O::Close {
                    result_present: false,
                } => A::ClearRequestCloseReturnValue,
                O::Removed => Self::completed(),
                _ => return Err(HtmlDialogCursorErrorKind::WrongReply.error()),
            },
            (A::WriteReturnValue, R::Applied) => A::ClearRequestCloseReturnValue,
            (A::ClearRequestCloseReturnValue, R::Applied) => A::ClearRequestCloseSource,
            (A::ClearRequestCloseSource, R::Applied) => A::ReadPreviousFocusPresent,
            (A::ReadPreviousFocusPresent, R::PreviousFocusPresent(present)) => {
                if present {
                    A::ClearPreviouslyFocusedElement
                } else {
                    A::QueueClose
                }
            }
            (A::ClearPreviouslyFocusedElement, R::Applied) => A::ReadFocusedAnchorInside,
            (A::ReadFocusedAnchorInside, R::FocusedAnchorInside(inside)) => {
                if inside || self.was_modal {
                    A::RestorePreviousFocusWithoutScrolling
                } else {
                    A::QueueClose
                }
            }
            (A::RestorePreviousFocusWithoutScrolling, R::Applied) => A::QueueClose,
            (A::CaptureCloseWatcher, R::Applied) => A::SetForceCloseWatcherEnabled(true),
            (A::SetForceCloseWatcherEnabled(true), R::Applied) => A::WriteRequestCloseReturnValue,
            (A::WriteRequestCloseReturnValue, R::Applied) => A::WriteNullRequestCloseSource,
            (A::WriteNullRequestCloseSource, R::Applied) => A::RequestWatcherClose {
                require_history_action_activation: false,
            },
            (A::RequestWatcherClose { .. }, R::WatcherRequestResult(_)) => {
                A::SetForceCloseWatcherEnabled(false)
            }
            (A::SetForceCloseWatcherEnabled(false), R::Applied) => Self::completed(),
            (A::ReadFullyActive, R::FullyActive(active)) => {
                if !active && matches!(self.operation, O::ShowModal) {
                    A::Complete(HtmlDialogOutcome::InvalidState)
                } else if !active {
                    Self::completed()
                } else if matches!(self.operation, O::RequestClose) {
                    A::CaptureCloseWatcher
                } else if matches!(self.operation, O::Inserted) {
                    A::ReadOpen
                } else {
                    A::ReadConnected
                }
            }
            (A::ReadConnected, R::Connected(connected)) => {
                let setup = matches!(self.operation, O::Inserted)
                    || matches!(self.operation, O::OpenChanged(change) if change.added());
                if matches!(self.operation, O::ShowModal) {
                    if connected {
                        A::ReadPopoverShowing
                    } else if self.after_dispatch {
                        Self::completed()
                    } else {
                        A::Complete(HtmlDialogOutcome::InvalidState)
                    }
                } else if !connected {
                    Self::completed()
                } else if matches!(self.operation, O::RequestClose) {
                    A::ReadFullyActive
                } else if setup {
                    A::AddOpenDialog
                } else {
                    Self::completed()
                }
            }
            (A::ReadPopoverShowing, R::PopoverShowing(showing)) => {
                if showing && self.after_dispatch {
                    Self::completed()
                } else if showing {
                    A::Complete(HtmlDialogOutcome::InvalidState)
                } else if self.after_dispatch {
                    A::QueueToggle {
                        old: HtmlDialogState::Closed,
                        new: HtmlDialogState::Open,
                        source: HtmlDialogEventSource::Null,
                    }
                } else {
                    A::FireBeforeToggle {
                        old: HtmlDialogState::Closed,
                        new: HtmlDialogState::Open,
                        cancelable: true,
                        source: HtmlDialogEventSource::Null,
                    }
                }
            }
            (A::AddOpenDialog, R::Applied) => A::EstablishCloseWatcher,
            (A::RemoveOpenDialog, R::Applied) => A::ReadWatcherPresent,
            (A::ReadWatcherPresent, R::WatcherPresent(present)) => {
                if present {
                    A::DestroyCloseWatcher
                } else {
                    self.after_cleanup()
                }
            }
            (A::DestroyCloseWatcher, R::Applied) => A::ClearCloseWatcher,
            (A::ClearCloseWatcher, R::Applied) => self.after_cleanup(),
            (A::ReadTopLayerContains, R::TopLayerContains(contains)) => {
                if matches!(self.operation, O::ShowModal) {
                    if contains {
                        A::RememberFocusedElement
                    } else {
                        A::AddTopLayer
                    }
                } else if contains {
                    A::RemoveTopLayerImmediately
                } else {
                    A::ClearModal
                }
            }
            (A::RemoveTopLayerImmediately, R::Applied) => A::ClearModal,
            _ => return Err(HtmlDialogCursorErrorKind::WrongReply.error()),
        };
        Ok(())
    }
}

/// Source computed closed-by state; lexical/default interpretation stays with
/// the existing shared closedby rule.
pub const fn computed_dialog_closed_by(state: HtmlClosedByState, modal: bool) -> HtmlClosedBy {
    match state {
        HtmlClosedByState::Auto => {
            if modal {
                HtmlClosedBy::CloseRequest
            } else {
                HtmlClosedBy::None
            }
        }
        HtmlClosedByState::Keyword(value) => value,
    }
}
/// Returns the dialog close watcher's get enabled state: `force` (the enable
/// close watcher for request close flag) or a computed closed-by state other than
/// `None`. See [`computed_dialog_closed_by`].
pub fn dialog_close_watcher_enabled(force: bool, state: HtmlClosedByState, modal: bool) -> bool {
    force || computed_dialog_closed_by(state, modal) != HtmlClosedBy::None
}

/// Fixed Dialog toggle task steps. The adapter owns real tasks and the nullable
/// source. Equal states still queue/fire. Dispatch's ClearTracker clears the
/// actual current tracker even when a callback replaced it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogToggleEffect {
    /// Remove the tracked toggle task from its task queue.
    RemoveTrackedTask,
    /// Set the dialog toggle task tracker to null.
    ClearTracker,
    /// Capture the latest queue request's source, never the replaced task's source.
    QueueEventFromLatestSource {
        /// The `oldState` of the queued event.
        old: HtmlDialogState,
        /// The `newState` of the queued event.
        new: HtmlDialogState,
    },
    /// Use the just-queued real task and the retained first old state.
    StoreTracker {
        /// The old state that the tracker retains.
        old: HtmlDialogState,
    },
    /// Use the queued source/payload, not current attributes or a live source lookup.
    FireQueuedEvent {
        /// The `oldState` of the fired event.
        old: HtmlDialogState,
        /// The `newState` of the fired event.
        new: HtmlDialogState,
    },
}
/// Language-only first-old/latest-new decision; it retains no task/source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDialogToggleDecision {
    old: HtmlDialogState,
    new: HtmlDialogState,
    replacing: bool,
}
impl HtmlDialogToggleDecision {
    /// Computes the states a queued toggle task reports. A tracked task's old state
    /// replaces `old`, so equal or reversed states still queue an event.
    pub const fn new(
        old: HtmlDialogState,
        new: HtmlDialogState,
        tracked_old: Option<HtmlDialogState>,
    ) -> Self {
        Self {
            old: match tracked_old {
                Some(first) => first,
                None => old,
            },
            new,
            replacing: tracked_old.is_some(),
        }
    }
    /// Returns the `oldState` that the next event reports.
    pub const fn old_state(self) -> HtmlDialogState {
        self.old
    }
    /// Returns the `newState` that the next event reports.
    pub const fn new_state(self) -> HtmlDialogState {
        self.new
    }
    /// Actual owner executes/admit these ordered steps before advancing its queue.
    pub fn queue_effects(self) -> impl Iterator<Item = HtmlDialogToggleEffect> {
        use HtmlDialogToggleEffect as E;
        [
            self.replacing.then_some(E::RemoveTrackedTask),
            self.replacing.then_some(E::ClearTracker),
            Some(E::QueueEventFromLatestSource {
                old: self.old,
                new: self.new,
            }),
            Some(E::StoreTracker { old: self.old }),
        ]
        .into_iter()
        .flatten()
    }
    /// Run ClearTracker only after real firing returns; no scheduler/dispatch
    /// completion is inferred from producing these instructions.
    pub const fn dispatch_effects(self) -> [HtmlDialogToggleEffect; 2] {
        [
            HtmlDialogToggleEffect::FireQueuedEvent {
                old: self.old,
                new: self.new,
            },
            HtmlDialogToggleEffect::ClearTracker,
        ]
    }
}

/// Pure manager policy. Runtime owns the ordered groups and watcher records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCloseWatcherManager {
    allowed_groups: usize,
    next_interaction_allows_new_group: bool,
}
/// Placement selected from the actual manager and current group count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCloseWatcherPlacement {
    /// The new watcher forms a new group in the Window's close watcher manager.
    NewGroup,
    /// The new watcher joins the last existing group.
    LastGroup,
}
impl HtmlCloseWatcherManager {
    /// Returns the Window defaults of HTML 6.10.2 (Close watcher infrastructure): one
    /// allowed group and a next user interaction that allows a new group.
    pub const fn fresh() -> Self {
        Self {
            allowed_groups: 1,
            next_interaction_allows_new_group: true,
        }
    }
    /// Returns the allowed number of groups, at least 1.
    pub const fn allowed_groups(self) -> usize {
        self.allowed_groups
    }
    /// Returns whether the next user interaction allows a new group.
    pub const fn next_interaction_allows_new_group(self) -> bool {
        self.next_interaction_allows_new_group
    }
    /// Chooses the placement of a new watcher among `groups` existing groups: a new
    /// group when `groups` is below the allowed number, otherwise the last group.
    pub const fn establishment(self, groups: usize) -> HtmlCloseWatcherPlacement {
        if groups < self.allowed_groups {
            HtmlCloseWatcherPlacement::NewGroup
        } else {
            HtmlCloseWatcherPlacement::LastGroup
        }
    }
}
/// Original fresh watcher cancellation state; no invocation route is implied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCloseWatcherState {
    running_cancel_action: bool,
}
impl HtmlCloseWatcherState {
    /// Returns a watcher state whose is running cancel action flag is `false`.
    pub const fn fresh() -> Self {
        Self {
            running_cancel_action: false,
        }
    }
    /// Returns whether the watcher's cancel action is running, which makes a nested
    /// request to close refuse.
    pub const fn running_cancel_action(self) -> bool {
        self.running_cancel_action
    }
    /// Begin only an actually reached cancel action. No drop-based reset occurs.
    pub fn begin_cancel_action(&mut self) -> Result<(), HtmlCloseWatcherStateError> {
        if self.running_cancel_action {
            return Err(HtmlCloseWatcherStateErrorKind::AlreadyRunning.error());
        }
        self.running_cancel_action = true;
        Ok(())
    }
    /// Finish only after the original cancel action has actually returned.
    pub fn finish_cancel_action(&mut self) -> Result<(), HtmlCloseWatcherStateError> {
        if !self.running_cancel_action {
            return Err(HtmlCloseWatcherStateErrorKind::NotRunning.error());
        }
        self.running_cancel_action = false;
        Ok(())
    }
}

#[cfg(test)]
mod close_watcher_tests {
    use super::*;
    #[test]
    fn fresh_manager_places_first_group_then_shares_last_without_enabling_auto_dialog() {
        let manager = HtmlCloseWatcherManager::fresh();
        assert_eq!(manager.allowed_groups(), 1);
        assert!(manager.next_interaction_allows_new_group());
        assert_eq!(
            manager.establishment(0),
            HtmlCloseWatcherPlacement::NewGroup
        );
        assert_eq!(
            manager.establishment(1),
            HtmlCloseWatcherPlacement::LastGroup
        );
        assert!(!HtmlCloseWatcherState::fresh().running_cancel_action());
        assert!(!dialog_close_watcher_enabled(
            false,
            HtmlClosedByState::Auto,
            false
        ));
        assert!(dialog_close_watcher_enabled(
            false,
            HtmlClosedByState::Keyword(HtmlClosedBy::CloseRequest),
            false
        ));
    }
}

#[cfg(test)]
mod allow_focus_tests {
    use super::{HtmlAllowFocusDecision as D, allow_focus_decision};
    #[test]
    fn permission_precedes_activation_and_unknown_is_not_false() {
        assert_eq!(allow_focus_decision(None, None), D::NeedsFeaturePermission);
        assert_eq!(
            allow_focus_decision(None, Some(false)),
            D::NeedsFeaturePermission
        );
        assert_eq!(
            allow_focus_decision(None, Some(true)),
            D::NeedsFeaturePermission
        );
        assert_eq!(allow_focus_decision(Some(true), None), D::Allowed);
        assert_eq!(allow_focus_decision(Some(true), Some(false)), D::Allowed);
        assert_eq!(
            allow_focus_decision(Some(false), None),
            D::NeedsTransientActivation
        );
        assert_eq!(allow_focus_decision(Some(false), Some(true)), D::Allowed);
        assert_eq!(allow_focus_decision(Some(false), Some(false)), D::Denied);
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_dialog_cursor_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlDialogCursorError) -> bool;
        let cases: Vec<(HtmlDialogCursorError, &str, Predicate)> = vec![
            (
                HtmlDialogCursorErrorKind::WrongReply.error(),
                "reply does not match the pending Dialog action",
                HtmlDialogCursorError::is_wrong_reply,
            ),
            (
                HtmlDialogCursorErrorKind::Terminal.error(),
                "Dialog transition is already complete",
                HtmlDialogCursorError::is_terminal,
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

//! HTML877d Dialog language transitions, without a tree or service executor.
//!
//! The adapter binds actual operands/identities, admits each read/effect and
//! acknowledges only fulfilled actions. Missing evidence is not a false reply.
//! Synchronous dispatch, focus, watchers, popover algorithms and layer services
//! remain real requirements. This producer grants no hosted Document capability.
//! The bounded operations are nonmodal show, close, insertion/removal and open
//! changes. showModal, requestClose and light-dismiss execution remain separate.
use crate::{HtmlClosedBy, HtmlClosedByState, KnownHtmlAttribute, Namespace};

/// Ordered partial observations for HTML's allow-focus steps. Unknown feature
/// permission cannot be replaced by a known activation state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAllowFocusDecision {
    NeedsFeaturePermission,
    NeedsTransientActivation,
    Allowed,
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
    pub const fn fresh() -> Self {
        Self { _private: () }
    }
    pub const fn return_value(&self) -> &'static str {
        ""
    }
    pub const fn is_modal(&self) -> bool {
        false
    }
    pub const fn has_toggle_task_tracker(&self) -> bool {
        false
    }
    pub const fn has_close_watcher(&self) -> bool {
        false
    }
    pub const fn has_request_close_return_value(&self) -> bool {
        false
    }
    pub const fn has_request_close_source(&self) -> bool {
        false
    }
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
    pub const fn old_present(self) -> bool {
        self.old_present
    }
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
    Show,
    Close { result_present: bool },
    Inserted,
    Removed,
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
    Closed,
    Open,
}
impl HtmlDialogState {
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
    Null,
    /// Use the original nullable close source retained by the adapter.
    OperationSource,
}
/// Normal source return versus the show() InvalidState error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogOutcome {
    Completed,
    InvalidState,
}

/// Next source read/effect. Each read uses actual state at this source position.
/// Effects run through the existing ordinary mutation/service owner before ack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogAction {
    ReadOpen,
    ReadModal,
    ReadFullyActive,
    ReadConnected,
    ReadWatcherPresent,
    ReadTopLayerContains,
    /// Retain the actual previous-focus identity locally when present.
    ReadPreviousFocusPresent,
    /// Use the then-current node Document's focused-area DOM anchor and actual
    /// shadow-including inclusive descendant relation to this subject.
    ReadFocusedAnchorInside,
    FireBeforeToggle {
        old: HtmlDialogState,
        new: HtmlDialogState,
        cancelable: bool,
        source: HtmlDialogEventSource,
    },
    /// Run the source Dialog toggle task operation on the DOM-manipulation task
    /// source, using the actual original nullable source. Not event delivery.
    QueueToggle {
        old: HtmlDialogState,
        new: HtmlDialogState,
        source: HtmlDialogEventSource,
    },
    /// Ordinary empty open addition, including its complete nested attribute hook.
    AddOpen,
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
        focus_previous_element: bool,
        fire_events: bool,
    },
    /// Fulfill the pinned Dialog focusing algorithm, including actual allow-focus,
    /// autofocus/delegate, focus, navigable/origin and autofocus-state requirements.
    /// An absent service cannot acknowledge this action as complete.
    RunDialogFocusingSteps,
    AddOpenDialog,
    RemoveOpenDialog,
    /// Establish the actual watcher with source cancelAction firing cancel with
    /// canPreventClose, closeAction using stored request result/source, and
    /// getEnabledState from dialog_close_watcher_enabled. Requires actual null
    /// watcher/open/fully-active setup preconditions, never a fabricated handle.
    EstablishCloseWatcher,
    DestroyCloseWatcher,
    ClearCloseWatcher,
    RequestTopLayerRemoval,
    RemoveTopLayerImmediately,
    ClearModal,
    /// Write the original supplied result, including supplied empty text.
    WriteReturnValue,
    ClearRequestCloseReturnValue,
    ClearRequestCloseSource,
    ClearPreviouslyFocusedElement,
    /// Focus the previously retained actual Element without scrolling viewport.
    RestorePreviousFocusWithoutScrolling,
    /// Queue the close event on the user-interaction task source.
    QueueClose,
    Complete(HtmlDialogOutcome),
}

/// A reply for exactly the pending read or fulfilled effect. False is known
/// false, never unavailable. Only cancelable beforetoggle consumes EventCanceled;
/// its noncancelable close counterpart consumes Applied after actual dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogReply {
    Open(bool),
    Modal(bool),
    FullyActive(bool),
    Connected(bool),
    WatcherPresent(bool),
    TopLayerContains(bool),
    PreviousFocusPresent(bool),
    FocusedAnchorInside(bool),
    EventCanceled(bool),
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogCursorError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlDialogCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending Dialog action",
            Self::Terminal => "Dialog transition is already complete",
        })
    }
}
impl std::error::Error for HtmlDialogCursorError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModalRead {
    Show,
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
    pub const fn new(operation: HtmlDialogOperation) -> Self {
        use HtmlDialogAction as A;
        let pending = match operation {
            HtmlDialogOperation::Show
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
    pub const fn action(&self) -> HtmlDialogAction {
        self.pending
    }

    /// Wrong/terminal replies preserve all fields and the exact pending action.
    /// Missing facts, cancellation or unfinished services make no reply. The
    /// actual adapter retains its source obligations under the owning protocol.
    pub fn reply(&mut self, reply: HtmlDialogReply) -> Result<(), HtmlDialogCursorError> {
        if matches!(self.pending, HtmlDialogAction::Complete(_)) {
            return Err(HtmlDialogCursorError::Terminal);
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
            O::OpenChanged(_) => return Err(HtmlDialogCursorError::WrongReply),
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
                if matches!(self.operation, O::Show) {
                    A::AddOpen
                } else {
                    A::RemoveOpen
                }
            }
            (A::AddOpen, R::Applied) => A::RememberFocusedElement,
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
                _ => return Err(HtmlDialogCursorError::WrongReply),
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
            (A::ReadFullyActive, R::FullyActive(active)) => {
                if !active {
                    Self::completed()
                } else if matches!(self.operation, O::Inserted) {
                    A::ReadOpen
                } else {
                    A::ReadConnected
                }
            }
            (A::ReadConnected, R::Connected(connected)) => {
                let setup = matches!(self.operation, O::Inserted)
                    || matches!(self.operation, O::OpenChanged(change) if change.added());
                if connected && setup {
                    A::AddOpenDialog
                } else {
                    Self::completed()
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
                if contains {
                    A::RemoveTopLayerImmediately
                } else {
                    A::ClearModal
                }
            }
            (A::RemoveTopLayerImmediately, R::Applied) => A::ClearModal,
            _ => return Err(HtmlDialogCursorError::WrongReply),
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
pub fn dialog_close_watcher_enabled(force: bool, state: HtmlClosedByState, modal: bool) -> bool {
    force || computed_dialog_closed_by(state, modal) != HtmlClosedBy::None
}

/// Fixed Dialog toggle task steps. The adapter owns real tasks and the nullable
/// source. Equal states still queue/fire. Dispatch's ClearTracker clears the
/// actual current tracker even when a callback replaced it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogToggleEffect {
    RemoveTrackedTask,
    ClearTracker,
    /// Capture the latest queue request's source, never the replaced task's source.
    QueueEventFromLatestSource {
        old: HtmlDialogState,
        new: HtmlDialogState,
    },
    /// Use the just-queued real task and the retained first old state.
    StoreTracker {
        old: HtmlDialogState,
    },
    /// Use the queued source/payload, not current attributes or a live source lookup.
    FireQueuedEvent {
        old: HtmlDialogState,
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
    pub const fn old_state(self) -> HtmlDialogState {
        self.old
    }
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
    NewGroup,
    LastGroup,
}
impl HtmlCloseWatcherManager {
    pub const fn fresh() -> Self {
        Self {
            allowed_groups: 1,
            next_interaction_allows_new_group: true,
        }
    }
    pub const fn allowed_groups(self) -> usize {
        self.allowed_groups
    }
    pub const fn next_interaction_allows_new_group(self) -> bool {
        self.next_interaction_allows_new_group
    }
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
    pub const fn fresh() -> Self {
        Self {
            running_cancel_action: false,
        }
    }
    pub const fn running_cancel_action(self) -> bool {
        self.running_cancel_action
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

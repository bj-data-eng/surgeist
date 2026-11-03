//! Dialog language transitions and the close-watcher protocols, without a tree or service executor.

pub use crate::internal::elements::dialog::focusing::{
    HtmlDialogFocusControl, HtmlDialogFocusingAction, HtmlDialogFocusingCursor,
    HtmlDialogFocusingError, HtmlDialogFocusingReply, HtmlDialogSubjectFocusFacts,
    HtmlDialogSubjectFocusingDecision, dialog_subject_focusing_decision,
};
pub use crate::internal::elements::dialog::request::{
    HtmlCloseWatcherRequestAction, HtmlCloseWatcherRequestCursor, HtmlCloseWatcherRequestError,
    HtmlCloseWatcherRequestReply, HtmlCloseWatcherStateError,
};
pub use crate::internal::elements::dialog::{
    HtmlAllowFocusDecision, HtmlCloseWatcherManager, HtmlCloseWatcherPlacement,
    HtmlCloseWatcherState, HtmlDialogAction, HtmlDialogCursor, HtmlDialogCursorError,
    HtmlDialogEventSource, HtmlDialogFreshState, HtmlDialogOpenChange, HtmlDialogOperation,
    HtmlDialogOutcome, HtmlDialogReply, HtmlDialogState, HtmlDialogToggleDecision,
    HtmlDialogToggleEffect, allow_focus_decision, computed_dialog_closed_by,
    dialog_attribute_operation, dialog_close_watcher_enabled,
};

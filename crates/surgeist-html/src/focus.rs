//! Document-level focus algorithms.
//!
//! Autofocus, the currently focused area, focus chains, delegates, focus events,
//! focus updates and focusable areas.

pub use crate::internal::focus::autofocus::{
    HtmlAutofocusInsertionAction, HtmlAutofocusInsertionCursor, HtmlAutofocusInsertionError,
    HtmlAutofocusInsertionReply, autofocus_presence,
};
pub use crate::internal::focus::current_focus::{
    HtmlCurrentlyFocusedAreaAction, HtmlCurrentlyFocusedAreaCursor, HtmlCurrentlyFocusedAreaError,
    HtmlCurrentlyFocusedAreaOutcome, HtmlCurrentlyFocusedAreaReply, HtmlFocusedAreaKind,
};
pub use crate::internal::focus::focus_chain::{
    HtmlCurrentFocusChainAction, HtmlCurrentFocusChainCursor, HtmlCurrentFocusChainError,
    HtmlCurrentFocusChainReply, HtmlFocusChainAction, HtmlFocusChainCursor, HtmlFocusChainError,
    HtmlFocusChainReply,
};
pub use crate::internal::focus::focus_delegate::{
    HtmlAutofocusDelegateAction, HtmlAutofocusDelegateCursor, HtmlAutofocusDelegateError,
    HtmlAutofocusDelegateOutcome, HtmlAutofocusDelegateReply, HtmlFocusDelegateAction,
    HtmlFocusDelegateCandidate, HtmlFocusDelegateCursor, HtmlFocusDelegateError,
    HtmlFocusDelegateOutcome, HtmlFocusDelegateReply, HtmlFocusDelegateRoot,
    HtmlFocusDelegateShadowRoot, HtmlFocusDelegateTargetKind, HtmlFocusTriggerKind,
};
pub use crate::internal::focus::focus_event::{
    HtmlFocusEventAction, HtmlFocusEventCursor, HtmlFocusEventError, HtmlFocusEventInterface,
    HtmlFocusEventName, HtmlFocusEventReply, HtmlFocusEventTargetKind, HtmlFocusEventView,
};
pub use crate::internal::focus::focus_update::{
    HtmlFocusUpdateAction, HtmlFocusUpdateChain, HtmlFocusUpdateCursor, HtmlFocusUpdateEntryKind,
    HtmlFocusUpdateError, HtmlFocusUpdateEventTarget, HtmlFocusUpdateInputData,
    HtmlFocusUpdateRelatedTarget, HtmlFocusUpdateReply,
};
pub use crate::internal::focus::focusing::{
    HtmlFocusingAction, HtmlFocusingCursor, HtmlFocusingError, HtmlFocusingFallback,
    HtmlFocusingReply,
};
pub use crate::internal::focus::get_focusable_area::{
    HtmlGetFocusableAreaAction, HtmlGetFocusableAreaCursor, HtmlGetFocusableAreaError,
    HtmlGetFocusableAreaOutcome, HtmlGetFocusableAreaReply,
};

//! HTML877d's multiple Select user toggle and ordered update notification.

use super::control_values::HtmlOptionFlags;

/// Qualified mode and disabledness of the actual Select receiving the user toggle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectUserToggleState {
    /// The current Select lacks `multiple`.
    Single,
    /// The current multiple Select is disabled.
    DisabledMultiple,
    /// The current multiple Select is enabled.
    EnabledMultiple,
}

/// The target's qualified relationship to the actual Select's current option list.
#[derive(Debug)]
pub enum HtmlSelectUserToggleTarget<'a> {
    /// The target is a current member and is enabled.
    EnabledMember(&'a mut HtmlOptionFlags),
    /// The target is a current member and is disabled.
    DisabledMember,
    /// The target is outside the actual option list.
    OutsideList,
}

/// Applies the local selectedness and dirtiness effects of a qualified user toggle.
/// The owner publishes the flag change and queues the returned notification
/// before the related interaction event. An ineligible toggle changes nothing.
#[must_use = "a successful toggle requires its original user-interaction notification"]
pub fn toggle_select_option_for_user(
    state: HtmlSelectUserToggleState,
    target: HtmlSelectUserToggleTarget<'_>,
) -> Option<HtmlSelectNotificationCursor> {
    if let (
        HtmlSelectUserToggleState::EnabledMultiple,
        HtmlSelectUserToggleTarget::EnabledMember(flags),
    ) = (state, target)
    {
        flags.toggle_selectedness_for_user();
        Some(HtmlSelectNotificationCursor {
            action: Some(HtmlSelectNotificationAction::MarkUserValidity),
        })
    } else {
        None
    }
}

/// A trusted event delivered by the original Select user-interaction task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectNotificationEvent {
    /// The bubbling and composed `input` event.
    Input,
    /// The bubbling `change` event.
    Change,
}

impl HtmlSelectNotificationEvent {
    /// Both events bubble.
    pub const fn bubbles(self) -> bool {
        true
    }

    /// Only `input` is composed.
    pub const fn composed(self) -> bool {
        matches!(self, Self::Input)
    }

    /// Neither notification can be canceled.
    pub const fn cancelable(self) -> bool {
        false
    }
}

/// The next effect of the original Select user-interaction task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectNotificationAction {
    /// Set the actual Select's user validity to true.
    MarkUserValidity,
    /// Update the actual Select's current descendant selectedcontent.
    UpdateSelectedcontent,
    /// Update the actual UA fallback node with the current first selected
    /// Option's label, or the empty string when no Option is selected.
    UpdateFallbackLabel,
    /// Dispatch the named trusted event on the original Select.
    Dispatch(HtmlSelectNotificationEvent),
}

/// Completion of the corresponding reached effect on the original Select.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectNotificationReply {
    /// The original Select's user validity was set.
    UserValiditySet,
    /// The descendant selectedcontent algorithm returned.
    SelectedcontentUpdated,
    /// The current label was written to the actual UA fallback node.
    FallbackLabelUpdated,
    /// The actual event dispatch returned.
    Dispatched(HtmlSelectNotificationEvent),
}

/// A notification reply was rejected without changing the pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectNotificationReplyError {
    /// The reply did not match the pending action.
    WrongAction,
    /// The original notification already completed.
    Complete,
}

impl std::fmt::Display for HtmlSelectNotificationReplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongAction => "reply does not match the pending Select notification action",
            Self::Complete => "the Select notification already completed",
        })
    }
}

impl std::error::Error for HtmlSelectNotificationReplyError {}

/// The ordered effects carried by one queued Select notification.
/// The owner retains the original target, task identity, and receipts and drives
/// this sequence only when that user-interaction task runs.
#[derive(Debug, Eq, PartialEq)]
pub struct HtmlSelectNotificationCursor {
    action: Option<HtmlSelectNotificationAction>,
}

impl HtmlSelectNotificationCursor {
    /// Returns the reached action, or `None` after the final dispatch returned.
    pub const fn action(&self) -> Option<HtmlSelectNotificationAction> {
        self.action
    }

    /// Accepts the actual result of the reached action. An unavailable or
    /// pending effect leaves the action pending without a reply.
    ///
    /// # Errors
    /// Rejects a mismatched reply or a reply after completion.
    pub fn accept(
        &mut self,
        reply: HtmlSelectNotificationReply,
    ) -> Result<(), HtmlSelectNotificationReplyError> {
        use HtmlSelectNotificationAction as Action;
        use HtmlSelectNotificationEvent::{Change, Input};
        use HtmlSelectNotificationReply as Reply;
        let action = self
            .action
            .ok_or(HtmlSelectNotificationReplyError::Complete)?;
        self.action = match (action, reply) {
            (Action::MarkUserValidity, Reply::UserValiditySet) => {
                Some(Action::UpdateSelectedcontent)
            }
            (Action::UpdateSelectedcontent, Reply::SelectedcontentUpdated) => {
                Some(Action::UpdateFallbackLabel)
            }
            (Action::UpdateFallbackLabel, Reply::FallbackLabelUpdated) => {
                Some(Action::Dispatch(Input))
            }
            (Action::Dispatch(Input), Reply::Dispatched(Input)) => Some(Action::Dispatch(Change)),
            (Action::Dispatch(Change), Reply::Dispatched(Change)) => None,
            _ => return Err(HtmlSelectNotificationReplyError::WrongAction),
        };
        Ok(())
    }
}

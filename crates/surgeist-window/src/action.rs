use crate::{Instant, registry::Id};

/// Handler result telling the event loop how to continue.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// Wait for the next host event without scheduling a draw.
    Wait,
    /// Request a draw for `Id` in the current scheduling opportunity.
    DrawNow(Id),
    /// Request a draw for `Id` on the next scheduling opportunity.
    DrawNext(Id),
    /// Request a draw for one window at an absolute monotonic time.
    DrawAt { id: Id, time: Instant },
    /// Begin the close-request lifecycle for the target window.
    CloseRequested(Id),
    /// Begin terminal event-loop exit processing.
    Exit,
    /// Apply nested actions in order using the callback action transaction rules.
    Batch(Vec<Action>),
}

impl Action {
    /// Returns the window targeted by this non-batch action, when it has one.
    #[must_use]
    pub(crate) const fn target(&self) -> Option<Id> {
        match self {
            Self::DrawNow(id)
            | Self::DrawNext(id)
            | Self::DrawAt { id, .. }
            | Self::CloseRequested(id) => Some(*id),
            Self::Wait | Self::Exit | Self::Batch(_) => None,
        }
    }

    /// Returns this action with every nested action for `id` removed.
    pub(crate) fn without_target(&self, id: Id) -> Option<Self> {
        match self {
            Self::DrawNow(target)
            | Self::DrawNext(target)
            | Self::DrawAt { id: target, .. }
            | Self::CloseRequested(target)
                if *target == id =>
            {
                None
            }
            Self::Batch(actions) => {
                let actions: Vec<_> = actions
                    .iter()
                    .filter_map(|action| action.without_target(id))
                    .collect();
                (!actions.is_empty()).then_some(Self::Batch(actions))
            }
            action => Some(action.clone()),
        }
    }
}

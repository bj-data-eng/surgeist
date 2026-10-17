//! Complete HTML877d focus-delegate and autofocus-delegate language protocols
//! (raw60113–60159), without identity, traversal or focus-service ownership.
//!
//! Adapters retain the actual target, selected root, current descendant, areas,
//! original trigger and nested called-algorithm custody. Missing facts make no
//! reply. Neither tabindex nor absent rendering services certify focusability.

/// Classification of the original focus target, not the current descendant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateTargetKind {
    Dialog,
    Other,
}
/// Only the branch classification of the original trigger. Other includes every
/// string except exact `click`. The adapter must still retain and forward the
/// actual original trigger to called resolution and nested delegation; this
/// classification is not a replacement string. Dialog supplies default `other`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusTriggerKind {
    Click,
    Other,
}
/// Role of the actual whereToLook local, retained by the adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateRoot {
    Target,
    ShadowRoot,
}
/// Actual shadow-host/root relation and delegates-focus state. When present,
/// the adapter retains the exact root observed with this reply.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateShadowRoot {
    Absent,
    DelegatesFocus,
    DoesNotDelegateFocus,
}
/// Role of the actual selected area, never its identity or a fabricated anchor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateCandidate {
    Descendant,
    ResolvedArea,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofocusDelegateOutcome {
    Null,
    Candidate(HtmlFocusDelegateCandidate),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateOutcome {
    Null,
    /// Return the actual area produced by the completed autofocus child.
    AutofocusDelegate,
    Candidate(HtmlFocusDelegateCandidate),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateAction {
    ReadShadowRoot,
    /// Complete the entire autofocus-delegate call for the selected actual root
    /// and original trigger before supplying its nullable result.
    RunAutofocusDelegate {
        root: HtmlFocusDelegateRoot,
    },
    /// Initialize a NEW ordinary-descendant preorder pass of the actual root,
    /// excluding the root itself. No implicit shadow, slot or template edges.
    BeginDescendants {
        root: HtmlFocusDelegateRoot,
    },
    /// Advance exactly once in that pass; retain the actual current descendant.
    /// False means actual exhaustion, never unavailable traversal evidence.
    NextDescendant,
    /// Read actual sequential focusability, not a tabindex sign heuristic.
    ReadSequentiallyFocusable,
    ReadFocusableArea,
    /// Fulfill called get-the-focusable-area for the actual current descendant
    /// and unchanged original trigger, retaining its actual nullable result.
    /// Nested delegation is a real called algorithm, not an implicit tree edge.
    GetFocusableArea,
    Complete(HtmlFocusDelegateOutcome),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateReply {
    ShadowRoot(HtmlFocusDelegateShadowRoot),
    AutofocusDelegatePresent(bool),
    DescendantPresent(bool),
    SequentiallyFocusable(bool),
    FocusableArea(bool),
    ResolvedAreaPresent(bool),
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlFocusDelegateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending focus delegate action",
            Self::Terminal => "focus delegate is already complete",
        })
    }
}
impl std::error::Error for HtmlFocusDelegateError {}

/// Constant-state complete language cursor. Actual ordinary preorder traversal
/// and its finite storage/work admission belong to the original caller. Copying
/// this cursor grants no execution, identity or replay authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFocusDelegateCursor {
    pending: HtmlFocusDelegateAction,
    target_kind: HtmlFocusDelegateTargetKind,
}
impl HtmlFocusDelegateCursor {
    pub const fn new(target_kind: HtmlFocusDelegateTargetKind) -> Self {
        Self {
            pending: HtmlFocusDelegateAction::ReadShadowRoot,
            target_kind,
        }
    }
    pub const fn action(&self) -> HtmlFocusDelegateAction {
        self.pending
    }
    /// Wrong-stage and terminal replies preserve every field of the cursor.
    pub fn reply(&mut self, reply: HtmlFocusDelegateReply) -> Result<(), HtmlFocusDelegateError> {
        use HtmlFocusDelegateAction as A;
        use HtmlFocusDelegateOutcome as O;
        use HtmlFocusDelegateReply as R;
        if matches!(self.pending, A::Complete(_)) {
            return Err(HtmlFocusDelegateError::Terminal);
        }
        let mut next = *self;
        next.pending = match (self.pending, reply) {
            (
                A::ReadShadowRoot,
                R::ShadowRoot(HtmlFocusDelegateShadowRoot::DoesNotDelegateFocus),
            ) => A::Complete(O::Null),
            (A::ReadShadowRoot, R::ShadowRoot(HtmlFocusDelegateShadowRoot::Absent)) => {
                A::RunAutofocusDelegate {
                    root: HtmlFocusDelegateRoot::Target,
                }
            }
            (A::ReadShadowRoot, R::ShadowRoot(HtmlFocusDelegateShadowRoot::DelegatesFocus)) => {
                A::RunAutofocusDelegate {
                    root: HtmlFocusDelegateRoot::ShadowRoot,
                }
            }
            (A::RunAutofocusDelegate { .. }, R::AutofocusDelegatePresent(true)) => {
                A::Complete(O::AutofocusDelegate)
            }
            (A::RunAutofocusDelegate { root }, R::AutofocusDelegatePresent(false)) => {
                A::BeginDescendants { root }
            }
            (A::BeginDescendants { .. }, R::Applied) => A::NextDescendant,
            (A::NextDescendant, R::DescendantPresent(false)) => A::Complete(O::Null),
            (A::NextDescendant, R::DescendantPresent(true)) => match next.target_kind {
                HtmlFocusDelegateTargetKind::Dialog => A::ReadSequentiallyFocusable,
                HtmlFocusDelegateTargetKind::Other => A::ReadFocusableArea,
            },
            (A::ReadSequentiallyFocusable, R::SequentiallyFocusable(true))
            | (A::ReadFocusableArea, R::FocusableArea(true)) => {
                A::Complete(O::Candidate(HtmlFocusDelegateCandidate::Descendant))
            }
            (A::ReadSequentiallyFocusable, R::SequentiallyFocusable(false))
            | (A::ReadFocusableArea, R::FocusableArea(false)) => A::GetFocusableArea,
            (A::GetFocusableArea, R::ResolvedAreaPresent(false)) => A::NextDescendant,
            (A::GetFocusableArea, R::ResolvedAreaPresent(true)) => {
                A::Complete(O::Candidate(HtmlFocusDelegateCandidate::ResolvedArea))
            }
            _ => return Err(HtmlFocusDelegateError::WrongReply),
        };
        *self = next;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofocusDelegateAction {
    /// Initialize ordinary descendants of this call's actual target/root in
    /// preorder, excluding that root itself, without implicit hosted edges.
    BeginDescendants,
    /// Advance the admitted walk once, retaining the actual current descendant.
    NextDescendant,
    /// Read actual qualified autofocus presence, using autofocus_presence for
    /// actual Elements. Traversal itself must not silently filter non-Elements.
    ReadAutofocus,
    ReadFocusableArea,
    /// Run real called resolution with the original actual trigger and retain
    /// its nullable area. Non-Element input-domain uncertainty is unavailable
    /// resolution evidence, not a synthetic null or a traversal filter.
    GetFocusableArea,
    /// Read actual click focusability of the selected actual area, which may
    /// differ from its DOM anchor and from the current descendant.
    ReadClickFocusable {
        candidate: HtmlFocusDelegateCandidate,
    },
    Complete(HtmlAutofocusDelegateOutcome),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofocusDelegateReply {
    DescendantPresent(bool),
    Autofocus(bool),
    FocusableArea(bool),
    ResolvedAreaPresent(bool),
    ClickFocusable(bool),
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofocusDelegateError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlAutofocusDelegateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending autofocus delegate action",
            Self::Terminal => "autofocus delegate is already complete",
        })
    }
}
impl std::error::Error for HtmlAutofocusDelegateError {}

/// Complete identity-free autofocus-delegate language cursor. Actual search root,
/// traversal, current descendant/selected area and original trigger remain owned
/// by the adapter. This cursor performs no hidden scan or recursive execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAutofocusDelegateCursor {
    pending: HtmlAutofocusDelegateAction,
    trigger_kind: HtmlFocusTriggerKind,
}
impl HtmlAutofocusDelegateCursor {
    pub const fn new(trigger_kind: HtmlFocusTriggerKind) -> Self {
        Self {
            pending: HtmlAutofocusDelegateAction::BeginDescendants,
            trigger_kind,
        }
    }
    pub const fn action(&self) -> HtmlAutofocusDelegateAction {
        self.pending
    }
    fn select_candidate(
        &self,
        candidate: HtmlFocusDelegateCandidate,
    ) -> HtmlAutofocusDelegateAction {
        if self.trigger_kind == HtmlFocusTriggerKind::Click {
            HtmlAutofocusDelegateAction::ReadClickFocusable { candidate }
        } else {
            HtmlAutofocusDelegateAction::Complete(HtmlAutofocusDelegateOutcome::Candidate(
                candidate,
            ))
        }
    }
    /// Wrong-stage and terminal replies preserve every field of the cursor.
    pub fn reply(
        &mut self,
        reply: HtmlAutofocusDelegateReply,
    ) -> Result<(), HtmlAutofocusDelegateError> {
        use HtmlAutofocusDelegateAction as A;
        use HtmlAutofocusDelegateOutcome as O;
        use HtmlAutofocusDelegateReply as R;
        if matches!(self.pending, A::Complete(_)) {
            return Err(HtmlAutofocusDelegateError::Terminal);
        }
        let mut next = *self;
        next.pending = match (self.pending, reply) {
            (A::BeginDescendants, R::Applied) => A::NextDescendant,
            (A::NextDescendant, R::DescendantPresent(false)) => A::Complete(O::Null),
            (A::NextDescendant, R::DescendantPresent(true)) => A::ReadAutofocus,
            (A::ReadAutofocus, R::Autofocus(false)) => A::NextDescendant,
            (A::ReadAutofocus, R::Autofocus(true)) => A::ReadFocusableArea,
            (A::ReadFocusableArea, R::FocusableArea(true)) => {
                next.select_candidate(HtmlFocusDelegateCandidate::Descendant)
            }
            (A::ReadFocusableArea, R::FocusableArea(false)) => A::GetFocusableArea,
            (A::GetFocusableArea, R::ResolvedAreaPresent(false)) => A::NextDescendant,
            (A::GetFocusableArea, R::ResolvedAreaPresent(true)) => {
                next.select_candidate(HtmlFocusDelegateCandidate::ResolvedArea)
            }
            (A::ReadClickFocusable { .. }, R::ClickFocusable(false)) => A::NextDescendant,
            (A::ReadClickFocusable { candidate }, R::ClickFocusable(true)) => {
                A::Complete(O::Candidate(candidate))
            }
            _ => return Err(HtmlAutofocusDelegateError::WrongReply),
        };
        *self = next;
        Ok(())
    }
}

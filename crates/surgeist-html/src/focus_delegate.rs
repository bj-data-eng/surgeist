//! Complete HTML877d focus-delegate and autofocus-delegate language protocols
//! (raw60113–60159), without identity, traversal or focus-service ownership.
//!
//! Adapters retain the actual target, selected root, current descendant, areas,
//! original trigger and nested called-algorithm custody. Missing facts make no
//! reply. Neither tabindex nor absent rendering services certify focusability.

/// Classification of the original focus target, not the current descendant.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusDelegateTargetKind {
    /// The focus target is a `dialog` element, so a descendant is chosen when it is
    /// sequentially focusable.
    Dialog,
    /// The focus target is not a `dialog` element, so a descendant is chosen when it
    /// is a focusable area or resolves to one.
    Other,
}
/// Only the branch classification of the original trigger. Other includes every
/// string except exact `click`. The adapter must still retain and forward the
/// actual original trigger to called resolution and nested delegation; this
/// classification is not a replacement string. Dialog supplies default `other`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusTriggerKind {
    /// The trigger is exactly the string `click`, so a candidate must also be click
    /// focusable.
    Click,
    /// The trigger is any other string, including the default `other`.
    Other,
}
/// Role of the actual whereToLook local, retained by the adapter.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusDelegateRoot {
    /// `whereToLook` is the focus target itself, which is not a shadow host.
    Target,
    /// `whereToLook` is the shadow root of the focus target, a shadow host that
    /// delegates focus.
    ShadowRoot,
}
/// Actual shadow-host/root relation and delegates-focus state. When present,
/// the adapter retains the exact root observed with this reply.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusDelegateShadowRoot {
    /// The focus target is not a shadow host.
    Absent,
    /// The focus target is a shadow host whose shadow root delegates focus.
    DelegatesFocus,
    /// The focus target is a shadow host whose shadow root does not delegate focus,
    /// so the delegate is null.
    DoesNotDelegateFocus,
}
/// Role of the actual selected area, never its identity or a fabricated anchor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusDelegateCandidate {
    /// The current descendant is itself the focusable area.
    Descendant,
    /// The area that getting the focusable area resolved for the current descendant.
    ResolvedArea,
}
/// The result of an autofocus delegate: null, or the selected area's role.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlAutofocusDelegateOutcome {
    /// No descendant with an `autofocus` attribute yielded a suitable area.
    Null,
    /// A descendant with an `autofocus` attribute yielded this area.
    Candidate(HtmlFocusDelegateCandidate),
}
/// The result of a focus delegate: null, an area from the autofocus delegate, or
/// the selected area's role.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusDelegateOutcome {
    /// No suitable focusable area exists below `whereToLook`, or the target is a
    /// shadow host that does not delegate focus.
    Null,
    /// Return the actual area produced by the completed autofocus child.
    AutofocusDelegate,
    /// The first suitable descendant in tree order yielded this area.
    Candidate(HtmlFocusDelegateCandidate),
}

/// The read or effect that the adapter must fulfill next for the focus delegate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateAction {
    /// Read whether the focus target is a shadow host and whether its shadow root
    /// delegates focus.
    ReadShadowRoot,
    /// Complete the entire autofocus-delegate call for the selected actual root
    /// and original trigger before supplying its nullable result.
    RunAutofocusDelegate {
        /// The root the autofocus delegate searches.
        root: HtmlFocusDelegateRoot,
    },
    /// Initialize a NEW ordinary-descendant preorder pass of the actual root,
    /// excluding the root itself. No implicit shadow, slot or template edges.
    BeginDescendants {
        /// The root whose descendants are walked.
        root: HtmlFocusDelegateRoot,
    },
    /// Advance exactly once in that pass; retain the actual current descendant.
    /// False means actual exhaustion, never unavailable traversal evidence.
    NextDescendant,
    /// Read actual sequential focusability, not a tabindex sign heuristic.
    ReadSequentiallyFocusable,
    /// Read whether the current descendant is itself a focusable area, for a focus
    /// target that is not a `dialog`.
    ReadFocusableArea,
    /// Fulfill called get-the-focusable-area for the actual current descendant
    /// and unchanged original trigger, retaining its actual nullable result.
    /// Nested delegation is a real called algorithm, not an implicit tree edge.
    GetFocusableArea,
    /// The focus delegate is finished with this result. No reply is accepted afterward.
    Complete(HtmlFocusDelegateOutcome),
}
/// Observations and acknowledgments for the focus delegate cursor. Each reply
/// answers exactly the pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusDelegateReply {
    /// The relation of the focus target to a shadow root.
    ShadowRoot(HtmlFocusDelegateShadowRoot),
    /// Whether the autofocus delegate for `whereToLook` is nonnull.
    AutofocusDelegatePresent(bool),
    /// Whether another descendant of `whereToLook` exists in tree order. `false`
    /// means actual exhaustion.
    DescendantPresent(bool),
    /// Whether the current descendant is sequentially focusable.
    SequentiallyFocusable(bool),
    /// Whether the current descendant is a focusable area.
    FocusableArea(bool),
    /// Whether getting the focusable area for the current descendant returned a
    /// nonnull area.
    ResolvedAreaPresent(bool),
    /// The pending preparation effect was fulfilled.
    Applied,
}
/// Why a focus delegate cursor refused a reply, queried through `is_*`
/// predicates. The cursor stays unchanged after a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFocusDelegateError {
    kind: HtmlFocusDelegateErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlFocusDelegateErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlFocusDelegateErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlFocusDelegateError {
        HtmlFocusDelegateError { kind: self }
    }
}

impl From<HtmlFocusDelegateErrorKind> for HtmlFocusDelegateError {
    fn from(kind: HtmlFocusDelegateErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlFocusDelegateError {
    /// Whether the cause is: reply does not match the pending focus delegate action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlFocusDelegateErrorKind::WrongReply)
    }
    /// Whether the cause is: focus delegate is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlFocusDelegateErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlFocusDelegateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlFocusDelegateErrorKind::WrongReply => {
                f.write_str("reply does not match the pending focus delegate action")
            }
            HtmlFocusDelegateErrorKind::Terminal => {
                f.write_str("focus delegate is already complete")
            }
        }
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
    /// Starts the focus delegate of HTML 6.6.4 (Processing model). The target kind
    /// selects the descendant test that applies to a `dialog` element.
    pub const fn new(target_kind: HtmlFocusDelegateTargetKind) -> Self {
        Self {
            pending: HtmlFocusDelegateAction::ReadShadowRoot,
            target_kind,
        }
    }
    /// Returns the read or effect that must be fulfilled before the next reply.
    pub const fn action(&self) -> HtmlFocusDelegateAction {
        self.pending
    }
    /// Wrong-stage and terminal replies preserve every field of the cursor.
    pub fn reply(&mut self, reply: HtmlFocusDelegateReply) -> Result<(), HtmlFocusDelegateError> {
        use HtmlFocusDelegateAction as A;
        use HtmlFocusDelegateOutcome as O;
        use HtmlFocusDelegateReply as R;
        if matches!(self.pending, A::Complete(_)) {
            return Err(HtmlFocusDelegateErrorKind::Terminal.error());
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
            _ => return Err(HtmlFocusDelegateErrorKind::WrongReply.error()),
        };
        *self = next;
        Ok(())
    }
}

/// The read or effect that the adapter must fulfill next for the autofocus
/// delegate.
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
    /// Read whether the descendant with an `autofocus` attribute is itself a
    /// focusable area.
    ReadFocusableArea,
    /// Run real called resolution with the original actual trigger and retain
    /// its nullable area. Non-Element input-domain uncertainty is unavailable
    /// resolution evidence, not a synthetic null or a traversal filter.
    GetFocusableArea,
    /// Read actual click focusability of the selected actual area, which may
    /// differ from its DOM anchor and from the current descendant.
    ReadClickFocusable {
        /// Which area was selected, for the click focusability read.
        candidate: HtmlFocusDelegateCandidate,
    },
    /// The autofocus delegate is finished with this result. No reply is accepted
    /// afterward.
    Complete(HtmlAutofocusDelegateOutcome),
}
/// Observations and acknowledgments for the autofocus delegate cursor. Each reply
/// answers exactly the pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAutofocusDelegateReply {
    /// Whether another descendant exists in tree order.
    DescendantPresent(bool),
    /// Whether the current descendant has an `autofocus` content attribute.
    Autofocus(bool),
    /// Whether the descendant is a focusable area.
    FocusableArea(bool),
    /// Whether getting the focusable area for the descendant returned a nonnull area.
    ResolvedAreaPresent(bool),
    /// Whether the selected area is click focusable.
    ClickFocusable(bool),
    /// The pending traversal effect was fulfilled.
    Applied,
}
/// Why an autofocus delegate cursor refused a reply, queried through `is_*`
/// predicates. The cursor stays unchanged after a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAutofocusDelegateError {
    kind: HtmlAutofocusDelegateErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlAutofocusDelegateErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlAutofocusDelegateErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlAutofocusDelegateError {
        HtmlAutofocusDelegateError { kind: self }
    }
}

impl From<HtmlAutofocusDelegateErrorKind> for HtmlAutofocusDelegateError {
    fn from(kind: HtmlAutofocusDelegateErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlAutofocusDelegateError {
    /// Whether the cause is: reply does not match the pending autofocus delegate action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlAutofocusDelegateErrorKind::WrongReply)
    }
    /// Whether the cause is: autofocus delegate is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlAutofocusDelegateErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlAutofocusDelegateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlAutofocusDelegateErrorKind::WrongReply => {
                f.write_str("reply does not match the pending autofocus delegate action")
            }
            HtmlAutofocusDelegateErrorKind::Terminal => {
                f.write_str("autofocus delegate is already complete")
            }
        }
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
    /// Starts the autofocus delegate of HTML 6.6.4 (Processing model). Only a `click`
    /// trigger requires the selected area to be click focusable.
    pub const fn new(trigger_kind: HtmlFocusTriggerKind) -> Self {
        Self {
            pending: HtmlAutofocusDelegateAction::BeginDescendants,
            trigger_kind,
        }
    }
    /// Returns the read or effect that must be fulfilled before the next reply.
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
            return Err(HtmlAutofocusDelegateErrorKind::Terminal.error());
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
            _ => return Err(HtmlAutofocusDelegateErrorKind::WrongReply.error()),
        };
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_focus_delegate_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlFocusDelegateError) -> bool;
        let cases: Vec<(HtmlFocusDelegateError, &str, Predicate)> = vec![
            (
                HtmlFocusDelegateErrorKind::WrongReply.error(),
                "reply does not match the pending focus delegate action",
                HtmlFocusDelegateError::is_wrong_reply,
            ),
            (
                HtmlFocusDelegateErrorKind::Terminal.error(),
                "focus delegate is already complete",
                HtmlFocusDelegateError::is_terminal,
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
    fn html_autofocus_delegate_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlAutofocusDelegateError) -> bool;
        let cases: Vec<(HtmlAutofocusDelegateError, &str, Predicate)> = vec![
            (
                HtmlAutofocusDelegateErrorKind::WrongReply.error(),
                "reply does not match the pending autofocus delegate action",
                HtmlAutofocusDelegateError::is_wrong_reply,
            ),
            (
                HtmlAutofocusDelegateErrorKind::Terminal.error(),
                "autofocus delegate is already complete",
                HtmlAutofocusDelegateError::is_terminal,
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

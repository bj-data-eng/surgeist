//! HTML877d #reset-the-form-owner decisions, without identities or a live tree.

/// A source-qualified association-reset cause for one actual associated element.
///
/// Adapters establish native or custom association, event applicability and the
/// affected subjects. Attribute causes mean the applicable no-namespace `form`
/// attribute was set/changed/removed. ID causes mean the source-required ID
/// change or ID-bearing insertion/removal/moving event has actually occurred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFormOwnerResetTrigger {
    /// The element was inserted.
    Insertion {
        /// Whether the parser inserted the element, in which case the reset is skipped.
        parser_inserted: bool,
    },
    /// The element was removed or moved.
    RemovingOrMoving {
        /// Whether the element has a form owner now. Without one nothing needs resetting.
        has_owner: bool,
    },
    /// The `form` attribute of the element was set, changed or removed.
    FormAttributeChanged {
        /// Whether the element is listed. Only a listed element reacts to its `form` attribute.
        listed: bool,
    },
    /// The ID of an element in the tree changed, or an element with an ID was inserted, removed or
    /// moved.
    IdEvent {
        /// Whether the element is listed. Only a listed element with a `form` attribute reacts.
        listed: bool,
    },
}

/// The remaining local condition, without performing its qualified observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFormOwnerResetDecision {
    /// The cause does not reset the form owner, and no further observation is needed.
    Skip,
    /// The cause resets the form owner (HTML877d #reset-the-form-owner) with no further condition.
    Reset,
    /// Compare the actual nonnull owner's ordinary tree with the subject's.
    ResetIfOwnerInDifferentTree,
    /// Read actual applicable form-attribute presence, including an empty value.
    ResetIfFormAttributePresent,
}

/// Classifies a reset cause without probing identity, topology or attributes.
///
/// Skip requires no later observation: absent owners do not need an owner-tree
/// comparison, and nonlisted ID recipients do not need an attribute read. The
/// adapter retains source hook ordering, affected-subject traversal, ID matching,
/// actual ordinary-root comparison and effects through the reset cursor below.
pub const fn form_owner_reset_trigger(
    trigger: HtmlFormOwnerResetTrigger,
) -> HtmlFormOwnerResetDecision {
    use HtmlFormOwnerResetDecision as D;
    use HtmlFormOwnerResetTrigger as T;
    match trigger {
        T::Insertion {
            parser_inserted: true,
        }
        | T::RemovingOrMoving { has_owner: false }
        | T::FormAttributeChanged { listed: false }
        | T::IdEvent { listed: false } => D::Skip,
        T::Insertion {
            parser_inserted: false,
        }
        | T::FormAttributeChanged { listed: true } => D::Reset,
        T::RemovingOrMoving { has_owner: true } => D::ResetIfOwnerInDifferentTree,
        T::IdEvent { listed: true } => D::ResetIfFormAttributePresent,
    }
}

/// The source lookup after the previous owner has been cleared.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFormOwnerLookup {
    /// Look up the first element in the subject's tree, in tree order, whose ID equals the `form`
    /// attribute's value, and associate the subject only if that element is a form.
    FirstId,
    /// Associate the subject with its nearest ancestor form, if it has one.
    NearestAncestor,
}

/// An explicit listed form attribute uses first-ID lookup only when connected.
///
/// `explicit_listed_form_attribute` means actual listed applicability and
/// attribute presence, including a present empty value. Connectedness is
/// irrelevant otherwise: the adapter must not query unavailable connectedness
/// or present an ignored argument as established disconnection.
pub const fn form_owner_lookup(
    explicit_listed_form_attribute: bool,
    connected: bool,
) -> HtmlFormOwnerLookup {
    if explicit_listed_form_attribute && connected {
        HtmlFormOwnerLookup::FirstId
    } else {
        HtmlFormOwnerLookup::NearestAncestor
    }
}

/// The next admitted operation of an actual form-owner reset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFormOwnerResetAction {
    /// Unset the element's parser inserted flag. It is the first step of every reset.
    ClearParserInserted,
    /// Read whether the listed element has a `form` content attribute, including an empty value.
    ReadFormAttributePresence,
    /// Read whether the current form owner is the nearest ancestor form after the tree change,
    /// which lets the reset return early without clearing the owner.
    ReadCurrentOwnerIsNearestAncestor,
    /// Set the form owner to null.
    ClearOwner,
    /// Apply intrinsic effects against the actual intermediate owner state.
    /// Radio adapters use the shared FormOwner trigger and their real group.
    /// This is not event dispatch or a custom-element execution capability.
    ApplyOwnerChangeEffects,
    /// Read whether the element is connected, which decides between the first-ID and ancestor
    /// lookups.
    ReadConnectedness,
    /// Find the first exact-ID element in the subject's ordinary tree. A first
    /// non-form blocks any later form; no match has no ancestor fallback.
    FindFirstIdMatch,
    /// Find the nearest ancestor form of the element.
    FindNearestAncestor,
    /// Install the actual qualified candidate retained by the preceding lookup.
    AssociateFoundOwner,
    /// The reset is done; no further reply is accepted.
    Complete,
}

/// Established observations or acknowledgement of the pending operation.
///
/// Missing evidence is never a false reply. A lookup's FormFound(false) means
/// a completed negative result (including a first-ID non-form). The adapter
/// retains and validates the actual found identity before acknowledging its
/// association; the cursor stores no identity or candidate cache.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFormOwnerResetReply {
    /// The pending clear, effect or association action was performed.
    Applied,
    /// Whether the element has a `form` content attribute, answering `ReadFormAttributePresence`.
    FormAttributePresent(bool),
    /// Whether the current owner is the nearest ancestor form, answering
    /// `ReadCurrentOwnerIsNearestAncestor`.
    CurrentOwnerIsNearestAncestor(bool),
    /// Whether the element is connected, answering `ReadConnectedness`.
    Connected(bool),
    /// Whether the lookup found a form to associate with, answering `FindFirstIdMatch` or
    /// `FindNearestAncestor`. A negative result is complete and leaves the owner null.
    FormFound(bool),
}

/// A reply which does not match the current pending operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFormOwnerResetCursorError {
    kind: HtmlFormOwnerResetCursorErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlFormOwnerResetCursorErrorKind {
    Reply,
}

impl HtmlFormOwnerResetCursorErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlFormOwnerResetCursorError {
        HtmlFormOwnerResetCursorError { kind: self }
    }
}

impl From<HtmlFormOwnerResetCursorErrorKind> for HtmlFormOwnerResetCursorError {
    fn from(kind: HtmlFormOwnerResetCursorErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlFormOwnerResetCursorError {
    /// Whether the cause is: form-owner reply does not match its pending action.
    pub fn is_reply(&self) -> bool {
        matches!(self.kind, HtmlFormOwnerResetCursorErrorKind::Reply)
    }
}

impl std::fmt::Display for HtmlFormOwnerResetCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlFormOwnerResetCursorErrorKind::Reply => {
                f.write_str("form-owner reply does not match its pending action")
            }
        }
    }
}

impl std::error::Error for HtmlFormOwnerResetCursorError {}

#[derive(Clone, Copy, Debug)]
enum Phase {
    ClearParserInserted,
    Attribute,
    KeepOwner,
    ClearOwner,
    EffectsAfterClear,
    Connection,
    FirstId,
    Nearest,
    Associate,
    EffectsAfterAssociate,
    Complete,
}

/// Allocation-free, constant-state ordering for one selected form-owner reset.
///
/// The caller first establishes source applicability, actual listed membership
/// and current owner presence. It owns qualified identities, topology, lookup,
/// radio effects, admission, cancellation and atomic publication. Every action
/// must be admitted and fulfilled before replying. A stopped adapter publishes
/// no completed reset merely because it can still inspect this cursor.
///
/// The early nearest-owner probe is only a retention test. If retention fails,
/// association lookup runs afresh after clearing the owner and its effects.
/// Insertion/removal/moving/attribute/ID trigger selection remains with the
/// consuming adapter; this is not the event-bearing form-wide reset algorithm.
#[derive(Clone, Copy, Debug)]
pub struct HtmlFormOwnerResetCursor {
    listed: bool,
    had_owner: bool,
    explicit: bool,
    phase: Phase,
}
impl HtmlFormOwnerResetCursor {
    /// Starts a reset for an element that is `listed` or not and that currently has an owner or not
    /// (`has_current_owner`). The first action is `ClearParserInserted`.
    pub const fn new(listed: bool, has_current_owner: bool) -> Self {
        Self {
            listed,
            had_owner: has_current_owner,
            explicit: false,
            phase: Phase::ClearParserInserted,
        }
    }

    /// The operation the adapter must perform next, or `Complete` once the reset is done.
    pub fn action(&self) -> HtmlFormOwnerResetAction {
        use HtmlFormOwnerResetAction as A;
        match self.phase {
            Phase::ClearParserInserted => A::ClearParserInserted,
            Phase::Attribute => A::ReadFormAttributePresence,
            Phase::KeepOwner => A::ReadCurrentOwnerIsNearestAncestor,
            Phase::ClearOwner => A::ClearOwner,
            Phase::EffectsAfterClear | Phase::EffectsAfterAssociate => A::ApplyOwnerChangeEffects,
            Phase::Connection => A::ReadConnectedness,
            Phase::FirstId => A::FindFirstIdMatch,
            Phase::Nearest => A::FindNearestAncestor,
            Phase::Associate => A::AssociateFoundOwner,
            Phase::Complete => A::Complete,
        }
    }

    /// A wrong reply, including any reply after Complete, changes no state.
    pub fn reply(
        &mut self,
        reply: HtmlFormOwnerResetReply,
    ) -> Result<(), HtmlFormOwnerResetCursorError> {
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }

    fn after_attribute(&mut self) {
        self.phase = if self.had_owner && !self.explicit {
            Phase::KeepOwner
        } else {
            Phase::ClearOwner
        };
    }

    fn begin_lookup(&mut self) {
        self.phase = if self.explicit {
            Phase::Connection
        } else {
            Phase::Nearest
        };
    }

    fn advance(
        &mut self,
        reply: HtmlFormOwnerResetReply,
    ) -> Result<(), HtmlFormOwnerResetCursorError> {
        use HtmlFormOwnerResetReply as R;
        match (self.phase, reply) {
            (Phase::ClearParserInserted, R::Applied) => {
                if self.listed {
                    self.phase = Phase::Attribute;
                } else {
                    self.after_attribute();
                }
            }
            (Phase::Attribute, R::FormAttributePresent(present)) => {
                self.explicit = present;
                self.after_attribute();
            }
            (Phase::KeepOwner, R::CurrentOwnerIsNearestAncestor(true)) => {
                self.phase = Phase::Complete;
            }
            (Phase::KeepOwner, R::CurrentOwnerIsNearestAncestor(false)) => {
                self.phase = Phase::ClearOwner;
            }
            (Phase::ClearOwner, R::Applied) => {
                if self.had_owner {
                    self.phase = Phase::EffectsAfterClear;
                } else {
                    self.begin_lookup();
                }
            }
            (Phase::EffectsAfterClear, R::Applied) => self.begin_lookup(),
            (Phase::Connection, R::Connected(connected)) => {
                self.phase = match form_owner_lookup(self.explicit, connected) {
                    HtmlFormOwnerLookup::FirstId => Phase::FirstId,
                    HtmlFormOwnerLookup::NearestAncestor => Phase::Nearest,
                };
            }
            (Phase::FirstId | Phase::Nearest, R::FormFound(found)) => {
                self.phase = if found {
                    Phase::Associate
                } else {
                    Phase::Complete
                };
            }
            (Phase::Associate, R::Applied) => self.phase = Phase::EffectsAfterAssociate,
            (Phase::EffectsAfterAssociate, R::Applied) => self.phase = Phase::Complete,
            _ => return Err(HtmlFormOwnerResetCursorErrorKind::Reply.error()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_form_owner_reset_cursor_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlFormOwnerResetCursorError) -> bool;
        let cases: Vec<(HtmlFormOwnerResetCursorError, &str, Predicate)> = vec![(
            HtmlFormOwnerResetCursorErrorKind::Reply.error(),
            "form-owner reply does not match its pending action",
            HtmlFormOwnerResetCursorError::is_reply,
        )];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}

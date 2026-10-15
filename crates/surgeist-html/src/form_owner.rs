//! HTML877d #reset-the-form-owner decisions, without identities or a live tree.

/// The source lookup after the previous owner has been cleared.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFormOwnerLookup {
    FirstId,
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
    ClearParserInserted,
    ReadFormAttributePresence,
    ReadCurrentOwnerIsNearestAncestor,
    ClearOwner,
    /// Apply intrinsic effects against the actual intermediate owner state.
    /// Radio adapters use the shared FormOwner trigger and their real group.
    /// This is not event dispatch or a custom-element execution capability.
    ApplyOwnerChangeEffects,
    ReadConnectedness,
    /// Find the first exact-ID element in the subject's ordinary tree. A first
    /// non-form blocks any later form; no match has no ancestor fallback.
    FindFirstIdMatch,
    FindNearestAncestor,
    /// Install the actual qualified candidate retained by the preceding lookup.
    AssociateFoundOwner,
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
    Applied,
    FormAttributePresent(bool),
    CurrentOwnerIsNearestAncestor(bool),
    Connected(bool),
    FormFound(bool),
}

/// A reply which does not match the current pending operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFormOwnerResetCursorError {
    Reply,
}
impl std::fmt::Display for HtmlFormOwnerResetCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("form-owner reply does not match its pending action")
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
    pub const fn new(listed: bool, has_current_owner: bool) -> Self {
        Self {
            listed,
            had_owner: has_current_owner,
            explicit: false,
            phase: Phase::ClearParserInserted,
        }
    }

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
            _ => return Err(HtmlFormOwnerResetCursorError::Reply),
        }
        Ok(())
    }
}

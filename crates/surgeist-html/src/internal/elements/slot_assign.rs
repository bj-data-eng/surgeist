//! HTML877d #dom-slot-assign, over caller-owned identity-bearing storage.
//!
//! Manual intent is separate from DOM's stored assignment state and fresh
//! find-slot/flattened/flat-tree queries. Step5 visits only the target's root.

/// One source read or effect, admitted and performed by the consuming owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSlotAssignAction {
    /// Step 1: set the manual slot assignment of the target's previous manually assigned node at
    /// this ordinal to null.
    ClearPreviousManualInverse {
        /// Zero-based position in the target's previous manually assigned nodes.
        previous_ordinal: usize,
    },
    /// Step 2: create the empty ordered set that will replace the target's manually assigned nodes.
    BeginPendingManualSet,
    /// Step 3.1, first half: read whether the incoming node at this ordinal has a manual slot
    /// assignment referring to a slot; reply `ManualAssignmentPresent`.
    ReadIncomingManualAssignment {
        /// Zero-based position in the raw incoming nodes, duplicates included.
        ordinal: usize,
    },
    /// Step 3.1, second half: remove the incoming node at this ordinal from the manually assigned
    /// nodes of the slot its manual assignment referred to.
    RemoveIncomingFromReferredManualSet {
        /// Zero-based position in the raw incoming nodes, duplicates included.
        ordinal: usize,
    },
    /// Step 3.2: set the incoming node's manual slot assignment to the target slot.
    SetIncomingManualAssignmentToTarget {
        /// Zero-based position in the raw incoming nodes, duplicates included.
        ordinal: usize,
    },
    /// Step 3.3: append the incoming node at this ordinal to the pending ordered set. The ordered
    /// set append does nothing when the set already contains the node, so the consumer answers
    /// that identity question itself and replies `Applied` either way.
    AppendIncomingToPendingSet {
        /// Zero-based position in the raw incoming nodes, duplicates included.
        ordinal: usize,
    },
    /// Step 4: set the target's manually assigned nodes to the pending ordered set.
    ReplaceTargetManualSet,
    /// DOM's assign-slottables-for-a-tree operation for this target's root only.
    /// Do not refresh former roots or infer a globally consistent inverse.
    AssignSlottablesForTargetRoot,
    /// Every step has been performed; no further reply is accepted.
    Complete,
}

/// Actual acknowledged effects or established identity/slot-pointer reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSlotAssignReply {
    /// The pending effect was performed, with no data to report.
    Applied,
    /// From the current raw input's actual manual pointer, after prior effects.
    /// A present pointer is retained by the adapter for the next remove action.
    ManualAssignmentPresent(bool),
}

/// A reply that does not match the pending slot-assign action.
///
/// The cause is private; query it with [`is_reply`](Self::is_reply). A refused reply leaves the
/// cursor unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSlotAssignCursorError {
    kind: HtmlSlotAssignCursorErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlSlotAssignCursorErrorKind {
    Reply,
}

impl HtmlSlotAssignCursorErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlSlotAssignCursorError {
        HtmlSlotAssignCursorError { kind: self }
    }
}

impl From<HtmlSlotAssignCursorErrorKind> for HtmlSlotAssignCursorError {
    fn from(kind: HtmlSlotAssignCursorErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlSlotAssignCursorError {
    /// Whether the cause is: slot assign reply does not match its pending action.
    pub fn is_reply(&self) -> bool {
        matches!(self.kind, HtmlSlotAssignCursorErrorKind::Reply)
    }
}

impl std::fmt::Display for HtmlSlotAssignCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlSlotAssignCursorErrorKind::Reply => {
                f.write_str("slot assign reply does not match its pending action")
            }
        }
    }
}

impl std::error::Error for HtmlSlotAssignCursorError {}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Clear(usize),
    Begin,
    Read(usize),
    Remove(usize),
    Set(usize),
    Append(usize),
    Replace,
    AssignRoot,
    Complete,
}

/// Constant-state HTML slot.assign decisions over actual complete extents.
///
/// The owner supplies the target's previous manual ordered-set extent and the
/// raw incoming extent, including duplicates. Unknown state is not count0.
/// Incoming ordinals remain stable through completion; previous ordinals are
/// used only before incoming transfers can modify the target's old set.
///
/// The adapter retains and qualifies all IDs, pointers, lists and temporary
/// storage. It admits each action before its operand reads/writes and replies
/// only after success. Missing facts, cancellation, allocation and atomic
/// rollback belong to that owner. This cursor allocates and traverses no tree.
/// It asks no per-pair question: one call makes at most one action per previous node, at most
/// four per raw incoming node and a constant number of others. The ordered-set append of a node
/// the pending set already holds is the consumer's identity check within
/// `AppendIncomingToPendingSet`; a consumer without an identity map admits and charges its own
/// scan of the pending set before it answers.
#[derive(Clone, Copy, Debug)]
pub struct HtmlSlotAssignCursor {
    previous_count: usize,
    incoming_count: usize,
    phase: Phase,
}
impl HtmlSlotAssignCursor {
    /// Starts `slot.assign()` for a target with `previous_manual_count` manually assigned nodes
    /// and `raw_incoming_count` arguments, duplicates included.
    ///
    /// The counts are the complete extents, not estimates. The first action clears the previous
    /// nodes' inverse, or begins the pending set when there are none.
    pub const fn new(previous_manual_count: usize, raw_incoming_count: usize) -> Self {
        Self {
            previous_count: previous_manual_count,
            incoming_count: raw_incoming_count,
            phase: if previous_manual_count == 0 {
                Phase::Begin
            } else {
                Phase::Clear(0)
            },
        }
    }

    /// The action the consuming owner must perform next; `Complete` once every step is done.
    pub fn action(&self) -> HtmlSlotAssignAction {
        use HtmlSlotAssignAction as A;
        match self.phase {
            Phase::Clear(previous_ordinal) => A::ClearPreviousManualInverse { previous_ordinal },
            Phase::Begin => A::BeginPendingManualSet,
            Phase::Read(ordinal) => A::ReadIncomingManualAssignment { ordinal },
            Phase::Remove(ordinal) => A::RemoveIncomingFromReferredManualSet { ordinal },
            Phase::Set(ordinal) => A::SetIncomingManualAssignmentToTarget { ordinal },
            Phase::Append(ordinal) => A::AppendIncomingToPendingSet { ordinal },
            Phase::Replace => A::ReplaceTargetManualSet,
            Phase::AssignRoot => A::AssignSlottablesForTargetRoot,
            Phase::Complete => A::Complete,
        }
    }

    /// Wrong replies, including replies after completion, preserve every field.
    pub fn reply(&mut self, reply: HtmlSlotAssignReply) -> Result<(), HtmlSlotAssignCursorError> {
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }

    fn next_incoming(&mut self, ordinal: usize) {
        // Every incoming phase proves ordinal < incoming_count. This addition
        // remains representable even when the complete extent is usize::MAX.
        let next = ordinal + 1;
        self.phase = if next < self.incoming_count {
            Phase::Read(next)
        } else {
            Phase::Replace
        };
    }

    fn advance(&mut self, reply: HtmlSlotAssignReply) -> Result<(), HtmlSlotAssignCursorError> {
        use HtmlSlotAssignReply as R;
        match (self.phase, reply) {
            (Phase::Clear(ordinal), R::Applied) => {
                // ordinal < previous_count, including an extent of usize::MAX.
                let next = ordinal + 1;
                self.phase = if next < self.previous_count {
                    Phase::Clear(next)
                } else {
                    Phase::Begin
                };
            }
            (Phase::Begin, R::Applied) => {
                self.phase = if self.incoming_count == 0 {
                    Phase::Replace
                } else {
                    Phase::Read(0)
                };
            }
            (Phase::Read(ordinal), R::ManualAssignmentPresent(present)) => {
                self.phase = if present {
                    Phase::Remove(ordinal)
                } else {
                    Phase::Set(ordinal)
                };
            }
            (Phase::Remove(ordinal), R::Applied) => self.phase = Phase::Set(ordinal),
            (Phase::Set(ordinal), R::Applied) => self.phase = Phase::Append(ordinal),
            (Phase::Append(ordinal), R::Applied) => self.next_incoming(ordinal),
            (Phase::Replace, R::Applied) => self.phase = Phase::AssignRoot,
            (Phase::AssignRoot, R::Applied) => self.phase = Phase::Complete,
            _ => return Err(HtmlSlotAssignCursorErrorKind::Reply.error()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod extent_tests {
    use super::*;

    #[test]
    fn final_ordinals_at_maximum_extent_do_not_overflow_or_skip_the_final_effects() {
        use HtmlSlotAssignAction as A;
        use HtmlSlotAssignReply as R;
        let mut cursor = HtmlSlotAssignCursor {
            previous_count: usize::MAX,
            incoming_count: usize::MAX,
            phase: Phase::Clear(usize::MAX - 1),
        };
        cursor.reply(R::Applied).unwrap();
        assert_eq!(cursor.action(), A::BeginPendingManualSet);
        cursor.phase = Phase::Set(usize::MAX - 1);
        cursor.reply(R::Applied).unwrap();
        assert_eq!(
            cursor.action(),
            A::AppendIncomingToPendingSet {
                ordinal: usize::MAX - 1
            }
        );
        cursor.reply(R::Applied).unwrap();
        assert_eq!(cursor.action(), A::ReplaceTargetManualSet);
        cursor.reply(R::Applied).unwrap();
        assert_eq!(cursor.action(), A::AssignSlottablesForTargetRoot);
        cursor.reply(R::Applied).unwrap();
        assert_eq!(cursor.action(), A::Complete);
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_slot_assign_cursor_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlSlotAssignCursorError) -> bool;
        let cases: Vec<(HtmlSlotAssignCursorError, &str, Predicate)> = vec![(
            HtmlSlotAssignCursorErrorKind::Reply.error(),
            "slot assign reply does not match its pending action",
            HtmlSlotAssignCursorError::is_reply,
        )];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}

//! HTML877d #dom-slot-assign, over caller-owned identity-bearing storage.
//!
//! Manual intent is separate from DOM's stored assignment state and fresh
//! find-slot/flattened/flat-tree queries. Step5 visits only the target's root.

/// One source read or effect, admitted and performed by the consuming owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSlotAssignAction {
    ClearPreviousManualInverse {
        previous_ordinal: usize,
    },
    BeginPendingManualSet,
    ReadIncomingManualAssignment {
        ordinal: usize,
    },
    RemoveIncomingFromReferredManualSet {
        ordinal: usize,
    },
    SetIncomingManualAssignmentToTarget {
        ordinal: usize,
    },
    CompareIncoming {
        ordinal: usize,
        previous_ordinal: usize,
    },
    AppendIncomingToPendingSet {
        ordinal: usize,
    },
    ReplaceTargetManualSet,
    /// DOM's assign-slottables-for-a-tree operation for this target's root only.
    /// Do not refresh former roots or infer a globally consistent inverse.
    AssignSlottablesForTargetRoot,
    Complete,
}

/// Actual acknowledged effects or established identity/slot-pointer reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSlotAssignReply {
    Applied,
    /// From the current raw input's actual manual pointer, after prior effects.
    /// A present pointer is retained by the adapter for the next remove action.
    ManualAssignmentPresent(bool),
    /// Exact identity equality of the two requested raw input ordinals.
    SameNode(bool),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSlotAssignCursorError {
    Reply,
}
impl std::fmt::Display for HtmlSlotAssignCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("slot assign reply does not match its pending action")
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
    Compare { ordinal: usize, previous: usize },
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
/// Pairwise ordered-set comparisons are at worst quadratic in the admitted raw
/// extent; each comparison is separately visible before its actual read.
#[derive(Clone, Copy, Debug)]
pub struct HtmlSlotAssignCursor {
    previous_count: usize,
    incoming_count: usize,
    phase: Phase,
}
impl HtmlSlotAssignCursor {
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

    pub fn action(&self) -> HtmlSlotAssignAction {
        use HtmlSlotAssignAction as A;
        match self.phase {
            Phase::Clear(previous_ordinal) => A::ClearPreviousManualInverse { previous_ordinal },
            Phase::Begin => A::BeginPendingManualSet,
            Phase::Read(ordinal) => A::ReadIncomingManualAssignment { ordinal },
            Phase::Remove(ordinal) => A::RemoveIncomingFromReferredManualSet { ordinal },
            Phase::Set(ordinal) => A::SetIncomingManualAssignmentToTarget { ordinal },
            Phase::Compare { ordinal, previous } => A::CompareIncoming {
                ordinal,
                previous_ordinal: previous,
            },
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
            (Phase::Set(0), R::Applied) => self.phase = Phase::Append(0),
            (Phase::Set(ordinal), R::Applied) => {
                self.phase = Phase::Compare {
                    ordinal,
                    previous: 0,
                };
            }
            (Phase::Compare { ordinal, .. }, R::SameNode(true)) => {
                self.next_incoming(ordinal);
            }
            (Phase::Compare { ordinal, previous }, R::SameNode(false)) => {
                // previous < ordinal; even ordinal == usize::MAX -1 is safe.
                let next = previous + 1;
                self.phase = if next < ordinal {
                    Phase::Compare {
                        ordinal,
                        previous: next,
                    }
                } else {
                    Phase::Append(ordinal)
                };
            }
            (Phase::Append(ordinal), R::Applied) => self.next_incoming(ordinal),
            (Phase::Replace, R::Applied) => self.phase = Phase::AssignRoot,
            (Phase::AssignRoot, R::Applied) => self.phase = Phase::Complete,
            _ => return Err(HtmlSlotAssignCursorError::Reply),
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
        cursor.phase = Phase::Compare {
            ordinal: usize::MAX - 1,
            previous: usize::MAX - 2,
        };
        cursor.reply(R::SameNode(false)).unwrap();
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
        cursor.phase = Phase::Compare {
            ordinal: usize::MAX - 1,
            previous: 0,
        };
        cursor.reply(R::SameNode(true)).unwrap();
        assert_eq!(cursor.action(), A::ReplaceTargetManualSet);
    }
}

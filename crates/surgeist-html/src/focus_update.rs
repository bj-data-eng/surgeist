//! Complete focus-update source protocol, WHATWG HTML revision
//! 877d6146487f15b15bc8dd0e3c625063a37b456a, retained raw60227–60298.
//!
//! The tracked focus-algorithm editorial-gap selections establish fresh owning
//! Document locals for new area entries and exclusive event-target classification.
//! Original adapters own actual chain banks, iterations, entries/areas/anchors,
//! new target, Document/Window locals, work/admission and authentic callbacks.
//! This protocol neither constructs chains nor implements dispatch or native focus.

use crate::focus_event::HtmlFocusEventName;

/// Which chain of the focus update steps an entry belongs to.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusUpdateChain {
    /// The old chain, whose entries are blurred.
    Old,
    /// The new chain, whose entries are focused.
    New,
}
/// Classification of the actual ENTRY, independently of its DOM anchor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusUpdateEntryKind {
    /// The entry is an element.
    Element,
    /// The entry is a Document object.
    Document,
    /// The entry is neither an element nor a Document, such as an area element's
    /// shape, a scrollable region or a viewport. No focus or blur event is fired for
    /// it.
    Other,
}
/// The input data that the `change` event conditions inspect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusUpdateInputData {
    /// The element's value.
    Value,
    /// The element's list of selected files.
    SelectedFiles,
}
/// What the focus or blur event is fired at.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusUpdateEventTarget {
    /// The entry itself, an element.
    EntryElement,
    /// The relevant global object of the entry, a Document.
    EntryDocumentGlobal,
}
/// The related target of the focus or blur event.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlFocusUpdateRelatedTarget {
    /// There is no related target.
    Null,
    /// The last entry of the opposite trimmed chain, an element. It is used only for
    /// the last entry of this chain when both are elements.
    OppositeChainLastElement,
}

/// The read, effect or called algorithm that the adapter must fulfill next for
/// the focus update steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateAction {
    /// Read whether the old chain has a last entry.
    ReadOldTailPresent,
    /// Read whether the new chain has a last entry.
    ReadNewTailPresent,
    /// Compare actual identity of both present tails, not their DOM anchors.
    ReadTailIdentityEqual,
    /// Pop the last entry from the old chain.
    PopOldTail,
    /// Pop the last entry from the new chain.
    PopNewTail,
    /// Begin the trimmed old chain in forward order. Iteration does not erase
    /// entries or tails needed later for actual related-target selection.
    BeginOldEntries,
    /// Advance to the next entry of the trimmed old chain, in order.
    NextOldEntry,
    /// Read whether the current old entry is an input element.
    ReadOldEntryIsInput,
    /// Read whether the `change` event applies to the element.
    ReadChangeEventApplies,
    /// Read whether the element has a defined activation behavior.
    ReadHasActivationBehavior,
    /// Read whether the user changed the given data while the control was focused.
    ReadInputUserChangedDataWhileFocused {
        /// The data to inspect.
        data: HtmlFocusUpdateInputData,
    },
    /// Read whether the change made while the control was focused is uncommitted.
    ReadInputChangeUncommitted {
        /// The data to inspect.
        data: HtmlFocusUpdateInputData,
    },
    /// Read whether the data differs from what it was when the control was first
    /// focused.
    ReadInputDiffersFromFirstFocus {
        /// The data to inspect.
        data: HtmlFocusUpdateInputData,
    },
    /// Set the entry's user validity to true.
    SetUserValidityTrue,
    /// Fire change at the original actual old input entry using Event, not
    /// FocusEvent. True bubbles and false default cancelable/composed are exact.
    /// Resume only after authentic dispatch and incurred callback work complete.
    FireChange {
        /// The event's `bubbles` attribute, which is `true` for this event.
        bubbles: bool,
        /// The event's `cancelable` attribute, which is `false` for this event.
        cancelable: bool,
        /// The event's `composed` flag, which is `false` for this event.
        composed: bool,
    },
    /// Fresh exclusive Element / else Document / else Other classification of
    /// the actual entry, including after change-event callbacks.
    ReadEntryKind {
        /// The chain that the entry belongs to.
        chain: HtmlFocusUpdateChain,
    },
    /// Retain this actual Document ENTRY's relevant Global as event target.
    CaptureEntryRelevantGlobal {
        /// The chain that the Document entry belongs to.
        chain: HtmlFocusUpdateChain,
    },
    /// Actual list-last membership in the corresponding TRIMMED chain. For a
    /// reverse iteration this is not the last item visited by the iterator.
    ReadEntryIsLast {
        /// The trimmed chain to test the entry against.
        chain: HtmlFocusUpdateChain,
    },
    /// Inspect the opposite trimmed chain's actual last entry, or known absence.
    /// Retain that exact Element only when classified Element; no anchor proxy.
    ReadOppositeTailKind {
        /// The chain whose opposite trimmed chain is inspected.
        chain: HtmlFocusUpdateChain,
    },
    /// Run the complete fire-a-focus-event child with these actual operand roles.
    /// It must finish genuinely before FocusEventCompleted. Null event targets
    /// skip this call; an area's Element anchor is not an event-target substitute.
    FireFocusEvent {
        /// The name of the event to fire.
        name: HtmlFocusEventName,
        /// What the event is fired at.
        target: HtmlFocusUpdateEventTarget,
        /// The related target of the event.
        related: HtmlFocusUpdateRelatedTarget,
    },
    /// Fulfill actual applicable platform conventions for the original new target.
    ApplyPlatformConventions,
    /// Begin the trimmed new chain in reverse order.
    BeginNewEntriesReverse,
    /// Advance to the next entry of the trimmed new chain, in reverse order.
    NextNewEntry,
    /// Read whether the current new entry is a focusable area.
    ReadNewEntryFocusableArea,
    /// For EACH reached area entry, capture its actual anchor's owning Document
    /// AFTER all earlier callbacks. An Element anchor supplies its CURRENT node
    /// Document; a Document anchor supplies itself. Retain this exact Document
    /// through comparison, original-Window flag write and designation.
    CaptureEntryDocument,
    /// Read whether the focused area of the captured Document is already this entry.
    ReadEntryMatchesDesignation,
    /// Set literal true on the captured Document's actual relevant Global's
    /// original Window Navigation API, not a newly sampled Document/Window.
    SetWindowFocusChangedDuringNavigationTrue,
    /// Designate this exact actual area in the same captured Document. No other
    /// entry, DOM anchor, viewport or previous-focus slot may substitute for it.
    DesignateEntryFocusedArea,
    /// The focus update steps are finished. No reply is accepted afterward.
    Complete,
}
/// Observations and acknowledgments for the focus update cursor. Each reply
/// answers exactly the pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateReply {
    /// Whether the old chain has a last entry.
    OldTailPresent(bool),
    /// Whether the new chain has a last entry.
    NewTailPresent(bool),
    /// Whether the last entries of both chains are the same.
    TailIdentityEqual(bool),
    /// Whether the old chain has another entry to visit.
    OldEntryPresent(bool),
    /// Whether the current old entry is an input element.
    OldEntryIsInput(bool),
    /// Whether the `change` event applies to the element.
    ChangeEventApplies(bool),
    /// Whether the element has a defined activation behavior.
    HasActivationBehavior(bool),
    /// The answer to the user-changed-data read.
    InputUserChangedDataWhileFocused {
        /// The data that was inspected.
        data: HtmlFocusUpdateInputData,
        /// Whether the user changed it while the control was focused.
        value: bool,
    },
    /// The answer to the uncommitted-change read.
    InputChangeUncommitted {
        /// The data that was inspected.
        data: HtmlFocusUpdateInputData,
        /// Whether the change is uncommitted.
        value: bool,
    },
    /// The answer to the differs-from-first-focus read.
    InputDiffersFromFirstFocus {
        /// The data that was inspected.
        data: HtmlFocusUpdateInputData,
        /// Whether the data differs from what it was at first focus.
        value: bool,
    },
    /// The classification of an entry.
    EntryKind {
        /// The chain that the entry belongs to.
        chain: HtmlFocusUpdateChain,
        /// Whether the entry is an element, a Document or another object.
        kind: HtmlFocusUpdateEntryKind,
    },
    /// Whether the entry is the last entry of its trimmed chain.
    EntryIsLast {
        /// The chain that the entry belongs to.
        chain: HtmlFocusUpdateChain,
        /// Whether the entry is the last entry.
        value: bool,
    },
    /// The classification of the last entry of the opposite trimmed chain.
    OppositeTailKind {
        /// The chain whose opposite chain was inspected.
        chain: HtmlFocusUpdateChain,
        /// The kind of the opposite chain's last entry, or `None` when that chain is
        /// empty.
        kind: Option<HtmlFocusUpdateEntryKind>,
    },
    /// Whether the new chain has another entry to visit.
    NewEntryPresent(bool),
    /// Whether the current new entry is a focusable area.
    NewEntryFocusableArea(bool),
    /// Whether the captured Document's focused area is already this entry.
    EntryMatchesDesignation(bool),
    /// The `change` event was dispatched, with its callbacks complete.
    ChangeDispatched,
    /// The fire-a-focus-event child finished.
    FocusEventCompleted,
    /// The pending effect was fulfilled.
    Applied,
}
/// Why a focus update cursor refused a reply, queried through `is_*` predicates.
/// The cursor stays unchanged after a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFocusUpdateError {
    kind: HtmlFocusUpdateErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlFocusUpdateErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlFocusUpdateErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlFocusUpdateError {
        HtmlFocusUpdateError { kind: self }
    }
}

impl From<HtmlFocusUpdateErrorKind> for HtmlFocusUpdateError {
    fn from(kind: HtmlFocusUpdateErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlFocusUpdateError {
    /// Whether the cause is: reply does not match the pending focus-update action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlFocusUpdateErrorKind::WrongReply)
    }
    /// Whether the cause is: focus update is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlFocusUpdateErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlFocusUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlFocusUpdateErrorKind::WrongReply => {
                f.write_str("reply does not match the pending focus-update action")
            }
            HtmlFocusUpdateErrorKind::Terminal => f.write_str("focus update is already complete"),
        }
    }
}

impl std::error::Error for HtmlFocusUpdateError {}

/// Constant-state, non-Copy complete source-body cursor. All suffix trimming,
/// ordered loops, input conjunction/OR and event selection are HTML-owned; the
/// adapter supplies the distinct underlying facts rather than should-fire flags.
/// Missing facts and incomplete callbacks provide no reply. Dropping performs no
/// designation, restoration, event dispatch or other external effect.
#[derive(Debug, Eq, PartialEq)]
pub struct HtmlFocusUpdateCursor {
    pending: HtmlFocusUpdateAction,
    entry_kind: HtmlFocusUpdateEntryKind,
}
impl HtmlFocusUpdateCursor {
    /// Starts the focus update steps of HTML 6.6.4 (Processing model) with the trim
    /// of the last entries of both chains.
    pub const fn new() -> Self {
        Self {
            pending: HtmlFocusUpdateAction::ReadOldTailPresent,
            entry_kind: HtmlFocusUpdateEntryKind::Other,
        }
    }
    /// Returns the read, effect or called algorithm that must be fulfilled before the
    /// next reply.
    pub const fn action(&self) -> HtmlFocusUpdateAction {
        self.pending
    }
    const fn next_entry(chain: HtmlFocusUpdateChain) -> HtmlFocusUpdateAction {
        match chain {
            HtmlFocusUpdateChain::Old => HtmlFocusUpdateAction::NextOldEntry,
            HtmlFocusUpdateChain::New => HtmlFocusUpdateAction::NextNewEntry,
        }
    }
    const fn after_failed_data(data: HtmlFocusUpdateInputData) -> HtmlFocusUpdateAction {
        match data {
            HtmlFocusUpdateInputData::Value => {
                HtmlFocusUpdateAction::ReadInputUserChangedDataWhileFocused {
                    data: HtmlFocusUpdateInputData::SelectedFiles,
                }
            }
            HtmlFocusUpdateInputData::SelectedFiles => HtmlFocusUpdateAction::ReadEntryKind {
                chain: HtmlFocusUpdateChain::Old,
            },
        }
    }
    const fn event_or_next(
        chain: HtmlFocusUpdateChain,
        kind: HtmlFocusUpdateEntryKind,
        related: HtmlFocusUpdateRelatedTarget,
    ) -> HtmlFocusUpdateAction {
        let target = match kind {
            HtmlFocusUpdateEntryKind::Element => HtmlFocusUpdateEventTarget::EntryElement,
            HtmlFocusUpdateEntryKind::Document => HtmlFocusUpdateEventTarget::EntryDocumentGlobal,
            HtmlFocusUpdateEntryKind::Other => return Self::next_entry(chain),
        };
        HtmlFocusUpdateAction::FireFocusEvent {
            name: match chain {
                HtmlFocusUpdateChain::Old => HtmlFocusEventName::Blur,
                HtmlFocusUpdateChain::New => HtmlFocusEventName::Focus,
            },
            target,
            related,
        }
    }
    /// Validate the entire reply before assigning any field. Wrong stage, data
    /// branch, chain or terminal replies preserve the complete exact cursor.
    pub fn reply(&mut self, reply: HtmlFocusUpdateReply) -> Result<(), HtmlFocusUpdateError> {
        use HtmlFocusUpdateAction as A;
        use HtmlFocusUpdateChain as C;
        use HtmlFocusUpdateEntryKind as K;
        use HtmlFocusUpdateInputData as D;
        use HtmlFocusUpdateRelatedTarget as T;
        use HtmlFocusUpdateReply as R;
        if self.pending == A::Complete {
            return Err(HtmlFocusUpdateErrorKind::Terminal.error());
        }
        let mut kind = self.entry_kind;
        let pending = match (self.pending, reply) {
            (A::ReadOldTailPresent, R::OldTailPresent(false)) => A::BeginOldEntries,
            (A::ReadOldTailPresent, R::OldTailPresent(true)) => A::ReadNewTailPresent,
            (A::ReadNewTailPresent, R::NewTailPresent(false)) => A::BeginOldEntries,
            (A::ReadNewTailPresent, R::NewTailPresent(true)) => A::ReadTailIdentityEqual,
            (A::ReadTailIdentityEqual, R::TailIdentityEqual(true)) => A::PopOldTail,
            (A::ReadTailIdentityEqual, R::TailIdentityEqual(false)) => A::BeginOldEntries,
            (A::PopOldTail, R::Applied) => A::PopNewTail,
            (A::PopNewTail, R::Applied) => A::ReadOldTailPresent,
            (A::BeginOldEntries, R::Applied) => A::NextOldEntry,
            (A::NextOldEntry, R::OldEntryPresent(true)) => A::ReadOldEntryIsInput,
            (A::NextOldEntry, R::OldEntryPresent(false)) => A::ApplyPlatformConventions,
            (A::ReadOldEntryIsInput, R::OldEntryIsInput(false)) => {
                A::ReadEntryKind { chain: C::Old }
            }
            (A::ReadOldEntryIsInput, R::OldEntryIsInput(true)) => A::ReadChangeEventApplies,
            (A::ReadChangeEventApplies, R::ChangeEventApplies(false)) => {
                A::ReadEntryKind { chain: C::Old }
            }
            (A::ReadChangeEventApplies, R::ChangeEventApplies(true)) => {
                A::ReadHasActivationBehavior
            }
            (A::ReadHasActivationBehavior, R::HasActivationBehavior(true)) => {
                A::ReadEntryKind { chain: C::Old }
            }
            (A::ReadHasActivationBehavior, R::HasActivationBehavior(false)) => {
                A::ReadInputUserChangedDataWhileFocused { data: D::Value }
            }
            (
                A::ReadInputUserChangedDataWhileFocused { data },
                R::InputUserChangedDataWhileFocused {
                    data: replied,
                    value,
                },
            ) if data == replied => {
                if value {
                    A::ReadInputChangeUncommitted { data }
                } else {
                    Self::after_failed_data(data)
                }
            }
            (
                A::ReadInputChangeUncommitted { data },
                R::InputChangeUncommitted {
                    data: replied,
                    value,
                },
            ) if data == replied => {
                if value {
                    A::ReadInputDiffersFromFirstFocus { data }
                } else {
                    Self::after_failed_data(data)
                }
            }
            (
                A::ReadInputDiffersFromFirstFocus { data },
                R::InputDiffersFromFirstFocus {
                    data: replied,
                    value,
                },
            ) if data == replied => {
                if value {
                    A::SetUserValidityTrue
                } else {
                    Self::after_failed_data(data)
                }
            }
            (A::SetUserValidityTrue, R::Applied) => A::FireChange {
                bubbles: true,
                cancelable: false,
                composed: false,
            },
            (A::FireChange { .. }, R::ChangeDispatched) => A::ReadEntryKind { chain: C::Old },
            (
                A::ReadEntryKind { chain },
                R::EntryKind {
                    chain: replied,
                    kind: observed,
                },
            ) if chain == replied => {
                kind = observed;
                if kind == K::Document {
                    A::CaptureEntryRelevantGlobal { chain }
                } else {
                    A::ReadEntryIsLast { chain }
                }
            }
            (A::CaptureEntryRelevantGlobal { chain }, R::Applied) => A::ReadEntryIsLast { chain },
            (
                A::ReadEntryIsLast { chain },
                R::EntryIsLast {
                    chain: replied,
                    value,
                },
            ) if chain == replied => {
                if value && kind == K::Element {
                    A::ReadOppositeTailKind { chain }
                } else {
                    Self::event_or_next(chain, kind, T::Null)
                }
            }
            (
                A::ReadOppositeTailKind { chain },
                R::OppositeTailKind {
                    chain: replied,
                    kind: opposite,
                },
            ) if chain == replied => Self::event_or_next(
                chain,
                kind,
                if opposite == Some(K::Element) {
                    T::OppositeChainLastElement
                } else {
                    T::Null
                },
            ),
            (A::FireFocusEvent { name, .. }, R::FocusEventCompleted) => {
                Self::next_entry(match name {
                    HtmlFocusEventName::Blur => C::Old,
                    HtmlFocusEventName::Focus => C::New,
                })
            }
            (A::ApplyPlatformConventions, R::Applied) => A::BeginNewEntriesReverse,
            (A::BeginNewEntriesReverse, R::Applied) => A::NextNewEntry,
            (A::NextNewEntry, R::NewEntryPresent(false)) => A::Complete,
            (A::NextNewEntry, R::NewEntryPresent(true)) => A::ReadNewEntryFocusableArea,
            (A::ReadNewEntryFocusableArea, R::NewEntryFocusableArea(false)) => {
                A::ReadEntryKind { chain: C::New }
            }
            (A::ReadNewEntryFocusableArea, R::NewEntryFocusableArea(true)) => {
                A::CaptureEntryDocument
            }
            (A::CaptureEntryDocument, R::Applied) => A::ReadEntryMatchesDesignation,
            (A::ReadEntryMatchesDesignation, R::EntryMatchesDesignation(true)) => {
                A::ReadEntryKind { chain: C::New }
            }
            (A::ReadEntryMatchesDesignation, R::EntryMatchesDesignation(false)) => {
                A::SetWindowFocusChangedDuringNavigationTrue
            }
            (A::SetWindowFocusChangedDuringNavigationTrue, R::Applied) => {
                A::DesignateEntryFocusedArea
            }
            (A::DesignateEntryFocusedArea, R::Applied) => A::ReadEntryKind { chain: C::New },
            _ => return Err(HtmlFocusUpdateErrorKind::WrongReply.error()),
        };
        self.pending = pending;
        self.entry_kind = kind;
        Ok(())
    }
}
impl Default for HtmlFocusUpdateCursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_focus_update_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlFocusUpdateError) -> bool;
        let cases: Vec<(HtmlFocusUpdateError, &str, Predicate)> = vec![
            (
                HtmlFocusUpdateErrorKind::WrongReply.error(),
                "reply does not match the pending focus-update action",
                HtmlFocusUpdateError::is_wrong_reply,
            ),
            (
                HtmlFocusUpdateErrorKind::Terminal.error(),
                "focus update is already complete",
                HtmlFocusUpdateError::is_terminal,
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

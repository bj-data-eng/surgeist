//! Complete focus-update source protocol, WHATWG HTML revision
//! 877d6146487f15b15bc8dd0e3c625063a37b456a, retained raw60227–60298.
//!
//! The tracked focus-algorithm editorial-gap selections establish fresh owning
//! Document locals for new area entries and exclusive event-target classification.
//! Original adapters own actual chain banks, iterations, entries/areas/anchors,
//! new target, Document/Window locals, work/admission and authentic callbacks.
//! This protocol neither constructs chains nor implements dispatch or native focus.

use crate::focus_event::HtmlFocusEventName;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateChain {
    Old,
    New,
}
/// Classification of the actual ENTRY, independently of its DOM anchor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateEntryKind {
    Element,
    Document,
    Other,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateInputData {
    Value,
    SelectedFiles,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateEventTarget {
    EntryElement,
    EntryDocumentGlobal,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateRelatedTarget {
    Null,
    OppositeChainLastElement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateAction {
    ReadOldTailPresent,
    ReadNewTailPresent,
    /// Compare actual identity of both present tails, not their DOM anchors.
    ReadTailIdentityEqual,
    PopOldTail,
    PopNewTail,
    /// Begin the trimmed old chain in forward order. Iteration does not erase
    /// entries or tails needed later for actual related-target selection.
    BeginOldEntries,
    NextOldEntry,
    ReadOldEntryIsInput,
    ReadChangeEventApplies,
    ReadHasActivationBehavior,
    ReadInputUserChangedDataWhileFocused {
        data: HtmlFocusUpdateInputData,
    },
    ReadInputChangeUncommitted {
        data: HtmlFocusUpdateInputData,
    },
    ReadInputDiffersFromFirstFocus {
        data: HtmlFocusUpdateInputData,
    },
    SetUserValidityTrue,
    /// Fire change at the original actual old input entry using Event, not
    /// FocusEvent. True bubbles and false default cancelable/composed are exact.
    /// Resume only after authentic dispatch and incurred callback work complete.
    FireChange {
        bubbles: bool,
        cancelable: bool,
        composed: bool,
    },
    /// Fresh exclusive Element / else Document / else Other classification of
    /// the actual entry, including after change-event callbacks.
    ReadEntryKind {
        chain: HtmlFocusUpdateChain,
    },
    /// Retain this actual Document ENTRY's relevant Global as event target.
    CaptureEntryRelevantGlobal {
        chain: HtmlFocusUpdateChain,
    },
    /// Actual list-last membership in the corresponding TRIMMED chain. For a
    /// reverse iteration this is not the last item visited by the iterator.
    ReadEntryIsLast {
        chain: HtmlFocusUpdateChain,
    },
    /// Inspect the opposite trimmed chain's actual last entry, or known absence.
    /// Retain that exact Element only when classified Element; no anchor proxy.
    ReadOppositeTailKind {
        chain: HtmlFocusUpdateChain,
    },
    /// Run the complete fire-a-focus-event child with these actual operand roles.
    /// It must finish genuinely before FocusEventCompleted. Null event targets
    /// skip this call; an area's Element anchor is not an event-target substitute.
    FireFocusEvent {
        name: HtmlFocusEventName,
        target: HtmlFocusUpdateEventTarget,
        related: HtmlFocusUpdateRelatedTarget,
    },
    /// Fulfill actual applicable platform conventions for the original new target.
    ApplyPlatformConventions,
    BeginNewEntriesReverse,
    NextNewEntry,
    ReadNewEntryFocusableArea,
    /// For EACH reached area entry, capture its actual anchor's owning Document
    /// AFTER all earlier callbacks. An Element anchor supplies its CURRENT node
    /// Document; a Document anchor supplies itself. Retain this exact Document
    /// through comparison, original-Window flag write and designation.
    CaptureEntryDocument,
    ReadEntryMatchesDesignation,
    /// Set literal true on the captured Document's actual relevant Global's
    /// original Window Navigation API, not a newly sampled Document/Window.
    SetWindowFocusChangedDuringNavigationTrue,
    /// Designate this exact actual area in the same captured Document. No other
    /// entry, DOM anchor, viewport or previous-focus slot may substitute for it.
    DesignateEntryFocusedArea,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateReply {
    OldTailPresent(bool),
    NewTailPresent(bool),
    TailIdentityEqual(bool),
    OldEntryPresent(bool),
    OldEntryIsInput(bool),
    ChangeEventApplies(bool),
    HasActivationBehavior(bool),
    InputUserChangedDataWhileFocused {
        data: HtmlFocusUpdateInputData,
        value: bool,
    },
    InputChangeUncommitted {
        data: HtmlFocusUpdateInputData,
        value: bool,
    },
    InputDiffersFromFirstFocus {
        data: HtmlFocusUpdateInputData,
        value: bool,
    },
    EntryKind {
        chain: HtmlFocusUpdateChain,
        kind: HtmlFocusUpdateEntryKind,
    },
    EntryIsLast {
        chain: HtmlFocusUpdateChain,
        value: bool,
    },
    OppositeTailKind {
        chain: HtmlFocusUpdateChain,
        kind: Option<HtmlFocusUpdateEntryKind>,
    },
    NewEntryPresent(bool),
    NewEntryFocusableArea(bool),
    EntryMatchesDesignation(bool),
    ChangeDispatched,
    FocusEventCompleted,
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusUpdateError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlFocusUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending focus-update action",
            Self::Terminal => "focus update is already complete",
        })
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
    pub const fn new() -> Self {
        Self {
            pending: HtmlFocusUpdateAction::ReadOldTailPresent,
            entry_kind: HtmlFocusUpdateEntryKind::Other,
        }
    }
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
            return Err(HtmlFocusUpdateError::Terminal);
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
            _ => return Err(HtmlFocusUpdateError::WrongReply),
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

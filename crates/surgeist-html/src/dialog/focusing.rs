//! HTML877d Dialog focusing steps (raw46762–46782).
//!
//! This language producer owns ordering and control selection. Actual identities,
//! focus services, navigable/origin observations and autofocus state remain with
//! their existing owners. An unavailable fact is never a false observation.

/// The selected control's source role. The adapter retains the actual delegate
/// identity returned by FindFocusDelegate; this value carries no Node identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogFocusControl {
    Subject,
    Delegate,
}

/// A source read or effect requiring actual fulfillment before a reply.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogFocusingAction {
    /// Fulfill the allow-focus steps for the subject's actual node Document.
    ReadAllowFocus,
    ReadAutofocus,
    /// Find the actual focus delegate of the subject, retaining it when nonnull.
    FindFocusDelegate,
    /// Run the called focusing steps for the selected actual control, with no
    /// fallback target or focus trigger supplied by this Dialog algorithm.
    RunFocusingSteps {
        control: HtmlDialogFocusControl,
    },
    /// After focusing returns, retain the selected control's node navigable's
    /// top-level traversable's actual active Document as topDocument.
    CaptureControlTopDocument {
        control: HtmlDialogFocusControl,
    },
    /// Compare the selected control's then-current node Document origin with
    /// the retained topDocument origin, using actual same-origin semantics.
    ReadSameOrigin {
        control: HtmlDialogFocusControl,
    },
    /// Empty the retained topDocument's actual autofocus candidates.
    EmptyAutofocusCandidates,
    /// Set the retained topDocument's autofocus processed flag to literal true.
    SetAutofocusProcessed,
    Complete,
}

/// Actual observations and fulfilled-effect acknowledgments. Only Applied
/// acknowledges completed called focusing or autofocus writes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogFocusingReply {
    AllowFocus(bool),
    Autofocus(bool),
    FocusDelegatePresent(bool),
    SameOrigin(bool),
    Applied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogFocusingError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlDialogFocusingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending Dialog focusing action",
            Self::Terminal => "Dialog focusing is already complete",
        })
    }
}
impl std::error::Error for HtmlDialogFocusingError {}

/// Constant-state, identity-free complete Dialog focusing language protocol.
/// The outer Dialog cursor remains at RunDialogFocusingSteps until this child
/// reaches Complete through actual fulfilled actions. Copies grant no execution
/// or replay authority; the adapter owns admission and original invocation custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDialogFocusingCursor {
    pending: HtmlDialogFocusingAction,
    control: HtmlDialogFocusControl,
}
impl HtmlDialogFocusingCursor {
    pub const fn new() -> Self {
        Self {
            pending: HtmlDialogFocusingAction::ReadAllowFocus,
            control: HtmlDialogFocusControl::Subject,
        }
    }
    pub const fn action(&self) -> HtmlDialogFocusingAction {
        self.pending
    }
    /// Wrong-stage and terminal replies preserve the entire original cursor.
    pub fn reply(&mut self, reply: HtmlDialogFocusingReply) -> Result<(), HtmlDialogFocusingError> {
        use HtmlDialogFocusingAction as A;
        use HtmlDialogFocusingReply as R;
        if self.pending == A::Complete {
            return Err(HtmlDialogFocusingError::Terminal);
        }
        let mut next = *self;
        next.pending = match (self.pending, reply) {
            (A::ReadAllowFocus, R::AllowFocus(false)) => A::Complete,
            (A::ReadAllowFocus, R::AllowFocus(true)) => A::ReadAutofocus,
            (A::ReadAutofocus, R::Autofocus(true)) => A::RunFocusingSteps {
                control: next.control,
            },
            (A::ReadAutofocus, R::Autofocus(false)) => A::FindFocusDelegate,
            (A::FindFocusDelegate, R::FocusDelegatePresent(present)) => {
                next.control = if present {
                    HtmlDialogFocusControl::Delegate
                } else {
                    HtmlDialogFocusControl::Subject
                };
                A::RunFocusingSteps {
                    control: next.control,
                }
            }
            (A::RunFocusingSteps { .. }, R::Applied) => A::CaptureControlTopDocument {
                control: next.control,
            },
            (A::CaptureControlTopDocument { .. }, R::Applied) => A::ReadSameOrigin {
                control: next.control,
            },
            (A::ReadSameOrigin { .. }, R::SameOrigin(false)) => A::Complete,
            (A::ReadSameOrigin { .. }, R::SameOrigin(true)) => A::EmptyAutofocusCandidates,
            (A::EmptyAutofocusCandidates, R::Applied) => A::SetAutofocusProcessed,
            (A::SetAutofocusProcessed, R::Applied) => A::Complete,
            _ => return Err(HtmlDialogFocusingError::WrongReply),
        };
        *self = next;
        Ok(())
    }
}
impl Default for HtmlDialogFocusingCursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Qualified facts about the actual Dialog subject only, never a delegate,
/// descendant, arbitrary focusable area, or renderer's guessed geometry.
/// None means unavailable; false is an actual negative observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDialogSubjectFocusFacts {
    /// Actual inert attribute interpretation on the subject itself. Ancestor
    /// inertness is insufficient: a modal Dialog escapes its ancestors' inertness.
    pub own_inert: Option<bool>,
    pub is_document_element: Option<bool>,
    /// Whether the subject has a shadow root whose delegates-focus flag is true.
    pub has_delegating_focus_root: Option<bool>,
}

/// Only the called focusing invocation's no-change proof, not completion of
/// Dialog focusing: control topDocument/origin/autofocus steps still remain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDialogSubjectFocusingDecision {
    NoChange,
    NeedsActualFocusing,
}

/// Narrow HTML877d no-change proof for focusing the actual Dialog subject.
///
/// Own inertness excludes the subject's ordinary focusable area, subwidgets and
/// its OWN scrollable regions: each table row requires a non-inert element and
/// anchors the area to that element (raw59650–59720). If the UA nevertheless
/// provides a subject area, its same inert anchor returns at raw60180. Otherwise
/// get-the-focusable-area returns null: Dialog is neither Area, navigable nor
/// navigable container; its own scroll regions cannot qualify; the facts exclude
/// Document-element viewport and delegated-root branches (raw60083–60110).
/// Called focusing supplies no fallback and therefore returns (raw60169–60176).
/// This proves no focus effects without asserting rendering or UA focusability.
/// Unknown or nonqualifying facts require actual focusing service evidence.
pub const fn dialog_subject_focusing_decision(
    facts: HtmlDialogSubjectFocusFacts,
) -> HtmlDialogSubjectFocusingDecision {
    match (
        facts.own_inert,
        facts.is_document_element,
        facts.has_delegating_focus_root,
    ) {
        (Some(true), Some(false), Some(false)) => HtmlDialogSubjectFocusingDecision::NoChange,
        _ => HtmlDialogSubjectFocusingDecision::NeedsActualFocusing,
    }
}

//! Ordered designMode setter protocol from WHATWG HTML revision
//! 877d6146487f15b15bc8dd0e3c625063a37b456a, retained raw61117–61130.
//!
//! The adapter owns the invoking Document, its current active association and
//! exact Range identity, document element, mutation authority and genuine called
//! focusing continuation. No identity, mutation or focus fact is supplied here.
//! The explicit NoActiveRange branch adopts the selected #1243 correction for
//! the pinned setter's undefined reset of a null active range.

/// ASCII-insensitive designMode input interpretation, without trimming.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDesignModeInput {
    On,
    Off,
    Invalid,
}
impl HtmlDesignModeInput {
    /// Inspect at most three UTF-16 code units with no allocation. Other lengths
    /// are invalid without inspecting their contents. Non-ASCII units, including
    /// lone surrogates, cannot match either keyword.
    pub const fn from_utf16(value: &[u16]) -> Self {
        match value {
            [o, n] if matches!(*o, 0x4f | 0x6f) && matches!(*n, 0x4e | 0x6e) => Self::On,
            [o, f, last_f]
                if matches!(*o, 0x4f | 0x6f)
                    && matches!(*f, 0x46 | 0x66)
                    && matches!(*last_f, 0x46 | 0x66) =>
            {
                Self::Off
            }
            _ => Self::Invalid,
        }
    }
}

/// Source reads, writes and genuine called-algorithm obligations. All reads use
/// current actual state at the reached stage, rather than an initial snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDesignModeAction {
    ReadEnabled,
    SetEnabled {
        enabled: bool,
    },
    /// Capture the current active association under its actual owner. Missing
    /// selection support is not evidence of a null active range.
    ReadActiveRange,
    /// Reset BOTH boundaries of the exact captured current active Range to
    /// (invoking Document, 0), preserving its identity and association. Revalidate
    /// authority and identity at fulfillment; never create or substitute a Range.
    ResetActiveRange,
    /// Capture this Document's actual current document element, if any, AFTER
    /// range reset or the explicit no-active-range disposition.
    ReadDocumentElement,
    /// Run HtmlFocusingCursor::new(HtmlFocusingFallback::Omitted) for that exact
    /// element, with the trigger omitted. Retain the genuine child and its
    /// actual operands until Complete; no Dialog predicate or unavailable fact
    /// can stand in for this call. Reentry must not restore an earlier flag.
    RunFocusingSteps,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDesignModeReply {
    Enabled(bool),
    /// Authoritatively known null: no reset and no Range creation; still proceed
    /// to the Document's document-element/focusing tail.
    NoActiveRange,
    /// Actual nonnull association captured; retain that exact Range identity.
    ActiveRange,
    DocumentElementPresent(bool),
    /// The genuine focusing child reached its own Complete action.
    FocusingStepsCompleted,
    /// The exact pending flag write or both-boundary reset actually completed.
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDesignModeError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlDesignModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending designMode action",
            Self::Terminal => "designMode setter is already complete",
        })
    }
}
impl std::error::Error for HtmlDesignModeError {}

/// Allocation-free constant-state language cursor. Actual identities, Source
/// publication/reentry, finite work and child custody belong to the adapter.
/// Unavailable state supplies no successful reply. Copy grants no execution or
/// replay authority. There is no HTML-class, fully-active or no-BC guard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDesignModeCursor {
    pending: HtmlDesignModeAction,
}
impl HtmlDesignModeCursor {
    pub const fn new(input: HtmlDesignModeInput) -> Self {
        Self {
            pending: match input {
                HtmlDesignModeInput::On => HtmlDesignModeAction::ReadEnabled,
                HtmlDesignModeInput::Off => HtmlDesignModeAction::SetEnabled { enabled: false },
                HtmlDesignModeInput::Invalid => HtmlDesignModeAction::Complete,
            },
        }
    }
    pub const fn action(&self) -> HtmlDesignModeAction {
        self.pending
    }
    /// Wrong-stage and terminal replies preserve the entire exact cursor.
    pub fn reply(&mut self, reply: HtmlDesignModeReply) -> Result<(), HtmlDesignModeError> {
        use HtmlDesignModeAction as A;
        use HtmlDesignModeReply as R;
        if self.pending == A::Complete {
            return Err(HtmlDesignModeError::Terminal);
        }
        let next = match (self.pending, reply) {
            (A::ReadEnabled, R::Enabled(true)) => A::Complete,
            (A::ReadEnabled, R::Enabled(false)) => A::SetEnabled { enabled: true },
            (A::SetEnabled { enabled: true }, R::Applied) => A::ReadActiveRange,
            (A::SetEnabled { enabled: false }, R::Applied) => A::Complete,
            (A::ReadActiveRange, R::NoActiveRange) => A::ReadDocumentElement,
            (A::ReadActiveRange, R::ActiveRange) => A::ResetActiveRange,
            (A::ResetActiveRange, R::Applied) => A::ReadDocumentElement,
            (A::ReadDocumentElement, R::DocumentElementPresent(false)) => A::Complete,
            (A::ReadDocumentElement, R::DocumentElementPresent(true)) => A::RunFocusingSteps,
            (A::RunFocusingSteps, R::FocusingStepsCompleted) => A::Complete,
            _ => return Err(HtmlDesignModeError::WrongReply),
        };
        self.pending = next;
        Ok(())
    }
}

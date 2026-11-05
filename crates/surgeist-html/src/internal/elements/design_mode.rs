//! Ordered designMode setter protocol from WHATWG HTML revision
//! 877d6146487f15b15bc8dd0e3c625063a37b456a, retained raw61117–61130.
//!
//! The adapter owns the invoking Document, its current active association and
//! exact Range identity, document element, mutation authority and called
//! focusing continuation. No identity, mutation or focus fact is supplied here.
//! The explicit NoActiveRange branch adopts the selected #1243 correction for
//! the pinned setter's undefined reset of a null active range.

/// ASCII-insensitive designMode input interpretation, without trimming.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDesignModeInput {
    /// The value is `on` in any ASCII case; the designMode setter enables design mode.
    On,
    /// The value is `off` in any ASCII case; the designMode setter disables design mode.
    Off,
    /// Any other value, including non-ASCII units: the setter does nothing.
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

/// Source reads, writes and called-algorithm obligations. All reads use
/// current actual state at the reached stage, rather than an initial snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDesignModeAction {
    /// Read whether the invoking Document's design mode is currently enabled.
    ReadEnabled,
    /// Write the Document's design-mode flag.
    SetEnabled {
        /// The value to store: `true` for design mode on, `false` for off.
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
    /// element, with the trigger omitted. Retain the child and its
    /// actual operands until Complete; no Dialog predicate or unavailable fact
    /// can stand in for this call. Reentry must not restore an earlier flag.
    RunFocusingSteps,
    /// The setter has finished; no further action or reply is expected.
    Complete,
}
/// The adapter's answer to a pending [`HtmlDesignModeAction`].
///
/// A reply is accepted only for the action that asks for it; any other pairing is
/// [`HtmlDesignModeError::is_wrong_reply`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDesignModeReply {
    /// Answer to `ReadEnabled`: whether design mode is currently enabled.
    Enabled(bool),
    /// Authoritatively known null: no reset and no Range creation; still proceed
    /// to the Document's document-element/focusing tail.
    NoActiveRange,
    /// Actual nonnull association captured; retain that exact Range identity.
    ActiveRange,
    /// Answer to `ReadDocumentElement`: whether the Document currently has a document element.
    DocumentElementPresent(bool),
    /// The focusing child reached its own Complete action.
    FocusingStepsCompleted,
    /// The exact pending flag write or both-boundary reset actually completed.
    Applied,
}
/// A designMode cursor reply that cannot be applied.
///
/// The cause is private; query it with [`is_wrong_reply`](Self::is_wrong_reply) and
/// [`is_terminal`](Self::is_terminal). A refused reply leaves the cursor unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDesignModeError {
    kind: HtmlDesignModeErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlDesignModeErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlDesignModeErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlDesignModeError {
        HtmlDesignModeError { kind: self }
    }
}

impl From<HtmlDesignModeErrorKind> for HtmlDesignModeError {
    fn from(kind: HtmlDesignModeErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlDesignModeError {
    /// Whether the cause is: reply does not match the pending designMode action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlDesignModeErrorKind::WrongReply)
    }
    /// Whether the cause is: designMode setter is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlDesignModeErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlDesignModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlDesignModeErrorKind::WrongReply => {
                f.write_str("reply does not match the pending designMode action")
            }
            HtmlDesignModeErrorKind::Terminal => {
                f.write_str("designMode setter is already complete")
            }
        }
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
    /// Starts the setter for an interpreted value.
    ///
    /// `On` begins by reading the current flag, `Off` begins by writing `false`, and an invalid
    /// value is already complete.
    pub const fn new(input: HtmlDesignModeInput) -> Self {
        Self {
            pending: match input {
                HtmlDesignModeInput::On => HtmlDesignModeAction::ReadEnabled,
                HtmlDesignModeInput::Off => HtmlDesignModeAction::SetEnabled { enabled: false },
                HtmlDesignModeInput::Invalid => HtmlDesignModeAction::Complete,
            },
        }
    }
    /// The action the adapter must perform next, or `Complete` when the setter is finished.
    pub const fn action(&self) -> HtmlDesignModeAction {
        self.pending
    }
    /// Wrong-stage and terminal replies preserve the entire exact cursor.
    pub fn reply(&mut self, reply: HtmlDesignModeReply) -> Result<(), HtmlDesignModeError> {
        use HtmlDesignModeAction as A;
        use HtmlDesignModeReply as R;
        if self.pending == A::Complete {
            return Err(HtmlDesignModeErrorKind::Terminal.error());
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
            _ => return Err(HtmlDesignModeErrorKind::WrongReply.error()),
        };
        self.pending = next;
        Ok(())
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_design_mode_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlDesignModeError) -> bool;
        let cases: Vec<(HtmlDesignModeError, &str, Predicate)> = vec![
            (
                HtmlDesignModeErrorKind::WrongReply.error(),
                "reply does not match the pending designMode action",
                HtmlDesignModeError::is_wrong_reply,
            ),
            (
                HtmlDesignModeErrorKind::Terminal.error(),
                "designMode setter is already complete",
                HtmlDesignModeError::is_terminal,
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

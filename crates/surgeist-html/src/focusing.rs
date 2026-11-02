//! Complete focusing-steps source-body protocol from WHATWG HTML revision
//! 877d6146487f15b15bc8dd0e3c625063a37b456a, retained raw60163–60185.
//!
//! The original adapter owns one current newFocusTarget, actual optional fallback,
//! original trigger value/suppliedness, nullable current area, and old/new chains.
//! Complete called algorithms are real obligations; this module does not provide
//! focus facts, event execution, chain construction or focus-update services.

/// Whether this invocation actually supplied a fallback target. This is not the
/// nullable result of get-the-focusable-area; the adapter retains the exact value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusingFallback {
    Omitted,
    Supplied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusingAction {
    ReadInitialTargetFocusableArea,
    /// Complete get-the-focusable-area for the actual current target, forwarding
    /// the original trigger IF supplied. Omission remains omission and therefore
    /// uses that algorithm's default other; a supplied value is never rewritten.
    /// Replace the adapter's current target with the actual nullable result.
    RunGetFocusableArea,
    /// Set the current target to the exact original fallback once. Do not run
    /// get-the-focusable-area again or normalize/re-resolve that supplied value.
    UseFallbackTarget,
    ReadTargetIsNavigableContainer,
    ReadContentNavigable,
    /// Replace the current target with the actual content navigable's active
    /// Document; do not substitute a viewport or preserve the old area kind.
    CaptureContentActiveDocument,
    /// Fresh area test AFTER possible resolution/fallback/content replacement.
    ReadReplacementTargetFocusableArea,
    /// Read the actual current target area's DOM anchor's inertness, without
    /// assuming the area itself is an Element or the originally supplied target.
    ReadActualAnchorInert,
    /// Complete the existing current-area child on the current target's actual
    /// top-level traversable and retain the genuine nullable result.
    RunCurrentlyFocusedArea,
    /// Compare actual current target identity against that actual nullable area;
    /// DOM-anchor equality is insufficient for distinct subwidgets/regions.
    ReadSameTarget,
    /// Run the actual current-focus-chain dependency for the current target's
    /// actual top-level traversable, retaining old chain. Known absent traversable
    /// OR known null current area yields an empty chain; only actual nonnull area
    /// enters focus-chain. Unknown facts do not establish empty-chain evidence.
    RunCurrentFocusChain,
    /// Run actual focus-chain for the current new target and retain new chain,
    /// preserving source order and any non-Element area/Document identities.
    RunFocusChain,
    /// Run the real focus-update call with retained old chain, retained new chain
    /// and actual current target. Its callbacks/publications are not implemented
    /// by producing this action or by reaching a previous source stage.
    RunFocusUpdate,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusingReply {
    InitialTargetFocusableArea(bool),
    /// The actual completed get-area call's nullable return, retained as target.
    ResolvedTargetPresent(bool),
    TargetIsNavigableContainer(bool),
    ContentNavigablePresent(bool),
    ReplacementTargetFocusableArea(bool),
    ActualAnchorInert(bool),
    /// Existing current-area child completed and its actual nullable result is
    /// retained by the original adapter. This does not answer the identity test.
    CurrentlyFocusedAreaReady,
    SameTarget(bool),
    CurrentFocusChainReady,
    FocusChainReady,
    FocusUpdateCompleted,
    /// Actual fallback installation or content-Document replacement completed.
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFocusingError {
    kind: HtmlFocusingErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlFocusingErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlFocusingErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlFocusingError {
        HtmlFocusingError { kind: self }
    }
}

impl From<HtmlFocusingErrorKind> for HtmlFocusingError {
    fn from(kind: HtmlFocusingErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlFocusingError {
    /// Whether the cause is: reply does not match the pending focusing action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlFocusingErrorKind::WrongReply)
    }
    /// Whether the cause is: focusing steps are already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlFocusingErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlFocusingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlFocusingErrorKind::WrongReply => {
                f.write_str("reply does not match the pending focusing action")
            }
            HtmlFocusingErrorKind::Terminal => f.write_str("focusing steps are already complete"),
        }
    }
}

impl std::error::Error for HtmlFocusingError {}

/// Constant-state complete source-body protocol. The adapter qualifies source
/// inputs (area, non-area Element or navigable) and retains actual operands,
/// finite admission and called-algorithm custody. Copy grants no execution/replay
/// authority. No activity predicate, focusability proof, callback or identity
/// registry is added. Focus-chain/current-chain/update remain genuine called
/// dependencies rather than implemented services of this pure cursor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFocusingCursor {
    pending: HtmlFocusingAction,
    fallback: HtmlFocusingFallback,
}
impl HtmlFocusingCursor {
    pub const fn new(fallback: HtmlFocusingFallback) -> Self {
        Self {
            pending: HtmlFocusingAction::ReadInitialTargetFocusableArea,
            fallback,
        }
    }
    pub const fn action(&self) -> HtmlFocusingAction {
        self.pending
    }
    /// Wrong-stage and terminal replies preserve the entire exact cursor.
    pub fn reply(&mut self, reply: HtmlFocusingReply) -> Result<(), HtmlFocusingError> {
        use HtmlFocusingAction as A;
        use HtmlFocusingReply as R;
        if self.pending == A::Complete {
            return Err(HtmlFocusingErrorKind::Terminal.error());
        }
        let next = match (self.pending, reply) {
            (A::ReadInitialTargetFocusableArea, R::InitialTargetFocusableArea(true)) => {
                A::ReadTargetIsNavigableContainer
            }
            (A::ReadInitialTargetFocusableArea, R::InitialTargetFocusableArea(false)) => {
                A::RunGetFocusableArea
            }
            (A::RunGetFocusableArea, R::ResolvedTargetPresent(true)) => {
                A::ReadTargetIsNavigableContainer
            }
            (A::RunGetFocusableArea, R::ResolvedTargetPresent(false)) => match self.fallback {
                HtmlFocusingFallback::Omitted => A::Complete,
                HtmlFocusingFallback::Supplied => A::UseFallbackTarget,
            },
            (A::UseFallbackTarget, R::Applied) => A::ReadTargetIsNavigableContainer,
            (A::ReadTargetIsNavigableContainer, R::TargetIsNavigableContainer(true)) => {
                A::ReadContentNavigable
            }
            (A::ReadTargetIsNavigableContainer, R::TargetIsNavigableContainer(false)) => {
                A::ReadReplacementTargetFocusableArea
            }
            (A::ReadContentNavigable, R::ContentNavigablePresent(true)) => {
                A::CaptureContentActiveDocument
            }
            (A::ReadContentNavigable, R::ContentNavigablePresent(false)) => {
                A::ReadReplacementTargetFocusableArea
            }
            (A::CaptureContentActiveDocument, R::Applied) => A::ReadReplacementTargetFocusableArea,
            (A::ReadReplacementTargetFocusableArea, R::ReplacementTargetFocusableArea(true)) => {
                A::ReadActualAnchorInert
            }
            (A::ReadReplacementTargetFocusableArea, R::ReplacementTargetFocusableArea(false)) => {
                A::RunCurrentlyFocusedArea
            }
            (A::ReadActualAnchorInert, R::ActualAnchorInert(true)) => A::Complete,
            (A::ReadActualAnchorInert, R::ActualAnchorInert(false)) => A::RunCurrentlyFocusedArea,
            (A::RunCurrentlyFocusedArea, R::CurrentlyFocusedAreaReady) => A::ReadSameTarget,
            (A::ReadSameTarget, R::SameTarget(true)) => A::Complete,
            (A::ReadSameTarget, R::SameTarget(false)) => A::RunCurrentFocusChain,
            (A::RunCurrentFocusChain, R::CurrentFocusChainReady) => A::RunFocusChain,
            (A::RunFocusChain, R::FocusChainReady) => A::RunFocusUpdate,
            (A::RunFocusUpdate, R::FocusUpdateCompleted) => A::Complete,
            _ => return Err(HtmlFocusingErrorKind::WrongReply.error()),
        };
        self.pending = next;
        Ok(())
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_focusing_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlFocusingError) -> bool;
        let cases: Vec<(HtmlFocusingError, &str, Predicate)> = vec![
            (
                HtmlFocusingErrorKind::WrongReply.error(),
                "reply does not match the pending focusing action",
                HtmlFocusingError::is_wrong_reply,
            ),
            (
                HtmlFocusingErrorKind::Terminal.error(),
                "focusing steps are already complete",
                HtmlFocusingError::is_terminal,
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

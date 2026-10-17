//! Complete get-the-focusable-area language protocol from WHATWG HTML revision
//! 877d6146487f15b15bc8dd0e3c625063a37b456a, retained raw60083–60110.
//!
//! The adapter owns the actual original target, original trigger value and
//! suppliedness, selected objects, graph qualification and invocation custody.
//! Omission uses the source default `other`; a supplied trigger is forwarded
//! unchanged through existing focus-delegate and nested resolution calls.

use crate::HtmlCurrentlyFocusedAreaOutcome;

/// The exact source result's role. Actual objects remain adapter-owned: a shape,
/// scroll region, viewport, Document, anchor and delegated area are distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlGetFocusableAreaOutcome {
    Null,
    MapShape,
    ScrollRegion,
    Viewport,
    NavigableActiveDocument,
    ContentActiveDocument,
    CurrentAnchor,
    Delegate,
}

/// First-matching-clause reads and real selection/called-algorithm obligations.
/// Missing facts do not provide a false reply or a completed selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlGetFocusableAreaAction {
    /// Whether the actual target is an Area element with at least one shape
    /// that is an actual focusable area; not merely an authored coords value.
    ReadAreaWithFocusableShapes,
    /// Select the shape corresponding to the FIRST img in ordinary tree order
    /// that uses the target's map. This is not a scan of just retained shapes
    /// or a choice of the first already-known focusable img.
    SelectFirstImageMapShape,
    ReadElementWithFocusableScrollRegions,
    /// Select the first scrollable region by flat-tree preorder depth-first
    /// traversal. Ordinary tree order or an arbitrary region is insufficient.
    SelectFirstScrollableRegion,
    ReadDocumentElement,
    CaptureViewport,
    ReadNavigable,
    CaptureNavigableActiveDocument,
    ReadNavigableContainer,
    /// Retain the actual content navigable if nonnull.
    ReadContentNavigable,
    CaptureContentActiveDocument,
    ReadDelegatingShadowHost,
    /// Complete the existing currently-focused-area algorithm for the actual
    /// top-level traversable, retaining its exact Null, CandidateDocument or
    /// CandidateFocusedArea result. A non-area Document has no area anchor.
    RunCurrentlyFocusedArea,
    /// Capture the actual DOM anchor of that nonnull returned area. Do not
    /// replace the returned area with an Element earlier in its child protocol.
    CaptureCurrentAreaAnchor,
    /// Test the original target's actual shadow-including inclusive ancestry
    /// of the retained anchor, including equality.
    ReadShadowIncludingInclusiveAncestor,
    /// Complete the existing focus-delegate algorithm for the original target
    /// with the unchanged original trigger, retaining its nullable result.
    RunFocusDelegate,
    Complete(HtmlGetFocusableAreaOutcome),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlGetFocusableAreaReply {
    AreaWithFocusableShapes(bool),
    ElementWithFocusableScrollRegions(bool),
    DocumentElement(bool),
    Navigable(bool),
    NavigableContainer(bool),
    ContentNavigablePresent(bool),
    DelegatingShadowHost(bool),
    /// Exact result role of the completed current-area child. CandidateDocument
    /// is not null and does not acquire a DOM anchor; a genuine viewport area
    /// remains CandidateFocusedArea even when its actual anchor is a Document.
    CurrentlyFocusedArea(HtmlCurrentlyFocusedAreaOutcome),
    ShadowIncludingInclusiveAncestor(bool),
    FocusDelegatePresent(bool),
    /// Selection/capture actually fulfilled by the original adapter.
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlGetFocusableAreaError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlGetFocusableAreaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending get-focusable-area action",
            Self::Terminal => "get-focusable-area is already complete",
        })
    }
}
impl std::error::Error for HtmlGetFocusableAreaError {}

/// Constant-state, allocation-free complete source-body protocol. The caller
/// qualifies the source input domain: an Element that is not a focusable area,
/// or a navigable. This cursor introduces no area identity or service provider,
/// performs no hidden scans and grants no replay authority through Copy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlGetFocusableAreaCursor {
    pending: HtmlGetFocusableAreaAction,
}
impl HtmlGetFocusableAreaCursor {
    pub const fn new() -> Self {
        Self {
            pending: HtmlGetFocusableAreaAction::ReadAreaWithFocusableShapes,
        }
    }
    pub const fn action(&self) -> HtmlGetFocusableAreaAction {
        self.pending
    }
    /// Wrong-stage and terminal replies preserve the entire exact cursor.
    pub fn reply(
        &mut self,
        reply: HtmlGetFocusableAreaReply,
    ) -> Result<(), HtmlGetFocusableAreaError> {
        use HtmlGetFocusableAreaAction as A;
        use HtmlGetFocusableAreaOutcome as O;
        use HtmlGetFocusableAreaReply as R;
        if matches!(self.pending, A::Complete(_)) {
            return Err(HtmlGetFocusableAreaError::Terminal);
        }
        let next = match (self.pending, reply) {
            (A::ReadAreaWithFocusableShapes, R::AreaWithFocusableShapes(true)) => {
                A::SelectFirstImageMapShape
            }
            (A::ReadAreaWithFocusableShapes, R::AreaWithFocusableShapes(false)) => {
                A::ReadElementWithFocusableScrollRegions
            }
            (A::SelectFirstImageMapShape, R::Applied) => A::Complete(O::MapShape),
            (
                A::ReadElementWithFocusableScrollRegions,
                R::ElementWithFocusableScrollRegions(true),
            ) => A::SelectFirstScrollableRegion,
            (
                A::ReadElementWithFocusableScrollRegions,
                R::ElementWithFocusableScrollRegions(false),
            ) => A::ReadDocumentElement,
            (A::SelectFirstScrollableRegion, R::Applied) => A::Complete(O::ScrollRegion),
            (A::ReadDocumentElement, R::DocumentElement(true)) => A::CaptureViewport,
            (A::ReadDocumentElement, R::DocumentElement(false)) => A::ReadNavigable,
            (A::CaptureViewport, R::Applied) => A::Complete(O::Viewport),
            (A::ReadNavigable, R::Navigable(true)) => A::CaptureNavigableActiveDocument,
            (A::ReadNavigable, R::Navigable(false)) => A::ReadNavigableContainer,
            (A::CaptureNavigableActiveDocument, R::Applied) => {
                A::Complete(O::NavigableActiveDocument)
            }
            (A::ReadNavigableContainer, R::NavigableContainer(true)) => A::ReadContentNavigable,
            (A::ReadNavigableContainer, R::NavigableContainer(false)) => {
                A::ReadDelegatingShadowHost
            }
            (A::ReadContentNavigable, R::ContentNavigablePresent(true)) => {
                A::CaptureContentActiveDocument
            }
            (A::ReadContentNavigable, R::ContentNavigablePresent(false)) => {
                A::ReadDelegatingShadowHost
            }
            (A::CaptureContentActiveDocument, R::Applied) => A::Complete(O::ContentActiveDocument),
            (A::ReadDelegatingShadowHost, R::DelegatingShadowHost(true)) => {
                A::RunCurrentlyFocusedArea
            }
            (A::ReadDelegatingShadowHost, R::DelegatingShadowHost(false)) => A::Complete(O::Null),
            // Only a genuine area result has the anchor required by this fast
            // path. Null and a non-area Document both proceed to real delegation.
            (
                A::RunCurrentlyFocusedArea,
                R::CurrentlyFocusedArea(
                    HtmlCurrentlyFocusedAreaOutcome::Null
                    | HtmlCurrentlyFocusedAreaOutcome::CandidateDocument,
                ),
            ) => A::RunFocusDelegate,
            (
                A::RunCurrentlyFocusedArea,
                R::CurrentlyFocusedArea(HtmlCurrentlyFocusedAreaOutcome::CandidateFocusedArea),
            ) => A::CaptureCurrentAreaAnchor,
            (A::CaptureCurrentAreaAnchor, R::Applied) => A::ReadShadowIncludingInclusiveAncestor,
            (
                A::ReadShadowIncludingInclusiveAncestor,
                R::ShadowIncludingInclusiveAncestor(true),
            ) => A::Complete(O::CurrentAnchor),
            (
                A::ReadShadowIncludingInclusiveAncestor,
                R::ShadowIncludingInclusiveAncestor(false),
            ) => A::RunFocusDelegate,
            (A::RunFocusDelegate, R::FocusDelegatePresent(true)) => A::Complete(O::Delegate),
            (A::RunFocusDelegate, R::FocusDelegatePresent(false)) => A::Complete(O::Null),
            _ => return Err(HtmlGetFocusableAreaError::WrongReply),
        };
        self.pending = next;
        Ok(())
    }
}
impl Default for HtmlGetFocusableAreaCursor {
    fn default() -> Self {
        Self::new()
    }
}

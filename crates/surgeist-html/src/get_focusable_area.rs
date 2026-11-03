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
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlGetFocusableAreaOutcome {
    /// No focusable area: the target matched no case, or it is a delegating shadow
    /// host whose focus delegate returned null.
    Null,
    /// The shape that corresponds to the first `img` in tree order that uses the area
    /// element's image map.
    MapShape,
    /// The element's first scrollable region in a pre-order, depth-first traversal of
    /// the flat tree.
    ScrollRegion,
    /// The viewport of the Document whose document element is the target.
    Viewport,
    /// The active document of the target, which is a navigable.
    NavigableActiveDocument,
    /// The active document of the content navigable of the target, a navigable
    /// container.
    ContentActiveDocument,
    /// The DOM anchor of the currently focused area, returned for a delegating shadow
    /// host that is a shadow-including inclusive ancestor of it.
    CurrentAnchor,
    /// The nonnull area returned by the focus delegate of a delegating shadow host.
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
    /// Read whether the target is an element with one or more scrollable regions that
    /// are focusable areas.
    ReadElementWithFocusableScrollRegions,
    /// Select the first scrollable region by flat-tree preorder depth-first
    /// traversal. Ordinary tree order or an arbitrary region is insufficient.
    SelectFirstScrollableRegion,
    /// Read whether the target is the document element of its Document.
    ReadDocumentElement,
    /// Capture the viewport of the target's Document.
    CaptureViewport,
    /// Read whether the target is a navigable.
    ReadNavigable,
    /// Capture the active document of the target navigable.
    CaptureNavigableActiveDocument,
    /// Read whether the target is a navigable container.
    ReadNavigableContainer,
    /// Retain the actual content navigable if nonnull.
    ReadContentNavigable,
    /// Capture the active document of the content navigable.
    CaptureContentActiveDocument,
    /// Read whether the target is a shadow host whose shadow root delegates focus.
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
    /// The algorithm is finished with this result. No reply is accepted afterward.
    Complete(HtmlGetFocusableAreaOutcome),
}

/// Observations and acknowledgments for the get-the-focusable-area cursor. Each
/// reply answers exactly the pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlGetFocusableAreaReply {
    /// Whether the target is an area element with one or more shapes that are
    /// focusable areas.
    AreaWithFocusableShapes(bool),
    /// Whether the target is an element with one or more scrollable regions that are
    /// focusable areas.
    ElementWithFocusableScrollRegions(bool),
    /// Whether the target is the document element of its Document.
    DocumentElement(bool),
    /// Whether the target is a navigable.
    Navigable(bool),
    /// Whether the target is a navigable container.
    NavigableContainer(bool),
    /// Whether the navigable container's content navigable is nonnull.
    ContentNavigablePresent(bool),
    /// Whether the target is a shadow host whose shadow root delegates focus.
    DelegatingShadowHost(bool),
    /// Exact result role of the completed current-area child. CandidateDocument
    /// is not null and does not acquire a DOM anchor; a genuine viewport area
    /// remains CandidateFocusedArea even when its actual anchor is a Document.
    CurrentlyFocusedArea(HtmlCurrentlyFocusedAreaOutcome),
    /// Whether the target is a shadow-including inclusive ancestor of the retained
    /// anchor, including equality.
    ShadowIncludingInclusiveAncestor(bool),
    /// Whether the focus delegate returned a nonnull area.
    FocusDelegatePresent(bool),
    /// Selection/capture actually fulfilled by the original adapter.
    Applied,
}
/// Why a get-focusable-area cursor refused a reply, queried through `is_*`
/// predicates. The cursor stays unchanged after a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlGetFocusableAreaError {
    kind: HtmlGetFocusableAreaErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlGetFocusableAreaErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlGetFocusableAreaErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlGetFocusableAreaError {
        HtmlGetFocusableAreaError { kind: self }
    }
}

impl From<HtmlGetFocusableAreaErrorKind> for HtmlGetFocusableAreaError {
    fn from(kind: HtmlGetFocusableAreaErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlGetFocusableAreaError {
    /// Whether the cause is: reply does not match the pending get-focusable-area action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlGetFocusableAreaErrorKind::WrongReply)
    }
    /// Whether the cause is: get-focusable-area is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlGetFocusableAreaErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlGetFocusableAreaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlGetFocusableAreaErrorKind::WrongReply => {
                f.write_str("reply does not match the pending get-focusable-area action")
            }
            HtmlGetFocusableAreaErrorKind::Terminal => {
                f.write_str("get-focusable-area is already complete")
            }
        }
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
    /// Starts get the focusable area of HTML 6.6.4 (Processing model), which tests the
    /// target's cases in source order.
    pub const fn new() -> Self {
        Self {
            pending: HtmlGetFocusableAreaAction::ReadAreaWithFocusableShapes,
        }
    }
    /// Returns the read, selection or called algorithm that must be fulfilled before
    /// the next reply.
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
            return Err(HtmlGetFocusableAreaErrorKind::Terminal.error());
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
            _ => return Err(HtmlGetFocusableAreaErrorKind::WrongReply.error()),
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

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_get_focusable_area_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlGetFocusableAreaError) -> bool;
        let cases: Vec<(HtmlGetFocusableAreaError, &str, Predicate)> = vec![
            (
                HtmlGetFocusableAreaErrorKind::WrongReply.error(),
                "reply does not match the pending get-focusable-area action",
                HtmlGetFocusableAreaError::is_wrong_reply,
            ),
            (
                HtmlGetFocusableAreaErrorKind::Terminal.error(),
                "get-focusable-area is already complete",
                HtmlGetFocusableAreaError::is_terminal,
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

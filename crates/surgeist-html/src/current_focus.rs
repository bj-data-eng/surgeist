//! Complete currently-focused-area-of-a-top-level-traversable language protocol.
//!
//! Source: WHATWG HTML revision 877d6146487f15b15bc8dd0e3c625063a37b456a,
//! retained complete source raw59747–59758. The actual traversable, candidate
//! Document, focused area and content navigable remain with the original adapter.
//! A focused area is not restricted to an Element or replaced with its DOM anchor.

/// Classification of the candidate Document's actual focused area at the loop
/// test. The adapter retains the exact area when present. This is not a proof
/// about the separately reached final focused-area read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusedAreaKind {
    Null,
    NavigableContainer,
    OtherArea,
}

/// Source result role. The adapter supplies the actual retained identity/area;
/// CandidateDocument is a real result, not a fabricated nullable Element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCurrentlyFocusedAreaOutcome {
    Null,
    CandidateDocument,
    CandidateFocusedArea,
}

/// Actual reads and captures, in source order. Each capture must complete before
/// Applied; missing graph or designation evidence supplies no acknowledgment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCurrentlyFocusedAreaAction {
    /// Read actual system focus of the original top-level traversable.
    ReadSystemFocus,
    /// Retain that traversable's actual active Document as candidate. Missing
    /// active-Document evidence is unavailable, never a null result or fallback.
    CaptureTraversableActiveDocument,
    /// Read the candidate's actual focused-area designation for the while test.
    /// Retain the exact navigable container when that classification is returned.
    ReadLoopFocusedArea,
    /// Read the retained loop container's actual nullable content navigable,
    /// retaining that navigable when nonnull.
    ReadContentNavigable,
    /// Set candidate to the retained content navigable's actual active Document.
    /// Complete the capture before proceeding to the next loop test.
    CaptureContentActiveDocument,
    /// Read candidate's focused area again at the final source test. Retain its
    /// actual nonnull area for the result; never reuse the loop predicate or
    /// project this observation to an Element/DOM anchor.
    ReadResultFocusedArea,
    Complete(HtmlCurrentlyFocusedAreaOutcome),
}

/// Known actual facts or completed captures, for exactly the pending action.
/// False is an actual negative, never an unavailable observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCurrentlyFocusedAreaReply {
    SystemFocus(bool),
    LoopFocusedArea(HtmlFocusedAreaKind),
    ContentNavigablePresent(bool),
    ResultFocusedAreaPresent(bool),
    Applied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCurrentlyFocusedAreaError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlCurrentlyFocusedAreaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending current focus action",
            Self::Terminal => "current focus query is already complete",
        })
    }
}
impl std::error::Error for HtmlCurrentlyFocusedAreaError {}

/// Constant-state, allocation-free language cursor. Original adapters own actual
/// graph qualification, finite traversal work/storage, cancellation and execution
/// custody. This producer has no callbacks, focus facts or identity registry; it
/// adds no fully-active predicate or filtering to the source algorithm. Copying
/// grants no execution/replay authority. Dropping performs no external effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCurrentlyFocusedAreaCursor {
    pending: HtmlCurrentlyFocusedAreaAction,
}
impl HtmlCurrentlyFocusedAreaCursor {
    pub const fn new() -> Self {
        Self {
            pending: HtmlCurrentlyFocusedAreaAction::ReadSystemFocus,
        }
    }
    pub const fn action(&self) -> HtmlCurrentlyFocusedAreaAction {
        self.pending
    }
    /// Wrong-stage and terminal replies preserve the entire exact cursor.
    pub fn reply(
        &mut self,
        reply: HtmlCurrentlyFocusedAreaReply,
    ) -> Result<(), HtmlCurrentlyFocusedAreaError> {
        use HtmlCurrentlyFocusedAreaAction as A;
        use HtmlCurrentlyFocusedAreaOutcome as O;
        use HtmlCurrentlyFocusedAreaReply as R;
        if matches!(self.pending, A::Complete(_)) {
            return Err(HtmlCurrentlyFocusedAreaError::Terminal);
        }
        let next = match (self.pending, reply) {
            (A::ReadSystemFocus, R::SystemFocus(false)) => A::Complete(O::Null),
            (A::ReadSystemFocus, R::SystemFocus(true)) => A::CaptureTraversableActiveDocument,
            (A::CaptureTraversableActiveDocument, R::Applied)
            | (A::CaptureContentActiveDocument, R::Applied) => A::ReadLoopFocusedArea,
            (
                A::ReadLoopFocusedArea,
                R::LoopFocusedArea(HtmlFocusedAreaKind::NavigableContainer),
            ) => A::ReadContentNavigable,
            (
                A::ReadLoopFocusedArea,
                R::LoopFocusedArea(HtmlFocusedAreaKind::Null | HtmlFocusedAreaKind::OtherArea),
            ) => A::ReadResultFocusedArea,
            (A::ReadContentNavigable, R::ContentNavigablePresent(true)) => {
                A::CaptureContentActiveDocument
            }
            (A::ReadContentNavigable, R::ContentNavigablePresent(false)) => {
                A::ReadResultFocusedArea
            }
            (A::ReadResultFocusedArea, R::ResultFocusedAreaPresent(false)) => {
                A::Complete(O::CandidateDocument)
            }
            (A::ReadResultFocusedArea, R::ResultFocusedAreaPresent(true)) => {
                A::Complete(O::CandidateFocusedArea)
            }
            _ => return Err(HtmlCurrentlyFocusedAreaError::WrongReply),
        };
        self.pending = next;
        Ok(())
    }
}
impl Default for HtmlCurrentlyFocusedAreaCursor {
    fn default() -> Self {
        Self::new()
    }
}

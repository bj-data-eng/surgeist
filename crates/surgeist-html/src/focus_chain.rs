//! Focus-chain protocols from WHATWG HTML revision
//! 877d6146487f15b15bc8dd0e3c625063a37b456a, raw59761–59808.
//!
//! The selected editorial interpretations give a known-null current area an
//! empty current chain and ascend from a child Document through that child's
//! own navigable container Element. Identities, ordered output, current graph
//! qualification, finite work/storage, cancellation and execution custody belong
//! to the original adapter. These cursors perform no native focusing or dispatch.

/// Calls and output effects for the current focus chain of the original actual
/// nullable traversable. Missing observations supply no reply.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCurrentFocusChainAction {
    ReadTraversable,
    /// Complete the currently-focused-area child for the original traversable,
    /// retaining its actual nullable area-or-Document result. Do not substitute
    /// an anchor Element or convert a Document result into a viewport area.
    RunCurrentlyFocusedArea,
    /// Establish a real empty output list before acknowledging this effect.
    InitializeEmptyChain,
    /// Complete focus-chain for the actual nonnull area or Document returned by the child,
    /// retaining the entire ordered result before supplying FocusChainReady.
    RunFocusChain,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCurrentFocusChainReply {
    TraversablePresent(bool),
    CurrentlyFocusedAreaPresent(bool),
    FocusChainReady,
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCurrentFocusChainError {
    WrongReply,
    Terminal,
}
impl std::fmt::Display for HtmlCurrentFocusChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending current focus chain action",
            Self::Terminal => "current focus chain is already complete",
        })
    }
}
impl std::error::Error for HtmlCurrentFocusChainError {}

/// Constant-state language continuation. Called algorithms must actually finish;
/// a request is not an acknowledgment. Dropping performs no output or service.
#[derive(Debug, Eq, PartialEq)]
pub struct HtmlCurrentFocusChainCursor {
    pending: HtmlCurrentFocusChainAction,
}
impl HtmlCurrentFocusChainCursor {
    pub const fn new() -> Self {
        Self {
            pending: HtmlCurrentFocusChainAction::ReadTraversable,
        }
    }
    pub const fn action(&self) -> HtmlCurrentFocusChainAction {
        self.pending
    }
    /// Wrong-stage and terminal replies leave the entire cursor unchanged.
    pub fn reply(
        &mut self,
        reply: HtmlCurrentFocusChainReply,
    ) -> Result<(), HtmlCurrentFocusChainError> {
        use HtmlCurrentFocusChainAction as A;
        use HtmlCurrentFocusChainReply as R;
        if self.pending == A::Complete {
            return Err(HtmlCurrentFocusChainError::Terminal);
        }
        let next = match (self.pending, reply) {
            (A::ReadTraversable, R::TraversablePresent(false)) => A::InitializeEmptyChain,
            (A::ReadTraversable, R::TraversablePresent(true)) => A::RunCurrentlyFocusedArea,
            (A::RunCurrentlyFocusedArea, R::CurrentlyFocusedAreaPresent(false)) => {
                A::InitializeEmptyChain
            }
            (A::RunCurrentlyFocusedArea, R::CurrentlyFocusedAreaPresent(true)) => A::RunFocusChain,
            (A::InitializeEmptyChain, R::Applied) | (A::RunFocusChain, R::FocusChainReady) => {
                A::Complete
            }
            _ => return Err(HtmlCurrentFocusChainError::WrongReply),
        };
        self.pending = next;
        Ok(())
    }
}
impl Default for HtmlCurrentFocusChainCursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Actual reads, captures and ordered-list appends in focus-chain source order.
/// Every Applied acknowledges that exact effect on the adapter's retained local
/// or output. In particular a refused append must not advance the cursor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusChainAction {
    InitializeEmptyChain,
    /// Retain the original qualified area-or-Document subject as currentObject.
    CaptureSubject,
    AppendCurrentObject,
    ReadAreaShape,
    /// Append the shape's corresponding area Element, not its image DOM anchor.
    AppendCorrespondingAreaElement,
    /// Reached only for an actual area. Read whether its actual DOM anchor is
    /// an Element unequal to currentObject; never project a non-area Document.
    ReadAnchorIsDifferentElement,
    AppendAnchorElement,
    /// Classify actual area membership before any optional anchor query.
    ReadFocusableArea,
    /// Replace currentObject with its actual anchor's node Document. A Document
    /// anchor denotes itself. Do not use a cached original subject Document.
    CaptureAnchorNodeDocument,
    /// Qualify the non-area currentObject as a Document. False is outside the
    /// chain's area-or-Document domain and fails without completing the chain.
    ReadDocument,
    /// Retain the actual current Document's node navigable, defined by active
    /// Document equality (raw70917). Missing graph evidence is not actual null.
    ReadNodeNavigable,
    /// Read the retained CHILD navigable's actual nullable parent (raw70837).
    ReadNavigableParent,
    /// Replace currentObject with the retained child's own actual container
    /// Element (raw71050), never the parent Navigable, its container, or a direct
    /// parent Document. Before Applied, qualify the exact child/parent/container,
    /// container content-navigable and enclosing Document relations at the actual
    /// basis. Parent-present but missing/unrepresentable/inconsistent container
    /// is unavailable, not successful shortened completion. No activity filter.
    CaptureChildNavigableContainerElement,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusChainReply {
    AreaShape(bool),
    AnchorIsDifferentElement(bool),
    FocusableArea(bool),
    Document(bool),
    NodeNavigablePresent(bool),
    NavigableParentPresent(bool),
    Applied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusChainError {
    WrongReply,
    /// An actual non-area currentObject is not a Document either.
    InvalidCurrentObject,
    Terminal,
}
impl std::fmt::Display for HtmlFocusChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongReply => "reply does not match the pending focus chain action",
            Self::InvalidCurrentObject => "focus chain object is neither an area nor a Document",
            Self::Terminal => "focus chain is already complete",
        })
    }
}
impl std::error::Error for HtmlFocusChainError {}

/// Allocation-free, constant-state, non-Copy language continuation. The original
/// adapter owns the subject, currentObject, exact area/anchor/navigable identities
/// and output list. A subject can be an actual area or the actual non-area
/// Document returned by current-area resolution, without inventing an anchor for
/// that Document. Before the initial output effect, the adapter must qualify
/// this area-or-actual-Document input domain. Arbitrary non-area Nodes are outside
/// it; InvalidCurrentObject also rejects an inconsistent later classification.
/// This cursor
/// adds no fully-active predicate, identity registry
/// or graph traversal. No external effect runs on reply failure or drop.
#[derive(Debug, Eq, PartialEq)]
pub struct HtmlFocusChainCursor {
    pending: HtmlFocusChainAction,
}
impl HtmlFocusChainCursor {
    pub const fn new() -> Self {
        Self {
            pending: HtmlFocusChainAction::InitializeEmptyChain,
        }
    }
    pub const fn action(&self) -> HtmlFocusChainAction {
        self.pending
    }
    /// Every invalid reply preserves the exact pending action and continuation.
    pub fn reply(&mut self, reply: HtmlFocusChainReply) -> Result<(), HtmlFocusChainError> {
        use HtmlFocusChainAction as A;
        use HtmlFocusChainReply as R;
        if self.pending == A::Complete {
            return Err(HtmlFocusChainError::Terminal);
        }
        let next = match (self.pending, reply) {
            (A::InitializeEmptyChain, R::Applied) => A::CaptureSubject,
            (
                A::CaptureSubject
                | A::CaptureAnchorNodeDocument
                | A::CaptureChildNavigableContainerElement,
                R::Applied,
            ) => A::AppendCurrentObject,
            (A::AppendCurrentObject, R::Applied) => A::ReadAreaShape,
            (A::ReadAreaShape, R::AreaShape(true)) => A::AppendCorrespondingAreaElement,
            (A::ReadAreaShape, R::AreaShape(false)) => A::ReadFocusableArea,
            (A::ReadAnchorIsDifferentElement, R::AnchorIsDifferentElement(true)) => {
                A::AppendAnchorElement
            }
            (A::ReadAnchorIsDifferentElement, R::AnchorIsDifferentElement(false))
            | (A::AppendCorrespondingAreaElement | A::AppendAnchorElement, R::Applied) => {
                A::CaptureAnchorNodeDocument
            }
            (A::ReadFocusableArea, R::FocusableArea(true)) => A::ReadAnchorIsDifferentElement,
            (A::ReadFocusableArea, R::FocusableArea(false)) => A::ReadDocument,
            (A::ReadDocument, R::Document(true)) => A::ReadNodeNavigable,
            (A::ReadDocument, R::Document(false)) => {
                return Err(HtmlFocusChainError::InvalidCurrentObject);
            }
            (A::ReadNodeNavigable, R::NodeNavigablePresent(true)) => A::ReadNavigableParent,
            (A::ReadNodeNavigable, R::NodeNavigablePresent(false))
            | (A::ReadNavigableParent, R::NavigableParentPresent(false)) => A::Complete,
            (A::ReadNavigableParent, R::NavigableParentPresent(true)) => {
                A::CaptureChildNavigableContainerElement
            }
            _ => return Err(HtmlFocusChainError::WrongReply),
        };
        self.pending = next;
        Ok(())
    }
}
impl Default for HtmlFocusChainCursor {
    fn default() -> Self {
        Self::new()
    }
}

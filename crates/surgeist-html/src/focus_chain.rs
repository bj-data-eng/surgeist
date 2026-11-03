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
    /// Read whether the original top-level traversable is nonnull.
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
    /// The current focus chain is established: either an empty list or the complete
    /// ordered focus chain. No reply is accepted afterward.
    Complete,
}
/// Observations and acknowledgments for the current focus chain cursor. Each
/// reply answers exactly the pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCurrentFocusChainReply {
    /// Whether the top-level traversable is nonnull. Null yields an empty chain.
    TraversablePresent(bool),
    /// Whether the traversable's currently focused area is nonnull. A known-null area
    /// yields an empty chain.
    CurrentlyFocusedAreaPresent(bool),
    /// The complete focus chain of the currently focused area was produced and its
    /// ordered result retained.
    FocusChainReady,
    /// The pending effect, establishing the empty output list, was fulfilled.
    Applied,
}
/// Why a current focus chain cursor refused a reply, queried through `is_*`
/// predicates. The cursor stays unchanged after a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCurrentFocusChainError {
    kind: HtmlCurrentFocusChainErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlCurrentFocusChainErrorKind {
    WrongReply,
    Terminal,
}

impl HtmlCurrentFocusChainErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlCurrentFocusChainError {
        HtmlCurrentFocusChainError { kind: self }
    }
}

impl From<HtmlCurrentFocusChainErrorKind> for HtmlCurrentFocusChainError {
    fn from(kind: HtmlCurrentFocusChainErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlCurrentFocusChainError {
    /// Whether the cause is: reply does not match the pending current focus chain action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlCurrentFocusChainErrorKind::WrongReply)
    }
    /// Whether the cause is: current focus chain is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlCurrentFocusChainErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlCurrentFocusChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlCurrentFocusChainErrorKind::WrongReply => {
                f.write_str("reply does not match the pending current focus chain action")
            }
            HtmlCurrentFocusChainErrorKind::Terminal => {
                f.write_str("current focus chain is already complete")
            }
        }
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
    /// Starts the current focus chain of a top-level traversable, which HTML 6.6.2
    /// (Data model) defines as the focus chain of its currently focused area, or an
    /// empty list for a null traversable. The first action reads the traversable.
    pub const fn new() -> Self {
        Self {
            pending: HtmlCurrentFocusChainAction::ReadTraversable,
        }
    }
    /// Returns the call or effect that must finish before the next reply.
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
            return Err(HtmlCurrentFocusChainErrorKind::Terminal.error());
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
            _ => return Err(HtmlCurrentFocusChainErrorKind::WrongReply.error()),
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
    /// Establish `output` as an empty list.
    InitializeEmptyChain,
    /// Retain the original qualified area-or-Document subject as currentObject.
    CaptureSubject,
    /// Append `currentObject` to `output`.
    AppendCurrentObject,
    /// Read whether `currentObject` is an area element's shape.
    ReadAreaShape,
    /// Append the shape's corresponding area Element, not its image DOM anchor.
    AppendCorrespondingAreaElement,
    /// Reached only for an actual area. Read whether its actual DOM anchor is
    /// an Element unequal to currentObject; never project a non-area Document.
    ReadAnchorIsDifferentElement,
    /// Append `currentObject`'s DOM anchor, an element other than `currentObject`
    /// itself, to `output`.
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
    /// The focus chain is complete. No reply is accepted afterward.
    Complete,
}
/// Observations and acknowledgments for the focus chain cursor. Each reply
/// answers exactly the pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusChainReply {
    /// Whether `currentObject` is an area element's shape.
    AreaShape(bool),
    /// Whether `currentObject`'s DOM anchor is an element other than `currentObject`.
    AnchorIsDifferentElement(bool),
    /// Whether `currentObject` is a focusable area.
    FocusableArea(bool),
    /// Whether the non-area `currentObject` is a Document.
    Document(bool),
    /// Whether the current Document's node navigable is nonnull.
    NodeNavigablePresent(bool),
    /// Whether the child navigable's parent is nonnull.
    NavigableParentPresent(bool),
    /// The pending capture or append effect was fulfilled.
    Applied,
}
/// Why a focus chain cursor refused a reply, queried through `is_*` predicates.
/// The cursor stays unchanged after a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlFocusChainError {
    kind: HtmlFocusChainErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlFocusChainErrorKind {
    WrongReply,
    /// An actual non-area currentObject is not a Document either.
    InvalidCurrentObject,
    Terminal,
}

impl HtmlFocusChainErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlFocusChainError {
        HtmlFocusChainError { kind: self }
    }
}

impl From<HtmlFocusChainErrorKind> for HtmlFocusChainError {
    fn from(kind: HtmlFocusChainErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlFocusChainError {
    /// Whether the cause is: reply does not match the pending focus chain action.
    pub fn is_wrong_reply(&self) -> bool {
        matches!(self.kind, HtmlFocusChainErrorKind::WrongReply)
    }
    /// Whether the cause is: focus chain object is neither an area nor a Document.
    pub fn is_invalid_current_object(&self) -> bool {
        matches!(self.kind, HtmlFocusChainErrorKind::InvalidCurrentObject)
    }
    /// Whether the cause is: focus chain is already complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, HtmlFocusChainErrorKind::Terminal)
    }
}

impl std::fmt::Display for HtmlFocusChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlFocusChainErrorKind::WrongReply => {
                f.write_str("reply does not match the pending focus chain action")
            }
            HtmlFocusChainErrorKind::InvalidCurrentObject => {
                f.write_str("focus chain object is neither an area nor a Document")
            }
            HtmlFocusChainErrorKind::Terminal => f.write_str("focus chain is already complete"),
        }
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
    /// Starts the focus chain of a focusable area subject (HTML 6.6.2, Data model)
    /// with the empty output list.
    pub const fn new() -> Self {
        Self {
            pending: HtmlFocusChainAction::InitializeEmptyChain,
        }
    }
    /// Returns the read or effect that must be fulfilled before the next reply.
    pub const fn action(&self) -> HtmlFocusChainAction {
        self.pending
    }
    /// Every invalid reply preserves the exact pending action and continuation.
    pub fn reply(&mut self, reply: HtmlFocusChainReply) -> Result<(), HtmlFocusChainError> {
        use HtmlFocusChainAction as A;
        use HtmlFocusChainReply as R;
        if self.pending == A::Complete {
            return Err(HtmlFocusChainErrorKind::Terminal.error());
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
                return Err(HtmlFocusChainErrorKind::InvalidCurrentObject.error());
            }
            (A::ReadNodeNavigable, R::NodeNavigablePresent(true)) => A::ReadNavigableParent,
            (A::ReadNodeNavigable, R::NodeNavigablePresent(false))
            | (A::ReadNavigableParent, R::NavigableParentPresent(false)) => A::Complete,
            (A::ReadNavigableParent, R::NavigableParentPresent(true)) => {
                A::CaptureChildNavigableContainerElement
            }
            _ => return Err(HtmlFocusChainErrorKind::WrongReply.error()),
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

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_current_focus_chain_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlCurrentFocusChainError) -> bool;
        let cases: Vec<(HtmlCurrentFocusChainError, &str, Predicate)> = vec![
            (
                HtmlCurrentFocusChainErrorKind::WrongReply.error(),
                "reply does not match the pending current focus chain action",
                HtmlCurrentFocusChainError::is_wrong_reply,
            ),
            (
                HtmlCurrentFocusChainErrorKind::Terminal.error(),
                "current focus chain is already complete",
                HtmlCurrentFocusChainError::is_terminal,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }

    #[test]
    fn html_focus_chain_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlFocusChainError) -> bool;
        let cases: Vec<(HtmlFocusChainError, &str, Predicate)> = vec![
            (
                HtmlFocusChainErrorKind::WrongReply.error(),
                "reply does not match the pending focus chain action",
                HtmlFocusChainError::is_wrong_reply,
            ),
            (
                HtmlFocusChainErrorKind::InvalidCurrentObject.error(),
                "focus chain object is neither an area nor a Document",
                HtmlFocusChainError::is_invalid_current_object,
            ),
            (
                HtmlFocusChainErrorKind::Terminal.error(),
                "focus chain is already complete",
                HtmlFocusChainError::is_terminal,
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

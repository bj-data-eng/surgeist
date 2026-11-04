//! HTML §5.2 exact token observations; URL, vocabulary and tree proof are separate.
use super::*;
use crate::internal::microsyntax::tokens::HtmlSpaceToken;
use crate::internal::values::finish::HtmlFinishFailure;

/// Which of the three microdata token-list attributes is being read (HTML877d #encoding-microdata).
/// The kind decides the attribute name and the nonemptiness and duplicate rules that apply.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenKind {
    /// The `itemtype` attribute (HTML877d #items): the item types of an item. The standard requires
    /// at least one token and no identical tokens.
    ItemType,
    /// The `itemprop` attribute (HTML877d #names:-the-itemprop-attribute): the property names an
    /// element adds. The standard requires at least one token and no identical tokens. Repeated
    /// tokens are skipped when processing.
    ItemProp,
    /// The `itemref` attribute (HTML877d #items): element IDs to crawl for additional properties.
    /// It may be empty, but identical tokens are still forbidden.
    ItemRef,
}
impl HtmlMicrodataTokenKind {
    /// The known attribute that this kind reads.
    pub fn attribute(self) -> KnownHtmlAttribute {
        match self {
            Self::ItemType => KnownHtmlAttribute::Itemtype,
            Self::ItemProp => KnownHtmlAttribute::Itemprop,
            Self::ItemRef => KnownHtmlAttribute::Itemref,
        }
    }
}
/// One occurrence in the original UTF-16 backing, not a normalized copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataToken {
    start: usize,
    end: usize,
    duplicate_of: Option<usize>,
    dot_or_colon: bool,
}
impl HtmlMicrodataToken {
    /// The offset of the token's first UTF-16 code unit in the raw value.
    pub fn start(&self) -> usize {
        self.start
    }
    /// The offset one past the token's last UTF-16 code unit in the raw value. A token is never
    /// empty, so `end` is greater than `start`.
    pub fn end(&self) -> usize {
        self.end
    }
    /// The index, among the report's tokens, of the first earlier token with identical code units,
    /// or `None` when this token is the first occurrence. Comparison is case-sensitive.
    pub fn duplicate_of(&self) -> Option<usize> {
        self.duplicate_of
    }
    /// Whether the token contains U+002E FULL STOP or U+003A COLON, which the standard excludes
    /// from non-URL property names (HTML877d #names:-the-itemprop-attribute). This is a fact about
    /// the code units, not a verdict on the token.
    pub fn contains_dot_or_colon(&self) -> bool {
        self.dot_or_colon
    }
}
/// The tokens of one microdata attribute, split on ASCII whitespace, with the exact raw value they
/// index.
///
/// An absent attribute has no raw value and no tokens. Reports are produced only by the interpreter
/// and the controlled interpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTokenReport {
    kind: HtmlMicrodataTokenKind,
    raw: Option<DocumentString>,
    tokens: Vec<HtmlMicrodataToken>,
}
impl HtmlMicrodataTokenReport {
    /// The attribute this report was read from.
    pub fn kind(&self) -> HtmlMicrodataTokenKind {
        self.kind
    }
    /// The exact raw value, or `None` when the attribute was absent. Every token offset indexes
    /// this value's UTF-16 code units.
    pub fn raw(&self) -> Option<&DocumentString> {
        self.raw.as_ref()
    }
    /// Every token in source order, duplicates included. A token that repeats an earlier one names
    /// that earlier token through `duplicate_of`.
    pub fn tokens(&self) -> &[HtmlMicrodataToken] {
        &self.tokens
    }
    /// The raw units the tokens index; empty exactly when no raw was present,
    /// in which case there are no tokens either.
    fn units(&self) -> &[u16] {
        self.raw.as_ref().map_or(&[], DocumentString::code_units)
    }
    /// The UTF-16 code units of the token at `index`, or `None` when `index` is not a token index.
    pub fn token_units(&self, index: usize) -> Option<&[u16]> {
        self.tokens
            .get(index)
            .map(|t| &self.units()[t.start..t.end])
    }
    /// The tokens that processing uses, in source order. For `itemprop` the standard keeps only the
    /// first occurrence of each name (HTML877d #names:-the-itemprop-attribute), so repeated tokens
    /// are skipped. For `itemtype` and `itemref` every token is yielded.
    pub fn processing_tokens(&self) -> impl Iterator<Item = &[u16]> {
        self.tokens
            .iter()
            .filter(|t| self.kind != HtmlMicrodataTokenKind::ItemProp || t.duplicate_of.is_none())
            .map(|t| &self.units()[t.start..t.end])
    }
    /// Local presence/nonempty/uniqueness only; contextual conformance is separate.
    pub fn local_author_conforming(&self) -> bool {
        self.raw.is_none()
            || ((self.kind == HtmlMicrodataTokenKind::ItemRef || !self.tokens.is_empty())
                && self.tokens.iter().all(|t| t.duplicate_of.is_none()))
    }
}
/// Why a token list cannot be built from the given tokens. The enum is `#[non_exhaustive]`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlMicrodataTokenListError {
    /// The kind requires at least one token (`itemtype` and `itemprop`) and none were given.
    Empty,
    /// Two tokens were identical in code units; the payload names the duplicate.
    Duplicate(crate::internal::microsyntax::tokens::HtmlDuplicateTokenError),
}
impl std::fmt::Display for HtmlMicrodataTokenListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("microdata token list contains no tokens"),
            Self::Duplicate(_) => f.write_str("microdata token list contains a duplicate token"),
        }
    }
}
impl From<crate::internal::microsyntax::tokens::HtmlDuplicateTokenError>
    for HtmlMicrodataTokenListError
{
    fn from(error: crate::internal::microsyntax::tokens::HtmlDuplicateTokenError) -> Self {
        Self::Duplicate(error)
    }
}
impl std::error::Error for HtmlMicrodataTokenListError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Empty => None,
            Self::Duplicate(error) => Some(error),
        }
    }
}
/// A unique native token list; type URLs and contextual author rules are unchecked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTokenList {
    kind: HtmlMicrodataTokenKind,
    tokens: crate::internal::microsyntax::tokens::HtmlUnorderedTokenSet,
}
impl HtmlMicrodataTokenList {
    /// Builds a list for `kind` from already-valid tokens.
    ///
    /// Returns [`HtmlMicrodataTokenListError::Empty`] when the kind is `ItemType` or `ItemProp` and
    /// `tokens` is empty. Returns [`HtmlMicrodataTokenListError::Duplicate`] when two tokens have
    /// identical code units; token comparison is case-sensitive.
    pub fn new(
        kind: HtmlMicrodataTokenKind,
        tokens: Vec<HtmlSpaceToken>,
    ) -> Result<Self, HtmlMicrodataTokenListError> {
        if kind != HtmlMicrodataTokenKind::ItemRef && tokens.is_empty() {
            return Err(HtmlMicrodataTokenListError::Empty);
        }
        let tokens = crate::internal::microsyntax::tokens::HtmlUnorderedTokenSet::new(
            crate::internal::microsyntax::tokens::HtmlTokenComparison::CaseSensitive,
            tokens,
        )?;
        Ok(Self { kind, tokens })
    }
    /// Ordinary allocation, literal single-space separators, no URL validation.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            self.kind.attribute().qualified_name(),
            self.tokens.as_document_string(),
        )
    }
}
impl HtmlValueInterpreter {
    /// Whole raw cap precedes scanning. Storage is reserved with the fallible allocator and no
    /// semantic work budget applies: a refused reservation returns
    /// [`HtmlValueInterpretationError::AllocationFailed`].
    pub fn microdata_tokens(
        &self,
        kind: HtmlMicrodataTokenKind,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlMicrodataTokenReport, HtmlValueInterpretationError> {
        self.admit_attribute_raw(raw)?;
        HtmlMicrodataTokenInterpretation::new(kind, raw).run_unmetered()
    }
}

/// The meter of the standalone operation. It admits all work and every reservation, so the only
/// refusals are the allocator's and the platform's, which it hands back as the engine's own error.
struct UnmeteredTokens;
impl HtmlMicrodataTokenMeter for UnmeteredTokens {
    type Error = HtmlMicrodataTokenInterpretationError;
    fn work(&mut self, _: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn reserve(
        &mut self,
        reservation: &mut HtmlMicrodataTokenReservation<'_>,
    ) -> Result<(), Self::Error> {
        reservation.try_reserve()
    }
}
/// The standalone operation's cause for an engine failure.
fn ordinary(error: HtmlMicrodataTokenInterpretationError) -> HtmlValueInterpretationError {
    HtmlValueInterpretationError::from_engine(
        error.is_allocation_failed(),
        error.is_representation_exhausted(),
    )
}
impl HtmlMicrodataTokenInterpretation<'_> {
    /// Runs the interpretation to its report with a meter that admits everything. This is the one
    /// driver of the standalone operation.
    fn run_unmetered(mut self) -> Result<HtmlMicrodataTokenReport, HtmlValueInterpretationError> {
        while let Some(action) = self.next_action().map_err(ordinary)? {
            action
                .advance(&mut UnmeteredTokens)
                .map_err(|error| match error {
                    HtmlMicrodataTokenActionError::Admission(error)
                    | HtmlMicrodataTokenActionError::Protocol(error) => ordinary(error),
                })?;
        }
        self.finish()
            .map_err(|failure| ordinary(failure.into_parts().1))
    }
}

use crate::internal::microsyntax::tokens::{HtmlSpaceTokenCursor, HtmlSpaceTokenStep};

/// A protocol refusal leaves the interpretation and its operands in place.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTokenInterpretationError {
    kind: HtmlMicrodataTokenInterpretationErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlMicrodataTokenInterpretationErrorKind {
    NotReady,
    Stopped,
    ReservationNotPerformed,
    ReservationAlreadyAttempted,
    RepresentationExhausted,
    AllocationFailed,
}

impl HtmlMicrodataTokenInterpretationErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlMicrodataTokenInterpretationError {
        HtmlMicrodataTokenInterpretationError { kind: self }
    }
}

impl From<HtmlMicrodataTokenInterpretationErrorKind> for HtmlMicrodataTokenInterpretationError {
    fn from(kind: HtmlMicrodataTokenInterpretationErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlMicrodataTokenInterpretationError {
    /// Whether the cause is: microdata token interpretation has an unfinished step.
    pub fn is_not_ready(&self) -> bool {
        matches!(
            self.kind,
            HtmlMicrodataTokenInterpretationErrorKind::NotReady
        )
    }
    /// Whether the cause is: microdata token interpretation was stopped.
    pub fn is_stopped(&self) -> bool {
        matches!(
            self.kind,
            HtmlMicrodataTokenInterpretationErrorKind::Stopped
        )
    }
    /// Whether the cause is: microdata token storage reservation was not performed.
    pub fn is_reservation_not_performed(&self) -> bool {
        matches!(
            self.kind,
            HtmlMicrodataTokenInterpretationErrorKind::ReservationNotPerformed
        )
    }
    /// Whether the cause is: microdata token storage reservation was already attempted.
    pub fn is_reservation_already_attempted(&self) -> bool {
        matches!(
            self.kind,
            HtmlMicrodataTokenInterpretationErrorKind::ReservationAlreadyAttempted
        )
    }
    /// Whether the cause is: microdata token interpretation size cannot be represented on this platform.
    pub fn is_representation_exhausted(&self) -> bool {
        matches!(
            self.kind,
            HtmlMicrodataTokenInterpretationErrorKind::RepresentationExhausted
        )
    }
    /// Whether the cause is: reserving microdata token storage failed.
    pub fn is_allocation_failed(&self) -> bool {
        matches!(
            self.kind,
            HtmlMicrodataTokenInterpretationErrorKind::AllocationFailed
        )
    }
}

impl std::fmt::Display for HtmlMicrodataTokenInterpretationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlMicrodataTokenInterpretationErrorKind::NotReady => {
                f.write_str("microdata token interpretation has an unfinished step")
            }
            HtmlMicrodataTokenInterpretationErrorKind::Stopped => {
                f.write_str("microdata token interpretation was stopped")
            }
            HtmlMicrodataTokenInterpretationErrorKind::ReservationNotPerformed => {
                f.write_str("microdata token storage reservation was not performed")
            }
            HtmlMicrodataTokenInterpretationErrorKind::ReservationAlreadyAttempted => {
                f.write_str("microdata token storage reservation was already attempted")
            }
            HtmlMicrodataTokenInterpretationErrorKind::RepresentationExhausted => f.write_str(
                "microdata token interpretation size cannot be represented on this platform",
            ),
            HtmlMicrodataTokenInterpretationErrorKind::AllocationFailed => {
                f.write_str("reserving microdata token storage failed")
            }
        }
    }
}

impl std::error::Error for HtmlMicrodataTokenInterpretationError {}

/// Execution preserves the caller's original admission error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenActionError<E> {
    /// The caller's meter refused the work or reservation; the payload is the meter's own error,
    /// unchanged.
    Admission(E),
    /// The caller drove the interpretation in a way the protocol forbids.
    Protocol(HtmlMicrodataTokenInterpretationError),
}
impl<E> From<HtmlMicrodataTokenInterpretationError> for HtmlMicrodataTokenActionError<E> {
    fn from(error: HtmlMicrodataTokenInterpretationError) -> Self {
        Self::Protocol(error)
    }
}
impl<E> std::fmt::Display for HtmlMicrodataTokenActionError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Admission(_) => {
                "microdata token interpretation was refused by the caller's admission"
            }
            Self::Protocol(_) => "microdata token interpretation protocol was violated",
        })
    }
}
impl<E: std::fmt::Debug + std::error::Error + 'static> std::error::Error
    for HtmlMicrodataTokenActionError<E>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            Self::Protocol(error) => Some(error),
        }
    }
}

/// The only reservation allowed by a pending token is room for one additional record, together
/// with the duplicate-index entry that record may need. A full vector grows geometrically, so the
/// room may exceed that one record; the meter admits and charges the capacity the vectors reach.
#[derive(Debug)]
pub struct HtmlMicrodataTokenReservation<'a> {
    values: &'a mut Vec<HtmlMicrodataToken>,
    index: &'a mut Vec<IndexNode>,
    attempted: bool,
    completed: bool,
}
impl HtmlMicrodataTokenReservation<'_> {
    /// The number of tokens committed so far.
    pub fn len(&self) -> usize {
        self.values.len()
    }
    /// Whether no token has been committed yet.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    /// The capacity of the unpublished token vector, in tokens.
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }
    /// The size in bytes of one token record, for charging the reservation: the published record
    /// plus the entry of the unpublished duplicate index that grows with it. The charge may count
    /// index slots that a refused index growth did not allocate (see
    /// [`try_reserve`](Self::try_reserve)). The index is released
    /// when the report is assembled or the interpretation is dropped, and is never published.
    pub fn record_size(&self) -> usize {
        std::mem::size_of::<HtmlMicrodataToken>() + std::mem::size_of::<IndexNode>()
    }
    /// The capacity of the token vector, in tokens, once this guard's reservation has run. The
    /// current capacity when one more token fits. Otherwise the greater of one more token than the
    /// vector holds and double its capacity (at least 4), so that `n` tokens request `O(log n)`
    /// reallocations. It depends only on the vector, never on a limit: a meter that charges for the
    /// growth admits this capacity before calling [`try_reserve`](Self::try_reserve), and refuses
    /// the reservation when it cannot. The duplicate index grows to the same capacity.
    pub fn target_capacity(&self) -> usize {
        crate::internal::amortized::amortized_capacity(self.values.len(), self.values.capacity())
    }
    /// Performs this guard's fixed request at most once, on its actual vectors, growing them to
    /// [`target_capacity`](Self::target_capacity) when one more token does not fit. The index grows
    /// to the token vector's capacity, so [`capacity`](Self::capacity) states both. When the
    /// allocator refuses a larger capacity for either vector, the guard requests the one entry that
    /// vector needs instead, so an allocator refusal of the spare room never fails where exact
    /// growth would succeed; the meter charges the capacity the vectors actually reach. The index
    /// capacity never exceeds the token vector's. The meter charges the token vector's capacity
    /// growth at [`record_size`](Self::record_size), which includes one index entry per token slot;
    /// when a refused index growth fell back to exact growth, the charge counts index slots that
    /// were not allocated, so it may exceed what was allocated and never falls short.
    pub fn try_reserve(&mut self) -> Result<(), HtmlMicrodataTokenInterpretationError> {
        if self.attempted {
            return Err(
                HtmlMicrodataTokenInterpretationErrorKind::ReservationAlreadyAttempted.error(),
            );
        }
        self.attempted = true;
        self.values
            .len()
            .checked_add(1)
            .ok_or(HtmlMicrodataTokenInterpretationErrorKind::RepresentationExhausted)?;
        crate::internal::amortized::reserve_amortized(self.values)
            .map_err(|_| HtmlMicrodataTokenInterpretationErrorKind::AllocationFailed)?;
        let target = self.values.capacity();
        if target > self.index.capacity() {
            // The index never holds more entries than there are tokens, so `target`, the token
            // vector's capacity after its own reservation, covers the entry the pending token may
            // add and a token fallback lowers it. A refused larger request falls back to that
            // entry through the shared helper, which is tested on its own: an index-only refusal
            // would need memory exhaustion between the two allocations and cannot be reproduced.
            crate::internal::amortized::reserve_to(self.index, target)
                .map_err(|_| HtmlMicrodataTokenInterpretationErrorKind::AllocationFailed)?;
        }
        self.completed = true;
        Ok(())
    }
}

/// Admission policy; HTML retains the data and does not own a work ledger.
pub trait HtmlMicrodataTokenMeter {
    /// The meter's refusal type, returned unchanged inside
    /// [`HtmlMicrodataTokenActionError::Admission`].
    type Error;
    /// Charges `units` of abstract work for the next step. Returning an error stops the
    /// interpretation before the step runs.
    fn work(&mut self, units: usize) -> Result<(), Self::Error>;
    /// Decides whether to admit the reservation of room for one more record, which may be larger
    /// ([`HtmlMicrodataTokenReservation::target_capacity`]). Admitting it means calling
    /// [`HtmlMicrodataTokenReservation::try_reserve`] exactly once; returning `Ok` without a
    /// successful reservation is a protocol violation, and returning an error stops the
    /// interpretation.
    fn reserve(
        &mut self,
        reservation: &mut HtmlMicrodataTokenReservation<'_>,
    ) -> Result<(), Self::Error>;
}

/// What the next action needs from the meter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenActionKind {
    /// The step must be charged this many units of work before it runs.
    Work(usize),
    /// The step stores one more token record and needs a reservation decision from the meter.
    Reserve,
}
/// The result of one executed action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenStep {
    /// The interpretation advanced inside a token or between tokens; no token was added.
    Continue,
    /// One more token record was committed to the unpublished vector.
    TokenCommitted,
    /// The report is assembled; collect it with `finish`.
    Complete,
}

/// Every token is read as a string of 17-bit symbols: a present unit is `0x1_0000 | unit`, and every
/// position at or beyond the token's end is `0`. Equal bit strings are exactly equal tokens, so
/// `a` and `a\0` differ, and a token that is a prefix of another differs from it at the first
/// position the shorter one lacks.
const PRESENT: u32 = 0x1_0000;

/// A bit of the symbol string: the code-unit index and the bit within its symbol, counted from the
/// most significant bit. The derived order is the order of the bits in the string.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BitPosition {
    unit: usize,
    bit: u8,
}

/// The first bit at which two symbols differ, or `None` when they are equal.
fn first_difference(a: u32, b: u32) -> Option<u8> {
    let differing = a ^ b;
    if differing == 0 {
        return None;
    }
    // Symbols have 17 bits, so the count is at least 15 and the result is at most 16.
    u8::try_from(differing.leading_zeros().saturating_sub(15)).ok()
}

/// A subtree of the duplicate index: an internal node or the leaf for one distinct token.
#[derive(Clone, Copy, Debug)]
enum Child {
    Node(usize),
    Leaf(usize),
}

/// Where a subtree hangs: the root, or one side of an internal node.
#[derive(Clone, Copy, Debug)]
enum Link {
    Root,
    Side(usize, usize),
}

/// An internal node of the duplicate index, a path-compressed binary trie over the tokens' symbol
/// strings. Every token below the node agrees on all bits before `pos`, and differs there between
/// the two children. Leaves are the first occurrences of the distinct tokens.
#[derive(Clone, Copy, Debug)]
struct IndexNode {
    pos: BitPosition,
    /// Any token below this node, which stands for the bits the node's subtree shares.
    member: usize,
    children: [Child; 2],
}

/// How the pending token joins the duplicate index when it is committed.
#[derive(Clone, Copy, Debug)]
enum Placement {
    /// The index is empty: the token becomes its only leaf.
    Root,
    /// The token differs from the subtree `at` first at `crit`, and takes `side` of a new node
    /// there.
    Split {
        crit: BitPosition,
        link: Link,
        at: Child,
        side: usize,
    },
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Begin,
    Header,
    Scan,
    /// One step of the duplicate search: compare the pending token with `at` from code unit `unit`.
    Walk {
        link: Link,
        at: Child,
        unit: usize,
    },
    Punctuation(usize),
    Reserve,
    Commit,
    Assemble,
    Ready,
    Stopped,
}

/// One original raw operand, its scan state, and its actual unpublished vector.
#[derive(Debug)]
pub struct HtmlMicrodataTokenInterpretation<'raw> {
    kind: HtmlMicrodataTokenKind,
    raw: Option<&'raw DocumentString>,
    /// The scanned units: the raw's, or empty when no raw is present, in which
    /// case no scanning phase is ever issued.
    units: &'raw [u16],
    cursor: HtmlSpaceTokenCursor<'raw>,
    tokens: Vec<HtmlMicrodataToken>,
    /// Scratch for duplicate detection: the internal nodes of an index over the distinct tokens
    /// committed so far. It is dropped when the report is assembled and never published.
    index: Vec<IndexNode>,
    root: Option<Child>,
    start: usize,
    end: usize,
    duplicate_of: Option<usize>,
    placement: Placement,
    dot_or_colon: bool,
    phase: Phase,
    report: Option<HtmlMicrodataTokenReport>,
}
impl<'raw> HtmlMicrodataTokenInterpretation<'raw> {
    /// Allocation-free setup. The consumer admits the original raw before driving.
    pub fn new(kind: HtmlMicrodataTokenKind, raw: Option<&'raw DocumentString>) -> Self {
        let units = raw.map_or(&[][..], DocumentString::code_units);
        Self {
            kind,
            raw,
            units,
            cursor: HtmlSpaceTokenCursor::new(units),
            tokens: Vec::new(),
            index: Vec::new(),
            root: None,
            start: 0,
            end: 0,
            duplicate_of: None,
            placement: Placement::Root,
            dot_or_colon: false,
            phase: if raw.is_some() {
                Phase::Begin
            } else {
                Phase::Assemble
            },
            report: None,
        }
    }
    /// Abandons the interpretation. Later calls to `next_action` and `finish` report the
    /// stopped cause, and the partial tokens are never published.
    pub fn stop(&mut self) {
        self.phase = Phase::Stopped;
    }
    /// Gives back the attribute kind and the raw operand, discarding any partial tokens.
    pub fn abort(self) -> (HtmlMicrodataTokenKind, Option<&'raw DocumentString>) {
        (self.kind, self.raw)
    }
    /// Returns the next action, `None` when the report is ready, or the stopped cause after `stop`.
    /// The returned action borrows the interpretation exclusively, so it cannot be replayed or
    /// paired with another operand.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlMicrodataTokenAction<'_, 'raw>>, HtmlMicrodataTokenInterpretationError>
    {
        match self.phase {
            Phase::Stopped => Err(HtmlMicrodataTokenInterpretationErrorKind::Stopped.error()),
            Phase::Ready => Ok(None),
            _ => Ok(Some(HtmlMicrodataTokenAction { owner: self })),
        }
    }
    /// Consumes the interpretation and returns its finished report.
    ///
    /// A stopped interpretation is refused with the stopped cause and one with actions remaining
    /// with the not-ready cause. Either refusal hands the interpretation back unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn finish(
        mut self,
    ) -> Result<
        HtmlMicrodataTokenReport,
        HtmlFinishFailure<Self, HtmlMicrodataTokenInterpretationError>,
    > {
        match self.phase {
            Phase::Ready => match self.report.take() {
                Some(report) => Ok(report),
                None => Err(HtmlFinishFailure::new(
                    self,
                    HtmlMicrodataTokenInterpretationErrorKind::NotReady,
                )),
            },
            Phase::Stopped => Err(HtmlFinishFailure::new(
                self,
                HtmlMicrodataTokenInterpretationErrorKind::Stopped,
            )),
            _ => Err(HtmlFinishFailure::new(
                self,
                HtmlMicrodataTokenInterpretationErrorKind::NotReady,
            )),
        }
    }
    /// The symbol at `index` of the token occupying `start..end`.
    fn symbol(&self, start: usize, end: usize, index: usize) -> u32 {
        match start.checked_add(index) {
            Some(at) if at < end => self
                .units
                .get(at)
                .map_or(0, |&unit| PRESENT | u32::from(unit)),
            _ => 0,
        }
    }
    /// The side of a node at `pos` that the pending token takes: its bit there.
    fn side_at(&self, pos: BitPosition) -> usize {
        let symbol = self.symbol(self.start, self.end, pos.unit);
        let mask = PRESENT.checked_shr(u32::from(pos.bit)).unwrap_or(0);
        usize::from(symbol & mask != 0)
    }
    /// The first occurrence of the distinct token with record `token`, as its range.
    fn span(&self, token: usize) -> (usize, usize) {
        let record = &self.tokens[token];
        (record.start, record.end)
    }
    /// Begins the duplicate search for the token just scanned.
    fn begin_search(&mut self) {
        self.duplicate_of = None;
        self.placement = Placement::Root;
        self.phase = match self.root {
            Some(at) => Phase::Walk {
                link: Link::Root,
                at,
                unit: 0,
            },
            None => Phase::Punctuation(0),
        };
    }
    /// The pending token first differs from the subtree `at` at `crit`.
    fn split_at(&mut self, crit: BitPosition, link: Link, at: Child) {
        self.placement = Placement::Split {
            crit,
            link,
            at,
            side: self.side_at(crit),
        };
        self.phase = Phase::Punctuation(0);
    }
    /// One bounded step of the duplicate search. It compares one code unit of the pending token
    /// with the same unit of a token below `at`, and at a node whose bit it has reached it also
    /// descends one level. The shared bits of an edge are verified before the node's own bit
    /// picks a side, so the search never passes a bit the pending token does not have: it ends
    /// at or before the first position the token lacks, and descends at most 17 levels per code
    /// unit of the token. Its cost follows the pending token's length, not the index's size.
    fn walk(&mut self, link: Link, at: Child, unit: usize) {
        let here = self.symbol(self.start, self.end, unit);
        match at {
            Child::Leaf(token) => {
                let (start, end) = self.span(token);
                match first_difference(here, self.symbol(start, end, unit)) {
                    Some(bit) => self.split_at(BitPosition { unit, bit }, link, at),
                    None if unit + 1 >= (self.end - self.start).max(end - start) => {
                        self.duplicate_of = Some(token);
                        self.phase = Phase::Punctuation(0);
                    }
                    None => {
                        self.phase = Phase::Walk {
                            link,
                            at,
                            unit: unit + 1,
                        }
                    }
                }
            }
            Child::Node(node) => {
                let IndexNode {
                    pos,
                    member,
                    children,
                } = self.index[node];
                let (start, end) = self.span(member);
                let difference = first_difference(here, self.symbol(start, end, unit));
                if unit < pos.unit {
                    match difference {
                        Some(bit) => self.split_at(BitPosition { unit, bit }, link, at),
                        None => {
                            self.phase = Phase::Walk {
                                link,
                                at,
                                unit: unit + 1,
                            }
                        }
                    }
                } else {
                    match difference {
                        Some(bit) if bit < pos.bit => {
                            self.split_at(BitPosition { unit, bit }, link, at);
                        }
                        _ => {
                            let side = self.side_at(pos);
                            self.phase = Phase::Walk {
                                link: Link::Side(node, side),
                                at: children[side],
                                unit: pos.unit,
                            };
                        }
                    }
                }
            }
        }
    }
    /// Adds the committed distinct token `token` to the duplicate index.
    fn index_token(&mut self, token: usize) {
        let leaf = Child::Leaf(token);
        let (link, subtree) = match self.placement {
            Placement::Root => (Link::Root, leaf),
            Placement::Split {
                crit,
                link,
                at,
                side,
            } => {
                let node = self.index.len();
                self.index.push(IndexNode {
                    pos: crit,
                    member: token,
                    children: if side == 0 { [leaf, at] } else { [at, leaf] },
                });
                (link, Child::Node(node))
            }
        };
        match link {
            Link::Root => self.root = Some(subtree),
            Link::Side(parent, side) => self.index[parent].children[side] = subtree,
        }
    }
}

/// An exclusive action cannot be replayed or supplied with a different operand.
#[derive(Debug)]
pub struct HtmlMicrodataTokenAction<'owner, 'raw> {
    owner: &'owner mut HtmlMicrodataTokenInterpretation<'raw>,
}
impl HtmlMicrodataTokenAction<'_, '_> {
    /// What this action needs from the meter, decided by the interpretation's current phase.
    pub fn kind(&self) -> HtmlMicrodataTokenActionKind {
        use HtmlMicrodataTokenActionKind::{Reserve, Work};
        match self.owner.phase {
            Phase::Begin => Work(HtmlSpaceTokenCursor::BEGIN_WORK),
            Phase::Scan => Work(if self.owner.cursor.inspects_unit() {
                HtmlSpaceTokenCursor::STEP_WORK
            } else {
                2
            }),
            // One unit of each operand and at most one index node.
            Phase::Walk { .. } => Work(3),
            Phase::Reserve => Reserve,
            _ => Work(1),
        }
    }
    /// Runs the action against `meter`.
    ///
    /// A meter refusal stops the interpretation and returns it as `Admission`. A meter that returns
    /// `Ok` without a successful reservation yields a `Protocol` error carrying the
    /// not-performed cause; if it had attempted the reservation, the interpretation also stops.
    /// On success it returns the step taken.
    pub fn advance<M: HtmlMicrodataTokenMeter>(
        self,
        meter: &mut M,
    ) -> Result<HtmlMicrodataTokenStep, HtmlMicrodataTokenActionError<M::Error>> {
        let kind = self.kind();
        let o = self.owner;
        let HtmlMicrodataTokenActionKind::Work(units) = kind else {
            let mut guard = HtmlMicrodataTokenReservation {
                values: &mut o.tokens,
                index: &mut o.index,
                attempted: false,
                completed: false,
            };
            if let Err(error) = meter.reserve(&mut guard) {
                o.phase = Phase::Stopped;
                return Err(HtmlMicrodataTokenActionError::Admission(error));
            }
            if !guard.completed {
                if guard.attempted {
                    o.phase = Phase::Stopped;
                }
                return Err(HtmlMicrodataTokenActionError::Protocol(
                    HtmlMicrodataTokenInterpretationErrorKind::ReservationNotPerformed.error(),
                ));
            }
            o.phase = Phase::Commit;
            return Ok(HtmlMicrodataTokenStep::Continue);
        };
        if let Err(error) = meter.work(units) {
            o.phase = Phase::Stopped;
            return Err(HtmlMicrodataTokenActionError::Admission(error));
        }
        let mut result = HtmlMicrodataTokenStep::Continue;
        match o.phase {
            Phase::Begin => o.phase = Phase::Header,
            Phase::Header => o.phase = Phase::Scan,
            Phase::Scan => match o.cursor.step() {
                HtmlSpaceTokenStep::Continue => o.phase = Phase::Header,
                HtmlSpaceTokenStep::Done => o.phase = Phase::Assemble,
                HtmlSpaceTokenStep::Token { start, end } => {
                    o.start = start;
                    o.end = end;
                    o.dot_or_colon = false;
                    o.begin_search();
                }
            },
            Phase::Walk { link, at, unit } => o.walk(link, at, unit),
            Phase::Punctuation(offset) => {
                o.dot_or_colon |= matches!(o.units[o.start + offset], 0x2e | 0x3a);
                o.phase = if o.start + offset + 1 == o.end {
                    Phase::Reserve
                } else {
                    Phase::Punctuation(offset + 1)
                };
            }
            Phase::Commit => {
                if o.duplicate_of.is_none() {
                    o.index_token(o.tokens.len());
                }
                o.tokens.push(HtmlMicrodataToken {
                    start: o.start,
                    end: o.end,
                    duplicate_of: o.duplicate_of,
                    dot_or_colon: o.dot_or_colon,
                });
                o.phase = Phase::Header;
                result = HtmlMicrodataTokenStep::TokenCommitted;
            }
            Phase::Assemble => {
                // Release the scratch index before publishing.
                o.index = Vec::new();
                o.root = None;
                o.report = Some(HtmlMicrodataTokenReport {
                    kind: o.kind,
                    raw: o.raw.cloned(),
                    tokens: std::mem::take(&mut o.tokens),
                });
                o.phase = Phase::Ready;
                result = HtmlMicrodataTokenStep::Complete;
            }
            _ => unreachable!("action only issued for pending phase"),
        }
        Ok(result)
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    /// Every cause states its documented meaning, and exactly one of the
    /// cause predicates, its own, recognizes it.
    #[test]
    fn every_cause_has_its_message_and_exactly_one_matching_predicate() {
        type Predicate = fn(&HtmlMicrodataTokenInterpretationError) -> bool;
        let cases: Vec<(HtmlMicrodataTokenInterpretationError, &str, Predicate)> = vec![
            (
                HtmlMicrodataTokenInterpretationErrorKind::NotReady.error(),
                "microdata token interpretation has an unfinished step",
                HtmlMicrodataTokenInterpretationError::is_not_ready,
            ),
            (
                HtmlMicrodataTokenInterpretationErrorKind::Stopped.error(),
                "microdata token interpretation was stopped",
                HtmlMicrodataTokenInterpretationError::is_stopped,
            ),
            (
                HtmlMicrodataTokenInterpretationErrorKind::ReservationNotPerformed.error(),
                "microdata token storage reservation was not performed",
                HtmlMicrodataTokenInterpretationError::is_reservation_not_performed,
            ),
            (
                HtmlMicrodataTokenInterpretationErrorKind::ReservationAlreadyAttempted.error(),
                "microdata token storage reservation was already attempted",
                HtmlMicrodataTokenInterpretationError::is_reservation_already_attempted,
            ),
            (
                HtmlMicrodataTokenInterpretationErrorKind::RepresentationExhausted.error(),
                "microdata token interpretation size cannot be represented on this platform",
                HtmlMicrodataTokenInterpretationError::is_representation_exhausted,
            ),
            (
                HtmlMicrodataTokenInterpretationErrorKind::AllocationFailed.error(),
                "reserving microdata token storage failed",
                HtmlMicrodataTokenInterpretationError::is_allocation_failed,
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

#[cfg(test)]
mod ordinary_allocation_tests {
    use super::*;
    use crate::internal::amortized::fault;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    /// A refused allocation is an expected condition of a fallible allocator, so the standalone
    /// operation reports it as its error. It must neither abort the caller by panicking nor claim
    /// a report for storage it could not reserve.
    #[test]
    fn a_refused_reservation_is_an_error_and_never_a_panic() {
        let interpreter = HtmlValueInterpreter::new(1_000);
        let raw = DocumentString::from("a b c d e f g h i j");
        let counting = fault::count_only();
        let baseline = interpreter
            .microdata_tokens(HtmlMicrodataTokenKind::ItemProp, Some(&raw))
            .unwrap();
        let requests = fault::requests();
        drop(counting);
        assert_eq!(baseline.tokens().len(), 10);
        // Ten tokens grow the vectors from empty: more than one reservation, so a refusal can
        // arrive after earlier ones succeeded.
        assert!(requests >= 2, "{requests} reservation requests");
        for refuse_from in 1..=requests {
            let _armed = fault::arm(refuse_from);
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                interpreter.microdata_tokens(HtmlMicrodataTokenKind::ItemProp, Some(&raw))
            }));
            let verdict = match &outcome {
                Err(_) => "panicked",
                Ok(Ok(_)) => "reported success",
                Ok(Err(HtmlValueInterpretationError::AllocationFailed)) => {
                    "returned the allocation error"
                }
                Ok(Err(_)) => "returned another error",
            };
            assert_eq!(
                verdict, "returned the allocation error",
                "reservation {refuse_from} of {requests} refused"
            );
        }
        let _armed = fault::arm(requests + 1);
        let report = interpreter
            .microdata_tokens(HtmlMicrodataTokenKind::ItemProp, Some(&raw))
            .unwrap();
        assert_eq!(report, baseline);
    }
}

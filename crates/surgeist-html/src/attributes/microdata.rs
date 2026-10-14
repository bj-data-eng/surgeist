//! HTML §5.2 exact token observations; URL, vocabulary and tree proof are separate.
use super::*;
use crate::HtmlSpaceToken;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenKind {
    ItemType,
    ItemProp,
    ItemRef,
}
impl HtmlMicrodataTokenKind {
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
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
    pub fn duplicate_of(&self) -> Option<usize> {
        self.duplicate_of
    }
    pub fn contains_dot_or_colon(&self) -> bool {
        self.dot_or_colon
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTokenReport {
    kind: HtmlMicrodataTokenKind,
    raw: Option<DocumentString>,
    tokens: Vec<HtmlMicrodataToken>,
}
impl HtmlMicrodataTokenReport {
    pub fn kind(&self) -> HtmlMicrodataTokenKind {
        self.kind
    }
    pub fn raw(&self) -> Option<&DocumentString> {
        self.raw.as_ref()
    }
    pub fn tokens(&self) -> &[HtmlMicrodataToken] {
        &self.tokens
    }
    pub fn token_units(&self, index: usize) -> Option<&[u16]> {
        self.tokens
            .get(index)
            .map(|t| &self.raw.as_ref().expect("token has raw").code_units()[t.start..t.end])
    }
    pub fn processing_tokens(&self) -> impl Iterator<Item = &[u16]> {
        self.tokens
            .iter()
            .filter(|t| self.kind != HtmlMicrodataTokenKind::ItemProp || t.duplicate_of.is_none())
            .map(|t| &self.raw.as_ref().expect("token has raw").code_units()[t.start..t.end])
    }
    /// Local presence/nonempty/uniqueness only; contextual conformance is separate.
    pub fn local_author_conforming(&self) -> bool {
        self.raw.is_none()
            || ((self.kind == HtmlMicrodataTokenKind::ItemRef || !self.tokens.is_empty())
                && self.tokens.iter().all(|t| t.duplicate_of.is_none()))
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenListError {
    Empty,
    Duplicate(crate::HtmlDuplicateTokenError),
}
impl std::fmt::Display for HtmlMicrodataTokenListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HtmlMicrodataTokenListError {}
/// A unique native token list; type URLs and contextual author rules are unchecked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTokenList {
    kind: HtmlMicrodataTokenKind,
    tokens: crate::HtmlUnorderedTokenSet,
}
impl HtmlMicrodataTokenList {
    pub fn new(
        kind: HtmlMicrodataTokenKind,
        tokens: Vec<HtmlSpaceToken>,
    ) -> Result<Self, HtmlMicrodataTokenListError> {
        if kind != HtmlMicrodataTokenKind::ItemRef && tokens.is_empty() {
            return Err(HtmlMicrodataTokenListError::Empty);
        }
        let tokens =
            crate::HtmlUnorderedTokenSet::new(crate::HtmlTokenComparison::CaseSensitive, tokens)
                .map_err(HtmlMicrodataTokenListError::Duplicate)?;
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
    /// Whole raw cap precedes scanning. Ordinary allocation; no semantic work budget.
    pub fn microdata_tokens(
        &self,
        kind: HtmlMicrodataTokenKind,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlMicrodataTokenReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw)?;
        use crate::bounded::{
            HtmlMicrodataTokenInterpretation, HtmlMicrodataTokenMeter,
            HtmlMicrodataTokenReservation,
        };
        struct Ordinary;
        impl HtmlMicrodataTokenMeter for Ordinary {
            type Error = std::convert::Infallible;
            fn work(&mut self, _: usize) -> Result<(), Self::Error> {
                Ok(())
            }
            fn reserve(
                &mut self,
                reservation: &mut HtmlMicrodataTokenReservation<'_>,
            ) -> Result<(), Self::Error> {
                reservation
                    .try_reserve()
                    .expect("ordinary microdata allocation");
                Ok(())
            }
        }
        let mut owner = Some(HtmlMicrodataTokenInterpretation::new(kind, raw));
        while let Some(action) = owner
            .as_mut()
            .expect("retained owner")
            .next_action()
            .expect("ordinary protocol")
        {
            action
                .advance(&mut Ordinary)
                .expect("ordinary microdata execution");
        }
        Ok(HtmlMicrodataTokenInterpretation::try_finish(&mut owner)
            .expect("completed ordinary interpretation"))
    }
}

use crate::tokens::{SpaceTokenCursor, SpaceTokenStep};

/// A protocol refusal leaves the interpretation and its operands in place.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlMicrodataTokenInterpretationError {
    Consumed,
    NotReady,
    Stopped,
    ReservationNotPerformed,
    ReservationAlreadyAttempted,
    RepresentationExhausted,
    AllocationFailed,
}

/// Execution preserves the caller's original admission error.
#[derive(Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenActionError<E> {
    Admission(E),
    Protocol(HtmlMicrodataTokenInterpretationError),
}

/// The only reservation allowed by a pending token is one additional record.
pub struct HtmlMicrodataTokenReservation<'a> {
    values: &'a mut Vec<HtmlMicrodataToken>,
    attempted: bool,
    completed: bool,
}
impl HtmlMicrodataTokenReservation<'_> {
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }
    pub fn record_size(&self) -> usize {
        std::mem::size_of::<HtmlMicrodataToken>()
    }
    /// Performs this guard's fixed request at most once, on its actual vector.
    pub fn try_reserve(&mut self) -> Result<(), HtmlMicrodataTokenInterpretationError> {
        if self.attempted {
            return Err(HtmlMicrodataTokenInterpretationError::ReservationAlreadyAttempted);
        }
        self.attempted = true;
        let needed = self
            .values
            .len()
            .checked_add(1)
            .ok_or(HtmlMicrodataTokenInterpretationError::RepresentationExhausted)?;
        if needed > self.values.capacity() {
            self.values
                .try_reserve_exact(1)
                .map_err(|_| HtmlMicrodataTokenInterpretationError::AllocationFailed)?;
        }
        self.completed = true;
        Ok(())
    }
}

/// Admission policy; HTML retains the data and does not own a work ledger.
pub trait HtmlMicrodataTokenMeter {
    type Error;
    fn work(&mut self, units: usize) -> Result<(), Self::Error>;
    fn reserve(
        &mut self,
        reservation: &mut HtmlMicrodataTokenReservation<'_>,
    ) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenActionKind {
    Work(usize),
    Reserve,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenStep {
    Continue,
    TokenCommitted,
    Complete,
}

#[derive(Clone, Copy)]
enum Phase {
    Begin,
    Header,
    Scan,
    PriorVisit(usize),
    PriorLength(usize),
    PriorUnit(usize, usize),
    Punctuation(usize),
    Reserve,
    Commit,
    Assemble,
    Ready,
    Stopped,
}

/// One original raw operand, its scan state, and its actual unpublished vector.
pub struct HtmlMicrodataTokenInterpretation<'raw> {
    kind: HtmlMicrodataTokenKind,
    raw: Option<&'raw DocumentString>,
    cursor: Option<SpaceTokenCursor<'raw>>,
    tokens: Vec<HtmlMicrodataToken>,
    start: usize,
    end: usize,
    duplicate_of: Option<usize>,
    dot_or_colon: bool,
    phase: Phase,
    report: Option<HtmlMicrodataTokenReport>,
}
impl<'raw> HtmlMicrodataTokenInterpretation<'raw> {
    /// Allocation-free setup. The consumer admits the original raw before driving.
    pub fn new(kind: HtmlMicrodataTokenKind, raw: Option<&'raw DocumentString>) -> Self {
        Self {
            kind,
            raw,
            cursor: None,
            tokens: Vec::new(),
            start: 0,
            end: 0,
            duplicate_of: None,
            dot_or_colon: false,
            phase: if raw.is_some() {
                Phase::Begin
            } else {
                Phase::Assemble
            },
            report: None,
        }
    }
    pub fn stop(&mut self) {
        self.phase = Phase::Stopped;
    }
    pub fn abort(self) -> (HtmlMicrodataTokenKind, Option<&'raw DocumentString>) {
        (self.kind, self.raw)
    }
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlMicrodataTokenAction<'_, 'raw>>, HtmlMicrodataTokenInterpretationError>
    {
        match self.phase {
            Phase::Stopped => Err(HtmlMicrodataTokenInterpretationError::Stopped),
            Phase::Ready => Ok(None),
            _ => Ok(Some(HtmlMicrodataTokenAction { owner: self })),
        }
    }
    pub fn try_finish(
        slot: &mut Option<Self>,
    ) -> Result<HtmlMicrodataTokenReport, HtmlMicrodataTokenInterpretationError> {
        let owner = slot
            .as_ref()
            .ok_or(HtmlMicrodataTokenInterpretationError::Consumed)?;
        match owner.phase {
            Phase::Ready => Ok(slot
                .take()
                .expect("checked owner")
                .report
                .expect("assembled report")),
            Phase::Stopped => Err(HtmlMicrodataTokenInterpretationError::Stopped),
            _ => Err(HtmlMicrodataTokenInterpretationError::NotReady),
        }
    }
    fn next_prior(&mut self, index: usize) {
        self.phase = if index < self.tokens.len() {
            Phase::PriorVisit(index)
        } else {
            Phase::Punctuation(0)
        };
    }
}

/// An exclusive action cannot be replayed or supplied with a different operand.
pub struct HtmlMicrodataTokenAction<'owner, 'raw> {
    owner: &'owner mut HtmlMicrodataTokenInterpretation<'raw>,
}
impl HtmlMicrodataTokenAction<'_, '_> {
    pub fn kind(&self) -> HtmlMicrodataTokenActionKind {
        use HtmlMicrodataTokenActionKind::{Reserve, Work};
        match self.owner.phase {
            Phase::Begin => Work(SpaceTokenCursor::BEGIN_WORK),
            Phase::Scan => Work(
                if self.owner.cursor.as_ref().expect("begun").inspects_unit() {
                    SpaceTokenCursor::STEP_WORK
                } else {
                    2
                },
            ),
            Phase::PriorUnit(_, _) => Work(2),
            Phase::Reserve => Reserve,
            _ => Work(1),
        }
    }
    pub fn advance<M: HtmlMicrodataTokenMeter>(
        self,
        meter: &mut M,
    ) -> Result<HtmlMicrodataTokenStep, HtmlMicrodataTokenActionError<M::Error>> {
        let kind = self.kind();
        let o = self.owner;
        if kind == HtmlMicrodataTokenActionKind::Reserve {
            let mut guard = HtmlMicrodataTokenReservation {
                values: &mut o.tokens,
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
                    HtmlMicrodataTokenInterpretationError::ReservationNotPerformed,
                ));
            }
            o.phase = Phase::Commit;
            return Ok(HtmlMicrodataTokenStep::Continue);
        }
        let HtmlMicrodataTokenActionKind::Work(units) = kind else {
            unreachable!()
        };
        if let Err(error) = meter.work(units) {
            o.phase = Phase::Stopped;
            return Err(HtmlMicrodataTokenActionError::Admission(error));
        }
        let mut result = HtmlMicrodataTokenStep::Continue;
        match o.phase {
            Phase::Begin => {
                o.cursor = Some(SpaceTokenCursor::new(o.raw.expect("present").code_units()));
                o.phase = Phase::Header;
            }
            Phase::Header => o.phase = Phase::Scan,
            Phase::Scan => match o.cursor.as_mut().expect("begun").step() {
                SpaceTokenStep::Continue => o.phase = Phase::Header,
                SpaceTokenStep::Done => o.phase = Phase::Assemble,
                SpaceTokenStep::Token { start, end } => {
                    o.start = start;
                    o.end = end;
                    o.duplicate_of = None;
                    o.dot_or_colon = false;
                    o.next_prior(0);
                }
            },
            Phase::PriorVisit(index) => o.phase = Phase::PriorLength(index),
            Phase::PriorLength(index) => {
                let prior = &o.tokens[index];
                if prior.end - prior.start == o.end - o.start {
                    o.phase = Phase::PriorUnit(index, 0);
                } else {
                    o.next_prior(index + 1);
                }
            }
            Phase::PriorUnit(index, offset) => {
                let raw = o.raw.expect("present").code_units();
                if raw[o.tokens[index].start + offset] != raw[o.start + offset] {
                    o.next_prior(index + 1);
                } else if offset + 1 == o.end - o.start {
                    o.duplicate_of = Some(index);
                    o.phase = Phase::Punctuation(0);
                } else {
                    o.phase = Phase::PriorUnit(index, offset + 1);
                }
            }
            Phase::Punctuation(offset) => {
                o.dot_or_colon |= matches!(
                    o.raw.expect("present").code_units()[o.start + offset],
                    0x2e | 0x3a
                );
                o.phase = if o.start + offset + 1 == o.end {
                    Phase::Reserve
                } else {
                    Phase::Punctuation(offset + 1)
                };
            }
            Phase::Commit => {
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

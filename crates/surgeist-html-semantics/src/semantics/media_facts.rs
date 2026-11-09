//! Concrete CSS producer attestations for one exact captured HTML media field.
use super::admission::{FactAdmission, FactError, fact_limits, producer_revision};
use super::index::FactIndex;
use super::scan::ScanError;
use super::*;
use surgeist_html::bounded::media::{HtmlMediaField, media_field_matches};
use surgeist_html::values::DocumentString;

identity!(HtmlMediaProducerId, fresh);

producer_revision!(HtmlMediaGrammarRevision, HtmlMediaProducerId);
producer_revision!(HtmlMediaEnvironmentRevision, HtmlMediaProducerId);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMediaContextRevision {
    grammar: HtmlMediaGrammarRevision,
    environment: HtmlMediaEnvironmentRevision,
}
impl HtmlMediaContextRevision {
    pub fn new(
        grammar: HtmlMediaGrammarRevision,
        environment: HtmlMediaEnvironmentRevision,
    ) -> Self {
        Self {
            grammar,
            environment,
        }
    }
    pub fn grammar(&self) -> &HtmlMediaGrammarRevision {
        &self.grammar
    }
    pub fn environment(&self) -> &HtmlMediaEnvironmentRevision {
        &self.environment
    }
}

/// Exact immutable capture/subject/attribute-position key, never a text alias.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMediaAttributeInput {
    subject: HtmlCaptureSubjectId,
    ordinal: usize,
}
impl HtmlMediaAttributeInput {
    /// Ordinary fixed-name validation; does not parse or copy the original value.
    pub fn new(
        capture: &HtmlCapture,
        subject: HtmlCaptureSubjectId,
        ordinal: usize,
    ) -> Result<Self, HtmlMediaFactsError> {
        let key = Self { subject, ordinal };
        key.raw(capture)?;
        Ok(key)
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn raw<'a>(
        &self,
        capture: &'a HtmlCapture,
    ) -> Result<&'a DocumentString, HtmlMediaFactsError> {
        let node = capture
            .node(&self.subject)
            .map_err(HtmlMediaFactsError::Capture)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Err(HtmlMediaFactsError::InvalidInput);
        };
        // Fixed selected roles, not a scan of the full vocabulary or raw value.
        let local = element.name().local_name().code_units();
        if element.name().namespace().uri().is_none_or(|uri| {
            !uri.code_units()
                .iter()
                .copied()
                .eq("http://www.w3.org/1999/xhtml".encode_utf16())
        }) || !matches!(
            local,
            [108, 105, 110, 107]
                | [115, 116, 121, 108, 101]
                | [115, 111, 117, 114, 99, 101]
                | [109, 101, 116, 97]
        ) {
            return Err(HtmlMediaFactsError::InvalidInput);
        }
        let a = element
            .attributes()
            .as_slice()
            .get(self.ordinal)
            .ok_or(HtmlMediaFactsError::InvalidInput)?;
        if !media_field_matches(a, HtmlMediaField::Media) {
            return Err(HtmlMediaFactsError::InvalidInput);
        }
        Ok(a.value())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaExternalCapability {
    GrammarFeature,
    EnvironmentFeature,
    Edition,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaGrammarOutcome {
    Valid,
    Invalid,
    Unavailable,
    Unsupported(HtmlMediaExternalCapability),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaMatchOutcome {
    Matches,
    DoesNotMatch,
    Unavailable,
    Unsupported(HtmlMediaExternalCapability),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMediaGrammarAttestation {
    revision: HtmlMediaGrammarRevision,
    outcome: HtmlMediaGrammarOutcome,
}
impl HtmlMediaGrammarAttestation {
    pub fn new(revision: HtmlMediaGrammarRevision, outcome: HtmlMediaGrammarOutcome) -> Self {
        Self { revision, outcome }
    }
    pub fn revision(&self) -> &HtmlMediaGrammarRevision {
        &self.revision
    }
    pub fn outcome(&self) -> HtmlMediaGrammarOutcome {
        self.outcome
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMediaMatchAttestation {
    revision: HtmlMediaContextRevision,
    outcome: HtmlMediaMatchOutcome,
}
impl HtmlMediaMatchAttestation {
    pub fn new(revision: HtmlMediaContextRevision, outcome: HtmlMediaMatchOutcome) -> Self {
        Self { revision, outcome }
    }
    pub fn revision(&self) -> &HtmlMediaContextRevision {
        &self.revision
    }
    pub fn outcome(&self) -> HtmlMediaMatchOutcome {
        self.outcome
    }
}
/// Producer truth is external. The facts owner validates identity and revisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMediaFact {
    input: HtmlMediaAttributeInput,
    grammar: Option<HtmlMediaGrammarAttestation>,
    matching: Option<HtmlMediaMatchAttestation>,
}
impl HtmlMediaFact {
    pub fn new(
        input: HtmlMediaAttributeInput,
        grammar: Option<HtmlMediaGrammarAttestation>,
        matching: Option<HtmlMediaMatchAttestation>,
    ) -> Self {
        Self {
            input,
            grammar,
            matching,
        }
    }
    pub fn input(&self) -> &HtmlMediaAttributeInput {
        &self.input
    }
    pub fn grammar(&self) -> Option<&HtmlMediaGrammarAttestation> {
        self.grammar.as_ref()
    }
    pub fn matching(&self) -> Option<&HtmlMediaMatchAttestation> {
        self.matching.as_ref()
    }
}
fact_limits!(HtmlMediaFactsLimits);
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMediaFactsError {
    Capture(HtmlCaptureError),
    InvalidInput,
    HostRevisionMismatch,
    GrammarRevisionMismatch,
    MatchRevisionMismatch,
    DuplicateInput(HtmlMediaAttributeInput),
    RecordLimit,
    ByteLimit,
    WorkLimit,
    /// The ledger that enforces the limits reported a state its own sequencing rules out.
    LedgerState,
    RepresentationExhausted,
}
impl ScanError for HtmlMediaFactsError {
    fn representation_exhausted() -> Self {
        Self::RepresentationExhausted
    }
    fn work_exceeded(_limit: usize, _required: usize) -> Self {
        Self::WorkLimit
    }
    fn capture(error: HtmlCaptureError) -> Self {
        Self::Capture(error)
    }
    fn ledger_state() -> Self {
        Self::LedgerState
    }
}
impl FactError for HtmlMediaFactsError {
    fn host_revision_mismatch() -> Self {
        Self::HostRevisionMismatch
    }
    fn record_limit() -> Self {
        Self::RecordLimit
    }
    fn byte_limit() -> Self {
        Self::ByteLimit
    }
}
impl fmt::Display for HtmlMediaFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlMediaFactsError {}
/// Finite incoming storage admission, charged to its own ledger scope.
/// Input capture backing is shared; no raw text or tree is projected/copied.
#[derive(Debug)]
pub struct HtmlMediaFacts {
    capture: HtmlCaptureId,
    host: HtmlHostFactsRevision,
    revision: HtmlMediaContextRevision,
    facts: Vec<HtmlMediaFact>,
    /// The first fact of each subject.
    pub(super) subjects: FactIndex,
}
impl HtmlMediaFacts {
    pub fn new(
        capture: &HtmlCapture,
        host: HtmlHostFactsRevision,
        revision: HtmlMediaContextRevision,
        facts: Vec<HtmlMediaFact>,
        limits: HtmlMediaFactsLimits,
    ) -> Result<Self, HtmlMediaFactsError> {
        use HtmlMediaFactsError as E;
        FactAdmission::<E>::host(capture, &host)?;
        let mut a = FactAdmission::<E>::new(HtmlCapScope::MediaFacts, limits)?;
        a.add_records(facts.capacity())?;
        a.add_bytes(
            facts
                .capacity()
                .checked_mul(std::mem::size_of::<HtmlMediaFact>())
                .ok_or(E::RepresentationExhausted)?,
        )?;
        let mut inputs = FactIndex::new(
            capture.identity(),
            facts.len(),
            |n| a.work(n),
            E::RepresentationExhausted,
        )?;
        let mut subjects = FactIndex::new(
            capture.identity(),
            facts.len(),
            |n| a.work(n),
            E::RepresentationExhausted,
        )?;
        for (i, fact) in facts.iter().enumerate() {
            inputs.enter_bound(&fact.input.subject, fact.input.ordinal, i);
            subjects.enter(&fact.input.subject, i);
        }
        for (i, fact) in facts.iter().enumerate() {
            // Fixed role, namespace and field comparisons in raw(): ≤64 units.
            a.work(64)?;
            fact.input.raw(capture)?;
            a.work(1)?;
            if fact
                .grammar
                .as_ref()
                .is_some_and(|g| g.revision != revision.grammar)
            {
                return Err(E::GrammarRevisionMismatch);
            }
            a.work(1)?;
            if fact
                .matching
                .as_ref()
                .is_some_and(|m| m.revision != revision)
            {
                return Err(E::MatchRevisionMismatch);
            }
            if inputs.is_repeat(&fact.input.subject, fact.input.ordinal, i) {
                return Err(E::DuplicateInput(fact.input.clone()));
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host,
            revision,
            facts,
            subjects,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host
    }
    pub fn revision(&self) -> &HtmlMediaContextRevision {
        &self.revision
    }
    pub fn facts(&self) -> &[HtmlMediaFact] {
        &self.facts
    }
}

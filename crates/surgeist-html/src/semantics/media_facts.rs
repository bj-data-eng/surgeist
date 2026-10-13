//! Concrete CSS producer attestations for one exact captured HTML media field.
use super::*;
use crate::attributes::media::media_field;

#[derive(Clone, Debug)]
pub struct HtmlMediaProducerId(Arc<()>);
impl HtmlMediaProducerId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlMediaProducerId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlMediaProducerId {}

macro_rules! media_revision {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            producer: HtmlMediaProducerId,
            configuration: u64,
            revision: u64,
        }
        impl $name {
            pub fn new(producer: HtmlMediaProducerId, configuration: u64, revision: u64) -> Self {
                Self {
                    producer,
                    configuration,
                    revision,
                }
            }
            pub fn producer(&self) -> &HtmlMediaProducerId {
                &self.producer
            }
            pub fn configuration(&self) -> u64 {
                self.configuration
            }
            pub fn revision(&self) -> u64 {
                self.revision
            }
        }
    };
}
media_revision!(HtmlMediaGrammarRevision);
media_revision!(HtmlMediaEnvironmentRevision);

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
        if !media_field(a, &[109, 101, 100, 105, 97]) {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlMediaFactsLimits {
    records: usize,
    bytes: usize,
    work: usize,
}
impl HtmlMediaFactsLimits {
    pub fn new(records: usize, bytes: usize, work: usize) -> Self {
        Self {
            records,
            bytes,
            work,
        }
    }
    pub fn records(self) -> usize {
        self.records
    }
    pub fn bytes(self) -> usize {
        self.bytes
    }
    pub fn work(self) -> usize {
        self.work
    }
}
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
    RepresentationExhausted,
}
impl fmt::Display for HtmlMediaFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlMediaFactsError {}
/// Finite incoming storage admission, separate from the semantic session ledger.
/// Input capture backing is shared; no raw text or tree is projected/copied.
#[derive(Debug)]
pub struct HtmlMediaFacts {
    capture: HtmlCaptureId,
    host: HtmlHostFactsRevision,
    revision: HtmlMediaContextRevision,
    facts: Vec<HtmlMediaFact>,
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
        if &host != capture.host_facts_revision() {
            return Err(E::HostRevisionMismatch);
        }
        if facts.capacity() > limits.records {
            return Err(E::RecordLimit);
        }
        if facts
            .capacity()
            .checked_mul(std::mem::size_of::<HtmlMediaFact>())
            .ok_or(E::RepresentationExhausted)?
            > limits.bytes
        {
            return Err(E::ByteLimit);
        }
        let mut work = 0usize;
        let mut charge = |amount: usize| {
            work = work.checked_add(amount).ok_or(E::RepresentationExhausted)?;
            if work > limits.work {
                Err(E::WorkLimit)
            } else {
                Ok(())
            }
        };
        for (i, fact) in facts.iter().enumerate() {
            // Fixed role, namespace and field comparisons in raw(): ≤64 units.
            charge(64)?;
            fact.input.raw(capture)?;
            charge(1)?;
            if fact
                .grammar
                .as_ref()
                .is_some_and(|g| g.revision != revision.grammar)
            {
                return Err(E::GrammarRevisionMismatch);
            }
            charge(1)?;
            if fact
                .matching
                .as_ref()
                .is_some_and(|m| m.revision != revision)
            {
                return Err(E::MatchRevisionMismatch);
            }
            for prior in &facts[..i] {
                charge(1)?;
                if prior.input == fact.input {
                    return Err(E::DuplicateInput(fact.input.clone()));
                }
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host,
            revision,
            facts,
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

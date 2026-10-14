//! Capture-qualified HTML877d relationships; external definitions are finite facts.
use super::*;
mod noopener;
mod selectors;
use crate::bounded as rel;
use crate::bounded::{
    HtmlRelActionError, HtmlRelInterpretation, HtmlRelMeter, HtmlRelRefinement, HtmlRelReservation,
    RelInputs,
};
use crate::{
    HtmlCaptureDocumentId, HtmlRelEffect, HtmlRelKeyword, HtmlRelLinks, HtmlRelReport, HtmlRelRole,
    HtmlRelSpelling, HtmlSpaceToken,
};
pub use noopener::*;
pub use selectors::*;

#[derive(Clone, Debug)]
pub struct HtmlRelProducerId(Arc<()>);
impl HtmlRelProducerId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlRelProducerId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlRelProducerId {}
/// Actual producer/configuration/revision, independently expected by the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelRevision {
    producer: HtmlRelProducerId,
    configuration: u64,
    revision: u64,
}
impl HtmlRelRevision {
    pub fn new(producer: HtmlRelProducerId, configuration: u64, revision: u64) -> Self {
        Self {
            producer,
            configuration,
            revision,
        }
    }
    pub fn producer(&self) -> &HtmlRelProducerId {
        &self.producer
    }
    pub fn configuration(&self) -> u64 {
        self.configuration
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HtmlRelExpectedRevisions {
    registry: Option<HtmlRelRevision>,
    supported: Option<HtmlRelRevision>,
    grammar: Option<HtmlRelRevision>,
    section: Option<HtmlRelRevision>,
    url: Option<HtmlRelRevision>,
    settings: Option<HtmlRelRevision>,
}
impl HtmlRelExpectedRevisions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_registry(mut self, revision: HtmlRelRevision) -> Self {
        self.registry = Some(revision);
        self
    }
    pub fn with_supported_tokens(mut self, revision: HtmlRelRevision) -> Self {
        self.supported = Some(revision);
        self
    }
    pub fn with_url_grammar(mut self, revision: HtmlRelRevision) -> Self {
        self.grammar = Some(revision);
        self
    }
    pub fn with_bookmark_sections(mut self, revision: HtmlRelRevision) -> Self {
        self.section = Some(revision);
        self
    }
    pub fn with_url_records(mut self, revision: HtmlRelRevision) -> Self {
        self.url = Some(revision);
        self
    }
    pub fn with_settings(mut self, revision: HtmlRelRevision) -> Self {
        self.settings = Some(revision);
        self
    }
    pub fn url_records(&self) -> Option<&HtmlRelRevision> {
        self.url.as_ref()
    }
    pub fn settings(&self) -> Option<&HtmlRelRevision> {
        self.settings.as_ref()
    }
    pub fn registry(&self) -> Option<&HtmlRelRevision> {
        self.registry.as_ref()
    }
    pub fn supported_tokens(&self) -> Option<&HtmlRelRevision> {
        self.supported.as_ref()
    }
    pub fn url_grammar(&self) -> Option<&HtmlRelRevision> {
        self.grammar.as_ref()
    }
    pub fn bookmark_sections(&self) -> Option<&HtmlRelRevision> {
        self.section.as_ref()
    }
}
#[derive(Clone, Debug)]
pub struct HtmlRelDefinitionId(Arc<()>);
impl HtmlRelDefinitionId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlRelDefinitionId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlRelDefinitionId {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelDefinitionKey {
    identity: HtmlRelDefinitionId,
    revision: u64,
}
impl HtmlRelDefinitionKey {
    pub fn new(identity: HtmlRelDefinitionId, revision: u64) -> Self {
        Self { identity, revision }
    }
    pub fn identity(&self) -> &HtmlRelDefinitionId {
        &self.identity
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelRegistryStatus {
    Proposed,
    Ratified,
    Discontinued,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelExtensionEffect {
    Unavailable,
    NotAllowed,
    Hyperlink,
    ExternalResource,
    InternalResource,
    Annotation,
    StylesheetModifier,
}
impl HtmlRelExtensionEffect {
    fn effect(self) -> Option<HtmlRelEffect> {
        match self {
            Self::Hyperlink => Some(HtmlRelEffect::Hyperlink),
            Self::ExternalResource => Some(HtmlRelEffect::ExternalResource),
            Self::InternalResource => Some(HtmlRelEffect::InternalResource),
            Self::Annotation => Some(HtmlRelEffect::Annotation),
            Self::StylesheetModifier => Some(HtmlRelEffect::StylesheetModifier),
            _ => None,
        }
    }
    fn from_standard(effect: Option<HtmlRelEffect>) -> Self {
        match effect {
            Some(HtmlRelEffect::Hyperlink) => Self::Hyperlink,
            Some(HtmlRelEffect::ExternalResource) => Self::ExternalResource,
            Some(HtmlRelEffect::InternalResource) => Self::InternalResource,
            Some(HtmlRelEffect::Annotation) => Self::Annotation,
            Some(HtmlRelEffect::StylesheetModifier) => Self::StylesheetModifier,
            None => Self::NotAllowed,
        }
    }
}
/// A finite actual definition. Its key includes canonical definition revision;
/// aliases refer to this key and cannot supply competing effects.
#[derive(Debug)]
pub struct HtmlRelDefinition {
    key: HtmlRelDefinitionKey,
    canonical: HtmlSpaceToken,
    status: HtmlRelRegistryStatus,
    effects: [HtmlRelExtensionEffect; 4],
    standard: Option<HtmlRelKeyword>,
}
impl HtmlRelDefinition {
    pub fn extension(
        key: HtmlRelDefinitionKey,
        canonical: HtmlSpaceToken,
        status: HtmlRelRegistryStatus,
        link: HtmlRelExtensionEffect,
        anchor_area: HtmlRelExtensionEffect,
        form: HtmlRelExtensionEffect,
    ) -> Self {
        Self {
            key,
            canonical,
            status,
            effects: [link, anchor_area, anchor_area, form],
            standard: None,
        }
    }
    /// Registered synonyms of a standard type use its canonical HTML meaning.
    pub fn standard(
        key: HtmlRelDefinitionKey,
        keyword: HtmlRelKeyword,
        status: HtmlRelRegistryStatus,
    ) -> Self {
        Self {
            key,
            canonical: HtmlSpaceToken::new(keyword.keyword())
                .expect("standard keywords are nonempty ASCII tokens"),
            status,
            effects: [HtmlRelExtensionEffect::Unavailable; 4],
            standard: Some(keyword),
        }
    }
    pub fn key(&self) -> &HtmlRelDefinitionKey {
        &self.key
    }
    pub fn canonical(&self) -> &HtmlSpaceToken {
        &self.canonical
    }
}
#[derive(Debug)]
enum RegistryOccurrence {
    Unavailable,
    ProvenAbsent,
    Definition {
        key: HtmlRelDefinitionKey,
        synonym: bool,
    },
}
/// The original occurrence is retained literally, including its surrounding rel.
#[derive(Debug)]
pub struct HtmlRelRegistryOccurrence {
    subject: HtmlCaptureSubjectId,
    raw: DocumentString,
    start: usize,
    end: usize,
    state: RegistryOccurrence,
}
impl HtmlRelRegistryOccurrence {
    pub fn unavailable(
        subject: HtmlCaptureSubjectId,
        raw: DocumentString,
        range: std::ops::Range<usize>,
    ) -> Self {
        Self {
            subject,
            raw,
            start: range.start,
            end: range.end,
            state: RegistryOccurrence::Unavailable,
        }
    }
    pub fn proven_absent(
        subject: HtmlCaptureSubjectId,
        raw: DocumentString,
        range: std::ops::Range<usize>,
    ) -> Self {
        Self {
            subject,
            raw,
            start: range.start,
            end: range.end,
            state: RegistryOccurrence::ProvenAbsent,
        }
    }
    pub fn definition(
        subject: HtmlCaptureSubjectId,
        raw: DocumentString,
        range: std::ops::Range<usize>,
        key: HtmlRelDefinitionKey,
    ) -> Self {
        Self {
            subject,
            raw,
            start: range.start,
            end: range.end,
            state: RegistryOccurrence::Definition {
                key,
                synonym: false,
            },
        }
    }
    pub fn synonym(
        subject: HtmlCaptureSubjectId,
        raw: DocumentString,
        range: std::ops::Range<usize>,
        canonical_definition: HtmlRelDefinitionKey,
    ) -> Self {
        Self {
            subject,
            raw,
            start: range.start,
            end: range.end,
            state: RegistryOccurrence::Definition {
                key: canonical_definition,
                synonym: true,
            },
        }
    }
}
#[derive(Debug)]
pub struct HtmlRelSupportedTokens {
    subject: HtmlCaptureSubjectId,
    tokens: Vec<HtmlRelKeyword>,
}
impl HtmlRelSupportedTokens {
    pub fn new(subject: HtmlCaptureSubjectId, tokens: Vec<HtmlRelKeyword>) -> Self {
        Self { subject, tokens }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelGrammarInput {
    Href,
    ExtensionOccurrence { start: usize, end: usize },
}
#[derive(Debug)]
pub struct HtmlRelUrlGrammar {
    subject: HtmlCaptureSubjectId,
    input: HtmlRelGrammarInput,
    raw: DocumentString,
    valid: bool,
}
impl HtmlRelUrlGrammar {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        input: HtmlRelGrammarInput,
        raw: DocumentString,
        valid: bool,
    ) -> Self {
        Self {
            subject,
            input,
            raw,
            valid,
        }
    }
}
/// Associated sections are source concepts that need not identify an Element.
#[derive(Clone, Debug)]
pub struct HtmlRelSectionId(Arc<()>);
impl HtmlRelSectionId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlRelSectionId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlRelSectionId {}
#[derive(Debug)]
pub struct HtmlRelBookmarkSection {
    subject: HtmlCaptureSubjectId,
    section: HtmlRelSectionId,
}
impl HtmlRelBookmarkSection {
    pub fn new(subject: HtmlCaptureSubjectId, section: HtmlRelSectionId) -> Self {
        Self { subject, section }
    }
}
/// Moved concrete vectors; neither callbacks nor online registry state are retained.
#[derive(Debug, Default)]
pub struct HtmlRelFactsInput {
    revisions: HtmlRelExpectedRevisions,
    definitions: Vec<HtmlRelDefinition>,
    occurrences: Vec<HtmlRelRegistryOccurrence>,
    supported: Vec<HtmlRelSupportedTokens>,
    grammar: Vec<HtmlRelUrlGrammar>,
    sections: Vec<HtmlRelBookmarkSection>,
    urls: Vec<HtmlRelUrlObservation>,
    same_site: Vec<HtmlRelSameSiteObservation>,
}
impl HtmlRelFactsInput {
    pub fn with_url_records(
        mut self,
        revision: HtmlRelRevision,
        records: Vec<HtmlRelUrlObservation>,
    ) -> Self {
        self.revisions.url = Some(revision);
        self.urls = records;
        self
    }
    pub fn with_settings(
        mut self,
        revision: HtmlRelRevision,
        records: Vec<HtmlRelSameSiteObservation>,
    ) -> Self {
        self.revisions.settings = Some(revision);
        self.same_site = records;
        self
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_registry(
        mut self,
        revision: HtmlRelRevision,
        definitions: Vec<HtmlRelDefinition>,
        occurrences: Vec<HtmlRelRegistryOccurrence>,
    ) -> Self {
        self.revisions.registry = Some(revision);
        self.definitions = definitions;
        self.occurrences = occurrences;
        self
    }
    pub fn with_supported_tokens(
        mut self,
        revision: HtmlRelRevision,
        records: Vec<HtmlRelSupportedTokens>,
    ) -> Self {
        self.revisions.supported = Some(revision);
        self.supported = records;
        self
    }
    pub fn with_url_grammar(
        mut self,
        revision: HtmlRelRevision,
        records: Vec<HtmlRelUrlGrammar>,
    ) -> Self {
        self.revisions.grammar = Some(revision);
        self.grammar = records;
        self
    }
    pub fn with_bookmark_sections(
        mut self,
        revision: HtmlRelRevision,
        records: Vec<HtmlRelBookmarkSection>,
    ) -> Self {
        self.revisions.section = Some(revision);
        self.sections = records;
        self
    }
}
/// Capacities bound incoming vector backing, including nested supported-token
/// vectors. Bytes also reserve each retained UTF-16 backing reference's capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlRelFactsLimits {
    records: usize,
    bytes: usize,
    work: usize,
}
impl HtmlRelFactsLimits {
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
pub enum HtmlRelFactsError {
    SelectorBindingMismatch,
    IconObservationCoverage,
    IconOrderingInvalid,
    IconMediaContradiction,
    IconNotAttempted,
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    UrlRevisionMismatch,
    UrlBindingMismatch,
    RecordLimit,
    ByteLimit,
    WorkLimit,
    RepresentationExhausted,
    DuplicateDefinition,
    DuplicateObservation,
    InvalidRole,
    InvalidEffect,
    InvalidOccurrence,
    RawMismatch,
    MissingDefinition,
    CanonicalMismatch,
    InvalidSupportedToken,
    GrammarContradiction,
}
impl fmt::Display for HtmlRelFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlRelFactsError {}
#[derive(Debug)]
pub struct HtmlRelFacts {
    capture: HtmlCaptureId,
    host: HtmlHostFactsRevision,
    input: HtmlRelFactsInput,
}
struct FactsAdmission {
    limits: HtmlRelFactsLimits,
    records: usize,
    bytes: usize,
    work: usize,
}
impl FactsAdmission {
    fn work(&mut self, n: usize) -> Result<(), HtmlRelFactsError> {
        self.work = self
            .work
            .checked_add(n)
            .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
        if self.work > self.limits.work {
            Err(HtmlRelFactsError::WorkLimit)
        } else {
            Ok(())
        }
    }
    fn vector<T>(&mut self, v: &Vec<T>) -> Result<(), HtmlRelFactsError> {
        self.work(1)?;
        self.records = self
            .records
            .checked_add(v.capacity())
            .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
        self.bytes = self
            .bytes
            .checked_add(
                v.capacity()
                    .checked_mul(std::mem::size_of::<T>())
                    .ok_or(HtmlRelFactsError::RepresentationExhausted)?,
            )
            .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
        if self.records > self.limits.records {
            return Err(HtmlRelFactsError::RecordLimit);
        }
        if self.bytes > self.limits.bytes {
            return Err(HtmlRelFactsError::ByteLimit);
        }
        Ok(())
    }
    fn raw(&mut self, raw: &DocumentString) -> Result<(), HtmlRelFactsError> {
        self.work(1)?;
        let bytes = raw
            .shared_code_units()
            .capacity()
            .checked_mul(2)
            .and_then(|n| {
                n.checked_add(std::mem::size_of::<Vec<u16>>() + 2 * std::mem::size_of::<usize>())
            })
            .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
        if self.bytes > self.limits.bytes {
            Err(HtmlRelFactsError::ByteLimit)
        } else {
            Ok(())
        }
    }
    fn exact(&mut self, a: &DocumentString, b: &DocumentString) -> Result<bool, HtmlRelFactsError> {
        self.work(1)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.code_units().iter().zip(b.code_units()) {
            self.work(1)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn folded(
        &mut self,
        a: &DocumentString,
        b: &DocumentString,
    ) -> Result<bool, HtmlRelFactsError> {
        self.work(1)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.code_units().iter().zip(b.code_units()) {
            self.work(3)?;
            if rel::fold_rel_ascii(*a) != rel::fold_rel_ascii(*b) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn attr<'a>(
        &mut self,
        element: &'a HtmlCapturedElement,
        local: &str,
    ) -> Result<Option<&'a DocumentString>, HtmlRelFactsError> {
        for a in element.attributes().iter() {
            self.work(1)?;
            if a.name().namespace().uri().is_some() {
                continue;
            }
            let envelope = a
                .name()
                .local_name()
                .len()
                .checked_add(local.len())
                .and_then(|n| n.checked_add(1))
                .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
            self.work(envelope)?;
            if a.name()
                .local_name()
                .code_units()
                .iter()
                .copied()
                .eq(local.bytes().map(u16::from))
            {
                return Ok(Some(a.value()));
            }
        }
        Ok(None)
    }
    fn role(
        &mut self,
        capture: &HtmlCapture,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlRelRole, HtmlRelFactsError> {
        self.work(1)?;
        let node = capture.node(id).map_err(HtmlRelFactsError::Capture)?;
        let HtmlCapturedNodeKind::Element(e) = node.kind() else {
            return Err(HtmlRelFactsError::InvalidRole);
        };
        let envelope = e
            .name()
            .local_name()
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| n.checked_add(e.name().namespace().uri().map_or(0, DocumentString::len)))
            .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
        self.work(envelope)?;
        HtmlRelRole::from_element(e.name()).ok_or(HtmlRelFactsError::InvalidRole)
    }
    fn occurrence(
        &mut self,
        capture: &HtmlCapture,
        id: &HtmlCaptureSubjectId,
        raw: &DocumentString,
        start: usize,
        end: usize,
    ) -> Result<(), HtmlRelFactsError> {
        self.role(capture, id)?;
        self.work(1)?;
        let node = capture.node(id).map_err(HtmlRelFactsError::Capture)?;
        let HtmlCapturedNodeKind::Element(e) = node.kind() else {
            unreachable!()
        };
        let Some(actual) = self.attr(e, "rel")? else {
            return Err(HtmlRelFactsError::RawMismatch);
        };
        if !self.exact(actual, raw)? {
            return Err(HtmlRelFactsError::RawMismatch);
        }
        self.work(crate::bounded::HtmlSpaceTokenCursor::BEGIN_WORK)?;
        let mut cursor = crate::bounded::HtmlSpaceTokenCursor::new(raw.code_units());
        loop {
            self.work(crate::bounded::HtmlSpaceTokenCursor::STEP_WORK)?;
            match cursor.step() {
                crate::bounded::HtmlSpaceTokenStep::Continue => {}
                crate::bounded::HtmlSpaceTokenStep::Done => {
                    return Err(HtmlRelFactsError::InvalidOccurrence);
                }
                crate::bounded::HtmlSpaceTokenStep::Token { start: s, end: e }
                    if s == start && e == end =>
                {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
impl HtmlRelFacts {
    pub fn new(
        capture: &HtmlCapture,
        expected_host: &HtmlHostFactsRevision,
        input: HtmlRelFactsInput,
        limits: HtmlRelFactsLimits,
    ) -> Result<Self, HtmlRelFactsError> {
        if capture.host_facts_revision() != expected_host {
            return Err(HtmlRelFactsError::HostRevisionMismatch);
        }
        let mut a = FactsAdmission {
            limits,
            records: 0,
            bytes: 0,
            work: 0,
        };
        a.vector(&input.definitions)?;
        a.vector(&input.occurrences)?;
        a.vector(&input.supported)?;
        a.vector(&input.grammar)?;
        a.vector(&input.sections)?;
        for (i, d) in input.definitions.iter().enumerate() {
            a.work(1)?;
            a.raw(d.canonical.value())?;
            for prior in &input.definitions[..i] {
                a.work(1)?;
                if prior.key == d.key || a.folded(prior.canonical.value(), d.canonical.value())? {
                    return Err(HtmlRelFactsError::DuplicateDefinition);
                }
            }
            if d.standard.is_none() {
                a.work(5)?;
                if d.effects[0] == HtmlRelExtensionEffect::Annotation
                    || d.effects.iter().any(|e| {
                        matches!(
                            e,
                            HtmlRelExtensionEffect::InternalResource
                                | HtmlRelExtensionEffect::StylesheetModifier
                        )
                    })
                {
                    return Err(HtmlRelFactsError::InvalidEffect);
                }
                for keyword in HtmlRelKeyword::ALL {
                    a.work(
                        d.canonical
                            .value()
                            .len()
                            .checked_add(1)
                            .ok_or(HtmlRelFactsError::RepresentationExhausted)?,
                    )?;
                    if rel::rel_ascii_keyword(d.canonical.value().code_units(), keyword.keyword()) {
                        return Err(HtmlRelFactsError::CanonicalMismatch);
                    }
                }
            }
        }
        for (i, o) in input.occurrences.iter().enumerate() {
            a.raw(&o.raw)?;
            a.occurrence(capture, &o.subject, &o.raw, o.start, o.end)?;
            for prior in &input.occurrences[..i] {
                a.work(3)?;
                if prior.subject == o.subject && prior.start == o.start && prior.end == o.end {
                    return Err(HtmlRelFactsError::DuplicateObservation);
                }
            }
            if let RegistryOccurrence::Definition { key, synonym } = &o.state {
                let mut found = None;
                for d in &input.definitions {
                    a.work(1)?;
                    if &d.key == key {
                        found = Some(d);
                        break;
                    }
                }
                let d = found.ok_or(HtmlRelFactsError::MissingDefinition)?;
                if !synonym {
                    let units = &o.raw.code_units()[o.start..o.end];
                    let canonical = d.canonical.value().code_units();
                    a.work(
                        units
                            .len()
                            .checked_add(canonical.len())
                            .and_then(|n| n.checked_add(1))
                            .ok_or(HtmlRelFactsError::RepresentationExhausted)?,
                    )?;
                    if units.len() != canonical.len()
                        || !units
                            .iter()
                            .zip(canonical)
                            .all(|(a, b)| rel::fold_rel_ascii(*a) == rel::fold_rel_ascii(*b))
                    {
                        return Err(HtmlRelFactsError::CanonicalMismatch);
                    }
                }
            }
        }
        for (i, o) in input.supported.iter().enumerate() {
            let role = a.role(capture, &o.subject)?;
            a.vector(&o.tokens)?;
            for prior in &input.supported[..i] {
                a.work(1)?;
                if prior.subject == o.subject {
                    return Err(HtmlRelFactsError::DuplicateObservation);
                }
            }
            for (j, &token) in o.tokens.iter().enumerate() {
                a.work(1)?;
                if !token.possible_supported_token(role) {
                    return Err(HtmlRelFactsError::InvalidSupportedToken);
                }
                for prior in &o.tokens[..j] {
                    a.work(1)?;
                    if *prior == token {
                        return Err(HtmlRelFactsError::DuplicateObservation);
                    }
                }
            }
        }
        for (i, o) in input.grammar.iter().enumerate() {
            a.role(capture, &o.subject)?;
            a.raw(&o.raw)?;
            for prior in &input.grammar[..i] {
                a.work(2)?;
                if prior.subject == o.subject && prior.input == o.input {
                    return Err(HtmlRelFactsError::DuplicateObservation);
                }
            }
            match o.input {
                HtmlRelGrammarInput::ExtensionOccurrence { start, end } => {
                    a.occurrence(capture, &o.subject, &o.raw, start, end)?
                }
                HtmlRelGrammarInput::Href => {
                    let node = capture
                        .node(&o.subject)
                        .map_err(HtmlRelFactsError::Capture)?;
                    let HtmlCapturedNodeKind::Element(e) = node.kind() else {
                        unreachable!()
                    };
                    let actual = a.attr(e, "href")?.ok_or(HtmlRelFactsError::RawMismatch)?;
                    if !a.exact(actual, &o.raw)? {
                        return Err(HtmlRelFactsError::RawMismatch);
                    }
                    let role = a.role(capture, &o.subject)?;
                    if role == HtmlRelRole::Form {
                        return Err(HtmlRelFactsError::InvalidRole);
                    }
                    if role == HtmlRelRole::Link && o.valid {
                        let mut nonspace = false;
                        for &u in actual.code_units() {
                            a.work(6)?;
                            nonspace |= !matches!(u, 9 | 10 | 12 | 13 | 32);
                        }
                        if !nonspace {
                            return Err(HtmlRelFactsError::GrammarContradiction);
                        }
                    }
                }
            }
        }
        for (i, o) in input.sections.iter().enumerate() {
            a.role(capture, &o.subject)?;
            for prior in &input.sections[..i] {
                a.work(1)?;
                if prior.subject == o.subject {
                    return Err(HtmlRelFactsError::DuplicateObservation);
                }
            }
        }
        noopener::validate(&input, capture, &mut a)?;
        Ok(Self {
            capture: capture.identity().clone(),
            host: expected_host.clone(),
            input,
        })
    }
    pub fn revisions(&self) -> &HtmlRelExpectedRevisions {
        &self.input.revisions
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelExtensionReadback {
    occurrence: usize,
    status: Option<HtmlRelRegistryStatus>,
    proven_absent: bool,
    definition: Option<HtmlRelDefinitionKey>,
    canonical: Option<HtmlSpaceToken>,
    synonym: Option<bool>,
    effect: HtmlRelExtensionEffect,
    standard: Option<HtmlRelKeyword>,
    absolute_url: Option<bool>,
    author: HtmlSemanticValue<bool>,
}
impl HtmlRelExtensionReadback {
    pub fn occurrence(&self) -> usize {
        self.occurrence
    }
    pub fn registry_status(&self) -> Option<HtmlRelRegistryStatus> {
        self.status
    }
    pub fn proven_absent(&self) -> bool {
        self.proven_absent
    }
    pub fn definition(&self) -> Option<&HtmlRelDefinitionKey> {
        self.definition.as_ref()
    }
    pub fn canonical(&self) -> Option<&HtmlSpaceToken> {
        self.canonical.as_ref()
    }
    pub fn forbidden_synonym(&self) -> Option<bool> {
        self.synonym
    }
    pub fn effect(&self) -> HtmlRelExtensionEffect {
        self.effect
    }
    pub fn standard_keyword(&self) -> Option<HtmlRelKeyword> {
        self.standard
    }
    pub fn absolute_url_grammar(&self) -> Option<bool> {
        self.absolute_url
    }
    pub fn author_conformance(&self) -> &HtmlSemanticValue<bool> {
        &self.author
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlRelAssociation {
    Page(HtmlCaptureDocumentId),
    Article(HtmlCaptureSubjectId),
    ParentAndChildren(Option<HtmlCaptureSubjectId>),
    Section(HtmlRelSectionId),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelMeaning {
    keyword: HtmlRelKeyword,
    association: HtmlSemanticValue<HtmlRelAssociation>,
}
impl HtmlRelMeaning {
    pub fn keyword(&self) -> HtmlRelKeyword {
        self.keyword
    }
    pub fn association(&self) -> &HtmlSemanticValue<HtmlRelAssociation> {
        &self.association
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct RelContextData {
    local: HtmlRelReport,
    extensions: Vec<HtmlRelExtensionReadback>,
    supported: HtmlSemanticValue<Vec<HtmlRelKeyword>>,
    links: HtmlRelLinks,
    meanings: Vec<HtmlRelMeaning>,
    translation: HtmlSemanticValue<bool>,
    href_grammar: Option<bool>,
    href_author: HtmlSemanticValue<bool>,
    image_source_preload_author: HtmlSemanticValue<bool>,
    history: HtmlSemanticValue<crate::HtmlLinkHistory>,
    implicit_render_blocking: HtmlSemanticValue<bool>,
}
/// One admitted owning record keeps semantic fact variants compact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelContextReadback {
    data: Vec<RelContextData>,
}
impl HtmlRelContextReadback {
    pub fn local(&self) -> &HtmlRelReport {
        &self.data[0].local
    }
    pub fn extensions(&self) -> &[HtmlRelExtensionReadback] {
        &self.data[0].extensions
    }
    pub fn supported_tokens(&self) -> &HtmlSemanticValue<Vec<HtmlRelKeyword>> {
        &self.data[0].supported
    }
    pub fn links(&self) -> HtmlRelLinks {
        self.data[0].links
    }
    pub fn meanings(&self) -> &[HtmlRelMeaning] {
        &self.data[0].meanings
    }
    pub fn alternate_translation(&self) -> &HtmlSemanticValue<bool> {
        &self.data[0].translation
    }
    pub fn href_url_grammar(&self) -> Option<bool> {
        self.data[0].href_grammar
    }
    /// Link nonempty URL author conformance. Missing grammar remains conditional.
    pub fn href_url_author(&self) -> &HtmlSemanticValue<bool> {
        &self.data[0].href_author
    }
    /// Author context for an actual imagesrcset slot, preserving unknown rel
    /// extension classification rather than treating it as an unsuitable context.
    pub fn image_source_preload_author(&self) -> &HtmlSemanticValue<bool> {
        &self.data[0].image_source_preload_author
    }
    pub fn link_history(&self) -> &HtmlSemanticValue<crate::HtmlLinkHistory> {
        &self.data[0].history
    }
    pub fn implicitly_potentially_render_blocking(&self) -> &HtmlSemanticValue<bool> {
        &self.data[0].implicit_render_blocking
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelMissingFact {
    Registry,
    UrlGrammar,
    SupportedTokens,
    BookmarkSection,
    DocumentElementLanguage,
    LinkHistory,
    NoopenerInput,
    UrlRecord,
    IconSuitability,
    DefaultFeedApplicability,
    IconOrdering,
    IconCloserExamination,
}
struct RelControl<'s, 'a> {
    session: &'s mut HtmlSemanticSession<'a>,
}
impl HtmlRelMeter for RelControl<'_, '_> {
    type Error = HtmlSemanticFailure;
    fn work(&mut self, n: usize) -> Result<(), Self::Error> {
        self.session.charge(n)
    }
    fn reserve(&mut self, guard: &mut HtmlRelReservation<'_>) -> Result<(), Self::Error> {
        self.session.reserve_record_capacity(
            guard.len(),
            guard.capacity(),
            guard.record_size(),
            1,
            HtmlSemanticResource::NestedOutputRecords,
            || {
                guard.try_reserve().map_err(|_| ())?;
                Ok(guard.capacity())
            },
        )
    }
}
impl<'a> HtmlSemanticSession<'a> {
    fn rel_action_failure(
        &mut self,
        error: HtmlRelActionError<HtmlSemanticFailure>,
    ) -> HtmlSemanticFailure {
        match error {
            HtmlRelActionError::Admission(error) => error,
            HtmlRelActionError::Protocol(_) => {
                self.stop(HtmlSemanticFailureKind::RepresentationExhausted)
            }
        }
    }
    fn rel_missing(
        subject: &HtmlCaptureSubjectId,
        fact: HtmlRelMissingFact,
    ) -> HtmlSemanticMissing {
        HtmlSemanticMissing::LinkRelationship {
            subject: subject.clone(),
            fact,
        }
    }
    pub(super) fn link_relationships(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlRelContextReadback>, HtmlSemanticFailure> {
        let mut slots = [None; 9];
        for (i, name) in [
            "rel",
            "rev",
            "href",
            "imagesrcset",
            "itemprop",
            "title",
            "type",
            "hreflang",
            "as",
        ]
        .into_iter()
        .enumerate()
        {
            slots[i] = self.attribute(element, name)?.map(Attribute::value);
            self.charge(1)?;
            self.interpreter
                .admit_attribute_raw(slots[i])
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        }
        let role = self.known(element)?.and_then(HtmlRelRole::from_known);
        self.charge(1)?;
        if role.is_none() {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let as_destination = if role == Some(HtmlRelRole::Link) {
            let raw = self.attribute(element, "as")?;
            self.admit_intrinsic(element, raw, 13)?;
            self.interpreter
                .link_as(element.name(), raw)
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?
                .intrinsic()
                .copied()
        } else {
            None
        };
        let inputs = RelInputs::new(role)
            .with_rel(slots[0])
            .with_rev(slots[1])
            .with_href(slots[2])
            .with_imagesrcset(slots[3])
            .with_itemprop(slots[4])
            .with_title(slots[5])
            .with_type_hint(slots[6])
            .with_as_destination(as_destination);
        let mut local = HtmlRelInterpretation::new(inputs);
        local
            .interpret(&mut RelControl { session: self })
            .map_err(|error| self.rel_action_failure(error))?;
        let role = role.expect("checked actual role");
        let facts = self.context.rel_facts.as_ref();
        let document = self.node(subject)?.node_document();
        let mut extensions = Vec::new();
        for (i, token) in local
            .occurrences()
            .expect("interpreted occurrences")
            .iter()
            .enumerate()
        {
            self.charge(1)?;
            if token.spelling() != HtmlRelSpelling::Unknown {
                continue;
            }
            let mut read = HtmlRelExtensionReadback {
                occurrence: i,
                status: None,
                proven_absent: false,
                definition: None,
                canonical: None,
                synonym: None,
                effect: HtmlRelExtensionEffect::Unavailable,
                standard: None,
                absolute_url: None,
                author: HtmlSemanticValue::Missing(Self::rel_missing(
                    subject,
                    HtmlRelMissingFact::Registry,
                )),
            };
            if let Some(facts) = facts {
                for observed in &facts.input.occurrences {
                    self.charge(3)?;
                    if &observed.subject != subject
                        || observed.start != token.code_unit_range().start
                        || observed.end != token.code_unit_range().end
                    {
                        continue;
                    }
                    match &observed.state {
                        RegistryOccurrence::Unavailable => {}
                        RegistryOccurrence::ProvenAbsent => {
                            read.proven_absent = true;
                            read.effect = HtmlRelExtensionEffect::NotAllowed
                        }
                        RegistryOccurrence::Definition { key, synonym } => {
                            for d in &facts.input.definitions {
                                self.charge(1)?;
                                if &d.key == key {
                                    self.charge(5)?;
                                    read.status = Some(d.status);
                                    read.definition = Some(d.key.clone());
                                    read.canonical = Some(d.canonical.clone());
                                    read.synonym = Some(*synonym);
                                    read.standard = d.standard;
                                    read.effect = if let Some(k) = d.standard {
                                        HtmlRelExtensionEffect::from_standard(k.effect(role))
                                    } else {
                                        d.effects[match role {
                                            HtmlRelRole::Link => 0,
                                            HtmlRelRole::Anchor => 1,
                                            HtmlRelRole::Area => 2,
                                            HtmlRelRole::Form => 3,
                                        }]
                                    };
                                    break;
                                }
                            }
                        }
                    }
                    break;
                }
                for g in &facts.input.grammar {
                    self.charge(3)?;
                    if &g.subject == subject
                        && g.input
                            == (HtmlRelGrammarInput::ExtensionOccurrence {
                                start: token.code_unit_range().start,
                                end: token.code_unit_range().end,
                            })
                    {
                        read.absolute_url = Some(g.valid);
                        break;
                    }
                }
            }
            let units = local
                .occurrence_units(i)
                .expect("interpreted owner")
                .expect("actual occurrence");
            let mut colon = false;
            for unit in units {
                self.charge(1)?;
                if *unit == 58 {
                    colon = true;
                    break;
                }
            }
            self.charge(6)?;
            read.author = if read.proven_absent
                || read.status == Some(HtmlRelRegistryStatus::Discontinued)
                || read.synonym == Some(true)
                || read.effect == HtmlRelExtensionEffect::NotAllowed
                || colon && read.absolute_url == Some(false)
            {
                HtmlSemanticValue::Known(false)
            } else if read.status.is_none() || read.effect == HtmlRelExtensionEffect::Unavailable {
                HtmlSemanticValue::Missing(Self::rel_missing(subject, HtmlRelMissingFact::Registry))
            } else if colon && read.absolute_url.is_none() {
                HtmlSemanticValue::Missing(Self::rel_missing(
                    subject,
                    HtmlRelMissingFact::UrlGrammar,
                ))
            } else {
                HtmlSemanticValue::Known(true)
            };
            self.reserve_nested_output(&mut extensions, 1)?;
            self.charge(1)?;
            extensions.push(read);
        }
        let supported = if let Some(facts) = facts {
            let mut found = None;
            for record in &facts.input.supported {
                self.charge(1)?;
                if &record.subject == subject {
                    let mut values = Vec::new();
                    self.reserve_nested_output(&mut values, record.tokens.len())?;
                    for &keyword in &record.tokens {
                        self.charge(1)?;
                        values.push(keyword)
                    }
                    found = Some(values);
                    break;
                }
            }
            found.map_or_else(
                || {
                    HtmlSemanticValue::Missing(Self::rel_missing(
                        subject,
                        HtmlRelMissingFact::SupportedTokens,
                    ))
                },
                HtmlSemanticValue::Known,
            )
        } else {
            HtmlSemanticValue::Missing(Self::rel_missing(
                subject,
                HtmlRelMissingFact::SupportedTokens,
            ))
        };
        local
            .begin_refinement()
            .map_err(|_| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        for extension in &extensions {
            let refinement = if let Some(keyword) = extension.standard {
                HtmlRelRefinement::Standard(keyword)
            } else if extension.effect == HtmlRelExtensionEffect::Unavailable {
                HtmlRelRefinement::Unavailable
            } else if let Some(effect) = extension.effect.effect() {
                HtmlRelRefinement::Extension(effect)
            } else {
                HtmlRelRefinement::NoEffect
            };
            local
                .refine(
                    extension.occurrence,
                    refinement,
                    &mut RelControl { session: self },
                )
                .map_err(|error| self.rel_action_failure(error))?;
        }
        local
            .refresh(&mut RelControl { session: self })
            .map_err(|error| self.rel_action_failure(error))?;
        let local = HtmlRelInterpretation::try_finish(&mut Some(local))
            .map_err(|_| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        for extension in &mut extensions {
            self.charge(2)?;
            if extension.standard.is_some() {
                extension.effect = HtmlRelExtensionEffect::from_standard(
                    local.occurrences()[extension.occurrence].effect(),
                )
            }
        }
        let links = local.links();
        let mut meanings = Vec::new();
        for token in local.occurrences() {
            self.charge(1)?;
            let Some(keyword) = token.keyword() else {
                continue;
            };
            if token.effect().is_none()
                || !matches!(
                    keyword,
                    HtmlRelKeyword::Author
                        | HtmlRelKeyword::Bookmark
                        | HtmlRelKeyword::Help
                        | HtmlRelKeyword::Tag
                        | HtmlRelKeyword::License
                        | HtmlRelKeyword::Canonical
                        | HtmlRelKeyword::Alternate
                        | HtmlRelKeyword::Next
                        | HtmlRelKeyword::Prev
                        | HtmlRelKeyword::Search
                        | HtmlRelKeyword::PrivacyPolicy
                        | HtmlRelKeyword::TermsOfService
                )
            {
                continue;
            }
            let association = match keyword {
                HtmlRelKeyword::Author
                    if matches!(role, HtmlRelRole::Anchor | HtmlRelRole::Area) =>
                {
                    self.rel_article_or_page(subject, false)?
                }
                HtmlRelKeyword::Bookmark => self.rel_article_or_page(subject, true)?,
                HtmlRelKeyword::Help if role != HtmlRelRole::Link => {
                    let node = self.node(subject)?;
                    match self.parent(node)? {
                        Ok(parent) => {
                            HtmlSemanticValue::Known(HtmlRelAssociation::ParentAndChildren(parent))
                        }
                        Err(missing) => HtmlSemanticValue::Missing(missing),
                    }
                }
                _ => HtmlSemanticValue::Known(HtmlRelAssociation::Page(document.clone())),
            };
            self.reserve_nested_output(&mut meanings, 1)?;
            self.charge(1)?;
            meanings.push(HtmlRelMeaning {
                keyword,
                association,
            });
        }
        let translation =
            if local.alternate() != crate::HtmlRelAlternate::Representation || slots[7].is_none() {
                HtmlSemanticValue::Inapplicable
            } else {
                self.rel_translation(subject, slots[7].expect("selected hreflang"))?
            };
        let mut href_grammar = None;
        if let Some(facts) = facts {
            for g in &facts.input.grammar {
                self.charge(2)?;
                if &g.subject == subject && g.input == HtmlRelGrammarInput::Href {
                    href_grammar = Some(g.valid);
                    break;
                }
            }
        }
        self.charge(3)?;
        let href_author = match local.href_author() {
            crate::HtmlRelHrefAuthor::NotRequired => HtmlSemanticValue::Inapplicable,
            crate::HtmlRelHrefAuthor::EmptyOrOnlyAsciiSpaces => HtmlSemanticValue::Known(false),
            crate::HtmlRelHrefAuthor::GrammarUnchecked => href_grammar.map_or_else(
                || {
                    HtmlSemanticValue::Missing(Self::rel_missing(
                        subject,
                        HtmlRelMissingFact::UrlGrammar,
                    ))
                },
                HtmlSemanticValue::Known,
            ),
        };
        let history = if role != HtmlRelRole::Link {
            HtmlSemanticValue::Inapplicable
        } else {
            element.link_history().map_or_else(
                || {
                    HtmlSemanticValue::Missing(Self::rel_missing(
                        subject,
                        HtmlRelMissingFact::LinkHistory,
                    ))
                },
                HtmlSemanticValue::Known,
            )
        };
        let mut stylesheet = false;
        let mut unresolved = false;
        for occurrence in local.occurrences() {
            self.charge(4)?;
            stylesheet |= occurrence.keyword() == Some(HtmlRelKeyword::Stylesheet)
                && occurrence.effect().is_some();
            unresolved |= occurrence.spelling() == HtmlRelSpelling::Unknown
                && !occurrence.registry_resolved();
        }
        self.charge(4)?;
        let image_source_preload_author = if role != HtmlRelRole::Link
            || !local.imagesrcset_present()
        {
            HtmlSemanticValue::Inapplicable
        } else if local.image_source_preload_context() {
            HtmlSemanticValue::Known(true)
        } else if local.as_destination() != Some(crate::HtmlLinkDestination::Image) || !unresolved {
            HtmlSemanticValue::Known(false)
        } else {
            HtmlSemanticValue::Missing(Self::rel_missing(subject, HtmlRelMissingFact::Registry))
        };
        let implicit_render_blocking = if role != HtmlRelRole::Link {
            HtmlSemanticValue::Inapplicable
        } else if stylesheet {
            match &history {
                HtmlSemanticValue::Known(h) => {
                    HtmlSemanticValue::Known(h.created_by_node_document_parser())
                }
                HtmlSemanticValue::Missing(m) => HtmlSemanticValue::Missing(m.clone()),
                _ => unreachable!(),
            }
        } else if unresolved {
            HtmlSemanticValue::Missing(Self::rel_missing(subject, HtmlRelMissingFact::Registry))
        } else {
            HtmlSemanticValue::Known(false)
        };
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(RelContextData {
            local,
            extensions,
            supported,
            links,
            meanings,
            translation,
            href_grammar,
            href_author,
            image_source_preload_author,
            history,
            implicit_render_blocking,
        });
        Ok(HtmlSemanticValue::Known(HtmlRelContextReadback { data }))
    }
    fn rel_article_or_page(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        bookmark: bool,
    ) -> Result<HtmlSemanticValue<HtmlRelAssociation>, HtmlSemanticFailure> {
        let initial = self.node(subject)?;
        let document = initial.node_document();
        let mut node = initial;
        loop {
            match self.parent(node)? {
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                Ok(None) => break,
                Ok(Some(parent)) => {
                    node = self.node(&parent)?;
                    if let HtmlCapturedNodeKind::Element(e) = node.kind()
                        && self.known(e)? == Some(KnownHtmlElement::Article)
                    {
                        return Ok(HtmlSemanticValue::Known(HtmlRelAssociation::Article(
                            parent,
                        )));
                    }
                }
            }
        }
        if !bookmark {
            return Ok(HtmlSemanticValue::Known(HtmlRelAssociation::Page(document)));
        }
        if let Some(facts) = self.context.rel_facts.as_ref() {
            for record in &facts.input.sections {
                self.charge(1)?;
                if &record.subject == subject {
                    return Ok(HtmlSemanticValue::Known(HtmlRelAssociation::Section(
                        record.section.clone(),
                    )));
                }
            }
        }
        Ok(HtmlSemanticValue::Missing(Self::rel_missing(
            subject,
            HtmlRelMissingFact::BookmarkSection,
        )))
    }
    fn rel_translation(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        hreflang: &DocumentString,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let document = node.node_document();
        let node = self.node(document.subject())?;
        let mut children = match node.children() {
            HtmlCapturedChildren::Complete(children) => children,
            HtmlCapturedChildren::Unavailable => {
                return Ok(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::OrderedChildren(node.id()),
                ));
            }
        };
        let mut root = None;
        loop {
            self.charge(1)?;
            let Some(child) = children.next() else { break };
            let node = self.node(&child)?;
            if matches!(node.kind(), HtmlCapturedNodeKind::Element(_)) {
                if root.is_some() {
                    return Ok(HtmlSemanticValue::Unsupported(
                        HtmlSemanticUnsupported::AmbiguousDocumentElement,
                    ));
                }
                root = Some(child)
            }
        }
        let Some(root) = root else {
            return Ok(HtmlSemanticValue::Missing(Self::rel_missing(
                subject,
                HtmlRelMissingFact::DocumentElementLanguage,
            )));
        };
        match self.language(&root)? {
            HtmlSemanticValue::Known(language) => {
                let raw = language.tag();
                let envelope = raw
                    .len()
                    .checked_add(hreflang.len())
                    .and_then(|n| n.checked_add(1))
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
                self.charge(envelope)?;
                Ok(HtmlSemanticValue::Known(raw != hreflang))
            }
            HtmlSemanticValue::Missing(m) => Ok(HtmlSemanticValue::Missing(m)),
            HtmlSemanticValue::Inapplicable => Ok(HtmlSemanticValue::Inapplicable),
            HtmlSemanticValue::Unsupported(u) => Ok(HtmlSemanticValue::Unsupported(u)),
        }
    }
}

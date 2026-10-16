//! HTML877d document-base and effective-target rules over declared URL facts.
//!
//! URL parsing, serialization, record predicates and CSP truth are adapter
//! attestations. This owner checks their qualification; it is no URL engine.

use crate::{
    DocumentString, HtmlCapture, HtmlCaptureDocumentId, HtmlCaptureError, HtmlCaptureId,
    HtmlCaptureSubjectId, HtmlCapturedNodeKind, HtmlHostFactsRevision,
};
use std::{error::Error, fmt, sync::Arc};

/// Identity of an external URL/CSP observation producer.
#[derive(Clone, Debug)]
pub struct HtmlBaseProducerId(Arc<()>);
impl HtmlBaseProducerId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlBaseProducerId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlBaseProducerId {}

/// Every influencing external configuration, URL and policy revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlBaseFactsRevision {
    producer: HtmlBaseProducerId,
    configuration: u64,
    urls: u64,
    csp: u64,
}
impl HtmlBaseFactsRevision {
    pub fn new(producer: HtmlBaseProducerId, configuration: u64, urls: u64, csp: u64) -> Self {
        Self {
            producer,
            configuration,
            urls,
            csp,
        }
    }
    pub fn producer(&self) -> &HtmlBaseProducerId {
        &self.producer
    }
    pub fn configuration(&self) -> u64 {
        self.configuration
    }
    pub fn urls(&self) -> u64 {
        self.urls
    }
    pub fn csp(&self) -> u64 {
        self.csp
    }
}

/// URL scheme branches consumed by the base algorithm, externally observed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlBaseUrlScheme {
    Data,
    Javascript,
    Other,
}

#[derive(Debug)]
struct UrlRecord {
    revision: HtmlBaseFactsRevision,
    serialization: DocumentString,
    scheme: HtmlBaseUrlScheme,
    about_blank: bool,
}
/// An actual external record observation. Equality is record identity, not text.
/// Its constructor attests URL-standard truth; HTML does not parse its spelling.
#[derive(Clone, Debug)]
pub struct HtmlBaseUrlRecord(Arc<UrlRecord>);
impl HtmlBaseUrlRecord {
    pub fn observed(
        revision: HtmlBaseFactsRevision,
        serialization: DocumentString,
        scheme: HtmlBaseUrlScheme,
        matches_about_blank: bool,
    ) -> Result<Self, HtmlBaseFactsError> {
        if matches_about_blank && scheme != HtmlBaseUrlScheme::Other {
            return Err(HtmlBaseFactsError::ContradictoryObservation);
        }
        Ok(Self(Arc::new(UrlRecord {
            revision,
            serialization,
            scheme,
            about_blank: matches_about_blank,
        })))
    }
    pub fn serialization(&self) -> &DocumentString {
        &self.0.serialization
    }
    pub fn revision(&self) -> &HtmlBaseFactsRevision {
        &self.0.revision
    }
    pub fn scheme(&self) -> HtmlBaseUrlScheme {
        self.0.scheme
    }
    pub fn matches_about_blank(&self) -> bool {
        self.0.about_blank
    }
}
impl PartialEq for HtmlBaseUrlRecord {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlBaseUrlRecord {}

/// External character-encoding identity, independent of source coordinates.
#[derive(Clone, Debug)]
pub struct HtmlBaseEncoding(Arc<()>);
impl HtmlBaseEncoding {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlBaseEncoding {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlBaseEncoding {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAboutBaseUrl {
    KnownNull,
    KnownRecord(HtmlBaseUrlRecord),
    Unavailable,
}

/// A genuinely influencing unavailable input, never a URL failure or CSP block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlBaseMissing {
    DocumentFacts,
    DocumentUrl,
    AboutBaseUrl,
    CharacterEncoding,
    ParseObservation,
    Csp,
    FrozenBase,
    ResponseDomain,
}

/// Exact document environment; iframe-srcdoc classification is supplied truth.
#[derive(Clone, Debug)]
pub struct HtmlBaseDocumentFacts {
    revision: HtmlBaseFactsRevision,
    url: Option<HtmlBaseUrlRecord>,
    about: HtmlAboutBaseUrl,
    srcdoc: bool,
    encoding: Option<HtmlBaseEncoding>,
}
impl HtmlBaseDocumentFacts {
    pub fn new(
        revision: HtmlBaseFactsRevision,
        url: Option<HtmlBaseUrlRecord>,
        about: HtmlAboutBaseUrl,
        iframe_srcdoc: bool,
        encoding: Option<HtmlBaseEncoding>,
    ) -> Result<Self, HtmlBaseFactsError> {
        if iframe_srcdoc && matches!(about, HtmlAboutBaseUrl::KnownNull) {
            return Err(HtmlBaseFactsError::ContradictoryObservation);
        }
        if url.as_ref().is_some_and(|url| url.revision() != &revision) {
            return Err(HtmlBaseFactsError::RevisionMismatch);
        }
        // about-base may be a historical record; its producing revision need
        // not equal today's document URL revision.
        Ok(Self {
            revision,
            url,
            about,
            srcdoc: iframe_srcdoc,
            encoding,
        })
    }
    pub fn revision(&self) -> &HtmlBaseFactsRevision {
        &self.revision
    }
    pub fn document_url(&self) -> Option<&HtmlBaseUrlRecord> {
        self.url.as_ref()
    }
    pub fn about_base_url(&self) -> &HtmlAboutBaseUrl {
        &self.about
    }
    pub fn iframe_srcdoc(&self) -> bool {
        self.srcdoc
    }
    pub fn encoding(&self) -> Option<&HtmlBaseEncoding> {
        self.encoding.as_ref()
    }
    /// Selects the existing document fallback from qualified URL/about facts.
    /// This borrows a declared record and performs no URL parsing.
    pub fn fallback(&self) -> Result<&HtmlBaseUrlRecord, HtmlBaseMissing> {
        if self.srcdoc {
            return match &self.about {
                HtmlAboutBaseUrl::KnownRecord(url) => Ok(url),
                _ => Err(HtmlBaseMissing::AboutBaseUrl),
            };
        }
        let url = self.url.as_ref().ok_or(HtmlBaseMissing::DocumentUrl)?;
        if url.matches_about_blank() {
            match &self.about {
                HtmlAboutBaseUrl::KnownRecord(about) => return Ok(about),
                HtmlAboutBaseUrl::Unavailable => return Err(HtmlBaseMissing::AboutBaseUrl),
                HtmlAboutBaseUrl::KnownNull => {}
            }
        }
        Ok(url)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlBaseAllowed {
    Allowed,
    Blocked,
    Unavailable,
}
#[derive(Clone, Debug)]
pub enum HtmlBaseParseOutcome {
    Failure,
    Parsed {
        record: HtmlBaseUrlRecord,
        base_allowed: HtmlBaseAllowed,
    },
}
/// Declared parse of these exact value/environment inputs, not a node selector.
#[derive(Debug)]
pub struct HtmlBaseParseObservation {
    raw: DocumentString,
    fallback: HtmlBaseUrlRecord,
    encoding: HtmlBaseEncoding,
    revision: HtmlBaseFactsRevision,
    outcome: HtmlBaseParseOutcome,
}
impl HtmlBaseParseObservation {
    pub fn new(
        raw: DocumentString,
        fallback: HtmlBaseUrlRecord,
        encoding: HtmlBaseEncoding,
        revision: HtmlBaseFactsRevision,
        outcome: HtmlBaseParseOutcome,
    ) -> Result<Self, HtmlBaseFactsError> {
        if matches!(&outcome, HtmlBaseParseOutcome::Parsed { record, .. } if record.revision() != &revision)
        {
            return Err(HtmlBaseFactsError::RevisionMismatch);
        }
        Ok(Self {
            raw,
            fallback,
            encoding,
            revision,
            outcome,
        })
    }
    /// Borrows the exact original observed input, without normalization.
    pub fn raw(&self) -> &DocumentString {
        &self.raw
    }
    /// Compares the URL-record and encoding identities and producer revision.
    /// It does not compare URL serialization or qualify the raw input.
    pub fn inputs_match(
        &self,
        fallback: &HtmlBaseUrlRecord,
        encoding: &HtmlBaseEncoding,
        revision: &HtmlBaseFactsRevision,
    ) -> bool {
        &self.fallback == fallback && &self.encoding == encoding && &self.revision == revision
    }
    /// Borrows the declared parse/CSP observation.
    pub fn outcome(&self) -> &HtmlBaseParseOutcome {
        &self.outcome
    }
}

/// Bounded external input formation, separate from parse execution usage.
#[derive(Clone, Copy, Debug)]
pub struct HtmlBaseFactsLimits {
    records: usize,
    utf16_units: usize,
    validation_work: usize,
}
impl HtmlBaseFactsLimits {
    pub fn new(records: usize, utf16_units: usize, validation_work: usize) -> Self {
        Self {
            records,
            utf16_units,
            validation_work,
        }
    }
}
/// External input formation observations, separate from parser-owned tickets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlBaseFactsUsage {
    records: usize,
    utf16_units: usize,
    validation_work: usize,
}
impl HtmlBaseFactsUsage {
    pub fn record_capacity(&self) -> usize {
        self.records
    }
    pub fn utf16_extent(&self) -> usize {
        self.utf16_units
    }
    pub fn validation_work(&self) -> usize {
        self.validation_work
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlBaseFactsError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    RevisionMismatch,
    ContradictoryObservation,
    DuplicateObservation,
    RecordLimit,
    Utf16Limit,
    WorkLimit,
    RepresentationExhausted,
    WrongBase,
    WrongDocument,
}
impl fmt::Display for HtmlBaseFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "base facts: {self:?}")
    }
}
impl Error for HtmlBaseFactsError {}
impl From<HtmlCaptureError> for HtmlBaseFactsError {
    fn from(error: HtmlCaptureError) -> Self {
        Self::Capture(error)
    }
}
pub(crate) struct Validation {
    limits: HtmlBaseFactsLimits,
    work: usize,
    units: usize,
}
impl Validation {
    pub(crate) fn new(limits: HtmlBaseFactsLimits) -> Self {
        Self {
            limits,
            work: 0,
            units: 0,
        }
    }
    pub(crate) fn records(&self, capacity: usize) -> Result<(), HtmlBaseFactsError> {
        if capacity > self.limits.records {
            return Err(HtmlBaseFactsError::RecordLimit);
        }
        Ok(())
    }
    pub(crate) fn usage(&self, records: usize) -> HtmlBaseFactsUsage {
        HtmlBaseFactsUsage {
            records,
            utf16_units: self.units,
            validation_work: self.work,
        }
    }
    pub(crate) fn step(&mut self) -> Result<(), HtmlBaseFactsError> {
        self.work = self
            .work
            .checked_add(1)
            .ok_or(HtmlBaseFactsError::RepresentationExhausted)?;
        if self.work > self.limits.validation_work {
            return Err(HtmlBaseFactsError::WorkLimit);
        }
        Ok(())
    }
    pub(crate) fn string(&mut self, text: &DocumentString) -> Result<(), HtmlBaseFactsError> {
        self.step()?;
        self.units = self
            .units
            .checked_add(text.len())
            .ok_or(HtmlBaseFactsError::RepresentationExhausted)?;
        if self.units > self.limits.utf16_units {
            return Err(HtmlBaseFactsError::Utf16Limit);
        }
        Ok(())
    }
    pub(crate) fn retained_string(
        &mut self,
        text: &DocumentString,
    ) -> Result<(), HtmlBaseFactsError> {
        self.step()?;
        self.units = self
            .units
            .checked_add(text.shared_code_units().capacity())
            .ok_or(HtmlBaseFactsError::RepresentationExhausted)?;
        if self.units > self.limits.utf16_units {
            return Err(HtmlBaseFactsError::Utf16Limit);
        }
        Ok(())
    }
    pub(crate) fn equal(
        &mut self,
        a: &DocumentString,
        b: &DocumentString,
    ) -> Result<bool, HtmlBaseFactsError> {
        self.step()?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for i in 0..a.len() {
            self.step()?;
            if a.code_units()[i] != b.code_units()[i] {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn equal_ascii(&mut self, a: &DocumentString, b: &[u8]) -> Result<bool, HtmlBaseFactsError> {
        self.step()?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (i, byte) in b.iter().enumerate() {
            self.step()?;
            if a.code_units()[i] != u16::from(*byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
#[derive(Debug)]
struct ParseFacts {
    document: HtmlBaseDocumentFacts,
    observations: Vec<HtmlBaseParseObservation>,
    usage: HtmlBaseFactsUsage,
}
/// Immutable original input backing. Clones share its issuer and actual values.
#[derive(Clone, Debug)]
pub struct HtmlParseBaseFacts(Arc<ParseFacts>);
impl HtmlParseBaseFacts {
    pub fn new(
        document: HtmlBaseDocumentFacts,
        observations: Vec<HtmlBaseParseObservation>,
        limits: HtmlBaseFactsLimits,
    ) -> Result<Self, HtmlBaseFactsError> {
        if observations.capacity() > limits.records {
            return Err(HtmlBaseFactsError::RecordLimit);
        }
        let mut v = Validation {
            limits,
            work: 0,
            units: 0,
        };
        if let Some(url) = &document.url {
            v.string(url.serialization())?;
        }
        if let HtmlAboutBaseUrl::KnownRecord(url) = &document.about {
            v.string(url.serialization())?;
        }
        for (i, observation) in observations.iter().enumerate() {
            v.step()?;
            if observation.revision != document.revision {
                return Err(HtmlBaseFactsError::RevisionMismatch);
            }
            v.string(&observation.raw)?;
            v.string(observation.fallback.serialization())?;
            if let HtmlBaseParseOutcome::Parsed { record, .. } = &observation.outcome {
                v.string(record.serialization())?;
            }
            for previous in &observations[..i] {
                v.step()?;
                if previous.inputs_match(
                    &observation.fallback,
                    &observation.encoding,
                    &observation.revision,
                ) && v.equal(&previous.raw, &observation.raw)?
                {
                    return Err(HtmlBaseFactsError::DuplicateObservation);
                }
            }
        }
        let usage = HtmlBaseFactsUsage {
            records: observations.capacity(),
            utf16_units: v.units,
            validation_work: v.work,
        };
        Ok(Self(Arc::new(ParseFacts {
            document,
            observations,
            usage,
        })))
    }
    pub fn document(&self) -> &HtmlBaseDocumentFacts {
        &self.0.document
    }
    pub fn formation_usage(&self) -> HtmlBaseFactsUsage {
        self.0.usage
    }
    /// Borrows the retained observations in their original order.
    pub fn observations(&self) -> &[HtmlBaseParseObservation] {
        &self.0.observations
    }
    /// Whether these values share the exact immutable parse-facts backing.
    /// Equal revisions or observation values alone do not imply the same owner.
    pub fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// Captured historical frozen slot, including the exact current href binding.
#[derive(Debug)]
pub struct HtmlCapturedFrozenBase {
    pub(crate) subject: HtmlCaptureSubjectId,
    pub(crate) document: HtmlCaptureDocumentId,
    pub(crate) href: DocumentString,
    pub(crate) value: Option<HtmlBaseUrlRecord>,
    pub(crate) frozen_revision: HtmlBaseFactsRevision,
}
impl HtmlCapturedFrozenBase {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        document: HtmlCaptureDocumentId,
        href: DocumentString,
        value: Option<HtmlBaseUrlRecord>,
        frozen_revision: HtmlBaseFactsRevision,
    ) -> Self {
        Self {
            subject,
            document,
            href,
            value,
            frozen_revision,
        }
    }
}
#[derive(Debug)]
pub struct HtmlCapturedBaseDocument {
    pub(crate) document: HtmlCaptureDocumentId,
    pub(crate) facts: HtmlParseBaseFacts,
}
impl HtmlCapturedBaseDocument {
    pub fn new(document: HtmlCaptureDocumentId, facts: HtmlParseBaseFacts) -> Self {
        Self { document, facts }
    }
}
/// Qualified external facts over one immutable capture and host revision.
#[derive(Debug)]
pub struct HtmlCapturedBaseFacts {
    capture: HtmlCaptureId,
    host: HtmlHostFactsRevision,
    revision: HtmlBaseFactsRevision,
    pub(crate) documents: Vec<HtmlCapturedBaseDocument>,
    pub(crate) frozen: Vec<HtmlCapturedFrozenBase>,
    usage: HtmlBaseFactsUsage,
}
impl HtmlCapturedBaseFacts {
    pub fn new(
        capture: &HtmlCapture,
        host: HtmlHostFactsRevision,
        revision: HtmlBaseFactsRevision,
        documents: Vec<HtmlCapturedBaseDocument>,
        frozen: Vec<HtmlCapturedFrozenBase>,
        limits: HtmlBaseFactsLimits,
    ) -> Result<Self, HtmlBaseFactsError> {
        if &host != capture.host_facts_revision() {
            return Err(HtmlBaseFactsError::HostRevisionMismatch);
        }
        if documents
            .capacity()
            .checked_add(frozen.capacity())
            .ok_or(HtmlBaseFactsError::RepresentationExhausted)?
            > limits.records
        {
            return Err(HtmlBaseFactsError::RecordLimit);
        }
        let mut v = Validation {
            limits,
            work: 0,
            units: 0,
        };
        for (i, row) in documents.iter().enumerate() {
            v.step()?;
            capture.document_id(row.document.subject())?;
            if row.facts.document().revision() != &revision {
                return Err(HtmlBaseFactsError::RevisionMismatch);
            }
            for old in &documents[..i] {
                v.step()?;
                if old.document == row.document {
                    return Err(HtmlBaseFactsError::DuplicateObservation);
                }
            }
        }
        for (i, row) in frozen.iter().enumerate() {
            v.step()?;
            let node = capture.node(&row.subject)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlBaseFactsError::WrongBase);
            };
            if !v.equal_ascii(element.name().local_name(), b"base")?
                || !v.equal_ascii(
                    element
                        .name()
                        .namespace()
                        .uri()
                        .ok_or(HtmlBaseFactsError::WrongBase)?,
                    b"http://www.w3.org/1999/xhtml",
                )?
            {
                return Err(HtmlBaseFactsError::WrongBase);
            }
            if node.node_document() != row.document {
                return Err(HtmlBaseFactsError::WrongDocument);
            }
            let mut href = None;
            for attribute in element.attributes().as_slice() {
                v.step()?;
                if attribute.name().namespace().uri().is_none()
                    && v.equal_ascii(attribute.name().local_name(), b"href")?
                {
                    href = Some(attribute.value());
                    break;
                }
            }
            if !v.equal(href.ok_or(HtmlBaseFactsError::WrongBase)?, &row.href)? {
                return Err(HtmlBaseFactsError::WrongBase);
            }
            v.string(&row.href)?;
            if let Some(value) = &row.value {
                if value.revision() != &row.frozen_revision {
                    return Err(HtmlBaseFactsError::RevisionMismatch);
                }
                v.string(value.serialization())?;
            }
            for old in &frozen[..i] {
                v.step()?;
                if old.subject == row.subject {
                    return Err(HtmlBaseFactsError::DuplicateObservation);
                }
            }
        }
        let usage = HtmlBaseFactsUsage {
            records: documents.capacity() + frozen.capacity(),
            utf16_units: v.units,
            validation_work: v.work,
        };
        Ok(Self {
            capture: capture.identity().clone(),
            host,
            revision,
            documents,
            frozen,
            usage,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host
    }
    pub fn revision(&self) -> &HtmlBaseFactsRevision {
        &self.revision
    }
    pub fn formation_usage(&self) -> HtmlBaseFactsUsage {
        self.usage
    }
}

/// Actual ordinary-tree first Base selection, supplied by the topology owner.
#[derive(Clone, Copy, Debug)]
pub enum HtmlDocumentFirstBase<'a> {
    Absent,
    /// A first Base exists; None is unavailable frozen state, never fallback.
    Present(Option<&'a HtmlBaseUrlRecord>),
}
/// HTML #document-base-url selection without a second tree or URL engine.
/// The first frozen record may be historical; its revision need not equal the
/// current Document facts. That branch requires no unrelated fallback facts.
pub fn select_document_base<'a>(
    document: Option<&'a HtmlBaseDocumentFacts>,
    first: HtmlDocumentFirstBase<'a>,
) -> Result<&'a HtmlBaseUrlRecord, HtmlBaseMissing> {
    match first {
        HtmlDocumentFirstBase::Present(value) => value.ok_or(HtmlBaseMissing::FrozenBase),
        HtmlDocumentFirstBase::Absent => document.ok_or(HtmlBaseMissing::DocumentFacts)?.fallback(),
    }
}

/// Selects the historical frozen base from a qualified external outcome.
/// The boolean requests the existing base-response steps. The returned record
/// borrows the supplied fallback or outcome; this does not qualify provenance
/// or install parser state.
pub fn select_frozen_base<'a>(
    fallback: &'a HtmlBaseUrlRecord,
    outcome: &'a HtmlBaseParseOutcome,
) -> Result<(&'a HtmlBaseUrlRecord, bool), HtmlBaseMissing> {
    match outcome {
        HtmlBaseParseOutcome::Failure => Ok((fallback, false)),
        HtmlBaseParseOutcome::Parsed {
            record,
            base_allowed,
        } => {
            if matches!(
                record.scheme(),
                HtmlBaseUrlScheme::Data | HtmlBaseUrlScheme::Javascript
            ) {
                return Ok((fallback, false));
            }
            match base_allowed {
                HtmlBaseAllowed::Blocked => Ok((fallback, false)),
                HtmlBaseAllowed::Allowed => Ok((record, true)),
                HtmlBaseAllowed::Unavailable => Err(HtmlBaseMissing::Csp),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn revision() -> HtmlBaseFactsRevision {
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1)
    }
    fn record(r: &HtmlBaseFactsRevision, blank: bool) -> HtmlBaseUrlRecord {
        HtmlBaseUrlRecord::observed(
            r.clone(),
            if blank {
                "about:blank?x#y"
            } else {
                "https://fallback.test/"
            }
            .into(),
            HtmlBaseUrlScheme::Other,
            blank,
        )
        .unwrap()
    }
    #[test]
    fn fallback_guards_consult_about_only_for_blank_or_srcdoc() {
        let r = revision();
        let normal = record(&r, false);
        let blank = record(&r, true);
        let about = HtmlBaseUrlRecord::observed(
            r.clone(),
            "https://creator.test/".into(),
            HtmlBaseUrlScheme::Other,
            false,
        )
        .unwrap();
        for facts in [
            HtmlBaseDocumentFacts::new(
                r.clone(),
                Some(normal.clone()),
                HtmlAboutBaseUrl::Unavailable,
                false,
                None,
            )
            .unwrap(),
            HtmlBaseDocumentFacts::new(
                r.clone(),
                Some(blank.clone()),
                HtmlAboutBaseUrl::KnownNull,
                false,
                None,
            )
            .unwrap(),
            HtmlBaseDocumentFacts::new(
                r.clone(),
                Some(blank.clone()),
                HtmlAboutBaseUrl::KnownRecord(about.clone()),
                false,
                None,
            )
            .unwrap(),
            HtmlBaseDocumentFacts::new(
                r.clone(),
                None,
                HtmlAboutBaseUrl::KnownRecord(about.clone()),
                true,
                None,
            )
            .unwrap(),
        ]
        .into_iter()
        .zip([&normal, &blank, &about, &about])
        {
            assert_eq!(facts.0.fallback(), Ok(facts.1));
        }
        assert_eq!(
            HtmlBaseDocumentFacts::new(
                r.clone(),
                Some(blank),
                HtmlAboutBaseUrl::Unavailable,
                false,
                None
            )
            .unwrap()
            .fallback(),
            Err(HtmlBaseMissing::AboutBaseUrl)
        );
        assert_eq!(
            HtmlBaseDocumentFacts::new(r.clone(), None, HtmlAboutBaseUrl::Unavailable, false, None)
                .unwrap()
                .fallback(),
            Err(HtmlBaseMissing::DocumentUrl)
        );
        assert_eq!(
            HtmlBaseDocumentFacts::new(r.clone(), None, HtmlAboutBaseUrl::Unavailable, true, None)
                .unwrap()
                .fallback(),
            Err(HtmlBaseMissing::AboutBaseUrl)
        );
        assert_eq!(
            HtmlBaseDocumentFacts::new(r, None, HtmlAboutBaseUrl::KnownNull, true, None)
                .unwrap_err(),
            HtmlBaseFactsError::ContradictoryObservation
        );
    }
    #[test]
    fn no_observation_formation_charges_one_real_header_and_exact_retained_units() {
        let r = revision();
        let url =
            HtmlBaseUrlRecord::observed(r.clone(), "X".into(), HtmlBaseUrlScheme::Other, false)
                .unwrap();
        let facts =
            HtmlBaseDocumentFacts::new(r, Some(url), HtmlAboutBaseUrl::KnownNull, false, None)
                .unwrap();
        // Validation::string admits its one header; the document wrapper itself
        // performs no input scan. One actual external URL unit is retained.
        let exact =
            HtmlParseBaseFacts::new(facts.clone(), vec![], HtmlBaseFactsLimits::new(0, 1, 1))
                .unwrap();
        assert_eq!(
            exact.formation_usage(),
            HtmlBaseFactsUsage {
                records: 0,
                utf16_units: 1,
                validation_work: 1
            }
        );
        assert_eq!(
            HtmlParseBaseFacts::new(facts.clone(), vec![], HtmlBaseFactsLimits::new(0, 0, 1))
                .unwrap_err(),
            HtmlBaseFactsError::Utf16Limit
        );
        assert_eq!(
            HtmlParseBaseFacts::new(facts, vec![], HtmlBaseFactsLimits::new(0, 1, 0)).unwrap_err(),
            HtmlBaseFactsError::WorkLimit
        );
    }
}

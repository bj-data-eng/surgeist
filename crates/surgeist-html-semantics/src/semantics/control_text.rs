//! Exact textarea/output text getters, HTML 877d6146487f §§4.10.11–12.
//! DOM child/descendant text content preserves ordinary Text data. Setters,
//! reset and establishing an output's actual default override are external.

use super::index::FactIndex;
use super::scan::{Caps, ScanError, ScanMeter};
use super::*;
use surgeist_html::accounting::HtmlAccountingFault;
use surgeist_html::accounting::{HtmlCapScope, HtmlResource};
use surgeist_html::{values::DocumentString, vocabulary::KnownHtmlElement};

/// Actual output slot: established null differs from missing evidence and empty.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlOutputDefaultValueOverride {
    Unavailable,
    Null,
    Value(DocumentString),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlOutputDefaultFact {
    subject: HtmlCaptureSubjectId,
    default_value_override: HtmlOutputDefaultValueOverride,
}
impl HtmlOutputDefaultFact {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        default_value_override: HtmlOutputDefaultValueOverride,
    ) -> Self {
        Self {
            subject,
            default_value_override,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn default_value_override(&self) -> &HtmlOutputDefaultValueOverride {
        &self.default_value_override
    }
}
/// Moved vector capacity and logical retained string units are finite. Existing
/// immutable backing allocation is not reallocated or measured by this input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlOutputDefaultLimits {
    record_capacity: usize,
    code_units: usize,
    validation_work: usize,
}
impl HtmlOutputDefaultLimits {
    pub fn new(record_capacity: usize, code_units: usize, validation_work: usize) -> Self {
        Self {
            record_capacity,
            code_units,
            validation_work,
        }
    }
    pub fn record_capacity(self) -> usize {
        self.record_capacity
    }
    pub fn code_units(self) -> usize {
        self.code_units
    }
    pub fn validation_work(self) -> usize {
        self.validation_work
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlOutputDefaultError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    RecordCapacity {
        limit: usize,
        required: usize,
    },
    CodeUnits {
        limit: usize,
        required: usize,
    },
    ValidationWork {
        limit: usize,
        required: usize,
    },
    RepresentationExhausted,
    /// The ledger that carries the accounting failed in a way that is not a refused limit: a release
    /// that matches nothing held, a ledger that cannot allocate, a scope that is not the
    /// operation's, or a change the operation does not charge. It carries the accounting fault.
    Accounting(HtmlAccountingFault),
    DuplicateSubject(HtmlCaptureSubjectId),
    InapplicableSubject(HtmlCaptureSubjectId),
}
impl HtmlOutputDefaultError {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Accounting(fault) => Some(*fault),
            Self::Capture(error) => error.accounting_fault(),
            Self::HostRevisionMismatch
            | Self::RecordCapacity { .. }
            | Self::CodeUnits { .. }
            | Self::ValidationWork { .. }
            | Self::RepresentationExhausted
            | Self::DuplicateSubject(_)
            | Self::InapplicableSubject(_) => None,
        }
    }
}
impl ScanError for HtmlOutputDefaultError {
    fn representation_exhausted() -> Self {
        Self::RepresentationExhausted
    }
    fn work_exceeded(limit: usize, required: usize) -> Self {
        Self::ValidationWork { limit, required }
    }
    fn capture(error: HtmlCaptureError) -> Self {
        Self::Capture(error)
    }
    fn accounting(fault: HtmlAccountingFault) -> Self {
        Self::Accounting(fault)
    }
}
impl fmt::Display for HtmlOutputDefaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlOutputDefaultError {}

/// Checked actual output-default observations for one capture and host revision.
/// No initial null state is inferred for an arbitrary existing capture.
#[derive(Debug)]
pub struct HtmlOutputDefaultFacts {
    capture: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    records: Vec<HtmlOutputDefaultFact>,
    index: FactIndex,
    validation_work: usize,
}
impl HtmlOutputDefaultFacts {
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        records: Vec<HtmlOutputDefaultFact>,
        limits: HtmlOutputDefaultLimits,
    ) -> Result<Self, HtmlOutputDefaultError> {
        Self::build(capture, host_revision, records, limits, None)
    }
    /// Charges below the scope `accounting` names, in the host's operation, as [`Self::new`]
    /// charges an operation of its own. The schema module documentation of
    /// `surgeist_html::accounting` lists the scopes it creates and what the host keeps.
    pub fn with_accounting(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        records: Vec<HtmlOutputDefaultFact>,
        limits: HtmlOutputDefaultLimits,
        accounting: &HtmlHostScope,
    ) -> Result<Self, HtmlOutputDefaultError> {
        Self::build(capture, host_revision, records, limits, Some(accounting))
    }
    fn build(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        records: Vec<HtmlOutputDefaultFact>,
        limits: HtmlOutputDefaultLimits,
        hosted: Option<&HtmlHostScope>,
    ) -> Result<Self, HtmlOutputDefaultError> {
        if &host_revision != capture.host_facts_revision() {
            return Err(HtmlOutputDefaultError::HostRevisionMismatch);
        }
        let mut meter = ScanMeter::charging(
            HtmlCapScope::OutputDefault,
            Caps::work(limits.validation_work)
                .records(limits.record_capacity)
                .code_units(limits.code_units),
            hosted,
        )?;
        meter.acquire(
            HtmlResource::FactRecords,
            records.capacity(),
            |limit, required| HtmlOutputDefaultError::RecordCapacity { limit, required },
        )?;
        let mut index = FactIndex::new(
            capture.identity(),
            records.len(),
            |n| meter.charge(n),
            HtmlOutputDefaultError::RepresentationExhausted,
        )?;
        for (position, record) in records.iter().enumerate() {
            index.enter(&record.subject, position);
        }
        for (position, record) in records.iter().enumerate() {
            let node = meter.node(capture, &record.subject)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlOutputDefaultError::InapplicableSubject(
                    record.subject.clone(),
                ));
            };
            // Finite admission for the same exact vocabulary recognition owner.
            if meter.known(element)? != Some(KnownHtmlElement::Output) {
                return Err(HtmlOutputDefaultError::InapplicableSubject(
                    record.subject.clone(),
                ));
            }
            if index.position(&record.subject) != Some(position) {
                return Err(HtmlOutputDefaultError::DuplicateSubject(
                    record.subject.clone(),
                ));
            }
            meter.charge(1)?;
            if let HtmlOutputDefaultValueOverride::Value(value) = &record.default_value_override {
                meter.acquire(
                    HtmlResource::FactCodeUnits,
                    value.len(),
                    |limit, required| HtmlOutputDefaultError::CodeUnits { limit, required },
                )?;
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host_revision,
            records,
            index,
            validation_work: meter.work()?,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn records(&self) -> &[HtmlOutputDefaultFact] {
        &self.records
    }
    /// Admitted construction work, useful when selecting a finite producer cap.
    ///
    /// The index of the records charges 1 for each, and each record charges 1 for
    /// its subject lookup, the vocabulary envelope
    /// `(local-name units + 1) * KnownHtmlElement::ALL.len() + namespace units`
    /// and 1 for the override check.
    pub fn validation_work(&self) -> usize {
        self.validation_work
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlControlTextOrigin {
    ChildText,
    DescendantText,
    OutputDefaultOverride,
}
/// Exact getter value with its selecting text/state source; no normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlControlTextReadback {
    value: DocumentString,
    origin: HtmlControlTextOrigin,
}
impl HtmlControlTextReadback {
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    pub fn origin(&self) -> HtmlControlTextOrigin {
        self.origin
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn control_text_readback(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        requested: HtmlControlTextOrigin,
    ) -> Result<HtmlSemanticValue<HtmlControlTextReadback>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let expected = if requested == HtmlControlTextOrigin::ChildText {
            KnownHtmlElement::Textarea
        } else {
            KnownHtmlElement::Output
        };
        if self.known(element)? != Some(expected) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let origin = if requested == HtmlControlTextOrigin::OutputDefaultOverride {
            if let Some(facts) = self.context.output_defaults() {
                self.charge(1)?;
                let selected = facts
                    .index
                    .position(subject)
                    .map(|position| facts.records[position].default_value_override());
                match selected {
                    Some(HtmlOutputDefaultValueOverride::Value(value)) => {
                        self.charge(1)?;
                        return Ok(HtmlSemanticValue::Known(HtmlControlTextReadback {
                            value: value.clone(),
                            origin: HtmlControlTextOrigin::OutputDefaultOverride,
                        }));
                    }
                    Some(HtmlOutputDefaultValueOverride::Null) => {
                        HtmlControlTextOrigin::DescendantText
                    }
                    _ => {
                        return Ok(HtmlSemanticValue::Missing(
                            HtmlSemanticMissing::OutputDefaultValueOverride(subject.clone()),
                        ));
                    }
                }
            } else {
                return Ok(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::OutputDefaultValueOverride(subject.clone()),
                ));
            }
        } else {
            requested
        };
        Ok(self
            .control_text_content(subject, origin)?
            .map(|value| HtmlControlTextReadback { value, origin }))
    }

    pub(super) fn control_text_content(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        origin: HtmlControlTextOrigin,
    ) -> Result<HtmlSemanticValue<DocumentString>, HtmlSemanticFailure> {
        Ok(match self.control_text_or_missing(subject, origin)? {
            Ok(text) => HtmlSemanticValue::Known(text),
            Err(missing) => HtmlSemanticValue::Missing(missing),
        })
    }
    /// The text content, or the missing evidence that stopped it: the query has no other
    /// outcome, which callers that need only these two can rely on.
    pub(super) fn control_text_or_missing(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        origin: HtmlControlTextOrigin,
    ) -> Result<Result<DocumentString, HtmlSemanticMissing>, HtmlSemanticFailure> {
        let mut frames = Vec::new();
        let result = self.control_text_content_inner(subject, origin, &mut frames);
        self.release_scratch(&frames);
        result
    }
    fn control_text_content_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        origin: HtmlControlTextOrigin,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<Result<DocumentString, HtmlSemanticMissing>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(mut children) = node.children() else {
            return Ok(Err(HtmlSemanticMissing::OrderedChildren(subject.clone())));
        };
        let mut units = Vec::new();
        if origin == HtmlControlTextOrigin::ChildText {
            // Direct child mode never enters an ignored element or reserves DFS frames.
            loop {
                self.charge(1)?;
                let Some(id) = children.next() else {
                    break;
                };
                let node = self.node(&id)?;
                if let HtmlCapturedNodeKind::Text(text) = node.kind() {
                    self.control_text_append(&mut units, text)?;
                }
            }
        } else {
            if children.len() != 0 {
                self.reserve_scratch(frames, 1)?;
                self.charge(1)?;
                frames.push(children);
            }
            while let Some(frame) = frames.last_mut() {
                self.charge(1)?;
                let Some(id) = frame.next() else {
                    self.charge(1)?;
                    frames.pop();
                    continue;
                };
                let node = self.node(&id)?;
                if let HtmlCapturedNodeKind::Text(text) = node.kind() {
                    self.control_text_append(&mut units, text)?;
                } else if matches!(
                    node.kind(),
                    HtmlCapturedNodeKind::Element(_)
                        | HtmlCapturedNodeKind::Document(_)
                        | HtmlCapturedNodeKind::Fragment(_)
                ) {
                    self.charge(1)?;
                    let HtmlCapturedChildren::Complete(children) = node.children() else {
                        return Ok(Err(HtmlSemanticMissing::OrderedChildren(id)));
                    };
                    if children.len() != 0 {
                        self.reserve_scratch(frames, 1)?;
                        self.charge(1)?;
                        frames.push(children);
                    }
                }
            }
        }
        self.charge(1)?;
        Ok(Ok(if units.is_empty() {
            self.context.empty_language().clone()
        } else {
            // Move the admitted variable buffer; only the fixed Arc wrapper is created.
            DocumentString::from_utf16(units)
        }))
    }
    fn control_text_append(
        &mut self,
        units: &mut Vec<u16>,
        text: &DocumentString,
    ) -> Result<(), HtmlSemanticFailure> {
        for unit in text.code_units() {
            self.charge(1)?;
            self.reserve_nested_output(units, 1)?;
            self.charge(1)?;
            units.push(*unit);
        }
        Ok(())
    }
}

//! Exact textarea/output text getters, HTML 877d6146487f §§4.10.11–12.
//! DOM child/descendant text content preserves ordinary Text data. Setters,
//! reset and establishing an output's actual default override are external.

use super::*;

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
    RecordCapacity { limit: usize, required: usize },
    CodeUnits { limit: usize, required: usize },
    ValidationWork { limit: usize, required: usize },
    RepresentationExhausted,
    DuplicateSubject(HtmlCaptureSubjectId),
    InapplicableSubject(HtmlCaptureSubjectId),
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
    validation_work: usize,
}
impl HtmlOutputDefaultFacts {
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        records: Vec<HtmlOutputDefaultFact>,
        limits: HtmlOutputDefaultLimits,
    ) -> Result<Self, HtmlOutputDefaultError> {
        if &host_revision != capture.host_facts_revision() {
            return Err(HtmlOutputDefaultError::HostRevisionMismatch);
        }
        if records.capacity() > limits.record_capacity {
            return Err(HtmlOutputDefaultError::RecordCapacity {
                limit: limits.record_capacity,
                required: records.capacity(),
            });
        }
        let mut work = 0usize;
        let mut units = 0usize;
        for (index, record) in records.iter().enumerate() {
            output_validation_charge(&mut work, limits.validation_work, 1)?;
            let node = capture
                .node(&record.subject)
                .map_err(HtmlOutputDefaultError::Capture)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlOutputDefaultError::InapplicableSubject(
                    record.subject.clone(),
                ));
            };
            // Finite admission for the same exact vocabulary recognition owner.
            let envelope = element
                .name()
                .local_name()
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
                .and_then(|n| {
                    n.checked_add(
                        element
                            .name()
                            .namespace()
                            .uri()
                            .map_or(0, DocumentString::len),
                    )
                })
                .ok_or(HtmlOutputDefaultError::RepresentationExhausted)?;
            output_validation_charge(&mut work, limits.validation_work, envelope)?;
            if KnownHtmlElement::from_name(element.name()) != Some(KnownHtmlElement::Output) {
                return Err(HtmlOutputDefaultError::InapplicableSubject(
                    record.subject.clone(),
                ));
            }
            for previous in &records[..index] {
                output_validation_charge(&mut work, limits.validation_work, 1)?;
                if previous.subject == record.subject {
                    return Err(HtmlOutputDefaultError::DuplicateSubject(
                        record.subject.clone(),
                    ));
                }
            }
            output_validation_charge(&mut work, limits.validation_work, 1)?;
            if let HtmlOutputDefaultValueOverride::Value(value) = &record.default_value_override {
                units = units
                    .checked_add(value.len())
                    .ok_or(HtmlOutputDefaultError::RepresentationExhausted)?;
                if units > limits.code_units {
                    return Err(HtmlOutputDefaultError::CodeUnits {
                        limit: limits.code_units,
                        required: units,
                    });
                }
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host_revision,
            records,
            validation_work: work,
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
    pub fn validation_work(&self) -> usize {
        self.validation_work
    }
}
fn output_validation_charge(
    work: &mut usize,
    limit: usize,
    amount: usize,
) -> Result<(), HtmlOutputDefaultError> {
    let required = work
        .checked_add(amount)
        .ok_or(HtmlOutputDefaultError::RepresentationExhausted)?;
    if required > limit {
        return Err(HtmlOutputDefaultError::ValidationWork { limit, required });
    }
    *work = required;
    Ok(())
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
                let mut selected = None;
                for record in facts.records() {
                    self.charge(1)?;
                    if record.subject() == subject {
                        selected = Some(record.default_value_override());
                        break;
                    }
                }
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
        let mut frames = Vec::new();
        let result = self.control_text_content_inner(subject, origin, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }
    fn control_text_content_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        origin: HtmlControlTextOrigin,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<DocumentString>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(mut children) = node.children() else {
            return Ok(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::OrderedChildren(subject.clone()),
            ));
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
            while !frames.is_empty() {
                self.charge(1)?;
                let Some(id) = frames.last_mut().expect("nonempty frame stack").next() else {
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
                        return Ok(HtmlSemanticValue::Missing(
                            HtmlSemanticMissing::OrderedChildren(id),
                        ));
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
        Ok(HtmlSemanticValue::Known(if units.is_empty() {
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

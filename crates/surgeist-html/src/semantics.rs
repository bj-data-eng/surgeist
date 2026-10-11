//! Capture-qualified hidden state and actual disabledness, HTML 877d6146487f.
//!
//! These queries implement `attr-hidden`, `concept-element-disabled`,
//! `concept-fe-disabled`, `concept-fieldset-disabled`, `concept-option-disabled`
//! and `option-element-nearest-ancestor-select`. They do not compute rendering,
//! focus, inertness, validation, event dispatch or custom-element execution.

use crate::{
    Attribute, HtmlCapture, HtmlCaptureError, HtmlCaptureId, HtmlCaptureSubjectId,
    HtmlCapturedChildren, HtmlCapturedElement, HtmlCapturedNode, HtmlCapturedNodeKind,
    HtmlCapturedParent, HtmlCustomControlAttributeContext, HtmlHiddenState, HtmlHostFactsRevision,
    HtmlValueAdmissionError, HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement,
};
use std::{
    error::Error,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// One producer-established custom-control classification at the context revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCustomControlClassification {
    subject: HtmlCaptureSubjectId,
    form_associated: bool,
}
impl HtmlCustomControlClassification {
    /// `false` is established nonparticipation, rather than missing evidence.
    pub fn new(subject: HtmlCaptureSubjectId, form_associated: bool) -> Self {
        Self {
            subject,
            form_associated,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn form_associated(&self) -> bool {
        self.form_associated
    }
}

/// Identity of immutable supplied classifications; equal records do not imply identity.
#[derive(Clone, Debug)]
pub struct HtmlSemanticContextId(Arc<()>);
impl PartialEq for HtmlSemanticContextId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlSemanticContextId {}

/// Checked custom-control facts, shared by many queries over one capture.
#[derive(Debug)]
pub struct HtmlSemanticContext {
    identity: HtmlSemanticContextId,
    capture: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    classifications: Vec<HtmlCustomControlClassification>,
}
/// Rejected producer context; no query result is created.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticContextError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    ClassificationLimit {
        limit: usize,
        retained_capacity: usize,
    },
    DuplicateSubject(HtmlCaptureSubjectId),
}
impl fmt::Display for HtmlSemanticContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlSemanticContextError {}
impl HtmlSemanticContext {
    /// Moves already supplied records without projecting or copying the tree.
    /// The explicit cap bounds retained vector capacity and the quadratic
    /// duplicate validation. Every record must identify an element of `capture`.
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        classifications: Vec<HtmlCustomControlClassification>,
        max_classifications: usize,
    ) -> Result<Self, HtmlSemanticContextError> {
        if &host_revision != capture.host_facts_revision() {
            return Err(HtmlSemanticContextError::HostRevisionMismatch);
        }
        if classifications.capacity() > max_classifications {
            return Err(HtmlSemanticContextError::ClassificationLimit {
                limit: max_classifications,
                retained_capacity: classifications.capacity(),
            });
        }
        for (i, fact) in classifications.iter().enumerate() {
            capture
                .element_id(&fact.subject)
                .map_err(HtmlSemanticContextError::Capture)?;
            if classifications[..i]
                .iter()
                .any(|other| other.subject == fact.subject)
            {
                return Err(HtmlSemanticContextError::DuplicateSubject(
                    fact.subject.clone(),
                ));
            }
        }
        Ok(Self {
            identity: HtmlSemanticContextId(Arc::new(())),
            capture: capture.identity().clone(),
            host_revision,
            classifications,
        })
    }
    pub fn identity(&self) -> &HtmlSemanticContextId {
        &self.identity
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn classifications(&self) -> &[HtmlCustomControlClassification] {
        &self.classifications
    }
}

/// Finite cumulative admission for one semantic session.
///
/// Work charges visits, result writes, inspected UTF-16 units and conservative
/// inspection envelopes for the existing intrinsic producers. Output counts
/// result records reserved across all batches, including failed batches. These
/// are semantic-query controls, independent of capture retention and parsing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSemanticLimits {
    work: usize,
    output_records: usize,
}
impl HtmlSemanticLimits {
    pub fn new(work: usize, output_records: usize) -> Self {
        Self {
            work,
            output_records,
        }
    }
    pub fn work(self) -> usize {
        self.work
    }
    pub fn output_records(self) -> usize {
        self.output_records
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlSemanticUsage {
    work: usize,
    output_records: usize,
}
impl HtmlSemanticUsage {
    pub fn work(self) -> usize {
        self.work
    }
    pub fn output_records(self) -> usize {
        self.output_records
    }
}
/// Caller-owned cancellation; already returned completed reports remain valid.
#[derive(Clone, Debug, Default)]
pub struct HtmlSemanticCancellation(Arc<AtomicBool>);
impl HtmlSemanticCancellation {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSemanticQuery {
    Hidden,
    Disabledness,
}
impl HtmlSemanticQuery {
    pub fn source_clause(self) -> &'static str {
        match self {
            Self::Hidden => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hidden"
            }
            Self::Disabledness => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-element-disabled"
            }
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSemanticRequest {
    subject: HtmlCaptureSubjectId,
    query: HtmlSemanticQuery,
}
impl HtmlSemanticRequest {
    pub fn new(subject: HtmlCaptureSubjectId, query: HtmlSemanticQuery) -> Self {
        Self { subject, query }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn query(&self) -> HtmlSemanticQuery {
        self.query
    }
}
/// Evidence needed by an implemented query but not established by its producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticMissing {
    Parent(HtmlCaptureSubjectId),
    OrderedChildren(HtmlCaptureSubjectId),
    CustomControlClassification(HtmlCaptureSubjectId),
}
/// This checkpoint has no foreign-language semantic implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSemanticUnsupported {
    ForeignNamespace,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticValue<T> {
    Known(T),
    Inapplicable,
    Missing(HtmlSemanticMissing),
    Unsupported(HtmlSemanticUnsupported),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticFact {
    Hidden(HtmlSemanticValue<HtmlHiddenState>),
    Disabledness(HtmlSemanticValue<bool>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSemanticResult {
    request: HtmlSemanticRequest,
    fact: HtmlSemanticFact,
}
impl HtmlSemanticResult {
    pub fn request(&self) -> &HtmlSemanticRequest {
        &self.request
    }
    pub fn fact(&self) -> &HtmlSemanticFact {
        &self.fact
    }
}
/// Invalidation contract: recompute when any qualification changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSemanticDependencies {
    ConservativeRecompute,
}
/// Immutable qualification of one admitted session's semantic inputs and controls.
/// This stores identities and finite configuration, never tree or query payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSemanticQualification {
    capture: HtmlCaptureId,
    context: HtmlSemanticContextId,
    host_revision: HtmlHostFactsRevision,
    limits: HtmlSemanticLimits,
    attribute_input_limit: usize,
}
impl HtmlSemanticQualification {
    pub fn source_revision(&self) -> &'static str {
        "877d6146487f15b15bc8dd0e3c625063a37b456a"
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn context(&self) -> &HtmlSemanticContextId {
        &self.context
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn limits(&self) -> HtmlSemanticLimits {
        self.limits
    }
    pub fn attribute_input_limit(&self) -> usize {
        self.attribute_input_limit
    }
}
/// Complete ordered results, qualified by every influencing input.
#[derive(Debug)]
pub struct HtmlSemanticReport {
    qualification: Arc<HtmlSemanticQualification>,
    usage: HtmlSemanticUsage,
    results: Vec<HtmlSemanticResult>,
}
impl HtmlSemanticReport {
    pub fn qualification(&self) -> &HtmlSemanticQualification {
        &self.qualification
    }
    pub fn source_revision(&self) -> &'static str {
        self.qualification.source_revision()
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        self.qualification.capture()
    }
    pub fn context(&self) -> &HtmlSemanticContextId {
        self.qualification.context()
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        self.qualification.host_revision()
    }
    pub fn limits(&self) -> HtmlSemanticLimits {
        self.qualification.limits()
    }
    pub fn attribute_input_limit(&self) -> usize {
        self.qualification.attribute_input_limit()
    }
    pub fn usage(&self) -> HtmlSemanticUsage {
        self.usage
    }
    pub fn results(&self) -> &[HtmlSemanticResult] {
        &self.results
    }
    pub fn dependencies(&self) -> HtmlSemanticDependencies {
        HtmlSemanticDependencies::ConservativeRecompute
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSemanticResource {
    Work,
    OutputRecords,
}
/// Terminal failure returns usage and no requested result prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticFailureKind {
    Cancelled,
    LimitExceeded {
        resource: HtmlSemanticResource,
        limit: usize,
        required: usize,
    },
    RepresentationExhausted,
    AllocationFailed,
    Capture(HtmlCaptureError),
    ContextCaptureMismatch,
    Attribute(HtmlValueAdmissionError),
    IntrinsicUnavailable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSemanticFailure {
    kind: HtmlSemanticFailureKind,
    usage: HtmlSemanticUsage,
    qualification: Option<Arc<HtmlSemanticQualification>>,
}
impl HtmlSemanticFailure {
    pub fn kind(&self) -> &HtmlSemanticFailureKind {
        &self.kind
    }
    pub fn usage(&self) -> HtmlSemanticUsage {
        self.usage
    }
    /// Every terminal query failure retains its original admitted session key.
    /// `None` denotes a constructor rejection before any valid session existed.
    pub fn qualification(&self) -> Option<&HtmlSemanticQualification> {
        self.qualification.as_deref()
    }
}
impl fmt::Display for HtmlSemanticFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
impl Error for HtmlSemanticFailure {}

/// Reusable borrowed query boundary. The first terminal failure is sticky.
pub struct HtmlSemanticSession<'a> {
    capture: &'a HtmlCapture,
    context: &'a HtmlSemanticContext,
    interpreter: HtmlValueInterpreter,
    limits: HtmlSemanticLimits,
    cancellation: HtmlSemanticCancellation,
    qualification: Arc<HtmlSemanticQualification>,
    usage: HtmlSemanticUsage,
    failure: Option<HtmlSemanticFailure>,
}
impl<'a> HtmlSemanticSession<'a> {
    pub fn new(
        capture: &'a HtmlCapture,
        context: &'a HtmlSemanticContext,
        interpreter: HtmlValueInterpreter,
        limits: HtmlSemanticLimits,
        cancellation: HtmlSemanticCancellation,
    ) -> Result<Self, HtmlSemanticFailure> {
        if capture.identity() != context.capture() {
            return Err(HtmlSemanticFailure {
                kind: HtmlSemanticFailureKind::ContextCaptureMismatch,
                usage: HtmlSemanticUsage::default(),
                qualification: None,
            });
        }
        Ok(Self {
            capture,
            context,
            interpreter,
            limits,
            cancellation,
            // Prepare this fixed-size key before execution. Terminal stops and
            // repeated failures only clone its Arc; they do not allocate.
            qualification: Arc::new(HtmlSemanticQualification {
                capture: capture.identity().clone(),
                context: context.identity.clone(),
                host_revision: context.host_revision.clone(),
                limits,
                attribute_input_limit: interpreter.max_input_code_units(),
            }),
            usage: HtmlSemanticUsage::default(),
            failure: None,
        })
    }
    pub fn usage(&self) -> HtmlSemanticUsage {
        self.usage
    }
    pub fn failure(&self) -> Option<&HtmlSemanticFailure> {
        self.failure.as_ref()
    }
    fn stop(&mut self, kind: HtmlSemanticFailureKind) -> HtmlSemanticFailure {
        self.failure
            .get_or_insert(HtmlSemanticFailure {
                kind,
                usage: self.usage,
                qualification: Some(self.qualification.clone()),
            })
            .clone()
    }
    fn poll(&mut self) -> Result<(), HtmlSemanticFailure> {
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        if self.cancellation.is_cancelled() {
            return Err(self.stop(HtmlSemanticFailureKind::Cancelled));
        }
        Ok(())
    }
    fn charge(&mut self, work: usize) -> Result<(), HtmlSemanticFailure> {
        self.poll()?;
        let Some(required) = self.usage.work.checked_add(work) else {
            return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
        };
        if required > self.limits.work {
            return Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: self.limits.work,
                required,
            }));
        }
        self.usage.work = required;
        Ok(())
    }
    fn node(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCapturedNode<'a>, HtmlSemanticFailure> {
        self.charge(1)?;
        self.capture
            .node(subject)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Capture(e)))
    }
    fn known(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<Option<KnownHtmlElement>, HtmlSemanticFailure> {
        // Envelope covers every candidate spelling and all supplied-name units
        // before the existing vocabulary owner performs its bounded recognition.
        let units = element.name().local_name().len();
        let ns_units = element.name().namespace().uri().map_or(0, |s| s.len());
        let envelope = units
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| n.checked_add(ns_units));
        let Some(envelope) = envelope else {
            return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
        };
        self.charge(envelope)?;
        Ok(KnownHtmlElement::from_name(element.name()))
    }
    fn attribute(
        &mut self,
        element: &'a HtmlCapturedElement,
        local: &str,
    ) -> Result<Option<&'a Attribute>, HtmlSemanticFailure> {
        for attribute in element.attributes().iter() {
            self.charge(1)?;
            if attribute.name().namespace().uri().is_some() {
                continue;
            }
            let units = attribute.name().local_name().code_units();
            if units.len() != local.len() {
                continue;
            }
            let mut equal = true;
            for (unit, byte) in units.iter().zip(local.bytes()) {
                self.charge(1)?;
                if *unit != u16::from(byte) {
                    equal = false;
                    break;
                }
            }
            if equal {
                return Ok(Some(attribute));
            }
        }
        Ok(None)
    }
    fn html_namespace(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        let Some(uri) = element.name().namespace().uri() else {
            return Ok(false);
        };
        let expected = "http://www.w3.org/1999/xhtml";
        if uri.len() != expected.len() {
            return Ok(false);
        }
        for (unit, byte) in uri.code_units().iter().zip(expected.bytes()) {
            self.charge(1)?;
            if *unit != u16::from(byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn parent(
        &mut self,
        node: HtmlCapturedNode<'a>,
    ) -> Result<Result<Option<HtmlCaptureSubjectId>, HtmlSemanticMissing>, HtmlSemanticFailure>
    {
        self.charge(1)?;
        Ok(match node.parent() {
            HtmlCapturedParent::Root => Ok(None),
            HtmlCapturedParent::Subject(id) => Ok(Some(id)),
            HtmlCapturedParent::Unavailable => Err(HtmlSemanticMissing::Parent(node.id())),
        })
    }
    fn disabled_attribute(
        &mut self,
        element: &'a HtmlCapturedElement,
        custom_control: bool,
    ) -> Result<bool, HtmlSemanticFailure> {
        let raw = self.attribute(element, "disabled")?;
        self.admit_intrinsic(element, raw, 1)?;
        // The intrinsic owner supplies presence, conformance and applicability.
        let report = if custom_control {
            self.interpreter.disabled_for_custom_control(
                element.name(),
                raw,
                HtmlCustomControlAttributeContext::FormAssociated,
            )
        } else {
            self.interpreter.disabled(element.name(), raw)
        }
        .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        report
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }
    fn admit_intrinsic(
        &mut self,
        element: &HtmlCapturedElement,
        raw: Option<&Attribute>,
        keyword_count: usize,
    ) -> Result<(), HtmlSemanticFailure> {
        // Existing intrinsic rules own recognition and defaults. This envelope
        // admits their finite vocabulary searches and keyword inspections as
        // one atomic bounded operation; it is not parser byte accounting.
        let element_units = element.name().local_name().len();
        let namespace_units = element.name().namespace().uri().map_or(0, |v| v.len());
        let attribute_units = raw.map_or(0, |a| a.name().local_name().len());
        let value_units = raw.map_or(0, |a| a.value().len());
        let envelope = element_units
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| n.checked_mul(3))
            .and_then(|n| {
                attribute_units
                    .checked_add(1)
                    .and_then(|a| a.checked_mul(KnownHtmlAttribute::ALL.len()))
                    .and_then(|a| n.checked_add(a))
            })
            .and_then(|n| {
                value_units
                    .checked_mul(keyword_count)
                    .and_then(|v| n.checked_add(v))
            })
            .and_then(|n| {
                namespace_units
                    .checked_add(1)
                    .and_then(|s| s.checked_mul(3))
                    .and_then(|s| n.checked_add(s))
            });
        let Some(envelope) = envelope else {
            return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
        };
        self.charge(envelope)
    }
    fn form_disabled(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        custom_control: bool,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if self.disabled_attribute(element, custom_control)? {
            return Ok(HtmlSemanticValue::Known(true));
        }
        let mut current = node;
        let mut branch = node.id();
        let mut missing_children = None;
        loop {
            let id = match self.parent(current)? {
                Ok(Some(id)) => id,
                Ok(None) => {
                    return Ok(missing_children
                        .map_or(HtmlSemanticValue::Known(false), HtmlSemanticValue::Missing));
                }
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            let ancestor = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(e) = ancestor.kind()
                && self.known(e)? == Some(KnownHtmlElement::Fieldset)
                && self.disabled_attribute(e, false)?
            {
                // A descendant can be inside the first legend only through a
                // direct HTML legend child. Other branches need no child-order
                // evidence to establish this fieldset's disabling effect.
                let branch_node = self.node(&branch)?;
                let legend_branch = match branch_node.kind() {
                    HtmlCapturedNodeKind::Element(e) => {
                        self.known(e)? == Some(KnownHtmlElement::Legend)
                    }
                    _ => false,
                };
                if !legend_branch {
                    return Ok(HtmlSemanticValue::Known(true));
                }
                self.charge(1)?;
                match ancestor.children() {
                    HtmlCapturedChildren::Unavailable => {
                        missing_children = Some(HtmlSemanticMissing::OrderedChildren(id.clone()))
                    }
                    HtmlCapturedChildren::Complete(children) => {
                        let mut first_legend = None;
                        let mut children = children;
                        loop {
                            self.charge(1)?;
                            let Some(child) = children.next() else {
                                break;
                            };
                            let child_node = self.node(&child)?;
                            if let HtmlCapturedNodeKind::Element(e) = child_node.kind()
                                && self.known(e)? == Some(KnownHtmlElement::Legend)
                            {
                                first_legend = Some(child);
                                break;
                            }
                        }
                        if first_legend.as_ref() != Some(&branch) {
                            return Ok(HtmlSemanticValue::Known(true));
                        }
                    }
                }
            }
            branch = id;
            current = ancestor;
        }
    }
    fn nearest_select(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Result<Option<HtmlCaptureSubjectId>, HtmlSemanticMissing>, HtmlSemanticFailure>
    {
        let mut current = self.node(subject)?;
        let mut optgroup = false;
        loop {
            let id = match self.parent(current)? {
                Ok(Some(id)) => id,
                other => return Ok(other),
            };
            current = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(e) = current.kind() {
                match self.known(e)? {
                    Some(
                        KnownHtmlElement::Datalist
                        | KnownHtmlElement::Hr
                        | KnownHtmlElement::Option,
                    ) => return Ok(Ok(None)),
                    Some(KnownHtmlElement::Optgroup) if optgroup => return Ok(Ok(None)),
                    Some(KnownHtmlElement::Optgroup) => optgroup = true,
                    Some(KnownHtmlElement::Select) => return Ok(Ok(Some(id))),
                    _ => {}
                }
            }
        }
    }
    fn option_disabled(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let mut current = self.node(subject)?;
        if let HtmlCapturedNodeKind::Element(e) = current.kind()
            && self.disabled_attribute(e, false)?
        {
            return Ok(HtmlSemanticValue::Known(true));
        }
        loop {
            let id = match self.parent(current)? {
                Ok(Some(id)) => id,
                Ok(None) => return Ok(HtmlSemanticValue::Known(false)),
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            current = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(e) = current.kind() {
                match self.known(e)? {
                    Some(
                        KnownHtmlElement::Select
                        | KnownHtmlElement::Hr
                        | KnownHtmlElement::Datalist
                        | KnownHtmlElement::Option,
                    ) => return Ok(HtmlSemanticValue::Known(false)),
                    Some(KnownHtmlElement::Optgroup) => {
                        return Ok(HtmlSemanticValue::Known(self.disabled_attribute(e, false)?));
                    }
                    _ => {}
                }
            }
        }
    }
    fn disabledness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        match self.known(element)? {
            Some(
                KnownHtmlElement::Button
                | KnownHtmlElement::Input
                | KnownHtmlElement::Select
                | KnownHtmlElement::Textarea
                | KnownHtmlElement::Fieldset,
            ) => self.form_disabled(subject, false),
            Some(kind @ (KnownHtmlElement::Option | KnownHtmlElement::Optgroup)) => {
                let local = if kind == KnownHtmlElement::Option {
                    self.option_disabled(subject)?
                } else {
                    HtmlSemanticValue::Known(self.disabled_attribute(element, false)?)
                };
                if local == HtmlSemanticValue::Known(true) {
                    return Ok(local);
                }
                let select = match self.nearest_select(subject)? {
                    Ok(Some(id)) => self.form_disabled(&id, false)?,
                    Ok(None) => HtmlSemanticValue::Known(false),
                    Err(missing) => HtmlSemanticValue::Missing(missing),
                };
                Ok(if select == HtmlSemanticValue::Known(true) {
                    select
                } else if matches!(local, HtmlSemanticValue::Missing(_)) {
                    local
                } else {
                    select
                })
            }
            Some(_) => Ok(HtmlSemanticValue::Inapplicable),
            None => {
                let context = self.context;
                for fact in &context.classifications {
                    self.charge(1)?;
                    if &fact.subject == subject {
                        return if fact.form_associated {
                            self.form_disabled(subject, true)
                        } else {
                            Ok(HtmlSemanticValue::Inapplicable)
                        };
                    }
                }
                Ok(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::CustomControlClassification(subject.clone()),
                ))
            }
        }
    }
    /// Requests are retained in input order, including duplicates. Any terminal
    /// failure discards this whole batch; unavailable semantic evidence is a
    /// complete typed fact, not an interrupted query.
    pub fn query(
        &mut self,
        requests: &[HtmlSemanticRequest],
    ) -> Result<HtmlSemanticReport, HtmlSemanticFailure> {
        self.poll()?;
        let Some(required) = self.usage.output_records.checked_add(requests.len()) else {
            return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
        };
        if required > self.limits.output_records {
            return Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::OutputRecords,
                limit: self.limits.output_records,
                required,
            }));
        }
        self.usage.output_records = required;
        let mut results = Vec::new();
        results
            .try_reserve_exact(requests.len())
            .map_err(|_| self.stop(HtmlSemanticFailureKind::AllocationFailed))?;
        if results.capacity() > requests.len() {
            let Some(actual) = self
                .usage
                .output_records
                .checked_add(results.capacity() - requests.len())
            else {
                return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
            };
            if actual > self.limits.output_records {
                return Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                    resource: HtmlSemanticResource::OutputRecords,
                    limit: self.limits.output_records,
                    required: actual,
                }));
            }
            self.usage.output_records = actual;
        }
        for request in requests {
            let node = self.node(&request.subject)?;
            let fact = match node.kind() {
                HtmlCapturedNodeKind::Element(element) => {
                    let foreign = !self.html_namespace(element)?;
                    match request.query {
                        HtmlSemanticQuery::Hidden => {
                            let value = if foreign {
                                HtmlSemanticValue::Unsupported(
                                    HtmlSemanticUnsupported::ForeignNamespace,
                                )
                            } else {
                                let raw = self.attribute(element, "hidden")?;
                                self.admit_intrinsic(element, raw, 2)?;
                                let state =
                                    self.interpreter.hidden(element.name(), raw).map_err(|e| {
                                        self.stop(HtmlSemanticFailureKind::Attribute(e))
                                    })?;
                                HtmlSemanticValue::Known(state.intrinsic().copied().ok_or_else(
                                    || self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable),
                                )?)
                            };
                            HtmlSemanticFact::Hidden(value)
                        }
                        HtmlSemanticQuery::Disabledness => {
                            HtmlSemanticFact::Disabledness(if foreign {
                                HtmlSemanticValue::Unsupported(
                                    HtmlSemanticUnsupported::ForeignNamespace,
                                )
                            } else {
                                self.disabledness(&request.subject, element)?
                            })
                        }
                    }
                }
                _ => match request.query {
                    HtmlSemanticQuery::Hidden => {
                        HtmlSemanticFact::Hidden(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::Disabledness => {
                        HtmlSemanticFact::Disabledness(HtmlSemanticValue::Inapplicable)
                    }
                },
            };
            self.charge(1)?;
            results.push(HtmlSemanticResult {
                request: request.clone(),
                fact,
            });
        }
        self.poll()?;
        Ok(HtmlSemanticReport {
            qualification: self.qualification.clone(),
            usage: self.usage,
            results,
        })
    }
}

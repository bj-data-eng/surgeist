//! HTML 877d6146487f §3.2.6.2 and §3.2.6.4: exact language and directionality.
//! Language tags are BCP 47-designated payloads, without validity or service claims.

use super::*;
use crate::bidi::{BidiClass, bidi_class};
use crate::{
    DocumentString, HtmlCapturedFragmentKind, HtmlCapturedHost, HtmlDir, HtmlDirState,
    HtmlInputType,
};

/// Established processing history, rather than a scan of current meta elements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlPragmaLanguage {
    Unavailable,
    Absent,
    Tag(DocumentString),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlProtocolLanguage {
    Unavailable,
    Absent,
    Single(DocumentString),
    Multiple,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDocumentLanguageFallback {
    document: HtmlCaptureSubjectId,
    pragma: HtmlPragmaLanguage,
    protocol: HtmlProtocolLanguage,
}
impl HtmlDocumentLanguageFallback {
    pub fn new(
        document: HtmlCaptureSubjectId,
        pragma: HtmlPragmaLanguage,
        protocol: HtmlProtocolLanguage,
    ) -> Self {
        Self {
            document,
            pragma,
            protocol,
        }
    }
    pub fn document(&self) -> &HtmlCaptureSubjectId {
        &self.document
    }
    pub fn pragma(&self) -> &HtmlPragmaLanguage {
        &self.pragma
    }
    pub fn protocol(&self) -> &HtmlProtocolLanguage {
        &self.protocol
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlControlCurrentValue {
    subject: HtmlCaptureSubjectId,
    value: Option<DocumentString>,
}
impl HtmlControlCurrentValue {
    pub fn new(subject: HtmlCaptureSubjectId, value: Option<DocumentString>) -> Self {
        Self { subject, value }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn value(&self) -> Option<&DocumentString> {
        self.value.as_ref()
    }
}
/// `None` is unavailable assignment; `Some(vec![])` is established empty.
#[derive(Debug)]
pub struct HtmlSlotAssignedNodes {
    slot: HtmlCaptureSubjectId,
    nodes: Option<Vec<HtmlCaptureSubjectId>>,
}
impl HtmlSlotAssignedNodes {
    pub fn new(slot: HtmlCaptureSubjectId, nodes: Option<Vec<HtmlCaptureSubjectId>>) -> Self {
        Self { slot, nodes }
    }
    pub fn slot(&self) -> &HtmlCaptureSubjectId {
        &self.slot
    }
    pub fn nodes(&self) -> Option<&[HtmlCaptureSubjectId]> {
        self.nodes.as_deref()
    }
}

/// Capacity limits apply to moved vectors; code units bound retained logical
/// string payloads, without promising control of preexisting Arc allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlLanguageDirectionLimits {
    records: usize,
    assigned_nodes: usize,
    code_units: usize,
    validation_work: usize,
}
impl HtmlLanguageDirectionLimits {
    pub fn new(
        record_capacity: usize,
        assigned_node_capacity: usize,
        code_units: usize,
        validation_work: usize,
    ) -> Self {
        Self {
            records: record_capacity,
            assigned_nodes: assigned_node_capacity,
            code_units,
            validation_work,
        }
    }
    pub fn record_capacity(self) -> usize {
        self.records
    }
    pub fn assigned_node_capacity(self) -> usize {
        self.assigned_nodes
    }
    pub fn code_units(self) -> usize {
        self.code_units
    }
    pub fn validation_work(self) -> usize {
        self.validation_work
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlLanguageDirectionError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    RecordCapacity { limit: usize, required: usize },
    AssignedNodeCapacity { limit: usize, required: usize },
    CodeUnits { limit: usize, required: usize },
    ValidationWork { limit: usize, required: usize },
    RepresentationExhausted,
    DuplicateSubject(HtmlCaptureSubjectId),
    EmptyPragma(HtmlCaptureSubjectId),
    InapplicableCurrentValue(HtmlCaptureSubjectId),
    InvalidSlot(HtmlCaptureSubjectId),
    UnavailableRelation(HtmlCaptureSubjectId),
    InvalidAssignedNode(HtmlCaptureSubjectId),
    DuplicateAssignment(HtmlCaptureSubjectId),
    AssignmentOrder(HtmlCaptureSubjectId),
}
impl fmt::Display for HtmlLanguageDirectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlLanguageDirectionError {}

/// Concrete checked immutable host supplement; no callbacks or second tree.
#[derive(Debug)]
pub struct HtmlLanguageDirectionFacts {
    capture: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    fallbacks: Vec<HtmlDocumentLanguageFallback>,
    current_values: Vec<HtmlControlCurrentValue>,
    assignments: Vec<HtmlSlotAssignedNodes>,
}

struct LanguageValidation {
    work: usize,
    limit: usize,
}
impl LanguageValidation {
    fn charge(&mut self, amount: usize) -> Result<(), HtmlLanguageDirectionError> {
        let required = self
            .work
            .checked_add(amount)
            .ok_or(HtmlLanguageDirectionError::RepresentationExhausted)?;
        if required > self.limit {
            return Err(HtmlLanguageDirectionError::ValidationWork {
                limit: self.limit,
                required,
            });
        }
        self.work = required;
        Ok(())
    }
    fn node<'a>(
        &mut self,
        capture: &'a HtmlCapture,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCapturedNode<'a>, HtmlLanguageDirectionError> {
        self.charge(1)?;
        capture
            .node(id)
            .map_err(HtmlLanguageDirectionError::Capture)
    }
    fn known(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<Option<KnownHtmlElement>, HtmlLanguageDirectionError> {
        let amount = element
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
            .ok_or(HtmlLanguageDirectionError::RepresentationExhausted)?;
        self.charge(amount)?;
        Ok(KnownHtmlElement::from_name(element.name()))
    }
    fn slot_host(
        &mut self,
        capture: &HtmlCapture,
        slot: &HtmlCaptureSubjectId,
    ) -> Result<(HtmlCaptureSubjectId, bool), HtmlLanguageDirectionError> {
        let mut current = slot.clone();
        loop {
            let node = self.node(capture, &current)?;
            if let HtmlCapturedNodeKind::Fragment(fragment) = node.kind()
                && let HtmlCapturedFragmentKind::ShadowRoot(properties) = fragment.kind()
            {
                self.charge(1)?;
                let named =
                    properties.properties().slot_assignment() == crate::HtmlSlotAssignment::Named;
                return match node.host() {
                    Some(HtmlCapturedHost::Element(host)) => Ok((host.subject().clone(), named)),
                    _ => Err(HtmlLanguageDirectionError::UnavailableRelation(current)),
                };
            }
            self.charge(1)?;
            match node.parent() {
                HtmlCapturedParent::Subject(parent) => current = parent,
                HtmlCapturedParent::Unavailable => {
                    return Err(HtmlLanguageDirectionError::UnavailableRelation(current));
                }
                HtmlCapturedParent::Root => {
                    return Err(HtmlLanguageDirectionError::InvalidSlot(slot.clone()));
                }
            }
        }
    }
}

fn language_sum(left: usize, right: usize) -> Result<usize, HtmlLanguageDirectionError> {
    left.checked_add(right)
        .ok_or(HtmlLanguageDirectionError::RepresentationExhausted)
}
impl HtmlLanguageDirectionFacts {
    /// Moves bounded facts. Complete assignments must be unique slottables of
    /// the established shadow host. Named roots require ordinary child order;
    /// manual roots retain the supplied assignment order. Source history
    /// and actual control values are host-established, not reconstructed here.
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        fallbacks: Vec<HtmlDocumentLanguageFallback>,
        current_values: Vec<HtmlControlCurrentValue>,
        assignments: Vec<HtmlSlotAssignedNodes>,
        limits: HtmlLanguageDirectionLimits,
    ) -> Result<Self, HtmlLanguageDirectionError> {
        if capture.host_facts_revision() != &host_revision {
            return Err(HtmlLanguageDirectionError::HostRevisionMismatch);
        }
        let records = language_sum(
            language_sum(fallbacks.capacity(), current_values.capacity())?,
            assignments.capacity(),
        )?;
        if records > limits.records {
            return Err(HtmlLanguageDirectionError::RecordCapacity {
                limit: limits.records,
                required: records,
            });
        }
        let mut validation = LanguageValidation {
            work: 0,
            limit: limits.validation_work,
        };
        let mut units = 0;
        let mut assigned_capacity = 0;
        for (index, fact) in fallbacks.iter().enumerate() {
            validation.charge(1)?;
            let node = validation.node(capture, &fact.document)?;
            if !matches!(node.kind(), HtmlCapturedNodeKind::Document(_)) {
                validation.charge(1)?;
                capture
                    .document_id(&fact.document)
                    .map_err(HtmlLanguageDirectionError::Capture)?;
            }
            for previous in &fallbacks[..index] {
                validation.charge(1)?;
                if previous.document == fact.document {
                    return Err(HtmlLanguageDirectionError::DuplicateSubject(
                        fact.document.clone(),
                    ));
                }
            }
            if let HtmlPragmaLanguage::Tag(tag) = &fact.pragma {
                validation.charge(1)?;
                if tag.is_empty() {
                    return Err(HtmlLanguageDirectionError::EmptyPragma(
                        fact.document.clone(),
                    ));
                }
                units = language_sum(units, tag.len())?;
            }
            if let HtmlProtocolLanguage::Single(tag) = &fact.protocol {
                validation.charge(1)?;
                units = language_sum(units, tag.len())?;
            }
        }
        for (index, fact) in current_values.iter().enumerate() {
            validation.charge(1)?;
            let node = validation.node(capture, &fact.subject)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlLanguageDirectionError::InapplicableCurrentValue(
                    fact.subject.clone(),
                ));
            };
            let known = validation.known(element)?;
            let applicable = if known == Some(KnownHtmlElement::Textarea) {
                true
            } else if known == Some(KnownHtmlElement::Input) {
                let mut raw = None;
                for attribute in element.attributes().iter() {
                    validation.charge(1)?;
                    if attribute.name().namespace().uri().is_none() {
                        validation.charge(4)?;
                        if attribute.name().local_name().code_units() == [116, 121, 112, 101] {
                            raw = Some(attribute);
                            break;
                        }
                    }
                }
                let amount = element
                    .name()
                    .local_name()
                    .len()
                    .checked_add(1)
                    .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len() * 3))
                    .and_then(|n| {
                        raw.map_or(Some(n), |a| {
                            a.name()
                                .local_name()
                                .len()
                                .checked_add(1)
                                .and_then(|x| x.checked_mul(KnownHtmlAttribute::ALL.len()))
                                .and_then(|x| n.checked_add(x))
                                .and_then(|x| {
                                    a.value()
                                        .len()
                                        .checked_mul(23)
                                        .and_then(|v| x.checked_add(v))
                                })
                        })
                    })
                    .and_then(|n| {
                        element.name().namespace().uri().map_or(Some(n), |s| {
                            s.len()
                                .checked_add(1)
                                .and_then(|x| x.checked_mul(3))
                                .and_then(|x| n.checked_add(x))
                        })
                    })
                    .ok_or(HtmlLanguageDirectionError::RepresentationExhausted)?;
                validation.charge(amount)?;
                let state = HtmlValueInterpreter::new(limits.code_units)
                    .input_type(element.name(), raw)
                    .map_err(|_| {
                        HtmlLanguageDirectionError::InapplicableCurrentValue(fact.subject.clone())
                    })?;
                matches!(
                    state.intrinsic(),
                    Some(
                        HtmlInputType::Hidden
                            | HtmlInputType::Text
                            | HtmlInputType::Search
                            | HtmlInputType::Telephone
                            | HtmlInputType::Url
                            | HtmlInputType::Email
                            | HtmlInputType::Password
                            | HtmlInputType::Submit
                            | HtmlInputType::Reset
                            | HtmlInputType::Button
                    )
                )
            } else {
                false
            };
            if !applicable {
                return Err(HtmlLanguageDirectionError::InapplicableCurrentValue(
                    fact.subject.clone(),
                ));
            }
            for previous in &current_values[..index] {
                validation.charge(1)?;
                if previous.subject == fact.subject {
                    return Err(HtmlLanguageDirectionError::DuplicateSubject(
                        fact.subject.clone(),
                    ));
                }
            }
            if let Some(value) = &fact.value {
                validation.charge(1)?;
                units = language_sum(units, value.len())?;
            }
        }
        for (index, fact) in assignments.iter().enumerate() {
            validation.charge(1)?;
            let node = validation.node(capture, &fact.slot)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlLanguageDirectionError::InvalidSlot(fact.slot.clone()));
            };
            if validation.known(element)? != Some(KnownHtmlElement::Slot) {
                return Err(HtmlLanguageDirectionError::InvalidSlot(fact.slot.clone()));
            }
            for previous in &assignments[..index] {
                validation.charge(1)?;
                if previous.slot == fact.slot {
                    return Err(HtmlLanguageDirectionError::DuplicateSubject(
                        fact.slot.clone(),
                    ));
                }
            }
            let Some(nodes) = &fact.nodes else {
                continue;
            };
            assigned_capacity = language_sum(assigned_capacity, nodes.capacity())?;
            if assigned_capacity > limits.assigned_nodes {
                return Err(HtmlLanguageDirectionError::AssignedNodeCapacity {
                    limit: limits.assigned_nodes,
                    required: assigned_capacity,
                });
            }
            let (host, named) = validation.slot_host(capture, &fact.slot)?;
            let mut host_children = if named && !nodes.is_empty() {
                let host_node = validation.node(capture, &host)?;
                validation.charge(1)?;
                let HtmlCapturedChildren::Complete(children) = host_node.children() else {
                    return Err(HtmlLanguageDirectionError::UnavailableRelation(host));
                };
                Some(children)
            } else {
                None
            };
            for (node_index, assigned) in nodes.iter().enumerate() {
                validation.charge(1)?;
                let node = validation.node(capture, assigned)?;
                if !matches!(
                    node.kind(),
                    HtmlCapturedNodeKind::Text(_) | HtmlCapturedNodeKind::Element(_)
                ) {
                    return Err(HtmlLanguageDirectionError::InvalidAssignedNode(
                        assigned.clone(),
                    ));
                }
                validation.charge(1)?;
                if node.parent() != HtmlCapturedParent::Subject(host.clone()) {
                    return Err(HtmlLanguageDirectionError::InvalidAssignedNode(
                        assigned.clone(),
                    ));
                }
                for previous in &nodes[..node_index] {
                    validation.charge(1)?;
                    if previous == assigned {
                        return Err(HtmlLanguageDirectionError::DuplicateAssignment(
                            assigned.clone(),
                        ));
                    }
                }
                for previous in &assignments[..index] {
                    validation.charge(1)?;
                    if let Some(previous_nodes) = &previous.nodes {
                        for previous in previous_nodes {
                            validation.charge(1)?;
                            if previous == assigned {
                                return Err(HtmlLanguageDirectionError::DuplicateAssignment(
                                    assigned.clone(),
                                ));
                            }
                        }
                    }
                }
                if let Some(children) = &mut host_children {
                    loop {
                        validation.charge(1)?;
                        match children.next() {
                            Some(child) if &child == assigned => break,
                            Some(_) => {}
                            None => {
                                return Err(HtmlLanguageDirectionError::AssignmentOrder(
                                    fact.slot.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        if units > limits.code_units {
            return Err(HtmlLanguageDirectionError::CodeUnits {
                limit: limits.code_units,
                required: units,
            });
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host_revision,
            fallbacks,
            current_values,
            assignments,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn fallbacks(&self) -> &[HtmlDocumentLanguageFallback] {
        &self.fallbacks
    }
    pub fn current_values(&self) -> &[HtmlControlCurrentValue] {
        &self.current_values
    }
    pub fn assignments(&self) -> &[HtmlSlotAssignedNodes] {
        &self.assignments
    }
}

/// The original selecting source is retained through inheritance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlLanguageOrigin {
    XmlAttribute(HtmlCaptureSubjectId),
    LangAttribute(HtmlCaptureSubjectId),
    Pragma(HtmlCaptureSubjectId),
    Protocol(HtmlCaptureSubjectId),
    NoInformation(HtmlCaptureSubjectId),
}

/// Exact effective tag; an empty tag means explicitly unknown.
/// Nonempty tags designate BCP 47, including invalid or unsupported spellings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlLanguage {
    tag: DocumentString,
    origin: HtmlLanguageOrigin,
}
impl HtmlLanguage {
    pub fn tag(&self) -> &DocumentString {
        &self.tag
    }
    pub fn origin(&self) -> &HtmlLanguageOrigin {
        &self.origin
    }
    pub fn is_explicitly_unknown(&self) -> bool {
        self.tag.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDirectionality {
    Ltr,
    Rtl,
}

enum LanguageStop {
    Failure(HtmlSemanticFailure),
    Missing(HtmlSemanticMissing),
}
impl From<HtmlSemanticFailure> for LanguageStop {
    fn from(value: HtmlSemanticFailure) -> Self {
        Self::Failure(value)
    }
}
type LanguageResult<T> = Result<T, LanguageStop>;

// Continuations own only IDs and borrowed iterators, never a projected tree.
enum DirectionTask<'a> {
    Direction(HtmlCaptureSubjectId),
    Auto(HtmlCaptureSubjectId),
    Contained(HtmlCaptureSubjectId, bool),
    Scan(HtmlCapturedChildIds<'a>),
    Assigned(std::slice::Iter<'a, HtmlCaptureSubjectId>),
    DefaultLtr,
    AssignedNext(std::slice::Iter<'a, HtmlCaptureSubjectId>),
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn language(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlLanguage>, HtmlSemanticFailure> {
        match self.language_inner(subject) {
            Ok(value) => Ok(HtmlSemanticValue::Known(value)),
            Err(LanguageStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(LanguageStop::Failure(failure)) => Err(failure),
        }
    }

    fn language_inner(&mut self, subject: &HtmlCaptureSubjectId) -> LanguageResult<HtmlLanguage> {
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                if let Some(attribute) = self.attribute_ns(
                    element,
                    Some("http://www.w3.org/XML/1998/namespace"),
                    "lang",
                )? {
                    self.interpreter
                        .admit_attribute_raw(Some(attribute.value()))
                        .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                    self.charge(1)?;
                    return Ok(HtmlLanguage {
                        tag: attribute.value().clone(),
                        origin: HtmlLanguageOrigin::XmlAttribute(current),
                    });
                }
                if (self.html_namespace(element)?
                    || self.language_namespace(element, "http://www.w3.org/2000/svg")?)
                    && let Some(attribute) = self.attribute(element, "lang")?
                {
                    self.interpreter
                        .admit_attribute_raw(Some(attribute.value()))
                        .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                    self.charge(1)?;
                    return Ok(HtmlLanguage {
                        tag: attribute.value().clone(),
                        origin: HtmlLanguageOrigin::LangAttribute(current),
                    });
                }
            }
            if let Some(parent) = self.language_parent(node)? {
                current = parent;
                continue;
            }
            let document = node.node_document();
            return self.language_fallback(document.subject());
        }
    }

    fn language_namespace(
        &mut self,
        element: &HtmlCapturedElement,
        expected: &str,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        let Some(uri) = element.name().namespace().uri() else {
            return Ok(false);
        };
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

    fn language_parent(
        &mut self,
        node: HtmlCapturedNode<'a>,
    ) -> LanguageResult<Option<HtmlCaptureSubjectId>> {
        let parent = self
            .parent(node)
            .map_err(LanguageStop::Failure)?
            .map_err(LanguageStop::Missing)?;
        let Some(parent) = parent else {
            return Ok(None);
        };
        let parent_node = self.node(&parent)?;
        match parent_node.kind() {
            HtmlCapturedNodeKind::Element(_) => Ok(Some(parent)),
            HtmlCapturedNodeKind::Fragment(fragment)
                if matches!(fragment.kind(), HtmlCapturedFragmentKind::ShadowRoot(_)) =>
            {
                self.charge(1)?;
                match parent_node.host() {
                    Some(HtmlCapturedHost::Element(host)) => Ok(Some(host.subject().clone())),
                    _ => Err(LanguageStop::Missing(HtmlSemanticMissing::ShadowHost(
                        parent,
                    ))),
                }
            }
            _ => Ok(None),
        }
    }

    fn language_fallback(
        &mut self,
        document: &HtmlCaptureSubjectId,
    ) -> LanguageResult<HtmlLanguage> {
        let Some(facts) = self.context.language_direction() else {
            return Err(LanguageStop::Missing(
                HtmlSemanticMissing::LanguageFallback(document.clone()),
            ));
        };
        let mut found = None;
        for fact in facts.fallbacks() {
            self.charge(1)?;
            if fact.document() == document {
                found = Some(fact);
                break;
            }
        }
        let Some(fact) = found else {
            return Err(LanguageStop::Missing(
                HtmlSemanticMissing::LanguageFallback(document.clone()),
            ));
        };
        self.charge(1)?;
        match fact.pragma() {
            HtmlPragmaLanguage::Tag(tag) => {
                return Ok(HtmlLanguage {
                    tag: tag.clone(),
                    origin: HtmlLanguageOrigin::Pragma(document.clone()),
                });
            }
            HtmlPragmaLanguage::Unavailable => {
                return Err(LanguageStop::Missing(
                    HtmlSemanticMissing::LanguageFallback(document.clone()),
                ));
            }
            HtmlPragmaLanguage::Absent => {}
        }
        self.charge(1)?;
        match fact.protocol() {
            HtmlProtocolLanguage::Single(tag) => Ok(HtmlLanguage {
                tag: tag.clone(),
                origin: HtmlLanguageOrigin::Protocol(document.clone()),
            }),
            HtmlProtocolLanguage::Absent | HtmlProtocolLanguage::Multiple => Ok(HtmlLanguage {
                tag: self.context.empty_language().clone(),
                origin: HtmlLanguageOrigin::NoInformation(document.clone()),
            }),
            HtmlProtocolLanguage::Unavailable => Err(LanguageStop::Missing(
                HtmlSemanticMissing::LanguageFallback(document.clone()),
            )),
        }
    }

    pub(super) fn directionality(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlDirectionality>, HtmlSemanticFailure> {
        if !matches!(self.node(subject)?.kind(), HtmlCapturedNodeKind::Element(_)) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let mut tasks = Vec::new();
        let result = self.direction_inner(subject, &mut tasks);
        self.release_scratch(tasks.capacity());
        match result {
            Ok(value) => Ok(HtmlSemanticValue::Known(value)),
            Err(LanguageStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(LanguageStop::Failure(failure)) => Err(failure),
        }
    }

    fn direction_push(
        &mut self,
        tasks: &mut Vec<DirectionTask<'a>>,
        task: DirectionTask<'a>,
    ) -> LanguageResult<()> {
        self.reserve_scratch(tasks, 1)?;
        self.charge(1)?;
        tasks.push(task);
        Ok(())
    }

    fn direction_state(
        &mut self,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlDirState, HtmlSemanticFailure> {
        if !self.html_namespace(element)? {
            return Ok(HtmlDirState::Undefined);
        }
        let raw = self.attribute(element, "dir")?;
        self.admit_intrinsic(element, raw, 3)?;
        self.interpreter
            .dir(element.name(), raw)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }

    fn direction_input_type(
        &mut self,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlInputType, HtmlSemanticFailure> {
        let raw = self.attribute(element, "type")?;
        self.admitted_input_type_report(element, raw)?
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }

    fn direction_children(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> LanguageResult<HtmlCapturedChildIds<'a>> {
        let node = self.node(subject)?;
        self.charge(1)?;
        match node.children() {
            HtmlCapturedChildren::Complete(children) => Ok(children),
            HtmlCapturedChildren::Unavailable => Err(LanguageStop::Missing(
                HtmlSemanticMissing::OrderedChildren(subject.clone()),
            )),
        }
    }

    fn direction_excluded(
        &mut self,
        element: &'a HtmlCapturedElement,
    ) -> Result<bool, HtmlSemanticFailure> {
        Ok(matches!(
            self.known(element)?,
            Some(
                KnownHtmlElement::Bdi
                    | KnownHtmlElement::Script
                    | KnownHtmlElement::Style
                    | KnownHtmlElement::Textarea
            )
        ) || self.direction_state(element)? != HtmlDirState::Undefined)
    }

    fn direction_shadow_host(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> LanguageResult<Option<HtmlCaptureSubjectId>> {
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            if let HtmlCapturedNodeKind::Fragment(fragment) = node.kind()
                && matches!(fragment.kind(), HtmlCapturedFragmentKind::ShadowRoot(_))
            {
                self.charge(1)?;
                return match node.host() {
                    Some(HtmlCapturedHost::Element(host)) => Ok(Some(host.subject().clone())),
                    _ => Err(LanguageStop::Missing(HtmlSemanticMissing::ShadowHost(
                        current,
                    ))),
                };
            }
            match self.parent(node)?.map_err(LanguageStop::Missing)? {
                Some(parent) => current = parent,
                None => return Ok(None),
            }
        }
    }

    pub(super) fn control_current_value(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Result<&'a DocumentString, HtmlSemanticMissing>, HtmlSemanticFailure> {
        if let Some(facts) = self.context.language_direction() {
            for fact in facts.current_values() {
                self.charge(1)?;
                if fact.subject() == subject {
                    return Ok(fact
                        .value()
                        .ok_or_else(|| HtmlSemanticMissing::CurrentValue(subject.clone())));
                }
            }
        }
        Ok(Err(HtmlSemanticMissing::CurrentValue(subject.clone())))
    }

    fn direction_assigned(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> LanguageResult<&'a [HtmlCaptureSubjectId]> {
        if let Some(facts) = self.context.language_direction() {
            for fact in facts.assignments() {
                self.charge(1)?;
                if fact.slot() == subject {
                    return fact.nodes().ok_or_else(|| {
                        LanguageStop::Missing(HtmlSemanticMissing::SlotAssignment(subject.clone()))
                    });
                }
            }
        }
        Err(LanguageStop::Missing(HtmlSemanticMissing::SlotAssignment(
            subject.clone(),
        )))
    }

    fn direction_first_strong(
        &mut self,
        value: &DocumentString,
    ) -> Result<Option<HtmlDirectionality>, HtmlSemanticFailure> {
        let units = value.code_units();
        let mut offset = 0;
        while offset < units.len() {
            self.charge(1)?;
            let first = units[offset];
            offset += 1;
            let mut code_point = u32::from(first);
            if (0xd800..=0xdbff).contains(&first) && offset < units.len() {
                self.charge(1)?;
                let second = units[offset];
                if (0xdc00..=0xdfff).contains(&second) {
                    offset += 1;
                    code_point =
                        0x10000 + ((u32::from(first) - 0xd800) << 10) + u32::from(second) - 0xdc00;
                }
            }
            // Provider admits at most ten range-start comparisons, one domain
            // and one end comparison. Surrogates retain their UCD L property.
            self.charge(12)?;
            let result = match bidi_class(code_point) {
                Some(BidiClass::L) => Some(HtmlDirectionality::Ltr),
                Some(BidiClass::R | BidiClass::Al) => Some(HtmlDirectionality::Rtl),
                Some(BidiClass::Other) => None,
                None => return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)),
            };
            self.poll()?;
            if result.is_some() {
                return Ok(result);
            }
        }
        Ok(None)
    }

    fn direction_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        tasks: &mut Vec<DirectionTask<'a>>,
    ) -> LanguageResult<HtmlDirectionality> {
        self.direction_push(tasks, DirectionTask::Direction(subject.clone()))?;
        let mut result = None;
        let mut returned = false;
        while !tasks.is_empty() {
            self.charge(1)?;
            let Some(task) = tasks.pop() else {
                break;
            };
            match task {
                DirectionTask::DefaultLtr => {
                    result = Some(result.unwrap_or(HtmlDirectionality::Ltr));
                    returned = true;
                }
                DirectionTask::AssignedNext(nodes) => {
                    if result.is_none() {
                        returned = false;
                        self.direction_push(tasks, DirectionTask::Assigned(nodes))?;
                    }
                }
                DirectionTask::Scan(_) if returned => {}
                DirectionTask::Direction(id) => {
                    let node = self.node(&id)?;
                    let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                        return Err(self
                            .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                            .into());
                    };
                    match self.direction_state(element)? {
                        HtmlDirState::Keyword(HtmlDir::Ltr) => {
                            result = Some(HtmlDirectionality::Ltr);
                            returned = true;
                        }
                        HtmlDirState::Keyword(HtmlDir::Rtl) => {
                            result = Some(HtmlDirectionality::Rtl);
                            returned = true;
                        }
                        HtmlDirState::Keyword(HtmlDir::Auto) => {
                            self.direction_push(tasks, DirectionTask::DefaultLtr)?;
                            self.direction_push(tasks, DirectionTask::Auto(id))?;
                        }
                        HtmlDirState::Undefined => {
                            let kind = self.known(element)?;
                            if kind == Some(KnownHtmlElement::Bdi) {
                                self.direction_push(tasks, DirectionTask::DefaultLtr)?;
                                self.direction_push(tasks, DirectionTask::Auto(id))?;
                            } else if kind == Some(KnownHtmlElement::Input)
                                && self.direction_input_type(element)? == HtmlInputType::Telephone
                            {
                                result = Some(HtmlDirectionality::Ltr);
                                returned = true;
                            } else if let Some(parent) = self.language_parent(node)? {
                                self.direction_push(tasks, DirectionTask::Direction(parent))?;
                            } else {
                                result = Some(HtmlDirectionality::Ltr);
                                returned = true;
                            }
                        }
                    }
                }
                DirectionTask::Auto(id) => {
                    let node = self.node(&id)?;
                    let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                        return Err(self
                            .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                            .into());
                    };
                    let kind = self.known(element)?;
                    let control = kind == Some(KnownHtmlElement::Textarea)
                        || (kind == Some(KnownHtmlElement::Input)
                            && matches!(
                                self.direction_input_type(element)?,
                                HtmlInputType::Hidden
                                    | HtmlInputType::Text
                                    | HtmlInputType::Search
                                    | HtmlInputType::Telephone
                                    | HtmlInputType::Url
                                    | HtmlInputType::Email
                                    | HtmlInputType::Password
                                    | HtmlInputType::Submit
                                    | HtmlInputType::Reset
                                    | HtmlInputType::Button
                            ));
                    if control {
                        let value = self
                            .control_current_value(&id)?
                            .map_err(LanguageStop::Missing)?;
                        result = self.direction_first_strong(value)?;
                        if result.is_none() && !value.is_empty() {
                            result = Some(HtmlDirectionality::Ltr);
                        }
                        returned = true;
                    } else if kind == Some(KnownHtmlElement::Slot)
                        && self.direction_shadow_host(&id)?.is_some()
                    {
                        let assigned = self.direction_assigned(&id)?;
                        if assigned.is_empty() {
                            self.direction_push(tasks, DirectionTask::Contained(id, false))?;
                        } else {
                            self.direction_push(tasks, DirectionTask::Assigned(assigned.iter()))?;
                        }
                    } else {
                        self.direction_push(tasks, DirectionTask::Contained(id, false))?;
                    }
                }
                DirectionTask::Contained(id, can_exclude_root) => {
                    let node = self.node(&id)?;
                    let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                        return Err(self
                            .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                            .into());
                    };
                    if can_exclude_root && self.direction_excluded(element)? {
                        result = None;
                        returned = true;
                    } else {
                        let children = self.direction_children(&id)?;
                        self.direction_push(tasks, DirectionTask::Scan(children))?;
                    }
                }
                DirectionTask::Assigned(mut nodes) => {
                    self.charge(1)?;
                    if let Some(id) = nodes.next() {
                        let node = self.node(id)?;
                        match node.kind() {
                            HtmlCapturedNodeKind::Text(text) => {
                                result = self.direction_first_strong(text)?;
                                if result.is_some() {
                                    returned = true;
                                } else {
                                    self.direction_push(tasks, DirectionTask::Assigned(nodes))?;
                                }
                            }
                            HtmlCapturedNodeKind::Element(_) => {
                                self.direction_push(tasks, DirectionTask::AssignedNext(nodes))?;
                                self.direction_push(
                                    tasks,
                                    DirectionTask::Contained(id.clone(), true),
                                )?;
                            }
                            _ => {
                                return Err(self
                                    .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                                    .into());
                            }
                        }
                    } else {
                        result = None;
                        returned = true;
                    }
                }
                DirectionTask::Scan(mut children) => {
                    self.charge(1)?;
                    if let Some(id) = children.next() {
                        let node = self.node(&id)?;
                        match node.kind() {
                            HtmlCapturedNodeKind::Element(element) => {
                                if self.direction_excluded(element)? {
                                    self.direction_push(tasks, DirectionTask::Scan(children))?;
                                } else if self.known(element)? == Some(KnownHtmlElement::Slot)
                                    && let Some(host) = self.direction_shadow_host(&id)?
                                {
                                    // A contained slot returns host directionality,
                                    // abandoning all later ordinary scan frames.
                                    self.direction_push(tasks, DirectionTask::Direction(host))?;
                                } else {
                                    self.direction_push(tasks, DirectionTask::Scan(children))?;
                                    let descendants = self.direction_children(&id)?;
                                    self.direction_push(tasks, DirectionTask::Scan(descendants))?;
                                }
                            }
                            HtmlCapturedNodeKind::Text(text) => {
                                result = self.direction_first_strong(text)?;
                                if result.is_some() {
                                    returned = true;
                                } else {
                                    self.direction_push(tasks, DirectionTask::Scan(children))?;
                                }
                            }
                            _ => self.direction_push(tasks, DirectionTask::Scan(children))?,
                        }
                    }
                }
            }
        }
        Ok(result.unwrap_or(HtmlDirectionality::Ltr))
    }
}

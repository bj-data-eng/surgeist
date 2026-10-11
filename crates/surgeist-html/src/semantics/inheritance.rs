//! HTML 877d6146487f inherited global state and the delegated editable
//! definition from w3c/editing 85de9ff4a981. No editing commands or host
//! state transitions are executed. Ordinary and shadow-host traversal remain
//! distinct where the selected algorithms require them.

use super::*;
use crate::{
    HtmlCapturedFragmentKind, HtmlCapturedHost, HtmlContentEditable, HtmlContentEditableState,
    HtmlSpellcheck, HtmlSpellcheckState, HtmlTranslate, HtmlTranslateState,
};

#[derive(Clone, Debug)]
pub struct HtmlGlobalInheritanceProducerId(Arc<()>);
impl HtmlGlobalInheritanceProducerId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlGlobalInheritanceProducerId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlGlobalInheritanceProducerId {}
/// Actual joined document-state/policy producer configuration and revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlGlobalInheritanceRevision {
    producer: HtmlGlobalInheritanceProducerId,
    configuration: u64,
    revision: u64,
}
impl HtmlGlobalInheritanceRevision {
    pub fn new(
        producer: HtmlGlobalInheritanceProducerId,
        configuration: u64,
        revision: u64,
    ) -> Self {
        Self {
            producer,
            configuration,
            revision,
        }
    }
    pub fn producer(&self) -> &HtmlGlobalInheritanceProducerId {
        &self.producer
    }
    pub fn configuration(&self) -> u64 {
        self.configuration
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDesignMode {
    Enabled,
    Disabled,
    Unavailable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDocumentDesignModeFact {
    document: HtmlCaptureSubjectId,
    state: HtmlDesignMode,
}
impl HtmlDocumentDesignModeFact {
    pub fn new(document: HtmlCaptureSubjectId, state: HtmlDesignMode) -> Self {
        Self { document, state }
    }
    pub fn document(&self) -> &HtmlCaptureSubjectId {
        &self.document
    }
    pub fn state(&self) -> HtmlDesignMode {
        self.state
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSpellcheckDefaultBehavior {
    TrueByDefault,
    FalseByDefault,
    InheritByDefault,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSpellcheckDefaultFact {
    subject: HtmlCaptureSubjectId,
    behavior: Option<HtmlSpellcheckDefaultBehavior>,
}
impl HtmlSpellcheckDefaultFact {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        behavior: Option<HtmlSpellcheckDefaultBehavior>,
    ) -> Self {
        Self { subject, behavior }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn behavior(&self) -> Option<HtmlSpellcheckDefaultBehavior> {
        self.behavior
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlGlobalInheritanceLimits {
    record_capacity: usize,
    validation_work: usize,
}
impl HtmlGlobalInheritanceLimits {
    pub fn new(record_capacity: usize, validation_work: usize) -> Self {
        Self {
            record_capacity,
            validation_work,
        }
    }
    pub fn record_capacity(self) -> usize {
        self.record_capacity
    }
    pub fn validation_work(self) -> usize {
        self.validation_work
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlGlobalInheritanceError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    RepresentationExhausted,
    RecordCapacity { limit: usize, required: usize },
    ValidationWork { limit: usize, required: usize },
    DuplicateSubject(HtmlCaptureSubjectId),
    InapplicableSubject(HtmlCaptureSubjectId),
    ParentlessInherit(HtmlCaptureSubjectId),
}
impl fmt::Display for HtmlGlobalInheritanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlGlobalInheritanceError {}
/// Immutable moved host-state facts, qualified by one capture and actual
/// joined producer key. Coverage may be partial; missing is never a default.
#[derive(Debug)]
pub struct HtmlGlobalInheritanceFacts {
    capture: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    revision: HtmlGlobalInheritanceRevision,
    design_modes: Vec<HtmlDocumentDesignModeFact>,
    spellcheck_defaults: Vec<HtmlSpellcheckDefaultFact>,
}
struct Validation {
    work: usize,
    limit: usize,
}
impl Validation {
    fn charge(&mut self, amount: usize) -> Result<(), HtmlGlobalInheritanceError> {
        let required = self
            .work
            .checked_add(amount)
            .ok_or(HtmlGlobalInheritanceError::RepresentationExhausted)?;
        if required > self.limit {
            return Err(HtmlGlobalInheritanceError::ValidationWork {
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
    ) -> Result<HtmlCapturedNode<'a>, HtmlGlobalInheritanceError> {
        self.charge(1)?;
        capture
            .node(id)
            .map_err(HtmlGlobalInheritanceError::Capture)
    }
    fn html(&mut self, element: &HtmlCapturedElement) -> Result<bool, HtmlGlobalInheritanceError> {
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
}
impl HtmlGlobalInheritanceFacts {
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        revision: HtmlGlobalInheritanceRevision,
        design_modes: Vec<HtmlDocumentDesignModeFact>,
        spellcheck_defaults: Vec<HtmlSpellcheckDefaultFact>,
        limits: HtmlGlobalInheritanceLimits,
    ) -> Result<Self, HtmlGlobalInheritanceError> {
        if &host_revision != capture.host_facts_revision() {
            return Err(HtmlGlobalInheritanceError::HostRevisionMismatch);
        }
        let required = design_modes
            .capacity()
            .checked_add(spellcheck_defaults.capacity())
            .ok_or(HtmlGlobalInheritanceError::RepresentationExhausted)?;
        if required > limits.record_capacity {
            return Err(HtmlGlobalInheritanceError::RecordCapacity {
                limit: limits.record_capacity,
                required,
            });
        }
        let mut validation = Validation {
            work: 0,
            limit: limits.validation_work,
        };
        for (i, fact) in design_modes.iter().enumerate() {
            let node = validation.node(capture, &fact.document)?;
            validation.charge(1)?;
            if !matches!(node.kind(), HtmlCapturedNodeKind::Document(_)) {
                return Err(HtmlGlobalInheritanceError::InapplicableSubject(
                    fact.document.clone(),
                ));
            }
            for previous in &design_modes[..i] {
                validation.charge(1)?;
                if previous.document == fact.document {
                    return Err(HtmlGlobalInheritanceError::DuplicateSubject(
                        fact.document.clone(),
                    ));
                }
            }
        }
        for (i, fact) in spellcheck_defaults.iter().enumerate() {
            let node = validation.node(capture, &fact.subject)?;
            validation.charge(1)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlGlobalInheritanceError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            };
            if !validation.html(element)? {
                return Err(HtmlGlobalInheritanceError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            }
            for previous in &spellcheck_defaults[..i] {
                validation.charge(1)?;
                if previous.subject == fact.subject {
                    return Err(HtmlGlobalInheritanceError::DuplicateSubject(
                        fact.subject.clone(),
                    ));
                }
            }
            validation.charge(1)?;
            if fact.behavior == Some(HtmlSpellcheckDefaultBehavior::InheritByDefault) {
                validation.charge(1)?;
                let parent = match node.parent() {
                    HtmlCapturedParent::Root => None,
                    HtmlCapturedParent::Unavailable => continue,
                    HtmlCapturedParent::Subject(id) => Some(validation.node(capture, &id)?),
                };
                validation.charge(1)?;
                if !parent
                    .is_some_and(|node| matches!(node.kind(), HtmlCapturedNodeKind::Element(_)))
                {
                    return Err(HtmlGlobalInheritanceError::ParentlessInherit(
                        fact.subject.clone(),
                    ));
                }
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host_revision,
            revision,
            design_modes,
            spellcheck_defaults,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn revision(&self) -> &HtmlGlobalInheritanceRevision {
        &self.revision
    }
    pub fn design_modes(&self) -> &[HtmlDocumentDesignModeFact] {
        &self.design_modes
    }
    pub fn spellcheck_defaults(&self) -> &[HtmlSpellcheckDefaultFact] {
        &self.spellcheck_defaults
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTranslationMode {
    Enabled,
    Disabled,
}
/// Source-computed level, including HTML's saturation at nine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlComputedHeadingLevel(u8);
impl HtmlComputedHeadingLevel {
    pub fn value(self) -> u8 {
        self.0
    }
}
/// Exact external definition selected only for the editable predicate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlEditabilitySource;
impl HtmlEditabilitySource {
    pub fn revision(self) -> &'static str {
        "85de9ff4a9816139d06fc2dc39d5dacdc456fddc"
    }
    pub fn complete_source_sha256(self) -> &'static str {
        "da222a4ea70349eba9d9036a20708f1fa63690118f6c657da3f7f06c56fa78cf"
    }
}
impl<'a> HtmlSemanticSession<'a> {
    fn inherited_contenteditable(
        &mut self,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlContentEditableState, HtmlSemanticFailure> {
        let raw = self.attribute(element, "contenteditable")?;
        self.admit_intrinsic(element, raw, 3)?;
        let report = self
            .interpreter
            .contenteditable(element.name(), raw)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        report
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }
    fn design_mode(
        &mut self,
        document: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        if let Some(facts) = self.context.global_inheritance() {
            for fact in facts.design_modes() {
                self.charge(1)?;
                if fact.document() == document {
                    self.charge(1)?;
                    match fact.state() {
                        HtmlDesignMode::Enabled => return Ok(HtmlSemanticValue::Known(true)),
                        HtmlDesignMode::Disabled => return Ok(HtmlSemanticValue::Known(false)),
                        HtmlDesignMode::Unavailable => break,
                    }
                }
            }
        }
        Ok(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::DocumentDesignMode(document.clone()),
        ))
    }
    pub(super) fn editing_host(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Known(false));
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Known(false));
        }
        if matches!(
            self.inherited_contenteditable(element)?,
            HtmlContentEditableState::Keyword(
                HtmlContentEditable::True | HtmlContentEditable::PlaintextOnly
            )
        ) {
            return Ok(HtmlSemanticValue::Known(true));
        }
        let parent = match self.parent(node)? {
            Ok(parent) => parent,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        if let Some(parent) = parent {
            let parent_node = self.node(&parent)?;
            if matches!(parent_node.kind(), HtmlCapturedNodeKind::Document(_)) {
                return self.design_mode(&parent);
            }
        }
        Ok(HtmlSemanticValue::Known(false))
    }
    fn inheritance_exact(
        &mut self,
        value: &DocumentString,
        expected: &str,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        if value.len() != expected.len() {
            return Ok(false);
        }
        for (unit, byte) in value.code_units().iter().zip(expected.bytes()) {
            self.charge(1)?;
            if *unit != u16::from(byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn inheritance_entry(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<bool, HtmlSemanticFailure> {
        let Some(uri) = element.name().namespace().uri() else {
            self.charge(1)?;
            return Ok(false);
        };
        Ok((self.inheritance_exact(uri, "http://www.w3.org/2000/svg")?
            && self.inheritance_exact(element.name().local_name(), "svg")?)
            || (self.inheritance_exact(uri, "http://www.w3.org/1998/Math/MathML")?
                && self.inheritance_exact(element.name().local_name(), "math")?))
    }
    pub(super) fn editable(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            self.charge(1)?;
            let non_element = match node.kind() {
                HtmlCapturedNodeKind::Document(_) | HtmlCapturedNodeKind::Fragment(_) => {
                    return Ok(HtmlSemanticValue::Known(false));
                }
                HtmlCapturedNodeKind::Element(element) => {
                    if self.html_namespace(element)? {
                        // A local host or False state independently defeats the
                        // delegated conjunction, without consulting design mode.
                        if !matches!(
                            self.inherited_contenteditable(element)?,
                            HtmlContentEditableState::Inherit
                        ) {
                            return Ok(HtmlSemanticValue::Known(false));
                        }
                    } else if !self.inheritance_entry(element)? {
                        return Ok(HtmlSemanticValue::Known(false));
                    }
                    false
                }
                _ => true,
            };
            let parent = match self.parent(node)? {
                Ok(Some(parent)) => parent,
                Ok(None) => return Ok(HtmlSemanticValue::Known(false)),
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            let parent_node = self.node(&parent)?;
            let HtmlCapturedNodeKind::Element(parent_element) = parent_node.kind() else {
                return Ok(HtmlSemanticValue::Known(false));
            };
            if non_element && !self.html_namespace(parent_element)? {
                return Ok(HtmlSemanticValue::Known(false));
            }
            match self.editing_host(&parent)? {
                HtmlSemanticValue::Known(true) => return Ok(HtmlSemanticValue::Known(true)),
                HtmlSemanticValue::Known(false) => current = parent,
                value => return Ok(value),
            }
        }
    }
    pub(super) fn is_content_editable(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let host = self.editing_host(subject)?;
        if host == HtmlSemanticValue::Known(true) {
            return Ok(host);
        }
        let editable = self.editable(subject)?;
        Ok(match (host, editable) {
            (_, HtmlSemanticValue::Known(true)) => HtmlSemanticValue::Known(true),
            (HtmlSemanticValue::Known(false), value) => value,
            (value, _) => value,
        })
    }
    pub(super) fn translation_mode(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTranslationMode>, HtmlSemanticFailure> {
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Ok(HtmlSemanticValue::Inapplicable);
            };
            if self.html_namespace(element)? {
                let raw = self.attribute(element, "translate")?;
                self.admit_intrinsic(element, raw, 2)?;
                let report = self
                    .interpreter
                    .translate(element.name(), raw)
                    .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                match report
                    .intrinsic()
                    .copied()
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?
                {
                    HtmlTranslateState::Keyword(HtmlTranslate::Yes) => {
                        return Ok(HtmlSemanticValue::Known(HtmlTranslationMode::Enabled));
                    }
                    HtmlTranslateState::Keyword(HtmlTranslate::No) => {
                        return Ok(HtmlSemanticValue::Known(HtmlTranslationMode::Disabled));
                    }
                    HtmlTranslateState::Inherit => {}
                }
            }
            current = match self.parent(node)? {
                Ok(Some(id))
                    if matches!(self.node(&id)?.kind(), HtmlCapturedNodeKind::Element(_)) =>
                {
                    id
                }
                Ok(_) => return Ok(HtmlSemanticValue::Known(HtmlTranslationMode::Enabled)),
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
        }
    }
    pub(super) fn spellcheck(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Ok(HtmlSemanticValue::Inapplicable);
            };
            if !self.html_namespace(element)? {
                return Ok(if &current == subject {
                    HtmlSemanticValue::Inapplicable
                } else {
                    HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
                });
            }
            let raw = self.attribute(element, "spellcheck")?;
            self.admit_intrinsic(element, raw, 2)?;
            let report = self
                .interpreter
                .spellcheck(element.name(), raw)
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
            match report
                .intrinsic()
                .copied()
                .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?
            {
                HtmlSpellcheckState::Keyword(HtmlSpellcheck::True) => {
                    return Ok(HtmlSemanticValue::Known(true));
                }
                HtmlSpellcheckState::Keyword(HtmlSpellcheck::False) => {
                    return Ok(HtmlSemanticValue::Known(false));
                }
                HtmlSpellcheckState::Default => {}
            }
            let mut behavior = None;
            if let Some(facts) = self.context.global_inheritance() {
                for fact in facts.spellcheck_defaults() {
                    self.charge(1)?;
                    if fact.subject() == &current {
                        self.charge(1)?;
                        behavior = fact.behavior();
                        break;
                    }
                }
            }
            match behavior {
                Some(HtmlSpellcheckDefaultBehavior::TrueByDefault) => {
                    return Ok(HtmlSemanticValue::Known(true));
                }
                Some(HtmlSpellcheckDefaultBehavior::FalseByDefault) => {
                    return Ok(HtmlSemanticValue::Known(false));
                }
                Some(HtmlSpellcheckDefaultBehavior::InheritByDefault) => {
                    current = match self.parent(node)? {
                        Ok(Some(id))
                            if matches!(
                                self.node(&id)?.kind(),
                                HtmlCapturedNodeKind::Element(_)
                            ) =>
                        {
                            id
                        }
                        Ok(_) => {
                            return Ok(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(
                                current,
                            )));
                        }
                        Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                    };
                }
                None => {
                    return Ok(HtmlSemanticValue::Missing(
                        HtmlSemanticMissing::SpellcheckDefault(current),
                    ));
                }
            }
        }
    }
    pub(super) fn computed_heading_level(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlComputedHeadingLevel>, HtmlSemanticFailure> {
        let mut current = subject.clone();
        let node = self.node(&current)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        let mut level = 0_u8;
        for (name, base) in [
            ("h1", 1),
            ("h2", 2),
            ("h3", 3),
            ("h4", 4),
            ("h5", 5),
            ("h6", 6),
        ] {
            if self.inheritance_exact(element.name().local_name(), name)? {
                level = base;
                break;
            }
        }
        if level == 0 {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        loop {
            let node = self.node(&current)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Ok(HtmlSemanticValue::Known(HtmlComputedHeadingLevel(level)));
            };
            if self.html_namespace(element)? {
                if let Some(raw) = self.attribute(element, "headingoffset")? {
                    self.interpreter
                        .admit_attribute_raw(Some(raw.value()))
                        .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                    self.numeric_budget(raw.value().len())?;
                    let offset = crate::numeric::parse_nonnegative_integer_capped(raw.value(), 9)
                        .unwrap_or(0);
                    self.poll()?;
                    level = (usize::from(level) + offset).min(9) as u8;
                    if level == 9 {
                        return Ok(HtmlSemanticValue::Known(HtmlComputedHeadingLevel(level)));
                    }
                }
                let raw = self.attribute(element, "headingreset")?;
                self.admit_intrinsic(element, raw, 1)?;
                let reset = self
                    .interpreter
                    .boolean_attribute(HtmlBooleanAttribute::HeadingReset, element.name(), raw)
                    .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                if reset
                    .intrinsic()
                    .copied()
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?
                {
                    return Ok(HtmlSemanticValue::Known(HtmlComputedHeadingLevel(level)));
                }
            }
            let parent = match self.parent(node)? {
                Ok(Some(id)) => id,
                Ok(None) => return Ok(HtmlSemanticValue::Known(HtmlComputedHeadingLevel(level))),
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            let parent_node = self.node(&parent)?;
            self.charge(1)?;
            current = match parent_node.kind() {
                HtmlCapturedNodeKind::Element(_) => parent,
                HtmlCapturedNodeKind::Fragment(fragment)
                    if matches!(fragment.kind(), HtmlCapturedFragmentKind::ShadowRoot(_)) =>
                {
                    self.charge(1)?;
                    match parent_node.host() {
                        Some(HtmlCapturedHost::Element(host)) => host.subject().clone(),
                        _ => {
                            return Ok(HtmlSemanticValue::Missing(
                                HtmlSemanticMissing::ShadowHost(parent),
                            ));
                        }
                    }
                }
                _ => return Ok(HtmlSemanticValue::Known(HtmlComputedHeadingLevel(level))),
            };
        }
    }
}

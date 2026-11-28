//! HTML 877d6146487f inherited global state and the delegated editable
//! definition from w3c/editing 85de9ff4a981. No editing commands or host
//! state transitions are executed. Ordinary and shadow-host traversal remain
//! distinct where the selected algorithms require them.

use super::admission::producer_revision;
mod spellcheck_index;
use super::index::FactIndex;
use super::scan::{Caps, ScanError, ScanMeter};
use super::*;
use crate::{HtmlCapturedFragmentKind, HtmlCapturedHost};
use spellcheck_index::SpellcheckIndex;
use surgeist_html::accounting::HtmlAccountingFault;
use surgeist_html::accounting::{HtmlCapScope, HtmlResource};
use surgeist_html::attributes::{
    HtmlContentEditable, HtmlContentEditableState, HtmlSpellcheck, HtmlSpellcheckState,
    HtmlTranslate, HtmlTranslateState,
};
use surgeist_html::values::HtmlNameInput;
use surgeist_html::{attributes::HtmlBooleanAttribute, values::DocumentString};

identity!(HtmlGlobalInheritanceProducerId, fresh);
producer_revision!(
    /// Actual joined document-state/policy producer configuration and revision.
    HtmlGlobalInheritanceRevision,
    HtmlGlobalInheritanceProducerId
);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDesignMode {
    Enabled,
    Disabled,
    Unavailable,
}

/// The editing rule selected by an HTML element's intrinsic contenteditable state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlEditableAttribute {
    /// The attribute itself establishes an editing host.
    ExplicitHost,
    /// The attribute permits editability from an ordinary parent Element.
    Inherit,
    /// The attribute blocks Element inheritance. Direct Document design mode
    /// can still establish an editing host.
    LocalFalse,
}

impl From<HtmlContentEditableState> for HtmlEditableAttribute {
    fn from(state: HtmlContentEditableState) -> Self {
        match state {
            HtmlContentEditableState::Keyword(
                HtmlContentEditable::True | HtmlContentEditable::PlaintextOnly,
            ) => Self::ExplicitHost,
            HtmlContentEditableState::Keyword(HtmlContentEditable::False) => Self::LocalFalse,
            HtmlContentEditableState::Inherit => Self::Inherit,
        }
    }
}

/// Check whether a foreign expanded name can inherit editability from an
/// ordinary parent Element. Only SVG `svg` and MathML `math` qualify.
/// The caller admits each reached comparison through `step`.
pub fn html_foreign_root_inherits_editability_with<E>(
    name: &(impl HtmlNameInput + ?Sized),
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    use super::scan::name_part_is_with;

    let Some(uri) = name.namespace_uri() else {
        step()?;
        return Ok(false);
    };
    Ok(
        (name_part_is_with(|| Some(uri), b"http://www.w3.org/2000/svg", &mut step)?
            && name_part_is_with(|| Some(name.local_name()), b"svg", &mut step)?)
            || (name_part_is_with(
                || Some(uri),
                b"http://www.w3.org/1998/Math/MathML",
                &mut step,
            )? && name_part_is_with(|| Some(name.local_name()), b"math", &mut step)?),
    )
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
    /// Bounds the two input vectors and validation work. The spellcheck index
    /// admits at most four position words per allowed record.
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
    /// The ledger that carries the accounting failed in a way that is not a refused limit: a release
    /// that matches nothing held, a ledger that cannot allocate, a scope that is not the
    /// operation's, or a change the operation does not charge. It carries the accounting fault.
    Accounting(HtmlAccountingFault),
    RecordCapacity {
        limit: usize,
        required: usize,
    },
    SpellcheckIndexBytes {
        limit: usize,
        required: usize,
    },
    ValidationWork {
        limit: usize,
        required: usize,
    },
    DuplicateSubject(HtmlCaptureSubjectId),
    InapplicableSubject(HtmlCaptureSubjectId),
    ParentlessInherit(HtmlCaptureSubjectId),
}
impl HtmlGlobalInheritanceError {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Accounting(fault) => Some(*fault),
            Self::Capture(error) => error.accounting_fault(),
            Self::HostRevisionMismatch
            | Self::RepresentationExhausted
            | Self::RecordCapacity { .. }
            | Self::SpellcheckIndexBytes { .. }
            | Self::ValidationWork { .. }
            | Self::DuplicateSubject(_)
            | Self::InapplicableSubject(_)
            | Self::ParentlessInherit(_) => None,
        }
    }
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
    design_mode_index: FactIndex,
    spellcheck_index: SpellcheckIndex,
}
impl ScanError for HtmlGlobalInheritanceError {
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
impl HtmlGlobalInheritanceFacts {
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        revision: HtmlGlobalInheritanceRevision,
        design_modes: Vec<HtmlDocumentDesignModeFact>,
        spellcheck_defaults: Vec<HtmlSpellcheckDefaultFact>,
        limits: HtmlGlobalInheritanceLimits,
    ) -> Result<Self, HtmlGlobalInheritanceError> {
        Self::build(
            capture,
            host_revision,
            revision,
            design_modes,
            spellcheck_defaults,
            limits,
            None,
        )
    }
    /// Charges below the scope `accounting` names, in the host's operation, as [`Self::new`]
    /// charges an operation of its own. The schema module documentation of
    /// `surgeist_html::accounting` lists the scopes it creates and what the host keeps.
    pub fn with_accounting(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        revision: HtmlGlobalInheritanceRevision,
        design_modes: Vec<HtmlDocumentDesignModeFact>,
        spellcheck_defaults: Vec<HtmlSpellcheckDefaultFact>,
        limits: HtmlGlobalInheritanceLimits,
        accounting: &HtmlHostScope,
    ) -> Result<Self, HtmlGlobalInheritanceError> {
        Self::build(
            capture,
            host_revision,
            revision,
            design_modes,
            spellcheck_defaults,
            limits,
            Some(accounting),
        )
    }
    fn build(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        revision: HtmlGlobalInheritanceRevision,
        design_modes: Vec<HtmlDocumentDesignModeFact>,
        spellcheck_defaults: Vec<HtmlSpellcheckDefaultFact>,
        limits: HtmlGlobalInheritanceLimits,
        hosted: Option<&HtmlHostScope>,
    ) -> Result<Self, HtmlGlobalInheritanceError> {
        if &host_revision != capture.host_facts_revision() {
            return Err(HtmlGlobalInheritanceError::HostRevisionMismatch);
        }
        let required = design_modes
            .capacity()
            .checked_add(spellcheck_defaults.capacity())
            .ok_or(HtmlGlobalInheritanceError::RepresentationExhausted)?;
        let mut validation = ScanMeter::charging(
            HtmlCapScope::GlobalInheritance,
            Caps::work(limits.validation_work)
                .records(limits.record_capacity)
                .bytes(SpellcheckIndex::byte_ceiling(limits.record_capacity)),
            hosted,
        )?;
        validation.acquire(HtmlResource::FactRecords, required, |limit, required| {
            HtmlGlobalInheritanceError::RecordCapacity { limit, required }
        })?;
        let mut design_mode_index = FactIndex::new(
            capture.identity(),
            design_modes.len(),
            |n| validation.charge(n),
            HtmlGlobalInheritanceError::RepresentationExhausted,
        )?;
        for (position, fact) in design_modes.iter().enumerate() {
            design_mode_index.enter(&fact.document, position);
        }
        for (position, fact) in design_modes.iter().enumerate() {
            let node = validation.node(capture, &fact.document)?;
            validation.charge(1)?;
            if !matches!(node.kind(), HtmlCapturedNodeKind::Document(_)) {
                return Err(HtmlGlobalInheritanceError::InapplicableSubject(
                    fact.document.clone(),
                ));
            }
            if design_mode_index.position(&fact.document) != Some(position) {
                return Err(HtmlGlobalInheritanceError::DuplicateSubject(
                    fact.document.clone(),
                ));
            }
        }
        let mut spellcheck_index =
            SpellcheckIndex::new(spellcheck_defaults.len(), &mut validation)?;
        for (position, fact) in spellcheck_defaults.iter().enumerate() {
            let node = validation.node(capture, &fact.subject)?;
            validation.charge(1)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlGlobalInheritanceError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            };
            if !validation.html_namespace(element)? {
                return Err(HtmlGlobalInheritanceError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            }
            spellcheck_index.insert(&spellcheck_defaults, position, &mut validation)?;
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
            design_mode_index,
            spellcheck_index,
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
impl HtmlTranslationMode {
    /// Select a local mode from an HTML element's intrinsic translation state.
    /// `None` continues the ordinary parent Element walk.
    pub const fn from_attribute(state: HtmlTranslateState) -> Option<Self> {
        match state {
            HtmlTranslateState::Keyword(HtmlTranslate::Yes) => Some(Self::Enabled),
            HtmlTranslateState::Keyword(HtmlTranslate::No) => Some(Self::Disabled),
            HtmlTranslateState::Inherit => None,
        }
    }
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
    ) -> Result<HtmlEditableAttribute, HtmlSemanticFailure> {
        let raw = self.attribute(element, "contenteditable")?;
        self.admit_intrinsic(element, raw, 3)?;
        let report = self
            .interpreter
            .contenteditable(element.name(), raw)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        report
            .intrinsic()
            .copied()
            .map(HtmlEditableAttribute::from)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }
    fn design_mode(
        &mut self,
        document: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        if let Some(facts) = self.context.global_inheritance() {
            self.charge(1)?;
            if let Some(position) = facts.design_mode_index.position(document) {
                self.charge(1)?;
                match facts.design_modes[position].state() {
                    HtmlDesignMode::Enabled => return Ok(HtmlSemanticValue::Known(true)),
                    HtmlDesignMode::Disabled => return Ok(HtmlSemanticValue::Known(false)),
                    HtmlDesignMode::Unavailable => {}
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
        if self.inherited_contenteditable(element)? == HtmlEditableAttribute::ExplicitHost {
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
                        if self.inherited_contenteditable(element)?
                            != HtmlEditableAttribute::Inherit
                        {
                            return Ok(HtmlSemanticValue::Known(false));
                        }
                    } else if !html_foreign_root_inherits_editability_with(element.name(), || {
                        self.charge(1)
                    })? {
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
    /// Selected execCommand editing-host-of definition, ordinary ancestry only.
    pub(super) fn editing_host_of(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        match self.editing_host(subject)? {
            HtmlSemanticValue::Known(true) => {
                self.charge(1)?;
                return Ok(HtmlSemanticValue::Known(Some(subject.clone())));
            }
            HtmlSemanticValue::Known(false) => {}
            value => return Ok(value.map(|_| None)),
        }
        match self.editable(subject)? {
            HtmlSemanticValue::Known(false) => return Ok(HtmlSemanticValue::Known(None)),
            HtmlSemanticValue::Known(true) => {}
            value => return Ok(value.map(|_| None)),
        }
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            current = match self.parent(node)? {
                Ok(Some(parent)) => parent,
                Ok(None) => return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)),
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            match self.editing_host(&current)? {
                HtmlSemanticValue::Known(true) => {
                    return Ok(HtmlSemanticValue::Known(Some(current)));
                }
                HtmlSemanticValue::Known(false) => {}
                value => return Ok(value.map(|_| None)),
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
                let state = report
                    .intrinsic()
                    .copied()
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
                if let Some(mode) = HtmlTranslationMode::from_attribute(state) {
                    return Ok(HtmlSemanticValue::Known(mode));
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
                self.charge(1)?;
                if let Some(position) = facts.spellcheck_index.position_with(
                    &facts.spellcheck_defaults,
                    &current,
                    || self.charge(1),
                )? {
                    self.charge(1)?;
                    behavior = facts.spellcheck_defaults[position].behavior();
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
                    let offset = surgeist_html::bounded::numeric::parse_nonnegative_integer_capped(
                        raw.value(),
                        9,
                    )
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

//! Immutable form associations, label targeting, control labels and current internal flags.
//! HTML 877d6146487f §4.10.2, §4.10.4, §4.10.5, §4.10.10 and §4.10.18.
//! Host adapters establish history and current slots; queries execute no transitions.

use super::*;
use crate::{HtmlCapturedFragmentKind, HtmlCapturedHost, HtmlInputType};

/// Evidence of the actual association phase at this capture's host revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlFormAssociationState {
    Unavailable,
    /// The actual current pointer, including parser-established nonancestor owners.
    EstablishedCurrent(Option<HtmlCaptureSubjectId>),
    /// The producer completed `reset-the-form-owner` against exactly this tree
    /// and attributes. This is transition evidence, not a structural guess.
    ResetAgainstCapture,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlFormAssociation {
    subject: HtmlCaptureSubjectId,
    state: HtmlFormAssociationState,
}
impl HtmlFormAssociation {
    pub fn new(subject: HtmlCaptureSubjectId, state: HtmlFormAssociationState) -> Self {
        Self { subject, state }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn state(&self) -> &HtmlFormAssociationState {
        &self.state
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFormOwnerOrigin {
    EstablishedCurrent,
    ResetAgainstCapture,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlFormOwner {
    owner: Option<HtmlCaptureSubjectId>,
    origin: HtmlFormOwnerOrigin,
}
impl HtmlFormOwner {
    pub fn owner(&self) -> Option<&HtmlCaptureSubjectId> {
        self.owner.as_ref()
    }
    pub fn origin(&self) -> HtmlFormOwnerOrigin {
        self.origin
    }
}

/// Complete immutable label association snapshot in ordinary tree order.
/// Custom-control associations describe the source ElementInternals label list,
/// not attachment, a live NodeList or an element labels IDL attribute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlControlLabels {
    labels: Vec<HtmlCaptureSubjectId>,
}
impl HtmlControlLabels {
    pub fn labels(&self) -> &[HtmlCaptureSubjectId] {
        &self.labels
    }
}

/// All input elements have these internal slots. Checkedness has control effects
/// in Checkbox/Radio states, and indeterminateness has effects in Checkbox state.
/// Each `None` is unavailable evidence, independently of the other slots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlInputControlState {
    subject: HtmlCaptureSubjectId,
    checkedness: Option<bool>,
    dirty_checkedness: Option<bool>,
    dirty_value: Option<bool>,
    indeterminateness: Option<bool>,
}
impl HtmlInputControlState {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        checkedness: Option<bool>,
        dirty_checkedness: Option<bool>,
        dirty_value: Option<bool>,
        indeterminateness: Option<bool>,
    ) -> Self {
        Self {
            subject,
            checkedness,
            dirty_checkedness,
            dirty_value,
            indeterminateness,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn checkedness(&self) -> Option<bool> {
        self.checkedness
    }
    pub fn dirty_checkedness(&self) -> Option<bool> {
        self.dirty_checkedness
    }
    pub fn dirty_value(&self) -> Option<bool> {
        self.dirty_value
    }
    pub fn indeterminateness(&self) -> Option<bool> {
        self.indeterminateness
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlOptionControlState {
    subject: HtmlCaptureSubjectId,
    selectedness: Option<bool>,
    dirtiness: Option<bool>,
}
impl HtmlOptionControlState {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        selectedness: Option<bool>,
        dirtiness: Option<bool>,
    ) -> Self {
        Self {
            subject,
            selectedness,
            dirtiness,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn selectedness(&self) -> Option<bool> {
        self.selectedness
    }
    pub fn dirtiness(&self) -> Option<bool> {
        self.dirtiness
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTextareaControlState {
    subject: HtmlCaptureSubjectId,
    dirty_value: Option<bool>,
}
impl HtmlTextareaControlState {
    pub fn new(subject: HtmlCaptureSubjectId, dirty_value: Option<bool>) -> Self {
        Self {
            subject,
            dirty_value,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn dirty_value(&self) -> Option<bool> {
        self.dirty_value
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlControlFlag {
    Checkedness,
    Selectedness,
    DirtyValue,
    DirtyCheckedness,
    OptionDirtiness,
    Indeterminateness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlControlFactsLimits {
    records: usize,
    validation_work: usize,
}
impl HtmlControlFactsLimits {
    /// Caps moved record capacity and validation work, including the remaining
    /// classification join when these facts enter a semantic context.
    pub fn new(record_capacity: usize, validation_work: usize) -> Self {
        Self {
            records: record_capacity,
            validation_work,
        }
    }
    pub fn record_capacity(self) -> usize {
        self.records
    }
    pub fn validation_work(self) -> usize {
        self.validation_work
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlControlFactsError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    RecordCapacity { limit: usize, required: usize },
    ValidationWork { limit: usize, required: usize },
    RepresentationExhausted,
    DuplicateSubject(HtmlCaptureSubjectId),
    InapplicableSubject(HtmlCaptureSubjectId),
    InvalidFormOwner(HtmlCaptureSubjectId),
    CrossDocumentFormOwner(HtmlCaptureSubjectId),
    CrossTreeFormOwner(HtmlCaptureSubjectId),
    ContradictoryCustomControlAssociation(HtmlCaptureSubjectId),
}
impl fmt::Display for HtmlControlFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlControlFactsError {}
/// Moved bounded records. Native kind checks do not reconstruct state from markup.
#[derive(Debug)]
pub struct HtmlControlFacts {
    capture: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    associations: Vec<HtmlFormAssociation>,
    inputs: Vec<HtmlInputControlState>,
    options: Vec<HtmlOptionControlState>,
    textareas: Vec<HtmlTextareaControlState>,
    validation_work: usize,
    validation_limit: usize,
}
struct ControlValidation {
    work: usize,
    limit: usize,
}
impl ControlValidation {
    fn root(
        &mut self,
        capture: &HtmlCapture,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Option<HtmlCaptureSubjectId>, HtmlControlFactsError> {
        let mut current = subject.clone();
        loop {
            let node = self.node(capture, &current)?;
            self.charge(1)?;
            match node.parent() {
                HtmlCapturedParent::Subject(parent) => current = parent,
                HtmlCapturedParent::Root => return Ok(Some(current)),
                HtmlCapturedParent::Unavailable => return Ok(None),
            }
        }
    }
    fn charge(&mut self, amount: usize) -> Result<(), HtmlControlFactsError> {
        let required = self
            .work
            .checked_add(amount)
            .ok_or(HtmlControlFactsError::RepresentationExhausted)?;
        if required > self.limit {
            return Err(HtmlControlFactsError::ValidationWork {
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
    ) -> Result<HtmlCapturedNode<'a>, HtmlControlFactsError> {
        self.charge(1)?;
        capture.node(id).map_err(HtmlControlFactsError::Capture)
    }
    fn kind(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<Option<KnownHtmlElement>, HtmlControlFactsError> {
        let amount = element
            .name()
            .local_name()
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| n.checked_add(element.name().namespace().uri().map_or(0, |s| s.len())))
            .ok_or(HtmlControlFactsError::RepresentationExhausted)?;
        self.charge(amount)?;
        Ok(KnownHtmlElement::from_name(element.name()))
    }
    fn html(&mut self, element: &HtmlCapturedElement) -> Result<bool, HtmlControlFactsError> {
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
fn native_form_associated(kind: KnownHtmlElement) -> bool {
    matches!(
        kind,
        KnownHtmlElement::Button
            | KnownHtmlElement::Fieldset
            | KnownHtmlElement::Input
            | KnownHtmlElement::Object
            | KnownHtmlElement::Output
            | KnownHtmlElement::Select
            | KnownHtmlElement::Textarea
            | KnownHtmlElement::Img
    )
}
impl HtmlControlFacts {
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        associations: Vec<HtmlFormAssociation>,
        inputs: Vec<HtmlInputControlState>,
        options: Vec<HtmlOptionControlState>,
        textareas: Vec<HtmlTextareaControlState>,
        limits: HtmlControlFactsLimits,
    ) -> Result<Self, HtmlControlFactsError> {
        if capture.host_facts_revision() != &host_revision {
            return Err(HtmlControlFactsError::HostRevisionMismatch);
        }
        let required = associations
            .capacity()
            .checked_add(inputs.capacity())
            .and_then(|n| n.checked_add(options.capacity()))
            .and_then(|n| n.checked_add(textareas.capacity()))
            .ok_or(HtmlControlFactsError::RepresentationExhausted)?;
        if required > limits.records {
            return Err(HtmlControlFactsError::RecordCapacity {
                limit: limits.records,
                required,
            });
        }
        let mut validation = ControlValidation {
            work: 0,
            limit: limits.validation_work,
        };
        for (index, fact) in associations.iter().enumerate() {
            validation.charge(1)?;
            let node = validation.node(capture, &fact.subject)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlControlFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            };
            if !validation.html(element)?
                || validation
                    .kind(element)?
                    .is_some_and(|kind| !native_form_associated(kind))
            {
                return Err(HtmlControlFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            }
            for previous in &associations[..index] {
                validation.charge(1)?;
                if previous.subject == fact.subject {
                    return Err(HtmlControlFactsError::DuplicateSubject(
                        fact.subject.clone(),
                    ));
                }
            }
            if let HtmlFormAssociationState::EstablishedCurrent(Some(owner)) = &fact.state {
                let owner_node = validation.node(capture, owner)?;
                let HtmlCapturedNodeKind::Element(owner_element) = owner_node.kind() else {
                    return Err(HtmlControlFactsError::InvalidFormOwner(owner.clone()));
                };
                if validation.kind(owner_element)? != Some(KnownHtmlElement::Form) {
                    return Err(HtmlControlFactsError::InvalidFormOwner(owner.clone()));
                }
                validation.charge(2)?;
                if node.node_document() != owner_node.node_document() {
                    return Err(HtmlControlFactsError::CrossDocumentFormOwner(owner.clone()));
                }
                let subject_root = validation.root(capture, &fact.subject)?;
                let owner_root = validation.root(capture, owner)?;
                if let (Some(subject_root), Some(owner_root)) = (subject_root, owner_root)
                    && subject_root != owner_root
                {
                    return Err(HtmlControlFactsError::CrossTreeFormOwner(owner.clone()));
                }
            }
        }
        for (index, fact) in inputs.iter().enumerate() {
            validation.charge(1)?;
            let node = validation.node(capture, &fact.subject)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlControlFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            };
            if validation.kind(element)? != Some(KnownHtmlElement::Input) {
                return Err(HtmlControlFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            }
            for previous in &inputs[..index] {
                validation.charge(1)?;
                if previous.subject == fact.subject {
                    return Err(HtmlControlFactsError::DuplicateSubject(
                        fact.subject.clone(),
                    ));
                }
            }
        }
        for (index, fact) in options.iter().enumerate() {
            validation.charge(1)?;
            let node = validation.node(capture, &fact.subject)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlControlFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            };
            if validation.kind(element)? != Some(KnownHtmlElement::Option) {
                return Err(HtmlControlFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            }
            for previous in &options[..index] {
                validation.charge(1)?;
                if previous.subject == fact.subject {
                    return Err(HtmlControlFactsError::DuplicateSubject(
                        fact.subject.clone(),
                    ));
                }
            }
        }
        for (index, fact) in textareas.iter().enumerate() {
            validation.charge(1)?;
            let node = validation.node(capture, &fact.subject)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlControlFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            };
            if validation.kind(element)? != Some(KnownHtmlElement::Textarea) {
                return Err(HtmlControlFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            }
            for previous in &textareas[..index] {
                validation.charge(1)?;
                if previous.subject == fact.subject {
                    return Err(HtmlControlFactsError::DuplicateSubject(
                        fact.subject.clone(),
                    ));
                }
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host_revision,
            associations,
            inputs,
            options,
            textareas,
            validation_work: validation.work,
            validation_limit: validation.limit,
        })
    }
    /// Complete the bounded context join using the constructor's remaining work.
    pub(super) fn validate_classifications(
        mut self,
        capture: &HtmlCapture,
        classifications: &[HtmlCustomControlClassification],
    ) -> Result<Self, HtmlControlFactsError> {
        let mut validation = ControlValidation {
            work: self.validation_work,
            limit: self.validation_limit,
        };
        for association in &self.associations {
            validation.charge(1)?;
            let node = validation.node(capture, &association.subject)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlControlFactsError::InapplicableSubject(
                    association.subject.clone(),
                ));
            };
            if validation.kind(element)?.is_some() {
                continue;
            }
            for classification in classifications {
                validation.charge(1)?;
                if classification.subject == association.subject {
                    validation.charge(1)?;
                    if !classification.form_associated {
                        return Err(
                            HtmlControlFactsError::ContradictoryCustomControlAssociation(
                                association.subject.clone(),
                            ),
                        );
                    }
                    break;
                }
            }
        }
        self.validation_work = validation.work;
        Ok(self)
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn associations(&self) -> &[HtmlFormAssociation] {
        &self.associations
    }
    pub fn inputs(&self) -> &[HtmlInputControlState] {
        &self.inputs
    }
    pub fn options(&self) -> &[HtmlOptionControlState] {
        &self.options
    }
    pub fn textareas(&self) -> &[HtmlTextareaControlState] {
        &self.textareas
    }
}

enum ControlStop {
    Failure(HtmlSemanticFailure),
    Missing(HtmlSemanticMissing),
}
impl From<HtmlSemanticFailure> for ControlStop {
    fn from(value: HtmlSemanticFailure) -> Self {
        Self::Failure(value)
    }
}
type ControlResult<T> = Result<T, ControlStop>;
impl<'a> HtmlSemanticSession<'a> {
    fn control_custom(&mut self, subject: &HtmlCaptureSubjectId) -> ControlResult<bool> {
        for fact in self.context.classifications() {
            self.charge(1)?;
            if fact.subject() == subject {
                return Ok(fact.form_associated());
            }
        }
        Err(ControlStop::Missing(
            HtmlSemanticMissing::CustomControlClassification(subject.clone()),
        ))
    }
    fn control_category(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
        labelable: bool,
    ) -> ControlResult<bool> {
        let kind = self.known(element)?;
        if let Some(kind) = kind {
            return Ok(if labelable {
                match kind {
                    KnownHtmlElement::Button
                    | KnownHtmlElement::Meter
                    | KnownHtmlElement::Output
                    | KnownHtmlElement::Progress
                    | KnownHtmlElement::Select
                    | KnownHtmlElement::Textarea => true,
                    KnownHtmlElement::Input => {
                        self.control_input_type(element)? != HtmlInputType::Hidden
                    }
                    _ => false,
                }
            } else {
                native_form_associated(kind)
            });
        }
        if !self.html_namespace(element)? {
            return Ok(false);
        }
        self.control_custom(subject)
    }
    fn control_input_type(
        &mut self,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlInputType, HtmlSemanticFailure> {
        let raw = self.attribute(element, "type")?;
        self.admit_intrinsic(element, raw, 23)?;
        self.interpreter
            .input_type(element.name(), raw)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }
    fn control_root(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> ControlResult<HtmlCaptureSubjectId> {
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            match self.parent(node)?.map_err(ControlStop::Missing)? {
                Some(parent) => current = parent,
                None => return Ok(current),
            }
        }
    }
    fn control_connected(&mut self, subject: &HtmlCaptureSubjectId) -> ControlResult<bool> {
        let mut current = subject.clone();
        loop {
            let root = self.control_root(&current)?;
            let node = self.node(&root)?;
            match node.kind() {
                HtmlCapturedNodeKind::Document(_) => return Ok(true),
                HtmlCapturedNodeKind::Fragment(fragment)
                    if matches!(fragment.kind(), HtmlCapturedFragmentKind::ShadowRoot(_)) =>
                {
                    self.charge(1)?;
                    match node.host() {
                        Some(HtmlCapturedHost::Element(host)) => current = host.subject().clone(),
                        _ => {
                            return Err(ControlStop::Missing(HtmlSemanticMissing::ShadowHost(
                                root,
                            )));
                        }
                    }
                }
                _ => return Ok(false),
            }
        }
    }
    fn control_nearest_form(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> ControlResult<Option<HtmlCaptureSubjectId>> {
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            let Some(parent) = self.parent(node)?.map_err(ControlStop::Missing)? else {
                return Ok(None);
            };
            let parent_node = self.node(&parent)?;
            if let HtmlCapturedNodeKind::Element(element) = parent_node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Form)
            {
                return Ok(Some(parent));
            }
            current = parent;
        }
    }
    fn control_children(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> ControlResult<HtmlCapturedChildIds<'a>> {
        let node = self.node(subject)?;
        self.charge(1)?;
        match node.children() {
            HtmlCapturedChildren::Complete(children) => Ok(children),
            HtmlCapturedChildren::Unavailable => Err(ControlStop::Missing(
                HtmlSemanticMissing::OrderedChildren(subject.clone()),
            )),
        }
    }
    fn control_equal(
        &mut self,
        left: &crate::DocumentString,
        right: &crate::DocumentString,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        for (left, right) in left.code_units().iter().zip(right.code_units()) {
            self.charge(2)?;
            if left != right {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn control_find(
        &mut self,
        root: &HtmlCaptureSubjectId,
        id: Option<&crate::DocumentString>,
        include_root: bool,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> ControlResult<Option<HtmlCaptureSubjectId>> {
        let mut current = if include_root {
            Some(root.clone())
        } else {
            None
        };
        if !include_root {
            let children = self.control_children(root)?;
            self.reserve_scratch(frames, 1)?;
            self.charge(1)?;
            frames.push(children);
        }
        loop {
            if let Some(subject) = current.take() {
                let node = self.node(&subject)?;
                if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                    if let Some(id) = id {
                        if let Some(attribute) = self.attribute(element, "id")?
                            && !attribute.value().is_empty()
                            && self.control_equal(attribute.value(), id)?
                        {
                            return Ok(Some(subject));
                        }
                    } else if self.control_category(&subject, element, true)? {
                        return Ok(Some(subject));
                    }
                }
                if matches!(
                    node.kind(),
                    HtmlCapturedNodeKind::Element(_)
                        | HtmlCapturedNodeKind::Document(_)
                        | HtmlCapturedNodeKind::Fragment(_)
                ) {
                    let children = self.control_children(&subject)?;
                    self.reserve_scratch(frames, 1)?;
                    self.charge(1)?;
                    frames.push(children);
                }
            }
            loop {
                self.charge(1)?;
                let Some(frame) = frames.last_mut() else {
                    return Ok(None);
                };
                if let Some(subject) = frame.next() {
                    current = Some(subject);
                    break;
                }
                frames.pop();
            }
        }
    }
    fn control_association(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> ControlResult<&'a HtmlFormAssociationState> {
        if let Some(facts) = self.context.controls() {
            for fact in facts.associations() {
                self.charge(1)?;
                if fact.subject() == subject {
                    return Ok(fact.state());
                }
            }
        }
        Err(ControlStop::Missing(HtmlSemanticMissing::FormAssociation(
            subject.clone(),
        )))
    }
    pub(super) fn form_owner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlFormOwner>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let mut scratch = Vec::new();
        let result = self.control_form_owner_inner(subject, element, &mut scratch);
        self.release_scratch(scratch.capacity());
        match result {
            Ok(Some(value)) => Ok(HtmlSemanticValue::Known(value)),
            Ok(None) => Ok(HtmlSemanticValue::Inapplicable),
            Err(ControlStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(ControlStop::Failure(failure)) => Err(failure),
        }
    }
    fn control_form_owner_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
        scratch: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> ControlResult<Option<HtmlFormOwner>> {
        if !self.control_category(subject, element, false)? {
            return Ok(None);
        }
        match self.control_association(subject)? {
            HtmlFormAssociationState::Unavailable => Err(ControlStop::Missing(
                HtmlSemanticMissing::FormAssociation(subject.clone()),
            )),
            HtmlFormAssociationState::EstablishedCurrent(owner) => {
                self.charge(1)?;
                Ok(Some(HtmlFormOwner {
                    owner: owner.clone(),
                    origin: HtmlFormOwnerOrigin::EstablishedCurrent,
                }))
            }
            HtmlFormAssociationState::ResetAgainstCapture => {
                let listed = self.known(element)? != Some(KnownHtmlElement::Img);
                let raw = if listed {
                    self.attribute(element, "form")?
                } else {
                    None
                };
                let owner = if let Some(raw) = raw {
                    self.interpreter
                        .admit_attribute_raw(Some(raw.value()))
                        .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                    if self.control_connected(subject)? {
                        let root = self.control_root(subject)?;
                        let candidate = if raw.value().is_empty() {
                            None
                        } else {
                            self.control_find(&root, Some(raw.value()), true, scratch)?
                        };
                        if let Some(candidate) = candidate {
                            let node = self.node(&candidate)?;
                            if let HtmlCapturedNodeKind::Element(e) = node.kind()
                                && self.known(e)? == Some(KnownHtmlElement::Form)
                            {
                                Some(candidate)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        self.control_nearest_form(subject)?
                    }
                } else {
                    self.control_nearest_form(subject)?
                };
                self.charge(1)?;
                Ok(Some(HtmlFormOwner {
                    owner,
                    origin: HtmlFormOwnerOrigin::ResetAgainstCapture,
                }))
            }
        }
    }
    pub(super) fn labeled_control(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Label) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let mut scratch = Vec::new();
        let result = self.control_label_inner(subject, element, &mut scratch);
        self.release_scratch(scratch.capacity());
        match result {
            Ok(value) => Ok(HtmlSemanticValue::Known(value)),
            Err(ControlStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(ControlStop::Failure(failure)) => Err(failure),
        }
    }
    fn control_label_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
        scratch: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> ControlResult<Option<HtmlCaptureSubjectId>> {
        if let Some(raw) = self.attribute(element, "for")? {
            self.interpreter
                .admit_attribute_raw(Some(raw.value()))
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
            if raw.value().is_empty() {
                return Ok(None);
            }
            let root = self.control_root(subject)?;
            if let Some(candidate) = self.control_find(&root, Some(raw.value()), true, scratch)? {
                let node = self.node(&candidate)?;
                if let HtmlCapturedNodeKind::Element(e) = node.kind()
                    && self.control_category(&candidate, e, true)?
                {
                    return Ok(Some(candidate));
                }
            }
            Ok(None)
        } else {
            self.control_find(subject, None, false, scratch)
        }
    }

    pub(super) fn control_labels(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlControlLabels>>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let mut frames = Vec::new();
        let result = self.control_labels_inner(subject, element, &mut frames);
        self.release_scratch(frames.capacity());
        match result {
            Ok(value) => Ok(value),
            Err(ControlStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(ControlStop::Failure(failure)) => Err(failure),
        }
    }

    fn control_labels_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> ControlResult<HtmlSemanticValue<Option<HtmlControlLabels>>> {
        if self.known(element)? == Some(KnownHtmlElement::Input) {
            if self.control_input_type(element)? == HtmlInputType::Hidden {
                return Ok(HtmlSemanticValue::Known(None));
            }
        } else if !self.control_category(subject, element, true)? {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let root = self.control_root(subject)?;
        let mut current = Some(root.clone());
        let mut labels = Vec::new();
        while let Some(candidate) = current {
            let node = self.node(&candidate)?;
            if let HtmlCapturedNodeKind::Element(label) = node.kind()
                && self.known(label)? == Some(KnownHtmlElement::Label)
                && self.control_label_matches(&candidate, label, subject, element, &root)?
            {
                self.reserve_nested_output(&mut labels, 1)?;
                self.charge(1)?;
                labels.push(candidate);
            }
            if matches!(
                node.kind(),
                HtmlCapturedNodeKind::Element(_)
                    | HtmlCapturedNodeKind::Document(_)
                    | HtmlCapturedNodeKind::Fragment(_)
            ) {
                let children = self.control_children(&node.id())?;
                self.reserve_scratch(frames, 1)?;
                self.charge(1)?;
                frames.push(children);
            }
            current = loop {
                self.charge(1)?;
                let Some(frame) = frames.last_mut() else {
                    break None;
                };
                if let Some(next) = frame.next() {
                    break Some(next);
                }
                frames.pop();
            };
        }
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(Some(HtmlControlLabels { labels })))
    }

    fn control_label_matches(
        &mut self,
        label: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
        target: &HtmlCaptureSubjectId,
        target_element: &'a HtmlCapturedElement,
        root: &HtmlCaptureSubjectId,
    ) -> ControlResult<bool> {
        let mut scratch = Vec::new();
        let result = (|| {
            if let Some(raw) = self.attribute(element, "for")? {
                let Some(id) = self.attribute(target_element, "id")? else {
                    return Ok(false);
                };
                self.charge(1)?;
                if id.value().is_empty() {
                    return Ok(false);
                }
                // A consulted for value keeps the same original-input admission
                // as the shared label resolver, before any comparison or lookup.
                self.interpreter
                    .admit_attribute_raw(Some(raw.value()))
                    .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
                if !self.control_equal(raw.value(), id.value())? {
                    return Ok(false);
                }
                let first = self.control_find(root, Some(raw.value()), true, &mut scratch)?;
                self.charge(1)?;
                if first.as_ref() != Some(target) {
                    return Ok(false);
                }
                // The first-ID proof excludes another unknown custom target
                // without interpreting that target's labelability. Reuse the
                // actual association owner for potentially matching labels.
                self.charge(scratch.len())?;
                scratch.clear();
            } else {
                let mut ancestor = target.clone();
                loop {
                    let node = self.node(&ancestor)?;
                    let Some(parent) = self.parent(node)?.map_err(ControlStop::Missing)? else {
                        return Ok(false);
                    };
                    self.charge(1)?;
                    if &parent == label {
                        break;
                    }
                    ancestor = parent;
                }
            }
            let associated = self.control_label_inner(label, element, &mut scratch)?;
            self.charge(1)?;
            Ok(associated.as_ref() == Some(target))
        })();
        self.release_scratch(scratch.capacity());
        result
    }

    pub(super) fn control_flag(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        flag: HtmlControlFlag,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let kind = self.known(element)?;
        let applicable = match flag {
            HtmlControlFlag::Checkedness
            | HtmlControlFlag::DirtyCheckedness
            | HtmlControlFlag::Indeterminateness => kind == Some(KnownHtmlElement::Input),
            HtmlControlFlag::Selectedness | HtmlControlFlag::OptionDirtiness => {
                kind == Some(KnownHtmlElement::Option)
            }
            HtmlControlFlag::DirtyValue => matches!(
                kind,
                Some(KnownHtmlElement::Input | KnownHtmlElement::Textarea)
            ),
        };
        if !applicable {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let mut value = None;
        if let Some(facts) = self.context.controls() {
            match kind {
                Some(KnownHtmlElement::Input) => {
                    for fact in facts.inputs() {
                        self.charge(1)?;
                        if fact.subject() == subject {
                            value = match flag {
                                HtmlControlFlag::Checkedness => fact.checkedness(),
                                HtmlControlFlag::DirtyCheckedness => fact.dirty_checkedness(),
                                HtmlControlFlag::DirtyValue => fact.dirty_value(),
                                HtmlControlFlag::Indeterminateness => fact.indeterminateness(),
                                _ => None,
                            };
                            break;
                        }
                    }
                }
                Some(KnownHtmlElement::Option) => {
                    for fact in facts.options() {
                        self.charge(1)?;
                        if fact.subject() == subject {
                            value = if flag == HtmlControlFlag::Selectedness {
                                fact.selectedness()
                            } else {
                                fact.dirtiness()
                            };
                            break;
                        }
                    }
                }
                Some(KnownHtmlElement::Textarea) => {
                    for fact in facts.textareas() {
                        self.charge(1)?;
                        if fact.subject() == subject {
                            value = fact.dirty_value();
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
        self.charge(1)?;
        Ok(match value {
            Some(value) => HtmlSemanticValue::Known(value),
            None => HtmlSemanticValue::Missing(HtmlSemanticMissing::ControlState {
                subject: subject.clone(),
                flag,
            }),
        })
    }
}

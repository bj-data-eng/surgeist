//! Reflected Element associations and static command support, HTML 877d6146487f.
//! These readbacks execute no activation, events, form transition or target effect.

use super::control::ControlStop;
use super::*;
use crate::{
    HtmlAttributeReport, HtmlCapturedFragmentKind, HtmlCapturedHost, HtmlCommandReadback,
    HtmlCommandState, HtmlInputType, HtmlPopoverState, HtmlStandardCommand,
};

/// The two concrete reflected Element slots consumed by this owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlElementReferenceKey {
    CommandFor,
    PopoverTarget,
}
impl HtmlElementReferenceKey {
    fn attribute(self) -> &'static str {
        match self {
            Self::CommandFor => "commandfor",
            Self::PopoverTarget => "popovertarget",
        }
    }
    fn applicable(self, kind: Option<KnownHtmlElement>) -> bool {
        kind == Some(KnownHtmlElement::Button)
            || (self == Self::PopoverTarget && kind == Some(KnownHtmlElement::Input))
    }
}
/// Actual weak-pointer slot, not an inference from the content attribute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlExplicitElementReference {
    Unavailable,
    Null,
    Element(HtmlCaptureSubjectId),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlElementReferenceFact {
    subject: HtmlCaptureSubjectId,
    key: HtmlElementReferenceKey,
    state: HtmlExplicitElementReference,
}
impl HtmlElementReferenceFact {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        key: HtmlElementReferenceKey,
        state: HtmlExplicitElementReference,
    ) -> Self {
        Self {
            subject,
            key,
            state,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn key(&self) -> HtmlElementReferenceKey {
        self.key
    }
    pub fn state(&self) -> &HtmlExplicitElementReference {
        &self.state
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlElementReferenceFactsLimits {
    records: usize,
    validation_work: usize,
}
impl HtmlElementReferenceFactsLimits {
    /// Moved vector capacity and finite validation work; caller allocation is
    /// outside these limits. No allocator-header, RSS or cancellation guarantee.
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
pub enum HtmlElementReferenceFactsError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    RecordCapacity {
        limit: usize,
        required: usize,
    },
    ValidationWork {
        limit: usize,
        required: usize,
    },
    RepresentationExhausted,
    DuplicateReference {
        subject: HtmlCaptureSubjectId,
        key: HtmlElementReferenceKey,
    },
    InapplicableSubject(HtmlCaptureSubjectId),
    InvalidTarget(HtmlCaptureSubjectId),
}
impl fmt::Display for HtmlElementReferenceFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlElementReferenceFactsError {}
/// Checked capture/host-qualified actual pointer records. Missing records remain
/// unavailable. Valid out-of-scope pointers are retained for HTML to interpret.
#[derive(Debug)]
pub struct HtmlElementReferenceFacts {
    capture: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    records: Vec<HtmlElementReferenceFact>,
}
impl HtmlElementReferenceFacts {
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        records: Vec<HtmlElementReferenceFact>,
        limits: HtmlElementReferenceFactsLimits,
    ) -> Result<Self, HtmlElementReferenceFactsError> {
        if &host_revision != capture.host_facts_revision() {
            return Err(HtmlElementReferenceFactsError::HostRevisionMismatch);
        }
        if records.capacity() > limits.records {
            return Err(HtmlElementReferenceFactsError::RecordCapacity {
                limit: limits.records,
                required: records.capacity(),
            });
        }
        let mut work = 0usize;
        // This is input validation, not a query/controller or a tree projection.
        let mut charge = |amount: usize| {
            let required = work
                .checked_add(amount)
                .ok_or(HtmlElementReferenceFactsError::RepresentationExhausted)?;
            if required > limits.validation_work {
                return Err(HtmlElementReferenceFactsError::ValidationWork {
                    limit: limits.validation_work,
                    required,
                });
            }
            work = required;
            Ok(())
        };
        for (index, fact) in records.iter().enumerate() {
            charge(2)?; // record and source lookup
            let node = capture
                .node(&fact.subject)
                .map_err(HtmlElementReferenceFactsError::Capture)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlElementReferenceFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            };
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
                .ok_or(HtmlElementReferenceFactsError::RepresentationExhausted)?;
            charge(envelope)?;
            if !fact
                .key
                .applicable(KnownHtmlElement::from_name(element.name()))
            {
                return Err(HtmlElementReferenceFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            }
            for previous in &records[..index] {
                charge(1)?;
                if previous.subject == fact.subject && previous.key == fact.key {
                    return Err(HtmlElementReferenceFactsError::DuplicateReference {
                        subject: fact.subject.clone(),
                        key: fact.key,
                    });
                }
            }
            if let HtmlExplicitElementReference::Element(target) = &fact.state {
                charge(1)?;
                let target_node = capture
                    .node(target)
                    .map_err(HtmlElementReferenceFactsError::Capture)?;
                if !matches!(target_node.kind(), HtmlCapturedNodeKind::Element(_)) {
                    return Err(HtmlElementReferenceFactsError::InvalidTarget(
                        target.clone(),
                    ));
                }
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host_revision,
            records,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn records(&self) -> &[HtmlElementReferenceFact] {
        &self.records
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlElementReferenceOrigin {
    ExplicitPointer,
    /// Null-pointer branch, including missing or empty content and no match.
    ContentAttribute,
}
/// Fixed retained observation of the generic reflected Element algorithm.
/// Raw authorship and same-tree author-reference conformance are not inferred.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAssociatedElement {
    raw: Option<Attribute>,
    state: HtmlExplicitElementReference,
    target: Option<HtmlCaptureSubjectId>,
    origin: HtmlElementReferenceOrigin,
}
impl HtmlAssociatedElement {
    pub fn raw(&self) -> Option<&Attribute> {
        self.raw.as_ref()
    }
    pub fn explicit_reference(&self) -> &HtmlExplicitElementReference {
        &self.state
    }
    pub fn target(&self) -> Option<&HtmlCaptureSubjectId> {
        self.target.as_ref()
    }
    pub fn origin(&self) -> HtmlElementReferenceOrigin {
        self.origin
    }
}
/// Source support only. None means the association is known null; it is not
/// false validity. Author-invalid Custom still has its processing support.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCommandValidity {
    association: HtmlAssociatedElement,
    command: HtmlAttributeReport<HtmlCommandReadback>,
    valid: Option<bool>,
}
impl HtmlCommandValidity {
    pub fn association(&self) -> &HtmlAssociatedElement {
        &self.association
    }
    pub fn command(&self) -> &HtmlAttributeReport<HtmlCommandReadback> {
        &self.command
    }
    pub fn valid(&self) -> Option<bool> {
        self.valid
    }
}

impl<'a> HtmlSemanticSession<'a> {
    fn explicit_reference(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        key: HtmlElementReferenceKey,
    ) -> Result<Result<&'a HtmlExplicitElementReference, HtmlSemanticMissing>, HtmlSemanticFailure>
    {
        if let Some(facts) = self.context.element_references() {
            for fact in facts.records() {
                self.charge(1)?;
                if fact.subject() == subject
                    && fact.key() == key
                    && fact.state() != &HtmlExplicitElementReference::Unavailable
                {
                    return Ok(Ok(fact.state()));
                }
            }
        }
        Ok(Err(HtmlSemanticMissing::ElementReference {
            subject: subject.clone(),
            key,
        }))
    }
    // Only shadow-root host edges extend the source ancestor chain. Ordinary
    // target ancestors below never cross hosts or template contents.
    fn reference_shadow_parent(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Result<Option<HtmlCaptureSubjectId>, HtmlSemanticMissing>, HtmlSemanticFailure>
    {
        let node = self.node(subject)?;
        let parent = self.parent(node)?;
        if parent != Ok(None) {
            return Ok(parent);
        }
        if let HtmlCapturedNodeKind::Fragment(fragment) = node.kind()
            && matches!(fragment.kind(), HtmlCapturedFragmentKind::ShadowRoot(_))
        {
            self.charge(1)?;
            return Ok(match node.host() {
                Some(HtmlCapturedHost::Element(host)) => Ok(Some(host.subject().clone())),
                _ => Err(HtmlSemanticMissing::ShadowHost(subject.clone())),
            });
        }
        Ok(Ok(None))
    }
    fn reference_scope(
        &mut self,
        source: &HtmlCaptureSubjectId,
        target: &HtmlCaptureSubjectId,
    ) -> Result<Result<bool, HtmlSemanticMissing>, HtmlSemanticFailure> {
        let mut source_current = source.clone();
        let mut first_missing = None;
        loop {
            let ancestor = match self.reference_shadow_parent(&source_current)? {
                Ok(Some(ancestor)) => ancestor,
                Ok(None) => break,
                Err(missing) => {
                    first_missing.get_or_insert(missing);
                    // A target with no ordinary ancestors cannot satisfy the
                    // strict descendant relation, even with unknown source data.
                    let target_node = self.node(target)?;
                    if self.parent(target_node)? == Ok(None) {
                        return Ok(Ok(false));
                    }
                    break;
                }
            };
            let mut target_current = target.clone();
            let mut saw_target_ancestor = false;
            loop {
                let node = self.node(&target_current)?;
                match self.parent(node)? {
                    Ok(Some(parent)) => {
                        saw_target_ancestor = true;
                        self.charge(1)?;
                        if parent == ancestor {
                            return Ok(Ok(true));
                        }
                        target_current = parent;
                    }
                    Ok(None) => {
                        if !saw_target_ancestor {
                            return Ok(Ok(false));
                        }
                        break;
                    }
                    Err(missing) => {
                        first_missing.get_or_insert(missing);
                        break;
                    }
                }
            }
            source_current = ancestor;
        }
        Ok(match first_missing {
            Some(missing) => Err(missing),
            None => Ok(false),
        })
    }
    pub(super) fn associated_element(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        key: HtmlElementReferenceKey,
    ) -> Result<HtmlSemanticValue<HtmlAssociatedElement>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !key.applicable(self.known(element)?) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, key.attribute())?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let state = match self.explicit_reference(subject, key)? {
            Ok(state) => state,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        let (target, origin) = match state {
            HtmlExplicitElementReference::Unavailable => {
                unreachable!("checked actual state lookup")
            }
            HtmlExplicitElementReference::Element(target) => {
                let scoped = match self.reference_scope(subject, target)? {
                    Ok(v) => v,
                    Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                };
                (
                    scoped.then(|| target.clone()),
                    HtmlElementReferenceOrigin::ExplicitPointer,
                )
            }
            HtmlExplicitElementReference::Null => {
                let mut frames = Vec::new();
                let outcome = (|| {
                    if raw.is_none_or(|a| a.value().is_empty()) {
                        return Ok(Ok(None));
                    }
                    let root = match self.ordinary_tree_root(subject)? {
                        Ok(root) => root,
                        Err(missing) => return Ok(Err(missing)),
                    };
                    match self.control_find(&root, raw.map(Attribute::value), true, &mut frames) {
                        Ok(target) => Ok(Ok(target)),
                        Err(ControlStop::Missing(missing)) => Ok(Err(missing)),
                        Err(ControlStop::Failure(failure)) => Err(failure),
                    }
                })();
                self.release_scratch(frames.capacity());
                let target = match outcome? {
                    Ok(target) => target,
                    Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                };
                (target, HtmlElementReferenceOrigin::ContentAttribute)
            }
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlAssociatedElement {
            raw: raw.cloned(),
            state: state.clone(),
            target,
            origin,
        }))
    }
    pub(super) fn command_validity(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlCommandValidity>, HtmlSemanticFailure> {
        let association = match self
            .associated_element(subject, HtmlElementReferenceKey::CommandFor)?
        {
            HtmlSemanticValue::Known(value) => value,
            HtmlSemanticValue::Missing(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            HtmlSemanticValue::Inapplicable => return Ok(HtmlSemanticValue::Inapplicable),
            HtmlSemanticValue::Unsupported(unsupported) => {
                return Ok(HtmlSemanticValue::Unsupported(unsupported));
            }
        };
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
        };
        let raw = self.attribute(element, "command")?;
        self.admit_intrinsic(element, raw, 6)?;
        let command = self
            .interpreter
            .command(element.name(), raw)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        self.poll()?;
        let state = command
            .intrinsic()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?
            .state();
        let valid = if let Some(target) = association.target() {
            Some(self.command_target_support(target, state)?)
        } else {
            None
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlCommandValidity {
            association,
            command,
            valid,
        }))
    }
    fn command_target_support(
        &mut self,
        target: &HtmlCaptureSubjectId,
        state: HtmlCommandState,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        match state {
            HtmlCommandState::Unknown => Ok(false),
            HtmlCommandState::Custom => Ok(true),
            HtmlCommandState::Standard(command) => {
                let node = self.node(target)?;
                let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                    return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
                };
                if !self.html_namespace(element)? {
                    return Ok(false);
                }
                if matches!(
                    command,
                    HtmlStandardCommand::TogglePopover
                        | HtmlStandardCommand::ShowPopover
                        | HtmlStandardCommand::HidePopover
                ) {
                    return Ok(true);
                }
                Ok(self.known(element)? == Some(KnownHtmlElement::Dialog))
            }
        }
    }
    pub(super) fn popover_target_element(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Known(None));
        };
        let kind = self.known(element)?;
        if !HtmlElementReferenceKey::PopoverTarget.applicable(kind) {
            return Ok(HtmlSemanticValue::Known(None));
        }
        if kind == Some(KnownHtmlElement::Input) {
            let raw = self.attribute(element, "type")?;
            self.admit_intrinsic(element, raw, 22)?;
            let state = self
                .interpreter
                .input_type(element.name(), raw)
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?
                .intrinsic()
                .copied()
                .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
            self.poll()?;
            if !matches!(
                state,
                HtmlInputType::Submit
                    | HtmlInputType::Image
                    | HtmlInputType::Reset
                    | HtmlInputType::Button
            ) {
                return Ok(HtmlSemanticValue::Known(None));
            }
        }
        match self.disabledness(subject, element)? {
            HtmlSemanticValue::Known(true) => return Ok(HtmlSemanticValue::Known(None)),
            HtmlSemanticValue::Known(false) => {}
            other => return Ok(other.map(|_| None)),
        }
        let owner = self.form_owner(subject)?;
        if !matches!(&owner, HtmlSemanticValue::Known(owner) if owner.owner().is_none()) {
            let submit = self
                .form_submitter_role(subject, element, kind)?
                .submit_button;
            match submit {
                Ok(false) => {}
                Ok(true) => match owner {
                    HtmlSemanticValue::Known(_) => return Ok(HtmlSemanticValue::Known(None)),
                    other => return Ok(other.map(|_| None)),
                },
                Err(missing) => match owner {
                    HtmlSemanticValue::Known(_) => return Ok(HtmlSemanticValue::Missing(missing)),
                    other => return Ok(other.map(|_| None)),
                },
            }
        }
        let association =
            match self.associated_element(subject, HtmlElementReferenceKey::PopoverTarget)? {
                HtmlSemanticValue::Known(value) => value,
                other => return Ok(other.map(|_| None)),
            };
        let Some(target) = association.target else {
            return Ok(HtmlSemanticValue::Known(None));
        };
        let node = self.node(&target)?;
        let HtmlCapturedNodeKind::Element(target_element) = node.kind() else {
            return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
        };
        // Source-qualified disposition #1134 comment6117820267: keep generic
        // association intact; foreign guarded targets are null before popover.
        if !self.html_namespace(target_element)? {
            return Ok(HtmlSemanticValue::Known(None));
        }
        let raw = self.attribute(target_element, "popover")?;
        self.admit_intrinsic(target_element, raw, 3)?;
        let state = self
            .interpreter
            .popover(target_element.name(), raw)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        self.poll()?;
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(
            (state != HtmlPopoverState::NoPopover).then_some(target),
        ))
    }
}

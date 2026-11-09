//! Reflected Element associations and static command support, HTML 877d6146487f.
//! These readbacks execute no activation, events, form transition or target effect.

use super::control::ControlStop;
use super::index::FactIndex;
use super::scan::{Caps, ScanError, ScanMeter};
use super::*;
use crate::{HtmlCapturedFragmentKind, HtmlCapturedHost};
use surgeist_html::accounting::HtmlAccountingFault;
use surgeist_html::accounting::{HtmlCapScope, HtmlResource};
use surgeist_html::attributes::{
    HtmlAttributeReport, HtmlInputType, HtmlPopoverState,
    command::{HtmlCommandReadback, HtmlCommandState},
};
use surgeist_html::{values::Attribute, vocabulary::KnownHtmlElement};

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
    fn is_popover_target(self) -> bool {
        self == Self::PopoverTarget
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
    /// The ledger that carries the accounting failed in a way that is not a refused limit: a release
    /// that matches nothing held, a ledger that cannot allocate, a scope that is not the
    /// operation's, or a change the operation does not charge. It carries the accounting fault.
    Accounting(HtmlAccountingFault),
    DuplicateReference {
        subject: HtmlCaptureSubjectId,
        key: HtmlElementReferenceKey,
    },
    InapplicableSubject(HtmlCaptureSubjectId),
    InvalidTarget(HtmlCaptureSubjectId),
}
impl HtmlElementReferenceFactsError {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Accounting(fault) => Some(*fault),
            Self::Capture(error) => error.accounting_fault(),
            Self::HostRevisionMismatch
            | Self::RecordCapacity { .. }
            | Self::ValidationWork { .. }
            | Self::RepresentationExhausted
            | Self::DuplicateReference { .. }
            | Self::InapplicableSubject(_)
            | Self::InvalidTarget(_) => None,
        }
    }
}
impl ScanError for HtmlElementReferenceFactsError {
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
    /// Records by subject and whether the key is `PopoverTarget`.
    index: FactIndex<bool>,
}
impl HtmlElementReferenceFacts {
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        records: Vec<HtmlElementReferenceFact>,
        limits: HtmlElementReferenceFactsLimits,
    ) -> Result<Self, HtmlElementReferenceFactsError> {
        Self::build(capture, host_revision, records, limits, None)
    }
    /// Charges below the scope `accounting` names, in the host's operation, as [`Self::new`]
    /// charges an operation of its own. The schema module documentation of
    /// `surgeist_html::accounting` lists the scopes it creates and what the host keeps.
    pub fn with_accounting(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        records: Vec<HtmlElementReferenceFact>,
        limits: HtmlElementReferenceFactsLimits,
        accounting: &HtmlHostScope,
    ) -> Result<Self, HtmlElementReferenceFactsError> {
        Self::build(capture, host_revision, records, limits, Some(accounting))
    }
    fn build(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        records: Vec<HtmlElementReferenceFact>,
        limits: HtmlElementReferenceFactsLimits,
        hosted: Option<&HtmlHostScope>,
    ) -> Result<Self, HtmlElementReferenceFactsError> {
        if &host_revision != capture.host_facts_revision() {
            return Err(HtmlElementReferenceFactsError::HostRevisionMismatch);
        }
        // This is input validation, not a query/controller or a tree projection.
        let mut meter = ScanMeter::charging(
            HtmlCapScope::ElementReferenceFacts,
            Caps::work(limits.validation_work).records(limits.records),
            hosted,
        )?;
        meter.acquire(
            HtmlResource::FactRecords,
            records.capacity(),
            |limit, required| HtmlElementReferenceFactsError::RecordCapacity { limit, required },
        )?;
        let mut index = FactIndex::new(
            capture.identity(),
            records.len(),
            |n| meter.charge(n),
            HtmlElementReferenceFactsError::RepresentationExhausted,
        )?;
        for (position, fact) in records.iter().enumerate() {
            index.enter_bound(&fact.subject, fact.key.is_popover_target(), position);
        }
        for (position, fact) in records.iter().enumerate() {
            meter.charge(2)?; // record and source lookup
            let node = capture
                .node(&fact.subject)
                .map_err(HtmlElementReferenceFactsError::Capture)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(HtmlElementReferenceFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            };
            if !fact.key.applicable(meter.known(element)?) {
                return Err(HtmlElementReferenceFactsError::InapplicableSubject(
                    fact.subject.clone(),
                ));
            }
            if index.bound_position(&fact.subject, fact.key.is_popover_target()) != Some(position) {
                return Err(HtmlElementReferenceFactsError::DuplicateReference {
                    subject: fact.subject.clone(),
                    key: fact.key,
                });
            }
            if let HtmlExplicitElementReference::Element(target) = &fact.state {
                let target_node = meter.node(capture, target)?;
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
            index,
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

/// An explicit reference whose slot was actually observed, so it is never `Unavailable`.
enum ActualReference<'a> {
    Null,
    Element(&'a HtmlCaptureSubjectId),
}
impl ActualReference<'_> {
    fn state(&self) -> HtmlExplicitElementReference {
        match self {
            Self::Null => HtmlExplicitElementReference::Null,
            Self::Element(target) => HtmlExplicitElementReference::Element((*target).clone()),
        }
    }
}
impl<'a> HtmlSemanticSession<'a> {
    fn explicit_reference(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        key: HtmlElementReferenceKey,
    ) -> Result<Result<ActualReference<'a>, HtmlSemanticMissing>, HtmlSemanticFailure> {
        if let Some(facts) = self.context.element_references() {
            self.charge(1)?;
            if let Some(position) = facts.index.bound_position(subject, key.is_popover_target()) {
                match facts.records[position].state() {
                    HtmlExplicitElementReference::Unavailable => {}
                    HtmlExplicitElementReference::Null => {
                        return Ok(Ok(ActualReference::Null));
                    }
                    HtmlExplicitElementReference::Element(target) => {
                        return Ok(Ok(ActualReference::Element(target)));
                    }
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
    /// Whether `target` is a descendant of an ancestor of `source`, going from
    /// `source` to its ordinary parent and, at a shadow root, to its host. The
    /// ancestors of the target are collected once and sorted, so each ancestor of
    /// the source is found among them by search: the work is that of the two
    /// chains, not of every pair of their elements.
    ///
    /// A source without ancestors has no scope. A target without ancestors is no
    /// strict descendant of any. Unavailable ancestry on the target's side, which
    /// ends what can be known of its ancestors, is reported before unavailable
    /// ancestry on the source's side, as the first the walk meets.
    fn reference_scope(
        &mut self,
        source: &HtmlCaptureSubjectId,
        target: &HtmlCaptureSubjectId,
    ) -> Result<Result<bool, HtmlSemanticMissing>, HtmlSemanticFailure> {
        let first = match self.reference_shadow_parent(source)? {
            Ok(Some(ancestor)) => ancestor,
            Ok(None) => return Ok(Ok(false)),
            Err(missing) => {
                // A target with no ordinary ancestors cannot satisfy the strict
                // descendant relation, even with unknown source data.
                let target_node = self.node(target)?;
                if self.parent(target_node)? == Ok(None) {
                    return Ok(Ok(false));
                }
                return Ok(Err(missing));
            }
        };
        let mut ancestors = Vec::new();
        let result = self.reference_scope_among(first, target, &mut ancestors);
        self.release_scratch(&ancestors);
        result
    }
    fn reference_scope_among(
        &mut self,
        first: HtmlCaptureSubjectId,
        target: &HtmlCaptureSubjectId,
        ancestors: &mut Vec<usize>,
    ) -> Result<Result<bool, HtmlSemanticMissing>, HtmlSemanticFailure> {
        let mut target_missing = None;
        let mut current = target.clone();
        loop {
            let node = self.node(&current)?;
            match self.parent(node)? {
                Ok(Some(parent)) => {
                    self.reserve_scratch(ancestors, 1)?;
                    self.charge(1)?;
                    ancestors.push(parent.ordinal());
                    current = parent;
                }
                Ok(None) => {
                    if ancestors.is_empty() {
                        return Ok(Ok(false));
                    }
                    break;
                }
                Err(missing) => {
                    target_missing = Some(missing);
                    break;
                }
            }
        }
        let bits = (usize::BITS - ancestors.len().leading_zeros()) as usize;
        let sort = ancestors
            .len()
            .checked_mul(bits)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(sort)?;
        ancestors.sort_unstable();
        let mut source_missing = None;
        let mut ancestor = first;
        loop {
            self.charge(bits)?;
            if ancestors.binary_search(&ancestor.ordinal()).is_ok() {
                return Ok(Ok(true));
            }
            match self.reference_shadow_parent(&ancestor)? {
                Ok(Some(next)) => ancestor = next,
                Ok(None) => break,
                Err(missing) => {
                    source_missing = Some(missing);
                    break;
                }
            }
        }
        Ok(match target_missing.or(source_missing) {
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
            ActualReference::Element(target) => {
                let scoped = match self.reference_scope(subject, target)? {
                    Ok(v) => v,
                    Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                };
                (
                    scoped.then(|| target.clone()),
                    HtmlElementReferenceOrigin::ExplicitPointer,
                )
            }
            ActualReference::Null => {
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
                self.release_scratch(&frames);
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
            state: state.state(),
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
                if command.is_valid_for_every_html_element() {
                    return Ok(true);
                }
                Ok(command.is_valid_by_element_steps(self.known(element)?))
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
            let state = self
                .admitted_input_type_report(element, raw)?
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

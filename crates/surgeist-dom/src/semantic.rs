//! Direct canonical-snapshot adaptation into the HTML semantic owner.

mod control;
mod slot;
pub use slot::DomSlotSemanticLimits;

use std::{
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

use surgeist_html::{
    HtmlBrowsingContextPresence, HtmlDoctype, HtmlHeaderDeliveredCsp, HtmlNonceDocumentEnvironment,
    HtmlValueInterpreter,
};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureDefinitionInput,
    HtmlCaptureError, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCaptureSubjectId,
    HtmlCaptureTemplateCacheInput, HtmlCapturedCustomElementState, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlDocumentClass, HtmlHostFactsRevision, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextError,
    HtmlSemanticContextInput, HtmlSemanticFailure, HtmlSemanticLimits, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticSession, HtmlTemplateDocumentRole,
};

use crate::{
    DocumentId, DomDocumentClass, DomDocumentStamp, DomElementDefinition, DomError, DomHostCapture,
    DomHostError, DomNodeKind, DomRegistryBinding, DomSnapshot, NodeId,
};

/// Adapter-owned construction storage/work, separate from producer capture retention limits.
/// `work` independently bounds adapter work and shared control-fact validation
/// including its context join. Each closure/mapping/staging vector and the reusable child scratch have at most
/// `subjects` entries. Nullable reference facts have at most twice `subjects`
/// entries (two per Button), with their validation independently bounded by
/// `work`. Actual HTML/null-definition classifications also use subjects, with
/// their context qualification/duplicate comparisons charged to adapter work.
/// Producer allocations obey the supplied HtmlCaptureLimits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHtmlCaptureLimits {
    subjects: usize,
    work: usize,
}
impl DomHtmlCaptureLimits {
    pub fn new(subjects: usize, work: usize) -> Self {
        Self { subjects, work }
    }
    pub fn subjects(self) -> usize {
        self.subjects
    }
    pub fn work(self) -> usize {
        self.work
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHtmlCaptureResource {
    Subjects,
    Work,
}

/// A refused read-only composition publishes no partial capture.
#[derive(Debug)]
pub enum DomHtmlCaptureError {
    Source(DomError),
    Host(DomHostError),
    Capture(HtmlCaptureError),
    Context(HtmlSemanticContextError),
    Semantic(HtmlSemanticFailure),
    Language(surgeist_html_semantics::HtmlLanguageDirectionError),
    Inert(surgeist_html_semantics::HtmlInertFactError),
    Controls(surgeist_html_semantics::HtmlControlFactsError),
    ElementReferences(surgeist_html_semantics::HtmlElementReferenceFactsError),
    ResourceLimit {
        resource: DomHtmlCaptureResource,
        limit: usize,
    },
    Cancelled,
    AllocationFailed,
    RepresentationExhausted,
    UnmappedSubject,
}
impl fmt::Display for DomHtmlCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DOM HTML capture rejected: {self:?}")
    }
}
impl std::error::Error for DomHtmlCaptureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(e) => Some(e),
            Self::Host(e) => Some(e),
            Self::Capture(e) => Some(e),
            Self::Context(e) => Some(e),
            Self::Semantic(e) => Some(e),
            Self::Language(e) => Some(e),
            Self::Inert(e) => Some(e),
            Self::Controls(e) => Some(e),
            Self::ElementReferences(e) => Some(e),
            Self::ResourceLimit { .. }
            | Self::Cancelled
            | Self::AllocationFailed
            | Self::RepresentationExhausted
            | Self::UnmappedSubject => None,
        }
    }
}
impl From<DomError> for DomHtmlCaptureError {
    fn from(value: DomError) -> Self {
        Self::Source(value)
    }
}
impl From<HtmlCaptureError> for DomHtmlCaptureError {
    fn from(value: HtmlCaptureError) -> Self {
        Self::Capture(value)
    }
}

#[derive(Debug)]
struct Entry {
    node: NodeId,
    subject: HtmlCaptureSubjectId,
}
struct StagedEntry {
    node: NodeId,
    subject: HtmlCaptureBuilderSubject,
}
#[derive(Clone, Copy)]
struct PlannedEntry {
    node: NodeId,
    complete_children: bool,
    expanded: bool,
}

/// One exact canonical document, its optional host capture, and HTML's immutable
/// interpretation input. Detached nodes belong to the requested document closure;
/// actual shadow associations can add members of other captured documents.
/// Missing external facts remain missing; focus/hover do not establish other
/// HTML domains. Retaining this value retains the original snapshot and map.
#[derive(Debug)]
pub struct DomHtmlCapture {
    source: DomSnapshot,
    basis: DomDocumentStamp,
    host: Option<DomHostCapture>,
    capture: HtmlCapture,
    context: HtmlSemanticContext,
    entries: Vec<Entry>,
}
impl DomHtmlCapture {
    pub fn basis(&self) -> DomDocumentStamp {
        self.basis
    }
    /// The actual immutable canonical state, including common slots which the
    /// HTML capture schema does not duplicate or expose as semantic facts.
    pub fn source(&self) -> &DomSnapshot {
        &self.source
    }
    pub fn host(&self) -> Option<&DomHostCapture> {
        self.host.as_ref()
    }
    pub fn capture(&self) -> &HtmlCapture {
        &self.capture
    }
    pub fn subject(&self, node: NodeId) -> Result<HtmlCaptureSubjectId, DomHtmlCaptureError> {
        self.source.node(node)?;
        let mapped = self
            .entries
            .iter()
            .find(|e| e.node == node)
            .map(|e| e.subject.clone())
            .ok_or(DomHtmlCaptureError::UnmappedSubject);
        // Preserve the ordinary wrong-document diagnostic for unmapped nodes,
        // while allowing actual supplemental association members.
        if mapped.is_err() {
            self.source.document(self.basis.document())?.node(node)?;
        }
        mapped
    }
    pub fn node(&self, subject: &HtmlCaptureSubjectId) -> Result<NodeId, DomHtmlCaptureError> {
        self.capture.node(subject)?;
        self.entries
            .iter()
            .find(|e| &e.subject == subject)
            .map(|e| e.node)
            .ok_or(DomHtmlCaptureError::UnmappedSubject)
    }
    /// Executes the real HTML semantic session over this exact composition.
    /// Reports retain the producer's Missing/Inapplicable/Unsupported outcomes
    /// and conservative dependency contract; no default host facts are supplied.
    pub fn query(
        &self,
        requests: &[HtmlSemanticRequest],
        interpreter: HtmlValueInterpreter,
        limits: HtmlSemanticLimits,
        cancellation: HtmlSemanticCancellation,
    ) -> Result<HtmlSemanticReport, DomHtmlCaptureError> {
        HtmlSemanticSession::new(
            &self.capture,
            &self.context,
            interpreter,
            limits,
            cancellation,
        )
        .map_err(DomHtmlCaptureError::Semantic)?
        .query(requests)
        .map_err(DomHtmlCaptureError::Semantic)
    }
}

struct Control<'a> {
    limits: DomHtmlCaptureLimits,
    work: usize,
    cancellation: &'a AtomicBool,
}
impl Control<'_> {
    fn charge(&mut self, work: usize) -> Result<(), DomHtmlCaptureError> {
        if self.cancellation.load(Ordering::Acquire) {
            return Err(DomHtmlCaptureError::Cancelled);
        }
        let next = self
            .work
            .checked_add(work)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        if next > self.limits.work {
            return Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Work,
                limit: self.limits.work,
            });
        }
        self.work = next;
        Ok(())
    }
    fn reserve<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), DomHtmlCaptureError> {
        self.reserve_extent(values, additional, self.limits.subjects)
    }
    fn reserve_extent<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
        extent: usize,
    ) -> Result<(), DomHtmlCaptureError> {
        self.charge(1)?;
        let count = values
            .len()
            .checked_add(additional)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        if count > extent {
            return Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Subjects,
                limit: extent,
            });
        }
        count
            .checked_mul(std::mem::size_of::<T>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        values
            .try_reserve_exact(additional)
            .map_err(|_| DomHtmlCaptureError::AllocationFailed)
    }
    fn subject(
        &mut self,
        entries: &[StagedEntry],
        node: NodeId,
    ) -> Result<HtmlCaptureBuilderSubject, DomHtmlCaptureError> {
        for entry in entries {
            self.charge(1)?;
            if entry.node == node {
                return Ok(entry.subject.clone());
            }
        }
        Err(DomHtmlCaptureError::UnmappedSubject)
    }
    fn include(
        &mut self,
        entries: &mut Vec<PlannedEntry>,
        node: NodeId,
        complete_children: bool,
    ) -> Result<(), DomHtmlCaptureError> {
        for entry in entries.iter_mut() {
            self.charge(1)?;
            if entry.node == node {
                if complete_children && !entry.complete_children {
                    entry.complete_children = true;
                    entry.expanded = false;
                }
                return Ok(());
            }
        }
        self.reserve(entries, 1)?;
        entries.push(PlannedEntry {
            node,
            complete_children,
            expanded: false,
        });
        Ok(())
    }
}

fn add_document(
    builder: &mut HtmlCaptureBuilder,
    snapshot: &DomSnapshot,
    document: DocumentId,
) -> Result<HtmlCaptureBuilderSubject, DomHtmlCaptureError> {
    let metadata = snapshot.document(document)?.metadata();
    let environment = metadata.native_environment().browsing_context();
    // HTML877d #concept-n-noscript: null browsing context independently
    // disables scripting for Nodes. Present alone would not prove Enabled.
    let scripting = match environment {
        HtmlBrowsingContextPresence::Absent => HtmlScriptingStatus::Disabled,
        HtmlBrowsingContextPresence::Present | HtmlBrowsingContextPresence::Unavailable => {
            HtmlScriptingStatus::Unavailable
        }
    };
    let registry = match metadata.custom_element_registry() {
        DomRegistryBinding::Null => HtmlCaptureRegistryInput::Null,
    };
    let subject = builder.add_document(metadata.mode(), scripting, registry)?;
    builder.establish_document_template_facts(
        &subject,
        match metadata.class() {
            DomDocumentClass::Xml => HtmlDocumentClass::NonHtml,
            DomDocumentClass::Html => HtmlDocumentClass::Html,
        },
        HtmlTemplateDocumentRole::Ordinary,
        HtmlCaptureTemplateCacheInput::Absent,
    )?;
    builder.establish_document_nonce_environment(
        &subject,
        HtmlNonceDocumentEnvironment::new(environment, HtmlHeaderDeliveredCsp::Unavailable),
    )?;
    Ok(subject)
}

impl DomSnapshot {
    /// Captures every managed node belonging to this exact document, including
    /// detached roots, plus the actual shadow association/ancestor closure.
    /// Supplemental documents retain actual metadata without claiming their
    /// unrelated ordinary trees are complete.
    pub fn html_capture(
        &self,
        document: DocumentId,
        host: Option<&DomHostCapture>,
        capture_limits: HtmlCaptureLimits,
        adapter_limits: DomHtmlCaptureLimits,
        cancellation: &AtomicBool,
    ) -> Result<DomHtmlCapture, DomHtmlCaptureError> {
        let mut control = Control {
            limits: adapter_limits,
            work: 0,
            cancellation,
        };
        control.charge(1)?;
        let view = self.document(document)?;
        if let Some(host) = host {
            host.validate_for(view).map_err(DomHtmlCaptureError::Host)?;
        }
        let revision = HtmlHostFactsRevision::fresh();
        let mut builder = HtmlCaptureBuilder::new(capture_limits, revision.clone());
        let mut planned = Vec::new();
        // Charge every arena slot, including destroyed/retired entries which
        // the public live-node iterator skips. Sparse snapshots still require
        // bounded scanning and cancellation polling.
        for (index, slot) in self.state.slots.iter().enumerate() {
            control.charge(1)?;
            let Some(value) = &slot.node else {
                continue;
            };
            let node = NodeId {
                owner: self.owner,
                slot: index,
                generation: slot.generation,
            };
            if value.document == document {
                control.include(&mut planned, node, true)?;
            }
        }
        // Close actual associations without expanding every supplemental
        // document's ordinary tree. Hosts/ancestors may be partial containers;
        // associated shadow subtrees have complete ordinary child lists.
        loop {
            let mut next = None;
            for (index, entry) in planned.iter().enumerate() {
                control.charge(1)?;
                if !entry.expanded {
                    next = Some(index);
                    break;
                }
            }
            let Some(index) = next else { break };
            let entry = planned[index];
            planned[index].expanded = true;
            let node = self.node(entry.node)?;
            control.include(&mut planned, node.node.document.node(), false)?;
            if let Some(parent) = node.parent() {
                control.include(&mut planned, parent, false)?;
            }
            if let Some(host) = node.shadow_host() {
                control.include(&mut planned, host, false)?;
            }
            if let Some(root) = node.hosted_shadow_root() {
                control.include(&mut planned, root, true)?;
            }
            if let Some(button) = node.as_button() {
                for target in [
                    button.explicit_command_for(),
                    button.explicit_popover_target(),
                ]
                .into_iter()
                .flatten()
                {
                    control.charge(1)?;
                    control.include(&mut planned, target, false)?;
                }
            }
            if let Some(owner) = node.node.control_owner {
                control.include(&mut planned, owner, false)?;
            }
            if let Some(map) = &node.node.form_past_names {
                for entry in map.entries.iter() {
                    control.charge(1)?;
                    control.include(&mut planned, entry.target, false)?;
                }
            }
            // Directionality consumes stored history, including references
            // outside this node-document. Preserve actual closure/ancestors.
            if let Some(slot) = node.as_slot() {
                for assigned in slot.assigned_nodes() {
                    control.charge(1)?;
                    control.include(&mut planned, *assigned, true)?;
                }
            }
            if entry.complete_children {
                for child in node.children() {
                    control.include(&mut planned, *child, true)?;
                }
            }
        }
        let mut staged = Vec::new();
        // Explicit document records precede their actual member nodes.
        for entry in &planned {
            control.charge(1)?;
            if matches!(self.node(entry.node)?.kind(), DomNodeKind::Document(_)) {
                control.reserve(&mut staged, 1)?;
                staged.push(StagedEntry {
                    node: entry.node,
                    subject: add_document(&mut builder, self, DocumentId(entry.node))?,
                });
            }
        }
        // Hosts precede their shadow roots even when arena order differs.
        for entry in &planned {
            control.charge(1)?;
            let node = self.node(entry.node)?;
            if matches!(
                node.kind(),
                DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_)
            ) {
                continue;
            }
            let root = control.subject(&staged, node.node.document.node())?;
            control.reserve(&mut staged, 1)?;
            let subject = match node.kind() {
                DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_) => {
                    unreachable!("handled in document and shadow passes")
                }
                // Exhaustive: a new hosted kind must define real host/document
                // associations here instead of inheriting plain-fragment absence.
                DomNodeKind::Element(element) => {
                    let saved_is = match element.saved_is() {
                        Some(value) => HtmlCapturedIsValue::Value(value.clone()),
                        None => HtmlCapturedIsValue::Absent,
                    };
                    let registry = match element.custom_element_registry() {
                        DomRegistryBinding::Null => HtmlCaptureRegistryInput::Null,
                    };
                    let subject = builder.add_element(
                        &root,
                        element.name().clone(),
                        element.attributes().clone(),
                        saved_is,
                        registry,
                    )?;
                    let definition = match element.custom_element_definition() {
                        DomElementDefinition::Null => HtmlCaptureDefinitionInput::Null,
                    };
                    builder.establish_custom_element_context(
                        &subject,
                        HtmlCapturedCustomElementState::Known(element.custom_element_state()),
                        definition,
                    )?;
                    if let Some(nonce) = element.nonce() {
                        builder.establish_element_nonce(&subject, nonce.clone())?;
                    }
                    if node.hosted_shadow_root().is_none() {
                        builder.establish_no_shadow_root(&subject)?;
                    }
                    subject
                }
                DomNodeKind::DocumentFragment => {
                    let subject = builder.add_fragment(&root)?;
                    builder.establish_unhosted(&subject)?;
                    subject
                }
                DomNodeKind::Text(data) => builder.add_text(&root, data.clone())?,
                DomNodeKind::Comment(data) => builder.add_comment(&root, data.clone())?,
                DomNodeKind::ProcessingInstruction(value) => builder.add_processing_instruction(
                    &root,
                    value.target().clone(),
                    value.data().clone(),
                )?,
                DomNodeKind::DocumentType(value) => builder.add_doctype(
                    &root,
                    HtmlDoctype::new(
                        value.name().clone(),
                        Some(value.public_id().clone()),
                        Some(value.system_id().clone()),
                    ),
                )?,
            };
            staged.push(StagedEntry {
                node: node.id(),
                subject,
            });
        }
        for entry in &planned {
            control.charge(1)?;
            let node = self.node(entry.node)?;
            if let DomNodeKind::ShadowRoot(state) = node.kind() {
                let document = control.subject(&staged, node.node.document.node())?;
                let host = control.subject(&staged, self.shadow_root(entry.node)?.host())?;
                let registry = match state.custom_element_registry() {
                    DomRegistryBinding::Null => HtmlCaptureRegistryInput::Null,
                };
                control.reserve(&mut staged, 1)?;
                let subject = builder.add_shadow_root(
                    &document,
                    &host,
                    HtmlCapturedShadowProperties::new(
                        state.properties(),
                        state.declarative(),
                        state.available_to_element_internals(),
                        state.keep_custom_element_registry_null(),
                    ),
                    registry,
                )?;
                staged.push(StagedEntry {
                    node: entry.node,
                    subject,
                });
            }
        }
        let mut children = Vec::new();
        for plan in &planned {
            control.charge(1)?;
            let node = self.node(plan.node)?;
            let subject = control.subject(&staged, plan.node)?;
            match node.kind() {
                DomNodeKind::Document(_)
                | DomNodeKind::DocumentFragment
                | DomNodeKind::ShadowRoot(_)
                | DomNodeKind::Element(_) => {
                    if plan.complete_children {
                        children.clear();
                        control.reserve(&mut children, node.children().len())?;
                        for child in node.children() {
                            children.push(control.subject(&staged, *child)?);
                        }
                        builder.establish_children(&subject, &children)?;
                    }
                }
                DomNodeKind::Text(_)
                | DomNodeKind::Comment(_)
                | DomNodeKind::ProcessingInstruction(_)
                | DomNodeKind::DocumentType(_) => {}
            }
            if node.parent().is_none() {
                match node.kind() {
                    DomNodeKind::Element(_)
                    | DomNodeKind::Text(_)
                    | DomNodeKind::Comment(_)
                    | DomNodeKind::ProcessingInstruction(_)
                    | DomNodeKind::DocumentType(_) => builder.establish_root(&subject)?,
                    DomNodeKind::Document(_)
                    | DomNodeKind::DocumentFragment
                    | DomNodeKind::ShadowRoot(_) => {}
                }
            } else if let Some(parent) = node.parent() {
                // Complete parents establish their reciprocal child edges;
                // partial supplemental ancestors establish only the true edge.
                let mut complete_parent = false;
                for entry in &planned {
                    control.charge(1)?;
                    if entry.node == parent {
                        complete_parent = entry.complete_children;
                        break;
                    }
                }
                if !complete_parent {
                    builder.establish_parent(&subject, &control.subject(&staged, parent)?)?;
                }
            }
        }
        control.charge(1)?;
        let capture = builder.finish()?;
        let mut entries = Vec::new();
        control.reserve(&mut entries, staged.len())?;
        for entry in staged {
            control.charge(1)?;
            entries.push(Entry {
                node: entry.node,
                subject: capture.subject_id(&entry.subject)?,
            });
        }
        let context = HtmlSemanticContext::new(
            &capture,
            revision,
            control::with_native_state(
                control::native_input(self, &entries, &mut control)?,
                self,
                &capture,
                &entries,
                &mut control,
            )?,
            control.limits.subjects,
        )
        .map_err(DomHtmlCaptureError::Context)?;
        control.charge(1)?;
        Ok(DomHtmlCapture {
            source: self.clone(),
            basis: view.stamp(),
            host: host.cloned(),
            capture,
            context,
            entries,
        })
    }
}

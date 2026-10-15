//! Direct canonical-snapshot adaptation into the HTML semantic owner.

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
    HtmlDocumentClass, HtmlHostFactsRevision, HtmlScriptingStatus, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextError, HtmlSemanticContextInput, HtmlSemanticFailure,
    HtmlSemanticLimits, HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticSession,
    HtmlTemplateDocumentRole,
};

use crate::{
    DocumentId, DomDocumentClass, DomDocumentStamp, DomElementDefinition, DomError, DomHostCapture,
    DomHostError, DomNodeKind, DomRegistryBinding, DomSnapshot, NodeId,
};

/// Adapter-owned construction storage/work, separate from producer capture retention limits.
/// Each mapping/staging vector and the reusable child scratch have at most
/// `subjects` entries. Producer allocations obey the supplied HtmlCaptureLimits.
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

/// One exact canonical document, its optional host capture, and HTML's immutable
/// interpretation input. Detached nodes belong to the same document closure.
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
        self.source.document(self.basis.document())?.node(node)?;
        self.entries
            .iter()
            .find(|e| e.node == node)
            .map(|e| e.subject.clone())
            .ok_or(DomHtmlCaptureError::UnmappedSubject)
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
        self.charge(1)?;
        let count = values
            .len()
            .checked_add(additional)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        if count > self.limits.subjects {
            return Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Subjects,
                limit: self.limits.subjects,
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
}

impl DomSnapshot {
    /// Captures every managed node belonging to this exact document, including
    /// detached roots. This current ordinary/common profile has no admitted
    /// template/shadow/control families; future hosted kinds need their actual
    /// association closure, which can cross node-document membership boundaries.
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
        let metadata = view.metadata();
        let environment = metadata.native_environment().browsing_context();
        // HTML877d #concept-n-noscript: null browsing context independently
        // disables scripting for Nodes. Present alone would not prove Enabled.
        let scripting = match environment {
            HtmlBrowsingContextPresence::Absent => HtmlScriptingStatus::Disabled,
            HtmlBrowsingContextPresence::Present | HtmlBrowsingContextPresence::Unavailable => {
                HtmlScriptingStatus::Unavailable
            }
        };
        let revision = HtmlHostFactsRevision::fresh();
        let mut builder = HtmlCaptureBuilder::new(capture_limits, revision.clone());
        let mut staged = Vec::new();
        control.reserve(&mut staged, 1)?;
        let registry = match metadata.custom_element_registry() {
            DomRegistryBinding::Null => HtmlCaptureRegistryInput::Null,
        };
        let root = builder.add_document(metadata.mode(), scripting, registry)?;
        builder.establish_document_template_facts(
            &root,
            match metadata.class() {
                DomDocumentClass::Xml => HtmlDocumentClass::NonHtml,
                DomDocumentClass::Html => HtmlDocumentClass::Html,
            },
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )?;
        builder.establish_document_nonce_environment(
            &root,
            HtmlNonceDocumentEnvironment::new(environment, HtmlHeaderDeliveredCsp::Unavailable),
        )?;
        staged.push(StagedEntry {
            node: document.node(),
            subject: root.clone(),
        });
        // Charge every arena slot, including destroyed/retired entries which
        // the public live-node iterator skips. Sparse snapshots still require
        // bounded scanning and cancellation polling.
        for (index, slot) in self.state.slots.iter().enumerate() {
            control.charge(1)?;
            if slot.node.is_none() {
                continue;
            }
            let node = NodeId {
                owner: self.owner,
                slot: index,
                generation: slot.generation,
            };
            if node == document.node() {
                continue;
            }
            let node = match view.node(node) {
                Ok(node) => node,
                Err(DomError::WrongDocument) => continue,
                Err(error) => return Err(error.into()),
            };
            control.reserve(&mut staged, 1)?;
            // Exhaustive: a new hosted kind must define real host/document
            // associations here instead of inheriting plain-fragment absence.
            let subject = match node.kind() {
                DomNodeKind::Document(_) => return Err(DomError::WrongDocument.into()),
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
                    builder.establish_no_shadow_root(&subject)?;
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
        let mut children = Vec::new();
        for entry in &staged {
            control.charge(1)?;
            let node = view.node(entry.node)?;
            match node.kind() {
                DomNodeKind::Document(_)
                | DomNodeKind::DocumentFragment
                | DomNodeKind::Element(_) => {
                    children.clear();
                    control.reserve(&mut children, node.children().len())?;
                    for child in node.children() {
                        children.push(control.subject(&staged, *child)?);
                    }
                    builder.establish_children(&entry.subject, &children)?;
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
                    | DomNodeKind::DocumentType(_) => builder.establish_root(&entry.subject)?,
                    DomNodeKind::Document(_) | DomNodeKind::DocumentFragment => {}
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
            HtmlSemanticContextInput::new(Vec::new()),
            0,
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

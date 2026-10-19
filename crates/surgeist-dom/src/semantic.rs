//! Direct canonical-snapshot adaptation into the HTML semantic owner.

mod base;
mod control;
mod inheritance;
mod modal;
mod slot;
pub use slot::DomSlotSemanticLimits;

use std::{
    convert::Infallible,
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

use surgeist_html::{
    HtmlDoctype, HtmlHeaderDeliveredCsp, HtmlNonceDocumentEnvironment, HtmlValueInterpreter,
};
#[cfg(test)]
use surgeist_html_semantics::HtmlScriptingStatus;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderRegistry, HtmlCaptureBuilderSubject,
    HtmlCaptureDefinitionInput, HtmlCaptureElementError, HtmlCaptureError, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCaptureTemplateCacheInput,
    HtmlCapturedCustomElementState, HtmlCapturedDefinitions, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlDocumentClass, HtmlHostFactsRevision, HtmlRegistryScope,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextError,
    HtmlSemanticContextInput, HtmlSemanticFailure, HtmlSemanticLimits, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticSession, HtmlTemplateDocumentRole,
};

use crate::{
    DocumentId, DomDocumentClass, DomDocumentStamp, DomElementDefinition, DomError, DomHostCapture,
    DomHostError, DomNodeKind, DomRegistryBinding, DomRegistryId, DomSnapshot, NodeId,
};

/// Adapter-owned construction storage/work, separate from producer capture retention limits.
/// `work` independently bounds adapter work and shared control-fact validation
/// including its context join. Each closure/mapping/staging vector and the reusable child scratch have at most
/// `subjects` entries. Nullable reference facts have at most twice `subjects`
/// entries (two per Button, one per Input), with their validation independently bounded by
/// `work`. Actual HTML/null-definition classifications also use subjects, with
/// their context qualification/duplicate comparisons charged to adapter work.
/// Document design-mode rows use at most `subjects` records. The same finite
/// `work` bound independently limits GlobalInheritance producer validation per
/// formation, including Slot context reconstruction; it is not a cumulative
/// producer work meter across separate formations.
/// Producer allocations obey the supplied HtmlCaptureLimits.
/// Control facts admit up to twice subjects (association plus actual Input flags).
/// Applicable current values use the shared HTML type predicate and the supplied
/// capture UTF-16 ceiling; Slot supplementation uses its explicit language limits.
/// Initial Document modal rows use `subjects` and independently bounded producer
/// `work`, without supplying flat-node coverage. Slot supplementation admits the
/// combined flat-node and Document modal vector capacities through its explicit
/// `HtmlInertFactLimits`.
/// Owned element-value export uses the producer's separate cumulative
/// `HtmlCaptureLimits::element_value_bytes` allowance. It covers initial names,
/// attributes, saved-is values and pending validation, rather than total capture
/// heap. Adapter work and cancellation remain independently admitted here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHtmlCaptureLimits {
    subjects: usize,
    work: usize,
    base: Option<BaseCapturePolicy>,
}
impl DomHtmlCaptureLimits {
    pub fn new(subjects: usize, work: usize) -> Self {
        Self {
            subjects,
            work,
            base: None,
        }
    }
    /// Explicit producer capture-fact formation limits. Validation work is
    /// cumulative across this capture and subsequent Slot reconstruction;
    /// record/unit limits bound each retained fact formation independently of
    /// canonical retained observations and adapter subject/work admission.
    pub fn with_base_facts(
        mut self,
        records: usize,
        utf16_units: usize,
        validation_work: usize,
    ) -> Self {
        self.base = Some(BaseCapturePolicy {
            records,
            units: utf16_units,
            work: validation_work,
        });
        self
    }
    pub fn subjects(self) -> usize {
        self.subjects
    }
    pub fn work(self) -> usize {
        self.work
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BaseCapturePolicy {
    records: usize,
    units: usize,
    work: usize,
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
    ElementValues(HtmlCaptureElementError<Infallible>),
    Context(HtmlSemanticContextError),
    Semantic(HtmlSemanticFailure),
    Language(surgeist_html_semantics::HtmlLanguageDirectionError),
    Inert(surgeist_html_semantics::HtmlInertFactError),
    GlobalInheritance(surgeist_html_semantics::HtmlGlobalInheritanceError),
    Controls(surgeist_html_semantics::HtmlControlFactsError),
    BaseFacts(surgeist_html_semantics::HtmlBaseFactsError),
    OutputDefaults(surgeist_html_semantics::HtmlOutputDefaultError),
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
            Self::ElementValues(e) => Some(e),
            Self::Context(e) => Some(e),
            Self::Semantic(e) => Some(e),
            Self::Language(e) => Some(e),
            Self::Inert(e) => Some(e),
            Self::GlobalInheritance(e) => Some(e),
            Self::Controls(e) => Some(e),
            Self::BaseFacts(e) => Some(e),
            Self::OutputDefaults(e) => Some(e),
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

fn element_capture_error(
    error: HtmlCaptureElementError<DomHtmlCaptureError>,
) -> DomHtmlCaptureError {
    match error {
        HtmlCaptureElementError::Capture(error) => DomHtmlCaptureError::Capture(error),
        HtmlCaptureElementError::Interrupted(error) => error,
        HtmlCaptureElementError::Attributes(error) => {
            DomHtmlCaptureError::ElementValues(HtmlCaptureElementError::Attributes(error))
        }
        HtmlCaptureElementError::InconsistentAttributes { index } => {
            DomHtmlCaptureError::ElementValues(HtmlCaptureElementError::InconsistentAttributes {
                index,
            })
        }
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
struct StagedRegistry {
    canonical: DomRegistryId,
    captured: HtmlCaptureBuilderRegistry,
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
    output_default_units: usize,
    base_policy: Option<BaseCapturePolicy>,
    base_work: usize,
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
    registries: &mut Vec<StagedRegistry>,
    control: &mut Control<'_>,
) -> Result<HtmlCaptureBuilderSubject, DomHtmlCaptureError> {
    let metadata = snapshot.document(document)?.metadata();
    let environment = metadata.document_environment();
    let registry = registry_input(
        builder,
        snapshot,
        metadata.custom_element_registry(),
        registries,
        control,
    )?;
    let subject = builder.add_document(metadata.mode(), environment.scripting(), registry)?;
    builder.establish_document_nonce_environment(
        &subject,
        HtmlNonceDocumentEnvironment::new(
            environment.browsing_context(),
            HtmlHeaderDeliveredCsp::Unavailable,
        ),
    )?;
    Ok(subject)
}

fn registry_input(
    builder: &mut HtmlCaptureBuilder,
    snapshot: &DomSnapshot,
    binding: DomRegistryBinding,
    registries: &mut Vec<StagedRegistry>,
    control: &mut Control<'_>,
) -> Result<HtmlCaptureRegistryInput, DomHtmlCaptureError> {
    let DomRegistryBinding::Registry(canonical) = binding else {
        return Ok(HtmlCaptureRegistryInput::Null);
    };
    for entry in registries.iter() {
        control.charge(1)?;
        if entry.canonical == canonical {
            return Ok(HtmlCaptureRegistryInput::Registry(entry.captured.clone()));
        }
    }
    control.charge(1)?;
    let record = snapshot
        .state
        .registries
        .as_ref()
        .ok_or(DomError::StaleNode)?
        .get(canonical)?;
    control.charge(1)?;
    let definitions = record.capture.definitions(&record.local)?;
    // The canonical owner currently issues only the actual complete-empty
    // Global profile. Preserve unavailable/unsupported evidence as a refusal;
    // neither can become a null or guessed empty registry.
    match definitions {
        HtmlCapturedDefinitions::Complete([]) => {}
        HtmlCapturedDefinitions::Complete(_) => return Err(DomError::NotSupported.into()),
        HtmlCapturedDefinitions::Unavailable => {
            return Err(HtmlCaptureError::UnavailableDefinitions(record.local.clone()).into());
        }
    }
    control.charge(1)?;
    let scope = record.capture.registry_scope(&record.local)?;
    match scope {
        HtmlRegistryScope::Global => {}
        HtmlRegistryScope::Scoped => return Err(DomError::NotSupported.into()),
        HtmlRegistryScope::Unavailable => {
            return Err(HtmlCaptureError::MissingFact {
                subject: None,
                fact: surgeist_html_semantics::HtmlCaptureFact::Registry,
            }
            .into());
        }
    }
    control.reserve(registries, 1)?;
    let captured = builder.add_registry(Some(Vec::new()))?;
    control.charge(1)?;
    builder.establish_registry_scope(&captured, scope)?;
    registries.push(StagedRegistry {
        canonical,
        captured: captured.clone(),
    });
    Ok(HtmlCaptureRegistryInput::Registry(captured))
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
        self.require_complete_source()?;
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
        let mut registries = Vec::new();
        let mut planned = Vec::new();
        // Charge every arena slot, including destroyed/retired entries which
        // the public live-node iterator skips. Sparse snapshots still require
        // bounded scanning and cancellation polling.
        for (index, slot) in self.state.slots.iter() {
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
            if let Some(host) = node.fragment_host() {
                control.include(&mut planned, host, false)?;
            }
            if let Some(contents) = node.node.template_contents {
                control.include(&mut planned, contents, true)?;
            }
            if let Some(inert) = node.node.template_document.and_then(|state| state.inert()) {
                control.include(&mut planned, inert, false)?;
            }
            if let Some(root) = node.hosted_shadow_root() {
                control.include(&mut planned, root, true)?;
            }
            if let Some(targets) = node.node.reflected_targets {
                for key in [
                    crate::reflected::ReflectedTarget::CommandFor,
                    crate::reflected::ReflectedTarget::PopoverTarget,
                ] {
                    let Some(Some(target)) = targets.get(key) else {
                        continue;
                    };
                    control.charge(1)?;
                    control.include(&mut planned, target, false)?;
                }
            }
            if let Some(owner) = node.node.control_owner {
                control.include(&mut planned, owner, false)?;
            }
            if let Some(parsed) = node.node.hyperlink.as_ref().and_then(|h| h.parsed.as_ref()) {
                control.include(&mut planned, parsed.document, false)?;
            }
            if let Some(script) = node.node.script {
                for document in [script.parser_document, script.preparation_document]
                    .into_iter()
                    .flatten()
                {
                    control.charge(1)?;
                    control.include(&mut planned, document, false)?;
                }
            }
            if let Some(map) = &node.node.form_past_names {
                for entry in map.entries().iter() {
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
                    subject: add_document(
                        &mut builder,
                        self,
                        DocumentId(entry.node),
                        &mut registries,
                        &mut control,
                    )?,
                });
            }
        }
        // All subjects exist before cache links, and inert roles precede their
        // referring ordinary Documents regardless of arena creation order.
        for inert_pass in [true, false] {
            for entry in &planned {
                control.charge(1)?;
                let node = self.node(entry.node)?;
                let Some(document) = node.as_document() else {
                    continue;
                };
                let inert =
                    document.template_role() == crate::DomTemplateDocumentRole::InertTemplate;
                if inert != inert_pass {
                    continue;
                }
                let subject = control.subject(&staged, entry.node)?;
                let cache = match document.associated_inert_template_document() {
                    Some(owner) => HtmlCaptureTemplateCacheInput::Document(
                        control.subject(&staged, owner.node())?,
                    ),
                    None => HtmlCaptureTemplateCacheInput::Absent,
                };
                builder.establish_document_template_facts(
                    &subject,
                    match document.metadata().class() {
                        DomDocumentClass::Xml => HtmlDocumentClass::NonHtml,
                        DomDocumentClass::Html => HtmlDocumentClass::Html,
                    },
                    if inert {
                        HtmlTemplateDocumentRole::InertTemplate
                    } else {
                        HtmlTemplateDocumentRole::Ordinary
                    },
                    cache,
                )?;
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
                    let registry = registry_input(
                        &mut builder,
                        self,
                        element.custom_element_registry(),
                        &mut registries,
                        &mut control,
                    )?;
                    // The HTML owner admits and materializes its destination
                    // array while this exact canonical snapshot stays retained.
                    // No HTML name/bank or duplicate identity sidecar is staged
                    // in the DOM adapter.
                    let subject = builder
                        .add_element_from_inputs(
                            &root,
                            element.name(),
                            element.attributes(),
                            saved_is,
                            registry,
                            || control.charge(1),
                        )
                        .map_err(element_capture_error)?;
                    let definition = match element.custom_element_definition() {
                        DomElementDefinition::Null => HtmlCaptureDefinitionInput::Null,
                    };
                    builder.establish_custom_element_context(
                        &subject,
                        HtmlCapturedCustomElementState::Known(element.custom_element_state()),
                        definition,
                    )?;
                    if let Some(link) = node.as_link() {
                        control.charge(1)?;
                        builder.establish_link_history(&subject, link.history())?;
                    }
                    if let Some(hyperlink) = node.as_hyperlink() {
                        let history = if let Some(observation) = hyperlink.parse_observation() {
                            let document = control.subject(
                                &staged,
                                hyperlink
                                    .parse_document()
                                    .expect("actual parsed provenance")
                                    .node(),
                            )?;
                            Some((document, observation.clone()))
                        } else {
                            None
                        };
                        builder
                            .establish_hyperlink_url_with(
                                &subject,
                                history
                                    .as_ref()
                                    .map(|(document, snapshot)| (document, snapshot.clone())),
                                || control.charge(1),
                            )
                            .map_err(|e| match e {
                                surgeist_html_semantics::HtmlHyperlinkCaptureError::Capture(e) => {
                                    DomHtmlCaptureError::Capture(e)
                                }
                                surgeist_html_semantics::HtmlHyperlinkCaptureError::Interrupted(
                                    e,
                                ) => e,
                            })?;
                    }
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
                    if node.fragment_host().is_none() {
                        builder.establish_unhosted(&subject)?;
                    }
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
                let registry = registry_input(
                    &mut builder,
                    self,
                    state.custom_element_registry(),
                    &mut registries,
                    &mut control,
                )?;
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
        for entry in &planned {
            control.charge(1)?;
            if let Some(template) = self.node(entry.node)?.as_template() {
                let host = control.subject(&staged, entry.node)?;
                let contents_id = template.contents()?;
                let contents = control.subject(&staged, contents_id)?;
                if matches!(self.node(contents_id)?.kind(), DomNodeKind::ShadowRoot(_)) {
                    builder.establish_template_shadow_reference(&host, &contents)?;
                } else {
                    builder.establish_template_contents(&host, &contents)?;
                }
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
        // Only the checked source snapshot/node map is retained. These local
        // registry handles exist solely to deduplicate this capture's formation.
        drop(registries);
        let mut entries = Vec::new();
        control.reserve(&mut entries, staged.len())?;
        for entry in staged {
            control.charge(1)?;
            entries.push(Entry {
                node: entry.node,
                subject: capture.subject_id(&entry.subject)?,
            });
        }
        let mut language_fallbacks = Vec::new();
        for entry in &entries {
            control.charge(1)?;
            if let Some(document) = self.node(entry.node)?.as_document() {
                control.reserve(&mut language_fallbacks, 1)?;
                language_fallbacks.push(crate::meta::language_fallback(
                    document.metadata(),
                    entry.subject.clone(),
                ));
            }
        }
        let language = surgeist_html_semantics::HtmlLanguageDirectionFacts::new(
            &capture,
            revision.clone(),
            language_fallbacks,
            control::current_values(self, &entries, &mut control, capture_limits.utf16_units())?,
            Vec::new(),
            surgeist_html_semantics::HtmlLanguageDirectionLimits::new(
                control.limits.subjects,
                0,
                capture_limits.utf16_units(),
                control.limits.work,
            ),
        )
        .map_err(DomHtmlCaptureError::Language)?;
        let input = modal::with_facts(
            control::native_input(self, &entries, &mut control)?.with_language_direction(language),
            self,
            &capture,
            &entries,
            Vec::new(),
            surgeist_html_semantics::HtmlInertFactLimits::new(
                control.limits.subjects,
                control.limits.work,
            ),
            &mut control,
        )?;
        let input = control::with_native_state(
            input,
            self,
            &capture,
            &entries,
            &mut control,
            capture_limits.utf16_units(),
        )?;
        let input = inheritance::with_facts(input, self, &capture, &entries, &mut control)?;
        let base_policy = control.limits.base;
        let (input, base_work) = base::with_facts(
            input,
            self,
            &capture,
            &entries,
            &mut control,
            base_policy.map(|p| (p, 0)),
        )?;
        let context = HtmlSemanticContext::new(&capture, revision, input, control.limits.subjects)
            .map_err(DomHtmlCaptureError::Context)?;
        control.charge(1)?;
        Ok(DomHtmlCapture {
            source: self.clone(),
            basis: view.stamp(),
            host: host.cloned(),
            capture,
            context,
            entries,
            output_default_units: capture_limits.utf16_units(),
            base_policy,
            base_work,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DocumentString, Dom, DomBatch, DomLimits, DomNameLimits, Namespace};
    use std::{mem::size_of, sync::Arc};
    use surgeist_html::{Attribute, QualifiedName};
    use surgeist_html_semantics::{HtmlCaptureResource, HtmlCapturedNodeKind};

    fn fixture() -> (Dom, DomSnapshot, DocumentId, NodeId) {
        let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
        let mut dom =
            Dom::new(DomLimits::new(16, 4096, 32, 0, 0, 1, 65536, names).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let element =
            batch.create_element_ns(&document, Namespace::from_uri("urn:element"), "p:Root");
        let mut units = Vec::with_capacity(37);
        units.extend([0xd800, 65]);
        batch.set_attribute_ns(
            &element,
            Namespace::from_uri("urn:attribute"),
            "q:data",
            DocumentString::from_utf16(units),
        );
        batch.set_attribute(&element, "empty", "");
        batch.insert_before(&element, &document, None);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let document = commit.document(&document).unwrap();
        let element = commit.created(&element).unwrap().unwrap();
        let snapshot = dom.snapshot();
        (dom, snapshot, document, element)
    }
    fn limits(bytes: usize) -> HtmlCaptureLimits {
        HtmlCaptureLimits::new(16, 64, 8, 4096, 16, 0, 0, bytes)
    }
    fn adapter() -> DomHtmlCaptureLimits {
        DomHtmlCaptureLimits::new(16, 1_000_000)
    }
    fn string_bytes(value: &DocumentString) -> usize {
        size_of::<Vec<u16>>() + value.shared_code_units().capacity() * size_of::<u16>()
    }
    fn name_bytes(name: &crate::DomQualifiedName) -> usize {
        name.namespace_uri()
            .into_iter()
            .chain(name.prefix())
            .chain([name.local_name()])
            .map(string_bytes)
            .sum()
    }
    fn assert_name(source: &crate::DomQualifiedName, output: &QualifiedName) {
        assert_eq!(output.namespace().uri(), source.namespace_uri());
        assert_eq!(output.prefix(), source.prefix());
        assert_eq!(output.local_name(), source.local_name());
        for (left, right) in source
            .namespace_uri()
            .into_iter()
            .chain(source.prefix())
            .chain([source.local_name()])
            .zip(
                output
                    .namespace()
                    .uri()
                    .into_iter()
                    .chain(output.prefix())
                    .chain([output.local_name()]),
            )
        {
            assert!(Arc::ptr_eq(
                left.shared_code_units(),
                right.shared_code_units()
            ));
        }
    }

    #[test]
    fn canonical_capture_exports_admitted_html_array_with_shared_text_and_retained_snapshot() {
        let (dom, snapshot, document, element) = fixture();
        let stamp = snapshot.stamp();
        let DomNodeKind::Element(source) = snapshot.node(element).unwrap().kind() else {
            panic!("literal element")
        };
        let captured = snapshot
            .html_capture(
                document,
                None,
                limits(1_000_000),
                adapter(),
                &AtomicBool::new(false),
            )
            .unwrap();
        let subject = captured.subject(element).unwrap();
        let HtmlCapturedNodeKind::Element(output) =
            captured.capture().node(&subject).unwrap().kind()
        else {
            panic!("exported element")
        };
        assert_eq!(captured.node(&subject).unwrap(), element);
        assert_eq!(
            captured.basis(),
            snapshot.document(document).unwrap().stamp()
        );
        assert_eq!(captured.source().stamp(), stamp);
        assert_name(source.name(), output.name());
        assert_eq!(
            output.name().namespace().uri(),
            Some(&DocumentString::from("urn:element"))
        );
        assert_eq!(output.name().prefix(), Some(&DocumentString::from("p")));
        assert_eq!(output.name().local_name(), &DocumentString::from("Root"));
        assert_eq!(output.attributes().len(), 2);
        assert!(
            !std::ptr::eq(
                source.attributes().as_ptr().cast::<()>(),
                output.attributes().as_slice().as_ptr().cast::<()>()
            ),
            "two simultaneously retained nonempty arrays belong to their separate owners"
        );
        for (input, output) in source.attributes().iter().zip(output.attributes().iter()) {
            assert_name(input.name(), output.name());
            assert_eq!(input.value(), output.value());
            assert!(Arc::ptr_eq(
                input.value().shared_code_units(),
                output.value().shared_code_units()
            ));
        }
        let first = &output.attributes().as_slice()[0];
        assert_eq!(
            first.name().namespace().uri(),
            Some(&DocumentString::from("urn:attribute"))
        );
        assert_eq!(first.name().prefix(), Some(&DocumentString::from("q")));
        assert_eq!(first.name().local_name(), &DocumentString::from("data"));
        assert_eq!(first.value().code_units(), &[0xd800, 65]);
        let second = &output.attributes().as_slice()[1];
        assert!(second.name().namespace().uri().is_none() && second.name().prefix().is_none());
        assert_eq!(second.name().local_name(), &DocumentString::from("empty"));
        assert!(second.value().is_empty());
        // Compute this coordinate from the actual HTML destination capacity and
        // the immutable input backing capacities, not a DOM-bank layout or the
        // producer's cached receipt. Source/snapshot storage stays retained.
        let bytes = size_of::<Vec<Attribute>>()
            + output.attributes().shared_attributes().capacity() * size_of::<Attribute>()
            + name_bytes(source.name())
            + source
                .attributes()
                .iter()
                .map(|a| name_bytes(a.name()) + string_bytes(a.value()))
                .sum::<usize>();
        assert_eq!(captured.capture().usage().element_value_bytes(), bytes);
        assert!(
            matches!(snapshot.html_capture(document, None, limits(bytes - 1), adapter(), &AtomicBool::new(false)),
            Err(DomHtmlCaptureError::Capture(HtmlCaptureError::LimitExceeded { resource: HtmlCaptureResource::ElementValueBytes, limit, required })) if limit == bytes - 1 && required == bytes)
        );
        let exact = snapshot
            .html_capture(
                document,
                None,
                limits(bytes),
                adapter(),
                &AtomicBool::new(false),
            )
            .unwrap();
        assert_eq!(exact.capture().usage().element_value_bytes(), bytes);
        assert_eq!(snapshot.stamp(), stamp);
        drop(dom);
        assert_eq!(captured.source().stamp(), stamp);
        assert_eq!(
            exact.node(&exact.subject(element).unwrap()).unwrap(),
            element
        );
        assert_eq!(first.value().code_units(), &[0xd800, 65]);
    }

    #[test]
    fn borrowed_element_export_preserves_first_adapter_work_and_cancellation_errors() {
        let (_dom, snapshot, _document, element) = fixture();
        let DomNodeKind::Element(source) = snapshot.node(element).unwrap().kind() else {
            panic!("literal element")
        };
        for cancelled in [false, true] {
            let mut builder =
                HtmlCaptureBuilder::new(limits(1_000_000), HtmlHostFactsRevision::fresh());
            let document = builder
                .add_document(
                    crate::HtmlDocumentMode::NoQuirks,
                    HtmlScriptingStatus::Disabled,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            let before = builder.usage();
            let cancellation = AtomicBool::new(cancelled);
            let mut control = Control {
                limits: DomHtmlCaptureLimits::new(16, 0),
                work: 0,
                cancellation: &cancellation,
            };
            let backing = source.attributes()[0].value().shared_code_units();
            let strong = Arc::strong_count(backing);
            let result = builder
                .add_element_from_inputs(
                    &document,
                    source.name(),
                    source.attributes(),
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Null,
                    || control.charge(1),
                )
                .map_err(element_capture_error);
            if cancelled {
                assert!(matches!(result, Err(DomHtmlCaptureError::Cancelled)));
            } else {
                assert!(matches!(
                    result,
                    Err(DomHtmlCaptureError::ResourceLimit {
                        resource: DomHtmlCaptureResource::Work,
                        limit: 0
                    })
                ));
            }
            assert_eq!(control.work, 0);
            assert_eq!(builder.usage(), before);
            assert_eq!(Arc::strong_count(backing), strong);
        }
    }
}

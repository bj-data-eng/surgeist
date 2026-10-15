//! Concrete Slot directionality and CSS-flat inert supplement composition.
use super::*;
use surgeist_html_semantics::{
    HtmlDocumentFlatTreeRelation, HtmlFlatTreeProducerId, HtmlFlatTreeRevision,
    HtmlInertFactLimits, HtmlInertFacts, HtmlInertNodeFact, HtmlLanguageDirectionFacts,
    HtmlLanguageDirectionLimits, HtmlSlotAssignedNodes,
};

/// Independent finite admission for the query, mapping and shared producers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSlotSemanticLimits {
    queries: crate::DomSlotQueryLimits,
    adapter: DomHtmlCaptureLimits,
    language: HtmlLanguageDirectionLimits,
    inert: HtmlInertFactLimits,
}
impl DomSlotSemanticLimits {
    pub fn new(
        queries: crate::DomSlotQueryLimits,
        adapter: DomHtmlCaptureLimits,
        language: HtmlLanguageDirectionLimits,
        inert: HtmlInertFactLimits,
    ) -> Self {
        Self {
            queries,
            adapter,
            language,
            inert,
        }
    }
}
impl DomHtmlCapture {
    fn slot_subject(
        &self,
        control: &mut Control<'_>,
        node: NodeId,
    ) -> Result<HtmlCaptureSubjectId, DomHtmlCaptureError> {
        for entry in &self.entries {
            control.charge(1)?;
            if entry.node == node {
                return Ok(entry.subject.clone());
            }
        }
        Err(DomHtmlCaptureError::UnmappedSubject)
    }
    /// Installs real canonical stored assignments and current flat relations
    /// once for reuse by this capture's ordinary HTML semantic queries.
    /// Actual canonical control facts are rebuilt into the same context;
    /// missing language fallback/modal facts stay unavailable.
    pub fn with_slot_semantics(
        mut self,
        limits: DomSlotSemanticLimits,
        cancel: &AtomicBool,
    ) -> Result<Self, DomHtmlCaptureError> {
        let mut control = Control {
            limits: limits.adapter,
            work: 0,
            cancellation: cancel,
        };
        control.charge(1)?;
        let mut assignments = Vec::new();
        let mut inert_nodes = Vec::new();
        for entry in &self.entries {
            control.charge(1)?;
            let node = self.source.node(entry.node)?;
            if let Some(slot) = node.as_slot() {
                let mut nodes = Vec::new();
                for assigned in slot.assigned_nodes() {
                    control.reserve(&mut nodes, 1)?;
                    nodes.push(self.slot_subject(&mut control, *assigned)?);
                }
                control.reserve(&mut assignments, 1)?;
                assignments.push(HtmlSlotAssignedNodes::new(
                    entry.subject.clone(),
                    Some(nodes),
                ));
            }
            if !node.is_connected() {
                control.reserve(&mut inert_nodes, 1)?;
                inert_nodes.push(HtmlInertNodeFact::new(
                    entry.subject.clone(),
                    Some(false),
                    HtmlDocumentFlatTreeRelation::Excluded,
                ));
            }
        }
        // Actual document-flat membership follows shadow-including roots,
        // independently of node_document. Only one derived flat buffer is
        // retained at a time; related supplemental documents are not guessed.
        for document in &self.entries {
            control.charge(1)?;
            let Some(view) = self.source.node(document.node)?.as_document() else {
                continue;
            };
            let flat = self.source.flat_tree(view.id(), limits.queries, cancel)?;
            for entry in &self.entries {
                control.charge(1)?;
                let node = self.source.node(entry.node)?;
                if node.shadow_including_root() != document.node {
                    continue;
                }
                control.charge(flat.nodes().count())?;
                let relation = match flat.relation(entry.node)? {
                    crate::DomFlatRelation::Excluded => HtmlDocumentFlatTreeRelation::Excluded,
                    crate::DomFlatRelation::Root => HtmlDocumentFlatTreeRelation::Root,
                    crate::DomFlatRelation::Parent(parent) => HtmlDocumentFlatTreeRelation::Parent(
                        self.slot_subject(&mut control, parent)?,
                    ),
                };
                control.reserve(&mut inert_nodes, 1)?;
                inert_nodes.push(HtmlInertNodeFact::new(
                    entry.subject.clone(),
                    Some(true),
                    relation,
                ));
            }
        }
        let revision = self.capture.host_facts_revision().clone();
        let language = HtmlLanguageDirectionFacts::new(
            &self.capture,
            revision.clone(),
            Vec::new(),
            super::control::current_values(
                &self.source,
                &self.entries,
                &mut control,
                limits.language.code_units(),
            )?,
            assignments,
            limits.language,
        )
        .map_err(DomHtmlCaptureError::Language)?;
        control.charge(1)?;
        // One concrete selected flat algorithm/configuration, qualified by its
        // own opaque producer identity and the exact canonical revision.
        let flat_revision = HtmlFlatTreeRevision::new(
            HtmlFlatTreeProducerId::fresh(),
            0,
            self.source.revision().get(),
        );
        let inert = HtmlInertFacts::new(
            &self.capture,
            revision.clone(),
            flat_revision.clone(),
            inert_nodes,
            Vec::new(),
            limits.inert,
        )
        .map_err(DomHtmlCaptureError::Inert)?;
        control.charge(1)?;
        self.context = HtmlSemanticContext::new(
            &self.capture,
            revision,
            super::control::with_native_state(
                super::control::native_input(&self.source, &self.entries, &mut control)?
                    .with_language_direction(language)
                    .with_inert(flat_revision, Some(inert)),
                &self.source,
                &self.capture,
                &self.entries,
                &mut control,
            )?,
            control.limits.subjects,
        )
        .map_err(DomHtmlCaptureError::Context)?;
        control.charge(1)?;
        Ok(self)
    }
}

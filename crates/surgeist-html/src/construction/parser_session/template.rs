//! Source-defined template documents and state in the existing arena registry.
//! HTML 877d6146487f: appropriate template contents owner, creation and adoption.

use super::*;

/// A real Document node qualified by the issuing arena. The index is private.
#[cfg(test)]
pub(crate) struct ParserDocument {
    node: ParserNode,
}

/// An actual open template subject, never a caller-supplied parsing flag.
pub(crate) enum CreationContext<'a> {
    Ordinary,
    Template(&'a ParserNode),
}

/// Borrowed, actual creation-location facts qualified by the session producer.
pub(crate) struct CreationLocation<'a> {
    pub(crate) intended_parent: &'a ParserNode,
    pub(crate) current_form: Option<&'a ParserNode>,
    pub(crate) context: CreationContext<'a>,
}

pub(super) struct PreparedDocuments {
    binding: Option<usize>,
    contents_binding: Option<usize>,
    facts: Option<usize>,
    pub(super) registry: Option<usize>,
}

#[derive(Clone, Copy)]
pub(super) enum DocumentState {
    Ordinary { inert: Option<usize> },
    InertTemplate,
}
#[derive(Clone, Copy)]
pub(super) struct TemplateMetadata {
    pub(super) contents: usize,
    pub(super) target: Option<usize>,
    pub(super) start: Option<usize>,
    pub(super) end: Option<usize>,
}
pub(crate) struct TemplateState {
    pub(crate) contents: ParserNode,
    pub(crate) target: Option<ParserNode>,
    pub(crate) _start: Option<ParserNode>,
    pub(crate) end: Option<ParserNode>,
}

impl SessionStorage {
    pub(super) fn validate_template_facts(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        c.require_identity(&self.control)?;
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            match payload {
                RetainedPayload::BaseDocument { .. } => self.validate_base_facts(c)?,
                RetainedPayload::NodeDocument { node, document } => {
                    let node = node.ok_or(SessionError::NodeDocumentUnavailable)?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if node >= self.records.len() {
                        return Err(SessionError::NodeDocumentUnavailable);
                    }
                    self.validate_document_ref(c, *document)?;
                    if matches!(self.records[node].kind, HtmlNodeKind::Document(_))
                        && DocumentRef::Owned(node) != *document
                    {
                        return Err(SessionError::NodeDocumentUnavailable);
                    }
                }
                RetainedPayload::Document { node, state, .. } => {
                    let node = node.ok_or(SessionError::NodeDocumentUnavailable)?;
                    self.document_slot(c, node)?;
                    if let DocumentState::Ordinary { inert: Some(inert) } = state {
                        let slot = self.document_slot(c, *inert)?;
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if !matches!(
                            self.payloads.values[slot],
                            RetainedPayload::Document {
                                state: DocumentState::InertTemplate,
                                ..
                            }
                        ) {
                            return Err(SessionError::NodeDocumentUnavailable);
                        }
                    }
                }
                RetainedPayload::Template { node, .. } => {
                    self.template_slot(c, node.ok_or(SessionError::TemplateMetadataUnavailable)?)?;
                }
                RetainedPayload::PiAttributes { node, ready, .. } => {
                    let node = node.ok_or(SessionError::ProcessingInstructionMapUnavailable)?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if !ready
                        || !matches!(
                            self.records.get(node).map(|r| &r.kind),
                            Some(HtmlNodeKind::ProcessingInstruction(_))
                        )
                    {
                        return Err(SessionError::ProcessingInstructionMapUnavailable);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub(super) fn creation_document(
        &self,
        c: &mut ParseControl,
        parent: Option<&ParserNode>,
    ) -> Result<Option<DocumentRef>, SessionError> {
        if let Some(parent) = parent {
            let parent = self.index(c, parent)?;
            self.container(c, parent)?;
            return self.node_document_ref(c, parent).map(Some);
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        match self.records[0].kind {
            HtmlNodeKind::Document(_) => {
                self.document_slot(c, 0)?;
                Ok(Some(DocumentRef::Owned(0)))
            }
            HtmlNodeKind::Fragment => Ok(None),
            _ => Err(SessionError::NotDocument),
        }
    }
    fn pi_target(
        &self,
        c: &mut ParseControl,
        node: usize,
        target: &str,
    ) -> Result<bool, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        match &self.records[node].kind {
            HtmlNodeKind::ProcessingInstruction(pi) => exact_ascii(c, Some(pi.target()), target),
            _ => Ok(false),
        }
    }
    fn next_sibling(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Some(parent) = self.records[node].parent else {
            return Ok(None);
        };
        for (position, child) in self.records[parent].children.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if *child == node {
                c.charge(WorkCategory::RecordVisit, 1)?;
                return Ok(self.records[parent].children.get(position + 1).copied());
            }
        }
        Err(SessionError::WrongSibling)
    }
    fn find_markers(
        &self,
        c: &mut ParseControl,
        scope: usize,
        name: &DocumentString,
    ) -> Result<(Option<usize>, Option<usize>), SessionError> {
        use crate::template_markers::{self, First, Siblings, Target};
        let mut next = self.next_view_node(c, scope, scope, None)?;
        while let Some(node) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(
                self.records[node].kind,
                HtmlNodeKind::ProcessingInstruction(_)
            ) && let Some(value) = self.pi_attribute(c, node, "name")?
                && exact_string_equal(c, value, name)?
            {
                let target = if self.pi_target(c, node, "marker")? {
                    Target::Marker
                } else if self.pi_target(c, node, "start")? {
                    Target::Start
                } else {
                    Target::Other
                };
                if template_markers::first(target, true) == First::Marker {
                    return Ok((Some(node), Some(node)));
                }
                if template_markers::first(target, true) == First::Start {
                    let mut nesting = Siblings::default();
                    let mut sibling = self.next_sibling(c, node)?;
                    while let Some(current) = sibling {
                        let is_start = self.pi_target(c, current, "start")?;
                        let is_end = self.pi_target(c, current, "end")?;
                        payload_write::<Siblings>(c, 1)?;
                        let target = if is_start {
                            Target::Start
                        } else if is_end {
                            Target::End
                        } else {
                            Target::Other
                        };
                        if nesting.inspect(target).map_err(|_| {
                            c.representation::<()>(HtmlParseResource::AuxiliaryEntries)
                                .unwrap_err()
                        })? {
                            return Ok((Some(node), Some(current)));
                        }
                        payload_write::<Option<usize>>(c, 1)?;
                        sibling = self.next_sibling(c, current)?;
                    }
                    return Ok((Some(node), None));
                }
            }
            payload_write::<Option<usize>>(c, 1)?;
            next = self.next_view_node(c, scope, node, None)?;
        }
        Ok((None, None))
    }
    pub(super) fn document_slot(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<usize, SessionError> {
        c.require_identity(&self.control)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if !matches!(
            self.records.get(document).map(|r| &r.kind),
            Some(HtmlNodeKind::Document(_))
        ) {
            return Err(SessionError::NotDocument);
        }
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(payload, RetainedPayload::Document { node: Some(node), .. } if *node == document)
            {
                return Ok(slot);
            }
        }
        Err(SessionError::NodeDocumentUnavailable)
    }
    pub(super) fn node_document_ref(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<DocumentRef, SessionError> {
        c.require_identity(&self.control)?;
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::NodeDocument {
                node: Some(bound),
                document,
            } = payload
                && *bound == node
            {
                self.validate_document_ref(c, *document)?;
                c.charge(WorkCategory::CopyByte, mem::size_of::<DocumentRef>() as u64)?;
                return Ok(*document);
            }
        }
        Err(SessionError::NodeDocumentUnavailable)
    }
    pub(super) fn set_node_document(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        document: DocumentRef,
    ) -> Result<(), SessionError> {
        self.validate_document_ref(c, document)?;
        for slot in 0..self.payloads.values.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(self.payloads.values[slot],RetainedPayload::NodeDocument{node:Some(bound),..} if bound==node)
            {
                payload_write::<DocumentRef>(c, 1)?;
                let RetainedPayload::NodeDocument {
                    document: target, ..
                } = &mut self.payloads.values[slot]
                else {
                    unreachable!()
                };
                *target = document;
                return Ok(());
            }
        }
        Err(SessionError::NodeDocumentUnavailable)
    }
    pub(super) fn template_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(usize, TemplateMetadata), SessionError> {
        if !self.html_local(c, node, "template")? {
            return Err(SessionError::WrongToken);
        }
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::Template {
                node: Some(bound),
                state,
            } = payload
                && *bound == node
            {
                c.charge(WorkCategory::RecordVisit, 2)?;
                if self.records[node].template_contents != Some(state.contents)
                    || (!matches!(&self.records[state.contents].owner,
                        Some(HtmlFragmentOwner::Template(owner)) if owner.index == node
                            && Arc::ptr_eq(&owner.identity, &self.identity))
                        && !self.shadow_template_contents(c, node, state.contents)?)
                {
                    return Err(SessionError::TemplateMetadataUnavailable);
                }
                c.charge(
                    WorkCategory::CopyByte,
                    mem::size_of::<TemplateMetadata>() as u64,
                )?;
                return Ok((slot, *state));
            }
        }
        Err(SessionError::TemplateInsertionStateUnavailable)
    }
    /// The cache is a real identity relation, with no fabricated ordinary edge.
    pub(super) fn appropriate_contents_document(
        &mut self,
        c: &mut ParseControl,
        document: DocumentRef,
    ) -> Result<DocumentRef, SessionError> {
        if let DocumentRef::Captured(index) = document {
            return self.captured_contents_document(c, index);
        }
        let document = document.owned()?;
        let slot = self.document_slot(c, document)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let RetainedPayload::Document { state, class, .. } = self.payloads.values[slot] else {
            unreachable!()
        };
        match state {
            DocumentState::InertTemplate => Ok(DocumentRef::Owned(document)),
            DocumentState::Ordinary { inert: Some(inert) } => {
                self.document_slot(c, inert)?;
                Ok(DocumentRef::Owned(inert))
            }
            DocumentState::Ordinary { inert: None } => {
                // Both finite facts are admitted before the native Document is
                // installed. No fallible alias/binding gap follows allocation.
                self.prepare_payload_slot_owned(c)?;
                let fact = self.payloads.values.len();
                self.payloads.values.push(RetainedPayload::Document {
                    node: None,
                    state: DocumentState::InertTemplate,
                    class,
                });
                self.prepare_payload_slot_owned(c)?;
                let binding = self.payloads.values.len();
                self.payloads.values.push(RetainedPayload::NodeDocument {
                    node: None,
                    document: DocumentRef::Owned(self.records.len()),
                });
                payload_write::<[Option<usize>; 2]>(c, 2)?;
                payload_write::<DocumentState>(c, 1)?;
                let node = self.allocate_native(
                    c,
                    HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks),
                    false,
                    0,
                    0,
                )?;
                let RetainedPayload::Document { node: bound, .. } = &mut self.payloads.values[fact]
                else {
                    unreachable!()
                };
                *bound = Some(node.index);
                let RetainedPayload::NodeDocument { node: bound, .. } =
                    &mut self.payloads.values[binding]
                else {
                    unreachable!()
                };
                *bound = Some(node.index);
                let RetainedPayload::Document { state, .. } = &mut self.payloads.values[slot]
                else {
                    unreachable!()
                };
                *state = DocumentState::Ordinary {
                    inert: Some(node.index),
                };
                Ok(DocumentRef::Owned(node.index))
            }
        }
    }
    pub(super) fn allocate_in_document_owned(
        &mut self,
        c: &mut ParseControl,
        kind: HtmlNodeKind,
        template: bool,
        attributes: usize,
        units: usize,
        document: Option<DocumentRef>,
    ) -> Result<ParserNode, SessionError> {
        let prepared = self.prepare_document_allocation(c, document, template)?;
        self.allocate_prepared_owned(c, kind, template, attributes, units, prepared)
    }
    pub(super) fn prepare_document_allocation(
        &mut self,
        c: &mut ParseControl,
        document: Option<DocumentRef>,
        template: bool,
    ) -> Result<Option<PreparedDocuments>, SessionError> {
        let Some(document) = document else {
            if !template {
                return Ok(None);
            }
            self.prepare_payload_slot_owned(c)?;
            let facts = self.payloads.values.len();
            self.payloads.values.push(RetainedPayload::Template {
                node: None,
                state: TemplateMetadata {
                    contents: self.records.len() + 1,
                    target: None,
                    start: None,
                    end: None,
                },
            });
            return Ok(Some(PreparedDocuments {
                binding: None,
                contents_binding: None,
                facts: Some(facts),
                registry: None,
            }));
        };
        self.validate_document_ref(c, document)?;
        let inert_missing = if let DocumentRef::Owned(owned) = document {
            let slot = self.document_slot(c, owned)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            template
                && matches!(
                    self.payloads.values[slot],
                    RetainedPayload::Document {
                        state: DocumentState::Ordinary { inert: None },
                        ..
                    }
                )
        } else {
            false
        };
        let native = 1 + usize::from(template) + usize::from(inert_missing);
        let facts = 1 + 2 * usize::from(template) + 2 * usize::from(inert_missing);
        c.check_acquire_many(&[
            (HtmlParseResource::ManagedNodes, native as u64),
            (HtmlParseResource::ManagedEdges, u64::from(template)),
            (HtmlParseResource::AuxiliaryEntries, (native + facts) as u64),
        ])?;
        c.reserve_vector(&mut self.records, &mut self.records_allocation, native)?;
        c.reserve_vector(
            &mut self.children_allocations,
            &mut self.tickets_allocation,
            native,
        )?;
        c.reserve_vector(
            &mut self.payloads.values,
            &mut self.payloads.allocation,
            facts,
        )?;
        let contents_document = if template {
            Some(self.appropriate_contents_document(c, document)?)
        } else {
            None
        };
        self.prepare_payload_slot_owned(c)?;
        let binding = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::NodeDocument {
            node: None,
            document,
        });
        let contents_binding = if let Some(document) = contents_document {
            self.prepare_payload_slot_owned(c)?;
            let slot = self.payloads.values.len();
            self.payloads.values.push(RetainedPayload::NodeDocument {
                node: None,
                document,
            });
            Some(slot)
        } else {
            None
        };
        let facts = if template {
            self.prepare_payload_slot_owned(c)?;
            let slot = self.payloads.values.len();
            self.payloads.values.push(RetainedPayload::Template {
                node: None,
                state: TemplateMetadata {
                    contents: self.records.len() + 1,
                    target: None,
                    start: None,
                    end: None,
                },
            });
            Some(slot)
        } else {
            None
        };
        Ok(Some(PreparedDocuments {
            binding: Some(binding),
            contents_binding,
            facts,
            registry: None,
        }))
    }
    pub(super) fn allocate_prepared_owned(
        &mut self,
        c: &mut ParseControl,
        kind: HtmlNodeKind,
        template: bool,
        attributes: usize,
        units: usize,
        prepared: Option<PreparedDocuments>,
    ) -> Result<ParserNode, SessionError> {
        let Some(PreparedDocuments {
            binding,
            contents_binding,
            facts,
            registry,
        }) = prepared
        else {
            return self.allocate_native(c, kind, template, attributes, units);
        };
        payload_write::<Option<usize>>(
            c,
            u64::from(binding.is_some())
                + u64::from(contents_binding.is_some())
                + u64::from(facts.is_some())
                + u64::from(registry.is_some()),
        )?;
        if facts.is_some() {
            payload_write::<usize>(c, 1)?;
        }
        let node = self.allocate_native(c, kind, template, attributes, units)?;
        if let Some(slot) = registry {
            let RetainedPayload::LookupRegistry { node: bound, .. } =
                &mut self.payloads.values[slot]
            else {
                unreachable!()
            };
            *bound = Some(node.index);
        }
        if let Some(binding) = binding {
            let RetainedPayload::NodeDocument { node: bound, .. } =
                &mut self.payloads.values[binding]
            else {
                unreachable!()
            };
            *bound = Some(node.index);
        }
        if let Some(slot) = contents_binding {
            let RetainedPayload::NodeDocument { node: bound, .. } = &mut self.payloads.values[slot]
            else {
                unreachable!()
            };
            *bound = Some(node.index + 1);
        }
        if let Some(slot) = facts {
            let RetainedPayload::Template { node: bound, state } = &mut self.payloads.values[slot]
            else {
                unreachable!()
            };
            *bound = Some(node.index);
            state.contents = node.index + 1;
        }
        Ok(node)
    }
}

#[cfg(test)]
impl ParserDocument {
    #[cfg(test)]
    pub(crate) fn node(
        &self,
        c: &mut ParseControl,
        session: &ParserArenaSession,
    ) -> Result<ParserNode, SessionError> {
        let storage = session.active()?;
        let index = storage.index(c, &self.node)?;
        storage.document_slot(c, index)?;
        storage.node(c, index)
    }
}

impl FrozenArena {
    #[cfg(test)]
    pub(crate) fn node_document(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<HtmlNodeId, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        let arena = match &storage.view {
            FrozenView::Document(d) => &d.arena,
            FrozenView::Fragment(f) => &f.arena,
        };
        c.charge(WorkCategory::ProviderCall, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if !Arc::ptr_eq(&node.identity, &arena.identity) || node.index >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        let index = node.index;
        for payload in &storage.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::NodeDocument {
                node: Some(bound),
                document,
            } = payload
                && *bound == index
            {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if !matches!(
                    arena.records.get(document.owned()?).map(|r| &r.kind),
                    Some(HtmlNodeKind::Document(_))
                ) {
                    return Err(SessionError::NotDocument);
                }
                c.charge(WorkCategory::ProviderCall, 1)?;
                payload_write::<HtmlNodeId>(c, 1)?;
                return Ok(HtmlNodeId {
                    identity: arena.identity.clone(),
                    index: document.owned()?,
                });
            }
        }
        Err(SessionError::NodeDocumentUnavailable)
    }
}

impl ParserArenaSession {
    pub(crate) fn is_document_body(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<bool, SessionError> {
        let storage = self.active()?;
        let node = storage.index(c, node)?;
        let document = storage.node_document_ref(c, node)?;
        let DocumentRef::Owned(document) = document else {
            // This newly constructed node is rooted in the output fragment,
            // never a body child in the immutable captured Document.
            return Ok(false);
        };
        for child in &storage.records[document].children {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if storage.html_local(c, *child, "html")? {
                for candidate in &storage.records[*child].children {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if storage.html_local(c, *candidate, "body")?
                        || storage.html_local(c, *candidate, "frameset")?
                    {
                        return Ok(*candidate == node);
                    }
                }
                return Ok(false);
            }
        }
        Ok(false)
    }
    pub(crate) fn prepare_template_patching(
        &mut self,
        c: &mut ParseControl,
        scope: &ParserNode,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<bool, SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active_mut()?;
        let scope = storage.index(c, scope)?;
        storage.container(c, scope)?;
        let node = storage.index(c, node)?;
        let (slot, mut state) = storage.template_slot(c, node)?;
        let name = storage
            .form_attribute(c, node, "for")?
            .ok_or(SessionError::WrongToken)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if name.is_empty() {
            return Ok(false);
        }
        let (start, end) = storage.find_markers(c, scope, name)?;
        let Some(start) = start else { return Ok(false) };
        c.charge(WorkCategory::RecordVisit, 1)?;
        state.target = storage.records[start].parent;
        state.start = Some(start);
        state.end = end;
        payload_write::<TemplateMetadata>(c, 3)?;
        let RetainedPayload::Template { state: stored, .. } = &mut storage.payloads.values[slot]
        else {
            unreachable!()
        };
        *stored = state;
        if Some(start) == end {
            return Ok(true);
        }
        storage.prepare_payload_slot_owned(c)?;
        let list = storage.payloads.values.len();
        storage
            .payloads
            .values
            .push(RetainedPayload::WorkList(Some(OwnedVec::new(
                HtmlParseResource::AuxiliaryEntries,
            )?)));
        let mut next = storage.next_sibling(c, start)?;
        while let Some(current) = next {
            if Some(current) == end {
                break;
            }
            let RetainedPayload::WorkList(Some(nodes)) = &mut storage.payloads.values[list] else {
                unreachable!()
            };
            nodes.try_push(c, current)?;
            payload_write::<Option<usize>>(c, 1)?;
            next = storage.next_sibling(c, current)?;
        }
        storage.start_effect(c, effects::EffectFrame::PatchRemoval { list, cursor: 0 })?;
        storage.drain_construction_effects(c)?;
        Ok(true)
    }
    pub(crate) fn finish_template_patching(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        if let Some(capability) = capability {
            capability.validate_session(c, self)?;
        }
        let storage = self.active_mut()?;
        let index = storage.index(c, node)?;
        let (slot, mut state) = storage.template_slot(c, index)?;
        if state.target.is_none() {
            return Ok(());
        }
        for marker in [state.start, state.end].into_iter().flatten() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if storage.records[marker].parent.is_some() {
                storage.remove_index(c, marker)?;
                storage.drain_construction_effects(c)?;
            }
        }
        state.target = None;
        state.start = None;
        state.end = None;
        payload_write::<TemplateMetadata>(c, 3)?;
        let RetainedPayload::Template { state: stored, .. } = &mut storage.payloads.values[slot]
        else {
            unreachable!()
        };
        *stored = state;
        Ok(())
    }
    pub(crate) fn qualify_creation_context(
        &self,
        c: &mut ParseControl,
        context: &CreationContext<'_>,
    ) -> Result<bool, SessionError> {
        let storage = self.active()?;
        storage.require(c)?;
        match context {
            CreationContext::Ordinary => Ok(false),
            CreationContext::Template(node) => {
                let index = storage.index(c, node)?;
                storage.template_slot(c, index)?;
                Ok(true)
            }
        }
    }
    #[cfg(test)]
    pub(crate) fn node_document(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<ParserDocument, SessionError> {
        let storage = self.active()?;
        let index = storage.index(c, node)?;
        let document = storage.node_document_ref(c, index)?;
        Ok(ParserDocument {
            node: storage.node(c, document.owned()?)?,
        })
    }
    pub(crate) fn template_state(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<TemplateState, SessionError> {
        let storage = self.active()?;
        let index = storage.index(c, node)?;
        let (_, state) = storage.template_slot(c, index)?;
        Ok(TemplateState {
            contents: storage.node(c, state.contents)?,
            target: state.target.map(|n| storage.node(c, n)).transpose()?,
            _start: state.start.map(|n| storage.node(c, n)).transpose()?,
            end: state.end.map(|n| storage.node(c, n)).transpose()?,
        })
    }
}

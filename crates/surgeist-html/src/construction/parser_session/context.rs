//! Context-fragment documents and ready contextual effects in the one session.
//! HTML 877d §13.4; document/cache identity is not inferred from native roots.

use super::*;

/// Compact references are qualified against this session's retained capture.
/// A captured index never indexes the native records or escapes as a native ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DocumentRef {
    Owned(usize),
    Captured(usize),
}
impl DocumentRef {
    pub(super) fn owned(self) -> Result<usize, SessionError> {
        match self {
            Self::Owned(index) => Ok(index),
            Self::Captured(_) => Err(SessionError::CapturedDocument),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum RegistryRef {
    Null,
    Captured(usize),
    Unavailable,
}

/// No-definition DOM creation facts. The saved is slot has its own backing;
/// current attributes and registry reassignment cannot rewrite it.
#[derive(Clone, Copy)]
pub(super) struct ElementCreationState {
    pub(super) state: crate::HtmlCustomElementState,
    pub(super) saved_is: Option<usize>,
}

enum InternalIs<'a> {
    Known(Option<&'a DocumentString>),
    Unavailable,
}

pub(super) struct FragmentContextStorage {
    pub(super) context: crate::HtmlFragmentContext,
    pub(super) mode: crate::HtmlParserScriptingMode,
    pub(super) allow_shadow: bool,
    pub(super) output: Option<usize>,
    pub(super) effects: OwnedVec<ContextEffect>,
    pub(super) source_entries: u64,
}

#[cfg(test)]
pub(crate) enum ParserDocumentReference<N = ParserNode> {
    Owned(N),
    Captured(crate::HtmlCaptureDocumentId),
}

#[derive(Clone, Copy)]
pub(super) struct ShadowState {
    pub(super) keep_registry_null: bool,
    pub(super) declarative: bool,
    pub(super) available_to_internals: bool,
}

pub(crate) enum DeclarativeShadowHost<'a> {
    Owned(&'a ParserNode),
    Captured(&'a crate::HtmlCaptureElementId),
}
pub(super) enum ShadowHostKey {
    Owned(usize),
    Captured(crate::HtmlCaptureElementId),
}
#[cfg(test)]
pub(crate) enum QualifiedFragmentHost<N = ParserNode> {
    Owned(N),
    Captured(crate::HtmlCaptureElementId),
}
pub(crate) struct FragmentBootstrap {
    pub(crate) root: ParserNode,
    pub(crate) output: ParserNode,
}

/// Completed cache association data, not a runnable task or a fake DOM node.
pub(super) enum ContextEffect {
    TemplateCache {
        document: usize,
        inert: Option<usize>,
    },
    AttachShadow {
        host: ShadowHostKey,
        template: usize,
        contents: Option<usize>,
        properties: HtmlShadowRootProperties,
        keep_registry_null: bool,
        declarative: bool,
        available_to_internals: bool,
        source: crate::HtmlSourceRange,
    },
}

impl SessionStorage {
    pub(super) fn captured_contents_document(
        &mut self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<DocumentRef, SessionError> {
        self.validate_document_ref(c, DocumentRef::Captured(document))?;
        let fragment = self
            .fragment
            .as_ref()
            .ok_or(SessionError::ContextUnavailable)?;
        for effect in fragment.effects.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let ContextEffect::TemplateCache {
                document: owner,
                inert,
            } = effect
                && *owner == document
            {
                c.charge(
                    WorkCategory::CopyByte,
                    mem::size_of::<Option<usize>>() as u64,
                )?;
                return inert
                    .map(DocumentRef::Owned)
                    .ok_or(SessionError::ContextUnavailable);
            }
        }
        c.charge(WorkCategory::RecordVisit, 3)?;
        let capture = fragment.context.capture();
        let node = capture
            .document_at(document)
            .map_err(|_| SessionError::ContextUnavailable)?;
        let crate::HtmlCapturedNodeKind::Document(facts) = node.kind() else {
            unreachable!("qualified captured document")
        };
        match facts.template_role() {
            crate::HtmlTemplateDocumentRole::InertTemplate => {
                return Ok(DocumentRef::Captured(document));
            }
            crate::HtmlTemplateDocumentRole::Unavailable => {
                return Err(SessionError::ContextFactUnavailable(
                    crate::HtmlCaptureFact::TemplateDocumentRole,
                ));
            }
            crate::HtmlTemplateDocumentRole::Ordinary => {}
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        match capture
            .template_cache_index(document)
            .map_err(|_| SessionError::ContextUnavailable)?
        {
            crate::capture::CapturedTemplateCacheKey::Document(index) => {
                self.validate_document_ref(c, DocumentRef::Captured(index))?;
                return Ok(DocumentRef::Captured(index));
            }
            crate::capture::CapturedTemplateCacheKey::Unavailable => {
                return Err(SessionError::ContextFactUnavailable(
                    crate::HtmlCaptureFact::TemplateDocumentCache,
                ));
            }
            crate::capture::CapturedTemplateCacheKey::Absent => {}
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        let class = facts.document_class();
        if class == crate::HtmlDocumentClass::Unavailable {
            return Err(SessionError::ContextFactUnavailable(
                crate::HtmlCaptureFact::DocumentClass,
            ));
        }
        // Retain the actual initialized effect slot before any new Document.
        // A later stop owns this unbound slot and refuses publication.
        let effects = &mut self.fragment.as_mut().expect("qualified context").effects;
        let effect = effects.len();
        effects.try_push(
            c,
            ContextEffect::TemplateCache {
                document,
                inert: None,
            },
        )?;
        self.prepare_payload_slot_owned(c)?;
        let fact = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::Document {
            node: None,
            state: template::DocumentState::InertTemplate,
            class,
        });
        self.prepare_payload_slot_owned(c)?;
        let binding = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::NodeDocument {
            node: None,
            document: DocumentRef::Owned(self.records.len()),
        });
        payload_write::<Option<usize>>(c, 3)?;
        let node = self.allocate_native(
            c,
            HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks),
            false,
            0,
            0,
        )?;
        let RetainedPayload::Document { node: bound, .. } = &mut self.payloads.values[fact] else {
            unreachable!()
        };
        *bound = Some(node.index);
        let RetainedPayload::NodeDocument { node: bound, .. } = &mut self.payloads.values[binding]
        else {
            unreachable!()
        };
        *bound = Some(node.index);
        let ContextEffect::TemplateCache { inert, .. } = self
            .fragment
            .as_mut()
            .expect("retained context")
            .effects
            .get_mut(effect, c)?
            .expect("retained cache effect")
        else {
            unreachable!()
        };
        *inert = Some(node.index);
        Ok(DocumentRef::Owned(node.index))
    }
    pub(super) fn validate_document_ref(
        &self,
        c: &mut ParseControl,
        document: DocumentRef,
    ) -> Result<(), SessionError> {
        match document {
            DocumentRef::Owned(index) => {
                self.document_slot(c, index)?;
            }
            DocumentRef::Captured(index) => {
                c.require_identity(&self.control)?;
                c.charge(WorkCategory::RecordVisit, 2)?;
                self.fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context
                    .capture()
                    .document_at(index)
                    .map_err(|_| SessionError::ContextUnavailable)?;
            }
        }
        Ok(())
    }
}

impl ParserArenaSession {
    /// Source-defined parser Document/root/output creation in the actual arena.
    /// The retained capture is evidence, never a mutable imported tree.
    pub(crate) fn initialize_fragment(
        &mut self,
        c: &mut ParseControl,
        request: &crate::HtmlFragmentParseRequest,
    ) -> Result<FragmentBootstrap, SessionError> {
        c.require_source(request.source())?;
        let storage = self.active_mut()?;
        storage.require(c)?;
        c.charge(WorkCategory::RecordVisit, 3)?;
        if storage.fragment.is_some()
            || storage.records.len() != 1
            || !matches!(storage.records[0].kind, HtmlNodeKind::Document(_))
        {
            return Err(SessionError::ContextUnavailable);
        }
        let context = request.context();
        c.charge(WorkCategory::RecordVisit, 4)?;
        let target = context
            .capture()
            .document_index(context.target_document())
            .map_err(|_| SessionError::ContextUnavailable)?;
        context
            .capture()
            .document_index(context.context_document())
            .map_err(|_| SessionError::ContextUnavailable)?;
        let mode = request.effective_scripting_mode();
        if !matches!(
            mode,
            crate::HtmlParserScriptingMode::Inert | crate::HtmlParserScriptingMode::Disabled
        ) {
            return Err(SessionError::ContextUnavailable);
        }
        let mut effects = OwnedVec::new(HtmlParseResource::ContextEffects)?;
        effects.require_owner(c)?;
        // The exact retained context consists of five unconditional Arc-backed
        // handles plus the present nearest-form and registry handles.
        payload_write::<Option<FragmentContextStorage>>(c, 1)?;
        c.charge(
            WorkCategory::ProviderCall,
            5 + u64::from(context.nearest_form().is_some())
                + u64::from(matches!(
                    context.root_registry(),
                    crate::HtmlCapturedRegistry::Registry(_)
                )),
        )?;
        storage.fragment = Some(FragmentContextStorage {
            context: context.clone(),
            mode,
            allow_shadow: request.allow_declarative_shadow_roots(),
            output: None,
            effects,
            source_entries: 0,
        });
        let root = self.allocate_implicit_html(c)?;
        let storage = self.active_mut()?;
        let registry = storage.captured_registry_ref(c, request.context().root_registry())?;
        storage.prepare_payload_slot_owned(c)?;
        storage
            .payloads
            .values
            .push(RetainedPayload::LookupRegistry {
                node: Some(root.index),
                registry,
                creation: Some(ElementCreationState {
                    state: crate::HtmlCustomElementState::Uncustomized,
                    saved_is: None,
                }),
            });
        let document = self.root(c)?;
        self.insert(c, &document, &root, None, None)?;
        let storage = self.active_mut()?;
        let output = storage.allocate_in_document_owned(
            c,
            HtmlNodeKind::Fragment,
            false,
            0,
            0,
            Some(DocumentRef::Captured(target)),
        )?;
        payload_write::<Option<usize>>(c, 1)?;
        storage.fragment.as_mut().expect("retained context").output = Some(output.index);
        payload_write::<FragmentBootstrap>(c, 2)?;
        Ok(FragmentBootstrap { root, output })
    }
}

impl ParserArenaSession {
    pub(crate) fn validate_fragment_proof(
        &self,
        c: &mut ParseControl,
        document: &ParserNode,
    ) -> Result<(), SessionError> {
        self.validate_standalone_proof(c, document)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let fragment = self
            .active()?
            .fragment
            .as_ref()
            .ok_or(SessionError::ContextUnavailable)?;
        if !matches!(
            fragment.mode,
            crate::HtmlParserScriptingMode::Inert | crate::HtmlParserScriptingMode::Disabled
        ) || fragment.output.is_none()
        {
            return Err(c
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance)
                .into());
        }
        Ok(())
    }
    pub(crate) fn fragment_context<'a>(
        &'a self,
        c: &mut ParseControl,
    ) -> Result<Option<&'a crate::HtmlFragmentContext>, SessionError> {
        let storage = self.active()?;
        storage.require(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        Ok(storage.fragment.as_ref().map(|fragment| &fragment.context))
    }
    pub(crate) fn retain_fragment_bootstrap(
        &mut self,
        c: &mut ParseControl,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        capability.validate_fragment_session(c, self)?;
        let storage = self.active_mut()?;
        storage.require(c)?;
        c.charge(WorkCategory::RecordVisit, 4)?;
        let fragment = storage
            .fragment
            .as_ref()
            .ok_or(SessionError::ContextUnavailable)?;
        let output = fragment.output.ok_or(SessionError::ContextUnavailable)?;
        let root = *storage.records[0]
            .children
            .first()
            .ok_or(SessionError::ContextUnavailable)?;
        if output >= storage.records.len()
            || root >= storage.records.len()
            || !matches!(storage.records[output].kind, HtmlNodeKind::Fragment)
        {
            return Err(SessionError::ContextUnavailable);
        }
        let mut root_slot = None;
        let mut output_slot = None;
        for (slot, payload) in storage.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::ConstructionCause {
                node: Some(node), ..
            } = payload
            {
                if *node == root {
                    if root_slot.is_some() {
                        return Err(SessionError::WrongToken);
                    }
                    root_slot = Some(slot);
                } else if *node == output {
                    if output_slot.is_some() {
                        return Err(SessionError::WrongToken);
                    }
                    output_slot = Some(slot);
                }
            }
        }
        let root_slot = root_slot.ok_or(SessionError::WrongToken)?;
        let output_slot = output_slot.ok_or(SessionError::WrongToken)?;
        let evidence = storage.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        let cause = evidence.retain_captured_cause(
            c,
            provenance::CapturedParserTrigger::new(
                &fragment.context,
                provenance::CapturedCauseRole::FragmentBootstrap,
            ),
        )?;
        let copy = evidence.copy_cause_ref(c, &cause)?;
        // Both qualified replacements are admitted before either canonical
        // slot changes. Superseded source-less causes stay in the sole issuer.
        payload_write::<[provenance::ConstructionCauseRef; 2]>(c, 2)?;
        let RetainedPayload::ConstructionCause {
            cause: root_cause, ..
        } = &mut storage.payloads.values[root_slot]
        else {
            unreachable!()
        };
        *root_cause = cause;
        let RetainedPayload::ConstructionCause {
            cause: output_cause,
            ..
        } = &mut storage.payloads.values[output_slot]
        else {
            unreachable!()
        };
        *output_cause = copy;
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn fragment_output(&self, c: &mut ParseControl) -> Result<ParserNode, SessionError> {
        let storage = self.active()?;
        storage.require(c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let index = storage
            .fragment
            .as_ref()
            .and_then(|fragment| fragment.output)
            .ok_or(SessionError::ContextUnavailable)?;
        storage.node(c, index)
    }
}

impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn qualified_node_document(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<ParserDocumentReference, SessionError> {
        let storage = self.active()?;
        let index = storage.index(c, node)?;
        match storage.node_document_ref(c, index)? {
            DocumentRef::Owned(index) => {
                Ok(ParserDocumentReference::Owned(storage.node(c, index)?))
            }
            DocumentRef::Captured(index) => {
                let fragment = storage
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?;
                c.charge(WorkCategory::ProviderCall, 2)?;
                payload_write::<ParserDocumentReference>(c, 1)?;
                let node = fragment
                    .context
                    .capture()
                    .document_at(index)
                    .map_err(|_| SessionError::ContextUnavailable)?;
                let document = fragment
                    .context
                    .capture()
                    .document_id(&node.id())
                    .map_err(|_| SessionError::ContextUnavailable)?;
                Ok(ParserDocumentReference::Captured(document))
            }
        }
    }
}

impl SessionStorage {
    pub(super) fn copy_context_namespace(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(Namespace, Option<usize>), SessionError> {
        c.require_identity(&self.control)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let context = &self
            .fragment
            .as_ref()
            .ok_or(SessionError::ContextUnavailable)?
            .context;
        let node = context
            .capture()
            .node(context.context_element().subject())
            .map_err(|_| SessionError::ContextUnavailable)?;
        let crate::HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Err(SessionError::ContextUnavailable);
        };
        let Some(uri) = element.name().namespace().uri() else {
            return Ok((Namespace::none(), None));
        };
        let mut buffer = Utf16Buffer::new(c)?;
        if let Err(stop) = buffer.extend_units(uri.code_units(), c) {
            buffer.release(c)?;
            return Err(stop.into());
        }
        let frozen = buffer.freeze(c)?;
        // This private path is reached only after actual fragment creation
        // qualification. The canonical evidence owner retains captured URI
        // qualification; it cannot invent a lexical TextOrigin for this copy.
        let previous = if let Some(evidence) = &self.evidence {
            match evidence.current_cause(c) {
                Ok(previous) => previous,
                Err(error) => {
                    frozen.release(c)?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        let group = if let Some(evidence) = &mut self.evidence {
            match evidence.retain_captured_namespace(
                c,
                provenance::CapturedParserTrigger::new(
                    &self
                        .fragment
                        .as_ref()
                        .expect("qualified actual context")
                        .context,
                    provenance::CapturedCauseRole::ContextNamespace,
                ),
                frozen.text(),
            ) {
                Ok(group) => Some(group),
                Err(error) => {
                    frozen.release(c)?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        // Capture qualification belongs to the copied field. The element and
        // subsequent attributes retain the actual triggering lexical cause.
        if let Err(error) = self.end_node_copy(c, previous.as_ref()) {
            frozen.release(c)?;
            return Err(error);
        }
        let (value, ticket) = self.register_group_string_owned(c, frozen, group)?;
        Ok((Namespace::from_uri(value), Some(ticket)))
    }
}

impl SessionStorage {
    fn captured_registry_ref(
        &self,
        c: &mut ParseControl,
        registry: &crate::HtmlCapturedRegistry,
    ) -> Result<RegistryRef, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        match registry {
            crate::HtmlCapturedRegistry::Null => Ok(RegistryRef::Null),
            crate::HtmlCapturedRegistry::Unavailable => Ok(RegistryRef::Unavailable),
            crate::HtmlCapturedRegistry::Registry(id) => {
                let context = &self
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context;
                let index = context
                    .capture()
                    .registry_index(id)
                    .map_err(|_| SessionError::ContextUnavailable)?;
                Ok(RegistryRef::Captured(index))
            }
        }
    }
    pub(super) fn lookup_registry_ref(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<RegistryRef, SessionError> {
        c.require_identity(&self.control)?;
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::LookupRegistry {
                node: Some(bound),
                registry,
                ..
            } = payload
                && *bound == node
            {
                c.charge(WorkCategory::CopyByte, mem::size_of::<RegistryRef>() as u64)?;
                return Ok(*registry);
            }
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        if matches!(self.records[node].kind, HtmlNodeKind::Fragment) {
            return Ok(RegistryRef::Null);
        }
        if self.fragment.is_none() {
            return Ok(RegistryRef::Null);
        }
        Err(SessionError::ContextFactUnavailable(
            crate::HtmlCaptureFact::Registry,
        ))
    }
    pub(super) fn element_creation_state(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<ElementCreationState, SessionError> {
        c.require_identity(&self.control)?;
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::LookupRegistry {
                node: Some(bound),
                creation: Some(state),
                ..
            } = payload
                && *bound == node
            {
                c.charge(
                    WorkCategory::CopyByte,
                    mem::size_of::<ElementCreationState>() as u64,
                )?;
                return Ok(*state);
            }
        }
        Err(SessionError::ContextUnavailable)
    }
    /// Shared DOM create-element qualification for original and clone creation.
    /// A found definition requires a real reaction domain absent from this profile.
    pub(super) fn require_no_definition(
        &self,
        c: &mut ParseControl,
        registry: RegistryRef,
        name: &DocumentString,
        is_value: Option<&DocumentString>,
        html: bool,
    ) -> Result<bool, SessionError> {
        c.require_identity(&self.control)?;
        if matches!(registry, RegistryRef::Unavailable) {
            return Err(SessionError::ContextFactUnavailable(
                crate::HtmlCaptureFact::Registry,
            ));
        }
        // Capture admission guarantees definition names satisfy the custom-name
        // predicate. The first lookup cannot match any other local name, and a
        // second lookup is possible only for a non-null internal is.
        let potential_definition = html && (is_value.is_some() || charged_custom_name(name, c)?);
        if potential_definition && let RegistryRef::Captured(index) = registry {
            let context = &self.fragment.as_ref().expect("qualified fragment").context;
            c.charge(WorkCategory::ProviderCall, 1)?;
            let id = context
                .capture()
                .registry_at(index)
                .map_err(|_| SessionError::ContextUnavailable)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(
                context
                    .capture()
                    .definitions(&id)
                    .map_err(|_| SessionError::ContextUnavailable)?,
                crate::HtmlCapturedDefinitions::Unavailable
            ) {
                return Err(SessionError::ContextFactUnavailable(
                    crate::HtmlCaptureFact::Definitions,
                ));
            }
            let lookup = crate::HtmlCapturedRegistry::Registry(id);
            let mut cursor = context
                .capture()
                .definition_lookup_cursor(&lookup, name, is_value)
                .map_err(|_| {
                    SessionError::ContextFactUnavailable(crate::HtmlCaptureFact::Definitions)
                })?;
            loop {
                let cost = cursor.next_cost();
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(
                    WorkCategory::InspectByte,
                    (cost * mem::size_of::<u16>()) as u64,
                )?;
                if cursor.next_clones_registry() {
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    payload_write::<crate::capture::DefinitionLookupStep>(c, 1)?;
                }
                match cursor.step() {
                    crate::capture::DefinitionLookupStep::Searching => {}
                    crate::capture::DefinitionLookupStep::Ready(found) => {
                        if found.is_some() {
                            // Capture describes definitions, not a constructor
                            // or custom-element reaction execution domain.
                            return Err(SessionError::ContextReactionUnavailable);
                        }
                        break;
                    }
                }
            }
        }
        Ok(potential_definition)
    }
    pub(super) fn prepare_creation_registry(
        &mut self,
        c: &mut ParseControl,
        parent: usize,
        name: &DocumentString,
        is_value: Option<&Text>,
        html: bool,
        prepared: &mut Option<template::PreparedDocuments>,
    ) -> Result<(), SessionError> {
        c.require_identity(&self.control)?;
        if self.fragment.is_none() {
            return Ok(());
        }
        let registry = self.lookup_registry_ref(c, parent)?;
        let potential_definition = self.require_no_definition(
            c,
            registry,
            name,
            is_value.map(|text| text.value.text()),
            html,
        )?;
        let state = if potential_definition {
            crate::HtmlCustomElementState::Undefined
        } else {
            crate::HtmlCustomElementState::Uncustomized
        };
        let saved_is = if let Some(text) = is_value {
            let copy = copy_formatting_text(c, text)?;
            let (value, slot) = self.register_text(c, copy)?;
            drop(value);
            Some(slot)
        } else {
            None
        };
        // The payload slot below admits its complete creation metadata. This
        // separate move writes the actual PreparedDocuments registry receiver.
        payload_write::<Option<usize>>(c, 1)?;
        self.prepare_payload_slot_owned(c)?;
        let slot = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::LookupRegistry {
            node: None,
            registry,
            creation: Some(ElementCreationState { state, saved_is }),
        });
        prepared
            .as_mut()
            .ok_or(SessionError::NodeDocumentUnavailable)?
            .registry = Some(slot);
        Ok(())
    }
    pub(super) fn adopt_registry(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        document: DocumentRef,
        detached: usize,
    ) -> Result<(), SessionError> {
        if self.fragment.is_none() {
            return Ok(());
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        let shadow = if matches!(self.records[node].kind, HtmlNodeKind::Fragment)
            && matches!(
                self.records[node].owner,
                Some(HtmlFragmentOwner::ShadowHost(_))
            ) {
            Some(self.shadow_state(c, node)?)
        } else {
            None
        };
        if shadow.is_none() && !matches!(self.records[node].kind, HtmlNodeKind::Element(_)) {
            return Ok(());
        }
        let previous = self.lookup_registry_ref(c, node)?;
        if matches!(previous, RegistryRef::Null) && shadow.is_some_and(|s| s.keep_registry_null) {
            return Ok(());
        }
        let context = &self.fragment.as_ref().expect("actual context").context;
        let global = match previous {
            RegistryRef::Null => true,
            RegistryRef::Unavailable => {
                return Err(SessionError::ContextFactUnavailable(
                    crate::HtmlCaptureFact::Registry,
                ));
            }
            RegistryRef::Captured(index) => {
                c.charge(WorkCategory::ProviderCall, 1)?;
                let id = context
                    .capture()
                    .registry_at(index)
                    .map_err(|_| SessionError::ContextUnavailable)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                match context
                    .capture()
                    .registry_scope(&id)
                    .map_err(|_| SessionError::ContextUnavailable)?
                {
                    crate::HtmlRegistryScope::Global => true,
                    crate::HtmlRegistryScope::Scoped => false,
                    crate::HtmlRegistryScope::Unavailable => {
                        return Err(SessionError::ContextFactUnavailable(
                            crate::HtmlCaptureFact::RegistryScope,
                        ));
                    }
                }
            }
        };
        if !global {
            return Ok(());
        }
        // Source adoption sees the moved root detached, even when physical
        // order buffers already contain the destination insertion.
        let parent = self.ordinary_parent(c, node, Some(detached))?;
        let registry = match parent {
            Some(parent)
                if matches!(previous, RegistryRef::Null)
                    && !matches!(self.records[parent].kind, HtmlNodeKind::Fragment) =>
            {
                self.lookup_registry_ref(c, parent)?
            }
            _ => match document {
                DocumentRef::Owned(_) => RegistryRef::Null,
                DocumentRef::Captured(index) => {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let context = &self.fragment.as_ref().expect("actual context").context;
                    let document = context
                        .capture()
                        .document_at(index)
                        .map_err(|_| SessionError::ContextUnavailable)?;
                    self.borrowed_captured_registry(c, document)
                }
            },
        };
        let effective = self.effective_global_registry(c, registry)?;
        for slot in 0..self.payloads.values.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(self.payloads.values[slot],RetainedPayload::LookupRegistry {node:Some(bound),..} if bound==node)
            {
                payload_write::<RegistryRef>(c, 1)?;
                let RetainedPayload::LookupRegistry { registry, .. } =
                    &mut self.payloads.values[slot]
                else {
                    unreachable!()
                };
                *registry = effective;
                return Ok(());
            }
        }
        Err(SessionError::ContextFactUnavailable(
            crate::HtmlCaptureFact::Registry,
        ))
    }
}

impl ParserArenaSession {
    /// Availability-qualified lookup facts from the actual canonical record.
    /// Captured registry IDs remain qualified by the retained capture.
    #[cfg(test)]
    pub(crate) fn lookup_registry(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<crate::HtmlCapturedRegistry, SessionError> {
        let storage = self.active()?;
        let index = storage.index(c, node)?;
        match storage.lookup_registry_ref(c, index)? {
            RegistryRef::Null => Ok(crate::HtmlCapturedRegistry::Null),
            RegistryRef::Unavailable => Ok(crate::HtmlCapturedRegistry::Unavailable),
            RegistryRef::Captured(index) => {
                c.charge(WorkCategory::ProviderCall, 1)?;
                payload_write::<crate::HtmlCapturedRegistry>(c, 1)?;
                let context = &storage
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context;
                let id = context
                    .capture()
                    .registry_at(index)
                    .map_err(|_| SessionError::ContextUnavailable)?;
                Ok(crate::HtmlCapturedRegistry::Registry(id))
            }
        }
    }
}

fn charged_custom_name(name: &DocumentString, c: &mut ParseControl) -> Result<bool, SessionError> {
    use crate::custom_names::{CustomNameScanAction as A, validate_custom_element_name_with};
    Ok(validate_custom_element_name_with(name, |action| {
        match action {
            A::Header | A::ReservedHeader => c.charge(WorkCategory::RecordVisit, 1)?,
            A::Unit => c.charge(WorkCategory::InspectByte, 2)?,
            A::ReservedPair => c.charge(WorkCategory::InspectByte, 4)?,
        }
        Ok::<(), ParseStop>(())
    })?
    .is_ok())
}
fn shadow_host_name(
    name: &QualifiedName,
    c: &mut ParseControl,
) -> Result<(bool, bool), SessionError> {
    let Some(uri) = name.namespace().uri() else {
        return Ok((false, false));
    };
    if !exact_ascii(c, Some(uri), "http://www.w3.org/1999/xhtml")? {
        return Ok((false, false));
    }
    let custom = charged_custom_name(name.local_name(), c)?;
    if custom {
        return Ok((true, true));
    }
    for local in [
        "article",
        "aside",
        "blockquote",
        "body",
        "div",
        "footer",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "main",
        "nav",
        "p",
        "section",
        "span",
    ] {
        if exact_ascii(c, Some(name.local_name()), local)? {
            return Ok((true, false));
        }
    }
    Ok((false, false))
}
fn insensitive(
    value: &DocumentString,
    literal: &str,
    c: &mut ParseControl,
) -> Result<bool, SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    if value.len() != literal.len() {
        return Ok(false);
    }
    for (offset, right) in literal.bytes().enumerate() {
        c.charge(WorkCategory::InspectByte, 3)?;
        let left = value.code_units()[offset];
        let left = if matches!(left, 65..=90) {
            left + 32
        } else {
            left
        };
        if left != u16::from(right) {
            return Ok(false);
        }
    }
    Ok(true)
}
impl FragmentContextStorage {
    pub(super) fn release(
        self,
        c: &mut ParseControl,
        owner: &ControlIdentity,
    ) -> Result<(), ParseStop> {
        release_logical(
            c,
            owner,
            HtmlParseResource::SourceMapEntries,
            self.source_entries,
        );
        self.effects.release(c)
    }
}
impl ParserArenaSession {
    pub(crate) fn allows_declarative_shadow(
        &self,
        c: &mut ParseControl,
    ) -> Result<bool, SessionError> {
        let storage = self.active()?;
        storage.require(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        Ok(storage.fragment.as_ref().is_some_and(|f| f.allow_shadow))
    }
    /// Actual stack-only template attachment; fallback preserves the SAME E.
    pub(crate) fn attach_declarative_shadow(
        &mut self,
        c: &mut ParseControl,
        template: &ParserNode,
        host: DeclarativeShadowHost<'_>,
        source: &crate::HtmlSourceRange,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<bool, SessionError> {
        capability.validate_session(c, self)?;
        c.require_source_range(source)?;
        let storage = self.active_mut()?;
        storage.require(c)?;
        let template = storage.index(c, template)?;
        let (template_slot, state) = storage.template_slot(c, template)?;
        let HtmlNodeKind::Element(element) = &storage.records[template].kind else {
            unreachable!()
        };
        let mut mode = None;
        let mut delegates = false;
        let mut clonable = false;
        let mut serializable = false;
        let mut manual = false;
        let mut keep_null = false;
        for attr in element.attributes().iter() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if attr.name().namespace().uri().is_some() {
                continue;
            }
            let name = attr.name().local_name();
            if exact_ascii(c, Some(name), "shadowrootmode")? {
                if insensitive(attr.value(), "open", c)? {
                    mode = Some(crate::HtmlShadowRootMode::Open);
                } else if insensitive(attr.value(), "closed", c)? {
                    mode = Some(crate::HtmlShadowRootMode::Closed);
                }
            } else if exact_ascii(c, Some(name), "shadowrootdelegatesfocus")? {
                delegates = true;
            } else if exact_ascii(c, Some(name), "shadowrootclonable")? {
                clonable = true;
            } else if exact_ascii(c, Some(name), "shadowrootserializable")? {
                serializable = true;
            } else if exact_ascii(c, Some(name), "shadowrootslotassignment")? {
                manual = insensitive(attr.value(), "manual", c)?;
            } else if exact_ascii(c, Some(name), "shadowrootcustomelementregistry")? {
                keep_null = true;
            }
        }
        let Some(mode) = mode else {
            return Ok(false);
        };
        let properties = HtmlShadowRootProperties::new(
            mode,
            if manual {
                crate::HtmlSlotAssignment::Manual
            } else {
                crate::HtmlSlotAssignment::Named
            },
            delegates,
            clonable,
            serializable,
        );
        let (key, document) = match host {
            DeclarativeShadowHost::Owned(host) => {
                let host = storage.index(c, host)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                let HtmlNodeKind::Element(element) = &storage.records[host].kind else {
                    return Ok(false);
                };
                let (eligible, custom) = shadow_host_name(element.name(), c)?;
                if !eligible || storage.records[host].shadow.is_some() {
                    return Ok(false);
                }
                let creation = storage.element_creation_state(c, host)?;
                let saved_is = if let Some(slot) = creation.saved_is {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let RetainedPayload::String { value, .. } = &storage.payloads.values[slot]
                    else {
                        return Err(SessionError::ContextUnavailable);
                    };
                    Some(value)
                } else {
                    None
                };
                if custom || saved_is.is_some() {
                    let registry = storage.lookup_registry_ref(c, host)?;
                    if storage.definition_disables_shadow(
                        c,
                        registry,
                        element.name().local_name(),
                        InternalIs::Known(saved_is),
                        custom,
                    )? {
                        return Ok(false);
                    }
                }
                let document = storage.node_document_ref(c, host)?;
                (ShadowHostKey::Owned(host), document)
            }
            DeclarativeShadowHost::Captured(host) => {
                let context = &storage
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context;
                c.charge(WorkCategory::ProviderCall, 1)?;
                if host != context.context_element() {
                    return Err(SessionError::ContextUnavailable);
                }
                let captured = context
                    .capture()
                    .node(host.subject())
                    .map_err(|_| SessionError::ContextUnavailable)?;
                let crate::HtmlCapturedNodeKind::Element(element) = captured.kind() else {
                    return Err(SessionError::ContextUnavailable);
                };
                let (eligible, custom) = shadow_host_name(element.name(), c)?;
                if !eligible {
                    return Ok(false);
                }
                for effect in storage
                    .fragment
                    .as_ref()
                    .expect("context")
                    .effects
                    .as_slice()
                {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if let ContextEffect::AttachShadow {
                        host: ShadowHostKey::Captured(actual),
                        contents,
                        ..
                    } = effect
                    {
                        c.charge(WorkCategory::ProviderCall, 1)?;
                        if actual == host {
                            if contents.is_none() {
                                return Err(SessionError::ContextUnavailable);
                            }
                            return Ok(false);
                        }
                    }
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                match context
                    .capture()
                    .shadow_presence(host)
                    .map_err(|_| SessionError::ContextUnavailable)?
                {
                    Some(true) => return Ok(false),
                    None => {
                        return Err(SessionError::ContextFactUnavailable(
                            crate::HtmlCaptureFact::ShadowRoot,
                        ));
                    }
                    Some(false) => {}
                }
                if custom || !matches!(element.is_value(), crate::HtmlCapturedIsValue::Absent) {
                    let registry = storage.borrowed_captured_registry(c, captured);
                    if storage.definition_disables_shadow(
                        c,
                        registry,
                        element.name().local_name(),
                        match element.is_value() {
                            crate::HtmlCapturedIsValue::Absent => InternalIs::Known(None),
                            crate::HtmlCapturedIsValue::Value(value) => {
                                InternalIs::Known(Some(value))
                            }
                            crate::HtmlCapturedIsValue::Unavailable => InternalIs::Unavailable,
                        },
                        custom,
                    )? {
                        return Ok(false);
                    }
                }
                c.charge(WorkCategory::ProviderCall, 1)?;
                let document = captured.node_document();
                let document = DocumentRef::Captured(
                    context
                        .capture()
                        .document_index(&document)
                        .map_err(|_| SessionError::ContextUnavailable)?,
                );
                c.charge(WorkCategory::ProviderCall, 1)?;
                (ShadowHostKey::Captured(host.clone()), document)
            }
        };
        let registry = if keep_null {
            RegistryRef::Null
        } else {
            storage.document_registry(c, document)?
        };
        if matches!(registry, RegistryRef::Unavailable) {
            return Err(SessionError::ContextFactUnavailable(
                crate::HtmlCaptureFact::Registry,
            ));
        }
        // Slot/trigger ownership is admitted before allocating the shadow root.
        // Any later stop leaves all records and this unbound effect cleanup-owned.
        c.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<u64>(c, 1)?;
        c.acquire(HtmlParseResource::SourceMapEntries, 1)?;
        let fragment = storage
            .fragment
            .as_mut()
            .ok_or(SessionError::ContextUnavailable)?;
        let slot = fragment.effects.len();
        let effect = ContextEffect::AttachShadow {
            host: key,
            template,
            contents: None,
            properties,
            keep_registry_null: false,
            declarative: false,
            available_to_internals: false,
            source: source.clone(),
        };
        if let Err((stop, _)) = fragment.effects.try_push_recover(c, effect) {
            release_logical(c, &storage.control, HtmlParseResource::SourceMapEntries, 1);
            return Err(stop.into());
        }
        fragment.source_entries += 1;
        let result = (|| {
            storage.prepare_payload_slot_owned(c)?;
            let shadow_slot = storage.payloads.values.len();
            storage.payloads.values.push(RetainedPayload::Shadow {
                root: None,
                state: ShadowState {
                    keep_registry_null: keep_null,
                    declarative: true,
                    available_to_internals: true,
                },
            });
            payload_write::<Option<usize>>(c, 1)?;
            let root = storage.allocate_in_document_owned(
                c,
                HtmlNodeKind::Fragment,
                false,
                0,
                0,
                Some(document),
            )?;
            let RetainedPayload::Shadow { root: bound, .. } =
                &mut storage.payloads.values[shadow_slot]
            else {
                unreachable!()
            };
            *bound = Some(root.index);
            storage.prepare_payload_slot_owned(c)?;
            storage
                .payloads
                .values
                .push(RetainedPayload::LookupRegistry {
                    node: Some(root.index),
                    registry,
                    creation: None,
                });
            let local_host = match &storage
                .fragment
                .as_ref()
                .expect("context")
                .effects
                .as_slice()[slot]
            {
                ContextEffect::AttachShadow {
                    host: ShadowHostKey::Owned(host),
                    ..
                } => Some(*host),
                _ => None,
            };
            let depth = if let Some(host) = local_host {
                storage.depth(c, host, None)? + 1
            } else {
                0
            };
            c.observe_tree_depth(storage.usage.depth.max(depth) as u64)?;
            if local_host.is_some() {
                c.charge(WorkCategory::ProviderCall, 3)?;
            }
            if local_host.is_some() {
                payload_write::<Option<HtmlDeclarativeShadowRoot>>(c, 1)?;
            }
            payload_write::<Option<HtmlFragmentOwner>>(c, 1 + u64::from(local_host.is_some()))?;
            payload_write::<Option<usize>>(c, 2)?;
            payload_write::<usize>(c, 3 + u64::from(local_host.is_none()))?;
            payload_write::<[bool; 3]>(c, 3)?;
            // Preflight the real effect receiver before the fixed native commit.
            let effect = storage
                .fragment
                .as_mut()
                .expect("context")
                .effects
                .get_mut(slot, c)?
                .expect("owned effect");
            let ContextEffect::AttachShadow {
                contents,
                declarative,
                available_to_internals,
                keep_registry_null,
                ..
            } = effect
            else {
                unreachable!()
            };
            if local_host.is_none() {
                c.release(HtmlParseResource::ManagedEdges, 1)?;
            }
            *contents = Some(root.index);
            *declarative = true;
            *available_to_internals = true;
            *keep_registry_null = keep_null;
            storage.records[state.contents].owner = None;
            storage.records[template].template_contents = Some(root.index);
            let RetainedPayload::Template { state, .. } =
                &mut storage.payloads.values[template_slot]
            else {
                unreachable!()
            };
            state.contents = root.index;
            if let Some(host) = local_host {
                storage.records[root.index].owner =
                    Some(HtmlFragmentOwner::ShadowHost(HtmlNodeId {
                        identity: storage.identity.clone(),
                        index: host,
                    }));
                storage.records[host].shadow = Some(HtmlDeclarativeShadowRoot {
                    host: HtmlNodeId {
                        identity: storage.identity.clone(),
                        index: host,
                    },
                    contents: HtmlNodeId {
                        identity: storage.identity.clone(),
                        index: root.index,
                    },
                    properties,
                });
            } else {
                storage.usage.edges -= 1;
            }
            storage.records[root.index].depth = depth;
            storage.usage.depth = storage.usage.depth.max(depth);
            Ok::<(), SessionError>(())
        })();
        if let Err(error) = result {
            c.stop_unsupported_standalone(StandaloneUnsupported::Provenance);
            return Err(error);
        }
        Ok(true)
    }
}
impl SessionStorage {
    fn borrowed_captured_registry(
        &self,
        _c: &mut ParseControl,
        node: crate::HtmlCapturedNode<'_>,
    ) -> RegistryRef {
        // The caller already qualified this node through this retained capture.
        // No Arc-backed registry handle is cloned by the fixed borrowed read.
        match node.lookup_registry_index() {
            Some(Some(index)) => RegistryRef::Captured(index),
            Some(None) => RegistryRef::Null,
            None => RegistryRef::Unavailable,
        }
    }
    fn document_registry(
        &self,
        c: &mut ParseControl,
        document: DocumentRef,
    ) -> Result<RegistryRef, SessionError> {
        self.validate_document_ref(c, document)?;
        match document {
            DocumentRef::Owned(_) => Ok(RegistryRef::Null),
            DocumentRef::Captured(index) => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let context = &self
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context;
                let document = context
                    .capture()
                    .document_at(index)
                    .map_err(|_| SessionError::ContextUnavailable)?;
                Ok(self.borrowed_captured_registry(c, document))
            }
        }
    }
    fn definition_disables_shadow(
        &self,
        c: &mut ParseControl,
        registry: RegistryRef,
        local: &DocumentString,
        is_value: InternalIs<'_>,
        custom: bool,
    ) -> Result<bool, SessionError> {
        let RegistryRef::Captured(index) = registry else {
            return if matches!(registry, RegistryRef::Unavailable) {
                Err(SessionError::ContextFactUnavailable(
                    crate::HtmlCaptureFact::Registry,
                ))
            } else {
                Ok(false)
            };
        };
        let is_value = if custom {
            None
        } else {
            match is_value {
                InternalIs::Known(None) => return Ok(false),
                InternalIs::Unavailable => {
                    return Err(SessionError::ContextFactUnavailable(
                        crate::HtmlCaptureFact::IsValue,
                    ));
                }
                InternalIs::Known(Some(value)) => Some(value),
            }
        };
        let context = &self
            .fragment
            .as_ref()
            .ok_or(SessionError::ContextUnavailable)?
            .context;
        c.charge(WorkCategory::ProviderCall, 1)?;
        let id = context
            .capture()
            .registry_at(index)
            .map_err(|_| SessionError::ContextUnavailable)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if matches!(
            context
                .capture()
                .definitions(&id)
                .map_err(|_| SessionError::ContextUnavailable)?,
            crate::HtmlCapturedDefinitions::Unavailable
        ) {
            return Err(SessionError::ContextFactUnavailable(
                crate::HtmlCaptureFact::Definitions,
            ));
        }
        let registry = crate::HtmlCapturedRegistry::Registry(id);
        let mut cursor = context
            .capture()
            .definition_lookup_cursor(&registry, local, is_value)
            .map_err(|_| {
                SessionError::ContextFactUnavailable(crate::HtmlCaptureFact::Definitions)
            })?;
        loop {
            let cost = cursor.next_cost();
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, (cost * 2) as u64)?;
            if cursor.next_clones_registry() {
                c.charge(WorkCategory::ProviderCall, 1)?;
                payload_write::<crate::capture::DefinitionLookupStep>(c, 1)?;
            }
            match cursor.step() {
                crate::capture::DefinitionLookupStep::Searching => {}
                crate::capture::DefinitionLookupStep::Ready(None) => return Ok(false),
                crate::capture::DefinitionLookupStep::Ready(Some(id)) => {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    return Ok(context
                        .capture()
                        .definition(&id)
                        .map_err(|_| SessionError::ContextUnavailable)?
                        .disable_shadow());
                }
            }
        }
    }
    pub(super) fn shadow_template_contents(
        &self,
        c: &mut ParseControl,
        template: usize,
        contents: usize,
    ) -> Result<bool, SessionError> {
        let Some(fragment) = &self.fragment else {
            return Ok(false);
        };
        for effect in fragment.effects.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(effect,ContextEffect::AttachShadow{template:actual,contents:Some(root),..}
                if *actual==template && *root==contents)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
pub(crate) struct DeclarativeShadowState<'a, N = ParserNode> {
    pub(crate) host: QualifiedFragmentHost<N>,
    pub(crate) template: N,
    pub(crate) contents: N,
    pub(crate) properties: HtmlShadowRootProperties,
    pub(crate) registry: crate::HtmlCapturedRegistry,
    pub(crate) keep_registry_null: bool,
    pub(crate) declarative: bool,
    pub(crate) available_to_internals: bool,
    pub(crate) source: &'a crate::HtmlSourceRange,
}
impl SessionStorage {
    pub(super) fn require_context_ready(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        let Some(fragment) = &self.fragment else {
            return Ok(());
        };
        for effect in fragment.effects.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(
                effect,
                ContextEffect::TemplateCache { inert: None, .. }
                    | ContextEffect::AttachShadow { contents: None, .. }
            ) {
                c.stop_unsupported_standalone(StandaloneUnsupported::Provenance);
                return Err(SessionError::ContextUnavailable);
            }
        }
        Ok(())
    }
    #[cfg(test)]
    fn qualified_registry(
        &self,
        c: &mut ParseControl,
        registry: RegistryRef,
    ) -> Result<crate::HtmlCapturedRegistry, SessionError> {
        Ok(match registry {
            RegistryRef::Null => crate::HtmlCapturedRegistry::Null,
            RegistryRef::Unavailable => crate::HtmlCapturedRegistry::Unavailable,
            RegistryRef::Captured(index) => {
                c.charge(WorkCategory::ProviderCall, 1)?;
                payload_write::<crate::HtmlCapturedRegistry>(c, 1)?;
                let context = &self
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context;
                crate::HtmlCapturedRegistry::Registry(
                    context
                        .capture()
                        .registry_at(index)
                        .map_err(|_| SessionError::ContextUnavailable)?,
                )
            }
        })
    }
}
impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn declarative_shadow<'a>(
        &'a self,
        c: &mut ParseControl,
        ordinal: usize,
    ) -> Result<Option<DeclarativeShadowState<'a>>, SessionError> {
        let storage = self.active()?;
        storage.require(c)?;
        let Some(fragment) = &storage.fragment else {
            return Ok(None);
        };
        let mut current = 0;
        for effect in fragment.effects.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let ContextEffect::AttachShadow {
                host,
                template,
                contents: Some(contents),
                properties,
                keep_registry_null,
                declarative,
                available_to_internals,
                source,
            } = effect
            else {
                continue;
            };
            if current != ordinal {
                current += 1;
                continue;
            }
            let host = match host {
                ShadowHostKey::Owned(index) => {
                    QualifiedFragmentHost::Owned(storage.node(c, *index)?)
                }
                ShadowHostKey::Captured(host) => {
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    payload_write::<QualifiedFragmentHost>(c, 1)?;
                    QualifiedFragmentHost::Captured(host.clone())
                }
            };
            let template = storage.node(c, *template)?;
            let registry = storage.lookup_registry_ref(c, *contents)?;
            let contents = storage.node(c, *contents)?;
            let registry = storage.qualified_registry(c, registry)?;
            payload_write::<DeclarativeShadowState>(c, 1)?;
            return Ok(Some(DeclarativeShadowState {
                host,
                template,
                contents,
                properties: *properties,
                registry,
                keep_registry_null: *keep_registry_null,
                declarative: *declarative,
                available_to_internals: *available_to_internals,
                source,
            }));
        }
        Ok(None)
    }
}

impl SessionStorage {
    fn effective_global_registry(
        &self,
        c: &mut ParseControl,
        registry: RegistryRef,
    ) -> Result<RegistryRef, SessionError> {
        Ok(match registry {
            RegistryRef::Captured(index) => {
                c.charge(WorkCategory::ProviderCall, 1)?;
                let context = &self.fragment.as_ref().expect("actual context").context;
                let id = context
                    .capture()
                    .registry_at(index)
                    .map_err(|_| SessionError::ContextUnavailable)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                match context
                    .capture()
                    .registry_scope(&id)
                    .map_err(|_| SessionError::ContextUnavailable)?
                {
                    crate::HtmlRegistryScope::Global => registry,
                    crate::HtmlRegistryScope::Scoped => RegistryRef::Null,
                    crate::HtmlRegistryScope::Unavailable => {
                        return Err(SessionError::ContextFactUnavailable(
                            crate::HtmlCaptureFact::RegistryScope,
                        ));
                    }
                }
            }
            RegistryRef::Unavailable => {
                return Err(SessionError::ContextFactUnavailable(
                    crate::HtmlCaptureFact::Registry,
                ));
            }
            RegistryRef::Null => registry,
        })
    }
    pub(super) fn shadow_state(
        &self,
        c: &mut ParseControl,
        root: usize,
    ) -> Result<ShadowState, SessionError> {
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::Shadow {
                root: Some(actual),
                state,
            } = payload
                && *actual == root
            {
                c.charge(WorkCategory::CopyByte, mem::size_of::<ShadowState>() as u64)?;
                return Ok(*state);
            }
        }
        Err(SessionError::ContextUnavailable)
    }
    /// DOM clone step6, reached after the complete light-child traversal. The
    /// clone root and every field are actual same-owner native records.
    pub(super) fn clone_shadow(
        &mut self,
        c: &mut ParseControl,
        source: usize,
        copy: usize,
        document: DocumentRef,
    ) -> Result<Option<(usize, usize)>, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Some(shadow) = &self.records[source].shadow else {
            return Ok(None);
        };
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<HtmlShadowRootProperties>() as u64,
        )?;
        if !shadow.properties.clonable() {
            return Ok(None);
        }
        c.charge(
            WorkCategory::CopyByte,
            mem::size_of::<HtmlShadowRootProperties>() as u64 + mem::size_of::<usize>() as u64,
        )?;
        let properties = shadow.properties;
        let original = shadow.contents.index;
        let state = self.shadow_state(c, original)?;
        let previous = self.lookup_registry_ref(c, original)?;
        let registry = match previous {
            RegistryRef::Captured(index) => {
                let fragment = self
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?;
                c.charge(WorkCategory::ProviderCall, 1)?;
                let id = fragment
                    .context
                    .capture()
                    .registry_at(index)
                    .map_err(|_| SessionError::ContextUnavailable)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                match fragment
                    .context
                    .capture()
                    .registry_scope(&id)
                    .map_err(|_| SessionError::ContextUnavailable)?
                {
                    crate::HtmlRegistryScope::Global => {
                        let registry = self.document_registry(c, document)?;
                        self.effective_global_registry(c, registry)?
                    }
                    crate::HtmlRegistryScope::Scoped => previous,
                    crate::HtmlRegistryScope::Unavailable => {
                        return Err(SessionError::ContextFactUnavailable(
                            crate::HtmlCaptureFact::RegistryScope,
                        ));
                    }
                }
            }
            RegistryRef::Null => RegistryRef::Null,
            RegistryRef::Unavailable => {
                return Err(SessionError::ContextFactUnavailable(
                    crate::HtmlCaptureFact::Registry,
                ));
            }
        };
        self.prepare_payload_slot_owned(c)?;
        let fact = self.payloads.values.len();
        // Attach creates false available-to-internals for these actual
        // uncustomized same-owner clone producers; DOM clone copies only the
        // declarative and keep-null flags after attachment.
        let state = ShadowState {
            declarative: state.declarative,
            keep_registry_null: state.keep_registry_null,
            available_to_internals: false,
        };
        self.payloads
            .values
            .push(RetainedPayload::Shadow { root: None, state });
        self.prepare_payload_slot_owned(c)?;
        let lookup = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::LookupRegistry {
            node: None,
            registry,
            creation: None,
        });
        payload_write::<Option<usize>>(c, 2)?;
        let root = self.allocate_in_document_owned(
            c,
            HtmlNodeKind::Fragment,
            false,
            0,
            0,
            Some(document),
        )?;
        let RetainedPayload::Shadow { root: bound, .. } = &mut self.payloads.values[fact] else {
            unreachable!()
        };
        *bound = Some(root.index);
        let RetainedPayload::LookupRegistry { node: bound, .. } = &mut self.payloads.values[lookup]
        else {
            unreachable!()
        };
        *bound = Some(root.index);
        let depth = self.depth(c, copy, None)? + 1;
        c.observe_tree_depth(self.usage.depth.max(depth) as u64)?;
        c.charge(WorkCategory::ProviderCall, 3)?;
        payload_write::<Option<HtmlFragmentOwner>>(c, 1)?;
        payload_write::<Option<HtmlDeclarativeShadowRoot>>(c, 1)?;
        payload_write::<usize>(c, 3)?;
        c.acquire(HtmlParseResource::ManagedEdges, 1)?;
        self.records[root.index].owner = Some(HtmlFragmentOwner::ShadowHost(HtmlNodeId {
            identity: self.identity.clone(),
            index: copy,
        }));
        self.records[copy].shadow = Some(HtmlDeclarativeShadowRoot {
            host: HtmlNodeId {
                identity: self.identity.clone(),
                index: copy,
            },
            contents: HtmlNodeId {
                identity: self.identity.clone(),
                index: root.index,
            },
            properties,
        });
        self.records[root.index].depth = depth;
        self.usage.depth = self.usage.depth.max(depth);
        self.usage.edges += 1;
        Ok(Some((original, root.index)))
    }
}

#[cfg(test)]
pub(crate) struct TemplateCacheState<N = ParserNode> {
    pub(crate) document: crate::HtmlCaptureDocumentId,
    pub(crate) inert: N,
}

impl FrozenStorage {
    #[cfg(test)]
    fn context_arena<'a>(&'a self, c: &mut ParseControl) -> Result<&'a Arena, SessionError> {
        c.require_identity(&self.control)?;
        Ok(match &self.view {
            FrozenView::Document(document) => &document.arena,
            FrozenView::Fragment(fragment) => &fragment.arena,
        })
    }
    #[cfg(test)]
    fn context_index(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<usize, SessionError> {
        let arena = self.context_arena(c)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if !Arc::ptr_eq(&node.identity, &arena.identity) || node.index >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        Ok(node.index)
    }
    #[cfg(test)]
    fn context_node(&self, c: &mut ParseControl, index: usize) -> Result<HtmlNodeId, SessionError> {
        let arena = self.context_arena(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if index >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        c.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<HtmlNodeId>(c, 1)?;
        Ok(HtmlNodeId {
            identity: arena.identity.clone(),
            index,
        })
    }
    #[cfg(test)]
    fn context_registry(
        &self,
        c: &mut ParseControl,
        registry: RegistryRef,
    ) -> Result<crate::HtmlCapturedRegistry, SessionError> {
        self.context_arena(c)?;
        payload_write::<crate::HtmlCapturedRegistry>(c, 1)?;
        Ok(match registry {
            RegistryRef::Null => crate::HtmlCapturedRegistry::Null,
            RegistryRef::Unavailable => crate::HtmlCapturedRegistry::Unavailable,
            RegistryRef::Captured(index) => {
                let context = &self
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context;
                c.charge(WorkCategory::ProviderCall, 1)?;
                crate::HtmlCapturedRegistry::Registry(
                    context
                        .capture()
                        .registry_at(index)
                        .map_err(|_| SessionError::ContextUnavailable)?,
                )
            }
        })
    }
}
impl FrozenStorage {
    #[cfg(test)]
    fn context_lookup_registry(
        &self,
        c: &mut ParseControl,
        index: usize,
    ) -> Result<crate::HtmlCapturedRegistry, SessionError> {
        self.context_arena(c)?;
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::LookupRegistry {
                node: Some(bound),
                registry,
                ..
            } = payload
                && *bound == index
            {
                return self.context_registry(c, *registry);
            }
        }
        Err(SessionError::ContextFactUnavailable(
            crate::HtmlCaptureFact::Registry,
        ))
    }
}

impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn element_creation(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(crate::HtmlCustomElementState, Option<&DocumentString>), SessionError> {
        let storage = self.active()?;
        storage.require(c)?;
        let index = storage.index(c, node)?;
        let state = storage.element_creation_state(c, index)?;
        let saved = if let Some(slot) = state.saved_is {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let RetainedPayload::String { value, .. } = &storage.payloads.values[slot] else {
                return Err(SessionError::ContextUnavailable);
            };
            Some(value)
        } else {
            None
        };
        Ok((state.state, saved))
    }
}

impl FrozenArena {
    pub(crate) fn frozen_fragment_context(
        &self,
    ) -> Result<&crate::HtmlFragmentContext, SessionError> {
        Ok(&self
            .storage
            .as_ref()
            .ok_or(SessionError::Consumed)?
            .fragment
            .as_ref()
            .ok_or(SessionError::ContextUnavailable)?
            .context)
    }

    #[cfg(test)]
    pub(crate) fn element_creation(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<(crate::HtmlCustomElementState, Option<&DocumentString>), SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let index = storage.context_index(c, node)?;
        for payload in &storage.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::LookupRegistry {
                node: Some(bound),
                creation: Some(state),
                ..
            } = payload
                && *bound == index
            {
                c.charge(
                    WorkCategory::CopyByte,
                    mem::size_of::<ElementCreationState>() as u64,
                )?;
                let saved = if let Some(slot) = state.saved_is {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let RetainedPayload::String { value, .. } = &storage.payloads.values[slot]
                    else {
                        return Err(SessionError::ContextUnavailable);
                    };
                    Some(value)
                } else {
                    None
                };
                return Ok((state.state, saved));
            }
        }
        Err(SessionError::ContextUnavailable)
    }
}
impl FrozenArena {
    #[cfg(test)]
    pub(crate) fn document_class(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<crate::HtmlDocumentClass, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let index = storage.context_index(c, node)?;
        let arena = storage.context_arena(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if !matches!(arena.records[index].kind, HtmlNodeKind::Document(_)) {
            return Err(SessionError::NotDocument);
        }
        for payload in &storage.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::Document {
                node: Some(bound),
                class,
                ..
            } = payload
                && *bound == index
            {
                c.charge(
                    WorkCategory::CopyByte,
                    mem::size_of::<crate::HtmlDocumentClass>() as u64,
                )?;
                return Ok(*class);
            }
        }
        Err(SessionError::NodeDocumentUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn qualified_node_document(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<ParserDocumentReference<HtmlNodeId>, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let index = storage.context_index(c, node)?;
        for payload in &storage.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::NodeDocument {
                node: Some(bound),
                document,
            } = payload
                && *bound == index
            {
                payload_write::<ParserDocumentReference<HtmlNodeId>>(c, 1)?;
                return Ok(match document {
                    DocumentRef::Owned(index) => {
                        let arena = storage.context_arena(c)?;
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if !matches!(
                            arena.records.get(*index).map(|r| &r.kind),
                            Some(HtmlNodeKind::Document(_))
                        ) {
                            return Err(SessionError::NotDocument);
                        }
                        ParserDocumentReference::Owned(storage.context_node(c, *index)?)
                    }
                    DocumentRef::Captured(index) => {
                        let context = &storage
                            .fragment
                            .as_ref()
                            .ok_or(SessionError::ContextUnavailable)?
                            .context;
                        c.charge(WorkCategory::ProviderCall, 1)?;
                        ParserDocumentReference::Captured(
                            context
                                .capture()
                                .document_at(*index)
                                .map_err(|_| SessionError::ContextUnavailable)?
                                .node_document(),
                        )
                    }
                });
            }
        }
        Err(SessionError::NodeDocumentUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn lookup_registry(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<crate::HtmlCapturedRegistry, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let index = storage.context_index(c, node)?;
        storage.context_lookup_registry(c, index)
    }
    #[cfg(test)]
    pub(crate) fn template_cache(
        &self,
        c: &mut ParseControl,
        ordinal: usize,
    ) -> Result<Option<TemplateCacheState<HtmlNodeId>>, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        storage.context_arena(c)?;
        let Some(fragment) = &storage.fragment else {
            return Ok(None);
        };
        let mut current = 0;
        for effect in fragment.effects.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let ContextEffect::TemplateCache {
                document,
                inert: Some(inert),
            } = effect
            else {
                continue;
            };
            if current != ordinal {
                current += 1;
                continue;
            }
            c.charge(WorkCategory::ProviderCall, 1)?;
            payload_write::<TemplateCacheState<HtmlNodeId>>(c, 1)?;
            let document = fragment
                .context
                .capture()
                .document_at(*document)
                .map_err(|_| SessionError::ContextUnavailable)?
                .node_document();
            return Ok(Some(TemplateCacheState {
                document,
                inert: storage.context_node(c, *inert)?,
            }));
        }
        Ok(None)
    }
    #[cfg(test)]
    pub(crate) fn declarative_shadow<'a>(
        &'a self,
        c: &mut ParseControl,
        ordinal: usize,
    ) -> Result<Option<DeclarativeShadowState<'a, HtmlNodeId>>, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        storage.context_arena(c)?;
        let Some(fragment) = &storage.fragment else {
            return Ok(None);
        };
        let mut current = 0;
        for effect in fragment.effects.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let ContextEffect::AttachShadow {
                host,
                template,
                contents: Some(contents),
                properties,
                keep_registry_null,
                declarative,
                available_to_internals,
                source,
            } = effect
            else {
                continue;
            };
            if current != ordinal {
                current += 1;
                continue;
            }
            payload_write::<DeclarativeShadowState<HtmlNodeId>>(c, 1)?;
            let host = match host {
                ShadowHostKey::Owned(index) => {
                    QualifiedFragmentHost::Owned(storage.context_node(c, *index)?)
                }
                ShadowHostKey::Captured(host) => {
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    payload_write::<QualifiedFragmentHost<HtmlNodeId>>(c, 1)?;
                    QualifiedFragmentHost::Captured(host.clone())
                }
            };
            return Ok(Some(DeclarativeShadowState {
                host,
                template: storage.context_node(c, *template)?,
                contents: storage.context_node(c, *contents)?,
                properties: *properties,
                registry: storage.context_lookup_registry(c, *contents)?,
                keep_registry_null: *keep_registry_null,
                declarative: *declarative,
                available_to_internals: *available_to_internals,
                source,
            }));
        }
        Ok(None)
    }
}
// Fixed borrowed views over the actual frozen records. These are immutable
// observations, not controller operations or a copied publication registry.
#[cfg(test)]
pub(crate) fn registry_binding_layout() -> usize {
    mem::size_of::<usize>() + mem::size_of::<RegistryRef>()
}
pub struct FrozenFragmentEffects<'a> {
    storage: &'a FrozenStorage,
    arena: &'a Arena,
    fragment: &'a FragmentContextStorage,
}
pub enum FrozenFragmentHost<'a> {
    Owned(HtmlNode<'a>),
    Captured(&'a crate::HtmlCaptureElementId),
}
pub enum FrozenFragmentEffect<'a> {
    TemplateCache {
        document: crate::HtmlCapturedNode<'a>,
        inert: HtmlNode<'a>,
    },
    AttachShadow {
        host: FrozenFragmentHost<'a>,
        template: HtmlNode<'a>,
        contents: HtmlNode<'a>,
        properties: &'a HtmlShadowRootProperties,
        keep_registry_null: bool,
        declarative: bool,
        available_to_internals: bool,
        source: &'a crate::HtmlSourceRange,
    },
}
pub struct FrozenCapturedRegistry<'a> {
    capture: &'a crate::HtmlCapture,
    index: usize,
}
impl FrozenCapturedRegistry<'_> {
    pub fn is_registry(&self, id: &crate::HtmlCaptureRegistryId) -> bool {
        self.capture
            .registry_index(id)
            .is_ok_and(|index| index == self.index)
    }
}
pub enum FrozenFragmentRegistry<'a> {
    Null,
    Unavailable,
    Captured(FrozenCapturedRegistry<'a>),
}
pub struct FrozenFragmentRegistryBinding<'a> {
    pub(crate) node: HtmlNode<'a>,
    pub(crate) registry: FrozenFragmentRegistry<'a>,
}
impl<'a> FrozenFragmentEffects<'a> {
    pub fn len(&self) -> usize {
        self.fragment.effects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn registry_slots(&self) -> usize {
        self.storage.payloads.values.len()
    }
    fn node(&self, index: usize) -> Result<HtmlNode<'a>, SessionError> {
        self.arena
            .records
            .get(index)
            .ok_or(SessionError::ForeignNode)?;
        Ok(HtmlNode {
            arena: self.arena,
            index,
        })
    }
    /// Exactly one actual effect slot; there is no hidden ordinal scan.
    pub(crate) fn get(
        &self,
        index: usize,
    ) -> Result<Option<FrozenFragmentEffect<'a>>, SessionError> {
        let Some(effect) = self.fragment.effects.as_slice().get(index) else {
            return Ok(None);
        };
        Ok(Some(match effect {
            ContextEffect::TemplateCache {
                document,
                inert: Some(inert),
            } => FrozenFragmentEffect::TemplateCache {
                document: self
                    .fragment
                    .context
                    .capture()
                    .document_at(*document)
                    .map_err(|_| SessionError::ContextUnavailable)?,
                inert: self.node(*inert)?,
            },
            ContextEffect::AttachShadow {
                host,
                template,
                contents: Some(contents),
                properties,
                keep_registry_null,
                declarative,
                available_to_internals,
                source,
            } => FrozenFragmentEffect::AttachShadow {
                host: match host {
                    ShadowHostKey::Owned(index) => FrozenFragmentHost::Owned(self.node(*index)?),
                    ShadowHostKey::Captured(host) => FrozenFragmentHost::Captured(host),
                },
                template: self.node(*template)?,
                contents: self.node(*contents)?,
                properties,
                keep_registry_null: *keep_registry_null,
                declarative: *declarative,
                available_to_internals: *available_to_internals,
                source,
            },
            _ => return Err(SessionError::ContextUnavailable),
        }))
    }
    /// Exactly one canonical payload slot. The reader chooses its next index;
    /// the lender never scans, clones a registry handle or duplicates facts.
    pub(crate) fn registry_binding(
        &self,
        index: usize,
    ) -> Result<Option<FrozenFragmentRegistryBinding<'a>>, SessionError> {
        let Some(RetainedPayload::LookupRegistry {
            node: Some(node),
            registry,
            ..
        }) = self.storage.payloads.values.get(index)
        else {
            return Ok(None);
        };
        Ok(Some(FrozenFragmentRegistryBinding {
            node: self.node(*node)?,
            registry: match registry {
                RegistryRef::Null => FrozenFragmentRegistry::Null,
                RegistryRef::Unavailable => FrozenFragmentRegistry::Unavailable,
                RegistryRef::Captured(index) => {
                    FrozenFragmentRegistry::Captured(FrozenCapturedRegistry {
                        capture: self.fragment.context.capture(),
                        index: *index,
                    })
                }
            },
        }))
    }
}
impl FrozenArena {
    pub(crate) fn frozen_fragment_effects(
        &self,
    ) -> Result<FrozenFragmentEffects<'_>, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = match &storage.view {
            FrozenView::Document(document) => &document.arena,
            FrozenView::Fragment(fragment) => &fragment.arena,
        };
        let fragment = storage
            .fragment
            .as_ref()
            .ok_or(SessionError::ContextUnavailable)?;
        Ok(FrozenFragmentEffects {
            storage,
            arena,
            fragment,
        })
    }
    /// Qualified before sealing. All later reads are fixed/indexed borrows of
    /// these same immutable rows, including their original source and capture.
    pub(crate) fn prepare_fragment_effects(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        let view = self.frozen_fragment_effects()?;
        for effect in view.fragment.effects.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<ContextEffect>() as u64,
            )?;
            let mut node = |index: usize, fragment: bool| -> Result<(), SessionError> {
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<HtmlNodeKind>() as u64,
                )?;
                let record = view
                    .arena
                    .records
                    .get(index)
                    .ok_or(SessionError::ForeignNode)?;
                if fragment && !matches!(record.kind, HtmlNodeKind::Fragment) {
                    return Err(SessionError::ContextUnavailable);
                }
                Ok(())
            };
            match effect {
                ContextEffect::TemplateCache {
                    document,
                    inert: Some(inert),
                } => {
                    node(*inert, false)?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    view.fragment
                        .context
                        .capture()
                        .document_at(*document)
                        .map_err(|_| SessionError::ContextUnavailable)?;
                    if !matches!(view.arena.records[*inert].kind, HtmlNodeKind::Document(_)) {
                        return Err(SessionError::ContextUnavailable);
                    }
                }
                ContextEffect::AttachShadow {
                    host,
                    template,
                    contents: Some(contents),
                    source,
                    ..
                } => {
                    node(*template, false)?;
                    node(*contents, true)?;
                    match host {
                        ShadowHostKey::Owned(index) => node(*index, false)?,
                        ShadowHostKey::Captured(host) => {
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            view.fragment
                                .context
                                .capture()
                                .node(host.subject())
                                .map_err(|_| SessionError::ContextUnavailable)?;
                        }
                    }
                    c.require_source_range(source)?;
                }
                _ => return Err(SessionError::ContextUnavailable),
            }
        }
        for payload in &storage.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let RetainedPayload::LookupRegistry {
                node: Some(node),
                registry,
                ..
            } = payload
            else {
                continue;
            };
            c.charge(
                WorkCategory::InspectByte,
                (mem::size_of::<usize>() + mem::size_of::<RegistryRef>()) as u64,
            )?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            view.node(*node)?;
            if let RegistryRef::Captured(index) = registry {
                // This validation creates one actual fixed capture handle
                // before commit; the immutable lender itself creates none.
                payload_write::<crate::HtmlCaptureRegistryId>(c, 1)?;
                c.charge(WorkCategory::ProviderCall, 1)?;
                view.fragment
                    .context
                    .capture()
                    .registry_at(*index)
                    .map_err(|_| SessionError::ContextUnavailable)?;
            }
        }
        Ok(())
    }
}

impl<'a> FrozenFragmentEffects<'a> {
    pub fn effect(&self, index: usize) -> Option<FrozenFragmentEffect<'a>> {
        self.get(index).expect("prepublication-qualified effect")
    }
    pub fn registry(&self, index: usize) -> Option<FrozenFragmentRegistryBinding<'a>> {
        self.registry_binding(index)
            .expect("prepublication-qualified registry")
    }
}
impl FrozenFragmentRegistryBinding<'_> {
    pub fn node(&self) -> HtmlNode<'_> {
        HtmlNode {
            arena: self.node.arena,
            index: self.node.index,
        }
    }
    pub fn registry(&self) -> &FrozenFragmentRegistry<'_> {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{
        states::State,
        tests::LexerFixture,
        token::{Event, TokenKind},
    };
    use crate::*;

    fn lexical_tag(lexer: &mut LexerFixture<'_>, c: &mut ParseControl) -> Option<Tag> {
        loop {
            match lexer.next(c, false).unwrap().unwrap() {
                Event::Token(event) => {
                    let token = event.into_token(c).unwrap();
                    let TokenKind::Tag(tag) = token.kind else {
                        panic!("literal start tag")
                    };
                    return Some(tag);
                }
                event => event.release(c).unwrap(),
            }
        }
    }

    #[test]
    fn context_freeze_selects_actual_nonzero_output_and_preserves_it_on_alias_retry() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(10, 10, 10, 1000, 10, 10, 10),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let request = HtmlFragmentParseRequest::new(
            HtmlSource::utf8(""),
            context,
            crate::parser::tree_builder::tests::limits(100),
        )
        .unwrap();
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_standalone_document(&mut c).unwrap();
        let bootstrap = session.initialize_fragment(&mut c, &request).unwrap();
        assert_ne!(bootstrap.output.index, 0);
        let output = bootstrap.output.index;
        let mut frozen = session.freeze(&mut c).unwrap();
        assert!(frozen.document().is_none());
        let fragment = frozen.fragment().unwrap();
        assert_eq!(fragment.root().id().index, output);
        assert!(matches!(fragment.root().kind(), HtmlNodeKind::Fragment));
        assert_eq!(fragment.root().children().count(), 0);
        let root = fragment.root().id().clone();
        let ParserDocumentReference::Captured(actual) =
            frozen.qualified_node_document(&mut c, &root).unwrap()
        else {
            panic!("F keeps actual captured target document")
        };
        assert_eq!(&actual, request.context().target_document());
        assert_eq!(
            frozen.lookup_registry(&mut c, &root).err(),
            Some(SessionError::ContextFactUnavailable(
                HtmlCaptureFact::Registry
            ))
        );
        assert!(frozen.template_cache(&mut c, 0).unwrap().is_none());
        assert!(frozen.declarative_shadow(&mut c, 0).unwrap().is_none());
        let alias = frozen.fragment().unwrap().clone();
        assert!(frozen.release(&mut c).is_err());
        let first = c.stop().unwrap();
        assert_eq!(frozen.fragment().unwrap().root().id().index, output);
        drop(alias);
        frozen.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(first));
        crate::parser::tree_builder::tests::assert_released(&c);
    }

    #[test]
    fn registry_adoption_observes_detached_root_before_scoped_destination_attachment() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let global = builder.add_registry(Some(Vec::new())).unwrap();
        builder
            .establish_registry_scope(&global, HtmlRegistryScope::Global)
            .unwrap();
        let scoped = builder.add_registry(Some(Vec::new())).unwrap();
        builder
            .establish_registry_scope(&scoped, HtmlRegistryScope::Scoped)
            .unwrap();
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Registry(global.clone()),
            )
            .unwrap();
        builder
            .establish_document_template_facts(
                &document,
                HtmlDocumentClass::Html,
                HtmlTemplateDocumentRole::Ordinary,
                HtmlCaptureTemplateCacheInput::Absent,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Registry(scoped.clone()),
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let global = capture.registry_id(&global).unwrap();
        let scoped = capture.registry_id(&scoped).unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let request = HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<div><template><span>"),
            context,
            crate::parser::tree_builder::tests::limits(100),
        )
        .unwrap();
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_standalone_document(&mut c).unwrap();
        let bootstrap = session.initialize_fragment(&mut c, &request).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let mut input = lexical_tag(&mut lexer, &mut c);
        let destination = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &bootstrap.root,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        session
            .insert(&mut c, &bootstrap.output, &destination, None, None)
            .unwrap();
        assert_eq!(
            session.lookup_registry(&mut c, &destination).unwrap(),
            HtmlCapturedRegistry::Registry(scoped)
        );
        let mut input = lexical_tag(&mut lexer, &mut c);
        let template = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &bootstrap.root,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        let contents = session
            .unpatched_template_contents(&mut c, &template)
            .unwrap()
            .unwrap();
        let mut input = lexical_tag(&mut lexer, &mut c);
        let moved = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &contents,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        session
            .insert(&mut c, &contents, &moved, None, None)
            .unwrap();
        assert_eq!(
            session.lookup_registry(&mut c, &moved).unwrap(),
            HtmlCapturedRegistry::Null
        );
        // Source DOM adoption sees this root detached. Its previous null
        // registry selects the destination Document's effective global registry,
        // regardless of the later scoped parent in the staged physical record.
        session
            .insert(&mut c, &destination, &moved, None, None)
            .unwrap();
        assert_eq!(
            session.lookup_registry(&mut c, &moved).unwrap(),
            HtmlCapturedRegistry::Registry(global)
        );
        let ParserDocumentReference::Captured(actual) =
            session.qualified_node_document(&mut c, &moved).unwrap()
        else {
            panic!("actual destination capture Document")
        };
        assert_eq!(&actual, request.context().target_document());
        assert_eq!(
            session.active().unwrap().records[moved.index].parent,
            Some(destination.index)
        );
        drop(actual);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }

    #[test]
    fn template_adoption_uses_conditional_captured_class_and_reuses_actual_inert_owner() {
        for class in [
            HtmlDocumentClass::Html,
            HtmlDocumentClass::NonHtml,
            HtmlDocumentClass::Unavailable,
        ] {
            for inert in [false, true] {
                let mut builder = HtmlCaptureBuilder::new(
                    HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
                    HtmlHostFactsRevision::fresh(),
                );
                let document = builder
                    .add_document(
                        HtmlDocumentMode::NoQuirks,
                        HtmlScriptingStatus::Disabled,
                        HtmlCaptureRegistryInput::Null,
                    )
                    .unwrap();
                builder
                    .establish_document_template_facts(
                        &document,
                        class,
                        if inert {
                            HtmlTemplateDocumentRole::InertTemplate
                        } else {
                            HtmlTemplateDocumentRole::Ordinary
                        },
                        HtmlCaptureTemplateCacheInput::Absent,
                    )
                    .unwrap();
                let context = builder
                    .add_element(
                        &document,
                        QualifiedName::new(Namespace::html(), None, "div"),
                        OrderedAttributes::default(),
                        HtmlCapturedIsValue::Absent,
                        HtmlCaptureRegistryInput::Null,
                    )
                    .unwrap();
                builder.establish_root(&context).unwrap();
                let capture = builder.finish().unwrap();
                let target = capture
                    .document_id(&capture.subject_id(&document).unwrap())
                    .unwrap();
                let context =
                    HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap())
                        .unwrap();
                let request = HtmlFragmentParseRequest::new(
                    HtmlSource::utf8("<template id=t>"),
                    context,
                    crate::parser::tree_builder::tests::limits(100),
                )
                .unwrap();
                let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
                c.admit_source(request.source()).unwrap();
                let mut session =
                    ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                        .unwrap();
                session.initialize_standalone_document(&mut c).unwrap();
                let bootstrap = session.initialize_fragment(&mut c, &request).unwrap();
                let mut lexer =
                    LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
                let mut input = lexical_tag(&mut lexer, &mut c);
                let element = session
                    .allocate_html_tag_for_creation(
                        &mut c,
                        &mut input,
                        &bootstrap.root,
                        None,
                        CreationContext::Ordinary,
                    )
                    .unwrap();
                assert!(input.is_none());
                let contents = session
                    .unpatched_template_contents(&mut c, &element)
                    .unwrap()
                    .unwrap();
                let ParserDocumentReference::Owned(original) =
                    session.qualified_node_document(&mut c, &contents).unwrap()
                else {
                    panic!("creation uses real local parser inert owner")
                };
                let parser = session.root(&mut c).unwrap();
                assert!(!session.same_node(&mut c, &original, &parser).unwrap());
                let result = session.insert(&mut c, &bootstrap.output, &element, None, None);
                if !inert && class == HtmlDocumentClass::Unavailable {
                    assert!(matches!(
                        result,
                        Err(SessionError::ContextFactUnavailable(
                            HtmlCaptureFact::DocumentClass
                        ))
                    ));
                    let first = c
                        .stop()
                        .expect("missing actual allocation class is terminal");
                    assert_eq!(
                        session.freeze(&mut c).err(),
                        Some(SessionError::Stop(first))
                    );
                    assert!(session.active().unwrap().payloads.values.iter().any(|payload|
                        matches!(payload,RetainedPayload::NodeDocument {node:Some(node),document:DocumentRef::Owned(owner)}
                            if *node==contents.index && *owner==original.index)));
                } else {
                    result.unwrap();
                    let reference = session.qualified_node_document(&mut c, &contents).unwrap();
                    if inert {
                        let ParserDocumentReference::Captured(actual) = reference else {
                            panic!("same captured inert owner")
                        };
                        assert_eq!(actual, target);
                        assert_eq!(
                            session
                                .active()
                                .unwrap()
                                .fragment
                                .as_ref()
                                .unwrap()
                                .effects
                                .len(),
                            0
                        );
                    } else {
                        let ParserDocumentReference::Owned(actual) = reference else {
                            panic!("new retained inert owner")
                        };
                        let storage = session.active().unwrap();
                        let slot = storage.document_slot(&mut c, actual.index).unwrap();
                        assert!(
                            matches!(storage.payloads.values[slot],RetainedPayload::Document {
                            state:template::DocumentState::InertTemplate,class:actual,..} if actual==class)
                        );
                        assert_ne!(actual.index, original.index);
                        assert_eq!(storage.fragment.as_ref().unwrap().effects.len(), 1);
                    }
                }
                lexer.release(&mut c).unwrap();
                if !inert && class != HtmlDocumentClass::Unavailable {
                    let mut frozen = session.freeze(&mut c).unwrap();
                    let root = frozen.fragment().unwrap().root().id().clone();
                    let element = frozen.fragment().unwrap().root().children().next().unwrap();
                    let cache = frozen.template_cache(&mut c, 0).unwrap().unwrap();
                    assert_eq!(cache.document, target);
                    let ParserDocumentReference::Captured(actual) =
                        frozen.qualified_node_document(&mut c, &element).unwrap()
                    else {
                        panic!("frozen E retains captured target")
                    };
                    assert_eq!(actual, target);
                    assert_eq!(
                        frozen.lookup_registry(&mut c, &element).unwrap(),
                        HtmlCapturedRegistry::Null
                    );
                    let contents = frozen
                        .fragment()
                        .unwrap()
                        .node(&element)
                        .unwrap()
                        .template_contents()
                        .unwrap();
                    let ParserDocumentReference::Owned(actual) =
                        frozen.qualified_node_document(&mut c, &contents).unwrap()
                    else {
                        panic!("frozen C retains result-owned inert document")
                    };
                    assert_eq!(actual, cache.inert);
                    assert_eq!(frozen.document_class(&mut c, &actual).unwrap(), class);
                    assert_ne!(actual, root);
                    assert!(frozen.template_cache(&mut c, 1).unwrap().is_none());
                    frozen.release(&mut c).unwrap();
                } else {
                    session.release(&mut c).unwrap();
                }
                crate::parser::tree_builder::tests::assert_released(&c);
            }
        }
    }

    #[test]
    fn fragment_bootstrap_actual_three_node_bound_preserves_cleanup_and_foreign_source_refusal() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(10, 10, 10, 1000, 10, 10, 10),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        for cap in [2, 3] {
            // Actual P and internal root consume two nodes; F is the third.
            let limits = crate::parser::tree_builder::tests::limits(100)
                .into_builder()
                .managed_nodes(cap)
                .finish()
                .unwrap();
            let request =
                HtmlFragmentParseRequest::new(HtmlSource::utf8(""), context.clone(), limits)
                    .unwrap();
            let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
            c.admit_source(request.source()).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            session.initialize_standalone_document(&mut c).unwrap();
            let result = session.initialize_fragment(&mut c, &request);
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                cap
            );
            if cap == 3 {
                let bootstrap = result.unwrap();
                assert_eq!(bootstrap.root.index, 1);
                assert_eq!(bootstrap.output.index, 2);
                assert!(matches!(
                    session.active().unwrap().records[bootstrap.output.index].kind,
                    HtmlNodeKind::Fragment
                ));
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedNodes,
                        ..
                    }))
                ));
                let first = c.stop().unwrap();
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(first))
                );
                assert!(
                    session
                        .active()
                        .unwrap()
                        .fragment
                        .as_ref()
                        .unwrap()
                        .output
                        .is_none()
                );
            }
            session.release(&mut c).unwrap();
            crate::parser::tree_builder::tests::assert_released(&c);
        }
        let limits = crate::parser::tree_builder::tests::limits(100);
        let request =
            HtmlFragmentParseRequest::new(HtmlSource::utf8("A"), context, limits).unwrap();
        let original = HtmlSource::utf8("A");
        let mut c = ParseControl::new(limits, HtmlParseCancellation::new());
        c.admit_source(&original).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_standalone_document(&mut c).unwrap();
        let slots = session.active().unwrap().payloads.values.len();
        let result = session.initialize_fragment(&mut c, &request);
        assert!(matches!(
            result,
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignSource
            )))
        ));
        assert!(session.active().unwrap().fragment.is_none());
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(session.active().unwrap().payloads.values.len(), slots);
        assert_eq!(
            c.stop(),
            Some(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignSource
            ))
        );
        session.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }
    #[test]
    fn frozen_creation_facts_keep_independent_saved_is_and_lexical_maps() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let request = HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<div is='A&amp;B'>"),
            context,
            crate::parser::tree_builder::tests::limits(100),
        )
        .unwrap();
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_standalone_document(&mut c).unwrap();
        let bootstrap = session.initialize_fragment(&mut c, &request).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let mut input = lexical_tag(&mut lexer, &mut c);
        let original = &input.as_ref().unwrap().attributes.as_slice()[0].value;
        let original_backing = original.value.text().code_units().as_ptr();
        let original_ranges: Vec<_> = original
            .origins
            .as_slice()
            .iter()
            .map(|origin| (origin.start, origin.end, origin.source.clone(), origin.kind))
            .collect();
        let node = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &bootstrap.root,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        let storage = session.active().unwrap();
        let fact = storage.element_creation_state(&mut c, node.index).unwrap();
        let saved = fact.saved_is.unwrap();
        let RetainedPayload::String { value, origins, .. } = &storage.payloads.values[saved] else {
            panic!()
        };
        assert_eq!(value.code_units(), &[65, 38, 66]);
        assert_ne!(value.code_units().as_ptr(), original_backing);
        assert_eq!(
            storage
                .observed_origins(origins.as_ref().unwrap(), &mut c)
                .unwrap()
                .len(),
            original_ranges.len()
        );
        for (actual, expected) in storage
            .observed_origins(origins.as_ref().unwrap(), &mut c)
            .unwrap()
            .iter()
            .zip(&original_ranges)
        {
            assert_eq!(
                (actual.start, actual.end, &actual.source, actual.kind),
                (expected.0, expected.1, &expected.2, expected.3)
            );
        }
        session
            .insert(&mut c, &bootstrap.output, &node, None, None)
            .unwrap();
        lexer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let id = frozen.fragment().unwrap().root().children().next().unwrap();
        let (state, saved) = frozen.element_creation(&mut c, &id).unwrap();
        assert_eq!(state, crate::HtmlCustomElementState::Undefined);
        assert_eq!(saved.unwrap().code_units(), &[65, 38, 66]);
        drop(id);
        frozen.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }
    #[test]
    fn saved_is_copy_has_source_derived_map_and_managed_coexistence_caps() {
        for (maps, units, resource, success, taken) in [
            (33, 1000, HtmlParseResource::SourceMapEntries, false, false),
            (34, 1000, HtmlParseResource::SourceMapEntries, false, true),
            (41, 1000, HtmlParseResource::SourceMapEntries, false, true),
            (42, 1000, HtmlParseResource::SourceMapEntries, true, true),
            (1000, 34, HtmlParseResource::ManagedUtf16Units, false, false),
            (1000, 35, HtmlParseResource::ManagedUtf16Units, false, true),
            (1000, 70, HtmlParseResource::ManagedUtf16Units, false, true),
            (1000, 71, HtmlParseResource::ManagedUtf16Units, true, true),
        ] {
            let mut builder = HtmlCaptureBuilder::new(
                HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
                HtmlHostFactsRevision::fresh(),
            );
            let document = builder
                .add_document(
                    HtmlDocumentMode::NoQuirks,
                    HtmlScriptingStatus::Disabled,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            let context = builder
                .add_element(
                    &document,
                    QualifiedName::new(Namespace::html(), None, "div"),
                    OrderedAttributes::default(),
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            builder.establish_root(&context).unwrap();
            let capture = builder.finish().unwrap();
            let context =
                HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap())
                    .unwrap();
            let limits = crate::parser::tree_builder::tests::limits(100)
                .into_builder()
                .source_map_entries(maps)
                .managed_utf16_units(units)
                .finish()
                .unwrap();
            let request = HtmlFragmentParseRequest::new(
                HtmlSource::utf8("<div is='A&amp;B'>"),
                context,
                limits,
            )
            .unwrap();
            let mut c = ParseControl::new(limits, request.cancellation().clone());
            c.admit_source(request.source()).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            session.initialize_standalone_document(&mut c).unwrap();
            let bootstrap = session.initialize_fragment(&mut c, &request).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let mut input = lexical_tag(&mut lexer, &mut c);
            // Name div(3), name is(2), decoded A&B(3) own eight
            // original map entries. Lexical retention adds two occurrence
            // headers, their two causes and seventeen parts: tag <, space, >
            // and three name atoms; attribute two name atoms, three value
            // atoms, equals and two quotes, plus two empty value boundary
            // anchors and the closing end anchor. Thus 29 coexist before creation.
            // Saved is adds three maps, one cause and one group header before
            // take: 34 coexist. Final five fields add ten fixed entries,
            // giving the old 21 creation entries plus 21 lexical entries=42.
            // Managed strings: root html+URI=32; saved is=3 BEFORE take;
            // div+URI+is+A&B=36 after take, hence final 71.
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::SourceMapEntries)
                    .current,
                29
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                32
            );
            let result = session.allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &bootstrap.root,
                None,
                CreationContext::Ordinary,
            );
            assert_eq!(input.is_none(), taken);
            if success {
                let node = result.unwrap();
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::SourceMapEntries)
                        .current,
                    42
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    71
                );
                let (state, value) = session.element_creation(&mut c, &node).unwrap();
                assert_eq!(state, crate::HtmlCustomElementState::Undefined);
                assert_eq!(value.unwrap().code_units(), &[65, 38, 66]);
            } else {
                let first = c.stop().unwrap();
                assert!(matches!(result,Err(SessionError::Stop(stop)) if stop==first));
                assert!(
                    matches!(first,ParseStop::LimitExceeded {resource:actual,..} if actual==resource)
                );
                assert_eq!(session.active().unwrap().records.len(), 3);
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(first))
                );
                assert_eq!(c.stop(), Some(first));
            }
            if let Some(tag) = input.take() {
                tag.name.release(&mut c).unwrap();
                release_token_attributes(tag.attributes, &mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            crate::parser::tree_builder::tests::assert_released(&c);
        }
    }

    #[test]
    fn every_saved_is_creation_work_cut_keeps_original_or_registered_backing_cleanup_owned() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let limits = crate::parser::tree_builder::tests::limits(100);
        let request =
            HtmlFragmentParseRequest::new(HtmlSource::utf8("<div is='A&amp;B'>"), context, limits)
                .unwrap();
        let mut complete_work = None;
        // The first run measures ONLY the finite coverage extent. It supplies
        // no expected category totals or behavioral oracle for subsequent cuts.
        // Every actual admission from borrowed preflight through native binding
        // is then interrupted independently with the same lexical input.
        let mut credit = None;
        loop {
            let mut c = ParseControl::new(limits, request.cancellation().clone());
            c.admit_source(request.source()).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            session.initialize_standalone_document(&mut c).unwrap();
            let bootstrap = session.initialize_fragment(&mut c, &request).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let mut input = lexical_tag(&mut lexer, &mut c);
            if let Some(credit) = credit {
                let remaining = limits.work_credits()
                    - c.usage().resource(HtmlParseResource::WorkCredits).current;
                c.charge(WorkCategory::StateStep, remaining - credit)
                    .unwrap();
            }
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let result = session.allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &bootstrap.root,
                None,
                CreationContext::Ordinary,
            );
            match credit {
                None => {
                    complete_work =
                        Some(c.usage().resource(HtmlParseResource::WorkCredits).current - before);
                    assert!(
                        complete_work.unwrap() < 60_000,
                        "bounded literal coverage extent: {}",
                        complete_work.unwrap()
                    );
                    assert!(result.is_ok());
                }
                Some(credit) if credit < complete_work.unwrap() => {
                    let first = c.stop().unwrap();
                    assert!(matches!(
                        first,
                        ParseStop::LimitExceeded {
                            resource: HtmlParseResource::WorkCredits,
                            ..
                        }
                    ));
                    assert!(matches!(result, Err(SessionError::Stop(stop)) if stop == first));
                    assert_eq!(
                        session.freeze(&mut c).err(),
                        Some(SessionError::Stop(first))
                    );
                    assert_eq!(c.stop(), Some(first));
                }
                Some(_) => {
                    assert!(result.is_ok());
                    assert!(input.is_none());
                }
            }
            if let Some(tag) = input.take() {
                tag.name.release(&mut c).unwrap();
                release_token_attributes(tag.attributes, &mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            // At the exact successful creation budget, cleanup's own active
            // preflight may latch the first stop and retain the session. Retry
            // only that same owner through its terminal disposal contract.
            if let Err(error) = session.release(&mut c) {
                let first = c.stop().expect("cleanup preflight first stop");
                assert_eq!(error, SessionError::Stop(first));
                session.release(&mut c).unwrap();
                assert_eq!(c.stop(), Some(first));
            }
            crate::parser::tree_builder::tests::assert_released(&c);
            match credit {
                None => credit = Some(0),
                Some(value) if value < complete_work.unwrap() => credit = Some(value + 1),
                Some(_) => break,
            }
        }
    }

    #[test]
    fn creation_registry_preparation_admits_actual_prepared_receiver_and_one_payload_slot() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let limits = crate::parser::tree_builder::tests::limits(100);
        let request =
            HtmlFragmentParseRequest::new(HtmlSource::utf8("<div>"), context, limits).unwrap();
        let mut c = ParseControl::new(limits, request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_standalone_document(&mut c).unwrap();
        let bootstrap = session.initialize_fragment(&mut c, &request).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let mut input = lexical_tag(&mut lexer, &mut c);
        let tag = input.as_mut().unwrap();
        SessionStorage::check_html_tag(&mut c, tag).unwrap();
        let storage = session.active_mut().unwrap();
        let parent = storage.index(&mut c, &bootstrap.root).unwrap();
        let mut prepared = storage
            .prepare_document_allocation(&mut c, Some(DocumentRef::Owned(0)), false)
            .unwrap();
        // Admit actual vector growth before measuring the separate fixed writes.
        // No initialized slot or logical entry is manufactured by this reserve.
        c.reserve_vector(
            &mut storage.payloads.values,
            &mut storage.payloads.allocation,
            1,
        )
        .unwrap();
        let entries = storage.payloads.values.len();
        let bytes = c.usage().work(WorkCategory::CopyByte);
        let writes = c.usage().work(WorkCategory::RecordWrite);
        storage
            .prepare_creation_registry(
                &mut c,
                parent,
                tag.name.value.text(),
                None,
                true,
                &mut prepared,
            )
            .unwrap();
        // RegistryRef read, actual Option<usize> receiver assignment, and the
        // one fully admitted RetainedPayload copy. CreationState is inside that
        // payload, not a second independently moved Option receiver.
        let expected_bytes = mem::size_of::<RegistryRef>()
            + mem::size_of::<Option<usize>>()
            + mem::size_of::<RetainedPayload>();
        let actual_bytes = c.usage().work(WorkCategory::CopyByte) - bytes;
        assert_eq!(c.usage().work(WorkCategory::RecordWrite) - writes, 2);
        assert_eq!(prepared.unwrap().registry, Some(entries));
        assert_eq!(storage.payloads.values.len(), entries + 1);
        let tag = input.take().unwrap();
        tag.name.release(&mut c).unwrap();
        release_token_attributes(tag.attributes, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
        assert_eq!(actual_bytes, expected_bytes as u64);
    }
}

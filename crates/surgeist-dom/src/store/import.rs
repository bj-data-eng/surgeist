//! One iterative clone executor for published imports and actual staged clones.
use super::*;

#[derive(Clone)]
pub(super) enum Source {
    Published(DomSnapshot, NodeId, source::input::InputCapture),
    Staged(Key),
}
#[derive(Clone, Copy)]
pub(super) enum Placement {
    Detached,
    BeforeChildren(Key),
    AfterClone(Key),
}
pub(super) struct CloneRequest {
    pub document: Key,
    pub creation_global: Option<crate::DomHostedGlobalId>,
    pub fallback_registry: crate::DomRegistryBinding,
    pub placement: Placement,
    pub subtree: bool,
    pub scoped: bool,
    pub root_ticket: bool,
}
pub(super) struct Frame {
    source: Source,
    copy: Key,
    document: Key,
    creation_global: Option<crate::DomHostedGlobalId>,
    fallback_registry: crate::DomRegistryBinding,
    placement: Placement,
    template_child: usize,
    appended: bool,
    next_child: usize,
    subtree: bool,
    shadow_started: bool,
    root_ticket: bool,
}
impl Frame {
    pub(super) fn retained_payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        step()?;
        Ok(source::storage::Payload::default())
    }
    pub(super) fn checkpoint(&self) -> Self {
        Self {
            source: self.source.clone(),
            copy: self.copy,
            document: self.document,
            creation_global: self.creation_global,
            fallback_registry: self.fallback_registry,
            placement: self.placement,
            template_child: self.template_child,
            appended: self.appended,
            next_child: self.next_child,
            subtree: self.subtree,
            shadow_started: self.shadow_started,
            root_ticket: self.root_ticket,
        }
    }
}

impl Draft {
    fn published_clone_source(
        &mut self,
        snapshot: &DomSnapshot,
        id: NodeId,
        cancel: &AtomicBool,
    ) -> Result<(Source, Option<source::checkpoint::Charge>), DomError> {
        let capture = self.retain_input_capture(&snapshot.state, cancel)?;
        Ok((Source::Published(snapshot.clone(), id, capture), None))
    }
    fn source_child(
        &self,
        source: &Source,
        index: usize,
        template: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<(Source, Option<source::checkpoint::Charge>)>, DomError> {
        self.source_step(cancel)?;
        match source {
            Source::Published(s, id, capture) => {
                let n = s.node(*id)?;
                let n = if template {
                    if let Some(t) = n.as_template() {
                        s.node(t.contents()?)?
                    } else {
                        return Ok(None);
                    }
                } else {
                    n
                };
                Ok(n.children()
                    .get(index)
                    .copied()
                    .map(|id| (Source::Published(s.clone(), id, capture.clone()), None)))
            }
            Source::Staged(k) => {
                let n = self.node(*k)?;
                let n = if template {
                    if !matches!(n.kind(), transaction::KindRead::Element(element) if element.is_template())
                    {
                        return Ok(None);
                    }
                    self.node(
                        n.template_contents()
                            .ok_or(DomError::TemplateContentsUnavailable)?,
                    )?
                } else {
                    n
                };
                Ok(n.children()
                    .get(index)
                    .map(|key| (Source::Staged(key), None)))
            }
        }
    }
    fn source_shadow(
        &self,
        source: &Source,
        cancel: &AtomicBool,
    ) -> Result<
        Option<(
            Source,
            crate::DomShadowRootState,
            Option<source::checkpoint::Charge>,
        )>,
        DomError,
    > {
        self.source_step(cancel)?;
        match source {
            Source::Published(s, id, capture) => {
                let Some(root) = s.node(*id)?.hosted_shadow_root() else {
                    return Ok(None);
                };
                let state = *s.shadow_root(root)?.state();
                Ok(Some((
                    Source::Published(s.clone(), root, capture.clone()),
                    state,
                    None,
                )))
            }
            Source::Staged(k) => self
                .node(*k)?
                .shadow_root()
                .map(|root| match self.node(root)?.kind() {
                    transaction::KindRead::ShadowRoot(state) => {
                        Ok((Source::Staged(root), *state, None))
                    }
                    _ => unreachable!("owned root"),
                })
                .transpose(),
        }
    }
    pub(super) fn import_node(
        &mut self,
        snapshot: &DomSnapshot,
        source: NodeId,
        document: Key,
        subtree: bool,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        let (source, payload) = self.published_clone_source(snapshot, source, cancel)?;
        let root = self.begin_clone(
            source,
            payload,
            CloneRequest {
                document,
                creation_global: None,
                fallback_registry: crate::DomRegistryBinding::Null,
                placement: Placement::Detached,
                subtree,
                scoped: false,
                root_ticket: false,
            },
            limits,
            cancel,
        )?;
        self.run_effects(limits, cancel)?;
        Ok(root)
    }
    pub(super) fn clone_node(
        &mut self,
        source: Key,
        subtree: bool,
        fallback: Option<crate::DomCreationRealmFallback>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if matches!(
            self.node(source)?.kind(),
            transaction::KindRead::ShadowRoot(_)
        ) {
            return Err(DomError::NotSupported);
        }
        let document = self.node(source)?.document();
        let creation_global = self.resolve_creation_global(document, fallback, cancel)?;
        self.begin_clone(
            Source::Staged(source),
            None,
            CloneRequest {
                document,
                creation_global,
                fallback_registry: crate::DomRegistryBinding::Null,
                placement: Placement::Detached,
                subtree,
                scoped: self.effect_scoped,
                root_ticket: true,
            },
            limits,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }

    pub(super) fn import_live_node(
        &mut self,
        source: Key,
        document: Key,
        (subtree, registry, fallback): (
            bool,
            Option<crate::DomRegistryId>,
            crate::DomCreationRealmFallback,
        ),
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if matches!(
            self.node(source)?.kind(),
            transaction::KindRead::Document(_) | transaction::KindRead::ShadowRoot(_)
        ) {
            return Err(DomError::NotSupported);
        }
        let destination = self.document_metadata(document)?.custom_element_registry();
        let fallback_registry = if let Some(id) = registry {
            let selected = crate::DomRegistryBinding::Registry(id);
            let registry = self.creation_registry(id, cancel)?;
            match registry
                .capture
                .registry_scope(&registry.local)
                .map_err(DomError::SelectCapture)?
            {
                surgeist_html_semantics::HtmlRegistryScope::Scoped => selected,
                surgeist_html_semantics::HtmlRegistryScope::Global if destination == selected => {
                    selected
                }
                surgeist_html_semantics::HtmlRegistryScope::Global => {
                    return Err(DomError::NotSupported);
                }
                surgeist_html_semantics::HtmlRegistryScope::Unavailable => {
                    return Err(DomError::HostedSourceUnavailable);
                }
            }
        } else {
            destination
        };
        let creation_global = self.resolve_creation_global(document, Some(fallback), cancel)?;
        self.begin_clone(
            Source::Staged(source),
            None,
            CloneRequest {
                document,
                creation_global,
                fallback_registry,
                subtree,
                placement: Placement::Detached,
                scoped: self.effect_scoped,
                root_ticket: true,
            },
            limits,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }
    pub(super) fn begin_clone(
        &mut self,
        source: Source,
        payload: Option<source::checkpoint::Charge>,
        request: CloneRequest,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        let CloneRequest {
            document,
            creation_global,
            fallback_registry,
            placement,
            subtree,
            scoped,
            root_ticket,
        } = request;
        let start = self.created.len();
        let copy = self.clone_single(&source, document, fallback_registry, limits, cancel)?;
        if creation_global.is_some() && self.node(copy)?.relevant_global().is_none() {
            self.establish_creation_global(copy, creation_global, cancel)?;
            let count = self.node(copy)?.attribute_ids().len();
            for index in 0..count {
                self.source_step(cancel)?;
                let attribute = self
                    .node(copy)?
                    .attribute_ids()
                    .get(index)
                    .ok_or(DomError::StaleNode)?;
                self.establish_creation_global(attribute, creation_global, cancel)?;
            }
        }
        if creation_global.is_some()
            && let Some(contents) = self.node(copy)?.template_contents()
            && self.node(contents)?.relevant_global().is_none()
        {
            self.establish_creation_global(contents, creation_global, cancel)?;
        }
        let document = if matches!(self.node(copy)?.kind(), transaction::KindRead::Document(_)) {
            copy
        } else {
            document
        };
        if scoped {
            for i in start..self.created.len() {
                self.mark_source_private(Key::Created(i), cancel)?;
            }
        }
        self.push_effect_retained(
            select::Effect::Clone(Frame {
                source,
                copy,
                document,
                creation_global,
                fallback_registry,
                placement,
                template_child: 0,
                appended: false,
                next_child: 0,
                subtree,
                shadow_started: false,
                root_ticket,
            }),
            scoped,
            payload,
            cancel,
        )?;
        Ok(copy)
    }
    pub(super) fn clone_effect(
        &mut self,
        mut frame: Frame,
        scoped: bool,
        payload: Option<source::checkpoint::Charge>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if frame.subtree
            && let Some((source, source_payload)) =
                self.source_child(&frame.source, frame.template_child, true, cancel)?
        {
            frame.template_child += 1;
            let parent = self
                .node(frame.copy)?
                .template_contents()
                .expect("fresh contents");
            let document = self.node(parent)?.document();
            let creation_global = frame.creation_global;
            self.push_effect_retained(select::Effect::Clone(frame), scoped, payload, cancel)?;
            self.begin_clone(
                source,
                source_payload,
                CloneRequest {
                    document,
                    creation_global,
                    fallback_registry: crate::DomRegistryBinding::Null,
                    placement: Placement::BeforeChildren(parent),
                    subtree: true,
                    scoped,
                    root_ticket: false,
                },
                limits,
                cancel,
            )?;
            return Ok(());
        }
        if !frame.appended {
            frame.appended = true;
            let placement = frame.placement;
            let copy = frame.copy;
            self.push_effect_retained(select::Effect::Clone(frame), scoped, payload, cancel)?;
            if let Placement::BeforeChildren(parent) = placement {
                self.pre_insert(copy, parent, None, limits, cancel)?;
            }
            return Ok(());
        }
        if frame.subtree
            && let Some((source, source_payload)) =
                self.source_child(&frame.source, frame.next_child, false, cancel)?
        {
            frame.next_child += 1;
            let parent = frame.copy;
            let document = frame.document;
            let creation_global = frame.creation_global;
            let fallback_registry = frame.fallback_registry;
            self.push_effect_retained(select::Effect::Clone(frame), scoped, payload, cancel)?;
            self.begin_clone(
                source,
                source_payload,
                CloneRequest {
                    document,
                    creation_global,
                    fallback_registry,
                    placement: Placement::BeforeChildren(parent),
                    subtree: true,
                    scoped,
                    root_ticket: false,
                },
                limits,
                cancel,
            )?;
            return Ok(());
        }
        if !frame.shadow_started {
            frame.shadow_started = true;
            if let Some((source, state, source_payload)) =
                self.source_shadow(&frame.source, cancel)?
                && state.clonable()
            {
                let mut init = crate::DomShadowRootInit::from(state.properties());
                if state.custom_element_registry() == crate::DomRegistryBinding::Null {
                    init = init.with_null_registry();
                }
                let root = self.attach_shadow(frame.copy, init, limits, cancel)?;
                if frame.creation_global.is_some() && self.node(root)?.relevant_global().is_none() {
                    self.establish_creation_global(root, frame.creation_global, cancel)?;
                }
                let mut cloned = self.shadow_state_mut(root, cancel)?;
                cloned.retain_clone_flags(state);
                cloned.commit(cancel)?;
                let document = frame.document;
                let creation_global = frame.creation_global;
                self.push_effect_retained(select::Effect::Clone(frame), scoped, payload, cancel)?;
                self.push_effect_retained(
                    select::Effect::Clone(Frame {
                        source,
                        copy: root,
                        document,
                        creation_global,
                        fallback_registry: crate::DomRegistryBinding::Null,
                        placement: Placement::Detached,
                        template_child: 0,
                        appended: true,
                        next_child: 0,
                        subtree: true,
                        shadow_started: false,
                        root_ticket: false,
                    }),
                    scoped,
                    source_payload,
                    cancel,
                )?;
                return Ok(());
            }
        }
        if let Placement::AfterClone(parent) = frame.placement {
            self.pre_insert(frame.copy, parent, None, limits, cancel)?;
        }
        // The root's one caller ticket belongs to this retained native tail,
        // including when a connected Document descendant yielded Source work.
        if frame.root_ticket {
            self.register_ticket(frame.copy, cancel)?;
        }
        Ok(())
    }
    fn clone_single(
        &mut self,
        source: &Source,
        document: Key,
        fallback_registry: crate::DomRegistryBinding,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        check_cancel(cancel)?;
        let original = match source {
            Source::Published(snapshot, id, _) => {
                self.freeze_published_node_input(snapshot, *id, cancel)?
            }
            Source::Staged(key) => self.freeze_node_input(*key, cancel)?,
        };
        if let Source::Published(snapshot, _, _) = source
            && snapshot.owner() != self.store
            && matches!(original.kind(), transaction::KindRead::Element(element)
                if !matches!(element.custom_element_registry(), crate::DomRegistryBinding::Null))
        {
            return Err(DomError::NotSupported);
        }
        let script = match source {
            Source::Published(s, id, _) => s.node(*id)?.node.script().map(|v| v.flags),
            Source::Staged(k) => self.node(*k)?.script().map(|v| v.flags),
        };
        let formation = source::formation::Formation::new(self, cancel)?;
        // Captured import preserves its original refusal point; only the live
        // canonical clone command can reach Document cloning below.
        if matches!(source, Source::Published(..))
            && matches!(original.kind(), transaction::KindRead::Document(_))
        {
            return Err(DomError::NotSupported);
        }
        let copy = if let transaction::KindRead::Document(metadata) = original.kind() {
            let registry = match metadata.custom_element_registry() {
                crate::DomRegistryBinding::Null => crate::DomRegistryBinding::Null,
                crate::DomRegistryBinding::Registry(id) => {
                    // The current canonical registry producer admits only Global
                    // registries. A Global registry is never copied to a clone.
                    let record = self
                        .registries
                        .as_deref()
                        .ok_or(DomError::HostedSourceUnavailable)?
                        .get(id)?;
                    match record
                        .capture
                        .registry_scope(&record.local)
                        .map_err(DomError::SelectCapture)?
                    {
                        surgeist_html_semantics::HtmlRegistryScope::Scoped => {
                            crate::DomRegistryBinding::Registry(id)
                        }
                        surgeist_html_semantics::HtmlRegistryScope::Global => {
                            crate::DomRegistryBinding::Null
                        }
                        surgeist_html_semantics::HtmlRegistryScope::Unavailable => {
                            return Err(DomError::NotSupported);
                        }
                    }
                }
            };
            let metadata = metadata.clone_document(
                registry,
                &mut |value| source::formation::share(value, formation.as_ref(), cancel),
                &mut |url| {
                    self.source_step(cancel)?;
                    if let Some(formation) = &formation {
                        let usage = url.retained_storage_usage();
                        formation.retain_payload(
                            source::storage::Payload {
                                bytes: usage.retained_bytes(),
                                units: usage.utf16_extent(),
                            },
                            cancel,
                        )?;
                    }
                    Ok(())
                },
            )?;
            let global = match source {
                Source::Staged(key) => self.node(*key)?.relevant_global(),
                Source::Published(snapshot, id, _) => snapshot
                    .state
                    .node(snapshot.owner(), *id)?
                    .relevant_global(),
            };
            let document = Key::Created(self.created.len());
            let copy = self.allocate_formed(
                NewNode::Document(metadata),
                document,
                limits,
                cancel,
                formation,
            )?;
            let mut common = self.common_fields_mut(copy, cancel)?;
            common.relevant_global = global;
            common.commit(cancel)?;
            copy
        } else if let transaction::KindRead::Element(element) = original.kind() {
            let name = self.bind_canonical_name(element.name(), formation.as_ref(), cancel)?;
            let name = self.form_element_name(name, formation.as_ref(), cancel)?;
            let mut admit = |next| self.admit_custody_usage(next, cancel);
            let children = crate::element::ElementFormation::new(
                self.custody_registry(),
                &self.blocks,
                formation.as_ref(),
                cancel,
                &mut admit,
            );
            let mut formed = element.clone_initialized(
                name.value,
                self.document_metadata(document)?.document_environment(),
                children,
            )?;
            formed.children.name = Some(name.custody);
            self.allocate_element_formed(formed, document, limits, cancel, formation)?
        } else if let transaction::KindRead::Attr(attr) = original.kind() {
            let name = self.bind_canonical_name(attr.name(), formation.as_ref(), cancel)?;
            let value = source::formation::share_listed(attr.value(), formation.as_ref(), cancel)?;
            self.allocate_formed(
                NewNode::Attr(crate::DomAttribute::from_parts(name, value)),
                document,
                limits,
                cancel,
                formation,
            )?
        } else {
            if matches!(original.kind(), transaction::KindRead::ShadowRoot(_)) {
                return Err(DomError::NotSupported);
            }
            // A text or comment is priced where it is shared, as an occurrence that lists the block
            // it borrows, so that the block is charged once for the store and not again by the
            // capacity of the occurrence.
            if let Some(budget) = &formation
                && !matches!(
                    original.kind(),
                    transaction::KindRead::Text(_) | transaction::KindRead::Comment(_)
                )
            {
                budget.retain_payload(
                    source::storage::kind_read(original.kind(), || self.source_step(cancel))?,
                    cancel,
                )?;
            }
            let kind = match original.kind() {
                transaction::KindRead::Element(_)
                | transaction::KindRead::Document(_)
                | transaction::KindRead::Attr(_)
                | transaction::KindRead::ShadowRoot(_) => return Err(DomError::WrongNodeKind),
                transaction::KindRead::DocumentFragment => NewNode::DocumentFragment,
                transaction::KindRead::Text(value) => NewNode::Text(
                    source::formation::share_listed(value, formation.as_ref(), cancel)?,
                ),
                transaction::KindRead::Comment(value) => NewNode::Comment(
                    source::formation::share_listed(value, formation.as_ref(), cancel)?,
                ),
                transaction::KindRead::ProcessingInstruction(value) => {
                    NewNode::ProcessingInstruction(value.clone())
                }
                transaction::KindRead::DocumentType(value) => NewNode::DocumentType(value.clone()),
            };
            self.allocate_formed(kind, document, limits, cancel, formation)?
        };
        if matches!(source, Source::Staged(_))
            && let transaction::KindRead::Element(element) = original.kind()
        {
            let original_registry = match element.custom_element_registry() {
                crate::DomRegistryBinding::Null => fallback_registry,
                registry => registry,
            };
            let registry = match original_registry {
                crate::DomRegistryBinding::Null => crate::DomRegistryBinding::Null,
                crate::DomRegistryBinding::Registry(registry) => {
                    let record = self.creation_registry(registry, cancel)?;
                    match record
                        .capture
                        .registry_scope(&record.local)
                        .map_err(DomError::SelectCapture)?
                    {
                        surgeist_html_semantics::HtmlRegistryScope::Global => {
                            self.document_metadata(document)?.custom_element_registry()
                        }
                        surgeist_html_semantics::HtmlRegistryScope::Scoped => {
                            crate::DomRegistryBinding::Registry(registry)
                        }
                        surgeist_html_semantics::HtmlRegistryScope::Unavailable => {
                            return Err(DomError::NotSupported);
                        }
                    }
                }
            };
            self.establish_element_registry(copy, registry, cancel)?;
            self.record_custom_element_clone_upgrade(copy, registry, cancel)?;
            self.initialize_native_element_features(copy, limits, cancel)?;
        } else {
            self.initialize_native_element(copy, None, limits, cancel)?;
        }
        if let transaction::KindRead::Element(element) = original.kind() {
            let cause = if element
                .input_state()
                .is_some_and(|s| s.input_type() == surgeist_html::attributes::HtmlInputType::Color)
            {
                attributes::AttributeCause::CloneInputColor
            } else {
                attributes::AttributeCause::Change
            };
            match source {
                Source::Published(snapshot, id, _) => {
                    let map = snapshot
                        .node(*id)?
                        .attributes()
                        .ok_or(DomError::WrongNodeKind)?;
                    for index in 0..map.len() {
                        self.source_step(cancel)?;
                        let attr = map.item(index).ok_or(DomError::StaleNode)?;
                        self.append_canonical_attribute(copy, attr.data, cause, limits, cancel)?;
                    }
                }
                Source::Staged(id) => {
                    let count = self.node(*id)?.attribute_ids().len();
                    for index in 0..count {
                        self.source_step(cancel)?;
                        let id = self
                            .node(*id)?
                            .attribute_ids()
                            .get(index)
                            .ok_or(DomError::StaleNode)?;
                        let input = self.freeze_node_input(id, cancel)?;
                        let transaction::KindRead::Attr(attr) = input.kind() else {
                            return Err(DomError::WrongNodeKind);
                        };
                        self.append_canonical_attribute(copy, attr, cause, limits, cancel)?;
                    }
                }
            }
            if let Some(nonce) = element.nonce() {
                self.set_nonce(copy, nonce, limits, cancel)?;
            }
            if let Some(state) = element.svg_script_state() {
                let mut element = self.element_fields_mut(copy, cancel)?;
                *element.svg_script_state_mut().expect("fresh SVG Script") =
                    state.clone_for_element();
                element.commit(cancel)?;
            }
            if let Some(state) = element.input_state() {
                self.clone_input_overlay(copy, state, limits, cancel)?;
            }
            if let Some(state) = element.textarea_state() {
                self.clone_textarea_overlay(copy, state, limits, cancel)?;
            }
        }
        if let Some(flags) = script {
            let mut script = self.script_fields_mut(copy, cancel)?;
            script
                .as_mut()
                .expect("fresh HTML Script")
                .flags
                .clone_already_started_from(flags);
            script.commit(cancel)?;
        }
        Ok(copy)
    }
}

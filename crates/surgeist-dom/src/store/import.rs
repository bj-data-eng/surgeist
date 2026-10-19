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
    pub placement: Placement,
    pub subtree: bool,
    pub scoped: bool,
}
pub(super) struct Frame {
    source: Source,
    copy: Key,
    document: Key,
    placement: Placement,
    template_child: usize,
    appended: bool,
    next_child: usize,
    subtree: bool,
    shadow_started: bool,
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
            placement: self.placement,
            template_child: self.template_child,
            appended: self.appended,
            next_child: self.next_child,
            subtree: self.subtree,
            shadow_started: self.shadow_started,
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
                placement: Placement::Detached,
                subtree,
                scoped: false,
            },
            limits,
            cancel,
        )?;
        self.run_effects(limits, cancel)?;
        Ok(root)
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
            placement,
            subtree,
            scoped,
        } = request;
        let start = self.created.len();
        let copy = self.clone_single(&source, document, limits, cancel)?;
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
                placement,
                template_child: 0,
                appended: false,
                next_child: 0,
                subtree,
                shadow_started: false,
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
            self.push_effect_retained(select::Effect::Clone(frame), scoped, payload, cancel)?;
            self.begin_clone(
                source,
                source_payload,
                CloneRequest {
                    document,
                    placement: Placement::BeforeChildren(parent),
                    subtree: true,
                    scoped,
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
            self.push_effect_retained(select::Effect::Clone(frame), scoped, payload, cancel)?;
            self.begin_clone(
                source,
                source_payload,
                CloneRequest {
                    document,
                    placement: Placement::BeforeChildren(parent),
                    subtree: true,
                    scoped,
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
                let root =
                    self.attach_shadow(frame.copy, state.properties().into(), limits, cancel)?;
                let document = frame.document;
                self.push_effect_retained(select::Effect::Clone(frame), scoped, payload, cancel)?;
                self.push_effect_retained(
                    select::Effect::Clone(Frame {
                        source,
                        copy: root,
                        document,
                        placement: Placement::Detached,
                        template_child: 0,
                        appended: true,
                        next_child: 0,
                        subtree: true,
                        shadow_started: false,
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
        Ok(())
    }
    fn clone_single(
        &mut self,
        source: &Source,
        document: Key,
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
        let copy = if let transaction::KindRead::Element(element) = original.kind() {
            let name = self.bind_canonical_name(element.name(), formation.as_ref(), cancel)?;
            let name = self.form_element_name(name, formation.as_ref(), cancel)?;
            let mut admit = |next| self.admit_custody_usage(next, cancel);
            let children = crate::element::ElementFormation::new(
                self.custody_registry(),
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
        } else {
            if matches!(
                original.kind(),
                transaction::KindRead::Document(_) | transaction::KindRead::ShadowRoot(_)
            ) {
                return Err(DomError::NotSupported);
            }
            if let Some(budget) = &formation {
                budget.retain_payload(
                    source::storage::kind_read(original.kind(), || self.source_step(cancel))?,
                    cancel,
                )?;
            }
            let kind = match original.kind() {
                transaction::KindRead::Element(_)
                | transaction::KindRead::Document(_)
                | transaction::KindRead::ShadowRoot(_) => unreachable!("handled kind"),
                transaction::KindRead::DocumentFragment => NewNode::DocumentFragment,
                transaction::KindRead::Text(value) => NewNode::Text(value.clone()),
                transaction::KindRead::Comment(value) => NewNode::Comment(value.clone()),
                transaction::KindRead::ProcessingInstruction(value) => {
                    NewNode::ProcessingInstruction(value.clone())
                }
                transaction::KindRead::DocumentType(value) => NewNode::DocumentType(value.clone()),
            };
            self.allocate_formed(kind, document, limits, cancel, formation)?
        };
        self.initialize_native_element(copy, limits, cancel)?;
        if let transaction::KindRead::Element(element) = original.kind() {
            let cause = if element
                .input_state()
                .is_some_and(|s| s.input_type() == surgeist_html::HtmlInputType::Color)
            {
                attributes::AttributeCause::CloneInputColor
            } else {
                attributes::AttributeCause::Change
            };
            for attribute in element.attributes().iter() {
                check_cancel(cancel)?;
                self.append_canonical_attribute(copy, attribute, cause, limits, cancel)?;
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

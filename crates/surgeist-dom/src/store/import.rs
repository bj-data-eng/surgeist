//! One iterative clone executor for published imports and actual staged clones.
use super::*;

#[derive(Clone)]
pub(super) enum Source {
    Published(DomSnapshot, NodeId),
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
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        match &self.source {
            Source::Published(snapshot, _) => source::storage::state(&snapshot.state, step),
            Source::Staged(_) => Ok(source::storage::Payload::default()),
        }
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
    fn source_kind(&self, source: &Source) -> Result<DomNodeKind, DomError> {
        Ok(match source {
            Source::Published(s, id) => s.node(*id)?.kind().clone(),
            Source::Staged(k) => self.node(*k)?.kind.clone(),
        })
    }
    fn published_clone_source(
        &self,
        snapshot: &DomSnapshot,
        id: NodeId,
        cancel: &AtomicBool,
    ) -> Result<(Source, Option<source::checkpoint::Charge>), DomError> {
        let formation = source::formation::Formation::new(self, cancel)?;
        if let Some(formation) = &formation {
            formation.retain_payload(
                source::storage::state(&snapshot.state, || self.source_step(cancel))?,
                cancel,
            )?;
        }
        Ok((
            Source::Published(snapshot.clone(), id),
            formation.map(source::formation::Formation::into_charge),
        ))
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
            Source::Published(s, id) => {
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
                n.children()
                    .get(index)
                    .copied()
                    .map(|id| self.published_clone_source(s, id, cancel))
                    .transpose()
            }
            Source::Staged(k) => {
                let n = self.node(*k)?;
                let n = if template {
                    if !matches!(&n.kind, DomNodeKind::Element(element) if element.is_template()) {
                        return Ok(None);
                    }
                    self.node(
                        n.template_contents
                            .ok_or(DomError::TemplateContentsUnavailable)?,
                    )?
                } else {
                    n
                };
                Ok(n.children
                    .get(index)
                    .copied()
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
            Source::Published(s, id) => {
                let Some(root) = s.node(*id)?.hosted_shadow_root() else {
                    return Ok(None);
                };
                let state = *s.shadow_root(root)?.state();
                let (source, payload) = self.published_clone_source(s, root, cancel)?;
                Ok(Some((source, state, payload)))
            }
            Source::Staged(k) => self
                .node(*k)?
                .shadow_root
                .map(|root| match &self.node(root)?.kind {
                    DomNodeKind::ShadowRoot(state) => Ok((Source::Staged(root), *state, None)),
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
                self.mark_source_private(Key::Created(i));
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
                .template_contents
                .expect("fresh contents");
            let document = self.node(parent)?.document;
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
        let original_formation = source::formation::Formation::new(self, cancel)?;
        if let Some(budget) = &original_formation {
            let original = match source {
                Source::Published(snapshot, id) => snapshot.node(*id)?.kind(),
                Source::Staged(key) => &self.node(*key)?.kind,
            };
            budget.retain_payload(
                source::storage::kind(original, || self.source_step(cancel))?,
                cancel,
            )?;
        }
        let original = self.source_kind(source)?;
        if let Source::Published(snapshot, _) = source
            && snapshot.owner() != self.store
            && matches!(&original, DomNodeKind::Element(element)
                if !matches!(element.custom_element_registry(), crate::DomRegistryBinding::Null))
        {
            return Err(DomError::NotSupported);
        }
        let script = match source {
            Source::Published(s, id) => s.node(*id)?.node.script.map(|v| v.flags),
            Source::Staged(k) => self.node(*k)?.script.map(|v| v.flags),
        };
        let formation = source::formation::Formation::new(self, cancel)?;
        let kind = match &original {
            DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_) => {
                return Err(DomError::NotSupported);
            }
            DomNodeKind::Element(e) => DomNodeKind::Element(e.clone_initialized(
                self.document_environment(document)?,
                cancel,
                formation.as_ref(),
            )?),
            kind => {
                if let Some(budget) = &formation {
                    budget.retain_payload(
                        source::storage::kind(kind, || self.source_step(cancel))?,
                        cancel,
                    )?;
                }
                kind.clone()
            }
        };
        let copy = self.allocate(kind, document, limits, cancel)?;
        drop(formation);
        self.initialize_native_element(copy, limits, cancel)?;
        if let DomNodeKind::Element(element) = &original {
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
                self.append_attribute(copy, attribute, cause, limits, cancel)?;
            }
            if let Some(nonce) = element.nonce() {
                self.set_nonce(copy, nonce, limits, cancel)?;
            }
            if let Some(state) = element.svg_script_state() {
                let DomNodeKind::Element(e) = &mut self.node_mut(copy).kind else {
                    unreachable!()
                };
                *e.svg_script_state_mut().expect("fresh SVG Script") = state.clone_for_element();
            }
            if let Some(state) = element.input_state() {
                self.clone_input_overlay(copy, state, limits, cancel)?;
            }
            if let Some(state) = element.textarea_state() {
                self.clone_textarea_overlay(copy, state, limits, cancel)?;
            }
        }
        if let Some(flags) = script {
            self.node_mut(copy)
                .script
                .as_mut()
                .expect("fresh HTML Script")
                .flags
                .clone_already_started_from(flags);
        }
        Ok(copy)
    }
}

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
impl Draft {
    fn source_kind(&self, source: &Source) -> Result<DomNodeKind, DomError> {
        Ok(match source {
            Source::Published(s, id) => s.node(*id)?.kind().clone(),
            Source::Staged(k) => self.node(*k)?.kind.clone(),
        })
    }
    fn source_child(
        &self,
        source: &Source,
        index: usize,
        template: bool,
    ) -> Result<Option<Source>, DomError> {
        Ok(match source {
            Source::Published(s, id) => {
                let n = s.node(*id)?;
                let n = if template {
                    if let Some(t) = n.as_template() {
                        s.node(t.contents())?
                    } else {
                        return Ok(None);
                    }
                } else {
                    n
                };
                n.children()
                    .get(index)
                    .map(|&id| Source::Published(s.clone(), id))
            }
            Source::Staged(k) => {
                let n = self.node(*k)?;
                let n = if template {
                    if let Some(t) = n.template_contents {
                        self.node(t)?
                    } else {
                        return Ok(None);
                    }
                } else {
                    n
                };
                n.children.get(index).copied().map(Source::Staged)
            }
        })
    }
    fn source_shadow(
        &self,
        source: &Source,
    ) -> Result<Option<(Source, crate::DomShadowRootState)>, DomError> {
        match source {
            Source::Published(s, id) => s
                .node(*id)?
                .hosted_shadow_root()
                .map(|r| Ok((Source::Published(s.clone(), r), *s.shadow_root(r)?.state())))
                .transpose(),
            Source::Staged(k) => self
                .node(*k)?
                .shadow_root
                .map(|r| match &self.node(r)?.kind {
                    DomNodeKind::ShadowRoot(state) => Ok((Source::Staged(r), *state)),
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
        let root = self.begin_clone(
            Source::Published(snapshot.clone(), source),
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
        let copy = self.clone_single(&source, document, limits, cancel)?;
        self.push_effect(
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
        )?;
        Ok(copy)
    }
    pub(super) fn clone_effect(
        &mut self,
        mut frame: Frame,
        scoped: bool,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if frame.subtree
            && let Some(source) = self.source_child(&frame.source, frame.template_child, true)?
        {
            frame.template_child += 1;
            let parent = self
                .node(frame.copy)?
                .template_contents
                .expect("fresh contents");
            let document = self.node(parent)?.document;
            self.push_effect(select::Effect::Clone(frame), scoped)?;
            self.begin_clone(
                source,
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
            self.push_effect(select::Effect::Clone(frame), scoped)?;
            if let Placement::BeforeChildren(parent) = placement {
                self.pre_insert(copy, parent, None, limits, cancel)?;
            }
            return Ok(());
        }
        if frame.subtree
            && let Some(source) = self.source_child(&frame.source, frame.next_child, false)?
        {
            frame.next_child += 1;
            let parent = frame.copy;
            let document = frame.document;
            self.push_effect(select::Effect::Clone(frame), scoped)?;
            self.begin_clone(
                source,
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
            if let Some((source, state)) = self.source_shadow(&frame.source)?
                && state.clonable()
            {
                let root =
                    self.attach_shadow(frame.copy, state.properties().into(), limits, cancel)?;
                let document = frame.document;
                self.push_effect(select::Effect::Clone(frame), scoped)?;
                self.push_effect(
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
        let original = self.source_kind(source)?;
        let script = match source {
            Source::Published(s, id) => s.node(*id)?.node.script.map(|v| v.flags),
            Source::Staged(k) => self.node(*k)?.script.map(|v| v.flags),
        };
        let kind = match &original {
            DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_) => {
                return Err(DomError::NotSupported);
            }
            DomNodeKind::Element(e) => DomNodeKind::Element(
                e.clone_initialized(self.native_environment(document)?, cancel)?,
            ),
            kind => kind.clone(),
        };
        let copy = self.allocate(kind, document, limits)?;
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
                self.append_attribute(copy, attribute.clone(), cause, limits, cancel)?;
            }
            if let Some(nonce) = element.nonce() {
                self.set_nonce(copy, nonce.clone(), limits, cancel)?;
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
                self.clone_textarea_overlay(copy, state, limits)?;
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

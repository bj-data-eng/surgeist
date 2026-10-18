//! Scalar node edits are provisional until their admitted commit. Sequence
//! members of document metadata are empty sentinels; selected sequence owners
//! live beside these fields and can only be edited through typed operations.
use super::super::*;

#[derive(Clone)]
pub(in crate::store) struct NodeFields {
    pub active_range: crate::DomActiveRangeState,
    pub relevant_global: Option<crate::hosted::DomHostedGlobalId>,
    pub document: Key,
    pub kind: DomNodeKind,
    pub parent: Option<Key>,
    pub shadow_root: Option<Key>,
    pub host: Option<Key>,
    pub template_contents: Option<Key>,
    pub template_patch: Option<crate::template::TemplatePatch<Key>>,
    pub template_document: Option<crate::template::TemplateDocument<Key>>,
    pub manual_slot: Option<Key>,
    pub assigned_slot: Option<Key>,
    pub reflected_targets: Option<crate::reflected::ReflectedTargets<Key>>,
    pub control_owner: Option<Key>,
    pub option_select: Option<Key>,
    pub form_past_names: Option<super::Names>,
    pub script: Option<crate::script::ScriptState<Key>>,
    pub style_parser_document: Option<Key>,
    pub base: Option<crate::base::BaseState<Key>>,
    pub base_document: Option<crate::base::BaseDocument<Key>>,
    pub link: Option<crate::base::LinkState<Key>>,
    pub image: Option<crate::image::ImageState<Key>>,
    pub hyperlink: Option<crate::hyperlink::HyperlinkState<Key>>,
    pub encoding_url_document: Option<crate::hyperlink::EncodingUrlDocument>,
    pub interactive_document: Option<crate::dialog::DocumentInteractiveState<Key>>,
    pub autofocus_document: Option<crate::autofocus::DocumentAutofocusState<Key>>,
}

pub(in crate::store) struct NodeFieldsMut<'a> {
    pub(super) draft: &'a mut Draft,
    pub(super) key: Key,
    pub(super) candidate: Option<custody::Held<Box<NodeFields>>>,
    pub(super) retain_undo: bool,
}
impl std::ops::Deref for NodeFieldsMut<'_> {
    type Target = NodeFields;
    fn deref(&self) -> &Self::Target {
        self.candidate
            .as_ref()
            .expect("uncommitted node field candidate")
            .value()
    }
}
impl std::ops::DerefMut for NodeFieldsMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.candidate
            .as_mut()
            .expect("uncommitted node field candidate")
            .value_mut()
    }
}
impl NodeFieldsMut<'_> {
    pub fn commit_charged(
        mut self,
        charge: Option<source::checkpoint::Charge>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(incoming) = charge.and_then(source::checkpoint::Charge::into_debit) {
            self.candidate
                .as_mut()
                .expect("selected candidate")
                .debit_mut()
                .absorb(incoming)
                .expect("same controlled field ledger");
        }
        self.commit(cancel)
    }
    pub fn commit_formed(
        mut self,
        formation: Option<source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(formation) = formation
            && let Some(incoming) = formation.into_debit()
        {
            self.candidate
                .as_mut()
                .expect("selected candidate")
                .debit_mut()
                .absorb(incoming)
                .expect("same controlled field ledger");
        }
        self.commit(cancel)
    }
    pub fn commit(mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        let candidate = self.candidate.take().expect("one field candidate commit");
        self.draft
            .commit_node_fields(self.key, candidate, self.retain_undo, cancel)
    }
}

pub(in crate::store) struct ElementFieldsMut<'a>(pub(super) NodeFieldsMut<'a>);
impl std::ops::Deref for ElementFieldsMut<'_> {
    type Target = DomElement;
    fn deref(&self) -> &Self::Target {
        let DomNodeKind::Element(element) = &self.0.kind else {
            unreachable!("validated Element")
        };
        element
    }
}
impl std::ops::DerefMut for ElementFieldsMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let DomNodeKind::Element(element) = &mut self.0.kind else {
            unreachable!("validated Element")
        };
        element
    }
}
impl ElementFieldsMut<'_> {
    pub fn commit_charged(
        self,
        charge: Option<source::checkpoint::Charge>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.0.commit_charged(charge, cancel)
    }
    pub fn commit_formed(
        self,
        formation: Option<source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.0.commit_formed(formation, cancel)
    }
    pub fn commit(self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.0.commit(cancel)
    }
}

impl NodeFields {
    pub(in crate::store) fn from_canonical(node: &Node) -> Self {
        let key = Key::Existing;
        Self {
            active_range: node.active_range,
            relevant_global: node.relevant_global,
            document: key(node.document.0),
            kind: node.kind.clone(),
            parent: node.parent.map(key),
            shadow_root: node.shadow_root.map(key),
            host: node.host.map(key),
            template_contents: node.template_contents.map(key),
            template_patch: node.template_patch.map(|v| v.map(key)),
            template_document: node.template_document.map(|v| v.map(key)),
            manual_slot: node.manual_slot.map(key),
            assigned_slot: node.assigned_slot.map(key),
            reflected_targets: node.reflected_targets.map(|v| v.map(key)),
            control_owner: node.control_owner.map(key),
            option_select: node.option_select.map(key),
            form_past_names: node.form_past_names.as_ref().map(super::Names::canonical),
            script: node.script.map(|v| v.map(key)),
            style_parser_document: node.style_parser_document.map(key),
            base: node.base.as_ref().map(|v| v.map(key)),
            link: node.link.as_ref().map(|v| v.map(key)),
            image: node.image.as_ref().map(|v| v.map(key)),
            hyperlink: node.hyperlink.as_ref().map(|v| v.map(key)),
            encoding_url_document: node.encoding_url_document.clone(),
            base_document: node
                .base_document
                .as_ref()
                .map(|v| crate::base::BaseDocument {
                    facts: v.facts.clone(),
                    first: v.first.map(key),
                    render_blocking: Vec::new(),
                }),
            autofocus_document: node.autofocus_document.as_ref().map(|v| match v {
                canonical::Autofocus::Known { processed, .. } => {
                    crate::autofocus::DocumentAutofocusState::Known {
                        candidates: Vec::new(),
                        processed: *processed,
                    }
                }
                canonical::Autofocus::Unavailable => {
                    crate::autofocus::DocumentAutofocusState::Unavailable
                }
            }),
            interactive_document: node.interactive_document.as_ref().map(|v| match v {
                canonical::Interactive::Known { .. } => {
                    crate::dialog::DocumentInteractiveState::native_closed()
                }
                canonical::Interactive::Unavailable => {
                    crate::dialog::DocumentInteractiveState::Unavailable
                }
            }),
        }
    }
    pub(in crate::store) fn payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        let mut payload = source::storage::kind(&self.kind, &mut step)?;
        if let Some(frozen) = self.base.as_ref().and_then(|v| v.frozen.as_ref()) {
            step()?;
            payload.producer(frozen.value.retained_storage_usage())?;
        }
        if let Some(facts) = self.base_document.as_ref().and_then(|v| v.facts.as_ref()) {
            step()?;
            payload.producer(facts.retained_storage_usage())?;
        }
        if let Some(parsed) = self.hyperlink.as_ref().and_then(|v| v.parsed.as_ref()) {
            step()?;
            let usage = parsed.value.retained_storage_usage();
            payload.add_bytes(
                usage
                    .retained_bytes()
                    .checked_sub(size_of::<surgeist_html_semantics::HtmlHyperlinkUrlSnapshot>())
                    .ok_or(DomError::RepresentationExhausted)?,
            )?;
            payload.add_units(usage.utf16_extent())?;
        }
        if let Some(facts) = self
            .encoding_url_document
            .as_ref()
            .and_then(|v| v.facts.as_ref())
        {
            step()?;
            payload.producer(facts.retained_storage_usage())?;
        }
        if let Some(image) = &self.image {
            payload.string(image.native.current_request().current_url())?;
        }
        Ok(payload)
    }
    pub(in crate::store) fn usage(
        &self,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<custody::Usage, DomError> {
        let payload = self.payload(step)?;
        Ok(custody::Usage::new(
            size_of::<NodeFields>()
                .checked_add(payload.bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            payload.units,
        ))
    }
}

impl NodeFields {
    pub(in crate::store) fn aliases(&mut self, ctx: &source::AllocationIds) {
        let map = |key| source::alias_key(key, ctx);
        self.document = map(self.document);
        self.parent = self.parent.map(map);
        self.shadow_root = self.shadow_root.map(map);
        self.host = self.host.map(map);
        self.template_contents = self.template_contents.map(map);
        self.template_patch = self.template_patch.map(|v| v.map(map));
        self.template_document = self.template_document.map(|v| v.map(map));
        self.manual_slot = self.manual_slot.map(map);
        self.assigned_slot = self.assigned_slot.map(map);
        self.reflected_targets = self.reflected_targets.map(|v| v.map(map));
        self.control_owner = self.control_owner.map(map);
        self.option_select = self.option_select.map(map);
        self.script = self.script.map(|v| v.map(map));
        self.style_parser_document = self.style_parser_document.map(map);
        if let Some(base) = &mut self.base
            && let Some(frozen) = &mut base.frozen
        {
            frozen.document = map(frozen.document);
        }
        if let Some(base) = &mut self.base_document {
            base.first = base.first.map(map);
        }
        if let Some(link) = &mut self.link {
            link.parser_document = link.parser_document.map(map);
        }
        if let Some(image) = &mut self.image {
            image.dimension_source = map(image.dimension_source);
        }
        if let Some(hyperlink) = &mut self.hyperlink
            && let Some(parsed) = &mut hyperlink.parsed
        {
            parsed.document = map(parsed.document);
        }
    }
}

impl Draft {
    pub(in crate::store) fn with_node_fields_formed<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        formation: Option<source::formation::Formation>,
        edit: impl FnOnce(&mut NodeFields) -> R,
    ) -> Result<R, DomError> {
        let mut fields = self.node_fields_mut(key, cancel)?;
        let result = edit(&mut fields);
        fields.commit_formed(formation, cancel)?;
        Ok(result)
    }
    pub(in crate::store) fn with_element_fields_formed<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        formation: Option<source::formation::Formation>,
        edit: impl FnOnce(&mut DomElement) -> R,
    ) -> Result<R, DomError> {
        let mut fields = self.element_fields_mut(key, cancel)?;
        let result = edit(&mut fields);
        fields.commit_formed(formation, cancel)?;
        Ok(result)
    }
}

impl Draft {
    pub(in crate::store) fn with_node_fields_charged<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        charge: Option<source::checkpoint::Charge>,
        edit: impl FnOnce(&mut NodeFields) -> R,
    ) -> Result<R, DomError> {
        let mut fields = self.node_fields_mut(key, cancel)?;
        let result = edit(&mut fields);
        fields.commit_charged(charge, cancel)?;
        Ok(result)
    }
    pub(in crate::store) fn with_element_fields_charged<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        charge: Option<source::checkpoint::Charge>,
        edit: impl FnOnce(&mut DomElement) -> R,
    ) -> Result<R, DomError> {
        let mut fields = self.element_fields_mut(key, cancel)?;
        let result = edit(&mut fields);
        fields.commit_charged(charge, cancel)?;
        Ok(result)
    }
}

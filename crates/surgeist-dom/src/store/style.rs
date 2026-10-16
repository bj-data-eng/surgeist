//! The canonical source engine's Style cursor. Runtime supplies real external
//! owner work; ordinary atomic edits stop without publishing that work.
use super::*;
pub(super) mod checkpoint;
pub(crate) mod formation;
pub(super) mod headroom;
pub(super) mod initial;
pub(crate) mod operation;
pub(crate) mod storage;
use surgeist_html::{
    HtmlStyleUpdateAction as Action, HtmlStyleUpdateCursor, HtmlStyleUpdateReply as Reply,
};

#[derive(Clone, Copy)]
pub(super) struct Frame {
    key: Key,
    cursor: Cursor,
    invocation: Invocation,
}
#[derive(Clone, Copy)]
enum Invocation {
    Unissued,
    Exposed(u64),
}
#[derive(Clone, Copy)]
enum Cursor {
    Update(HtmlStyleUpdateCursor),
    OwnerAttributes,
    SetDisabled(bool),
    Complete,
}
impl Cursor {
    fn action(self) -> crate::DomStyleAction {
        match self {
            Self::Update(c) => crate::DomStyleAction::Update(c.action()),
            Self::OwnerAttributes => crate::DomStyleAction::OwnerAttributes,
            Self::SetDisabled(value) => crate::DomStyleAction::SetDisabled(value),
            Self::Complete => crate::DomStyleAction::Update(Action::Complete),
        }
    }
    fn reply(&mut self, reply: Reply) -> Result<(), surgeist_html::HtmlStyleProtocolError> {
        match self {
            Self::Update(cursor) => cursor.reply(reply),
            Self::OwnerAttributes | Self::SetDisabled(_) if reply == Reply::Applied => {
                *self = Self::Complete;
                Ok(())
            }
            _ => Err(surgeist_html::HtmlStyleProtocolError),
        }
    }
}
pub(super) struct PublicationMap {
    pub slots: Vec<Slot>,
    pub occupants: Vec<Option<Key>>,
    pub ids: Vec<Option<NodeId>>,
}
/// Apply the one Source growth plan admitted by its actual Draft owner. The
/// ordinary path retains its existing allocator policy. No callback/allocation
/// may intervene between planning and this owned bank reservation.
pub(super) fn reserve_bank<T>(
    bank: &mut Vec<T>,
    additional: usize,
    admitted: Option<usize>,
) -> Result<(), DomError> {
    match admitted {
        Some(amount) => bank
            .try_reserve_exact(amount)
            .map_err(|_| DomError::AllocationFailed),
        None => reserve(bank, additional),
    }
}
impl Draft {
    pub(super) fn source_ascii(
        &self,
        value: &DocumentString,
        expected: &str,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        if value.len() != expected.len() {
            return Ok(false);
        }
        for (actual, expected) in value.code_units().iter().zip(expected.bytes()) {
            self.source_step(cancel)?;
            if *actual != u16::from(expected) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn style_potentially_render_blocking(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        let node = self.node(key)?;
        let state = self
            .element(key)?
            .style_state()
            .expect("Style")
            .creation_state();
        let explicit = surgeist_html::blocking_tokens_with(
            self.control_attribute(key, surgeist_html::KnownHtmlAttribute::Blocking, cancel)?,
            || self.source_step(cancel),
        )?;
        Ok(surgeist_html::style_is_potentially_render_blocking(
            explicit,
            state,
            node.style_parser_document == Some(node.document),
        ))
    }
    fn style_render_blocking_required(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<bool>, DomError> {
        if !self.style_potentially_render_blocking(key, cancel)? {
            return Ok(Some(false));
        }
        let media =
            self.control_attribute(key, surgeist_html::KnownHtmlAttribute::Media, cancel)?;
        self.source_step(cancel)?;
        // Missing/empty media is the source-defined all-media default. An actual
        // nonempty query requires a qualified environment match, not readiness.
        Ok(media.is_none_or(DocumentString::is_empty).then_some(true))
    }
    fn style_document_has_body(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        use surgeist_html::HtmlDocumentBodyRole as Role;
        let mut root = None;
        for child in &self.node(document)?.children {
            self.source_step(cancel)?;
            if let DomNodeKind::Element(element) = &self.node(*child)?.kind {
                if surgeist_html::document_body_role_with(element.name(), |_| {
                    self.source_step(cancel)
                })? != Role::Html
                {
                    return Ok(false);
                }
                root = Some(*child);
                break;
            }
        }
        let Some(root) = root else { return Ok(false) };
        for child in &self.node(root)?.children {
            self.source_step(cancel)?;
            if let DomNodeKind::Element(element) = &self.node(*child)?.kind
                && surgeist_html::document_body_role_with(element.name(), |_| {
                    self.source_step(cancel)
                })? == Role::Body
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn style_block_rendering(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        let document = self.node(key)?.document;
        let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let html = self.source_ascii(metadata.content_type(), "text/html", cancel)?;
        if !html {
            return Ok(());
        }
        let body = self.style_document_has_body(document, cancel)?;
        if !surgeist_html::document_allows_render_blocking(html, body) {
            return Ok(());
        }
        let list = &self
            .node(document)?
            .base_document
            .as_ref()
            .ok_or(DomError::WrongNodeKind)?
            .render_blocking;
        for member in list {
            self.source_step(cancel)?;
            if *member == key {
                return Ok(());
            }
        }
        let amount = self.source_reserve_amount::<Key>(list.len(), list.capacity(), 1, cancel)?;
        let list = &mut self
            .node_mut(document)
            .base_document
            .as_mut()
            .expect("Document")
            .render_blocking;
        match amount {
            Some(amount) => list
                .try_reserve_exact(amount)
                .map_err(|_| DomError::AllocationFailed)?,
            None => reserve(list, 1)?,
        }
        self.source_admit(0, cancel)?;
        self.node_mut(document)
            .base_document
            .as_mut()
            .expect("Document")
            .render_blocking
            .push(key);
        Ok(())
    }
    fn style_unblock_rendering(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        let document = self.node(key)?.document;
        let length = self
            .node(document)?
            .base_document
            .as_ref()
            .ok_or(DomError::WrongNodeKind)?
            .render_blocking
            .len();
        for _ in 0..length {
            self.source_step(cancel)?;
        }
        self.node_mut(document)
            .base_document
            .as_mut()
            .expect("Document")
            .render_blocking
            .retain(|member| *member != key);
        Ok(())
    }
    pub(super) fn retire_style_render_blocking(
        &mut self,
        retired: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        for index in 0..self.existing.len() + self.created.len() {
            self.source_step(cancel)?;
            let key = if index < self.existing.len() {
                Key::Existing(index)
            } else {
                Key::Created(index - self.existing.len())
            };
            let Ok(node) = self.node(key) else { continue };
            let Some(document) = &node.base_document else {
                continue;
            };
            for _ in &document.render_blocking {
                self.source_step(cancel)?;
            }
            self.node_mut(key)
                .base_document
                .as_mut()
                .expect("Document")
                .render_blocking
                .retain(|member| *member != retired);
        }
        Ok(())
    }
    pub(super) fn source_admit_units(
        &self,
        extra: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(limits) = self.source_limits else {
            return Ok(());
        };
        let mut units = self
            .source_other_units
            .checked_add(extra)
            .and_then(|n| n.checked_add(self.source_temporary_units.load(Ordering::Relaxed)))
            .ok_or(DomError::RepresentationExhausted)?;
        units = units
            .checked_add(self.source_storage_units(cancel)?)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            units,
            limits.retained_units,
            DomResource::StyleRetainedUnits,
        )
    }
    pub(super) fn source_storage_units(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        let mut units = 0usize;
        for node in self.existing.iter().chain(&self.created).flatten() {
            units = units
                .checked_add(storage::node(node, || self.source_step(cancel))?.units)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(units)
    }
    /// Preadmit the replacement bank while its old backing remains live. The
    /// caller applies this exact growth policy and then checks actual storage.
    pub(super) fn source_reserve_amount<T>(
        &self,
        length: usize,
        capacity: usize,
        additional: usize,
        cancel: &AtomicBool,
    ) -> Result<Option<usize>, DomError> {
        if self.source_limits.is_none() {
            return Ok(None);
        }
        self.source_step(cancel)?;
        let required = length
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if required <= capacity {
            self.source_admit(0, cancel)?;
            return Ok(Some(0));
        }
        let maximum = isize::MAX as usize / size_of::<T>().max(1);
        let target = capacity
            .checked_mul(2)
            .filter(|n| *n <= maximum)
            .unwrap_or(required)
            .max(required);
        let bytes = target
            .checked_mul(size_of::<T>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        self.source_admit(bytes, cancel)?;
        Ok(Some(target - length))
    }
    pub(super) fn source_admit(
        &self,
        additional_bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(limits) = self.source_limits else {
            return Ok(());
        };
        let total = self
            .source_storage_bytes(cancel)?
            .checked_add(self.source_other_bytes)
            .and_then(|n| n.checked_add(self.source_formation_bytes.load(Ordering::Relaxed)))
            .and_then(|n| n.checked_add(additional_bytes))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            total,
            limits.retained_bytes,
            DomResource::StyleRetainedBytes,
        )
    }
    pub(super) fn source_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if let Some(limits) = self.source_limits {
            let next = self
                .source_work
                .load(Ordering::Relaxed)
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            if next > limits.work {
                return Err(DomError::StyleWorkLimit { limit: limits.work });
            }
            self.source_work.store(next, Ordering::Relaxed);
        }
        Ok(())
    }

    pub(super) fn style_update(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !matches!(&self.node(key)?.kind, DomNodeKind::Element(e) if e.style_state().is_some()) {
            return Ok(());
        }
        if self.headroom_style(key, headroom::Kind::Update, cancel)? {
            return Ok(());
        }
        if self.source_limits.is_some() {
            self.element(key)?
                .style_state()
                .expect("Style")
                .next_association
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        let invocation = Invocation::Unissued;
        self.push_effect(
            select::Effect::Style(Frame {
                key,
                cursor: Cursor::Update(HtmlStyleUpdateCursor::new()),
                invocation,
            }),
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }
    pub(super) fn style_effect(
        &mut self,
        mut frame: Frame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let action = match frame.cursor {
            Cursor::OwnerAttributes => {
                self.push_effect(select::Effect::Style(frame), self.effect_scoped, cancel)?;
                return Err(DomError::StyleOwnerAttributesCompositionRequired);
            }
            Cursor::SetDisabled(_) => {
                self.push_effect(select::Effect::Style(frame), self.effect_scoped, cancel)?;
                return Err(DomError::StyleDisabledCompositionRequired);
            }
            Cursor::Complete => return Ok(()),
            Cursor::Update(cursor) => cursor.action(),
        };
        let reply = match action {
            Action::RemoveAssociatedSheet => {
                let state = match &self.node(frame.key)?.kind {
                    DomNodeKind::Element(element) => element.style_state().expect("actual Style"),
                    _ => unreachable!("actual Style"),
                };
                if state.availability == crate::DomStyleSheetAvailability::PendingParsedProcessing {
                    return Err(DomError::StyleProcessingUnavailable);
                }
                if state.sheet.is_some() {
                    self.push_effect(select::Effect::Style(frame), self.effect_scoped, cancel)?;
                    return Err(DomError::StyleCompositionRequired(
                        Action::RemoveAssociatedSheet,
                    ));
                }
                Reply::Applied
            }
            Action::ReadConnected => Reply::Connected(self.shadow_connected(frame.key, cancel)?),
            Action::ReadType => Reply::TypeEligible(surgeist_html::style_type_allows_css_with(
                self.control_attribute(frame.key, surgeist_html::KnownHtmlAttribute::Type, cancel)?,
                || self.effect_step(cancel),
            )?),
            Action::ReadScriptBlocking
                if !self
                    .element(frame.key)?
                    .style_state()
                    .expect("Style")
                    .creation_state()
                    .created_by_document_parser()
                    || self.node(frame.key)?.style_parser_document
                        != Some(self.node(frame.key)?.document) =>
            {
                // The first conjunct refers to THIS Document's parser, not a
                // historical boolean after template ownership or adoption.
                Reply::Blocking(false)
            }
            Action::ReadRenderBlocking => {
                match self.style_render_blocking_required(frame.key, cancel)? {
                    Some(required) => Reply::Blocking(required),
                    None => {
                        self.push_effect(select::Effect::Style(frame), self.effect_scoped, cancel)?;
                        return Err(DomError::StyleCompositionRequired(
                            Action::ReadRenderBlocking,
                        ));
                    }
                }
            }
            Action::BlockRendering => {
                self.style_block_rendering(frame.key, cancel)?;
                Reply::Applied
            }
            Action::Complete => return Ok(()),
            action => {
                self.push_effect(select::Effect::Style(frame), self.effect_scoped, cancel)?;
                return Err(DomError::StyleCompositionRequired(action));
            }
        };
        frame
            .cursor
            .reply(reply)
            .map_err(|_| DomError::StyleProtocol)?;
        self.push_effect(select::Effect::Style(frame), self.effect_scoped, cancel)
    }
    pub(super) fn set_style_disabled(
        &mut self,
        key: Key,
        disabled: bool,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let state = self
            .element(key)?
            .style_state()
            .ok_or(DomError::WrongNodeKind)?;
        if state.availability == crate::DomStyleSheetAvailability::PendingParsedProcessing {
            return Err(DomError::StyleProcessingUnavailable);
        }
        // HTML877d disabled setter returns before any stored flag or update
        // when the actual association is null. The content attribute is separate.
        let associated = state.sheet.is_some();
        if self.headroom_style(key, headroom::Kind::SetDisabled, cancel)? {
            return Ok(());
        }
        if !associated {
            return Ok(());
        }
        self.source_step(cancel)?;
        let invocation = Invocation::Unissued;
        self.push_effect(
            select::Effect::Style(Frame {
                key,
                cursor: Cursor::SetDisabled(disabled),
                invocation,
            }),
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }
    pub(super) fn style_attribute_change(
        &mut self,
        key: Key,
        name: &crate::QualifiedName,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.element(key)?.style_state().is_none() {
            return Ok(());
        }
        let attribute = surgeist_html::KnownHtmlAttribute::from_name(name);
        let Some(attribute) = attribute else {
            return Ok(());
        };
        if name.namespace().uri().is_none()
            && attribute == surgeist_html::KnownHtmlAttribute::Blocking
        {
            if !self.style_potentially_render_blocking(key, cancel)? {
                self.style_unblock_rendering(key, cancel)?;
            }
            return Ok(());
        }
        if surgeist_html::style_attribute_effect(name.namespace(), attribute)
            != surgeist_html::HtmlStyleAttributeEffect::OwnerAttributes
        {
            return Ok(());
        }
        let state = self.element(key)?.style_state().expect("Style");
        if state.availability == crate::DomStyleSheetAvailability::PendingParsedProcessing {
            return Err(DomError::StyleProcessingUnavailable);
        }
        let associated = state.sheet.is_some();
        if self.headroom_style(key, headroom::Kind::OwnerAttributes, cancel)? {
            return Ok(());
        }
        if !associated {
            return Ok(());
        }
        self.source_step(cancel)?;
        let invocation = Invocation::Unissued;
        self.push_effect(
            select::Effect::Style(Frame {
                key,
                cursor: Cursor::OwnerAttributes,
                invocation,
            }),
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }
    pub(super) fn style_tree_change(
        &mut self,
        key: Key,
        old_parent: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(state) = (match &self.node(key)?.kind {
            DomNodeKind::Element(e) => e.style_state().map(crate::DomStyleState::creation_state),
            _ => None,
        }) else {
            return Ok(());
        };
        let trigger = if let Some(parent) = old_parent {
            surgeist_html::HtmlStyleTrigger::Removal {
                disconnected: self.shadow_connected(parent, cancel)?
                    && !self.shadow_connected(key, cancel)?,
                on_parser_stack: state.on_parser_stack(),
            }
        } else {
            surgeist_html::HtmlStyleTrigger::Insertion {
                connected: self.shadow_connected(key, cancel)?,
                on_parser_stack: state.on_parser_stack(),
            }
        };
        if surgeist_html::style_update_is_required(trigger) {
            self.style_update(key, limits, cancel)?;
        }
        Ok(())
    }
}

/// Original Draft keys survive every publication; only first exposure assigns a
/// canonical generation. Private construction uses the same created-node bank.
pub(super) struct PhaseMap {
    pub ids: Vec<Option<NodeId>>,
    pub private: Vec<bool>,
}
impl Draft {
    pub(super) fn published_id(&self, key: Key) -> Option<NodeId> {
        match key {
            Key::Existing(slot) => Some(NodeId {
                owner: self.store,
                slot,
                generation: self.generations[slot],
            }),
            Key::Created(i) => self.phase_map.as_ref().and_then(|m| m.ids[i]),
        }
    }
    pub(super) fn is_private(&self, key: Key) -> bool {
        match key {
            Key::Existing(_) => false,
            Key::Created(i) => self.phase_map.as_ref().is_some_and(|m| m.private[i]),
        }
    }
    pub(super) fn mark_source_private(&mut self, key: Key) {
        if let (Some(m), Key::Created(i)) = (&mut self.phase_map, key) {
            debug_assert!(m.ids[i].is_none(), "published nodes cannot become private");
            m.private[i] = true;
        }
    }
    pub(super) fn expose_source_subtree(
        &mut self,
        root: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // Includes actual hosted/auxiliary descendants, preserving one storage
        // owner and its actual document/cache closure.
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, 1, cancel)?;
        pending.push(root);
        while !pending.is_empty() {
            self.source_step(cancel)?;
            let key = pending.pop().expect("admitted public subtree node");
            self.effect_step(cancel)?;
            if let (Some(m), Key::Created(i)) = (&mut self.phase_map, key) {
                m.private[i] = false;
            }
            let n = self.node(key)?;
            self.select_reserve(&mut pending, n.children.len() + 2, cancel)?;
            for child in &n.children {
                self.source_step(cancel)?;
                pending.push(*child);
            }
            pending.extend(n.shadow_root);
            pending.extend(self.owned_template_contents(key, cancel)?);
        }
        Ok(())
    }
    pub(super) fn public_nodes(&self) -> usize {
        self.existing.iter().flatten().count()
            + self
                .created
                .iter()
                .enumerate()
                .filter(|(i, n)| n.is_some() && !self.is_private(Key::Created(*i)))
                .count()
    }
    pub(super) fn public_units(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        let mut units = self.units;
        for (i, n) in self.created.iter().enumerate() {
            if self.is_private(Key::Created(i))
                && let Some(n) = n
            {
                self.source_step(cancel)?;
                let names = form::name_units(n.form_past_names.as_ref(), cancel)?;
                let hidden = kind_units(&n.kind)?
                    .checked_add(n.base.as_ref().map_or(0, crate::base::BaseState::units))
                    .and_then(|n0| {
                        n0.checked_add(
                            n.hyperlink
                                .as_ref()
                                .map_or(0, crate::hyperlink::HyperlinkState::units),
                        )
                    })
                    .and_then(|n0| {
                        n0.checked_add(
                            n.encoding_url_document
                                .as_ref()
                                .map_or(0, crate::hyperlink::EncodingUrlDocument::units),
                        )
                    })
                    .and_then(|n0| {
                        n0.checked_add(
                            n.base_document
                                .as_ref()
                                .map_or(0, crate::base::BaseDocument::units),
                        )
                    })
                    .and_then(|n0| n0.checked_add(names))
                    .ok_or(DomError::RepresentationExhausted)?;
                units = units
                    .checked_sub(hidden)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        Ok(units)
    }
    pub(super) fn publication_map(
        &self,
        dom: &Dom,
        formation: Option<&formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<PublicationMap, DomError> {
        let mut occupants = Vec::new();
        formation::reserve(&mut occupants, dom.state.slots.len(), formation, cancel)?;
        occupants.resize(dom.state.slots.len(), None);
        for (i, n) in self.existing.iter().enumerate() {
            self.source_step(cancel)?;
            if n.is_some() {
                occupants[i] = Some(Key::Existing(i));
            }
        }
        if let Some(m) = &self.phase_map {
            for (i, id) in m.ids.iter().enumerate() {
                self.source_step(cancel)?;
                if let Some(id) = id
                    && self.created[i].is_some()
                {
                    debug_assert!(!m.private[i]);
                    occupants[id.slot] = Some(Key::Created(i));
                }
            }
        }
        let mut slots = Vec::new();
        formation::reserve(&mut slots, occupants.len(), formation, cancel)?;
        for (i, old) in dom.state.slots.iter().enumerate() {
            self.source_step(cancel)?;
            let destroyed = old.node.is_some() && occupants[i].is_none();
            slots.push(Slot {
                generation: if destroyed {
                    old.generation.checked_add(1).unwrap_or(old.generation)
                } else {
                    old.generation
                },
                retired: old.retired || (destroyed && old.generation == u64::MAX),
                node: None,
            });
        }
        let mut allocated = Vec::new();
        formation::reserve(&mut allocated, self.created.len(), formation, cancel)?;
        for (i, n) in self.created.iter().enumerate() {
            self.source_step(cancel)?;
            if n.is_none() || self.is_private(Key::Created(i)) {
                allocated.push(None);
                continue;
            }
            if let Some(id) = self.phase_map.as_ref().and_then(|m| m.ids[i]) {
                allocated.push(Some(id));
                continue;
            }
            let slot = if let Some(i) = occupants
                .iter()
                .enumerate()
                .position(|(i, k)| k.is_none() && !slots[i].retired)
            {
                i
            } else {
                formation::reserve(&mut slots, 1, formation, cancel)?;
                formation::reserve(&mut occupants, 1, formation, cancel)?;
                slots.push(Slot {
                    generation: 0,
                    retired: false,
                    node: None,
                });
                occupants.push(None);
                slots.len() - 1
            };
            occupants[slot] = Some(Key::Created(i));
            allocated.push(Some(NodeId {
                owner: dom.owner,
                slot,
                generation: slots[slot].generation,
            }));
        }
        Ok(PublicationMap {
            slots,
            occupants,
            ids: allocated,
        })
    }
}

impl Draft {
    pub(super) fn style_document_base(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<&DocumentString, DomError> {
        use surgeist_html_semantics::HtmlDocumentFirstBase;
        self.source_step(cancel)?;
        let node = self.node(document)?;
        let DomNodeKind::Document(metadata) = &node.kind else {
            return Err(DomError::WrongDocument);
        };
        // The admitted native owner establishes this actual fallback, rather
        // than asking the producer to guess srcdoc/about-base or navigable state.
        if metadata.document_environment().has_ever_had_navigable()
            || metadata.url().code_units() != [97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107]
        {
            return Err(DomError::NotSupported);
        }
        let first = match node
            .base_document
            .as_ref()
            .ok_or(DomError::WrongDocument)?
            .first
        {
            None => HtmlDocumentFirstBase::Absent,
            Some(key) => HtmlDocumentFirstBase::Present(
                self.node(key)?
                    .base
                    .as_ref()
                    .and_then(|b| b.frozen.as_ref())
                    .map(|f| &f.value),
            ),
        };
        surgeist_html_semantics::select_document_base_serialization(metadata.url(), first)
            .map_err(|missing| DomError::BaseFacts(crate::DomBaseFactsError::Missing(missing)))
    }
}

impl Draft {
    pub(super) fn map_public_ids(
        &self,
        keys: &[Key],
        id: &impl Fn(Key) -> NodeId,
        budget: Option<&formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Vec<NodeId>, DomError> {
        let mut out = Vec::new();
        formation::reserve(
            &mut out,
            keys.iter().filter(|k| !self.is_private(**k)).count(),
            budget,
            cancel,
        )?;
        for key in keys {
            self.source_step(cancel)?;
            if !self.is_private(*key) {
                out.push(id(*key));
            }
        }
        Ok(out)
    }
}

//! The canonical source engine's Style cursor. Runtime supplies real external
//! owner work; ordinary atomic edits stop without publishing that work.
use super::source::headroom;
use super::*;
use surgeist_html::{
    HtmlStyleUpdateAction as Action, HtmlStyleUpdateCursor, HtmlStyleUpdateReply as Reply,
};

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub(in crate::store) key: Key,
    pub(in crate::store) cursor: Cursor,
    pub(in crate::store) invocation: Invocation,
}
#[derive(Clone, Copy)]
pub(in crate::store) enum Invocation {
    Unissued,
    Exposed(u64),
}
#[derive(Clone, Copy)]
pub(in crate::store) enum Cursor {
    Update(HtmlStyleUpdateCursor),
    OwnerAttributes,
    SetDisabled(bool),
    Complete,
}
impl Cursor {
    pub(in crate::store) fn action(self) -> crate::DomStyleAction {
        match self {
            Self::Update(c) => crate::DomStyleAction::Update(c.action()),
            Self::OwnerAttributes => crate::DomStyleAction::OwnerAttributes,
            Self::SetDisabled(value) => crate::DomStyleAction::SetDisabled(value),
            Self::Complete => crate::DomStyleAction::Update(Action::Complete),
        }
    }
    pub(in crate::store) fn reply(
        &mut self,
        reply: Reply,
    ) -> Result<(), surgeist_html::HtmlStyleProtocolError> {
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

impl Draft {
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

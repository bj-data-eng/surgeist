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
use super::source::Invocation;
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
        for child in self.node(document)?.children.iter() {
            self.source_step(cancel)?;
            if let DomNodeKind::Element(element) = &self.node(child)?.kind {
                if surgeist_html::document_body_role_with(element.name(), |_| {
                    self.source_step(cancel)
                })? != Role::Html
                {
                    return Ok(false);
                }
                root = Some(child);
                break;
            }
        }
        let Some(root) = root else { return Ok(false) };
        for child in self.node(root)?.children.iter() {
            self.source_step(cancel)?;
            if let DomNodeKind::Element(element) = &self.node(child)?.kind
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
        let node = self.node(document)?;
        let list = &node
            .base_document
            .as_ref()
            .ok_or(DomError::WrongNodeKind)?
            .render_blocking;
        for member in list.iter() {
            self.source_step(cancel)?;
            if member == key {
                return Ok(());
            }
        }
        self.relations_insert(
            document,
            transaction::RelationField::RenderBlocking,
            list.len(),
            &[key],
            cancel,
        )
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
        self.relations_retain(
            document,
            transaction::RelationField::RenderBlocking,
            |member| member != key,
            cancel,
        )
    }
    pub(super) fn retire_style_render_blocking(
        &mut self,
        retired: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let owners = self.inbound_keys(retired, cancel)?;
        for key in owners.iter().copied() {
            self.source_step(cancel)?;
            let Ok(node) = self.node(key) else { continue };
            let Some(document) = &node.base_document else {
                continue;
            };
            for _ in document.render_blocking.iter() {
                self.source_step(cancel)?;
            }
            self.relations_retain(
                key,
                transaction::RelationField::RenderBlocking,
                |member| member != retired,
                cancel,
            )?;
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
        // Without live Ranges, the conservative fixed-counter scan needs no
        // prospective boundary validation. Otherwise run the original guards
        // through the first real wait before qualifying that exposure.
        if self.source_headroom.is_some()
            && !self.has_live_ranges(cancel)?
            && self.headroom_style(key, headroom::Kind::Update, cancel)?
        {
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
                return self.style_suspend(
                    frame,
                    DomError::StyleOwnerAttributesCompositionRequired,
                    cancel,
                );
            }
            Cursor::SetDisabled(_) => {
                return self.style_suspend(
                    frame,
                    DomError::StyleDisabledCompositionRequired,
                    cancel,
                );
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
                    return self.style_suspend(
                        frame,
                        DomError::StyleCompositionRequired(Action::RemoveAssociatedSheet),
                        cancel,
                    );
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
                        return self.style_suspend(
                            frame,
                            DomError::StyleCompositionRequired(Action::ReadRenderBlocking),
                            cancel,
                        );
                    }
                }
            }
            Action::BlockRendering => {
                self.style_block_rendering(frame.key, cancel)?;
                Reply::Applied
            }
            Action::Complete => return Ok(()),
            action => {
                return self.style_suspend(
                    frame,
                    DomError::StyleCompositionRequired(action),
                    cancel,
                );
            }
        };
        frame
            .cursor
            .reply(reply)
            .map_err(|_| DomError::StyleProtocol)?;
        self.push_effect(select::Effect::Style(frame), self.effect_scoped, cancel)
    }
    fn style_suspend(
        &mut self,
        frame: Frame,
        error: DomError,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let kind = match frame.cursor {
            Cursor::Update(_) => headroom::Kind::Update,
            Cursor::OwnerAttributes => headroom::Kind::OwnerAttributes,
            Cursor::SetDisabled(_) => headroom::Kind::SetDisabled,
            Cursor::Complete => return Err(DomError::StyleProtocol),
        };
        // This is the original cursor's first actually reached service wait.
        // Headroom does not acknowledge a hypothetical external reply.
        if self.headroom_style(frame.key, kind, cancel)? {
            return Ok(());
        }
        self.push_effect(select::Effect::Style(frame), self.effect_scoped, cancel)?;
        Err(error)
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

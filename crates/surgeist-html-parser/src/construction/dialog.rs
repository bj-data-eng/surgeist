//! Current Dialog values and compact interaction history in the original owner.
//! The HTML cursor owns ordering; this adapter fulfills only actual closed facts.
use super::*;
use surgeist_html::{
    HtmlDialogAction as Action, HtmlDialogCursor, HtmlDialogOperation as Operation,
    HtmlDialogOutcome, HtmlDialogReply as Reply,
};

pub(super) struct DialogMetadata {
    pub(super) value: usize,
    pub(super) first: Option<Missing>,
}
/// First reached unfulfilled source action, never a transition log or task.
pub(super) struct Missing {
    pub(super) subject: usize,
    pub(super) document: DocumentRef,
    pub(super) action: Action,
    pub(super) cause: Option<provenance::ConstructionCauseRef>,
}
/// Issued at each actual new owned Document constructor. The admitted original
/// operation set has no show/modal/pointer/popover/fullscreen/host ingress.
/// Shared ordinary hooks preserve the prefix, or retain its first missing fact.
pub(super) struct InteractionHistory {
    pub(super) first: Option<Missing>,
}
impl InteractionHistory {
    pub(super) const fn new() -> Self {
        Self { first: None }
    }
}
#[cfg(test)]
std::thread_local! {
    static ACTIONS: std::cell::RefCell<Option<Vec<Action>>> = const { std::cell::RefCell::new(None) };
    static CANCEL: std::cell::RefCell<Option<(Action, crate::HtmlParseCancellation)>> = const { std::cell::RefCell::new(None) };
}
#[cfg(test)]
fn observe(action: Action) {
    ACTIONS.with(|log| {
        if let Some(log) = log.borrow_mut().as_mut() {
            log.push(action);
        }
    });
    CANCEL.with(|hook| {
        let mut hook = hook.borrow_mut();
        if hook.as_ref().is_some_and(|(wanted, _)| *wanted == action) {
            hook.take().unwrap().1.cancel();
        }
    });
}
impl SessionStorage {
    pub(super) fn initialize_dialog(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        if !self.html_local(c, node, "dialog")? {
            return Ok(());
        }
        // Real creation (also reached by the original clone owner), before any
        // ordinary source effects. The current value is independent of attrs.
        let fresh = surgeist_html::HtmlDialogFreshState::fresh();
        let raw = literal_string(c, fresh.return_value().as_bytes())?;
        let (value, raw) = self.register_string_owned(c, raw, None)?;
        drop(value);
        self.prepare_payload_slot_owned(c)?;
        payload_write::<DialogMetadata>(c, 1)?;
        self.payloads.push_row(RetainedPayload::Dialog {
            node,
            state: DialogMetadata {
                value: raw,
                first: None,
            },
        })?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let count = element.attributes().len();
        // A fused installed backing is safe here: only the unique exact open
        // attribute invokes this rule; it reads no other or later attribute.
        // Source-created nodes are still detached, with their real Document.
        for position in 0..count {
            self.initial_dialog_attribute(c, node, position)?;
        }
        Ok(())
    }
    fn dialog_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        if !self.html_local(c, node, "dialog")? {
            return Ok(None);
        }
        self.payloads
            .joined(c, node, Join::Dialog)?
            .map(Some)
            .ok_or(SessionError::WrongToken)
    }
    fn dialog_missing(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        slot: usize,
        action: Action,
    ) -> Result<(), SessionError> {
        let document = self.node_document_ref(c, node)?;
        let owned = match document {
            DocumentRef::Owned(owner) => Some(self.document_slot(c, owner)?),
            DocumentRef::Captured(_) => None,
        };
        let document_missing = if let Some(owner) = owned {
            let Some(RetainedPayload::Document { interaction, .. }) =
                self.payloads.values.get(owner)
            else {
                return Err(latch_registry_row(c));
            };
            interaction.first.is_none()
        } else {
            false
        };
        let cause = self.associated_cause(c)?;
        let duplicate = if document_missing {
            let document = document.try_clone(c)?;
            let cause = self.associated_cause(c)?;
            payload_write::<Missing>(c, 1)?;
            Some(Missing {
                subject: node,
                document,
                action,
                cause,
            })
        } else {
            None
        };
        payload_write::<Missing>(c, 1)?;
        let Some(RetainedPayload::Dialog { state, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(latch_registry_row(c));
        };
        state.first = Some(Missing {
            subject: node,
            document,
            action,
            cause,
        });
        if let Some(missing) = duplicate {
            let Some(RetainedPayload::Document { interaction, .. }) =
                owned.and_then(|owner| self.payloads.values.get_mut(owner))
            else {
                return Err(SessionError::NodeDocumentUnavailable);
            };
            interaction.first = Some(missing);
        }
        Ok(())
    }
    fn dialog_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        operation: Operation,
        detached: Option<usize>,
    ) -> Result<(), SessionError> {
        let Some(slot) = self.dialog_slot(c, node)? else {
            return Ok(());
        };
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::Dialog { state, .. }) = self.payloads.values.get(slot) else {
            return Err(latch_registry_row(c));
        };
        if state.first.is_some() {
            // Structural parsing continues; unknown services cannot be resumed
            // or re-certified by a later independent false activity guard.
            return Ok(());
        }
        payload_write::<HtmlDialogCursor>(c, 1)?;
        let mut cursor = HtmlDialogCursor::new(operation);
        loop {
            c.charge(WorkCategory::ProviderCall, 1)?;
            let action = cursor.action();
            #[cfg(test)]
            observe(action);
            let reply = match action {
                Action::Complete(HtmlDialogOutcome::Completed) => return Ok(()),
                Action::ReadOpen => Reply::Open(self.form_attribute(c, node, "open")?.is_some()),
                Action::ReadFullyActive => match self.document_fully_active(c, node)? {
                    Some(value) => Reply::FullyActive(value),
                    None => return self.dialog_missing(c, node, slot, action),
                },
                Action::ReadConnected => {
                    Reply::Connected(self.shadow_including_connected(c, node, detached)?)
                }
                Action::ReadModal => Reply::Modal(false),
                Action::ReadWatcherPresent => Reply::WatcherPresent(false),
                Action::ReadTopLayerContains => Reply::TopLayerContains(false),
                Action::RemoveOpenDialog | Action::ClearModal => Reply::Applied,
                _ => return self.dialog_missing(c, node, slot, action),
            };
            // The current known prefix contains no operation which added this
            // newly created subject to any list/layer, created a watcher, or
            // set modal. Captured pre-existing whole lists are not read/reset:
            // the fresh result subject could not already be their member.
            payload_write::<HtmlDialogCursor>(c, 1)?;
            cursor.reply(reply).map_err(|_| SessionError::WrongToken)?;
        }
    }
    pub(super) fn initial_dialog_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        self.dialog_attribute_at(c, node, position, false)
    }
    pub(super) fn dialog_attribute_at(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
        old_present: bool,
    ) -> Result<(), SessionError> {
        if !self.html_local(c, node, "dialog")? {
            return Ok(());
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let attr = element
            .attributes()
            .as_slice()
            .get(position)
            .ok_or(SessionError::WrongToken)?;
        if attr.name().namespace().uri().is_some()
            || !exact_ascii(c, Some(attr.name().local_name()), "open")?
        {
            return Ok(());
        }
        self.dialog_open_changed(c, node, old_present, true)
    }
    pub(super) fn dialog_open_changed(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        old: bool,
        new: bool,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::ProviderCall, 1)?;
        let operation = surgeist_html::dialog_attribute_operation(
            &Namespace::none(),
            KnownHtmlAttribute::Open,
            old,
            new,
        )
        .expect("actual exact null-namespace open change");
        self.dialog_hook(c, node, operation, None)
    }
    pub(super) fn dialog_tree_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        removal: bool,
    ) -> Result<(), SessionError> {
        self.dialog_hook(
            c,
            node,
            if removal {
                Operation::Removed
            } else {
                Operation::Inserted
            },
            detached,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::Event};

    fn next_tag(lexer: &mut LexerFixture<'_>, c: &mut ParseControl) -> Tag {
        loop {
            match lexer.next(c, false).unwrap().unwrap() {
                Event::Token(event) => {
                    let token = event.into_token(c).unwrap();
                    if let TokenKind::Tag(tag) = token.kind {
                        return tag;
                    }
                    release_token_kind(&mut Some(token.kind), c).unwrap();
                }
                event => event.release(c).unwrap(),
            }
        }
    }
    fn actions() -> Vec<Action> {
        ACTIONS.with(|log| log.borrow_mut().take().unwrap())
    }
    fn watch() {
        ACTIONS.with(|log| *log.borrow_mut() = Some(vec![]));
    }
    fn request(source: &str) -> crate::HtmlDocumentParseRequest {
        crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source),
            super::super::tests::limits(),
        )
    }
    fn dialog(session: &ParserArenaSession, node: usize) -> &DialogMetadata {
        session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|payload| match payload {
                RetainedPayload::Dialog { node: owner, state } if *owner == node => Some(state),
                _ => None,
            })
            .unwrap()
    }
    #[test]
    fn actual_managed_changed_equal_removed_and_tree_hooks_keep_source_order() {
        let request =
            request("<dialog open=old><dialog open=new><dialog open=new><dialog x:open=ignored>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        // The real initialized driver already owns an html child. Insert
        // beneath it, preserving the parser's one-document-element invariant.
        let html = session
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .unwrap()
            .child_at(0)
            .unwrap()
            .position();
        let parent = session.active().unwrap().node(&mut c, html).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let tag = next_tag(&mut lexer, &mut c);
        let node = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &parent,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        watch();
        session
            .insert(&mut c, &parent, &node, None, Some(&proof))
            .unwrap();
        assert_eq!(
            actions(),
            vec![
                Action::ReadFullyActive,
                Action::Complete(HtmlDialogOutcome::Completed)
            ]
        );
        for _ in 0..2 {
            let mut tag = next_tag(&mut lexer, &mut c);
            let mut input = tag.attributes.try_pop(&mut c).unwrap();
            release_token_attributes(tag.attributes, &mut c).unwrap();
            tag.name.release(&mut c).unwrap();
            watch();
            session
                .set_managed_attribute(
                    &mut c,
                    &node,
                    &mut input,
                    &proof,
                    mutation::ManagedFamily::Dialog,
                )
                .unwrap();
            assert_eq!(
                actions(),
                vec![
                    Action::ReadFullyActive,
                    Action::Complete(HtmlDialogOutcome::Completed)
                ]
            );
        }
        watch();
        session
            .remove_managed_attribute(&mut c, &node, "open", mutation::ManagedFamily::Dialog)
            .unwrap();
        assert_eq!(
            actions(),
            vec![
                Action::RemoveOpenDialog,
                Action::ReadWatcherPresent,
                Action::ReadFullyActive,
                Action::Complete(HtmlDialogOutcome::Completed)
            ]
        );
        let mut tag = next_tag(&mut lexer, &mut c);
        let mut attribute = tag.attributes.try_pop(&mut c).unwrap();
        release_token_attributes(tag.attributes, &mut c).unwrap();
        tag.name.release(&mut c).unwrap();
        let attribute = attribute.take().unwrap();
        let storage = session.active_mut().unwrap();
        let container = storage
            .element_attribute_container(&mut c, node.index)
            .unwrap();
        let (name, name_slot) = storage.register_text(&mut c, attribute.name).unwrap();
        let (value, value_slot) = storage.register_text(&mut c, attribute.value).unwrap();
        watch();
        storage
            .append_registered_attribute(
                &mut c,
                node.index,
                container,
                Attribute::new(QualifiedName::new(Namespace::none(), None, name), value),
                [name_slot, value_slot],
            )
            .unwrap();
        assert!(
            actions().is_empty(),
            "literal x:open is not the exact open attribute"
        );
        watch();
        session.remove(&mut c, &node, Some(&proof)).unwrap();
        assert_eq!(
            actions(),
            vec![
                Action::ReadOpen,
                Action::ReadTopLayerContains,
                Action::ClearModal,
                Action::Complete(HtmlDialogOutcome::Completed)
            ]
        );
        assert!(dialog(&session, node.index).first.is_none());
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
    #[test]
    fn original_clone_initializes_a_new_value_then_runs_copied_open_and_removal() {
        let request = request("<dialog open>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        // The real initialized driver already owns an html child. Insert
        // beneath it, preserving the parser's one-document-element invariant.
        let html = session
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .unwrap()
            .child_at(0)
            .unwrap()
            .position();
        let parent = session.active().unwrap().node(&mut c, html).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let tag = next_tag(&mut lexer, &mut c);
        let node = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &parent,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        let original = dialog(&session, node.index).value;
        watch();
        let copy = session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, node.index, DocumentRef::Owned(0))
            .unwrap();
        assert_eq!(
            actions(),
            vec![
                Action::ReadFullyActive,
                Action::Complete(HtmlDialogOutcome::Completed)
            ]
        );
        assert_ne!(dialog(&session, copy).value, original);
        assert!(dialog(&session, copy).first.is_none());
        let copy = session.active().unwrap().node(&mut c, copy).unwrap();
        session
            .insert(&mut c, &parent, &copy, None, Some(&proof))
            .unwrap();
        watch();
        session.remove(&mut c, &copy, Some(&proof)).unwrap();
        assert_eq!(
            actions(),
            vec![
                Action::ReadOpen,
                Action::RemoveOpenDialog,
                Action::ReadWatcherPresent,
                Action::ReadTopLayerContains,
                Action::ClearModal,
                Action::Complete(HtmlDialogOutcome::Completed)
            ]
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
    #[test]
    fn cancellation_at_real_cursor_action_keeps_first_stop_and_disposes_actual_owner() {
        let request = request("<dialog open>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        // The real initialized driver already owns an html child. Insert
        // beneath it, preserving the parser's one-document-element invariant.
        let html = session
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .unwrap()
            .child_at(0)
            .unwrap()
            .position();
        let parent = session.active().unwrap().node(&mut c, html).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let tag = next_tag(&mut lexer, &mut c);
        let node = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &parent,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        CANCEL.with(|hook| {
            *hook.borrow_mut() = Some((Action::ReadFullyActive, request.cancellation().clone()))
        });
        assert!(
            session
                .insert(&mut c, &parent, &node, None, Some(&proof))
                .is_err()
        );
        let stop = c.stop();
        assert!(stop.is_some());
        assert!(
            dialog(&session, node.index).first.is_none(),
            "cancellation is a real parser stop, not unavailable data"
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), stop);
        super::super::tests::storage_zero(&c);
    }
}

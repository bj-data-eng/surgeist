//! Native Script hooks over actual staged topology, attributes and documents.
use super::*;
use surgeist_html::accounting::{HtmlChange, HtmlResource, HtmlWork};
use surgeist_html::bounded::script::{
    HtmlScriptPreparationAction as Action, HtmlScriptPreparationCursor,
    HtmlScriptPreparationReply as Reply, HtmlScriptTypeCursor,
};
use surgeist_ledger::{Change, Charge};

/// Checks cancellation before each script type cursor action and the representation of the work
/// credits they describe. DOM admits the raw strings before the hooks, so the borrowed kernel's
/// work is finite in those operands; there is no hidden allocation or copied table.
struct TypeWork<'a> {
    draft: &'a Draft,
    cancel: &'a AtomicBool,
}

impl Charge<HtmlWork, HtmlResource> for TypeWork<'_> {
    type Stop = DomError;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), DomError> {
        check_cancel(self.cancel)?;
        for change in batch {
            let Change::Work(_, amount) = *change else {
                return Err(DomError::HtmlChargeProtocol);
            };
            for _ in 0..amount {
                self.draft.source_step(self.cancel)?;
            }
        }
        Ok(())
    }

    fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), DomError> {
        Err(DomError::HtmlChargeProtocol)
    }
}

use surgeist_html::{
    elements::script::{HtmlScriptKind, HtmlScriptResult},
    vocabulary::KnownHtmlAttribute,
};

fn attribute_name_is(name: &crate::DomQualifiedName, selected: &str) -> bool {
    name.namespace_uri().is_none()
        && name
            .local_name()
            .code_units()
            .iter()
            .copied()
            .eq(selected.encode_utf16())
}

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub(super) node: Key,
    pub(super) cursor: HtmlScriptPreparationCursor,
    saved_document: Option<Key>,
    pub(super) boundary: Option<Boundary>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Boundary {
    Settings(Option<u64>),
    Executable(u64),
}

impl Draft {
    pub(super) fn pending_script(&self) -> Option<Frame> {
        match self.effects.last()?.effect {
            select::Effect::Script(frame) if frame.boundary.is_some() => Some(frame),
            _ => None,
        }
    }

    fn script_attribute(
        &self,
        node: Key,
        selected: &str,
        cancel: &AtomicBool,
    ) -> Result<Option<&DocumentString>, DomError> {
        let transaction::KindRead::Element(element) = &self.node(node)?.kind() else {
            unreachable!("actual Script family requires Element")
        };
        for attribute in element.attributes().iter() {
            self.source_step(cancel)?;
            if attribute_name_is(attribute.name(), selected) {
                return Ok(Some(attribute.value()));
            }
        }
        Ok(None)
    }

    fn script_kind(
        &self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<HtmlScriptKind>, DomError> {
        let explicit =
            self.script_attribute(node, KnownHtmlAttribute::Type.local_name(), cancel)?;
        // Obsolete language is not in the modern attribute vocabulary. The
        // canonical type cursor owns its meaning; this is only an exact lookup.
        let language = self.script_attribute(node, "language", cancel)?;
        let mut cursor = HtmlScriptTypeCursor::preparation(explicit, language);
        let mut work = TypeWork {
            draft: self,
            cancel,
        };
        while cursor.result().is_none() {
            cursor.advance(&mut work)?;
        }
        Ok(cursor.result().expect("completed actual type cursor"))
    }

    pub(super) fn script_effect(
        &mut self,
        mut frame: Frame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let node = frame.node;
        let mut state = self
            .node(node)?
            .script()
            .ok_or(DomError::InvalidSourceOperation)?;
        loop {
            self.source_step(cancel)?;
            let reply = match frame.cursor.action() {
                Action::ReadAlreadyStarted => Reply::AlreadyStarted(state.flags.already_started()),
                Action::SaveAndClearParserDocument => {
                    frame.saved_document = state.parser_document.take();
                    {
                        let mut fields = self.script_fields_mut(node, cancel)?;
                        (*fields) = Some(state);
                        fields.commit(cancel)
                    }?;
                    Reply::ParserDocumentSaved(frame.saved_document.is_some())
                }
                Action::ReadAsyncPresence => Reply::AsyncPresent(
                    self.script_attribute(node, KnownHtmlAttribute::Async.local_name(), cancel)?
                        .is_some(),
                ),
                Action::SetForceAsync => {
                    state.flags.parser_document_cleared_without_async();
                    {
                        let mut fields = self.script_fields_mut(node, cancel)?;
                        (*fields) = Some(state);
                        fields.commit(cancel)
                    }?;
                    Reply::Applied
                }
                Action::ReadChildTextEmpty => {
                    let mut empty = true;
                    for child in self.node(node)?.children().iter() {
                        self.source_step(cancel)?;
                        if let transaction::KindRead::Text(text) = &self.node(child)?.kind()
                            && !text.is_empty()
                        {
                            empty = false;
                            break;
                        }
                    }
                    Reply::ChildTextEmpty(empty)
                }
                Action::ReadSrcPresence => Reply::SrcPresent(
                    self.script_attribute(node, KnownHtmlAttribute::Src.local_name(), cancel)?
                        .is_some(),
                ),
                Action::ReadConnected => Reply::Connected(self.shadow_connected(node, cancel)?),
                Action::ClassifyType => Reply::Classified(self.script_kind(node, cancel)?),
                Action::StoreType(kind) => {
                    state.kind = Some(kind);
                    {
                        let mut fields = self.script_fields_mut(node, cancel)?;
                        (*fields) = Some(state);
                        fields.commit(cancel)
                    }?;
                    Reply::Applied
                }
                Action::RestoreParserDocument => {
                    state.parser_document = frame.saved_document.take();
                    state.flags.parser_document_restored();
                    {
                        let mut fields = self.script_fields_mut(node, cancel)?;
                        (*fields) = Some(state);
                        fields.commit(cancel)
                    }?;
                    Reply::Applied
                }
                Action::MarkAlreadyStarted => {
                    state.flags.mark_already_started();
                    {
                        let mut fields = self.script_fields_mut(node, cancel)?;
                        (*fields) = Some(state);
                        fields.commit(cancel)
                    }?;
                    Reply::Applied
                }
                Action::StorePreparationDocument => {
                    state.preparation_document = Some(self.node(node)?.document());
                    {
                        let mut fields = self.script_fields_mut(node, cancel)?;
                        (*fields) = Some(state);
                        fields.commit(cancel)
                    }?;
                    Reply::Applied
                }
                Action::CompareDocuments => {
                    Reply::DocumentsEqual(state.parser_document == state.preparation_document)
                }
                Action::RequireDisabledScripting => {
                    if !self.script_scripting_disabled(node, cancel)? {
                        if self.headroom_script_settings(cancel)? {
                            return Ok(());
                        }
                        if self.source_limits.is_some() {
                            frame.boundary = Some(Boundary::Settings(None));
                            self.push_effect(
                                select::Effect::Script(frame),
                                self.effect_scoped,
                                cancel,
                            )?;
                        }
                        return Err(DomError::ScriptPreparationRequired);
                    }
                    Reply::Disabled
                }
                Action::Complete(_) => return Ok(()),
            };
            check_cancel(cancel)?;
            frame
                .cursor
                .reply(reply)
                .map_err(|_| DomError::InvalidSourceOperation)?;
        }
    }

    fn script_scripting_disabled(&self, node: Key, cancel: &AtomicBool) -> Result<bool, DomError> {
        use crate::hosted::{DocumentPhase, ScriptRegistrationEnd};
        use surgeist_html::security::nonce::HtmlBrowsingContextPresence;
        let script = self.node(node)?;
        let document = self.document_metadata(script.document())?;
        if document.document_environment().browsing_context() == HtmlBrowsingContextPresence::Absent
        {
            return Ok(true);
        }
        let (Some(global), Some(globals)) = (script.relevant_global(), &self.hosted_globals) else {
            return Ok(false);
        };
        let origin = match globals.document_with(global, &mut || {
            self.source_step(cancel)
                .map_err(crate::DomHostedError::from)
        }) {
            Ok(document) => document,
            Err(crate::DomHostedError::StaleIdentity) => return Ok(false),
            Err(crate::DomHostedError::Dom(error)) => return Err(error),
            Err(_) => return Err(DomError::HostedSourceUnavailable),
        };
        let metadata = self.document_metadata(Key::Existing(origin.node()))?;
        let environment = metadata.document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(false);
        };
        if association.global != global || association.validate_live().is_err() {
            return Ok(false);
        }
        let (DocumentPhase::InitialEntry { navigable, .. }, Some(history)) =
            (association.phase, self.hosted_history.as_ref())
        else {
            return Ok(false);
        };
        let current = match history.navigable_with(navigable, || {
            self.source_step(cancel)
                .map_err(crate::DomHostedError::from)
        }) {
            Ok(current) => current,
            Err(crate::DomHostedError::StaleIdentity) => return Ok(false),
            Err(crate::DomHostedError::Dom(error)) => return Err(error),
            Err(_) => return Err(DomError::HostedSourceUnavailable),
        };
        if current.global != global || current.retiring || current.retired {
            return Ok(false);
        }
        if let Some(registration) = &self.script_registration
            && registration.global == global
        {
            return Ok(registration.key.document == origin
                && metadata.script_host_generation() == Some(registration.key.generation)
                && matches!(
                    registration.end.get(),
                    Some(ScriptRegistrationEnd::EngineDisposed)
                ));
        }
        Ok(true)
    }

    pub(super) fn script_post_connection(
        &mut self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(state) = self.node(node)?.script()
            && state.parser_document.is_none()
        {
            self.push_effect(
                select::Effect::Script(Frame {
                    node,
                    cursor: HtmlScriptPreparationCursor::new(),
                    saved_document: None,
                    boundary: None,
                }),
                self.effect_scoped,
                cancel,
            )?;
        }
        Ok(())
    }
    pub(super) fn script_attribute_change(
        &mut self,
        node: Key,
        name: &crate::DomQualifiedName,
        old_present: bool,
        removed: bool,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        if self.node(node)?.script().is_none() {
            return Ok(false);
        }
        check_cancel(cancel)?;
        if !removed
            && !old_present
            && attribute_name_is(name, KnownHtmlAttribute::Async.local_name())
        {
            {
                let mut fields = self.script_fields_mut(node, cancel)?;
                (*fields)
                    .as_mut()
                    .expect("actual Script")
                    .flags
                    .async_attribute_added();
                fields.commit(cancel)
            }?;
        }
        Ok(!removed
            && attribute_name_is(name, KnownHtmlAttribute::Src.local_name())
            && self
                .node(node)?
                .script()
                .is_some_and(|script| script.parser_document.is_none())
            && self.shadow_connected(node, cancel)?)
    }
    pub(super) fn script_removing_node(
        &self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if let Some(state) = self.node(node)?.script() {
            // Only the actual closed Uninitialized result is admitted. The
            // source speculation-result unregister/reset branch does not apply;
            // detach/adopt never resets actual started/preparation history.
            match state.result {
                HtmlScriptResult::Uninitialized => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_script_writes_and_real_created_document_refs_cancel_or_exhaust_together() {
        for exhausted in [false, true] {
            let mut dom = Dom::new(
                DomLimits::new(
                    32,
                    4096,
                    64,
                    0,
                    0,
                    8,
                    32768,
                    crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
            let cancel = AtomicBool::new(false);
            let mut b = DomBatch::new();
            let d = b.create_document();
            let root = b.create_element_ns(&d, surgeist_html::values::Namespace::html(), "div");
            b.insert_before(&root, &d, None);
            let c = dom.apply(b, &cancel).unwrap();
            let root = c.created(&root).unwrap().unwrap();
            let d = c.document(&d).unwrap();
            if exhausted {
                dom.revision = crate::DomRevision(u64::MAX);
            }
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut b = DomBatch::new();
            let new_document = b.create_document();
            let script = b.create_element_ns(
                &new_document,
                surgeist_html::values::Namespace::html(),
                "script",
            );
            b.set_attribute(&script, "src", "");
            b.insert_before(&script, &new_document, None);
            b.adopt(&script, d);
            b.insert_before(&script, root, None);
            let prepared = dom.prepare(b, &cancel).unwrap();
            let script_id = prepared.created[script.index].unwrap();
            let document_id = prepared.created[new_document.index].unwrap();
            let actual = prepared.state.slots[script_id.slot]
                .node
                .as_ref()
                .unwrap()
                .script()
                .unwrap();
            assert!(actual.flags.already_started());
            assert_eq!(actual.kind, Some(HtmlScriptKind::Classic));
            assert_eq!(actual.preparation_document, Some(document_id));
            assert_eq!(actual.parser_document, None);
            if !exhausted {
                cancel.store(true, Ordering::Release);
            }
            assert_eq!(
                dom.publish(prepared, &cancel).unwrap_err(),
                if exhausted {
                    DomError::RevisionExhausted
                } else {
                    DomError::Cancelled
                }
            );
            assert_eq!(*dom.state, *before.state);
            assert_eq!(dom.usage(), usage);
            assert_eq!(dom.snapshot().stamp(), before.stamp());
        }
    }
}

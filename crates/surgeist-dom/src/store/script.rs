//! Native Script hooks over actual staged topology, attributes and documents.
use super::*;
use surgeist_html::bounded::{
    HtmlScriptPreparationAction as Action, HtmlScriptPreparationCursor,
    HtmlScriptPreparationReply as Reply, HtmlScriptPreparationResult, HtmlScriptTypeCursor,
    HtmlScriptTypeWork,
};
use surgeist_html::{
    HtmlBrowsingContextPresence, HtmlScriptKind, HtmlScriptResult, KnownHtmlAttribute,
};

fn attribute_name_is(name: &crate::QualifiedName, selected: &str) -> bool {
    name.namespace().uri().is_none()
        && name
            .local_name()
            .code_units()
            .iter()
            .copied()
            .eq(selected.encode_utf16())
}

impl Draft {
    fn script_attribute(
        &self,
        node: Key,
        selected: &str,
        cancel: &AtomicBool,
    ) -> Result<Option<&DocumentString>, DomError> {
        let DomNodeKind::Element(element) = &self.node(node)?.kind else {
            unreachable!("actual Script family requires Element")
        };
        for attribute in element.attributes().iter() {
            check_cancel(cancel)?;
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
        let mut work = 0_u64;
        while let Some(required) = cursor.required_work() {
            check_cancel(cancel)?;
            // DOM admits raw strings before hooks; this borrowed kernel's work
            // is finite in those admitted operands. Check every actual credit
            // before advancing; there is no hidden allocation or copied table.
            let amount = match required {
                HtmlScriptTypeWork::RecordVisit(amount)
                | HtmlScriptTypeWork::InspectByte(amount) => amount,
            };
            work = work
                .checked_add(amount)
                .ok_or(DomError::RepresentationExhausted)?;
            cursor.advance();
        }
        Ok(cursor.result().expect("completed actual type cursor"))
    }

    fn prepare_html_script(
        &mut self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<HtmlScriptPreparationResult, DomError> {
        let mut state = self.node(node)?.script.expect("actual HTML Script");
        let mut cursor = HtmlScriptPreparationCursor::new();
        let mut saved_document = None;
        loop {
            check_cancel(cancel)?;
            let reply = match cursor.action() {
                Action::ReadAlreadyStarted => Reply::AlreadyStarted(state.flags.already_started()),
                Action::SaveAndClearParserDocument => {
                    saved_document = state.parser_document.take();
                    self.node_mut(node).script = Some(state);
                    Reply::ParserDocumentSaved(saved_document.is_some())
                }
                Action::ReadAsyncPresence => Reply::AsyncPresent(
                    self.script_attribute(node, KnownHtmlAttribute::Async.local_name(), cancel)?
                        .is_some(),
                ),
                Action::SetForceAsync => {
                    state.flags.parser_document_cleared_without_async();
                    self.node_mut(node).script = Some(state);
                    Reply::Applied
                }
                Action::ReadChildTextEmpty => {
                    let mut empty = true;
                    for child in &self.node(node)?.children {
                        check_cancel(cancel)?;
                        if let DomNodeKind::Text(text) = &self.node(*child)?.kind
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
                    self.node_mut(node).script = Some(state);
                    Reply::Applied
                }
                Action::RestoreParserDocument => {
                    state.parser_document = saved_document.take();
                    state.flags.parser_document_restored();
                    self.node_mut(node).script = Some(state);
                    Reply::Applied
                }
                Action::MarkAlreadyStarted => {
                    state.flags.mark_already_started();
                    self.node_mut(node).script = Some(state);
                    Reply::Applied
                }
                Action::StorePreparationDocument => {
                    state.preparation_document = Some(self.node(node)?.document);
                    self.node_mut(node).script = Some(state);
                    Reply::Applied
                }
                Action::CompareDocuments => {
                    Reply::DocumentsEqual(state.parser_document == state.preparation_document)
                }
                Action::RequireDisabledScripting => {
                    let document = self.node(node)?.document;
                    let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
                        unreachable!("canonical Script node document")
                    };
                    match metadata.document_environment().browsing_context() {
                        HtmlBrowsingContextPresence::Absent => Reply::Disabled,
                        HtmlBrowsingContextPresence::Present => Reply::Enabled,
                        HtmlBrowsingContextPresence::Unavailable => Reply::Unavailable,
                    }
                }
                Action::Complete(result) => return Ok(result),
            };
            check_cancel(cancel)?;
            cursor.reply(reply).map_err(|_| DomError::NotSupported)?;
        }
    }

    pub(super) fn script_post_connection(
        &mut self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(state) = self.node(node)?.script
            && state.parser_document.is_none()
        {
            self.prepare_html_script(node, cancel)?;
        }
        Ok(())
    }
    pub(super) fn script_attribute_change(
        &mut self,
        node: Key,
        name: &crate::QualifiedName,
        old_present: bool,
        removed: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.node(node)?.script.is_none() {
            return Ok(());
        }
        check_cancel(cancel)?;
        if !removed
            && !old_present
            && attribute_name_is(name, KnownHtmlAttribute::Async.local_name())
        {
            self.node_mut(node)
                .script
                .as_mut()
                .expect("actual Script")
                .flags
                .async_attribute_added();
        }
        if !removed
            && attribute_name_is(name, KnownHtmlAttribute::Src.local_name())
            && self.shadow_connected(node, cancel)?
        {
            self.script_post_connection(node, cancel)?;
        }
        Ok(())
    }
    pub(super) fn script_removing_node(
        &self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if let Some(state) = self.node(node)?.script {
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
            let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 32768).unwrap()).unwrap();
            let cancel = AtomicBool::new(false);
            let mut b = DomBatch::new();
            let d = b.create_document();
            let root = b.create_element_ns(&d, crate::Namespace::html(), "div");
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
            let script = b.create_element_ns(&new_document, crate::Namespace::html(), "script");
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
                .script
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

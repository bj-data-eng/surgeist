//! Script preparation over the active session storage.

use super::*;
use surgeist_html::construction::ConstructionNodeKind;

impl SessionStorage {
    pub(super) fn prepare_script_owned(
        &mut self,
        control: &mut ParseControl,
        index: usize,
        detached: Option<usize>,
    ) -> Result<DisabledScriptPreparation, SessionError> {
        use surgeist_html::bounded::script::{
            HtmlScriptPreparationAction as Action, HtmlScriptPreparationReply as Reply,
        };
        let (slot, mut metadata) = script_metadata(&self.payloads, control, index)?;
        payload_write::<surgeist_html::bounded::script::HtmlScriptPreparationCursor>(control, 1)?;
        let mut cursor = surgeist_html::bounded::script::HtmlScriptPreparationCursor::new();
        let mut saved_document = None;
        loop {
            let reply = match cursor.action() {
                Action::ReadAlreadyStarted => {
                    Reply::AlreadyStarted(metadata.flags.already_started())
                }
                Action::SaveAndClearParserDocument => {
                    saved_document = metadata.parser_document.take();
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::ParserDocumentSaved(saved_document.is_some())
                }
                Action::ReadAsyncPresence => {
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let ConstructionNodeKind::Element(element) = self
                        .construction
                        .node_at(index)
                        .ok_or_else(|| latch_registry_row(control))?
                        .kind()
                    else {
                        return Err(SessionError::ScriptMetadataUnavailable);
                    };
                    Reply::AsyncPresent(script_attribute(control, element, "async")?.is_some())
                }
                Action::SetForceAsync => {
                    metadata.flags.parser_document_cleared_without_async();
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::ReadChildTextEmpty => {
                    let mut empty = true;
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let children = self
                        .construction
                        .node_at(index)
                        .ok_or_else(|| latch_registry_row(control))?
                        .child_count();
                    for position in 0..children {
                        control.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
                        let child = self
                            .construction
                            .node_at(index)
                            .ok_or_else(|| latch_registry_row(control))?
                            .child_at(position)
                            .ok_or_else(|| latch_registry_row(control))?
                            .position();
                        let record = self
                            .construction
                            .node_at(child)
                            .ok_or(SessionError::ForeignNode)?;
                        if Some(child) != detached
                            && let ConstructionNodeKind::Text(text) = record.kind()
                            && !text.is_empty()
                        {
                            empty = false;
                            break;
                        }
                    }
                    // Preserve the original element read after source-text
                    // inspection, including when nonempty text skips src lookup.
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if !matches!(
                        self.construction
                            .node_at(index)
                            .ok_or_else(|| latch_registry_row(control))?
                            .kind(),
                        ConstructionNodeKind::Element(_)
                    ) {
                        return Err(SessionError::ScriptMetadataUnavailable);
                    }
                    Reply::ChildTextEmpty(empty)
                }
                Action::ReadSrcPresence => {
                    let ConstructionNodeKind::Element(element) = self
                        .construction
                        .node_at(index)
                        .ok_or_else(|| latch_registry_row(control))?
                        .kind()
                    else {
                        return Err(SessionError::ScriptMetadataUnavailable);
                    };
                    Reply::SrcPresent(script_attribute(control, element, "src")?.is_some())
                }
                Action::ReadConnected => {
                    // Connectivity follows the parser-document and empty-source
                    // writes. The actual detached construction view stays here.
                    let mut ancestor = Some(index);
                    let mut connected = false;
                    for _ in 0..self.construction.len() {
                        let Some(current) = ancestor else { break };
                        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        let record = self
                            .construction
                            .node_at(current)
                            .ok_or(SessionError::ForeignNode)?;
                        if matches!(record.kind(), ConstructionNodeKind::Document(_)) {
                            connected = true;
                            ancestor = None;
                            break;
                        }
                        ancestor = if Some(current) == detached {
                            None
                        } else {
                            record.parent().map(|node| node.position())
                        };
                    }
                    if ancestor.is_some() {
                        return Err(SessionError::Cycle);
                    }
                    Reply::Connected(connected)
                }
                Action::ClassifyType => {
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let ConstructionNodeKind::Element(element) = self
                        .construction
                        .node_at(index)
                        .ok_or_else(|| latch_registry_row(control))?
                        .kind()
                    else {
                        return Err(SessionError::ScriptMetadataUnavailable);
                    };
                    Reply::Classified(classify_script(control, element)?)
                }
                Action::StoreType(kind) => {
                    metadata.script_type = Some(kind);
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::RestoreParserDocument => {
                    metadata.parser_document = saved_document.take();
                    metadata.flags.parser_document_restored();
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::MarkAlreadyStarted => {
                    metadata.flags.mark_already_started();
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::StorePreparationDocument => {
                    metadata.preparation_document = Some(self.node_document_ref(control, index)?);
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::CompareDocuments => {
                    Reply::DocumentsEqual(metadata.parser_document == metadata.preparation_document)
                }
                Action::RequireDisabledScripting => {
                    // The actual ordinary/qualified-fragment construction
                    // capability admits only this closed Disabled execution
                    // domain. This is not inferred from flags or XML class.
                    Reply::Disabled
                }
                Action::Complete(result) => return Ok(result),
            };
            // Account the actual new owned state before each transition. The
            // metadata write boundaries and first reached failure remain intact.
            payload_write::<surgeist_html::bounded::script::HtmlScriptPreparationCursor>(
                control, 1,
            )?;
            cursor.reply(reply).map_err(|_| SessionError::WrongToken)?;
        }
    }
}

//! Disabled script allocation, standalone proofs and script preparation in the parser session.

use super::*;
use surgeist_html::{
    construction::ConstructionNodeKind,
    elements::script::{HtmlScriptFlags, HtmlScriptResult},
};

impl ParserArenaSession {
    pub(crate) fn validate_standalone_document(
        &self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if let Some(reason) = storage.standalone_ineligibility {
            return Err(control.stop_unsupported_standalone(reason).into());
        }
        if !matches!(
            storage
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            ConstructionNodeKind::Document(_)
        ) {
            return Err(control
                .stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance)
                .into());
        }
        Ok(())
    }
    /// Borrowed sealed-proof qualification, including legitimate checkpoint
    /// reentry. Useful external producers independently enforce readiness.
    pub(crate) fn validate_standalone_proof(
        &self,
        c: &mut ParseControl,
        document: &ParserNode,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        c.require_identity(&storage.control)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 4)?;
        if let Some(reason) = storage.standalone_ineligibility {
            return Err(c.stop_unsupported_standalone(reason).into());
        }
        if storage
            .construction
            .storage(c)?
            .position(&document.handle)
            .ok()
            != Some(0)
            || !matches!(
                storage
                    .construction
                    .node_at(0)
                    .ok_or_else(|| latch_registry_row(c))?
                    .kind(),
                ConstructionNodeKind::Document(_)
            )
        {
            return Err(c
                .stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance)
                .into());
        }
        Ok(())
    }
    /// The sealed execution owner, not a generic Document constructor, proves
    /// the selected ordinary Disabled environment before any payload transfer.
    pub(crate) fn allocate_disabled_parser_script(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        intended_parent: Option<&ParserNode>,
        context: CreationContext<'_>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<ParserNode, SessionError> {
        capability.validate_session(control, self)?;
        let tag = input.as_ref().ok_or(SessionError::Consumed)?;
        check_disposal_text(control, &tag.name)?;
        tag.attributes.require_disposal_owner(control)?;
        check_text_source(control, &tag.name)?;
        for attribute in tag.attributes.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            check_disposal_text(control, &attribute.name)?;
            check_disposal_text(control, &attribute.value)?;
            control.require_source_range(&attribute.source)?;
            check_text_source(control, &attribute.name)?;
            check_text_source(control, &attribute.value)?;
        }
        if !exact_ascii(control, Some(tag.name.value.text()), "script")? {
            return Err(SessionError::WrongToken);
        }
        self.qualify_creation_context(control, &context)?;
        let parent = match intended_parent {
            Some(parent) => {
                let index = self.active()?.index(control, parent)?;
                self.active()?.node(control, index)?
            }
            None => self.root(control)?,
        };
        self.active()?.node_document_ref(control, parent.index)?;
        let storage = self.active_mut()?;
        storage.document_slot(control, 0)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let inert = storage.fragment.as_ref().is_some_and(|fragment| {
            fragment.mode == surgeist_html::parsing::HtmlParserScriptingMode::Inert
        });
        let prepared = storage.prepare_payload_slot(control)?;
        payload_write::<Option<usize>>(control, 1)?;
        let script = storage
            .payloads
            .push_row(
                prepared,
                RetainedPayload::Script {
                    node: None,
                    metadata: ScriptMetadata {
                        parser_document: Some(DocumentRef::Owned(0)),
                        preparation_document: None,
                        flags: {
                            let mut flags = HtmlScriptFlags::fresh();
                            flags.mark_parser_inserted();
                            if inert {
                                flags.mark_already_started();
                            }
                            flags
                        },
                        script_type: None,
                        result: HtmlScriptResult::Uninitialized,
                        steps_when_result_ready: None,
                        start: storage.token_end,
                    },
                },
            )
            .map_err(|_| latch_registry_row(control))?;
        let node = self.allocate_recorded_creation(
            control,
            input,
            &parent,
            None,
            CreationProfile::Ordinary,
            context,
        )?;
        let Some(RetainedPayload::Script { node: bound, .. }) =
            self.active_mut()?.payloads.values.get_mut(script)
        else {
            return Err(latch_registry_row(control));
        };
        *bound = Some(node.index);
        self.active_mut()?
            .payloads
            .join_row(script)
            .map_err(|_| latch_registry_row(control))?;
        Ok(node)
    }
    #[cfg(test)]
    pub(crate) fn script_metadata(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<ScriptMetadata, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        script_metadata(&storage.payloads, control, node).map(|(_, metadata)| metadata)
    }
    #[cfg(test)]
    pub(crate) fn script_parser_document(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.script_metadata(control, node)?
            .parser_document
            .map(|index| self.active()?.node(control, index.owned()?))
            .transpose()
    }
    #[cfg(test)]
    pub(crate) fn script_preparation_document(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.script_metadata(control, node)?
            .preparation_document
            .map(|index| self.active()?.node(control, index.owned()?))
            .transpose()
    }
    /// Text-mode EOF marks the current script started; it does not prepare it.
    pub(crate) fn mark_script_eof(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        capability.validate_session(control, self)?;
        let storage = self.active_mut()?;
        let index = storage.index(control, node)?;
        let (slot, mut metadata) = script_metadata(&storage.payloads, control, index)?;
        metadata.flags.mark_already_started();
        set_script_metadata(storage, control, slot, &metadata)
    }
    /// Exact preparation prefix through HTML's scripting-disabled step18.
    /// No loader, compilation, result or execution callback is invoked.
    pub(crate) fn prepare_disabled_script(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<DisabledScriptPreparation, SessionError> {
        capability.validate_session(control, self)?;
        let storage = self.active_mut()?;
        let index = storage.index(control, node)?;
        storage.prepare_script_owned(control, index, None)
    }
}

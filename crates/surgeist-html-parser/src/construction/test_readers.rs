//! Test-only readers of retained input, output, textarea and form state.

use super::*;
use surgeist_html::{
    construction::HtmlNodeId, elements::form::HtmlNativeFormAssociationKind, values::DocumentString,
};

impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn input_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<InputState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        Ok(find_input_state(&storage.payloads, control, node)?.1)
    }
    /// Returns original token/value-attribute ranges only when creation actually
    /// rewrote the native value. A completed no-writeback input returns None;
    /// unavailable/imported/incomplete metadata returns its typed error.
    #[cfg(test)]
    pub(crate) fn input_writeback_sources<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<(&'a crate::HtmlSourceRange, &'a crate::HtmlSourceRange)>, SessionError>
    {
        let storage = self.active()?;
        let index = storage.index(control, node)?;
        find_input_sources(&storage.payloads, control, index)
    }
    #[cfg(test)]
    pub(crate) fn append_input_internal_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        let raw = find_input_state(&storage.payloads, control, node)?.0;
        destination.extend_units(
            textarea_raw(&storage.payloads, control, raw)?.code_units(),
            control,
        )?;
        Ok(())
    }
    #[cfg(test)]
    /// Native input value readback, distinct from internal and default value.
    pub(crate) fn append_input_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        let state = self.input_state(control, node)?;
        let storage = self.active()?;
        append_native_input_value(
            ConstructionView::Active(&storage.construction),
            &storage.payloads,
            control,
            node.index,
            state,
            destination,
        )
    }
    #[cfg(test)]
    pub(crate) fn append_input_default_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.input_state(control, node)?;
        append_input_content(
            ConstructionView::Active(&self.active()?.construction),
            control,
            node.index,
            destination,
            &[],
        )
    }

    #[cfg(test)]
    pub(crate) fn image_form_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<FormAssociationState, SessionError> {
        let state = self
            .form_association_state(control, node)
            .map_err(|error| match error {
                SessionError::FormAssociationMetadataUnavailable => {
                    SessionError::ImageFormMetadataUnavailable
                }
                other => other,
            })?;
        if state.kind != HtmlNativeFormAssociationKind::Image {
            return Err(SessionError::ImageFormMetadataUnavailable);
        }
        Ok(state)
    }
    #[cfg(test)]
    pub(crate) fn output_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<OutputState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        find_output_state(&storage.payloads, control, node)
    }
    #[cfg(test)]
    pub(crate) fn append_output_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.output_state(control, node)?;
        append_descendant_text(
            ConstructionView::Active(&self.active()?.construction),
            control,
            node.index,
            destination,
        )
    }
    #[cfg(test)]
    pub(crate) fn append_output_default_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        // Actual retained Null override delegates to descendant text. There is
        // no nonnull override/setter producer or guessed stored default.
        self.append_output_value(control, node, destination)
    }
    #[cfg(test)]
    pub(crate) fn textarea_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<TextareaState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        storage
            .textarea_slot(control, node)?
            .map(|(_, state)| state)
            .ok_or(SessionError::TextareaMetadataUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn textarea_raw_value<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<&'a DocumentString, SessionError> {
        let state = self.textarea_state(control, node)?;
        textarea_raw(&self.active()?.payloads, control, state.raw)
    }
    #[cfg(test)]
    pub(crate) fn image_form_owner(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.image_form_state(control, node)?
            .owner
            .map(|owner| self.active()?.node(control, owner))
            .transpose()
    }
    #[cfg(test)]
    pub(crate) fn form_association_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<FormAssociationState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        find_form_association_state(&storage.payloads, control, node)?
            .map(|(_, state)| state)
            .ok_or(SessionError::FormAssociationMetadataUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn form_association_owner(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.form_association_state(control, node)?
            .owner
            .map(|owner| self.active()?.node(control, owner))
            .transpose()
    }
}

impl FrozenArena {
    #[cfg(test)]
    pub(crate) fn input_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<InputState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = &storage.view;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        Ok(find_input_state(&storage.payloads, control, position)?.1)
    }
    /// Same conditional availability as the active owner, qualified to this arena.
    #[cfg(test)]
    pub(crate) fn input_writeback_sources<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<(&'a crate::HtmlSourceRange, &'a crate::HtmlSourceRange)>, SessionError>
    {
        self.input_state(control, node)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let arena = &self.storage.as_ref().ok_or(SessionError::Consumed)?.view;
        find_input_sources(
            &self
                .storage
                .as_ref()
                .ok_or(SessionError::Consumed)?
                .payloads,
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
        )
    }
    #[cfg(test)]
    pub(crate) fn append_input_internal_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.input_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let arena = &storage.view;
        let raw = find_input_state(
            &storage.payloads,
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
        )?
        .0;
        destination.extend_units(
            textarea_raw(&storage.payloads, control, raw)?.code_units(),
            control,
        )?;
        Ok(())
    }
    #[cfg(test)]
    /// Native input value readback, distinct from internal and default value.
    pub(crate) fn append_input_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        let state = self.input_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = &storage.view;
        append_native_input_value(
            ConstructionView::Frozen(arena),
            &storage.payloads,
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
            state,
            destination,
        )
    }
    #[cfg(test)]
    pub(crate) fn append_input_default_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.input_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = &storage.view;
        append_input_content(
            ConstructionView::Frozen(arena),
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
            destination,
            &[],
        )
    }

    #[cfg(test)]
    pub(crate) fn output_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<OutputState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = &storage.view;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        find_output_state(&storage.payloads, control, position)
    }
    #[cfg(test)]
    pub(crate) fn append_output_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.output_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = &storage.view;
        append_descendant_text(
            ConstructionView::Frozen(arena),
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
            destination,
        )
    }
    #[cfg(test)]
    pub(crate) fn append_output_default_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        self.append_output_value(control, node, destination)
    }
    #[cfg(test)]
    pub(crate) fn textarea_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<TextareaState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = &storage.view;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        find_textarea_state(&storage.payloads, control, position)?
            .map(|(_, state)| state)
            .ok_or(SessionError::TextareaMetadataUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn textarea_raw_value<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<&'a DocumentString, SessionError> {
        let state = self.textarea_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        textarea_raw(&storage.payloads, control, state.raw)
    }
    #[cfg(test)]
    pub(crate) fn image_form_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<FormAssociationState, SessionError> {
        let state = self
            .form_association_state(control, node)
            .map_err(|error| match error {
                SessionError::FormAssociationMetadataUnavailable => {
                    SessionError::ImageFormMetadataUnavailable
                }
                other => other,
            })?;
        if state.kind != HtmlNativeFormAssociationKind::Image {
            return Err(SessionError::ImageFormMetadataUnavailable);
        }
        Ok(state)
    }
    #[cfg(test)]
    pub(crate) fn image_form_owner(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let state = self.image_form_state(control, node)?;
        self.resolve_form_owner(control, state.owner)
    }
    #[cfg(test)]
    pub(crate) fn form_association_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<FormAssociationState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = &storage.view;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        find_form_association_state(&storage.payloads, control, position)?
            .map(|(_, state)| state)
            .ok_or(SessionError::FormAssociationMetadataUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn form_association_owner(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let state = self.form_association_state(control, node)?;
        self.resolve_form_owner(control, state.owner)
    }
    #[cfg(test)]
    pub(super) fn resolve_form_owner(
        &self,
        control: &mut ParseControl,
        owner: Option<usize>,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let Some(owner) = owner else {
            return Ok(None);
        };
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = &storage.view;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if owner >= arena.len() {
            return Err(SessionError::ForeignNode);
        }
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<HtmlNodeId>() as u64,
        )?;
        Ok(Some(
            arena
                .node_at(owner)
                .expect("qualified form owner")
                .as_node()
                .id(),
        ))
    }
    #[cfg(test)]
    pub(crate) fn script_metadata(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<ScriptMetadata, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let position = storage
            .view
            .position(&node.handle)
            .map_err(|_| SessionError::ForeignNode)?;
        script_metadata(&storage.payloads, control, position).map(|(_, metadata)| metadata)
    }
}

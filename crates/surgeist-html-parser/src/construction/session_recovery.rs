//! Recovered tags, prologue nodes and document mode in the parser session.

use super::*;
use surgeist_html::construction::{ConstructionNodeKind, HtmlDocumentMode};

impl ParserArenaSession {
    /// Pinned InBody `image` recovery rewrites the actual token name only.
    /// The exact original name range is retained as RecoverySyntax; all other
    /// lexical fields and their owners remain in their original slots.
    pub(crate) fn recover_image_tag(
        &mut self,
        control: &mut ParseControl,
        source: &crate::HtmlSource,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        self.active()?.require(control)?;
        control.require_source(source)?;
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if tag.kind != TagKind::Start {
            return Err(SessionError::WrongToken);
        }
        SessionStorage::check_text(control, &mut tag.name)?;
        if !exact_ascii(control, Some(tag.name.value.text()), "image")? {
            return Err(SessionError::WrongToken);
        }
        tag.attributes.require_owner(control)?;
        for index in 0..tag.attributes.len() {
            let Some(attribute) = tag.attributes.get_mut(index, control)? else {
                return Err(latch_registry_row(control));
            };
            SessionStorage::check_text(control, &mut attribute.name)?;
            SessionStorage::check_text(control, &mut attribute.value)?;
            check_text_source(control, &attribute.name)?;
            check_text_source(control, &attribute.value)?;
            control.require_source_range(&attribute.source)?;
        }
        if !control.detail().records_graph() {
            // No origin names the replaced spelling, so the replacement has none.
            let value = literal_string(control, b"img")?;
            let origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
            payload_write::<Text>(control, 1)?;
            let original = mem::replace(&mut tag.name, Text { value, origins });
            original.release(control)?;
            return Ok(());
        }
        let mut covered = 0;
        let mut start = None;
        let mut end = None;
        for origin in tag.name.origins.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.require_source_range(&origin.source)?;
            if origin.start != covered
                || origin.end <= origin.start
                || origin.end > tag.name.value.text().len()
                || end.is_some_and(|end| end != origin.source.start())
            {
                return Err(SessionError::OriginSplitUnavailable);
            }
            start.get_or_insert(origin.source.start());
            end = Some(origin.source.end());
            covered = origin.end;
        }
        if covered != tag.name.value.text().len() {
            return Err(SessionError::OriginSplitUnavailable);
        }
        let start = start.ok_or(SessionError::OriginSplitUnavailable)?;
        let end = end.ok_or(SessionError::OriginSplitUnavailable)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        payload_write::<crate::HtmlSourceRange>(control, 1)?;
        // HtmlSource::range reads each interior UTF8 endpoint once. Pre-admit
        // both reads together immediately before that bounded native helper.
        if source.as_utf8().is_some() {
            let reads = [start, end]
                .into_iter()
                .filter(|&endpoint| endpoint != 0 && endpoint != source.len())
                .count() as u64;
            control.charge(HtmlParseWorkCategory::InputUnitRead, reads)?;
        }
        let range = source
            .range(start, end)
            .map_err(|_| SessionError::OriginSplitUnavailable)?;
        let value = literal_string(control, b"img")?;
        let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let prepared = (|| {
            origins.try_push(
                control,
                HtmlParseTextOrigin {
                    start: 0,
                    end: 3,
                    source: range,
                    kind: crate::parser::tokenizer::token::HtmlParseOriginKind::RecoverySyntax,
                },
            )?;
            payload_write::<Text>(control, 1)
        })();
        if let Err(error) = prepared {
            value.release(control)?;
            origins.release(control)?;
            return Err(error);
        }
        let original = mem::replace(&mut tag.name, Text { value, origins });
        original.release(control)?;
        Ok(())
    }
    /// Pinned end-br recovery discards actual attributes before start-tag
    /// reprocessing. Name text/origins stay lexical: their units did not change;
    /// the driver retains the original end-token range and recovery diagnostic.
    pub(crate) fn recover_br_end_tag(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        self.active()?.require(control)?;
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if tag.kind != TagKind::End {
            return Err(SessionError::WrongToken);
        }
        SessionStorage::check_text(control, &mut tag.name)?;
        if !exact_ascii(control, Some(tag.name.value.text()), "br")? {
            return Err(SessionError::WrongToken);
        }
        check_text_source(control, &tag.name)?;
        tag.attributes.require_owner(control)?;
        for index in 0..tag.attributes.len() {
            let Some(attribute) = tag.attributes.get_mut(index, control)? else {
                return Err(latch_registry_row(control));
            };
            SessionStorage::check_text(control, &mut attribute.name)?;
            SessionStorage::check_text(control, &mut attribute.value)?;
            check_text_source(control, &attribute.name)?;
            check_text_source(control, &attribute.value)?;
            control.require_source_range(&attribute.source)?;
        }
        let empty = OwnedVec::new(HtmlParseResource::TokenAttributes)?;
        payload_write::<OwnedVec<crate::parser::tokenizer::token::Attribute>>(control, 1)?;
        payload_write::<TagKind>(control, 1)?;
        let original = mem::replace(&mut tag.attributes, empty);
        // This disposal finishes all nested reclamation even on a newly latched
        // stop. The retained Tag then owns its unchanged name and empty attrs;
        // kind remains End unless the complete disposal succeeded.
        release_token_attributes(original, control)?;
        tag.kind = TagKind::Start;
        Ok(())
    }
    pub(super) fn allocate_implied_html(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
        local: ImpliedHtmlName,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        let document = storage.creation_document(control, intended_parent)?;
        let ordinary_document = storage.fragment.is_none() && document.is_some();
        let mut prepared = storage.prepare_document_allocation(control, document, false)?;
        let attributes = OwnedVec::new(HtmlParseResource::TokenAttributes)?;
        let name = literal_string(
            control,
            match local {
                ImpliedHtmlName::Html => b"html",
                ImpliedHtmlName::Head => b"head",
                ImpliedHtmlName::Body => b"body",
                ImpliedHtmlName::P => b"p",
                ImpliedHtmlName::Colgroup => b"colgroup",
                ImpliedHtmlName::Tbody => b"tbody",
                ImpliedHtmlName::Tr => b"tr",
            },
        )?;
        // Fragment bootstrap establishes its captured root registry separately;
        // an ordinary implied root instead belongs to the actual root Document.
        let parent = intended_parent
            .map(|parent| parent.index)
            .or_else(|| ordinary_document.then_some(0));
        if let Some(parent) = parent
            && let Err(error) = storage.prepare_creation_registry(
                control,
                parent,
                name.text(),
                None,
                true,
                &mut prepared,
            )
        {
            name.release(control)?;
            attributes.release(control)?;
            return Err(error);
        }
        storage.allocate_html_payload_prepared(control, name, None, attributes, prepared)
    }
    pub(crate) fn allocate_comment(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Text>,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        SessionStorage::check_text(control, input.as_mut().ok_or(SessionError::Consumed)?)?;
        let document = storage.creation_document(control, intended_parent)?;
        let prepared = storage.prepare_document_allocation(control, document, false)?;
        let text = take_payload(input, control)?;
        let field = storage.text_field(control, text)?;
        let created = (|| {
            control.charge(HtmlParseWorkCategory::RecordWrite, 2)?;
            storage.allocate_prepared_owned(
                control,
                NewRecord::Comment(field.units()),
                false,
                0,
                field.units().len(),
                prepared,
            )
        })();
        let FieldString { value, group } = field;
        let released = value.release(control);
        let node = created?;
        released?;
        storage.bind_field(control, FieldRef::Comment(node.index), group, 0)?;
        Ok(node)
    }
    pub(crate) fn allocate_processing_instruction(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<(Text, Text)>,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        let (target, data) = input.as_mut().ok_or(SessionError::Consumed)?;
        SessionStorage::check_text(control, target)?;
        SessionStorage::check_text(control, data)?;
        let document = storage.creation_document(control, intended_parent)?;
        let prepared = storage.prepare_document_allocation(control, document, false)?;
        let map = storage.prepare_pi_map(control)?;
        payload_write::<Option<usize>>(control, 1)?;
        let (target, data) = take_payload(input, control)?;
        let target = match storage.text_field(control, target) {
            Ok(value) => value,
            Err(error) => {
                data.release(control)?;
                return Err(error);
            }
        };
        let data = match storage.text_field(control, data) {
            Ok(value) => value,
            Err(error) => {
                target.value.release(control)?;
                return Err(error);
            }
        };
        // Recovered PI data is exact storage, not checked native authoring input.
        let created = (|| {
            control.charge(HtmlParseWorkCategory::RecordWrite, 3)?;
            storage.allocate_prepared_owned(
                control,
                NewRecord::ProcessingInstruction(target.units(), data.units()),
                false,
                0,
                target.units().len().saturating_add(data.units().len()),
                prepared,
            )
        })();
        let FieldString {
            value: target,
            group: target_group,
        } = target;
        let FieldString {
            value: data,
            group: data_group,
        } = data;
        let released = target.release(control);
        let released_data = data.release(control);
        let node = created?;
        released?;
        released_data?;
        storage.bind_field(
            control,
            FieldRef::ProcessingInstructionTarget(node.index),
            target_group,
            0,
        )?;
        storage.bind_field(
            control,
            FieldRef::ProcessingInstructionData(node.index),
            data_group,
            0,
        )?;
        storage.bind_pi_map(control, map, node.index)?;
        storage.parse_pi_map(control, node.index, map)?;
        Ok(node)
    }
    pub(crate) fn allocate_doctype(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Doctype>,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        let doctype = input.as_mut().ok_or(SessionError::Consumed)?;
        for text in [
            &mut doctype.name,
            &mut doctype.public_id,
            &mut doctype.system_id,
        ]
        .into_iter()
        .flatten()
        {
            SessionStorage::check_text(control, text)?;
        }
        let Doctype {
            name,
            public_id,
            system_id,
            ..
        } = take_payload(input, control)?;
        let mut incoming = [name, public_id, system_id];
        let mut registered: [Option<FieldString>; 3] = [None, None, None];
        let result = (|| {
            let mut units = 0usize;
            for (field, slot) in incoming.iter_mut().zip(registered.iter_mut()) {
                payload_write::<Option<Text>>(control, 2)?;
                let value = if let Some(text) = field.take() {
                    storage.text_field(control, text)?
                } else {
                    let empty = literal_string(control, b"")?;
                    storage.field_string(control, empty, None)?
                };
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                units = units.checked_add(value.units().len()).ok_or_else(|| {
                    control.stop_representation(HtmlParseResource::ManagedUtf16Units)
                })?;
                payload_write::<Option<FieldString>>(control, 2)?;
                *slot = Some(value);
            }
            payload_write::<[Option<FieldString>; 3]>(control, 3)?;
            let [Some(name), Some(public_id), Some(system_id)] = &mut registered else {
                return Err(latch_registry_row(control));
            };
            control.charge(HtmlParseWorkCategory::RecordWrite, 4)?;
            // HTML's DOM creation step lowers missing token fields to empty
            // strings. Native authored Option data keeps its separate meaning.
            let node = storage.allocate(
                control,
                NewRecord::Doctype(name.units(), public_id.units(), system_id.units()),
                false,
                0,
                units,
            )?;
            storage.bind_field(
                control,
                FieldRef::DoctypeName(node.index),
                name.group.take(),
                0,
            )?;
            storage.bind_field(
                control,
                FieldRef::DoctypePublic(node.index),
                public_id.group.take(),
                0,
            )?;
            storage.bind_field(
                control,
                FieldRef::DoctypeSystem(node.index),
                system_id.group.take(),
                0,
            )?;
            Ok(node)
        })();
        let mut released = Ok(());
        for field in registered.into_iter().flatten() {
            let freed = field.value.release(control);
            if released.is_ok() {
                released = freed;
            }
        }
        for text in incoming.into_iter().flatten() {
            text.release(control)?;
        }
        let node = result?;
        released?;
        Ok(node)
    }
    pub(crate) fn document_accepts_doctype(
        &self,
        control: &mut ParseControl,
    ) -> Result<bool, SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if !matches!(
            storage
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            ConstructionNodeKind::Document(_)
        ) {
            return Err(SessionError::NotDocument);
        }
        let mut next = storage
            .construction
            .node_at(0)
            .ok_or_else(|| latch_registry_row(control))?
            .first_child()
            .map(|child| child.position());
        while let Some(child) = next {
            control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            next = storage
                .construction
                .node_at(child)
                .ok_or_else(|| latch_registry_row(control))?
                .next_sibling()
                .map(|sibling| sibling.position());
            if matches!(
                storage
                    .construction
                    .node_at(child)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                ConstructionNodeKind::Doctype(_) | ConstructionNodeKind::Element(_)
            ) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub(crate) fn document_mode(
        &self,
        control: &mut ParseControl,
    ) -> Result<HtmlDocumentMode, SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Document(mode) = &storage
            .construction
            .node_at(0)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(SessionError::NotDocument);
        };
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<HtmlDocumentMode>() as u64,
        )?;
        Ok(*mode)
    }
    pub(crate) fn set_document_mode(
        &mut self,
        control: &mut ParseControl,
        mode: HtmlDocumentMode,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if !matches!(
            storage
                .construction
                .node_at(0)
                .ok_or(SessionError::ForeignNode)?
                .kind(),
            ConstructionNodeKind::Document(_)
        ) {
            return Err(SessionError::NotDocument);
        }
        payload_write::<HtmlDocumentMode>(control, 1)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let Some(document) = storage.construction.node_at(0).map(|root| root.handle()) else {
            return Err(latch_registry_row(control));
        };
        storage.set_document_mode_record(control, &document, mode)?;
        Ok(())
    }
    pub(crate) fn insert_text(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        before: Option<&ParserNode>,
        input: &mut Option<Text>,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<Option<TextInsertion>, SessionError> {
        let storage = self.active()?;
        let index = storage.index(control, parent)?;
        if storage.html_local(control, index, "script")?
            && script_metadata(&storage.payloads, control, index)?
                .1
                .parser_document
                .is_none()
        {
            let capability = capability.ok_or_else(|| {
                SessionError::Stop(
                    control.stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance),
                )
            })?;
            capability.validate_session(control, self)?;
        }
        self.active_mut()?
            .insert_text(control, parent, before, input)
    }
}

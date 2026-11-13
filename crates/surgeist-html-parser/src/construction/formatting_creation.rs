//! Original lexical creation records retained for active-formatting replay.

use super::*;
use surgeist_html::construction::HtmlBuilderNode;

/// Original lexical creation record, not an alias of installed arena backing.
/// Only this arena can issue it; every replay copies into independent tickets.
pub(crate) struct FormattingCreation {
    pub(super) token: Option<Tag>,
    source: crate::HtmlSourceRange,
    control: ControlIdentity,
    arena: HtmlBuilderNode,
    cause: Option<provenance::ConstructionCauseRef>,
}

impl FormattingCreation {
    /// Borrowed disposal retains originals on any preflight rejection. Once a
    /// destructive disposal begins, newly observed stops complete reclamation.
    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        cleanup_owner(control, &self.control)?;
        release_formatting_tag(&mut self.token, control)
    }
}

pub(super) fn check_formatting_text(
    control: &mut ParseControl,
    text: &Text,
    source: &crate::HtmlSourceRange,
) -> Result<(), SessionError> {
    control.require_document_string(text.value.allocation(), text.value.text())?;
    text.origins.require_disposal_owner(control)?;
    control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
    if text.value.text().block().holders() != 1 {
        return Err(SessionError::AliasedPayload);
    }
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    if !control.detail().records_graph() {
        return Ok(());
    }
    let len = text.value.text().len();
    let mut covered = 0;
    for origin in text.origins.as_slice() {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        control.require_source_range(&origin.source)?;
        if origin.start != covered
            || origin.end <= origin.start
            || origin.end > len
            || origin.source.start() < source.start()
            || origin.source.end() > source.end()
        {
            return Err(SessionError::OriginSplitUnavailable);
        }
        covered = origin.end;
    }
    if covered != len {
        return Err(SessionError::OriginSplitUnavailable);
    }
    Ok(())
}

pub(super) fn check_formatting_tag(
    control: &mut ParseControl,
    tag: &Tag,
    source: &crate::HtmlSourceRange,
) -> Result<(), SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    if tag.kind != TagKind::Start {
        return Err(SessionError::WrongToken);
    }
    check_formatting_text(control, &tag.name, source)?;
    lowercase_token_name(control, tag.name.value.text())?;
    tag.attributes.require_disposal_owner(control)?;
    // Pair matching for Noah relies on actual unique parsed names, not the token's
    // duplicate-recovery flag or an untrusted counter substitute.
    token_names::require_distinct_names(
        control,
        tag.attributes.as_slice(),
        |control, _, attribute| {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            check_formatting_text(control, &attribute.name, source)?;
            check_formatting_text(control, &attribute.value, source)?;
            lowercase_token_name(control, attribute.name.value.text())?;
            control.require_source_range(&attribute.source)?;
            if attribute.source.start() < source.start() || attribute.source.end() > source.end() {
                return Err(SessionError::OriginSplitUnavailable);
            }
            Ok(())
        },
    )?;
    Ok(())
}

pub(super) fn copy_formatting_text(
    control: &mut ParseControl,
    text: &Text,
) -> Result<Text, SessionError> {
    let mut value = Utf16Buffer::new(control)?;
    let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
    let copied = (|| {
        // The concrete buffer copies/polls bounded real slices; no scalar or
        // lossy conversion, lexical scan or implicit Arc alias is introduced.
        value.extend_units(text.value.text().code_units(), control)?;
        for origin in text.origins.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            payload_write::<HtmlParseTextOrigin>(control, 1)?;
            origins.try_push_geometric(
                control,
                HtmlParseTextOrigin {
                    start: origin.start,
                    end: origin.end,
                    source: origin.source.clone(),
                    kind: origin.kind,
                },
            )?;
        }
        payload_write::<Text>(control, 1)
    })();
    if let Err(error) = copied {
        value.release(control)?;
        origins.release(control)?;
        return Err(error);
    }
    match value.freeze(control) {
        Ok(value) => Ok(Text { value, origins }),
        Err(stop) => {
            origins.release(control)?;
            Err(stop.into())
        }
    }
}

pub(super) fn copy_formatting_tag(
    control: &mut ParseControl,
    tag: &Tag,
) -> Result<Tag, SessionError> {
    let name = copy_formatting_text(control, &tag.name)?;
    let mut attributes = OwnedVec::new(HtmlParseResource::TokenAttributes)?;
    let copied = (|| {
        for original in tag.attributes.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let name = copy_formatting_text(control, &original.name)?;
            let value = match copy_formatting_text(control, &original.value) {
                Ok(value) => value,
                Err(error) => {
                    name.release(control)?;
                    return Err(error);
                }
            };
            let admission = (|| {
                control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                payload_write::<crate::parser::tokenizer::token::Attribute>(control, 1)
            })();
            if let Err(error) = admission {
                name.release(control)?;
                value.release(control)?;
                return Err(error);
            }
            let attribute = crate::parser::tokenizer::token::Attribute {
                // A copied formatting payload has existing original field
                // lineage; it is not a second consumed lexical occurrence.
                lexical: None,
                name,
                value,
                source: original.source.clone(),
            };
            if let Err((stop, attribute)) =
                attributes.try_push_recover_geometric(control, attribute)
            {
                attribute.name.release(control)?;
                attribute.value.release(control)?;
                return Err(stop.into());
            }
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        payload_write::<Tag>(control, 1)
    })();
    if let Err(error) = copied {
        name.release(control)?;
        release_token_attributes(attributes, control)?;
        return Err(error);
    }
    Ok(Tag {
        kind: TagKind::Start,
        name,
        self_closing: tag.self_closing,
        attributes,
        had_duplicate_attributes: tag.had_duplicate_attributes,
    })
}

pub(super) fn release_formatting_tag(
    input: &mut Option<Tag>,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
    let Some(tag) = input.as_ref() else {
        return Ok(());
    };
    check_disposal_text(control, &tag.name)?;
    tag.attributes.require_disposal_owner(control)?;
    for attribute in tag.attributes.as_slice() {
        cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
        check_disposal_text(control, &attribute.name)?;
        check_disposal_text(control, &attribute.value)?;
    }
    if control.stop().is_none() {
        payload_write::<Option<Tag>>(control, 2)?;
    }
    let Some(tag) = input.take() else {
        return Err(SessionError::Consumed);
    };
    tag.name.release(control)?;
    release_token_attributes(tag.attributes, control)
}

pub(super) fn discard_formatting_tag(
    input: &mut Option<Tag>,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    match release_formatting_tag(input, control) {
        Err(_) if control.stop().is_some() && input.is_some() => {
            release_formatting_tag(input, control)
        }
        Err(_) if control.stop().is_some() && input.is_none() => Ok(()),
        result => result,
    }
}

impl ParserArenaSession {
    /// Retains the actual fourteen ordinary formatting-start token families.
    /// Marker wrappers use ordinary allocation and never manufacture a recipe.
    pub(crate) fn retain_formatting_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        source: crate::HtmlSourceRange,
    ) -> Result<FormattingCreation, SessionError> {
        self.validate_standalone_document(control)?;
        control.require_source_range(&source)?;
        let tag = input.as_ref().ok_or(SessionError::Consumed)?;
        check_formatting_tag(control, tag, &source)?;
        let mut formatting = false;
        for local in [
            "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike", "strong",
            "tt", "u",
        ] {
            if exact_ascii(control, Some(tag.name.value.text()), local)? {
                formatting = true;
                break;
            }
        }
        if !formatting {
            return Err(SessionError::WrongToken);
        }
        // Every fallible admission precedes the original token take.
        control.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
        payload_write::<FormattingCreation>(control, 1)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let arena = self
            .active()?
            .construction
            .node_at(0)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let owner = control.identity();
        let cause = self
            .active()?
            .evidence
            .as_ref()
            .map(|evidence| evidence.current_cause(control))
            .transpose()?
            .flatten();
        let tag = take_payload(input, control)?;
        Ok(FormattingCreation {
            token: Some(tag),
            source,
            control: owner,
            arena,
            cause,
        })
    }

    pub(super) fn require_formatting_creation<'a>(
        &self,
        control: &mut ParseControl,
        creation: &'a FormattingCreation,
    ) -> Result<&'a Tag, SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.require_identity(&creation.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        storage
            .construction
            .position(&creation.arena)
            .map_err(|_| SessionError::ForeignNode)?;
        control.require_source_range(&creation.source)?;
        let tag = creation.token.as_ref().ok_or(SessionError::Consumed)?;
        check_formatting_tag(control, tag, &creation.source)?;
        Ok(tag)
    }

    /// Creates a detached native element for the original token. The caller
    /// supplies the actual source-required intended parent, not current-node
    /// inference. Partial copied payloads remain in this arena or are disposed.
    pub(crate) fn create_formatting_from_creation(
        &mut self,
        control: &mut ParseControl,
        creation: &FormattingCreation,
        intended_parent: &ParserNode,
    ) -> Result<ParserNode, SessionError> {
        self.validate_standalone_document(control)?;
        let storage = self.active()?;
        let parent = storage.index(control, intended_parent)?;
        storage.creation_parent(control, parent)?;
        if self
            .active_mut()?
            .associated_document_parent(control, parent)?
        {
            return Err(control
                .stop_unsupported_standalone(HtmlParseUnsupportedState::ScriptDocument)
                .into());
        }
        let storage = self.active()?;
        let tag = self.require_formatting_creation(control, creation)?;
        let document = storage.node_document_ref(control, parent)?;
        let mut prepared =
            self.active_mut()?
                .prepare_document_allocation(control, Some(document), false)?;
        let mut is_value = None;
        for attr in tag.attributes.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if exact_ascii(control, Some(attr.name.value.text()), "is")? {
                is_value = Some(&attr.value);
                break;
            }
        }
        self.active_mut()?.prepare_creation_registry(
            control,
            parent,
            tag.name.value.text(),
            is_value,
            true,
            &mut prepared,
        )?;
        let mut copy = Some(copy_formatting_tag(control, tag)?);
        let copied = match take_payload(&mut copy, control) {
            Ok(copied) => copied,
            Err(error) => {
                discard_formatting_tag(&mut copy, control)?;
                return Err(error);
            }
        };
        let previous = match self.active_mut()?.begin_formatting_replay(
            control,
            creation.cause.as_ref(),
            Some(parent),
        ) {
            Ok(previous) => previous,
            Err(error) => {
                copied.name.release(control)?;
                release_token_attributes(copied.attributes, control)?;
                return Err(error);
            }
        };
        // The copy keeps the positions of the tag it was created from, not of
        // the token that caused the reconstruction.
        let current_end = self.active_mut()?.token_end.replace(creation.source.end());
        let result = self.active_mut()?.allocate_html_payload_prepared(
            control,
            copied.name.value,
            Some(copied.name.origins),
            copied.attributes,
            prepared,
        );
        self.active_mut()?.token_end = current_end;
        if control.stop().is_none() {
            self.active_mut()?
                .end_node_copy(control, previous.as_ref())?;
        }
        match result {
            Ok(node) => Ok(node),
            Err(error) => {
                // Preconsumption failure still owns the actual independent
                // copy; postconsumption failure owns it in the arena registry.
                discard_formatting_tag(&mut copy, control)?;
                Err(error)
            }
        }
    }

    /// Noah equality compares parsed attributes as created, independent of
    /// order. This closed issuer uses HTML elements and no-namespace parsed
    /// attribute names; prefixes/colon units are not reinterpreted namespaces.
    pub(crate) fn same_formatting_creation(
        &self,
        control: &mut ParseControl,
        left: &FormattingCreation,
        right: &FormattingCreation,
    ) -> Result<bool, SessionError> {
        let left = self.require_formatting_creation(control, left)?;
        let right = self.require_formatting_creation(control, right)?;
        if !exact_string_equal(control, left.name.value.text(), right.name.value.text())? {
            return Ok(false);
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if left.attributes.len() != right.attributes.len() {
            return Ok(false);
        }
        for left_attribute in left.attributes.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let mut found = false;
            for right_attribute in right.attributes.as_slice() {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if exact_string_equal(
                    control,
                    left_attribute.name.value.text(),
                    right_attribute.name.value.text(),
                )? {
                    if !exact_string_equal(
                        control,
                        left_attribute.value.value.text(),
                        right_attribute.value.value.text(),
                    )? {
                        return Ok(false);
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Ordered, individually atomic moves. Each committed prefix has coherent
    /// backlinks; a stopped suffix remains owned by its original parent. Each
    /// move takes the first child of the source. The owner probes the last
    /// entry of the order and then the first, which it removes at the front
    /// without visiting or shifting the entries between, so each move costs a
    /// constant and the work is linear in the children moved (plus the work of
    /// each child's subtree).
    pub(crate) fn reparent_children(
        &mut self,
        control: &mut ParseControl,
        from: &ParserNode,
        to: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        let source = storage.index(control, from)?;
        let destination = storage.index(control, to)?;
        storage.container(control, source)?;
        storage.container(control, destination)?;
        if source == destination {
            return Err(SessionError::Cycle);
        }
        // Check the whole transfer before an earlier sibling can commit.
        control.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
        let source_handle = storage
            .construction
            .node_at(source)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let destination_handle = storage
            .construction
            .node_at(destination)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        self.active()?
            .check_child_transfer_record(control, &source_handle, &destination_handle)?;
        loop {
            let storage = self.active()?;
            storage.require(control)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if storage
                .construction
                .node_at(source)
                .ok_or_else(|| latch_registry_row(control))?
                .first_child()
                .is_none()
            {
                return Ok(());
            }
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<usize>() as u64,
            )?;
            let index = storage
                .construction
                .node_at(source)
                .ok_or_else(|| latch_registry_row(control))?
                .first_child()
                .ok_or_else(|| latch_registry_row(control))?
                .position();
            let child = storage.node(control, index)?;
            self.insert(control, to, &child, None, capability)?;
        }
    }
}

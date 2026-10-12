//! Managed copies in the same immutable-construction arena. No lexical token is
//! fabricated for DOM cloning; original source maps remain original evidence.

use super::*;

#[derive(Clone, Copy, Default)]
struct CloneBindings {
    association: Option<usize>,
    select: Option<usize>,
    textarea: Option<usize>,
    hidden: Option<usize>,
    output: Option<usize>,
    script: Option<usize>,
}

impl SessionStorage {
    /// Qualify the entire source closure before a selectedcontent target changes.
    pub(super) fn qualify_clone_subtree(
        &self,
        c: &mut ParseControl,
        root: usize,
    ) -> Result<(), SessionError> {
        let mut next = self.next_view_node(c, root, root, None)?;
        while let Some(node) = next {
            c.charge(WorkCategory::RecordVisit, 4)?;
            let record = &self.records[node];
            if record.owner.is_some()
                || record.shadow.is_some()
                || record.template_contents.is_some()
            {
                return Err(SessionError::AssociationMove);
            }
            match &record.kind {
                HtmlNodeKind::Element(element) => {
                    if !exact_ascii(
                        c,
                        element.name().namespace().uri(),
                        "http://www.w3.org/1999/xhtml",
                    )? || element.name().prefix().is_some()
                    {
                        return Err(SessionError::WrongToken);
                    }
                    let name = self.field_slot(c, StringLocation::ElementName(node))?;
                    control_field_owner(&self.payloads, c, name)?;
                    let namespace = self.field_slot(c, StringLocation::Namespace(node))?;
                    control_field_owner(&self.payloads, c, namespace)?;
                    let container = self.element_attribute_container(c, node)?;
                    for position in 0..element.attributes().len() {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let attr = &element.attributes().as_slice()[position];
                        if attr.name().namespace().uri().is_some() || attr.name().prefix().is_some()
                        {
                            return Err(SessionError::WrongToken);
                        }
                        let name =
                            self.field_slot(c, StringLocation::AttributeName(container, position))?;
                        control_field_owner(&self.payloads, c, name)?;
                        let value = self
                            .field_slot(c, StringLocation::AttributeValue(container, position))?;
                        control_field_owner(&self.payloads, c, value)?;
                    }
                    for name in ["select", "option", "selectedcontent"] {
                        if self.html_local(c, node, name)? {
                            self.select_metadata(c, node)?;
                        }
                    }
                    if self.html_local(c, node, "textarea")?
                        && self.textarea_slot(c, node)?.is_none()
                    {
                        return Err(SessionError::TextareaMetadataUnavailable);
                    }
                    if self.html_local(c, node, "script")? {
                        script_metadata(&self.payloads, c, node)?;
                    }
                    if self.html_local(c, node, "input")? {
                        self.hidden_clone_slot(c, node)?;
                    }
                    if self.html_local(c, node, "output")? {
                        self.output_clone_slot(c, node)?;
                    }
                }
                HtmlNodeKind::Text(_) => {
                    self.field_slot(c, StringLocation::Text(node))?;
                }
                HtmlNodeKind::Comment(_) => {
                    self.field_slot(c, StringLocation::Comment(node))?;
                }
                HtmlNodeKind::ProcessingInstruction(_) => {
                    self.field_slot(c, StringLocation::ProcessingInstructionTarget(node))?;
                    self.field_slot(c, StringLocation::ProcessingInstructionData(node))?;
                }
                _ => return Err(SessionError::WrongToken),
            }
            next = self.next_view_node(c, root, node, None)?;
        }
        Ok(())
    }

    /// One shallow managed clone. Child cloning is an owning iterative frame.
    pub(super) fn clone_node(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<usize, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        match &self.records[source].kind {
            HtmlNodeKind::Element(_) => self.clone_element(c, source),
            HtmlNodeKind::Text(_) | HtmlNodeKind::Comment(_) => {
                let text = matches!(self.records[source].kind, HtmlNodeKind::Text(_));
                let location = if text {
                    StringLocation::Text(source)
                } else {
                    StringLocation::Comment(source)
                };
                let source_slot = self.field_slot(c, location)?;
                let (value, slot) = self.copy_registered_field(c, source_slot)?;
                let len = value.len();
                // Admit the registry binding before installing its native
                // alias. Allocation success and binding must have no fallible
                // gap, including a stop at the first binding-copy admission.
                payload_write::<StringLocation>(c, 1)?;
                let node = self.allocate_owned(
                    c,
                    if text {
                        HtmlNodeKind::Text(value)
                    } else {
                        HtmlNodeKind::Comment(value)
                    },
                    false,
                    0,
                    0,
                )?;
                self.bind_string(
                    slot,
                    if text {
                        StringLocation::Text(node.index)
                    } else {
                        StringLocation::Comment(node.index)
                    },
                );
                payload_write::<usize>(c, 1)?;
                self.usage.utf16_units += len;
                Ok(node.index)
            }
            HtmlNodeKind::ProcessingInstruction(_) => {
                let slot =
                    self.field_slot(c, StringLocation::ProcessingInstructionTarget(source))?;
                let (target, target_slot) = self.copy_registered_field(c, slot)?;
                let slot = self.field_slot(c, StringLocation::ProcessingInstructionData(source))?;
                let (data, data_slot) = self.copy_registered_field(c, slot)?;
                let len = target.len().checked_add(data.len()).ok_or_else(|| {
                    c.fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                })?;
                payload_write::<[StringLocation; 2]>(c, 2)?;
                let node = self.allocate_owned(
                    c,
                    HtmlNodeKind::ProcessingInstruction(HtmlProcessingInstruction { target, data }),
                    false,
                    0,
                    0,
                )?;
                self.bind_string(
                    target_slot,
                    StringLocation::ProcessingInstructionTarget(node.index),
                );
                self.bind_string(
                    data_slot,
                    StringLocation::ProcessingInstructionData(node.index),
                );
                payload_write::<usize>(c, 1)?;
                self.usage.utf16_units += len;
                Ok(node.index)
            }
            _ => Err(SessionError::WrongToken),
        }
    }

    fn clone_element(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<usize, SessionError> {
        let bindings = self.prepare_clone_metadata(c, source)?;
        let slot = self.field_slot(c, StringLocation::ElementName(source))?;
        let (name, name_slot) = self.copy_registered_field(c, slot)?;
        let slot = self.field_slot(c, StringLocation::Namespace(source))?;
        let (namespace, namespace_slot) = self.copy_registered_field(c, slot)?;
        let units = name.len() + namespace.len();
        let (values, allocation) = c.new_vector()?;
        if let Err(error) = self.prepare_payload_slot_owned(c) {
            release_vector(c, values, allocation)?;
            return Err(error);
        }
        let container = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::Attributes(Some(
            AttributeStorage::Building {
                values,
                allocation,
                original_len: 0,
                reversed_pairs: 0,
            },
        )));
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        let RetainedPayload::Attributes(storage) = &mut self.payloads.values[container] else {
            unreachable!()
        };
        let AttributeStorage::Building {
            values, allocation, ..
        } = storage.take().expect("owned empty clone attributes")
        else {
            unreachable!()
        };
        let (backing, shared) = match c.share_owned(values) {
            Ok(value) => value,
            Err((stop, values)) => {
                *storage = Some(AttributeStorage::Building {
                    values,
                    allocation,
                    original_len: 0,
                    reversed_pairs: 0,
                });
                return Err(stop.into());
            }
        };
        let ordered = match OrderedAttributes::from_parser_storage(backing, c) {
            Ok(value) => value,
            Err((error, backing)) => {
                let values = c
                    .release_shared(backing, shared)
                    .unwrap_or_else(|_| panic!("unique clone attribute backing"));
                *storage = Some(AttributeStorage::Building {
                    values,
                    allocation,
                    original_len: 0,
                    reversed_pairs: 0,
                });
                return Err(match error {
                    ParserAttributeError::Stop(stop) => stop.into(),
                    ParserAttributeError::Duplicate { .. } => SessionError::DuplicateAttribute,
                });
            }
        };
        *storage = Some(AttributeStorage::Frozen {
            values: ordered,
            allocation,
            shared,
            element: None,
        });
        c.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<OrderedAttributes>(c, 1)?;
        let RetainedPayload::Attributes(Some(AttributeStorage::Frozen { values, .. })) =
            &self.payloads.values[container]
        else {
            unreachable!()
        };
        let attributes = values.clone();
        // Native installation and all payload alias locations are one fixed
        // commit. A terminal stop may precede allocation, never strand an
        // installed native backing with a detached/unbound registry location.
        payload_write::<[StringLocation; 2]>(c, 2)?;
        payload_write::<Option<usize>>(c, 1)?;
        let node = self.allocate_owned(
            c,
            HtmlNodeKind::Element(HtmlElement {
                name: QualifiedName::new(Namespace::from_uri(namespace), None, name),
                attributes,
            }),
            false,
            0,
            0,
        )?;
        self.bind_string(name_slot, StringLocation::ElementName(node.index));
        self.bind_string(namespace_slot, StringLocation::Namespace(node.index));
        let RetainedPayload::Attributes(Some(AttributeStorage::Frozen { element, .. })) =
            &mut self.payloads.values[container]
        else {
            unreachable!()
        };
        *element = Some(node.index);
        self.bind_clone_metadata(c, node.index, bindings)?;
        payload_write::<usize>(c, 1)?;
        self.usage.utf16_units += units;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(original) = &self.records[source].kind else {
            unreachable!()
        };
        let count = original.attributes().len();
        let original_container = self.element_attribute_container(c, source)?;
        for position in 0..count {
            let name_slot = self.field_slot(
                c,
                StringLocation::AttributeName(original_container, position),
            )?;
            let (name, name_slot) = self.copy_registered_field(c, name_slot)?;
            let value_slot = self.field_slot(
                c,
                StringLocation::AttributeValue(original_container, position),
            )?;
            let (value, value_slot) = self.copy_registered_field(c, value_slot)?;
            let units = name.len() + value.len();
            let (values, allocation) = c.new_vector()?;
            if let Err(error) = self.prepare_payload_slot_owned(c) {
                release_vector(c, values, allocation)?;
                return Err(error);
            }
            let stage = self.payloads.values.len();
            self.payloads.values.push(RetainedPayload::Attributes(Some(
                AttributeStorage::Building {
                    values,
                    allocation,
                    original_len: 1,
                    reversed_pairs: 0,
                },
            )));
            let RetainedPayload::Attributes(Some(AttributeStorage::Building {
                values: actual,
                allocation: ticket,
                ..
            })) = &mut self.payloads.values[stage]
            else {
                unreachable!()
            };
            c.reserve_vector(actual, ticket, 1)?;
            payload_write::<Attribute>(c, 1)?;
            payload_write::<[StringLocation; 2]>(c, 2)?;
            c.acquire(HtmlParseResource::ManagedAttributes, 1)?;
            actual.push(Attribute::new(
                QualifiedName::new(Namespace::none(), None, name),
                value,
            ));
            self.bind_string(name_slot, StringLocation::AttributeName(stage, 0));
            self.bind_string(value_slot, StringLocation::AttributeValue(stage, 0));
            self.edit_attributes(c, node.index, container, stage, 1, false)?;
            self.begin_attribute_transfer(c, stage, container, position)?;
            self.edit_attributes(c, node.index, container, stage, 1, true)?;
            self.clone_attribute_hook(c, node.index, bindings, container, position)?;
            self.finish_attribute_merge(
                c,
                PreparedAttributeMerge {
                    target: node.index,
                    container,
                    stage,
                    start: position,
                    count: 0,
                    units,
                },
            )?;
        }
        self.finish_clone_metadata(c, source, node.index, bindings)?;
        Ok(node.index)
    }

    pub(super) fn element_attribute_container(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<usize, SessionError> {
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(payload, RetainedPayload::Attributes(Some(AttributeStorage::Frozen { element: Some(owner), .. })) if *owner == node)
            {
                return Ok(slot);
            }
        }
        Err(SessionError::AttributeStorageUnavailable)
    }

    fn retain_clone_metadata(
        &mut self,
        c: &mut ParseControl,
        payload: RetainedPayload,
    ) -> Result<usize, SessionError> {
        self.prepare_payload_slot_owned(c)?;
        let slot = self.payloads.values.len();
        self.payloads.values.push(payload);
        Ok(slot)
    }
    fn prepare_clone_metadata(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<CloneBindings, SessionError> {
        payload_write::<CloneBindings>(c, 1)?;
        let mut result = CloneBindings::default();
        if let Some((_, state)) = find_form_association_state(&self.payloads, c, source)? {
            payload_write::<Option<usize>>(c, 1)?;
            result.association = Some(self.retain_clone_metadata(
                c,
                RetainedPayload::FormAssociation {
                    node: None,
                    state: FormAssociationState {
                        kind: state.kind,
                        owner: None,
                        parser_inserted: false,
                        evidence: FormAssociationEvidence::Created,
                    },
                },
            )?);
        }
        for (name, state) in [
            (
                "select",
                SelectMetadata::Select(select::SelectState {
                    user_validity: false,
                }),
            ),
            (
                "option",
                SelectMetadata::Option(select::OptionState::fresh()),
            ),
            (
                "selectedcontent",
                SelectMetadata::Selectedcontent(select::SelectedcontentState { disabled: false }),
            ),
        ] {
            if self.html_local(c, source, name)? {
                self.select_metadata(c, source)?;
                payload_write::<Option<usize>>(c, 1)?;
                result.select = Some(self.retain_clone_metadata(
                    c,
                    RetainedPayload::SelectFamily { node: None, state },
                )?);
                break;
            }
        }
        if self.html_local(c, source, "textarea")? {
            self.textarea_slot(c, source)?
                .ok_or(SessionError::TextareaMetadataUnavailable)?;
            let empty = literal_string(c, b"")?;
            let raw = self.register_raw(c, empty)?;
            payload_write::<Option<usize>>(c, 1)?;
            result.textarea = Some(self.retain_clone_metadata(
                c,
                RetainedPayload::Textarea {
                    node: None,
                    state: TextareaState {
                        raw,
                        dirty_value: false,
                        user_validity: false,
                    },
                },
            )?);
        }
        if self.html_local(c, source, "input")? {
            self.hidden_clone_slot(c, source)?;
            let empty = literal_string(c, b"")?;
            let raw = self.register_raw(c, empty)?;
            payload_write::<Option<usize>>(c, 1)?;
            result.hidden = Some(self.retain_clone_metadata(
                c,
                RetainedPayload::HiddenInput {
                    node: None,
                    raw,
                    hidden: false,
                    complete: false,
                    state: HiddenInputState {
                        dirty_value: false,
                        dirty_checkedness: false,
                        checkedness: false,
                        indeterminate: false,
                        user_validity: false,
                        selected_files_empty: true,
                        value_rewritten_during_creation: false,
                    },
                },
            )?);
        }
        if self.html_local(c, source, "output")? {
            self.output_clone_slot(c, source)?;
            payload_write::<Option<usize>>(c, 1)?;
            result.output = Some(self.retain_clone_metadata(
                c,
                RetainedPayload::Output {
                    node: None,
                    state: OutputState {
                        default_value_override: OutputDefaultValueOverride::Null,
                    },
                },
            )?);
        }
        if self.html_local(c, source, "script")? {
            script_metadata(&self.payloads, c, source)?;
            payload_write::<Option<usize>>(c, 1)?;
            result.script = Some(self.retain_clone_metadata(
                c,
                RetainedPayload::Script {
                    node: None,
                    metadata: ScriptMetadata {
                        parser_document: None,
                        preparation_document: None,
                        force_async: true,
                        already_started: false,
                        script_type: None,
                        external_file: false,
                        ready_to_be_parser_executed: false,
                        delaying_load: false,
                        result: ScriptResult::Uninitialized,
                        steps_when_result_ready: None,
                    },
                },
            )?);
        }
        Ok(result)
    }
    fn bind_clone_metadata(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        bindings: CloneBindings,
    ) -> Result<(), SessionError> {
        for slot in [
            bindings.association,
            bindings.select,
            bindings.textarea,
            bindings.hidden,
            bindings.output,
            bindings.script,
        ]
        .into_iter()
        .flatten()
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            payload_write::<Option<usize>>(c, 1)?;
            match &mut self.payloads.values[slot] {
                RetainedPayload::FormAssociation { node: bound, .. }
                | RetainedPayload::SelectFamily { node: bound, .. }
                | RetainedPayload::Textarea { node: bound, .. }
                | RetainedPayload::HiddenInput { node: bound, .. }
                | RetainedPayload::Output { node: bound, .. }
                | RetainedPayload::Script { node: bound, .. } => *bound = Some(node),
                _ => return Err(SessionError::WrongToken),
            }
        }
        Ok(())
    }
    fn clone_attribute_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        bindings: CloneBindings,
        container: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        if let Some(input) = bindings.hidden {
            self.initial_input_attribute(
                c,
                node,
                input,
                bindings
                    .association
                    .ok_or(SessionError::FormAssociationMetadataUnavailable)?,
                container,
                position,
            )?;
        }
        if bindings.select.is_some() || self.html_local(c, node, "optgroup")? {
            self.initial_select_attribute(c, node, position)?;
        }
        if let Some(slot) = bindings.script {
            c.charge(WorkCategory::RecordVisit, 2)?;
            let HtmlNodeKind::Element(element) = &self.records[node].kind else {
                unreachable!()
            };
            if exact_ascii(
                c,
                Some(
                    element.attributes().as_slice()[position]
                        .name()
                        .local_name(),
                ),
                "async",
            )? {
                let (_, mut metadata) = script_metadata(&self.payloads, c, node)?;
                metadata.force_async = false;
                set_script_metadata(self, c, slot, metadata)?;
            }
        }
        Ok(())
    }
    fn finish_clone_metadata(
        &mut self,
        c: &mut ParseControl,
        source: usize,
        copy: usize,
        bindings: CloneBindings,
    ) -> Result<(), SessionError> {
        if let Some(slot) = bindings.script {
            let (_, original) = script_metadata(&self.payloads, c, source)?;
            let (_, mut state) = script_metadata(&self.payloads, c, copy)?;
            state.already_started = original.already_started;
            set_script_metadata(self, c, slot, state)?;
        }
        if let Some(slot) = bindings.textarea {
            let (_, original) = self
                .textarea_slot(c, source)?
                .ok_or(SessionError::TextareaMetadataUnavailable)?;
            let raw = self.copy_raw_field(c, original.raw)?;
            let (_, mut state) = self
                .textarea_slot(c, copy)?
                .ok_or(SessionError::TextareaMetadataUnavailable)?;
            let retired = state.raw;
            state.raw = raw;
            state.dirty_value = original.dirty_value;
            payload_write::<TextareaState>(c, 1)?;
            let RetainedPayload::Textarea { state: stored, .. } = &mut self.payloads.values[slot]
            else {
                unreachable!()
            };
            *stored = state;
            self.retire_raw_field(c, retired)?;
        }
        if let Some(slot) = bindings.hidden {
            let (_, original_raw, original) = self.hidden_clone_slot(c, source)?;
            let text = copy_input_value(&self.payloads, c, original_raw, false)?;
            self.replace_input_raw(c, slot, text)?;
            let mut state = self.read_input_state(c, slot)?;
            state.dirty_value = original.dirty_value;
            state.dirty_checkedness = original.dirty_checkedness;
            state.checkedness = original.checkedness;
            state.indeterminate = original.indeterminate;
            self.write_input_state(c, slot, state)?;
            payload_write::<[bool; 2]>(c, 2)?;
            let RetainedPayload::HiddenInput {
                hidden, complete, ..
            } = &mut self.payloads.values[slot]
            else {
                unreachable!()
            };
            *hidden = true;
            *complete = true;
        }
        if bindings.select.is_some() {
            self.initial_select_reset(c, copy)?;
        }
        Ok(())
    }
    fn hidden_clone_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(usize, usize, HiddenInputState), SessionError> {
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::HiddenInput {
                node: Some(owner),
                raw,
                complete: true,
                state,
                ..
            } = payload
                && *owner == node
            {
                c.charge(
                    WorkCategory::CopyByte,
                    (mem::size_of::<usize>() + mem::size_of::<HiddenInputState>()) as u64,
                )?;
                return Ok((slot, *raw, *state));
            }
        }
        Err(SessionError::HiddenInputMetadataUnavailable)
    }
    fn output_clone_slot(&self, c: &mut ParseControl, node: usize) -> Result<usize, SessionError> {
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(payload, RetainedPayload::Output { node: Some(owner), .. } if *owner == node)
            {
                return Ok(slot);
            }
        }
        Err(SessionError::OutputMetadataUnavailable)
    }
    fn copy_raw_field(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<usize, SessionError> {
        control_field_owner(&self.payloads, c, source)?;
        let mut buffer = Utf16Buffer::new(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let RetainedPayload::String { value, .. } = &self.payloads.values[source] else {
            unreachable!()
        };
        if let Err(stop) = buffer.extend_units(value.code_units(), c) {
            buffer.release(c)?;
            return Err(stop.into());
        }
        let frozen = buffer.freeze(c)?;
        self.register_raw(c, frozen)
    }
    fn retire_raw_field(&mut self, c: &mut ParseControl, slot: usize) -> Result<(), SessionError> {
        payload_write::<RetainedPayload>(c, 2)?;
        let RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        } = mem::replace(&mut self.payloads.values[slot], RetainedPayload::Retired)
        else {
            return Err(SessionError::WrongToken);
        };
        drop(value);
        c.release_document_string(allocation)?;
        if let Some(maps) = origins {
            maps.release(c)?;
        }
        Ok(())
    }
    /// Copy a registered field without aliasing its native backing or ticket.
    pub(super) fn copy_registered_field(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<(DocumentString, usize), SessionError> {
        control_field_owner(&self.payloads, c, source)?;
        let mut buffer = Utf16Buffer::new(c)?;
        let mut maps = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let result = (|| {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let RetainedPayload::String { value, origins, .. } = &self.payloads.values[source]
            else {
                return Err(SessionError::WrongToken);
            };
            buffer.extend_units(value.code_units(), c)?;
            if let Some(origins) = origins {
                for origin in origins.as_slice() {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    c.require_source_range(&origin.source)?;
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    payload_write::<TextOrigin>(c, 1)?;
                    maps.try_push(
                        c,
                        TextOrigin {
                            start: origin.start,
                            end: origin.end,
                            source: origin.source.clone(),
                            kind: origin.kind,
                        },
                    )?;
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            buffer.release(c)?;
            maps.release(c)?;
            return Err(error);
        }
        let frozen = match buffer.freeze(c) {
            Ok(value) => value,
            Err(stop) => {
                maps.release(c)?;
                return Err(stop.into());
            }
        };
        if let Err(error) = self.prepare_payload_slot_owned(c) {
            frozen.release(c)?;
            maps.release(c)?;
            return Err(error);
        }
        let (value, allocation) = frozen.into_parts();
        let slot = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::String {
            value,
            allocation,
            origins: Some(maps),
            location: StringLocation::Detached,
        });
        let RetainedPayload::String {
            value, allocation, ..
        } = &mut self.payloads.values[slot]
        else {
            unreachable!()
        };
        c.move_document_string_to_managed(allocation, value)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        c.charge(
            WorkCategory::CopyByte,
            mem::size_of::<DocumentString>() as u64,
        )?;
        Ok((value.clone(), slot))
    }
    pub(super) fn field_slot(
        &self,
        c: &mut ParseControl,
        location: StringLocation,
    ) -> Result<usize, SessionError> {
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::String {
                location: stored, ..
            } = payload
                && same_location(*stored, location)
            {
                return Ok(slot);
            }
        }
        Err(SessionError::WrongToken)
    }
}

fn control_field_owner(
    payloads: &PayloadStorage,
    c: &mut ParseControl,
    slot: usize,
) -> Result<(), SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    let Some(RetainedPayload::String {
        value,
        allocation,
        origins,
        ..
    }) = payloads.values.get(slot)
    else {
        return Err(SessionError::WrongToken);
    };
    c.require_document_string(allocation, value)?;
    if let Some(maps) = origins {
        maps.require_disposal_owner(c)?;
    }
    Ok(())
}

fn same_location(a: StringLocation, b: StringLocation) -> bool {
    match (a, b) {
        (StringLocation::ElementName(a), StringLocation::ElementName(b))
        | (StringLocation::Namespace(a), StringLocation::Namespace(b))
        | (StringLocation::Text(a), StringLocation::Text(b))
        | (StringLocation::Comment(a), StringLocation::Comment(b))
        | (
            StringLocation::ProcessingInstructionTarget(a),
            StringLocation::ProcessingInstructionTarget(b),
        )
        | (
            StringLocation::ProcessingInstructionData(a),
            StringLocation::ProcessingInstructionData(b),
        ) => a == b,
        (StringLocation::AttributeName(a, x), StringLocation::AttributeName(b, y))
        | (StringLocation::AttributeValue(a, x), StringLocation::AttributeValue(b, y)) => {
            a == b && x == y
        }
        _ => false,
    }
}

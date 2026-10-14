//! Managed copies in the same immutable-construction arena. No lexical token is
//! fabricated for DOM cloning; original source maps remain original evidence.

use super::*;

#[derive(Clone, Copy, Default)]
struct CloneBindings {
    details: Option<usize>,
    link: Option<usize>,
    association: Option<usize>,
    select: Option<usize>,
    textarea: Option<usize>,
    input: Option<usize>,
    output: Option<usize>,
    script: Option<usize>,
    svg: Option<usize>,
    registry: Option<usize>,
}

impl SessionStorage {
    /// Qualify the entire source closure before a selectedcontent target changes.
    pub(super) fn qualify_clone_subtree(
        &self,
        c: &mut ParseControl,
        root: usize,
    ) -> Result<(), SessionError> {
        let mut pending = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let result = (|| {
            self.node_document_ref(c, root)?;
            for child in &self.records[root].children {
                c.charge(WorkCategory::RecordVisit, 1)?;
                pending.try_push(c, *child)?;
            }
            while let Some(node) = pending.try_pop(c)? {
                self.node_document_ref(c, node)?;
                c.charge(WorkCategory::RecordVisit, 4)?;
                let record = &self.records[node];
                if let Some(shadow) = &record.shadow {
                    if self.fragment.is_none() {
                        return Err(SessionError::AssociationMove);
                    }
                    self.shadow_state(c, shadow.contents.index)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        mem::size_of::<HtmlShadowRootProperties>() as u64,
                    )?;
                    if shadow.properties.clonable() {
                        self.node_document_ref(c, shadow.contents.index)?;
                        self.lookup_registry_ref(c, shadow.contents.index)?;
                        for child in &self.records[shadow.contents.index].children {
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            pending.try_push(c, *child)?;
                        }
                    }
                }
                match &record.kind {
                    HtmlNodeKind::Element(element) => {
                        let namespace = element.name().namespace().uri();
                        let supported = self.fragment.is_some()
                            || exact_ascii(c, namespace, "http://www.w3.org/1999/xhtml")?
                            || exact_ascii(c, namespace, "http://www.w3.org/2000/svg")?
                            || exact_ascii(c, namespace, "http://www.w3.org/1998/Math/MathML")?;
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if !supported || element.name().prefix().is_some() {
                            return Err(SessionError::WrongToken);
                        }
                        let name = self.field_slot(c, StringLocation::ElementName(node))?;
                        control_field_owner(&self.payloads, c, name)?;
                        if namespace.is_some() {
                            let namespace = self.field_slot(c, StringLocation::Namespace(node))?;
                            control_field_owner(&self.payloads, c, namespace)?;
                        }
                        let container = self.element_attribute_container(c, node)?;
                        for position in 0..element.attributes().len() {
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            let attr = &element.attributes().as_slice()[position];
                            c.charge(WorkCategory::RecordVisit, 2)?;
                            for location in [
                                attr.name().namespace().uri().map(|_| {
                                    StringLocation::AttributeNamespace(container, position)
                                }),
                                attr.name()
                                    .prefix()
                                    .map(|_| StringLocation::AttributePrefix(container, position)),
                            ]
                            .into_iter()
                            .flatten()
                            {
                                let slot = self.field_slot(c, location)?;
                                control_field_owner(&self.payloads, c, slot)?;
                            }
                            let name = self.field_slot(
                                c,
                                StringLocation::AttributeName(container, position),
                            )?;
                            control_field_owner(&self.payloads, c, name)?;
                            let value = self.field_slot(
                                c,
                                StringLocation::AttributeValue(container, position),
                            )?;
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
                        if self.svg_script_local(c, node)? {
                            self.svg_state_at(c, node)?;
                        }
                        if self.html_local(c, node, "script")? {
                            script_metadata(&self.payloads, c, node)?;
                        }
                        if self.html_local(c, node, "input")? {
                            self.input_clone_slot(c, node)?;
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
                        self.pi_attribute(c, node, "name")?;
                    }
                    _ => return Err(SessionError::WrongToken),
                }

                if let Some(contents) = self.records[node].template_contents {
                    self.template_slot(c, node)?;
                    self.node_document_ref(c, contents)?;
                    for child in &self.records[contents].children {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        pending.try_push(c, *child)?;
                    }
                }
                for child in &self.records[node].children {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    pending.try_push(c, *child)?;
                }
            }
            Ok(())
        })();
        let cleanup = pending.release(c);
        result.and_then(|()| cleanup.map_err(SessionError::from))?;
        Ok(())
    }

    pub(super) fn start_template_clone(
        &mut self,
        c: &mut ParseControl,
        source: usize,
        copy: usize,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.records[source].template_contents.is_some() {
            let (_, source) = self.template_slot(c, source)?;
            let (_, copy) = self.template_slot(c, copy)?;
            let document = self.node_document_ref(c, copy.contents)?;
            self.start_effect(
                c,
                select::EffectFrame::CloneChildren {
                    source: source.contents,
                    copy: copy.contents,
                    cursor: 0,
                    document,
                },
            )?;
        }
        Ok(())
    }

    /// One shallow managed clone. Child cloning is an owning iterative frame.
    pub(super) fn clone_node(
        &mut self,
        c: &mut ParseControl,
        source: usize,
        document: DocumentRef,
    ) -> Result<usize, SessionError> {
        let previous = self.begin_node_copy(c, source)?;
        let result = self.clone_node_inner(c, source, document);
        if c.stop().is_none() {
            self.end_node_copy(c, previous.as_ref())?;
        }
        result
    }
    fn clone_node_inner(
        &mut self,
        c: &mut ParseControl,
        source: usize,
        document: DocumentRef,
    ) -> Result<usize, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        match &self.records[source].kind {
            HtmlNodeKind::Element(_) => self.clone_element(c, source, document),
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
                let node = self.allocate_in_document_owned(
                    c,
                    if text {
                        HtmlNodeKind::Text(value)
                    } else {
                        HtmlNodeKind::Comment(value)
                    },
                    false,
                    0,
                    0,
                    Some(document),
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
                let map = self.prepare_pi_map(c)?;
                payload_write::<Option<usize>>(c, 1)?;
                let slot =
                    self.field_slot(c, StringLocation::ProcessingInstructionTarget(source))?;
                let (target, target_slot) = self.copy_registered_field(c, slot)?;
                let slot = self.field_slot(c, StringLocation::ProcessingInstructionData(source))?;
                let (data, data_slot) = self.copy_registered_field(c, slot)?;
                let len = target.len().checked_add(data.len()).ok_or_else(|| {
                    c.fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                })?;
                payload_write::<[StringLocation; 2]>(c, 2)?;
                let node = self.allocate_in_document_owned(
                    c,
                    HtmlNodeKind::ProcessingInstruction(HtmlProcessingInstruction { target, data }),
                    false,
                    0,
                    0,
                    Some(document),
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
                self.bind_pi_map(map, node.index);
                self.parse_pi_map(c, node.index, map)?;
                Ok(node.index)
            }
            _ => Err(SessionError::WrongToken),
        }
    }

    fn clone_element(
        &mut self,
        c: &mut ParseControl,
        source: usize,
        document: DocumentRef,
    ) -> Result<usize, SessionError> {
        let template = self.html_local(c, source, "template")?;
        if template {
            self.template_slot(c, source)?;
        }
        let prepared = self.prepare_document_allocation(c, Some(document), template)?;
        let bindings = self.prepare_clone_metadata(c, source)?;
        let slot = self.field_slot(c, StringLocation::ElementName(source))?;
        let (name, name_slot) = self.copy_registered_field(c, slot)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(original) = &self.records[source].kind else {
            unreachable!()
        };
        let namespace_present = original.name().namespace().uri().is_some();
        let (namespace, namespace_slot, namespace_units) = if namespace_present {
            let slot = self.field_slot(c, StringLocation::Namespace(source))?;
            let (namespace, slot) = self.copy_registered_field(c, slot)?;
            let units = namespace.len();
            (Namespace::from_uri(namespace), Some(slot), units)
        } else {
            (Namespace::none(), None, 0)
        };
        let units = name.len() + namespace_units;
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
        let ordered = match validate_parser_attributes(backing, c) {
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
                return Err(error);
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
        payload_write::<StringLocation>(c, 1 + u64::from(namespace_slot.is_some()))?;
        payload_write::<Option<usize>>(c, 1)?;
        let link_flags = self
            .html_local(c, source, "link")?
            .then(crate::HtmlLinkFlags::fresh);
        let node = self.allocate_prepared_owned(
            c,
            HtmlNodeKind::Element(HtmlElement {
                link_flags,
                name: QualifiedName::new(namespace, None, name),
                attributes,
            }),
            template,
            0,
            0,
            prepared,
        )?;
        self.bind_string(name_slot, StringLocation::ElementName(node.index));
        if let Some(slot) = namespace_slot {
            self.bind_string(slot, StringLocation::Namespace(node.index));
        }
        let RetainedPayload::Attributes(Some(AttributeStorage::Frozen { element, .. })) =
            &mut self.payloads.values[container]
        else {
            unreachable!()
        };
        *element = Some(node.index);
        self.bind_clone_metadata(c, node.index, bindings)?;
        if base::is_base(&self.records[source]) {
            let slot = self.base_slot(c, source)?;
            c.charge(
                WorkCategory::CopyByte,
                mem::size_of::<base::ResponseDomain>() as u64,
            )?;
            let RetainedPayload::Base { domain, .. } = &self.payloads.values[slot] else {
                unreachable!()
            };
            let domain = *domain;
            self.register_base(c, node.index, domain)?;
        }
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
            c.charge(WorkCategory::RecordVisit, 2)?;
            let HtmlNodeKind::Element(original) = &self.records[source].kind else {
                unreachable!()
            };
            let attribute = &original.attributes().as_slice()[position];
            let has_namespace = attribute.name().namespace().uri().is_some();
            let has_prefix = attribute.name().prefix().is_some();
            let (namespace, namespace_slot) = if has_namespace {
                let slot = self.field_slot(
                    c,
                    StringLocation::AttributeNamespace(original_container, position),
                )?;
                let (value, slot) = self.copy_registered_field(c, slot)?;
                (Namespace::from_uri(value), Some(slot))
            } else {
                (Namespace::none(), None)
            };
            let (prefix, prefix_slot) = if has_prefix {
                let slot = self.field_slot(
                    c,
                    StringLocation::AttributePrefix(original_container, position),
                )?;
                let (value, slot) = self.copy_registered_field(c, slot)?;
                (Some(value), Some(slot))
            } else {
                (None, None)
            };
            let units = name.len()
                + value.len()
                + namespace.uri().map_or(0, DocumentString::len)
                + prefix.as_ref().map_or(0, DocumentString::len);
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
            if namespace_slot.is_some() {
                payload_write::<StringLocation>(c, 1)?;
            }
            if prefix_slot.is_some() {
                payload_write::<StringLocation>(c, 1)?;
            }
            c.acquire(HtmlParseResource::ManagedAttributes, 1)?;
            actual.push(Attribute::new(
                QualifiedName::new(namespace, prefix, name),
                value,
            ));
            self.bind_string(name_slot, StringLocation::AttributeName(stage, 0));
            self.bind_string(value_slot, StringLocation::AttributeValue(stage, 0));
            if let Some(slot) = namespace_slot {
                self.bind_string(slot, StringLocation::AttributeNamespace(stage, 0));
            }
            if let Some(slot) = prefix_slot {
                self.bind_string(slot, StringLocation::AttributePrefix(stage, 0));
            }
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
        if self.html_local(c, source, "link")? {
            payload_write::<Option<usize>>(c, 1)?;
            result.link = Some(self.retain_clone_metadata(
                c,
                RetainedPayload::Link {
                    node: None,
                    parser_document: None,
                },
            )?);
        }
        if self.fragment.is_some() {
            let registry = self.lookup_registry_ref(c, source)?;
            let original = self.element_creation_state(c, source)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let HtmlNodeKind::Element(element) = &self.records[source].kind else {
                return Err(SessionError::WrongToken);
            };
            let html = exact_ascii(
                c,
                element.name().namespace().uri(),
                "http://www.w3.org/1999/xhtml",
            )?;
            let saved = if let Some(slot) = original.saved_is {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let RetainedPayload::String { value, .. } = &self.payloads.values[slot] else {
                    return Err(SessionError::ContextUnavailable);
                };
                Some(value)
            } else {
                None
            };
            self.require_no_definition(c, registry, element.name().local_name(), saved, html)?;
            let saved_is = if let Some(slot) = original.saved_is {
                let (value, copied) = self.copy_registered_field(c, slot)?;
                drop(value);
                Some(copied)
            } else {
                None
            };
            payload_write::<Option<usize>>(c, 1)?;
            result.registry = Some(self.retain_clone_metadata(
                c,
                RetainedPayload::LookupRegistry {
                    node: None,
                    registry,
                    creation: Some(context::ElementCreationState {
                        state: original.state,
                        saved_is,
                    }),
                },
            )?);
        }
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
                    flags: crate::HtmlSelectFlags::default(),
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
                        flags: HtmlTextareaFlags::default(),
                    },
                },
            )?);
        }
        if self.html_local(c, source, "input")? {
            let (_, _, original) = self.input_clone_slot(c, source)?;
            // This incomplete clone is already qualified as the actual Color
            // source. Keep fresh dirty flags; copy current and source flags only
            // at completion, independently of its copied native default.
            let flags = if original.flags.input_type() == crate::HtmlInputType::Color {
                crate::HtmlInputFlags::fresh().type_changed(crate::HtmlInputType::Color)
            } else {
                crate::HtmlInputFlags::fresh()
            };
            let empty = literal_string(c, b"")?;
            let raw = self.register_raw(c, empty)?;
            payload_write::<Option<usize>>(c, 1)?;
            result.input = Some(self.retain_clone_metadata(
                c,
                RetainedPayload::Input {
                    node: None,
                    raw,
                    complete: false,
                    state: InputState {
                        flags,
                        cursor: 0,
                        selected_files_empty: true,
                        value_rewritten_during_creation: false,
                        email_value_count: None,
                        image: input_image::ImageState::NeverEntered,
                    },
                },
            )?);
        }
        if self.html_local(c, source, "details")? {
            let notification = self.details_domain(c, source)?;
            if self.form_attribute(c, source, "open")?.is_some() {
                self.require_details_notification(c, source)?;
            }
            payload_write::<Option<usize>>(c, 1)?;
            result.details = Some(self.retain_clone_metadata(
                c,
                RetainedPayload::Details {
                    node: None,
                    notification,
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
        if self.svg_script_local(c, source)? {
            let (_, original) = self.svg_state_at(c, source)?;
            payload_write::<Option<usize>>(c, 1)?;
            result.svg = Some(self.retain_svg_slot(
                c,
                SvgScriptState {
                    processed: original.processed,
                    origin: SvgScriptOrigin::Clone,
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
            bindings.details,
            bindings.link,
            bindings.association,
            bindings.select,
            bindings.textarea,
            bindings.input,
            bindings.output,
            bindings.script,
            bindings.svg,
            bindings.registry,
        ]
        .into_iter()
        .flatten()
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            payload_write::<Option<usize>>(c, 1)?;
            match &mut self.payloads.values[slot] {
                RetainedPayload::Details { node: bound, .. }
                | RetainedPayload::Link { node: bound, .. }
                | RetainedPayload::LookupRegistry { node: bound, .. }
                | RetainedPayload::FormAssociation { node: bound, .. }
                | RetainedPayload::SelectFamily { node: bound, .. }
                | RetainedPayload::Textarea { node: bound, .. }
                | RetainedPayload::Input { node: bound, .. }
                | RetainedPayload::Output { node: bound, .. }
                | RetainedPayload::Script { node: bound, .. }
                | RetainedPayload::SvgScript { node: bound, .. } => *bound = Some(node),
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
        self.initial_nonce_attribute(c, node, position)?;
        if bindings.details.is_some() {
            self.initial_details_attribute(c, node, position)?;
        }
        if let Some(input) = bindings.input {
            let initialization = if self.read_input_state(c, input)?.flags.input_type()
                == crate::HtmlInputType::Color
            {
                InputAttributeInitialization::ClonedColor
            } else {
                InputAttributeInitialization::Installed
            };
            self.initial_input_attribute(
                c,
                node,
                input,
                bindings
                    .association
                    .ok_or(SessionError::FormAssociationMetadataUnavailable)?,
                (container, position),
                initialization,
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
        self.clone_nonce(c, source, copy)?;
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
            state.flags.clone_dirty_value_from(&original.flags);
            payload_write::<TextareaState>(c, 1)?;
            let RetainedPayload::Textarea { state: stored, .. } = &mut self.payloads.values[slot]
            else {
                unreachable!()
            };
            *stored = state;
            self.retire_raw_field(c, retired)?;
        }
        if let Some(slot) = bindings.input {
            let (_, original_raw, original) = self.input_clone_slot(c, source)?;
            let text = if matches!(
                original.flags.input_type(),
                crate::HtmlInputType::Range | crate::HtmlInputType::Color
            ) {
                provenance::copy_generated_input_value(
                    &self.payloads,
                    self.evidence.as_ref(),
                    c,
                    original_raw,
                )?
            } else {
                copy_preserved_input_value(&self.payloads, self.evidence.as_ref(), c, original_raw)?
            };
            self.replace_input_raw(c, slot, text)?;
            let mut state = self.read_input_state(c, slot)?;
            state.flags = state.flags.clone_flags_from(original.flags);
            state.email_value_count = original.email_value_count;
            self.write_input_state(c, slot, state)?;
            if state
                .flags
                .radio_normalization_required(crate::HtmlRadioTrigger::Checkedness)
            {
                self.normalize_radio(c, copy, None)?;
            }
            payload_write::<bool>(c, 1)?;
            let RetainedPayload::Input { complete, .. } = &mut self.payloads.values[slot] else {
                unreachable!()
            };
            *complete = true;
        }
        if bindings.select.is_some() {
            self.initial_select_reset(c, copy)?;
        }
        Ok(())
    }
    fn input_clone_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(usize, usize, InputState), SessionError> {
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::Input {
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
                    (mem::size_of::<usize>() + mem::size_of::<InputState>()) as u64,
                )?;
                return Ok((slot, *raw, *state));
            }
        }
        Err(SessionError::InputMetadataUnavailable)
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
    pub(super) fn copy_raw_field(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<usize, SessionError> {
        let (value, slot) = self.copy_registered_field(c, source)?;
        drop(value);
        Ok(slot)
    }
    pub(super) fn retire_raw_field(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        self.retire_registered_string(c, slot)
    }
    /// Copy a registered field without aliasing its native backing or ticket.
    pub(super) fn copy_registered_field(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<(DocumentString, usize), SessionError> {
        control_field_owner(&self.payloads, c, source)?;
        let mut buffer = Utf16Buffer::new(c)?;
        let result = (|| {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let RetainedPayload::String { value, origins, .. } = &self.payloads.values[source]
            else {
                return Err(SessionError::WrongToken);
            };
            buffer.extend_units(value.code_units(), c)?;
            origins
                .as_ref()
                .map(|origins| {
                    self.evidence
                        .as_mut()
                        .ok_or(SessionError::WrongToken)?
                        .copy_group(c, origins)
                })
                .transpose()
        })();
        let origins = match result {
            Ok(origins) => origins,
            Err(error) => {
                buffer.release(c)?;
                return Err(error);
            }
        };
        let frozen = buffer.freeze(c)?;
        self.register_group_string_owned(c, frozen, origins)
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
        | (StringLocation::AttributeValue(a, x), StringLocation::AttributeValue(b, y))
        | (StringLocation::AttributeNamespace(a, x), StringLocation::AttributeNamespace(b, y))
        | (StringLocation::AttributePrefix(a, x), StringLocation::AttributePrefix(b, y)) => {
            a == b && x == y
        }
        _ => false,
    }
}

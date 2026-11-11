//! Managed copies in the same immutable-construction arena. No lexical token is
//! fabricated for DOM cloning; original source maps remain original evidence.

use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlShadowRootProperties},
    elements::{
        form::HtmlTextareaFlags,
        script::{HtmlScriptFlags, HtmlScriptResult},
    },
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

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
            for ordinal in 0..self
                .construction
                .node_at(root)
                .ok_or_else(|| latch_registry_row(c))?
                .child_count()
            {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let child = self
                    .construction
                    .node_at(root)
                    .ok_or_else(|| latch_registry_row(c))?
                    .child_at(ordinal)
                    .ok_or_else(|| latch_registry_row(c))?
                    .position();
                pending.try_push_geometric(c, child)?;
            }
            while let Some(node) = pending.try_pop(c)? {
                self.node_document_ref(c, node)?;
                c.charge(HtmlParseWorkCategory::RecordVisit, 4)?;
                let record = self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?;
                if let Some(shadow) = record.shadow_root() {
                    c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                    let shadow_contents = self
                        .construction
                        .node_by_result_id(&shadow.contents())
                        .map_err(|_| SessionError::ForeignNode)?
                        .position();
                    if self.fragment.is_none() {
                        return Err(SessionError::AssociationMove);
                    }
                    self.shadow_state(c, shadow_contents)?;
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
                        mem::size_of::<HtmlShadowRootProperties>() as u64,
                    )?;
                    if self
                        .construction
                        .node_at(shadow_contents)
                        .ok_or_else(|| latch_registry_row(c))?
                        .shadow_properties()
                        .ok_or_else(|| latch_registry_row(c))?
                        .clonable()
                    {
                        self.node_document_ref(c, shadow_contents)?;
                        self.lookup_registry_ref(c, shadow_contents)?;
                        for ordinal in 0..self
                            .construction
                            .node_at(shadow_contents)
                            .ok_or_else(|| latch_registry_row(c))?
                            .child_count()
                        {
                            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                            let child = self
                                .construction
                                .node_at(shadow_contents)
                                .ok_or_else(|| latch_registry_row(c))?
                                .child_at(ordinal)
                                .ok_or_else(|| latch_registry_row(c))?
                                .position();
                            pending.try_push_geometric(c, child)?;
                        }
                    }
                }
                match record.kind() {
                    ConstructionNodeKind::Element(element) => {
                        let namespace = element.name().namespace().uri();
                        let supported = self.fragment.is_some()
                            || exact_ascii(c, namespace, "http://www.w3.org/1999/xhtml")?
                            || exact_ascii(c, namespace, "http://www.w3.org/2000/svg")?
                            || exact_ascii(c, namespace, "http://www.w3.org/1998/Math/MathML")?;
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        if !supported || element.name().prefix().is_some() {
                            return Err(SessionError::WrongToken);
                        }
                        let name = self.field_slot(c, FieldRef::ElementName(node))?;
                        control_field_owner(&self.payloads, c, name)?;
                        if namespace.is_some() {
                            let namespace = self.field_slot(c, FieldRef::Namespace(node))?;
                            control_field_owner(&self.payloads, c, namespace)?;
                        }
                        for (position, attr) in element.attributes().iter().enumerate() {
                            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                            for location in [
                                attr.name()
                                    .namespace()
                                    .uri()
                                    .map(|_| FieldRef::AttributeNamespace(node, position)),
                                attr.name()
                                    .prefix()
                                    .map(|_| FieldRef::AttributePrefix(node, position)),
                            ]
                            .into_iter()
                            .flatten()
                            {
                                let slot = self.field_slot(c, location)?;
                                control_field_owner(&self.payloads, c, slot)?;
                            }
                            let name =
                                self.field_slot(c, FieldRef::AttributeName(node, position))?;
                            control_field_owner(&self.payloads, c, name)?;
                            let value =
                                self.field_slot(c, FieldRef::AttributeValue(node, position))?;
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
                    ConstructionNodeKind::Text(_) | ConstructionNodeKind::Comment(_) => {}
                    ConstructionNodeKind::ProcessingInstruction(_) => {
                        self.pi_attribute(c, node, "name")?;
                    }
                    _ => return Err(SessionError::WrongToken),
                }

                if let Some(contents) = self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?
                    .template_contents()
                    .map(|node| node.position())
                {
                    self.template_slot(c, node)?;
                    self.node_document_ref(c, contents)?;
                    for ordinal in 0..self
                        .construction
                        .node_at(contents)
                        .ok_or_else(|| latch_registry_row(c))?
                        .child_count()
                    {
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        let child = self
                            .construction
                            .node_at(contents)
                            .ok_or_else(|| latch_registry_row(c))?
                            .child_at(ordinal)
                            .ok_or_else(|| latch_registry_row(c))?
                            .position();
                        pending.try_push_geometric(c, child)?;
                    }
                }
                for ordinal in 0..self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?
                    .child_count()
                {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let child = self
                        .construction
                        .node_at(node)
                        .ok_or_else(|| latch_registry_row(c))?
                        .child_at(ordinal)
                        .ok_or_else(|| latch_registry_row(c))?
                        .position();
                    pending.try_push_geometric(c, child)?;
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self
            .construction
            .node_at(source)
            .ok_or_else(|| latch_registry_row(c))?
            .template_contents()
            .is_some()
        {
            let (_, source) = self.template_slot(c, source)?;
            let (_, copy) = self.template_slot(c, copy)?;
            let document = self.node_document_ref(c, copy.contents)?;
            self.start_effect(
                c,
                effects::EffectFrame::CloneChildren {
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        match &self
            .construction
            .node_at(source)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        {
            ConstructionNodeKind::Element(_) => self.clone_element(c, source, document),
            ConstructionNodeKind::Text(_) | ConstructionNodeKind::Comment(_) => {
                let text = matches!(
                    self.construction
                        .node_at(source)
                        .ok_or_else(|| latch_registry_row(c))?
                        .kind(),
                    ConstructionNodeKind::Text(_)
                );
                let location = if text {
                    FieldRef::Text(source)
                } else {
                    FieldRef::Comment(source)
                };
                let field = self.copy_record_field(c, location)?;
                // The binding of the field follows the record's creation; a refusal of it stops
                // the parse, which publishes no record.
                let created = (|| {
                    payload_write::<FieldRef>(c, 1)?;
                    self.allocate_in_document_owned(
                        c,
                        if text {
                            NewRecord::Text(field.units())
                        } else {
                            NewRecord::Comment(field.units())
                        },
                        false,
                        0,
                        field.units().len(),
                        Some(document),
                    )
                })();
                let FieldString { value, group } = field;
                let released = value.release(c);
                let node = created?;
                released?;
                self.bind_field(
                    c,
                    if text {
                        FieldRef::Text(node.index)
                    } else {
                        FieldRef::Comment(node.index)
                    },
                    group,
                    0,
                )?;
                payload_write::<usize>(c, 1)?;

                Ok(node.index)
            }
            ConstructionNodeKind::ProcessingInstruction(_) => {
                let map = self.prepare_pi_map(c)?;
                payload_write::<Option<usize>>(c, 1)?;
                let target =
                    self.copy_record_field(c, FieldRef::ProcessingInstructionTarget(source))?;
                let data =
                    match self.copy_record_field(c, FieldRef::ProcessingInstructionData(source)) {
                        Ok(data) => data,
                        Err(error) => {
                            target.value.release(c)?;
                            return Err(error);
                        }
                    };
                let created = (|| {
                    payload_write::<[FieldRef; 2]>(c, 2)?;
                    self.allocate_in_document_owned(
                        c,
                        NewRecord::ProcessingInstruction(target.units(), data.units()),
                        false,
                        0,
                        target.units().len().saturating_add(data.units().len()),
                        Some(document),
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
                let released = target.release(c);
                let released_data = data.release(c);
                let node = created?;
                released?;
                released_data?;
                self.bind_field(
                    c,
                    FieldRef::ProcessingInstructionTarget(node.index),
                    target_group,
                    0,
                )?;
                self.bind_field(
                    c,
                    FieldRef::ProcessingInstructionData(node.index),
                    data_group,
                    0,
                )?;
                payload_write::<usize>(c, 1)?;

                self.bind_pi_map(c, map, node.index)?;
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
        let slot = self.field_slot(c, FieldRef::ElementName(source))?;
        let (name, name_slot) = self.copy_registered_field(c, slot)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let namespace_present = element_record(&self.construction, c, source)?
            .name()
            .namespace()
            .uri()
            .is_some();
        let (namespace, namespace_slot) = if namespace_present {
            let slot = self.field_slot(c, FieldRef::Namespace(source))?;
            let (namespace, slot) = self.copy_registered_literal_field(c, slot, false)?;
            (Namespace::from_uri(namespace), Some(slot))
        } else {
            (Namespace::none(), None)
        };
        // The element is created without attributes; they are appended one at a time below.
        let stage = StagedAttributes::new(c)?;
        let (values, allocation) = stage.into_parts();
        let (backing, shared) = match c.share_owned(values) {
            Ok(value) => value,
            Err((stop, values)) => {
                release_vector(c, values, allocation)?;
                return Err(stop.into());
            }
        };
        let ordered = match validate_parser_attributes(backing, c) {
            Ok(value) => value,
            Err((error, backing)) => {
                // The backing here is the empty vector just created, and validating fewer than
                // two attributes has no comparison to charge or refuse, so no limit or
                // cancellation reaches this arm and no test can force it with a work cut.
                return match c.release_shared(backing, shared) {
                    Ok(values) => {
                        release_vector(c, values, allocation)?;
                        Err(error)
                    }
                    Err(refused) => {
                        let (_, backing, shared) = refused.into_parts();
                        drop(backing);
                        self.holds.absorb_attribute_list(c, allocation, shared)?;
                        Err(latch_registry_row(c))
                    }
                };
            }
        };
        self.holds.absorb_attribute_list(c, allocation, shared)?;
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        payload_write::<OrderedAttributes>(c, 1)?;
        let attributes = ordered;
        // Native installation and all payload alias locations are one fixed
        // commit. A terminal stop may precede allocation, never strand an
        // installed native backing with a detached/unbound registry location.
        payload_write::<FieldRef>(c, u64::from(namespace_slot.is_some()).saturating_add(1))?;
        payload_write::<Option<usize>>(c, 1)?;
        let link_flags = self
            .html_local(c, source, "link")?
            .then(surgeist_html::elements::link::HtmlLinkFlags::fresh);
        let node = self.allocate_prepared_owned(
            c,
            NewRecord::Element(
                QualifiedName::new(namespace, None, name),
                attributes,
                link_flags,
            ),
            template,
            0,
            0,
            prepared,
        )?;
        self.bind_string(name_slot, FieldRef::ElementName(node.index))
            .map_err(|_| latch_registry_row(c))?;
        if let Some(slot) = namespace_slot {
            self.bind_string(slot, FieldRef::Namespace(node.index))
                .map_err(|_| latch_registry_row(c))?;
        }
        self.bind_clone_metadata(c, node.index, bindings)?;
        self.initial_associated_state(c, node.index)?;
        if base::is_base(
            self.construction
                .node_at(source)
                .ok_or_else(|| latch_registry_row(c))?,
        ) {
            let slot = self.base_slot(c, source)?;
            c.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<base::ResponseDomain>() as u64,
            )?;
            let Some(RetainedPayload::Base { domain, .. }) = self.payloads.values.get(slot) else {
                return Err(latch_registry_row(c));
            };
            let domain = *domain;
            self.register_base(c, node.index, domain)?;
        }
        payload_write::<usize>(c, 1)?;

        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let count = element_record(&self.construction, c, source)?
            .attributes()
            .len();
        // The clone's attributes are appended in one list, grown once to the source's length.
        self.reserve_attributes(c, node.index, count)?;
        for position in 0..count {
            let name_slot = self.field_slot(c, FieldRef::AttributeName(source, position))?;
            let (name, name_slot) = self.copy_registered_field(c, name_slot)?;
            let value_slot = self.field_slot(c, FieldRef::AttributeValue(source, position))?;
            let (value, value_slot) = self.copy_registered_field(c, value_slot)?;
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            let attribute = element_attribute(&self.construction, c, source, position)?;
            let has_namespace = attribute.name().namespace().uri().is_some();
            let has_prefix = attribute.name().prefix().is_some();
            let (namespace, namespace_slot) = if has_namespace {
                let slot = self.field_slot(c, FieldRef::AttributeNamespace(source, position))?;
                let (value, slot) = self.copy_registered_literal_field(c, slot, false)?;
                (Namespace::from_uri(value), Some(slot))
            } else {
                (Namespace::none(), None)
            };
            let (prefix, prefix_slot) = if has_prefix {
                let slot = self.field_slot(c, FieldRef::AttributePrefix(source, position))?;
                let (value, slot) = self.copy_registered_literal_field(c, slot, true)?;
                (Some(value), Some(slot))
            } else {
                (None, None)
            };
            let first_row = name_slot.min(value_slot);
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            payload_write::<Attribute>(c, 1)?;
            payload_write::<[FieldRef; 2]>(c, 2)?;
            if namespace_slot.is_some() {
                payload_write::<FieldRef>(c, 1)?;
            }
            if prefix_slot.is_some() {
                payload_write::<FieldRef>(c, 1)?;
            }
            self.holds.acquire_attribute(c)?;
            self.bind_string(name_slot, FieldRef::AttributeName(node.index, position))
                .map_err(|_| latch_registry_row(c))?;
            self.bind_string(value_slot, FieldRef::AttributeValue(node.index, position))
                .map_err(|_| latch_registry_row(c))?;
            if let Some(slot) = namespace_slot {
                self.bind_string(slot, FieldRef::AttributeNamespace(node.index, position))
                    .map_err(|_| latch_registry_row(c))?;
            }
            if let Some(slot) = prefix_slot {
                self.bind_string(slot, FieldRef::AttributePrefix(node.index, position))
                    .map_err(|_| latch_registry_row(c))?;
            }
            let mut incoming = Some(Attribute::new(
                QualifiedName::new(namespace, prefix, name),
                value,
            ));
            self.payloads.forget_attribute_strings(node.index);
            self.append_attribute_with_effects(c, node.index, &mut incoming, None)?;
            let first_row = [
                Some(name_slot),
                Some(value_slot),
                namespace_slot,
                prefix_slot,
            ]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(first_row);
            self.flush_attribute_origins(c, first_row)?;
            self.clone_attribute_hook(c, node.index, bindings, position)?;
        }
        self.finish_clone_metadata(c, source, node.index, bindings)?;
        Ok(node.index)
    }

    fn retain_clone_metadata(
        &mut self,
        c: &mut ParseControl,
        payload: RetainedPayload,
    ) -> Result<usize, SessionError> {
        let prepared = self.prepare_payload_slot_for(c, row_list(&payload))?;
        self.payloads
            .push_row(prepared, payload)
            .map_err(|_| latch_registry_row(c))
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
        {
            let registry = self.lookup_registry_ref(c, source)?;
            let original = self.element_creation_state(c, source)?;
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let ConstructionNodeKind::Element(element) = &self
                .construction
                .node_at(source)
                .ok_or_else(|| latch_registry_row(c))?
                .kind()
            else {
                return Err(SessionError::WrongToken);
            };
            let html = exact_ascii(
                c,
                element.name().namespace().uri(),
                "http://www.w3.org/1999/xhtml",
            )?;
            let saved = if let Some(slot) = original.saved_is {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let Some(RetainedPayload::String { value, .. }) = self.payloads.values.get(slot)
                else {
                    return Err(SessionError::ContextUnavailable);
                };
                Some(value)
            } else {
                None
            };
            self.require_no_definition(c, &registry, element.name().local_name(), saved, html)?;
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
                        common_scope: original.common_scope,
                        state: original.state,
                        saved_is,
                    }),
                },
            )?);
        }
        if let Some((_, state)) = find_form_association_state(&self.payloads, c, source)? {
            payload_write::<Option<usize>>(c, 1)?;
            // The clone copies the source's attributes, so it has a form
            // attribute exactly when the source has.
            let explicit_form =
                state.kind.listed() && self.form_attribute(c, source, "form")?.is_some();
            let prepared =
                self.prepare_payload_slot_with(c, Some(KindList::FormAssociation), explicit_form)?;
            let slot = self
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::FormAssociation {
                        node: None,
                        state: FormAssociationState {
                            kind: state.kind,
                            owner: None,
                            parser_inserted: false,
                            evidence: HtmlParseFormEvidence::Created,
                        },
                    },
                )
                .map_err(|_| latch_registry_row(c))?;
            result.association = Some(slot);
        }
        for (name, state) in [
            (
                "select",
                SelectMetadata::Select(select::SelectState::fresh()),
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
            let flags =
                if original.flags.input_type() == surgeist_html::attributes::HtmlInputType::Color {
                    surgeist_html::elements::input::HtmlInputFlags::fresh()
                        .type_changed(surgeist_html::attributes::HtmlInputType::Color)
                } else {
                    surgeist_html::elements::input::HtmlInputFlags::fresh()
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
                        default_value_override: HtmlParseOutputOverride::Null,
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
                        flags: HtmlScriptFlags::fresh(),
                        script_type: None,
                        result: HtmlScriptResult::Uninitialized,
                        steps_when_result_ready: None,
                        start: None,
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
                    flags: original.flags.clone_for_element(),
                    origin: HtmlParseSvgScriptOrigin::Clone,
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
        if bindings.textarea.is_some() {
            payload_write::<Option<usize>>(c, 1)?;
        }
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            payload_write::<Option<usize>>(c, 1)?;
            match self.payloads.values.get_mut(slot) {
                Some(
                    RetainedPayload::Details { node: bound, .. }
                    | RetainedPayload::Link { node: bound, .. }
                    | RetainedPayload::LookupRegistry { node: bound, .. }
                    | RetainedPayload::FormAssociation { node: bound, .. }
                    | RetainedPayload::SelectFamily { node: bound, .. }
                    | RetainedPayload::Textarea { node: bound, .. }
                    | RetainedPayload::Input { node: bound, .. }
                    | RetainedPayload::Output { node: bound, .. }
                    | RetainedPayload::Script { node: bound, .. }
                    | RetainedPayload::SvgScript { node: bound, .. },
                ) => *bound = Some(node),
                _ => return Err(SessionError::WrongToken),
            }
            self.payloads
                .join_row(slot)
                .map_err(|_| latch_registry_row(c))?;
        }
        if let Some(slot) = bindings.textarea {
            self.payloads
                .join_textarea(node, slot)
                .map_err(|_| latch_registry_row(c))?;
        }
        Ok(())
    }
    fn clone_attribute_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        bindings: CloneBindings,
        position: usize,
    ) -> Result<(), SessionError> {
        self.initial_nonce_attribute(c, node, position)?;
        self.initial_associated_attribute(c, node, position)?;
        self.initial_meta_attribute(c, node, position)?;
        self.association_attribute_hook(c, node, position)?;
        self.initial_dialog_attribute(c, node, position)?;
        if bindings.details.is_some() {
            self.initial_details_attribute(c, node, position)?;
        }
        if let Some(input) = bindings.input {
            let initialization = if self.read_input_state(c, input)?.flags.input_type()
                == surgeist_html::attributes::HtmlInputType::Color
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
                position,
                initialization,
            )?;
        }
        if bindings.select.is_some() || self.html_local(c, node, "optgroup")? {
            self.initial_select_attribute(c, node, position)?;
        }
        if let Some(slot) = bindings.script {
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            let attribute = element_attribute(&self.construction, c, node, position)?;
            if exact_ascii(c, Some(attribute.name().local_name()), "async")? {
                let (_, mut metadata) = script_metadata(&self.payloads, c, node)?;
                metadata.flags.async_attribute_added();
                set_script_metadata(self, c, slot, &metadata)?;
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
            state.flags.clone_already_started_from(original.flags);
            set_script_metadata(self, c, slot, &state)?;
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
            let Some(RetainedPayload::Textarea { state: stored, .. }) =
                self.payloads.values.get_mut(slot)
            else {
                return Err(latch_registry_row(c));
            };
            *stored = state;
            self.retire_raw_field(c, retired)?;
        }
        if let Some(slot) = bindings.input {
            let (_, original_raw, original) = self.input_clone_slot(c, source)?;
            let text = if matches!(
                original.flags.input_type(),
                surgeist_html::attributes::HtmlInputType::Range
                    | surgeist_html::attributes::HtmlInputType::Color
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
            if state.flags.radio_normalization_required(
                surgeist_html::elements::input::HtmlRadioTrigger::Checkedness,
            ) {
                self.normalize_radio(c, copy, None)?;
            }
            payload_write::<bool>(c, 1)?;
            let Some(RetainedPayload::Input { complete, .. }) = self.payloads.values.get_mut(slot)
            else {
                return Err(latch_registry_row(c));
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
        if let Some(slot) = self.payloads.joined(c, node, Join::Input)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::Input {
                node: Some(owner),
                raw,
                complete: true,
                state,
                ..
            }) = self.payloads.values.get(slot)
                && *owner == node
            {
                c.charge(
                    HtmlParseWorkCategory::CopyByte,
                    const { (mem::size_of::<usize>() + mem::size_of::<InputState>()) as u64 },
                )?;
                return Ok((slot, *raw, *state));
            }
        }
        Err(SessionError::InputMetadataUnavailable)
    }
    fn output_clone_slot(&self, c: &mut ParseControl, node: usize) -> Result<usize, SessionError> {
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
    /// Copies a registered namespace URI, or an attribute prefix when `prefix`, without aliasing
    /// its ticket. A value that is one of the parse's literals registers a row that shares the
    /// literal's string and keeps the copy of the field's origin group; any other value is copied
    /// as [`Self::copy_registered_field`] copies it, with the charges of that copy alone. Whether
    /// the value is a literal is read before anything is charged: it compares at most the units
    /// of the longest literal, and only when the lengths are equal.
    pub(super) fn copy_registered_literal_field(
        &mut self,
        c: &mut ParseControl,
        source: usize,
        prefix: bool,
    ) -> Result<(DocumentString, usize), SessionError> {
        let literal = match self.payloads.values.get(source) {
            Some(RetainedPayload::String { value, .. }) if prefix => {
                Literal::of_prefix_units(value.code_units())
            }
            Some(RetainedPayload::String { value, .. }) => {
                Literal::of_uri_units(value.code_units())
            }
            _ => None,
        };
        let Some(literal) = literal else {
            return self.copy_registered_field(c, source);
        };
        control_field_owner(&self.payloads, c, source)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::String { value, .. }) = self.payloads.values.get(source) else {
            return Err(latch_registry_row(c));
        };
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            std::mem::size_of_val(value.code_units()) as u64,
        )?;
        let origins = self
            .payloads
            .row_group(c, source)?
            .map(|origins| {
                self.evidence
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .copy_group(c, origins)
            })
            .transpose()?;
        self.register_literal_copy(c, literal, origins)
    }
    /// Copies the string of a non-element field of a record into a string of its own, with a
    /// copy of the field's group when it has one.
    pub(super) fn copy_record_field(
        &mut self,
        c: &mut ParseControl,
        field: FieldRef,
    ) -> Result<FieldString, SessionError> {
        let mut buffer = Utf16Buffer::new(c)?;
        let result = (|| {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let records = ConstructionView::Active(&self.construction);
            let Some(units) = field.units(records) else {
                return Err(latch_registry_row(c));
            };
            buffer.extend_units(units, c)?;
            self.copied_field_group(c, field)
        })();
        let group = match result {
            Ok(group) => group,
            Err(error) => {
                buffer.release(c)?;
                return Err(error);
            }
        };
        let frozen = buffer.freeze(c)?;
        Ok(FieldString {
            value: frozen,
            group,
        })
    }
    /// Copies a registered field without aliasing its native backing or ticket.
    pub(super) fn copy_registered_field(
        &mut self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<(DocumentString, usize), SessionError> {
        control_field_owner(&self.payloads, c, source)?;
        let mut buffer = Utf16Buffer::new(c)?;
        let result = (|| {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let Some(RetainedPayload::String { value, .. }) = self.payloads.values.get(source)
            else {
                return Err(latch_registry_row(c));
            };
            buffer.extend_units(value.code_units(), c)?;
            self.payloads
                .row_group(c, source)?
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
        location: FieldRef,
    ) -> Result<usize, SessionError> {
        // The string of a non-element field is the record's own: no registry row holds it.
        if location.is_record_string() {
            return Err(latch_registry_row(c));
        }
        // Deliberately a registry scan: every other location is looked up on
        // clone paths only, not per parsed node, and has no join.
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let Some(RetainedPayload::String {
        value, allocation, ..
    }) = payloads.values.get(slot)
    else {
        return Err(SessionError::WrongToken);
    };
    c.require_document_string(allocation, value)?;
    if let Some(maps) = payloads.row_group(c, slot)? {
        maps.require_disposal_owner(c)?;
    }
    Ok(())
}

pub(super) fn same_location(a: FieldRef, b: FieldRef) -> bool {
    match (a, b) {
        (FieldRef::ElementName(a), FieldRef::ElementName(b))
        | (FieldRef::Namespace(a), FieldRef::Namespace(b))
        | (FieldRef::Text(a), FieldRef::Text(b))
        | (FieldRef::Comment(a), FieldRef::Comment(b))
        | (FieldRef::ProcessingInstructionTarget(a), FieldRef::ProcessingInstructionTarget(b))
        | (FieldRef::ProcessingInstructionData(a), FieldRef::ProcessingInstructionData(b)) => {
            a == b
        }
        (FieldRef::AttributeName(a, x), FieldRef::AttributeName(b, y))
        | (FieldRef::AttributeValue(a, x), FieldRef::AttributeValue(b, y))
        | (FieldRef::AttributeNamespace(a, x), FieldRef::AttributeNamespace(b, y))
        | (FieldRef::AttributePrefix(a, x), FieldRef::AttributePrefix(b, y)) => a == b && x == y,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::effects::tests::{graph, named, request};

    #[test]
    fn a_clone_binding_to_a_missing_record_latches_the_registry_invariant() {
        let request = request("<body><script></script>");
        let (mut c, mut session, _proof, nodes) = graph(&request);
        let script = named(&nodes, "script", 0).index;
        let storage = session.active_mut().unwrap();
        let slot = storage.payloads.node_joins[script]
            .get(Join::Script)
            .unwrap();
        // A record past the retained ones: binding the row names no join cell,
        // so the controller-less `join_row` refuses and this caller, which has
        // the controller, must record the refusal as the first stop.
        let missing = storage.payloads.node_joins.len();
        let bindings = CloneBindings {
            script: Some(slot),
            ..CloneBindings::default()
        };
        let failure = storage.bind_clone_metadata(&mut c, missing, bindings);
        let invariant = HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::RegistryRow,
        );
        assert_eq!(failure.err(), Some(SessionError::Stop(invariant)));
        // The same stop the public failure path would report: `session_reason`
        // maps `SessionError::Stop(stop)` to `HtmlParseFailureReason::Stop(stop)`.
        assert_eq!(c.stop(), Some(invariant));
        // Put the row back on its record so the release check holds.
        let Some(RetainedPayload::Script { node, .. }) = storage.payloads.values.get_mut(slot)
        else {
            panic!("script row")
        };
        *node = Some(script);
        session.release(&mut c).unwrap();
        crate::construction::tests::storage_zero(&c);
    }
}

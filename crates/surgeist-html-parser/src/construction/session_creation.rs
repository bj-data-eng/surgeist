//! Creation of elements for the tree builder, including implied and profiled elements.

use super::*;
use surgeist_html::accounting::HtmlChargedError;
use surgeist_html::{
    bounded::values::{
        HtmlAttributeNameIndex, HtmlAttributeStorage, HtmlAttributeValidationError,
        HtmlNameIndexSlot,
    },
    construction::HtmlDocumentMode,
    elements::form::{HtmlNativeFormAssociationKind, HtmlTextareaFlags},
    values::{Attribute, OrderedAttributes},
    vocabulary::KnownHtmlElement,
};

pub(crate) enum RootKind {
    Document(HtmlDocumentMode),
    #[cfg(test)]
    Fragment,
}

pub(super) enum ImpliedHtmlName {
    Html,
    Head,
    Body,
    P,
    Colgroup,
    Tbody,
    Tr,
}

#[derive(Clone, Copy)]
pub(super) enum CreationProfile<'a, 'source> {
    Ordinary,
    Details(
        &'a crate::parser::tree_builder::ConstructionCapability<'source>,
        &'a crate::HtmlSourceRange,
    ),
    Input(
        &'a crate::parser::tree_builder::ConstructionCapability<'source>,
        &'a crate::HtmlSourceRange,
    ),
    Select(
        &'a crate::parser::tree_builder::ConstructionCapability<'source>,
        &'a crate::HtmlSourceRange,
    ),
}

#[derive(Clone, Copy)]
pub(super) enum OrderedCreation {
    Details {
        metadata: usize,
    },
    Input {
        input: usize,
        association: usize,
        source: usize,
    },
    Select {
        metadata: Option<usize>,
    },
}

pub(super) fn validate_parser_attributes(
    backing: Arc<Vec<Attribute>>,
    control: &mut ParseControl,
) -> Result<OrderedAttributes, (SessionError, Arc<Vec<Attribute>>)> {
    // The validation owner holds the only other count on `backing`; keeping one
    // here hands the allocation back on every refusal without recovering it
    // from the owner. On success the spare is simply dropped when this
    // function exits.
    let spare = Arc::clone(&backing);
    let count = backing.len();
    let storage = HtmlAttributeStorage::from_shared_attributes(backing);
    // A long list is validated through a table of its names, so that the work is linear in the
    // list; a short one compares every pair, which is cheaper than building the table.
    let mut slots = None;
    let mut validation = if count >= super::storage_holds::NAME_INDEX_MIN_ENTRIES
        && let Some(required) = HtmlAttributeNameIndex::slots_required(count)
    {
        let reserved = (|| {
            let (mut vector, mut ticket) = control.new_vector::<HtmlNameIndexSlot>()?;
            match control.reserve_vector(&mut vector, &mut ticket, required) {
                Ok(()) => Ok((vector, ticket)),
                Err(stop) => {
                    // The reserve may have changed the capacity before it stopped.
                    release_vector(control, vector, ticket)?;
                    Err(SessionError::from(stop))
                }
            }
        })();
        match reserved {
            Ok((vector, ticket)) => {
                slots = Some(ticket);
                storage.validate_indexed(HtmlAttributeNameIndex::new(vector))
            }
            Err(error) => return Err((error, spare)),
        }
    } else {
        storage.validate()
    };
    let validation_error = |error| match error {
        HtmlAttributeValidationError::Duplicate { .. } => SessionError::DuplicateAttribute,
        _ => SessionError::AttributeStorageUnavailable,
    };
    let outcome = (|| {
        while let Some(action) = validation.next_action().map_err(validation_error)? {
            action.advance(control).map_err(|error| match error {
                HtmlChargedError::Stopped(stop) => SessionError::from(stop),
                HtmlChargedError::Faulted(fault) => validation_error(fault),
            })?;
        }
        Ok(())
    })();
    // The table goes back whatever the outcome: finishing consumes the cursor that holds it, so
    // it is taken out first.
    let table = validation.take_index();
    let result = match outcome {
        Ok(()) => validation
            .finish()
            .map(|validated| validated.into_attributes())
            .map_err(|failure| validation_error(*failure.cause())),
        Err(error) => {
            drop(validation);
            Err(error)
        }
    };
    let released = match (table, slots) {
        (Some(table), Some(ticket)) => release_vector(control, table.into_storage(), ticket),
        _ => Ok(()),
    };
    match result {
        Ok(attributes) => released
            .map(|()| attributes)
            .map_err(|error| (error, spare)),
        Err(error) => Err((error, spare)),
    }
}

impl ParserArenaSession {
    /// Preexisting immutable input only; this does not construct parser foreign
    /// names or admit the input backing physically.
    #[cfg(test)]
    pub(crate) fn allocate_element(
        &mut self,
        control: &mut ParseControl,
        element: PreexistingElement,
    ) -> Result<ParserNode, SessionError> {
        self.active_mut()?.allocate_element(control, element)
    }
    pub(crate) fn allocate_html_tag(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
    ) -> Result<ParserNode, SessionError> {
        self.active_mut()?.allocate_html_tag(control, input)
    }
    /// Actual fresh ordinary parser creation context. The source form pointer
    /// remains driver-owned; exact admitted form-associated kinds own metadata.
    pub(crate) fn allocate_html_tag_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        intended_parent: &ParserNode,
        current_form: Option<&ParserNode>,
        context: CreationContext<'_>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_recorded_creation(
            control,
            input,
            intended_parent,
            current_form,
            CreationProfile::Ordinary,
            context,
        )
    }
    pub(crate) fn allocate_input_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        location: CreationLocation<'_>,
        token_source: &crate::HtmlSourceRange,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<ParserNode, SessionError> {
        capability.validate_session(control, self)?;
        self.allocate_recorded_creation(
            control,
            input,
            location.intended_parent,
            location.current_form,
            CreationProfile::Input(capability, token_source),
            location.context,
        )
    }
    pub(crate) fn allocate_details_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        location: CreationLocation<'_>,
        token_source: &crate::HtmlSourceRange,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<ParserNode, SessionError> {
        capability.validate_session(control, self)?;
        self.allocate_recorded_creation(
            control,
            input,
            location.intended_parent,
            location.current_form,
            CreationProfile::Details(capability, token_source),
            location.context,
        )
    }
    pub(crate) fn allocate_select_family_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        location: CreationLocation<'_>,
        token_source: &crate::HtmlSourceRange,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<ParserNode, SessionError> {
        capability.validate_session(control, self)?;
        self.allocate_recorded_creation(
            control,
            input,
            location.intended_parent,
            location.current_form,
            CreationProfile::Select(capability, token_source),
            location.context,
        )
    }
    /// Creates the element and records the start positions of its event
    /// handler content attributes.
    pub(super) fn allocate_recorded_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        intended_parent: &ParserNode,
        current_form: Option<&ParserNode>,
        profile: CreationProfile<'_, '_>,
        context: CreationContext<'_>,
    ) -> Result<ParserNode, SessionError> {
        let profile_installs_later = !matches!(profile, CreationProfile::Ordinary);
        let node = self.allocate_for_creation(
            control,
            input,
            intended_parent,
            current_form,
            profile,
            context,
        )?;
        // The elements that install their attributes one at a time have them
        // all once the creation is complete; the others recorded theirs when
        // they were allocated.
        if profile_installs_later {
            self.active_mut()?
                .record_event_handler_starts(control, node.index, 0)?;
        }
        Ok(node)
    }
    pub(super) fn allocate_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        intended_parent: &ParserNode,
        current_form: Option<&ParserNode>,
        profile: CreationProfile<'_, '_>,
        context: CreationContext<'_>,
    ) -> Result<ParserNode, SessionError> {
        self.validate_standalone_document(control)?;
        let parsing_template = self.qualify_creation_context(control, &context)?;
        let storage = self.active()?;
        let parent = storage.index(control, intended_parent)?;
        storage.container(control, parent)?;
        if self
            .active_mut()?
            .associated_document_parent(control, parent)?
        {
            return Err(control
                .stop_unsupported_standalone(HtmlParseUnsupportedState::ScriptDocument)
                .into());
        }
        let storage = self.active()?;
        let document = storage.node_document_ref(control, parent)?;
        let form = current_form
            .map(|form| storage.index(control, form))
            .transpose()?;
        if let Some(form) = form
            && !storage.html_local(control, form, "form")?
        {
            return Err(SessionError::WrongToken);
        }
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        SessionStorage::check_html_tag(control, tag)?;
        check_text_source(control, &tag.name)?;
        // No recoverable duplicate rejection may follow an initialized metadata slot. Compare
        // actual unique no-namespace token names.
        token_names::require_distinct_names(
            control,
            tag.attributes.as_slice(),
            |control, _, attribute| {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                control.require_source_range(&attribute.source)?;
                check_text_source(control, &attribute.name)?;
                check_text_source(control, &attribute.value)?;
                Ok(())
            },
        )?;
        let mut kind = surgeist_html::bounded::form::native_form_association_with(|candidate| {
            // These two metadata families are admitted only below, after the
            // actual profile/capability/origin checks. Avoid inapplicable reads.
            if matches!(
                candidate,
                KnownHtmlElement::Input | KnownHtmlElement::Select
            ) {
                return Ok(false);
            }
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            exact_ascii(control, Some(tag.name.value.text()), candidate.local_name())
        })?;
        let token_source = match profile {
            CreationProfile::Ordinary => None,
            CreationProfile::Details(capability, source)
            | CreationProfile::Input(capability, source)
            | CreationProfile::Select(capability, source) => {
                // Concrete borrowed capability, not a caller-supplied eligibility flag.
                capability.validate_session(control, self)?;
                Some(source)
            }
        };
        let details_domain = if let CreationProfile::Details(capability, _) = profile {
            if !exact_ascii(control, Some(tag.name.value.text()), "details")? {
                return Err(SessionError::WrongToken);
            }
            let closed = capability.details_notification_domain(control, self)?;
            for attr in tag.attributes.as_slice() {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if exact_ascii(control, Some(attr.name.value.text()), "open")? && !closed {
                    return Err(control
                        .stop_unsupported_standalone(HtmlParseUnsupportedState::DetailsNotification)
                        .into());
                }
            }
            Some(if closed {
                details::NotificationDomain::ClosedOwned
            } else {
                details::NotificationDomain::Unavailable
            })
        } else {
            None
        };
        let select_metadata = if matches!(profile, CreationProfile::Select(..)) {
            select::initial_metadata(control, tag)?
        } else {
            None
        };
        if matches!(profile, CreationProfile::Select(..))
            && exact_ascii(control, Some(tag.name.value.text()), "select")?
        {
            kind = Some(HtmlNativeFormAssociationKind::Select);
        }
        let storage = self.active_mut()?;
        if !matches!(profile, CreationProfile::Ordinary) {
            if matches!(profile, CreationProfile::Input(..))
                && !exact_ascii(control, Some(tag.name.value.text()), "input")?
            {
                return Err(SessionError::WrongToken);
            }
            let source = token_source.ok_or(SessionError::InputMetadataUnavailable)?;
            control.require_source_range(source)?;
            // Validate every original field against the actual token, including
            // empty-value attributes, before retaining any construction slot.
            check_formatting_tag(control, tag, source)?;
            if matches!(profile, CreationProfile::Input(..)) {
                validate_initial_input_origins(control, tag)?;
                kind = Some(HtmlNativeFormAssociationKind::Input);
            }
        }
        let mut explicit_form = false;
        if kind.is_some_and(HtmlNativeFormAssociationKind::listed) {
            for attribute in tag.attributes.as_slice() {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if exact_ascii(control, Some(attribute.name.value.text()), "form")? {
                    explicit_form = true;
                    break;
                }
            }
        }
        // §13.2.6.1's fragment-context-null guard is independent of a retained
        // nearest-form pointer used by insertion-mode recovery.
        let owner = if kind.is_some()
            && !explicit_form
            && !parsing_template
            && storage.fragment.is_none()
        {
            match form {
                Some(form)
                    if storage.ordinary_root(control, parent, None)?
                        == storage.ordinary_root(control, form, None)? =>
                {
                    Some(form)
                }
                _ => None,
            }
        } else {
            None
        };
        let value_source = if kind == Some(HtmlNativeFormAssociationKind::Input) {
            let mut selected = None;
            for attribute in tag.attributes.as_slice() {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if exact_ascii(control, Some(attribute.name.value.text()), "value")? {
                    selected = Some(&attribute.source);
                    break;
                }
            }
            selected
        } else {
            None
        };
        let input_sources = if let Some(token) =
            token_source.filter(|_| kind == Some(HtmlNativeFormAssociationKind::Input))
        {
            control.charge(
                HtmlParseWorkCategory::ProviderCall,
                if value_source.is_some() { 2 } else { 1 },
            )?;
            payload_write::<(crate::HtmlSourceRange, Option<crate::HtmlSourceRange>)>(control, 1)?;
            Some((token.clone(), value_source.cloned()))
        } else {
            None
        };
        let template = exact_ascii(control, Some(tag.name.value.text()), "template")?;
        let storage = self.active_mut()?;
        let mut prepared =
            storage.prepare_document_allocation(control, Some(document), template)?;
        let mut is_value = None;
        for attr in tag.attributes.as_slice() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if exact_ascii(control, Some(attr.name.value.text()), "is")? {
                is_value = Some(&attr.value);
                break;
            }
        }
        storage.prepare_creation_registry(
            control,
            parent,
            tag.name.value.text(),
            is_value,
            true,
            &mut prepared,
        )?;
        let link_slot = if exact_ascii(control, Some(tag.name.value.text()), "link")? {
            storage.document_slot(control, 0)?;
            let prepared = storage.prepare_payload_slot(control)?;
            payload_write::<(Option<usize>, Option<DocumentRef>)>(control, 1)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Link {
                        node: None,
                        parser_document: Some(DocumentRef::Owned(0)),
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let details_slot = if let Some(notification) = details_domain {
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Details {
                        node: None,
                        notification,
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let select_slot = if let Some(state) = select_metadata {
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::SelectFamily { node: None, state },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        // Reserve actual independent empty raw backing and initialized metadata
        // BEFORE lexical ownership transfer. A stopped unbound slot is cleanup-only.
        let textarea_slot = if kind == Some(HtmlNativeFormAssociationKind::Textarea) {
            let empty = literal_string(control, b"")?;
            let raw = storage.register_raw(control, empty)?;
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Textarea {
                        node: None,
                        state: TextareaState {
                            raw,
                            flags: HtmlTextareaFlags::default(),
                        },
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let output_slot = if kind == Some(HtmlNativeFormAssociationKind::Output) {
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Output {
                        node: None,
                        state: OutputState {
                            default_value_override: HtmlParseOutputOverride::Null,
                        },
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let input_slot = if kind == Some(HtmlNativeFormAssociationKind::Input) {
            let empty = literal_string(control, b"")?;
            let raw = storage.register_raw(control, empty)?;
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Input {
                        node: None,
                        raw,
                        complete: false,
                        state: InputState {
                            flags: surgeist_html::elements::input::HtmlInputFlags::fresh(),
                            cursor: 0,
                            selected_files_empty: true,
                            value_rewritten_during_creation: false,
                            email_value_count: None,
                            image: input_image::ImageState::NeverEntered,
                        },
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let source_slot = if let Some((token, value_attribute)) = input_sources {
            let entries = if value_attribute.is_some() { 2 } else { 1 };
            control.acquire(HtmlParseResource::SourceMapEntries, entries)?;
            let prepared = match storage.prepare_payload_slot(control) {
                Ok(prepared) => prepared,
                Err(error) => {
                    // The slot refusal is the error reported; a refused logical
                    // release has latched its own stop in the controller.
                    let _ = release_logical(
                        control,
                        &storage.control,
                        HtmlParseResource::SourceMapEntries,
                        entries,
                    );
                    return Err(error);
                }
            };
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::InputSource {
                        node: None,
                        token,
                        value_attribute,
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let reset_before_association = textarea_slot.is_some()
            || output_slot.is_some()
            || input_slot.is_some()
            || kind == Some(HtmlNativeFormAssociationKind::Select);
        let slot = if let Some(kind) = kind {
            storage.require(control)?;
            let prepared = storage.prepare_payload_slot_with(
                control,
                Some(KindList::FormAssociation),
                explicit_form,
            )?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::FormAssociation {
                        node: None,
                        state: FormAssociationState {
                            kind,
                            owner: if reset_before_association {
                                None
                            } else {
                                owner
                            },
                            parser_inserted: !reset_before_association && owner.is_some(),
                            evidence: if !reset_before_association && owner.is_some() {
                                HtmlParseFormEvidence::ParserAssociated
                            } else {
                                HtmlParseFormEvidence::Created
                            },
                        },
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let result = (|| {
            // The creation metadata is resolved before the tag leaves its
            // caller, so an impossible state keeps the token owned there.
            let creation = if let Some(metadata) = details_slot {
                Some(OrderedCreation::Details { metadata })
            } else if let Some(input) = input_slot {
                let (Some(association), Some(source)) = (slot, source_slot) else {
                    return Err(latch_registry_row(control));
                };
                Some(OrderedCreation::Input {
                    input,
                    association,
                    source,
                })
            } else if matches!(profile, CreationProfile::Select(..)) {
                Some(OrderedCreation::Select {
                    metadata: select_slot,
                })
            } else {
                None
            };
            let Tag {
                name, attributes, ..
            } = take_payload(input, control)?;
            let node = match creation {
                Some(creation) => storage
                    .allocate_ordered_payload(control, name, attributes, prepared, creation)?,
                None => storage.allocate_html_payload_prepared(
                    control,
                    name.value,
                    Some(name.origins),
                    attributes,
                    prepared,
                )?,
            };
            storage.mark_document_parser_style(control, node.index)?;
            if let Some(link) = link_slot {
                payload_write::<Option<usize>>(control, 1)?;
                let Some(RetainedPayload::Link { node: bound, .. }) =
                    storage.payloads.values.get_mut(link)
                else {
                    return Err(latch_registry_row(control));
                };
                *bound = Some(node.index);
            }
            if let Some(textarea) = textarea_slot {
                control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                if storage
                    .construction
                    .node_at(node.index)
                    .ok_or_else(|| latch_registry_row(control))?
                    .child_count()
                    != 0
                {
                    return Err(SessionError::WrongToken);
                }
                // Actual fresh node has no children. Its already admitted empty
                // raw is therefore exactly child text; reset precedes association.
                payload_write::<Option<usize>>(control, 1)?;
                payload_write::<Option<usize>>(control, 1)?;
                payload_write::<TextareaState>(control, 1)?;
                let Some(RetainedPayload::Textarea { node: bound, state }) =
                    storage.payloads.values.get_mut(textarea)
                else {
                    return Err(latch_registry_row(control));
                };
                *bound = Some(node.index);
                state.flags.reset();
                storage
                    .payloads
                    .join_textarea(node.index, textarea)
                    .map_err(|_| latch_registry_row(control))?;
                #[cfg(test)]
                if let Some(cancellation) = storage.cancel_after_textarea_reset.take() {
                    cancellation.cancel();
                }
            }
            if let Some(output) = output_slot {
                // Syntax3537 reset: actual freshly allocated output has a null
                // override and empty descendants. DOM1593/716 therefore replaces
                // all with null: zero removed/added children, no mutation record.
                control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                if storage
                    .construction
                    .node_at(node.index)
                    .ok_or_else(|| latch_registry_row(control))?
                    .child_count()
                    != 0
                {
                    return Err(SessionError::WrongToken);
                }
                payload_write::<Option<usize>>(control, 1)?;
                payload_write::<OutputState>(control, 1)?;
                let Some(RetainedPayload::Output { node: bound, state }) =
                    storage.payloads.values.get_mut(output)
                else {
                    return Err(latch_registry_row(control));
                };
                *bound = Some(node.index);
                state.default_value_override = HtmlParseOutputOverride::Null;
                #[cfg(test)]
                if let Some(cancellation) = storage.cancel_after_output_reset.take() {
                    cancellation.cancel();
                }
            }
            #[cfg(test)]
            if let Some(lost) = storage.lose_association_row.as_mut()
                && let Some(slot) = slot
                && let Some(row) = storage.payloads.values.get_mut(slot)
            {
                *lost = Some((slot, mem::replace(row, RetainedPayload::Retired)));
            }
            if let Some(slot) = slot {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                payload_write::<Option<usize>>(control, 1)?;
                let Some(RetainedPayload::FormAssociation { node: bound, .. }) =
                    storage.payloads.values.get_mut(slot)
                else {
                    return Err(SessionError::FormAssociationMetadataUnavailable);
                };
                *bound = Some(node.index);
                storage
                    .payloads
                    .join_row(slot)
                    .map_err(|_| latch_registry_row(control))?;
                if reset_before_association {
                    payload_write::<FormAssociationState>(control, 1)?;
                    let Some(RetainedPayload::FormAssociation { state, .. }) =
                        storage.payloads.values.get_mut(slot)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    state.owner = owner;
                    state.parser_inserted = owner.is_some();
                    state.evidence = if owner.is_some() {
                        HtmlParseFormEvidence::ParserAssociated
                    } else {
                        HtmlParseFormEvidence::Created
                    };
                }
                if reset_before_association && owner.is_some() && input_slot.is_some() {
                    storage.normalize_radio(control, node.index, None)?;
                }
            }
            Ok(node)
        })();
        match result {
            Err(error)
                if slot.is_some()
                    || link_slot.is_some()
                    || matches!(
                        profile,
                        CreationProfile::Select(..) | CreationProfile::Details(..)
                    ) =>
            {
                Err(latch_creation_fault(control, error))
            }
            other => other,
        }
    }
    pub(crate) fn add_attributes_if_missing(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        let before = storage.attribute_count(control, target)?;
        storage.add_attributes_if_missing(control, target, input)?;
        storage.record_merged_event_handlers(control, target, before)
    }
    pub(crate) fn add_body_attributes_if_missing(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        let before = storage.attribute_count(control, target)?;
        if let Some(prepared) = storage.prepare_named_attribute_merge(
            control,
            target,
            input,
            AttributeMergeName::Body,
        )? {
            storage.finish_attribute_merge(control, prepared)?;
        }
        storage.record_merged_event_handlers(control, target, before)
    }
    pub(crate) fn allocate_implicit_html(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, None, ImpliedHtmlName::Html)
    }
    pub(crate) fn allocate_implicit_head(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Head)
    }
    pub(crate) fn allocate_implicit_body(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Body)
    }
    pub(crate) fn allocate_implicit_p(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::P)
    }
    pub(crate) fn allocate_implicit_colgroup(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Colgroup)
    }
    pub(crate) fn allocate_implicit_tbody(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Tbody)
    }
    pub(crate) fn allocate_implicit_tr(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Tr)
    }
}

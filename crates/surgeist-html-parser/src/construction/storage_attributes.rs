//! Attribute merging, transfer and editing of installed element attributes.

use super::*;
use surgeist_html::{
    bounded::construction::HtmlConstructionAttributeEditMode,
    construction::HtmlNodeKind,
    values::{Attribute, Namespace, OrderedAttributes, QualifiedName},
};

#[derive(Clone, Copy)]
pub(super) enum AttributeMergeName {
    Html,
    Body,
}

impl AttributeMergeName {
    fn local(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Body => "body",
        }
    }
}

pub(super) struct PreparedAttributeMerge {
    pub(super) target: usize,
    pub(super) container: usize,
    pub(super) stage: usize,
    pub(super) start: usize,
    pub(super) count: usize,
    /// Rows registered before the stage row and bound to it; the stagers that
    /// register their strings afterwards leave this empty.
    pub(super) earlier: [Option<usize>; 4],
}

impl SessionStorage {
    /// Same parser-owned HTML root, real slots and original lexical origins.
    /// Imported/native attribute backing requires a later admitted importer.
    pub(super) fn add_attributes_if_missing(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        if let Some(prepared) = self.prepare_attribute_merge(control, target, input)? {
            self.finish_attribute_merge(control, prepared)?;
        }
        Ok(())
    }
    /// Real borrowed-input admission and owned staging shared by the complete
    /// operation and its bounded commit. Rejection before take preserves Tag.
    pub(super) fn prepare_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<Option<PreparedAttributeMerge>, SessionError> {
        self.prepare_named_attribute_merge(control, target, input, AttributeMergeName::Html)
    }
    pub(super) fn prepare_named_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
        name: AttributeMergeName,
    ) -> Result<Option<PreparedAttributeMerge>, SessionError> {
        let target = self.index(control, target)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(target)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        if !exact_ascii(
            control,
            element.name().namespace().uri(),
            "http://www.w3.org/1999/xhtml",
        )? || !exact_ascii(control, Some(element.name().local_name()), name.local())?
        {
            return Err(SessionError::WrongToken);
        }
        let container = self.element_attribute_container(control, target)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        if Arc::weak_count(element.attributes().shared_attributes()) != 1 {
            return Err(SessionError::AliasedPayload);
        }
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if tag.kind != TagKind::Start {
            return Err(SessionError::WrongToken);
        }
        Self::check_text(control, &mut tag.name)?;
        if !exact_ascii(control, Some(tag.name.value.text()), name.local())? {
            return Err(SessionError::WrongToken);
        }
        check_text_source(control, &tag.name)?;
        tag.attributes.require_owner(control)?;
        for index in 0..tag.attributes.len() {
            let Some(a) = tag.attributes.get_mut(index, control)? else {
                return Err(latch_registry_row(control));
            };
            Self::check_text(control, &mut a.name)?;
            Self::check_text(control, &mut a.value)?;
            lowercase_token_name(control, a.name.value.text())?;
            check_text_source(control, &a.name)?;
            check_text_source(control, &a.value)?;
            control.require_source_range(&a.source)?;
        }
        let entries = element
            .attributes()
            .len()
            .saturating_add(tag.attributes.len());
        if entries >= construction_owner::NAME_INDEX_MIN_ENTRIES {
            self.construction.provide_name_index(control, entries)?;
        }
        self.begin_attribute_edit(
            control,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge {
                candidate_count: tag.attributes.len(),
            },
        )?;
        let planned = (|| {
            for index in 0..tag.attributes.len() {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                self.construction.plan_attribute_candidate(
                    control,
                    index,
                    tag.attributes.as_slice(),
                )?;
            }
            self.construction
                .attribute_edit(control)?
                .selected_count()
                .map_err(|error| construction_owner::construction_error(control, error))
        })();
        let count = match planned {
            Ok(count) => count,
            Err(error) => {
                self.construction.abort_attributes(control)?;
                return Err(error);
            }
        };
        // Final borrowed-input admission leaves the complete Tag caller-owned.
        if let Err(error) = payload_write::<Option<Tag>>(control, 1) {
            self.construction.abort_attributes(control)?;
            return Err(error);
        }
        let Some(Tag {
            name,
            mut attributes,
            ..
        }) = input.take()
        else {
            // Nothing was taken: only the edit begun above needs ending.
            self.construction.abort_attributes(control)?;
            return Err(SessionError::Consumed);
        };
        let result = (|| {
            name.release(control)?;
            let (values, allocation) = control.new_vector()?;
            let prepared = match self.prepare_payload_slot(control) {
                Ok(prepared) => prepared,
                Err(error) => {
                    release_vector(control, values, allocation)?;
                    return Err(error);
                }
            };
            let stage = self
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Attributes(Some(AttributeStorage::Building {
                        values,
                        allocation,
                        original_len: count,
                        reversed_pairs: 0,
                    })),
                )
                .map_err(|_| latch_registry_row(control))?;
            let mut remaining = count;
            while !attributes.as_slice().is_empty() {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let decision_index = attributes
                    .len()
                    .checked_sub(1)
                    .ok_or_else(|| latch_registry_row(control))?;
                let keep = self
                    .construction
                    .attribute_edit(control)?
                    .decision(decision_index)
                    .map_err(|error| construction_owner::construction_error(control, error))?;
                let Some(attribute) = attributes.try_pop(control)? else {
                    return Err(latch_registry_row(control));
                };
                if !keep {
                    let retained = (|| {
                        if let Some(evidence) = self.evidence.as_mut() {
                            evidence
                                .lexical_attribute_ignored(control, attribute.lexical.as_ref())?;
                        }
                        Ok::<_, SessionError>(())
                    })();
                    if let Err(error) = retained {
                        attribute.name.release(control)?;
                        attribute.value.release(control)?;
                        return Err(error);
                    }
                    attribute.name.release(control)?;
                    attribute.value.release(control)?;
                    continue;
                }
                let (local, name_ticket) = match self.register_text(control, attribute.name) {
                    Ok(value) => value,
                    Err(error) => {
                        attribute.value.release(control)?;
                        return Err(error);
                    }
                };
                let (value, value_ticket) = self.register_text(control, attribute.value)?;
                let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    ..
                }))) = self.payloads.values.get_mut(stage)
                else {
                    return Err(latch_registry_row(control));
                };
                control.reserve_vector_geometric_toward(values, allocation, 1, count)?;
                control.charge(
                    HtmlParseWorkCategory::CopyByte,
                    (const { mem::size_of::<Attribute>() + 2 * mem::size_of::<StringLocation>() })
                        as u64,
                )?;
                control.charge(HtmlParseWorkCategory::RecordWrite, 3)?;
                control.acquire(HtmlParseResource::ManagedAttributes, 1)?;
                values.push(Attribute::new(
                    QualifiedName::new(Namespace::none(), None, local),
                    value,
                ));
                remaining = remaining
                    .checked_sub(1)
                    .ok_or_else(|| latch_registry_row(control))?;
                self.bind_string(name_ticket, StringLocation::AttributeName(stage, remaining))
                    .map_err(|_| latch_registry_row(control))?;
                self.bind_string(
                    value_ticket,
                    StringLocation::AttributeValue(stage, remaining),
                )
                .map_err(|_| latch_registry_row(control))?;
            }
            if count == 0 {
                self.construction.finish_attributes(control)?;
                self.construction.retire_name_index(control)?;
                return Ok(None);
            }
            // The actual stage remains reversed: last-pop yields source order
            // without a variable shift or a second attribute representation.
            self.edit_attributes(control, target, container, stage, count, false)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let old_len = element_record(&self.construction, control, target)?
                .attributes()
                .len();
            self.begin_attribute_transfer(control, stage, container, old_len)?;
            payload_write::<PreparedAttributeMerge>(control, 1)?;
            Ok(Some(PreparedAttributeMerge {
                target,
                container,
                stage,
                start: old_len,
                count,
                earlier: [None; 4],
            }))
        })();
        let cleanup = release_token_attributes(attributes, control);
        result.and_then(|prepared| cleanup.map(|()| prepared))
    }
    pub(super) fn finish_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        prepared: PreparedAttributeMerge,
    ) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        let PreparedAttributeMerge {
            target,
            container,
            stage,
            start,
            count,
            earlier,
        } = prepared;
        for _ in 0..count {
            self.edit_attributes(control, target, container, stage, 1, true)?;
            if self.effects.form.is_some() {
                self.drain_form_effects(control)?;
            }
        }
        // The strings staged for this merge are the registry rows from the stage on
        // and the few rows registered before it that the stager names.
        #[cfg(test)]
        for (index, row) in self.payloads.values[..stage].iter().enumerate() {
            if let RetainedPayload::String { location, .. } = row {
                assert!(
                    !matches!(
                        location,
                        StringLocation::AttributeName(owner, _)
                            | StringLocation::AttributeValue(owner, _)
                            | StringLocation::AttributeNamespace(owner, _)
                            | StringLocation::AttributePrefix(owner, _)
                            if *owner == stage
                    ) || earlier.contains(&Some(index)),
                    "row {index} before stage {stage} names the stage unlisted"
                );
            }
        }
        for index in earlier
            .into_iter()
            .flatten()
            .chain(stage..self.payloads.values.len())
        {
            self.rebind_attribute_location(control, index, stage, container, start)?;
        }
        // Clearing the redirect is safe only after every original location was
        // rebound. A stop above retains the exact transfer phase.
        payload_write::<Option<AttributeStorage>>(control, 2)?;
        payload_write::<usize>(control, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(stage) else {
            return Err(latch_registry_row(control));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Transferring {
            values, allocation, ..
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(control));
        };
        *storage = Some(AttributeStorage::Building {
            values,
            allocation,
            original_len: 0,
            reversed_pairs: 0,
        });
        if self.construction.has_attribute_edit() {
            self.construction.finish_attributes(control)?;
        }
        // Planning is over: the table of names, if the merge borrowed one, is no longer needed.
        self.construction.retire_name_index(control)?;
        let end = start
            .checked_add(count)
            .ok_or_else(|| latch_registry_row(control))?;
        for position in start..end {
            self.initial_nonce_attribute(control, target, position)?;
            self.initial_meta_attribute(control, target, position)?;
            self.association_attribute_hook(control, target, position)?;
            self.initial_dialog_attribute(control, target, position)?;
        }
        if base::is_base(
            self.construction
                .node_at(target)
                .ok_or_else(|| latch_registry_row(control))?,
        ) {
            // Only an actual appended no-namespace href changes the pinned
            // first-Base href trigger. Other fields share this merge owner.
            let mut href_changed = false;
            for position in start..end {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let attribute = element_attribute(&self.construction, control, target, position)?;
                if attribute.name().namespace().uri().is_none()
                    && exact_ascii(control, Some(attribute.name().local_name()), "href")?
                {
                    href_changed = true;
                    break;
                }
            }
            self.base_change(control, target, None, href_changed, None)?;
        }
        Ok(())
    }

    pub(super) fn begin_attribute_transfer(
        &mut self,
        control: &mut ParseControl,
        stage: usize,
        destination: usize,
        start: usize,
    ) -> Result<(), SessionError> {
        payload_write::<Option<AttributeStorage>>(control, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(stage) else {
            return Err(latch_registry_row(control));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Building {
            values,
            allocation,
            original_len,
            reversed_pairs,
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(control));
        };
        // Last-pop consumes the unchanged reverse stage; a stage that was
        // already reversed is handed back to the row untouched.
        if reversed_pairs != 0 {
            *storage = Some(AttributeStorage::Building {
                values,
                allocation,
                original_len,
                reversed_pairs,
            });
            return Err(latch_registry_row(control));
        }
        *storage = Some(AttributeStorage::Transferring {
            values,
            allocation,
            destination,
            start,
            original_len,
            moved: 0,
        });
        Ok(())
    }

    pub(super) fn rebind_attribute_location(
        &mut self,
        control: &mut ParseControl,
        index: usize,
        stage: usize,
        destination: usize,
        start: usize,
    ) -> Result<(), SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        #[cfg(test)]
        control.observe_registry_iteration();
        if let Some(RetainedPayload::String { location, .. }) = self.payloads.values.get_mut(index)
        {
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                (const { 2 * mem::size_of::<StringLocation>() }) as u64,
            )?;
            control.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
            #[cfg(test)]
            control.observe_registry_location_write(mem::size_of::<StringLocation>());
            *location = match *location {
                StringLocation::AttributeName(owner, index) if owner == stage => {
                    StringLocation::AttributeName(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control.stop_representation(HtmlParseResource::ManagedAttributes)
                        })?,
                    )
                }
                StringLocation::AttributeValue(owner, index) if owner == stage => {
                    StringLocation::AttributeValue(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control.stop_representation(HtmlParseResource::ManagedAttributes)
                        })?,
                    )
                }
                StringLocation::AttributeNamespace(owner, index) if owner == stage => {
                    StringLocation::AttributeNamespace(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control.stop_representation(HtmlParseResource::ManagedAttributes)
                        })?,
                    )
                }
                StringLocation::AttributePrefix(owner, index) if owner == stage => {
                    StringLocation::AttributePrefix(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control.stop_representation(HtmlParseResource::ManagedAttributes)
                        })?,
                    )
                }
                other => other,
            };
        }
        Ok(())
    }

    pub(super) fn prepare_id_attribute_effects(
        &self,
        control: &mut ParseControl,
        target: usize,
        stage: usize,
    ) -> Result<Option<FormEffects>, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
            values, ..
        }))) = self.payloads.values.get(stage)
        else {
            return Err(latch_registry_row(control));
        };
        let attribute = values.last().ok_or_else(|| latch_registry_row(control))?;
        if attribute.name().namespace().uri().is_some()
            || !exact_ascii(control, Some(attribute.name().local_name()), "id")?
        {
            return Ok(None);
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if attribute.value().is_empty() {
            return Ok(None);
        }
        #[cfg(test)]
        self.assert_explicit_forms_exact();
        for &index in self.payloads.explicit_form_list() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::FormAssociation {
                node: Some(node),
                state,
            }) = self.payloads.values.get(index)
                && state.kind.listed()
                && self.form_attribute(control, *node, "form")?.is_some()
            {
                return Ok(Some(FormEffects {
                    root: target,
                    old_parent: None,
                    new_parent: None,
                    stage: FormEffectStage::IdChange,
                    insert: false,
                    cause: InsertionCause::NonParser,
                    next: Some(target),
                    pass: FormEffectPass::Local,
                    href_base: base::HrefBaseAhead::Unscanned,
                }));
            }
        }
        Ok(None)
    }

    /// Every attribute edit begins here, so the creation-time registry hint for
    /// the record's attribute strings is dropped before the first change.
    pub(super) fn begin_attribute_edit(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        mode: HtmlConstructionAttributeEditMode,
    ) -> Result<(), SessionError> {
        self.payloads.forget_attribute_strings(target);
        self.construction.begin_attributes(control, target, mode)
    }
    /// Removes/restores only the registry's known strong owner. Both restoration
    /// paths are admitted before removal; no new wrapper allocation is needed.
    pub(super) fn edit_attributes(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        container: usize,
        stage: usize,
        count: usize,
        append: bool,
    ) -> Result<(), SessionError> {
        self.payloads.forget_attribute_strings(target);
        let form_effects = if append {
            self.prepare_id_attribute_effects(control, target, stage)?
        } else {
            None
        };
        if form_effects.is_some() {
            payload_write::<Option<FormEffects>>(control, 1)?;
        }
        control.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
        payload_write::<Option<AttributeStorage>>(control, 2)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<OrderedAttributes>() as u64,
        )?;
        if append {
            // No fallible operation may intervene between the actual slot move
            // and this fixed progress write; cleanup resolves it immediately.
            payload_write::<usize>(control, 1)?;
        }
        // Everything that can refuse is checked before ownership leaves the
        // container; the target stays an element while its attributes change.
        if !matches!(
            self.construction
                .storage(control)?
                .node_at(target)
                .map(|node| node.kind()),
            Some(HtmlNodeKind::Element(_))
        ) || stage > self.payloads.values.len()
        {
            return Err(latch_registry_row(control));
        }
        let (earlier, later) = self.payloads.values.split_at_mut(stage);
        let Some(RetainedPayload::Attributes(storage)) = earlier.get_mut(container) else {
            return Err(latch_registry_row(control));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Frozen {
            values,
            mut allocation,
            mut shared,
            element,
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(control));
        };
        drop(values);
        let was_named = self.construction.has_attribute_edit();
        let started = if was_named {
            Ok(())
        } else {
            self.construction.begin_attributes(
                control,
                target,
                HtmlConstructionAttributeEditMode::Slots,
            )
        };
        let result = (|| {
            started?;
            let group = self.construction.attribute_edit(control)?;
            let slot = if append {
                let Some(RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
                    values,
                    allocation: incoming_allocation,
                    ..
                }))) = later.first_mut()
                else {
                    return Err(latch_registry_row(control));
                };
                control.require_incoming_attribute_vector(values, incoming_allocation)?;
                group.append(values)
            } else {
                group.reserve(count)
            }
            .map_err(|error| construction_owner::construction_error(control, error))?;
            let ready = prepare_attribute_slot(control, slot, &mut shared, &mut allocation)?;
            #[cfg(test)]
            if append {
                control.observe_construction_attribute_append();
            }
            ready.commit().map_err(|refusal| {
                construction_owner::construction_error(control, refusal.into_parts().1)
            })?;
            Ok::<_, SessionError>(())
        })();
        // Restore the sole ticket before creating the registry's known alias.
        let Some(HtmlNodeKind::Element(record)) = self
            .construction
            .storage(control)?
            .node_at(target)
            .map(|node| node.kind())
        else {
            return Err(latch_registry_row(control));
        };
        control.restore_attribute_weak(record.attributes().shared_attributes(), &mut shared);
        let values = record.attributes().clone();
        *storage = Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        });
        let mut result = result;
        if append && result.is_ok() {
            match later.first_mut() {
                Some(RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
                    moved,
                    ..
                }))) => match moved.checked_add(1) {
                    Some(next) => {
                        *moved = next;
                        if form_effects.is_some() {
                            self.effects.form = form_effects;
                        }
                    }
                    None => result = Err(latch_registry_row(control)),
                },
                _ => result = Err(latch_registry_row(control)),
            }
        }
        if result.is_err() {
            self.construction.abort_attributes(control)?;
        } else if !was_named {
            self.construction.finish_attributes(control)?;
        }
        result
    }
    pub(super) fn reverse_attributes(
        &mut self,
        control: &mut ParseControl,
        container: usize,
    ) -> Result<(), SessionError> {
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building { values, .. }))) =
            self.payloads.values.get(container)
        else {
            return Err(latch_registry_row(control));
        };
        let len = values.len();
        for left in 0..len / 2 {
            let right = mirrored_position(len, left).ok_or_else(|| latch_registry_row(control))?;
            control.admit_atomic_layout(const { mem::size_of::<Attribute>() * 2 })?;
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                (const { mem::size_of::<Attribute>() * 2 }) as u64,
            )?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.charge(HtmlParseWorkCategory::RecordWrite, 3)?;
            let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                values,
                reversed_pairs,
                ..
            }))) = self.payloads.values.get_mut(container)
            else {
                return Err(latch_registry_row(control));
            };
            values.swap(left, right);
            *reversed_pairs = reversed_pairs
                .checked_add(1)
                .ok_or_else(|| latch_registry_row(control))?;
        }
        Ok(())
    }
}

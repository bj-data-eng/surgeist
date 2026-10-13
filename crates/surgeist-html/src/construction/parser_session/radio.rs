//! HTML877d #radio-button-group over the actual canonical input records.
//! Each source trigger drains before its caller advances. Terminal prefixes retain
//! progress in ConstructionEffects; no public resume, peer list or group cache.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct RadioProgress {
    subject: usize,
    root: usize,
    detached: Option<usize>,
    next: Option<usize>,
}

impl SessionStorage {
    pub(super) fn radio_input(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<(usize, InputState)>, SessionError> {
        for slot in 0..self.payloads.values.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::Input {
                node: Some(owner),
                state,
                ..
            } = &self.payloads.values[slot]
                && *owner == node
            {
                c.charge(WorkCategory::CopyByte, mem::size_of::<InputState>() as u64)?;
                return Ok(Some((slot, *state)));
            }
        }
        Ok(None)
    }
    pub(super) fn normalize_radio(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<(), SessionError> {
        c.require_identity(&self.control)?;
        let Some((_, state)) = self.radio_input(c, node)? else {
            return Ok(());
        };
        if !state
            .flags
            .radio_normalization_required(crate::HtmlRadioTrigger::Checkedness)
        {
            return Ok(());
        }
        let Some(name) = self.form_attribute(c, node, "name")? else {
            return Ok(());
        };
        c.charge(WorkCategory::RecordVisit, 1)?;
        if name.is_empty() {
            return Ok(());
        }
        let root = self.ordinary_root(c, node, detached)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if root == node && self.records[node].children.is_empty() {
            return Ok(());
        }
        payload_write::<Option<RadioProgress>>(c, 1)?;
        if self.effects.radio.is_some() {
            return Err(c
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into());
        }
        self.effects.radio = Some(RadioProgress {
            subject: node,
            root,
            detached,
            next: Some(root),
        });
        let result = (|| {
            loop {
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.admit_atomic_layout(mem::size_of::<Option<RadioProgress>>())?;
                c.charge(
                    WorkCategory::CopyByte,
                    mem::size_of::<Option<RadioProgress>>() as u64,
                )?;
                let Some(progress) = self.effects.radio else {
                    return Ok(());
                };
                let Some(peer) = progress.next else {
                    payload_write::<Option<RadioProgress>>(c, 1)?;
                    self.effects.radio = None;
                    return Ok(());
                };
                #[cfg(test)]
                let mut peer_written = false;
                if peer != progress.subject && self.html_local(c, peer, "input")? {
                    let actual = self.radio_input(c, peer)?;
                    if let Some((slot, mut state)) = actual {
                        if state.flags.kind() == crate::HtmlInputStringKind::Radio
                            && let Some(peer_name) = self.form_attribute(c, peer, "name")?
                        {
                            let subject_name = self
                                .form_attribute(c, progress.subject, "name")?
                                .ok_or(SessionError::InputMetadataUnavailable)?;
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            if !peer_name.is_empty()
                                && exact_string_equal(c, subject_name, peer_name)?
                            {
                                let (_, subject_owner) = find_form_association_state(
                                    &self.payloads,
                                    c,
                                    progress.subject,
                                )?
                                .ok_or(SessionError::FormAssociationMetadataUnavailable)?;
                                let (_, peer_owner) =
                                    find_form_association_state(&self.payloads, c, peer)?
                                        .ok_or(SessionError::FormAssociationMetadataUnavailable)?;
                                c.charge(WorkCategory::ProviderCall, 1)?;
                                if subject_owner.owner == peer_owner.owner {
                                    state.flags = state.flags.radio_peer_unchecked();
                                    self.write_input_state(c, slot, state)?;
                                    #[cfg(test)]
                                    {
                                        peer_written = true;
                                    }
                                }
                            }
                        }
                    } else {
                        let value = self.form_attribute(c, peer, "type")?;
                        let kind = match value {
                            Some(value) => input_type_state(c, value)?,
                            None => crate::attributes::input_type_default(),
                        };
                        // Imported native Radio is not a canonical mutable input.
                        // Only a potentially matching peer requires unavailable state.
                        if kind == crate::HtmlInputType::Radio
                            && let Some(peer_name) = self.form_attribute(c, peer, "name")?
                        {
                            let subject_name = self
                                .form_attribute(c, progress.subject, "name")?
                                .ok_or(SessionError::InputMetadataUnavailable)?;
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            if !peer_name.is_empty()
                                && exact_string_equal(c, subject_name, peer_name)?
                            {
                                return Err(SessionError::InputMetadataUnavailable);
                            }
                        }
                    }
                }
                let next = self.next_view_node(c, progress.root, peer, progress.detached)?;
                payload_write::<Option<RadioProgress>>(c, 1)?;
                self.effects.radio = Some(RadioProgress { next, ..progress });
                // Test scheduling only: original request token after the actual
                // peer write AND retained cursor commit; the next real poll stops.
                #[cfg(test)]
                if peer_written && let Some(token) = self.cancel_after_radio_peer.take() {
                    token.cancel();
                }
            }
        })();
        match result {
            Ok(()) => Ok(()),
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(error) => {
                c.stop_unsupported_standalone(StandaloneUnsupported::Provenance);
                Err(error)
            }
        }
    }
    pub(super) fn radio_insertion(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        let Some((_, state)) = self.radio_input(c, node)? else {
            return Ok(());
        };
        if state
            .flags
            .radio_normalization_required(crate::HtmlRadioTrigger::Connected)
            && self.shadow_including_connected(c, node, None)?
        {
            self.normalize_radio(c, node, None)?;
        }
        Ok(())
    }
}

impl ParserArenaSession {
    /// Native checked setter: no input/change event, same actual flags and group.
    pub(crate) fn set_input_checked(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        checked: bool,
    ) -> Result<(), SessionError> {
        self.active()?.require(c)?;
        let storage = self.active_mut()?;
        let node = storage.index(c, node)?;
        let (slot, mut state) = storage
            .radio_input(c, node)?
            .ok_or(SessionError::InputMetadataUnavailable)?;
        let previous = storage.begin_attribute_change(c, node, None)?;
        let result: Result<(), SessionError> = (|| {
            state.flags = state.flags.checked_setter(checked);
            storage.write_input_state(c, slot, state)?;
            if state
                .flags
                .radio_normalization_required(crate::HtmlRadioTrigger::Checkedness)
            {
                storage.normalize_radio(c, node, None)?;
            }
            Ok(())
        })();
        if c.stop().is_none() {
            storage.end_node_copy(c, previous.as_ref())?;
        }
        result?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) enum InputGroupAttribute {
    Name,
    Checked,
}
impl ParserArenaSession {
    pub(crate) fn set_input_attribute(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        input: &mut Option<crate::parser::tokenizer::token::Attribute>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        self.set_managed_attribute(c, node, input, capability, mutation::ManagedFamily::Input)
    }
    pub(crate) fn remove_input_attribute(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        attribute: InputGroupAttribute,
    ) -> Result<(), SessionError> {
        self.remove_managed_attribute(
            c,
            node,
            match attribute {
                InputGroupAttribute::Name => "name",
                InputGroupAttribute::Checked => "checked",
            },
            mutation::ManagedFamily::Input,
        )
    }
}

//! HTML877d #radio-button-group over the actual canonical input records.
//! Each source trigger drains before its caller advances. Terminal prefixes retain
//! progress in ConstructionEffects; no public resume, peer list or group cache.
//!
//! The walk visits every node of the checked radio's tree for each insertion, as the
//! group definition is written, so n checked radios of one name cost record visits
//! quadratic in n: 7.1 million at 500, 22.7 million at 1,000, 79.5 million at 2,000 and
//! 294.9 million at 4,000. It stays a walk, with the same work in the same order. A group
//! is the radios of one name, one tree and one form owner, so an index of the checked
//! ones would have to follow every move of a radio between trees, every change of its
//! form owner, name or type, and every clone, and a stopped parse reports the first
//! charge it reached, which a walk fixes at a record and an index would not.
//! (docs/reference/html-operations.md, "group-walk-cost")
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
        if let Some(slot) = self.payloads.joined(c, node, Join::Input)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::Input {
                node: Some(owner),
                state,
                ..
            }) = self.payloads.values.get(slot)
                && *owner == node
            {
                c.charge(
                    HtmlParseWorkCategory::CopyByte,
                    mem::size_of::<InputState>() as u64,
                )?;
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
        if !state.flags.radio_normalization_required(
            surgeist_html::elements::input::HtmlRadioTrigger::Checkedness,
        ) {
            return Ok(());
        }
        let Some(name) = self.form_attribute(c, node, "name")? else {
            return Ok(());
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if name.is_empty() {
            return Ok(());
        }
        let root = self.ordinary_root(c, node, detached)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if root == node
            && self
                .construction
                .node_at(node)
                .ok_or_else(|| latch_registry_row(c))?
                .first_child()
                .is_none()
        {
            return Ok(());
        }
        payload_write::<Option<RadioProgress>>(c, 1)?;
        if self.effects.radio.is_some() {
            return Err(c
                .fail_invariant(crate::parser::control::HtmlParseInvariant::PendingFormEffects)
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
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                c.admit_atomic_layout(mem::size_of::<Option<RadioProgress>>())?;
                c.charge(
                    HtmlParseWorkCategory::CopyByte,
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
                        if state.flags.input_type()
                            == surgeist_html::attributes::HtmlInputType::Radio
                            && let Some(peer_name) = self.form_attribute(c, peer, "name")?
                        {
                            let subject_name = self
                                .form_attribute(c, progress.subject, "name")?
                                .ok_or(SessionError::InputMetadataUnavailable)?;
                            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
                                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
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
                            None => {
                                surgeist_html::attributes::HtmlInputType::missing_value_default()
                            }
                        };
                        // Imported native Radio is not a canonical mutable input.
                        // Only a potentially matching peer requires unavailable state.
                        if kind == surgeist_html::attributes::HtmlInputType::Radio
                            && let Some(peer_name) = self.form_attribute(c, peer, "name")?
                        {
                            let subject_name = self
                                .form_attribute(c, progress.subject, "name")?
                                .ok_or(SessionError::InputMetadataUnavailable)?;
                            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                            if !peer_name.is_empty()
                                && exact_string_equal(c, subject_name, peer_name)?
                            {
                                // The group cannot be evaluated without state
                                // the closed environment never gave the peer.
                                return Err(c
                                    .stop_unsupported_standalone(
                                        HtmlParseUnsupportedState::Provenance,
                                    )
                                    .into());
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
            Err(error) => Err(super::end_session_on(c, error)),
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
        if state.flags.radio_normalization_required(
            surgeist_html::elements::input::HtmlRadioTrigger::Connected,
        ) && self.shadow_including_connected(c, node, None)?
        {
            self.normalize_radio(c, node, None)?;
        }
        Ok(())
    }
}

#[cfg(test)]
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
            if state.flags.radio_normalization_required(
                surgeist_html::elements::input::HtmlRadioTrigger::Checkedness,
            ) {
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

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum InputGroupAttribute {
    Name,
    Checked,
}
#[cfg(test)]
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

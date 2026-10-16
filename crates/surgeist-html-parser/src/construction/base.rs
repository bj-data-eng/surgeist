//! Immediate first-base hooks in the canonical construction effects owner.
use super::*;
use crate::{HtmlBaseMissing, HtmlBaseUrlRecord, HtmlParseBaseFacts};

#[derive(Clone, Copy)]
pub(super) enum ResponseDomain {
    Unavailable,
    ClosedOwned,
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Response {
    Returned,
    Pending,
    Finished,
}
pub(super) struct FrozenBase {
    pub(super) value: HtmlBaseUrlRecord,
    pub(super) environment: HtmlParseBaseFacts,
    pub(super) document: usize,
    pub(super) response: Response,
}

pub(super) enum BaseValue {
    Unfrozen,
    Frozen(FrozenBase),
    Unavailable {
        missing: usize,
        previous: Option<FrozenBase>,
    },
}
impl BaseValue {
    pub(super) fn previous(&self) -> Option<&FrozenBase> {
        match self {
            Self::Unfrozen => None,
            Self::Frozen(value) => Some(value),
            Self::Unavailable { previous, .. } => previous.as_ref(),
        }
    }
}

#[cfg(test)]
std::thread_local! {
    static CANCEL_AFTER_BINDING: std::cell::RefCell<Option<crate::HtmlParseCancellation>> = const { std::cell::RefCell::new(None) };
}

/// A fixed filter, at most 4+28 native name units. It does not scan attributes,
/// registers, ancestors or source; the owning transition already polls.
pub(super) fn is_base(record: bounded::HtmlConstructionNode<'_>) -> bool {
    let HtmlNodeKind::Element(element) = record.kind() else {
        return false;
    };
    element.name().local_name().code_units() == [98, 97, 115, 101]
        && element.name().namespace().uri().is_some_and(|uri| {
            uri.code_units()
                == [
                    104, 116, 116, 112, 58, 47, 47, 119, 119, 119, 46, 119, 51, 46, 111, 114, 103,
                    47, 49, 57, 57, 57, 47, 120, 104, 116, 109, 108,
                ]
        })
}

impl ParserArenaSession {
    pub(crate) fn install_document_base_facts(
        &mut self,
        c: &mut ParseControl,
        facts: &HtmlParseBaseFacts,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        storage.require(c)?;
        storage.document_slot(c, 0)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if facts.document().iframe_srcdoc() {
            return Err(SessionError::WrongToken);
        }
        c.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<HtmlParseBaseFacts>(c, 1)?;
        storage.prepare_payload_slot_owned(c)?;
        storage.payloads.values.push(RetainedPayload::BaseDocument {
            node: 0,
            facts: facts.clone(),
            first: None,
        });
        Ok(())
    }
    pub(super) fn qualify_base_response(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        let index = self.active()?.index(c, node)?;
        if !is_base(
            self.active()?
                .construction
                .node_at(index)
                .expect("retained construction position"),
        ) {
            return Ok(());
        }
        capability.validate_session(c, self)?;
        // The sealed actual issuer excludes browsing-context UI/CSS visited
        // consumers, loaders and speculative parser recipients, realms and
        // external callbacks. Disabled/Inert preparation returns before any
        // speculation-rules result exists. Captures add no live recipients.
        // Thus this newly OWNED node has the full closed construction domain;
        // no claim is made for attachment/mutation of captured live documents.
        let storage = self.active_mut()?;
        let slot = storage.base_slot(c, index)?;
        payload_write::<ResponseDomain>(c, 1)?;
        let RetainedPayload::Base { domain, .. } = &mut storage.payloads.values[slot] else {
            unreachable!()
        };
        *domain = ResponseDomain::ClosedOwned;
        Ok(())
    }
}

impl SessionStorage {
    pub(super) fn register_base(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        domain: ResponseDomain,
    ) -> Result<(), SessionError> {
        if !is_base(
            self.construction
                .node_at(node)
                .expect("retained construction position"),
        ) {
            return Ok(());
        }
        self.prepare_payload_slot_owned(c)?;
        self.payloads.values.push(RetainedPayload::Base {
            node: Some(node),
            domain,
            frozen: BaseValue::Unfrozen,
        });
        Ok(())
    }
    pub(super) fn base_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<usize, SessionError> {
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(payload, RetainedPayload::Base { node: Some(bound), .. } if *bound == node)
            {
                return Ok(slot);
            }
        }
        Err(SessionError::WrongToken)
    }
    fn base_document_slot(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Option<usize>, SessionError> {
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(payload, RetainedPayload::BaseDocument { node, .. } if *node == document) {
                return Ok(Some(slot));
            }
        }
        Ok(None)
    }
    fn first_base_href(
        &self,
        c: &mut ParseControl,
        document: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        let mut next = Some(document);
        while let Some(node) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if is_base(
                self.construction
                    .node_at(node)
                    .expect("retained construction position"),
            ) && self.form_attribute(c, node, "href")?.is_some()
            {
                return Ok(Some(node));
            }
            next = self.next_view_node(c, document, node, detached)?;
        }
        Ok(None)
    }
    /// Removal's virtual detached subtree is passed through every topology read.
    /// Physical staging is not interpreted as destination ancestry.
    pub(super) fn base_change(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        href_changed: bool,
    ) -> Result<(), SessionError> {
        c.require_identity(&self.control)?;
        if !is_base(
            self.construction
                .node_at(node)
                .expect("retained construction position"),
        ) {
            return Ok(());
        }
        let base_slot = self.base_slot(c, node)?;
        let document = match self.node_document_ref(c, node)? {
            DocumentRef::Owned(document) => document,
            DocumentRef::Captured(_) => return Ok(()), // F is not captured attachment.
        };
        // Ordinary document membership, never hosted/template/shadow contents.
        // During removal, find the next first in the old actual node document.
        if detached.is_none() && self.ordinary_root(c, node, None)? != document {
            return Ok(());
        }
        let first = self.first_base_href(c, document, detached)?;
        let missing_policy = self.meta_policy_missing(c, document)?;
        if let Some(missing) = missing_policy {
            let previous_first = if let Some(slot) = self.base_document_slot(c, document)? {
                let RetainedPayload::BaseDocument { first, .. } = &self.payloads.values[slot]
                else {
                    unreachable!()
                };
                *first
            } else {
                None
            };
            if first == previous_first && !(href_changed && first == Some(node)) {
                return Ok(());
            }
            if let Some(slot) = self.base_document_slot(c, document)? {
                payload_write::<Option<usize>>(c, 1)?;
                let RetainedPayload::BaseDocument { first: binding, .. } =
                    &mut self.payloads.values[slot]
                else {
                    unreachable!()
                };
                *binding = first;
            }
            if let Some(first) = first {
                let slot = if first == node {
                    base_slot
                } else {
                    self.base_slot(c, first)?
                };
                payload_write::<BaseValue>(c, 1)?;
                let RetainedPayload::Base { frozen, .. } = &mut self.payloads.values[slot] else {
                    unreachable!()
                };
                let previous = match mem::replace(frozen, BaseValue::Unfrozen) {
                    BaseValue::Unfrozen => None,
                    BaseValue::Frozen(value) => Some(value),
                    BaseValue::Unavailable { previous, .. } => previous,
                };
                *frozen = BaseValue::Unavailable { missing, previous };
            }
            return Ok(());
        }
        let Some(document_slot) = self.base_document_slot(c, document)? else {
            if first.is_some() {
                return Err(c.stop_base_facts(HtmlBaseMissing::DocumentFacts).into());
            }
            return Ok(());
        };
        let RetainedPayload::BaseDocument {
            facts, first: old, ..
        } = &self.payloads.values[document_slot]
        else {
            unreachable!()
        };
        if first == *old && !(href_changed && first == Some(node)) {
            return Ok(());
        }
        let Some(first) = first else {
            payload_write::<Option<usize>>(c, 1)?;
            let RetainedPayload::BaseDocument { first, .. } =
                &mut self.payloads.values[document_slot]
            else {
                unreachable!()
            };
            *first = None;
            return Ok(());
        };
        let chosen_slot = if first == node {
            base_slot
        } else {
            self.base_slot(c, first)?
        };
        let environment = facts.document();
        let fallback = environment
            .fallback()
            .map_err(|missing| SessionError::Stop(c.stop_base_facts(missing)))?;
        let encoding = environment.encoding().ok_or_else(|| {
            SessionError::Stop(c.stop_base_facts(HtmlBaseMissing::CharacterEncoding))
        })?;
        let raw = self
            .form_attribute(c, first, "href")?
            .ok_or(SessionError::WrongToken)?;
        let mut selected = None;
        for observation in facts.observations() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<crate::HtmlBaseParseObservation>() as u64,
            )?;
            c.charge(WorkCategory::ProviderCall, 3)?;
            if observation.inputs_match(fallback, encoding, environment.revision())
                && exact_string_equal(c, observation.raw(), raw)?
            {
                selected = Some(observation.outcome());
                break;
            }
        }
        let outcome = selected.ok_or_else(|| {
            SessionError::Stop(c.stop_base_facts(HtmlBaseMissing::ParseObservation))
        })?;
        let (value, respond) = crate::select_frozen_base(fallback, outcome)
            .map_err(|missing| SessionError::Stop(c.stop_base_facts(missing)))?;
        c.charge(WorkCategory::ProviderCall, 2)?;
        payload_write::<BaseValue>(c, 1)?;
        payload_write::<Option<usize>>(c, 1)?;
        let frozen = FrozenBase {
            value: value.clone(),
            environment: facts.clone(),
            document,
            response: if respond {
                Response::Pending
            } else {
                Response::Returned
            },
        };
        let RetainedPayload::BaseDocument { first: binding, .. } =
            &mut self.payloads.values[document_slot]
        else {
            unreachable!()
        };
        *binding = Some(first);
        let RetainedPayload::Base {
            frozen: binding, ..
        } = &mut self.payloads.values[chosen_slot]
        else {
            unreachable!()
        };
        *binding = BaseValue::Frozen(frozen);
        #[cfg(test)]
        CANCEL_AFTER_BINDING.with(|slot| {
            if let Some(cancellation) = slot.borrow_mut().take() {
                cancellation.cancel();
            }
        });
        if respond {
            // This real checkpoint is after assignment and before response.
            c.charge(WorkCategory::RecordVisit, 1)?;
            let RetainedPayload::Base {
                domain: ResponseDomain::ClosedOwned,
                ..
            } = &self.payloads.values[chosen_slot]
            else {
                return Err(c.stop_base_facts(HtmlBaseMissing::ResponseDomain).into());
            };
            payload_write::<Response>(c, 1)?;
            let RetainedPayload::Base {
                frozen: BaseValue::Frozen(frozen),
                ..
            } = &mut self.payloads.values[chosen_slot]
            else {
                unreachable!()
            };
            frozen.response = Response::Finished;
        }
        Ok(())
    }
    pub(super) fn validate_base_facts(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::BaseDocument { node, first, .. } = payload {
                self.document_slot(c, *node)?;
                if self.first_base_href(c, *node, None)? != *first {
                    return Err(SessionError::WrongToken);
                }
            }
            if let RetainedPayload::Base {
                frozen: BaseValue::Unavailable { missing, .. },
                ..
            } = payload
            {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if !matches!(
                    self.payloads.values.get(*missing),
                    Some(RetainedPayload::MetaMissing {
                        kind: meta::MetaMissingKind::Csp,
                        ..
                    })
                ) {
                    return Err(SessionError::WrongToken);
                }
            }
            if let RetainedPayload::Base {
                node,
                frozen: BaseValue::Frozen(frozen),
                ..
            } = payload
            {
                let node = node.ok_or(SessionError::WrongToken)?;
                if !is_base(
                    self.construction
                        .node_at(node)
                        .expect("retained construction position"),
                ) || frozen.response == Response::Pending
                {
                    return Err(c
                        .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                        .into());
                }
                let slot = self
                    .base_document_slot(c, frozen.document)?
                    .ok_or(SessionError::WrongToken)?;
                let RetainedPayload::BaseDocument { facts, .. } = &self.payloads.values[slot]
                else {
                    unreachable!()
                };
                c.charge(WorkCategory::ProviderCall, 1)?;
                if !facts.same_owner(&frozen.environment) {
                    return Err(SessionError::WrongToken);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::Event};

    fn request(text: &str) -> crate::HtmlDocumentParseRequest {
        let revision =
            crate::HtmlBaseFactsRevision::new(crate::HtmlBaseProducerId::fresh(), 1, 1, 1);
        let fallback = crate::HtmlBaseUrlRecord::observed(
            revision.clone(),
            "https://base.test/".into(),
            crate::HtmlBaseUrlScheme::Other,
            false,
        )
        .unwrap();
        let encoding = crate::HtmlBaseEncoding::fresh();
        let observations = ["A", "B", "C"]
            .into_iter()
            .map(|raw| {
                let output = match raw {
                    "A" => "https://base.test/A",
                    "B" => "https://base.test/B",
                    _ => "https://base.test/C",
                };
                crate::HtmlBaseParseObservation::new(
                    raw.into(),
                    fallback.clone(),
                    encoding.clone(),
                    revision.clone(),
                    crate::HtmlBaseParseOutcome::Parsed {
                        record: crate::HtmlBaseUrlRecord::observed(
                            revision.clone(),
                            output.into(),
                            crate::HtmlBaseUrlScheme::Other,
                            false,
                        )
                        .unwrap(),
                        base_allowed: crate::HtmlBaseAllowed::Allowed,
                    },
                )
                .unwrap()
            })
            .collect();
        let facts = crate::HtmlParseBaseFacts::new(
            crate::HtmlBaseDocumentFacts::new(
                revision,
                Some(fallback),
                crate::HtmlAboutBaseUrl::KnownNull,
                false,
                Some(encoding),
            )
            .unwrap(),
            observations,
            crate::HtmlBaseFactsLimits::new(3, 1000, 10000),
        )
        .unwrap();
        crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(text),
            super::super::tests::limits(),
        )
        .with_base_facts(facts)
        .unwrap()
    }
    fn tag(lexer: &mut LexerFixture<'_>, c: &mut ParseControl) -> Tag {
        loop {
            match lexer.next(c, false).unwrap().unwrap() {
                Event::Token(event) => {
                    let token = event.into_token(c).unwrap();
                    if let TokenKind::Tag(tag) = token.kind {
                        return tag;
                    }
                    release_token_kind(&mut Some(token.kind), c).unwrap();
                }
                event => event.release(c).unwrap(),
            }
        }
    }
    fn node(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        lexer: &mut LexerFixture<'_>,
    ) -> ParserNode {
        let mut incoming = Some(tag(lexer, c));
        session.allocate_html_tag(c, &mut incoming).unwrap()
    }
    fn attribute(
        lexer: &mut LexerFixture<'_>,
        c: &mut ParseControl,
    ) -> Option<crate::parser::tokenizer::token::Attribute> {
        let Tag {
            name,
            mut attributes,
            ..
        } = tag(lexer, c);
        name.release(c).unwrap();
        let value = attributes.try_pop(c).unwrap();
        release_token_attributes(attributes, c).unwrap();
        value
    }
    fn frozen(session: &ParserArenaSession, node: &ParserNode) -> Option<(Vec<u16>, Response)> {
        session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|payload| match payload {
                RetainedPayload::Base {
                    node: Some(owner),
                    frozen: BaseValue::Frozen(value),
                    ..
                } if *owner == node.index => Some((
                    value.value.serialization().code_units().to_vec(),
                    value.response,
                )),
                _ => None,
            })
    }
    fn first(session: &ParserArenaSession) -> Option<usize> {
        session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|payload| match payload {
                RetainedPayload::BaseDocument { first, .. } => Some(*first),
                _ => None,
            })
            .unwrap()
    }
    fn assert_value(session: &ParserArenaSession, node: &ParserNode, expected: &str) {
        let (value, response) = frozen(session, node).unwrap();
        assert_eq!(value, expected.encode_utf16().collect::<Vec<_>>());
        assert!(response == Response::Finished);
    }

    #[test]
    fn already_first_base_target_append_keeps_finished_frozen_response() {
        // The pinned trigger is a change of first href-bearing Base or of its
        // href. Adding target through the same canonical merge owner is neither.
        let request = request("<base href=A><i target=_blank>");
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let base = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &root, &base, None, &capability)
            .unwrap();
        assert_value(&session, &base, "https://base.test/A");
        let original_record = match &session.active().unwrap().payloads.values[session
            .active()
            .unwrap()
            .base_slot(&mut c, base.index)
            .unwrap()]
        {
            RetainedPayload::Base {
                frozen: BaseValue::Frozen(value),
                ..
            } => value.value.clone(),
            _ => unreachable!(),
        };
        let mut incoming = attribute(&mut lexer, &mut c);
        CANCEL_AFTER_BINDING.with(|slot| *slot.borrow_mut() = Some(request.cancellation().clone()));
        let result = session.set_managed_attribute(
            &mut c,
            &base,
            &mut incoming,
            &capability,
            mutation::ManagedFamily::Base,
        );
        let observer_unfired = CANCEL_AFTER_BINDING.with(|slot| slot.borrow_mut().take().is_some());
        let slot = session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|payload| match payload {
                RetainedPayload::Base {
                    node: Some(owner),
                    frozen: BaseValue::Frozen(value),
                    ..
                } if *owner == base.index => Some((value.value.clone(), value.response)),
                _ => None,
            })
            .unwrap();
        let first_after = first(&session);
        let parent_after = session
            .active()
            .unwrap()
            .construction
            .node_at(base.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position());
        let attributes = match &session
            .active()
            .unwrap()
            .construction
            .node_at(base.index)
            .expect("retained construction position")
            .kind()
        {
            HtmlNodeKind::Element(element) => element
                .attributes()
                .iter()
                .map(|a| {
                    (
                        a.name().local_name().code_units().to_vec(),
                        a.value().code_units().to_vec(),
                    )
                })
                .collect::<Vec<_>>(),
            _ => unreachable!(),
        };
        let stop = c.stop();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(result, Ok(()));
        assert!(incoming.is_none());
        assert!(observer_unfired);
        assert_eq!(stop, None);
        assert!(!request.cancellation().is_cancelled());
        assert_eq!(first_after, Some(base.index));
        assert_eq!(parent_after, Some(root.index));
        assert_eq!(slot.0, original_record);
        assert!(slot.1 == Response::Finished);
        assert_eq!(
            attributes,
            vec![
                (
                    "href".encode_utf16().collect(),
                    "A".encode_utf16().collect()
                ),
                (
                    "target".encode_utf16().collect(),
                    "_blank".encode_utf16().collect()
                ),
            ]
        );
    }

    #[test]
    fn actual_first_base_href_append_and_change_assign_before_response() {
        for text in ["<base target=_self><i href=B>", "<base href=A><i href=B>"] {
            let request = request(text);
            let (mut c, mut session, capability) =
                super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let base = node(&mut session, &mut c, &mut lexer);
            session
                .insert_parser(&mut c, &root, &base, None, &capability)
                .unwrap();
            let mut incoming = attribute(&mut lexer, &mut c);
            CANCEL_AFTER_BINDING
                .with(|slot| *slot.borrow_mut() = Some(request.cancellation().clone()));
            let result = session.set_managed_attribute(
                &mut c,
                &base,
                &mut incoming,
                &capability,
                mutation::ManagedFamily::Base,
            );
            CANCEL_AFTER_BINDING.with(|slot| assert!(slot.borrow().is_none()));
            assert_eq!(result, Err(SessionError::Stop(ParseStop::Cancelled)));
            assert!(incoming.is_none());
            assert_eq!(first(&session), Some(base.index));
            let (value, response) = frozen(&session, &base).unwrap();
            assert_eq!(
                value,
                "https://base.test/B".encode_utf16().collect::<Vec<_>>()
            );
            assert!(response == Response::Pending);
            let at_stop = c.first_stop_usage().unwrap().clone();
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(ParseStop::Cancelled))
            );
            assert_eq!(c.first_stop_usage(), Some(&at_stop));
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            assert_eq!(c.first_stop_usage(), Some(&at_stop));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                at_stop.resource(HtmlParseResource::WorkCredits).current
            );
            super::super::tests::storage_zero(&c);
        }
    }

    #[test]
    fn real_insert_reorder_remove_href_change_and_clone_bind_the_immediate_first() {
        let request = request("<base href=A><base href=B><i href=C>");
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let a = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &root, &a, None, &capability)
            .unwrap();
        assert_eq!(first(&session), Some(a.index));
        assert_value(&session, &a, "https://base.test/A");
        let b = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &root, &b, None, &capability)
            .unwrap();
        assert!(frozen(&session, &b).is_none());
        session
            .insert(&mut c, &root, &b, Some(&a), Some(&capability))
            .unwrap();
        assert_eq!(first(&session), Some(b.index));
        assert_value(&session, &b, "https://base.test/B");
        let mut incoming = attribute(&mut lexer, &mut c);
        session
            .set_managed_attribute(
                &mut c,
                &b,
                &mut incoming,
                &capability,
                mutation::ManagedFamily::Base,
            )
            .unwrap();
        assert!(incoming.is_none());
        assert_value(&session, &b, "https://base.test/C");
        session
            .active_mut()
            .unwrap()
            .remove_managed_field(&mut c, b.index, "href")
            .unwrap();
        assert_eq!(first(&session), Some(a.index));
        assert_value(&session, &a, "https://base.test/A");
        let storage = session.active_mut().unwrap();
        let document = storage.node_document_ref(&mut c, a.index).unwrap();
        let copy = storage.clone_node(&mut c, a.index, document).unwrap();
        let copy = storage.node(&mut c, copy).unwrap();
        assert_ne!(copy.index, a.index);
        assert!(frozen(&session, &copy).is_none());
        session
            .insert(&mut c, &root, &copy, Some(&a), Some(&capability))
            .unwrap();
        assert_eq!(first(&session), Some(copy.index));
        assert_value(&session, &copy, "https://base.test/A");
        session.remove(&mut c, &copy, Some(&capability)).unwrap();
        assert_eq!(first(&session), Some(a.index));
        session.remove(&mut c, &a, Some(&capability)).unwrap();
        assert_eq!(first(&session), None);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn actual_first_href_exact_one_short_and_pre_cancel_preserve_binding_and_cleanup() {
        for short in [false, true] {
            let request = request("<base href=A>");
            let (mut c, mut session, capability) =
                super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let base = node(&mut session, &mut c, &mut lexer);
            session
                .qualify_base_response(&mut c, &base, &capability)
                .unwrap();
            let storage = session.active_mut().unwrap();
            // Real pre-insert owner commits topology/effect retention. This test
            // then selects its immediate intrinsic base hook, not a public
            // insertion total or an invented checkpoint.
            let before = storage
                .construction
                .node_at(root.index)
                .expect("retained construction position")
                .child_at(0)
                .map(|node| node.position());
            storage
                .insert_indices_with_cause(
                    &mut c,
                    root.index,
                    base.index,
                    before,
                    InsertionCause::Parser,
                )
                .unwrap();
            assert_eq!(
                storage
                    .construction
                    .node_at(root.index)
                    .expect("retained construction position")
                    .child_at(0)
                    .expect("retained child ordinal")
                    .position(),
                base.index
            );
            assert_eq!(
                storage
                    .construction
                    .node_at(base.index)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position()),
                Some(root.index)
            );
            let position = |predicate: fn(&RetainedPayload) -> bool| {
                storage.payloads.values.iter().position(predicate).unwrap() as u64 + 1
            };
            let base_slot = storage.base_slot(&mut c, base.index).unwrap() as u64 + 1;
            let node_document = storage.payloads.values.iter().position(|p| matches!(p, RetainedPayload::NodeDocument { node: Some(n), .. } if *n == base.index)).unwrap() as u64 + 1;
            let document =
                position(|p| matches!(p, RetainedPayload::Document { node: Some(0), .. }));
            let environment =
                position(|p| matches!(p, RetainedPayload::BaseDocument { node: 0, .. }));
            // Source-derived cost: each registry prefix visits exactly its
            // admitted slots. NodeDocument adds document header+prefix+fixed
            // copy; ordinary root has two parent reads. First selection has
            // root,next-child,base visits4 + href lookup16 =20. The second
            // href lookup costs16; observation header+3 providers+2 headers+
            // one UTF16 pair costs sizeof(observation)+10. Finally two
            // provider clones and two actual binding receivers commit before
            // the response's visit+fixed receiver. Current-policy qualification
            // adds a Document header+prefix and one history observation (the
            // extra document+2). The binding receiver is the actual BaseValue
            // enum, not the former nullable frozen slot. No usage sample is oracle.
            let total = base_slot
                + node_document
                + 1
                + document
                + mem::size_of::<DocumentRef>() as u64
                + 2
                + 20
                + environment
                + document
                + 2
                + 16
                + mem::size_of::<crate::HtmlBaseParseObservation>() as u64
                + 10
                + 2
                + mem::size_of::<BaseValue>() as u64
                + 1
                + mem::size_of::<Option<usize>>() as u64
                + 1
                + 1
                + mem::size_of::<Response>() as u64
                + 1;
            let remaining = request.limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                remaining - total + u64::from(short),
            )
            .unwrap();
            let result = storage.base_change(&mut c, base.index, None, false);
            if short {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
                assert!(frozen(&session, &base).unwrap().1 == Response::Pending);
                assert_eq!(first(&session), Some(base.index));
                let stop = c.stop();
                assert_eq!(session.freeze(&mut c).err(), stop.map(SessionError::Stop));
                request.cancellation().cancel();
                assert_eq!(
                    session
                        .active_mut()
                        .unwrap()
                        .base_change(&mut c, base.index, None, false)
                        .err(),
                    stop.map(SessionError::Stop)
                );
                assert_eq!(c.stop(), stop);
            } else {
                assert_eq!(result, Ok(()));
                assert_value(&session, &base, "https://base.test/A");
                assert!(c.charge(WorkCategory::StateStep, 1).is_err());
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
        }
        let request = request("<base href=A>");
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let base = node(&mut session, &mut c, &mut lexer);
        request.cancellation().cancel();
        assert_eq!(
            session
                .insert_parser(&mut c, &root, &base, None, &capability)
                .err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        assert!(frozen(&session, &base).is_none());
        assert_eq!(first(&session), None);
        assert_eq!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(base.index)
                .expect("retained construction position")
                .parent()
                .map(|node| node.position()),
            None
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn template_adoption_keeps_historical_slot_but_insertion_recomputes_and_alias_refusal_retries()
    {
        let request = request("<base href=A><template>");
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let base = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &root, &base, None, &capability)
            .unwrap();
        let template = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &root, &template, None, &capability)
            .unwrap();
        let contents = session
            .template_contents(&mut c, &template)
            .unwrap()
            .unwrap();
        let contents_document = session
            .active()
            .unwrap()
            .node_document_ref(&mut c, contents.index)
            .unwrap();
        assert!(contents_document != DocumentRef::Owned(0));
        session
            .insert(&mut c, &contents, &base, None, Some(&capability))
            .unwrap();
        assert_eq!(first(&session), None);
        assert_eq!(
            session
                .active()
                .unwrap()
                .node_document_ref(&mut c, base.index)
                .unwrap(),
            contents_document
        );
        assert_value(&session, &base, "https://base.test/A");
        let storage = session.active_mut().unwrap();
        let copy = storage
            .clone_node(&mut c, base.index, contents_document)
            .unwrap();
        let copy = storage.node(&mut c, copy).unwrap();
        assert!(frozen(&session, &copy).is_none());
        session
            .insert(&mut c, &contents, &copy, None, Some(&capability))
            .unwrap();
        assert_eq!(first(&session), None);
        assert!(frozen(&session, &copy).is_none());
        session
            .insert(&mut c, &root, &copy, None, Some(&capability))
            .unwrap();
        assert_eq!(first(&session), Some(copy.index));
        assert_eq!(
            session
                .active()
                .unwrap()
                .node_document_ref(&mut c, copy.index)
                .unwrap(),
            DocumentRef::Owned(0)
        );
        assert_value(&session, &copy, "https://base.test/A");
        let alias = match &session
            .active()
            .unwrap()
            .construction
            .node_at(copy.index)
            .expect("retained construction position")
            .kind()
        {
            HtmlNodeKind::Element(element) => element.attributes().clone(),
            _ => unreachable!(),
        };
        assert_eq!(
            session.preflight_completed_freeze(&mut c),
            Err(SessionError::AliasedPayload)
        );
        assert!(c.stop().is_none());
        assert_eq!(first(&session), Some(copy.index));
        drop(alias);
        lexer.release(&mut c).unwrap();
        session.preflight_completed_freeze(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        request.cancellation().cancel();
        frozen.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn original_cancel_after_frozen_assignment_prevents_response_and_publication() {
        let request = request("<base href=A>");
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let base = node(&mut session, &mut c, &mut lexer);
        CANCEL_AFTER_BINDING.with(|slot| *slot.borrow_mut() = Some(request.cancellation().clone()));
        let result = session.insert_parser(&mut c, &root, &base, None, &capability);
        assert_eq!(result, Err(SessionError::Stop(ParseStop::Cancelled)));
        CANCEL_AFTER_BINDING.with(|slot| assert!(slot.borrow().is_none()));
        assert_eq!(first(&session), Some(base.index));
        assert_eq!(
            frozen(&session, &base).unwrap().0,
            "https://base.test/A".encode_utf16().collect::<Vec<_>>()
        );
        assert!(frozen(&session, &base).unwrap().1 == Response::Pending);
        assert_eq!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(base.index)
                .expect("retained construction position")
                .parent()
                .map(|node| node.position()),
            Some(root.index)
        );
        let at_stop = c.first_stop_usage().unwrap().clone();
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(
            session.insert_parser(&mut c, &root, &base, None, &capability),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(c.first_stop_usage(), Some(&at_stop));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            at_stop.resource(HtmlParseResource::WorkCredits).current
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        assert_eq!(c.first_stop_usage(), Some(&at_stop));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            at_stop.resource(HtmlParseResource::WorkCredits).current
        );
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn nonce_only_append_preserves_first_base_frozen_response() {
        // Base freezes for a new first href-bearing element or its href change.
        // Appending nonce exercises the actual independent installed-attribute
        // hook and shared merge owner without satisfying either Base trigger.
        let request = request("<base href=A>");
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let base = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &root, &base, None, &capability)
            .unwrap();
        assert_value(&session, &base, "https://base.test/A");
        let value = literal_string(&mut c, b"secret").unwrap();
        CANCEL_AFTER_BINDING.with(|slot| *slot.borrow_mut() = Some(request.cancellation().clone()));
        let result =
            session
                .active_mut()
                .unwrap()
                .set_nonce_content(&mut c, base.index, Some(value));
        let observer_unfired = CANCEL_AFTER_BINDING.with(|slot| slot.borrow_mut().take().is_some());
        let first_after = first(&session);
        let parent_after = session
            .active()
            .unwrap()
            .construction
            .node_at(base.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position());
        let frozen_after = frozen(&session, &base);
        let nonce_after = if result.is_ok() {
            Some(
                session
                    .cryptographic_nonce(&mut c, &base)
                    .unwrap()
                    .code_units()
                    .to_vec(),
            )
        } else {
            None
        };
        let attributes = match &session
            .active()
            .unwrap()
            .construction
            .node_at(base.index)
            .expect("retained construction position")
            .kind()
        {
            HtmlNodeKind::Element(element) => element
                .attributes()
                .iter()
                .map(|a| {
                    (
                        a.name().local_name().code_units().to_vec(),
                        a.value().code_units().to_vec(),
                    )
                })
                .collect::<Vec<_>>(),
            _ => unreachable!(),
        };
        let stop = c.stop();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(result, Ok(()));
        assert!(observer_unfired);
        assert_eq!(stop, None);
        assert!(!request.cancellation().is_cancelled());
        assert_eq!(first_after, Some(base.index));
        assert_eq!(parent_after, Some(root.index));
        let (url, response) = frozen_after.unwrap();
        assert_eq!(
            url,
            "https://base.test/A".encode_utf16().collect::<Vec<_>>()
        );
        assert!(response == Response::Finished);
        assert_eq!(nonce_after, Some(vec![115, 101, 99, 114, 101, 116]));
        assert_eq!(
            attributes,
            vec![
                (vec![104, 114, 101, 102], vec![65]),
                (
                    vec![110, 111, 110, 99, 101],
                    vec![115, 101, 99, 114, 101, 116]
                ),
            ]
        );
    }

    #[test]
    fn unavailable_policy_recomputation_keeps_real_prior_frozen_value() {
        let request = request(
            "<head><base href=A><meta http-equiv=content-security-policy content=policy><i href=B>",
        );
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let head = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &root, &head, None, &capability)
            .unwrap();
        let base = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &head, &base, None, &capability)
            .unwrap();
        assert_value(&session, &base, "https://base.test/A");
        let meta = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &head, &meta, None, &capability)
            .unwrap();
        let mut incoming = attribute(&mut lexer, &mut c);
        session
            .set_managed_attribute(
                &mut c,
                &base,
                &mut incoming,
                &capability,
                mutation::ManagedFamily::Base,
            )
            .unwrap();
        let storage = session.active().unwrap();
        let slot = storage.base_slot(&mut c, base.index).unwrap();
        let RetainedPayload::Base {
            frozen:
                BaseValue::Unavailable {
                    missing,
                    previous: Some(value),
                },
            ..
        } = &storage.payloads.values[slot]
        else {
            panic!("unknown recomputation retains real historical state")
        };
        assert_eq!(
            value.value.serialization(),
            &DocumentString::from("https://base.test/A")
        );
        assert!(value.response == Response::Finished);
        assert!(matches!(
            storage.payloads.values[*missing],
            RetainedPayload::MetaMissing {
                kind: meta::MetaMissingKind::Csp,
                ..
            }
        ));
        assert_eq!(first(&session), Some(base.index));
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
}

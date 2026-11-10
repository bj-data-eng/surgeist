//! Immediate first-base hooks in the canonical construction effects owner.
use super::*;
use crate::{HtmlBaseMissing, HtmlBaseUrlRecord, HtmlParseBaseFacts};
use surgeist_html::construction::ConstructionNodeKind;

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
/// What a subtree's base hooks still wait for: nothing known yet, no base with an `href` yet to
/// run, or the last base with an `href` of the subtree, whose hook has yet to run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HrefBaseAhead {
    Unscanned,
    None,
    Last(usize),
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
pub(super) fn is_base(
    record: surgeist_html::bounded::construction::HtmlConstructionNode<'_>,
) -> bool {
    let ConstructionNodeKind::Element(element) = record.kind() else {
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if facts.document().iframe_srcdoc() {
            return Err(SessionError::WrongToken);
        }
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        payload_write::<HtmlParseBaseFacts>(c, 1)?;
        let prepared = storage.prepare_payload_slot_owned(c)?;
        storage
            .payloads
            .push_row(
                prepared,
                RetainedPayload::BaseDocument {
                    node: 0,
                    facts: facts.clone(),
                    first: None,
                },
            )
            .map_err(|_| latch_registry_row(c))?;
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
                .ok_or_else(|| latch_registry_row(c))?,
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
        let Some(RetainedPayload::Base { domain, .. }) = storage.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(c));
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
                .ok_or_else(|| latch_registry_row(c))?,
        ) {
            return Ok(());
        }
        let prepared = self.prepare_payload_slot_for(c, Some(KindList::Base))?;
        self.payloads
            .push_row(
                prepared,
                RetainedPayload::Base {
                    node: Some(node),
                    domain,
                    frozen: BaseValue::Unfrozen,
                },
            )
            .map_err(|_| latch_registry_row(c))?;
        Ok(())
    }
    pub(super) fn base_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<usize, SessionError> {
        if let Some(slot) = self.payloads.joined(c, node, Join::Base)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if matches!(self.payloads.values.get(slot), Some(RetainedPayload::Base { node: Some(bound), .. }) if *bound == node)
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
        if let Some(slot) = self.payloads.joined(c, document, Join::BaseDocument)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if matches!(self.payloads.values.get(slot), Some(RetainedPayload::BaseDocument { node, .. }) if *node == document)
            {
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if is_base(
                self.construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?,
            ) && self.form_attribute(c, node, "href")?.is_some()
            {
                return Ok(Some(node));
            }
            next = self.next_view_node(c, document, node, detached)?;
        }
        Ok(None)
    }
    /// Whether inserting or removing `node`, a base element, leaves the document's first
    /// base element with an `href` as it was, so that no search of the document is needed
    /// (HTML 877d6146487f, the base element: the document base URL follows "the first base
    /// element in tree order that has an href attribute"). A base without an `href` is never
    /// that element, and the element does not change when one is added before or after it.
    /// Removing one is no change unless the removed subtree held the first base with an
    /// `href`, which the document's recorded first tells without a search: no first, or no
    /// record of one because the document has no base facts, which stops the parse at the
    /// first `href` that could make one.
    ///
    /// An inserted subtree's hooks run in tree order once the whole subtree is attached, so
    /// a base with an `href` later in it is already the first in the tree the search reads and
    /// not yet the one recorded. Nothing is skipped while one remains: the first hook of a
    /// subtree that needs the answer scans the subtree once for its last such base, and the
    /// hook of that base ends the wait. A change that is not one of a subtree's hooks
    /// (`effect_root` is `None`) is never skipped. So the answer, and every record made for the
    /// nodes between, are those of the search.
    fn base_cannot_move_first(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        document: usize,
        detached: Option<usize>,
        effect_root: Option<usize>,
    ) -> Result<bool, SessionError> {
        let Some(root) = effect_root else {
            return Ok(false);
        };
        if self.form_attribute(c, node, "href")?.is_some() {
            if let Some(effects) = self.effects.form.as_mut()
                && effects.href_base == HrefBaseAhead::Last(node)
            {
                payload_write::<HrefBaseAhead>(c, 1)?;
                effects.href_base = HrefBaseAhead::None;
            }
            return Ok(false);
        }
        if detached.is_none() {
            return Ok(self.href_base_ahead(c, root, node)? == HrefBaseAhead::None);
        }
        let Some(slot) = self.base_document_slot(c, document)? else {
            return Ok(true);
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::BaseDocument { first, .. }) = self.payloads.values.get(slot)
        else {
            return Err(latch_registry_row(c));
        };
        Ok(first.is_none())
    }
    /// The last base element with an `href` in the subtree of `root` whose hook has yet to run,
    /// found by one scan of the nodes after `node`, the first whose hook asks, for the whole
    /// effect: the hooks run in tree order, so the nodes before it have had theirs.
    fn href_base_ahead(
        &mut self,
        c: &mut ParseControl,
        root: usize,
        node: usize,
    ) -> Result<HrefBaseAhead, SessionError> {
        let known = match self.effects.form.as_ref() {
            Some(effects) if effects.root == root => effects.href_base,
            _ => return Err(latch_registry_row(c)),
        };
        if known != HrefBaseAhead::Unscanned {
            return Ok(known);
        }
        let mut last = HrefBaseAhead::None;
        let mut next = self.next_effect_node(c, root, node)?;
        while let Some(current) = next {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if is_base(
                self.construction
                    .node_at(current)
                    .ok_or_else(|| latch_registry_row(c))?,
            ) && self.form_attribute(c, current, "href")?.is_some()
            {
                last = HrefBaseAhead::Last(current);
            }
            next = self.next_effect_node(c, root, current)?;
        }
        payload_write::<HrefBaseAhead>(c, 1)?;
        let Some(effects) = self.effects.form.as_mut() else {
            return Err(latch_registry_row(c));
        };
        effects.href_base = last;
        Ok(last)
    }
    /// Removal's virtual detached subtree is passed through every topology read.
    /// Physical staging is not interpreted as destination ancestry.
    pub(super) fn base_change(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        href_changed: bool,
        effect_root: Option<usize>,
    ) -> Result<(), SessionError> {
        self.base_change_inner(c, node, detached, href_changed, effect_root)?;
        self.record_association(
            c,
            node,
            pending_association::HtmlParseAssociationOccurrenceKind::BaseChanged,
            detached,
            None,
            false,
        )
    }
    fn base_change_inner(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        href_changed: bool,
        effect_root: Option<usize>,
    ) -> Result<(), SessionError> {
        c.require_identity(&self.control)?;
        if !is_base(
            self.construction
                .node_at(node)
                .ok_or_else(|| latch_registry_row(c))?,
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
        if !href_changed && self.base_cannot_move_first(c, node, document, detached, effect_root)? {
            return Ok(());
        }
        let first = self.first_base_href(c, document, detached)?;
        let missing_policy = self.meta_policy_missing(c, document)?;
        if let Some(missing) = missing_policy {
            let previous_first = if let Some(slot) = self.base_document_slot(c, document)? {
                let Some(RetainedPayload::BaseDocument { first, .. }) =
                    self.payloads.values.get(slot)
                else {
                    return Err(latch_registry_row(c));
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
                let Some(RetainedPayload::BaseDocument { first: binding, .. }) =
                    self.payloads.values.get_mut(slot)
                else {
                    return Err(latch_registry_row(c));
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
                let Some(RetainedPayload::Base { frozen, .. }) = self.payloads.values.get_mut(slot)
                else {
                    return Err(latch_registry_row(c));
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
        let Some(RetainedPayload::BaseDocument {
            facts, first: old, ..
        }) = self.payloads.values.get(document_slot)
        else {
            return Err(latch_registry_row(c));
        };
        if first == *old && !(href_changed && first == Some(node)) {
            return Ok(());
        }
        let Some(first) = first else {
            payload_write::<Option<usize>>(c, 1)?;
            let Some(RetainedPayload::BaseDocument { first, .. }) =
                self.payloads.values.get_mut(document_slot)
            else {
                return Err(latch_registry_row(c));
            };
            *first = None;
            return Ok(());
        };
        let chosen_slot = if first == node {
            base_slot
        } else {
            self.base_slot(c, first)?
        };
        if !self.csp_base_current(c, document)? {
            return Err(c.stop_base_facts(HtmlBaseMissing::DocumentFacts).into());
        }
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                mem::size_of::<crate::HtmlBaseParseObservation>() as u64,
            )?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 3)?;
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
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
        let Some(RetainedPayload::BaseDocument { first: binding, .. }) =
            self.payloads.values.get_mut(document_slot)
        else {
            return Err(latch_registry_row(c));
        };
        *binding = Some(first);
        let Some(RetainedPayload::Base {
            frozen: binding, ..
        }) = self.payloads.values.get_mut(chosen_slot)
        else {
            return Err(latch_registry_row(c));
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let Some(RetainedPayload::Base {
                domain: ResponseDomain::ClosedOwned,
                ..
            }) = self.payloads.values.get(chosen_slot)
            else {
                return Err(c.stop_base_facts(HtmlBaseMissing::ResponseDomain).into());
            };
            payload_write::<Response>(c, 1)?;
            let Some(RetainedPayload::Base {
                frozen: BaseValue::Frozen(frozen),
                ..
            }) = self.payloads.values.get_mut(chosen_slot)
            else {
                return Err(latch_registry_row(c));
            };
            frozen.response = Response::Finished;
        }
        Ok(())
    }
    pub(super) fn validate_base_facts(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        for payload in &self.payloads.values {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::BaseDocument { node, first, .. } = payload {
                self.document_slot(c, *node)?;
                if self.first_base_href(c, *node, None)? != *first {
                    return Err(latch_registry_row(c));
                }
            }
            if let RetainedPayload::Base {
                frozen: BaseValue::Unavailable { missing, .. },
                ..
            } = payload
            {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if !matches!(
                    self.payloads.values.get(*missing),
                    Some(RetainedPayload::MetaMissing {
                        kind: meta::HtmlParseMetaMissingKind::Csp,
                        ..
                    })
                ) {
                    return Err(latch_registry_row(c));
                }
            }
            if let RetainedPayload::Base {
                node,
                frozen: BaseValue::Frozen(frozen),
                ..
            } = payload
            {
                let node = node.ok_or_else(|| latch_registry_row(c))?;
                if !is_base(
                    self.construction
                        .node_at(node)
                        .ok_or_else(|| latch_registry_row(c))?,
                ) {
                    return Err(latch_registry_row(c));
                }
                if frozen.response == Response::Pending {
                    return Err(c
                        .fail_invariant(
                            crate::parser::control::HtmlParseInvariant::PendingFormEffects,
                        )
                        .into());
                }
                let slot = self
                    .base_document_slot(c, frozen.document)?
                    .ok_or_else(|| latch_registry_row(c))?;
                let Some(RetainedPayload::BaseDocument { facts, .. }) =
                    self.payloads.values.get(slot)
                else {
                    return Err(latch_registry_row(c));
                };
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                if !facts.same_owner(&frozen.environment) {
                    return Err(latch_registry_row(c));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture, token::Event};
    use surgeist_html::{construction::ConstructionNodeKind, values::DocumentString};

    pub(in crate::construction) fn request(text: &str) -> crate::HtmlDocumentParseRequest {
        let revision =
            crate::HtmlBaseFactsRevision::new(crate::HtmlBaseProducerId::fresh(), 1, 1, 1);
        let fallback = crate::HtmlBaseUrlRecord::observed(
            revision.clone(),
            "https://base.test/".into(),
            crate::HtmlBaseUrlScheme::Other,
            false,
            None,
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
                            None,
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
    // A parsed base element inserted at the root, ready for its intrinsic hook.
    macro_rules! inserted_base_fixture {
        ($request:ident, $c:ident, $session:ident, $root:ident, $lexer:ident, $base:ident) => {
            let (mut $c, mut $session, capability) =
                super::super::tests::fresh_script_owner(&$request);
            let $root = $session.root(&mut $c).unwrap();
            let mut $lexer =
                LexerFixture::new($request.source(), HtmlTokenizerState::Data, None, &mut $c)
                    .unwrap();
            let $base = node(&mut $session, &mut $c, &mut $lexer);
            $session
                .qualify_base_response(&mut $c, &$base, &capability)
                .unwrap();
            {
                let storage = $session.active_mut().unwrap();
                let before = storage
                    .construction
                    .node_at($root.index)
                    .unwrap()
                    .child_at(0)
                    .map(|n| n.position());
                storage
                    .insert_indices_with_cause(
                        &mut $c,
                        $root.index,
                        $base.index,
                        before,
                        InsertionCause::Parser,
                    )
                    .unwrap();
            }
        };
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
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
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
            ConstructionNodeKind::Element(element) => element
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
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
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
            assert_eq!(result, Err(SessionError::Stop(HtmlParseStop::Cancelled)));
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
                Some(SessionError::Stop(HtmlParseStop::Cancelled))
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

    /// Validation of a frozen base whose row is inconsistent. A row naming no
    /// node, or a node that is not a base element, is a row of another kind
    /// (the registry row); a response that is still owed is a pending effect.
    #[test]
    fn validating_a_frozen_base_labels_row_faults_and_a_pending_response_apart() {
        use crate::parser::control::HtmlParseInvariant;
        let registry_row = HtmlParseStop::InternalInvariant(HtmlParseInvariant::RegistryRow);
        let pending = HtmlParseStop::InternalInvariant(HtmlParseInvariant::PendingFormEffects);
        for (case, expected) in [(0, registry_row), (1, registry_row), (2, pending)] {
            let request = request("<base href=A>");
            let (mut c, mut session, capability) =
                super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let base = node(&mut session, &mut c, &mut lexer);
            session
                .insert_parser(&mut c, &root, &base, None, &capability)
                .unwrap();
            session
                .active()
                .unwrap()
                .validate_base_facts(&mut c)
                .unwrap();
            let storage = session.active_mut().unwrap();
            let row = storage
                .payloads
                .values
                .iter_mut()
                .find_map(|payload| match payload {
                    RetainedPayload::Base {
                        node: owner,
                        frozen: BaseValue::Frozen(frozen),
                        ..
                    } if *owner == Some(base.index) => Some((owner, frozen)),
                    _ => None,
                })
                .unwrap();
            let original = row.1.response;
            match case {
                // The row names no node at all.
                0 => *row.0 = None,
                // The row names a node that is not a base element.
                1 => *row.0 = Some(root.index),
                // The base response is still owed.
                _ => row.1.response = Response::Pending,
            }
            let result = session.active().unwrap().validate_base_facts(&mut c);
            assert_eq!(result, Err(SessionError::Stop(expected)), "case {case}");
            assert_eq!(c.stop(), Some(expected), "case {case}");
            // The registry's own join check runs at release, so restore the row.
            for payload in &mut session.active_mut().unwrap().payloads.values {
                if let RetainedPayload::Base { node, frozen, .. } = payload
                    && let BaseValue::Frozen(frozen) = frozen
                    && (*node != Some(base.index) || case == 2)
                {
                    *node = Some(base.index);
                    frozen.response = original;
                }
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
        }
    }

    /// A base document row that disagrees with the tree, and a frozen base
    /// whose document has no base-document row, are retained rows that are
    /// inconsistent or missing.
    #[test]
    fn validating_base_rows_that_disagree_with_the_tree_reports_the_registry_row() {
        use crate::parser::control::HtmlParseInvariant;
        let registry_row = HtmlParseStop::InternalInvariant(HtmlParseInvariant::RegistryRow);
        for case in 0..2 {
            let request = request("<base href=A>");
            let (mut c, mut session, capability) =
                super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let base = node(&mut session, &mut c, &mut lexer);
            session
                .insert_parser(&mut c, &root, &base, None, &capability)
                .unwrap();
            session
                .active()
                .unwrap()
                .validate_base_facts(&mut c)
                .unwrap();
            let mut original = None;
            for payload in &mut session.active_mut().unwrap().payloads.values {
                match payload {
                    // The document row no longer names the tree's first base.
                    RetainedPayload::BaseDocument { first, .. } if case == 0 => {
                        original = Some(*first);
                        *first = None;
                    }
                    // The frozen base names a document without a base row.
                    RetainedPayload::Base {
                        frozen: BaseValue::Frozen(frozen),
                        ..
                    } if case == 1 => {
                        original = Some(Some(frozen.document));
                        frozen.document = base.index;
                    }
                    _ => {}
                }
            }
            let result = session.active().unwrap().validate_base_facts(&mut c);
            assert_eq!(result, Err(SessionError::Stop(registry_row)), "case {case}");
            assert_eq!(c.stop(), Some(registry_row), "case {case}");
            for payload in &mut session.active_mut().unwrap().payloads.values {
                match payload {
                    RetainedPayload::BaseDocument { first, .. } if case == 0 => {
                        *first = original.unwrap();
                    }
                    RetainedPayload::Base {
                        frozen: BaseValue::Frozen(frozen),
                        ..
                    } if case == 1 => {
                        frozen.document = original.unwrap().unwrap();
                    }
                    _ => {}
                }
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
        }
    }

    /// A frozen base whose environment facts come from another owner than the
    /// document's base row cannot have been frozen against that document: the
    /// two retained rows disagree about whose facts they hold.
    #[test]
    fn validating_a_frozen_base_with_another_owners_facts_reports_the_registry_row() {
        use crate::parser::control::HtmlParseInvariant;
        let registry_row = HtmlParseStop::InternalInvariant(HtmlParseInvariant::RegistryRow);
        let other = request("<base href=A>");
        let foreign = other.base_facts().unwrap().clone();
        let request = request("<base href=A>");
        assert!(!request.base_facts().unwrap().same_owner(&foreign));
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let base = node(&mut session, &mut c, &mut lexer);
        session
            .insert_parser(&mut c, &root, &base, None, &capability)
            .unwrap();
        session
            .active()
            .unwrap()
            .validate_base_facts(&mut c)
            .unwrap();
        let mut original = None;
        for payload in &mut session.active_mut().unwrap().payloads.values {
            if let RetainedPayload::Base {
                frozen: BaseValue::Frozen(frozen),
                ..
            } = payload
            {
                original = Some(mem::replace(&mut frozen.environment, foreign.clone()));
            }
        }
        let result = session.active().unwrap().validate_base_facts(&mut c);
        assert_eq!(result, Err(SessionError::Stop(registry_row)));
        assert_eq!(c.stop(), Some(registry_row));
        for payload in &mut session.active_mut().unwrap().payloads.values {
            if let RetainedPayload::Base {
                frozen: BaseValue::Frozen(frozen),
                ..
            } = payload
            {
                frozen.environment = original.take().unwrap();
            }
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    /// An unavailable base names the missing-policy row that made it so. A row
    /// of another kind, or one that is not a missing-policy row at all, is a
    /// retained row that is not what the base says it is.
    #[test]
    fn validating_an_unavailable_base_whose_missing_row_is_not_a_csp_row_reports_the_registry_row()
    {
        use crate::parser::control::HtmlParseInvariant;
        let registry_row = HtmlParseStop::InternalInvariant(HtmlParseInvariant::RegistryRow);
        for case in 0..2 {
            let request = request(
                "<head><base href=A><meta http-equiv=content-security-policy content=policy><i href=B>",
            );
            let (mut c, mut session, capability) =
                super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let head = node(&mut session, &mut c, &mut lexer);
            session
                .insert_parser(&mut c, &root, &head, None, &capability)
                .unwrap();
            let base = node(&mut session, &mut c, &mut lexer);
            session
                .insert_parser(&mut c, &head, &base, None, &capability)
                .unwrap();
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
            session
                .active()
                .unwrap()
                .validate_base_facts(&mut c)
                .unwrap();
            let base_slot = session
                .active()
                .unwrap()
                .base_slot(&mut c, base.index)
                .unwrap();
            let storage = session.active_mut().unwrap();
            let RetainedPayload::Base {
                frozen: BaseValue::Unavailable { missing, .. },
                ..
            } = &mut storage.payloads.values[base_slot]
            else {
                panic!("the policy recomputation left the base unavailable")
            };
            let named = *missing;
            if case == 0 {
                // The named row is a missing-policy row of another kind.
                let RetainedPayload::MetaMissing { kind, .. } = &mut storage.payloads.values[named]
                else {
                    panic!("the base names its missing-policy row")
                };
                *kind = meta::HtmlParseMetaMissingKind::ThemeColor;
            } else {
                // The named row is not a missing-policy row at all.
                *missing = base_slot;
            }
            let result = session.active().unwrap().validate_base_facts(&mut c);
            assert_eq!(result, Err(SessionError::Stop(registry_row)), "case {case}");
            assert_eq!(c.stop(), Some(registry_row), "case {case}");
            // The registry's own join check runs at release, so restore the rows.
            let storage = session.active_mut().unwrap();
            if let RetainedPayload::Base {
                frozen: BaseValue::Unavailable { missing, .. },
                ..
            } = &mut storage.payloads.values[base_slot]
            {
                *missing = named;
            }
            if let RetainedPayload::MetaMissing { kind, .. } = &mut storage.payloads.values[named] {
                *kind = meta::HtmlParseMetaMissingKind::Csp;
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
        }
    }

    #[test]
    fn real_insert_reorder_remove_href_change_and_clone_bind_the_immediate_first() {
        let request = request("<base href=A><base href=B><i href=C>");
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
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
    fn original_base_wrapper_association_rows_exact_one_short_and_cleanup() {
        // The wrapper retains a measured number of owned logical rows (one
        // Document summary, one occurrence receiver and their registry
        // entries); the Base input and provider already belong to this
        // original owner.
        let needed = {
            let request = request("<base href=A>");
            inserted_base_fixture!(request, c, session, root, lexer, base);
            let storage = session.active_mut().unwrap();
            storage
                .base_change_inner(&mut c, base.index, None, false, None)
                .unwrap();
            let prior = c
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current;
            storage
                .base_change(&mut c, base.index, None, false, None)
                .unwrap();
            let needed = c
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current
                - prior;
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
            needed
        };
        for short in [false, true] {
            let request = request("<base href=A>");
            inserted_base_fixture!(request, c, session, root, lexer, base);
            let storage = session.active_mut().unwrap();
            // Finish the existing intrinsic owner first.
            storage
                .base_change_inner(&mut c, base.index, None, false, None)
                .unwrap();
            let prior = c
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current;
            // Earlier tokenizer bootstrap has a larger transient logical peak
            // than this retained prefix. Keep that original prefix and cap, and
            // use real controlled fixture storage to select the later boundary.
            let mut pressure = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
            let available = needed - u64::from(short);
            let pressure_rows = request.limits().auxiliary_entries() - prior - available;
            for _ in 0..pressure_rows {
                pressure.try_push(&mut c, 0u8).unwrap();
            }
            let result = storage.base_change(&mut c, base.index, None, false, None);
            if short {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        ..
                    }))
                ));
                let stop = c.stop();
                assert_eq!(
                    storage
                        .base_change(&mut c, base.index, None, false, None)
                        .err(),
                    stop.map(SessionError::Stop)
                );
                assert_eq!(c.stop(), stop);
                assert!(
                    !storage
                        .payloads
                        .values
                        .iter()
                        .any(|row| matches!(row, RetainedPayload::AssociationOccurrence(_)))
                );
            } else {
                result.unwrap();
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    prior + pressure_rows + needed
                );
                assert_eq!(
                    storage
                        .payloads
                        .values
                        .iter()
                        .filter(|row| matches!(row, RetainedPayload::AssociationOccurrence(_)))
                        .count(),
                    1
                );
            }
            assert_value(&session, &base, "https://base.test/A");
            pressure.release(&mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
        }
    }

    #[test]
    fn actual_first_href_exact_one_short_and_pre_cancel_preserve_binding_and_cleanup() {
        // The immediate intrinsic base hook's whole cost is measured from an
        // unconstrained twin; the exact budget completes it and one credit less
        // stops it with the binding still pending.
        let total = {
            let request = request("<base href=A>");
            inserted_base_fixture!(request, c, session, root, lexer, base);
            let storage = session.active_mut().unwrap();
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            storage
                .base_change_inner(&mut c, base.index, None, false, None)
                .unwrap();
            let total = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
            assert_value(&session, &base, "https://base.test/A");
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
            total
        };
        for short in [false, true] {
            let request = request("<base href=A>");
            // Real pre-insert owner commits topology/effect retention. This test
            // then selects its immediate intrinsic base hook, not a public
            // insertion total or an invented checkpoint.
            inserted_base_fixture!(request, c, session, root, lexer, base);
            let storage = session.active_mut().unwrap();
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
            let remaining = request.limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                HtmlParseWorkCategory::StateStep,
                remaining - total + u64::from(short),
            )
            .unwrap();
            let result = storage.base_change_inner(&mut c, base.index, None, false, None);
            if short {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
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
                        .base_change(&mut c, base.index, None, false, None)
                        .err(),
                    stop.map(SessionError::Stop)
                );
                assert_eq!(c.stop(), stop);
            } else {
                assert_eq!(result, Ok(()));
                assert_value(&session, &base, "https://base.test/A");
                assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
        }
        let request = request("<base href=A>");
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let base = node(&mut session, &mut c, &mut lexer);
        request.cancellation().cancel();
        assert_eq!(
            session
                .insert_parser(&mut c, &root, &base, None, &capability)
                .err(),
            Some(SessionError::Stop(HtmlParseStop::Cancelled))
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
    fn original_cancel_after_frozen_assignment_prevents_response_and_publication() {
        let request = request("<base href=A>");
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let base = node(&mut session, &mut c, &mut lexer);
        CANCEL_AFTER_BINDING.with(|slot| *slot.borrow_mut() = Some(request.cancellation().clone()));
        let result = session.insert_parser(&mut c, &root, &base, None, &capability);
        assert_eq!(result, Err(SessionError::Stop(HtmlParseStop::Cancelled)));
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
            Some(SessionError::Stop(HtmlParseStop::Cancelled))
        );
        assert_eq!(
            session.insert_parser(&mut c, &root, &base, None, &capability),
            Err(SessionError::Stop(HtmlParseStop::Cancelled))
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
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
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
            ConstructionNodeKind::Element(element) => element
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
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
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
                kind: meta::HtmlParseMetaMissingKind::Csp,
                ..
            }
        ));
        assert_eq!(first(&session), Some(base.index));
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
}

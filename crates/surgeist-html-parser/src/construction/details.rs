//! HTML877d Details source-ordered hooks over the canonical ordinary tree.
use super::*;
use surgeist_html::construction::ConstructionNodeKind;

#[derive(Clone, Copy)]
pub(super) enum NotificationDomain {
    ClosedOwned,
    Unavailable,
}

impl SessionStorage {
    pub(super) fn details_domain(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<NotificationDomain, SessionError> {
        if let Some(slot) = self.payloads.joined(c, node, Join::Details)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::Details {
                node: Some(owner),
                notification,
            }) = self.payloads.values.get(slot)
                && *owner == node
            {
                c.charge(
                    HtmlParseWorkCategory::CopyByte,
                    mem::size_of::<NotificationDomain>() as u64,
                )?;
                return Ok(*notification);
            }
        }
        Ok(NotificationDomain::Unavailable)
    }
    pub(super) fn require_details_notification(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some(reason) = self.standalone_ineligibility {
            return Err(c.stop_unsupported_standalone(reason).into());
        }
        if matches!(
            self.details_domain(c, node)?,
            NotificationDomain::ClosedOwned
        ) {
            // Issued by a qualified real construction capability and bound to
            // this actual node, including owned fragment/template/clone trees.
            // No consumer observes a notification tracker or queued task in
            // this closed owner; canonical attribute mutations remain required.
            Ok(())
        } else {
            Err(c
                .stop_unsupported_standalone(HtmlParseUnsupportedState::DetailsNotification)
                .into())
        }
    }
    pub(super) fn initial_details_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let Some(attr) = element.attributes().at(c, position)? else {
            return Err(latch_registry_row(c));
        };
        if attr.name().namespace().uri().is_some() {
            return Ok(());
        }
        let attribute = if exact_ascii(c, Some(attr.name().local_name()), "open")? {
            surgeist_html::elements::details::HtmlDetailsAttribute::Open
        } else if exact_ascii(c, Some(attr.name().local_name()), "name")? {
            surgeist_html::elements::details::HtmlDetailsAttribute::Name
        } else {
            surgeist_html::elements::details::HtmlDetailsAttribute::Other
        };
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        payload_write::<surgeist_html::elements::details::HtmlDetailsAttributeEffect>(c, 1)?;
        let value = attr.value().units(c)?;
        let effect = surgeist_html::elements::details::details_attribute_effect(
            attr.name().namespace(),
            attribute,
            None,
            Some(value),
        );
        self.details_change(c, node, effect)
    }
    pub(super) fn details_change(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        effect: surgeist_html::elements::details::HtmlDetailsAttributeEffect,
    ) -> Result<(), SessionError> {
        match effect {
            surgeist_html::elements::details::HtmlDetailsAttributeEffect::None => Ok(()),
            surgeist_html::elements::details::HtmlDetailsAttributeEffect::CloseGiven => self
                .start_details(
                    c,
                    node,
                    surgeist_html::elements::details::HtmlDetailsExclusivity::CloseGiven,
                ),
            surgeist_html::elements::details::HtmlDetailsAttributeEffect::Notify {
                close_other,
                ..
            } => {
                self.require_details_notification(c, node)?;
                if close_other {
                    self.start_details(
                        c,
                        node,
                        surgeist_html::elements::details::HtmlDetailsExclusivity::CloseOther,
                    )?;
                }
                Ok(())
            }
        }
    }
    pub(super) fn start_details(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        close: surgeist_html::elements::details::HtmlDetailsExclusivity,
    ) -> Result<(), SessionError> {
        if self.form_attribute(c, node, "open")?.is_none() {
            return Ok(());
        }
        let Some(name) = self.form_attribute(c, node, "name")? else {
            return Ok(());
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if name.is_empty() {
            return Ok(());
        }
        let root = self.ordinary_root(c, node, None)?;
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
        self.start_effect(
            c,
            effects::EffectFrame::Details {
                node,
                close,
                root,
                next: Some(root),
            },
        )
    }
    /// One step of the details name group walk (HTML 877d6146487f, the details element:
    /// opening one closes the others of its name in its tree). The walk visits every node of
    /// the tree for each open details with a name, so n of distinct names cost record visits
    /// quadratic in n: 5.2 million at 500, 16.0 million at 1,000, 54.0 million at 2,000 and
    /// 196.0 million at 4,000. It stays a walk, as the group definition is written and in the
    /// same order of charges: an index of open details by name would have to follow every
    /// move, rename and attribute change, and a stopped parse reports the first charge reached.
    /// (docs/reference/html-operations.md, "group-walk-cost")
    pub(super) fn step_details(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        close: surgeist_html::elements::details::HtmlDetailsExclusivity,
        root: usize,
        next: Option<usize>,
    ) -> Result<(), SessionError> {
        let Some(other) = next else {
            self.write_current_effect(c, None)?;
            return Ok(());
        };
        let matches = if other != node && self.html_local(c, other, "details")? {
            match (
                self.form_attribute(c, node, "name")?,
                self.form_attribute(c, other, "name")?,
            ) {
                (Some(left), Some(right)) => exact_string_equal(c, left, right)?,
                _ => false,
            }
        } else {
            false
        };
        if matches && self.form_attribute(c, other, "open")?.is_some() {
            let target = match close {
                surgeist_html::elements::details::HtmlDetailsExclusivity::CloseGiven => node,
                surgeist_html::elements::details::HtmlDetailsExclusivity::CloseOther => other,
            };
            self.require_details_notification(c, target)?;
            self.remove_details_open(c, target)?;
            self.write_current_effect(c, None)?;
            return Ok(());
        }
        let next = self.next_view_node(c, root, other, None)?;
        self.write_current_effect(
            c,
            Some(effects::EffectFrame::Details {
                node,
                close,
                root,
                next,
            }),
        )?;
        Ok(())
    }
    /// Remove the no-namespace open attribute.
    fn remove_details_open(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        self.remove_managed_field(c, node, "open")?;
        self.details_change(
            c,
            node,
            surgeist_html::elements::details::HtmlDetailsAttributeEffect::Notify {
                old: surgeist_html::elements::details::HtmlDetailsState::Open,
                new: surgeist_html::elements::details::HtmlDetailsState::Closed,
                close_other: false,
            },
        )
    }
}

#[cfg(test)]
impl ParserArenaSession {
    pub(crate) fn set_details_attribute(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        input: &mut Option<crate::parser::tokenizer::token::Attribute>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        self.set_managed_attribute(c, node, input, capability, mutation::ManagedFamily::Details)
    }
    pub(crate) fn remove_details_attribute(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        attribute: surgeist_html::elements::details::HtmlDetailsAttribute,
    ) -> Result<(), SessionError> {
        let local = match attribute {
            surgeist_html::elements::details::HtmlDetailsAttribute::Open => "open",
            surgeist_html::elements::details::HtmlDetailsAttribute::Name => "name",
            _ => return Err(SessionError::WrongToken),
        };
        self.remove_managed_attribute(c, node, local, mutation::ManagedFamily::Details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture, token::Event};
    use surgeist_html::{construction::ConstructionNodeKind, values::DocumentString};

    fn request(source: &str) -> crate::HtmlDocumentParseRequest {
        crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source),
            super::super::tests::limits(),
        )
    }
    fn next_tag(
        lexer: &mut LexerFixture<'_>,
        c: &mut ParseControl,
    ) -> (Tag, crate::HtmlSourceRange) {
        loop {
            let event = lexer.next(c, false).unwrap().unwrap();
            match event {
                Event::Token(event) => {
                    let token = event.into_token(c).unwrap();
                    if let TokenKind::Tag(tag) = token.kind {
                        return (tag, token.source);
                    }
                    release_token_kind(&mut Some(token.kind), c).unwrap();
                }
                event => event.release(c).unwrap(),
            }
        }
    }
    fn detail(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        lexer: &mut LexerFixture<'_>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
        parent: &ParserNode,
    ) -> ParserNode {
        let (tag, range) = next_tag(lexer, c);
        session
            .allocate_details_for_creation(
                c,
                &mut Some(tag),
                CreationLocation {
                    intended_parent: parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                capability,
            )
            .unwrap()
    }
    fn attributes(session: &ParserArenaSession, node: &ParserNode) -> Vec<(Vec<u16>, Vec<u16>)> {
        let ConstructionNodeKind::Element(element) = &session
            .active()
            .unwrap()
            .construction
            .node_at(node.index)
            .expect("retained construction position")
            .kind()
        else {
            unreachable!()
        };
        test_support::attribute_units(element)
    }
    fn attribute(
        lexer: &mut LexerFixture<'_>,
        c: &mut ParseControl,
    ) -> Option<crate::parser::tokenizer::token::Attribute> {
        let (tag, _) = next_tag(lexer, c);
        let Tag {
            name,
            mut attributes,
            ..
        } = tag;
        name.release(c).unwrap();
        let value = attributes.try_pop(c).unwrap();
        release_token_attributes(attributes, c).unwrap();
        value
    }
    fn zero(c: &ParseControl) {
        super::super::tests::storage_zero(c);
    }

    #[test]
    fn a_join_naming_another_row_leaves_the_details_domain_unavailable() {
        let request = request("<details>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
        let storage = session.active_mut().unwrap();
        assert!(matches!(
            storage.details_domain(&mut c, node.index),
            Ok(NotificationDomain::ClosedOwned)
        ));
        let good = storage.payloads.node_joins[node.index];
        // Row 0 is not a details row.
        *storage.payloads.node_joins[node.index]
            .rows
            .get_mut(Join::Details) = Some(joined_cell(0).unwrap());
        assert!(matches!(
            storage.details_domain(&mut c, node.index),
            Ok(NotificationDomain::Unavailable)
        ));
        assert!(
            storage
                .require_details_notification(&mut c, node.index)
                .is_err()
        );
        storage.payloads.node_joins[node.index] = good;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn details_insert_before_existing_closes_given_and_preserves_remaining_order() {
        let request = request("<details a=1 open=v b=2 name=g c=3><details name=g open>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let parent = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let first = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
        session
            .insert(&mut c, &parent, &first, None, Some(&proof))
            .unwrap();
        let second = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
        session
            .insert(&mut c, &parent, &second, Some(&first), Some(&proof))
            .unwrap();
        assert_eq!(
            attributes(&session, &second),
            vec![(vec![110, 97, 109, 101], vec![103])]
        );
        session
            .remove_details_attribute(
                &mut c,
                &first,
                surgeist_html::elements::details::HtmlDetailsAttribute::Open,
            )
            .unwrap();
        assert_eq!(
            attributes(&session, &first),
            vec![
                (vec![97], vec![49]),
                (vec![98], vec![50]),
                (vec![110, 97, 109, 101], vec![103]),
                (vec![99], vec![51])
            ]
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }
    #[test]
    fn details_name_change_closes_given_but_later_open_closes_other() {
        let request =
            request("<details name=g open><details name=h open><i name=g><i open=x><i open=y>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let parent = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let first = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
        session
            .insert(&mut c, &parent, &first, None, Some(&proof))
            .unwrap();
        let second = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
        session
            .insert(&mut c, &parent, &second, None, Some(&proof))
            .unwrap();
        let mut name = attribute(&mut lexer, &mut c);
        session
            .set_details_attribute(&mut c, &second, &mut name, &proof)
            .unwrap();
        assert!(name.is_none());
        assert_eq!(
            attributes(&session, &second),
            vec![(vec![110, 97, 109, 101], vec![103])]
        );
        let mut open = attribute(&mut lexer, &mut c);
        session
            .set_details_attribute(&mut c, &second, &mut open, &proof)
            .unwrap();
        assert_eq!(
            attributes(&session, &first),
            vec![(vec![110, 97, 109, 101], vec![103])]
        );
        assert_eq!(
            attributes(&session, &second),
            vec![
                (vec![110, 97, 109, 101], vec![103]),
                (vec![111, 112, 101, 110], vec![120])
            ]
        );
        // Present-to-present changes do not close the same member or create a
        // notification transition. The actual replacement remains in its slot.
        let mut open = attribute(&mut lexer, &mut c);
        session
            .set_details_attribute(&mut c, &second, &mut open, &proof)
            .unwrap();
        assert_eq!(attributes(&session, &second)[1].1, vec![121]);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }
    #[test]
    fn details_empty_exact_names_and_distinct_ordinary_roots_do_not_close() {
        for (left, right) in [("", ""), ("g", "G"), (" g", "g")] {
            let request = request(&format!(
                "<details name='{left}' open><details name='{right}' open>"
            ));
            let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let a = detail(&mut session, &mut c, &mut lexer, &proof, &root);
            let b = detail(&mut session, &mut c, &mut lexer, &proof, &root);
            // Detached nodes are separate ordinary trees. Connectivity itself
            // is not the membership predicate; insertion makes their tree same.
            assert_eq!(attributes(&session, &a).len(), 2);
            assert_eq!(attributes(&session, &b).len(), 2);
            session
                .insert(&mut c, &root, &a, None, Some(&proof))
                .unwrap();
            session
                .insert(&mut c, &root, &b, None, Some(&proof))
                .unwrap();
            assert_eq!(attributes(&session, &a).len(), 2);
            assert_eq!(attributes(&session, &b).len(), 2);
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            zero(&c);
        }
    }
    #[test]
    fn details_removal_each_work_cut_retains_unique_cleanup_and_first_stop() {
        let source = "<details a=1 open=v b=2 name=g c=3>";
        let request = self::request(source);
        let (mut baseline, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut baseline).unwrap();
        let mut lexer = LexerFixture::new(
            request.source(),
            HtmlTokenizerState::Data,
            None,
            &mut baseline,
        )
        .unwrap();
        let node = detail(&mut session, &mut baseline, &mut lexer, &proof, &root);
        let before = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session
            .remove_details_attribute(
                &mut baseline,
                &node,
                surgeist_html::elements::details::HtmlDetailsAttribute::Open,
            )
            .unwrap();
        let cost = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        lexer.release(&mut baseline).unwrap();
        session.release(&mut baseline).unwrap();
        zero(&baseline);
        for admitted in 0..=cost {
            let request = self::request(source);
            let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
            let remaining = request.limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(HtmlParseWorkCategory::StateStep, remaining - admitted)
                .unwrap();
            let result = session.remove_details_attribute(
                &mut c,
                &node,
                surgeist_html::elements::details::HtmlDetailsAttribute::Open,
            );
            if admitted == cost {
                assert_eq!(result, Ok(()));
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
                let stop = c.stop();
                assert_eq!(session.freeze(&mut c).err(), stop.map(SessionError::Stop));
                assert_eq!(c.stop(), stop);
            }
            if c.stop().is_none() {
                assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            zero(&c);
        }
    }
    #[test]
    fn details_managed_clone_owns_independent_attributes_and_notification_domain() {
        let request = request("<details name=g open a=x>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
        session
            .insert(&mut c, &root, &node, None, Some(&proof))
            .unwrap();
        let storage = session.active_mut().unwrap();
        let document = storage.node_document_ref(&mut c, node.index).unwrap();
        let copy = storage.clone_node(&mut c, node.index, document).unwrap();
        let copy = storage.node(&mut c, copy).unwrap();
        assert_eq!(attributes(&session, &copy), attributes(&session, &node));
        session
            .insert(&mut c, &root, &copy, None, Some(&proof))
            .unwrap();
        assert_eq!(
            attributes(&session, &copy),
            vec![(vec![110, 97, 109, 101], vec![103]), (vec![97], vec![120])]
        );
        assert_eq!(attributes(&session, &node).len(), 3);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }
    #[test]
    fn details_template_contents_is_a_distinct_ordinary_group_tree() {
        let request = request("<details name=g open><template><details name=g open x:open=q>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let outer = detail(&mut session, &mut c, &mut lexer, &proof, &root);
        session
            .insert(&mut c, &root, &outer, None, Some(&proof))
            .unwrap();
        let (tag, _) = next_tag(&mut lexer, &mut c);
        let template = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
        session
            .insert(&mut c, &root, &template, None, Some(&proof))
            .unwrap();
        let contents = session
            .template_contents(&mut c, &template)
            .unwrap()
            .unwrap();
        let inner = detail(&mut session, &mut c, &mut lexer, &proof, &contents);
        session
            .insert(&mut c, &contents, &inner, None, Some(&proof))
            .unwrap();
        assert_eq!(attributes(&session, &outer).len(), 2);
        assert_eq!(
            attributes(&session, &inner),
            vec![
                (vec![110, 97, 109, 101], vec![103]),
                (vec![111, 112, 101, 110], vec![]),
                (vec![120, 58, 111, 112, 101, 110], vec![113])
            ]
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }
    #[test]
    fn details_retired_originals_and_current_attribute_destinations_survive_removal() {
        let request = request("<details a=1 open=v b=2 name=g c=3>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
        session
            .remove_details_attribute(
                &mut c,
                &node,
                surgeist_html::elements::details::HtmlDetailsAttribute::Open,
            )
            .unwrap();
        // The original open/v allocations and sources are immutable history;
        // removing their current destination must not retire a surviving b/2.
        let storage = session.active().unwrap();
        let mut removed = Vec::new();
        for (slot, payload) in storage.payloads.values.iter().enumerate() {
            if let RetainedPayload::String {
                value, location, ..
            } = payload
            {
                let group = storage
                    .payloads
                    .row_group(slot)
                    .unwrap()
                    .expect("every string of the details element has an origin group");
                let evidence = storage.evidence.as_ref().unwrap();
                let header = evidence.require_group(group).unwrap();
                if value.code_units() == [111, 112, 101, 110] || value.code_units() == [118] {
                    assert!(matches!(location, FieldRef::Detached));
                    assert!(header.retirement.is_some());
                    removed.push(value.code_units().to_vec());
                }
            }
        }
        // The surviving attributes hold their groups at the places they have now, and the leaf of
        // each names that place.
        let records = ConstructionView::Active(&storage.construction);
        let mut current = Vec::new();
        for position in 0..4 {
            let field = FieldRef::AttributeName(node.index, position);
            let group = storage
                .payloads
                .detail_origins
                .get(&mut c, field)
                .unwrap()
                .expect("every attribute of the details element has an origin group");
            let header = storage
                .evidence
                .as_ref()
                .unwrap()
                .require_group(group)
                .unwrap();
            assert!(header.retirement.is_none());
            let provenance::GroupBody::Leaf { value, .. } = header.body else {
                panic!("the name of an attribute is a leaf of tokenizer origins")
            };
            let provenance::LeafValue::Attribute {
                position: named, ..
            } = value
            else {
                panic!("a live attribute string is read from the attributes")
            };
            assert_eq!(named as usize, position);
            if field.units(records) == Some(&[98]) {
                current.push(position);
            }
        }
        assert_eq!(removed, vec![vec![111, 112, 101, 110], vec![118]]);
        assert_eq!(current, vec![1]);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }
    #[test]
    fn details_setter_every_work_cut_reclaims_staged_real_fields() {
        for initial in ["name=g", "name=g open=old"] {
            let source = format!("<details {initial}><i open=new>");
            let request = request(&source);
            let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
            let mut input = attribute(&mut lexer, &mut c);
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            session
                .set_details_attribute(&mut c, &node, &mut input, &proof)
                .unwrap();
            let cost = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            zero(&c);
            for admitted in 0..=cost {
                let request = self::request(&source);
                let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
                let root = session.root(&mut c).unwrap();
                let mut lexer =
                    LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                        .unwrap();
                let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
                let mut input = attribute(&mut lexer, &mut c);
                let remaining = request.limits().work_credits()
                    - c.usage().resource(HtmlParseResource::WorkCredits).current;
                c.charge(HtmlParseWorkCategory::StateStep, remaining - admitted)
                    .unwrap();
                let result = session.set_details_attribute(&mut c, &node, &mut input, &proof);
                if admitted == cost {
                    assert_eq!(result, Ok(()));
                } else {
                    assert!(result.is_err());
                    assert!(c.stop().is_some());
                }
                if c.stop().is_none() {
                    assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
                }
                if let Some(attr) = input.take() {
                    attr.name.release(&mut c).unwrap();
                    attr.value.release(&mut c).unwrap();
                }
                lexer.release(&mut c).unwrap();
                session.release(&mut c).unwrap();
                zero(&c);
            }
        }
    }
    #[test]
    fn details_true_completed_driver_projects_retired_open_original_source() {
        use crate::parser::tree_builder::{StandaloneDocumentDriver, head::HeadStep};
        let request = request(
            "<!doctype html><details name=g open></details><details name=g open></details>",
        );
        let mut driver = Some(crate::parser::tree_builder::entry::initialized_driver(
            &request,
        ));
        let mut complete = false;
        for _ in 0..1000 {
            if driver.as_mut().unwrap().step().unwrap() == HeadStep::Complete {
                complete = true;
                break;
            }
        }
        assert!(complete);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
        let expected_start = 62;
        let mut retired = Vec::new();
        {
            let mut rows = completed.native_associations().unwrap();
            let mut group = 0;
            while let Some(header) = rows.group(group).unwrap() {
                if header.retirement.is_some() {
                    let mut text = Some(rows.materialize_group(group).unwrap());
                    let units = text.as_ref().unwrap().text().code_units().to_vec();
                    let mut cursor = Some(rows.group_origin_cursor(group).unwrap());
                    let mut origins = Vec::new();
                    while let Some(origin) =
                        rows.next_group_origin(cursor.as_mut().unwrap()).unwrap()
                    {
                        origins.push((origin.source.start(), origin.source.end()));
                    }
                    rows.release_group_origin_cursor(&mut cursor).unwrap();
                    rows.release_materialized_group(&mut text).unwrap();
                    retired.push((units, origins));
                }
                group += 1;
            }
        }
        completed.release().unwrap();
        assert_eq!(
            retired,
            vec![
                (
                    vec![111, 112, 101, 110],
                    vec![
                        (expected_start, expected_start + 1),
                        (expected_start + 1, expected_start + 2),
                        (expected_start + 2, expected_start + 3),
                        (expected_start + 3, expected_start + 4)
                    ]
                ),
                (vec![], vec![])
            ]
        );
    }
    #[test]
    fn details_names_preserve_literal_utf16_group_equality() {
        let mut units = "<details name='".encode_utf16().collect::<Vec<_>>();
        units.extend([0xd800, 0xd83d, 0xde00]);
        units.extend("' open><details name='".encode_utf16());
        units.extend([0xd800, 0xd83d, 0xde00]);
        units.extend("' open>".encode_utf16());
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf16(DocumentString::from_utf16(units)),
            super::super::tests::limits(),
        );
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let first = detail(&mut session, &mut c, &mut lexer, &proof, &root);
        let second = detail(&mut session, &mut c, &mut lexer, &proof, &root);
        session
            .insert(&mut c, &root, &first, None, Some(&proof))
            .unwrap();
        session
            .insert(&mut c, &root, &second, None, Some(&proof))
            .unwrap();
        assert_eq!(
            attributes(&session, &second),
            vec![(vec![110, 97, 109, 101], vec![0xd800, 0xd83d, 0xde00])]
        );
        assert_eq!(attributes(&session, &first).len(), 2);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }
    #[test]
    fn details_foreign_and_pre_cancelled_setter_keep_incoming_owner() {
        for cancelled in [false, true] {
            let request = request("<details name=g><i open=new>");
            let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
            let mut input = attribute(&mut lexer, &mut c);
            let pointer = input
                .as_ref()
                .unwrap()
                .value
                .value
                .text()
                .code_units()
                .as_ptr();
            if cancelled {
                request.cancellation().cancel();
                assert_eq!(
                    session.set_details_attribute(&mut c, &node, &mut input, &proof),
                    Err(SessionError::Stop(HtmlParseStop::Cancelled))
                );
            } else {
                let before = c.usage().clone();
                let mut foreign = ParseControl::new(
                    super::super::tests::limits(),
                    crate::HtmlParseCancellation::new(),
                );
                assert!(matches!(
                    session.set_details_attribute(&mut foreign, &node, &mut input, &proof),
                    Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
                        crate::parser::control::HtmlParseInvariant::ForeignControl
                    )))
                ));
                assert_eq!(c.usage(), before);
                zero(&foreign);
            }
            assert_eq!(
                input
                    .as_ref()
                    .unwrap()
                    .value
                    .value
                    .text()
                    .code_units()
                    .as_ptr(),
                pointer
            );
            assert_eq!(
                attributes(&session, &node),
                vec![(vec![110, 97, 109, 101], vec![103])]
            );
            let attr = input.take().unwrap();
            attr.name.release(&mut c).unwrap();
            attr.value.release(&mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            zero(&c);
        }
    }

    #[test]
    fn details_setter_exact_and_one_short_managed_slot_admission() {
        for cap in [1, 2] {
            let limits = super::super::tests::limits()
                .into_builder()
                .managed_attributes(cap)
                .finish()
                .unwrap();
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<details name=g><i open=new>"),
                limits,
            );
            let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
            let mut input = attribute(&mut lexer, &mut c);
            // The actual details owns one native name slot. Appending open
            // requires exactly one additional managed slot, first staged.
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedAttributes)
                    .current,
                1
            );
            let result = session.set_details_attribute(&mut c, &node, &mut input, &proof);
            if cap == 2 {
                result.unwrap();
                assert_eq!(
                    attributes(&session, &node),
                    vec![
                        (vec![110, 97, 109, 101], vec![103]),
                        (vec![111, 112, 101, 110], vec![110, 101, 119])
                    ]
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedAttributes)
                        .current,
                    2
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedAttributes,
                        ..
                    }))
                ));
                assert_eq!(
                    attributes(&session, &node),
                    vec![(vec![110, 97, 109, 101], vec![103])]
                );
                let stop = c.stop().unwrap();
                assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            }
            if let Some(attr) = input.take() {
                attr.name.release(&mut c).unwrap();
                attr.value.release(&mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            zero(&c);
        }
    }
    #[test]
    fn details_dispatch_keeps_ordinary_self_closing_diagnostic() {
        use crate::parser::tree_builder::{
            StandaloneDocumentDriver,
            head::{HeadStep, HtmlTreeDiagnostic},
        };
        let request = request("<!doctype html><details name=g open/></details>");
        let mut driver = Some(crate::parser::tree_builder::entry::initialized_driver(
            &request,
        ));
        let mut complete = false;
        for _ in 0..1000 {
            if driver.as_mut().unwrap().step().unwrap() == HeadStep::Complete {
                complete = true;
                break;
            }
        }
        assert!(complete);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
        let evidence = completed.diagnostics().unwrap();
        assert_eq!(evidence.len(), 1);
        let (record, location) = evidence.retained_record(0).unwrap().unwrap();
        assert_eq!(
            record.kind,
            evidence::HtmlParseDiagnosticKind::Tree(HtmlTreeDiagnostic::UnacknowledgedSelfClosing)
        );
        assert_eq!(record.rule.branch, "ordinary-start/self-closing");
        assert_eq!((location.start(), location.end()), (15, 37));
        completed.release().unwrap();
    }
}

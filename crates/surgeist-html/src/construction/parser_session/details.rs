//! HTML877d Details source-ordered hooks over the canonical ordinary tree.
use super::*;

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
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::Details {
                node: Some(owner),
                notification,
            } = payload
                && *owner == node
            {
                c.charge(
                    WorkCategory::CopyByte,
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
        c.charge(WorkCategory::RecordVisit, 1)?;
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
                .stop_unsupported_standalone(StandaloneUnsupported::DetailsNotification)
                .into())
        }
    }
    pub(super) fn initial_details_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        let HtmlNodeKind::Element(element) = &self.records[node].kind else {
            return Err(SessionError::WrongToken);
        };
        let attr = &element.attributes().as_slice()[position];
        if attr.name().namespace().uri().is_some() {
            return Ok(());
        }
        let attribute = if exact_ascii(c, Some(attr.name().local_name()), "open")? {
            crate::HtmlDetailsAttribute::Open
        } else if exact_ascii(c, Some(attr.name().local_name()), "name")? {
            crate::HtmlDetailsAttribute::Name
        } else {
            crate::HtmlDetailsAttribute::Other
        };
        c.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<crate::HtmlDetailsAttributeEffect>(c, 1)?;
        let effect = crate::details_attribute_effect(
            attr.name().namespace(),
            attribute,
            None,
            Some(attr.value()),
        );
        self.details_change(c, node, effect)
    }
    pub(super) fn details_change(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        effect: crate::HtmlDetailsAttributeEffect,
    ) -> Result<(), SessionError> {
        match effect {
            crate::HtmlDetailsAttributeEffect::None => Ok(()),
            crate::HtmlDetailsAttributeEffect::CloseGiven => {
                self.start_details(c, node, crate::HtmlDetailsExclusivity::CloseGiven)
            }
            crate::HtmlDetailsAttributeEffect::Notify { close_other, .. } => {
                self.require_details_notification(c, node)?;
                if close_other {
                    self.start_details(c, node, crate::HtmlDetailsExclusivity::CloseOther)?;
                }
                Ok(())
            }
        }
    }
    pub(super) fn start_details(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        close: crate::HtmlDetailsExclusivity,
    ) -> Result<(), SessionError> {
        if self.form_attribute(c, node, "open")?.is_none() {
            return Ok(());
        }
        let Some(name) = self.form_attribute(c, node, "name")? else {
            return Ok(());
        };
        c.charge(WorkCategory::RecordVisit, 1)?;
        if name.is_empty() {
            return Ok(());
        }
        let root = self.ordinary_root(c, node, None)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if root == node && self.records[node].children.is_empty() {
            return Ok(());
        }
        self.start_effect(
            c,
            select::EffectFrame::Details {
                node,
                close,
                root,
                next: Some(root),
            },
        )
    }
    pub(super) fn step_details(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        close: crate::HtmlDetailsExclusivity,
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
                crate::HtmlDetailsExclusivity::CloseGiven => node,
                crate::HtmlDetailsExclusivity::CloseOther => other,
            };
            self.require_details_notification(c, target)?;
            self.remove_details_open(c, target)?;
            self.write_current_effect(c, None)?;
            return Ok(());
        }
        let next = self.next_view_node(c, root, other, None)?;
        self.write_current_effect(
            c,
            Some(select::EffectFrame::Details {
                node,
                close,
                root,
                next,
            }),
        )?;
        Ok(())
    }
    /// Remove one actual no-namespace open slot without shifting an unpolled
    /// suffix. The existing registry retains both capacity and precise progress.
    fn remove_details_open(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        self.remove_managed_field(c, node, "open")?;
        self.details_change(
            c,
            node,
            crate::HtmlDetailsAttributeEffect::Notify {
                old: crate::HtmlDetailsState::Open,
                new: crate::HtmlDetailsState::Closed,
                close_other: false,
            },
        )
    }
    pub(super) fn edit_removing_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        container: usize,
        position: Option<usize>,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::ProviderCall, 2)?;
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        c.charge(
            WorkCategory::CopyByte,
            mem::size_of::<OrderedAttributes>() as u64,
        )?;
        payload_write::<usize>(c, 1)?;
        payload_write::<bool>(c, 1)?;
        let RetainedPayload::Attributes(storage) = &mut self.payloads.values[container] else {
            unreachable!()
        };
        let AttributeStorage::Removing {
            values,
            allocation,
            mut shared,
            element,
            index,
            mut shifted,
            mut popped,
            rebound,
        } = storage.take().expect("real removal")
        else {
            unreachable!()
        };
        drop(values);
        let HtmlNodeKind::Element(element_record) = &mut self.records[node].kind else {
            unreachable!()
        };
        let result = if let Some(position) = position {
            c.shift_attribute_slot(
                element_record.attributes.parser_backing_mut(),
                &mut shared,
                &allocation,
                position,
            )
        } else {
            c.pop_attribute_slot(
                element_record.attributes.parser_backing_mut(),
                &mut shared,
                &allocation,
            )
            .map(drop)
        };
        if result.is_ok() {
            if position.is_some() {
                shifted += 1;
            } else {
                popped = true;
            }
        }
        let values = element_record.attributes.clone();
        *storage = Some(AttributeStorage::Removing {
            values,
            allocation,
            shared,
            element,
            index,
            shifted,
            popped,
            rebound,
        });
        #[cfg(test)]
        if result.is_ok()
            && position.is_some()
            && let Some(cancellation) = self.cancel_after_details_shift.take()
        {
            cancellation.cancel();
        }
        result.map_err(Into::into)
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
        attribute: crate::HtmlDetailsAttribute,
    ) -> Result<(), SessionError> {
        let local = match attribute {
            crate::HtmlDetailsAttribute::Open => "open",
            crate::HtmlDetailsAttribute::Name => "name",
            _ => return Err(SessionError::WrongToken),
        };
        self.remove_managed_attribute(c, node, local, mutation::ManagedFamily::Details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::Event};

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
        let HtmlNodeKind::Element(element) = &session.active().unwrap().records[node.index].kind
        else {
            unreachable!()
        };
        element
            .attributes()
            .iter()
            .map(|a| {
                (
                    a.name().local_name().code_units().to_vec(),
                    a.value().code_units().to_vec(),
                )
            })
            .collect()
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
    fn details_insert_before_existing_closes_given_and_preserves_remaining_order() {
        let request = request("<details a=1 open=v b=2 name=g c=3><details name=g open>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let parent = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
            .remove_details_attribute(&mut c, &first, crate::HtmlDetailsAttribute::Open)
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
        session
            .active()
            .unwrap()
            .payloads
            .check_aliases(&session.active().unwrap().records, &mut c)
            .unwrap();
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
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
        let mut lexer =
            LexerFixture::new(request.source(), State::Data, None, &mut baseline).unwrap();
        let node = detail(&mut session, &mut baseline, &mut lexer, &proof, &root);
        let before = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session
            .remove_details_attribute(&mut baseline, &node, crate::HtmlDetailsAttribute::Open)
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
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
            let remaining = request.limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(WorkCategory::StateStep, remaining - admitted)
                .unwrap();
            let result =
                session.remove_details_attribute(&mut c, &node, crate::HtmlDetailsAttribute::Open);
            if admitted == cost {
                assert_eq!(result, Ok(()));
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
                let stop = c.stop();
                assert_eq!(session.freeze(&mut c).err(), stop.map(SessionError::Stop));
                assert_eq!(c.stop(), stop);
            }
            if c.stop().is_none() {
                assert!(c.charge(WorkCategory::StateStep, 1).is_err());
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            zero(&c);
        }
    }
    #[test]
    fn details_committed_shift_cancellation_keeps_phase_for_true_cleanup() {
        let request = request("<details open=v a=1 name=g b=2>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
        // Scheduling instrumentation only: no alternate operation or minted
        // phase. Cancel the real token after the actual adjacent slot commit.
        session.active_mut().unwrap().cancel_after_details_shift =
            Some(request.cancellation().clone());
        assert_eq!(
            session.remove_details_attribute(&mut c, &node, crate::HtmlDetailsAttribute::Open),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert!(
            session
                .active()
                .unwrap()
                .payloads
                .values
                .iter()
                .any(|payload| matches!(
                    payload,
                    RetainedPayload::Attributes(Some(AttributeStorage::Removing {
                        shifted: 1,
                        popped: false,
                        ..
                    }))
                ))
        );
        assert_eq!(
            attributes(&session, &node),
            vec![
                (vec![97], vec![49]),
                (vec![111, 112, 101, 110], vec![118]),
                (vec![110, 97, 109, 101], vec![103]),
                (vec![98], vec![50])
            ]
        );
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
    }
    #[test]
    fn details_alias_refusal_preserves_attributes_and_original_setter_input() {
        let request = request("<details name=g><i open=v>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
        let mut input = attribute(&mut lexer, &mut c);
        let HtmlNodeKind::Element(element) = &session.active().unwrap().records[node.index].kind
        else {
            unreachable!()
        };
        let alias = element.attributes().clone();
        let original = input
            .as_ref()
            .unwrap()
            .value
            .value
            .text()
            .shared_code_units()
            .as_ptr();
        assert_eq!(
            session.set_details_attribute(&mut c, &node, &mut input, &proof),
            Err(SessionError::AliasedPayload)
        );
        assert_eq!(
            input
                .as_ref()
                .unwrap()
                .value
                .value
                .text()
                .shared_code_units()
                .as_ptr(),
            original
        );
        assert_eq!(
            attributes(&session, &node),
            vec![(vec![110, 97, 109, 101], vec![103])]
        );
        drop(alias);
        session
            .set_details_attribute(&mut c, &node, &mut input, &proof)
            .unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }
    #[test]
    fn details_managed_clone_owns_independent_attributes_and_notification_domain() {
        let request = request("<details name=g open a=x>");
        let (mut c, mut session, proof) = super::super::tests::fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
        session
            .remove_details_attribute(&mut c, &node, crate::HtmlDetailsAttribute::Open)
            .unwrap();
        // The original open/v allocations and sources are immutable history;
        // removing their current destination must not retire a surviving b/2.
        let storage = session.active().unwrap();
        let mut removed = Vec::new();
        let mut current = Vec::new();
        for payload in &storage.payloads.values {
            if let RetainedPayload::String {
                value,
                location,
                origins: Some(group),
                ..
            } = payload
            {
                let evidence = storage.evidence.as_ref().unwrap();
                let header = evidence.require_group(group).unwrap();
                if value.code_units() == [111, 112, 101, 110] || value.code_units() == [118] {
                    assert!(matches!(location, StringLocation::Detached));
                    assert!(header.retirement.is_some());
                    removed.push(value.code_units().to_vec());
                }
                if let StringLocation::AttributeName(_, position) = location
                    && value.code_units() == [98]
                {
                    current.push(*position);
                }
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
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
                    LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
                let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
                let mut input = attribute(&mut lexer, &mut c);
                let remaining = request.limits().work_credits()
                    - c.usage().resource(HtmlParseResource::WorkCredits).current;
                c.charge(WorkCategory::StateStep, remaining - admitted)
                    .unwrap();
                let result = session.set_details_attribute(&mut c, &node, &mut input, &proof);
                if admitted == cost {
                    assert_eq!(result, Ok(()));
                } else {
                    assert!(result.is_err());
                    assert!(c.stop().is_some());
                }
                if c.stop().is_none() {
                    assert!(c.charge(WorkCategory::StateStep, 1).is_err());
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
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let node = detail(&mut session, &mut c, &mut lexer, &proof, &root);
            let mut input = attribute(&mut lexer, &mut c);
            let pointer = input
                .as_ref()
                .unwrap()
                .value
                .value
                .text()
                .shared_code_units()
                .as_ptr();
            if cancelled {
                request.cancellation().cancel();
                assert_eq!(
                    session.set_details_attribute(&mut c, &node, &mut input, &proof),
                    Err(SessionError::Stop(ParseStop::Cancelled))
                );
            } else {
                let before = c.usage().clone();
                let mut foreign = ParseControl::new(
                    super::super::tests::limits(),
                    crate::HtmlParseCancellation::new(),
                );
                assert!(matches!(
                    session.set_details_attribute(&mut foreign, &node, &mut input, &proof),
                    Err(SessionError::Stop(ParseStop::InternalInvariant(
                        crate::parser::control::Invariant::ForeignControl
                    )))
                ));
                assert_eq!(c.usage(), &before);
                zero(&foreign);
            }
            assert_eq!(
                input
                    .as_ref()
                    .unwrap()
                    .value
                    .value
                    .text()
                    .shared_code_units()
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
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
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
            head::{HeadCondition, HeadStep},
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
        let (record, location) = evidence.retained_record(0).unwrap();
        assert_eq!(
            record.kind,
            evidence::DiagnosticKind::Tree(HeadCondition::UnacknowledgedSelfClosing)
        );
        assert_eq!(record.rule.branch, "ordinary-start/self-closing");
        assert_eq!((location.start(), location.end()), (15, 37));
        completed.release().unwrap();
    }
}

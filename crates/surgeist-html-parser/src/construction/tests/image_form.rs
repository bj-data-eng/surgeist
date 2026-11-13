use super::*;
use surgeist_html::construction::{HtmlConstructionLimits, HtmlDocument, HtmlDocumentMode};

#[test]
fn image_creation_uses_actual_same_tree_and_preserves_nonancestor_parser_owner() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><img form=other><form><img>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
    session.insert(&mut c, &body, &form, None, None).unwrap();
    let image = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&form));
    assert_form_owner(&session, &mut c, &image, Some(&form), true);
    session.insert(&mut c, &body, &image, None, None).unwrap();
    assert_form_owner(&session, &mut c, &image, Some(&form), true);
    let detached_form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
    let other_image = form_creation(
        &mut session,
        &mut tokenizer,
        &mut c,
        &body,
        Some(&detached_form),
    );
    assert_form_owner(&session, &mut c, &other_image, None, false);
    session
        .insert(&mut c, &form, &other_image, None, None)
        .unwrap();
    assert_form_owner(&session, &mut c, &other_image, Some(&form), false);
    assert_eq!(
        session
            .image_form_state(&mut c, &other_image)
            .unwrap()
            .evidence,
        HtmlParseFormEvidence::Reset
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn image_remove_then_reinsert_resets_historical_owner_to_actual_nearest_form() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><form><img>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let first = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
    let second = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
    session.insert(&mut c, &body, &first, None, None).unwrap();
    session.insert(&mut c, &body, &second, None, None).unwrap();
    let image = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&first));
    session.insert(&mut c, &body, &image, None, None).unwrap();
    session.insert(&mut c, &second, &image, None, None).unwrap();
    assert_form_owner(&session, &mut c, &image, Some(&second), false);
    session.remove(&mut c, &image, None).unwrap();
    assert_form_owner(&session, &mut c, &image, None, false);
    session.insert(&mut c, &first, &image, None, None).unwrap();
    assert_form_owner(&session, &mut c, &image, Some(&first), false);
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn image_subtree_move_keeps_owner_when_form_and_image_share_removed_tree() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><img><div>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
    session.insert(&mut c, &body, &form, None, None).unwrap();
    let image = form_creation(&mut session, &mut tokenizer, &mut c, &form, Some(&form));
    session.insert(&mut c, &form, &image, None, None).unwrap();
    let div = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
    session.insert(&mut c, &body, &div, None, None).unwrap();
    session.insert(&mut c, &div, &form, None, None).unwrap();
    assert_form_owner(&session, &mut c, &image, Some(&form), true);
    session.remove(&mut c, &form, None).unwrap();
    assert_form_owner(&session, &mut c, &image, Some(&form), true);
    session.insert(&mut c, &body, &form, None, None).unwrap();
    assert_form_owner(&session, &mut c, &image, Some(&form), true);
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn image_same_parent_reorder_and_adoption_child_transfer_apply_real_resets() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><form><img><img>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let a = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
    let b = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
    session.insert(&mut c, &body, &a, None, None).unwrap();
    session.insert(&mut c, &body, &b, None, None).unwrap();
    let one = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&a));
    let two = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&a));
    session.insert(&mut c, &body, &one, None, None).unwrap();
    session.insert(&mut c, &body, &two, None, None).unwrap();
    session.insert(&mut c, &body, &one, None, None).unwrap();
    assert_form_owner(&session, &mut c, &one, None, false);
    session.insert(&mut c, &a, &one, None, None).unwrap();
    session.insert(&mut c, &a, &two, None, None).unwrap();
    session.reparent_children(&mut c, &a, &b, None).unwrap();
    assert_form_owner(&session, &mut c, &one, Some(&b), false);
    assert_form_owner(&session, &mut c, &two, Some(&b), false);
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(b.index)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>(),
        [one.index, two.index]
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn image_frozen_metadata_resolves_actual_native_identity_and_unavailable_is_not_null() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><img><img>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
    session.insert(&mut c, &body, &form, None, None).unwrap();
    let image = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&form));
    session.insert(&mut c, &body, &image, None, None).unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("img")
    };
    let raw = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
    session.insert(&mut c, &body, &raw, None, None).unwrap();
    assert_eq!(
        session.image_form_state(&mut c, &raw),
        Err(SessionError::ImageFormMetadataUnavailable)
    );
    tokenizer.release(&mut c).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    let (image_id, form_id, raw_id) = {
        let doc = frozen.document().unwrap();
        let html = doc.root().children().next().unwrap();
        let body = doc.node(&html).unwrap().children().next().unwrap();
        let ids: Vec<_> = doc.node(&body).unwrap().children().collect();
        (ids[1], ids[0], ids[2])
    };
    assert_eq!(
        frozen.image_form_owner(&mut c, &image_id).unwrap(),
        Some(form_id)
    );
    assert!(
        frozen
            .image_form_state(&mut c, &image_id)
            .unwrap()
            .parser_inserted
    );
    assert_eq!(
        frozen.image_form_state(&mut c, &raw_id),
        Err(SessionError::ImageFormMetadataUnavailable)
    );
    let other = HtmlDocument::builder(
        HtmlConstructionLimits::new(10, 10, 10, 100, 10),
        HtmlDocumentMode::NoQuirks,
    )
    .unwrap()
    .finish()
    .unwrap();
    let foreign = other.root().id();
    assert_eq!(
        frozen.image_form_state(&mut c, &foreign),
        Err(SessionError::ForeignNode)
    );
    drop((image_id, form_id, raw_id, foreign, other));
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn image_creation_rejects_wrong_arena_alias_and_foreign_lexical_owner_before_take() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<img><img>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let mut other =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let foreign = other.allocate_implicit_html(&mut c).unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("img")
    };
    let mut input = Some(tag);
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    assert_eq!(
        session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &foreign,
                None,
                crate::construction::CreationContext::Ordinary
            )
            .unwrap_err(),
        SessionError::ForeignNode
    );
    assert!(input.is_some());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        nodes
    );
    let alias = input.as_ref().unwrap().name.value.text().clone();
    assert_eq!(
        session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &body,
                None,
                crate::construction::CreationContext::Ordinary
            )
            .unwrap_err(),
        SessionError::AliasedPayload
    );
    assert!(input.is_some());
    drop(alias);
    release_formatting_tag(&mut input, &mut c).unwrap();
    let mut foreign_control = control();
    foreign_control.admit_source(&source).unwrap();
    let mut foreign_tokenizer = LexerFixture::new(
        &source,
        HtmlTokenizerState::Data,
        None,
        &mut foreign_control,
    )
    .unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut foreign_tokenizer, &mut foreign_control) else {
        panic!("img")
    };
    let mut input = Some(tag);
    assert!(matches!(
        session.allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &body,
            None,
            crate::construction::CreationContext::Ordinary
        ),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert!(input.is_some());
    release_formatting_tag(&mut input, &mut foreign_control).unwrap();
    foreign_tokenizer.release(&mut foreign_control).unwrap();
    storage_zero(&foreign_control);
    tokenizer.release(&mut c).unwrap();
    other.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

fn image_move_fixture(
    c: &mut ParseControl,
) -> (ParserArenaSession, ParserNode, ParserNode, ParserNode) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><form><img>");
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, c).unwrap();
    let (mut session, body) = form_session(c);
    let old = form_creation(&mut session, &mut tokenizer, c, &body, None);
    let new = form_creation(&mut session, &mut tokenizer, c, &body, None);
    session.insert(c, &body, &old, None, None).unwrap();
    session.insert(c, &body, &new, None, None).unwrap();
    let image = form_creation(&mut session, &mut tokenizer, c, &body, Some(&old));
    session.insert(c, &body, &image, None, None).unwrap();
    tokenizer.release(c).unwrap();
    (session, image, old, new)
}
fn retained_image_state(session: &ParserArenaSession, image: &ParserNode) -> FormAssociationState {
    session
        .active()
        .unwrap()
        .payloads
        .values
        .iter()
        .find_map(|entry| match entry {
            RetainedPayload::FormAssociation { node, state } if *node == Some(image.index) => {
                Some(*state)
            }
            _ => None,
        })
        .unwrap()
}
#[test]
fn image_effect_work_exact_and_one_short_keep_real_completed_or_unfreezable_owners() {
    let mut measured = control();
    let (mut session, image, _, target) = image_move_fixture(&mut measured);
    let start = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session
        .insert(&mut measured, &target, &image, None, None)
        .unwrap();
    let work = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - start;
    session.release(&mut measured).unwrap();
    storage_zero(&measured);
    // Empty receive tail: runner reads2, actual fixed Option receiver
    // copy/layout + write1, empty OwnedVec pop visit1 (no Some move).
    let tail = 2 + mem::size_of::<Option<effects::EffectFrame>>() as u64 + 1 + 1;
    for remaining in [work, work - tail - 1] {
        let mut c = control();
        let (mut session, image, _, target) = image_move_fixture(&mut c);
        let before_nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        let unused =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(HtmlParseWorkCategory::StateStep, unused - remaining)
            .unwrap();
        let result = session.insert(&mut c, &target, &image, None, None);
        if remaining == work {
            assert_eq!(result, Ok(()));
            assert!(session.active().unwrap().effects.form.is_none());
            let state = retained_image_state(&session, &image);
            assert_eq!(state.owner, Some(target.index));
            assert!(!state.parser_inserted);
            // Exhausted successful work budget: request one actual credit
            // to latch terminal disposal without pretending cleanup is free.
            assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
        } else {
            let stop = c.stop().unwrap();
            assert!(matches!(
                stop,
                HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }
            ));
            assert_eq!(result, Err(SessionError::Stop(stop)));
            assert_eq!(
                session
                    .active()
                    .unwrap()
                    .construction
                    .node_at(image.index)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position()),
                Some(target.index)
            );
            let state = retained_image_state(&session, &image);
            assert_eq!(state.owner, Some(target.index));
            assert!(!state.parser_inserted);
            let storage = session.active().unwrap();
            assert!(storage.effects.form.is_none());
            let (list, cursor) = storage
                .effects
                .retained_post_connection()
                .expect("specific unfinished final postconnection clear");
            assert_eq!(cursor, 1);
            assert!(matches!(
                storage.payloads.values[list],
                RetainedPayload::Retired
            ));
            assert_eq!(
                session.image_form_state(&mut c, &image),
                Err(SessionError::Stop(stop))
            );
            assert_eq!(session.root(&mut c).unwrap_err(), SessionError::Stop(stop));
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            assert_eq!(c.checkpoint(), Err(stop));
        }
        let high = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water;
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let stop = c.stop();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            before_nodes
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            used
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water,
            high
        );
    }
}
#[test]
fn image_metadata_admission_failure_preserves_original_tag_and_arena_before_take() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<img>");
    let mut c = ParseControl::new(
        limits()
            .into_builder()
            .auxiliary_entries(17)
            .active_formatting_entries(17)
            .finish()
            .unwrap(),
        HtmlParseCancellation::new(),
    );
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let html = session.allocate_implicit_html(&mut c).unwrap();
    session.insert(&mut c, &root, &html, None, None).unwrap();
    // Produce the actual lexical owner before filling the final retained
    // slots: tokenizer pending events have their own transient admission.
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("real img token")
    };
    // This actual detached allocation fills the last five retained slots.
    // Attaching it would require unrelated temporary link writes;
    // the image's actual intended parent is the attached HTML container.
    let _body = session.allocate_implicit_body(&mut c, None).unwrap();
    // Root/html/body own three node-join cells, four field payload slots, four document facts, and four nonce slots.
    // Two independent creation rows accompany those records.
    // No auxiliary entry remains for the image's required metadata slot.
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        17
    );
    let name_backing = tag.name.value.text().block().address();
    let original_origins: Vec<_> = tag
        .name
        .origins
        .as_slice()
        .iter()
        .map(|origin| {
            (
                origin.start,
                origin.end,
                origin.source.start(),
                origin.source.end(),
            )
        })
        .collect();
    let original_topology: Vec<_> = (0..session.active().unwrap().construction.len())
        .map(|position| {
            let record = session
                .active()
                .unwrap()
                .construction
                .node_at(position)
                .unwrap();
            (
                record.parent().map(|node| node.position()),
                record
                    .children()
                    .map(|node| node.position())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
    let mut input = Some(tag);
    let result = session.allocate_html_tag_for_creation(
        &mut c,
        &mut input,
        &html,
        None,
        crate::construction::CreationContext::Ordinary,
    );
    let stop = c.stop().unwrap();
    let retained = input.as_ref().map(|tag| {
        (
            tag.kind,
            tag.name.value.text().block().address(),
            tag.name.value.text().code_units().to_vec(),
            tag.attributes.len(),
            tag.name
                .origins
                .as_slice()
                .iter()
                .map(|origin| {
                    (
                        origin.start,
                        origin.end,
                        origin.source.start(),
                        origin.source.end(),
                    )
                })
                .collect::<Vec<_>>(),
        )
    });
    let topology: Vec<_> = (0..session.active().unwrap().construction.len())
        .map(|position| {
            let record = session
                .active()
                .unwrap()
                .construction
                .node_at(position)
                .unwrap();
            (
                record.parent().map(|node| node.position()),
                record
                    .children()
                    .map(|node| node.position())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        result.err(),
        Some(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::AuxiliaryEntries,
            limit: 17,
            current: 17,
            requested_delta: 2,
        }))
    );
    assert_eq!(topology, original_topology);
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        nodes
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedEdges).current,
        edges
    );
    // Reclaim actual surviving owners before the regression assertion. A
    // retained Tag must still be releasable by its original sole controller.
    release_formatting_tag(&mut input, &mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(stop));
    assert_eq!(
        retained,
        Some((
            TagKind::Start,
            name_backing,
            vec![u16::from(b'i'), u16::from(b'm'), u16::from(b'g')],
            0,
            original_origins,
        )),
        "required image metadata admission precedes rejectable lexical take"
    );
}
#[test]
fn image_creation_original_rejections_precede_any_metadata_slot_or_capacity() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("</img><img x=one><img x=two>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let payloads = session.active().unwrap().payloads.values.len();
    let capacity = session.active().unwrap().payloads.values.capacity();
    let auxiliary = c
        .usage()
        .resource(HtmlParseResource::AuxiliaryEntries)
        .current;
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("actual end tag")
    };
    let mut end = Some(tag);
    assert_eq!(
        session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut end,
                &body,
                None,
                crate::construction::CreationContext::Ordinary
            )
            .unwrap_err(),
        SessionError::WrongToken
    );
    assert!(end.is_some());
    assert_eq!(session.active().unwrap().payloads.values.len(), payloads);
    assert_eq!(
        session.active().unwrap().payloads.values.capacity(),
        capacity
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        auxiliary
    );
    assert!(c.stop().is_none());
    release_formatting_tag(&mut end, &mut c).unwrap();
    let TokenKind::Tag(mut first) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("first actual start tag")
    };
    let TokenKind::Tag(mut second) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("second actual start tag")
    };
    // Independently owned actual parsed fields with the same source/control.
    // This invalid private input has two exact x names, not a forged token
    // duplicate flag or aliased backing that could bypass uniqueness.
    let duplicate = second.attributes.try_pop(&mut c).unwrap().unwrap();
    first.attributes.try_push(&mut c, duplicate).unwrap();
    let mut second = Some(second);
    let mut input = Some(first);
    let alias = input.as_ref().unwrap().name.value.text().clone();
    assert_eq!(
        session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &body,
                None,
                crate::construction::CreationContext::Ordinary
            )
            .unwrap_err(),
        SessionError::AliasedPayload
    );
    assert!(input.is_some());
    drop(alias);
    assert_eq!(
        session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &body,
                None,
                crate::construction::CreationContext::Ordinary
            )
            .unwrap_err(),
        SessionError::DuplicateAttribute
    );
    assert_eq!(input.as_ref().unwrap().attributes.len(), 2);
    assert_eq!(session.active().unwrap().payloads.values.len(), payloads);
    assert_eq!(
        session.active().unwrap().payloads.values.capacity(),
        capacity
    );
    assert_eq!(session.active().unwrap().construction.len(), 3);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        auxiliary
    );
    assert!(c.stop().is_none());
    release_formatting_tag(&mut input, &mut c).unwrap();
    release_formatting_tag(&mut second, &mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn image_creation_binding_stop_retains_initialized_slot_and_actual_node_for_cleanup() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<img>");
    let mut measured = control();
    measured.admit_source(&source).unwrap();
    let mut lexer =
        LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut measured).unwrap();
    let (mut session, body) = form_session(&mut measured);
    let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut measured) else {
        panic!("actual img")
    };
    let start = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session
        .allocate_html_tag_for_creation(
            &mut measured,
            &mut Some(tag),
            &body,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    let complete = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - start;
    let complete_entries = measured
        .usage()
        .resource(HtmlParseResource::AuxiliaryEntries)
        .current;
    lexer.release(&mut measured).unwrap();
    session.release(&mut measured).unwrap();
    storage_zero(&measured);
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
        panic!("actual img")
    };
    let mut input = Some(tag);
    let unused =
        limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
    // The final binding has a real last RecordWrite credit after its visit,
    // layout admission and Option<usize> copy. Deny only that final write.
    c.charge(HtmlParseWorkCategory::StateStep, unused - (complete - 1))
        .unwrap();
    let error = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &body,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap_err();
    let stop = c.stop().unwrap();
    assert_eq!(error, SessionError::Stop(stop));
    assert!(matches!(
        stop,
        HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            requested_delta: 1,
            ..
        }
    ));
    assert!(input.is_none());
    let storage = session.active().unwrap();
    assert_eq!(storage.construction.len(), 4);
    assert_eq!(
        storage
            .construction
            .node_at(3)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        None
    );
    assert!(
        storage
            .payloads
            .values
            .iter()
            .any(|entry| matches!(entry, RetainedPayload::FormAssociation { node: None, .. }))
    );
    // The stopped creation retains every entry the completed one holds.
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        complete_entries
    );
    assert_eq!(session.root(&mut c).unwrap_err(), SessionError::Stop(stop));
    assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
    assert_eq!(
        session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &body,
                None,
                crate::construction::CreationContext::Ordinary
            )
            .unwrap_err(),
        SessionError::Stop(stop)
    );
    let high = c
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .high_water;
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(stop));
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        4
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water,
        high
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
}
#[test]
fn image_creation_actual_metadata_slot_has_exact_auxiliary_admission_and_terminal_cleanup() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // The completed prefix plus exactly one retained standalone Image row is
    // the measured total. Its empty request Vec owns bytes but no auxiliary
    // entries. One entry short independently rejects the NEW Image row
    // after physical node installation; two short is the earlier
    // record-ticket rejection.
    let (prefix, needed) = {
        let source = crate::HtmlSource::utf8("<img>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let prefix = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("img")
        };
        session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &body,
                None,
                crate::construction::CreationContext::Ordinary,
            )
            .unwrap();
        let needed = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        (prefix, needed)
    };
    for cap in [needed, needed - 1, needed - 2] {
        let source = crate::HtmlSource::utf8("<img>");
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .auxiliary_entries(cap)
                .active_formatting_entries(cap)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        // Root plus html/body: 3 records/tickets, 4 fields, 4 document facts and 4 nonce slots.
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            prefix
        );
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("img")
        };
        let mut input = Some(tag);
        let result = session.allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &body,
            None,
            crate::construction::CreationContext::Ordinary,
        );
        assert!(input.is_none());
        if cap == needed {
            let image = result.unwrap();
            assert_form_owner(&session, &mut c, &image, None, false);
            // Prior name/namespace + record ticket + form/creation
            // metadata + two nonce slots, plus one standalone Image row.
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                needed
            );
            assert_eq!(
                session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .filter(|entry| matches!(entry, RetainedPayload::Image { .. }))
                    .count(),
                1
            );
        } else if cap == needed - 1 {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    limit,
                    current,
                    requested_delta: 1
                })) if limit == cap && current == cap
            ));
            let storage = session.active().unwrap();
            assert_eq!(storage.construction.len(), 4);
            assert!(
                !storage
                    .payloads
                    .values
                    .iter()
                    .any(|entry| matches!(entry, RetainedPayload::Image { .. }))
            );
            assert!(
                storage.payloads.values.iter().any(|entry| matches!(
                    entry,
                    RetainedPayload::FormAssociation { node: None, .. }
                ))
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                cap
            );
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    limit,
                    ..
                })) if limit == cap
            ));
            // Metadata, name and namespace are initialized;
            // the next actual record-ticket entry is the rejected slot.
            assert_eq!(session.active().unwrap().construction.len(), 3);
            assert_eq!(session.active().unwrap().payloads.values.len(), 21);
            assert!(
                session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .any(|entry| matches!(
                        entry,
                        RetainedPayload::FormAssociation { node: None, .. }
                    ))
            );
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(c.stop().unwrap()))
            );
        }
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
#[test]
fn image_move_rejections_and_cancel_preserve_links_facts_and_first_reason() {
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let (mut session, image, old, target) = image_move_fixture(&mut c);
    let before = retained_image_state(&session, &image);
    let mut other =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let foreign = other.root(&mut c).unwrap();
    assert_eq!(
        session.insert(&mut c, &foreign, &image, None, None),
        Err(SessionError::ForeignNode)
    );
    assert_eq!(retained_image_state(&session, &image), before);
    assert!(session.active().unwrap().effects.form.is_none());
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(image.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(2)
    );
    cancellation.cancel();
    assert_eq!(
        session.insert(&mut c, &target, &image, None, None),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(retained_image_state(&session, &image), before);
    assert!(session.active().unwrap().effects.form.is_none());
    assert_eq!(
        session.image_form_owner(&mut c, &image).unwrap_err(),
        SessionError::Stop(HtmlParseStop::Cancelled)
    );
    assert_eq!(
        session.remove(&mut c, &old, None),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    other.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}

#[test]
fn image_real_cancellation_between_removal_and_insertion_preserves_completed_removal() {
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let (mut session, image, old, target) = image_move_fixture(&mut c);
    session.active_mut().unwrap().cancel_after_removal = Some(cancellation);
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    assert_eq!(
        session.insert(&mut c, &target, &image, None, None),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let storage = session.active().unwrap();
    assert_eq!(
        storage
            .construction
            .node_at(image.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(target.index)
    );
    assert!(
        !storage
            .construction
            .node_at(old.index)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>()
            .contains(&image.index)
    );
    assert_eq!(
        storage
            .construction
            .node_at(target.index)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>(),
        [image.index]
    );
    assert_eq!(
        storage.effects.form.unwrap().stage,
        FormEffectStage::Adoption
    );
    let state = retained_image_state(&session, &image);
    assert_eq!(state.owner, None);
    assert!(!state.parser_inserted);
    assert_eq!(state.evidence, HtmlParseFormEvidence::Reset);
    assert_eq!(
        session.image_form_state(&mut c, &image),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(
        session.insert(&mut c, &old, &image, None, None),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        nodes
    );
}

#[test]
fn image_removal_clears_parser_flag_before_ancestor_work_can_stop() {
    // Raise the work allowance of the removal drain one credit at a time.
    // The first allowance that lets the reset clear the parser flag stops
    // inside the later ancestor resolution: the flag is already cleared,
    // the owner is not, and the retained state is exactly that of the
    // stopped reset. No allowance below it clears the flag.
    let mut boundary = None;
    for allowance in 1..4000_u64 {
        let mut c = control();
        let (mut session, image, old, target) = image_move_fixture(&mut c);
        for node in [&image, &old, &target] {
            session.validate_node(&mut c, node).unwrap();
        }
        let old_parent = session
            .active()
            .unwrap()
            .construction
            .node_at(image.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position());
        session
            .active_mut()
            .unwrap()
            .insert_indices(&mut c, target.index, image.index, None)
            .unwrap();
        let storage = session.active().unwrap();
        let effects = storage.effects.form.unwrap();
        assert_eq!(effects.root, image.index);
        assert_eq!(effects.next, Some(image.index));
        assert!(matches!(effects.stage, FormEffectStage::Removal));
        assert!(matches!(effects.pass, FormEffectPass::Local));
        assert_eq!(effects.old_parent, old_parent);
        assert_eq!(effects.new_parent, Some(target.index));
        assert!(effects.insert);
        let unused =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(HtmlParseWorkCategory::StateStep, unused - allowance)
            .unwrap();
        let result = session
            .active_mut()
            .unwrap()
            .drain_construction_effects(&mut c);
        let cleared = !retained_image_state(&session, &image).parser_inserted;
        if !cleared {
            assert!(result.is_err(), "allowance {allowance} completed untouched");
            session.release(&mut c).unwrap();
            storage_zero(&c);
            continue;
        }
        let error = result.unwrap_err();
        let stop = c.stop().unwrap();
        assert_eq!(error, SessionError::Stop(stop));
        assert!(matches!(
            stop,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }
        ));
        assert_eq!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(image.index)
                .expect("retained construction position")
                .parent()
                .map(|node| node.position()),
            Some(target.index)
        );
        let retained = retained_image_state(&session, &image);
        assert!(
            !retained.parser_inserted,
            "reset step1 precedes ordinary ancestor resolution"
        );
        assert_eq!(
            retained.owner,
            Some(old.index),
            "later clear-owner step has not run"
        );
        assert_eq!(retained.evidence, HtmlParseFormEvidence::Reset);
        assert_eq!(
            session.active().unwrap().effects.form.unwrap().stage,
            FormEffectStage::Removal
        );
        assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
        boundary = Some(allowance);
        break;
    }
    assert!(boundary.is_some(), "some allowance stops after the reset");
}

#[test]
fn image_self_reference_insertion_runs_removal_before_insertion_with_unchanged_links() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    for subtree in [false, true] {
        let source = crate::HtmlSource::utf8(if subtree {
            "<form><div><img>"
        } else {
            "<form><img>"
        });
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        session.insert(&mut c, &body, &form, None, None).unwrap();
        let parent = if subtree {
            let div = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&form));
            session.insert(&mut c, &body, &div, None, None).unwrap();
            div
        } else {
            body
        };
        let image = form_creation(&mut session, &mut tokenizer, &mut c, &parent, Some(&form));
        session.insert(&mut c, &parent, &image, None, None).unwrap();
        assert_form_owner(&session, &mut c, &image, Some(&form), true);
        let moved = if subtree { &parent } else { &image };
        let old_order = session
            .active()
            .unwrap()
            .construction
            .node_at(body.index)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>()
            .clone();
        session
            .insert(&mut c, &body, moved, Some(moved), None)
            .unwrap();
        assert_eq!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(body.index)
                .expect("retained construction position")
                .children()
                .map(|node| node.position())
                .collect::<Vec<_>>(),
            old_order
        );
        let result = session.image_form_state(&mut c, &image).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        // Pre-insert reference adjustment preserves final position, but
        // adoption still removes the actual root before inserting it again.
        assert!(!result.parser_inserted);
        assert_eq!(result.owner, None);
        assert_eq!(result.evidence, HtmlParseFormEvidence::Reset);
    }
}

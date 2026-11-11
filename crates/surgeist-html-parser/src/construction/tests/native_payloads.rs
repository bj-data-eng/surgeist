use super::*;
use surgeist_html::{construction::ConstructionNodeKind, values::Namespace};

#[test]
fn newly_latched_work_or_cancellation_disposes_all_retained_attribute_fields() {
    for cancelled in [false, true] {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let Tag {
            name, attributes, ..
        } = owned_tag(&mut c, "div", &[("x", &[0xd800])]);
        name.release(&mut c).unwrap();
        let before = c.usage().resource(HtmlParseResource::ManagedCapacityBytes);
        if cancelled {
            cancellation.cancel();
            assert!(c.stop().is_none());
        } else {
            let remaining = limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(HtmlParseWorkCategory::StateStep, remaining)
                .unwrap();
        }
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        let error = release_token_attributes(attributes, &mut c).unwrap_err();
        if cancelled {
            assert_eq!(error, SessionError::Stop(HtmlParseStop::Cancelled));
        } else {
            assert!(matches!(
                error,
                SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                })
            ));
        }
        storage_zero(&c);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water,
            before.high_water
        );
    }
}

#[test]
fn final_empty_attribute_cleanup_work_stop_releases_the_actual_lexical_capacity_ticket() {
    // Select the exact final empty-pop boundary from the successful real
    // producer, then remove its single remaining RecordVisit credit.
    let mut probe = control();
    let mut session = ParserArenaSession::new(&mut probe, RootKind::Fragment).unwrap();
    let mut input = Some(owned_tag(&mut probe, "div", &[("x", &[0x61])]));
    let lexical_bytes = input.as_ref().unwrap().attributes.capacity()
        * mem::size_of::<crate::parser::tokenizer::token::Attribute>();
    session.allocate_html_tag(&mut probe, &mut input).unwrap();
    let completed_work = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session.release(&mut probe).unwrap();
    storage_zero(&probe);

    let caps = limits()
        .into_builder()
        .work_credits(completed_work - 1)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let mut input = Some(owned_tag(&mut c, "div", &[("x", &[0x61])]));
    assert!(matches!(
        session.allocate_html_tag(&mut c, &mut input),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            requested_delta: 1,
            ..
        }))
    ));
    assert!(input.is_none());
    assert_eq!(session.active().unwrap().construction.len(), 2);
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        2
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::TokenAttributes)
            .current,
        0
    );
    let stop = c.stop();
    let work = c.usage().resource(HtmlParseResource::WorkCredits);
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes);
    let capacity_high_water = c
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .high_water;
    session.release(&mut c).unwrap();
    assert_eq!(c.stop(), stop);
    assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
    assert_eq!(c.usage().resource(HtmlParseResource::ManagedNodes), nodes);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water,
        capacity_high_water
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current,
        0,
        "all actual backing must be reclaimed; the lexical attribute Vec alone occupied {lexical_bytes} bytes"
    );
    storage_zero(&c);
}

#[test]
fn actual_payload_registry_capacity_and_uninitialized_slot_failure_are_reclaimable() {
    // Independently sum the root record bank, one exact frozen
    // unit+Vec wrapper, and one source-origin record. None includes an Arc
    // header or preexisting source backing.
    let mut probe_control = control();
    let mut probe = ParserArenaSession::new(&mut probe_control, RootKind::Fragment).unwrap();
    let root_capacity = probe_control.usage().actual_capacity_bytes as usize;
    probe.release(&mut probe_control).unwrap();
    storage_zero(&probe_control);
    let baseline = root_capacity
        + mem::size_of::<u16>()
        + mem::size_of::<Vec<u16>>()
        + mem::size_of::<HtmlParseTextOrigin>();
    let cap = baseline + mem::size_of::<RetainedPayload>() - 1;
    let caps = limits()
        .into_builder()
        .managed_capacity_bytes(cap as u64)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut input = Some(owned_text(&mut c, &[0x61]));
    assert_eq!(c.usage().actual_capacity_bytes, baseline as u64);
    assert!(matches!(
        session.insert_text(&mut c, &root, None, &mut input, None),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        }))
    ));
    assert_eq!(session.active().unwrap().construction.len(), 1);
    // The record's units are acquired when the record is made, so the refusal of its bank
    // leaves none held.
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        0
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);

    let caps = limits()
        .into_builder()
        .auxiliary_entries(1)
        .active_formatting_entries(1)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut input = Some(owned_text(&mut c, &[0x61]));
    assert!(matches!(
        session.insert_text(&mut c, &root, None, &mut input, None),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::AuxiliaryEntries,
            limit: 1,
            current: 1,
            // The record's join cell: no registry row is asked for.
            requested_delta: 1
        }))
    ));
    assert!(session.active().unwrap().payloads.values.is_empty());
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        1
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn isolated_token_name_units_are_stored_without_utf8_conversion_or_name_restriction() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let name = owned_text(&mut c, &[0x61, 0xd800, 0x3a]);
    let attributes = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
    let mut tag = Some(Tag {
        kind: TagKind::Start,
        name,
        attributes,
        self_closing: false,
        had_duplicate_attributes: false,
    });
    let node = session.allocate_html_tag(&mut c, &mut tag).unwrap();
    let name = session.element_name(&mut c, &node).unwrap().unwrap();
    assert_eq!(name.local_name().code_units(), &[0x61, 0xd800, 0x3a]);
    assert_eq!(name.prefix(), None);
    assert!(name.local_name().to_utf8().is_err());
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn real_tokenizer_case_duplicate_recovery_and_reference_data_reach_managed_native_element() {
    use crate::parser::tokenizer::{
        states::HtmlTokenizerState,
        tests::LexerFixture,
        token::{Event, TokenKind},
    };
    let source = crate::HtmlSource::utf8("<DiV Z=x z=y A:B='&#x1F600;'>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut inserted = None;
    while let Some(event) = tokenizer.next(&mut c, false).unwrap() {
        match event {
            Event::Token(event) => {
                let token = event.into_token(&mut c).unwrap();
                match token.kind {
                    TokenKind::Tag(tag) => {
                        assert!(tag.had_duplicate_attributes);
                        let mut input = Some(tag);
                        let node = session.allocate_html_tag(&mut c, &mut input).unwrap();
                        session.insert(&mut c, &root, &node, None, None).unwrap();
                        inserted = Some(node);
                    }
                    TokenKind::Eof => {}
                    _ => panic!("fixture emits only one start tag and EOF"),
                }
            }
            event => event.release(&mut c).unwrap(),
        }
    }
    tokenizer.release(&mut c).unwrap();
    let node = inserted.unwrap();
    let ConstructionNodeKind::Element(element) = &session
        .active()
        .unwrap()
        .construction
        .node_at(node.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("element")
    };
    assert_eq!(
        element.name().local_name().code_units(),
        &[0x64, 0x69, 0x76]
    );
    assert_eq!(element.attributes().len(), 2);
    assert_eq!(
        element
            .attributes()
            .get(0)
            .unwrap()
            .name()
            .local_name()
            .code_units(),
        &[0x7a]
    );
    assert_eq!(
        element.attributes().get(0).unwrap().value().code_units(),
        &[0x78]
    );
    assert_eq!(
        element
            .attributes()
            .get(1)
            .unwrap()
            .name()
            .local_name()
            .code_units(),
        &[0x61, 0x3a, 0x62]
    );
    assert_eq!(
        element.attributes().get(1).unwrap().value().code_units(),
        &[0xd83d, 0xde00]
    );
    let mut frozen = session.freeze(&mut c).unwrap();
    assert_eq!(frozen.fragment().unwrap().root().children().count(), 1);
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn merged_append_cap_refusal_retains_old_native_text_and_sticky_cleanup() {
    // The text holds one unit. Appending one more reaches two, so a managed cap of one refuses
    // the append and names the unit it would add.
    let caps = limits()
        .into_builder()
        .managed_utf16_units(1)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut a = Some(owned_text(&mut c, &[0x61]));
    let a = session
        .insert_text(&mut c, &root, None, &mut a, None)
        .unwrap()
        .unwrap();
    let mut b = Some(owned_text(&mut c, &[0x62]));
    assert!(matches!(
        session.insert_text(&mut c, &root, None, &mut b, None),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedUtf16Units,
            limit: 1,
            current: 1,
            requested_delta: 1,
        }))
    ));
    assert_eq!(native_text(&session, &a._node), [0x61]);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        1
    );
    // The token's string is released with the refusal.
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        0
    );
    let stop = c.stop();
    let high = c
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .high_water;
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), stop);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water,
        high
    );
}

#[test]
fn topology_hooks_borrow_exact_names_and_distinguish_real_template_creation() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    assert!(session.element_name(&mut c, &root).unwrap().is_none());
    assert!(session.fragment_root(&mut c).unwrap().is_some());
    session.validate_node(&mut c, &root).unwrap();
    let old = session.allocate_element(&mut c, html("template")).unwrap();
    assert!(matches!(
        session.unpatched_template_contents(&mut c, &old),
        Err(SessionError::TemplateInsertionStateUnavailable)
    ));
    let mut tag = Some(owned_tag(&mut c, "template", &[]));
    let new = session.allocate_html_tag(&mut c, &mut tag).unwrap();
    assert!(
        session
            .unpatched_template_contents(&mut c, &new)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        session
            .element_name(&mut c, &new)
            .unwrap()
            .unwrap()
            .local_name()
            .code_units(),
        &[0x74, 0x65, 0x6d, 0x70, 0x6c, 0x61, 0x74, 0x65]
    );
    assert!(!session.same_node(&mut c, &old, &new).unwrap());
    assert!(session.parent(&mut c, &new).unwrap().is_none());
    session.insert(&mut c, &root, &new, None, None).unwrap();
    let parent = session.parent(&mut c, &new).unwrap().unwrap();
    assert!(session.same_node(&mut c, &parent, &root).unwrap());
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn token_backing_transfers_exact_names_order_attributes_and_template_pair_to_native_view() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut tag = Some(owned_tag(
        &mut c,
        "template",
        &[("z", &[0xd800, 0, 0x20]), ("a:b", &[0xd83d, 0xde00])],
    ));
    let original = tag
        .as_ref()
        .unwrap()
        .name
        .value
        .text()
        .code_units()
        .as_ptr();
    let element = session.allocate_html_tag(&mut c, &mut tag).unwrap();
    assert!(tag.is_none());
    session.insert(&mut c, &root, &element, None, None).unwrap();
    let contents = session
        .template_contents(&mut c, &element)
        .unwrap()
        .unwrap();
    assert_ne!(contents.index, element.index);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        0
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        2
    );
    let mut frozen = session.freeze(&mut c).unwrap();
    let fragment = frozen.fragment().unwrap();
    let id = fragment.root().children().next().unwrap();
    let surgeist_html::construction::HtmlNodeKind::Element(value) =
        fragment.node(&id).unwrap().kind()
    else {
        panic!("element")
    };
    assert_eq!(value.name().local_name().code_units().as_ptr(), original);
    assert_eq!(value.name().namespace(), &Namespace::html());
    let attrs: Vec<_> = value.attributes().iter().collect();
    assert_eq!(attrs[0].name().local_name().code_units(), &[0x7a]);
    assert_eq!(attrs[0].value().code_units(), &[0xd800, 0, 0x20]);
    assert_eq!(
        attrs[1].name().local_name().code_units(),
        &[0x61, 0x3a, 0x62]
    );
    assert_eq!(attrs[1].name().namespace(), &Namespace::none());
    assert_eq!(attrs[1].name().prefix(), None);
    assert_eq!(attrs[1].value().code_units(), &[0xd83d, 0xde00]);
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        3
    );
}

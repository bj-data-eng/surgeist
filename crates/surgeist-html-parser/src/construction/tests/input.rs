use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlDocumentMode},
    elements::form::HtmlNativeFormAssociationKind,
    values::DocumentString,
};

#[test]
fn hidden_input_ordered_initialization_reset_and_frozen_values_are_independent() {
    for (attrs, expected, rewritten) in [
        (
            "value='A&#13;B&#10;C' type=hidden checked",
            vec![65, 66, 67],
            true,
        ),
        (
            "type=HiDdEn value='A&#13;B&#10;C' checked",
            vec![65, 13, 66, 10, 67],
            false,
        ),
        ("value='&#13;&#10;' type=hidden", vec![13, 10], false),
        (
            "value='A&#13;B' type=hidden value=ignored type=text",
            vec![65, 66],
            true,
        ),
        ("type=hidden", vec![], false),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(format!("<html><head></head><input {attrs}>")),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        let original_value_source = input
            .as_ref()
            .unwrap()
            .attributes
            .as_slice()
            .iter()
            .find(|a| a.name.value.text().code_units() == [118, 97, 108, 117, 101])
            .map(|a| a.source.clone());
        let node = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            )
            .unwrap();
        assert!(input.is_none());
        match session.input_writeback_sources(&mut c, &node).unwrap() {
            Some((token, value)) => {
                assert!(rewritten);
                assert_eq!(token, &token_source);
                assert_eq!(Some(value), original_value_source.as_ref());
                assert_eq!(token.coordinates(), crate::HtmlSourceCoordinates::Utf8Bytes);
            }
            None => assert!(!rewritten),
        }
        let state = session.input_state(&mut c, &node).unwrap();
        assert!(
            !state.flags.dirty_value()
                && !state.flags.dirty_checkedness()
                && !state.flags.indeterminate()
                && !state.flags.user_validity()
        );
        assert!(state.selected_files_empty);
        assert_eq!(state.flags.checkedness(), attrs.contains("checked"));
        assert_eq!(state.value_rewritten_during_creation, rewritten);
        let association = session.form_association_state(&mut c, &node).unwrap();
        assert_eq!(association.kind, HtmlNativeFormAssociationKind::Input);
        assert!(
            session
                .form_association_owner(&mut c, &node)
                .unwrap()
                .is_none()
        );
        let mut value = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &node, &mut value)
            .unwrap();
        assert_eq!(value.units(), expected);
        value.clear(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &node, &mut value)
            .unwrap();
        assert_eq!(value.units(), expected);
        value.release(&mut c).unwrap();
        let storage = session.active().unwrap();
        let raw = find_input_state(&storage.payloads, &mut c, node.index)
            .unwrap()
            .0;
        let RetainedPayload::String {
            value: internal, ..
        } = &storage.payloads.values[raw]
        else {
            unreachable!()
        };
        if let Some(native) = storage.form_attribute(&mut c, node.index, "value").unwrap() {
            assert!(!native.shares_storage_with(internal));
        }
        assert!(storage.payloads.values.iter().any(|p| matches!(
            p,
            RetainedPayload::HistoricalString { .. } | RetainedPayload::Retired
        )));
        if rewritten {
            assert!(storage.payloads.values.iter().any(|p| matches!(p, RetainedPayload::String { value, origins:Some(_), location:StringLocation::Detached, .. } if value.code_units() == [65,13,66,10,67] || value.code_units() == [65,13,66])));
        }
        session.insert(&mut c, &parent, &node, None, None).unwrap();
        lexer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let document = frozen.document().unwrap();
        let html = document.root().children().next().unwrap();
        let id = document.node(&html).unwrap().children().last().unwrap();
        assert_eq!(frozen.input_state(&mut c, &id).unwrap(), state);
        match frozen.input_writeback_sources(&mut c, &id).unwrap() {
            Some((token, value)) => {
                assert!(rewritten);
                assert_eq!(token, &token_source);
                assert_eq!(Some(value), original_value_source.as_ref());
            }
            None => assert!(!rewritten),
        }
        let mut value = Utf16Buffer::new(&mut c).unwrap();
        frozen
            .append_input_internal_value(&mut c, &id, &mut value)
            .unwrap();
        assert_eq!(value.units(), expected);
        value.clear(&mut c).unwrap();
        frozen
            .append_input_default_value(&mut c, &id, &mut value)
            .unwrap();
        assert_eq!(value.units(), expected);
        value.release(&mut c).unwrap();
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn checkbox_exact_work_and_one_short_preserve_original_stop_and_dispose_all_prefixes() {
    let source = "<html><head></head><input value='A&#13;B&#10;C' type=checkbox checked>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        )
        .unwrap();
    let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    // Measure the complete real issuer transaction, then verify its exact
    // boundary and multiple actual partial allocation/write prefixes.
    for tail in [0, 1, 32, 128, 512, 1024, 2048] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source),
            limits()
                .into_builder()
                .work_credits(total - tail)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        let result = session.allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        );
        if tail == 0 {
            assert!(result.is_ok());
            assert!(input.is_none());
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                total
            );
            assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
        }
        let first = c.stop().unwrap();
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(first))
        );
        let at_stop = c.usage().resource(HtmlParseResource::WorkCredits).current;
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(first));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            at_stop
        );
    }
}

#[test]
fn checkbox_reset_cancellation_retains_completed_value_before_association_and_cleanup() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=checkbox value='A&#13;B&#10;C' checked>",
        ),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    session.active_mut().unwrap().cancel_after_input_reset = Some(request.cancellation().clone());
    assert_eq!(
        session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability
            )
            .err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert!(input.is_none());
    let storage = session.active().unwrap();
    let (node, raw, state) = storage
        .payloads
        .values
        .iter()
        .find_map(|p| match p {
            RetainedPayload::Input {
                node: Some(node),
                raw,
                complete: true,
                state,
                ..
            } => Some((*node, *raw, *state)),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        state.flags.input_type(),
        surgeist_html::attributes::HtmlInputType::Checkbox
    );
    assert_eq!(state.cursor, 0);
    assert!(state.flags.checkedness());
    assert!(!state.value_rewritten_during_creation);
    let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
        unreachable!()
    };
    assert_eq!(value.code_units(), [65, 13, 66, 10, 67]);
    assert!(
        storage
            .construction
            .node_at(node)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position())
            .is_none()
    );
    let association = storage
        .payloads
        .values
        .iter()
        .find_map(|p| match p {
            RetainedPayload::FormAssociation {
                node: Some(owner),
                state,
            } if *owner == node => Some(*state),
            _ => None,
        })
        .unwrap();
    assert!(!association.parser_inserted);
    assert_eq!(association.evidence, HtmlParseFormEvidence::Created);
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}

#[test]
fn checkbox_utf16_preserves_surrogates_and_all_actual_cr_lf_origin_rows() {
    let mut units: Vec<u16> = "<html><head></head><input type=checkbox value='"
        .encode_utf16()
        .collect();
    units.extend([0xd83d, 0xde00, 0xd800, 0x41]);
    units.extend("&#13;B&#10;C'>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        )
        .unwrap();
    let mut buffer = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_internal_value(&mut c, &node, &mut buffer)
        .unwrap();
    assert_eq!(buffer.units(), [0xd83d, 0xde00, 0xd800, 65, 13, 66, 10, 67]);
    buffer.clear(&mut c).unwrap();
    session
        .append_input_default_value(&mut c, &node, &mut buffer)
        .unwrap();
    assert_eq!(buffer.units(), [0xd83d, 0xde00, 0xd800, 65, 13, 66, 10, 67]);
    buffer.release(&mut c).unwrap();
    let storage = session.active().unwrap();
    let raw = find_input_state(&storage.payloads, &mut c, node.index)
        .unwrap()
        .0;
    let RetainedPayload::String {
        origins: Some(group),
        ..
    } = &storage.payloads.values[raw]
    else {
        unreachable!()
    };
    let evidence = storage.evidence.as_ref().unwrap();
    let mut cursor = provenance::GroupOriginCursor::new(evidence, group, &mut c).unwrap();
    let mut end = 0;
    while let Some(origin) = cursor.next(evidence, &mut c).unwrap() {
        assert_eq!(origin.start, end);
        end = origin.end;
        assert_eq!(
            origin.source.coordinates(),
            crate::HtmlSourceCoordinates::Utf16CodeUnits
        );
    }
    assert_eq!(end, 8);
    cursor.release(&mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn checkbox_original_sources_reject_foreign_or_incomplete_token_before_take() {
    for case in 0..3 {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input value='A&#13;B' type=checkbox>"),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        let wrong = if case == 1 {
            crate::HtmlSource::utf8("different").range(0, 9).unwrap()
        } else if case == 2 {
            crate::HtmlSource::utf16(surgeist_html::values::DocumentString::from("different"))
                .range(0, 9)
                .unwrap()
        } else {
            request
                .source()
                .range(token_source.start(), token_source.end() - 3)
                .unwrap()
        };
        let slots = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        let ptr = input
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .as_ptr();
        let error = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::CreationContext::Ordinary,
                },
                &wrong,
                &capability,
            )
            .unwrap_err();
        if case != 0 {
            assert_eq!(
                error,
                SessionError::Stop(HtmlParseStop::InternalInvariant(
                    crate::parser::control::HtmlParseInvariant::ForeignSource
                ))
            );
        } else {
            // The trailing token delimiter has no field mapping; narrowing
            // into the final type attribute must exclude its actual range.
            assert_eq!(error, SessionError::OriginSplitUnavailable);
        }
        assert_eq!(
            input
                .as_ref()
                .unwrap()
                .name
                .value
                .text()
                .code_units()
                .as_ptr(),
            ptr
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            slots
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            nodes
        );
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn checkbox_original_ranges_charge_two_entries_before_transfer_and_cleanup_terminal_slots() {
    let source = "<html><head></head><input value='A&#13;B' type=checkbox>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut c, mut session, _capability, lexer, _parent, mut input, _token_source) =
        input_owner(&request);
    let baseline = c
        .usage()
        .resource(HtmlParseResource::SourceMapEntries)
        .current;
    release_formatting_tag(&mut input, &mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    // One actual initialized raw group now precedes the unchanged pair
    // of original token/value-attribute source ranges.
    for extra in [2, 3] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source),
            limits()
                .into_builder()
                .source_map_entries(baseline + extra)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        let error = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceMapEntries,
                ..
            })
        ));
        if extra == 2 {
            assert!(input.is_some());
            assert!(
                !session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .any(|p| matches!(p, RetainedPayload::InputSource { .. }))
            );
        } else {
            // The fixed two original ranges fit. Later actual text-origin
            // coexistence exhausts the cap, with the source record retained.
            assert!(input.is_none());
            assert!(session.active().unwrap().payloads.values.iter().any(|p| matches!(p, RetainedPayload::InputSource { token, value_attribute: Some(_), .. } if token==&token_source)));
        }
        let first = c.stop();
        assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(_))));
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), first);
        storage_zero(&c);
    }
}

#[test]
fn checkbox_alias_foreign_parent_and_cancel_refuse_before_take_with_live_owner_recovery() {
    for case in 0..3 {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=checkbox value=x>"),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        let records = session.active().unwrap().construction.test_storage().len();
        let slots = session.active().unwrap().payloads.values.len();
        let alias = (case == 0).then(|| {
            input.as_ref().unwrap().attributes.as_slice()[1]
                .value
                .value
                .text()
                .clone()
        });
        let mut foreign = None;
        let target = if case == 1 {
            let other =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = other.root(&mut c).unwrap();
            foreign = Some(other);
            root
        } else {
            parent
        };
        if case == 2 {
            request.cancellation().cancel();
        }
        let error = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &target,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability,
            )
            .unwrap_err();
        assert_eq!(
            error,
            match case {
                0 => SessionError::AliasedPayload,
                1 => SessionError::ForeignNode,
                _ => SessionError::Stop(HtmlParseStop::Cancelled),
            }
        );
        assert!(input.is_some());
        assert_eq!(
            session.active().unwrap().construction.test_storage().len(),
            records
        );
        assert_eq!(session.active().unwrap().payloads.values.len(), slots);
        drop(alias);
        let first = c.stop();
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        if let Some(mut other) = foreign {
            other.release(&mut c).unwrap();
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), first);
    }
}

#[test]
fn checkbox_clone_partial_work_and_cancellation_keep_original_owner_and_first_stop() {
    let source = "<html><head></head><input type=checkbox value='A&#13;B&#10;C' checked>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        )
        .unwrap();
    let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session
        .active_mut()
        .unwrap()
        .clone_node(&mut c, node.index, DocumentRef::Owned(0))
        .unwrap();
    let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
    assert!(total - before > 512);
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    for tail in [0, 1, 32, 128, 512] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source),
            limits()
                .into_builder()
                .work_credits(total - tail)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        let node = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability,
            )
            .unwrap();
        let result =
            session
                .active_mut()
                .unwrap()
                .clone_node(&mut c, node.index, DocumentRef::Owned(0));
        if tail == 0 {
            assert!(result.is_ok());
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                total
            );
            assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
        }
        let first = c.stop().unwrap();
        let usage = c.usage().clone();
        assert_eq!(
            session
                .active_mut()
                .unwrap()
                .clone_node(&mut c, node.index, DocumentRef::Owned(0))
                .err(),
            Some(SessionError::Stop(first))
        );
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(first))
        );
        assert_eq!(c.usage(), usage);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(first));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            usage.resource(HtmlParseResource::WorkCredits).current
        );
    }
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        )
        .unwrap();
    request.cancellation().cancel();
    assert_eq!(
        session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, node.index, DocumentRef::Owned(0))
            .err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let usage = c.usage().clone();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        usage.resource(HtmlParseResource::WorkCredits).current
    );
}

#[test]
fn checkbox_on_fallback_exact_two_unit_admission_preserves_an_existing_output_prefix() {
    for frozen_phase in [false, true] {
        for cap in [1001, 1002] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<html><head></head><input type=checkbox>"),
                limits()
                    .into_builder()
                    .buffered_utf16_units(cap)
                    .finish()
                    .unwrap(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, range) =
                input_owner(&request);
            let node = session
                .allocate_input_for_creation(
                    &mut c,
                    &mut input,
                    CreationLocation {
                        intended_parent: &parent,
                        current_form: None,
                        context: CreationContext::Ordinary,
                    },
                    &range,
                    &capability,
                )
                .unwrap();
            session.insert(&mut c, &parent, &node, None, None).unwrap();
            lexer.release(&mut c).unwrap();
            let mut frozen = if frozen_phase {
                Some(session.freeze(&mut c).unwrap())
            } else {
                None
            };
            let id = frozen.as_ref().map(|arena| {
                let doc = arena.document().unwrap();
                let html = doc.root().children().next().unwrap();
                doc.node(&html).unwrap().children().last().unwrap()
            });
            // This append consumer legitimately has existing output. The
            // two-unit absent fallback is the only remaining contribution.
            let mut buffer = Utf16Buffer::new(&mut c).unwrap();
            buffer.extend_units(&[81; 1000], &mut c).unwrap();
            let result = if let Some(arena) = &frozen {
                arena.append_input_value(&mut c, id.as_ref().unwrap(), &mut buffer)
            } else {
                session.append_input_value(&mut c, &node, &mut buffer)
            };
            if cap == 1002 {
                result.unwrap();
                assert_eq!(&buffer.units()[..1000], &[81; 1000]);
                assert_eq!(&buffer.units()[1000..], &[111, 110]);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .high_water,
                    1002
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::BufferedUtf16Units,
                        ..
                    }))
                ));
                assert_eq!(buffer.units(), &[81; 1000]);
                let first = c.stop().unwrap();
                let usage = c.usage().clone();
                let retry = if let Some(arena) = &frozen {
                    arena.append_input_value(&mut c, id.as_ref().unwrap(), &mut buffer)
                } else {
                    session.append_input_value(&mut c, &node, &mut buffer)
                };
                assert_eq!(retry.err(), Some(SessionError::Stop(first)));
                assert_eq!(c.usage(), usage);
            }
            let first = c.stop();
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            buffer.release(&mut c).unwrap();
            if let Some(arena) = &mut frozen {
                arena.release(&mut c).unwrap();
            } else {
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
            assert_eq!(c.stop(), first);
            if first.is_some() {
                assert_eq!(
                    c.usage().resource(HtmlParseResource::WorkCredits).current,
                    work
                );
            }
        }
    }
}

#[test]
fn checkbox_ordered_creation_and_native_readbacks_keep_absence_distinct_from_empty() {
    for (attrs, expected, readback, checked, rewritten) in [
        ("type=checkbox", vec![], vec![111, 110], false, false),
        ("type=checkbox value=''", vec![], vec![], false, false),
        (
            "type=ChEcKbOx value='A&#13;B&#10;C' checked=false",
            vec![65, 13, 66, 10, 67],
            vec![65, 13, 66, 10, 67],
            true,
            false,
        ),
        (
            "value='A&#13;B&#10;C' type=checkbox checked",
            vec![65, 66, 67],
            vec![65, 66, 67],
            true,
            true,
        ),
        (
            "value='&#13;&#10;' type=checkbox",
            vec![13, 10],
            vec![13, 10],
            false,
            false,
        ),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(format!("<html><head></head><input {attrs}>")),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        let original_value = input
            .as_ref()
            .unwrap()
            .attributes
            .as_slice()
            .iter()
            .find(|attr| attr.name.value.text().code_units() == [118, 97, 108, 117, 101])
            .map(|attr| {
                (
                    attr.value.value.text().code_units().to_vec(),
                    attr.source.clone(),
                )
            });
        let node = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            )
            .unwrap();
        let state = session.input_state(&mut c, &node).unwrap();
        assert_eq!(
            state.flags.input_type(),
            surgeist_html::attributes::HtmlInputType::Checkbox
        );
        assert_eq!(state.flags.checkedness(), checked);
        assert!(
            !state.flags.dirty_value()
                && !state.flags.dirty_checkedness()
                && !state.flags.indeterminate()
                && !state.flags.user_validity()
        );
        assert_eq!(state.cursor, 0);
        assert!(state.selected_files_empty);
        assert_eq!(state.value_rewritten_during_creation, rewritten);
        if let Some((original, _)) = &original_value {
            assert_eq!(
                original,
                if attrs.starts_with("value='&#") {
                    &[13, 10][..]
                } else if attrs.contains("value=''") {
                    &[][..]
                } else {
                    &[65, 13, 66, 10, 67][..]
                }
            );
        }
        match session.input_writeback_sources(&mut c, &node).unwrap() {
            Some((token, value)) => {
                assert!(rewritten);
                assert_eq!(token, &token_source);
                assert_eq!(value, &original_value.as_ref().unwrap().1);
            }
            None => assert!(!rewritten),
        }
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &node, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), expected);
        buffer.clear(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &node, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), expected);
        buffer.clear(&mut c).unwrap();
        session
            .append_input_value(&mut c, &node, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), readback);
        buffer.release(&mut c).unwrap();
        session.insert(&mut c, &parent, &node, None, None).unwrap();
        lexer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let doc = frozen.document().unwrap();
        let html = doc.root().children().next().unwrap();
        let id = doc.node(&html).unwrap().children().last().unwrap();
        assert_eq!(frozen.input_state(&mut c, &id).unwrap(), state);
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        frozen.append_input_value(&mut c, &id, &mut buffer).unwrap();
        assert_eq!(buffer.units(), readback);
        buffer.clear(&mut c).unwrap();
        frozen
            .append_input_internal_value(&mut c, &id, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), expected);
        buffer.clear(&mut c).unwrap();
        frozen
            .append_input_default_value(&mut c, &id, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), expected);
        buffer.release(&mut c).unwrap();
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn checkbox_clone_copies_actual_dirty_current_and_four_flags_but_not_user_validity() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=checkbox value=DEFAULT checked data-current='C&#13;&#10;'><form>",
        ),
        limits(),
    );
    let (mut c, mut session, capability, mut lexer, parent, mut input, range) =
        input_owner(&request);
    let form = form_creation(&mut session, &mut lexer, &mut c, &parent, None);
    session.insert(&mut c, &parent, &form, None, None).unwrap();
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: Some(&form),
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        )
        .unwrap();
    let actual_owner = session
        .form_association_owner(&mut c, &node)
        .unwrap()
        .unwrap();
    assert!(session.same_node(&mut c, &actual_owner, &form).unwrap());
    assert!(
        session
            .form_association_state(&mut c, &node)
            .unwrap()
            .parser_inserted
    );
    let slot = session
        .active()
        .unwrap()
        .payloads
        .values
        .iter()
        .position(
            |p| matches!(p, RetainedPayload::Input { node: Some(id), .. } if *id == node.index),
        )
        .unwrap();
    let mut state = session
        .active()
        .unwrap()
        .read_input_state(&mut c, slot)
        .unwrap();
    state.flags = surgeist_html::elements::input::HtmlInputFlags::from_retained(
        surgeist_html::attributes::HtmlInputType::Checkbox,
        true,
        true,
        false,
        true,
        true,
    );
    session
        .active_mut()
        .unwrap()
        .write_input_state(&mut c, slot, state)
        .unwrap();
    // Established current backing uses an actual same-source retained
    // attribute field and its complete tokenizer origins, not a fabricated
    // originless literal accepted only by a test adapter.
    let storage = session.active().unwrap();
    let current_source = storage.attribute_value_slot(&mut c, node.index, 3).unwrap();
    let current = copy_input_value(
        &storage.payloads,
        storage.evidence.as_ref(),
        &mut c,
        current_source,
        surgeist_html::elements::input::HtmlInputSanitization::Preserve,
    )
    .unwrap();
    session
        .active_mut()
        .unwrap()
        .replace_input_raw(&mut c, slot, current)
        .unwrap();
    let copy = session
        .active_mut()
        .unwrap()
        .clone_node(&mut c, node.index, DocumentRef::Owned(0))
        .unwrap();
    let copy = session.active().unwrap().node(&mut c, copy).unwrap();
    assert_ne!(node.index, copy.index);
    assert!(
        session
            .form_association_owner(&mut c, &copy)
            .unwrap()
            .is_none()
    );
    assert!(
        !session
            .form_association_state(&mut c, &copy)
            .unwrap()
            .parser_inserted
    );
    let cloned = session.input_state(&mut c, &copy).unwrap();
    assert_eq!(
        cloned.flags.input_type(),
        surgeist_html::attributes::HtmlInputType::Checkbox
    );
    assert!(
        cloned.flags.dirty_value()
            && cloned.flags.dirty_checkedness()
            && cloned.flags.indeterminate()
    );
    assert!(!cloned.flags.checkedness() && !cloned.flags.user_validity());
    assert!(
        session
            .input_state(&mut c, &node)
            .unwrap()
            .flags
            .user_validity()
    );
    let storage = session.active().unwrap();
    let a = find_input_state(&storage.payloads, &mut c, node.index)
        .unwrap()
        .0;
    let b = find_input_state(&storage.payloads, &mut c, copy.index)
        .unwrap()
        .0;
    let RetainedPayload::String { value: a, .. } = &storage.payloads.values[a] else {
        panic!("actual raw");
    };
    let RetainedPayload::String { value: b, .. } = &storage.payloads.values[b] else {
        panic!("cloned raw");
    };
    assert!(!a.shares_storage_with(b));
    assert_eq!(a.code_units(), [67, 13, 10]);
    assert_eq!(b.code_units(), [67, 13, 10]);
    let mut buffer = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_internal_value(&mut c, &copy, &mut buffer)
        .unwrap();
    assert_eq!(buffer.units(), [67, 13, 10]);
    buffer.clear(&mut c).unwrap();
    session
        .append_input_value(&mut c, &copy, &mut buffer)
        .unwrap();
    assert_eq!(buffer.units(), [68, 69, 70, 65, 85, 76, 84]);
    buffer.release(&mut c).unwrap();
    let storage = session.active_mut().unwrap();
    let copy_slot = storage
        .payloads
        .values
        .iter()
        .position(
            |p| matches!(p, RetainedPayload::Input { node: Some(id), .. } if *id == copy.index),
        )
        .unwrap();
    storage.reset_input(&mut c, copy.index, copy_slot).unwrap();
    let reset = session.input_state(&mut c, &copy).unwrap();
    assert!(
        !reset.flags.dirty_value()
            && !reset.flags.dirty_checkedness()
            && !reset.flags.user_validity()
    );
    assert!(reset.flags.checkedness() && reset.flags.indeterminate());
    let mut buffer = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_internal_value(&mut c, &copy, &mut buffer)
        .unwrap();
    assert_eq!(buffer.units(), [68, 69, 70, 65, 85, 76, 84]);
    buffer.release(&mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn checkbox_native_getter_foreign_destination_is_first_stop_and_preserves_foreign_buffer() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head></head><input type=checkbox>"),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        )
        .unwrap();
    let mut foreign = control();
    let mut buffer = Utf16Buffer::new(&mut foreign).unwrap();
    buffer.push_unit(81, &mut foreign).unwrap();
    assert!(matches!(
        session.append_input_value(&mut c, &node, &mut buffer),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert_eq!(buffer.units(), [81]);
    let first = c.stop().unwrap();
    let usage = c.usage().clone();
    assert_eq!(
        session.append_input_value(&mut c, &node, &mut buffer).err(),
        Some(SessionError::Stop(first))
    );
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(first))
    );
    buffer.release(&mut foreign).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    storage_zero(&foreign);
    assert_eq!(c.stop(), Some(first));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        usage.resource(HtmlParseResource::WorkCredits).current
    );
}

#[test]
fn text_input_types_reset_exact_defaults_internal_values_and_selection_in_one_owner() {
    use surgeist_html::attributes::HtmlInputType;
    for (attrs, kind) in [
        ("value='A&#13;B&#10;C' checked", HtmlInputType::Text),
        ("type=TeXt value='A&#13;B&#10;C'", HtmlInputType::Text),
        ("value='A&#13;B&#10;C' type=SEARCH", HtmlInputType::Search),
        ("type=tel value='A&#13;B&#10;C'", HtmlInputType::Telephone),
        (
            "value='A&#13;B&#10;C' type=password",
            HtmlInputType::Password,
        ),
        ("type='' value='A&#13;B&#10;C'", HtmlInputType::Text),
        ("type=' hidden' value='A&#13;B&#10;C'", HtmlInputType::Text),
        ("type=bogus value='A&#13;B&#10;C'", HtmlInputType::Text),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(format!("<html><head></head><input {attrs}>")),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        let node = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            )
            .unwrap();
        assert!(input.is_none());
        let state = session.input_state(&mut c, &node).unwrap();
        assert_eq!(state.flags.input_type(), kind, "{attrs}");
        assert_eq!(state.cursor, 0);
        assert!(
            !state.flags.dirty_value()
                && !state.flags.dirty_checkedness()
                && !state.flags.indeterminate()
                && !state.flags.user_validity()
        );
        assert!(state.selected_files_empty);
        assert_eq!(state.flags.checkedness(), attrs.ends_with("checked"));
        assert!(!state.value_rewritten_during_creation);
        assert!(
            session
                .input_writeback_sources(&mut c, &node)
                .unwrap()
                .is_none()
        );
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &node, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), [65, 66, 67]);
        buffer.clear(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &node, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), [65, 13, 66, 10, 67]);
        buffer.release(&mut c).unwrap();
        session.insert(&mut c, &parent, &node, None, None).unwrap();
        lexer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let document = frozen.document().unwrap();
        let html = document.root().children().next().unwrap();
        let id = document.node(&html).unwrap().children().last().unwrap();
        assert_eq!(frozen.input_state(&mut c, &id).unwrap(), state);
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        frozen
            .append_input_internal_value(&mut c, &id, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), [65, 66, 67]);
        buffer.clear(&mut c).unwrap();
        frozen
            .append_input_default_value(&mut c, &id, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), [65, 13, 66, 10, 67]);
        buffer.release(&mut c).unwrap();
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn default_button_input_clone_retains_exact_native_default_raw_and_fresh_flags() {
    for (keyword, kind) in [
        ("button", surgeist_html::attributes::HtmlInputType::Button),
        ("reset", surgeist_html::attributes::HtmlInputType::Reset),
        ("submit", surgeist_html::attributes::HtmlInputType::Submit),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(format!(
                "<html><head></head><input type={keyword} value='A&#13;B&#10;C' checked readonly>"
            )),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        let node = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability,
            )
            .unwrap();
        assert!(input.is_none());
        let original = session.input_state(&mut c, &node).unwrap();
        assert_eq!(original.flags.input_type(), kind);
        assert!(original.flags.checkedness());
        assert!(
            !original.flags.dirty_value()
                && !original.flags.dirty_checkedness()
                && !original.flags.user_validity()
                && !original.flags.indeterminate()
        );
        assert!(original.selected_files_empty);
        assert_eq!(original.cursor, 0);
        assert!(!original.value_rewritten_during_creation);
        assert!(
            session
                .input_writeback_sources(&mut c, &node)
                .unwrap()
                .is_none()
        );
        let copy = session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, node.index, DocumentRef::Owned(0))
            .unwrap();
        let copy = session.active().unwrap().node(&mut c, copy).unwrap();
        assert_ne!(node.index, copy.index);
        assert_eq!(session.input_state(&mut c, &copy).unwrap(), original);
        assert!(
            session
                .form_association_owner(&mut c, &copy)
                .unwrap()
                .is_none()
        );
        let storage = session.active().unwrap();
        let a = find_input_state(&storage.payloads, &mut c, node.index)
            .unwrap()
            .0;
        let b = find_input_state(&storage.payloads, &mut c, copy.index)
            .unwrap()
            .0;
        let RetainedPayload::String { value: a, .. } = &storage.payloads.values[a] else {
            panic!("original raw")
        };
        let RetainedPayload::String { value: b, .. } = &storage.payloads.values[b] else {
            panic!("copy raw")
        };
        assert!(!a.shares_storage_with(b));
        assert_eq!(a.code_units(), [65, 13, 66, 10, 67]);
        assert_eq!(b.code_units(), [65, 13, 66, 10, 67]);
        // Author-prohibited checked/readonly remain actual native attributes;
        // parser construction does not discard them as author validation.
        let ConstructionNodeKind::Element(element) = &storage
            .construction
            .node_at(copy.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("native copy")
        };
        let names: Vec<_> = element
            .attributes()
            .iter()
            .map(|a| a.name().local_name().code_units().to_vec())
            .collect();
        assert_eq!(
            names,
            [
                vec![116, 121, 112, 101],
                vec![118, 97, 108, 117, 101],
                vec![99, 104, 101, 99, 107, 101, 100],
                vec![114, 101, 97, 100, 111, 110, 108, 121]
            ]
        );
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_value(&mut c, &copy, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), [65, 13, 66, 10, 67]);
        buffer.clear(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &copy, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), [65, 13, 66, 10, 67]);
        buffer.clear(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &copy, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), [65, 13, 66, 10, 67]);
        buffer.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn default_button_input_alias_foreign_parent_and_cancel_preserve_incoming_token() {
    for keyword in ["button", "reset", "submit"] {
        for case in 0..3 {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(format!(
                    "<html><head></head><input type={keyword} value=x>"
                )),
                limits(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, range) =
                input_owner(&request);
            let records = session.active().unwrap().construction.test_storage().len();
            let slots = session.active().unwrap().payloads.values.len();
            let alias = (case == 0).then(|| {
                input.as_ref().unwrap().attributes.as_slice()[1]
                    .value
                    .value
                    .text()
                    .clone()
            });
            let mut foreign = None;
            let target = if case == 1 {
                let other =
                    ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                        .unwrap();
                let root = other.root(&mut c).unwrap();
                foreign = Some(other);
                root
            } else {
                parent
            };
            if case == 2 {
                request.cancellation().cancel();
            }
            let error = session
                .allocate_input_for_creation(
                    &mut c,
                    &mut input,
                    CreationLocation {
                        intended_parent: &target,
                        current_form: None,
                        context: CreationContext::Ordinary,
                    },
                    &range,
                    &capability,
                )
                .unwrap_err();
            assert_eq!(
                error,
                match case {
                    0 => SessionError::AliasedPayload,
                    1 => SessionError::ForeignNode,
                    _ => SessionError::Stop(HtmlParseStop::Cancelled),
                }
            );
            assert!(input.is_some());
            assert_eq!(
                session.active().unwrap().construction.test_storage().len(),
                records
            );
            assert_eq!(session.active().unwrap().payloads.values.len(), slots);
            assert_eq!(
                input.as_ref().unwrap().attributes.as_slice()[1]
                    .value
                    .value
                    .text()
                    .code_units(),
                [120]
            );
            drop(alias);
            let first = c.stop();
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            if let Some(mut other) = foreign {
                other.release(&mut c).unwrap();
            }
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), first);
        }
    }
}

#[test]
fn default_button_input_exact_work_and_stopped_prefixes_keep_tickets_and_first_stop() {
    for keyword in ["button", "reset", "submit"] {
        let source =
            format!("<html><head></head><input value='A&#13;B&#10;C' type={keyword} checked>");
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source.as_str()),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability,
            )
            .unwrap();
        let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        // Real full transaction establishes the admitted boundary; exact
        // and one-short then assert different outcomes and identical cleanup.
        for tail in [0, 1, 32, 128, 512] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(source.as_str()),
                limits()
                    .into_builder()
                    .work_credits(total - tail)
                    .finish()
                    .unwrap(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, range) =
                input_owner(&request);
            let result = session.allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability,
            );
            if tail == 0 {
                assert!(result.is_ok());
                assert!(input.is_none());
                assert_eq!(
                    c.usage().resource(HtmlParseResource::WorkCredits).current,
                    total
                );
                assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
            }
            let first = c.stop().unwrap();
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            let at_stop = c.usage().resource(HtmlParseResource::WorkCredits).current;
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), Some(first));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                at_stop
            );
        }
    }
}

#[test]
fn default_button_input_two_preserved_raw_copies_have_exact_and_one_short_unit_caps() {
    for (keyword, exact) in [("button", 95), ("submit", 95), ("reset", 94)] {
        for cap in [exact - 1, exact] {
            // html name+namespace32; input name+namespace33; type/value
            // names9; type keyword6 (Reset5), original A/CR/B/LF/C5.
            // The value-attribute installation and fresh reset each retain
            // an independent five-unit raw backing, alongside original fields.
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(format!(
                    "<html><head></head><input type={keyword} value='A&#13;B&#10;C'>"
                )),
                limits()
                    .into_builder()
                    .managed_utf16_units(cap)
                    .finish()
                    .unwrap(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, range) =
                input_owner(&request);
            let result = session.allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability,
            );
            if cap == exact {
                assert!(result.is_ok());
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .high_water,
                    exact
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedUtf16Units,
                        ..
                    }))
                ));
                let first = c.stop().unwrap();
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(first))
                );
            }
            let first = c.stop();
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), first);
        }
    }
}

#[test]
fn color_invalid_authored_value_installs_black_and_preserves_native_default() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head></head><input type=color value=x>"),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
        input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &token_source,
            &capability,
        )
        .unwrap();
    assert!(input.is_none());
    let state = session.input_state(&mut c, &node).unwrap();
    assert_eq!(
        state.flags.input_type(),
        surgeist_html::attributes::HtmlInputType::Color
    );
    assert!(!state.flags.dirty_value());
    assert!(!state.value_rewritten_during_creation);
    let mut output = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_internal_value(&mut c, &node, &mut output)
        .unwrap();
    assert_eq!(output.units(), [35, 48, 48, 48, 48, 48, 48]);
    output.release(&mut c).unwrap();
    let mut output = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_default_value(&mut c, &node, &mut output)
        .unwrap();
    assert_eq!(output.units(), [120]);
    assert_eq!(
        session
            .active()
            .unwrap()
            .form_attribute(&mut c, node.index, "value")
            .unwrap()
            .unwrap()
            .code_units(),
        [120]
    );
    output.release(&mut c).unwrap();
    assert!(c.stop().is_none());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn text_input_copy_clone_preserves_actual_type_and_independent_internal_default_owners() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=password value='A&#13;B&#10;C' checked>",
        ),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
        input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &token_source,
            &capability,
        )
        .unwrap();
    let original = session.input_state(&mut c, &node).unwrap();
    let storage = session.active_mut().unwrap();
    let copy = storage
        .clone_node(&mut c, node.index, DocumentRef::Owned(0))
        .unwrap();
    let copy = storage.node(&mut c, copy).unwrap();
    assert_ne!(node.index, copy.index);
    assert_eq!(session.input_state(&mut c, &copy).unwrap(), original);
    let storage = session.active().unwrap();
    let raw_a = find_input_state(&storage.payloads, &mut c, node.index)
        .unwrap()
        .0;
    let raw_b = find_input_state(&storage.payloads, &mut c, copy.index)
        .unwrap()
        .0;
    let RetainedPayload::String { value: a, .. } = &storage.payloads.values[raw_a] else {
        unreachable!()
    };
    let RetainedPayload::String { value: b, .. } = &storage.payloads.values[raw_b] else {
        unreachable!()
    };
    assert!(!a.shares_storage_with(b));
    assert_eq!(a.code_units(), [65, 66, 67]);
    assert_eq!(b.code_units(), [65, 66, 67]);
    let mut buffer = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_default_value(&mut c, &copy, &mut buffer)
        .unwrap();
    assert_eq!(buffer.units(), [65, 13, 66, 10, 67]);
    buffer.release(&mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn text_input_exact_work_and_one_short_preserve_original_stop_and_dispose_all_prefixes() {
    let source = "<html><head></head><input value='A&#13;B&#10;C' type=search checked>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        )
        .unwrap();
    let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    // Measure the complete real issuer transaction, then verify its exact
    // boundary and multiple actual partial allocation/write prefixes.
    for tail in [0, 1, 32, 128, 512, 1024, 2048] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source),
            limits()
                .into_builder()
                .work_credits(total - tail)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        let result = session.allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        );
        if tail == 0 {
            assert!(result.is_ok());
            assert!(input.is_none());
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                total
            );
            assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
        }
        let first = c.stop().unwrap();
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(first))
        );
        let at_stop = c.usage().resource(HtmlParseResource::WorkCredits).current;
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(first));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            at_stop
        );
    }
}

#[test]
fn text_input_reset_cancellation_retains_completed_value_before_association_and_cleanup() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=tel value='A&#13;B&#10;C' checked>",
        ),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    session.active_mut().unwrap().cancel_after_input_reset = Some(request.cancellation().clone());
    assert_eq!(
        session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability
            )
            .err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert!(input.is_none());
    let storage = session.active().unwrap();
    let (node, raw, state) = storage
        .payloads
        .values
        .iter()
        .find_map(|p| match p {
            RetainedPayload::Input {
                node: Some(node),
                raw,
                complete: true,
                state,
                ..
            } => Some((*node, *raw, *state)),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        state.flags.input_type(),
        surgeist_html::attributes::HtmlInputType::Telephone
    );
    assert_eq!(state.cursor, 0);
    assert!(state.flags.checkedness());
    assert!(!state.value_rewritten_during_creation);
    let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
        unreachable!()
    };
    assert_eq!(value.code_units(), [65, 66, 67]);
    assert!(
        storage
            .construction
            .node_at(node)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position())
            .is_none()
    );
    let association = storage
        .payloads
        .values
        .iter()
        .find_map(|p| match p {
            RetainedPayload::FormAssociation {
                node: Some(owner),
                state,
            } if *owner == node => Some(*state),
            _ => None,
        })
        .unwrap();
    assert!(!association.parser_inserted);
    assert_eq!(association.evidence, HtmlParseFormEvidence::Created);
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}

#[test]
fn text_input_utf16_preserves_surrogates_and_filters_only_actual_cr_lf_origin_rows() {
    let mut units: Vec<u16> = "<html><head></head><input type=password value='"
        .encode_utf16()
        .collect();
    units.extend([0xd83d, 0xde00, 0xd800, 0x41]);
    units.extend("&#13;B&#10;C'>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        )
        .unwrap();
    let mut buffer = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_internal_value(&mut c, &node, &mut buffer)
        .unwrap();
    assert_eq!(buffer.units(), [0xd83d, 0xde00, 0xd800, 65, 66, 67]);
    buffer.clear(&mut c).unwrap();
    session
        .append_input_default_value(&mut c, &node, &mut buffer)
        .unwrap();
    assert_eq!(buffer.units(), [0xd83d, 0xde00, 0xd800, 65, 13, 66, 10, 67]);
    buffer.release(&mut c).unwrap();
    let storage = session.active().unwrap();
    let raw = find_input_state(&storage.payloads, &mut c, node.index)
        .unwrap()
        .0;
    let RetainedPayload::String {
        origins: Some(group),
        ..
    } = &storage.payloads.values[raw]
    else {
        unreachable!()
    };
    let evidence = storage.evidence.as_ref().unwrap();
    let mut cursor = provenance::GroupOriginCursor::new(evidence, group, &mut c).unwrap();
    let mut end = 0;
    while let Some(origin) = cursor.next(evidence, &mut c).unwrap() {
        assert_eq!(origin.start, end);
        end = origin.end;
        assert_eq!(
            origin.source.coordinates(),
            crate::HtmlSourceCoordinates::Utf16CodeUnits
        );
    }
    assert_eq!(end, 6);
    cursor.release(&mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn text_input_alias_foreign_parent_and_cancel_refuse_before_take_with_live_owner_recovery() {
    for case in 0..3 {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=search value=x>"),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        let records = session.active().unwrap().construction.test_storage().len();
        let slots = session.active().unwrap().payloads.values.len();
        let alias = (case == 0).then(|| {
            input.as_ref().unwrap().attributes.as_slice()[1]
                .value
                .value
                .text()
                .clone()
        });
        let mut foreign = None;
        let target = if case == 1 {
            let other =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = other.root(&mut c).unwrap();
            foreign = Some(other);
            root
        } else {
            parent
        };
        if case == 2 {
            request.cancellation().cancel();
        }
        let error = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &target,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability,
            )
            .unwrap_err();
        assert_eq!(
            error,
            match case {
                0 => SessionError::AliasedPayload,
                1 => SessionError::ForeignNode,
                _ => SessionError::Stop(HtmlParseStop::Cancelled),
            }
        );
        assert!(input.is_some());
        assert_eq!(
            session.active().unwrap().construction.test_storage().len(),
            records
        );
        assert_eq!(session.active().unwrap().payloads.values.len(), slots);
        drop(alias);
        let first = c.stop();
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        if let Some(mut other) = foreign {
            other.release(&mut c).unwrap();
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), first);
    }
}

#[test]
fn text_input_source_unit_coexistence_exact_and_one_short_own_every_raw_copy() {
    for cap in [100, 101] {
        // html name+namespace=32, input name+namespace=33, attribute
        // names=16 and original values=11: 92 lexical/native units.
        // Value insertion, type sanitization, and creation reset each
        // own one independent ABC (3 units), with original leaves retained.
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(
                "<html><head></head><input value='A&#13;B&#10;C' type=search checked>",
            ),
            limits()
                .into_builder()
                .managed_utf16_units(cap)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        let result = session.allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        );
        if cap == 101 {
            assert!(result.is_ok());
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .high_water,
                101
            );
        } else {
            let first = limit_stop_of(&result, HtmlParseResource::ManagedUtf16Units, cap);
            assert_freeze_reports_first_stop(&mut session, &mut c, first);
        }
        let first = c.stop();
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), first);
    }
}
#[test]
fn text_input_same_state_type_installation_skips_type_change_only_effects() {
    use surgeist_html::attributes::HtmlInputType;
    for (keyword, kind, units) in [
        ("text", HtmlInputType::Text, 89),
        ("bogus", HtmlInputType::Text, 90),
        ("search", HtmlInputType::Search, 94),
    ] {
        for short in [false, true] {
            // Names/namespaces html32+input33, attribute names9, and
            // values5+keyword length. The value-change and reset paths
            // each own ABC. Only actual Text→Search additionally runs
            // the type-change sanitizer; Text→Text has no such trigger.
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(format!(
                    "<html><head></head><input value='A&#13;B&#10;C' type={keyword}>"
                )),
                limits()
                    .into_builder()
                    .managed_utf16_units(units - u64::from(short))
                    .finish()
                    .unwrap(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, range) =
                input_owner(&request);
            let result = session.allocate_input_for_creation(
                &mut c,
                &mut input,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &capability,
            );
            if !short && result.is_err() {
                let error = result.err();
                let first = c.stop();
                release_formatting_tag(&mut input, &mut c).unwrap();
                lexer.release(&mut c).unwrap();
                session.release(&mut c).unwrap();
                storage_zero(&c);
                assert_eq!(c.stop(), first);
                assert_eq!(error, None, "{keyword}: source-required work fits");
                return;
            }
            if short {
                let first = limit_stop_of(&result, HtmlParseResource::ManagedUtf16Units, units - 1);
                assert_freeze_reports_first_stop(&mut session, &mut c, first);
            } else {
                let node = result.unwrap();
                assert_eq!(
                    session
                        .input_state(&mut c, &node)
                        .unwrap()
                        .flags
                        .input_type(),
                    kind
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .high_water,
                    units
                );
                assert!(
                    session
                        .input_writeback_sources(&mut c, &node)
                        .unwrap()
                        .is_none()
                );
                let mut buffer = Utf16Buffer::new(&mut c).unwrap();
                session
                    .append_input_internal_value(&mut c, &node, &mut buffer)
                    .unwrap();
                assert_eq!(buffer.units(), [65, 66, 67]);
                buffer.clear(&mut c).unwrap();
                session
                    .append_input_default_value(&mut c, &node, &mut buffer)
                    .unwrap();
                assert_eq!(buffer.units(), [65, 13, 66, 10, 67]);
                buffer.release(&mut c).unwrap();
            }
            let stop = c.stop();
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
    }
}
#[test]
fn hidden_input_sanitization_preserves_exact_utf16_origin_groups_and_original_evidence() {
    let mut units: Vec<u16> = "<html><head></head><input value='".encode_utf16().collect();
    units.extend([0xd83d, 0xde00, 0xd800, 65]);
    units.extend("&#13;B&#10;C' type=hidden>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
        input_owner(&request);
    let original_value_source = input.as_ref().unwrap().attributes.as_slice()[0]
        .source
        .clone();
    let expected = [0xd83d, 0xde00, 0xd800, 65, 66, 67];
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            crate::construction::CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: crate::construction::CreationContext::Ordinary,
            },
            &token_source,
            &capability,
        )
        .unwrap();
    let (token, value) = session
        .input_writeback_sources(&mut c, &node)
        .unwrap()
        .unwrap();
    assert_eq!(token, &token_source);
    assert_eq!(value, &original_value_source);
    assert_eq!(
        token.coordinates(),
        crate::HtmlSourceCoordinates::Utf16CodeUnits
    );
    let source_slot = session
        .active()
        .unwrap()
        .attribute_value_slot(&mut c, node.index, 0)
        .unwrap();
    let RetainedPayload::String {
        value,
        origins: Some(origins),
        ..
    } = &session.active().unwrap().payloads.values[source_slot]
    else {
        unreachable!()
    };
    assert_eq!(value.code_units(), expected);
    assert_eq!(
        session
            .active()
            .unwrap()
            .observed_origins(origins, &mut c)
            .unwrap()
            .iter()
            .map(|o| (o.start, o.end))
            .collect::<Vec<_>>(),
        [(0, 2), (2, 3), (3, 4), (4, 5), (5, 6)]
    );
    assert!(
        session
            .active()
            .unwrap()
            .observed_origins(origins, &mut c)
            .unwrap()
            .iter()
            .all(|o| o.kind != crate::parser::tokenizer::token::HtmlParseOriginKind::RecoverySyntax)
    );
    let originals = session
        .active()
        .unwrap()
        .payloads
        .values
        .iter()
        .find_map(|p| match p {
            RetainedPayload::String {
                value,
                origins: Some(origins),
                location: StringLocation::Detached,
                ..
            } if value.code_units() == [0xd83d, 0xde00, 0xd800, 65, 13, 66, 10, 67] => Some(
                session
                    .active()
                    .unwrap()
                    .observed_origins(origins, &mut c)
                    .unwrap(),
            ),
            _ => None,
        })
        .unwrap();
    assert_eq!(originals.len(), 7);
    assert_eq!(
        originals[4].source,
        session
            .active()
            .unwrap()
            .observed_origins(origins, &mut c)
            .unwrap()[3]
            .source
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn hidden_input_foreign_proof_alias_and_generic_keep_independent_color_admission() {
    for case in 0..3 {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=hidden value=x>"),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        if case == 0 {
            let alias = input.as_ref().unwrap().attributes.as_slice()[1]
                .value
                .value
                .text()
                .clone();
            assert_eq!(
                session
                    .allocate_input_for_creation(
                        &mut c,
                        &mut input,
                        crate::construction::CreationLocation {
                            intended_parent: &parent,
                            current_form: None,
                            context: crate::construction::CreationContext::Ordinary
                        },
                        &token_source,
                        &capability
                    )
                    .err(),
                Some(SessionError::AliasedPayload)
            );
            assert!(input.is_some());
            drop(alias);
        } else if case == 1 {
            let mut other =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = other.root(&mut c).unwrap();
            assert!(matches!(
                other.allocate_input_for_creation(
                    &mut c,
                    &mut input,
                    crate::construction::CreationLocation {
                        intended_parent: &root,
                        current_form: None,
                        context: crate::construction::CreationContext::Ordinary
                    },
                    &token_source,
                    &capability
                ),
                Err(SessionError::Stop(
                    HtmlParseStop::UnsupportedStandaloneState(
                        HtmlParseUnsupportedState::Provenance
                    )
                ))
            ));
            assert!(input.is_some());
            other.release(&mut c).unwrap();
        } else {
            let generic = session
                .allocate_html_tag_for_creation(
                    &mut c,
                    &mut input,
                    &parent,
                    None,
                    crate::construction::CreationContext::Ordinary,
                )
                .unwrap();
            assert_eq!(
                session.input_state(&mut c, &generic),
                Err(SessionError::InputMetadataUnavailable)
            );
            assert!(matches!(
                session.input_writeback_sources(&mut c, &generic),
                Err(SessionError::InputMetadataUnavailable)
            ));
        }
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head></head><input type=color>"),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
        input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &token_source,
            &capability,
        )
        .unwrap();
    assert!(input.is_none());
    let state = session.input_state(&mut c, &node).unwrap();
    assert_eq!(
        state.flags.input_type(),
        surgeist_html::attributes::HtmlInputType::Color
    );
    assert!(!state.flags.dirty_value());
    assert!(!state.value_rewritten_during_creation);
    assert!(
        session
            .active()
            .unwrap()
            .form_attribute(&mut c, node.index, "value")
            .unwrap()
            .is_none()
    );
    let mut output = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_internal_value(&mut c, &node, &mut output)
        .unwrap();
    assert_eq!(output.units(), [35, 48, 48, 48, 48, 48, 48]);
    output.release(&mut c).unwrap();
    let mut output = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_default_value(&mut c, &node, &mut output)
        .unwrap();
    assert!(output.units().is_empty());
    output.release(&mut c).unwrap();
    assert!(c.stop().is_none());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn hidden_input_every_stopped_writeback_tail_retains_alias_valid_owners_and_first_stop() {
    let source = "<html><head></head><input value='A&#13;B&#10;C' type=hidden checked>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
        input_owner(&request);
    session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            crate::construction::CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: crate::construction::CreationContext::Ordinary,
            },
            &token_source,
            &capability,
        )
        .unwrap();
    let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    let mut partial = false;
    for tail in (1..=2500).step_by(25) {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source),
            limits()
                .into_builder()
                .work_credits(total - tail)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        assert!(matches!(
            session.allocate_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::CreationContext::Ordinary
                },
                &token_source,
                &capability
            ),
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }))
        ));
        let first = c.stop().unwrap();
        partial |= session.active().unwrap().payloads.values.iter().any(|p| matches!(p,RetainedPayload::Input { complete:false,state,.. } if state.value_rewritten_during_creation));
        assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first));
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(first));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
    }
    assert!(
        partial,
        "a real stopped writeback/reset prefix was exercised"
    );
}

#[test]
fn hidden_input_raw_rewrite_coexistence_exact_104_units_and_one_short_reclaims_every_backing() {
    for cap in [103, 104] {
        // Fresh html=4+28. New input=5+28; source attribute names=5+4+7
        // and values=5+6+0. The retained lexical total is92. Sanitized raw,
        // replacement native value and a coexisting next raw each add3.
        // The first transformed ABC leaf remains reconstructible when its
        // raw slot is replaced; one later raw therefore adds3 over old101.
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(
                "<html><head></head><input value='A&#13;B&#10;C' type=hidden checked>",
            ),
            limits()
                .into_builder()
                .managed_utf16_units(cap)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        let result = session.allocate_input_for_creation(
            &mut c,
            &mut input,
            crate::construction::CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: crate::construction::CreationContext::Ordinary,
            },
            &token_source,
            &capability,
        );
        if cap == 104 {
            let node = result.unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .high_water,
                104
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                101
            );
            assert_eq!(
                session
                    .active()
                    .unwrap()
                    .construction
                    .test_storage()
                    .usage()
                    .utf16_units(),
                90
            );
            assert!(
                session
                    .input_state(&mut c, &node)
                    .unwrap()
                    .flags
                    .checkedness()
            );
            assert_eq!(
                session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .filter(|v| matches!(
                        v,
                        RetainedPayload::Retired | RetainedPayload::HistoricalString { .. }
                    ))
                    .count(),
                2
            );
        } else {
            let first = limit_stop_of(&result, HtmlParseResource::ManagedUtf16Units, cap);
            assert_freeze_reports_first_stop(&mut session, &mut c, first);
        }
        let stop = c.stop();
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }
}
#[test]
fn hidden_input_cancel_after_real_reset_precedes_parser_association() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><form><input value='A&#13;B' type=hidden checked>",
        ),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
        input_owner(&request);
    let mut form_lexer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut form = Some(next_fixture_tag(
        &mut form_lexer,
        &mut c,
        &[102, 111, 114, 109],
    ));
    let form = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut form,
            &parent,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    session.insert(&mut c, &parent, &form, None, None).unwrap();
    form_lexer.release(&mut c).unwrap();
    session.active_mut().unwrap().cancel_after_input_reset = Some(request.cancellation().clone());
    assert!(matches!(
        session.allocate_input_for_creation(
            &mut c,
            &mut input,
            crate::construction::CreationLocation {
                intended_parent: &form,
                current_form: Some(&form),
                context: crate::construction::CreationContext::Ordinary
            },
            &token_source,
            &capability
        ),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    assert!(input.is_none());
    let storage = session.active().unwrap();
    let (node, raw, state) = storage
        .payloads
        .values
        .iter()
        .find_map(|v| match v {
            RetainedPayload::Input {
                node: Some(node),
                raw,
                complete: true,
                state,
                ..
            } => Some((*node, *raw, *state)),
            _ => None,
        })
        .unwrap();
    assert!(
        !state.flags.dirty_value()
            && !state.flags.dirty_checkedness()
            && !state.flags.user_validity()
    );
    assert!(state.flags.checkedness() && state.value_rewritten_during_creation);
    assert_eq!(
        textarea_raw(&storage.payloads, &mut c, raw).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
        unreachable!()
    };
    assert_eq!(value.code_units(), [65, 66]);
    let association = storage
        .payloads
        .values
        .iter()
        .find_map(|v| match v {
            RetainedPayload::FormAssociation {
                node: Some(owner),
                state,
            } if *owner == node => Some(*state),
            _ => None,
        })
        .unwrap();
    assert_eq!(association.owner, None);
    assert!(!association.parser_inserted);
    assert_eq!(association.evidence, HtmlParseFormEvidence::Created);
    assert!(
        storage
            .construction
            .node_at(node)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position())
            .is_none()
    );
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn hidden_input_value_getters_qualify_destination_and_refuse_imported_and_cancelled_state() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head></head><input type=hidden value=x>"),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
        input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            crate::construction::CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: crate::construction::CreationContext::Ordinary,
            },
            &token_source,
            &capability,
        )
        .unwrap();
    let mut foreign = control();
    let mut buffer = Utf16Buffer::new(&mut foreign).unwrap();
    buffer.push_unit(81, &mut foreign).unwrap();
    assert!(matches!(
        session.append_input_default_value(&mut c, &node, &mut buffer),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert_eq!(buffer.units(), [81]);
    let first = c.stop();
    assert_eq!(
        session.input_state(&mut c, &node).err(),
        first.map(SessionError::Stop)
    );
    assert_eq!(session.freeze(&mut c).err(), first.map(SessionError::Stop));
    buffer.release(&mut foreign).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    storage_zero(&foreign);
}

#[test]
fn hidden_input_original_sources_reject_foreign_or_incomplete_token_before_take() {
    for case in 0..3 {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input value='A&#13;B' type=hidden>"),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        let wrong = if case == 1 {
            crate::HtmlSource::utf8("different").range(0, 9).unwrap()
        } else if case == 2 {
            crate::HtmlSource::utf16(surgeist_html::values::DocumentString::from("different"))
                .range(0, 9)
                .unwrap()
        } else {
            request
                .source()
                .range(token_source.start(), token_source.end() - 3)
                .unwrap()
        };
        let slots = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        let ptr = input
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .as_ptr();
        let error = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::CreationContext::Ordinary,
                },
                &wrong,
                &capability,
            )
            .unwrap_err();
        if case != 0 {
            assert_eq!(
                error,
                SessionError::Stop(HtmlParseStop::InternalInvariant(
                    crate::parser::control::HtmlParseInvariant::ForeignSource
                ))
            );
        } else {
            // The trailing token delimiter has no field mapping; narrowing
            // into the final type attribute must exclude its actual range.
            assert_eq!(error, SessionError::OriginSplitUnavailable);
        }
        assert_eq!(
            input
                .as_ref()
                .unwrap()
                .name
                .value
                .text()
                .code_units()
                .as_ptr(),
            ptr
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            slots
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            nodes
        );
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn hidden_input_original_ranges_charge_two_entries_before_transfer_and_cleanup_terminal_slots() {
    let source = "<html><head></head><input value='A&#13;B' type=hidden>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut c, mut session, _capability, lexer, _parent, mut input, _token_source) =
        input_owner(&request);
    let baseline = c
        .usage()
        .resource(HtmlParseResource::SourceMapEntries)
        .current;
    release_formatting_tag(&mut input, &mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    // One actual initialized raw group now precedes the unchanged pair
    // of original token/value-attribute source ranges.
    for extra in [2, 3] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source),
            limits()
                .into_builder()
                .source_map_entries(baseline + extra)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            input_owner(&request);
        let error = session
            .allocate_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceMapEntries,
                ..
            })
        ));
        if extra == 2 {
            assert!(input.is_some());
            assert!(
                !session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .any(|p| matches!(p, RetainedPayload::InputSource { .. }))
            );
        } else {
            // The fixed two original ranges fit. Later actual text-origin
            // coexistence exhausts the cap, with the source record retained.
            assert!(input.is_none());
            assert!(session.active().unwrap().payloads.values.iter().any(|p| matches!(p, RetainedPayload::InputSource { token, value_attribute: Some(_), .. } if token==&token_source)));
        }
        let first = c.stop();
        assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(_))));
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), first);
        storage_zero(&c);
    }
}

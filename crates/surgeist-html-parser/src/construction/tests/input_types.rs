use super::*;
use surgeist_html::construction::HtmlDocumentMode;

#[test]
fn email_multiple_current_origins_and_clone_keep_actual_count_and_source() {
    for (attribute, expected, expected_count) in [
        ("value=' a&#10;b , &#9;c '", vec![97, 10, 98, 44, 99], 2),
        ("value='a, '", vec![97, 44], 2),
    ] {
        let source = format!("<html><head></head><input type=email multiple {attribute}>");
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
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
        let original = session.input_state(&mut c, &node).unwrap();
        let raw = find_input_state(&session.active().unwrap().payloads, &mut c, node.index)
            .unwrap()
            .0;
        let observed = {
            let storage = session.active().unwrap();
            let RetainedPayload::String {
                value,
                origins: Some(origins),
                ..
            } = &storage.payloads.values[raw]
            else {
                panic!("actual current field");
            };
            assert_eq!(value.code_units(), expected);
            storage
                .observed_origins(origins, &mut c)
                .unwrap()
                .iter()
                .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                .collect::<Vec<_>>()
        };
        let copy = session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, node.index, DocumentRef::Owned(0))
            .unwrap();
        let copy = session.active().unwrap().node(&mut c, copy).unwrap();
        let cloned = session.input_state(&mut c, &copy).unwrap();
        let copy_raw = find_input_state(&session.active().unwrap().payloads, &mut c, copy.index)
            .unwrap()
            .0;
        let independent = {
            let storage = session.active().unwrap();
            let RetainedPayload::String { value: a, .. } = &storage.payloads.values[raw] else {
                unreachable!()
            };
            let RetainedPayload::String { value: b, .. } = &storage.payloads.values[copy_raw]
            else {
                unreachable!()
            };
            assert_eq!(b.code_units(), expected);
            !a.shares_storage_with(b)
        };
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &copy, &mut buffer)
            .unwrap();
        let default = buffer.units().to_vec();
        buffer.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.stop(), None);
        assert!(independent);
        assert_eq!(original.email_value_count, Some(expected_count));
        assert_eq!(cloned, original);
        if attribute.starts_with("value=' ") {
            assert_eq!(
                observed,
                [
                    (0, 1, 54, 55),
                    (1, 2, 55, 60),
                    (2, 3, 60, 61),
                    (3, 4, 62, 63),
                    (4, 5, 68, 69)
                ]
            );
            assert_eq!(default, [32, 97, 10, 98, 32, 44, 32, 9, 99, 32]);
        } else {
            assert_eq!(default, [97, 44, 32]);
        }
    }
}

#[test]
fn email_url_reset_cancellation_retains_sanitized_value_and_actual_count_before_association() {
    for (source, expected, count) in [
        (
            "<html><head></head><input type=url value=' a&#13;b&#10;c '>",
            vec![97, 98, 99],
            None,
        ),
        (
            "<html><head></head><input type=email value=' a&#13;b&#10;c '>",
            vec![97, 98, 99],
            None,
        ),
        (
            "<html><head></head><input type=email multiple value=' a&#10;b , c '>",
            vec![97, 10, 98, 44, 99],
            Some(2),
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        session.active_mut().unwrap().cancel_after_input_reset =
            Some(request.cancellation().clone());
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
        assert_eq!(
            result.err(),
            Some(SessionError::Stop(HtmlParseStop::Cancelled))
        );
        assert!(input.is_none());
        let (actual, observed_count, attached, parser_inserted) = {
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
            let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
                unreachable!()
            };
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
            assert!(!state.flags.dirty_value());
            assert!(!state.flags.dirty_checkedness());
            (
                value.code_units().to_vec(),
                state.email_value_count,
                storage
                    .construction
                    .node_at(node)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position())
                    .is_some(),
                association.parser_inserted,
            )
        };
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(HtmlParseStop::Cancelled))
        );
        let stopped_work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            stopped_work
        );
        assert_eq!(actual, expected);
        assert_eq!(observed_count, count);
        assert!(!attached);
        assert!(!parser_inserted);
    }
}

#[test]
fn email_url_exact_work_and_stopped_copy_prefixes_release_same_actual_owners() {
    for source in [
        "<html><head></head><input type=url value=' &#9;a&#13;b&#10;c '>",
        "<html><head></head><input type=email value=' &#9;a&#13;b&#10;c '>",
        "<html><head></head><input type=email multiple value=' a&#10;b , &#9;c '>",
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
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
        let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let state = session.input_state(&mut c, &node).unwrap();
        let mut output = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_value(&mut c, &node, &mut output)
            .unwrap();
        let value = output.units().to_vec();
        output.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(
            value,
            if source.contains("multiple") {
                vec![97, 10, 98, 44, 99]
            } else {
                vec![97, 98, 99]
            }
        );
        assert_eq!(
            state.email_value_count,
            source.contains("multiple").then_some(2)
        );
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
            let stopped_work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.usage().actual_capacity_bytes, 0);
            assert_eq!(c.stop(), Some(first));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                stopped_work
            );
        }
    }
}
#[test]
fn temporal_input_reset_cancellation_retains_real_current_before_association() {
    for (source, expected, count) in [
        (
            "<html><head></head><input type=date value='2024-02-29'>",
            "2024-02-29".encode_utf16().collect::<Vec<_>>(),
            None,
        ),
        (
            "<html><head></head><input type=time value='12:34:56.1234'>",
            vec![],
            None,
        ),
        (
            "<html><head></head><input type=datetime-local value='2024-02-29&#32;12:34:00.000'>",
            "2024-02-29T12:34".encode_utf16().collect::<Vec<_>>(),
            None,
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        session.active_mut().unwrap().cancel_after_input_reset =
            Some(request.cancellation().clone());
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
        assert_eq!(
            result.err(),
            Some(SessionError::Stop(HtmlParseStop::Cancelled))
        );
        assert!(input.is_none());
        let (actual, observed_count, attached, parser_inserted) = {
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
            let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
                unreachable!()
            };
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
            assert!(!state.flags.dirty_value());
            assert!(!state.flags.dirty_checkedness());
            (
                value.code_units().to_vec(),
                state.email_value_count,
                storage
                    .construction
                    .node_at(node)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position())
                    .is_some(),
                association.parser_inserted,
            )
        };
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(HtmlParseStop::Cancelled))
        );
        let stopped_work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            stopped_work
        );
        assert_eq!(actual, expected);
        assert_eq!(observed_count, count);
        assert!(!attached);
        assert!(!parser_inserted);
    }
}

#[test]
fn temporal_input_exact_work_and_stopped_scan_copy_prefixes_release_actual_owners() {
    for (source, expected) in [
        (
            "<html><head></head><input type=date value='002024-02-29'>",
            "002024-02-29",
        ),
        ("<html><head></head><input type=week value='2021-W53'>", ""),
        (
            "<html><head></head><input type=datetime-local value='2024-02-29&#32;12:34:01.120'>",
            "2024-02-29T12:34:01.12",
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
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
        let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let state = session.input_state(&mut c, &node).unwrap();
        let mut output = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_value(&mut c, &node, &mut output)
            .unwrap();
        let value = output.units().to_vec();
        output.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(value, expected.encode_utf16().collect::<Vec<_>>());
        assert_eq!(state.email_value_count, None);
        assert!(!state.flags.input_type().is_selectable());
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
            let stopped_work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.usage().actual_capacity_bytes, 0);
            assert_eq!(c.stop(), Some(first));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                stopped_work
            );
        }
    }
}
#[test]
fn temporal_input_ordered_type_reset_and_clone_keep_normalization_cause_and_exact_default() {
    use crate::parser::tokenizer::token::HtmlParseOriginKind;
    for (source, source_start, source_end) in [
        (
            "<html><head></head><input value='2024-02-29&#32;12:34:01.120' type=datetime-local>",
            43,
            48,
        ),
        (
            "<html><head></head><input type=datetime-local value='2024-02-29 12:34:01.120'>",
            63,
            64,
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
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
        let before = session.input_state(&mut c, &node).unwrap();
        let raw = find_input_state(&session.active().unwrap().payloads, &mut c, node.index)
            .unwrap()
            .0;
        let normalized = {
            let storage = session.active().unwrap();
            let RetainedPayload::String {
                value,
                origins: Some(group),
                ..
            } = &storage.payloads.values[raw]
            else {
                panic!("actual current owner");
            };
            let value = value.code_units().to_vec();
            let normalized = storage
                .observed_origins(group, &mut c)
                .unwrap()
                .into_iter()
                .filter(|origin| origin.kind == HtmlParseOriginKind::InputValueNormalization)
                .map(|origin| {
                    (
                        origin.start,
                        origin.end,
                        origin.source.source().clone(),
                        origin.source.start(),
                        origin.source.end(),
                    )
                })
                .collect::<Vec<_>>();
            (value, normalized)
        };
        let copy = session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, node.index, DocumentRef::Owned(0))
            .unwrap();
        let copy = session.active().unwrap().node(&mut c, copy).unwrap();
        let clone_state = session.input_state(&mut c, &copy).unwrap();
        let copy_raw = find_input_state(&session.active().unwrap().payloads, &mut c, copy.index)
            .unwrap()
            .0;
        let independent = {
            let storage = session.active().unwrap();
            let RetainedPayload::String { value: a, .. } = &storage.payloads.values[raw] else {
                unreachable!()
            };
            let RetainedPayload::String { value: b, .. } = &storage.payloads.values[copy_raw]
            else {
                unreachable!()
            };
            !a.shares_storage_with(b)
        };
        let storage = session.active_mut().unwrap();
        let copy_slot = storage.payloads.values.iter().position(
            |payload| matches!(payload, RetainedPayload::Input { node: Some(id), .. } if *id == copy.index)
        ).unwrap();
        storage.reset_input(&mut c, copy.index, copy_slot).unwrap();
        let reset = session.input_state(&mut c, &copy).unwrap();
        let mut output = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &copy, &mut output)
            .unwrap();
        let reset_current = output.units().to_vec();
        output.clear(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &copy, &mut output)
            .unwrap();
        let default = output.units().to_vec();
        output.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.stop(), None);
        assert!(independent);
        assert_eq!(before, clone_state);
        assert_eq!(
            reset.flags.input_type(),
            surgeist_html::attributes::HtmlInputType::LocalDateTime
        );
        assert!(
            !reset.flags.dirty_value()
                && !reset.flags.dirty_checkedness()
                && !reset.flags.user_validity()
        );
        assert!(!reset.flags.input_type().is_selectable());
        assert_eq!(
            normalized.0,
            "2024-02-29T12:34:01.12".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(reset_current, normalized.0);
        assert_eq!(
            default,
            "2024-02-29 12:34:01.120".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            normalized.1,
            [(
                10,
                11,
                request.source().identity().clone(),
                source_start,
                source_end
            )]
        );
    }
}
#[test]
fn number_input_exact_managed_units_owns_every_independent_current_copy() {
    for cap in [101, 102] {
        // Real html/input names+namespaces: 32+33; three attribute names:
        // value5+type4+checked7; values: 1e999(5)+number(6).
        // Value-before-type performs three independent five-unit copies:
        // initial Text value, Number type sanitizer, fresh creation reset.
        // Each replacement releases the preceding aggregate after the new
        // copy is admitted, so at most two five-unit copies coexist.
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input value=1e999 type=number checked>"),
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
        if cap == 102 {
            assert!(result.is_ok());
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .high_water,
                102
            );
        } else {
            let first = limit_stop_of(&result, HtmlParseResource::ManagedUtf16Units, cap);
            assert_freeze_reports_first_stop(&mut session, &mut c, first);
        }
        let first = c.stop();
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.stop(), first);
        if first.is_some() {
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                work
            );
        }
    }
}

#[test]
fn number_input_exact_work_partial_grammar_and_copy_stops_keep_all_owners() {
    for (source, expected) in [
        (
            "<html><head></head><input value=1e999 type=number>",
            "1e999",
        ),
        ("<html><head></head><input type=number value=12junk>", ""),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
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
        let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let state = session.input_state(&mut c, &node).unwrap();
        let mut output = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &node, &mut output)
            .unwrap();
        let actual = output.units().to_vec();
        output.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(actual, expected.encode_utf16().collect::<Vec<_>>());
        assert_eq!(
            state.flags.input_type(),
            surgeist_html::attributes::HtmlInputType::Number
        );
        assert_eq!(state.email_value_count, None);
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
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.usage().actual_capacity_bytes, 0);
            assert_eq!(c.stop(), Some(first));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                work
            );
        }
    }
}

#[test]
fn number_input_clone_reset_and_original_source_keep_exact_independent_strings() {
    for (value, expected) in [("1e999", "1e999"), ("12junk", "")] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(format!(
                "<html><head></head><input value={value} type=number checked>"
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
        let before = session.input_state(&mut c, &node).unwrap();
        let raw = find_input_state(&session.active().unwrap().payloads, &mut c, node.index)
            .unwrap()
            .0;
        let copy_index = session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, node.index, DocumentRef::Owned(0))
            .unwrap();
        let copy = session.active().unwrap().node(&mut c, copy_index).unwrap();
        let clone_state = session.input_state(&mut c, &copy).unwrap();
        let copy_raw = find_input_state(&session.active().unwrap().payloads, &mut c, copy.index)
            .unwrap()
            .0;
        let independent = {
            let storage = session.active().unwrap();
            let RetainedPayload::String { value: a, .. } = &storage.payloads.values[raw] else {
                unreachable!()
            };
            let RetainedPayload::String { value: b, .. } = &storage.payloads.values[copy_raw]
            else {
                unreachable!()
            };
            !a.shares_storage_with(b)
        };
        let slot = session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .position(
                |p| matches!(p, RetainedPayload::Input { node: Some(id), .. } if *id == copy.index),
            )
            .unwrap();
        session
            .active_mut()
            .unwrap()
            .reset_input(&mut c, copy.index, slot)
            .unwrap();
        let reset = session.input_state(&mut c, &copy).unwrap();
        let mut output = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &copy, &mut output)
            .unwrap();
        let current = output.units().to_vec();
        output.clear(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &copy, &mut output)
            .unwrap();
        let default = output.units().to_vec();
        output.release(&mut c).unwrap();
        assert!(
            session
                .input_writeback_sources(&mut c, &node)
                .unwrap()
                .is_none()
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert!(independent);
        assert_eq!(before, clone_state);
        assert_eq!(current, expected.encode_utf16().collect::<Vec<_>>());
        assert_eq!(default, value.encode_utf16().collect::<Vec<_>>());
        assert_eq!(
            reset.flags.input_type(),
            surgeist_html::attributes::HtmlInputType::Number
        );
        assert!(reset.flags.checkedness());
        assert!(!reset.flags.dirty_value() && !reset.flags.dirty_checkedness());
        assert!(!reset.flags.input_type().is_selectable());
    }
}

#[test]
fn number_input_real_reset_cancellation_retains_prefix_before_association() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head></head><input type=number value=1e999>"),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    // Existing test scheduling hook cancels the supplied request token only
    // after the actual creation reset, before the next normal association read.
    session.active_mut().unwrap().cancel_after_input_reset = Some(request.cancellation().clone());
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
    assert_eq!(
        result.err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert!(input.is_none());
    let (actual, attached, associated) = {
        let storage = session.active().unwrap();
        let (node, raw) = storage
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
                } if state.flags.input_type()
                    == surgeist_html::attributes::HtmlInputType::Number =>
                {
                    Some((*node, *raw))
                }
                _ => None,
            })
            .unwrap();
        let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
            unreachable!()
        };
        let associated = storage
            .payloads
            .values
            .iter()
            .find_map(|p| match p {
                RetainedPayload::FormAssociation {
                    node: Some(id),
                    state,
                } if *id == node => Some(state.parser_inserted),
                _ => None,
            })
            .unwrap();
        (
            value.code_units().to_vec(),
            storage
                .construction
                .node_at(node)
                .expect("retained construction position")
                .parent()
                .map(|node| node.position())
                .is_some(),
            associated,
        )
    };
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(actual, [49, 101, 57, 57, 57]);
    assert!(!attached && !associated);
}

#[test]
fn number_input_alias_foreign_and_pre_cancelled_rejections_preserve_original_tag() {
    for case in 0..3 {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=number value=1e999>"),
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
fn range_input_exact_work_cuts_keep_real_scratch_and_first_stop_cleanup_owned() {
    for (source, expected) in [
        (
            "<html><head></head><input type=range value=50 step=any>",
            &[53, 48][..],
        ),
        (
            "<html><head></head><input type=range step=any value=.1>",
            &[46, 49][..],
        ),
        (
            "<html><head></head><input type=range step=any value=0.1234567890123456789>",
            &[
                48, 46, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
            ][..],
        ),
    ] {
        let selected = limits()
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), selected);
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
        let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let state = session.input_state(&mut c, &node).unwrap();
        let mut output = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &node, &mut output)
            .unwrap();
        let actual = output.units().to_vec();
        output.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(actual, expected);
        assert_eq!(
            state.flags.input_type(),
            surgeist_html::attributes::HtmlInputType::Range
        );
        assert!(!state.flags.input_type().is_selectable());
        assert!(!state.flags.dirty_value() && !state.flags.dirty_checkedness());
        for tail in [0, 1, 1024, 65536, total / 2] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(source),
                selected
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
            let at_stop = c.first_stop_usage().unwrap().clone();
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.usage().actual_capacity_bytes, 0);
            assert_eq!(c.stop(), Some(first));
            assert_eq!(c.first_stop_usage(), Some(&at_stop));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                work
            );
        }
    }
}

#[test]
fn range_input_precancel_alias_and_foreign_refusal_preserve_original_lexical_fields() {
    for case in 0..3 {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=range value=50>"),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, range) =
            input_owner(&request);
        let records = session.active().unwrap().construction.test_storage().len();
        let payloads = session.active().unwrap().payloads.values.len();
        let alias = (case == 0).then(|| {
            input.as_ref().unwrap().attributes.as_slice()[1]
                .value
                .value
                .text()
                .clone()
        });
        let mut other = None;
        let target = if case == 1 {
            let foreign =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = foreign.root(&mut c).unwrap();
            other = Some(foreign);
            root
        } else {
            parent
        };
        if case == 2 {
            request.cancellation().cancel();
        }
        let result = session.allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &target,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &capability,
        );
        assert_eq!(
            result.err(),
            Some(match case {
                0 => SessionError::AliasedPayload,
                1 => SessionError::ForeignNode,
                _ => SessionError::Stop(HtmlParseStop::Cancelled),
            })
        );
        assert!(input.is_some());
        assert_eq!(
            session.active().unwrap().construction.test_storage().len(),
            records
        );
        assert_eq!(session.active().unwrap().payloads.values.len(), payloads);
        drop(alias);
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        if let Some(mut other) = other {
            other.release(&mut c).unwrap();
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn range_generated_history_lenders_keep_role_qualified_inputs_without_digit_origins() {
    use provenance::{HtmlParseRangeBranch, HtmlParseRangeOperandKind, HtmlParseRangeOperandRole};
    let source =
        crate::HtmlSource::utf8("<!doctype html><input type=range min=0 max=100 step=20 value=50>");
    let source_id = source.identity().clone();
    let report = crate::HtmlDocumentParseRequest::offline(
        source,
        limits()
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap(),
    )
    .parse()
    .unwrap();
    assert_eq!(report.source().identity(), &source_id);
    let rows = report.associations();
    let history = report.history_view();
    let mut step = None;
    for index in 0..history.len() {
        let group = history.group(index).unwrap();
        let header = history.header(group).unwrap();
        if let Some(computation) = header.cause.range_computation() {
            assert!(matches!(
                header.body,
                history::HtmlParseHistoryBody::Leaf(_)
            ));
            if computation.branch() == HtmlParseRangeBranch::StepCorrection {
                step = Some((group, computation));
            }
        }
    }
    let (group, computation) = step.expect("literal 50 with min0 step20 selects60");
    assert!(!computation.original_is_fresh_empty());
    assert!(computation.original_group().unwrap().is_some());
    assert!(computation.original_cause().unwrap().is_some());
    for role in [
        HtmlParseRangeOperandRole::Minimum,
        HtmlParseRangeOperandRole::Maximum,
        HtmlParseRangeOperandRole::Step,
    ] {
        let operand = computation.operand(role);
        assert_eq!(operand.kind(), HtmlParseRangeOperandKind::Finite);
        assert!(operand.group().unwrap().is_some() && operand.cause().unwrap().is_some());
        assert!(
            (0..rows.len()).any(|i| rows.row(i).unwrap().unwrap().relation()
                == provenance::HtmlParseAssociationRelation::RangeDependency(role))
        );
    }
    assert_eq!(
        computation
            .operand(HtmlParseRangeOperandRole::ValueContent)
            .kind(),
        HtmlParseRangeOperandKind::NotConsulted
    );
    let mut scratch = [history::HtmlParseHistoryFrame::default(); 32];
    let mut cursor = history.cursor(group, &mut scratch, None).unwrap();
    let first = cursor.advance();
    assert!(
        matches!(first, history::HtmlParseHistoryStep::Leaf(atom) if atom.origins().is_empty() && atom.units()==[54,48])
    );
    assert!(matches!(
        cursor.advance(),
        history::HtmlParseHistoryStep::Complete
    ));
}

#[test]
fn range_later_minimum_resanitization_retains_generated_original_and_constraint_history() {
    use provenance::{HtmlParseRangeBranch, HtmlParseRangeOperandKind, HtmlParseRangeOperandRole};
    // type first produces50; bad again defaults to50. Installing min75
    // then clamps that real generated current to75. Later max60 disables
    // upper correction because it is below min, and any disables steps.
    let report = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<!doctype html><input type=range value=bad min=75 max=60 step=any>",
        ),
        limits()
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap(),
    )
    .parse()
    .unwrap();
    let history = report.history_view();
    let mut underflow = None;
    for index in 0..history.len() {
        let group = history.group(index).unwrap();
        let header = history.header(group).unwrap();
        if let Some(computation) = header.cause.range_computation()
            && computation.branch() == HtmlParseRangeBranch::Underflow
        {
            underflow = Some((group, computation));
        }
    }
    let (group, computation) = underflow.expect("later min75 clamps generated50");
    assert!(!computation.original_is_fresh_empty());
    let original = computation.original_group().unwrap().unwrap();
    let original_cause = computation.original_cause().unwrap().unwrap();
    assert_eq!(
        original_cause.range_computation().unwrap().branch(),
        HtmlParseRangeBranch::InvalidDefaultMidpoint
    );
    let minimum = computation.operand(HtmlParseRangeOperandRole::Minimum);
    assert_eq!(minimum.kind(), HtmlParseRangeOperandKind::Finite);
    assert!(minimum.cause().unwrap().unwrap().source().is_some());
    assert_eq!(
        computation
            .operand(HtmlParseRangeOperandRole::Maximum)
            .kind(),
        HtmlParseRangeOperandKind::Absent
    );
    let mut frames = [history::HtmlParseHistoryFrame::default(); 32];
    for (target, expected, origin_free) in [
        (original, &[53, 48][..], true),
        (minimum.group().unwrap().unwrap(), &[55, 53][..], false),
        (group, &[55, 53][..], true),
    ] {
        let mut cursor = history.cursor(target, &mut frames, None).unwrap();
        let history::HtmlParseHistoryStep::Leaf(atom) = cursor.advance() else {
            panic!("actual retained leaf")
        };
        assert_eq!(atom.units(), expected);
        assert_eq!(atom.origins().is_empty(), origin_free);
        assert!(matches!(
            cursor.advance(),
            history::HtmlParseHistoryStep::Complete
        ));
    }
    let mut minimum_cursor = history
        .cursor(minimum.group().unwrap().unwrap(), &mut frames, None)
        .unwrap();
    let history::HtmlParseHistoryStep::Leaf(minimum_atom) = minimum_cursor.advance() else {
        panic!("actual minimum lexical leaf")
    };
    let minimum_source = &minimum_atom.origins()[0].source;
    let associations = report.associations();
    let mut reverse = associations.reverse(minimum_source, None).unwrap();
    let mut minimum_match = false;
    let mut complete = false;
    for _ in 0..=associations.len() {
        match reverse.advance() {
            read::HtmlParseReverseStep::Fault(fault) => panic!("{fault}"),
            read::HtmlParseReverseStep::Match(row) => {
                if row.relation()
                    == provenance::HtmlParseAssociationRelation::RangeDependency(
                        HtmlParseRangeOperandRole::Minimum,
                    )
                {
                    let source = row.source().unwrap();
                    assert_eq!(source.source(), minimum_source.source());
                    assert!(source.start() < minimum_source.end());
                    assert!(minimum_source.start() < source.end());
                    assert_eq!((row.start(), row.end()), (0, 2));
                    minimum_match = true;
                }
            }
            read::HtmlParseReverseStep::Skipped => {}
            read::HtmlParseReverseStep::Complete => {
                complete = true;
                break;
            }
            read::HtmlParseReverseStep::Cancelled => panic!("no cancellation supplied"),
        }
    }
    assert!(complete && minimum_match);
}

#[test]
fn range_later_maximum_resanitization_keeps_lexical_constraint_and_generated_history() {
    use provenance::{HtmlParseRangeBranch, HtmlParseRangeOperandKind, HtmlParseRangeOperandRole};
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<!doctype html><input type=range value=75 max=60 step=any>"),
        limits()
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap(),
    );
    let maximum_source = request.source().range(46, 48).unwrap();
    let report = request.parse().unwrap();
    let history = report.history_view();
    let mut correction = None;
    for index in 0..history.len() {
        let group = history.group(index).unwrap();
        if let Some(computation) = history.header(group).unwrap().cause.range_computation()
            && computation.branch() == HtmlParseRangeBranch::Overflow
        {
            correction = Some((group, computation));
        }
    }
    let (group, computation) = correction.expect("later max60 clamps actual75");
    let maximum = computation.operand(HtmlParseRangeOperandRole::Maximum);
    assert_eq!(maximum.kind(), HtmlParseRangeOperandKind::Finite);
    let mut frames = [history::HtmlParseHistoryFrame::default(); 32];
    for (target, expected, origin_free) in [
        // Valid authored75 was preserved, so this original is lexical.
        (
            computation.original_group().unwrap().unwrap(),
            &[55, 53][..],
            false,
        ),
        (maximum.group().unwrap().unwrap(), &[54, 48][..], false),
        (group, &[54, 48][..], true),
    ] {
        let mut cursor = history.cursor(target, &mut frames, None).unwrap();
        let history::HtmlParseHistoryStep::Leaf(atom) = cursor.advance() else {
            panic!("actual retained leaf")
        };
        assert_eq!(atom.units(), expected);
        assert_eq!(atom.origins().is_empty(), origin_free);
        assert!(matches!(
            cursor.advance(),
            history::HtmlParseHistoryStep::Complete
        ));
    }
    let associations = report.associations();
    let mut reverse = associations.reverse(&maximum_source, None).unwrap();
    let mut matched = false;
    let mut complete = false;
    for _ in 0..=associations.len() {
        match reverse.advance() {
            read::HtmlParseReverseStep::Fault(fault) => panic!("{fault}"),
            read::HtmlParseReverseStep::Match(row)
                if row.relation()
                    == provenance::HtmlParseAssociationRelation::RangeDependency(
                        HtmlParseRangeOperandRole::Maximum,
                    ) =>
            {
                assert_eq!((row.start(), row.end()), (0, 2));
                matched = true;
            }
            read::HtmlParseReverseStep::Match(_) | read::HtmlParseReverseStep::Skipped => {}
            read::HtmlParseReverseStep::Complete => {
                complete = true;
                break;
            }
            read::HtmlParseReverseStep::Cancelled => panic!("no cancellation supplied"),
        }
    }
    assert!(complete && matched);
}

#[test]
fn range_actual_first_scratch_append_cancellation_retains_prefix_and_all_tickets() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head></head><input type=range value=50>"),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    // Deterministic scheduling instrumentation cancels only the supplied
    // request handle after the real first admitted decimal cell append.
    payload_write::<
        Option<(
            Option<surgeist_html::bounded::numeric::HtmlRangeScratchRole>,
            crate::HtmlParseCancellation,
        )>,
    >(&mut c, 1)
    .unwrap();
    c.charge(HtmlParseWorkCategory::ProviderCall, 1).unwrap();
    session.active_mut().unwrap().cancel_after_range_append =
        Some((None, request.cancellation().clone()));
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
    assert_eq!(
        result.err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert!(input.is_none());
    assert!(
        session
            .active()
            .unwrap()
            .effects
            .range
            .as_ref()
            .unwrap()
            .owned_cells()
            > 0
    );
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let at_stop = c.first_stop_usage().unwrap().clone();
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(c.first_stop_usage(), Some(&at_stop));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
}

#[test]
fn range_small_decimal_committed_cell_and_power_cancellation_keep_real_owners() {
    use range::{RangeIngressCancellation, RangeIngressCancellationPoint};
    use surgeist_html::bounded::numeric::HtmlRangeScratchRole;
    for (point, label, remaining_replies) in [
        (
            RangeIngressCancellationPoint::CoefficientFold,
            "coefficient fold",
            8,
        ),
        (
            RangeIngressCancellationPoint::DecimalPower,
            "decimal power",
            6,
        ),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=range step=any value=.1>"),
            limits()
                .into_builder()
                .work_credits(100_000_000)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, source_range) =
            input_owner(&request);
        payload_write::<Option<RangeIngressCancellation>>(&mut c, 1).unwrap();
        c.charge(HtmlParseWorkCategory::ProviderCall, 1).unwrap();
        session.active_mut().unwrap().cancel_after_range_ingress = Some(RangeIngressCancellation {
            point,
            scalars_after_fold: 0,
            token: request.cancellation().clone(),
        });
        let result = session.allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &source_range,
            &capability,
        );
        assert_eq!(
            result.err(),
            Some(SessionError::Stop(HtmlParseStop::Cancelled))
        );
        assert!(input.is_none());
        // Terminal fixture inspection of the actual retained producer, not
        // a manufactured cursor/phase or a public test observation API.
        let storage = session.active().unwrap();
        assert!(storage.cancel_after_range_ingress.is_none());
        let (source, coefficient, right, trial, cursor) = storage
            .effects
            .range
            .as_ref()
            .unwrap()
            .ingress_fixture_state();
        assert_eq!(coefficient, [1]);
        assert_eq!((right, trial), (0, 0));
        let RetainedPayload::String { value, .. } = &storage.payloads.values[source] else {
            panic!("actual source backing");
        };
        assert_eq!(value.code_units(), [46, 49]);
        let cursor = cursor.unwrap();
        assert!(cursor.result().is_none());
        let headers = cursor.headers();
        assert_eq!(
            headers.buffer(HtmlRangeScratchRole::Coefficient).len(),
            coefficient.len()
        );
        // The cancellation hook runs after the cursor committed the reply it
        // was servicing, so the pending request is the one queued next. The
        // small-ratio protocol queues ScalarStep after every committed
        // reply except the coefficient read (Read -> Next -> Power ...).
        assert_eq!(
            cursor.request(),
            Some(surgeist_html::bounded::numeric::HtmlRangeNumericRequest::ScalarStep),
            "{label}: the next pending request"
        );
        // The committed numeric state is observed through the cursor's own
        // protocol: resuming a copy of the cancelled cursor with a faithful
        // servicer over the retained source and coefficient cell must finish
        // with the value of the literal .1. The oracle is Rust's own
        // binary64 nearest-value conversion of that literal, so a wrong
        // committed numerator, denominator or power would change the result.
        // The reply count tells the two stops apart, since both resume to
        // the same value. After the fold (A = 1) the small ratio takes one
        // reply each for Next, Power (commits B = 10), Power (the power
        // count reaches zero), Exponent, Scale, Quotient, Round and Finish:
        // eight from the fold point. The power stop follows the second of
        // those replies (the one committing B = 10), leaving six.
        let (resumed, replies) = resume_range_ingress(*cursor, &[46, 49], coefficient);
        assert_eq!(resumed, Some(Ok(0.1_f64.to_bits())), "{label}");
        assert_eq!(replies, remaining_replies, "{label}: replies left");
        let raw = storage
            .payloads
            .values
            .iter()
            .find_map(|p| match p {
                RetainedPayload::Input { raw, .. } => Some(*raw),
                _ => None,
            })
            .unwrap();
        let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
            panic!("real previous current owner");
        };
        assert_eq!(value.code_units(), [53, 48]);
        let first = c.first_stop_usage().unwrap().clone();
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        assert_eq!(
            session.input_state(&mut c, &parent).err(),
            Some(SessionError::Stop(HtmlParseStop::Cancelled))
        );
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(HtmlParseStop::Cancelled))
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
        assert_eq!(c.first_stop_usage(), Some(&first));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
    }
}

#[test]
fn range_right_cache_append_cancellation_keeps_actual_partial_lease_owned() {
    use surgeist_html::bounded::numeric::HtmlRangeScratchRole;
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=range step=any value=0.10000000000000000000>",
        ),
        // Twenty actual coefficient cells take the unchanged exact search,
        // not SOURCE7s small-decimal route. The ordinary budget admits default50
        // serialization; the smaller owner fixture stops in that Best phase.
        limits()
            .into_builder()
            .work_credits(100_000_000)
            .finish()
            .unwrap(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    // Scheduling only: the original supplied token is cancelled after the
    // actual Right append has committed. Earlier roles use the normal path.
    payload_write::<Option<(Option<HtmlRangeScratchRole>, crate::HtmlParseCancellation)>>(
        &mut c, 1,
    )
    .unwrap();
    c.charge(HtmlParseWorkCategory::ProviderCall, 1).unwrap();
    session.active_mut().unwrap().cancel_after_range_append = Some((
        Some(HtmlRangeScratchRole::Right),
        request.cancellation().clone(),
    ));
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
    assert_eq!(
        result.err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert!(input.is_none());
    let (len, capacity) = session
        .active()
        .unwrap()
        .effects
        .range
        .as_ref()
        .unwrap()
        .right_cache_extent();
    assert!(len > 0 && capacity >= len);
    let first = c.first_stop_usage().unwrap().clone();
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    assert_eq!(
        session.input_state(&mut c, &parent).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(c.first_stop_usage(), Some(&first));
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(c.first_stop_usage(), Some(&first));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
}

#[test]
fn range_input_clone_and_reset_preserve_generated_value_original_default_and_distinct_backing() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head></head><input type=range value=bad checked>"),
        limits()
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap(),
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
    let before = session.input_state(&mut c, &node).unwrap();
    let original_raw = find_input_state(&session.active().unwrap().payloads, &mut c, node.index)
        .unwrap()
        .0;
    let copy_index = session
        .active_mut()
        .unwrap()
        .clone_node(&mut c, node.index, DocumentRef::Owned(0))
        .unwrap();
    let copy = session.active().unwrap().node(&mut c, copy_index).unwrap();
    let copied = session.input_state(&mut c, &copy).unwrap();
    let copy_raw = find_input_state(&session.active().unwrap().payloads, &mut c, copy.index)
        .unwrap()
        .0;
    let independent = {
        let storage = session.active().unwrap();
        let RetainedPayload::String { value: a, .. } = &storage.payloads.values[original_raw]
        else {
            unreachable!()
        };
        let RetainedPayload::String { value: b, .. } = &storage.payloads.values[copy_raw] else {
            unreachable!()
        };
        !a.shares_storage_with(b)
    };
    let slot = session
        .active()
        .unwrap()
        .payloads
        .values
        .iter()
        .position(|p| matches!(p,RetainedPayload::Input {node:Some(id),..} if *id==copy.index))
        .unwrap();
    session
        .active_mut()
        .unwrap()
        .reset_input(&mut c, copy.index, slot)
        .unwrap();
    let reset = session.input_state(&mut c, &copy).unwrap();
    let mut output = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_internal_value(&mut c, &copy, &mut output)
        .unwrap();
    let current = output.units().to_vec();
    output.clear(&mut c).unwrap();
    session
        .append_input_default_value(&mut c, &copy, &mut output)
        .unwrap();
    let default = output.units().to_vec();
    output.release(&mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert!(independent);
    assert_eq!(before, copied);
    assert_eq!(current, [53, 48]);
    assert_eq!(default, [98, 97, 100]);
    assert_eq!(
        reset.flags.input_type(),
        surgeist_html::attributes::HtmlInputType::Range
    );
    assert!(reset.flags.checkedness());
    assert!(!reset.flags.dirty_value() && !reset.flags.dirty_checkedness());
}

#[test]
fn range_copy_requires_actual_generated_group_and_rejects_a_foreign_controller() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head></head><input type=range value=bad>"),
        limits()
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, source) = input_owner(&request);
    let node = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &source,
            &capability,
        )
        .unwrap();
    let raw = find_input_state(&session.active().unwrap().payloads, &mut c, node.index)
        .unwrap()
        .0;
    let copied = {
        let storage = session.active().unwrap();
        provenance::copy_generated_input_value(
            &storage.payloads,
            storage.evidence.as_ref(),
            &mut c,
            raw,
        )
        .unwrap()
    };
    assert_eq!(copied.value.text().code_units(), [53, 48]);
    assert!(copied.origins.as_slice().is_empty());
    assert!(copied.original.is_some());
    copied.release(&mut c).unwrap();

    let mut foreign = control();
    let before = c.usage().clone();
    let refused = {
        let storage = session.active().unwrap();
        provenance::copy_generated_input_value(
            &storage.payloads,
            storage.evidence.as_ref(),
            &mut foreign,
            raw,
        )
    };
    assert!(matches!(
        refused,
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert_eq!(c.usage(), before);
    storage_zero(&foreign);

    // A real source-less implied literal is not a Range computation. Its
    // empty lexical map cannot authorize an origin-free generated copy.
    let body = session.allocate_implicit_body(&mut c, None).unwrap();
    let literal = session.active().unwrap().payloads.values.iter().position(|value|
        matches!(value, RetainedPayload::String { location: StringLocation::ElementName(id), .. } if *id == body.index)).unwrap();
    let refused = {
        let storage = session.active().unwrap();
        provenance::copy_generated_input_value(
            &storage.payloads,
            storage.evidence.as_ref(),
            &mut c,
            literal,
        )
    };
    assert!(matches!(
        refused,
        Err(SessionError::WrongToken | SessionError::OriginSplitUnavailable)
    ));
    assert!(c.stop().is_none());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn range_absent_default_to_image_creates_actual_value_attribute_before_default_mode() {
    // HTML877d #input-type-change: Value -> Default with nonempty current
    // sets the content value to current. Range absent value has midpoint50;
    // Image uses Default mode. No mutable host or fabricated lexical input.
    let literal = "<html><head></head><input type=range><i type=image>";
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(literal),
        limits()
            .into_builder()
            .work_credits(100_000_000)
            .finish()
            .unwrap(),
    );
    let (mut c, mut session, capability, mut lexer, parent, mut input, source) =
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
            &source,
            &capability,
        )
        .unwrap();
    assert!(input.is_none());
    assert!(
        session
            .active()
            .unwrap()
            .form_attribute(&mut c, node.index, "value")
            .unwrap()
            .is_none()
    );
    let mut value = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_internal_value(&mut c, &node, &mut value)
        .unwrap();
    assert_eq!(value.units(), [53, 48]);
    value.clear(&mut c).unwrap();
    let mut attribute = radio_next_attribute(&mut lexer, &mut c);
    let changed = session.set_input_attribute(&mut c, &node, &mut attribute, &capability);
    let observed = if changed.is_ok() {
        assert!(attribute.is_none());
        let state = session.input_state(&mut c, &node).unwrap();
        let native = session
            .active()
            .unwrap()
            .form_attribute(&mut c, node.index, "value")
            .unwrap()
            .unwrap()
            .code_units()
            .to_vec();
        session
            .append_input_internal_value(&mut c, &node, &mut value)
            .unwrap();
        let current = value.units().to_vec();
        value.clear(&mut c).unwrap();
        session
            .append_input_default_value(&mut c, &node, &mut value)
            .unwrap();
        Some((
            state.flags.input_type(),
            native,
            current,
            value.units().to_vec(),
        ))
    } else {
        None
    };
    let first = c.stop();
    if let Some(attribute) = attribute.take() {
        attribute.name.release(&mut c).unwrap();
        attribute.value.release(&mut c).unwrap();
    }
    value.release(&mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
    assert_eq!(c.stop(), first);
    assert!(
        changed.is_ok(),
        "required Range50 -> Image content writeback failed after actual cleanup: {changed:?}"
    );
    assert_eq!(
        observed,
        Some((
            surgeist_html::attributes::HtmlInputType::Image,
            vec![53, 48],
            vec![53, 48],
            vec![53, 48]
        ))
    );
}

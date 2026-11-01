use super::tests::{control, limits, native_text, owned_text, storage_zero};
use super::*;
use crate::HtmlParseCancellation;

fn aggregate(c: &mut ParseControl) -> (ParserArenaSession, ParserNode, ParserNode) {
    let mut session = ParserArenaSession::new(c, RootKind::Fragment).unwrap();
    let root = session.root(c).unwrap();
    let mut node = None;
    for unit in [65, 66] {
        let mut input = Some(owned_text(c, &[unit]));
        node = Some(
            session
                .insert_text(c, &root, None, &mut input, None)
                .unwrap()
                .unwrap()
                ._node,
        );
    }
    (session, root, node.unwrap())
}

fn exact_current_ticket(session: &ParserArenaSession, c: &mut ParseControl, node: &ParserNode) {
    let storage = session.active().unwrap();
    let slot = storage.payloads.node_joins[node.index].text().unwrap();
    let RetainedPayload::String {
        value,
        allocation,
        location: StringLocation::Text(owner),
        ..
    } = &storage.payloads.values[slot]
    else {
        panic!("actual indexed current ticket")
    };
    assert_eq!(*owner, node.index);
    let HtmlNodeKind::Text(native) = storage.construction.node_at(node.index).unwrap().kind()
    else {
        panic!("native text")
    };
    assert!(Arc::ptr_eq(
        value.shared_code_units(),
        native.shared_code_units()
    ));
    assert_eq!(allocation.units(), value.len() as u64);
    assert_eq!(
        allocation.vector_capacity(),
        value.shared_code_units().capacity()
    );
    if c.stop().is_some() {
        c.require_document_string_for_disposal(allocation, value)
            .unwrap();
    }
}

/// Successful measurements bound finite cut coverage only. Literal AB + the
/// incoming units and actual ticket/native identity are the independent oracle.
#[test]
fn every_append_work_cut_preserves_original_or_commits_exact_input_and_reclaims_storage() {
    fn run(cap: Option<u64>, shared: bool) -> (u64, u64, bool) {
        let selected = limits()
            .into_builder()
            .work_credits(cap.unwrap_or(10_000_000))
            .finish()
            .unwrap();
        let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
        let (mut session, root, node) = if shared {
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let root = session.root(&mut c).unwrap();
            let mut input = Some(owned_text(&mut c, &[65, 66]));
            let node = session
                .insert_text(&mut c, &root, None, &mut input, None)
                .unwrap()
                .unwrap()
                ._node;
            (session, root, node)
        } else {
            aggregate(&mut c)
        };
        let incoming = [0xd800, 0, 0xdc00].repeat(100);
        let mut input = Some(owned_text(&mut c, &incoming));
        let start = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let result = session.insert_text(&mut c, &root, None, &mut input, None);
        let end = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let mut rolled_back_tail = false;
        match result {
            Ok(Some(inserted)) => {
                assert!(input.is_none());
                assert_eq!(inserted._node.index, node.index);
                assert_eq!(inserted._offset, 2);
                let mut expected = vec![65, 66];
                expected.extend_from_slice(&incoming);
                assert_eq!(native_text(&session, &node), expected);
                assert_eq!(
                    session.active().unwrap().construction.usage().utf16_units(),
                    302
                );
            }
            Err(SessionError::Stop(
                stop @ ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                },
            )) => {
                assert_eq!(native_text(&session, &node), [65, 66]);
                assert_eq!(
                    session.active().unwrap().construction.usage().utf16_units(),
                    2
                );
                let first = c.first_stop_usage().unwrap().clone();
                rolled_back_tail = first.resource(HtmlParseResource::ManagedUtf16Units).current
                    > c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current;
                let work = c.usage().resource(HtmlParseResource::WorkCredits);
                assert_eq!(c.charge(WorkCategory::StateStep, 1), Err(stop));
                assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
                assert_eq!(c.first_stop_usage(), Some(&first));
            }
            _ => panic!("only exact success or actual work refusal"),
        }
        exact_current_ticket(&session, &mut c, &node);
        if let Some(text) = input.take() {
            text.release(&mut c).unwrap();
        }
        let mut first = c.first_stop_usage().cloned();
        if session.release(&mut c).is_err() {
            // Exact parsing work need not include ordinary disposal work.
            // The owning terminal retry must preserve its first observation.
            if first.is_none() {
                first = c.first_stop_usage().cloned();
            }
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        assert_eq!(c.first_stop_usage(), first.as_ref());
        (start, end, rolled_back_tail)
    }
    for shared in [false, true] {
        let (start, complete, _) = run(None, shared);
        assert!(complete - start <= 32_768);
        eprintln!(
            "actual controlled append finite work-cut envelope shared={shared}: {}",
            complete - start
        );
        let mut tail_cut = false;
        for cap in start..complete {
            tail_cut |= run(Some(cap), shared).2;
        }
        if !shared {
            assert!(
                tail_cut,
                "cuts reached actual initialized tail and rollback"
            );
        }
        run(Some(complete), shared); // exact; loop included exact one-short.
    }
}

#[test]
fn cancellation_after_real_tail_chunk_keeps_old_text_and_actual_growth_and_first_stop() {
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let (mut session, root, node) = aggregate(&mut c);
    session.active_mut().unwrap().cancel_after_text_append_chunk = Some(cancellation);
    let mut input = Some(owned_text(&mut c, &vec![0xd800; 300]));
    assert_eq!(
        session
            .insert_text(&mut c, &root, None, &mut input, None)
            .err(),
        Some(SessionError::Stop(ParseStop::Cancelled))
    );
    assert!(input.is_none());
    assert_eq!(native_text(&session, &node), [65, 66]);
    assert_eq!(
        session.active().unwrap().construction.usage().utf16_units(),
        2
    );
    let first = c.first_stop_usage().unwrap().clone();
    assert_eq!(
        first.resource(HtmlParseResource::ManagedUtf16Units).current,
        4 + 300 + 128
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        4 + 300
    );
    let storage = session.active().unwrap();
    let HtmlNodeKind::Text(native) = storage.construction.node_at(node.index).unwrap().kind()
    else {
        panic!("text")
    };
    assert!(
        native.shared_code_units().capacity() >= 302,
        "actual growth survives rollback"
    );
    exact_current_ticket(&session, &mut c, &node);
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.first_stop_usage(), Some(&first));
}

#[test]
fn external_weak_preservation_and_detached_append_keep_current_ticket_and_history() {
    let mut c = control();
    let (mut session, root, node) = aggregate(&mut c);
    let weak = {
        let storage = session.active().unwrap();
        let HtmlNodeKind::Text(value) = storage.construction.node_at(node.index).unwrap().kind()
        else {
            panic!("text")
        };
        Arc::downgrade(value.shared_code_units())
    };
    let mut input = Some(owned_text(&mut c, &[0, 0xd800]));
    session
        .insert_text(&mut c, &root, None, &mut input, None)
        .unwrap();
    assert!(
        weak.upgrade().is_none(),
        "old aggregate leaves only immutable provenance leaves"
    );
    exact_current_ticket(&session, &mut c, &node);
    session.remove(&mut c, &node, None).unwrap();
    exact_current_ticket(&session, &mut c, &node);
    session.insert(&mut c, &root, &node, None, None).unwrap();
    let mut input = Some(owned_text(&mut c, &[0xdc00]));
    let next = session
        .insert_text(&mut c, &root, None, &mut input, None)
        .unwrap()
        .unwrap();
    assert_eq!(next._node.index, node.index);
    assert_eq!(native_text(&session, &node), [65, 66, 0, 0xd800, 0xdc00]);
    exact_current_ticket(&session, &mut c, &node);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn increasing_whole_and_progressive_unicode_entity_feeds_have_linear_append_copy_and_lookup_work() {
    use crate::parser::tokenizer::{Tokenizer, TokenizerPull, states::State, token::Event};
    for repetitions in [8usize, 16, 32, 64, 128] {
        for utf16 in [false, true] {
            let source = if utf16 {
                crate::HtmlSource::utf16(DocumentString::from_utf16(
                    [65, 0xd800, 13, 10, 0xd83d, 0xde42].repeat(repetitions),
                ))
            } else {
                crate::HtmlSource::utf8("A&amp;🙂\r\n".repeat(repetitions))
            };
            let pattern: &[u16] = if utf16 {
                &[65, 0xd800, 10, 0xd83d, 0xde42]
            } else {
                &[65, 38, 0xd83d, 0xde42, 10]
            };
            let expected = pattern.repeat(repetitions);
            let expected_origins: Vec<_> = (0..repetitions)
                .flat_map(|n| {
                    let out = n * 5;
                    let raw = n * if utf16 { 6 } else { 12 };
                    let spans = if utf16 {
                        [(0, 1, 0, 1), (1, 2, 1, 2), (2, 3, 2, 4), (3, 5, 4, 6)]
                    } else {
                        [(0, 1, 0, 1), (1, 2, 1, 6), (2, 4, 6, 10), (4, 5, 10, 12)]
                    };
                    spans.map(|(a, b, c, d)| (out + a, out + b, raw + c, raw + d))
                })
                .collect();
            for progressive in [false, true] {
                // Bounds cover the actual lexical atoms/groups/causes and
                // registry cells for this finite source, without product changes.
                let n = source.len() as u64;
                let selected = limits()
                    .into_builder()
                    .source_units(n)
                    .auxiliary_entries(32 * n + 64)
                    .source_map_entries(32 * n + 64)
                    .managed_capacity_bytes(32_768 * n + 1024)
                    .peak_transient_bytes(65_536 * n + 2048)
                    .finish()
                    .unwrap();
                let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
                c.admit_source(&source).unwrap();
                let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
                session.initialize_parse_evidence(&mut c, &source).unwrap();
                let root = session.root(&mut c).unwrap();
                let mut lexer = if progressive {
                    Tokenizer::progressive(&source, State::Data, None, &mut c).unwrap()
                } else {
                    Tokenizer::new(&source, State::Data, None, &mut c).unwrap()
                };
                let cuts: Vec<_> = if !progressive {
                    vec![source.len()]
                } else if utf16 {
                    (1..=source.len()).collect()
                } else {
                    "A&amp;🙂\r\n"
                        .repeat(repetitions)
                        .char_indices()
                        .map(|(start, ch)| start + ch.len_utf8())
                        .collect()
                };
                let mut texts = 0u64;
                let (mut copies, mut visits, mut lookup_visits, mut relocation_bound) =
                    (0u64, 0u64, 0u64, 0usize);
                let mut node: Option<ParserNode> = None;
                let mut old_capacity = 0;
                let mut old_address = 0;
                let mut exhausted = false;
                for (step, cut) in cuts
                    .iter()
                    .copied()
                    .enumerate()
                    .chain(std::iter::once((cuts.len(), source.len())))
                {
                    if progressive {
                        if step == cuts.len() {
                            lexer.seal_input(&mut c).unwrap();
                        } else {
                            lexer.make_available(cut, &mut c).unwrap();
                        }
                    } else if step != 0 {
                        break;
                    }
                    loop {
                        let pull = lexer
                            .next(
                                &mut c,
                                false,
                                session.active_mut().unwrap().evidence.as_mut().unwrap(),
                            )
                            .unwrap();
                        match pull {
                            TokenizerPull::NeedInput => {
                                assert!(progressive && step < cuts.len());
                                break;
                            }
                            TokenizerPull::Exhausted => {
                                exhausted = true;
                                break;
                            }
                            TokenizerPull::Event(Event::Token(event)) => {
                                match event.into_token(&mut c).unwrap().kind {
                                    TokenKind::Text(text) => {
                                        let before_copy = c.usage().work(WorkCategory::CopyByte);
                                        let before_visits =
                                            c.usage().work(WorkCategory::RecordVisit);
                                        let inserted = session
                                            .insert_text(&mut c, &root, None, &mut Some(text), None)
                                            .unwrap()
                                            .unwrap();
                                        copies +=
                                            c.usage().work(WorkCategory::CopyByte) - before_copy;
                                        visits += c.usage().work(WorkCategory::RecordVisit)
                                            - before_visits;
                                        if let Some(old) = &node {
                                            assert_eq!(inserted._node.index, old.index);
                                        }
                                        node = Some(inserted._node);
                                        texts += 1;
                                        let node = node.as_ref().unwrap();
                                        let storage = session.active().unwrap();
                                        let HtmlNodeKind::Text(value) = storage
                                            .construction
                                            .node_at(node.index)
                                            .unwrap()
                                            .kind()
                                        else {
                                            panic!("text")
                                        };
                                        let capacity = value.shared_code_units().capacity();
                                        let address = value.code_units().as_ptr() as usize;
                                        if old_capacity != 0
                                            && (capacity != old_capacity || address != old_address)
                                        {
                                            relocation_bound += 2 * old_capacity;
                                        }
                                        old_capacity = capacity;
                                        old_address = address;
                                        let before = c.usage().work(WorkCategory::RecordVisit);
                                        storage
                                            .field_slot(&mut c, StringLocation::Text(node.index))
                                            .unwrap();
                                        lookup_visits +=
                                            c.usage().work(WorkCategory::RecordVisit) - before;
                                    }
                                    TokenKind::Eof => {}
                                    _ => panic!("text-only source"),
                                }
                            }
                            TokenizerPull::Event(event) => event.release(&mut c).unwrap(),
                        }
                    }
                    if exhausted {
                        break;
                    }
                }
                assert!(exhausted);
                let node = node.unwrap();
                assert_eq!(native_text(&session, &node), expected);
                assert_eq!(
                    lookup_visits,
                    3 * texts,
                    "index slot + actual row + ticket header"
                );
                // Fixed protocol/header copies are bounded per actual Text.
                // UTF-16 incoming/preserved-leaf bytes add a linear term.
                assert!(
                    copies <= 4096 * texts + 2 * expected.len() as u64,
                    "copy={copies}, texts={texts}"
                );
                assert!(visits <= 256 * texts + 64, "visits={visits}, texts={texts}");
                assert!(old_capacity < 2 * expected.len());
                assert!(
                    relocation_bound < 4 * old_capacity,
                    "geometric actual capacity relocation bound"
                );
                let storage = session.active().unwrap();
                let slot = storage.payloads.node_joins[node.index].text().unwrap();
                let RetainedPayload::String {
                    origins: Some(group),
                    ..
                } = &storage.payloads.values[slot]
                else {
                    panic!("current exact group")
                };
                let evidence = storage.evidence.as_ref().unwrap();
                let origins = evidence.observed_origins(group, &mut c).unwrap();
                assert_eq!(
                    origins
                        .iter()
                        .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                        .collect::<Vec<_>>(),
                    expected_origins
                );
                let historical = evidence
                    .materialize_group(&mut c, &storage.payloads, group)
                    .unwrap();
                assert_eq!(historical.text().code_units(), expected);
                ParseEvidence::release_materialized_group(&mut c, &mut Some(historical)).unwrap();
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    2 * expected.len() as u64
                );
                eprintln!(
                    "append-feed reps={repetitions} utf16={utf16} progressive={progressive} texts={texts} copied={copies} visits={visits} lookup={lookup_visits} capacity={old_capacity} relocation_bound={relocation_bound}"
                );
                lexer.release(&mut c).unwrap();
                session.release(&mut c).unwrap();
                storage_zero(&c);
            }
        }
    }
}

#[test]
fn text_append_preserving_copy_has_exact_managed_boundary_and_retains_failed_buffer() {
    for cap in [3, 4] {
        let selected = limits()
            .into_builder()
            .managed_utf16_units(cap)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut first = Some(owned_text(&mut c, &[65]));
        let node = session
            .insert_text(&mut c, &root, None, &mut first, None)
            .unwrap()
            .unwrap()
            ._node;
        let mut next = Some(owned_text(&mut c, &[66]));
        let result = session.insert_text(&mut c, &root, None, &mut next, None);
        assert!(next.is_none());
        if cap == 4 {
            assert_eq!(result.unwrap().unwrap()._node.index, node.index);
            assert_eq!(native_text(&session, &node), [65, 66]);
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                4
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                0
            );
        } else {
            assert_eq!(
                result.err(),
                Some(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedUtf16Units,
                    limit: 3,
                    current: 2,
                    requested_delta: 2,
                }))
            );
            assert_eq!(native_text(&session, &node), [65]);
            // Both immutable leaves remain managed, and the actual failed AB
            // buffer stays Buffered until explicit terminal owner cleanup.
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                2
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                2
            );
            assert!(session.active().unwrap().failed_text_append.is_some());
        }
        exact_current_ticket(&session, &mut c, &node);
        let first = c.first_stop_usage().cloned();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.first_stop_usage(), first.as_ref());
    }
}

// Characterization of existing behaviour after B07 (a malformed reserve
// request from the producer is a typed stop, not a panic); not RED against an
// earlier commit. The capacity is real, observed from an actual append
// operation; only the requested size is made impossible.
#[test]
fn characterization_a_reserve_below_the_initialized_length_is_a_capacity_mismatch() {
    use crate::bounded::HtmlConstructionTextAppend as Append;
    let mut c = control();
    let (mut session, _root, node) = aggregate(&mut c);
    let storage = session.active_mut().unwrap();
    let slot = storage.payloads.node_joins[node.index].text().unwrap();
    let RetainedPayload::String {
        value, allocation, ..
    } = mem::replace(&mut storage.payloads.values[slot], RetainedPayload::Retired)
    else {
        panic!("registered text")
    };
    let mut allocation = allocation;
    c.suspend_text_append_ticket(&mut allocation, &value)
        .unwrap();
    // Without the registry's strong handle the original is exclusive, as it is
    // when the real append starts.
    drop(value);
    let mut operation = Some(Append::begin(
        storage.construction.owner.take().unwrap(),
        node.handle.clone(),
        DocumentString::from("C"),
    ));
    let before = {
        let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
        let observation = action.perform(None).unwrap();
        let before = observation.capacity().unwrap();
        observation.accept();
        before
    };
    assert_eq!(before.initialized_len(), 2);
    let stop = c
        .prepare_text_append_reserve(&allocation, before, before.initialized_len() - 1)
        .unwrap_err();
    let mismatch =
        ParseStop::InternalInvariant(crate::parser::control::Invariant::CapacityMismatch);
    assert_eq!(stop, mismatch);
    assert_eq!(c.stop(), Some(mismatch));
    let (owner, ..) = operation.take().unwrap().abort().into_parts();
    storage.construction.owner = Some(owner);
    let value = match storage.construction.node_at(node.index).unwrap().kind() {
        HtmlNodeKind::Text(value) => value.clone(),
        _ => panic!("native text"),
    };
    storage.payloads.values[slot] = RetainedPayload::String {
        value,
        allocation,
        origins: None,
        location: StringLocation::Text(node.index),
    };
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

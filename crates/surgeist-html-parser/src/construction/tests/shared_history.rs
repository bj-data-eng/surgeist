use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlDocumentMode},
    values::DocumentString,
};

#[test]
fn shared_history_reconstructs_every_literal_prefix_without_retired_prefix_backings() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("abcdefgh");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    session.initialize_parse_evidence(&mut c, &source).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut histories = Vec::new();
    let mut node = None;
    for length in 1..=8 {
        let mut input = Some(match lexical_kind(&mut lexer, &mut c) {
            TokenKind::Text(text) => text,
            _ => panic!("literal text"),
        });
        let inserted = session
            .insert_text(&mut c, &root, None, &mut input, None)
            .unwrap()
            .unwrap();
        assert_eq!(inserted._offset, length - 1);
        if let Some(old) = &node {
            assert!(session.same_node(&mut c, old, &inserted._node).unwrap());
        }
        node = Some(inserted._node);
        let storage = session.active().unwrap();
        let group = storage
            .payloads
            .detail_origins
            .peek(FieldRef::Text(node.as_ref().unwrap().index))
            .expect("current actual group");
        histories.push(
            storage
                .evidence
                .as_ref()
                .unwrap()
                .copy_group_reference(&mut c, group)
                .unwrap(),
        );
    }
    // Exactly the eight units of the text: the tokens it absorbed are ranges of it, and the
    // retired aggregates carry fixed history descriptors, not Vecs.
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        8
    );
    assert_eq!(
        session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .filter(|p| matches!(p, RetainedPayload::HistoricalString { .. }))
            .count(),
        0
    );
    let before = c
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .current;
    for (index, group) in histories.iter().enumerate() {
        let storage = session.active().unwrap();
        let evidence = storage.evidence.as_ref().unwrap();
        let reconstructed = evidence
            .materialize_group(&mut c, storage.leaf_strings(), group)
            .unwrap();
        assert_eq!(
            reconstructed.text().code_units(),
            &[97, 98, 99, 100, 101, 102, 103, 104][..index + 1]
        );
        ParseEvidence::release_materialized_group(&mut c, &mut Some(reconstructed)).unwrap();
        let maps = evidence.observed_origins(group, &mut c).unwrap();
        assert_eq!(
            maps.iter()
                .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                .collect::<Vec<_>>(),
            (0..index + 1)
                .map(|i| (i, i + 1, i, i + 1))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            before
        );
    }
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
fn two_leaf_history(
    limits: crate::HtmlParseLimits,
    cancel: HtmlParseCancellation,
) -> (
    ParseControl,
    ParserArenaSession,
    OriginGroupRef,
    crate::HtmlSource,
) {
    let source = crate::HtmlSource::utf8("AB");
    let mut c = ParseControl::new(limits, cancel);
    c.admit_source(&source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    session.initialize_parse_evidence(&mut c, &source).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut lexer = crate::parser::tokenizer::tests::LexerFixture::new(
        &source,
        crate::parser::tokenizer::states::HtmlTokenizerState::Data,
        None,
        &mut c,
    )
    .unwrap();
    let mut node = None;
    for _ in 0..2 {
        let mut text = Some(match lexical_kind(&mut lexer, &mut c) {
            TokenKind::Text(text) => text,
            _ => panic!("literal"),
        });
        node = Some(
            session
                .insert_text(&mut c, &root, None, &mut text, None)
                .unwrap()
                .unwrap()
                ._node,
        );
    }
    lexer.release(&mut c).unwrap();
    let storage = session.active().unwrap();
    let group = storage
        .payloads
        .detail_origins
        .peek(FieldRef::Text(node.unwrap().index))
        .expect("group");
    let group = storage
        .evidence
        .as_ref()
        .unwrap()
        .copy_group_reference(&mut c, group)
        .unwrap();
    (c, session, group, source)
}
#[test]
fn history_cursor_cancellation_keeps_canonical_snapshot_and_reclaims_real_frames() {
    let cancel = HtmlParseCancellation::new();
    let (mut c, mut session, group, _) = two_leaf_history(limits(), cancel.clone());
    let evidence = session.active().unwrap().evidence.as_ref().unwrap();
    let before = c
        .usage()
        .resource(HtmlParseResource::AuxiliaryEntries)
        .current;
    let mut cursor = provenance::GroupOriginCursor::new(evidence, &group, &mut c).unwrap();
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        before + 1
    );
    cancel.cancel();
    assert_eq!(
        cursor.next(evidence, &mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let snapshot = c.first_stop_usage().unwrap().clone();
    let work = c.usage().resource(HtmlParseResource::WorkCredits);
    cursor.release(&mut c).unwrap();
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        before
    );
    assert_eq!(c.first_stop_usage(), Some(&snapshot));
    assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
    assert!(matches!(
        evidence.materialize_group(&mut c, session.active().unwrap().leaf_strings(), &group),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.first_stop_usage(), Some(&snapshot));
}
#[test]
fn history_cursor_foreign_control_or_evidence_cannot_advance_the_owning_cursor() {
    let (mut c, mut session, group, _) = two_leaf_history(limits(), HtmlParseCancellation::new());
    let (mut other, mut other_session, _, _) =
        two_leaf_history(limits(), HtmlParseCancellation::new());
    let evidence = session.active().unwrap().evidence.as_ref().unwrap();
    let mut cursor = provenance::GroupOriginCursor::new(evidence, &group, &mut c).unwrap();
    assert!(matches!(
        cursor.next(evidence, &mut other),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert!(c.stop().is_none());
    assert!(cursor.release(&mut other).is_err());
    let first = cursor.next(evidence, &mut c).unwrap().unwrap();
    assert_eq!((first.start, first.end), (0, 1));
    cursor.release(&mut c).unwrap();
    other_session.release(&mut other).unwrap();
    storage_zero(&other);
    let mut cursor = provenance::GroupOriginCursor::new(evidence, &group, &mut c).unwrap();
    let foreign = ParseEvidence::prepare(&mut c).unwrap();
    assert!(matches!(
        cursor.next(&foreign, &mut c),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignEvidence
        )))
    ));
    cursor.release(&mut c).unwrap();
    foreign.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn history_cursor_lender_disposal_is_safe_for_owning_control_and_cancellation() {
    for cancelled in [false, true] {
        let cancel = HtmlParseCancellation::new();
        let (mut c, mut session, _, source) = two_leaf_history(limits(), cancel.clone());
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let baseline_capacity = c.usage().actual_capacity_bytes;
        {
            let mut rows = session.native_associations(&mut c).unwrap();
            let mut cursor = rows.group_origin_cursor(2).unwrap();
            if cancelled {
                cancel.cancel();
                assert!(matches!(
                    rows.next_group_origin(&mut cursor),
                    Err(SessionError::Stop(HtmlParseStop::Cancelled))
                ));
            } else {
                let first = rows.next_group_origin(&mut cursor).unwrap().unwrap();
                let second = rows.next_group_origin(&mut cursor).unwrap().unwrap();
                assert_eq!((first.start, first.end), (0, 1));
                assert_eq!((second.start, second.end), (1, 2));
                assert_eq!(first.source, source.range(0, 1).unwrap());
                assert_eq!(second.source, source.range(1, 2).unwrap());
                assert!(rows.next_group_origin(&mut cursor).unwrap().is_none());
            }
            let mut cursor = Some(cursor);
            rows.release_group_origin_cursor(&mut cursor).unwrap();
            assert!(cursor.is_none());
            assert_eq!(
                rows.release_group_origin_cursor(&mut cursor),
                Err(SessionError::Consumed)
            );
        }
        assert_eq!(c.usage().actual_capacity_bytes, baseline_capacity);
        let first = c.first_stop_usage().cloned();
        if cancelled {
            assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
            for category in [
                HtmlParseWorkCategory::StateStep,
                HtmlParseWorkCategory::InputUnitRead,
                HtmlParseWorkCategory::InspectByte,
                HtmlParseWorkCategory::CopyByte,
                HtmlParseWorkCategory::RecordVisit,
                HtmlParseWorkCategory::RecordWrite,
                HtmlParseWorkCategory::ReserveByte,
                HtmlParseWorkCategory::ProviderCall,
            ] {
                assert_eq!(
                    c.usage().work(category),
                    first.as_ref().unwrap().work(category)
                );
            }
        } else {
            assert!(c.stop().is_none());
            assert!(first.is_none());
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.first_stop_usage(), first.as_ref());
    }
}

#[test]
fn foreign_history_cursor_lender_disposal_keeps_cleanup_owned() {
    let (mut c, mut session, group, source) =
        two_leaf_history(limits(), HtmlParseCancellation::new());
    let (mut foreign, mut foreign_session, _, foreign_source) =
        two_leaf_history(limits(), HtmlParseCancellation::new());
    session
        .prepare_native_associations(&mut c, &source)
        .unwrap();
    foreign_session
        .prepare_native_associations(&mut foreign, &foreign_source)
        .unwrap();
    // Actual A/B ingress creates Leaf0, Leaf1 and their Concat2. Obtain
    // the owned scratch through the real completed lender, not a forged
    // cursor or a private field/ticket mutation.
    let baseline = c.usage().clone();
    let cursor = {
        let mut rows = session.native_associations(&mut c).unwrap();
        let mut cursor = rows.group_origin_cursor(2).unwrap();
        let first = rows.next_group_origin(&mut cursor).unwrap().unwrap();
        assert_eq!((first.start, first.end), (0, 1));
        assert_eq!(first.source, source.range(0, 1).unwrap());
        cursor
    };
    let mut cursor = Some(cursor);
    let owning_usage = c.usage().clone();
    assert!(
        owning_usage
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current
            > baseline
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current
    );
    {
        let mut rows = foreign_session.native_associations(&mut foreign).unwrap();
        assert_eq!(
            rows.release_group_origin_cursor(&mut cursor),
            Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
                crate::parser::control::HtmlParseInvariant::ForeignControl
            )))
        );
    }
    assert_eq!(c.usage(), owning_usage);
    assert!(c.stop().is_none());
    assert!(cursor.is_some());
    {
        let mut rows = session.native_associations(&mut c).unwrap();
        // Retry the exact refused cursor through its owning lender.
        rows.release_group_origin_cursor(&mut cursor).unwrap();
        assert!(cursor.is_none());
        assert_eq!(
            rows.release_group_origin_cursor(&mut cursor),
            Err(SessionError::Consumed)
        );
    }
    assert_eq!(
        c.usage().actual_capacity_bytes,
        baseline.actual_capacity_bytes
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        baseline
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current
    );
    let root = session.root(&mut c).unwrap();
    let storage = session.active().unwrap();
    let text = storage
        .node(
            &mut c,
            storage
                .construction
                .node_at(root.index)
                .expect("retained construction position")
                .child_at(0)
                .expect("retained child ordinal")
                .position(),
        )
        .unwrap();
    assert_eq!(native_text(&session, &text), [65, 66]);
    // The original native Text and historical value remain source-qualified
    // and usable; only the foreign disposal must be refused.
    let storage = session.active().unwrap();
    let output = storage
        .evidence
        .as_ref()
        .unwrap()
        .materialize_group(&mut c, storage.leaf_strings(), &group)
        .unwrap();
    assert_eq!(output.text().code_units(), [65, 66]);
    ParseEvidence::release_materialized_group(&mut c, &mut Some(output)).unwrap();
    let foreign_stop = foreign.first_stop_usage().unwrap().clone();
    foreign_session.release(&mut foreign).unwrap();
    storage_zero(&foreign);
    assert_eq!(foreign.first_stop_usage(), Some(&foreign_stop));
    session.release(&mut c).unwrap();
    assert!(c.stop().is_none());
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        0
    );
    // Foreign refusal and the actual owning retry must not abandon the
    // real traversal allocation or leave logical scratch charged after
    // disposal of the original session.
    storage_zero(&c);
}

#[test]
fn historical_materialization_exact_one_short_releases_output_and_traversal_on_first_stop() {
    let (mut measured, mut session, group, _) =
        two_leaf_history(limits(), HtmlParseCancellation::new());
    let base = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    let storage = session.active().unwrap();
    let output = storage
        .evidence
        .as_ref()
        .unwrap()
        .materialize_group(&mut measured, storage.leaf_strings(), &group)
        .unwrap();
    assert_eq!(output.text().code_units(), [65, 66]);
    let complete = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    ParseEvidence::release_materialized_group(&mut measured, &mut Some(output)).unwrap();
    session.release(&mut measured).unwrap();
    storage_zero(&measured);
    for short in [false, true] {
        let selected = limits()
            .into_builder()
            .work_credits(complete - u64::from(short))
            .finish()
            .unwrap();
        let (mut c, mut session, group, _) =
            two_leaf_history(selected, HtmlParseCancellation::new());
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            base
        );
        let before = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let storage = session.active().unwrap();
        let result = storage.evidence.as_ref().unwrap().materialize_group(
            &mut c,
            storage.leaf_strings(),
            &group,
        );
        if short {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
            assert!(c.stop().is_some());
        } else {
            let output = result.unwrap();
            assert_eq!(output.text().code_units(), [65, 66]);
            ParseEvidence::release_materialized_group(&mut c, &mut Some(output)).unwrap();
            assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
        }
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            before
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        let snapshot = c.first_stop_usage().unwrap().clone();
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        assert_eq!(c.first_stop_usage(), Some(&snapshot));
    }
}
#[test]
fn invalidated_pi_map_keeps_exact_retired_decoded_values_for_completion() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<?marker a='A' a='B'?>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    session.initialize_parse_evidence(&mut c, &source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut lexer, &mut c) else {
        panic!("literal PI");
    };
    let node = session
        .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
        .unwrap();
    let ConstructionNodeKind::ProcessingInstruction(pi) = &session
        .active()
        .unwrap()
        .construction
        .node_at(node.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("installed PI");
    };
    assert_eq!(
        pi.data().code_units(),
        DocumentString::from("a='A' a='B'").code_units()
    );
    let storage = session.active().unwrap();
    assert!(storage.payloads.values.iter().any(|p| matches!(p, RetainedPayload::PiAttributes { node:Some(owner),values,ready:true } if *owner==node.index && values.len()==0)));
    let evidence = storage.evidence.as_ref().unwrap();
    let mut retired = Vec::new();
    for payload in &storage.payloads.values {
        if let RetainedPayload::String {
            origins: Some(group),
            ..
        } = payload
            && evidence.require_group(group).unwrap().retirement.is_some()
        {
            let reconstructed = evidence
                .materialize_group(&mut c, storage.leaf_strings(), group)
                .unwrap();
            let rows = evidence.observed_origins(group, &mut c).unwrap();
            retired.push((
                reconstructed.text().code_units().to_vec(),
                rows.iter()
                    .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                    .collect::<Vec<_>>(),
            ));
            ParseEvidence::release_materialized_group(&mut c, &mut Some(reconstructed)).unwrap();
        }
    }
    assert_eq!(
        retired,
        vec![
            (vec![97], vec![(0, 1, 9, 10)]),
            (vec![65], vec![(0, 1, 12, 13)])
        ]
    );
    lexer.release(&mut c).unwrap();
    session
        .prepare_native_associations(&mut c, &source)
        .unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn raw_retirement_keeps_transformed_leaf_but_reclaims_equal_copy_aggregate() {
    let (mut c, mut session, original, _) =
        two_leaf_history(limits(), HtmlParseCancellation::new());
    for (units, expected_leaf) in [([65, 66], false), ([88, 89], true)] {
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        buffer.extend_units(&units, &mut c).unwrap();
        let mut maps = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        let storage = session.active_mut().unwrap();
        let evidence = storage.evidence.as_ref().unwrap();
        if !expected_leaf {
            for origin in evidence.observed_origins(&original, &mut c).unwrap() {
                maps.try_push(&mut c, origin).unwrap();
            }
        }
        let copied = evidence.copy_group_reference(&mut c, &original).unwrap();
        let value = buffer.freeze(&mut c).unwrap();
        let (native, slot) = storage
            .register_field_copy(
                &mut c,
                FieldCopy {
                    value,
                    origins: maps,
                    original: Some(copied),
                },
            )
            .unwrap();
        drop(native);
        let before = c
            .usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current;
        storage.retire_registered_string(&mut c, slot).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            before - if expected_leaf { 0 } else { 2 }
        );
        let group = match &storage.payloads.values[slot] {
            RetainedPayload::String {
                origins: Some(group),
                ..
            } if expected_leaf => group,
            RetainedPayload::HistoricalString { origins } if !expected_leaf => origins,
            _ => panic!("actual retired storage"),
        };
        let evidence = storage.evidence.as_ref().unwrap();
        let header = evidence.require_group(group).unwrap();
        assert!(header.retirement.is_some());
        assert_eq!(
            matches!(header.body, provenance::GroupBody::Leaf { .. }),
            expected_leaf
        );
        let observed = evidence
            .materialize_group(&mut c, storage.leaf_strings(), group)
            .unwrap();
        assert_eq!(observed.text().code_units(), units);
        ParseEvidence::release_materialized_group(&mut c, &mut Some(observed)).unwrap();
    }
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn equal_units_with_distinct_atomic_origin_partition_keep_a_new_leaf() {
    let (mut c, mut session, original, source) =
        two_leaf_history(limits(), HtmlParseCancellation::new());
    let mut buffer = Utf16Buffer::new(&mut c).unwrap();
    buffer.extend_units(&[65, 66], &mut c).unwrap();
    let mut maps = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
    maps.try_push(
        &mut c,
        HtmlParseTextOrigin {
            start: 0,
            end: 2,
            source: source.range(0, 2).unwrap(),
            kind: crate::parser::tokenizer::token::HtmlParseOriginKind::Literal,
        },
    )
    .unwrap();
    let value = buffer.freeze(&mut c).unwrap();
    let storage = session.active_mut().unwrap();
    let copied = storage
        .evidence
        .as_ref()
        .unwrap()
        .copy_group_reference(&mut c, &original)
        .unwrap();
    let (native, slot) = storage
        .register_field_copy(
            &mut c,
            FieldCopy {
                value,
                origins: maps,
                original: Some(copied),
            },
        )
        .unwrap();
    drop(native);
    let RetainedPayload::String {
        origins: Some(group),
        ..
    } = &storage.payloads.values[slot]
    else {
        panic!("actual leaf");
    };
    let evidence = storage.evidence.as_ref().unwrap();
    assert!(matches!(
        evidence.require_group(group).unwrap().body,
        provenance::GroupBody::Leaf { length: 1, .. }
    ));
    let rows = evidence.observed_origins(group, &mut c).unwrap();
    assert_eq!(
        rows.iter()
            .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
            .collect::<Vec<_>>(),
        [(0, 2, 0, 2)]
    );
    assert_eq!(
        evidence
            .observed_origins(&original, &mut c)
            .unwrap()
            .iter()
            .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
            .collect::<Vec<_>>(),
        [(0, 1, 0, 1), (1, 2, 1, 2)]
    );
    session
        .prepare_native_associations(&mut c, &source)
        .unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn materialized_history_alias_and_foreign_refusals_preserve_slot_for_real_cleanup() {
    let (mut c, mut session, group, _) = two_leaf_history(limits(), HtmlParseCancellation::new());
    let storage = session.active().unwrap();
    let mut output = Some(
        storage
            .evidence
            .as_ref()
            .unwrap()
            .materialize_group(&mut c, storage.leaf_strings(), &group)
            .unwrap(),
    );
    let alias = output.as_ref().unwrap().text().clone();
    let before = c.usage().clone();
    assert_eq!(
        ParseEvidence::release_materialized_group(&mut c, &mut output),
        Err(SessionError::AliasedPayload)
    );
    assert!(output.is_some());
    assert_eq!(alias.code_units(), [65, 66]);
    assert_eq!(c.usage(), before);
    assert!(c.stop().is_none());
    let mut foreign = control();
    assert_eq!(
        ParseEvidence::release_materialized_group(&mut foreign, &mut output),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    );
    assert!(output.is_some());
    assert_eq!(c.usage(), before);
    assert!(c.stop().is_none());
    drop(alias);
    // The original slot can be released successfully after its alias dies.
    ParseEvidence::release_materialized_group(&mut c, &mut output).unwrap();
    assert!(output.is_none());
    assert!(c.stop().is_none());
    let storage = session.active().unwrap();
    output = Some(
        storage
            .evidence
            .as_ref()
            .unwrap()
            .materialize_group(&mut c, storage.leaf_strings(), &group)
            .unwrap(),
    );
    // Actual work exhaustion latches the snapshot before cleanup.
    c.charge(HtmlParseWorkCategory::StateStep, limits().work_credits())
        .unwrap_err();
    let first = c.first_stop_usage().unwrap().clone();
    let work = c.usage().resource(HtmlParseResource::WorkCredits);
    ParseEvidence::release_materialized_group(&mut c, &mut output).unwrap();
    assert!(output.is_none());
    assert_eq!(c.first_stop_usage(), Some(&first));
    assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
    assert_eq!(
        ParseEvidence::release_materialized_group(&mut c, &mut output),
        Err(SessionError::Consumed)
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.first_stop_usage(), Some(&first));
    storage_zero(&foreign);
}

#[test]
fn completed_projection_refuses_real_pending_leaf_attachment() {
    let source = crate::HtmlSource::utf8("A");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    session.initialize_parse_evidence(&mut c, &source).unwrap();
    let group = session
        .evidence(&mut c)
        .unwrap()
        .rule_group(&mut c, 1)
        .unwrap();
    assert_eq!(
        session.prepare_native_associations(&mut c, &source).err(),
        Some(SessionError::WrongToken)
    );
    drop(group);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn append_series_retains_unique_leaves_plus_current_output_at_every_sample() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // N>0 retains N origin atoms, 2N-1 group headers and 2N-1
    // causes; projection adds two native node rows plus N current atoms.
    // The native prefix therefore retains 6N SourceMapEntries. Complete
    // D1 adds one fixed Cause row per retained cause (2N-1); all source-less
    // fixture causes emit no lineage rows, and no markup was consumed.
    // Empty input retains one Request, its root row and one Cause row.
    // Keep existing sample caps; only N256 uses this independently derived
    // logical allowance, including the meaningful final-row one-short cut.
    for (count, one_short) in [
        (0usize, false),
        (1, false),
        (8, false),
        (16, false),
        (32, false),
        (64, false),
        (256, false),
        (256, true),
    ] {
        let source = crate::HtmlSource::utf8("x".repeat(count));
        let caps = if count == 256 {
            limits()
                .into_builder()
                .source_map_entries((8 * count - 1 - usize::from(one_short)) as u64)
                // Retained descriptors/tickets/index (2N+5), indexed cause/group
                // slots (4N-2), and at most (2N-1) pending links coexist.
                // All fixture causes lack source, so the closure is empty.
                // Only N256 changes; smaller caps/work limits stay fixed.
                .auxiliary_entries((8 * count + 2) as u64)
                .active_formatting_entries((8 * count + 2) as u64)
                .finish()
                .unwrap()
        } else {
            limits()
        };
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut last = None;
        for position in 0..count {
            let mut text = Some(match lexical_kind(&mut lexer, &mut c) {
                TokenKind::Text(text) => text,
                _ => panic!("literal x"),
            });
            last = Some(
                session
                    .insert_text(&mut c, &root, None, &mut text, None)
                    .unwrap()
                    .unwrap(),
            );
            assert_eq!(last.as_ref().unwrap()._offset, position);
            // The text holds the units of its data: one for each token merged so far.
            let allowance = position + 1;
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                allowance as u64
            );
        }
        let last = last.map(|inserted| inserted._node);
        if let Some(last) = &last {
            assert_eq!(native_text(&session, last), vec![120; count]);
        } else {
            assert!(
                session
                    .active()
                    .unwrap()
                    .construction
                    .node_at(0)
                    .expect("retained construction position")
                    .child_count()
                    == 0
            );
        }
        // Metadata is one leaf per ingress, one Concat per merge and
        // fixed cause/retirement links. No historical-prefix atom dump.
        let evidence = session.active().unwrap().evidence.as_ref().unwrap();
        assert_eq!(
            evidence.history_counts(),
            if count == 0 {
                (0, 0, 1)
            } else {
                (count, 2 * count - 1, 2 * count - 1)
            }
        );
        lexer.release(&mut c).unwrap();
        let retained = c.usage().clone();
        if count == 256 {
            let storage = session.active().unwrap();
            assert_eq!(storage.construction.len(), 2);
            // The registry holds the two node-cause rows and no string row: the text is the
            // record's own, and the groups of its tokens are parts of one group.
            assert_eq!(storage.payloads.values.len(), 2);
            assert_eq!(
                storage
                    .payloads
                    .values
                    .iter()
                    .filter(|payload| matches!(payload, RetainedPayload::ConstructionCause { .. }))
                    .count(),
                2
            );
            assert_eq!(
                storage
                    .payloads
                    .values
                    .iter()
                    .filter(|payload| matches!(
                        payload,
                        RetainedPayload::String { .. } | RetainedPayload::HistoricalString { .. }
                    ))
                    .count(),
                0
            );
            // Two registry rows, a join cell for each of the two records, and the one entry of
            // the table of origins that holds the text's group.
            assert_eq!(
                retained
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                2 + 2 + 1
            );
        }
        let projection = session.prepare_native_associations(&mut c, &source);
        if one_short {
            assert_eq!(
                projection,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::SourceMapEntries,
                    limit: 2046,
                    current: 2046,
                    requested_delta: 1,
                }))
            );
            let first = c.first_stop_usage().unwrap().clone();
            assert_eq!(
                first.resource(HtmlParseResource::SourceMapEntries).current,
                2046
            );
            // The merged text holds its 256 units and nothing more.
            assert_eq!(
                first.resource(HtmlParseResource::ManagedUtf16Units).current,
                256
            );
            assert_eq!(
                session.native_associations(&mut c).err(),
                Some(SessionError::Stop(c.stop().unwrap()))
            );
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.first_stop_usage(), Some(&first));
            continue;
        }
        projection.unwrap();
        let projected = c.usage().clone();
        if count == 256 {
            assert!(
                projected
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .high_water
                    <= (8 * count + 2) as u64
            );
        }
        assert_eq!(
            projected
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            if count == 0 {
                3
            } else {
                (6 * count + 2 * count - 1) as u64
            }
        );
        {
            let mut rows = session.native_associations(&mut c).unwrap();
            // The node-cause rows come first, in registry order; the rows of the text's field
            // follow them, as the rows of the non-element fields do. Only current Text emits
            // output rows.
            assert_eq!(rows.len(), if count == 0 { 1 } else { count + 2 });
            assert!(rows.node_for(0, &root).unwrap());
            if let Some(last) = &last {
                assert!(rows.node_for(1, last).unwrap());
            }
            for index in 0..count {
                let output_row = index + 2;
                assert!(rows.text_for(output_row, last.as_ref().unwrap()).unwrap());
                let (row, _) = rows.row(output_row).unwrap().unwrap();
                assert_eq!((row.start, row.end), (index, index + 1));
                assert_eq!(
                    row.source.as_ref(),
                    Some(&source.range(index, index + 1).unwrap())
                );
                assert_eq!(
                    row.kind,
                    Some(crate::parser::tokenizer::token::HtmlParseOriginKind::Literal)
                );
            }
        }
        let categories = [
            HtmlParseWorkCategory::StateStep,
            HtmlParseWorkCategory::InputUnitRead,
            HtmlParseWorkCategory::InspectByte,
            HtmlParseWorkCategory::CopyByte,
            HtmlParseWorkCategory::RecordVisit,
            HtmlParseWorkCategory::RecordWrite,
            HtmlParseWorkCategory::ReserveByte,
            HtmlParseWorkCategory::ProviderCall,
        ];
        eprintln!(
            "append={count} retained_capacity={} retained_peak={} retained_work={:?} projection_work_delta={:?} projected_capacity={} projected_peak={}",
            retained.actual_capacity_bytes,
            retained.actual_capacity_high_water,
            categories.map(|category| retained.work(category)),
            categories.map(|category| projected.work(category) - retained.work(category)),
            projected.actual_capacity_bytes,
            projected.actual_capacity_high_water
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn source_less_append_lineage_visits_scale_with_retained_records_not_intervals_times_history() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // Independent fixture: Data emits N one-unit x leaves; real sink merges
    // them into one native Text. The direct sink's Request/Merge causes have
    // no direct source: complete D1 emits no causal lineage rows here.
    for count in [8usize, 16, 32] {
        let source = crate::HtmlSource::utf8("x".repeat(count));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut last = None;
        for offset in 0..count {
            let mut text = Some(match lexical_kind(&mut lexer, &mut c) {
                TokenKind::Text(text) => text,
                _ => panic!("literal x"),
            });
            let inserted = session
                .insert_text(&mut c, &root, None, &mut text, None)
                .unwrap()
                .unwrap();
            assert!(text.is_none());
            assert_eq!(inserted._offset, offset);
            last = Some(inserted._node);
        }
        assert_eq!(
            native_text(&session, last.as_ref().unwrap()),
            vec![120; count]
        );
        let evidence = session.active().unwrap().evidence.as_ref().unwrap();
        assert_eq!(
            evidence.history_counts(),
            (count, 2 * count - 1, 2 * count - 1)
        );
        // Actual tokenizer exhaustion qualifies the lexical EOF anchor in
        // the real session, not a manually supplied completion flag.
        let event = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap();
        assert!(
            matches!(&event, crate::parser::tokenizer::token::Event::Token(token) if matches!(&token.token_for_tests().kind, TokenKind::Eof))
        );
        event.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        let before = c.usage().work(HtmlParseWorkCategory::RecordVisit);
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let visits = c.usage().work(HtmlParseWorkCategory::RecordVisit) - before;
        assert_eq!(
            session.native_associations(&mut c).unwrap().len(),
            count + 2
        );
        let evidence = session.active().unwrap().evidence.as_ref().unwrap();
        let view = evidence.associations().unwrap();
        assert_eq!(view.len(), count + 2 + 2 * count - 1 + 1);
        assert!((0..2 * count - 1).all(|index| view.cause(index).unwrap().source().is_none()));
        let rows: Vec<_> = (0..view.len())
            .map(|index| view.row(index).unwrap())
            .collect();
        assert_eq!(
            rows.iter()
                .filter(|row| row.relation == provenance::HtmlParseAssociationRelation::Output)
                .count(),
            count
        );
        assert_eq!(
            rows.iter()
                .filter(|row| row.relation == provenance::HtmlParseAssociationRelation::Cause)
                .count(),
            2 * count - 1
        );
        assert_eq!(
            rows.iter()
                .filter(
                    |row| row.relation == provenance::HtmlParseAssociationRelation::LexicalAnchor
                )
                .count(),
            1
        );
        assert!(rows.iter().all(|row| matches!(
            row.relation,
            provenance::HtmlParseAssociationRelation::Output
                | provenance::HtmlParseAssociationRelation::Creation
                | provenance::HtmlParseAssociationRelation::Cause
                | provenance::HtmlParseAssociationRelation::LexicalAnchor
        )));
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            (8 * count) as u64
        );
        let first_stop = c.first_stop_usage().cloned();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.first_stop_usage(), first_stop.as_ref());
        // Fixed required record count: N origins, 2N-1 headers, 2N-1
        // causes, N Output/two node rows, 2N-1 final fixed Cause rows,
        // and the one actual EOF anchor.
        // A deliberately generous 256 record visits per required record
        // admits fixed ownership/preflight/stack/index operations, without
        // accepting rescanning the same cause closure for every interval.
        // This literal fixture allowance is not a normative global limit.
        let required_records = 8 * count;
        eprintln!(
            "source-less N={count}, required_records={required_records}, emitted_lineage=0, visits={visits}, allowance={}",
            256 * required_records
        );
        assert!(
            visits <= (256 * required_records) as u64,
            "source-less flattening repeats history: N={count}, visits={visits}, required_records={required_records}, no emitted lineage rows"
        );
    }
}

fn assert_merged_text_retains_qualified_origins(
    source: &crate::HtmlSource,
    expected: Vec<(
        usize,
        usize,
        crate::HtmlSourceRange,
        crate::parser::tokenizer::token::HtmlParseOriginKind,
    )>,
) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let mut c = control();
    c.admit_source(source).unwrap();
    let mut lexer = LexerFixture::new(source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut original: Option<ParserNode> = None;
    let mut first_origins = None;
    let mut incoming_origins = Vec::new();
    let mut output_length = 0;
    loop {
        match lexical_kind(&mut lexer, &mut c) {
            TokenKind::Text(text) => {
                let incoming_length = text.value.text().len();
                incoming_origins.extend(text.origins.as_slice().iter().map(|origin| {
                    (
                        output_length + origin.start,
                        output_length + origin.end,
                        origin.source.clone(),
                        origin.kind,
                    )
                }));
                let mut text = Some(text);
                let inserted = session
                    .insert_text(&mut c, &root, None, &mut text, None)
                    .unwrap()
                    .unwrap();
                assert!(text.is_none());
                assert_eq!(inserted._offset, output_length);
                output_length += incoming_length;
                if let Some(node) = &original {
                    assert_eq!(inserted._node.index, node.index);
                } else {
                    let storage = session.active_mut().unwrap();
                    let origins = storage
                        .payloads
                        .detail_origins
                        .peek(FieldRef::Text(inserted._node.index));
                    first_origins = origins.as_ref().map(|origins| {
                        storage
                            .observed_origins(origins, &mut c)
                            .unwrap()
                            .iter()
                            .map(|origin| {
                                (origin.start, origin.end, origin.source.clone(), origin.kind)
                            })
                            .collect::<Vec<_>>()
                    });
                    original = Some(inserted._node);
                }
            }
            TokenKind::Eof => break,
            _ => panic!("literal fixture contains only text and EOF"),
        }
    }
    let node = original.unwrap();
    let units = native_text(&session, &node);
    let storage = session.active_mut().unwrap();
    let origins = storage
        .payloads
        .detail_origins
        .peek(FieldRef::Text(node.index));
    let actual = origins.as_ref().map(|origins| {
        storage
            .observed_origins(origins, &mut c)
            .unwrap()
            .iter()
            .map(|origin| (origin.start, origin.end, origin.source.clone(), origin.kind))
            .collect::<Vec<_>>()
    });
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(
        c.usage().resource(HtmlParseResource::Diagnostics).current,
        0
    );
    assert_eq!(units, [0x41, 0xd83d, 0xde42, 0x0a, 0x2242, 0x0338]);
    assert_eq!(first_origins, Some(vec![expected[0].clone()]));
    assert_eq!(
        incoming_origins, expected,
        "actual tokenizer origins before native transfer"
    );
    assert_eq!(
        actual,
        Some(expected),
        "merged native field must retain rebased source origins"
    );
}

#[test]
fn merged_text_retains_rebased_utf8_scalar_newline_and_entity_origins() {
    use crate::parser::tokenizer::token::HtmlParseOriginKind::{
        CharacterReference, Literal, Newline,
    };
    let source = crate::HtmlSource::utf8("A🙂\r\n&NotEqualTilde;");
    assert_merged_text_retains_qualified_origins(
        &source,
        vec![
            (0, 1, source.range(0, 1).unwrap(), Literal),
            (1, 3, source.range(1, 5).unwrap(), Literal),
            (3, 4, source.range(5, 7).unwrap(), Newline),
            (4, 5, source.range(7, 22).unwrap(), CharacterReference),
            (5, 6, source.range(7, 22).unwrap(), CharacterReference),
        ],
    );
}

#[test]
fn merged_text_retains_rebased_utf16_scalar_newline_and_entity_origins() {
    use crate::parser::tokenizer::token::HtmlParseOriginKind::{
        CharacterReference, Literal, Newline,
    };
    let source = crate::HtmlSource::utf16(DocumentString::from("A🙂\r\n&NotEqualTilde;"));
    assert_merged_text_retains_qualified_origins(
        &source,
        vec![
            (0, 1, source.range(0, 1).unwrap(), Literal),
            (1, 3, source.range(1, 3).unwrap(), Literal),
            (3, 4, source.range(3, 5).unwrap(), Newline),
            (4, 5, source.range(5, 20).unwrap(), CharacterReference),
            (5, 6, source.range(5, 20).unwrap(), CharacterReference),
        ],
    );
}

#[test]
fn before_sibling_merges_only_preceding_text_and_document_ignores_text() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let element = session.allocate_element(&mut c, html("div")).unwrap();
    session.insert(&mut c, &root, &element, None, None).unwrap();
    let mut a = Some(owned_text(&mut c, &[0x61]));
    let a = session
        .insert_text(&mut c, &root, Some(&element), &mut a, None)
        .unwrap()
        .unwrap();
    let mut b = Some(owned_text(&mut c, &[0x62]));
    let b = session
        .insert_text(&mut c, &root, Some(&element), &mut b, None)
        .unwrap()
        .unwrap();
    assert_eq!(a._node.index, b._node.index);
    assert_eq!(native_text(&session, &a._node), [0x61, 0x62]);
    let mut after = Some(owned_text(&mut c, &[0x63]));
    let after = session
        .insert_text(&mut c, &root, None, &mut after, None)
        .unwrap()
        .unwrap();
    assert_ne!(a._node.index, after._node.index);
    session.release(&mut c).unwrap();
    storage_zero(&c);
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    assert!(session.fragment_root(&mut c).unwrap().is_none());
    let mut text = Some(owned_text(&mut c, &[0x61]));
    assert!(
        session
            .insert_text(&mut c, &root, None, &mut text, None)
            .unwrap()
            .is_none()
    );
    assert!(text.is_none());
    assert_eq!(session.active().unwrap().construction.len(), 1);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

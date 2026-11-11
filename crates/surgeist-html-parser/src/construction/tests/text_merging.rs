use super::*;

#[test]
fn text_append_spare_capacity_copy_work_depends_on_incoming_not_prefix() {
    let mut copied = Vec::new();
    for prefix in [8usize, 16, 32, 64, 128] {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut expected = vec![0x61; prefix];
        expected[0] = 0xd800;
        expected[1] = 0;
        // Build a real aggregate from two registered immutable lexical
        // leaves. Reserve its actual controlled backing before freezing:
        // ordinary insert_text currently makes exact-length aggregates,
        // so allocator rounding is not a sound spare-capacity fixture.
        let first_text = owned_text(&mut c, &expected);
        let second_text = owned_text(&mut c, &[0x62]);
        let storage = session.active_mut().unwrap();
        let (left, left_slot) = storage.register_text(&mut c, first_text).unwrap();
        let (right, right_slot) = storage.register_text(&mut c, second_text).unwrap();
        let mut aggregate = Utf16Buffer::new(&mut c).unwrap();
        aggregate.reserve_units(&mut c, prefix + 5).unwrap();
        aggregate.extend_units(left.code_units(), &mut c).unwrap();
        aggregate.extend_units(right.code_units(), &mut c).unwrap();
        drop((left, right));
        let origins = storage
            .merge_field_groups(&mut c, left_slot, right_slot, prefix, prefix + 1)
            .unwrap();
        let aggregate = aggregate.freeze(&mut c).unwrap();
        let (value, slot) = storage
            .register_group_string_owned(&mut c, aggregate, origins)
            .unwrap();
        let node = storage
            .allocate_owned(&mut c, NewRecord::Text(value), false, 0, 0)
            .unwrap();
        storage
            .bind_string(slot, FieldRef::Text(node.index))
            .unwrap();
        storage
            .insert(&mut c, &root, &node, None, InsertionCause::NonParser)
            .unwrap();
        let first = TextInsertion {
            _node: node,
            _offset: 0,
        };
        expected.push(0x62);
        let historical = expected.clone();
        let group = {
            let storage = session.active().unwrap();
            let slot = storage
                .field_slot(&mut c, FieldRef::Text(first._node.index))
                .unwrap();
            let RetainedPayload::String {
                origins: Some(group),
                ..
            } = &storage.payloads.values[slot]
            else {
                panic!("actual current text group")
            };
            let value = storage
                .construction
                .node_at(first._node.index)
                .unwrap()
                .stored_string(surgeist_html::bounded::construction::HtmlStoredString::Text)
                .expect("actual text");
            assert!(value.retained_units() - value.len() >= 4);
            assert_eq!(value.block().holders(), 2);
            assert_eq!(value.block().weak_holders(), 1);
            storage
                .evidence
                .as_ref()
                .unwrap()
                .copy_group_reference(&mut c, group)
                .unwrap()
        };
        let mut work = 0;
        for unit in [0xdc00, 0, 0xd800, 0x63] {
            let mut input = Some(owned_text(&mut c, &[unit]));
            let before = c.usage().work(HtmlParseWorkCategory::CopyByte);
            let next = session
                .insert_text(&mut c, &root, None, &mut input, None)
                .unwrap()
                .unwrap();
            work += c.usage().work(HtmlParseWorkCategory::CopyByte) - before;
            assert!(input.is_none());
            assert_eq!(next._node.index, first._node.index);
            assert_eq!(next._offset, expected.len());
            expected.push(unit);
            assert_eq!(native_text(&session, &first._node), expected);
        }
        let storage = session.active().unwrap();
        let evidence = storage.evidence.as_ref().unwrap();
        let old = evidence
            .materialize_group(&mut c, &storage.payloads, &group)
            .unwrap();
        assert_eq!(old.text().code_units(), historical);
        ParseEvidence::release_materialized_group(&mut c, &mut Some(old)).unwrap();
        assert_eq!(
            storage.construction.test_storage().usage().utf16_units(),
            prefix + 5
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            2 * (prefix + 5) as u64
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        copied.push((prefix, work));
    }
    // Same four incoming units, actual spare capacity, same fixed metadata
    // shape. Retained prefix length cannot add payload-copy work here.
    assert!(
        copied.iter().all(|(_, work)| *work == copied[0].1),
        "spare aggregate append copy work: {copied:?}"
    );
}

#[test]
fn text_append_current_ticket_lookup_is_independent_of_unrelated_payloads() {
    let mut visits = Vec::new();
    for comments in [0usize, 64] {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        for _ in 0..comments {
            let mut input = Some(owned_text(&mut c, &[0x78]));
            session.allocate_comment(&mut c, &mut input, None).unwrap();
            assert!(input.is_none());
        }
        let mut text = None;
        for unit in [0xd800, 0] {
            let mut input = Some(owned_text(&mut c, &[unit]));
            text = Some(
                session
                    .insert_text(&mut c, &root, None, &mut input, None)
                    .unwrap()
                    .unwrap()
                    ._node,
            );
        }
        let text = text.unwrap();
        let storage = session.active().unwrap();
        let before = c.usage().work(HtmlParseWorkCategory::RecordVisit);
        let slot = storage
            .field_slot(&mut c, FieldRef::Text(text.index))
            .unwrap();
        let work = c.usage().work(HtmlParseWorkCategory::RecordVisit) - before;
        let RetainedPayload::String {
            value,
            location: FieldRef::Text(owner),
            origins: Some(_),
            ..
        } = &storage.payloads.values[slot]
        else {
            panic!("registered current text")
        };
        assert_eq!(*owner, text.index);
        assert_eq!(value.code_units(), [0xd800, 0]);
        let native = storage
            .construction
            .node_at(text.index)
            .unwrap()
            .stored_string(surgeist_html::bounded::construction::HtmlStoredString::Text)
            .expect("actual text");
        assert!(native.shares_storage_with(value));
        session.release(&mut c).unwrap();
        storage_zero(&c);
        visits.push(work);
    }
    assert_eq!(
        visits[0], visits[1],
        "current text lookup visits real unrelated rows: {visits:?}"
    );
}

#[test]
fn text_append_callback_lookup_does_not_scan_unrelated_payloads() {
    let mut visits = Vec::new();
    for comments in [0usize, 64] {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut text = None;
        for unit in [0x61, 0x62] {
            let mut input = Some(owned_text(&mut c, &[unit]));
            text = Some(
                session
                    .insert_text(&mut c, &root, None, &mut input, None)
                    .unwrap()
                    .unwrap()
                    ._node,
            );
        }
        let text = text.unwrap();
        // These real detached records follow the current Text slot, so
        // they cannot affect its existing first-match lookup prefix.
        for _ in 0..comments {
            let mut input = Some(owned_text(&mut c, &[0x78]));
            session.allocate_comment(&mut c, &mut input, None).unwrap();
            assert!(input.is_none());
        }
        let mut input = Some(owned_text(&mut c, &[0xdc00]));
        let before = c.usage().work(HtmlParseWorkCategory::RecordVisit);
        let next = session
            .insert_text(&mut c, &root, None, &mut input, None)
            .unwrap()
            .unwrap();
        let work = c.usage().work(HtmlParseWorkCategory::RecordVisit) - before;
        assert!(input.is_none());
        assert_eq!(next._node.index, text.index);
        assert_eq!(next._offset, 2);
        assert_eq!(native_text(&session, &text), [0x61, 0x62, 0xdc00]);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            comments as u64 + 6
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        visits.push(work);
    }
    // At most eight conditional reserve/header sites in this append can
    // differ, each with at most four fixed record reads. The detached
    // comments are outside the native child walk and current-ticket prefix.
    assert!(
        visits[1] <= visits[0] + 32,
        "append callback work scans unrelated rows: {visits:?}"
    );
}

#[test]
fn character_insertion_merges_preceding_text_keeps_identity_and_retained_units() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut first = Some(owned_text(&mut c, &[0xd800, 0x20]));
    let first = session
        .insert_text(&mut c, &root, None, &mut first, None)
        .unwrap()
        .unwrap();
    assert_eq!(first._offset, 0);
    let mut next = Some(owned_text(&mut c, &[0xd83d, 0xde00, 0]));
    let next = session
        .insert_text(&mut c, &root, None, &mut next, None)
        .unwrap()
        .unwrap();
    assert_eq!(next._node.index, first._node.index);
    assert_eq!(next._offset, 2);
    assert_eq!(
        native_text(&session, &first._node),
        [0xd800, 0x20, 0xd83d, 0xde00, 0]
    );
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .test_storage()
            .usage()
            .utf16_units(),
        5
    );
    // Actual retained strings: old2 + incoming3 + merged5. Four origin
    // records are the two unique leaf atoms; three groups, two field causes,
    // merge and retirement give nine actual source-map entries.
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        10
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current,
        9
    );
    let mut frozen = session.freeze(&mut c).unwrap();
    assert_eq!(frozen.fragment().unwrap().usage().utf16_units(), 5);
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

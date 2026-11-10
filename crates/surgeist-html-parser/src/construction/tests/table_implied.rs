use super::*;
use surgeist_html::construction::{ConstructionNodeKind, HtmlDocumentMode};

#[test]
fn table_implied_names_are_real_managed_native_elements_without_lexical_origins() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let nodes = [
        session.allocate_implicit_colgroup(&mut c, None).unwrap(),
        session.allocate_implicit_tbody(&mut c, None).unwrap(),
        session.allocate_implicit_tr(&mut c, None).unwrap(),
    ];
    for node in &nodes {
        session.insert(&mut c, &root, node, None, None).unwrap();
    }
    // Three actual names (8+5+2) and three exact HTML URI strings (28 each).
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        99
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        4
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        0
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current,
        0
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        0
    );
    let storage = session.active().unwrap();
    assert!(storage.standalone_ineligibility.is_none());
    assert!(storage.payloads.values.iter().all(|entry| !matches!(
        entry,
        RetainedPayload::String {
            origins: Some(_),
            ..
        }
    )));
    let mut result = session.freeze(&mut c).unwrap();
    let fragment = result.fragment().unwrap();
    let actual: Vec<_> = fragment
        .root()
        .children()
        .map(|id| {
            let surgeist_html::construction::HtmlNodeKind::Element(element) =
                fragment.node(&id).unwrap().kind()
            else {
                panic!("native table element")
            };
            assert!(element.name().prefix().is_none());
            assert_eq!(element.attributes().len(), 0);
            assert_eq!(
                element.name().namespace().uri().unwrap().code_units(),
                b"http://www.w3.org/1999/xhtml"
                    .iter()
                    .map(|byte| u16::from(*byte))
                    .collect::<Vec<_>>()
            );
            element.name().local_name().code_units().to_vec()
        })
        .collect();
    assert_eq!(
        actual,
        vec![
            vec![99, 111, 108, 103, 114, 111, 117, 112],
            vec![116, 98, 111, 100, 121],
            vec![116, 114],
        ]
    );
    result.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn table_implied_names_have_exact_managed_unit_caps_and_reclaim_partial_backing() {
    for (kind, units) in [(0, 36), (1, 33), (2, 30)] {
        for cap in [units, units - 1] {
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .managed_utf16_units(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let result = match kind {
                0 => session.allocate_implicit_colgroup(&mut c, None),
                1 => session.allocate_implicit_tbody(&mut c, None),
                _ => session.allocate_implicit_tr(&mut c, None),
            };
            if cap == units {
                assert!(result.is_ok());
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    units
                );
                assert_eq!(
                    session.active().unwrap().construction.test_storage().len(),
                    2
                );
            } else {
                assert!(
                    matches!(result, Err(SessionError::Stop(HtmlParseStop::LimitExceeded { resource: HtmlParseResource::ManagedUtf16Units, limit, .. })) if limit == cap)
                );
                assert_eq!(
                    session.active().unwrap().construction.test_storage().len(),
                    1
                );
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(c.stop().unwrap()))
                );
            }
            let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
            let high = c
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water;
            let stop = c.stop();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                nodes
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .high_water,
                high
            );
        }
    }
}

#[test]
fn table_document_mode_reads_actual_root_and_has_independent_finite_cost() {
    for mode in [
        HtmlDocumentMode::NoQuirks,
        HtmlDocumentMode::LimitedQuirks,
        HtmlDocumentMode::Quirks,
    ] {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Document(mode)).unwrap();
        let visits = c.usage().work(HtmlParseWorkCategory::RecordVisit);
        let copies = c.usage().work(HtmlParseWorkCategory::CopyByte);
        let writes = c.usage().work(HtmlParseWorkCategory::RecordWrite);
        assert_eq!(session.document_mode(&mut c), Ok(mode));
        // Form readiness plus seven intrinsic-owner reads including Radio and Range + root discriminant, then the mode copy.
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::RecordVisit) - visits,
            9
        );
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::CopyByte) - copies,
            mem::size_of::<HtmlDocumentMode>() as u64
        );
        assert_eq!(c.usage().work(HtmlParseWorkCategory::RecordWrite), writes);
        session
            .set_document_mode(&mut c, HtmlDocumentMode::LimitedQuirks)
            .unwrap();
        assert_eq!(
            session.document_mode(&mut c),
            Ok(HtmlDocumentMode::LimitedQuirks)
        );
        let mut result = session.freeze(&mut c).unwrap();
        assert_eq!(
            result.document().unwrap().mode(),
            HtmlDocumentMode::LimitedQuirks
        );
        result.release(&mut c).unwrap();
        storage_zero(&c);
    }
    let mut c = control();
    let mut fragment = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    assert_eq!(
        fragment.document_mode(&mut c),
        Err(SessionError::NotDocument)
    );
    assert!(c.stop().is_none());
    fragment.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn table_producers_and_mode_reject_foreign_cancel_and_one_short_before_effects() {
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let mut foreign = control();
    let wrong = SessionError::Stop(HtmlParseStop::InternalInvariant(
        crate::parser::control::HtmlParseInvariant::ForeignControl,
    ));
    assert_eq!(
        session
            .allocate_implicit_colgroup(&mut foreign, None)
            .unwrap_err(),
        wrong
    );
    assert_eq!(session.document_mode(&mut foreign), Err(wrong));
    assert_eq!(
        session.active().unwrap().construction.test_storage().len(),
        1
    );
    assert_eq!(
        session.document_mode(&mut c),
        Ok(HtmlDocumentMode::NoQuirks)
    );
    cancellation.cancel();
    assert_eq!(
        session.allocate_implicit_tr(&mut c, None).unwrap_err(),
        SessionError::Stop(HtmlParseStop::Cancelled)
    );
    assert_eq!(
        session.document_mode(&mut c),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(
        session.active().unwrap().construction.test_storage().len(),
        1
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    // Independent getter envelope: nine actual reads (including Radio and Range readiness) and fixed mode copy.
    let complete = 9 + mem::size_of::<HtmlDocumentMode>() as u64;
    for remaining in [complete, complete - 1] {
        let mut c = control();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::Quirks)).unwrap();
        let unused =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(HtmlParseWorkCategory::StateStep, unused - remaining)
            .unwrap();
        let result = session.document_mode(&mut c);
        if remaining == complete {
            assert_eq!(result, Ok(HtmlDocumentMode::Quirks));
            assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
        } else {
            let stop = c.stop().unwrap();
            assert_eq!(result, Err(SessionError::Stop(stop)));
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
        }
        assert!(matches!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(0)
                .expect("retained construction position")
                .kind(),
            ConstructionNodeKind::Document(HtmlDocumentMode::Quirks)
        ));
        let stop = c.stop();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }
}

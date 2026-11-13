use super::*;
use surgeist_html::bounded::construction::HtmlConstructionNode;
use surgeist_html::construction::HtmlDocumentMode;

#[test]
fn combined_template_record_reserve_one_short_precedes_inert_document_and_cache_creation() {
    let mut baseline = control();
    let mut original = ParserArenaSession::new(
        &mut baseline,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
    )
    .unwrap();
    let initial = baseline.usage().actual_capacity_bytes;
    let (capacity, element_size) = test_support::record_bank(original.active().unwrap());
    let required = original.active().unwrap().construction.len() + 3; // inert Document + template + contents
    assert!(required > capacity);
    let one_short =
        initial - (capacity * element_size) as u64 + (required * element_size) as u64 - 1;
    original.release(&mut baseline).unwrap();
    let selected = limits()
        .into_builder()
        .managed_capacity_bytes(one_short)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let payloads = session.active().unwrap().payloads.values.len();
    let before_capacity = c.usage().actual_capacity_bytes;
    let error = session
        .active_mut()
        .unwrap()
        .prepare_document_allocation(&mut c, Some(DocumentRef::Owned(0)), true)
        .err();
    assert!(
        matches!(error,Some(SessionError::Stop(HtmlParseStop::LimitExceeded{resource:HtmlParseResource::ManagedCapacityBytes,limit,..})) if limit==one_short)
    );
    let storage = session.active().unwrap();
    assert_eq!(storage.construction.len(), 1);
    assert_eq!(storage.payloads.values.len(), payloads);
    assert!(storage.payloads.values.iter().any(|payload| matches!(
        payload,
        RetainedPayload::Document {
            node: Some(0),
            state: template::DocumentState::Ordinary { inert: None },
            ..
        }
    )));
    assert!(!storage.payloads.values.iter().any(|payload| matches!(
        payload,
        RetainedPayload::Template { .. }
            | RetainedPayload::Document {
                state: template::DocumentState::InertTemplate,
                ..
            }
    )));
    // The reservation grows two banks, the rows and then their depths. The limit that is one byte
    // short of both refuses the second, and the first keeps the room it reserved, as a grown bank
    // does: the parse holds what it held and the rows the geometric rule chose, the greater of the
    // records required and twice the capacity, and nothing of the depths.
    let row_bytes = HtmlConstructionNode::ROW_BYTES;
    assert_eq!(element_size, row_bytes + size_of::<u32>());
    let grown = required.max(2 * capacity);
    let held = c.usage().actual_capacity_bytes;
    assert_eq!(before_capacity, initial);
    assert_eq!(
        held,
        before_capacity + ((grown - capacity) * row_bytes) as u64
    );
    assert!(held < one_short);
    let first = c.stop();
    session.release(&mut c).unwrap();
    assert_eq!(c.stop(), first);
    assert_eq!(c.usage().actual_capacity_bytes, 0);
}

#[test]
fn record_reserve_grows_the_bank_geometrically_and_keeps_the_exact_layout_gate() {
    let mut c = control();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let storage = session.active_mut().unwrap();
    let held = storage.construction.len();
    let mut capacities = vec![test_support::record_bank(storage).0];
    // Ask for ever more records, as a growing document does. An exact
    // policy changes the bank for every larger request.
    for additional in 1..=40 {
        storage.reserve_records(&mut c, additional).unwrap();
        let capacity = test_support::record_bank(storage).0;
        assert!(capacity >= held + additional);
        if capacities.last() != Some(&capacity) {
            capacities.push(capacity);
        }
    }
    assert!(capacities.len() <= 8, "{capacities:?}");
    assert!(capacities.windows(3).all(|run| run[2] >= 2 * run[1]));
    session.release(&mut c).unwrap();
    assert_eq!(c.usage().actual_capacity_bytes, 0);
}

#[test]
fn record_reserve_admits_the_doubled_target_and_a_limit_below_it_refuses_the_reservation() {
    // Grows an eight-slot bank to ten records, the bank doubling to sixteen.
    let session_at_eight = |limits: HtmlParseLimits| {
        let mut c = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let storage = session.active_mut().unwrap();
        storage.reserve_records(&mut c, 7).unwrap();
        assert_eq!(test_support::record_bank(storage).0, 8);
        (c, session)
    };
    let (mut baseline, mut original) = session_at_eight(limits());
    let at_eight = baseline.usage().actual_capacity_bytes;
    let storage = original.active().unwrap();
    let (capacity, element_size) = test_support::record_bank(storage);
    let required = storage.construction.len() + 9;
    assert!(capacity < required && required < 2 * capacity);
    let exact_fit = at_eight - (capacity * element_size) as u64 + (required * element_size) as u64;
    let doubled_fit =
        at_eight - (capacity * element_size) as u64 + (2 * capacity * element_size) as u64;
    assert!(exact_fit < doubled_fit);
    original.release(&mut baseline).unwrap();

    // The doubled target fits exactly: the bank reaches sixteen slots and all of them are charged.
    let (mut fits, mut session) = session_at_eight(
        limits()
            .into_builder()
            .managed_capacity_bytes(doubled_fit)
            .finish()
            .unwrap(),
    );
    let storage = session.active_mut().unwrap();
    storage.reserve_records(&mut fits, 9).unwrap();
    assert_eq!(test_support::record_bank(storage).0, 2 * capacity);
    assert_eq!(fits.usage().actual_capacity_bytes, doubled_fit);
    session.release(&mut fits).unwrap();

    // One byte less refuses, although the required ten records alone would fit: a limit never
    // shrinks the target. A limit that holds the required layout exactly refuses the same way.
    for limit in [doubled_fit - 1, exact_fit] {
        let (mut refuses, mut session) = session_at_eight(
            limits()
                .into_builder()
                .managed_capacity_bytes(limit)
                .finish()
                .unwrap(),
        );
        let error = session
            .active_mut()
            .unwrap()
            .reserve_records(&mut refuses, 9)
            .err();
        assert!(
            matches!(error, Some(SessionError::Stop(HtmlParseStop::LimitExceeded { resource: HtmlParseResource::ManagedCapacityBytes, limit: refused, .. })) if refused == limit)
        );
        session.release(&mut refuses).unwrap();
    }
}

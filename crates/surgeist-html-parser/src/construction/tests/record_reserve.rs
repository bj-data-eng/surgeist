use super::*;

#[test]
fn combined_template_record_reserve_one_short_precedes_inert_document_and_cache_creation() {
    let mut baseline = control();
    let mut original = ParserArenaSession::new(
        &mut baseline,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
    )
    .unwrap();
    let initial = baseline.usage().actual_capacity_bytes;
    let (capacity, element_size) = original
        .active()
        .unwrap()
        .construction
        .actual_record_layout();
    let required = original.active().unwrap().construction.test_storage().len() + 3; // inert Document + template + contents
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
    assert_eq!(storage.construction.test_storage().len(), 1);
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
    assert_eq!(c.usage().actual_capacity_bytes, before_capacity);
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
    let held = storage.construction.test_storage().len();
    let mut capacities = vec![storage.construction.actual_record_layout().0];
    // Ask for ever more records, as a growing document does. An exact
    // policy changes the bank for every larger request.
    for additional in 1..=40 {
        storage
            .construction
            .reserve_records(&mut c, additional)
            .unwrap();
        let capacity = storage.construction.actual_record_layout().0;
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
fn record_reserve_clamps_spare_capacity_to_the_limit_and_keeps_the_exact_refusal() {
    // Grows an eight-slot bank to ten records, the bank doubling to sixteen.
    let session_at_eight = |limits: HtmlParseLimits| {
        let mut c = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let storage = session.active_mut().unwrap();
        storage.construction.reserve_records(&mut c, 7).unwrap();
        assert_eq!(storage.construction.actual_record_layout().0, 8);
        (c, session)
    };
    let (mut baseline, mut original) = session_at_eight(limits());
    let at_eight = baseline.usage().actual_capacity_bytes;
    let storage = original.active().unwrap();
    let (capacity, element_size) = storage.construction.actual_record_layout();
    let required = storage.construction.test_storage().len() + 9;
    assert!(capacity < required && required < 2 * capacity);
    let exact_fit = at_eight - (capacity * element_size) as u64 + (required * element_size) as u64;
    original.release(&mut baseline).unwrap();

    let (mut fits, mut session) = session_at_eight(
        limits()
            .into_builder()
            .managed_capacity_bytes(exact_fit)
            .finish()
            .unwrap(),
    );
    let storage = session.active_mut().unwrap();
    storage.construction.reserve_records(&mut fits, 9).unwrap();
    // Spare capacity stops at the limit instead of doubling past it.
    assert_eq!(storage.construction.actual_record_layout().0, required);
    assert_eq!(fits.usage().actual_capacity_bytes, exact_fit);
    session.release(&mut fits).unwrap();

    let (mut refuses, mut session) = session_at_eight(
        limits()
            .into_builder()
            .managed_capacity_bytes(exact_fit - 1)
            .finish()
            .unwrap(),
    );
    let error = session
        .active_mut()
        .unwrap()
        .construction
        .reserve_records(&mut refuses, 9)
        .err();
    assert!(
        matches!(error, Some(SessionError::Stop(HtmlParseStop::LimitExceeded { resource: HtmlParseResource::ManagedCapacityBytes, limit, .. })) if limit == exact_fit - 1)
    );
    session.release(&mut refuses).unwrap();
}

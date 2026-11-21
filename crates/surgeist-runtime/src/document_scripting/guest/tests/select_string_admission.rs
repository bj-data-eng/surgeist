use super::*;
use surgeist_dom::DomBatch;
use surgeist_html::values::{DocumentString, Namespace};

const EXPECTED: &[u16] = &[0x0041, 0x0020, 0xd800, 0x0042, 0x0020, 0x0043];

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session_with_dom_limits(
            &cancel,
            0,
            0,
            |limits| {
                limits
                    .with_select_processing(8, 4_000_000, 100_000)
                    .unwrap()
            },
        );
    let doc = target.document();
    let mut batch = DomBatch::new();
    let select = batch.create_element_ns(doc, Namespace::html(), "select");
    let option = batch.create_element_ns(doc, Namespace::html(), "option");
    let nested = batch.create_element_ns(doc, Namespace::html(), "b");
    let opening = batch.create_text(doc, "  A\t ");
    let trailing = batch.create_text(
        doc,
        DocumentString::from_utf16(vec![0xd800, 0x0042, 0x0020, 0x0020, 0x0043, 0x0020]),
    );
    batch.set_attribute(&select, "multiple", "");
    batch.insert_before(&option, &select, None);
    batch.insert_before(&nested, &option, None);
    batch.insert_before(&opening, &nested, None);
    batch.insert_before(&trailing, &option, None);
    batch.set_option_selected(&option, true);
    let select = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&select)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, select, "selectValueProbe");
    (host, permit)
}

fn observe(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| {
        let result = scope.evaluate(&units("selectValueProbe.value"))?;
        assert_eq!(scope.to_utf16(&result)?, EXPECTED);
        Ok(())
    })
}

pub(in crate::document_scripting) fn select_value_admits_native_text_and_its_guest_copy() {
    let (mut host, permit) = fixture();
    let events = host.guest_events().unwrap();
    let initial = events.budget.used();
    let prior_work = events.budget.work().unwrap();
    let (result, peak) = events.budget.measure_peak(|| observe(&mut host, &permit));
    result.unwrap();
    let required_work = events.budget.work().unwrap() - prior_work;
    let required_storage = peak - initial;
    assert!(required_work > 0);
    assert!(required_storage >= size_of_val(EXPECTED));
    assert_eq!(events.budget.used(), initial);
    let budget = events.budget.clone();
    drop(events);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);

    for (work, bytes) in [
        (required_work, required_storage),
        (required_work - 1, required_storage),
        (required_work, required_storage - 1),
    ] {
        let (mut host, permit) = fixture();
        let events = host.guest_events().unwrap();
        let spare = events.limits.storage_bytes - events.budget.used() - bytes;
        let fill = events.reserve(&host.native, spare).unwrap();
        events.budget.set_work_remaining(work);
        let stamp = host.snapshot().unwrap().stamp();
        let initial = events.budget.used();
        let result = observe(&mut host, &permit);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(events.budget.used(), initial);
        if work == required_work && bytes == required_storage {
            result.unwrap();
            assert_eq!(events.budget.work_remaining().unwrap(), 0);
            assert_eq!(events.budget.peak_used(), events.limits.storage_bytes);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short Select value read must refuse: {result:?}");
            };
            let cause = failure.cause::<DocumentGuestEventError>().unwrap();
            assert!(
                matches!(
                    (work < required_work, cause),
                    (true, DocumentGuestEventError::WorkLimit)
                        | (
                            false,
                            DocumentGuestEventError::Document(DocumentScriptError::Native(
                                DocumentScriptNativeError::StorageLimit
                            ))
                        )
                ),
                "{cause:?}"
            );
            let Err(DocumentScriptError::Script(Error::HostFailure(repeated))) =
                host.in_document(&permit, |_| Ok(()))
            else {
                panic!("the original Select value refusal must remain terminal");
            };
            assert!(std::ptr::eq(
                cause,
                repeated.cause::<DocumentGuestEventError>().unwrap(),
            ));
        }
        drop(fill);
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

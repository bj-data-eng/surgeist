use super::*;

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    (host, permit)
}

fn observe(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| {
        let result = scope.evaluate(&units("document.URL === 'about:blank'"))?;
        assert!(
            scope.boolean(&result)?,
            "Document.URL must return the canonical native URL"
        );
        Ok(())
    })
}

#[test]
fn metadata_getter_admits_its_native_observation_and_copy() {
    let (mut host, permit) = fixture();
    let events = host.guest_events().unwrap();
    let baseline = events.budget.used();
    let before_work = events.budget.work().unwrap();
    let (result, peak) = events.budget.measure_peak(|| observe(&mut host, &permit));
    result.unwrap();
    let required_work = events.budget.work().unwrap() - before_work;
    let required_bytes = peak - baseline;
    assert_eq!(events.budget.used(), baseline);
    drop(events);
    host.stop().into_session().unwrap();

    for (work, bytes) in [
        (required_work, required_bytes),
        (required_work - 1, required_bytes),
        (required_work, required_bytes - 1),
    ] {
        let (mut host, permit) = fixture();
        let events = host.guest_events().unwrap();
        let spare = events.limits.storage_bytes - events.budget.used() - bytes;
        let fill = events.reserve(&host.native, spare).unwrap();
        events.budget.set_work_remaining(work);
        let stamp = host.snapshot().unwrap().stamp();
        let used = events.budget.used();
        let observed = observe(&mut host, &permit);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(events.budget.used(), used);
        if work == required_work && bytes == required_bytes {
            observed.unwrap();
            assert_eq!(events.budget.work_remaining().unwrap(), 0);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = observed else {
                panic!("a one-short metadata observation must refuse: {observed:?}");
            };
            let cause = failure.cause::<DocumentGuestEventError>().unwrap();
            assert!(matches!(
                (work < required_work, cause),
                (true, DocumentGuestEventError::WorkLimit)
                    | (
                        false,
                        DocumentGuestEventError::Document(DocumentScriptError::Native(
                            DocumentScriptNativeError::StorageLimit
                        ))
                    )
            ));
            let Err(DocumentScriptError::Script(Error::HostFailure(repeated))) = host
                .in_document::<()>(&permit, |_| {
                    panic!("refused metadata getter reentered guest code")
                })
            else {
                panic!("the original metadata refusal must remain terminal");
            };
            assert!(std::ptr::eq(
                cause,
                repeated.cause::<DocumentGuestEventError>().unwrap()
            ));
        }
        drop(fill);
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

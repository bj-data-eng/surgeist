//! Exact and one-short work and storage for live member collection getters.

use super::*;

struct MemberCall {
    receiver: Root,
    getter: Root,
}

impl MemberCall {
    fn prepare(
        host: &mut DocumentScriptHost,
        permit: &DocumentScriptPermit,
        map: bool,
        cached: bool,
    ) -> Self {
        let call = host.in_document(permit, |scope| {
            let (root, getter) = if map {
                ("document.createElement('map')",
                    "Object.getOwnPropertyDescriptor(HTMLMapElement.prototype, 'areas').get")
            } else {
                ("document.createElement('datalist')",
                    "Object.getOwnPropertyDescriptor(HTMLDataListElement.prototype, 'options').get")
            };
            let receiver = scope.evaluate(&units(root))?;
            let getter = scope.evaluate(&units(getter))?;
            // Publish the SameObject association before either measurement.
            // Preparing through this same original getter also authenticates
            // the member; no prototype-name lookup occurs inside the measure.
            if cached {
                let value = scope.call(&getter, &receiver, &[])?;
                assert!(matches!(scope.kind(&value)?, surgeist_bindings_js::ValueKind::Object));
            }
            Ok(Self {receiver: scope.retain(&receiver), getter: scope.retain(&getter)})
        }).unwrap();
        fund_guest_operation_after_setup(host);
        call
    }

    fn read(
        &self,
        host: &mut DocumentScriptHost,
        permit: &DocumentScriptPermit,
    ) -> Result<(), DocumentScriptError> {
        host.in_document(permit, |scope| {
            let receiver = scope.import(&self.receiver)?;
            let getter = scope.import(&self.getter)?;
            scope.call(&getter, &receiver, &[]).map(drop)
        })
    }
}

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, top) = crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let host = attach_public_nodes(session, top, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    (host, permit)
}

pub(in crate::document_scripting) fn member_getters_admit_exact_native_work_and_storage() {
    for (map, cached) in [(true, true), (false, true), (true, false), (false, false)] {
        let (mut host, permit) = fixture();
        let call = MemberCall::prepare(&mut host, &permit, map, cached);
        let events = host.guest_events().unwrap();
        let before = (events.budget.work().unwrap(), events.budget.used());
        let (result, peak) = events.budget.measure_peak(|| call.read(&mut host, &permit));
        result.unwrap();
        let work = events.budget.work().unwrap() - before.0;
        let bytes = peak - before.1;
        let retained = events.budget.used() - before.1;
        assert!(work > 0 && bytes > 0);
        assert_eq!(retained == 0, cached);
        drop(call);
        drop(events);
        host.stop().into_session().unwrap();

        for (remaining_work, remaining_bytes) in
            [(work, bytes), (work - 1, bytes), (work, bytes - 1)]
        {
            let (mut host, permit) = fixture();
            let call = MemberCall::prepare(&mut host, &permit, map, cached);
            let events = host.guest_events().unwrap();
            let fill = events
                .reserve(
                    &host.native,
                    events.limits.storage_bytes - events.budget.used() - remaining_bytes,
                )
                .unwrap();
            events.budget.set_work_remaining(remaining_work);
            let stamp = host.snapshot().unwrap().stamp();
            let adapter = events.budget.used() - events.budget.capture_used();
            let (result, peak) = events.budget.measure_peak(|| call.read(&mut host, &permit));
            let accepted = result.is_ok();
            match result {
                Ok(()) => {
                    assert_eq!((remaining_work, remaining_bytes), (work, bytes));
                    assert_eq!(events.budget.work_remaining().unwrap(), 0);
                    assert_eq!(peak, events.limits.storage_bytes);
                }
                Err(DocumentScriptError::Script(Error::HostFailure(failure))) => {
                    if remaining_work < work {
                        assert!(matches!(
                            failure.cause::<DocumentGuestEventError>(),
                            Some(DocumentGuestEventError::WorkLimit)
                        ));
                    } else {
                        assert_eq!(remaining_bytes, bytes - 1);
                        assert!(matches!(
                            failure.cause::<DocumentGuestEventError>(),
                            Some(DocumentGuestEventError::Document(
                                DocumentScriptError::Native(
                                    DocumentScriptNativeError::StorageLimit
                                )
                            ))
                        ));
                    }
                    drop(failure);
                }
                Err(error) => panic!("unexpected member getter admission: {error:?}"),
            }
            assert_eq!(host.snapshot().unwrap().stamp(), stamp);
            // Original failure reporting may retain its preparation record;
            // keep the established scope exclusion, not an ad hoc byte slack.
            assert_eq!(
                events.budget.used() - events.budget.capture_used(),
                adapter + if accepted { retained } else { 0 },
            );
            let budget = events.budget.clone();
            drop(call);
            drop(fill);
            drop(events);
            host.stop().into_session().unwrap();
            assert_eq!(budget.used(), 0);
        }
    }
}

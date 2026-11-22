use super::*;

struct FactoryCall {
    receiver: Root,
    method: Root,
    alias: Root,
}

impl FactoryCall {
    fn prepare(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) -> Self {
        host.in_document(permit, |scope| {
            let receiver = scope.evaluate(&units("document"))?;
            let method = scope.get(&receiver, PropertyKey::String(&units("createEvent")))?;
            let alias = scope.string(&units("Event"))?;
            Ok(Self {
                receiver: scope.retain(&receiver),
                method: scope.retain(&method),
                alias: scope.retain(&alias),
            })
        })
        .unwrap()
    }

    fn create(
        &self,
        host: &mut DocumentScriptHost,
        permit: &DocumentScriptPermit,
    ) -> Result<Root, DocumentScriptError> {
        host.in_document(permit, |scope| {
            let receiver = scope.import(&self.receiver)?;
            let method = scope.import(&self.method)?;
            let alias = scope.import(&self.alias)?;
            let event = scope.call(&method, &receiver, &[alias])?;
            Ok(scope.retain(&event))
        })
    }
}

fn event_bytes(host: &mut DocumentScriptHost) -> usize {
    host.with_native(|_, hosted, _| hosted.event_storage.used_bytes())
        .unwrap()
}

pub(in crate::document_scripting) fn legacy_creation_releases_each_owner() {
    let mut host = public_node_host(20_000, 200_000);
    let permit = host.permit().unwrap();
    let call = FactoryCall::prepare(&mut host, &permit);
    let events = host.guest_events().unwrap();
    let baseline = events.budget.used();
    let starting_work = events.budget.work().unwrap();
    let native_baseline = event_bytes(&mut host);
    let (result, peak) = events
        .budget
        .measure_peak(|| call.create(&mut host, &permit));
    let event = result.unwrap();
    let exact_work = events.budget.work().unwrap() - starting_work;
    let storage = peak - baseline;
    assert!(exact_work > 0 && storage > 0);
    assert!(events.budget.used() > baseline);
    assert!(event_bytes(&mut host) > native_baseline);

    drop(event);
    host.engine
        .as_mut()
        .unwrap()
        .process
        .execution()
        .memory_pressure();
    assert_eq!(events.budget.used(), baseline);
    assert_eq!(event_bytes(&mut host), native_baseline);
    drop(call);
    drop(events);
    host.stop().into_session().unwrap();

    for (bytes, work) in [
        (storage, exact_work),
        (storage - 1, exact_work),
        (storage, exact_work - 1),
    ] {
        let mut host = public_node_host(20_000, 200_000);
        let permit = host.permit().unwrap();
        let call = FactoryCall::prepare(&mut host, &permit);
        let events = host.guest_events().unwrap();
        let fill = events
            .reserve(
                &host.native,
                events.limits.storage_bytes - events.budget.used() - bytes,
            )
            .unwrap();
        events.budget.set_work_remaining(work);
        let used = events.budget.used() - events.budget.capture_used();
        let native = event_bytes(&mut host);
        let before = host.snapshot().unwrap().stamp();
        let (result, peak) = events
            .budget
            .measure_peak(|| call.create(&mut host, &permit));
        match result {
            Ok(event) => {
                assert_eq!((bytes, work), (storage, exact_work));
                assert_eq!(events.budget.work_remaining().unwrap(), 0);
                assert_eq!(peak, events.limits.storage_bytes);
                assert!(event_bytes(&mut host) > native);
                drop(event);
            }
            Err(DocumentScriptError::Script(Error::HostFailure(failure))) => {
                let cause = failure.cause::<DocumentGuestEventError>();
                if bytes < storage {
                    assert!(matches!(
                        cause,
                        Some(DocumentGuestEventError::Document(
                            DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
                        ))
                    ));
                } else {
                    assert_eq!(work, exact_work - 1);
                    assert!(matches!(cause, Some(DocumentGuestEventError::WorkLimit)));
                }
                drop(failure);
                assert_eq!(
                    events.budget.used() - events.budget.capture_used(),
                    used,
                    "the refused projection must release its adapter backing"
                );
                assert_eq!(event_bytes(&mut host), native);
            }
            Err(error) => panic!("unexpected factory admission: {error:?}"),
        }
        assert_eq!(host.snapshot().unwrap().stamp(), before);
        let budget = events.budget.clone();
        drop(call);
        drop(fill);
        drop(events);
        let session = host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
        assert_eq!(
            session.hosted_execution().event_storage.used_bytes(),
            native
        );
    }
}

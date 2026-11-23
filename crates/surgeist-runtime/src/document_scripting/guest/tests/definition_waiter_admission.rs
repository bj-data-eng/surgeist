use super::*;

struct PendingCall {
    receiver: Root,
    method: Root,
    name: Root,
}

impl PendingCall {
    fn prepare(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) -> Self {
        host.in_document(permit, |scope| {
            let receiver = scope.evaluate(&units("customElements"))?;
            let method = scope.get(&receiver, PropertyKey::String(&units("whenDefined")))?;
            let name = scope.string(&units("x-funded-promise"))?;
            Ok(Self {
                receiver: scope.retain(&receiver),
                method: scope.retain(&method),
                name: scope.retain(&name),
            })
        })
        .unwrap()
    }

    fn invoke(
        &self,
        host: &mut DocumentScriptHost,
        permit: &DocumentScriptPermit,
    ) -> Result<Root, DocumentScriptError> {
        host.in_document(permit, |scope| {
            let receiver = scope.import(&self.receiver)?;
            let method = scope.import(&self.method)?;
            let name = scope.import(&self.name)?;
            let promise = scope.call(&method, &receiver, &[name])?;
            Ok(scope.retain(&promise))
        })
    }
}

fn measure_pending_call() -> usize {
    let mut host = public_node_host(20_000, 200_000);
    let permit = host.permit().unwrap();
    let call = PendingCall::prepare(&mut host, &permit);
    let events = host.guest_events().unwrap();
    let starting = events.budget.used();
    let (result, peak) = events
        .budget
        .measure_peak(|| call.invoke(&mut host, &permit));
    let first = result.unwrap();
    let repeated = call.invoke(&mut host, &permit).unwrap();
    host.in_document(&permit, |scope| {
        let first = scope.import(&first)?;
        let repeated = scope.import(&repeated)?;
        assert!(scope.strict_equals(&first, &repeated)?);
        Ok(())
    })
    .unwrap();
    peak.checked_sub(starting).unwrap()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    Accepted,
    OneByteShort,
}

fn storage_refusal(error: DocumentScriptError) -> surgeist_bindings_js::HostFailure {
    let DocumentScriptError::Script(Error::HostFailure(failure)) = error else {
        panic!("expected waiter storage refusal, got {error:?}");
    };
    assert!(
        matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
            ))
        ) || matches!(
            failure.cause::<DocumentScriptNativeError>(),
            Some(DocumentScriptNativeError::StorageLimit)
        ),
        "expected the reached storage cause, got {failure:?}"
    );
    failure
}

pub(in crate::document_scripting) fn waiter_publication_preserves_admission() {
    let required = measure_pending_call();
    assert!(required > 0);
    for (available, expected) in [
        (required, Outcome::Accepted),
        (required - 1, Outcome::OneByteShort),
    ] {
        let mut host = public_node_host(20_000, 200_000);
        let permit = host.permit().unwrap();
        let call = PendingCall::prepare(&mut host, &permit);
        let events = host.guest_events().unwrap();
        let fill = events
            .reserve(
                &host.native,
                events.limits.storage_bytes - events.budget.used() - available,
            )
            .unwrap();
        let total = events.budget.used();
        let preparation = events.budget.capture_used();
        let stamp = host.snapshot().unwrap().stamp();
        let (result, peak) = events
            .budget
            .measure_peak(|| call.invoke(&mut host, &permit));
        match result {
            Ok(promise) => {
                assert_eq!(expected, Outcome::Accepted);
                assert_eq!(peak, events.limits.storage_bytes);
                assert!(events.budget.used() > total);
                drop(promise);
                drop(fill);
            }
            Err(error) => {
                assert_eq!(expected, Outcome::OneByteShort);
                let failure = storage_refusal(error);
                assert_eq!(events.budget.used(), total);
                assert_eq!(events.budget.capture_used(), preparation);
                drop(fill);
                let remaining = events.budget.used();
                let retry = storage_refusal(call.invoke(&mut host, &permit).unwrap_err());
                assert!(std::ptr::eq(
                    std::error::Error::source(&failure).unwrap(),
                    std::error::Error::source(&retry).unwrap(),
                ));
                drop(retry);
                drop(failure);
                assert_eq!(events.budget.used(), remaining);
                assert_eq!(events.budget.capture_used(), preparation);
            }
        }
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        let budget = events.budget.clone();
        drop(call);
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

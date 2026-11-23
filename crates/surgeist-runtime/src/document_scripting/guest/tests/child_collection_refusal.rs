//! A warm child collection must preserve the first native stale-node refusal.

use super::*;
use surgeist_bindings_js::ValueKind;
use surgeist_dom::{DomBatch, tree::DomNodeKind};

struct RetiredCall {
    receiver: Root,
    getter: Root,
    // Keep the original warm collection observable through retirement too.
    _collection: Option<Root>,
}

impl RetiredCall {
    fn fixture(
        descriptor: &str,
        member: &str,
        cached: bool,
    ) -> (DocumentScriptHost, DocumentScriptPermit, Self) {
        let mut host = public_node_host(200_000, 200_000);
        let permit = host.permit().unwrap();
        let (id, call) = host
            .in_document(&permit, |scope| {
                let getter = scope.evaluate(&units(descriptor))?;
                assert!(scope.is_callable(&getter)?, "missing {member} getter");
                let receiver = scope.evaluate(&units("new DocumentFragment()"))?;
                let id = projected_node_id(scope, &receiver)?;
                let collection = if cached {
                    let value = scope.call(&getter, &receiver, &[])?;
                    assert_eq!(scope.kind(&value)?, ValueKind::Object);
                    Some(scope.retain(&value))
                } else {
                    None
                };
                Ok((
                    id,
                    Self {
                        receiver: scope.retain(&receiver),
                        getter: scope.retain(&getter),
                        _collection: collection,
                    },
                ))
            })
            .unwrap();
        host.with_native(|dom, _, _| {
            assert!(matches!(
                dom.snapshot().node(id).unwrap().kind(),
                DomNodeKind::DocumentFragment
            ));
            let mut batch = DomBatch::new();
            batch.destroy(id);
            dom.apply(batch, &AtomicBool::new(false)).unwrap();
            assert!(matches!(dom.snapshot().node(id), Err(DomError::StaleNode)));
        })
        .unwrap();
        fund_guest_operation_after_setup(&host);
        (host, permit, call)
    }

    fn read(
        &self,
        host: &mut DocumentScriptHost,
        permit: &DocumentScriptPermit,
    ) -> DocumentScriptError {
        host.in_document(permit, |scope| {
            let receiver = scope.import(&self.receiver)?;
            let getter = scope.import(&self.getter)?;
            scope.call(&getter, &receiver, &[]).map(drop)
        })
        .expect_err("destroyed DocumentFragment collection receiver must refuse")
    }
}

fn original_cause(failure: &DocumentScriptError) -> &DocumentGuestEventError {
    let DocumentScriptError::Script(Error::HostFailure(failure)) = failure else {
        panic!("expected a typed child collection host refusal: {failure:?}");
    };
    failure.cause::<DocumentGuestEventError>().unwrap()
}

fn stale_node(cause: &DocumentGuestEventError) -> bool {
    matches!(
        cause,
        DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::Dom(DomError::StaleNode)
        ))
    )
}

pub(in crate::document_scripting) fn retired_child_getters_deliver_the_same_first_native_refusal() {
    let mut warm_mismatches = Vec::new();
    for (member, descriptor) in [
        (
            "children",
            "Object.getOwnPropertyDescriptor(DocumentFragment.prototype, 'children')?.get",
        ),
        (
            "childNodes",
            "Object.getOwnPropertyDescriptor(Node.prototype, 'childNodes')?.get",
        ),
    ] {
        let (mut host, permit, call) = RetiredCall::fixture(descriptor, member, false);
        let events = host.guest_events().unwrap();
        let stamp = host.snapshot().unwrap().stamp();
        let retained = events.budget.used() - events.budget.capture_used();
        let before = events.budget.work().unwrap();
        let failure = call.read(&mut host, &permit);
        let exact = events.budget.work().unwrap() - before;
        assert!(
            stale_node(original_cause(&failure)),
            "{member}: {failure:?}"
        );
        assert!(exact > 0, "{member} must admit the attached native read");
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(
            events.budget.used() - events.budget.capture_used(),
            retained
        );
        let budget = events.budget.clone();
        drop((failure, call, events));
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);

        for (cached, sufficient) in [(false, true), (true, true), (false, false), (true, false)] {
            let (mut host, permit, call) = RetiredCall::fixture(descriptor, member, cached);
            let events = host.guest_events().unwrap();
            let stamp = host.snapshot().unwrap().stamp();
            let retained = events.budget.used() - events.budget.capture_used();
            let work = if sufficient { exact } else { exact - 1 };
            events.budget.set_work_remaining(work);
            let failure = call.read(&mut host, &permit);
            let cause = original_cause(&failure);
            if sufficient {
                if stale_node(cause) {
                    assert_eq!(events.budget.work_remaining().unwrap(), 0);
                } else {
                    assert!(
                        cached,
                        "cold {member} exact budget must reach StaleNode: {cause:?}"
                    );
                    warm_mismatches.push(format!(
                        "warm {member}, {work} units: expected StaleNode; got {cause:?}"
                    ));
                }
            } else {
                assert!(
                    matches!(cause, DocumentGuestEventError::WorkLimit),
                    "{member} cached={cached}, {work} units: expected WorkLimit; got {cause:?}"
                );
            }
            let repeated = host
                .in_document::<()>(&permit, |_| {
                    panic!("terminal collection refusal reentered guest code")
                })
                .expect_err("original child collection refusal must remain terminal");
            assert!(std::ptr::eq(cause, original_cause(&repeated)));
            assert_eq!(host.snapshot().unwrap().stamp(), stamp);
            assert_eq!(
                events.budget.used() - events.budget.capture_used(),
                retained
            );
            let budget = events.budget.clone();
            drop((repeated, failure, call, events));
            host.stop().into_session().unwrap();
            assert_eq!(budget.used(), 0);
        }
    }
    assert!(warm_mismatches.is_empty(), "{}", warm_mismatches.join("\n"));
}

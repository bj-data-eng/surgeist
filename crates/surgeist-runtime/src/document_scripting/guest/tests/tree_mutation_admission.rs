use super::*;
use surgeist_dom::DomBatch;

const SCRIPT: &str = r#"(() => {
    const original = new Error('conversion stop');
    try {
        probeReceiver.append('abc', {toString() {
            observeMutationAdmission();
            throw original;
        }});
    } catch (error) { return error === original; }
    return false;
})()"#;

type Observation = Rc<Cell<Option<(usize, u64)>>>;

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit, Observation) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let receiver = batch.create_element(target.document(), "div");
    let receiver = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&receiver)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 200_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, receiver, "probeReceiver");
    let events = Rc::downgrade(&host.guest_events().unwrap());
    let observed = Rc::new(Cell::new(None));
    let marker = observed.clone();
    host.in_document(&permit, |scope| {
        let callback = scope.function(&Callback::new(move |_, _| {
            let events = events.upgrade().unwrap();
            marker.set(Some((events.budget.used(), events.budget.work().unwrap())));
            Ok(Completion::Undefined)
        }))?;
        let global = scope.global()?;
        scope.set(
            &global,
            PropertyKey::String(&units("observeMutationAdmission")),
            &callback,
        )
    })
    .unwrap();
    (host, permit, observed)
}

fn invoke(scope: &mut Scope<'_>) -> Result<(), Error> {
    let actual = scope.evaluate(&units(SCRIPT))?;
    let expected = scope.primitive(Primitive::Boolean(true))?;
    assert!(scope.strict_equals(&actual, &expected)?);
    Ok(())
}

pub(in crate::document_scripting) fn normalized_operands_admit_exact_backing_and_work() {
    let normalized_bytes = 2
        * (size_of::<surgeist_dom::DomNodeOrString>() + size_of::<GuestBacking>())
        + size_of::<Vec<u16>>()
        + 2 * size_of::<usize>()
        + 3 * size_of::<u16>();
    // One buffer reservation, two operand visits, one native text reservation
    // and the exact three UTF16 copy steps. Conversion has its own account.
    let normalized_work = 1 + 2 + 1 + 3;
    let (mut control, permit, observed) = fixture();
    // The outer call captures its receiver, two selected operands and their
    // buffer. The observation callback captures its receiver with no buffer.
    let retention_bytes = size_of::<GuestBacking>() + size_of::<Rc<reports::DispatchFailure>>();
    let root_bytes = control
        .engine
        .as_ref()
        .unwrap()
        .process
        .managed_object_storage_bytes()
        + retention_bytes;
    let capture_bytes = 4 * root_bytes + 2 * size_of::<Value<'_>>() + retention_bytes;
    let bytes = normalized_bytes + capture_bytes;
    let work = normalized_work + 5;
    let events = control.guest_events().unwrap();
    let before = (events.budget.used(), events.budget.work().unwrap());
    let stamp = control.snapshot().unwrap().stamp();
    control.in_document(&permit, invoke).unwrap();
    assert_eq!(observed.get(), Some((before.0 + bytes, before.1 + work)));
    assert_eq!(events.budget.used(), before.0);
    assert_eq!(control.snapshot().unwrap().stamp(), stamp);
    drop((control, events));

    for (available_bytes, available_work, accepted) in [
        (bytes, work, true),
        (bytes - 1, work, false),
        (bytes, work - 1, false),
    ] {
        let (mut host, permit, observed) = fixture();
        let events = host.guest_events().unwrap();
        let spare = events.limits.storage_bytes - events.budget.used() - available_bytes;
        let fill = events.reserve(&host.native, spare).unwrap();
        let before = events.budget.used();
        let stamp = host.snapshot().unwrap().stamp();
        events.budget.set_work_remaining(available_work);
        let result = host.in_document(&permit, invoke);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(events.budget.used(), before);
        if accepted {
            result.unwrap();
            assert_eq!(observed.get().unwrap().0, events.limits.storage_bytes);
            assert_eq!(events.budget.work_remaining().unwrap(), 0);
        } else {
            assert!(
                observed.get().is_none(),
                "refusal must precede the second conversion"
            );
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short normalization must refuse: {result:?}");
            };
            assert!(matches!(
                (
                    available_work < work,
                    failure.cause::<DocumentGuestEventError>()
                ),
                (true, Some(DocumentGuestEventError::WorkLimit))
                    | (
                        false,
                        Some(DocumentGuestEventError::Document(
                            DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
                        ))
                    )
            ));
        }
        drop((fill, events));
        host.stop().into_session().unwrap();
    }
}

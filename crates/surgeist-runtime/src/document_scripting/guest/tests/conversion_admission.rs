use super::*;
use surgeist_dom::DomBatch;

pub(in crate::document_scripting) fn conversion_work_precedes_guest_coercion() {
    // Compilation and evaluation each prepare one outcome record. The setter
    // prepares its callback frame, receiver, argument buffer and argument root.
    let preparation_prefix = 2 + 1 + 1 + 1 + 1;
    // Direct DOMString conversion visits one value, then prepares one outcome
    // record before ToString. This setter has no argument-presence decision.
    let required = preparation_prefix + 1 + 1;
    for (available, reaches_coercion) in [(required - 1, false), (required, true)] {
        let cancellation = Arc::new(AtomicBool::new(false));
        let (mut session, target) =
            crate::document_scripting::cross_realm_tests::hosted_session(&cancellation, 0);
        let mut batch = DomBatch::new();
        let text = batch.create_text(target.document(), "original");
        let text = session
            .dom()
            .apply(batch, &cancellation)
            .unwrap()
            .created(&text)
            .unwrap()
            .unwrap();
        let mut host = attach_public_nodes(session, target, cancellation, 8_000, 200_000);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, text, "conversionText");
        let marker = host
            .in_document(&permit, |scope| {
                let marker = scope.evaluate(&units(
                    "globalThis.conversionMarker = {};\n\
                     globalThis.conversionInput = { toString() { throw conversionMarker; } };\n\
                     conversionMarker",
                ))?;
                Ok(scope.retain(&marker))
            })
            .unwrap();
        let events = host.guest_events().unwrap();
        events.budget.set_work_remaining(available);
        let before = host.snapshot().unwrap().stamp();
        let result = host.in_document(&permit, |scope| {
            scope
                .evaluate(&units("conversionText.data = conversionInput"))
                .map(drop)
        });
        assert_eq!(host.snapshot().unwrap().stamp(), before);
        if let Err(DocumentScriptError::Script(Error::Exception(exception))) = &result {
            // Verify that any unexpected guest exception really came from
            // the reached coercion, using only existing identity imports.
            host.in_document(&permit, |scope| {
                let expected = scope.import(&marker)?;
                let actual = scope.import(exception.value())?;
                assert!(scope.strict_equals(&actual, &expected)?);
                Ok(())
            })
            .unwrap();
        }
        if reaches_coercion {
            assert!(matches!(
                result,
                Err(DocumentScriptError::Script(Error::Exception(_)))
            ));
            assert_eq!(events.budget.work_remaining().unwrap(), 0);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("semantic conversion work escaped the original guest budget: {result:?}");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::WorkLimit)
            ));
            let Err(DocumentScriptError::Script(Error::HostFailure(repeated))) =
                host.in_document(&permit, |_| Ok(()))
            else {
                panic!("the original aggregate refusal must remain terminal");
            };
            assert!(std::ptr::eq(
                failure.cause::<DocumentGuestEventError>().unwrap(),
                repeated.cause::<DocumentGuestEventError>().unwrap(),
            ));
        }
        drop(marker);
        host.stop().into_session().unwrap();
        assert_eq!(events.budget.used(), 0);
    }
}

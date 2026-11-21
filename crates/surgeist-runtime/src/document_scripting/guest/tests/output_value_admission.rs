use super::*;
use crate::{
    document_scripting::cross_realm_tests::hosted_session,
    internal::dom_source::DomSourceDriveError,
};
use surgeist_dom::{DomBatch, identity::NodeId};
use surgeist_html::values::{DocumentString, Namespace};

const TEXT: [u16; 128] = [0xd800; 128];

fn fixture(source_work: u64) -> (DocumentScriptHost, DocumentScriptPermit, NodeId, NodeId) {
    let cancellation = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancellation, 0);
    let document = target.document();
    let mut batch = DomBatch::new();
    let output = batch.create_element_ns(document, Namespace::html(), "output");
    let child = batch.create_text(document, DocumentString::from_utf16(TEXT.to_vec()));
    batch.set_output_value(&output, "");
    batch.set_custom_validity(&output, "prior\r\nmessage");
    batch.insert_before(&child, &output, None);
    let commit = session.dom().apply(batch, &cancellation).unwrap();
    let subject = commit.created(&output).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let mut host = attach_public_nodes(session, target, cancellation, 200_000, source_work);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, subject, "outputProbe");
    (host, permit, subject, child)
}

fn read(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    script: &str,
    expected: &[u16],
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| {
        let result = scope.evaluate(&units(script))?;
        assert_eq!(scope.to_utf16(&result)?.as_slice(), expected);
        Ok(())
    })
}

pub(in crate::document_scripting) fn output_default_override_uses_only_the_reached_read() {
    let (mut sample, permit, _, _) = fixture(200_000);
    let events = sample.guest_events().unwrap();
    let initial_work = events.budget.work().unwrap();
    read(&mut sample, &permit, "outputProbe.defaultValue", &[]).unwrap();
    let required = events.budget.work().unwrap() - initial_work;
    assert!(required > 0);
    read(&mut sample, &permit, "outputProbe.value", &TEXT).unwrap();
    drop(events);
    sample.stop().into_session().unwrap();

    for (work, script, expected, succeeds) in [
        (required, "outputProbe.defaultValue", &[][..], true),
        (required - 1, "outputProbe.defaultValue", &[][..], false),
        (required, "outputProbe.value", &TEXT[..], false),
    ] {
        let (mut host, permit, _, _) = fixture(200_000);
        let events = host.guest_events().unwrap();
        events.budget.set_work_remaining(work);
        let backing = events.budget.used();
        let stamp = host.snapshot().unwrap().stamp();
        let result = read(&mut host, &permit, script, expected);
        if succeeds {
            result.unwrap();
            assert_eq!(events.budget.work_remaining().unwrap(), 0);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("expected the original read refusal for {script}: {result:?}");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::WorkLimit)
            ));
        }
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(events.budget.used(), backing);
        drop(events);
        host.stop().into_session().unwrap();
    }
}

pub(in crate::document_scripting) fn output_edits_preserve_the_first_source_refusal() {
    for script in [
        "outputProbe.value = 'next'",
        "outputProbe.defaultValue = 'next'",
        "outputProbe.setCustomValidity('next')",
    ] {
        let (mut host, permit, subject, child) = fixture(0);
        let stamp = host.snapshot().unwrap().stamp();
        let result = host.in_document(&permit, |scope| scope.evaluate(&units(script)).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("expected the original Output Source refusal for {script}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                DomError::SourceWorkLimit { limit: 0 }
            )))
        ));
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), stamp);
        let node = after.node(subject).unwrap();
        assert_eq!(node.children(), &[child]);
        assert_eq!(
            after.node(child).unwrap().data().unwrap().code_units(),
            &TEXT
        );
        let output = node.as_output().unwrap().state();
        assert_eq!(
            output.default_value_override(),
            Some(&DocumentString::from(""))
        );
        assert_eq!(
            output.custom_validity_message(),
            &DocumentString::from("prior\nmessage")
        );
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

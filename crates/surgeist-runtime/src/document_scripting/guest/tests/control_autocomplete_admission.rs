//! Measure the autocomplete adapter over an already projected native Select.

use super::*;
use surgeist_dom::{DomBatch, DomError, identity::NodeId};
use surgeist_html::values::Namespace;

const NORMALIZED: &str = "section-home shipping home TeL WEBAUTHN";

fn fixture(raw: &str, source_work: u64) -> (DocumentScriptHost, DocumentScriptPermit, NodeId) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let control = batch.create_element_ns(target.document(), Namespace::html(), "select");
    batch.set_attribute_ns(&control, Namespace::none(), "autocomplete", raw);
    let control = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&control)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, source_work);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, control, "autocompleteProbe");
    (host, permit, control)
}

fn observe(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    expected: &str,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| {
        let actual = scope.evaluate(&units("autocompleteProbe.autocomplete"))?;
        assert_eq!(scope.to_utf16(&actual)?, units(expected));
        Ok(())
    })
}

fn observe_work(raw: &str, expected: &str) -> u64 {
    let (mut host, permit, _) = fixture(raw, 200_000);
    let budget = host.guest_events().unwrap().budget.clone();
    let before = budget.work().unwrap();
    observe(&mut host, &permit, expected).unwrap();
    let reached = budget.work().unwrap() - before;
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);
    reached
}

pub(in crate::document_scripting) fn autocomplete_read_stops_after_six_reached_tokens() {
    let prefix = "a b c d e f ";
    let with_suffix = format!("{prefix}{}", "remainder ".repeat(32));
    let required = observe_work(prefix, "");
    assert!(required > observe_work("", ""));
    assert_eq!(observe_work(&with_suffix, ""), required);

    for allowance in [required, required - 1] {
        let (mut host, permit, _) = fixture(&with_suffix, 200_000);
        let budget = host.guest_events().unwrap().budget.clone();
        let initial = budget.used();
        let stamp = host.snapshot().unwrap().stamp();
        budget.set_work_remaining(allowance);
        let result = observe(&mut host, &permit, "");
        if allowance == required {
            result.unwrap();
            assert_eq!(budget.work_remaining().unwrap(), 0);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("expected backing operation work refusal");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::WorkLimit),
            ));
        }
        assert_eq!(budget.used(), initial);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

pub(in crate::document_scripting) fn autocomplete_read_releases_working_text_backing() {
    let raw = "  SECTION-Home\tSHIPPING HOME TeL WEBAUTHN ";
    let (mut host, permit, _) = fixture(raw, 200_000);
    let budget = host.guest_events().unwrap().budget.clone();
    let initial = budget.used();
    let (result, peak) = budget.measure_peak(|| observe(&mut host, &permit, NORMALIZED));
    result.unwrap();
    let working_bytes = peak - initial;
    assert!(working_bytes >= NORMALIZED.encode_utf16().count() * size_of::<u16>());
    assert_eq!(budget.used(), initial);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);

    for bytes in [working_bytes, working_bytes - 1] {
        let (mut host, permit, _) = fixture(raw, 200_000);
        let events = host.guest_events().unwrap();
        let budget = events.budget.clone();
        let unused = events.limits.storage_bytes - budget.used() - bytes;
        let fill = events.reserve(&host.native, unused).unwrap();
        let initial = budget.used();
        let stamp = host.snapshot().unwrap().stamp();
        let result = observe(&mut host, &permit, NORMALIZED);
        if bytes == working_bytes {
            result.unwrap();
            assert_eq!(budget.peak_used(), events.limits.storage_bytes);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("expected one-short autocomplete backing refusal");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::Document(
                    DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit,)
                )),
            ));
        }
        assert_eq!(budget.used(), initial);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        drop(fill);
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

pub(in crate::document_scripting) fn autocomplete_setter_returns_original_source_work_refusal() {
    use crate::internal::dom_source::DomSourceDriveError;

    let (mut host, permit, control) = fixture("name", 0);
    let before = host.snapshot().unwrap();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("autocompleteProbe.autocomplete = 'email'"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected the native Source refusal");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(
            crate::internal::dom_source::DomSourceRuntimeError::Dom(DomError::SourceWorkLimit {
                limit: 0
            },)
        )),
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after
            .node(control)
            .unwrap()
            .attribute(&"autocomplete".into()),
        Some(&"name".into()),
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

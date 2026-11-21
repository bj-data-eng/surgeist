//! Native range reflection bounds, first failure, and temporary backing.

use super::*;
use crate::internal::dom_source::{DomSourceDriveError, DomSourceRuntimeError};
use surgeist_dom::{DomBatch, DomError, identity::NodeId};
use surgeist_html::values::Namespace;

fn fixture(
    tag: &str,
    attributes: &[(&str, &str)],
    source_work: u64,
) -> (DocumentScriptHost, DocumentScriptPermit, NodeId) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let indicator = batch.create_element_ns(target.document(), Namespace::html(), tag);
    for &(name, value) in attributes {
        batch.set_attribute_ns(&indicator, Namespace::none(), name, value);
    }
    let indicator = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&indicator)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, source_work);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, indicator, "rangeProbe");
    (host, permit, indicator)
}

fn read_minimum(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| {
        let value = scope.evaluate(&units("rangeProbe.min"))?;
        let expected = scope.primitive(Primitive::Number(7.5))?;
        assert!(scope.strict_equals(&value, &expected)?);
        Ok(())
    })
}

pub(in crate::document_scripting) fn meter_read_admits_only_its_reached_input() {
    let raw = format!("  {}7.5tail", "0".repeat(32));
    let unrelated = "9".repeat(512);
    let short = [("min", raw.as_str()), ("max", "9")];
    let long = [("min", raw.as_str()), ("max", unrelated.as_str())];
    let (mut host, permit, _) = fixture("meter", &short, 200_000);
    let budget = host.guest_events().unwrap().budget.clone();
    let baseline = budget.used();
    let before = budget.work().unwrap();
    let (result, peak) = budget.measure_peak(|| read_minimum(&mut host, &permit));
    result.unwrap();
    let required = budget.work().unwrap() - before;
    let scratch = peak - baseline;
    // The recovered token contains 32 zeroes and the three units of "7.5".
    assert!(scratch >= 35);
    assert_eq!(budget.used(), baseline);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);

    // An unrelated value must not change the exact work necessary to read min.
    for allowance in [required, required - 1] {
        let (mut host, permit, _) = fixture("meter", &long, 200_000);
        let budget = host.guest_events().unwrap().budget.clone();
        let baseline = budget.used();
        let stamp = host.snapshot().unwrap().stamp();
        budget.set_work_remaining(allowance);
        let result = read_minimum(&mut host, &permit);
        if allowance == required {
            result.unwrap();
            assert_eq!(budget.work_remaining().unwrap(), 0);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("expected one-short native meter work refusal: {result:?}");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::WorkLimit),
            ));
        }
        assert_eq!(budget.used(), baseline);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }

    for allowance in [scratch, scratch - 1] {
        let (mut host, permit, _) = fixture("meter", &short, 200_000);
        let events = host.guest_events().unwrap();
        let budget = events.budget.clone();
        let fill = events
            .reserve(
                &host.native,
                events.limits.storage_bytes - budget.used() - allowance,
            )
            .unwrap();
        let baseline = budget.used();
        let stamp = host.snapshot().unwrap().stamp();
        let (result, peak) = budget.measure_peak(|| read_minimum(&mut host, &permit));
        if allowance == scratch {
            result.unwrap();
            assert_eq!(peak, events.limits.storage_bytes);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("expected one-short native meter scratch refusal: {result:?}");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::Document(
                    DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
                )),
            ));
        }
        assert_eq!(budget.used(), baseline);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        drop(fill);
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

pub(in crate::document_scripting) fn progress_max_preserves_source_noop_and_refusal() {
    let (mut host, permit, indicator) = fixture("progress", &[("max", "7")], 0);
    let before = host.snapshot().unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(scope, "(rangeProbe.max = -0, rangeProbe.max === 7)")
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    let result = host.in_document(&permit, |scope| {
        scope.evaluate(&units("rangeProbe.max = 9")).map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected the original progress Source refusal: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::SourceWorkLimit { limit: 0 }
        ))),
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.node(indicator).unwrap().attribute(&"max".into()),
        Some(&"7".into()),
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

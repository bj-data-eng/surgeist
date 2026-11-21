//! Native integer reflection admission over already projected controls.

use super::*;
use surgeist_dom::{DomBatch, identity::NodeId};
use surgeist_html::values::Namespace;

fn attach_control(
    tag: &str,
    attribute: &str,
    raw: &str,
    source_work: u64,
) -> (DocumentScriptHost, DocumentScriptPermit, NodeId) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let control = batch.create_element_ns(target.document(), Namespace::html(), tag);
    batch.set_attribute_ns(&control, Namespace::none(), attribute, raw);
    let control = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&control)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, source_work);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, control, "numericProbe");
    (host, permit, control)
}

fn observe_seven(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    expression: &str,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| {
        let actual = scope.evaluate(&units(expression))?;
        let expected = scope.primitive(Primitive::Number(7.0))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    })
}

pub(in crate::document_scripting) fn integer_reflections_admit_reached_native_units() {
    const ZEROES: u64 = 32;
    let raw = format!("{}7!", "0".repeat(usize::try_from(ZEROES).unwrap()));
    for (tag, attribute, expression) in [
        ("input", "maxlength", "numericProbe.maxLength"),
        ("textarea", "rows", "numericProbe.rows"),
    ] {
        let (mut host, permit, _) = attach_control(tag, attribute, "7!", 100_000);
        let budget = host.guest_events().unwrap().budget.clone();
        let before = budget.work().unwrap();
        observe_seven(&mut host, &permit, expression).unwrap();
        let short_work = budget.work().unwrap() - before;
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);

        // Each added leading zero is reached in the grammar scan and in
        // integer projection. The native identity and attribute remain fixed.
        let required = short_work + 2 * ZEROES;
        for allowance in [required, required - 1] {
            let (mut host, permit, _) = attach_control(tag, attribute, &raw, 100_000);
            let budget = host.guest_events().unwrap().budget.clone();
            let stamp = host.snapshot().unwrap().stamp();
            let bytes = budget.used();
            budget.set_work_remaining(allowance);
            let result = observe_seven(&mut host, &permit, expression);
            if allowance == required {
                result.unwrap();
                assert_eq!(budget.work_remaining().unwrap(), 0);
            } else {
                let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                    panic!("expected one-short native integer admission for {expression}");
                };
                assert!(matches!(
                    failure.cause::<DocumentGuestEventError>(),
                    Some(DocumentGuestEventError::WorkLimit),
                ));
            }
            assert_eq!(host.snapshot().unwrap().stamp(), stamp);
            assert_eq!(budget.used(), bytes);
            host.stop().into_session().unwrap();
            assert_eq!(budget.used(), 0);
        }
    }
}

pub(in crate::document_scripting) fn local_integer_rejection_precedes_source_admission() {
    use crate::internal::dom_source::DomSourceDriveError;

    for (tag, attribute, rejected, accepted) in [
        (
            "input",
            "maxlength",
            "numericProbe.maxLength = -1",
            "numericProbe.maxLength = 9",
        ),
        (
            "input",
            "size",
            "numericProbe.size = 0",
            "numericProbe.size = 9",
        ),
    ] {
        let (mut host, permit, control) = attach_control(tag, attribute, "7", 0);
        let before = host.snapshot().unwrap();
        let rejection = format!(
            "(() => {{ try {{ {rejected}; }} catch (e) {{ return e instanceof DOMException && e.name === 'IndexSizeError' && e.code === 1; }} return false; }})()"
        );
        host.in_document(&permit, |scope| assert_guest(scope, &rejection))
            .unwrap();
        assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
        let result = host.in_document(&permit, |scope| scope.evaluate(&units(accepted)).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("expected Source admission refusal for {accepted}");
        };
        assert!(matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(
                crate::internal::dom_source::DomSourceRuntimeError::Dom(
                    surgeist_dom::DomError::SourceWorkLimit { limit: 0 },
                )
            )),
        ));
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), before.stamp());
        assert_eq!(
            after.node(control).unwrap().attribute(&attribute.into()),
            Some(&"7".into()),
        );
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

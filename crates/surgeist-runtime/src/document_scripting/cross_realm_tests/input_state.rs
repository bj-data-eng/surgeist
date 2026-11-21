//! Input bindings retain native processing and refusal custody.

use super::*;
use crate::{
    document_scripting::guest::tests::{attach_public_nodes, expose_public_node_for_source_test},
    internal::dom_source::DomSourceDriveError,
};
use surgeist_html::{attributes::HtmlInputType, values::DocumentString};

fn attached_input(
    source_work: u64,
    configure: impl FnOnce(DomLimits) -> DomLimits,
) -> (DocumentScriptHost, DocumentScriptPermit, NodeId) {
    let cancellation = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancellation, 0, 0, configure);
    let mut batch = DomBatch::new();
    let input = batch.create_element_ns(target.document(), Namespace::html(), "input");
    batch.set_input_value(&input, "original");
    let input = session
        .dom()
        .apply(batch, &cancellation)
        .unwrap()
        .created(&input)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancellation, 200_000, source_work);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, input, "inputProbe");
    (host, permit, input)
}

pub(super) fn input_reflections_drive_native_range_and_color_processing() {
    let (mut host, permit, input) = attached_input(20_000_000, |limits| {
        limits
            .with_input_processing(2_000_000, 2_000_000_000)
            .unwrap()
    });
    for (script, attribute, authored, kind, value) in [
        (
            "inputProbe.type = 'range'; inputProbe.min = '0'; inputProbe.max = '10'; inputProbe.step = 'any'; inputProbe.value = '6'; inputProbe.step = '4'",
            "step",
            Some("4"),
            HtmlInputType::Range,
            "8",
        ),
        (
            "inputProbe.step = 'any'; inputProbe.max = '7'",
            "max",
            Some("7"),
            HtmlInputType::Range,
            "7",
        ),
        (
            "inputProbe.value = '2'; inputProbe.min = '5'",
            "min",
            Some("5"),
            HtmlInputType::Range,
            "5",
        ),
        (
            "inputProbe.type = 'color'; inputProbe.alpha = true; inputProbe.value = 'rgb(0 0 0 / 0.5)'; inputProbe.colorSpace = 'DISPLAY-P3'",
            "colorspace",
            Some("DISPLAY-P3"),
            HtmlInputType::Color,
            "color(display-p3 0 0 0 / 0.5)",
        ),
        (
            "inputProbe.alpha = false",
            "alpha",
            None,
            HtmlInputType::Color,
            "color(display-p3 0 0 0)",
        ),
    ] {
        host.in_document(&permit, |scope| evaluate(scope, script).map(drop))
            .unwrap();
        let snapshot = host.snapshot().unwrap();
        let node = snapshot.node(input).unwrap();
        let state = node.as_input().unwrap().state();
        assert_eq!(state.input_type(), kind);
        assert_eq!(
            state.current_value(),
            &DocumentString::from(value),
            "{script}"
        );
        assert_eq!(
            node.attribute(&DocumentString::from(attribute)),
            authored.map(DocumentString::from).as_ref(),
        );
    }
    host.stop().into_session().unwrap();
}

pub(super) fn input_reflections_propagate_native_processing_refusal() {
    for (setup, setter) in [
        ("inputProbe.type = 'range'", "inputProbe.step = '4'"),
        ("inputProbe.type = 'color'", "inputProbe.alpha = true"),
    ] {
        let (mut host, permit, input) = attached_input(200_000, |limits| {
            limits
                .with_input_processing(2_000_000, 2_000_000_000)
                .unwrap()
        });
        host.in_document(&permit, |scope| evaluate(scope, setup).map(drop))
            .unwrap();
        host.with_native(|dom, _, _| {
            let limits = dom.limits().with_input_processing(2_000_000, 0).unwrap();
            dom.raw_mut().set_limits(limits).unwrap();
        })
        .unwrap();
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| evaluate(scope, setter).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("expected a native Input limit error for {setter}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                DomError::InputProcessingWorkLimit { limit: 0 }
            )))
        ));
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), before.stamp());
        assert_eq!(
            after.node(input).unwrap().as_input().unwrap().state(),
            before.node(input).unwrap().as_input().unwrap().state(),
        );
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

pub(super) fn input_bindings_use_configured_native_processing() {
    let (mut host, permit, input) = attached_input(20_000_000, |limits| {
        limits
            .with_input_processing(2_000_000, 2_000_000_000)
            .unwrap()
    });
    for (script, kind, value, email_count) in [
        (
            r#"(() => {
                inputProbe.type = 'range';
                inputProbe.setAttributeNS(null, 'min', '0');
                inputProbe.setAttributeNS(null, 'max', '10');
                inputProbe.setAttributeNS(null, 'step', '4');
                inputProbe.value = '6';
                return inputProbe.type === 'range' && inputProbe.value === '8' ? 'ok' : 'Range tie';
            })()"#,
            HtmlInputType::Range,
            "8",
            None,
        ),
        (
            r#"(() => {
                inputProbe.type = 'color';
                inputProbe.value = 'red';
                return inputProbe.type === 'color' && inputProbe.value === '#ff0000' ? 'ok' : 'Color value';
            })()"#,
            HtmlInputType::Color,
            "#ff0000",
            None,
        ),
        (
            r#"(() => {
                inputProbe.type = 'email';
                inputProbe.multiple = true;
                inputProbe.value = ' a@example.test , , b@example.test ';
                return inputProbe.multiple && inputProbe.value === 'a@example.test,,b@example.test'
                    ? 'ok' : 'multiple Email value';
            })()"#,
            HtmlInputType::Email,
            "a@example.test,,b@example.test",
            Some(3),
        ),
        (
            r#"(() => {
                inputProbe.multiple = false;
                return !inputProbe.multiple && inputProbe.value === 'a@example.test,,b@example.test'
                    ? 'ok' : 'single Email value';
            })()"#,
            HtmlInputType::Email,
            "a@example.test,,b@example.test",
            None,
        ),
    ] {
        host.in_document(&permit, |scope| {
            let result = evaluate(scope, script)?;
            assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
            Ok(())
        })
        .unwrap();
        let snapshot = host.snapshot().unwrap();
        let view = snapshot.node(input).unwrap().as_input().unwrap();
        assert_eq!(view.state().input_type(), kind);
        assert_eq!(view.state().current_value(), &DocumentString::from(value));
        assert_eq!(view.state().email_value_count(), email_count);
    }
    host.stop().into_session().unwrap();
}

pub(super) fn input_source_and_processing_refusals_preserve_native_state() {
    for (work, script, expected) in [
        (
            0,
            "inputProbe.checked = true",
            DomError::SourceWorkLimit { limit: 0 },
        ),
        (
            0,
            "inputProbe.indeterminate = true",
            DomError::SourceWorkLimit { limit: 0 },
        ),
        (
            0,
            "inputProbe.value = 'next'",
            DomError::SourceWorkLimit { limit: 0 },
        ),
        (
            200_000,
            "inputProbe.type = 'range'",
            DomError::InputProcessingWorkLimit { limit: 0 },
        ),
    ] {
        let (mut host, permit, input) = attached_input(work, |limits| limits);
        let before = host.snapshot().unwrap();
        let state = before.node(input).unwrap().as_input().unwrap().state();
        let result = host.in_document(&permit, |scope| evaluate(scope, script).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("expected the original Input refusal for {script}: {result:?}");
        };
        assert!(
            matches!(
                failure.cause::<DomSourceDriveError>(),
                Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(error)))
                    if error == &expected
            ),
            "{script}: {:?}",
            failure.cause::<DomSourceDriveError>()
        );
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), before.stamp());
        assert_eq!(
            after.node(input).unwrap().as_input().unwrap().state(),
            state
        );
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

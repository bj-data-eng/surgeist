use super::*;
use crate::{
    DomElementTaskRuntimeLimits, DomHostedRuntimeLimits, DomSessionLimits,
    DomSourceEventRuntimeLimits, DomStyleRuntimeLimits,
};
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_dom::{
    Dom, DomBatch, DomHostedLimits, DomHostedRecordLimits, DomLimits, DomNameLimits,
};

fn hosted_session(cancel: &AtomicBool) -> (DomSession, DomHostedTopLevel) {
    let dom = Dom::new(
        DomLimits::new(
            128,
            200_000,
            256,
            256,
            256,
            32,
            1_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut session = DomSession::new(
        dom,
        DomSessionLimits {
            source_events: DomSourceEventRuntimeLimits {
                jobs: 0,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 32,
                reports: 32,
                storage_bytes: 16_000_000,
                work: 1_000_000,
                css_input: CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 32,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    16_000_000,
                    16_000_000,
                    1_000_000,
                ),
                tasks: 32,
                storage_bytes: 16_000_000,
                scratch_bytes: 16_000_000,
                work: 1_000_000,
            },
        },
    )
    .unwrap();
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let owner = hosted.create_host(&mut dom, cancel).unwrap();
    let target = hosted.create_top_level(&mut dom, owner, cancel).unwrap();
    (session, target)
}

#[test]
fn an_installed_hook_delegates_its_document_to_a_direct_cross_realm_call() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel);
    let read_name = "readChildCount".encode_utf16().collect::<Vec<_>>();
    let mut host = PreparedDocumentScriptHost::prepare(session, 1_000_000, cancel.clone())
        .unwrap()
        .enable(
            target,
            HtmlScriptingPolicy {
                user_allows: true,
                automation_allows: true,
            },
            DocumentScriptHooks {
                child_count: &read_name,
                detach_source_node_if: &"detachIf".encode_utf16().collect::<Vec<_>>(),
            },
        )
        .unwrap();
    let permit = host.permit().unwrap();
    let hook = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            let callback = scope.get(&global, PropertyKey::String(&read_name))?;
            Ok(scope.retain(&callback))
        })
        .unwrap();

    // Only the test enters another realm through the private Process. Give
    // that realm the actual installed function, not a new native callback.
    let caller = {
        let engine = host.engine.as_mut().unwrap();
        let mut execution = engine.process.execution();
        let caller = execution.create_realm().unwrap();
        execution
            .in_realm(&caller, |scope| {
                assert_ne!(scope.realm_authority()?.identity(), permit.realm);
                let global = scope.global()?;
                let delegate = scope.import(&hook)?;
                scope.set(&global, PropertyKey::String(&read_name), &delegate)?;
                Ok(())
            })
            .unwrap();
        caller
    };

    for expected in 1..=2 {
        let engine = host.engine.as_mut().unwrap();
        engine
            .process
            .execution()
            .in_realm(&caller, |scope| {
                assert_ne!(scope.realm_authority()?.identity(), permit.realm);
                let count =
                    scope.evaluate(&"readChildCount()".encode_utf16().collect::<Vec<_>>())?;
                let value = scope.primitive(Primitive::Number(f64::from(expected)))?;
                assert!(scope.strict_equals(&count, &value)?);
                Ok(())
            })
            .unwrap();
        if expected == 1 {
            host.with_native(|dom, _, _| {
                let mut batch = DomBatch::new();
                let document = target.document().node();
                let comment = batch.create_comment(document, "native child");
                batch.insert_before(&comment, document, None);
                dom.apply(batch, &cancel).unwrap();
            })
            .unwrap();
        }
    }

    drop(caller);
    drop(hook);
    let session = host.stop().into_session().unwrap();
    assert_eq!(
        session
            .snapshot()
            .node(target.document().node())
            .unwrap()
            .children()
            .len(),
        2
    );
}

//! The original native Script Source retains its live settings read and suffix.

use super::*;

const LIMITS: DomSourceLimits =
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 8);

fn body(session: &DomSession, document: DocumentId) -> NodeId {
    let snapshot = session.snapshot();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    snapshot.node(html).unwrap().children()[1]
}

fn apply(host: &mut DocumentScriptHost, edit: DomSourceEdit) -> DomSourceDriveProgress {
    host.with_native(|dom, _, mut source| {
        source
            .apply(
                dom,
                edit,
                LIMITS,
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap()
    })
    .unwrap()
}

fn resume(
    host: &mut DocumentScriptHost,
    operation: DomSourceOperationId,
) -> DomSourceDriveProgress {
    host.with_native(|dom, _, mut source| {
        source
            .resume(
                dom,
                operation,
                LIMITS,
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap()
    })
    .unwrap()
}

fn request(progress: DomSourceDriveProgress, step: DomSourceScriptStep) -> DomSourceScriptRequest {
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Script(request),
        reason: DomSourceUnavailable::Script(reason),
    } = progress
    else {
        panic!("actual Script request required: {progress:?}");
    };
    assert_eq!(request.step(), step);
    assert_eq!(reason, step);
    request
}

fn abandon(host: &mut DocumentScriptHost, pending: DomSourceScriptRequest) {
    host.with_native(|dom, _, source| {
        source
            .cancel_script_preparation(dom, pending.operation(), LIMITS, &cancel())
            .unwrap();
    })
    .unwrap();
    let progress = resume(host, pending.operation());
    assert!(
        matches!(
            progress,
            DomSourceDriveProgress::Complete(ref complete)
                if complete.operation() == pending.operation() && complete.cancellation_requested()
        ),
        "{progress:?}"
    );
}

fn text_and_attribute_keep_the_private_segment() -> DomSourceScriptRequest {
    let (mut session, top) = setup();
    let document = top.document();
    let parent = body(&session, document);
    let mut batch = DomBatch::new();
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    let text = batch.create_text(document, "");
    batch.insert_before(&text, &script, None);
    batch.insert_before(&script, parent, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let script = commit.created(&script).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();

    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let original = host.snapshot().unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute(script, "src", "unavailable.js");
        assert!(matches!(
            dom.apply(batch, &cancel()),
            Err(DomError::ScriptPreparationRequired)
        ));
        assert_eq!(dom.snapshot().stamp(), original.stamp());
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();

    let text_read = request(
        apply(
            &mut host,
            DomSourceEdit::replace_data(text, 0, 0, "globalThis.ran = 1"),
        ),
        DomSourceScriptStep::ReadScripting,
    );
    assert_eq!(text_read.subject(), Some(script));
    assert_eq!(text_read.node_document(), Some(document));
    assert_eq!(text_read.relevant_global(), Some(top.global()));
    assert_eq!(text_read.basis(), original.stamp());
    assert_eq!(
        host.snapshot().unwrap().node(text).unwrap().data(),
        Some(&"".into())
    );
    assert_eq!(
        request(
            resume(&mut host, text_read.operation()),
            DomSourceScriptStep::ReadScripting
        ),
        text_read
    );
    abandon(&mut host, text_read);

    let no_identities = DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 0, 8);
    let pending = host
        .with_native(|dom, _, mut source| {
            let refused = source.apply(
                dom,
                DomSourceEdit::set_attribute(script, "src", "unavailable.js"),
                no_identities,
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            );
            assert!(
                matches!(
                    refused,
                    Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                        DomError::SourceIdentityLimit
                    )))
                ),
                "{refused:?}"
            );
            let pending = dom.pending_source_edit().unwrap();
            assert_eq!(pending.phase(), DomSourcePhase::ScriptPreparation);
            assert!(!pending.has_published());
            pending
        })
        .unwrap();
    let attribute_read = request(
        resume(&mut host, pending.operation()),
        DomSourceScriptStep::ReadScripting,
    );
    assert_eq!(attribute_read.operation(), pending.operation());
    qualify_reissue_limits(&mut host, attribute_read);
    assert_eq!(attribute_read.subject(), Some(script));
    assert_ne!(attribute_read.operation(), text_read.operation());
    assert!(matches!(
        host.service_script_settings(&permit, text_read, LIMITS),
        Err(DocumentScriptError::Native(DocumentScriptNativeError::Dom(
            DomError::InvalidSourceOperation
        )))
    ));
    let observed = host
        .service_script_settings(&permit, attribute_read, LIMITS)
        .unwrap()
        .unwrap();
    assert!(observed.enabled());
    assert_eq!(observed.request(), attribute_read);
    let executable = request(
        resume(&mut host, attribute_read.operation()),
        DomSourceScriptStep::ExecutablePreparation,
    );
    assert_eq!(executable.operation(), attribute_read.operation());
    assert_eq!(
        request(
            resume(&mut host, attribute_read.operation()),
            DomSourceScriptStep::ExecutablePreparation
        ),
        executable
    );
    assert!(matches!(
        host.service_script_settings(&permit, attribute_read, LIMITS),
        Err(DocumentScriptError::Native(DocumentScriptNativeError::Dom(
            DomError::InvalidSourceOperation
        )))
    ));
    assert!(
        host.snapshot()
            .unwrap()
            .node(script)
            .unwrap()
            .attribute(&"src".into())
            .is_none()
    );
    abandon(&mut host, executable);
    assert_eq!(host.snapshot().unwrap().stamp(), original.stamp());
    eval(
        &mut host,
        &permit,
        "typeof globalThis.ran === 'undefined' ? 1 : 0",
        1.0,
    );
    let _session = host.stop().into_session().unwrap();
    text_read
}

fn make_script(batch: &mut DomBatch, document: DocumentId) -> CreatedNode {
    let script = batch.create_element_ns(document, Namespace::html(), "script");
    let text = batch.create_text(document, "globalThis.ran = 1");
    batch.insert_before(&text, &script, None);
    script
}

fn adopted_script_uses_original_settings_and_current_browsing_context(
    foreign: DomSourceScriptRequest,
) {
    let (mut session, a) = setup();
    let (_, b) = create(&mut session);
    let (_, retiring) = create(&mut session);
    let a_body = body(&session, a.document());
    let b_body = body(&session, b.document());

    let mut batch = DomBatch::new();
    let no_context = batch.create_document();
    let root = batch.create_element_ns(&no_context, Namespace::html(), "div");
    batch.insert_before(&root, &no_context, None);
    let a_to_b = make_script(&mut batch, a.document());
    let b_to_a = make_script(&mut batch, b.document());
    let a_to_no_context = make_script(&mut batch, a.document());
    let retired_to_a = make_script(&mut batch, retiring.document());
    batch.adopt(&a_to_b, b.document().node());
    batch.adopt(&b_to_a, a.document().node());
    batch.adopt(&a_to_no_context, &no_context);
    batch.adopt(&retired_to_a, a.document().node());
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let [a_to_b, b_to_a, a_to_no_context, retired_to_a, root] =
        [a_to_b, b_to_a, a_to_no_context, retired_to_a, root]
            .map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let retired = hosted
        .destroy_top_level(&mut dom, retiring.navigable(), &cancel())
        .unwrap();
    let mut finished = false;
    for _ in 0..256 {
        if matches!(
            hosted.run_next(&mut dom, retired.traversable(), &cancel()).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == retired.operation()
        ) {
            finished = true;
            break;
        }
    }
    assert!(finished);

    let mut host = prepare(session, Arc::new(cancel()))
        .enable(a, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    for (script, parent) in [(b_to_a, a_body), (a_to_no_context, root)] {
        let progress = apply(
            &mut host,
            DomSourceEdit::insert_before(script, parent, None),
        );
        assert!(
            matches!(progress, DomSourceDriveProgress::Complete(_)),
            "{progress:?}"
        );
        let snapshot = host.snapshot().unwrap();
        assert!(
            snapshot
                .node(script)
                .unwrap()
                .as_html_script()
                .unwrap()
                .flags()
                .already_started()
        );
    }

    let read = request(
        apply(
            &mut host,
            DomSourceEdit::insert_before(a_to_b, b_body, None),
        ),
        DomSourceScriptStep::ReadScripting,
    );
    assert_eq!(read.relevant_global(), Some(a.global()));
    assert_eq!(read.node_document(), Some(b.document()));
    assert!(matches!(
        host.service_script_settings(&permit, foreign, LIMITS),
        Err(DocumentScriptError::Native(DocumentScriptNativeError::Dom(
            DomError::ForeignOwner
        )))
    ));

    assert!(
        host.service_script_settings(&permit, read, LIMITS)
            .unwrap()
            .unwrap()
            .enabled()
    );
    let executable = request(
        resume(&mut host, read.operation()),
        DomSourceScriptStep::ExecutablePreparation,
    );
    abandon(&mut host, executable);

    let unavailable = request(
        apply(
            &mut host,
            DomSourceEdit::insert_before(retired_to_a, a_body, None),
        ),
        DomSourceScriptStep::ReadScripting,
    );
    assert_eq!(unavailable.relevant_global(), Some(retiring.global()));
    assert_eq!(unavailable.node_document(), Some(a.document()));
    assert_eq!(
        host.service_script_settings(&permit, unavailable, LIMITS)
            .unwrap(),
        None
    );
    assert_eq!(
        request(
            resume(&mut host, unavailable.operation()),
            DomSourceScriptStep::ReadScripting
        ),
        unavailable
    );
    abandon(&mut host, unavailable);
    let _session = host.stop().into_session().unwrap();
}

fn spent(host: &mut DocumentScriptHost) -> u64 {
    host.with_native(|dom, _, _| dom.pending_source_edit().unwrap().spent_work())
        .unwrap()
}

fn qualify_reissue_limits(host: &mut DocumentScriptHost, read: DomSourceScriptRequest) {
    let before = spent(host);
    assert_eq!(request(resume(host, read.operation()), read.step()), read);
    let after = spent(host);
    let work = after.checked_sub(before).unwrap();
    assert!(work > 0);
    let one_short = after + work - 1;
    let attempt = |host: &mut DocumentScriptHost, retained_bytes, work| {
        host.with_native(|dom, _, mut source| {
            let result = source.resume(
                dom,
                read.operation(),
                DomSourceLimits::new(retained_bytes, 1_000_000, work, 128, 4096, 4096, 8),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            );
            match result {
                Ok(result) => {
                    assert_eq!(request(result, read.step()), read);
                    Ok(())
                }
                Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(error))) => Err(error),
                other => panic!("unexpected Source admission: {other:?}"),
            }
        })
        .unwrap()
    };
    assert_eq!(
        attempt(host, 32_000_000, one_short),
        Err(DomError::SourceWorkLimit { limit: one_short }),
    );
    assert_eq!(spent(host), one_short);
    let exact_work = one_short + work;
    assert_eq!(attempt(host, 32_000_000, exact_work), Ok(()));
    assert_eq!(spent(host), exact_work);

    let (mut short, mut enough) = (0, 32_000_000);
    while enough - short > 1 {
        let middle = short + (enough - short) / 2;
        match attempt(host, middle, 10_000_000) {
            Ok(()) => enough = middle,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit,
            }) if limit == middle => short = middle,
            other => panic!("unexpected Script retention cutoff: {other:?}"),
        }
    }
    assert_eq!(
        attempt(host, short, 10_000_000),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            limit: short
        }),
    );
    assert_eq!(attempt(host, enough, 10_000_000), Ok(()));
}

fn published_iframe_prefix_keeps_its_script_tail_after_cancel() {
    let (mut session, top) = setup();
    let document = top.document();
    let parent = body(&session, document);
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let wrapper = batch.create_element_ns(&detached, Namespace::html(), "div");
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let script = make_script(&mut batch, document);
    batch.insert_before(&iframe, &wrapper, None);
    batch.insert_before(&script, &wrapper, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let [wrapper, iframe, script] =
        [wrapper, iframe, script].map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let read = request(
        apply(
            &mut host,
            DomSourceEdit::insert_before(wrapper, parent, None),
        ),
        DomSourceScriptStep::ReadScripting,
    );
    assert_eq!(read.subject(), Some(script));
    let prefix = host.snapshot().unwrap();
    assert_eq!(prefix.node(wrapper).unwrap().parent(), Some(parent));
    assert!(
        prefix
            .node(iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable()
            .unwrap()
            .is_some()
    );
    assert_eq!(prefix.incomplete_source_operation(), Some(read.operation()));
    host.with_native(|dom, _, source| {
        let pending = dom.pending_source_edit().unwrap();
        assert!(pending.has_published());
        source
            .cancel_script_preparation(dom, read.operation(), LIMITS, &cancel())
            .unwrap();
    })
    .unwrap();
    assert_eq!(
        request(
            resume(&mut host, read.operation()),
            DomSourceScriptStep::ReadScripting
        ),
        read
    );
    assert!(
        host.service_script_settings(&permit, read, LIMITS)
            .unwrap()
            .unwrap()
            .enabled()
    );
    let frontier = request(
        resume(&mut host, read.operation()),
        DomSourceScriptStep::ExecutablePreparation,
    );
    assert_eq!(frontier.operation(), read.operation());
    assert_eq!(host.snapshot().unwrap().stamp(), prefix.stamp());
    let mut original = host.stop().into_session().unwrap();
    let status = original.dom().pending_source_edit().unwrap();
    assert_eq!(status.operation(), read.operation());
    assert!(status.has_published());
    assert!(status.cancellation_requested());
    assert!(matches!(
        original.try_shutdown(),
        DomShutdownOutcome::Pending { .. }
    ));
}

pub(super) fn stopped_host_resumes_original_settings_read() {
    use surgeist_html::elements::script::{HtmlScriptKind, HtmlScriptResult};

    let (mut session, top) = setup();
    let document = top.document();
    let body = body(&session, document);
    let mut batch = DomBatch::new();
    let script = make_script(&mut batch, document);
    let script = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&script)
        .unwrap()
        .unwrap();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let queued_calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, queued_calls.clone());
    let read = request(
        apply(&mut host, DomSourceEdit::insert_before(script, body, None)),
        DomSourceScriptStep::ReadScripting,
    );
    assert_eq!(read.relevant_global(), Some(top.global()));

    let mut session = host.stop().into_session().unwrap();
    assert_eq!(queued_calls.load(Ordering::Relaxed), 0);
    assert_eq!(Arc::strong_count(&queued_calls), 1);
    let stopped = session.snapshot();
    assert_eq!(stopped.stamp(), read.basis());
    assert_eq!(stopped.node(script).unwrap().parent(), None);
    let mut source = session.source_execution();
    let progress = source
        .resume(
            &mut session.dom(),
            read.operation(),
            LIMITS,
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(complete) = progress else {
        panic!("the stopped owner must resolve the original Script read: {progress:?}");
    };
    assert_eq!(complete.operation(), read.operation());
    assert!(session.dom().pending_source_edit().is_none());
    let final_snapshot = session.snapshot();
    let script = final_snapshot.node(script).unwrap();
    assert_eq!(script.parent(), Some(body));
    let prepared = script.as_html_script().unwrap();
    assert!(prepared.flags().already_started());
    assert_eq!(prepared.preparation_document(), Some(document));
    assert_eq!(prepared.script_type(), Some(HtmlScriptKind::Classic));
    assert_eq!(prepared.result(), HtmlScriptResult::Uninitialized);
}

pub(super) fn stopped_host_preserves_executable_frontier() {
    let (mut session, top) = setup();
    let document = top.document();
    let body = body(&session, document);
    let mut batch = DomBatch::new();
    let script = make_script(&mut batch, document);
    let script = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&script)
        .unwrap()
        .unwrap();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let read = request(
        apply(&mut host, DomSourceEdit::insert_before(script, body, None)),
        DomSourceScriptStep::ReadScripting,
    );
    assert!(
        host.service_script_settings(&permit, read, LIMITS)
            .unwrap()
            .unwrap()
            .enabled()
    );
    let executable = request(
        resume(&mut host, read.operation()),
        DomSourceScriptStep::ExecutablePreparation,
    );
    assert_eq!(executable.operation(), read.operation());

    let mut session = host.stop().into_session().unwrap();
    let mut source = session.source_execution();
    let resumed = source
        .resume(
            &mut session.dom(),
            executable.operation(),
            LIMITS,
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        request(resumed, DomSourceScriptStep::ExecutablePreparation),
        executable
    );
    let status = session.dom().pending_source_edit().unwrap();
    assert_eq!(status.operation(), executable.operation());
    assert!(!status.has_published());
    let snapshot = session.snapshot();
    assert_eq!(snapshot.stamp(), read.basis());
    assert_eq!(snapshot.node(script).unwrap().parent(), None);
}

pub(super) fn stopped_script_precedes_replacement_child_service() {
    use surgeist_html::elements::script::{HtmlScriptKind, HtmlScriptResult};
    use surgeist_html_semantics::{
        HtmlBaseFactsRevision, HtmlBaseParseObservation, HtmlBaseParseOutcome, HtmlBaseProducerId,
    };

    const SRC: &str = "http://[";
    let (mut session, top) = setup();
    let document = top.document();
    let parent = body(&session, document);
    let mut batch = DomBatch::new();
    let old = batch.create_element_ns(document, Namespace::html(), "span");
    let after = batch.create_element_ns(document, Namespace::html(), "span");
    let wrapper = batch.create_element_ns(document, Namespace::html(), "div");
    let script = make_script(&mut batch, document);
    let detached = batch.create_html_document(document, None);
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    batch.set_attribute(&iframe, "src", SRC);
    batch.set_attribute(&iframe, "allow", "fullscreen *");
    batch.insert_before(&old, parent, None);
    batch.insert_before(&after, parent, None);
    batch.insert_before(&script, &wrapper, None);
    batch.insert_before(&iframe, &wrapper, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let [old, after, wrapper, script, iframe] = [old, after, wrapper, script, iframe]
        .map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let before = host.snapshot().unwrap();
    let read = request(
        apply(
            &mut host,
            DomSourceEdit::replace_child(wrapper, parent, old),
        ),
        DomSourceScriptStep::ReadScripting,
    );
    assert_eq!(read.subject(), Some(script));
    assert_eq!(read.relevant_global(), Some(top.global()));
    assert_eq!(read.basis(), before.stamp());
    let parked = host.snapshot().unwrap();
    assert_eq!(parked.stamp(), before.stamp());
    assert_eq!(parked.node(parent).unwrap().children(), [old, after]);
    assert_eq!(parked.node(wrapper).unwrap().parent(), None);
    assert_eq!(parked.node(wrapper).unwrap().children(), [script, iframe]);
    assert_eq!(
        parked.node(iframe).unwrap().owner_document(),
        Some(document)
    );

    let mut session = host.stop().into_session().unwrap();
    assert_eq!(session.snapshot().stamp(), read.basis());
    let operation = read.operation();
    let resume_without_url = |session: &mut DomSession| {
        let reached = session.source_execution().resume(
            &mut session.dom(),
            operation,
            LIMITS,
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        );
        let Ok(DomSourceDriveProgress::Pending {
            request: DomSourceRequest::HostedChild(child),
            reason: DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin),
        }) = reached
        else {
            panic!("replacement must reach the original iframe service after stop: {reached:?}");
        };
        child
    };
    let child = resume_without_url(&mut session);
    assert_eq!(child.operation(), operation);
    assert_eq!(child.container(), iframe);
    assert_eq!(child.parent_document().document(), document);
    assert_eq!(child.parent_global(), top.global());
    assert_eq!(child.parent_navigable(), top.navigable());
    let published = session.snapshot();
    assert_eq!(published.incomplete_source_operation(), Some(operation));
    assert_eq!(published.node(parent).unwrap().children(), [wrapper, after]);
    assert_eq!(published.node(old).unwrap().parent(), None);
    assert_eq!(
        published.node(wrapper).unwrap().children(),
        [script, iframe]
    );
    let prepared = published.node(script).unwrap().as_html_script().unwrap();
    assert!(prepared.flags().already_started());
    assert_eq!(prepared.preparation_document(), Some(document));
    assert_eq!(prepared.script_type(), Some(HtmlScriptKind::Classic));
    assert_eq!(prepared.result(), HtmlScriptResult::Uninitialized);
    assert_eq!(resume_without_url(&mut session), child);
    assert_eq!(session.snapshot().stamp(), published.stamp());

    let facts = published
        .document(document)
        .unwrap()
        .base_document_facts()
        .unwrap();
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1);
    let mut stages = Vec::new();
    let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
        assert_eq!(request.operation(), operation);
        assert_eq!(request.embedder(), iframe);
        assert_eq!(request.raw(), &DocumentString::from(SRC));
        stages.push(request.stage());
        let failure = HtmlBaseParseObservation::new(
            SRC.into(),
            facts.document_url().unwrap().clone(),
            facts.encoding().unwrap().clone(),
            revision.clone(),
            HtmlBaseParseOutcome::Failure,
        )
        .unwrap();
        Ok(Some(request.observe(failure, None).unwrap()))
    };
    let complete = session
        .source_execution()
        .resume(
            &mut session.dom(),
            operation,
            LIMITS,
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: Some(DomSourceUrlService {
                    revision: &revision,
                    observe: &mut observe,
                }),
            },
            &cancel(),
        )
        .unwrap();
    assert!(
        matches!(complete, DomSourceDriveProgress::Complete(done) if done.operation() == operation)
    );
    assert_eq!(
        stages,
        [
            DomSourceUrlStage::IframeDeclaredOrigin,
            DomSourceUrlStage::FrameIframeSrc
        ]
    );
    assert!(session.dom().pending_source_edit().is_none());
    let final_snapshot = session.snapshot();
    assert_eq!(
        final_snapshot.node(parent).unwrap().children(),
        [wrapper, after]
    );
    assert_eq!(
        final_snapshot.node(wrapper).unwrap().children(),
        [script, iframe]
    );
    let navigable = final_snapshot
        .node(iframe)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    assert_eq!(
        final_snapshot.hosted_navigable(navigable).unwrap().parent(),
        Some(top.navigable())
    );
}

pub(super) fn run() {
    let foreign = text_and_attribute_keep_the_private_segment();
    adopted_script_uses_original_settings_and_current_browsing_context(foreign);
    published_iframe_prefix_keeps_its_script_tail_after_cancel();
    stopped_host_resumes_original_settings_read();
    stopped_host_preserves_executable_frontier();
    stopped_script_precedes_replacement_child_service();
}

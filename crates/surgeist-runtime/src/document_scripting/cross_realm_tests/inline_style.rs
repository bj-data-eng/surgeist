use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use crate::{DomSourceDriveError, DomStyleRuntimeError};
use std::sync::atomic::Ordering;
use surgeist_cssom::CssomStore;
use surgeist_dom::DomStyleCssomRoot;
use surgeist_html::values::DocumentString;

pub(super) fn guest_insertion_publishes_a_sheet_under_the_known_empty_policy() {
    guest_insertion(None, None, true);
}

pub(super) fn style_runtime_refusal_preserves_the_original_guest_edit() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let document = target.document();
    let body = session
        .snapshot()
        .node(
            session
                .snapshot()
                .document(document)
                .unwrap()
                .document_element()
                .unwrap(),
        )
        .unwrap()
        .children()[1];
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, "p { color: red }");
    batch.insert_before(&text, &style, None);
    let style = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let mut styles = session.style_execution();
    let mut limits = DomStyleRuntimeLimits {
        roots: 4,
        completions: 32,
        reports: 32,
        storage_bytes: 16_000_000,
        work: 4,
        css_input: CssUtf16InputLimits::default(),
    };
    styles.set_limits(&session.dom(), limits, &cancel).unwrap();
    let mut host = attach_public_nodes(session, target, cancel.clone(), 5_000, 60_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, body, "styleParent");
    expose_public_node_for_source_test(&mut host, &permit, style, "inlineStyle");
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = host
        .in_document(&permit, |scope| {
            evaluate(scope, "styleParent.appendChild(inlineStyle)").map(drop)
        })
    else {
        panic!("the Style Runtime work gate must refuse");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Style(
            DomStyleRuntimeError::Admission
        )))
    ));
    let parked = host
        .with_native(|dom, _, _| dom.pending_source_edit().unwrap())
        .unwrap();
    assert!(parked.has_published());
    assert_eq!(
        host.snapshot().unwrap().node(style).unwrap().parent(),
        Some(body)
    );
    limits.work = 1_000_000;
    host.with_native(|dom, _, _| {
        styles.set_limits(dom, limits, &cancel).unwrap();
        styles
            .bind_root(
                dom,
                "refused-style".into(),
                &mut Some(DomStyleCssomRoot::new(
                    document.node(),
                    CssomStore::new(Default::default(), Default::default()).unwrap(),
                )),
                &cancel,
            )
            .unwrap();
    })
    .unwrap();
    cancel.store(true, Ordering::Relaxed);
    let cancelled = host.resume_pending_style_edit(&permit, parked.operation(), SOURCE);
    assert!(
        matches!(
            cancelled,
            Ok(Err(DomSourceDriveError::Source(
                DomSourceRuntimeError::Dom(DomError::Cancelled)
            ))) | Ok(Err(DomSourceDriveError::Source(
                DomSourceRuntimeError::Style(DomStyleRuntimeError::Dom(DomError::Cancelled))
            )))
        ),
        "{cancelled:?}"
    );
    cancel.store(false, Ordering::Relaxed);
    assert_eq!(
        host.with_native(|dom, _, _| dom.pending_source_edit())
            .unwrap(),
        Some(parked)
    );
    assert!(matches!(
        host.resume_pending_style_edit(&permit, parked.operation(), SOURCE).unwrap().unwrap(),
        DomSourceDriveProgress::Complete(done) if done.operation() == parked.operation()
    ));
    let snapshot = host.snapshot().unwrap();
    assert!(
        snapshot
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_some()
    );
}

pub(super) fn guest_style_preserves_foreign_and_unavailable_policy() {
    for select_other_processor in [false, true] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
            limits.with_meta_processing_work(1_000_000)
        });
        let document = target.document();
        let snapshot = session.snapshot();
        let html = snapshot
            .document(document)
            .unwrap()
            .document_element()
            .unwrap();
        let head = snapshot.node(html).unwrap().children()[0];
        let body = snapshot.node(html).unwrap().children()[1];
        let mut batch = DomBatch::new();
        let meta = batch.create_element_ns(document, Namespace::html(), "meta");
        batch.set_attribute(&meta, "http-equiv", "content-security-policy");
        batch.set_attribute(&meta, "content", "style-src 'unsafe-inline'");
        let style = batch.create_element_ns(document, Namespace::html(), "style");
        let text = batch.create_text(document, "p { color: red }");
        batch.insert_before(&text, &style, None);
        let commit = session.dom().apply(batch, &cancel).unwrap();
        let meta = commit.created(&meta).unwrap().unwrap();
        let style = commit.created(&style).unwrap().unwrap();
        session.enable_trusted_types_csp(&cancel).unwrap();
        install_policy_before_style_retry(&mut session, meta, head, &cancel);
        let original_processor = session.trusted_types_csp.take().unwrap();
        if select_other_processor {
            session.enable_trusted_types_csp(&cancel).unwrap();
        }
        let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, body, "styleParent");
        expose_public_node_for_source_test(&mut host, &permit, style, "inlineStyle");
        let result = host.in_document(&permit, |scope| {
            evaluate(scope, "styleParent.appendChild(inlineStyle)").map(drop)
        });
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("unqualified policy must keep the edit pending: {result:?}");
        };
        let original = host
            .with_native(|dom, _, _| dom.pending_source_edit().unwrap())
            .unwrap();
        assert!(original.has_published());
        let before = host.snapshot().unwrap();
        assert_eq!(before.node(style).unwrap().parent(), Some(body));
        let retried = host
            .resume_pending_style_edit(&permit, original.operation(), SOURCE)
            .unwrap();
        if select_other_processor {
            assert!(matches!(
                failure.cause::<DomSourceDriveError>(),
                Some(DomSourceDriveError::Source(DomSourceRuntimeError::Style(
                    DomStyleRuntimeError::Protocol
                )))
            ));
            assert!(
                matches!(
                    retried,
                    Err(DomSourceDriveError::Source(DomSourceRuntimeError::Style(
                        DomStyleRuntimeError::Protocol
                    )))
                ),
                "{retried:?}"
            );
        } else {
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::SourcePending(operation)) if *operation == original.operation()
            ));
            assert!(
                matches!(
                    retried,
                    Ok(DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Style(ref request),
                        reason: DomSourceUnavailable::Style(crate::DomStyleUnavailable::InlinePolicy),
                    }) if request.source().operation() == original.operation()
                ),
                "{retried:?}"
            );
        }
        assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
        crate::document_scripting::guest::tests::assert_queued_trusted_types_violations(&host, &[]);
        assert!(host.stop().into_session().is_err());
        drop(original_processor);
    }
}

pub(super) fn character_data_edit_in_a_native_listener_settles_style_before_return() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target, dialog) = hosted_element(&cancel, "dialog");
    let document = target.document();
    let body = session.snapshot().node(dialog).unwrap().parent().unwrap();
    let mut styles = session.style_execution();
    styles
        .bind_root(
            &session.dom(),
            "reentrant-inline-style".into(),
            &mut Some(DomStyleCssomRoot::new(
                document.node(),
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &cancel,
        )
        .unwrap();
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, "p { color: red }");
    batch.insert_before(&text, &style, None);
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let mut host = attach_public_nodes(session, target, cancel.clone(), 5_000, 60_000);
    let permit = host.permit().unwrap();
    for (node, name) in [
        (dialog, "dialog"),
        (body, "styleParent"),
        (style, "inlineStyle"),
        (text, "styleText"),
    ] {
        expose_public_node_for_source_test(&mut host, &permit, node, name);
    }
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            r#"
        styleParent.appendChild(inlineStyle);
        globalThis.changedDuringListener = false;
        dialog.addEventListener('cancel', event => {
            styleText.replaceData(11, 3, 'blue');
            changedDuringListener = styleText.data === 'p { color: blue }';
            event.preventDefault();
        });
    "#,
        )
        .map(drop)
    })
    .unwrap();
    let snapshot = host.snapshot().unwrap();
    let before = snapshot
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap()
        .id()
        .clone();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let request = start_attached_cancel(&mut host, dialog, &mut services, &cancel);
    let result = host.service_attached_source_event(&permit, &request, EVENT);
    if let Err(DocumentScriptEventError::Access(DocumentScriptError::Script(Error::HostFailure(
        failure,
    )))) = &result
    {
        panic!(
            "the nested CharacterData edit refused: {:?}",
            failure.cause::<DocumentGuestEventError>()
        );
    }
    let DomSourceEventServiceProgress::Serviced { receipt } = result.unwrap() else {
        panic!("the original cancel listener must settle");
    };
    assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
    assert!(receipt.default_prevented());
    let cssom = styles.snapshot(document.node()).unwrap().unwrap();
    let [after] = cssom.sheets() else {
        panic!("the nested Style edit must leave one associated sheet");
    };
    assert_ne!(after, &before);
    host.with_native(|dom, _, mut source| {
        assert!(matches!(
            source.resume(dom, request.operation(), SOURCE, &mut services, &cancel).unwrap(),
            DomSourceDriveProgress::Complete(done) if done.operation() == request.operation()
        ));
    })
    .unwrap();
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap()
            .id(),
        after
    );
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "changedDuringListener")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
}

pub(super) fn guest_insertion_uses_the_effective_meta_style_directive() {
    guest_insertion(
        Some("default-src 'none'; style-src-elem 'unsafe-inline'"),
        None,
        true,
    );
    guest_insertion(
        Some("style-src 'unsafe-inline'; style-src-elem 'none'"),
        None,
        false,
    );
}

pub(super) fn guest_style_update_matches_the_exact_nonce() {
    let policy = Some("style-src 'unsafe-inline' 'nonce-AbC'");
    guest_insertion(policy, Some(("AbC", false)), true);
    guest_insertion(policy, Some(("abc", false)), false);
    guest_insertion(policy, Some(("AbC", true)), false);
    guest_insertion(Some("style-src 'unsafe-inline' 'nonce-='"), None, true);
}

pub(super) fn guest_style_violations_retarget_at_dispatch() {
    for report in [
        StyleReport::Connected,
        StyleReport::Detached,
        StyleReport::Stop,
        StyleReport::Retry,
        StyleReport::RetryStopped,
    ] {
        let policy = if matches!(report, StyleReport::Retry | StyleReport::RetryStopped) {
            format!(
                "default-src 'none' 'report-sample'; extension {}",
                "x".repeat(6_000)
            )
        } else {
            "default-src 'none' 'report-sample'".to_owned()
        };
        guest_style(
            Some(&policy),
            None,
            "p { color: red }".into(),
            false,
            false,
            Some(report),
        );
    }
}

#[derive(Clone, Copy)]
enum StyleReport {
    Connected,
    Detached,
    Stop,
    Retry,
    RetryStopped,
}

fn guest_insertion(policy: Option<&str>, nonce: Option<(&str, bool)>, expected_sheet: bool) {
    guest_style(
        policy,
        nonce,
        "p { color: red }".into(),
        expected_sheet,
        false,
        None,
    );
}

pub(super) fn guest_style_hashes_exact_utf16_source() {
    let text: DocumentString = "p { color: red }".into();
    for policy in [
        "style-src 'sha256-ngewhhP73WDIbgwseeu52VAAJgKdGUsu1IUQQsAm8m4='",
        "style-src 'sha384-ZpslxsY6hfdNIsLGwaQZKfU02y2K0sdrZNXJHB4s9COv3KEMfNvwaUs_zHVcjxej'",
        "style-src 'sha512-/N6GXxQAO2l9ump/adcHcT/yWxws7as1i5T93IlKmYXjD6shH1GqcykwLwi1Ob0uUTyxFBrXFgwg/Zk5WuqePg=='",
    ] {
        guest_style(Some(policy), None, text.clone(), true, true, None);
    }
    let mut units: Vec<u16> = "p::before { content: \"é😀".encode_utf16().collect();
    units.push(0xd800);
    units.extend("\" }".encode_utf16());
    guest_style(
        Some("style-src 'sha256-5H3f8oP6R3NLYXB-VcyFFpkTiKnnt2oYSNZyH_2SfT0='"),
        None,
        DocumentString::from_utf16(units),
        true,
        false,
        None,
    );
    guest_style(
        Some("style-src 'unsafe-inline' 'sha256-ngewhhP73WDIbgwseeu52VAAJgKdGUsu1IUQQsAm8m4'"),
        None,
        text,
        false,
        false,
        None,
    );
}

fn guest_style(
    policy: Option<&str>,
    nonce: Option<(&str, bool)>,
    text: DocumentString,
    expected_sheet: bool,
    edit_after_match: bool,
    report: Option<StyleReport>,
) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits.with_meta_processing_work(1_000_000)
    });
    let document = target.document();
    let mut styles = session.style_execution();
    styles
        .bind_root(
            &session.dom(),
            "guest-inline-style".into(),
            &mut Some(DomStyleCssomRoot::new(
                document.node(),
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &cancel,
        )
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(html).unwrap().children()[1];
    let head = snapshot.node(html).unwrap().children()[0];
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, text);
    batch.insert_before(&text, &style, None);
    let retry = matches!(report, Some(StyleReport::Retry | StyleReport::RetryStopped));
    let retry_tail = retry.then(|| {
        let fragment = batch.create_fragment(document);
        let meta = batch.create_element_ns(document, Namespace::html(), "meta");
        batch.set_attribute(&meta, "http-equiv", "content-security-policy");
        batch.set_attribute(&meta, "content", "trusted-types follow-up");
        batch.insert_before(&style, &fragment, None);
        batch.insert_before(&meta, &fragment, None);
        (fragment, meta)
    });
    let meta = policy.map(|policy| {
        let meta = batch.create_element_ns(document, Namespace::html(), "meta");
        batch.set_attribute(&meta, "http-equiv", "content-security-policy");
        batch.set_attribute(&meta, "content", policy);
        meta
    });
    let committed = session.dom().apply(batch, &cancel).unwrap();
    let style = committed.created(&style).unwrap().unwrap();
    let text = committed.created(&text).unwrap().unwrap();
    let meta = meta.map(|meta| committed.created(&meta).unwrap().unwrap());
    let retry_tail = retry_tail.map(|(fragment, meta)| {
        (
            committed.created(&fragment).unwrap().unwrap(),
            committed.created(&meta).unwrap().unwrap(),
        )
    });
    if meta.is_some() {
        session.enable_trusted_types_csp(&cancel).unwrap();
    }
    let meta = if let Some(meta) = meta.filter(|_| retry) {
        install_policy_before_style_retry(&mut session, meta, head, &cancel);
        None
    } else {
        meta
    };
    if let Some((value, true)) = nonce {
        let mut slot = DomBatch::new();
        slot.set_nonce(style, value);
        session.dom().apply(slot, &cancel).unwrap();
    }
    let source_work = if retry { 30_000 } else { 60_000 };
    let mut host = attach_public_nodes(session, target, cancel, 5_000, source_work);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, body, "styleParent");
    expose_public_node_for_source_test(&mut host, &permit, style, "inlineStyle");
    if let Some((fragment, _)) = retry_tail {
        expose_public_node_for_source_test(&mut host, &permit, head, "styleHead");
        expose_public_node_for_source_test(&mut host, &permit, fragment, "styleFragment");
    }
    if report.is_some() {
        host.in_document(&permit, |scope| {
            evaluate(scope, r#"
                globalThis.styleViolations = [];
                document.addEventListener('securitypolicyviolation', event => styleViolations.push(event));
            "#).map(drop)
        }).unwrap();
    }
    if let Some((value, slot_only)) = nonce {
        host.in_document(&permit, |scope| {
            evaluate(scope, "styleParent.appendChild(inlineStyle)")?;
            if !slot_only {
                let script = format!("inlineStyle.setAttribute('nonce', '{value}')");
                evaluate(scope, &script)?;
            }
            Ok(())
        })
        .unwrap();
    }
    if let Some(meta) = meta {
        expose_public_node_for_source_test(&mut host, &permit, head, "styleHead");
        expose_public_node_for_source_test(&mut host, &permit, meta, "styleMeta");
        host.in_document(&permit, |scope| {
            evaluate(scope, "styleHead.appendChild(styleMeta)").map(drop)
        })
        .unwrap();
    }
    let inserted = host.in_document(&permit, |scope| {
        let script = if nonce.is_some() {
            "inlineStyle.textContent = 'p { color: blue }'; inlineStyle.textContent === 'p { color: blue }'"
        } else if retry {
            "styleHead.appendChild(styleFragment) === styleFragment"
        } else {
            "styleParent.appendChild(inlineStyle) === inlineStyle"
        };
        let result = evaluate(scope, script)?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&result, &expected)?);
        Ok(())
    });
    if retry {
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = inserted else {
            panic!("expected a metered Style refusal, got {inserted:?}");
        };
        let cause = failure.cause::<DomSourceDriveError>();
        assert!(
            matches!(
                cause,
                Some(DomSourceDriveError::Source(DomSourceRuntimeError::Style(
                    DomStyleRuntimeError::Dom(DomError::SourceWorkLimit { limit })
                ))) if *limit == source_work
            ),
            "{cause:?}"
        );
        let original = host
            .with_native(|dom, _, _| dom.pending_source_edit().unwrap())
            .unwrap();
        assert!(original.has_published());
        assert_eq!(original.spent_work(), source_work);
        crate::document_scripting::guest::tests::assert_queued_trusted_types_violations(&host, &[]);
        if matches!(report, Some(StyleReport::RetryStopped)) {
            let mut stopped = host
                .stop()
                .into_session()
                .expect_err("the stopped host must retain the original Style continuation");
            assert!(matches!(
                stopped.settle_pending_style_edit(original.operation(), SOURCE).unwrap().unwrap(),
                DomSourceDriveProgress::Complete(done) if done.operation() == original.operation()
            ));
            let mut session = stopped.into_session().unwrap();
            assert!(session.dom().pending_source_edit().is_none());
            let snapshot = session.snapshot();
            assert_eq!(snapshot.node(style).unwrap().parent(), Some(head));
            assert!(
                styles
                    .snapshot(document.node())
                    .unwrap()
                    .unwrap()
                    .sheets()
                    .is_empty()
            );
            return;
        }
        assert!(matches!(
            host.resume_pending_style_edit(&permit, original.operation(), SOURCE).unwrap().unwrap(),
            DomSourceDriveProgress::Complete(done) if done.operation() == original.operation()
        ));
        assert_queued_style_violation(&host);
        let (fragment, tail) = retry_tail.unwrap();
        let snapshot = host.snapshot().unwrap();
        assert!(snapshot.node(fragment).unwrap().children().is_empty());
        assert_eq!(snapshot.node(head).unwrap().children()[1..], [style, tail]);
        let session = host.native.session.borrow();
        let processor = session
            .trusted_types_csp
            .as_ref()
            .unwrap()
            .processor
            .borrow();
        let document = snapshot.document(document).unwrap();
        let policies = processor
            .document_policies_with(document.metadata().policy_container().csp(), &mut |_| {
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(policies.len(), 2);
        assert_eq!(policies[1].serialized(), &"trusted-types follow-up".into());
        drop(processor);
        drop(session);
        assert!(matches!(
            host.resume_pending_style_edit(&permit, original.operation(), SOURCE),
            Err(DocumentScriptNativeError::SourceAccess)
        ));
    } else if inserted.is_err() {
        let pending = host
            .with_native(|dom, _, mut source| {
                dom.pending_source_edit().map(|status| {
                    let progress = source.resume(
                        dom,
                        status.operation(),
                        DomSourceLimits::new(16_000_000, 200_000, 60_000, 64, 64, 64, 2),
                        &mut DomSourceServices {
                            meta: None,
                            execution_scope: None,
                            style_policy: &mut |_| None,
                            url: None,
                        },
                        &AtomicBool::new(false),
                    );
                    match progress {
                        Ok(DomSourceDriveProgress::Pending { reason, .. }) => format!("{reason:?}"),
                        other => format!("{other:?}"),
                    }
                })
            })
            .unwrap();
        panic!("guest Style insertion refused: {inserted:?}; original Source: {pending:?}");
    }
    let snapshot = host.snapshot().unwrap();
    let node = snapshot.node(style).unwrap();
    assert_eq!(node.parent(), Some(if retry { head } else { body }));
    let sheet = node.as_style().unwrap().sheet().unwrap();
    let cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert_eq!(sheet.is_some(), expected_sheet);
    if let Some(sheet) = sheet {
        assert_eq!(cssom.sheets(), &[sheet.id().clone()]);
        assert_eq!(cssom.sheet_rules(sheet.id()).unwrap().len(), 1);
    } else {
        assert!(cssom.sheets().is_empty());
    }
    if edit_after_match {
        expose_public_node_for_source_test(&mut host, &permit, text, "inlineStyleText");
        host.in_document(&permit, |scope| {
            evaluate(scope, "inlineStyleText.replaceData(11, 3, 'blue')").map(drop)
        })
        .unwrap();
        let snapshot = host.snapshot().unwrap();
        assert!(
            snapshot
                .node(style)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap()
                .is_none()
        );
        assert!(
            styles
                .snapshot(document.node())
                .unwrap()
                .unwrap()
                .sheets()
                .is_empty()
        );
    }
    if let Some(report) = report {
        if matches!(report, StyleReport::Retry) {
            return;
        }
        let detached = matches!(report, StyleReport::Detached);
        host.in_document(&permit, |scope| {
            let count = evaluate(scope, "styleViolations.length")?;
            let zero = scope.primitive(Primitive::Number(0.0))?;
            assert!(scope.strict_equals(&count, &zero)?);
            if detached {
                evaluate(scope, "styleParent.removeChild(inlineStyle)")?;
            }
            Ok(())
        })
        .unwrap();
        if matches!(report, StyleReport::Stop) {
            assert_queued_style_violation(&host);
            let mut session = host.stop().into_session().unwrap();
            assert!(session.dom().pending_source_edit().is_none());
            assert_eq!(session.snapshot().node(style).unwrap().parent(), Some(body));
            return;
        }
        assert!(matches!(
            host.run_attached_csp_violation_task(&permit, EVENT.dispatch)
                .unwrap(),
            DocumentCspViolationProgress::Complete(_)
        ));
        host.in_document(&permit, |scope| {
            let target = if detached { "document" } else { "inlineStyle" };
            let result = evaluate(
                scope,
                &format!(
                    r#"
                (() => {{
                    const event = styleViolations[0];
                    return styleViolations.length === 1 &&
                        event instanceof SecurityPolicyViolationEvent &&
                        event.isTrusted && event.target === {target} &&
                        event.blockedURI === 'inline' &&
                        event.effectiveDirective === 'style-src-elem' &&
                        event.originalPolicy === "default-src 'none' 'report-sample'" &&
                        event.disposition === 'enforce' && event.sample === 'p {{ color: red }}';
                }})()
            "#
                ),
            )?;
            let yes = scope.primitive(Primitive::Boolean(true))?;
            assert!(scope.strict_equals(&result, &yes)?);
            Ok(())
        })
        .unwrap();
        assert!(matches!(
            host.run_attached_csp_violation_task(&permit, EVENT.dispatch)
                .unwrap(),
            DocumentCspViolationProgress::Empty
        ));
    }
}

fn install_policy_before_style_retry(
    session: &mut DomSession,
    meta: NodeId,
    head: NodeId,
    cancel: &AtomicBool,
) {
    let mut source = session.source_execution();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::MetaCsp(request),
        ..
    } = source
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(meta, head, None),
            SOURCE,
            &mut services,
            cancel,
        )
        .unwrap()
    else {
        panic!("the Meta fixture must reach its native processor");
    };
    session
        .service_trusted_types_meta_csp(request, SOURCE, cancel)
        .unwrap();
    assert!(matches!(
        source
            .resume(
                &mut session.dom(),
                request.operation(),
                SOURCE,
                &mut services,
                cancel,
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
}

fn assert_queued_style_violation(host: &DocumentScriptHost) {
    crate::document_scripting::guest::tests::assert_queued_trusted_types_violations(
        host,
        &[(0, "inline", "p { color: red }")],
    );
}

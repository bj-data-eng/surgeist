use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use surgeist_cssom::CssomStore;
use surgeist_dom::DomStyleCssomRoot;
use surgeist_html::values::DocumentString;

pub(super) fn guest_insertion_publishes_a_sheet_under_the_known_empty_policy() {
    guest_insertion(None, None, true);
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
    for detached in [false, true] {
        guest_style(
            Some("default-src 'none' 'report-sample'"),
            None,
            "p { color: red }".into(),
            false,
            false,
            Some(detached),
        );
    }
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
    detach_before_report: Option<bool>,
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
    if meta.is_some() {
        session.enable_trusted_types_csp(&cancel).unwrap();
    }
    if let Some((value, true)) = nonce {
        let mut slot = DomBatch::new();
        slot.set_nonce(style, value);
        session.dom().apply(slot, &cancel).unwrap();
    }
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, body, "styleParent");
    expose_public_node_for_source_test(&mut host, &permit, style, "inlineStyle");
    if detach_before_report.is_some() {
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
        } else {
            "styleParent.appendChild(inlineStyle) === inlineStyle"
        };
        let result = evaluate(scope, script)?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&result, &expected)?);
        Ok(())
    });
    if inserted.is_err() {
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
    assert_eq!(node.parent(), Some(body));
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
    if let Some(detached) = detach_before_report {
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

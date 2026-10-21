//! An idle Style owner remains a real hosted-destruction obligation.
use super::*;

#[test]
fn settled_native_style_retains_exact_hosted_destruction_until_root_custody_cleanup() {
    let mut session = new_session();
    let mut hosted = session.hosted_execution();
    let host = hosted.create_host(&mut session.dom(), &cancel()).unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), host, &cancel())
        .unwrap();
    let document = top.document();
    let initial = session.snapshot();
    assert!(
        initial
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let base = initial
        .document(document)
        .unwrap()
        .document_base_record()
        .unwrap()
        .clone();
    assert_eq!(base.serialization(), &DocumentString::from("about:blank"));
    let settings = hosted
        .settings(&session.dom(), top.global(), &cancel())
        .unwrap();
    assert_eq!(settings.document().document_base_record().unwrap(), &base);

    let mut styles = session.style_execution();
    let mut input = Some(DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    ));
    styles
        .bind_root(
            &session.dom(),
            "idle-destruction".into(),
            &mut input,
            &cancel(),
        )
        .unwrap();
    assert!(input.is_none());
    let empty_cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert!(empty_cssom.sheets().is_empty());
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    batch.set_attribute(&style, "title", "A");
    let text = batch.create_text(document, "p { color: red }");
    batch.insert_before(&text, &style, None);
    let style = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let mut policy_calls = Vec::new();
    let insertion = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(style, body(&initial, document), None),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |source: &DomStyleSource| {
                    assert_eq!(source.element(), style);
                    assert_eq!(source.document(), document);
                    assert_eq!(source.root(), document.node());
                    assert_eq!(source.base_url(), base.serialization());
                    assert_eq!(
                        source.text().as_slice(),
                        "p { color: red }".encode_utf16().collect::<Vec<_>>()
                    );
                    policy_calls.push(source.operation());
                    Some(DomStyleInlineCspObservation::complete_empty(source, 1))
                },
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(insertion) = insertion else {
        panic!("real constructor-qualified Style creation must finish")
    };
    assert_eq!(policy_calls, [insertion.operation()]);
    assert!(!insertion.cancellation_requested());
    let associated_dom = session.snapshot();
    let sheet = associated_dom
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap()
        .clone();
    let associated_cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert_eq!(sheet.root(), document.node());
    assert_eq!(associated_cssom.sheets(), std::slice::from_ref(sheet.id()));
    let created = associated_cssom.sheet(sheet.id()).unwrap();
    assert_eq!(created.authored().rules().len(), 1);
    assert!(created.diagnostics().is_empty());
    assert_eq!(
        created.inputs().owner.as_deref(),
        Some("idle-destruction:1")
    );
    assert_eq!(created.inputs().version.identity, "idle-destruction:1");
    assert_eq!(created.inputs().version.revision, 1);
    assert_eq!(created.inputs().title.as_slice(), &[65]);
    assert!(!created.inputs().disabled);

    let first_names = styles
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap();
    assert_eq!(first_names.preferred_name(), &DocumentString::from("A"));
    assert_eq!(first_names.last_selected_name(), None);
    let selected = styles
        .select_sheet_set(&session.dom(), &first_names, &"A".into(), 1, &cancel())
        .unwrap();
    let preferred = styles
        .change_preferred_sheet_set_name(
            &session.dom(),
            selected.observation(),
            &"B".into(),
            1,
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        preferred.observation().preferred_name(),
        &DocumentString::from("B")
    );
    assert_eq!(
        preferred.observation().last_selected_name(),
        Some(&DocumentString::from("A"))
    );
    assert!(preferred.observation().revision() > selected.observation().revision());
    assert!(selected.observation().revision() > first_names.revision());

    let removal = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::detach(style),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_: &DomStyleSource| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(removal, DomSourceDriveProgress::Complete(_)));
    assert_eq!(policy_calls, [insertion.operation()]);
    let mut notifications = Notifications {
        styles: session.style_execution(),
        delivered: Vec::new(),
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    let listener = dispatcher
        .add_listener(
            &mut session.dom(),
            style,
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut Notifications, event| {
                context.delivered.push(event.target().unwrap());
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let dispatch_limits = DomEventDispatchLimits {
        path_items: 32,
        snapshot_listeners: 8,
        failure_records: 8,
        callbacks: 8,
        work: 100_000,
    };
    let delivery = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut notifications,
        dispatch_limits,
        &cancel(),
    )
    .unwrap();
    let DomStyleNetworkingProgress::Complete { receipt, .. } = delivery else {
        panic!("real no-import Load must dispatch and finish cleanup")
    };
    assert_eq!(receipt.event_type(), DomEventType::Load);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert_eq!(notifications.delivered, [style]);
    assert!(!dispatcher.remove_listener(listener));
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut notifications,
            dispatch_limits,
            &cancel()
        )
        .unwrap(),
        DomStyleNetworkingProgress::Empty
    ));
    assert_eq!(notifications.delivered, [style]);
    assert_eq!(styles.waiting_resources().unwrap(), 0);
    assert!(styles.pending_sheet(0).unwrap().is_none());
    let idle_dom = session.snapshot();
    assert_eq!(idle_dom.incomplete_source_operation(), None);
    assert_eq!(idle_dom.incomplete_hosted_operation(), None);
    assert!(session.dom().pending_source_edit().is_none());
    assert!(idle_dom.node(style).unwrap().parent().is_none());
    assert!(
        idle_dom
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(
        !idle_dom
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .is_render_blocking()
            .unwrap()
    );
    let idle_cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert!(idle_cssom.sheets().is_empty());
    assert!(
        idle_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    let idle_names = styles
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap();
    assert_eq!(
        idle_names.preferred_name(),
        preferred.observation().preferred_name()
    );
    assert_eq!(
        idle_names.last_selected_name(),
        preferred.observation().last_selected_name()
    );
    assert_eq!(idle_names.revision(), preferred.observation().revision());
    let capability = session
        .dom()
        .style_cssom_root(document.node())
        .unwrap()
        .unwrap();
    assert_eq!(
        capability.snapshot().unwrap().revision(),
        idle_cssom.revision()
    );

    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
        .unwrap();
    let mut blocked = false;
    for _ in 0..256 {
        match hosted.run_next(&mut session.dom(), queued.traversable(), &cancel()) {
            Err(DomHostedRuntimeError::Dom(DomHostedError::PendingDocumentObligations)) => {
                blocked = true;
                break;
            }
            Ok(DomHostedTaskProgress::Complete { operation })
                if operation == queued.operation() =>
            {
                panic!("canonical stylesheet custody cannot be silently discarded")
            }
            Ok(DomHostedTaskProgress::Idle) => {
                panic!("the retained original destruction cannot disappear")
            }
            Ok(_) => (),
            Err(error) => panic!("wrong cleanup refusal: {error:?}"),
        }
    }
    assert!(
        blocked,
        "must reach the actual Document cleanup custody guard"
    );
    let guarded = session.snapshot();
    assert!(guarded.node(document.node()).is_ok());
    assert_eq!(
        guarded.document(document).unwrap().navigable(),
        Some(top.navigable())
    );
    assert!(
        guarded
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        idle_dom
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    // This parallel destruction retains its own operation/task, rather than
    // publishing the creation protocol's global incomplete-hosted marker.
    assert_eq!(guarded.incomplete_hosted_operation(), None);
    assert_eq!(
        hosted
            .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
            .unwrap(),
        queued
    );
    assert!(matches!(
        hosted.run_next(&mut session.dom(), queued.traversable(), &cancel()),
        Err(DomHostedRuntimeError::Dom(
            DomHostedError::PendingDocumentObligations
        ))
    ));
    assert_eq!(session.snapshot().incomplete_hosted_operation(), None);
    assert_eq!(session.snapshot().stamp(), guarded.stamp());
    assert!(
        session
            .dom()
            .style_cssom_root(document.node())
            .unwrap()
            .is_some()
    );
    assert_eq!(
        capability.snapshot().unwrap().revision(),
        idle_cssom.revision()
    );
    assert_eq!(
        styles
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .revision(),
        idle_cssom.revision()
    );
    assert_eq!(styles.waiting_resources().unwrap(), 0);
    assert!(styles.pending_sheet(0).unwrap().is_none());
    assert_eq!(notifications.delivered, [style]);
    assert_eq!(policy_calls, [insertion.operation()]);
    let pending_names = styles
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap();
    assert_eq!(pending_names.preferred_name(), idle_names.preferred_name());
    assert_eq!(
        pending_names.last_selected_name(),
        idle_names.last_selected_name()
    );
    assert_eq!(pending_names.revision(), idle_names.revision());
    assert_eq!(receipt.callbacks(), 1);
    assert!(
        associated_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    assert_eq!(
        associated_dom
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap(),
        Some(&sheet)
    );
    assert_eq!(settings.document().document_base_record().unwrap(), &base);
    assert_eq!(idle_names.preferred_name(), &DocumentString::from("B"));
    assert_eq!(
        idle_names.last_selected_name(),
        Some(&DocumentString::from("A"))
    );
}

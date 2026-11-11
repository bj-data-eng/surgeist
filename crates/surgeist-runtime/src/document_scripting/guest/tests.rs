use super::*;

fn public_node_host(node_work: u64, source_work: u64) -> DocumentScriptHost {
    let cancellation = Arc::new(AtomicBool::new(false));
    let (session, target) = super::super::cross_realm_tests::hosted_session(&cancellation, 0);
    attach_public_nodes(session, target, cancellation, node_work, source_work)
}

fn attach_public_nodes(
    mut session: DomSession,
    target: DomHostedTopLevel,
    cancellation: Arc<AtomicBool>,
    node_work: u64,
    source_work: u64,
) -> DocumentScriptHost {
    let mut registry = DomEventDispatcher::new(crate::DomEventRegistryLimits {
        registrations: 8,
        pending_events: 2,
        nested_dispatches: 2,
        storage_bytes: 32_000,
        work: 10_000,
    })
    .unwrap();
    registry.bind(&mut session.dom(), &cancellation).unwrap();
    let mut registry = Some(registry);
    PreparedDocumentScriptHost::prepare(session, 2_000_000, cancellation)
        .unwrap()
        .enable_with_guest_event_registry(
            target,
            HtmlScriptingPolicy {
                user_allows: true,
                automation_allows: true,
            },
            DocumentScriptHooks {
                child_count: &units("readChildCount"),
                detach_source_node_if: &units("detachIf"),
            },
            &mut registry,
            DocumentGuestEventLimits {
                storage_bytes: 64_000,
                work: 10_000,
                pending_exception_reports: 4,
                node_retirement: DomNodeRetirementLimits::new(16_000_000, node_work),
                source_edits: DomSourceLimits::new(16_000_000, 200_000, source_work, 64, 64, 64, 2),
                dispatch: DomEventDispatchLimits {
                    path_items: 2,
                    snapshot_listeners: 8,
                    failure_records: 4,
                    callbacks: 8,
                    work: 10_000,
                },
            },
        )
        .unwrap()
}

pub(in crate::document_scripting) fn adopted_element_uses_its_registered_creation_realm() {
    use surgeist_dom::DomBatch;
    use surgeist_html::values::Namespace;

    let cancellation = Arc::new(AtomicBool::new(false));
    let (mut session, target) = super::super::cross_realm_tests::hosted_session(&cancellation, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted
            .create_headless_host(&mut dom, &cancellation)
            .unwrap();
        hosted
            .create_top_level(&mut dom, owner, &cancellation)
            .unwrap()
    };
    let element = {
        let mut batch = DomBatch::new();
        let element = batch.create_element_ns(origin.document(), Namespace::html(), "div");
        let commit = session.dom().apply(batch, &cancellation).unwrap();
        commit.created(&element).unwrap().unwrap()
    };
    let watch_limits = DomNodeRetirementLimits::new(16_000_000, 1_000);
    let mut rust_inbox = session
        .dom()
        .raw()
        .node_retirement_inbox(watch_limits, &cancellation)
        .unwrap();
    let _rust_watch = session
        .dom()
        .raw_mut()
        .watch_node_retirement(element, &rust_inbox, 71, watch_limits, &cancellation)
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancellation.clone(), 1_000, 20_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("globalThis.destination = new Document()"))
            .map(drop)
    })
    .unwrap();
    let snapshot = host.snapshot().unwrap();
    let destination = snapshot
        .nodes()
        .filter_map(|node| snapshot.node(node).unwrap().as_document())
        .map(|view| view.id())
        .find(|document| *document != target.document() && *document != origin.document())
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(element, destination);
        batch.insert_before(element, destination, None);
        dom.apply(batch, &cancellation).unwrap();
    })
    .unwrap();
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot.relevant_global(element).unwrap(),
        Some(origin.global())
    );
    assert_eq!(
        snapshot.document(destination).unwrap().document_element(),
        Some(element)
    );

    assert!(rust_inbox.take().is_none());
    let projected = host.in_document(&permit, |scope| {
        let wrapper = scope.evaluate(&units(
            "globalThis.adopted = destination.documentElement; adopted.marker = Symbol(); adopted",
        ))?;
        let realm = scope
            .callback_object(&wrapper)?
            .associated_realm()
            .identity();
        assert_ne!(realm, scope.realm_authority()?.identity());
        assert_guest(
            scope,
            "adopted === destination.documentElement && adopted.localName === 'div' && \
             adopted.ownerDocument === destination && adopted.getRootNode() === destination",
        )?;
        Ok((realm, scope.retain(&wrapper)))
    });
    let (realm, wrapper) = match projected {
        Ok(projection) => projection,
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => panic!(
            "registered origin could not expose the adopted root: {:?}",
            failure.cause::<DocumentGuestEventError>()
        ),
        Err(error) => panic!("registered origin could not expose the adopted root: {error:?}"),
    };
    let engine = host.engine.as_mut().unwrap();
    let original = engine
        .creations
        .find(origin.global(), || Ok::<(), Error>(()))
        .unwrap()
        .unwrap();
    let (original_marker, origin_dom_exception) = engine.process.execution().in_realm(original, |scope| {
        assert_eq!(scope.realm_authority()?.identity(), realm);
        let global = scope.global()?;
        let wrapper = scope.import(&wrapper)?;
        scope.set(&global, PropertyKey::String(&units("adopted")), &wrapper)?;
        assert_guest(
            scope,
            "Object.getPrototypeOf(adopted) === HTMLDivElement.prototype && \
             adopted instanceof Node && adopted instanceof Element && adopted instanceof EventTarget",
        )?;
        assert_guest(
            scope,
            r#"(() => {
                try { EventTarget.prototype.dispatchEvent.call(globalThis, null); }
                catch (error) {
                    return error instanceof TypeError && error.message === "EventTarget required";
                }
                return false;
            })()"#,
        )?;
        let marker = scope.evaluate(&units("adopted.marker"))?;
        let exception = scope.evaluate(&units("DOMException"))?;
        scope.evaluate(&units("delete globalThis.adopted"))?;
        Ok((scope.retain(&marker), scope.retain(&exception)))
    }).unwrap();

    host.with_native(|dom, mut hosted, _| {
        let retirement = hosted
            .destroy_top_level(dom, origin.navigable(), &cancellation)
            .unwrap();
        assert!((0..256).any(|_| matches!(
            hosted.run_next(dom, retirement.traversable(), &cancellation).unwrap(),
            crate::DomHostedTaskProgress::Complete { operation } if operation == retirement.operation()
        )));
        hosted.destroy_host(dom, origin.host(), &cancellation).unwrap();
        let mut detach = DomBatch::new();
        detach.detach(element);
        dom.apply(detach, &cancellation).unwrap();
    })
    .unwrap();
    assert!(rust_inbox.take().is_none());
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "destination.documentElement === null && adopted.getRootNode() === adopted",
        )
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut insert = DomBatch::new();
        insert.insert_before(element, destination, None);
        dom.apply(insert, &cancellation).unwrap();
    })
    .unwrap();
    let dispatched = host.in_document(&permit, |scope| {
        let marker = scope.import(&original_marker)?;
        let wrapper = scope.import(&wrapper)?;
        let observed = scope.get(&wrapper, PropertyKey::String(&units("marker")))?;
        assert!(scope.strict_equals(&marker, &observed)?);
        let global = scope.global()?;
        let exception = scope.import(&origin_dom_exception)?;
        scope.set(
            &global,
            PropertyKey::String(&units("originDomException")),
            &exception,
        )?;
        assert_guest(
            scope,
            r#"(() => {
                const event = new Event("adopted");
                const calls = [];
                const listener = { handleEvent(value) {
                    calls.push(this === listener && value === event
                        && value.target === adopted && value.currentTarget === adopted);
                    try { adopted.dispatchEvent(value); }
                    catch (error) {
                        calls.push(error instanceof originDomException
                            && !(error instanceof DOMException)
                            && error.name === "InvalidStateError");
                    }
                } };
                adopted.addEventListener("adopted", listener, { once: true });
                const same = adopted === destination.documentElement;
                const dispatched = adopted.dispatchEvent(event);
                return same && dispatched && calls.length === 2 && calls.every(Boolean)
                    && event.target === adopted && event.currentTarget === null;
            })()"#,
        )
    });
    match dispatched {
        Ok(()) => {}
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => panic!(
            "inherited EventTarget dispatch refused for the adopted root: {:?}",
            failure.cause::<DocumentGuestEventError>()
        ),
        Err(error) => panic!("inherited EventTarget dispatch refused: {error:?}"),
    }
    host.with_native(|dom, _, _| {
        let mut destroy = DomBatch::new();
        destroy.detach(element);
        destroy.destroy(element);
        dom.apply(destroy, &cancellation).unwrap();
    })
    .unwrap();
    let notice = rust_inbox.take().unwrap();
    assert_eq!(notice.node(), element);
    assert_eq!(notice.custody_key(), 71);
    assert!(rust_inbox.take().is_none());
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = host
        .in_document(&permit, |scope| {
            scope.evaluate(&units("adopted.localName")).map(drop)
        })
    else {
        panic!("the original V8 wrapper must observe exact native retirement");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(in crate::document_scripting) fn element_projection_admits_exact_work_and_backing() {
    const ROOT: &str = "window.document.documentElement";
    // Entry, bank lookup, one radix page, first index formation and custody
    // admission, followed by the prepaid worst-case exact disarm.
    let native_work = 5 + 2 * u64::from(usize::BITS) + 1;
    let read = |scope: &mut Scope<'_>| {
        let root = scope.evaluate(&units(ROOT))?;
        assert_eq!(scope.kind(&root)?, surgeist_bindings_js::ValueKind::Object);
        Ok(())
    };
    let mut control = public_node_host(native_work, 20_000);
    let permit = control.permit().unwrap();
    let events = control.guest_events().unwrap();
    let before = (events.budget.work.get(), events.budget.used.get());
    control.in_document(&permit, read).unwrap();
    let binding_work = events.budget.work.get() - before.0;
    let bytes = events.budget.used.get() - before.1;
    assert!(binding_work > 0 && bytes > 0);
    drop((control, events));

    for (work, bytes, accepted) in [
        (binding_work, bytes, true),
        (binding_work - 1, bytes, false),
        (binding_work, bytes - 1, false),
    ] {
        let mut host = public_node_host(native_work, 20_000);
        let permit = host.permit().unwrap();
        let events = host.guest_events().unwrap();
        let spare = events.limits.storage_bytes - events.budget.used.get() - bytes;
        let backing = events.reserve(&host.native, spare).unwrap();
        events.budget.work.set(events.limits.work - work);
        let stamp = host.snapshot().unwrap().stamp();
        let before = events.budget.used.get();
        let result = host.in_document(&permit, read);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        if accepted {
            result.unwrap();
            assert_eq!(events.budget.work.get(), events.limits.work);
            assert_eq!(events.budget.used.get(), events.limits.storage_bytes);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short Element projection must refuse");
            };
            assert!(matches!(
                (
                    work < binding_work,
                    failure.cause::<DocumentGuestEventError>()
                ),
                (true, Some(DocumentGuestEventError::WorkLimit))
                    | (
                        false,
                        Some(DocumentGuestEventError::Document(
                            DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
                        ))
                    )
            ));
            assert_eq!(events.budget.used.get(), before);
        }
        drop(backing);
    }

    let mut host = public_node_host(native_work - 1, 20_000);
    let permit = host.permit().unwrap();
    let events = host.guest_events().unwrap();
    let before = events.budget.used.get();
    let stamp = host.snapshot().unwrap().stamp();
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) =
        host.in_document(&permit, read)
    else {
        panic!("one-short canonical watch must refuse Element publication");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::Dom(DomError::NodeRetirementWorkLimit { limit })
        ))) if *limit == native_work - 1
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), stamp);
    assert_eq!(
        events.budget.used.get(),
        before,
        "watch refusal must release all wrapper backing"
    );
    drop((host, events));

    let mut host = public_node_host(native_work, 20_000);
    let permit = host.permit().unwrap();
    let events = host.guest_events().unwrap();
    let before = (events.budget.work.get(), events.budget.used.get());
    let stamp = host.snapshot().unwrap().stamp();
    let cancel = host.native.cancellation.clone();
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(matches!(
        host.in_document(&permit, read),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Cancelled)
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), stamp);
    assert_eq!((events.budget.work.get(), events.budget.used.get()), before);
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    host.in_document(&permit, read).unwrap();
}

pub(in crate::document_scripting) fn refused_element_realm_installation_releases_guest_backing() {
    let create_origin = |host: &mut DocumentScriptHost| {
        let cancel = host.native.cancellation.clone();
        host.with_native(|dom, mut hosted, _| {
            let owner = hosted.create_headless_host(dom, &cancel).unwrap();
            hosted.create_top_level(dom, owner, &cancel).unwrap()
        })
        .unwrap()
    };
    let mut control = public_node_host(1_000, 20_000);
    let permit = control.permit().unwrap();
    let origin = create_origin(&mut control);
    let events = control.guest_events().unwrap();
    let before = (events.budget.used.get(), events.budget.work.get());
    control
        .register_node_creation_realm(&permit, origin)
        .unwrap();
    let exact = events.budget.used.get() - before.0;
    let work = events.budget.work.get() - before.1;
    assert!(exact > 0 && work > 0);
    drop((control, events));

    let mut host = public_node_host(1_000, 20_000);
    let permit = host.permit().unwrap();
    let origin = create_origin(&mut host);
    let events = host.guest_events().unwrap();
    let spare = events.limits.storage_bytes - events.budget.used.get() - exact;
    let one_short = events.reserve(&host.native, spare + 1).unwrap();
    let before = events.budget.used.get();
    assert!(matches!(
        host.register_node_creation_realm(&permit, origin),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::StorageLimit
        ))
    ));
    assert_eq!(
        events.budget.used.get(),
        before,
        "an unpublished realm must release its guest backing"
    );
    drop(one_short);
    let exact_backing = events.reserve(&host.native, spare).unwrap();
    events.budget.work.set(events.limits.work - work + 1);
    let before = (events.budget.used.get(), events.budget.work.get());
    assert!(matches!(
        host.register_node_creation_realm(&permit, origin),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::CreationRealmWorkLimit
        ))
    ));
    assert_eq!((events.budget.used.get(), events.budget.work.get()), before);
    events.budget.work.set(events.limits.work - work);
    host.register_node_creation_realm(&permit, origin).unwrap();
    assert_eq!(events.budget.used.get(), events.limits.storage_bytes);
    assert_eq!(events.budget.work.get(), events.limits.work);
    assert!(matches!(
        host.register_node_creation_realm(&permit, origin),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::CreationRealmAlreadyRegistered
        ))
    ));
    drop(exact_backing);
}

pub(in crate::document_scripting) fn fragment_custody_survives_collection_and_releases_on_retirement()
 {
    let mut host = public_node_host(1_000, 20_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                r#"(() => {
            const fragment = new DocumentFragment();
            const listener = () => {};
            globalThis.originalMarker = Symbol();
            fragment.marker = originalMarker;
            globalThis.fragmentProbe = new WeakRef(fragment);
            globalThis.listenerProbe = new WeakRef(listener);
            fragment.addEventListener("retire", listener);
        })()"#,
            ))
            .map(drop)
    })
    .unwrap();
    host.checkpoint().unwrap();
    host.engine
        .as_mut()
        .unwrap()
        .process
        .execution()
        .memory_pressure();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "fragmentProbe.deref()?.marker === originalMarker && !!listenerProbe.deref()",
        )
    })
    .unwrap();

    let snapshot = host.snapshot().unwrap();
    let fragment = snapshot
        .nodes()
        .find(|id| {
            matches!(
                snapshot.node(*id).unwrap().kind(),
                surgeist_dom::DomNodeKind::DocumentFragment
            )
        })
        .unwrap();
    let events = host.guest_events().unwrap();
    let before = events.budget.used.get();
    let id = events
        .registry
        .borrow()
        .next_guest_registration(
            &fragment.into(),
            &units("retire"),
            false,
            &mut 0,
            &AtomicBool::new(false),
        )
        .unwrap()
        .unwrap()
        .0;
    host.with_native(|dom, _, _| {
        let mut batch = surgeist_dom::DomBatch::new();
        batch.destroy(fragment);
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
    })
    .unwrap();
    assert!(!events.registry.borrow().has_registration(id));
    assert!(events.budget.used.get() < before);
    let collected = |host: &mut DocumentScriptHost| {
        host.in_document(&permit, |scope| {
            let result = scope.evaluate(&units(
                "fragmentProbe.deref() === undefined && listenerProbe.deref() === undefined",
            ))?;
            let yes = scope.primitive(Primitive::Boolean(true))?;
            scope.strict_equals(&result, &yes)
        })
        .unwrap()
    };
    for _ in 0..8 {
        host.checkpoint().unwrap();
        host.engine
            .as_mut()
            .unwrap()
            .process
            .execution()
            .memory_pressure();
        if collected(&mut host) {
            return;
        }
    }
    panic!("retirement must release the last managed Fragment and callback references");
}

pub(in crate::document_scripting) fn fragment_watch_refusal_preserves_native_state() {
    let mut host = public_node_host(2, 20_000);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap();
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = host
        .in_document(&permit, |scope| {
            scope.evaluate(&units("new DocumentFragment()")).map(drop)
        })
    else {
        panic!("watch admission must refuse the Fragment");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                DomError::NodeRetirementWorkLimit { limit: 2 }
            ))
        ))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.nodes().count(), before.nodes().count());
    host.stop().into_session().unwrap();
}

pub(in crate::document_scripting) fn character_data_source_refusal_preserves_native_state() {
    let mut host = public_node_host(1_000, 0);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("globalThis.refusedText = new Text('original')"))
            .map(drop)
    })
    .unwrap();
    let before = host.snapshot().unwrap();
    let original = units("original");
    let node = before
        .nodes()
        .find(|id| {
            matches!(
                before.node(*id).unwrap().kind(),
                surgeist_dom::DomNodeKind::Text(data) if data.code_units() == original.as_slice()
            )
        })
        .unwrap();
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) =
        host.in_document(&permit, |scope| {
            scope
                .evaluate(&units("refusedText.replaceData(0, 8, 'replacement')"))
                .map(drop)
        })
    else {
        panic!("the original canonical Source limit must refuse the edit");
    };
    assert!(matches!(
        failure.cause::<crate::DomSourceDriveError>(),
        Some(crate::DomSourceDriveError::Source(
            crate::DomSourceRuntimeError::Dom(DomError::SourceWorkLimit { limit: 0 })
        ))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.node(node).unwrap().data(),
        before.node(node).unwrap().data()
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}
use crate::DomEventRegistryLimits;
use surgeist_bindings_js::{ExceptionPhase, Primitive, PropertyKey};

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn host(cancel: Arc<AtomicBool>) -> DocumentScriptHost {
    let (session, target) = super::super::cross_realm_tests::hosted_session(&cancel, 0);
    PreparedDocumentScriptHost::prepare(session, 2_000_000, cancel)
        .unwrap()
        .enable(
            target,
            HtmlScriptingPolicy {
                user_allows: true,
                automation_allows: true,
            },
            DocumentScriptHooks {
                child_count: &units("readChildCount"),
                detach_source_node_if: &units("detachIf"),
            },
        )
        .unwrap()
}

fn in_realm_root(scope: &mut Scope<'_>, root: &Root) -> Result<Root, Error> {
    let input = scope.import(root)?;
    let holder = scope.object(ObjectPrototype::Default)?;
    let key = PropertyKey::String(&[b'x'.into()]);
    scope.set(&holder, key, &input)?;
    let read = scope.get(&holder, key)?;
    Ok(scope.retain(&read))
}

fn attach_registry(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) {
    attach_registry_with_limits(host, permit, 4, 32);
}

fn attach_registry_with_limits(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    nested_dispatches: usize,
    pending_exception_reports: usize,
) {
    let mut registry = Some(
        DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 8,
            pending_events: 4,
            nested_dispatches,
            storage_bytes: 1_000_000,
            work: 100_000,
        })
        .unwrap(),
    );
    host.attach_guest_event_registry(
        permit,
        &mut registry,
        DocumentGuestEventLimits {
            storage_bytes: 32_000,
            work: 1000,
            pending_exception_reports,
            node_retirement: DomNodeRetirementLimits::new(16_000_000, 1_000),
            source_edits: DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
            dispatch: DomEventDispatchLimits {
                path_items: 1,
                snapshot_listeners: 8,
                failure_records: 8,
                callbacks: 8,
                work: 10_000,
            },
        },
    )
    .unwrap();
    assert!(registry.is_none());
}

pub(in crate::document_scripting) fn unpublished_window_requires_its_original_bound_registry() {
    let cancellation = Arc::new(AtomicBool::new(false));
    let (mut session, target) = super::super::cross_realm_tests::hosted_session(&cancellation, 0);
    let (mut foreign, _) = super::super::cross_realm_tests::hosted_session(&cancellation, 0);
    let registry_limits = DomEventRegistryLimits {
        registrations: 2,
        pending_events: 2,
        nested_dispatches: 2,
        storage_bytes: 32_000,
        work: 1_000,
    };
    let mut bound = DomEventDispatcher::new(registry_limits).unwrap();
    bound.bind(&mut session.dom(), &cancellation).unwrap();
    let kind = units("retained");
    let original_registration = bound
        .add_listener(
            &mut session.dom(),
            target.global(),
            &kind,
            DomEventCallback::new(|_, _, _: &mut (), _| Ok(())),
            Default::default(),
        )
        .unwrap();
    let mut unbound = Some(DomEventDispatcher::new(registry_limits).unwrap());
    let mut foreign_registry = DomEventDispatcher::new(registry_limits).unwrap();
    foreign_registry
        .bind(&mut foreign.dom(), &cancellation)
        .unwrap();
    let mut foreign_registry = Some(foreign_registry);
    let mut bound = Some(bound);
    let limits = DocumentGuestEventLimits {
        storage_bytes: 64_000,
        work: 1_000,
        pending_exception_reports: 2,
        node_retirement: DomNodeRetirementLimits::new(16_000_000, 1_000),
        source_edits: DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
        dispatch: DomEventDispatchLimits {
            path_items: 1,
            snapshot_listeners: 2,
            failure_records: 2,
            callbacks: 2,
            work: 1_000,
        },
    };
    let policy = HtmlScriptingPolicy {
        user_allows: true,
        automation_allows: true,
    };
    let read = units("readChildCount");
    let detach = units("detachIf");
    let hooks = || DocumentScriptHooks {
        child_count: &read,
        detach_source_node_if: &detach,
    };
    let prepared = PreparedDocumentScriptHost::prepare(session, 2_000_000, cancellation).unwrap();

    let Err(unbound_failure) =
        prepared.enable_with_guest_event_registry(target, policy, hooks(), &mut unbound, limits)
    else {
        panic!("an unbound competitor must not publish a Window");
    };
    assert!(matches!(
        unbound_failure.error,
        DocumentGuestEventError::RegistryUnbound
    ));
    assert!(unbound.is_some());
    assert!(unbound_failure.owner.native.attachment.get().is_none());

    let Err(foreign_failure) = unbound_failure.owner.enable_with_guest_event_registry(
        target,
        policy,
        hooks(),
        &mut foreign_registry,
        limits,
    ) else {
        panic!("a foreign bound registry must not publish a Window");
    };
    assert!(matches!(
        foreign_failure.error,
        DocumentGuestEventError::Dispatch(DomEventDispatchError::Dom(
            surgeist_dom::DomError::ForeignOwner
        ))
    ));
    assert!(foreign_registry.is_some());
    assert!(foreign_failure.owner.native.attachment.get().is_none());

    let Err(storage_failure) = foreign_failure.owner.enable_with_guest_event_registry(
        target,
        policy,
        hooks(),
        &mut bound,
        DocumentGuestEventLimits {
            storage_bytes: 0,
            ..limits
        },
    ) else {
        panic!("insufficient event backing must leave the Window unpublished");
    };
    assert!(matches!(
        storage_failure.error,
        DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::StorageLimit
        ))
    ));
    assert!(bound.is_some());
    assert!(storage_failure.owner.native.attachment.get().is_none());

    let mut host = storage_failure
        .owner
        .enable_with_guest_event_registry(target, policy, hooks(), &mut bound, limits)
        .unwrap();
    assert!(bound.is_none());
    let exact_activation = host.guest_events().unwrap().budget.used.get();
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "window.document.custodyMarker = Symbol(); globalThis.marker = window.document.custodyMarker",
            ))
            .map(drop)
    })
    .unwrap();
    host.checkpoint().unwrap();
    host.engine
        .as_mut()
        .unwrap()
        .process
        .execution()
        .memory_pressure();
    host.in_document(&permit, |scope| {
        assert_guest(scope, "window.document.custodyMarker === marker")
    })
    .unwrap();
    assert!(
        host.remove_guest_event_listener(&permit, original_registration)
            .unwrap()
    );
    let events = host.guest_events().unwrap();
    host.in_document(&permit, |scope| {
        let before = events.budget.used.get();
        scope.evaluate(&units(
            r#"(() => {
                const callback = () => {};
                const api = EventTarget.prototype;
                api.addEventListener.call(window, "release", callback);
                api.removeEventListener.call(window, "release", callback);
            })()"#,
        ))?;
        assert_eq!(
            events.budget.used.get(),
            before,
            "removal releases the callback holder before the guest turn ends",
        );
        let event = scope.evaluate(&units("new Event('release')"))?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("releaseEvent")), &event)?;
        let before = events.budget.used.get();
        scope.evaluate(&units(
            r#"(() => {
                const callback = () => {};
                const api = EventTarget.prototype;
                api.addEventListener.call(window, "release", callback, { once: true });
                api.dispatchEvent.call(window, releaseEvent);
            })()"#,
        ))?;
        assert_eq!(
            events.budget.used.get(),
            before,
            "once delivery releases the callback holder before the guest turn ends",
        );
        Ok(())
    })
    .unwrap();
    let event_target = host.create_guest_event_target(&permit).unwrap();
    let event = host
        .create_guest_event(&permit, &kind, Default::default())
        .unwrap();
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let value = event.object(scope)?;
        scope.set(&global, PropertyKey::String(&units("rustEvent")), &value)?;
        assert_guest(
            scope,
            r#"(() => {
            const trusted = Object.getOwnPropertyDescriptor(rustEvent, "isTrusted");
            const type = Object.getOwnPropertyDescriptor(Event.prototype, "type");
            let rejected = false;
            try { type.get.call(Object.create(Event.prototype)); }
            catch (error) { rejected = error instanceof TypeError; }
            return rustEvent instanceof Event && rustEvent.type === "retained" &&
                   trusted.enumerable && !trusted.configurable &&
                   trusted.set === undefined && !trusted.get.call(rustEvent) &&
                   Event.NONE === 0 && Event.prototype.AT_TARGET === 2 && rejected &&
                   new EventTarget().dispatchEvent(new Event("actual"));
        })()"#,
        )
    })
    .unwrap();
    let report = host
        .dispatch_guest_event(&permit, &event_target, &event)
        .unwrap();
    assert_eq!(report.receipt().event_type().code_units(), kind);
    assert_eq!(report.receipt().callbacks(), 0);

    // Settle the earlier temporary dispatch before measuring this cycle.
    host.checkpoint().unwrap();
    host.engine
        .as_mut()
        .unwrap()
        .process
        .execution()
        .memory_pressure();
    host.in_document(&permit, |_| Ok(())).unwrap();
    let events = host.guest_events().unwrap();
    let before = events.budget.used.get();
    let id = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&units(
                r#"(() => {
            const target = new EventTarget();
            const callback = () => target;
            target.addEventListener("cycle", callback);
            return target;
        })()"#,
            ))?;
            let projected = scope
                .native_object::<GuestTargetProjection>(&value)?
                .unwrap();
            let id = events
                .registry
                .borrow()
                .next_guest_registration(
                    &projected.payload().native.clone().into(),
                    &units("cycle"),
                    false,
                    &mut 0,
                    &AtomicBool::new(false),
                )
                .unwrap()
                .unwrap()
                .0;
            Ok(id)
        })
        .unwrap();
    assert!(events.registry.borrow().has_registration(id));
    assert!(events.budget.used.get() > before);
    for _ in 0..8 {
        host.checkpoint().unwrap();
        host.engine
            .as_mut()
            .unwrap()
            .process
            .execution()
            .memory_pressure();
        host.in_document(&permit, |_| Ok(())).unwrap();
        if !events.registry.borrow().has_registration(id) {
            break;
        }
    }
    assert!(!events.registry.borrow().has_registration(id));
    assert_eq!(events.budget.used.get(), before);

    let kind = units("once budget");
    let event = host
        .create_guest_event(&permit, &kind, Default::default())
        .unwrap();
    let (id, slot) = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            let receiver = event_target.object(scope)?;
            scope.set(
                &global,
                PropertyKey::String(&units("onceTarget")),
                &receiver,
            )?;
            scope.evaluate(&units(
                "onceTarget.addEventListener('once budget', () => { throw 1; }, { once: true })",
            ))?;
            let mut cursor = 0;
            let (id, registered) = events
                .registry
                .borrow()
                .next_guest_registration(
                    &event_target.native.clone().into(),
                    &kind,
                    false,
                    &mut cursor,
                    &AtomicBool::new(false),
                )
                .unwrap()
                .unwrap();
            let RegisteredListenerObject::Traced(traced) = &registered.object else {
                panic!("public registration must be traced from its target");
            };
            assert!(
                scope
                    .native_object_reference::<GuestTargetProjection>(&receiver, traced.slot)?
                    .is_some()
            );
            Ok((id, traced.slot))
        })
        .unwrap();

    // Reserve the direct report vector, its root bank and two root slots,
    // then admit projection qualification and the first native advance.
    // Callback work itself is one unit beyond this exact prefix.
    let prefix = 4 + u64::try_from(limits.dispatch.failure_records).unwrap();
    events.budget.work.set(limits.work - prefix);
    let Err(refused) = host.dispatch_guest_event(&permit, &event_target, &event) else {
        panic!("callback work must refuse after native once selection");
    };
    assert!(
        matches!(refused.error, DocumentGuestEventError::WorkLimit),
        "callback work must be the first refusal, got {:?}",
        refused.error,
    );
    assert!(refused.exceptions().is_empty());
    assert!(!events.registry.borrow().has_registration(id));
    host.in_document(&permit, |scope| {
        let receiver = event_target.object(scope)?;
        assert!(
            scope
                .native_object_reference::<GuestTargetProjection>(&receiver, slot)?
                .is_none(),
            "the irreversible native once removal must release its traced edge"
        );
        Ok(())
    })
    .unwrap();
    drop((host, events, event, event_target, report));

    for storage_bytes in [exact_activation - 1, exact_activation] {
        let cancellation = Arc::new(AtomicBool::new(false));
        let (mut session, target) =
            super::super::cross_realm_tests::hosted_session(&cancellation, 0);
        let mut registry = DomEventDispatcher::new(registry_limits).unwrap();
        registry.bind(&mut session.dom(), &cancellation).unwrap();
        let mut registry = Some(registry);
        let prepared =
            PreparedDocumentScriptHost::prepare(session, 2_000_000, cancellation).unwrap();
        let result = prepared.enable_with_guest_event_registry(
            target,
            policy,
            hooks(),
            &mut registry,
            DocumentGuestEventLimits {
                storage_bytes,
                ..limits
            },
        );
        if storage_bytes == exact_activation {
            let host = result.unwrap();
            assert!(registry.is_none());
            assert_eq!(
                host.guest_events().unwrap().budget.used.get(),
                exact_activation
            );
        } else {
            let Err(refused) = result else {
                panic!("one byte short must leave the Window unpublished");
            };
            assert!(matches!(
                refused.error,
                DocumentGuestEventError::Document(DocumentScriptError::Native(
                    DocumentScriptNativeError::StorageLimit,
                ))
            ));
            assert!(registry.is_some());
            assert!(refused.owner.native.attachment.get().is_none());
        }
    }
}

pub(in crate::document_scripting) fn refused_first_target_releases_prototype_capture() {
    let (prototype_bytes, target_bytes) = {
        let mut host = host(Arc::new(AtomicBool::new(false)));
        let permit = host.permit().unwrap();
        attach_registry(&mut host, &permit);
        let events = host.guest_events().unwrap();
        let owner = host.native.clone();
        let before = events.budget.used.get();
        host.in_document(&permit, |scope| {
            Ok(events.target_prototype(&owner, scope).map(drop))
        })
        .unwrap()
        .unwrap();
        let with_prototype = events.budget.used.get();
        let _target = host.create_guest_event_target(&permit).unwrap();
        (
            with_prototype - before,
            events.budget.used.get() - with_prototype,
        )
    };
    assert!(prototype_bytes > 0 && target_bytes > 0);

    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let events = host.guest_events().unwrap();
    let available = events.limits.storage_bytes - events.budget.used.get();
    let one_short = events
        .reserve(&host.native, available - prototype_bytes + 1)
        .unwrap();
    let at_refusal = events.budget.used.get();
    assert!(matches!(
        host.create_guest_event_target(&permit),
        Err(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
        ))
    ));
    assert!(events.target_prototype.get().is_none());
    assert_eq!(
        events.budget.used.get(),
        at_refusal,
        "a refused prototype must release its method capture without collection"
    );
    drop(one_short);

    let exact = events
        .reserve(&host.native, available - prototype_bytes - target_bytes)
        .unwrap();
    let target = host.create_guest_event_target(&permit).unwrap();
    assert_eq!(events.budget.used.get(), events.limits.storage_bytes);
    host.in_document(&permit, |scope| {
        let target = target.object(scope)?;
        let method = scope.get(&target, PropertyKey::String(&units("dispatchEvent")))?;
        assert!(scope.is_callable(&method)?);
        Ok(())
    })
    .unwrap();
    drop(exact);
}

pub(in crate::document_scripting) fn refused_projection_releases_native_backing_before_retry() {
    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let events = host.guest_events().unwrap();
    let kind = units("projection");
    let before = events.budget.used.get();
    let control = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let with_control = events.budget.used.get();
    let projection = with_control - before;
    assert!(projection > 0);

    // Measure the complete backing of the same Event projection. Hold its
    // positive control live so the second attempt cannot borrow its storage.
    let exact_margin = events.limits.storage_bytes - with_control - projection;
    let one_short = events.reserve(&host.native, exact_margin + 1).unwrap();
    let at_refusal = events.budget.used.get();
    assert!(matches!(
        host.create_guest_event(&permit, &kind, DomEventInit::default()),
        Err(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
        ))
    ));
    assert_eq!(
        events.budget.used.get(),
        at_refusal,
        "an unpublished Event must release its backing without collection"
    );
    drop(one_short);

    let exact = events.reserve(&host.native, exact_margin).unwrap();
    let retry = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    assert_eq!(events.budget.used.get(), events.limits.storage_bytes);
    host.in_document(&permit, |scope| {
        let control = control.object(scope)?;
        let retry = retry.object(scope)?;
        assert!(!scope.strict_equals(&control, &retry)?);
        Ok(())
    })
    .unwrap();
    drop(exact);
}

pub(in crate::document_scripting) fn attached_registered_function_uses_the_exact_native_invocation()
{
    let cancel = Arc::new(AtomicBool::new(false));
    let mut host = host(cancel);
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = [0xd800];
    let event = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let (first, second, thrown) = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            let target = target.object(scope)?;
            let event = event.object(scope)?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedTarget")),
                &target,
            )?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedEvent")),
                &event,
            )?;
            let first = scope.evaluate(&units(
                "globalThis.marker = {}; globalThis.trace = []; (function(e) {
                if (this !== expectedTarget || e !== expectedEvent) throw 1;
                trace.push(1); throw marker;
            })",
            ))?;
            let second = scope.evaluate(&units(
                "(function(e) {
            if (this !== expectedTarget || e !== expectedEvent) throw 2;
            trace.push(2);
        })",
            ))?;
            let thrown = scope.get(&global, PropertyKey::String(&units("marker")))?;
            Ok((
                scope.retain(&first),
                scope.retain(&second),
                scope.retain(&thrown),
            ))
        })
        .unwrap();
    let (from_other_scope, unattached) = {
        let engine = host.engine.as_mut().unwrap();
        let mut execution = engine.process.execution();
        let other = execution.create_realm().unwrap();
        let from_other_scope = execution
            .in_realm(&other, |scope| in_realm_root(scope, &first))
            .unwrap();
        let unattached = execution
            .in_realm(&other, |scope| {
                let callback = scope.evaluate(&units("(function() { throw 3 })"))?;
                Ok(scope.retain(&callback))
            })
            .unwrap();
        let unattached = execution
            .in_realm(&engine.realm, |scope| in_realm_root(scope, &unattached))
            .unwrap();
        (from_other_scope, unattached)
    };
    let once = DomEventListenerOptions {
        once: true,
        ..Default::default()
    };
    assert!(matches!(
        host.add_guest_event_listener(&permit, &target, &kind, &unattached, once),
        Err(DocumentGuestEventError::CallbackRealmUnavailable)
    ));
    let first_id = host
        .add_guest_event_listener(&permit, &target, &kind, &from_other_scope, once)
        .unwrap();
    assert_eq!(
        host.add_guest_event_listener(
            &permit,
            &target,
            &kind,
            &first,
            DomEventListenerOptions::default()
        )
        .unwrap(),
        first_id
    );
    let second_id = host
        .add_guest_event_listener(
            &permit,
            &target,
            &kind,
            &second,
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 2);
    assert_eq!(report.receipt().event_type().code_units(), kind);
    assert_eq!(report.receipt().failures().len(), 1);
    let [exception] = report.exceptions() else {
        panic!("exactly one ordinary exception")
    };
    assert_eq!(exception.registration, first_id);
    assert_eq!(exception.realm, permit.realm);
    host.in_document(&permit, |scope| {
        let original = scope.import(&thrown)?;
        let reported = scope.import(exception.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        let trace = scope.evaluate(&units("trace.join(',') === '1,2'"))?;
        let yes = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&trace, &yes)?);
        Ok(())
    })
    .unwrap();
    let events = host.guest_events().unwrap();
    let escaped = exception.exception.value().clone();
    drop(report);
    host.in_document(&permit, |scope| {
        let expected = scope.import(&thrown)?;
        let escaped = scope.import(&escaped)?;
        assert!(scope.strict_equals(&expected, &escaped)?);
        Ok(())
    })
    .unwrap();
    let before_root_reap = events.budget.used.get();
    drop(escaped);
    assert_eq!(events.budget.used.get(), before_root_reap);
    host.in_document(&permit, |_| Ok(())).unwrap();
    assert_eq!(
        before_root_reap - events.budget.used.get(),
        events.exception_root_bytes + size_of::<GuestBacking>()
    );
    assert!(!host.remove_guest_event_listener(&permit, first_id).unwrap());
    assert!(
        host.remove_guest_event_listener(&permit, second_id)
            .unwrap()
    );
    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 0);
    assert!(report.exceptions().is_empty());
    assert_eq!(event.native().current_target(), None);
    assert_eq!(event.native().composed_path().count(), 0);
}

fn proxy_in_scope(scope: &mut Scope<'_>, target: &Root) -> Result<Root, Error> {
    let target = scope.import(target)?;
    let create = scope.evaluate(&units(
        "globalThis.proxyReads = 0; globalThis.proxyCalls = 0;
        (target => new Proxy(target, {
            get(t, k, r) { proxyReads++; return Reflect.get(t, k, r); },
            getPrototypeOf(t) { proxyReads++; return Reflect.getPrototypeOf(t); },
            apply(t, r, args) { proxyCalls++; return Reflect.apply(t, r, args); }
        }))",
    ))?;
    let receiver = scope.primitive(Primitive::Undefined)?;
    let proxy = scope.call(&create, &receiver, &[target])?;
    Ok(scope.retain(&proxy))
}

fn assert_guest(scope: &mut Scope<'_>, expression: &str) -> Result<(), Error> {
    let observed = scope.evaluate(&units(expression))?;
    let yes = scope.primitive(Primitive::Boolean(true))?;
    assert!(scope.strict_equals(&observed, &yes)?, "{expression}");
    Ok(())
}

pub(in crate::document_scripting) fn nested_guest_reports_keep_observation_order_and_depth() {
    for depth in [1, 2] {
        let mut host = host(Arc::new(AtomicBool::new(false)));
        let permit = host.permit().unwrap();
        attach_registry_with_limits(&mut host, &permit, depth, 32);
        let target = host.create_guest_event_target(&permit).unwrap();
        let outer = host
            .create_guest_event(&permit, &units("outer"), Default::default())
            .unwrap();
        let inner = host
            .create_guest_event(&permit, &units("inner"), Default::default())
            .unwrap();
        let callbacks = host
            .in_document(&permit, |scope| {
                let global = scope.global()?;
                for (key, value) in [
                    ("target", target.object(scope)?),
                    ("outer", outer.object(scope)?),
                    ("inner", inner.object(scope)?),
                ] {
                    scope.set(&global, PropertyKey::String(&units(key)), &value)?;
                }
                scope.evaluate(&units("globalThis.first = {}; globalThis.second = {};"))?;
                let outer_throw = scope.evaluate(&units("(() => { throw first; })"))?;
                let nested = scope.evaluate(&units(
                    "(() => {
                if (!target.dispatchEvent(inner) || globalThis.event !== outer) throw 5;
            })",
                ))?;
                let inner_throw = scope.evaluate(&units("(() => { throw second; })"))?;
                Ok([
                    scope.retain(&outer_throw),
                    scope.retain(&nested),
                    scope.retain(&inner_throw),
                ])
            })
            .unwrap();
        let registrations = ["outer", "outer", "inner"]
            .map(units)
            .into_iter()
            .zip(&callbacks)
            .map(|(kind, listener)| {
                host.add_guest_event_listener(&permit, &target, &kind, listener, Default::default())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let result = host.in_document(&permit, |scope| {
            assert_guest(
                scope,
                "target.dispatchEvent(outer) && globalThis.event === undefined",
            )
        });
        let first = host
            .take_guest_listener_exception(&permit)
            .unwrap()
            .unwrap();
        assert_eq!(first.registration, registrations[0]);
        if depth == 2 {
            result.unwrap();
            let second = host
                .take_guest_listener_exception(&permit)
                .unwrap()
                .unwrap();
            assert_eq!(second.registration, registrations[2]);
            host.in_document(&permit, |scope| {
                let global = scope.global()?;
                for (key, report) in [("first", first), ("second", second)] {
                    let original = scope.get(&global, PropertyKey::String(&units(key)))?;
                    let observed = scope.import(report.exception.value())?;
                    assert!(scope.strict_equals(&original, &observed)?);
                }
                Ok(())
            })
            .unwrap();
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("depth one must refuse the actual inner invocation")
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::Dispatch(
                    DomEventDispatchError::Admission(crate::DomEventResource::NestedDispatches),
                ))
            ));
            assert!(inner.native().target().is_none());
            assert!(host.engine.as_ref().unwrap().window_event.is_idle());
        }
        assert!(
            host.take_guest_listener_exception(&permit)
                .unwrap()
                .is_none()
        );
        assert_eq!(outer.native().current_target(), None);
        assert_eq!(outer.native().composed_path().count(), 0);
    }
}

pub(in crate::document_scripting) fn full_guest_report_bank_preserves_original_registry_work() {
    #[derive(Debug, PartialEq, Eq)]
    enum Refusal {
        ReportStorage,
        RegistryWork,
    }

    // Standalone construction costs one registry unit. First registration of
    // custom one-unit `x` costs six static name lengths and one copied unit.
    // A successful guest dispatch costs one unit of fixed native preparation.
    // A second standalone construction is an independent one-unit probe.
    let mut observed = Vec::new();
    for (work, attempt_refused_dispatch) in [(10, false), (10, true), (9, true)] {
        let cancel = Arc::new(AtomicBool::new(false));
        let mut host = host(cancel.clone());
        let permit = host.permit().unwrap();
        let global = host.attachment().unwrap().target.global();
        let mut registry = Some(
            DomEventDispatcher::new(DomEventRegistryLimits {
                registrations: 1,
                pending_events: 2,
                nested_dispatches: 1,
                storage_bytes: 1_000_000,
                work,
            })
            .unwrap(),
        );
        host.attach_guest_event_registry(
            &permit,
            &mut registry,
            DocumentGuestEventLimits {
                storage_bytes: 32_000,
                work: 1000,
                pending_exception_reports: 1,
                node_retirement: DomNodeRetirementLimits::new(16_000_000, 1_000),
                source_edits: DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
                dispatch: DomEventDispatchLimits {
                    path_items: 1,
                    snapshot_listeners: 1,
                    failure_records: 1,
                    callbacks: 1,
                    work: 10_000,
                },
            },
        )
        .unwrap();
        assert!(registry.is_none());
        let target = host.create_guest_event_target(&permit).unwrap();
        let kind = units("x");
        let first = host
            .create_guest_event(&permit, &kind, Default::default())
            .unwrap();
        let blocked = host
            .create_guest_event(&permit, &kind, Default::default())
            .unwrap();
        let listener = host
            .in_document(&permit, |scope| {
                let global = scope.global()?;
                for (key, value) in [
                    ("target", target.object(scope)?),
                    ("first", first.object(scope)?),
                    ("blocked", blocked.object(scope)?),
                ] {
                    scope.set(&global, PropertyKey::String(&units(key)), &value)?;
                }
                scope.evaluate(&units("globalThis.calls = 0; globalThis.marker = {};"))?;
                let listener = scope.evaluate(&units("(() => { calls++; throw marker; })"))?;
                Ok(scope.retain(&listener))
            })
            .unwrap();
        let registration = host
            .add_guest_event_listener(
                &permit,
                &target,
                &kind,
                &listener,
                DomEventListenerOptions {
                    passive: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
        host.in_document(&permit, |scope| {
            assert_guest(scope, "target.dispatchEvent(first) === true && calls === 1 && globalThis.event === undefined")
        }).unwrap();
        assert!(first.native().target().is_some());
        let refused = if attempt_refused_dispatch {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) =
                host.in_document(&permit, |scope| {
                    scope.evaluate(&units(
                        "try { target.dispatchEvent(blocked); } catch (_) { throw 7; }",
                    ))?;
                    Ok(())
                })
            else {
                panic!("full report bank must refuse the actual guest method")
            };
            Some(match failure.cause::<DocumentGuestEventError>() {
                Some(DocumentGuestEventError::Document(DocumentScriptError::Native(
                    DocumentScriptNativeError::StorageLimit,
                ))) => Refusal::ReportStorage,
                Some(DocumentGuestEventError::Dispatch(DomEventDispatchError::Admission(
                    crate::DomEventResource::Work,
                ))) => Refusal::RegistryWork,
                cause => panic!("unexpected original dispatch refusal: {cause:?}"),
            })
        } else {
            None
        };
        assert!(blocked.native().target().is_none());
        let original = host
            .take_guest_listener_exception(&permit)
            .unwrap()
            .unwrap();
        assert_eq!(original.registration, registration);
        assert_eq!(original.realm, permit.realm);
        assert!(
            host.take_guest_listener_exception(&permit)
                .unwrap()
                .is_none()
        );

        // Enter the original native owner directly after the sticky guest
        // refusal. This deliberately does not request any guest execution.
        let events = host.guest_events().unwrap();
        let probe = host
            .with_native(|dom, mut hosted, _| {
                hosted.create_event_target(dom, &events.registry.borrow(), global, &cancel)
            })
            .unwrap();
        let probe_admitted = match probe {
            Ok(_) => true,
            Err(DomEventConstructionError::Event(DomEventDispatchError::Admission(
                crate::DomEventResource::Work,
            ))) => false,
            Err(other) => panic!("unexpected native work-probe refusal: {other:?}"),
        };
        observed.push((refused, probe_admitted));
    }
    assert_eq!(
        observed,
        [
            (None, true),
            (Some(Refusal::ReportStorage), true),
            (Some(Refusal::ReportStorage), false),
        ]
    );
}

pub(in crate::document_scripting) fn full_guest_report_reservation_keeps_exact_invalid_state() {
    let cancel = Arc::new(AtomicBool::new(false));
    let mut host = host(cancel.clone());
    let permit = host.permit().unwrap();
    attach_registry_with_limits(&mut host, &permit, 4, 8);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = units("active");
    let event = host
        .create_guest_event(&permit, &kind, Default::default())
        .unwrap();
    let empty = host
        .create_guest_event(&permit, &[], Default::default())
        .unwrap();
    let legacy = host
        .with_native(|dom, mut hosted, _| {
            hosted.create_legacy_event(dom, permit.document(), &units("Event"), &cancel)
        })
        .unwrap()
        .unwrap();
    let events = host.guest_events().unwrap();
    let owner = host.native.clone();
    let listener = host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let legacy = events.project(&owner, scope, ObjectPrototype::Default, GuestEventProjection(legacy.clone())).unwrap();
        for (key, value) in [
            ("target", target.object(scope)?),
            ("active", event.object(scope)?),
            ("legacy", scope.open_callback_object(&legacy)?),
            ("empty", empty.object(scope)?),
        ] {
            scope.set(&global, PropertyKey::String(&units(key)), &value)?;
        }
        scope.evaluate(&units("globalThis.checked = 0;"))?;
        let listener = scope.evaluate(&units(
            "(function(e) {
                if (e !== active || globalThis.event !== e) throw 1;
                for (const candidate of [e, legacy]) {
                    try {
                        target.dispatchEvent(candidate);
                        throw 2;
                    } catch (error) {
                        if (!(error instanceof DOMException) || error.name !== 'InvalidStateError') throw error;
                        checked++;
                    }
                    if (globalThis.event !== e) throw 3;
                }
            })",
        ))?;
        Ok(scope.retain(&listener))
    }).unwrap();
    host.add_guest_event_listener(&permit, &target, &kind, &listener, Default::default())
        .unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(scope, "target.dispatchEvent(active) === true && checked === 2 && globalThis.event === undefined")?;
        assert_guest(scope, "target.dispatchEvent(empty) === true")
    }).unwrap();
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    assert!(legacy.target().is_none());
    assert!(empty.native().target().is_some());
}

pub(in crate::document_scripting) fn direct_guest_dispatch_preserves_reports_and_object_custody() {
    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry_with_limits(&mut host, &permit, 4, 8);
    let left = host.create_guest_event_target(&permit).unwrap();
    let right = host.create_guest_event_target(&permit).unwrap();
    let kind = units("reported");
    let event = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let blocked = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let (listener, marker) = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            for (key, value) in [
                ("left", left.object(scope)?),
                ("right", right.object(scope)?),
                ("expected", event.object(scope)?),
                ("blocked", blocked.object(scope)?),
            ] {
                scope.set(&global, PropertyKey::String(&units(key)), &value)?;
            }
            scope.evaluate(&units("globalThis.marker = {}; globalThis.calls = 0;"))?;
            let marker = scope.get(&global, PropertyKey::String(&units("marker")))?;
            let listener = scope.evaluate(&units(
                "(function(e) {
            if (this !== right || e !== expected || globalThis.event !== e) throw 17;
            calls++;
            throw marker;
        })",
            ))?;
            Ok((scope.retain(&listener), scope.retain(&marker)))
        })
        .unwrap();
    let id = host
        .add_guest_event_listener(&permit, &right, &kind, &listener, Default::default())
        .unwrap();
    drop((left, right, event));
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "(() => {
            const dispatch = left.dispatchEvent;
            let traps = 0;
            const handler = { get() { traps++; }, getPrototypeOf() { traps++; } };
            for (const [receiver, argument] of [
                [new Proxy(right, handler), expected], [Object.create(right), expected],
                [right, new Proxy(expected, handler)], [right, Object.create(expected)], [right, 1],
            ]) {
                try { dispatch.call(receiver, argument); return false; }
                catch (error) { if (!(error instanceof TypeError)) return false; }
            }
            return traps === 0 && calls === 0 && dispatch.call(right, expected) === true &&
                   calls === 1 && globalThis.event === undefined;
        })()",
        )
    })
    .unwrap();
    let original = host
        .take_guest_listener_exception(&permit)
        .unwrap()
        .unwrap();
    assert_eq!(original.registration, id);
    host.in_document(&permit, |scope| {
        let expected = scope.import(&marker)?;
        let reported = scope.import(original.exception.value())?;
        assert!(scope.strict_equals(&expected, &reported)?);
        Ok(())
    })
    .unwrap();
    let events = host.guest_events().unwrap();
    let before_root_reap = events.budget.used.get();
    drop(original);
    assert_eq!(events.budget.used.get(), before_root_reap);
    host.in_document(&permit, |_| Ok(())).unwrap();
    assert_eq!(
        before_root_reap - events.budget.used.get(),
        events.exception_root_bytes + size_of::<GuestBacking>()
    );
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "right.dispatchEvent(expected) === true && calls === 2",
        )
    })
    .unwrap();
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) =
        host.in_document(&permit, |scope| {
            scope.evaluate(&units(
                "try { right.dispatchEvent(blocked); } catch (_) { throw 7; }",
            ))?;
            Ok(())
        })
    else {
        panic!("the backlog must refuse before dispatch")
    };
    let cause = failure.cause::<DocumentGuestEventError>().unwrap();
    assert!(matches!(
        cause,
        DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::StorageLimit
        ))
    ));
    assert!(blocked.native().target().is_none());
    let prior = host
        .take_guest_listener_exception(&permit)
        .unwrap()
        .unwrap();
    assert_eq!(prior.registration, id);
    assert_eq!(prior.realm, permit.realm);
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    let Err(DocumentScriptError::Script(Error::HostFailure(again))) = host.checkpoint() else {
        panic!("the original terminal cause must survive observation")
    };
    assert!(std::ptr::eq(
        cause,
        again.cause::<DocumentGuestEventError>().unwrap()
    ));
}

pub(in crate::document_scripting) fn attached_window_event_follows_the_listener_invocation() {
    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = units("current");
    let event = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let (throwing, normal, original_exception) = host
        .in_document(&permit, |scope| {
            assert_guest(scope, "globalThis.event === undefined")?;
            let global = scope.global()?;
            let projected = event.object(scope)?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedEvent")),
                &projected,
            )?;
            scope.evaluate(&units(
                "globalThis.observed = [];
                 globalThis.replacement = {};
                 globalThis.marker = {};
                 globalThis.currentGetter = Object.getOwnPropertyDescriptor(globalThis, 'event')?.get;
                 globalThis.recordEvent = e => observed.push([
                     e === expectedEvent,
                     globalThis.event === e,
                     globalThis.event === replacement,
                     currentGetter?.call(globalThis) === e,
                 ]);",
            ))?;
            let throwing = scope.evaluate(&units("(e => { recordEvent(e); throw marker; })"))?;
            let normal = scope.evaluate(&units("(e => { recordEvent(e); })"))?;
            let marker = scope.get(&global, PropertyKey::String(&units("marker")))?;
            Ok((
                scope.retain(&throwing),
                scope.retain(&normal),
                scope.retain(&marker),
            ))
        })
        .unwrap();

    let throwing_id = host
        .add_guest_event_listener(
            &permit,
            &target,
            &kind,
            &throwing,
            DomEventListenerOptions {
                once: true,
                ..Default::default()
            },
        )
        .unwrap();
    let thrown = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(thrown.receipt().callbacks(), 1);
    assert_eq!(thrown.receipt().failures().len(), 1);
    let [exception] = thrown.exceptions() else {
        panic!("the ordinary listener exception was reported")
    };
    assert_eq!(exception.registration, throwing_id);
    host.in_document(&permit, |scope| {
        let original = scope.import(&original_exception)?;
        let reported = scope.import(exception.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        assert_guest(scope, "observed.length === 1 && observed[0][0]")?;
        assert_guest(scope, "observed[0][1] && !observed[0][2] && observed[0][3]")?;
        assert_guest(
            scope,
            "globalThis.event === undefined && currentGetter.call(globalThis) === undefined",
        )
    })
    .unwrap();

    host.add_guest_event_listener(&permit, &target, &kind, &normal, Default::default())
        .unwrap();
    let completed = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(completed.receipt().callbacks(), 1);
    assert!(completed.receipt().failures().is_empty());
    assert!(completed.exceptions().is_empty());
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "observed.length === 2 && observed[1][0] && observed[1][1] &&
             !observed[1][2] && observed[1][3]",
        )?;
        assert_guest(
            scope,
            "globalThis.event === undefined && currentGetter.call(globalThis) === undefined",
        )?;
        scope.evaluate(&units("globalThis.event = replacement"))?;
        assert_guest(
            scope,
            "(() => {
                 const property = Object.getOwnPropertyDescriptor(globalThis, 'event');
                 return property.value === replacement && property.writable &&
                        property.enumerable && property.configurable;
             })()",
        )
    })
    .unwrap();

    let shadowed = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(shadowed.receipt().callbacks(), 1);
    assert!(shadowed.receipt().failures().is_empty());
    assert!(shadowed.exceptions().is_empty());
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "observed.length === 3 && observed[2][0] && !observed[2][1] &&
             observed[2][2] && observed[2][3]",
        )?;
        assert_guest(
            scope,
            "globalThis.event === replacement && currentGetter.call(globalThis) === undefined",
        )
    })
    .unwrap();
}

pub(in crate::document_scripting) fn attached_window_event_clears_on_terminal_listener_refusal() {
    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = units("terminal");
    let event = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let ordinary = host
        .in_document(&permit, |scope| {
            let callback = scope.evaluate(&units(
                "(function(e) { if (globalThis.event !== e) return; throw { ordinary: true }; })",
            ))?;
            Ok(scope.retain(&callback))
        })
        .unwrap();
    let ordinary_id = host
        .add_guest_event_listener(&permit, &target, &kind, &ordinary, Default::default())
        .unwrap();
    let callback = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            let projected = event.object(scope)?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedEvent")),
                &projected,
            )?;
            let callback = scope.evaluate(&units(
                "({ get handleEvent() {
                    if (globalThis.event !== expectedEvent) throw new Error('getter Event');
                    return function(e) {
                        if (globalThis.event !== e || e !== expectedEvent) throw new Error('method Event');
                        try { detachIf(); } catch (_) { throw new Error('caught host refusal'); }
                    };
                } })",
            ))?;
            Ok(scope.retain(&callback))
        })
        .unwrap();
    host.add_guest_event_listener(&permit, &target, &kind, &callback, Default::default())
        .unwrap();
    let window_event = host.engine.as_ref().unwrap().window_event.clone();
    let refused = match host.dispatch_guest_event(&permit, &target, &event) {
        Ok(_) => panic!("the selected object listener must reach the real terminal Source refusal"),
        Err(refused) => refused,
    };
    let [reported] = refused.exceptions() else {
        panic!("the earlier ordinary exception must survive terminal refusal");
    };
    assert_eq!(reported.registration, ordinary_id);
    assert_eq!(reported.realm, permit.realm);
    assert_eq!(reported.exception.phase(), ExceptionPhase::Invoke);
    let DocumentGuestEventError::Document(DocumentScriptError::Script(Error::HostFailure(failure))) =
        &refused.error
    else {
        panic!("the original terminal Source refusal must be returned");
    };
    let cause = failure.cause::<DocumentScriptNativeError>().unwrap();
    assert!(matches!(cause, DocumentScriptNativeError::SourceAccess));
    assert!(window_event.is_idle());
    assert_eq!(event.native().current_target(), None);
    assert_eq!(event.native().composed_path().count(), 0);
    let Err(DocumentScriptError::Script(Error::HostFailure(again))) = host.checkpoint() else {
        panic!("the original sticky refusal must survive dispatch cleanup")
    };
    assert!(std::ptr::eq(
        cause,
        again.cause::<DocumentScriptNativeError>().unwrap()
    ));
}

pub(in crate::document_scripting) fn attached_guest_dispatch_restores_the_outer_window_event() {
    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = units("nested");
    let outer = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let inner = host
        .create_guest_event(
            &permit,
            &kind,
            DomEventInit {
                cancelable: true,
                ..Default::default()
            },
        )
        .unwrap();
    inner.native().prevent_default();
    let (once, listener, original_exception) = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            for (key, value) in [
                ("target", target.object(scope)?),
                ("outer", outer.object(scope)?),
                ("inner", inner.object(scope)?),
            ] {
                scope.set(&global, PropertyKey::String(&units(key)), &value)?;
            }
            scope.evaluate(&units(
                "globalThis.trace = [];
                 globalThis.marker = {};
                 globalThis.check = (condition, detail) => { if (!condition) throw new Error(detail); };",
            ))?;
            assert_guest(scope, "globalThis.event === undefined && outer !== inner")?;
            let once = scope.evaluate(&units(
                "(function(e) {
                    check(this === target && e === outer && globalThis.event === e, 'outer entry');
                    trace.push('once:outer');
                    check(target.dispatchEvent(inner) === false, 'inner native cancellation');
                    check(globalThis.event === e, 'outer current event restored');
                    trace.push('restored:outer');
                    try {
                        target.dispatchEvent(e);
                        throw new Error('redispatched active Event');
                    } catch (error) {
                        check(error instanceof DOMException && error.name === 'InvalidStateError', 'active Event refusal');
                    }
                    check(globalThis.event === e, 'active Event refusal retained outer scope');
                    throw marker;
                })",
            ))?;
            let listener = scope.evaluate(&units(
                "({ get handleEvent() {
                    check(globalThis.event === inner || globalThis.event === outer, 'late lookup scope');
                    return function(e) {
                        check(this === listener && globalThis.event === e, 'object listener scope');
                        check(e === inner || e === outer, 'exact projected Event');
                        trace.push(e === inner ? 'object:inner' : 'object:outer');
                    };
                } })",
            ))?;
            scope.set(&global, PropertyKey::String(&units("listener")), &listener)?;
            let marker = scope.get(&global, PropertyKey::String(&units("marker")))?;
            Ok((scope.retain(&once), scope.retain(&listener), scope.retain(&marker)))
        })
        .unwrap();
    let once_id = host
        .add_guest_event_listener(
            &permit,
            &target,
            &kind,
            &once,
            DomEventListenerOptions {
                once: true,
                ..Default::default()
            },
        )
        .unwrap();
    host.add_guest_event_listener(&permit, &target, &kind, &listener, Default::default())
        .unwrap();
    let report = host.dispatch_guest_event(&permit, &target, &outer).unwrap();
    assert_eq!(report.receipt().callbacks(), 2);
    assert_eq!(report.receipt().failures().len(), 1);
    assert!(!report.receipt().default_prevented());
    let [exception] = report.exceptions() else {
        panic!("outer listener's original exception")
    };
    assert_eq!(exception.registration, once_id);
    host.in_document(&permit, |scope| {
        assert_guest(scope, "trace[0] === 'once:outer'")?;
        let trace = scope.evaluate(&units("trace.join(',')"))?;
        assert_eq!(
            String::from_utf16_lossy(&scope.to_utf16(&trace)?),
            "once:outer,object:inner,restored:outer,object:outer"
        );
        assert_guest(scope, "globalThis.event === undefined")?;
        let original = scope.import(&original_exception)?;
        let reported = scope.import(exception.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        Ok(())
    })
    .unwrap();
    assert!(!host.remove_guest_event_listener(&permit, once_id).unwrap());
    for event in [&inner, &outer] {
        assert_eq!(
            event.native().target(),
            Some(report.receipt().original_target())
        );
        assert_eq!(event.native().current_target(), None);
        assert_eq!(event.native().composed_path().count(), 0);
    }
}

pub(in crate::document_scripting) fn attached_object_listeners_look_up_handle_event_when_invoked() {
    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = [u16::from(b'x')];
    let event = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let callbacks = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            let target = target.object(scope)?;
            let event = event.object(scope)?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedTarget")),
                &target,
            )?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedEvent")),
                &event,
            )?;
            scope.evaluate(&units(
                "globalThis.trace = [];
            globalThis.getterMarker = {};
            globalThis.callMarker = {};
            globalThis.attachedTypeError = TypeError;
            globalThis.TypeError = function ReplacementTypeError() {};
            globalThis.normalMethod = 7;
            globalThis.normalListener = {
                get handleEvent() {
                    if (this !== normalListener) throw 1;
                    trace.push('normal-get');
                    return normalMethod;
                }
            };
            globalThis.getterFailure = {
                get handleEvent() {
                    if (this !== getterFailure) throw 2;
                    trace.push('throw-get');
                    throw getterMarker;
                }
            };
            globalThis.callFailure = {
                get handleEvent() {
                    if (this !== callFailure) throw 3;
                    trace.push('call-get');
                    return function(e) {
                        if (this !== callFailure || e !== expectedEvent) throw 4;
                        trace.push('call-throw');
                        throw callMarker;
                    };
                }
            };
            globalThis.noncallable = {
                get handleEvent() { trace.push('number-get'); return 17; }
            };
            globalThis.later = function(e) {
                if (this !== expectedTarget || e !== expectedEvent) throw 5;
                trace.push('later');
            };",
            ))?;
            let mut root = |name| {
                let value = scope.get(&global, PropertyKey::String(&units(name)))?;
                Ok::<_, Error>(scope.retain(&value))
            };
            Ok([
                root("normalListener")?,
                root("getterFailure")?,
                root("callFailure")?,
                root("noncallable")?,
                root("later")?,
            ])
        })
        .unwrap();
    let attempted = callbacks.each_ref().map(|callback| {
        host.add_guest_event_listener(
            &permit,
            &target,
            &kind,
            callback,
            DomEventListenerOptions::default(),
        )
    });
    let duplicate = host
        .in_document(&permit, |scope| {
            assert_guest(scope, "trace.length === 0")?;
            let original = scope.evaluate(&units("normalListener"))?;
            Ok(scope.retain(&original))
        })
        .unwrap();
    let [normal, getter, caller, number, later] = attempted.map(|registration| {
        registration
            .unwrap_or_else(|error| panic!("attached EventListener registration failed: {error:?}"))
    });
    let replacement_options = DomEventListenerOptions {
        once: true,
        ..Default::default()
    };
    assert_eq!(
        host.add_guest_event_listener(&permit, &target, &kind, &duplicate, replacement_options)
            .unwrap(),
        normal
    );
    host.in_document(&permit, |scope| {
        assert_guest(scope, "trace.length === 0")?;
        scope.evaluate(&units(
            "normalMethod = function(e) {
                if (this !== normalListener || e !== expectedEvent) throw 6;
                trace.push('normal-call');
            };",
        ))?;
        Ok(())
    })
    .unwrap();

    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 5);
    assert_eq!(
        report
            .receipt()
            .failures()
            .iter()
            .map(|failure| failure.registration)
            .collect::<Vec<_>>(),
        [getter, caller, number]
    );
    let [getter_error, call_error, not_callable] = report.exceptions() else {
        panic!("expected exactly three ordinary listener exceptions");
    };
    assert_eq!(getter_error.registration, getter);
    assert_eq!(getter_error.exception.phase(), ExceptionPhase::PropertyRead);
    assert_eq!(call_error.registration, caller);
    assert_eq!(call_error.exception.phase(), ExceptionPhase::Invoke);
    assert_eq!(not_callable.registration, number);
    assert_eq!(not_callable.realm, permit.realm);
    host.in_document(&permit, |scope| {
        for (error, marker) in [(getter_error, "getterMarker"), (call_error, "callMarker")] {
            let original = scope.evaluate(&units(marker))?;
            let reported = scope.import(error.exception.value())?;
            assert!(scope.strict_equals(&original, &reported)?);
        }
        let type_error = scope.import(not_callable.exception.value())?;
        let check = scope.evaluate(&units(
            "value => value instanceof attachedTypeError && !(value instanceof TypeError)",
        ))?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        let is_original_intrinsic = scope.call(&check, &undefined, &[type_error])?;
        let yes = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&is_original_intrinsic, &yes)?);
        assert_guest(scope,
            "trace.join(',') === 'normal-get,normal-call,throw-get,call-get,call-throw,number-get,later'",
        )
    }).unwrap();
    for registration in [normal, getter, caller, number, later] {
        assert!(
            host.remove_guest_event_listener(&permit, registration)
                .unwrap()
        );
    }
    assert!(!host.remove_guest_event_listener(&permit, normal).unwrap());
    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 0);
    assert!(report.exceptions().is_empty());
}

pub(in crate::document_scripting) fn attached_proxy_uses_its_own_realm() {
    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = [u16::from(b'x')];
    let event = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let in_attached = host
        .in_document(&permit, |scope| {
            let callable = scope.evaluate(&units(
                "globalThis.targetCalls = 0; (() => { targetCalls++; })",
            ))?;
            Ok(scope.retain(&callable))
        })
        .unwrap();
    let (other, allowed, denied) = {
        let engine = host.engine.as_mut().unwrap();
        let mut execution = engine.process.execution();
        let other = execution.create_realm().unwrap();
        let in_other = execution
            .in_realm(&other, |scope| {
                let global = scope.global()?;
                let target = target.object(scope)?;
                let event = event.object(scope)?;
                scope.set(
                    &global,
                    PropertyKey::String(&units("expectedTarget")),
                    &target,
                )?;
                scope.set(
                    &global,
                    PropertyKey::String(&units("expectedEvent")),
                    &event,
                )?;
                let callable = scope.evaluate(&units(
                    "globalThis.targetCalls = 0; (function(e) {
                    if (this !== expectedTarget || e !== expectedEvent) throw 1;
                    targetCalls++;
                })",
                ))?;
                Ok(scope.retain(&callable))
            })
            .unwrap();
        let allowed = execution
            .in_realm(&engine.realm, |scope| proxy_in_scope(scope, &in_other))
            .unwrap();
        let denied = execution
            .in_realm(&other, |scope| proxy_in_scope(scope, &in_attached))
            .unwrap();
        let allowed = execution
            .in_realm(&other, |scope| in_realm_root(scope, &allowed))
            .unwrap();
        let denied = execution
            .in_realm(&engine.realm, |scope| in_realm_root(scope, &denied))
            .unwrap();
        (other, allowed, denied)
    };
    let allow = host.add_guest_event_listener(
        &permit,
        &target,
        &kind,
        &allowed,
        DomEventListenerOptions::default(),
    );
    let deny = host.add_guest_event_listener(
        &permit,
        &target,
        &kind,
        &denied,
        DomEventListenerOptions::default(),
    );
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "proxyReads === 0 && proxyCalls === 0 && targetCalls === 0",
        )
    })
    .unwrap();
    {
        let engine = host.engine.as_mut().unwrap();
        engine
            .process
            .execution()
            .in_realm(&other, |scope| {
                assert_guest(
                    scope,
                    "proxyReads === 0 && proxyCalls === 0 && targetCalls === 0",
                )
            })
            .unwrap();
    }
    assert!(
        allow.is_ok() && matches!(deny, Err(DocumentGuestEventError::CallbackRealmUnavailable)),
        "accept the attached Proxy and refuse the unattached one: attached {allow:?}, unattached {deny:?}"
    );
    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 1);
    assert!(report.receipt().failures().is_empty());
    assert!(report.exceptions().is_empty());
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "proxyReads === 0 && proxyCalls === 1 && targetCalls === 0",
        )
    })
    .unwrap();
    let engine = host.engine.as_mut().unwrap();
    engine
        .process
        .execution()
        .in_realm(&other, |scope| {
            assert_guest(
                scope,
                "proxyReads === 0 && proxyCalls === 0 && targetCalls === 1",
            )
        })
        .unwrap();
}

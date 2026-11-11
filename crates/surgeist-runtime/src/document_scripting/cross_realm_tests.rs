use super::*;
use crate::{
    DomElementTaskRuntimeLimits, DomEventCallback, DomEventDispatchError, DomEventDispatchLimits,
    DomEventDispatcher, DomEventHostError, DomEventListenerOptions, DomEventRegistryLimits,
    DomEventStorageLimits, DomEventType, DomHostedRuntimeLimits, DomHostedTaskProgress,
    DomSessionLimits, DomShutdownOutcome, DomSourceDriveProgress, DomSourceEventRuntimeError,
    DomSourceEventRuntimeLimits, DomSourceEventServiceLimits, DomSourceEventServiceProgress,
    DomSourceExecutionScope, DomSourceRuntimeError, DomSourceServices, DomSourceUnavailable,
    DomStyleRuntimeLimits,
};
use surgeist_bindings_js::{Root, Value};
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_dom::{
    Dom, DomBatch, DomDispatchCustodyLimits, DomHostedLimits, DomHostedRecordLimits, DomLimits,
    DomNameLimits, DomSourceEdit, DomSourceEventRequest, DomSourceLimits, DomSourceRequest, NodeId,
};
use surgeist_html::values::Namespace;

pub(super) fn hosted_session(
    cancel: &AtomicBool,
    source_jobs: usize,
) -> (DomSession, DomHostedTopLevel) {
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
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ),
    )
    .unwrap();
    let mut session = DomSession::new(
        dom,
        DomSessionLimits {
            authored_events: DomEventStorageLimits {
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            source_events: DomSourceEventRuntimeLimits {
                jobs: source_jobs,
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
    let (session, target) = hosted_session(&cancel, 0);
    let read_name = "readChildCount".encode_utf16().collect::<Vec<_>>();
    let mut host = enable(session, target, cancel.clone());
    let permit = host.permit().unwrap();
    let (hook, escape, window_type_error) = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            let callback = scope.get(&global, PropertyKey::String(&read_name))?;
            let escape = scope.evaluate(&"CSS.escape".encode_utf16().collect::<Vec<_>>())?;
            let type_error = scope.evaluate(&"TypeError".encode_utf16().collect::<Vec<_>>())?;
            Ok((
                scope.retain(&callback),
                scope.retain(&escape),
                scope.retain(&type_error),
            ))
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
                let escape = scope.import(&escape)?;
                let window_type_error = scope.import(&window_type_error)?;
                scope.set(
                    &global,
                    PropertyKey::String(&"windowEscape".encode_utf16().collect::<Vec<_>>()),
                    &escape,
                )?;
                scope.set(
                    &global,
                    PropertyKey::String(&"windowTypeError".encode_utf16().collect::<Vec<_>>()),
                    &window_type_error,
                )?;
                let observed = scope.evaluate(
                    &r#"(() => {
                        let fromWindow = false;
                        try { windowEscape(); }
                        catch (error) {
                            fromWindow = error instanceof windowTypeError
                                && !(error instanceof TypeError);
                        }
                        return !Object.hasOwn(globalThis, "CSS") && fromWindow
                            && windowEscape("\ud800") === "\ud800";
                    })()"#
                        .encode_utf16()
                        .collect::<Vec<_>>(),
                )?;
                let expected = scope.primitive(Primitive::Boolean(true))?;
                assert!(scope.strict_equals(&observed, &expected)?);
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
    drop(window_type_error);
    drop(escape);
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
    drop(session);
    projected_comment_custody_outlives_collection_and_the_origin_realm_token();
    rejected_comment_requires_each_diagnostic_utf16_work_unit();
    window_binding_storage_admits_exact_and_refuses_one_byte_short();
    super::guest::tests::attached_registered_function_uses_the_exact_native_invocation();
    super::guest::tests::attached_proxy_uses_its_own_realm();
}

fn window_binding_storage_admits_exact_and_refuses_one_byte_short() {
    use surgeist_bindings_js::conversion::{CleanupStatus, ConversionAccount};
    use surgeist_css::serialization::CssomIdentifierLimits;

    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let probe = enable(session, target, cancel.clone());
    let permit = probe.permit().unwrap();
    let read = child_count_callback(Rc::downgrade(&probe.native), permit.clone());
    let detach = source::detach_callback(Rc::downgrade(&probe.native), permit);

    // Independent layout oracle for the native budget. The Bindings native
    // qualification checks these account field shapes against the actual
    // retained Rc<Account> targets. The hosted budget counts requested Rust
    // payload bytes and excludes allocator and V8 overhead.
    let account = size_of::<(Weak<()>, Arc<()>, (u64, usize), Cell<CleanupStatus>)>();
    let dom_exception = size_of::<(Rc<ConversionAccount>, DomHostedRuntimeRetention)>()
        + size_of::<ConversionAccount>()
        + size_of::<Weak<ConversionAccount>>()
        + account;
    let escape = size_of::<(
        ConversionAccount,
        CssomIdentifierLimits,
        HostFailureSlot<CssSerializationError, DomHostedRuntimeRetention>,
        DomHostedRuntimeRetention,
    )>() + account;
    let exact = size_of::<NativeOwner>()
        + size_of_val(&read)
        + size_of_val(&detach)
        + FailureSlot::storage_bytes()
        + SourceFailureSlot::storage_bytes()
        + DomDocumentScriptRegistration::storage_bytes()
        + escape
        + HostFailureSlot::<CssSerializationError, DomHostedRuntimeRetention>::storage_bytes()
        + dom_exception;
    drop(read);
    drop(detach);

    let policy = HtmlScriptingPolicy {
        user_allows: true,
        automation_allows: true,
    };
    let child_count = "readChildCount".encode_utf16().collect::<Vec<_>>();
    let detach_source_node_if = "detachIf".encode_utf16().collect::<Vec<_>>();
    let hooks = || DocumentScriptHooks {
        child_count: &child_count,
        detach_source_node_if: &detach_source_node_if,
    };
    let native_bindings_are_live = |host: &mut DocumentScriptHost| {
        let permit = host.permit().unwrap();
        host.in_document(&permit, |scope| {
            let script = r#"new DOMException().name === "Error" && CSS.escape("0") === "\\30 ""#
                .encode_utf16()
                .collect::<Vec<_>>();
            let value = scope.evaluate(&script)?;
            assert_eq!(scope.to_utf16(&value)?, [116, 114, 117, 101]);
            Ok(())
        })
        .unwrap();
    };

    let session = probe.stop().into_session().unwrap();
    let mut host = PreparedDocumentScriptHost::prepare(session, exact, cancel.clone())
        .unwrap()
        .enable(target, policy, hooks())
        .unwrap();
    native_bindings_are_live(&mut host);
    let session = host.stop().into_session().unwrap();
    let original = session.snapshot().stamp();
    let one_short = exact - 1;
    let refused = match PreparedDocumentScriptHost::prepare(session, one_short, cancel.clone())
        .unwrap()
        .enable(target, policy, hooks())
    {
        Err(refused) => refused,
        Ok(mut host) => {
            native_bindings_are_live(&mut host);
            let charged = host.engine.as_ref().unwrap().retained_bytes;
            panic!(
                "one-byte-short native Window published CSS.escape and DOMException: limit={one_short}, required={exact}, charged={charged}, account={account}"
            );
        }
    };
    assert!(matches!(
        refused.error,
        DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
    ));
    let session = refused.owner.into_session().unwrap();
    let snapshot = session.snapshot();
    assert_eq!(snapshot.stamp(), original);
    assert_eq!(
        snapshot.document(target.document()).unwrap().scripting(),
        surgeist_html_semantics::HtmlScriptingStatus::Disabled
    );
    let mut restored = PreparedDocumentScriptHost::prepare(session, exact, cancel)
        .unwrap()
        .enable(target, policy, hooks())
        .unwrap();
    assert_eq!(restored.engine.as_ref().unwrap().retained_bytes, exact);
    native_bindings_are_live(&mut restored);
    restored.stop().into_session().unwrap();
}

fn enable(
    session: DomSession,
    target: DomHostedTopLevel,
    cancel: Arc<AtomicBool>,
) -> DocumentScriptHost {
    PreparedDocumentScriptHost::prepare(session, 1_000_000, cancel)
        .unwrap()
        .enable(
            target,
            HtmlScriptingPolicy {
                user_allows: true,
                automation_allows: true,
            },
            DocumentScriptHooks {
                child_count: &"readChildCount".encode_utf16().collect::<Vec<_>>(),
                detach_source_node_if: &"detachIf".encode_utf16().collect::<Vec<_>>(),
            },
        )
        .unwrap()
}

const SOURCE: DomSourceLimits =
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 8);
const EVENT: DomSourceEventServiceLimits = DomSourceEventServiceLimits {
    source: SOURCE,
    dispatch: DomEventDispatchLimits {
        path_items: 16,
        snapshot_listeners: 16,
        failure_records: 16,
        callbacks: 16,
        work: 100_000,
    },
};

struct Projection {
    host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    execution: DomSourceExecutionScope,
    dispatcher: DomEventDispatcher<()>,
    dialog: NodeId,
    comment: NodeId,
}

impl Projection {
    fn expose(&mut self, handler: &str, cancel: &AtomicBool) {
        let (request, ()) = self.invoke(handler, SOURCE, cancel, |outcome| {
            assert!(
                matches!(outcome, DocumentScriptGuestResult::Returned),
                "{outcome:?}"
            );
        });
        self.finish(&request, cancel);
    }

    fn invoke<T>(
        &mut self,
        handler: &str,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
        observe: impl FnOnce(&DocumentScriptGuestResult) -> T,
    ) -> (DomSourceEventRequest, T) {
        let handler: Root = self
            .host
            .in_document(&self.permit, |scope| {
                let handler = evaluate(scope, handler)?;
                Ok(scope.retain(&handler))
            })
            .unwrap();
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: Some(&self.execution),
            style_policy: &mut |_| None,
            url: None,
        };
        let request = self
            .host
            .with_native(|dom, _, mut source| {
                assert!(matches!(
                    source
                        .apply(
                            dom,
                            DomSourceEdit::set_attribute(self.dialog, "open", ""),
                            SOURCE,
                            &mut services,
                            cancel,
                        )
                        .unwrap(),
                    DomSourceDriveProgress::Complete(_)
                ));
                let progress = source
                    .apply(
                        dom,
                        DomSourceEdit::close_dialog(self.dialog, None),
                        limits,
                        &mut services,
                        cancel,
                    )
                    .unwrap();
                let DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Event(request),
                    reason: DomSourceUnavailable::Event,
                } = progress
                else {
                    panic!("actual Dialog Close must reach beforetoggle: {progress:?}");
                };
                request
            })
            .unwrap();
        let progress = self
            .host
            .select_source_event(
                &self.permit,
                &request,
                &mut self.dispatcher,
                &mut (),
                DomSourceEventServiceLimits {
                    source: limits,
                    ..EVENT
                },
            )
            .unwrap();
        let DocumentScriptEventProgress::Selected(selected) = progress else {
            panic!("actual native listener must be selected");
        };
        let mut report = selected.invoke(&handler, self.comment, limits);
        let outcome = observe(report.outcome());
        report.report(&self.dispatcher).unwrap();
        (request, outcome)
    }

    fn finish(&mut self, request: &DomSourceEventRequest, cancel: &AtomicBool) {
        {
            let progress = self
                .host
                .select_source_event(&self.permit, request, &mut self.dispatcher, &mut (), EVENT)
                .unwrap();
            let DocumentScriptEventProgress::Native(DomSourceEventServiceProgress::Serviced {
                receipt,
            }) = progress
            else {
                panic!("the original dispatcher must publish the event receipt");
            };
            assert!(receipt.failures().is_empty());
        }
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: Some(&self.execution),
            style_policy: &mut |_| None,
            url: None,
        };
        self.host
            .with_native(|dom, _, mut source| {
                let progress = source
                    .resume(dom, request.operation(), SOURCE, &mut services, cancel)
                    .unwrap();
                assert!(matches!(
                    progress,
                    DomSourceDriveProgress::Complete(done)
                        if done.operation() == request.operation()
                ));
                assert!(dom.pending_source_edit().is_none());
            })
            .unwrap();
    }

    fn parent(&self) -> Option<NodeId> {
        self.host
            .snapshot()
            .unwrap()
            .node(self.comment)
            .unwrap()
            .parent()
    }
}

fn native_projection(cancel: Arc<AtomicBool>) -> (Projection, DomHostedTopLevel) {
    let (mut session, receiving) = hosted_session(&cancel, 8);
    let execution = session
        .source_execution_scope(receiving.host(), &cancel)
        .unwrap();
    let document = receiving.document();
    let html = session
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = session.snapshot().node(html).unwrap().children()[1];
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut create = DomBatch::new();
    let dialog = create.create_element_ns(document.node(), Namespace::html(), "dialog");
    let comment = create.create_comment(origin.document().node(), "A's native Comment");
    create.insert_before(&dialog, body, None);
    create.insert_before(&comment, origin.document().node(), None);
    let commit = session.dom().apply(create, &cancel).unwrap();
    let dialog = commit.created(&dialog).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    assert_eq!(
        session.snapshot().node(comment).unwrap().owner_document(),
        Some(origin.document())
    );
    let mut adopt = DomBatch::new();
    adopt.insert_before(comment, document.node(), None);
    session.dom().apply(adopt, &cancel).unwrap();
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.node(comment).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(
        snapshot.relevant_global(comment).unwrap(),
        Some(origin.global())
    );
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 4,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BEFORE_TOGGLE,
            DomEventCallback::host(77),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut host = enable(session, receiving, cancel);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    (
        Projection {
            host,
            permit,
            execution,
            dispatcher,
            dialog,
            comment,
        },
        origin,
    )
}

fn evaluate<'scope>(scope: &mut Scope<'scope>, text: &str) -> Result<Value<'scope>, Error> {
    scope.evaluate(&text.encode_utf16().collect::<Vec<_>>())
}

struct DropReceipt(Rc<Cell<usize>>);

impl Drop for DropReceipt {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn collect(projection: &mut Projection, marker: &Cell<usize>, expected: usize, phase: &str) {
    let witness = Rc::new(Cell::new(0));
    let receipt = DropReceipt(witness.clone());
    projection
        .host
        .in_document(&projection.permit, |scope| {
            let transient = Callback::new(move |_, _| {
                let _receipt = &receipt;
                Ok(Completion::Undefined)
            });
            scope.function(&transient)?;
            Ok(())
        })
        .unwrap();

    let engine = projection.host.engine.as_mut().unwrap();
    let mut execution = engine.process.execution();
    // Leave the last guest job before applying pressure. The transient
    // callback must actually collect before a surviving marker is evidence.
    for _ in 0..8 {
        execution.checkpoint().unwrap();
        execution.memory_pressure();
        if witness.get() == 1 && (expected == 0 || marker.get() == expected) {
            break;
        }
    }
    assert_eq!(
        witness.get(),
        1,
        "{phase}: transient capture was not collected"
    );
    assert_eq!(marker.get(), expected, "{phase}: projected capture drops");
}

fn install_origin_marker(
    projection: &mut Projection,
    origin: DomHostedTopLevel,
) -> Rc<Cell<usize>> {
    let marker_drops = Rc::new(Cell::new(0));
    let receipt = DropReceipt(marker_drops.clone());
    let engine = projection.host.engine.as_mut().unwrap();
    let origin_realm = engine
        .creations
        .find(origin.global(), || Ok::<_, Error>(()))
        .unwrap()
        .unwrap();
    let marker = engine
        .process
        .execution()
        .in_realm(origin_realm, |scope| {
            evaluate(scope, "globalThis.custodyMarker = 97")?;
            let identity = scope.realm_authority()?.identity();
            let callback = Callback::new(move |scope, _| {
                let _receipt = &receipt;
                assert_eq!(scope.realm_authority()?.identity(), identity);
                Ok(Completion::Return(evaluate(scope, "custodyMarker")?))
            });
            let marker = scope.function(&callback)?;
            Ok(scope.retain(&marker))
        })
        .unwrap();
    projection
        .host
        .in_document(&projection.permit, |scope| {
            evaluate(scope, "globalThis.custodyMarker = 19")?;
            let global = scope.global()?;
            let marker = scope.import(&marker)?;
            scope.set(
                &global,
                PropertyKey::String(&"projectionMarker".encode_utf16().collect::<Vec<_>>()),
                &marker,
            )
        })
        .unwrap();
    marker_drops
}

fn projected_comment_custody_outlives_collection_and_the_origin_realm_token() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut projection, origin) = native_projection(cancel.clone());
    let marker = install_origin_marker(&mut projection, origin);
    let document = projection.permit.document();
    projection
        .host
        .with_native(|dom, mut hosted, _| {
            let task = hosted
                .destroy_top_level(dom, origin.navigable(), &cancel)
                .unwrap();
            assert!((0..256).any(|_| matches!(
                hosted.run_next(dom, task.traversable(), &cancel).unwrap(),
                DomHostedTaskProgress::Complete { operation } if operation == task.operation()
            )));
            hosted.destroy_host(dom, origin.host(), &cancel).unwrap();
            let snapshot = dom.snapshot();
            assert!(
                !snapshot
                    .document(origin.document())
                    .unwrap()
                    .is_fully_active()
                    .unwrap()
            );
            assert!(
                snapshot
                    .document(document)
                    .unwrap()
                    .is_fully_active()
                    .unwrap()
            );
            assert_eq!(
                snapshot.relevant_global(projection.comment).unwrap(),
                Some(origin.global())
            );
        })
        .unwrap();

    projection.expose(
        "(comment) => {
            globalThis.projected = new WeakRef(comment);
            comment.marker = projectionMarker;
            if (!delete globalThis.projectionMarker || comment.marker() !== 97)
                throw Error('origin marker');
        }",
        &cancel,
    );
    assert_eq!(projection.parent(), Some(document.node()));
    collect(&mut projection, &marker, 0, "connected");

    projection.expose(
        "(comment) => {
            if (projected.deref() !== comment || comment.marker() !== 97)
                throw Error('connected expando');
            detachIf(1, comment);
        }",
        &cancel,
    );
    assert_eq!(projection.parent(), None);
    collect(&mut projection, &marker, 0, "detached");

    projection
        .host
        .with_native(|dom, _, _| {
            let mut insert = DomBatch::new();
            insert.insert_before(projection.comment, document.node(), None);
            dom.apply(insert, &cancel).unwrap();
        })
        .unwrap();
    projection.expose(
        "(comment) => {
            if (projected.deref() !== comment || comment.marker() !== 97)
                throw Error('detached expando');
            detachIf(1, comment);
        }",
        &cancel,
    );
    assert_eq!(projection.parent(), None);

    // Future first projection needs the production registration. After the
    // last exposure this private harness can release its sole Rust Realm
    // token while leaving the already admitted native pin in B's Process.
    let engine = projection.host.engine.as_mut().unwrap();
    let CreationRealm {
        next,
        global,
        realm,
        _backing: creation_backing,
    } = *engine.creations.0.take().unwrap();
    assert!(next.is_none());
    assert_eq!(global, origin.global());
    drop(realm);
    collect(&mut projection, &marker, 0, "origin token released");

    let engine = projection.host.engine.as_mut().unwrap();
    let key = engine
        .comments
        .find(projection.comment, || Ok(()))
        .unwrap()
        .unwrap();
    let holder = engine.comments.object(key).unwrap();
    engine
        .process
        .execution()
        .in_realm(&engine.realm, |scope| {
            let wrapper = scope.open_managed_object(holder)?;
            let weak = evaluate(scope, "projected.deref()")?;
            assert!(scope.strict_equals(&wrapper, &weak)?);
            let callback = scope.get(
                &wrapper,
                PropertyKey::String(&"marker".encode_utf16().collect::<Vec<_>>()),
            )?;
            let value = scope.call(&callback, &wrapper, &[])?;
            let expected = scope.primitive(Primitive::Number(97.0))?;
            assert!(scope.strict_equals(&value, &expected)?);
            Ok(())
        })
        .unwrap();

    projection
        .host
        .with_native(|dom, _, _| {
            let mut destroy = DomBatch::new();
            destroy.destroy(projection.comment);
            dom.apply(destroy, &cancel).unwrap();
            assert!(matches!(
                dom.snapshot().node(projection.comment),
                Err(DomError::StaleNode)
            ));
        })
        .unwrap();
    let engine = projection.host.engine.as_ref().unwrap();
    assert!(engine.comments.object(key).is_none());
    collect(&mut projection, &marker, 1, "native retirement");
    drop(creation_backing);
    projection
        .host
        .in_document(&projection.permit, |scope| {
            let count = evaluate(scope, "readChildCount()")?;
            let expected = scope.primitive(Primitive::Number(1.0))?;
            assert!(scope.strict_equals(&count, &expected)?);
            let wrapper = evaluate(scope, "projected.deref()")?;
            let undefined = scope.primitive(Primitive::Undefined)?;
            assert!(scope.strict_equals(&wrapper, &undefined)?);
            Ok(())
        })
        .unwrap();
}

fn install_brand_work_control(
    projection: &mut Projection,
    work: u64,
    remaining: u64,
) -> Rc<Cell<(usize, usize)>> {
    let native = Rc::downgrade(&projection.host.native);
    let hits = Rc::new(Cell::new((0, 0)));
    let witness = hits.clone();
    projection
        .host
        .in_document(&projection.permit, |scope| {
            let burn = Callback::new(move |_, _| {
                let native = native.upgrade().unwrap();
                let active = native.active_source().unwrap();
                let mut session = native.session.borrow_mut();
                let dom = session.dom();
                let status = dom.pending_source_edit().unwrap();
                assert_eq!(status.operation(), active.request().operation());
                assert!(status.spent_work() < work - remaining);
                for _ in status.spent_work()..work - remaining {
                    dom.raw()
                        .source_event_step(active.request(), &native.cancellation)
                        .unwrap();
                }
                Ok(Completion::Undefined)
            });
            let mark = Callback::new(move |scope, args| {
                let (diagnostic, coercion) = witness.get();
                witness.set(if scope.boolean(args.get(0).unwrap())? {
                    (diagnostic + 1, coercion)
                } else {
                    (diagnostic, coercion + 1)
                });
                Ok(Completion::Undefined)
            });
            let global = scope.global()?;
            for (name, callback) in [("limitBrandWork", burn), ("recordBrandResult", mark)] {
                let function = scope.function(&callback)?;
                scope.set(
                    &global,
                    PropertyKey::String(&name.encode_utf16().collect::<Vec<_>>()),
                    &function,
                )?;
            }
            Ok(())
        })
        .unwrap();
    hits
}

fn rejected_comment_requires_each_diagnostic_utf16_work_unit() {
    const LIMIT: DomSourceLimits =
        DomSourceLimits::new(32_000_000, 1_000_000, 8_192, 128, 4096, 4096, 8);
    const MESSAGE: &str = "Delegated Comment required";
    const HANDLER: &str = "(comment) => {
        const forged = Object.create(Object.getPrototypeOf(comment));
        limitBrandWork();
        try {
            detachIf({ valueOf() { recordBrandResult(false); return 1; } }, forged);
        } catch (error) {
            if (!(error instanceof TypeError) || error.message !== 'Delegated Comment required')
                throw error;
            recordBrandResult(true);
            return;
        }
        throw Error('forged Comment accepted');
    }";
    // The existing entry spends one step. The diagnostic's authored UTF-16
    // code units each need a step before the V8 TypeError can be allocated.
    let exact = 1 + u64::try_from(MESSAGE.encode_utf16().count()).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    for remaining in [exact, exact - 1] {
        let (mut projection, _) = native_projection(cancel.clone());
        let hits = install_brand_work_control(&mut projection, LIMIT.work(), remaining);
        let (request, failure) = projection.invoke(HANDLER, LIMIT, &cancel, |outcome| {
            if remaining == exact {
                assert!(
                    matches!(outcome, DocumentScriptGuestResult::Returned),
                    "{outcome:?}"
                );
                return None;
            }
            let DocumentScriptGuestResult::Refused(Error::HostFailure(failure)) = outcome else {
                panic!("one diagnostic UTF-16 unit short must refuse before throwing: {outcome:?}");
            };
            let cause = failure.cause::<DocumentScriptNativeError>().unwrap();
            assert!(
                matches!(
                    cause,
                    DocumentScriptNativeError::Dom(DomError::SourceWorkLimit { limit })
                        if *limit == LIMIT.work()
                ),
                "{cause:?}"
            );
            Some(failure.clone())
        });
        assert_eq!(hits.get(), (usize::from(remaining == exact), 0));
        assert!(projection.host.native.active_source().is_none());
        let document = projection.permit.document();
        assert_eq!(projection.parent(), Some(document.node()));
        projection
            .host
            .with_native(|dom, _, _| {
                assert_eq!(
                    dom.pending_source_edit().unwrap().operation(),
                    request.operation()
                );
                dom.raw_mut()
                    .raise_source_event_limits(&request, SOURCE, &cancel)
                    .unwrap();
            })
            .unwrap();

        if let Some(failure) = failure {
            let Err(DocumentScriptError::Script(Error::HostFailure(checkpoint))) =
                projection.host.checkpoint()
            else {
                panic!("the original sticky refusal must survive cleanup");
            };
            assert!(std::ptr::eq(
                failure.cause::<DocumentScriptNativeError>().unwrap(),
                checkpoint.cause::<DocumentScriptNativeError>().unwrap(),
            ));
            assert_eq!(projection.host.engine.as_ref().unwrap().comments.len(), 0);
            projection
                .host
                .with_native(|dom, _, source| {
                    assert!(matches!(
                        source.service_event(
                            dom,
                            &request,
                            &mut projection.dispatcher,
                            &mut (),
                            EVENT,
                            &cancel
                        ),
                        Err(DomSourceRuntimeError::SourceEvent(
                            DomSourceEventRuntimeError::Dispatch(
                                DomEventDispatchError::HostRefused(DomEventHostError { code: 77 })
                            )
                        ))
                    ));
                    assert_eq!(
                        dom.pending_source_edit().unwrap().operation(),
                        request.operation()
                    );
                    assert!(
                        dom.snapshot()
                            .node(projection.dialog)
                            .unwrap()
                            .attribute(&"open".into())
                            .is_some()
                    );
                })
                .unwrap();
            drop(failure);
            drop(checkpoint);
            let mut session = projection.host.stop().into_session().unwrap();
            assert_eq!(
                session.snapshot().document(document).unwrap().scripting(),
                HtmlScriptingStatus::Disabled
            );
            assert_eq!(
                session.dom().pending_source_edit().unwrap().operation(),
                request.operation()
            );
            assert_eq!(Rc::strong_count(&hits), 1);
            assert!(matches!(
                session.try_shutdown(),
                DomShutdownOutcome::Pending { .. }
            ));
        } else {
            projection.finish(&request, &cancel);
            assert_eq!(projection.parent(), Some(document.node()));
        }
    }
}

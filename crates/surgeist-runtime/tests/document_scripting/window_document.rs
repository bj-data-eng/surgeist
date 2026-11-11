//! The standard Window document getter over an actual attached native owner.

use super::*;

pub(super) fn window_projects_the_live_document_and_node_brands() {
    let (mut host, _) = prepare_event_window();
    let permit = host.permit().unwrap();
    let before = count_documents(&host);
    let script = r##"(() => {
        const descriptor = Object.getOwnPropertyDescriptor(window, "document");
        if (typeof descriptor?.get !== "function") return "missing Window.document getter";
        if (descriptor.configurable || !descriptor.enumerable || descriptor.set !== undefined) {
            return "Window.document must be an own unforgeable readonly attribute";
        }
        const actual = window.document;
        if (Reflect.apply(descriptor.get, window, []) !== actual || window.document !== actual) {
            return "Window.document must keep the original Document identity";
        }
        let branded = false;
        try { Reflect.apply(descriptor.get, {}, []); }
        catch (error) { branded = error instanceof TypeError; }
        if (!branded) return "a foreign receiver must be refused";
        if (!(actual instanceof Document && actual instanceof Node && actual instanceof EventTarget) ||
            Object.getPrototypeOf(actual) !== Document.prototype ||
            Object.getPrototypeOf(Document.prototype) !== Node.prototype ||
            Object.getPrototypeOf(Node.prototype) !== EventTarget.prototype) {
            return "the real Document must use the native interface chain";
        }
        for (const call of [() => Node(), () => new Node()]) {
            let refused = false;
            try { call(); }
            catch (error) { refused = error instanceof TypeError; }
            if (!refused) return "direct Node invocation must throw";
        }
        let nodePrototype;
        try { nodePrototype = Object.getPrototypeOf(Reflect.construct(function () {}, [], Node)); }
        catch (error) { return "Node must be usable as NewTarget"; }
        if (nodePrototype !== Node.prototype) return "Node must expose its interface prototype";
        if (actual.nodeType !== 9 || actual.nodeName !== "#document" ||
            actual.ownerDocument !== null || actual.parentNode !== null || !actual.isConnected ||
            actual.getRootNode() !== actual) {
            return "Node reads must observe the canonical Document";
        }
        let reads = 0;
        let wrongNodeRejected = false;
        const options = { get composed() { reads++; return true; } };
        try { Node.prototype.getRootNode.call(Object.create(Node.prototype), options); }
        catch (error) { wrongNodeRejected = error instanceof TypeError; }
        if (!wrongNodeRejected || reads !== 0 || actual.getRootNode(options) !== actual || reads !== 1) {
            return "Node receiver qualification must precede options conversion";
        }
        const sentinel = {};
        const newTarget = new Proxy(function () {}, {
            get(target, key) {
                if (key === "prototype") throw sentinel;
                return Reflect.get(target, key);
            }
        });
        let abrupt = false;
        try { Reflect.construct(Document, [], newTarget); }
        catch (error) { abrupt = error === sentinel; }
        if (!abrupt) return "Document must preserve abrupt prototype lookup";
        class DerivedDocument extends Document {}
        const created = new DerivedDocument();
        if (created === actual || !(created instanceof Document) ||
            Object.getPrototypeOf(created) !== DerivedDocument.prototype ||
            created.getRootNode() !== created || created.nodeType !== 9) {
            return "Document construction must use native identity and the selected prototype";
        }
        const capture = [];
        const observe = label => function (event) {
            const path = event.composedPath();
            capture.push(label, this === event.currentTarget, event.target === actual,
                path.length === 2 && path[0] === actual && path[1] === window);
        };
        const api = EventTarget.prototype;
        api.addEventListener.call(window, "native-route", observe("capture"), true);
        actual.addEventListener("native-route", observe("target"));
        api.addEventListener.call(window, "native-route", observe("bubble"));
        const event = new Event("native-route", { bubbles: true });
        if (!actual.dispatchEvent(event) || event.target !== actual ||
            event.currentTarget !== null || event.composedPath().length !== 0 ||
            capture.join(",") !== "capture,true,true,true,target,true,true,true,bubble,true,true,true") {
            return "Document dispatch must preserve the native Window path and recipient order";
        }
        return "ok";
    })()"##;
    let result = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
    assert_eq!(count_documents(&host), before + 1);
}

pub(super) fn native_document_retirement_releases_registered_callbacks() {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    host.in_document(&permit, |scope| {
        let counter = calls.clone();
        let observer = Callback::new(move |_, _| {
            counter.fetch_add(1, Ordering::Relaxed);
            Ok(Completion::Undefined)
        });
        let function = scope.function(&observer)?;
        let global = scope.global()?;
        let key: Vec<_> = "observeRetirement".encode_utf16().collect();
        scope.set(&global, PropertyKey::String(&key), &function)?;
        assert_script_number(
            scope,
            r#"Number((() => {
                const document = window.document;
                const listener = observeRetirement;
                delete window.observeRetirement;
                document.addEventListener("retire", listener);
                EventTarget.prototype.addEventListener.call(window, "retire", listener);
                return true;
            })())"#,
            1.0,
        )
    })
    .unwrap();
    assert_eq!(Arc::strong_count(&calls), 2);
    retire_attached(&mut host, top);
    assert_immediate_retirement(&host, &calls);
    assert!(matches!(
        host.in_document::<()>(&permit, |_| panic!("retired Document reentered guest code")),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::Revoked
        ))
    ));
    host.stop().into_session().unwrap();
}

pub(super) fn document_dispatch_preserves_native_observer_and_refusal() {
    let cancellation = Arc::new(cancel());
    let observer_calls = Arc::new(AtomicUsize::new(0));
    let (mut host, _) = prepare_event_window_with(cancellation.clone(), |dom, registry, top| {
        let document = DomEventTarget::from(top.document().node());
        let window = DomEventTarget::from(top.global());
        for (kind, refuse) in [("observed", false), ("refused", true)] {
            let expected_document = document.clone();
            let expected_window = window.clone();
            let calls = observer_calls.clone();
            let cancel_dispatch = cancellation.clone();
            registry
                .add_listener(
                    dom,
                    window.clone(),
                    kind.encode_utf16().collect::<Vec<_>>(),
                    DomEventCallback::new(move |_, _, _, event| {
                        assert_eq!(event.target().as_ref(), Some(&expected_document));
                        assert_eq!(event.current_target().as_ref(), Some(&expected_window));
                        assert_eq!(
                            event.composed_path().collect::<Vec<_>>(),
                            [expected_document.clone(), expected_window.clone()]
                        );
                        calls.fetch_add(1, Ordering::Relaxed);
                        if refuse {
                            cancel_dispatch.store(true, Ordering::Relaxed);
                        } else {
                            event.prevent_default();
                        }
                        Ok(())
                    }),
                    DomEventListenerOptions {
                        capture: true,
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    });
    let permit = host.permit().unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            let reached = false;
            window.document.addEventListener("observed", event => {
                reached = event.defaultPrevented;
            });
            return !window.document.dispatchEvent(new Event("observed", { cancelable: true }))
                && reached;
        })())"#,
        1.0,
    );
    assert_eq!(observer_calls.load(Ordering::Relaxed), 1);
    let first = native_failure(host.in_document(&permit, |scope| {
        let source = "window.document.dispatchEvent(new Event('refused'))";
        scope
            .evaluate(&source.encode_utf16().collect::<Vec<_>>())
            .map(drop)
    }));
    let cause = first.cause::<DocumentGuestEventError>().unwrap();
    assert!(matches!(
        cause,
        DocumentGuestEventError::Dispatch(DomEventDispatchError::Dom(DomError::Cancelled))
    ));
    cancellation.store(false, Ordering::Relaxed);
    let repeated = native_failure(host.checkpoint());
    assert!(std::ptr::eq(
        cause,
        repeated.cause::<DocumentGuestEventError>().unwrap()
    ));
    assert_eq!(observer_calls.load(Ordering::Relaxed), 2);
}

fn count_documents(host: &DocumentScriptHost) -> usize {
    let snapshot = host.snapshot().unwrap();
    snapshot
        .nodes()
        .filter(|id| snapshot.node(*id).unwrap().as_document().is_some())
        .count()
}

pub(super) fn window_uses_the_real_global_event_target() {
    let (mut host, _) = prepare_event_window();
    let permit = host.permit().unwrap();
    let script = r#"(() => {
        const trace = [];
        function listener(event) {
            const path = event.composedPath();
            trace.push(this === window && event.target === window &&
                event.currentTarget === window && path.length === 1 && path[0] === window);
            event.preventDefault();
        }
        const api = EventTarget.prototype;
        try { Reflect.apply(api.addEventListener, window, ["probe", listener, { once: true }]); }
        catch (error) { return "Window was refused as an EventTarget"; }
        const original = new Event("probe", { cancelable: true });
        if (Reflect.apply(api.dispatchEvent, window, [original]) ||
            original.target !== window || original.currentTarget !== null ||
            original.composedPath().length !== 0 || trace.join(",") !== "true") {
            return "the actual native Global route must expose the original Window";
        }
        const later = new Event("probe", { cancelable: true });
        if (!Reflect.apply(api.dispatchEvent, window, [later]) || trace.length !== 1) {
            return "a Window once listener must retire";
        }
        return "ok";
    })()"#;
    let result = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}

pub(super) fn window_resolves_the_inherited_event_target_operations() {
    let (mut host, _) = prepare_event_window();
    let permit = host.permit().unwrap();
    let script = r#"(() => {
        if (typeof Window !== "function") return "missing Window interface object";
        const target = EventTarget.prototype;
        if (Object.getPrototypeOf(window) !== Window.prototype
            || Object.getPrototypeOf(Window) !== EventTarget
            || !target.isPrototypeOf(window) || !(window instanceof Window)
            || Reflect.setPrototypeOf(window, {})) {
            return "Window must inherit EventTarget through its immutable global prototype";
        }
        for (const key of ["addEventListener", "removeEventListener", "dispatchEvent"]) {
            if (Object.hasOwn(window, key) || window[key] !== target[key]) {
                return "Window must resolve the original inherited EventTarget operations";
            }
        }
        let calls = 0;
        function observe(event) {
            if (this === window && event.target === window && event.currentTarget === window) calls++;
            event.preventDefault();
        }
        window.addEventListener("inherited", observe);
        if (dispatchEvent(new Event("inherited", { cancelable: true })) || calls !== 1) {
            return "ordinary registration and unqualified dispatch must use the native Global";
        }
        removeEventListener("inherited", observe);
        if (!window.dispatchEvent(new Event("inherited", { cancelable: true })) || calls !== 1) {
            return "unqualified removal must remove the native registration";
        }
        addEventListener("inherited", observe, { once: true });
        window.dispatchEvent(new Event("inherited"));
        if (calls !== 2) return "unqualified registration must use the native Global";
        let conversions = 0;
        try {
            target.addEventListener.call(Object.create(Window.prototype), {
                toString() { conversions++; return "inherited"; },
            }, observe);
            return "a prototype forgery must fail the Window receiver check";
        } catch (error) {
            if (!(error instanceof TypeError) || conversions !== 0) {
                return "receiver qualification must precede conversion";
            }
        }
        return "ok";
    })()"#;
    let result = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}

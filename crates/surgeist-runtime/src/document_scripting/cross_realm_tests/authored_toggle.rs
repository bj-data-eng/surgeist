//! Authored ToggleEvent conversion in the original attached Window.

use super::*;

pub(super) fn verify_authored_toggle_constructor() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target, source) = hosted_element(&cancel, "div");
    let mut batch = DomBatch::new();
    let root = batch.attach_shadow(
        source,
        surgeist_dom::DomShadowRootInit::new(surgeist_dom::HtmlShadowRootMode::Closed),
    );
    let inside = batch.create_element_ns(target.document(), Namespace::html(), "span");
    batch.insert_before(&inside, &root, None);
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let inside = commit.created(&inside).unwrap().unwrap();
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    registry.bind(&mut session.dom(), &cancel).unwrap();
    let mut registry = Some(registry);
    let mut host = super::super::guest::tests::attach_public_nodes_with_registry(
        session,
        target,
        cancel,
        1_000,
        20_000,
        &mut registry,
    );
    let permit = host.permit().unwrap();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        source,
        "authoredSource",
    );
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        inside,
        "shadowSource",
    );
    host.in_document(&permit, |scope| {
        let actual = evaluate(scope, r#"
            (() => {
                const defaults = new ToggleEvent("toggle", null);
                if (defaults.oldState !== "" || defaults.newState !== "" || defaults.source !== null
                    || defaults.bubbles || defaults.cancelable || defaults.composed || defaults.isTrusted)
                    return false;

                const seen = [];
                const type = { toString() { seen.push("type"); return "\ud800toggle"; } };
                const init = {
                    get bubbles() { seen.push("bubbles"); return true; },
                    get cancelable() { seen.push("cancelable"); return true; },
                    get composed() { seen.push("composed"); return true; },
                    get newState() { seen.push("newState"); return "\udc00next"; },
                    get oldState() { seen.push("oldState"); return "\ud800\u0000old"; },
                    get source() { seen.push("source"); return authoredSource; },
                };
                function Derived() {}
                Derived.prototype = Object.create(ToggleEvent.prototype);
                const newTarget = new Proxy(Derived, {
                    get(target, key, receiver) {
                        if (key === "prototype") seen.push("prototype");
                        return Reflect.get(target, key, receiver);
                    },
                });
                const event = Reflect.construct(ToggleEvent, [type, init], newTarget);
                let rejectsDocument = false;
                let requiresType = false;
                try { new ToggleEvent("bad", {source: document}); }
                catch (error) { rejectsDocument = error instanceof TypeError; }
                try { new ToggleEvent(); }
                catch (error) { requiresType = error instanceof TypeError; }
                const sentinel = {};
                let originalThrow = false;
                let selectedAfterThrow = false;
                const unusedNewTarget = new Proxy(Derived, {get() {
                    selectedAfterThrow = true;
                    return Derived.prototype;
                }});
                try {
                    Reflect.construct(ToggleEvent, ["abrupt", {get source() { throw sentinel; }}],
                        unusedNewTarget);
                } catch (error) { originalThrow = error === sentinel; }
                let originalPrototypeThrow = false;
                const throwingNewTarget = new Proxy(Derived, {get() { throw sentinel; }});
                try {
                    Reflect.construct(ToggleEvent, ["abrupt", {source: authoredSource}],
                        throwingNewTarget);
                } catch (error) { originalPrototypeThrow = error === sentinel; }
                function DefaultPrototype() {}
                DefaultPrototype.prototype = undefined;
                const fallback = Reflect.construct(ToggleEvent, ["fallback"], DefaultPrototype);
                return seen.join(",") === "type,bubbles,cancelable,composed,newState,oldState,source,prototype"
                    && Object.getPrototypeOf(event) === Derived.prototype
                    && event instanceof ToggleEvent && event instanceof Event
                    && event.type === "\ud800toggle"
                    && event.oldState === "\ud800\u0000old" && event.newState === "\udc00next"
                    && event.source === authoredSource && event.bubbles && event.cancelable
                    && event.composed && !event.isTrusted && rejectsDocument && requiresType
                    && originalThrow && !selectedAfterThrow && originalPrototypeThrow
                    && Object.getPrototypeOf(fallback) === ToggleEvent.prototype;
            })()
        "#)?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    }).expect("the actual attached Window must support authored ToggleEvent construction");

    host.in_document(&permit, |scope| {
        let actual = evaluate(scope, r#"
            (() => {
                const event = new ToggleEvent("observe-source", {
                    source: shadowSource, oldState: "earlier", newState: "later",
                    bubbles: true, composed: true, cancelable: true,
                });
                if (event.source !== authoredSource || event.currentTarget !== null) return false;
                const observations = [];
                shadowSource.addEventListener(event.type, delivered => {
                    observations.push(delivered === event && delivered.source === shadowSource
                        && delivered.currentTarget === shadowSource && delivered.oldState === "earlier");
                    delivered.preventDefault();
                });
                authoredSource.addEventListener(event.type, delivered => {
                    observations.push(delivered === event && delivered.source === authoredSource
                        && delivered.currentTarget === authoredSource && delivered.defaultPrevented);
                });
                EventTarget.prototype.addEventListener.call(window, event.type, delivered => {
                    observations.push(delivered === event && delivered.source === authoredSource
                        && delivered.currentTarget === window && delivered.newState === "later");
                });
                return shadowSource.dispatchEvent(event) === false
                    && observations.length === 3 && observations.every(Boolean)
                    && event.currentTarget === null && event.source === authoredSource;
            })()
        "#)?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    }).expect("ToggleEvent.source must retarget against the actual invocation target");

    let (constructor, source) = host
        .in_document(&permit, |scope| {
            let constructor = evaluate(scope, "ToggleEvent")?;
            let source = evaluate(scope, "authoredSource")?;
            Ok((scope.retain(&constructor), scope.retain(&source)))
        })
        .unwrap();
    let mut execution = host.engine.as_mut().unwrap().process.execution();
    let caller = execution.create_realm().unwrap();
    execution
        .in_realm(&caller, |scope| {
            let global = scope.global()?;
            let constructor = scope.import(&constructor)?;
            let source = scope.import(&source)?;
            let units = |text: &str| text.encode_utf16().collect::<Vec<_>>();
            scope.set(
                &global,
                PropertyKey::String(&units("ToggleEvent")),
                &constructor,
            )?;
            scope.set(&global, PropertyKey::String(&units("element")), &source)?;
            let actual = scope.evaluate(&units(
                r#"
                (() => {
                    class Derived extends ToggleEvent {}
                    const event = new Derived("foreign-caller", {source: element});
                    return Object.getPrototypeOf(event) === Derived.prototype
                        && event instanceof ToggleEvent && event.source === element
                        && event.type === "foreign-caller" && !event.isTrusted;
                })()
            "#,
            ))?;
            let expected = scope.primitive(Primitive::Boolean(true))?;
            assert!(scope.strict_equals(&actual, &expected)?);
            Ok(())
        })
        .unwrap();
}

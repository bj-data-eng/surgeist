//! Authored ToggleEvent conversion in the original attached Window.

use super::*;

pub(super) fn verify_authored_toggle_constructor() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target, source) = hosted_element(&cancel, "div");
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
                return seen.join(",") === "type,bubbles,cancelable,composed,newState,oldState,source,prototype"
                    && Object.getPrototypeOf(event) === Derived.prototype
                    && event instanceof ToggleEvent && event instanceof Event
                    && event.type === "\ud800toggle"
                    && event.oldState === "\ud800\u0000old" && event.newState === "\udc00next"
                    && event.source === authoredSource && event.bubbles && event.cancelable
                    && event.composed && !event.isTrusted && rejectsDocument && requiresType;
            })()
        "#)?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    }).expect("the actual attached Window must support authored ToggleEvent construction");
}

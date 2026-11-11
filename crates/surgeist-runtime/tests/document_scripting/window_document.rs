//! The standard Window document getter over an actual attached native owner.

use super::*;

pub(super) fn window_projects_the_live_document_and_node_brands() {
    let (mut host, _) = prepare_event_window();
    let permit = host.permit().unwrap();
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
        if (actual.nodeType !== 9 || actual.nodeName !== "#document" ||
            actual.ownerDocument !== null || actual.parentNode !== null || !actual.isConnected ||
            actual.getRootNode() !== actual) {
            return "Node reads must observe the canonical Document";
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

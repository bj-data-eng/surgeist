//! Ordinary HTML interface prototypes finish with their own constructor link.

use super::*;

pub(super) fn attached_html_prototypes_link_constructors_after_own_members() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let result = host
        .in_document(&permit, |scope| {
            let source = r#"(() => {
                const families = [
                    [HTMLElement, Element, 'article', ['title', 'hidden'], []],
                    [HTMLCanvasElement, HTMLElement, 'canvas', ['width', 'height'], []],
                    [HTMLTableRowElement, HTMLElement, 'tr', ['align', 'chOff'], []],
                    [HTMLDialogElement, HTMLElement, 'dialog', ['open'], ['close']],
                ];
                for (const [type, base, localName, attributes, operations] of families) {
                    const prototype = type.prototype;
                    if (Object.getOwnPropertyNames(prototype).at(-1) !== 'constructor') {
                        return type.name + ' constructor links before members';
                    }
                    for (const name of attributes) {
                        const member = Object.getOwnPropertyDescriptor(prototype, name);
                        if (typeof member?.get !== 'function') return type.name + ' own ' + name + ' getter';
                    }
                    for (const name of operations) {
                        const member = Object.getOwnPropertyDescriptor(prototype, name);
                        if (typeof member?.value !== 'function') return type.name + ' own ' + name + ' method';
                    }
                    const link = Object.getOwnPropertyDescriptor(prototype, 'constructor');
                    if (link?.value !== type || !link.writable || link.enumerable || !link.configurable) {
                        return type.name + ' constructor link descriptor';
                    }
                    const backLink = Object.getOwnPropertyDescriptor(type, 'prototype');
                    if (backLink?.value !== prototype || backLink.writable ||
                        backLink.enumerable || backLink.configurable) {
                        return type.name + ' prototype link descriptor';
                    }
                    if (Object.getPrototypeOf(type) !== base ||
                        Object.getPrototypeOf(prototype) !== base.prototype) {
                        return type.name + ' constructor and prototype inheritance';
                    }

                    const element = document.createElement(localName);
                    if (!(element instanceof type) || Object.getPrototypeOf(element) !== prototype ||
                        element.constructor !== type || Object.hasOwn(element, 'constructor')) {
                        return type.name + ' native element constructor lookup';
                    }
                }
                return 'ok';
            })()"#;
            let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
    host.stop().into_session().unwrap();
}

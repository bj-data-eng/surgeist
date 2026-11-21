//! Live scoped reflection on the two PopoverTargetAttributes includers.

use super::*;

pub(super) fn attached_controls_reflect_their_live_popover_target() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    for script in [ACCESSORS, ASSOCIATIONS, EXPLICIT_SCOPE] {
        let result = host
            .in_document(&permit, |scope| {
                let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
                scope.to_utf16(&value)
            })
            .unwrap();
        assert_eq!(String::from_utf16(&result).unwrap(), "ok");
    }
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

const ACCESSORS: &str = r#"(() => {
    const throwsTypeError = action => {
        try { action(); } catch (error) { return error instanceof TypeError; }
        return false;
    };
    const types = [[HTMLButtonElement, 'button'], [HTMLInputElement, 'input']];
    for (const [ctor, local] of types) {
        const node = document.createElement(local);
        const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:' + local);
        const foreign = document.createElementNS('urn:foreign', local);
        Object.setPrototypeOf(foreign, ctor.prototype);
        for (const member of ['popoverTargetElement', 'popoverTargetAction']) {
            const d = Object.getOwnPropertyDescriptor(ctor.prototype, member);
            if (typeof d?.get !== 'function') return 'missing ' + ctor.name + '.' + member;
            if (d.get.name !== 'get ' + member || d.get.length !== 0 ||
                typeof d.set !== 'function' || d.set.name !== 'set ' + member || d.set.length !== 1 ||
                !d.configurable || !d.enumerable || 'value' in d) return 'accessor metadata';
            for (const receiver of [{}, foreign, new Proxy(node, {}), document.body]) {
                let coerced = false;
                const operand = {toString() { coerced = true; return 'show'; }};
                if (!throwsTypeError(() => d.get.call(receiver)) ||
                    !throwsTypeError(() => d.set.call(receiver, operand)) || coerced) {
                    return 'receiver must be authenticated before conversion';
                }
            }
        }
        if (prefixed.popoverTargetElement !== null || prefixed.popoverTargetAction !== 'toggle') {
            return 'prefix does not change interface or missing defaults';
        }
        if (!throwsTypeError(() => { node.popoverTargetElement = document; }) ||
            !throwsTypeError(() => { node.popoverTargetElement = new Proxy(node, {}); }) ||
            node.hasAttribute('popovertarget')) return 'Element operand brand';
        node.popoverTargetAction = {toString() { node.setAttribute('popovertargetaction', 'show'); return 'HiDe'; }};
        if (node.popoverTargetAction !== 'hide' || node.getAttribute('popovertargetaction') !== 'HiDe') {
            return 'action conversion reentry and raw write';
        }
        const original = {};
        try {
            node.popoverTargetAction = {toString() { throw original; }};
            return 'action setter lost the original throw';
        } catch (error) {
            if (error !== original || node.popoverTargetAction !== 'hide') return 'original throw custody';
        }
        node.popoverTargetAction = 'invalid\ud800';
        if (node.popoverTargetAction !== 'toggle' ||
            node.getAttribute('popovertargetaction') !== 'invalid\ud800') return 'invalid default and UTF-16';
    }
    return 'ok';
})()"#;

const ASSOCIATIONS: &str = r#"(() => {
    const zone = document.body.appendChild(document.createElement('section'));
    const button = zone.appendChild(document.createElement('button'));
    const input = zone.appendChild(document.createElement('input'));
    const first = zone.appendChild(document.createElementNS('http://www.w3.org/2000/svg', 'svg'));
    const second = zone.appendChild(document.createElement('span'));
    const id = 'target-\ud800';
    first.setAttribute('id', id);
    second.setAttribute('id', id);
    button.disabled = true;
    input.setAttribute('type', 'text');
    for (const control of [button, input]) {
        control.setAttributeNS('urn:foreign', 'p:popovertarget', id);
        if (control.popoverTargetElement !== null) return 'namespaced attribute is ignored';
        control.setAttribute('popovertarget', id);
        if (control.popoverTargetElement !== first) return 'first Element by exact ID independently of activation';
    }
    zone.insertBefore(second, first);
    if (input.popoverTargetElement !== second || button.popoverTargetElement !== second) {
        return 'live tree order';
    }
    second.removeAttribute('id');
    first.setAttribute('id', 'TARGET-\ud800');
    if (button.popoverTargetElement !== null) return 'exact ID case';
    input.popoverTargetElement = first;
    if (input.popoverTargetElement !== first || input.getAttribute('popovertarget') !== '') {
        return 'explicit Element and empty content attribute';
    }
    input.setAttribute('popovertarget', id);
    if (input.popoverTargetElement !== null) return 'raw change clears explicit reference';
    input.popoverTargetElement = first;
    input.popoverTargetElement = undefined;
    if (input.popoverTargetElement !== null || input.hasAttribute('popovertarget')) {
        return 'nullable assignment removes attribute and explicit association';
    }
    zone.remove();
    return 'ok';
})()"#;

const EXPLICIT_SCOPE: &str = r#"(() => {
    const wrapper = document.body.appendChild(document.createElement('div'));
    const outside = wrapper.appendChild(document.createElement('button'));
    const shadowHost = wrapper.appendChild(document.createElement('div'));
    const root = shadowHost.attachShadow({mode: 'open'});
    const inside = root.appendChild(document.createElement('input'));
    const shadowOnly = root.appendChild(document.createElement('span'));
    const target = wrapper.appendChild(document.createElement('span'));
    outside.popoverTargetElement = shadowOnly;
    if (outside.popoverTargetElement !== null) return 'ordinary target ancestry stops at ShadowRoot';
    inside.popoverTargetElement = target;
    if (inside.popoverTargetElement !== target) return 'invoker ancestry crosses the shadow host';
    target.remove();
    if (inside.popoverTargetElement !== null) return 'detached explicit target is outside scope';
    wrapper.appendChild(target);
    if (inside.popoverTargetElement !== target) return 'reconnection restores a still-live weak target';
    target.id = 'outside-only';
    inside.setAttribute('popovertarget', 'outside-only');
    if (inside.popoverTargetElement !== null) return 'ID fallback stays in the ordinary root';
    shadowOnly.id = 'outside-only';
    if (inside.popoverTargetElement !== shadowOnly) return 'ID fallback sees the current shadow tree';
    wrapper.remove();
    return 'ok';
})()"#;

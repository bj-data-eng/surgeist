//! Live validation candidacy for the four native non-barred control families.

use super::*;

pub(super) fn attached_controls_observe_live_validation_candidacy() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    for script in [LOCAL_RULES, ANCESTRY] {
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

const LOCAL_RULES: &str = r#"(() => {
    const html = 'http://www.w3.org/1999/xhtml';
    const constructors = [
        ['button', HTMLButtonElement], ['input', HTMLInputElement],
        ['select', HTMLSelectElement], ['textarea', HTMLTextAreaElement]
    ];
    for (const [name, ctor] of constructors) {
        const d = Object.getOwnPropertyDescriptor(ctor.prototype, 'willValidate');
        if (typeof d?.get !== 'function') return 'missing ' + ctor.name + '.willValidate';
        if (d.set !== undefined || !d.enumerable || !d.configurable ||
            d.get.name !== 'get willValidate' || d.get.length !== 0) return 'willValidate descriptor';
        const control = document.createElementNS(html, 'p:' + name);
        for (const wrong of [document.body, Object.create(ctor.prototype), new Proxy(control, {})]) {
            try { d.get.call(wrong); return 'wrong receiver accepted'; }
            catch (error) { if (!(error instanceof TypeError)) return 'wrong receiver error'; }
        }
        control.setAttributeNS('urn:foreign', 'p:disabled', '');
        if (control.willValidate !== true) return 'detached candidate with foreign disabled';
        control.setAttribute('disabled', 'false');
        if (control.willValidate !== false) return 'disabled attribute presence';
        control.removeAttribute('disabled');
        if (control.willValidate !== true) return 'disabled removal';
    }

    const button = document.createElement('button');
    button.setAttribute('type', 'unknown');
    if (button.willValidate !== true) return 'Auto button without command is submitter';
    button.setAttribute('commandfor', '');
    if (button.willValidate !== false) return 'Auto commandfor presence bars Button';
    button.type = 'SuBmIt';
    if (button.willValidate !== true) return 'explicit Submit ignores commandfor';
    for (const type of ['reset', 'button']) {
        button.type = type;
        if (button.willValidate !== false) return 'non-submit Button type';
    }
    button.removeAttribute('commandfor');
    button.removeAttribute('type');
    const select = document.createElement('select');
    select.appendChild(button);
    if (button.willValidate !== false) return 'Auto Button with Select parent';
    select.removeChild(button);
    if (button.willValidate !== true) return 'Auto Button after Select removal';

    const input = document.createElement('input');
    input.readOnly = true;
    for (const [type, candidate] of [
        ['text', false], ['number', false], ['checkbox', true], ['radio', true],
        ['hidden', false], ['reset', false], ['button', false], ['submit', true]
    ]) {
        input.type = type;
        if (input.willValidate !== candidate) return 'Input readonly or type rule for ' + type;
    }
    input.type = 'text';
    input.readOnly = false;
    if (input.willValidate !== true) return 'Input readonly removal';
    const textarea = document.createElement('textarea');
    textarea.readOnly = true;
    if (textarea.willValidate !== false) return 'readonly TextArea is barred';
    textarea.readOnly = false;
    if (textarea.willValidate !== true) return 'TextArea readonly removal';
    return 'ok';
})()"#;

const ANCESTRY: &str = r#"(() => {
    const add = (parent, name) => parent.appendChild(document.createElement(name));
    const zone = add(document.body, 'section');
    const outer = add(zone, 'fieldset');
    const fieldset = add(outer, 'fieldset');
    const first = add(fieldset, 'legend');
    const second = add(fieldset, 'legend');
    const controls = [add(first, 'button'), add(first, 'input'), add(first, 'select'), add(first, 'textarea')];
    const elsewhere = add(second, 'button');
    fieldset.disabled = true;
    if (controls.some(control => control.willValidate !== true) || elsewhere.willValidate !== false) {
        return 'only the first Legend branch is exempt';
    }
    fieldset.insertBefore(second, first);
    if (controls.some(control => control.willValidate !== false) || elsewhere.willValidate !== true) {
        return 'live Legend ordering';
    }
    outer.disabled = true;
    if (elsewhere.willValidate !== false) return 'outer disabled Fieldset still bars inner exemption';
    outer.disabled = false;
    fieldset.disabled = false;
    const list = add(zone, 'datalist');
    for (const control of controls) {
        list.appendChild(control);
        if (control.willValidate !== false) return 'ordinary Datalist ancestor';
    }
    const foreign = zone.appendChild(document.createElementNS('urn:foreign', 'datalist'));
    foreign.setAttribute('disabled', '');
    for (const control of controls) {
        foreign.appendChild(control);
        if (control.willValidate !== true) return 'a foreign Datalist name does not bar';
    }
    fieldset.disabled = true;
    const shadowHost = add(fieldset, 'div');
    const shadow = shadowHost.attachShadow({mode: 'open'});
    for (const control of controls) {
        shadow.appendChild(control);
        if (control.willValidate !== true) return 'disabled ancestry stops at ShadowRoot';
    }
    zone.remove();
    return 'ok';
})()"#;

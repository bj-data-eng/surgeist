//! Live validation candidacy for the four native non-barred control families.

use super::*;

pub(super) fn attached_controls_observe_live_validation_candidacy() {
    native_candidacy_stops_at_first_refusal();
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

fn native_candidacy_stops_at_first_refusal() {
    use std::cell::Cell;

    let mut dom = native_dom();
    let mut edit = DomBatch::new();
    let document = edit.create_document();
    let fieldset = edit.create_element_ns(&document, Namespace::html(), "fieldset");
    let legend = edit.create_element_ns(&document, Namespace::html(), "legend");
    let input = edit.create_element_ns(&document, Namespace::html(), "input");
    edit.set_attribute(&fieldset, "disabled", "");
    edit.insert_before(&legend, &fieldset, None);
    edit.insert_before(&input, &legend, None);
    let receipt = dom.apply(edit, &cancel()).unwrap();
    let fieldset = receipt.created(&fieldset).unwrap().unwrap();
    let input = receipt.created(&input).unwrap().unwrap();
    let before = dom.snapshot();
    let read = |snapshot: &DomSnapshot, limit| {
        let count = Cell::new(0);
        let result = snapshot.node(input).unwrap().html_will_validate_with(|| {
            let attempted = count.get();
            count.set(attempted + 1);
            if attempted == limit {
                return Err(DomError::Cancelled);
            }
            Ok(())
        });
        (result, count.get())
    };
    let (found, exact) = read(&before, usize::MAX);
    assert!(found.unwrap());
    assert!(exact > 2);
    for cutoff in 0..exact {
        let (refused, attempted) = read(&before, cutoff);
        assert!(matches!(refused, Err(DomError::Cancelled)));
        assert_eq!(attempted, cutoff + 1);
    }
    assert!(read(&before, exact).0.unwrap());

    let mut edit = DomBatch::new();
    edit.insert_before(input, fieldset, None);
    dom.apply(edit, &cancel()).unwrap();
    assert!(!read(&dom.snapshot(), usize::MAX).0.unwrap());
    assert!(read(&before, exact).0.unwrap());
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

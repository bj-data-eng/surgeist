//! Attached Input accessors observe the canonical value modes and live flags.

use super::*;
use surgeist_html::attributes::HtmlInputType;

pub(super) fn attached_input_exposes_canonical_value_modes_and_checkedness() {
    let (session, top) = setup();
    let cancellation = Arc::new(cancel());
    let (mut host, _) =
        attach_event_window_with_work(session, top, cancellation.clone(), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const p = HTMLInputElement.prototype;
        const names = ['defaultChecked', 'checked', 'dirName', 'disabled', 'form', 'indeterminate',
            'multiple', 'name', 'placeholder', 'readOnly', 'required', 'type', 'defaultValue', 'value'];
        const callbacks = [];
        for (const name of names) {
            const d = Object.getOwnPropertyDescriptor(p, name);
            if (typeof d?.get !== 'function') return 'missing HTMLInputElement.prototype.' + name + ' getter';
            if (!d.enumerable || !d.configurable || d.get.name !== 'get ' + name || d.get.length !== 0 ||
                (name === 'form' ? d.set !== undefined :
                    typeof d.set !== 'function' || d.set.name !== 'set ' + name || d.set.length !== 1)) {
                return name + ' Input accessor descriptor';
            }
            callbacks.push(d.get);
            if (d.set) callbacks.push(d.set);
        }
        if (Object.getOwnPropertyNames(p).filter(name => names.includes(name)).join() !== names.join()) {
            return 'Input declaration order';
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const native = document.createElement('input');
        const prefixed = document.createElementNS(html, 'p:input');
        const upper = document.createElementNS(html, 'INPUT');
        const foreign = document.createElementNS('urn:foreign', 'input');
        Object.setPrototypeOf(foreign, p);
        if (Object.getPrototypeOf(native) !== p || Object.getPrototypeOf(p) !== HTMLElement.prototype ||
            !(prefixed instanceof HTMLInputElement) || prefixed.form !== null || prefixed.type !== 'text') {
            return 'exact native Input brands';
        }
        let conversions = 0;
        const argument = { [Symbol.toPrimitive]() { conversions++; return 'premature'; } };
        for (const callback of callbacks) {
            for (const wrong of [document.body, upper, foreign, Object.create(p), new Proxy(native, {})]) {
                try {
                    Reflect.apply(callback, wrong, [argument]);
                    return callback.name + ' accepted a non-Input receiver';
                } catch (error) {
                    if (!(error instanceof TypeError) || conversions !== 0) return callback.name + ' native brand first';
                }
            }
        }
        const exact = ' Raw\ud800\0\udc00 ';
        for (const [member, attribute] of [['dirName','dirname'], ['name','name'], ['placeholder','placeholder']]) {
            prefixed.setAttributeNS('urn:foreign', 'f:' + attribute, 'foreign');
            prefixed.setAttributeNS(null, attribute.toUpperCase(), 'uppercase');
            if (prefixed[member] !== '') return member + ' ignores other attribute names';
            prefixed[member] = exact;
            if (prefixed[member] !== exact || prefixed.getAttributeNS(null, attribute) !== exact ||
                prefixed.getAttributeNS(null, attribute.toUpperCase()) !== 'uppercase' ||
                prefixed.getAttributeNS('urn:foreign', attribute) !== 'foreign') return member + ' exact DOMString';
        }
        for (const [member, attribute] of [['defaultChecked','checked'], ['disabled','disabled'],
            ['multiple','multiple'], ['readOnly','readonly'], ['required','required']]) {
            prefixed.setAttributeNS('urn:foreign', 'f:' + attribute, '');
            if (prefixed[member]) return member + ' ignores foreign presence';
            prefixed.setAttributeNS(null, attribute, 'false');
            if (!prefixed[member]) return member + ' reflects presence';
            prefixed[member] = argument;
            if (!prefixed[member] || conversions !== 0 || prefixed.getAttributeNS(null, attribute) !== '') {
                return member + ' Web IDL Boolean';
            }
            prefixed[member] = false;
            if (prefixed[member] || prefixed.getAttributeNS(null, attribute) !== null ||
                prefixed.getAttributeNS('urn:foreign', attribute) !== '') return member + ' own presence only';
        }
        prefixed.setAttributeNS('urn:foreign', 'f:type', 'radio');
        prefixed.setAttributeNS(null, 'TYPE', 'hidden');
        if (prefixed.type !== 'text') return 'type ignores other attributes';
        prefixed.type = 'TeL';
        if (prefixed.type !== 'tel' || prefixed.getAttributeNS(null, 'type') !== 'TeL') return 'known Input type';
        prefixed.type = ' radio ';
        if (prefixed.type !== 'text' || prefixed.getAttributeNS(null, 'type') !== ' radio ') return 'invalid type';
        native.defaultValue = null;
        if (native.defaultValue !== 'null' || native.value !== 'null') return 'defaultValue uses ordinary null conversion';
        native.value = null;
        if (native.value !== '' || native.defaultValue !== 'null') return 'value uses legacy null to empty';
        native.value = undefined;
        if (native.value !== 'undefined' || native.defaultValue !== 'null') return 'value converts undefined normally';
        const mode = document.createElement('input');
        mode.type = 'checkbox';
        if (mode.value !== 'on' || mode.defaultValue !== '') return 'default/on selects on for absence';
        mode.value = null;
        if (mode.value !== '' || mode.getAttributeNS(null, 'value') !== '') return 'present empty wins in default/on';
        mode.removeAttributeNS(null, 'value');
        mode.type = 'HiDdEn';
        if (mode.value !== '' || mode.getAttributeNS(null, 'value') !== null) return 'default mode empty for absence';
        mode.value = 'raw\r\n\udc00';
        return mode.value === 'raw\r\n\udc00' && mode.defaultValue === mode.value &&
            mode.getAttributeNS(null, 'value') === mode.value ? 'ok' : 'default mode preserves raw UTF-16';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const first = document.body.appendChild(document.createElement('form'));
        const next = document.body.appendChild(document.createElement('form'));
        const nextId = 'input-owner\ud800';
        next.setAttributeNS(null, 'id', nextId);

        const observed = [];
        const checkboxType = 'ChEcKbOx';
        class ObservedInput extends HTMLInputElement {
            static observedAttributes = ['type', 'value'];
            attributeChangedCallback(name, oldValue, value, namespace) {
                observed.push([this, name, oldValue, value, namespace, this.type, this.value, this.form]);
                if (name === 'type' && value === checkboxType) this.placeholder = 'reacted\udc00';
            }
        }
        customElements.define('x-input-state-probe', ObservedInput, {extends: 'input'});
        const input = first.appendChild(document.createElement('input', {is: 'x-input-state-probe'}));
        if (!(input instanceof ObservedInput) || input.form !== first) return 'custom Input native Form';
        input.defaultValue = 'seed\r\n\udc00';
        if (input.value !== 'seed\udc00' || input.defaultValue !== 'seed\r\n\udc00') return 'clean current follows default';
        input.value = 'dirty\r\n\ud800\0\udc00';
        input.defaultValue = 'later\r\n\udc00';
        input.defaultChecked = true;
        if (!input.checked || !input.defaultChecked) return 'clean checkedness follows presence';
        input.checked = false;
        input.defaultChecked = false;
        input.defaultChecked = true;
        input.indeterminate = true;
        if (input.checked || !input.defaultChecked || !input.indeterminate ||
            input.value !== 'dirty\ud800\0\udc00' || input.defaultValue !== 'later\r\n\udc00') {
            return 'dirty current and checkedness survive default edits';
        }

        const radio = parent => {
            const node = parent.appendChild(document.createElement('input'));
            node.type = 'radio';
            node.name = 'shared\ud800';
            return node;
        };
        const left = radio(first);
        const right = radio(first);
        const other = radio(next);
        left.defaultChecked = true;
        right.checked = true;
        other.checked = true;
        if (left.checked || !left.defaultChecked || !right.checked || right.defaultChecked || !other.checked) {
            return 'radio peers change checkedness in their real Form';
        }
        right.indeterminate = true;
        left.checked = true;
        if (!left.checked || right.checked || !right.indeterminate || !other.checked ||
            left.form !== first || right.form !== first || other.form !== next) return 'radio owner isolation';

        const file = next.appendChild(document.createElement('input'));
        file.type = 'FiLe';
        file.defaultValue = 'metadata\ud800';
        file.value = null;
        file.value = '';
        if (file.value !== '' || file.type !== 'file' || file.defaultValue !== 'metadata\ud800') {
            return 'admitted File has a real empty value';
        }
        observed.length = 0;
        globalThis.inputStateProbe = {input, first, next, nextId, other, file, observed, checkboxType};
        return 'ok';
    })()"#,
    );

    let before = host.snapshot().unwrap();
    let html = before
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = before.node(html).unwrap().children()[1];
    let [first, next]: [NodeId; 2] = before.node(body).unwrap().children().try_into().unwrap();
    let [subject, left, right]: [NodeId; 3] =
        before.node(first).unwrap().children().try_into().unwrap();
    let [other, file]: [NodeId; 2] = before.node(next).unwrap().children().try_into().unwrap();
    let native = input(&before, subject);
    assert_eq!(native.form_owner().unwrap(), Some(first));
    assert_eq!(
        native.value(),
        units("dirty", &[0xd800, 0, 0xdc00]).code_units()
    );
    assert_eq!(
        native.state().selection().unwrap().cursor_position(),
        Some(8)
    );
    let flags = native.state().flags();
    assert!(flags.dirty_value());
    assert!(flags.dirty_checkedness());
    assert!(!flags.checkedness());
    assert!(flags.indeterminate());
    assert!(input(&before, left).state().flags().checkedness());
    let peer = input(&before, right).state().flags();
    assert!(!peer.checkedness());
    assert!(peer.dirty_checkedness());
    assert!(peer.indeterminate());
    assert_eq!(input(&before, other).form_owner().unwrap(), Some(next));
    assert!(input(&before, other).state().flags().checkedness());
    let empty_file = input(&before, file).state();
    assert_eq!(empty_file.input_type(), HtmlInputType::File);
    assert!(empty_file.selected_files().unwrap().is_empty());

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {file} = inputStateProbe;
        try {
            file.value = 'unselected\ud800';
            return 'File accepted a nonempty value';
        } catch (error) {
            if (!(error instanceof DOMException) || Object.getPrototypeOf(error) !== DOMException.prototype ||
                error.name !== 'InvalidStateError' || error.code !== 11) return 'File value typed realm exception';
        }
        return file.value === '' && file.defaultValue === 'metadata\ud800' &&
            file.getAttributeNS(null, 'value') === 'metadata\ud800' ? 'ok' : 'File failure changed the native value';
    })()"#,
    );

    let refused = host.snapshot().unwrap();
    assert_eq!(refused.revision(), before.revision());
    assert_eq!(input(&refused, file).state(), empty_file);
    assert!(input(&refused, file).value().is_empty());
    assert_eq!(
        attribute(&refused, file, "value"),
        Some(&units("metadata", &[0xd800]))
    );
    assert_eq!(refused.incomplete_source_operation(), None);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {input, first, next, nextId, other, observed, checkboxType} = inputStateProbe;
        const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
        const carried = 'dirty\ud800\0\udc00';
        const written = 'authored\r\n\ud800\0\udc00';
        let receiver = input;
        let conversions = 0;
        const result = setValue.call(receiver, { [Symbol.toPrimitive](hint) {
            if (hint !== 'string') throw new Error('Input value conversion hint');
            conversions++;
            receiver = other;
            next.appendChild(input);
            input.setAttributeNS(null, 'form', nextId);
            input.type = checkboxType;
            if (input.form !== next || input.type !== 'checkbox' || input.value !== carried ||
                input.placeholder !== 'reacted\udc00' || observed.length !== 2) {
                throw new Error('Input type transition reactions did not publish before coercion returned');
            }
            return written;
        } });
        if (result !== undefined || conversions !== 1 || receiver !== other ||
            input.form !== next || input.form === first || other.form !== next ||
            input.value !== written || input.defaultValue !== written || other.value !== 'on' || !other.checked) {
            return 'resumed setter must use the original Input and its new mode and Form';
        }
        const [type, transfer, write] = observed;
        if (observed.length !== 3 || !observed.every(row =>
            row[0] === input && row[4] === null && row[5] === 'checkbox' && row[7] === next)) {
            return 'Input reactions must see the actual receiver, type and Form';
        }
        if (type[1] !== 'type' || type[2] !== null || type[3] !== checkboxType || type[6] !== carried ||
            transfer[1] !== 'value' || transfer[2] !== 'later\r\n\udc00' || transfer[3] !== carried ||
            transfer[6] !== carried || write[1] !== 'value' || write[2] !== carried ||
            write[3] !== written || write[6] !== written) return 'Input type transfer and write reaction order';
        return 'ok';
    })()"#,
    );

    let moved = host.snapshot().unwrap();
    let moved_input = input(&moved, subject);
    let authored = units("authored\r\n", &[0xd800, 0, 0xdc00]);
    assert_eq!(moved_input.form_owner().unwrap(), Some(next));
    assert_eq!(moved.node(subject).unwrap().parent(), Some(next));
    assert_eq!(moved_input.state().input_type(), HtmlInputType::Checkbox);
    assert_eq!(moved_input.value(), authored.code_units());
    assert_eq!(attribute(&moved, subject, "value"), Some(&authored));
    assert_eq!(
        attribute(&moved, subject, "type"),
        Some(&DocumentString::from("ChEcKbOx"))
    );
    assert_eq!(
        attribute(&moved, subject, "form"),
        Some(&units("input-owner", &[0xd800]))
    );
    assert_eq!(
        attribute(&moved, subject, "placeholder"),
        Some(&units("reacted", &[0xdc00]))
    );
    assert_eq!(input(&moved, other).value(), &[0x6f, 0x6e]);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {input} = inputStateProbe;
        input.type = 'tExT';
        if (input.type !== 'text' || input.value !== 'authored\ud800\0\udc00' || input.checked) {
            return 're-entering Value mode installs and sanitizes content';
        }
        input.value = 'set\r\n\ud800\0\udc00';
        input.defaultValue = 'reset\r\n\ud800\0\udc00';
        return input.value === 'set\ud800\0\udc00' && input.defaultChecked &&
            !input.checked && input.indeterminate ? 'ok' : 'native reset preconditions';
    })()"#,
    );
    let dirty = host.snapshot().unwrap();
    let state = input(&dirty, subject).state();
    assert!(state.flags().dirty_value());
    assert!(state.flags().dirty_checkedness());
    assert_eq!(state.current_value(), &units("set", &[0xd800, 0, 0xdc00]));
    assert_eq!(state.selection().unwrap().cursor_position(), Some(6));

    host.with_native(|dom, _, mut source| {
        let result = source
            .apply(
                dom,
                DomSourceEdit::reset_input(subject),
                DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancellation,
            )
            .unwrap();
        assert!(
            matches!(result, DomSourceDriveProgress::Complete(_)),
            "{result:?}"
        );
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    let reset = host.snapshot().unwrap();
    let state = input(&reset, subject).state();
    assert_eq!(state.current_value(), &units("reset", &[0xd800, 0, 0xdc00]));
    assert_eq!(
        attribute(&reset, subject, "value"),
        Some(&units("reset\r\n", &[0xd800, 0, 0xdc00]))
    );
    assert!(!state.flags().dirty_value());
    assert!(!state.flags().dirty_checkedness());
    assert!(state.flags().checkedness());
    assert!(state.flags().indeterminate());

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {input, next} = inputStateProbe;
        if (input.defaultValue !== 'reset\r\n\ud800\0\udc00' || input.value !== 'reset\ud800\0\udc00' ||
            !input.checked || !input.defaultChecked || !input.indeterminate || input.form !== next) {
            return 'guest must observe the native Input reset';
        }
        input.defaultValue = 'again\r\udc00';
        input.defaultChecked = false;
        return input.value === 'again\udc00' && !input.checked && input.indeterminate
            ? 'ok' : 'reset restored following the defaults';
    })()"#,
    );
    let after = host.snapshot().unwrap();
    let state = input(&after, subject).state();
    assert_eq!(state.current_value(), &units("again", &[0xdc00]));
    assert!(!state.flags().checkedness());
    assert!(state.flags().indeterminate());
    assert_eq!(after.incomplete_source_operation(), None);
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn input(snapshot: &DomSnapshot, node: NodeId) -> surgeist_dom::form::DomInput<'_> {
    snapshot.node(node).unwrap().as_input().unwrap()
}

fn attribute<'a>(
    snapshot: &'a DomSnapshot,
    node: NodeId,
    name: &str,
) -> Option<&'a DocumentString> {
    snapshot
        .node(node)
        .unwrap()
        .attribute_ns(&Namespace::none(), &name.into())
}

fn units(prefix: &str, suffix: &[u16]) -> DocumentString {
    DocumentString::from_utf16(
        prefix
            .encode_utf16()
            .chain(suffix.iter().copied())
            .collect::<Vec<_>>(),
    )
}

fn observe(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str) {
    let result = host
        .in_document(permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}

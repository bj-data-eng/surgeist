//! ElementContentEditable over actual attached V8 and canonical native edits.

use super::*;

pub(super) fn attached_html_elements_use_native_content_editability() {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLElement.prototype;
            if (typeof Object.getOwnPropertyDescriptor(prototype, 'contentEditable')?.get !== 'function') {
                return 'missing HTMLElement.prototype.contentEditable getter';
            }
            const input = document.createElement('input');
            if (!(input instanceof HTMLInputElement) || !(input instanceof HTMLElement)) {
                return 'attached realm must supply a native derived HTML element';
            }
            const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            const math = document.createElementNS('http://www.w3.org/1998/Math/MathML', 'math');
            const throwsTypeError = invoke => {
                try { invoke(); } catch (error) { return error instanceof TypeError; }
                return false;
            };
            for (const key of ['contentEditable', 'enterKeyHint', 'isContentEditable', 'inputMode']) {
                const d = Object.getOwnPropertyDescriptor(prototype, key);
                if (!d || typeof d.get !== 'function' || d.get.length !== 0 || d.get.name !== 'get ' + key ||
                    !d.enumerable || !d.configurable || 'value' in d || 'writable' in d ||
                    (key === 'isContentEditable' ? d.set !== undefined :
                        typeof d.set !== 'function' || d.set.length !== 1 || d.set.name !== 'set ' + key)) {
                    return 'ordinary HTMLElement accessor descriptor for ' + key;
                }
                if (Object.hasOwn(HTMLInputElement.prototype, key) || Object.hasOwn(Element.prototype, key) ||
                    key in svg || key in math || input[key] !== d.get.call(input)) {
                    return 'HTMLElement-only mixin inheritance for ' + key;
                }
                for (const accessor of d.set === undefined ? [d.get] : [d.get, d.set]) {
                    if (Object.hasOwn(accessor, 'prototype') ||
                        !throwsTypeError(() => Reflect.construct(function () {}, [], accessor))) {
                        return 'nonconstructible accessor for ' + key;
                    }
                    for (const other of [null, undefined, {}, Object.create(prototype), new Proxy(input, {}), svg, math]) {
                        let converted = false;
                        const value = { [Symbol.toPrimitive]() { converted = true; return 'true'; } };
                        if (!throwsTypeError(() => Reflect.apply(accessor, other, [value])) || converted) {
                            return 'receiver must be checked before coercion for ' + key;
                        }
                    }
                }
            }
            return 'ok';
        })()"#,
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const element = document.createElement('input');
            element.setAttributeNS('urn:foreign', 'x:contenteditable', 'true');
            element.setAttributeNS('urn:foreign', 'x:enterkeyhint', 'enter');
            element.setAttributeNS('urn:foreign', 'x:inputmode', 'numeric');
            if (element.contentEditable !== 'inherit' || element.isContentEditable ||
                element.enterKeyHint !== '' || element.inputMode !== '') return 'null namespace lookup';
            for (const [raw, state, editable] of [['','true',true], ['TrUe','true',true],
                ['FaLsE','false',false], ['PlAiNtExT-OnLy','plaintext-only',true],
                ['true ','inherit',false], ['\ud800\0\udc00','inherit',false]]) {
                element.setAttributeNS(null, 'contenteditable', raw);
                if (element.contentEditable !== state || element.isContentEditable !== editable ||
                    element.getAttributeNS(null, 'contenteditable') !== raw) return 'native contenteditable state';
            }
            for (const [member, attribute, values] of [
                ['enterKeyHint', 'enterkeyhint', [['PrEvIoUs','previous'], ['SeArCh','search'], [' enter',''], ['','']]],
                ['inputMode', 'inputmode', [['DeCiMaL','decimal'], ['NuMeRiC','numeric'], ['TEL','tel'], ['none ',''], ['','']]],
            ]) {
                for (const [input, reflected] of values) {
                    element[member] = input;
                    if (element[member] !== reflected || element.getAttributeNS(null, attribute) !== input) {
                        return 'limited known getter and raw string setter for ' + member;
                    }
                }
                const raw = '\ud800\0\udc00';
                let conversions = 0;
                element[member] = { [Symbol.toPrimitive](hint) {
                    if (hint !== 'string') throw new Error('wrong DOMString coercion hint');
                    conversions++;
                    element.setAttributeNS(null, attribute, 'go');
                    return raw;
                } };
                if (conversions !== 1 || element[member] !== '' || element.getAttributeNS(null, attribute) !== raw) {
                    return 'one DOMString coercion and exact UTF-16 storage for ' + member;
                }
            }
            const calls = [];
            customElements.define('x-editability-observed', class extends HTMLElement {
                static observedAttributes = ['contenteditable', 'enterkeyhint', 'inputmode'];
                attributeChangedCallback(key, previous, next, namespace) {
                    calls.push([key, previous, next, namespace, this.getAttributeNS(null, key), this.isContentEditable]);
                }
            });
            const watched = document.createElement('x-editability-observed');
            let conversions = 0;
            watched.contentEditable = { [Symbol.toPrimitive](hint) {
                if (hint !== 'string') throw new Error('wrong contentEditable coercion hint');
                conversions++;
                if (watched.contentEditable !== 'inherit') throw new Error('early mutation');
                return 'PlAiNtExT-OnLy';
            } };
            watched.enterKeyHint = 'PREVIOUS';
            watched.inputMode = 'DECIMAL';
            watched.contentEditable = 'InHeRiT';
            watched.contentEditable = true;
            watched.contentEditable = 'FALSE';
            const expected = [
                ['contenteditable', null, 'plaintext-only', null, 'plaintext-only', true],
                ['enterkeyhint', null, 'PREVIOUS', null, 'PREVIOUS', true],
                ['inputmode', null, 'DECIMAL', null, 'DECIMAL', true],
                ['contenteditable', 'plaintext-only', null, null, null, false],
                ['contenteditable', null, 'true', null, 'true', true],
                ['contenteditable', 'true', 'false', null, 'false', false],
            ];
            if (conversions !== 1 || JSON.stringify(calls) !== JSON.stringify(expected)) {
                return 'canonical mutation and synchronous reactions with live editability';
            }
            globalThis.watchedEditability = {watched, calls};
            return 'ok';
        })()"#,
    );
    let before = host.snapshot().unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {watched, calls} = watchedEditability;
            const length = calls.length;
            for (const input of ['', ' true', 'true\0', 'plaintext-only\ud800', 'inherit ', null, undefined]) {
                try { watched.contentEditable = input; return 'invalid contentEditable accepted'; }
                catch (error) {
                    if (Object.getPrototypeOf(error) !== DOMException.prototype || error.name !== 'SyntaxError' ||
                        error.code !== DOMException.SYNTAX_ERR) return 'invalid value must throw the actual realm SyntaxError DOMException';
                }
            }
            const original = {};
            try {
                watched.contentEditable = {toString() { throw original; }};
                return 'DOMString abrupt completion accepted';
            } catch (error) { if (error !== original) return 'original coercion throw identity'; }
            try { watched.contentEditable = Symbol(); return 'DOMString Symbol accepted'; }
            catch (error) { if (!(error instanceof TypeError)) return 'DOMString Symbol failure'; }
            return calls.length === length && watched.contentEditable === 'false' &&
                watched.getAttributeNS(null, 'contenteditable') === 'false' ? 'ok' : 'failed setter changed raw value or reactions';
        })()"#,
    );
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    host.stop().into_session().unwrap();

    ordinary_ancestry_reopens_after_native_mutation();
    genuine_design_mode_is_read_from_the_parent_document();
}

fn ordinary_ancestry_reopens_after_native_mutation() {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    let document = top.document();
    let [root, target, barrier] = host
        .with_native(|dom, _, _| {
            let parent = dom
                .snapshot()
                .document(document)
                .unwrap()
                .document_element()
                .unwrap();
            let mut batch = DomBatch::new();
            let root = batch.create_element_ns(document, Namespace::html(), "div");
            let target = batch.create_element_ns(document, Namespace::html(), "span");
            let barrier = batch.create_element_ns(document, Namespace::html(), "section");
            batch.set_attribute(&root, "contenteditable", "");
            batch.set_attribute(&barrier, "contenteditable", "false");
            batch.insert_before(&target, &root, None);
            batch.insert_before(&barrier, &root, None);
            batch.insert_before(&root, parent, None);
            let commit = dom.apply(batch, &cancel()).unwrap();
            [&root, &target, &barrier].map(|ticket| commit.created(ticket).unwrap().unwrap())
        })
        .unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const root = document.documentElement.lastChild;
            const target = root.firstChild;
            const barrier = target.nextSibling;
            globalThis.editabilityTree = {root, target, barrier};
            if (root.localName !== 'div' || target.localName !== 'span' || barrier.localName !== 'section' ||
                !root.isContentEditable || !target.isContentEditable || barrier.isContentEditable ||
                target.contentEditable !== 'inherit') return 'current ordinary HTML inheritance';
            for (const [namespace, local, nonroot] of [
                ['http://www.w3.org/2000/svg', 'svg', 'g'],
                ['http://www.w3.org/1998/Math/MathML', 'math', 'mrow'],
            ]) {
                const exact = root.appendChild(document.createElementNS(namespace, 'x:' + local));
                const nested = exact.appendChild(document.createElementNS(namespace, nonroot));
                if (!exact.appendChild(document.createElement('span')).isContentEditable ||
                    nested.appendChild(document.createElement('span')).isContentEditable) {
                    return 'exact foreign-root participation and foreign barrier: ' + local;
                }
                const lookalike = root.appendChild(document.createElementNS('urn:foreign', local));
                if (lookalike.appendChild(document.createElement('span')).isContentEditable) {
                    return 'namespace-exact root qualification: ' + local;
                }
            }
            const shadowHost = root.appendChild(document.createElement('div'));
            const shadow = shadowHost.attachShadow({mode: 'open'});
            const shadowChild = shadow.appendChild(document.createElement('span'));
            return shadowHost.isContentEditable && !shadowChild.isContentEditable
                ? 'ok' : 'ordinary inheritance must stop at the shadow root';
        })()"#,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.insert_before(target, barrier, None);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(
        host.snapshot().unwrap().node(target).unwrap().parent(),
        Some(barrier)
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {root, target, barrier} = editabilityTree;
            return target.parentNode === barrier && root.isContentEditable && !target.isContentEditable
                ? 'ok' : 'existing wrapper must reopen current ancestry after native move';
        })()"#,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(target);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(
        host.snapshot().unwrap().node(root).unwrap().children()[0],
        barrier
    );
    assert_eq!(
        host.snapshot().unwrap().node(target).unwrap().parent(),
        None
    );
    observe(
        &mut host,
        &permit,
        "editabilityTree.target.parentNode === null && !editabilityTree.target.isContentEditable ? 'ok' : 'detached inheritance'",
    );
    host.stop().into_session().unwrap();
}

fn genuine_design_mode_is_read_from_the_parent_document() {
    let (mut host, _) = prepare_event_window();
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap();
    observe(
        &mut host,
        &permit,
        "globalThis.editabilityDesignDocument = new Document(); editabilityDesignDocument.documentElement === null ? 'ok' : 'rootless native Document'",
    );
    let snapshot = host.snapshot().unwrap();
    let native = snapshot
        .nodes()
        .filter(|id| before.node(*id).is_err())
        .filter_map(|id| snapshot.node(id).unwrap().as_document())
        .map(|document| document.id())
        .collect::<Vec<_>>();
    let [document] = native.as_slice() else {
        panic!("one original constructed Document: {native:?}");
    };
    let document = *document;
    assert!(
        !snapshot
            .document(document)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    // Enable on the original rootless Document. The real Source operation
    // completes without a focusing requirement, then the same Document gets a root.
    set_native_design_mode(&mut host, document, "oN");
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        let root = batch.create_element_ns(document, Namespace::html(), "div");
        let child = batch.create_element_ns(document, Namespace::html(), "span");
        batch.set_attribute(&root, "contenteditable", "false");
        batch.insert_before(&child, &root, None);
        batch.insert_before(&root, document, None);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert!(
        host.snapshot()
            .unwrap()
            .document(document)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const root = editabilityDesignDocument.documentElement;
            globalThis.editabilityDesignRoot = root;
            return root.contentEditable === 'false' && root.isContentEditable && root.firstChild.isContentEditable
                ? 'ok' : 'native design-mode host and ordinary descendant';
        })()"#,
    );
    set_native_design_mode(&mut host, document, "off");
    assert!(
        !host
            .snapshot()
            .unwrap()
            .document(document)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    observe(
        &mut host,
        &permit,
        "!editabilityDesignRoot.isContentEditable && !editabilityDesignRoot.firstChild.isContentEditable ? 'ok' : 'live native design-mode disable'",
    );
    host.stop().into_session().unwrap();
}

fn set_native_design_mode(host: &mut DocumentScriptHost, document: DocumentId, value: &str) {
    host.with_native(|dom, _, mut source| {
        let progress = source
            .apply(
                dom,
                DomSourceEdit::set_design_mode(document, value),
                DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap();
        assert!(
            matches!(progress, DomSourceDriveProgress::Complete(_)),
            "{progress:?}"
        );
    })
    .unwrap();
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

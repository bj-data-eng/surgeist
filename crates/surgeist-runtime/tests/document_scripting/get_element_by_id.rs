//! Native Document and Fragment ID search through the original attached wrappers.

use super::*;

pub(super) fn attached_id_lookup_uses_the_current_ordinary_tree() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const doc = Object.getOwnPropertyDescriptor(Document.prototype, 'getElementById');
        if (typeof doc?.value !== 'function') return 'missing Document.prototype.getElementById';
        const frag = Object.getOwnPropertyDescriptor(DocumentFragment.prototype, 'getElementById');
        const typeError = action => {
            try { action(); } catch (error) { return error instanceof TypeError; }
            return false;
        };
        const fragment = document.createDocumentFragment();
        const shadow = document.createElement('div').attachShadow({mode: 'open'});
        for (const [descriptor, valid, other] of [[doc, document, fragment], [frag, fragment, document]]) {
            if (typeof descriptor?.value !== 'function' || descriptor.value.name !== 'getElementById' ||
                descriptor.value.length !== 1 || !descriptor.writable || !descriptor.enumerable ||
                !descriptor.configurable || 'get' in descriptor || 'set' in descriptor) {
                return 'ordinary getElementById method descriptor';
            }
            const method = descriptor.value;
            if (!typeError(() => Reflect.construct(function () {}, [], method)) ||
                !typeError(() => Reflect.apply(method, valid, []))) {
                return 'nonconstructible method and required argument';
            }
            for (const invalid of [null, {}, other, Object.create(Object.getPrototypeOf(valid)),
                new Proxy(valid, {}), document.body]) {
                let coerced = false;
                const argument = {toString() { coerced = true; return 'id'; }};
                if (!typeError(() => Reflect.apply(method, invalid, [argument])) || coerced) {
                    return 'native receiver check precedes DOMString conversion';
                }
            }
        }
        if (doc.value === frag.value || document.getElementById !== doc.value ||
            fragment.getElementById !== frag.value || shadow.getElementById !== frag.value ||
            Object.hasOwn(ShadowRoot.prototype, 'getElementById') || 'getElementById' in Element.prototype) {
            return 'Document and Fragment inclusion with inherited ShadowRoot operation';
        }

        for (const name of ['null', 'undefined']) {
            const found = fragment.appendChild(document.createElement('i'));
            found.setAttributeNS(null, 'id', name);
            if (fragment.getElementById(name === 'null' ? null : undefined) !== found ||
                document.getElementById(name) !== null) return 'explicit null and undefined DOMString in detached Fragment';
        }
        return 'ok';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const add = (parent, local) => parent.appendChild(document.createElement(local));
        const parent = add(document.body, 'div');
        const wrong = add(parent, 'span');
        const foreign = parent.appendChild(document.createElementNS('http://www.w3.org/2000/svg', 'svg'));
        const second = add(parent, 'span');
        const exact = ' Lookup-\ud800\0\udc00 ';
        wrong.setAttributeNS('urn:id', 'p:id', exact);
        wrong.setAttributeNS(null, 'ID', exact);
        const blank = add(parent, 'span');
        blank.setAttributeNS(null, 'id', '');
        foreign.setAttributeNS(null, 'id', exact);
        second.setAttributeNS(null, 'id', exact);
        if (document.getElementById(exact) !== foreign || document.getElementById('') !== null ||
            document.getElementById(exact.trim()) !== null ||
            document.getElementById(exact.replace('L', 'l')) !== null) {
            return 'first exact nonempty null-namespace ID across element namespaces';
        }

        let conversions = 0;
        const shifted = document.getElementById({[Symbol.toPrimitive](hint) {
            if (hint !== 'string') throw Error('wrong ID conversion hint');
            conversions++;
            parent.insertBefore(second, foreign);
            return exact;
        }});
        if (conversions !== 1 || shifted !== second || document.getElementById(exact) !== second) {
            return 'search uses tree order after DOMString reentry';
        }

        const thrown = {};
        try {
            document.getElementById({toString() {
                second.removeAttributeNS(null, 'id');
                throw thrown;
            }});
            return 'conversion throw did not propagate';
        } catch (error) {
            if (error !== thrown) return 'conversion throw identity';
        }
        if (document.getElementById(exact) !== foreign) return 'accepted reentrant ID removal remains visible';
        const gone = document.getElementById({toString() {
            parent.removeChild(foreign);
            return exact;
        }});
        if (gone !== null || foreign.parentNode !== null) return 'removed match is absent after coercion';
        return 'ok';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const add = (parent, local, id) => {
            const node = parent.appendChild(document.createElement(local));
            node.setAttributeNS(null, 'id', id);
            return node;
        };
        const host = add(document.body, 'div', 'scope-host');
        const root = host.attachShadow({mode: 'open'});
        const hidden = add(root, 'span', 'tree-private');
        const light = add(host, 'span', 'light-private');
        const shared = add(root, 'span', 'ordinary-duplicate');
        const visible = add(host, 'span', 'ordinary-duplicate');
        if (document.getElementById('tree-private') !== null || root.getElementById('tree-private') !== hidden ||
            root.getElementById('light-private') !== null || document.getElementById('light-private') !== light ||
            document.getElementById('ordinary-duplicate') !== visible || root.getElementById('ordinary-duplicate') !== shared) {
            return 'ShadowRoot and Document search separate ordinary trees';
        }

        const template = document.body.appendChild(document.createElement('template'));
        const content = template.content;
        const inTemplate = add(content, 'b', 'template-private');
        if (content.getElementById('template-private') !== inTemplate ||
            document.getElementById('template-private') !== null ||
            root.getElementById('template-private') !== null) return 'Template contents are a separate Fragment';

        const detached = document.createDocumentFragment();
        const nested = add(detached, 'div', 'fragment-container');
        const moved = add(nested, 'span', 'moving-id');
        if (detached.getElementById('moving-id') !== moved || document.getElementById('moving-id') !== null) {
            return 'detached Fragment ordinary descendant and wrapper identity';
        }
        if (detached.getElementById({toString() {
            document.body.appendChild(moved);
            return 'moving-id';
        }}) !== null || document.getElementById('moving-id') !== moved) {
            return 'original Fragment receiver searches its current tree after coercion';
        }
        return 'ok';
    })()"#,
    );

    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
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

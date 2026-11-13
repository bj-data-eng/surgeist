use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

pub(super) fn html_reflections_use_exact_live_attributes_and_html_brands() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 200_000);
    let permit = host.permit().unwrap();
    let element = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const selected = [['title', 'title'], ['lang', 'lang'],
                    ['inert', 'inert'], ['accessKey', 'accesskey']];
                const keys = Object.keys(HTMLElement.prototype);
                let last = -1;
                const html = document.createElement('div');
                const svg = document.createElementNS('http://www.w3.org/2000/svg', 'g');
                const math = document.createElementNS('http://www.w3.org/1998/Math/MathML', 'mrow');
                const generic = document.createElementNS(null, 'div');
                const throws = (run) => {
                    try { run(); } catch (error) { return error instanceof TypeError; }
                    return false;
                };
                for (const [key, attribute] of selected) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLElement.prototype, key);
                    const index = keys.indexOf(key);
                    if (!descriptor || !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.name !== 'get ' + key || descriptor.get.length !== 0 ||
                        descriptor.set.name !== 'set ' + key || descriptor.set.length !== 1 ||
                        index <= last) return 'descriptor ' + key;
                    last = index;
                    let coerced = false;
                    for (const wrong of [svg, math, generic, new Proxy(html, {}), Object.create(html)]) {
                        if (!throws(() => descriptor.get.call(wrong)) ||
                            !throws(() => descriptor.set.call(wrong, {toString() {
                                coerced = true; return 'unexpected';
                            }})) || coerced) return 'receiver ' + key;
                    }
                    html.setAttributeNS('urn:shadow', attribute, 'namespaced');
                    if (descriptor.get.call(html) !== (key === 'inert' ? false : ''))
                        return 'null namespace ' + key;
                }
                const exact = '\ud800\u0000\udc00';
                for (const key of ['title', 'lang', 'accessKey']) {
                    html[key] = exact;
                    if (html[key] !== exact ||
                        html.getAttributeNS(null, key === 'accessKey' ? 'accesskey' : key) !== exact)
                        return 'exact string ' + key;
                }
                html.title = undefined;
                html.lang = null;
                if (html.title !== 'undefined' || html.lang !== 'null') return 'string conversion';
                html.accessKey = {toString() {
                    html.setAttribute('accesskey', 'nested');
                    return exact;
                }};
                if (html.accessKey !== exact) return 'reentry';
                html.inert = 'false';
                if (!html.inert || html.getAttributeNS(null, 'inert') !== '') return 'truthy inert';
                html.inert = 0;
                if (html.inert || html.hasAttributeNS(null, 'inert')) return 'false inert';
                html.inert = {valueOf() {throw Error('unexpected Boolean coercion');}};
                if (!html.inert) return 'object Boolean';
                const unknown = document.createElement('unknown-html-name');
                unknown.title = exact;
                if (!(unknown instanceof HTMLElement) || unknown.title !== exact) return 'inheritance';
                const calls = [];
                customElements.define('x-reflection-observed', class extends HTMLElement {
                    static observedAttributes = ['title', 'lang', 'inert', 'accesskey'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([name, oldValue, newValue, namespace, this.getAttribute(name)]);
                    }
                });
                const watched = document.createElement('x-reflection-observed');
                for (const [key, attr] of selected) {
                    const count = calls.length;
                    watched[key] = key === 'inert' ? true : exact;
                    const value = key === 'inert' ? '' : exact;
                    const change = calls[count];
                    if (calls.length !== count + 1 || change[0] !== attr || change[1] !== null ||
                        change[2] !== value || change[3] !== null || change[4] !== value)
                        return 'reaction ' + key;
                }
                watched.inert = true;
                watched.inert = false;
                if (calls.length !== 6 || calls[4][1] !== '' || calls[4][2] !== '' ||
                    calls[5][1] !== '' || calls[5][2] !== null || watched.inert)
                    return 'Boolean reactions';
                globalThis.reflectionElement = html;
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let value = evaluate(scope, "reflectionElement")?;
        projected_node_id(scope, &value)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(element, Namespace::none(), "title", "native");
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let actual = evaluate(scope, "reflectionElement.title === 'native'")?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    })
    .unwrap();
    host.stop().into_session().unwrap();
    html_reflection_setters_admit_the_native_change_before_publication();
}

fn html_reflection_setters_admit_the_native_change_before_publication() {
    for (property, outputs) in [("title", 1), ("title", 2), ("inert", 1), ("inert", 2)] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let element = batch.create_element(target.document(), "div");
        let element = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&element)
            .unwrap()
            .unwrap();
        let mut host =
            attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, element, "limitedReflection");
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            evaluate(
                scope,
                if property == "title" {
                    "limitedReflection.title = '\\ud800'"
                } else {
                    "limitedReflection.inert = true"
                },
            )
            .map(drop)
        });
        let after = host.snapshot().unwrap();
        if outputs == 1 {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short reflection output must refuse before publication: {result:?}");
            };
            assert!(matches!(
                failure.cause::<crate::DomSourceDriveError>(),
                Some(crate::DomSourceDriveError::Source(
                    crate::DomSourceRuntimeError::Dom(surgeist_dom::DomError::SourceOutputLimit)
                ))
            ));
            assert_eq!(after.stamp(), before.stamp());
            assert_eq!(after.nodes().count(), before.nodes().count());
            assert!(
                after
                    .node(element)
                    .unwrap()
                    .attributes()
                    .unwrap()
                    .is_empty()
            );
        } else {
            result.unwrap();
            let name = property.into();
            let attr = after
                .node(element)
                .unwrap()
                .get_attribute_node_ns(&Namespace::none(), &name)
                .unwrap();
            assert_eq!(
                attr.value().code_units(),
                if property == "title" {
                    &[0xd800][..]
                } else {
                    &[]
                }
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

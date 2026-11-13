use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

pub(super) fn heading_offset_reflects_the_distinct_getter_and_setter_ranges() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 200_000);
    let permit = host.permit().unwrap();
    let element = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"(() => {
            const d = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'headingOffset');
            if (!d || !d.get || !d.set || !d.enumerable || !d.configurable ||
                d.get.length !== 0 || d.set.length !== 1 ||
                d.get.name !== 'get headingOffset' || d.set.name !== 'set headingOffset')
                return 'headingOffset descriptor';
            const keys = Object.keys(HTMLElement.prototype);
            if (!(keys.indexOf('popover') < keys.indexOf('headingOffset') &&
                keys.indexOf('headingOffset') < keys.indexOf('headingReset'))) return 'order';
            const html = document.createElement('div');
            for (const receiver of [{}, Object.create(HTMLElement.prototype),
                document.createElementNS('urn:foreign', 'div')]) {
                let converted = false, getRejected = false, setRejected = false;
                try { d.get.call(receiver); } catch(e) { getRejected = e instanceof TypeError; }
                try { d.set.call(receiver, {valueOf() {converted = true; return 1; }}); }
                catch(e) { setRejected = e instanceof TypeError; }
                if (!getRejected || !setRejected || converted) return 'brand';
            }
            html.setAttributeNS('urn:foreign', 'headingoffset', '7');
            if (html.headingOffset !== 0) return 'missing/null namespace';
            for (const [raw, expected] of [['0',0], ['8',8], ['9',8], [' +12tail',8],
                ['-0',0], ['-1',0], ['',0], ['garbage',0], ['0x10',0], ['7.9',7],
                ['\ud800',0], ['7\ud800',7], ['9'.repeat(100),8]]) {
                html.setAttributeNS(null, 'headingoffset', raw);
                if (html.headingOffset !== expected || html.getAttributeNS(null,'headingoffset') !== raw)
                    return 'get ' + raw;
            }
            for (const [input, stored, expected] of [[-0,'0',0], [7.9,'7',7], [9,'9',8],
                [2147483647,'2147483647',8], [2147483648,'0',0], [4294967295,'0',0],
                [4294967296,'0',0], [4294967297,'1',1], [-1,'0',0], [NaN,'0',0],
                [Infinity,'0',0], [-Infinity,'0',0], [null,'0',0], [undefined,'0',0], [true,'1',1]]) {
                html.headingOffset = input;
                if (html.headingOffset !== expected || html.getAttributeNS(null,'headingoffset') !== stored)
                    return 'set ' + String(input);
            }
            for (const input of [Symbol(), 1n]) {
                let rejected = false;
                try { html.headingOffset = input; } catch(e) { rejected = e instanceof TypeError; }
                if (!rejected || html.getAttributeNS(null,'headingoffset') !== '1') return 'ToNumber';
            }
            html.headingOffset = {valueOf() { html.headingOffset = 4; return 9; }};
            if (html.headingOffset !== 8 || html.getAttributeNS(null,'headingoffset') !== '9')
                return 'conversion reentry';
            const original = new Error('original');
            try { html.headingOffset = {valueOf() {throw original;}}; return 'missing abrupt'; }
            catch(e) { if (e !== original) return 'wrong abrupt'; }
            if (html.getAttributeNS(null,'headingoffset') !== '9') return 'abrupt mutation';
            const calls = [];
            customElements.define('x-heading-offset', class extends HTMLElement {
                static observedAttributes = ['headingoffset'];
                attributeChangedCallback(name, oldValue, newValue, namespace) {
                    calls.push([oldValue,newValue,namespace,this.headingOffset]);
                }
            });
            const custom = document.createElement('x-heading-offset');
            custom.headingOffset = 12;
            custom.headingOffset = 2147483648;
            if (JSON.stringify(calls) !== JSON.stringify([[null,'12',null,8],['12','0',null,0]]))
                return 'synchronous reactions';
            const unknown = document.createElement('heading-offset-unknown');
            unknown.headingOffset = 6;
            if (unknown.headingOffset !== 6) return 'unknown HTML';
            globalThis.headingElement = html;
            return 'ok';
        })()"#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let element = evaluate(scope, "headingElement")?;
        projected_node_id(scope, &element)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(element, Namespace::none(), "headingoffset", " +99tail");
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let actual = evaluate(scope, "headingElement.headingOffset === 8 && headingElement.getAttributeNS(null,'headingoffset') === ' +99tail'")?;
        let yes = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &yes)?);
        Ok(())
    }).unwrap();
    host.stop().into_session().unwrap();
}

pub(super) fn local_keyword_reflections_follow_current_native_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 200_000);
    let permit = host.permit().unwrap();
    let element = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"(() => {
            const html = document.createElement('div');
            const names = ['dir', 'popover', 'headingReset'];
            const keys = Object.keys(HTMLElement.prototype);
            let previous = -1;
            for (const name of names) {
                const d = Object.getOwnPropertyDescriptor(HTMLElement.prototype, name);
                if (!d || !d.get || !d.set || !d.enumerable || !d.configurable ||
                    d.get.length !== 0 || d.set.length !== 1 ||
                    d.get.name !== 'get ' + name || d.set.name !== 'set ' + name ||
                    keys.indexOf(name) <= previous) return 'descriptor ' + name;
                previous = keys.indexOf(name);
                for (const receiver of [{}, Object.create(HTMLElement.prototype),
                    document.createElementNS('urn:foreign', 'div')]) {
                    let converted = false, getRejected = false, setRejected = false;
                    try { d.get.call(receiver); } catch (e) { getRejected = e instanceof TypeError; }
                    try { d.set.call(receiver, {toString() { converted = true; return 'auto'; }}); }
                    catch (e) { setRejected = e instanceof TypeError; }
                    if (!getRejected || !setRejected || converted) return 'receiver ' + name;
                }
            }
            html.setAttributeNS('urn:foreign', 'dir', 'ltr');
            html.setAttributeNS('urn:foreign', 'popover', 'hint');
            html.setAttributeNS('urn:foreign', 'headingreset', '');
            if (html.dir !== '' || html.popover !== null || html.headingReset !== false)
                return 'null namespace';
            for (const [raw, expected] of [['LTR','ltr'], ['rtl','rtl'], ['AuTo','auto'],
                ['', ''], [' ltr',''], ['unknown',''], ['\ud800','']]) {
                html.dir = raw;
                if (html.dir !== expected || html.getAttributeNS(null, 'dir') !== raw)
                    return 'dir ' + raw;
            }
            for (const [raw, expected] of [['','auto'], ['AUTO','auto'], ['MANUAL','manual'],
                ['HiNt','hint'], ['unknown','manual'], [' auto','manual'], ['\ud800','manual']]) {
                html.popover = raw;
                if (html.popover !== expected || html.getAttributeNS(null, 'popover') !== raw)
                    return 'popover ' + raw;
            }
            const exact = '\ud800\u0000\udc00';
            html.dir = {toString() { html.dir = 'rtl'; return exact; }};
            if (html.dir !== '' || html.getAttributeNS(null, 'dir') !== exact) return 'dir reentry';
            html.popover = {toString() { html.popover = 'hint'; return exact; }};
            if (html.popover !== 'manual' || html.getAttributeNS(null, 'popover') !== exact)
                return 'popover reentry';
            for (const value of [null, undefined]) {
                html.popover = 'hint';
                html.popover = value;
                if (html.popover !== null || html.hasAttributeNS(null, 'popover')) return 'nullable';
            }
            html.headingReset = {valueOf() { throw Error('Boolean coercion'); }};
            if (!html.headingReset || html.getAttributeNS(null, 'headingreset') !== '') return 'truthy';
            html.headingReset = 0;
            if (html.headingReset || html.hasAttributeNS(null, 'headingreset')) return 'falsy';
            const unknown = document.createElement('local-reflection-unknown');
            unknown.dir = 'LTR'; unknown.popover = ''; unknown.headingReset = true;
            if (unknown.dir !== 'ltr' || unknown.popover !== 'auto' || !unknown.headingReset)
                return 'unknown HTML';
            const calls = [];
            customElements.define('x-local-reflection', class extends HTMLElement {
                static observedAttributes = ['dir', 'popover', 'headingreset'];
                attributeChangedCallback(name, oldValue, newValue, namespace) {
                    calls.push([name, oldValue, newValue, namespace, this.getAttributeNS(null,name)]);
                }
            });
            const watched = document.createElement('x-local-reflection');
            watched.dir = 'RTL'; watched.popover = ''; watched.headingReset = true;
            watched.popover = null; watched.headingReset = false;
            const expected = [['dir',null,'RTL',null,'RTL'], ['popover',null,'',null,''],
                ['headingreset',null,'',null,''], ['popover','',null,null,null],
                ['headingreset','',null,null,null]];
            if (JSON.stringify(calls) !== JSON.stringify(expected)) return 'synchronous reactions';
            globalThis.localReflection = html;
            return 'ok';
        })()"#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let element = evaluate(scope, "localReflection")?;
        projected_node_id(scope, &element)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(element, Namespace::none(), "dir", "RTL");
        batch.set_attribute_ns(element, Namespace::none(), "popover", "");
        batch.set_attribute_ns(element, Namespace::none(), "headingreset", "native");
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let actual = evaluate(scope, "localReflection.dir === 'rtl' && localReflection.popover === 'auto' && localReflection.headingReset")?;
        let yes = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &yes)?);
        Ok(())
    }).unwrap();
    host.stop().into_session().unwrap();
    nullable_popover_removal_admits_source_output();
}

fn nullable_popover_removal_admits_source_output() {
    // Removal changes the element's attribute order and the detached Attr's
    // owner. Both canonical node changes need Source output admission.
    for outputs in [0, 1, 2] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let element = batch.create_element(target.document(), "div");
        batch.set_attribute_ns(&element, Namespace::none(), "popover", "hint");
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
        expose_public_node_for_source_test(&mut host, &permit, element, "removalReflection");
        let before = host.snapshot().unwrap().stamp();
        let result = host.in_document(&permit, |scope| {
            evaluate(scope, "removalReflection.popover = null").map(drop)
        });
        let after = host.snapshot().unwrap();
        let attribute = after
            .node(element)
            .unwrap()
            .get_attribute_node_ns(&Namespace::none(), &"popover".into());
        if outputs < 2 {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("removal must refuse before Source publication: {result:?}");
            };
            assert!(matches!(
                failure.cause::<crate::DomSourceDriveError>(),
                Some(crate::DomSourceDriveError::Source(
                    crate::DomSourceRuntimeError::Dom(surgeist_dom::DomError::SourceOutputLimit)
                ))
            ));
            assert_eq!(after.stamp(), before);
            assert_eq!(
                attribute.unwrap().value().code_units(),
                &[0x68, 0x69, 0x6e, 0x74]
            );
        } else {
            result.unwrap();
            assert!(attribute.is_none());
        }
        host.stop().into_session().unwrap();
    }
}

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
    for (property, attribute, source, expected) in [
        (
            "title",
            "title",
            "limitedReflection.title = '\\ud800'",
            &[0xd800][..],
        ),
        ("inert", "inert", "limitedReflection.inert = true", &[][..]),
        (
            "dir",
            "dir",
            "limitedReflection.dir = 'RTL'",
            &[0x52, 0x54, 0x4c][..],
        ),
        (
            "popover",
            "popover",
            "limitedReflection.popover = '\\ud800'",
            &[0xd800][..],
        ),
        (
            "headingOffset",
            "headingoffset",
            "limitedReflection.headingOffset = 12",
            &[0x31, 0x32][..],
        ),
        (
            "headingReset",
            "headingreset",
            "limitedReflection.headingReset = true",
            &[][..],
        ),
    ] {
        for outputs in [1, 2] {
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
            let result = host.in_document(&permit, |scope| evaluate(scope, source).map(drop));
            let after = host.snapshot().unwrap();
            if outputs == 1 {
                let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                    panic!(
                        "one-short reflection output must refuse before publication: {result:?}"
                    );
                };
                assert!(matches!(
                    failure.cause::<crate::DomSourceDriveError>(),
                    Some(crate::DomSourceDriveError::Source(
                        crate::DomSourceRuntimeError::Dom(
                            surgeist_dom::DomError::SourceOutputLimit
                        )
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
                let name = attribute.into();
                let attr = after
                    .node(element)
                    .unwrap()
                    .get_attribute_node_ns(&Namespace::none(), &name)
                    .unwrap();
                assert_eq!(
                    attr.value().code_units(),
                    expected,
                    "{property} native content"
                );
            }
            host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
                .unwrap();
            host.stop().into_session().unwrap();
        }
    }
}

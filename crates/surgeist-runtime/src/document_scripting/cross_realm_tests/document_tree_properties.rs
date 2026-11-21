use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, fund_guest_operation_after_setup, guest_backing_used,
    projected_node_id, refuse_next_source_node_validation,
};

// HTML877d tree accessors select direct HTML children. Document direction
// reflects the html element; obsolete color properties reflect the body.
pub(super) fn document_tree_properties_select_and_forward_native_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let snapshot = session.snapshot();
    let html = snapshot
        .document(target.document())
        .unwrap()
        .document_element()
        .unwrap();
    let children = snapshot.node(html).unwrap().children();
    let head = children[0];
    let body = children[1];
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(html, Namespace::none(), "dir", "RtL");
    for (attribute, value) in [
        ("text", " raw text "),
        ("link", " raw link "),
        ("vlink", " raw vlink "),
        ("alink", " raw alink "),
        ("bgcolor", " raw bgcolor "),
    ] {
        batch.set_attribute_ns(body, Namespace::none(), attribute, value);
    }
    session.dom().apply(batch, &cancel).unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    for (node, name) in [
        (html, "expectedHtml"),
        (head, "expectedHead"),
        (body, "expectedBody"),
    ] {
        expose_public_node_for_source_test(&mut host, &permit, node, name);
    }
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                if (!(document instanceof Document) || document.documentElement !== expectedHtml ||
                    expectedHtml.localName !== 'html' || expectedHead.localName !== 'head' ||
                    expectedBody.localName !== 'body') return 'native document fixture';
                for (const member of ['head', 'body', 'dir', 'title', 'fgColor', 'linkColor', 'vlinkColor', 'alinkColor', 'bgColor']) {
                    const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, member);
                    if (typeof descriptor?.get !== 'function') return 'document missing tree accessors';
                    if (!descriptor.enumerable || !descriptor.configurable || descriptor.get.length !== 0 ||
                        descriptor.get.name !== 'get ' + member ||
                        (member === 'head' ? descriptor.set !== undefined :
                            typeof descriptor.set !== 'function' || descriptor.set.length !== 1 ||
                            descriptor.set.name !== 'set ' + member)) return 'document accessor descriptor ' + member;
                }
                if (document.head !== expectedHead || document.body !== expectedBody)
                    return 'native head and body identity';
                document.body = expectedBody;
                if (document.body !== expectedBody || expectedBody.parentNode !== expectedHtml)
                    return 'same body identity';
                if (document.dir !== 'rtl' || expectedHtml.getAttributeNS(null, 'dir') !== 'RtL')
                    return 'direction keyword and raw attribute';
                document.dir = 'AuTo';
                if (document.dir !== 'auto' || expectedHtml.getAttributeNS(null, 'dir') !== 'AuTo')
                    return 'direction setter stores raw text';
                document.dir = null;
                if (document.dir !== '' || expectedHtml.getAttributeNS(null, 'dir') !== 'null')
                    return 'direction ordinary DOMString null';
                for (const [member, attribute, raw] of [
                    ['fgColor', 'text', ' raw text '], ['linkColor', 'link', ' raw link '],
                    ['vlinkColor', 'vlink', ' raw vlink '], ['alinkColor', 'alink', ' raw alink '],
                    ['bgColor', 'bgcolor', ' raw bgcolor ']]) {
                    if (document[member] !== raw) return 'raw body color getter ' + member;
                    document[member] = null;
                    if (document[member] !== '' || expectedBody.getAttributeNS(null, attribute) !== '')
                        return 'color null to empty ' + member;
                    const value = 'Not-A-Color ' + member;
                    document[member] = value;
                    if (document[member] !== value || expectedBody.getAttributeNS(null, attribute) !== value ||
                        expectedHtml.hasAttributeNS(null, attribute)) return 'exact body color forwarding ' + member;
                }
                const empty = new Document();
                return empty.head === null && empty.body === null && empty.dir === '' &&
                    empty.fgColor === '' && empty.linkColor === '' && empty.vlinkColor === '' &&
                    empty.alinkColor === '' && empty.bgColor === '' ? 'ok' : 'empty document defaults';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

// Pinned HTML source: selectors 12367..12566, dir 15280..15289,
// and obsolete body-color forwarding 156237..156300. Shared Source and
// conversion admission/cancellation cohorts run in the same entry-thread suite.
pub(super) fn document_tree_properties_preserve_selection_conversion_and_source_contracts() {
    document_tree_descriptors_and_brands_precede_conversion();
    document_tree_selects_direct_children_in_actual_order();
    document_tree_rejects_non_html_roots_and_wrong_expanded_names();
    document_body_converts_nullable_html_elements_before_hierarchy();
    document_body_replaces_appends_adopts_and_preserves_identity();
    document_body_uses_native_source_hierarchy_validation();
    document_dir_keeps_known_keywords_separate_from_raw_strings();
    document_dir_selects_the_postconversion_html_root();
    for (member, attribute) in [
        ("fgColor", "text"),
        ("linkColor", "link"),
        ("vlinkColor", "vlink"),
        ("alinkColor", "alink"),
        ("bgColor", "bgcolor"),
    ] {
        document_colors_reflect_raw_dom_strings(member, attribute);
        document_colors_convert_before_noop_and_fresh_selection(member, attribute);
    }
    document_forwarding_delivers_original_reactions_without_reselection();
    document_body_replacement_delivers_connection_reactions();
    document_tree_reads_follow_native_edits_and_retirement();
    document_tree_accessors_use_registered_function_and_creation_realms();
    document_tree_getter_projection_refusal_keeps_native_state();
    document_tree_body_operand_refusal_keeps_native_state();
    document_tree_source_output_and_work_admission_precede_publication();
}

fn check_document_tree_script(label: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, script)?;
        assert_eq!(
            String::from_utf16_lossy(&scope.to_utf16(&result)?),
            "ok",
            "{label}"
        );
        Ok(())
    })
    .unwrap_or_else(|error| match &error {
        DocumentScriptError::Script(Error::HostFailure(failure)) => panic!(
            "document tree {label}: guest={:?}; source={:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::internal::dom_source::DomSourceDriveError>()
        ),
        _ => panic!("document tree {label}: {error:?}"),
    });
}

fn document_tree_descriptors_and_brands_precede_conversion() {
    check_document_tree_script(
        "descriptors and brands",
        r#"
        (() => {
            // HTML877d IDL 11689..11691 declares dir, body, then head.
            const members = ['dir', 'body', 'head', 'fgColor', 'linkColor', 'vlinkColor', 'alinkColor', 'bgColor'];
            const keys = Object.keys(Document.prototype);
            let previous = -1;
            const html = document.implementation.createHTMLDocument();
            const xml = document.implementation.createDocument(null, 'root');
            if (!(html instanceof Document) || !(xml instanceof XMLDocument) || !(xml instanceof Document) ||
                Object.getPrototypeOf(XMLDocument.prototype) !== Document.prototype) return 'Document interface inheritance';
            const element = document.createElement('div');
            Object.setPrototypeOf(element, Document.prototype);
            const object = Object.create(Document.prototype);
            for (const member of members) {
                const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, member);
                const index = keys.indexOf(member);
                if (index <= previous || !descriptor.enumerable || !descriptor.configurable ||
                    descriptor.get.name !== 'get ' + member || descriptor.get.length !== 0 ||
                    (member === 'head' ? descriptor.set !== undefined :
                        descriptor.set.name !== 'set ' + member || descriptor.set.length !== 1))
                    return 'ordered accessor declarations ' + member;
                previous = index;
                for (const receiver of [null, {}, element, object, Object.create(document), new Proxy(document, {})]) {
                    let coerced = false;
                    try { descriptor.get.call(receiver); return 'getter accepted wrong Document'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'getter brand error'; }
                    if (descriptor.set) {
                        try { descriptor.set.call(receiver, {[Symbol.toPrimitive]() { coerced = true; throw 0; },
                            toString() { coerced = true; throw 0; }, valueOf() { coerced = true; throw 0; }});
                            return 'setter accepted wrong Document'; }
                        catch (error) { if (!(error instanceof TypeError) || coerced) return 'Document before conversion'; }
                    }
                }
                if (member === 'head' || member === 'body') {
                    if (descriptor.get.call(xml) !== null) return 'inherited XMLDocument selector';
                } else {
                    if (descriptor.get.call(xml) !== '') return 'inherited XMLDocument string getter';
                    descriptor.set.call(xml, 'unused');
                    if (descriptor.get.call(xml) !== '') return 'non-html XML root no-op';
                }
            }
            const head = document.head;
            if (Reflect.set(document, 'head', document.body) !== false || document.head !== head || Object.hasOwn(document, 'head'))
                return 'readonly head reflection';
            try { (function() { 'use strict'; document.head = null; })(); return 'strict readonly assignment'; }
            catch (error) { if (!(error instanceof TypeError) || document.head !== head) return 'readonly TypeError'; }
            return 'ok';
        })()
    "#,
    );
}

fn document_tree_selects_direct_children_in_actual_order() {
    check_document_tree_script(
        "direct children and order",
        r#"
        (() => {
            const ns = 'http://www.w3.org/1999/xhtml';
            const doc = document.implementation.createDocument(ns, 'p:html');
            const root = doc.documentElement;
            root.appendChild(doc.createTextNode('text'));
            root.appendChild(doc.createComment('comment'));
            const nested = root.appendChild(doc.createElementNS(ns, 'div'));
            nested.appendChild(doc.createElementNS(ns, 'head'));
            nested.appendChild(doc.createElementNS(ns, 'body'));
            const foreignHead = root.appendChild(doc.createElementNS('urn:foreign', 'head'));
            const upperBody = root.appendChild(doc.createElementNS(ns, 'BODY'));
            if (doc.head !== null || doc.body !== null) return 'nested and wrong expanded names excluded';
            const head = root.appendChild(doc.createElementNS(ns, 'p:head'));
            const secondHead = root.appendChild(doc.createElementNS(ns, 'head'));
            const frameset = root.appendChild(doc.createElementNS(ns, 'f:frameset'));
            const body = root.appendChild(doc.createElementNS(ns, 'b:body'));
            if (doc.head !== head || doc.body !== frameset || head.prefix !== 'p' || body.prefix !== 'b')
                return 'prefix independent selection in actual order';
            root.insertBefore(body, frameset);
            root.insertBefore(secondHead, head);
            if (doc.head !== secondHead || doc.body !== body) return 'live reordered first child';
            doc.dir = 'RTL'; doc.bgColor = 'not a color';
            if (doc.dir !== 'rtl' || root.getAttributeNS(null, 'dir') !== 'RTL' ||
                doc.bgColor !== 'not a color' || body.getAttributeNS(null, 'bgcolor') !== 'not a color')
                return 'XMLDocument with HTML root forwards normally';
            root.removeChild(secondHead); root.removeChild(body);
            if (doc.head !== head || doc.body !== frameset) return 'live removal fallback';
            frameset.setAttributeNS(null, 'text', 'frameset is excluded');
            doc.fgColor = 'ignored';
            if (doc.fgColor !== '' || frameset.getAttributeNS(null, 'text') !== 'frameset is excluded')
                return 'frameset body selector does not authorize colors';
            return foreignHead.parentNode === root && upperBody.parentNode === root ? 'ok' : 'nonselected children retained';
        })()
    "#,
    );
}

fn document_tree_rejects_non_html_roots_and_wrong_expanded_names() {
    for (namespace, qualified) in [
        ("urn:foreign", "html"),
        ("http://www.w3.org/1999/xhtml", "HTML"),
        ("http://www.w3.org/1999/xhtml", "div"),
        ("http://www.w3.org/2000/svg", "svg"),
    ] {
        let script = format!(
            r#"
            (() => {{
                const doc = document.implementation.createDocument('{namespace}', '{qualified}');
                const root = doc.documentElement;
                const ns = 'http://www.w3.org/1999/xhtml';
                root.appendChild(doc.createElementNS(ns, 'head'));
                const body = root.appendChild(doc.createElementNS(ns, 'body'));
                root.setAttributeNS(null, 'dir', 'rtl');
                body.setAttributeNS(null, 'text', 'raw');
                if (doc.head !== null || doc.body !== null || doc.dir !== '' || doc.fgColor !== '')
                    return 'root expanded name';
                doc.dir = 'ltr'; doc.fgColor = 'ignored';
                return root.getAttributeNS(null, 'dir') === 'rtl' && body.getAttributeNS(null, 'text') === 'raw'
                    ? 'ok' : 'no forwarding without html root';
            }})()
        "#
        );
        check_document_tree_script(qualified, &script);
    }
}

fn document_body_converts_nullable_html_elements_before_hierarchy() {
    check_document_tree_script(
        "body argument conversion",
        r#"
        (() => {
            const original = document.body;
            const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'body');
            const throws = (value, name) => {
                try { descriptor.set.call(document, value); return false; }
                catch (error) { return error.name === name && (name !== 'TypeError' || error instanceof TypeError) &&
                    document.body === original; }
            };
            for (const value of [null, undefined, document.createElement('div'), document.createElement('head')])
                if (!throws(value, 'HierarchyRequestError')) return 'nullable HTMLElement HTML hierarchy';
            let coerced = false;
            const forged = Object.create(HTMLBodyElement.prototype);
            const other = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            Object.setPrototypeOf(other, HTMLBodyElement.prototype);
            const object = {[Symbol.toPrimitive]() { coerced = true; throw 0; },
                toString() { coerced = true; throw 0; }, valueOf() { coerced = true; throw 0; }};
            for (const value of [object, forged, other, new Proxy(original, {}), document,
                document.createTextNode('body'), 'body', 1, Symbol(), 1n,
                document.createElementNS('urn:foreign', 'body')]) {
                if (!throws(value, 'TypeError') || coerced) return 'HTMLElement conversion without coercion';
            }
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'BODY');
            return upper instanceof HTMLElement && throws(upper, 'HierarchyRequestError')
                ? 'ok' : 'HTML case sensitivity after interface conversion';
        })()
    "#,
    );
}

fn document_body_replaces_appends_adopts_and_preserves_identity() {
    check_document_tree_script(
        "body replace and adoption",
        r#"
        (() => {
            const old = document.body;
            const head = document.head;
            const next = old.nextSibling;
            const foreign = document.implementation.createHTMLDocument();
            const incoming = foreign.body;
            const prototype = Object.getPrototypeOf(incoming);
            incoming.appendChild(foreign.createTextNode('retained'));
            document.body = incoming;
            if (document.body !== incoming || incoming.ownerDocument !== document || incoming.parentNode !== document.documentElement ||
                Object.getPrototypeOf(incoming) !== prototype || incoming.firstChild.ownerDocument !== document ||
                incoming.firstChild.data !== 'retained' || old.parentNode !== null || foreign.body !== null ||
                document.head !== head || incoming.nextSibling !== next) return 'replace with original adopted subtree';
            document.body = incoming;
            if (document.body !== incoming) return 'same body identity return';
            const frameset = document.createElement('frameset');
            document.body = frameset;
            if (document.body !== frameset || incoming.parentNode !== null) return 'frameset replacement';
            document.documentElement.removeChild(frameset);
            document.body = incoming;
            if (document.body !== incoming || document.documentElement.lastChild !== incoming)
                return 'append when no selected body';
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:body');
            document.body = prefixed;
            if (document.body !== prefixed || prefixed.prefix !== 'p' || incoming.parentNode !== null)
                return 'prefixed HTML body setter';
            const empty = new Document();
            const unattached = document.createElement('body');
            try { empty.body = unattached; return 'missing document element accepted'; }
            catch (error) { if (error.name !== 'HierarchyRequestError' || unattached.parentNode !== null)
                return 'no-root hierarchy branch'; }
            const svg = document.implementation.createDocument('http://www.w3.org/2000/svg', 'svg');
            svg.body = unattached;
            return svg.body === null && unattached.parentNode === svg.documentElement && unattached.ownerDocument === svg
                ? 'ok' : 'append under non-html root with null getter';
        })()
    "#,
    );
}

fn document_body_uses_native_source_hierarchy_validation() {
    check_document_tree_script(
        "body native hierarchy",
        r#"
        (() => {
            const doc = new Document();
            const ancestor = doc.appendChild(doc.createElementNS('http://www.w3.org/1999/xhtml', 'body'));
            const original = doc.documentElement;
            try { doc.body = ancestor; return 'cycle accepted'; }
            catch (error) {
                return error.name === 'HierarchyRequestError' && doc.documentElement === original &&
                    doc.body === null && ancestor.parentNode === doc ? 'ok' : 'native cycle failure preserves tree';
            }
        })()
    "#,
    );
}

fn document_dir_keeps_known_keywords_separate_from_raw_strings() {
    check_document_tree_script(
        "dir raw strings",
        r#"
        (() => {
            const root = document.documentElement;
            root.setAttributeNS('urn:foreign', 'dir', 'foreign');
            root.setAttributeNS(null, 'DIR', 'uppercase');
            for (const [input, expected, raw] of [
                ['LTR', 'ltr', 'LTR'], ['rTl', 'rtl', 'rTl'], ['AuTo', 'auto', 'AuTo'],
                ['', '', ''], [' rtl ', '', ' rtl '], ['invalid', '', 'invalid'],
                [null, '', 'null'], [undefined, '', 'undefined'],
                [{toString() { return null; }}, '', 'null'], ['\ud800\0\udc00', '', '\ud800\0\udc00']]) {
                document.dir = input;
                if (document.dir !== expected || root.getAttributeNS(null, 'dir') !== raw ||
                    root.getAttributeNS('urn:foreign', 'dir') !== 'foreign' || root.getAttributeNS(null, 'DIR') !== 'uppercase')
                    return 'limited known values preserve raw storage';
            }
            const raw = root.getAttributeNS(null, 'dir');
            if (raw.length !== 3 || raw.charCodeAt(0) !== 0xd800 || raw.charCodeAt(1) !== 0 ||
                raw.charCodeAt(2) !== 0xdc00) return 'direction exact UTF16 units';
            const original = {};
            for (const input of [Symbol(), {toString() { throw original; }}]) {
                try { document.dir = input; return 'conversion accepted'; }
                catch (error) { if ((typeof input === 'symbol' ? !(error instanceof TypeError) : error !== original) ||
                    root.getAttributeNS(null, 'dir') !== raw) return 'original conversion error before mutation'; }
            }
            root.removeAttributeNS(null, 'dir');
            return document.dir === '' && root.getAttributeNS('urn:foreign', 'dir') === 'foreign'
                ? 'ok' : 'absent direction default';
        })()
    "#,
    );
}

fn document_dir_selects_the_postconversion_html_root() {
    check_document_tree_script(
        "dir fresh target",
        r#"
        (() => {
            const ns = 'http://www.w3.org/1999/xhtml';
            const empty = new Document();
            let calls = 0;
            empty.dir = {toString() { ++calls; return 'rtl'; }};
            if (calls !== 1 || empty.dir !== '' || empty.documentElement !== null) return 'convert before no-root no-op';
            const original = {};
            try { empty.dir = {toString() { throw original; }}; return 'no-root throw'; }
            catch (error) { if (error !== original) return 'no-root original throw'; }
            try { empty.dir = Symbol(); return 'no-root Symbol'; }
            catch (error) { if (!(error instanceof TypeError)) return 'no-root conversion error'; }
            let created;
            empty.dir = {[Symbol.toPrimitive](hint) {
                if (hint !== 'string') throw 0;
                created = empty.appendChild(empty.createElementNS(ns, 'p:html'));
                return 'RTL';
            }};
            if (empty.dir !== 'rtl' || created.getAttributeNS(null, 'dir') !== 'RTL') return 'coercion creates current html root';
            const old = document.documentElement;
            old.setAttributeNS(null, 'dir', 'old');
            const replacement = document.createElement('html');
            document.dir = {toString() {
                document.replaceChild(replacement, old);
                replacement.setAttributeNS(null, 'dir', 'inner');
                return 'AuTo';
            }};
            if (document.documentElement !== replacement || document.dir !== 'auto' ||
                replacement.getAttributeNS(null, 'dir') !== 'AuTo' || old.getAttributeNS(null, 'dir') !== 'old')
                return 'postconversion root selected once';
            try { document.dir = {toString() { replacement.setAttributeNS(null, 'dir', 'LTR'); throw original; }};
                return 'reentrant throw'; }
            catch (error) { if (error !== original || document.dir !== 'ltr' ||
                replacement.getAttributeNS(null, 'dir') !== 'LTR') return 'nested edit survives original throw'; }
            document.dir = {toString() { document.removeChild(replacement); return 'rtl'; }};
            return document.documentElement === null && document.dir === '' &&
                replacement.getAttributeNS(null, 'dir') === 'LTR' ? 'ok' : 'coercion removes target';
        })()
    "#,
    );
}

fn check_document_color_script(member: &str, attribute: &str, script: &str) {
    let script = format!(
        "globalThis.colorMember = '{member}'; globalThis.colorAttribute = '{attribute}';\n{script}"
    );
    check_document_tree_script(member, &script);
}

fn document_colors_reflect_raw_dom_strings(member: &str, attribute: &str) {
    check_document_color_script(
        member,
        attribute,
        r#"
        (() => {
            const body = document.body;
            body.setAttributeNS('urn:foreign', colorAttribute, 'foreign');
            body.setAttributeNS(null, colorAttribute.toUpperCase(), 'uppercase');
            if (document[colorMember] !== '') return 'exact null namespace lowercase forwarding';
            const exact = ' Raw\ud800\0\udc00 ';
            if (exact.length !== 8 || exact.charCodeAt(4) !== 0xd800 || exact.charCodeAt(5) !== 0 ||
                exact.charCodeAt(6) !== 0xdc00) return 'color literal units';
            for (const [input, expected] of [[null, ''], [undefined, 'undefined'],
                [{toString() { return null; }}, 'null'], ['', ''], ['not a color', 'not a color'],
                ['#ABCDEF', '#ABCDEF'], [exact, exact]]) {
                document[colorMember] = input;
                if (document[colorMember] !== expected || body.getAttributeNS(null, colorAttribute) !== expected ||
                    body.getAttributeNS('urn:foreign', colorAttribute) !== 'foreign' ||
                    body.getAttributeNS(null, colorAttribute.toUpperCase()) !== 'uppercase') return 'raw color DOMString mapping';
            }
            const original = {};
            try { document[colorMember] = {toString() { throw original; }}; return 'conversion throw'; }
            catch (error) { if (error !== original || document[colorMember] !== exact) return 'original throw identity'; }
            try { document[colorMember] = Symbol(); return 'Symbol accepted'; }
            catch (error) { if (!(error instanceof TypeError) || document[colorMember] !== exact) return 'Symbol before mutation'; }
            let hint;
            document[colorMember] = {[Symbol.toPrimitive](value) { hint = value; return null; }};
            if (hint !== 'string' || document[colorMember] !== 'null') return 'LegacyNullToEmptyString only applies to original null';
            body.removeAttributeNS(null, colorAttribute);
            return document[colorMember] === '' && body.getAttributeNS('urn:foreign', colorAttribute) === 'foreign'
                ? 'ok' : 'absent color default';
        })()
    "#,
    );
}

fn document_colors_convert_before_noop_and_fresh_selection(member: &str, attribute: &str) {
    check_document_color_script(
        member,
        attribute,
        r#"
        (() => {
            const ns = 'http://www.w3.org/1999/xhtml';
            const empty = new Document();
            let calls = 0;
            empty[colorMember] = {toString() { ++calls; return 'ignored'; }};
            if (calls !== 1 || empty[colorMember] !== '' || empty.documentElement !== null) return 'convert before empty no-op';
            const original = {};
            try { empty[colorMember] = {toString() { throw original; }}; return 'empty throw'; }
            catch (error) { if (error !== original) return 'empty original throw'; }
            try { empty[colorMember] = Symbol(); return 'empty Symbol'; }
            catch (error) { if (!(error instanceof TypeError)) return 'empty Symbol error'; }
            const root = empty.appendChild(empty.createElementNS(ns, 'html'));
            const frameset = root.appendChild(empty.createElementNS(ns, 'frameset'));
            const laterBody = root.appendChild(empty.createElementNS(ns, 'body'));
            frameset.setAttributeNS(null, colorAttribute, 'frameset');
            laterBody.setAttributeNS(null, colorAttribute, 'later');
            empty[colorMember] = {toString() { ++calls; return 'ignored'; }};
            if (calls !== 2 || empty[colorMember] !== '' || frameset.getAttributeNS(null, colorAttribute) !== 'frameset' ||
                laterBody.getAttributeNS(null, colorAttribute) !== 'later') return 'frameset selected excludes later body';
            root.removeChild(frameset); root.removeChild(laterBody);
            empty[colorMember] = {toString() { empty.body = laterBody; return 'created'; }};
            if (empty[colorMember] !== 'created' || laterBody.getAttributeNS(null, colorAttribute) !== 'created')
                return 'coercion supplies previously missing body';
            const old = document.body;
            old.setAttributeNS(null, colorAttribute, 'old');
            const replacement = document.createElement('body');
            document[colorMember] = {toString() { document.body = replacement; return 'new'; }};
            if (document.body !== replacement || document[colorMember] !== 'new' ||
                replacement.getAttributeNS(null, colorAttribute) !== 'new' || old.getAttributeNS(null, colorAttribute) !== 'old')
                return 'postconversion body selection';
            const changedToFrameset = document.createElement('frameset');
            document[colorMember] = {toString() { document.body = changedToFrameset; return 'ignored'; }};
            if (document[colorMember] !== '' || changedToFrameset.hasAttributeNS(null, colorAttribute))
                return 'postconversion frameset no-op';
            try { document[colorMember] = {toString() {
                document.body = replacement;
                replacement.setAttributeNS(null, colorAttribute, 'retained');
                throw original;
            }}; return 'missing throw'; }
            catch (error) { return error === original && document.body === replacement &&
                document[colorMember] === 'retained' ? 'ok' : 'nested edit and original throw'; }
        })()
    "#,
    );
}

fn document_forwarding_delivers_original_reactions_without_reselection() {
    check_document_tree_script(
        "dir reaction reentry",
        r#"
        (() => {
            const calls = [];
            let reenter = false;
            const replacement = document.createElement('html');
            replacement.setAttributeNS(null, 'dir', 'RTL');
            class Root extends HTMLHtmlElement {
                static observedAttributes = ['dir'];
                attributeChangedCallback(name, oldValue, newValue, namespace) {
                    calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                    if (reenter) { reenter = false; document.replaceChild(replacement, this); }
                }
            }
            customElements.define('x-tree-root', Root, {extends: 'html'});
            const root = new Root();
            document.replaceChild(root, document.documentElement);
            document.dir = 'AuTo';
            document.dir = 'AuTo';
            if (calls.length !== 2 || calls[0][0] !== root || calls[0][1] !== 'dir' ||
                calls[0][2] !== null || calls[0][3] !== 'AuTo' || calls[0][4] !== null || calls[0][5] !== 'AuTo' ||
                calls[1][2] !== 'AuTo' || calls[1][3] !== 'AuTo') return 'original same-value direction reactions';
            reenter = true;
            document.dir = 'LTR';
            return calls.length === 3 && root.parentNode === null && root.getAttributeNS(null, 'dir') === 'LTR' &&
                document.documentElement === replacement && document.dir === 'rtl' &&
                replacement.getAttributeNS(null, 'dir') === 'RTL' ? 'ok' : 'accepted target not reselected after reaction';
        })()
    "#,
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const calls = [];
                let reenter = false, throwNext = false;
                const replacement = document.createElement('body');
                replacement.setAttributeNS(null, 'text', 'replacement');
                class Body extends HTMLBodyElement {
                    static observedAttributes = ['text', 'link', 'vlink', 'alink', 'bgcolor'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                        if (reenter) { reenter = false; document.body = replacement; }
                        if (throwNext) { throwNext = false; document.body.setAttributeNS(null, 'link', 'nested');
                            throw globalThis.treeColorReactionThrow = {}; }
                    }
                }
                customElements.define('x-tree-body', Body, {extends: 'body'});
                const body = document.createElement('body', {is: 'x-tree-body'});
                document.body = body;
                for (const [member, attribute] of [['fgColor', 'text'], ['linkColor', 'link'],
                    ['vlinkColor', 'vlink'], ['alinkColor', 'alink'], ['bgColor', 'bgcolor']]) {
                    document[member] = null;
                    const call = calls[calls.length - 1];
                    if (call[0] !== body || call[1] !== attribute || call[2] !== null || call[3] !== '' ||
                        call[4] !== null || call[5] !== '') return 'forwarded original body reaction ' + member;
                }
                document.fgColor = null;
                if (calls.length !== 6 || calls[5][2] !== '' || calls[5][3] !== '') return 'same-value color reaction';
                reenter = true;
                document.fgColor = 'outer';
                if (calls.length !== 7 || body.getAttributeNS(null, 'text') !== 'outer' ||
                    document.body !== replacement || document.fgColor !== 'replacement') return 'body callback reentry without replay';
                document.body = body;
                throwNext = true;
                document.bgColor = 'reported';
                return document.bgColor === 'reported' && document.linkColor === 'nested' && calls.length === 9
                    ? 'ok' : 'nested Source retained through callback report';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    let report = host
        .take_guest_custom_element_callback_exception(&permit)
        .unwrap()
        .unwrap();
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.in_document(&permit, |scope| {
        let original = evaluate(scope, "treeColorReactionThrow")?;
        let reported = scope.import(report.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        Ok(())
    })
    .unwrap();
}

fn document_body_replacement_delivers_connection_reactions() {
    check_document_tree_script(
        "body connection reactions",
        r#"
        (() => {
            const calls = [];
            class Body extends HTMLBodyElement {
                connectedCallback() { calls.push(['connected', this, this.ownerDocument]); }
                disconnectedCallback() { calls.push(['disconnected', this, this.ownerDocument]); }
                adoptedCallback(oldDocument, newDocument) { calls.push(['adopted', this, oldDocument, newDocument]); }
            }
            customElements.define('x-adopted-tree-body', Body, {extends: 'body'});
            const body = new Body();
            const other = document.implementation.createHTMLDocument();
            other.body = body;
            calls.length = 0;
            document.body = body;
            if (document.body !== body || body.ownerDocument !== document || other.body !== null ||
                calls.length !== 3 || calls[0][0] !== 'disconnected' || calls[1][0] !== 'adopted' ||
                calls[1][2] !== other || calls[1][3] !== document || calls[2][0] !== 'connected' ||
                calls.some(call => call[1] !== body)) return 'original disconnect adopt connect reactions';
            calls.length = 0;
            document.body = body;
            return calls.length === 0 ? 'ok' : 'same identity body setter is a no-op';
        })()
    "#,
    );
}

fn document_tree_reads_follow_native_edits_and_retirement() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let snapshot = session.snapshot();
    let html = snapshot
        .document(target.document())
        .unwrap()
        .document_element()
        .unwrap();
    let old_head = snapshot.node(html).unwrap().children()[0];
    let old_body = snapshot.node(html).unwrap().children()[1];
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| evaluate(scope,
        "globalThis.savedTreeDocument = document; globalThis.oldTreeHead = document.head; globalThis.oldTreeBody = document.body").map(drop)).unwrap();
    let (head, frameset) = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let head = batch.create_element_ns(target.document(), Namespace::html(), "head");
            let frameset =
                batch.create_element_ns(target.document(), Namespace::html(), "frameset");
            batch.insert_before(&head, html, Some(old_head.into()));
            batch.insert_before(&frameset, html, Some(old_body.into()));
            batch.set_attribute_ns(html, Namespace::none(), "dir", "RtL");
            batch.set_attribute_ns(old_body, Namespace::none(), "text", "hidden body");
            let commit = dom.apply(batch, &cancel).unwrap();
            (
                commit.created(&head).unwrap().unwrap(),
                commit.created(&frameset).unwrap().unwrap(),
            )
        })
        .unwrap();
    expose_public_node_for_source_test(&mut host, &permit, head, "nativeTreeHead");
    expose_public_node_for_source_test(&mut host, &permit, frameset, "nativeTreeFrameset");
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            savedTreeDocument === document && document.head === nativeTreeHead && document.body === nativeTreeFrameset &&
            oldTreeHead.parentNode === document.documentElement && oldTreeBody.parentNode === document.documentElement &&
            document.dir === 'rtl' && document.fgColor === ''
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(head);
        batch.destroy(head);
        batch.detach(frameset);
        batch.destroy(frameset);
        batch.set_attribute_ns(
            old_body,
            Namespace::none(),
            "text",
            surgeist_html::values::DocumentString::from_utf16([0xd800, 0, 0xdc00]),
        );
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            document.head === oldTreeHead && document.body === oldTreeBody && document.fgColor === '\ud800\0\udc00' &&
            document.fgColor.length === 3 && document.fgColor.charCodeAt(0) === 0xd800 &&
            document.fgColor.charCodeAt(1) === 0 && document.fgColor.charCodeAt(2) === 0xdc00
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    let retained = host.in_document(&permit, |scope| {
        evaluate(scope, r#"
            globalThis.retainedTreeDocument = new Document();
            globalThis.savedRetainedTreeDocument = retainedTreeDocument;
            const root = retainedTreeDocument.appendChild(retainedTreeDocument.createElementNS('http://www.w3.org/1999/xhtml', 'html'));
            root.appendChild(retainedTreeDocument.createElementNS('http://www.w3.org/1999/xhtml', 'body'));
        "#)?;
        let value = evaluate(scope, "retainedTreeDocument")?;
        projected_node_id(scope, &value)
    }).unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.with_native(|dom, _, _| {
        // DomBatch::destroy accepts detached non-Document subtrees.
        let mut batch = DomBatch::new();
        batch.destroy(retained);
        assert!(matches!(
            dom.apply(batch, &cancel),
            Err(DomError::InvalidDestruction)
        ));
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            retainedTreeDocument === savedRetainedTreeDocument && retainedTreeDocument.head === null &&
            retainedTreeDocument.body.ownerDocument === retainedTreeDocument && retainedTreeDocument.dir === '' &&
            retainedTreeDocument.fgColor === '' && retainedTreeDocument.linkColor === '' &&
            retainedTreeDocument.vlinkColor === '' && retainedTreeDocument.alinkColor === '' && retainedTreeDocument.bgColor === ''
        "#)?;
        assert!(scope.boolean(&result)?);
        evaluate(scope, "retainedTreeDocument.dir = 'RTL'; retainedTreeDocument.bgColor = 'raw retained'")?;
        let result = evaluate(scope, "retainedTreeDocument.dir === 'rtl' && retainedTreeDocument.bgColor === 'raw retained'")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
}

fn document_tree_accessors_use_registered_function_and_creation_realms() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(
        &mut host,
        &permit,
        origin.document().node(),
        "originTreeDocument",
    );
    let foreign_type_error = {
        let engine = host.engine.as_mut().unwrap();
        let realm = engine
            .creations
            .find(origin.global(), || Ok::<_, Error>(()))
            .unwrap()
            .unwrap();
        engine
            .process
            .execution()
            .in_realm(realm, |scope| {
                let value = evaluate(scope, "TypeError")?;
                Ok(scope.retain(&value))
            })
            .unwrap()
    };
    // Fund the receiver checks after installing both realms.
    fund_guest_operation_after_setup(&host);
    host.in_document(&permit, |scope| {
        let value = evaluate(scope, "originTreeDocument")?;
        assert_ne!(scope.callback_object(&value)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignTreeTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"
            (() => {
                const prototype = Object.getPrototypeOf(originTreeDocument);
                if (originTreeDocument instanceof Document || prototype.constructor.name !== 'Document') return 'registered Document realm';
                const original = {};
                for (const member of ['head', 'body', 'dir', 'fgColor', 'linkColor', 'vlinkColor', 'alinkColor', 'bgColor']) {
                    const descriptor = Object.getOwnPropertyDescriptor(prototype, member);
                    for (const accessor of [descriptor.get, descriptor.set].filter(Boolean)) {
                        try { accessor.call({}, null); return 'wrong foreign receiver accepted'; }
                        catch (error) { if (!(error instanceof foreignTreeTypeError) || error instanceof TypeError)
                            return 'accessor function realm TypeError'; }
                    }
                    if (member !== 'head' && member !== 'body') {
                        try { descriptor.set.call(originTreeDocument, {toString() { throw original; }}); return 'foreign conversion throw'; }
                        catch (error) { if (error !== original) return 'foreign accessor preserves original object'; }
                    }
                }
                const foreignHead = originTreeDocument.head;
                const foreignBody = originTreeDocument.body;
                if (foreignHead.ownerDocument !== originTreeDocument || foreignBody.ownerDocument !== originTreeDocument ||
                    foreignHead instanceof HTMLHeadElement || foreignBody instanceof HTMLBodyElement ||
                    Object.getPrototypeOf(foreignHead).constructor.name !== 'HTMLHeadElement') return 'selected node creation realm';
                const borrowedBody = Object.getOwnPropertyDescriptor(prototype, 'body');
                try { borrowedBody.set.call(originTreeDocument, {}); return 'foreign interface conversion accepted'; }
                catch (error) { if (!(error instanceof foreignTreeTypeError) || error instanceof TypeError)
                    return 'interface conversion error realm'; }
                const incoming = document.createElement('body');
                const saved = Object.getPrototypeOf(incoming);
                borrowedBody.set.call(originTreeDocument, incoming);
                if (originTreeDocument.body !== incoming || incoming.ownerDocument !== originTreeDocument ||
                    Object.getPrototypeOf(incoming) !== saved || !(incoming instanceof HTMLBodyElement))
                    return 'body adoption preserves wrapper creation realm';
                const dir = Object.getOwnPropertyDescriptor(prototype, 'dir');
                const color = Object.getOwnPropertyDescriptor(prototype, 'fgColor');
                const title = Object.getOwnPropertyDescriptor(prototype, 'title');
                const referrer = Object.getOwnPropertyDescriptor(prototype, 'referrer');
                const cookie = Object.getOwnPropertyDescriptor(prototype, 'cookie');
                try { cookie.set.call(originTreeDocument, Symbol()); return 'Symbol accepted by borrowed cookie setter'; }
                catch (error) { if (!(error instanceof foreignTreeTypeError) || error instanceof TypeError)
                    return 'cookie conversion uses function realm'; }
                if (cookie.get.call(originTreeDocument) !== '' || cookie.set.call(document, 'foreign=1') !== undefined)
                    return 'borrowed cookie access';
                try { referrer.get.call({}); return 'wrong foreign referrer receiver accepted'; }
                catch (error) { if (!(error instanceof foreignTreeTypeError) || error instanceof TypeError)
                    return 'referrer uses accessor function realm'; }
                if (referrer.get.call(originTreeDocument) !== '' || referrer.get.call(document) !== '')
                    return 'referrer uses actual Document receiver';
                dir.set.call(originTreeDocument, 'RTL');
                color.set.call(originTreeDocument, 'foreign\ud800\0');
                const mainTitle = document.title;
                const exact = '\t foreign\udc00\0  title ';
                title.set.call(originTreeDocument, exact);
                if (title.get.call(originTreeDocument) !== 'foreign\udc00\0 title' ||
                    foreignHead.lastChild?.text !== exact || document.title !== mainTitle)
                    return 'borrowed title uses receiver Document';
                const value = color.get.call(originTreeDocument);
                return dir.get.call(originTreeDocument) === 'rtl' && value === 'foreign\ud800\0' &&
                    value.length === 9 && value.charCodeAt(7) === 0xd800 && value.charCodeAt(8) === 0 &&
                    originTreeDocument.body === incoming && document.body !== incoming ? 'ok' : 'borrowed forwarding uses receiver Document';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

fn document_tree_getter_projection_refusal_keeps_native_state() {
    for member in ["head", "body"] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (session, target) = hosted_session(&cancel, 0);
        let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
        let permit = host.permit().unwrap();
        let before = host.snapshot().unwrap();
        let backing = guest_backing_used(&host);
        refuse_next_source_node_validation(&host);
        let result = host.in_document(&permit, |scope| {
            evaluate(scope, &format!("document.{member}")).map(drop)
        });
        assert!(
            matches!(
                result,
                Err(DocumentScriptError::Script(Error::BackendUnavailable))
            ),
            "{member}: {result:?}"
        );
        assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
        assert_eq!(guest_backing_used(&host), backing);
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        // A host error escaping a V8 callback terminally refuses its owner,
        // even though the projection-injection hook itself is one-shot.
        let entered = Cell::new(false);
        let result = host.in_document(&permit, |scope| {
            entered.set(true);
            evaluate(scope, "42").map(drop)
        });
        assert!(
            matches!(
                result,
                Err(DocumentScriptError::Script(Error::BackendUnavailable))
            ),
            "{member} must retain its original host refusal: {result:?}"
        );
        assert!(!entered.get());
        assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
        assert_eq!(guest_backing_used(&host), backing);
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

fn document_tree_body_operand_refusal_keeps_native_state() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let element = host
        .in_document(&permit, |scope| {
            let value = evaluate(
                scope,
                "globalThis.retiredBodyOperand = document.createElement('body')",
            )?;
            projected_node_id(scope, &value)
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(element);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    let backing = guest_backing_used(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "document.body = retiredBodyOperand").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("retired body operand must refuse before Source: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    assert_eq!(guest_backing_used(&host), backing);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}

fn document_tree_source_output_and_work_admission_precede_publication() {
    // New raw attributes publish Created(Attr) plus Attributes(element).
    for (property, attribute, expected) in [
        ("document.dir", "dir", "RTL"),
        ("document.fgColor", "text", "raw color"),
        ("document.body.background", "background", "../raw image.png"),
    ] {
        for outputs in [1, 2] {
            let cancel = Arc::new(AtomicBool::new(false));
            let (session, target) = hosted_session(&cancel, 0);
            let snapshot = session.snapshot();
            let root = snapshot
                .document(target.document())
                .unwrap()
                .document_element()
                .unwrap();
            let element = if property == "document.dir" {
                root
            } else {
                snapshot.node(root).unwrap().children()[1]
            };
            let mut host =
                attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
            let permit = host.permit().unwrap();
            if property == "document.body.background" {
                // Retain the receiver before measuring the setter's backing.
                host.in_document(&permit, |scope| {
                    evaluate(scope, "globalThis.bodyOutputReceiver = document.body").map(drop)
                })
                .unwrap();
            }
            let before = host.snapshot().unwrap();
            let backing = guest_backing_used(&host);
            let result = host.in_document(&permit, |scope| {
                evaluate(scope, &format!("{property} = '{expected}'")).map(drop)
            });
            let after = host.snapshot().unwrap();
            if outputs == 1 {
                let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                    panic!("one-short {property} output: {result:?}");
                };
                assert!(matches!(
                    failure.cause::<crate::internal::dom_source::DomSourceDriveError>(),
                    Some(crate::internal::dom_source::DomSourceDriveError::Source(
                        crate::internal::dom_source::DomSourceRuntimeError::Dom(
                            DomError::SourceOutputLimit
                        )
                    ))
                ));
                assert_eq!(after.stamp(), before.stamp());
                assert_eq!(after.nodes().count(), before.nodes().count());
                assert!(
                    after
                        .node(element)
                        .unwrap()
                        .attribute(&attribute.into())
                        .is_none()
                );
            } else {
                result.unwrap();
                assert_eq!(
                    after.node(element).unwrap().attribute(&attribute.into()),
                    Some(&expected.into())
                );
            }
            assert_eq!(
                guest_backing_used(&host),
                backing,
                "{property} with {outputs} outputs"
            );
            host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
                .unwrap();
            host.stop().into_session().unwrap();
        }
    }
    for replacing in [false, true] {
        // Append changes Parent(incoming) and Children(root). Replacement
        // additionally changes Parent(old), while both operands already exist.
        let exact = if replacing { 3 } else { 2 };
        for outputs in [exact - 1, exact] {
            let cancel = Arc::new(AtomicBool::new(false));
            let (mut session, target) = hosted_session(&cancel, 0);
            let snapshot = session.snapshot();
            let root = snapshot
                .document(target.document())
                .unwrap()
                .document_element()
                .unwrap();
            let old = snapshot.node(root).unwrap().children()[1];
            let mut batch = DomBatch::new();
            if !replacing {
                batch.detach(old);
            }
            let incoming = batch.create_element_ns(target.document(), Namespace::html(), "body");
            let incoming = session
                .dom()
                .apply(batch, &cancel)
                .unwrap()
                .created(&incoming)
                .unwrap()
                .unwrap();
            let mut host =
                attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
            let permit = host.permit().unwrap();
            expose_public_node_for_source_test(&mut host, &permit, incoming, "admittedBodyOperand");
            let before = host.snapshot().unwrap();
            let backing = guest_backing_used(&host);
            let result = host.in_document(&permit, |scope| {
                evaluate(scope, "document.body = admittedBodyOperand").map(drop)
            });
            let after = host.snapshot().unwrap();
            if outputs < exact {
                let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                    panic!("one-short Document body replace={replacing}: {result:?}");
                };
                assert!(matches!(
                    failure.cause::<crate::internal::dom_source::DomSourceDriveError>(),
                    Some(crate::internal::dom_source::DomSourceDriveError::Source(
                        crate::internal::dom_source::DomSourceRuntimeError::Dom(
                            DomError::SourceOutputLimit
                        )
                    ))
                ));
                assert_eq!(after.stamp(), before.stamp());
                assert_eq!(
                    after.node(root).unwrap().children(),
                    before.node(root).unwrap().children()
                );
                assert_eq!(after.node(incoming).unwrap().parent(), None);
            } else {
                result.unwrap();
                assert_eq!(after.node(incoming).unwrap().parent(), Some(root));
                assert_eq!(after.node(old).unwrap().parent(), None);
                assert_eq!(after.node(root).unwrap().children().last(), Some(&incoming));
            }
            assert_eq!(after.nodes().count(), before.nodes().count());
            assert_eq!(guest_backing_used(&host), backing);
            host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
                .unwrap();
            host.stop().into_session().unwrap();
        }
    }
    for member in ["body", "dir", "fgColor"] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let snapshot = session.snapshot();
        let root = snapshot
            .document(target.document())
            .unwrap()
            .document_element()
            .unwrap();
        let current = snapshot.node(root).unwrap().children()[1];
        let mut batch = DomBatch::new();
        let incoming = batch.create_element_ns(target.document(), Namespace::html(), "body");
        let incoming = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&incoming)
            .unwrap()
            .unwrap();
        let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, current, "existingBodyOperand");
        expose_public_node_for_source_test(&mut host, &permit, incoming, "workBodyOperand");
        let before = host.snapshot().unwrap();
        host.in_document(&permit, |scope| {
            let result = evaluate(scope, r#"
                (() => {
                    document.body = existingBodyOperand;
                    try { document.body = null; return 'nullable body accepted'; }
                    catch (error) { if (error.name !== 'HierarchyRequestError') return 'HTML failure before Source work'; }
                    const original = {};
                    for (const member of ['dir', 'fgColor']) {
                        try { document[member] = {toString() { throw original; }}; return 'conversion throw accepted'; }
                        catch (error) { if (error !== original) return 'original conversion precedes Source refusal'; }
                    }
                    return 'ok';
                })()
            "#)?;
            assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
            Ok(())
        }).unwrap();
        assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
        let expression = if member == "body" {
            "document.body = workBodyOperand".to_owned()
        } else {
            format!("document.{member} = 'raw'")
        };
        let result = host.in_document(&permit, |scope| evaluate(scope, &expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("Document {member} must retain original Source work refusal: {result:?}");
        };
        assert!(matches!(
            failure.cause::<crate::internal::dom_source::DomSourceDriveError>(),
            Some(crate::internal::dom_source::DomSourceDriveError::Source(
                crate::internal::dom_source::DomSourceRuntimeError::Dom(
                    DomError::SourceWorkLimit { limit: 0 }
                )
            ))
        ));
        assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
        assert_eq!(
            host.snapshot().unwrap().nodes().count(),
            before.nodes().count()
        );
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

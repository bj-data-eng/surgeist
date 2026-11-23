// Copy to tests/document_scripting/frame_attributes.rs. Register this module
// alongside frameset_reflections. Add "frame-attributes" and
// "frame-attribute-source-refusal" selectors and the corresponding aggregate calls.
//
// Pinned HTML #htmlframeelement defines all six local members. URL members
// belong to #3257. The readonly content getters need the child producers.

use super::{window_length::observe, *};

pub(super) fn frame_attributes_follow_native_identity_and_source() {
    let (session, top) = setup();
    let snapshot = session.snapshot();
    let body = snapshot
        .node(
            snapshot
                .document(top.document())
                .unwrap()
                .document_element()
                .unwrap(),
        )
        .unwrap()
        .children()[1];
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 300_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLFrameElement.prototype;
            const members = [
                ['name', 'name'], ['scrolling', 'scrolling'], ['frameBorder', 'frameborder'],
                ['noResize', 'noresize'], ['marginHeight', 'marginheight'], ['marginWidth', 'marginwidth'],
            ];
            for (const [name] of members) {
                if (typeof Object.getOwnPropertyDescriptor(prototype, name)?.get !== 'function')
                    return 'missing HTMLFrameElement.' + name + ' getter';
            }
            const order = members.map(([name]) => name).concat('constructor');
            if (Object.getOwnPropertyNames(prototype).filter(name => order.includes(name)).join() !== order.join())
                return 'Frame local declaration order';
            for (const name of ['contentDocument', 'contentWindow']) {
                const d = Object.getOwnPropertyDescriptor(prototype, name);
                if (d && (typeof d.get !== 'function' || d.set !== undefined))
                    return 'Frame content access must be readonly ' + name;
            }
            if (Object.hasOwn(prototype, 'sandbox') || Object.hasOwn(prototype, 'srcdoc') ||
                Object.hasOwn(prototype, 'width') || Object.hasOwn(prototype, 'height') ||
                Object.hasOwn(HTMLIFrameElement.prototype, 'noResize'))
                return 'Frame and Iframe expose distinct interfaces';

            const inert = document.implementation.createHTMLDocument();
            const html = 'http://www.w3.org/1999/xhtml';
            const foreign = 'urn:frame-reflection-foreign';
            const frame = inert.createElementNS(html, 'f:frame');
            const wrong = [inert.createElement('iframe'), inert.createElementNS(foreign, 'frame'),
                inert.createElementNS(html, 'FRAME'), new Proxy(frame, {}), Object.create(prototype)];
            Object.setPrototypeOf(wrong[0], prototype);
            Object.setPrototypeOf(wrong[1], prototype);
            if (Object.getPrototypeOf(frame) !== prototype || frame.prefix !== 'f' ||
                frame.ownerDocument !== inert || inert.defaultView !== null || frame.isConnected)
                return 'actual never-hosted native Frame';
            const typeError = action => {
                try { action(); } catch (error) {
                    return Object.getPrototypeOf(error) === TypeError.prototype;
                }
                return false;
            };
            const raw = ' 07\ud800\0\udc00 ';
            for (const [name, attr] of members) {
                const d = Object.getOwnPropertyDescriptor(prototype, name);
                if (typeof d.set !== 'function' || !d.enumerable || !d.configurable ||
                    'value' in d || 'writable' in d || d.get.name !== 'get ' + name ||
                    d.set.name !== 'set ' + name || d.get.length !== 0 || d.set.length !== 1 ||
                    [d.get, d.set].some(f => Object.hasOwn(f, 'prototype') ||
                        !typeError(() => Reflect.construct(function(){}, [], f))))
                    return 'invalid Frame descriptor ' + name;
                for (const receiver of wrong) {
                    let coerced = false;
                    const input = {toString() {coerced = true; return raw;}};
                    if (!typeError(() => d.get.call(receiver)) ||
                        !typeError(() => d.set.call(receiver, input)) || coerced)
                        return 'native Frame receiver before conversion ' + name;
                }

                frame.setAttributeNS(foreign, 'p:' + attr, 'foreign');
                frame.setAttributeNS(null, attr.toUpperCase(), 'uppercase');
                if (frame[name] !== (name === 'noResize' ? false : '') ||
                    frame.hasAttributeNS(null, attr))
                    return 'exact missing Frame attribute ' + name;
                if (name === 'noResize') {
                    frame.noResize = {valueOf() { throw Error('Boolean coerced an object'); }};
                    if (frame.noResize !== true || frame.getAttributeNS(null, attr) !== '')
                        return 'Boolean presence';
                    frame.noResize = false;
                    if (frame.noResize || frame.hasAttributeNS(null, attr)) return 'Boolean removal';
                } else {
                    const nullValue = name === 'marginHeight' || name === 'marginWidth' ? '' : 'null';
                    for (const [value, expected] of [[null, nullValue], [undefined, 'undefined'], [raw, raw]]) {
                        frame[name] = value;
                        if (frame[name] !== expected || frame.getAttributeNS(null, attr) !== expected)
                            return 'exact string conversion ' + name;
                    }
                }
                if (frame.getAttributeNS(foreign, attr) !== 'foreign' ||
                    frame.getAttributeNS(null, attr.toUpperCase()) !== 'uppercase')
                    return 'unrelated Frame attributes changed ' + name;
            }

            const width = Object.getOwnPropertyDescriptor(prototype, 'marginWidth');
            const original = frame.getAttributeNodeNS(null, 'marginwidth');
            const thrown = {};
            try {
                width.set.call(frame, {toString() { throw thrown; }});
                return 'abrupt Frame coercion returned';
            } catch (error) {
                if (error !== thrown || frame.getAttributeNodeNS(null, 'marginwidth') !== original)
                    return 'abrupt Frame coercion mutated the Attr';
            }
            let current;
            let hint;
            Object.setPrototypeOf(frame, HTMLElement.prototype);
            width.set.call(frame, {[Symbol.toPrimitive](kind) {
                hint = kind;
                frame.removeAttributeNS(null, 'marginwidth');
                frame.setAttributeNS(null, 'marginwidth', 'reentrant');
                current = frame.getAttributeNodeNS(null, 'marginwidth');
                return raw;
            }});
            Object.setPrototypeOf(frame, prototype);
            if (hint !== 'string' || original.ownerElement !== null ||
                current === original || frame.getAttributeNodeNS(null, 'marginwidth') !== current ||
                width.get.call(frame) !== raw || current.value !== raw)
                return 'Frame reentry must edit the current Attr on the native receiver';

            frame.setAttributeNS(null, 'src', 'ordinary raw source');
            if (frame.getAttributeNS(null, 'src') !== 'ordinary raw source' || window.length !== 0)
                return 'never-hosted src mutation is purely local';
            frame.removeAttributeNS(null, 'src');
            frame.name = 'nativeFrameProbe';
            frame.setAttributeNS(null, 'data-frame-reflection-probe', 'exact');
            globalThis.frameReflection = frame;
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");

    let frame = host
        .with_native(|dom, _, _| {
            let snapshot = dom.snapshot();
            let namespace = Namespace::none();
            let marker = DocumentString::from("data-frame-reflection-probe");
            let mut found = snapshot.nodes().filter_map(|id| {
                let node = snapshot.node(id).unwrap();
                node.attribute_ns(&namespace, &marker).map(|_| node)
            });
            let frame = found.next().unwrap();
            assert!(found.next().is_none());
            assert_eq!(frame.as_element().unwrap().child_navigable(), Some(None));
            assert_eq!(
                frame
                    .attribute_ns(&namespace, &"marginwidth".into())
                    .unwrap()
                    .code_units(),
                [0x20, 0x30, 0x37, 0xd800, 0, 0xdc00, 0x20]
            );
            let mut batch = DomBatch::new();
            batch.set_attribute_ns(frame.id(), namespace.clone(), "marginheight", "auto 12");
            batch.set_attribute_ns(frame.id(), namespace, "noresize", "native value");
            dom.apply(batch, &cancel()).unwrap();
            frame.id()
        })
        .unwrap();

    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    let snapshot = host.snapshot().unwrap();
    let navigable = window_length::content_navigable(&snapshot, frame).unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const frame = frameReflection;
            const published = document.getElementsByTagNameNS('http://www.w3.org/1999/xhtml', 'frame')[0];
            if (frame !== published || frame.ownerDocument !== document ||
                Object.getPrototypeOf(frame) !== HTMLFrameElement.prototype ||
                frame.marginHeight !== 'auto 12' || !frame.noResize)
                return 'native edits and adoption must preserve Frame identity';
            const child = window[0];
            const other = child.document.createElement('frame');
            const local = Object.getOwnPropertyDescriptor(HTMLFrameElement.prototype, 'marginWidth');
            const borrowed = Object.getOwnPropertyDescriptor(child.HTMLFrameElement.prototype, 'marginWidth');
            if (Object.getPrototypeOf(other) !== child.HTMLFrameElement.prototype ||
                borrowed.set === local.set || Object.getPrototypeOf(borrowed.set) !== child.Function.prototype)
                return 'actual child-realm Frame accessors';
            local.set.call(other, 'child');
            borrowed.set.call(frame, 'attached');
            if (local.get.call(frame) !== 'attached' || borrowed.get.call(other) !== 'child' ||
                window[0] !== child || window.length !== 1)
                return 'borrowed local writes changed the original child association';
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");
    assert_eq!(
        window_length::content_navigable(&host.snapshot().unwrap(), frame),
        Some(navigable)
    );
    assert_original_frame_source_refusal(
        &mut host,
        &permit,
        "frameReflection.setAttributeNS(null, 'src', 'https://example.invalid/frame')",
        DomError::HostedSourceUnavailable,
    );
    assert_eq!(
        window_length::content_navigable(&host.snapshot().unwrap(), frame),
        Some(navigable)
    );
    host.stop().into_session().unwrap();
}

pub(super) fn frame_attribute_setter_preserves_source_refusal() {
    let (mut session, top) = setup();
    let snapshot = session.snapshot();
    let body = snapshot
        .node(
            snapshot
                .document(top.document())
                .unwrap()
                .document_element()
                .unwrap(),
        )
        .unwrap()
        .children()[1];
    let mut batch = DomBatch::new();
    let inert = batch.create_html_document(top.document(), None);
    let frame = batch.create_element_ns(&inert, Namespace::html(), "frame");
    let frame = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&frame)
        .unwrap()
        .unwrap();
    let (mut host, _) = attach_event_window_with_source_limits(
        session,
        top,
        Arc::new(cancel()),
        160_000,
        DomSourceLimits::new(16_000_000, 200_000, 0, 64, 64, 64, 2),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    assert_eq!(
        observe(
            &mut host,
            &permit,
            "typeof Object.getOwnPropertyDescriptor(HTMLFrameElement.prototype, 'marginWidth')?.set === 'function' ? 'ok' : 'missing HTMLFrameElement.marginWidth setter'"
        ),
        "ok"
    );
    assert_original_frame_source_refusal(
        &mut host,
        &permit,
        "Object.getOwnPropertyDescriptor(HTMLFrameElement.prototype, 'marginWidth').set.call(document.getElementsByTagName('frame')[0], 'unpublished')",
        DomError::SourceWorkLimit { limit: 0 },
    );
    host.stop().into_session().unwrap();
}

fn assert_original_frame_source_refusal(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    input: &str,
    expected: DomError,
) {
    let before = host.snapshot().unwrap().stamp();
    let result = host.in_document(permit, |scope| {
        scope
            .evaluate(&input.encode_utf16().collect::<Vec<_>>())
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected typed native Frame refusal, got {result:?}");
    };
    let Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(actual))) =
        failure.cause::<DomSourceDriveError>()
    else {
        panic!(
            "wrong native Frame refusal: {:?}",
            std::error::Error::source(&failure)
        );
    };
    assert_eq!(actual, &expected);
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}

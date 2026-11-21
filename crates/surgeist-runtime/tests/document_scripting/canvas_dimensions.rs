//! Attached Canvas dimensions reflect canonical attributes in the native none mode.

use super::*;
use surgeist_html::elements::media::HtmlCanvasBitmapDimension;

pub(super) fn attached_canvas_dimensions_follow_native_bitmap_and_reflection() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLCanvasElement.prototype;
        const fields = ['width', 'height'];
        const descriptors = [];
        for (const name of fields) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.get !== 'function') return 'missing HTMLCanvasElement.prototype.' + name + ' getter';
            if (d.get.name !== 'get ' + name || d.get.length !== 0 ||
                typeof d.set !== 'function' || d.set.name !== 'set ' + name || d.set.length !== 1 ||
                !d.enumerable || !d.configurable) return 'Canvas ' + name + ' accessor descriptor';
            descriptors.push(d);
        }
        if (Object.getOwnPropertyNames(prototype).filter(key => fields.includes(key)).join() !== fields.join()) {
            return 'Canvas dimension declaration order';
        }

        const html = 'http://www.w3.org/1999/xhtml';
        const large = document.body.appendChild(document.createElementNS(html, 'p:canvas'));
        const calls = [];
        class WatchedCanvas extends HTMLCanvasElement {
            static observedAttributes = ['width', 'height'];
            attributeChangedCallback(name, oldValue, newValue, namespace) {
                calls.push([this, name, oldValue, newValue, namespace, this.width, this.height]);
                if (namespace === null && name === 'width' && newValue === '13') this.height = 7;
            }
        }
        customElements.define('x-canvas-dimension-probe', WatchedCanvas, {extends: 'canvas'});
        const watched = document.body.appendChild(new WatchedCanvas());
        if (Object.getPrototypeOf(large) !== prototype ||
            Object.getPrototypeOf(watched) !== WatchedCanvas.prototype || !(watched instanceof HTMLCanvasElement)) {
            return 'authentic prefixed and customized Canvas wrappers';
        }
        let coercions = 0;
        const poison = { [Symbol.toPrimitive]() { coercions++; return 9; } };
        const foreign = document.createElementNS('urn:foreign', 'canvas');
        Object.setPrototypeOf(foreign, prototype);
        const wrong = [foreign, document.createElementNS(html, 'CANVAS'), document.createElement('div'),
            Object.create(prototype)];
        for (const descriptor of descriptors) {
            for (const callback of [descriptor.get, descriptor.set]) {
                for (const receiver of wrong) {
                    try {
                        callback.call(receiver, poison);
                        return 'Canvas accessor accepted a wrong receiver';
                    } catch (error) {
                        if (!(error instanceof TypeError) || coercions !== 0) return 'Canvas brand before coercion';
                    }
                }
            }
        }

        watched.setAttributeNS('urn:foreign', 'width', '77');
        watched.setAttributeNS('urn:foreign', 'p:height', '99');
        watched.setAttributeNS(null, 'WIDTH', '17');
        if (watched.width !== 300 || watched.height !== 150 ||
            watched.getAttributeNS(null, 'width') !== null || watched.getAttributeNS(null, 'height') !== null) {
            return 'absent Canvas dimensions and exact attribute names';
        }
        large.setAttributeNS(null, 'width', ' 000340282366920938463463374607431768211456tail');
        large.setAttributeNS(null, 'height', '2147483648');
        if (large.width !== 300 || large.height !== 150) {
            return 'Canvas IDL defaults beyond its signed-long maximum';
        }
        globalThis.canvasDimensionProbe = {large, watched, calls};
        return 'ok';
    })()"#,
    );

    const LARGE: &str = "340282366920938463463374607431768211456";
    const LARGE_RAW: &str = " 000340282366920938463463374607431768211456tail";
    let initial = host.snapshot().unwrap();
    let html = initial
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = initial.node(html).unwrap().children()[1];
    let [large, watched]: [NodeId; 2] = initial.node(body).unwrap().children().try_into().unwrap();
    assert_bitmap(&initial, watched, ["300", "150"]);
    assert_bitmap(&initial, large, [LARGE, "2147483648"]);
    assert_eq!(attribute(&initial, large, "width"), Some(&LARGE_RAW.into()));
    assert_eq!(
        attribute(&initial, large, "height"),
        Some(&"2147483648".into())
    );
    assert_eq!(attribute(&initial, watched, "width"), None);
    assert_eq!(attribute(&initial, watched, "height"), None);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {watched} = canvasDimensionProbe;
        watched.setAttributeNS(null, 'width', ' \t+00037px');
        watched.setAttributeNS(null, 'height', '-3');
        if (watched.width !== 37 || watched.height !== 150) return 'Canvas prefix and invalid recovery';
        watched.setAttributeNS(null, 'height', '0tail');
        if (watched.height !== 0) return 'zero Canvas dimension';
        watched.setAttributeNS(null, 'height', '2147483647tail');
        return watched.width === 37 && watched.height === 2147483647 ? 'ok' : 'maximum Canvas IDL dimension';
    })()"#,
    );

    let recovered = host.snapshot().unwrap();
    assert_bitmap(&recovered, watched, ["37", "2147483647"]);
    assert_eq!(
        attribute(&recovered, watched, "width"),
        Some(&" \t+00037px".into())
    );
    assert_eq!(
        attribute(&recovered, watched, "height"),
        Some(&"2147483647tail".into())
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {large, watched, calls} = canvasDimensionProbe;
        for (const [member, operand, raw, result] of [
            ['width', 2147483647, '2147483647', 2147483647], ['height', 2147483648, '150', 150],
            ['width', -1, '300', 300], ['height', 4294967301.75, '5', 5],
            ['width', 0, '0', 0], ['height', 1.9, '1', 1],
        ]) {
            const other = member === 'width' ? 'height' : 'width';
            const previous = watched.getAttributeNS(null, other);
            watched[member] = operand;
            if (watched[member] !== result || watched.getAttributeNS(null, member) !== raw ||
                watched.getAttributeNS(null, other) !== previous) return member + ' Canvas unsigned-long write';
        }

        calls.length = 0;
        const {set} = Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype, 'width');
        const thrown = {};
        try {
            set.call(watched, {valueOf() { throw thrown; }});
            return 'Canvas abrupt conversion returned';
        } catch (error) {
            if (error !== thrown || watched.getAttributeNS(null, 'width') !== '0' || calls.length !== 0) {
                return 'Canvas abrupt conversion changed the original element';
            }
        }

        let receiver = watched;
        const hints = [];
        const result = set.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = large;
            watched.width = 13;
            return 4294967305;
        } });
        const expected = [
            [watched, 'width', '0', '13', null, 13, 1],
            [watched, 'height', '1', '7', null, 13, 7],
            [watched, 'width', '13', '9', null, 9, 7],
        ];
        if (result !== undefined || hints.join() !== 'number' || receiver !== large ||
            calls.length !== expected.length || calls.some((row, i) =>
                expected[i].some((value, j) => value !== row[j]))) {
            return 'Canvas keeps the original receiver and synchronous nested Source reactions';
        }
        return large.width === 300 && large.height === 150 && watched.width === 9 && watched.height === 7
            ? 'ok' : 'independent Canvas dimensions after coercion';
    })()"#,
    );

    let published = host.snapshot().unwrap();
    assert_bitmap(&published, watched, ["9", "7"]);
    assert_bitmap(&published, large, [LARGE, "2147483648"]);
    assert_eq!(
        attribute(&published, large, "width"),
        Some(&LARGE_RAW.into())
    );
    assert_eq!(
        attribute(&published, large, "height"),
        Some(&"2147483648".into())
    );
    assert_eq!(attribute(&published, watched, "width"), Some(&"9".into()));
    assert_eq!(attribute(&published, watched, "height"), Some(&"7".into()));
    assert_eq!(attribute(&published, watched, "WIDTH"), Some(&"17".into()));
    let node = published.node(watched).unwrap();
    let foreign = Namespace::from_uri("urn:foreign");
    assert_eq!(
        node.attribute_ns(&foreign, &"width".into()),
        Some(&"77".into())
    );
    assert_eq!(
        node.attribute_ns(&foreign, &"height".into()),
        Some(&"99".into())
    );
    assert_eq!(published.incomplete_source_operation(), None);
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn assert_bitmap(snapshot: &DomSnapshot, id: NodeId, expected: [&str; 2]) {
    let state = snapshot
        .node(id)
        .unwrap()
        .as_element()
        .unwrap()
        .canvas_none_state()
        .unwrap();
    assert!(state.context_is_none());
    assert!(!state.has_bound_context());
    assert!(state.origin_clean());
    assert!(state.bitmap_is_transparent_black());
    for (dimension, expected) in [state.width(), state.height()].into_iter().zip(expected) {
        match dimension {
            HtmlCanvasBitmapDimension::Default(value) => assert_eq!(value.to_string(), expected),
            HtmlCanvasBitmapDimension::Parsed(value) => {
                assert_eq!(value.decimal_digits(), expected)
            }
        }
    }
}

fn attribute<'a>(snapshot: &'a DomSnapshot, id: NodeId, name: &str) -> Option<&'a DocumentString> {
    snapshot
        .node(id)
        .unwrap()
        .attribute_ns(&Namespace::none(), &name.into())
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

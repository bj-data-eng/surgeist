//! Attached Progress and Meter getters recover live numeric state from raw attributes.

use super::*;

pub(super) fn attached_range_indicators_reflect_native_numbers() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const families = [
            [HTMLProgressElement, 'progress', ['value', 'max', 'position']],
            [HTMLMeterElement, 'meter', ['value', 'min', 'max', 'low', 'high', 'optimum']],
        ];
        for (const [constructor, , order] of families) {
            const prototype = constructor.prototype;
            for (const name of order) {
                const d = Object.getOwnPropertyDescriptor(prototype, name);
                if (typeof d?.get !== 'function') return 'missing ' + constructor.name + '.prototype.' + name + ' getter';
                const setter = name === 'position' ? d.set === undefined :
                    typeof d.set === 'function' && d.set.name === 'set ' + name && d.set.length === 1;
                if (d.get.name !== 'get ' + name || d.get.length !== 0 || !d.enumerable ||
                    !d.configurable || !setter) return constructor.name + '.' + name + ' descriptor';
            }
            if (Object.getOwnPropertyNames(prototype).filter(key => order.includes(key)).join() !== order.join()) {
                return constructor.name + ' numeric declaration order';
            }
        }

        const html = 'http://www.w3.org/1999/xhtml';
        const calls = [];
        class WatchedProgress extends HTMLProgressElement {
            static observedAttributes = ['max', 'value'];
            attributeChangedCallback(name, oldValue, newValue, namespace) {
                calls.push([this, name, oldValue, newValue, namespace, this.value, this.max, this.position]);
            }
        }
        customElements.define('x-range-indicator-progress', WatchedProgress, {extends: 'progress'});
        const progress = document.body.appendChild(new WatchedProgress());
        const otherProgress = document.body.appendChild(document.createElementNS(html, 'p:progress'));
        const meter = document.body.appendChild(document.createElementNS(html, 'p:meter'));
        if (Object.getPrototypeOf(progress) !== WatchedProgress.prototype ||
            Object.getPrototypeOf(otherProgress) !== HTMLProgressElement.prototype ||
            Object.getPrototypeOf(meter) !== HTMLMeterElement.prototype) return 'authentic range indicator prototypes';

        let conversions = 0;
        const operand = { [Symbol.toPrimitive]() { conversions++; return 2; } };
        for (const [i, [constructor, local]] of families.entries()) {
            const p = constructor.prototype;
            const foreign = document.createElementNS('urn:foreign', local);
            Object.setPrototypeOf(foreign, p);
            const wrong = [i === 0 ? meter : otherProgress, foreign, Object.create(p),
                document.createElementNS(html, local.toUpperCase())];
            const descriptor = Object.getOwnPropertyDescriptor(p, 'value');
            const accessors = [descriptor.get, descriptor.set];
            if (i === 0) accessors.push(Object.getOwnPropertyDescriptor(p, 'position').get);
            for (const callback of accessors) {
                for (const receiver of wrong) {
                    try {
                        callback.call(receiver, operand);
                        return constructor.name + ' accepted wrong receiver';
                    } catch (error) {
                        if (!(error instanceof TypeError) || conversions !== 0) return constructor.name + ' brand before coercion';
                    }
                }
            }
        }

        progress.setAttributeNS('urn:foreign', 'p:value', '2');
        progress.setAttributeNS(null, 'MAX', '100');
        if (progress.value !== 0 || progress.max !== 1 || progress.position !== -1) {
            return 'foreign value and uppercase max must not affect Progress';
        }
        progress.setAttributeNS(null, 'max', ' 4tail');
        if (progress.value !== 0 || progress.max !== 4 || progress.position !== -1) {
            return 'indeterminate Progress with authored maximum';
        }
        calls.length = 0;
        for (const value of [0, -0, -2]) progress.max = value;
        if (progress.getAttributeNS(null, 'max') !== ' 4tail' || progress.max !== 4 || calls.length !== 0) {
            return 'nonpositive Progress max neither writes nor reacts';
        }
        globalThis.rangeIndicatorProbe = {progress, otherProgress, meter, calls};
        return 'ok';
    })()"#,
    );

    let initial = host.snapshot().unwrap();
    let html = initial
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = initial.node(html).unwrap().children()[1];
    let [progress, other_progress, meter]: [NodeId; 3] =
        initial.node(body).unwrap().children().try_into().unwrap();
    assert_eq!(attribute(&initial, progress, "max"), Some(&" 4tail".into()));
    assert_eq!(attribute(&initial, progress, "value"), None);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {progress, otherProgress, meter, calls} = rangeIndicatorProbe;
        progress.setAttributeNS(null, 'max', '0');
        progress.setAttributeNS(null, 'value', 'invalid');
        if (progress.max !== 1 || progress.value !== 0 || progress.position !== 0) {
            return 'present malformed value is determinate and nonpositive raw max falls back';
        }
        progress.max = 4;
        progress.value = -2;
        if (progress.value !== 0 || progress.position !== 0 || progress.getAttributeNS(null, 'value') !== '-2') {
            return 'negative Progress value is stored and read as zero';
        }
        otherProgress.value = -0;
        if (otherProgress.getAttributeNS(null, 'value') !== '0' || !Object.is(otherProgress.value, 0)) {
            return 'finite negative zero serializes to zero';
        }
        otherProgress.value = 1e-7;
        if (otherProgress.value !== 1e-7 || otherProgress.getAttributeNS(null, 'value') !== '1e-7') {
            return 'finite exponent serialization';
        }

        const meterValues = () => [meter.value, meter.min, meter.max, meter.low, meter.high, meter.optimum];
        const equal = (actual, expected) => actual.every((value, i) => Object.is(value, expected[i]));
        if (!equal(meterValues(), [0, 0, 1, 0, 1, 0.5])) return 'Meter defaults';
        meter.min = -2;
        meter.max = 6;
        meter.value = 9;
        meter.low = 5;
        meter.high = 3;
        if (!equal(meterValues(), [6, -2, 6, 5, 5, 2]) ||
            meter.getAttributeNS(null, 'value') !== '9' || meter.getAttributeNS(null, 'high') !== '3') {
            return 'Meter effective range, high depends on low, and default midpoint';
        }
        meter.optimum = 10;
        if (meter.optimum !== 6 || meter.getAttributeNS(null, 'optimum') !== '10') return 'Meter optimum clamp';
        meter.max = -7;
        if (!equal(meterValues(), [-2, -2, -2, -2, -2, -2]) ||
            meter.getAttributeNS(null, 'max') !== '-7') return 'Meter effective range collapses to min';

        calls.length = 0;
        for (const [node, member, raw] of [[progress, 'max', '4'], [progress, 'value', '-2'], [meter, 'value', '9']]) {
            for (const invalid of [NaN, Infinity, -Infinity, undefined]) {
                try {
                    node[member] = invalid;
                    return member + ' accepted nonfinite double';
                } catch (error) {
                    if (!(error instanceof TypeError) || node.getAttributeNS(null, member) !== raw) {
                        return member + ' nonfinite conversion changed native content';
                    }
                }
            }
        }
        const {set} = Object.getOwnPropertyDescriptor(HTMLProgressElement.prototype, 'value');
        const thrown = {};
        try {
            set.call(progress, {valueOf() { throw thrown; }});
            return 'abrupt conversion returned';
        } catch (error) {
            if (error !== thrown || progress.getAttributeNS(null, 'value') !== '-2' || calls.length !== 0) {
                return 'abrupt conversion lost value or wrote an attribute';
            }
        }
        const hints = [];
        let receiver = progress;
        const result = set.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = otherProgress;
            progress.value = 2;
            return 9;
        } });
        const expected = [
            [progress, 'value', '-2', '2', null, 2, 4, 0.5],
            [progress, 'value', '2', '9', null, 4, 4, 1],
        ];
        if (result !== undefined || hints.join() !== 'number' || receiver !== otherProgress ||
            calls.length !== expected.length || calls.some((row, i) =>
                expected[i].some((value, j) => value !== row[j]))) {
            return 'reentrant Progress write and live Source reaction readback';
        }
        return progress.getAttributeNS(null, 'value') === '9' && progress.value === 4 &&
            otherProgress.getAttributeNS(null, 'value') === '1e-7' ? 'ok' : 'preserve the original Progress receiver';
    })()"#,
    );

    let published = host.snapshot().unwrap();
    assert_eq!(attribute(&published, progress, "value"), Some(&"9".into()));
    assert_eq!(attribute(&published, progress, "max"), Some(&"4".into()));
    assert_eq!(attribute(&published, progress, "MAX"), Some(&"100".into()));
    assert_eq!(
        published
            .node(progress)
            .unwrap()
            .attribute_ns(&Namespace::from_uri("urn:foreign"), &"value".into()),
        Some(&"2".into())
    );
    assert_eq!(
        attribute(&published, other_progress, "value"),
        Some(&"1e-7".into())
    );
    assert_eq!(attribute(&published, other_progress, "max"), None);
    for (name, raw) in [
        ("value", "9"),
        ("min", "-2"),
        ("max", "-7"),
        ("low", "5"),
        ("high", "3"),
        ("optimum", "10"),
    ] {
        assert_eq!(
            attribute(&published, meter, name),
            Some(&DocumentString::from(raw)),
            "raw Meter {name}"
        );
    }
    assert_eq!(published.incomplete_source_operation(), None);
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
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

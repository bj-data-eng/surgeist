//! The standard Option factory publishes a real detached Option and native state.

use super::*;

fn option_session() -> (DomSession, DomHostedTopLevel) {
    let mut dom = native_dom();
    dom.set_limits(
        dom.limits()
            .with_select_processing(32, 1_000_000, 1_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut session = DomSession::new(dom, limits()).unwrap();
    let (_, top) = create(&mut session);
    (session, top)
}

pub(super) fn attached_option_factory_creates_clean_native_selectedness() {
    let (session, top) = option_session();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let original = host.snapshot().unwrap();
    let root = original
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = original.node(root).unwrap().children()[1];
    let result = host
        .in_document(&permit, |scope| {
            let source = r#"(() => {
                const global = Object.getOwnPropertyDescriptor(window, 'Option');
                if (typeof global?.value !== 'function') return 'missing native Window.Option';
                const prototype = Object.getOwnPropertyDescriptor(Option, 'prototype');
                if (!global.writable || global.enumerable || !global.configurable ||
                    Option.name !== 'Option' || Option.length !== 0 || Option === HTMLOptionElement ||
                    prototype.value !== HTMLOptionElement.prototype ||
                    prototype.writable || prototype.enumerable || prototype.configurable ||
                    prototype.value.constructor !== HTMLOptionElement) return 'legacy factory descriptors';

                let calls = 0;
                try {
                    Option({toString() { calls++; return 'ignored'; }});
                    return 'ordinary call unexpectedly constructed an Option';
                } catch (error) {
                    if (!(error instanceof TypeError) || calls !== 0) return 'NewTarget check before conversion';
                }
                const empty = new Option();
                const absentValue = new Option('label', undefined);
                const emptyValue = new Option(undefined, '');
                if (!(empty instanceof Option) || !(empty instanceof HTMLOptionElement) ||
                    empty.ownerDocument !== document || empty.parentNode !== null ||
                    empty.firstChild !== null || empty.hasAttribute('value') || empty.selected ||
                    absentValue.hasAttribute('value') || absentValue.value !== 'label' ||
                    emptyValue.getAttribute('value') !== '' || emptyValue.firstChild !== null ||
                    new Option('', null).getAttribute('value') !== 'null') return 'native default and omission rules';

                const baseline = new Option('label', undefined, true);
                if (!baseline.defaultSelected || baseline.selected) return 'separate selectedness and attribute';
                baseline.defaultSelected = false;
                baseline.defaultSelected = true;
                if (!baseline.selected) return 'factory selectedness must start clean';
                const dirty = new Option('', undefined, true, false);
                dirty.selected = false;
                dirty.defaultSelected = false;
                dirty.defaultSelected = true;
                if (dirty.selected) return 'the later selected setter marks the Option dirty';
                function Fallback() {}
                Fallback.prototype = null;
                if (Object.getPrototypeOf(Reflect.construct(Option, [], Fallback)) !== prototype.value) {
                    return 'realm-local Option prototype fallback';
                }

                const order = [];
                const raw = 'A\ud800\0\udc00';
                const input = step => ({toString() { order.push(step); return raw; }});
                const customPrototype = Object.create(HTMLOptionElement.prototype);
                const target = new Proxy(function () {}, {get(target, key, receiver) {
                    if (key === 'prototype') { order.push('prototype'); return customPrototype; }
                    return Reflect.get(target, key, receiver);
                }});
                const native = Reflect.construct(Option, [
                    input('text'), input('value'), false, true, input('unreached'),
                ], target);
                if (order.join() !== 'text,value,prototype' || Object.getPrototypeOf(native) !== customPrototype ||
                    native.ownerDocument !== document || native.parentNode !== null ||
                    native.firstChild.data !== raw || native.value !== raw ||
                    native.defaultSelected || !native.selected) return 'exact inputs, reentry and creation realm';

                const thrown = {};
                const comment = document.createComment('retained conversion effect');
                order.length = 0;
                try {
                    Reflect.construct(Option, [input('text'), {toString() {
                        document.body.appendChild(comment);
                        throw thrown;
                    }}], target);
                    return 'conversion throw was lost';
                } catch (error) {
                    if (error !== thrown || order.join() !== 'text' || comment.parentNode !== document.body) {
                        return 'original conversion error precedes NewTarget access and retains accepted effects';
                    }
                }
                globalThis.savedOption = native;
                if (document.body.appendChild(native) !== native) return 'ordinary tree uses the same Option';
                return 'ok';
            })()"#;
            let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap_or_else(|error| {
            if let DocumentScriptError::Script(Error::HostFailure(failure)) = &error {
                panic!(
                    "Option factory native failure: {:?}",
                    failure.cause::<DocumentGuestEventError>()
                );
            }
            panic!("Option factory guest execution: {error:?}");
        });
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");

    let snapshot = host.snapshot().unwrap();
    let id = *snapshot.node(body).unwrap().children().last().unwrap();
    let option = snapshot.node(id).unwrap();
    let child = snapshot.node(option.children()[0]).unwrap();
    assert_eq!(child.data().unwrap().code_units(), &[65, 0xd800, 0, 0xdc00]);
    assert_eq!(
        option
            .attribute_ns(&Namespace::none(), &"value".into())
            .unwrap()
            .code_units(),
        child.data().unwrap().code_units(),
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(id);
        batch.destroy(id);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let retired = host.in_document(&permit, |scope| {
        scope
            .evaluate(&"savedOption.value".encode_utf16().collect::<Vec<_>>())
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = retired else {
        panic!("retired Option must refuse through its original custody");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(super) fn attached_option_factory_uses_its_original_source_limits() {
    for (bytes, work, expected) in [
        (16_000_000, 0, DomError::SourceWorkLimit { limit: 0 }),
        (
            1,
            20_000,
            DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: 1,
            },
        ),
    ] {
        let (session, top) = option_session();
        let (mut host, _) = attach_event_window_with_source_limits(
            session,
            top,
            Arc::new(cancel()),
            160_000,
            DomSourceLimits::new(bytes, 200_000, work, 64, 64, 64, 2),
            |_, _, _| {},
        );
        let permit = host.permit().unwrap();
        let before = host.snapshot().unwrap();
        let script = "new Option('text', 'value', true, false)";
        let result = host.in_document(&permit, |scope| {
            scope
                .evaluate(&script.encode_utf16().collect::<Vec<_>>())
                .map(drop)
        });
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("attached Option creation must refuse its Source limit: {result:?}");
        };
        assert!(
            matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::Document(DocumentScriptError::Native(
                    DocumentScriptNativeError::Dom(error)
                ))) if *error == expected
            ),
            "wrong Option refusal: {:?}",
            failure.cause::<DocumentGuestEventError>()
        );
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), before.stamp());
        assert_eq!(after.nodes().count(), before.nodes().count());
    }
}

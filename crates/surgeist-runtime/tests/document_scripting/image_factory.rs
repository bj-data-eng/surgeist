//! The native Window Image factory and its original Source admission.

use super::*;

pub(super) fn attached_image_factory_creates_and_qualifies_native_nodes() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let result = host
        .in_document(&permit, |scope| {
            let source = r#"(() => {
              try {
                const global = Object.getOwnPropertyDescriptor(window, 'Image');
                if (typeof global?.value !== 'function') return 'missing native Window.Image';
                const prototype = Object.getOwnPropertyDescriptor(Image, 'prototype');
                if (!global.writable || global.enumerable || !global.configurable ||
                    Image.name !== 'Image' || Image.length !== 0 || Image === HTMLImageElement ||
                    prototype.value !== HTMLImageElement.prototype ||
                    prototype.writable || prototype.enumerable || prototype.configurable ||
                    prototype.value.constructor !== HTMLImageElement) return 'Image factory descriptors';

                let used = false;
                try {
                    Image({valueOf() { used = true; return 30; }});
                    return 'ordinary call constructed an Image';
                } catch (error) {
                    if (!(error instanceof TypeError) || used) return 'ordinary call converted width';
                }

                const fresh = new Image();
                if (!(fresh instanceof Image) || !(fresh instanceof HTMLImageElement) ||
                    fresh.ownerDocument !== document || fresh.parentNode !== null ||
                    fresh.firstChild !== null || fresh.hasAttribute('width') ||
                    fresh.hasAttribute('height') || fresh.hasAttribute('src')) return 'ordinary empty Image';
                const omitted = new Image(undefined, 6);
                const coerced = new Image(null, NaN);
                const wrapped = new Image(-1, 4294967298);
                if (omitted.hasAttribute('width') || omitted.getAttribute('height') !== '6' ||
                    coerced.getAttribute('width') !== '0' || coerced.getAttribute('height') !== '0' ||
                    wrapped.getAttribute('width') !== '4294967295' ||
                    wrapped.getAttribute('height') !== '2') return 'numeric omission and unsigned-long conversion';

                function Fallback() {}
                Fallback.prototype = null;
                if (Object.getPrototypeOf(Reflect.construct(Image, [], Fallback)) !== prototype.value) {
                    return 'realm-local Image prototype fallback';
                }

                const order = [];
                const dimension = (name, value) => ({[Symbol.toPrimitive](hint) {
                    if (hint !== 'number') throw Error('wrong numeric hint');
                    order.push(name);
                    return value;
                }});
                const customPrototype = Object.create(HTMLImageElement.prototype);
                const newTarget = new Proxy(function () {}, { get(target, key, receiver) {
                    if (key === 'prototype') { order.push('prototype'); return customPrototype; }
                    return Reflect.get(target, key, receiver);
                }});
                const native = Reflect.construct(Image, [
                    dimension('width', 314159.9), dimension('height', 271828), dimension('unreached', 3),
                ], newTarget);
                if (order.join() !== 'width,height,prototype' ||
                    Object.getPrototypeOf(native) !== customPrototype || native.ownerDocument !== document ||
                    native.parentNode !== null || native.getAttribute('width') !== '314159' ||
                    native.getAttribute('height') !== '271828') return 'exact ordering and native factory identity';

                const sentinel = {};
                const comment = document.createComment('accepted effect');
                order.length = 0;
                try {
                    Reflect.construct(Image, [dimension('width', 1), {valueOf() {
                        document.body.appendChild(comment);
                        throw sentinel;
                    }}], newTarget);
                    return 'conversion throw lost';
                } catch (error) {
                    if (error !== sentinel || order.join() !== 'width' ||
                        comment.parentNode !== document.body) return 'original throw and accepted effect';
                }
                globalThis.savedImage = native;
                return 'ok';
              } catch (error) {
                return "unexpected guest exception: " + (error.stack ?? String(error));
              }
            })()"#;
            let result = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&result)
        })
        .unwrap_or_else(|error| {
            if let DocumentScriptError::Script(Error::HostFailure(failure)) = &error {
                panic!("Image native failure: {:?}", std::error::Error::source(failure));
            }
            panic!("Image guest execution: {error:?}");
        });
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");

    let snapshot = host.snapshot().unwrap();
    let image = snapshot
        .nodes()
        .filter_map(|id| snapshot.node(id).ok())
        .find(|node| {
            node.as_element().is_some_and(|element| {
                element.name().namespace_uri() == Namespace::html().uri()
                    && element.name().local_name() == &"img".into()
            }) && node.attribute_ns(&Namespace::none(), &"width".into()) == Some(&"314159".into())
        })
        .unwrap();
    assert_eq!(image.owner_document(), Some(top.document()));
    assert_eq!(image.parent(), None);
    assert_eq!(
        image.attribute_ns(&Namespace::none(), &"height".into()),
        Some(&"271828".into())
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(image.id());
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let retired = host.in_document(&permit, |scope| {
        scope
            .evaluate(
                &"savedImage.getAttribute('width')"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = retired else {
        panic!("retired Image must refuse through its original custody: {retired:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));

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
        let (session, top) = setup();
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
        let result = host.in_document(&permit, |scope| {
            scope
                .evaluate(&"new Image(1, 2)".encode_utf16().collect::<Vec<_>>())
                .map(drop)
        });
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("attached Image creation must refuse its original Source limit: {result:?}");
        };
        assert!(
            matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::Document(DocumentScriptError::Native(
                    DocumentScriptNativeError::Dom(error)
                ))) if *error == expected
            ),
            "wrong Image refusal: {:?}",
            std::error::Error::source(&failure)
        );
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), before.stamp());
        assert_eq!(after.nodes().count(), before.nodes().count());
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
    }
}

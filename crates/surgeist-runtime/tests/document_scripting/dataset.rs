use super::{window_length::observe, *};

pub(super) fn dataset_observes_and_mutates_canonical_attributes() {
    let (session, top) = setup();
    let (mut host, _) = attach_event_window_with_source_limits(
        session,
        top,
        Arc::new(cancel()),
        300_000,
        DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
          try {
            const detached = document.implementation.createHTMLDocument();
            const element = detached.createElement('div');
            const svg = detached.createElementNS('http://www.w3.org/2000/svg', 'svg');
            const math = detached.createElementNS('http://www.w3.org/1998/Math/MathML', 'math');
            const map = element.dataset;
            if (typeof DOMStringMap !== 'function' || !(map instanceof DOMStringMap))
                return 'missing live DOMStringMap';
            if (map !== element.dataset || map === svg.dataset || svg.dataset === math.dataset ||
                !(svg.dataset instanceof DOMStringMap) || !(math.dataset instanceof DOMStringMap) ||
                detached.createElementNS('urn:other', 'div').dataset !== undefined)
                return 'dataset exposed on the wrong native interface or with unstable identity';
            element.setAttributeNS(null, 'data-z-z', 'html');
            element.setAttributeNS(null, 'data-', 'empty suffix');
            element.setAttributeNS(null, 'data-Caps', 'ignore uppercase');
            element.setAttributeNS('urn:foreign', 'p:data-same', 'first');
            element.setAttributeNS(null, 'data-same', 'later');
            element.setAttributeNS(null, 'data-2', 'named numeric');
            element.setAttributeNS(null, 'data-to-string', 'override');
            const expectedKeys = ['zZ', '', 'same', '2', 'toString'];
            const descriptor = Object.getOwnPropertyDescriptor(map, 'same');
            if (JSON.stringify(Object.keys(map)) !== JSON.stringify(expectedKeys) ||
                map.zZ !== 'html' || map[''] !== 'empty suffix' || Object.hasOwn(map, 'Caps') ||
                map.same !== 'first' || map.toString !== 'override' || map[2] !== 'named numeric' ||
                !descriptor?.configurable || !descriptor.enumerable || !descriptor.writable)
                return 'wrong native pair eligibility, order, value or descriptor';
            map.same = 'updated null namespace';
            if (map.same !== 'first' || element.getAttributeNS(null, 'data-same') !== 'updated null namespace' ||
                element.getAttributeNS('urn:foreign', 'data-same') !== 'first')
                return 'named setter must target the exact null-namespace local name';
            if (!Reflect.deleteProperty(map, 'same') ||
                element.getAttributeNS(null, 'data-same') !== null || map.same !== 'first')
                return 'deletion did not use the qualified unprefixed name';
            element.removeAttributeNS('urn:foreign', 'data-same');
            if (Object.hasOwn(map, 'same') || !Reflect.deleteProperty(map, 'same'))
                return 'retained map missed native removal';
            map['a:b'] = 'colon';
            const colon = element.getAttributeNodeNS(null, 'data-a:b');
            if (!colon || colon.prefix !== null || colon.localName !== 'data-a:b' ||
                colon.value !== 'colon' || map['a:b'] !== 'colon')
                return 'colon in a valid null-namespace local name was split';
            delete map['a:b'];
            if (element.hasAttributeNS(null, 'data-a:b'))
                return 'deletion lost exact colon spelling';
            element.setAttributeNS(null, 'data-foo--bar', 'double dash');
            if (map['foo-Bar'] !== 'double dash') return 'run of dashes folded incorrectly';
            const originalPrototype = Object.getPrototypeOf(map);
            map.__proto__ = 'named own value';
            if (!Object.hasOwn(map, '__proto__') || map.__proto__ !== 'named own value' ||
                Object.getPrototypeOf(map) !== originalPrototype ||
                element.getAttributeNS(null, 'data-__proto__') !== 'named own value')
                return 'named setter was displaced by a prototype accessor';
            delete map.__proto__;
            const receiver = Object.create(map);
            if (!Reflect.set(map, 'zZ', 'receiver', receiver) || receiver.zZ !== 'receiver' ||
                map.zZ !== 'html' || element.getAttributeNS(null, 'data-z-z') !== 'html')
                return 'foreign receiver reached the native named setter';
            if (!Reflect.defineProperty(map, 'fixed', {value: 12}) ||
                map.fixed !== '12' || element.getAttributeNS(null, 'data-fixed') !== '12' ||
                Reflect.defineProperty(map, 'rejected', {get: () => 1}) ||
                element.hasAttributeNS(null, 'data-rejected') || Reflect.preventExtensions(map))
                return 'named definition or extensibility did not follow Web IDL';
            const symbol = Symbol('ordinary');
            map[symbol] = 7;
            if (map[symbol] !== 7 || Reflect.ownKeys(map).at(-1) !== symbol ||
                !Reflect.deleteProperty(map, symbol) || map[symbol] !== undefined)
                return 'ordinary Symbol operations escaped the named-only distinction';
            const sentinel = {};
            let conversions = 0;
            try {
                map['invalid-name'] = {toString() { conversions++; throw sentinel; }};
                return 'expected value conversion exception';
            } catch (error) {
                if (error !== sentinel || conversions !== 1)
                    return 'name validation ran ahead of conversion';
            }
            try {
                map['invalid-name'] = 'cannot publish';
                return 'expected syntax exception';
            } catch (error) {
                if (error.name !== 'SyntaxError' || element.hasAttributeNS(null, 'data-invalid-name'))
                    return 'wrong syntax failure for dash followed by ASCII lowercase';
            }
            return 'ok';
          } catch (error) {
            return 'unexpected dataset exception: ' + String(error) + '\n' + (error.stack ?? '');
          }
        })()"#,
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}

//! Live HTML Input suggestions-source resolution in an attached Window.

use super::*;

pub(super) fn attached_input_list_follows_the_first_exact_id_in_its_tree() {
    let mut dom = native_dom();
    dom.set_limits(
        dom.limits()
            .with_input_processing(2_000_000, 2_000_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut session = DomSession::new(dom, limits()).unwrap();
    let (_, top) = create(&mut session);
    let (mut host, _) = attach_event_window_with_source_limits(
        session,
        top,
        Arc::new(cancel()),
        160_000,
        DomSourceLimits::new(16_000_000, 200_000, 20_000_000, 64, 64, 64, 2),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let result = host
        .in_document(&permit, |scope| {
            let script = r#"(() => {
                const prototype = HTMLInputElement.prototype;
                const descriptor = Object.getOwnPropertyDescriptor(prototype, 'list');
                if (typeof descriptor?.get !== 'function') return 'missing HTMLInputElement.list getter';
                if (descriptor.set !== undefined || !descriptor.enumerable || !descriptor.configurable ||
                    descriptor.get.name !== 'get list' || descriptor.get.length !== 0) return 'list descriptor';
                const fields = Object.keys(prototype);
                if (fields.indexOf('list') <= fields.indexOf('indeterminate') ||
                    fields.indexOf('list') >= fields.indexOf('max')) return 'list declaration order';

                const input = document.createElement('input');
                const html = 'http://www.w3.org/1999/xhtml';
                const wrongInput = document.createElementNS('urn:foreign', 'input');
                Object.setPrototypeOf(wrongInput, prototype);
                for (const wrong of [document, wrongInput, Object.create(prototype), new Proxy(input, {})]) {
                    try {
                        descriptor.get.call(wrong);
                        return 'unbranded receiver accepted';
                    } catch (error) {
                        if (!(error instanceof TypeError)) return 'wrong receiver error';
                    }
                }
                const raw = ' Key-\ud800\0\udc00 ';
                const container = document.body.appendChild(document.createElement('section'));
                container.appendChild(input);
                const first = container.appendChild(document.createElementNS(html, 'h:datalist'));
                const second = container.appendChild(document.createElement('datalist'));
                first.setAttribute('id', raw);
                second.setAttribute('id', raw);
                if (input.list !== null) return 'missing list attribute';
                input.setAttributeNS('urn:foreign', 'q:list', raw);
                input.setAttribute('list', '');
                if (input.list !== null) return 'empty and foreign list attributes';
                input.setAttribute('list', raw.trim());
                if (input.list !== null) return 'literal UTF-16 without trimming';
                input.setAttribute('list', raw);
                if (input.list !== first || input.list !== input.list ||
                    !(input.list instanceof HTMLDataListElement)) return 'canonical first list identity';

                const blocker = document.createElementNS('urn:foreign', 'datalist');
                blocker.setAttribute('id', raw);
                container.insertBefore(blocker, first);
                if (input.list !== null) return 'first exact ID blocks later HTML Datalist';
                blocker.setAttribute('id', raw.toLowerCase());
                if (input.list !== first) return 'ID comparison is case sensitive';
                container.insertBefore(second, first);
                if (input.list !== second) return 'live duplicate-ID order';
                second.removeAttribute('id');
                if (input.list !== first) return 'live candidate attribute removal';

                for (const type of ['hidden', 'password', 'checkbox', 'radio', 'file', 'submit', 'reset', 'button']) {
                    input.type = type;
                    if (input.list !== null) return 'nonapplicable Input type ' + type;
                }
                for (const type of ['text', 'search', 'email', 'number', 'range', 'color', 'unknown']) {
                    input.type = type;
                    if (input.list !== first) return 'applicable Input type ' + type;
                }
                input.type = 'text';
                const shadow = document.body.appendChild(document.createElement('div')).attachShadow({mode: 'open'});
                shadow.appendChild(input);
                if (input.list !== null) return 'no lookup across a shadow boundary';
                shadow.appendChild(first);
                if (input.list !== first) return 'ordinary tree inside ShadowRoot';
                const root = document.createElement('datalist');
                root.setAttribute('id', raw);
                root.appendChild(input);
                if (input.list !== root) return 'detached matching ordinary root is included';
                root.removeChild(input);
                if (input.list !== null) return 'removed Input has no matching node in its tree';
                return 'ok';
            })()"#;
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap_or_else(|error| {
            if let DocumentScriptError::Script(Error::HostFailure(failure)) = &error {
                panic!("Input list native failure: {:?}", std::error::Error::source(failure));
            }
            panic!("Input list guest execution: {error:?}");
        });
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

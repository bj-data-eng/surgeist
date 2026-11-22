use super::{window_length::observe, *};

pub(super) fn html_token_lists_share_the_native_ordered_set_surface() {
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
            const detached = document.implementation.createHTMLDocument();
            const buckets = {};
            const expected = [
                ['script', 'blocking', 'blocking'], ['link', 'blocking', 'blocking'],
                ['style', 'blocking', 'blocking'], ['a', 'relList', 'rel'],
                ['area', 'relList', 'rel'], ['form', 'relList', 'rel'],
                ['link', 'relList', 'rel'], ['link', 'sizes', 'sizes'],
                ['iframe', 'sandbox', 'sandbox'], ['output', 'htmlFor', 'for'],
            ];
            const throwsTypeError = action => {
                try { action(); } catch (error) { return error instanceof TypeError; }
                return false;
            };
            for (const [element, property, attribute] of expected) {
                const node = buckets[element] ??= detached.createElement(element);
                const list = node[property];
                if (!(list instanceof DOMTokenList)) return 'missing ' + element + '.' + property;
                const descriptor = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(node), property);
                if (!descriptor?.enumerable || !descriptor.configurable ||
                    descriptor.get.name !== 'get ' + property || descriptor.get.length !== 0 ||
                    descriptor.set.name !== 'set ' + property || descriptor.set.length !== 1 ||
                    !throwsTypeError(() => descriptor.get.call(detached.createElement('div'))))
                    return 'incorrect descriptor for ' + element + '.' + property;
                node.setAttributeNS('urn:other', 'p:' + attribute, 'untouched');
                node.setAttributeNS(null, attribute, '  one  one two ');
                if (list !== node[property] || list === node.classList ||
                    list.value !== '  one  one two ' || list[1] !== 'two')
                    return 'native edit not reflected for ' + element + '.' + property;
                list.add('three');
                if (node.getAttributeNS(null, attribute) !== 'one two three' ||
                    node.getAttributeNS('urn:other', attribute) !== 'untouched')
                    return 'list mutation missed exact content attribute for ' + element + '.' + property;
                node[property] = { toString() { return ' new\t  new retained '; } };
                if (list.value !== ' new\t  new retained ' || list[1] !== 'retained')
                    return 'value forwarding failed for ' + element + '.' + property;
                node.removeAttributeNS(null, attribute);
                if (list !== node[property] || list.length !== 0)
                    return 'attribute removal lost stable association for ' + element + '.' + property;
            }
            if (buckets.link.blocking === buckets.link.relList || buckets.link.relList === buckets.link.sizes ||
                buckets.link.blocking === buckets.link.sizes)
                return 'Link associations shared a traced slot';
            for (const name of ['script', 'link', 'style']) {
                const {blocking} = buckets[name];
                if (!blocking.supports('RENDER') || blocking.supports('render next') || blocking.supports('none'))
                    return 'blocking vocabulary did not follow HTML';
            }
            for (const name of ['a', 'area', 'form']) {
                if (buckets[name].relList.supports('stylesheet') || buckets[name].relList.supports('custom-token'))
                    return 'noncandidate tokens must be false for ' + name;
            }
            if (buckets.link.relList.supports('noopener') ||
                !throwsTypeError(() => buckets.link.sizes.supports('any')) ||
                !throwsTypeError(() => buckets.output.htmlFor.supports('retained')))
                return 'independent supported-token policies';
            if (!buckets.iframe.sandbox.supports('ALLOW-SCRIPTS') ||
                !buckets.iframe.sandbox.supports('allow-same-origin') ||
                buckets.iframe.sandbox.supports('nonstandard-token'))
                return 'native iframe policy was not qualified';
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}

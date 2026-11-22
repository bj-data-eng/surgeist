use super::{window_length::observe, *};

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let mut session = DomSession::new(native_dom(), limits()).unwrap();
    let (_, top) = create(&mut session);
    let (host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 180_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    (host, permit)
}

pub(super) fn ordinary_element_traversal() {
    let (mut host, permit) = fixture();
    let traversal = observe(
        &mut host,
        &permit,
        r"(() => {
        const fragment = new DocumentFragment();
        const parent = document.createElement('section');
        const old = parent.children;
        if (!(old instanceof HTMLCollection) || !(fragment.children instanceof HTMLCollection)
            || !(document.children instanceof HTMLCollection))
            return 'ParentNode must project live Element children in all native includers';
        if (old !== parent.children || old === fragment.children || old === document.children
            || old.length !== 0 || parent.firstElementChild !== null || parent.lastElementChild !== null
            || parent.childElementCount !== 0)
            return 'an empty child view must retain identity and report empty traversal';
        const make = name => document.createElement(name);
        const first = make('div');
        const last = make('span');
        const middle = document.createTextNode('between');
        first.appendChild(make('em'));
        first.id = 'native-first';
        const head = parent.appendChild(document.createComment('head'));
        parent.appendChild(first);
        parent.appendChild(middle);
        parent.appendChild(last);
        const tail = parent.appendChild(document.createComment('tail'));
        fragment.appendChild(parent);
        if (fragment.children.length !== 1 || fragment.children[0] !== parent
            || fragment.firstElementChild !== parent || fragment.lastElementChild !== parent
            || fragment.childElementCount !== 1 || document.children[0] !== document.documentElement
            || document.firstElementChild !== document.documentElement)
            return 'Fragment and Document getters must expose their canonical direct Elements';
        if (parent.children !== old || old.length !== 2 || old.item(0) !== first || old[1] !== last
            || old.namedItem('native-first') !== first || parent.childElementCount !== old.length
            || parent.firstElementChild !== first || parent.lastElementChild !== last)
            return 'the live collection and scalar getters must agree over direct Element children';
        if (head.previousElementSibling !== null || head.nextElementSibling !== first
            || first.previousElementSibling !== null || first.nextElementSibling !== last
            || middle.previousElementSibling !== first || middle.nextElementSibling !== last
            || last.previousElementSibling !== first || last.nextElementSibling !== null
            || tail.previousElementSibling !== last || tail.nextElementSibling !== null)
            return 'Element and CharacterData siblings must skip non-Elements and stop at native edges';
        for (const proto of [Element.prototype, CharacterData.prototype]) {
            const descriptor = Object.getOwnPropertyDescriptor(proto, 'nextElementSibling');
            if (!descriptor?.get || !descriptor.enumerable || !descriptor.configurable
                || descriptor.set !== undefined || descriptor.get.length !== 0)
                return 'sibling getters must be on the applying interface with standard attributes';
            try { descriptor.get.call(Object.create(first)); return 'forged sibling receiver accepted'; }
            catch (error) { if (!(error instanceof TypeError)) throw error; }
        }
        if (Object.hasOwn(Node.prototype, 'nextElementSibling'))
            return 'ordinary Node lacks the non-DocumentType sibling getter';
        const childrenGetter = Object.getOwnPropertyDescriptor(DocumentFragment.prototype, 'children').get;
        try { childrenGetter.call(parent); return 'fragment getter accepted an Element'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        const shadowHost = make('div');
        const shadow = shadowHost.attachShadow({mode: 'open'});
        const hidden = shadow.appendChild(make('span'));
        if (shadow.children[0] !== hidden || shadow.firstElementChild !== hidden
            || shadowHost.children.length !== 0 || shadowHost.firstElementChild !== null)
            return 'ShadowRoot inherits ordinary Fragment behavior within its own tree';
        parent.insertBefore(last, first);
        last.remove();
        if (last.previousElementSibling !== null || last.nextElementSibling !== null
            || parent.firstElementChild !== first || parent.lastElementChild !== first
            || parent.childElementCount !== 1 || old.length !== 1 || old[0] !== first)
            return 'retained children and siblings must follow mutations';
        return 'ok';
    })()",
    );
    assert_eq!(traversal, "ok");
}

pub(super) fn ordinary_element_descendant_queries() {
    let (mut host, permit) = fixture();
    let queries = observe(
        &mut host,
        &permit,
        r"(() => {
        const root = document.body.appendChild(document.createElement('section'));
        const methodNames = ['getElementsByTagName', 'getElementsByTagNameNS', 'getElementsByClassName'];
        if (methodNames.some(name => typeof Element.prototype[name] !== 'function' || typeof document[name] !== 'function'))
            return 'Element and Document must expose the standard live descendant queries';
        const svgNS = 'http://www.w3.org/2000/svg';
        const html = root.appendChild(document.createElement('div'));
        const svg = root.appendChild(document.createElementNS(svgNS, 'DIV'));
        const prefixed = html.appendChild(document.createElementNS(svgNS, 'p:DIV'));
        const noNS = root.appendChild(document.createElementNS(null, 'none'));
        const uncounted = root.appendChild(document.createElement('template'));
        uncounted.content.appendChild(document.createElement('div'));
        const shield = root.appendChild(document.createElement('div'));
        shield.attachShadow({mode: 'open'}).appendChild(document.createElement('div'));
        html.setAttribute('class', 'Token\tAlpha\nextra');
        svg.setAttribute('class', 'Token\fAlpha');
        prefixed.setAttribute('class', 'Token\u00a0Alpha');
        noNS.setAttributeNS('urn:other', 'p:class', 'Token Alpha');
        root.setAttribute('class', 'Token Alpha');
        const qualified = root.getElementsByTagName('DIV');
        const exactQualified = root.getElementsByTagName('p:DIV');
        const expanded = root.getElementsByTagNameNS(svgNS, 'DIV');
        const requested = root.getElementsByClassName('Token\rToken Alpha');
        if (!(qualified instanceof HTMLCollection) || qualified.length !== 3
            || qualified[0] !== html || qualified[1] !== svg || qualified[2] !== shield
            || exactQualified.length !== 1 || exactQualified[0] !== prefixed
            || root.getElementsByTagName('section').length !== 0
            || expanded.length !== 2 || expanded[0] !== prefixed || expanded[1] !== svg)
            return 'qualified and expanded matching must preserve ordinary order, prefixes and casing';
        if (document.getElementsByTagName('p:DIV')[0] !== prefixed
            || document.getElementsByTagNameNS('*', 'DIV').length !== 2
            || root.getElementsByTagNameNS(svgNS, '*').length !== 2
            || root.getElementsByTagNameNS(null, 'none')[0] !== noNS
            || root.getElementsByTagNameNS('', 'none')[0] !== noNS
            || root.getElementsByTagNameNS(svgNS, 'div').length !== 0
            || root.getElementsByTagNameNS('*', '*').length !== 6)
            return 'namespace and local-name wildcards must apply independently and empty namespace means null';
        if (requested.length !== 2 || requested[0] !== html || requested[1] !== svg
            || root.getElementsByClassName('token').length !== 0
            || root.getElementsByClassName(' \t\n\r\f').length !== 0
            || root.getElementsByClassName('Token\u00a0Alpha')[0] !== prefixed
            || document.getElementsByClassName('Token Alpha')[0] !== root)
            return 'class selection must use the requested tokens and exact native attributes';
        for (const invoke of [() => root.getElementsByTagName(),
            () => root.getElementsByTagNameNS(svgNS), () => root.getElementsByClassName(),
            () => Element.prototype.getElementsByClassName.call(document, 'Token'),
            () => Document.prototype.getElementsByTagNameNS.call(root, '*', '*')]) {
            try { invoke(); return 'invalid query accepted'; }
            catch (error) { if (!(error instanceof TypeError)) throw error; }
        }
        svg.removeAttribute('class');
        prefixed.setAttribute('class', 'Token Alpha');
        root.insertBefore(svg, html);
        if (requested.length !== 2 || requested[0] !== html || requested[1] !== prefixed
            || qualified[0] !== svg || expanded[0] !== svg || expanded[1] !== prefixed)
            return 'retained queries must observe current order and attributes';
        const xml = document.implementation.createDocument('urn:xml', 'root');
        xml.adoptNode(root);
        const current = root.getElementsByTagName('DIV');
        if (root.ownerDocument !== xml || current.length !== 1 || current[0] !== svg
            || qualified.length !== 3 || ![...qualified].includes(html)
            || expanded.length !== 2 || requested.length !== 2)
            return 'retained filters and new XML-mode queries must keep their specified semantics after adoption';
        return 'ok';
    })()",
    );
    assert_eq!(queries, "ok");
}

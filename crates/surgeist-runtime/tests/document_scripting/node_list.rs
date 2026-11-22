use super::{window_length::observe, *};

pub(super) fn node_lists_follow_current_canonical_nodes() {
    let mut session = DomSession::new(native_dom(), limits()).unwrap();
    let (_, top) = create(&mut session);
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 180_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    let child_nodes = observe(
        &mut host,
        &permit,
        r#"(() => {
            const fragment = document.createDocumentFragment();
            const children = fragment.childNodes;
            if (typeof NodeList !== 'function' || typeof document.getElementsByName !== 'function'
                || !(children instanceof NodeList))
                return 'NodeList, Node.childNodes and Document.getElementsByName must be exposed';

            const prototype = NodeList.prototype;
            const childGetter = Object.getOwnPropertyDescriptor(Node.prototype, 'childNodes').get;
            const lengthGetter = Object.getOwnPropertyDescriptor(prototype, 'length').get;
            if (Object.getPrototypeOf(prototype) !== Object.prototype
                || Object.getOwnPropertyNames(prototype).join() !== 'length,item,entries,keys,values,forEach,constructor'
                || prototype.constructor !== NodeList || NodeList.length !== 0 || prototype.item.length !== 1
                || Object.prototype.toString.call(children) !== '[object NodeList]')
                return 'NodeList must have its standard interface prototype';
            for (const name of ['entries', 'keys', 'values', 'forEach']) {
                const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
                if (descriptor.value !== Array.prototype[name] || !descriptor.writable
                    || !descriptor.enumerable || !descriptor.configurable)
                    return 'NodeList methods must be enumerable original Array intrinsics';
            }
            const iterator = Object.getOwnPropertyDescriptor(prototype, Symbol.iterator);
            if (iterator.value !== Array.prototype.values || iterator.enumerable
                || !iterator.writable || !iterator.configurable)
                return 'the default iterator must alias the original Array values method';
            for (const operation of [() => NodeList(), () => new NodeList(),
                () => childGetter.call(Object.create(fragment)),
                () => lengthGetter.call(Object.create(children)),
                () => prototype.item.call(Object.create(children), 0), () => children.item()]) {
                try { operation(); return 'accepted a forbidden call'; }
                catch (error) { if (!(error instanceof TypeError)) throw error; }
            }

            const element = document.createElement('div');
            const text = document.createTextNode('text');
            const comment = document.createComment('comment');
            if (children !== childGetter.call(fragment) || children === element.childNodes
                || children.length !== 0 || children.item(0) !== null || children[0] !== undefined)
                return 'childNodes must cache only its own empty ordinary child view';
            element.appendChild(document.createTextNode('descendant'));
            fragment.appendChild(text);
            fragment.appendChild(element);
            fragment.appendChild(comment);
            const index = Object.getOwnPropertyDescriptor(children, '1');
            if (children.length !== 3 || children[0] !== text || children.item(1) !== element
                || children[2] !== comment || Object.keys(children).join() !== '0,1,2'
                || Reflect.ownKeys(children).join() !== '0,1,2' || !index.enumerable
                || index.writable || !index.configurable || index.value !== element)
                return 'indices must expose original direct children and standard descriptors';
            if (Reflect.set(children, '1', comment) || Reflect.deleteProperty(children, '0')
                || Reflect.defineProperty(children, '0', {value: comment}) || children[0] !== text)
                return 'supported indices must refuse ordinary writes, definitions and deletes';

            const iter = children.entries();
            const [firstIndex, first] = iter.next().value;
            element.remove();
            const [secondIndex, second] = iter.next().value;
            if (firstIndex !== 0 || first !== text || secondIndex !== 1 || second !== comment
                || !iter.next().done || [...children.keys()].join() !== '0,1'
                || [...children][1] !== comment || children.item(-1) !== null || children[2] !== undefined)
                return 'retained collections and their iterators must read the live tree';
            const seen = [];
            children.forEach((node, index, owner) => { if (owner === children) seen[index] = node; });
            if (seen.length !== 2 || seen[0] !== text || seen[1] !== comment)
                return 'the original forEach must operate on the live indexed receiver';

            const other = new Document();
            other.adoptNode(fragment);
            fragment.insertBefore(element, text);
            const current = children.item({valueOf() { text.remove(); return 1; }});
            if (fragment.ownerDocument !== other || children !== fragment.childNodes
                || children.length !== 2 || children[0] !== element || current !== comment)
                return 'the same child view must survive adoption and read after index coercion';
            return 'ok';
        })()"#,
    );
    assert_eq!(child_nodes, "ok");

    let names = observe(
        &mut host,
        &permit,
        r#"(() => {
            const query = 'N\ud800\0\udc00';
            const read = document.getElementsByName;
            for (const operation of [() => read.call(Object.create(document), query),
                () => document.getElementsByName()]) {
                try { operation(); return 'accepted a forbidden name query'; }
                catch (error) { if (!(error instanceof TypeError)) throw error; }
            }
            const result = document.getElementsByName(query);
            const empty = document.getElementsByName('');
            const otherDoc = document.implementation.createHTMLDocument();
            const foreign = read.call(otherDoc, query);
            if (!(result instanceof NodeList) || result === empty || result === foreign
                || result.length !== 0 || empty.length !== 0 || foreign.length !== 0)
                return 'the result must belong to the exact query and Document';
            const parent = document.body.appendChild(document.createElement('div'));
            const custom = parent.appendChild(document.createElement('x-name-witness'));
            const child = parent.appendChild(document.createElement('span'));
            const withoutAttribute = parent.appendChild(document.createElement('p'));
            const explicitEmpty = parent.appendChild(document.createElement('p'));
            const svg = parent.appendChild(document.createElementNS('http://www.w3.org/2000/svg', 'g'));
            const namespaced = parent.appendChild(document.createElement('span'));
            const named = name => {
                const node = document.createElement('span');
                node.setAttribute('name', name);
                return node;
            };
            const template = parent.appendChild(document.createElement('template'));
            template.content.appendChild(named(query));
            const shadowHost = parent.appendChild(document.createElement('div'));
            shadowHost.attachShadow({mode: 'open'}).appendChild(named(query));
            const detached = named(query);
            custom.setAttribute('name', query);
            child.setAttribute('name', query);
            svg.setAttribute('name', query);
            namespaced.setAttributeNS('urn:other', 'p:name', query);
            explicitEmpty.setAttribute('name', '');
            if (result.length !== 2 || result[0] !== custom || result[1] !== child
                || empty.length !== 1 || empty[0] !== explicitEmpty || empty[0] === withoutAttribute
                || document.getElementsByName(query.toLowerCase()).length !== 0)
                return 'match exact UTF-16 on ordinary HTML descendants with a null-namespace name';
            parent.insertBefore(child, custom);
            namespaced.setAttribute('name', query);
            otherDoc.body.appendChild(detached);
            if ([...result].length !== 3 || result.item(0) !== child || result[1] !== custom
                || result[2] !== namespaced || foreign.length !== 1 || foreign[0] !== detached)
                return 'live query membership must follow reordering and remain isolated by Document';
            child.removeAttribute('name');
            custom.remove();
            explicitEmpty.removeAttribute('name');
            if (result.length !== 1 || result[0] !== namespaced || !('0' in result) || '1' in result
                || empty.length !== 0 || foreign[0] !== detached)
                return 'the retained queries must observe removals and attribute changes';
            parent.remove();
            if (result.length !== 0) return 'the retained query must exclude detached subtrees';
            return 'ok';
        })()"#,
    );
    assert_eq!(names, "ok");
}

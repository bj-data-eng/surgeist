use super::{window_length::observe, *};

pub(super) fn class_list_observes_and_edits_the_canonical_attribute() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 200_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const element = document.createElement('section');
            const list = element.classList;
            if (typeof DOMTokenList !== 'function' || list === undefined) return 'missing DOMTokenList or classList';
            const throws = (action, name) => {
                try { action(); } catch (error) { return error.name === name; }
                return false;
            };
            const descriptor = Object.getOwnPropertyDescriptor(Element.prototype, 'classList');
            if (!(list instanceof DOMTokenList) || list !== element.classList ||
                !descriptor.enumerable || !descriptor.configurable ||
                descriptor.get.name !== 'get classList' || descriptor.get.length !== 0 ||
                descriptor.set.name !== 'set classList' || descriptor.set.length !== 1 ||
                !throws(() => new DOMTokenList(), 'TypeError') ||
                !throws(() => list.supports('one'), 'TypeError')) return 'classList interface and descriptor';
            list.add();
            list.remove('absent');
            if (element.hasAttribute('class')) return 'empty update must leave absent attribute absent';
            element.setAttributeNS('urn:other', 'p:class', 'foreign');
            const raw = '  one\ttwo one\r\nthree\f\ud800\0\udc00  non\u00a0breaking vertical\u000btab ';
            list.value = raw;
            const canonical = ['one', 'two', 'three', '\ud800\0\udc00', 'non\u00a0breaking', 'vertical\u000btab'];
            const own = Object.getOwnPropertyDescriptor(list, '0');
            if (element.getAttributeNS(null, 'class') !== raw ||
                element.getAttributeNS('urn:other', 'class') !== 'foreign' ||
                list.value !== raw || list.toString() !== raw || String(list) !== raw ||
                list.length !== canonical.length || list.item(0) !== 'one' || list[0] !== 'one' ||
                list[6] !== undefined || list.item(6) !== null || !list.contains(canonical[3]) ||
                list.contains('') || list.contains('one two') ||
                Object.keys(list).join() !== '0,1,2,3,4,5' || own?.value !== 'one' ||
                !own.enumerable || !own.configurable || own.writable ||
                Array.from(list).some((value, index) => value !== canonical[index]))
                return 'raw class value and first-occurrence UTF-16 ordered set';
            if (!throws(() => list.add('valid', ''), 'SyntaxError') ||
                !throws(() => list.remove('one', 'two three'), 'InvalidCharacterError') ||
                !throws(() => list.replace('bad token', ''), 'SyntaxError') ||
                list.value !== raw) return 'validation must precede mutation in pinned order';
            if (list.toggle('one', true) !== true || list.toggle('missing', false) !== false ||
                list.replace('missing', 'other') !== false || list.value !== raw)
                return 'branch-specific skipped updates must preserve raw attribute';
            list.add('one');
            if (list.value !== canonical.join(' ')) return 'duplicate add must run ordered-set update';
            list.value = ' a b c a ';
            if (!list.replace('a', 'c') || list.value !== 'c b' ||
                !list.replace('b', 'b') || list.toggle('b', undefined) !== false ||
                list.value !== 'c') return 'replace order and omitted-force toggle';
            element.removeAttribute('class');
            if (list !== element.classList || list.value !== '' || list.length !== 0)
                return 'retained classList missed native attribute removal';
            let calls = 0;
            const reentrant = {toString() { ++calls; element.setAttribute('class', 'reentered'); return 'added'; }};
            list.add(reentrant);
            if (calls !== 1 || list.value !== 'reentered added')
                return 'edit must use canonical state after argument coercion';
            const rhs = {toString() { return ' a   a b '; }};
            element.classList = rhs;
            if (list.value !== ' a   a b ' || list[1] !== 'b') return 'ordinary value forwarding';
            const setter = descriptor.set;
            let actual;
            Object.defineProperty(element, 'classList', {value: {set value(value) { actual = value; }}, configurable: true});
            setter.call(element, rhs);
            if (actual !== rhs || list.value !== ' a   a b ')
                return 'forward original RHS through the dynamically reached object';
            const frozen = Object.freeze({value: 'fixed'});
            Object.defineProperty(element, 'classList', {value: frozen, configurable: true});
            if (setter.call(element, rhs) !== undefined || frozen.value !== 'fixed')
                return 'ordinary Set rejection must remain non-throwing';
            Object.defineProperty(element, 'classList', {value: 0, configurable: true});
            if (!throws(() => setter.call(element, rhs), 'TypeError'))
                return 'forwarded target must be an object';
            const coercesLater = {};
            try {
                list.add('', {toString() { throw coercesLater; }});
                return 'validation skipped later coercion';
            } catch (error) {
                if (error !== coercesLater || list.value !== ' a   a b ')
                    return 'argument conversion must precede validation and preserve the exception';
            }
            const sentinel = {};
            Object.defineProperty(element, 'classList', {get() { throw sentinel; }, configurable: true});
            try { setter.call(element, rhs); return 'forwarded getter swallowed throw'; }
            catch (error) { if (error !== sentinel) return 'forwarded getter changed throw identity'; }
            delete element.classList;
            if (!throws(() => setter.call(new Proxy(element, {}), rhs), 'TypeError') ||
                !throws(() => DOMTokenList.prototype.contains.call(Object.create(list), rhs), 'TypeError'))
                return 'receiver brand is exact before forwarding and coercion';
            const iterator = list.values();
            if (iterator.next().value !== 'a') return 'iterator starts on canonical first token';
            element.setAttribute('class', 'a new third');
            if (iterator.next().value !== 'new' || list.item(1) !== 'new' ||
                DOMTokenList.prototype[Symbol.iterator] !== Array.prototype.values ||
                DOMTokenList.prototype.values !== Array.prototype.values)
                return 'iterator must observe live values using realm intrinsics';
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}

pub(super) fn class_list_uses_native_attribute_reactions_once() {
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
            if (typeof DOMTokenList !== 'function') return 'missing DOMTokenList';
            const notes = [];
            let removeReturnedToken = false;
            customElements.define('x-token-list-reaction', class extends HTMLElement {
                static observedAttributes = ['class'];
                attributeChangedCallback(name, before, after, namespace) {
                    notes.push({receiver: this, name, before, after, namespace});
                    if (removeReturnedToken && after === 'a toggled') {
                        this.classList.remove('toggled');
                    }
                }
            });
            const element = document.createElement('x-token-list-reaction');
            document.body.appendChild(element);
            const list = element.classList;
            element.setAttribute('class', 'a');
            notes.length = 0;
            list.add('a');
            list.remove();
            list.remove('absent');
            list.replace('a', 'a');
            if (notes.length !== 4 || notes.some(note => note.receiver !== element ||
                note.name !== 'class' || note.namespace !== null ||
                note.before !== 'a' || note.after !== 'a')) return 'same-value updates lost native reactions';
            notes.length = 0;
            list.toggle('a', true);
            list.toggle('absent', false);
            list.replace('absent', 'another');
            if (notes.length !== 0) return 'skipped updates produced a reaction';
            removeReturnedToken = true;
            if (list.toggle('toggled') !== true || list.contains('toggled') ||
                notes.length !== 2 || notes[0].before !== 'a' || notes[0].after !== 'a toggled' ||
                notes[1].before !== 'a toggled' || notes[1].after !== 'a')
                return 'the original toggle completion must survive its reentrant reaction';
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}

pub(super) fn class_list_keeps_its_element_creation_realm() {
    let (mut session, top) = setup();
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(top.document(), None);
    let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let frame = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&frame)
        .unwrap()
        .unwrap();
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 220_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    let before = observe(
        &mut host,
        &permit,
        r"(() => {
        const child = window[0];
        const first = child.document.createElement('div');
        const late = child.document.createElement('div');
        const get = Object.getOwnPropertyDescriptor(Element.prototype, 'classList').get;
        const childGet = Object.getOwnPropertyDescriptor(child.Element.prototype, 'classList').get;
        const firstList = get.call(first);
        const local = document.createElement('div');
        if (firstList !== childGet.call(first) || !(firstList instanceof child.DOMTokenList) ||
            firstList instanceof DOMTokenList || childGet.call(local) !== local.classList ||
            firstList.values !== child.Array.prototype.values || get === childGet)
            return 'wrong first projection for borrowed methods';
        firstList.value = 'live old';
        if (document.adoptNode(first) !== first || document.adoptNode(late) !== late ||
            first.ownerDocument !== document || first.classList !== firstList)
            return 'adoption lost the list or receiving Document';
        globalThis.classListProbe = {
            first, late, firstList, get, prototype: child.DOMTokenList.prototype
        };
        return 'ok';
    })()",
    );
    assert_eq!(before, "ok");

    let snapshot = host.snapshot().unwrap();
    let child = window_length::content_navigable(&snapshot, frame).unwrap();
    let document = snapshot
        .hosted_navigable(child)
        .unwrap()
        .active_document()
        .unwrap();
    window_length::edit(&mut host, DomSourceEdit::detach(frame));
    host.with_native(|dom, mut hosted, _| {
        for _ in 0..128 {
            if dom
                .snapshot()
                .document(document)
                .unwrap()
                .navigable()
                .is_none()
            {
                return;
            }
            hosted.run_next(dom, top.navigable(), &cancel()).unwrap();
        }
        panic!("native child Document was not disassociated");
    })
    .unwrap();
    let after = observe(
        &mut host,
        &permit,
        r"(() => {
        const {first, late, firstList, get, prototype} = classListProbe;
        const lateList = get.call(late);
        first.setAttribute('class', 'retired origin');
        lateList.add('late');
        if (window.length !== 0 || first.classList !== firstList || firstList[1] !== 'origin' ||
            lateList !== late.classList || late.getAttribute('class') !== 'late' ||
            Object.getPrototypeOf(firstList) !== prototype || Object.getPrototypeOf(lateList) !== prototype)
            return 'retirement lost existing or first-exposed classList identity';
        return 'ok';
    })()",
    );
    assert_eq!(after, "ok");
    host.stop().into_session().unwrap();
}

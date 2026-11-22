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
          try {
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
                    !throwsTypeError(() => descriptor.get.call(detached.createElement('div'))) ||
                    !throwsTypeError(() => descriptor.set.call(detached.createElement('div'), 'wrong')))
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
            const style = buckets.style;
            const scriptGetter = Object.getOwnPropertyDescriptor(HTMLScriptElement.prototype, 'blocking');
            if (!throwsTypeError(() => scriptGetter.get.call(style)) ||
                !throwsTypeError(() => scriptGetter.set.call(style, 'wrong')))
                return 'wrong interface shared a blocking cache entry';
            const styleSetter = Object.getOwnPropertyDescriptor(HTMLStyleElement.prototype, 'blocking').set;
            const exactRhs = { toString() { throw new Error('forwarding converted too early'); } };
            const rejected = Object.defineProperty({}, 'value', { value: 'unchanged' });
            Object.defineProperty(style, 'blocking', { configurable: true, get: () => rejected });
            styleSetter.call(style, exactRhs);
            if (rejected.value !== 'unchanged') return 'read-only forwarded value changed';
            const thrown = {};
            Object.defineProperty(style, 'blocking', {
                configurable: true,
                get() { throw thrown; },
            });
            try {
                styleSetter.call(style, exactRhs);
                return 'forwarded getter exception lost';
            } catch (error) {
                if (error !== thrown) return 'forwarding changed the thrown value';
            }
            let forwarded;
            Object.defineProperty(style, 'blocking', {
                get: () => ({ set value(value) { forwarded = value; } }),
            });
            styleSetter.call(style, exactRhs);
            if (forwarded !== exactRhs) return 'forwarding changed the assigned value';
            delete style.blocking;
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
            const effects = [];
            class WatchedScript extends HTMLScriptElement {
                static observedAttributes = ['blocking'];
                attributeChangedCallback(name, before, after, namespace) {
                    effects.push([name, before, after, namespace, this.blocking.value]);
                    if (after === 'render reentrant') this.blocking.remove('reentrant');
                }
            }
            customElements.define('x-observed-blocking', WatchedScript, { extends: 'script' });
            const watched = document.createElement('script', { is: 'x-observed-blocking' });
            watched.blocking = 'render';
            if (watched.blocking.toggle('reentrant') !== true || watched.blocking.value !== 'render' ||
                JSON.stringify(effects) !== JSON.stringify([
                    ['blocking', null, 'render', null, 'render'],
                    ['blocking', 'render', 'render reentrant', null, 'render reentrant'],
                    ['blocking', 'render reentrant', 'render', null, 'render'],
                ]))
                return 'HTML token-list Source lost the original reaction or toggle result';
            return 'ok';
          } catch (error) {
            return 'unexpected HTML token-list exception: ' + String(error) + '\n' + (error.stack ?? '');
          }
        })()"#,
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}

pub(super) fn html_token_lists_preserve_hosted_refusals() {
    for (element, expression, native) in [
        (
            "a",
            "subject.relList.add('prefetch')",
            Some(DomError::HyperlinkProcessingUnavailable),
        ),
        (
            "area",
            "subject.relList.toggle('prefetch')",
            Some(DomError::HyperlinkProcessingUnavailable),
        ),
        (
            "link",
            "subject.sizes = '16x16'",
            Some(DomError::LinkProcessingUnavailable),
        ),
        ("link", "subject.relList.supports('stylesheet')", None),
        ("iframe", "subject.sandbox.supports('allow-popups')", None),
    ] {
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
        let document = if element == "iframe" {
            "document.implementation.createHTMLDocument()"
        } else {
            "document"
        };
        observe(
            &mut host,
            &permit,
            &format!("globalThis.subject = {document}.createElement('{element}'); 'ok'"),
        );
        let before = host.snapshot().unwrap().stamp();
        let outcome = host.in_document(&permit, |scope| {
            scope
                .evaluate(&expression.encode_utf16().collect::<Vec<_>>())
                .map(drop)
        });
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = outcome else {
            panic!("{expression} should preserve its unavailable native capability: {outcome:?}");
        };
        if let Some(expected) = native {
            assert!(matches!(
                failure.cause::<DomSourceDriveError>(),
                Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(actual)))
                    if *actual == expected
            ));
        } else {
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::HostRouteUnavailable)
            ));
        }
        assert_eq!(host.snapshot().unwrap().stamp(), before);
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

pub(super) fn html_token_lists_preserve_adopted_creation_identity() {
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
    let result = observe(
        &mut host,
        &permit,
        r"(() => {
            const frame = document.getElementsByTagName('iframe')[0];
            const child = window[0];
            const childDocument = child.document;
            const sandbox = frame.sandbox;
            sandbox.add('allow-scripts');
            frame.sandbox = 'allow-same-origin';
            if (frame.sandbox !== sandbox || frame.getAttribute('sandbox') !== 'allow-same-origin' ||
                child.document !== childDocument)
                return 'iframe Source publication changed the associated child Document';
            const first = childDocument.createElement('output');
            const late = childDocument.createElement('output');
            const parentGetter = Object.getOwnPropertyDescriptor(HTMLOutputElement.prototype, 'htmlFor').get;
            const childGetter = Object.getOwnPropertyDescriptor(child.HTMLOutputElement.prototype, 'htmlFor').get;
            const list = parentGetter.call(first);
            if (childGetter.call(first) !== list || list.values !== child.Array.prototype.values ||
                Object.getPrototypeOf(list) !== child.DOMTokenList.prototype ||
                list instanceof DOMTokenList || parentGetter === childGetter)
                return 'borrowed HTML accessor selected the wrong creation realm';
            if (document.adoptNode(first) !== first || document.adoptNode(late) !== late ||
                first.htmlFor !== list || late.ownerDocument !== document)
                return 'native adoption changed the HTML list association';
            globalThis.htmlListProbe = {
                first, late, list, parentGetter, prototype: child.DOMTokenList.prototype
            };
            return 'ok';
        })()",
    );
    assert_eq!(result, "ok");
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
    let result = observe(
        &mut host,
        &permit,
        r"(() => {
            const {first, late, list, parentGetter, prototype} = htmlListProbe;
            first.setAttribute('for', 'label initial');
            const lateList = parentGetter.call(late);
            late.htmlFor = 'label later';
            if (first.htmlFor !== list || list[1] !== 'initial' ||
                lateList === list || late.htmlFor !== lateList || late.getAttribute('for') !== 'label later' ||
                Object.getPrototypeOf(lateList) !== prototype)
                return 'retired creation realm lost a live or newly exposed list';
            return 'ok';
        })()",
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}

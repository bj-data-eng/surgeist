use super::{evaluate, hosted_session};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn document_title_selects_and_updates_the_live_title() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const first = document.body.appendChild(document.createElement('title'));
                first.appendChild(document.createTextNode(' \t A\ud800 \n'));
                first.appendChild(document.createElement('span')).textContent = 'ignored descendant';
                first.appendChild(document.createTextNode('\r B \t'));
                const second = document.body.appendChild(document.createElement('title'));
                second.text = 'later title';
                if (!(document instanceof Document) || first.parentNode !== document.body ||
                    first.text !== ' \t A\ud800 \n\r B \t' || second.text !== 'later title')
                    return 'native title fixture';
                const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'title');
                if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function')
                    return 'document missing title accessor';
                if (document.title !== 'A\ud800 B') return 'first ordinary title and direct text normalization';
                const raw = '\n\udc00\0  replacement \r';
                document.title = raw;
                if (first.text !== raw || first.firstChild !== first.lastChild ||
                    second.text !== 'later title' || document.title !== '\udc00\0 replacement')
                    return 'exact replacement and normalized read';
                document.body.removeChild(first);
                document.body.removeChild(second);
                document.title = '';
                if (document.head.lastChild?.localName !== 'title' ||
                    document.head.lastChild.hasChildNodes() || document.title !== '')
                    return 'empty setter creates a title in the existing head';
                const ns = 'http://www.w3.org/2000/svg';
                const svg = document.implementation.createDocument(ns, 's:svg');
                const oldFirst = svg.documentElement.appendChild(svg.createComment('kept'));
                if (svg.title !== '') return 'empty SVG title';
                svg.title = ' SVG\t title ';
                const svgTitle = svg.documentElement.firstChild;
                return svgTitle?.namespaceURI === ns && svgTitle.localName === 'title' &&
                    svgTitle.nextSibling === oldFirst && svgTitle.textContent === ' SVG\t title ' &&
                    svg.title === 'SVG title' ? 'ok' : 'SVG first-child creation, exact replacement and read';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}

pub(super) fn document_title_observes_ordinary_names_and_creation_boundaries() {
    check_title_script(
        r#"
        (() => {
            const html = 'http://www.w3.org/1999/xhtml';
            const svg = 'http://www.w3.org/2000/svg';
            const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'title');
            if (!descriptor.enumerable || !descriptor.configurable ||
                descriptor.get.length !== 0 || descriptor.get.name !== 'get title' ||
                descriptor.set.length !== 1 || descriptor.set.name !== 'set title')
                return 'Document title descriptor';
            const cold = new Document();
            cold.title = 'unused';
            if (cold.title !== '' || cold.firstChild !== null) return 'empty document';

            const doc = document.implementation.createDocument(html, 'p:body');
            const root = doc.documentElement;
            const head = root.appendChild(doc.createElementNS(html, 'p:head'));
            doc.title = 'unused';
            if (head.firstChild !== null || doc.title !== '') return 'non-html local root cannot create title';
            const wrongNamespace = root.appendChild(doc.createElementNS('urn:foreign', 'title'));
            const wrongCase = root.appendChild(doc.createElementNS(html, 'p:TITLE'));
            wrongNamespace.textContent = wrongCase.textContent = 'ignored';
            const template = root.appendChild(doc.createElementNS(html, 'template'));
            template.content.appendChild(doc.createElementNS(html, 'title')).textContent = 'inert';
            const host = root.appendChild(doc.createElementNS(html, 'div'));
            const shadow = host.attachShadow({mode: 'closed'});
            shadow.appendChild(doc.createElementNS(html, 'title')).textContent = 'shadow';
            const parent = root.appendChild(doc.createElementNS(html, 'div'));
            const actual = parent.appendChild(doc.createElementNS(html, 'p:title'));
            actual.textContent = '  visible\u00a0unicode\u000b  title  ';
            if (doc.title !== 'visible\u00a0unicode\u000b title') return 'ordinary prefixed title';
            doc.title = ' exact\t';
            if (parent.firstChild !== actual || actual.textContent !== ' exact\t' || head.firstChild !== null)
                return 'non-html local root updates an existing HTML title';

            const foreign = document.implementation.createDocument(svg, 's:g');
            const foreignTitle = foreign.documentElement.appendChild(foreign.createElementNS(html, 'h:title'));
            foreignTitle.textContent = '  ordinary  ';
            foreign.title = 'ignored';
            if (foreign.title !== 'ordinary' || foreignTitle.textContent !== '  ordinary  ')
                return 'SVG non-svg root getter and no-op setter';
            const svgDoc = document.implementation.createDocument(svg, 's:svg');
            const svgRoot = svgDoc.documentElement;
            svgRoot.appendChild(svgDoc.createElementNS(html, 'title')).textContent = 'HTML sibling';
            svgRoot.appendChild(svgDoc.createElementNS(svg, 'g'))
                .appendChild(svgDoc.createElementNS(svg, 'title')).textContent = 'nested';
            svgRoot.appendChild(svgDoc.createElementNS(svg, 'TITLE')).textContent = 'uppercase';
            if (svgDoc.title !== '') return 'only direct exact SVG title';
            const svgFirst = svgRoot.appendChild(svgDoc.createElementNS(svg, 's:title'));
            const svgLater = svgRoot.appendChild(svgDoc.createElementNS(svg, 'title'));
            svgFirst.textContent = ' first\t'; svgLater.textContent = 'second';
            if (svgDoc.title !== 'first') return 'first direct SVG title';
            svgDoc.title = '';
            if (svgFirst.hasChildNodes() || svgFirst.nextSibling !== svgLater ||
                svgLater.textContent !== 'second' || svgDoc.title !== '') return 'existing SVG identity';
            return 'ok';
        })()
    "#,
    );
}

pub(super) fn document_title_preserves_coercion_and_native_reaction_order() {
    check_title_script(
        r#"
        (() => {
            const first = document.body.appendChild(document.createElement('title'));
            const second = document.body.appendChild(document.createElement('title'));
            first.text = 'original'; second.text = 'later';
            const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'title');
            let called = false;
            try { descriptor.set.call(Object.create(Document.prototype), {toString() {called = true;}});
                return 'forged Document accepted'; }
            catch (error) { if (!(error instanceof TypeError) || called) return 'receiver before coercion'; }
            const original = {};
            try { document.title = {toString() { throw original; }}; return 'coercion succeeded'; }
            catch (error) { if (error !== original || first.text !== 'original') return 'original coercion error'; }

            const observations = [];
            customElements.define('x-document-title-child', class extends HTMLElement {
                disconnectedCallback() {
                    observations.push([this, document.title, second.text, this.parentNode]);
                }
            });
            const custom = second.appendChild(document.createElement('x-document-title-child'));
            document.title = {toString() {
                document.body.removeChild(first);
                return ' \n exact\ud800\0  text ';
            }};
            const [event] = observations;
            if (observations.length !== 1 || event[0] !== custom || event[1] !== 'exact\ud800\0 text' ||
                event[2] !== ' \n exact\ud800\0  text ' || event[3] !== null ||
                second.parentNode !== document.body || first.text !== 'original') return 'live target and reaction order';
            descriptor.set.call(document);
            return second.text === 'undefined' && document.title === 'undefined'
                ? 'ok' : 'exact DOMString conversion';
        })()
    "#,
    );
}

fn check_title_script(script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, script)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    })
    .unwrap();
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}

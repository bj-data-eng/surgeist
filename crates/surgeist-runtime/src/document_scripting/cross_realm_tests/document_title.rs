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

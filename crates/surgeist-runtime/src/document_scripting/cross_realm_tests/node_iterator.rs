use super::{Error, PropertyKey, evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::tree::DomNodeIteratorFilter;

pub(super) fn iterator_tracks_the_native_pointer_through_removal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_node_iterators(8));
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const root = document.createElement('div');
                const left = root.appendChild(document.createTextNode('left'));
                const right = root.appendChild(document.createTextNode('right'));
                if (root.firstChild !== left || left.nextSibling !== right) return 'native tree fixture';
                if (typeof document.createNodeIterator !== 'function') return 'missing native iterator factory';
                const cursor = document.createNodeIterator(root, NodeFilter.SHOW_TEXT);
                if (!(cursor instanceof NodeIterator) || cursor.root !== root || cursor.referenceNode !== root ||
                    !cursor.pointerBeforeReferenceNode || cursor.whatToShow !== 4 || cursor.filter !== null)
                    return 'native initial iterator state';
                if (cursor.nextNode() !== left || cursor.referenceNode !== left || cursor.pointerBeforeReferenceNode)
                    return 'first matching text and cursor direction';
                root.removeChild(left);
                if (cursor.referenceNode !== root || cursor.pointerBeforeReferenceNode || cursor.nextNode() !== right)
                    return 'native pre-remove cursor adjustment';
                if (cursor.previousNode() !== right || !cursor.pointerBeforeReferenceNode ||
                    cursor.previousNode() !== null || cursor.referenceNode !== right)
                    return 'backward traversal and exhausted cursor';
                return cursor.detach() === undefined && cursor.nextNode() === right ? 'ok' : 'detach no-op';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn filter_preserves_abrupt_completion_reentry_and_live_removal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_node_iterators(8));
    let independent = session
        .dom()
        .raw_mut()
        .create_node_iterator(
            target.document().node(),
            u32::MAX,
            DomNodeIteratorFilter::None,
            &cancel,
        )
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const root = document.createElement('p');
                const first = root.appendChild(document.createTextNode('first'));
                const last = root.appendChild(document.createTextNode('last'));
                const thrown = {};
                const visited = [];
                let lookups = 0;
                let cursor;
                const filter = {
                    get acceptNode() {
                        if (++lookups === 1) throw thrown;
                        return function(node) {
                            if (this !== filter) throw 'object filter receiver';
                            visited.push(node);
                            if (node === first) {
                                try { cursor.nextNode(); throw 'recursive traversal returned'; }
                                catch (error) {
                                    if (!(error instanceof DOMException) || error.name !== 'InvalidStateError') throw error;
                                }
                                root.removeChild(first);
                            }
                            return {valueOf() { return 65537; }};
                        };
                    }
                };
                cursor = document.createNodeIterator(root, NodeFilter.SHOW_TEXT, filter);
                if (cursor.filter !== filter || lookups !== 0) return 'callback stored without lookup';
                try { cursor.nextNode(); return 'missing original abrupt completion'; }
                catch (error) {
                    if (error !== thrown || cursor.referenceNode !== root || !cursor.pointerBeforeReferenceNode)
                        return 'abrupt completion and unchanged cursor';
                }
                if (cursor.nextNode() !== first || cursor.referenceNode !== root || cursor.pointerBeforeReferenceNode)
                    return 'accepted detached node and adjusted cursor';
                if (cursor.nextNode() !== last || lookups !== 3 || visited.length !== 2 ||
                    visited[0] !== first || visited[1] !== last) return 'live callback sequence';
                const adopted = new Document();
                if (adopted.adoptNode(root) !== root || root.ownerDocument !== adopted ||
                    cursor.root !== root || cursor.referenceNode !== last || cursor.previousNode() !== last)
                    return 'native cursor and root survive adoption';
                globalThis.retainedIterator = cursor;
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| assert_eq!(dom.usage().node_iterators(), 2))
        .unwrap();
    let mut session = host.stop().into_session().unwrap();
    let dom = session.dom();
    assert_eq!(dom.usage().node_iterators(), 1);
    assert_eq!(
        dom.raw().node_iterator(independent).unwrap().root(),
        target.document().node()
    );
}

pub(super) fn filter_interface_and_factory_follow_the_pinned_descriptors() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_node_iterators(8));
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                if (typeof NodeFilter !== 'function' || NodeFilter.name !== 'NodeFilter' ||
                    NodeFilter.length !== 0 || 'prototype' in NodeFilter) return 'legacy callback interface object';
                const all = Object.getOwnPropertyDescriptor(NodeFilter, 'SHOW_ALL');
                if (all.value !== 4294967295 || all.writable || !all.enumerable || all.configurable ||
                    NodeFilter.SHOW_ENTITY_REFERENCE !== 16 || NodeFilter.SHOW_NOTATION !== 2048)
                    return 'constant descriptors and legacy bitmask';
                for (const call of [() => NodeFilter(), () => new NodeFilter(), () => new NodeIterator()]) {
                    try { call(); return 'illegal interface call accepted'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'interface call error'; }
                }
                const factory = Document.prototype.createNodeIterator;
                if (factory.name !== 'createNodeIterator' || factory.length !== 1) return 'factory descriptor';
                if (Object.keys(NodeIterator.prototype).join() !==
                    'root,referenceNode,pointerBeforeReferenceNode,whatToShow,filter,nextNode,previousNode,detach')
                    return 'iterator declaration order';
                let maskConverted = false;
                const mask = {valueOf() { maskConverted = true; return -1; }};
                try { document.createNodeIterator({}, mask, 0); return 'invalid root accepted'; }
                catch (error) { if (!(error instanceof TypeError) || maskConverted) return 'root before mask'; }
                try { document.createNodeIterator(document, mask, 0); return 'invalid filter accepted'; }
                catch (error) { if (!(error instanceof TypeError) || !maskConverted) return 'mask before filter'; }
                const callback = function() {
                    'use strict';
                    if (this !== undefined) throw 'function receiver';
                    return NodeFilter.FILTER_ACCEPT;
                };
                callback.acceptNode = () => { throw 'looked up method on callable'; };
                const first = document.createNodeIterator(document, undefined, callback);
                const second = document.createNodeIterator(document);
                if (first === second || first.root !== document || second.root !== document ||
                    first.whatToShow !== NodeFilter.SHOW_ALL || first.filter !== callback ||
                    first.nextNode() !== document || !second.pointerBeforeReferenceNode) return 'independent native cursors';
                const next = NodeIterator.prototype.nextNode;
                try { next.call(Object.create(first)); return 'forged cursor accepted'; }
                catch (error) { return error instanceof TypeError ? 'ok' : 'iterator method brand'; }
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn filter_dispatch_uses_its_associated_realm() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_node_iterators(8));
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    let (filter, type_error) = {
        let engine = host.engine.as_mut().unwrap();
        let realm = engine
            .creations
            .find(origin.global(), || Ok::<_, Error>(()))
            .unwrap()
            .unwrap();
        engine
            .process
            .execution()
            .in_realm(realm, |scope| {
                let filter = evaluate(scope, "({ acceptNode: 7 })")?;
                let error = evaluate(scope, "TypeError")?;
                Ok((scope.retain(&filter), scope.retain(&error)))
            })
            .unwrap()
    };
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        for (name, root) in [("foreignIteratorFilter", filter), ("foreignIteratorTypeError", type_error)] {
            let input = scope.import(&root)?;
            scope.set(&global, PropertyKey::String(&name.encode_utf16().collect::<Vec<_>>()), &input)?;
        }
        let result = evaluate(scope, r#"
            (() => {
                const cursor = document.createNodeIterator(document, undefined, foreignIteratorFilter);
                try { cursor.nextNode(); return 'noncallable callback accepted'; }
                catch (error) {
                    if (!(error instanceof foreignIteratorTypeError) || error instanceof TypeError ||
                        !cursor.pointerBeforeReferenceNode) return 'callback operation error realm';
                }
                foreignIteratorFilter.acceptNode = () => Symbol();
                try { cursor.nextNode(); return 'invalid callback result accepted'; }
                catch (error) {
                    if (!(error instanceof foreignIteratorTypeError) || error instanceof TypeError ||
                        !cursor.pointerBeforeReferenceNode) return 'callback return conversion realm';
                }
                foreignIteratorFilter.acceptNode = (node) => node === document ? 1 : 3;
                return cursor.nextNode() === document ? 'ok' : 'canonical callback output';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

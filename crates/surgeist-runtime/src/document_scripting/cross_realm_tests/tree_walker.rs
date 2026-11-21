use super::{Error, PropertyKey, evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn the_factory_exposes_a_live_cursor_and_standard_prototype() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_tree_walkers(8));
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const root = document.createElement('div');
                const left = root.appendChild(document.createElement('p'));
                const leaf = left.appendChild(document.createTextNode('leaf'));
                const right = root.appendChild(document.createElement('span'));
                if (root.firstChild !== left || left.firstChild !== leaf || left.nextSibling !== right)
                    return 'native tree fixture';
                const factory = Document.prototype.createTreeWalker;
                if (typeof factory !== 'function') return 'missing native TreeWalker factory';
                if (factory.name !== 'createTreeWalker' || factory.length !== 1) return 'factory descriptor';
                if (TreeWalker.name !== 'TreeWalker' || TreeWalker.length !== 0 ||
                    Object.keys(TreeWalker.prototype).join() !==
                    'root,whatToShow,filter,currentNode,parentNode,firstChild,lastChild,previousSibling,nextSibling,previousNode,nextNode')
                    return 'TreeWalker interface shape';
                try { new TreeWalker(); return 'constructible TreeWalker'; }
                catch (error) { if (!(error instanceof TypeError)) return 'TreeWalker constructor error'; }
                const walk = document.createTreeWalker(root);
                const separate = document.createTreeWalker(root);
                if (walk === separate || !(walk instanceof TreeWalker) || walk.root !== root ||
                    walk.currentNode !== root || walk.whatToShow !== NodeFilter.SHOW_ALL || walk.filter !== null)
                    return 'TreeWalker initial state';
                const current = Object.getOwnPropertyDescriptor(TreeWalker.prototype, 'currentNode');
                try { current.get.call(Object.create(walk)); return 'forged TreeWalker receiver'; }
                catch (error) { if (!(error instanceof TypeError)) return 'TreeWalker getter brand'; }
                try { NodeIterator.prototype.nextNode.call(walk); return 'walker accepted as iterator'; }
                catch (error) { if (!(error instanceof TypeError)) return 'separate cursor brands'; }
                if (walk.firstChild() !== left || walk.nextNode() !== leaf || walk.parentNode() !== left ||
                    walk.nextSibling() !== right || walk.previousSibling() !== left || walk.parentNode() !== root ||
                    walk.lastChild() !== right || walk.previousNode() !== leaf || walk.currentNode !== leaf ||
                    separate.currentNode !== root) return 'seven native directions and independent positions';
                const outside = document.createElement('section');
                walk.currentNode = outside;
                if (walk.currentNode !== outside || walk.root !== root || walk.parentNode() !== null)
                    return 'outside-root current node';
                try { walk.currentNode = null; return 'null current node accepted'; }
                catch (error) { if (!(error instanceof TypeError) || walk.currentNode !== outside) return 'Node setter brand'; }
                let converted = false;
                const mask = {valueOf() { converted = true; return -1; }};
                try { document.createTreeWalker({}, mask, 0); return 'non-Node root accepted'; }
                catch (error) { if (!(error instanceof TypeError) || converted) return 'root before mask'; }
                try { document.createTreeWalker(root, mask, 0); return 'invalid filter accepted'; }
                catch (error) { if (!(error instanceof TypeError) || !converted) return 'mask before filter'; }
                const adopted = new Document();
                if (adopted.adoptNode(root) !== root || root.ownerDocument !== adopted || walk.root !== root)
                    return 'root identity through adoption';
                walk.currentNode = root;
                return walk.firstChild() === left ? 'ok' : 'live navigation after adoption';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn filter_calls_preserve_live_edits_identity_and_abrupt_completion() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_tree_walkers(8));
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const root = document.createElement('div');
                const first = root.appendChild(document.createTextNode('first'));
                const last = root.appendChild(document.createTextNode('last'));
                const abrupt = {};
                let lookups = 0;
                let walk;
                const filter = {
                    get acceptNode() {
                        if (++lookups === 1) throw abrupt;
                        return function(node) {
                            if (this !== filter || node !== first) throw 'filter receiver or argument';
                            try { walk.firstChild(); throw 'recursive filter returned'; }
                            catch (error) {
                                if (!(error instanceof DOMException) || error.name !== 'InvalidStateError') throw error;
                            }
                            walk.currentNode = last;
                            root.removeChild(first);
                            return {valueOf() { return 65537; }};
                        };
                    }
                };
                walk = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, filter);
                if (walk.filter !== filter || lookups !== 0) return 'filter stored without lookup';
                try { walk.firstChild(); return 'abrupt filter returned'; }
                catch (error) {
                    if (error !== abrupt || walk.currentNode !== root) return 'original abrupt completion';
                }
                if (walk.firstChild() !== first || walk.currentNode !== first || first.parentNode !== null ||
                    root.firstChild !== last || lookups !== 2) return 'accepted detached result publishes over reentrant write';
                globalThis.retainedTreeWalker = walk;
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| assert_eq!(dom.usage().tree_walkers(), 1))
        .unwrap();
    let mut session = host.stop().into_session().unwrap();
    assert_eq!(session.dom().usage().tree_walkers(), 0);
}

pub(super) fn other_unsigned_short_results_follow_each_navigation_algorithm() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_tree_walkers(8));
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const root = document.createElement('div');
                const branch = root.appendChild(document.createElement('span'));
                const leaf = branch.appendChild(document.createTextNode('nested'));
                const peer = root.appendChild(document.createTextNode('peer'));
                const filter = node => node === branch ? 65536 : NodeFilter.FILTER_ACCEPT;
                const walk = document.createTreeWalker(root, NodeFilter.SHOW_ALL, filter);
                if (walk.filter !== filter || walk.firstChild() !== peer) return 'child axis requires exact skip';
                walk.currentNode = root;
                return walk.nextNode() === leaf && walk.currentNode === leaf ? 'ok' : 'next node descends unless rejected';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn filter_errors_use_the_callbacks_associated_realm() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_tree_walkers(1));
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    let (filter, error_constructor) = {
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
        for (name, root) in [
            ("foreignWalkerFilter", filter),
            ("foreignWalkerError", error_constructor),
        ] {
            let key: Vec<_> = name.encode_utf16().collect();
            let input = scope.import(&root)?;
            scope.set(&global, PropertyKey::String(&key), &input)?;
        }
        let result = evaluate(
            scope,
            r#"
            (() => {
                const root = document.createElement('div');
                const child = root.appendChild(document.createTextNode('text'));
                const walk = document.createTreeWalker(root, undefined, foreignWalkerFilter);
                try { walk.nextNode(); return 'noncallable callback accepted'; }
                catch (error) {
                    if (!(error instanceof foreignWalkerError) || error instanceof TypeError ||
                        walk.currentNode !== root) return 'callback operation error realm';
                }
                foreignWalkerFilter.acceptNode = () => Symbol();
                try { walk.nextNode(); return 'invalid callback result accepted'; }
                catch (error) {
                    if (!(error instanceof foreignWalkerError) || error instanceof TypeError ||
                        walk.currentNode !== root) return 'callback result conversion realm';
                }
                foreignWalkerFilter.acceptNode = node => node === child ? 1 : 3;
                return walk.nextNode() === child ? 'ok' : 'canonical callback output';
            })()
        "#,
        )?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    })
    .unwrap();
}

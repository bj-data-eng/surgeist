//! A cached tree root answers what a full climb along the parent edges does,
//! after every kind of move. The expected roots come from how each tree was
//! built, not from the cache.
use super::*;
use crate::construction::effects::tests::{graph, named, request};
use std::num::NonZeroU32;

/// Position of the document, the root of the tree the source builds.
const DOCUMENT: usize = 0;

const SOURCE: &str =
    "<body><div><p><span></span></p></div><section><i></i></section><aside></aside></body>";

fn root_of(session: &mut ParserArenaSession, c: &mut ParseControl, node: &ParserNode) -> usize {
    let storage = session.active_mut().unwrap();
    let view = &storage.construction;
    storage.payloads.tree_root(c, view, node.index).unwrap()
}

fn associated(session: &mut ParserArenaSession, c: &mut ParseControl, node: &ParserNode) -> bool {
    session
        .active_mut()
        .unwrap()
        .associated_document_parent(c, node.index)
        .unwrap()
}

#[test]
fn moves_and_detaches_keep_every_cached_root_exact() {
    let request = request(SOURCE);
    let (mut c, mut session, proof, nodes) = graph(&request);
    let [div, p, span, section, i, aside] =
        ["div", "p", "span", "section", "i", "aside"].map(|name| named(&nodes, name, 0));
    for node in [&div, &p, &span, &section, &i, &aside] {
        assert_eq!(root_of(&mut session, &mut c, node), DOCUMENT);
    }
    // A subtree moves inside the tree: nothing it holds changes root.
    session
        .insert(&mut c, &section, &p, None, Some(&proof))
        .unwrap();
    assert_eq!(root_of(&mut session, &mut c, &span), DOCUMENT);
    assert_eq!(root_of(&mut session, &mut c, &p), DOCUMENT);
    // A subtree with children leaves the tree: everything in it changes root.
    session.remove(&mut c, &section, Some(&proof)).unwrap();
    for node in [&section, &i, &p, &span] {
        assert_eq!(root_of(&mut session, &mut c, node), section.index);
    }
    assert_eq!(root_of(&mut session, &mut c, &div), DOCUMENT);
    assert!(!associated(&mut session, &mut c, &span));
    // It joins another tree: everything in it changes back.
    session
        .insert(&mut c, &aside, &section, None, Some(&proof))
        .unwrap();
    for node in [&section, &i, &p, &span] {
        assert_eq!(root_of(&mut session, &mut c, node), DOCUMENT);
    }
    // A record with no children leaves and joins alone.
    session.remove(&mut c, &span, Some(&proof)).unwrap();
    assert_eq!(root_of(&mut session, &mut c, &span), span.index);
    session
        .insert(&mut c, &i, &span, None, Some(&proof))
        .unwrap();
    assert_eq!(root_of(&mut session, &mut c, &span), DOCUMENT);
    // A detached subtree built first and attached last.
    session.remove(&mut c, &div, Some(&proof)).unwrap();
    session.remove(&mut c, &aside, Some(&proof)).unwrap();
    session
        .insert(&mut c, &aside, &div, None, Some(&proof))
        .unwrap();
    for (name, node) in [
        ("aside", &aside),
        ("div", &div),
        ("section", &section),
        ("span", &span),
    ] {
        assert_eq!(root_of(&mut session, &mut c, node), aside.index, "{name}");
    }
    session.release(&mut c).unwrap();
}

#[test]
fn a_cache_that_cannot_name_another_generation_is_off_and_stays_exact() {
    let request = request(SOURCE);
    let (mut c, mut session, proof, nodes) = graph(&request);
    let [section, i, span, aside] = ["section", "i", "span", "aside"].map(|n| named(&nodes, n, 0));
    for node in [&section, &i, &span, &aside] {
        assert_eq!(root_of(&mut session, &mut c, node), DOCUMENT);
    }
    // The last generation: the next move that must start a new one switches the
    // cache off.
    session.active_mut().unwrap().payloads.tree_stamp = NonZeroU32::new(u32::MAX);
    session.remove(&mut c, &section, Some(&proof)).unwrap();
    assert_eq!(session.active().unwrap().payloads.tree_stamp, None);
    for node in [&section, &i] {
        assert_eq!(root_of(&mut session, &mut c, node), section.index);
    }
    session
        .insert(&mut c, &aside, &section, None, Some(&proof))
        .unwrap();
    for node in [&section, &i] {
        assert_eq!(root_of(&mut session, &mut c, node), DOCUMENT);
    }
    session.release(&mut c).unwrap();
}

#[test]
fn a_refused_insertion_ends_its_announcement_and_leaves_exact_roots() {
    // A tree no deeper than the limit allows. The div, holding the span, is
    // detached and its roots are cached, then its move under the deepest
    // element is refused for the depth it would add, after the announcement
    // that its subtree may change root. The refusal latches the stop, so the
    // cache is read without a charge: no cell may hold a root other than the
    // one a full climb finds, the generation the announcement started leaves
    // none holding any, and the announcement was counted at both ends.
    let limits = super::super::tests::limits()
        .into_builder()
        .tree_depth(5)
        .finish()
        .unwrap();
    let request = crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(SOURCE), limits);
    let (mut c, mut session, proof, nodes) = graph(&request);
    let [div, span, i] = ["div", "span", "i"].map(|name| named(&nodes, name, 0));
    session.remove(&mut c, &div, Some(&proof)).unwrap();
    for node in [&div, &span] {
        assert_eq!(root_of(&mut session, &mut c, node), div.index);
    }
    let storage = session.active_mut().unwrap();
    let before = storage.payloads.topology_changes.unwrap();
    let refused = storage.insert(&mut c, &i, &div, None, InsertionCause::NonParser);
    assert!(matches!(
        refused,
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::TreeDepth,
            ..
        }))
    ));
    assert_eq!(storage.payloads.topology_changes, Some(before + 2));
    let view = &storage.construction;
    for node in [&div, &span] {
        assert_eq!(
            storage.payloads.cached_root(node.index),
            None,
            "record {}",
            node.index
        );
        assert_eq!(
            PayloadStorage::full_climb_root(view, node.index),
            Some(div.index)
        );
    }
    session.release(&mut c).unwrap();
}

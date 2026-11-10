use super::*;
use surgeist_html::construction::{ConstructionNodeKind, HtmlDocumentMode};

#[test]
fn document_mode_update_rejects_fragment_and_work_stop_before_the_real_mode_write() {
    let mut c = control();
    let mut fragment = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    assert_eq!(
        fragment.document_accepts_doctype(&mut c),
        Err(SessionError::NotDocument)
    );
    assert_eq!(
        fragment.set_document_mode(&mut c, HtmlDocumentMode::Quirks),
        Err(SessionError::NotDocument)
    );
    assert!(c.stop().is_none());
    fragment.release(&mut c).unwrap();
    let mut document =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
    c.charge(
        HtmlParseWorkCategory::StateStep,
        limits().work_credits() - used - 1,
    )
    .unwrap();
    assert!(matches!(
        document.set_document_mode(&mut c, HtmlDocumentMode::Quirks),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        }))
    ));
    assert!(matches!(
        document
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .expect("retained construction position")
            .kind(),
        ConstructionNodeKind::Document(HtmlDocumentMode::NoQuirks)
    ));
    document.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn element_guard_reads_document_children_without_restricting_generic_moves() {
    let mut c = control();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let first = session.allocate_element(&mut c, html("html")).unwrap();
    let second = session.allocate_element(&mut c, html("body")).unwrap();
    let visits = c.usage().work(HtmlParseWorkCategory::RecordVisit);
    assert_eq!(
        session.element_insertion_ignored(&mut c, &first, &root),
        Ok(None)
    );
    // Sixteen readiness (two added Range phase reads) visits + two checked handles, element kind,
    // ancestry root and target kind.
    assert_eq!(
        c.usage().work(HtmlParseWorkCategory::RecordVisit) - visits,
        23
    );
    session.insert(&mut c, &root, &first, None, None).unwrap();
    let visits = c.usage().work(HtmlParseWorkCategory::RecordVisit);
    assert_eq!(
        session.element_insertion_ignored(&mut c, &second, &root),
        Ok(Some(ElementInsertionIgnored::DocumentAlreadyHasElement))
    );
    // The additional real child-index and referenced-kind reads cost two.
    assert_eq!(
        c.usage().work(HtmlParseWorkCategory::RecordVisit) - visits,
        25
    );
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>(),
        [first.index]
    );
    assert!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(second.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position())
            .is_none()
    );
    // The guard is HTML placement policy; generic native moves are unchanged.
    session.insert(&mut c, &root, &second, None, None).unwrap();
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>(),
        [first.index, second.index]
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn element_guard_uses_actual_template_and_ordinary_ancestry_and_includes_self() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let template = session.allocate_element(&mut c, html("template")).unwrap();
    let contents = session
        .template_contents(&mut c, &template)
        .unwrap()
        .unwrap();
    let child = session.allocate_element(&mut c, html("div")).unwrap();
    session
        .insert(&mut c, &contents, &child, None, None)
        .unwrap();
    assert!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(contents.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position())
            .is_none()
    );
    let visits = c.usage().work(HtmlParseWorkCategory::RecordVisit);
    let providers = c.usage().work(HtmlParseWorkCategory::ProviderCall);
    assert_eq!(
        session.element_insertion_ignored(&mut c, &template, &child),
        Ok(Some(
            ElementInsertionIgnored::HostIncludingInclusiveAncestor
        ))
    );
    // Sixteen readiness (two Radio and two Range phase reads) visits + two handles + element kind +
    // the shallow-element test, which fails here because the template's
    // contents hold the child, + child/contents/template ancestors.
    assert_eq!(
        c.usage().work(HtmlParseWorkCategory::RecordVisit) - visits,
        25
    );
    assert_eq!(
        c.usage().work(HtmlParseWorkCategory::ProviderCall) - providers,
        1
    );
    assert_eq!(
        session.element_insertion_ignored(&mut c, &template, &template),
        Ok(Some(
            ElementInsertionIgnored::HostIncludingInclusiveAncestor
        ))
    );
    assert_eq!(
        session.element_insertion_ignored(&mut c, &child, &root),
        Ok(Some(ElementInsertionIgnored::ParentAlreadyPresent))
    );
    assert!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(template.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position())
            .is_none()
    );
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(contents.index)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>(),
        [child.index]
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn element_guard_on_an_element_with_nothing_below_it_finds_only_itself_and_its_contents() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let template = session.allocate_element(&mut c, html("template")).unwrap();
    let contents = session
        .template_contents(&mut c, &template)
        .unwrap()
        .unwrap();
    let div = session.allocate_element(&mut c, html("div")).unwrap();
    let ancestor = Ok(Some(
        ElementInsertionIgnored::HostIncludingInclusiveAncestor,
    ));
    // The element itself, and a template's own empty contents.
    assert_eq!(
        session.element_insertion_ignored(&mut c, &div, &div),
        ancestor
    );
    assert_eq!(
        session.element_insertion_ignored(&mut c, &template, &template),
        ancestor
    );
    assert_eq!(
        session.element_insertion_ignored(&mut c, &template, &contents),
        ancestor
    );
    // Nothing else lies below them: neither the root, an unrelated element
    // nor another element's contents.
    let other = session.allocate_element(&mut c, html("template")).unwrap();
    let other_contents = session.template_contents(&mut c, &other).unwrap().unwrap();
    for (element, target) in [
        (&div, &root),
        (&div, &template),
        (&div, &contents),
        (&template, &root),
        (&template, &div),
        (&template, &other_contents),
    ] {
        assert_eq!(
            session.element_insertion_ignored(&mut c, element, target),
            Ok(None),
            "{} below {}",
            element.index,
            target.index
        );
    }
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn element_guard_work_stop_precedes_document_child_reads_and_preserves_topology() {
    let mut c = control();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let first = session.allocate_element(&mut c, html("html")).unwrap();
    let second = session.allocate_element(&mut c, html("div")).unwrap();
    session.insert(&mut c, &root, &first, None, None).unwrap();
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes);
    let edges = c.usage().resource(HtmlParseResource::ManagedEdges);
    let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
    c.charge(
        HtmlParseWorkCategory::StateStep,
        limits().work_credits() - used - 24,
    )
    .unwrap();
    let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
    assert!(matches!(
        session.element_insertion_ignored(&mut c, &second, &root),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            requested_delta: 2,
            ..
        }))
    ));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        used + 23
    );
    assert_eq!(c.usage().resource(HtmlParseResource::ManagedNodes), nodes);
    assert_eq!(c.usage().resource(HtmlParseResource::ManagedEdges), edges);
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>(),
        [first.index]
    );
    assert!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(second.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position())
            .is_none()
    );
    let stop = c.stop();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), stop);
}

#[test]
fn element_guard_rejects_foreign_handles_controls_and_cancellation_before_mutation() {
    for cancel in [false, true] {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let node = session.allocate_element(&mut c, html("div")).unwrap();
        if cancel {
            cancellation.cancel();
            assert_eq!(
                session.element_insertion_ignored(&mut c, &node, &root),
                Err(SessionError::Stop(HtmlParseStop::Cancelled))
            );
        } else {
            let mut other = control();
            let mut foreign = ParserArenaSession::new(&mut other, RootKind::Fragment).unwrap();
            let foreign_node = foreign.allocate_element(&mut other, html("div")).unwrap();
            assert_eq!(
                session.element_insertion_ignored(&mut c, &node, &foreign_node),
                Err(SessionError::ForeignNode)
            );
            assert!(c.stop().is_none());
            assert!(matches!(
                session.element_insertion_ignored(&mut other, &node, &root),
                Err(SessionError::Stop(HtmlParseStop::InternalInvariant(_)))
            ));
            foreign.release(&mut other).unwrap();
            storage_zero(&other);
        }
        assert!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(0)
                .expect("retained construction position")
                .child_count()
                == 0
        );
        assert!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(node.index)
                .expect("retained construction position")
                .parent()
                .map(|node| node.position())
                .is_none()
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

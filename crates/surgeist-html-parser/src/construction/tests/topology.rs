use super::*;
use surgeist_html::{
    construction::HtmlConstructionResult,
    construction::{ConstructionNodeKind, HtmlDocumentMode, HtmlFragmentOwner},
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

#[test]
fn batch_cycle_guard_preserves_earlier_sibling_before_later_ancestor_refusal() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let from = session.root(&mut c).unwrap();
    let earlier = session
        .allocate_element(&mut c, element(Namespace::none(), "a"))
        .unwrap();
    let branch = session
        .allocate_element(&mut c, element(Namespace::none(), "b"))
        .unwrap();
    let destination = session
        .allocate_element(&mut c, element(Namespace::none(), "c"))
        .unwrap();
    session.insert(&mut c, &from, &earlier, None, None).unwrap();
    session.insert(&mut c, &from, &branch, None, None).unwrap();
    session
        .insert(&mut c, &branch, &destination, None, None)
        .unwrap();
    let before = session
        .active()
        .unwrap()
        .construction
        .test_storage()
        .usage();
    assert_eq!(
        session
            .reparent_children(&mut c, &from, &destination, None)
            .err(),
        Some(SessionError::Cycle)
    );
    let storage = session.active().unwrap();
    assert_eq!(
        storage
            .construction
            .node_at(from.index)
            .unwrap()
            .children()
            .map(surgeist_html::bounded::construction::HtmlConstructionNode::position)
            .collect::<Vec<_>>(),
        [earlier.index, branch.index]
    );
    assert_eq!(
        storage
            .construction
            .node_at(earlier.index)
            .unwrap()
            .parent()
            .unwrap()
            .position(),
        from.index
    );
    assert_eq!(
        storage
            .construction
            .node_at(destination.index)
            .unwrap()
            .parent()
            .unwrap()
            .position(),
        branch.index
    );
    assert_eq!(storage.construction.test_storage().usage(), before);
    assert!(c.stop().is_none());
    session.release(&mut c).unwrap();
    assert_eq!(c.usage().actual_capacity_bytes, 0);
}

#[test]
fn detached_allocations_and_ordered_moves_keep_permanent_nodes_and_exact_views() {
    let mut control = control();
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let a = session.allocate_element(&mut control, html("a")).unwrap();
    let b = session.allocate_element(&mut control, html("b")).unwrap();
    let c = session.allocate_element(&mut control, html("c")).unwrap();
    for node in [&a, &b, &c] {
        session
            .insert(&mut control, &root, node, None, None)
            .unwrap();
    }
    session
        .insert(&mut control, &root, &a, Some(&c), None)
        .unwrap();
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
        [b.index, a.index, c.index]
    );
    session
        .insert(&mut control, &root, &a, Some(&a), None)
        .unwrap();
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
        [b.index, a.index, c.index]
    );
    session
        .insert(&mut control, &root, &c, Some(&b), None)
        .unwrap();
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
        [c.index, b.index, a.index]
    );
    session.remove(&mut control, &b, None).unwrap();
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        4
    );
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedEdges)
            .current,
        2
    );
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedEdges)
            .high_water,
        3
    );
    let mut output = frozen(session, &mut control);
    let fragment = output.fragment().unwrap();
    assert_eq!(locals(fragment), ["c", "a"]);
    assert_eq!(fragment.usage().nodes(), 4);
    assert_eq!(fragment.usage().edges(), 2);
    let detached_id = HtmlConstructionResult::Fragment(fragment.clone())
        .node_at(b.index)
        .unwrap()
        .as_node()
        .id();
    assert!(fragment.node(&detached_id).unwrap().parent().is_none());
    assert_eq!(
        test_owning_depth(&HtmlConstructionResult::Fragment(fragment.clone()), b.index),
        0
    );
    assert!(output.release(&mut control).is_ok());
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        4
    );
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedEdges)
            .current,
        0
    );
    assert!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water
            > 0
    );
}

#[test]
fn paired_template_contents_are_separate_and_association_cycles_are_rejected() {
    let mut control = control();
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let template = session
        .allocate_element(&mut control, html("template"))
        .unwrap();
    let contents = session
        .template_contents(&mut control, &template)
        .unwrap()
        .unwrap();
    let child = session.allocate_element(&mut control, html("x")).unwrap();
    session
        .insert(&mut control, &root, &template, None, None)
        .unwrap();
    session
        .insert(&mut control, &contents, &child, None, None)
        .unwrap();
    assert_eq!(
        session.insert(&mut control, &child, &template, None, None),
        Err(SessionError::Cycle)
    );
    assert_eq!(
        session.insert(&mut control, &template, &child, None, None),
        Err(SessionError::TemplateContentsRequired)
    );
    assert_eq!(
        session.insert(&mut control, &root, &contents, None, None),
        Err(SessionError::AssociationMove)
    );
    let mut output = frozen(session, &mut control);
    let fragment = output.fragment().unwrap();
    assert_eq!(fragment.usage().nodes(), 4);
    assert_eq!(fragment.usage().edges(), 3);
    assert_eq!(fragment.usage().depth(), 3);
    let template_id = fragment.root().children().next().unwrap();
    let template_node = fragment.node(&template_id).unwrap();
    assert_eq!(template_node.children().len(), 0);
    let contents_id = template_node.template_contents().unwrap();
    assert_eq!(
        fragment.node(&contents_id).unwrap().fragment_owner(),
        Some(HtmlFragmentOwner::Template(template_id))
    );
    assert!(fragment.node(&contents_id).unwrap().parent().is_none());
    assert_eq!(
        fragment
            .fragment(&contents_id)
            .unwrap()
            .unwrap()
            .root()
            .children()
            .len(),
        1
    );
    assert!(output.release(&mut control).is_ok());
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn foreign_template_and_namespaced_attributes_keep_preexisting_exact_slots() {
    let mut control = control();
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let payload = PreexistingElement::new(
        QualifiedName::new(
            Namespace::none(),
            Some(DocumentString::from("p")),
            DocumentString::from_utf16(vec![0xD800, 32]),
        ),
        OrderedAttributes::new(vec![Attribute::new(
            QualifiedName::new(Namespace::none(), None, "v"),
            DocumentString::from_utf16(vec![0xDC00]),
        )])
        .unwrap(),
    );
    let backing = payload.name().local_name().code_units().as_ptr();
    let raw = session.allocate_element(&mut control, payload).unwrap();
    let foreign = session
        .allocate_element(&mut control, element(Namespace::svg(), "template"))
        .unwrap();
    assert!(
        session
            .template_contents(&mut control, &foreign)
            .unwrap()
            .is_none()
    );
    session
        .insert(&mut control, &root, &raw, None, None)
        .unwrap();
    session
        .insert(&mut control, &root, &foreign, None, None)
        .unwrap();
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        1
    );
    // 2 local + 1 prefix + 1 attribute local + 1 attribute value; SVG URI
    // http://www.w3.org/2000/svg (26) plus template (8).
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        39
    );
    let mut output = frozen(session, &mut control);
    let fragment = output.fragment().unwrap();
    let id = fragment.root().children().next().unwrap();
    let surgeist_html::construction::HtmlNodeKind::Element(element) =
        fragment.node(&id).unwrap().kind()
    else {
        panic!("expected element")
    };
    assert_eq!(element.name().local_name().code_units(), [0xD800, 32]);
    assert_eq!(element.name().local_name().code_units().as_ptr(), backing);
    assert_eq!(
        element.attributes().get(0).unwrap().value().code_units(),
        [0xDC00]
    );
    assert!(output.release(&mut control).is_ok());
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn node_and_depth_cap_failures_leave_topology_unmodified_and_cleanup_releases_capacity() {
    let mut selected = limits();
    selected = selected.into_builder().managed_nodes(2).finish().unwrap();
    let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    assert!(matches!(
        session.allocate_element(&mut control, html("template")),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedNodes,
            ..
        }))
    ));
    assert_eq!(
        session.active().unwrap().construction.test_storage().len(),
        1
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
        []
    );
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        1
    );
    assert!(session.release(&mut control).is_ok());
    assert_eq!(control.usage().actual_capacity_bytes, 0);

    let mut selected = limits();
    selected = selected.into_builder().tree_depth(2).finish().unwrap();
    let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let template = session
        .allocate_element(&mut control, html("template"))
        .unwrap();
    let contents = session
        .template_contents(&mut control, &template)
        .unwrap()
        .unwrap();
    let child = session.allocate_element(&mut control, html("x")).unwrap();
    session
        .insert(&mut control, &contents, &child, None, None)
        .unwrap();
    assert!(matches!(
        session.insert(&mut control, &root, &template, None, None),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::TreeDepth,
            ..
        }))
    ));
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
    assert!(session.release(&mut control).is_ok());
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn foreign_handles_controls_roots_and_wrong_siblings_reject_before_topology_change() {
    let mut owner = control();
    let mut foreign = control();
    let mut session = ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap();
    let mut other = ParserArenaSession::new(&mut foreign, RootKind::Fragment).unwrap();
    let root = session.root(&mut owner).unwrap();
    let foreign_root = other.root(&mut foreign).unwrap();
    let a = session.allocate_element(&mut owner, html("a")).unwrap();
    let b = session.allocate_element(&mut owner, html("b")).unwrap();
    assert_eq!(
        session.insert(&mut owner, &root, &foreign_root, None, None),
        Err(SessionError::ForeignNode)
    );
    assert_eq!(
        session.insert(&mut owner, &a, &root, None, None),
        Err(SessionError::RootMove)
    );
    assert_eq!(
        session.insert(&mut owner, &root, &a, Some(&b), None),
        Err(SessionError::WrongSibling)
    );
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
    let before = foreign.usage().clone();
    assert!(matches!(
        session.insert(&mut foreign, &root, &a, None, None),
        Err(SessionError::Stop(_))
    ));
    assert_eq!(
        foreign.usage().actual_capacity_bytes,
        before.actual_capacity_bytes
    );
    assert_eq!(
        foreign.usage().resource(HtmlParseResource::WorkCredits),
        before.resource(HtmlParseResource::WorkCredits)
    );
    assert!(other.release(&mut foreign).is_ok());
    assert!(session.release(&mut owner).is_ok());
    assert_eq!(owner.usage().actual_capacity_bytes, 0);
    assert_eq!(foreign.usage().actual_capacity_bytes, 0);
}

#[test]
fn failed_freeze_and_foreign_cleanup_retain_owned_tickets_for_recovery() {
    let mut owner = control();
    let mut foreign = control();
    let mut session = ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap();
    assert!(matches!(
        session.release(&mut foreign),
        Err(SessionError::Stop(_))
    ));
    let cancellation = HtmlParseCancellation::new();
    let mut stopped = ParseControl::new(limits(), cancellation.clone());
    let mut candidate = ParserArenaSession::new(&mut stopped, RootKind::Fragment).unwrap();
    candidate.allocate_element(&mut stopped, html("x")).unwrap();
    cancellation.cancel();
    assert!(matches!(
        candidate.freeze(&mut stopped),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    assert!(candidate.release(&mut stopped).is_ok());
    assert_eq!(stopped.usage().actual_capacity_bytes, 0);
    assert_eq!(
        stopped
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        2
    );
    assert!(session.release(&mut owner).is_ok());
    assert_eq!(owner.usage().actual_capacity_bytes, 0);
}

#[test]
fn frozen_document_uses_existing_mode_and_old_arena_survives_later_session() {
    let mut first = control();
    let mut session = ParserArenaSession::new(
        &mut first,
        RootKind::Document(HtmlDocumentMode::LimitedQuirks),
    )
    .unwrap();
    let root = session.root(&mut first).unwrap();
    let node = session.allocate_element(&mut first, html("html")).unwrap();
    session
        .insert(&mut first, &root, &node, None, None)
        .unwrap();
    let mut output = frozen(session, &mut first);
    let old = output.document().unwrap().clone();
    let old_id = old.root().children().next().unwrap();
    assert_eq!(old.mode(), HtmlDocumentMode::LimitedQuirks);
    let mut second = control();
    let mut later = ParserArenaSession::new(&mut second, RootKind::Fragment).unwrap();
    later.allocate_element(&mut second, html("new")).unwrap();
    let mut later = frozen(later, &mut second);
    assert!(later.fragment().unwrap().node(&old_id).is_err());
    assert_eq!(old.root().children().next().unwrap(), old_id);
    assert!(matches!(
        output.release(&mut first),
        Err(SessionError::Stop(_))
    ));
    drop(old);
    assert!(output.release(&mut first).is_ok());
    assert!(later.release(&mut second).is_ok());
    assert_eq!(first.usage().actual_capacity_bytes, 0);
    assert_eq!(second.usage().actual_capacity_bytes, 0);
}

#[test]
fn deep_topology_freezes_without_recursive_tree_or_child_depth_cache() {
    let mut control = control();
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let mut parent = session.root(&mut control).unwrap();
    for _ in 0..48 {
        let child = session.allocate_element(&mut control, html("x")).unwrap();
        session
            .insert(&mut control, &parent, &child, None, None)
            .unwrap();
        parent = child;
    }
    let mut output = frozen(session, &mut control);
    assert_eq!(output.fragment().unwrap().usage().depth(), 48);
    assert_eq!(
        test_owning_depth(&output.storage.as_ref().unwrap().view, parent.index),
        48
    );
    assert!(output.release(&mut control).is_ok());
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn cross_parent_move_changes_both_orders_and_backlinks_without_new_nodes_or_edges() {
    let mut control = control();
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let a = session.allocate_element(&mut control, html("a")).unwrap();
    let b = session.allocate_element(&mut control, html("b")).unwrap();
    let c = session.allocate_element(&mut control, html("c")).unwrap();
    let d = session.allocate_element(&mut control, html("d")).unwrap();
    session.insert(&mut control, &root, &a, None, None).unwrap();
    session.insert(&mut control, &root, &b, None, None).unwrap();
    session.insert(&mut control, &a, &c, None, None).unwrap();
    session.insert(&mut control, &b, &d, None, None).unwrap();
    session
        .insert(&mut control, &b, &c, Some(&d), None)
        .unwrap();
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        5
    );
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedEdges)
            .current,
        4
    );
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedEdges)
            .high_water,
        4
    );
    let mut output = frozen(session, &mut control);
    let fragment = output.fragment().unwrap();
    let a_id = HtmlConstructionResult::Fragment(fragment.clone())
        .node_at(a.index)
        .unwrap()
        .as_node()
        .id();
    let b_id = HtmlConstructionResult::Fragment(fragment.clone())
        .node_at(b.index)
        .unwrap()
        .as_node()
        .id();
    let c_id = HtmlConstructionResult::Fragment(fragment.clone())
        .node_at(c.index)
        .unwrap()
        .as_node()
        .id();
    let d_id = HtmlConstructionResult::Fragment(fragment.clone())
        .node_at(d.index)
        .unwrap()
        .as_node()
        .id();
    assert_eq!(fragment.node(&a_id).unwrap().children().len(), 0);
    assert_eq!(
        fragment.node(&b_id).unwrap().children().collect::<Vec<_>>(),
        [c_id, d_id]
    );
    assert_eq!(fragment.node(&c_id).unwrap().parent(), Some(b_id));
    assert_eq!(fragment.usage().depth(), 2);
    assert!(output.release(&mut control).is_ok());
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn edge_preflight_and_zero_physical_capacity_reject_before_actual_record_admission() {
    let mut selected = limits();
    selected = selected.into_builder().managed_edges(0).finish().unwrap();
    let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let bytes_before = control.usage().actual_capacity_bytes;
    assert!(matches!(
        session.allocate_element(&mut control, html("template")),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedEdges,
            ..
        }))
    ));
    assert_eq!(
        session.active().unwrap().construction.test_storage().len(),
        1
    );
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        1
    );
    assert_eq!(control.usage().actual_capacity_bytes, bytes_before);
    assert!(session.release(&mut control).is_ok());
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    let mut selected = limits();
    selected = selected
        .into_builder()
        .managed_capacity_bytes(0)
        .finish()
        .unwrap();
    let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
    assert!(matches!(
        ParserArenaSession::new(&mut control, RootKind::Fragment),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        }))
    ));
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        0
    );
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

/// The address of the element name that the build view reports for record `position`. The name
/// lives in the record, so the address changes when the records are copied to another allocation.
fn build_name_address(session: &ParserArenaSession, position: usize) -> *const QualifiedName {
    let ConstructionNodeKind::Element(element) = session
        .active()
        .unwrap()
        .construction
        .node_at(position)
        .expect("retained construction position")
        .kind()
    else {
        panic!("element record");
    };
    std::ptr::from_ref(element.name())
}

#[test]
fn successful_transfers_consume_each_owner_once_without_failure_time_allocation() {
    let mut control = control();
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let node = session.allocate_element(&mut control, html("x")).unwrap();
    session
        .insert(&mut control, &root, &node, None, None)
        .unwrap();
    let records_address = build_name_address(&session, node.index);
    let mut output = session
        .freeze(&mut control)
        .unwrap_or_else(|error| panic!("freeze failed: {error:?}"));
    // Publication moves the records without copying them: the published view names the very
    // element name the build view named.
    let published = output.fragment().unwrap();
    let child = published.root().children().next().unwrap();
    let surgeist_html::construction::HtmlNodeKind::Element(element) =
        published.node(&child).unwrap().kind()
    else {
        panic!("element record");
    };
    assert_eq!(std::ptr::from_ref(element.name()), records_address);
    let before = control.usage().clone();
    assert!(matches!(
        session.root(&mut control),
        Err(SessionError::Consumed)
    ));
    assert!(matches!(
        session.freeze(&mut control),
        Err(SessionError::Consumed)
    ));
    assert_eq!(session.release(&mut control), Err(SessionError::Consumed));
    assert_eq!(control.usage(), before);
    output.release(&mut control).unwrap();
    assert!(output.fragment().is_none());
    assert!(output.document().is_none());
    let before = control.usage().clone();
    assert_eq!(output.release(&mut control), Err(SessionError::Consumed));
    assert_eq!(control.usage(), before);
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn foreign_freeze_and_frozen_cleanup_preserve_the_borrowed_original_owner() {
    let mut owner = control();
    let mut foreign = control();
    let mut session = ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap();
    let root = session.root(&mut owner).unwrap();
    let node = session.allocate_element(&mut owner, html("x")).unwrap();
    session
        .insert(&mut owner, &root, &node, None, None)
        .unwrap();
    let before = owner.usage().clone();
    assert!(matches!(
        session.freeze(&mut foreign),
        Err(SessionError::Stop(_))
    ));
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
        [node.index]
    );
    assert_eq!(owner.usage(), before);
    let mut output = session
        .freeze(&mut owner)
        .unwrap_or_else(|error| panic!("freeze failed: {error:?}"));
    let before = owner.usage().clone();
    assert!(matches!(
        output.release(&mut foreign),
        Err(SessionError::Stop(_))
    ));
    assert_eq!(locals(output.fragment().unwrap()), ["x"]);
    assert_eq!(owner.usage(), before);
    output.release(&mut owner).unwrap();
    assert_eq!(owner.usage().actual_capacity_bytes, 0);
}

#[test]
fn final_shared_payload_cap_stop_restores_actual_vectors_and_all_tickets() {
    let mut selected = limits();
    // A root and one element child own two actual record slots and their child-vector tickets.
    let mut probe_control = control();
    let mut probe_session =
        ParserArenaSession::new(&mut probe_control, RootKind::Fragment).unwrap();
    let probe_root = probe_session.root(&mut probe_control).unwrap();
    let probe_node = probe_session
        .allocate_element(&mut probe_control, html("x"))
        .unwrap();
    probe_session
        .insert(&mut probe_control, &probe_root, &probe_node, None, None)
        .unwrap();
    let root_capacity = probe_control.usage().actual_capacity_bytes as usize;
    let mut probe_output = frozen(probe_session, &mut probe_control);
    let wrapper_bytes = probe_output
        .storage
        .as_ref()
        .unwrap()
        .lease
        .as_ref()
        .unwrap()
        .allocation()
        .payload_bytes();
    probe_output.release(&mut probe_control).unwrap();
    storage_zero(&probe_control);
    selected = selected
        .into_builder()
        .managed_capacity_bytes(root_capacity as u64)
        .finish()
        .unwrap();
    let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let node = session.allocate_element(&mut control, html("x")).unwrap();
    session
        .insert(&mut control, &root, &node, None, None)
        .unwrap();
    let records_address = build_name_address(&session, node.index);
    assert!(
        matches!(session.freeze(&mut control), Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
        resource: HtmlParseResource::ManagedCapacityBytes,
        current,
        requested_delta,
        ..
    })) if current == root_capacity as u64 && requested_delta == wrapper_bytes as u64)
    );
    assert_eq!(build_name_address(&session, node.index), records_address);
    assert_eq!(
        session.active().unwrap().construction.test_storage().len(),
        2
    );
    assert_eq!(control.usage().actual_capacity_bytes, root_capacity as u64);
    session.release(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        2
    );
}

#[test]
fn parser_nodes_are_copy_values_that_name_the_same_record_after_a_copy() {
    fn assert_copy<T: Copy>() {}
    assert_copy::<ParserNode>();
    // A builder handle (storage serial and index, 16 bytes) and the parser's dense join index.
    #[cfg(target_pointer_width = "64")]
    assert_eq!(std::mem::size_of::<ParserNode>(), 24);
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let copy = root;
    let child = session
        .allocate_element(&mut c, element(Namespace::none(), "x"))
        .unwrap();
    // Both the original and the copy are usable after the copy was made.
    session.insert(&mut c, &root, &child, None, None).unwrap();
    session.remove(&mut c, &child, None).unwrap();
    session.insert(&mut c, &copy, &child, None, None).unwrap();
    assert_eq!(root.index, copy.index);
}

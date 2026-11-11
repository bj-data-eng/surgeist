mod depth;
mod focus;
mod order;
mod text;
mod views;
use crate::html::support::{Cause, Direct, cause, granted, has_cause};
use std::sync::Arc;
use surgeist_html::{
    bounded::construction::{HtmlConstructionBank, HtmlConstructionNode, HtmlConstructionStorage},
    construction::{
        ConstructionNodeKind, ForeignHtmlResult, HtmlBuilderNode, HtmlConstructionError,
        HtmlConstructionResult, HtmlDocumentMode, HtmlFragmentOwner, HtmlNodeKind,
        HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment, HtmlTemplateContentsKind,
    },
    values::{Namespace, OrderedAttributes, QualifiedName},
};

/// A record a test creates through the direct creation method of its kind.
enum Make {
    Element {
        name: QualifiedName,
        attributes: OrderedAttributes,
        link_flags: Option<surgeist_html::elements::link::HtmlLinkFlags>,
    },
    Fragment,
    Document {
        mode: HtmlDocumentMode,
    },
    ShadowContents {
        properties: HtmlShadowRootProperties,
    },
    Text(Vec<u16>),
    Comment(Vec<u16>),
    ProcessingInstruction(Vec<u16>, Vec<u16>),
    Doctype(Vec<u16>, Option<Vec<u16>>, Option<Vec<u16>>),
}

/// A change a test makes through the direct method of its kind.
enum Op {
    Insert {
        node: HtmlBuilderNode,
        parent: HtmlBuilderNode,
        before: Option<HtmlBuilderNode>,
    },
    Detach {
        node: HtmlBuilderNode,
    },
    AttachShadow {
        host: HtmlBuilderNode,
        contents: HtmlBuilderNode,
    },
    AttachTemplateShadow {
        template: HtmlBuilderNode,
        host: HtmlBuilderNode,
        contents: HtmlBuilderNode,
    },
    SetTemplateShadowContents {
        template: HtmlBuilderNode,
        contents: HtmlBuilderNode,
    },
    SetDocumentMode {
        document: HtmlBuilderNode,
        mode: HtmlDocumentMode,
    },
    ReserveRecords {
        additional: usize,
    },
}

fn storage() -> HtmlConstructionStorage {
    granted(HtmlConstructionStorage::new(&mut Direct::default())).unwrap()
}
fn try_create(
    storage: &mut HtmlConstructionStorage,
    make: Make,
    admission: &mut Direct,
) -> Result<HtmlBuilderNode, HtmlConstructionError> {
    granted(match make {
        Make::Element {
            name,
            attributes,
            link_flags,
        } => storage.create_element(&name, attributes, link_flags, admission),
        Make::Fragment => storage.create_fragment(admission),
        Make::Document { mode } => storage.create_document(mode, admission),
        Make::ShadowContents { properties } => {
            storage.create_shadow_contents(properties, admission)
        }
        Make::Text(units) => storage.create_text(&units, admission),
        Make::Comment(units) => storage.create_comment(&units, admission),
        Make::ProcessingInstruction(target, data) => {
            storage.create_processing_instruction(&target, &data, admission)
        }
        Make::Doctype(name, public, system) => {
            storage.create_doctype(&name, public.as_deref(), system.as_deref(), admission)
        }
    })
}
fn try_change(
    storage: &mut HtmlConstructionStorage,
    op: Op,
    admission: &mut Direct,
) -> Result<(), HtmlConstructionError> {
    granted(match op {
        Op::Insert {
            node,
            parent,
            before,
        } => storage.insert(&node, &parent, before.as_ref(), admission),
        Op::Detach { node } => storage.detach(&node, admission),
        Op::AttachShadow { host, contents } => storage.attach_shadow(&host, &contents, admission),
        Op::AttachTemplateShadow {
            template,
            host,
            contents,
        } => storage.attach_template_shadow(&template, &host, &contents, admission),
        Op::SetTemplateShadowContents { template, contents } => {
            storage.set_template_shadow_contents(&template, &contents, admission)
        }
        Op::SetDocumentMode { document, mode } => {
            storage.set_document_mode(&document, mode, admission)
        }
        Op::ReserveRecords { additional } => storage.reserve_records(additional, admission),
    })
}
fn create(
    mut storage: HtmlConstructionStorage,
    make: Make,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let created = try_create(&mut storage, make, &mut Direct::default()).unwrap();
    (storage, created)
}
fn change(
    mut storage: HtmlConstructionStorage,
    op: Op,
) -> (HtmlConstructionStorage, Option<HtmlBuilderNode>) {
    try_change(&mut storage, op, &mut Direct::default()).unwrap();
    (storage, None)
}
fn element(local: &str) -> Make {
    Make::Element {
        name: QualifiedName::new(Namespace::html(), None, local),
        attributes: OrderedAttributes::default(),
        link_flags: None,
    }
}
fn fragment(storage: HtmlConstructionStorage) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(storage, Make::Fragment)
}
fn properties() -> HtmlShadowRootProperties {
    HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Closed,
        HtmlSlotAssignment::Manual,
        true,
        true,
        true,
    )
}
fn publish(storage: HtmlConstructionStorage, root: HtmlBuilderNode) -> HtmlConstructionResult {
    match storage.publish(&root, &mut Direct::default()) {
        Ok(result) => result,
        Err(_) => panic!("publication refused"),
    }
}
/// Refuses `op` with the expected cause and checks that the usage and the record count are as
/// they were.
fn refuse(
    mut storage: HtmlConstructionStorage,
    op: Op,
    expected: Cause,
) -> HtmlConstructionStorage {
    let (usage, len) = (storage.usage(), storage.len());
    let error =
        try_change(&mut storage, op, &mut Direct::default()).expect_err("the operation is refused");
    assert_eq!(error, expected);
    assert_eq!((storage.usage(), storage.len()), (usage, len));
    storage
}

/// Appends `node` last under `parent` with an admission that doubles a full child order, and
/// returns the capacities it asked to be given.
fn append_amortized(
    storage: &mut HtmlConstructionStorage,
    node: &HtmlBuilderNode,
    parent: &HtmlBuilderNode,
) -> Vec<usize> {
    let mut admission = Direct {
        doubling: true,
        ..Direct::default()
    };
    try_change(
        storage,
        Op::Insert {
            node: *node,
            parent: *parent,
            before: None,
        },
        &mut admission,
    )
    .unwrap();
    admission
        .growths
        .iter()
        .map(|growth| {
            growth
                .minimum_capacity()
                .max(2 * growth.before().capacity())
        })
        .collect()
}

#[test]
fn amortized_appends_grow_the_child_order_geometrically_and_keep_their_order() {
    let (mut owner, root) = fragment(storage());
    let mut appended = Vec::new();
    let mut growth = Vec::new();
    for _ in 0..1024 {
        let (next, node) = create(owner, element("c"));
        owner = next;
        growth.extend(append_amortized(&mut owner, &node, &root));
        appended.push(node);
    }
    // One growth at each power of two, never one per append.
    assert_eq!(growth.len(), 11, "{growth:?}");
    assert!(growth.iter().sum::<usize>() <= 2 * appended.len());
    assert_eq!(
        owner
            .node(&root)
            .unwrap()
            .children()
            .map(HtmlConstructionNode::handle)
            .collect::<Vec<_>>(),
        appended
    );
    assert!(
        appended
            .iter()
            .all(|node| owner.node(node).unwrap().parent().unwrap().handle() == root)
    );
}

#[test]
fn append_with_spare_child_order_capacity_requests_no_growth() {
    let (mut owner, root) = fragment(storage());
    let mut requests = Vec::new();
    for _ in 0..4 {
        let (next, node) = create(owner, element("c"));
        owner = next;
        requests.push(append_amortized(&mut owner, &node, &root));
    }
    // Capacity 1, 2 and 4 fill at lengths 1, 2 and 3; the fourth child fits.
    assert_eq!(requests, [vec![1], vec![2], vec![4], vec![]]);
}

#[test]
fn a_child_order_growth_names_the_entries_of_the_bank_and_the_capacity_it_reaches() {
    // A child-order bank holds 32-bit record-index entries inline in its record, so the growth
    // names a four-byte element and no wrapper. The first append reserves exactly the one slot it
    // needs, and the second the capacity of 8 that the admission chooses.
    let (owner, root) = fragment(storage());
    let (owner, a) = create(owner, element("a"));
    let (mut owner, b) = create(owner, element("b"));
    let mut first = Direct::default();
    let insert = |node, parent| Op::Insert {
        node,
        parent,
        before: None,
    };
    try_change(&mut owner, insert(a, root), &mut first).unwrap();
    let mut second = Direct {
        fixed: Some(8),
        ..Direct::default()
    };
    try_change(&mut owner, insert(b, root), &mut second).unwrap();
    for (admission, len, capacity, chosen) in [(&first, 0, 0, 1), (&second, 1, 1, 8)] {
        let [growth] = admission.growths[..] else {
            panic!("one child order grows: {:?}", admission.growths);
        };
        assert_eq!(
            growth.bank(),
            HtmlConstructionBank::Children { position: 0 }
        );
        assert_eq!(growth.before().initialized_len(), len);
        assert_eq!(growth.before().capacity(), capacity);
        assert_eq!(growth.element_size(), size_of::<u32>());
        assert_eq!(growth.wrapper_bytes(), 0);
        let last = admission.records.last().unwrap();
        assert_eq!(last.chosen(), chosen);
        assert_eq!(last.reached().initialized_len(), len);
        assert!(last.reached().capacity() >= chosen);
    }
}

#[test]
fn append_cycle_refuses_before_any_child_order_growth() {
    let (owner, outer) = create(storage(), element("outer"));
    let (owner, inner) = create(owner, element("inner"));
    let (mut owner, _) = change(
        owner,
        Op::Insert {
            node: inner,
            parent: outer,
            before: None,
        },
    );
    // `inner` has no spare child capacity, yet the cycle is reached first.
    let mut admission = Direct::default();
    let error = try_change(
        &mut owner,
        Op::Insert {
            node: outer,
            parent: inner,
            before: None,
        },
        &mut admission,
    )
    .expect_err("a cycle is refused");
    assert_eq!(error, cause::CYCLE);
    assert!(admission.growths.is_empty());
    assert!(owner.node(&outer).unwrap().parent().is_none());
}

#[test]
fn checked_handles_and_ordinals_preserve_identity_across_nonzero_document_publication() {
    let (owner, inert) = fragment(storage());
    let (owner, document) = create(
        owner,
        Make::Document {
            mode: HtmlDocumentMode::Quirks,
        },
    );
    let (owner, template) = create(owner, element("template"));
    let contents = owner
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let (owner, _) = change(
        owner,
        Op::Insert {
            node: template,
            parent: document,
            before: None,
        },
    );
    let id = owner.result_id(&document).unwrap();
    let template_id = owner.result_id(&template).unwrap();
    let contents_id = owner.result_id(&contents).unwrap();
    assert_eq!(owner.position(&document).unwrap(), 1);
    assert!(owner.node_at(usize::MAX).is_none());
    assert_eq!(owner.node_by_result_id(&id).unwrap().handle(), document);
    let (foreign, foreign_handle) = fragment(storage());
    assert!(has_cause(
        owner.position(&foreign_handle).err().as_ref(),
        cause::FOREIGN_BUILDER
    ));
    assert!(
        owner
            .node_by_result_id(&foreign.result_id(&foreign_handle).unwrap())
            .is_err()
    );
    let owner = refuse(
        owner,
        Op::Insert {
            node: foreign_handle,
            parent: inert,
            before: None,
        },
        cause::FOREIGN_BUILDER,
    );
    let result = publish(owner, document);
    let document = result.document().unwrap();
    assert_eq!(document.root().id(), id);
    assert_eq!(document.mode(), HtmlDocumentMode::Quirks);
    assert_eq!(
        document.node(&template_id).unwrap().template_contents(),
        Some(contents_id)
    );
    assert_eq!(
        document.node(&contents_id).unwrap().fragment_owner(),
        Some(HtmlFragmentOwner::Template(template_id))
    );
    assert_eq!(document.usage().nodes(), 4);
    assert_eq!(document.usage().edges(), 2);
    assert_eq!(document.usage().depth(), 2);
}

#[test]
fn template_shadow_reference_is_not_an_owning_depth_or_edge() {
    let (owner, root) = fragment(storage());
    let (owner, host) = create(owner, element("div"));
    let (owner, template) = create(owner, element("template"));
    let ordinary = owner
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let (owner, shadow) = create(
        owner,
        Make::ShadowContents {
            properties: properties(),
        },
    );
    let (owner, _) = change(
        owner,
        Op::Insert {
            node: host,
            parent: root,
            before: None,
        },
    );
    let edges = owner.usage().edges();
    let (owner, _) = change(
        owner,
        Op::AttachTemplateShadow {
            template,
            host,
            contents: shadow,
        },
    );
    // The template gave up its owned contents edge and took a reference, and the host gained a
    // shadow root edge.
    assert_eq!(owner.usage().edges(), edges);
    assert_eq!(owner.usage().depth(), 2);
    assert_eq!(
        owner.node(&template).unwrap().template_contents_kind(),
        Some(HtmlTemplateContentsKind::ShadowReference)
    );
    assert!(owner.node(&ordinary).unwrap().fragment_owner().is_none());
    assert_eq!(
        owner.node(&shadow).unwrap().shadow_properties(),
        Some(properties())
    );
    assert_eq!(
        owner
            .node(&host)
            .unwrap()
            .shadow_root()
            .unwrap()
            .properties(),
        properties()
    );
    let owner = refuse(
        owner,
        Op::AttachTemplateShadow {
            template,
            host,
            contents: shadow,
        },
        cause::INVALID_TEMPLATE_ASSOCIATION,
    );
    let shadow_id = owner.result_id(&shadow).unwrap();
    let host_id = owner.result_id(&host).unwrap();
    assert_eq!(
        owner.node(&shadow).unwrap().fragment_owner(),
        Some(&HtmlFragmentOwner::ShadowHost(host_id))
    );
    let result = publish(owner, root);
    assert_eq!(
        result
            .node(&host_id)
            .unwrap()
            .declarative_shadow_root()
            .unwrap()
            .contents(),
        shadow_id
    );
}

#[test]
fn captured_shadow_role_retains_properties_without_a_local_owner() {
    let (owner, root) = fragment(storage());
    let (owner, template) = create(owner, element("template"));
    let (owner, shadow) = create(
        owner,
        Make::ShadowContents {
            properties: properties(),
        },
    );
    let old = owner.usage();
    let (owner, _) = change(
        owner,
        Op::SetTemplateShadowContents {
            template,
            contents: shadow,
        },
    );
    assert_eq!(owner.usage().edges(), old.edges() - 1);
    assert_eq!(owner.usage().depth(), 0);
    assert!(owner.node(&shadow).unwrap().fragment_owner().is_none());
    let owner = refuse(
        owner,
        Op::SetTemplateShadowContents {
            template,
            contents: shadow,
        },
        cause::INVALID_TEMPLATE_ASSOCIATION,
    );
    let template_id = owner.result_id(&template).unwrap();
    let shadow_id = owner.result_id(&shadow).unwrap();
    let result = publish(owner, root);
    assert_eq!(
        result.node(&template_id).unwrap().template_contents(),
        Some(shadow_id)
    );
    let contents = result.node(&shadow_id).unwrap();
    assert!(contents.is_shadow_contents());
    assert_eq!(contents.shadow_properties(), Some(properties()));
    assert!(contents.fragment_owner().is_none());
}

#[test]
fn a_published_result_names_its_records_by_the_ids_of_the_storage_it_came_from() {
    let (owner, root) = fragment(storage());
    let (owner, child) = create(owner, element("child"));
    let (owner, _) = change(
        owner,
        Op::Insert {
            node: child,
            parent: root,
            before: None,
        },
    );
    let root_id = owner.result_id(&root).unwrap();
    let child_id = owner.result_id(&child).unwrap();
    let result = publish(owner, root);
    assert_eq!(result.len(), 2);
    assert_eq!(result.node(&root_id).unwrap().id(), root_id);
    assert_eq!(result.node(&child_id).unwrap().parent(), Some(root_id));
    assert_eq!(result.node_by_result_id(&child_id).unwrap().position(), 1);
    // Another storage's result refuses these ids.
    let (other, other_root) = fragment(storage());
    let other_result = publish(other, other_root);
    assert_eq!(other_result.node(&child_id).unwrap_err(), ForeignHtmlResult);
}

#[test]
fn record_layout_exhaustion_refuses_before_provider_growth_and_preserves_existing_handles() {
    let (mut owner, root) = fragment(storage());
    let before = owner.usage();
    // The first request that needs room shows the size of a record.
    let mut sizing = Direct::default();
    granted(owner.reserve_records(1, &mut sizing)).unwrap();
    let size = sizing.growths[0].element_size();
    let additional = (isize::MAX as usize) / size + 1;
    let mut admission = Direct::default();
    let error = granted(owner.reserve_records(additional, &mut admission)).unwrap_err();
    assert_eq!(error, cause::REPRESENTATION_EXHAUSTED);
    assert!(admission.growths.is_empty());
    assert_eq!(owner.usage(), before);
    assert_eq!(owner.position(&root), Ok(0));
    assert_eq!(owner.len(), 1);
}

/// Two storages issue handles and ids at the same record index; each storage and result refuses
/// the other's with the foreign-builder cause or the foreign-result error.
#[test]
fn handles_and_ids_of_another_storage_are_refused_at_an_equal_record_index() {
    let (first, first_root) = fragment(storage());
    let (second, second_root) = fragment(storage());
    assert_eq!(first.position(&first_root), second.position(&second_root));
    assert_ne!(first_root, second_root);
    let first_id = first.result_id(&first_root).unwrap();
    let second_id = second.result_id(&second_root).unwrap();
    assert_ne!(first_id, second_id);
    assert!(has_cause(
        second.position(&first_root).err().as_ref(),
        cause::FOREIGN_BUILDER
    ));
    assert!(has_cause(
        second.result_id(&first_root).err().as_ref(),
        cause::FOREIGN_BUILDER
    ));
    assert!(has_cause(
        second.node(&first_root).err().as_ref(),
        cause::FOREIGN_BUILDER
    ));
    assert_eq!(
        second.node_by_result_id(&first_id).unwrap_err(),
        ForeignHtmlResult
    );
    let (second, child) = create(second, element("child"));
    let usage = second.usage();
    let second = refuse(
        second,
        Op::Insert {
            node: child,
            parent: first_root,
            before: None,
        },
        cause::FOREIGN_BUILDER,
    );
    assert_eq!(second.usage(), usage);
    let published = publish(first, first_root);
    assert_eq!(published.node(&second_id).unwrap_err(), ForeignHtmlResult);
    assert_eq!(
        published.node_by_result_id(&second_id).unwrap_err(),
        ForeignHtmlResult
    );
    assert_eq!(published.node(&first_id).unwrap().id(), first_id);
}

/// A record count past what a 32-bit node index names is refused before any record storage grows,
/// and the existing handles and ids stay valid.
#[cfg(target_pointer_width = "64")]
#[test]
fn record_count_beyond_the_node_index_refuses_before_provider_growth() {
    let (mut owner, root) = fragment(storage());
    let root_id = owner.result_id(&root).unwrap();
    let before = owner.usage();
    // One retained record plus u32::MAX more is 2^32 records: the last index would be u32::MAX.
    let mut admission = Direct::default();
    let error = granted(owner.reserve_records(u32::MAX as usize, &mut admission)).unwrap_err();
    assert_eq!(error, cause::REPRESENTATION_EXHAUSTED);
    assert!(admission.growths.is_empty());
    assert_eq!(owner.usage(), before);
    assert_eq!(owner.len(), 1);
    assert_eq!(owner.position(&root), Ok(0));
    assert_eq!(owner.result_id(&root).unwrap(), root_id);
}

#[test]
fn already_correct_insertions_change_no_order_and_allocate_nothing() {
    for before_successor in [false, true] {
        let (owner, root) = fragment(storage());
        let (owner, a) = create(owner, element("a"));
        let (owner, b) = create(owner, element("b"));
        let (owner, _) = change(
            owner,
            Op::Insert {
                node: a,
                parent: root,
                before: None,
            },
        );
        let (mut owner, _) = change(
            owner,
            Op::Insert {
                node: b,
                parent: root,
                before: None,
            },
        );
        let usage = owner.usage();
        // The node leaves the order before it returns, so the full order needs no growth.
        let mut admission = Direct::default();
        try_change(
            &mut owner,
            Op::Insert {
                node: if before_successor { a } else { b },
                parent: root,
                before: before_successor.then_some(b),
            },
            &mut admission,
        )
        .unwrap();
        assert!(admission.growths.is_empty());
        assert_eq!(owner.usage(), usage);
        assert_eq!(
            owner
                .node(&root)
                .unwrap()
                .children()
                .map(HtmlConstructionNode::handle)
                .collect::<Vec<_>>(),
            [a, b]
        );
        assert_eq!(owner.node(&a).unwrap().parent().unwrap().handle(), root);
        assert_eq!(owner.node(&b).unwrap().parent().unwrap().handle(), root);
    }
}

#[test]
fn self_reference_validates_parent_before_noop_and_valid_noop_has_no_allocation() {
    let (owner, root) = fragment(storage());
    let (owner, other) = create(owner, element("other"));
    let (owner, a) = create(owner, element("a"));
    let (owner, _) = change(
        owner,
        Op::Insert {
            node: a,
            parent: root,
            before: None,
        },
    );
    let usage = owner.usage();
    let mut owner = refuse(
        owner,
        Op::Insert {
            node: a,
            parent: other,
            before: Some(a),
        },
        cause::WRONG_SIBLING,
    );
    assert_eq!(owner.node(&a).unwrap().parent().unwrap().handle(), root);
    assert_eq!(owner.node(&other).unwrap().child_count(), 0);
    let mut admission = Direct::default();
    try_change(
        &mut owner,
        Op::Insert {
            node: a,
            parent: root,
            before: Some(a),
        },
        &mut admission,
    )
    .unwrap();
    assert!(admission.growths.is_empty());
    assert_eq!(owner.usage(), usage);
    assert_eq!(
        owner
            .node(&root)
            .unwrap()
            .children()
            .map(HtmlConstructionNode::handle)
            .collect::<Vec<_>>(),
        [a]
    );
}

#[test]
fn element_creation_installs_supplied_link_history_with_the_record() {
    for supplied in [
        None,
        Some(surgeist_html::elements::link::HtmlLinkFlags::fresh()),
    ] {
        let (owner, _) = fragment(storage());
        let attributes = OrderedAttributes::new([surgeist_html::values::Attribute::new(
            QualifiedName::new(Namespace::none(), None, "rel"),
            "stylesheet",
        )])
        .unwrap();
        let backing = attributes.shared_attributes().clone();
        let (owner, link) = create(
            owner,
            Make::Element {
                name: QualifiedName::new(Namespace::html(), None, "link"),
                attributes,
                link_flags: supplied,
            },
        );
        let ConstructionNodeKind::Element(element) = owner.node(&link).unwrap().kind() else {
            panic!()
        };
        assert_eq!(element.link_flags(), supplied);
        assert!(Arc::ptr_eq(
            element.attributes().to_ordered().shared_attributes(),
            &backing
        ));
        assert_eq!(
            element
                .attributes()
                .get(0)
                .unwrap()
                .value()
                .to_utf8()
                .unwrap(),
            "stylesheet"
        );
        assert_eq!(owner.usage().attributes(), 1);
    }
}

#[test]
fn invalid_link_history_refuses_after_the_name_and_changes_no_record() {
    for (namespace, local) in [
        (Namespace::html(), "div"),
        (Namespace::svg(), "link"),
        (Namespace::html(), "LINK"),
    ] {
        let mut owner = storage();
        let name = QualifiedName::new(namespace, None, local);
        let attributes = OrderedAttributes::new([surgeist_html::values::Attribute::new(
            QualifiedName::new(Namespace::none(), None, "x"),
            "v",
        )])
        .unwrap();
        let mut admission = Direct::default();
        let error = try_create(
            &mut owner,
            Make::Element {
                name,
                attributes,
                link_flags: Some(surgeist_html::elements::link::HtmlLinkFlags::fresh()),
            },
            &mut admission,
        )
        .unwrap_err();
        assert_eq!(error, cause::WRONG_NODE_KIND);
        // The name was interned and charged first; no record bank grew and no record exists.
        assert!(!admission.growths.is_empty());
        assert!(admission.growths.iter().all(|growth| matches!(
            growth.bank(),
            HtmlConstructionBank::Names
                | HtmlConstructionBank::NameSlots
                | HtmlConstructionBank::NameEntry
                | HtmlConstructionBank::Text
        )));
        assert_eq!(owner.len(), 0);
        assert_eq!(owner.usage().nodes(), 0);
    }
}

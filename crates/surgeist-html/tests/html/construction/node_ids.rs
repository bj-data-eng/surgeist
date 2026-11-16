//! Node ids and builder handles are small `Copy` values that compare by storage serial and index.
use crate::html::support::{cause, construction_limits as limits};
use std::collections::HashSet;
use surgeist_html::{
    construction::{
        ForeignHtmlResult, HtmlBuilderNode, HtmlDocument, HtmlDocumentMode, HtmlFragment,
        HtmlNodeId,
    },
    values::{Namespace, OrderedAttributes, QualifiedName},
};

fn assert_copy<T: Copy>() {}

/// A value of the type can be used after it is copied, which a move would forbid.
fn use_twice<T: Copy + std::fmt::Debug + Eq>(value: T) {
    let first = value;
    let second = value;
    assert_eq!(first, second);
    assert_eq!(first, value);
}

#[test]
fn node_ids_and_builder_handles_are_copy_values_of_sixteen_bytes() {
    assert_copy::<HtmlNodeId>();
    assert_copy::<HtmlBuilderNode>();
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(size_of::<HtmlNodeId>(), 16);
        assert_eq!(size_of::<HtmlBuilderNode>(), 16);
    }
    let builder = HtmlFragment::builder(limits()).unwrap();
    use_twice(builder.root());
    let fragment = builder.finish().unwrap();
    use_twice(fragment.root().id());
}

#[test]
fn builds_with_an_equal_record_index_issue_different_ids_and_handles() {
    let first = HtmlFragment::builder(limits()).unwrap();
    let second = HtmlFragment::builder(limits()).unwrap();
    assert_ne!(first.root(), second.root());
    let (first, second) = (first.finish().unwrap(), second.finish().unwrap());
    // Both roots are the first record of their storage.
    let (first_root, second_root) = (first.root().id(), second.root().id());
    assert_ne!(first_root, second_root);
    let ids: HashSet<HtmlNodeId> = [first_root, second_root, first_root, second_root].into();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&first_root) && ids.contains(&second_root));
}

#[test]
fn a_cloned_result_keeps_ids_and_a_copied_id_outlives_its_builder() {
    let mut builder = HtmlDocument::builder(limits(), HtmlDocumentMode::NoQuirks).unwrap();
    let root = builder.root();
    let element = builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let handle = element;
    let document = builder.finish().unwrap();
    let id = document.root().children().next().unwrap();
    let copy = id;
    let clone = document.clone();
    assert_eq!(clone.root().id(), document.root().id());
    assert_eq!(clone.root().children().next(), Some(id));
    assert_eq!(clone.node(&copy).unwrap().id(), id);
    // The handle was copied before `finish` consumed the builder, and equals the original.
    assert_eq!(handle, element);
    let ids: HashSet<HtmlNodeId> = clone.root().children().collect();
    assert!(ids.contains(&id));
}

#[test]
fn a_foreign_id_is_refused_by_every_reader_with_the_foreign_result_error() {
    let mut first = HtmlFragment::builder(limits()).unwrap();
    let mut second = HtmlFragment::builder(limits()).unwrap();
    let (first_root, second_root) = (first.root(), second.root());
    first.append_text(&first_root, "first").unwrap();
    second.append_text(&second_root, "second").unwrap();
    // The foreign handle is a serial mismatch, with no change to either builder.
    let usage = second.usage();
    assert_eq!(
        second.append_text(&first_root, "foreign").unwrap_err(),
        cause::FOREIGN_BUILDER
    );
    assert_eq!(second.usage(), usage);
    let (first, second) = (first.finish().unwrap(), second.finish().unwrap());
    // The text node of each result sits at the same record index.
    let first_text = first.root().children().next().unwrap();
    let second_text = second.root().children().next().unwrap();
    assert_ne!(first_text, second_text);
    assert_eq!(second.node(&first_text).unwrap_err(), ForeignHtmlResult);
    assert_eq!(second.fragment(&first_text).unwrap_err(), ForeignHtmlResult);
    assert_eq!(first.node(&second_text).unwrap_err(), ForeignHtmlResult);
    assert_eq!(first.fragment(&second_text).unwrap_err(), ForeignHtmlResult);
}

#[test]
fn debug_output_names_the_serial_and_the_zero_based_position() {
    let builder = HtmlFragment::builder(limits()).unwrap();
    let handle = format!("{:?}", builder.root());
    assert!(
        handle.starts_with("HtmlBuilderNode { tree: ") && handle.ends_with(", index: 0 }"),
        "{handle}"
    );
    let fragment = builder.finish().unwrap();
    let id = format!("{:?}", fragment.root().id());
    assert!(
        id.starts_with("HtmlNodeId { tree: ") && id.ends_with(", index: 0 }"),
        "{id}"
    );
}

/// Every node of a result has its own record position below the node count of the result's usage,
/// whatever its place in the tree, and the position of a record is the same in two results built
/// the same way: it names a record of one result and carries no order.
#[test]
fn a_node_position_is_its_record_below_the_node_count_of_the_result() {
    let build = || {
        let mut builder = HtmlDocument::builder(limits(), HtmlDocumentMode::NoQuirks).unwrap();
        let root = builder.root();
        let template = builder
            .append_element(
                &root,
                QualifiedName::new(Namespace::html(), None, "template"),
                OrderedAttributes::default(),
            )
            .unwrap();
        let contents = builder.template_contents(&template).unwrap().unwrap();
        // Created after the contents, placed before the template.
        builder.append_text(&contents, "inside").unwrap();
        builder
            .append_element(
                &root,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
            )
            .unwrap();
        builder.finish().unwrap()
    };
    let document = build();
    let mut positions = Vec::new();
    let mut pending = vec![document.root().id()];
    while let Some(id) = pending.pop() {
        let node = document.node(&id).unwrap();
        positions.push(id.position());
        pending.extend(node.children());
        pending.extend(node.template_contents());
    }
    let nodes = document.usage().nodes();
    positions.sort_unstable();
    assert_eq!(positions, (0..nodes).collect::<Vec<_>>());
    // The records of a second build sit at the same positions, in a result of their own.
    let other = build();
    assert_eq!(
        other.root().id().position(),
        document.root().id().position()
    );
    assert_ne!(other.root().id(), document.root().id());
}

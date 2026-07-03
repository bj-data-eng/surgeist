use super::{
    RetainedStyleTree, clear_style_cache_for_retained_changes, style_change_from_retained_flags,
};
use crate::{retained, style};

#[test]
fn retained_style_tree_exposes_retained_snapshot_to_style_resolver() {
    let mut model = retained::Model::empty();
    let root = model.root();
    let panel = insert(&mut model, root, 0, element("panel"));
    let declarations = style::Declarations::new()
        .try_bg(style::Color::BLACK)
        .unwrap();
    let snapshot = model.snapshot();
    let tree = RetainedStyleTree::new(snapshot);

    let resolved = style::Resolver::new(style::Sheet::new())
        .resolve(style::Context::new(&tree, panel).local(&declarations))
        .unwrap();

    assert_eq!(
        resolved.get(style::Property::Background),
        &style::Value::Color(style::Color::BLACK)
    );
}

#[test]
fn retained_style_tree_exposes_canonical_tree_facts() {
    let mut model = retained::Model::empty();
    let root = model.root();
    let parent = insert(&mut model, root, 0, element("panel"));
    let first = insert(&mut model, parent, 0, element("label"));
    let second = insert(
        &mut model,
        parent,
        1,
        retained::Element::text(retained::Text::new("hello").unwrap()),
    );
    let snapshot = model.snapshot();
    let tree = RetainedStyleTree::new(snapshot);

    let parent_node = style::Tree::node(&tree, parent).unwrap();
    assert_eq!(parent_node.tag, Some(&retained::Tag::new("panel").unwrap()));
    assert!(!parent_node.text);
    let text_node = style::Tree::node(&tree, second).unwrap();
    assert_eq!(text_node.tag, None);
    assert!(text_node.text);

    let children = style::Tree::children(&tree, parent, style::Traversal::Canonical)
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(children, vec![first, second]);
    assert_eq!(
        style::Tree::previous_sibling(&tree, second, style::Traversal::Canonical).unwrap(),
        Some(first)
    );
}

#[test]
fn retained_style_change_mapping_marks_local_text_invalidation() {
    let change = style_change_from_retained_flags(retained::ChangeFlags::empty().label().text());

    assert!(change.scope.node);
    assert!(!change.scope.siblings);
    assert!(!change.scope.descendants);
    assert!(!change.rematch);
    assert!(change.invalidation.layout);
    assert!(change.invalidation.text);
    assert!(change.invalidation.paint);
}

#[test]
fn retained_style_cache_clearing_distinguishes_local_and_broad_scopes() {
    let mut model = retained::Model::empty();
    let root = model.root();
    let first = insert(&mut model, root, 0, element("one"));
    let second = insert(&mut model, root, 1, element("two"));
    let local_one = style::Declarations::new()
        .try_text_color(style::Color::BLACK)
        .unwrap();
    let local_two = style::Declarations::new()
        .try_bg(style::Color::BLACK)
        .unwrap();
    let mut resolver = style::Resolver::new(style::Sheet::new());

    let snapshot = model.snapshot();
    let tree = RetainedStyleTree::new(snapshot);
    resolver
        .resolve(style::Context::new(&tree, first).local(&local_one))
        .unwrap();
    resolver
        .resolve(style::Context::new(&tree, second).local(&local_two))
        .unwrap();
    resolver
        .resolve(style::Context::new(&tree, first).local(&local_one))
        .unwrap();
    resolver
        .resolve(style::Context::new(&tree, second).local(&local_two))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 2);

    let local_change = model
        .apply(retained::Patch::SetLabel {
            id: first,
            label: Some(retained::Text::new("updated label").unwrap()),
        })
        .unwrap();
    clear_style_cache_for_retained_changes(&mut resolver, local_change.changes());
    let snapshot = model.snapshot();
    let tree = RetainedStyleTree::new(snapshot);
    resolver
        .resolve(style::Context::new(&tree, second).local(&local_two))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 2);
    resolver
        .resolve(style::Context::new(&tree, first).local(&local_one))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 2);

    resolver
        .resolve(style::Context::new(&tree, first).local(&local_one))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 3);
    let broad_change = model
        .apply(retained::Patch::SetClasses {
            id: first,
            classes: vec![retained::Class::new("featured").unwrap()],
        })
        .unwrap();
    clear_style_cache_for_retained_changes(&mut resolver, broad_change.changes());

    assert_eq!(resolver.cache_hits(), 0);
    let snapshot = model.snapshot();
    let tree = RetainedStyleTree::new(snapshot);
    resolver
        .resolve(style::Context::new(&tree, second).local(&local_two))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 0);
    resolver
        .resolve(style::Context::new(&tree, second).local(&local_two))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 1);
}

fn insert(
    model: &mut retained::Model,
    parent: retained::Id,
    index: usize,
    element: retained::Element,
) -> retained::Id {
    model
        .apply(retained::Patch::Insert {
            parent,
            index,
            element,
        })
        .unwrap()
        .changes()
        .inserted()[0]
}

fn element(name: &str) -> retained::Element {
    retained::Element::tagged(retained::Tag::new(name).unwrap())
}

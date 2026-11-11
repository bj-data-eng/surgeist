//! `HtmlAttributesInput::get` hands out a view by value at every index below `len`, and `None` at
//! and past it. The expectations are the authored attributes; a view's name and value are the
//! collection's own storage, which the tests show by identity and by shared backing.

use crate::html::support::construction_limits as limits;
use surgeist_html::{
    construction::{HtmlFragment, HtmlNodeKind},
    values::{
        Attribute, DocumentString, HtmlAttributeInput, HtmlAttributesInput, HtmlNameInput,
        Namespace, OrderedAttributes, QualifiedName,
    },
};

const XLINK: &str = "http://www.w3.org/1999/xlink";

fn authored() -> Vec<Attribute> {
    let plain = |local: &str, value: DocumentString| {
        Attribute::new(QualifiedName::new(Namespace::none(), None, local), value)
    };
    vec![
        plain("id", "a".into()),
        Attribute::new(
            QualifiedName::new(Namespace::from_uri(XLINK), Some("xlink".into()), "href"),
            "far",
        ),
        plain("class", DocumentString::from_utf16(vec![0x63, 0, 0xD800])),
    ]
}

/// Reads every index of `collection` through the trait and compares it with `authored`, then
/// checks that `len`, `len + 1` and `usize::MAX` hold no attribute.
fn observe<A: HtmlAttributesInput + ?Sized>(collection: &A, authored: &[Attribute]) {
    assert_eq!(collection.len(), authored.len());
    assert_eq!(collection.is_empty(), authored.is_empty());
    for (index, expected) in authored.iter().enumerate() {
        let view = collection
            .get(index)
            .unwrap_or_else(|| panic!("index {index}"));
        let name = view.name();
        assert_eq!(
            name.namespace_uri(),
            expected.name().namespace().uri(),
            "namespace at {index}"
        );
        assert_eq!(name.prefix(), expected.name().prefix(), "prefix at {index}");
        assert_eq!(
            name.local_name(),
            expected.name().local_name(),
            "local at {index}"
        );
        assert_eq!(view.value(), expected.value(), "value at {index}");
        assert_eq!(
            view.value().code_units(),
            expected.value().code_units(),
            "units at {index}"
        );
    }
    assert!(collection.get(authored.len()).is_none());
    assert!(collection.get(authored.len() + 1).is_none());
    assert!(collection.get(usize::MAX).is_none());
}

/// A view lends the storage of the row it names: the same `DocumentString`, not a copy.
fn lends<A: HtmlAttributesInput + ?Sized>(collection: &A, rows: &[&Attribute]) {
    for (index, row) in rows.iter().enumerate() {
        let view = collection.get(index).unwrap();
        assert!(std::ptr::eq(view.value(), row.value()), "value row {index}");
        assert!(view.value().shares_storage_with(row.value()));
        let name = view.name();
        assert!(std::ptr::eq(name.local_name(), row.name().local_name()));
        assert!(
            name.local_name()
                .shares_storage_with(row.name().local_name())
        );
        assert_eq!(name.prefix().is_some(), row.name().prefix().is_some());
        if let (Some(viewed), Some(stored)) = (name.prefix(), row.name().prefix()) {
            assert!(std::ptr::eq(viewed, stored), "prefix row {index}");
            assert!(
                viewed.shares_storage_with(stored),
                "prefix backing row {index}"
            );
        }
    }
}

#[test]
fn a_slice_returns_each_row_by_view_and_none_at_and_past_its_length() {
    let rows = authored();
    observe(rows.as_slice(), &rows);
    lends(rows.as_slice(), &rows.iter().collect::<Vec<_>>());
    let none: [Attribute; 0] = [];
    observe(none.as_slice(), &[]);
}

#[test]
fn ordered_attributes_return_each_row_by_view_and_none_at_and_past_their_length() {
    let rows = authored();
    let ordered = OrderedAttributes::new(rows.clone()).unwrap();
    observe(&ordered, &rows);
    lends(&ordered, &ordered.as_slice().iter().collect::<Vec<_>>());
    // The values are the authored values' backing, not copies of their units.
    for (view, expected) in ordered.as_slice().iter().zip(&rows) {
        assert!(view.value().shares_storage_with(expected.value()));
    }
    observe(&OrderedAttributes::default(), &[]);
}

#[test]
fn a_published_element_returns_each_row_by_view_and_none_at_and_past_its_length() {
    let rows = authored();
    let mut builder = HtmlFragment::builder(limits()).unwrap();
    let root = builder.root();
    let name = QualifiedName::new(Namespace::html(), None, "a");
    builder
        .append_element(&root, name, OrderedAttributes::new(rows.clone()).unwrap())
        .unwrap();
    let fragment = builder.finish().unwrap();
    let child = fragment.root().children().next().unwrap();
    let HtmlNodeKind::Element(element) = fragment.node(&child).unwrap().kind() else {
        panic!("expected an element");
    };
    let attributes = element.attributes();
    observe(&attributes, &rows);
    let stored: Vec<&Attribute> = attributes.iter().collect();
    lends(&attributes, &stored);
    for (view, expected) in stored.iter().zip(&rows) {
        assert!(view.value().shares_storage_with(expected.value()));
    }
}

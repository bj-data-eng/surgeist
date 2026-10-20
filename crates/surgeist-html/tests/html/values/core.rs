use surgeist_html::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName};

#[test]
fn validated_attribute_backing_keeps_identity_capacity_and_exact_units() {
    use std::sync::Arc;

    let mut raw = Vec::with_capacity(8);
    raw.push(Attribute::new(
        QualifiedName::new(Namespace::none(), None, "data-x"),
        DocumentString::from_utf16([0xd800, 0, 0xdc00]),
    ));
    let capacity = raw.capacity();
    let attributes = OrderedAttributes::new(raw).unwrap();
    assert_eq!(attributes.shared_attributes().capacity(), capacity);
    assert_eq!(Arc::strong_count(attributes.shared_attributes()), 1);
    let alias = attributes.clone();
    let backing = attributes.into_shared_attributes();
    assert!(Arc::ptr_eq(&backing, alias.shared_attributes()));
    assert_eq!(backing.capacity(), capacity);
    assert_eq!(
        backing[0].name().local_name().code_units(),
        [100, 97, 116, 97, 45, 120]
    );
    assert_eq!(backing[0].value().code_units(), [0xd800, 0, 0xdc00]);
    drop(alias);
    assert_eq!(Arc::strong_count(&backing), 1);
}

#[test]
fn supplementary_characters_round_trip_with_code_unit_lengths() {
    let value = DocumentString::from("a\u{1F642}z");
    assert_eq!(value.code_units(), &[0x0061, 0xD83D, 0xDE42, 0x007A]);
    assert_eq!(value.len(), 4);
    assert_eq!(
        value.code_points().collect::<Vec<_>>(),
        [0x61, 0x1F642, 0x7A]
    );
    assert_eq!(value.to_utf8().unwrap(), "a\u{1F642}z");
    assert_eq!(value, DocumentString::from(String::from("a\u{1F642}z")));
}

#[test]
fn raw_units_preserve_whitespace_nulls_and_isolated_surrogates() {
    let units = [0x20, 0xD800, 0x09, 0, 0xDC00, 0x0A, 0x20];
    let value = DocumentString::from_utf16(units.as_slice());
    assert_eq!(value.code_units(), units);
    assert_eq!(value.len(), 7);
    assert_eq!(
        value.code_points().collect::<Vec<_>>(),
        [0x20, 0xD800, 0x09, 0, 0xDC00, 0x0A, 0x20]
    );
    let clone = value.clone();
    assert_eq!(clone.code_units(), units);
    assert_eq!(
        DocumentString::from(" \t\0\n ").to_utf8().unwrap(),
        " \t\0\n "
    );
}

#[test]
fn utf8_conversion_reports_the_first_isolated_surrogate_in_code_units() {
    for (units, offset, surrogate) in [
        (vec![0xD800], 0, 0xD800),
        (vec![0xDC00], 0, 0xDC00),
        (
            vec![0xD83D, 0xDE42, 0x0061, 0xD800, 0xDC00, 0xDFFF],
            5,
            0xDFFF,
        ),
        (vec![0xD800, 0xD801, 0xDC00], 0, 0xD800),
        (vec![0x0061, 0xDC00, 0xD800], 1, 0xDC00),
    ] {
        let value = DocumentString::from_utf16(units.clone());
        let error = value.to_utf8().unwrap_err();
        assert_eq!(error.code_unit_offset(), offset);
        assert_eq!(error.surrogate(), surrogate);
        assert_eq!(value.code_units(), units);
    }
}

#[test]
fn code_point_iteration_combines_only_adjacent_valid_pairs() {
    let value = DocumentString::from_utf16(vec![
        0xD800, 0xD801, 0xDC00, 0xDC01, 0xDBFF, 0xDFFF, 0x0061, 0xD800,
    ]);
    assert_eq!(
        value.code_points().collect::<Vec<_>>(),
        [0xD800, 0x10400, 0xDC01, 0x10FFFF, 0x61, 0xD800]
    );
}

#[test]
fn empty_document_strings_have_a_lossless_empty_conversion() {
    for value in [DocumentString::default(), DocumentString::from("")] {
        assert!(value.is_empty());
        assert_eq!(value.len(), 0);
        assert_eq!(value.code_units(), []);
        assert_eq!(value.code_points().next(), None);
        assert_eq!(value.to_utf8().unwrap(), "");
    }
}

#[test]
fn named_and_custom_namespaces_use_exact_uri_identity() {
    for (namespace, uri) in [
        (Namespace::html(), "http://www.w3.org/1999/xhtml"),
        (Namespace::svg(), "http://www.w3.org/2000/svg"),
        (Namespace::mathml(), "http://www.w3.org/1998/Math/MathML"),
    ] {
        assert_eq!(namespace, Namespace::from_uri(uri));
        assert_eq!(namespace.uri().unwrap().to_utf8().unwrap(), uri);
    }
    assert_ne!(Namespace::html(), Namespace::svg());
    assert_ne!(Namespace::svg(), Namespace::mathml());
    assert_ne!(Namespace::html(), Namespace::none());
    assert_ne!(
        Namespace::html(),
        Namespace::from_uri("HTTP://www.w3.org/1999/xhtml")
    );
    assert_eq!(Namespace::none(), Namespace::from_uri(""));
    assert_eq!(Namespace::default().uri(), None);
    let raw_uri = DocumentString::from_utf16(vec![0x20, 0xD800, 0x20]);
    assert_eq!(Namespace::from_uri(raw_uri.clone()).uri(), Some(&raw_uri));
}

#[test]
fn raw_qualified_names_do_not_validate_fold_or_trim_components() {
    let prefix = DocumentString::from_utf16(vec![0x20, 0xD800]);
    let local = DocumentString::from_utf16(vec![0x41, 0, 0x3A, 0xDC00, 0x20]);
    let name = QualifiedName::new(Namespace::none(), Some(prefix.clone()), local.clone());
    assert_eq!(name.namespace(), &Namespace::none());
    assert_eq!(name.prefix(), Some(&prefix));
    assert_eq!(name.local_name(), &local);
    assert_ne!(
        QualifiedName::new(Namespace::none(), None, ""),
        QualifiedName::new(Namespace::none(), Some(DocumentString::from("")), "")
    );
    assert_eq!(
        QualifiedName::new(Namespace::html(), None, " A:B ")
            .local_name()
            .to_utf8()
            .unwrap(),
        " A:B "
    );
}

#[test]
fn ordered_attributes_preserve_prefix_order_raw_value_and_namespace_distinctions() {
    let raw_value = DocumentString::from_utf16(vec![0x20, 0xD800, 0, 0x20]);
    let first = Attribute::new(
        QualifiedName::new(Namespace::svg(), Some(DocumentString::from("s")), "id"),
        raw_value.clone(),
    );
    let second = Attribute::new(QualifiedName::new(Namespace::none(), None, "id"), "two");
    let third = Attribute::new(QualifiedName::new(Namespace::svg(), None, "ID"), "three");
    let attributes =
        OrderedAttributes::new([first.clone(), second.clone(), third.clone()]).unwrap();
    assert_eq!(attributes.len(), 3);
    assert!(!attributes.is_empty());
    assert_eq!(
        attributes.as_slice(),
        [first.clone(), second.clone(), third]
    );
    assert_eq!(
        attributes.iter().cloned().collect::<Vec<_>>(),
        attributes.as_slice()
    );
    assert_eq!(attributes.as_slice()[0].value(), &raw_value);
    assert_eq!(
        attributes.as_slice()[0]
            .name()
            .prefix()
            .unwrap()
            .to_utf8()
            .unwrap(),
        "s"
    );
    assert_eq!(
        attributes.get(&Namespace::svg(), &DocumentString::from("id")),
        Some(&first)
    );
    assert_eq!(
        attributes.get(&Namespace::none(), &DocumentString::from("id")),
        Some(&second)
    );
    assert_eq!(
        attributes.get(&Namespace::html(), &DocumentString::from("id")),
        None
    );
}

#[test]
fn duplicate_expanded_names_are_rejected_even_with_different_prefixes_and_values() {
    let uri = Namespace::from_uri("urn:example:attributes");
    let attributes = [
        Attribute::new(
            QualifiedName::new(uri.clone(), Some(DocumentString::from("a")), "name"),
            "one",
        ),
        Attribute::new(QualifiedName::new(Namespace::none(), None, "name"), "two"),
        Attribute::new(
            QualifiedName::new(uri.clone(), Some(DocumentString::from("b")), "name"),
            "three",
        ),
    ];
    let error = OrderedAttributes::new(attributes).unwrap_err();
    assert_eq!(error.namespace(), &uri);
    assert_eq!(error.local_name(), &DocumentString::from("name"));
    assert_eq!(error.first_index(), 0);
    assert_eq!(error.duplicate_index(), 2);
}

#[test]
fn empty_attributes_are_a_valid_neutral_construction() {
    let attributes = OrderedAttributes::new([]).unwrap();
    assert_eq!(attributes, OrderedAttributes::default());
    assert!(attributes.is_empty());
    assert_eq!(attributes.len(), 0);
    assert_eq!(attributes.iter().next(), None);
}

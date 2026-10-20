use surgeist_html::{
    Attribute, DocumentString, KnownHtmlAttribute, KnownHtmlElement, Namespace, QualifiedName,
    VOCABULARY_SOURCE, attribute_inventory, element_inventory,
};

#[test]
fn exact_html_names_recognize_newer_pinned_elements() {
    // The selected edition's ordinary element definitions include both names.
    let selected_content = QualifiedName::new(Namespace::html(), None, "selectedcontent");
    let search = QualifiedName::new(Namespace::html(), None, "search");
    assert_eq!(
        KnownHtmlElement::from_name(&selected_content),
        Some(KnownHtmlElement::Selectedcontent)
    );
    assert_eq!(
        KnownHtmlElement::from_name(&search),
        Some(KnownHtmlElement::Search)
    );
    let row = KnownHtmlElement::Selectedcontent.inventory_row();
    assert_eq!(row.html_elements(), &[KnownHtmlElement::Selectedcontent]);
    assert!(
        row.references()
            .iter()
            .any(|reference| { reference.ends_with("/#the-selectedcontent-element") })
    );
}

#[test]
fn element_recognition_uses_exact_expanded_identity_without_case_folding() {
    let prefixed = QualifiedName::new(Namespace::html(), Some("h".into()), "title");
    assert_eq!(
        KnownHtmlElement::from_name(&prefixed),
        Some(KnownHtmlElement::Title)
    );
    for name in [
        QualifiedName::new(Namespace::svg(), None, "title"),
        QualifiedName::new(Namespace::mathml(), None, "title"),
        QualifiedName::new(Namespace::none(), None, "title"),
        QualifiedName::new(Namespace::html(), None, "TITLE"),
        QualifiedName::new(Namespace::html(), None, "title "),
        QualifiedName::new(
            Namespace::from_uri("HTTP://www.w3.org/1999/xhtml"),
            None,
            "title",
        ),
    ] {
        assert_eq!(KnownHtmlElement::from_name(&name), None);
    }
    assert_eq!(prefixed.prefix(), Some(&DocumentString::from("h")));
}

#[test]
fn attribute_recognition_requires_no_namespace_and_preserves_the_prefix() {
    let prefixed = QualifiedName::new(Namespace::none(), Some("raw".into()), "for");
    assert_eq!(
        KnownHtmlAttribute::from_name(&prefixed),
        Some(KnownHtmlAttribute::For)
    );
    for name in [
        QualifiedName::new(Namespace::html(), None, "for"),
        QualifiedName::new(Namespace::svg(), None, "for"),
        QualifiedName::new(Namespace::none(), None, "FOR"),
        QualifiedName::new(Namespace::none(), None, "data-customer"),
        QualifiedName::new(Namespace::none(), None, "onclick"),
    ] {
        assert_eq!(KnownHtmlAttribute::from_name(&name), None);
    }
    assert_eq!(prefixed.prefix(), Some(&DocumentString::from("raw")));
}

#[test]
fn unknown_and_foreign_records_keep_exact_names_and_values_after_recognition() {
    let name_units = [0x66, 0x6F, 0x72, 0xD800];
    let name = QualifiedName::new(
        Namespace::from_uri("urn:foreign:exact"),
        Some(DocumentString::from_utf16([0xDC00].as_slice())),
        DocumentString::from_utf16(name_units.as_slice()),
    );
    let value_units = [0, 0x20, 0xDFFF];
    let attribute = Attribute::new(name, DocumentString::from_utf16(value_units.as_slice()));
    assert_eq!(KnownHtmlAttribute::from_name(attribute.name()), None);
    assert_eq!(KnownHtmlElement::from_name(attribute.name()), None);
    assert_eq!(attribute.name().local_name().code_units(), name_units);
    assert_eq!(attribute.name().prefix().unwrap().code_units(), &[0xDC00]);
    assert_eq!(attribute.value().code_units(), value_units);
    assert_eq!(
        attribute.name().namespace().uri(),
        Some(&DocumentString::from("urn:foreign:exact"))
    );
    let custom = QualifiedName::new(Namespace::html(), None, "x-customer");
    assert_eq!(KnownHtmlElement::from_name(&custom), None);
    assert_eq!(custom.local_name(), &DocumentString::from("x-customer"));
}

#[test]
fn for_inventory_keeps_label_output_and_processing_instruction_contexts_separate() {
    // HTML's three distinct definitions do not share a reference algorithm.
    let rows = KnownHtmlAttribute::For.inventory_rows().collect::<Vec<_>>();
    let label = rows
        .iter()
        .find(|row| row.html_elements() == [KnownHtmlElement::Label])
        .unwrap();
    let output = rows
        .iter()
        .find(|row| row.html_elements() == [KnownHtmlElement::Output])
        .unwrap();
    let template = rows
        .iter()
        .find(|row| row.html_elements() == [KnownHtmlElement::Template])
        .unwrap();
    assert_eq!(label.value_syntax(), "ID*");
    assert!(
        output
            .value_syntax()
            .contains("unique space-separated tokens")
    );
    assert_eq!(template.value_syntax(), "Text");
    assert!(
        template
            .references()
            .iter()
            .any(|reference| reference.ends_with("/#attr-template-for"))
    );
    assert_ne!(label.row(), template.row());
    assert_ne!(label.row(), output.row());
}

#[test]
fn type_and_value_inventory_do_not_assign_one_universal_value_grammar() {
    let input_type = KnownHtmlAttribute::Type
        .inventory_rows()
        .find(|row| row.html_elements() == [KnownHtmlElement::Input])
        .unwrap();
    let ordered_list_type = KnownHtmlAttribute::Type
        .inventory_rows()
        .find(|row| row.html_elements() == [KnownHtmlElement::Ol])
        .unwrap();
    assert_eq!(input_type.value_syntax(), "input type keyword");
    assert_eq!(
        ordered_list_type.value_syntax(),
        "\"1\"; \"a\"; \"A\"; \"i\"; \"I\""
    );
    let input_value = KnownHtmlAttribute::Value
        .inventory_rows()
        .find(|row| row.html_elements() == [KnownHtmlElement::Input])
        .unwrap();
    let list_item_value = KnownHtmlAttribute::Value
        .inventory_rows()
        .find(|row| row.html_elements() == [KnownHtmlElement::Li])
        .unwrap();
    assert_eq!(input_value.value_syntax(), "Varies*");
    assert_eq!(list_item_value.value_syntax(), "Valid integer");
    assert_ne!(input_value.row(), list_item_value.row());
}

#[test]
fn controls_inventory_includes_the_pinned_image_control_definition() {
    let controls = KnownHtmlAttribute::Controls
        .inventory_rows()
        .next()
        .unwrap();
    assert_eq!(
        controls.html_elements(),
        &[
            KnownHtmlElement::Audio,
            KnownHtmlElement::Video,
            KnownHtmlElement::Img
        ]
    );
    assert_eq!(controls.value_syntax(), "Boolean attribute");
    assert!(
        controls
            .references()
            .iter()
            .any(|reference| reference.ends_with("/#attr-img-controls"))
    );
    assert!(
        controls
            .references()
            .iter()
            .any(|reference| reference.ends_with("/#attr-media-controls"))
    );
}

#[test]
fn global_and_custom_family_entries_remain_open_and_parent_qualifications_remain_visible() {
    let language = KnownHtmlAttribute::Lang.inventory_rows().next().unwrap();
    assert!(language.all_html_elements());
    assert!(language.html_elements().is_empty());
    let form = KnownHtmlAttribute::Form.inventory_rows().next().unwrap();
    assert!(form.form_associated_custom_elements());
    assert!(!form.html_elements().contains(&KnownHtmlElement::Form));
    let height = KnownHtmlAttribute::Height.inventory_rows().next().unwrap();
    assert!(height.html_elements().contains(&KnownHtmlElement::Source));
    assert!(!height.html_elements().contains(&KnownHtmlElement::Picture));
    assert!(height.applicability().contains("source (in picture)"));
    let custom_family = element_inventory()
        .iter()
        .find(|row| row.name() == "autonomous custom elements")
        .unwrap();
    assert!(custom_family.html_elements().is_empty());
    let svg = element_inventory()
        .iter()
        .find(|row| row.name() == "SVG svg")
        .unwrap();
    assert!(svg.html_elements().is_empty());
    assert!(
        svg.references()
            .iter()
            .any(|reference| reference.contains("svgwg"))
    );
}

#[test]
fn canonical_names_round_trip_and_heading_metadata_preserves_its_source_group() {
    for known in KnownHtmlElement::ALL {
        assert_eq!(
            KnownHtmlElement::from_name(&known.qualified_name()),
            Some(*known)
        );
        assert!(known.inventory_row().html_elements().contains(known));
    }
    for known in KnownHtmlAttribute::ALL {
        assert_eq!(
            KnownHtmlAttribute::from_name(&known.qualified_name()),
            Some(*known)
        );
        assert!(known.inventory_rows().all(|row| row.attribute() == *known));
        assert!(known.inventory_rows().next().is_some());
    }
    assert_eq!(
        KnownHtmlElement::H1.inventory_row(),
        KnownHtmlElement::H6.inventory_row()
    );
    assert_eq!(
        KnownHtmlElement::H1.inventory_row().name(),
        "h1, h2, h3, h4, h5, h6"
    );
    assert!(
        attribute_inventory()
            .iter()
            .all(|row| row.references().iter().all(|reference| {
                !reference.starts_with("html-language-") && !reference.starts_with('#')
            }))
    );
}

#[test]
fn inventory_source_identity_selects_the_html_edition_independently_of_the_parser() {
    assert_eq!(
        VOCABULARY_SOURCE.revision(),
        "877d6146487f15b15bc8dd0e3c625063a37b456a"
    );
    assert_eq!(
        VOCABULARY_SOURCE.original_html_sha256(),
        "877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f"
    );
    assert!(VOCABULARY_SOURCE.elements_url().ends_with("/#elements-3"));
    assert!(
        VOCABULARY_SOURCE
            .attributes_url()
            .ends_with("/#attributes-3")
    );
}

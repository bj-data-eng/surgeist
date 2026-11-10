use surgeist_html::{
    values::{Attribute, DocumentString, Namespace, QualifiedName},
    vocabulary::{
        HtmlEventHandlerApplicability, KnownHtmlAttribute, KnownHtmlElement, KnownHtmlEventHandler,
        VOCABULARY_SOURCE, attribute_inventory, element_inventory,
    },
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
fn element_recognition_agrees_with_the_inventory_spelling_for_every_name_and_its_neighbours() {
    let by_scan = |local: &[u16]| {
        KnownHtmlElement::ALL
            .iter()
            .copied()
            .find(|known| known.local_name().encode_utf16().eq(local.iter().copied()))
    };
    let mut probes: Vec<Vec<u16>> = vec![Vec::new(), vec![0xD800], vec![0x61, 0xDFFF]];
    for known in KnownHtmlElement::ALL {
        let units: Vec<u16> = known.local_name().encode_utf16().collect();
        probes.push(units.clone());
        probes.push(units[..units.len() - 1].to_vec());
        probes.push([units.as_slice(), &[0x78]].concat());
        probes.push([units.as_slice(), &[0xD83D, 0xDE00]].concat());
        probes.push([&[0x78], units.as_slice()].concat());
        probes.push(
            units
                .iter()
                .map(|unit| match *unit {
                    0x61..=0x7A => unit - 0x20,
                    other => other,
                })
                .collect(),
        );
    }
    for units in probes {
        let name = QualifiedName::new(
            Namespace::html(),
            None,
            DocumentString::from_utf16(units.as_slice()),
        );
        assert_eq!(
            KnownHtmlElement::from_name(&name),
            by_scan(&units),
            "{units:?}"
        );
    }
    for known in KnownHtmlElement::ALL {
        assert_eq!(
            KnownHtmlElement::from_name(&known.qualified_name()),
            Some(*known)
        );
    }
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
    assert_eq!(
        VOCABULARY_SOURCE.retained_syntax_capture(),
        "references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md"
    );
    assert_eq!(
        VOCABULARY_SOURCE.retained_syntax_capture_sha256(),
        "db1eaacc388d494736080301dd18e9147feb39f2651e2bc7fbeed36ee2cfb214"
    );
    assert!(VOCABULARY_SOURCE.elements_url().ends_with("/#elements-3"));
    assert!(
        VOCABULARY_SOURCE
            .attributes_url()
            .ends_with("/#attributes-3")
    );
}

/// Every recognized element names the first original index row that mentions it, and that row
/// mentions it. The expected row is found by scanning the printed inventory in order.
#[test]
fn every_element_has_the_first_inventory_row_that_mentions_it() {
    assert_eq!(KnownHtmlElement::ALL.len(), 113);
    assert_eq!(element_inventory().len(), 111);
    for element in KnownHtmlElement::ALL {
        let expected = element_inventory()
            .iter()
            .find(|row| row.html_elements().contains(element))
            .unwrap_or_else(|| panic!("{element:?} is mentioned by an inventory row"));
        let row = element.inventory_row();
        assert_eq!(row.row(), expected.row(), "{element:?}");
        assert!(row.html_elements().contains(element), "{element:?}");
    }
    // The heading elements share their single original index row.
    let headings = KnownHtmlElement::H1.inventory_row();
    for heading in [
        KnownHtmlElement::H2,
        KnownHtmlElement::H3,
        KnownHtmlElement::H4,
        KnownHtmlElement::H5,
        KnownHtmlElement::H6,
    ] {
        assert_eq!(heading.inventory_row().row(), headings.row());
    }
}

/// The event handler content attributes the retained capture of the pinned
/// standard lists, read without the generator: each row of the table whose
/// caption names them carries the attribute in its first `code` element and
/// states its elements in the link of its second cell.
fn retained_event_handlers() -> Vec<(String, HtmlEventHandlerApplicability)> {
    let capture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../references/html-language-elements-attributes--877d6146487f--2ca83b5ac984.md");
    let text = std::fs::read_to_string(capture).expect("retained capture is readable");
    let table = text
        .lines()
        .find(|line| line.contains("List of event handler content attributes</caption>"))
        .expect("the capture holds the event handler table");
    let mut rows = table.split("<tr>").skip(2);
    let mut found = Vec::new();
    for row in rows.by_ref() {
        let name = row
            .split("<code>")
            .nth(1)
            .and_then(|rest| rest.split("</code>").next())
            .expect("a row names its attribute");
        let scope = if row.contains(">HTML elements</a>") {
            HtmlEventHandlerApplicability::AllHtmlElements
        } else if row.contains(">body</a>") {
            HtmlEventHandlerApplicability::BodyElement
        } else {
            panic!("row {name} states no known elements")
        };
        found.push((name.to_owned(), scope));
    }
    found
}

#[test]
fn event_handler_vocabulary_equals_the_retained_pinned_list() {
    let mut expected = retained_event_handlers();
    expected.sort_by(|left, right| left.0.cmp(&right.0));
    let mut actual: Vec<_> = KnownHtmlEventHandler::ALL
        .iter()
        .map(|handler| (handler.local_name().to_owned(), handler.applicability()))
        .collect();
    actual.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(expected.len(), 89);
    assert_eq!(actual, expected);
}

#[test]
fn event_handler_recognition_is_exact_and_applies_by_element() {
    let name = |namespace, prefix: Option<&str>, local: &str| {
        QualifiedName::new(namespace, prefix.map(Into::into), local)
    };
    for handler in KnownHtmlEventHandler::ALL {
        let plain = name(Namespace::none(), None, handler.local_name());
        assert_eq!(KnownHtmlEventHandler::from_name(&plain), Some(*handler));
        let prefixed = name(Namespace::none(), Some("p"), handler.local_name());
        assert_eq!(KnownHtmlEventHandler::from_name(&prefixed), Some(*handler));
        let foreign = name(Namespace::svg(), None, handler.local_name());
        assert_eq!(KnownHtmlEventHandler::from_name(&foreign), None);
    }
    // A name that only starts with `on` is not an event handler content attribute.
    for local in [
        "on", "one", "onfoo", "ONCLICK", "onClick", "onclic", "onclickk", "online",
    ] {
        let candidate = name(Namespace::none(), None, local);
        assert_eq!(
            KnownHtmlEventHandler::from_name(&candidate),
            None,
            "{local}"
        );
    }
    let window = KnownHtmlEventHandler::Onafterprint;
    assert!(window.applies_to(KnownHtmlElement::Body));
    assert!(!window.applies_to(KnownHtmlElement::Div));
    let global = KnownHtmlEventHandler::Onclick;
    assert!(global.applies_to(KnownHtmlElement::Div));
    assert!(global.applies_to(KnownHtmlElement::Body));
}

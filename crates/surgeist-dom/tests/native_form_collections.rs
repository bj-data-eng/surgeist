//! Live Form and Fieldset membership over the current canonical tree.

use std::{cell::Cell, sync::atomic::AtomicBool};

use surgeist_dom::{
    Dom, DomBatch, DomError,
    edit::CreatedNode,
    form::{DomFormNamedValue, DomHtmlFormControlsCollection},
    identity::NodeId,
    limits::{DomLimits, DomNameLimits, DomResource},
    tree::{DomHtmlCollection, DomHtmlCollectionLimits},
};
use surgeist_html::values::{DocumentString, Namespace, TextBlock};

const READ_LIMITS: DomHtmlCollectionLimits = DomHtmlCollectionLimits::new(100_000, 32, 4_096);

fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            128,
            16_384,
            256,
            0,
            0,
            16,
            131_072,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn append(
    batch: &mut DomBatch,
    document: &CreatedNode,
    parent: &CreatedNode,
    name: &str,
) -> CreatedNode {
    let child = batch.create_element_ns(document, Namespace::html(), name);
    batch.insert_before(&child, parent, None);
    child
}

fn assert_form(dom: &Dom, collection: &DomHtmlFormControlsCollection, expected: &[NodeId]) {
    let cancel = AtomicBool::new(false);
    let read = dom
        .read_html_form_controls_collection(
            collection,
            READ_LIMITS,
            || Ok::<_, DomError>(()),
            &cancel,
        )
        .unwrap();
    assert_eq!(read.length().unwrap(), expected.len());
    for (index, &member) in expected.iter().enumerate() {
        assert_eq!(read.item(index).unwrap(), Some(member));
    }
    assert_eq!(read.item(expected.len()).unwrap(), None);
}

fn assert_fieldset(dom: &Dom, collection: &DomHtmlCollection, expected: &[NodeId]) {
    let cancel = AtomicBool::new(false);
    let read = dom
        .read_html_collection(collection, READ_LIMITS, || Ok::<_, DomError>(()), &cancel)
        .unwrap();
    assert_eq!(read.length().unwrap(), expected.len());
    for (index, &member) in expected.iter().enumerate() {
        assert_eq!(read.item(index).unwrap(), Some(member));
    }
    assert_eq!(read.item(expected.len()).unwrap(), None);
}

#[test]
fn live_form_ownership_and_fieldset_listedness_follow_distinct_rules() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = append(&mut batch, &doc, &doc, "div");
    let external = append(&mut batch, &doc, &root, "button");
    batch.set_attribute(&external, "form", "main");
    batch.set_attribute(&external, "id", "external-id");
    batch.set_attribute(&external, "name", "external-name");
    let form = append(&mut batch, &doc, &root, "form");
    batch.set_attribute(&form, "id", "main");
    let fieldset = append(&mut batch, &doc, &form, "fieldset");
    batch.set_attribute(&fieldset, "id", "group");
    let image = append(&mut batch, &doc, &fieldset, "input");
    batch.set_attribute(&image, "type", "image");
    batch.set_attribute(&image, "name", "picture");
    let hidden = append(&mut batch, &doc, &fieldset, "p:input");
    batch.set_attribute(&hidden, "type", "hidden");
    batch.set_attribute(&hidden, "name", "hidden");
    let legacy = append(&mut batch, &doc, &fieldset, "img");
    batch.set_attribute(&legacy, "name", "legacy");
    let other_form = append(&mut batch, &doc, &root, "form");
    batch.set_attribute(&other_form, "id", "other");
    let redirected = append(&mut batch, &doc, &fieldset, "output");
    batch.set_attribute(&redirected, "form", "other");
    let commit = dom.apply(batch, &cancel).unwrap();
    let [external, form, fieldset, image, hidden, redirected] =
        [external, form, fieldset, image, hidden, redirected]
            .map(|ticket| commit.created(&ticket).unwrap().unwrap());

    let controls = dom.form_elements(form, &cancel).unwrap();
    let descendants = dom.fieldset_elements(fieldset, &cancel).unwrap();
    assert!(matches!(
        dom.form_elements(fieldset, &cancel),
        Err(DomError::WrongNodeKind)
    ));
    assert_form(&dom, &controls, &[external, fieldset, hidden]);
    assert_fieldset(&dom, &descendants, &[image, hidden, redirected]);
    let read = dom
        .read_html_form_controls_collection(
            &controls,
            READ_LIMITS,
            || Ok::<_, DomError>(()),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        read.supported_names().unwrap().as_slice(),
        &["external-id", "external-name", "group", "hidden"].map(DocumentString::from)
    );
    let short_names = dom
        .read_html_form_controls_collection(
            &controls,
            DomHtmlCollectionLimits::new(100_000, 3, 4_096),
            || Ok::<_, DomError>(()),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        short_names.supported_names(),
        Err(DomError::ResourceLimit {
            resource: DomResource::HtmlCollectionNames,
            limit: 3,
        })
    ));

    let work = Cell::new(0);
    let measured = dom
        .read_html_form_controls_collection(
            &controls,
            READ_LIMITS,
            || {
                work.set(work.get() + 1);
                Ok::<_, DomError>(())
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(measured.item(0).unwrap(), Some(external));
    let prefix = work.get();
    for (limit, expected) in [
        (prefix, Ok(Some(external))),
        (
            prefix - 1,
            Err(DomError::WorkLimit {
                resource: DomResource::HtmlCollectionWork,
                limit: prefix - 1,
            }),
        ),
    ] {
        let read = dom
            .read_html_form_controls_collection(
                &controls,
                DomHtmlCollectionLimits::new(limit, 32, 4_096),
                || Ok::<_, DomError>(()),
                &cancel,
            )
            .unwrap();
        assert_eq!(read.item(0), expected);
    }

    let mut batch = DomBatch::new();
    batch.set_attribute(image, "type", "text");
    batch.set_attribute(external, "form", "missing");
    dom.apply(batch, &cancel).unwrap();
    assert_form(&dom, &controls, &[fieldset, image, hidden]);
    assert_fieldset(&dom, &descendants, &[image, hidden, redirected]);

    let mut batch = DomBatch::new();
    batch.detach(fieldset);
    dom.apply(batch, &cancel).unwrap();
    assert_form(&dom, &controls, &[]);
    assert_fieldset(&dom, &descendants, &[image, hidden, redirected]);
}

#[test]
fn retained_form_collection_requalifies_the_current_root_and_generation() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    let first_root = append(&mut batch, &first, &first, "div");
    let second_root = append(&mut batch, &second, &second, "div");
    let form = append(&mut batch, &first, &first_root, "form");
    batch.set_attribute(&form, "id", "owner");
    let first_control = append(&mut batch, &first, &first_root, "input");
    batch.set_attribute(&first_control, "form", "owner");
    let second_control = append(&mut batch, &second, &second_root, "button");
    batch.set_attribute(&second_control, "form", "owner");
    let commit = dom.apply(batch, &cancel).unwrap();
    let [form, first_control, second_control, second_root] =
        [form, first_control, second_control, second_root]
            .map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let collection = dom.form_elements(form, &cancel).unwrap();
    assert_form(&dom, &collection, &[first_control]);

    let mut batch = DomBatch::new();
    batch.insert_before(form, second_root, None);
    dom.apply(batch, &cancel).unwrap();
    assert_form(&dom, &collection, &[second_control]);

    let mut batch = DomBatch::new();
    batch.detach(form);
    batch.destroy(form);
    dom.apply(batch, &cancel).unwrap();
    assert!(matches!(
        dom.read_html_form_controls_collection(
            &collection,
            READ_LIMITS,
            || Ok::<_, DomError>(()),
            &cancel
        ),
        Err(DomError::StaleNode)
    ));
}

#[test]
fn named_item_retains_live_exact_matches_and_reads_the_first_checked_radio_value() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let form = append(&mut batch, &document, &document, "form");
    let first = append(&mut batch, &document, &form, "input");
    let second = append(&mut batch, &document, &form, "input");
    let checkbox = append(&mut batch, &document, &form, "input");
    for radio in [&first, &second] {
        batch.set_attribute(radio, "type", "radio");
        batch.set_attribute(radio, "name", "choice");
    }
    batch.set_attribute(&first, "id", "choice");
    batch.set_attribute(&first, "checked", "");
    batch.set_attribute(&first, "value", "");
    batch.set_attribute(&checkbox, "type", "checkbox");
    batch.set_attribute(&checkbox, "name", "choice");
    let commit = dom.apply(batch, &cancel).unwrap();
    let [form, first, second, checkbox] =
        [form, first, second, checkbox].map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let controls = dom.form_elements(form, &cancel).unwrap();
    let lookup = |dom: &Dom, name: &DocumentString, retained| {
        dom.read_html_form_controls_collection(
            &controls,
            READ_LIMITS,
            || Ok::<_, DomError>(()),
            &cancel,
        )
        .unwrap()
        .named_item(name, retained)
    };
    assert!(matches!(
        lookup(&dom, &"".into(), 0).unwrap(),
        DomFormNamedValue::Absent
    ));
    assert!(matches!(
        lookup(&dom, &"Choice".into(), 0).unwrap(),
        DomFormNamedValue::Absent
    ));
    let block = TextBlock::from_utf16([0, 0x63, 0x68, 0x6f, 0x69, 0x63, 0x65, 0]);
    let name = block.string(1..7).unwrap();
    assert_eq!(name.retained_units(), 8);
    assert!(matches!(
        lookup(&dom, &name, 7),
        Err(DomError::ResourceLimit {
            resource: DomResource::HtmlCollectionQueryUnits,
            limit: 7
        })
    ));
    let DomFormNamedValue::List(list) = lookup(&dom, &name, 8).unwrap() else {
        panic!("the named lookup should retain a live list");
    };
    let assert_list = |dom: &Dom, expected: &[NodeId], value: &[u16]| {
        let read = dom
            .read_radio_node_list(&list, READ_LIMITS, || Ok::<_, DomError>(()), &cancel)
            .unwrap();
        assert_eq!(read.length().unwrap(), expected.len());
        for (index, &member) in expected.iter().enumerate() {
            assert_eq!(read.item(index).unwrap(), Some(member));
        }
        assert_eq!(read.item(expected.len()).unwrap(), None);
        assert_eq!(read.value().unwrap(), value);
    };
    assert_list(&dom, &[first, second, checkbox], &[]);

    let mut batch = DomBatch::new();
    batch.set_input_checked(second, true);
    dom.apply(batch, &cancel).unwrap();
    assert_list(&dom, &[first, second, checkbox], &[0x6f, 0x6e]);

    let mut batch = DomBatch::new();
    batch.set_attribute(first, "name", "separate-group");
    batch.set_input_checked(first, true);
    dom.apply(batch, &cancel).unwrap();
    assert_list(&dom, &[first, second, checkbox], &[]);

    let mut batch = DomBatch::new();
    batch.remove_attribute_ns(first, Namespace::none(), "id");
    batch.set_attribute(first, "name", "different");
    batch.set_attribute(second, "name", "different");
    dom.apply(batch, &cancel).unwrap();
    assert_list(&dom, &[checkbox], &[]);
    assert!(matches!(
        lookup(&dom, &"choice".into(), 0).unwrap(),
        DomFormNamedValue::Element(node) if node == checkbox
    ));
    assert_eq!(
        dom.snapshot()
            .node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        0
    );
}

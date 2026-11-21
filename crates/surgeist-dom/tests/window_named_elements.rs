//! Native Window element candidates from the pinned Window and HTMLCollection rules.

use std::{cell::Cell, sync::atomic::AtomicBool};
use surgeist_dom::{
    Dom, DomBatch, DomError,
    identity::DocumentId,
    limits::{DomLimits, DomNameLimits, DomResource},
    shadow::DomShadowRootInit,
    tree::{
        DomHtmlCollection, DomHtmlCollectionLimits, DomHtmlCollectionRead, DomWindowNamedElements,
    },
};
use surgeist_html::{
    construction::HtmlShadowRootMode,
    values::{DocumentString, Namespace, TextBlock},
};

const READ: DomHtmlCollectionLimits = DomHtmlCollectionLimits::new(100_000, 32, 8192);

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            256,
            65_536,
            128,
            0,
            0,
            8,
            32_768,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn resolve(dom: &Dom, document: DocumentId, name: DocumentString) -> DomWindowNamedElements {
    dom.window_named_elements(
        document,
        name,
        8192,
        100_000,
        || Ok::<_, DomError>(()),
        &AtomicBool::new(false),
    )
    .unwrap()
}

fn collection(result: DomWindowNamedElements) -> DomHtmlCollection {
    match result {
        DomWindowNamedElements::Collection(collection) => Some(collection),
        _ => None,
    }
    .expect("multiple independent matching Elements")
}

fn read<'a>(
    dom: &'a Dom,
    collection: &'a DomHtmlCollection,
    cancel: &'a AtomicBool,
) -> DomHtmlCollectionRead<'a, impl Fn() -> Result<(), DomError>> {
    dom.read_html_collection(collection, READ, || Ok(()), cancel)
        .unwrap()
}

#[test]
fn window_uses_exact_null_namespace_attributes_and_ordinary_tree_membership() {
    let mut dom = owner();
    let cancel = AtomicBool::new(false);
    let name = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    let mut edit = DomBatch::new();
    let document = edit.create_document();
    let other_document = edit.create_document();
    let root = edit.create_element(&document, "root");
    let other_root = edit.create_element(&other_document, "root");
    edit.set_attribute(&other_root, "id", name.clone());
    edit.insert_before(&other_root, &other_document, None);
    edit.insert_before(&root, &document, None);
    let image = edit.create_element_ns(&document, Namespace::html(), "p:img");
    edit.set_attribute(&image, "id", name.clone());
    edit.set_attribute(&image, "name", name.clone());
    let div = edit.create_element_ns(&document, Namespace::html(), "div");
    edit.set_attribute(&div, "name", name.clone());
    let foreign = edit.create_element_ns(&document, Namespace::svg(), "embed");
    edit.set_attribute(&foreign, "name", name.clone());
    let uppercase = edit.create_element_ns(&document, Namespace::html(), "IMG");
    edit.set_attribute(&uppercase, "name", name.clone());
    let scoped = edit.create_element_ns(&document, Namespace::html(), "img");
    edit.set_attribute_ns(&scoped, Namespace::svg(), "q:id", name.clone());
    edit.set_attribute_ns(&scoped, Namespace::svg(), "q:name", name.clone());
    edit.set_attribute_ns(&scoped, Namespace::none(), "ID", name.clone());
    edit.set_attribute(&scoped, "id", "");
    edit.set_attribute(&scoped, "name", "");
    let detached = edit.create_element(&document, "detached");
    edit.set_attribute(&detached, "id", name.clone());
    let shadow = edit.attach_shadow(&div, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let hidden = edit.create_element(&document, "hidden");
    edit.set_attribute(&hidden, "id", name.clone());
    edit.insert_before(&hidden, &shadow, None);
    let template = edit.create_element_ns(&document, Namespace::html(), "template");
    for child in [&image, &div, &foreign, &uppercase, &scoped, &template] {
        edit.insert_before(child, &root, None);
    }
    let done = dom.apply(edit, &cancel).unwrap();
    let document = done.document(&document).unwrap();
    let image = done.created(&image).unwrap().unwrap();
    let foreign = done.created(&foreign).unwrap().unwrap();
    let template = done.created(&template).unwrap().unwrap();

    let snapshot = dom.snapshot();
    let contents = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let contents_document = snapshot.node(contents).unwrap().owner_document().unwrap();
    let mut edit = DomBatch::new();
    let stored = edit.create_element(contents_document, "stored");
    edit.set_attribute(&stored, "id", name.clone());
    edit.insert_before(&stored, contents, None);
    dom.apply(edit, &cancel).unwrap();

    assert!(
        matches!(resolve(&dom, document, name.clone()), DomWindowNamedElements::Element(id) if id == image)
    );
    assert!(matches!(
        resolve(&dom, document, "".into()),
        DomWindowNamedElements::Absent
    ));
    let nearby = DocumentString::from_utf16(vec![0xd800, 0, 0xdc01]);
    assert!(matches!(
        resolve(&dom, document, nearby),
        DomWindowNamedElements::Absent
    ));

    let mut edit = DomBatch::new();
    edit.remove_attribute(image, "id");
    dom.apply(edit, &cancel).unwrap();
    assert!(matches!(
        resolve(&dom, document, name.clone()),
        DomWindowNamedElements::Element(id) if id == image
    ));

    let mut edit = DomBatch::new();
    edit.set_attribute(foreign, "id", name.clone());
    dom.apply(edit, &cancel).unwrap();
    let live = collection(resolve(&dom, document, name));
    let read = dom
        .read_html_collection(&live, READ, || Ok::<_, DomError>(()), &cancel)
        .unwrap();
    assert_eq!(read.length().unwrap(), 2);
    assert_eq!(read.item(0).unwrap(), Some(image));
    assert_eq!(read.item(1).unwrap(), Some(foreign));
}

#[test]
fn a_window_collection_keeps_its_filter_and_ordinary_collection_aliases_across_edits() {
    let mut dom = owner();
    let cancel = AtomicBool::new(false);
    let mut edit = DomBatch::new();
    let document = edit.create_document();
    let root = edit.create_element(&document, "root");
    edit.insert_before(&root, &document, None);
    let id_match = edit.create_element_ns(&document, Namespace::html(), "div");
    let eligible = ["form", "img", "p:img"]
        .map(|local| edit.create_element_ns(&document, Namespace::html(), local));
    edit.set_attribute(&id_match, "id", "target");
    edit.set_attribute(&id_match, "name", "collection-alias");
    edit.set_attribute(&eligible[0], "name", "target");
    edit.insert_before(&id_match, &root, None);
    for child in &eligible {
        edit.insert_before(child, &root, None);
    }
    let done = dom.apply(edit, &cancel).unwrap();
    let document = done.document(&document).unwrap();
    let id_match = done.created(&id_match).unwrap().unwrap();
    let eligible = eligible.map(|ticket| done.created(&ticket).unwrap().unwrap());

    assert!(matches!(
        resolve(&dom, document, "collection-alias".into()),
        DomWindowNamedElements::Absent
    ));
    let live = collection(resolve(&dom, document, "target".into()));
    {
        let selected = read(&dom, &live, &cancel);
        assert_eq!(selected.length().unwrap(), 2);
        assert_eq!(selected.item(0).unwrap(), Some(id_match));
        assert_eq!(selected.item(1).unwrap(), Some(eligible[0]));
        assert_eq!(
            selected.named_item(&"collection-alias".into()).unwrap(),
            Some(id_match)
        );
        assert_eq!(
            selected.supported_names().unwrap().as_slice(),
            ["target".into(), "collection-alias".into()]
        );
    }

    let mut edit = DomBatch::new();
    edit.remove_attribute(eligible[0], "name");
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(read(&dom, &live, &cancel).length().unwrap(), 1);
    assert!(
        matches!(resolve(&dom, document, "target".into()), DomWindowNamedElements::Element(id) if id == id_match)
    );

    let mut edit = DomBatch::new();
    edit.remove_attribute(id_match, "id");
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(read(&dom, &live, &cancel).length().unwrap(), 0);
    assert!(matches!(
        resolve(&dom, document, "target".into()),
        DomWindowNamedElements::Absent
    ));

    let mut edit = DomBatch::new();
    for node in &eligible[1..] {
        edit.set_attribute(*node, "name", "target");
    }
    dom.apply(edit, &cancel).unwrap();
    let selected = read(&dom, &live, &cancel);
    assert_eq!(selected.length().unwrap(), 2);
    assert_eq!(selected.item(0).unwrap(), Some(eligible[1]));
    assert_eq!(selected.item(1).unwrap(), Some(eligible[2]));
}

#[derive(Debug, PartialEq, Eq)]
enum ReadError {
    Native(DomError),
    Refused,
}

impl From<DomError> for ReadError {
    fn from(error: DomError) -> Self {
        Self::Native(error)
    }
}

#[test]
fn window_resolution_stops_at_the_second_match_and_admits_exact_backing() {
    let mut dom = owner();
    let cancel = AtomicBool::new(false);
    let mut edit = DomBatch::new();
    let document = edit.create_document();
    let root = edit.create_element(&document, "root");
    let children = ["first", "second", "later"].map(|local| edit.create_element(&document, local));
    for node in &children {
        edit.set_attribute(node, "id", "x");
        edit.insert_before(node, &root, None);
    }
    edit.insert_before(&root, &document, None);
    let done = dom.apply(edit, &cancel).unwrap();
    let document = done.document(&document).unwrap();
    let [_, _, later] = children.map(|ticket| done.created(&ticket).unwrap().unwrap());
    let block = TextBlock::from_utf16(vec![u16::from(b'x'); 32]);
    let operand = block.string(0..1).unwrap();
    assert!(matches!(
        dom.window_named_elements(
            document,
            operand.clone(),
            31,
            100_000,
            || Ok::<_, DomError>(()),
            &cancel
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::HtmlCollectionQueryUnits,
            limit: 31
        })
    ));
    let measure = |dom: &Dom| {
        let steps = Cell::new(0);
        let result = dom
            .window_named_elements(
                document,
                operand.clone(),
                32,
                100_000,
                || {
                    steps.set(steps.get() + 1);
                    Ok::<_, DomError>(())
                },
                &cancel,
            )
            .unwrap();
        assert!(matches!(result, DomWindowNamedElements::Collection(_)));
        steps.get()
    };
    let exact = measure(&dom);
    let mut edit = DomBatch::new();
    edit.set_attribute(later, "id", "another");
    edit.set_attribute(later, "name", "many ignored units");
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(measure(&dom), exact);
    assert!(matches!(
        dom.window_named_elements(document, operand.clone(), 32, exact - 1, || Ok::<_, DomError>(()), &cancel),
        Err(DomError::WorkLimit { resource: DomResource::HtmlCollectionWork, limit }) if limit == exact - 1
    ));
    let steps = Cell::new(0);
    let refused = dom.window_named_elements(
        document,
        operand.clone(),
        32,
        exact,
        || {
            steps.set(steps.get() + 1);
            if steps.get() == exact {
                Err(ReadError::Refused)
            } else {
                Ok(())
            }
        },
        &cancel,
    );
    assert_eq!(refused.unwrap_err(), ReadError::Refused);
    let live = collection(
        dom.window_named_elements(
            document,
            operand,
            32,
            exact,
            || Ok::<_, DomError>(()),
            &cancel,
        )
        .unwrap(),
    );
    let selected = dom
        .read_html_collection(
            &live,
            DomHtmlCollectionLimits::new(100_000, 1, 0),
            || Ok::<_, DomError>(()),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        selected.supported_names(),
        Err(DomError::ResourceLimit {
            resource: DomResource::HtmlCollectionNameUnits,
            limit: 0
        })
    ));
    assert_eq!(selected.length().unwrap(), 2);
}

//! Independent DOM3071 collection, qualified-name, and ordinary-tree oracles.

use std::{cell::Cell, sync::atomic::AtomicBool};

use surgeist_dom::{
    Dom, DomBatch, DomError,
    identity::DocumentId,
    limits::{DomLimits, DomNameLimits, DomResource},
    shadow::DomShadowRootInit,
    tree::{DomHtmlCollection, DomHtmlCollectionLimits, DomHtmlCollectionRead},
};
use surgeist_html::{
    construction::HtmlShadowRootMode,
    values::{DocumentString, Namespace, TextBlock},
};

const COLLECTION: DomHtmlCollectionLimits = DomHtmlCollectionLimits::new(200_000, 32, 8192);

fn dom() -> Dom {
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

fn read<'a>(
    dom: &'a Dom,
    collection: &'a DomHtmlCollection,
    cancel: &'a AtomicBool,
) -> DomHtmlCollectionRead<'a, impl Fn() -> Result<(), DomError>> {
    dom.read_html_collection(collection, COLLECTION, || Ok(()), cancel)
        .unwrap()
}

fn query(dom: &Dom, document: DocumentId, tag: &str) -> DomHtmlCollection {
    dom.get_elements_by_tag_name(document, tag.into(), 1024, &AtomicBool::new(false))
        .unwrap()
}

#[test]
fn tag_name_filter_distinguishes_html_spelling_foreign_names_and_ordinary_edges() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let xml = batch.create_document();
    let html = batch.create_html_document(&xml, None);
    let documents = dom.apply(batch, &cancel).unwrap();
    let xml = documents.document(&xml).unwrap();
    let html = documents.document(&html).unwrap();
    let snapshot = dom.snapshot();
    let html_root = snapshot.document(html).unwrap().document_element().unwrap();
    let html_children = snapshot.node(html_root).unwrap().children();
    let [head, body] = *html_children else {
        panic!("native createHTMLDocument establishes its own head and body");
    };

    let mut batch = DomBatch::new();
    let html_lower = batch.create_element_ns(html, Namespace::html(), "div");
    let html_prefixed = batch.create_element_ns(html, Namespace::html(), "p:div");
    let svg_exact = batch.create_element_ns(html, Namespace::svg(), "DIV");
    let svg_lower = batch.create_element_ns(html, Namespace::svg(), "div");
    let host = batch.create_element_ns(html, Namespace::html(), "section");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let hosted = batch.create_element_ns(html, Namespace::html(), "div");
    let detached = batch.create_element_ns(html, Namespace::html(), "div");
    let fragment = batch.create_fragment(html);
    let xml_root = batch.create_element(xml, "root");
    let xml_lower = batch.create_element_ns(xml, Namespace::html(), "div");
    let xml_upper = batch.create_element_ns(xml, Namespace::html(), "DIV");
    for child in [&html_lower, &html_prefixed, &svg_exact, &svg_lower, &host] {
        batch.insert_before(child, body, None);
    }
    batch.insert_before(&hosted, &shadow, None);
    batch.insert_before(&detached, &fragment, None);
    batch.insert_before(&xml_root, xml, None);
    batch.insert_before(&xml_lower, &xml_root, None);
    batch.insert_before(&xml_upper, &xml_root, None);
    let created = dom.apply(batch, &cancel).unwrap();
    let id = |ticket| created.created(ticket).unwrap().unwrap();
    let html_lower = id(&html_lower);
    let html_prefixed = id(&html_prefixed);
    let svg_exact = id(&svg_exact);
    let svg_lower = id(&svg_lower);
    let host = id(&host);
    let xml_upper = id(&xml_upper);

    let html_upper = query(&dom, html, "DIV");
    let selected = read(&dom, &html_upper, &cancel);
    assert_eq!(selected.length().unwrap(), 2);
    assert_eq!(selected.item(0).unwrap(), Some(html_lower));
    assert_eq!(selected.item(1).unwrap(), Some(svg_exact));
    assert_eq!(selected.item(2).unwrap(), None);
    assert_eq!(
        read(&dom, &query(&dom, html, "P:DIV"), &cancel)
            .item(0)
            .unwrap(),
        Some(html_prefixed)
    );
    let xml_uppercase = query(&dom, xml, "DIV");
    let xml_only = read(&dom, &xml_uppercase, &cancel);
    assert_eq!(xml_only.length().unwrap(), 1);
    assert_eq!(xml_only.item(0).unwrap(), Some(xml_upper));
    let all = query(&dom, html, "*");
    let all = read(&dom, &all, &cancel);
    let expected = [
        html_root,
        head,
        body,
        html_lower,
        html_prefixed,
        svg_exact,
        svg_lower,
        host,
    ];
    assert_eq!(all.length().unwrap(), expected.len());
    for (index, node) in expected.into_iter().enumerate() {
        assert_eq!(all.item(index).unwrap(), Some(node));
    }
}

#[test]
fn named_items_and_supported_names_follow_current_tree_order_and_exact_values() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element(&document, "root");
    let early = batch.create_element_ns(&document, Namespace::html(), "entry");
    let late = batch.create_element_ns(&document, Namespace::svg(), "entry");
    let last = batch.create_element_ns(&document, Namespace::html(), "entry");
    let isolated_surrogate = DocumentString::from_utf16(vec![0xd800, u16::from(b'x')]);
    batch.set_attribute(&early, "id", "first");
    batch.set_attribute(&early, "name", "same");
    batch.set_attribute(&late, "id", "same");
    batch.set_attribute(&late, "name", "foreign-name");
    batch.set_attribute(&last, "id", "");
    batch.set_attribute(&last, "name", isolated_surrogate.clone());
    batch.insert_before(&parent, &document, None);
    for child in [&early, &late, &last] {
        batch.insert_before(child, &parent, None);
    }
    let created = dom.apply(batch, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let id = |ticket| created.created(ticket).unwrap().unwrap();
    let parent = id(&parent);
    let early = id(&early);
    let late = id(&late);
    let last = id(&last);
    let collection = query(&dom, document, "entry");

    let read_names = |dom: &Dom| read(dom, &collection, &cancel).supported_names().unwrap();
    {
        let selected = read(&dom, &collection, &cancel);
        assert_eq!(selected.length().unwrap(), 3);
        assert_eq!(selected.named_item(&"same".into()).unwrap(), Some(early));
        assert_eq!(selected.named_item(&"foreign-name".into()).unwrap(), None);
        assert_eq!(selected.named_item(&"".into()).unwrap(), None);
        assert_eq!(
            selected.named_item(&isolated_surrogate).unwrap(),
            Some(last)
        );
        assert_eq!(
            selected.supported_names().unwrap().as_slice(),
            ["first".into(), "same".into(), isolated_surrogate.clone()]
        );
    }

    let mut edit = DomBatch::new();
    edit.insert_before(late, parent, Some(early.into()));
    edit.set_attribute(early, "name", "revised");
    edit.detach(last);
    dom.apply(edit, &cancel).unwrap();
    {
        let selected = read(&dom, &collection, &cancel);
        assert_eq!(selected.length().unwrap(), 2);
        assert_eq!(selected.item(0).unwrap(), Some(late));
        assert_eq!(selected.named_item(&"same".into()).unwrap(), Some(late));
        assert_eq!(selected.named_item(&isolated_surrogate).unwrap(), None);
        assert_eq!(
            read_names(&dom).as_slice(),
            ["same".into(), "first".into(), "revised".into()]
        );
    }
}

#[test]
fn query_and_read_limits_preserve_the_collection_and_return_the_first_refusal() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "item");
    batch.set_attribute(&root, "id", "name");
    batch.insert_before(&root, &document, None);
    let created = dom.apply(batch, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let root = created.created(&root).unwrap().unwrap();
    let collection = query(&dom, document, "item");
    let large = TextBlock::from_utf16(vec![u16::from(b'*'); 32]);
    let short = large.string(0..1).unwrap();
    assert_eq!(short.len(), 1);
    assert!(matches!(
        dom.get_elements_by_tag_name(document, short, 1, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::HtmlCollectionQueryUnits,
            limit: 1
        }),
    ));

    assert!(matches!(
        dom.get_elements_by_tag_name(document, "item".into(), 3, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::HtmlCollectionQueryUnits,
            limit: 3
        }),
    ));
    assert!(matches!(
        dom.read_html_collection(
            &collection,
            COLLECTION,
            || Ok::<_, DomError>(()),
            &AtomicBool::new(true)
        ),
        Err(DomError::Cancelled),
    ));
    assert!(matches!(
        self::dom().read_html_collection(
            &collection,
            COLLECTION,
            || Ok::<_, DomError>(()),
            &cancel
        ),
        Err(DomError::ForeignOwner),
    ));
    assert_eq!(
        DomError::WorkLimit {
            resource: DomResource::HtmlCollectionWork,
            limit: 0
        },
        dom.read_html_collection(
            &collection,
            DomHtmlCollectionLimits::new(0, 32, 64),
            || Ok::<_, DomError>(()),
            &cancel
        )
        .unwrap_err()
    );
    let limited = dom
        .read_html_collection(
            &collection,
            DomHtmlCollectionLimits::new(10_000, 0, 64),
            || Ok::<_, DomError>(()),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        limited.supported_names(),
        Err(DomError::ResourceLimit {
            resource: DomResource::HtmlCollectionNames,
            limit: 0
        }),
    ));
    assert_eq!(
        read(&dom, &collection, &cancel).item(0).unwrap(),
        Some(root)
    );
}

#[test]
fn item_does_not_inspect_the_rest_of_the_collection_and_retirement_keeps_its_identity() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "item");
    batch.insert_before(&root, &document, None);
    for _ in 0..8 {
        let item = batch.create_element(&document, "item");
        batch.insert_before(&item, &root, None);
    }
    let created = dom.apply(batch, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let root = created.created(&root).unwrap().unwrap();
    let collection = query(&dom, document, "item");
    let spent = Cell::new(0);
    {
        let read = dom
            .read_html_collection(
                &collection,
                COLLECTION,
                || {
                    spent.set(spent.get() + 1);
                    Ok::<_, DomError>(())
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(read.item(0).unwrap(), Some(root));
    }
    let first_item_work = spent.get();
    let limited = DomHtmlCollectionLimits::new(first_item_work, 32, 8192);
    assert_eq!(
        dom.read_html_collection(&collection, limited, || Ok::<_, DomError>(()), &cancel)
            .unwrap()
            .item(0)
            .unwrap(),
        Some(root)
    );
    assert_eq!(
        dom.read_html_collection(&collection, limited, || Ok::<_, DomError>(()), &cancel)
            .unwrap()
            .length(),
        Err(DomError::WorkLimit {
            resource: DomResource::HtmlCollectionWork,
            limit: first_item_work,
        })
    );

    let mut edit = DomBatch::new();
    edit.detach(root);
    edit.destroy(root);
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(read(&dom, &collection, &cancel).length().unwrap(), 0);
    let mut replacement = DomBatch::new();
    let node = replacement.create_element(document, "item");
    replacement.insert_before(&node, document, None);
    let replacement = dom
        .apply(replacement, &cancel)
        .unwrap()
        .created(&node)
        .unwrap()
        .unwrap();
    assert_ne!(replacement, root);
    assert!(matches!(
        dom.snapshot().node(root),
        Err(DomError::StaleNode)
    ));
    assert_eq!(
        read(&dom, &collection, &cancel).item(0).unwrap(),
        Some(replacement)
    );
}

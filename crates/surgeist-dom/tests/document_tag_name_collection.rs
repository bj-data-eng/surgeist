//! Independent DOM3071 collection, qualified-name, and ordinary-tree oracles.

use std::{cell::Cell, sync::atomic::AtomicBool};

use surgeist_dom::{
    Dom, DomBatch, DomError,
    identity::DocumentId,
    limits::{DomLimits, DomNameLimits, DomResource},
    shadow::DomShadowRootInit,
    tree::{
        DomDescendantQueryRoot, DomHtmlAllNamedValue, DomHtmlCollection, DomHtmlCollectionLimits,
        DomHtmlCollectionRead, DomSiblingDirection,
    },
};
use surgeist_html::{
    construction::HtmlShadowRootMode,
    values::{DocumentString, Namespace, TextBlock},
    vocabulary::HtmlDocumentCollection,
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
    dom.get_elements_by_tag_name(
        DomDescendantQueryRoot::Document(document),
        tag.into(),
        1024,
        &AtomicBool::new(false),
    )
    .unwrap()
}

#[test]
fn node_lists_read_current_members_under_exact_work_and_query_limits() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let query = DocumentString::from_utf16(vec![0xd800, 0, 0x4e]);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let comment = batch.create_comment(&document, "child but not named HTML");
    let html = batch.create_element_ns(&document, Namespace::html(), "x-native-name");
    let svg = batch.create_element_ns(&document, Namespace::svg(), "g");
    let namespaced = batch.create_element_ns(&document, Namespace::html(), "span");
    batch.set_attribute(&html, "name", query.clone());
    batch.set_attribute(&svg, "name", query.clone());
    batch.set_attribute_ns(
        &namespaced,
        Namespace::from_uri("urn:witness"),
        "p:name",
        query.clone(),
    );
    batch.insert_before(&root, &document, None);
    for child in [&comment, &html, &svg, &namespaced] {
        batch.insert_before(child, &root, None);
    }
    let receipt = dom.apply(batch, &cancel).unwrap();
    let document = receipt.document(&document).unwrap();
    let root = receipt.created(&root).unwrap().unwrap();
    let comment = receipt.created(&comment).unwrap().unwrap();
    let html = receipt.created(&html).unwrap().unwrap();
    let svg = receipt.created(&svg).unwrap().unwrap();
    let namespaced = receipt.created(&namespaced).unwrap().unwrap();
    assert!(matches!(
        dom.get_elements_by_name(document, query.clone(), 2, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::NodeListQueryUnits,
            limit: 2
        })
    ));
    let names = dom
        .get_elements_by_name(document, query, 3, &cancel)
        .unwrap();
    let children = dom.child_nodes(root, &cancel).unwrap();
    let allow = || Ok::<_, DomError>(());
    let current = dom
        .read_node_list(&children, 10_000, allow, &cancel)
        .unwrap();
    assert_eq!(current.length().unwrap(), 4);
    for (index, node) in [comment, html, svg, namespaced].into_iter().enumerate() {
        assert_eq!(current.item(index).unwrap(), Some(node));
    }
    assert_eq!(current.item(4).unwrap(), None);
    let current = dom.read_node_list(&names, 10_000, allow, &cancel).unwrap();
    assert_eq!(current.item(0).unwrap(), Some(html));
    assert_eq!(current.item(1).unwrap(), None);
    let work = Cell::new(0);
    let count = |limit| {
        work.set(0);
        dom.read_node_list(
            &names,
            limit,
            || {
                work.set(work.get() + 1);
                Ok::<_, DomError>(())
            },
            &cancel,
        )?
        .length()
    };
    assert_eq!(count(10_000).unwrap(), 1);
    let exact = work.get();
    assert_eq!(count(exact).unwrap(), 1);
    assert_eq!(work.get(), exact);
    assert!(matches!(
        count(exact - 1),
        Err(DomError::WorkLimit { resource: DomResource::NodeListWork, limit }) if limit == exact - 1
    ));
    assert_eq!(work.get(), exact - 1);
    assert!(matches!(
        dom.read_node_list(&names, exact, || Err(DomError::AllocationFailed), &cancel),
        Err(DomError::AllocationFailed)
    ));
    assert!(matches!(
        dom.read_node_list(
            &children,
            exact,
            || Err(DomError::AllocationFailed),
            &AtomicBool::new(true)
        ),
        Err(DomError::Cancelled)
    ));
    let foreign = self::dom();
    assert!(matches!(
        foreign.read_node_list(&children, exact, allow, &cancel),
        Err(DomError::ForeignOwner)
    ));

    let mut update = DomBatch::new();
    update.detach(html);
    update.insert_before(svg, root, Some(comment.into()));
    dom.apply(update, &cancel).unwrap();
    let current = dom
        .read_node_list(&children, 10_000, allow, &cancel)
        .unwrap();
    assert_eq!(current.length().unwrap(), 3);
    assert_eq!(current.item(0).unwrap(), Some(svg));
    let current = dom.read_node_list(&names, 10_000, allow, &cancel).unwrap();
    assert_eq!(current.length().unwrap(), 0);
    assert_eq!(current.item(0).unwrap(), None);
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
fn element_rooted_collections_preserve_live_members_and_call_time_tag_casing() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let xml = batch.create_document();
    let html = batch.create_html_document(&xml, None);
    let docs = dom.apply(batch, &cancel).unwrap();
    let xml = docs.document(&xml).unwrap();
    let html = docs.document(&html).unwrap();

    let mut batch = DomBatch::new();
    let root = batch.create_element_ns(html, Namespace::html(), "div");
    let lower = batch.create_element_ns(html, Namespace::html(), "div");
    let text = batch.create_text(html, "skip");
    let foreign = batch.create_element_ns(html, Namespace::from_uri("urn:x"), "p:DIV");
    let nested = batch.create_element_ns(html, Namespace::none(), "DIV");
    batch.set_attribute(&foreign, "id", "foreign");
    batch.insert_before(&nested, &lower, None);
    for child in [&lower, &text, &foreign] {
        batch.insert_before(child, &root, None);
    }
    let created = dom.apply(batch, &cancel).unwrap();
    let id = |ticket| created.created(ticket).unwrap().unwrap();
    let (root, lower, text, foreign, nested) =
        (id(&root), id(&lower), id(&text), id(&foreign), id(&nested));
    let root_query = DomDescendantQueryRoot::Element(root);
    let children = dom.child_elements(root, &cancel).unwrap();
    let old_tag = dom
        .get_elements_by_tag_name(root_query, "DIV".into(), 64, &cancel)
        .unwrap();
    let wildcard = DocumentString::from_utf16([u16::from(b'*')]);
    let local_name = DocumentString::from_utf16(b"DIV".map(u16::from));
    let namespace = DocumentString::from_utf16(b"urn:x".map(u16::from));
    let empty_namespace = TextBlock::from_utf16(vec![0x20; 32]).string(0..0).unwrap();
    let null_names = dom
        .get_elements_by_tag_name_ns(
            root_query,
            Some(empty_namespace),
            wildcard.clone(),
            1,
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        dom.get_elements_by_tag_name_ns(
            root_query,
            Some(namespace.clone()),
            local_name.clone(),
            7,
            &cancel
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::HtmlCollectionQueryUnits,
            limit: 7
        })
    ));
    let exact_namespace = dom
        .get_elements_by_tag_name_ns(root_query, Some(namespace), local_name.clone(), 8, &cancel)
        .unwrap();
    assert_eq!(
        read(&dom, &exact_namespace, &cancel).item(0).unwrap(),
        Some(foreign)
    );
    let expanded = dom
        .get_elements_by_tag_name_ns(root_query, Some(wildcard), local_name, 4, &cancel)
        .unwrap();
    {
        let current = read(&dom, &children, &cancel);
        assert_eq!(current.length().unwrap(), 2);
        assert_eq!(current.item(0).unwrap(), Some(lower));
        assert_eq!(current.last_item().unwrap(), Some(foreign));
        assert_eq!(
            current.named_item(&"foreign".into()).unwrap(),
            Some(foreign)
        );
    }
    assert_eq!(
        read(&dom, &null_names, &cancel).item(0).unwrap(),
        Some(nested)
    );
    {
        let current = read(&dom, &expanded, &cancel);
        assert_eq!(current.item(0).unwrap(), Some(nested));
        assert_eq!(current.item(1).unwrap(), Some(foreign));
    }

    let previous = DomSiblingDirection::Previous;
    let next = DomSiblingDirection::Next;
    let count = Cell::new(0);
    let previous_with_limit = |limit| {
        count.set(0);
        dom.read_element_sibling(
            text,
            previous,
            limit,
            || {
                count.set(count.get() + 1);
                Ok::<_, DomError>(())
            },
            &cancel,
        )
    };
    assert_eq!(previous_with_limit(1000).unwrap(), Some(lower));
    let exact = count.get();
    assert_eq!(previous_with_limit(exact).unwrap(), Some(lower));
    assert!(matches!(
        previous_with_limit(exact - 1),
        Err(DomError::WorkLimit { resource: DomResource::HtmlCollectionWork, limit }) if limit == exact - 1
    ));
    assert_eq!(count.get(), exact - 1);
    assert_eq!(
        dom.read_element_sibling(text, next, 1000, || Ok::<_, DomError>(()), &cancel)
            .unwrap(),
        Some(foreign)
    );

    let mut edit = DomBatch::new();
    edit.adopt(root, xml);
    edit.insert_before(foreign, root, Some(lower.into()));
    dom.apply(edit, &cancel).unwrap();
    let current = read(&dom, &children, &cancel);
    assert_eq!(current.item(0).unwrap(), Some(foreign));
    assert_eq!(current.last_item().unwrap(), Some(lower));
    assert_eq!(read(&dom, &old_tag, &cancel).item(0).unwrap(), Some(lower));
    let new_tag = dom
        .get_elements_by_tag_name(root_query, "DIV".into(), 64, &cancel)
        .unwrap();
    let current = read(&dom, &new_tag, &cancel);
    assert_eq!(current.length().unwrap(), 1);
    assert_eq!(current.item(0).unwrap(), Some(nested));
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
fn all_collection_uses_html_name_eligibility_and_retains_the_full_live_query() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element(&document, "root");
    let frame = batch.create_element_ns(&document, Namespace::html(), "p:frame");
    let svg = batch.create_element_ns(&document, Namespace::svg(), "input");
    let div = batch.create_element_ns(&document, Namespace::html(), "div");
    let exact = DocumentString::from_utf16([0xd800, 0, 0xdc00]);
    batch.set_attribute(&frame, "id", exact.clone());
    batch.set_attribute(&frame, "name", "shared");
    batch.set_attribute(&svg, "id", "shared");
    batch.set_attribute(&svg, "name", "foreign");
    batch.set_attribute(&div, "name", "unsupported");
    batch.insert_before(&parent, &document, None);
    for child in [&frame, &svg, &div] {
        batch.insert_before(child, &parent, None);
    }
    let created = dom.apply(batch, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let frame = created.created(&frame).unwrap().unwrap();
    let svg = created.created(&svg).unwrap().unwrap();
    let all = dom.html_all_collection(document, &cancel).unwrap();
    let mut units = [0; 32];
    units[..6].copy_from_slice(&[0x73, 0x68, 0x61, 0x72, 0x65, 0x64]);
    let block = TextBlock::from_utf16(units);
    let shared = block.string(0..6).unwrap();
    assert_eq!(shared.retained_units(), 32);
    let retained = {
        let view = dom
            .read_html_all_collection(&all, COLLECTION, || Ok::<_, DomError>(()), &cancel)
            .unwrap();
        assert_eq!(view.length().unwrap(), 4);
        assert!(view.has_named_item(&shared).unwrap());
        assert!(!view.has_named_item(&"unsupported".into()).unwrap());
        assert!(!view.has_named_item(&"foreign".into()).unwrap());
        assert_eq!(
            view.supported_names().unwrap().as_slice(),
            [exact.clone(), shared.clone()]
        );
        assert!(
            matches!(view.named_item(&exact, 0), Ok(DomHtmlAllNamedValue::Element(node)) if node == frame)
        );
        assert!(matches!(
            view.named_item(&shared, 31),
            Err(DomError::ResourceLimit {
                resource: DomResource::HtmlCollectionQueryUnits,
                limit: 31,
            })
        ));
        let DomHtmlAllNamedValue::Collection(retained) = view.named_item(&shared, 32).unwrap()
        else {
            panic!("two distinct elements share the name");
        };
        retained
    };
    assert_eq!(read(&dom, &retained, &cancel).item(1).unwrap(), Some(svg));

    let mut edit = DomBatch::new();
    edit.detach(svg);
    dom.apply(edit, &cancel).unwrap();
    let view = dom
        .read_html_all_collection(&all, COLLECTION, || Ok::<_, DomError>(()), &cancel)
        .unwrap();
    assert!(
        matches!(view.named_item(&shared, 0), Ok(DomHtmlAllNamedValue::Element(node)) if node == frame)
    );
    assert_eq!(read(&dom, &retained, &cancel).length().unwrap(), 1);
    assert_eq!(read(&dom, &retained, &cancel).item(0).unwrap(), Some(frame));
}

#[test]
fn document_collection_filters_track_present_attributes_and_current_identity() {
    let foreign_owner = dom();
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let anchor = batch.create_element_ns(&document, Namespace::html(), "p:a");
    let area = batch.create_element_ns(&document, Namespace::html(), "area");
    let foreign = batch.create_element_ns(&document, Namespace::svg(), "a");
    batch.set_attribute(&anchor, "name", "");
    batch.set_attribute(&area, "name", "");
    batch.set_attribute(&foreign, "name", "");
    batch.insert_before(&root, &document, None);
    for child in [&anchor, &area, &foreign] {
        batch.insert_before(child, &root, None);
    }
    let created = dom.apply(batch, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let root = created.created(&root).unwrap().unwrap();
    let anchor = created.created(&anchor).unwrap().unwrap();
    let links = dom
        .document_html_collection(document, HtmlDocumentCollection::Links, &cancel)
        .unwrap();
    let anchors = dom
        .document_html_collection(document, HtmlDocumentCollection::Anchors, &cancel)
        .unwrap();
    let applets = dom
        .document_html_collection(document, HtmlDocumentCollection::Applets, &cancel)
        .unwrap();
    assert_eq!(read(&dom, &links, &cancel).length().unwrap(), 0);
    assert_eq!(read(&dom, &anchors, &cancel).length().unwrap(), 1);
    assert_eq!(read(&dom, &anchors, &cancel).item(0).unwrap(), Some(anchor));
    {
        let empty = read(&dom, &applets, &cancel);
        assert_eq!(empty.length().unwrap(), 0);
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(empty.length(), Err(DomError::Cancelled));
        cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    let mut edit = DomBatch::new();
    edit.remove_attribute(anchor, "name");
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(read(&dom, &anchors, &cancel).length().unwrap(), 0);
    let mut edit = DomBatch::new();
    edit.set_attribute(anchor, "name", "");
    edit.detach(root);
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(read(&dom, &links, &cancel).length().unwrap(), 0);
    assert_eq!(read(&dom, &anchors, &cancel).length().unwrap(), 0);
    assert!(matches!(
        foreign_owner.read_html_collection(&links, COLLECTION, || Ok::<_, DomError>(()), &cancel),
        Err(DomError::ForeignOwner)
    ));
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
        dom.get_elements_by_tag_name(
            DomDescendantQueryRoot::Document(document),
            short,
            1,
            &cancel
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::HtmlCollectionQueryUnits,
            limit: 1
        }),
    ));

    assert!(matches!(
        dom.get_elements_by_tag_name(
            DomDescendantQueryRoot::Document(document),
            "item".into(),
            3,
            &cancel
        ),
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

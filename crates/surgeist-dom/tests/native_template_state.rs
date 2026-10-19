//! Independent HTML877d Template and DOM3071 clone/adopt/fragment oracles.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlCapturedHost, HtmlCapturedNodeKind, HtmlCapturedTemplateCache,
    HtmlCapturedTemplateContents, HtmlTemplateDocumentRole,
};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(nodes: usize, units: usize) -> DomLimits {
    DomLimits::new(
        nodes,
        units,
        1024,
        1024,
        512,
        32,
        262144,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn dom() -> Dom {
    Dom::new(limits(512, 65536)).unwrap()
}
fn apply(dom: &mut Dom, b: DomBatch) -> DomCommit {
    dom.apply(b, &cancel()).unwrap()
}
fn contents(s: &DomSnapshot, t: NodeId) -> NodeId {
    s.node(t)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap()
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId, NodeId, DocumentId) {
    let mut b = DomBatch::new();
    let d = b.create_document();
    let t = b.create_element_ns(&d, Namespace::html(), "template");
    let c = apply(dom, b);
    let d = c.document(&d).unwrap();
    let t = c.created(&t).unwrap().unwrap();
    let s = dom.snapshot();
    let f = contents(&s, t);
    (d, t, f, s.node(f).unwrap().owner_document().unwrap())
}
fn capture(s: &DomSnapshot, d: DocumentId) -> DomHtmlCapture {
    s.html_capture(
        d,
        None,
        HtmlCaptureLimits::new(512, 8192, 512, 65536, 512, 0, 0, 16_777_216),
        DomHtmlCaptureLimits::new(512, 1000000),
        &cancel(),
    )
    .unwrap()
}

#[test]
fn real_contents_cache_defaults_class_and_opaque_origin_have_exact_costs() {
    let mut dom = dom();
    let (d, t, f, i) = setup(&mut dom);
    let s = dom.snapshot();
    assert_eq!(dom.usage().managed_nodes(), 4);
    assert_eq!(dom.usage().string_code_units(), 98); //31 outer+36 name+31 inert
    assert_eq!(
        s.document(d).unwrap().template_role(),
        DomTemplateDocumentRole::Ordinary
    );
    assert_eq!(
        s.document(d).unwrap().associated_inert_template_document(),
        Some(i)
    );
    let inert = s.document(i).unwrap();
    assert_eq!(
        inert.template_role(),
        DomTemplateDocumentRole::InertTemplate
    );
    assert_eq!(inert.associated_inert_template_document(), None);
    let m = inert.metadata();
    assert_eq!(m.class(), DomDocumentClass::Xml);
    assert_eq!(m.content_type(), &DocumentString::from("application/xml"));
    assert_eq!(m.encoding(), &DocumentString::from("UTF-8"));
    assert_eq!(m.url(), &DocumentString::from("about:blank"));
    assert_eq!(m.mode(), HtmlDocumentMode::NoQuirks);
    assert!(!m.allow_declarative_shadow_roots());
    assert_ne!(m.origin(), s.document(d).unwrap().metadata().origin());
    assert_eq!(m.custom_element_registry(), DomRegistryBinding::Null);
    assert_eq!(
        m.document_environment().browsing_context(),
        surgeist_html::HtmlBrowsingContextPresence::Absent
    );
    assert_eq!(s.node(f).unwrap().fragment_host(), Some(t));
    assert_eq!(s.node(f).unwrap().parent(), None);
    assert!(matches!(
        s.node(f).unwrap().kind(),
        DomNodeKind::DocumentFragment
    ));
    assert_eq!(
        {
            let p = s.node(t).unwrap().as_template().unwrap().insertion_state();
            (p.target(), p.start(), p.end())
        },
        (None, None, None)
    );
    let mut b = DomBatch::new();
    let sibling = b.create_element_ns(d, Namespace::html(), "template");
    let nested = b.create_element_ns(i, Namespace::html(), "template");
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    for ticket in [&sibling, &nested] {
        let id = c.created(ticket).unwrap().unwrap();
        assert_eq!(s.node(contents(&s, id)).unwrap().owner_document(), Some(i));
    }
    assert_eq!(dom.usage().managed_nodes(), 8);
    assert_eq!(dom.usage().string_code_units(), 170);
}

#[test]
fn html_marking_does_not_copy_html_document_metadata_or_skeleton() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let xml = b.create_document();
    let html = b.create_html_document(&xml, None);
    let t = b.create_element(&html, "TEMPLATE");
    let other = b.create_element_ns(&xml, Namespace::html(), "p:template");
    let generic = b.create_element(&xml, "template");
    let upper = b.create_element_ns(&xml, Namespace::html(), "Template");
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    let d = c.document(&html).unwrap();
    let t = c.created(&t).unwrap().unwrap();
    let inert = s.node(contents(&s, t)).unwrap().owner_document().unwrap();
    let m = s.document(inert).unwrap().metadata();
    assert_eq!(m.class(), DomDocumentClass::Html);
    assert_eq!(m.content_type(), &DocumentString::from("application/xml"));
    assert_ne!(m.origin(), s.document(d).unwrap().metadata().origin());
    assert!(s.node(inert.node()).unwrap().children().is_empty());
    assert!(
        s.node(c.created(&other).unwrap().unwrap())
            .unwrap()
            .as_template()
            .is_some()
    );
    for id in [&generic, &upper] {
        assert!(
            s.node(c.created(id).unwrap().unwrap())
                .unwrap()
                .as_template()
                .is_none()
        );
    }
}

#[test]
fn ordinary_children_stay_distinct_and_contents_do_not_become_shadow_connected() {
    let mut dom = dom();
    let (d, t, f, i) = setup(&mut dom);
    let mut b = DomBatch::new();
    let ordinary = b.create_text(d, "ordinary");
    let body = b.create_text(d, "body");
    let frame = b.create_element_ns(i, Namespace::html(), "frame");
    let form = b.create_element_ns(i, Namespace::html(), "form");
    let button = b.create_element_ns(i, Namespace::html(), "button");
    b.insert_before(&ordinary, t, None);
    b.insert_before(&body, f, None);
    b.insert_before(&frame, f, None);
    b.insert_before(&form, f, None);
    b.insert_before(&button, &form, None);
    b.insert_before(t, d, None);
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    let ordinary = c.created(&ordinary).unwrap().unwrap();
    let body = c.created(&body).unwrap().unwrap();
    assert_eq!(s.node(t).unwrap().children(), &[ordinary]);
    assert!(s.node(ordinary).unwrap().is_connected());
    assert_eq!(s.node(body).unwrap().owner_document(), Some(i));
    assert!(!s.node(body).unwrap().is_connected());
    assert_eq!(s.node(f).unwrap().shadow_host(), None);
    assert_eq!(s.node(f).unwrap().host_including_parent(), Some(t));
    assert_eq!(s.node(f).unwrap().shadow_including_parent(), None);
    assert_eq!(s.node(body).unwrap().tree_scope().host(), None);
    let form = c.created(&form).unwrap().unwrap();
    let button = c.created(&button).unwrap().unwrap();
    assert_eq!(
        s.node(button)
            .unwrap()
            .as_button()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(form)
    );
    assert!(!s.node(button).unwrap().is_connected());
    assert_eq!(
        s.node(t)
            .unwrap()
            .shadow_including_tree_order()
            .collect::<Vec<_>>(),
        vec![t, ordinary]
    );
    let mut b = DomBatch::new();
    b.insert_before(t, f, None);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), s.stamp());
    assert_eq!(
        s.flat_tree(d, DomSlotQueryLimits::new(64, 128).unwrap(), &cancel())
            .unwrap()
            .relation(body)
            .unwrap(),
        DomFlatRelation::Excluded
    );
    capture(&s, d);
}

#[test]
fn direct_fragment_adoption_and_replacement_preserve_host_and_historical_document() {
    let mut dom = dom();
    let (d, t, f, i) = setup(&mut dom);
    let mut b = DomBatch::new();
    let target = b.create_document();
    let parent = b.create_element(&target, "parent");
    let child = b.create_comment(&target, "old");
    b.insert_before(&child, &parent, None);
    let c = apply(&mut dom, b);
    let target = c.document(&target).unwrap();
    let parent = c.created(&parent).unwrap().unwrap();
    let child = c.created(&child).unwrap().unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.insert_before(f, parent, None);
    assert!(apply(&mut dom, b).changes().changes().is_empty()); //empty transfer leaves root
    assert_eq!(dom.snapshot().node(f).unwrap().owner_document(), Some(i));
    let mut b = DomBatch::new();
    b.replace_child(f, parent, child);
    apply(&mut dom, b);
    assert_eq!(
        dom.snapshot().node(f).unwrap().owner_document(),
        Some(target)
    );
    let mut b = DomBatch::new();
    b.adopt(t, d);
    apply(&mut dom, b);
    assert_eq!(
        dom.snapshot().node(f).unwrap().owner_document(),
        Some(target)
    );
    let mut b = DomBatch::new();
    b.adopt(t, target);
    let change = apply(&mut dom, b);
    let s = dom.snapshot();
    let target_inert = s
        .document(target)
        .unwrap()
        .associated_inert_template_document()
        .unwrap();
    assert_ne!(target_inert, i);
    assert_eq!(s.node(f).unwrap().owner_document(), Some(target_inert));
    assert_eq!(s.node(f).unwrap().fragment_host(), Some(t));
    assert!(
        change
            .changes()
            .changes()
            .contains(&DomChange::NodeDocument(f))
    );
    assert!(
        change
            .changes()
            .changes()
            .contains(&DomChange::TemplateRelations)
    );
    assert_eq!(old.node(f).unwrap().owner_document(), Some(i));
}

#[test]
fn adopted_empty_contents_still_cannot_be_destroyed_and_capture_keeps_actual_relations() {
    let mut dom = dom();
    let (d, t, f, i) = setup(&mut dom);
    let mut b = DomBatch::new();
    let other = b.create_document();
    let c = apply(&mut dom, b);
    let other = c.document(&other).unwrap();
    let mut b = DomBatch::new();
    b.adopt(f, other);
    apply(&mut dom, b);
    let old = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.destroy(f);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::InvalidDestruction
    );
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    assert_eq!(dom.usage(), usage);
    let cap = capture(&old, d);
    let view = cap.capture().node(&cap.subject(t).unwrap()).unwrap();
    let Some(HtmlCapturedTemplateContents::Present(id)) = view.template_contents() else {
        panic!("real contents")
    };
    assert_eq!(cap.node(id.subject()).unwrap(), f);
    let fv = cap.capture().node(&cap.subject(f).unwrap()).unwrap();
    assert_eq!(
        cap.node(fv.node_document().subject()).unwrap(),
        other.node()
    );
    let Some(HtmlCapturedHost::Element(host)) = fv.host() else {
        panic!("real host")
    };
    assert_eq!(cap.node(host.subject()).unwrap(), t);
    let dv = cap.capture().node(&cap.subject(d.node()).unwrap()).unwrap();
    let Some(HtmlCapturedTemplateCache::Document(cache)) = dv.template_document_cache() else {
        panic!("cached owner")
    };
    assert_eq!(cap.node(cache.subject()).unwrap(), i.node());
    let HtmlCapturedNodeKind::Document(facts) = cap
        .capture()
        .node(&cap.subject(i.node()).unwrap())
        .unwrap()
        .kind()
    else {
        panic!("Document")
    };
    assert_eq!(
        facts.template_role(),
        HtmlTemplateDocumentRole::InertTemplate
    );
}

#[test]
fn populated_owned_contents_remain_indestructible_after_actual_adoption_and_emptying() {
    let mut dom = dom();
    let (document, template, fragment, inert) = setup(&mut dom);
    let mut b = DomBatch::new();
    let text = b.create_text(inert, "preserved text");
    let comment = b.create_comment(inert, "preserved comment");
    b.insert_before(&text, fragment, None);
    b.insert_before(&comment, fragment, None);
    let other = b.create_document();
    let receiver = b.create_element(&other, "receiver");
    let commit = apply(&mut dom, b);
    let text = commit.created(&text).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let other = commit.document(&other).unwrap();
    let receiver = commit.created(&receiver).unwrap().unwrap();
    let populated = dom.snapshot();
    assert_eq!(
        populated.node(fragment).unwrap().children(),
        &[text, comment]
    );

    // #1232's native ownership rule follows the actual Template host, not
    // child count, ordinary parentage, or current node-document equality.
    let transferred_children = [text, comment];
    for empty in [false, true] {
        if empty {
            let mut b = DomBatch::new();
            b.adopt(fragment, other);
            apply(&mut dom, b);
            let adopted = dom.snapshot();
            assert_eq!(
                adopted.node(fragment).unwrap().owner_document(),
                Some(other)
            );
            assert_eq!(
                adopted.node(fragment).unwrap().fragment_host(),
                Some(template)
            );
            assert_eq!(adopted.node(fragment).unwrap().children(), &[text, comment]);
            assert_eq!(
                adopted.node(template).unwrap().owner_document(),
                Some(document)
            );
            let mut b = DomBatch::new();
            // Inserting a Fragment transfers its real children; the owned root
            // itself remains parentless and retains its genuine Template host.
            b.insert_before(fragment, receiver, None);
            apply(&mut dom, b);
        }
        let before = dom.snapshot();
        let usage = dom.usage();
        let expected_document = if empty { other } else { inert };
        let expected_children: &[NodeId] = if empty { &[] } else { &[text, comment] };
        assert_eq!(contents(&before, template), fragment);
        assert_eq!(before.node(fragment).unwrap().children(), expected_children);
        assert_eq!(before.node(fragment).unwrap().parent(), None);
        assert_eq!(
            before.node(fragment).unwrap().fragment_host(),
            Some(template)
        );
        assert_eq!(
            before.node(fragment).unwrap().owner_document(),
            Some(expected_document)
        );

        let mut b = DomBatch::new();
        b.destroy(fragment);
        assert_eq!(
            dom.apply(b, &cancel()).unwrap_err(),
            DomError::InvalidDestruction
        );
        let after = dom.snapshot();
        assert_eq!(after.stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        let history = dom.changes_since(before.stamp()).unwrap();
        assert_eq!(history.from(), before.stamp());
        assert_eq!(history.to(), before.stamp());
        assert!(history.changes().is_empty());
        assert_eq!(contents(&after, template), fragment);
        assert_eq!(
            after.node(fragment).unwrap().fragment_host(),
            Some(template)
        );
        assert_eq!(after.node(fragment).unwrap().parent(), None);
        assert_eq!(
            after.node(fragment).unwrap().host_including_parent(),
            Some(template)
        );
        assert_eq!(
            after.node(fragment).unwrap().owner_document(),
            Some(expected_document)
        );
        assert_eq!(after.node(fragment).unwrap().children(), expected_children);
        assert_eq!(
            after.node(template).unwrap().owner_document(),
            Some(document)
        );
        assert_eq!(
            after
                .document(document)
                .unwrap()
                .associated_inert_template_document(),
            Some(inert)
        );
        for id in [
            document.node(),
            inert.node(),
            other.node(),
            template,
            fragment,
            receiver,
            text,
            comment,
        ] {
            assert!(
                after.node(id).is_ok(),
                "direct destroy must not retire {id:?}"
            );
        }
        for child in [text, comment] {
            assert_eq!(
                after.node(child).unwrap().parent(),
                Some(if empty { receiver } else { fragment })
            );
            assert_eq!(
                after.node(child).unwrap().owner_document(),
                Some(expected_document)
            );
        }
        assert_eq!(
            after.node(receiver).unwrap().children(),
            if empty {
                &transferred_children[..]
            } else {
                &[]
            }
        );
        assert_eq!(
            after.node(text).unwrap().data(),
            Some(&DocumentString::from("preserved text"))
        );
        assert_eq!(
            after.node(comment).unwrap().data(),
            Some(&DocumentString::from("preserved comment"))
        );
    }
    // The original populated history retains the same full identities and
    // original inert node Document after the real transfer and both refusals.
    assert_eq!(contents(&populated, template), fragment);
    assert_eq!(
        populated.node(fragment).unwrap().fragment_host(),
        Some(template)
    );
    assert_eq!(
        populated.node(fragment).unwrap().owner_document(),
        Some(inert)
    );
    assert_eq!(
        populated.node(fragment).unwrap().children(),
        &[text, comment]
    );
    assert_eq!(populated.node(text).unwrap().parent(), Some(fragment));
    assert_eq!(populated.node(comment).unwrap().parent(), Some(fragment));
}

#[test]
fn shallow_and_deep_import_preserve_both_lists_and_use_copy_contents_documents() {
    let mut source = dom();
    let (d, t, f, i) = setup(&mut source);
    let mut b = DomBatch::new();
    let ordinary = b.create_text(d, "ordinary");
    let nested = b.create_element_ns(i, Namespace::html(), "template");
    b.insert_before(&ordinary, t, None);
    b.insert_before(&nested, f, None);
    let c = apply(&mut source, b);
    let nested = c.created(&nested).unwrap().unwrap();
    let nf = contents(&source.snapshot(), nested);
    let mut b = DomBatch::new();
    let text = b.create_text(i, DocumentString::from_utf16(vec![0xd800, 65, 0xdc00]));
    b.insert_before(&text, nf, None);
    apply(&mut source, b);
    let old = source.snapshot();
    let mut target = dom();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let shallow = b.import_node(&old, t, &doc, false);
    let deep = b.import_node(&old, t, &doc, true);
    let fragment = b.import_node(&old, f, &doc, true);
    let c = apply(&mut target, b);
    let doc = c.document(&doc).unwrap();
    let s = target.snapshot();
    let shallow = c.created(&shallow).unwrap().unwrap();
    let deep = c.created(&deep).unwrap().unwrap();
    assert!(s.node(shallow).unwrap().children().is_empty());
    assert!(s.node(contents(&s, shallow)).unwrap().children().is_empty());
    let df = contents(&s, deep);
    let owner = s.node(df).unwrap().owner_document().unwrap();
    assert_ne!(owner, doc);
    assert_eq!(
        s.node(contents(&s, shallow)).unwrap().owner_document(),
        Some(owner)
    );
    let ordinary = s.node(deep).unwrap().children()[0];
    assert_eq!(
        s.node(ordinary).unwrap().data(),
        Some(&DocumentString::from("ordinary"))
    );
    assert_eq!(s.node(ordinary).unwrap().owner_document(), Some(doc));
    let nested_copy = s.node(df).unwrap().children()[0];
    let ncf = contents(&s, nested_copy);
    assert_eq!(s.node(ncf).unwrap().owner_document(), Some(owner));
    assert_eq!(
        s.node(s.node(ncf).unwrap().children()[0])
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        &[0xd800, 65, 0xdc00]
    );
    let fragment = c.created(&fragment).unwrap().unwrap();
    assert_eq!(s.node(fragment).unwrap().fragment_host(), None);
    assert_eq!(s.node(fragment).unwrap().owner_document(), Some(doc));
    assert_ne!(nested_copy, nested);
    assert_eq!(old.node(f).unwrap().children(), &[nested]);
    drop(source);
    assert_eq!(
        old.node(t)
            .unwrap()
            .as_template()
            .unwrap()
            .contents()
            .unwrap(),
        f
    );
}

#[test]
fn exact_creation_limits_and_late_failure_never_publish_auxiliary_identities_or_cache() {
    for (nodes, units) in [(3, 98), (4, 97)] {
        let mut dom = Dom::new(limits(nodes, units)).unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let c = apply(&mut dom, b);
        let d = c.document(&d).unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.create_element_ns(d, Namespace::html(), "template");
        assert!(matches!(
            dom.apply(b, &cancel()),
            Err(DomError::ResourceLimit { .. })
        ));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert_eq!(
            dom.snapshot()
                .document(d)
                .unwrap()
                .associated_inert_template_document(),
            None
        );
        dom.set_limits(limits(4, 98)).unwrap();
        let mut b = DomBatch::new();
        let t = b.create_element_ns(d, Namespace::html(), "template");
        let c = apply(&mut dom, b);
        assert!(
            dom.snapshot()
                .node(c.created(&t).unwrap().unwrap())
                .unwrap()
                .as_template()
                .is_some()
        );
        assert_eq!(dom.usage().string_code_units(), 98);
    }
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.create_element_ns(d, Namespace::html(), "template");
    b.insert_before(d, d, None);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage().managed_nodes(), 1);
    assert_eq!(
        dom.snapshot()
            .document(d)
            .unwrap()
            .associated_inert_template_document(),
        None
    );
}

#[test]
fn owner_destruction_retires_nested_contents_but_preserves_shared_cache_and_enclosing_shadow() {
    let mut dom = dom();
    let (d, t, f, i) = setup(&mut dom);
    let mut b = DomBatch::new();
    let sibling = b.create_element_ns(d, Namespace::html(), "template");
    let nested = b.create_element_ns(i, Namespace::html(), "template");
    b.insert_before(&nested, f, None);
    let host = b.create_element_ns(d, Namespace::html(), "div");
    let root = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    b.insert_before(t, &root, None);
    let c = apply(&mut dom, b);
    let nested = c.created(&nested).unwrap().unwrap();
    let sibling = c.created(&sibling).unwrap().unwrap();
    let root = c.created(&root).unwrap().unwrap();
    let old = dom.snapshot();
    let nf = contents(&old, nested);
    let mut b = DomBatch::new();
    b.detach(t);
    b.destroy(t);
    apply(&mut dom, b);
    let s = dom.snapshot();
    for id in [t, f, nested, nf] {
        assert!(matches!(s.node(id), Err(DomError::StaleNode)));
        assert!(old.node(id).is_ok());
    }
    assert!(s.shadow_root(root).is_ok());
    assert_eq!(
        s.document(d).unwrap().associated_inert_template_document(),
        Some(i)
    );
    assert!(s.document(i).is_ok());
    assert!(s.node(sibling).unwrap().as_template().is_some());
    let mut b = DomBatch::new();
    let next = b.create_element_ns(d, Namespace::html(), "template");
    let c = apply(&mut dom, b);
    let next = c.created(&next).unwrap().unwrap();
    assert_ne!(next, t);
    assert_eq!(
        dom.snapshot()
            .node(contents(&dom.snapshot(), next))
            .unwrap()
            .owner_document(),
        Some(i)
    );
}

#[test]
fn template_clone_extension_signals_contents_before_ordinary_children_and_keeps_shadow_hooks() {
    let mut source = dom();
    let (d, t, f, i) = setup(&mut source);
    let mut b = DomBatch::new();
    for (document, parent) in [(i, f), (d, t)] {
        let host = b.create_element_ns(document, Namespace::html(), "div");
        let text = b.create_text(document, "light");
        let root = b.attach_shadow(
            &host,
            HtmlShadowRootProperties::new(
                HtmlShadowRootMode::Open,
                HtmlSlotAssignment::Named,
                false,
                true,
                false,
            ),
        );
        let slot = b.create_element_ns(document, Namespace::html(), "slot");
        b.insert_before(&text, &host, None);
        b.insert_before(&slot, &root, None);
        b.insert_before(&host, parent, None);
    }
    apply(&mut source, b);
    let old = source.snapshot();
    let mut target = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let copy = b.import_node(&old, t, &d, true);
    let c = apply(&mut target, b);
    let copy = c.created(&copy).unwrap().unwrap();
    let s = target.snapshot();
    let cf = contents(&s, copy);
    let content_host = s.node(cf).unwrap().children()[0];
    let ordinary_host = s.node(copy).unwrap().children()[0];
    let mut expected = Vec::new();
    for host in [content_host, ordinary_host] {
        let root = s.node(host).unwrap().hosted_shadow_root().unwrap();
        let slot = s.node(root).unwrap().children()[0];
        let text = s.node(host).unwrap().children()[0];
        assert_eq!(s.slot(slot).unwrap().assigned_nodes(), &[text]);
        assert_eq!(
            s.node(root).unwrap().owner_document(),
            s.node(host).unwrap().owner_document()
        );
        expected.push(slot);
    }
    assert_eq!(s.pending_slot_signals(), expected);
    let mut b = DomBatch::new();
    b.destroy(copy);
    let c = apply(&mut target, b);
    for slot in &expected {
        assert!(matches!(
            target.snapshot().node(*slot),
            Err(DomError::StaleNode)
        ));
        assert!(
            c.changes()
                .changes()
                .contains(&DomChange::SlotSignalCancelled {
                    slot: *slot,
                    reason: DomSlotSignalCancellationReason::Destroyed
                })
        );
    }
    assert!(target.snapshot().pending_slot_signals().is_empty());
    assert_eq!(s.pending_slot_signals(), expected);
}

#[test]
fn nested_content_adoption_preserves_ids_and_resets_only_the_appropriate_document_cause() {
    let mut dom = dom();
    let (d, t, f, i) = setup(&mut dom);
    let mut b = DomBatch::new();
    let nested = b.create_element_ns(i, Namespace::html(), "template");
    b.insert_before(&nested, f, None);
    let c = apply(&mut dom, b);
    let nested = c.created(&nested).unwrap().unwrap();
    let nf = contents(&dom.snapshot(), nested);
    let mut b = DomBatch::new();
    let text = b.create_text(i, "kept");
    b.insert_before(&text, nf, None);
    let d2 = b.create_document();
    let c = apply(&mut dom, b);
    let d2 = c.document(&d2).unwrap();
    let text = c.created(&text).unwrap().unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.adopt(t, d2);
    apply(&mut dom, b);
    let s = dom.snapshot();
    let new = s
        .document(d2)
        .unwrap()
        .associated_inert_template_document()
        .unwrap();
    for id in [f, nested, nf, text] {
        assert_eq!(s.node(id).unwrap().owner_document(), Some(new));
    }
    assert_eq!(s.node(f).unwrap().fragment_host(), Some(t));
    assert_eq!(s.node(nf).unwrap().fragment_host(), Some(nested));
    assert_eq!(old.node(text).unwrap().owner_document(), Some(i));
    assert_eq!(
        s.document(d).unwrap().associated_inert_template_document(),
        Some(i)
    );
    assert_eq!(
        s.document(new)
            .unwrap()
            .associated_inert_template_document(),
        None
    );
    capture(&s, d2);
}

#[test]
fn native_for_and_shadow_attributes_do_not_create_parser_patching_or_shadow_state() {
    let mut dom = dom();
    let (d, t, f, _) = setup(&mut dom);
    let mut b = DomBatch::new();
    let marker = b.create_processing_instruction(d, "marker", "name=\"x\"");
    b.insert_before(&marker, t, None);
    b.set_attribute(t, "for", "x");
    b.set_attribute(t, "shadowrootmode", "open");
    b.set_attribute(t, "shadowrootclonable", "");
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(contents(&s, t), f);
    assert!(s.node(f).unwrap().children().is_empty());
    assert_eq!(
        s.node(t).unwrap().children(),
        &[c.created(&marker).unwrap().unwrap()]
    );
    assert_eq!(s.node(t).unwrap().shadow_root(), None);
    assert_eq!(
        {
            let p = s.node(t).unwrap().as_template().unwrap().insertion_state();
            (p.target(), p.start(), p.end())
        },
        (None, None, None)
    );
    let stamp = s.stamp();
    let mut b = DomBatch::new();
    b.set_attribute(t, "for", "x");
    assert!(apply(&mut dom, b).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
    dom.set_limits(
        DomLimits::new(
            512,
            65536,
            1024,
            1024,
            512,
            0,
            0,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(t, "for", "y");
    apply(&mut dom, b);
    assert!(matches!(
        dom.changes_since(stamp),
        Err(DomHistoryError::FullRecomputeRequired { .. })
    ));
}

#[test]
fn deeply_nested_owned_contents_clone_adopt_and_destroy_without_recursive_walks() {
    let mut dom = Dom::new(limits(2048, 262144)).unwrap();
    let (d, t, f, i) = setup(&mut dom);
    let mut parent = f;
    for _ in 0..130 {
        let mut b = DomBatch::new();
        let child = b.create_element_ns(i, Namespace::html(), "template");
        b.insert_before(&child, parent, None);
        let c = apply(&mut dom, b);
        parent = contents(&dom.snapshot(), c.created(&child).unwrap().unwrap());
    }
    let mut b = DomBatch::new();
    let text = b.create_text(i, "bottom");
    b.insert_before(&text, parent, None);
    let other = b.create_document();
    let c = apply(&mut dom, b);
    let other = c.document(&other).unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    let copy = b.import_node(&old, t, d, true);
    b.adopt(t, other);
    let c = apply(&mut dom, b);
    let copy = c.created(&copy).unwrap().unwrap();
    let s = dom.snapshot();
    let mut current = copy;
    for _ in 0..130 {
        current = s.node(contents(&s, current)).unwrap().children()[0];
    }
    assert_eq!(
        s.node(s.node(contents(&s, current)).unwrap().children()[0])
            .unwrap()
            .data(),
        Some(&DocumentString::from("bottom"))
    );
    let mut b = DomBatch::new();
    b.destroy(t);
    apply(&mut dom, b);
    assert!(matches!(
        dom.snapshot().node(parent),
        Err(DomError::StaleNode)
    ));
    assert!(old.node(parent).is_ok());
    assert!(dom.snapshot().node(copy).is_ok());
}

#[test]
fn temporary_template_retirement_preserves_the_source_created_document_cache() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let t = b.create_element_ns(&d, Namespace::html(), "template");
    b.destroy(&t);
    let c = apply(&mut dom, b);
    assert_eq!(c.created(&t).unwrap(), None);
    let d = c.document(&d).unwrap();
    let s = dom.snapshot();
    let i = s
        .document(d)
        .unwrap()
        .associated_inert_template_document()
        .unwrap();
    assert_eq!(
        s.document(i).unwrap().template_role(),
        DomTemplateDocumentRole::InertTemplate
    );
    assert_eq!(dom.usage().managed_nodes(), 2);
    assert_eq!(dom.usage().string_code_units(), 62);
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::TemplateRelations)
    );
    let mut b = DomBatch::new();
    let next = b.create_element_ns(d, Namespace::html(), "template");
    let c = apply(&mut dom, b);
    let next = c.created(&next).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .node(contents(&dom.snapshot(), next))
            .unwrap()
            .owner_document(),
        Some(i)
    );
}

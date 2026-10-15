//! Actual Frame state follows source Slot insertion/removal and fresh clone steps.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

#[test]
fn frame_slottable_clone_and_adoption_preserve_both_canonical_families() {
    let mut dom = Dom::new(DomLimits::new(32, 8192, 64, 64, 0, 16, 65536).unwrap()).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let target = b.create_document();
    let host = b.create_element_ns(&d, Namespace::html(), "div");
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
    let slot = b.create_element_ns(&d, Namespace::html(), "slot");
    let frame = b.create_element_ns(&d, Namespace::html(), "frame");
    b.set_attribute(&frame, "src", "literal");
    b.insert_before(&slot, &root, None);
    b.insert_before(&frame, &host, None);
    b.insert_before(&host, &d, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let target = c.document(&target).unwrap();
    let host = c.created(&host).unwrap().unwrap();
    let slot = c.created(&slot).unwrap().unwrap();
    let frame = c.created(&frame).unwrap().unwrap();
    let old = dom.snapshot();
    let state = old
        .node(frame)
        .unwrap()
        .as_element()
        .unwrap()
        .frame_null_content()
        .unwrap();
    assert!(state.content_navigable_is_null());
    assert!(state.content_document_is_null());
    assert!(state.content_window_is_null());
    assert_eq!(old.slot(slot).unwrap().assigned_nodes(), &[frame]);
    assert_eq!(
        old.node(frame)
            .unwrap()
            .assigned_slot(&AtomicBool::new(false))
            .unwrap(),
        Some(slot)
    );

    let mut b = DomBatch::new();
    let cloned = b.import_node(&old, host, d, true);
    let c = apply(&mut dom, b);
    let cloned = c.created(&cloned).unwrap().unwrap();
    let s = dom.snapshot();
    let cloned_host = s.node(cloned).unwrap();
    let cloned_frame = cloned_host.children()[0];
    let cloned_root = cloned_host.hosted_shadow_root().unwrap();
    let cloned_slot = s.node(cloned_root).unwrap().children()[0];
    assert_ne!(cloned_frame, frame);
    assert_ne!(cloned_slot, slot);
    assert_eq!(
        s.slot(cloned_slot).unwrap().assigned_nodes(),
        &[cloned_frame]
    );
    let element = s.node(cloned_frame).unwrap().as_element().unwrap();
    assert_eq!(element.frame_null_content(), Some(state));
    assert_eq!(element.attribute(&"src".into()), Some(&"literal".into()));

    let mut b = DomBatch::new();
    b.adopt(frame, target);
    apply(&mut dom, b);
    let s = dom.snapshot();
    let adopted = s.node(frame).unwrap();
    assert_eq!(adopted.parent(), None);
    assert_eq!(adopted.owner_document(), Some(target));
    assert_eq!(
        adopted.as_element().unwrap().frame_null_content(),
        Some(state)
    );
    assert_eq!(
        adopted.assigned_slot(&AtomicBool::new(false)).unwrap(),
        None
    );
    assert_eq!(s.slot(slot).unwrap().assigned_nodes(), &[]);
    assert_eq!(
        s.slot(cloned_slot).unwrap().assigned_nodes(),
        &[cloned_frame]
    );
    assert_eq!(old.slot(slot).unwrap().assigned_nodes(), &[frame]);
    assert_eq!(old.node(frame).unwrap().owner_document(), Some(d));
}

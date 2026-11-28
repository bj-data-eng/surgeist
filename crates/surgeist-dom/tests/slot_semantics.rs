use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    Dom, DomBatch, DomCommit, DomSnapshot,
    capture::{DomHtmlCapture, DomHtmlCaptureError, DomHtmlCaptureLimits, DomSlotSemanticLimits},
    identity::{DocumentId, NodeId},
    limits::DomLimits,
    shadow::{DomFlatRelation, DomShadowRootInit, DomSlotQueryLimits},
};
use surgeist_html::values::HtmlValueInterpreter;
use surgeist_html::{
    construction::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment},
    values::Namespace,
};
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlDirectionality, HtmlInertFactLimits, HtmlLanguageDirectionLimits,
    HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits, HtmlSemanticMissing,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            64,
            16384,
            128,
            128,
            128,
            8,
            65536,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, b: DomBatch) -> DomCommit {
    dom.apply(b, &cancel()).unwrap()
}
fn limits() -> DomSlotSemanticLimits {
    DomSlotSemanticLimits::new(
        DomSlotQueryLimits::new(64, 128).unwrap(),
        DomHtmlCaptureLimits::new(64, 1000000),
        HtmlLanguageDirectionLimits::new(128, 128, 16384, 1000000),
        HtmlInertFactLimits::new(128, 1000000),
    )
}
fn capture(s: &DomSnapshot, d: DocumentId) -> DomHtmlCapture {
    s.html_capture(
        d,
        None,
        HtmlCaptureLimits::new(128, 4096, 256, 32768, 128, 0, 0, 16_777_216),
        DomHtmlCaptureLimits::new(128, 1000000),
        &cancel(),
    )
    .unwrap()
    .with_slot_semantics(limits(), &cancel())
    .unwrap()
}
fn fact(c: &DomHtmlCapture, n: NodeId, q: HtmlSemanticQuery) -> HtmlSemanticFact {
    c.query(
        &[HtmlSemanticRequest::new(c.subject(n).unwrap(), q)],
        HtmlValueInterpreter::new(16384),
        HtmlSemanticLimits::new(1000000, 16, 1024, 256, 1000000, 1024),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .results()[0]
        .fact()
        .clone()
}

#[test]
fn directionality_consumes_stored_history_while_fresh_flat_uses_fallback() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let top = b.create_element_ns(&d, Namespace::html(), "div");
    let h1 = b.create_element_ns(&d, Namespace::html(), "div");
    let h2 = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&top, &d, None);
    b.insert_before(&h1, &top, None);
    b.insert_before(&h2, &top, None);
    let p = HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Open,
        HtmlSlotAssignment::Manual,
        false,
        false,
        false,
    );
    let r1 = b.attach_shadow(&h1, p);
    let r2 = b.attach_shadow(&h2, p);
    let s1 = b.create_element_ns(&d, Namespace::html(), "slot");
    let s2 = b.create_element_ns(&d, Namespace::html(), "slot");
    b.set_attribute(&s1, "dir", "auto");
    b.insert_before(&s1, &r1, None);
    b.insert_before(&s2, &r2, None);
    let x = b.create_text(&d, "\u{05d0}");
    b.insert_before(&x, &h1, None);
    let fallback = b.create_text(&d, "LTR");
    b.insert_before(&fallback, &s1, None);
    b.assign_slot(&s1, vec![(&x).into()]);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let s1 = c.created(&s1).unwrap().unwrap();
    let s2 = c.created(&s2).unwrap().unwrap();
    let x = c.created(&x).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.assign_slot(s2, vec![x.into()]);
    apply(&mut dom, b);
    let snapshot = dom.snapshot();
    let c = capture(&snapshot, d);
    assert_eq!(
        fact(&c, s1, HtmlSemanticQuery::Directionality),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
    );
    assert_eq!(
        snapshot
            .flat_tree(d, DomSlotQueryLimits::new(64, 64).unwrap(), &cancel())
            .unwrap()
            .relation(x)
            .unwrap(),
        DomFlatRelation::Excluded
    );
    assert_eq!(c.node(&c.subject(x).unwrap()).unwrap(), x);
    assert_eq!(c.basis(), snapshot.document(d).unwrap().stamp());
}

#[test]
fn light_and_detached_slots_supply_known_empty_assignments_without_fake_hosts() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let slot = b.create_element_ns(&d, Namespace::html(), "slot");
    let t = b.create_text(&d, "\u{05d0}");
    b.set_attribute(&slot, "dir", "auto");
    b.insert_before(&t, &slot, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let slot = c.created(&slot).unwrap().unwrap();
    let c = capture(&dom.snapshot(), d);
    assert_eq!(
        fact(&c, slot, HtmlSemanticQuery::Directionality),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
    );
}

#[test]
fn inertness_uses_real_slot_flat_ancestry_and_detached_exclusion_stays_missing() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let host = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&host, &d, None);
    let r = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let slot = b.create_element_ns(&d, Namespace::html(), "slot");
    b.set_attribute(&slot, "inert", "");
    b.insert_before(&slot, &r, None);
    let x = b.create_element_ns(&d, Namespace::html(), "span");
    b.insert_before(&x, &host, None);
    let detached = b.create_element_ns(&d, Namespace::html(), "div");
    b.set_attribute(&detached, "inert", "");
    let child = b.create_element_ns(&d, Namespace::html(), "span");
    b.insert_before(&child, &detached, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let slot = c.created(&slot).unwrap().unwrap();
    let x = c.created(&x).unwrap().unwrap();
    let detached = c.created(&detached).unwrap().unwrap();
    let child = c.created(&child).unwrap().unwrap();
    let c = capture(&dom.snapshot(), d);
    assert_eq!(
        c.source()
            .flat_tree(d, DomSlotQueryLimits::new(64, 64).unwrap(), &cancel())
            .unwrap()
            .relation(x)
            .unwrap(),
        DomFlatRelation::Parent(slot)
    );
    assert_eq!(
        fact(&c, slot, HtmlSemanticQuery::Inertness),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        fact(&c, x, HtmlSemanticQuery::Inertness),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        fact(&c, detached, HtmlSemanticQuery::Inertness),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    assert!(matches!(
        fact(&c, child, HtmlSemanticQuery::Inertness),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::InertAncestry(_)
        ))
    ));
}

#[test]
fn mixed_root_and_host_documents_preserve_real_flat_parent_and_known_empty_blocking() {
    use surgeist_dom::{DomError, dialog::DomNearestClickedDialog::Found};
    let mut dom = owner();
    let mut b = DomBatch::new();
    let a = b.create_document();
    let d = b.create_document();
    let open_dialog = b.create_element_ns(&a, Namespace::html(), "dialog");
    let host = b.create_element_ns(&a, Namespace::html(), "div");
    b.set_attribute(&open_dialog, "open", "");
    b.set_attribute(&host, "inert", "");
    b.insert_before(&open_dialog, &a, None);
    b.insert_before(&host, &open_dialog, None);
    let r = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let target = b.create_element(&d, "Target");
    let old = b.create_comment(&d, "old");
    b.insert_before(&target, &d, None);
    b.insert_before(&old, &target, None);
    b.replace_child(&r, &target, &old);
    let slot = b.create_element_ns(&d, Namespace::html(), "slot");
    b.insert_before(&slot, &r, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let open_dialog = c.created(&open_dialog).unwrap().unwrap();
    let host = c.created(&host).unwrap().unwrap();
    let slot = c.created(&slot).unwrap().unwrap();
    let s = dom.snapshot();
    assert_ne!(
        s.node(host).unwrap().owner_document(),
        s.node(slot).unwrap().owner_document()
    );
    let c = capture(&s, d);
    let flat = s
        .flat_tree(
            s.node(host).unwrap().owner_document().unwrap(),
            DomSlotQueryLimits::new(64, 64).unwrap(),
            &cancel(),
        )
        .unwrap();
    assert_eq!(flat.relation(slot).unwrap(), DomFlatRelation::Parent(host));
    let Found(dialog) = flat
        .nearest_clicked_dialog_with(slot, || Ok::<_, DomError>(()))
        .unwrap()
    else {
        panic!("the captured flat parent route leads to the open Dialog");
    };
    assert_eq!(dialog.id(), open_dialog);
    assert_eq!(
        fact(&c, slot, HtmlSemanticQuery::Inertness),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
}

#[test]
fn semantic_composition_is_cancelled_or_bounded_without_partial_capture() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    b.create_element_ns(&d, Namespace::html(), "slot");
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    let c = s
        .html_capture(
            c.document(&d).unwrap(),
            None,
            HtmlCaptureLimits::new(64, 4096, 128, 16384, 64, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(64, 1000000),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        c.with_slot_semantics(limits(), &AtomicBool::new(true)),
        Err(DomHtmlCaptureError::Cancelled)
    ));
}
#[test]
fn nearest_clicked_dialog_uses_flat_ancestors_and_preserves_held_results() {
    use surgeist_dom::DomError;
    use surgeist_dom::dialog::DomNearestClickedDialog::{Absent, Found};

    let mut dom = owner();
    let mut b = DomBatch::new();
    let document = b.create_document();
    let other_document = b.create_document();
    let outer = b.create_element_ns(&document, Namespace::html(), "dialog");
    let host = b.create_element_ns(&document, Namespace::html(), "div");
    b.set_attribute(&outer, "open", "");
    b.insert_before(&outer, &document, None);
    b.insert_before(&host, &outer, None);
    let root = b.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Closed,
            HtmlSlotAssignment::Manual,
            false,
            false,
            false,
        ),
    );
    let inner = b.create_element_ns(&document, Namespace::html(), "dialog");
    b.set_attribute(&inner, "open", "");
    let closed = b.create_element_ns(&document, Namespace::html(), "dialog");
    b.set_attribute_ns(&closed, Namespace::from_uri("urn:example"), "q:open", "");
    let slot = b.create_element_ns(&document, Namespace::html(), "slot");
    b.insert_before(&inner, &root, None);
    b.insert_before(&closed, &inner, None);
    b.insert_before(&slot, &closed, None);
    let assigned = b.create_element_ns(&document, Namespace::html(), "span");
    let excluded = b.create_element_ns(&document, Namespace::html(), "span");
    b.insert_before(&assigned, &host, None);
    b.insert_before(&excluded, &host, None);
    b.assign_slot(&slot, vec![(&assigned).into()]);
    let wrong_document = b.create_text(&other_document, "another Document");
    let commit = apply(&mut dom, b);
    let document = commit.document(&document).unwrap();
    let [outer, inner, assigned, excluded, wrong_document] =
        [outer, inner, assigned, excluded, wrong_document]
            .map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let held = dom
        .snapshot()
        .flat_tree(
            document,
            DomSlotQueryLimits::new(64, 64).unwrap(),
            &cancel(),
        )
        .unwrap();
    let nearest = |tree: &surgeist_dom::shadow::DomFlatTree, target, expected| {
        let Found(dialog) = tree
            .nearest_clicked_dialog_with(target, || Ok::<_, DomError>(()))
            .unwrap()
        else {
            panic!("expected an open Dialog in the actual flat ancestry");
        };
        assert_eq!(dialog.id(), expected);
    };
    nearest(&held, assigned, inner);
    nearest(&held, inner, inner);
    assert!(matches!(
        held.nearest_clicked_dialog_with(excluded, || Ok::<_, DomError>(())),
        Ok(Absent)
    ));
    assert!(matches!(
        held.nearest_clicked_dialog_with(wrong_document, || Ok::<_, DomError>(())),
        Err(DomError::WrongDocument)
    ));
    assert!(matches!(
        held.nearest_clicked_dialog_with(assigned, || Err(DomError::Cancelled)),
        Err(DomError::Cancelled)
    ));
    let mut remaining = 1usize;
    held.source()
        .node_with(assigned, || {
            remaining += 1;
            Ok::<_, DomError>(())
        })
        .unwrap();
    // Admit the original node lookup and its kind read, then refuse the
    // first flat relation read.
    assert!(matches!(
        held.nearest_clicked_dialog_with(assigned, || {
            remaining = remaining.checked_sub(1).ok_or(DomError::Cancelled)?;
            Ok(())
        }),
        Err(DomError::Cancelled)
    ));
    assert_eq!(remaining, 0);

    let mut b = DomBatch::new();
    b.remove_attribute(inner, "open");
    b.detach(excluded);
    b.destroy(excluded);
    apply(&mut dom, b);
    let current = dom
        .snapshot()
        .flat_tree(
            document,
            DomSlotQueryLimits::new(64, 64).unwrap(),
            &cancel(),
        )
        .unwrap();
    nearest(&current, assigned, outer);
    nearest(&held, assigned, inner);
    assert!(matches!(
        current.nearest_clicked_dialog_with(excluded, || Ok::<_, DomError>(())),
        Err(DomError::StaleNode)
    ));

    let mut unrelated = owner();
    let mut b = DomBatch::new();
    let document = b.create_document();
    let foreign = apply(&mut unrelated, b).document(&document).unwrap().node();
    assert!(matches!(
        held.nearest_clicked_dialog_with(foreign, || Ok::<_, DomError>(())),
        Err(DomError::ForeignOwner)
    ));
}

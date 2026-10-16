//! Literal HTML877d Img update/form and Picture/Source mutation outcomes.
//! Wholly inactive waiting is observed separately from request/resource state.

use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlImageUpdateFlags, HtmlValueInterpreter};
use surgeist_html_semantics::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn owner() -> Dom {
    Dom::new(DomLimits::new(256, 65536, 1024, 1024, 512, 16, 1_000_000).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
fn update(snapshot: &DomSnapshot, image: NodeId) -> DomImageUpdate {
    snapshot
        .node(image)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap()
}
fn single() -> (Dom, DocumentId, NodeId) {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let image = batch.create_element_ns(&document, Namespace::html(), "img");
    let commit = apply(&mut dom, batch);
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&image).unwrap().unwrap(),
    )
}
fn capture(snapshot: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(512, 8192, 512, 65536, 512, 0, 0),
            DomHtmlCaptureLimits::new(512, 1_000_000),
            &cancel(),
        )
        .unwrap()
}
fn fact(capture: &DomHtmlCapture, node: NodeId, query: HtmlSemanticQuery) -> HtmlSemanticFact {
    capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(node).unwrap(),
                query,
            )],
            HtmlValueInterpreter::new(65536),
            HtmlSemanticLimits::new(1_000_000, 16, 1024, 512, 4_000_000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .results()[0]
        .fact()
        .clone()
}

#[test]
fn equal_src_restart_is_replaced_by_the_latest_update_flags_and_failed_edits_roll_back() {
    let (mut dom, document, image) = single();
    let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    let mut batch = DomBatch::new();
    batch.set_attribute(image, "src", raw.clone());
    apply(&mut dom, batch);
    let first = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute(image, "src", raw.clone());
    apply(&mut dom, batch);
    let restarted = dom.snapshot();
    assert_ne!(update(&first, image).id(), update(&restarted, image).id());
    assert_eq!(
        update(&restarted, image).flags(),
        HtmlImageUpdateFlags::new(true, false)
    );

    let mut batch = DomBatch::new();
    batch.set_attribute(image, "srcset", "candidate.png 2x");
    apply(&mut dom, batch);
    let latest = dom.snapshot();
    assert_ne!(update(&restarted, image).id(), update(&latest, image).id());
    assert_eq!(
        update(&latest, image).flags(),
        HtmlImageUpdateFlags::new(false, false)
    );
    assert_eq!(update(&latest, image).document(), document);
    assert_eq!(update(&latest, image).id().target(), image);
    assert_eq!(
        update(&restarted, image).flags(),
        HtmlImageUpdateFlags::new(true, false)
    );

    let mut batch = DomBatch::new();
    batch.set_attribute(image, "src", raw.clone());
    batch.set_attribute(image, "invalid name", "late");
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::InvalidCharacter
    );
    assert_eq!(dom.snapshot().stamp(), latest.stamp());
    assert_eq!(
        update(&dom.snapshot(), image).id(),
        update(&latest, image).id()
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(image, "src", raw);
    apply(&mut dom, batch);
    assert_eq!(
        update(&dom.snapshot(), image).flags(),
        HtmlImageUpdateFlags::new(true, false)
    );
}

#[test]
fn cors_and_referrer_updates_follow_changed_states_rather_than_changed_spellings() {
    let (mut dom, _, image) = single();
    for (attribute, value, changed) in [
        ("crossorigin", "", true),
        ("crossorigin", "anonymous", false),
        ("crossorigin", "invalid-anonymous-value", false),
        ("crossorigin", "use-credentials", true),
        ("crossorigin", "USE-CREDENTIALS", false),
        ("referrerpolicy", "invalid-policy", false),
        ("referrerpolicy", "no-referrer", true),
        ("referrerpolicy", "NO-REFERRER", false),
        ("referrerpolicy", "origin", true),
    ] {
        let before = update(&dom.snapshot(), image).id();
        let mut batch = DomBatch::new();
        batch.set_attribute(image, attribute, value);
        apply(&mut dom, batch);
        let after = update(&dom.snapshot(), image);
        assert_eq!(after.id() != before, changed, "{attribute}={value:?}");
        assert_eq!(after.flags(), HtmlImageUpdateFlags::new(false, false));
    }
    let before = update(&dom.snapshot(), image).id();
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(image, Namespace::from_uri("urn:test"), "p:src", "ignored");
    batch.set_attribute_ns(image, Namespace::from_uri("urn:test"), "p:crossorigin", "");
    batch.set_attribute(image, "alt", "alternative text");
    apply(&mut dom, batch);
    assert_eq!(update(&dom.snapshot(), image).id(), before);
}

#[test]
fn source_attributes_update_only_following_images_but_source_insertion_and_removal_visit_all_direct_images()
 {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let picture = batch.create_element_ns(&document, Namespace::html(), "picture");
    let a = batch.create_element_ns(&document, Namespace::html(), "img");
    let source = batch.create_element_ns(&document, Namespace::html(), "source");
    let b = batch.create_element_ns(&document, Namespace::html(), "img");
    let wrapper = batch.create_element_ns(&document, Namespace::html(), "div");
    let nested = batch.create_element_ns(&document, Namespace::html(), "img");
    let d = batch.create_element_ns(&document, Namespace::html(), "img");
    batch.insert_before(&nested, &wrapper, None);
    for node in [&a, &source, &b, &wrapper, &d] {
        batch.insert_before(node, &picture, None);
    }
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let picture = commit.created(&picture).unwrap().unwrap();
    let source = commit.created(&source).unwrap().unwrap();
    let images = [a, b, nested, d].map(|ticket| commit.created(&ticket).unwrap().unwrap());
    for (name, value) in [
        ("srcset", "a.png"),
        ("sizes", "10px"),
        ("media", "print"),
        ("type", "image/png"),
        ("width", "10"),
        ("height", "20"),
    ] {
        let before = images.map(|id| update(&dom.snapshot(), id).id());
        let mut batch = DomBatch::new();
        batch.set_attribute(source, name, value);
        apply(&mut dom, batch);
        let after = images.map(|id| update(&dom.snapshot(), id).id());
        assert_eq!(
            std::array::from_fn::<_, 4, _>(|i| after[i] != before[i]),
            [false, true, false, true],
            "Source {name}"
        );
    }
    let before = images.map(|id| update(&dom.snapshot(), id).id());
    let mut batch = DomBatch::new();
    batch.set_attribute(source, "src", "not-picture-srcset.png");
    batch.set_attribute_ns(source, Namespace::from_uri("urn:test"), "p:media", "screen");
    apply(&mut dom, batch);
    assert_eq!(images.map(|id| update(&dom.snapshot(), id).id()), before);

    let mut batch = DomBatch::new();
    let tail_source = batch.create_element_ns(document, Namespace::html(), "source");
    batch.insert_before(&tail_source, picture, None);
    let commit = apply(&mut dom, batch);
    let tail_source = commit.created(&tail_source).unwrap().unwrap();
    let after = images.map(|id| update(&dom.snapshot(), id).id());
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| after[i] != before[i]),
        [true, true, false, true]
    );
    let mut batch = DomBatch::new();
    batch.detach(tail_source);
    apply(&mut dom, batch);
    let removed = images.map(|id| update(&dom.snapshot(), id).id());
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| removed[i] != after[i]),
        [true, true, false, true]
    );
}

#[test]
fn image_form_owner_is_nonlisted_and_does_not_grant_custom_validity() {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let a = batch.create_element_ns(&document, Namespace::html(), "form");
    let b = batch.create_element_ns(&document, Namespace::html(), "form");
    let image = batch.create_element_ns(&document, Namespace::html(), "img");
    batch.set_attribute(&b, "id", "B");
    batch.set_attribute(&image, "form", "B");
    batch.insert_before(&image, &a, None);
    batch.insert_before(&a, &root, None);
    batch.insert_before(&b, &root, None);
    batch.insert_before(&root, &document, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let a = commit.created(&a).unwrap().unwrap();
    let b = commit.created(&b).unwrap().unwrap();
    let image = commit.created(&image).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let view = snapshot.node(image).unwrap().as_img().unwrap();
    assert_eq!(view.form_owner().unwrap(), Some(a));
    assert!(!view.parser_inserted());
    let captured = capture(&snapshot, document);
    let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) =
        fact(&captured, image, HtmlSemanticQuery::FormOwner)
    else {
        panic!("actual Img form owner must be known");
    };
    assert_eq!(owner.owner().map(|id| captured.node(id).unwrap()), Some(a));
    let HtmlSemanticFact::FormControls(HtmlSemanticValue::Known(controls)) =
        fact(&captured, a, HtmlSemanticQuery::FormControls)
    else {
        panic!("actual listed collection must be known");
    };
    assert!(controls.controls().is_empty());

    let mut batch = DomBatch::new();
    batch.set_custom_validity(image, "not a validity control");
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::CustomValidityInapplicable
    );
    assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
    let mut batch = DomBatch::new();
    batch.set_attribute(b, "id", "changed");
    batch.set_attribute(image, "form", "changed");
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot()
            .node(image)
            .unwrap()
            .as_img()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(a)
    );
    let mut batch = DomBatch::new();
    batch.insert_before(image, b, None);
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot()
            .node(image)
            .unwrap()
            .as_img()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(b)
    );
    assert_eq!(
        snapshot
            .node(image)
            .unwrap()
            .as_img()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(a)
    );
}

#[test]
fn clone_has_its_own_fresh_wait_and_adoption_replaces_flags_without_cancelling_the_image() {
    let (mut dom, document, image) = single();
    let mut batch = DomBatch::new();
    batch.set_attribute(image, "src", "same.png");
    batch.set_attribute(image, "src", "same.png");
    let second = batch.create_document();
    let commit = apply(&mut dom, batch);
    let second = commit.document(&second).unwrap();
    let before = dom.snapshot();
    let old_update = update(&before, image);
    assert_eq!(old_update.flags(), HtmlImageUpdateFlags::new(true, false));
    let mut batch = DomBatch::new();
    let copy = batch.import_node(&before, image, document, false);
    batch.adopt(image, second);
    let commit = apply(&mut dom, batch);
    let copy = commit.created(&copy).unwrap().unwrap();
    let current = dom.snapshot();
    let new_update = update(&current, image);
    let clone_update = update(&current, copy);
    assert_ne!(new_update.id(), old_update.id());
    assert_eq!(new_update.document(), second);
    assert_eq!(new_update.id().target(), image);
    assert_eq!(clone_update.id().target(), copy);
    assert_eq!(clone_update.document(), document);
    assert_eq!(
        clone_update.flags(),
        HtmlImageUpdateFlags::new(false, false)
    );
    assert_eq!(new_update.flags(), HtmlImageUpdateFlags::new(false, false));
    assert_eq!(
        current
            .node(copy)
            .unwrap()
            .as_img()
            .unwrap()
            .dimension_attribute_source(),
        copy
    );
    assert_eq!(
        current.node(image).unwrap().as_img().unwrap().state(),
        before.node(image).unwrap().as_img().unwrap().state()
    );
    assert_eq!(update(&before, image).document(), document);
    assert_eq!(
        update(&before, image).flags(),
        HtmlImageUpdateFlags::new(true, false)
    );
    assert!(
        !commit
            .changes()
            .changes()
            .iter()
            .any(|change| matches!(change, DomChange::ImageUpdateCancelled { .. }))
    );

    let mut batch = DomBatch::new();
    batch.destroy(image);
    let commit = apply(&mut dom, batch);
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::ImageUpdateCancelled {
                update: new_update.id(),
                flags: HtmlImageUpdateFlags::new(false, false),
                reason: DomImageUpdateCancellationReason::TargetDestroyed,
            })
    );
    assert_eq!(dom.snapshot().node(image).unwrap_err(), DomError::StaleNode);
    assert_eq!(update(&dom.snapshot(), copy).id(), clone_update.id());
    assert_eq!(update(&current, image).id(), new_update.id());
    let mut batch = DomBatch::new();
    let replacement = batch.create_element_ns(second, Namespace::html(), "img");
    let commit = apply(&mut dom, batch);
    let replacement = commit.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, image);
    assert_eq!(
        update(&dom.snapshot(), replacement).id().target(),
        replacement
    );
    assert_ne!(update(&dom.snapshot(), replacement).id(), new_update.id());
}

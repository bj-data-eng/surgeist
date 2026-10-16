//! HTML877d Img's inactive prefix and exact-generation native retirement.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{
    HtmlConstructionLimits, HtmlFragment, HtmlImageUpdateFlags, HtmlImgFreshRequestState,
};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(nodes: usize, units: usize) -> DomLimits {
    DomLimits::new(nodes, units, 4096, 0, 0, 16, 262144).unwrap()
}
fn fixture() -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(limits(64, 16384)).unwrap();
    let mut b = DomBatch::new();
    let document = b.create_document();
    let img = b.create_element_ns(&document, Namespace::html(), "img");
    let c = dom.apply(b, &cancel()).unwrap();
    (
        dom,
        c.document(&document).unwrap(),
        c.created(&img).unwrap().unwrap(),
    )
}
fn wait(dom: &Dom, img: NodeId) -> DomImageUpdate {
    dom.snapshot()
        .node(img)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap()
}
#[test]
fn fresh_request_is_unavailable_and_real_inactive_wait_precedes_disabled_images() {
    let (dom, doc, img) = fixture();
    let s = dom.snapshot();
    let image = s.node(img).unwrap().as_img().unwrap();
    assert_eq!(image.dimension_attribute_source(), img);
    assert_eq!(image.form_owner().unwrap(), None);
    assert!(!image.parser_inserted());
    let state = image.state();
    assert_eq!(
        state.current_request().state(),
        HtmlImgFreshRequestState::Unavailable
    );
    assert!(state.current_request().current_url().is_empty());
    assert_eq!(state.current_request().current_pixel_density(), 1);
    assert!(state.current_request().preferred_dimensions_are_null());
    assert!(!state.current_request().image_data_is_available());
    assert!(state.pending_request_is_null());
    assert!(state.last_selected_source_is_null());
    assert!(state.last_auto_sizes_width_is_null());
    assert!(state.lazy_load_resumption_is_null());
    let update = image.suspended_update().unwrap().unwrap();
    assert_eq!(update.id().target(), img);
    assert_eq!(update.document(), doc);
    assert_eq!(update.flags(), HtmlImageUpdateFlags::new(false, false));
}
#[test]
fn superseded_wholly_inactive_waits_use_one_fixed_node_slot_and_latest_exact_flags() {
    let (mut dom, _, img) = fixture();
    let mut b = DomBatch::new();
    b.set_attribute(img, "src", "x");
    dom.apply(b, &cancel()).unwrap();
    let before = dom.snapshot();
    let initial = wait(&dom, img);
    // Exact canonical live-node and string bounds are sufficient for any
    // admitted number of wholly inactive superseding invocations.
    let units = dom.usage().string_code_units();
    dom.set_limits(limits(2, units)).unwrap();
    let mut b = DomBatch::new();
    for _ in 0..512 {
        b.set_attribute(img, "src", "x");
    }
    let c = dom.apply(b, &cancel()).unwrap();
    let latest = wait(&dom, img);
    assert_ne!(latest.id(), initial.id());
    assert_eq!(latest.flags(), HtmlImageUpdateFlags::new(true, false));
    assert_eq!(dom.usage().string_code_units(), units);
    assert_eq!(dom.usage().managed_nodes(), 2);
    assert_eq!(c.changes().changes(), [DomChange::ElementState(img)]);
    assert_eq!(
        before
            .node(img)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update()
            .unwrap()
            .unwrap()
            .id(),
        initial.id()
    );
    dom.set_limits(limits(2, units + 64)).unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(img, "sizes", "100vw");
    dom.apply(b, &cancel()).unwrap();
    assert_eq!(
        wait(&dom, img).flags(),
        HtmlImageUpdateFlags::new(false, false)
    );
}
#[test]
fn retirement_captures_the_actual_same_batch_latest_wait_and_never_follows_reused_generation() {
    let (mut dom, doc, img) = fixture();
    let mut b = DomBatch::new();
    b.set_attribute(img, "src", "x");
    dom.apply(b, &cancel()).unwrap();
    let before = dom.snapshot();
    let prior = wait(&dom, img);
    let mut b = DomBatch::new();
    b.set_attribute(img, "src", "x"); // true restart flag, newer than prior.
    b.destroy(img);
    let c = dom.apply(b, &cancel()).unwrap();
    let records: Vec<_> = c
        .changes()
        .changes()
        .iter()
        .filter_map(|change| {
            if let DomChange::ImageUpdateCancelled {
                update,
                flags,
                reason,
            } = change
            {
                Some((*update, *flags, *reason))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].0.target(), img);
    assert_ne!(records[0].0, prior.id());
    assert_eq!(records[0].1, HtmlImageUpdateFlags::new(true, false));
    assert_eq!(
        records[0].2,
        DomImageUpdateCancellationReason::TargetDestroyed
    );
    assert!(matches!(dom.snapshot().node(img), Err(DomError::StaleNode)));
    assert_eq!(
        before
            .node(img)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update()
            .unwrap()
            .unwrap()
            .id(),
        prior.id()
    );
    let mut b = DomBatch::new();
    let replacement = b.create_element_ns(doc, Namespace::html(), "img");
    let c = dom.apply(b, &cancel()).unwrap();
    let replacement = c.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, img);
    assert_ne!(wait(&dom, replacement).id(), records[0].0);
    assert!(
        !c.changes()
            .changes()
            .iter()
            .any(|change| matches!(change, DomChange::ImageUpdateCancelled { .. }))
    );
}
#[test]
fn late_refusal_and_private_creation_retirement_preserve_identity_units_and_history() {
    let (mut dom, doc, img) = fixture();
    let before = dom.snapshot();
    let prior = wait(&dom, img);
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.set_attribute(img, "src", "x");
    b.insert_before(img, img, None);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(wait(&dom, img).id(), prior.id());
    assert_eq!(dom.usage(), usage);
    let mut b = DomBatch::new();
    let private = b.create_element_ns(doc, Namespace::html(), "img");
    b.set_attribute(&private, "src", "never-published");
    b.destroy(&private);
    let c = dom.apply(b, &cancel()).unwrap();
    assert_eq!(c.created(&private).unwrap(), None);
    assert!(c.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    dom.set_limits(limits(2, 16384)).unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(img, "src", "private-update");
    b.create_element_ns(doc, Namespace::html(), "img");
    assert!(matches!(
        dom.apply(b, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            ..
        })
    ));
    assert_eq!(wait(&dom, img).id(), prior.id());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}
#[test]
fn one_short_raw_string_admission_preserves_the_previous_wait_then_exact_admission_starts() {
    let (mut dom, _, img) = fixture();
    let before = dom.snapshot();
    let prior = wait(&dom, img);
    let units = dom.usage().string_code_units();
    dom.set_limits(limits(64, units + 5)).unwrap(); // src3 + abc3 requires6.
    let mut b = DomBatch::new();
    b.set_attribute(img, "src", "abc");
    assert!(matches!(
        dom.apply(b, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(wait(&dom, img).id(), prior.id());
    dom.set_limits(limits(64, units + 6)).unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(img, "src", "abc");
    dom.apply(b, &cancel()).unwrap();
    assert_eq!(dom.usage().string_code_units(), units + 6);
    assert_ne!(wait(&dom, img).id(), prior.id());
}
#[test]
fn native_authoring_installation_starts_fresh_updates_and_self_dimension_sources() {
    let (mut dom, doc, _) = fixture();
    let mut b = DomBatch::new();
    let form = b.create_element_ns(doc, Namespace::html(), "form");
    let c = dom.apply(b, &cancel()).unwrap();
    let form = c.created(&form).unwrap().unwrap();
    let mut builder =
        HtmlFragment::builder(HtmlConstructionLimits::new(16, 32, 8, 4096, 16)).unwrap();
    let root = builder.root();
    let picture = builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::html(), None, "picture"),
            OrderedAttributes::default(),
        )
        .unwrap();
    builder
        .append_element(
            &picture,
            QualifiedName::new(Namespace::html(), None, "source"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "srcset"),
                "first 1x",
            )])
            .unwrap(),
        )
        .unwrap();
    builder
        .append_element(
            &picture,
            QualifiedName::new(Namespace::html(), None, "img"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "src"),
                "fallback",
            )])
            .unwrap(),
        )
        .unwrap();
    let source = builder.finish();
    let picture = source.root().children().next().unwrap();
    let source_img = source.node(&picture).unwrap().children().nth(1).unwrap();
    let report = dom
        .install_native_html_fragment(
            dom.snapshot().document(doc).unwrap().stamp(),
            form,
            None,
            &source,
            DomNativeHtmlInstallLimits::new(32, 100000, 131072),
            &cancel(),
        )
        .unwrap();
    let image = report.node_for(&source_img).unwrap();
    let view = report.snapshot().node(image).unwrap().as_img().unwrap();
    assert_eq!(view.dimension_attribute_source(), image);
    assert_eq!(view.form_owner().unwrap(), Some(form));
    assert!(!view.parser_inserted());
    assert_eq!(
        view.suspended_update().unwrap().unwrap().id().target(),
        image
    );
    assert_eq!(
        view.suspended_update().unwrap().unwrap().flags(),
        HtmlImageUpdateFlags::new(false, false)
    );
    assert!(view.state().current_request().current_url().is_empty());
}

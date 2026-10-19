//! Raw source transitions and immediate consuming handoff; no task delivery.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlDetailsState::{Closed, Open};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(pending: usize) -> DomLimits {
    DomLimits::new(
        128,
        8192,
        512,
        256,
        64,
        8,
        131072,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_details_toggle_intentions(pending)
    .unwrap()
}
fn apply(dom: &mut Dom, b: DomBatch) -> DomCommit {
    dom.apply(b, &cancel()).unwrap()
}
fn pair(pending: usize) -> (Dom, DocumentId, NodeId, NodeId, NodeId) {
    let mut dom = Dom::new(limits(pending)).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    let a = b.create_element_ns(&d, Namespace::html(), "details");
    let z = b.create_element_ns(&d, Namespace::html(), "details");
    for node in [&a, &z] {
        b.set_attribute(node, "name", "g");
        b.insert_before(node, &root, None);
    }
    let c = apply(&mut dom, b);
    (
        dom,
        c.document(&d).unwrap(),
        c.created(&root).unwrap().unwrap(),
        c.created(&a).unwrap().unwrap(),
        c.created(&z).unwrap().unwrap(),
    )
}
fn facts(
    t: &DomDetailsToggleTransfer,
) -> Vec<(
    NodeId,
    surgeist_html::HtmlDetailsState,
    surgeist_html::HtmlDetailsState,
)> {
    t.intentions()
        .iter()
        .map(|i| (i.target(), i.old_state(), i.new_state()))
        .collect()
}
fn state(d: &Dom, id: NodeId) -> surgeist_html::HtmlDetailsState {
    d.snapshot().node(id).unwrap().as_details().unwrap().state()
}

#[test]
fn transitions_preserve_primary_secondary_order_duplicates_and_net_equal_states() {
    let (mut d, _, _, a, b) = pair(16);
    let before = d.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "open", "false");
    batch.set_attribute(b, "open", ""); // B opening before A's secondary close.
    batch.remove_attribute(b, "open");
    batch.set_attribute(b, "open", "again");
    batch.set_attribute(b, "open", "redundant");
    batch.remove_attribute(a, "open"); // absent removal contributes nothing.
    apply(&mut d, batch);
    assert_eq!((state(&d, a), state(&d, b)), (Closed, Open));
    assert_eq!(
        before.node(a).unwrap().as_details().unwrap().state(),
        Closed
    );
    assert_eq!(d.pending_details_toggle_intentions(), 5);
    assert_eq!(d.snapshot().pending_details_toggle_intentions(), 5);
    let queued = d.snapshot();
    let t = d.take_details_toggle_intentions(5, &cancel()).unwrap();
    assert_eq!(
        facts(&t),
        [
            (a, Closed, Open),
            (b, Closed, Open),
            (a, Open, Closed),
            (b, Open, Closed),
            (b, Closed, Open)
        ]
    );
    let ids: std::collections::HashSet<_> = t.intentions().iter().map(|i| i.operation()).collect();
    assert_eq!(ids.len(), 5);
    assert_eq!(queued.pending_details_toggle_intentions(), 5);
    assert_eq!(t.source().stamp(), d.snapshot().stamp());
    assert!(
        t.changes()
            .changes()
            .contains(&DomChange::DetailsToggleIntentions)
    );
    assert_eq!(d.pending_details_toggle_intentions(), 0);
    assert_eq!(t.into_iter().count(), 5); // consumes actual owned records.
    let after = d.snapshot();
    assert!(
        d.take_details_toggle_intentions(0, &cancel())
            .unwrap()
            .is_empty()
    );
    assert_eq!(d.snapshot().stamp(), after.stamp());
}

#[test]
fn default_zero_pending_and_one_short_output_refuse_atomically_then_retry() {
    let (mut d, _, _, a, _) = pair(0);
    let before = d.snapshot();
    let mut b = DomBatch::new();
    b.set_attribute(a, "open", "");
    assert!(matches!(
        d.apply(b, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::DetailsToggleIntentions,
            ..
        })
    ));
    assert_eq!(d.snapshot().stamp(), before.stamp());
    assert_eq!(state(&d, a), Closed);
    d.set_limits(limits(2)).unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(a, "open", "");
    b.remove_attribute(a, "open");
    let c = apply(&mut d, b);
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::DetailsToggleIntentions)
    );
    let before = d.snapshot();
    assert!(matches!(
        d.take_details_toggle_intentions(1, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::DetailsToggleOutput,
            ..
        })
    ));
    assert_eq!(d.snapshot().stamp(), before.stamp());
    assert_eq!(d.pending_details_toggle_intentions(), 2);
    assert!(matches!(
        d.take_details_toggle_intentions(2, &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    ));
    assert_eq!(d.snapshot().stamp(), before.stamp());
    assert!(matches!(
        d.set_limits(limits(1)),
        Err(DomError::ResourceLimit {
            resource: DomResource::DetailsToggleIntentions,
            ..
        })
    ));
    assert_eq!(
        facts(&d.take_details_toggle_intentions(2, &cancel()).unwrap()),
        [(a, Closed, Open), (a, Open, Closed)]
    );
}

#[test]
fn secondary_removal_one_short_pending_and_late_failure_preserve_prior_queue() {
    let (mut d, _, _, a, b) = pair(2);
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "open", "");
    apply(&mut d, batch);
    let before = d.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute(b, "open", "");
    assert!(matches!(
        d.apply(batch, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::DetailsToggleIntentions,
            ..
        })
    ));
    assert_eq!(d.snapshot().stamp(), before.stamp());
    assert_eq!((state(&d, a), state(&d, b)), (Open, Closed));
    assert_eq!(d.pending_details_toggle_intentions(), 1);
    d.set_limits(limits(4)).unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(b, "open", "");
    batch.set_attribute(b, "bad name", "");
    assert_eq!(
        d.apply(batch, &cancel()).unwrap_err(),
        DomError::InvalidCharacter
    );
    assert_eq!(d.snapshot().stamp(), before.stamp());
    let t = d.take_details_toggle_intentions(1, &cancel()).unwrap();
    assert_eq!(facts(&t), [(a, Closed, Open)]);
}

#[test]
fn detach_adopt_and_destroy_keep_exact_ownership_and_finite_cancellation_evidence() {
    let (mut d, _, root, a, b) = pair(8);
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "open", "");
    batch.set_attribute(b, "open", "");
    apply(&mut d, batch); // A open, B open, A close.
    let historic = d.snapshot();
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    batch.adopt(root, &other);
    let c = apply(&mut d, batch);
    let other = c.document(&other).unwrap();
    assert_eq!(d.snapshot().node(a).unwrap().owner_document(), Some(other));
    assert_eq!(d.pending_details_toggle_intentions(), 3);
    let mut batch = DomBatch::new();
    batch.detach(b);
    batch.destroy(root);
    let c = apply(&mut d, batch);
    let cancellations: Vec<_> = c
        .changes()
        .changes()
        .iter()
        .filter_map(|change| match change {
            DomChange::DetailsToggleIntentionCancelled {
                operation,
                target,
                old,
                new,
                reason,
            } => Some((*operation, *target, *old, *new, *reason)),
            _ => None,
        })
        .collect();
    assert_eq!(cancellations.len(), 2);
    assert_eq!(
        cancellations
            .iter()
            .map(|x| (x.1, x.2, x.3, x.4))
            .collect::<Vec<_>>(),
        [
            (
                a,
                Closed,
                Open,
                DomDetailsToggleCancellationReason::TargetDestroyed
            ),
            (
                a,
                Open,
                Closed,
                DomDetailsToggleCancellationReason::TargetDestroyed
            )
        ]
    );
    assert_ne!(cancellations[0].0, cancellations[1].0);
    assert_eq!(d.pending_details_toggle_intentions(), 1);
    assert!(historic.node(a).is_ok());
    assert!(matches!(d.snapshot().node(a), Err(DomError::StaleNode)));
    let handed = d.take_details_toggle_intentions(1, &cancel()).unwrap();
    assert_eq!(facts(&handed), [(b, Closed, Open)]);
    let handed_id = handed.intentions()[0].operation();
    let mut batch = DomBatch::new();
    batch.destroy(b);
    let c = apply(&mut d, batch);
    assert!(
        !c.changes()
            .changes()
            .iter()
            .any(|c| matches!(c, DomChange::DetailsToggleIntentionCancelled { .. }))
    );
    assert_eq!(handed.intentions()[0].operation(), handed_id);
    assert!(handed.source().node(b).is_ok());
    assert!(matches!(d.snapshot().node(b), Err(DomError::StaleNode)));
    let mut batch = DomBatch::new();
    let replacement = batch.create_element_ns(other, Namespace::html(), "details");
    let c = apply(&mut d, batch);
    let replacement = c.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, b);
    assert!(matches!(d.snapshot().node(b), Err(DomError::StaleNode)));
    let foreign = Dom::new(limits(0)).unwrap();
    assert!(matches!(
        foreign.snapshot().node(b),
        Err(DomError::ForeignOwner)
    ));
}

#[test]
fn clone_copies_attributes_into_fresh_hooks_and_preserves_source_history() {
    let (mut d, doc, root, a, _) = pair(8);
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "open", "");
    apply(&mut d, batch);
    let original = d.take_details_toggle_intentions(1, &cancel()).unwrap();
    let history = d.snapshot();
    let mut batch = DomBatch::new();
    let clone = batch.import_node(&history, a, doc, true);
    batch.insert_before(&clone, root, None);
    let c = apply(&mut d, batch);
    let clone = c.created(&clone).unwrap().unwrap();
    assert_eq!((state(&d, a), state(&d, clone)), (Open, Closed));
    let copied = d.take_details_toggle_intentions(2, &cancel()).unwrap();
    assert_eq!(
        facts(&copied),
        [(clone, Closed, Open), (clone, Open, Closed)]
    );
    assert_ne!(
        copied.intentions()[0].operation(),
        original.intentions()[0].operation()
    );
    assert_eq!(history.node(a).unwrap().as_details().unwrap().state(), Open);
}

#[test]
fn native_immutable_installation_creates_fresh_attribute_transitions_and_mapping() {
    use surgeist_html::{HtmlConstructionLimits, HtmlFragment};
    let (mut d, doc, root, a, _) = pair(8);
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "open", "");
    apply(&mut d, batch);
    d.take_details_toggle_intentions(1, &cancel()).unwrap();
    let mut builder =
        HtmlFragment::builder(HtmlConstructionLimits::new(8, 16, 8, 1024, 8)).unwrap();
    let attrs = OrderedAttributes::new([
        Attribute::new(QualifiedName::new(Namespace::none(), None, "open"), "false"),
        Attribute::new(QualifiedName::new(Namespace::none(), None, "name"), "g"),
    ])
    .unwrap();
    builder
        .append_element(
            &builder.root(),
            QualifiedName::new(Namespace::html(), None, "details"),
            attrs,
        )
        .unwrap();
    let source = builder.finish();
    let report = d
        .install_native_html_fragment(
            d.snapshot().document(doc).unwrap().stamp(),
            root,
            None,
            &source,
            DomNativeHtmlInstallLimits::new(16, 10000, 65536),
            &cancel(),
        )
        .unwrap();
    let installed = *d.snapshot().node(root).unwrap().children().last().unwrap();
    assert!(report.snapshot().node(installed).is_ok());
    assert_eq!((state(&d, a), state(&d, installed)), (Open, Closed));
    assert_eq!(
        facts(&d.take_details_toggle_intentions(2, &cancel()).unwrap()),
        [(installed, Closed, Open), (installed, Open, Closed)]
    );
}

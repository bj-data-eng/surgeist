//! Native fullscreen model transitions and canonical snapshot observations.
//!
//! Independent oracles: retained Fullscreen snapshot 7c38d773117aa1e6bfa13754afe77483f40c908f,
//! model lines 45–186, keyboard-lock lines 245–255, getters lines 703–749,
//! simple-document lines 751–767, and inactive ExitFullscreen lines 831–843.
//! Cross-Document refusal follows #136's selected detached-membership policy.
//!
//! Public composition gaps: true keyboard/iframe flags, Showing popovers,
//! unavailable imported Document feature state, and pending top-layer membership
//! cannot be installed by the selected native model surface. Do not manufacture
//! them with public setters or private test fixtures here. A real ShowModal/close
//! composition must eventually cover an unflagged raw member and pending member;
//! precise unavailable-popover and active-exit refusal need their actual owning
//! state or hosted composition.
//! These model helpers do not establish permissions, platform fullscreen,
//! fullscreenEnabled, ancestor-Document collection, event delivery or unload.

use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomChange, DomCommit, DomError, DomLimits, DomResource,
    DomShadowRootInit, DomSnapshot, DomSourceCompletion, DomSourceEdit, DomSourceLimits,
    DomSourceProgress, DomTopLayerUaRule, HtmlShadowRootMode, Namespace, NodeId,
};

fn limits() -> DomLimits {
    DomLimits::new(
        256,
        100_000,
        1024,
        1024,
        1024,
        64,
        4_000_000,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn owner() -> Dom {
    Dom::new(limits()).unwrap()
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 100_000_000, 64, 1024, 128, 8)
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn document(dom: &mut Dom) -> DocumentId {
    let mut batch = DomBatch::new();
    let ticket = batch.create_document();
    apply(dom, batch).document(&ticket).unwrap()
}
fn element(dom: &mut Dom, document: DocumentId, namespace: Namespace, name: &str) -> NodeId {
    let mut batch = DomBatch::new();
    let ticket = batch.create_element_ns(document, namespace, name);
    apply(dom, batch).created(&ticket).unwrap().unwrap()
}
fn flag(snapshot: &DomSnapshot, node: NodeId) -> bool {
    snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .fullscreen_flag()
}
fn assert_layer(snapshot: &DomSnapshot, document: DocumentId, raw: &[NodeId], pending: &[NodeId]) {
    let document = snapshot.document(document).unwrap();
    let view = document.top_layer().unwrap();
    assert_eq!(view.stamp(), document.stamp());
    assert_eq!(view.raw_members(), raw);
    assert_eq!(view.pending_removals(), pending);
    // Each caller supplies the independent expected source order. This checks
    // the producer projection too, without using actual membership as its oracle.
    let expected = std::iter::once(DomTopLayerUaRule::UniversalOverlayNone)
        .chain(
            raw.iter()
                .copied()
                .filter(|id| !pending.contains(id))
                .map(DomTopLayerUaRule::ElementOverlayAuto),
        )
        .collect::<Vec<_>>();
    assert_eq!(view.ua_rules().collect::<Vec<_>>(), expected);
}
fn assert_document_model(
    snapshot: &DomSnapshot,
    document: DocumentId,
    raw_last: Option<NodeId>,
    simple: bool,
) {
    let view = snapshot.document(document).unwrap();
    assert_eq!(view.fullscreen_model_element().unwrap(), raw_last);
    assert_eq!(view.fullscreen().unwrap(), raw_last.is_some());
    assert_eq!(view.is_simple_fullscreen_document().unwrap(), simple);
    assert!(!view.keyboard_lock_active().unwrap());
}
fn fullscreen(dom: &mut Dom, node: NodeId) -> DomCommit {
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(node);
    apply(dom, batch)
}
fn complete(dom: &mut Dom, edit: DomSourceEdit) -> DomSourceCompletion {
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(edit, source_limits(), &cancel)
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("the actual empty native popover prelude has no external wait");
    };
    assert_eq!(done.output(), None);
    done
}
fn assert_unchanged(dom: &Dom, before: &DomSnapshot) {
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.nodes().collect::<Vec<_>>(),
        before.nodes().collect::<Vec<_>>()
    );
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn native_elements_in_every_namespace_start_unflagged_and_only_actual_html_iframe_has_its_flag() {
    let mut dom = owner();
    let document = document(&mut dom);
    for (namespace, name, iframe) in [
        (Namespace::none(), "iframe", None),
        (Namespace::from_uri("urn:test"), "iframe", None),
        (
            Namespace::from_uri("http://www.w3.org/2000/svg"),
            "svg",
            None,
        ),
        (Namespace::html(), "div", None),
        (Namespace::html(), "iframe", Some(false)),
        (Namespace::html(), "IFRAME", None),
    ] {
        let node = element(&mut dom, document, namespace, name);
        let snapshot = dom.snapshot();
        let value = snapshot.node(node).unwrap().as_element().unwrap();
        assert!(!value.fullscreen_flag());
        assert!(!value.keyboard_lock_flag());
        assert_eq!(value.iframe_fullscreen_flag(), iframe);
    }
    let snapshot = dom.snapshot();
    assert_layer(&snapshot, document, &[], &[]);
    assert_document_model(&snapshot, document, None, false);
    assert_eq!(
        snapshot
            .document(document)
            .unwrap()
            .fullscreen_element()
            .unwrap(),
        None
    );
}

#[test]
fn native_hidden_popover_attribute_does_not_block_detached_model_fullscreen() {
    for source in [false, true] {
        let mut dom = owner();
        let document = document(&mut dom);
        let node = element(&mut dom, document, Namespace::html(), "div");
        let mut batch = DomBatch::new();
        batch.set_attribute(node, "popover", "auto");
        apply(&mut dom, batch);
        let old = dom.snapshot();
        assert!(
            old.node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .common_state()
                .html()
                .unwrap()
                .popover_is_hidden()
        );
        let changes = if source {
            complete(&mut dom, DomSourceEdit::fullscreen_element_model(node))
                .commit()
                .changes()
                .changes()
                .to_vec()
        } else {
            fullscreen(&mut dom, node).changes().changes().to_vec()
        };
        let current = dom.snapshot();
        assert!(flag(&current, node));
        assert_eq!(current.node(node).unwrap().parent(), None);
        assert!(!current.node(node).unwrap().is_connected());
        assert!(
            current
                .node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .common_state()
                .html()
                .unwrap()
                .popover_is_hidden()
        );
        assert_layer(&current, document, &[node], &[]);
        assert_document_model(&current, document, Some(node), true);
        // The standard getter applies same-tree filtering; legacy fullscreen
        // reads the raw model and is true despite this detached candidate.
        assert_eq!(
            current
                .document(document)
                .unwrap()
                .fullscreen_element()
                .unwrap(),
            None
        );
        assert!(changes.contains(&DomChange::ElementState(node)));
        assert!(changes.contains(&DomChange::DocumentInteractiveState(document)));
        assert!(!flag(&old, node));
        assert_layer(&old, document, &[], &[]);
        assert!(dom.pending_source_edit().is_none());
        assert!(current.incomplete_source_operation().is_none());
    }
}

#[test]
fn non_html_elements_and_actual_iframe_can_enter_the_model_without_request_eligibility_checks() {
    let mut dom = owner();
    let document = document(&mut dom);
    for (namespace, name, iframe) in [
        (Namespace::none(), "generic", None),
        (Namespace::from_uri("urn:test"), "widget", None),
        (
            Namespace::from_uri("http://www.w3.org/2000/svg"),
            "svg",
            None,
        ),
        (Namespace::html(), "iframe", Some(false)),
        (Namespace::html(), "dialog", None),
    ] {
        let node = element(&mut dom, document, namespace, name);
        fullscreen(&mut dom, node);
        let snapshot = dom.snapshot();
        let value = snapshot.node(node).unwrap().as_element().unwrap();
        assert!(value.fullscreen_flag());
        assert!(!value.keyboard_lock_flag());
        assert_eq!(value.iframe_fullscreen_flag(), iframe);
        let mut batch = DomBatch::new();
        batch.unfullscreen_element_model(node);
        apply(&mut dom, batch);
        assert!(!flag(&dom.snapshot(), node));
        assert_layer(&dom.snapshot(), document, &[], &[]);
    }
}

#[test]
fn repeating_fullscreen_removes_then_appends_an_existing_active_member() {
    for source in [false, true] {
        let mut dom = owner();
        let document = document(&mut dom);
        let a = element(&mut dom, document, Namespace::html(), "div");
        let b = element(&mut dom, document, Namespace::html(), "span");
        fullscreen(&mut dom, a);
        fullscreen(&mut dom, b);
        let old = dom.snapshot();
        assert_layer(&old, document, &[a, b], &[]);
        if source {
            complete(&mut dom, DomSourceEdit::fullscreen_element_model(a));
        } else {
            fullscreen(&mut dom, a);
        }
        let current = dom.snapshot();
        assert!(flag(&current, a));
        assert!(flag(&current, b));
        assert_layer(&current, document, &[b, a], &[]);
        assert_document_model(&current, document, Some(a), false);
        assert_layer(&old, document, &[a, b], &[]);
        assert_document_model(&old, document, Some(b), false);
        let before_repeat = dom.snapshot();
        fullscreen(&mut dom, a);
        // A sole last entry is removed and appended with identical final state.
        assert_eq!(dom.snapshot().stamp(), before_repeat.stamp());
    }
}

#[test]
fn unfullscreen_element_removes_exact_member_and_document_unfullscreen_visits_all_flagged_members()
{
    for source in [false, true] {
        let mut dom = owner();
        let document = document(&mut dom);
        let other = self::document(&mut dom);
        let a = element(&mut dom, document, Namespace::html(), "div");
        let b = element(&mut dom, document, Namespace::html(), "iframe");
        let unrelated = element(&mut dom, other, Namespace::html(), "div");
        for node in [a, b, unrelated] {
            fullscreen(&mut dom, node);
        }
        let old = dom.snapshot();
        if source {
            complete(&mut dom, DomSourceEdit::unfullscreen_element_model(a));
        } else {
            let mut batch = DomBatch::new();
            batch.unfullscreen_element_model(a);
            apply(&mut dom, batch);
        }
        assert!(!flag(&dom.snapshot(), a));
        assert_layer(&dom.snapshot(), document, &[b], &[]);
        assert_document_model(&dom.snapshot(), document, Some(b), true);
        fullscreen(&mut dom, a);
        if source {
            complete(
                &mut dom,
                DomSourceEdit::unfullscreen_document_model(document),
            );
        } else {
            let mut batch = DomBatch::new();
            batch.unfullscreen_document_model(document);
            apply(&mut dom, batch);
        }
        let current = dom.snapshot();
        for node in [a, b] {
            let value = current.node(node).unwrap().as_element().unwrap();
            assert!(!value.fullscreen_flag());
            assert!(!value.keyboard_lock_flag());
        }
        assert_eq!(
            current
                .node(b)
                .unwrap()
                .as_element()
                .unwrap()
                .iframe_fullscreen_flag(),
            Some(false)
        );
        assert_layer(&current, document, &[], &[]);
        assert_document_model(&current, document, None, false);
        assert_layer(&current, other, &[unrelated], &[]);
        assert!(flag(&current, unrelated));
        assert_layer(&old, document, &[a, b], &[]);
        assert!(flag(&old, a));
        let before_noop = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.unfullscreen_element_model(a);
        batch.unfullscreen_document_model(document);
        assert!(apply(&mut dom, batch).changes().changes().is_empty());
        assert_eq!(dom.snapshot().stamp(), before_noop.stamp());
    }
}

#[test]
fn document_and_nested_shadow_getters_retarget_and_reject_a_different_tree() {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let outer = batch.create_element_ns(&document, Namespace::html(), "div");
    let outer_root =
        batch.attach_shadow(&outer, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let inner = batch.create_element_ns(&document, Namespace::html(), "span");
    let inner_root = batch.attach_shadow(&inner, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let leaf = batch.create_element_ns(&document, Namespace::html(), "p");
    let unrelated = batch.create_element_ns(&document, Namespace::html(), "div");
    let unrelated_root =
        batch.attach_shadow(&unrelated, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    batch.insert_before(&outer, &document, None);
    batch.insert_before(&inner, &outer_root, None);
    batch.insert_before(&leaf, &inner_root, None);
    batch.insert_before(&unrelated, &outer, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let outer = commit.created(&outer).unwrap().unwrap();
    let outer_root = commit.created(&outer_root).unwrap().unwrap();
    let inner = commit.created(&inner).unwrap().unwrap();
    let inner_root = commit.created(&inner_root).unwrap().unwrap();
    let leaf = commit.created(&leaf).unwrap().unwrap();
    let unrelated_root = commit.created(&unrelated_root).unwrap().unwrap();
    fullscreen(&mut dom, leaf);
    let snapshot = dom.snapshot();
    assert_document_model(&snapshot, document, Some(leaf), true);
    assert_eq!(
        snapshot
            .document(document)
            .unwrap()
            .fullscreen_element()
            .unwrap(),
        Some(outer)
    );
    assert_eq!(
        snapshot
            .shadow_root(outer_root)
            .unwrap()
            .fullscreen_element()
            .unwrap(),
        Some(inner)
    );
    assert_eq!(
        snapshot
            .shadow_root(inner_root)
            .unwrap()
            .fullscreen_element()
            .unwrap(),
        Some(leaf)
    );
    assert_eq!(
        snapshot
            .shadow_root(unrelated_root)
            .unwrap()
            .fullscreen_element()
            .unwrap(),
        None
    );
}

#[test]
fn disconnected_shadow_host_returns_null_even_while_raw_fullscreen_exists() {
    let mut dom = owner();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let root = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let leaf = batch.create_element_ns(document, Namespace::html(), "span");
    batch.insert_before(&leaf, &root, None);
    let commit = apply(&mut dom, batch);
    let host = commit.created(&host).unwrap().unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let leaf = commit.created(&leaf).unwrap().unwrap();
    fullscreen(&mut dom, leaf);
    let snapshot = dom.snapshot();
    assert!(!snapshot.node(host).unwrap().is_connected());
    assert_layer(&snapshot, document, &[leaf], &[]);
    assert_document_model(&snapshot, document, Some(leaf), true);
    assert_eq!(
        snapshot
            .document(document)
            .unwrap()
            .fullscreen_element()
            .unwrap(),
        None
    );
    assert_eq!(
        snapshot
            .shadow_root(root)
            .unwrap()
            .fullscreen_element()
            .unwrap(),
        None
    );
}

#[test]
fn inactive_removal_keeps_the_last_flag_after_exit_rejection_but_removes_lingering_membership() {
    for source in [false, true] {
        let mut dom = owner();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let host = batch.create_element_ns(&document, Namespace::html(), "div");
        let root = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
        let shadow_child = batch.create_element_ns(&document, Namespace::html(), "span");
        let light_child = batch.create_element_ns(&document, Namespace::html(), "p");
        batch.insert_before(&host, &document, None);
        batch.insert_before(&shadow_child, &root, None);
        batch.insert_before(&light_child, &host, None);
        let commit = apply(&mut dom, batch);
        let document = commit.document(&document).unwrap();
        let host = commit.created(&host).unwrap().unwrap();
        let shadow_child = commit.created(&shadow_child).unwrap().unwrap();
        let light_child = commit.created(&light_child).unwrap().unwrap();
        for node in [host, shadow_child, light_child] {
            fullscreen(&mut dom, node);
        }
        let old = dom.snapshot();
        assert_eq!(
            old.document(document)
                .unwrap()
                .metadata()
                .document_environment()
                .browsing_context(),
            surgeist_html::HtmlBrowsingContextPresence::Absent
        );
        if source {
            complete(&mut dom, DomSourceEdit::detach(host));
        } else {
            let mut batch = DomBatch::new();
            batch.detach(host);
            apply(&mut dom, batch);
        }
        let current = dom.snapshot();
        // Static shadow-including tree order is host, shadow_child, light_child.
        // The first two are not then-current fullscreen element and unfullscreen.
        // The last takes inactive ExitFullscreen, which rejects before flag work.
        assert!(!flag(&current, host));
        assert!(!flag(&current, shadow_child));
        assert!(flag(&current, light_child));
        assert_layer(&current, document, &[], &[]);
        assert_document_model(&current, document, None, false);
        assert_eq!(current.node(host).unwrap().parent(), None);
        for node in [host, shadow_child, light_child] {
            assert!(flag(&old, node));
            assert!(!current.node(node).unwrap().is_connected());
        }
        assert_layer(&old, document, &[host, shadow_child, light_child], &[]);
        // Document unfullscreen visits flagged RAW members, not every flag in
        // the Document. The lingering flag is cleared only by Element operation.
        let mut batch = DomBatch::new();
        batch.unfullscreen_document_model(document);
        apply(&mut dom, batch);
        assert!(flag(&dom.snapshot(), light_child));
        let mut batch = DomBatch::new();
        batch.unfullscreen_element_model(light_child);
        apply(&mut dom, batch);
        assert!(!flag(&dom.snapshot(), light_child));
    }
}

#[test]
fn detached_cross_document_member_adoption_and_insertion_refuse_without_a_prefix() {
    for source in [false, true] {
        for insertion in [false, true] {
            let mut dom = owner();
            let original = document(&mut dom);
            let target = document(&mut dom);
            let node = element(&mut dom, original, Namespace::html(), "div");
            fullscreen(&mut dom, node);
            let before = dom.snapshot();
            let usage = dom.usage();
            let error = if source {
                let edit = if insertion {
                    DomSourceEdit::insert_before(node, target.node(), None)
                } else {
                    DomSourceEdit::adopt(node, target.node())
                };
                let cancel = AtomicBool::new(false);
                match dom.begin_source_edit(edit, source_limits(), &cancel) {
                    Err(error) => error,
                    Ok(mut lease) => lease.advance(&cancel).unwrap_err(),
                }
            } else {
                let mut batch = DomBatch::new();
                if insertion {
                    batch.insert_before(node, target, None);
                } else {
                    batch.adopt(node, target);
                }
                dom.apply(batch, &AtomicBool::new(false)).unwrap_err()
            };
            assert!(matches!(
                error,
                DomError::TopLayerAdoptionUnavailable { .. }
            ));
            assert_unchanged(&dom, &before);
            assert_eq!(dom.usage(), usage);
            let current = dom.snapshot();
            assert_eq!(current.node(node).unwrap().owner_document(), Some(original));
            assert_eq!(current.node(node).unwrap().parent(), None);
            assert!(flag(&current, node));
            assert_layer(&current, original, &[node], &[]);
            assert_layer(&current, target, &[], &[]);
            assert!(dom.pending_source_edit().is_none());
        }
    }
}

#[test]
fn adoption_refusal_walks_detached_ordinary_and_owned_shadow_descendants() {
    for shadow in [false, true] {
        let mut dom = owner();
        let original = document(&mut dom);
        let target = document(&mut dom);
        let mut batch = DomBatch::new();
        let host = batch.create_element_ns(original, Namespace::html(), "div");
        let child = batch.create_element_ns(original, Namespace::html(), "span");
        if shadow {
            let root =
                batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
            batch.insert_before(&child, &root, None);
        } else {
            batch.insert_before(&child, &host, None);
        }
        let commit = apply(&mut dom, batch);
        let host = commit.created(&host).unwrap().unwrap();
        let child = commit.created(&child).unwrap().unwrap();
        fullscreen(&mut dom, child);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.adopt(host, target);
        assert!(matches!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::TopLayerAdoptionUnavailable { .. }
        ));
        assert_unchanged(&dom, &before);
        assert_eq!(dom.usage(), usage);
        for node in before.node(host).unwrap().shadow_including_tree_order() {
            assert_eq!(
                dom.snapshot().node(node).unwrap().owner_document(),
                Some(original)
            );
        }
        assert_layer(&dom.snapshot(), original, &[child], &[]);
        assert_layer(&dom.snapshot(), target, &[], &[]);
    }
}

#[test]
fn template_adoption_checks_actual_separately_adopted_inert_contents() {
    let mut dom = owner();
    let original = document(&mut dom);
    let target = document(&mut dom);
    let template = element(&mut dom, original, Namespace::html(), "template");
    let snapshot = dom.snapshot();
    let contents = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    assert_ne!(inert, original);
    let mut batch = DomBatch::new();
    let child = batch.create_element_ns(inert, Namespace::html(), "div");
    batch.insert_before(&child, contents, None);
    let child = apply(&mut dom, batch).created(&child).unwrap().unwrap();
    fullscreen(&mut dom, child);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.adopt(template, target);
    assert!(matches!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::TopLayerAdoptionUnavailable { .. }
    ));
    assert_unchanged(&dom, &before);
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot().node(template).unwrap().owner_document(),
        Some(original)
    );
    assert_eq!(
        dom.snapshot().node(contents).unwrap().owner_document(),
        Some(inert)
    );
    assert_eq!(
        dom.snapshot().node(child).unwrap().owner_document(),
        Some(inert)
    );
    assert_layer(&dom.snapshot(), inert, &[child], &[]);
    assert_eq!(
        dom.snapshot()
            .document(target)
            .unwrap()
            .associated_inert_template_document(),
        None
    );
}

#[test]
fn same_document_detached_adoption_preserves_membership_and_explicit_cleanup_permits_cross_document_adoption()
 {
    let mut dom = owner();
    let original = document(&mut dom);
    let target = document(&mut dom);
    let node = element(&mut dom, original, Namespace::html(), "div");
    fullscreen(&mut dom, node);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.adopt(node, original);
    apply(&mut dom, batch);
    assert_unchanged(&dom, &before);
    assert_layer(&dom.snapshot(), original, &[node], &[]);
    let mut batch = DomBatch::new();
    batch.unfullscreen_element_model(node);
    batch.adopt(node, target);
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot().node(node).unwrap().owner_document(),
        Some(target)
    );
    assert!(!flag(&dom.snapshot(), node));
    assert_layer(&dom.snapshot(), original, &[], &[]);
    assert_layer(&dom.snapshot(), target, &[], &[]);
    assert!(flag(&before, node));
    assert_layer(&before, original, &[node], &[]);
}

#[test]
fn source_required_removal_cleans_membership_and_a_residual_flag_alone_does_not_refuse_adoption() {
    for separate_removal in [false, true] {
        let mut dom = owner();
        let original = document(&mut dom);
        let target = document(&mut dom);
        let node = element(&mut dom, original, Namespace::html(), "div");
        let mut batch = DomBatch::new();
        batch.insert_before(node, original, None);
        apply(&mut dom, batch);
        fullscreen(&mut dom, node);
        let old = dom.snapshot();
        if separate_removal {
            let mut batch = DomBatch::new();
            batch.detach(node);
            apply(&mut dom, batch);
            assert!(flag(&dom.snapshot(), node));
            assert_layer(&dom.snapshot(), original, &[], &[]);
        }
        let mut batch = DomBatch::new();
        batch.adopt(node, target);
        apply(&mut dom, batch);
        let current = dom.snapshot();
        assert!(flag(&current, node));
        assert_eq!(current.node(node).unwrap().owner_document(), Some(target));
        assert_eq!(current.node(node).unwrap().parent(), None);
        assert_layer(&current, original, &[], &[]);
        assert_layer(&current, target, &[], &[]);
        assert_document_model(&current, target, None, false);
        assert_layer(&old, original, &[node], &[]);
    }
}

#[test]
fn snapshot_import_and_same_owner_clone_initialize_fresh_flags_without_copying_membership() {
    let mut dom = owner();
    let document = document(&mut dom);
    let target = self::document(&mut dom);
    let node = element(&mut dom, document, Namespace::html(), "iframe");
    fullscreen(&mut dom, node);
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    let cloned = batch.import_node(&old, node, document, true);
    let imported = batch.import_node(&old, node, target, true);
    let commit = apply(&mut dom, batch);
    let cloned = commit.created(&cloned).unwrap().unwrap();
    let imported = commit.created(&imported).unwrap().unwrap();
    let current = dom.snapshot();
    for (fresh, owner) in [(cloned, document), (imported, target)] {
        assert_ne!(fresh, node);
        let value = current.node(fresh).unwrap().as_element().unwrap();
        assert!(!value.fullscreen_flag());
        assert!(!value.keyboard_lock_flag());
        assert_eq!(value.iframe_fullscreen_flag(), Some(false));
        assert_eq!(current.node(fresh).unwrap().parent(), None);
        assert_eq!(current.node(fresh).unwrap().owner_document(), Some(owner));
    }
    assert!(flag(&current, node));
    assert!(flag(&old, node));
    assert_layer(&current, document, &[node], &[]);
    assert_layer(&current, target, &[], &[]);
    assert_layer(&old, document, &[node], &[]);
}

#[test]
fn staged_document_and_element_fullscreen_publish_with_resolved_canonical_identities() {
    let mut dom = owner();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let node = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.fullscreen_element_model(&node);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let node = commit.created(&node).unwrap().unwrap();
    let current = dom.snapshot();
    assert_eq!(current.revision().get(), before.revision().get() + 1);
    assert!(flag(&current, node));
    assert_layer(&current, document, &[node], &[]);
    assert_document_model(&current, document, Some(node), true);
    assert_eq!(current.node(node).unwrap().owner_document(), Some(document));
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::Created(node))
    );
    assert!(before.nodes().next().is_none());
}

#[test]
fn staged_membership_and_adoption_refusal_roll_back_creation_and_generation_allocation() {
    let mut dom = owner();
    let original = document(&mut dom);
    let target = document(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    let ticket = batch.create_element_ns(original, Namespace::html(), "div");
    batch.fullscreen_element_model(&ticket);
    batch.adopt(&ticket, target);
    assert!(matches!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::TopLayerAdoptionUnavailable { .. }
    ));
    assert_unchanged(&dom, &before);
    assert_eq!(dom.usage(), usage);
    assert_layer(&dom.snapshot(), original, &[], &[]);
    assert_layer(&dom.snapshot(), target, &[], &[]);
    let node = element(&mut dom, original, Namespace::html(), "div");
    assert!(!flag(&dom.snapshot(), node));
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(node);
    batch.unfullscreen_element_model(node);
    assert!(apply(&mut dom, batch).changes().changes().is_empty());
}

#[test]
fn late_invalid_command_and_cancelled_batch_do_not_publish_flags_layer_identity_or_history() {
    let mut dom = owner();
    let document = document(&mut dom);
    let node = element(&mut dom, document, Namespace::html(), "div");
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "x");
    let text = apply(&mut dom, batch).created(&text).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    for cancelled in [false, true] {
        let mut batch = DomBatch::new();
        batch.create_comment(document, "unpublished");
        batch.fullscreen_element_model(node);
        batch.replace_data(text, 99, 0, "bad");
        let error = dom.apply(batch, &AtomicBool::new(cancelled)).unwrap_err();
        assert_eq!(
            error,
            if cancelled {
                DomError::Cancelled
            } else {
                DomError::IndexSize
            }
        );
        assert_unchanged(&dom, &before);
        assert_eq!(dom.usage(), usage);
        assert!(!flag(&dom.snapshot(), node));
        assert_layer(&dom.snapshot(), document, &[], &[]);
    }
    fullscreen(&mut dom, node);
    assert!(flag(&dom.snapshot(), node));
    assert!(!flag(&before, node));
}

#[test]
fn one_short_command_allowance_refuses_the_whole_batch_and_exact_allowance_still_enters_model() {
    let mut dom = Dom::new(
        DomLimits::new(
            8,
            4096,
            1,
            64,
            64,
            8,
            65_536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let document = document(&mut dom);
    let node = element(&mut dom, document, Namespace::html(), "div");
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(node);
    batch.unfullscreen_element_model(node);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::Commands,
            limit: 1
        }
    );
    assert_unchanged(&dom, &before);
    assert_eq!(dom.usage(), usage);
    assert!(!flag(&dom.snapshot(), node));
    assert_layer(&dom.snapshot(), document, &[], &[]);
    fullscreen(&mut dom, node);
    assert!(flag(&dom.snapshot(), node));
    assert_layer(&dom.snapshot(), document, &[node], &[]);
}

#[test]
fn source_zero_storage_work_and_cancel_refuse_before_any_model_prefix_and_allow_retry() {
    let mut dom = owner();
    let document = document(&mut dom);
    let node = element(&mut dom, document, Namespace::html(), "div");
    let before = dom.snapshot();
    let usage = dom.usage();
    for (policy, cancelled, expected) in [
        (
            DomSourceLimits::new(0, 1_000_000, 100_000_000, 8, 16, 0, 1),
            false,
            DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: 0,
            },
        ),
        (
            DomSourceLimits::new(32_000_000, 1_000_000, 0, 8, 16, 0, 1),
            false,
            DomError::SourceWorkLimit { limit: 0 },
        ),
        (source_limits(), true, DomError::Cancelled),
    ] {
        let cancel = AtomicBool::new(cancelled);
        let error = match dom.begin_source_edit(
            DomSourceEdit::fullscreen_element_model(node),
            policy,
            &cancel,
        ) {
            Err(error) => error,
            Ok(mut lease) => lease.advance(&cancel).unwrap_err(),
        };
        assert_eq!(error, expected);
        assert_unchanged(&dom, &before);
        assert_eq!(dom.usage(), usage);
        assert!(!flag(&dom.snapshot(), node));
        assert_layer(&dom.snapshot(), document, &[], &[]);
        assert!(dom.pending_source_edit().is_none());
    }
    complete(&mut dom, DomSourceEdit::fullscreen_element_model(node));
    assert!(flag(&dom.snapshot(), node));
    assert_layer(&dom.snapshot(), document, &[node], &[]);
}

#[test]
fn wrong_kind_foreign_and_retired_operands_cannot_target_new_generation_or_change_captures() {
    // Capacity is just the Document and one Element: replacement must reuse
    // retired capacity rather than hide stale-handle behavior behind growth.
    let mut dom = Dom::new(
        DomLimits::new(
            2,
            4096,
            16,
            64,
            64,
            8,
            65_536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let document = document(&mut dom);
    let node = element(&mut dom, document, Namespace::html(), "div");
    fullscreen(&mut dom, node);
    let historical = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.unfullscreen_element_model(node);
    batch.destroy(node);
    apply(&mut dom, batch);
    let replacement = element(&mut dom, document, Namespace::html(), "div");
    assert_ne!(node, replacement);
    assert_eq!(dom.usage().managed_nodes(), 2);
    assert!(!flag(&dom.snapshot(), replacement));
    let mut foreign = owner();
    let foreign_document = self::document(&mut foreign);
    let foreign_node = element(&mut foreign, foreign_document, Namespace::html(), "div");
    let before = dom.snapshot();
    for (operand, error) in [
        (document.node(), DomError::WrongNodeKind),
        (foreign_node, DomError::ForeignOwner),
        (node, DomError::StaleNode),
    ] {
        let mut batch = DomBatch::new();
        batch.fullscreen_element_model(operand);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error
        );
        assert_unchanged(&dom, &before);
        assert!(!flag(&dom.snapshot(), replacement));
        assert_layer(&dom.snapshot(), document, &[], &[]);
    }
    assert!(flag(&historical, node));
    assert_layer(&historical, document, &[node], &[]);
}

#[test]
fn matching_intermediate_shadow_document_does_not_hide_deeper_adoption_membership() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            100_000,
            256,
            256,
            256,
            32,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let a = batch.create_document();
    let b = batch.create_document();
    let c = batch.create_document();
    let outer = batch.create_element_ns(&a, Namespace::html(), "div");
    let middle_root =
        batch.attach_shadow(&outer, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let b_parent = batch.create_element(&b, "replacement-parent");
    let b_old = batch.create_comment(&b, "replace with empty outer ShadowRoot");
    batch.insert_before(&b_old, &b_parent, None);
    let c_parent = batch.create_element(&c, "replacement-parent");
    let c_old = batch.create_comment(&c, "replace with empty inner ShadowRoot");
    batch.insert_before(&c_old, &c_parent, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let a = commit.document(&a).unwrap();
    let b = commit.document(&b).unwrap();
    let c = commit.document(&c).unwrap();
    let outer = commit.created(&outer).unwrap().unwrap();
    let middle_root = commit.created(&middle_root).unwrap().unwrap();
    let b_parent = commit.created(&b_parent).unwrap().unwrap();
    let b_old = commit.created(&b_old).unwrap().unwrap();
    let c_parent = commit.created(&c_parent).unwrap().unwrap();
    let c_old = commit.created(&c_old).unwrap().unwrap();

    // The ordinary public replacement algorithm adopts the empty fragment
    // operand before transferring children. Direct ShadowRoot adopt is unused.
    let mut batch = DomBatch::new();
    batch.replace_child(middle_root, b_parent, b_old);
    dom.apply(batch, &cancel).unwrap();
    let middle = dom.snapshot();
    assert_eq!(middle.node(outer).unwrap().owner_document(), Some(a));
    assert_eq!(middle.node(outer).unwrap().parent(), None);
    assert_eq!(middle.node(middle_root).unwrap().owner_document(), Some(b));
    assert_eq!(middle.node(middle_root).unwrap().parent(), None);
    assert_eq!(middle.shadow_root(middle_root).unwrap().host(), outer);
    assert_eq!(
        middle.node(outer).unwrap().hosted_shadow_root(),
        Some(middle_root)
    );
    assert!(middle.node(middle_root).unwrap().children().is_empty());
    assert!(middle.node(b_parent).unwrap().children().is_empty());

    let mut batch = DomBatch::new();
    let inner = batch.create_element_ns(b, Namespace::html(), "span");
    let deep_root = batch.attach_shadow(&inner, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    batch.insert_before(&inner, middle_root, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let inner = commit.created(&inner).unwrap().unwrap();
    let deep_root = commit.created(&deep_root).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.replace_child(deep_root, c_parent, c_old);
    dom.apply(batch, &cancel).unwrap();
    let nested = dom.snapshot();
    assert_eq!(nested.node(inner).unwrap().owner_document(), Some(b));
    assert_eq!(nested.node(inner).unwrap().parent(), Some(middle_root));
    assert_eq!(nested.node(deep_root).unwrap().owner_document(), Some(c));
    assert_eq!(nested.node(deep_root).unwrap().parent(), None);
    assert_eq!(nested.shadow_root(deep_root).unwrap().host(), inner);
    assert_eq!(
        nested.node(inner).unwrap().hosted_shadow_root(),
        Some(deep_root)
    );
    assert!(nested.node(c_parent).unwrap().children().is_empty());

    let mut batch = DomBatch::new();
    let member = batch.create_element_ns(c, Namespace::html(), "p");
    batch.insert_before(&member, deep_root, None);
    batch.fullscreen_element_model(&member);
    let member = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&member)
        .unwrap()
        .unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    // This literal mixed closure is the stimulus. Middle members already in B
    // do not imply that the deeper root and its member will stay in C.
    for (node, document) in [
        (outer, a),
        (middle_root, b),
        (inner, b),
        (deep_root, c),
        (member, c),
    ] {
        assert_eq!(before.node(node).unwrap().owner_document(), Some(document));
        assert!(!before.node(node).unwrap().is_connected());
    }
    assert_eq!(before.node(member).unwrap().parent(), Some(deep_root));
    assert_eq!(before.node(member).unwrap().root(), deep_root);
    assert_eq!(before.node(member).unwrap().shadow_including_root(), outer);
    assert_eq!(
        before
            .node(outer)
            .unwrap()
            .shadow_including_tree_order()
            .collect::<Vec<_>>(),
        [outer, middle_root, inner, deep_root, member]
    );
    assert!(
        before
            .node(member)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(
        before
            .document(c)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[member]
    );
    assert_eq!(
        before
            .document(c)
            .unwrap()
            .top_layer()
            .unwrap()
            .pending_removals(),
        &[]
    );
    for document in [a, b] {
        assert!(
            before
                .document(document)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members()
                .is_empty()
        );
    }

    let mut batch = DomBatch::new();
    batch.adopt(outer, b);
    assert_eq!(
        dom.apply(batch, &cancel).unwrap_err(),
        DomError::TopLayerAdoptionUnavailable {
            subject: Some(member),
            old_document: Some(c),
        }
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        after.nodes().collect::<Vec<_>>(),
        before.nodes().collect::<Vec<_>>()
    );
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    for (node, document) in [
        (outer, a),
        (middle_root, b),
        (inner, b),
        (deep_root, c),
        (member, c),
    ] {
        assert_eq!(after.node(node).unwrap().owner_document(), Some(document));
        assert_eq!(
            after.node(node).unwrap().parent(),
            before.node(node).unwrap().parent()
        );
        assert_eq!(
            after.node(node).unwrap().children(),
            before.node(node).unwrap().children()
        );
    }
    for document in [a, b] {
        let layer = after.document(document).unwrap().top_layer().unwrap();
        assert!(layer.raw_members().is_empty());
        assert!(layer.pending_removals().is_empty());
        assert_eq!(
            layer.ua_rules().collect::<Vec<_>>(),
            [DomTopLayerUaRule::UniversalOverlayNone]
        );
    }
    let layer = after.document(c).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), &[member]);
    assert!(layer.pending_removals().is_empty());
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(member),
        ]
    );
    assert!(
        after
            .node(member)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert!(dom.pending_source_edit().is_none());
}

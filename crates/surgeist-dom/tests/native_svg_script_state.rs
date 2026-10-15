//! Selected Tiny processing, DOM3071 topology and actual native owner closure.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn limits() -> DomLimits {
    DomLimits::new(256, 65536, 1024, 256, 256, 32, 1048576).unwrap()
}
fn dom() -> Dom {
    Dom::new(limits()).unwrap()
}
fn apply(dom: &mut Dom, b: DomBatch) -> DomCommit {
    dom.apply(b, &AtomicBool::new(false)).unwrap()
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId) {
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&root, &d, None);
    let c = apply(dom, b);
    (c.document(&d).unwrap(), c.created(&root).unwrap().unwrap())
}
fn script(s: &DomSnapshot, n: NodeId) -> DomSvgScript<'_> {
    s.node(n).unwrap().as_svg_script().unwrap()
}
fn unchanged(dom: &Dom, before: &DomSnapshot, usage: DomUsage) {
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn fresh_exact_svg_name_is_unprocessed_and_distinct_from_other_script_families() {
    let mut dom = dom();
    let (d, _) = setup(&mut dom);
    let mut b = DomBatch::new();
    let svg = b.create_element_ns(d, Namespace::svg(), "script");
    let upper = b.create_element_ns(d, Namespace::svg(), "SCRIPT");
    let html = b.create_element_ns(d, Namespace::html(), "script");
    let foreign = b.create_element_ns(d, Namespace::from_uri("urn:x"), "script");
    let plain = b.create_element(d, "script");
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    let svg = c.created(&svg).unwrap().unwrap();
    let view = script(&s, svg);
    assert_eq!(view.id(), svg);
    assert!(!view.flags().already_processed());
    assert_eq!(view.origin(), DomSvgScriptOrigin::Native);
    for ticket in [&upper, &html, &foreign, &plain] {
        assert!(
            s.node(c.created(ticket).unwrap().unwrap())
                .unwrap()
                .as_svg_script()
                .is_none()
        );
    }
    assert!(
        s.node(c.created(&html).unwrap().unwrap())
            .unwrap()
            .as_html_script()
            .is_some()
    );
    assert!(!s.node(svg).unwrap().is_connected());
}

#[test]
fn closed_native_processing_has_no_html_empty_type_href_or_handler_guard() {
    for href in [
        None,
        Some(""),
        Some("data:text/javascript,ignored"),
        Some("#same-document"),
        Some("https://example.invalid/script.js"),
    ] {
        let mut dom = dom();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let n = b.create_element_ns(&d, Namespace::svg(), "script");
        if let Some(href) = href {
            b.set_attribute_ns(
                &n,
                Namespace::from_uri("http://www.w3.org/1999/xlink"),
                "xlink:href",
                href,
            );
            b.set_attribute(&n, "href", href);
        }
        b.set_attribute(&n, "type", "arbitrary unsupported HTML type");
        b.set_attribute(&n, "onload", "inert authored handler");
        b.set_attribute(&n, "onerror", "inert authored handler");
        b.insert_before(&n, &d, None);
        let c = apply(&mut dom, b);
        let n = c.created(&n).unwrap().unwrap();
        let s = dom.snapshot();
        assert!(script(&s, n).flags().already_processed());
        assert!(s.node(n).unwrap().children().is_empty());
        assert_eq!(script(&s, n).origin(), DomSvgScriptOrigin::Native);
        // This is state publication, not a fabricated SVG event/task/resource.
        assert!(!c.changes().changes().contains(&DomChange::IntrinsicTasks));
        assert!(matches!(
            dom.native_checkpoint_step(
                surgeist_dom::DomNativeCheckpointLimits::new(
                    0,
                    0,
                    DomSlotQueryLimits::new(256, 1024).unwrap()
                )
                .unwrap(),
                &AtomicBool::new(false)
            )
            .unwrap(),
            surgeist_dom::DomNativeCheckpoint::Idle
        ));
    }
}

#[test]
fn direct_connected_open_and_closed_shadow_insertion_processes_empty_scripts() {
    for mode in [HtmlShadowRootMode::Open, HtmlShadowRootMode::Closed] {
        let mut dom = dom();
        let (d, host) = setup(&mut dom);
        let mut b = DomBatch::new();
        let root = b.attach_shadow(host, DomShadowRootInit::new(mode));
        let n = b.create_element_ns(d, Namespace::svg(), "script");
        b.insert_before(&n, &root, None);
        let c = apply(&mut dom, b);
        let n = c.created(&n).unwrap().unwrap();
        let s = dom.snapshot();
        assert!(s.node(n).unwrap().is_connected());
        assert!(script(&s, n).flags().already_processed());
    }
}

#[test]
fn detached_host_populated_before_connection_processes_both_shadow_and_light_descendants() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut b = DomBatch::new();
    let host = b.create_element_ns(d, Namespace::html(), "span");
    let root = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let shadow = b.create_element_ns(d, Namespace::svg(), "script");
    let light = b.create_element_ns(d, Namespace::svg(), "script");
    b.insert_before(&shadow, &root, None);
    b.insert_before(&light, &host, None);
    let c = apply(&mut dom, b);
    let host = c.created(&host).unwrap().unwrap();
    let shadow = c.created(&shadow).unwrap().unwrap();
    let light = c.created(&light).unwrap().unwrap();
    let old = dom.snapshot();
    for n in [shadow, light] {
        assert!(!script(&old, n).flags().already_processed());
    }
    let mut b = DomBatch::new();
    b.insert_before(host, parent, None);
    apply(&mut dom, b);
    let s = dom.snapshot();
    for n in [shadow, light] {
        assert!(script(&s, n).flags().already_processed());
        assert!(!script(&old, n).flags().already_processed());
    }
}

#[test]
fn detached_fragment_template_contents_and_manual_slot_intent_do_not_connect() {
    let mut dom = dom();
    let (d, host) = setup(&mut dom);
    let mut b = DomBatch::new();
    let root = b.attach_shadow(host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let slot = b.create_element_ns(d, Namespace::html(), "slot");
    b.insert_before(&slot, &root, None);
    let detached = b.create_element_ns(d, Namespace::svg(), "script");
    b.assign_slot(&slot, vec![(&detached).into()]);
    let fragment = b.create_fragment(d);
    let in_fragment = b.create_element_ns(d, Namespace::svg(), "script");
    b.insert_before(&in_fragment, &fragment, None);
    let template = b.create_element_ns(d, Namespace::html(), "template");
    b.insert_before(&template, host, None);
    let c = apply(&mut dom, b);
    let template = c.created(&template).unwrap().unwrap();
    let detached = c.created(&detached).unwrap().unwrap();
    let in_fragment = c.created(&in_fragment).unwrap().unwrap();
    let s = dom.snapshot();
    let contents = s.node(template).unwrap().as_template().unwrap().contents();
    let inert = s.node(contents).unwrap().owner_document().unwrap();
    let mut b = DomBatch::new();
    let nested = b.create_element_ns(inert, Namespace::svg(), "script");
    b.insert_before(&nested, contents, None);
    let c = apply(&mut dom, b);
    let nested = c.created(&nested).unwrap().unwrap();
    let s = dom.snapshot();
    for n in [detached, in_fragment, nested] {
        assert!(!s.node(n).unwrap().is_connected());
        assert!(!script(&s, n).flags().already_processed());
    }
    let mut b = DomBatch::new();
    b.insert_before(nested, host, None);
    apply(&mut dom, b);
    assert!(script(&dom.snapshot(), nested).flags().already_processed());
}

#[test]
fn actual_shadow_node_document_remains_distinct_from_connected_host_document() {
    let mut dom = dom();
    let (d1, host) = setup(&mut dom);
    let mut b = DomBatch::new();
    let d2 = b.create_document();
    let shadow = b.attach_shadow(host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let target = b.create_fragment(&d2);
    let old = b.create_comment(&d2, "old");
    b.insert_before(&old, &target, None);
    let c = apply(&mut dom, b);
    let d2 = c.document(&d2).unwrap();
    let shadow = c.created(&shadow).unwrap().unwrap();
    let target = c.created(&target).unwrap().unwrap();
    let old = c.created(&old).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.replace_child(shadow, target, old);
    let n = b.create_element_ns(d2, Namespace::svg(), "script");
    b.insert_before(&n, shadow, None);
    let c = apply(&mut dom, b);
    let n = c.created(&n).unwrap().unwrap();
    let s = dom.snapshot();
    assert_eq!(s.node(host).unwrap().owner_document(), Some(d1));
    assert_eq!(s.node(shadow).unwrap().owner_document(), Some(d2));
    assert_eq!(s.node(n).unwrap().owner_document(), Some(d2));
    assert!(s.node(n).unwrap().is_connected());
    assert!(script(&s, n).flags().already_processed());
}

#[test]
fn true_and_false_clones_preserve_actual_bit_before_insertion_and_cross_owner_import() {
    let mut source = dom();
    let (d, root) = setup(&mut source);
    let mut b = DomBatch::new();
    let started = b.create_element_ns(d, Namespace::svg(), "script");
    let eligible = b.create_element_ns(d, Namespace::svg(), "script");
    b.insert_before(&started, root, None);
    let text = b.create_text(d, "exact\r\n");
    b.insert_before(&text, &started, None);
    let c = apply(&mut source, b);
    let started = c.created(&started).unwrap().unwrap();
    let eligible = c.created(&eligible).unwrap().unwrap();
    let old = source.snapshot();
    let mut target = dom();
    let (td, tr) = setup(&mut target);
    let mut b = DomBatch::new();
    let true_copy = b.import_node(&old, started, td, true);
    let false_copy = b.import_node(&old, eligible, td, false);
    let c = apply(&mut target, b);
    let true_copy = c.created(&true_copy).unwrap().unwrap();
    let false_copy = c.created(&false_copy).unwrap().unwrap();
    let s = target.snapshot();
    assert!(script(&s, true_copy).flags().already_processed());
    assert!(!script(&s, false_copy).flags().already_processed());
    for n in [true_copy, false_copy] {
        assert_eq!(script(&s, n).origin(), DomSvgScriptOrigin::Clone);
        assert!(!s.node(n).unwrap().is_connected());
        assert_eq!(s.node(n).unwrap().owner_document(), Some(td));
    }
    let text = s.node(true_copy).unwrap().children()[0];
    assert!(
        matches!(s.node(text).unwrap().kind(),DomNodeKind::Text(v) if v==&DocumentString::from("exact\r\n"))
    );
    let mut b = DomBatch::new();
    b.insert_before(true_copy, tr, None);
    b.insert_before(false_copy, tr, None);
    apply(&mut target, b);
    let after = target.snapshot();
    assert!(script(&after, false_copy).flags().already_processed());
    assert!(script(&after, true_copy).flags().already_processed());
    assert!(!script(&s, false_copy).flags().already_processed());
    assert!(!script(&old, eligible).flags().already_processed());
    assert_eq!(script(&old, started).origin(), DomSvgScriptOrigin::Native);
}

#[test]
fn shallow_host_clone_copies_clonable_shadow_scripts_true_and_false_before_connection() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut b = DomBatch::new();
    let connected = b.create_element_ns(d, Namespace::html(), "span");
    let detached = b.create_element_ns(d, Namespace::html(), "span");
    let properties = surgeist_html::HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Closed,
        surgeist_html::HtmlSlotAssignment::Named,
        false,
        true,
        false,
    );
    let connected_root = b.attach_shadow(&connected, properties);
    let detached_root = b.attach_shadow(&detached, properties);
    let started = b.create_element_ns(d, Namespace::svg(), "script");
    let eligible = b.create_element_ns(d, Namespace::svg(), "script");
    b.insert_before(&started, &connected_root, None);
    b.insert_before(&eligible, &detached_root, None);
    b.insert_before(&connected, parent, None);
    let c = apply(&mut dom, b);
    let connected = c.created(&connected).unwrap().unwrap();
    let detached = c.created(&detached).unwrap().unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    let copied_true = b.import_node(&old, connected, d, false);
    let copied_false = b.import_node(&old, detached, d, false);
    let c = apply(&mut dom, b);
    let copied_true = c.created(&copied_true).unwrap().unwrap();
    let copied_false = c.created(&copied_false).unwrap().unwrap();
    let s = dom.snapshot();
    let clone_script = |host| {
        let root = s.node(host).unwrap().hosted_shadow_root().unwrap();
        s.node(root).unwrap().children()[0]
    };
    let true_script = clone_script(copied_true);
    let false_script = clone_script(copied_false);
    assert!(script(&s, true_script).flags().already_processed());
    assert!(!script(&s, false_script).flags().already_processed());
    for n in [true_script, false_script] {
        assert_eq!(script(&s, n).origin(), DomSvgScriptOrigin::Clone);
        assert!(!s.node(n).unwrap().is_connected());
        assert_eq!(s.node(n).unwrap().owner_document(), Some(d));
    }
    let mut b = DomBatch::new();
    b.insert_before(copied_true, parent, None);
    b.insert_before(copied_false, parent, None);
    apply(&mut dom, b);
    let after = dom.snapshot();
    assert!(script(&after, true_script).flags().already_processed());
    assert!(script(&after, false_script).flags().already_processed());
    assert!(!script(&s, false_script).flags().already_processed());
}

#[test]
fn insert_then_detach_leaves_processed_state_and_later_edits_do_not_restart() {
    let mut dom = dom();
    let (d, root) = setup(&mut dom);
    let mut b = DomBatch::new();
    let n = b.create_element_ns(d, Namespace::svg(), "script");
    let c = apply(&mut dom, b);
    let n = c.created(&n).unwrap().unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.insert_before(n, root, None);
    b.detach(n);
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(s.node(n).unwrap().parent(), None);
    assert!(script(&s, n).flags().already_processed());
    assert_eq!(c.changes().changes(), &[DomChange::ElementState(n)]);
    assert!(!script(&old, n).flags().already_processed());
    let mut b = DomBatch::new();
    let d2 = b.create_document();
    b.adopt(n, &d2);
    b.set_attribute(n, "href", "data:ignored");
    let text = b.create_text(&d2, "later");
    b.insert_before(&text, n, None);
    b.insert_before(n, &d2, None);
    b.replace_data(&text, 0, usize::MAX, "edited");
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    assert!(script(&s, n).flags().already_processed());
    assert_eq!(script(&s, n).origin(), DomSvgScriptOrigin::Native);
    assert_eq!(
        s.node(n).unwrap().owner_document(),
        Some(c.document(&d2).unwrap())
    );
    let before = s.clone();
    let mut b = DomBatch::new();
    b.set_attribute(n, "href", "data:ignored");
    b.replace_data(c.created(&text).unwrap().unwrap(), 0, usize::MAX, "edited");
    let c = apply(&mut dom, b);
    assert_eq!(c.changes().changes(), &[]);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn native_installation_processes_svg_and_clone_history_survives_disconnected_template_contents() {
    let mut dom = dom();
    let (d, root) = setup(&mut dom);
    let mut b = surgeist_html::HtmlFragment::builder(surgeist_html::HtmlConstructionLimits::new(
        16, 16, 16, 1024, 16,
    ))
    .unwrap();
    let n = b
        .append_element(
            &b.root(),
            QualifiedName::new(Namespace::svg(), None, "script"),
            OrderedAttributes::default(),
        )
        .unwrap();
    b.append_text(&n, "ignored execution").unwrap();
    let source = b.finish();
    let subject = source.root().children().next().unwrap();
    let result = dom
        .install_native_html_fragment(
            dom.snapshot().document(d).unwrap().stamp(),
            root,
            None,
            &source,
            DomNativeHtmlInstallLimits::new(16, 100000, 65536),
            &AtomicBool::new(false),
        )
        .unwrap();
    let n = result.node_for(&subject).unwrap();
    assert!(script(result.snapshot(), n).flags().already_processed());
    assert_eq!(
        script(result.snapshot(), n).origin(),
        DomSvgScriptOrigin::Native
    );
    let mut b = DomBatch::new();
    let template = b.create_element_ns(d, Namespace::html(), "template");
    b.insert_before(&template, root, None);
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    let contents = s
        .node(c.created(&template).unwrap().unwrap())
        .unwrap()
        .as_template()
        .unwrap()
        .contents();
    let inert = s.node(contents).unwrap().owner_document().unwrap();
    let mut b = DomBatch::new();
    let copy = b.import_node(result.snapshot(), n, inert, true);
    b.insert_before(&copy, contents, None);
    let c = apply(&mut dom, b);
    let copy = c.created(&copy).unwrap().unwrap();
    assert!(script(&dom.snapshot(), copy).flags().already_processed());
    assert!(!dom.snapshot().node(copy).unwrap().is_connected());
}

#[test]
fn svg_and_html_scripts_preserve_real_slot_intentions_and_distinct_processing_state() {
    let mut dom = dom();
    let (d, host) = setup(&mut dom);
    let mut b = DomBatch::new();
    let shadow = b.attach_shadow(host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let slot = b.create_element_ns(d, Namespace::html(), "slot");
    b.insert_before(&slot, &shadow, None);
    let fragment = b.create_fragment(d);
    let svg = b.create_element_ns(d, Namespace::svg(), "script");
    let html = b.create_element_ns(d, Namespace::html(), "script");
    b.set_attribute(&html, "src", "");
    b.insert_before(&svg, &fragment, None);
    b.insert_before(&html, &fragment, None);
    let c = apply(&mut dom, b);
    let fragment = c.created(&fragment).unwrap().unwrap();
    let svg = c.created(&svg).unwrap().unwrap();
    let html = c.created(&html).unwrap().unwrap();
    let slot = c.created(&slot).unwrap().unwrap();
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.insert_before(fragment, host, None);
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    assert!(script(&s, svg).flags().already_processed());
    assert!(s.node(svg).unwrap().as_html_script().is_none());
    let actual = s.node(html).unwrap().as_html_script().unwrap();
    assert!(actual.flags().already_started());
    assert_eq!(actual.preparation_document(), Some(d));
    assert_eq!(
        actual.script_type(),
        Some(surgeist_html::HtmlScriptKind::Classic)
    );
    assert!(s.node(html).unwrap().as_svg_script().is_none());
    assert_eq!(s.slot(slot).unwrap().assigned_nodes(), &[svg, html]);
    assert!(c.changes().changes().contains(&DomChange::IntrinsicTasks));
    let notifications = slot_checkpoint(
        &mut dom,
        DomSlotQueryLimits::new(256, 1024).unwrap(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(notifications.targets(), &[slot]);
    assert!(dom.snapshot().pending_slot_signals().is_empty());
    assert!(!script(&before, svg).flags().already_processed());
    assert!(
        !before
            .node(html)
            .unwrap()
            .as_html_script()
            .unwrap()
            .flags()
            .already_started()
    );
    let captured = s
        .html_capture(
            d,
            None,
            surgeist_html_semantics::HtmlCaptureLimits::new(256, 1024, 256, 65536, 256, 0, 0),
            DomHtmlCaptureLimits::new(256, 1000000),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(script(captured.source(), svg).flags().already_processed());
}

#[test]
fn exact_node_command_admission_and_state_only_history_fallback_remain_finite() {
    for (nodes, commands) in [(1, 3), (2, 2), (2, 3)] {
        let mut dom =
            Dom::new(DomLimits::new(nodes, 63, commands, 0, 0, 4, 8192).unwrap()).unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let svg = b.create_element_ns(&d, Namespace::svg(), "script");
        b.insert_before(&svg, &d, None);
        let result = dom.apply(b, &AtomicBool::new(false));
        if nodes == 1 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::ManagedNodes,
                    limit: 1
                }
            );
            unchanged(&dom, &before, usage);
        } else if commands == 2 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::Commands,
                    limit: 2
                }
            );
            unchanged(&dom, &before, usage);
        } else {
            let c = result.unwrap();
            assert!(
                script(&dom.snapshot(), c.created(&svg).unwrap().unwrap())
                    .flags()
                    .already_processed()
            );
            assert_eq!(dom.usage().managed_nodes(), 2);
        }
    }
    let mut dom = Dom::new(DomLimits::new(3, 94, 4, 0, 0, 0, 0).unwrap()).unwrap();
    let (d, root) = setup(&mut dom);
    let mut b = DomBatch::new();
    let svg = b.create_element_ns(d, Namespace::svg(), "script");
    let c = apply(&mut dom, b);
    let svg = c.created(&svg).unwrap().unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.insert_before(svg, root, None);
    b.detach(svg);
    apply(&mut dom, b);
    assert!(script(&dom.snapshot(), svg).flags().already_processed());
    assert!(!script(&old, svg).flags().already_processed());
    assert!(matches!(
        dom.changes_since(old.stamp()),
        Err(DomHistoryError::FullRecomputeRequired { .. })
    ));
}

#[test]
fn scalar_state_adds_no_string_cost_and_limits_late_failure_retirement_are_atomic() {
    for cap in [62, 63] {
        let mut dom = Dom::new(DomLimits::new(2, cap, 8, 0, 0, 4, 8192).unwrap()).unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let n = b.create_element_ns(&d, Namespace::svg(), "script");
        let result = dom.apply(b, &AtomicBool::new(false));
        // Native Document31 +SVG namespace26 +local script6 =63 units.
        if cap == 62 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 62
                }
            );
            unchanged(&dom, &before, usage);
        } else {
            let c = result.unwrap();
            assert_eq!(dom.usage().string_code_units(), 63);
            assert!(
                !script(&dom.snapshot(), c.created(&n).unwrap().unwrap())
                    .flags()
                    .already_processed()
            );
        }
    }
    let mut dom = dom();
    let (d, root) = setup(&mut dom);
    let mut b = DomBatch::new();
    let n = b.create_element_ns(d, Namespace::svg(), "script");
    let c = apply(&mut dom, b);
    let n = c.created(&n).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.insert_before(n, root, None);
    b.insert_before(root, n, None);
    assert_eq!(
        dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    unchanged(&dom, &before, usage);
    let mut b = DomBatch::new();
    b.insert_before(n, root, None);
    assert_eq!(
        dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    unchanged(&dom, &before, usage);
    let mut b = DomBatch::new();
    b.insert_before(n, root, None);
    apply(&mut dom, b);
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.detach(n);
    b.destroy(n);
    let replacement = b.create_element_ns(d, Namespace::svg(), "script");
    let c = apply(&mut dom, b);
    let replacement = c.created(&replacement).unwrap().unwrap();
    assert_ne!(n, replacement);
    assert!(matches!(dom.snapshot().node(n), Err(DomError::StaleNode)));
    assert!(
        !script(&dom.snapshot(), replacement)
            .flags()
            .already_processed()
    );
    assert!(script(&old, n).flags().already_processed());
}

fn slot_checkpoint(
    dom: &mut Dom,
    slots: DomSlotQueryLimits,
    cancel: &AtomicBool,
) -> Result<surgeist_dom::DomSlotNotifications, DomError> {
    match dom.native_checkpoint_step(
        surgeist_dom::DomNativeCheckpointLimits::new(0, 0, slots).unwrap(),
        cancel,
    )? {
        surgeist_dom::DomNativeCheckpoint::Slots(n) => Ok(n),
        other => panic!("expected reached Slot marker, got {other:?}"),
    }
}

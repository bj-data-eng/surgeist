//! Literal oracles: HTML877d Canvas none-mode and obsolete Marquee algorithms.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomError, DomLimits, DomResource,
    DomSnapshot, Namespace, NodeId,
};
use surgeist_html::HtmlCanvasBitmapDimension;
const HTML: &str = "http://www.w3.org/1999/xhtml";
fn dom() -> Dom {
    Dom::new(DomLimits::new(128, 32768, 256, 0, 0, 16, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId, NodeId) {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let canvas = batch.create_element_ns(&document, Namespace::from_uri(HTML), "canvas");
    let marquee = batch.create_element_ns(&document, Namespace::from_uri(HTML), "marquee");
    let commit = apply(dom, batch);
    (
        commit.document(&document).unwrap(),
        commit.created(&canvas).unwrap().unwrap(),
        commit.created(&marquee).unwrap().unwrap(),
    )
}
fn dimensions(snapshot: &DomSnapshot, canvas: NodeId) -> (String, String) {
    fn value(dimension: HtmlCanvasBitmapDimension<'_>) -> String {
        match dimension {
            HtmlCanvasBitmapDimension::Default(value) => value.to_string(),
            HtmlCanvasBitmapDimension::Parsed(value) => value.decimal_digits().to_owned(),
        }
    }
    let state = snapshot
        .node(canvas)
        .unwrap()
        .as_element()
        .unwrap()
        .canvas_none_state()
        .unwrap();
    assert!(state.context_is_none());
    assert!(!state.has_bound_context());
    assert!(state.origin_clean());
    assert!(state.bitmap_is_transparent_black());
    (value(state.width()), value(state.height()))
}
fn marquee(snapshot: &DomSnapshot, node: NodeId) -> (bool, u128) {
    let state = snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .marquee_state()
        .unwrap();
    (state.turned_on(), state.current_loop_index())
}

#[test]
fn native_families_have_real_fresh_values_and_exact_namespace_case_applicability() {
    let mut dom = dom();
    let (document, canvas, moving) = setup(&mut dom);
    assert_eq!(
        dimensions(&dom.snapshot(), canvas),
        ("300".into(), "150".into())
    );
    assert_eq!(marquee(&dom.snapshot(), moving), (true, 0));
    let mut batch = DomBatch::new();
    let cases = [
        (Namespace::none(), "canvas"),
        (Namespace::from_uri(HTML), "Canvas"),
        (Namespace::from_uri("http://www.w3.org/2000/svg"), "canvas"),
        (
            Namespace::from_uri("http://www.w3.org/1998/Math/MathML"),
            "marquee",
        ),
    ];
    let tickets: Vec<_> = cases
        .into_iter()
        .map(|(ns, name)| batch.create_element_ns(document, ns, name))
        .collect();
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    for ticket in tickets {
        let element = snapshot
            .node(commit.created(&ticket).unwrap().unwrap())
            .unwrap()
            .as_element()
            .unwrap();
        assert!(element.canvas_none_state().is_none());
        assert!(element.marquee_state().is_none());
    }
}

#[test]
fn bitmap_dimensions_use_shared_prefix_recovery_and_preserve_exact_raw_utf16() {
    let mut dom = dom();
    let (_, canvas, _) = setup(&mut dom);
    let huge = "99999999999999999999999999999999999999999999999";
    for (raw, expected) in [
        ("", "300"),
        ("-1", "300"),
        ("-0", "0"),
        ("0", "0"),
        ("+0080tail", "80"),
        (" \t12.5", "12"),
        ("000300", "300"),
        ("2147483648", "2147483648"),
        ("4294967296", "4294967296"),
        (huge, huge),
    ] {
        let mut batch = DomBatch::new();
        batch.set_attribute(canvas, "width", raw);
        apply(&mut dom, batch);
        assert_eq!(
            dimensions(&dom.snapshot(), canvas),
            (expected.into(), "150".into())
        );
    }
    for (raw, expected) in [
        (DocumentString::from_utf16(vec![0xd800, 0x31]), "300"),
        (DocumentString::from_utf16(vec![0x31, 0xd800]), "1"),
    ] {
        let mut batch = DomBatch::new();
        batch.set_attribute(canvas, "width", raw.clone());
        batch.set_attribute(canvas, "height", "0");
        apply(&mut dom, batch);
        let snapshot = dom.snapshot();
        assert_eq!(dimensions(&snapshot, canvas), (expected.into(), "0".into()));
        assert_eq!(
            snapshot
                .node(canvas)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"width".into()),
            Some(&raw)
        );
    }
}

#[test]
fn actual_expanded_attribute_identity_controls_dimensions_even_with_qualified_collisions() {
    let mut dom = dom();
    let (_, canvas, _) = setup(&mut dom);
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(canvas, Namespace::from_uri("urn:x"), "width", "1");
    batch.set_attribute(canvas, "width", "2"); // Updates first qualified spelling in urn:x.
    batch.set_attribute(canvas, "Width", "3");
    batch.set_attribute(canvas, "p:height", "4");
    apply(&mut dom, batch);
    assert_eq!(
        dimensions(&dom.snapshot(), canvas),
        ("300".into(), "150".into())
    );
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(canvas, Namespace::none(), "width", "100");
    batch.set_attribute(canvas, "width", "777"); // Still the earlier foreign attribute.
    apply(&mut dom, batch);
    assert_eq!(
        dimensions(&dom.snapshot(), canvas),
        ("100".into(), "150".into())
    );
    let mut batch = DomBatch::new();
    batch.remove_attribute(canvas, "width"); // Removes foreign first, not null-namespace.
    apply(&mut dom, batch);
    assert_eq!(
        dimensions(&dom.snapshot(), canvas),
        ("100".into(), "150".into())
    );
    let mut batch = DomBatch::new();
    batch.remove_attribute_ns(canvas, Namespace::none(), "width");
    apply(&mut dom, batch);
    assert_eq!(
        dimensions(&dom.snapshot(), canvas),
        ("300".into(), "150".into())
    );
}

#[test]
fn dimension_and_marquee_changes_have_exact_domains_old_captures_and_final_noops() {
    let mut dom = dom();
    let (_, canvas, moving) = setup(&mut dom);
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute(canvas, "width", "301");
    batch.stop_marquee(moving);
    let changes = apply(&mut dom, batch);
    assert!(
        changes
            .changes()
            .changes()
            .contains(&DomChange::Attributes(canvas))
    );
    assert!(
        changes
            .changes()
            .changes()
            .contains(&DomChange::ElementState(canvas))
    );
    assert!(
        changes
            .changes()
            .changes()
            .contains(&DomChange::ElementState(moving))
    );
    assert_eq!(dimensions(&old, canvas), ("300".into(), "150".into()));
    assert_eq!(marquee(&old, moving), (true, 0));
    let stamp = dom.snapshot().stamp();
    let mut batch = DomBatch::new();
    batch.set_attribute(canvas, "width", "301");
    batch.stop_marquee(moving);
    batch.start_marquee(moving);
    batch.stop_marquee(moving);
    apply(&mut dom, batch);
    assert_eq!(dom.snapshot().stamp(), stamp);
    let mut batch = DomBatch::new();
    batch.set_attribute(canvas, "width", "0301");
    let changes = apply(&mut dom, batch);
    assert_eq!(
        changes.changes().changes(),
        &[DomChange::Attributes(canvas)]
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(moving, "loop", "1");
    apply(&mut dom, batch);
    assert_eq!(marquee(&dom.snapshot(), moving), (false, 0));
    let mut batch = DomBatch::new();
    batch.start_marquee(moving);
    let changes = apply(&mut dom, batch);
    assert_eq!(
        changes.changes().changes(),
        &[DomChange::ElementState(moving)]
    );
}

#[test]
fn native_import_uses_copied_dimensions_but_fresh_on_marquee_instead_of_snapshot_copy() {
    let mut source = dom();
    let (document, canvas, moving) = setup(&mut source);
    let mut batch = DomBatch::new();
    batch.set_attribute(canvas, "width", "00400tail");
    batch.stop_marquee(moving);
    batch.set_attribute(moving, "nonce", "content");
    batch.set_nonce(moving, "slot");
    apply(&mut source, batch);
    let old = source.snapshot();
    let mut batch = DomBatch::new();
    batch.destroy(canvas);
    batch.destroy(moving);
    apply(&mut source, batch);
    drop(source);
    assert_eq!(marquee(&old, moving), (false, 0));
    let mut target = dom();
    let mut batch = DomBatch::new();
    let destination = batch.create_document();
    let copied_canvas = batch.import_node(&old, canvas, &destination, false);
    let copied_marquee = batch.import_node(&old, moving, &destination, false);
    let commit = apply(&mut target, batch);
    let copied_canvas = commit.created(&copied_canvas).unwrap().unwrap();
    let copied_marquee = commit.created(&copied_marquee).unwrap().unwrap();
    let snapshot = target.snapshot();
    assert_ne!(
        snapshot.node(copied_canvas).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(
        dimensions(&snapshot, copied_canvas),
        ("400".into(), "150".into())
    );
    assert_eq!(marquee(&snapshot, copied_marquee), (true, 0));
    assert_eq!(
        snapshot
            .node(copied_marquee)
            .unwrap()
            .as_element()
            .unwrap()
            .nonce(),
        Some(&"slot".into())
    );
}

#[test]
fn exact_limits_count_canonical_digits_but_inline_defaults_cost_no_digit_storage() {
    // 31 document +28 HTML URI +6 canvas =65; width name contributes5.
    for (raw, exact) in [("300", 73), ("301", 76), ("000300", 76), ("0", 72)] {
        for capacity in [exact - 1, exact] {
            let mut dom = Dom::new(DomLimits::new(2, capacity, 8, 0, 0, 0, 0).unwrap()).unwrap();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let canvas = batch.create_element_ns(&document, Namespace::from_uri(HTML), "canvas");
            batch.set_attribute(&canvas, "width", raw);
            let result = dom.apply(batch, &AtomicBool::new(false));
            if capacity < exact {
                assert_eq!(
                    result.unwrap_err(),
                    DomError::ResourceLimit {
                        resource: DomResource::StringCodeUnits,
                        limit: capacity
                    }
                );
                assert_eq!(dom.usage().managed_nodes(), 0);
            } else {
                let commit = result.unwrap();
                let canvas = commit.created(&canvas).unwrap().unwrap();
                assert_eq!(dom.usage().string_code_units(), exact);
                if raw == "301" {
                    let mut batch = DomBatch::new();
                    batch.set_attribute(canvas, "width", "300");
                    apply(&mut dom, batch);
                    assert_eq!(dom.usage().string_code_units(), 73);
                    let mut batch = DomBatch::new();
                    batch.set_attribute(canvas, "width", "301");
                    apply(&mut dom, batch);
                    assert_eq!(dom.usage().string_code_units(), 76);
                }
                let mut batch = DomBatch::new();
                batch.remove_attribute(canvas, "width");
                apply(&mut dom, batch);
                assert_eq!(dom.usage().string_code_units(), 65);
                let mut batch = DomBatch::new();
                batch.destroy(canvas);
                apply(&mut dom, batch);
                assert_eq!(dom.usage().string_code_units(), 31);
            }
        }
    }
}

#[test]
fn late_errors_and_cancelled_commands_preserve_all_prior_family_values() {
    let mut dom = dom();
    let (document, canvas, moving) = setup(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(canvas, "width", "12345");
    batch.stop_marquee(moving);
    batch.insert_before(document, canvas, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let mut batch = DomBatch::new();
    batch.stop_marquee(moving);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    for (node, error) in [
        (canvas, DomError::MarqueeInapplicable),
        (document.node(), DomError::WrongNodeKind),
    ] {
        let mut batch = DomBatch::new();
        batch.start_marquee(node);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error
        );
    }
    let mut other = self::dom();
    let (_, _, foreign) = setup(&mut other);
    let mut batch = DomBatch::new();
    batch.stop_marquee(foreign);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignOwner
    );
    let mut batch = DomBatch::new();
    batch.destroy(moving);
    apply(&mut dom, batch);
    let mut batch = DomBatch::new();
    batch.stop_marquee(moving);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::StaleNode
    );
}

#[test]
fn shadow_descendant_adoption_cloning_and_destruction_preserve_actual_family_semantics() {
    use surgeist_dom::{
        HtmlShadowRootMode as Mode, HtmlShadowRootProperties, HtmlSlotAssignment as Assignment,
    };
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    let root = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(Mode::Closed, Assignment::Named, false, true, false),
    );
    let canvas = batch.create_element_ns(&document, Namespace::from_uri(HTML), "canvas");
    let moving = batch.create_element_ns(&document, Namespace::from_uri(HTML), "marquee");
    let text = batch.create_text(&document, "before");
    batch.set_attribute(&canvas, "width", "42");
    batch.stop_marquee(&moving);
    batch.insert_before(&text, &canvas, None);
    batch.insert_before(&canvas, &root, None);
    batch.insert_before(&moving, &root, None);
    batch.insert_before(&host, &document, None);
    let commit = apply(&mut dom, batch);
    let target = commit.document(&target).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let canvas = commit.created(&canvas).unwrap().unwrap();
    let moving = commit.created(&moving).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.adopt(host, target);
    batch.replace_data(text, 0, 6, "after");
    apply(&mut dom, batch);
    let capture = dom.snapshot();
    assert_eq!(capture.node(canvas).unwrap().owner_document(), Some(target));
    assert_eq!(dimensions(&capture, canvas), ("42".into(), "150".into()));
    assert_eq!(marquee(&capture, moving), (false, 0));
    let mut batch = DomBatch::new();
    let copied_host = batch.import_node(&capture, host, target, false);
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let copied_host = commit.created(&copied_host).unwrap().unwrap();
    let copied_root = snapshot
        .node(copied_host)
        .unwrap()
        .hosted_shadow_root()
        .unwrap();
    let children = snapshot.node(copied_root).unwrap().children();
    assert_eq!(
        dimensions(&snapshot, children[0]),
        ("42".into(), "150".into())
    );
    assert_eq!(marquee(&snapshot, children[1]), (true, 0));
    let mut batch = DomBatch::new();
    batch.destroy(host);
    apply(&mut dom, batch);
    assert!(matches!(
        dom.snapshot().node(canvas),
        Err(DomError::StaleNode)
    ));
    assert_eq!(dimensions(&old, canvas), ("42".into(), "150".into()));
    assert_eq!(marquee(&old, moving), (false, 0));
}

#[test]
fn family_changes_publish_complete_history_or_explicit_fallback_under_zero_history_budget() {
    for history in [0, 8] {
        let mut dom = Dom::new(DomLimits::new(8, 1024, 16, 0, 0, history, 65536).unwrap()).unwrap();
        let (_, canvas, moving) = setup(&mut dom);
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_attribute(canvas, "height", "12");
        batch.stop_marquee(moving);
        let commit = apply(&mut dom, batch);
        if history == 0 {
            assert!(matches!(
                dom.changes_since(before.stamp()),
                Err(surgeist_dom::DomHistoryError::FullRecomputeRequired { .. })
            ));
        } else {
            assert_eq!(
                dom.changes_since(before.stamp()).unwrap().changes(),
                commit.changes().changes()
            );
        }
        assert_eq!(
            dimensions(&dom.snapshot(), canvas),
            ("300".into(), "12".into())
        );
        assert_eq!(marquee(&dom.snapshot(), moving), (false, 0));
    }
}

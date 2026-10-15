//! Composed DOM3071 HTML case rules with HTML877d native Canvas/Marquee state.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomCommit, DomError, DomLimits,
    DomResource, DomSnapshot, Namespace, NodeId,
};
use surgeist_html::HtmlCanvasBitmapDimension;
const HTML: &str = "http://www.w3.org/1999/xhtml";
fn owner() -> Dom {
    Dom::new(DomLimits::new(128, 32768, 256, 16, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn documents(dom: &mut Dom) -> (DocumentId, DocumentId) {
    let mut batch = DomBatch::new();
    let xml = batch.create_document();
    let html = batch.create_html_document(&xml, None);
    let commit = apply(dom, batch);
    (
        commit.document(&xml).unwrap(),
        commit.document(&html).unwrap(),
    )
}
fn dimensions(snapshot: &DomSnapshot, canvas: NodeId) -> (String, String) {
    fn value(value: HtmlCanvasBitmapDimension<'_>) -> String {
        match value {
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
fn moving(snapshot: &DomSnapshot, node: NodeId) -> (bool, u128) {
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
fn html_whole_name_creation_selects_real_canvas_and_marquee_families_after_ascii_folding() {
    let mut dom = owner();
    let (_, html) = documents(&mut dom);
    let mut batch = DomBatch::new();
    let cases = [
        ("canvas", "canvas", "CANVAS", true),
        ("CaNvAs", "canvas", "CANVAS", true),
        ("marquee", "marquee", "MARQUEE", false),
        ("MARQUEE", "marquee", "MARQUEE", false),
    ];
    let tickets: Vec<_> = cases
        .iter()
        .map(|(name, _, _, _)| batch.create_element(html, *name))
        .collect();
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    for ((_, local, display, canvas), ticket) in cases.iter().zip(&tickets) {
        let id = commit.created(ticket).unwrap().unwrap();
        let node = snapshot.node(id).unwrap();
        let element = node.as_element().unwrap();
        assert_eq!(node.node_name(), DocumentString::from(*display));
        assert_eq!(element.name().namespace(), &Namespace::from_uri(HTML));
        assert_eq!(element.name().prefix(), None);
        assert_eq!(element.name().local_name(), &DocumentString::from(*local));
        assert_eq!(node.owner_document(), Some(html));
        if *canvas {
            assert_eq!(dimensions(&snapshot, id), ("300".into(), "150".into()));
            assert!(element.marquee_state().is_none());
        } else {
            assert_eq!(moving(&snapshot, id), (true, 0));
            assert!(element.canvas_none_state().is_none());
        }
    }
}
#[test]
fn html_uppercase_attribute_inputs_resize_canvas_but_raw_ns_uppercase_names_remain_distinct() {
    let mut dom = owner();
    let (_, html) = documents(&mut dom);
    let mut batch = DomBatch::new();
    let canvas = batch.create_element(html, "CANVAS");
    let commit = apply(&mut dom, batch);
    let canvas = commit.created(&canvas).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(canvas, Namespace::none(), "WIDTH", "1");
    batch.set_attribute_ns(canvas, Namespace::none(), "HEIGHT", "2");
    let changes = apply(&mut dom, batch);
    let old = dom.snapshot();
    assert_eq!(dimensions(&old, canvas), ("300".into(), "150".into()));
    assert_eq!(
        changes.changes().changes(),
        &[DomChange::Attributes(canvas)]
    );
    let node = old.node(canvas).unwrap();
    assert_eq!(node.attribute(&"WIDTH".into()), None);
    assert_eq!(
        node.attribute_ns(&Namespace::none(), &"WIDTH".into()),
        Some(&"1".into())
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(canvas, "WIDTH", "+00400tail");
    batch.set_attribute(canvas, "HEIGHT", " 20.5");
    let changes = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let node = snapshot.node(canvas).unwrap();
    assert_eq!(dimensions(&snapshot, canvas), ("400".into(), "20".into()));
    assert_eq!(node.as_element().unwrap().attributes().len(), 4);
    assert_eq!(node.attribute(&"WIDTH".into()), Some(&"+00400tail".into()));
    assert_eq!(
        node.attribute_ns(&Namespace::none(), &"WIDTH".into()),
        Some(&"1".into())
    );
    assert_eq!(
        node.attribute_ns(&Namespace::none(), &"height".into()),
        Some(&" 20.5".into())
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
            .contains(&DomChange::Attributes(canvas))
    );
    assert_eq!(dimensions(&old, canvas), ("300".into(), "150".into()));
    let mut batch = DomBatch::new();
    batch.remove_attribute(canvas, "WIDTH");
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(dimensions(&snapshot, canvas), ("300".into(), "20".into()));
    assert_eq!(
        snapshot.node(canvas).unwrap().attribute(&"WIDTH".into()),
        None
    );
    assert_eq!(
        snapshot
            .node(canvas)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"WIDTH".into()),
        Some(&"1".into())
    );
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(canvas, Namespace::from_uri("urn:x"), "width", "9");
    batch.set_attribute(canvas, "WIDTH", "10");
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(dimensions(&snapshot, canvas), ("300".into(), "20".into()));
    assert_eq!(
        snapshot
            .node(canvas)
            .unwrap()
            .attribute_ns(&Namespace::from_uri("urn:x"), &"width".into()),
        Some(&"10".into())
    );
}
#[test]
fn adoption_preserves_family_state_and_import_reinitializes_families_while_raw_names_stay_exact() {
    let mut dom = owner();
    let (xml, html) = documents(&mut dom);
    let mut batch = DomBatch::new();
    let canvas = batch.create_element(html, "CANVAS");
    let marquee = batch.create_element(html, "MARQUEE");
    batch.set_attribute_ns(&canvas, Namespace::none(), "WIDTH", "9");
    batch.set_attribute(&canvas, "WIDTH", "42");
    batch.set_attribute(&marquee, "DATA-X", "value");
    batch.stop_marquee(&marquee);
    let commit = apply(&mut dom, batch);
    let canvas = commit.created(&canvas).unwrap().unwrap();
    let marquee = commit.created(&marquee).unwrap().unwrap();
    let old_html = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.adopt(canvas, xml);
    batch.adopt(marquee, xml);
    let changes = apply(&mut dom, batch);
    let in_xml = dom.snapshot();
    for (id, display) in [(canvas, "canvas"), (marquee, "marquee")] {
        let node = in_xml.node(id).unwrap();
        assert_eq!(node.owner_document(), Some(xml));
        assert_eq!(node.node_name(), display.into());
        assert_eq!(
            node.as_element().unwrap().name(),
            old_html.node(id).unwrap().as_element().unwrap().name()
        );
        assert_eq!(
            node.as_element().unwrap().attributes(),
            old_html
                .node(id)
                .unwrap()
                .as_element()
                .unwrap()
                .attributes()
        );
    }
    assert_eq!(dimensions(&in_xml, canvas), ("42".into(), "150".into()));
    assert_eq!(moving(&in_xml, marquee), (false, 0));
    assert_eq!(
        in_xml.node(canvas).unwrap().attribute(&"WIDTH".into()),
        Some(&"9".into())
    );
    assert_eq!(
        old_html.node(canvas).unwrap().attribute(&"WIDTH".into()),
        Some(&"42".into())
    );
    assert_eq!(
        changes.changes().changes(),
        &[
            DomChange::NodeDocument(canvas),
            DomChange::NodeDocument(marquee)
        ]
    );
    let mut batch = DomBatch::new();
    let to_html_canvas = batch.import_node(&in_xml, canvas, html, false);
    let to_html_marquee = batch.import_node(&in_xml, marquee, html, false);
    let to_xml_canvas = batch.import_node(&old_html, canvas, xml, false);
    let to_xml_marquee = batch.import_node(&old_html, marquee, xml, false);
    let commit = apply(&mut dom, batch);
    let copied = dom.snapshot();
    for (canvas_ticket, marquee_ticket, document, display, lookup) in [
        (&to_html_canvas, &to_html_marquee, html, "CANVAS", "42"),
        (&to_xml_canvas, &to_xml_marquee, xml, "canvas", "9"),
    ] {
        let copied_canvas = commit.created(canvas_ticket).unwrap().unwrap();
        let copied_marquee = commit.created(marquee_ticket).unwrap().unwrap();
        assert_ne!(copied_canvas, canvas);
        assert_ne!(copied_marquee, marquee);
        assert_eq!(
            dimensions(&copied, copied_canvas),
            ("42".into(), "150".into())
        );
        assert_eq!(moving(&copied, copied_marquee), (true, 0));
        let node = copied.node(copied_canvas).unwrap();
        assert_eq!(node.owner_document(), Some(document));
        assert_eq!(node.node_name(), display.into());
        assert_eq!(node.attribute(&"WIDTH".into()), Some(&lookup.into()));
        assert_eq!(
            node.as_element().unwrap().name(),
            old_html.node(canvas).unwrap().as_element().unwrap().name()
        );
        assert_eq!(
            node.as_element().unwrap().attributes(),
            old_html
                .node(canvas)
                .unwrap()
                .as_element()
                .unwrap()
                .attributes()
        );
    }
    let mut batch = DomBatch::new();
    batch.adopt(canvas, html);
    batch.adopt(marquee, html);
    apply(&mut dom, batch);
    let back = dom.snapshot();
    assert_eq!(back.node(canvas).unwrap().node_name(), "CANVAS".into());
    assert_eq!(
        back.node(canvas).unwrap().attribute(&"WIDTH".into()),
        Some(&"42".into())
    );
    assert_eq!(dimensions(&back, canvas), ("42".into(), "150".into()));
    assert_eq!(moving(&back, marquee), (false, 0));
    assert_eq!(in_xml.node(canvas).unwrap().node_name(), "canvas".into());
}
#[test]
fn composed_html_canvas_dimension_storage_obeys_independent_exact_capacity() {
    // Source XML 31 + HTML skeleton 125 + (HTML URI 28 + canvas 6) +
    // lowercase width name 5 + raw 301 3 + derived canonical digits 3 = 201.
    for capacity in [200, 201] {
        let mut dom = Dom::new(DomLimits::new(7, capacity, 8, 0, 0).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let source = batch.create_document();
        let html = batch.create_html_document(&source, None);
        let canvas = batch.create_element(&html, "CANVAS");
        batch.set_attribute(&canvas, "WIDTH", "301");
        let result = dom.apply(batch, &AtomicBool::new(false));
        if capacity == 200 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 200
                }
            );
            assert_eq!(dom.usage().managed_nodes(), 0);
            assert_eq!(dom.usage().string_code_units(), 0);
        } else {
            let commit = result.unwrap();
            let canvas = commit.created(&canvas).unwrap().unwrap();
            assert_eq!(dom.usage().managed_nodes(), 7);
            assert_eq!(dom.usage().string_code_units(), 201);
            assert_eq!(
                dimensions(&dom.snapshot(), canvas),
                ("301".into(), "150".into())
            );
            let mut batch = DomBatch::new();
            batch.set_attribute(canvas, "WIDTH", "300");
            apply(&mut dom, batch);
            assert_eq!(dom.usage().string_code_units(), 198);
            let mut batch = DomBatch::new();
            batch.remove_attribute(canvas, "WIDTH");
            apply(&mut dom, batch);
            assert_eq!(dom.usage().string_code_units(), 190);
        }
    }
}

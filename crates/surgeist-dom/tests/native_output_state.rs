//! Literal HTML877d Output state and DOM3071 tree outcomes.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html_semantics::*;

fn limits() -> DomLimits {
    DomLimits::new(256, 65536, 1024, 256, 256, 32, 1048576).unwrap()
}
fn apply(dom: &mut Dom, b: DomBatch) -> DomCommit {
    dom.apply(b, &AtomicBool::new(false)).unwrap()
}
fn setup() -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(limits()).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    let output = b.create_element_ns(&d, Namespace::html(), "output");
    b.insert_before(&root, &d, None);
    b.insert_before(&output, &root, None);
    let c = apply(&mut dom, b);
    (
        dom,
        c.document(&d).unwrap(),
        c.created(&root).unwrap().unwrap(),
        c.created(&output).unwrap().unwrap(),
    )
}
fn state(s: &DomSnapshot, n: NodeId) -> &DomOutputState {
    s.node(n).unwrap().as_output().unwrap().state()
}
fn capture(s: &DomSnapshot, d: DocumentId) -> DomHtmlCapture {
    s.html_capture(
        d,
        None,
        HtmlCaptureLimits::new(256, 1024, 256, 65536, 256, 0, 0),
        DomHtmlCaptureLimits::new(256, 1000000),
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn fact(c: &DomHtmlCapture, n: NodeId, q: HtmlSemanticQuery) -> HtmlSemanticFact {
    c.query(
        &[HtmlSemanticRequest::new(c.subject(n).unwrap(), q)],
        surgeist_html::HtmlValueInterpreter::new(65536),
        HtmlSemanticLimits::new(1000000, 32, 1024, 1024, 1000000, 1024),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .results()[0]
        .fact()
        .clone()
}
fn read(
    c: &DomHtmlCapture,
    n: NodeId,
    q: HtmlSemanticQuery,
) -> (DocumentString, HtmlControlTextOrigin) {
    match fact(c, n, q) {
        HtmlSemanticFact::OutputValue(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::OutputDefaultValue(HtmlSemanticValue::Known(v)) => {
            (v.value().clone(), v.origin())
        }
        f => panic!("expected actual Output readback: {f:?}"),
    }
}
fn values(s: &DomSnapshot, d: DocumentId, n: NodeId) -> (DocumentString, DocumentString) {
    let c = capture(s, d);
    (
        read(&c, n, HtmlSemanticQuery::OutputValue).0,
        read(&c, n, HtmlSemanticQuery::OutputDefaultValue).0,
    )
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
fn fresh_null_override_real_form_owner_and_barred_validation() {
    let (mut dom, d, root, n) = setup();
    let s = dom.snapshot();
    assert_eq!(state(&s, n).default_value_override(), None);
    assert!(!state(&s, n).parser_inserted());
    assert!(state(&s, n).custom_validity_message().is_empty());
    assert_eq!(values(&s, d, n), ("".into(), "".into()));
    assert_eq!(s.node(n).unwrap().as_output().unwrap().form_owner(), None);
    let mut b = DomBatch::new();
    let form = b.create_element_ns(d, Namespace::html(), "form");
    b.insert_before(&form, root, None);
    b.insert_before(n, &form, None);
    b.set_custom_validity(n, "m\r\nn");
    let c = apply(&mut dom, b);
    let form = c.created(&form).unwrap().unwrap();
    let s = dom.snapshot();
    assert_eq!(
        s.node(n).unwrap().as_output().unwrap().form_owner(),
        Some(form)
    );
    assert_eq!(
        state(&s, n).custom_validity_message(),
        &DocumentString::from("m\nn")
    );
    let c = capture(&s, d);
    assert_eq!(
        fact(&c, n, HtmlSemanticQuery::ConstraintValidationCandidate),
        HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(false))
    );
    let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) =
        fact(&c, n, HtmlSemanticQuery::FormOwner)
    else {
        panic!("actual owner")
    };
    assert_eq!(owner.owner(), Some(&c.subject(form).unwrap()));
}

#[test]
fn descendant_text_is_exact_nested_physical_text_not_control_current_or_owned_contents() {
    let (mut dom, d, _, n) = setup();
    let mut b = DomBatch::new();
    let first = b.create_text(d, "A\r");
    let nested = b.create_element_ns(d, Namespace::html(), "textarea");
    b.set_textarea_default_value(
        &nested,
        DocumentString::from_utf16(vec![10, 0, 0xd800, 0xdc00]),
    );
    b.set_textarea_value(&nested, "ignored current");
    let foreign = b.create_element_ns(d, Namespace::from_uri("urn:foreign"), "x");
    let text = b.create_text(d, "foreign");
    let comment = b.create_comment(d, "ignored comment");
    let host = b.create_element_ns(d, Namespace::html(), "span");
    let shadow = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let shadow_text = b.create_text(d, "ignored shadow");
    let template = b.create_element_ns(d, Namespace::html(), "template");
    for node in [&first, &nested, &foreign, &comment, &host, &template] {
        b.insert_before(node, n, None);
    }
    b.insert_before(&text, &foreign, None);
    b.insert_before(&shadow_text, &shadow, None);
    let c = apply(&mut dom, b);
    let template = c.created(&template).unwrap().unwrap();
    let s = dom.snapshot();
    let contents = s.node(template).unwrap().as_template().unwrap().contents();
    let inert = s.node(contents).unwrap().owner_document().unwrap();
    let mut b = DomBatch::new();
    let ignored = b.create_text(inert, "ignored template contents");
    b.insert_before(&ignored, contents, None);
    apply(&mut dom, b);
    let expected = DocumentString::from_utf16(vec![
        65, 13, 10, 0, 0xd800, 0xdc00, 102, 111, 114, 101, 105, 103, 110,
    ]);
    assert_eq!(
        values(&dom.snapshot(), d, n),
        (expected.clone(), expected.clone())
    );
    let mut b = DomBatch::new();
    b.set_output_value(n, "new");
    apply(&mut dom, b);
    assert_eq!(
        state(&dom.snapshot(), n).default_value_override(),
        Some(&expected)
    );
    assert_eq!(values(&dom.snapshot(), d, n), ("new".into(), expected));
}

#[test]
fn value_freezes_default_default_setter_branches_and_reset_replaces_before_clear() {
    let (mut dom, d, _, n) = setup();
    let mut b = DomBatch::new();
    b.set_output_default_value(n, "initial\r\n");
    apply(&mut dom, b);
    assert_eq!(state(&dom.snapshot(), n).default_value_override(), None);
    let old = dom.snapshot();
    let old_text = old.node(n).unwrap().children()[0];
    let mut b = DomBatch::new();
    b.set_output_value(n, "first");
    b.set_output_value(n, "second");
    apply(&mut dom, b);
    assert_eq!(
        values(&dom.snapshot(), d, n),
        ("second".into(), "initial\r\n".into())
    );
    assert_eq!(dom.snapshot().node(old_text).unwrap().parent(), None);
    let child = dom.snapshot().node(n).unwrap().children()[0];
    let mut b = DomBatch::new();
    b.set_output_default_value(n, "override");
    apply(&mut dom, b);
    assert_eq!(dom.snapshot().node(n).unwrap().children(), &[child]);
    assert_eq!(
        values(&dom.snapshot(), d, n),
        ("second".into(), "override".into())
    );
    let mut b = DomBatch::new();
    b.replace_data(child, 0, usize::MAX, "edited");
    b.reset_output(n);
    apply(&mut dom, b);
    assert_eq!(
        values(&dom.snapshot(), d, n),
        ("override".into(), "override".into())
    );
    assert_eq!(state(&dom.snapshot(), n).default_value_override(), None);
    assert_ne!(dom.snapshot().node(n).unwrap().children(), &[child]);
    assert_eq!(
        values(&old, d, n),
        ("initial\r\n".into(), "initial\r\n".into())
    );
}

#[test]
fn empty_override_and_state_only_history_survive_initial_and_slot_capture() {
    let (mut dom, d, root, n) = setup();
    let before = dom.snapshot();
    let old = capture(&before, d);
    let mut b = DomBatch::new();
    b.set_output_value(n, "");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_ne!(s.stamp(), before.stamp());
    assert!(
        !dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    assert_eq!(
        state(&s, n).default_value_override(),
        Some(&DocumentString::default())
    );
    assert_eq!(
        read(&old, n, HtmlSemanticQuery::OutputDefaultValue),
        ("".into(), HtmlControlTextOrigin::DescendantText)
    );
    let mut b = DomBatch::new();
    let slot = b.create_element_ns(d, Namespace::html(), "slot");
    b.insert_before(&slot, root, None);
    b.assign_slot(&slot, vec![n.into()]);
    b.set_attribute(n, "value", "not a default");
    let text = b.create_text(d, "live");
    b.insert_before(&text, n, None);
    apply(&mut dom, b);
    let s = dom.snapshot();
    for supplement in [false, true] {
        let c = if supplement {
            capture(&s, d)
                .with_slot_semantics(
                    DomSlotSemanticLimits::new(
                        DomSlotQueryLimits::new(256, 1024).unwrap(),
                        DomHtmlCaptureLimits::new(256, 1000000),
                        HtmlLanguageDirectionLimits::new(512, 1024, 0, 1000000),
                        HtmlInertFactLimits::new(512, 1000000),
                    ),
                    &AtomicBool::new(false),
                )
                .unwrap()
        } else {
            capture(&s, d)
        };
        assert_eq!(
            read(&c, n, HtmlSemanticQuery::OutputValue).0,
            DocumentString::from("live")
        );
        assert_eq!(
            read(&c, n, HtmlSemanticQuery::OutputDefaultValue),
            ("".into(), HtmlControlTextOrigin::OutputDefaultOverride)
        );
    }
    let mut b = DomBatch::new();
    b.reset_output(n);
    apply(&mut dom, b);
    assert!(dom.snapshot().node(n).unwrap().children().is_empty());
    assert_eq!(state(&dom.snapshot(), n).default_value_override(), None);
}

#[test]
fn reset_null_flattens_nested_text_and_equal_value_still_replaces_identity() {
    let (mut dom, d, _, n) = setup();
    let mut b = DomBatch::new();
    let span = b.create_element_ns(d, Namespace::html(), "span");
    let text = b.create_text(d, "abc");
    b.insert_before(&span, n, None);
    b.insert_before(&text, &span, None);
    let c = apply(&mut dom, b);
    let span = c.created(&span).unwrap().unwrap();
    let text = c.created(&text).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.reset_output(n);
    apply(&mut dom, b);
    let s = dom.snapshot();
    let replacement = s.node(n).unwrap().children()[0];
    assert!(
        matches!(s.node(replacement).unwrap().kind(),DomNodeKind::Text(v) if v == &DocumentString::from("abc"))
    );
    assert_eq!(s.node(span).unwrap().parent(), None);
    assert_eq!(s.node(text).unwrap().parent(), Some(span));
    assert_eq!(state(&s, n).default_value_override(), None);
    let mut b = DomBatch::new();
    b.set_output_value(n, "abc");
    apply(&mut dom, b);
    assert_ne!(dom.snapshot().node(n).unwrap().children()[0], replacement);
    assert_eq!(dom.snapshot().node(replacement).unwrap().parent(), None);
    assert_eq!(values(&dom.snapshot(), d, n), ("abc".into(), "abc".into()));
}

#[test]
fn clone_import_are_fresh_while_adoption_preserves_established_override() {
    let (mut dom, d, _, n) = setup();
    let mut b = DomBatch::new();
    b.set_output_default_value(n, "default");
    b.set_output_value(n, "current");
    b.set_custom_validity(n, "error");
    apply(&mut dom, b);
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    let shallow = b.import_node(&old, n, d, false);
    let deep = b.import_node(&old, n, d, true);
    let destination = b.create_document();
    b.adopt(n, &destination);
    let c = apply(&mut dom, b);
    let shallow = c.created(&shallow).unwrap().unwrap();
    let deep = c.created(&deep).unwrap().unwrap();
    let s = dom.snapshot();
    for node in [shallow, deep] {
        assert_eq!(state(&s, node).default_value_override(), None);
        assert!(state(&s, node).custom_validity_message().is_empty());
        assert!(!state(&s, node).parser_inserted());
        assert_eq!(
            s.node(node).unwrap().as_output().unwrap().form_owner(),
            None
        );
    }
    assert_eq!(values(&s, d, shallow), ("".into(), "".into()));
    assert_eq!(values(&s, d, deep), ("current".into(), "current".into()));
    assert_eq!(state(&s, n), state(&old, n));
    assert_eq!(
        values(&s, c.document(&destination).unwrap(), n),
        ("current".into(), "default".into())
    );
    let mut other = Dom::new(limits()).unwrap();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let imported = b.import_node(&old, n, &doc, true);
    let c = apply(&mut other, b);
    assert_eq!(
        values(
            &other.snapshot(),
            c.document(&doc).unwrap(),
            c.created(&imported).unwrap().unwrap()
        ),
        ("current".into(), "current".into())
    );
}

#[test]
fn replace_all_removes_actual_nested_controls_and_preserves_detached_nodes() {
    let (mut dom, d, root, n) = setup();
    let mut b = DomBatch::new();
    let form = b.create_element_ns(d, Namespace::html(), "form");
    b.insert_before(&form, root, None);
    b.insert_before(n, &form, None);
    let radio = b.create_element_ns(d, Namespace::html(), "input");
    b.set_attribute(&radio, "type", "radio");
    b.set_attribute(&radio, "name", "g");
    b.set_input_checked(&radio, true);
    b.insert_before(&radio, n, None);
    let textarea = b.create_element_ns(d, Namespace::html(), "textarea");
    b.set_textarea_default_value(&textarea, "old");
    b.insert_before(&textarea, n, None);
    let c = apply(&mut dom, b);
    let form = c.created(&form).unwrap().unwrap();
    let radio = c.created(&radio).unwrap().unwrap();
    let textarea = c.created(&textarea).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .node(radio)
            .unwrap()
            .as_input()
            .unwrap()
            .form_owner(),
        Some(form)
    );
    let mut b = DomBatch::new();
    b.set_output_value(n, "new");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(s.node(radio).unwrap().parent(), None);
    assert_eq!(
        s.node(radio).unwrap().as_input().unwrap().form_owner(),
        None
    );
    assert!(
        s.node(radio)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .flags()
            .checkedness()
    );
    assert_eq!(
        s.node(textarea)
            .unwrap()
            .as_textarea()
            .unwrap()
            .form_owner(),
        None
    );
    assert_eq!(values(&s, d, n), ("new".into(), "old".into()));
}

#[test]
fn logical_override_new_text_and_detached_text_have_exact_independent_costs() {
    let (mut dom, _, _, n) = setup();
    // Native Document31 +HTML div31 +HTML output34; no override allocation.
    assert_eq!(dom.usage().string_code_units(), 96);
    let mut b = DomBatch::new();
    b.set_output_default_value(n, "abc");
    b.set_custom_validity(n, "m\r\nn");
    apply(&mut dom, b);
    assert_eq!(dom.usage().string_code_units(), 102);
    let mut b = DomBatch::new();
    b.set_output_value(n, "xy");
    apply(&mut dom, b);
    // Old detached Text3 +override3 +new Text2 +message3 +base96.
    assert_eq!(dom.usage().string_code_units(), 107);
    let mut b = DomBatch::new();
    b.reset_output(n);
    apply(&mut dom, b);
    // Detached texts3+2, new text3, message3; override cleared only after insertion.
    assert_eq!(dom.usage().string_code_units(), 107);
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.detach(n);
    b.destroy(n);
    apply(&mut dom, b);
    assert_eq!(dom.usage().string_code_units(), 67);
    assert!(matches!(dom.snapshot().node(n), Err(DomError::StaleNode)));
    assert!(state(&old, n).default_value_override().is_none());
}

#[test]
fn source_reached_override_then_new_text_admission_refuses_atomically_one_short() {
    for cap in [103, 104] {
        let (mut dom, d, _, n) = setup();
        let mut b = DomBatch::new();
        b.set_output_default_value(n, "abc");
        apply(&mut dom, b);
        dom.set_limits(DomLimits::new(256, cap, 1024, 256, 256, 32, 1048576).unwrap())
            .unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_output_value(n, "xy");
        if cap == 103 {
            assert_eq!(
                dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 103
                }
            );
            unchanged(&dom, &before, usage);
            assert_eq!(state(&dom.snapshot(), n).default_value_override(), None);
        } else {
            apply(&mut dom, b);
            assert_eq!(dom.usage().string_code_units(), 104);
            assert_eq!(values(&dom.snapshot(), d, n), ("xy".into(), "abc".into()));
        }
    }
}

#[test]
fn node_capacity_precedes_old_child_removal_and_late_wrong_family_rolls_back() {
    let (mut dom, _, root, n) = setup();
    let mut b = DomBatch::new();
    b.set_output_default_value(n, "old");
    apply(&mut dom, b);
    dom.set_limits(DomLimits::new(4, 65536, 1024, 256, 256, 32, 1048576).unwrap())
        .unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.set_output_value(n, "new");
    assert_eq!(
        dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 4
        }
    );
    unchanged(&dom, &before, usage);
    let mut b = DomBatch::new();
    b.set_output_default_value(n, "");
    b.set_output_value(root, "wrong");
    assert_eq!(
        dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::OutputInapplicable
    );
    unchanged(&dom, &before, usage);
    let mut b = DomBatch::new();
    b.reset_output(n);
    assert_eq!(
        dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    unchanged(&dom, &before, usage);
}

#[test]
fn native_installation_has_null_override_and_exact_live_descendants() {
    let (mut dom, d, root, _) = setup();
    let mut b = surgeist_html::HtmlFragment::builder(surgeist_html::HtmlConstructionLimits::new(
        16, 16, 16, 1024, 16,
    ))
    .unwrap();
    let n = b
        .append_element(
            &b.root(),
            QualifiedName::new(Namespace::html(), None, "output"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let span = b
        .append_element(
            &n,
            QualifiedName::new(Namespace::html(), None, "span"),
            OrderedAttributes::default(),
        )
        .unwrap();
    b.append_text(&span, "A\r\n").unwrap();
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
    assert_eq!(state(result.snapshot(), n).default_value_override(), None);
    assert_eq!(
        values(result.snapshot(), d, n),
        ("A\r\n".into(), "A\r\n".into())
    );
    let mut b = DomBatch::new();
    b.set_output_value(n, "live");
    apply(&mut dom, b);
    assert_eq!(
        values(&dom.snapshot(), d, n),
        ("live".into(), "A\r\n".into())
    );
    assert_eq!(
        values(result.snapshot(), d, n),
        ("A\r\n".into(), "A\r\n".into())
    );
}

#[test]
fn first_id_form_lookup_and_owner_retirement_use_the_existing_canonical_relation() {
    let (mut dom, d, root, n) = setup();
    let mut b = DomBatch::new();
    let blocker = b.create_element_ns(d, Namespace::html(), "span");
    b.set_attribute(&blocker, "id", "f");
    b.insert_before(&blocker, root, None);
    let form = b.create_element_ns(d, Namespace::html(), "form");
    b.set_attribute(&form, "id", "f");
    b.insert_before(&form, root, None);
    b.insert_before(n, &form, None);
    b.set_attribute(n, "form", "f");
    let c = apply(&mut dom, b);
    let blocker = c.created(&blocker).unwrap().unwrap();
    let form = c.created(&form).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .node(n)
            .unwrap()
            .as_output()
            .unwrap()
            .form_owner(),
        None
    );
    let mut b = DomBatch::new();
    b.detach(blocker);
    apply(&mut dom, b);
    assert_eq!(
        dom.snapshot()
            .node(n)
            .unwrap()
            .as_output()
            .unwrap()
            .form_owner(),
        Some(form)
    );
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.detach(n);
    b.detach(form);
    b.destroy(form);
    apply(&mut dom, b);
    assert_eq!(
        dom.snapshot()
            .node(n)
            .unwrap()
            .as_output()
            .unwrap()
            .form_owner(),
        None
    );
    assert!(matches!(
        dom.snapshot().node(form),
        Err(DomError::StaleNode)
    ));
    assert_eq!(
        old.node(n).unwrap().as_output().unwrap().form_owner(),
        Some(form)
    );
}

#[test]
fn actual_override_units_have_their_own_capture_admission_and_query_work_is_finite() {
    let (mut dom, d, _, n) = setup();
    let mut b = DomBatch::new();
    b.set_output_value(n, "");
    b.set_output_default_value(n, "x".repeat(2048));
    apply(&mut dom, b);
    let s = dom.snapshot();
    // The ordinary captured tree has only short names and no Text. This limit
    // is reached by the actual retained override, not by an invented text node.
    assert!(matches!(
        s.html_capture(
            d,
            None,
            HtmlCaptureLimits::new(256, 1024, 256, 1024, 256, 0, 0),
            DomHtmlCaptureLimits::new(256, 1000000),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::OutputDefaults(
            HtmlOutputDefaultError::CodeUnits { limit: 1024, .. }
        ))
    ));
    let c = capture(&s, d);
    assert_eq!(
        read(&c, n, HtmlSemanticQuery::OutputDefaultValue).0,
        DocumentString::from("x".repeat(2048))
    );
    assert!(
        c.query(
            &[HtmlSemanticRequest::new(
                c.subject(n).unwrap(),
                HtmlSemanticQuery::OutputDefaultValue
            )],
            surgeist_html::HtmlValueInterpreter::new(65536),
            HtmlSemanticLimits::new(0, 32, 1024, 1024, 1000000, 1024),
            HtmlSemanticCancellation::new()
        )
        .is_err()
    );
    assert!(matches!(
        s.html_capture(
            d,
            None,
            HtmlCaptureLimits::new(256, 1024, 256, 65536, 256, 0, 0),
            DomHtmlCaptureLimits::new(256, 1000000),
            &AtomicBool::new(true)
        ),
        Err(DomHtmlCaptureError::Cancelled)
    ));
}

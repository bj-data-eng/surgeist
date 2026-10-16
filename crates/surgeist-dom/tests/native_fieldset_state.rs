//! Literal HTML877d Fieldset state, mutation and semantic composition oracles.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlNativeFormAssociationKind, HtmlValueInterpreter};
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlDirectionality, HtmlFormOwnerOrigin, HtmlInertFactLimits,
    HtmlLanguageDirectionLimits, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn owner() -> Dom {
    Dom::new(DomLimits::new(128, 16384, 256, 256, 128, 16, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId) {
    let mut b = DomBatch::new();
    let d = b.create_document();
    let f = b.create_element_ns(&d, Namespace::html(), "fieldset");
    let c = apply(dom, b);
    (c.document(&d).unwrap(), c.created(&f).unwrap().unwrap())
}
fn state(snapshot: &DomSnapshot, node: NodeId) -> &DomFieldsetState {
    let state = snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .fieldset_state()
        .unwrap();
    assert_eq!(state.kind(), HtmlNativeFormAssociationKind::Fieldset);
    assert_eq!(
        snapshot
            .node(node)
            .unwrap()
            .as_fieldset()
            .unwrap()
            .form_owner()
            .unwrap(),
        None
    );
    assert!(!state.parser_inserted());
    state
}
fn capture(snapshot: &DomSnapshot, d: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(
            d,
            None,
            HtmlCaptureLimits::new(256, 4096, 256, 32768, 256, 0, 0),
            DomHtmlCaptureLimits::new(256, 1000000),
            &cancel(),
        )
        .unwrap()
}
fn supplement(c: DomHtmlCapture) -> DomHtmlCapture {
    c.with_slot_semantics(
        DomSlotSemanticLimits::new(
            DomSlotQueryLimits::new(256, 512).unwrap(),
            DomHtmlCaptureLimits::new(256, 1000000),
            HtmlLanguageDirectionLimits::new(512, 512, 32768, 1000000),
            HtmlInertFactLimits::new(512, 1000000),
        ),
        &cancel(),
    )
    .unwrap()
}
fn fact(c: &DomHtmlCapture, n: NodeId, q: HtmlSemanticQuery) -> HtmlSemanticFact {
    c.query(
        &[HtmlSemanticRequest::new(c.subject(n).unwrap(), q)],
        HtmlValueInterpreter::new(32768),
        HtmlSemanticLimits::new(1000000, 16, 1024, 512, 1000000, 1024),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .results()[0]
        .fact()
        .clone()
}
fn null_owner(c: &DomHtmlCapture, n: NodeId) {
    let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) =
        fact(c, n, HtmlSemanticQuery::FormOwner)
    else {
        panic!("actual stored association required")
    };
    assert_eq!(owner.owner(), None);
    assert_eq!(owner.origin(), HtmlFormOwnerOrigin::EstablishedCurrent);
}
#[test]
fn exact_fieldset_names_retain_initial_state_without_admitting_other_native_controls() {
    let mut dom = owner();
    let (d, f) = setup(&mut dom);
    assert_eq!(
        state(&dom.snapshot(), f).custom_validity_message(),
        &DocumentString::from("")
    );
    assert_eq!(dom.usage().string_code_units(), 67); // XML metadata31 + namespace28 + local8.
    let mut b = DomBatch::new();
    let prefixed = b.create_element_ns(d, Namespace::html(), "p:fieldset");
    let ordinary = [
        (Namespace::none(), "fieldset"),
        (Namespace::html(), "Fieldset"),
        (Namespace::svg(), "fieldset"),
    ];
    let tickets: Vec<_> = ordinary
        .into_iter()
        .map(|(ns, name)| b.create_element_ns(d, ns, name))
        .collect();
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    state(&s, c.created(&prefixed).unwrap().unwrap());
    for t in tickets {
        assert!(
            s.node(c.created(&t).unwrap().unwrap())
                .unwrap()
                .as_element()
                .unwrap()
                .fieldset_state()
                .is_none()
        );
    }
    for name in ["object", "audio"] {
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        b.create_element_ns(d, Namespace::html(), name);
        assert!(matches!(
            dom.apply(b, &cancel()),
            Err(DomError::NativeInitializationRequired(_))
        ));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
}
#[test]
fn custom_message_normalizes_exact_units_and_reports_only_stored_state() {
    let mut dom = owner();
    let (_, f) = setup(&mut dom);
    let old = dom.snapshot();
    let raw = DocumentString::from_utf16(vec![65, 13, 10, 0xd800, 13, 13, 10, 0, 0xdc00]);
    let expected = DocumentString::from_utf16(vec![65, 10, 0xd800, 10, 10, 0, 0xdc00]);
    let mut b = DomBatch::new();
    b.set_custom_validity(f, raw);
    let c = apply(&mut dom, b);
    assert_eq!(c.changes().changes(), &[DomChange::ElementState(f)]);
    assert_eq!(
        state(&dom.snapshot(), f).custom_validity_message(),
        &expected
    );
    assert_eq!(
        state(&old, f).custom_validity_message(),
        &DocumentString::from("")
    );
    assert_eq!(dom.usage().string_code_units(), 74);
    assert_eq!(
        dom.changes_since(old.stamp()).unwrap().changes(),
        &[DomChange::ElementState(f)]
    );
    let stamp = dom.snapshot().stamp();
    let mut b = DomBatch::new();
    b.set_custom_validity(f, expected);
    assert!(apply(&mut dom, b).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
    let mut b = DomBatch::new();
    b.set_custom_validity(f, "");
    apply(&mut dom, b);
    assert_eq!(dom.usage().string_code_units(), 67);
}
#[test]
fn custom_message_limits_inapplicability_cancellation_and_late_failure_are_atomic() {
    let mut dom = owner();
    let (d, f) = setup(&mut dom);
    dom.set_limits(DomLimits::new(128, 69, 256, 256, 128, 16, 65536).unwrap())
        .unwrap();
    let mut b = DomBatch::new();
    b.set_custom_validity(f, "\r\n\r\n");
    apply(&mut dom, b); // 2 retained LF, not 4 raw units.
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.set_custom_validity(f, "abc");
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: 69
        }
    );
    let mut b = DomBatch::new();
    b.set_custom_validity(f, "x");
    b.insert_before(d, f, None);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::HierarchyRequest
    );
    let mut b = DomBatch::new();
    b.set_custom_validity(f, "");
    assert_eq!(
        dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    let mut b = DomBatch::new();
    b.set_custom_validity(d, "");
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::CustomValidityInapplicable
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        state(&dom.snapshot(), f)
            .custom_validity_message()
            .code_units(),
        &[10, 10]
    );
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}
#[test]
fn actual_form_attribute_and_id_events_keep_completed_negative_associations() {
    let mut dom = owner();
    let (d, f) = setup(&mut dom);
    let mut b = DomBatch::new();
    let top = b.create_element_ns(d, Namespace::html(), "div");
    let collision = b.create_element(d, "form");
    let uppercase = b.create_element_ns(d, Namespace::html(), "Form");
    b.set_attribute(&collision, "id", "chosen");
    b.set_attribute(&uppercase, "id", "chosen");
    b.insert_before(&top, d, None);
    b.insert_before(&collision, &top, None);
    b.insert_before(&uppercase, &top, None);
    b.insert_before(f, &top, None);
    let c = apply(&mut dom, b);
    let collision = c.created(&collision).unwrap().unwrap();
    for value in ["", "chosen", "missing"] {
        let mut b = DomBatch::new();
        b.set_attribute(f, "form", value);
        b.set_attribute(f, "form", value);
        apply(&mut dom, b);
        null_owner(&capture(&dom.snapshot(), d), f);
    }
    let mut b = DomBatch::new();
    b.set_attribute_ns(f, Namespace::from_uri("urn:foreign"), "p:form", "ignored");
    b.set_attribute(f, "FORM", "raw uppercase in XML document");
    b.set_attribute(collision, "id", "new");
    b.remove_attribute(f, "form");
    b.detach(collision);
    b.detach(f);
    apply(&mut dom, b);
    null_owner(&capture(&dom.snapshot(), d), f);
    state(&dom.snapshot(), f);
}
#[test]
fn clone_import_is_fresh_while_adoption_preserves_message_and_old_captures() {
    let mut dom = owner();
    let (a, f) = setup(&mut dom);
    let mut b = DomBatch::new();
    let d = b.create_document();
    b.set_custom_validity(f, "retained\rmessage");
    b.set_attribute(f, "form", "chosen");
    b.insert_before(f, a, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let old = dom.snapshot();
    let old_capture = capture(&old, a);
    let mut b = DomBatch::new();
    let copy = b.import_node(&old, f, d, true);
    b.adopt(f, d);
    let c = apply(&mut dom, b);
    let copy = c.created(&copy).unwrap().unwrap();
    let current = dom.snapshot();
    assert_eq!(
        state(&current, copy).custom_validity_message(),
        &DocumentString::from("")
    );
    assert_eq!(
        state(&current, f).custom_validity_message(),
        &DocumentString::from("retained\nmessage")
    );
    assert_eq!(current.node(f).unwrap().owner_document(), Some(d));
    assert_eq!(old.node(f).unwrap().owner_document(), Some(a));
    null_owner(&old_capture, f);
    null_owner(&capture(&current, d), f);
    null_owner(&capture(&current, d), copy);
    let mut b = DomBatch::new();
    b.destroy(f);
    apply(&mut dom, b);
    assert!(matches!(dom.snapshot().node(f), Err(DomError::StaleNode)));
    assert_eq!(
        state(&old, f).custom_validity_message(),
        &DocumentString::from("retained\nmessage")
    );
}
#[test]
fn fieldset_semantics_survive_slot_supplement_with_literal_tree_queries() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let top = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&top, &d, None);
    let outer = b.create_element_ns(&d, Namespace::html(), "fieldset");
    b.set_attribute(&outer, "disabled", "");
    b.insert_before(&outer, &top, None);
    let first = b.create_element_ns(&d, Namespace::html(), "legend");
    let second = b.create_element_ns(&d, Namespace::html(), "legend");
    b.insert_before(&first, &outer, None);
    b.insert_before(&second, &outer, None);
    let inside_first = b.create_element_ns(&d, Namespace::html(), "fieldset");
    let inside_second = b.create_element_ns(&d, Namespace::html(), "fieldset");
    let direct = b.create_element_ns(&d, Namespace::html(), "fieldset");
    b.insert_before(&inside_first, &first, None);
    b.insert_before(&inside_second, &second, None);
    b.insert_before(&direct, &outer, None);
    b.set_custom_validity(&inside_first, "custom error");
    let host = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&host, &top, None);
    let shadow = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let slot = b.create_element_ns(&d, Namespace::html(), "slot");
    b.set_attribute(&slot, "dir", "auto");
    b.set_attribute(&slot, "inert", "");
    let text = b.create_text(&d, "\u{05d0}");
    b.insert_before(&slot, &shadow, None);
    b.insert_before(&text, &slot, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let id = |t: &CreatedNode| c.created(t).unwrap().unwrap();
    let (outer, first, second, one, two, direct, slot) = (
        id(&outer),
        id(&first),
        id(&second),
        id(&inside_first),
        id(&inside_second),
        id(&direct),
        id(&slot),
    );
    let old = dom.snapshot();
    for c in [capture(&old, d), supplement(capture(&old, d))] {
        null_owner(&c, one);
        assert_eq!(
            fact(&c, first, HtmlSemanticQuery::LegendFormOwner),
            HtmlSemanticFact::LegendFormOwner(HtmlSemanticValue::Known(None))
        );
        let HtmlSemanticFact::FieldsetControls(HtmlSemanticValue::Known(list)) =
            fact(&c, outer, HtmlSemanticQuery::FieldsetControls)
        else {
            panic!("actual collection")
        };
        let actual: Vec<_> = list.controls().iter().map(|n| c.node(n).unwrap()).collect();
        assert_eq!(actual, vec![one, two, direct]);
        for (node, disabled) in [(outer, true), (one, false), (two, true), (direct, true)] {
            assert_eq!(
                fact(&c, node, HtmlSemanticQuery::Disabledness),
                HtmlSemanticFact::Disabledness(HtmlSemanticValue::Known(disabled))
            );
        }
        assert_eq!(
            fact(&c, one, HtmlSemanticQuery::ConstraintValidationCandidate),
            HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(false))
        );
    }
    let c = supplement(capture(&old, d));
    assert_eq!(
        fact(&c, slot, HtmlSemanticQuery::Directionality),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
    );
    assert_eq!(
        fact(&c, slot, HtmlSemanticQuery::Inertness),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    let mut b = DomBatch::new();
    b.insert_before(second, outer, Some(first.into()));
    apply(&mut dom, b);
    let c = supplement(capture(&dom.snapshot(), d));
    assert_eq!(
        fact(&c, one, HtmlSemanticQuery::Disabledness),
        HtmlSemanticFact::Disabledness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        fact(&c, two, HtmlSemanticQuery::Disabledness),
        HtmlSemanticFact::Disabledness(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        fact(
            &supplement(capture(&old, d)),
            one,
            HtmlSemanticQuery::Disabledness
        ),
        HtmlSemanticFact::Disabledness(HtmlSemanticValue::Known(false))
    );
}

#[test]
fn fragment_replacement_shadow_adoption_and_frame_hooks_share_one_atomic_owner() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let a = b.create_document();
    let d = b.create_document();
    let top = b.create_element_ns(&a, Namespace::html(), "div");
    b.insert_before(&top, &a, None);
    let shadow = b.attach_shadow(
        &top,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Closed,
            HtmlSlotAssignment::Named,
            false,
            true,
            false,
        ),
    );
    let f = b.create_element_ns(&a, Namespace::html(), "fieldset");
    let frame = b.create_element_ns(&a, Namespace::html(), "frame");
    b.set_custom_validity(&f, "shadow\rerror");
    b.set_attribute(&f, "form", "named");
    b.insert_before(&f, &shadow, None);
    b.insert_before(&frame, &f, None);
    let fragment = b.create_fragment(&a);
    let first = b.create_element_ns(&a, Namespace::html(), "fieldset");
    let second = b.create_element_ns(&a, Namespace::html(), "fieldset");
    b.set_custom_validity(&first, "one");
    b.set_custom_validity(&second, "two");
    b.insert_before(&first, &fragment, None);
    b.insert_before(&second, &fragment, None);
    let old = b.create_comment(&a, "replace");
    b.insert_before(&old, &top, None);
    b.replace_child(&fragment, &top, &old);
    let c = apply(&mut dom, b);
    let id = |t: &CreatedNode| c.created(t).unwrap().unwrap();
    let (a, d, top, shadow, f, frame, fragment, first, second) = (
        c.document(&a).unwrap(),
        c.document(&d).unwrap(),
        id(&top),
        id(&shadow),
        id(&f),
        id(&frame),
        id(&fragment),
        id(&first),
        id(&second),
    );
    let old = dom.snapshot();
    assert!(old.node(fragment).unwrap().children().is_empty());
    assert_eq!(old.node(top).unwrap().children(), &[first, second]);
    let mut b = DomBatch::new();
    b.adopt(top, d);
    b.insert_before(top, d, None);
    b.set_attribute(frame, "src", "still-null");
    let clone = b.import_node(&old, top, a, true);
    let c = apply(&mut dom, b);
    let clone = c.created(&clone).unwrap().unwrap();
    let s = dom.snapshot();
    for n in [top, shadow, f, frame, first, second] {
        assert_eq!(s.node(n).unwrap().owner_document(), Some(d));
    }
    assert_eq!(
        state(&s, f).custom_validity_message(),
        &DocumentString::from("shadow\nerror")
    );
    assert!(
        s.node(frame)
            .unwrap()
            .as_element()
            .unwrap()
            .frame_null_content()
            .unwrap()
            .content_navigable_is_null()
    );
    let copied_shadow = s.node(clone).unwrap().hosted_shadow_root().unwrap();
    let copied_f = s.node(copied_shadow).unwrap().children()[0];
    assert_eq!(
        state(&s, copied_f).custom_validity_message(),
        &DocumentString::from("")
    );
    null_owner(&supplement(capture(&s, d)), f);
    null_owner(&supplement(capture(&old, a)), f);
    let mut b = DomBatch::new();
    b.detach(top);
    b.destroy(top);
    apply(&mut dom, b);
    for n in [top, shadow, f, frame, first, second] {
        assert!(matches!(dom.snapshot().node(n), Err(DomError::StaleNode)));
    }
    state(&old, f);
}
#[test]
fn html_create_element_casing_and_semantic_failure_keep_actual_snapshot_state() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let xml = b.create_document();
    let html = b.create_html_document(&xml, None);
    let c = apply(&mut dom, b);
    let html = c.document(&html).unwrap();
    let mut b = DomBatch::new();
    let f = b.create_element(html, "FIELDSET");
    b.set_attribute(&f, "FORM", "chosen");
    let c = apply(&mut dom, b);
    let f = c.created(&f).unwrap().unwrap();
    let s = dom.snapshot();
    state(&s, f);
    assert_eq!(
        s.node(f).unwrap().node_name(),
        DocumentString::from("FIELDSET")
    );
    assert_eq!(
        s.node(f)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&DocumentString::from("form")),
        Some(&DocumentString::from("chosen"))
    );
    null_owner(&capture(&s, html), f);
    let result = capture(&s, html).with_slot_semantics(
        DomSlotSemanticLimits::new(
            DomSlotQueryLimits::new(128, 256).unwrap(),
            DomHtmlCaptureLimits::new(128, 0),
            HtmlLanguageDirectionLimits::new(128, 128, 16384, 10000),
            HtmlInertFactLimits::new(128, 10000),
        ),
        &cancel(),
    );
    assert!(matches!(
        result,
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            limit: 0
        })
    ));
    assert_eq!(dom.snapshot().stamp(), s.stamp());
    null_owner(&supplement(capture(&s, html)), f);
    let result = capture(&s, html).with_slot_semantics(
        DomSlotSemanticLimits::new(
            DomSlotQueryLimits::new(128, 256).unwrap(),
            DomHtmlCaptureLimits::new(128, 1000000),
            HtmlLanguageDirectionLimits::new(128, 128, 16384, 1000000),
            HtmlInertFactLimits::new(128, 1000000),
        ),
        &AtomicBool::new(true),
    );
    assert!(matches!(result, Err(DomHtmlCaptureError::Cancelled)));
    assert_eq!(dom.snapshot().stamp(), s.stamp());
}

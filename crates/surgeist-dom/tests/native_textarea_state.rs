//! Literal DOM3071/HTML877d Textarea outcomes through the canonical owner.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{
    HtmlInputEmptySelectionSupport as Empty, HtmlInputNoneDirectionSupport as NoneDirection,
    HtmlInputSelectionDirection as Direction,
};

fn limits() -> DomLimits {
    DomLimits::new(
        256,
        65536,
        1024,
        256,
        256,
        32,
        1048576,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn owner() -> Dom {
    Dom::new(limits()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId, NodeId) {
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    let textarea = b.create_element_ns(&d, Namespace::html(), "textarea");
    b.insert_before(&root, &d, None);
    b.insert_before(&textarea, &root, None);
    let c = apply(dom, b);
    (
        c.document(&d).unwrap(),
        c.created(&root).unwrap().unwrap(),
        c.created(&textarea).unwrap().unwrap(),
    )
}
fn state(s: &DomSnapshot, n: NodeId) -> &DomTextareaState {
    s.node(n).unwrap().as_textarea().unwrap().state()
}
fn value(s: &DomSnapshot, n: NodeId) -> DocumentString {
    s.node(n)
        .unwrap()
        .as_textarea()
        .unwrap()
        .value(
            DomTextareaReadLimits::new(65536, 1000000),
            &AtomicBool::new(false),
        )
        .unwrap()
}
fn default(s: &DomSnapshot, n: NodeId) -> DocumentString {
    s.node(n)
        .unwrap()
        .as_textarea()
        .unwrap()
        .default_value(
            DomTextareaReadLimits::new(65536, 1000000),
            &AtomicBool::new(false),
        )
        .unwrap()
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
fn fresh_textarea_and_template_contents_use_both_independent_platform_facts() {
    for empty in [Empty::Supported, Empty::Unsupported] {
        for none in [NoneDirection::Supported, NoneDirection::Unsupported] {
            let platform = DomNativeInputPlatform::new(empty, none);
            let mut dom = Dom::with_native_input_platform(limits(), platform).unwrap();
            let (d, root, textarea) = setup(&mut dom);
            let s = dom.snapshot();
            let actual = state(&s, textarea);
            assert!(!actual.flags().dirty_value());
            assert!(!actual.flags().user_validity());
            assert!(!actual.parser_inserted());
            assert!(actual.custom_validity_message().is_empty());
            assert!(actual.raw_value().is_empty());
            assert_eq!(value(&s, textarea), "".into());
            assert_eq!(actual.selection().cursor_position(), Some(0));
            assert_eq!(
                actual.selection().direction(),
                if none == NoneDirection::Supported {
                    Direction::None
                } else {
                    Direction::Forward
                }
            );
            assert_eq!(
                s.node(textarea)
                    .unwrap()
                    .as_textarea()
                    .unwrap()
                    .form_owner()
                    .unwrap(),
                None
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
                .contents()
                .unwrap();
            let inert = s.node(contents).unwrap().owner_document().unwrap();
            assert_eq!(
                s.document(inert)
                    .unwrap()
                    .metadata()
                    .document_environment()
                    .input_platform(),
                platform
            );
            let mut b = DomBatch::new();
            let nested = b.create_element_ns(inert, Namespace::html(), "textarea");
            b.insert_before(&nested, contents, None);
            let c = apply(&mut dom, b);
            let s = dom.snapshot();
            assert_eq!(
                state(&s, c.created(&nested).unwrap().unwrap())
                    .selection()
                    .direction(),
                actual.selection().direction()
            );
        }
    }
}

#[test]
fn direct_text_raw_and_normalized_api_remain_distinct_from_nested_and_dirty_defaults() {
    let mut dom = owner();
    let (d, _, textarea) = setup(&mut dom);
    let mut b = DomBatch::new();
    let first = b.create_text(d, "A\r");
    let last = b.create_text(
        d,
        DocumentString::from_utf16(vec![10, 66, 0, 0xd800, 13, 0xdc00]),
    );
    let nested = b.create_element_ns(d, Namespace::html(), "span");
    let ignored = b.create_text(d, "ignored");
    let comment = b.create_comment(d, "ignored too");
    b.insert_before(&first, textarea, None);
    b.insert_before(&nested, textarea, None);
    b.insert_before(&ignored, &nested, None);
    b.insert_before(&comment, textarea, None);
    b.insert_before(&last, textarea, None);
    let c = apply(&mut dom, b);
    let first = c.created(&first).unwrap().unwrap();
    let nested = c.created(&nested).unwrap().unwrap();
    let old = dom.snapshot();
    let raw = DocumentString::from_utf16(vec![65, 13, 10, 66, 0, 0xd800, 13, 0xdc00]);
    assert_eq!(state(&old, textarea).raw_value(), &raw);
    assert_eq!(default(&old, textarea), raw);
    assert_eq!(
        value(&old, textarea).code_units(),
        &[65, 10, 66, 0, 0xd800, 10, 0xdc00]
    );
    let mut b = DomBatch::new();
    b.set_textarea_value(textarea, "current\r\n");
    b.set_custom_validity(textarea, "m\r\nn");
    b.replace_data(first, 0, usize::MAX, "new\r");
    b.detach(nested);
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(value(&s, textarea), "current\n".into());
    assert!(state(&s, textarea).flags().dirty_value());
    assert_eq!(
        default(&s, textarea).code_units(),
        &[110, 101, 119, 13, 10, 66, 0, 0xd800, 13, 0xdc00]
    );
    let mut b = DomBatch::new();
    b.reset_textarea(textarea);
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert!(!state(&s, textarea).flags().dirty_value());
    assert_eq!(
        value(&s, textarea).code_units(),
        &[110, 101, 119, 10, 66, 0, 0xd800, 10, 0xdc00]
    );
    assert_eq!(
        state(&s, textarea).custom_validity_message(),
        &DocumentString::from("m\nn")
    );
    assert_eq!(
        value(&old, textarea).code_units(),
        &[65, 10, 66, 0, 0xd800, 10, 0xdc00]
    );
}

#[test]
fn current_setter_preserves_exact_raw_and_compares_normalized_api_values() {
    let mut dom = owner();
    let (_, _, textarea) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_attribute(textarea, "value", "attribute only");
    apply(&mut dom, b);
    assert!(value(&dom.snapshot(), textarea).is_empty());
    assert!(default(&dom.snapshot(), textarea).is_empty());
    let mut b = DomBatch::new();
    b.set_textarea_value(textarea, "A\r\nB");
    apply(&mut dom, b);
    let old = dom.snapshot();
    assert_eq!(state(&old, textarea).selection().cursor_position(), Some(3));
    let mut b = DomBatch::new();
    b.set_textarea_value(textarea, "A\nB");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(
        state(&s, textarea).raw_value(),
        &DocumentString::from("A\nB")
    );
    assert_eq!(value(&s, textarea), "A\nB".into());
    assert_eq!(
        state(&s, textarea).selection(),
        state(&old, textarea).selection()
    );
    let mut b = DomBatch::new();
    b.set_textarea_value(textarea, "xy");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(state(&s, textarea).selection().cursor_position(), Some(2));
    assert!(state(&s, textarea).flags().dirty_value());
    assert_eq!(
        state(&old, textarea).raw_value(),
        &DocumentString::from("A\r\nB")
    );
}

#[test]
fn moving_the_only_text_observes_the_empty_intermediate_even_when_final_children_match() {
    let mut dom = owner();
    let (d, _, textarea) = setup(&mut dom);
    let mut b = DomBatch::new();
    let text = b.create_text(d, "abc");
    b.insert_before(&text, textarea, None);
    b.set_textarea_value(textarea, "abc!");
    b.reset_textarea(textarea);
    let c = apply(&mut dom, b);
    let text = c.created(&text).unwrap().unwrap();
    let old = dom.snapshot();
    assert_eq!(state(&old, textarea).selection().cursor_position(), Some(3));
    let mut b = DomBatch::new();
    b.insert_before(text, textarea, None);
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(s.node(textarea).unwrap().children(), &[text]);
    assert_eq!(value(&s, textarea), "abc".into());
    assert_eq!(state(&s, textarea).selection().cursor_position(), Some(0));
    assert_ne!(s.stamp(), old.stamp());
    assert_eq!(state(&old, textarea).selection().cursor_position(), Some(3));
}

#[test]
fn default_setter_replaces_real_children_and_keeps_dirty_current_value() {
    let mut dom = owner();
    let (d, _, textarea) = setup(&mut dom);
    let mut b = DomBatch::new();
    let old_text = b.create_text(d, "old");
    b.insert_before(&old_text, textarea, None);
    let c = apply(&mut dom, b);
    let old_text = c.created(&old_text).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.set_textarea_default_value(textarea, "new\r\n");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(default(&s, textarea), "new\r\n".into());
    assert_eq!(value(&s, textarea), "new\n".into());
    assert_eq!(s.node(old_text).unwrap().parent(), None);
    let new_text = s.node(textarea).unwrap().children()[0];
    assert_ne!(new_text, old_text);
    let mut b = DomBatch::new();
    b.set_textarea_value(textarea, "dirty");
    b.set_textarea_default_value(textarea, "");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert!(s.node(textarea).unwrap().children().is_empty());
    assert_eq!(value(&s, textarea), "dirty".into());
    assert!(default(&s, textarea).is_empty());
    assert_eq!(s.node(new_text).unwrap().parent(), None);
    let mut b = DomBatch::new();
    b.reset_textarea(textarea);
    apply(&mut dom, b);
    assert!(value(&dom.snapshot(), textarea).is_empty());
}

#[test]
fn fragment_drain_and_character_data_hooks_use_completed_actual_child_order() {
    let mut dom = owner();
    let (d, _, textarea) = setup(&mut dom);
    let mut b = DomBatch::new();
    let fragment = b.create_fragment(d);
    let a = b.create_text(d, "X\r");
    let z = b.create_text(d, "\nY");
    b.insert_before(&a, &fragment, None);
    b.insert_before(&z, &fragment, None);
    b.insert_before(&fragment, textarea, None);
    let c = apply(&mut dom, b);
    let fragment = c.created(&fragment).unwrap().unwrap();
    let a = c.created(&a).unwrap().unwrap();
    let z = c.created(&z).unwrap().unwrap();
    let s = dom.snapshot();
    assert!(s.node(fragment).unwrap().children().is_empty());
    assert_eq!(s.node(textarea).unwrap().children(), &[a, z]);
    assert_eq!(
        state(&s, textarea).raw_value(),
        &DocumentString::from("X\r\nY")
    );
    assert_eq!(value(&s, textarea), "X\nY".into());
    let mut b = DomBatch::new();
    b.replace_data(a, 1, 1, "\r\r");
    b.detach(z);
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), textarea), "X\n\n".into());
}

#[test]
fn shallow_and_deep_clones_copy_only_raw_and_dirty_while_adoption_preserves_state() {
    let mut source = owner();
    let (_, _, textarea) = setup(&mut source);
    let mut b = DomBatch::new();
    b.set_textarea_default_value(textarea, "default");
    apply(&mut source, b);
    let clean = source.snapshot();
    let mut b = DomBatch::new();
    b.set_textarea_value(textarea, "dirty\r\n");
    b.set_custom_validity(textarea, "error");
    apply(&mut source, b);
    let dirty = source.snapshot();
    let mut target = Dom::with_native_input_platform(
        limits(),
        DomNativeInputPlatform::new(Empty::Unsupported, NoneDirection::Unsupported),
    )
    .unwrap();
    let (d, _, _) = setup(&mut target);
    let mut b = DomBatch::new();
    let shallow_clean = b.import_node(&clean, textarea, d, false);
    let deep_clean = b.import_node(&clean, textarea, d, true);
    let shallow_dirty = b.import_node(&dirty, textarea, d, false);
    let deep_dirty = b.import_node(&dirty, textarea, d, true);
    let c = apply(&mut target, b);
    let sc = c.created(&shallow_clean).unwrap().unwrap();
    let dc = c.created(&deep_clean).unwrap().unwrap();
    let sd = c.created(&shallow_dirty).unwrap().unwrap();
    let dd = c.created(&deep_dirty).unwrap().unwrap();
    let s = target.snapshot();
    for n in [sc, dc] {
        assert_eq!(value(&s, n), "default".into());
        assert!(!state(&s, n).flags().dirty_value());
    }
    for n in [sd, dd] {
        assert_eq!(state(&s, n).raw_value(), &DocumentString::from("dirty\r\n"));
        assert!(state(&s, n).flags().dirty_value());
        assert!(!state(&s, n).flags().user_validity());
    }
    for n in [sc, dc, sd, dd] {
        assert!(state(&s, n).custom_validity_message().is_empty());
        assert!(!state(&s, n).parser_inserted());
        assert_eq!(state(&s, n).selection().cursor_position(), Some(0));
        assert_eq!(state(&s, n).selection().direction(), Direction::Forward);
        assert_eq!(
            s.node(n)
                .unwrap()
                .as_textarea()
                .unwrap()
                .form_owner()
                .unwrap(),
            None
        );
    }
    assert!(s.node(sc).unwrap().children().is_empty());
    assert_eq!(default(&s, sc), "".into());
    assert_eq!(default(&s, dc), "default".into());
    assert_eq!(default(&s, dd), "default".into());
    // String-replace-all(null) on an already empty child list runs no remove or
    // insert children hook. It must not globally repair a clean shallow clone.
    let mut b = DomBatch::new();
    b.set_textarea_default_value(sc, "");
    let newdoc = b.create_document();
    b.adopt(sd, &newdoc);
    apply(&mut target, b);
    let after = target.snapshot();
    assert_eq!(value(&after, sc), "default".into());
    assert_eq!(state(&after, sd), state(&s, sd));
    assert_ne!(
        after.node(sd).unwrap().owner_document(),
        s.node(sd).unwrap().owner_document()
    );
    assert_eq!(
        state(&dirty, textarea).custom_validity_message(),
        &DocumentString::from("error")
    );
}

#[test]
fn form_owner_first_id_and_weak_retirement_remain_canonical() {
    let mut dom = owner();
    let (d, root, textarea) = setup(&mut dom);
    let mut b = DomBatch::new();
    let form = b.create_element_ns(d, Namespace::html(), "form");
    b.set_attribute(&form, "id", "f");
    b.insert_before(&form, root, None);
    b.insert_before(textarea, &form, None);
    let c = apply(&mut dom, b);
    let form = c.created(&form).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .node(textarea)
            .unwrap()
            .as_textarea()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(form)
    );
    let mut b = DomBatch::new();
    let blocker = b.create_element_ns(d, Namespace::html(), "div");
    b.set_attribute(&blocker, "id", "f");
    b.insert_before(&blocker, root, Some(form.into()));
    b.set_attribute(textarea, "form", "f");
    let c = apply(&mut dom, b);
    let blocker = c.created(&blocker).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .node(textarea)
            .unwrap()
            .as_textarea()
            .unwrap()
            .form_owner()
            .unwrap(),
        None
    );
    let mut b = DomBatch::new();
    b.detach(blocker);
    apply(&mut dom, b);
    assert_eq!(
        dom.snapshot()
            .node(textarea)
            .unwrap()
            .as_textarea()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(form)
    );
    let mut b = DomBatch::new();
    b.detach(textarea);
    b.detach(form);
    b.destroy(form);
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(
        s.node(textarea)
            .unwrap()
            .as_textarea()
            .unwrap()
            .form_owner()
            .unwrap(),
        None
    );
    assert!(matches!(s.node(form), Err(DomError::StaleNode)));
}

fn capture(s: &DomSnapshot, d: DocumentId) -> DomHtmlCapture {
    s.html_capture(
        d,
        None,
        surgeist_html_semantics::HtmlCaptureLimits::new(
            256, 1024, 256, 65536, 256, 0, 0, 16_777_216,
        ),
        DomHtmlCaptureLimits::new(256, 1000000),
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn fact(
    c: &DomHtmlCapture,
    n: NodeId,
    q: surgeist_html_semantics::HtmlSemanticQuery,
) -> surgeist_html_semantics::HtmlSemanticFact {
    use surgeist_html_semantics::*;
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
#[test]
fn semantic_current_flags_values_and_owner_survive_slot_supplement_and_old_capture() {
    use surgeist_html_semantics::*;
    let mut dom = owner();
    let (d, root, textarea) = setup(&mut dom);
    let mut b = DomBatch::new();
    let form = b.create_element_ns(d, Namespace::html(), "form");
    let slot = b.create_element_ns(d, Namespace::html(), "slot");
    b.insert_before(&form, root, None);
    b.insert_before(textarea, &form, None);
    b.insert_before(&slot, root, None);
    b.set_textarea_default_value(textarea, "LTR default");
    b.set_attribute(textarea, "dir", "auto");
    b.set_attribute(textarea, "required", "");
    b.set_textarea_value(textarea, "\r\nאב");
    b.assign_slot(&slot, vec![textarea.into()]);
    let c = apply(&mut dom, b);
    let form = c.created(&form).unwrap().unwrap();
    let s = dom.snapshot();
    let old = capture(&s, d);
    for supplemented in [false, true] {
        let c = if supplemented {
            capture(&s, d)
                .with_slot_semantics(
                    DomSlotSemanticLimits::new(
                        DomSlotQueryLimits::new(256, 1024).unwrap(),
                        DomHtmlCaptureLimits::new(256, 1000000),
                        HtmlLanguageDirectionLimits::new(512, 1024, 65536, 1000000),
                        HtmlInertFactLimits::new(512, 1000000),
                    ),
                    &AtomicBool::new(false),
                )
                .unwrap()
        } else {
            capture(&s, d)
        };
        assert_eq!(
            fact(&c, textarea, HtmlSemanticQuery::DirtyValueFlag),
            HtmlSemanticFact::DirtyValueFlag(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            fact(&c, textarea, HtmlSemanticQuery::Directionality),
            HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
        );
        assert_eq!(
            fact(&c, textarea, HtmlSemanticQuery::TextareaValueMissing),
            HtmlSemanticFact::TextareaValueMissing(HtmlSemanticValue::Known(false))
        );
        let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) =
            fact(&c, textarea, HtmlSemanticQuery::FormOwner)
        else {
            panic!("actual owner")
        };
        assert_eq!(owner.owner(), Some(&c.subject(form).unwrap()));
    }
    let mut b = DomBatch::new();
    b.set_textarea_value(textarea, "");
    apply(&mut dom, b);
    assert_eq!(
        fact(
            &capture(&dom.snapshot(), d),
            textarea,
            HtmlSemanticQuery::TextareaValueMissing
        ),
        HtmlSemanticFact::TextareaValueMissing(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        fact(&old, textarea, HtmlSemanticQuery::Directionality),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
    );
}

#[test]
fn native_fragment_installation_creates_real_clean_raw_children_and_later_live_edits() {
    let mut dom = owner();
    let (d, root, _) = setup(&mut dom);
    let mut builder = surgeist_html::HtmlFragment::builder(
        surgeist_html::HtmlConstructionLimits::new(16, 16, 16, 1024, 16),
    )
    .unwrap();
    let textarea = builder
        .append_element(
            &builder.root(),
            QualifiedName::new(Namespace::html(), None, "textarea"),
            OrderedAttributes::default(),
        )
        .unwrap();
    builder.append_text(&textarea, "A\r").unwrap();
    builder.append_text(&textarea, "\nB").unwrap();
    let source = builder.finish();
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
    let textarea = result.node_for(&subject).unwrap();
    assert_eq!(value(result.snapshot(), textarea), "A\nB".into());
    assert_eq!(
        state(result.snapshot(), textarea).raw_value(),
        &DocumentString::from("A\r\nB")
    );
    assert!(!state(result.snapshot(), textarea).flags().dirty_value());
    let mut b = DomBatch::new();
    b.set_textarea_value(textarea, "new");
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), textarea), "new".into());
    assert_eq!(value(result.snapshot(), textarea), "A\nB".into());
}

#[test]
fn immutable_reads_have_exact_unit_work_and_cancellation_boundaries() {
    let mut dom = owner();
    let (_, _, textarea) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_textarea_default_value(textarea, "A\r\nB");
    apply(&mut dom, b);
    let s = dom.snapshot();
    let v = s.node(textarea).unwrap().as_textarea().unwrap();
    let cancel = AtomicBool::new(false);
    // Four count reads + reservation + four materialization reads + three
    // output writes =12 actual documented read credits, independent of values.
    assert_eq!(
        v.value(DomTextareaReadLimits::new(3, 12), &cancel).unwrap(),
        "A\nB".into()
    );
    assert_eq!(
        v.value(DomTextareaReadLimits::new(3, 11), &cancel),
        Err(DomError::TextareaReadWorkLimit { limit: 11 })
    );
    assert_eq!(
        v.value(DomTextareaReadLimits::new(2, 12), &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: 2
        })
    );
    assert_eq!(
        v.text_length(DomTextareaReadLimits::new(3, 4), &cancel)
            .unwrap(),
        3
    );
    assert_eq!(
        v.text_length(DomTextareaReadLimits::new(3, 3), &cancel),
        Err(DomError::TextareaReadWorkLimit { limit: 3 })
    );
    // One child visited twice + reservation + four exact raw output units.
    assert_eq!(
        v.default_value(DomTextareaReadLimits::new(4, 7), &cancel)
            .unwrap(),
        "A\r\nB".into()
    );
    assert_eq!(
        v.default_value(DomTextareaReadLimits::new(4, 6), &cancel),
        Err(DomError::TextareaReadWorkLimit { limit: 6 })
    );
    assert_eq!(
        v.value(DomTextareaReadLimits::new(3, 12), &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    );
}

#[test]
fn raw_message_and_default_text_units_have_independent_exact_retained_costs() {
    let mut dom = owner();
    let (_, _, textarea) = setup(&mut dom);
    // Native Document31, HTML div31, HTML textarea36; fixed flags/selection0.
    assert_eq!(dom.usage().string_code_units(), 98);
    let mut b = DomBatch::new();
    b.set_textarea_default_value(textarea, "abc");
    b.set_custom_validity(textarea, "m\r\nn");
    apply(&mut dom, b);
    assert_eq!(dom.usage().string_code_units(), 107);
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.set_textarea_default_value(textarea, "z");
    apply(&mut dom, b);
    // Old detached Text3 stays managed, new Text1 + current raw1 + message3.
    assert_eq!(dom.usage().string_code_units(), 106);
    let mut b = DomBatch::new();
    b.detach(textarea);
    b.destroy(textarea);
    apply(&mut dom, b);
    // Document31 +div31 +the old detached Text3; retired family/subtree releases41.
    assert_eq!(dom.usage().string_code_units(), 65);
    assert_eq!(value(&old, textarea), "abc".into());
}

#[test]
fn one_short_current_units_and_late_wrong_family_refuse_the_whole_private_batch() {
    for cap in [100, 101] {
        let mut dom = owner();
        let (_, root, textarea) = setup(&mut dom);
        dom.set_limits(
            DomLimits::new(
                256,
                cap,
                1024,
                256,
                256,
                32,
                1048576,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_textarea_value(textarea, "abc");
        if cap == 100 {
            assert_eq!(
                dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 100
                }
            );
            unchanged(&dom, &before, usage);
        } else {
            apply(&mut dom, b);
            assert_eq!(dom.usage().string_code_units(), 101);
        }
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_textarea_value(textarea, "xy");
        b.set_textarea_value(root, "invalid");
        assert_eq!(
            dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::TextareaInapplicable
        );
        unchanged(&dom, &before, usage);
    }
}

#[test]
fn default_setter_admits_its_real_text_node_and_reached_raw_copy_before_publication() {
    for units in [103, 104] {
        let mut dom = owner();
        let (_, _, textarea) = setup(&mut dom);
        dom.set_limits(
            DomLimits::new(
                4,
                units,
                32,
                0,
                0,
                8,
                65536,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_textarea_default_value(textarea, "abc");
        // Base98 +new Text3 +clean current raw3. At103 the Text and actual
        // insertion are reached, then the required clean raw copy refuses.
        if units == 103 {
            assert_eq!(
                dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 103
                }
            );
            unchanged(&dom, &before, usage);
            assert!(dom.snapshot().node(textarea).unwrap().children().is_empty());
        } else {
            apply(&mut dom, b);
            assert_eq!(dom.usage().string_code_units(), 104);
            assert_eq!(dom.usage().managed_nodes(), 4);
        }
    }
    let mut dom = owner();
    let (_, _, textarea) = setup(&mut dom);
    dom.set_limits(
        DomLimits::new(
            3,
            1024,
            32,
            0,
            0,
            8,
            65536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.set_textarea_default_value(textarea, "abc");
    assert_eq!(
        dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 3
        }
    );
    unchanged(&dom, &before, usage);
}

#[test]
fn required_readonly_and_disabled_first_legend_are_derived_from_actual_tree_and_attributes() {
    use surgeist_html_semantics::*;
    let mut dom = owner();
    let (d, root, textarea) = setup(&mut dom);
    let mut b = DomBatch::new();
    let fieldset = b.create_element_ns(d, Namespace::html(), "fieldset");
    let legend = b.create_element_ns(d, Namespace::html(), "legend");
    let outside = b.create_element_ns(d, Namespace::html(), "textarea");
    b.set_attribute(&fieldset, "disabled", "");
    b.insert_before(&fieldset, root, None);
    b.insert_before(&legend, &fieldset, None);
    b.insert_before(textarea, &legend, None);
    b.insert_before(&outside, &fieldset, None);
    b.set_attribute(textarea, "required", "");
    b.set_attribute(&outside, "required", "");
    let c = apply(&mut dom, b);
    let outside = c.created(&outside).unwrap().unwrap();
    let captured = capture(&dom.snapshot(), d);
    assert_eq!(
        fact(&captured, textarea, HtmlSemanticQuery::Disabledness),
        HtmlSemanticFact::Disabledness(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        fact(&captured, outside, HtmlSemanticQuery::Disabledness),
        HtmlSemanticFact::Disabledness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        fact(
            &captured,
            textarea,
            HtmlSemanticQuery::ConstraintValidationCandidate
        ),
        HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        fact(&captured, textarea, HtmlSemanticQuery::TextareaValueMissing),
        HtmlSemanticFact::TextareaValueMissing(HtmlSemanticValue::Known(true))
    );
    let mut b = DomBatch::new();
    b.set_attribute(textarea, "readonly", "");
    b.set_textarea_value(textarea, "");
    apply(&mut dom, b);
    assert_eq!(
        fact(
            &capture(&dom.snapshot(), d),
            textarea,
            HtmlSemanticQuery::ConstraintValidationCandidate
        ),
        HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        fact(
            &captured,
            textarea,
            HtmlSemanticQuery::ConstraintValidationCandidate
        ),
        HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(true))
    );
}

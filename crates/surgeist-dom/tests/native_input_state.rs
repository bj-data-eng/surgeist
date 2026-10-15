//! Literal HTML877d native Input state/hook expectations through the real owner.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{
    HtmlInputEmptySelectionSupport as Empty, HtmlInputImageHistory,
    HtmlInputNoneDirectionSupport as NoneDirection, HtmlInputSelectionDirection as Direction,
    HtmlInputType as Type,
};

#[path = "support/input_processing.rs"]
mod processing_envelope;

fn limits() -> DomLimits {
    // At most four processing updates occur in any one batch in these cases:
    // the four Range min/max/step/current hooks dominate the three Color hooks.
    // Each Color literal is nonrecursive, <=64 UTF-16 units and <=4 components.
    let (scratch, work) = processing_envelope::color_limits(4);
    DomLimits::new(512, 131072, 2048, 512, 512, 32, 1048576)
        .unwrap()
        .with_input_processing(scratch, work)
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
    let input = b.create_element_ns(&d, Namespace::html(), "input");
    b.insert_before(&root, &d, None);
    b.insert_before(&input, &root, None);
    let c = apply(dom, b);
    (
        c.document(&d).unwrap(),
        c.created(&root).unwrap().unwrap(),
        c.created(&input).unwrap().unwrap(),
    )
}
fn value(s: &DomSnapshot, n: NodeId) -> DocumentString {
    DocumentString::from_utf16(s.node(n).unwrap().as_input().unwrap().value().to_vec())
}
fn state(s: &DomSnapshot, n: NodeId) -> &DomInputState {
    s.node(n).unwrap().as_input().unwrap().state()
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
fn fresh_state_and_template_environment_use_each_independent_selected_platform() {
    for empty in [Empty::Supported, Empty::Unsupported] {
        for none in [NoneDirection::Supported, NoneDirection::Unsupported] {
            let platform = DomNativeInputPlatform::new(empty, none);
            let mut dom = Dom::with_native_input_platform(limits(), platform).unwrap();
            let (d, root, input) = setup(&mut dom);
            let s = dom.snapshot();
            let environment = s.document(d).unwrap().metadata().native_environment();
            assert_eq!(environment.input_platform(), platform);
            assert_eq!(
                environment.input_images(),
                surgeist_html::HtmlInputImagesDisabled::ExplicitlyDisabled
            );
            let input_state = state(&s, input);
            let flags = input_state.flags();
            assert_eq!(flags.input_type(), Type::Text);
            assert!(!flags.dirty_value());
            assert!(!flags.dirty_checkedness());
            assert!(!flags.checkedness());
            assert!(!flags.indeterminate());
            assert!(!flags.user_validity());
            assert!(!input_state.parser_inserted());
            assert!(input_state.custom_validity_message().is_empty());
            assert_eq!(value(&s, input), "".into());
            assert_eq!(
                s.node(input).unwrap().as_input().unwrap().form_owner(),
                None
            );
            let selection = input_state.selection().unwrap();
            assert_eq!(selection.cursor_position(), Some(0));
            assert_eq!(
                selection.direction(),
                if none == NoneDirection::Supported {
                    Direction::None
                } else {
                    Direction::Forward
                }
            );
            assert!(matches!(
                input_state.image_history(),
                HtmlInputImageHistory::NeverEntered
            ));
            assert_eq!(input_state.image_history().coordinate().x(), 0);
            assert_eq!(input_state.image_history().coordinate().y(), 0);
            assert_eq!(input_state.selected_files(), None);
            let mut b = DomBatch::new();
            let template = b.create_element_ns(d, Namespace::html(), "template");
            b.insert_before(&template, root, None);
            let c = apply(&mut dom, b);
            let template = c.created(&template).unwrap().unwrap();
            let s = dom.snapshot();
            let contents = s.node(template).unwrap().as_template().unwrap().contents();
            let inert = s.node(contents).unwrap().owner_document().unwrap();
            assert_eq!(
                s.document(inert)
                    .unwrap()
                    .metadata()
                    .native_environment()
                    .input_platform(),
                platform
            );
            let mut b = DomBatch::new();
            let other = b.create_element_ns(inert, Namespace::html(), "input");
            b.insert_before(&other, contents, None);
            let c = apply(&mut dom, b);
            let other = c.created(&other).unwrap().unwrap();
            assert_eq!(state(&dom.snapshot(), other).selection(), Some(selection));
        }
    }
}

#[test]
fn clean_defaults_dirty_values_checkedness_and_per_control_reset_stay_distinct() {
    let mut dom = owner();
    let (_, _, input) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_attribute(input, "value", "a\r\nb");
    b.set_attribute(input, "checked", "");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(value(&s, input), "ab".into());
    assert!(state(&s, input).flags().checkedness());
    assert!(!state(&s, input).flags().dirty_checkedness());
    let mut b = DomBatch::new();
    b.set_input_value(input, " x\r\ny ");
    b.set_input_checked(input, false);
    b.set_input_indeterminate(input, true);
    b.set_custom_validity(input, "bad\r\nmessage");
    b.set_attribute(input, "value", "later");
    b.set_attribute(input, "checked", "still present");
    apply(&mut dom, b);
    let old = dom.snapshot();
    assert_eq!(value(&old, input), " xy ".into());
    let flags = state(&old, input).flags();
    assert!(flags.dirty_value());
    assert!(flags.dirty_checkedness());
    assert!(!flags.checkedness());
    assert!(flags.indeterminate());
    assert_eq!(
        state(&old, input).custom_validity_message(),
        &"bad\nmessage".into()
    );
    let mut b = DomBatch::new();
    b.reset_input(input);
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(value(&s, input), "later".into());
    let flags = state(&s, input).flags();
    assert!(!flags.dirty_value());
    assert!(!flags.dirty_checkedness());
    assert!(flags.checkedness());
    assert!(flags.indeterminate());
    assert!(!flags.user_validity());
    assert_eq!(
        state(&s, input).custom_validity_message(),
        &"bad\nmessage".into()
    );
    assert_eq!(value(&old, input), " xy ".into());
    let mut b = DomBatch::new();
    b.remove_attribute(input, "value");
    b.remove_attribute(input, "checked");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(value(&s, input), "".into());
    assert!(!state(&s, input).flags().checkedness());
}

#[test]
fn every_type_uses_its_actual_value_mode_and_source_sanitizer() {
    for (kind, raw, expected) in [
        ("text", " a\r\nb ", " ab "),
        ("search", " a\r\nb ", " ab "),
        ("tel", " a\r\nb ", " ab "),
        ("password", " a\r\nb ", " ab "),
        ("url", "\t a\r\nb \n", "ab"),
        ("email", "\t a\r\nb \n", "ab"),
        ("date", "2024-02-29", "2024-02-29"),
        ("month", "2024-02", "2024-02"),
        ("week", "2020-W53", "2020-W53"),
        ("time", "12:34:56.789", "12:34:56.789"),
        (
            "datetime-local",
            "2024-02-29 12:30:00.000",
            "2024-02-29T12:30",
        ),
        ("number", "-0", "-0"),
        ("range", "6", "6"),
        ("color", "red", "#ff0000"),
        ("hidden", "\rraw\n", "\rraw\n"),
        ("submit", "\rraw\n", "\rraw\n"),
        ("reset", "\rraw\n", "\rraw\n"),
        ("button", "\rraw\n", "\rraw\n"),
        ("image", "\rraw\n", "\rraw\n"),
        ("checkbox", "\rraw\n", "\rraw\n"),
        ("radio", "\rraw\n", "\rraw\n"),
    ] {
        let mut dom = owner();
        let (_, _, input) = setup(&mut dom);
        let mut b = DomBatch::new();
        b.set_attribute(input, "type", kind);
        b.set_input_value(input, raw);
        apply(&mut dom, b);
        assert_eq!(
            value(&dom.snapshot(), input),
            DocumentString::from(expected),
            "{kind}"
        );
    }
    for (kind, raw) in [
        ("date", "2023-02-29"),
        ("month", "2024-13"),
        ("week", "2021-W53"),
        ("time", "25:00"),
        ("datetime-local", "2024-02-30T12:00"),
        ("number", "NaN"),
        ("number", "1 "),
    ] {
        let mut dom = owner();
        let (_, _, input) = setup(&mut dom);
        let mut b = DomBatch::new();
        b.set_attribute(input, "type", kind);
        b.set_input_value(input, raw);
        apply(&mut dom, b);
        assert_eq!(value(&dom.snapshot(), input), "".into(), "{kind} {raw}");
    }
    let mut dom = owner();
    let (_, _, input) = setup(&mut dom);
    let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00, 13, 10]);
    let mut b = DomBatch::new();
    b.set_input_value(input, raw);
    apply(&mut dom, b);
    assert_eq!(
        value(&dom.snapshot(), input).code_units(),
        &[0xd800, 0, 0xdc00]
    );
}

#[test]
fn type_transfers_default_on_file_and_selection_follow_the_ordered_effects() {
    let mut dom = owner();
    let (_, _, input) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_input_value(input, "abc");
    b.set_attribute(input, "type", "checkbox");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(value(&s, input), "abc".into());
    assert_eq!(
        s.node(input)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"value".into()),
        Some(&"abc".into())
    );
    assert!(state(&s, input).flags().dirty_value());
    assert_eq!(state(&s, input).selection(), None);
    let mut b = DomBatch::new();
    b.remove_attribute(input, "value");
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), input), "on".into());
    let mut b = DomBatch::new();
    b.set_attribute(input, "value", "a\r\nb");
    b.set_attribute(input, "type", "text");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(value(&s, input), "ab".into());
    assert!(!state(&s, input).flags().dirty_value());
    assert_eq!(
        state(&s, input).selection().unwrap().cursor_position(),
        Some(0)
    );
    let mut b = DomBatch::new();
    b.set_attribute(input, "type", "file");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(value(&s, input), "".into());
    assert_eq!(
        state(&s, input).selected_files(),
        Some(surgeist_html::HtmlInputFileSelection::Empty)
    );
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.set_input_value(input, "fake.txt");
    assert_eq!(
        dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::InputFilenameInvalidState
    );
    unchanged(&dom, &s, usage);
    let mut b = DomBatch::new();
    b.set_input_value(input, "");
    b.reset_input(input);
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), input), "".into());
}

#[test]
fn email_multiple_uses_real_produced_tokens_and_repeated_flag_changes() {
    let mut dom = owner();
    let (_, _, input) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_attribute(input, "type", "email");
    b.set_attribute(input, "multiple", "");
    b.set_input_value(input, " a@example.test , , b@example.test ");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(value(&s, input), "a@example.test,,b@example.test".into());
    assert_eq!(state(&s, input).email_value_count(), Some(3));
    let mut b = DomBatch::new();
    b.set_attribute(input, "multiple", "again");
    apply(&mut dom, b);
    assert_eq!(state(&dom.snapshot(), input).email_value_count(), Some(3));
    let mut b = DomBatch::new();
    b.remove_attribute(input, "multiple");
    apply(&mut dom, b);
    assert_eq!(state(&dom.snapshot(), input).email_value_count(), None);
    assert_eq!(
        value(&dom.snapshot(), input),
        "a@example.test,,b@example.test".into()
    );
    let mut b = DomBatch::new();
    b.set_attribute(input, "multiple", "");
    b.set_input_value(input, "");
    apply(&mut dom, b);
    assert_eq!(state(&dom.snapshot(), input).email_value_count(), Some(0));
}

#[test]
fn range_bounds_step_ties_and_signed_zero_use_the_shared_exact_sequence() {
    let mut dom = owner();
    let (_, _, input) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_attribute(input, "type", "range");
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), input), "50".into());
    let mut b = DomBatch::new();
    b.set_attribute(input, "min", "0");
    b.set_attribute(input, "max", "10");
    b.set_attribute(input, "step", "4");
    b.set_input_value(input, "6");
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), input), "8".into());
    let mut b = DomBatch::new();
    b.set_attribute(input, "step", "any");
    b.set_input_value(input, "-0");
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), input), "-0".into());
    let mut b = DomBatch::new();
    b.set_attribute(input, "min", "10");
    b.set_attribute(input, "max", "0");
    b.set_input_value(input, "not finite");
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), input), "10".into());
}

#[test]
fn color_standard_failure_dirty_source_and_missing_facts_are_distinct() {
    let mut dom = owner();
    let (d, _, input) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_attribute(input, "type", "color");
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), input), "#000000".into());
    let mut b = DomBatch::new();
    b.set_input_value(input, "red");
    b.set_attribute(input, "value", "currentColor");
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), input), "#ff0000".into());
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.set_input_value(input, "currentColor");
    assert!(matches!(
        dom.apply(b, &AtomicBool::new(false)),
        Err(DomError::InputColorUnavailable(_))
    ));
    unchanged(&dom, &before, usage);
    let mut b = DomBatch::new();
    let copy = b.import_node(&before, input, d, false);
    let c = apply(&mut dom, b);
    let copy = c.created(&copy).unwrap().unwrap();
    assert_eq!(value(&dom.snapshot(), copy), "#ff0000".into());
    assert!(state(&dom.snapshot(), copy).flags().dirty_value());
    let mut b = DomBatch::new();
    b.set_input_value(input, "not a color");
    b.set_attribute(input, "alpha", "");
    b.set_attribute(input, "colorspace", "display-p3");
    apply(&mut dom, b);
    assert_eq!(
        value(&dom.snapshot(), input),
        "color(display-p3 0 0 0)".into()
    );
}

#[test]
fn image_history_distinguishes_absent_empty_and_reentry_without_acquisition() {
    let mut dom = owner();
    let (_, _, input) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_attribute(input, "type", "image");
    apply(&mut dom, b);
    assert!(matches!(
        state(&dom.snapshot(), input).image_history(),
        HtmlInputImageHistory::Last { source: None, .. }
    ));
    let mut b = DomBatch::new();
    b.set_attribute(input, "src", "");
    apply(&mut dom, b);
    assert!(
        matches!(state(&dom.snapshot(),input).image_history(),HtmlInputImageHistory::Last {source:Some(raw),..} if raw.is_empty())
    );
    let mut b = DomBatch::new();
    b.set_attribute(input, "src", "img");
    b.set_attribute(input, "type", "text");
    b.set_attribute(input, "src", "other");
    apply(&mut dom, b);
    let old = dom.snapshot();
    assert!(
        matches!(state(&old,input).image_history(),HtmlInputImageHistory::Last {source:Some(raw),..} if raw==&DocumentString::from("img"))
    );
    let mut b = DomBatch::new();
    b.set_attribute(input, "type", "image");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert!(
        matches!(state(&s,input).image_history(),HtmlInputImageHistory::Last {source:Some(raw),..} if raw==&DocumentString::from("other"))
    );
    assert_eq!(state(&s, input).image_history().coordinate().x(), 0);
    assert_eq!(state(&s, input).image_history().coordinate().y(), 0);
    let mut b = DomBatch::new();
    b.set_attribute(input, "src", "other");
    apply(&mut dom, b);
    assert_eq!(dom.snapshot().stamp(), s.stamp());
}

#[test]
fn radios_use_exact_nonempty_names_ordinary_roots_and_real_owner_groups() {
    let mut dom = owner();
    let (d, root, first) = setup(&mut dom);
    let mut b = DomBatch::new();
    let second = b.create_element_ns(d, Namespace::html(), "input");
    let separate = b.create_element_ns(d, Namespace::html(), "input");
    for node in [
        NodeOperand::from(first),
        (&second).into(),
        (&separate).into(),
    ] {
        b.set_attribute(node.clone(), "type", "radio");
        b.set_attribute(node.clone(), "name", "x");
        b.set_input_checked(node, true);
    }
    b.insert_before(&second, root, None);
    let c = apply(&mut dom, b);
    let second = c.created(&second).unwrap().unwrap();
    let separate = c.created(&separate).unwrap().unwrap();
    let s = dom.snapshot();
    assert!(!state(&s, first).flags().checkedness());
    assert!(state(&s, second).flags().checkedness());
    assert!(state(&s, separate).flags().checkedness());
    assert!(state(&s, first).flags().dirty_checkedness());
    let mut b = DomBatch::new();
    b.set_input_indeterminate(first, true);
    b.set_input_checked(first, true);
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert!(state(&s, first).flags().checkedness());
    assert!(!state(&s, second).flags().checkedness());
    assert!(state(&s, first).flags().indeterminate());
    assert!(state(&s, second).flags().dirty_checkedness());
    let mut b = DomBatch::new();
    b.set_attribute(second, "name", "X");
    b.set_input_checked(second, true);
    apply(&mut dom, b);
    assert!(state(&dom.snapshot(), first).flags().checkedness());
    let mut b = DomBatch::new();
    b.set_attribute(second, "name", "x");
    apply(&mut dom, b);
    assert!(!state(&dom.snapshot(), first).flags().checkedness());
    assert!(state(&dom.snapshot(), second).flags().checkedness());
    let mut b = DomBatch::new();
    b.remove_attribute(second, "name");
    b.remove_attribute(first, "name");
    b.set_input_checked(first, true);
    b.set_input_checked(second, true);
    apply(&mut dom, b);
    assert!(state(&dom.snapshot(), first).flags().checkedness());
    assert!(state(&dom.snapshot(), second).flags().checkedness());
}

#[test]
fn owner_reset_normalizes_the_intermediate_null_group_but_nearest_retention_does_not() {
    let mut dom = owner();
    let (d, root, null_peer) = setup(&mut dom);
    let mut b = DomBatch::new();
    let form = b.create_element_ns(d, Namespace::html(), "form");
    let owned = b.create_element_ns(d, Namespace::html(), "input");
    b.set_attribute(&form, "id", "f");
    b.insert_before(&form, root, None);
    b.insert_before(&owned, &form, None);
    for node in [NodeOperand::from(null_peer), (&owned).into()] {
        b.set_attribute(node.clone(), "type", "radio");
        b.set_attribute(node.clone(), "name", "x");
        b.set_input_checked(node, true);
    }
    let c = apply(&mut dom, b);
    let form = c.created(&form).unwrap().unwrap();
    let owned = c.created(&owned).unwrap().unwrap();
    let s = dom.snapshot();
    assert!(state(&s, null_peer).flags().checkedness());
    assert!(state(&s, owned).flags().checkedness());
    assert_eq!(
        s.node(owned).unwrap().as_input().unwrap().form_owner(),
        Some(form)
    );
    let mut b = DomBatch::new();
    b.set_attribute(owned, "form", "f");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(
        s.node(owned).unwrap().as_input().unwrap().form_owner(),
        Some(form)
    );
    assert!(!state(&s, null_peer).flags().checkedness());
    assert!(state(&s, owned).flags().checkedness());
    let mut b = DomBatch::new();
    b.set_input_checked(null_peer, true);
    b.remove_attribute(owned, "form");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(
        s.node(owned).unwrap().as_input().unwrap().form_owner(),
        Some(form)
    );
    assert!(state(&s, null_peer).flags().checkedness());
    assert!(state(&s, owned).flags().checkedness());
    let mut b = DomBatch::new();
    let collision = b.create_element_ns(d, Namespace::html(), "span");
    b.set_attribute(&collision, "id", "f");
    b.insert_before(&collision, root, Some(form.into()));
    b.set_attribute(owned, "form", "f");
    let c = apply(&mut dom, b);
    let collision = c.created(&collision).unwrap().unwrap();
    let s = dom.snapshot();
    assert_eq!(
        s.node(owned).unwrap().as_input().unwrap().form_owner(),
        None
    );
    assert!(!state(&s, null_peer).flags().checkedness());
    assert!(state(&s, owned).flags().checkedness());
    assert_eq!(s.node(collision).unwrap().parent(), Some(root));
}

#[test]
fn input_popover_has_one_real_weak_slot_with_same_batch_cross_document_and_retirement_rules() {
    let mut dom = owner();
    let (d, root, input) = setup(&mut dom);
    let mut b = DomBatch::new();
    let other_doc = b.create_document();
    let target = b.create_element_ns(&other_doc, Namespace::html(), "div");
    b.set_popover_target_element(input, Some((&target).into()));
    let c = apply(&mut dom, b);
    let target = c.created(&target).unwrap().unwrap();
    let other_doc = c.document(&other_doc).unwrap();
    assert_eq!(
        dom.snapshot()
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .explicit_popover_target(),
        Some(target)
    );
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.adopt(input, other_doc);
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(s.node(input).unwrap().owner_document().unwrap(), other_doc);
    assert_eq!(
        s.node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .explicit_popover_target(),
        Some(target)
    );
    let mut b = DomBatch::new();
    b.set_attribute(input, "popovertarget", "");
    apply(&mut dom, b);
    assert_eq!(
        dom.snapshot()
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .explicit_popover_target(),
        None
    );
    let mut b = DomBatch::new();
    b.set_popover_target_element(input, Some(input.into()));
    apply(&mut dom, b);
    assert_eq!(
        dom.snapshot()
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .explicit_popover_target(),
        Some(input)
    );
    let mut b = DomBatch::new();
    b.set_popover_target_element(input, Some(target.into()));
    b.destroy(target);
    let c = apply(&mut dom, b);
    assert!(!c.changes().changes().is_empty());
    assert_eq!(
        dom.snapshot()
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .explicit_popover_target(),
        None
    );
    assert_eq!(
        old.node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .explicit_popover_target(),
        Some(target)
    );
    let mut b = DomBatch::new();
    let reused = b.create_element_ns(other_doc, Namespace::html(), "div");
    b.set_command_for_element(input, Some((&reused).into()));
    let before = dom.snapshot();
    let usage = dom.usage();
    assert_eq!(
        dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ButtonInapplicable
    );
    unchanged(&dom, &before, usage);
    assert_eq!(before.node(root).unwrap().owner_document().unwrap(), d);
}

#[test]
fn clone_copies_only_input_overlay_while_adoption_preserves_live_state() {
    let mut source = owner();
    let (d, _, input) = setup(&mut source);
    let mut b = DomBatch::new();
    let target = b.create_element_ns(d, Namespace::html(), "div");
    b.set_input_value(input, "abc");
    b.set_input_checked(input, true);
    b.set_input_indeterminate(input, true);
    b.set_custom_validity(input, "custom");
    b.set_popover_target_element(input, Some((&target).into()));
    let c = apply(&mut source, b);
    let target = c.created(&target).unwrap().unwrap();
    let old = source.snapshot();
    assert_eq!(
        state(&old, input).selection().unwrap().cursor_position(),
        Some(3)
    );
    let mut destination = Dom::with_native_input_platform(
        limits(),
        DomNativeInputPlatform::new(Empty::Unsupported, NoneDirection::Unsupported),
    )
    .unwrap();
    let (target_doc, _, _) = setup(&mut destination);
    let mut b = DomBatch::new();
    let copy = b.import_node(&old, input, target_doc, false);
    let c = apply(&mut destination, b);
    let copy = c.created(&copy).unwrap().unwrap();
    let s = destination.snapshot();
    let copied = state(&s, copy);
    assert_eq!(value(&s, copy), "abc".into());
    assert!(copied.flags().dirty_value());
    assert!(copied.flags().dirty_checkedness());
    assert!(copied.flags().checkedness());
    assert!(copied.flags().indeterminate());
    assert!(copied.custom_validity_message().is_empty());
    assert!(!copied.parser_inserted());
    assert_eq!(copied.selection().unwrap().cursor_position(), Some(0));
    assert_eq!(copied.selection().unwrap().direction(), Direction::Forward);
    assert_eq!(
        s.node(copy)
            .unwrap()
            .as_input()
            .unwrap()
            .explicit_popover_target(),
        None
    );
    assert_eq!(s.node(copy).unwrap().as_input().unwrap().form_owner(), None);
    let mut b = DomBatch::new();
    let new_doc = b.create_document();
    b.adopt(input, &new_doc);
    let c = apply(&mut source, b);
    let new_doc = c.document(&new_doc).unwrap();
    let s = source.snapshot();
    assert_eq!(s.node(input).unwrap().owner_document().unwrap(), new_doc);
    assert_eq!(state(&s, input), state(&old, input));
    assert_eq!(
        s.node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .explicit_popover_target(),
        Some(target)
    );
    assert_eq!(old.node(input).unwrap().owner_document().unwrap(), d);
}

#[test]
fn reached_processing_limits_and_late_invalid_operations_are_atomic() {
    let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 65536).unwrap()).unwrap();
    let (_, _, input) = setup(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.set_input_value(input, "private");
    b.set_attribute(input, "type", "range");
    assert!(matches!(
        dom.apply(b, &AtomicBool::new(false)),
        Err(DomError::InputProcessingWorkLimit { limit: 0 })
    ));
    unchanged(&dom, &before, usage);
    // Fixed service entry first charges work. Isolate zero scratch with an
    // explicit nonzero work ceiling, rather than expecting a later first stop.
    dom.set_limits(dom.limits().with_input_processing(0, 100000000).unwrap())
        .unwrap();
    let mut b = DomBatch::new();
    b.set_input_value(input, "private");
    b.set_attribute(input, "type", "range");
    assert!(matches!(
        dom.apply(b, &AtomicBool::new(false)),
        Err(DomError::ResourceLimit {
            resource: DomResource::InputProcessingScratch,
            limit: 0
        })
    ));
    unchanged(&dom, &before, usage);
    dom.set_limits(dom.limits().with_input_processing(16777216, 0).unwrap())
        .unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(input, "type", "color");
    assert!(matches!(
        dom.apply(b, &AtomicBool::new(false)),
        Err(DomError::InputProcessingWorkLimit { limit: 0 })
    ));
    unchanged(&dom, &before, usage);
    dom.set_limits(limits()).unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(input, "type", "range");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(value(&s, input), "50".into());
    dom.set_limits(dom.limits().with_input_processing(0, 0).unwrap())
        .unwrap();
    assert_eq!(value(&dom.snapshot(), input), "50".into());
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.set_input_value(input, "51");
    assert!(matches!(
        dom.apply(b, &AtomicBool::new(false)),
        Err(DomError::InputProcessingWorkLimit { limit: 0 })
    ));
    unchanged(&dom, &s, usage);
}

fn capture(s: &DomSnapshot, d: DocumentId) -> DomHtmlCapture {
    s.html_capture(
        d,
        None,
        surgeist_html_semantics::HtmlCaptureLimits::new(512, 2048, 512, 131072, 512, 0, 0),
        DomHtmlCaptureLimits::new(512, 10000000),
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
        surgeist_html::HtmlValueInterpreter::new(131072),
        HtmlSemanticLimits::new(10000000, 32, 2048, 2048, 10000000, 2048),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .results()[0]
        .fact()
        .clone()
}
#[test]
fn semantic_current_values_flags_and_form_owner_survive_actual_slot_supplementation() {
    use surgeist_html_semantics::*;
    let mut dom = owner();
    let (d, root, input) = setup(&mut dom);
    let mut b = DomBatch::new();
    let form = b.create_element_ns(d, Namespace::html(), "form");
    let slot = b.create_element_ns(d, Namespace::html(), "slot");
    b.insert_before(&form, root, None);
    b.insert_before(input, &form, None);
    b.insert_before(&slot, root, None);
    b.set_attribute(input, "dir", "auto");
    b.set_attribute(input, "value", "LTR default");
    b.set_input_value(input, "אב");
    b.set_input_checked(input, true);
    b.set_input_indeterminate(input, true);
    b.assign_slot(&slot, vec![input.into()]);
    let c = apply(&mut dom, b);
    let form = c.created(&form).unwrap().unwrap();
    let current = dom.snapshot();
    let initial = capture(&current, d);
    for supplemented in [false, true] {
        let capture = if supplemented {
            capture(&current, d)
                .with_slot_semantics(
                    DomSlotSemanticLimits::new(
                        DomSlotQueryLimits::new(512, 2048).unwrap(),
                        DomHtmlCaptureLimits::new(512, 10000000),
                        HtmlLanguageDirectionLimits::new(1024, 2048, 131072, 10000000),
                        HtmlInertFactLimits::new(1024, 10000000),
                    ),
                    &AtomicBool::new(false),
                )
                .unwrap()
        } else {
            capture(&current, d)
        };
        assert_eq!(
            fact(&capture, input, HtmlSemanticQuery::Directionality),
            HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
        );
        assert_eq!(
            fact(&capture, input, HtmlSemanticQuery::CurrentCheckedness),
            HtmlSemanticFact::CurrentCheckedness(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            fact(&capture, input, HtmlSemanticQuery::DirtyValueFlag),
            HtmlSemanticFact::DirtyValueFlag(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            fact(&capture, input, HtmlSemanticQuery::DirtyCheckednessFlag),
            HtmlSemanticFact::DirtyCheckednessFlag(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            fact(&capture, input, HtmlSemanticQuery::Indeterminateness),
            HtmlSemanticFact::Indeterminateness(HtmlSemanticValue::Known(true))
        );
        let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) =
            fact(&capture, input, HtmlSemanticQuery::FormOwner)
        else {
            panic!("actual Form owner required")
        };
        assert_eq!(owner.owner(), Some(&capture.subject(form).unwrap()));
        assert_eq!(owner.origin(), HtmlFormOwnerOrigin::EstablishedCurrent);
    }
    let mut b = DomBatch::new();
    b.set_input_value(input, "new LTR");
    b.set_input_checked(input, false);
    apply(&mut dom, b);
    assert_eq!(
        fact(&initial, input, HtmlSemanticQuery::Directionality),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
    );
    assert_eq!(
        fact(
            &capture(&dom.snapshot(), d),
            input,
            HtmlSemanticQuery::Directionality
        ),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Ltr))
    );
}

#[test]
fn capture_admits_both_association_and_input_records_for_every_detached_input() {
    use surgeist_html_semantics::*;
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let first = b.create_element_ns(&d, Namespace::html(), "input");
    let second = b.create_element_ns(&d, Namespace::html(), "input");
    b.set_input_checked(&first, true);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let first = c.created(&first).unwrap().unwrap();
    let second = c.created(&second).unwrap().unwrap();
    let capture = dom
        .snapshot()
        .html_capture(
            d,
            None,
            HtmlCaptureLimits::new(3, 8, 0, 1024, 8, 0, 0),
            DomHtmlCaptureLimits::new(3, 1000000),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(
        fact(&capture, first, HtmlSemanticQuery::CurrentCheckedness),
        HtmlSemanticFact::CurrentCheckedness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        fact(&capture, second, HtmlSemanticQuery::CurrentCheckedness),
        HtmlSemanticFact::CurrentCheckedness(HtmlSemanticValue::Known(false))
    );
}

#[test]
fn native_fragment_installation_uses_real_input_defaults_and_live_commands() {
    let mut dom = owner();
    let (d, root, _) = setup(&mut dom);
    let mut builder = surgeist_html::HtmlFragment::builder(
        surgeist_html::HtmlConstructionLimits::new(16, 16, 16, 1024, 16),
    )
    .unwrap();
    builder
        .append_element(
            &builder.root(),
            QualifiedName::new(Namespace::html(), None, "input"),
            OrderedAttributes::new([
                Attribute::new(QualifiedName::new(Namespace::none(), None, "type"), "email"),
                Attribute::new(QualifiedName::new(Namespace::none(), None, "multiple"), ""),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "value"),
                    " a@example.test, b@example.test ",
                ),
            ])
            .unwrap(),
        )
        .unwrap();
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
    let input = result.node_for(&subject).unwrap();
    assert_eq!(
        value(result.snapshot(), input),
        "a@example.test,b@example.test".into()
    );
    assert!(!state(result.snapshot(), input).flags().dirty_value());
    assert_eq!(state(result.snapshot(), input).email_value_count(), Some(2));
    let mut b = DomBatch::new();
    b.set_input_value(input, "new@example.test");
    apply(&mut dom, b);
    assert_eq!(value(&dom.snapshot(), input), "new@example.test".into());
    assert_eq!(
        value(result.snapshot(), input),
        "a@example.test,b@example.test".into()
    );
}

#[test]
fn checked_attribute_redundant_set_is_not_added_and_shadow_roots_partition_radios() {
    let mut dom = owner();
    let (d, root, first) = setup(&mut dom);
    let mut b = DomBatch::new();
    let second = b.create_element_ns(d, Namespace::html(), "input");
    b.insert_before(&second, root, None);
    for node in [NodeOperand::from(first), (&second).into()] {
        b.set_attribute(node.clone(), "type", "radio");
        b.set_attribute(node.clone(), "name", "group");
        b.set_attribute(node, "checked", "");
    }
    let c = apply(&mut dom, b);
    let second = c.created(&second).unwrap().unwrap();
    let s = dom.snapshot();
    assert!(!state(&s, first).flags().checkedness());
    assert!(!state(&s, first).flags().dirty_checkedness());
    assert!(state(&s, second).flags().checkedness());
    let mut b = DomBatch::new();
    b.set_attribute(first, "checked", "same presence");
    apply(&mut dom, b);
    assert!(!state(&dom.snapshot(), first).flags().checkedness());
    let mut b = DomBatch::new();
    b.remove_attribute(first, "checked");
    b.set_attribute(first, "checked", "");
    let host = b.create_element_ns(d, Namespace::html(), "div");
    b.insert_before(&host, root, None);
    let shadow = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let isolated = b.create_element_ns(d, Namespace::html(), "input");
    b.set_attribute(&isolated, "type", "radio");
    b.set_attribute(&isolated, "name", "group");
    b.set_attribute(&isolated, "checked", "");
    b.insert_before(&isolated, &shadow, None);
    let c = apply(&mut dom, b);
    let isolated = c.created(&isolated).unwrap().unwrap();
    let s = dom.snapshot();
    assert!(state(&s, first).flags().checkedness());
    assert!(!state(&s, second).flags().checkedness());
    assert!(state(&s, isolated).flags().checkedness());
    assert!(!state(&s, isolated).flags().dirty_checkedness());
    assert!(s.node(isolated).unwrap().is_connected());
}

#[test]
fn retained_current_message_and_image_history_units_are_independent_and_released() {
    let mut dom = owner();
    let (_, _, input) = setup(&mut dom);
    assert_eq!(dom.usage().string_code_units(), 95);
    let mut b = DomBatch::new();
    b.set_input_value(input, "abc");
    b.set_attribute(input, "value", "xyz");
    b.set_custom_validity(input, "err");
    apply(&mut dom, b);
    assert_eq!(dom.usage().string_code_units(), 109);
    let mut b = DomBatch::new();
    b.set_attribute(input, "type", "image");
    b.set_attribute(input, "src", "img");
    apply(&mut dom, b);
    let old = dom.snapshot();
    assert_eq!(dom.usage().string_code_units(), 127);
    assert_eq!(value(&old, input), "abc".into());
    assert_eq!(state(&old, input).current_value(), &"abc".into());
    let mut b = DomBatch::new();
    b.detach(input);
    b.destroy(input);
    apply(&mut dom, b);
    assert_eq!(dom.usage().string_code_units(), 62);
    assert_eq!(state(&old, input).custom_validity_message(), &"err".into());
    assert!(
        matches!(state(&old,input).image_history(),HtmlInputImageHistory::Last {source:Some(value),..} if value==&DocumentString::from("img"))
    );
}

#[test]
fn exact_current_string_capacity_and_non_input_commands_preserve_atomicity() {
    for (units, passes) in [(97, false), (98, true)] {
        let mut dom = Dom::new(DomLimits::new(16, units, 64, 0, 0, 8, 65536).unwrap()).unwrap();
        let (_, root, input) = setup(&mut dom);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_input_value(input, "abc");
        let result = dom.apply(b, &AtomicBool::new(false));
        if passes {
            result.unwrap();
            assert_eq!(dom.usage().string_code_units(), 98);
            assert!(state(&dom.snapshot(), input).flags().dirty_value());
        } else {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: units
                }
            );
            unchanged(&dom, &before, usage);
        }
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_input_checked(input, true);
        b.reset_input(root);
        assert_eq!(
            dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::InputInapplicable
        );
        unchanged(&dom, &before, usage);
    }
}

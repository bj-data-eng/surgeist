//! HTML877d native Script state and source hooks through the Disabled return.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlScriptKind as Kind, HtmlScriptResult};
use surgeist_html_semantics::HtmlCaptureLimits;

fn dom() -> Dom {
    Dom::new(DomLimits::new(256, 32768, 512, 128, 64, 16, 262144).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn flags(script: DomHtmlScript<'_>) -> [bool; 5] {
    let f = script.flags();
    [
        f.force_async(),
        f.already_started(),
        f.external_file(),
        f.ready_to_be_parser_executed(),
        f.delaying_load(),
    ]
}
fn script(s: &DomSnapshot, id: NodeId) -> DomHtmlScript<'_> {
    s.node(id).unwrap().as_html_script().unwrap()
}
fn fresh(s: &DomSnapshot, id: NodeId) {
    let state = script(s, id);
    assert_eq!(flags(state), [true, false, false, false, false]);
    assert_eq!(state.script_type(), None);
    assert_eq!(state.parser_document(), None);
    assert_eq!(state.preparation_document(), None);
    assert_eq!(state.result(), HtmlScriptResult::Uninitialized);
    assert_eq!(state.steps_when_result_ready(), None);
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId) {
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&root, &d, None);
    let c = apply(dom, b);
    (c.document(&d).unwrap(), c.created(&root).unwrap().unwrap())
}

#[test]
fn fresh_native_family_and_name_context_are_exact() {
    let mut dom = dom();
    let (d, _) = setup(&mut dom);
    let mut b = DomBatch::new();
    let exact = b.create_element_ns(d, Namespace::html(), "p:script");
    let upper = b.create_element_ns(d, Namespace::html(), "Script");
    let plain = b.create_element_ns(d, Namespace::none(), "script");
    let html = b.create_html_document(d, None);
    let folded = b.create_element(&html, "SCRIPT");
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    for t in [&exact, &folded] {
        fresh(&s, c.created(t).unwrap().unwrap());
    }
    for t in [&upper, &plain] {
        assert!(
            s.node(c.created(t).unwrap().unwrap())
                .unwrap()
                .as_html_script()
                .is_none()
        );
    }
    assert_eq!(
        s.node(c.created(&exact).unwrap().unwrap())
            .unwrap()
            .node_name(),
        "p:script".into()
    );
    assert_eq!(
        s.node(c.created(&folded).unwrap().unwrap())
            .unwrap()
            .node_name(),
        "SCRIPT".into()
    );
}

#[test]
fn recognized_types_write_started_type_and_actual_created_document_before_disabled() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    let cases = [
        (None, Some(Kind::Classic)),
        (Some(""), Some(Kind::Classic)),
        (Some(" \ttext/javascript\n"), Some(Kind::Classic)),
        (Some("MoDuLe"), Some(Kind::Module)),
        (Some("importmap"), Some(Kind::ImportMap)),
        (Some("speculationrules"), Some(Kind::SpeculationRules)),
        (Some(" module "), None),
        (Some("text/javascript; charset=utf-8"), None),
    ];
    let tickets: Vec<_> = cases
        .iter()
        .map(|(kind, _)| {
            let s = b.create_element_ns(&d, Namespace::html(), "script");
            if let Some(kind) = kind {
                b.set_attribute(&s, "type", *kind);
            }
            b.set_attribute(&s, "src", "");
            b.insert_before(&s, &root, None);
            s
        })
        .collect();
    b.insert_before(&root, &d, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let s = dom.snapshot();
    for (ticket, (_, kind)) in tickets.iter().zip(cases) {
        let state = script(&s, c.created(ticket).unwrap().unwrap());
        assert_eq!(flags(state), [true, kind.is_some(), false, false, false]);
        assert_eq!(state.script_type(), kind);
        assert_eq!(state.preparation_document(), kind.map(|_| d));
        assert_eq!(state.parser_document(), None);
        assert_eq!(state.result(), HtmlScriptResult::Uninitialized);
        assert_eq!(state.steps_when_result_ready(), None);
    }
}

#[test]
fn only_nonempty_immediate_text_counts_without_unicode_repair_or_trimming() {
    let mut dom = dom();
    let (d, root) = setup(&mut dom);
    let mut b = DomBatch::new();
    let empty = b.create_element_ns(d, Namespace::html(), "script");
    let empty_text = b.create_text(d, "");
    b.insert_before(&empty_text, &empty, None);
    let whitespace = b.create_element_ns(d, Namespace::html(), "script");
    let text = b.create_text(d, " \n");
    b.insert_before(&text, &whitespace, None);
    let units = b.create_element_ns(d, Namespace::html(), "script");
    let raw = b.create_text(d, DocumentString::from_utf16(vec![0, 0xd800]));
    b.insert_before(&raw, &units, None);
    let nested = b.create_element_ns(d, Namespace::html(), "script");
    let span = b.create_element_ns(d, Namespace::html(), "span");
    let text = b.create_text(d, "descendant");
    b.insert_before(&text, &span, None);
    b.insert_before(&span, &nested, None);
    let comment = b.create_comment(d, "not source text");
    b.insert_before(&comment, &nested, None);
    for node in [&empty, &whitespace, &units, &nested] {
        b.insert_before(node, root, None);
    }
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    for t in [&empty, &nested] {
        fresh(&s, c.created(t).unwrap().unwrap());
    }
    for t in [&whitespace, &units] {
        let state = script(&s, c.created(t).unwrap().unwrap());
        assert_eq!(flags(state), [true, true, false, false, false]);
        assert_eq!(state.script_type(), Some(Kind::Classic));
        assert_eq!(state.preparation_document(), Some(d));
    }
}

fn invalid_connected(dom: &mut Dom, d: DocumentId, root: NodeId) -> NodeId {
    let mut b = DomBatch::new();
    let s = b.create_element_ns(d, Namespace::html(), "script");
    b.set_attribute(&s, "type", "not-supported");
    b.set_attribute(&s, "src", "same");
    b.insert_before(&s, root, None);
    let c = apply(dom, b);
    c.created(&s).unwrap().unwrap()
}

#[test]
fn redundant_src_runs_preparation_after_type_repair_and_async_only_changes_on_added() {
    let mut dom = dom();
    let (d, root) = setup(&mut dom);
    let node = invalid_connected(&mut dom, d, root);
    let mut b = DomBatch::new();
    b.set_attribute(node, "type", "module");
    apply(&mut dom, b);
    fresh(&dom.snapshot(), node);
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.set_attribute(node, "src", "same");
    let c = apply(&mut dom, b);
    assert_eq!(c.changes().changes(), &[DomChange::ElementState(node)]);
    let s = dom.snapshot();
    assert_eq!(script(&s, node).script_type(), Some(Kind::Module));
    assert_eq!(script(&s, node).preparation_document(), Some(d));
    fresh(&before, node);
    let mut b = DomBatch::new();
    b.set_attribute(node, "async", "");
    apply(&mut dom, b);
    assert_eq!(
        flags(script(&dom.snapshot(), node)),
        [false, true, false, false, false]
    );
    let mut b = DomBatch::new();
    b.set_attribute(node, "async", "");
    assert!(apply(&mut dom, b).changes().changes().is_empty());
    let mut b = DomBatch::new();
    b.remove_attribute(node, "async");
    apply(&mut dom, b);
    assert!(!script(&dom.snapshot(), node).flags().force_async());

    let untouched = invalid_connected(&mut dom, d, root);
    let mut b = DomBatch::new();
    b.set_attribute(untouched, "type", "module");
    b.set_attribute_ns(untouched, Namespace::from_uri("urn:other"), "p:src", "x");
    b.set_attribute(untouched, "SRC", "x");
    b.set_attribute_ns(untouched, Namespace::from_uri("urn:other"), "p:async", "");
    b.remove_attribute(untouched, "src");
    apply(&mut dom, b);
    fresh(&dom.snapshot(), untouched);
}

#[test]
fn equal_character_data_and_empty_fragment_still_invoke_parent_children_changed() {
    for trigger in ["text", "comment", "pi", "empty-fragment", "remove"] {
        let mut dom = dom();
        let (d, root) = setup(&mut dom);
        let node = invalid_connected(&mut dom, d, root);
        let mut b = DomBatch::new();
        let text = b.create_text(d, "body");
        let comment = b.create_comment(d, "comment");
        let pi = b.create_processing_instruction(d, "target", "pi");
        for child in [&text, &comment, &pi] {
            b.insert_before(child, node, None);
        }
        b.set_attribute(node, "type", "");
        let c = apply(&mut dom, b);
        fresh(&dom.snapshot(), node);
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        match trigger {
            "text" => b.replace_data(c.created(&text).unwrap().unwrap(), 0, 0, ""),
            "comment" => b.replace_data(c.created(&comment).unwrap().unwrap(), 0, 0, ""),
            "pi" => b.replace_data(c.created(&pi).unwrap().unwrap(), 0, 0, ""),
            "remove" => b.detach(c.created(&comment).unwrap().unwrap()),
            _ => {
                let empty = b.create_fragment(d);
                b.insert_before(&empty, node, None);
            }
        }
        let c = apply(&mut dom, b);
        assert!(
            c.changes()
                .changes()
                .contains(&DomChange::ElementState(node))
        );
        assert!(
            !c.changes()
                .changes()
                .iter()
                .any(|c| matches!(c, DomChange::Data(_)))
        );
        assert_eq!(
            flags(script(&dom.snapshot(), node)),
            [true, true, false, false, false],
            "{trigger}"
        );
        assert_eq!(
            script(&dom.snapshot(), node).preparation_document(),
            Some(d)
        );
        fresh(&before, node);
    }
}

#[test]
fn language_is_borrowed_by_shared_classifier_and_explicit_type_takes_precedence() {
    let mut dom = dom();
    let (d, root) = setup(&mut dom);
    let mut b = DomBatch::new();
    let language = b.create_element_ns(d, Namespace::html(), "script");
    b.set_attribute(&language, "language", "javascript");
    b.set_attribute(&language, "src", "");
    b.insert_before(&language, root, None);
    let overridden = b.create_element_ns(d, Namespace::html(), "script");
    b.set_attribute(&overridden, "language", "javascript");
    b.set_attribute(
        &overridden,
        "type",
        DocumentString::from_utf16(vec![0xd800]),
    );
    b.set_attribute(&overridden, "src", "");
    b.insert_before(&overridden, root, None);
    let c = apply(&mut dom, b);
    assert_eq!(
        script(&dom.snapshot(), c.created(&language).unwrap().unwrap()).script_type(),
        Some(Kind::Classic)
    );
    fresh(&dom.snapshot(), c.created(&overridden).unwrap().unwrap());
}

#[test]
fn clone_copies_only_started_after_actual_copied_async_and_import_reads_old_snapshot() {
    let mut source = dom();
    let (d, root) = setup(&mut source);
    let mut b = DomBatch::new();
    let started = b.create_element_ns(d, Namespace::html(), "script");
    b.set_attribute(&started, "src", "old");
    b.set_attribute(&started, "async", "");
    b.set_attribute(&started, "nonce", "content");
    b.set_nonce(&started, "saved");
    b.insert_before(&started, root, None);
    let eligible = b.create_element_ns(d, Namespace::html(), "script");
    b.set_attribute(&eligible, "src", "old");
    b.set_attribute(&eligible, "async", "");
    let c = apply(&mut source, b);
    let started = c.created(&started).unwrap().unwrap();
    let eligible = c.created(&eligible).unwrap().unwrap();
    let old = source.snapshot();
    assert_eq!(
        flags(script(&old, eligible)),
        [false, false, false, false, false]
    );
    let mut b = DomBatch::new();
    let started_copy = b.import_node(&old, started, d, true);
    let eligible_copy = b.import_node(&old, eligible, d, false);
    b.insert_before(&started_copy, root, None);
    b.insert_before(&eligible_copy, root, None);
    let c = apply(&mut source, b);
    let started_copy = c.created(&started_copy).unwrap().unwrap();
    let eligible_copy = c.created(&eligible_copy).unwrap().unwrap();
    let s = source.snapshot();
    let copied = script(&s, started_copy);
    assert_ne!(started_copy, started);
    assert_eq!(flags(copied), [false, true, false, false, false]);
    assert_eq!(copied.script_type(), None);
    assert_eq!(copied.preparation_document(), None);
    assert_eq!(copied.parser_document(), None);
    assert_eq!(copied.result(), HtmlScriptResult::Uninitialized);
    assert_eq!(copied.steps_when_result_ready(), None);
    assert_eq!(script(&s, eligible_copy).preparation_document(), Some(d));
    assert_eq!(
        flags(script(&s, eligible_copy)),
        [false, true, false, false, false]
    );
    let mut b = DomBatch::new();
    b.set_attribute(started, "src", "new live");
    b.detach(started);
    b.destroy(started);
    apply(&mut source, b);
    drop(source);
    let mut target = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let imported = b.import_node(&old, started, &d, true);
    b.insert_before(&imported, &d, None);
    let c = apply(&mut target, b);
    let imported = c.created(&imported).unwrap().unwrap();
    let s = target.snapshot();
    assert_eq!(
        flags(script(&s, imported)),
        [false, true, false, false, false]
    );
    assert_eq!(script(&s, imported).preparation_document(), None);
    assert_eq!(script(&s, imported).script_type(), None);
    assert_eq!(
        s.node(imported)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"src".into()),
        Some(&"old".into())
    );
    assert_eq!(
        s.node(imported).unwrap().as_element().unwrap().nonce(),
        Some(&"saved".into())
    );
}

#[test]
fn adoption_retains_preparation_document_and_capture_closes_that_real_history() {
    let mut dom = dom();
    let (d1, root) = setup(&mut dom);
    let mut b = DomBatch::new();
    let d2 = b.create_document();
    let node = b.create_element_ns(d1, Namespace::html(), "script");
    b.set_attribute(&node, "src", "");
    b.insert_before(&node, root, None);
    let c = apply(&mut dom, b);
    let d2 = c.document(&d2).unwrap();
    let node = c.created(&node).unwrap().unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.adopt(node, d2);
    b.insert_before(node, d2, None);
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert_eq!(s.node(node).unwrap().owner_document(), Some(d2));
    assert_eq!(script(&s, node).preparation_document(), Some(d1));
    assert_eq!(script(&old, node).preparation_document(), Some(d1));
    assert_eq!(old.node(node).unwrap().owner_document(), Some(d1));
    let capture = s
        .html_capture(
            d2,
            None,
            HtmlCaptureLimits::new(64, 1024, 128, 16384, 64, 0, 0),
            DomHtmlCaptureLimits::new(64, 10000),
            &AtomicBool::new(false),
        )
        .unwrap();
    let subject = capture.subject(d1.node()).unwrap();
    assert_eq!(capture.node(&subject).unwrap(), d1.node());
    assert_eq!(
        script(capture.source(), node).preparation_document(),
        Some(d1)
    );
}

#[test]
fn template_contents_stay_disconnected_and_extracted_script_prepares_in_actual_document() {
    let mut dom = dom();
    let (d, root) = setup(&mut dom);
    let mut b = DomBatch::new();
    let template = b.create_element_ns(d, Namespace::html(), "template");
    b.insert_before(&template, root, None);
    let c = apply(&mut dom, b);
    let template = c.created(&template).unwrap().unwrap();
    let s = dom.snapshot();
    let contents = s.node(template).unwrap().as_template().unwrap().contents();
    let inert = s.node(contents).unwrap().owner_document().unwrap();
    let mut b = DomBatch::new();
    let inner = b.create_element_ns(inert, Namespace::html(), "script");
    b.set_attribute(&inner, "src", "");
    b.insert_before(&inner, contents, None);
    let ordinary = b.create_element_ns(d, Namespace::html(), "script");
    b.set_attribute(&ordinary, "src", "");
    b.insert_before(&ordinary, template, None);
    let c = apply(&mut dom, b);
    let inner = c.created(&inner).unwrap().unwrap();
    let ordinary = c.created(&ordinary).unwrap().unwrap();
    let old = dom.snapshot();
    fresh(&old, inner);
    assert!(!old.node(inner).unwrap().is_connected());
    assert_eq!(script(&old, ordinary).preparation_document(), Some(d));
    let mut b = DomBatch::new();
    let cloned = b.import_node(&old, template, d, true);
    b.insert_before(inner, root, None);
    let c = apply(&mut dom, b);
    let cloned = c.created(&cloned).unwrap().unwrap();
    let s = dom.snapshot();
    let cloned_contents = s.node(cloned).unwrap().as_template().unwrap().contents();
    let cloned_inner = s.node(cloned_contents).unwrap().children()[0];
    fresh(&s, cloned_inner);
    assert_eq!(s.node(cloned_inner).unwrap().owner_document(), Some(inert));
    assert_eq!(script(&s, inner).preparation_document(), Some(d));
    fresh(&old, inner);
    assert_eq!(
        dom.snapshot().document(inert).unwrap().template_role(),
        DomTemplateDocumentRole::InertTemplate
    );
}

#[test]
fn actual_shadow_node_document_controls_preparation_independently_of_host() {
    let mut dom = dom();
    let (d1, host) = setup(&mut dom);
    let mut b = DomBatch::new();
    let d2 = b.create_document();
    let root = b.attach_shadow(
        host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Named,
            false,
            false,
            false,
        ),
    );
    let fragment = b.create_fragment(&d2);
    let old = b.create_comment(&d2, "old");
    b.insert_before(&old, &fragment, None);
    let c = apply(&mut dom, b);
    let d2 = c.document(&d2).unwrap();
    let root = c.created(&root).unwrap().unwrap();
    let fragment = c.created(&fragment).unwrap().unwrap();
    let old = c.created(&old).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.replace_child(root, fragment, old);
    let node = b.create_element_ns(d2, Namespace::html(), "script");
    b.set_attribute(&node, "src", "");
    b.insert_before(&node, root, None);
    let c = apply(&mut dom, b);
    let node = c.created(&node).unwrap().unwrap();
    let s = dom.snapshot();
    assert!(s.node(node).unwrap().is_connected());
    assert_eq!(s.node(host).unwrap().owner_document(), Some(d1));
    assert_eq!(s.node(root).unwrap().owner_document(), Some(d2));
    assert_eq!(script(&s, node).preparation_document(), Some(d2));
}

#[test]
fn exact_storage_failure_cancellation_and_finite_history_preserve_script_state() {
    for units in [64, 65] {
        let mut dom = Dom::new(DomLimits::new(2, units, 8, 0, 0, 4, 8192).unwrap()).unwrap();
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let node = b.create_element_ns(&d, Namespace::html(), "script");
        let result = dom.apply(b, &AtomicBool::new(false));
        if units == 64 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 64
                }
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage().managed_nodes(), 0);
        } else {
            let c = result.unwrap();
            fresh(&dom.snapshot(), c.created(&node).unwrap().unwrap());
            assert_eq!(dom.usage().string_code_units(), 65); //31metadata+28namespace+6script
            assert_eq!(dom.usage().managed_nodes(), 2);
        }
    }
    for history in [0, 8] {
        let mut dom =
            Dom::new(DomLimits::new(16, 2048, 32, 0, 0, history, 65536).unwrap()).unwrap();
        let (d, root) = setup(&mut dom);
        let node = invalid_connected(&mut dom, d, root);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_attribute(node, "type", "module");
        b.set_attribute(node, "src", "same");
        b.insert_before(d, node, None);
        assert_eq!(
            dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::HierarchyRequest
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        fresh(&dom.snapshot(), node);
        let mut b = DomBatch::new();
        b.set_attribute(node, "src", "cancelled");
        assert_eq!(
            dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        let mut b = DomBatch::new();
        b.set_attribute(node, "type", "module");
        b.set_attribute(node, "src", "same");
        let c = apply(&mut dom, b);
        if history == 0 {
            assert!(matches!(
                dom.changes_since(before.stamp()),
                Err(DomHistoryError::FullRecomputeRequired { .. })
            ));
        } else {
            assert_eq!(
                dom.changes_since(before.stamp()).unwrap().changes(),
                c.changes().changes()
            );
        }
    }
}

#[test]
fn fragment_script_hooks_compose_with_real_slot_form_and_clonable_shadow_state() {
    let mut dom = dom();
    let (d, host) = setup(&mut dom);
    let mut b = DomBatch::new();
    let destination = b.create_document();
    let shadow = b.attach_shadow(
        host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Named,
            false,
            true,
            false,
        ),
    );
    let slot = b.create_element_ns(d, Namespace::html(), "slot");
    let form = b.create_element_ns(d, Namespace::html(), "form");
    let button = b.create_element_ns(d, Namespace::html(), "button");
    let outer = b.create_element_ns(d, Namespace::html(), "script");
    b.insert_before(&slot, &shadow, None);
    b.insert_before(&button, &form, None);
    b.insert_before(&outer, &form, None);
    b.insert_before(&form, host, None);
    let c = apply(&mut dom, b);
    let destination = c.document(&destination).unwrap();
    let form = c.created(&form).unwrap().unwrap();
    let button = c.created(&button).unwrap().unwrap();
    let outer = c.created(&outer).unwrap().unwrap();
    let slot = c.created(&slot).unwrap().unwrap();
    fresh(&dom.snapshot(), outer);
    let mut b = DomBatch::new();
    let fragment = b.create_fragment(d);
    let start = b.create_text(d, " ");
    let inner = b.create_element_ns(d, Namespace::html(), "script");
    b.set_attribute(&inner, "type", "module");
    b.set_attribute(&inner, "src", "");
    let end = b.create_text(d, "\n");
    for child in [&start, &inner, &end] {
        b.insert_before(child, &fragment, None);
    }
    b.insert_before(&fragment, outer, None);
    let c = apply(&mut dom, b);
    let inner = c.created(&inner).unwrap().unwrap();
    let s = dom.snapshot();
    assert_eq!(
        s.node(outer).unwrap().children(),
        &[
            c.created(&start).unwrap().unwrap(),
            inner,
            c.created(&end).unwrap().unwrap()
        ]
    );
    assert_eq!(script(&s, outer).script_type(), Some(Kind::Classic));
    assert_eq!(script(&s, inner).script_type(), Some(Kind::Module));
    for node in [outer, inner] {
        assert_eq!(script(&s, node).preparation_document(), Some(d));
    }
    assert_eq!(s.slot(slot).unwrap().assigned_nodes(), &[form]);
    assert_eq!(
        s.node(button).unwrap().as_button().unwrap().form_owner(),
        Some(form)
    );
    assert_eq!(
        s.node(form).unwrap().as_form().unwrap().past_names().len(),
        0
    );
    let mut b = DomBatch::new();
    let cloned = b.import_node(&s, host, d, true);
    b.adopt(form, destination);
    let c = apply(&mut dom, b);
    let cloned = c.created(&cloned).unwrap().unwrap();
    let current = dom.snapshot();
    let cloned_host = current.node(cloned).unwrap();
    let cloned_form = cloned_host.children()[0];
    let cloned_children = current.node(cloned_form).unwrap().children();
    let cloned_outer = cloned_children[1];
    let cloned_inner = current.node(cloned_outer).unwrap().children()[1];
    for node in [cloned_outer, cloned_inner] {
        let state = script(&current, node);
        assert!(state.flags().already_started());
        assert_eq!(state.preparation_document(), None);
        assert_eq!(state.script_type(), None);
    }
    assert_eq!(
        current
            .node(cloned_children[0])
            .unwrap()
            .as_button()
            .unwrap()
            .form_owner(),
        Some(cloned_form)
    );
    assert_eq!(
        current
            .node(cloned_form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        0
    );
    let cloned_slot = current
        .node(cloned_host.hosted_shadow_root().unwrap())
        .unwrap()
        .children()[0];
    assert_eq!(
        current.slot(cloned_slot).unwrap().assigned_nodes(),
        &[cloned_form]
    );
    assert!(current.slot(slot).unwrap().assigned_nodes().is_empty());
    for node in [outer, inner] {
        assert_eq!(
            current.node(node).unwrap().owner_document(),
            Some(destination)
        );
        assert_eq!(script(&current, node).preparation_document(), Some(d));
    }
    assert_eq!(
        current
            .node(button)
            .unwrap()
            .as_button()
            .unwrap()
            .form_owner(),
        Some(form)
    );
}

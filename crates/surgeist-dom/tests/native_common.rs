//! Source oracles: DOM3071 §1.4/createElementNS; HTML877d native common
//! initialization and §2.5.6 nonce attribute/IDL/insertion/cloning clauses.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentString, Dom, DomBatch, DomChange, DomError, DomLimits, DomResource,
    HtmlCustomElementState, Namespace,
};
use surgeist_html::{
    HtmlBrowsingContextPresence, HtmlNativeInitializationRequirement as Requirement,
    KnownHtmlElement,
};

const HTML: &str = "http://www.w3.org/1999/xhtml";
const SVG: &str = "http://www.w3.org/2000/svg";
const MATH: &str = "http://www.w3.org/1998/Math/MathML";
fn dom() -> Dom {
    Dom::new(DomLimits::new(128, 16384, 256, 0, 0, 16, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

#[test]
fn exact_expanded_names_select_common_state_without_parser_folding() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let cases = [
        (HTML, "p:div", true, true),
        (HTML, "Input", true, true),
        (HTML, "x-widget", true, true),
        (HTML, "X-widget", true, true),
        (HTML, "annotation-xml", true, true),
        (SVG, "input", false, true),
        (SVG, "Script", false, true),
        (MATH, "template", false, true),
        ("urn:example", "input", false, false),
        ("http://www.w3.org/1999/XHTML", "input", false, false),
    ];
    let tickets: Vec<_> = cases
        .iter()
        .map(|(ns, name, _, _)| batch.create_element_ns(&document, Namespace::from_uri(*ns), *name))
        .collect();
    let commit = apply(&mut dom, batch);
    let capture = dom.snapshot();
    let environment = capture
        .document(commit.document(&document).unwrap())
        .unwrap()
        .metadata()
        .native_environment();
    assert_eq!(
        environment.browsing_context(),
        HtmlBrowsingContextPresence::Absent
    );
    assert!(!environment.has_ever_had_navigable());
    assert!(!environment.is_fully_active());
    for ((ns, name, html, nonce), ticket) in cases.iter().zip(&tickets) {
        let node = capture
            .node(commit.created(ticket).unwrap().unwrap())
            .unwrap();
        let element = node.as_element().unwrap();
        assert_eq!(node.node_name(), DocumentString::from(*name));
        assert_eq!(element.name().namespace(), &Namespace::from_uri(*ns));
        assert_eq!(element.common_state().html().is_some(), *html);
        assert_eq!(
            element.nonce(),
            nonce.then(|| DocumentString::from("")).as_ref()
        );
        let common = element.common_state().element();
        assert!(common.reaction_queue_is_empty());
        assert!(!common.click_in_progress());
        if let Some(common) = element.common_state().html() {
            assert!(!common.has_previously_focused_element());
            assert!(!common.has_attached_internals());
            assert!(common.popover_is_hidden());
            assert!(!common.has_popover_trigger());
            assert!(!common.popover_is_hiding());
            assert!(!common.has_popover_toggle_task_tracker());
            assert!(!common.has_popover_close_watcher());
            assert!(!common.has_popover_opened_in_mode());
        }
    }
    let prefixed = capture
        .node(commit.created(&tickets[0]).unwrap().unwrap())
        .unwrap();
    assert_eq!(
        prefixed.as_element().unwrap().name().prefix(),
        Some(&"p".into())
    );
    assert_eq!(
        prefixed.as_element().unwrap().name().local_name(),
        &"div".into()
    );
    assert_eq!(
        capture
            .node(commit.created(&tickets[2]).unwrap().unwrap())
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_state(),
        HtmlCustomElementState::Undefined
    );
    assert_eq!(
        capture
            .node(commit.created(&tickets[5]).unwrap().unwrap())
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_state(),
        HtmlCustomElementState::Uncustomized
    );
    for index in [3, 4] {
        assert_eq!(
            capture
                .node(commit.created(&tickets[index]).unwrap().unwrap())
                .unwrap()
                .as_element()
                .unwrap()
                .custom_element_state(),
            HtmlCustomElementState::Uncustomized
        );
    }
}

#[test]
fn namespace_element_validation_uses_element_context_and_preserves_utf16() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let unusual = DocumentString::from_utf16(vec![0x70, 0x3a, 0x61, 0xd800, 0x3d]);
    let element = batch.create_element_ns(&document, Namespace::from_uri("urn:x"), unusual.clone());
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    assert_eq!(dom.snapshot().node(element).unwrap().node_name(), unusual);
    for (ns, name, error) in [
        (Namespace::none(), "p:1", DomError::InvalidCharacter),
        (Namespace::none(), "p:a", DomError::Namespace),
        (
            Namespace::from_uri("urn:x"),
            ":a",
            DomError::InvalidCharacter,
        ),
        (Namespace::from_uri("urn:x"), "xml:a", DomError::Namespace),
        (Namespace::from_uri("urn:x"), "xmlns", DomError::Namespace),
    ] {
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.create_element_ns(document, ns, name);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(element, Namespace::from_uri("urn:x"), "p:a=", "x");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidCharacter
    );
}

#[test]
fn unmet_builtin_profiles_reject_the_whole_batch_with_the_actual_requirement() {
    let mut dom = dom();
    for (ns, name, requirement) in [
        (
            HTML,
            "audio",
            Requirement::KnownHtml(KnownHtmlElement::Audio),
        ),
        (SVG, "script", Requirement::SvgScript),
    ] {
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        batch.create_element(&document, "not-published");
        batch.create_element_ns(&document, Namespace::from_uri(ns), name);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::NativeInitializationRequired(requirement)
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage().managed_nodes(), 0);
    }
}

#[test]
fn nonce_content_and_idl_slot_change_independently_with_precise_change_domains() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    batch.set_attribute(&element, "nonce", "content");
    let commit = apply(&mut dom, batch);
    let element = commit.created(&element).unwrap().unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_nonce(element, "slot");
    let changed = apply(&mut dom, batch);
    assert_eq!(
        changed.changes().changes(),
        &[DomChange::ElementState(element)]
    );
    let capture = dom.snapshot();
    let current = capture.node(element).unwrap().as_element().unwrap();
    assert_eq!(current.nonce(), Some(&"slot".into()));
    assert_eq!(current.attribute(&"nonce".into()), Some(&"content".into()));
    assert_eq!(
        old.node(element).unwrap().as_element().unwrap().nonce(),
        Some(&"content".into())
    );
    let stamp = capture.stamp();
    let mut batch = DomBatch::new();
    batch.set_nonce(element, "slot");
    apply(&mut dom, batch);
    assert_eq!(dom.snapshot().stamp(), stamp);
    // Setting the same content still executes nonce change steps, resetting a
    // diverged slot even though the attribute's final value does not change.
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "nonce", "content");
    let changed = apply(&mut dom, batch);
    assert_eq!(
        changed.changes().changes(),
        &[DomChange::ElementState(element)]
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "Nonce", "case");
    batch.set_attribute_ns(
        element,
        Namespace::from_uri("urn:x"),
        "p:nonce",
        "namespace",
    );
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .nonce(),
        Some(&"content".into())
    );
    let mut batch = DomBatch::new();
    batch.remove_attribute(element, "nonce");
    let changed = apply(&mut dom, batch);
    assert!(
        changed
            .changes()
            .changes()
            .contains(&DomChange::Attributes(element))
    );
    assert!(
        changed
            .changes()
            .changes()
            .contains(&DomChange::ElementState(element))
    );
    assert_eq!(
        dom.snapshot()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .nonce(),
        Some(&"".into())
    );
}

#[test]
fn generic_nonce_content_is_ordinary_and_has_no_cryptographic_slot() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element(&document, "input");
    batch.set_attribute(&element, "nonce", "ordinary");
    let commit = apply(&mut dom, batch);
    let element = commit.created(&element).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .nonce(),
        None
    );
    let mut batch = DomBatch::new();
    batch.set_nonce(element, "unsupported");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::NonceInapplicable
    );
}

#[test]
fn captured_import_copies_nonce_after_attributes_and_native_insertion_keeps_content() {
    let mut source = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    let child = batch.create_element_ns(&document, Namespace::from_uri(SVG), "g");
    batch.set_attribute(&child, "nonce", "public");
    batch.set_nonce(&child, "private");
    batch.insert_before(&child, &parent, None);
    batch.insert_before(&parent, &document, None);
    let commit = apply(&mut source, batch);
    let parent = commit.created(&parent).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let captured = source.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(parent);
    batch.destroy(parent);
    apply(&mut source, batch);
    drop(source);
    let mut target = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let imported = batch.import_node(&captured, parent, &document, true);
    batch.insert_before(&imported, &document, None);
    let commit = apply(&mut target, batch);
    let document = commit.document(&document).unwrap();
    let imported = commit.created(&imported).unwrap().unwrap();
    let capture = target.snapshot();
    let copied_child = capture.node(imported).unwrap().children()[0];
    assert_ne!(copied_child, child);
    let element = capture.node(copied_child).unwrap().as_element().unwrap();
    assert_eq!(element.nonce(), Some(&"private".into()));
    assert_eq!(element.attribute(&"nonce".into()), Some(&"public".into()));
    assert_eq!(
        capture.node(copied_child).unwrap().owner_document(),
        Some(document)
    );
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    batch.adopt(imported, &other);
    batch.insert_before(imported, &other, None);
    let commit = apply(&mut target, batch);
    let capture = target.snapshot();
    assert_eq!(
        capture.node(copied_child).unwrap().owner_document(),
        Some(commit.document(&other).unwrap())
    );
    assert_eq!(
        capture
            .node(copied_child)
            .unwrap()
            .as_element()
            .unwrap()
            .nonce(),
        Some(&"private".into())
    );
}

#[test]
fn exact_nonce_string_capacity_counts_both_content_and_slot_and_rolls_back_late_failure() {
    // 31 document + 28 HTML URI + 3 div + 5 nonce + 2 content + 2 slot = 71.
    for capacity in [70, 71] {
        let mut dom = Dom::new(DomLimits::new(2, capacity, 16, 0, 0, 0, 0).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let element = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
        batch.set_attribute(&element, "nonce", "xy");
        let result = dom.apply(batch, &AtomicBool::new(false));
        if capacity == 70 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 70
                }
            );
            assert_eq!(dom.usage().managed_nodes(), 0);
        } else {
            let commit = result.unwrap();
            let document = commit.document(&document).unwrap();
            let element = commit.created(&element).unwrap().unwrap();
            assert_eq!(dom.usage().string_code_units(), 71);
            let before = dom.snapshot();
            let mut batch = DomBatch::new();
            batch.set_nonce(element, "z");
            batch.insert_before(document, element, None);
            assert_eq!(
                dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
                DomError::HierarchyRequest
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage().string_code_units(), 71);
            assert_eq!(
                dom.snapshot()
                    .node(element)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .nonce(),
                Some(&"xy".into())
            );
            let mut batch = DomBatch::new();
            batch.set_nonce(element, "long");
            assert_eq!(
                dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 71
                }
            );
        }
    }
}

#[test]
fn common_state_survives_character_data_replacement_fragment_moves_and_child_replacement() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    let old_child = batch.create_element_ns(&document, Namespace::from_uri(MATH), "input");
    let new_child = batch.create_element_ns(&document, Namespace::from_uri(HTML), "span");
    let text = batch.create_text(&document, "old");
    let fragment = batch.create_fragment(&document);
    let nonce = DocumentString::from_utf16(vec![0xd800, 0x41, 0xdc00]);
    batch.set_attribute(&new_child, "nonce", nonce.clone());
    batch.set_attribute(&new_child, "popover", "auto");
    batch.set_attribute(&new_child, "autofocus", "");
    batch.set_attribute(&new_child, "is", "x-later");
    batch.insert_before(&text, &new_child, None);
    batch.insert_before(&old_child, &root, None);
    batch.insert_before(&new_child, &fragment, None);
    batch.insert_before(&root, &document, None);
    let commit = apply(&mut dom, batch);
    let root = commit.created(&root).unwrap().unwrap();
    let old_child = commit.created(&old_child).unwrap().unwrap();
    let new_child = commit.created(&new_child).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_data(text, 0, 3, "new");
    batch.replace_child(fragment, root, old_child);
    batch.remove_attribute(new_child, "popover");
    apply(&mut dom, batch);
    let capture = dom.snapshot();
    assert_eq!(capture.node(root).unwrap().children(), &[new_child]);
    assert!(capture.node(fragment).unwrap().children().is_empty());
    assert_eq!(capture.node(old_child).unwrap().parent(), None);
    let element = capture.node(new_child).unwrap().as_element().unwrap();
    assert_eq!(element.nonce(), Some(&nonce));
    assert_eq!(element.attribute(&"nonce".into()), Some(&nonce));
    assert_eq!(element.saved_is(), None);
    assert_eq!(
        element.custom_element_state(),
        HtmlCustomElementState::Uncustomized
    );
    assert!(element.common_state().html().unwrap().popover_is_hidden());
    assert_eq!(old.node(root).unwrap().children(), &[old_child]);
    assert_eq!(old.node(new_child).unwrap().parent(), Some(fragment));
}

#[test]
fn state_only_edits_have_dom_history_while_equal_host_facts_keep_their_revision() {
    use surgeist_dom::{DomHostDomains, DomHostObservations, DomHostPublisher, Observed};
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let before = dom.snapshot();
    let mut host = DomHostPublisher::new(DomHostDomains::Focus);
    let view = before.document(document).unwrap();
    let first = host
        .publish(
            view,
            view.stamp(),
            DomHostObservations::Focus(Observed::Supported(None)),
        )
        .unwrap();
    let mut batch = DomBatch::new();
    batch.set_nonce(element, "state");
    let commit = apply(&mut dom, batch);
    assert_eq!(
        dom.changes_since(before.stamp()).unwrap().changes(),
        &[DomChange::ElementState(element)]
    );
    assert_eq!(
        commit.changes().changes(),
        &[DomChange::ElementState(element)]
    );
    let capture = dom.snapshot();
    let view = capture.document(document).unwrap();
    let next = host
        .publish(
            view,
            view.stamp(),
            DomHostObservations::Focus(Observed::Supported(None)),
        )
        .unwrap();
    assert_eq!(
        first.capture().stamp().host_revision(),
        next.capture().stamp().host_revision()
    );
    assert_ne!(
        first.capture().stamp().document(),
        next.capture().stamp().document()
    );
}

//! HTML877d update-a-style-block guards and DOM immediate-parent triggers.
//! These exercise the ordinary DOM boundary before inline CSP composition.

use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlStyleUpdateAction;

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            256,
            65536,
            1024,
            1024,
            512,
            16,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

fn requires_csp(dom: &mut Dom, batch: DomBatch) {
    assert!(matches!(
        dom.apply(batch, &AtomicBool::new(false)),
        Err(DomError::StyleCompositionRequired(
            HtmlStyleUpdateAction::CheckInlineCsp
        ))
    ));
}

#[test]
fn connected_type_guard_uses_exact_null_namespace_value_and_ascii_case() {
    // HTML877d steps 3–5: no trimming or MIME-parameter interpretation.
    for (value, eligible) in [
        (None, true),
        (Some(""), true),
        (Some("text/css"), true),
        (Some("TeXt/CsS"), true),
        (Some(" text/css"), false),
        (Some("text/css "), false),
        (Some("text/css; charset=utf-8"), false),
        (Some("text/plain"), false),
    ] {
        let mut dom = owner();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let style = batch.create_element_ns(&document, Namespace::html(), "style");
        if let Some(value) = value {
            batch.set_attribute(&style, "type", value);
        }
        // A lookalike in another namespace cannot override the real type.
        batch.set_attribute_ns(
            &style,
            Namespace::from_uri("urn:style-test"),
            "x:type",
            "text/plain",
        );
        let commit = apply(&mut dom, batch);
        let document = commit.document(&document).unwrap();
        let style = commit.created(&style).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.insert_before(style, document, None);
        if eligible {
            requires_csp(&mut dom, batch);
            assert_eq!(dom.snapshot().stamp(), before.stamp(), "{value:?}");
            assert_eq!(dom.snapshot().node(style).unwrap().parent(), None);
        } else {
            apply(&mut dom, batch);
            assert_eq!(
                dom.snapshot().node(style).unwrap().parent(),
                Some(document.node()),
                "{value:?}"
            );
        }
    }
}

fn connected_ineligible() -> (Dom, DocumentId, NodeId) {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    batch.set_attribute(&style, "type", "text/plain");
    batch.insert_before(&style, &document, None);
    let commit = apply(&mut dom, batch);
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&style).unwrap().unwrap(),
    )
}

#[test]
fn changing_type_alone_does_not_run_the_style_update_algorithm() {
    let (mut dom, document, style) = connected_ineligible();
    for value in ["text/css", "text/plain", "TeXt/CsS", ""] {
        let mut batch = DomBatch::new();
        batch.set_attribute(style, "type", value);
        apply(&mut dom, batch);
        assert_eq!(
            dom.snapshot()
                .node(style)
                .unwrap()
                .attribute(&DocumentString::from("type")),
            Some(&DocumentString::from(value))
        );
    }
    let mut batch = DomBatch::new();
    batch.remove_attribute(style, "type");
    apply(&mut dom, batch);
    let mut batch = DomBatch::new();
    let comment = batch.create_comment(document, "not CSS text");
    batch.insert_before(&comment, style, None);
    requires_csp(&mut dom, batch);

    // The disconnected guard precedes type/CSP, even for eligible content.
    let mut batch = DomBatch::new();
    batch.detach(style);
    let text = batch.create_text(document, "body { color: red }");
    batch.insert_before(&text, style, None);
    apply(&mut dom, batch);
    assert_eq!(dom.snapshot().node(style).unwrap().parent(), None);
}

#[test]
fn character_data_triggers_only_its_immediate_parent_including_comments() {
    let (mut dom, document, style) = connected_ineligible();
    let mut batch = DomBatch::new();
    let wrapper = batch.create_element_ns(document, Namespace::html(), "span");
    let nested_text = batch.create_text(document, "nested");
    let nested_comment = batch.create_comment(document, "nested comment");
    let direct_text = batch.create_text(document, "direct");
    let direct_comment = batch.create_comment(document, "direct comment");
    batch.insert_before(&nested_text, &wrapper, None);
    batch.insert_before(&nested_comment, &wrapper, None);
    batch.insert_before(&wrapper, style, None);
    batch.insert_before(&direct_text, style, None);
    batch.insert_before(&direct_comment, style, None);
    let commit = apply(&mut dom, batch);
    let nested_text = commit.created(&nested_text).unwrap().unwrap();
    let nested_comment = commit.created(&nested_comment).unwrap().unwrap();
    let direct_text = commit.created(&direct_text).unwrap().unwrap();
    let direct_comment = commit.created(&direct_comment).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(style, "type", "text/css");
    batch.replace_data(nested_text, 0, usize::MAX, "changed nested text");
    batch.replace_data(nested_comment, 0, usize::MAX, "changed nested comment");
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot().node(nested_text).unwrap().data(),
        Some(&DocumentString::from("changed nested text"))
    );
    // DOM replace-data step 13 runs the immediate parent's children-changed
    // steps. Comments are excluded from child text content, not this trigger.
    for child in [direct_text, direct_comment] {
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.replace_data(child, 0, usize::MAX, "changed direct child");
        requires_csp(&mut dom, batch);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            dom.snapshot().node(child).unwrap().data(),
            before.node(child).unwrap().data()
        );
    }
}

#[test]
fn late_style_refusal_discards_prior_image_update_structure_and_allocations() {
    let (mut dom, document, style) = connected_ineligible();
    let mut batch = DomBatch::new();
    let image = batch.create_element_ns(document, Namespace::html(), "img");
    batch.set_attribute(&image, "src", "one.png");
    let commit = apply(&mut dom, batch);
    let image = commit.created(&image).unwrap().unwrap();
    let before = dom.snapshot();
    let previous_update = before
        .node(image)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap();
    let mut batch = DomBatch::new();
    // Equal src restarts the existing Img wait before the later refusal.
    batch.set_attribute(image, "src", "one.png");
    let wrapper = batch.create_element_ns(document, Namespace::html(), "div");
    batch.insert_before(image, &wrapper, None);
    batch.set_attribute(style, "type", "text/css");
    let comment = batch.create_comment(document, "late trigger");
    batch.insert_before(&comment, style, None);
    requires_csp(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.nodes().collect::<Vec<_>>(),
        before.nodes().collect::<Vec<_>>()
    );
    assert_eq!(after.node(image).unwrap().parent(), None);
    assert!(after.node(style).unwrap().children().is_empty());
    assert_eq!(
        after
            .node(style)
            .unwrap()
            .attribute(&DocumentString::from("type")),
        Some(&DocumentString::from("text/plain"))
    );
    let retained_update = after
        .node(image)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap();
    assert_eq!(retained_update.id(), previous_update.id());
    assert_eq!(retained_update.flags(), previous_update.flags());
    let changes = dom.changes_since(before.stamp()).unwrap();
    assert_eq!(changes.from(), before.stamp());
    assert_eq!(changes.to(), before.stamp());
    assert!(changes.changes().is_empty());
}

//! HTML877d local control rules consume canonical policy after real teardown.
//!
//! Literal expectations come from the retained forms selection/value/reset,
//! Dialog inactive-hook, and standalone Img updating-image-data clauses. These
//! tests use no interactive selection, form-wide reset, URL or resource service.
use super::support::{HostedFixture, cancel};
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{
    HtmlDialogAction, HtmlImageUpdateFlags, HtmlImgFreshRequestState,
    HtmlInputEmptySelectionSupport as Empty, HtmlInputImageHistory,
    HtmlInputNoneDirectionSupport as NoneDirection, HtmlInputSelectionDirection as Direction,
    HtmlInputType,
};

#[derive(Clone, Copy, Debug)]
enum Destination {
    NeverHosted,
    Active,
    Retired,
}

fn apply(fixture: &mut HostedFixture, batch: DomBatch) -> DomCommit {
    fixture.session.dom().apply(batch, &cancel()).unwrap()
}

fn create(fixture: &mut HostedFixture, document: DocumentId, name: &str) -> NodeId {
    let mut batch = DomBatch::new();
    let node = batch.create_element_ns(document, Namespace::html(), name);
    apply(fixture, batch).created(&node).unwrap().unwrap()
}

fn document(fixture: &HostedFixture, destination: Destination) -> DocumentId {
    match destination {
        Destination::NeverHosted => fixture.closed,
        Destination::Active | Destination::Retired => fixture.top.document(),
    }
}

fn finish_setup(fixture: &mut HostedFixture, destination: Destination) {
    if matches!(destination, Destination::Retired) {
        fixture.retire();
        fixture.assert_retired();
    }
}

fn input(snapshot: &DomSnapshot, node: NodeId) -> &DomInputState {
    snapshot.node(node).unwrap().as_input().unwrap().state()
}

fn input_value(snapshot: &DomSnapshot, node: NodeId) -> Vec<u16> {
    snapshot
        .node(node)
        .unwrap()
        .as_input()
        .unwrap()
        .value()
        .to_vec()
}

fn textarea(snapshot: &DomSnapshot, node: NodeId) -> &DomTextareaState {
    snapshot.node(node).unwrap().as_textarea().unwrap().state()
}

fn textarea_value(snapshot: &DomSnapshot, node: NodeId) -> DocumentString {
    snapshot
        .node(node)
        .unwrap()
        .as_textarea()
        .unwrap()
        .value(DomTextareaReadLimits::new(1_000_000, 4_000_000), &cancel())
        .unwrap()
}

fn textarea_default(snapshot: &DomSnapshot, node: NodeId) -> DocumentString {
    snapshot
        .node(node)
        .unwrap()
        .as_textarea()
        .unwrap()
        .default_value(DomTextareaReadLimits::new(1_000_000, 4_000_000), &cancel())
        .unwrap()
}

fn expected_direction(none: NoneDirection) -> Direction {
    match none {
        NoneDirection::Supported => Direction::None,
        NoneDirection::Unsupported => Direction::Forward,
    }
}

#[test]
fn existing_input_uses_fixed_platform_for_utf16_value_reset_and_type_after_teardown() {
    // HTML Text sanitation removes CR/LF. Cursor positions count UTF-16 units;
    // reset uses the default, clears dirty flags and clamps the existing cursor.
    for empty in [Empty::Supported, Empty::Unsupported] {
        for none in [NoneDirection::Supported, NoneDirection::Unsupported] {
            for destination in [
                Destination::NeverHosted,
                Destination::Active,
                Destination::Retired,
            ] {
                let platform = DomNativeInputPlatform::new(empty, none);
                let mut fixture = HostedFixture::active_with_platform(platform);
                let target_document = document(&fixture, destination);
                let node = create(&mut fixture, target_document, "input");
                let mut batch = DomBatch::new();
                batch.set_attribute(node, "value", "Z\r\n");
                batch.set_attribute(node, "checked", "");
                apply(&mut fixture, batch);
                finish_setup(&mut fixture, destination);
                let old = fixture.session.snapshot();
                assert_eq!(input_value(&old, node), [90]);
                assert!(!input(&old, node).flags().dirty_value());
                assert_eq!(
                    input(&old, node).selection().unwrap().cursor_position(),
                    Some(0)
                );
                assert_eq!(
                    input(&old, node).selection().unwrap().direction(),
                    expected_direction(none)
                );

                let mut batch = DomBatch::new();
                batch.set_input_value(
                    node,
                    DocumentString::from_utf16(vec![65, 0xd83d, 0xde00, 13, 10, 66]),
                );
                batch.set_input_checked(node, false);
                batch.set_input_indeterminate(node, true);
                batch.set_custom_validity(node, "bad\r\nmessage");
                apply(&mut fixture, batch);
                let dirty = fixture.session.snapshot();
                assert_eq!(input_value(&dirty, node), [65, 0xd83d, 0xde00, 66]);
                assert!(input(&dirty, node).flags().dirty_value());
                assert!(input(&dirty, node).flags().dirty_checkedness());
                assert!(!input(&dirty, node).flags().checkedness());
                assert!(input(&dirty, node).flags().indeterminate());
                assert_eq!(
                    input(&dirty, node).selection().unwrap().cursor_position(),
                    Some(4)
                );
                assert_eq!(
                    input(&dirty, node).selection().unwrap().direction(),
                    expected_direction(none)
                );
                assert_ne!(dirty.stamp(), old.stamp());

                let mut batch = DomBatch::new();
                batch.reset_input(node);
                apply(&mut fixture, batch);
                let reset = fixture.session.snapshot();
                assert_eq!(input_value(&reset, node), [90]);
                assert!(!input(&reset, node).flags().dirty_value());
                assert!(!input(&reset, node).flags().dirty_checkedness());
                assert!(input(&reset, node).flags().checkedness());
                assert!(input(&reset, node).flags().indeterminate());
                assert!(!input(&reset, node).flags().user_validity());
                assert_eq!(
                    input(&reset, node).selection().unwrap().cursor_position(),
                    Some(1)
                );
                assert_eq!(
                    input(&reset, node).selection().unwrap().direction(),
                    expected_direction(none)
                );
                assert_eq!(
                    input(&reset, node).custom_validity_message(),
                    &DocumentString::from("bad\nmessage")
                );

                let mut batch = DomBatch::new();
                batch.set_attribute(node, "type", "checkbox");
                batch.remove_attribute(node, "value");
                apply(&mut fixture, batch);
                let checkbox = fixture.session.snapshot();
                assert_eq!(input_value(&checkbox, node), [111, 110]);
                assert_eq!(input(&checkbox, node).selection(), None);
                let mut batch = DomBatch::new();
                batch.set_attribute(node, "value", "a\r\nb");
                batch.set_attribute(node, "type", "text");
                apply(&mut fixture, batch);
                let current = fixture.session.snapshot();
                assert_eq!(input_value(&current, node), [97, 98]);
                assert_eq!(input(&current, node).input_type(), HtmlInputType::Text);
                assert!(!input(&current, node).flags().dirty_value());
                assert_eq!(
                    input(&current, node).selection().unwrap().cursor_position(),
                    Some(0)
                );
                assert_eq!(
                    input(&current, node).selection().unwrap().direction(),
                    expected_direction(none)
                );
                assert_eq!(
                    current.node(node).unwrap().owner_document(),
                    Some(target_document)
                );
                assert_eq!(
                    current
                        .document(target_document)
                        .unwrap()
                        .metadata()
                        .document_environment()
                        .input_platform(),
                    platform
                );
                assert_eq!(input_value(&old, node), [90]);
                assert_eq!(input_value(&dirty, node), [65, 0xd83d, 0xde00, 66]);
            }
        }
    }
}

#[test]
fn input_image_disabled_history_observes_absent_empty_and_changed_reentry_after_teardown() {
    // Image Button's explicit disabled-images guard precedes URL/request work;
    // it is separate from standalone Img's not-fully-active wait.
    for destination in [
        Destination::NeverHosted,
        Destination::Active,
        Destination::Retired,
    ] {
        let mut fixture = HostedFixture::active();
        let target_document = document(&fixture, destination);
        let node = create(&mut fixture, target_document, "input");
        finish_setup(&mut fixture, destination);
        let mut batch = DomBatch::new();
        batch.set_attribute(node, "type", "image");
        apply(&mut fixture, batch);
        let absent = fixture.session.snapshot();
        assert!(matches!(
            input(&absent, node).image_history(),
            HtmlInputImageHistory::Last { source: None, .. }
        ));
        let mut batch = DomBatch::new();
        batch.set_attribute(node, "src", "");
        apply(&mut fixture, batch);
        let empty = fixture.session.snapshot();
        assert!(
            matches!(input(&empty, node).image_history(), HtmlInputImageHistory::Last { source: Some(raw), .. } if raw.is_empty())
        );
        let mut batch = DomBatch::new();
        batch.set_attribute(node, "src", "unparsed image source");
        apply(&mut fixture, batch);
        let observed = fixture.session.snapshot();
        assert!(
            matches!(input(&observed, node).image_history(), HtmlInputImageHistory::Last { source: Some(raw), .. } if raw == &DocumentString::from("unparsed image source"))
        );
        let mut batch = DomBatch::new();
        batch.set_attribute(node, "src", "unparsed image source");
        let unchanged = apply(&mut fixture, batch);
        assert!(unchanged.changes().changes().is_empty());
        assert_eq!(fixture.session.snapshot().stamp(), observed.stamp());
        let mut batch = DomBatch::new();
        batch.set_attribute(node, "type", "text");
        batch.set_attribute(node, "src", "different source");
        apply(&mut fixture, batch);
        let not_image = fixture.session.snapshot();
        assert!(
            matches!(input(&not_image, node).image_history(), HtmlInputImageHistory::Last { source: Some(raw), .. } if raw == &DocumentString::from("unparsed image source"))
        );
        let mut batch = DomBatch::new();
        batch.set_attribute(node, "type", "image");
        apply(&mut fixture, batch);
        let current = fixture.session.snapshot();
        assert!(
            matches!(input(&current, node).image_history(), HtmlInputImageHistory::Last { source: Some(raw), .. } if raw == &DocumentString::from("different source"))
        );
        assert_eq!(input(&current, node).image_history().coordinate().x(), 0);
        assert_eq!(input(&current, node).image_history().coordinate().y(), 0);
        let mut batch = DomBatch::new();
        batch.set_attribute(node, "type", "text");
        batch.set_attribute(node, "type", "image");
        apply(&mut fixture, batch);
        let unchanged_source = fixture.session.snapshot();
        assert_eq!(
            input(&unchanged_source, node).image_history(),
            input(&current, node).image_history()
        );
        let mut batch = DomBatch::new();
        batch.remove_attribute(node, "src");
        apply(&mut fixture, batch);
        assert!(matches!(
            input(&fixture.session.snapshot(), node).image_history(),
            HtmlInputImageHistory::Last { source: None, .. }
        ));
        assert_eq!(
            current
                .document(target_document)
                .unwrap()
                .metadata()
                .document_environment()
                .input_images(),
            surgeist_html::HtmlInputImagesDisabled::ExplicitlyDisabled
        );
        assert_eq!(current.incomplete_source_operation(), None);
        assert_eq!(
            fixture.session.details_execution().pending_tasks().unwrap(),
            0
        );
        assert!(matches!(
            input(&absent, node).image_history(),
            HtmlInputImageHistory::Last { source: None, .. }
        ));
    }
}

#[test]
fn textarea_dirty_default_children_and_local_reset_keep_utf16_and_platform_after_teardown() {
    // Child default is direct Text concatenation; API value normalizes CRLF
    // across Text boundaries. Dirty current value ignores later child changes.
    for empty in [Empty::Supported, Empty::Unsupported] {
        for none in [NoneDirection::Supported, NoneDirection::Unsupported] {
            for destination in [
                Destination::NeverHosted,
                Destination::Active,
                Destination::Retired,
            ] {
                let platform = DomNativeInputPlatform::new(empty, none);
                let mut fixture = HostedFixture::active_with_platform(platform);
                let target_document = document(&fixture, destination);
                let node = create(&mut fixture, target_document, "textarea");
                let mut batch = DomBatch::new();
                let first = batch.create_text(target_document, "A\r");
                let last = batch.create_text(target_document, "\nB");
                batch.insert_before(&first, node, None);
                batch.insert_before(&last, node, None);
                let commit = apply(&mut fixture, batch);
                let first = commit.created(&first).unwrap().unwrap();
                finish_setup(&mut fixture, destination);
                let old = fixture.session.snapshot();
                assert_eq!(textarea_default(&old, node), "A\r\nB".into());
                assert_eq!(textarea_value(&old, node), "A\nB".into());
                assert!(!textarea(&old, node).flags().dirty_value());
                let mut batch = DomBatch::new();
                batch.set_textarea_value(
                    node,
                    DocumentString::from_utf16(vec![88, 0xd83d, 0xde00, 13, 10, 89]),
                );
                batch.set_custom_validity(node, "kept\r\nmessage");
                batch.replace_data(first, 0, usize::MAX, "D\r");
                apply(&mut fixture, batch);
                let dirty = fixture.session.snapshot();
                assert_eq!(
                    textarea_value(&dirty, node).code_units(),
                    [88, 0xd83d, 0xde00, 10, 89]
                );
                assert_eq!(
                    textarea(&dirty, node).raw_value().code_units(),
                    [88, 0xd83d, 0xde00, 13, 10, 89]
                );
                assert_eq!(textarea_default(&dirty, node), "D\r\nB".into());
                assert!(textarea(&dirty, node).flags().dirty_value());
                assert_eq!(
                    textarea(&dirty, node).selection().cursor_position(),
                    Some(5)
                );
                assert_eq!(
                    textarea(&dirty, node).selection().direction(),
                    expected_direction(none)
                );
                let mut batch = DomBatch::new();
                batch.reset_textarea(node);
                apply(&mut fixture, batch);
                let reset = fixture.session.snapshot();
                assert_eq!(
                    textarea(&reset, node).raw_value(),
                    &DocumentString::from("D\r\nB")
                );
                assert_eq!(textarea_value(&reset, node), "D\nB".into());
                assert!(!textarea(&reset, node).flags().dirty_value());
                assert!(!textarea(&reset, node).flags().user_validity());
                assert_eq!(
                    textarea(&reset, node).selection().cursor_position(),
                    Some(3)
                );
                assert_eq!(
                    textarea(&reset, node).selection().direction(),
                    expected_direction(none)
                );
                assert_eq!(
                    textarea(&reset, node).custom_validity_message(),
                    &DocumentString::from("kept\nmessage")
                );
                let mut batch = DomBatch::new();
                batch.replace_data(first, 0, usize::MAX, "E\r");
                apply(&mut fixture, batch);
                assert_eq!(
                    textarea_value(&fixture.session.snapshot(), node),
                    "E\nB".into()
                );
                let mut batch = DomBatch::new();
                batch.set_textarea_default_value(node, "Q\r\n");
                apply(&mut fixture, batch);
                let current = fixture.session.snapshot();
                assert_eq!(textarea_default(&current, node), "Q\r\n".into());
                assert_eq!(textarea_value(&current, node), "Q\n".into());
                // Replace-all removes the old children before inserting the
                // replacement Text. The clean empty intermediate clamps to 0;
                // later insertion does not synthesize a value-setter cursor.
                assert_eq!(
                    textarea(&current, node).selection().cursor_position(),
                    Some(0)
                );
                assert_eq!(
                    textarea(&current, node).selection().direction(),
                    expected_direction(none)
                );
                assert_eq!(textarea_value(&old, node), "A\nB".into());
                assert_eq!(
                    textarea_value(&dirty, node).code_units(),
                    [88, 0xd83d, 0xde00, 10, 89]
                );
                assert_ne!(current.stamp(), old.stamp());
            }
        }
    }
}

fn no_dialog_services(snapshot: &DomSnapshot, node: NodeId) {
    let dialog = snapshot.node(node).unwrap().as_dialog().unwrap();
    assert!(!dialog.is_modal());
    assert!(!dialog.has_toggle_task_tracker());
    assert!(!dialog.has_close_watcher());
    assert_eq!(dialog.request_close_return_value(), None);
    assert_eq!(dialog.request_close_source(), None);
    assert!(!dialog.force_close_watcher_enabled());
    assert!(
        !snapshot
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .has_previously_focused_element()
    );
}

#[test]
fn closed_dialog_insert_and_open_hooks_use_actual_retired_inactivity_after_empty_cleanup() {
    for destination in [Destination::NeverHosted, Destination::Retired] {
        let mut fixture = HostedFixture::active();
        let target_document = document(&fixture, destination);
        let node = create(&mut fixture, target_document, "dialog");
        let parent = if matches!(destination, Destination::NeverHosted) {
            super::support::body(&fixture.session.snapshot(), target_document)
        } else {
            fixture.body
        };
        finish_setup(&mut fixture, destination);
        let old = fixture.session.snapshot();
        no_dialog_services(&old, node);
        let mut batch = DomBatch::new();
        batch.insert_before(node, parent, None);
        apply(&mut fixture, batch);
        let inserted = fixture.session.snapshot();
        assert_eq!(inserted.node(node).unwrap().parent(), Some(parent));
        assert!(inserted.node(node).unwrap().is_connected());
        no_dialog_services(&inserted, node);
        let mut batch = DomBatch::new();
        batch.set_attribute(node, "open", "");
        apply(&mut fixture, batch);
        let open = fixture.session.snapshot();
        assert_eq!(
            open.node(node).unwrap().attribute(&"open".into()),
            Some(&DocumentString::from(""))
        );
        no_dialog_services(&open, node);
        let mut batch = DomBatch::new();
        batch.remove_attribute(node, "open");
        apply(&mut fixture, batch);
        let current = fixture.session.snapshot();
        assert_eq!(current.node(node).unwrap().attribute(&"open".into()), None);
        assert_eq!(current.node(node).unwrap().parent(), Some(parent));
        no_dialog_services(&current, node);
        assert_eq!(current.incomplete_source_operation(), None);
        assert_eq!(
            fixture.session.details_execution().pending_tasks().unwrap(),
            0
        );
        assert_eq!(old.node(node).unwrap().parent(), None);
        assert_eq!(
            open.node(node).unwrap().attribute(&"open".into()),
            Some(&DocumentString::from(""))
        );
    }
}

#[test]
fn active_dialog_open_still_refuses_genuine_unimplemented_setup_atomically() {
    let mut fixture = HostedFixture::active();
    let doc = fixture.top.document();
    let node = create(&mut fixture, doc, "dialog");
    let mut batch = DomBatch::new();
    batch.insert_before(node, fixture.body, None);
    apply(&mut fixture, batch);
    let before = fixture.session.snapshot();
    let usage = fixture.session.dom().usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(node, "open", "");
    assert_eq!(
        fixture.session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::DialogProcessingUnavailable {
            action: HtmlDialogAction::AddOpenDialog
        }
    );
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_eq!(fixture.session.dom().usage(), usage);
    assert_eq!(
        fixture
            .session
            .snapshot()
            .node(node)
            .unwrap()
            .attribute(&"open".into()),
        None
    );
    no_dialog_services(&fixture.session.snapshot(), node);
}

#[test]
fn standalone_img_adopted_into_retired_document_waits_before_resources_and_supersedes_exactly() {
    let mut fixture = HostedFixture::retired();
    let closed = fixture.closed;
    let node = create(&mut fixture, closed, "img");
    let old = fixture.session.snapshot();
    let initial = old
        .node(node)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap();
    let native = old.node(node).unwrap().as_img().unwrap().state().clone();
    assert_eq!(initial.document(), closed);
    assert_eq!(
        native.current_request().state(),
        HtmlImgFreshRequestState::Unavailable
    );
    assert_eq!(initial.flags(), HtmlImageUpdateFlags::new(false, false));
    let mut batch = DomBatch::new();
    batch.adopt(node, fixture.top.document());
    // Do not accept the old registry/adoption gate as an image service boundary.
    apply(&mut fixture, batch);
    let adopted = fixture.session.snapshot();
    let image = adopted.node(node).unwrap().as_img().unwrap();
    let waiting = image.suspended_update().unwrap().unwrap();
    assert_eq!(
        adopted.node(node).unwrap().owner_document(),
        Some(fixture.top.document())
    );
    assert_eq!(waiting.document(), fixture.top.document());
    assert_eq!(waiting.id().target(), node);
    assert_ne!(waiting.id(), initial.id());
    assert_eq!(waiting.flags(), HtmlImageUpdateFlags::new(false, false));
    assert_eq!(image.state(), &native);
    let mut batch = DomBatch::new();
    batch.set_attribute(node, "src", "never parsed or fetched");
    apply(&mut fixture, batch);
    let first_src = fixture.session.snapshot();
    let first = first_src
        .node(node)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap();
    // HTML877d relevant mutations sets restart-animation only for an equal
    // src SET. Adding the first src changes its actual value.
    assert_eq!(first.flags(), HtmlImageUpdateFlags::new(false, false));
    assert_ne!(first.id(), waiting.id());
    let units = fixture.session.dom().usage().string_code_units();
    let mut batch = DomBatch::new();
    batch.set_attribute(node, "src", "never parsed or fetched");
    apply(&mut fixture, batch);
    let current = fixture.session.snapshot();
    let image = current.node(node).unwrap().as_img().unwrap();
    let latest = image.suspended_update().unwrap().unwrap();
    assert_ne!(latest.id(), first.id());
    assert_eq!(latest.document(), fixture.top.document());
    assert_eq!(latest.flags(), HtmlImageUpdateFlags::new(true, false));
    assert_eq!(image.state(), &native);
    assert_eq!(fixture.session.dom().usage().string_code_units(), units);
    assert_eq!(
        old.node(node)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update()
            .unwrap()
            .unwrap()
            .id(),
        initial.id()
    );
    assert_eq!(
        first_src
            .node(node)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update()
            .unwrap()
            .unwrap()
            .id(),
        first.id()
    );
}

#[test]
fn active_img_adoption_keeps_actual_resource_requirement_and_rolls_back() {
    let mut fixture = HostedFixture::active();
    let closed = fixture.closed;
    let node = create(&mut fixture, closed, "img");
    let before = fixture.session.snapshot();
    let usage = fixture.session.dom().usage();
    let prior = before
        .node(node)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap();
    let mut batch = DomBatch::new();
    batch.adopt(node, fixture.top.document());
    assert_eq!(
        fixture.session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::ImageProcessingUnavailable
    );
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_eq!(fixture.session.dom().usage(), usage);
    let current = fixture.session.snapshot();
    assert_eq!(current.node(node).unwrap().owner_document(), Some(closed));
    assert_eq!(
        current
            .node(node)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update()
            .unwrap()
            .unwrap()
            .id(),
        prior.id()
    );
}

#[test]
fn retired_control_cancellation_late_invalidity_and_string_limit_preserve_atomic_state() {
    let mut fixture = HostedFixture::active();
    let doc = fixture.top.document();
    let input_node = create(&mut fixture, doc, "input");
    let textarea_node = create(&mut fixture, doc, "textarea");
    let mut batch = DomBatch::new();
    batch.set_input_value(input_node, "before");
    batch.set_textarea_value(textarea_node, "before\r\n");
    apply(&mut fixture, batch);
    fixture.retire();
    let before = fixture.session.snapshot();
    let usage = fixture.session.dom().usage();
    let mut batch = DomBatch::new();
    batch.set_input_value(input_node, "canceled");
    batch.set_textarea_value(textarea_node, "canceled");
    assert_eq!(
        fixture
            .session
            .dom()
            .apply(batch, &AtomicBool::new(true))
            .unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_eq!(fixture.session.dom().usage(), usage);
    let mut batch = DomBatch::new();
    batch.set_input_value(input_node, "would commit");
    batch.set_textarea_value(textarea_node, "would commit");
    batch.insert_before(input_node, input_node, None);
    assert_eq!(
        fixture.session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_eq!(fixture.session.dom().usage(), usage);
    let limit = fixture.session.dom().limits().string_code_units();
    let mut batch = DomBatch::new();
    batch.set_textarea_value(
        textarea_node,
        DocumentString::from_utf16(vec![120; limit + 1]),
    );
    assert!(matches!(
        fixture.session.dom().apply(batch, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            ..
        })
    ));
    let current = fixture.session.snapshot();
    assert_eq!(current.stamp(), before.stamp());
    assert_eq!(fixture.session.dom().usage(), usage);
    assert_eq!(
        input_value(&current, input_node),
        "before".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(textarea_value(&current, textarea_node), "before\n".into());
    assert_eq!(input(&current, input_node), input(&before, input_node));
    assert_eq!(
        textarea(&current, textarea_node),
        textarea(&before, textarea_node)
    );
}

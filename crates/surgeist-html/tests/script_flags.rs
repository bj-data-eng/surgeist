//! Independent flag expectations from HTML877d #script-processing-model,
//! #prepare-the-script-element and the HTML script cloning steps.

use surgeist_html::HtmlScriptFlags;

fn slots(flags: HtmlScriptFlags) -> [bool; 5] {
    [
        flags.force_async(),
        flags.already_started(),
        flags.external_file(),
        flags.ready_to_be_parser_executed(),
        flags.delaying_load(),
    ]
}

#[test]
fn native_creation_and_parser_insertion_have_distinct_initial_flags() {
    let native = HtmlScriptFlags::fresh();
    assert_eq!(slots(native), [true, false, false, false, false]);

    let mut parsed = native;
    parsed.mark_parser_inserted();
    assert_eq!(slots(parsed), [false, false, false, false, false]);

    parsed.mark_already_started();
    assert_eq!(slots(parsed), [false, true, false, false, false]);
    assert_eq!(slots(native), [true, false, false, false, false]);
}

#[test]
fn retained_flags_preserve_established_slots_without_native_reinitialization() {
    let flags = HtmlScriptFlags::from_retained(false, true, true, true, true);
    assert_eq!(slots(flags), [false, true, true, true, true]);
}

#[test]
fn async_addition_and_preparation_change_only_their_source_selected_slots() {
    let mut flags = HtmlScriptFlags::from_retained(true, false, true, true, true);
    flags.async_attribute_added();
    assert_eq!(slots(flags), [false, false, true, true, true]);
    flags.parser_document_cleared_without_async();
    assert_eq!(slots(flags), [true, false, true, true, true]);
    flags.parser_document_restored();
    assert_eq!(slots(flags), [false, false, true, true, true]);
    flags.mark_already_started();
    flags.mark_already_started();
    assert_eq!(slots(flags), [false, true, true, true, true]);
}

#[test]
fn cloning_overlays_started_and_preserves_target_attribute_effects() {
    let started_source = HtmlScriptFlags::from_retained(true, true, true, true, true);
    let mut target = HtmlScriptFlags::fresh();
    target.async_attribute_added();
    target.clone_already_started_from(started_source);
    assert_eq!(slots(target), [false, true, false, false, false]);

    let unstarted_source = HtmlScriptFlags::from_retained(false, false, true, true, true);
    target.clone_already_started_from(unstarted_source);
    assert_eq!(slots(target), [false, false, false, false, false]);

    let mut without_async = HtmlScriptFlags::fresh();
    without_async.clone_already_started_from(started_source);
    assert_eq!(slots(without_async), [true, true, false, false, false]);
}

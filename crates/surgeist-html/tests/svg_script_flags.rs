//! SVG Tiny processed state and the separately selected actual-bit clone rule.
use surgeist_html::{HtmlScriptFlags, SvgScriptFlags};

#[test]
fn fresh_svg_script_has_not_been_processed() {
    assert!(!SvgScriptFlags::fresh().already_processed());
}

#[test]
fn actual_retained_false_and_true_are_preserved() {
    assert!(!SvgScriptFlags::from_retained(false).already_processed());
    assert!(SvgScriptFlags::from_retained(true).already_processed());
}

#[test]
fn selected_clones_copy_false_and_true_and_have_independent_state() {
    let source_false = SvgScriptFlags::from_retained(false);
    let mut clone_false = source_false.clone_for_element();
    assert!(!clone_false.already_processed());
    clone_false.mark_processed();
    assert!(clone_false.already_processed());
    assert!(!source_false.already_processed());

    let source_true = SvgScriptFlags::from_retained(true);
    assert!(source_true.clone_for_element().already_processed());
    assert!(source_true.already_processed());
}

#[test]
fn marking_processed_is_monotonic_and_idempotent() {
    let mut flags = SvgScriptFlags::fresh();
    flags.mark_processed();
    assert!(flags.already_processed());
    flags.mark_processed();
    assert!(flags.already_processed());
    assert!(!SvgScriptFlags::fresh().already_processed());
}

#[test]
fn svg_processed_state_is_independent_of_html_preparation_flags() {
    let mut svg = SvgScriptFlags::fresh();
    let mut html = HtmlScriptFlags::fresh();
    assert!(html.force_async());
    assert!(!html.already_started());
    svg.mark_processed();
    assert!(svg.already_processed());
    assert!(!html.already_started());
    let fresh_svg = SvgScriptFlags::fresh();
    html.mark_already_started();
    assert!(html.already_started());
    assert!(!fresh_svg.already_processed());
}

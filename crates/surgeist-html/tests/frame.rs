use surgeist_html::{
    HtmlBrowsingContextPresence, HtmlFrameNullContent, HtmlFramePostConnectionDecision,
    frame_post_connection,
};

#[test]
fn absent_browsing_context_returns_before_frame_creation_or_attribute_work() {
    assert_eq!(
        frame_post_connection(HtmlBrowsingContextPresence::Absent),
        HtmlFramePostConnectionDecision::Return
    );
}

#[test]
fn present_browsing_context_continues_the_actual_frame_algorithm() {
    assert_eq!(
        frame_post_connection(HtmlBrowsingContextPresence::Present),
        HtmlFramePostConnectionDecision::Continue
    );
}

#[test]
fn unavailable_browsing_context_establishes_neither_source_branch() {
    assert_eq!(
        frame_post_connection(HtmlBrowsingContextPresence::Unavailable),
        HtmlFramePostConnectionDecision::NeedsBrowsingContext
    );
}

#[test]
fn actual_fresh_null_content_has_null_getters_and_no_src_or_destruction_work() {
    // #child-navigables: initially null; #frames: src processing only when
    // non-null; #destroy-a-child-navigable: null returns before any effect.
    let content = HtmlFrameNullContent::fresh();
    assert!(content.content_navigable_is_null());
    assert!(content.content_document_is_null());
    assert!(content.content_window_is_null());
    assert!(!content.src_change_requires_processing());
    assert!(!content.removal_requires_child_destruction());
    // Repeating src/removal decisions does not invent a child or change state.
    assert!(!content.src_change_requires_processing());
    assert!(!content.removal_requires_child_destruction());
    assert!(content.content_navigable_is_null());
}

#[test]
fn frame_potential_classification_does_not_create_actual_load_delay() {
    let content = HtmlFrameNullContent::fresh();
    assert!(content.potentially_delays_load());
    assert!(!content.delays_load());
}

use surgeist_html::{
    HtmlBrowsingContextPresence, HtmlChildNavigableAccessDecision,
    HtmlChildNavigableLoadDelayDecision, HtmlChildNavigablePresence,
    HtmlFramePostConnectionDecision, frame_post_connection, frame_potentially_delays_load,
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
    let content = HtmlChildNavigablePresence::Null;
    assert_eq!(content, HtmlChildNavigablePresence::Null);
    assert_eq!(
        content.content_access(),
        HtmlChildNavigableAccessDecision::ReturnNull
    );
    assert_eq!(
        content.content_access(),
        HtmlChildNavigableAccessDecision::ReturnNull
    );
    assert!(!content.attribute_change_requires_processing());
    assert!(!content.removal_requires_child_destruction());
    // Repeating src/removal decisions does not invent a child or change state.
    assert!(!content.attribute_change_requires_processing());
    assert!(!content.removal_requires_child_destruction());
    assert_eq!(content, HtmlChildNavigablePresence::Null);
}

#[test]
fn frame_potential_classification_does_not_create_actual_load_delay() {
    let content = HtmlChildNavigablePresence::Null;
    assert!(frame_potentially_delays_load());
    assert_eq!(
        content.load_delay(),
        HtmlChildNavigableLoadDelayDecision::DoesNotDelay
    );
}

#[test]
fn present_frame_requires_actual_child_processing_access_load_and_destruction() {
    let content = HtmlChildNavigablePresence::Present;
    assert!(content.attribute_change_requires_processing());
    assert!(content.removal_requires_child_destruction());
    assert_eq!(
        content.content_access(),
        HtmlChildNavigableAccessDecision::Continue
    );
    assert!(frame_potentially_delays_load());
    assert_eq!(
        content.load_delay(),
        HtmlChildNavigableLoadDelayDecision::Continue
    );
}

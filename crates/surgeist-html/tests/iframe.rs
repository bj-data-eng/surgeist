use surgeist_html::{
    Attribute, DocumentString, HtmlBrowsingContextPresence, HtmlChildNavigableAccessDecision,
    HtmlChildNavigableLoadDelayDecision, HtmlChildNavigablePresence, HtmlIframeInitialState,
    HtmlIframePostConnectionDecision, HtmlLoading, HtmlSandboxDirectiveState, HtmlSandboxFlag,
    HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement, iframe_loading_became_eager,
    iframe_post_connection,
};

#[test]
fn fresh_native_iframe_has_source_false_null_tuple_without_actual_load_delay() {
    // #child-navigables, #current-navigation-was-lazy-loaded,
    // #iframe-pending-resource-timing-start-time, #iframe-pending-resource-timing-url
    // and #lazy-load-resumption-steps give these independent literal values.
    let state = HtmlIframeInitialState::fresh();
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
    assert!(!state.current_navigation_was_lazy_loaded());
    assert!(state.pending_resource_timing_start_time_is_null());
    assert!(state.pending_resource_timing_url_is_null());
    assert!(state.lazy_load_resumption_is_null());
    assert!(state.potentially_delays_load());
    assert_eq!(
        content.load_delay(),
        HtmlChildNavigableLoadDelayDecision::DoesNotDelay
    );
}

#[test]
fn post_connection_distinguishes_actual_null_present_and_unavailable_context() {
    for (context, expected) in [
        (
            HtmlBrowsingContextPresence::Absent,
            HtmlIframePostConnectionDecision::Return,
        ),
        (
            HtmlBrowsingContextPresence::Present,
            HtmlIframePostConnectionDecision::Continue,
        ),
        (
            HtmlBrowsingContextPresence::Unavailable,
            HtmlIframePostConnectionDecision::NeedsBrowsingContext,
        ),
    ] {
        assert_eq!(iframe_post_connection(context), expected);
    }
}

#[test]
fn null_content_stops_srcdoc_src_sandbox_and_removing_effects() {
    let state = HtmlIframeInitialState::fresh();
    let content = HtmlChildNavigablePresence::Null;
    // #the-iframe-element: content must be nonnull before any srcdoc/src
    // processing or sandbox parse/clear. #destroy-a-child-navigable returns.
    for _ in 0..2 {
        assert!(!content.attribute_change_requires_processing());
        assert!(!content.attribute_change_requires_processing());
        assert!(!content.attribute_change_requires_processing());
        assert!(!content.attribute_change_requires_processing());
        assert!(!content.removal_requires_child_destruction());
        assert_eq!(content, HtmlChildNavigablePresence::Null);
        assert!(!state.current_navigation_was_lazy_loaded());
        assert!(state.pending_resource_timing_start_time_is_null());
        assert!(state.pending_resource_timing_url_is_null());
        assert!(state.lazy_load_resumption_is_null());
    }
    let copied = state;
    assert_eq!(copied, state);
    assert_eq!(content, HtmlChildNavigablePresence::Null);
}

#[test]
fn only_semantic_lazy_to_eager_reaches_the_null_resumption_return() {
    for (previous, current, expected) in [
        (HtmlLoading::Lazy, HtmlLoading::Eager, true),
        (HtmlLoading::Lazy, HtmlLoading::Lazy, false),
        (HtmlLoading::Eager, HtmlLoading::Eager, false),
        (HtmlLoading::Eager, HtmlLoading::Lazy, false),
    ] {
        assert_eq!(iframe_loading_became_eager(previous, current), expected);
    }
    // Even the reached Eager hook returns before invoking anything in this
    // source-initialized phase, because its resumption is actually null.
    let state = HtmlIframeInitialState::fresh();
    assert!(state.lazy_load_resumption_is_null());
    assert!(!state.current_navigation_was_lazy_loaded());
}

#[test]
fn existing_loading_interpreter_preserves_defaults_and_semantic_transition_identity() {
    let interpreter = HtmlValueInterpreter::new(64);
    let iframe = KnownHtmlElement::Iframe.qualified_name();
    let lazy = Attribute::new(KnownHtmlAttribute::Loading.qualified_name(), "LAZY");
    assert_eq!(
        interpreter
            .loading(&iframe, Some(&lazy))
            .unwrap()
            .intrinsic(),
        Some(&HtmlLoading::Lazy)
    );
    for raw in [None, Some(""), Some("invalid"), Some("EAGER")] {
        let attribute =
            raw.map(|value| Attribute::new(KnownHtmlAttribute::Loading.qualified_name(), value));
        let report = interpreter.loading(&iframe, attribute.as_ref()).unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlLoading::Eager));
        assert!(!iframe_loading_became_eager(
            HtmlLoading::Eager,
            *report.intrinsic().unwrap()
        ));
        assert!(iframe_loading_became_eager(
            HtmlLoading::Lazy,
            *report.intrinsic().unwrap()
        ));
    }
}

#[test]
fn absent_empty_and_raw_token_sandbox_reports_are_separate_from_unprocessed_native_policy() {
    let interpreter = HtmlValueInterpreter::new(64);
    let iframe = KnownHtmlElement::Iframe.qualified_name();
    let state = HtmlIframeInitialState::fresh();
    let content = HtmlChildNavigablePresence::Null;
    assert_eq!(
        interpreter
            .iframe_sandbox(&iframe, None::<&surgeist_html::Attribute>)
            .unwrap()
            .intrinsic(),
        Some(&HtmlSandboxDirectiveState::Absent)
    );
    let empty = Attribute::new(KnownHtmlAttribute::Sandbox.qualified_name(), "");
    let report = interpreter.iframe_sandbox(&iframe, Some(&empty)).unwrap();
    assert_eq!(report.raw_attribute(), Some(&empty));
    let Some(HtmlSandboxDirectiveState::Present(flags)) = report.intrinsic() else {
        panic!("present-empty is a directive with restrictions");
    };
    assert_eq!(flags.len(), 16);
    assert!(flags.contains(HtmlSandboxFlag::Scripts));
    assert!(flags.contains(HtmlSandboxFlag::AutomaticFeatures));
    assert!(!flags.is_empty());
    // The isolated surrogate is an exact unknown token, never repaired by the
    // native state or by the independent local attribute report.
    let mut units: Vec<_> = "allow-scripts ".encode_utf16().collect();
    units.push(0xd800);
    let tokens = Attribute::new(
        KnownHtmlAttribute::Sandbox.qualified_name(),
        DocumentString::from_utf16(units.clone()),
    );
    let report = interpreter.iframe_sandbox(&iframe, Some(&tokens)).unwrap();
    assert_eq!(report.raw_attribute(), Some(&tokens));
    assert_eq!(report.raw().unwrap().code_units(), units);
    let Some(HtmlSandboxDirectiveState::Present(flags)) = report.intrinsic() else {
        panic!("token-valued directive has independent local flags");
    };
    assert_eq!(flags.len(), 14);
    assert!(!flags.contains(HtmlSandboxFlag::Scripts));
    assert!(!flags.contains(HtmlSandboxFlag::AutomaticFeatures));
    assert!(!content.attribute_change_requires_processing());
    assert!(!content.attribute_change_requires_processing());
    assert_eq!(content, HtmlChildNavigablePresence::Null);
    assert!(state.pending_resource_timing_url_is_null());
}

#[test]
fn present_child_continues_real_access_and_load_rules_without_changing_initial_intrinsics() {
    // Installing a child is not an intrinsic write. The selected absent-src,
    // no-srcdoc initial path returns at the shared ancestor-URL guard before
    // either lazy setup or navigation timing. These are independent source
    // values; Continue establishes no Window, Document or completed load delay.
    let state = HtmlIframeInitialState::fresh();
    let content = HtmlChildNavigablePresence::Present;
    assert_eq!(
        content.content_access(),
        HtmlChildNavigableAccessDecision::Continue
    );
    assert_eq!(
        content.load_delay(),
        HtmlChildNavigableLoadDelayDecision::Continue
    );
    assert!(content.attribute_change_requires_processing());
    assert!(content.removal_requires_child_destruction());
    assert!(!state.current_navigation_was_lazy_loaded());
    assert!(state.pending_resource_timing_start_time_is_null());
    assert!(state.pending_resource_timing_url_is_null());
    assert!(state.lazy_load_resumption_is_null());
    assert!(state.potentially_delays_load());
}

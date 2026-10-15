use surgeist_html::{
    Attribute, DocumentString, HtmlBrowsingContextPresence, HtmlIframeNativeState,
    HtmlIframePostConnectionDecision, HtmlLoading, HtmlSandboxDirectiveState, HtmlSandboxFlag,
    HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement, iframe_loading_became_eager,
    iframe_post_connection,
};

#[test]
fn fresh_native_iframe_has_source_false_null_tuple_without_actual_load_delay() {
    // #child-navigables, #current-navigation-was-lazy-loaded,
    // #iframe-pending-resource-timing-start-time, #iframe-pending-resource-timing-url
    // and #lazy-load-resumption-steps give these independent literal values.
    let state = HtmlIframeNativeState::fresh();
    assert!(state.content_navigable_is_null());
    assert!(state.content_document_is_null());
    assert!(state.content_window_is_null());
    assert!(!state.current_navigation_was_lazy_loaded());
    assert!(state.pending_resource_timing_start_time_is_null());
    assert!(state.pending_resource_timing_url_is_null());
    assert!(state.lazy_load_resumption_is_null());
    assert!(state.potentially_delays_load());
    assert!(!state.delays_load());
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
    let state = HtmlIframeNativeState::fresh();
    // #the-iframe-element: content must be nonnull before any srcdoc/src
    // processing or sandbox parse/clear. #destroy-a-child-navigable returns.
    for _ in 0..2 {
        assert!(!state.srcdoc_change_requires_processing());
        assert!(!state.src_change_requires_processing());
        assert!(!state.sandbox_change_requires_processing());
        assert!(!state.sandbox_removal_requires_clearing());
        assert!(!state.removal_requires_child_destruction());
        assert!(state.content_navigable_is_null());
        assert!(!state.current_navigation_was_lazy_loaded());
        assert!(state.pending_resource_timing_start_time_is_null());
        assert!(state.pending_resource_timing_url_is_null());
        assert!(state.lazy_load_resumption_is_null());
    }
    let copied = state;
    assert_eq!(copied, state);
    assert!(copied.content_navigable_is_null());
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
    let state = HtmlIframeNativeState::fresh();
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
    let state = HtmlIframeNativeState::fresh();
    assert_eq!(
        interpreter
            .iframe_sandbox(&iframe, None)
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
    assert!(!state.sandbox_change_requires_processing());
    assert!(!state.sandbox_removal_requires_clearing());
    assert!(state.content_navigable_is_null());
    assert!(state.pending_resource_timing_url_is_null());
}

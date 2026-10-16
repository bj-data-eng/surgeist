use std::sync::Arc;
use surgeist_html::{
    DocumentString, HtmlFrameIframeAfterAncestors, HtmlFrameIframeSrcDecision,
    HtmlIframeAttributeBranch,
    bounded::{frame_iframe_after_ancestors_with, frame_iframe_src_with},
    iframe_attribute_branch,
};

#[test]
fn srcdoc_presence_including_empty_selects_the_earlier_iframe_branch() {
    // Process-iframe-attributes step 1 tests presence, before shared src steps.
    for raw in [
        Some(DocumentString::from("")),
        Some(DocumentString::from("<p>content")),
        Some(DocumentString::from_utf16(vec![0xd800])),
    ] {
        assert_eq!(
            iframe_attribute_branch(raw.is_some()),
            HtmlIframeAttributeBranch::Srcdoc
        );
    }
    assert_eq!(
        iframe_attribute_branch(false),
        HtmlIframeAttributeBranch::Shared
    );
}

#[test]
fn absent_and_empty_src_keep_the_shared_about_blank_candidate() {
    // Shared step 2 requires both presence AND nonempty value. This does not
    // replace Permissions Policy's earlier present-src declared-origin rule.
    let empty = DocumentString::from("");
    for raw in [None, Some(&empty)] {
        let mut admitted = 0;
        let decision = frame_iframe_src_with(raw, || {
            admitted += 1;
            Ok::<_, ()>(())
        })
        .unwrap();
        assert!(matches!(decision, HtmlFrameIframeSrcDecision::AboutBlank));
        assert_eq!(admitted, 1);
    }
}

#[test]
fn nonempty_src_borrows_exact_units_without_trimming_repair_or_retention() {
    for units in [
        vec![0x20],
        vec![0xd800],
        "path#part".encode_utf16().collect(),
    ] {
        let backing = Arc::new(units);
        let raw = DocumentString::from_shared_code_units(backing.clone());
        let owners = Arc::strong_count(&backing);
        let decision = frame_iframe_src_with(Some(&raw), || Ok::<_, ()>(())).unwrap();
        let HtmlFrameIframeSrcDecision::ParseSrc(borrowed) = decision else {
            panic!("every nonempty value reaches the qualified encoding parser");
        };
        assert!(std::ptr::eq(borrowed, &raw));
        assert_eq!(borrowed.code_units(), backing.as_slice());
        assert_eq!(Arc::strong_count(&backing), owners);
    }
}

#[test]
fn caller_refusal_precedes_even_the_absent_or_empty_src_header() {
    let empty = DocumentString::from("");
    let nonempty = DocumentString::from("x");
    for raw in [None, Some(&empty), Some(&nonempty)] {
        let result = frame_iframe_src_with(raw, || Err("original stop"));
        assert!(matches!(result, Err("original stop")));
    }
}

#[test]
fn ancestor_match_returns_null_before_any_later_about_blank_read() {
    // Shared step 3 precedes step 4's predicate/history update. Even an
    // unavailable later input cannot replace this already-established return.
    for initial in [false, true] {
        let mut reads = 0;
        assert_eq!(
            frame_iframe_after_ancestors_with(true, initial, || {
                reads += 1;
                Err("later input unavailable")
            }),
            Ok(HtmlFrameIframeAfterAncestors::ReturnNull)
        );
        assert_eq!(reads, 0);
    }
}

#[test]
fn only_a_nonmatching_ancestor_and_initial_about_blank_select_history_update() {
    for (initial, blank, expected) in [
        (
            true,
            true,
            HtmlFrameIframeAfterAncestors::UpdateUrlAndHistory,
        ),
        (false, true, HtmlFrameIframeAfterAncestors::ReturnUrl),
        (true, false, HtmlFrameIframeAfterAncestors::ReturnUrl),
        (false, false, HtmlFrameIframeAfterAncestors::ReturnUrl),
    ] {
        let mut reads = 0;
        assert_eq!(
            frame_iframe_after_ancestors_with(false, initial, || {
                reads += 1;
                Ok::<_, ()>(blank)
            }),
            Ok(expected)
        );
        assert_eq!(reads, 1);
    }
    assert_eq!(
        frame_iframe_after_ancestors_with(false, true, || Err("original stop")),
        Err("original stop")
    );
}

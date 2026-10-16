//! Literal source-framing oracles shared by native execution and static reports.
use std::{convert::Infallible, ops::Range};
use surgeist_html::{DocumentString, HtmlReferrerPolicy, bounded::*};

#[test]
fn language_rejects_any_comma_and_preserves_exact_first_token_units() {
    for (raw, expected) in [
        (" \tEN-private extra", Some(2..12)),
        ("en extra,tail", None),
        (" \r\n", None),
        ("invalid_tag", Some(0..11)),
    ] {
        assert_eq!(
            meta_language_token_with(&raw.into(), || Ok::<_, Infallible>(())).unwrap(),
            expected
        );
    }
    let raw = DocumentString::from_utf16(vec![32, 0xd800, 32, 0xdc00]);
    assert_eq!(
        meta_language_token_with(&raw, || Ok::<_, Infallible>(())).unwrap(),
        Some(1..2)
    );
}

#[test]
fn refresh_frames_recovery_without_integer_truncation_or_url_claims() {
    use HtmlMetaRefreshSyntaxTarget::{DocumentUrl, UrlString};
    for (raw, expected, digits) in [
        ("0", Some(DocumentUrl), Some(0..1)),
        (" .", Some(DocumentUrl), Some(1..1)),
        (
            "123456789012345678901234567890..9; \t",
            Some(DocumentUrl),
            Some(0..30),
        ),
        ("-1", None, None),
        ("2x", None, Some(0..1)),
        ("0; URL='next'ignored", Some(UrlString(8..12)), Some(0..1)),
        ("0; UrX='next'", Some(UrlString(3..13)), Some(0..1)),
        ("0; 'next'ignored", Some(UrlString(4..8)), Some(0..1)),
        ("0; URL=", Some(UrlString(7..7)), Some(0..1)),
    ] {
        let mut observed: Option<Range<usize>> = None;
        let target = meta_refresh_syntax_with(&raw.into(), |action| {
            if let HtmlMetaRefreshAction::DelayDigits(range) = action {
                observed = Some(range);
            }
            Ok::<_, Infallible>(())
        })
        .unwrap();
        assert_eq!(target, expected, "{raw}");
        assert_eq!(observed, digits, "{raw}");
    }
}

#[test]
fn refresh_delay_admission_precedes_suffix_and_stops_without_later_callbacks() {
    let mut calls = 0;
    let mut delay_seen = false;
    let result = meta_refresh_syntax_with(&"12; URL='later'".into(), |action| {
        assert!(!delay_seen, "no callbacks follow a refused delay");
        calls += 1;
        match action {
            HtmlMetaRefreshAction::DelayDigits(range) => {
                assert_eq!(range, 0..2);
                delay_seen = true;
                Err("delay")
            }
            HtmlMetaRefreshAction::Step => Ok(()),
        }
    });
    assert_eq!(result, Err("delay"));
    assert!(delay_seen);
    assert_eq!(calls, 6);
}

#[test]
fn meta_referrer_reuses_actual_vocabulary_and_exact_legacy_aliases() {
    for (raw, expected) in [
        ("NEVER", Some(HtmlReferrerPolicy::NoReferrer)),
        (
            "default",
            Some(HtmlReferrerPolicy::StrictOriginWhenCrossOrigin),
        ),
        ("always", Some(HtmlReferrerPolicy::UnsafeUrl)),
        (
            "origin-when-crossorigin",
            Some(HtmlReferrerPolicy::OriginWhenCrossOrigin),
        ),
        ("SAME-ORIGIN", Some(HtmlReferrerPolicy::SameOrigin)),
        ("", None),
        (" no-referrer", None),
        ("no-referrer ", None),
    ] {
        assert_eq!(
            meta_referrer_policy_with(&raw.into(), || Ok::<_, Infallible>(())).unwrap(),
            expected
        );
    }
}

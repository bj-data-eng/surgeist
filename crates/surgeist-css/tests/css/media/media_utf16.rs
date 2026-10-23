use std::sync::Arc;
use surgeist_css::*;

fn parse(raw: Arc<Vec<u16>>) -> CssParseReport<CssMediaQueryList> {
    CssParserContext::default()
        .parse_media_query_list_utf16_with_limits(raw, CssUtf16InputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .unwrap()
}

#[test]
fn media_preprocessing_uses_exact_original_units_and_normalized_diagnostic_source() {
    let raw = Arc::new(vec![13, 10, 12, 0, 0xd800, 0xd83d, 0xde00, 44, 63]);
    let report = parse(raw.clone());
    let source = report.syntax().parse_source().unwrap().source();
    assert_eq!(source.as_str(), "\n\n��😀,?");
    assert!(Arc::ptr_eq(source.original_utf16().unwrap(), &raw));
    for (bytes, units) in [
        (0..1, 0..2),
        (1..2, 2..3),
        (2..5, 3..4),
        (5..8, 4..5),
        (8..12, 5..7),
        (12..13, 7..8),
        (13..14, 8..9),
        (14..14, 9..9),
    ] {
        assert_eq!(source.original_utf16_range(bytes), Some(units));
    }
    assert!(!report.diagnostics().is_empty());
    let diagnostic = report.diagnostics()[0].clone();
    assert!(diagnostic.source().unwrap().same_snapshot(source));
    drop(report);
    assert_eq!(
        diagnostic
            .source()
            .unwrap()
            .original_utf16()
            .unwrap()
            .as_slice(),
        raw.as_slice()
    );
}

#[test]
fn empty_and_comment_only_media_keep_complete_context_without_changing_equality() {
    let context =
        CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical();
    for text in ["", "/* no members */"] {
        let a = context
            .parse_media_query_list_utf16_with_limits(
                Arc::new(text.encode_utf16().collect()),
                CssUtf16InputLimits::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap();
        let b = context.parse_media_query_list(text);
        assert!(a.syntax().queries().is_empty());
        assert_eq!(a.syntax(), &CssMediaQueryList::new(vec![]));
        assert_eq!(a.syntax(), b.syntax());
        assert_eq!(a.syntax().parse_source().unwrap().context(), context);
        assert_eq!(b.syntax().parse_source().unwrap().context(), context);
        assert!(
            !a.syntax()
                .parse_source()
                .unwrap()
                .source()
                .same_snapshot(b.syntax().parse_source().unwrap().source())
        );
        assert_eq!(
            parse_media_query_list(text)
                .syntax()
                .parse_source()
                .unwrap()
                .context(),
            CssParserContext::default()
        );
    }
    assert!(CssMediaQueryList::new(vec![]).parse_source().is_none());
}

#[test]
fn media_preparation_limits_actual_backing_and_stops_at_late_interruption() {
    let mut units = Vec::with_capacity(512);
    units.extend("screen".encode_utf16());
    let raw = Arc::new(units);
    let report = parse(raw.clone());
    let bytes = report
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .storage_usage()
        .retained_bytes();
    for (maximum, success) in [(bytes, true), (bytes - 1, false)] {
        let result = CssParserContext::default().parse_media_query_list_utf16_with_limits(
            raw.clone(),
            CssUtf16InputLimits {
                max_retained_bytes: maximum,
                ..Default::default()
            },
            || Ok::<_, ()>(()),
        );
        assert_eq!(result.is_ok(), success);
        if !success {
            assert!(result.as_ref().is_err_and(
                |error| error.exhausted_limit() == Some(CssUtf16Resource::RetainedBytes)
            ));
        }
    }
    drop(report);
    let mut total = 0;
    drop(
        CssParserContext::default()
            .parse_media_query_list_utf16_with_limits(
                raw.clone(),
                CssUtf16InputLimits::default(),
                || {
                    total += 1;
                    Ok::<_, usize>(())
                },
            )
            .unwrap(),
    );
    for stop in [1, 3, total - 1, total] {
        let mut calls = 0;
        let result = CssParserContext::default().parse_media_query_list_utf16_with_limits(
            raw.clone(),
            CssUtf16InputLimits::default(),
            || {
                calls += 1;
                if calls == stop { Err(stop) } else { Ok(()) }
            },
        );
        assert!(
            result
                .as_ref()
                .is_err_and(|error| error.interruption().is_some_and(|actual| *actual == stop))
        );
        assert_eq!(calls, stop);
        assert_eq!(Arc::strong_count(&raw), 1);
    }
}

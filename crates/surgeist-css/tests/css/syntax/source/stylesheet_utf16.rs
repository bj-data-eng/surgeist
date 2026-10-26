use std::sync::Arc;
use surgeist_css::*;

fn parse(units: Vec<u16>) -> CssParseReport<CssSheet> {
    CssParser::default()
        .parse_sheet_utf16(Arc::new(units), CssUtf16InputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .unwrap()
}

#[test]
fn preprocessing_preserves_original_units_and_exact_scalar_boundaries() {
    let raw = vec![13, 10, 12, 0, 0xd83d, 0xde00, 0xd800, 97, 0xdc00];
    let report = parse(raw.clone());
    let source = report.syntax().parse_source().unwrap().source();
    assert_eq!(source.as_str(), "\n\n\u{fffd}😀\u{fffd}a\u{fffd}");
    assert_eq!(source.original_utf16().unwrap().as_slice(), raw);
    for (bytes, units) in [
        (0..1, 0..2),
        (1..2, 2..3),
        (2..5, 3..4),
        (5..9, 4..6),
        (9..12, 6..7),
        (12..13, 7..8),
        (13..16, 8..9),
        (16..16, 9..9),
    ] {
        assert_eq!(source.original_utf16_range(bytes), Some(units));
    }
    assert_eq!(source.original_utf16_range(6..9), None);
    assert_eq!(source.original_utf16_range(17..17), None);
    assert_eq!(
        source.original_utf16_range(std::ops::Range { start: 9, end: 5 }),
        None
    );
}

#[test]
fn empty_recovered_and_diagnostic_clones_retain_the_actual_source() {
    let empty = parse(Vec::new());
    assert_eq!(empty.syntax(), &CssSheet::new());
    assert!(empty.syntax().parse_source().is_some());
    let report = parse("@unknown value;".encode_utf16().collect());
    assert!(report.syntax().rules().is_empty());
    let snapshot = report.syntax().parse_source().unwrap().source().clone();
    let diagnostic = report.diagnostics()[0].clone();
    assert!(diagnostic.source().unwrap().same_snapshot(&snapshot));
    drop(report);
    assert_eq!(diagnostic.source().unwrap().as_str(), "@unknown value;");
}

#[test]
fn real_utf8_parses_are_qualified_and_equality_ignores_qualification() {
    let a = CssParser::default().parse_sheet("/* comment */");
    let b = CssParser::default().parse_sheet("/* comment */");
    assert_eq!(a.syntax(), b.syntax());
    let x = a.syntax().parse_source().unwrap();
    let y = b.syntax().parse_source().unwrap();
    assert!(!x.source().same_snapshot(y.source()));
    assert_eq!(x.source(), y.source());
    let context =
        CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical();
    assert_eq!(
        CssParser::new(context, Default::default())
            .parse_sheet("")
            .syntax()
            .parse_source()
            .unwrap()
            .context(),
        context
    );
    assert!(CssSheet::new().parse_source().is_none());
    let decoded = CssDecodedStylesheetInput::decode(
        b"/* decoded */",
        CssStylesheetDecodeHints::default(),
        Default::default(),
    )
    .unwrap();
    let report = CssParser::default()
        .parse_sheet_bytes(&decoded)
        .with_location(None);
    let qualified = report.syntax().parse_source().unwrap();
    assert!(qualified.source().same_snapshot(decoded.source()));
    assert_eq!(qualified.context(), CssParserContext::default());
    assert_eq!(
        CssParser::default()
            .parse_sheet("")
            .with_location(None)
            .syntax()
            .parse_source()
            .unwrap()
            .context(),
        CssParserContext::default()
    );
}

#[test]
fn finite_preparation_rejects_zero_allowances_and_stops_at_first_interruption() {
    let original = Arc::new(vec![97]);
    let base = CssUtf16InputLimits::default();
    for limits in [
        CssUtf16InputLimits {
            max_input_units: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_output_bytes: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_map_entries: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_checkpoints: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_scratch_bytes: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_retained_bytes: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_work: 0,
            ..base
        },
    ] {
        assert!(
            CssParser::default()
                .parse_sheet_utf16(original.clone(), limits, || Ok::<_, ()>(()))
                .as_ref()
                .is_err_and(|error| error.exhausted_limit().is_some())
        );
    }
    for stop in 1..=7 {
        let mut calls = 0;
        let result = CssParser::default().parse_sheet_utf16(original.clone(), base, || {
            calls += 1;
            if calls == stop { Err(stop) } else { Ok(()) }
        });
        assert_eq!(result.unwrap_err().interruption(), Some(&stop));
        assert_eq!(calls, stop);
    }
}

#[test]
fn actual_padded_original_backing_counts_independently_of_text_length() {
    let mut raw = Vec::with_capacity(4096);
    raw.push(97);
    let limits = CssUtf16InputLimits {
        max_retained_bytes: 8191,
        ..Default::default()
    };
    assert!(
        CssParser::default()
            .parse_sheet_utf16(Arc::new(raw), limits, || Ok::<_, ()>(()))
            .as_ref()
            .is_err_and(|error| error.exhausted_limit() == Some(CssUtf16Resource::RetainedBytes))
    );
}

//! CSS Syntax 3 (2026-10-01), section 3.2 "The input byte stream" and section
//! 8.3 "The @charset Rule", with the decision recorded on issue #2868: the
//! exact encoding declaration at the start of a stylesheet is consumed, so it
//! yields no rule and no diagnostic; every other `@charset` is an unrecognized
//! at-rule that is dropped with one diagnostic.
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#input-byte-stream>
//!
//! The declaration is the characters `@charset "`, any ASCII characters other
//! than `"`, then `";`, at offset 0 and within the first 1024 bytes. The
//! stylesheet fronts (text, decoded bytes and UTF-16) give the same report for
//! the same text; the other fronts keep reporting `@charset` as an unknown
//! at-rule. The expectations are spelled from the clause text and the offsets
//! of the sources below, never from a run.

use std::sync::Arc;

use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::rule::{CssRule, CssSheet};
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::source::{CssDecodedStylesheetInput, CssUtf16InputLimits};
use surgeist_css::{CssParseReport, CssParser};
use surgeist_css_parser::input::{DecodeHints, Encoding, EncodingSelection};

/// What a report shows about the declaration: every diagnostic as `(code,
/// action, start, end)` and every retained rule as `(is a style rule, start)`.
#[derive(Debug, Eq, PartialEq)]
struct Observed {
    clean: bool,
    diagnostics: Vec<(CssErrorCode, CssRecoveryAction, usize, usize)>,
    style_starts: Vec<usize>,
    rule_count: usize,
}

fn observe(report: &CssParseReport<CssSheet>) -> Observed {
    Observed {
        clean: report.is_clean(),
        diagnostics: report
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.error().code(),
                    diagnostic.action(),
                    diagnostic.span().start().byte_offset().value(),
                    diagnostic.span().end().byte_offset().value(),
                )
            })
            .collect(),
        style_starts: report
            .syntax()
            .rules()
            .iter()
            .filter_map(|rule| match rule {
                CssRule::Style(style) => Some(style.position().byte_offset().value()),
                _ => None,
            })
            .collect(),
        rule_count: report.syntax().rules().len(),
    }
}

fn text_front(source: &str) -> Observed {
    observe(&CssParser::default().parse_sheet(source))
}

fn utf16_front(source: &str) -> Observed {
    let units: Vec<u16> = source.encode_utf16().collect();
    let report = CssParser::default()
        .parse_sheet_utf16(Arc::new(units), CssUtf16InputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .expect("an unlimited UTF-16 preparation succeeds");
    observe(&report)
}

fn decode(bytes: &[u8], hints: DecodeHints<'_>) -> CssDecodedStylesheetInput {
    CssDecodedStylesheetInput::decode(bytes, hints, Default::default())
        .expect("an unlimited decode succeeds")
}

fn byte_front(bytes: &[u8], hints: DecodeHints<'_>) -> Observed {
    observe(&CssParser::default().parse_sheet_bytes(&decode(bytes, hints)))
}

/// The text, UTF-16 and UTF-8 byte fronts agree on `source`.
fn all_fronts(source: &str) -> Observed {
    let text = text_front(source);
    assert_eq!(utf16_front(source), text, "UTF-16 front: {source:?}");
    assert_eq!(
        byte_front(source.as_bytes(), DecodeHints::default()),
        text,
        "byte front: {source:?}"
    );
    text
}

const RULE: &str = ".a{color:red}";

fn consumed(declaration: &str) {
    let source = format!("{declaration}{RULE}");
    let observed = all_fronts(&source);
    assert!(observed.clean, "{declaration:?}: {observed:?}");
    assert_eq!(observed.diagnostics, [], "{declaration:?}");
    // The one rule after the declaration keeps its coordinates in the text.
    assert_eq!(
        observed.style_starts,
        [declaration.len()],
        "{declaration:?}"
    );
    assert_eq!(observed.rule_count, 1, "{declaration:?}");
    let report = CssParser::default().parse_sheet(&source);
    assert!(
        report.into_validation_result().is_ok(),
        "{declaration:?} is accepted by validation"
    );
}

fn kept(source: &str, dropped: (usize, usize), style_starts: &[usize]) {
    let observed = all_fronts(source);
    assert_eq!(
        observed.diagnostics,
        [(
            CssErrorCode::UnknownAtRule,
            CssRecoveryAction::DropAtRule,
            dropped.0,
            dropped.1
        )],
        "{source:?}"
    );
    assert!(!observed.clean, "{source:?}");
    assert_eq!(observed.style_starts, style_starts, "{source:?}");
    assert_eq!(observed.rule_count, style_starts.len(), "{source:?}");
    assert!(
        CssParser::default()
            .parse_sheet(source)
            .into_validation_result()
            .is_err(),
        "{source:?} is rejected by validation"
    );
}

#[test]
fn the_exact_declaration_is_consumed_whether_or_not_the_label_names_an_encoding() {
    for declaration in [
        "@charset \"utf-8\";",
        "@charset \"UTF-8\";",
        "@charset \"windows-1252\";",
        "@charset \"utf-16le\";",
        "@charset \"no-such-encoding\";",
        // Zero characters between the quotation marks are still the pattern.
        "@charset \"\";",
        // Any ASCII character but `"`, including `;`, `\`, `'` and a space.
        "@charset \"a;b\\\";",
        "@charset \"'x y'\";",
    ] {
        consumed(declaration);
    }
}

#[test]
fn a_stylesheet_of_only_the_declaration_is_clean_and_empty() {
    let observed = all_fronts("@charset \"utf-8\";");
    assert!(observed.clean, "{observed:?}");
    assert_eq!(observed.diagnostics, []);
    assert_eq!(observed.rule_count, 0);
    assert!(
        CssParser::default()
            .parse_sheet("@charset \"utf-8\";")
            .into_validation_result()
            .is_ok()
    );
}

#[test]
fn the_rules_after_the_declaration_keep_their_coordinates_and_order() {
    // Offsets: the declaration is 17 bytes, the line break one more, so the
    // style rule starts at line 1 column 0 of offset 18.
    let source = "@charset \"utf-8\";\n.a{color:red}@unknown x;.b{color:blue}";
    let report = CssParser::default().parse_sheet(source);
    let [a, b] = report.syntax().rules() else {
        panic!("two style rules: {report:?}")
    };
    let (CssRule::Style(a), CssRule::Style(b)) = (a, b) else {
        panic!("style rules")
    };
    assert_eq!(a.position().byte_offset().value(), 18);
    assert_eq!(a.position().line().value(), 1);
    assert_eq!(a.position().column().value(), 0);
    assert_eq!(
        b.position().byte_offset().value(),
        18 + ".a{color:red}@unknown x;".len()
    );
    // Only the later at-rule is reported, at its own position.
    let [diagnostic] = report.diagnostics() else {
        panic!("one unknown at-rule: {report:?}")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownAtRule);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 31);
    assert_eq!(diagnostic.span().start().line().value(), 1);
    assert_eq!(diagnostic.span().start().column().value(), 13);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 42);
    assert_eq!(utf16_front(source), observe(&report));
}

#[test]
fn the_declaration_does_not_start_the_import_phase_or_end_it() {
    // An `@import` after the declaration is still first in the sheet.
    let source = "@charset \"utf-8\";@import \"a.css\";.a{color:red}";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    assert!(
        matches!(
            report.syntax().rules(),
            [CssRule::Import(_), CssRule::Style(_)]
        ),
        "{report:?}"
    );
}

#[test]
fn bytes_with_the_declaration_are_clean_with_and_without_a_byte_order_mark() {
    let rule = RULE.as_bytes();
    for label in ["utf-8", "windows-1252", "no-such-encoding"] {
        let declaration = format!("@charset \"{label}\";");
        let expected = Observed {
            clean: true,
            diagnostics: Vec::new(),
            style_starts: vec![declaration.len()],
            rule_count: 1,
        };
        let plain = [declaration.as_bytes(), rule].concat();
        assert_eq!(
            byte_front(&plain, DecodeHints::default()),
            expected,
            "{label}"
        );
        // A UTF-8 byte order mark is removed by decoding; the declaration is
        // recognized after it, at offset 0 of the decoded text.
        let utf8_bom = [&b"\xEF\xBB\xBF"[..], declaration.as_bytes(), rule].concat();
        let decoded = decode(&utf8_bom, DecodeHints::default());
        assert_eq!(decoded.selection(), EncodingSelection::ByteOrderMark);
        assert!(decoded.source().as_str().starts_with("@charset"));
        assert_eq!(
            observe(&CssParser::default().parse_sheet_bytes(&decoded)),
            expected,
            "{label} after a UTF-8 byte order mark"
        );
        // So are the UTF-16 byte order marks, whose bytes are not ASCII.
        let text = format!("{declaration}{RULE}");
        let units = || text.encode_utf16();
        let little = [
            b"\xFF\xFE".to_vec(),
            units().flat_map(u16::to_le_bytes).collect(),
        ]
        .concat();
        let big = [
            b"\xFE\xFF".to_vec(),
            units().flat_map(u16::to_be_bytes).collect(),
        ]
        .concat();
        for (bytes, name) in [(little, "UTF-16LE"), (big, "UTF-16BE")] {
            let decoded = decode(&bytes, DecodeHints::default());
            assert_eq!(decoded.encoding().name(), name);
            assert_eq!(
                observe(&CssParser::default().parse_sheet_bytes(&decoded)),
                expected,
                "{label} in {name}"
            );
        }
    }
}

#[test]
fn the_declaration_is_consumed_whichever_source_selected_the_encoding() {
    // A protocol label decides over the declaration, the declaration decides
    // over the environment encoding, and a byte order mark decides over both;
    // the declaration is the leading declaration of the text in each case.
    let bytes = b"@charset \"windows-1252\";.a{color:red}";
    let protocol = DecodeHints {
        protocol_label: Some("utf-8"),
        environment_encoding: None,
    };
    let environment = DecodeHints {
        protocol_label: None,
        environment_encoding: Encoding::for_label("windows-1251"),
    };
    let with_mark = [&b"\xEF\xBB\xBF"[..], bytes].concat();
    for (bytes, hints, selection) in [
        (&bytes[..], protocol, EncodingSelection::Protocol),
        (&bytes[..], environment, EncodingSelection::CharsetPrefix),
        (
            &with_mark[..],
            environment,
            EncodingSelection::ByteOrderMark,
        ),
    ] {
        let input = decode(bytes, hints);
        assert_eq!(input.selection(), selection);
        let report = CssParser::default().parse_sheet_bytes(&input);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert!(report.into_validation_result().is_ok());
    }
}

#[test]
fn decoded_bytes_and_the_same_text_give_the_same_report() {
    let bytes = b"@charset \"windows-1252\";.a{content:\"\x80\";mystery:1}@charset \"x\";";
    let input = decode(bytes, DecodeHints::default());
    assert_eq!(input.selection(), EncodingSelection::CharsetPrefix);
    let from_bytes = CssParser::default().parse_sheet_bytes(&input);
    let from_text = CssParser::default().parse_sheet(input.source().as_str());
    assert_eq!(observe(&from_bytes), observe(&from_text));
    assert_eq!(from_bytes.diagnostics(), from_text.diagnostics());
    // The unknown property and the later `@charset` are the only diagnostics.
    let mut found: Vec<_> = from_bytes
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.span().start().byte_offset().value(),
                diagnostic.error().code(),
            )
        })
        .collect();
    found.sort_by_key(|(start, _)| *start);
    // 24 bytes of declaration, `.a{content:"` 12 bytes, the euro sign 3 and
    // `";` 2 put `mystery` at 41; `mystery:1}` is 10 bytes.
    assert_eq!(
        found,
        [
            (41, CssErrorCode::UnknownProperty),
            (51, CssErrorCode::UnknownAtRule)
        ]
    );
}

#[test]
fn a_byte_order_mark_character_in_text_is_not_removed_so_it_precedes_the_declaration() {
    // Only decoding removes a byte order mark; a U+FEFF in a string front is
    // text before `@charset`, so the declaration is not at offset 0.
    let source = "\u{feff}@charset \"utf-8\";.a{color:red}";
    assert!(!text_front(source).clean);
    assert_eq!(text_front(source), utf16_front(source));
}

#[test]
fn every_other_spelling_keeps_exactly_one_unknown_at_rule_diagnostic() {
    // Two spaces, single quotes, a space before the semicolon, a different
    // case, an escape in the name, a non-ASCII label and a comment or
    // whitespace before the declaration are not the pattern. Each drop spans
    // the whole at-rule, which ends at its semicolon.
    for (source, dropped, styles) in [
        ("@charset  \"utf-8\";.a{color:red}", (0, 18), 18),
        ("@charset 'utf-8';.a{color:red}", (0, 17), 17),
        ("@charset \"utf-8\" ;.a{color:red}", (0, 18), 18),
        ("@CHARSET \"utf-8\";.a{color:red}", (0, 17), 17),
        ("@\\63harset \"utf-8\";.a{color:red}", (0, 19), 19),
        ("@charset\t\"utf-8\";.a{color:red}", (0, 17), 17),
        ("@charset \"\u{e9}\";.a{color:red}", (0, 14), 14),
        (" @charset \"utf-8\";.a{color:red}", (1, 18), 18),
        ("\n@charset \"utf-8\";.a{color:red}", (1, 18), 18),
        ("/**/@charset \"utf-8\";.a{color:red}", (4, 21), 21),
    ] {
        kept(source, dropped, &[styles]);
    }
}

#[test]
fn a_declaration_without_a_semicolon_keeps_the_diagnostic() {
    kept("@charset \"utf-8\"", (0, 16), &[]);
    kept("@charset \"utf-8\".a{color:red}", (0, 29), &[]);
}

#[test]
fn only_the_first_of_two_declarations_is_consumed() {
    kept(
        "@charset \"utf-8\";@charset \"utf-8\";.a{color:red}",
        (17, 34),
        &[34],
    );
}

#[test]
fn a_declaration_after_any_other_text_keeps_the_diagnostic() {
    kept(
        ".b{color:red}@charset \"utf-8\";.a{color:red}",
        (13, 30),
        &[0, 30],
    );
}

#[test]
fn the_declaration_is_recognized_only_within_the_first_1024_bytes() {
    // `@charset "` is 10 bytes and `";` is 2, so a label of 1012 bytes makes
    // a declaration of 1024 bytes, which is within the bound; one more byte
    // is not.
    let within = format!("@charset \"{}\";", "a".repeat(1012));
    assert_eq!(within.len(), 1024);
    consumed(&within);
    let beyond = format!("@charset \"{}\";{RULE}", "a".repeat(1013));
    assert_eq!(beyond.len(), 1025 + RULE.len());
    kept(&beyond, (0, 1025), &[1025]);
}

#[test]
fn a_declaration_inside_a_longer_text_does_not_hide_a_later_unit() {
    // A label containing an opening brace or a line break does not open a
    // block or end the declaration: the pattern, not the tokens, decides.
    consumed("@charset \"{\";");
    consumed("@charset \"a\nb\";");
}

#[test]
fn single_rule_fronts_keep_reporting_the_charset_declaration() {
    let source = "@charset \"utf-8\";";
    let report = CssRule::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
    );
    assert!(report.syntax().is_none(), "{report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejection: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

#[test]
fn block_contents_keep_reporting_the_charset_declaration() {
    // A nested at-rule of block contents is dropped with one diagnostic; the
    // declaration is no exception at the start of the contents. An unknown
    // at-rule of the same length is the reference for the report.
    let parser = CssParser::default();
    let reference = parser.parse_block_contents("@charsex \"utf-8\";color:red");
    let [expected] = reference.diagnostics() else {
        panic!("one dropped at-rule: {reference:?}")
    };
    let source = "@charset \"utf-8\";color:red";
    let report = parser.parse_block_contents(source);
    assert!(!report.is_clean(), "{report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one dropped at-rule: {report:?}")
    };
    assert_eq!(diagnostic.error().code(), expected.error().code());
    assert_eq!(diagnostic.action(), expected.action());
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 17);
    assert_eq!(report.syntax().len(), 1);
    let units: Vec<u16> = source.encode_utf16().collect();
    let (_, utf16) = parser
        .parse_block_contents_utf16(Arc::new(units), CssUtf16InputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .expect("an unlimited UTF-16 preparation succeeds")
        .into_parts();
    let [diagnostic] = utf16.diagnostics() else {
        panic!("one dropped at-rule: {utf16:?}")
    };
    assert_eq!(diagnostic.error().code(), expected.error().code());
    assert_eq!(diagnostic.span().end().byte_offset().value(), 17);
}

#[test]
fn nested_rule_lists_and_style_blocks_keep_reporting_the_charset_declaration() {
    // The declaration is the first unit of a block, not of the stylesheet.
    for (source, start, end) in [
        ("a{@charset \"utf-8\";color:red}", 2, 19),
        ("@media all{@charset \"utf-8\";.a{color:red}}", 11, 28),
        ("@layer{@charset \"utf-8\";.a{color:red}}", 7, 24),
    ] {
        let observed = all_fronts(source);
        assert_eq!(
            observed.diagnostics,
            [(
                CssErrorCode::UnknownAtRule,
                CssRecoveryAction::DropAtRule,
                start,
                end
            )],
            "{source:?}"
        );
        assert!(!observed.clean, "{source:?}");
    }
}

// ---- Preprocessing differences of the UTF-16 front ---------------------------
//
// The text and byte fronts read the text as given, with CR, FF and NUL still in
// it (section 3.3 filters them in the tokenizer). The UTF-16 front preprocesses
// first: CR, FF and CRLF become one LF and NUL becomes U+FFFD, which is not
// ASCII. The declaration is found in the text each front parses, so the fronts
// agree except where that preprocessing changes the pattern's input.

#[test]
fn a_nul_in_the_label_is_the_pattern_in_text_and_bytes_but_not_after_utf16_preprocessing() {
    // 0x00 is an ASCII byte other than the quotation mark, so the declaration
    // is 13 bytes of text; after preprocessing the label is U+FFFD (3 bytes of
    // UTF-8), which is not ASCII, so the UTF-16 front sees no declaration.
    let source = "@charset \"\0\";.a{color:red}";
    let consumed = Observed {
        clean: true,
        diagnostics: Vec::new(),
        style_starts: vec![13],
        rule_count: 1,
    };
    assert_eq!(text_front(source), consumed);
    assert_eq!(
        byte_front(source.as_bytes(), DecodeHints::default()),
        consumed
    );
    // The normalized text is `@charset "\u{fffd}";`, 15 bytes.
    assert_eq!(
        utf16_front(source),
        Observed {
            clean: false,
            diagnostics: vec![(
                CssErrorCode::UnknownAtRule,
                CssRecoveryAction::DropAtRule,
                0,
                15
            )],
            style_starts: vec![15],
            rule_count: 1,
        }
    );
}

#[test]
fn a_line_break_or_form_feed_in_the_label_is_consumed_by_every_front() {
    // CR, FF and LF are ASCII before and after preprocessing, and CRLF is two
    // ASCII bytes of text and one after it, so each front finds a declaration.
    for (label, text_length, utf16_length) in [
        ("\n", 13, 13),
        ("\r", 13, 13),
        ("\x0c", 13, 13),
        ("\r\n", 14, 13),
    ] {
        let source = format!("@charset \"{label}\";{RULE}");
        let text = text_front(&source);
        assert!(text.clean, "{label:?}: {text:?}");
        assert_eq!(text.style_starts, [text_length], "{label:?}");
        assert_eq!(
            byte_front(source.as_bytes(), DecodeHints::default()),
            text,
            "{label:?}"
        );
        let utf16 = utf16_front(&source);
        assert!(utf16.clean, "{label:?}: {utf16:?}");
        assert_eq!(utf16.style_starts, [utf16_length], "{label:?}");
        assert_eq!(utf16.rule_count, 1, "{label:?}");
    }
}

#[test]
fn a_crlf_in_the_label_can_bring_the_declaration_inside_the_window_only_for_utf16() {
    // The declaration is 1025 bytes of text, so its semicolon is outside the
    // first 1024 bytes of the text and byte fronts. The UTF-16 front
    // preprocesses CRLF to LF first, which makes it 1024 bytes of text.
    let label = format!("{}\r\n", "a".repeat(1011));
    let source = format!("@charset \"{label}\";{RULE}");
    assert_eq!(source.len(), 1025 + RULE.len());
    let text = text_front(&source);
    assert!(!text.clean, "{text:?}");
    assert_eq!(byte_front(source.as_bytes(), DecodeHints::default()), text);
    let utf16 = utf16_front(&source);
    assert!(utf16.clean, "{utf16:?}");
    assert_eq!(utf16.style_starts, [1024]);
    // Without the CRLF the fronts agree at the same length.
    let label = "a".repeat(1013);
    let source = format!("@charset \"{label}\";{RULE}");
    assert!(!text_front(&source).clean);
    assert_eq!(utf16_front(&source), text_front(&source));
}

// ---- Coordinates after the declaration ---------------------------------------

/// Every position a report holds, as `(byte offset, line, UTF-16 column)`:
/// each diagnostic's error position and span ends, then each style rule's start.
type Position = (usize, u32, u32);

fn positions(report: &CssParseReport<CssSheet>) -> Vec<Position> {
    let at = |position: surgeist_css::source::CssSourcePosition| {
        (
            position.byte_offset().value(),
            position.line().value(),
            position.column().value(),
        )
    };
    let mut found = Vec::new();
    for diagnostic in report.diagnostics() {
        found.push(at(diagnostic.error().position()));
        found.push(at(diagnostic.span().start()));
        found.push(at(diagnostic.span().end()));
    }
    for rule in report.syntax().rules() {
        if let CssRule::Style(style) = rule {
            found.push(at(style.position()));
        }
    }
    found
}

/// The UTF-16 unit ranges of the original input that the diagnostics' spans map
/// to, for a report of the UTF-16 front.
fn unit_ranges(report: &CssParseReport<CssSheet>) -> Vec<Option<std::ops::Range<usize>>> {
    let source = report
        .syntax()
        .parse_source()
        .expect("a parsed sheet has a source")
        .source();
    report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            let span = diagnostic.span();
            source.original_utf16_range(
                span.start().byte_offset().value()..span.end().byte_offset().value(),
            )
        })
        .collect()
}

#[test]
fn coordinates_after_the_declaration_equal_those_of_the_same_text_with_it_blanked() {
    // An emoji (4 bytes, 2 UTF-16 units) and an accented letter (2 bytes, 1
    // unit) precede an unknown property and a retained rule on later lines.
    const TAIL: &str =
        "/*\u{1f600}\u{e9}*/\n.a{content:\"\u{e9}\u{1f600}\";mystery:1}\n.b{color:red}";
    let declaration = "@charset \"utf-8\";";
    let blanked = format!("{}{TAIL}", " ".repeat(declaration.len()));
    let declared = format!("{declaration}{TAIL}");

    // Offsets by hand, from the end of the 17-byte declaration: the comment
    // is 10 bytes (7 units) and a line break, so `.a` starts at byte 28 on
    // line 1; its 12 characters up to the string, the string's 6 bytes (3
    // units) and `";` put `mystery` at byte 48, column 17, and `mystery:1`
    // ends at byte 57, column 26; `.b` starts line 2 at byte 59.
    let expected: Vec<Position> = vec![
        (48, 1, 17),
        (48, 1, 17),
        (57, 1, 26),
        (28, 1, 0),
        (59, 2, 0),
    ];

    let parser = CssParser::default();
    let units = |source: &str| -> Arc<Vec<u16>> { Arc::new(source.encode_utf16().collect()) };
    let utf16 = |source: &str| {
        parser
            .parse_sheet_utf16(units(source), CssUtf16InputLimits::default(), || {
                Ok::<_, ()>(())
            })
            .expect("an unlimited UTF-16 preparation succeeds")
    };
    let bytes =
        |source: &str| parser.parse_sheet_bytes(&decode(source.as_bytes(), DecodeHints::default()));

    for (name, with_declaration, with_spaces) in [
        (
            "text",
            parser.parse_sheet(&declared),
            parser.parse_sheet(&blanked),
        ),
        ("bytes", bytes(&declared), bytes(&blanked)),
        ("UTF-16", utf16(&declared), utf16(&blanked)),
    ] {
        assert_eq!(positions(&with_declaration), expected, "{name}");
        assert_eq!(
            positions(&with_declaration),
            positions(&with_spaces),
            "{name}"
        );
        let codes: Vec<_> = with_declaration
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.error().code())
            .collect();
        assert_eq!(codes, [CssErrorCode::UnknownProperty], "{name}");
        assert_eq!(
            with_declaration.diagnostics(),
            with_spaces.diagnostics(),
            "{name}"
        );
    }

    // The UTF-16 front also maps each span to the original units: the
    // declaration is 17 units, the comment 7, the line break 1, then 12 units
    // before the string, its 3 units and `";`, so `mystery` is unit 42 and its
    // declaration ends at unit 51, with or without the declaration.
    let with_declaration = utf16(&declared);
    let with_spaces = utf16(&blanked);
    assert_eq!(unit_ranges(&with_declaration), [Some(42..51)]);
    assert_eq!(unit_ranges(&with_declaration), unit_ranges(&with_spaces));
    // The original units are those of the whole input, declaration included.
    let source = with_declaration.syntax().parse_source().unwrap().source();
    assert_eq!(
        source.original_utf16().unwrap().as_slice(),
        declared.encode_utf16().collect::<Vec<_>>()
    );
}

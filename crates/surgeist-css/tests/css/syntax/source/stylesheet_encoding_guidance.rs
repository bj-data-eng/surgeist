//! CSS Syntax 3 (2026-10-01), section 3.2 "The input byte stream": the
//! encoding guidance an author or a host adapter follows, observed through the
//! public byte front (`CssDecodedStylesheetInput::decode`,
//! `CssParser::parse_sheet_bytes`).
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#input-byte-stream>
//!
//! The expectations are spelled from the clause text and from the Encoding
//! Standard's tables (windows-1252 byte 0x80 is U+20AC, windows-1251 byte 0xCF
//! is U+041F), never from a run. The fallback order, the byte limit and the
//! label rules are pinned in `stylesheet_byte_input.rs` and in the parser
//! crate; these tests pin the guidance paths themselves, the near-misses of
//! the exact declaration that the clause's note lists, bomless UTF-16, the
//! coordinates of the diagnostics of a decoded sheet and the clean report of a
//! sheet that begins with the exact declaration.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::rule::CssRule;
use surgeist_css::source::CssDecodedStylesheetInput;
use surgeist_css_parser::input::{DecodeHints, Encoding, EncodingSelection};

fn decode(bytes: &[u8], hints: DecodeHints<'_>) -> CssDecodedStylesheetInput {
    CssDecodedStylesheetInput::decode(bytes, hints, Default::default())
        .expect("an unlimited decode succeeds")
}

fn protocol(label: &str) -> DecodeHints<'_> {
    DecodeHints {
        protocol_label: Some(label),
        environment_encoding: None,
    }
}

fn environment(label: &str) -> DecodeHints<'static> {
    DecodeHints {
        protocol_label: None,
        environment_encoding: Some(Encoding::for_label(label).expect("a known label")),
    }
}

/// U+00E9, U+20AC and U+1F600 in UTF-8.
const TEXT_BYTES: &[u8] = b"\xC3\xA9\xE2\x82\xAC\xF0\x9F\x98\x80";
const TEXT: &str = "\u{e9}\u{20ac}\u{1f600}";

#[test]
fn every_guided_utf8_path_selects_utf8_and_decodes_non_ascii_text() {
    // "Authors should author their stylesheets in UTF-8" and declare it by a
    // protocol label, by the referring document, by a UTF-8 BOM or by the exact
    // characters `@charset "utf-8";`; with none of these the fallback is UTF-8.
    let with = |prefix: &[u8]| [prefix, TEXT_BYTES].concat();
    let cases: [(&str, Vec<u8>, DecodeHints<'_>, EncodingSelection, String); 6] = [
        (
            "protocol",
            with(b""),
            protocol("utf-8"),
            EncodingSelection::Protocol,
            TEXT.to_owned(),
        ),
        (
            "referring document",
            with(b""),
            environment("utf-8"),
            EncodingSelection::Environment,
            TEXT.to_owned(),
        ),
        (
            "byte order mark",
            with(b"\xEF\xBB\xBF"),
            DecodeHints::default(),
            EncodingSelection::ByteOrderMark,
            TEXT.to_owned(),
        ),
        (
            "exact declaration",
            with(b"@charset \"utf-8\";"),
            DecodeHints::default(),
            EncodingSelection::CharsetPrefix,
            format!("@charset \"utf-8\";{TEXT}"),
        ),
        (
            "no information",
            with(b""),
            DecodeHints::default(),
            EncodingSelection::DefaultUtf8,
            TEXT.to_owned(),
        ),
        (
            "protocol label with ASCII whitespace and case",
            with(b""),
            protocol(" \tUTF-8\n"),
            EncodingSelection::Protocol,
            TEXT.to_owned(),
        ),
    ];
    for (path, bytes, hints, selection, text) in cases {
        let input = decode(&bytes, hints);
        assert_eq!(input.encoding().name(), "UTF-8", "{path}");
        assert_eq!(input.selection(), selection, "{path}");
        assert_eq!(input.source().as_str(), text, "{path}");
        assert!(!input.had_replacements(), "{path}");
        assert_eq!(input.original_bytes(), bytes, "{path}");
    }
}

#[test]
fn a_legacy_encoded_sheet_without_a_hint_is_not_guessed_as_legacy() {
    // Step 4 returns utf-8: a new format that supplies no environment encoding
    // gets UTF-8, so windows-1252 byte 0xE9 is a malformed UTF-8 sequence.
    let bytes = b"a{content:\"\xE9\"}";
    let unhinted = decode(bytes, DecodeHints::default());
    assert_eq!(unhinted.selection(), EncodingSelection::DefaultUtf8);
    assert_eq!(unhinted.encoding().name(), "UTF-8");
    assert_eq!(unhinted.source().as_str(), "a{content:\"\u{fffd}\"}");
    assert!(unhinted.had_replacements());
    // Only an explicit hint, here the legacy environment encoding of step 3,
    // reads the same bytes as windows-1252.
    let hinted = decode(bytes, environment("windows-1252"));
    assert_eq!(hinted.selection(), EncodingSelection::Environment);
    assert_eq!(hinted.source().as_str(), "a{content:\"\u{e9}\"}");
    assert!(!hinted.had_replacements());
}

#[test]
fn the_exact_utf8_declaration_decides_over_a_legacy_environment_encoding() {
    // The discriminating byte is 0x80: U+20AC in windows-1252, malformed UTF-8.
    for label in ["utf-8", "UTF-8", "Utf-8", "utf8", "unicode-1-1-utf-8"] {
        let bytes = [format!("@charset \"{label}\";").as_bytes(), b"\x80"].concat();
        let input = decode(&bytes, environment("windows-1252"));
        assert_eq!(input.encoding().name(), "UTF-8", "{label}");
        assert_eq!(
            input.selection(),
            EncodingSelection::CharsetPrefix,
            "{label}"
        );
        assert_eq!(
            input.source().as_str(),
            format!("@charset \"{label}\";\u{fffd}"),
            "{label}"
        );
        assert!(input.had_replacements(), "{label}");
    }
}

#[test]
fn near_misses_of_the_exact_declaration_do_not_select_an_encoding() {
    // "multiple spaces, comments, or single quotes will cause the encoding
    // declaration to not be recognized". Each near-miss leaves the legacy
    // environment encoding in charge, so 0x80 decodes as U+20AC.
    for bytes in [
        &b"@charset\t\"utf-8\";\x80"[..],
        &b"@charset\"utf-8\";\x80"[..],
        &b"@charset\r\n\"utf-8\";\x80"[..],
        &b"@charset \"utf-8\"\r\n;\x80"[..],
        &b"@charset \"utf-8\"\";\x80"[..],
        &b"@charset \"utf-8\\\";\x80"[..],
        &b"@charset \"utf-8';\x80"[..],
        &b"\0@charset \"utf-8\";\x80"[..],
        "@charset \u{201c}utf-8\u{201d};\u{80}".as_bytes(),
    ] {
        let input = decode(bytes, environment("windows-1252"));
        let shown = String::from_utf8_lossy(bytes);
        assert_eq!(input.selection(), EncodingSelection::Environment, "{shown}");
        assert_eq!(input.encoding().name(), "windows-1252", "{shown}");
    }
}

#[test]
fn a_utf16_declaration_or_sheet_without_a_byte_order_mark_is_not_recognized() {
    // Syntax 3 §3.2 step 2 matches ASCII bytes, which UTF-16 is not, so a
    // declaration written in UTF-16 does not select windows-1250, and a sheet
    // without a BOM is decoded by the fallback. The ASCII-only text is the
    // same under UTF-8 and windows-1251, so the NULs interleave unchanged.
    let text = "@charset \"windows-1250\";a{}";
    let little: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let big: Vec<u8> = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
    let little_text: String = text.chars().flat_map(|c| [c, '\0']).collect();
    let big_text: String = text.chars().flat_map(|c| ['\0', c]).collect();
    for (bytes, expected) in [(&little, &little_text), (&big, &big_text)] {
        let unhinted = decode(bytes, DecodeHints::default());
        assert_eq!(unhinted.selection(), EncodingSelection::DefaultUtf8);
        assert_eq!(unhinted.encoding().name(), "UTF-8");
        assert_eq!(unhinted.source().as_str(), expected.as_str());
        assert!(!unhinted.had_replacements());
        let legacy = decode(bytes, environment("windows-1251"));
        assert_eq!(legacy.selection(), EncodingSelection::Environment);
        assert_eq!(legacy.encoding().name(), "windows-1251");
        assert_eq!(legacy.source().as_str(), expected.as_str());
    }
    // The same text with a BOM is UTF-16, whatever the hints and its content.
    let bom: Vec<u8> = [&b"\xFF\xFE"[..], &little].concat();
    let with_bom = decode(&bom, environment("windows-1251"));
    assert_eq!(with_bom.selection(), EncodingSelection::ByteOrderMark);
    assert_eq!(with_bom.encoding().name(), "UTF-16LE");
    assert_eq!(with_bom.source().as_str(), text);
}

#[test]
fn diagnostics_of_a_legacy_decoded_sheet_use_decoded_source_coordinates() {
    // Two windows-1252 bytes 0x80 decode to two U+20AC of three UTF-8 bytes
    // each, so every later offset is four bytes past the original byte index.
    let bytes = b"a{content:\"\x80\x80\";mystery:1;color:red}";
    let mystery_in_bytes = 15;
    assert_eq!(&bytes[mystery_in_bytes..mystery_in_bytes + 7], b"mystery");
    let input = decode(bytes, protocol("windows-1252"));
    assert_eq!(
        input.source().as_str(),
        "a{content:\"\u{20ac}\u{20ac}\";mystery:1;color:red}"
    );
    let report = CssParser::default().parse_sheet_bytes(&input);
    let [diagnostic] = report.diagnostics() else {
        panic!("one unknown property: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let position = diagnostic.error().position();
    assert_eq!(position.byte_offset().value(), 19);
    assert_eq!(position.line().value(), 0);
    assert_eq!(position.column().value(), 15);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 19);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 29);
    assert_eq!(&input.source().as_str()[19..29], "mystery:1;");
    // The original bytes keep their own indices.
    assert_eq!(&input.original_bytes()[15..25], b"mystery:1;");
}

#[test]
fn a_leading_declaration_is_consumed_and_later_coordinates_keep_their_decoded_values() {
    // §3.2 calls the declaration an encoding declaration that only looks like
    // an at-rule, and §8.3 says no `@charset` rule exists. The exact leading
    // declaration yields no rule and no diagnostic; decoding keeps it in the
    // text, so every later coordinate is shifted by its 24 bytes.
    let declaration = b"@charset \"windows-1252\";";
    let bytes = [
        &declaration[..],
        b"a{content:\"\x80\x80\";mystery:1;color:red}",
    ]
    .concat();
    let input = decode(&bytes, DecodeHints::default());
    assert_eq!(input.selection(), EncodingSelection::CharsetPrefix);
    let report = CssParser::default().parse_sheet_bytes(&input);
    let [property] = report.diagnostics() else {
        panic!(
            "only the unknown property is reported: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(property.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(property.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(property.error().position().byte_offset().value(), 43);
    assert_eq!(property.error().position().column().value(), 39);
    let [CssRule::Style(style)] = report.syntax().rules() else {
        panic!("the style rule follows the declaration")
    };
    assert_eq!(style.position().byte_offset().value(), 24);
    // `content` and `color` stay; `mystery` is the dropped declaration.
    assert_eq!(style.declarations().len(), 2);
}

#[test]
fn a_clean_stylesheet_may_begin_with_the_leading_declaration() {
    // The same valid rule is clean through a protocol label, a BOM, the
    // environment encoding, the UTF-8 fallback and the exact leading
    // declaration, which is consumed (§3.2), so `into_validation_result`
    // accepts each of them.
    let rule = b"a{color:red}";
    let declared = decode(
        &[&b"@charset \"utf-8\";"[..], rule].concat(),
        DecodeHints::default(),
    );
    assert_eq!(declared.selection(), EncodingSelection::CharsetPrefix);
    let declared_bytes = declared.original_bytes().to_vec();
    for (path, bytes, hints) in [
        ("protocol", rule.to_vec(), protocol("utf-8")),
        ("environment", rule.to_vec(), environment("utf-8")),
        (
            "byte order mark",
            [&b"\xEF\xBB\xBF"[..], rule].concat(),
            DecodeHints::default(),
        ),
        ("fallback", rule.to_vec(), DecodeHints::default()),
        (
            "declaration",
            declared_bytes.clone(),
            DecodeHints::default(),
        ),
        (
            "byte order mark and declaration",
            [&b"\xEF\xBB\xBF"[..], &declared_bytes].concat(),
            DecodeHints::default(),
        ),
    ] {
        let report = CssParser::default().parse_sheet_bytes(&decode(&bytes, hints));
        assert!(report.is_clean(), "{path}: {:?}", report.diagnostics());
        assert!(report.into_validation_result().is_ok(), "{path}");
    }
}

#[test]
fn the_how_to_example_decodes_a_protocol_declared_sheet_without_a_diagnostic() {
    // The call sequence of docs/how-to.md, "Decode a stylesheet from bytes".
    let hints = DecodeHints {
        protocol_label: Some("utf-8"),
        environment_encoding: None,
    };
    let input = CssDecodedStylesheetInput::decode(
        "a { content: \"\u{e9}\" }".as_bytes(),
        hints,
        Default::default(),
    )
    .expect("within the limits");
    assert_eq!(input.selection(), EncodingSelection::Protocol);
    assert_eq!(input.encoding().name(), "UTF-8");
    assert!(CssParser::default().parse_sheet_bytes(&input).is_clean());
}

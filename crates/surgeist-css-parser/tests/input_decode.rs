//! Tests for `input::decode`, `Encoding` and the hints, limits and result types
//! (CSS Syntax 3 section 3.2, "The input byte stream").
//!
//! The expectations come from section 3.2 of the pinned edition and from the
//! Encoding Standard's labels, indexes and "decode" algorithm, which the
//! `encoding_rs` crate implements. The section has no WPT backing (its
//! `charset/*` tests observe HTTP headers and HTML documents), so these tests
//! are its whole evidence. Each expected text is spelled out by hand from the
//! standard's tables, never taken from a run.

use surgeist_css_parser::input::{
    DecodeError, DecodeHints, DecodeLimits, Decoded, Encoding, EncodingSelection, decode,
};

/// Decodes with the given hints and no size limit.
fn decoded(bytes: &[u8], hints: DecodeHints<'_>) -> Decoded {
    decode(bytes, hints, DecodeLimits::default()).expect("an unlimited decode succeeds")
}

fn no_hints() -> DecodeHints<'static> {
    DecodeHints::default()
}

fn protocol(label: &str) -> DecodeHints<'_> {
    DecodeHints {
        protocol_label: Some(label),
        ..DecodeHints::default()
    }
}

fn environment(label: &str) -> DecodeHints<'static> {
    DecodeHints {
        environment_encoding: Some(Encoding::for_label(label).expect("a known label")),
        ..DecodeHints::default()
    }
}

// ---- Labels (Encoding Standard, "get an encoding") -------------------------

#[test]
fn labels_find_the_encodings_the_encoding_standard_registers_them_for() {
    // (label, canonical name), written from the standard's table of encodings.
    let table = [
        ("utf-8", "UTF-8"),
        ("UTF8", "UTF-8"),
        ("unicode-1-1-utf-8", "UTF-8"),
        ("latin1", "windows-1252"),
        ("ascii", "windows-1252"),
        ("us-ascii", "windows-1252"),
        ("iso-8859-1", "windows-1252"),
        ("cp1251", "windows-1251"),
        ("windows-1251", "windows-1251"),
        ("sjis", "Shift_JIS"),
        ("ms_kanji", "Shift_JIS"),
        ("gb2312", "GBK"),
        ("gbk", "GBK"),
        ("big5", "Big5"),
        ("euc-kr", "EUC-KR"),
        ("iso-2022-jp", "ISO-2022-JP"),
        ("macintosh", "macintosh"),
        ("x-user-defined", "x-user-defined"),
        ("utf-16", "UTF-16LE"),
        ("ucs-2", "UTF-16LE"),
        ("unicode", "UTF-16LE"),
        ("utf-16le", "UTF-16LE"),
        ("utf-16be", "UTF-16BE"),
        ("unicodefffe", "UTF-16BE"),
        // The replacement encoding is valid and has its own labels.
        ("replacement", "replacement"),
        ("iso-2022-kr", "replacement"),
        ("hz-gb-2312", "replacement"),
        ("iso-2022-cn", "replacement"),
        ("csiso2022kr", "replacement"),
    ];
    for (label, name) in table {
        assert_eq!(
            Encoding::for_label(label).map(Encoding::name),
            Some(name),
            "label {label:?}"
        );
    }
}

#[test]
fn a_label_ignores_ascii_case_and_surrounding_ascii_whitespace() {
    // The standard removes leading and trailing ASCII whitespace (tab, line
    // feed, form feed, carriage return, space) and lowercases ASCII.
    for label in [
        "UTF-8",
        "Utf-8",
        " utf-8",
        "utf-8 ",
        "\t\n\x0c\r utf-8\t\n\x0c\r ",
    ] {
        assert_eq!(
            Encoding::for_label(label).map(Encoding::name),
            Some("UTF-8"),
            "label {label:?}"
        );
    }
}

#[test]
fn a_label_that_names_no_encoding_is_refused() {
    for label in [
        "",
        " ",
        "utf-9",
        "utf_8",
        "utf-8 x",
        "utf 8",
        "utf-8\0",
        // Whitespace that is not ASCII whitespace is not removed.
        "\u{a0}utf-8",
        // Fullwidth letters are not ASCII letters.
        "\u{ff55}\u{ff54}\u{ff46}-8",
        "unknown-encoding",
    ] {
        assert_eq!(Encoding::for_label(label), None, "label {label:?}");
    }
}

#[test]
fn encodings_compare_by_identity_and_are_copyable_values() {
    let first = Encoding::for_label("latin1").unwrap();
    let second = Encoding::for_label("iso-8859-1").unwrap();
    assert_eq!(first, second);
    assert_ne!(first, Encoding::for_label("utf-8").unwrap());
    let copy = first;
    assert_eq!(copy, first);
}

// ---- The fallback chain (3.2, "determine the fallback encoding") ----------

#[test]
fn a_valid_protocol_label_selects_the_fallback_before_anything_else() {
    // The declaration says windows-1251 and the environment says utf-8, but
    // the protocol says windows-1252, so 0xCF is U+00CF.
    let bytes = b"@charset \"windows-1251\"; \xCF";
    let hints = DecodeHints {
        protocol_label: Some("windows-1252"),
        environment_encoding: Encoding::for_label("utf-8"),
    };
    let result = decoded(bytes, hints);
    assert_eq!(result.encoding().name(), "windows-1252");
    assert_eq!(result.selection(), EncodingSelection::Protocol);
    assert_eq!(result.text(), "@charset \"windows-1251\"; \u{cf}");
}

#[test]
fn a_protocol_label_that_names_no_encoding_is_as_good_as_none() {
    // Step 1 returns only when "get an encoding" does not return failure.
    let bytes = b"@charset \"windows-1251\"; \xCF";
    let result = decoded(bytes, protocol("no-such-encoding"));
    assert_eq!(result.encoding().name(), "windows-1251");
    assert_eq!(result.selection(), EncodingSelection::CharsetPrefix);
    assert_eq!(result.text(), "@charset \"windows-1251\"; \u{41f}");
}

#[test]
fn an_invalid_protocol_label_falls_through_to_the_environment_and_then_to_utf8() {
    let result = decoded(
        b"a\xCF",
        DecodeHints {
            protocol_label: Some(""),
            environment_encoding: Encoding::for_label("windows-1251"),
        },
    );
    assert_eq!(result.selection(), EncodingSelection::Environment);
    assert_eq!(result.text(), "a\u{41f}");
    let result = decoded(b"a\xCF", protocol("bogus"));
    assert_eq!(result.selection(), EncodingSelection::DefaultUtf8);
    assert_eq!(result.text(), "a\u{fffd}");
}

#[test]
fn a_charset_declaration_selects_the_fallback_before_the_environment() {
    let result = decoded(
        b"@charset \"windows-1251\";\xCF",
        environment("windows-1252"),
    );
    assert_eq!(result.encoding().name(), "windows-1251");
    assert_eq!(result.selection(), EncodingSelection::CharsetPrefix);
    assert_eq!(result.text(), "@charset \"windows-1251\";\u{41f}");
}

#[test]
fn the_environment_encoding_is_the_fallback_when_nothing_earlier_applies() {
    let result = decoded(b"a\x80", environment("windows-1252"));
    assert_eq!(result.encoding().name(), "windows-1252");
    assert_eq!(result.selection(), EncodingSelection::Environment);
    // 0x80 is U+20AC in the Encoding Standard's windows-1252 index.
    assert_eq!(result.text(), "a\u{20ac}");
}

#[test]
fn utf8_is_the_fallback_when_no_hint_or_declaration_applies() {
    let result = decoded("a\u{20ac}".as_bytes(), no_hints());
    assert_eq!(result.encoding().name(), "UTF-8");
    assert_eq!(result.selection(), EncodingSelection::DefaultUtf8);
    assert_eq!(result.text(), "a\u{20ac}");
    assert!(!result.had_replacements());
}

#[test]
fn empty_input_decodes_to_empty_text_with_the_fallback_that_applies() {
    let result = decoded(b"", no_hints());
    assert_eq!(result.text(), "");
    assert_eq!(result.selection(), EncodingSelection::DefaultUtf8);
    let result = decoded(b"", protocol("windows-1252"));
    assert_eq!(result.text(), "");
    assert_eq!(result.selection(), EncodingSelection::Protocol);
    assert!(!result.had_replacements());
}

// ---- Byte order marks ("decode" gives them precedence) ---------------------

#[test]
fn a_utf8_byte_order_mark_overrides_every_fallback_and_is_not_text() {
    let bytes = b"\xEF\xBB\xBF@charset \"windows-1251\"; \xD0\x9F";
    let hints = DecodeHints {
        protocol_label: Some("windows-1252"),
        environment_encoding: Encoding::for_label("windows-1251"),
    };
    let result = decoded(bytes, hints);
    assert_eq!(result.encoding().name(), "UTF-8");
    assert_eq!(result.selection(), EncodingSelection::ByteOrderMark);
    // U+041F is D0 9F in UTF-8; the declaration is only text here.
    assert_eq!(result.text(), "@charset \"windows-1251\"; \u{41f}");
}

#[test]
fn a_utf16_little_endian_byte_order_mark_selects_that_encoding() {
    let result = decoded(b"\xFF\xFEa\0\xE9\0", protocol("windows-1252"));
    assert_eq!(result.encoding().name(), "UTF-16LE");
    assert_eq!(result.selection(), EncodingSelection::ByteOrderMark);
    assert_eq!(result.text(), "a\u{e9}");
}

#[test]
fn a_utf16_big_endian_byte_order_mark_selects_that_encoding() {
    let result = decoded(b"\xFE\xFF\0a\0\xE9", environment("windows-1252"));
    assert_eq!(result.encoding().name(), "UTF-16BE");
    assert_eq!(result.selection(), EncodingSelection::ByteOrderMark);
    assert_eq!(result.text(), "a\u{e9}");
}

#[test]
fn a_byte_order_mark_is_reported_even_when_it_agrees_with_the_fallback() {
    let result = decoded(b"\xEF\xBB\xBFa", protocol("utf-8"));
    assert_eq!(result.selection(), EncodingSelection::ByteOrderMark);
    assert_eq!(result.text(), "a");
}

#[test]
fn bytes_that_only_resemble_a_byte_order_mark_are_not_one() {
    // EF BB without the third byte, and FF/FE in other positions.
    let result = decoded(b"\xEF\xBBa", environment("windows-1252"));
    assert_eq!(result.selection(), EncodingSelection::Environment);
    assert_eq!(result.text(), "\u{ef}\u{bb}a");
    let result = decoded(b"a\xFF\xFE", environment("windows-1252"));
    assert_eq!(result.selection(), EncodingSelection::Environment);
    assert_eq!(result.text(), "a\u{ff}\u{fe}");
}

#[test]
fn a_protocol_label_for_utf16_decodes_without_a_byte_order_mark() {
    let result = decoded(b"a\0b\0", protocol("utf-16le"));
    assert_eq!(result.encoding().name(), "UTF-16LE");
    assert_eq!(result.selection(), EncodingSelection::Protocol);
    assert_eq!(result.text(), "ab");
    let result = decoded(b"\0a\0b", protocol("utf-16be"));
    assert_eq!(result.text(), "ab");
}

// ---- The encoding declaration (3.2 step 2) ---------------------------------

fn declaration_selects(bytes: &[u8], name: &str) {
    let result = decoded(bytes, no_hints());
    assert_eq!(
        (result.encoding().name(), result.selection()),
        (name, EncodingSelection::CharsetPrefix),
        "{:?}",
        String::from_utf8_lossy(bytes)
    );
}

fn declaration_is_ignored(bytes: &[u8]) {
    let result = decoded(bytes, no_hints());
    assert_eq!(
        (result.encoding().name(), result.selection()),
        ("UTF-8", EncodingSelection::DefaultUtf8),
        "{:?}",
        String::from_utf8_lossy(bytes)
    );
}

#[test]
fn a_declaration_at_the_start_selects_its_label_and_stays_in_the_text() {
    declaration_selects(b"@charset \"windows-1251\";", "windows-1251");
    declaration_selects(b"@charset \"windows-1251\"; a{}", "windows-1251");
    let result = decoded(b"@charset \"SHIFT_JIS\";\x82\xA0", no_hints());
    assert_eq!(result.encoding().name(), "Shift_JIS");
    // 82 A0 is U+3042 in the Encoding Standard's jis0208 index.
    assert_eq!(result.text(), "@charset \"SHIFT_JIS\";\u{3042}");
}

#[test]
fn a_declaration_must_match_the_byte_sequence_exactly() {
    // 40 63 68 61 72 73 65 74 20 22 XX* 22 3B, with nothing before it.
    declaration_is_ignored(b"@charset 'windows-1251';");
    declaration_is_ignored(b"@charset  \"windows-1251\";");
    declaration_is_ignored(b"@CHARSET \"windows-1251\";");
    declaration_is_ignored(b"@charset\"windows-1251\";");
    declaration_is_ignored(b" @charset \"windows-1251\";");
    declaration_is_ignored(b"\n@charset \"windows-1251\";");
    declaration_is_ignored(b"/* */@charset \"windows-1251\";");
    declaration_is_ignored(b"@charset \"windows-1251\" ;");
    declaration_is_ignored(b"@charset \"windows-1251\"");
    declaration_is_ignored(b"@charset \"windows-1251");
    declaration_is_ignored(b"@charset \"windows-1251\"\n;");
    declaration_is_ignored(b"@charset");
    declaration_is_ignored(b"");
}

#[test]
fn a_declaration_label_is_read_as_ascii_and_must_be_a_label() {
    // A byte above 0x7F is not an XX byte.
    declaration_is_ignored(b"@charset \"windows-1251\xFF\";");
    // 0x7F is an XX byte, but "windows-1251\x7F" is not a label.
    declaration_is_ignored(b"@charset \"windows-1251\x7F\";");
    // An empty or unknown label makes "get an encoding" fail.
    declaration_is_ignored(b"@charset \"\";");
    declaration_is_ignored(b"@charset \"no-such-encoding\";");
    // A control byte inside the label is an XX byte, but not part of a label.
    declaration_is_ignored(b"@charset \"windows\x01-1251\";");
}

#[test]
fn a_declaration_label_goes_through_get_an_encoding_so_ascii_whitespace_is_removed() {
    // 09 is an XX byte; the standard removes leading and trailing whitespace.
    declaration_selects(b"@charset \"\twindows-1251 \";", "windows-1251");
    declaration_selects(b"@charset \"  UTF-8\n\";", "UTF-8");
}

#[test]
fn a_declaration_of_a_utf16_encoding_means_utf8() {
    // The declaration is ASCII, which UTF-16 is not, so it "can only be lying".
    for label in [
        "utf-16be",
        "utf-16le",
        "UTF-16",
        "ucs-2",
        "unicode",
        "unicodefffe",
    ] {
        let bytes = format!("@charset \"{label}\";\u{20ac}");
        let result = decoded(bytes.as_bytes(), no_hints());
        assert_eq!(result.encoding().name(), "UTF-8", "{label}");
        assert_eq!(
            result.selection(),
            EncodingSelection::CharsetPrefix,
            "{label}"
        );
        assert_eq!(result.text(), bytes, "{label}");
    }
}

#[test]
fn a_declaration_of_the_replacement_encoding_is_valid_and_replaces_the_text() {
    // "replacement" is an encoding of the standard, so the declaration applies;
    // its decoder yields one U+FFFD for non-empty input and reports an error.
    let result = decoded(b"@charset \"iso-2022-kr\";a", no_hints());
    assert_eq!(result.encoding().name(), "replacement");
    assert_eq!(result.selection(), EncodingSelection::CharsetPrefix);
    assert_eq!(result.text(), "\u{fffd}");
    assert!(result.had_replacements());
}

#[test]
fn the_declaration_must_end_within_the_first_1024_bytes() {
    // The sequence `@charset "` (10 bytes), the label and `";` (2 bytes) must
    // all lie in the first 1024 bytes. Trailing spaces are part of the label
    // and removed by "get an encoding", so the label can be padded.
    let build = |total: usize| {
        let label = "windows-1251";
        let padding = total - 10 - label.len() - 2;
        let mut bytes = b"@charset \"".to_vec();
        bytes.extend_from_slice(label.as_bytes());
        bytes.extend(std::iter::repeat_n(b' ', padding));
        bytes.extend_from_slice(b"\";");
        bytes.extend_from_slice(b"\xCF");
        assert_eq!(bytes.len(), total + 1);
        bytes
    };
    // The semicolon is byte 1024 (one-based): the declaration is recognized.
    declaration_selects(&build(1024), "windows-1251");
    // The closing quotation mark is byte 1024 but the semicolon is byte 1025,
    // which is not within the first 1024 bytes.
    let bytes = build(1025);
    assert_eq!((bytes[1023], bytes[1024]), (b'"', b';'));
    declaration_is_ignored(&bytes);
    declaration_is_ignored(&build(1026));
}

#[test]
fn a_declaration_after_a_byte_order_mark_is_only_text() {
    let result = decoded(
        b"\xEF\xBB\xBF@charset \"windows-1251\";\xD0\x9F",
        no_hints(),
    );
    assert_eq!(result.selection(), EncodingSelection::ByteOrderMark);
    assert_eq!(result.encoding().name(), "UTF-8");
    assert_eq!(result.text(), "@charset \"windows-1251\";\u{41f}");
}

// ---- Decoding with the Encoding Standard's decoders ------------------------

#[test]
fn legacy_encodings_decode_through_the_standards_indexes() {
    // windows-1252 0x80, 0x93 and 0x94; windows-1251 0xCF; Shift_JIS 0x82A0;
    // x-user-defined 0x80 (the private-use mapping at U+F780).
    let cases: [(&str, &[u8], &str); 5] = [
        ("windows-1252", b"\x80\x93\x94", "\u{20ac}\u{201c}\u{201d}"),
        ("windows-1251", b"\xCF", "\u{41f}"),
        ("shift_jis", b"\x82\xA0", "\u{3042}"),
        ("x-user-defined", b"\x80a", "\u{f780}a"),
        ("windows-1252", b"\x81", "\u{81}"),
    ];
    for (label, bytes, text) in cases {
        let result = decoded(bytes, protocol(label));
        assert_eq!(result.text(), text, "{label}");
        assert!(!result.had_replacements(), "{label}");
    }
}

#[test]
fn malformed_bytes_become_replacement_characters_and_are_reported() {
    // A lone continuation byte, and a three-byte sequence cut short.
    let result = decoded(b"a\x80b", no_hints());
    assert_eq!(result.text(), "a\u{fffd}b");
    assert!(result.had_replacements());
    let result = decoded(b"a\xE2\x82", no_hints());
    assert_eq!(result.text(), "a\u{fffd}");
    assert!(result.had_replacements());
    // The same bytes in a single-byte encoding are not malformed.
    let result = decoded(b"a\x80b", protocol("windows-1252"));
    assert!(!result.had_replacements());
}

#[test]
fn decoding_leaves_the_code_points_that_section_3_3_filters() {
    // CR, LF, FF and NUL come out of the decoder unchanged; the filter step
    // is separate (and a byte order mark is not text).
    let result = decoded(b"a\r\nb\rc\x0cd\0e", no_hints());
    assert_eq!(result.text(), "a\r\nb\rc\x0cd\0e");
    assert!(!result.had_replacements());
}

#[test]
fn the_text_is_available_by_reference_and_by_value() {
    let result = decoded(b"abc", no_hints());
    let copy = result.clone();
    assert_eq!(result, copy);
    assert_eq!(result.into_text(), "abc");
}

#[test]
fn inputs_longer_than_one_buffer_decode_whole() {
    // 9,000 bytes cross the decoder's buffer (4,096 bytes) twice. A unit is nine
    // bytes, so 4,096 is not a multiple of it and a buffer ends inside a unit.
    let unit = "a\u{20ac}\u{10348}b";
    let text = unit.repeat(1000);
    let result = decoded(text.as_bytes(), no_hints());
    assert_eq!(result.text(), text);
    assert!(!result.had_replacements());
}

// ---- Limits ----------------------------------------------------------------

#[test]
fn the_original_byte_limit_accepts_its_exact_size_and_refuses_one_more() {
    let bytes = b"abcd";
    let exact = DecodeLimits::new(4, usize::MAX);
    assert_eq!(decode(bytes, no_hints(), exact).unwrap().text(), "abcd");
    let error = decode(bytes, no_hints(), DecodeLimits::new(3, usize::MAX)).unwrap_err();
    assert!(error.is_original_byte_limit());
    assert!(!error.is_decoded_utf8_byte_limit());
    assert!(!error.is_capacity_overflow());
    assert!(!error.is_allocation_failure());
}

#[test]
fn the_decoded_byte_limit_accepts_its_exact_size_and_refuses_one_more() {
    // Two bytes of windows-1252 decode to six bytes of UTF-8.
    let bytes = b"\x80\x80";
    let hints = protocol("windows-1252");
    let exact = DecodeLimits::new(usize::MAX, 6);
    assert_eq!(
        decode(bytes, hints, exact).unwrap().text(),
        "\u{20ac}\u{20ac}"
    );
    let error = decode(bytes, hints, DecodeLimits::new(usize::MAX, 5)).unwrap_err();
    assert!(error.is_decoded_utf8_byte_limit());
    assert!(!error.is_original_byte_limit());
}

#[test]
fn the_limits_are_independent_and_the_original_limit_is_checked_first() {
    // The decoded text would fit, but the input does not.
    let error = decode(b"\xEF\xBB\xBFab", no_hints(), DecodeLimits::new(4, 100)).unwrap_err();
    assert!(error.is_original_byte_limit());
    // The input fits (the byte order mark counts) and the text is two bytes.
    let result = decode(b"\xEF\xBB\xBFab", no_hints(), DecodeLimits::new(5, 2)).unwrap();
    assert_eq!(result.text(), "ab");
}

#[test]
fn a_decoded_limit_applies_to_the_whole_text_across_decoder_buffers() {
    let text = "a".repeat(9000);
    let exact = DecodeLimits::new(usize::MAX, 9000);
    assert_eq!(
        decode(text.as_bytes(), no_hints(), exact).unwrap().text(),
        text
    );
    let error = decode(
        text.as_bytes(),
        no_hints(),
        DecodeLimits::new(usize::MAX, 8999),
    )
    .unwrap_err();
    assert!(error.is_decoded_utf8_byte_limit());
}

#[test]
fn the_default_limits_impose_no_limit() {
    let limits = DecodeLimits::default();
    assert_eq!(limits.max_original_bytes(), usize::MAX);
    assert_eq!(limits.max_decoded_utf8_bytes(), usize::MAX);
    let limits = DecodeLimits::new(7, 9);
    assert_eq!(
        (limits.max_original_bytes(), limits.max_decoded_utf8_bytes()),
        (7, 9)
    );
}

#[test]
fn a_decode_error_describes_its_cause_and_is_an_error() {
    let error = decode(b"abc", no_hints(), DecodeLimits::new(1, 1)).unwrap_err();
    assert_eq!(
        error.to_string(),
        "style sheet original-byte limit exceeded"
    );
    let error = decode(b"abc", no_hints(), DecodeLimits::new(3, 1)).unwrap_err();
    assert_eq!(
        error.to_string(),
        "style sheet decoded UTF-8 byte limit exceeded"
    );
    fn is_error<E: std::error::Error + Send + Sync + 'static>(_: &E) {}
    is_error(&error);
    let copy: DecodeError = error;
    assert_eq!(copy, error);
}

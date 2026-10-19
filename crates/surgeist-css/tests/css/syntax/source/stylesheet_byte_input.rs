use surgeist_css::{
    CssEncoding, CssEncodingSelection, CssInputDecodeError, CssRule, CssStylesheetDecodeHints,
    CssStylesheetDecodeLimits, CssStylesheetLocation, CssStylesheetParseOptions,
    CssSupportsConditionKind, CssValueOrigin, decode_stylesheet_bytes,
    decode_stylesheet_bytes_with_limits, parse_decoded_stylesheet, parse_sheet,
    parse_sheet_with_options,
};

fn hints(label: &str) -> CssStylesheetDecodeHints<'_> {
    CssStylesheetDecodeHints {
        protocol_label: Some(label),
        environment_encoding: None,
    }
}

#[test]
fn standard_labels_are_validated_and_replacement_labels_remain_recognized() {
    for (label, canonical) in [
        (" \tUtF-8\r\n", "UTF-8"),
        ("latin1", "windows-1252"),
        ("ascii", "windows-1252"),
        ("utf-16", "UTF-16LE"),
        ("unicodefffe", "UTF-16BE"),
        ("csiso2022kr", "replacement"),
        ("hz-gb-2312", "replacement"),
        ("x-user-defined", "x-user-defined"),
    ] {
        assert_eq!(CssEncoding::for_label(label).unwrap().name(), canonical);
    }
    for label in ["", "not-an-encoding", "utf-8\0", "\u{a0}utf-8"] {
        assert_eq!(CssEncoding::for_label(label), None);
    }
    let decoded = decode_stylesheet_bytes(b"anything", hints("iso-2022-kr")).unwrap();
    assert_eq!(decoded.source().as_str(), "�");
    assert_eq!(decoded.encoding().name(), "replacement");
    assert_eq!(decoded.selection(), CssEncodingSelection::Protocol);
    assert!(decoded.had_replacements());
}

#[test]
fn unicode_legacy_east_asian_and_stateful_encodings_have_literal_outputs() {
    // Literal standard mapping pairs, independent of the decoder under test.
    for (label, bytes, expected) in [
        ("utf-8", &b"\xF0\x9F\x98\x80"[..], "😀"),
        ("utf-16le", &b"\x3D\xD8\x00\xDE"[..], "😀"),
        ("utf-16be", &b"\xD8\x3D\xDE\x00"[..], "😀"),
        ("windows-1252", &b"\x80\xE9"[..], "€é"),
        ("windows-1251", &b"\xC0"[..], "А"),
        ("koi8-r", &b"\xE1"[..], "А"),
        ("iso-8859-2", &b"\xA1"[..], "Ą"),
        ("ibm866", &b"\x80"[..], "А"),
        ("shift_jis", &b"\x82\xA0"[..], "あ"),
        ("euc-jp", &b"\xA4\xA2"[..], "あ"),
        ("euc-kr", &b"\xB0\xA1"[..], "가"),
        ("gbk", &b"\xD6\xD0"[..], "中"),
        ("gb18030", &b"\xD6\xD0"[..], "中"),
        ("big5", &b"\xA4\x40"[..], "一"),
        ("iso-2022-jp", &b"AAA\x1B(J\\\x1B(BZ"[..], "AAA¥Z"),
        ("x-user-defined", &b"\x80"[..], "\u{f780}"),
    ] {
        let decoded = decode_stylesheet_bytes(bytes, hints(label)).unwrap();
        assert_eq!(decoded.source().as_str(), expected, "{label}");
        assert!(!decoded.had_replacements(), "{label}");
        assert_eq!(decoded.original_bytes(), bytes);
        assert_eq!(decoded.selection(), CssEncodingSelection::Protocol);
    }
}

#[test]
fn every_encoded_bom_overrides_even_recognized_replacement_fallback() {
    for (bytes, text, encoding) in [
        (&b"\xEF\xBB\xBFa{}"[..], "a{}", "UTF-8"),
        (&b"\xFF\xFEa\0{\0}\0"[..], "a{}", "UTF-16LE"),
        (&b"\xFE\xFF\0a\0{\0}"[..], "a{}", "UTF-16BE"),
    ] {
        for label in ["windows-1252", "replacement", "utf-16be"] {
            let decoded = decode_stylesheet_bytes(bytes, hints(label)).unwrap();
            assert_eq!(decoded.source().as_str(), text);
            assert_eq!(decoded.encoding().name(), encoding);
            assert_eq!(decoded.selection(), CssEncodingSelection::ByteOrderMark);
            assert!(!decoded.had_replacements());
            assert_eq!(decoded.original_bytes(), bytes);
        }
    }
    let empty_bom =
        decode_stylesheet_bytes(b"\xEF\xBB\xBF", CssStylesheetDecodeHints::default()).unwrap();
    assert_eq!(empty_bom.source().as_str(), "");
    assert_eq!(empty_bom.selection(), CssEncodingSelection::ByteOrderMark);
}

#[test]
fn fallback_order_is_protocol_then_exact_prefix_then_environment_then_utf8() {
    let bytes = b"@charset \"windows-1252\";\x80";
    let environment = CssEncoding::for_label("windows-1251").unwrap();
    let protocol = decode_stylesheet_bytes(
        bytes,
        CssStylesheetDecodeHints {
            protocol_label: Some("utf-8"),
            environment_encoding: Some(environment),
        },
    )
    .unwrap();
    assert_eq!(protocol.selection(), CssEncodingSelection::Protocol);
    assert_eq!(protocol.source().as_str(), "@charset \"windows-1252\";�");
    let prefix = decode_stylesheet_bytes(
        bytes,
        CssStylesheetDecodeHints {
            protocol_label: Some("invalid"),
            environment_encoding: Some(environment),
        },
    )
    .unwrap();
    assert_eq!(prefix.selection(), CssEncodingSelection::CharsetPrefix);
    assert_eq!(prefix.source().as_str(), "@charset \"windows-1252\";€");
    let environment_input = decode_stylesheet_bytes(
        b"\xC0",
        CssStylesheetDecodeHints {
            protocol_label: Some("invalid"),
            environment_encoding: Some(environment),
        },
    )
    .unwrap();
    assert_eq!(
        environment_input.selection(),
        CssEncodingSelection::Environment
    );
    assert_eq!(environment_input.source().as_str(), "А");
    let default = decode_stylesheet_bytes(b"\xC0", hints("invalid")).unwrap();
    assert_eq!(default.selection(), CssEncodingSelection::DefaultUtf8);
    assert_eq!(default.source().as_str(), "�");
}

#[test]
fn malformed_prefixes_and_unknown_labels_fall_through_to_environment() {
    let environment = CssEncoding::for_label("windows-1252").unwrap();
    for bytes in [
        &b" @charset \"utf-8\";\x80"[..],
        &b"@Charset \"utf-8\";\x80"[..],
        &b"@charset  \"utf-8\";\x80"[..],
        &b"@charset 'utf-8';\x80"[..],
        &b"@charset/*x*/ \"utf-8\";\x80"[..],
        &b"@charset \"utf-8\" ;\x80"[..],
        &b"@charset \"utf-8\"\x80"[..],
        &b"@charset \"utf-\x808\";\x80"[..],
        &b"@charset \"unknown\";\x80"[..],
        &b"@charset \"utf-8\0\";\x80"[..],
        &b"@charset \"\";\x80"[..],
    ] {
        let decoded = decode_stylesheet_bytes(
            bytes,
            CssStylesheetDecodeHints {
                protocol_label: None,
                environment_encoding: Some(environment),
            },
        )
        .unwrap();
        assert_eq!(
            decoded.selection(),
            CssEncodingSelection::Environment,
            "{bytes:?}"
        );
        assert!(decoded.source().as_str().ends_with('€'));
    }
}

#[test]
fn prefix_terminator_at_byte_1024_is_admitted_but_1025_is_not() {
    // 10 prefix bytes + 12 label bytes + 1000 spaces + 2 terminator bytes = 1024.
    let mut bytes = b"@charset \"windows-1252".to_vec();
    bytes.extend(std::iter::repeat_n(b' ', 1000));
    bytes.extend_from_slice(b"\";\x80");
    assert_eq!(bytes.len(), 1025);
    assert_eq!(&bytes[1022..1024], b"\";");
    let boundary = decode_stylesheet_bytes(&bytes, CssStylesheetDecodeHints::default()).unwrap();
    assert_eq!(boundary.selection(), CssEncodingSelection::CharsetPrefix);
    assert!(boundary.source().as_str().ends_with("\";€"));
    bytes.insert(22, b' ');
    assert_eq!(bytes.len(), 1026);
    assert_eq!(&bytes[1023..1025], b"\";");
    let past_boundary =
        decode_stylesheet_bytes(&bytes, CssStylesheetDecodeHints::default()).unwrap();
    assert_eq!(past_boundary.selection(), CssEncodingSelection::DefaultUtf8);
    assert!(past_boundary.source().as_str().ends_with("\";�"));
}

#[test]
fn utf16_prefix_remapping_does_not_change_protocol_or_environment() {
    for label in ["utf-16le", "utf-16be", "utf-16"] {
        let bytes = format!("@charset \"{label}\";a{{}}");
        let decoded =
            decode_stylesheet_bytes(bytes.as_bytes(), CssStylesheetDecodeHints::default()).unwrap();
        assert_eq!(decoded.encoding().name(), "UTF-8");
        assert_eq!(decoded.selection(), CssEncodingSelection::CharsetPrefix);
        assert_eq!(decoded.source().as_str(), bytes);
    }
    let bytes = b"a\0{\0}\0";
    let encoding = CssEncoding::for_label("utf-16le").unwrap();
    let decoded = decode_stylesheet_bytes(
        bytes,
        CssStylesheetDecodeHints {
            protocol_label: None,
            environment_encoding: Some(encoding),
        },
    )
    .unwrap();
    assert_eq!(decoded.encoding().name(), "UTF-16LE");
    assert_eq!(decoded.source().as_str(), "a{}");
    assert_eq!(decoded.selection(), CssEncodingSelection::Environment);
}

#[test]
fn final_incomplete_sequences_replace_once_and_valid_replacement_scalar_is_not_an_error() {
    for (bytes, label, text) in [
        (&b"A\xE2\x82"[..], "utf-8", "A�"),
        (&b"\xED\xA0\x80"[..], "utf-8", "���"),
        (&b"a\0\x00\xD8"[..], "utf-16le", "a�"),
        (&b"\0a\xD8\0"[..], "utf-16be", "a�"),
        (&b"a\0\x00"[..], "utf-16le", "a�"),
        (&b"A\x82"[..], "shift_jis", "A�"),
        (&b"AAA\x1B(J\\\x1B(BZ\x1B"[..], "iso-2022-jp", "AAA¥Z�"),
    ] {
        let decoded = decode_stylesheet_bytes(bytes, hints(label)).unwrap();
        assert_eq!(decoded.source().as_str(), text, "{label}");
        assert!(decoded.had_replacements());
    }
    let decoded = decode_stylesheet_bytes("�".as_bytes(), hints("utf-8")).unwrap();
    assert_eq!(decoded.source().as_str(), "�");
    assert!(!decoded.had_replacements());
}

#[test]
fn limits_are_atomic_exact_in_original_and_decoded_byte_units() {
    let bytes = b"\x80\xE9";
    let exact = CssStylesheetDecodeLimits::new(2, 5);
    let decoded = decode_stylesheet_bytes_with_limits(bytes, hints("windows-1252"), exact).unwrap();
    assert_eq!(decoded.source().as_str(), "€é");
    assert_eq!(
        decode_stylesheet_bytes_with_limits(
            bytes,
            hints("windows-1252"),
            CssStylesheetDecodeLimits::new(1, 5)
        ),
        Err(CssInputDecodeError::OriginalByteLimit)
    );
    assert_eq!(
        decode_stylesheet_bytes_with_limits(
            bytes,
            hints("windows-1252"),
            CssStylesheetDecodeLimits::new(2, 4)
        ),
        Err(CssInputDecodeError::DecodedUtf8ByteLimit)
    );
    assert_eq!(
        decode_stylesheet_bytes_with_limits(bytes, hints("windows-1252"), exact)
            .unwrap()
            .source()
            .as_str(),
        "€é"
    );
    assert_eq!(
        decode_stylesheet_bytes_with_limits(
            "😀".as_bytes(),
            hints("utf-8"),
            CssStylesheetDecodeLimits::new(4, 3)
        ),
        Err(CssInputDecodeError::DecodedUtf8ByteLimit)
    );
    assert_eq!(
        decode_stylesheet_bytes_with_limits(
            b"\xEF\xBB\xBF",
            hints("utf-8"),
            CssStylesheetDecodeLimits::new(3, 0)
        )
        .unwrap()
        .source()
        .as_str(),
        ""
    );
    assert_eq!(
        decode_stylesheet_bytes_with_limits(
            b"",
            hints("utf-8"),
            CssStylesheetDecodeLimits::new(0, 0)
        )
        .unwrap()
        .source()
        .as_str(),
        ""
    );
}

#[test]
fn content_equality_does_not_merge_separate_input_or_snapshot_identity() {
    let first = decode_stylesheet_bytes(b"a{}", CssStylesheetDecodeHints::default()).unwrap();
    let second = decode_stylesheet_bytes(b"a{}", CssStylesheetDecodeHints::default()).unwrap();
    let cloned = first.clone();
    assert_eq!(first, second);
    assert!(!first.same_input(&second));
    assert!(!first.source().same_snapshot(second.source()));
    assert!(first.same_input(&cloned));
    assert!(first.source().same_snapshot(cloned.source()));
}

#[test]
fn utf16_sheet_components_share_decoded_snapshot_with_utf8_and_utf16_coordinates() {
    let text = "/*😀*/@supports (width: 2px){}";
    let mut bytes = vec![0xFF, 0xFE];
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    let input = decode_stylesheet_bytes(&bytes, hints("windows-1252")).unwrap();
    let report = parse_decoded_stylesheet(&input, None);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("supports rule")
    };
    let CssSupportsConditionKind::Declaration(declaration) = rule.condition().kind() else {
        panic!("declaration condition")
    };
    let CssValueOrigin::Parsed(origin) = declaration.property_component().origin() else {
        panic!("parsed property")
    };
    assert!(origin.source().same_snapshot(input.source()));
    assert_eq!(origin.source().as_str(), text);
    assert_eq!(origin.span().start().byte_offset().value(), 19);
    assert_eq!(origin.span().start().line().value(), 0);
    assert_eq!(origin.span().start().column().value(), 17);
    for component in declaration.components() {
        if let CssValueOrigin::Parsed(origin) = component.origin() {
            assert!(origin.source().same_snapshot(input.source()));
        }
    }
}

#[test]
fn byte_composition_preserves_prefilter_source_and_existing_string_recovery() {
    let text = "/*😀*/\r\n\x0c\r@supports (width: 2px){a{--x:a\0b; mystery:1; color:red}}";
    let input =
        decode_stylesheet_bytes(text.as_bytes(), CssStylesheetDecodeHints::default()).unwrap();
    assert_eq!(input.source().as_str(), text);
    let report = parse_decoded_stylesheet(&input, None);
    let control = parse_sheet(text);
    assert_eq!(report, control);
    assert_eq!(report.diagnostics().len(), 1);
    let position = report.diagnostics()[0].error().position();
    // Prefix has one CRLF, one FF and one CR: three source lines.
    assert_eq!(position.line().value(), 3);
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("supports rule")
    };
    let CssSupportsConditionKind::Declaration(declaration) = rule.condition().kind() else {
        panic!("supports declaration")
    };
    let CssValueOrigin::Parsed(origin) = declaration.property_component().origin() else {
        panic!("parsed property")
    };
    assert!(origin.source().same_snapshot(input.source()));
    assert_eq!(origin.span().start().byte_offset().value(), 23);
    assert_eq!(origin.span().start().line().value(), 3);
    assert_eq!(origin.span().start().column().value(), 11);
}

#[test]
fn byte_prefix_text_is_preserved_and_uses_the_current_string_grammar() {
    let input = decode_stylesheet_bytes(
        b"@charset \"windows-1252\";a{--x:\x80}",
        CssStylesheetDecodeHints::default(),
    )
    .unwrap();
    assert_eq!(
        input.source().as_str(),
        "@charset \"windows-1252\";a{--x:€}"
    );
    assert_eq!(
        parse_decoded_stylesheet(&input, None),
        parse_sheet("@charset \"windows-1252\";a{--x:€}")
    );
}

#[test]
fn symbolic_location_is_optional_and_shared_between_byte_and_unicode_fronts() {
    let location = CssStylesheetLocation::new("../sheet.css?x=%2f#原".to_owned());
    let input = decode_stylesheet_bytes(b"a{}", CssStylesheetDecodeHints::default()).unwrap();
    assert_eq!(parse_sheet("a{}").syntax().location(), None);
    assert_eq!(
        parse_decoded_stylesheet(&input, None).syntax().location(),
        None
    );
    let bytes_report = parse_decoded_stylesheet(&input, Some(location.clone()));
    let unicode_report = parse_sheet_with_options(
        "a{}",
        CssStylesheetParseOptions {
            location: Some(location.clone()),
        },
    );
    assert_eq!(bytes_report, unicode_report);
    assert_eq!(bytes_report.syntax().location(), Some(&location));
    assert_eq!(
        bytes_report.syntax().location().unwrap().as_str(),
        "../sheet.css?x=%2f#原"
    );
    let empty = CssStylesheetLocation::new(String::new());
    assert_eq!(
        parse_decoded_stylesheet(&input, Some(empty))
            .syntax()
            .location()
            .unwrap()
            .as_str(),
        ""
    );
    // The Unicode front treats U+FEFF as authored text, whereas encoded BOMs
    // belong to decoding. Reuse the real existing Unicode pipeline as control.
    assert_eq!(
        parse_sheet_with_options("\u{feff}a{}", CssStylesheetParseOptions::default()),
        parse_sheet("\u{feff}a{}")
    );
}

#[test]
fn malformed_utf16_sheet_diagnostics_address_decoded_source_not_network_bytes() {
    let text = "/*😀*/a{mystery:1;color:red}";
    let mut bytes = vec![0xFF, 0xFE];
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    let input = decode_stylesheet_bytes(&bytes, CssStylesheetDecodeHints::default()).unwrap();
    let report = parse_decoded_stylesheet(&input, None);
    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(
        diagnostic.error().code(),
        surgeist_css::CssErrorCode::UnknownProperty
    );
    assert_eq!(diagnostic.error().position().byte_offset().value(), 10);
    assert_eq!(diagnostic.error().position().column().value(), 8);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 10);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 20);
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("style rule")
    };
    assert_eq!(rule.declarations().len(), 1);
    assert_eq!(input.source().as_str(), text);
}

#[test]
fn null_filtering_changes_token_semantics_while_preserving_unfiltered_snapshot() {
    let text = "@supports (wid\0th: 2px){}";
    let input =
        decode_stylesheet_bytes(text.as_bytes(), CssStylesheetDecodeHints::default()).unwrap();
    let report = parse_decoded_stylesheet(&input, None);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("supports rule")
    };
    let CssSupportsConditionKind::Declaration(declaration) = rule.condition().kind() else {
        panic!("supports declaration")
    };
    let surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Ident(name)) =
        declaration.property_component().view()
    else {
        panic!("identifier token")
    };
    assert_eq!(name, "wid�th");
    let CssValueOrigin::Parsed(origin) = declaration.property_component().origin() else {
        panic!("parsed property")
    };
    assert!(origin.source().same_snapshot(input.source()));
    assert_eq!(origin.source().as_str(), text);
    assert_eq!(
        &text[origin.span().start().byte_offset().value()
            ..origin.span().end().byte_offset().value()],
        "wid\0th"
    );
}

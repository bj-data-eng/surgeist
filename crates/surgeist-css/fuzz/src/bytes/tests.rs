use super::*;
use crate::testing::{assert_detected_replayed_and_reduced, key};
use surgeist_fuzz::{OutcomeKind, engine};

fn decode(case: &ByteCase) -> CssDecodedStylesheetInput {
    CssDecodedStylesheetInput::decode(&case.bytes, case.hints(), DecodeLimits::default()).unwrap()
}

/// Handwritten facts about each literal, from the Encoding Standard and the
/// crate's stylesheet-encoding reference; none is read back from a replay.
#[test]
fn literals_decode_to_their_handwritten_selection_and_text() {
    // (id, encoding, selection, text, had replacements)
    let facts: [(u8, &str, EncodingSelection, &str, bool); 12] = [
        (
            0,
            "UTF-8",
            EncodingSelection::DefaultUtf8,
            ".a{color:red}",
            false,
        ),
        (
            1,
            "UTF-8",
            EncodingSelection::ByteOrderMark,
            ".a{color:red}",
            false,
        ),
        (
            2,
            "UTF-16LE",
            EncodingSelection::ByteOrderMark,
            ".a{b:c}",
            false,
        ),
        (
            3,
            "UTF-16BE",
            EncodingSelection::ByteOrderMark,
            ".a{b:c}",
            false,
        ),
        (
            4,
            "windows-1252",
            EncodingSelection::CharsetPrefix,
            "@charset \"windows-1252\"; a{content:\"\u{e9}\u{a0}\u{ff}\"}",
            false,
        ),
        (
            5,
            "UTF-8",
            EncodingSelection::Protocol,
            "@charset \"windows-1252\"; a{content:\"\u{fffd}\"}",
            true,
        ),
        (
            6,
            "windows-1252",
            EncodingSelection::Environment,
            "a{content:\"\u{e9}\"}",
            false,
        ),
        (
            7,
            "UTF-8",
            EncodingSelection::CharsetPrefix,
            "@charset \"utf-16le\"; a{b:c}",
            false,
        ),
        (
            9,
            "windows-1252",
            EncodingSelection::Environment,
            "@charset  \"windows-1252\"; a{b:\u{e9}}",
            false,
        ),
        (
            15,
            "replacement",
            EncodingSelection::Protocol,
            "\u{fffd}",
            true,
        ),
        // A lone high surrogate and the odd byte after it are one error at the
        // end of the queue (Encoding Standard, UTF-16 decoder).
        (
            16,
            "UTF-16LE",
            EncodingSelection::ByteOrderMark,
            "a\u{fffd}",
            true,
        ),
        (
            17,
            "UTF-8",
            EncodingSelection::DefaultUtf8,
            "a{}\r\n.b{\u{c}}\r.c{x:\0}\n\u{20ac}{}",
            false,
        ),
    ];
    for (id, encoding, selection, text, replaced) in facts {
        let decoded = decode(&literal(id).unwrap());
        assert_eq!(decoded.encoding().name(), encoding, "literal {id}");
        assert_eq!(decoded.selection(), selection, "literal {id}");
        assert_eq!(decoded.source().as_str(), text, "literal {id}");
        assert_eq!(decoded.had_replacements(), replaced, "literal {id}");
    }
}

/// The handwritten label table covers every label the vocabularies use, and
/// agrees with the registry the decoder consults. The table, not the registry,
/// supplies the expectation.
#[test]
fn the_handwritten_label_table_covers_every_vocabulary_label() {
    let vocabulary = ENVIRONMENT
        .iter()
        .chain(&PROTOCOL)
        .chain(&DECLARED)
        .chain(&[" UTF-16BE\n", "hz-gb-2312", "utf-16le", "windows-1252"]);
    for label in vocabulary {
        assert_eq!(
            standard_label(label),
            Encoding::for_label(label).map(Encoding::name),
            "{label:?}"
        );
    }
    for environment in ENVIRONMENT {
        assert!(standard_label(environment).is_some(), "{environment}");
    }
    // Case and surrounding ASCII whitespace do not matter; names that are not
    // labels resolve to nothing.
    assert_eq!(standard_label(" UTF-8\n"), Some("UTF-8"));
    assert_eq!(standard_label("\tLatin1\r"), Some("windows-1252"));
    for none in ["", "utf-7", "no-such-encoding", "unknown-label", "utf 8"] {
        assert_eq!(standard_label(none), None, "{none:?}");
    }
}

#[test]
fn the_charset_window_is_exactly_the_first_1024_bytes() {
    let recognized = decode(&literal(10).unwrap());
    let beyond = decode(&literal(11).unwrap());
    assert_eq!(recognized.selection(), EncodingSelection::CharsetPrefix);
    assert_eq!(beyond.selection(), EncodingSelection::Environment);
    assert_eq!(
        literal(10).unwrap().bytes().iter().position(|b| *b == b';'),
        Some(1023)
    );
    assert_eq!(
        literal(11).unwrap().bytes().iter().position(|b| *b == b';'),
        Some(1024)
    );
}

#[test]
fn limits_refuse_the_whole_decode_at_their_exact_edges() {
    let at = literal(12).unwrap();
    assert!(CssDecodedStylesheetInput::decode(&at.bytes, at.hints(), at.limits()).is_ok());
    let below = literal(13).unwrap();
    let refused = CssDecodedStylesheetInput::decode(&below.bytes, below.hints(), below.limits());
    assert!(refused.unwrap_err().is_original_byte_limit());
    let wide = literal(14).unwrap();
    let refused = CssDecodedStylesheetInput::decode(&wide.bytes, wide.hints(), wide.limits());
    assert!(refused.unwrap_err().is_decoded_utf8_byte_limit());
}

#[test]
fn exceeding_both_limits_is_one_refusal_whichever_limit_the_crate_reports() {
    // Neither limit order is documented; the whole decode is refused either way.
    let bytes = b".a{b:c}".to_vec();
    let case = ByteCase::new(bytes.clone(), None, None, 3, 2).unwrap();
    let refused = CssDecodedStylesheetInput::decode(&bytes, case.hints(), case.limits());
    let error = refused.unwrap_err();
    assert!(error.is_original_byte_limit() || error.is_decoded_utf8_byte_limit());
    let outcome = ByteTarget::new().evaluate(&case).unwrap();
    assert_eq!(outcome.kind(), OutcomeKind::ExpectedRejection);
    // Only the original limit exceeded, only the decoded limit exceeded.
    let original = ByteCase::new(bytes.clone(), None, None, 3, u32::MAX).unwrap();
    assert_eq!(
        ByteTarget::new().evaluate(&original).unwrap().kind(),
        OutcomeKind::ExpectedRejection
    );
    let decoded = ByteCase::new(bytes, None, None, u32::MAX, 2).unwrap();
    assert_eq!(
        ByteTarget::new().evaluate(&decoded).unwrap().kind(),
        OutcomeKind::ExpectedRejection
    );
}

#[test]
fn every_literal_captures_replays_and_checks_properties() {
    let target = ByteTarget::new();
    for id in 0..ByteTarget::LITERALS {
        let case = target.literal(id).unwrap();
        let mut captured = Vec::new();
        let observed = engine::evaluate_case(&target, &case, &mut |bytes| {
            captured = bytes.to_vec();
            Ok(())
        })
        .unwrap();
        let expected_kind = if matches!(id, 13 | 14) {
            OutcomeKind::ExpectedRejection
        } else {
            OutcomeKind::Evaluated
        };
        assert_eq!(observed.outcome.kind(), expected_kind, "literal {id}");
        assert!(!observed.outcome.checked_properties().is_empty());
        assert_eq!(target.decode(&captured).unwrap(), case);
        assert_eq!(target.encode(&case).unwrap(), captured);
        assert_eq!(
            engine::replay(&target, &captured, &mut |_| Ok(())).unwrap(),
            observed
        );
    }
    assert!(target.literal(ByteTarget::LITERALS).is_err());
}

#[test]
fn text_is_checked_against_the_standard_library_decoders_where_they_apply() {
    let target = ByteTarget::new();
    // UTF-8 and UTF-16 literals have an independent decoder; a Shift_JIS hint does not.
    let checked = |case: &ByteCase| {
        target
            .evaluate(case)
            .unwrap()
            .checked_properties()
            .iter()
            .any(|property| property.as_str() == TEXT)
    };
    assert!(checked(&literal(1).unwrap()));
    assert!(checked(&literal(2).unwrap()));
    assert!(checked(&literal(6).unwrap()));
    assert!(!checked(
        &ByteCase::unlimited(b"a{}".to_vec(), None, Some(5)).unwrap()
    ));
}

/// Syntax 3 section 3.2: `@charset "`, ASCII characters but `"`, `";`, at offset
/// 0 and wholly within the first 1024 bytes. The lengths are counted by hand.
#[test]
fn the_handwritten_declaration_pattern_has_the_clause_lengths() {
    for (text, length) in [
        ("@charset \"utf-8\";", Some(17)),
        ("@charset \"\";", Some(12)),
        ("@charset \"a;b\\\";x", Some(16)),
        ("@charset \"utf-8\";a{}", Some(17)),
        ("@charset  \"utf-8\";", None),
        ("@charset 'utf-8';", None),
        ("@charset \"utf-8\" ;", None),
        ("@charset \"utf-8\"", None),
        ("@charset \"\u{e9}\";", None),
        (" @charset \"utf-8\";", None),
        ("@Charset \"utf-8\";", None),
    ] {
        assert_eq!(leading_declaration_len(text), length, "{text:?}");
    }
    let within = format!("@charset \"{}\";", "a".repeat(1012));
    let beyond = format!("@charset \"{}\";", "a".repeat(1013));
    assert_eq!(leading_declaration_len(&within), Some(1024));
    assert_eq!(leading_declaration_len(&beyond), None);
}

/// The exact declaration gives no diagnostic and every other leading `@charset`
/// gives the unknown at-rule drop; the property fails on a report that differs.
#[test]
fn the_declaration_is_consumed_and_other_charset_rules_are_reported() {
    let parser = CssParser::default();
    let dropped = parser.parse_sheet("@charset 'x'; a{}");
    assert!(!dropped.diagnostics().is_empty());
    let clean = parser.parse_sheet("a{}");
    assert!(charset_reporting("@charset \"x\"; a{}", clean.diagnostics()).is_ok());
    assert!(charset_reporting("@charset 'x'; a{}", dropped.diagnostics()).is_ok());
    let consumed_reported = charset_reporting("@charset \"x\"; a{}", dropped.diagnostics());
    assert_eq!(
        consumed_reported.unwrap_err().discriminator(),
        "charset-declaration-reported"
    );
    let other_unreported = charset_reporting("@charset 'x'; a{}", clean.diagnostics());
    assert_eq!(
        other_unreported.unwrap_err().discriminator(),
        "charset-at-rule-not-dropped"
    );
    // A longer at-keyword is another unknown at-rule, outside this property.
    assert!(charset_reporting("@charsets x; a{}", clean.diagnostics()).is_ok());
    for bytes in [
        &b"@charset \"windows-1252\"; a{content:\"\xe9\"}"[..],
        &b"\xEF\xBB\xBF@charset \"no-such-label\"; a{}"[..],
        &b"@charset 'utf-8'; a{}"[..],
        &b"@charset  \"utf-8\"; a{}"[..],
        &b"@CHARSET \"utf-8\"; a{}"[..],
    ] {
        let case = ByteCase::unlimited(bytes.to_vec(), None, None).unwrap();
        let outcome = ByteTarget::new().evaluate(&case).unwrap();
        assert_eq!(outcome.kind(), OutcomeKind::Evaluated, "{bytes:?}");
    }
}

fn wrong_selection(case: &ByteCase) -> (String, EncodingSelection) {
    // Planted: forgets that a byte order mark overrides every other source.
    let (name, selection) = expected_selection(case);
    if selection == EncodingSelection::ByteOrderMark {
        ("UTF-8".into(), EncodingSelection::DefaultUtf8)
    } else {
        (name, selection)
    }
}

fn wrong_text(case: &ByteCase, encoding: &str) -> Option<(String, bool)> {
    // Planted: expects the replacement flag to be unset for malformed input.
    expected_text(case, encoding).map(|(text, _)| (text, false))
}

#[test]
fn a_broken_selection_expectation_is_detected_replayed_and_reduced() {
    let broken = ByteTarget::with_oracle(Oracle {
        selection: wrong_selection,
        ..INDEPENDENT
    });
    let sound = ByteTarget::new();
    let case = ByteCase::unlimited(
        [
            &[0xef, 0xbb, 0xbf][..],
            b"a{b:c}/* padding that reduction may drop */",
        ]
        .concat(),
        Some("windows-1252"),
        Some(3),
    )
    .unwrap();
    let reduced = assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &case,
        &key(SELECTION, "selection-differs"),
    );
    // Same-failure reduction keeps only what the failure needs: the BOM.
    let reduced = sound.decode(&reduced).unwrap();
    assert!(reduced.bytes.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(reduced.bytes.len() < case.bytes.len());
}

#[test]
fn a_broken_replacement_expectation_is_detected_replayed_and_reduced() {
    let broken = ByteTarget::with_oracle(Oracle {
        text: wrong_text,
        ..INDEPENDENT
    });
    let sound = ByteTarget::new();
    let case = ByteCase::unlimited(b"a{b:c}\xc0\x80 tail".to_vec(), None, None).unwrap();
    let reduced = assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &case,
        &key(TEXT, "replacement-flag-differs"),
    );
    assert!(sound.decode(&reduced).unwrap().bytes.len() <= case.bytes.len());
}

#[test]
fn generation_is_deterministic_bounded_and_reaches_every_selection_source() {
    let target = ByteTarget::new();
    let mut sources = std::collections::BTreeSet::new();
    let mut tight = 0;
    for seed in 0_u32..600 {
        let entropy: Vec<u8> = (0..96_u32)
            .map(|slot| (seed.wrapping_mul(2_654_435_761).wrapping_add(slot * 40_503) >> 7) as u8)
            .collect();
        let first = target.generate(&entropy).unwrap();
        let again = target.generate(&entropy).unwrap();
        assert_eq!(first, again);
        let Generation::Generated(case) = first else {
            panic!("byte generation never discards");
        };
        assert!(case.bytes.len() <= MAX_BYTES);
        sources.insert(format!("{:?}", expected_selection(&case).1));
        tight += usize::from(case.original_limit != u32::MAX || case.decoded_limit != u32::MAX);
        let observed = engine::evaluate_case(&target, &case, &mut |_| Ok(())).unwrap();
        assert_ne!(
            observed.outcome.kind(),
            OutcomeKind::Failure,
            "seed {seed}: {:?}",
            observed.outcome.detail()
        );
    }
    assert_eq!(sources.len(), 5, "{sources:?}");
    assert!(tight > 50);
}

#[test]
fn malformed_canonical_input_is_rejected_by_the_strict_codec() {
    let target = ByteTarget::new();
    let canonical = target.encode(&literal(4).unwrap()).unwrap();
    assert_eq!(target.decode(&canonical).unwrap(), literal(4).unwrap());
    let mut trailing = canonical.clone();
    trailing.push(0);
    assert!(target.decode(&trailing).is_err());
    assert!(target.decode(&canonical[..canonical.len() - 1]).is_err());
    let mut bad_environment = canonical.clone();
    bad_environment[13] = 9;
    assert!(target.decode(&bad_environment).is_err());
    let mut bad_tag = canonical;
    bad_tag[14] = 2;
    assert!(target.decode(&bad_tag).is_err());
}

#[test]
fn reduction_candidates_are_valid_bounded_and_stop_when_refused() {
    let target = ByteTarget::new();
    let case = literal(5).unwrap();
    let mut emitted = Vec::new();
    target
        .reduction_candidates(&case, &mut |candidate| {
            emitted.push(candidate);
            true
        })
        .unwrap();
    assert!(emitted.iter().any(|candidate| candidate.protocol.is_none()));
    assert!(
        emitted
            .iter()
            .all(|candidate| candidate.bytes.len() <= case.bytes.len())
    );
    let mut calls = 0;
    target
        .reduction_candidates(&case, &mut |_| {
            calls += 1;
            false
        })
        .unwrap();
    assert_eq!(calls, 1);
}

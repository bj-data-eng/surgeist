#![forbid(unsafe_code)]
//! Public Unicode-range oracles from CSS Syntax 3 (2021-12-24) §§4.2 and 7.1
//! and the catalog-selected CSS Fonts 3 (2018-09-20) §4.5.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#urange-syntax
//! https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#unicode-range-desc
//! These literal endpoints use consumed representations, not decoded numeric values.
use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue as Authored, CssComponentValue,
    CssComponentValueRef as Component, CssComponentValues, CssErrorCode,
    CssFontFaceDescriptorKind as Kind, CssFontFaceDescriptorValue as Value,
    CssFontFaceValueErrorKind, CssNamespaceContext, CssNumericTokenKind, CssRecoveryAction,
    CssRule, CssSelector, CssSelectorCombinator, CssSerializedOrigin,
    CssSpecifiedValueSerializationErrorKind as OutputError,
    CssSpecifiedValueSerializationLimits as OutputLimits, CssUnicodeRange, CssUnicodeRangeList,
    CssValueOrigin, CssValueTokenRef as Token, ErrorKind, parse_component_values,
    parse_font_face_descriptor_value, parse_selector, parse_sheet, validate_sheet,
};

fn endpoints(value: &Authored) -> Vec<(u32, u32)> {
    let Authored::Ordinary(Value::UnicodeRange(ranges)) = value else {
        panic!("ordinary Unicode ranges: {value:?}");
    };
    ranges
        .ranges()
        .iter()
        .map(|r| (r.start(), r.end()))
        .collect()
}

fn specified(value: &Authored) -> String {
    let Authored::Ordinary(value) = value else {
        panic!("ordinary descriptor: {value:?}");
    };
    value.serialize_specified().unwrap()
}

fn assert_lifecycle(source: &str, expected: &[(u32, u32)], canonical: &str) {
    let raw = parse_font_face_descriptor_value(source, Kind::UnicodeRange);
    assert!(raw.is_clean(), "{source:?}: {raw:?}");
    let value = raw.syntax().as_ref().unwrap();
    assert_eq!(endpoints(value), expected, "{source:?}");
    assert_eq!(specified(value), canonical, "{source:?}");
    assert_eq!(raw.clone().into_validation_result().unwrap(), *raw.syntax());

    let components = parse_component_values(source).unwrap();
    let before = components.clone();
    let checked = Authored::try_from_components(Kind::UnicodeRange, components.clone()).unwrap();
    assert_eq!(endpoints(&checked), expected, "{source:?}");
    assert_eq!(specified(&checked), canonical, "{source:?}");
    assert_eq!(components, before);

    let sheet_source =
        format!("@font-face{{unicode-range:{source};font-display:swap}}.after{{color:red}}");
    let sheet = parse_sheet(&sheet_source);
    assert!(sheet.is_clean(), "{source:?}: {sheet:?}");
    let [CssRule::FontFace(face), CssRule::Style(_)] = sheet.syntax().rules() else {
        panic!("font face and later rule: {sheet:?}");
    };
    let descriptor = face.descriptors().effective(Kind::UnicodeRange).unwrap();
    assert_eq!(endpoints(descriptor.value()), expected, "{source:?}");
    assert_eq!(specified(descriptor.value()), canonical, "{source:?}");
    assert_eq!(
        descriptor.position().unwrap().byte_offset().value(),
        "@font-face{".len()
    );
    assert!(face.descriptors().effective(Kind::FontDisplay).is_some());
    assert_eq!(validate_sheet(&sheet_source).unwrap(), *sheet.syntax());
}

fn assert_invalid_lifecycle(source: &str) {
    let raw = parse_font_face_descriptor_value(source, Kind::UnicodeRange);
    assert!(raw.syntax().is_none(), "{source:?}: {raw:?}");
    assert!(raw.clone().into_validation_result().is_err());
    assert!(
        raw.diagnostics().iter().any(|d| {
            d.error().code() == CssErrorCode::InvalidDescriptorValue
                && d.action() == CssRecoveryAction::RejectInput
        }),
        "{source:?}: {raw:?}"
    );

    let components = parse_component_values(source).unwrap();
    assert!(
        Authored::try_from_components(Kind::UnicodeRange, components).is_err(),
        "{source:?}"
    );
    let sheet_source =
        format!("@font-face{{unicode-range:{source};font-display:swap}}.after{{color:red}}");
    let sheet = parse_sheet(&sheet_source);
    let [CssRule::FontFace(face), CssRule::Style(_)] = sheet.syntax().rules() else {
        panic!("local descriptor recovery: {source:?}: {sheet:?}");
    };
    assert!(
        face.descriptors().effective(Kind::UnicodeRange).is_none(),
        "{source:?}"
    );
    assert!(face.descriptors().effective(Kind::FontDisplay).is_some());
    assert!(
        sheet.diagnostics().iter().any(|d| {
            d.error().code() == CssErrorCode::InvalidDescriptorValue
                && d.action() == CssRecoveryAction::DropDescriptor
        }),
        "{source:?}: {sheet:?}"
    );
    assert_eq!(
        validate_sheet(&sheet_source).unwrap_err().diagnostics(),
        sheet.diagnostics()
    );
}

#[test]
fn all_six_ordinary_token_patterns_share_descriptor_sheet_and_checked_admission() {
    // The six rows are the six alternatives printed in Syntax §7.1, in order.
    let cases = [
        ("u+a", (10, 10), "U+A"),
        ("u+0a", (10, 10), "U+A"),
        ("u+0", (0, 0), "U+0"),
        ("u+0-7F", (0, 127), "U+0-7F"),
        ("u+0-10", (0, 16), "U+0-10"),
        ("u+?", (0, 15), "U+0-F"),
    ];
    for (row, (source, expected, canonical)) in cases.into_iter().enumerate() {
        let values = parse_component_values(source).unwrap();
        let items = values.items();
        assert!(matches!(
            items[0].view(),
            Component::Token(Token::Ident("u"))
        ));
        match row {
            0 => {
                assert_eq!(items.len(), 3);
                assert!(matches!(
                    items[1].view(),
                    Component::Token(Token::Delim('+'))
                ));
                assert!(matches!(
                    items[2].view(),
                    Component::Token(Token::Ident("a"))
                ));
            }
            1 => {
                assert_eq!(items.len(), 2);
                let Component::Token(Token::Dimension { number, unit }) = items[1].view() else {
                    panic!("signed dimension representation");
                };
                assert_eq!(number.representation(), "+0");
                assert_eq!(unit, "a");
            }
            2 => {
                assert_eq!(items.len(), 2);
                assert!(matches!(
                    items[1].view(),
                    Component::Token(Token::Number(_))
                ));
            }
            3 => {
                assert_eq!(items.len(), 3);
                assert!(matches!(
                    items[1].view(),
                    Component::Token(Token::Number(_))
                ));
                assert!(matches!(
                    items[2].view(),
                    Component::Token(Token::Dimension { .. })
                ));
            }
            4 => {
                assert_eq!(items.len(), 3);
                assert!(matches!(
                    items[1].view(),
                    Component::Token(Token::Number(_))
                ));
                assert!(matches!(
                    items[2].view(),
                    Component::Token(Token::Number(_))
                ));
            }
            5 => {
                assert_eq!(items.len(), 3);
                assert!(matches!(
                    items[1].view(),
                    Component::Token(Token::Delim('+'))
                ));
                assert!(matches!(
                    items[2].view(),
                    Component::Token(Token::Delim('?'))
                ));
            }
            _ => unreachable!("six literal alternatives"),
        }
        assert_lifecycle(source, &[expected], canonical);
    }
}

#[test]
fn u_plus_a_remains_two_type_selectors_and_a_sibling_combinator() {
    let namespaces = CssNamespaceContext::default();
    for source in ["u+a", "u + a"] {
        let report = parse_selector(source, &namespaces);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let selector = report.into_validation_result().unwrap().unwrap();
        let CssSelector::Complex(complex) = &selector else {
            panic!("ordinary complex selector: {selector:?}");
        };
        assert_eq!(
            complex.first().type_selector().unwrap().local_name(),
            Some("u")
        );
        let [part] = complex.rest() else {
            panic!("one sibling relation");
        };
        assert_eq!(part.combinator(), CssSelectorCombinator::NextSibling);
        assert_eq!(
            part.selector().type_selector().unwrap().local_name(),
            Some("a")
        );
        assert_eq!(selector.to_specified_css().unwrap(), "u + a");
    }
    let sheet = parse_sheet("u+a{color:green}");
    assert!(sheet.is_clean(), "{sheet:?}");
    let [CssRule::Style(rule)] = sheet.syntax().rules() else {
        panic!("ordinary style rule");
    };
    let [selector] = rule.selectors().selectors() else {
        panic!("one ordinary selector");
    };
    assert_eq!(selector.selector().to_specified_css().unwrap(), "u + a");
}

#[test]
fn comments_can_separate_range_tokens_but_actual_internal_whitespace_cannot() {
    for (source, expected, canonical) in [
        ("U/**/+/**/A", (10, 10), "U+A"),
        ("u+1/**/2", (18, 18), "U+12"),
        ("u+1/**/-2", (1, 2), "U+1-2"),
        ("u/**/+?/**/?", (0, 255), "U+0-FF"),
        (" \t/*head*/u+0 /*tail*/ ", (0, 0), "U+0"),
    ] {
        assert_lifecycle(source, &[expected], canonical);
    }
    for source in ["u +a", "u+\ta", "u+1 /**/2", "u+?\r\n?", "u+1/**/ -2"] {
        assert_invalid_lifecycle(source);
    }
    assert_lifecycle(
        "u+0 /*tail*/, /*head*/ U+10FFFF",
        &[(0, 0), (0x10ffff, 0x10ffff)],
        "U+0, U+10FFFF",
    );
}

#[test]
fn numeric_and_escape_representations_control_ranges_instead_of_decoded_payloads() {
    for (source, expected, canonical) in [
        ("u+0e0", (224, 224), "U+E0"),
        ("u+1e1", (481, 481), "U+1E1"),
        ("u+12e-130", (302, 304), "U+12E-130"),
        (r"\75+0", (0, 0), "U+0"),
        (r"\75 +0", (0, 0), "U+0"),
    ] {
        assert_lifecycle(source, &[expected], canonical);
    }
    let values = parse_component_values("u+0e0").unwrap();
    let Component::Token(Token::Number(number)) = values.items()[1].view() else {
        panic!("one lexical exponent number");
    };
    assert_eq!(number.representation(), "+0e0");
    assert_eq!(number.kind(), CssNumericTokenKind::Number);
    assert!(number.has_sign());

    let values = parse_component_values(r"u+/**/\61").unwrap();
    assert!(matches!(
        values.items().last().unwrap().view(),
        Component::Token(Token::Ident("a"))
    ));
    // Decoded 'a' is a hex digit, but its original representation is an escape.
    for source in [
        r"u+/**/\61",
        r"u+/**/\000061",
        "u+0/**/-ff",
        "u+0.1",
        "u+/**/😀",
    ] {
        assert_invalid_lifecycle(source);
    }
}

#[test]
fn wildcard_length_and_endpoint_domain_boundaries_are_inclusive() {
    for (source, expected, canonical) in [
        ("u+000000", (0, 0), "U+0"),
        ("u+10FFFF", (0x10ffff, 0x10ffff), "U+10FFFF"),
        ("u+000001-000002", (1, 2), "U+1-2"),
        ("u+0-10FFFF", (0, 0x10ffff), "U+0-10FFFF"),
        ("u+D800-DFFF", (0xd800, 0xdfff), "U+D800-DFFF"),
        ("u+?", (0, 15), "U+0-F"),
        ("u+?????", (0, 0xfffff), "U+0-FFFFF"),
        ("u+10????", (0x100000, 0x10ffff), "U+100000-10FFFF"),
        ("u+00000?", (0, 15), "U+0-F"),
    ] {
        assert_lifecycle(source, &[expected], canonical);
    }
    for source in [
        "u+0000000",
        "u+1-0000000",
        "u+??????",
        "u+11????",
        "u+0?0",
        "u+?f",
        "u+1?-2",
        "u+110000",
        "u+0-110000",
        "u+2-1",
        "u+ff-0",
        "u+-1",
    ] {
        assert_invalid_lifecycle(source);
    }
}

#[test]
fn a_raw_value_requires_a_complete_nonempty_comma_list_and_consumes_no_trailing_syntax() {
    for source in [
        ",u+0",
        "u+0,",
        "u+0,,u+1",
        "u+0,u+",
        "u+0 extra",
        "u+0;",
        "u+0{}",
        "u+0,?",
    ] {
        let report = parse_font_face_descriptor_value(source, Kind::UnicodeRange);
        assert!(report.syntax().is_none(), "{source:?}: {report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one complete raw-value rejection: {source:?}: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
        assert!(report.into_validation_result().is_err());
        assert!(
            Authored::try_from_components(
                Kind::UnicodeRange,
                parse_component_values(source).unwrap()
            )
            .is_err(),
            "{source:?}"
        );
    }
    assert_lifecycle(
        "u+10FFFF,u+0,u+0-7F,u+0",
        &[(0x10ffff, 0x10ffff), (0, 0), (0, 127), (0, 0)],
        "U+10FFFF, U+0, U+0-7F, U+0",
    );
}

#[test]
fn invalid_members_drop_only_their_occurrence_and_keep_original_token_coordinates() {
    for (invalid, responsible, spelling) in [
        ("u+1,u+g", "g", "g"),
        (r"u+/**/\61", r"\61", r"\61"),
        ("u+110000", "+110000", "+110000"),
        ("u+1/**/ -2", " -2", " "),
    ] {
        let source = format!(
            "/*😀*/\r\n@font-face{{unicode-range:u+0;unicode-range:{invalid};font-display:swap;unicode-range:u+2}}.after{{color:red}}"
        );
        let report = parse_sheet(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid occurrence: {invalid:?}: {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        let second_name = source.find("unicode-range:u+0;").unwrap() + "unicode-range:u+0;".len();
        let value_start = second_name + "unicode-range:".len();
        let expected = value_start + invalid.find(responsible).unwrap();
        let position = diagnostic.error().position();
        assert_eq!(position.byte_offset().value(), expected);
        assert_eq!(position.line().value(), 1);
        let line_start = source.find('\n').unwrap() + 1;
        assert_eq!(
            position.column().value() as usize,
            source[line_start..expected].encode_utf16().count()
        );
        assert_eq!(diagnostic.span().start().byte_offset().value(), second_name);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            source.find("font-display").unwrap()
        );
        let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
            panic!("typed descriptor failure");
        };
        assert_eq!(detail.at_rule().as_str(), "font-face");
        assert_eq!(detail.descriptor().as_str(), "unicode-range");
        assert_eq!(detail.encountered().unwrap().authored(), spelling);

        let [CssRule::FontFace(face), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("later rule survives");
        };
        let occurrences: Vec<_> = face
            .descriptors()
            .occurrences()
            .filter_map(|descriptor| {
                if descriptor.value().kind() == Kind::UnicodeRange {
                    Some(endpoints(descriptor.value()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(occurrences, vec![vec![(0, 0)], vec![(2, 2)]]);
        assert_eq!(
            endpoints(
                face.descriptors()
                    .effective(Kind::UnicodeRange)
                    .unwrap()
                    .value()
            ),
            vec![(2, 2)]
        );
        assert!(face.descriptors().effective(Kind::FontDisplay).is_some());
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn missing_range_tokens_or_endpoints_use_actual_eof_without_an_encountered_token() {
    for source in ["", "u", "u+", "u+/**/", "u+ab-", "/*😀*/\r\nu+ab-/**/"] {
        let report = parse_font_face_descriptor_value(source, Kind::UnicodeRange);
        assert!(report.syntax().is_none(), "{source:?}: {report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one bounded EOF failure: {source:?}: {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
            panic!("descriptor error");
        };
        assert!(detail.encountered().is_none(), "{source:?}");
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn checked_mixed_origins_preserve_representation_and_map_the_actual_rejected_component() {
    let parsed_number = parse_component_values("+12e-130").unwrap().items()[0].clone();
    let mixed = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("u").unwrap(),
        parsed_number.clone(),
    ])
    .unwrap();
    let checked = Authored::try_from_components(Kind::UnicodeRange, mixed).unwrap();
    assert_eq!(endpoints(&checked), vec![(302, 304)]);
    assert_eq!(specified(&checked), "U+12E-130");
    let CssValueOrigin::Parsed(number_origin) = parsed_number.origin() else {
        panic!("parsed numeric origin");
    };
    assert_eq!(number_origin.source().as_str(), "+12e-130");

    let escaped = parse_component_values(r"\61").unwrap().items()[0].clone();
    let escaped_origin = escaped.origin().clone();
    let mixed = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("u").unwrap(),
        CssComponentValue::try_token("+").unwrap(),
        escaped,
    ])
    .unwrap();
    let error = Authored::try_from_components(Kind::UnicodeRange, mixed).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssFontFaceValueErrorKind::Grammar(ErrorKind::InvalidDescriptorValue(_))
    ));
    assert_eq!(error.origin(), &CssSerializedOrigin::Token(escaped_origin));
    let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
        panic!("actual parsed failing component");
    };
    assert_eq!(origin.source().as_str(), r"\61");
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), 3);

    let programmatic = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("u").unwrap(),
        CssComponentValue::try_token("+").unwrap(),
        CssComponentValue::try_ident("g").unwrap(),
    ])
    .unwrap();
    let error = Authored::try_from_components(Kind::UnicodeRange, programmatic).unwrap_err();
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
}

#[test]
fn checked_ranges_keep_domain_order_overlap_and_atomic_specified_limits() {
    assert_eq!(CssUnicodeRange::try_new(0, 0).unwrap().start(), 0);
    assert_eq!(
        CssUnicodeRange::try_new(0x10ffff, 0x10ffff).unwrap().end(),
        0x10ffff
    );
    assert!(CssUnicodeRange::try_new(2, 1).is_none());
    assert!(CssUnicodeRange::try_new(0, 0x110000).is_none());
    assert!(CssUnicodeRangeList::try_new(Vec::new()).is_none());

    let ranges = [
        CssUnicodeRange::try_new(0x10ffff, 0x10ffff).unwrap(),
        CssUnicodeRange::try_new(0, 127).unwrap(),
        CssUnicodeRange::try_new(0, 127).unwrap(),
    ];
    let list = CssUnicodeRangeList::try_new(ranges.to_vec()).unwrap();
    assert_eq!(list.ranges(), ranges);
    let value = Value::UnicodeRange(list);
    let before = value.clone();
    let expected = "U+10FFFF, U+0-7F, U+0-7F";
    // One list plus three retained range members, even for duplicate members.
    assert_eq!(
        value
            .serialize_specified_with_limits(OutputLimits::new(4, 4, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            OutputLimits::new(3, 4, expected.len()),
            OutputError::InputNodeLimit,
        ),
        (
            OutputLimits::new(4, 3, expected.len()),
            OutputError::ProjectionNodeLimit,
        ),
        (
            OutputLimits::new(4, 4, expected.len() - 1),
            OutputError::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
    }
    assert_eq!(value.serialize_specified().unwrap(), expected);
}

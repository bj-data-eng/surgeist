use surgeist_css::{CssErrorCode, CssRecoveryAction, CssRule, parse_sheet};

// Admission and numeric expectations: pinned CSS Syntax 3 (2021-12-24), §7.1.
#[test]
fn unicode_ranges_preserve_token_representations_across_comments() {
    for (value, expected) in [
        ("u+0", vec![(0, 0)]),
        ("u/**/+0", vec![(0, 0)]),
        ("u+/**/a", vec![(10, 10)]),
        ("u+1/**/2", vec![(0x12, 0x12)]),
        ("u+1/**/-2", vec![(1, 2)]),
        ("u+12e-130", vec![(0x12e, 0x130)]),
        (r"\75+0", vec![(0, 0)]),
        (r"\75 +0", vec![(0, 0)]),
        ("u+1 /*tail*/ , /*head*/ u+2", vec![(1, 1), (2, 2)]),
        ("u+?", vec![(0, 15)]),
        ("u+10????", vec![(0x100000, 0x10ffff)]),
        ("u+D800-DFFF", vec![(0xd800, 0xdfff)]),
        ("u+0, /*😀*/ u+10FFFF", vec![(0, 0), (0x10ffff, 0x10ffff)]),
    ] {
        let source = format!("@font-face{{unicode-range:{value};font-display:swap}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{value:?}: {:?}", report.diagnostics());
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("font-face");
        };
        let actual: Vec<_> = rule
            .descriptors()
            .unicode_range()
            .unwrap()
            .value()
            .ranges()
            .iter()
            .map(|range| (range.start(), range.end()))
            .collect();
        assert_eq!(actual, expected, "{value:?}");
        assert_eq!(
            surgeist_css::validate_sheet(&source).unwrap(),
            *report.syntax()
        );
    }
}

// Diagnostic origins follow Surgeist's responsible-token contract: spelling
// failures identify their original token; genuine missing input identifies EOF.
#[test]
fn invalid_unicode_ranges_point_to_the_responsible_original_token() {
    for (value, responsible, spelling) in [
        ("u+0/**/-ff", "-ff", "-ff"), // Number then Ident is not an admitted pair.
        ("u+1/**/-/**/2", "-/**/2", "-"), // Number then Delim is not an admitted pair.
        ("u+ /**/?", " /**/?", " "),  // Actual whitespace, not a comment.
        (r"u+/**/\61", r"\61", r"\61"), // Preserve original escaped representation.
        ("u+/**/😀", "😀", "😀"),
        ("u+12-", "-", "-"), // A lone hyphen is a Delim after Number(+12).
        ("u+12-/**/", "-", "-"),
        ("u+0g", "+0g", "+0g"), // Bad character within the original Dimension token.
        ("u+110000", "+110000", "+110000"),
        ("u+0/**/-110000", "-110000", "-110000"),
        ("u+ff-0", "ff-0", "ff-0"),
        ("u+1/**/-0", "-0", "-0"),
        ("u+??????", "?", "?"), // Wildcard endpoint domain overflow.
        ("u+1234567", "+1234567", "+1234567"),
        ("u+0, /*😀*/ u+/**/g", "g", "g"),
    ] {
        let source = format!(
            "/*😀*/\r\n@font-face{{unicode-range:{value};font-display:swap}} .after{{color:red}}"
        );
        let report = parse_sheet(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("{value:?}: {:?}", report.diagnostics());
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        let value_start = source.find("unicode-range:").unwrap() + "unicode-range:".len();
        let expected = value_start + value.find(responsible).unwrap();
        let position = diagnostic.error().position();
        assert_eq!(position.byte_offset().value(), expected, "{value:?}");
        let surgeist_css::ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind()
        else {
            panic!("descriptor error");
        };
        assert_eq!(
            detail.encountered().unwrap().authored(),
            spelling,
            "{value:?}"
        );
        assert_eq!(position.line().value(), 1);
        let line_start = source.find('\n').unwrap() + 1;
        assert_eq!(
            position.column().value() as usize,
            source[line_start..expected].encode_utf16().count()
        );
        let [CssRule::FontFace(rule), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("sibling recovery");
        };
        assert!(rule.descriptors().unicode_range().is_none());
        assert!(rule.descriptors().font_display().is_some());
        assert_eq!(
            surgeist_css::validate_sheet(&source)
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn incomplete_unicode_range_lists_drop_the_whole_descriptor() {
    for value in ["u+", "u+0,", "u+0,,u+1", "u+0,u+"] {
        let source = format!("@font-face{{unicode-range:{value};font-display:swap}}");
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{value:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one descriptor failure");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("font-face");
        };
        assert!(rule.descriptors().unicode_range().is_none());
        assert!(rule.descriptors().font_display().is_some());
    }
}

#[test]
fn incomplete_unicode_ranges_keep_their_genuine_end_position() {
    for value in ["u+", "u+ab-", "u+/**/", "u+ab-/**/"] {
        let source = format!("@font-face{{unicode-range:{value};font-display:swap}}");
        let report = parse_sheet(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("{value:?}: {:?}", report.diagnostics());
        };
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find(';').unwrap(),
            "{value:?}"
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.find("unicode-range").unwrap()
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            source.find("font-display").unwrap()
        );
    }
}

#[test]
fn rejected_unicode_range_occurrences_preserve_ordered_valid_occurrences() {
    let source = "@font-face{unicode-range:u+0;unicode-range:u+1,u+g;unicode-range:u/**/+2;font-display:swap}";
    let report = parse_sheet(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("{:?}", report.diagnostics());
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("font-face");
    };
    let ranges = rule.descriptors().unicode_range().unwrap().value().ranges();
    assert_eq!(ranges.len(), 1);
    assert_eq!((ranges[0].start(), ranges[0].end()), (2, 2));
    let occurrences: Vec<_> = rule
        .descriptors()
        .occurrences()
        .filter_map(|descriptor| {
            if let surgeist_css::CssFontFaceDescriptorRef::UnicodeRange(range) = descriptor {
                Some(range.value().ranges()[0].start())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(occurrences, [0, 2]);
}

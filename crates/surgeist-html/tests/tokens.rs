use surgeist_html::{
    DocumentString, HtmlCommaSeparatedTokens, HtmlCommaToken, HtmlOrderedTokenSet,
    HtmlSpaceSeparatedTokens, HtmlSpaceToken, HtmlTokenComparison, HtmlTokenError,
    HtmlUnorderedTokenSet,
};

fn space(value: &str) -> HtmlSpaceToken {
    HtmlSpaceToken::new(value).unwrap()
}

fn comma(value: &str) -> HtmlCommaToken {
    HtmlCommaToken::new(value).unwrap()
}

#[test]
fn native_space_tokens_reject_empty_and_every_ascii_whitespace_without_trimming() {
    assert_eq!(
        HtmlSpaceToken::new(""),
        Err(HtmlTokenError::EmptySpaceToken)
    );
    for whitespace in [0x09, 0x0A, 0x0C, 0x0D, 0x20] {
        let raw = DocumentString::from_utf16(vec![0xD83D, 0xDE42, whitespace, 0x61]);
        assert_eq!(
            HtmlSpaceToken::new(raw.clone()),
            Err(HtmlTokenError::SpaceTokenWhitespace {
                code_unit_offset: 2,
            })
        );
        assert_eq!(raw.code_units(), [0xD83D, 0xDE42, whitespace, 0x61]);
    }
    assert!(HtmlSpaceToken::new(" a ").is_err());
}

#[test]
fn native_tokens_preserve_surrogates_nulls_and_non_ascii_whitespace() {
    let raw = DocumentString::from_utf16(vec![0xA0, 0xD800, 0, 0x0B, 0xDC00, 0xA0]);
    assert_eq!(HtmlSpaceToken::new(raw.clone()).unwrap().value(), &raw);
    assert_eq!(HtmlCommaToken::new(raw.clone()).unwrap().value(), &raw);
    assert_eq!(raw.code_units(), [0xA0, 0xD800, 0, 0x0B, 0xDC00, 0xA0]);
}

#[test]
fn native_comma_tokens_allow_empty_and_internal_whitespace_but_reject_delimiters_and_edges() {
    assert_eq!(comma("").value().code_units(), []);
    assert_eq!(
        comma("a \t\n\u{C}\r b").value().to_utf8().unwrap(),
        "a \t\n\u{C}\r b"
    );
    for whitespace in [0x09, 0x0A, 0x0C, 0x0D, 0x20] {
        assert_eq!(
            HtmlCommaToken::new(DocumentString::from_utf16(vec![whitespace, 0x61])),
            Err(HtmlTokenError::CommaTokenBoundaryWhitespace {
                code_unit_offset: 0
            })
        );
        assert_eq!(
            HtmlCommaToken::new(DocumentString::from_utf16(vec![0x61, whitespace])),
            Err(HtmlTokenError::CommaTokenBoundaryWhitespace {
                code_unit_offset: 1
            })
        );
    }
    assert_eq!(
        HtmlCommaToken::new("\u{1F642},a"),
        Err(HtmlTokenError::CommaTokenComma {
            code_unit_offset: 2
        })
    );
}

#[test]
fn general_space_sequence_preserves_duplicates_order_and_exact_spelling() {
    let raw = DocumentString::from_utf16(vec![0xD800, 0, 0xA0]);
    let value = HtmlSpaceSeparatedTokens::new([
        space("B"),
        HtmlSpaceToken::new(raw.clone()).unwrap(),
        space("B"),
    ]);
    assert_eq!(value.tokens()[0].value(), &DocumentString::from("B"));
    assert_eq!(value.tokens()[1].value(), &raw);
    assert_eq!(value.tokens()[2].value(), &DocumentString::from("B"));
    assert_eq!(
        value.as_document_string().code_units(),
        [0x42, 0x20, 0xD800, 0, 0xA0, 0x20, 0x42]
    );
    assert_eq!(
        HtmlSpaceSeparatedTokens::new([])
            .as_document_string()
            .code_units(),
        []
    );
}

#[test]
fn comma_sequence_preserves_empty_and_repeated_tokens_including_trailing_empty() {
    let value =
        HtmlCommaSeparatedTokens::new([comma("a"), comma(""), comma("d d"), comma("a"), comma("")]);
    assert_eq!(
        value
            .tokens()
            .iter()
            .map(|t| t.value().to_utf8().unwrap())
            .collect::<Vec<_>>(),
        ["a", "", "d d", "a", ""]
    );
    assert_eq!(value.as_document_string().to_utf8().unwrap(), "a,,d d,a,");
    assert_eq!(HtmlCommaSeparatedTokens::new([]).tokens().len(), 0);
    assert_eq!(
        HtmlCommaSeparatedTokens::new([])
            .as_document_string()
            .to_utf8()
            .unwrap(),
        ""
    );
    assert_eq!(HtmlCommaSeparatedTokens::new([comma("")]).tokens().len(), 1);
    assert_eq!(
        HtmlCommaSeparatedTokens::new([comma("")])
            .as_document_string()
            .to_utf8()
            .unwrap(),
        ""
    );
}

#[test]
fn comma_sequence_serialization_preserves_exact_utf16_and_internal_whitespace() {
    let raw = DocumentString::from_utf16(vec![0xDC00, 0, 0x20, 0xD800]);
    let value =
        HtmlCommaSeparatedTokens::new([HtmlCommaToken::new(raw.clone()).unwrap(), comma("z")]);
    assert_eq!(value.tokens()[0].value(), &raw);
    assert_eq!(
        value.as_document_string().code_units(),
        [0xDC00, 0, 0x20, 0xD800, 0x2C, 0x7A]
    );
}

#[test]
fn ordered_unique_sets_reject_duplicates_using_the_selected_comparison() {
    let distinct =
        HtmlOrderedTokenSet::new(HtmlTokenComparison::CaseSensitive, [space("A"), space("a")])
            .unwrap();
    assert_eq!(distinct.tokens().len(), 2);
    assert!(distinct.contains(&DocumentString::from("A")));
    assert!(!distinct.contains(&DocumentString::from(" a ")));
    let error = HtmlOrderedTokenSet::new(
        HtmlTokenComparison::AsciiCaseInsensitive,
        [space("Aa"), space("other"), space("aA")],
    )
    .unwrap_err();
    assert_eq!(
        error.comparison(),
        HtmlTokenComparison::AsciiCaseInsensitive
    );
    assert_eq!(error.token(), &DocumentString::from("aA"));
    assert_eq!(error.first_index(), 0);
    assert_eq!(error.duplicate_index(), 2);
    let error =
        HtmlOrderedTokenSet::new(HtmlTokenComparison::CaseSensitive, [space("x"), space("x")])
            .unwrap_err();
    assert_eq!(error.comparison(), HtmlTokenComparison::CaseSensitive);
    assert_eq!((error.first_index(), error.duplicate_index()), (0, 1));
}

#[test]
fn ordered_unique_set_equality_preserves_meaningful_order_and_uses_ascii_comparison() {
    let comparison = HtmlTokenComparison::AsciiCaseInsensitive;
    let first = HtmlOrderedTokenSet::new(comparison, [space("A"), space("B")]).unwrap();
    let same = HtmlOrderedTokenSet::new(comparison, [space("a"), space("b")]).unwrap();
    let reordered = HtmlOrderedTokenSet::new(comparison, [space("b"), space("a")]).unwrap();
    assert_eq!(first, same);
    assert_ne!(first, reordered);
    assert_eq!(first.as_document_string().to_utf8().unwrap(), "A B");
    assert_eq!(same.as_document_string().to_utf8().unwrap(), "a b");
    assert!(first.contains(&DocumentString::from("a")));
    assert_eq!(first.comparison(), comparison);
}

#[test]
fn unordered_unique_set_equality_ignores_order_without_rewriting_spelling() {
    let comparison = HtmlTokenComparison::AsciiCaseInsensitive;
    let first = HtmlUnorderedTokenSet::new(comparison, [space("A"), space("B")]).unwrap();
    let reordered = HtmlUnorderedTokenSet::new(comparison, [space("b"), space("a")]).unwrap();
    assert_eq!(first, reordered);
    assert_eq!(first.as_document_string().to_utf8().unwrap(), "A B");
    assert_eq!(reordered.as_document_string().to_utf8().unwrap(), "b a");
    assert!(first.contains(&DocumentString::from("b")));
    assert_eq!(first.comparison(), comparison);
    let error = HtmlUnorderedTokenSet::new(comparison, [space("A"), space("a")]).unwrap_err();
    assert_eq!((error.first_index(), error.duplicate_index()), (0, 1));
}

#[test]
fn ascii_insensitive_uniqueness_does_not_fold_unicode_or_surrogates() {
    let comparison = HtmlTokenComparison::AsciiCaseInsensitive;
    let high = HtmlSpaceToken::new(DocumentString::from_utf16(vec![0xD800, 0x41])).unwrap();
    let low = HtmlSpaceToken::new(DocumentString::from_utf16(vec![0xDC00, 0x61])).unwrap();
    let value =
        HtmlUnorderedTokenSet::new(comparison, [space("É"), space("é"), high, low]).unwrap();
    assert_eq!(value.tokens().len(), 4);
    assert!(value.contains(&DocumentString::from_utf16(vec![0xD800, 0x61])));
    assert!(value.contains(&DocumentString::from_utf16(vec![0xDC00, 0x41])));
    let sensitive =
        HtmlUnorderedTokenSet::new(HtmlTokenComparison::CaseSensitive, [space("a")]).unwrap();
    let insensitive = HtmlUnorderedTokenSet::new(comparison, [space("a")]).unwrap();
    assert_ne!(sensitive, insensitive);
}

#[test]
fn unique_empty_sets_require_explicit_comparison_and_serialize_empty() {
    for comparison in [
        HtmlTokenComparison::CaseSensitive,
        HtmlTokenComparison::AsciiCaseInsensitive,
    ] {
        let ordered = HtmlOrderedTokenSet::new(comparison, []).unwrap();
        let unordered = HtmlUnorderedTokenSet::new(comparison, []).unwrap();
        assert!(ordered.tokens().is_empty());
        assert!(unordered.tokens().is_empty());
        assert!(!ordered.contains(&DocumentString::from("x")));
        assert!(!unordered.contains(&DocumentString::from("x")));
        assert_eq!(ordered.as_document_string().code_units(), []);
        assert_eq!(unordered.as_document_string().code_units(), []);
    }
}

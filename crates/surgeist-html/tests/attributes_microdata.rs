use surgeist_html::*;

// HTML 877d §5.2: exact Infra ASCII split, unique authored tokens, but only
// itemprop processing removes later equal occurrences.
#[test]
fn token_ranges_duplicates_and_processing_are_exact() {
    let raw = DocumentString::from(" \ta\nb\u{c}a\rc A\u{a0}z,x.y:q ");
    for kind in [
        HtmlMicrodataTokenKind::ItemType,
        HtmlMicrodataTokenKind::ItemProp,
        HtmlMicrodataTokenKind::ItemRef,
    ] {
        let r = HtmlValueInterpreter::new(raw.len())
            .microdata_tokens(kind, Some(&raw))
            .unwrap();
        let expected = ["a", "b", "a", "c", "A\u{a0}z,x.y:q"];
        for (i, e) in expected.iter().enumerate() {
            assert_eq!(
                r.token_units(i),
                Some(DocumentString::from(*e).code_units())
            );
        }
        assert_eq!(r.tokens()[2].duplicate_of(), Some(0));
        assert!(r.tokens()[4].contains_dot_or_colon());
        assert!(!r.local_author_conforming());
        assert_eq!(
            r.processing_tokens().count(),
            if kind == HtmlMicrodataTokenKind::ItemProp {
                4
            } else {
                5
            }
        );
    }
}
#[test]
fn absence_empty_and_unpaired_units_are_not_collapsed() {
    for kind in [
        HtmlMicrodataTokenKind::ItemType,
        HtmlMicrodataTokenKind::ItemProp,
        HtmlMicrodataTokenKind::ItemRef,
    ] {
        let i = HtmlValueInterpreter::new(3);
        assert!(
            i.microdata_tokens(kind, None)
                .unwrap()
                .local_author_conforming()
        );
        let empty = DocumentString::from("");
        let r = i.microdata_tokens(kind, Some(&empty)).unwrap();
        assert_eq!(r.raw(), Some(&empty));
        assert_eq!(
            r.local_author_conforming(),
            kind == HtmlMicrodataTokenKind::ItemRef
        );
        let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
        let r = i.microdata_tokens(kind, Some(&raw)).unwrap();
        assert_eq!(r.token_units(0), Some(raw.code_units()));
        assert!(
            HtmlValueInterpreter::new(2)
                .microdata_tokens(kind, Some(&raw))
                .is_err()
        );
    }
}
#[test]
fn native_unique_lists_emit_single_spaces_without_url_claims() {
    let a = HtmlMicrodataTokenList::new(
        HtmlMicrodataTokenKind::ItemType,
        vec![
            HtmlSpaceToken::new("not-url").unwrap(),
            HtmlSpaceToken::new("Case").unwrap(),
        ],
    )
    .unwrap()
    .into_attribute();
    assert_eq!(a.name(), &KnownHtmlAttribute::Itemtype.qualified_name());
    assert_eq!(a.value(), &DocumentString::from("not-url Case"));
    assert!(HtmlMicrodataTokenList::new(HtmlMicrodataTokenKind::ItemProp, vec![]).is_err());
    assert!(
        HtmlMicrodataTokenList::new(
            HtmlMicrodataTokenKind::ItemProp,
            vec![
                HtmlSpaceToken::new("x").unwrap(),
                HtmlSpaceToken::new("x").unwrap()
            ]
        )
        .is_err()
    );
    assert!(
        HtmlMicrodataTokenList::new(HtmlMicrodataTokenKind::ItemRef, vec![])
            .unwrap()
            .into_attribute()
            .value()
            .is_empty()
    );
}

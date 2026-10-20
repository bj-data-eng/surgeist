use surgeist_html::*;

#[test]
fn local_author_form_and_processing_guard_are_independent() {
    let interpreter = HtmlValueInterpreter::new(100);
    let missing = interpreter.hash_name_reference(None).unwrap();
    assert_eq!(missing.raw(), None);
    assert_eq!(missing.author_form(), HtmlHashNameAuthorForm::Missing);
    assert_eq!(missing.processing(), HtmlHashNameProcessing::Missing);
    assert_eq!(missing.first_hash_offset(), None);
    assert_eq!(missing.author_name_units(), None);
    assert_eq!(missing.processing_name_units(), None);
    for (text, first, leading, processing, author_tail, processing_tail) in [
        ("", None, false, HtmlHashNameProcessing::NoHash, None, None),
        (
            "abc",
            None,
            false,
            HtmlHashNameProcessing::NoHash,
            None,
            None,
        ),
        (
            "abc#",
            Some(3),
            false,
            HtmlHashNameProcessing::EmptyTail,
            None,
            None,
        ),
        (
            "#",
            Some(0),
            true,
            HtmlHashNameProcessing::EmptyTail,
            Some(""),
            None,
        ),
        (
            "prefix#x",
            Some(6),
            false,
            HtmlHashNameProcessing::Name,
            None,
            Some("x"),
        ),
        (
            "##x",
            Some(0),
            true,
            HtmlHashNameProcessing::Name,
            Some("#x"),
            Some("#x"),
        ),
        (
            "#x#y",
            Some(0),
            true,
            HtmlHashNameProcessing::Name,
            Some("x#y"),
            Some("x#y"),
        ),
    ] {
        let raw = DocumentString::from(text);
        let report = interpreter.hash_name_reference(Some(&raw)).unwrap();
        assert_eq!(report.raw(), Some(&raw));
        assert_eq!(report.first_hash_offset(), first);
        assert_eq!(
            report.author_form(),
            if leading {
                HtmlHashNameAuthorForm::LeadingHashName
            } else {
                HtmlHashNameAuthorForm::NoLeadingHash
            }
        );
        assert_eq!(report.processing(), processing);
        let author = author_tail.map(DocumentString::from);
        let parsed = processing_tail.map(DocumentString::from);
        assert_eq!(
            report.author_name_units(),
            author.as_ref().map(DocumentString::code_units)
        );
        assert_eq!(
            report.processing_name_units(),
            parsed.as_ref().map(DocumentString::code_units)
        );
        assert!(
            report
                .author_source_clause()
                .ends_with("/#valid-hash-name-reference")
        );
        assert!(
            report
                .processing_source_clause()
                .ends_with("/#rules-for-parsing-a-hash-name-reference")
        );
    }
}

#[test]
fn suffixes_borrow_exact_utf16_without_normalization_or_decoding() {
    let raw = DocumentString::from_utf16(vec![0xd83d, 0xde00, 35, 0xd800, 0, 32, 37, 50, 48, 35]);
    let report = HtmlValueInterpreter::new(10)
        .hash_name_reference(Some(&raw))
        .unwrap();
    assert_eq!(report.first_hash_offset(), Some(2));
    assert_eq!(
        report.processing_name_units(),
        Some(&[0xd800, 0, 32, 37, 50, 48, 35][..])
    );
    assert_eq!(report.author_form(), HtmlHashNameAuthorForm::NoLeadingHash);
    assert_eq!(report.author_name_units(), None);
    // The contract exposes a borrowed suffix of the retained shared raw units.
    assert_eq!(
        report.processing_name_units().unwrap().as_ptr(),
        raw.code_units()[3..].as_ptr()
    );
    let spaced = DocumentString::from(" # A%20");
    let result = HtmlValueInterpreter::new(20)
        .hash_name_reference(Some(&spaced))
        .unwrap();
    assert_eq!(result.author_form(), HtmlHashNameAuthorForm::NoLeadingHash);
    assert_eq!(
        result.processing_name_units(),
        Some(DocumentString::from(" A%20").code_units())
    );
}

#[test]
fn full_original_raw_cap_precedes_an_early_hash() {
    let raw = DocumentString::from("#a-long-tail");
    assert!(
        HtmlValueInterpreter::new(12)
            .hash_name_reference(Some(&raw))
            .is_ok()
    );
    let error = HtmlValueInterpreter::new(11)
        .hash_name_reference(Some(&raw))
        .unwrap_err();
    assert_eq!(error.capacity(), 11);
    assert_eq!(error.required(), 12);
    assert!(
        HtmlValueInterpreter::new(0)
            .hash_name_reference(None)
            .is_ok()
    );
    assert!(
        HtmlValueInterpreter::new(0)
            .hash_name_reference(Some(&DocumentString::from("")))
            .is_ok()
    );
}

#[test]
fn native_symbolic_references_emit_literal_hash_and_exact_checked_token() {
    for units in [vec![35], vec![0xd800, 0, 0x00a0], vec![65, 37, 50, 48]] {
        let name = DocumentString::from_utf16(units.clone());
        let reference =
            HtmlUsemapReference::for_map_name(HtmlSpaceToken::new(name.clone()).unwrap());
        assert_eq!(reference.map_name().value(), &name);
        let attribute = reference.into_attribute();
        let mut expected = vec![35];
        expected.extend(units);
        assert_eq!(
            attribute.name(),
            &KnownHtmlAttribute::Usemap.qualified_name()
        );
        assert_eq!(attribute.value().code_units(), expected);
        let local = HtmlValueInterpreter::new(100)
            .hash_name_reference(Some(attribute.value()))
            .unwrap();
        assert_eq!(local.author_form(), HtmlHashNameAuthorForm::LeadingHashName);
        assert_eq!(local.author_name_units(), Some(name.code_units()));
    }
    assert_eq!(
        HtmlSpaceToken::new(""),
        Err(HtmlTokenError::EmptySpaceToken)
    );
    assert!(matches!(
        HtmlSpaceToken::new("a b"),
        Err(HtmlTokenError::SpaceTokenWhitespace {
            code_unit_offset: 1
        })
    ));
}

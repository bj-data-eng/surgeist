use surgeist_html::{DocumentString, HtmlSource, HtmlSourceCoordinates, HtmlSourceRangeError};

#[test]
fn equal_text_is_not_source_identity_and_clones_share_identity() {
    let first = HtmlSource::utf8("same");
    let clone = first.clone();
    let other = HtmlSource::utf8("same");
    assert_eq!(first.identity(), clone.identity());
    assert_ne!(first.identity(), other.identity());
    let range = first.range(0, 4).unwrap();
    assert_eq!(clone.text(&range).unwrap().to_utf8().unwrap(), "same");
    assert_eq!(other.text(&range), Err(HtmlSourceRangeError::ForeignSource));
}

#[test]
fn utf8_coordinates_preserve_supplied_bom_and_require_scalar_boundaries() {
    let source = HtmlSource::utf8("\u{feff}a\u{1f642}\r\n");
    assert_eq!(source.coordinates(), HtmlSourceCoordinates::Utf8Bytes);
    assert_eq!(source.len(), 10);
    assert_eq!(source.as_utf8(), Some("\u{feff}a\u{1f642}\r\n"));
    assert_eq!(source.as_utf16(), None);
    assert_eq!(
        source
            .text(&source.range(0, 3).unwrap())
            .unwrap()
            .code_units(),
        &[0xfeff]
    );
    assert_eq!(
        source
            .text(&source.range(4, 8).unwrap())
            .unwrap()
            .code_units(),
        &[0xd83d, 0xde42]
    );
    assert_eq!(
        source
            .text(&source.range(8, 10).unwrap())
            .unwrap()
            .to_utf8()
            .unwrap(),
        "\r\n"
    );
    for offset in [1, 2, 5, 6, 7] {
        assert_eq!(
            source.range(offset, offset),
            Err(HtmlSourceRangeError::Utf8Boundary { offset })
        );
    }
    let anchor = source.range(10, 10).unwrap();
    assert!(anchor.is_empty());
    assert_eq!(anchor.len(), 0);
    assert_eq!(anchor.start(), 10);
    assert_eq!(anchor.end(), 10);
    assert_eq!(anchor.source(), source.identity());
    assert_eq!(anchor.coordinates(), HtmlSourceCoordinates::Utf8Bytes);
}

#[test]
fn utf16_coordinates_preserve_pairs_isolated_surrogates_and_unit_slices() {
    let units = DocumentString::from_utf16(vec![0xfeff, 0xd83d, 0xde42, 0xd800, 0, 0xdc00]);
    let source = HtmlSource::utf16(units.clone());
    assert_eq!(source.coordinates(), HtmlSourceCoordinates::Utf16CodeUnits);
    assert_eq!(source.len(), 6);
    assert_eq!(source.as_utf16(), Some(&units));
    assert_eq!(source.as_utf8(), None);
    assert_eq!(source.text(&source.range(0, 6).unwrap()).unwrap(), units);
    assert_eq!(
        source
            .text(&source.range(1, 2).unwrap())
            .unwrap()
            .code_units(),
        &[0xd83d]
    );
    assert_eq!(
        source
            .text(&source.range(2, 3).unwrap())
            .unwrap()
            .code_units(),
        &[0xde42]
    );
    assert_eq!(
        source
            .text(&source.range(3, 6).unwrap())
            .unwrap()
            .code_units(),
        &[0xd800, 0, 0xdc00]
    );
}

#[test]
fn invalid_ranges_are_typed_and_empty_source_accepts_its_end_anchor() {
    for source in [
        HtmlSource::utf8(""),
        HtmlSource::utf16(DocumentString::default()),
    ] {
        assert!(source.is_empty());
        assert!(
            source
                .text(&source.range(0, 0).unwrap())
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            source.range(1, 0),
            Err(HtmlSourceRangeError::Reversed { start: 1, end: 0 })
        );
        assert_eq!(
            source.range(0, 1),
            Err(HtmlSourceRangeError::OutOfBounds { end: 1, length: 0 })
        );
    }
}

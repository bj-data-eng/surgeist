use surgeist_html::{
    DocumentString, HtmlInputColorValueSource, input_color_standard_failure_fallback,
    input_color_value_source,
};

#[test]
fn dirty_color_reads_actual_current_utf16_without_reading_default() {
    // HTML877d update-a-color-well-control-color step 2.2. In particular an
    // unavailable default cannot force a failure when current is selected.
    let current = DocumentString::from_utf16(vec![0x72, 0xd800, 0, 0xdc00]);
    let selected = match input_color_value_source(true) {
        HtmlInputColorValueSource::CurrentValue => &current,
        HtmlInputColorValueSource::ValueAttribute => panic!("default was not requested"),
    };
    assert_eq!(selected.code_units(), &[0x72, 0xd800, 0, 0xdc00]);
    assert_eq!(
        input_color_value_source(false),
        HtmlInputColorValueSource::ValueAttribute
    );
}

#[test]
fn clean_color_selects_missing_empty_or_exact_content_without_current_fallback() {
    // Step 2.1 returns the actual attribute result. Neither its absence nor an
    // empty value means use the current value; CSS parsing follows separately.
    let current = DocumentString::from("#ff0000");
    let empty = DocumentString::from("");
    let content = DocumentString::from_utf16(vec![0x20, 0xd800, 0, 0x20]);
    for (attribute, expected) in [
        (None, None),
        (Some(&empty), Some(&[][..])),
        (Some(&content), Some(&[0x20, 0xd800, 0, 0x20][..])),
    ] {
        let selected = match input_color_value_source(false) {
            HtmlInputColorValueSource::CurrentValue => Some(&current),
            HtmlInputColorValueSource::ValueAttribute => attribute,
        };
        assert_eq!(selected.map(DocumentString::code_units), expected);
    }
}

#[test]
fn standard_invalid_color_selects_checked_opaque_srgb_black() {
    // HTML877d step 4's literal opaque black. No legacy grammar or spelling is
    // invoked; the existing normalized value domain proves opaque RGB8.
    let fallback = input_color_standard_failure_fallback();
    assert_eq!(fallback.channels(), [0, 0, 0]);
}

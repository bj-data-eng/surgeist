use surgeist_html::{
    attributes::input_colorspace::HtmlInputColorSpace,
    bounded::input::input_colorspace_with,
    elements::input::{
        HtmlInputColorValueSource, input_color_standard_failure_fallback, input_color_value_source,
    },
    values::DocumentString,
};

#[test]
fn colorspace_admits_only_reached_actions_on_exact_utf16() {
    use HtmlInputColorSpace::{DisplayP3, LimitedSrgb};

    let display = DocumentString::from("dIsPlAy-P3");
    let limited = DocumentString::from("LiMiTeD-sRgB");
    let padded = DocumentString::from(" display-p3 ");
    for (raw, expected, steps) in [
        (None, LimitedSrgb, 1),
        (Some(&[][..]), LimitedSrgb, 5),
        (Some(display.code_units()), DisplayP3, 14),
        (Some(limited.code_units()), LimitedSrgb, 14),
        (Some(padded.code_units()), LimitedSrgb, 6),
        (Some(&[0xd800; 10][..]), LimitedSrgb, 6),
    ] {
        for limit in 0..=steps {
            let mut reached = 0;
            let state = input_colorspace_with(raw, || {
                if reached == limit {
                    return Err(reached);
                }
                reached += 1;
                Ok(())
            });
            let outcome = if limit == steps {
                Ok(expected)
            } else {
                Err(limit)
            };
            assert_eq!(state, outcome, "{raw:?}");
            assert_eq!(reached, limit);
        }
    }
}

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

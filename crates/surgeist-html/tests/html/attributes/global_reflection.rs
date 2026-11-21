use std::convert::Infallible;
use surgeist_html::bounded::attributes::{
    dir_keyword_with, popover_keyword_with, template_shadow_root_mode_keyword_with,
    template_shadow_root_slot_assignment_keyword_with,
};

#[test]
fn local_reflection_keywords_keep_null_and_empty_distinct() {
    assert_eq!(
        dir_keyword_with(None, |_| Ok::<_, Infallible>(())).unwrap(),
        ""
    );
    assert_eq!(
        popover_keyword_with(None, |_| Ok::<_, Infallible>(())).unwrap(),
        None
    );
    for (raw, direction, popover) in [
        ("", "", Some("auto")),
        ("LTR", "ltr", Some("manual")),
        ("rTl", "rtl", Some("manual")),
        ("AUTO", "auto", Some("auto")),
        ("MANUAL", "", Some("manual")),
        ("HiNt", "", Some("hint")),
        (" auto", "", Some("manual")),
        ("unknown", "", Some("manual")),
    ] {
        let units: Vec<_> = raw.encode_utf16().collect();
        assert_eq!(
            dir_keyword_with(Some(&units), |_| Ok::<_, Infallible>(())).unwrap(),
            direction
        );
        assert_eq!(
            popover_keyword_with(Some(&units), |_| Ok::<_, Infallible>(())).unwrap(),
            popover
        );
    }
    for units in [vec![0xd800], vec![0xdc00], vec![0x61, 0x75, 0x74, 0x6f, 0]] {
        assert_eq!(
            dir_keyword_with(Some(&units), |_| Ok::<_, Infallible>(())).unwrap(),
            ""
        );
        assert_eq!(
            popover_keyword_with(Some(&units), |_| Ok::<_, Infallible>(())).unwrap(),
            Some("manual")
        );
    }
    assert_eq!(
        template_shadow_root_mode_keyword_with(None, |_| Ok::<_, Infallible>(())).unwrap(),
        ""
    );
    assert_eq!(
        template_shadow_root_slot_assignment_keyword_with(None, |_| Ok::<_, Infallible>(()))
            .unwrap(),
        "named"
    );
}

#[test]
fn reflection_admission_stops_at_each_reached_read_and_preserves_exact_budgets() {
    // Existing Template projection admits one presence read, then the slot
    // assignment default's first state header. Mode has no missing default.
    let mut mode_reads = 0;
    assert_eq!(
        template_shadow_root_mode_keyword_with(None, |_| {
            mode_reads += 1;
            Ok::<_, Infallible>(())
        })
        .unwrap(),
        ""
    );
    assert_eq!(mode_reads, 1);
    let mut slot_reads = 0;
    assert_eq!(
        template_shadow_root_slot_assignment_keyword_with(None, |_| {
            slot_reads += 1;
            Ok::<_, Infallible>(())
        })
        .unwrap(),
        "named"
    );
    assert_eq!(slot_reads, 2);
    let ltr: Vec<_> = "LTR".encode_utf16().collect();
    // The dir match admits presence, one keyword header and three pairs.
    for (budget, accepted) in [(5, true), (4, false), (0, false)] {
        let mut remaining: usize = budget;
        let result = dir_keyword_with(Some(&ltr), |_| {
            remaining = remaining.checked_sub(1).ok_or("work")?;
            Ok(())
        });
        assert_eq!(result, if accepted { Ok("ltr") } else { Err("work") });
        assert_eq!(remaining, 0);
    }
    // Empty popover reads presence and three length-mismatched keyword
    // headers, then the first canonical default's state header.
    for (budget, accepted) in [(5, true), (4, false), (0, false)] {
        let mut remaining: usize = budget;
        let result = popover_keyword_with(Some(&[]), |_| {
            remaining = remaining.checked_sub(1).ok_or("work")?;
            Ok(())
        });
        assert_eq!(
            result,
            if accepted {
                Ok(Some("auto"))
            } else {
                Err("work")
            }
        );
        assert_eq!(remaining, 0);
    }
    for raw in [
        None,
        Some(Vec::new()),
        Some(ltr),
        Some("HiNt".encode_utf16().collect()),
    ] {
        for popover in [false, true] {
            let mut reads = Vec::new();
            let mut observe = |read| {
                reads.push(read);
                Ok::<_, Infallible>(())
            };
            if popover {
                popover_keyword_with(raw.as_deref(), &mut observe).unwrap();
            } else {
                dir_keyword_with(raw.as_deref(), &mut observe).unwrap();
            }
            for (cut, &refused) in reads.iter().enumerate() {
                let mut reached = Vec::new();
                let mut admit = |read| {
                    reached.push(read);
                    if reached.len() == cut + 1 {
                        Err((cut, read))
                    } else {
                        Ok(())
                    }
                };
                let result = if popover {
                    popover_keyword_with(raw.as_deref(), &mut admit).map(|_| ())
                } else {
                    dir_keyword_with(raw.as_deref(), &mut admit).map(|_| ())
                };
                assert_eq!(result, Err((cut, refused)));
                assert_eq!(reached, reads[..=cut]);
            }
        }
    }
}

#[test]
fn content_editable_conversion_keeps_content_defaults_out_of_the_setter() {
    use surgeist_html::{
        attributes::{HtmlContentEditable as Keyword, HtmlContentEditableState as State},
        bounded::attributes::{contenteditable_processing_with, contenteditable_setter_state_with},
    };

    assert_eq!(
        contenteditable_processing_with(None, |_| Ok::<_, Infallible>(())).unwrap(),
        State::Inherit
    );
    for (raw, content, setter) in [
        ("", State::Keyword(Keyword::True), None),
        ("INHERIT", State::Inherit, Some(State::Inherit)),
        (
            "TRUE",
            State::Keyword(Keyword::True),
            Some(State::Keyword(Keyword::True)),
        ),
        (
            "FaLsE",
            State::Keyword(Keyword::False),
            Some(State::Keyword(Keyword::False)),
        ),
        (
            "PLainText-ONly",
            State::Keyword(Keyword::PlaintextOnly),
            Some(State::Keyword(Keyword::PlaintextOnly)),
        ),
        ("true ", State::Inherit, None),
        ("inherit\0", State::Inherit, None),
    ] {
        let units: Vec<_> = raw.encode_utf16().collect();
        assert_eq!(
            contenteditable_processing_with(Some(&units), |_| Ok::<_, Infallible>(())).unwrap(),
            content,
            "content {raw:?}"
        );
        assert_eq!(
            contenteditable_setter_state_with(&units, |_| Ok::<_, Infallible>(())).unwrap(),
            setter,
            "setter {raw:?}"
        );
    }
}

#[test]
fn editing_keyword_read_and_set_preserve_the_first_refusal() {
    use surgeist_html::bounded::attributes::{
        HtmlAttributeKeywordRead, contenteditable_setter_state_with, enterkeyhint_keyword_with,
        inputmode_keyword_with,
    };

    let hint: Vec<_> = "PREVIOUS".encode_utf16().collect();
    let mode: Vec<_> = "DECIMAL".encode_utf16().collect();
    assert_eq!(
        enterkeyhint_keyword_with(Some(&hint), |_| Ok::<_, Infallible>(())).unwrap(),
        "previous"
    );
    assert_eq!(
        inputmode_keyword_with(Some(&mode), |_| Ok::<_, Infallible>(())).unwrap(),
        "decimal"
    );
    let invalid = [0xd800, 0, 0xdc00];
    assert_eq!(
        enterkeyhint_keyword_with(Some(&invalid), |_| Ok::<_, Infallible>(())).unwrap(),
        ""
    );
    assert_eq!(
        inputmode_keyword_with(Some(&invalid), |_| Ok::<_, Infallible>(())).unwrap(),
        ""
    );
    for result in [
        enterkeyhint_keyword_with(Some(&hint), Err).map(|_| ()),
        inputmode_keyword_with(Some(&mode), Err).map(|_| ()),
        contenteditable_setter_state_with(&[], Err).map(|_| ()),
    ] {
        assert_eq!(result, Err(HtmlAttributeKeywordRead::ValueHeader));
    }
}

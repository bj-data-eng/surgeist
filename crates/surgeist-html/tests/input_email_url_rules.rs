use surgeist_html::{
    HtmlInputEmailMode, HtmlInputRuleWork, HtmlInputSanitization, HtmlInputSanitizationCursor,
    HtmlInputStringKind, HtmlInputType, HtmlInputTypeChange, HtmlInputTypeEffect,
    HtmlInputValueMode,
};

fn sanitize(
    policy: HtmlInputSanitization,
    units: &[u16],
    mode: HtmlInputEmailMode,
) -> (Vec<u16>, Option<usize>) {
    let mut cursor = HtmlInputSanitizationCursor::new(policy, units, mode);
    let mut result = Vec::new();
    while let Some(work) = cursor.required_work() {
        assert!(matches!(
            work,
            HtmlInputRuleWork::ReadField | HtmlInputRuleWork::InspectValueUnit
        ));
        if let Some(range) = cursor.advance() {
            for &unit in &units[range.start()..range.end()] {
                if cursor.retain_unit(unit) {
                    result.push(unit);
                }
            }
        }
    }
    (result, cursor.email_value_count())
}

#[test]
fn url_and_email_are_value_states_with_distinct_selection_support() {
    for (state, kind, selectable) in [
        (HtmlInputType::Url, HtmlInputStringKind::Url, true),
        (HtmlInputType::Email, HtmlInputStringKind::Email, false),
    ] {
        assert_eq!(HtmlInputStringKind::try_from(state).unwrap(), kind);
        assert_eq!(kind.input_type(), state);
        assert_eq!(kind.value_mode(), HtmlInputValueMode::Value);
        assert_eq!(kind.is_selectable(), selectable);
    }
}

#[test]
fn single_email_and_url_remove_crlf_then_only_boundary_ascii_whitespace() {
    let raw = [32, 9, 65, 13, 66, 10, 67, 12, 32];
    for kind in [HtmlInputStringKind::Email, HtmlInputStringKind::Url] {
        assert_eq!(
            sanitize(kind.sanitization(), &raw, HtmlInputEmailMode::Single),
            (vec![65, 66, 67], None)
        );
        assert_eq!(
            sanitize(
                kind.sanitization(),
                &[32, 9, 10, 13, 12],
                HtmlInputEmailMode::Single
            ),
            (vec![], None)
        );
        assert_eq!(
            sanitize(
                kind.sanitization(),
                &[0xa0, 32, 65, 9, 66, 32, 0xa0],
                HtmlInputEmailMode::Single
            ),
            (vec![0xa0, 32, 65, 9, 66, 32, 0xa0], None)
        );
    }
}

#[test]
fn multiple_email_preserves_actual_empty_field_count_after_joining() {
    for (raw, expected, count) in [
        ("", "", 0),
        (" ", "", 1),
        ("a,", "a", 1),
        ("a, ", "a,", 2),
        (",,", ",", 2),
        (", , ", ",,", 3),
        (" a\n@b.test , \tc@d.test ", "a\n@b.test,c@d.test", 2),
        ("a\r@b.test, c", "a\r@b.test,c", 2),
    ] {
        assert_eq!(
            sanitize(
                HtmlInputSanitization::Email,
                &raw.encode_utf16().collect::<Vec<_>>(),
                HtmlInputEmailMode::Multiple
            ),
            (expected.encode_utf16().collect::<Vec<_>>(), Some(count)),
            "{raw:?}"
        );
    }
}

#[test]
fn url_ignores_email_mode_and_all_sanitizers_preserve_surrogate_units() {
    let raw = [32, 0xd800, 13, 0xdc00, 44, 32, 0xa0, 10, 32];
    assert_eq!(
        sanitize(
            HtmlInputSanitization::StripCrLfAndTrim,
            &raw,
            HtmlInputEmailMode::Multiple
        ),
        (vec![0xd800, 0xdc00, 44, 32, 0xa0], None)
    );
    assert_eq!(
        sanitize(
            HtmlInputSanitization::Email,
            &raw,
            HtmlInputEmailMode::Multiple
        ),
        (vec![0xd800, 13, 0xdc00, 44, 0xa0], Some(2))
    );
}

#[test]
fn email_to_url_initializes_cursor_while_url_to_email_does_not() {
    fn effects(old: HtmlInputStringKind, next: HtmlInputStringKind) -> Vec<HtmlInputTypeEffect> {
        let mut cursor = HtmlInputTypeChange::new(old, next, 3);
        std::iter::from_fn(|| cursor.next_effect()).collect()
    }
    assert_eq!(
        effects(HtmlInputStringKind::Email, HtmlInputStringKind::Url),
        vec![
            HtmlInputTypeEffect::SetRendering,
            HtmlInputTypeEffect::SignalTypeChange,
            HtmlInputTypeEffect::Sanitize(HtmlInputSanitization::StripCrLfAndTrim),
            HtmlInputTypeEffect::SetInitialCursorAndNoneDirection,
        ]
    );
    assert_eq!(
        effects(HtmlInputStringKind::Url, HtmlInputStringKind::Email),
        vec![
            HtmlInputTypeEffect::SetRendering,
            HtmlInputTypeEffect::SignalTypeChange,
            HtmlInputTypeEffect::Sanitize(HtmlInputSanitization::Email),
        ]
    );
}

#[test]
fn a_cursor_has_no_final_email_count_before_its_real_completion() {
    let mut cursor = HtmlInputSanitizationCursor::new(
        HtmlInputSanitization::Email,
        &[32, 44, 32],
        HtmlInputEmailMode::Multiple,
    );
    assert_eq!(cursor.email_value_count(), None);
    let mut actions = 0;
    while cursor.required_work().is_some() {
        cursor.advance();
        actions += 1;
    }
    assert!(actions > 3);
    assert_eq!(cursor.email_value_count(), Some(2));
    assert_eq!(cursor.advance(), None);
    assert_eq!(cursor.required_work(), None);
}

use surgeist_html::HtmlInputValueSanitization;
use surgeist_html::{
    DocumentString, HtmlInputEmailMode, HtmlInputFlags, HtmlInputKind, HtmlInputSanitizationCursor,
    HtmlInputTemporalFormat, HtmlInputType, HtmlInputValueMode,
};

fn sanitized(kind: HtmlInputKind, input: &[u16]) -> Vec<u16> {
    let mut cursor = HtmlInputSanitizationCursor::new(
        string_policy(kind.sanitization()),
        input,
        HtmlInputEmailMode::Single,
    );
    let mut output = Vec::new();
    // The owner guarantees each action is one unit inspection or fixed action.
    // This source-length bound prevents a nonterminating rule test; it is not
    // a claim that adapters can skip per-step admissions.
    let mut remaining = input.len() * 4 + 64;
    while cursor.required_work().is_some() {
        assert!(remaining != 0);
        remaining -= 1;
        if let Some(range) = cursor.advance() {
            for (position, &unit) in input
                .iter()
                .enumerate()
                .take(range.end())
                .skip(range.start())
            {
                if cursor.retain_unit(unit) {
                    output.push(cursor.replacement_unit(position).unwrap_or(unit));
                }
            }
        }
    }
    assert_eq!(cursor.email_value_count(), None);
    assert!(cursor.advance().is_none());
    output
}

#[test]
fn temporal_states_are_value_mode_and_independently_nonselectable() {
    for (state, kind, format) in [
        (
            HtmlInputType::Date,
            HtmlInputKind::Date,
            HtmlInputTemporalFormat::Date,
        ),
        (
            HtmlInputType::Month,
            HtmlInputKind::Month,
            HtmlInputTemporalFormat::Month,
        ),
        (
            HtmlInputType::Week,
            HtmlInputKind::Week,
            HtmlInputTemporalFormat::Week,
        ),
        (
            HtmlInputType::Time,
            HtmlInputKind::Time,
            HtmlInputTemporalFormat::Time,
        ),
        (
            HtmlInputType::LocalDateTime,
            HtmlInputKind::LocalDateTime,
            HtmlInputTemporalFormat::LocalDateTime,
        ),
    ] {
        assert_eq!(HtmlInputKind::try_from(state).unwrap(), kind);
        assert_eq!(kind.input_type(), state);
        assert_eq!(kind.value_mode(), HtmlInputValueMode::Value);
        assert!(!kind.is_selectable());
        assert_eq!(
            string_policy(kind.sanitization()),
            surgeist_html::HtmlInputSanitization::Temporal(format)
        );
        let flags = HtmlInputFlags::fresh().type_changed(kind);
        assert_eq!(flags.kind(), kind);
        assert!(!flags.dirty_value());
    }
}

#[test]
fn strict_date_uses_exact_gregorian_year_and_preserves_original_digits() {
    for (input, expected) in [
        ("2024-02-29", "2024-02-29"),
        ("2023-02-29", ""),
        ("1900-02-29", ""),
        ("2000-02-29", "2000-02-29"),
        ("002024-02-29", "002024-02-29"),
        (
            "10000000000000000000000000000000000000000000400-02-29",
            "10000000000000000000000000000000000000000000400-02-29",
        ),
        ("0000-01-01", ""),
        ("+2024-01-01", ""),
        ("2024-04-31", ""),
        ("2024-02-29 ", ""),
        ("２０２４-02-29", ""),
    ] {
        assert_eq!(
            sanitized(
                HtmlInputKind::Date,
                DocumentString::from(input).code_units()
            ),
            expected.encode_utf16().collect::<Vec<_>>(),
            "{input}"
        );
    }
}

#[test]
fn month_and_week_use_strict_component_and_year_specific_bounds() {
    for (kind, input, expected) in [
        (HtmlInputKind::Month, "2024-12", "2024-12"),
        (HtmlInputKind::Month, "2024-13", ""),
        (HtmlInputKind::Month, "2024-1", ""),
        (HtmlInputKind::Week, "2020-W53", "2020-W53"),
        (HtmlInputKind::Week, "2021-W53", ""),
        (HtmlInputKind::Week, "2020-w53", ""),
        (HtmlInputKind::Week, "2020-W00", ""),
        (HtmlInputKind::Week, "002020-W53", "002020-W53"),
    ] {
        assert_eq!(
            sanitized(kind, DocumentString::from(input).code_units()),
            expected.encode_utf16().collect::<Vec<_>>(),
            "{input}"
        );
    }
}

#[test]
fn strict_time_preserves_optional_zero_seconds_but_rejects_recovery_only_precision() {
    for (input, expected) in [
        ("12:34", "12:34"),
        ("12:34:00.000", "12:34:00.000"),
        ("12:34:56.123", "12:34:56.123"),
        ("12:34:56.1234", ""),
        ("24:00", ""),
        ("23:59:60", ""),
        ("12:34:00.", ""),
        ("12:34:000", ""),
        (" 12:34", ""),
        ("12:34Z", ""),
    ] {
        assert_eq!(
            sanitized(
                HtmlInputKind::Time,
                DocumentString::from(input).code_units()
            ),
            expected.encode_utf16().collect::<Vec<_>>(),
            "{input}"
        );
    }
}

#[test]
fn local_datetime_uses_shortest_time_with_exact_date_spelling_and_t_separator() {
    for (input, expected) in [
        ("2024-02-29 12:34:00.000", "2024-02-29T12:34"),
        ("002024-02-29T12:34:01.120", "002024-02-29T12:34:01.12"),
        ("2024-02-29T12:34:00.001", "2024-02-29T12:34:00.001"),
        ("2024-02-29T12:34:00.100", "2024-02-29T12:34:00.1"),
        ("2024-02-29t12:34", ""),
        ("2024-02-29T12:34Z", ""),
        ("2023-02-29T12:34", ""),
        ("2024-02-29T12:34:00.1234", ""),
    ] {
        assert_eq!(
            sanitized(
                HtmlInputKind::LocalDateTime,
                DocumentString::from(input).code_units()
            ),
            expected.encode_utf16().collect::<Vec<_>>(),
            "{input}"
        );
    }
}

#[test]
fn temporal_empty_null_and_isolated_surrogates_are_not_trimmed_or_repaired() {
    for kind in [
        HtmlInputKind::Date,
        HtmlInputKind::Month,
        HtmlInputKind::Week,
        HtmlInputKind::Time,
        HtmlInputKind::LocalDateTime,
    ] {
        for input in [&[][..], &[0][..], &[0xd800][..], &[0xdc00][..], &[0x20][..]] {
            assert!(sanitized(kind, input).is_empty());
        }
    }
}

fn string_policy(policy: HtmlInputValueSanitization) -> surgeist_html::HtmlInputSanitization {
    match policy {
        HtmlInputValueSanitization::String(policy) => policy,
        HtmlInputValueSanitization::Range => {
            panic!("these literal cases require the string operation")
        }
    }
}

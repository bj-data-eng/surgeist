//! CSS Syntax preprocessing and Color Adjustment authored-property contracts.

use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::property_admission::{
    ValueTextRejection::{Closure, Component, Grammar, Syntax},
    assert_value_text_rejected,
};
use std::{cell::Cell, sync::Arc};
use surgeist_css::*;

fn limits() -> CssUtf16PropertyValueLimits {
    CssUtf16PropertyValueLimits {
        input: CssUtf16InputLimits::default(),
        max_work: 1_000_000,
        max_live_bytes: 2 * 1024 * 1024,
        max_retained_bytes: 2 * 1024 * 1024,
    }
}

fn parse_in(context: CssParserContext, units: Vec<u16>) -> CssColorSchemeParseReport {
    CssColorScheme::parse_utf16(
        &CssParser::new(context, Default::default()),
        Arc::new(units),
        limits(),
        |_| Ok::<_, ()>(()),
        |_| {},
    )
    .unwrap()
}

fn parse(text: &str) -> CssColorSchemeParseReport {
    parse_in(CssParserContext::default(), text.encode_utf16().collect())
}

#[test]
fn concrete_scheme_values_preserve_order_case_repetition_and_only() {
    for (input, expected, only, normal) in [
        ("normal", vec![], false, true),
        ("light dark", vec!["light", "dark"], false, false),
        (
            "only DARK light light",
            vec!["dark", "light", "light"],
            true,
            false,
        ),
        ("Future future only", vec!["Future", "future"], true, false),
        ("none", vec!["none"], false, false),
    ] {
        let report = parse(input);
        let declaration = report.result().as_ref().unwrap();
        assert_eq!(declaration.importance(), CssImportance::Normal);
        let known = declaration.known().unwrap();
        assert_eq!(known.property(), CssKnownProperty::ColorScheme);
        let CssKnownPropertyValueRef::ColorScheme(value) = known.property_value().unwrap() else {
            panic!("typed color-scheme: {input}");
        };
        assert_eq!(value.value().only(), only);
        assert_eq!(value.value().is_normal(), normal);
        let actual: Vec<_> = value
            .value()
            .schemes()
            .iter()
            .map(|entry| match entry {
                CssColorSchemeKeyword::Light => "light",
                CssColorSchemeKeyword::Dark => "dark",
                CssColorSchemeKeyword::Custom(name) => name.as_str(),
                other => panic!("unexpected scheme keyword: {other:?}"),
            })
            .collect();
        assert_eq!(actual, expected, "{input}");
    }
}

#[test]
fn global_and_substitution_values_remain_authored_and_url_fallbacks_use_css_tokens() {
    for (text, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let report = parse(text);
        assert_eq!(
            report.result().as_ref().unwrap().known().unwrap().global(),
            Some(expected)
        );
    }
    // A var() fallback is an arbitrary component list, not another color-scheme
    // value. Valid URL tokens must not be rejected by a color-only lexer.
    for text in [
        "var(--theme)",
        "var(--theme, url(asset))",
        "var(--theme, url(   \"asset\"))",
        "var(--theme, url(a\\)b))",
        "var(--theme, f([1px, #abc]))",
        "env(future calc(2.5))",
        "attr(foo \"<length>\")",
        r#"attr(foo "\"<length>\"")"#,
    ] {
        let report = parse(text);
        let declaration = report.result().as_ref().unwrap();
        assert!(
            matches!(declaration.expand().unwrap(), CssExpansion::Pending(_)),
            "{text}"
        );
        assert_eq!(
            report
                .source()
                .source()
                .original_utf16()
                .unwrap()
                .as_slice(),
            text.encode_utf16().collect::<Vec<_>>()
        );
    }
}

#[test]
fn decoded_attr_syntax_treats_escaped_css_newlines_as_whitespace() {
    for text in [
        r#"attr(foo "\d <length>")"#,
        r#"attr(foo "\c <length>")"#,
        r#"attr(foo "<length>\d |\c <color>")"#,
    ] {
        let report = parse(text);
        let declaration = report
            .result()
            .as_ref()
            .unwrap_or_else(|error| panic!("valid decoded syntax {text:?}: {error:?}"));
        assert!(matches!(
            declaration.expand().unwrap(),
            CssExpansion::Pending(_)
        ));
        assert_eq!(
            report
                .source()
                .source()
                .original_utf16()
                .unwrap()
                .as_slice(),
            text.encode_utf16().collect::<Vec<_>>()
        );
    }
}

#[test]
fn invalid_values_and_recovered_components_are_syntax_failures_with_original_source() {
    for (text, expected) in [
        ("", Grammar(None)),
        (" \t\n", Grammar(None)),
        ("only", Grammar(Some("only"))),
        ("normal dark", Grammar(Some("dark"))),
        ("light !important", Syntax("!")),
        ("light;dark", Syntax(";")),
        (
            "light}",
            Component(ComponentFailure::UnmatchedClosingDelimiter),
        ),
        ("var(--theme", Closure),
        (
            "var(--theme, url(a\"b))",
            Component(ComponentFailure::BadUrl),
        ),
        (
            "var(--theme, \"bad\n)",
            Component(ComponentFailure::BadString),
        ),
    ] {
        let report = parse(text);
        assert_value_text_rejected(report.result(), CssKnownProperty::ColorScheme, expected);
        assert_eq!(
            report
                .source()
                .source()
                .original_utf16()
                .unwrap()
                .as_slice(),
            text.encode_utf16().collect::<Vec<_>>()
        );
    }
}

#[test]
fn preprocessing_and_component_origins_keep_exact_original_utf16_intervals() {
    let raw = vec![
        13, 10, 12, 70, 117, 116, 117, 114, 101, 0, 0xd83d, 0xde00, 0xd800,
    ];
    let report = parse_in(CssParserContext::default(), raw.clone());
    let declaration = report.result().as_ref().unwrap();
    let source = report.source().source();
    assert_eq!(source.original_utf16().unwrap().as_slice(), raw);
    assert_eq!(source.as_str(), "\n\nFuture\u{fffd}😀\u{fffd}");
    for (bytes, units) in [
        (0..1, 0..2),
        (1..2, 2..3),
        (8..11, 9..10),
        (11..15, 10..12),
        (15..18, 12..13),
    ] {
        assert_eq!(source.original_utf16_range(bytes), Some(units));
    }
    for item in declaration.value_components().items() {
        let CssValueOrigin::Parsed(origin) = item.origin() else {
            panic!("actual parsed component");
        };
        assert!(origin.source().same_snapshot(source));
        let range =
            origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value();
        assert!(source.original_utf16_range(range).is_some());
    }
}

#[test]
fn meaningful_equivalence_excludes_fresh_identity_but_includes_raw_input_and_context() {
    let first = parse("\r\nlight dark");
    let equal = parse("\r\nlight dark");
    let normalized_equal = parse("\nlight dark");
    let quirks = parse_in(
        CssParserContext::new(CssParserMode::Quirks),
        "\r\nlight dark".encode_utf16().collect(),
    );
    assert!(
        !first
            .source()
            .source()
            .same_snapshot(equal.source().source())
    );
    assert!(first.equivalent_with(&equal, || Ok::<_, ()>(())).unwrap());
    assert_eq!(
        first.source().source().as_str(),
        normalized_equal.source().source().as_str()
    );
    assert!(
        !first
            .equivalent_with(&normalized_equal, || Ok::<_, ()>(()))
            .unwrap()
    );
    assert!(!first.equivalent_with(&quirks, || Ok::<_, ()>(())).unwrap());
}

#[test]
fn padded_original_capacity_and_retained_report_have_separate_limits() {
    let mut raw = Vec::with_capacity(4096);
    raw.extend("dark".encode_utf16());
    let capacity = raw.capacity();
    let report = parse_in(CssParserContext::default(), vec![100, 97, 114, 107]);
    // Equal logical input in independently owned tight and padded allocations.
    let padded = parse_in(CssParserContext::default(), raw);
    assert_eq!(padded.retained_storage().utf16_units(), capacity);
    assert!(padded.retained_storage().bytes() >= 2 * capacity);
    assert!(padded.retained_storage().bytes() > report.retained_storage().bytes());
    let bound = report.retained_storage().bytes();
    for (maximum, succeeds) in [(bound, true), (bound - 1, false)] {
        let final_live = Cell::new(0);
        let result = CssColorScheme::parse_utf16(
            &CssParser::default(),
            Arc::new(vec![100, 97, 114, 107]),
            CssUtf16PropertyValueLimits {
                max_retained_bytes: maximum,
                ..limits()
            },
            |event| {
                if let CssUtf16ValueAdmission::Live(usage) = event {
                    final_live.set(usage.bytes());
                }
                Ok::<_, ()>(())
            },
            |remaining| final_live.set(remaining.bytes()),
        );
        if succeeds {
            let result = result.unwrap();
            assert_eq!(final_live.get(), result.retained_storage().bytes());
        } else {
            assert!(
                result
                    .as_ref()
                    .is_err_and(|error| error.exhausted_limit().is_some())
            );
            assert_eq!(final_live.get(), 0);
        }
    }
}

fn assert_admission_cuts(text: &str) {
    let input: Arc<Vec<_>> = Arc::new(text.encode_utf16().collect());
    let calls = Cell::new(0usize);
    CssColorScheme::parse_utf16(
        &CssParser::default(),
        input.clone(),
        limits(),
        |_| {
            calls.set(calls.get() + 1);
            Ok::<_, usize>(())
        },
        |_| {},
    )
    .unwrap();
    assert!(calls.get() > input.len());
    for refusal in 0..calls.get() {
        let seen = Cell::new(0usize);
        let live = Cell::new(0usize);
        let result = CssColorScheme::parse_utf16(
            &CssParser::default(),
            input.clone(),
            limits(),
            |event| {
                let index = seen.get();
                seen.set(index + 1);
                if index == refusal {
                    return Err(index);
                }
                if let CssUtf16ValueAdmission::Live(usage) = event {
                    live.set(usage.bytes());
                }
                Ok(())
            },
            |remaining| live.set(remaining.bytes()),
        );
        assert!(
            result
                .as_ref()
                .is_err_and(|error| error.interruption().is_some_and(|index| *index == refusal))
        );
        assert_eq!(
            seen.get(),
            refusal + 1,
            "no admission after refusal {refusal}"
        );
        assert_eq!(live.get(), 0, "release after actual drops at {refusal}");
    }
}

#[test]
fn every_admission_cut_returns_original_cause_and_releases_all_actual_storage() {
    for text in [
        "var(--theme, url(a))",
        "env(future calc(2.5))",
        "attr(foo \"<length>\")",
    ] {
        assert_admission_cuts(text);
    }
}

#[test]
fn syntax_diagnostics_preserve_first_refusal_and_release_their_actual_payload() {
    for text in [
        "normal dark",
        "\"unexpected\"",
        "attr(foo \"<not-a-type>\")",
    ] {
        assert!(
            parse(text)
                .result()
                .as_ref()
                .is_err_and(|error| error.code().is_some())
        );
        assert_admission_cuts(text);
    }
    assert!(
        parse("light]")
            .result()
            .as_ref()
            .is_err_and(|error| error.component_error().is_some())
    );
    assert_admission_cuts("light]");
}

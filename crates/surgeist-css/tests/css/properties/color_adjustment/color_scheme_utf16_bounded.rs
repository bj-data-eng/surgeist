//! Functional bounds and retained-content contracts for the property ingress.

use std::{cell::Cell, sync::Arc};
use surgeist_css::*;

fn limits() -> CssUtf16PropertyValueLimits {
    CssUtf16PropertyValueLimits {
        input: CssUtf16InputLimits::default(),
        syntax: CssComponentValueLimits::default(),
        max_work: 4_000_000,
        max_live_bytes: 16 * 1024 * 1024,
        max_retained_bytes: 16 * 1024 * 1024,
    }
}

fn parse(input: Arc<Vec<u16>>) -> CssColorSchemeParseReport {
    CssParserContext::default()
        .parse_color_scheme_utf16_with_limits(input, limits(), |_| Ok::<_, ()>(()), |_| {})
        .unwrap()
}

#[test]
fn long_decoded_names_and_arbitrary_fallback_tokens_use_the_complete_grammar() {
    let name = "Future".repeat(2048);
    let report = parse(Arc::new(format!("{name} only").encode_utf16().collect()));
    let declaration = report.result().as_ref().unwrap();
    let CssKnownPropertyValueRef::ColorScheme(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color-scheme value")
    };
    assert!(value.value().only());
    let [CssColorSchemeKeyword::Custom(actual)] = value.value().schemes() else {
        panic!("one custom scheme")
    };
    assert_eq!(actual.as_str(), name);

    // Substitution fallback tokens are balanced CSS syntax, not a restricted
    // nested color-scheme grammar. Long numbers and escaped URL payloads do not
    // need a platform, cascade, variable lookup or floating-point resolution.
    let number = "9".repeat(2048);
    let fallback = format!("var(--theme, [f({number}px), url(a\\)b), \"{name}\"])");
    let report = parse(Arc::new(fallback.encode_utf16().collect()));
    let declaration = report.result().as_ref().unwrap();
    assert!(
        declaration
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
    assert_eq!(report.source().source().as_str(), fallback);
}

#[test]
fn work_refusal_and_final_output_refusal_release_original_custody_before_zero() {
    for maximum in [0, 1, 64] {
        let input = Arc::new("Future".repeat(1024).encode_utf16().collect::<Vec<_>>());
        let weak = Arc::downgrade(&input);
        let zero = Cell::new(false);
        let result = CssParserContext::default().parse_color_scheme_utf16_with_limits(
            input.clone(),
            CssUtf16PropertyValueLimits {
                max_work: maximum,
                ..limits()
            },
            |_| Ok::<_, ()>(()),
            |usage| {
                if usage == CssUtf16ValueStorage::default() {
                    assert_eq!(
                        weak.strong_count(),
                        1,
                        "all operation-owned source handles ended"
                    );
                    zero.set(true);
                }
            },
        );
        assert!(matches!(result, Err(CssUtf16InputError::Limit(_))));
        assert!(zero.get());
    }
    let input = Arc::new("light".encode_utf16().collect::<Vec<_>>());
    let weak = Arc::downgrade(&input);
    let zero = Cell::new(false);
    let result = CssParserContext::default().parse_color_scheme_utf16_with_limits(
        input.clone(),
        CssUtf16PropertyValueLimits {
            max_retained_bytes: 0,
            ..limits()
        },
        |_| Ok::<_, ()>(()),
        |usage| {
            if usage == CssUtf16ValueStorage::default() {
                assert_eq!(weak.strong_count(), 1);
                zero.set(true);
            }
        },
    );
    assert!(matches!(result, Err(CssUtf16InputError::Limit(_))));
    assert!(zero.get());
}

#[test]
fn initial_admission_refusal_drops_the_real_input_before_infallible_release() {
    let input = Arc::new("dark".encode_utf16().collect::<Vec<_>>());
    let weak = Arc::downgrade(&input);
    let releases = Cell::new(0);
    let result = CssParserContext::default().parse_color_scheme_utf16_with_limits(
        input.clone(),
        limits(),
        |_| Err("first real refusal"),
        |remaining| {
            assert_eq!(remaining, CssUtf16ValueStorage::default());
            assert_eq!(weak.strong_count(), 1);
            releases.set(releases.get() + 1);
        },
    );
    assert!(matches!(
        result,
        Err(CssUtf16InputError::Interrupted("first real refusal"))
    ));
    assert_eq!(releases.get(), 1);
}

#[test]
fn equivalence_uses_logical_content_not_spare_capacity_and_remains_interruptible() {
    let raw: Vec<_> = "Future dark only".encode_utf16().collect();
    let mut padded = Vec::with_capacity(4096);
    padded.extend_from_slice(&raw);
    let first = parse(Arc::new(raw));
    let second = parse(Arc::new(padded));
    assert_ne!(first.retained_storage(), second.retained_storage());
    let observations = Cell::new(0);
    assert!(
        first
            .equivalent_with(&second, || {
                observations.set(observations.get() + 1);
                Ok::<_, usize>(())
            })
            .unwrap()
    );
    assert!(observations.get() > 20);
    let before = (first.retained_storage(), second.retained_storage());
    for stop in 0..observations.get() {
        let calls = Cell::new(0);
        let result = first.equivalent_with(&second, || {
            let index = calls.get();
            calls.set(index + 1);
            if index == stop { Err(index) } else { Ok(()) }
        });
        assert!(matches!(result, Err(CssRetainedPayloadError::Admission(index)) if index == stop));
        assert_eq!(calls.get(), stop + 1);
        assert_eq!(
            (first.retained_storage(), second.retained_storage()),
            before
        );
    }
}

#[test]
fn empty_invalid_input_still_retains_one_complete_original_source() {
    let input = Arc::new(Vec::with_capacity(128));
    let report = parse(input.clone());
    assert!(report.result().is_err());
    assert!(Arc::ptr_eq(
        report.source().source().original_utf16().unwrap(),
        &input
    ));
    assert_eq!(report.retained_storage().utf16_units(), input.capacity());
    let mut sources = CssSourceRetention::new(1_000_000, 16, 8);
    sources.visit(&report).unwrap();
    assert!(
        sources
            .complete_payload_bytes_with(|| Ok::<_, ()>(()))
            .unwrap()
            >= 256
    );
}

#[test]
fn component_resource_limits_are_not_misreported_as_invalid_authored_css() {
    for (text, syntax) in [
        (
            "light dark",
            CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap(),
        ),
        (
            "dark",
            CssComponentValueLimits::try_new(256, usize::MAX, 2).unwrap(),
        ),
        (
            "var(--theme)",
            CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap(),
        ),
    ] {
        let result = CssParserContext::default().parse_color_scheme_utf16_with_limits(
            Arc::new(text.encode_utf16().collect()),
            CssUtf16PropertyValueLimits { syntax, ..limits() },
            |_| Ok::<_, ()>(()),
            |_| {},
        );
        assert!(
            matches!(result, Err(CssUtf16InputError::Limit(_))),
            "{text}"
        );
    }
}

#![forbid(unsafe_code)]

//! Counter Styles 3 (2021-07-27) §3; Fonts 4 (2026-09-07) §§4.9.1, 6.9.2,
//! with the repository's selected exact-integer/cardinality policy.

#[path = "support/descriptor_phases.rs"]
mod descriptor_phases;
use descriptor_phases::*;

use surgeist_css::{
    CssComponentValue, CssComponentValueRef, CssComponentValues,
    CssCounterStyleDescriptorKind as CounterKind, CssCounterStyleDescriptorValue as CounterValue,
    CssCounterStyleDescriptorValueRef as CounterRef, CssCounterStyleRange,
    CssCounterStyleRangeBound, CssCounterStyleSpeakAs, CssCounterStyleSystem, CssCounterSymbol,
    CssErrorCode, CssEscapeError, CssFontDisplay, CssFontFeatureValueKind as FeatureKind,
    CssFontFeatureValuesItem, CssImageValue, CssNumericTokenKind, CssParseReport, CssParsedOrigin,
    CssRecoveryAction, CssRule, CssSourcePosition, CssValueOrigin, CssValueTokenRef, ErrorKind,
    parse_counter_style_descriptor_value, parse_font_feature_display_value,
    parse_font_feature_value, parse_sheet,
};

#[derive(Clone, Copy, Debug)]
enum Symbol {
    String(&'static str),
    Ident(&'static str),
    Url(&'static str),
}

#[derive(Debug)]
enum Expected {
    System(CssCounterStyleSystem),
    Fixed(Option<&'static str>),
    Extends(&'static str),
    Negative(Symbol, Option<Symbol>),
    Symbols(Vec<Symbol>),
    Prefix(Symbol),
    Suffix(Symbol),
    AutoRange,
    // None is contextual infinite, not an absent bound.
    Ranges(Vec<(Option<&'static str>, Option<&'static str>)>),
    Pad(&'static str, Symbol),
    Fallback(&'static str),
    Additive(Vec<(&'static str, Symbol)>),
    Speak(CssCounterStyleSpeakAs),
    SpokenReference(&'static str),
}

fn symbol(actual: &CssCounterSymbol, expected: Symbol) {
    match (actual, expected) {
        (CssCounterSymbol::String(actual), Symbol::String(expected)) => {
            assert_eq!(actual.as_str(), expected);
        }
        (CssCounterSymbol::Ident(actual), Symbol::Ident(expected)) => {
            assert_eq!(actual.as_str(), expected);
        }
        (CssCounterSymbol::Image(actual), Symbol::Url(expected)) => {
            let CssImageValue::Url(url) = actual.value() else {
                panic!("URL image")
            };
            assert_eq!(url.as_str(), expected);
        }
        _ => panic!("wrong symbol: {actual:?}, expected {expected:?}"),
    }
}

fn bound(actual: &CssCounterStyleRangeBound, expected: Option<&str>) {
    match (actual, expected) {
        (CssCounterStyleRangeBound::Infinite, None) => {}
        (CssCounterStyleRangeBound::Integer(actual), Some(expected)) => {
            assert_eq!(actual.numeric().representation(), expected);
            assert_eq!(actual.numeric().kind(), CssNumericTokenKind::Integer);
        }
        _ => panic!("wrong bound: {actual:?}, expected {expected:?}"),
    }
}

fn counter_semantics(actual: &CounterValue, expected: &Expected) {
    match (actual.view(), expected) {
        (CounterRef::System(actual), Expected::System(expected)) => assert_eq!(actual, expected),
        (CounterRef::System(CssCounterStyleSystem::Fixed(actual)), Expected::Fixed(expected)) => {
            assert_eq!(
                actual
                    .first_symbol_value()
                    .map(|n| n.numeric().representation()),
                *expected
            );
            if let Some(number) = actual.first_symbol_value() {
                assert_eq!(number.numeric().kind(), CssNumericTokenKind::Integer);
                assert!(matches!(number.origin(), CssValueOrigin::Parsed(_)));
            }
        }
        (
            CounterRef::System(CssCounterStyleSystem::Extends(actual)),
            Expected::Extends(expected),
        ) => {
            assert_eq!(actual.as_str(), *expected);
        }
        (CounterRef::Negative(actual), Expected::Negative(prefix, suffix)) => {
            symbol(actual.prefix(), *prefix);
            match (actual.suffix(), suffix) {
                (Some(actual), Some(expected)) => symbol(actual, *expected),
                (None, None) => {}
                _ => panic!("wrong negative suffix"),
            }
        }
        (CounterRef::Symbols(actual), Expected::Symbols(expected)) => {
            assert_eq!(actual.symbols().len(), expected.len());
            for (actual, expected) in actual.symbols().iter().zip(expected) {
                symbol(actual, *expected);
            }
        }
        (CounterRef::Prefix(actual), Expected::Prefix(expected))
        | (CounterRef::Suffix(actual), Expected::Suffix(expected)) => symbol(actual, *expected),
        (CounterRef::Range(CssCounterStyleRange::Auto), Expected::AutoRange) => {}
        (CounterRef::Range(CssCounterStyleRange::Ranges(actual)), Expected::Ranges(expected)) => {
            assert_eq!(actual.ranges().len(), expected.len());
            for (actual, (lower, upper)) in actual.ranges().iter().zip(expected) {
                bound(actual.lower(), *lower);
                bound(actual.upper(), *upper);
            }
        }
        (CounterRef::Pad(actual), Expected::Pad(minimum, expected_symbol)) => {
            assert_eq!(actual.minimum_length().numeric().representation(), *minimum);
            assert!(!actual.minimum_length().is_negative());
            symbol(actual.symbol(), *expected_symbol);
        }
        (CounterRef::Fallback(actual), Expected::Fallback(expected)) => {
            assert_eq!(actual.as_str(), *expected)
        }
        (CounterRef::AdditiveSymbols(actual), Expected::Additive(expected)) => {
            assert_eq!(actual.tuples().len(), expected.len());
            for (actual, (weight, expected_symbol)) in actual.tuples().iter().zip(expected) {
                assert_eq!(actual.weight().numeric().representation(), *weight);
                assert!(!actual.weight().is_negative());
                symbol(actual.symbol(), *expected_symbol);
            }
        }
        (CounterRef::SpeakAs(actual), Expected::Speak(expected)) => assert_eq!(actual, expected),
        (
            CounterRef::SpeakAs(CssCounterStyleSpeakAs::CounterStyle(actual)),
            Expected::SpokenReference(expected),
        ) => {
            assert_eq!(actual.as_str(), *expected);
        }
        _ => panic!(
            "wrong typed view: {:?}, expected {expected:?}",
            actual.view()
        ),
    }
}

fn position(actual: CssSourcePosition, byte: usize, line: u32, column: u32) {
    assert_eq!(actual.byte_offset().value(), byte);
    assert_eq!(actual.line().value(), line);
    assert_eq!(actual.column().value(), column);
}

fn origin(
    actual: &CssParsedOrigin,
    source: &str,
    start: (usize, u32, u32),
    end: (usize, u32, u32),
) {
    assert_eq!(actual.source().as_str(), source);
    position(actual.span().start(), start.0, start.1, start.2);
    position(actual.span().end(), end.0, end.1, end.2);
}

fn parsed(component: &CssComponentValue) -> &CssParsedOrigin {
    let CssValueOrigin::Parsed(origin) = component.origin() else {
        panic!("actual source token")
    };
    origin
}

fn complete_components(values: &CssComponentValues, carrier: &CssParsedOrigin, source: &str) {
    // These success cases contain root leaves only. Check raw source coverage,
    // including trivia, without serializing/relexing or predicting token counts.
    let mut next = 0;
    for component in values.items() {
        let component_origin = parsed(component);
        assert!(carrier.source().same_snapshot(component_origin.source()));
        assert_eq!(component_origin.source().as_str(), source);
        assert_eq!(component_origin.span().start().byte_offset().value(), next);
        next = component_origin.span().end().byte_offset().value();
    }
    assert_eq!(next, source.len());
}

fn rejected<T: std::fmt::Debug>(report: CssParseReport<Option<T>>, source: &str) {
    assert!(report.syntax().is_none(), "{source:?}: {report:?}");
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .expect("whole input rejected at its owning grammar boundary");
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    assert!(report.into_validation_result().is_err());
}

const COUNTERS: [(CounterKind, &str, &str); 10] = [
    (CounterKind::System, "system", "cyclic"),
    (CounterKind::Negative, "negative", "\"-\""),
    (CounterKind::Symbols, "symbols", "x"),
    (CounterKind::Prefix, "prefix", "x"),
    (CounterKind::Suffix, "suffix", "\"\""),
    (CounterKind::Range, "range", "auto"),
    (CounterKind::Pad, "pad", "2 \"0\""),
    (CounterKind::Fallback, "fallback", "decimal"),
    (CounterKind::AdditiveSymbols, "additive-symbols", "1 I"),
    (CounterKind::SpeakAs, "speak-as", "words"),
];
const FEATURES: [FeatureKind; 7] = [
    FeatureKind::Stylistic,
    FeatureKind::HistoricalForms,
    FeatureKind::Styleset,
    FeatureKind::CharacterVariant,
    FeatureKind::Swash,
    FeatureKind::Ornaments,
    FeatureKind::Annotation,
];

#[test]
fn ten_counter_descriptor_grammars_return_literal_semantics() {
    use Symbol::{Ident as I, String as S, Url as U};
    let cases = [
        (
            CounterKind::System,
            "cyclic",
            Expected::System(CssCounterStyleSystem::Cyclic),
        ),
        (
            CounterKind::System,
            "numeric",
            Expected::System(CssCounterStyleSystem::Numeric),
        ),
        (
            CounterKind::System,
            "alphabetic",
            Expected::System(CssCounterStyleSystem::Alphabetic),
        ),
        (
            CounterKind::System,
            "symbolic",
            Expected::System(CssCounterStyleSystem::Symbolic),
        ),
        (
            CounterKind::System,
            "additive",
            Expected::System(CssCounterStyleSystem::Additive),
        ),
        (CounterKind::System, "fixed", Expected::Fixed(None)),
        (CounterKind::System, "fixed -2", Expected::Fixed(Some("-2"))),
        (
            CounterKind::System,
            "FiXeD +0002",
            Expected::Fixed(Some("+0002")),
        ),
        (
            CounterKind::System,
            "extends decimal",
            Expected::Extends("decimal"),
        ),
        (
            CounterKind::Negative,
            "\"-\"",
            Expected::Negative(S("-"), None),
        ),
        (
            CounterKind::Negative,
            "\"(\" \")\"",
            Expected::Negative(S("("), Some(S(")"))),
        ),
        (
            CounterKind::Negative,
            "none",
            Expected::Negative(I("none"), None),
        ),
        (
            CounterKind::Symbols,
            "\"0\" none url(icon.svg)",
            Expected::Symbols(vec![S("0"), I("none"), U("icon.svg")]),
        ),
        (CounterKind::Symbols, "x", Expected::Symbols(vec![I("x")])),
        (CounterKind::Prefix, "\"💡\"", Expected::Prefix(S("💡"))),
        (CounterKind::Prefix, "x", Expected::Prefix(I("x"))),
        (
            CounterKind::Prefix,
            "url(icon.svg)",
            Expected::Prefix(U("icon.svg")),
        ),
        (
            CounterKind::Prefix,
            "url(\"x;y!z\")",
            Expected::Prefix(U("x;y!z")),
        ),
        (CounterKind::Suffix, "\"\"", Expected::Suffix(S(""))),
        (CounterKind::Suffix, "\" \"", Expected::Suffix(S(" "))),
        (CounterKind::Suffix, "none", Expected::Suffix(I("none"))),
        (CounterKind::Range, "AuTo", Expected::AutoRange),
        (
            CounterKind::Range,
            "infinite -1, 1 infinite",
            Expected::Ranges(vec![(None, Some("-1")), (Some("1"), None)]),
        ),
        (
            CounterKind::Range,
            "+0002 2",
            Expected::Ranges(vec![(Some("+0002"), Some("2"))]),
        ),
        (
            CounterKind::Range,
            "infinite infinite",
            Expected::Ranges(vec![(None, None)]),
        ),
        (CounterKind::Pad, "2 \"0\"", Expected::Pad("2", S("0"))),
        (CounterKind::Pad, "\"0\" 2", Expected::Pad("2", S("0"))),
        (CounterKind::Pad, "-0 \"\"", Expected::Pad("-0", S(""))),
        (
            CounterKind::Fallback,
            "decimal",
            Expected::Fallback("decimal"),
        ),
        (CounterKind::Fallback, "base", Expected::Fallback("base")),
        (
            CounterKind::Fallback,
            r"b\61 se",
            Expected::Fallback("base"),
        ),
        (CounterKind::Fallback, "BaSe", Expected::Fallback("BaSe")),
        (
            CounterKind::AdditiveSymbols,
            "100 C, 10 X, 1 I, 0 N",
            Expected::Additive(vec![
                ("100", I("C")),
                ("10", I("X")),
                ("1", I("I")),
                ("0", I("N")),
            ]),
        ),
        (
            CounterKind::AdditiveSymbols,
            "C 100, X 10",
            Expected::Additive(vec![("100", I("C")), ("10", I("X"))]),
        ),
        (
            CounterKind::AdditiveSymbols,
            "-0 N",
            Expected::Additive(vec![("-0", I("N"))]),
        ),
        (
            CounterKind::SpeakAs,
            "auto",
            Expected::Speak(CssCounterStyleSpeakAs::Auto),
        ),
        (
            CounterKind::SpeakAs,
            "bullets",
            Expected::Speak(CssCounterStyleSpeakAs::Bullets),
        ),
        (
            CounterKind::SpeakAs,
            "numbers",
            Expected::Speak(CssCounterStyleSpeakAs::Numbers),
        ),
        (
            CounterKind::SpeakAs,
            "WoRdS",
            Expected::Speak(CssCounterStyleSpeakAs::Words),
        ),
        (
            CounterKind::SpeakAs,
            "spell-out",
            Expected::Speak(CssCounterStyleSpeakAs::SpellOut),
        ),
        (
            CounterKind::SpeakAs,
            "decimal",
            Expected::SpokenReference("decimal"),
        ),
        (
            CounterKind::SpeakAs,
            "base",
            Expected::SpokenReference("base"),
        ),
    ];
    for (kind, source, expected) in cases {
        let report = parse_counter_style_descriptor_value(source, kind);
        assert!(report.is_clean(), "{kind:?} {source:?}: {report:?}");
        let actual = report.syntax().as_ref().expect("typed counter value");
        assert_eq!(actual.kind(), kind);
        counter_semantics(actual, &expected);
        assert_eq!(
            actual
                .origin()
                .unwrap()
                .span()
                .start()
                .byte_offset()
                .value(),
            0
        );
        assert_eq!(
            actual.origin().unwrap().span().end().byte_offset().value(),
            source.len()
        );
        assert_eq!(actual.origin().unwrap().source().as_str(), source);
        assert!(report.into_validation_result().unwrap().is_some());
    }
    for (kind, css_name, _) in COUNTERS {
        assert_eq!(kind.css_name(), css_name);
    }
}

#[test]
fn counter_invalid_local_values_reject_without_enclosing_rule_requirements() {
    let cases: &[(CounterKind, &[&str])] = &[
        (
            CounterKind::System,
            &[
                "fixed 1.0",
                "fixed 1 2",
                "extends",
                "extends none",
                "banana",
            ],
        ),
        (CounterKind::Negative, &["\"(\" \")\" \"x\"", "1"]),
        (CounterKind::Symbols, &["1", "inherit", "url(a b)"]),
        (CounterKind::Prefix, &["x y", "1", "inherit"]),
        (CounterKind::Suffix, &["\"x\" \"y\"", "1"]),
        (
            CounterKind::Range,
            &["3 1", "1", "1 2,", "1.0 2", "auto 1 2"],
        ),
        (
            CounterKind::Pad,
            &["-1 \"0\"", "1.0 \"0\"", "2", "2 \"0\" x"],
        ),
        (
            CounterKind::Fallback,
            &["none", "default", "inherit", "base other"],
        ),
        (
            CounterKind::AdditiveSymbols,
            &["10 X, 10 I", "1 I, 10 X", "-1 I", "1.0 I", "1 I,"],
        ),
        (CounterKind::SpeakAs, &["none", "inherit", "words extra"]),
    ];
    for (kind, sources) in cases {
        for source in *sources {
            rejected(parse_counter_style_descriptor_value(source, *kind), source);
        }
    }
}

#[test]
fn exact_counter_weights_and_bounds_exceed_machine_integer_limits() {
    let report =
        parse_counter_style_descriptor_value("fixed 18446744073709551616", CounterKind::System);
    assert!(report.is_clean(), "{report:?}");
    counter_semantics(
        report.syntax().as_ref().unwrap(),
        &Expected::Fixed(Some("18446744073709551616")),
    );
    let report =
        parse_counter_style_descriptor_value("18446744073709551616 \"0\"", CounterKind::Pad);
    assert!(report.is_clean(), "{report:?}");
    counter_semantics(
        report.syntax().as_ref().unwrap(),
        &Expected::Pad("18446744073709551616", Symbol::String("0")),
    );
    let report = parse_counter_style_descriptor_value(
        "18446744073709551617 A, 18446744073709551616 B",
        CounterKind::AdditiveSymbols,
    );
    assert!(report.is_clean(), "{report:?}");
    counter_semantics(
        report.syntax().as_ref().unwrap(),
        &Expected::Additive(vec![
            ("18446744073709551617", Symbol::Ident("A")),
            ("18446744073709551616", Symbol::Ident("B")),
        ]),
    );
    // Adjacent exact values must not collapse to one binary64 magnitude.
    rejected(
        parse_counter_style_descriptor_value(
            "18446744073709551616 A, 18446744073709551617 B",
            CounterKind::AdditiveSymbols,
        ),
        "18446744073709551616 A, 18446744073709551617 B",
    );
    let source = "18446744073709551616 18446744073709551617";
    let report = parse_counter_style_descriptor_value(source, CounterKind::Range);
    assert!(report.is_clean(), "{report:?}");
    counter_semantics(
        report.syntax().as_ref().unwrap(),
        &Expected::Ranges(vec![(
            Some("18446744073709551616"),
            Some("18446744073709551617"),
        )]),
    );
    rejected(
        parse_counter_style_descriptor_value(
            "18446744073709551617 18446744073709551616",
            CounterKind::Range,
        ),
        "18446744073709551617 18446744073709551616",
    );
}

#[test]
fn five_display_values_and_decoded_keyword_spellings_have_literal_enum_values() {
    for (source, expected) in [
        ("auto", CssFontDisplay::Auto),
        ("block", CssFontDisplay::Block),
        ("swap", CssFontDisplay::Swap),
        ("fallback", CssFontDisplay::Fallback),
        ("optional", CssFontDisplay::Optional),
        ("sWaP", CssFontDisplay::Swap),
        (r"s\77 ap", CssFontDisplay::Swap),
    ] {
        let report = parse_font_feature_display_value(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let actual = report.syntax().as_ref().unwrap();
        assert_eq!(actual.ordinary_display(), expected);
        complete_components(actual.components(), actual.origin().unwrap(), source);
        assert_eq!(
            actual
                .origin()
                .unwrap()
                .span()
                .start()
                .byte_offset()
                .value(),
            0
        );
        assert_eq!(
            actual.origin().unwrap().span().end().byte_offset().value(),
            source.len()
        );
        assert!(report.into_validation_result().unwrap().is_some());
    }
    for source in [
        "inherit",
        "initial",
        "revert",
        "none",
        "banana",
        "\"swap\"",
        "1",
        "swap block",
        "swap,block",
        "var(--x)",
    ] {
        rejected(parse_font_feature_display_value(source), source);
    }
    let report = parse_font_feature_display_value("env(x)");
    assert!(report.is_clean());
    assert!(matches!(
        report.syntax().as_ref().unwrap().view(),
        surgeist_css::CssFontFeatureDisplayValueRef::Pending(_)
    ));
}

#[test]
fn seven_feature_kinds_enforce_their_selected_cardinalities() {
    let cases: &[(FeatureKind, &str, &[&str])] = &[
        (FeatureKind::Stylistic, "1", &["1"]),
        (FeatureKind::Stylistic, "0", &["0"]),
        (FeatureKind::HistoricalForms, "1", &["1"]),
        (FeatureKind::HistoricalForms, "1 2 3", &["1", "2", "3"]),
        (FeatureKind::Styleset, "1", &["1"]),
        (FeatureKind::Styleset, "0 100 101", &["0", "100", "101"]),
        (FeatureKind::CharacterVariant, "1", &["1"]),
        (FeatureKind::CharacterVariant, "0 101", &["0", "101"]),
        (FeatureKind::Swash, "2", &["2"]),
        (FeatureKind::Ornaments, "3", &["3"]),
        (FeatureKind::Annotation, "4", &["4"]),
    ];
    for (kind, source, expected) in cases {
        let report = parse_font_feature_value(source, *kind);
        assert!(report.is_clean(), "{kind:?} {source}: {report:?}");
        let actual = report.syntax().as_ref().unwrap();
        assert_eq!(actual.kind(), *kind);
        assert_eq!(
            actual
                .ordinary_indexes()
                .iter()
                .map(|n| n.as_decimal_str())
                .collect::<Vec<_>>(),
            *expected
        );
        complete_components(actual.components(), actual.origin().unwrap(), source);
        assert!(report.into_validation_result().unwrap().is_some());
    }
    for kind in [
        FeatureKind::Stylistic,
        FeatureKind::Swash,
        FeatureKind::Ornaments,
        FeatureKind::Annotation,
    ] {
        rejected(parse_font_feature_value("1 2", kind), "1 2");
    }
    rejected(
        parse_font_feature_value("1 2 3", FeatureKind::CharacterVariant),
        "1 2 3",
    );
}

#[test]
fn index_normalization_preserves_exact_integer_token_spellings_for_every_kind() {
    for kind in FEATURES {
        for (source, digits, small) in [
            ("+0002", "2", Some(2)),
            ("-0000", "0", Some(0)),
            ("18446744073709551616", "18446744073709551616", None),
        ] {
            let report = parse_font_feature_value(source, kind);
            assert!(report.is_clean(), "{kind:?} {source}: {report:?}");
            let actual = report.syntax().as_ref().unwrap();
            let [index] = actual.ordinary_indexes() else {
                panic!("one index")
            };
            assert_eq!(index.as_decimal_str(), digits);
            assert_eq!(index.to_u32(), small);
            let [component] = actual.components().items() else {
                panic!("one number component")
            };
            let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view()
            else {
                panic!("integer token")
            };
            assert_eq!(number.representation(), source);
            assert_eq!(number.kind(), CssNumericTokenKind::Integer);
            assert_eq!(number.has_sign(), source.starts_with(['+', '-']));
            assert_eq!(index.origin().unwrap(), parsed(component));
            assert!(
                actual
                    .origin()
                    .unwrap()
                    .source()
                    .same_snapshot(index.origin().unwrap().source())
            );
        }
        for source in [
            "-1", "1.0", "1e0", "1px", "1%", "\"1\"", "one", "var(--x)", "calc(1)", "1,2",
        ] {
            rejected(parse_font_feature_value(source, kind), source);
        }
        let report = parse_font_feature_value("env(x)", kind);
        assert!(report.is_clean());
        assert!(matches!(
            report.syntax().as_ref().unwrap().view(),
            surgeist_css::CssFontFeatureValueRef::Pending(_)
        ));
    }
}

#[test]
fn surplus_indexes_reject_at_the_first_authored_token_beyond_the_kind_cardinality() {
    for (source, kind, offset, token) in [
        ("1 2", FeatureKind::Stylistic, 2, "2"),
        ("1 2 3", FeatureKind::CharacterVariant, 4, "3"),
    ] {
        let report = parse_font_feature_value(source, kind);
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|d| d.action() == CssRecoveryAction::RejectInput)
            .unwrap();
        position(diagnostic.error().position(), offset, 0, offset as u32);
        let ErrorKind::UnexpectedToken(detail) = diagnostic.error().kind() else {
            panic!("authored surplus index token")
        };
        assert_eq!(detail.encountered().authored(), token);
        rejected(report, source);
    }
}

#[test]
fn original_unicode_trivia_belongs_to_full_carriers_and_precise_semantic_tokens() {
    let source = " /*é*/\r\nsWaP /*💡*/ ";
    let report = parse_font_feature_display_value(source);
    assert!(report.is_clean(), "{report:?}");
    let actual = report.syntax().as_ref().unwrap();
    assert_eq!(actual.ordinary_display(), CssFontDisplay::Swap);
    origin(actual.origin().unwrap(), source, (0, 0, 0), (23, 1, 12));
    complete_components(actual.components(), actual.origin().unwrap(), source);
    let ident = actual
        .components()
        .items()
        .iter()
        .find(|c| {
            matches!(
                c.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident("sWaP"))
            )
        })
        .unwrap();
    origin(parsed(ident), source, (9, 1, 0), (13, 1, 4));
    assert_eq!(
        actual
            .components()
            .items()
            .iter()
            .filter_map(|c| match c.view() {
                CssComponentValueRef::Comment(text) => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>(),
        ["é", "💡"]
    );

    let source = " /*é*/\r\n+0002 /*💡*/ ";
    let report = parse_font_feature_value(source, FeatureKind::Stylistic);
    assert!(report.is_clean(), "{report:?}");
    let actual = report.syntax().as_ref().unwrap();
    origin(actual.origin().unwrap(), source, (0, 0, 0), (24, 1, 13));
    complete_components(actual.components(), actual.origin().unwrap(), source);
    let [index] = actual.ordinary_indexes() else {
        panic!("one index")
    };
    assert_eq!(index.as_decimal_str(), "2");
    origin(index.origin().unwrap(), source, (9, 1, 0), (14, 1, 5));
    let number = actual
        .components()
        .items()
        .iter()
        .find(|c| {
            matches!(
                c.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Number(_))
            )
        })
        .unwrap();
    assert_eq!(parsed(number), index.origin().unwrap());

    let source = "\"💡\"";
    let report = parse_counter_style_descriptor_value(source, CounterKind::Prefix);
    assert!(report.is_clean(), "{report:?}");
    let actual = report.syntax().as_ref().unwrap();
    counter_semantics(actual, &Expected::Prefix(Symbol::String("💡")));
    origin(actual.origin().unwrap(), source, (0, 0, 0), (6, 0, 4));
    complete_components(actual.components(), actual.origin().unwrap(), source);
    let [component] = actual.components().items() else {
        panic!("one string")
    };
    origin(parsed(component), source, (0, 0, 0), (6, 0, 4));

    let source = " /*é*/\r\ncyclic /*💡*/ ";
    let report = parse_counter_style_descriptor_value(source, CounterKind::System);
    assert!(report.is_clean(), "{report:?}");
    let actual = report.syntax().as_ref().unwrap();
    counter_semantics(actual, &Expected::System(CssCounterStyleSystem::Cyclic));
    origin(actual.origin().unwrap(), source, (0, 0, 0), (25, 1, 14));
    complete_components(actual.components(), actual.origin().unwrap(), source);
    let ident = actual
        .components()
        .items()
        .iter()
        .find(|c| {
            matches!(
                c.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident("cyclic"))
            )
        })
        .unwrap();
    origin(parsed(ident), source, (9, 1, 0), (15, 1, 6));
}

#[test]
fn counter_numeric_semantic_origins_keep_order_separate_from_whole_value_origin() {
    let source = " \"0\"/*é*/ +0002 ";
    let report = parse_counter_style_descriptor_value(source, CounterKind::Pad);
    assert!(report.is_clean(), "{report:?}");
    let actual = report.syntax().as_ref().unwrap();
    origin(actual.origin().unwrap(), source, (0, 0, 0), (17, 0, 16));
    complete_components(actual.components(), actual.origin().unwrap(), source);
    let CounterRef::Pad(pad) = actual.view() else {
        panic!("pad")
    };
    assert_eq!(pad.minimum_length().numeric().representation(), "+0002");
    let CssValueOrigin::Parsed(number_origin) = pad.minimum_length().origin() else {
        panic!("number origin")
    };
    origin(number_origin, source, (11, 0, 10), (16, 0, 15));
    assert!(
        actual
            .origin()
            .unwrap()
            .source()
            .same_snapshot(number_origin.source())
    );
    assert!(matches!(
        actual.components().items()[1].view(),
        CssComponentValueRef::Token(CssValueTokenRef::String("0"))
    ));
}

#[test]
fn punctuation_inside_an_admitted_url_is_data_with_original_group_origins() {
    let source = "url(\"x;y!z\")";
    let report = parse_counter_style_descriptor_value(source, CounterKind::Prefix);
    assert!(report.is_clean(), "{report:?}");
    let actual = report.syntax().as_ref().unwrap();
    counter_semantics(actual, &Expected::Prefix(Symbol::Url("x;y!z")));
    origin(actual.origin().unwrap(), source, (0, 0, 0), (12, 0, 12));
    let [component] = actual.components().items() else {
        panic!("one original function")
    };
    origin(parsed(component), source, (0, 0, 0), (4, 0, 4));
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("URL function")
    };
    assert_eq!(function.name(), "url");
    let [quoted] = function.values().items() else {
        panic!("one quoted URL payload")
    };
    assert!(matches!(
        quoted.view(),
        CssComponentValueRef::Token(CssValueTokenRef::String("x;y!z"))
    ));
    origin(parsed(quoted), source, (4, 0, 4), (11, 0, 11));
    let CssValueOrigin::Parsed(closing) = function.closing_origin() else {
        panic!("actual closing delimiter")
    };
    origin(closing, source, (11, 0, 11), (12, 0, 12));
    assert!(
        actual
            .origin()
            .unwrap()
            .source()
            .same_snapshot(parsed(quoted).source())
    );
    assert!(
        actual
            .origin()
            .unwrap()
            .source()
            .same_snapshot(closing.source())
    );
}

#[test]
fn every_required_value_rejects_empty_source_and_trivia_without_inventing_empty_syntax() {
    for source in ["", " \t", "/**/"] {
        for (kind, _, _) in COUNTERS {
            empty(parse_counter_style_descriptor_value(source, kind), source);
        }
        empty(parse_font_feature_display_value(source), source);
        for kind in FEATURES {
            let report = parse_font_feature_value(source, kind);
            let diagnostic = report
                .diagnostics()
                .iter()
                .find(|d| d.action() == CssRecoveryAction::RejectInput)
                .unwrap();
            assert!(matches!(
                diagnostic.error().kind(),
                ErrorKind::UnexpectedEnd(_)
            ));
            empty(report, source);
        }
    }
    // A quoted empty String is an actual suffix value; it is not empty input.
    let report = parse_counter_style_descriptor_value("\"\"", CounterKind::Suffix);
    assert!(report.is_clean());
    counter_semantics(
        report.syntax().as_ref().unwrap(),
        &Expected::Suffix(Symbol::String("")),
    );
}

fn empty<T: std::fmt::Debug>(report: CssParseReport<Option<T>>, source: &str) {
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap();
    // No single error category is imposed across these independently owned grammars.
    position(
        diagnostic.error().position(),
        source.len(),
        0,
        source.len() as u32,
    );
    rejected(report, source);
}

fn boundary<T: std::fmt::Debug>(
    report: CssParseReport<Option<T>>,
    source: &str,
    offset: usize,
    annotation: bool,
) {
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap();
    position(diagnostic.error().position(), offset, 0, offset as u32);
    if annotation {
        assert!(matches!(
            diagnostic.error().kind(),
            ErrorKind::InvalidDeclarationAnnotation(_)
        ));
    }
    rejected(report, source);
}

#[test]
fn root_delimiters_and_importance_reject_the_complete_value_at_the_actual_boundary() {
    for suffix in [";", "!important", ")", "]", "}", "{}"] {
        let source = format!("cyclic{suffix}");
        boundary(
            parse_counter_style_descriptor_value(&source, CounterKind::System),
            &source,
            6,
            suffix == "!important",
        );
        let source = format!("swap{suffix}");
        boundary(
            parse_font_feature_display_value(&source),
            &source,
            4,
            suffix == "!important",
        );
        let source = format!("1{suffix}");
        let report = parse_font_feature_value(&source, FeatureKind::Stylistic);
        if suffix == "!important" {
            let diagnostic = report
                .diagnostics()
                .iter()
                .find(|d| d.action() == CssRecoveryAction::RejectInput)
                .unwrap();
            position(diagnostic.error().position(), 1, 0, 1);
            let ErrorKind::UnexpectedToken(detail) = diagnostic.error().kind() else {
                panic!("name-free index annotation token")
            };
            assert_eq!(detail.encountered().authored(), "!");
            rejected(report, &source);
        } else {
            boundary(report, &source, 1, false);
        }
    }
    for source in ["swap; font-display:block", "swap !important extra"] {
        rejected(parse_font_feature_display_value(source), source);
    }
    rejected(
        parse_counter_style_descriptor_value("x; suffix:y", CounterKind::Prefix),
        "x; suffix:y",
    );
    rejected(
        parse_font_feature_value("1; next:2", FeatureKind::Styleset),
        "1; next:2",
    );
}

#[test]
fn invalid_tokens_have_literal_unicode_positions_and_family_diagnostics() {
    let source = " /*é*/\r\nswap extra";
    let report = parse_font_feature_display_value(source);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap();
    position(diagnostic.error().position(), 14, 1, 5);
    let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
        panic!("display grammar diagnostic")
    };
    assert_eq!(detail.at_rule().as_str(), "font-feature-values");
    assert_eq!(detail.descriptor().as_str(), "font-display");
    assert_eq!(detail.encountered().unwrap().authored(), "extra");
    rejected(report, source);

    let source = " /*é*/\r\n1.0";
    let report = parse_font_feature_value(source, FeatureKind::Stylistic);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap();
    position(diagnostic.error().position(), 9, 1, 0);
    let ErrorKind::UnexpectedToken(detail) = diagnostic.error().kind() else {
        panic!("name-free index grammar diagnostic")
    };
    assert_eq!(detail.encountered().authored(), "1.0");
    rejected(report, source);

    let source = " /*é*/\r\nx y";
    let report = parse_counter_style_descriptor_value(source, CounterKind::Prefix);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap();
    position(diagnostic.error().position(), 11, 1, 2);
    let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
        panic!("counter grammar diagnostic")
    };
    assert_eq!(detail.at_rule().as_str(), "counter-style");
    assert_eq!(detail.descriptor().as_str(), "prefix");
    assert_eq!(detail.encountered().unwrap().authored(), "y");
    rejected(report, source);
}

#[test]
fn lexical_eof_recovery_retains_values_and_fails_clean_only_validation() {
    let source = "\"a";
    let report = parse_counter_style_descriptor_value(source, CounterKind::Prefix);
    counter_semantics(
        report.syntax().as_ref().unwrap(),
        &Expected::Prefix(Symbol::String("a")),
    );
    let closure = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .unwrap();
    assert_eq!(closure.error().code(), CssErrorCode::UnexpectedEnd);
    position(closure.error().position(), 2, 0, 2);
    position(closure.span().start(), 2, 0, 2);
    position(closure.span().end(), 2, 0, 2);
    origin(
        report.syntax().as_ref().unwrap().origin().unwrap(),
        source,
        (0, 0, 0),
        (2, 0, 2),
    );
    assert!(report.into_validation_result().is_err());

    let source = "swap/*x";
    let report = parse_font_feature_display_value(source);
    let actual = report.syntax().as_ref().unwrap();
    assert_eq!(actual.ordinary_display(), CssFontDisplay::Swap);
    origin(actual.origin().unwrap(), source, (0, 0, 0), (7, 0, 7));
    assert!(matches!(
        actual.components().items().last().unwrap().view(),
        CssComponentValueRef::Comment("x")
    ));
    let comment = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::IgnoreUnterminatedComment)
        .unwrap();
    assert_eq!(comment.error().code(), CssErrorCode::UnexpectedEnd);
    position(comment.error().position(), 7, 0, 7);
    position(comment.span().start(), 4, 0, 4);
    position(comment.span().end(), 7, 0, 7);
    assert!(report.into_validation_result().is_err());

    let source = "1/*x";
    let report = parse_font_feature_value(source, FeatureKind::Stylistic);
    let actual = report.syntax().as_ref().unwrap();
    assert_eq!(actual.ordinary_indexes()[0].as_decimal_str(), "1");
    origin(actual.origin().unwrap(), source, (0, 0, 0), (4, 0, 4));
    origin(
        actual.ordinary_indexes()[0].origin().unwrap(),
        source,
        (0, 0, 0),
        (1, 0, 1),
    );
    assert!(matches!(
        actual.components().items().last().unwrap().view(),
        CssComponentValueRef::Comment("x")
    ));
    let comment = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::IgnoreUnterminatedComment)
        .unwrap();
    assert_eq!(comment.error().code(), CssErrorCode::UnexpectedEnd);
    position(comment.error().position(), 4, 0, 4);
    position(comment.span().start(), 1, 0, 1);
    position(comment.span().end(), 4, 0, 4);
    assert!(report.into_validation_result().is_err());

    rejected(
        parse_counter_style_descriptor_value("\"a\nb", CounterKind::Prefix),
        "\"a\nb",
    );
    rejected(
        parse_counter_style_descriptor_value("url(a b)", CounterKind::Prefix),
        "url(a b)",
    );
}

fn eof_escape<T: std::fmt::Debug>(report: CssParseReport<Option<T>>, offset: usize) {
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| {
            matches!(
                d.error().kind(),
                ErrorKind::EscapeParseError(CssEscapeError::EndOfInput)
            )
        })
        .unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::RecoverEscape);
    position(diagnostic.error().position(), offset, 0, offset as u32);
    assert!(report.into_validation_result().is_err());
}

#[test]
fn lexical_escape_eof_is_reported_independently_of_grammar_retention() {
    let source = "base\\";
    let report = parse_counter_style_descriptor_value(source, CounterKind::Prefix);
    counter_semantics(
        report.syntax().as_ref().unwrap(),
        &Expected::Prefix(Symbol::Ident("base�")),
    );
    eof_escape(report, 5);
    let report = parse_font_feature_display_value("swap\\");
    assert!(report.syntax().is_none());
    eof_escape(report, 5);
    let report = parse_font_feature_value("1\\", FeatureKind::Stylistic);
    assert!(report.syntax().is_none());
    eof_escape(report, 2);
}

fn depth<T: std::fmt::Debug>(report: CssParseReport<Option<T>>, exceeded: bool) {
    assert!(
        report.syntax().is_none(),
        "f() is outside these value grammars"
    );
    if exceeded {
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
            .unwrap();
        let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
            panic!("typed resource stop")
        };
        assert_eq!(detail.limit(), 256);
        position(diagnostic.error().position(), 512, 0, 512);
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
    } else {
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.error().code() == CssErrorCode::NestingLimit)
        );
    }
    assert!(report.into_validation_result().is_err());
}

#[test]
fn the_257th_actual_component_opener_preserves_resource_precedence() {
    for (count, exceeded) in [(256, false), (257, true)] {
        let source = format!("{}x{}", "f(".repeat(count), ")".repeat(count));
        depth(
            parse_counter_style_descriptor_value(&source, CounterKind::Prefix),
            exceeded,
        );
        depth(parse_font_feature_display_value(&source), exceeded);
        depth(
            parse_font_feature_value(&source, FeatureKind::Stylistic),
            exceeded,
        );
    }
}

#[test]
fn real_counter_rule_owns_names_combination_checks_and_sibling_recovery() {
    let source = "@counter-style demo{system:cyclic;symbols:x;prefix:1;suffix:\"!\"}";
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("one retained counter rule")
    };
    assert_eq!(rule.name().as_str(), "demo");
    assert_eq!(
        rule.descriptors().system().unwrap().ordinary_system(),
        &CssCounterStyleSystem::Cyclic
    );
    symbol(
        &rule
            .descriptors()
            .symbols()
            .unwrap()
            .ordinary_symbols()
            .symbols()[0],
        Symbol::Ident("x"),
    );
    assert!(rule.descriptors().prefix().is_none());
    symbol(
        rule.descriptors().suffix().unwrap().ordinary_suffix(),
        Symbol::String("!"),
    );
    position(rule.descriptors().system().unwrap().position(), 20, 0, 20);
    position(rule.descriptors().suffix().unwrap().position(), 53, 0, 53);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
    );
    let direct = parse_counter_style_descriptor_value("cyclic", CounterKind::System);
    let CounterRef::System(system) = direct.syntax().as_ref().unwrap().view() else {
        panic!("system")
    };
    assert_eq!(
        system,
        rule.descriptors().system().unwrap().ordinary_system()
    );
    rejected(
        parse_counter_style_descriptor_value("1", CounterKind::Prefix),
        "1",
    );

    // The direct value owns no system/symbol combination policy. An actual rule does.
    let direct = parse_counter_style_descriptor_value("extends decimal", CounterKind::System);
    assert!(direct.is_clean());
    counter_semantics(
        direct.syntax().as_ref().unwrap(),
        &Expected::Extends("decimal"),
    );
    let report = parse_sheet("@counter-style demo{system:extends decimal;symbols:x}");
    assert!(report.syntax().rules().is_empty());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| matches!(d.error().kind(), ErrorKind::InvalidDescriptorCombination(_)))
    );
}

#[test]
fn real_mixed_font_rule_keeps_actual_names_positions_and_later_valid_definitions() {
    let source = "@font-feature-values Demo{font-display:swap;@stylistic{bad:1.0;Good:+0002;}font-display:optional;@styleset{Wide:0 100 101;}}";
    let report = parse_sheet(source);
    let [CssRule::FontFeatureValues(rule)] = report.syntax().rules() else {
        panic!("one real font rule")
    };
    assert_eq!(rule.families()[0].as_str(), "Demo");
    let [
        CssFontFeatureValuesItem::FontDisplay(first),
        CssFontFeatureValuesItem::Block(stylistic),
        CssFontFeatureValuesItem::FontDisplay(last),
        CssFontFeatureValuesItem::Block(styleset),
    ] = rule.items()
    else {
        panic!("interleaved real body")
    };
    assert_eq!(first.value().ordinary_display(), CssFontDisplay::Swap);
    assert_eq!(last.value().ordinary_display(), CssFontDisplay::Optional);
    position(first.position().unwrap(), 26, 0, 26);
    position(last.position().unwrap(), 75, 0, 75);
    assert_eq!(stylistic.kind(), FeatureKind::Stylistic);
    let [definition] = stylistic.definitions() else {
        panic!("bad definition dropped")
    };
    assert_eq!(definition.name().as_str(), "Good");
    position(definition.position().unwrap(), 63, 0, 63);
    assert_eq!(definition.ordinary_indexes()[0].as_decimal_str(), "2");
    origin(
        definition.ordinary_indexes()[0].origin().unwrap(),
        source,
        (68, 0, 68),
        (73, 0, 73),
    );
    assert_eq!(styleset.kind(), FeatureKind::Styleset);
    assert_eq!(styleset.definitions()[0].name().as_str(), "Wide");
    assert_eq!(
        styleset.definitions()[0]
            .ordinary_indexes()
            .iter()
            .map(|n| n.as_decimal_str())
            .collect::<Vec<_>>(),
        ["0", "100", "101"]
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
    );
    let direct = parse_font_feature_value("+0002", FeatureKind::Stylistic);
    assert!(direct.is_clean());
    assert_eq!(
        direct.syntax().as_ref().unwrap().ordinary_indexes()[0].as_decimal_str(),
        definition.ordinary_indexes()[0].as_decimal_str()
    );
    let direct = parse_font_feature_display_value("swap");
    assert!(direct.is_clean());
    assert_eq!(
        direct.syntax().as_ref().unwrap().ordinary_display(),
        first.value().ordinary_display()
    );
    rejected(
        parse_font_feature_value("1.0", FeatureKind::Stylistic),
        "1.0",
    );
}

//! Characterization of the checked component-native construction and pending
//! reentry of the `@counter-style`, `@font-feature-values`, `@color-profile` and
//! `@font-palette-values` descriptor values (`CssCounterStyleDescriptorValue`,
//! `CssFontFeatureDisplayValue`, `CssFontFeatureValue`,
//! `CssColorProfileDescriptorValue`, `CssFontPaletteDescriptorValue`), through
//! their public constructors only.
//!
//! Authorities: Syntax 3 token positions (a diagnostic names the first token that
//! violates the grammar, with a zero-based line and UTF-16 column beside the UTF-8
//! byte offset); Counter Styles 3 section 3 and Fonts 4 sections 4.9.1 and 6.9.2
//! for the grammars; the documented contract of `CssComponentValueLimits` (every
//! component, trivia included, counts once, a function opening adds one node and
//! its closer none, and the byte limit applies to the serialized output including
//! implicit closers) and of the constructors ("checks cumulative component and
//! byte limits before grammar or env qualification"; reentry "rejects recovered
//! closures and residual decoded var/env functions before ordinary grammar").
//! Existing tests pin the ordinary views, pending retention, origins, failure
//! predicates and the zero limits (`descriptor_env_phase_contract`); this module
//! adds the diagnostics, error messages and source chain, exact limit boundaries,
//! failure order, and serialization of reentered values.

use crate::css::support::authored_syntax::{assert_origin_region, assert_position};
use crate::css::support::failure_classes::{ComponentFailure, component_failure};
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValueLimits, CssComponentValues, CssSerializedOrigin, CssValueOrigin,
};
use surgeist_css::diagnostic::{CssError, CssErrorCode, CssTokenKind};
use surgeist_css::rule::color_profile::CssColorProfileDescriptorKind as ProfileKind;
use surgeist_css::rule::color_profile::CssColorProfileDescriptorValue as Profile;
use surgeist_css::rule::color_profile::CssColorProfileValueError as ProfileError;
use surgeist_css::rule::counter_style::CssCounterStyleDescriptorKind as CounterKind;
use surgeist_css::rule::counter_style::CssCounterStyleDescriptorValue as Counter;
use surgeist_css::rule::counter_style::CssCounterStyleValueError as CounterError;
use surgeist_css::rule::font_feature_values::CssFontFeatureDisplayValue as Display;
use surgeist_css::rule::font_feature_values::CssFontFeatureValue as Feature;
use surgeist_css::rule::font_feature_values::CssFontFeatureValueKind as FeatureKind;
use surgeist_css::rule::font_feature_values::CssFontFeatureValuesError as FontError;
use surgeist_css::rule::font_palette_values::CssFontPaletteDescriptorKind as PaletteKind;
use surgeist_css::rule::font_palette_values::CssFontPaletteDescriptorValue as Palette;
use surgeist_css::rule::font_palette_values::CssFontPaletteValuesError as PaletteError;

fn components(source: &str) -> CssComponentValues {
    CssComponentValues::parse(&CssParser::default(), source).unwrap()
}
fn limits(depth: u32, components: usize, bytes: usize) -> CssComponentValueLimits {
    CssComponentValueLimits::try_new(depth, components, bytes).unwrap()
}

/// What a grammar rejection must report: the code, the byte/line/UTF-16 column of
/// the responsible token, the token's kind and authored text (none at the end of
/// the input) and its span in the supplied source.
struct Rejection {
    code: CssErrorCode,
    position: Coordinates,
    token: Option<(CssTokenKind, &'static str)>,
    span: Option<(Coordinates, Coordinates)>,
}

/// A byte offset, zero-based line and zero-based UTF-16 column.
type Coordinates = (usize, u32, u32);

fn assert_rejection(
    diagnostic: Option<&CssError>,
    origin: Option<&CssSerializedOrigin>,
    source: &str,
    expected: &Rejection,
) {
    let diagnostic = diagnostic.expect("a grammar diagnostic");
    assert_eq!(diagnostic.code(), expected.code, "{source:?}");
    assert_position(
        diagnostic.position(),
        expected.position.0,
        expected.position.1,
        expected.position.2,
    );
    assert_eq!(
        diagnostic
            .encountered()
            .map(|token| (token.kind(), token.authored())),
        expected.token,
        "{source:?}"
    );
    match (expected.span, origin.expect("an origin")) {
        (Some((start, end)), CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin))) => {
            assert_origin_region(origin, source, start, end);
        }
        (None, CssSerializedOrigin::End(None)) => {}
        (_, origin) => panic!("unexpected origin {origin:?} for {source:?}"),
    }
}

fn token(kind: CssTokenKind, authored: &'static str) -> Option<(CssTokenKind, &'static str)> {
    Some((kind, authored))
}

#[test]
fn counter_style_grammar_rejections_name_the_first_violating_token_and_its_coordinates() {
    let value = CssErrorCode::InvalidDescriptorValue;
    let cases = [
        // A number is not a `<string>`/`<ident>`/`<image>` symbol.
        (
            "1",
            Rejection {
                code: value,
                position: (0, 0, 0),
                token: token(CssTokenKind::Number, "1"),
                span: Some(((0, 0, 0), (1, 0, 1))),
            },
        ),
        // Nothing was supplied: no token is responsible.
        (
            "",
            Rejection {
                code: value,
                position: (0, 0, 0),
                token: None,
                span: None,
            },
        ),
        // The second symbol is the first token past a one-symbol value, on line 1.
        (
            "\"x\"\n  1",
            Rejection {
                code: value,
                position: (6, 1, 2),
                token: token(CssTokenKind::Number, "1"),
                span: Some(((6, 1, 2), (7, 1, 3))),
            },
        ),
        (
            "\"x\" \"y\"",
            Rejection {
                code: value,
                position: (4, 0, 4),
                token: token(CssTokenKind::String, "\"y\""),
                span: Some(((4, 0, 4), (7, 0, 7))),
            },
        ),
        // The two-byte U+00E9 makes the byte offset exceed the UTF-16 column.
        (
            "\"\u{e9}\" 1",
            Rejection {
                code: value,
                position: (5, 0, 4),
                token: token(CssTokenKind::Number, "1"),
                span: Some(((5, 0, 4), (6, 0, 5))),
            },
        ),
        // A descriptor value never carries an importance annotation.
        (
            "env(x)!important",
            Rejection {
                code: CssErrorCode::InvalidDeclarationAnnotation,
                position: (6, 0, 6),
                token: token(CssTokenKind::Delim, "!"),
                span: Some(((6, 0, 6), (7, 0, 7))),
            },
        ),
    ];
    for (source, expected) in cases {
        let constructed = Counter::try_from_components(
            CounterKind::Prefix,
            components(source),
            Default::default(),
        )
        .unwrap_err();
        assert_rejection(
            constructed.diagnostic(),
            Some(constructed.origin()),
            source,
            &expected,
        );
        assert!(constructed.component_error().is_none());
        // A replacement that still holds `env()` is refused as residual before the grammar.
        if source.starts_with("env(") {
            continue;
        }
        // Reentry of a pending value runs the same grammar, so it reports the same rejection.
        let reentered = Counter::try_from_components(
            CounterKind::Prefix,
            components("env(x)"),
            Default::default(),
        )
        .unwrap()
        .reparse_after_substitution(components(source), Default::default())
        .unwrap_err();
        assert_rejection(
            reentered.diagnostic(),
            Some(reentered.origin()),
            source,
            &expected,
        );
        assert_eq!(constructed, reentered, "{source:?}");
    }
}

#[test]
fn font_display_grammar_rejections_name_the_first_violating_token_and_its_coordinates() {
    let cases = [
        (
            "",
            Rejection {
                code: CssErrorCode::UnexpectedEnd,
                position: (0, 0, 0),
                token: None,
                span: None,
            },
        ),
        // `block` is the first token past the single keyword.
        (
            "swap block",
            Rejection {
                code: CssErrorCode::UnexpectedToken,
                position: (5, 0, 5),
                token: token(CssTokenKind::Ident, "block"),
                span: Some(((5, 0, 5), (10, 0, 10))),
            },
        ),
        (
            "swap\n x",
            Rejection {
                code: CssErrorCode::UnexpectedToken,
                position: (6, 1, 1),
                token: token(CssTokenKind::Ident, "x"),
                span: Some(((6, 1, 1), (7, 1, 2))),
            },
        ),
    ];
    for (source, expected) in cases {
        let constructed =
            Display::try_from_components(components(source), Default::default()).unwrap_err();
        assert_rejection(
            constructed.diagnostic(),
            constructed.origin(),
            source,
            &expected,
        );
        assert!(constructed.component_error().is_none());
        let reentered = Display::try_from_components(components("env(x)"), Default::default())
            .unwrap()
            .reparse_after_substitution(components(source), Default::default())
            .unwrap_err();
        assert_rejection(
            reentered.diagnostic(),
            reentered.origin(),
            source,
            &expected,
        );
        assert_eq!(constructed, reentered, "{source:?}");
    }
}

#[test]
fn invalid_font_display_values_report_the_responsible_token_not_the_end_of_input() {
    // Fonts 4 section 9.1.1: `auto | block | swap | fallback | optional`. A word that
    // is not a keyword, a string, a number and a second token after a keyword are each
    // an unexpected token: the token the input holds, not an early end.
    let unexpected = CssErrorCode::UnexpectedToken;
    let cases = [
        (
            "banana",
            Rejection {
                code: unexpected,
                position: (0, 0, 0),
                token: token(CssTokenKind::Ident, "banana"),
                span: Some(((0, 0, 0), (6, 0, 6))),
            },
        ),
        (
            "\u{e9}t\u{e9}",
            Rejection {
                code: unexpected,
                position: (0, 0, 0),
                token: token(CssTokenKind::Ident, "\u{e9}t\u{e9}"),
                span: Some(((0, 0, 0), (5, 0, 3))),
            },
        ),
        (
            " \n BANANA",
            Rejection {
                code: unexpected,
                position: (3, 1, 1),
                token: token(CssTokenKind::Ident, "BANANA"),
                span: Some(((3, 1, 1), (9, 1, 7))),
            },
        ),
        (
            "\"swap\"",
            Rejection {
                code: unexpected,
                position: (0, 0, 0),
                token: token(CssTokenKind::String, "\"swap\""),
                span: Some(((0, 0, 0), (6, 0, 6))),
            },
        ),
        (
            "1",
            Rejection {
                code: unexpected,
                position: (0, 0, 0),
                token: token(CssTokenKind::Number, "1"),
                span: Some(((0, 0, 0), (1, 0, 1))),
            },
        ),
        (
            "swap 1",
            Rejection {
                code: unexpected,
                position: (5, 0, 5),
                token: token(CssTokenKind::Number, "1"),
                span: Some(((5, 0, 5), (6, 0, 6))),
            },
        ),
        (
            "swap banana",
            Rejection {
                code: unexpected,
                position: (5, 0, 5),
                token: token(CssTokenKind::Ident, "banana"),
                span: Some(((5, 0, 5), (11, 0, 11))),
            },
        ),
        // An empty value is the one input the end of which is the wrong thing.
        (
            "",
            Rejection {
                code: CssErrorCode::UnexpectedEnd,
                position: (0, 0, 0),
                token: None,
                span: None,
            },
        ),
    ];
    for (source, expected) in cases {
        let constructed =
            Display::try_from_components(components(source), Default::default()).unwrap_err();
        assert_rejection(
            constructed.diagnostic(),
            constructed.origin(),
            source,
            &expected,
        );
        let reentered = Display::try_from_components(components("env(x)"), Default::default())
            .unwrap()
            .reparse_after_substitution(components(source), Default::default())
            .unwrap_err();
        assert_rejection(
            reentered.diagnostic(),
            reentered.origin(),
            source,
            &expected,
        );
    }
    // The wrong keyword reports like the wrong token of a feature value, whose
    // checked constructor names no descriptor either.
    let display = Display::try_from_components(components("banana"), Default::default())
        .unwrap_err()
        .diagnostic()
        .map(|diagnostic| (diagnostic.code(), diagnostic.expectation()));
    let number = Display::try_from_components(components("1"), Default::default())
        .unwrap_err()
        .diagnostic()
        .map(|diagnostic| (diagnostic.code(), diagnostic.expectation()));
    assert_eq!(display, number);
}

#[test]
fn parsed_font_display_values_name_the_descriptor_and_the_same_responsible_token() {
    // The parsed front and the rule body name `font-display` (an invalid
    // descriptor value), and the token they hold is the one the checked
    // constructor reports.
    for (source, token_text, offset) in [
        ("banana", Some("banana"), 0),
        ("\"swap\"", Some("\"swap\""), 0),
        ("1", Some("1"), 0),
        ("swap block", Some("block"), 5),
        ("swap 1", Some("1"), 5),
        ("", None, 0),
    ] {
        let report = Display::parse(&CssParser::default(), source);
        assert!(report.syntax().is_none(), "{source:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejection for {source:?}")
        };
        let error = diagnostic.error();
        assert_eq!(error.code(), CssErrorCode::InvalidDescriptorValue);
        assert_eq!(
            error.encountered().map(|token| token.authored()),
            token_text,
            "{source:?}"
        );
        assert_position(error.position(), offset, 0, offset as u32);

        let sheet = format!("@font-feature-values Demo{{font-display:{source}}}");
        let report = CssParser::default().parse_sheet(&sheet);
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejection for {sheet:?}")
        };
        let error = diagnostic.error();
        assert_eq!(error.code(), CssErrorCode::InvalidDescriptorValue);
        assert_eq!(
            error.encountered().map(|token| token.authored()),
            token_text,
            "{sheet:?}"
        );

        let constructed =
            Display::try_from_components(components(source), Default::default()).unwrap_err();
        let constructed_token = constructed
            .diagnostic()
            .unwrap()
            .encountered()
            .map(|token| token.authored().to_owned());
        assert_eq!(constructed_token.as_deref(), token_text, "{source:?}");
    }
}

#[test]
fn font_feature_value_grammar_rejections_name_the_first_violating_token_and_its_coordinates() {
    let unexpected = CssErrorCode::UnexpectedToken;
    let cases = [
        (
            FeatureKind::Styleset,
            "",
            Rejection {
                code: CssErrorCode::UnexpectedEnd,
                position: (0, 0, 0),
                token: None,
                span: None,
            },
        ),
        // Index tokens are nonnegative integers: a decimal or a negative is refused whole.
        (
            FeatureKind::Styleset,
            "1.0",
            Rejection {
                code: unexpected,
                position: (0, 0, 0),
                token: token(CssTokenKind::Number, "1.0"),
                span: Some(((0, 0, 0), (3, 0, 3))),
            },
        ),
        (
            FeatureKind::Styleset,
            "-1",
            Rejection {
                code: unexpected,
                position: (0, 0, 0),
                token: token(CssTokenKind::Number, "-1"),
                span: Some(((0, 0, 0), (2, 0, 2))),
            },
        ),
        // The first surplus integer is responsible for an overfull list.
        (
            FeatureKind::Swash,
            "1 2",
            Rejection {
                code: unexpected,
                position: (2, 0, 2),
                token: token(CssTokenKind::Number, "2"),
                span: Some(((2, 0, 2), (3, 0, 3))),
            },
        ),
        (
            FeatureKind::CharacterVariant,
            "1 2 3",
            Rejection {
                code: unexpected,
                position: (4, 0, 4),
                token: token(CssTokenKind::Number, "3"),
                span: Some(((4, 0, 4), (5, 0, 5))),
            },
        ),
        (
            FeatureKind::Swash,
            "1\n 2",
            Rejection {
                code: unexpected,
                position: (3, 1, 1),
                token: token(CssTokenKind::Number, "2"),
                span: Some(((3, 1, 1), (4, 1, 2))),
            },
        ),
    ];
    for (kind, source, expected) in cases {
        let constructed =
            Feature::try_from_components(kind, components(source), Default::default()).unwrap_err();
        assert_rejection(
            constructed.diagnostic(),
            constructed.origin(),
            source,
            &expected,
        );
        assert!(constructed.component_error().is_none());
        let reentered =
            Feature::try_from_components(kind, components("env(x)"), Default::default())
                .unwrap()
                .reparse_after_substitution(components(source), Default::default())
                .unwrap_err();
        assert_rejection(
            reentered.diagnostic(),
            reentered.origin(),
            source,
            &expected,
        );
        assert_eq!(constructed, reentered, "{kind:?} {source:?}");
    }
}

/// One failure of each class a checked descriptor value reports, with the class's
/// source error, when it has one, rendered.
struct Failures<E> {
    component: E,
    grammar: E,
    recovered: E,
    residual: E,
    not_pending: E,
}

fn counter_failures() -> Failures<CounterError> {
    let pending = || {
        Counter::try_from_components(
            CounterKind::Prefix,
            components("env(x)"),
            Default::default(),
        )
        .unwrap()
    };
    Failures {
        component: Counter::try_from_components(
            CounterKind::Prefix,
            components("env(x)"),
            limits(256, 1, usize::MAX),
        )
        .unwrap_err(),
        grammar: Counter::try_from_components(
            CounterKind::Prefix,
            components("SECRET 1"),
            Default::default(),
        )
        .unwrap_err(),
        recovered: Counter::try_from_components(
            CounterKind::Prefix,
            components("env(SECRET"),
            Default::default(),
        )
        .unwrap_err(),
        residual: pending()
            .reparse_after_substitution(components("f(var(--SECRET))"), Default::default())
            .unwrap_err(),
        not_pending: Counter::try_from_components(
            CounterKind::Prefix,
            components("\"SECRET\""),
            Default::default(),
        )
        .unwrap()
        .reparse_after_substitution(components("env(x)"), Default::default())
        .unwrap_err(),
    }
}

fn font_failures() -> Failures<FontError> {
    let pending = || {
        Feature::try_from_components(
            FeatureKind::Styleset,
            components("env(x)"),
            Default::default(),
        )
        .unwrap()
    };
    Failures {
        component: Feature::try_from_components(
            FeatureKind::Styleset,
            components("env(x)"),
            limits(256, 1, usize::MAX),
        )
        .unwrap_err(),
        grammar: Feature::try_from_components(
            FeatureKind::Styleset,
            components("SECRET"),
            Default::default(),
        )
        .unwrap_err(),
        recovered: Feature::try_from_components(
            FeatureKind::Styleset,
            components("env(SECRET"),
            Default::default(),
        )
        .unwrap_err(),
        residual: pending()
            .reparse_after_substitution(components("f(var(--SECRET))"), Default::default())
            .unwrap_err(),
        not_pending: Feature::try_from_components(
            FeatureKind::Styleset,
            components("1"),
            Default::default(),
        )
        .unwrap()
        .reparse_after_substitution(components("env(x)"), Default::default())
        .unwrap_err(),
    }
}

/// The five classes are exclusive, each message names the owning rule and no
/// authored text, the messages differ by class, and only a component or grammar
/// failure has a source: the component error or the diagnostic itself.
fn assert_failure_surface<E: std::error::Error + Clone + PartialEq + 'static>(
    rule: &str,
    failures: &Failures<E>,
    predicates: impl Fn(&E) -> [bool; 5],
    component_source: impl Fn(&E) -> Option<String>,
    grammar_source: impl Fn(&E) -> Option<String>,
) {
    let all = [
        &failures.component,
        &failures.grammar,
        &failures.recovered,
        &failures.residual,
        &failures.not_pending,
    ];
    let mut messages = Vec::new();
    for (index, error) in all.into_iter().enumerate() {
        // [component, grammar, recovered, residual, not pending]
        let mut expected = [false; 5];
        expected[index] = true;
        assert_eq!(predicates(error), expected, "class {index}: {error:?}");
        let message = error.to_string();
        assert!(
            message.starts_with(&format!("invalid {rule} descriptor value: ")),
            "{message}"
        );
        assert!(!message.contains("SECRET"), "{message}");
        assert_eq!(error, &error.clone());
        messages.push(message);
    }
    messages.sort();
    messages.dedup();
    assert_eq!(messages.len(), 5, "one distinct message per class");
    assert_eq!(
        failures.component.source().map(ToString::to_string),
        component_source(&failures.component)
    );
    assert!(failures.component.source().is_some());
    assert_eq!(
        failures.grammar.source().map(ToString::to_string),
        grammar_source(&failures.grammar)
    );
    assert!(failures.grammar.source().is_some());
    for error in [
        &failures.recovered,
        &failures.residual,
        &failures.not_pending,
    ] {
        assert!(error.source().is_none(), "{error:?}");
    }
}

#[test]
fn checked_failures_are_exclusive_classes_with_rule_named_messages_and_sources() {
    assert_failure_surface(
        "counter-style",
        &counter_failures(),
        |error| {
            [
                error.component_error().is_some(),
                error.diagnostic().is_some(),
                error.is_recovered_component(),
                error.is_residual_substitution(),
                error.is_not_pending(),
            ]
        },
        |error| error.component_error().map(ToString::to_string),
        |error| error.diagnostic().map(ToString::to_string),
    );
    assert_failure_surface(
        "font-feature-values",
        &font_failures(),
        |error| {
            [
                error.component_error().is_some(),
                error.diagnostic().is_some(),
                error.is_recovered_component(),
                error.is_residual_substitution(),
                error.is_not_pending(),
            ]
        },
        |error| error.component_error().map(ToString::to_string),
        |error| error.diagnostic().map(ToString::to_string),
    );
    // A descriptor-value failure of the rule error is none of its construction kinds.
    let error = font_failures().grammar;
    assert!(
        !error.is_invalid_identifier()
            && !error.is_empty_families()
            && !error.is_invalid_integer_syntax()
            && !error.is_negative_index()
            && !error.is_invalid_index_count()
            && !error.is_kind_mismatch()
    );
    assert!(error.block().is_none() && error.definition_index().is_none());
}

/// Checks one family against the documented counting: `source` is admitted by
/// exactly `components` nodes and `bytes` serialized bytes, so one fewer of
/// either is refused with its own class.
fn assert_exact_budget<V, E>(
    source: &str,
    components_needed: usize,
    bytes_needed: usize,
    construct: impl Fn(CssComponentValues, CssComponentValueLimits) -> Result<V, E>,
    component_error: impl Fn(&E) -> Option<ComponentFailure>,
) {
    let supplied = components(source);
    assert!(
        construct(
            supplied.clone(),
            limits(256, components_needed, bytes_needed)
        )
        .is_ok(),
        "{source:?} fits exactly"
    );
    for (limit, expected) in [
        (
            limits(256, components_needed - 1, bytes_needed),
            ComponentFailure::ComponentLimit,
        ),
        (
            limits(256, components_needed, bytes_needed - 1),
            ComponentFailure::ByteLimit,
        ),
    ] {
        let Err(error) = construct(supplied.clone(), limit) else {
            panic!("{source:?} must exceed {limit:?}")
        };
        assert_eq!(component_error(&error), Some(expected), "{source:?}");
    }
}

#[test]
fn construction_admits_exactly_the_documented_component_and_byte_budget() {
    // `"a" "b"`: two strings and the whitespace between them, seven bytes.
    assert_exact_budget(
        "\"a\" \"b\"",
        3,
        7,
        |supplied, limits| Counter::try_from_components(CounterKind::Symbols, supplied, limits),
        |error| error.component_error().map(component_failure),
    );
    // A function opening counts once and its closer not at all: `env(x)` is two nodes.
    assert_exact_budget(
        "env(x)",
        2,
        6,
        |supplied, limits| Counter::try_from_components(CounterKind::Prefix, supplied, limits),
        |error| error.component_error().map(component_failure),
    );
    assert_exact_budget("env(x)", 2, 6, Display::try_from_components, |error| {
        error.component_error().map(component_failure)
    });
    assert_exact_budget("swap", 1, 4, Display::try_from_components, |error| {
        error.component_error().map(component_failure)
    });
    assert_exact_budget(
        "1 2",
        3,
        3,
        |supplied, limits| Feature::try_from_components(FeatureKind::Styleset, supplied, limits),
        |error| error.component_error().map(component_failure),
    );
    assert_exact_budget(
        "env(x)",
        2,
        6,
        |supplied, limits| Feature::try_from_components(FeatureKind::Styleset, supplied, limits),
        |error| error.component_error().map(component_failure),
    );
    // One nesting level is the depth `env(x)` needs.
    assert!(
        Display::try_from_components(components("env(x)"), limits(1, usize::MAX, usize::MAX))
            .is_ok()
    );
}

#[test]
fn reentry_charges_only_the_replacement_against_the_supplied_budget() {
    // The receiver needs more than the replacement does; only the replacement counts.
    let big = components("env(a, b, c, d)");
    let counter =
        Counter::try_from_components(CounterKind::Prefix, big.clone(), Default::default()).unwrap();
    let display = Display::try_from_components(big.clone(), Default::default()).unwrap();
    let feature =
        Feature::try_from_components(FeatureKind::Styleset, big, Default::default()).unwrap();
    assert_exact_budget(
        "\"x\"",
        1,
        3,
        |supplied, limits| counter.reparse_after_substitution(supplied, limits),
        |error| error.component_error().map(component_failure),
    );
    assert_exact_budget(
        "swap",
        1,
        4,
        |supplied, limits| display.reparse_after_substitution(supplied, limits),
        |error| error.component_error().map(component_failure),
    );
    assert_exact_budget(
        "1 2",
        3,
        3,
        |supplied, limits| feature.reparse_after_substitution(supplied, limits),
        |error| error.component_error().map(component_failure),
    );
}

#[test]
fn implicit_closers_count_toward_the_byte_limit_before_the_closure_is_refused() {
    // `env(x` serializes as `env(x)`: six bytes, one of them the implicit closer.
    let supplied = components("env(x");
    let at_limit = limits(256, usize::MAX, 6);
    let below_limit = limits(256, usize::MAX, 5);
    let counter =
        Counter::try_from_components(CounterKind::Prefix, supplied.clone(), at_limit).unwrap_err();
    let display = Display::try_from_components(supplied.clone(), at_limit).unwrap_err();
    let feature = Feature::try_from_components(FeatureKind::Styleset, supplied.clone(), at_limit)
        .unwrap_err();
    assert!(counter.is_recovered_component(), "{counter:?}");
    assert!(display.is_recovered_component(), "{display:?}");
    assert!(feature.is_recovered_component(), "{feature:?}");
    // A limit the closed form exceeds is reported as the limit, ahead of the closure.
    let counter = Counter::try_from_components(CounterKind::Prefix, supplied.clone(), below_limit)
        .unwrap_err();
    let display = Display::try_from_components(supplied.clone(), below_limit).unwrap_err();
    let feature =
        Feature::try_from_components(FeatureKind::Styleset, supplied, below_limit).unwrap_err();
    for failure in [
        counter.component_error().map(component_failure),
        display.component_error().map(component_failure),
        feature.component_error().map(component_failure),
    ] {
        assert_eq!(failure, Some(ComponentFailure::ByteLimit));
    }
}

#[test]
fn rejections_apply_in_the_documented_order_and_report_the_earlier_cause() {
    // Construction: a recovered closure is refused ahead of the grammar failure it also has.
    for source in ["1 env(x", "SECRET env(x"] {
        let counter = Counter::try_from_components(
            CounterKind::Prefix,
            components(source),
            Default::default(),
        )
        .unwrap_err();
        let display =
            Display::try_from_components(components(source), Default::default()).unwrap_err();
        let feature = Feature::try_from_components(
            FeatureKind::Styleset,
            components(source),
            Default::default(),
        )
        .unwrap_err();
        assert!(counter.is_recovered_component(), "{source}: {counter:?}");
        assert!(display.is_recovered_component(), "{source}: {display:?}");
        assert!(feature.is_recovered_component(), "{source}: {feature:?}");
        assert!(counter.diagnostic().is_none());
    }
    // Reentry: not pending, then limits, then closure, then residual substitution, then grammar.
    let pending_counter = Counter::try_from_components(
        CounterKind::Prefix,
        components("env(x)"),
        Default::default(),
    )
    .unwrap();
    let pending_display =
        Display::try_from_components(components("env(x)"), Default::default()).unwrap();
    let pending_feature = Feature::try_from_components(
        FeatureKind::Styleset,
        components("env(x)"),
        Default::default(),
    )
    .unwrap();
    let none = limits(0, 0, 0);
    // The limit is checked before the closure and the residual function.
    let replacement = components("f(var(--x)");
    assert_eq!(
        pending_counter
            .reparse_after_substitution(replacement.clone(), none)
            .unwrap_err()
            .component_error()
            .map(component_failure),
        Some(ComponentFailure::ComponentLimit)
    );
    assert_eq!(
        pending_display
            .reparse_after_substitution(replacement.clone(), none)
            .unwrap_err()
            .component_error()
            .map(component_failure),
        Some(ComponentFailure::ComponentLimit)
    );
    assert_eq!(
        pending_feature
            .reparse_after_substitution(replacement.clone(), none)
            .unwrap_err()
            .component_error()
            .map(component_failure),
        Some(ComponentFailure::ComponentLimit)
    );
    // The closure is refused before the residual function it contains.
    let counter = pending_counter
        .reparse_after_substitution(replacement.clone(), Default::default())
        .unwrap_err();
    let display = pending_display
        .reparse_after_substitution(replacement.clone(), Default::default())
        .unwrap_err();
    let feature = pending_feature
        .reparse_after_substitution(replacement, Default::default())
        .unwrap_err();
    assert!(counter.is_recovered_component(), "{counter:?}");
    assert!(display.is_recovered_component(), "{display:?}");
    assert!(feature.is_recovered_component(), "{feature:?}");
    // A residual function that is closed is refused before the grammar judges the rest.
    let replacement = components("SECRET 1 var(--x)");
    let counter = pending_counter
        .reparse_after_substitution(replacement.clone(), Default::default())
        .unwrap_err();
    let display = pending_display
        .reparse_after_substitution(replacement.clone(), Default::default())
        .unwrap_err();
    let feature = pending_feature
        .reparse_after_substitution(replacement, Default::default())
        .unwrap_err();
    assert!(counter.is_residual_substitution(), "{counter:?}");
    assert!(display.is_residual_substitution(), "{display:?}");
    assert!(feature.is_residual_substitution(), "{feature:?}");
}

#[test]
fn reentered_values_serialize_specified_exactly_like_their_replacement() {
    let pending = Display::try_from_components(components("env(x)"), Default::default()).unwrap();
    // Specified serialization is the value's canonical text: trivia is not part of it.
    for (source, expected) in [
        ("auto", "auto"),
        ("swap", "swap"),
        ("fallback", "fallback"),
        (" swap /* c */ ", "swap"),
    ] {
        let completed = pending
            .reparse_after_substitution(components(source), Default::default())
            .unwrap();
        assert_eq!(
            completed.serialize_specified(Default::default()).unwrap(),
            expected,
            "display {source:?}"
        );
    }
    let pending = Feature::try_from_components(
        FeatureKind::CharacterVariant,
        components("env(x)"),
        Default::default(),
    )
    .unwrap();
    for (source, expected) in [("1", "1"), ("1 2", "1 2"), (" 1  2 ", "1 2")] {
        let completed = pending
            .reparse_after_substitution(components(source), Default::default())
            .unwrap();
        assert_eq!(
            completed.serialize_specified(Default::default()).unwrap(),
            expected,
            "feature {source:?}"
        );
    }
    let pending =
        Counter::try_from_components(CounterKind::Range, components("env(x)"), Default::default())
            .unwrap();
    for source in ["auto", "1 9", "infinite 5, -3 infinite"] {
        let completed = pending
            .reparse_after_substitution(components(source), Default::default())
            .unwrap();
        assert_eq!(
            completed.serialize_specified(Default::default()).unwrap(),
            source,
            "counter range {source:?}"
        );
    }
}

// `@color-profile` and `@font-palette-values` descriptor values: the same checked
// construction and pending reentry, with the cases above that their own tests do
// not pin. Color 5 section 5.3 (`src`, `rendering-intent`, `components`) and
// Fonts 4 section 9.2 (`font-family`, `base-palette`, `override-colors`) give the
// grammars.

fn profile(kind: ProfileKind, source: &str) -> Profile {
    Profile::try_new(kind, components(source), Default::default()).unwrap()
}
fn palette(kind: PaletteKind, source: &str) -> Palette {
    Palette::try_new(kind, components(source), Default::default()).unwrap()
}
fn invalid_value(
    position: Coordinates,
    token: Option<(CssTokenKind, &'static str)>,
    span: Option<(Coordinates, Coordinates)>,
) -> Rejection {
    Rejection {
        code: CssErrorCode::InvalidDescriptorValue,
        position,
        token,
        span,
    }
}

#[test]
fn color_profile_grammar_rejections_name_the_first_violating_token_and_its_coordinates() {
    let cases = [
        (
            ProfileKind::Src,
            "1",
            invalid_value(
                (0, 0, 0),
                token(CssTokenKind::Number, "1"),
                Some(((0, 0, 0), (1, 0, 1))),
            ),
        ),
        (ProfileKind::Src, "", invalid_value((0, 0, 0), None, None)),
        (
            ProfileKind::Src,
            "url(a) url(b)",
            invalid_value(
                (7, 0, 7),
                token(CssTokenKind::Url, "url(b)"),
                Some(((7, 0, 7), (13, 0, 13))),
            ),
        ),
        (
            ProfileKind::Src,
            "url(a)\n  1",
            invalid_value(
                (9, 1, 2),
                token(CssTokenKind::Number, "1"),
                Some(((9, 1, 2), (10, 1, 3))),
            ),
        ),
        (
            ProfileKind::RenderingIntent,
            "auto",
            invalid_value(
                (0, 0, 0),
                token(CssTokenKind::Ident, "auto"),
                Some(((0, 0, 0), (4, 0, 4))),
            ),
        ),
        (
            ProfileKind::RenderingIntent,
            "perceptual saturation",
            invalid_value(
                (11, 0, 11),
                token(CssTokenKind::Ident, "saturation"),
                Some(((11, 0, 11), (21, 0, 21))),
            ),
        ),
        (
            ProfileKind::RenderingIntent,
            "",
            invalid_value((0, 0, 0), None, None),
        ),
        // The two-byte U+00E9 makes the token four bytes and three UTF-16 units long.
        (
            ProfileKind::RenderingIntent,
            "\"\u{e9}\" perceptual",
            invalid_value(
                (0, 0, 0),
                token(CssTokenKind::String, "\"\u{e9}\""),
                Some(((0, 0, 0), (4, 0, 3))),
            ),
        ),
        // `<ident>#` has no whitespace-separated second name.
        (
            ProfileKind::Components,
            "red green",
            invalid_value(
                (4, 0, 4),
                token(CssTokenKind::Ident, "green"),
                Some(((4, 0, 4), (9, 0, 9))),
            ),
        ),
        (
            ProfileKind::Components,
            "red,\n 1",
            invalid_value(
                (6, 1, 1),
                token(CssTokenKind::Number, "1"),
                Some(((6, 1, 1), (7, 1, 2))),
            ),
        ),
        // A list may not end in a comma: the comma is the first token past the list.
        (
            ProfileKind::Components,
            "red,",
            invalid_value(
                (3, 0, 3),
                token(CssTokenKind::Comma, ","),
                Some(((3, 0, 3), (4, 0, 4))),
            ),
        ),
        (
            ProfileKind::Components,
            "",
            invalid_value((0, 0, 0), None, None),
        ),
    ];
    for (kind, source, expected) in cases {
        let constructed =
            Profile::try_new(kind, components(source), Default::default()).unwrap_err();
        assert_rejection(
            constructed.diagnostic(),
            Some(constructed.origin()),
            source,
            &expected,
        );
        assert!(constructed.component_error().is_none());
        let reentered = profile(kind, "env(x)")
            .reparse_after_substitution(components(source), Default::default())
            .unwrap_err();
        assert_rejection(
            reentered.diagnostic(),
            Some(reentered.origin()),
            source,
            &expected,
        );
        assert_eq!(constructed, reentered, "{kind:?} {source:?}");
    }
}

#[test]
fn font_palette_grammar_rejections_name_the_first_violating_token_and_its_coordinates() {
    let cases = [
        (
            PaletteKind::FontFamily,
            "1",
            invalid_value(
                (0, 0, 0),
                token(CssTokenKind::Number, "1"),
                Some(((0, 0, 0), (1, 0, 1))),
            ),
        ),
        (
            PaletteKind::FontFamily,
            "",
            invalid_value((0, 0, 0), None, None),
        ),
        // A bare generic family is not a `<family-name>`.
        (
            PaletteKind::FontFamily,
            "serif sans-serif",
            invalid_value(
                (0, 0, 0),
                token(CssTokenKind::Ident, "serif"),
                Some(((0, 0, 0), (5, 0, 5))),
            ),
        ),
        (
            PaletteKind::BasePalette,
            "-1",
            invalid_value(
                (0, 0, 0),
                token(CssTokenKind::Number, "-1"),
                Some(((0, 0, 0), (2, 0, 2))),
            ),
        ),
        (
            PaletteKind::BasePalette,
            "1 2",
            invalid_value(
                (2, 0, 2),
                token(CssTokenKind::Number, "2"),
                Some(((2, 0, 2), (3, 0, 3))),
            ),
        ),
        (
            PaletteKind::BasePalette,
            "",
            invalid_value((0, 0, 0), None, None),
        ),
        (
            PaletteKind::BasePalette,
            "banana",
            invalid_value(
                (0, 0, 0),
                token(CssTokenKind::Ident, "banana"),
                Some(((0, 0, 0), (6, 0, 6))),
            ),
        ),
        (
            PaletteKind::BasePalette,
            "light\n dark",
            invalid_value(
                (7, 1, 1),
                token(CssTokenKind::Ident, "dark"),
                Some(((7, 1, 1), (11, 1, 5))),
            ),
        ),
        (
            PaletteKind::OverrideColors,
            "0 red 1",
            invalid_value(
                (6, 0, 6),
                token(CssTokenKind::Number, "1"),
                Some(((6, 0, 6), (7, 0, 7))),
            ),
        ),
        (
            PaletteKind::OverrideColors,
            "red",
            invalid_value(
                (0, 0, 0),
                token(CssTokenKind::Ident, "red"),
                Some(((0, 0, 0), (3, 0, 3))),
            ),
        ),
        (
            PaletteKind::OverrideColors,
            "",
            invalid_value((0, 0, 0), None, None),
        ),
        (
            PaletteKind::OverrideColors,
            "0 red,\n \"\u{e9}\" 1",
            invalid_value(
                (8, 1, 1),
                token(CssTokenKind::String, "\"\u{e9}\""),
                Some(((8, 1, 1), (12, 1, 4))),
            ),
        ),
    ];
    for (kind, source, expected) in cases {
        let constructed =
            Palette::try_new(kind, components(source), Default::default()).unwrap_err();
        assert_rejection(
            constructed.diagnostic(),
            constructed.origin(),
            source,
            &expected,
        );
        assert!(constructed.component_error().is_none());
        let reentered = palette(kind, "env(x)")
            .reparse_after_substitution(components(source), Default::default())
            .unwrap_err();
        assert_rejection(
            reentered.diagnostic(),
            reentered.origin(),
            source,
            &expected,
        );
        assert_eq!(constructed, reentered, "{kind:?} {source:?}");
    }
}

fn profile_failures() -> Failures<ProfileError> {
    let kind = ProfileKind::RenderingIntent;
    Failures {
        component: Profile::try_new(kind, components("env(x)"), limits(256, 1, usize::MAX))
            .unwrap_err(),
        grammar: Profile::try_new(kind, components("SECRET"), Default::default()).unwrap_err(),
        recovered: Profile::try_new(kind, components("env(SECRET"), Default::default())
            .unwrap_err(),
        residual: profile(kind, "env(x)")
            .reparse_after_substitution(components("f(var(--SECRET))"), Default::default())
            .unwrap_err(),
        not_pending: profile(kind, "perceptual")
            .reparse_after_substitution(components("env(x)"), Default::default())
            .unwrap_err(),
    }
}

fn palette_failures() -> Failures<PaletteError> {
    let kind = PaletteKind::BasePalette;
    Failures {
        component: Palette::try_new(kind, components("env(x)"), limits(256, 1, usize::MAX))
            .unwrap_err(),
        grammar: Palette::try_new(kind, components("SECRET"), Default::default()).unwrap_err(),
        recovered: Palette::try_new(kind, components("env(SECRET"), Default::default())
            .unwrap_err(),
        residual: palette(kind, "env(x)")
            .reparse_after_substitution(components("f(var(--SECRET))"), Default::default())
            .unwrap_err(),
        not_pending: palette(kind, "12")
            .reparse_after_substitution(components("env(x)"), Default::default())
            .unwrap_err(),
    }
}

#[test]
fn color_profile_and_palette_failures_are_exclusive_classes_with_rule_named_messages() {
    assert_failure_surface(
        "color-profile",
        &profile_failures(),
        |error| {
            [
                error.component_error().is_some(),
                error.diagnostic().is_some(),
                error.is_recovered_component(),
                error.is_residual_substitution(),
                error.is_not_pending(),
            ]
        },
        |error| error.component_error().map(ToString::to_string),
        |error| error.diagnostic().map(ToString::to_string),
    );
    assert_failure_surface(
        "font-palette-values",
        &palette_failures(),
        |error| {
            [
                error.component_error().is_some(),
                error.diagnostic().is_some(),
                error.is_recovered_component(),
                error.is_residual_substitution(),
                error.is_not_pending(),
            ]
        },
        |error| error.component_error().map(ToString::to_string),
        |error| error.diagnostic().map(ToString::to_string),
    );
}

#[test]
fn color_profile_and_palette_admit_exactly_the_documented_component_and_byte_budget() {
    let profile_budget = |kind: ProfileKind, source: &str, components: usize, bytes: usize| {
        assert_exact_budget(
            source,
            components,
            bytes,
            |supplied, limits| Profile::try_new(kind, supplied, limits),
            |error| error.component_error().map(component_failure),
        );
    };
    profile_budget(ProfileKind::Components, "red, green", 4, 10);
    profile_budget(ProfileKind::RenderingIntent, "perceptual", 1, 10);
    profile_budget(ProfileKind::Src, "env(x)", 2, 6);
    let palette_budget = |kind: PaletteKind, source: &str, components: usize, bytes: usize| {
        assert_exact_budget(
            source,
            components,
            bytes,
            |supplied, limits| Palette::try_new(kind, supplied, limits),
            |error| error.component_error().map(component_failure),
        );
    };
    palette_budget(PaletteKind::BasePalette, "12", 1, 2);
    palette_budget(PaletteKind::OverrideColors, "0 red", 3, 5);
    palette_budget(PaletteKind::FontFamily, "env(x)", 2, 6);
    // Reentry charges only the replacement, however large the receiver was.
    let big = "env(a, b, c, d)";
    let receiver = profile(ProfileKind::RenderingIntent, big);
    assert_exact_budget(
        "perceptual",
        1,
        10,
        |supplied, limits| receiver.reparse_after_substitution(supplied, limits),
        |error| error.component_error().map(component_failure),
    );
    let receiver = palette(PaletteKind::BasePalette, big);
    assert_exact_budget(
        "12",
        1,
        2,
        |supplied, limits| receiver.reparse_after_substitution(supplied, limits),
        |error| error.component_error().map(component_failure),
    );
}

#[test]
fn color_profile_and_palette_construction_count_implicit_closers_before_refusing_the_closure() {
    let supplied = components("env(x");
    for (kind_profile, kind_palette) in [
        (ProfileKind::Src, PaletteKind::FontFamily),
        (ProfileKind::RenderingIntent, PaletteKind::BasePalette),
        (ProfileKind::Components, PaletteKind::OverrideColors),
    ] {
        let at_limit = limits(256, usize::MAX, 6);
        let below_limit = limits(256, usize::MAX, 5);
        let profile_error = Profile::try_new(kind_profile, supplied.clone(), at_limit).unwrap_err();
        let palette_error = Palette::try_new(kind_palette, supplied.clone(), at_limit).unwrap_err();
        assert!(profile_error.is_recovered_component(), "{profile_error:?}");
        assert!(palette_error.is_recovered_component(), "{palette_error:?}");
        let profile_error =
            Profile::try_new(kind_profile, supplied.clone(), below_limit).unwrap_err();
        let palette_error =
            Palette::try_new(kind_palette, supplied.clone(), below_limit).unwrap_err();
        assert_eq!(
            profile_error.component_error().map(component_failure),
            Some(ComponentFailure::ByteLimit)
        );
        assert_eq!(
            palette_error.component_error().map(component_failure),
            Some(ComponentFailure::ByteLimit)
        );
    }
}

#[test]
fn color_profile_rejections_apply_limits_then_closure_then_residual_then_grammar() {
    let kind = ProfileKind::RenderingIntent;
    // Construction: the closure is refused before the grammar failure it also has.
    for source in ["1 env(x", "SECRET env(x"] {
        let error = Profile::try_new(kind, components(source), Default::default()).unwrap_err();
        assert!(error.is_recovered_component(), "{source}: {error:?}");
        assert!(error.diagnostic().is_none());
    }
    // Reentry: an ordinary receiver is refused before the limits are read.
    let error = profile(kind, "perceptual")
        .reparse_after_substitution(components("env(x)"), limits(0, 0, 0))
        .unwrap_err();
    assert!(error.is_not_pending(), "{error:?}");
    let pending = profile(kind, "env(x)");
    let replacement = components("f(var(--x)");
    let error = pending
        .reparse_after_substitution(replacement.clone(), limits(0, 0, 0))
        .unwrap_err();
    assert_eq!(
        error.component_error().map(component_failure),
        Some(ComponentFailure::ComponentLimit)
    );
    // The closure is refused before the residual function it contains.
    let error = pending
        .reparse_after_substitution(replacement, Default::default())
        .unwrap_err();
    assert!(error.is_recovered_component(), "{error:?}");
    // A closed residual function is refused before the grammar judges the rest.
    let error = pending
        .reparse_after_substitution(components("SECRET 1 var(--x)"), Default::default())
        .unwrap_err();
    assert!(error.is_residual_substitution(), "{error:?}");
    let error = pending
        .reparse_after_substitution(components("SECRET"), Default::default())
        .unwrap_err();
    assert!(error.diagnostic().is_some(), "{error:?}");
}

#[test]
fn palette_rejections_apply_limits_then_residual_then_closure_then_grammar() {
    let kind = PaletteKind::BasePalette;
    // Construction: the closure is refused before the grammar failure it also has.
    for source in ["1 env(x", "SECRET env(x"] {
        let error = Palette::try_new(kind, components(source), Default::default()).unwrap_err();
        assert!(error.is_recovered_component(), "{source}: {error:?}");
        assert!(error.diagnostic().is_none());
    }
    // Reentry: an ordinary receiver is refused before the limits are read.
    let error = palette(kind, "12")
        .reparse_after_substitution(components("env(x)"), limits(0, 0, 0))
        .unwrap_err();
    assert!(error.is_not_pending(), "{error:?}");
    let pending = palette(kind, "env(x)");
    let replacement = components("f(var(--x)");
    let error = pending
        .reparse_after_substitution(replacement.clone(), limits(0, 0, 0))
        .unwrap_err();
    assert_eq!(
        error.component_error().map(component_failure),
        Some(ComponentFailure::ComponentLimit)
    );
    // Unlike the other descriptor values, a residual function is refused before the
    // implicit closure that also holds it ("Residual ... anywhere in the tree is
    // rejected first"): the failure names the opening `var(`, not the closer.
    let error = pending
        .reparse_after_substitution(replacement, Default::default())
        .unwrap_err();
    assert!(error.is_residual_substitution(), "{error:?}");
    assert!(!error.is_recovered_component());
    let Some(CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin))) = error.origin() else {
        panic!("residual opener")
    };
    assert_origin_region(origin, "f(var(--x)", (2, 0, 2), (6, 0, 6));
    // A recovered closure that holds no residual function is still refused ...
    let error = pending
        .reparse_after_substitution(components("calc(1 + 1"), Default::default())
        .unwrap_err();
    assert!(error.is_recovered_component(), "{error:?}");
    // ... and so is a closed residual function ahead of the grammar.
    let error = pending
        .reparse_after_substitution(components("SECRET 1 var(--x)"), Default::default())
        .unwrap_err();
    assert!(error.is_residual_substitution(), "{error:?}");
    let error = pending
        .reparse_after_substitution(components("SECRET"), Default::default())
        .unwrap_err();
    assert!(error.diagnostic().is_some(), "{error:?}");
}

#[test]
fn reentered_color_profile_and_palette_values_serialize_specified_canonically() {
    for (kind, source, expected) in [
        (ProfileKind::RenderingIntent, " perceptual ", "perceptual"),
        (ProfileKind::RenderingIntent, " PERCEPTUAL ", "perceptual"),
        (ProfileKind::Components, "red,green", "red, green"),
        (ProfileKind::Components, " red ,  green ", "red, green"),
        (ProfileKind::Src, " url(a) ", "url(\"a\")"),
        (ProfileKind::Src, "url( \"a\" )", "url(\"a\")"),
    ] {
        let completed = profile(kind, "env(x)")
            .reparse_after_substitution(components(source), Default::default())
            .unwrap();
        assert_eq!(
            completed.serialize_specified(Default::default()).unwrap(),
            expected,
            "{kind:?} {source:?}"
        );
    }
    for (kind, source, expected) in [
        (PaletteKind::BasePalette, " 12 ", "12"),
        (PaletteKind::BasePalette, " LIGHT ", "light"),
        (PaletteKind::OverrideColors, "0 red", "0 red"),
        (
            PaletteKind::OverrideColors,
            " 0   red ,1 blue ",
            "0 red, 1 blue",
        ),
        (PaletteKind::FontFamily, " Foo  Bar ", "Foo Bar"),
    ] {
        let completed = palette(kind, "env(x)")
            .reparse_after_substitution(components(source), Default::default())
            .unwrap();
        assert_eq!(
            completed.serialize_specified(Default::default()).unwrap(),
            expected,
            "{kind:?} {source:?}"
        );
    }
}

//! Where the diagnostics of rejected descriptors are reported, for sources that
//! leave a function, a URL or a block open, put an annotation beside a missing
//! value, or put a closing token in a value. The expected positions and tokens
//! are the diagnostic contract of the `@font-palette-values`, `@color-profile`,
//! `@counter-style` and `@font-feature-values` bodies. Syntax 3 (2026 CRD)
//! §5.5.5 and §5.5.6 give the units.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};

/// The (position, span start, span end, code) of every diagnostic of the sheet
/// other than the rejection of the rule that holds the descriptors.
fn descriptor_diagnostics(source: &str) -> Vec<(usize, usize, usize, CssErrorCode)> {
    CssParser::default()
        .parse_sheet(source)
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.error().code() != CssErrorCode::InvalidQualifiedRule)
        .map(|diagnostic| {
            (
                diagnostic.error().position().byte_offset().value(),
                diagnostic.span().start().byte_offset().value(),
                diagnostic.span().end().byte_offset().value(),
                diagnostic.error().code(),
            )
        })
        .collect()
}

/// The (position, encountered token) of every diagnostic of the sheet other than
/// the rejection of the rule that holds the descriptors.
fn encountered_tokens(source: &str) -> Vec<(usize, Option<(String, String)>)> {
    CssParser::default()
        .parse_sheet(source)
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.error().code() != CssErrorCode::InvalidQualifiedRule)
        .map(|diagnostic| {
            (
                diagnostic.error().position().byte_offset().value(),
                diagnostic
                    .error()
                    .encountered()
                    .map(|token| (format!("{:?}", token.kind()), token.authored().to_owned())),
            )
        })
        .collect()
}

/// The four rules, each with a descriptor whose value is complete.
const RULES: [(&str, &str); 4] = [
    ("@font-palette-values --p", "base-palette: light"),
    ("@color-profile --c", "rendering-intent: perceptual"),
    ("@counter-style x", "system: cyclic"),
    ("@font-feature-values A", "font-display: swap"),
];

#[test]
fn a_function_closed_by_the_end_of_the_source_is_reported_at_that_end() {
    // The `}` and the newline belong to the unclosed function (Syntax 3 §5.5.8),
    // so the descriptor ends with the source, and its failure is reported there.
    assert_eq!(
        descriptor_diagnostics("@font-palette-values --p {base-palette: calc(1 + }\n"),
        [(51, 26, 51, CssErrorCode::InvalidDescriptorValue)]
    );
    assert_eq!(
        descriptor_diagnostics("@font-palette-values --p {override-colors: 0 rgb(1 2}\n"),
        [(54, 26, 54, CssErrorCode::InvalidDescriptorValue)]
    );
    assert_eq!(
        descriptor_diagnostics("@font-palette-values --p {base-palette: calc(1 + }"),
        [(50, 26, 50, CssErrorCode::InvalidDescriptorValue)]
    );
}

#[test]
fn an_unclosed_function_in_a_counter_descriptor_is_reported_with_the_end_of_the_block() {
    assert_eq!(
        descriptor_diagnostics("@counter-style x {symbols: func(1}\n"),
        [
            (35, 18, 35, CssErrorCode::InvalidDescriptorValue),
            (35, 35, 35, CssErrorCode::UnexpectedEnd),
        ]
    );
}

#[test]
fn a_missing_value_before_an_annotation_names_the_token_that_follows_it() {
    // The colour that the index needs is missing, and the `!` follows at once.
    assert_eq!(
        encountered_tokens("@font-palette-values --p {override-colors: 0!x}"),
        [(44, Some(("Delim".to_owned(), "!".to_owned())))]
    );
    assert_eq!(
        encountered_tokens("@font-palette-values --p {override-colors: 0 !x}"),
        [(45, Some(("Delim".to_owned(), "!".to_owned())))]
    );
    assert_eq!(
        encountered_tokens("@font-palette-values --p {override-colors: 0, !x}"),
        [(44, Some(("Comma".to_owned(), ",".to_owned())))]
    );
}

#[test]
fn an_unclosed_url_is_reported_at_the_end_of_the_source() {
    assert_eq!(
        descriptor_diagnostics("@color-profile --c {src: url(x}\n"),
        [
            (32, 32, 32, CssErrorCode::UnexpectedEnd),
            (32, 32, 32, CssErrorCode::UnexpectedEnd),
        ]
    );
}

#[test]
fn the_root_of_a_value_is_admitted_before_its_components_are_collected() {
    // A closing `]` at the root of the value is the stated fault in every
    // rule, not the failure to collect the components that follow it.
    for (head, declaration) in RULES {
        let source = format!("{head} {{ {declaration} ] }}");
        let at = source.find(']').unwrap();
        let tokens = encountered_tokens(&source);
        let descriptor = tokens
            .iter()
            .find(|(_, token)| token.is_some())
            .unwrap_or_else(|| panic!("{source:?}: {tokens:?}"));
        assert_eq!(
            descriptor,
            &(at, Some(("CloseSquareBracket".to_owned(), "]".to_owned()))),
            "{source:?}"
        );
    }
}

#[test]
fn a_unit_that_begins_with_no_break_space_and_holds_an_at_rule_is_dropped_by_the_heuristic_of_its_rule()
 {
    // Characterization of a heuristic that reads the raw text of the unit:
    // `str::trim_start` removes U+00A0, which the tokenizer reads as an
    // identifier. The palette, profile and feature bodies judge a unit that
    // begins with `@` after trimming to be an at-rule; the counter body judges
    // by its first token, so it is a descriptor. A unit that begins with an
    // identifier is a descriptor in all four. The step that replaces the text
    // heuristics changes this.
    let action = |source: &str| {
        CssParser::default()
            .parse_sheet(source)
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.error().code() == CssErrorCode::UnexpectedToken)
            .map(|diagnostic| diagnostic.action())
    };
    for (head, body, expected) in [
        (
            "@font-palette-values --p",
            "\u{a0} @x {}",
            CssRecoveryAction::DropAtRule,
        ),
        (
            "@color-profile --c",
            "\u{a0} @x {}",
            CssRecoveryAction::DropAtRule,
        ),
        (
            "@counter-style x",
            "\u{a0} @x {}",
            CssRecoveryAction::DropDescriptor,
        ),
        (
            "@font-feature-values A",
            "\u{a0} @x {}",
            CssRecoveryAction::DropAtRule,
        ),
        (
            "@font-palette-values --p",
            "a @x {}",
            CssRecoveryAction::DropDescriptor,
        ),
        (
            "@color-profile --c",
            "a @x {}",
            CssRecoveryAction::DropDescriptor,
        ),
        (
            "@counter-style x",
            "a @x {}",
            CssRecoveryAction::DropDescriptor,
        ),
        (
            "@font-feature-values A",
            "a @x {}",
            CssRecoveryAction::DropDescriptor,
        ),
    ] {
        let source = format!("{head} {{ {body} }}");
        assert_eq!(action(&source), Some(expected), "{source:?}");
    }
}

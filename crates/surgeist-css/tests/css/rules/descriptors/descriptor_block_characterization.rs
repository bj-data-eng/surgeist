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
fn a_unit_that_begins_with_no_break_space_is_not_judged_as_an_at_rule_in_every_descriptor_body() {
    // Syntax 3 section 4.2: whitespace is a newline, a tab or a space, so
    // U+00A0 is not whitespace and is not skipped before the unit. The unit
    // begins with U+00A0, not with the at-keyword that follows it, and a
    // descriptor body does not drop it as an at-rule. A unit that begins with
    // an identifier and holds the same text after it is dropped as a descriptor.
    let action = |source: &str| {
        CssParser::default()
            .parse_sheet(source)
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.error().code() == CssErrorCode::UnexpectedToken)
            .map(|diagnostic| diagnostic.action())
    };
    for head in [
        "@font-palette-values --p",
        "@color-profile --c",
        "@counter-style x",
        "@font-feature-values A",
    ] {
        let source = format!("{head} {{ \u{a0} @x {{}} }}");
        let dropped = action(&source);
        assert!(dropped.is_some(), "{source:?}");
        assert_ne!(dropped, Some(CssRecoveryAction::DropAtRule), "{source:?}");
        let source = format!("{head} {{ a @x {{}} }}");
        assert_eq!(
            action(&source),
            Some(CssRecoveryAction::DropDescriptor),
            "{source:?}"
        );
    }
}

#[test]
fn a_dropped_unit_that_does_not_begin_with_an_at_keyword_is_never_reported_as_an_at_rule() {
    // Syntax 3 §5.5.5: only an <at-keyword-token> begins an at-rule; any other
    // unit that is not a declaration is consumed as a qualified rule. The
    // block bodies that drive their own units (`@counter-style`, `@page` and
    // its margin boxes) report the unit as the qualified rule it is, and an
    // at-rule unit stays an at-rule.
    let actions_at = |source: &str, at: usize| {
        CssParser::default()
            .parse_sheet(source)
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.span().start().byte_offset().value() == at)
            .map(|diagnostic| (diagnostic.error().code(), diagnostic.action()))
            .collect::<Vec<_>>()
    };
    for template in [
        "@counter-style x { @@ }",
        "@page { @@ }",
        "@page { @top-left { @@ } }",
        "@page :first { margin: 1px; @@ ; size: a4 }",
    ] {
        for unit in [
            ".a {}", "1 {}", "& a {}", "#a {}", ":a {}", ".a;", ":x;", "1;",
        ] {
            let source = template.replace("@@", unit);
            let at = source.find(unit).unwrap();
            let found = actions_at(&source, at);
            let [(code, action)] = found.as_slice() else {
                panic!("{source:?}: {found:?}");
            };
            assert_eq!(*action, CssRecoveryAction::DropQualifiedRule, "{source:?}");
            if template.starts_with("@counter-style") && unit.ends_with("{}") {
                assert_eq!(*code, CssErrorCode::InvalidQualifiedRule, "{source:?}");
            }
        }
        let unit = "@x {}";
        let source = template.replace("@@", unit);
        let at = source.find(unit).unwrap();
        let found = actions_at(&source, at);
        assert_eq!(
            found.iter().map(|(_, action)| *action).collect::<Vec<_>>(),
            [CssRecoveryAction::DropAtRule],
            "{source:?}"
        );
    }
}

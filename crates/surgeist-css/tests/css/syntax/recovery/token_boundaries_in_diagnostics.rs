//! A diagnostic is positioned and classified by the tokens of the source, never
//! by the bytes the tokens are written with.
//!
//! Syntax 3 §4.3.2 and §4.3.5: a comment or a string is one token, and what it
//! holds (a `;`, `:`, `,`, `(`, `[` or `@`) is no delimiter. §4.2: whitespace is
//! a newline, a tab or a space, so U+00A0, U+0085, U+2028 and U+3000 are not
//! whitespace. §5.5.2: an at-rule ends at the `;` token that follows its prelude
//! in its own list, never at a `;` inside a string, a comment, an escape or an
//! unclosed block, and a unit is an at-rule when its first token is an
//! at-keyword.
use surgeist_css::CssParser;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction, CssRecoveryDiagnostic};
use surgeist_css::rule::font_face::CssFontFaceDescriptors;
use surgeist_css::rule::page::CssPageBody;
use surgeist_css::rule::{CssBlockContext, CssBlockFragment, CssRuleList};
use surgeist_css::selector::CssNamespaceContext;

fn sheet(source: &str) -> Vec<CssRecoveryDiagnostic> {
    CssParser::default()
        .parse_sheet(source)
        .diagnostics()
        .to_vec()
}

fn encountered(diagnostic: &CssRecoveryDiagnostic) -> Option<(String, String)> {
    diagnostic
        .error()
        .encountered()
        .map(|token| (format!("{:?}", token.kind()), token.authored().to_owned()))
}

fn offset(diagnostic: &CssRecoveryDiagnostic) -> usize {
    diagnostic.error().position().byte_offset().value()
}

/// What a comment holds changes no diagnostic: the comment is trivia.
#[test]
fn punctuation_inside_a_comment_before_the_end_of_a_value_names_no_token() {
    for punctuation in [";", ":", ",", "(", "[", "{", "@x", "!"] {
        let source = format!("a{{ width: /* {punctuation} */ }}");
        let [diagnostic] = sheet(&source)
            .try_into()
            .unwrap_or_else(|diagnostics: Vec<_>| panic!("{source:?}: {diagnostics:?}"));
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(encountered(&diagnostic), None, "{source:?}");
        assert_eq!(offset(&diagnostic), source.find('}').unwrap(), "{source:?}");
    }
}

#[test]
fn punctuation_inside_a_comment_after_a_token_leaves_that_token_the_last_one() {
    for punctuation in [";", ":", ",", "(", "["] {
        let source = format!("@media screen and /* {punctuation} */");
        let diagnostics = sheet(&source);
        let query = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidMediaQuery)
            .unwrap_or_else(|| panic!("{source:?}: {diagnostics:?}"));
        assert_eq!(
            encountered(query),
            Some(("Ident".to_owned(), "and".to_owned())),
            "{source:?}"
        );
        assert_eq!(offset(query), source.find("and").unwrap(), "{source:?}");
    }
}

#[test]
fn a_semicolon_inside_a_comment_before_the_end_of_a_prelude_is_trivia() {
    let source = "@import /* ; */ ;";
    let [diagnostic] = sheet(source).try_into().unwrap();
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
    assert_eq!(encountered(&diagnostic), None);
    assert_eq!(offset(&diagnostic), source.rfind(';').unwrap());
}

/// A string is one token, so the text of the token that a diagnostic names is
/// never a fragment of a string.
#[test]
fn a_string_holding_punctuation_is_never_named_by_a_fragment() {
    let parser = CssParser::default();
    let context = CssBlockContext::default();
    for punctuation in ["[", ";", "a,b"] {
        let source = format!("{{@media '{punctuation}' {{}}}}");
        let report = CssBlockFragment::<CssPageBody>::parse(&parser, &source, &context);
        let [diagnostic] = report.diagnostics() else {
            panic!("{source:?}: {report:?}");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidAtRuleBody);
        assert_eq!(encountered(diagnostic), None, "{source:?}");
    }
}

/// The unit ends at the end of the input when its `;` is an escaped one, is in
/// an unclosed block, or is in an unterminated string: the same text without
/// those reports the same end.
#[test]
fn a_semicolon_that_is_no_token_of_the_list_does_not_end_the_at_rule() {
    let plain = sheet("@media x");
    let last = |diagnostics: &[CssRecoveryDiagnostic]| {
        diagnostics
            .last()
            .map(|diagnostic| diagnostic.error().code())
    };
    assert_eq!(last(&plain), Some(CssErrorCode::UnexpectedEnd));
    for source in ["@media \\;", "@media ( ;", "@media [ ;", "@media 'a;"] {
        let diagnostics = sheet(source);
        assert_eq!(
            last(&diagnostics),
            Some(CssErrorCode::UnexpectedEnd),
            "{source:?}"
        );
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.error().code() != CssErrorCode::InvalidAtRuleBody),
            "{source:?}: {diagnostics:?}"
        );
    }
}

#[test]
fn a_comment_between_a_prelude_and_its_semicolon_hides_nothing() {
    let with_comment = sheet("@counter-style x /* c */ ;");
    let without = sheet("@counter-style x ;");
    let summary = |diagnostics: &[CssRecoveryDiagnostic]| -> Vec<_> {
        diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.error().code(),
                    encountered(diagnostic),
                    offset(diagnostic),
                    diagnostic.action(),
                )
            })
            .collect()
    };
    assert_eq!(summary(&with_comment), summary(&without));
    assert_eq!(summary(&with_comment)[0].0, CssErrorCode::InvalidAtRuleBody);
}

/// A rule that follows a comment holding a comma is reported like any other
/// rule with an invalid selector.
#[test]
fn a_rule_after_a_comment_that_holds_a_comma_is_reported() {
    let source = "a{/* , */ { a: b }}";
    let [diagnostic] = sheet(source)
        .try_into()
        .unwrap_or_else(|diagnostics: Vec<_>| panic!("{source:?}: {diagnostics:?}"));
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 10);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 18);

    let source = "{/* , */ { a: b }}";
    let namespaces = CssNamespaceContext::default();
    let report = CssBlockFragment::<CssRuleList>::parse(
        &CssParser::default(),
        source,
        &CssBlockContext::group(&namespaces),
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("{source:?}: {report:?}");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 9);
}

/// A unit that begins with U+00A0, U+0085, U+2028 or U+3000 begins with
/// something other than whitespace, so an at-keyword after it does not begin
/// the unit: it is a qualified rule, as it is when the same text begins with
/// any other character. A newline or a tab before the at-keyword is whitespace.
#[test]
fn a_look_alike_of_whitespace_does_not_make_a_unit_an_at_rule() {
    for look_alike in ['\u{a0}', '\u{85}', '\u{2028}', '\u{3000}'] {
        for rest in ["@media x {}", "@media x;", "@x y;", "@import x;"] {
            let source = format!("{look_alike}{rest}");
            let diagnostics = sheet(&source);
            let [diagnostic] = diagnostics.as_slice() else {
                panic!("{source:?}: {diagnostics:?}");
            };
            assert_eq!(
                diagnostic.action(),
                CssRecoveryAction::DropQualifiedRule,
                "{source:?}"
            );
        }
    }
    for space in ["\n", "\t", " ", "\r\n"] {
        let source = format!("{space}@x y;");
        let diagnostics = sheet(&source);
        let [diagnostic] = diagnostics.as_slice() else {
            panic!("{source:?}: {diagnostics:?}");
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropAtRule,
            "{source:?}"
        );
    }
}

/// A value that a block of descriptors hands to its grammar is a scope of the
/// source: its diagnostics are qualified with the offsets of the whole source,
/// whatever precedes the scope (a multi-byte scalar included).
#[test]
fn a_font_source_list_member_that_fails_after_multi_byte_scalars_is_reported() {
    let parser = CssParser::default();
    for source in [
        "@font-face { src: url(x) \u{a0}bogus, url(y) }",
        "@font-face { font-family: \u{1f600}; src: bogus \u{a0}x, url(y), \u{1f600}(z) }",
    ] {
        let diagnostics = sheet(source);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropFontSourceListItem),
            "{source:?}: {diagnostics:?}"
        );
    }
    let text = "{a{b:c}src:url(x) format(icc)\r\n/**/\u{a0}}\nunicode-range:U+25!important/**/}";
    let report = CssBlockFragment::<CssFontFaceDescriptors>::parse(
        &parser,
        text,
        &CssBlockContext::default(),
    );
    assert!(!report.diagnostics().is_empty(), "{report:?}");
}

/// A `;` inside an unclosed block is a token of that block, so it does not end
/// the at-rule that holds the block: the rule's own prelude is judged, as it is
/// when the block is closed.
#[test]
fn a_semicolon_inside_an_unclosed_block_of_an_at_rule_does_not_end_the_rule() {
    let unclosed = sheet("@keyframes a.b{ a.b;");
    let closed = sheet("@keyframes a.b{ a.b;}");
    let first = |diagnostics: &[CssRecoveryDiagnostic]| {
        diagnostics.first().map(|diagnostic| {
            (
                diagnostic.error().code(),
                encountered(diagnostic),
                offset(diagnostic),
            )
        })
    };
    assert_eq!(
        first(&unclosed),
        Some((
            CssErrorCode::InvalidAtRulePrelude,
            Some(("Delim".to_owned(), ".".to_owned())),
            12
        ))
    );
    assert_eq!(first(&unclosed), first(&closed));
}

/// An at-rule that needs a block and meets the end of its enclosing block is
/// reported as it is at the end of a sheet: the `}` that ends the enclosing
/// block is no text that follows the at-rule's prelude.
#[test]
fn an_at_rule_that_needs_a_block_is_reported_alike_at_the_end_of_a_block_and_of_a_sheet() {
    let at_sheet_end = sheet("@page");
    let [expected] = at_sheet_end.as_slice() else {
        panic!("{at_sheet_end:?}");
    };
    assert_eq!(expected.error().code(), CssErrorCode::InvalidAtRuleBody);
    let namespaces = CssNamespaceContext::default();
    let report = CssBlockFragment::<CssRuleList>::parse(
        &CssParser::default(),
        "{@page}",
        &CssBlockContext::group(&namespaces),
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("{report:?}");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidAtRuleBody);
    assert_eq!(diagnostic.action(), expected.action());
    assert_eq!(encountered(diagnostic), encountered(expected));
    assert_eq!(diagnostic.span().start().byte_offset().value(), 1);
}

/// Only an at-keyword begins an at-rule (Syntax 3 sections 5.5.1 and 5.5.5): a
/// `@` that starts no at-keyword is a delimiter, so a unit it begins is a
/// qualified rule, in a sheet, a style block and a descriptor block alike.
#[test]
fn a_lone_at_sign_does_not_begin_an_at_rule() {
    let delimiter = |source: &str| sheet(source);
    for unit in ["@ x;", "@ x {}", "@;", "@{}"] {
        for source in [
            unit.to_owned(),
            format!("a{{ {unit} }}"),
            format!("@font-palette-values --p {{ {unit} }}"),
            format!("@color-profile --c {{ {unit} }}"),
            format!("@font-feature-values A {{ {unit} }}"),
        ] {
            let diagnostics = delimiter(&source);
            assert!(!diagnostics.is_empty(), "{source:?}");
            // The unit is the last thing reported; the descriptor blocks'
            // own rule may be reported before it.
            let last = diagnostics.last().unwrap();
            assert_ne!(
                last.action(),
                CssRecoveryAction::DropAtRule,
                "{source:?}: {diagnostics:?}"
            );
        }
    }
    // The unit is dropped as the qualified rule it is, for a delimiter other than `@` too.
    for unit in ["@ x;", "@ x {}"] {
        let other = unit.replacen('@', "%", 1);
        let (a, b) = (sheet(unit), sheet(&other));
        let summary = |diagnostics: &[CssRecoveryDiagnostic]| -> Vec<_> {
            diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.error().code(), diagnostic.action()))
                .collect()
        };
        assert_eq!(summary(&a), summary(&b), "{unit:?}");
        assert_eq!(
            a[0].action(),
            CssRecoveryAction::DropQualifiedRule,
            "{unit:?}"
        );
    }
    // An at-keyword still begins an at-rule.
    let [diagnostic] = sheet("@x y;").try_into().unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
}

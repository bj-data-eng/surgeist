//! Spec-cited expectations for public parser reports.
//!
//! Each case states one authored input and its complete expected report: the
//! retained rules and declarations in source order, and the recovery
//! diagnostics in source order. A retained declaration is an `Accept` of its
//! property, authored value and value span; a diagnostic is a `Reject` of its
//! code, recovery action, responsible offset, recovered span and responsible
//! token or name. A report is clean exactly when a case expects no `Reject`.
//! Expectations are handwritten from the cited specification text and the
//! authored input, never from parser output. Typed value checks that the row
//! shape cannot express run as per-case callbacks. Line and UTF-16 column
//! coordinates are derived from each expected byte offset and the input.

use std::ops::Range;

use surgeist_css::{
    CssDeclaration, CssErrorCode, CssImportance, CssKnownPropertyValueRef, CssPropertyNameRef,
    CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssSourcePosition, CssTokenKind,
    CssTokenSummary, ErrorKind, parse_sheet, parse_style_attribute,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Entry {
    Sheet,
    Style,
}

/// One retained structure, in report preorder.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Retained {
    /// A style rule, by the specified serialization of its selectors.
    Style(String),
    Keyframes,
    /// One keyframe block, by its CSSOM key text.
    Keyframe(String),
    Container,
    Accept(Accept),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Accept {
    property: String,
    css: String,
    /// The retained value region: everything after the colon up to the end of
    /// the declaration, or up to the `!` of its `!important` annotation.
    span: Range<usize>,
    importance: CssImportance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Reject {
    code: CssErrorCode,
    action: CssRecoveryAction,
    offset: usize,
    span: Range<usize>,
    subject: Subject,
}

/// The responsible token or name a diagnostic reports.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Subject {
    Name(String),
    Token(CssTokenKind, String),
    End,
}

struct Case {
    basis: &'static str,
    entry: Entry,
    input: String,
    retained: Vec<Retained>,
    rejected: Vec<Reject>,
    check: Option<fn(&Observed<'_>)>,
}

/// The parsed report a callback inspects.
struct Observed<'a> {
    input: &'a str,
    rules: &'a [CssRule],
    declarations: Vec<&'a CssDeclaration>,
}

fn case(basis: &'static str, entry: Entry, input: impl Into<String>) -> Case {
    Case {
        basis,
        entry,
        input: input.into(),
        retained: Vec::new(),
        rejected: Vec::new(),
        check: None,
    }
}

impl Case {
    fn retains(mut self, retained: impl IntoIterator<Item = Retained>) -> Self {
        self.retained.extend(retained);
        self
    }

    fn rejects(mut self, rejected: impl IntoIterator<Item = Reject>) -> Self {
        self.rejected.extend(rejected);
        self
    }

    fn checks(mut self, check: fn(&Observed<'_>)) -> Self {
        self.check = Some(check);
        self
    }
}

fn style(selectors: &str) -> Retained {
    Retained::Style(selectors.to_owned())
}

fn keyframe(key_text: &str) -> Retained {
    Retained::Keyframe(key_text.to_owned())
}

fn accept(property: &str, css: &str, span: Range<usize>) -> Retained {
    Retained::Accept(Accept {
        property: property.to_owned(),
        css: css.to_owned(),
        span,
        importance: CssImportance::Normal,
    })
}

fn accept_important(property: &str, css: &str, span: Range<usize>) -> Retained {
    Retained::Accept(Accept {
        property: property.to_owned(),
        css: css.to_owned(),
        span,
        importance: CssImportance::Important,
    })
}

fn reject(
    code: CssErrorCode,
    action: CssRecoveryAction,
    offset: usize,
    span: Range<usize>,
    subject: Subject,
) -> Reject {
    Reject {
        code,
        action,
        offset,
        span,
        subject,
    }
}

fn name(name: &str) -> Subject {
    Subject::Name(name.to_owned())
}

fn token(kind: CssTokenKind, authored: &str) -> Subject {
    Subject::Token(kind, authored.to_owned())
}

fn check_cases(cases: impl IntoIterator<Item = Case>) {
    for case in cases {
        check_case(&case);
    }
}

fn check_case(case: &Case) {
    let input = case.input.as_str();
    let label = format!("{} ({input:?})", case.basis);
    let mut retained = Vec::new();
    match case.entry {
        Entry::Sheet => {
            let report = parse_sheet(input);
            let mut declarations = Vec::new();
            for rule in report.syntax().rules() {
                observe_rule(rule, &mut retained, &mut declarations);
            }
            let strict = surgeist_css::validate_sheet(input);
            let validation = report.clone().into_validation_result();
            if report.is_clean() {
                assert_eq!(strict, Ok(report.syntax().clone()), "{label}: strict sheet");
                assert_eq!(validation, Ok(report.syntax().clone()), "{label}");
            } else {
                assert_eq!(
                    strict.expect_err("recovered sheet").diagnostics(),
                    report.diagnostics(),
                    "{label}: strict sheet"
                );
                assert_eq!(
                    validation.expect_err("recovered sheet").diagnostics(),
                    report.diagnostics(),
                    "{label}"
                );
            }
            let observed = Observed {
                input,
                rules: report.syntax().rules(),
                declarations,
            };
            assert_report(
                case,
                &label,
                report.is_clean(),
                report.diagnostics(),
                &retained,
                &observed,
            );
        }
        Entry::Style => {
            let report = parse_style_attribute(input);
            let mut declarations = Vec::new();
            for declaration in report.syntax().iter() {
                retained.push(Retained::Accept(observe_declaration(declaration)));
                declarations.push(declaration);
            }
            let strict = surgeist_css::validate_style_attribute(input);
            let validation = report.clone().into_validation_result();
            if report.is_clean() {
                assert_eq!(strict, Ok(report.syntax().clone()), "{label}: strict style");
                assert_eq!(validation, Ok(report.syntax().clone()), "{label}");
            } else {
                assert_eq!(
                    strict.expect_err("recovered style").diagnostics(),
                    report.diagnostics(),
                    "{label}: strict style"
                );
                assert_eq!(
                    validation.expect_err("recovered style").diagnostics(),
                    report.diagnostics(),
                    "{label}"
                );
            }
            let observed = Observed {
                input,
                rules: &[],
                declarations,
            };
            assert_report(
                case,
                &label,
                report.is_clean(),
                report.diagnostics(),
                &retained,
                &observed,
            );
        }
    }
}

fn assert_report(
    case: &Case,
    label: &str,
    clean: bool,
    diagnostics: &[CssRecoveryDiagnostic],
    retained: &[Retained],
    observed: &Observed<'_>,
) {
    assert_eq!(retained, case.retained, "{label}: retained structure");
    let rejected = diagnostics
        .iter()
        .map(observe_diagnostic)
        .collect::<Vec<_>>();
    assert_eq!(rejected, case.rejected, "{label}: diagnostics");
    assert_eq!(clean, case.rejected.is_empty(), "{label}: clean report");
    for diagnostic in diagnostics {
        assert_coordinate(diagnostic.error().position(), observed.input, label);
        assert_coordinate(diagnostic.span().start(), observed.input, label);
        assert_coordinate(diagnostic.span().end(), observed.input, label);
    }
    assert_declarations(label, observed.input, &observed.declarations);
    if let Some(check) = case.check {
        check(observed);
    }
}

fn observe_rule<'a>(
    rule: &'a CssRule,
    retained: &mut Vec<Retained>,
    declarations: &mut Vec<&'a CssDeclaration>,
) {
    match rule {
        CssRule::Style(rule) => {
            let selectors = rule
                .selectors()
                .selectors()
                .iter()
                .map(|selector| selector.selector().to_specified_css().unwrap())
                .collect::<Vec<_>>();
            retained.push(Retained::Style(selectors.join(", ")));
            for declaration in rule.declarations().iter() {
                retained.push(Retained::Accept(observe_declaration(declaration)));
                declarations.push(declaration);
            }
            for child in rule.rules() {
                observe_rule(child, retained, declarations);
            }
        }
        CssRule::Keyframes(rule) => {
            retained.push(Retained::Keyframes);
            for block in rule.blocks() {
                retained.push(Retained::Keyframe(
                    block.selectors().serialize_key_text().unwrap(),
                ));
                for declaration in block.declarations().iter() {
                    let declaration = declaration.source();
                    retained.push(Retained::Accept(observe_declaration(declaration)));
                    declarations.push(declaration);
                }
            }
        }
        CssRule::Container(rule) => {
            retained.push(Retained::Container);
            for child in rule.rules() {
                observe_rule(child, retained, declarations);
            }
        }
        other => panic!("no expectation row shape for retained rule {other:?}"),
    }
}

fn observe_declaration(declaration: &CssDeclaration) -> Accept {
    let (property, css) = match declaration.property_name() {
        CssPropertyNameRef::Known(property) => (
            property.canonical_name().to_owned(),
            known_css(
                declaration
                    .known()
                    .and_then(|known| known.property_value())
                    .expect("typed known property value"),
            ),
        ),
        CssPropertyNameRef::Custom(name) => (
            name.as_str().to_owned(),
            declaration
                .custom()
                .and_then(|custom| custom.value().value())
                .expect("authored custom property value")
                .as_css()
                .to_owned(),
        ),
        other => panic!("no expectation row shape for property name {other:?}"),
    };
    let value = declaration
        .parsed_value()
        .expect("retained value provenance");
    Accept {
        property,
        css,
        span: value.span().start().byte_offset().value()..value.span().end().byte_offset().value(),
        importance: declaration.importance(),
    }
}

/// The authored value text of each property a case retains.
fn known_css(value: CssKnownPropertyValueRef<'_>) -> String {
    use CssKnownPropertyValueRef as Value;
    match value {
        Value::Color(value) => value.as_css(),
        Value::Opacity(value) => value.as_css(),
        Value::Height(value) => value.as_css(),
        Value::Display(value) => value.as_css(),
        Value::Overflow(value) => value.as_css(),
        Value::OverflowX(value) => value.as_css(),
        Value::OverflowY(value) => value.as_css(),
        Value::TextWrap(value) => value.as_css(),
        Value::TextAlignLast(value) => value.as_css(),
        Value::TextDecorationThickness(value) => value.as_css(),
        Value::Content(value) => value.as_css(),
        Value::AnimationName(value) => value.as_css(),
        Value::TransitionProperty(value) => value.as_css(),
        Value::Grid(value) => value.as_css(),
        other => panic!("no expectation row accessor for {other:?}"),
    }
    .to_owned()
}

/// Every retained name and value carries original source provenance.
fn assert_declarations(label: &str, input: &str, declarations: &[&CssDeclaration]) {
    for declaration in declarations {
        let name = declaration.parsed_name().expect("original name provenance");
        let value = declaration
            .parsed_value()
            .expect("original value provenance");
        assert_eq!(name.source().as_str(), input, "{label}: name source");
        assert!(name.source().same_snapshot(value.source()), "{label}");
        assert_eq!(declaration.position(), Some(name.span().start()), "{label}");
        let authored_name = &input
            [name.span().start().byte_offset().value()..name.span().end().byte_offset().value()];
        let property = match declaration.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name().to_owned(),
            CssPropertyNameRef::Custom(name) => name.as_str().to_owned(),
            other => panic!("{label}: unexpected property name {other:?}"),
        };
        assert!(
            authored_name.eq_ignore_ascii_case(&property),
            "{label}: authored name `{authored_name}` for `{property}`"
        );
        // The colon follows the name directly in every case input.
        assert_eq!(
            value.span().start().byte_offset().value(),
            name.span().end().byte_offset().value() + 1,
            "{label}: value region starts after the colon"
        );
        assert_coordinate(name.span().start(), input, label);
        assert_coordinate(value.span().end(), input, label);
    }
}

fn observe_diagnostic(diagnostic: &CssRecoveryDiagnostic) -> Reject {
    let error = diagnostic.error();
    Reject {
        code: error.code(),
        action: diagnostic.action(),
        offset: error.position().byte_offset().value(),
        span: diagnostic.span().start().byte_offset().value()
            ..diagnostic.span().end().byte_offset().value(),
        subject: subject(error.kind()),
    }
}

fn subject(kind: &ErrorKind) -> Subject {
    let token = |token: Option<&CssTokenSummary>| {
        token.map_or(Subject::End, |token| {
            Subject::Token(token.kind(), token.authored().to_owned())
        })
    };
    match kind {
        ErrorKind::UnexpectedEnd(_) => Subject::End,
        ErrorKind::UnexpectedToken(detail) => token(Some(detail.encountered())),
        ErrorKind::UnknownAtRule(detail) => Subject::Name(detail.name().as_str().to_owned()),
        ErrorKind::UnknownProperty(detail) => Subject::Name(detail.name().as_str().to_owned()),
        ErrorKind::InvalidAtRulePlacement(detail) => {
            Subject::Name(detail.name().as_str().to_owned())
        }
        ErrorKind::InvalidQualifiedRule(detail) => token(detail.encountered()),
        ErrorKind::InvalidSelector(detail) => token(detail.encountered()),
        ErrorKind::InvalidPropertyValue(detail) => token(detail.encountered()),
        other => panic!("no expectation row subject for {other:?}"),
    }
}

/// Lines count U+000A; columns count UTF-16 code units of the original input.
fn assert_coordinate(position: CssSourcePosition, input: &str, label: &str) {
    let byte = position.byte_offset().value();
    let before = &input[..byte];
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    assert_eq!(
        position.line().value() as usize,
        before.matches('\n').count(),
        "{label}: line of byte {byte}"
    );
    assert_eq!(
        position.column().value() as usize,
        before[line_start..].encode_utf16().count(),
        "{label}: UTF-16 column of byte {byte}"
    );
}

fn named_colors(observed: &Observed<'_>) {
    for declaration in &observed.declarations {
        let Some(CssKnownPropertyValueRef::Color(color)) =
            declaration.known().and_then(|known| known.property_value())
        else {
            continue;
        };
        assert_eq!(
            color.value().named().expect("named authored color").name(),
            color.as_css(),
            "{:?}: a named color stays its authored keyword",
            observed.input
        );
    }
}

// Syntax 3 (2026-10-01) §8.3: the encoding declaration only looks like an
// at-rule; no @charset rule exists, so in string input it is an unknown
// at-rule dropped through its semicolon, block or EOF (§5.5.2). Following
// rules are unaffected.
// https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#charset-rule
// https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-at-rule
#[test]
fn encoding_declarations_in_string_input_are_unknown_at_rules() {
    use CssErrorCode::UnknownAtRule;
    use CssRecoveryAction::DropAtRule;
    const BASIS: &str = "Syntax 3 §§5.5.2, 8.3";
    let after = |span: Range<usize>| [style(".after"), accept("color", "blue", span)];
    let charset = || name("charset");
    check_cases([
        case(
            BASIS,
            Entry::Sheet,
            "@charset \"UTF-8\"; .x { color: red; }",
        )
        .retains([style(".x"), accept("color", "red", 29..33)])
        .rejects([reject(UnknownAtRule, DropAtRule, 0, 0..17, charset())])
        .checks(named_colors),
        case(
            BASIS,
            Entry::Sheet,
            "@charset UTF-8; .after { color: blue; }",
        )
        .retains(after(31..36))
        .rejects([reject(UnknownAtRule, DropAtRule, 0, 0..15, charset())])
        .checks(named_colors),
        case(
            BASIS,
            Entry::Sheet,
            "@charset \"\"; .after { color: blue; }",
        )
        .retains(after(28..33))
        .rejects([reject(UnknownAtRule, DropAtRule, 0, 0..12, charset())])
        .checks(named_colors),
        case(
            BASIS,
            Entry::Sheet,
            "@charset \"UTF-8\" { ignored; } .after { color: blue; }",
        )
        .retains(after(45..50))
        .rejects([reject(UnknownAtRule, DropAtRule, 0, 0..29, charset())])
        .checks(named_colors),
        case(BASIS, Entry::Sheet, "@charset UTF-8;").rejects([reject(
            UnknownAtRule,
            DropAtRule,
            0,
            0..15,
            charset(),
        )]),
        case(BASIS, Entry::Sheet, "@charset \"\";").rejects([reject(
            UnknownAtRule,
            DropAtRule,
            0,
            0..12,
            charset(),
        )]),
        case(BASIS, Entry::Sheet, "@charset 'UTF-8';").rejects([reject(
            UnknownAtRule,
            DropAtRule,
            0,
            0..17,
            charset(),
        )]),
        case(BASIS, Entry::Sheet, "@charset /*comment*/ 'UTF-8';").rejects([reject(
            UnknownAtRule,
            DropAtRule,
            0,
            0..29,
            charset(),
        )]),
        case(BASIS, Entry::Sheet, "@charset \"UTF-8\"").rejects([reject(
            UnknownAtRule,
            DropAtRule,
            0,
            0..16,
            charset(),
        )]),
        case(BASIS, Entry::Sheet, "@charset \"UTF-8\" {}").rejects([reject(
            UnknownAtRule,
            DropAtRule,
            0,
            0..19,
            charset(),
        )]),
    ]);
}

// Syntax 3 (2026-10-01) §5.5.1 discards top-level <CDO-token> and <CDC-token>
// with no parse error; declarations inside the rules still recover normally.
// https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-stylesheet-contents
#[test]
fn top_level_cdo_and_cdc_tokens_are_discarded_without_diagnostics() {
    use CssErrorCode::{InvalidPropertyValue, UnknownProperty};
    use CssRecoveryAction::DropDeclaration;
    const BASIS: &str = "Syntax 3 §5.5.1";
    check_cases([
        case(
            BASIS,
            Entry::Sheet,
            "<!-- .x { mystery: 1; width: nope; } -->",
        )
        .retains([style(".x")])
        .rejects([
            reject(
                UnknownProperty,
                DropDeclaration,
                10,
                10..21,
                name("mystery"),
            ),
            reject(
                InvalidPropertyValue,
                DropDeclaration,
                29,
                22..34,
                token(CssTokenKind::Ident, "nope"),
            ),
        ]),
        case(BASIS, Entry::Sheet, "<!-- .x { color: red; }")
            .retains([style(".x"), accept("color", "red", 16..20)])
            .checks(named_colors),
        case(
            BASIS,
            Entry::Sheet,
            "<!-- .before { color: red; } <!-- .after { color: blue; }",
        )
        .retains([
            style(".before"),
            accept("color", "red", 21..25),
            style(".after"),
            accept("color", "blue", 49..54),
        ])
        .checks(named_colors),
        case(
            BASIS,
            Entry::Sheet,
            "--> .before { color: red; } --> .after { color: blue; }",
        )
        .retains([
            style(".before"),
            accept("color", "red", 20..24),
            style(".after"),
            accept("color", "blue", 47..52),
        ])
        .checks(named_colors),
        case(BASIS, Entry::Sheet, " \n/**/ <!-- --> \t"),
    ]);
}

// Syntax 3 (2026-10-01) §§5.5.1 and 5.5.3: at the top level a semicolon or an
// unmatched `}` starts a qualified rule's prelude rather than a dropped rule of
// its own. That prelude, including any at-keyword such as `@charset`, runs to
// the first {}-block or EOF; its invalid selector drops the whole unit, and
// EOF before any block is a parse error. NUL preprocesses to U+FFFD (§3.3)
// while coordinates stay in the original input.
// https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-qualified-rule
#[test]
fn top_level_semicolons_and_closers_open_qualified_rule_preludes() {
    use CssErrorCode::{InvalidQualifiedRule, InvalidSelector, UnknownAtRule};
    use CssRecoveryAction::{DropAtRule, DropQualifiedRule};
    const BASIS: &str = "Syntax 3 §§3.3, 5.5.1, 5.5.3";
    let semicolon = || token(CssTokenKind::Semicolon, ";");
    let closer = || token(CssTokenKind::CloseCurlyBracket, "}");
    let before = || [style(".before"), accept("color", "red", 16..20)];

    let repeated = format!("{}.after{{color:red}}", "@bad{};".repeat(256));
    assert_eq!(repeated.len(), 1809);
    // Only the first `@bad` dispatches as an at-rule; each later semicolon
    // owns the next `@bad{}` block, and the last one owns `.after{...}`.
    let repeated_rejects = std::iter::once(reject(UnknownAtRule, DropAtRule, 0, 0..6, name("bad")))
        .chain((0..256).map(|index| {
            let start = 6 + index * 7;
            let end = if index == 255 { 1809 } else { start + 7 };
            reject(
                InvalidSelector,
                DropQualifiedRule,
                start,
                start..end,
                semicolon(),
            )
        }));

    check_cases([
        case(BASIS, Entry::Sheet, ";;;;;}}}}\0\u{fffd}").rejects([reject(
            InvalidQualifiedRule,
            DropQualifiedRule,
            13,
            0..13,
            Subject::End,
        )]),
        case(BASIS, Entry::Sheet, repeated).rejects(repeated_rejects),
        case(BASIS, Entry::Sheet, "; .after { color: blue; }").rejects([reject(
            InvalidSelector,
            DropQualifiedRule,
            0,
            0..25,
            semicolon(),
        )]),
        case(
            BASIS,
            Entry::Sheet,
            ".before { color: red; } ; .after { color: blue; }",
        )
        .retains(before())
        .rejects([reject(
            InvalidSelector,
            DropQualifiedRule,
            24,
            24..49,
            semicolon(),
        )])
        .checks(named_colors),
        case(BASIS, Entry::Sheet, "} .after { color: blue; }").rejects([reject(
            InvalidSelector,
            DropQualifiedRule,
            0,
            0..25,
            closer(),
        )]),
        case(
            BASIS,
            Entry::Sheet,
            ".before { color: red; } } .after { color: blue; }",
        )
        .retains(before())
        .rejects([reject(
            InvalidSelector,
            DropQualifiedRule,
            24,
            24..49,
            closer(),
        )])
        .checks(named_colors),
        case(
            BASIS,
            Entry::Sheet,
            "; @charset \"UTF-8\"; .after { color: blue; }",
        )
        .rejects([reject(
            InvalidSelector,
            DropQualifiedRule,
            0,
            0..43,
            semicolon(),
        )]),
        case(
            BASIS,
            Entry::Sheet,
            "} @charset \"UTF-8\"; .after { color: blue; }",
        )
        .rejects([reject(
            InvalidSelector,
            DropQualifiedRule,
            0,
            0..43,
            closer(),
        )]),
    ]);
}

// CSS Style Attributes (REC 2013-11-07) §3: a `}` in a style attribute does
// not end the declaration list; it is an invalid token. A unit that starts
// with neither an identifier nor an at-keyword is a malformed declaration
// (CSS 2.1 §4.2) whose remnants run to the next top-level semicolon (Syntax 3
// (2026-10-01) §5.5.6), and a style attribute admits no at-rules or nested
// rules. Declarations between the dropped units are retained.
// https://www.w3.org/TR/2013/REC-css-style-attr-20131107/#syntax
// https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-the-remnants-of-a-bad-declaration
#[test]
fn style_attribute_non_declaration_units_drop_through_the_next_semicolon() {
    use CssErrorCode::{UnexpectedEnd, UnexpectedToken, UnknownAtRule};
    use CssRecoveryAction::DropDeclaration;
    const BASIS: &str = "Style Attributes §3; CSS 2.1 §4.2; Syntax 3 §5.5.6";
    check_cases([
        case(
            BASIS,
            Entry::Style,
            "@unknown screen { color: red; } } color: red; ) width: 2px; ] height: 3px;",
        )
        .rejects([
            reject(UnknownAtRule, DropDeclaration, 0, 0..31, name("unknown")),
            reject(
                UnexpectedToken,
                DropDeclaration,
                32,
                32..44,
                token(CssTokenKind::CloseCurlyBracket, "}"),
            ),
            reject(
                UnexpectedToken,
                DropDeclaration,
                46,
                46..58,
                token(CssTokenKind::CloseParenthesis, ")"),
            ),
            reject(
                UnexpectedToken,
                DropDeclaration,
                60,
                60..73,
                token(CssTokenKind::CloseSquareBracket, "]"),
            ),
        ]),
        case(
            BASIS,
            Entry::Style,
            "@unknown x; color: red; .nested { width: 1px; } opacity: 1; broken; height: 2px; ,; --kept: yes;",
        )
        .retains([
            accept("color", "red", 18..22),
            accept("height", "2px", 75..79),
            accept("--kept", "yes", 91..95),
        ])
        .rejects([
            reject(UnknownAtRule, DropDeclaration, 0, 0..11, name("unknown")),
            reject(
                UnexpectedToken,
                DropDeclaration,
                24,
                24..58,
                token(CssTokenKind::Delim, "."),
            ),
            reject(UnexpectedEnd, DropDeclaration, 66, 60..67, Subject::End),
            reject(
                UnexpectedToken,
                DropDeclaration,
                81,
                81..82,
                token(CssTokenKind::Comma, ","),
            ),
        ])
        .checks(named_colors),
    ]);
}

// Selectors 4 (2026-01-22) §16 and Nesting 1 (2026-01-22) §§3.1, 4: an explicit
// nesting selector makes `& || .title` an ordinary complex selector with a
// column combinator, not a leading relative selector, so the nested rule and
// its declaration are retained.
// https://www.w3.org/TR/2026/WD-selectors-4-20260122/#grammar
// https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting
#[test]
fn column_combinator_after_an_explicit_nesting_selector_is_retained() {
    check_cases([case(
        "Selectors 4 §16; Nesting 1 §§3.1, 4",
        Entry::Sheet,
        ".card { & || .title { color: red; } }",
    )
    .retains([
        style(".card"),
        style("& || .title"),
        accept("color", "red", 28..32),
    ])
    .checks(column_selector_structure)]);
}

fn column_selector_structure(observed: &Observed<'_>) {
    use surgeist_css::{
        CssComponentValueRef, CssSelector, CssSelectorCombinator, CssStyleSelector, CssValueOrigin,
        CssValueTokenRef,
    };
    let [CssRule::Style(parent)] = observed.rules else {
        panic!("one retained outer style rule")
    };
    assert_column_position(parent.position(), 0);
    assert!(parent.declarations().is_empty());
    let [CssStyleSelector::Selector(parent_selector)] = parent.selectors().selectors() else {
        panic!("one ordinary parent selector")
    };
    assert_eq!(parent_selector, &CssSelector::Class("card".into()));
    let [CssRule::Style(child)] = parent.rules() else {
        panic!("one retained nested style rule")
    };
    assert_column_position(child.position(), 8);
    assert!(child.rules().is_empty());
    let [CssStyleSelector::Selector(CssSelector::Complex(selector))] =
        child.selectors().selectors()
    else {
        panic!("explicit nesting anchor and Column form one ordinary complex selector")
    };
    assert_column_compound(selector.first(), 1, &[]);
    let [part] = selector.rest() else {
        panic!("one authored relation")
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Column);
    assert_column_compound(part.selector(), 0, &["title"]);

    let [declaration] = child.declarations().as_slice() else {
        panic!("one retained declaration")
    };
    assert!(declaration.custom().is_none());
    let known = declaration.known().expect("known color declaration");
    assert_eq!(known.property(), surgeist_css::CssKnownProperty::Color);
    let Some(CssKnownPropertyValueRef::Color(value)) = known.property_value() else {
        panic!("typed color value")
    };
    assert_eq!(value.value().named().expect("named color").name(), "red");
    assert_eq!(declaration.to_specified_css().unwrap(), "color: red;");
    let name = declaration.parsed_name().expect("original name token");
    let value = declaration.parsed_value().expect("original value region");
    assert_column_position(name.span().start(), 22);
    assert_column_position(name.span().end(), 27);
    assert_column_position(value.span().start(), 28);
    assert_column_position(value.span().end(), 32);
    let [space, red] = declaration.value_components().items() else {
        panic!("original whitespace and identifier tokens")
    };
    assert!(matches!(
        space.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(" "))
    ));
    assert!(matches!(
        red.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("red"))
    ));
    for (component, start, end) in [(space, 28, 29), (red, 29, 32)] {
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("parsed token origin")
        };
        assert_eq!(origin.source().as_str(), observed.input);
        assert_column_position(origin.span().start(), start);
        assert_column_position(origin.span().end(), end);
        assert!(name.source().same_snapshot(origin.source()));
    }
    let cloned = observed.rules.to_vec();
    let [CssRule::Style(cloned_parent)] = cloned.as_slice() else {
        panic!("cloned parent")
    };
    let [CssRule::Style(cloned_child)] = cloned_parent.rules() else {
        panic!("cloned child")
    };
    assert!(declaration.same_occurrence(&cloned_child.declarations()[0]));
}

fn assert_column_compound(
    compound: &surgeist_css::CssCompoundSelector,
    nesting_selectors: usize,
    classes: &[&str],
) {
    assert_eq!(compound.nesting_selectors(), nesting_selectors);
    assert_eq!(compound.scope_anchors(), 0);
    assert!(compound.type_selector().is_none());
    assert!(compound.ids().is_empty());
    assert_eq!(compound.classes(), classes);
    assert!(compound.attributes().is_empty());
    assert!(compound.pseudo_classes().is_empty());
    assert!(compound.pseudo_elements().is_none());
}

fn assert_column_position(position: CssSourcePosition, byte: usize) {
    assert_eq!(position.byte_offset().value(), byte);
    assert_eq!(position.line().value(), 0);
    assert_eq!(position.column().value() as usize, byte);
}

// Each value below is admitted by its property's current grammar:
// Text 4 §5.5 `text-wrap: auto` (the style longhand); Values 4 §4.2 reserves
// only CSS-wide keywords and `default`, so `auto` is a <custom-ident> name in
// Animations 1 §3 and Transitions 1 §2.1; Content 3 `contents`; Text
// Decoration 4 §2.4 signed <length>; Text 4 §7.4 `match-parent`; Overflow 3
// §3.1 `auto`; Display 3 §2 `inline` (inline flow).
// https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-text-wrap
// https://www.w3.org/TR/2024/WD-css-values-4-20240312/#custom-idents
// https://www.w3.org/TR/2023/WD-css-animations-1-20230302/#animation-name
// https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/#transition-property-property
// https://www.w3.org/TR/2025/WD-css-content-3-20251204/#valdef-content-contents
// https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/#text-decoration-thickness-property
// https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-text-align-last
// https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/#overflow-control
// https://www.w3.org/TR/2026/CRD-css-display-3-20260605/#the-display-properties
#[test]
fn property_values_admitted_by_current_grammars_are_retained() {
    check_cases([
        case("Text 4 §5.5", Entry::Style, "text-wrap: auto")
            .retains([accept("text-wrap", "auto", 10..15)])
            .checks(text_wrap_style_auto),
        case(
            "Values 4 §4.2; Animations 1 §3",
            Entry::Style,
            "animation-name: auto",
        )
        .retains([accept("animation-name", "auto", 15..20)])
        .checks(auto_timing_name),
        case(
            "Values 4 §4.2; Transitions 1 §2.1",
            Entry::Style,
            "transition-property: auto",
        )
        .retains([accept("transition-property", "auto", 20..25)])
        .checks(auto_timing_name),
        case("Content 3 contents", Entry::Style, "content: contents")
            .retains([accept("content", "contents", 8..17)])
            .checks(content_contents),
        case(
            "Text Decoration 4 §2.4",
            Entry::Style,
            "text-decoration-thickness: -1px",
        )
        .retains([accept("text-decoration-thickness", "-1px", 26..31)])
        .checks(signed_thickness),
        case("Text 4 §7.4", Entry::Style, "text-align-last: match-parent")
            .retains([accept("text-align-last", "match-parent", 16..29)])
            .checks(last_line_match_parent),
        case("Overflow 3 §3.1", Entry::Style, "overflow-x: auto")
            .retains([accept("overflow-x", "auto", 11..16)])
            .checks(overflow_auto),
        case("Overflow 3 §3.1", Entry::Style, "overflow-y: auto")
            .retains([accept("overflow-y", "auto", 11..16)])
            .checks(overflow_auto),
        case("Overflow 3 §3.1", Entry::Style, "overflow: auto")
            .retains([accept("overflow", "auto", 9..14)])
            .checks(overflow_auto),
        case("Display 3 §2", Entry::Style, "display: inline")
            .retains([accept("display", "inline", 8..15)])
            .checks(inline_flow_display),
    ]);
}

fn sole_value<'a>(observed: &'a Observed<'_>) -> CssKnownPropertyValueRef<'a> {
    let [declaration] = observed.declarations.as_slice() else {
        panic!("{:?}: one retained declaration", observed.input)
    };
    declaration
        .known()
        .and_then(|known| known.property_value())
        .expect("typed known value")
}

fn text_wrap_style_auto(observed: &Observed<'_>) {
    let CssKnownPropertyValueRef::TextWrap(value) = sole_value(observed) else {
        panic!("typed text-wrap value")
    };
    assert_eq!(value.value().mode(), None);
    assert_eq!(
        value.value().style(),
        Some(surgeist_css::CssTextWrapStyle::Auto)
    );
}

fn auto_timing_name(observed: &Observed<'_>) {
    let declaration = observed.declarations[0];
    match sole_value(observed) {
        CssKnownPropertyValueRef::AnimationName(value) => {
            assert!(matches!(
                value.names().names(),
                [surgeist_css::CssAnimationName::Custom(name)] if name.as_str() == "auto"
            ));
            assert_eq!(value.names().serialize_specified().unwrap(), "auto");
        }
        CssKnownPropertyValueRef::TransitionProperty(value) => {
            assert!(matches!(
                value.properties().properties(),
                [surgeist_css::CssTransitionProperty::Custom(name)] if name.as_str() == "auto"
            ));
            assert_eq!(value.properties().serialize_specified().unwrap(), "auto");
        }
        other => panic!("unexpected timing name value: {other:?}"),
    }
    let surgeist_css::CssValueOrigin::Parsed(token) = declaration
        .value_components()
        .items()
        .last()
        .unwrap()
        .origin()
    else {
        panic!("parsed name token provenance");
    };
    let end = observed.input.len();
    assert_eq!(token.source().as_str(), observed.input);
    assert_eq!(
        token.span().start().byte_offset().value(),
        end - "auto".len()
    );
    assert_eq!(token.span().end().byte_offset().value(), end);
    let canonical = format!("{};", observed.input);
    assert_eq!(declaration.to_specified_css().unwrap(), canonical);
    assert!(declaration.same_occurrence(&declaration.clone()));
    let reparsed = parse_style_attribute(&canonical);
    assert!(reparsed.is_clean());
    assert_eq!(reparsed.syntax()[0].body(), declaration.body());
    assert_eq!(reparsed.syntax()[0].to_specified_css().unwrap(), canonical);
}

fn content_contents(observed: &Observed<'_>) {
    let CssKnownPropertyValueRef::Content(value) = sole_value(observed) else {
        panic!("checked content wrapper")
    };
    let surgeist_css::CssContentValue::Generated(generated) = value.value() else {
        panic!("generated contents")
    };
    assert!(matches!(
        generated.items(),
        [surgeist_css::CssContentValueItem::Contents]
    ));
    assert!(matches!(
        generated.body(),
        surgeist_css::CssGeneratedContentBodyRef::List([
            surgeist_css::CssContentValueItem::Contents
        ])
    ));
    assert_eq!(value.value().serialize_specified().unwrap(), "contents");
    assert!(matches!(
        observed.declarations[0].value_components().items()[0].origin(),
        surgeist_css::CssValueOrigin::Parsed(_)
    ));
}

fn signed_thickness(observed: &Observed<'_>) {
    let CssKnownPropertyValueRef::TextDecorationThickness(value) = sole_value(observed) else {
        panic!("typed signed thickness")
    };
    let surgeist_css::CssTextDecorationThickness::Length(length) = value.value() else {
        panic!("signed length branch")
    };
    let component = length.literal_component().expect("exact literal");
    assert!(matches!(
        component.view(),
        surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension {
            number,
            unit: "px",
        }) if number.representation() == "-1"
    ));
    assert!(matches!(
        component.origin(),
        surgeist_css::CssValueOrigin::Parsed(_)
    ));
}

fn last_line_match_parent(observed: &Observed<'_>) {
    let CssKnownPropertyValueRef::TextAlignLast(value) = sole_value(observed) else {
        panic!("current last-line value")
    };
    assert_eq!(
        value.value(),
        &surgeist_css::CssTextAlignLastValue::Keyword(surgeist_css::CssTextAlign::MatchParent)
    );
}

fn overflow_auto(observed: &Observed<'_>) {
    use surgeist_css::CssOverflow;
    match sole_value(observed) {
        CssKnownPropertyValueRef::Overflow(value) => {
            assert_eq!(value.value().x(), CssOverflow::Auto);
            assert_eq!(value.value().authored_y(), None);
            assert_eq!(value.value().y(), CssOverflow::Auto);
        }
        CssKnownPropertyValueRef::OverflowX(value) => {
            assert_eq!(*value.value(), CssOverflow::Auto)
        }
        CssKnownPropertyValueRef::OverflowY(value) => {
            assert_eq!(*value.value(), CssOverflow::Auto)
        }
        other => panic!("wrong typed overflow value: {other:?}"),
    }
}

fn inline_flow_display(observed: &Observed<'_>) {
    let CssKnownPropertyValueRef::Display(value) = sole_value(observed) else {
        panic!("typed display")
    };
    assert_eq!(
        *value.value(),
        surgeist_css::CssDisplayValue::OutsideInside {
            outside: surgeist_css::CssDisplayOutside::Inline,
            inside: surgeist_css::CssDisplayInside::Flow,
        }
    );
}

// Grid 3 relaxes the auto-repeat body to general track sizes, so
// `repeat(auto-fit, 1fr)` is a valid explicit column axis of the `grid`
// shorthand's `auto-flow` form, in either case of the property name and with
// or without `!important`.
// https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#intrinsic-auto-repeat
#[test]
fn intrinsic_auto_repeat_in_the_grid_shorthand_is_retained() {
    const BASIS: &str = "Grid 3 intrinsic auto repeat";
    const VALUE: &str = "auto-flow dense 12px / repeat(auto-fit, 1fr)";
    check_cases([
        case(
            BASIS,
            Entry::Style,
            "grid: auto-flow dense 12px / repeat(auto-fit, 1fr)",
        )
        .retains([accept("grid", VALUE, 5..50)])
        .checks(auto_fit_grid),
        case(
            BASIS,
            Entry::Sheet,
            ".test { GRID: auto-flow dense 12px / repeat(auto-fit, 1fr) !important; }",
        )
        .retains([style(".test"), accept_important("grid", VALUE, 13..59)])
        .checks(auto_fit_grid),
        case(
            BASIS,
            Entry::Sheet,
            ".test { GRID: auto-flow dense 12px / repeat(auto-fit, 1fr); }",
        )
        .retains([style(".test"), accept("grid", VALUE, 13..58)])
        .checks(auto_fit_grid),
    ]);
}

fn auto_fit_grid(observed: &Observed<'_>) {
    use surgeist_css::{
        CssGridAutoFlowAxis, CssGridAutoRepeatKind, CssGridAutoTrackComponent,
        CssGridTrackBreadthKind, CssGridTrackRepeatComponent,
    };
    let CssKnownPropertyValueRef::Grid(value) = sole_value(observed) else {
        panic!("a complete current Grid value");
    };
    assert!(value.value().template_value().is_none());
    let flow = value.value().auto_flow().unwrap();
    assert_eq!(flow.axis(), CssGridAutoFlowAxis::Row);
    assert!(flow.dense());
    let [implicit] = value.value().auto_tracks().unwrap().sizes() else {
        panic!("one implicit track size");
    };
    assert_eq!(
        implicit
            .breadth()
            .unwrap()
            .length_percentage()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "12px"
    );
    let explicit = value.value().explicit_tracks().unwrap();
    assert!(explicit.general_list().is_none());
    let [CssGridAutoTrackComponent::AutoRepeat(repeat)] =
        explicit.auto_list().unwrap().components()
    else {
        panic!("one automatic repeat in the explicit column axis");
    };
    assert_eq!(repeat.kind(), CssGridAutoRepeatKind::AutoFit);
    let [CssGridTrackRepeatComponent::TrackSize(size)] = repeat.content().components() else {
        panic!("one nonrecursive general track size");
    };
    let breadth = size.breadth().unwrap();
    assert_eq!(breadth.kind(), CssGridTrackBreadthKind::Fraction);
    assert_eq!(
        breadth.flex().unwrap().serialize_specified().unwrap(),
        "1fr"
    );
}

// Fonts 4 §6.12: an <opentype-tag> is a <string> of exactly four printable
// ASCII characters, so the three-character `"abc"` is the token that makes
// the declaration invalid.
// https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-feature-settings-prop
#[test]
fn a_feature_tag_of_the_wrong_length_is_the_rejected_token() {
    check_cases([case(
        "Fonts 4 §6.12 <opentype-tag>",
        Entry::Style,
        "font-feature-settings: \"abc\" on",
    )
    .rejects([reject(
        CssErrorCode::InvalidPropertyValue,
        CssRecoveryAction::DropDeclaration,
        23,
        0..31,
        token(CssTokenKind::String, "\"abc\""),
    )])]);
}

// Syntax 3 (2026-10-01) §§5.5.5 and 7.1: an at-rule inside a keyframe's
// <declaration-list> is invalid and dropped alone, so the 25% keyframe keeps
// the declarations around it; an invalid keyframe selector drops only its own
// block. Animations 1 maps `from` and `to` to 0% and 100% key text.
// https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents
// https://www.w3.org/TR/2023/WD-css-animations-1-20230302/#interface-csskeyframerule
#[test]
fn an_at_rule_inside_a_keyframe_block_is_dropped_alone() {
    check_cases([case(
        "Syntax 3 §§5.5.5, 7.1; Animations 1 keyText",
        Entry::Sheet,
        "@keyframes fade { from { opacity: 0; } 25% { opacity: .25; @media fn(a, b) { width: 1px; } height: 2px; } fn(a, b) { opacity: .5; } to { opacity: 1; } }",
    )
    .retains([
        Retained::Keyframes,
        keyframe("0%"),
        accept("opacity", "0", 33..35),
        keyframe("25%"),
        accept("opacity", ".25", 53..57),
        accept("height", "2px", 98..102),
        keyframe("100%"),
        accept("opacity", "1", 145..147),
    ])
    .rejects([
        reject(
            CssErrorCode::InvalidAtRulePlacement,
            CssRecoveryAction::DropAtRule,
            59,
            59..90,
            name("media"),
        ),
        reject(
            CssErrorCode::UnexpectedToken,
            CssRecoveryAction::DropKeyframeBlock,
            106,
            106..131,
            token(CssTokenKind::Function, "fn("),
        ),
    ])]);
}

// Syntax 3 (2026-10-01) §§3.2 and 4.2: only decoding a byte stream removes a
// BOM, and U+FEFF is a non-ASCII ident code point, so string input that starts
// with U+FEFF begins a qualified rule's prelude. The following `@charset`
// at-keyword stays in that prelude and invalidates the whole rule through its
// block. Columns count U+FEFF as one UTF-16 unit.
// https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#non-ascii-ident-code-point
#[test]
fn a_leading_feff_in_string_input_starts_a_qualified_rule() {
    check_cases([case(
        "Syntax 3 §§3.2, 4.2, 5.5.3",
        Entry::Sheet,
        "\u{feff} /* leading */ @charset \"UTF-8\"; .after { color: blue; }",
    )
    .rejects([reject(
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropQualifiedRule,
        18,
        0..59,
        token(CssTokenKind::AtKeyword, "@charset"),
    )])]);
}

// Conditional 5: <query-in-parens> admits <general-enclosed>, `style()` takes
// a style query and `scroll-state()` a scroll-state query, so each container
// rule and its nested style rule are retained.
// https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule
#[test]
fn general_enclosed_style_and_scroll_state_container_queries_are_retained() {
    const BASIS: &str = "Conditional 5 container queries";
    let contents = |span: Range<usize>| {
        [
            Retained::Container,
            style(".x"),
            accept("color", "red", span),
        ]
    };
    check_cases([
        case(
            BASIS,
            Entry::Sheet,
            "@container scroll-state(stuck: top) { .x { color: red; } }",
        )
        .retains(contents(49..53))
        .checks(container_query),
        case(
            BASIS,
            Entry::Sheet,
            "@container style(color: red) { .x { color: red; } }",
        )
        .retains(contents(42..46))
        .checks(container_query),
        case(
            BASIS,
            Entry::Sheet,
            "@container (unknown-size > 1px) { .x { color: red; } }",
        )
        .retains(contents(45..49))
        .checks(container_query),
    ]);
}

fn container_query(observed: &Observed<'_>) {
    use surgeist_css::{
        CssContainerConditionKind, CssContainerScrollFeature, CssContainerScrollQueryKind,
        CssContainerStuckKeyword, CssContainerStuckValueRef,
    };
    named_colors(observed);
    let input = observed.input;
    let query = input["@container ".len()..input.find(" {").unwrap()].to_owned();
    let [CssRule::Container(container)] = observed.rules else {
        panic!("retained container")
    };
    let value = container.prelude().entries()[0].query().unwrap();
    match query.as_str() {
        "style(color: red)" => assert!(matches!(value.kind(), CssContainerConditionKind::Style(_))),
        "scroll-state(stuck: top)" => {
            let CssContainerConditionKind::ScrollState(scroll) = value.kind() else {
                panic!("scroll-state")
            };
            let CssContainerScrollQueryKind::Feature(CssContainerScrollFeature::Stuck(operand)) =
                scroll.kind()
            else {
                panic!("stuck feature")
            };
            assert!(matches!(
                operand.view(),
                CssContainerStuckValueRef::Keyword(CssContainerStuckKeyword::Top)
            ));
            assert_eq!(operand.serialize().unwrap().as_css(), "top");
        }
        _ => assert!(matches!(
            value.kind(),
            CssContainerConditionKind::GeneralEnclosed(_)
        )),
    }
    assert_eq!(value.serialize().unwrap().as_css(), format!("{query} "));
    let surgeist_css::CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("original query source")
    };
    assert_eq!(origin.source().as_str(), input);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        "@container ".len()
    );
    assert!(
        observed.declarations[0]
            .parsed_value()
            .unwrap()
            .source()
            .same_snapshot(origin.source())
    );
}

// CSSWG dropped `align-tracks` and `justify-tracks` on 2023-10-04, and the
// selected Grid 3 publication replaced `grid-flow-tolerance`; no selected
// specification defines them, so each is an unknown property whose
// declaration is dropped while its style rule and selector remain.
// https://github.com/w3c/csswg-drafts/issues/8207#issuecomment-1747805578
// https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#propdef-flow-tolerance
#[test]
fn removed_masonry_and_flow_tolerance_properties_are_unknown() {
    use CssErrorCode::UnknownProperty;
    use CssRecoveryAction::DropDeclaration;
    const BASIS: &str = "CSSWG #8207; Grid 3 flow-tolerance";
    let attribute = |input: &'static str, property: &str| {
        case(BASIS, Entry::Style, input).rejects([reject(
            UnknownProperty,
            DropDeclaration,
            0,
            0..input.len(),
            name(property),
        )])
    };
    let rule = |input: &'static str, property: &str, end: usize| {
        case(BASIS, Entry::Sheet, input)
            .retains([style(".test")])
            .rejects([reject(
                UnknownProperty,
                DropDeclaration,
                8,
                8..end,
                name(property),
            )])
    };
    check_cases([
        attribute("grid-flow-tolerance: solid", "grid-flow-tolerance"),
        attribute("grid-flow-tolerance: infinite", "grid-flow-tolerance"),
        rule(
            ".test { GRID-FLOW-TOLERANCE: infinite !important; }",
            "GRID-FLOW-TOLERANCE",
            49,
        ),
        rule(
            ".test { GRID-FLOW-TOLERANCE: infinite; }",
            "GRID-FLOW-TOLERANCE",
            38,
        ),
        attribute("align-tracks: auto", "align-tracks"),
        attribute("align-tracks: center", "align-tracks"),
        attribute("justify-tracks: auto", "justify-tracks"),
        attribute("justify-tracks: space-evenly", "justify-tracks"),
        rule(
            ".test { ALIGN-TRACKS: center !important; }",
            "ALIGN-TRACKS",
            40,
        ),
        rule(".test { ALIGN-TRACKS: center; }", "ALIGN-TRACKS", 29),
        rule(
            ".test { JUSTIFY-TRACKS: space-evenly !important; }",
            "JUSTIFY-TRACKS",
            48,
        ),
        rule(
            ".test { JUSTIFY-TRACKS: space-evenly; }",
            "JUSTIFY-TRACKS",
            37,
        ),
    ]);
}

#[test]
fn removed_track_declarations_preserve_adjacent_current_alignment_and_custom_semantics() {
    let input = "color: red; align-tracks: center; align-content: first baseline !important; justify-tracks: var(--old); --align-tracks: center";
    let report = parse_style_attribute(input);
    assert!(!report.is_clean());
    assert_eq!(report.diagnostics().len(), 2);
    for (diagnostic, name) in report
        .diagnostics()
        .iter()
        .zip(["align-tracks", "justify-tracks"])
    {
        let ErrorKind::UnknownProperty(detail) = diagnostic.error().kind() else {
            panic!("unknown obsolete property")
        };
        assert_eq!(detail.name().as_str(), name);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            input.find(name).unwrap()
        );
    }
    let [color, alignment, custom] = report.syntax().as_slice() else {
        panic!("three adjacent declarations remain")
    };
    assert_eq!(
        color.known().unwrap().property(),
        surgeist_css::CssKnownProperty::Color
    );
    let Some(CssKnownPropertyValueRef::Color(value)) = color.known().unwrap().property_value()
    else {
        panic!("adjacent color")
    };
    assert_eq!(value.as_css(), "red");
    assert_eq!(alignment.importance(), CssImportance::Important);
    let Some(CssKnownPropertyValueRef::AlignContent(value)) =
        alignment.known().unwrap().property_value()
    else {
        panic!("current alignment")
    };
    assert_eq!(
        value.value().value(),
        surgeist_css::CssAlignmentValue::Baseline(surgeist_css::CssBaselinePosition::First)
    );
    assert_eq!(value.value().serialize_specified().unwrap(), "baseline");
    let custom = custom.custom().unwrap();
    assert_eq!(custom.name().as_str(), "--align-tracks");
    assert_eq!(custom.value().value().unwrap().as_css(), "center");
}

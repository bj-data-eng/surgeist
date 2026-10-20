//! Structural comparison against upstream CSSTree canonical output.
//!
//! CSSTree's generator test (`lib/__tests/generate.js` at the pinned revision)
//! requires `generate(parse(source))` to equal the fixture's `generate` field,
//! or the `source` itself when that field is absent. The neutral corpus copies
//! that field into `canonical_css`; an absent field therefore states that the
//! upstream canonical form equals the input. This upstream text is independent
//! of Surgeist, unlike the parser-captured oracle.
//!
//! Every upstream-parsed case with a public Surgeist projection is serialized
//! from the retained parse result, never from the input text. Both sides then
//! pass through one documented [`normalize`] step that removes only differences
//! between two canonical generators. Combinators, retained declarations, values
//! and rule structure remain significant. Mismatches must be listed in
//! [`EXCEPTIONS`] with their kind, reason and pinned expected Surgeist form;
//! an excepted case whose serialization differs from its pinned form fails, as
//! does a stale exception.

use std::collections::{BTreeMap, BTreeSet};

use cssparser::{Parser, ParserInput, Token as CssToken};

use super::{
    Adapter, CssRecoveryActionName, EntryPoint, ExpectedClass, ExpectedUnsupportedPolicy,
    PropertyOrDescriptor, REGISTRY, SyntaxCount, UpstreamOutcome, ValidatedCase,
    corpus_namespace_context, load_neutral_csstree_inventory, resolve_case_registry,
};
use surgeist_css::{
    CssImportance, CssMediaSerializationError, CssPropertyNameRef, CssStyleSelector,
    parse_declaration, parse_font_face_descriptor_value, parse_media_query, parse_media_query_list,
    parse_property_value_text, parse_rule, parse_selector, parse_selector_list, parse_sheet,
    parse_style_attribute, parse_style_block,
};

/// Why a compared case may differ from its upstream canonical form.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalExceptionKind {
    /// Surgeist intentionally differs under a cited specification.
    Deliberate,
    /// A known Surgeist defect; the product fix belongs to separate work.
    Defect,
}

impl CanonicalExceptionKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Deliberate => "deliberate",
            Self::Defect => "defect",
        }
    }
}

/// One explicit per-case difference with its one-line reason and the
/// Surgeist form that reason requires.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CanonicalException {
    id: &'static str,
    kind: CanonicalExceptionKind,
    reason: &'static str,
    /// The expected Surgeist serialization in [`normalize`] rendering, or with
    /// the whitespace that rendering would lose between tokens. It is compared
    /// token by token with the actual serialization.
    surgeist: &'static str,
}

#[path = "canonical_exceptions.rs"]
mod exceptions;
pub(crate) use exceptions::EXCEPTIONS;

/// Upstream-parsed cases excluded without comparison, by category.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CanonicalExclusion {
    /// The CSS-owned expected class records an adapterless panic-freedom-only
    /// policy and the context has no truthful public entry point. Selector
    /// fixtures are the exception: they are projected through the public
    /// selector parser whatever their observation policy.
    NoPublicProjection,
    /// The public parser rejects the whole input and retains no syntax, and
    /// the CSS-owned expected class records that whole-input rejection and its
    /// basis: empty retained syntax with a `RejectInput` diagnostic, as a strict
    /// rejection or a fully observed unsupported disposition. Retaining nothing
    /// under any other expected class fails the comparison.
    NoRetainedSyntax,
}

impl CanonicalExclusion {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::NoPublicProjection => "no_public_projection",
            Self::NoRetainedSyntax => "no_retained_syntax",
        }
    }
}

/// Coverage of one complete comparison run.
#[derive(Debug, Default)]
pub(crate) struct CanonicalSummary {
    /// Context name to (matched, excepted) compared-case counts.
    compared: BTreeMap<&'static str, (usize, usize)>,
    /// Context and exclusion category to excluded-case count.
    excluded: BTreeMap<(&'static str, &'static str), usize>,
    /// Exception IDs by kind name.
    exceptions: BTreeMap<&'static str, Vec<&'static str>>,
}

impl CanonicalSummary {
    /// Context, matched and excepted counts of compared cases, by context.
    pub(crate) fn compared(&self) -> Vec<(&'static str, usize, usize)> {
        self.compared
            .iter()
            .map(|(context, (matched, excepted))| (*context, *matched, *excepted))
            .collect()
    }

    /// Context, exclusion category and count of uncompared cases.
    pub(crate) fn excluded(&self) -> Vec<(&'static str, &'static str, usize)> {
        self.excluded
            .iter()
            .map(|((context, category), count)| (*context, *category, *count))
            .collect()
    }

    /// Exercised exception IDs of one kind, in case order.
    pub(crate) fn exceptions(&self, kind: CanonicalExceptionKind) -> &[&'static str] {
        self.exceptions.get(kind.name()).map_or(&[], Vec::as_slice)
    }
}

/// Compares every upstream-parsed case and returns failures for unlisted
/// mismatches, stale or misplaced exceptions, and projection errors.
pub(crate) fn compare_csstree_canonical_forms() -> Result<CanonicalSummary, Vec<String>> {
    let inventory = load_neutral_csstree_inventory().map_err(|error| vec![error.to_owned()])?;
    let mut failures = Vec::new();
    let mut summary = CanonicalSummary::default();
    let exceptions = exception_index(&mut failures);
    let mut used = BTreeSet::new();
    let mut parsed_ids = BTreeSet::new();

    for case in &inventory.cases {
        if !matches!(case.upstream_outcome, UpstreamOutcome::Parsed) {
            continue;
        }
        parsed_ids.insert(case.id.as_str());
        let context = case.context.as_str();
        let projection = match project(case) {
            Ok(Projection::Excluded(category)) => {
                *summary
                    .excluded
                    .entry((context, category.name()))
                    .or_default() += 1;
                continue;
            }
            Ok(Projection::Compared { surgeist, upstream }) => (surgeist, upstream),
            Err(error) => {
                failures.push(format!("{}: projection failed: {error}", case.id));
                continue;
            }
        };
        let (serialized, upstream) = projection;
        let matches = canonically_equal(&serialized, &upstream);
        let surgeist = normalize(&serialized);
        let upstream = normalize(&upstream);
        let entry = summary.compared.entry(context).or_default();
        match (matches, exceptions.get(case.id.as_str())) {
            (true, None) => entry.0 += 1,
            (true, Some(listed)) => failures.push(format!(
                "{}: stale {} exception now matches upstream `{upstream}`",
                listed.id,
                listed.kind.name()
            )),
            (false, Some(listed)) if !canonically_equal(&serialized, listed.surgeist) => failures
                .push(format!(
                    "{}: surgeist `{surgeist}` != pinned {} form `{}` (upstream `{upstream}`, serialized `{serialized}`)",
                    listed.id,
                    listed.kind.name(),
                    listed.surgeist
                )),
            (false, Some(listed)) => {
                entry.1 += 1;
                used.insert(listed.id);
                summary
                    .exceptions
                    .entry(listed.kind.name())
                    .or_default()
                    .push(listed.id);
            }
            (false, None) => failures.push(format!(
                "{}: surgeist `{surgeist}` != upstream `{upstream}` (serialized `{serialized}`)",
                case.id
            )),
        }
    }
    for listed in EXCEPTIONS {
        let reported = format!("{}: ", listed.id);
        if !used.contains(listed.id) && !failures.iter().any(|f| f.starts_with(&reported)) {
            let location = if parsed_ids.contains(listed.id) {
                "is not a compared case"
            } else {
                "is not an upstream-parsed case"
            };
            failures.push(format!("{}: exception {location}", listed.id));
        }
    }
    if failures.is_empty() {
        Ok(summary)
    } else {
        Err(failures)
    }
}

fn exception_index(
    failures: &mut Vec<String>,
) -> BTreeMap<&'static str, &'static CanonicalException> {
    let mut index = BTreeMap::new();
    let mut previous: Option<&str> = None;
    for listed in EXCEPTIONS {
        if previous.is_some_and(|previous| previous >= listed.id) {
            failures.push(format!(
                "{}: exceptions are not sorted by unique ID",
                listed.id
            ));
        }
        if listed.reason.trim().is_empty() {
            failures.push(format!("{}: exception has no reason", listed.id));
        }
        // Pins are written as `normalize` renders them, so messages and pins
        // read alike and a pin cannot hide trivia the comparison ignores. A pin
        // differs from its rendering only where the rendering would tokenize
        // differently, such as `1%+1.5em`, where `+1.5em` is a signed number,
        // so a pin may keep only the whitespace or comment that separates
        // tokens.
        let rendered = normalize(listed.surgeist);
        if rendered != listed.surgeist && canonically_equal(&rendered, listed.surgeist) {
            failures.push(format!(
                "{}: pinned form `{}` is not in normalized rendering `{rendered}`",
                listed.id, listed.surgeist
            ));
        }
        previous = Some(listed.id);
        index.insert(listed.id, listed);
    }
    index
}

enum Projection {
    Excluded(CanonicalExclusion),
    Compared { surgeist: String, upstream: String },
}

fn project(case: &ValidatedCase) -> Result<Projection, String> {
    let expected = case
        .expected_class
        .as_ref()
        .ok_or("case has no validated expected class")?;
    let panic_freedom_only = matches!(
        expected,
        ExpectedClass::Unsupported {
            policy: ExpectedUnsupportedPolicy::PanicFreedomOnly,
            ..
        }
    );
    // The registry observes the combinator fixtures only for panic freedom,
    // but every selector fixture has a faithful public projection through
    // `parse_selector`; other panic-freedom-only contexts have none.
    let selector_projection = panic_freedom_only && case.context.as_str() == "selector";
    if panic_freedom_only && !selector_projection {
        return Ok(Projection::Excluded(CanonicalExclusion::NoPublicProjection));
    }
    let registry = REGISTRY
        .iter()
        .find(|entry| entry.fixture_path() == case.expectation_path)
        .copied()
        .ok_or("neutral expectation path has no registry entry")?;
    let registry = resolve_case_registry(case, registry)?;
    let input = case.input.as_str();
    let upstream = case.canonical_css.as_deref().unwrap_or(input);
    let namespaces = corpus_namespace_context();
    let mut upstream = upstream.to_owned();
    let entry_point = if selector_projection {
        EntryPoint::Selector
    } else {
        registry.entry_point()
    };
    let surgeist = match entry_point {
        EntryPoint::Sheet => Some(
            parse_sheet(input)
                .syntax()
                .to_specified_css()
                .map_err(|error| format!("sheet: {error:?}"))?,
        ),
        EntryPoint::Rule => parse_rule(input, &namespaces)
            .syntax()
            .as_ref()
            .map(|rule| rule.to_specified_css())
            .transpose()
            .map_err(|error| format!("rule: {error:?}"))?,
        EntryPoint::StyleBlock => match parse_style_block(input, &namespaces).syntax() {
            None => None,
            Some(block) => {
                let mut text = String::from("{");
                text.push_str(
                    &block
                        .declarations()
                        .to_specified_css()
                        .map_err(|error| format!("block declarations: {error:?}"))?,
                );
                for rule in block.rules() {
                    text.push_str(
                        &rule
                            .to_specified_css()
                            .map_err(|error| format!("block rule: {error:?}"))?,
                    );
                }
                text.push('}');
                Some(text)
            }
        },
        EntryPoint::StyleAttribute => Some(
            parse_style_attribute(input)
                .syntax()
                .to_specified_css()
                .map_err(|error| format!("declaration list: {error:?}"))?,
        ),
        EntryPoint::Declaration => parse_declaration(input)
            .syntax()
            .as_ref()
            .map(|declaration| declaration.to_specified_css())
            .transpose()
            .map_err(|error| format!("declaration: {error:?}"))?,
        EntryPoint::PropertyValueText => {
            let Some(PropertyOrDescriptor::Property(property)) = registry.property_or_descriptor()
            else {
                return Err("property value entry lacks its property".into());
            };
            // The upstream value is the declaration's value; supply the
            // adapter's property name so both sides describe one declaration.
            upstream = format!("{}:{upstream}", property.canonical_name());
            parse_property_value_text(
                input,
                CssPropertyNameRef::Known(property),
                CssImportance::Normal,
            )
            .syntax()
            .as_ref()
            .map(|declaration| declaration.to_specified_css())
            .transpose()
            .map_err(|error| format!("property value: {error:?}"))?
        }
        EntryPoint::FontFaceDescriptorValue => {
            let Some(PropertyOrDescriptor::FontFaceDescriptor(kind)) =
                registry.property_or_descriptor()
            else {
                return Err("descriptor value entry lacks its descriptor".into());
            };
            parse_font_face_descriptor_value(input, kind.css_kind())
                .syntax()
                .as_ref()
                .map(|value| value.serialize_specified())
                .transpose()
                .map_err(|error| format!("descriptor value: {error:?}"))?
        }
        EntryPoint::Selector => parse_selector(input, &namespaces)
            .syntax()
            .as_ref()
            .map(|selector| selector.to_specified_css())
            .transpose()
            .map_err(|error| format!("selector: {error:?}"))?,
        EntryPoint::SelectorList => match parse_selector_list(input, &namespaces).syntax() {
            None => None,
            Some(list) => {
                let mut members = Vec::with_capacity(list.selectors().len());
                for member in list.selectors() {
                    let CssStyleSelector::Selector(selector) = member else {
                        return Err("ordinary selector list retained a relative member".into());
                    };
                    members.push(
                        selector
                            .to_specified_css()
                            .map_err(|error| format!("selector list: {error:?}"))?,
                    );
                }
                Some(members.join(", "))
            }
        },
        EntryPoint::MediaQuery => {
            let report = parse_media_query(input);
            let query = report.syntax();
            // Authored serialization refuses a query recovered to `never`;
            // CSSOM text projects it to `not all` (Media Queries 4 §3.2).
            let serialized =
                match query.serialize() {
                    Ok(serialized) => serialized,
                    Err(CssMediaSerializationError::RecoveredNever { .. }) => query
                        .serialize_cssom()
                        .map_err(|error| format!("media query: {error:?}"))?,
                    Err(error) => return Err(format!("media query: {error:?}")),
                };
            Some(serialized.as_css().to_owned())
        }
        EntryPoint::MediaQueryList => Some(
            parse_media_query_list(input)
                .syntax()
                .serialize()
                .map_err(|error| format!("media query list: {error:?}"))?
                .as_css()
                .to_owned(),
        ),
    };
    debug_assert!(
        selector_projection || !matches!(registry.adapter(), Adapter::CustomPropertyContainment)
    );
    Ok(match surgeist {
        Some(surgeist) => Projection::Compared { surgeist, upstream },
        // A selector-projected rejection has no expected-class basis, so it is
        // compared as empty output and must carry its own cited exception.
        None if selector_projection => Projection::Compared {
            surgeist: String::new(),
            upstream,
        },
        None if records_whole_input_rejection(expected) => {
            Projection::Excluded(CanonicalExclusion::NoRetainedSyntax)
        }
        None => {
            return Err(format!(
                "public parser retained no syntax, but the expected class is not a whole-input rejection: {expected:?}"
            ));
        }
    })
}

/// Whether the CSS-owned expected class requires the public parser to reject
/// the whole input: it retains no syntax and expects a `RejectInput`
/// diagnostic, either as a strict rejection or as a fully observed unsupported
/// disposition (outside the selected profile, vendor syntax, or an upstream
/// option without a public equivalent).
fn records_whole_input_rejection(expected: &ExpectedClass) -> bool {
    let (retained_syntax, diagnostics) = match expected {
        ExpectedClass::StrictRejected {
            retained_syntax,
            diagnostics,
        }
        | ExpectedClass::Unsupported {
            policy:
                ExpectedUnsupportedPolicy::FullObservation {
                    retained_syntax,
                    is_clean: false,
                    diagnostics,
                },
            ..
        } => (retained_syntax, diagnostics),
        _ => return false,
    };
    matches!(
        retained_syntax.predicate,
        SyntaxCount::Empty | SyntaxCount::Exact { value: 0 }
    ) && diagnostics
        .iter()
        .any(|diagnostic| diagnostic.action == CssRecoveryActionName::RejectInput)
}

/// Normalizes canonical CSS text from either generator.
///
/// Both forms are tokenized by the crate's CSS Syntax 3 tokenizer dependency
/// (cssparser, the Servo and Firefox tokenizer), so token boundaries, escapes,
/// bad strings, bad urls, functions, CDO/CDC and input preprocessing follow the
/// specification rather than a hand-written approximation. On that token
/// sequence the step removes only presentation choices on which CSSTree's
/// compact generator and Surgeist's specified serializers legitimately differ:
///
/// 1. Comments are removed. They still separate the tokens around them, and
///    whitespace runs separated only by comments are one run.
/// 2. Identifiers, at-keywords, hashes, function names and strings compare by
///    decoded value. A `url()`, quoted or unquoted, compares by its decoded
///    value, as CSSOM serializes URLs through strings. A string or url that
///    reaches end of input unterminated, and a block that is never closed,
///    stay distinct from their terminated forms.
/// 3. Whitespace runs are dropped at either end, next to a string, URL or
///    attribute matcher, after `{ } ; , < > + ~ ( : [ = ! /` and a function, and
///    before `{ } ; , < > + ~ ) ] = ! /`. Whitespace stays before `:`, `[`, `.`,
///    `#`, `*`, `&`, `(` and `|`, between `<` or `>` and `=`, on only one side of
///    a `+`, and between other tokens, so a descendant combinator remains
///    distinct from a compound selector, `ns |a` from `ns|a`, `> =` from `>=`
///    and `1px+ 2px` from `1px + 2px`.
/// 4. A `;` immediately before `}` or at the end is dropped; both are optional
///    declaration terminators.
///
/// Decoded identifier case, numeric spelling, units, colors, punctuation, token
/// kinds, block closure and token order remain significant. [`normalize`] renders the same sequence
/// for messages.
pub(crate) fn normalize(css: &str) -> String {
    render(&normalized_tokens(css))
}

/// Whether two forms are equal under the documented normalization, compared
/// token by token rather than through their rendering.
pub(crate) fn canonically_equal(left: &str, right: &str) -> bool {
    normalized_tokens(left) == normalized_tokens(right)
}

fn normalized_tokens(css: &str) -> Vec<Token> {
    let tokens = tokenize(css);
    let mut out: Vec<Token> = Vec::with_capacity(tokens.len());
    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::Whitespace => {
                let Some(previous) = out.last() else {
                    continue;
                };
                // Whitespace runs are merged, so neighbours are adjacent tokens.
                let Some(next) = tokens.get(index + 1) else {
                    continue;
                };
                let comparison_equals = matches!(previous, Token::Delim('<' | '>'))
                    && matches!(next, Token::Delim('='));
                // `a + b` and `a+b` are the same; `1px+ 2px` and `1px +2px` are
                // not, since calc() needs whitespace on both sides of `+`.
                let one_sided_plus = (matches!(next, Token::Delim('+'))
                    && !matches!(tokens.get(index + 2), Some(Token::Whitespace)))
                    || (matches!(previous, Token::Delim('+'))
                        && !(index >= 2 && matches!(tokens[index - 2], Token::Whitespace)));
                if !comparison_equals
                    && !one_sided_plus
                    && (previous.drops_whitespace_after() || next.drops_whitespace_before())
                {
                    continue;
                }
                out.push(Token::Whitespace);
            }
            Token::Delim('}') => {
                while matches!(out.last(), Some(Token::Delim(';') | Token::Whitespace)) {
                    out.pop();
                }
                out.push(Token::Delim('}'));
            }
            other => out.push(other.clone()),
        }
    }
    while matches!(out.last(), Some(Token::Delim(';') | Token::Whitespace)) {
        out.pop();
    }
    out
}

fn render(tokens: &[Token]) -> String {
    let mut text = String::new();
    for token in tokens {
        match token {
            Token::Whitespace => text.push(' '),
            Token::Delim(delim) => text.push(*delim),
            Token::Ident(value) => text.push_str(value),
            Token::AtKeyword(value) => {
                text.push('@');
                text.push_str(value);
            }
            Token::Hash(value) => {
                text.push('#');
                text.push_str(value);
            }
            Token::Numeric(spelling) => text.push_str(spelling),
            Token::String(value) => push_string(&mut text, value),
            Token::Url(value) => {
                text.push_str("url(");
                push_string(&mut text, value);
                text.push(')');
            }
            Token::BadUrl(raw) | Token::BadString(raw) => text.push_str(raw),
            Token::UnterminatedString(value) => {
                text.push('"');
                text.push_str(value);
            }
            Token::UnterminatedUrl(value) => {
                text.push_str("url(");
                text.push_str(value);
            }
            Token::Function(name) => {
                text.push_str(name);
                text.push('(');
            }
            Token::Matcher(first) => {
                text.push(*first);
                text.push('=');
            }
            Token::Cdo => text.push_str("<!--"),
            Token::Cdc => text.push_str("-->"),
        }
    }
    text
}

fn push_string(text: &mut String, value: &str) {
    text.push('"');
    for character in value.chars() {
        if matches!(character, '"' | '\\') {
            text.push('\\');
        }
        text.push(character);
    }
    text.push('"');
}

/// A comparison token. Kinds stay distinct, so an escaped identifier never
/// equals an at-keyword, hash, number or string with the same text.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Token {
    Whitespace,
    Delim(char),
    Ident(String),
    AtKeyword(String),
    Hash(String),
    /// A number, percentage or dimension, by its source spelling.
    Numeric(String),
    String(String),
    /// A string that reaches end of input without its closing quote.
    UnterminatedString(String),
    Url(String),
    /// An unquoted url that reaches end of input without its `)`.
    UnterminatedUrl(String),
    BadUrl(String),
    BadString(String),
    Function(String),
    /// An attribute matcher: `~=`, `|=`, `^=`, `$=` or `*=`, by its first char.
    Matcher(char),
    Cdo,
    Cdc,
}

impl Token {
    fn drops_whitespace_after(&self) -> bool {
        matches!(
            self,
            Self::String(_)
                | Self::Url(_)
                | Self::Function(_)
                | Self::Matcher(_)
                | Self::Delim(
                    '{' | '}'
                        | ';'
                        | ','
                        | '<'
                        | '>'
                        | '+'
                        | '~'
                        | '('
                        | ':'
                        | '['
                        | '='
                        | '!'
                        | '/'
                )
        )
    }

    fn drops_whitespace_before(&self) -> bool {
        matches!(
            self,
            Self::String(_)
                | Self::Url(_)
                | Self::Matcher(_)
                | Self::Delim(
                    '{' | '}' | ';' | ',' | '<' | '>' | '+' | '~' | ')' | ']' | '=' | '!' | '/'
                )
        )
    }
}

/// Whether `source` ends with an unescaped `close` code point.
fn closes_with(source: &str, close: char) -> bool {
    let Some(rest) = source.strip_suffix(close) else {
        return false;
    };
    rest.chars().rev().take_while(|&c| c == '\\').count() % 2 == 0
}

fn tokenize(css: &str) -> Vec<Token> {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    let mut tokens = Vec::new();
    collect_tokens(&mut parser, &mut tokens);
    tokens
}

/// Appends the tokens of one block level, descending into nested blocks and
/// emitting their closing delimiter only when the source actually closes them.
fn collect_tokens(parser: &mut Parser<'_, '_>, tokens: &mut Vec<Token>) {
    loop {
        let start = parser.position();
        let Ok(token) = parser.next_including_whitespace_and_comments() else {
            return;
        };
        let (opening, closing) = match token {
            CssToken::WhiteSpace(_) => {
                if !matches!(tokens.last(), Some(Token::Whitespace)) {
                    tokens.push(Token::Whitespace);
                }
                continue;
            }
            CssToken::Comment(_) => continue,
            CssToken::Ident(value) => {
                tokens.push(Token::Ident(value.to_string()));
                continue;
            }
            CssToken::AtKeyword(value) => {
                tokens.push(Token::AtKeyword(value.to_string()));
                continue;
            }
            CssToken::Hash(value) | CssToken::IDHash(value) => {
                tokens.push(Token::Hash(value.to_string()));
                continue;
            }
            CssToken::QuotedString(value) => {
                let value = value.to_string();
                let source = parser.slice_from(start);
                let quote = source.chars().next().unwrap_or('"');
                tokens.push(if source.len() > 1 && closes_with(source, quote) {
                    Token::String(value)
                } else {
                    Token::UnterminatedString(value)
                });
                continue;
            }
            CssToken::UnquotedUrl(value) => {
                let value = value.to_string();
                tokens.push(if closes_with(parser.slice_from(start), ')') {
                    Token::Url(value)
                } else {
                    Token::UnterminatedUrl(value)
                });
                continue;
            }
            CssToken::Number { .. } | CssToken::Percentage { .. } | CssToken::Dimension { .. } => {
                tokens.push(Token::Numeric(parser.slice_from(start).to_owned()));
                continue;
            }
            CssToken::Delim(delim) => {
                tokens.push(Token::Delim(*delim));
                continue;
            }
            CssToken::Colon => {
                tokens.push(Token::Delim(':'));
                continue;
            }
            CssToken::Semicolon => {
                tokens.push(Token::Delim(';'));
                continue;
            }
            CssToken::Comma => {
                tokens.push(Token::Delim(','));
                continue;
            }
            CssToken::IncludeMatch => {
                tokens.push(Token::Matcher('~'));
                continue;
            }
            CssToken::DashMatch => {
                tokens.push(Token::Matcher('|'));
                continue;
            }
            CssToken::PrefixMatch => {
                tokens.push(Token::Matcher('^'));
                continue;
            }
            CssToken::SuffixMatch => {
                tokens.push(Token::Matcher('$'));
                continue;
            }
            CssToken::SubstringMatch => {
                tokens.push(Token::Matcher('*'));
                continue;
            }
            CssToken::CDO => {
                tokens.push(Token::Cdo);
                continue;
            }
            CssToken::CDC => {
                tokens.push(Token::Cdc);
                continue;
            }
            CssToken::BadUrl(_) => {
                tokens.push(Token::BadUrl(parser.slice_from(start).to_owned()));
                continue;
            }
            CssToken::BadString(_) => {
                tokens.push(Token::BadString(parser.slice_from(start).to_owned()));
                continue;
            }
            CssToken::CloseParenthesis => {
                tokens.push(Token::Delim(')'));
                continue;
            }
            CssToken::CloseSquareBracket => {
                tokens.push(Token::Delim(']'));
                continue;
            }
            CssToken::CloseCurlyBracket => {
                tokens.push(Token::Delim('}'));
                continue;
            }
            CssToken::Function(name) => (Some(Token::Function(name.to_string())), ')'),
            CssToken::ParenthesisBlock => (None, ')'),
            CssToken::SquareBracketBlock => (Some(Token::Delim('[')), ']'),
            CssToken::CurlyBracketBlock => (Some(Token::Delim('{')), '}'),
        };
        let mut nested = Vec::new();
        let mut inner_end = None;
        let _ = parser.parse_nested_block(|block| {
            collect_tokens(block, &mut nested);
            inner_end = Some(block.position());
            Ok::<(), cssparser::ParseError<'_, ()>>(())
        });
        // The nested parser stops before the closing token; the block is closed
        // exactly when the outer parser then consumed one beyond that point.
        let closed = inner_end.is_some_and(|end| parser.position() != end);
        // `url(` with one string argument is a URL compared by its value.
        if let Some(Token::Function(name)) = &opening
            && name.eq_ignore_ascii_case("url")
            && closed
        {
            let argument: Vec<&Token> = nested
                .iter()
                .filter(|token| !matches!(token, Token::Whitespace))
                .collect();
            if let [Token::String(value)] = argument.as_slice() {
                tokens.push(Token::Url(value.clone()));
                continue;
            }
        }
        tokens.push(opening.unwrap_or(Token::Delim('(')));
        tokens.append(&mut nested);
        if closed {
            tokens.push(Token::Delim(closing));
        }
    }
}

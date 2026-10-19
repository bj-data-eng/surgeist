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
//! [`EXCEPTIONS`] with their kind and reason; stale exceptions also fail.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    Adapter, EntryPoint, ExpectedClass, ExpectedUnsupportedPolicy, PropertyOrDescriptor, REGISTRY,
    UpstreamOutcome, ValidatedCase, corpus_namespace_context, load_neutral_csstree_inventory,
    resolve_case_registry,
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

/// One explicit per-case difference with its one-line reason.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CanonicalException {
    id: &'static str,
    kind: CanonicalExceptionKind,
    reason: &'static str,
}

#[path = "canonical_exceptions.rs"]
mod exceptions;
pub(crate) use exceptions::EXCEPTIONS;

/// Upstream-parsed cases excluded without comparison, by category.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CanonicalExclusion {
    /// The CSS-owned expected class records an adapterless panic-freedom-only
    /// policy: no truthful public entry point exists, so no projection exists.
    NoPublicProjection,
    /// The public parser rejects the whole input and retains no syntax. The
    /// CSS-owned expected class records the strict rejection and its basis.
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
        let surgeist = normalize(&serialized);
        let upstream = normalize(&upstream);
        let entry = summary.compared.entry(context).or_default();
        match (surgeist == upstream, exceptions.get(case.id.as_str())) {
            (true, None) => entry.0 += 1,
            (true, Some(listed)) => failures.push(format!(
                "{}: stale {} exception now matches upstream `{upstream}`",
                listed.id,
                listed.kind.name()
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
        if !used.contains(listed.id) && !failures.iter().any(|f| f.starts_with(listed.id)) {
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
    if matches!(
        expected,
        ExpectedClass::Unsupported {
            policy: ExpectedUnsupportedPolicy::PanicFreedomOnly,
            ..
        }
    ) {
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
    let surgeist = match registry.entry_point() {
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
    debug_assert!(!matches!(
        registry.adapter(),
        Adapter::CustomPropertyContainment
    ));
    Ok(match surgeist {
        Some(surgeist) => Projection::Compared { surgeist, upstream },
        None => Projection::Excluded(CanonicalExclusion::NoRetainedSyntax),
    })
}

/// Normalizes canonical CSS text from either generator.
///
/// The step removes only presentation choices on which CSSTree's compact
/// generator and Surgeist's specified serializers legitimately differ:
///
/// 1. Comments are removed (neither generator's structure depends on them).
/// 2. Strings are decoded and re-emitted with double quotes, escaping only
///    `"` and `\`. A quoted or unquoted `url()` becomes `url("value")` with its
///    decoded value, as CSSOM serializes URLs through strings.
/// 3. Whitespace runs collapse to one space. They are dropped at either end,
///    next to a string or URL, after `{ } ; , < > + ~ ( : [ = ! / |`, and before
///    `{ } ; , < > + ~ ) ] = ! / |`. Whitespace before `:`, `[`, `.`, `#`, `*`,
///    `&`, `(` and between words stays, so a descendant combinator remains
///    distinct from a compound selector.
/// 4. A `;` immediately before `}` or at the end is dropped; both are optional
///    declaration terminators.
///
/// Identifier case, escapes, numeric spelling, units, colors, punctuation and
/// token order remain significant.
pub(crate) fn normalize(css: &str) -> String {
    let tokens = tokenize(css);
    let mut out: Vec<Token> = Vec::with_capacity(tokens.len());
    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::Whitespace => {
                let Some(previous) = out.last() else {
                    continue;
                };
                let next = tokens[index + 1..]
                    .iter()
                    .find(|token| !matches!(token, Token::Whitespace));
                let Some(next) = next else {
                    continue;
                };
                if previous.drops_whitespace_after() || next.drops_whitespace_before() {
                    continue;
                }
                if !matches!(previous, Token::Whitespace) {
                    out.push(Token::Whitespace);
                }
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
    let mut text = String::with_capacity(css.len());
    for token in out {
        match token {
            Token::Whitespace => text.push(' '),
            Token::Delim(delim) => text.push(delim),
            Token::Word(word) => text.push_str(&word),
            Token::String(value) => push_string(&mut text, &value),
            Token::Url(value) => {
                text.push_str("url(");
                push_string(&mut text, &value);
                text.push(')');
            }
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum Token {
    Whitespace,
    Delim(char),
    Word(String),
    String(String),
    Url(String),
}

impl Token {
    fn drops_whitespace_after(&self) -> bool {
        matches!(
            self,
            Self::String(_)
                | Self::Url(_)
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
                        | '|'
                )
        )
    }

    fn drops_whitespace_before(&self) -> bool {
        matches!(
            self,
            Self::String(_)
                | Self::Url(_)
                | Self::Delim(
                    '{' | '}'
                        | ';'
                        | ','
                        | '<'
                        | '>'
                        | '+'
                        | '~'
                        | ')'
                        | ']'
                        | '='
                        | '!'
                        | '/'
                        | '|'
                )
        )
    }
}

fn tokenize(css: &str) -> Vec<Token> {
    let characters: Vec<char> = css.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if is_whitespace(character) {
            while index < characters.len() && is_whitespace(characters[index]) {
                index += 1;
            }
            tokens.push(Token::Whitespace);
        } else if character == '/' && characters.get(index + 1) == Some(&'*') {
            index += 2;
            while index < characters.len()
                && !(characters[index] == '*' && characters.get(index + 1) == Some(&'/'))
            {
                index += 1;
            }
            index = (index + 2).min(characters.len());
        } else if matches!(character, '"' | '\'') {
            let (value, next) = consume_string(&characters, index + 1, character);
            tokens.push(Token::String(value));
            index = next;
        } else if is_word_start(&characters, index) {
            let start = index;
            index += 1;
            while index < characters.len() && is_word_continue(&characters, index) {
                index += if characters[index] == '\\' { 2 } else { 1 };
            }
            index = index.min(characters.len());
            let word: String = characters[start..index].iter().collect();
            if word.eq_ignore_ascii_case("url") && characters.get(index) == Some(&'(') {
                let (value, next) = consume_url(&characters, index + 1);
                tokens.push(Token::Url(value));
                index = next;
            } else {
                tokens.push(Token::Word(word));
            }
        } else {
            tokens.push(Token::Delim(character));
            index += 1;
        }
    }
    tokens
}

const fn is_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r' | '\u{c}')
}

const fn is_name(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-') || !character.is_ascii()
}

fn is_word_start(characters: &[char], index: usize) -> bool {
    let character = characters[index];
    let next = characters.get(index + 1).copied();
    is_name(character)
        || (character == '\\' && next.is_some())
        || (matches!(character, '#' | '@')
            && next.is_some_and(|next| is_name(next) || next == '\\'))
        || (matches!(character, '+' | '.') && next.is_some_and(|next| next.is_ascii_digit()))
}

fn is_word_continue(characters: &[char], index: usize) -> bool {
    let character = characters[index];
    is_name(character)
        || character == '%'
        || (character == '\\' && index + 1 < characters.len())
        || (character == '.'
            && characters[index - 1].is_ascii_digit()
            && characters.get(index + 1).is_some_and(char::is_ascii_digit))
        || (matches!(character, '+' | '-')
            && matches!(characters[index - 1], 'e' | 'E')
            && characters.get(index + 1).is_some_and(char::is_ascii_digit)
            && characters[..index - 1]
                .last()
                .is_some_and(char::is_ascii_digit))
}

/// Consumes `url(` contents: one string, or unquoted text through `)` with
/// escapes decoded, both without surrounding whitespace.
fn consume_url(characters: &[char], mut index: usize) -> (String, usize) {
    while characters
        .get(index)
        .is_some_and(|&next| is_whitespace(next))
    {
        index += 1;
    }
    if let Some(&quote) = characters
        .get(index)
        .filter(|&&next| matches!(next, '"' | '\''))
    {
        let (value, mut next) = consume_string(characters, index + 1, quote);
        while characters.get(next).is_some_and(|&close| close != ')') {
            next += 1;
        }
        return (value, (next + 1).min(characters.len()));
    }
    let mut value = String::new();
    while let Some(&character) = characters.get(index) {
        index += 1;
        match character {
            ')' => break,
            '\\' => {
                let (decoded, next) = consume_escape(characters, index);
                value.push(decoded);
                index = next;
            }
            _ => value.push(character),
        }
    }
    let trimmed = value.trim_end_matches(is_whitespace).len();
    value.truncate(trimmed);
    (value, index)
}

/// Consumes string contents after the opening quote. An unescaped newline ends
/// a bad string without being consumed (CSS Syntax 3 §4.3.5).
fn consume_string(characters: &[char], mut index: usize, quote: char) -> (String, usize) {
    let mut value = String::new();
    while let Some(&character) = characters.get(index) {
        if matches!(character, '\n' | '\r' | '\u{c}') {
            break;
        }
        index += 1;
        if character == quote {
            break;
        }
        if character != '\\' {
            value.push(character);
            continue;
        }
        match characters.get(index) {
            None => break,
            Some('\n' | '\r' | '\u{c}') => index += 1,
            Some(_) => {
                let (decoded, next) = consume_escape(characters, index);
                value.push(decoded);
                index = next;
            }
        }
    }
    (value, index)
}

/// Decodes one escape whose backslash precedes `index` (CSS Syntax 3 §4.3.7).
fn consume_escape(characters: &[char], mut index: usize) -> (char, usize) {
    let Some(&escaped) = characters.get(index) else {
        return ('\u{fffd}', index);
    };
    if !escaped.is_ascii_hexdigit() {
        return (escaped, index + 1);
    }
    let start = index;
    while index < characters.len() && index - start < 6 && characters[index].is_ascii_hexdigit() {
        index += 1;
    }
    let digits: String = characters[start..index].iter().collect();
    let decoded = u32::from_str_radix(&digits, 16)
        .ok()
        .and_then(char::from_u32)
        .filter(|&decoded| decoded != '\0')
        .unwrap_or('\u{fffd}');
    if characters
        .get(index)
        .is_some_and(|&next| is_whitespace(next))
    {
        index += 1;
    }
    (decoded, index)
}

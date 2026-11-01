use cssparser::{ParseError, Parser};
use surgeist_css_parser::anb;
use surgeist_css_parser::stream::StopToken;
use surgeist_css_parser::token::HashKind;
use surgeist_css_parser::{Token, match_ignore_ascii_case};

use super::block_units::offset_of;
use super::descriptor_stream::{
    Value, ValueError, after_token, at_final_comma, consume_remaining, fail, native, parse_scope,
};
use super::property_input::StreamExt;
use super::recovery::{RecoveryState, comma_member_span, recovery_action_for_error};
use super::syntax_types::{Mark, Site, Stream, SyntaxError};
use crate::diagnostic::error::{
    from_parse_error, invalid_selector_at, selector_component_error, selector_encountered_at,
};
use crate::diagnostic::{CssError, CssFeatureId, CssTokenSummary};
use crate::numeric::expression::NumericInputContext;
use crate::rule::CssNamespacePrefix;
use crate::selector::selector_syntax::{
    CssCompoundMembers, CssPseudoSuffixContext, CssSelectorGrammarContext,
    selector_is_valid_pseudo_suffix,
};
use crate::selector::{
    CssAttributeCaseSensitivity, CssAttributeMatcher, CssAttributeName, CssAttributeSelector,
    CssComplexSelector, CssComplexSelectorPart, CssCompoundSelector, CssCompoundSelectorArgument,
    CssInvalidNestingSelectorItem, CssLanguageRange, CssLanguageRangeList, CssNamespaceConstraint,
    CssNthAnPlusB, CssNthChildPattern, CssNthPattern, CssPartName, CssPartNameList, CssPseudoClass,
    CssPseudoElement, CssPseudoElementSegment, CssPseudoElementSequence, CssPseudoSelectorList,
    CssPseudoSelectorListItem, CssQualifiedAttributeName, CssQualifiedNamePrefix,
    CssQualifiedSelectorName, CssRelativeSelector, CssRelativeSelectorList, CssScopeSelectorList,
    CssSelector, CssSelectorCombinator, CssStyleSelector, CssStyleSelectorList,
    CssUnknownWebkitPseudoElement,
};
use crate::values::common::{CssCustomIdent, CssIdent};
use crate::values::text::CssDirectionality;
use crate::values::view_transition::CssViewTransitionNameSelector;

pub(super) static IMPLEMENTED_SELECTORS: &[CssFeatureId] = &[
    CssFeatureId::new("baseline.selector.complex"),
    CssFeatureId::new("baseline.selector.pseudo-class"),
    CssFeatureId::new("baseline.selector.functional"),
    CssFeatureId::new("baseline.selector.extension-state"),
    CssFeatureId::new("baseline.selector.extension-functional"),
    CssFeatureId::new("baseline.selector.attribute-case"),
    CssFeatureId::new("official.selector.group"),
    CssFeatureId::new("official.selector.type"),
    CssFeatureId::new("official.selector.universal"),
    CssFeatureId::new("official.selector.attribute-presence-value"),
    CssFeatureId::new("official.selector.attribute-substring"),
    CssFeatureId::new("official.selector.class"),
    CssFeatureId::new("official.selector.id"),
    CssFeatureId::new("official.selector.dynamic"),
    CssFeatureId::new("official.selector.target"),
    CssFeatureId::new("official.selector.lang"),
    CssFeatureId::new("ext.selector.dir"),
    CssFeatureId::new("ext.selector.autofill"),
    CssFeatureId::new("official.selector.ui-state"),
    CssFeatureId::new("official.selector.structural"),
    CssFeatureId::new("official.selector.negation"),
    CssFeatureId::new("official.selector.first-line"),
    CssFeatureId::new("official.selector.first-letter"),
    CssFeatureId::new("official.selector.generated"),
    CssFeatureId::new("official.selector.combinator.descendant"),
    CssFeatureId::new("official.selector.combinator.child"),
    CssFeatureId::new("official.selector.combinator.next-sibling"),
    CssFeatureId::new("official.selector.combinator.subsequent-sibling"),
    CssFeatureId::new("official.selector.namespace-qualified-name"),
    CssFeatureId::new("ext.pseudo-element.marker"),
    CssFeatureId::new("ext.pseudo-element.selection"),
    CssFeatureId::new("ext.pseudo-element.prefix"),
    CssFeatureId::new("ext.pseudo-element.suffix"),
    CssFeatureId::new("ext.pseudo-element.search-text"),
    CssFeatureId::new("ext.pseudo-element.target-text"),
    CssFeatureId::new("ext.pseudo-element.spelling-error"),
    CssFeatureId::new("ext.pseudo-element.grammar-error"),
    CssFeatureId::new("ext.pseudo-element.highlight"),
    CssFeatureId::new("ext.pseudo-element.placeholder"),
    CssFeatureId::new("ext.pseudo-element.file-selector-button"),
    CssFeatureId::new("ext.pseudo-element.details-content"),
    CssFeatureId::new("ext.pseudo-element.search-text-current"),
    CssFeatureId::new("official.pseudo-element.view-transition"),
    CssFeatureId::new("official.pseudo-element.view-transition-group"),
    CssFeatureId::new("official.pseudo-element.view-transition-image-pair"),
    CssFeatureId::new("official.pseudo-element.view-transition-old"),
    CssFeatureId::new("official.pseudo-element.view-transition-new"),
    CssFeatureId::new("ext.pseudo-element.backdrop"),
    CssFeatureId::new("ext.pseudo-element.generated-marker"),
    CssFeatureId::new("ext.pseudo-element.unknown-webkit"),
];

pub(crate) struct SelectorRecovery<'a> {
    source: &'a str,
    diagnostics: &'a mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    state: RecoveryState,
}

/// The delimiter that a failure at an `=` names when it follows `~`, `|`, `^`,
/// `$` or `*` directly: the two make the operator of an attribute selector, and
/// the operator is where the failure begins. Anything else is as it was met.
fn matcher_start<'i>(at: Site<'i>, token: Option<Token<'i>>) -> (Site<'i>, Option<Token<'i>>) {
    if !matches!(token, Some(Token::Delim('='))) {
        return (at, token);
    }
    let document = at.document();
    let nodes = document.list(at.list());
    let at_equals = nodes
        .get(at.index())
        .is_some_and(|node| matches!(document.token(*node), Token::Delim('=')));
    let before = at
        .index()
        .checked_sub(1)
        .filter(|_| at_equals)
        .and_then(|previous| nodes.get(previous).map(|node| (previous, *node)));
    match before {
        Some((previous, node)) => match document.token(node) {
            Token::Delim(delim @ ('~' | '|' | '^' | '$' | '*')) => (
                document.site(at.list(), previous).unwrap_or(at),
                Some(Token::Delim(delim)),
            ),
            _ => (at, token),
        },
        None => (at, token),
    }
}

/// The summary of the unexpected `token` of the text at `at`.
fn encountered(at: Site<'_>, token: Token<'_>) -> CssTokenSummary {
    match token {
        // The whitespace is named as it was written.
        Token::Whitespace => CssTokenSummary::from_authored_syntax_token(
            token,
            super::descriptor_stream::token_spelling(at).unwrap_or(" "),
        ),
        _ => CssTokenSummary::from_syntax_token(token),
    }
}

/// The failure of a grammar whose stream ended, at `at`.
fn end_of<'i>(at: Site<'i>) -> ValueError<'i> {
    at.end_of_stream()
}

/// The failure of a grammar that met `token` at `at` where it needed another.
fn unexpected<'i>(at: Site<'i>, token: Token<'i>) -> ValueError<'i> {
    at.unexpected(token)
}

/// The byte where the opening token at `at` ends, its last byte.
fn opening_end(at: Site<'_>) -> usize {
    (offset_of(at) + at.spelling().map_or(1, str::len)).saturating_sub(1)
}

impl<'a> SelectorRecovery<'a> {
    pub(super) fn new(
        source: &'a str,
        diagnostics: &'a mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
        state: RecoveryState,
    ) -> Self {
        Self {
            source,
            diagnostics,
            state,
        }
    }

    fn unqualified_type_namespace(&self) -> CssNamespaceConstraint {
        if self.state.has_default_namespace() {
            CssNamespaceConstraint::Default
        } else {
            CssNamespaceConstraint::Any
        }
    }

    fn named_namespace(&self, prefix: &str) -> Option<CssNamespacePrefix> {
        self.state.active_namespace_prefix(prefix)
    }

    fn numeric(&self) -> NumericInputContext<'_> {
        NumericInputContext::parsed(self.state.source_snapshot())
    }

    /// An invalid selector, at the boundary `at`.
    fn invalid<'i>(&self, at: Site<'i>) -> ValueError<'i> {
        let (at, _) = matcher_start(at, Some(Token::Delim('=')));
        fail(&self.numeric(), at, invalid_selector_at)
    }

    /// The failure `error` of the token stream, as the selector failure that
    /// names the token it met.
    fn basic<'i>(&self, error: impl Into<ValueError<'i>>) -> ValueError<'i> {
        let error = error.into();
        let (at, unexpected) = matcher_start(error.at(), error.unexpected_token());
        let token = unexpected.map(|token| encountered(at, token));
        fail(&self.numeric(), at, |at| selector_encountered_at(at, token))
    }

    /// The failure that met the whitespace at `whitespace`, reported at `after`.
    fn basic_whitespace<'i>(&self, whitespace: Site<'i>, after: Site<'i>) -> ValueError<'i> {
        let token = encountered(whitespace, Token::Whitespace);
        fail(&self.numeric(), after, |at| {
            selector_encountered_at(at, Some(token))
        })
    }

    /// A failure of the component values of the selector, at `at`.
    fn component<'i>(
        &self,
        at: Site<'i>,
        error: crate::component::CssComponentValueError,
    ) -> ValueError<'i> {
        fail(&self.numeric(), at, |at| {
            selector_component_error(at, error)
        })
    }

    pub(super) fn check_depth<'i>(&self, input: &Stream<'i>) -> Result<(), ValueError<'i>> {
        self.state
            .check_specialized_components_from(
                self.source,
                offset_of(input.site()),
                "baseline.selector.complex",
            )
            .map(|_| ())
            .map_err(|error| SyntaxError::custom(input.site(), error))
    }

    fn check_forgiving_envelope<'i>(&self, input: &Stream<'i>) -> Result<(), ValueError<'i>> {
        // Selectors 4 checks the complete <any-value>? envelope before member
        // forgiveness. The component owner sees bad tokens and mismatched
        // closers at every depth, while allowing missing EOF delimiters.
        let start = input.site();
        input
            .remaining_components()
            .map(|_| ())
            .map_err(|error| self.component(start, error))
    }

    fn diagnose_forgiving_member(
        &mut self,
        error: ParseError<'_, CssError>,
        member_start: usize,
        member_end: usize,
        following_comma: Option<(usize, usize)>,
        preceding_comma: Option<(usize, usize)>,
        ordinary_action: crate::diagnostic::CssRecoveryAction,
    ) {
        let action = recovery_action_for_error(&error, ordinary_action);
        let error = from_parse_error(self.source, error);
        let Some(span) = comma_member_span(
            self.source,
            member_start,
            member_end,
            following_comma,
            preceding_comma,
        ) else {
            return;
        };
        if let Some(diagnostic) = crate::diagnostic::CssRecoveryDiagnostic::new(error, span, action)
        {
            self.diagnostics.push(diagnostic);
        }
    }

    /// Reads the scope of the native parser `input` with `parse`, and leaves
    /// the parser where the grammar left the stream.
    fn read<'i, T>(
        &mut self,
        input: &mut Parser<'i, '_>,
        parse: impl for<'d> FnOnce(&mut Stream<'d>, &mut Self) -> Value<'d, T>,
    ) -> Result<T, ParseError<'i, CssError>> {
        let snapshot = self.state.source_snapshot().clone();
        parse_scope(input, &snapshot, |stream, _| parse(stream, self))
    }

    /// Reads the stream `input` the driver delimited with `parse`, from where
    /// it stands.
    fn read_stream<'i, T>(
        &mut self,
        input: &mut Stream<'i>,
        parse: impl FnOnce(&mut Stream<'i>, &mut Self) -> Value<'i, T>,
    ) -> Result<T, ParseError<'i, CssError>> {
        let snapshot = self.state.source_snapshot().clone();
        let numeric = crate::numeric::expression::NumericInputContext::parsed(&snapshot);
        parse(input, self).map_err(|error| native(&numeric, &error))
    }
}

pub(crate) fn parse_rule_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<CssSelector>, ParseError<'i, CssError>> {
    recovery.read(input, rule_selector_list)
}

pub(crate) fn parse_rule_selector_list_stream<'i>(
    input: &mut Stream<'i>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<CssSelector>, ParseError<'i, CssError>> {
    recovery.read_stream(input, rule_selector_list)
}

fn rule_selector_list<'i>(
    input: &mut Stream<'i>,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, Vec<CssSelector>> {
    recovery.check_depth(input)?;
    let mut selectors = Vec::new();
    loop {
        selectors.push(read_rule_selector(input, recovery)?);
        if input.attempt(Stream::consume_comma).is_err() {
            break;
        }
    }
    expect_exhausted(input, recovery)?;
    Ok(selectors)
}

/// Requires that only whitespace remains, as a selector failure that names the
/// first token that remains where the whitespace begins.
fn expect_exhausted<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Result<(), ValueError<'i>> {
    super::descriptor_stream::expect_exhausted(input).map_err(|error| recovery.basic(error))
}

/// The selector list of a `@scope` boundary, in place in the boundary's block.
pub(crate) fn scope_boundary_selector_list<'i>(
    input: &mut Stream<'i>,
    recovery: &mut SelectorRecovery<'_>,
    anchors: SelectorAnchorMode,
    allow_relative: bool,
) -> Value<'i, CssScopeSelectorList> {
    recovery
        .state
        .check_component_contents_from(
            recovery.source,
            offset_of(input.site()),
            "baseline.selector.complex",
        )
        .map_err(|error| SyntaxError::custom(input.site(), error))?;
    let options = SelectorParseOptions::scope_boundary(anchors);
    let mut selectors = Vec::new();
    loop {
        let member = if allow_relative {
            parse_style_selector_with_options(input, options, recovery)?
        } else {
            CssStyleSelector::Selector(parse_rule_selector_with_options(input, options, recovery)?)
        };
        selectors.push(member);
        if input.attempt(Stream::consume_comma).is_err() {
            break;
        }
    }
    expect_exhausted(input, recovery)?;
    CssScopeSelectorList::try_new(selectors).ok_or_else(|| recovery.invalid(input.site()))
}

pub(crate) fn parse_scoped_style_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssStyleSelectorList, ParseError<'i, CssError>> {
    recovery.read(input, scoped_style_selector_list)
}

pub(crate) fn parse_scoped_style_selector_list_stream<'i>(
    input: &mut Stream<'i>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssStyleSelectorList, ParseError<'i, CssError>> {
    recovery.read_stream(input, scoped_style_selector_list)
}

fn scoped_style_selector_list<'i>(
    input: &mut Stream<'i>,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssStyleSelectorList> {
    recovery.check_depth(input)?;
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_style_selector_with_options(
            input,
            SelectorParseOptions::scoped_style(),
            recovery,
        )?);
        if input.attempt(Stream::consume_comma).is_err() {
            break;
        }
    }
    expect_exhausted(input, recovery)?;
    CssStyleSelectorList::try_new(selectors).ok_or_else(|| recovery.invalid(input.site()))
}

pub(super) fn parse_nested_style_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<CssStyleSelector>, ParseError<'i, CssError>> {
    recovery.read(input, nested_style_selector_list)
}

pub(super) fn parse_nested_style_selector_list_stream<'i>(
    input: &mut Stream<'i>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<CssStyleSelector>, ParseError<'i, CssError>> {
    recovery.read_stream(input, nested_style_selector_list)
}

fn nested_style_selector_list<'i>(
    input: &mut Stream<'i>,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, Vec<CssStyleSelector>> {
    recovery.check_depth(input)?;
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_style_selector_with_options(
            input,
            SelectorParseOptions::nested_style(),
            recovery,
        )?);
        if input.attempt(Stream::consume_comma).is_err() {
            break;
        }
    }
    expect_exhausted(input, recovery)?;
    Ok(selectors)
}

fn parse_style_selector_with_options<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssStyleSelector> {
    consume_selector_whitespace(input, recovery)?;
    let state = input.mark();
    match input.consume_token() {
        Ok(Token::Delim('>')) => parse_selector_after_leading_combinator_with_options(
            input,
            CssSelectorCombinator::Child,
            options,
            recovery,
        )
        .map(CssStyleSelector::Relative),
        Ok(Token::Delim('+')) => parse_selector_after_leading_combinator_with_options(
            input,
            CssSelectorCombinator::NextSibling,
            options,
            recovery,
        )
        .map(CssStyleSelector::Relative),
        Ok(Token::Delim('~')) => parse_selector_after_leading_combinator_with_options(
            input,
            CssSelectorCombinator::SubsequentSibling,
            options,
            recovery,
        )
        .map(CssStyleSelector::Relative),
        Ok(Token::Delim('|')) => {
            if matches!(input.consume_token(), Ok(Token::Delim('|'))) {
                return parse_selector_after_leading_combinator_with_options(
                    input,
                    CssSelectorCombinator::Column,
                    options,
                    recovery,
                )
                .map(CssStyleSelector::Relative);
            }
            input.restore(state);
            parse_rule_selector_with_options(input, options, recovery)
                .map(CssStyleSelector::Selector)
        }
        Ok(_) => {
            input.restore(state);
            parse_rule_selector_with_options(input, options, recovery)
                .map(CssStyleSelector::Selector)
        }
        Err(error) if error.is_end_of_stream() => {
            input.restore(state);
            Err(recovery.invalid(input.site()))
        }
        Err(error) => Err(recovery.basic(error)),
    }
}

pub(super) fn read_rule_selector<'i>(
    input: &mut Stream<'i>,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssSelector> {
    parse_rule_selector_with_options(input, SelectorParseOptions::standard(), recovery)
}

/// The selector that the native parser `input` holds, to the end of its scope.
pub(super) fn parse_rule_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssSelector, ParseError<'i, CssError>> {
    recovery.read(input, |input, recovery| read_rule_selector(input, recovery))
}

#[derive(Clone, Copy)]
pub(crate) enum SelectorAnchorMode {
    Nesting,
    Scope,
}

#[derive(Clone, Copy)]
struct SelectorParseOptions {
    allow_has: bool,
    anchors: SelectorAnchorMode,
    allow_pseudo_elements: bool,
    compound_only: bool,
    pseudo_suffix: Option<CssPseudoSuffixContext>,
}

impl SelectorParseOptions {
    const fn grammar(self) -> CssSelectorGrammarContext {
        CssSelectorGrammarContext::from_parser_restrictions(
            self.allow_pseudo_elements,
            self.allow_has,
            self.compound_only,
            self.pseudo_suffix,
        )
    }

    const fn standard() -> Self {
        Self {
            allow_has: true,
            anchors: SelectorAnchorMode::Nesting,
            allow_pseudo_elements: true,
            compound_only: false,
            pseudo_suffix: None,
        }
    }

    const fn without_nested_has(self) -> Self {
        Self {
            allow_has: false,
            ..self
        }
    }

    const fn nested_style() -> Self {
        Self::standard()
    }

    const fn scoped_style() -> Self {
        Self {
            allow_has: true,
            anchors: SelectorAnchorMode::Scope,
            allow_pseudo_elements: true,
            compound_only: false,
            pseudo_suffix: None,
        }
    }

    const fn scope_boundary(anchors: SelectorAnchorMode) -> Self {
        Self {
            allow_has: true,
            anchors,
            allow_pseudo_elements: false,
            compound_only: false,
            pseudo_suffix: None,
        }
    }

    const fn without_pseudo_elements(self) -> Self {
        Self {
            allow_pseudo_elements: false,
            ..self
        }
    }

    const fn independent_arguments(self) -> Self {
        Self {
            compound_only: false,
            pseudo_suffix: None,
            ..self
        }
    }
}

fn parse_rule_selector_with_options<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssSelector> {
    let first = parse_compound_selector_model_with_options(input, options, recovery)?;
    let selector = parse_selector_after_first_compound(input, first, options, recovery)?;
    if options.compound_only && matches!(selector, CssSelector::Complex(_)) {
        return Err(recovery.invalid(input.site()));
    }
    if options.pseudo_suffix.is_some_and(|suffix| {
        !selector_is_valid_pseudo_suffix(&selector, suffix, options.allow_has)
    }) {
        return Err(recovery.invalid(input.site()));
    }
    Ok(selector)
}

fn parse_selector_after_first_compound<'i>(
    input: &mut Stream<'i>,
    first: CssCompoundSelector,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssSelector> {
    let mut rest = Vec::new();

    loop {
        let had_whitespace = consume_selector_whitespace(input, recovery)?;
        let state = input.mark();
        // Selectors 4 §3.6.3: no combinator may follow a pseudo-element, so
        // that combinator is responsible.
        let after_pseudo_element = rest
            .last()
            .map_or(&first, CssComplexSelectorPart::selector)
            .has_pseudo_elements();
        match input.consume_token() {
            Err(error) if error.is_end_of_stream() => {
                input.restore(state);
                break;
            }
            Err(error) => return Err(recovery.basic(error)),
            Ok(Token::Comma) => {
                input.restore(state);
                break;
            }
            Ok(Token::Delim('>')) => {
                if after_pseudo_element {
                    return Err(recovery.invalid(state.site()));
                }
                rest.push(parse_complex_selector_part_with_options(
                    input,
                    CssSelectorCombinator::Child,
                    options,
                    recovery,
                )?);
            }
            Ok(Token::Delim('+')) => {
                if after_pseudo_element {
                    return Err(recovery.invalid(state.site()));
                }
                rest.push(parse_complex_selector_part_with_options(
                    input,
                    CssSelectorCombinator::NextSibling,
                    options,
                    recovery,
                )?);
            }
            Ok(Token::Delim('~')) => {
                if after_pseudo_element {
                    return Err(recovery.invalid(state.site()));
                }
                rest.push(parse_complex_selector_part_with_options(
                    input,
                    CssSelectorCombinator::SubsequentSibling,
                    options,
                    recovery,
                )?);
            }
            Ok(Token::Delim('|')) if input.attempt(expect_adjacent_bar).is_ok() => {
                if after_pseudo_element {
                    return Err(recovery.invalid(state.site()));
                }
                rest.push(parse_complex_selector_part_with_options(
                    input,
                    CssSelectorCombinator::Column,
                    options,
                    recovery,
                )?);
            }
            Ok(_) if had_whitespace => {
                if after_pseudo_element {
                    return Err(recovery.invalid(state.site()));
                }
                input.restore(state);
                let selector =
                    parse_compound_selector_model_with_options(input, options, recovery)?;
                rest.push(CssComplexSelectorPart::new(
                    CssSelectorCombinator::Descendant,
                    selector,
                ));
            }
            Ok(token) => {
                input.restore(state);
                return Err(recovery.basic(unexpected(state.site(), token)));
            }
        }
    }

    if rest.is_empty() {
        Ok(compound_selector_to_selector(first))
    } else if crate::selector::selector_syntax::complex_selector_has_non_terminal_pseudo_elements(
        &first, &rest,
    ) {
        Err(recovery.invalid(input.site()))
    } else {
        // The parser has proved each compound under its own grammar/recovery
        // budget. Public construction's separate specified-output limit is not
        // a parser admission limit or a pseudo-element diagnostic.
        Ok(CssSelector::Complex(CssComplexSelector::new(first, rest)))
    }
}

fn parse_complex_selector_part_with_options<'i>(
    input: &mut Stream<'i>,
    combinator: CssSelectorCombinator,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssComplexSelectorPart> {
    consume_selector_whitespace(input, recovery)?;
    let selector = parse_compound_selector_model_with_options(input, options, recovery)?;
    Ok(CssComplexSelectorPart::new(combinator, selector))
}

pub(super) fn consume_selector_whitespace<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, bool> {
    let mut consumed = false;
    loop {
        let state = input.mark();
        match input.consume_token() {
            Ok(Token::Whitespace) => consumed = true,
            Ok(_) => {
                input.restore(state);
                return Ok(consumed);
            }
            Err(error) if error.is_end_of_stream() => {
                input.restore(state);
                return Ok(consumed);
            }
            Err(error) => return Err(recovery.basic(error)),
        }
    }
}

fn parse_type_selector<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, Option<CssQualifiedSelectorName>> {
    let start = input.mark();
    let first = match input.consume_token() {
        Ok(token) => token,
        Err(error) if error.is_end_of_stream() => {
            input.restore(start);
            return Ok(None);
        }
        Err(error) => return Err(recovery.basic(error)),
    };
    let (namespace, prefix, local_name) = match first {
        Token::Ident(prefix_or_name) => {
            let prefix_or_name = prefix_or_name.to_string();
            if consume_namespace_separator(input, recovery)? {
                let local_name = parse_qualified_local_name(input, recovery)?;
                // Namespaces 3 §3.3: the undeclared prefix is responsible.
                let Some(prefix) = recovery.named_namespace(&prefix_or_name) else {
                    return Err(recovery.invalid(start.site()));
                };
                (
                    CssNamespaceConstraint::Named(prefix.clone()),
                    CssQualifiedNamePrefix::Named(prefix),
                    local_name,
                )
            } else {
                (
                    recovery.unqualified_type_namespace(),
                    CssQualifiedNamePrefix::Unqualified,
                    Some(prefix_or_name),
                )
            }
        }
        Token::Delim('*') => {
            if consume_namespace_separator(input, recovery)? {
                (
                    CssNamespaceConstraint::Any,
                    CssQualifiedNamePrefix::Any,
                    parse_qualified_local_name(input, recovery)?,
                )
            } else {
                (
                    recovery.unqualified_type_namespace(),
                    CssQualifiedNamePrefix::Unqualified,
                    None,
                )
            }
        }
        Token::Delim('|') => (
            CssNamespaceConstraint::ExplicitNone,
            CssQualifiedNamePrefix::ExplicitNone,
            parse_qualified_local_name(input, recovery)?,
        ),
        _ => {
            input.restore(start);
            return Ok(None);
        }
    };
    Ok(Some(match local_name {
        Some(local_name) => {
            CssQualifiedSelectorName::new(namespace, prefix, CssIdent::new(local_name))
        }
        None => CssQualifiedSelectorName::universal(namespace, prefix),
    }))
}

fn parse_qualified_local_name<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, Option<String>> {
    // A token that is neither an identifier nor `*` cannot follow `|`.
    let location = input.site();
    match input.consume_token() {
        Ok(Token::Ident(name)) => Ok(Some(name.to_string())),
        // A star and an equals sign directly after it make an attribute matcher.
        Ok(Token::Delim('*')) if equals_follows(input) => {
            Err(recovery.basic(unexpected(location, Token::Delim('*'))))
        }
        Ok(Token::Delim('*')) => Ok(None),
        Ok(_) => Err(recovery.invalid(location)),
        Err(error) if error.is_end_of_stream() => Err(recovery.invalid(input.end_site())),
        Err(error) => Err(recovery.basic(error)),
    }
}

fn parse_compound_selector_model_with_options<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssCompoundSelector> {
    loop {
        let state = input.mark();
        match input.consume_token() {
            Ok(Token::Whitespace) => continue,
            Ok(_) => {
                input.restore(state);
                break;
            }
            Err(error) if error.is_end_of_stream() => {
                input.restore(state);
                break;
            }
            Err(error) => return Err(recovery.basic(error)),
        }
    }

    let type_selector = parse_type_selector(input, recovery)?;
    let mut members = CssCompoundMembers::default();
    let mut pseudo_elements = None;

    loop {
        let state = input.mark();
        match input.consume_token() {
            Ok(Token::Whitespace | Token::Comma) => {
                input.restore(state);
                break;
            }
            Ok(_) => input.restore(state),
            Err(error) if error.is_end_of_stream() => {
                consume_remaining(input);
                break;
            }
            Err(error) => return Err(recovery.basic(error)),
        }

        if pseudo_elements.is_some() {
            return Err(recovery.invalid(input.site()));
        }

        if input.attempt(|input| input.consume_delim('&')).is_ok() {
            match options.anchors {
                SelectorAnchorMode::Nesting => members.push_nesting(),
                SelectorAnchorMode::Scope => members.push_scope_anchor(),
            }
            continue;
        }

        if input.attempt(|input| input.consume_delim('.')).is_ok() {
            members.push_class(parse_adjacent_class_name(input, recovery)?);
            continue;
        }

        if input.attempt(expect_open_square).is_ok() {
            let attribute =
                input.within_block(|input| parse_attribute_selector(input, recovery))?;
            members.push_attribute(attribute);
            continue;
        }

        if input.attempt(Stream::consume_colon).is_ok() {
            if input.attempt(expect_adjacent_colon).is_ok() {
                if !options.allow_pseudo_elements {
                    return Err(recovery.invalid(input.site()));
                }
                let sequence = parse_pseudo_element_sequence(input, options, recovery)?;
                pseudo_elements = Some(sequence);
            } else if let Ok(first) = input.attempt(parse_legacy_pseudo_element) {
                if !options.allow_pseudo_elements {
                    return Err(recovery.invalid(input.site()));
                }
                pseudo_elements = Some(parse_pseudo_element_sequence_from_first(
                    input, first, options, recovery,
                )?);
            } else {
                let pseudo_class = parse_pseudo_class_with_options(input, options, recovery)?;
                members.push_pseudo_class(pseudo_class);
            }
            continue;
        }

        let state = input.mark();
        match input.consume_after_whitespace() {
            Ok(Token::Hash {
                value: key,
                kind: HashKind::Id,
            }) => {
                members.push_id(key.to_string());
            }
            Ok(Token::Delim('|')) => {
                // Two bars make the column combinator, unless the second one
                // and the equals sign after it make an attribute matcher.
                if input.attempt(expect_adjacent_bar).is_ok() && !equals_follows(input) {
                    input.restore(state);
                    break;
                }
                // A lone `|` cannot continue the compound.
                return Err(recovery.invalid(state.site()));
            }
            Ok(token) => {
                input.restore(state);
                if type_selector.is_none() && members.is_empty() && pseudo_elements.is_none() {
                    return Err(recovery.basic(unexpected(state.site(), token)));
                }
                break;
            }
            Err(error) if error.is_end_of_stream() => {
                consume_remaining(input);
                break;
            }
            Err(error) => return Err(recovery.basic(error)),
        }
    }

    if type_selector.is_none() && members.is_empty() && pseudo_elements.is_none() {
        return Err(recovery.invalid(input.site()));
    }
    Ok(members.into_compound(type_selector, pseudo_elements))
}

fn compound_selector_to_selector(selector: CssCompoundSelector) -> CssSelector {
    if selector.has_scope_anchor()
        || selector.has_pseudo_elements()
        || selector.nesting_selectors() > 0
    {
        return CssSelector::Compound(selector);
    }

    if let (None, [], [class], [], []) = (
        selector.type_selector(),
        selector.ids(),
        selector.classes(),
        selector.attributes(),
        selector.pseudo_classes(),
    ) {
        return CssSelector::Class(class.clone());
    }
    if selector.has_unqualified_any_namespace()
        && let (Some(tag), [], [], [], []) = (
            selector
                .type_selector()
                .and_then(CssQualifiedSelectorName::local_name),
            selector.ids(),
            selector.classes(),
            selector.attributes(),
            selector.pseudo_classes(),
        )
    {
        return CssSelector::Tag(tag.to_owned());
    }
    if let (None, [key], [], [], []) = (
        selector.type_selector(),
        selector.ids(),
        selector.classes(),
        selector.attributes(),
        selector.pseudo_classes(),
    ) {
        return CssSelector::Key(key.clone());
    }
    if let (None, [], [], [], [pseudo_class]) = (
        selector.type_selector(),
        selector.ids(),
        selector.classes(),
        selector.attributes(),
        selector.pseudo_classes(),
    ) {
        return CssSelector::PseudoClass(pseudo_class.clone());
    }
    CssSelector::Compound(selector)
}

/// Consumes the next token, whitespace included, when it is `[`.
fn expect_open_square<'i>(input: &mut Stream<'i>) -> Result<(), ValueError<'i>> {
    input.discard_whitespace();
    let at = input.site();
    match input.consume_token()? {
        Token::OpenSquare => Ok(()),
        token => Err(unexpected(at, token)),
    }
}

fn expect_adjacent_colon<'i>(input: &mut Stream<'i>) -> Result<(), ValueError<'i>> {
    let at = input.site();
    match input.consume_token()? {
        Token::Colon => Ok(()),
        token => Err(unexpected(at, token)),
    }
}

/// Whether an equals sign is the next token with nothing between it and the
/// token consumed last: the second half of an attribute matcher.
fn equals_follows(input: &Stream<'_>) -> bool {
    let site = input.site();
    let document = site.document();
    document
        .list(site.list())
        .get(site.index())
        .is_some_and(|node| matches!(document.token(*node), Token::Delim('=')))
}

fn expect_adjacent_bar<'i>(input: &mut Stream<'i>) -> Result<(), ValueError<'i>> {
    let at = input.site();
    match input.consume_token()? {
        Token::Delim('|') => Ok(()),
        token => Err(unexpected(at, token)),
    }
}

/// A namespace separator is one adjacent bar, rather than the two-token column
/// combinator. Leave both column tokens for the complex-selector provider.
fn consume_namespace_separator<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, bool> {
    let start = input.mark();
    match input.consume_token() {
        Ok(Token::Delim('|')) => {
            let after_bar = input.mark();
            // Two bars make the column combinator, unless the second one and
            // the equals sign after it make an attribute matcher.
            let column = input.attempt(expect_adjacent_bar).is_ok() && !equals_follows(input);
            input.restore(if column { start } else { after_bar });
            Ok(!column)
        }
        Ok(_) => {
            input.restore(start);
            Ok(false)
        }
        Err(error) if error.is_end_of_stream() => {
            input.restore(start);
            Ok(false)
        }
        Err(error) => Err(recovery.basic(error)),
    }
}

fn parse_adjacent_class_name<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, String> {
    loop {
        let start = input.mark();
        match input.consume_token_with_comments() {
            Ok(Token::Comment) => continue,
            Ok(Token::Ident(value)) => return Ok(value.to_string()),
            Ok(token) => {
                input.restore(start);
                return Err(recovery.basic(unexpected(input.site(), token)));
            }
            Err(error) => return Err(recovery.basic(error)),
        }
    }
}

fn parse_pseudo_element_sequence<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssPseudoElementSequence> {
    let first = parse_pseudo_element(input, options, recovery)?;
    parse_pseudo_element_sequence_from_first(input, first, options, recovery)
}

fn parse_pseudo_element_sequence_from_first<'i>(
    input: &mut Stream<'i>,
    first: CssPseudoElement,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssPseudoElementSequence> {
    let mut suffix = first.suffix_context();
    let mut segments = vec![CssPseudoElementSegment::PseudoElement(first)];
    while input.attempt(expect_adjacent_colon).is_ok() {
        let element = if input.attempt(expect_adjacent_colon).is_ok() {
            Some(parse_pseudo_element(input, options, recovery)?)
        } else {
            input.attempt(parse_legacy_pseudo_element).ok()
        };
        if let Some(element) = element {
            suffix = element.suffix_context();
            segments.push(CssPseudoElementSegment::PseudoElement(element));
        } else {
            let pseudo = parse_pseudo_class_with_options(
                input,
                SelectorParseOptions {
                    pseudo_suffix: Some(suffix),
                    ..options
                },
                recovery,
            )?;
            segments.push(CssPseudoElementSegment::PseudoClass(pseudo));
        }
    }
    CssPseudoElementSequence::try_from_segments(segments)
        .ok_or_else(|| recovery.invalid(input.site()))
}

fn parse_legacy_pseudo_element<'i>(
    input: &mut Stream<'i>,
) -> Result<CssPseudoElement, ValueError<'i>> {
    let at = input.site();
    let name = match input.consume_token()? {
        Token::Ident(name) => name,
        token => return Err(unexpected(at, token)),
    };
    match_ignore_ascii_case! { name,
        "before" => Ok(CssPseudoElement::Before),
        "after" => Ok(CssPseudoElement::After),
        "first-line" => Ok(CssPseudoElement::FirstLine),
        "first-letter" => Ok(CssPseudoElement::FirstLetter),
        _ => Err(unexpected(at, Token::Ident(name))),
    }
}

fn parse_pseudo_element<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssPseudoElement> {
    let state = input.mark();
    match input.consume_token() {
        Ok(Token::Ident(name)) => match_ignore_ascii_case! { name,
            "before" => Ok(CssPseudoElement::Before),
            "after" => Ok(CssPseudoElement::After),
            "first-line" => Ok(CssPseudoElement::FirstLine),
            "first-letter" => Ok(CssPseudoElement::FirstLetter),
            "prefix" => Ok(CssPseudoElement::Prefix),
            "suffix" => Ok(CssPseudoElement::Suffix),
            "marker" => Ok(CssPseudoElement::Marker),
            "selection" => Ok(CssPseudoElement::Selection),
            "search-text" => Ok(CssPseudoElement::SearchText),
            "target-text" => Ok(CssPseudoElement::TargetText),
            "spelling-error" => Ok(CssPseudoElement::SpellingError),
            "grammar-error" => Ok(CssPseudoElement::GrammarError),
            "placeholder" => Ok(CssPseudoElement::Placeholder),
            "backdrop" => Ok(CssPseudoElement::Backdrop),
            "file-selector-button" => Ok(CssPseudoElement::FileSelectorButton),
            "details-content" => Ok(CssPseudoElement::DetailsContent),
            "view-transition" => Ok(CssPseudoElement::ViewTransition),
            _ => {
                if let Ok(name) = CssUnknownWebkitPseudoElement::try_new(name.to_string()) {
                    return Ok(CssPseudoElement::UnknownWebkit(name));
                }
                input.restore(state);
                Err(recovery.invalid(input.site()))
            }
        },
        Ok(Token::Function(name))
            if name.eq_ignore_ascii_case("slotted")
                || name.eq_ignore_ascii_case("part")
                || name.eq_ignore_ascii_case("highlight")
                || name.eq_ignore_ascii_case("view-transition-group")
                || name.eq_ignore_ascii_case("view-transition-image-pair")
                || name.eq_ignore_ascii_case("view-transition-old")
                || name.eq_ignore_ascii_case("view-transition-new") =>
        {
            let mut depth = recovery
                .state
                .enter_component_block_at(
                    recovery.source,
                    opening_end(state.site()),
                    "baseline.selector.complex",
                )
                .map_err(|error| SyntaxError::custom(state.site(), error))?;
            let result = input.within_block(|input| {
                if name.eq_ignore_ascii_case("slotted") {
                    parse_compound_argument(input, options, recovery).map(CssPseudoElement::Slotted)
                } else if name.eq_ignore_ascii_case("part") {
                    parse_part_names(input, recovery).map(CssPseudoElement::Part)
                } else if name.eq_ignore_ascii_case("highlight") {
                    parse_highlight_name(input, recovery).map(CssPseudoElement::Highlight)
                } else {
                    let argument = parse_view_transition_name_selector(input, recovery)?;
                    match_ignore_ascii_case! { name,
                        "view-transition-group" => Ok(CssPseudoElement::ViewTransitionGroup(argument)),
                        "view-transition-image-pair" => Ok(CssPseudoElement::ViewTransitionImagePair(argument)),
                        "view-transition-old" => Ok(CssPseudoElement::ViewTransitionOld(argument)),
                        "view-transition-new" => Ok(CssPseudoElement::ViewTransitionNew(argument)),
                        _ => Err(recovery.invalid(input.site())),
                    }
                }
            });
            if result.is_ok() {
                depth.retain();
            }
            result
        }
        Ok(_) => {
            input.restore(state);
            Err(recovery.invalid(input.site()))
        }
        Err(error) if error.is_end_of_stream() => Err(recovery.invalid(input.end_site())),
        Err(error) => Err(recovery.basic(error)),
    }
}

// Comments are absent from CSS grammar, but actual whitespace cannot separate
// the two components of an attribute matcher.
fn expect_adjacent_attribute_equals<'i>(input: &mut Stream<'i>) -> Result<(), ValueError<'i>> {
    loop {
        let location = input.site();
        match input.consume_token_with_comments()? {
            Token::Comment => continue,
            Token::Delim('=') => return Ok(()),
            token => return Err(unexpected(location, token)),
        }
    }
}

fn parse_attribute_selector<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssAttributeSelector> {
    consume_selector_whitespace(input, recovery)?;
    let name = parse_attribute_selector_name(input, recovery)?;

    // Each rejection below names the consumed token that cannot continue the
    // attribute selector, not the token after it.
    let matcher_location = input.site();
    let matcher = match input.consume_after_whitespace() {
        Err(error) if error.is_end_of_stream() => {
            return Ok(CssAttributeSelector::new_qualified(
                name,
                CssAttributeMatcher::Exists,
                CssAttributeCaseSensitivity::DocumentDefault,
            ));
        }
        Err(error) => return Err(recovery.basic(error)),
        Ok(Token::Delim('=')) => {
            CssAttributeMatcher::Equals(parse_attribute_selector_value(input, recovery)?)
        }
        // The two-character matchers are two delimiters: Syntax 3 §4.3.1 has
        // no token for them.
        Ok(Token::Delim('~')) => {
            expect_adjacent_attribute_equals(input).map_err(|error| recovery.basic(error))?;
            CssAttributeMatcher::Includes(parse_attribute_selector_value(input, recovery)?)
        }
        Ok(Token::Delim('|')) => {
            expect_adjacent_attribute_equals(input).map_err(|error| recovery.basic(error))?;
            CssAttributeMatcher::DashMatch(parse_attribute_selector_value(input, recovery)?)
        }
        Ok(Token::Delim('^')) => {
            expect_adjacent_attribute_equals(input).map_err(|error| recovery.basic(error))?;
            CssAttributeMatcher::Prefix(parse_attribute_selector_value(input, recovery)?)
        }
        Ok(Token::Delim('$')) => {
            expect_adjacent_attribute_equals(input).map_err(|error| recovery.basic(error))?;
            CssAttributeMatcher::Suffix(parse_attribute_selector_value(input, recovery)?)
        }
        Ok(Token::Delim('*')) => {
            expect_adjacent_attribute_equals(input).map_err(|error| recovery.basic(error))?;
            CssAttributeMatcher::Substring(parse_attribute_selector_value(input, recovery)?)
        }
        Ok(_) => {
            return Err(recovery.invalid(matcher_location));
        }
    };

    let case_sensitivity = parse_attribute_case_sensitivity(input, recovery)?;
    expect_exhausted(input, recovery)?;
    Ok(CssAttributeSelector::new_qualified(
        name,
        matcher,
        case_sensitivity,
    ))
}

fn parse_attribute_selector_name<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssQualifiedAttributeName> {
    let prefix_location = input.site();
    match input.consume_token() {
        Ok(Token::Ident(prefix_or_name)) => {
            let prefix_or_name = prefix_or_name.to_string();
            let after_ident = input.mark();
            match input.consume_token() {
                Ok(Token::Delim('|')) => {
                    // A split |= matcher belongs to this unqualified name;
                    // a namespace separator instead requires an adjacent ident.
                    if input.attempt(expect_adjacent_attribute_equals).is_ok() {
                        input.restore(after_ident);
                        return Ok(CssQualifiedAttributeName::new(
                            CssNamespaceConstraint::ExplicitNone,
                            CssQualifiedNamePrefix::Unqualified,
                            CssAttributeName::new(prefix_or_name),
                        ));
                    }
                    let local_location = input.site();
                    let local_name = input.consume_token();
                    let local_name = match local_name {
                        Ok(Token::Ident(name)) => name.to_string(),
                        Ok(_) => {
                            return Err(recovery.invalid(local_location));
                        }
                        Err(error) if error.is_end_of_stream() => {
                            return Err(recovery.invalid(input.end_site()));
                        }
                        Err(error) => return Err(recovery.basic(error)),
                    };
                    let Some(prefix) = recovery.named_namespace(&prefix_or_name) else {
                        return Err(recovery.invalid(prefix_location));
                    };
                    Ok(CssQualifiedAttributeName::new(
                        CssNamespaceConstraint::Named(prefix.clone()),
                        CssQualifiedNamePrefix::Named(prefix),
                        CssAttributeName::new(local_name),
                    ))
                }
                Ok(_) => {
                    input.restore(after_ident);
                    Ok(CssQualifiedAttributeName::new(
                        CssNamespaceConstraint::ExplicitNone,
                        CssQualifiedNamePrefix::Unqualified,
                        CssAttributeName::new(prefix_or_name),
                    ))
                }
                Err(error) if error.is_end_of_stream() => {
                    input.restore(after_ident);
                    Ok(CssQualifiedAttributeName::new(
                        CssNamespaceConstraint::ExplicitNone,
                        CssQualifiedNamePrefix::Unqualified,
                        CssAttributeName::new(prefix_or_name),
                    ))
                }
                Err(error) => Err(recovery.basic(error)),
            }
        }
        Ok(Token::Delim('*')) => {
            let bar_location = input.site();
            match input.consume_token() {
                Ok(Token::Delim('|')) => {}
                Ok(_) => {
                    return Err(recovery.invalid(bar_location));
                }
                Err(error) if error.is_end_of_stream() => {
                    return Err(recovery.invalid(input.end_site()));
                }
                Err(error) => return Err(recovery.basic(error)),
            }
            let name_location = input.site();
            let name = input.consume_token();
            match name {
                Ok(Token::Ident(name)) => Ok(CssQualifiedAttributeName::new(
                    CssNamespaceConstraint::Any,
                    CssQualifiedNamePrefix::Any,
                    CssAttributeName::new(name.to_string()),
                )),
                Ok(_) => Err(recovery.invalid(name_location)),
                Err(error) if error.is_end_of_stream() => Err(recovery.invalid(input.end_site())),
                Err(error) => Err(recovery.basic(error)),
            }
        }
        Ok(Token::Delim('|')) => {
            let name_location = input.site();
            match input.consume_token() {
                Ok(Token::Ident(name)) => Ok(CssQualifiedAttributeName::new(
                    CssNamespaceConstraint::ExplicitNone,
                    CssQualifiedNamePrefix::ExplicitNone,
                    CssAttributeName::new(name.to_string()),
                )),
                Ok(_) => Err(recovery.invalid(name_location)),
                Err(error) if error.is_end_of_stream() => Err(recovery.invalid(input.end_site())),
                Err(error) => Err(recovery.basic(error)),
            }
        }
        Ok(_) => Err(recovery.invalid(prefix_location)),
        Err(error) if error.is_end_of_stream() => Err(recovery.invalid(input.end_site())),
        Err(error) => Err(recovery.basic(error)),
    }
}

fn parse_attribute_selector_value<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, String> {
    let at = input.site();
    match input.consume_after_whitespace() {
        Ok(Token::Ident(value) | Token::String(value)) => Ok(value.to_string()),
        Ok(token) => Err(recovery.basic(unexpected(at, token))),
        Err(error) => Err(recovery.basic(error)),
    }
}

fn parse_attribute_case_sensitivity<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssAttributeCaseSensitivity> {
    let state = input.mark();
    let location = input.site();
    match input.consume_after_whitespace() {
        Err(error) if error.is_end_of_stream() => {
            input.restore(state);
            Ok(CssAttributeCaseSensitivity::DocumentDefault)
        }
        Err(error) => Err(recovery.basic(error)),
        Ok(Token::Ident(modifier)) if modifier.eq_ignore_ascii_case("i") => {
            Ok(CssAttributeCaseSensitivity::AsciiCaseInsensitive)
        }
        Ok(Token::Ident(modifier)) if modifier.eq_ignore_ascii_case("s") => {
            Ok(CssAttributeCaseSensitivity::ExplicitSensitive)
        }
        // The modifier itself cannot continue the attribute selector.
        Ok(_) => Err(recovery.invalid(location)),
    }
}

fn parse_pseudo_class_with_options<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssPseudoClass> {
    let state = input.mark();
    match input.consume_token() {
        Ok(Token::Ident(name)) => {
            let pseudo = parse_named_pseudo_class(name, input, state, recovery)?;
            if options.grammar().admits_pseudo(&pseudo) {
                Ok(pseudo)
            } else {
                input.restore(state);
                Err(recovery.invalid(input.site()))
            }
        }
        Ok(Token::Function(name)) => {
            let mut depth = recovery
                .state
                .enter_component_block_at(
                    recovery.source,
                    opening_end(state.site()),
                    "baseline.selector.complex",
                )
                .map_err(|error| SyntaxError::custom(state.site(), error))?;
            let result = input.within_block(|input| {
                parse_function_pseudo_class(name, state.site(), input, options, recovery)
            });
            if result.is_ok() {
                depth.retain();
            }
            result
        }
        Ok(_) => {
            input.restore(state);
            Err(recovery.invalid(input.site()))
        }
        Err(error) if error.is_end_of_stream() => Err(recovery.invalid(input.end_site())),
        Err(error) => Err(recovery.basic(error)),
    }
}

#[inline(never)]
fn parse_named_pseudo_class<'i>(
    name: &str,
    input: &mut Stream<'i>,
    name_start: Mark<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssPseudoClass> {
    match_ignore_ascii_case! { name,
        "host" => Ok(CssPseudoClass::Host),
        "root" => Ok(CssPseudoClass::Root),
        "scope" => Ok(CssPseudoClass::Scope),
        "defined" => Ok(CssPseudoClass::Defined),
        "any-link" => Ok(CssPseudoClass::AnyLink),
        "link" => Ok(CssPseudoClass::Link),
        "visited" => Ok(CssPseudoClass::Visited),
        "target" => Ok(CssPseudoClass::Target),
        "current" => Ok(CssPseudoClass::Current),
        "hover" => Ok(CssPseudoClass::Hover),
        "active" => Ok(CssPseudoClass::Active),
        "focus" => Ok(CssPseudoClass::Focus),
        "focus-visible" => Ok(CssPseudoClass::FocusVisible),
        "focus-within" => Ok(CssPseudoClass::FocusWithin),
        "disabled" => Ok(CssPseudoClass::Disabled),
        "enabled" => Ok(CssPseudoClass::Enabled),
        "checked" => Ok(CssPseudoClass::Checked),
        "unchecked" => Ok(CssPseudoClass::Unchecked),
        "required" => Ok(CssPseudoClass::Required),
        "optional" => Ok(CssPseudoClass::Optional),
        "valid" => Ok(CssPseudoClass::Valid),
        "invalid" => Ok(CssPseudoClass::Invalid),
        "user-valid" => Ok(CssPseudoClass::UserValid),
        "user-invalid" => Ok(CssPseudoClass::UserInvalid),
        "placeholder-shown" => Ok(CssPseudoClass::PlaceholderShown),
        "autofill" | "-webkit-autofill" => Ok(CssPseudoClass::Autofill),
        "first-child" => Ok(CssPseudoClass::FirstChild),
        "last-child" => Ok(CssPseudoClass::LastChild),
        "only-child" => Ok(CssPseudoClass::OnlyChild),
        "empty" => Ok(CssPseudoClass::Empty),
        "first-of-type" => Ok(CssPseudoClass::FirstOfType),
        "last-of-type" => Ok(CssPseudoClass::LastOfType),
        "only-of-type" => Ok(CssPseudoClass::OnlyOfType),
        "playing" => Ok(CssPseudoClass::Playing),
        "paused" => Ok(CssPseudoClass::Paused),
        "seeking" => Ok(CssPseudoClass::Seeking),
        "buffering" => Ok(CssPseudoClass::Buffering),
        "stalled" => Ok(CssPseudoClass::Stalled),
        "muted" => Ok(CssPseudoClass::Muted),
        "volume-locked" => Ok(CssPseudoClass::VolumeLocked),
        "open" => Ok(CssPseudoClass::Open),
        "modal" => Ok(CssPseudoClass::Modal),
        "fullscreen" => Ok(CssPseudoClass::Fullscreen),
        "picture-in-picture" => Ok(CssPseudoClass::PictureInPicture),
        "popover-open" => Ok(CssPseudoClass::PopoverOpen),
        "default" => Ok(CssPseudoClass::Default),
        "indeterminate" => Ok(CssPseudoClass::Indeterminate),
        "read-only" => Ok(CssPseudoClass::ReadOnly),
        "read-write" => Ok(CssPseudoClass::ReadWrite),
        "in-range" => Ok(CssPseudoClass::InRange),
        "out-of-range" => Ok(CssPseudoClass::OutOfRange),
        _ => {
            input.restore(name_start);
            Err(recovery.invalid(input.site()))
        },
    }
}

#[inline(never)]
fn parse_function_pseudo_class<'i>(
    name: &str,
    name_start: Site<'i>,
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssPseudoClass> {
    let arguments = options.independent_arguments();
    let pseudo_class = match_ignore_ascii_case! { name,
        "host" => CssPseudoClass::HostFunction(parse_compound_argument(input, arguments, recovery)?),
        "host-context" => CssPseudoClass::HostContext(parse_compound_argument(input, arguments, recovery)?),
        "nth-child" => CssPseudoClass::NthChild(parse_nth_child_pattern(input, arguments, recovery)?),
        "nth-last-child" => CssPseudoClass::NthLastChild(parse_nth_child_pattern(input, arguments, recovery)?),
        "nth-of-type" => CssPseudoClass::NthOfType(parse_nth_pattern(input, recovery)?),
        "nth-last-of-type" => CssPseudoClass::NthLastOfType(parse_nth_pattern(input, recovery)?),
        "dir" => CssPseudoClass::Dir(parse_directionality(input, recovery)?),
        "lang" => CssPseudoClass::Lang(parse_language_ranges(input, recovery)?),
        "not" => CssPseudoClass::Not(parse_pseudo_selector_list_with_options(input, options.without_pseudo_elements(), recovery)?),
        "is" => CssPseudoClass::Is(parse_forgiving_pseudo_selector_list(input, options.without_pseudo_elements(), recovery)?),
        "where" => CssPseudoClass::Where(parse_forgiving_pseudo_selector_list(input, options.without_pseudo_elements(), recovery)?),
        "has" if options.allow_has => CssPseudoClass::Has(parse_has_relative_selector_list(input, arguments, recovery)?),
        "has" => return Err(recovery.invalid(input.site())),
        _ => return Err(recovery.invalid(name_start)),
    };
    expect_exhausted(input, recovery)?;
    Ok(pseudo_class)
}

fn parse_compound_argument<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssCompoundSelectorArgument> {
    let options = SelectorParseOptions {
        compound_only: true,
        pseudo_suffix: None,
        ..options.without_pseudo_elements()
    };
    let selector = parse_rule_selector_with_options(input, options, recovery)?;
    expect_exhausted(input, recovery)?;
    CssCompoundSelectorArgument::try_new(selector).ok_or_else(|| recovery.invalid(input.site()))
}

fn parse_part_names<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssPartNameList> {
    let mut names = Vec::new();
    loop {
        input.discard_whitespace();
        if input.only_whitespace_remains() {
            break;
        }
        let start = input.mark();
        input
            .consume_ident()
            .map_err(|error| recovery.basic(error))?;
        input.restore(start);
        let name = input
            .collect_component()
            .map(std::borrow::Cow::into_owned)
            .and_then(CssPartName::from_component)
            .map_err(|error| recovery.component(start.site(), error))?;
        names.push(name);
    }
    CssPartNameList::try_new(names).ok_or_else(|| recovery.invalid(input.site()))
}

fn parse_view_transition_name_selector<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssViewTransitionNameSelector> {
    input.discard_whitespace();
    let start = input.site();
    let argument = match input
        .consume_after_whitespace()
        .map_err(|error| recovery.basic(error))?
    {
        Token::Delim('*') => CssViewTransitionNameSelector::Wildcard,
        Token::Ident(name) => CssViewTransitionNameSelector::Name(
            CssCustomIdent::try_new(name.to_string()).ok_or_else(|| recovery.invalid(start))?,
        ),
        _ => return Err(recovery.invalid(start)),
    };
    expect_exhausted(input, recovery)?;
    Ok(argument)
}

fn parse_highlight_name<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssCustomIdent> {
    input.discard_whitespace();
    let start = input.site();
    let name = input
        .consume_ident()
        .map_err(|error| recovery.basic(error))?
        .to_string();
    let name = CssCustomIdent::try_new(name).ok_or_else(|| recovery.invalid(start))?;
    expect_exhausted(input, recovery)?;
    Ok(name)
}

fn parse_directionality<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssDirectionality> {
    input.discard_whitespace();
    let start = input.mark();
    input
        .consume_ident()
        .map_err(|error| recovery.basic(error))?;
    input.restore(start);
    input
        .collect_component()
        .map(std::borrow::Cow::into_owned)
        .and_then(CssDirectionality::from_component)
        .map_err(|error| recovery.component(start.site(), error))
}

fn parse_language_ranges<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssLanguageRangeList> {
    let mut ranges = Vec::new();
    loop {
        input.discard_whitespace();
        let start = input.mark();
        let at = input.site();
        match input.consume_after_whitespace() {
            Ok(Token::Ident(_) | Token::String(_)) => {}
            Ok(token) => return Err(recovery.basic(unexpected(at, token))),
            Err(error) => return Err(recovery.basic(error)),
        }
        input.restore(start);
        let range = input
            .collect_component()
            .map(std::borrow::Cow::into_owned)
            .and_then(CssLanguageRange::from_component)
            .map_err(|error| recovery.component(start.site(), error))?;
        ranges.push(range);
        if input.attempt(Stream::consume_comma).is_err() {
            break;
        }
    }
    CssLanguageRangeList::try_new(ranges).map_err(|_| recovery.invalid(input.site()))
}

fn parse_pseudo_selector_list_with_options<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssPseudoSelectorList> {
    let selectors = parse_pseudo_selector_list_items_with_options(input, options, recovery)?;
    // This parser has proved each member in its actual containing context and
    // charged the shared parse budget. Programmatic output limits are separate.
    Ok(CssPseudoSelectorList::from_parsed_items(selectors))
}

fn capacity_overflow<'i>(recovery: &SelectorRecovery<'_>, at: Site<'i>) -> ValueError<'i> {
    recovery.component(
        at,
        crate::component::CssComponentValueError::new(
            crate::component::values::CssComponentValueErrorKind::CapacityOverflow,
            crate::component::CssValueOrigin::Programmatic,
        ),
    )
}

fn parse_pseudo_selector_list_items_with_options<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, Vec<CssPseudoSelectorListItem>> {
    let mut selectors = Vec::new();
    loop {
        let selector = parse_rule_selector_with_options(input, options, recovery)?;
        selectors
            .try_reserve(1)
            .map_err(|_| capacity_overflow(recovery, input.site()))?;
        selectors.push(CssPseudoSelectorListItem::Selector(selector));
        if input.attempt(Stream::consume_comma).is_err() {
            break;
        }
    }
    expect_exhausted(input, recovery)?;
    Ok(selectors)
}

/// Whether the failure is a selector that is not valid, rather than a fault of
/// the text or a resource.
fn is_invalid_selector(error: &ValueError<'_>) -> bool {
    matches!(
        error.custom_value().map(|error| &error.kind),
        Some(cssparser::ParseErrorKind::Custom(error))
            if matches!(error.kind(), crate::diagnostic::error::ErrorKind::InvalidSelector(_))
    )
}

fn parse_forgiving_pseudo_selector_list<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssPseudoSelectorList> {
    recovery.check_forgiving_envelope(input)?;
    let mut items = Vec::new();
    let mut preceding_comma = None;
    loop {
        let member_start = offset_of(input.site());
        let (result, invalid_components) = input.within_list_until(StopToken::Comma, |member| {
            let start = member.mark();
            let result = parse_rule_selector_with_options(member, options, recovery);
            let components = match result {
                Ok(_) => None,
                Err(ref error) if is_invalid_selector(error) => {
                    member.restore(start);
                    let components = member
                        .remaining_components()
                        .map_err(|error| recovery.component(start.site(), error));
                    consume_remaining(member);
                    Some(components?)
                }
                Err(error) => return Err(error),
            };
            Ok((result, components))
        })?;
        let member_end = offset_of(input.site());
        let comma_start = member_end;
        let following_comma = match input.consume_after_whitespace() {
            Ok(Token::Comma) => Some((comma_start, offset_of(input.site()))),
            Err(error) if error.is_end_of_stream() => None,
            Ok(_) => return Err(recovery.invalid(input.site())),
            Err(error) => return Err(recovery.basic(error)),
        };

        let item = match result {
            Ok(selector) => Some(CssPseudoSelectorListItem::Selector(selector)),
            Err(error) => {
                let components = invalid_components.expect("failed member components");
                let error = native(&recovery.numeric(), &error);
                let location = error.location;
                let contains_nesting = components.contains_delimiter('&').map_err(|error| {
                    SyntaxError::custom(input.site(), selector_component_error(location, error))
                })?;
                let action = if contains_nesting {
                    crate::diagnostic::CssRecoveryAction::PreserveInvalidSelectorListItem
                } else {
                    crate::diagnostic::CssRecoveryAction::DropSelectorListItem
                };
                recovery.diagnose_forgiving_member(
                    error,
                    member_start,
                    member_end,
                    following_comma,
                    preceding_comma,
                    action,
                );
                if contains_nesting {
                    let origin = crate::source::CssParsedOrigin::from_range(
                        recovery.state.source_snapshot(),
                        member_start..member_end,
                    )
                    .expect("parser proved original member boundaries");
                    Some(CssPseudoSelectorListItem::InvalidNesting(
                        CssInvalidNestingSelectorItem::new(origin, components, options.grammar()),
                    ))
                } else {
                    None
                }
            }
        };
        if let Some(item) = item {
            items
                .try_reserve(1)
                .map_err(|_| capacity_overflow(recovery, input.site()))?;
            items.push(item);
        }
        let Some(comma) = following_comma else {
            break;
        };
        // Selectors 4 "parse a list" splits with Syntax 3 (2026 CRD) §5.4.10.
        if at_final_comma(input) {
            break;
        }
        preceding_comma = Some(comma);
    }
    Ok(CssPseudoSelectorList::from_parsed_items(items))
}

fn parse_has_relative_selector_list<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssRelativeSelectorList> {
    parse_relative_selector_list_with_options(
        input,
        options.without_nested_has().without_pseudo_elements(),
        recovery,
    )
}

pub(super) fn parse_general_relative_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssRelativeSelectorList, ParseError<'i, CssError>> {
    recovery.read(input, |input, recovery| {
        recovery.check_depth(input)?;
        parse_relative_selector_list_with_options(input, SelectorParseOptions::standard(), recovery)
    })
}

fn parse_relative_selector_list_with_options<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssRelativeSelectorList> {
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_relative_selector_with_options(
            input, options, recovery,
        )?);
        if input.attempt(Stream::consume_comma).is_err() {
            break;
        }
    }
    expect_exhausted(input, recovery)?;
    // The caller's grammar and parse budget are already checked; avoid applying
    // an unrelated default specified-output budget or losing parse provenance.
    Ok(CssRelativeSelectorList::new(selectors))
}

fn parse_relative_selector_with_options<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssRelativeSelector> {
    match parse_style_selector_with_options(input, options, recovery)? {
        CssStyleSelector::Selector(selector) => Ok(CssRelativeSelector::new(
            CssSelectorCombinator::Descendant,
            selector,
        )),
        CssStyleSelector::Relative(selector) => Ok(selector),
    }
}

fn parse_selector_after_leading_combinator_with_options<'i>(
    input: &mut Stream<'i>,
    combinator: CssSelectorCombinator,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssRelativeSelector> {
    consume_selector_whitespace(input, recovery)?;
    let first = parse_compound_selector_model_with_options(input, options, recovery)?;
    let selector = parse_selector_after_first_compound(input, first, options, recovery)?;
    Ok(CssRelativeSelector::new(combinator, selector))
}

fn parse_nth_child_pattern<'i>(
    input: &mut Stream<'i>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Value<'i, CssNthChildPattern> {
    let pattern = parse_nth_pattern(input, recovery)?;
    let state = input.mark();
    match input.consume_after_whitespace() {
        Ok(Token::Ident(value)) if value.eq_ignore_ascii_case("of") => {
            let selector_list = parse_pseudo_selector_list_with_options(
                input,
                options.without_pseudo_elements(),
                recovery,
            )?;
            Ok(CssNthChildPattern::new(pattern, Some(selector_list)))
        }
        Ok(_) => {
            input.restore(state);
            Ok(CssNthChildPattern::new(pattern, None))
        }
        Err(error) if error.is_end_of_stream() => {
            input.restore(state);
            Ok(CssNthChildPattern::new(pattern, None))
        }
        Err(error) => Err(recovery.basic(error)),
    }
}

fn parse_nth_pattern<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Value<'i, CssNthPattern> {
    if input.attempt(|input| input.consume_keyword("odd")).is_ok() {
        return Ok(CssNthPattern::Odd);
    }
    if input.attempt(|input| input.consume_keyword("even")).is_ok() {
        return Ok(CssNthPattern::Even);
    }

    // A value outside the range of an integer is the nearest integer.
    let notation =
        anb::parse_saturating(input).map_err(|error| match error.into_stream_error() {
            Ok(error) => match error.unexpected_token() {
                // The token that does not fit is responsible where it ends.
                Some(Token::Whitespace) => {
                    // The whitespace is named as it was written, and the failure
                    // is where it ends.
                    recovery.basic_whitespace(error.at(), after_token(error.at()))
                }
                Some(token) => recovery.basic(unexpected(after_unit(error.at(), token), token)),
                None => recovery.basic(end_of(input.end_site())),
            },
            Err(overflow) => recovery.invalid(overflow.at()),
        })?;
    require_integer_after_sign(input, recovery)?;
    let (a, b) = (notation.a(), notation.b());
    if a == 0 {
        Ok(CssNthPattern::Integer(b))
    } else {
        Ok(CssNthPattern::AnPlusB(CssNthAnPlusB::new(a, b)))
    }
}

/// The boundary after the unit that begins with the token `token` at `at`: the
/// token, and the equals sign after it when the two make an attribute matcher.
fn after_unit<'i>(at: Site<'i>, token: Token<'i>) -> Site<'i> {
    let after = after_token(at);
    if matches!(token, Token::Delim('~' | '|' | '^' | '$' | '*')) {
        let document = after.document();
        if document
            .list(after.list())
            .get(after.index())
            .is_some_and(|node| matches!(document.token(*node), Token::Delim('=')))
        {
            return after_token(after);
        }
    }
    after
}

/// A sign that follows the `n` of a notation takes the integer that is its `B`:
/// a notation that ends at a bare `n` and is followed by a sign is not complete.
fn require_integer_after_sign<'i>(
    input: &mut Stream<'i>,
    recovery: &SelectorRecovery<'_>,
) -> Result<(), ValueError<'i>> {
    if !ends_with_bare_n(input.site()) {
        return Ok(());
    }
    let mark = input.mark();
    input.discard_whitespace();
    if !matches!(input.consume_token(), Ok(Token::Delim('+' | '-'))) {
        input.restore(mark);
        return Ok(());
    }
    input.discard_whitespace();
    let at = input.site();
    match input.consume_token() {
        Ok(token) => Err(recovery.basic(unexpected(after_unit(at, token), token))),
        Err(_) => Err(recovery.basic(end_of(input.end_site()))),
    }
}

/// Whether the last token before `site` spells an `n` that carries no `B`.
fn ends_with_bare_n(site: Site<'_>) -> bool {
    let document = site.document();
    let nodes = document.list(site.list());
    let last = nodes[..site.index().min(nodes.len())]
        .iter()
        .rev()
        .map(|node| document.token(*node))
        .find(|token| !token.is_trivia());
    let bare = |text: &str| text.eq_ignore_ascii_case("n") || text.eq_ignore_ascii_case("-n");
    match last {
        Some(Token::Ident(text)) => bare(text),
        Some(Token::Dimension { unit, .. }) => unit.eq_ignore_ascii_case("n"),
        _ => false,
    }
}

#[cfg(test)]
mod nth_tests {
    use super::*;
    use crate::source::CssSourceSnapshot;
    use crate::syntax_consumption::source_document;

    /// The notation of `source` and what follows it, read as the selectors do.
    fn read(source: &str) -> (CssNthPattern, Vec<String>) {
        let snapshot = CssSourceSnapshot::new(source);
        let document = source_document(source, &snapshot, 0).unwrap();
        let mut input = document.root();
        let mut diagnostics = Vec::new();
        let recovery = SelectorRecovery::new(
            source,
            &mut diagnostics,
            RecoveryState::at_depth(
                source,
                0,
                crate::parser::recovery::StyleContextCaptures::default(),
            ),
        );
        let pattern = parse_nth_pattern(&mut input, &recovery).unwrap();
        let rest = std::iter::from_fn(|| input.consume_after_whitespace().ok())
            .map(|token| token.to_string())
            .collect();
        (pattern, rest)
    }

    #[test]
    fn signless_negative_offset_interpretation_preserves_trivia_and_the_following_of_cursor() {
        for (source, a, b) in [
            ("n - 0002147483648 of .after", 1, i32::MIN),
            ("n- /**/0002147483648 of .after", 1, i32::MIN),
            ("+/**/n -/**/0002147483648 of .after", 1, i32::MIN),
            ("-n- /**/0002147483648 of .after", -1, i32::MIN),
            (r"2\6e -/**/0002147483648 of .after", 2, i32::MIN),
            (r"\6e - 0002147483648 of .after", 1, i32::MIN),
            ("n - 0002 of .after", 1, -2),
        ] {
            let (pattern, rest) = read(source);
            assert_eq!(
                pattern,
                CssNthPattern::AnPlusB(CssNthAnPlusB::new(a, b)),
                "{source}"
            );
            assert_eq!(rest, ["of", ".", "after"], "{source}");
        }
    }

    #[test]
    fn source_interpretation_preserves_signed_embedded_and_out_of_model_offsets() {
        for (source, b) in [
            ("n -2147483648", i32::MIN),
            ("n-2147483648", i32::MIN),
            ("n - 2147483649", -i32::MAX),
            ("n- 99999999999999999999999999", -i32::MAX),
            ("n + 2147483648", i32::MAX),
        ] {
            let (pattern, rest) = read(source);
            assert_eq!(
                pattern,
                CssNthPattern::AnPlusB(CssNthAnPlusB::new(1, b)),
                "{source}"
            );
            assert!(rest.is_empty(), "{source}");
        }
    }
}

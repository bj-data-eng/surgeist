use cssparser::{
    BasicParseErrorKind, Delimiter, ParseError, Parser, ParserState, ToCss, Token,
    match_ignore_ascii_case, parse_nth,
};

use super::recovery::{RecoveryState, comma_member_span, recovery_action_for_error};
use crate::error::{
    CssFeatureId, Error, from_parse_error, invalid_selector, invalid_selector_at, selector_basic,
    selector_component_error,
};
use crate::syntax::*;

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
    CssFeatureId::new("ext.pseudo-element.backdrop"),
    CssFeatureId::new("ext.pseudo-element.generated-marker"),
    CssFeatureId::new("ext.pseudo-element.unknown-webkit"),
];

pub(super) struct SelectorRecovery<'a> {
    source: &'a str,
    diagnostics: &'a mut Vec<crate::CssRecoveryDiagnostic>,
    state: RecoveryState,
}

impl<'a> SelectorRecovery<'a> {
    pub(super) fn new(
        source: &'a str,
        diagnostics: &'a mut Vec<crate::CssRecoveryDiagnostic>,
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

    pub(super) fn check_depth<'i, 't>(
        &self,
        input: &Parser<'i, 't>,
    ) -> std::result::Result<(), ParseError<'i, Error>> {
        self.state
            .check_specialized_components(self.source, input, "baseline.selector.complex")
            .map(|_| ())
    }

    fn check_forgiving_envelope<'i>(
        &self,
        input: &mut Parser<'i, '_>,
    ) -> Result<(), ParseError<'i, Error>> {
        // Selectors 4 checks the complete <any-value>? envelope before member
        // forgiveness. The component owner sees bad tokens and mismatched
        // closers at every depth, while allowing missing EOF delimiters.
        let start = input.state();
        let result =
            crate::CssComponentValues::collect_from_parser(input, self.state.source_snapshot());
        input.reset(&start);
        result
            .map(|_| ())
            .map_err(|error| selector_component_error(start.source_location(), error))
    }

    fn diagnose_forgiving_member(
        &mut self,
        error: ParseError<'_, Error>,
        member_start: usize,
        member_end: usize,
        following_comma: Option<(usize, usize)>,
        preceding_comma: Option<(usize, usize)>,
        ordinary_action: crate::CssRecoveryAction,
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
        if let Some(diagnostic) = crate::CssRecoveryDiagnostic::new(error, span, action) {
            self.diagnostics.push(diagnostic);
        }
    }
}

pub(super) fn parse_rule_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<CssSelector>, ParseError<'i, Error>> {
    recovery.check_depth(input)?;
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_rule_selector(input, recovery)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    input.expect_exhausted().map_err(selector_basic)?;
    Ok(selectors)
}

pub(super) fn parse_scope_boundary_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
    anchors: SelectorAnchorMode,
    allow_relative: bool,
) -> std::result::Result<CssScopeSelectorList, ParseError<'i, Error>> {
    recovery
        .state
        .check_component_contents(recovery.source, input, "baseline.selector.complex")?;
    let options = SelectorParseOptions::scope_boundary(anchors);
    let mut selectors = Vec::new();
    loop {
        let member = if allow_relative {
            match parse_style_selector_with_options(input, options, recovery)? {
                CssStyleSelector::Selector(selector) => CssScopeSelector::Selector(selector),
                CssStyleSelector::Relative(selector) => CssScopeSelector::Relative(selector),
            }
        } else {
            CssScopeSelector::Selector(parse_rule_selector_with_options(input, options, recovery)?)
        };
        selectors.push(member);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    input.expect_exhausted().map_err(selector_basic)?;
    CssScopeSelectorList::try_new(selectors)
        .ok_or_else(|| invalid_selector(input, "scope selector list must not be empty"))
}

pub(super) fn parse_scoped_style_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssScopedStyleSelectorList, ParseError<'i, Error>> {
    recovery.check_depth(input)?;
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_scoped_style_selector(input, recovery)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    input.expect_exhausted().map_err(selector_basic)?;
    CssScopedStyleSelectorList::try_new(selectors)
        .ok_or_else(|| invalid_selector(input, "scoped selector list must not be empty"))
}

fn parse_scoped_style_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssScopedStyleSelector, ParseError<'i, Error>> {
    match parse_style_selector_with_options(input, SelectorParseOptions::scoped_style(), recovery)?
    {
        CssStyleSelector::Selector(selector) => Ok(CssScopedStyleSelector::Selector(selector)),
        CssStyleSelector::Relative(selector) => Ok(CssScopedStyleSelector::Relative(selector)),
    }
}

pub(super) fn parse_nested_style_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<CssStyleSelector>, ParseError<'i, Error>> {
    recovery.check_depth(input)?;
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_style_selector_with_options(
            input,
            SelectorParseOptions::nested_style(),
            recovery,
        )?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    input.expect_exhausted().map_err(selector_basic)?;
    Ok(selectors)
}

fn parse_style_selector_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssStyleSelector, ParseError<'i, Error>> {
    consume_selector_whitespace(input)?;
    let state = input.state();
    match input.next_including_whitespace() {
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
            if matches!(input.next_including_whitespace(), Ok(Token::Delim('|'))) {
                return parse_selector_after_leading_combinator_with_options(
                    input,
                    CssSelectorCombinator::Column,
                    options,
                    recovery,
                )
                .map(CssStyleSelector::Relative);
            }
            input.reset(&state);
            parse_rule_selector_with_options(input, options, recovery)
                .map(CssStyleSelector::Selector)
        }
        Ok(_) => {
            input.reset(&state);
            parse_rule_selector_with_options(input, options, recovery)
                .map(CssStyleSelector::Selector)
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&state);
            Err(invalid_selector(input, "selector is missing"))
        }
        Err(error) => Err(selector_basic(error)),
    }
}

pub(super) fn parse_rule_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssSelector, ParseError<'i, Error>> {
    parse_rule_selector_with_options(input, SelectorParseOptions::standard(), recovery)
}

#[derive(Clone, Copy)]
pub(super) enum SelectorAnchorMode {
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

fn parse_rule_selector_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssSelector, ParseError<'i, Error>> {
    let first = parse_compound_selector_model_with_options(input, options, recovery)?;
    let selector = parse_selector_after_first_compound(input, first, options, recovery)?;
    if options.compound_only && matches!(selector, CssSelector::Complex(_)) {
        return Err(invalid_selector(
            input,
            "this argument requires a compound selector",
        ));
    }
    if options.pseudo_suffix.is_some_and(|suffix| {
        !selector_is_valid_pseudo_suffix(&selector, suffix, options.allow_has)
    }) {
        return Err(invalid_selector(
            input,
            "invalid pseudo-element suffix argument",
        ));
    }
    Ok(selector)
}

fn parse_selector_after_first_compound<'i, 't>(
    input: &mut Parser<'i, 't>,
    first: CssCompoundSelector,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssSelector, ParseError<'i, Error>> {
    let mut rest = Vec::new();

    loop {
        let had_whitespace = consume_selector_whitespace(input)?;
        let state = input.state();
        match input.next_including_whitespace().cloned() {
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                input.reset(&state);
                break;
            }
            Err(error) => return Err(selector_basic(error)),
            Ok(Token::Comma) => {
                input.reset(&state);
                break;
            }
            Ok(Token::Delim('>')) => {
                rest.push(parse_complex_selector_part_with_options(
                    input,
                    CssSelectorCombinator::Child,
                    options,
                    recovery,
                )?);
            }
            Ok(Token::Delim('+')) => {
                rest.push(parse_complex_selector_part_with_options(
                    input,
                    CssSelectorCombinator::NextSibling,
                    options,
                    recovery,
                )?);
            }
            Ok(Token::Delim('~')) => {
                rest.push(parse_complex_selector_part_with_options(
                    input,
                    CssSelectorCombinator::SubsequentSibling,
                    options,
                    recovery,
                )?);
            }
            Ok(Token::Delim('|')) if input.try_parse(expect_adjacent_bar).is_ok() => {
                rest.push(parse_complex_selector_part_with_options(
                    input,
                    CssSelectorCombinator::Column,
                    options,
                    recovery,
                )?);
            }
            Ok(_) if had_whitespace => {
                input.reset(&state);
                let selector =
                    parse_compound_selector_model_with_options(input, options, recovery)?;
                rest.push(CssComplexSelectorPart::new(
                    CssSelectorCombinator::Descendant,
                    selector,
                ));
            }
            Ok(token) => {
                let token = token.clone();
                input.reset(&state);
                return Err(selector_basic(
                    state
                        .source_location()
                        .new_basic_unexpected_token_error(token),
                ));
            }
        }
    }

    if rest.is_empty() {
        Ok(compound_selector_to_selector(first))
    } else if crate::syntax::complex_selector_has_non_terminal_pseudo_elements(&first, &rest) {
        Err(invalid_selector(
            input,
            "pseudo-element selector must be terminal",
        ))
    } else {
        // The parser has proved each compound under its own grammar/recovery
        // budget. Public construction's separate specified-output limit is not
        // a parser admission limit or a pseudo-element diagnostic.
        Ok(CssSelector::Complex(CssComplexSelector::new(first, rest)))
    }
}

fn parse_complex_selector_part_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    combinator: CssSelectorCombinator,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssComplexSelectorPart, ParseError<'i, Error>> {
    consume_selector_whitespace(input)?;
    let selector = parse_compound_selector_model_with_options(input, options, recovery)?;
    Ok(CssComplexSelectorPart::new(combinator, selector))
}

pub(super) fn consume_selector_whitespace<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<bool, ParseError<'i, Error>> {
    let mut consumed = false;
    loop {
        let state = input.state();
        match input.next_including_whitespace() {
            Ok(Token::WhiteSpace(_)) => consumed = true,
            Ok(_) => {
                input.reset(&state);
                return Ok(consumed);
            }
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                input.reset(&state);
                return Ok(consumed);
            }
            Err(error) => return Err(selector_basic(error)),
        }
    }
}

fn parse_type_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &SelectorRecovery<'_>,
) -> std::result::Result<Option<CssQualifiedSelectorName>, ParseError<'i, Error>> {
    let start = input.state();
    let first = match input.next_including_whitespace() {
        Ok(token) => token.clone(),
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&start);
            return Ok(None);
        }
        Err(error) => return Err(selector_basic(error)),
    };
    let (namespace, prefix, local_name) = match first {
        Token::Ident(prefix_or_name) => {
            let prefix_or_name = prefix_or_name.to_string();
            if consume_namespace_separator(input)? {
                let local_start = input.state();
                let local_name = parse_qualified_local_name(input)?;
                let Some(prefix) = recovery.named_namespace(&prefix_or_name) else {
                    input.reset(&local_start);
                    return Err(invalid_selector(
                        input,
                        format!("undeclared selector namespace prefix `{prefix_or_name}`"),
                    ));
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
            if consume_namespace_separator(input)? {
                (
                    CssNamespaceConstraint::Any,
                    CssQualifiedNamePrefix::Any,
                    parse_qualified_local_name(input)?,
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
            parse_qualified_local_name(input)?,
        ),
        _ => {
            input.reset(&start);
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

fn parse_qualified_local_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Option<String>, ParseError<'i, Error>> {
    match input.next_including_whitespace() {
        Ok(Token::Ident(name)) => Ok(Some(name.to_string())),
        Ok(Token::Delim('*')) => Ok(None),
        Ok(token) => {
            let authored = token.to_css_string();
            Err(invalid_selector(
                input,
                format!(
                    "selector namespace separator must be followed by a local name or `*`, found `{authored}`"
                ),
            ))
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => Err(
            invalid_selector(input, "selector namespace is missing a local name"),
        ),
        Err(error) => Err(selector_basic(error)),
    }
}

fn parse_compound_selector_model_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssCompoundSelector, ParseError<'i, Error>> {
    loop {
        let state = input.state();
        match input.next_including_whitespace() {
            Ok(Token::WhiteSpace(_)) => continue,
            Ok(_) => {
                input.reset(&state);
                break;
            }
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                input.reset(&state);
                break;
            }
            Err(error) => return Err(selector_basic(error)),
        }
    }

    let type_selector = parse_type_selector(input, recovery)?;
    let mut scope_anchors = 0;
    let mut nesting_selectors = 0;
    let mut id_names = Vec::new();
    let mut class_names = Vec::new();
    let mut attributes = Vec::new();
    let mut pseudo_classes = Vec::new();
    let mut pseudo_elements = None;

    loop {
        let state = input.state();
        match input.next_including_whitespace() {
            Ok(Token::WhiteSpace(_) | Token::Comma) => {
                input.reset(&state);
                break;
            }
            Ok(_) => input.reset(&state),
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => break,
            Err(error) => return Err(selector_basic(error)),
        }

        if pseudo_elements.is_some() {
            return Err(invalid_selector(
                input,
                "pseudo-element selector must be terminal",
            ));
        }

        if input.try_parse(|input| input.expect_delim('&')).is_ok() {
            match options.anchors {
                SelectorAnchorMode::Nesting => nesting_selectors += 1,
                SelectorAnchorMode::Scope => scope_anchors += 1,
            }
            continue;
        }

        if input.try_parse(|input| input.expect_delim('.')).is_ok() {
            class_names.push(parse_adjacent_class_name(input)?);
            continue;
        }

        if input.try_parse(Parser::expect_square_bracket_block).is_ok() {
            let attribute =
                input.parse_nested_block(|input| parse_attribute_selector(input, recovery))?;
            attributes.push(attribute);
            continue;
        }

        if input.try_parse(Parser::expect_colon).is_ok() {
            if input.try_parse(expect_adjacent_colon).is_ok() {
                if !options.allow_pseudo_elements {
                    return Err(invalid_selector(
                        input,
                        "pseudo-elements are not supported in this selector context",
                    ));
                }
                let sequence = parse_pseudo_element_sequence(input, options, recovery)?;
                pseudo_elements = Some(sequence);
            } else if let Ok(first) = input.try_parse(parse_legacy_pseudo_element) {
                if !options.allow_pseudo_elements {
                    return Err(invalid_selector(
                        input,
                        "pseudo-elements are not supported in this selector context",
                    ));
                }
                pseudo_elements = Some(parse_pseudo_element_sequence_from_first(
                    input, first, options, recovery,
                )?);
            } else {
                let pseudo_class = parse_pseudo_class_with_options(input, options, recovery)?;
                pseudo_classes.push(pseudo_class);
            }
            continue;
        }

        let state = input.state();
        match input.next() {
            Ok(Token::IDHash(key)) => {
                let key = key.to_string();
                id_names.push(key);
            }
            Ok(Token::Delim('|')) => {
                if input.try_parse(expect_adjacent_bar).is_ok() {
                    input.reset(&state);
                    break;
                }
                return Err(invalid_selector(input, "unsupported selector namespace"));
            }
            Ok(token) => {
                let token = token.clone();
                input.reset(&state);
                if type_selector.is_none()
                    && scope_anchors == 0
                    && nesting_selectors == 0
                    && id_names.is_empty()
                    && class_names.is_empty()
                    && attributes.is_empty()
                    && pseudo_classes.is_empty()
                    && pseudo_elements.is_none()
                {
                    return Err(selector_basic(
                        state
                            .source_location()
                            .new_basic_unexpected_token_error(token),
                    ));
                }
                break;
            }
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => break,
            Err(error) => return Err(selector_basic(error)),
        }
    }

    if type_selector.is_none()
        && scope_anchors == 0
        && nesting_selectors == 0
        && id_names.is_empty()
        && class_names.is_empty()
        && attributes.is_empty()
        && pseudo_classes.is_empty()
        && pseudo_elements.is_none()
    {
        return Err(invalid_selector(
            input,
            "selector is missing a simple selector",
        ));
    }
    Ok(
        CssCompoundSelector::new_with_qualified_type_and_pseudo_elements(
            scope_anchors,
            type_selector,
            id_names,
            class_names,
            attributes,
            pseudo_classes,
            pseudo_elements,
        )
        .with_nesting_selectors(nesting_selectors),
    )
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

fn expect_adjacent_colon<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<(), cssparser::BasicParseError<'i>> {
    match input.next_including_whitespace()?.clone() {
        Token::Colon => Ok(()),
        token => Err(input.new_basic_unexpected_token_error(token)),
    }
}

fn expect_adjacent_bar<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<(), cssparser::BasicParseError<'i>> {
    match input.next_including_whitespace()?.clone() {
        Token::Delim('|') => Ok(()),
        token => Err(input.new_basic_unexpected_token_error(token)),
    }
}

/// A namespace separator is one adjacent bar, rather than the two-token column
/// combinator. Leave both column tokens for the complex-selector provider.
fn consume_namespace_separator<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<bool, ParseError<'i, Error>> {
    let start = input.state();
    match input.next_including_whitespace() {
        Ok(Token::Delim('|')) => {
            let after_bar = input.state();
            let column = input.try_parse(expect_adjacent_bar).is_ok();
            input.reset(if column { &start } else { &after_bar });
            Ok(!column)
        }
        Ok(_) => {
            input.reset(&start);
            Ok(false)
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&start);
            Ok(false)
        }
        Err(error) => Err(selector_basic(error)),
    }
}

fn parse_adjacent_class_name<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<String, ParseError<'i, Error>> {
    loop {
        let start = input.state();
        match input.next_including_whitespace_and_comments().cloned() {
            Ok(Token::Comment(_)) => continue,
            Ok(Token::Ident(value)) => return Ok(value.to_string()),
            Ok(token) => {
                input.reset(&start);
                return Err(selector_basic(
                    input.new_basic_unexpected_token_error(token),
                ));
            }
            Err(error) => return Err(selector_basic(error)),
        }
    }
}

fn parse_pseudo_element_sequence<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssPseudoElementSequence, ParseError<'i, Error>> {
    let first = parse_pseudo_element(input, options, recovery)?;
    parse_pseudo_element_sequence_from_first(input, first, options, recovery)
}

fn parse_pseudo_element_sequence_from_first<'i, 't>(
    input: &mut Parser<'i, 't>,
    first: CssPseudoElement,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssPseudoElementSequence, ParseError<'i, Error>> {
    let mut suffix = first.suffix_context();
    let mut segments = vec![CssPseudoElementSegment::PseudoElement(first)];
    while input.try_parse(expect_adjacent_colon).is_ok() {
        let element = if input.try_parse(expect_adjacent_colon).is_ok() {
            Some(parse_pseudo_element(input, options, recovery)?)
        } else {
            input.try_parse(parse_legacy_pseudo_element).ok()
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
        .ok_or_else(|| invalid_selector(input, "unsupported pseudo-element sequence"))
}

fn parse_legacy_pseudo_element<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPseudoElement, ParseError<'i, Error>> {
    let name = match input.next_including_whitespace()?.clone() {
        Token::Ident(name) => name,
        token => return Err(input.new_basic_unexpected_token_error(token).into()),
    };
    match_ignore_ascii_case! { &name,
        "before" => Ok(CssPseudoElement::Before),
        "after" => Ok(CssPseudoElement::After),
        "first-line" => Ok(CssPseudoElement::FirstLine),
        "first-letter" => Ok(CssPseudoElement::FirstLetter),
        _ => Err(
            input
                .new_basic_unexpected_token_error(Token::Ident(name))
                .into(),
        ),
    }
}

fn parse_pseudo_element<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssPseudoElement, ParseError<'i, Error>> {
    let state = input.state();
    match input.next_including_whitespace() {
        Ok(Token::Ident(name)) => match_ignore_ascii_case! { &name,
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
            _ => {
                if let Ok(name) = CssUnknownWebkitPseudoElement::try_new(name.to_string()) {
                    return Ok(CssPseudoElement::UnknownWebkit(name));
                }
                let message = format!("unsupported pseudo-element `::{name}`");
                input.reset(&state);
                Err(invalid_selector(input, message))
            }
        },
        Ok(Token::Function(name))
            if name.eq_ignore_ascii_case("slotted")
                || name.eq_ignore_ascii_case("part")
                || name.eq_ignore_ascii_case("highlight") =>
        {
            let name = name.clone();
            let mut depth = recovery.state.enter_component_block(
                recovery.source,
                input,
                "baseline.selector.complex",
            )?;
            let result = input.parse_nested_block(|input| {
                if name.eq_ignore_ascii_case("slotted") {
                    parse_compound_argument(input, options, recovery).map(CssPseudoElement::Slotted)
                } else if name.eq_ignore_ascii_case("part") {
                    parse_part_names(input, recovery).map(CssPseudoElement::Part)
                } else {
                    parse_highlight_name(input).map(CssPseudoElement::Highlight)
                }
            });
            if result.is_ok() {
                depth.retain();
            }
            result
        }
        Ok(token) => {
            let message = format!("unsupported pseudo-element `::{}`", token.to_css_string());
            input.reset(&state);
            Err(invalid_selector(input, message))
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => Err(
            invalid_selector(input, "selector pseudo-element is missing a name"),
        ),
        Err(error) => Err(selector_basic(error)),
    }
}

// Comments are absent from CSS grammar, but actual whitespace cannot separate
// the two components of an attribute matcher.
fn expect_adjacent_attribute_equals<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<(), cssparser::BasicParseError<'i>> {
    loop {
        let location = input.current_source_location();
        match input.next_including_whitespace_and_comments()?.clone() {
            Token::Comment(_) => continue,
            Token::Delim('=') => return Ok(()),
            token => return Err(location.new_basic_unexpected_token_error(token)),
        }
    }
}

fn parse_attribute_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &SelectorRecovery<'_>,
) -> std::result::Result<CssAttributeSelector, ParseError<'i, Error>> {
    consume_selector_whitespace(input)?;
    let name = parse_attribute_selector_name(input, recovery)?;

    let matcher = match input.next() {
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            return Ok(CssAttributeSelector::new_qualified(
                name,
                CssAttributeMatcher::Exists,
                CssAttributeCaseSensitivity::DocumentDefault,
            ));
        }
        Err(error) => return Err(selector_basic(error)),
        Ok(Token::Delim('=')) => {
            CssAttributeMatcher::Equals(parse_attribute_selector_value(input)?)
        }
        Ok(Token::IncludeMatch) => {
            CssAttributeMatcher::Includes(parse_attribute_selector_value(input)?)
        }
        Ok(Token::DashMatch) => {
            CssAttributeMatcher::DashMatch(parse_attribute_selector_value(input)?)
        }
        Ok(Token::PrefixMatch) => {
            CssAttributeMatcher::Prefix(parse_attribute_selector_value(input)?)
        }
        Ok(Token::SuffixMatch) => {
            CssAttributeMatcher::Suffix(parse_attribute_selector_value(input)?)
        }
        Ok(Token::SubstringMatch) => {
            CssAttributeMatcher::Substring(parse_attribute_selector_value(input)?)
        }
        Ok(Token::Delim('~')) => {
            expect_adjacent_attribute_equals(input).map_err(selector_basic)?;
            CssAttributeMatcher::Includes(parse_attribute_selector_value(input)?)
        }
        Ok(Token::Delim('|')) => {
            expect_adjacent_attribute_equals(input).map_err(selector_basic)?;
            CssAttributeMatcher::DashMatch(parse_attribute_selector_value(input)?)
        }
        Ok(Token::Delim('^')) => {
            expect_adjacent_attribute_equals(input).map_err(selector_basic)?;
            CssAttributeMatcher::Prefix(parse_attribute_selector_value(input)?)
        }
        Ok(Token::Delim('$')) => {
            expect_adjacent_attribute_equals(input).map_err(selector_basic)?;
            CssAttributeMatcher::Suffix(parse_attribute_selector_value(input)?)
        }
        Ok(Token::Delim('*')) => {
            expect_adjacent_attribute_equals(input).map_err(selector_basic)?;
            CssAttributeMatcher::Substring(parse_attribute_selector_value(input)?)
        }
        Ok(token) => {
            let message = format!(
                "unsupported attribute selector token `{}`",
                token.to_css_string()
            );
            return Err(invalid_selector(input, message));
        }
    };

    let case_sensitivity = parse_attribute_case_sensitivity(input)?;
    input.expect_exhausted().map_err(selector_basic)?;
    Ok(CssAttributeSelector::new_qualified(
        name,
        matcher,
        case_sensitivity,
    ))
}

fn parse_attribute_selector_name<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &SelectorRecovery<'_>,
) -> std::result::Result<CssQualifiedAttributeName, ParseError<'i, Error>> {
    match input.next_including_whitespace() {
        Ok(Token::Ident(prefix_or_name)) => {
            let prefix_or_name = prefix_or_name.to_string();
            let after_ident = input.state();
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => {
                    // A split |= matcher belongs to this unqualified name;
                    // a namespace separator instead requires an adjacent ident.
                    if input.try_parse(expect_adjacent_attribute_equals).is_ok() {
                        input.reset(&after_ident);
                        return Ok(CssQualifiedAttributeName::new(
                            CssNamespaceConstraint::ExplicitNone,
                            CssQualifiedNamePrefix::Unqualified,
                            CssAttributeName::new(prefix_or_name),
                        ));
                    }
                    let local_start = input.state();
                    let local_name = input.next_including_whitespace();
                    let local_name = match local_name {
                        Ok(Token::Ident(name)) => name.to_string(),
                        Ok(token) => {
                            let authored = token.to_css_string();
                            return Err(invalid_selector(
                                input,
                                format!(
                                    "attribute namespace separator must be followed by a local name, found `{authored}`"
                                ),
                            ));
                        }
                        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                            return Err(invalid_selector(
                                input,
                                "attribute namespace is missing a local name",
                            ));
                        }
                        Err(error) => return Err(selector_basic(error)),
                    };
                    let Some(prefix) = recovery.named_namespace(&prefix_or_name) else {
                        input.reset(&local_start);
                        return Err(invalid_selector(
                            input,
                            format!("undeclared selector namespace prefix `{prefix_or_name}`"),
                        ));
                    };
                    Ok(CssQualifiedAttributeName::new(
                        CssNamespaceConstraint::Named(prefix.clone()),
                        CssQualifiedNamePrefix::Named(prefix),
                        CssAttributeName::new(local_name),
                    ))
                }
                Ok(_) => {
                    input.reset(&after_ident);
                    Ok(CssQualifiedAttributeName::new(
                        CssNamespaceConstraint::ExplicitNone,
                        CssQualifiedNamePrefix::Unqualified,
                        CssAttributeName::new(prefix_or_name),
                    ))
                }
                Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                    input.reset(&after_ident);
                    Ok(CssQualifiedAttributeName::new(
                        CssNamespaceConstraint::ExplicitNone,
                        CssQualifiedNamePrefix::Unqualified,
                        CssAttributeName::new(prefix_or_name),
                    ))
                }
                Err(error) => Err(selector_basic(error)),
            }
        }
        Ok(Token::Delim('*')) => {
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => {}
                Ok(token) => {
                    let authored = token.to_css_string();
                    return Err(invalid_selector(
                        input,
                        format!(
                            "attribute universal namespace must be followed by `|`, found `{authored}`"
                        ),
                    ));
                }
                Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                    return Err(invalid_selector(
                        input,
                        "attribute universal namespace is missing `|` and a local name",
                    ));
                }
                Err(error) => return Err(selector_basic(error)),
            }
            let name = input.next_including_whitespace();
            match name {
                Ok(Token::Ident(name)) => Ok(CssQualifiedAttributeName::new(
                    CssNamespaceConstraint::Any,
                    CssQualifiedNamePrefix::Any,
                    CssAttributeName::new(name.to_string()),
                )),
                Ok(token) => {
                    let authored = token.to_css_string();
                    Err(invalid_selector(
                        input,
                        format!(
                            "attribute namespace separator must be followed by a local name, found `{authored}`"
                        ),
                    ))
                }
                Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => Err(
                    invalid_selector(input, "attribute namespace is missing a local name"),
                ),
                Err(error) => Err(selector_basic(error)),
            }
        }
        Ok(Token::Delim('|')) => match input.next_including_whitespace() {
            Ok(Token::Ident(name)) => Ok(CssQualifiedAttributeName::new(
                CssNamespaceConstraint::ExplicitNone,
                CssQualifiedNamePrefix::ExplicitNone,
                CssAttributeName::new(name.to_string()),
            )),
            Ok(token) => {
                let authored = token.to_css_string();
                Err(invalid_selector(
                    input,
                    format!(
                        "attribute namespace separator must be followed by a local name, found `{authored}`"
                    ),
                ))
            }
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => Err(
                invalid_selector(input, "attribute namespace is missing a local name"),
            ),
            Err(error) => Err(selector_basic(error)),
        },
        Ok(token) => {
            let authored = token.to_css_string();
            Err(invalid_selector(
                input,
                format!("attribute selector is missing a name before `{authored}`"),
            ))
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => Err(
            invalid_selector(input, "attribute selector is missing a name"),
        ),
        Err(error) => Err(selector_basic(error)),
    }
}

fn parse_attribute_selector_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<String, ParseError<'i, Error>> {
    let value = input
        .expect_ident_or_string()
        .map_err(selector_basic)?
        .to_string();
    Ok(value)
}

fn parse_attribute_case_sensitivity<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAttributeCaseSensitivity, ParseError<'i, Error>> {
    let state = input.state();
    match input.next() {
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&state);
            Ok(CssAttributeCaseSensitivity::DocumentDefault)
        }
        Err(error) => Err(selector_basic(error)),
        Ok(Token::Ident(modifier)) if modifier.eq_ignore_ascii_case("i") => {
            Ok(CssAttributeCaseSensitivity::AsciiCaseInsensitive)
        }
        Ok(Token::Ident(modifier)) if modifier.eq_ignore_ascii_case("s") => {
            Ok(CssAttributeCaseSensitivity::ExplicitSensitive)
        }
        Ok(token) => {
            let message = format!(
                "unsupported attribute selector case modifier `{}`",
                token.to_css_string()
            );
            Err(invalid_selector(input, message))
        }
    }
}

fn parse_pseudo_class_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssPseudoClass, ParseError<'i, Error>> {
    let state = input.state();
    match input.next_including_whitespace() {
        Ok(Token::Ident(name)) => {
            let name = name.clone();
            let pseudo = parse_named_pseudo_class(name.as_ref(), input, &state)?;
            if options.grammar().admits_pseudo(&pseudo) {
                Ok(pseudo)
            } else {
                input.reset(&state);
                Err(invalid_selector(
                    input,
                    format!("unsupported pseudo-class `:{name}` in this receiving context"),
                ))
            }
        }
        Ok(Token::Function(name)) => {
            let name = name.clone();
            let mut depth = recovery.state.enter_component_block(
                recovery.source,
                input,
                "baseline.selector.complex",
            )?;
            let result = input.parse_nested_block(|input| {
                parse_function_pseudo_class(
                    name.as_ref(),
                    state.source_location(),
                    input,
                    options,
                    recovery,
                )
            });
            if result.is_ok() {
                depth.retain();
            }
            result
        }
        Ok(token) => {
            let message = format!("unsupported pseudo-class `:{}`", token.to_css_string());
            input.reset(&state);
            Err(invalid_selector(input, message))
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => Err(
            invalid_selector(input, "selector pseudo-class is missing a name"),
        ),
        Err(error) => Err(selector_basic(error)),
    }
}

#[inline(never)]
fn parse_named_pseudo_class<'i>(
    name: &str,
    input: &mut Parser<'i, '_>,
    name_start: &ParserState,
) -> std::result::Result<CssPseudoClass, ParseError<'i, Error>> {
    match_ignore_ascii_case! { name,
        "host" => Ok(CssPseudoClass::Host),
        "root" => Ok(CssPseudoClass::Root),
        "scope" => Ok(CssPseudoClass::Scope),
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
        "required" => Ok(CssPseudoClass::Required),
        "optional" => Ok(CssPseudoClass::Optional),
        "valid" => Ok(CssPseudoClass::Valid),
        "invalid" => Ok(CssPseudoClass::Invalid),
        "placeholder-shown" => Ok(CssPseudoClass::PlaceholderShown),
        "autofill" | "-webkit-autofill" => Ok(CssPseudoClass::Autofill),
        "first-child" => Ok(CssPseudoClass::FirstChild),
        "last-child" => Ok(CssPseudoClass::LastChild),
        "only-child" => Ok(CssPseudoClass::OnlyChild),
        "empty" => Ok(CssPseudoClass::Empty),
        "first-of-type" => Ok(CssPseudoClass::FirstOfType),
        "last-of-type" => Ok(CssPseudoClass::LastOfType),
        "only-of-type" => Ok(CssPseudoClass::OnlyOfType),
        "modal" => Ok(CssPseudoClass::Modal),
        "fullscreen" => Ok(CssPseudoClass::Fullscreen),
        "popover-open" => Ok(CssPseudoClass::PopoverOpen),
        "default" => Ok(CssPseudoClass::Default),
        "indeterminate" => Ok(CssPseudoClass::Indeterminate),
        "read-only" => Ok(CssPseudoClass::ReadOnly),
        "read-write" => Ok(CssPseudoClass::ReadWrite),
        "in-range" => Ok(CssPseudoClass::InRange),
        "out-of-range" => Ok(CssPseudoClass::OutOfRange),
        _ => {
            input.reset(name_start);
            Err(invalid_selector(input, format!("unsupported pseudo-class `:{name}`")))
        },
    }
}

#[inline(never)]
fn parse_function_pseudo_class<'i, 't>(
    name: &str,
    name_start: cssparser::SourceLocation,
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssPseudoClass, ParseError<'i, Error>> {
    let arguments = options.independent_arguments();
    let pseudo_class = match_ignore_ascii_case! { name,
        "host" => CssPseudoClass::HostFunction(parse_compound_argument(input, arguments, recovery)?),
        "host-context" => CssPseudoClass::HostContext(parse_compound_argument(input, arguments, recovery)?),
        "nth-child" => CssPseudoClass::NthChild(parse_nth_child_pattern(input, arguments, recovery)?),
        "nth-last-child" => CssPseudoClass::NthLastChild(parse_nth_child_pattern(input, arguments, recovery)?),
        "nth-of-type" => CssPseudoClass::NthOfType(parse_nth_pattern(input)?),
        "nth-last-of-type" => CssPseudoClass::NthLastOfType(parse_nth_pattern(input)?),
        "dir" => CssPseudoClass::Dir(parse_directionality(input, recovery)?),
        "lang" => CssPseudoClass::Lang(parse_language_ranges(input, recovery)?),
        "not" => CssPseudoClass::Not(parse_pseudo_selector_list_with_options(input, options.without_pseudo_elements(), recovery)?),
        "is" => CssPseudoClass::Is(parse_forgiving_pseudo_selector_list(input, options.without_pseudo_elements(), recovery)?),
        "where" => CssPseudoClass::Where(parse_forgiving_pseudo_selector_list(input, options.without_pseudo_elements(), recovery)?),
        "has" if options.allow_has => CssPseudoClass::Has(parse_has_relative_selector_list(input, arguments, recovery)?),
        "has" => return Err(invalid_selector(input, "nested `:has()` is unsupported")),
        _ => return Err(invalid_selector_at(name_start, format!("unsupported pseudo-class `:{name}(`"))),
    };
    input.expect_exhausted().map_err(selector_basic)?;
    Ok(pseudo_class)
}

fn parse_compound_argument<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> Result<CssCompoundSelectorArgument, ParseError<'i, Error>> {
    let options = SelectorParseOptions {
        compound_only: true,
        pseudo_suffix: None,
        ..options.without_pseudo_elements()
    };
    let selector = parse_rule_selector_with_options(input, options, recovery)?;
    input.expect_exhausted().map_err(selector_basic)?;
    CssCompoundSelectorArgument::try_new(selector)
        .ok_or_else(|| invalid_selector(input, "invalid compound selector argument"))
}

fn parse_part_names<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &SelectorRecovery<'_>,
) -> Result<CssPartNameList, ParseError<'i, Error>> {
    let mut names = Vec::new();
    loop {
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }
        let start = input.state();
        input.expect_ident().map_err(selector_basic)?;
        input.reset(&start);
        let name =
            crate::CssComponentValue::collect_from_parser(input, recovery.state.source_snapshot())
                .and_then(CssPartName::from_component)
                .map_err(|error| selector_component_error(start.source_location(), error))?;
        names.push(name);
    }
    CssPartNameList::try_new(names)
        .ok_or_else(|| invalid_selector(input, "part requires at least one identifier"))
}

fn parse_highlight_name<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<CssCustomIdent, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    let name = input.expect_ident().map_err(selector_basic)?.to_string();
    let name = CssCustomIdent::try_new(name).ok_or_else(|| {
        invalid_selector_at(
            start.source_location(),
            "highlight requires a custom identifier",
        )
    })?;
    input.expect_exhausted().map_err(selector_basic)?;
    Ok(name)
}

fn parse_directionality<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &SelectorRecovery<'_>,
) -> std::result::Result<CssDirectionality, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    input.expect_ident().map_err(selector_basic)?;
    input.reset(&start);
    crate::CssComponentValue::collect_from_parser(input, recovery.state.source_snapshot())
        .and_then(CssDirectionality::from_component)
        .map_err(|error| selector_component_error(start.source_location(), error))
}

fn parse_language_ranges<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &SelectorRecovery<'_>,
) -> std::result::Result<CssLanguageRangeList, ParseError<'i, Error>> {
    let mut ranges = Vec::new();
    loop {
        input.skip_whitespace();
        let start = input.state();
        input.expect_ident_or_string().map_err(selector_basic)?;
        input.reset(&start);
        let range =
            crate::CssComponentValue::collect_from_parser(input, recovery.state.source_snapshot())
                .and_then(CssLanguageRange::from_component)
                .map_err(|error| selector_component_error(start.source_location(), error))?;
        ranges.push(range);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    CssLanguageRangeList::try_new(ranges)
        .map_err(|_| invalid_selector(input, "`:lang()` requires at least one range"))
}

fn parse_pseudo_selector_list_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssPseudoSelectorList, ParseError<'i, Error>> {
    let selectors = parse_pseudo_selector_list_items_with_options(input, options, recovery)?;
    // This parser has proved each member in its actual containing context and
    // charged the shared parse budget. Programmatic output limits are separate.
    Ok(CssPseudoSelectorList::from_parsed_items(selectors))
}

fn parse_pseudo_selector_list_items_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<CssPseudoSelectorListItem>, ParseError<'i, Error>> {
    let mut selectors = Vec::new();
    loop {
        let selector = parse_rule_selector_with_options(input, options, recovery)?;
        selectors.try_reserve(1).map_err(|_| {
            selector_component_error(
                input.current_source_location(),
                crate::CssComponentValueError::new(
                    crate::CssComponentValueErrorKind::CapacityOverflow,
                    crate::CssValueOrigin::Programmatic,
                ),
            )
        })?;
        selectors.push(CssPseudoSelectorListItem::Selector(selector));
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    input.expect_exhausted().map_err(selector_basic)?;
    Ok(selectors)
}

fn parse_forgiving_pseudo_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssPseudoSelectorList, ParseError<'i, Error>> {
    recovery.check_forgiving_envelope(input)?;
    let mut items = Vec::new();
    let mut preceding_comma = None;
    loop {
        let member_start = input.position().byte_index();
        let (result, invalid_components) =
            input.parse_until_before(Delimiter::Comma, |member| {
                let start = member.state();
                let result = parse_rule_selector_with_options(member, options, recovery);
                let components = match &result {
                    Err(error)
                        if matches!(&error.kind,
                    cssparser::ParseErrorKind::Custom(error)
                    if matches!(error.kind(), crate::ErrorKind::InvalidSelector(_))) =>
                    {
                        member.reset(&start);
                        Some(
                            crate::CssComponentValues::collect_from_parser(
                                member,
                                recovery.state.source_snapshot(),
                            )
                            .map_err(|error| {
                                selector_component_error(start.source_location(), error)
                            })?,
                        )
                    }
                    Err(error) => return Err(error.clone()),
                    Ok(_) => None,
                };
                Ok((result, components))
            })?;
        let member_end = input.position().byte_index();
        let comma_start = member_end;
        let following_comma = match input.next() {
            Ok(Token::Comma) => Some((comma_start, input.position().byte_index())),
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => None,
            Ok(_) => return Err(invalid_selector(input, "invalid selector-list delimiter")),
            Err(error) => return Err(selector_basic(error)),
        };

        let item = match result {
            Ok(selector) => Some(CssPseudoSelectorListItem::Selector(selector)),
            Err(error) => {
                let components = invalid_components.expect("failed member components");
                let location = error.location;
                let contains_nesting = components
                    .contains_delimiter('&')
                    .map_err(|error| selector_component_error(location, error))?;
                let action = if contains_nesting {
                    crate::CssRecoveryAction::PreserveInvalidSelectorListItem
                } else {
                    crate::CssRecoveryAction::DropSelectorListItem
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
                    let origin = crate::CssParsedOrigin::from_range(
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
            items.try_reserve(1).map_err(|_| {
                selector_component_error(
                    input.current_source_location(),
                    crate::CssComponentValueError::new(
                        crate::CssComponentValueErrorKind::CapacityOverflow,
                        crate::CssValueOrigin::Programmatic,
                    ),
                )
            })?;
            items.push(item);
        }
        let Some(comma) = following_comma else {
            break;
        };
        preceding_comma = Some(comma);
    }
    Ok(CssPseudoSelectorList::from_parsed_items(items))
}

fn parse_has_relative_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssRelativeSelectorList, ParseError<'i, Error>> {
    parse_relative_selector_list_with_options(
        input,
        options.without_nested_has().without_pseudo_elements(),
        recovery,
    )
}

pub(super) fn parse_general_relative_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssRelativeSelectorList, ParseError<'i, Error>> {
    recovery.check_depth(input)?;
    parse_relative_selector_list_with_options(input, SelectorParseOptions::standard(), recovery)
}

fn parse_relative_selector_list_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssRelativeSelectorList, ParseError<'i, Error>> {
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_relative_selector_with_options(
            input, options, recovery,
        )?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    input.expect_exhausted().map_err(selector_basic)?;
    // The caller's grammar and parse budget are already checked; avoid applying
    // an unrelated default specified-output budget or losing parse provenance.
    Ok(CssRelativeSelectorList::new(selectors))
}

fn parse_relative_selector_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssRelativeSelector, ParseError<'i, Error>> {
    match parse_style_selector_with_options(input, options, recovery)? {
        CssStyleSelector::Selector(selector) => Ok(CssRelativeSelector::new(
            CssSelectorCombinator::Descendant,
            selector,
        )),
        CssStyleSelector::Relative(selector) => Ok(selector),
    }
}

fn parse_selector_after_leading_combinator_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    combinator: CssSelectorCombinator,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssRelativeSelector, ParseError<'i, Error>> {
    consume_selector_whitespace(input)?;
    let first = parse_compound_selector_model_with_options(input, options, recovery)?;
    let selector = parse_selector_after_first_compound(input, first, options, recovery)?;
    Ok(CssRelativeSelector::new(combinator, selector))
}

fn parse_nth_child_pattern<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssNthChildPattern, ParseError<'i, Error>> {
    let pattern = parse_nth_pattern(input)?;
    let state = input.state();
    match input.next() {
        Ok(Token::Ident(value)) if value.eq_ignore_ascii_case("of") => {
            let selector_list = parse_pseudo_selector_list_with_options(
                input,
                options.without_pseudo_elements(),
                recovery,
            )?;
            Ok(CssNthChildPattern::new(pattern, Some(selector_list)))
        }
        Ok(_) => {
            input.reset(&state);
            Ok(CssNthChildPattern::new(pattern, None))
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&state);
            Ok(CssNthChildPattern::new(pattern, None))
        }
        Err(error) => Err(selector_basic(error)),
    }
}

fn parse_nth_pattern<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssNthPattern, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("odd"))
        .is_ok()
    {
        return Ok(CssNthPattern::Odd);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("even"))
        .is_ok()
    {
        return Ok(CssNthPattern::Even);
    }

    let (a, b) = parse_nth(input).map_err(selector_basic)?;
    if a == 0 {
        Ok(CssNthPattern::Integer(b))
    } else {
        Ok(CssNthPattern::AnPlusB(CssNthAnPlusB::new(a, b)))
    }
}

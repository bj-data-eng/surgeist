use cssparser::{
    BasicParseErrorKind, Delimiter, ParseError, Parser, ToCss, Token, match_ignore_ascii_case,
    parse_nth,
};

use super::recovery::{RecoveryState, comma_member_span, recovery_action_for_error};
use crate::error::{CssFeatureId, Error, from_parse_error, invalid_selector, selector_basic};
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
    CssFeatureId::new("ext.pseudo-element.backdrop"),
    CssFeatureId::new("ext.pseudo-element.generated-marker"),
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

    fn drop_forgiving_member(
        &mut self,
        error: ParseError<'_, Error>,
        member_start: usize,
        member_end: usize,
        following_comma: Option<(usize, usize)>,
        preceding_comma: Option<(usize, usize)>,
    ) {
        let action =
            recovery_action_for_error(&error, crate::CssRecoveryAction::DropSelectorListItem);
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
) -> std::result::Result<CssScopeSelectorList, ParseError<'i, Error>> {
    recovery.check_depth(input)?;
    let selectors = parse_rule_selector_list_with_options(
        input,
        SelectorParseOptions::scope_boundary(),
        recovery,
    )?;
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
    consume_selector_whitespace(input)?;
    let state = input.state();
    match input.next_including_whitespace() {
        Ok(Token::Delim('>')) => parse_selector_after_leading_combinator_with_options(
            input,
            CssSelectorCombinator::Child,
            SelectorParseOptions::scoped_style(),
            recovery,
        )
        .map(CssScopedStyleSelector::Relative),
        Ok(Token::Delim('+')) => parse_selector_after_leading_combinator_with_options(
            input,
            CssSelectorCombinator::NextSibling,
            SelectorParseOptions::scoped_style(),
            recovery,
        )
        .map(CssScopedStyleSelector::Relative),
        Ok(Token::Delim('~')) => parse_selector_after_leading_combinator_with_options(
            input,
            CssSelectorCombinator::SubsequentSibling,
            SelectorParseOptions::scoped_style(),
            recovery,
        )
        .map(CssScopedStyleSelector::Relative),
        Ok(Token::Delim('|')) => Err(invalid_selector(
            input,
            "unsupported selector combinator `||`",
        )),
        Ok(_) => {
            input.reset(&state);
            parse_rule_selector_with_options(input, SelectorParseOptions::scoped_style(), recovery)
                .map(CssScopedStyleSelector::Selector)
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&state);
            Err(invalid_selector(input, "scoped selector is missing"))
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
struct SelectorParseOptions {
    allow_has: bool,
    allow_scope_anchor: bool,
    allow_pseudo_elements: bool,
}

impl SelectorParseOptions {
    const fn standard() -> Self {
        Self {
            allow_has: true,
            allow_scope_anchor: false,
            allow_pseudo_elements: true,
        }
    }

    const fn without_nested_has() -> Self {
        Self {
            allow_has: false,
            allow_scope_anchor: false,
            allow_pseudo_elements: true,
        }
    }

    const fn scoped_style() -> Self {
        Self {
            allow_has: true,
            allow_scope_anchor: true,
            allow_pseudo_elements: true,
        }
    }

    const fn scope_boundary() -> Self {
        Self {
            allow_has: true,
            allow_scope_anchor: false,
            allow_pseudo_elements: false,
        }
    }

    const fn without_pseudo_elements(self) -> Self {
        Self {
            allow_has: self.allow_has,
            allow_scope_anchor: self.allow_scope_anchor,
            allow_pseudo_elements: false,
        }
    }
}

fn parse_rule_selector_list_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<CssSelector>, ParseError<'i, Error>> {
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_rule_selector_with_options(input, options, recovery)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    input.expect_exhausted().map_err(selector_basic)?;
    Ok(selectors)
}

fn parse_rule_selector_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssSelector, ParseError<'i, Error>> {
    let first = parse_compound_selector_model_with_options(input, options, recovery)?;
    parse_selector_after_first_compound(input, first, options, recovery)
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
        match input.next_including_whitespace() {
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
            Ok(Token::Delim('|')) => {
                return Err(invalid_selector(
                    input,
                    "unsupported selector combinator `||`",
                ));
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
                let message = format!("unexpected selector token `{}`", token.to_css_string());
                input.reset(&state);
                return Err(invalid_selector(input, message));
            }
        }
    }

    if rest.is_empty() {
        Ok(compound_selector_to_selector(first))
    } else {
        CssComplexSelector::try_new(first, rest)
            .map(CssSelector::Complex)
            .ok_or_else(|| invalid_selector(input, "pseudo-element selector must be terminal"))
    }
}

pub(super) fn parse_complex_selector_part<'i, 't>(
    input: &mut Parser<'i, 't>,
    combinator: CssSelectorCombinator,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssComplexSelectorPart, ParseError<'i, Error>> {
    parse_complex_selector_part_with_options(
        input,
        combinator,
        SelectorParseOptions::standard(),
        recovery,
    )
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

pub(super) fn parse_compound_selector_model<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssCompoundSelector, ParseError<'i, Error>> {
    parse_compound_selector_model_with_options(input, SelectorParseOptions::standard(), recovery)
}

struct ParsedTypeSelector {
    name: CssQualifiedSelectorName,
    legacy_projection: bool,
}

fn parse_type_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &SelectorRecovery<'_>,
) -> std::result::Result<Option<ParsedTypeSelector>, ParseError<'i, Error>> {
    let start = input.state();
    let first = match input.next_including_whitespace() {
        Ok(token) => token.clone(),
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&start);
            return Ok(None);
        }
        Err(error) => return Err(selector_basic(error)),
    };

    match first {
        Token::Ident(prefix_or_name) => {
            let prefix_or_name = prefix_or_name.to_string();
            let after_ident = input.state();
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => {
                    let local_start = input.state();
                    let local_name = parse_qualified_local_name(input)?;
                    let Some(prefix) = recovery.named_namespace(&prefix_or_name) else {
                        input.reset(&local_start);
                        return Err(invalid_selector(
                            input,
                            format!("undeclared selector namespace prefix `{prefix_or_name}`"),
                        ));
                    };
                    let namespace = CssNamespaceConstraint::Named(prefix);
                    let name = if let Some(local_name) = local_name {
                        CssQualifiedSelectorName::new(namespace, local_name)
                    } else {
                        CssQualifiedSelectorName::universal(namespace)
                    };
                    Ok(Some(ParsedTypeSelector {
                        name,
                        legacy_projection: false,
                    }))
                }
                Ok(_) => {
                    input.reset(&after_ident);
                    let namespace = recovery.unqualified_type_namespace();
                    let legacy_projection = matches!(namespace, CssNamespaceConstraint::Any);
                    Ok(Some(ParsedTypeSelector {
                        name: CssQualifiedSelectorName::new(namespace, prefix_or_name),
                        legacy_projection,
                    }))
                }
                Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                    input.reset(&after_ident);
                    let namespace = recovery.unqualified_type_namespace();
                    let legacy_projection = matches!(namespace, CssNamespaceConstraint::Any);
                    Ok(Some(ParsedTypeSelector {
                        name: CssQualifiedSelectorName::new(namespace, prefix_or_name),
                        legacy_projection,
                    }))
                }
                Err(error) => Err(selector_basic(error)),
            }
        }
        Token::Delim('*') => {
            let after_star = input.state();
            if matches!(input.next_including_whitespace(), Ok(Token::Delim('|'))) {
                let local_name = parse_qualified_local_name(input)?;
                let namespace = CssNamespaceConstraint::Any;
                let name = if let Some(local_name) = local_name {
                    CssQualifiedSelectorName::new(namespace, local_name)
                } else {
                    CssQualifiedSelectorName::universal(namespace)
                };
                Ok(Some(ParsedTypeSelector {
                    name,
                    legacy_projection: false,
                }))
            } else {
                input.reset(&after_star);
                Ok(Some(ParsedTypeSelector {
                    name: CssQualifiedSelectorName::universal(
                        recovery.unqualified_type_namespace(),
                    ),
                    legacy_projection: false,
                }))
            }
        }
        Token::Delim('|') => {
            let local_name = parse_qualified_local_name(input)?;
            let namespace = CssNamespaceConstraint::ExplicitNone;
            let name = if let Some(local_name) = local_name {
                CssQualifiedSelectorName::new(namespace, local_name)
            } else {
                CssQualifiedSelectorName::universal(namespace)
            };
            Ok(Some(ParsedTypeSelector {
                name,
                legacy_projection: false,
            }))
        }
        _ => {
            input.reset(&start);
            Ok(None)
        }
    }
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

    let parsed_type_selector = parse_type_selector(input, recovery)?;
    let type_selector = parsed_type_selector.map(|parsed| (parsed.name, parsed.legacy_projection));
    let mut scope_anchor = false;
    let mut id_names = Vec::new();
    let mut class_names = Vec::new();
    let mut attributes = Vec::new();
    let mut pseudo_classes = Vec::new();
    let mut pseudo_elements = None;

    loop {
        let state = input.state();
        match input.next_including_whitespace() {
            Ok(Token::WhiteSpace(_)) => {
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
            if !options.allow_scope_anchor {
                return Err(invalid_selector(
                    input,
                    "scope anchor selector `&` is only supported inside scoped rules",
                ));
            }
            if scope_anchor {
                return Err(invalid_selector(
                    input,
                    "scope anchor selector `&` is only supported once per compound selector",
                ));
            }
            scope_anchor = true;
            continue;
        }

        if input.try_parse(|input| input.expect_delim('.')).is_ok() {
            let class = input.expect_ident_cloned().map_err(selector_basic)?;
            let class = class.to_string();
            class_names.push(class);
            continue;
        }

        if input.try_parse(Parser::expect_square_bracket_block).is_ok() {
            let attribute =
                input.parse_nested_block(|input| parse_attribute_selector(input, recovery))?;
            attributes.push(attribute);
            continue;
        }

        if input.try_parse(Parser::expect_colon).is_ok() {
            if input.try_parse(Parser::expect_colon).is_ok() {
                if !options.allow_pseudo_elements {
                    return Err(invalid_selector(
                        input,
                        "pseudo-elements are not supported in this selector context",
                    ));
                }
                let sequence = parse_pseudo_element_sequence(input)?;
                pseudo_elements = Some(sequence);
            } else if let Ok(first) = input.try_parse(parse_legacy_pseudo_element) {
                if !options.allow_pseudo_elements {
                    return Err(invalid_selector(
                        input,
                        "pseudo-elements are not supported in this selector context",
                    ));
                }
                pseudo_elements = Some(parse_pseudo_element_sequence_from_first(input, first)?);
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
                return Err(invalid_selector(input, "unsupported selector namespace"));
            }
            Ok(token) => {
                let message = format!("unexpected selector token `{}`", token.to_css_string());
                input.reset(&state);
                if type_selector.is_none()
                    && !scope_anchor
                    && id_names.is_empty()
                    && class_names.is_empty()
                    && attributes.is_empty()
                    && pseudo_classes.is_empty()
                    && pseudo_elements.is_none()
                {
                    return Err(invalid_selector(input, message));
                }
                break;
            }
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => break,
            Err(error) => return Err(selector_basic(error)),
        }
    }

    if type_selector.is_none()
        && !scope_anchor
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
            scope_anchor,
            type_selector,
            id_names,
            class_names,
            attributes,
            pseudo_classes,
            pseudo_elements,
        ),
    )
}

fn compound_selector_to_selector(selector: CssCompoundSelector) -> CssSelector {
    if selector.has_scope_anchor() || selector.has_pseudo_elements() {
        return CssSelector::Compound(selector);
    }

    if let (None, [], [class], [], []) = (
        selector.tag(),
        selector.ids(),
        selector.classes(),
        selector.attributes(),
        selector.pseudo_classes(),
    ) {
        return CssSelector::Class(class.clone());
    }
    if selector.has_legacy_type_projection()
        && let (Some(tag), [], [], [], []) = (
            selector.tag(),
            selector.ids(),
            selector.classes(),
            selector.attributes(),
            selector.pseudo_classes(),
        )
    {
        return CssSelector::Tag(tag.clone());
    }
    if let (None, [key], [], [], []) = (
        selector.tag(),
        selector.ids(),
        selector.classes(),
        selector.attributes(),
        selector.pseudo_classes(),
    ) {
        return CssSelector::Key(key.clone());
    }
    if let (None, [], [], [], [pseudo_class]) = (
        selector.tag(),
        selector.ids(),
        selector.classes(),
        selector.attributes(),
        selector.pseudo_classes(),
    ) {
        return CssSelector::PseudoClass(pseudo_class.clone());
    }
    CssSelector::Compound(selector)
}

fn parse_pseudo_element_sequence<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPseudoElementSequence, ParseError<'i, Error>> {
    let first = parse_pseudo_element(input)?;
    parse_pseudo_element_sequence_from_first(input, first)
}

fn parse_pseudo_element_sequence_from_first<'i, 't>(
    input: &mut Parser<'i, 't>,
    first: CssPseudoElement,
) -> std::result::Result<CssPseudoElementSequence, ParseError<'i, Error>> {
    let mut pseudo_elements = vec![first];
    loop {
        if input
            .try_parse(|input| {
                input.expect_colon()?;
                input.expect_colon()
            })
            .is_err()
        {
            break;
        }
        pseudo_elements.push(parse_pseudo_element(input)?);
    }

    CssPseudoElementSequence::try_new(pseudo_elements)
        .ok_or_else(|| invalid_selector(input, "unsupported pseudo-element sequence"))
}

fn parse_legacy_pseudo_element<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPseudoElement, ParseError<'i, Error>> {
    let name = input.expect_ident_cloned()?;
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
) -> std::result::Result<CssPseudoElement, ParseError<'i, Error>> {
    let state = input.state();
    match input.next() {
        Ok(Token::Ident(name)) => match_ignore_ascii_case! { &name,
            "before" => Ok(CssPseudoElement::Before),
            "after" => Ok(CssPseudoElement::After),
            "first-line" => Ok(CssPseudoElement::FirstLine),
            "first-letter" => Ok(CssPseudoElement::FirstLetter),
            "marker" => Ok(CssPseudoElement::Marker),
            "selection" => Ok(CssPseudoElement::Selection),
            "backdrop" => Ok(CssPseudoElement::Backdrop),
            _ => {
                let message = format!("unsupported pseudo-element `::{name}`");
                Err(invalid_selector(input, message))
            }
        },
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

fn parse_attribute_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &SelectorRecovery<'_>,
) -> std::result::Result<CssAttributeSelector, ParseError<'i, Error>> {
    consume_selector_whitespace(input)?;
    let (namespace, name) = parse_attribute_selector_name(input, recovery)?;

    let matcher = match input.next() {
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            return Ok(CssAttributeSelector::new_qualified(
                namespace,
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
        namespace,
        name,
        matcher,
        case_sensitivity,
    ))
}

fn parse_attribute_selector_name<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &SelectorRecovery<'_>,
) -> std::result::Result<(CssNamespaceConstraint, CssAttributeName), ParseError<'i, Error>> {
    match input.next_including_whitespace() {
        Ok(Token::Ident(prefix_or_name)) => {
            let prefix_or_name = prefix_or_name.to_string();
            let after_ident = input.state();
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => {
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
                    Ok((
                        CssNamespaceConstraint::Named(prefix),
                        CssAttributeName::new(local_name),
                    ))
                }
                Ok(_) => {
                    input.reset(&after_ident);
                    Ok((
                        CssNamespaceConstraint::ExplicitNone,
                        CssAttributeName::new(prefix_or_name),
                    ))
                }
                Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                    input.reset(&after_ident);
                    Ok((
                        CssNamespaceConstraint::ExplicitNone,
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
                Ok(Token::Ident(name)) => Ok((
                    CssNamespaceConstraint::Any,
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
            Ok(Token::Ident(name)) => Ok((
                CssNamespaceConstraint::ExplicitNone,
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
    match input.next() {
        Ok(Token::Ident(name)) => {
            let name = name.clone();
            parse_named_pseudo_class(name.as_ref(), input)
        }
        Ok(Token::Function(name)) => {
            let name = name.clone();
            let mut depth = recovery.state.enter_component_block(
                recovery.source,
                input,
                "baseline.selector.complex",
            )?;
            let result = input.parse_nested_block(|input| {
                parse_function_pseudo_class(name.as_ref(), input, options, recovery)
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
    input: &Parser<'i, '_>,
) -> std::result::Result<CssPseudoClass, ParseError<'i, Error>> {
    match_ignore_ascii_case! { name,
        "root" => Ok(CssPseudoClass::Root),
        "scope" => Ok(CssPseudoClass::Scope),
        "link" => Ok(CssPseudoClass::Link),
        "visited" => Ok(CssPseudoClass::Visited),
        "target" => Ok(CssPseudoClass::Target),
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
        _ => Err(invalid_selector(input, format!("unsupported pseudo-class `:{name}`"))),
    }
}

#[inline(never)]
fn parse_function_pseudo_class<'i, 't>(
    name: &str,
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssPseudoClass, ParseError<'i, Error>> {
    let pseudo_class = match_ignore_ascii_case! { name,
        "nth-child" => CssPseudoClass::NthChild(parse_nth_child_pattern(input, options, recovery)?),
        "nth-last-child" => CssPseudoClass::NthLastChild(parse_nth_child_pattern(input, options, recovery)?),
        "nth-of-type" => CssPseudoClass::NthOfType(parse_nth_pattern(input)?),
        "nth-last-of-type" => CssPseudoClass::NthLastOfType(parse_nth_pattern(input)?),
        "lang" => CssPseudoClass::Lang(parse_language_range(input)?),
        "not" => CssPseudoClass::Not(parse_pseudo_selector_list_with_options(input, options.without_pseudo_elements(), recovery)?),
        "is" => CssPseudoClass::Is(parse_forgiving_pseudo_selector_list(input, options.without_pseudo_elements(), recovery)?),
        "where" => CssPseudoClass::Where(parse_forgiving_pseudo_selector_list(input, options.without_pseudo_elements(), recovery)?),
        "has" if options.allow_has => CssPseudoClass::Has(parse_has_relative_selector_list(input, recovery)?),
        "has" => return Err(invalid_selector(input, "nested `:has()` is unsupported")),
        _ => return Err(invalid_selector(input, format!("unsupported pseudo-class `:{name}(`"))),
    };
    input.expect_exhausted().map_err(selector_basic)?;
    Ok(pseudo_class)
}

fn parse_language_range<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLanguageRange, ParseError<'i, Error>> {
    let value = input.expect_ident_cloned().map_err(selector_basic)?;
    CssLanguageRange::try_new(value.to_string())
        .ok_or_else(|| invalid_selector(input, "`:lang()` requires one CSS identifier"))
}

fn parse_pseudo_selector_list_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssPseudoSelectorList, ParseError<'i, Error>> {
    let selectors = parse_pseudo_selector_list_items_with_options(input, options, recovery)?;
    CssPseudoSelectorList::try_new(selectors)
        .ok_or_else(|| invalid_selector(input, "pseudo-class selector list must not be empty"))
}

fn parse_pseudo_selector_list_items_with_options<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: SelectorParseOptions,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<CssSelector>, ParseError<'i, Error>> {
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_rule_selector_with_options(input, options, recovery)?);
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
    let mut selectors = Vec::new();
    let mut preceding_comma = None;
    loop {
        let member_start = input.position().byte_index();
        let result = input.parse_until_before(Delimiter::Comma, |member| {
            parse_rule_selector_with_options(member, options, recovery)
        });
        let member_end = input.position().byte_index();
        let comma_start = member_end;
        let following_comma = match input.next() {
            Ok(Token::Comma) => Some((comma_start, input.position().byte_index())),
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => None,
            Ok(_) => return Err(invalid_selector(input, "invalid selector-list delimiter")),
            Err(error) => return Err(selector_basic(error)),
        };

        match result {
            Ok(selector) => selectors.push(selector),
            Err(error) => recovery.drop_forgiving_member(
                error,
                member_start,
                member_end,
                following_comma,
                preceding_comma,
            ),
        }

        let Some(comma) = following_comma else {
            break;
        };
        preceding_comma = Some(comma);
    }

    Ok(CssPseudoSelectorList::new_forgiving(selectors))
}

fn parse_has_relative_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssRelativeSelectorList, ParseError<'i, Error>> {
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_has_relative_selector(input, recovery)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    input.expect_exhausted().map_err(selector_basic)?;
    CssRelativeSelectorList::try_new(selectors)
        .ok_or_else(|| invalid_selector(input, "relative selector list must not be empty"))
}

fn parse_has_relative_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssRelativeSelector, ParseError<'i, Error>> {
    consume_selector_whitespace(input)?;
    let state = input.state();
    match input.next_including_whitespace() {
        Ok(Token::Delim('>')) => {
            parse_selector_after_leading_combinator(input, CssSelectorCombinator::Child, recovery)
        }
        Ok(Token::Delim('+')) => parse_selector_after_leading_combinator(
            input,
            CssSelectorCombinator::NextSibling,
            recovery,
        ),
        Ok(Token::Delim('~')) => parse_selector_after_leading_combinator(
            input,
            CssSelectorCombinator::SubsequentSibling,
            recovery,
        ),
        Ok(Token::Delim('|')) => Err(invalid_selector(
            input,
            "unsupported selector combinator `||`",
        )),
        Ok(_) => {
            input.reset(&state);
            let selector = parse_rule_selector_with_options(
                input,
                SelectorParseOptions::without_nested_has().without_pseudo_elements(),
                recovery,
            )?;
            Ok(CssRelativeSelector::new(
                CssSelectorCombinator::Descendant,
                selector,
            ))
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&state);
            Err(invalid_selector(input, "relative selector is missing"))
        }
        Err(error) => Err(selector_basic(error)),
    }
}

fn parse_selector_after_leading_combinator<'i, 't>(
    input: &mut Parser<'i, 't>,
    combinator: CssSelectorCombinator,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<CssRelativeSelector, ParseError<'i, Error>> {
    parse_selector_after_leading_combinator_with_options(
        input,
        combinator,
        SelectorParseOptions::without_nested_has().without_pseudo_elements(),
        recovery,
    )
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

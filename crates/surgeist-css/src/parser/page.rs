use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, match_ignore_ascii_case,
};

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::{
    DeclarationMode, block_item_diagnostic, is_declaration_recovery_unit, parse_declaration_core,
    top_level_only_at_rule_placement,
};
use crate::error::{
    CssFeatureId, Error, basic, invalid_at_rule_body, property_name_error, unexpected_at,
    with_property_context,
};
use crate::properties::{CssKnownProperty, CssKnownPropertyValueRef};
use crate::syntax::*;
use crate::{CssBoxSideKind, CssComponentValueRef, CssMarginValue, CssValueTokenRef};

pub(super) static IMPLEMENTED_SELECTORS: &[CssFeatureId] = &[
    CssFeatureId::new("official.selector.page-pseudo"),
    CssFeatureId::new("official.selector.logical-page-pseudo"),
];

pub(super) fn parse_page_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    snapshot: &crate::CssSourceSnapshot,
) -> Result<CssPageSelectorList, ParseError<'i, Error>> {
    let start = input.position().byte_index();
    let selectors = if input.is_exhausted() {
        Vec::new()
    } else {
        input.parse_comma_separated(|input| {
            input.skip_whitespace();
            let start = input.position().byte_index();
            let name = input
                .try_parse(|input| input.expect_ident_cloned())
                .ok()
                .map(|name| {
                    crate::CssIdent::try_new(name.to_string()).expect("decoded parser identifier")
                });
            let mut pseudos = Vec::new();
            while !input.is_exhausted() {
                let location = input.current_source_location();
                match input.next_including_whitespace().cloned().map_err(basic)? {
                    cssparser::Token::WhiteSpace(_) => {
                        input.skip_whitespace();
                        input.expect_exhausted().map_err(basic)?;
                        break;
                    }
                    cssparser::Token::Colon => {}
                    _ => return Err(unexpected_at(location)),
                }
                let location = input.current_source_location();
                let cssparser::Token::Ident(name) =
                    input.next_including_whitespace().cloned().map_err(basic)?
                else {
                    return Err(unexpected_at(location));
                };
                let pseudo = match_ignore_ascii_case! { &name,
                    "left" => CssPagePseudo::Left, "right" => CssPagePseudo::Right,
                    "first" => CssPagePseudo::First, "blank" => CssPagePseudo::Blank,
                    "recto" => CssPagePseudo::Recto, "verso" => CssPagePseudo::Verso,
                    _ => return Err(unexpected_at(location)),
                };
                pseudos.push(pseudo);
            }
            if name.is_none() && pseudos.is_empty() {
                return Err(unexpected_at(input.current_source_location()));
            }
            let origin =
                crate::CssParsedOrigin::from_range(snapshot, start..input.position().byte_index())
                    .expect("genuine Page selector");
            Ok(CssPageSelector::from_parsed(name, pseudos, origin))
        })?
    };
    let origin = crate::CssParsedOrigin::from_range(snapshot, start..input.position().byte_index())
        .expect("genuine Page selector list");
    Ok(CssPageSelectorList::from_parsed(selectors, origin))
}

pub(super) fn parse_page_rule<'i, 't>(
    source: &'i str,
    selector: CssPageSelectorList,
    input: &mut Parser<'i, 't>,
    start: &ParserState,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssPageRule, ParseError<'i, Error>> {
    let declarations = parse_body(source, input, diagnostics, recovery);
    Ok(CssPageRule::new(
        selector,
        declarations,
        crate::source::CssSourcePosition::from_cssparser(start.position(), start.source_location()),
    ))
}

pub(super) fn parse_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> CssPageBody {
    parse_body_inner(source, input, diagnostics, recovery, false)
}
pub(super) fn parse_margin_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> CssMarginDeclarationBlock {
    let body = parse_body_inner(source, input, diagnostics, recovery, true);
    CssMarginDeclarationBlock::from_parsed(body.declarations().properties().clone())
}
fn parse_body_inner<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
    margin: bool,
) -> CssPageBody {
    let mut declarations = Vec::new();
    let mut margin_rules = Vec::new();
    let mut parser = PageBodyParser {
        source,
        recovery,
        diagnostics: Vec::new(),
        margin,
    };
    let mut items = RuleBodyParser::new(input, &mut parser);
    loop {
        let progress = RecoveryProgress::record(items.input);
        let Some(item) = items.next() else {
            break;
        };
        let retained = item.is_ok();
        let progress_outcome = progress.finish(items.input, retained);
        let unit_end = items.input.position().byte_index();
        match item {
            Ok(PageBodyItem::Declaration(declaration)) => declarations.push(declaration),
            Ok(PageBodyItem::Margin(rule)) => margin_rules.push(rule),
            Err((error, failed_unit)) => {
                let action = if is_declaration_recovery_unit(failed_unit) {
                    crate::CssRecoveryAction::DropDeclaration
                } else {
                    crate::CssRecoveryAction::DropAtRule
                };
                if let Some(diagnostic) =
                    block_item_diagnostic(source, error, failed_unit, unit_end, action)
                {
                    diagnostics.push(diagnostic);
                }
            }
        }
        if progress_outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }

    diagnostics.append(&mut items.parser.diagnostics);
    CssPageBody::new(
        CssPageDeclarationBlock::from_parsed(declarations),
        margin_rules,
    )
}

struct PageBodyParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    margin: bool,
}

/// The detached boundary admits only the existing Page margin branch. Other
/// Page-body items never become a detached margin occurrence.
struct DetachedMarginParser<'s> {
    page: PageBodyParser<'s>,
}
impl<'i> AtRuleParser<'i> for DetachedMarginParser<'i> {
    type Prelude = PageBodyAtRulePrelude<'i>;
    type AtRule = PageBodyItem;
    type Error = Error;
    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Error>> {
        let location = input.current_source_location();
        match AtRuleParser::parse_prelude(&mut self.page, name.clone(), input)? {
            prelude @ PageBodyAtRulePrelude::MarginBox(_) => Ok(prelude),
            PageBodyAtRulePrelude::TopLevelOnly(name, location) => {
                Err(top_level_only_at_rule_placement(location, name.as_ref()))
            }
            PageBodyAtRulePrelude::Other => {
                Err(location.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name)))
            }
        }
    }
    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        AtRuleParser::rule_without_block(&mut self.page, prelude, start)
    }
    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Error>> {
        AtRuleParser::parse_block(&mut self.page, prelude, start, input)
    }
}
impl<'i> QualifiedRuleParser<'i> for DetachedMarginParser<'i> {
    type Prelude = ();
    type QualifiedRule = PageBodyItem;
    type Error = Error;
}

pub(super) fn admit_margin_rule(
    source: &str,
    recovery: RecoveryState,
    envelope: &super::rule_candidate::RuleEnvelope,
) -> crate::CssParseReport<Option<CssMarginRule>> {
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = cssparser::ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let mut parser = DetachedMarginParser {
        page: PageBodyParser {
            source,
            recovery: recovery.clone(),
            diagnostics: Vec::new(),
            margin: false,
        },
    };
    let admitted =
        super::syntax_bridge::admit_envelope(source, &mut input, &mut parser, &recovery, envelope);
    let (margin, mut diagnostics) = match admitted {
        Ok(PageBodyItem::Margin(margin)) => (Some(margin), parser.page.diagnostics),
        Ok(PageBodyItem::Declaration(_)) => unreachable!("rule driver cannot emit a declaration"),
        Err(error) => (
            None,
            vec![super::fragments::reject(
                source,
                error,
                crate::CssRecoveryAction::RejectInput,
            )],
        ),
    };
    let margin = if super::recovery::has_resource_failure(&diagnostics) {
        None
    } else {
        margin
    };
    diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
    crate::CssParseReport::new(margin, diagnostics)
}

pub(super) fn parse_contents(
    source: &str,
    limits: crate::CssComponentValueLimits,
    context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssPageDeclarationBlock>> {
    let (declarations, diagnostics) =
        super::declaration_block::parse_contents(source, limits, context, |recovery| {
            PageBodyParser {
                source,
                recovery,
                diagnostics: Vec::new(),
                margin: false,
            }
        })
        .into_parts();
    crate::CssParseReport::new(
        declarations.map(CssPageDeclarationBlock::from_parsed),
        diagnostics,
    )
}

impl super::declaration_block::Receiver for PageBodyParser<'_> {
    type Declaration = CssPageDeclaration;
    fn check_name<'i>(
        &self,
        name: &str,
        location: cssparser::SourceLocation,
    ) -> Result<(), ParseError<'i, Error>> {
        if CssPageDescriptorKind::from_name(name).is_some() {
            Ok(())
        } else {
            page_property_name(name, location).map(|_| ())
        }
    }
    fn parse_value<'i>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, '_>,
        start: &ParserState,
        _recovery: &RecoveryState,
    ) -> Result<CssPageDeclaration, ParseError<'i, Error>>
    where
        Self: 'i,
    {
        DeclarationParser::parse_value(self, name, input, start).map(|item| match item {
            PageBodyItem::Declaration(declaration) => declaration,
            PageBodyItem::Margin(_) => {
                unreachable!("declaration callback cannot emit a margin rule")
            }
        })
    }
    fn take_diagnostics(&mut self) -> Vec<crate::CssRecoveryDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}

fn page_property_name<'i>(
    name: &str,
    location: cssparser::SourceLocation,
) -> Result<Option<CssKnownProperty>, ParseError<'i, Error>> {
    let property = CssKnownProperty::from_name(name);
    if property.is_none() && !name.starts_with("--") {
        Err(property_name_error(location, name))
    } else {
        Ok(property)
    }
}

enum PageBodyItem {
    Declaration(CssPageDeclaration),
    Margin(CssMarginRule),
}

enum PageBodyAtRulePrelude<'i> {
    MarginBox(CssMarginBox),
    TopLevelOnly(CowRcStr<'i>, cssparser::SourceLocation),
    Other,
}

impl<'i> AtRuleParser<'i> for PageBodyParser<'i> {
    type Prelude = PageBodyAtRulePrelude<'i>;
    type AtRule = PageBodyItem;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let location = input.current_source_location();
        if let Some(margin) = CssMarginBox::from_name(name.as_ref()) {
            input.expect_exhausted().map_err(basic)?;
            if !self.margin {
                return Ok(PageBodyAtRulePrelude::MarginBox(margin));
            }
        }
        while input.next_including_whitespace_and_comments().is_ok() {}
        if name.eq_ignore_ascii_case("counter-style") || name.eq_ignore_ascii_case("page") {
            return Ok(PageBodyAtRulePrelude::TopLevelOnly(name, location));
        }
        Ok(PageBodyAtRulePrelude::Other)
    }

    fn rule_without_block(
        &mut self,
        _prelude: Self::Prelude,
        _start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        Err(())
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Self::Error>> {
        match prelude {
            PageBodyAtRulePrelude::MarginBox(name) => {
                let mut depth =
                    self.recovery
                        .enter_rule_block(self.source, input, "later.rule.page")?;
                let declarations = parse_margin_body(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                );
                depth.retain();
                Ok(PageBodyItem::Margin(CssMarginRule::from_parsed(
                    name,
                    declarations,
                    crate::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                )))
            }
            PageBodyAtRulePrelude::TopLevelOnly(name, location) => {
                Err(top_level_only_at_rule_placement(location, name.as_ref()))
            }
            PageBodyAtRulePrelude::Other => Err(invalid_at_rule_body(
                input,
                "page",
                "later.rule.page",
                "page-context margin declarations",
            )),
        }
    }
}

impl<'i> QualifiedRuleParser<'i> for PageBodyParser<'i> {
    type Prelude = ();
    type QualifiedRule = PageBodyItem;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        Err(invalid_at_rule_body(
            input,
            "page",
            "later.rule.page",
            "page-context margin declarations",
        ))
    }
}

impl<'i> RuleBodyItemParser<'i, PageBodyItem, Error> for PageBodyParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

impl<'i, 's: 'i> DeclarationParser<'i> for PageBodyParser<'s> {
    type Declaration = PageBodyItem;
    type Error = Error;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let implicit_closures =
            self.recovery
                .check_component_values(self.source, input, "css.declaration")?;
        if let Some(kind) = CssPageDescriptorKind::from_name(name.as_ref()) {
            if self.margin {
                return Err(property_name_error(
                    declaration_start.source_location(),
                    name.as_ref(),
                ));
            }
            let value_start = input.state();
            input.reset(declaration_start);
            input.expect_ident().map_err(basic)?;
            let name_origin = crate::CssParsedOrigin::from_range(
                self.recovery.source_snapshot(),
                declaration_start.position().byte_index()..input.position().byte_index(),
            )
            .expect("real descriptor name");
            input.reset(&value_start);
            let (value, importance) = input
                .parse_until_before(cssparser::Delimiter::Bang, |input| {
                    super::collect_declaration_value(
                        input,
                        self.recovery.source_snapshot(),
                        |input| {
                            parse_descriptor_value(input, kind, self.recovery.source_snapshot())
                        },
                    )
                })
                .and_then(|(data, components, origin)| {
                    let importance = if input.is_exhausted() {
                        crate::CssImportance::Normal
                    } else {
                        input.expect_delim('!').map_err(basic)?;
                        input.expect_ident_matching("important").map_err(basic)?;
                        input.expect_exhausted().map_err(basic)?;
                        crate::CssImportance::Important
                    };
                    Ok((
                        CssPageDescriptorValue::from_parsed(kind, data, components, origin),
                        importance,
                    ))
                })
                .map_err(|error| {
                    crate::error::with_descriptor_context(error, "page", kind.css_name())
                })?;
            self.recovery.retain_component_closures(implicit_closures);
            return Ok(PageBodyItem::Declaration(CssPageDeclaration::Descriptor(
                CssPageDescriptor::from_parsed(value, importance, name_origin),
            )));
        }
        let property = page_property_name(name.as_ref(), declaration_start.source_location())?;
        if property.is_some_and(|property| !is_page_margin_property(property)) {
            return Err(with_property_context(
                unexpected_at(input.current_source_location()),
                name.as_ref(),
            ));
        }

        let parsed = parse_declaration_core(
            DeclarationMode::Ordinary,
            name.clone(),
            input,
            declaration_start,
            self.recovery.source_snapshot(),
            self.recovery.parser_context(),
        )?;
        if let Some(violation) = page_declaration_violation(&parsed.body, &parsed.components) {
            let component = violation
                .component()
                .or_else(|| first_page_component(&parsed.components));
            let error = component.map_or_else(
                || unexpected_at(input.current_source_location()),
                |component| {
                    crate::error::unexpected_component_value(
                        component,
                        "an admitted Page margin value",
                    )
                },
            );
            return Err(with_property_context(error, name.as_ref()));
        }
        self.recovery.retain_component_closures(implicit_closures);
        Ok(PageBodyItem::Declaration(CssPageDeclaration::Property(
            parsed.into_declaration_in_context(DeclarationContext::Page),
        )))
    }
}

/// Paged Media 3 Appendix A membership only. Shared property metadata/grammar stays authoritative.
pub(crate) const fn is_page_margin_property(property: CssKnownProperty) -> bool {
    matches!(
        property,
        CssKnownProperty::Quotes
            | CssKnownProperty::Overflow
            | CssKnownProperty::BorderStyle
            | CssKnownProperty::WordSpacing
            | CssKnownProperty::Direction
            | CssKnownProperty::Visibility
            | CssKnownProperty::Content
            | CssKnownProperty::CounterReset
            | CssKnownProperty::CounterIncrement
            | CssKnownProperty::Width
            | CssKnownProperty::Height
            | CssKnownProperty::MinWidth
            | CssKnownProperty::MinHeight
            | CssKnownProperty::MaxWidth
            | CssKnownProperty::MaxHeight
            | CssKnownProperty::FontSize
            | CssKnownProperty::LineHeight
            | CssKnownProperty::UnicodeBidi
            | CssKnownProperty::TextAlign
            | CssKnownProperty::TextIndent
            | CssKnownProperty::VerticalAlign
            | CssKnownProperty::FontFamily
            | CssKnownProperty::Font
            | CssKnownProperty::FontWeight
            | CssKnownProperty::FontStyle
            | CssKnownProperty::FontVariant
            | CssKnownProperty::LetterSpacing
            | CssKnownProperty::WhiteSpace
            | CssKnownProperty::TextDecoration
            | CssKnownProperty::TextTransform
            | CssKnownProperty::ZIndex
            | CssKnownProperty::Margin
            | CssKnownProperty::MarginTop
            | CssKnownProperty::MarginRight
            | CssKnownProperty::MarginBottom
            | CssKnownProperty::MarginLeft
            | CssKnownProperty::Padding
            | CssKnownProperty::PaddingTop
            | CssKnownProperty::PaddingRight
            | CssKnownProperty::PaddingBottom
            | CssKnownProperty::PaddingLeft
            | CssKnownProperty::Border
            | CssKnownProperty::BorderTop
            | CssKnownProperty::BorderRight
            | CssKnownProperty::BorderBottom
            | CssKnownProperty::BorderLeft
            | CssKnownProperty::BorderWidth
            | CssKnownProperty::BorderTopWidth
            | CssKnownProperty::BorderRightWidth
            | CssKnownProperty::BorderBottomWidth
            | CssKnownProperty::BorderLeftWidth
            | CssKnownProperty::Color
            | CssKnownProperty::Background
            | CssKnownProperty::BackgroundColor
            | CssKnownProperty::BorderColor
            | CssKnownProperty::BorderTopColor
            | CssKnownProperty::BorderRightColor
            | CssKnownProperty::BorderBottomColor
            | CssKnownProperty::BorderLeftColor
            | CssKnownProperty::BackgroundImage
            | CssKnownProperty::BackgroundPosition
            | CssKnownProperty::BackgroundRepeat
            | CssKnownProperty::BackgroundAttachment
            | CssKnownProperty::BorderTopStyle
            | CssKnownProperty::BorderRightStyle
            | CssKnownProperty::BorderBottomStyle
            | CssKnownProperty::BorderLeftStyle
            | CssKnownProperty::Outline
            | CssKnownProperty::OutlineColor
            | CssKnownProperty::OutlineStyle
            | CssKnownProperty::OutlineWidth
    )
}

pub(crate) enum PageDeclarationViolation<'a> {
    LogicalMargin,
    NonzeroUnitlessLength(&'a crate::CssComponentValue),
}

pub(crate) fn first_page_component(
    values: &crate::CssComponentValues,
) -> Option<&crate::CssComponentValue> {
    values.items().iter().find(|value| {
        !matches!(
            value.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                | CssComponentValueRef::Comment(_)
        )
    })
}

impl<'a> PageDeclarationViolation<'a> {
    pub(crate) fn component(&self) -> Option<&'a crate::CssComponentValue> {
        match self {
            Self::LogicalMargin => None,
            Self::NonzeroUnitlessLength(value) => Some(value),
        }
    }
}

/// Applies only the additional Page restrictions to a shared checked body.
/// Substitution-dependent values are admitted whole, then checked after the
/// caller supplies a complete replacement. Custom tokens never use margin grammar.
pub(crate) fn page_declaration_violation<'a>(
    body: &'a CssDeclarationBody,
    _components: &'a crate::CssComponentValues,
) -> Option<PageDeclarationViolation<'a>> {
    let CssDeclarationBody::Known(known) = body else {
        return None;
    };
    if known.substitution_dependent().is_some() || known.global().is_some() {
        return None;
    }
    let unitless = match known.property_value() {
        Some(CssKnownPropertyValueRef::Margin(value)) => {
            if value.value().kind() != CssBoxSideKind::Physical {
                return Some(PageDeclarationViolation::LogicalMargin);
            }
            value
                .value()
                .assigned_values()
                .into_iter()
                .find_map(nonzero_unitless_page_length)
        }
        Some(CssKnownPropertyValueRef::MarginTop(value)) => {
            nonzero_unitless_page_length(value.value())
        }
        Some(CssKnownPropertyValueRef::MarginRight(value)) => {
            nonzero_unitless_page_length(value.value())
        }
        Some(CssKnownPropertyValueRef::MarginBottom(value)) => {
            nonzero_unitless_page_length(value.value())
        }
        Some(CssKnownPropertyValueRef::MarginLeft(value)) => {
            nonzero_unitless_page_length(value.value())
        }
        _ => None,
    };
    if let Some(value) = unitless {
        return Some(PageDeclarationViolation::NonzeroUnitlessLength(value));
    }

    None
}

fn nonzero_unitless_page_length(value: &CssMarginValue) -> Option<&crate::CssComponentValue> {
    let CssMarginValue::LengthPercentage(value) = value else {
        return None;
    };
    let component = value.literal_component()?;
    if matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number))
        if crate::exact_decimal::LexicalDecimal::new(number.representation()).len != 0)
    {
        Some(component)
    } else {
        None
    }
}

pub(super) fn parse_descriptor_value<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssPageDescriptorKind,
    snapshot: &crate::CssSourceSnapshot,
) -> Result<crate::syntax::PageValueData, ParseError<'i, Error>> {
    use crate::syntax::PageValueData as V;
    let start = input.state();
    let components =
        crate::CssComponentValues::collect_from_parser(input, snapshot).map_err(|error| {
            crate::error::invalid_component_value(input.current_source_location(), error)
        })?;
    input.reset(&start);
    let numeric = crate::numeric::NumericInputContext::parsed(snapshot);
    if super::variables::descriptor_substitution_qualifies(components.items(), &numeric).map_err(
        |error| crate::error::invalid_component_value(input.current_source_location(), error),
    )? {
        let start = input.position();
        super::descriptor_values::consume_remaining_components(input)?;
        return Ok(V::Pending(crate::CssSubstitutionDependentValue::new(
            crate::CssAuthoredDeclarationValue::new(input.slice_from(start)),
        )));
    }
    if let Ok(global) = input.try_parse(|input| {
        let ident = input.expect_ident_cloned().map_err(basic)?;
        let global = crate::validation::parse_global_keyword(&ident)
            .ok_or_else(|| unexpected_at(input.current_source_location()))?;
        input.expect_exhausted().map_err(basic)?;
        Ok::<_, ParseError<'i, Error>>(global)
    }) {
        return Ok(V::Global(global));
    }
    let value = match kind {
        CssPageDescriptorKind::Size => V::Size(parse_size(input, &numeric)?),
        CssPageDescriptorKind::PageOrientation => {
            let location = input.current_source_location();
            let name = input.expect_ident_cloned().map_err(basic)?;
            V::PageOrientation(
                match_ignore_ascii_case! {&name,"upright"=>CssPageOutputOrientation::Upright,"rotate-left"=>CssPageOutputOrientation::RotateLeft,"rotate-right"=>CssPageOutputOrientation::RotateRight,_=>return Err(unexpected_at(location))},
            )
        }
        CssPageDescriptorKind::Marks => {
            let location = input.current_source_location();
            let name = input.expect_ident_cloned().map_err(basic)?;
            let (crop, cross) = if name.eq_ignore_ascii_case("none") {
                (false, false)
            } else {
                let mut crop = false;
                let mut cross = false;
                for name in std::iter::once(name)
                    .chain(input.try_parse(|input| input.expect_ident_cloned()).ok())
                {
                    match_ignore_ascii_case! {&name,"crop"=>{if crop{return Err(unexpected_at(location));}crop=true;},"cross"=>{if cross{return Err(unexpected_at(location));}cross=true;},_=>return Err(unexpected_at(location))}
                }
                (crop, cross)
            };
            V::Marks(CssPageMarks::new(crop, cross))
        }
        CssPageDescriptorKind::Bleed => V::Bleed(
            if input
                .try_parse(|input| input.expect_ident_matching("auto"))
                .is_ok()
            {
                CssPageBleed::Auto
            } else {
                CssPageBleed::Length(super::values::parse_length(input, &numeric)?)
            },
        ),
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(value)
}
fn parse_size<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssPageSizeValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssPageSizeValue::Auto);
    }
    if let Ok(first) =
        input.try_parse(|input| super::values::parse_nonnegative_length(input, numeric))
    {
        let second = if input.is_exhausted() {
            None
        } else {
            Some(super::values::parse_nonnegative_length(input, numeric)?)
        };
        return Ok(CssPageSizeValue::Dimensions(first, second));
    }
    let mut size = None;
    let mut orientation = None;
    for _ in 0..2 {
        if input.is_exhausted() {
            break;
        }
        let location = input.current_source_location();
        let name = input.expect_ident_cloned().map_err(basic)?;
        if let Some(value) = match_ignore_ascii_case! {&name,"portrait"=>Some(CssPageOrientation::Portrait),"landscape"=>Some(CssPageOrientation::Landscape),_=>None}
        {
            if orientation.replace(value).is_some() {
                return Err(unexpected_at(location));
            }
        } else {
            let value = match_ignore_ascii_case! {&name,"a5"=>CssPageSize::A5,"a4"=>CssPageSize::A4,"a3"=>CssPageSize::A3,"b5"=>CssPageSize::B5,"b4"=>CssPageSize::B4,"jis-b5"=>CssPageSize::JisB5,"jis-b4"=>CssPageSize::JisB4,"letter"=>CssPageSize::Letter,"legal"=>CssPageSize::Legal,"ledger"=>CssPageSize::Ledger,_=>return Err(unexpected_at(location))};
            if size.replace(value).is_some() {
                return Err(unexpected_at(location));
            }
        }
    }
    if size.is_none() && orientation.is_none() {
        return Err(unexpected_at(input.current_source_location()));
    }
    Ok(CssPageSizeValue::Named(
        CssPageNamedSize::try_new(size, orientation).expect("parsed nonempty Page size"),
    ))
}

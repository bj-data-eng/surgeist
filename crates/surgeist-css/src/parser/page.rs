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
) -> Result<Option<CssPageSelector>, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(None);
    }

    input.expect_colon().map_err(basic)?;
    let pseudo = input.expect_ident_cloned().map_err(basic)?;
    let selector = match_ignore_ascii_case! { &pseudo,
        "left" => CssPageSelector::Left,
        "right" => CssPageSelector::Right,
        "first" => CssPageSelector::First,
        "recto" => CssPageSelector::Recto,
        "verso" => CssPageSelector::Verso,
        _ => return Err(unexpected_at(input.current_source_location())),
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(Some(selector))
}

pub(super) fn parse_page_rule<'i, 't>(
    source: &'i str,
    selector: Option<CssPageSelector>,
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
) -> CssDeclarationList {
    let mut declarations = Vec::new();
    let mut parser = PageBodyParser { source, recovery };
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
            Ok(declaration) => declarations.push(declaration),
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

    CssDeclarationList::new(declarations)
}

struct PageBodyParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
}

enum PageBodyAtRulePrelude<'i> {
    MarginBox(CowRcStr<'i>),
    TopLevelOnly(CowRcStr<'i>, cssparser::SourceLocation),
    Other,
}

impl<'i> AtRuleParser<'i> for PageBodyParser<'i> {
    type Prelude = PageBodyAtRulePrelude<'i>;
    type AtRule = CssDeclaration;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let location = input.current_source_location();
        while input.next_including_whitespace_and_comments().is_ok() {}
        if name.eq_ignore_ascii_case("counter-style") || name.eq_ignore_ascii_case("page") {
            return Ok(PageBodyAtRulePrelude::TopLevelOnly(name, location));
        }
        if is_page_margin_box(name.as_ref()) {
            return Ok(PageBodyAtRulePrelude::MarginBox(name));
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
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Self::Error>> {
        match prelude {
            PageBodyAtRulePrelude::MarginBox(name) => {
                Err(input.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name)))
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
    type QualifiedRule = CssDeclaration;
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

impl<'i> RuleBodyItemParser<'i, CssDeclaration, Error> for PageBodyParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

impl<'i> DeclarationParser<'i> for PageBodyParser<'i> {
    type Declaration = CssDeclaration;
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
        let property = CssKnownProperty::from_name(name.as_ref());
        if property.is_none() && !name.starts_with("--") {
            return Err(property_name_error(
                declaration_start.source_location(),
                name.as_ref(),
            ));
        }
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
        Ok(parsed.into_declaration_in_context(DeclarationContext::Page))
    }
}

pub(crate) const fn is_page_margin_property(property: CssKnownProperty) -> bool {
    matches!(
        property,
        CssKnownProperty::Margin
            | CssKnownProperty::MarginTop
            | CssKnownProperty::MarginRight
            | CssKnownProperty::MarginBottom
            | CssKnownProperty::MarginLeft
    )
}

pub(crate) enum PageDeclarationViolation<'a> {
    LogicalMargin,
    NonzeroUnitlessLength(&'a crate::CssComponentValue),
    FontRelativeUnit(&'a crate::CssComponentValue),
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
            Self::NonzeroUnitlessLength(value) | Self::FontRelativeUnit(value) => Some(value),
        }
    }
}

/// Applies only the additional Page restrictions to a shared checked body.
/// Substitution-dependent values are admitted whole, then checked after the
/// caller supplies a complete replacement. Custom tokens never use margin grammar.
pub(crate) fn page_declaration_violation<'a>(
    body: &'a CssDeclarationBody,
    components: &'a crate::CssComponentValues,
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
    // Inspect original components before any numeric projection can eliminate a
    // zero operand. The CSS2 Page exclusion remains lexical even inside math.
    let mut stack = vec![components.items().iter()];
    while let Some(iter) = stack.last_mut() {
        let Some(value) = iter.next() else {
            stack.pop();
            continue;
        };
        match value.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. })
                if unit.eq_ignore_ascii_case("em") || unit.eq_ignore_ascii_case("ex") =>
            {
                return Some(PageDeclarationViolation::FontRelativeUnit(value));
            }
            CssComponentValueRef::Function(function) => {
                stack.push(function.values().items().iter())
            }
            CssComponentValueRef::Block(block) => stack.push(block.values().items().iter()),
            _ => {}
        }
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

fn is_page_margin_box(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "top-left-corner"
            | "top-left"
            | "top-center"
            | "top-right"
            | "top-right-corner"
            | "bottom-left-corner"
            | "bottom-left"
            | "bottom-center"
            | "bottom-right"
            | "bottom-right-corner"
            | "left-top"
            | "left-middle"
            | "left-bottom"
            | "right-top"
            | "right-middle"
            | "right-bottom"
    )
}

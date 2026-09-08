use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, match_ignore_ascii_case,
};

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::{
    DeclarationMode, ParsedDeclaration, block_item_diagnostic, is_declaration_recovery_unit,
    parse_declaration_core, top_level_only_at_rule_placement,
};
use crate::error::{
    CssFeatureId, Error, basic, invalid_at_rule_body, property_name_error, unsupported_value,
    with_property_context,
};
use crate::properties::{CssKnownProperty, CssKnownPropertyValueRef};
use crate::syntax::*;

pub(super) static IMPLEMENTED_SELECTORS: &[CssFeatureId] =
    &[CssFeatureId::new("official.selector.page-pseudo")];

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
        _ => return Err(unsupported_value(input, None, "unsupported page pseudo selector")),
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

    Ok(CssPageRule::new(
        selector,
        CssDeclarationList::new(declarations),
        crate::source::CssSourcePosition::from_cssparser(start.position(), start.source_location()),
    ))
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
        let Some(property) = CssKnownProperty::from_name(name.as_ref()) else {
            return Err(property_name_error(
                declaration_start.source_location(),
                name.as_ref(),
            ));
        };
        if !is_page_margin_property(property) {
            return Err(with_property_context(
                unsupported_value(input, None, "property is not accepted in page context"),
                name.as_ref(),
            ));
        }

        let parsed = parse_declaration_core(
            DeclarationMode::Ordinary,
            name.clone(),
            input,
            declaration_start,
        )?;
        if !is_css2_page_margin_value(&parsed) {
            return Err(with_property_context(
                unsupported_value(input, None, "value is outside the CSS2 page margin domain"),
                name.as_ref(),
            ));
        }
        self.recovery.retain_component_closures(implicit_closures);
        Ok(CssDeclaration::new_with_importance(
            parsed.body,
            parsed.importance,
            parsed.position,
        ))
    }
}

const fn is_page_margin_property(property: CssKnownProperty) -> bool {
    matches!(
        property,
        CssKnownProperty::Margin
            | CssKnownProperty::MarginTop
            | CssKnownProperty::MarginRight
            | CssKnownProperty::MarginBottom
            | CssKnownProperty::MarginLeft
    )
}

fn is_css2_page_margin_value(parsed: &ParsedDeclaration) -> bool {
    let CssDeclarationBody::Known(known) = &parsed.body else {
        return false;
    };
    match known.property_value() {
        Some(CssKnownPropertyValueRef::Margin(value)) => value.i01_subset().is_some_and(|edges| {
            [&edges.top, &edges.right, &edges.bottom, &edges.left]
                .into_iter()
                .all(is_css2_page_length)
        }),
        Some(CssKnownPropertyValueRef::MarginTop(value)) => {
            value.i01_subset().is_some_and(is_css2_page_length)
        }
        Some(CssKnownPropertyValueRef::MarginRight(value)) => {
            value.i01_subset().is_some_and(is_css2_page_length)
        }
        Some(CssKnownPropertyValueRef::MarginBottom(value)) => {
            value.i01_subset().is_some_and(is_css2_page_length)
        }
        Some(CssKnownPropertyValueRef::MarginLeft(value)) => {
            value.i01_subset().is_some_and(is_css2_page_length)
        }
        _ => false,
    }
}

const fn is_css2_page_length(value: &CssLength) -> bool {
    match value {
        CssLength::Px(_) | CssLength::Percent(_) | CssLength::Zero | CssLength::Auto => true,
        CssLength::Dimension(value) => matches!(
            value.unit(),
            CssLengthUnit::Cm
                | CssLengthUnit::Mm
                | CssLengthUnit::In
                | CssLengthUnit::Pc
                | CssLengthUnit::Pt
        ),
        _ => false,
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

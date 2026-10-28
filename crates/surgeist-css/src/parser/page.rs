use super::block_units::{
    DeclarationUnit, GrammarError, NativeError, drive_units, dropped_unit, grammar_error,
    location_at, native_error, offset_of, refused_declaration,
};
use super::declaration_unit::{DeclarationText, parse_declaration_unit};
use super::property_input::NativeValueRange;
use super::syntax_types::{Attached, Stream, SyntaxError};
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser,
};
use surgeist_css_parser::grammar::{
    AtRule, BlockGrammar, QualifiedRule, RuleEnd, RuleGrammar, UnitKind,
};
use surgeist_css_parser::{Token, match_ignore_ascii_case};

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::{
    DeclarationMode, block_item_diagnostic, is_declaration_recovery_unit, parse_declaration_core,
    top_level_only_at_rule_placement,
};
use crate::component::{CssComponentValueRef, CssValueTokenRef};
use crate::declaration::CssDeclarationBody;
use crate::declaration::syntax::DeclarationContext;
use crate::diagnostic::error::{
    basic, invalid_at_rule_body, invalid_at_rule_body_at, property_name_error, unexpected_at,
    unexpected_syntax_end, unexpected_syntax_token, with_property_context,
};
use crate::diagnostic::{CssError, CssFeatureId};
use crate::property::{CssKnownProperty, CssKnownPropertyValueRef};
use crate::rule::page::{
    CssMarginBox, CssMarginDeclarationBlock, CssMarginRule, CssPageBody, CssPageDeclaration,
    CssPageDeclarationBlock, CssPageDescriptor, CssPageDescriptorKind, CssPageDescriptorValue,
    CssPageMarks, CssPageNamedSize, CssPageOrientation, CssPageOutputOrientation, CssPagePseudo,
    CssPageRule, CssPageSelector, CssPageSelectorList, CssPageSize, CssPageSizeValue,
};
use crate::values::box_model::CssMarginValue;
use crate::values::common::CssBoxSideKind;

pub(super) static IMPLEMENTED_SELECTORS: &[CssFeatureId] = &[
    CssFeatureId::new("official.selector.page-pseudo"),
    CssFeatureId::new("official.selector.logical-page-pseudo"),
];

pub(crate) fn parse_page_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    snapshot: &crate::source::CssSourceSnapshot,
) -> Result<CssPageSelectorList, ParseError<'i, CssError>> {
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
                    crate::values::common::CssIdent::try_new(name.to_string())
                        .expect("decoded parser identifier")
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
            let origin = crate::source::CssParsedOrigin::from_range(
                snapshot,
                start..input.position().byte_index(),
            )
            .expect("genuine Page selector");
            Ok(CssPageSelector::from_parsed(name, pseudos, origin))
        })?
    };
    let origin =
        crate::source::CssParsedOrigin::from_range(snapshot, start..input.position().byte_index())
            .expect("genuine Page selector list");
    Ok(CssPageSelectorList::from_parsed(selectors, origin))
}

pub(crate) fn parse_page_rule<'i, 't>(
    source: &'i str,
    selector: CssPageSelectorList,
    input: &mut Parser<'i, 't>,
    start: &ParserState,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssPageRule, ParseError<'i, CssError>> {
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
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> CssPageBody {
    let mut declarations = Vec::new();
    let mut margin_rules = Vec::new();
    let mut parser = PageBodyParser {
        source,
        recovery,
        diagnostics: Vec::new(),
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
                    crate::diagnostic::CssRecoveryAction::DropDeclaration
                } else {
                    crate::diagnostic::CssRecoveryAction::DropAtRule
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
    diagnostics: Vec<crate::diagnostic::CssRecoveryDiagnostic>,
}

/// The declarations of a margin box's block, from the native callback that was
/// handed the block: the contents of the block are the scope.
pub(super) fn parse_margin_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssMarginDeclarationBlock, ParseError<'i, CssError>> {
    let numeric =
        crate::numeric::expression::NumericInputContext::parsed(recovery.source_snapshot());
    let (declarations, mut recovered) = super::property_input::adapt_native(
        input,
        &numeric,
        NativeValueRange::Scope,
        |contents, _| Ok(drive_margin_body(contents, source, &recovery)),
    )?;
    diagnostics.append(&mut recovered);
    Ok(declarations)
}

/// Consumes the declarations of a margin box's block. A unit that is not a
/// declaration is dropped with its recovery diagnostic; the valid declarations
/// around it survive.
pub(super) fn drive_margin_body<'i>(
    contents: &mut Stream<'i>,
    source: &str,
    recovery: &RecoveryState,
) -> (
    CssMarginDeclarationBlock,
    Vec<crate::diagnostic::CssRecoveryDiagnostic>,
) {
    let snapshot = recovery.source_snapshot();
    let mut grammar = MarginBodyParser {
        source,
        recovery: recovery.clone(),
    };
    let mut readmit = MarginBodyParser {
        source,
        recovery: recovery.clone(),
    };
    let mut declarations = Vec::new();
    let mut diagnostics = Vec::new();
    drive_units(contents, &mut grammar, |unit| {
        let failed = match unit {
            Ok(declaration) => {
                declarations.push(declaration);
                return None;
            }
            Err(failed) => failed,
        };
        let mut dropped = dropped_unit(snapshot, source, &failed);
        if let Some((start, end)) = refused_declaration(&failed) {
            // A declaration that step 8 refuses is read to its `;` all the
            // same, and the grammar's verdict on it is reported (until the
            // declaration paths follow step 8).
            let text = DeclarationText::read(start, end);
            let name = match start.token() {
                Some(surgeist_css_parser::Token::Ident(name)) => name,
                _ => "",
            };
            match readmit.admit(name, &text) {
                Ok(declaration) => {
                    declarations.push(declaration);
                    return Some(end);
                }
                Err(error) => dropped.error = error,
            }
        }
        // The dropped unit of a failed rule ends after its block's opening
        // token; the block is skipped with the unit. An at-rule whose prelude
        // was accepted had its block entered and is dropped whole; one whose
        // prelude was rejected (a stream failure, see `MarginBodyParser::at_rule`)
        // ends before the `;` or `{` that ended it.
        let (text, end) = match dropped.kind {
            UnitKind::AtRule if failed.error().is_stream_error() => {
                (dropped.text_to_prelude_end(), dropped.text_to_block().1)
            }
            UnitKind::AtRule => (dropped.text(), dropped.end),
            _ => dropped.text_to_block(),
        };
        let action = if is_declaration_recovery_unit(text) {
            crate::diagnostic::CssRecoveryAction::DropDeclaration
        } else {
            crate::diagnostic::CssRecoveryAction::DropAtRule
        };
        if let Some(diagnostic) = block_item_diagnostic(source, dropped.error, text, end, action) {
            diagnostics.push(diagnostic);
        }
        None
    });
    let body = CssPageDeclarationBlock::from_parsed(declarations);
    (
        CssMarginDeclarationBlock::from_parsed(body.properties().clone()),
        diagnostics,
    )
}

/// The grammar of the block of a margin box: declarations of the properties
/// that apply to a margin box, and nothing else. A margin box admits no
/// descriptor and no rule (Paged Media 3, Appendix A).
struct MarginBodyParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
}

impl<'i> RuleGrammar<'i, Attached<'i>> for MarginBodyParser<'_> {
    type Item = CssPageDeclaration;
    type Error = NativeError<'i>;

    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let name = rule.name();
        let location = location_at(snapshot, offset_of(rule.prelude().start_site()));
        let fail = |error| grammar_error(rule.start(), error);
        // The prelude of a margin box's name has to be empty; any other is
        // read to its end. A rejected prelude is reported as the stream
        // failure itself, which tells it from a rejected block: the block of
        // the first was never entered, and the unit ends before it.
        if CssMarginBox::from_name(name).is_some() {
            rule.prelude().require_empty()?;
        }
        let Some(block) = rule.block() else {
            return Err(fail(unexpected_syntax_token(
                location_at(snapshot, offset_of(rule.end())),
                Token::Semicolon,
            )));
        };
        if name.eq_ignore_ascii_case("counter-style") || name.eq_ignore_ascii_case("page") {
            return Err(fail(top_level_only_at_rule_placement(location, name)));
        }
        Err(fail(page_margin_body_error(
            snapshot,
            offset_of(block.start_site()),
        )))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        Err(grammar_error(
            rule.start(),
            page_margin_body_error(snapshot, offset_of(rule.start())),
        ))
    }
}

impl<'i> BlockGrammar<'i, Attached<'i>> for MarginBodyParser<'_> {
    fn declaration(
        &mut self,
        declaration: DeclarationUnit<'i>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = declaration.start();
        self.admit(declaration.name(), &DeclarationText::of(&declaration))
            .map_err(|error| grammar_error(at, error))
    }
}

impl MarginBodyParser<'_> {
    /// Admits the declaration named `name` that lies at `text`.
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssPageDeclaration, NativeError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let implicit_closures = self.recovery.check_component_values_from(
            self.source,
            text.value_start,
            "css.declaration",
        )?;
        let declaration_start = location_at(snapshot, text.name_start());
        if CssPageDescriptorKind::from_name(name).is_some() {
            return Err(property_name_error(declaration_start, name));
        }
        let property = page_property_name(name, declaration_start)?;
        if property.is_some_and(|property| !is_page_margin_property(property)) {
            return Err(with_property_context(
                unexpected_at(location_at(snapshot, text.value_start)),
                name,
            ));
        }
        let parsed = parse_declaration_unit(
            DeclarationMode::Ordinary,
            name,
            text,
            snapshot,
            self.recovery.parser_context(),
        )?;
        if let Some(violation) = page_declaration_violation(&parsed.body, &parsed.components) {
            return Err(page_violation_error(
                &violation,
                &parsed.components,
                name,
                location_at(snapshot, text.end),
            ));
        }
        self.recovery.retain_component_closures(implicit_closures);
        Ok(CssPageDeclaration::Property(
            parsed.into_declaration_in_context(DeclarationContext::Page),
        ))
    }
}

/// The error of a rule in a margin box's block, whose contents are declarations
/// of the properties that apply to the box.
fn page_margin_body_error<'i>(
    snapshot: &crate::source::CssSourceSnapshot,
    offset: usize,
) -> NativeError<'i> {
    invalid_at_rule_body_at(
        snapshot
            .position_at(offset)
            .expect("a driver boundary of the source is a character boundary"),
        location_at(snapshot, offset),
        "page",
        "later.rule.page",
        "page-context margin declarations",
    )
}

/// The error of a declaration that applies to a margin box but breaks one of
/// the additional Page restrictions, at `location` when the violation names no
/// component.
fn page_violation_error<'i>(
    violation: &PageDeclarationViolation<'_>,
    components: &crate::component::CssComponentValues,
    name: &str,
    location: cssparser::SourceLocation,
) -> NativeError<'i> {
    let component = violation
        .component()
        .or_else(|| first_page_component(components));
    let error = component.map_or_else(
        || unexpected_at(location),
        |component| {
            crate::diagnostic::error::unexpected_component_value(
                component,
                "an admitted Page margin value",
            )
        },
    );
    with_property_context(error, name)
}

/// The detached boundary admits only the Page margin branch: an at-rule named
/// for a margin box, with the block of that box. Other Page-body items never
/// become a detached margin occurrence.
struct DetachedMarginParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
    /// What the recovery of the box's block reported.
    diagnostics: Vec<crate::diagnostic::CssRecoveryDiagnostic>,
}

impl<'i> RuleGrammar<'i, Attached<'i>> for DetachedMarginParser<'_> {
    type Item = CssMarginRule;
    type Error = NativeError<'i>;

    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        let at = rule.start();
        self.admit(&rule).map_err(|error| grammar_error(at, error))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        // A qualified rule is no margin box; the failed rule's block is
        // checked for the resources entering it would be charged.
        let resource = self.recovery.check_failed_rule_block_at(
            self.source,
            offset_of(rule.block().start_site()),
            "css.qualified-rule",
        );
        Err(match resource {
            Some(error) => grammar_error(rule.start(), error),
            None => SyntaxError::invalid_qualified_rule_error(rule.start()),
        })
    }
}

impl DetachedMarginParser<'_> {
    fn admit<'i>(
        &mut self,
        rule: &AtRule<'i, Attached<'i>>,
    ) -> Result<CssMarginRule, NativeError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let name = rule.name();
        let location = location_at(snapshot, offset_of(rule.prelude().start_site()));
        let prelude = match CssMarginBox::from_name(name) {
            Some(margin) => rule
                .prelude()
                .require_empty()
                .map(|()| margin)
                .map_err(|error| native_error(snapshot, &SyntaxError::from(error))),
            None if name.eq_ignore_ascii_case("counter-style")
                || name.eq_ignore_ascii_case("page") =>
            {
                Err(top_level_only_at_rule_placement(location, name))
            }
            None => Err(cssparser::ParseError {
                kind: cssparser::ParseErrorKind::Basic(
                    cssparser::BasicParseErrorKind::AtRuleInvalid(cssparser::CowRcStr::from(name)),
                ),
                location,
            }),
        };
        let margin = match prelude {
            Ok(margin) => margin,
            Err(error) => {
                // A failed prelude of a rule with a block: the block is
                // checked for the resources entering it would be charged.
                let resource = rule.block().and_then(|block| {
                    self.recovery.check_failed_rule_block_at(
                        self.source,
                        offset_of(block.start_site()),
                        "css.at-rule",
                    )
                });
                return Err(resource.unwrap_or(error));
            }
        };
        let Some(mut block) = rule.block() else {
            // A margin box has to have its block; the rule driver of a
            // detached rule, unlike a block's, tells a `;` from the end.
            let location = location_at(snapshot, offset_of(rule.end()));
            return Err(match rule.rule_end() {
                RuleEnd::Semicolon => unexpected_syntax_token(location, Token::Semicolon),
                _ => unexpected_syntax_end(location),
            });
        };
        let opening = offset_of(block.start_site()).saturating_sub(1);
        let mut depth =
            self.recovery
                .enter_rule_block_at(self.source, opening, "later.rule.page")?;
        let (declarations, mut recovered) =
            drive_margin_body(&mut block, self.source, &self.recovery);
        self.diagnostics.append(&mut recovered);
        depth.retain();
        Ok(CssMarginRule::from_parsed(
            margin,
            declarations,
            crate::source::CssSourcePosition::from_byte_offset_in(
                snapshot.as_str(),
                offset_of(rule.start()),
            ),
        ))
    }
}

pub(crate) fn admit_margin_rule(
    source: &str,
    recovery: RecoveryState,
) -> crate::CssParseReport<Option<CssMarginRule>> {
    let snapshot = recovery.source_snapshot().clone();
    let mut parser = DetachedMarginParser {
        source,
        recovery: recovery.clone(),
        diagnostics: Vec::new(),
    };
    let rejected = |error| {
        vec![super::fragments::reject(
            source,
            error,
            crate::diagnostic::CssRecoveryAction::RejectInput,
        )]
    };
    let (margin, mut diagnostics) = match recovery.syntax_document(source) {
        Ok(document) => match document.root().parse_rule(&mut parser) {
            Ok(margin) => (Some(margin), std::mem::take(&mut parser.diagnostics)),
            Err(failed) => (None, rejected(native_error(&snapshot, failed.error()))),
        },
        Err(error) => (
            None,
            rejected(crate::diagnostic::error::invalid_component_value(
                location_at(&snapshot, 0),
                error,
            )),
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
    limits: crate::component::CssComponentValueLimits,
    context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssPageDeclarationBlock>> {
    let (declarations, diagnostics) =
        super::declaration_block::parse_contents(source, limits, context, |recovery| {
            PageBodyParser {
                source,
                recovery,
                diagnostics: Vec::new(),
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
    ) -> Result<(), ParseError<'i, CssError>> {
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
    ) -> Result<CssPageDeclaration, ParseError<'i, CssError>>
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
    fn take_diagnostics(&mut self) -> Vec<crate::diagnostic::CssRecoveryDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}

fn page_property_name<'i>(
    name: &str,
    location: cssparser::SourceLocation,
) -> Result<Option<CssKnownProperty>, ParseError<'i, CssError>> {
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
    type Error = CssError;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let location = input.current_source_location();
        if let Some(margin) = CssMarginBox::from_name(name.as_ref()) {
            input.expect_exhausted().map_err(basic)?;
            return Ok(PageBodyAtRulePrelude::MarginBox(margin));
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
                )?;
                depth.retain();
                Ok(PageBodyItem::Margin(CssMarginRule::from_parsed(
                    name,
                    declarations,
                    crate::source::CssSourcePosition::from_cssparser(
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
    type Error = CssError;

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

impl<'i> RuleBodyItemParser<'i, PageBodyItem, CssError> for PageBodyParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

impl<'i, 's: 'i> DeclarationParser<'i> for PageBodyParser<'s> {
    type Declaration = PageBodyItem;
    type Error = CssError;

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
            let value_start = input.state();
            input.reset(declaration_start);
            input.expect_ident().map_err(basic)?;
            let name_origin = crate::source::CssParsedOrigin::from_range(
                self.recovery.source_snapshot(),
                declaration_start.position().byte_index()..input.position().byte_index(),
            )
            .expect("real descriptor name");
            input.reset(&value_start);
            // Page descriptors admit importance; a malformed annotation
            // reports its own first unexpected token.
            let (value, importance) = super::parse_importance_boundary(
                input,
                true,
                |_, malformed| basic(malformed.expect("an admitted annotation fails on a token")),
                |input| {
                    super::collect_declaration_value(
                        input,
                        self.recovery.source_snapshot(),
                        |input| {
                            parse_descriptor_value(input, kind, self.recovery.source_snapshot())
                        },
                    )
                },
            )
            .map(|((data, components, origin), importance)| {
                (
                    CssPageDescriptorValue::from_parsed(kind, data, components, origin),
                    importance,
                )
            })
            .map_err(|error| {
                crate::diagnostic::error::with_descriptor_context(error, "page", kind.as_str())
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
            return Err(page_violation_error(
                &violation,
                &parsed.components,
                name.as_ref(),
                input.current_source_location(),
            ));
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
    NonzeroUnitlessLength(&'a crate::component::CssComponentValue),
}

pub(crate) fn first_page_component(
    values: &crate::component::CssComponentValues,
) -> Option<&crate::component::CssComponentValue> {
    values.items().iter().find(|value| {
        !matches!(
            value.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                | CssComponentValueRef::Comment(_)
        )
    })
}

impl<'a> PageDeclarationViolation<'a> {
    pub(crate) fn component(&self) -> Option<&'a crate::component::CssComponentValue> {
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
    _components: &'a crate::component::CssComponentValues,
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
                .assigned()
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

fn nonzero_unitless_page_length(
    value: &CssMarginValue,
) -> Option<&crate::component::CssComponentValue> {
    let CssMarginValue::LengthPercentage(value) = value else {
        return None;
    };
    let component = value.literal_component()?;
    if matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number))
        if !crate::serialization::exact_decimal::LexicalDecimal::new(number.representation()).is_zero())
    {
        Some(component)
    } else {
        None
    }
}

pub(super) fn parse_descriptor_value<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssPageDescriptorKind,
    snapshot: &crate::source::CssSourceSnapshot,
) -> Result<crate::rule::page::syntax::PageValueData, ParseError<'i, CssError>> {
    use crate::rule::page::syntax::PageValueData as V;
    let start = input.state();
    let components = crate::component::CssComponentValues::collect_from_parser(input, snapshot)
        .map_err(|error| {
            crate::diagnostic::error::invalid_component_value(
                input.current_source_location(),
                error,
            )
        })?;
    input.reset(&start);
    let numeric = crate::numeric::expression::NumericInputContext::parsed(snapshot);
    if super::variables::descriptor_substitution_qualifies(components.items(), &numeric).map_err(
        |error| {
            crate::diagnostic::error::invalid_component_value(
                input.current_source_location(),
                error,
            )
        },
    )? {
        let start = input.position();
        super::descriptor_values::consume_remaining_components(input)?;
        return Ok(V::Pending(
            crate::declaration::CssSubstitutionDependentValue::new(
                crate::declaration::CssAuthoredDeclarationValue::new(input.slice_from(start)),
            ),
        ));
    }
    if let Ok(global) = input.try_parse(|input| {
        let keyword_location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        let global = crate::validation::parse_global_keyword(&ident)
            .ok_or_else(|| unexpected_at(keyword_location))?;
        input.expect_exhausted().map_err(basic)?;
        Ok::<_, ParseError<'i, CssError>>(global)
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
        CssPageDescriptorKind::Bleed => V::Bleed(super::property_input::adapt_native(
            input,
            &numeric,
            NativeValueRange::Component,
            |input, numeric| {
                super::side_shapes::parse_auto_or(input, |input| {
                    super::values::parse_length(input, numeric)
                })
            },
        )?),
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(value)
}
fn parse_size<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssPageSizeValue, ParseError<'i, CssError>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssPageSizeValue::Auto);
    }
    if let Ok(first) = input.try_parse(|input| {
        super::property_input::adapt_native(
            input,
            numeric,
            NativeValueRange::Component,
            super::values::parse_nonnegative_length,
        )
    }) {
        let second = if input.is_exhausted() {
            None
        } else {
            Some(super::property_input::adapt_native(
                input,
                numeric,
                NativeValueRange::Component,
                super::values::parse_nonnegative_length,
            )?)
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

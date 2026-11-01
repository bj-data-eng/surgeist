use super::block_units::{
    DeclarationUnit, GrammarError, NativeError, drive_units, dropped_unit, grammar_error,
    location_at, native_error, offset_of,
};
use super::declaration_unit::{DeclarationText, parse_declaration_unit};
use super::descriptor_stream::{
    Value, ValueError, byte_offset, collect_scope, consume_remaining, drive_block_items,
    expect_exhausted, fail, from_component, from_here, location, native, parse_scope,
};
use super::property_input::StreamExt;
use super::syntax_types::{Attached, Site, Stream, SyntaxError};
use cssparser::{ParseError, Parser};
use surgeist_css_parser::grammar::{
    AtRule, BlockGrammar, QualifiedRule, RuleEnd, RuleGrammar, UnitKind,
};
use surgeist_css_parser::{Offset, Token, match_ignore_ascii_case};

use super::recovery::RecoveryState;
use super::{
    DeclarationMode, block_item_diagnostic_from_start, is_declaration_recovery_unit,
    top_level_only_at_rule_placement,
};
use crate::component::{CssComponentValueRef, CssValueTokenRef};
use crate::declaration::CssDeclarationBody;
use crate::declaration::syntax::DeclarationContext;
use crate::diagnostic::error::{
    invalid_at_rule_body_at, property_name_error, unexpected_at, unexpected_syntax_end,
    unexpected_syntax_token, with_property_context,
};
use crate::diagnostic::{CssError, CssFeatureId};
use crate::numeric::expression::NumericInputContext;
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

/// The selector list of a `@page` rule that the native parser `input` holds to
/// its end.
pub(crate) fn parse_page_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    snapshot: &crate::source::CssSourceSnapshot,
) -> Result<CssPageSelectorList, ParseError<'i, CssError>> {
    super::descriptor_stream::parse_scope(input, snapshot, |stream, numeric| {
        page_selector_list(stream, numeric, snapshot)
    })
}

/// The selector list of a `@page` rule that the stream `input` holds.
pub(crate) fn read_page_selector<'i>(
    input: &mut Stream<'i>,
    snapshot: &crate::source::CssSourceSnapshot,
) -> Result<CssPageSelectorList, NativeError<'i>> {
    super::descriptor_stream::read_scope(input, snapshot, |stream, numeric| {
        page_selector_list(stream, numeric, snapshot)
    })
}

fn page_selector_list<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    snapshot: &crate::source::CssSourceSnapshot,
) -> Value<'i, CssPageSelectorList> {
    let start = byte_offset(numeric, input.site());
    let selectors = if input.only_whitespace_remains() {
        Vec::new()
    } else {
        input.comma_segments(|input| page_selector(input, numeric, snapshot))?
    };
    let origin = crate::source::CssParsedOrigin::from_range(
        snapshot,
        start..byte_offset(numeric, input.site()),
    )
    .expect("genuine Page selector list");
    Ok(CssPageSelectorList::from_parsed(selectors, origin))
}

fn page_selector<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    snapshot: &crate::source::CssSourceSnapshot,
) -> Value<'i, CssPageSelector> {
    let unexpected = |at: Site<'i>| SyntaxError::custom(at, unexpected_at(location(numeric, at)));
    // The end of the input is after the comments that end it.
    let next = |input: &mut Stream<'i>| {
        input.consume_token().map_err(|error| {
            if error.is_end_of_stream() {
                input.end_site().end_of_stream()
            } else {
                error.into()
            }
        })
    };
    input.discard_whitespace();
    let start = byte_offset(numeric, input.site());
    let name = input.attempt(Stream::consume_ident).ok().map(|name| {
        crate::values::common::CssIdent::try_new(name.to_string())
            .expect("decoded parser identifier")
    });
    let mut pseudos = Vec::new();
    while !input.only_whitespace_remains() {
        let at = input.site();
        match next(input)? {
            Token::Whitespace => {
                input.discard_whitespace();
                expect_exhausted(input)?;
                break;
            }
            Token::Colon => {}
            _ => return Err(unexpected(at)),
        }
        let at = input.site();
        let Token::Ident(name) = next(input)? else {
            return Err(unexpected(at));
        };
        let pseudo = match_ignore_ascii_case! { name,
            "left" => CssPagePseudo::Left, "right" => CssPagePseudo::Right,
            "first" => CssPagePseudo::First, "blank" => CssPagePseudo::Blank,
            "recto" => CssPagePseudo::Recto, "verso" => CssPagePseudo::Verso,
            _ => return Err(unexpected(at)),
        };
        pseudos.push(pseudo);
    }
    if name.is_none() && pseudos.is_empty() {
        return Err(unexpected(input.site()));
    }
    let origin = crate::source::CssParsedOrigin::from_range(
        snapshot,
        start..byte_offset(numeric, input.site()),
    )
    .expect("genuine Page selector");
    Ok(CssPageSelector::from_parsed(name, pseudos, origin))
}

pub(crate) fn parse_page_rule<'i>(
    source: &str,
    selector: CssPageSelectorList,
    contents: &mut Stream<'i>,
    position: crate::source::CssSourcePosition,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> CssPageRule {
    let declarations = parse_body(source, contents, diagnostics, recovery);
    CssPageRule::new(selector, declarations, position)
}

/// The declarations and margin rules of the block `contents` holds.
pub(super) fn parse_body<'i>(
    source: &str,
    contents: &mut Stream<'i>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> CssPageBody {
    let mut grammar = PageBodyParser {
        source,
        recovery: recovery.clone(),
        diagnostics: Vec::new(),
    };
    let (items, mut dropped) = drive_block_items(
        contents,
        &mut grammar,
        source,
        &recovery,
        crate::diagnostic::CssRecoveryAction::DropDeclaration,
    );
    diagnostics.append(&mut dropped);
    diagnostics.append(&mut grammar.diagnostics);
    let mut declarations = Vec::new();
    let mut margin_rules = Vec::new();
    for item in items {
        match item {
            PageBodyItem::Declaration(declaration) => declarations.push(declaration),
            PageBodyItem::Margin(rule) => margin_rules.push(rule),
        }
    }
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
    let mut declarations = Vec::new();
    let mut diagnostics = Vec::new();
    let refused = drive_units(contents, &mut grammar, |unit| {
        let failed = match unit {
            Ok(declaration) => {
                declarations.push(declaration);
                return;
            }
            Err(failed) => failed,
        };
        let dropped = dropped_unit(snapshot, source, &failed);
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
        // The span starts where the dropped unit does, whatever part of it the
        // text names: a rejected prelude names less than the unit, which runs
        // to the end of its block.
        if let Some(diagnostic) =
            block_item_diagnostic_from_start(source, dropped.error, dropped.start, end, action)
        {
            diagnostics.push(diagnostic);
        }
    });
    if refused
        && let Some(diagnostic) = crate::parser::block_units::refusal_diagnostic(
            snapshot,
            contents,
            crate::diagnostic::CssRecoveryAction::DropDeclaration,
        )
    {
        diagnostics.push(diagnostic);
    }
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
                .enter_rule_block(self.source, Offset::new(opening), "later.rule.page")?;
        let (declarations, mut recovered) =
            drive_margin_body(&mut block, self.source, &self.recovery);
        self.diagnostics.append(&mut recovered);
        depth.retain();
        Ok(CssMarginRule::from_parsed(
            margin,
            declarations,
            self.recovery.source_position(offset_of(rule.start())),
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
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
        _recovery: &RecoveryState,
    ) -> Result<CssPageDeclaration, ParseError<'i, CssError>> {
        PageBodyParser::admit(self, name, text).map(|item| match item {
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

impl<'i> RuleGrammar<'i, Attached<'i>> for PageBodyParser<'_> {
    type Item = PageBodyItem;
    type Error = NativeError<'i>;

    // A margin box is the one at-rule a page body admits.
    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let name = rule.name();
        let location = location_at(snapshot, offset_of(rule.prelude().start_site()));
        let fail = |error| grammar_error(rule.start(), error);
        // The prelude of a margin box's name has to be empty; any other is
        // read to its end. A rejected prelude is reported as the stream
        // failure itself, which tells it from a rejected block: the block of
        // the first was never entered, and the unit ends before it.
        let margin = CssMarginBox::from_name(name);
        if margin.is_some() {
            rule.prelude().require_empty()?;
        }
        let Some(mut block) = rule.block() else {
            return Err(fail(unexpected_syntax_token(
                location_at(snapshot, offset_of(rule.end())),
                Token::Semicolon,
            )));
        };
        let Some(margin) = margin else {
            if name.eq_ignore_ascii_case("counter-style") || name.eq_ignore_ascii_case("page") {
                return Err(fail(top_level_only_at_rule_placement(location, name)));
            }
            return Err(fail(page_margin_body_error(
                snapshot,
                offset_of(block.start_site()),
            )));
        };
        let opening = offset_of(block.start_site()).saturating_sub(1);
        let mut depth = self
            .recovery
            .enter_rule_block(self.source, Offset::new(opening), "later.rule.page")
            .map_err(fail)?;
        let (declarations, mut recovered) =
            drive_margin_body(&mut block, self.source, &self.recovery);
        self.diagnostics.append(&mut recovered);
        depth.retain();
        Ok(PageBodyItem::Margin(CssMarginRule::from_parsed(
            margin,
            declarations,
            self.recovery.source_position(offset_of(rule.start())),
        )))
    }

    // A page body admits no qualified rule.
    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        Err(grammar_error(
            rule.start(),
            page_margin_body_error(snapshot, offset_of(rule.prelude().start_site())),
        ))
    }
}

impl<'i> BlockGrammar<'i, Attached<'i>> for PageBodyParser<'_> {
    fn declaration(
        &mut self,
        declaration: DeclarationUnit<'i>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = declaration.start();
        self.admit(declaration.name(), &DeclarationText::of(&declaration))
            .map_err(|error| grammar_error(at, error))
    }
}

impl PageBodyParser<'_> {
    /// Admits the declaration named `name` that lies at `text`.
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<PageBodyItem, NativeError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let implicit_closures = self.recovery.check_component_values_from(
            self.source,
            text.value_start,
            "css.declaration",
        )?;
        let declaration_start = location_at(snapshot, text.name_start());
        if let Some(kind) = CssPageDescriptorKind::from_name(name) {
            let name_origin =
                crate::source::CssParsedOrigin::from_range(snapshot, text.name_range())
                    .expect("real descriptor name");
            // Page descriptors admit importance.
            let (value, importance) = descriptor_value(snapshot, text, kind).map_err(|error| {
                crate::diagnostic::error::with_descriptor_context(error, "page", kind.as_str())
            })?;
            self.recovery.retain_component_closures(implicit_closures);
            return Ok(PageBodyItem::Declaration(CssPageDeclaration::Descriptor(
                CssPageDescriptor::from_parsed(value, importance, name_origin),
            )));
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
        Ok(PageBodyItem::Declaration(CssPageDeclaration::Property(
            parsed.into_declaration_in_context(DeclarationContext::Page),
        )))
    }
}

/// The value of the page descriptor `kind` that lies at `text`, and its
/// importance.
fn descriptor_value<'i>(
    snapshot: &crate::source::CssSourceSnapshot,
    text: &DeclarationText,
    kind: CssPageDescriptorKind,
) -> Result<(CssPageDescriptorValue, crate::declaration::CssImportance), NativeError<'i>> {
    let numeric = NumericInputContext::parsed(snapshot);
    let range = text.value_start..text.value_end();
    let failed_at = |error| {
        crate::diagnostic::error::invalid_component_value(location_at(snapshot, range.end), error)
    };
    let document = crate::syntax_consumption::source_window_document(snapshot, range.clone())
        .map_err(failed_at)?;
    let data = read_descriptor_value(&mut document.root(), kind, &numeric)
        .map_err(|error| native(&numeric, &error))?;
    let components =
        crate::component::values::parse::promote_window(&document).map_err(failed_at)?;
    let origin = crate::source::CssParsedOrigin::from_range(snapshot, range.clone())
        .expect("parser value boundaries belong to the original source");
    let importance = if text.bang().is_some() {
        crate::declaration::CssImportance::Important
    } else {
        crate::declaration::CssImportance::Normal
    };
    Ok((
        CssPageDescriptorValue::from_parsed(kind, data, components, origin),
        importance,
    ))
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

/// The value of the page descriptor `kind` that the native parser `input` holds,
/// to the end of its scope.
pub(super) fn parse_descriptor_value<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssPageDescriptorKind,
    snapshot: &crate::source::CssSourceSnapshot,
) -> Result<crate::rule::page::syntax::PageValueData, ParseError<'i, CssError>> {
    parse_scope(input, snapshot, |value, numeric| {
        read_descriptor_value(value, kind, numeric)
    })
}

fn read_descriptor_value<'i>(
    input: &mut Stream<'i>,
    kind: CssPageDescriptorKind,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, crate::rule::page::syntax::PageValueData> {
    use crate::rule::page::syntax::PageValueData as V;
    let start = input.mark();
    let first = input.site();
    let components = collect_scope(input, numeric).map_err(|(error, at)| {
        SyntaxError::custom(
            first,
            crate::diagnostic::error::invalid_component_value(at, error),
        )
    })?;
    input.restore(start);
    if super::variables::descriptor_substitution_qualifies(components.items(), numeric).map_err(
        |error| {
            fail(numeric, first, |at| {
                crate::diagnostic::error::invalid_component_value(at, error)
            })
        },
    )? {
        consume_remaining(input);
        return Ok(V::Pending(
            crate::declaration::CssSubstitutionDependentValue::new(
                crate::declaration::CssAuthoredDeclarationValue::new(
                    input.authored_text_since(first),
                ),
            ),
        ));
    }
    if let Ok(global) = input.attempt(|input| {
        let keyword_location = input.site();
        let ident = from_here(input, Stream::consume_ident)?;
        let global = crate::validation::parse_global_keyword(ident)
            .ok_or_else(|| fail(numeric, keyword_location, unexpected_at))?;
        expect_exhausted(input)?;
        Ok::<_, ValueError<'i>>(global)
    }) {
        return Ok(V::Global(global));
    }
    let value = match kind {
        CssPageDescriptorKind::Size => V::Size(parse_size(input, numeric)?),
        CssPageDescriptorKind::PageOrientation => {
            let location = input.site();
            let name = from_here(input, Stream::consume_ident)?;
            V::PageOrientation(
                match_ignore_ascii_case! {name,"upright"=>CssPageOutputOrientation::Upright,"rotate-left"=>CssPageOutputOrientation::RotateLeft,"rotate-right"=>CssPageOutputOrientation::RotateRight,_=>return Err(fail(numeric, location, unexpected_at))},
            )
        }
        CssPageDescriptorKind::Marks => {
            let location = input.site();
            let name = from_here(input, Stream::consume_ident)?;
            let (crop, cross) = if name.eq_ignore_ascii_case("none") {
                (false, false)
            } else {
                let mut crop = false;
                let mut cross = false;
                for name in std::iter::once(name).chain(input.attempt(Stream::consume_ident).ok()) {
                    match_ignore_ascii_case! {name,"crop"=>{if crop{return Err(fail(numeric, location, unexpected_at));}crop=true;},"cross"=>{if cross{return Err(fail(numeric, location, unexpected_at));}cross=true;},_=>return Err(fail(numeric, location, unexpected_at))}
                }
                (crop, cross)
            };
            V::Marks(CssPageMarks::new(crop, cross))
        }
        CssPageDescriptorKind::Bleed => V::Bleed(from_component(numeric, input, |input| {
            super::side_shapes::parse_auto_or(input, |input| {
                super::values::parse_length(input, numeric)
            })
        })?),
    };
    expect_exhausted(input)?;
    Ok(value)
}

fn parse_size<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssPageSizeValue> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssPageSizeValue::Auto);
    }
    if let Ok(first) = input.attempt(|input| {
        from_component(numeric, input, |input| {
            super::values::parse_nonnegative_length(input, numeric)
        })
    }) {
        let second = if input.only_whitespace_remains() {
            None
        } else {
            Some(from_component(numeric, input, |input| {
                super::values::parse_nonnegative_length(input, numeric)
            })?)
        };
        return Ok(CssPageSizeValue::Dimensions(first, second));
    }
    let mut size = None;
    let mut orientation = None;
    for _ in 0..2 {
        if input.only_whitespace_remains() {
            break;
        }
        let location = input.site();
        let name = from_here(input, Stream::consume_ident)?;
        if let Some(value) = match_ignore_ascii_case! {name,"portrait"=>Some(CssPageOrientation::Portrait),"landscape"=>Some(CssPageOrientation::Landscape),_=>None}
        {
            if orientation.replace(value).is_some() {
                return Err(fail(numeric, location, unexpected_at));
            }
        } else {
            let value = match_ignore_ascii_case! {name,"a5"=>CssPageSize::A5,"a4"=>CssPageSize::A4,"a3"=>CssPageSize::A3,"b5"=>CssPageSize::B5,"b4"=>CssPageSize::B4,"jis-b5"=>CssPageSize::JisB5,"jis-b4"=>CssPageSize::JisB4,"letter"=>CssPageSize::Letter,"legal"=>CssPageSize::Legal,"ledger"=>CssPageSize::Ledger,_=>return Err(fail(numeric, location, unexpected_at))};
            if size.replace(value).is_some() {
                return Err(fail(numeric, location, unexpected_at));
            }
        }
    }
    if size.is_none() && orientation.is_none() {
        return Err(fail(numeric, input.site(), unexpected_at));
    }
    Ok(CssPageSizeValue::Named(
        CssPageNamedSize::try_new(size, orientation).expect("parsed nonempty Page size"),
    ))
}

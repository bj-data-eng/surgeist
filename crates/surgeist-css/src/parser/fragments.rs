//! Context-specific parsing over the caller's unmodified source.
use super::*;

pub(super) fn bounded<T: Send>(
    source: &str,
    parse: impl FnOnce() -> crate::CssParseReport<T> + Send,
) -> crate::CssParseReport<T> {
    recovery::finish_report(source, bounded_execution(source, parse))
}

pub(super) fn bounded_execution<T: Send>(source: &str, parse: impl FnOnce() -> T + Send) -> T {
    // Recursive value and selector grammar can have larger frames than
    // structural rule parsing. This threshold is not an admission limit.
    if recovery::maximum_nested_depth(source) < 64 {
        return parse();
    }
    std::thread::scope(|scope| {
        let thread = std::thread::Builder::new()
            .name("surgeist-css-fragment-parser".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, parse)
            .expect("bounded CSS parser thread must be available");
        match thread.join() {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

pub(super) fn reject(
    source: &str,
    error: ParseError<'_, Error>,
    action: crate::CssRecoveryAction,
) -> crate::CssRecoveryDiagnostic {
    let action = recovery_action_for_error(&error, action);
    reject_resolved(source, from_parse_error(source, error), action)
}

fn reject_resolved(
    source: &str,
    error: Error,
    action: crate::CssRecoveryAction,
) -> crate::CssRecoveryDiagnostic {
    let span = crate::CssSourceSpan::new(
        crate::CssSourcePosition::from_byte_offset_in(source, 0),
        crate::CssSourcePosition::from_byte_offset_in(source, source.len()),
    )
    .expect("complete source span");
    crate::CssRecoveryDiagnostic::new(error, span, action)
        .expect("fragment errors originate within the complete source")
}

pub(super) fn finish_nested_component<'i>(
    input: &mut Parser<'i, '_>,
    token: &Token<'i>,
) -> Result<(), ParseError<'i, Error>> {
    if matches!(
        token,
        Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock
    ) {
        // Finish this block now so the next root token's location and source
        // slice begin after it, rather than before implicit tokenizer skipping.
        input.parse_nested_block(|nested| {
            while nested.next_including_whitespace_and_comments().is_ok() {}
            Ok(())
        })?;
    }
    Ok(())
}

fn real_brace_block<T: Send>(
    source: &str,
    production: &'static str,
    parser_context: crate::CssParserContext,
    parse_body: impl for<'i, 't> FnOnce(
        &'i str,
        &mut Parser<'i, 't>,
        &mut Vec<crate::CssRecoveryDiagnostic>,
        RecoveryState,
    ) -> Result<T, ParseError<'i, Error>>
    + Send,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<T>>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
            .with_parser_context(parser_context);
        real_brace_block_inner(source, production, state, parse_body)
    })
}

fn real_brace_block_inner<T>(
    source: &str,
    production: &'static str,
    state: RecoveryState,
    parse_body: impl for<'i, 't> FnOnce(
        &'i str,
        &mut Parser<'i, 't>,
        &mut Vec<crate::CssRecoveryDiagnostic>,
        RecoveryState,
    ) -> Result<T, ParseError<'i, Error>>,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<T>>> {
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let mut diagnostics = Vec::new();
    let result = (|| {
        input.skip_whitespace();
        let start = input.position().byte_index();
        input.expect_curly_bracket_block()?;
        let body = input.parse_nested_block(|input| {
            let mut depth = state.enter_rule_block(source, input, production)?;
            // The entered native owner admits each unit before collecting its
            // components. Whole-body component admission would erase recovery
            // partitions and lose valid neighbors of a failed unit.
            let body = parse_body(source, input, &mut diagnostics, state.clone())?;
            depth.retain();
            Ok(body)
        })?;
        let end = input.position().byte_index();
        input.expect_exhausted()?;
        let origin = CssParsedOrigin::from_range(state.source_snapshot(), start..end)
            .expect("consumed block boundaries belong to the original source");
        Ok(crate::CssBlockFragment::from_parsed(body, origin))
    })();
    match result {
        Ok(fragment) => {
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(Some(fragment), diagnostics)
        }
        Err(error) => crate::CssParseReport::new(
            None,
            vec![reject(source, error, crate::CssRecoveryAction::RejectInput)],
        ),
    }
}

/// Parses one genuine curly block as an ordinary inner rule list.
///
/// Supplied namespace bindings apply to actual child selectors. Bare declarations
/// have rule-list recovery; imports and namespace statements have no inner-list
/// permission. No enclosing rule or selector is invented. Empty bodies are valid.
/// Missing/wrong braces or trailing nontrivia reject the whole input. Native unit
/// recovery retains admitted siblings; the actual outer brace counts toward the
/// fixed 256-depth limit. Conditional adjacency is checked on original source
/// after bounded reconstruction. Implicit closures and lexical recovery remain
/// diagnostics and fail clean-report validation. Free parsing uses default context.
#[must_use]
pub fn parse_group_block(
    source: &str,
    namespaces: &CssNamespaceContext,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssRuleList>>> {
    parse_group_block_with_context(source, namespaces, crate::CssParserContext::default())
}

pub(crate) fn parse_group_block_with_context(
    source: &str,
    namespaces: &CssNamespaceContext,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssRuleList>>> {
    bounded(source, || {
        let snapshot = CssSourceSnapshot::new(source);
        let (syntax, mut diagnostics) = parse_bounded(
            source,
            &snapshot,
            0,
            BoundedParseContext::GroupBlock,
            StyleContextCaptures::with_namespaces(namespaces),
            parser_context,
        )
        .into_parts();
        let BoundedParseSyntax::GroupBlock(block) = syntax else {
            unreachable!("ordinary group replay preserves its genuine block carrier")
        };
        let block = block.map(|block| {
            let (mut body, origin) = block.into_parts();
            conditional_chains::completed_list(
                source,
                crate::syntax::CssParserRuleChildrenMut::Ordinary(body.rules_mut()),
                &mut diagnostics,
            );
            crate::CssBlockFragment::from_parsed(body, origin)
        });
        crate::CssParseReport::new(block, diagnostics)
    })
}

pub(super) fn parse_group_block_inner(
    source: &str,
    recovery: RecoveryState,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssRuleList>>> {
    real_brace_block_inner(
        source,
        "css.rule-list",
        recovery,
        |source, input, diagnostics, recovery| {
            let mut parsed = parse_nested_group_rules(source, input, recovery)?;
            diagnostics.append(&mut parsed.diagnostics);
            Ok(crate::CssRuleList::from_parsed(parsed.syntax))
        },
    )
}

/// Parses one genuine scope body with explicit actual style ancestry.
///
/// `Present` admits direct scoped declaration runs; `Absent` does not. Scope-body
/// placement excludes Page rules. Supplied namespaces and default parser context
/// reach the existing scoped provider. Boundaries, original provenance, unit
/// recovery, conditional adjacency, depth and clean validation follow
/// [`parse_group_block`]. No scope root, limit or parent selector is invented.
#[must_use]
pub fn parse_scope_block(
    source: &str,
    namespaces: &CssNamespaceContext,
    ancestry: crate::CssStyleAncestor,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<CssScopedRuleList>>> {
    parse_scope_block_with_context(
        source,
        namespaces,
        ancestry,
        crate::CssParserContext::default(),
    )
}

pub(crate) fn parse_scope_block_with_context(
    source: &str,
    namespaces: &CssNamespaceContext,
    ancestry: crate::CssStyleAncestor,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<CssScopedRuleList>>> {
    scoped_block_with_context(
        source,
        namespaces,
        ancestry,
        ScopedBodyKind::Scope,
        parser_context,
    )
}

/// Parses an ordinary group body within a scope, with actual style ancestry.
///
/// Unlike a scope body, an ordinary scoped group without a style ancestor admits
/// Page rules. With `Present`, direct declarations inherit style permission and
/// global-only rules retain the scoped owner's exclusions. Source boundaries,
/// namespaces, recovery, limits and clean validation follow [`parse_scope_block`].
/// No enclosing group, scope or parent selector is constructed.
#[must_use]
pub fn parse_scoped_group_block(
    source: &str,
    namespaces: &CssNamespaceContext,
    ancestry: crate::CssStyleAncestor,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<CssScopedRuleList>>> {
    parse_scoped_group_block_with_context(
        source,
        namespaces,
        ancestry,
        crate::CssParserContext::default(),
    )
}

pub(crate) fn parse_scoped_group_block_with_context(
    source: &str,
    namespaces: &CssNamespaceContext,
    ancestry: crate::CssStyleAncestor,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<CssScopedRuleList>>> {
    scoped_block_with_context(
        source,
        namespaces,
        ancestry,
        ScopedBodyKind::OrdinaryGroup,
        parser_context,
    )
}

fn scoped_block_with_context(
    source: &str,
    namespaces: &CssNamespaceContext,
    ancestry: crate::CssStyleAncestor,
    body: ScopedBodyKind,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<CssScopedRuleList>>> {
    bounded(source, || {
        let snapshot = CssSourceSnapshot::new(source);
        let has_style_ancestor = matches!(ancestry, crate::CssStyleAncestor::Present);
        let (syntax, mut diagnostics) = parse_bounded(
            source,
            &snapshot,
            0,
            BoundedParseContext::ScopedBlock {
                has_style_ancestor,
                body,
            },
            StyleContextCaptures::with_namespaces(namespaces),
            parser_context,
        )
        .into_parts();
        let BoundedParseSyntax::ScopedBlock(block) = syntax else {
            unreachable!("scoped replay preserves its genuine block carrier")
        };
        let block = block.map(|block| {
            let (mut body, origin) = block.into_parts();
            conditional_chains::completed_list(
                source,
                crate::syntax::CssParserRuleChildrenMut::Scoped(body.rules_mut()),
                &mut diagnostics,
            );
            crate::CssBlockFragment::from_parsed(body, origin)
        });
        crate::CssParseReport::new(block, diagnostics)
    })
}

pub(super) fn parse_scoped_block_inner(
    source: &str,
    recovery: RecoveryState,
    has_style_ancestor: bool,
    body: ScopedBodyKind,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<CssScopedRuleList>>> {
    let production = match body {
        ScopedBodyKind::Scope => "css.scope-block",
        ScopedBodyKind::OrdinaryGroup => "css.scoped-group-block",
    };
    real_brace_block_inner(
        source,
        production,
        recovery,
        |source, input, diagnostics, recovery| {
            let mut parsed =
                parse_scoped_rule_list(source, input, recovery, has_style_ancestor, body)?;
            diagnostics.append(&mut parsed.diagnostics);
            Ok(parsed.syntax)
        },
    )
}

/// Parses a genuine curly block of recovering named-supports test candidates.
///
/// The original-source owner partitions declaration runs, generic qualified
/// tests and generic at-rule tests without inventing a name or prelude. Empty
/// bodies are valid. A statement ending at the real body close is admitted under
/// the selected modern block-contents source; missing braces and lexical closures
/// remain independently diagnosed. Whole-body native resource admission precedes
/// checked promotion and counts the actual brace toward the 256-depth ceiling.
/// Wrong/missing openers and trailing nontrivia reject the whole input. Retained
/// recovery fails clean validation; checked reuse still rejects recovered syntax.
#[must_use]
pub fn parse_supports_test_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssSupportsTestBody>>> {
    real_brace_block(
        source,
        "ext.rule.supports-condition",
        crate::CssParserContext::default(),
        |source, input, diagnostics, recovery| {
            named_supports::parse_body(source, input, diagnostics, &recovery)
                .map(|(body, _, _)| body)
        },
    )
}

/// Parses exactly one genuine curly block of `@font-face` descriptors.
///
/// Outer whitespace/comments are accepted; missing braces and trailing nontrivia
/// reject the complete input. Empty bodies are valid. Descriptor-local failures
/// retain admitted neighbors, while non-declaration structural failures reject
/// the complete body. Occurrence order and effective values use the same owner
/// as a complete rule. No family, source URL or at-keyword is invented.
///
/// The brace counts toward the existing nesting limit. Retained implicit closures
/// and lexical recovery use original-source diagnostics and fail clean validation.
#[must_use]
pub fn parse_font_face_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssFontFaceDescriptors>>> {
    real_brace_block(
        source,
        "baseline.rule.font-face",
        crate::CssParserContext::default(),
        font_face::parse_body,
    )
}

/// Parses exactly one genuine curly block of `@counter-style` descriptors.
///
/// Outer whitespace/comments are accepted; missing braces and trailing nontrivia
/// reject the complete input. Empty bodies retain no fabricated defaults. Native
/// descriptor/child recovery preserves occurrence order and valid neighbors;
/// intrinsic descriptor combinations, including `extends` with symbols, reject
/// the complete body. Counter activation and symbol usability remain downstream.
///
/// The origin covers the actual braces or original implicit EOF and shares the
/// descriptor snapshot. The brace counts toward the existing nesting limit;
/// recovery and implicit closure diagnostics prevent clean validation.
#[must_use]
pub fn parse_counter_style_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssCounterStyleDescriptors>>> {
    real_brace_block(
        source,
        "later.rule.counter-style",
        crate::CssParserContext::default(),
        counter_style::parse_body,
    )
}

/// Parses exactly one genuine curly block of `@font-palette-values` descriptors.
///
/// Outer whitespace/comments are accepted; missing braces and trailing nontrivia
/// reject the complete input. A retained body requires an admitted `font-family`,
/// using the same predicate as complete-rule construction. Native descriptor and
/// child recovery retains valid neighbors and ordered duplicates; missing family
/// rejects the complete body. No palette name or font resource is fabricated.
///
/// Origins and diagnostics use the original source. The brace counts toward the
/// existing nesting limit, including failed child admission. Retained implicit
/// closures and recovery diagnostics prevent clean validation.
#[must_use]
pub fn parse_font_palette_values_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<Vec<crate::CssFontPaletteDescriptor>>>> {
    real_brace_block(
        source,
        "later.rule.font-palette-values",
        crate::CssParserContext::default(),
        |source, input, diagnostics, state| {
            let descriptors = font_palette_values::parse_body(source, input, diagnostics, state);
            crate::font_palette_values::validate_descriptors(&descriptors)
                .map_err(|_| crate::error::invalid_syntax(input.current_source_location()))?;
            Ok(descriptors)
        },
    )
}

/// Parses exactly one genuine curly block of `@color-profile` descriptors.
///
/// Outer whitespace/comments are accepted; missing braces and trailing nontrivia
/// reject the complete input. Empty and incomplete authored bodies are retained;
/// profile completeness and resource loading remain downstream. Native descriptor
/// and child recovery preserves ordered admitted neighbors without a profile name.
///
/// The origin covers actual braces or original implicit EOF and shares descriptor
/// provenance. The brace counts toward the existing nesting limit, including
/// failed children. Recovery and implicit closure diagnostics fail clean validation.
#[must_use]
pub fn parse_color_profile_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<Vec<crate::CssColorProfileDescriptor>>>> {
    real_brace_block(
        source,
        "interop.rule.color-profile",
        crate::CssParserContext::default(),
        |source, input, diagnostics, state| {
            Ok(color_profile::parse_body(source, input, diagnostics, state))
        },
    )
}

/// Parses exactly one genuine curly body of `@font-feature-values`.
///
/// The mixed body retains ordered `font-display` occurrences and the seven
/// defined subsidiary block kinds. Empty bodies are valid; invalid declaration
/// or child units use the same local recovery as the complete rule, retaining
/// admitted neighbors. No font-family prelude or outer at-keyword is invented.
///
/// Optional outer whitespace/comments remain in the original snapshot, outside
/// the block origin. Missing braces or trailing nontrivia reject the whole input.
/// The genuine brace and actual child braces count toward the existing nesting
/// limit. Retained implicit closures and lexical/unit recovery diagnostics prevent
/// clean validation. Font feature activation and winning definitions are downstream.
#[must_use]
pub fn parse_font_feature_values_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<Vec<crate::CssFontFeatureValuesItem>>>> {
    real_brace_block(
        source,
        "later.rule.font-feature-values",
        crate::CssParserContext::default(),
        |source, input, diagnostics, state| {
            let recovered = font_feature_values::parse_outer_body(source, input, state);
            diagnostics.extend(recovered.diagnostics);
            Ok(recovered.syntax)
        },
    )
}

/// Parses one genuine curly block of friendly definitions for the supplied kind.
///
/// Empty bodies and duplicate friendly names remain authored syntax. Integer
/// token grammar, exact index provenance and the kind's cardinality policy use
/// the actual subsidiary owner. Invalid named definitions and nested rules are
/// recovered locally, retaining admitted neighbors. Names and numbers keep their
/// actual original-source positions; the returned block's `position()` is `None`
/// because the semantic kind supplies no authored at-keyword.
///
/// Optional outer whitespace/comments remain in the snapshot outside the brace
/// origin. Missing braces or trailing nontrivia reject the whole input. The brace
/// counts once toward the existing nesting limit; retained implicit closures,
/// lexical faults and unit recovery diagnostics prevent clean validation.
#[must_use]
pub fn parse_font_feature_value_block(
    source: &str,
    kind: crate::CssFontFeatureValueKind,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssFontFeatureValueBlock>>> {
    real_brace_block(
        source,
        "later.rule.font-feature-values",
        crate::CssParserContext::default(),
        move |source, input, diagnostics, state| {
            let recovered = font_feature_values::parse_definition_body(source, input, kind, state);
            let block = crate::CssFontFeatureValueBlock::try_new(kind, recovered.syntax)
                .map_err(|_| crate::error::invalid_syntax(input.current_source_location()))?;
            diagnostics.extend(recovered.diagnostics);
            Ok(block)
        },
    )
}

/// Parses exactly one genuine curly body containing keyframe selector blocks.
///
/// Empty bodies, duplicate selectors and authored child order are retained.
/// Invalid selector/at-rule children and children with fatal declaration-body
/// failures are dropped as complete keyframe blocks, retaining admitted siblings.
/// Actual child selectors and declarations share the original source snapshot;
/// no animation name, endpoint or outer at-keyword is invented.
///
/// Optional outer whitespace/comments lie outside the brace origin. Missing
/// braces and trailing nontrivia reject the whole fragment. Actual outer/child
/// braces count toward the existing nesting limit. Implicit closures and native
/// unit recovery diagnostics prevent clean validation. Free parsing uses the
/// default context; [`crate::CssParserContext::parse_keyframes_block`] selects the
/// document grammar carried by the real declaration provider.
#[must_use]
pub fn parse_keyframes_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<Vec<crate::CssKeyframeBlock>>>> {
    parse_keyframes_block_with_context(source, crate::CssParserContext::default())
}

pub(crate) fn parse_keyframes_block_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<Vec<crate::CssKeyframeBlock>>>> {
    real_brace_block(
        source,
        "baseline.rule.keyframes",
        parser_context,
        |source, input, diagnostics, state| {
            Ok(keyframes::parse_body(source, input, diagnostics, state))
        },
    )
}

/// Parses exactly one genuine curly body of keyframe declarations.
///
/// Empty bodies and ordered duplicate/custom declarations are retained. The
/// actual keyframe grammar rejects importance and defining animation properties
/// except `animation-timing-function`, recovering those declarations locally.
/// Structural children are fatal and reject the complete body. No keyframe
/// selector or animation name is invented; declaration origins remain genuine.
///
/// Optional outer whitespace/comments lie outside the brace origin. Missing
/// braces or trailing nontrivia reject the whole fragment. The actual brace counts
/// toward the existing nesting limit, with per-declaration resource recovery.
/// Implicit closures and recovery diagnostics prevent clean validation. Free
/// parsing uses the default context; the same-named [`crate::CssParserContext`]
/// method carries its context into the actual property provider.
#[must_use]
pub fn parse_keyframe_declaration_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssKeyframeDeclarationList>>> {
    parse_keyframe_declaration_block_with_context(source, crate::CssParserContext::default())
}

pub(crate) fn parse_keyframe_declaration_block_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssKeyframeDeclarationList>>> {
    real_brace_block(
        source,
        "baseline.keyframes.block",
        parser_context,
        keyframes::parse_declarations,
    )
}

/// Parses exactly one genuine curly body in the composed authored Page domain.
///
/// Retains the ordered Page descriptor/property inventory and canonical margin-box
/// children. Applicable represented properties use their shared intrinsic grammar;
/// custom and substitution-dependent values stay symbolic. Invalid declarations and
/// unsupported structural children recover locally. No selector or at-keyword is
/// fabricated. A margin child admits properties/custom values, excluding Page descriptors.
///
/// Optional outer whitespace/comments lie outside the original brace origin.
/// Missing braces or trailing nontrivia reject the whole fragment. The brace
/// counts toward the existing nesting limit. Recovery and implicit closures
/// prevent clean validation. Free parsing uses the default document context;
/// [`crate::CssParserContext::parse_page_block`] carries its context into ordinary
/// property parsing before the shared Page admission boundary is applied.
#[must_use]
pub fn parse_page_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssPageBody>>> {
    parse_page_block_with_context(source, crate::CssParserContext::default())
}

pub(crate) fn parse_page_block_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssPageBody>>> {
    real_brace_block(
        source,
        "later.rule.page",
        parser_context,
        |source, input, diagnostics, state| Ok(page::parse_body(source, input, diagnostics, state)),
    )
}

/// Parses exactly one complete, grammar-valid ordinary declaration from raw source.
///
/// Surrounding whitespace and comments are accepted. The source must contain a
/// recognized property or a valid custom property, its colon, and its value;
/// top-level semicolons and stray closing delimiters reject the entire input.
/// Use [`super::parse_style_attribute`] for a declaration list with separators.
/// Name and value origins share the original source snapshot, and importance is
/// recognized by the ordinary declaration grammar. No annotation span is added.
/// Rejection returns `None` with `RejectInput`, except resource exhaustion retains
/// `StopAtNestingLimit`. Implicit EOF closures are reported only after the complete
/// declaration survives. This validates property grammar beyond CSS Syntax's
/// generic consume-declaration algorithm and performs no contextual resolution.
pub fn parse_declaration(source: &str) -> crate::CssParseReport<Option<CssDeclaration>> {
    parse_declaration_with_context(source, crate::CssParserContext::default())
}

pub(crate) fn parse_declaration_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
            .with_parser_context(parser_context);
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            input.skip_whitespace();
            let declaration_start = input.state();
            let name = input.expect_ident_cloned()?;
            input.expect_colon()?;
            let openings = state.check_component_values(source, &input, "css.declaration")?;
            // RuleBodyParser normally supplies list boundaries. This singular
            // entry instead rejects all root delimiters, while letting the
            // tokenizer skip nested blocks (including custom-property braces).
            let value_start = input.state();
            loop {
                let location = input.current_source_location();
                let Ok(token) = input.next_including_whitespace_and_comments().cloned() else {
                    break;
                };
                if matches!(
                    token,
                    Token::Semicolon
                        | Token::CloseCurlyBracket
                        | Token::CloseParenthesis
                        | Token::CloseSquareBracket
                ) {
                    return Err(location.new_unexpected_token_error::<Error>(token));
                }
                finish_nested_component(&mut input, &token)?;
            }
            input.reset(&value_start);
            let declaration = parse_declaration_core(
                DeclarationMode::Ordinary,
                name,
                &mut input,
                &declaration_start,
                state.source_snapshot(),
                parser_context,
            )?;
            input.expect_exhausted()?;
            state.retain_component_closures(openings);
            state.retain_navigation_diagnostic(&declaration.body);
            Ok(declaration.into_declaration())
        })();
        match result {
            Ok(declaration) => crate::CssParseReport::new(
                Some(declaration),
                state.take_implicit_closure_diagnostics(source),
            ),
            Err(error) => crate::CssParseReport::new(
                None,
                vec![reject(source, error, crate::CssRecoveryAction::RejectInput)],
            ),
        }
    })
}

fn selector_fragment<T>(
    source: &str,
    context: &CssNamespaceContext,
    single: bool,
    parse: impl for<'i> FnOnce(
        &mut Parser<'i, '_>,
        &mut SelectorRecovery<'_>,
    ) -> Result<T, ParseError<'i, Error>>,
) -> crate::CssParseReport<Option<T>> {
    let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
    if let Some(name) = &context.0.default {
        state.activate_namespace(None, name.clone());
    }
    for (prefix, name) in &context.0.named {
        state.activate_namespace(Some(prefix.clone()), name.clone());
    }
    let working_source = crate::tokenization::prepare(source);
    let mut input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut input);
    let mut diagnostics = Vec::new();
    let openings = if single {
        state.check_comma_member_components(source, &input, "baseline.selector.complex")
    } else {
        state.check_specialized_components(source, &input, "baseline.selector.complex")
    };
    let result = openings.and_then(|mut openings| {
        let value = parse(
            &mut input,
            &mut SelectorRecovery::new(source, &mut diagnostics, state.clone()),
        )?;
        input
            .expect_exhausted()
            .map_err(crate::error::selector_basic)?;
        // Preflight sees every lexical opening. Forgiving recovery can discard
        // a whole member, so its functions must not claim semantic retention.
        openings.retain(|opening| {
            !diagnostics.iter().any(|diagnostic| {
                diagnostic.action() == crate::CssRecoveryAction::DropSelectorListItem
                    && diagnostic.span().start().byte_offset().value() <= *opening
                    && *opening < diagnostic.span().end().byte_offset().value()
            })
        });
        state.retain_component_closures(openings);
        Ok(value)
    });
    let syntax = match result {
        Ok(value) => {
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            Some(value)
        }
        Err(error) => {
            diagnostics.push(reject(source, error, crate::CssRecoveryAction::RejectInput));
            None
        }
    };
    crate::CssParseReport::new(syntax, diagnostics)
}

/// Parses exactly one ordinary selector using the supplied namespace bindings.
///
/// Rejected outer syntax is `None` with diagnostics. Forgiving inner lists retain
/// admitted members and diagnose invalid members. Invalid delimiter-`&` members
/// remain explicit match-nothing items; other invalid members are discarded.
/// Positions refer directly to `source`;
/// implicit EOF closures are reported only when the outer syntax is retained.
/// Explicit `&` anchors remain symbolic; without a parent selector list they match
/// the context's scope elements and contribute zero specificity. Leading relative
/// combinators are not admitted by this front door.
pub fn parse_selector(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssSelector>> {
    bounded(source, || {
        selector_fragment(source, context, true, selectors::parse_rule_selector)
    })
}

/// Parses a complete ordinary selector list atomically, retaining forgiving inner recovery.
pub fn parse_selector_list(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssStyleSelectorList>> {
    bounded(source, || {
        selector_fragment(source, context, false, |input, recovery| {
            selectors::parse_rule_selector_list(input, recovery).map(|selectors| {
                CssStyleSelectorList::new(
                    selectors
                        .into_iter()
                        .map(CssStyleSelector::Selector)
                        .collect(),
                )
            })
        })
    })
}

/// Parses a complete nonempty general relative selector list atomically.
///
/// Each member retains its explicit leading `>`, `+`, `~` or `||` relationship,
/// or an implicit descendant relation. Namespace bindings and diagnostics use
/// the caller's original source. One invalid outer member rejects the whole list;
/// forgiving inner Is/Where recovery retains its existing diagnostics. Use
/// `into_validation_result` when only a clean report is acceptable.
///
/// General relative grammar permits legal terminal pseudo-elements and independent
/// Has functions. Attaching the resulting list as Has's own argument imposes its
/// narrower pseudo-element and nested-Has restrictions. Detached anchors remain
/// symbolic; parsing does not bind or match them. Specified-output limits apply
/// to later serialization or checked readmission, separately from parsing.
///
/// ```
/// use surgeist_css::{CssNamespaceContext, CssSelectorCombinator, parse_relative_selector_list};
/// let list = parse_relative_selector_list(
///     "> .Child, .Descendant", &CssNamespaceContext::default(),
/// ).into_validation_result().unwrap().unwrap();
/// assert_eq!(list.selectors()[0].combinator(), CssSelectorCombinator::Child);
/// assert_eq!(list.selectors()[1].combinator(), CssSelectorCombinator::Descendant);
/// assert_eq!(list.to_specified_css().unwrap(), "> .Child, .Descendant");
/// ```
///
/// An ordinary checked list cannot take a relative carrier:
///
/// ```compile_fail
/// use surgeist_css::{CssRelativeSelector, CssSelector, CssSelectorCombinator, CssSelectorList};
/// let relative = CssRelativeSelector::new(CssSelectorCombinator::Child, CssSelector::Class("One".into()));
/// let _ = CssSelectorList::try_new(vec![relative]);
/// ```
pub fn parse_relative_selector_list(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssRelativeSelectorList>> {
    bounded(source, || {
        selector_fragment(
            source,
            context,
            false,
            selectors::parse_general_relative_selector_list,
        )
    })
}

/// Parses exactly one media query, replacing rejected input with a never-matching query.
pub fn parse_media_query(source: &str) -> crate::CssParseReport<CssMediaQuery> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut input);
        let result = queries::check_media_member_components(source, &mut input, &state).and_then(
            |openings| {
                let query = queries::parse_media_query(
                    source,
                    &mut input,
                    &crate::numeric::NumericInputContext::parsed(state.source_snapshot()),
                )?;
                input.expect_exhausted()?;
                state.retain_component_closures(openings);
                Ok(query)
            },
        );
        match result {
            Ok(query) => {
                crate::CssParseReport::new(query, state.take_implicit_closure_diagnostics(source))
            }
            Err(error) => crate::CssParseReport::new(
                CssMediaQuery::Never(CssNeverMediaQuery::new(
                    crate::CssParsedOrigin::from_range(
                        state.source_snapshot(),
                        recovery::first_non_trivia_position(source, 0, source.len())
                            .byte_offset()
                            .value()..source.len(),
                    )
                    .expect("parsed media recovery origin"),
                )),
                vec![reject(
                    source,
                    if queries::media_terminal_error(&error) {
                        error
                    } else {
                        with_media_query_context(error, None)
                    },
                    crate::CssRecoveryAction::ReplaceMediaQueryWithNever,
                )],
            ),
        }
    })
}

/// Parses a complete media query list with independent recovery for each comma member.
///
/// An empty list is valid. Malformed members become `Never`, while grammatically
/// valid unknown features retain unknown truth. This operation
/// performs no matching or contextual evaluation.
pub fn parse_media_query_list(source: &str) -> crate::CssParseReport<CssMediaQueryList> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut input);
        let mut diagnostics = Vec::new();
        let query = match queries::parse_media_query_list_with_closures(
            source,
            &mut input,
            &mut diagnostics,
            &state,
        ) {
            Ok(parsed) => {
                state.retain_component_closures(parsed.implicit_closures);
                parsed.queries
            }
            Err(error) => {
                diagnostics.push(reject(
                    source,
                    error,
                    crate::CssRecoveryAction::ReplaceMediaQueryWithNever,
                ));
                CssMediaQueryList::new(vec![CssMediaQuery::Never(CssNeverMediaQuery::new(
                    crate::CssParsedOrigin::from_range(
                        state.source_snapshot(),
                        recovery::first_non_trivia_position(source, 0, source.len())
                            .byte_offset()
                            .value()..source.len(),
                    )
                    .expect("parsed media recovery origin"),
                ))])
            }
        };
        diagnostics.extend(state.take_implicit_closure_diagnostics(source));
        crate::CssParseReport::new(query, diagnostics)
    })
}

/// Parses the CSSOM single-query boundary through the owning media-query list parser.
///
/// Exactly one list member is returned, including a diagnosed ignored member
/// whose CSSOM serialization is `not all`. Empty or multiple-member lists return
/// `None`. Diagnostics retain their original ordering, spans and recovery actions;
/// this operation performs no host evaluation and does not erase unknown syntax.
///
/// ```
/// use surgeist_css::parse_cssom_media_query;
/// assert!(parse_cssom_media_query("screen, print").syntax().is_none());
/// let report = parse_cssom_media_query("???");
/// assert!(!report.is_clean());
/// assert_eq!(
///     report.syntax().as_ref().unwrap().serialize_cssom().unwrap().as_css(),
///     "not all"
/// );
/// ```
pub fn parse_cssom_media_query(source: &str) -> crate::CssParseReport<Option<CssMediaQuery>> {
    let (list, diagnostics) = parse_media_query_list(source).into_parts();
    crate::media::with_media_stack(
        list.queries().iter().any(crate::media::query_is_deep),
        || crate::CssParseReport::new(list.into_single_query(), diagnostics),
    )
}

/// Parses one complete raw `@font-face` descriptor value in the selected grammar.
///
/// The source contains only the value, without a descriptor name, annotation or
/// declaration delimiter. The returned typed value has no fabricated name position.
/// Diagnostics use the original source's UTF-8 bytes, zero-based UTF-16 columns and
/// actual EOF. Invalid outer values return `None` with `RejectInput`; resource limits
/// retain `StopAtNestingLimit`. A retained `src` list can report discarded members.
/// Implicit closures are reported only for retained components. This does not require
/// surrounding `font-family` or `src` descriptors, match fonts, or load resources.
pub fn parse_font_face_descriptor_value(
    source: &str,
    descriptor: CssFontFaceDescriptorKind,
) -> crate::CssParseReport<Option<crate::CssAuthoredFontFaceDescriptorValue>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let mut diagnostics = Vec::new();
        let result = (|| {
            let mut openings = state.check_component_values(source, &input, "css.descriptor")?;
            // A stylesheet parser bounds descriptor values before semicolons. A raw
            // value has no such enclosing parser; reject its own delimiters before
            // src member recovery could discard them as part of an invalid member.
            let value = font_face::parse_authored_font_face_value(
                source,
                &mut input,
                descriptor,
                state.source_snapshot(),
                &crate::numeric::NumericInputContext::parsed(state.source_snapshot()),
                &mut diagnostics,
                &mut openings,
            )?;
            input.expect_exhausted().map_err(|error| {
                crate::error::with_descriptor_context(
                    error.into(),
                    "font-face",
                    descriptor.css_name(),
                )
            })?;
            state.retain_component_closures(openings);
            Ok(value)
        })();
        let syntax = match result {
            Ok(value) => {
                diagnostics.extend(state.take_implicit_closure_diagnostics(source));
                Some(value)
            }
            Err(error) => {
                // Failed enclosing values cannot claim partial member retention.
                diagnostics.clear();
                diagnostics.push(reject(source, error, crate::CssRecoveryAction::RejectInput));
                None
            }
        };
        crate::CssParseReport::new(syntax, diagnostics)
    })
}

/// Parses the complete raw Page prelude, including empty, named, compound and comma-list forms.
/// All source coordinates belong to the original input; no at-rule wrapper is synthesized.
#[must_use]
pub fn parse_page_selector_list(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssPageSelectorList>> {
    descriptor_value_fragment(source, None, |input, state| {
        page::parse_page_selector(input, state.source_snapshot())
    })
}
/// Parses one complete raw Page descriptor value without a descriptor-name origin or priority.
#[must_use]
pub fn parse_page_descriptor_value(
    source: &str,
    kind: crate::CssPageDescriptorKind,
) -> crate::CssParseReport<Option<crate::CssPageDescriptorValue>> {
    descriptor_value_fragment(source, Some(("page", kind.css_name())), |input, state| {
        let (value, components, origin) =
            collect_declaration_value(input, state.source_snapshot(), |input| {
                page::parse_descriptor_value(input, kind, state.source_snapshot())
            })?;
        Ok(crate::CssPageDescriptorValue::from_parsed(
            kind, value, components, origin,
        ))
    })
}
/// Parses one genuine margin descriptor block; Page-only descriptors and structural children recover locally.
#[must_use]
pub fn parse_margin_block(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssMarginDeclarationBlock>>> {
    parse_margin_block_with_context(source, crate::CssParserContext::default())
}
pub(crate) fn parse_margin_block_with_context(
    source: &str,
    context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::CssBlockFragment<crate::CssMarginDeclarationBlock>>> {
    real_brace_block(
        source,
        "later.rule.page",
        context,
        |source, input, diagnostics, state| {
            Ok(page::parse_margin_body(source, input, diagnostics, state))
        },
    )
}

// These name-free fronts share whole-source rejection, descriptor annotation
// policy and lexical/resource ownership; each closure uses its family provider.
fn descriptor_value_fragment<T: Send>(
    source: &str,
    descriptor: Option<(&str, &str)>,
    parse: impl for<'i, 't> FnOnce(
        &mut Parser<'i, 't>,
        &RecoveryState,
    ) -> Result<T, ParseError<'i, Error>>
    + Send,
) -> crate::CssParseReport<Option<T>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings = state.check_component_values(source, &input, "css.descriptor")?;
            let value = if let Some((owner, descriptor)) = descriptor {
                parse_descriptor_boundary(&mut input, owner, descriptor, |input| {
                    let value = parse(input, &state)?;
                    input.expect_exhausted().map_err(basic)?;
                    Ok(value)
                })?
            } else {
                parse(&mut input, &state)?
            };
            input.expect_exhausted().map_err(basic)?;
            state.retain_component_closures(openings);
            Ok(value)
        })()
        .map_err(|error| {
            if crate::error::is_resource_parse_error(&error)
                || matches!(
                    &error.kind,
                    cssparser::ParseErrorKind::Custom(error)
                        if matches!(error.kind(), crate::ErrorKind::InvalidComponentValue(_))
                )
            {
                error
            } else if let Some((owner, descriptor)) = descriptor {
                crate::error::with_descriptor_context(error, owner, descriptor)
            } else {
                error
            }
        });
        let (syntax, diagnostics) = match result {
            Ok(value) => (Some(value), state.take_implicit_closure_diagnostics(source)),
            Err(error) => (
                None,
                vec![reject(source, error, crate::CssRecoveryAction::RejectInput)],
            ),
        };
        crate::CssParseReport::new(syntax, diagnostics)
    })
}

/// Parses one complete raw Counter Styles 3 descriptor value.
///
/// The supplied kind selects the existing descriptor grammar without a rule name
/// or descriptor-name occurrence. Components and origin cover the complete input,
/// including trivia; exact numeric values retain their original token origins.
/// Cross-descriptor system/symbol requirements remain with the enclosing rule.
/// Empty, invalid, annotated or trailing input returns `None` with whole-input
/// rejection. The fixed 256-depth ceiling retains typed resource diagnostics.
/// Implicit lexical EOF recovery may retain a value with an unclean report;
/// generic clean-report validation accepts exactly reports without diagnostics.
#[must_use]
pub fn parse_counter_style_descriptor_value(
    source: &str,
    descriptor: crate::CssCounterStyleDescriptorKind,
) -> crate::CssParseReport<Option<crate::CssCounterStyleDescriptorValue>> {
    descriptor_value_fragment(
        source,
        Some(("counter-style", descriptor.css_name())),
        |input, state| {
            let (value, components, origin) =
                collect_declaration_value(input, state.source_snapshot(), |input| {
                    // Keep the collector's complete source range, while an
                    // empty grammar reports the EOF after leading trivia.
                    input.skip_whitespace();
                    counter_style::parse_authored_value(input, descriptor, state.source_snapshot())
                })?;
            Ok(crate::CssCounterStyleDescriptorValue::from_parsed(
                descriptor, value, components, origin,
            ))
        },
    )
}

/// Parses the outer `@font-feature-values` font-display descriptor's complete value.
///
/// The five font-display keywords admit decoded escaped and ASCII-insensitive
/// spellings; valid env defers the whole value. There is no invented
/// descriptor-name position. Components and origin retain all original trivia.
/// Invalid or trailing input and root annotations reject the complete fragment;
/// resource failures preserve the fixed 256-depth ceiling. Lexical EOF recovery
/// remains visible in diagnostics and fails generic clean-report validation.
#[must_use]
pub fn parse_font_feature_display_value(
    source: &str,
) -> crate::CssParseReport<Option<crate::CssFontFeatureDisplayValue>> {
    descriptor_value_fragment(
        source,
        Some(("font-feature-values", "font-display")),
        |input, state| {
            let (value, components, origin) =
                collect_declaration_value(input, state.source_snapshot(), |input| {
                    let value =
                        font_feature_values::parse_display_value(input, state.source_snapshot())?;
                    input.expect_exhausted().map_err(basic)?;
                    Ok(value)
                })?;
            Ok(crate::CssFontFeatureDisplayValue::from_parsed(
                value, components, origin,
            ))
        },
    )
}

/// Parses one authored subsidiary feature value, including whole-value env deferral.
///
/// Historical-forms/styleset admit one or more indexes, character-variant one or
/// two, and the other kinds exactly one. Integer token spelling is retained in
/// components; normalized exact indexes have no machine-integer upper bound.
/// Valid env defers the completed index grammar; plain var grants no exception.
/// No friendly definition name is created. Invalid, empty, annotated or trailing
/// input rejects the whole fragment; token failures use their original origins.
/// Components and carrier origin include all trivia. The fixed 256-depth ceiling
/// and lexical EOF diagnostics retain their existing owners; clean validation
/// accepts exactly reports without diagnostics and performs no font activation.
/// Name-free token/EOF diagnostics describe the selected integer grammar; root
/// `!` is rejected as its actual delimiter rather than a named annotation.
/// An overfull list reports its first surplus integer; empty input reports EOF.
#[must_use]
pub fn parse_font_feature_value(
    source: &str,
    kind: crate::CssFontFeatureValueKind,
) -> crate::CssParseReport<Option<crate::CssFontFeatureValue>> {
    descriptor_value_fragment(source, None, |input, state| {
        let (value, components, origin) =
            collect_declaration_value(input, state.source_snapshot(), |input| {
                font_feature_values::parse_feature_value(input, kind, state.source_snapshot())
            })?;
        Ok(crate::CssFontFeatureValue::from_parsed(
            kind, value, components, origin,
        ))
    })
}

/// Parses one complete raw `@font-palette-values` descriptor value.
///
/// The selected descriptor grammar is checked without a surrounding rule or
/// fabricated descriptor-name position. Valid var()/env() defers the whole
/// value. Retained components and diagnostics refer to the original input;
/// this function neither substitutes values nor performs font lookup.
pub fn parse_font_palette_descriptor_value(
    source: &str,
    descriptor: CssFontPaletteDescriptorKind,
) -> crate::CssParseReport<Option<CssFontPaletteDescriptorValue>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings = state.check_component_values(source, &input, "css.descriptor")?;
            let value = font_palette_values::parse_descriptor_value_from_parser(
                &mut input, descriptor, &state,
            )?;
            input.expect_exhausted().map_err(basic)?;
            state.retain_component_closures(openings);
            Ok(value)
        })();
        let (syntax, diagnostics) = match result {
            Ok(value) => (Some(value), state.take_implicit_closure_diagnostics(source)),
            Err(error) => (
                None,
                vec![reject(source, error, crate::CssRecoveryAction::RejectInput)],
            ),
        };
        crate::CssParseReport::new(syntax, diagnostics)
    })
}

/// Parses one complete authored color-profile descriptor with env-only deferral.
pub fn parse_color_profile_descriptor_value(
    source: &str,
    descriptor: CssColorProfileDescriptorKind,
) -> crate::CssParseReport<Option<CssColorProfileDescriptorValue>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings = state.check_component_values(source, &input, "css.descriptor")?;
            let value =
                color_profile::parse_descriptor_value_from_parser(&mut input, descriptor, &state)?;
            input.expect_exhausted().map_err(basic)?;
            state.retain_component_closures(openings);
            Ok(value)
        })();
        let (syntax, diagnostics) = match result {
            Ok(value) => (Some(value), state.take_implicit_closure_diagnostics(source)),
            Err(error) => (
                None,
                vec![reject(source, error, crate::CssRecoveryAction::RejectInput)],
            ),
        };
        crate::CssParseReport::new(syntax, diagnostics)
    })
}

/// Parses a complete raw property value using a supplied semantic property name.
///
/// Importance is supplied separately: root annotations, semicolons and stray
/// closing delimiters reject the entire source, including after substitutions.
/// Empty custom values and nested custom-property punctuation are accepted.
/// The declaration has no parsed name or name position; its parsed value and
/// components share the original source snapshot. No source wrapper or value
/// serialization is introduced. Values remain authored and symbolic.
/// Invalid grammar returns `None` with `RejectInput`; resource exhaustion uses
/// `StopAtNestingLimit`. Implicit EOF closures are reported only on retention.
pub fn parse_property_value_text(
    source: &str,
    property: CssPropertyNameRef<'_>,
    importance: CssImportance,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    parse_property_value_text_with_context(
        source,
        property,
        importance,
        crate::CssParserContext::default(),
    )
}

pub(crate) fn parse_property_value_text_with_context(
    source: &str,
    property: CssPropertyNameRef<'_>,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    let grammar = match property {
        CssPropertyNameRef::Known(property) => PropertyValueGrammar::Known(property.grammar()),
        CssPropertyNameRef::Custom(name) => PropertyValueGrammar::Custom(name),
        CssPropertyNameRef::SvgGlyphOrientationVertical => {
            PropertyValueGrammar::SvgGlyph(crate::svg_glyph::SvgGlyphAdmission::css(parser_context))
        }
    };
    property_value_text(source, grammar, importance, parser_context)
}

/// Parses a complete raw value with an explicit canonical or legacy grammar.
///
/// This shares source provenance, separate importance, resource bounds and
/// complete-input rejection with [`parse_property_value_text`]. The declaration
/// retains the selected grammar for ordinary, CSS-wide and symbolic values.
pub fn parse_property_value_text_for_grammar(
    source: &str,
    grammar: CssPropertyGrammar,
    importance: CssImportance,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    parse_property_value_text_for_grammar_with_context(
        source,
        grammar,
        importance,
        crate::CssParserContext::default(),
    )
}

pub(crate) fn parse_property_value_text_for_grammar_with_context(
    source: &str,
    grammar: CssPropertyGrammar,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    property_value_text(
        source,
        PropertyValueGrammar::Known(grammar),
        importance,
        parser_context,
    )
}

pub(crate) fn parse_svg_glyph_attribute_value(
    source: &str,
    context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    property_value_text(
        source,
        PropertyValueGrammar::SvgGlyph(crate::svg_glyph::SvgGlyphAdmission::attribute(context)),
        CssImportance::Normal,
        context,
    )
}

fn property_value_text(
    source: &str,
    grammar: PropertyValueGrammar<'_>,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
            .with_parser_context(parser_context);
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings = state.check_component_values(source, &input, "css.declaration")?;
            let (body, components, origin) =
                collect_declaration_value(&mut input, state.source_snapshot(), |input| {
                    parse_property_value_from_parser(
                        grammar,
                        source,
                        input,
                        &crate::numeric::NumericInputContext::parsed(state.source_snapshot()),
                        parser_context,
                    )
                })?;
            input.expect_exhausted()?;
            state.retain_component_closures(openings);
            state.retain_navigation_diagnostic(&body);
            Ok(CssDeclaration::new_parsed_value(
                parser_context,
                body,
                importance,
                components,
                origin,
            ))
        })();
        match result {
            Ok(declaration) => crate::CssParseReport::new(
                Some(declaration),
                state.take_implicit_closure_diagnostics(source),
            ),
            Err(error) => crate::CssParseReport::new(
                None,
                vec![reject(source, error, crate::CssRecoveryAction::RejectInput)],
            ),
        }
    })
}

// Distinguish a rejected first rule from parse_one_rule's trailing-input error.
// The latter must not be reinterpreted as an error in the completed rule body.
struct SingleRuleParser<'s> {
    grammar: StrictRuleParser<'s>,
    completed: bool,
}

impl<'i> AtRuleParser<'i> for SingleRuleParser<'i> {
    type Prelude = StrictAtRulePrelude;
    type AtRule = Vec<CssRule>;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Error>> {
        AtRuleParser::parse_prelude(&mut self.grammar, name, input)
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        let result = self.grammar.rule_without_block(prelude, start);
        self.completed = result.is_ok();
        result
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Error>> {
        let result = AtRuleParser::parse_block(&mut self.grammar, prelude, start, input);
        self.completed = result.is_ok();
        result
    }
}

impl<'i> QualifiedRuleParser<'i> for SingleRuleParser<'i> {
    type Prelude = Vec<CssSelector>;
    type QualifiedRule = Vec<CssRule>;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Error>> {
        QualifiedRuleParser::parse_prelude(&mut self.grammar, input)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Error>> {
        let result = QualifiedRuleParser::parse_block(&mut self.grammar, prelude, start, input);
        self.completed = result.is_ok();
        result
    }
}

/// Parses exactly one complete ordinary rule using supplied namespace bindings.
///
/// The original source must contain one supported style rule or at-rule, with
/// optional surrounding whitespace/comments. Multiple rules, trailing nontrivia
/// and rejected outer grammar return `None` with `RejectInput`. Encoding directives
/// are stylesheet metadata and cannot produce a rule; no BOM or CDO/CDC is stripped.
/// Imports and namespaces use isolated top-level grammar, without insertion-order
/// validation against an existing sheet. The supplied namespace context is immutable.
///
/// A retained rule preserves its inner declaration, selector, query and child-rule
/// recovery diagnostics. Implicit EOF closures are reported only when the outer
/// rule survives; resource diagnostics preserve `StopAtNestingLimit`. Positions
/// refer to the original UTF-8 source and zero-based UTF-16 coordinates. Nested
/// contents remain inside their owning rule. This performs no matching, cascade,
/// CSSOM insertion or contextual resolution.
pub fn parse_rule(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssRule>> {
    parse_rule_with_context(source, context, crate::CssParserContext::default())
}

pub(crate) fn parse_rule_with_context(
    source: &str,
    context: &CssNamespaceContext,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssRule>> {
    bounded(source, || {
        let snapshot = CssSourceSnapshot::new(source);
        let report = parse_bounded(
            source,
            &snapshot,
            0,
            BoundedParseContext::OneRule,
            StyleContextCaptures::with_namespaces(context),
            parser_context,
        );
        let (syntax, diagnostics) = report.into_parts();
        let BoundedParseSyntax::OneRule(rule) = syntax else {
            unreachable!("exact-one replay preserves its fragment carrier")
        };
        match rule {
            Some(rule) => conditional_chains::rule(source, rule, diagnostics),
            None => crate::CssParseReport::new(None, diagnostics),
        }
    })
}

pub(super) fn parse_rule_inner(
    source: &str,
    state: RecoveryState,
) -> crate::CssParseReport<Option<CssRule>> {
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let mut parser = SingleRuleParser {
        grammar: StrictRuleParser::top_level(source, state.clone()),
        completed: false,
    };
    input.skip_whitespace();
    let rule_start = input.position().byte_index();
    match super::syntax_bridge::one(source, &mut input, &mut parser, &state) {
        Ok(rules) => {
            // The ordinary grammar emits one outer node for a successful rule.
            let [rule]: [CssRule; 1] = rules
                .try_into()
                .expect("a successful isolated ordinary rule emits one outer node");
            let mut diagnostics = parser.grammar.diagnostics;
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(Some(rule), diagnostics)
        }
        Err(error) => {
            let action = recovery_action_for_error(&error, crate::CssRecoveryAction::RejectInput);
            let error = if parser.completed {
                from_parse_error(source, error)
            } else {
                let failed_unit = &source[rule_start..input.position().byte_index()];
                from_rule_parse_error(source, failed_unit, error)
            };
            crate::CssParseReport::new(None, vec![reject_resolved(source, error, action)])
        }
    }
}

/// Parses exactly one real-brace style block with supplied namespace bindings.
///
/// The source contains the braces and optional surrounding whitespace/comments,
/// without a selector. Empty and recovered-empty blocks are retained. A second
/// block, missing opening brace or trailing nontrivia rejects the complete input
/// with `RejectInput`; resource failures preserve `StopAtNestingLimit`.
///
/// Inner declarations, nested rules and subsequent declaration runs use ordinary
/// style-body grammar and preserve their recovery diagnostics. Relative child
/// selectors and explicit anchors remain symbolic. No parent selector is invented.
/// The block origin includes its actual braces or ends at implicit EOF, excludes
/// surrounding trivia, and shares the original source snapshot with declarations.
/// Implicit closure diagnostics are published only after the outer block survives.
pub fn parse_style_block(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssStyleBlock>> {
    parse_style_block_with_context(source, context, crate::CssParserContext::default())
}

pub(crate) fn parse_style_block_with_context(
    source: &str,
    context: &CssNamespaceContext,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssStyleBlock>> {
    bounded(source, || {
        let snapshot = CssSourceSnapshot::new(source);
        let report = parse_bounded(
            source,
            &snapshot,
            0,
            BoundedParseContext::StyleBlock,
            StyleContextCaptures::with_namespaces(context),
            parser_context,
        );
        let (syntax, diagnostics) = report.into_parts();
        let BoundedParseSyntax::StyleBlock(block) = syntax else {
            unreachable!("style-block replay preserves its genuine-brace carrier")
        };
        match block {
            Some(block) => conditional_chains::style_block(source, block, diagnostics),
            None => crate::CssParseReport::new(None, diagnostics),
        }
    })
}

pub(super) fn parse_style_block_inner(
    source: &str,
    state: RecoveryState,
) -> crate::CssParseReport<Option<CssStyleBlock>> {
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let result = (|| {
        input.skip_whitespace();
        let start = input.position().byte_index();
        input.expect_curly_bracket_block()?;
        let recovered = input.parse_nested_block(|input| {
            let mut depth = state.enter_rule_block(source, input, "official.value.style-block")?;
            let recovered = parse_style_contents(source, input, state.clone())?;
            depth.retain();
            Ok(recovered)
        })?;
        let end = input.position().byte_index();
        input.expect_exhausted()?;
        let origin = CssParsedOrigin::from_range(state.source_snapshot(), start..end)
            .expect("consumed block boundaries belong to the original source");
        Ok((
            CssStyleBlock::new(recovered.syntax, origin),
            recovered.diagnostics,
        ))
    })();
    match result {
        Ok((block, mut diagnostics)) => {
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(Some(block), diagnostics)
        }
        Err(error) => crate::CssParseReport::new(
            None,
            vec![reject(source, error, crate::CssRecoveryAction::RejectInput)],
        ),
    }
}

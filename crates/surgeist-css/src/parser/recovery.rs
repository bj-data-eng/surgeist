use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cssparser::{ParseError, Parser, ParserInput, Token};

use crate::diagnostic::CssError;
use crate::diagnostic::error::{is_nesting_limit_error, nesting_limit};
use crate::rule::{CssNamespaceName, CssNamespacePrefix};
use crate::source::CssSourcePosition;

use super::CssNamespaceBindings;

pub(super) static IMPLEMENTED_RULES: &[crate::diagnostic::CssFeatureId] =
    &[crate::diagnostic::CssFeatureId::new(
        "official.rule.at-rule",
    )];

pub(super) static IMPLEMENTED_QUALIFIED_RULES: &[crate::diagnostic::CssFeatureId] =
    &[crate::diagnostic::CssFeatureId::new(
        "official.qualified-rule.generic",
    )];

pub(super) static IMPLEMENTED_SHARED_VALUES: &[crate::diagnostic::CssFeatureId] = &[
    crate::diagnostic::CssFeatureId::new("official.value.rule-list"),
    crate::diagnostic::CssFeatureId::new("official.value.style-block"),
];

pub(super) use crate::STRUCTURAL_NESTING_LIMIT;

/// Domain replacement/admission must not publish a payload after a typed
/// resource failure, including one recovered inside a member callback.
pub(super) fn has_resource_failure(
    diagnostics: &[crate::diagnostic::CssRecoveryDiagnostic],
) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| match diagnostic.error().kind() {
            crate::diagnostic::error::ErrorKind::NestingLimit(_) => true,
            crate::diagnostic::error::ErrorKind::InvalidComponentValue(error) => {
                crate::diagnostic::error::is_component_resource_error(error)
            }
            _ => false,
        })
}

/// Publish tokenizer recovery once, after the public grammar entry finishes.
/// Lexical errors are independent of whether the enclosing grammar unit survived.
/// Internal probes and recursive parses leave this step to the caller with the
/// original complete source.
pub(crate) fn finish_report<T>(
    source: &str,
    report: crate::CssParseReport<T>,
) -> crate::CssParseReport<T> {
    let (syntax, mut diagnostics) = report.into_parts();
    for diagnostic in &mut diagnostics {
        diagnostic.clear_opening_owner();
    }
    let mut offset = 0;
    while let Some((token_start, token_end, token)) = next_source_token(source, offset) {
        offset = token_end;
        if matches!(token, Token::Delim('\\'))
            && matches!(
                source.as_bytes().get(token_end),
                Some(b'\n' | b'\r' | b'\x0c')
            )
        {
            diagnostics.push(escape_diagnostic(
                source,
                token_start..token_end,
                crate::diagnostic::CssEscapeFailure::Newline,
            ));
            continue;
        }
        if token_end == source.len()
            && matches!(
                token,
                Token::Ident(_)
                    | Token::AtKeyword(_)
                    | Token::Hash(_)
                    | Token::IDHash(_)
                    | Token::Dimension { .. }
            )
            && crate::component::values::parse::odd_trailing_backslashes(
                &source[token_start..token_end],
            )
        {
            diagnostics.push(escape_diagnostic(
                source,
                token_end - 1..token_end,
                crate::diagnostic::CssEscapeFailure::EndOfInput,
            ));
            continue;
        }
        if !matches!(token, Token::Comment(_)) {
            // Strings, URLs and bad tokens consume their own payload. Their
            // existing grammar/EOF diagnostics already describe that recovery.
            continue;
        }
        // Only comment tokens participate: strings and URL tokens (including
        // bad tokens) have already consumed their own comment-looking payload.
        // Exclude the opening delimiter before checking the terminator so `/*/`
        // cannot reuse its opening '*' as part of a closing delimiter.
        let contents = &source[token_start + 2..token_end];
        if contents.ends_with("*/") {
            continue;
        }
        let start = CssSourcePosition::from_byte_offset_in(source, token_start);
        let eof = CssSourcePosition::from_byte_offset_in(source, source.len());
        let span =
            crate::source::CssSourceSpan::new(start, eof).expect("comment ends at source EOF");
        let diagnostic = crate::diagnostic::CssRecoveryDiagnostic::new(
            crate::diagnostic::error::implicit_eof(source),
            span,
            crate::diagnostic::CssRecoveryAction::IgnoreUnterminatedComment,
        )
        .expect("comment EOF belongs to its consumed source span");
        diagnostics.push(diagnostic);
    }
    crate::CssParseReport::new(syntax, diagnostics)
}

fn escape_diagnostic(
    source: &str,
    range: std::ops::Range<usize>,
    detail: crate::diagnostic::CssEscapeFailure,
) -> crate::diagnostic::CssRecoveryDiagnostic {
    let responsible = match detail {
        crate::diagnostic::CssEscapeFailure::Newline => range.start,
        crate::diagnostic::CssEscapeFailure::EndOfInput => range.end,
    };
    let span = crate::source::CssSourceSpan::new(
        CssSourcePosition::from_byte_offset_in(source, range.start),
        CssSourcePosition::from_byte_offset_in(source, range.end),
    )
    .expect("consumed escape ranges are ordered UTF-8 boundaries");
    crate::diagnostic::CssRecoveryDiagnostic::new(
        crate::diagnostic::error::escape_parse_error(source, responsible, detail),
        span,
        crate::diagnostic::CssRecoveryAction::RecoverEscape,
    )
    .expect("escape error is anchored within its consumed backslash range")
}

pub(super) fn maximum_nested_depth(source: &str) -> u32 {
    scan_delimiters(source, 0, StructuralListContext::Stylesheet).maximum
}

/// Returns the first opening delimiter enclosed by exactly `depth` open blocks,
/// matching blocks as [`maximum_nested_depth`] does.
pub(super) fn first_opening_at_depth(source: &str, depth: u32) -> Option<usize> {
    let mut offset = 0;
    let mut blocks: Vec<BlockKind> = Vec::new();
    while let Some((token_start, token_end, token)) = next_source_token(source, offset) {
        offset = token_end;
        if let Some(opening) = opening_block(&token) {
            if blocks.len() as u32 == depth {
                return Some(token_start);
            }
            blocks.push(opening);
        } else if let Some(closing) = closing_block(&token)
            && blocks.last() == Some(&closing)
        {
            blocks.pop();
        }
    }
    None
}

pub(super) struct SpecializedEofLimit {
    pub(super) unit_start: usize,
    pub(super) opening_offset: usize,
    pub(super) enclosing_production: &'static str,
}

pub(super) fn preflight_specialized_eof_limit(
    source: &str,
    base_depth: u32,
    list: StructuralListContext,
) -> Option<SpecializedEofLimit> {
    let scan = scan_delimiters(source, base_depth, list);
    let target = scan.eof_limit?;
    let at_rule = next_source_token(source, target.unit_start).and_then(|(_, _, token)| {
        if let Token::AtKeyword(name) = token {
            Some(name)
        } else {
            None
        }
    });
    let rule_production = at_rule.as_deref().and_then(|name| {
        if name.eq_ignore_ascii_case("media") {
            Some("official.media.query-list-core")
        } else if name.eq_ignore_ascii_case("supports") {
            Some("baseline.rule.supports")
        } else if name.eq_ignore_ascii_case("container") {
            Some("baseline.rule.container")
        } else {
            None
        }
    });
    if target
        .first_root_curly
        .is_some_and(|curly| curly < target.opening_offset)
    {
        // The body parser owns limits after the rule's prelude has ended.
        return None;
    }
    Some(SpecializedEofLimit {
        unit_start: target.unit_start,
        opening_offset: target.opening_offset,
        enclosing_production: rule_production.unwrap_or("baseline.selector.complex"),
    })
}

struct DelimiterScan {
    maximum: u32,
    unclosed: Vec<usize>,
    eof_limit: Option<DelimiterLimitTarget>,
    over_limit_components: Vec<DeniedComponent>,
}

struct DeniedComponent {
    range: std::ops::Range<usize>,
    block: BlockKind,
}

struct DelimiterLimitTarget {
    unit_start: usize,
    opening_offset: usize,
    first_root_curly: Option<usize>,
}

// Root units end where the structural preflight's list consumer ends them:
// Syntax 3 (2026 CRD) §5.5.1 keeps a stylesheet-root `;` inside a qualified
// rule's prelude and discards CDO/CDC before a unit, while §5.5.5 ends every
// block-contents unit at a `;`. Only an at-rule ends at its own `;` (§5.5.2).
fn scan_delimiters(source: &str, base_depth: u32, list: StructuralListContext) -> DelimiterScan {
    let mut offset = 0;
    let mut blocks: Vec<(BlockKind, usize)> = Vec::new();
    let mut unclosed_token = None;
    let mut maximum = base_depth;
    let mut unit: Option<StructuralUnit> = None;
    let mut first_root_curly = None;
    let mut target = None;
    let mut over_limit_components = Vec::new();
    let mut over_limit_component = None;

    while let Some((token_start, token_end, token)) = next_source_token(source, offset) {
        offset = token_end;
        if matches!(token, Token::WhiteSpace(_) | Token::Comment(_)) {
            continue;
        }
        if blocks.is_empty() {
            if list.ignores_prefix(&token, unit.is_none()) {
                continue;
            }
            if matches!(token, Token::Semicolon) && list.semicolon_ends_unit(unit) {
                unit = None;
                first_root_curly = None;
                continue;
            }
            unit.get_or_insert(StructuralUnit {
                start: token_start,
                dispatch: crate::syntax_consumption::rule_dispatch(
                    crate::syntax_consumption::native_token_kind(&token),
                ),
            });
        }

        let token_closing = match &token {
            Token::UnquotedUrl(_) => Some(b')'),
            Token::QuotedString(_) => source.as_bytes().get(token_start).copied(),
            _ => None,
        };
        if let Some(closing) = token_closing {
            // URL and string tokens consume their own punctuation. Retained EOF
            // termination reports recovery without adding structural nesting.
            // Reuse the component owner's exact escaped-terminator predicate.
            if source.get(token_start..token_end).is_some_and(|spelling| {
                !crate::component::values::parse::has_unescaped_final(spelling, closing)
            }) {
                unclosed_token = Some(token_start);
            }
            continue;
        }

        if let Some(opening) = opening_block(&token) {
            // Guards identify the actual opening delimiter. A function token
            // ends at that delimiter even when its name contains CSS escapes.
            let opening_offset = if matches!(token, Token::Function(_)) {
                token_end.saturating_sub(1)
            } else {
                token_start
            };
            if blocks.is_empty() && opening == BlockKind::Curly {
                first_root_curly = Some(token_start);
            }
            let depth = base_depth.saturating_add(blocks.len() as u32);
            if over_limit_component.is_none() && depth >= STRUCTURAL_NESTING_LIMIT {
                over_limit_component = Some((token_start, blocks.len(), opening));
            }
            if target.is_none() && depth >= STRUCTURAL_NESTING_LIMIT {
                target = Some(DelimiterLimitTarget {
                    unit_start: unit.map_or(token_start, |unit| unit.start),
                    opening_offset: token_start,
                    first_root_curly,
                });
            }
            blocks.push((opening, opening_offset));
            maximum = maximum.max(base_depth.saturating_add(blocks.len() as u32));
            continue;
        }

        if let Some(closing) = closing_block(&token)
            && blocks.last().is_some_and(|(kind, _)| *kind == closing)
        {
            blocks.pop();
            if let Some((start, parent_depth, block)) = over_limit_component
                && blocks.len() == parent_depth
            {
                over_limit_components.push(DeniedComponent {
                    range: start..token_end,
                    block,
                });
                over_limit_component = None;
            }
            if blocks.is_empty() {
                // A completed prelude component does not terminate its rule.
                if closing == BlockKind::Curly {
                    unit = None;
                    first_root_curly = None;
                }
                target = None;
            }
        }
    }

    if let Some((start, _, block)) = over_limit_component {
        over_limit_components.push(DeniedComponent {
            range: start..source.len(),
            block,
        });
    }

    DelimiterScan {
        maximum,
        unclosed: blocks
            .into_iter()
            .map(|(_, offset)| offset)
            .chain(unclosed_token)
            .collect(),
        eof_limit: target,
        over_limit_components,
    }
}

// The generic source arena selects the actual enclosing units before native
// admission. Omit only components already beyond the shared depth ceiling from
// its private recovery view. Denied Curly boundaries remain opaque structural
// units, so their payload cannot absorb an independent following rule. Native
// callbacks retain their unchanged source and own resource diagnosis/consumption.
fn source_normalization_view(source: &str, base_depth: u32) -> Cow<'_, str> {
    let ranges = scan_delimiters(source, base_depth, StructuralListContext::Stylesheet)
        .over_limit_components;
    if ranges.is_empty() {
        return Cow::Borrowed(source);
    }
    let mut view = source.as_bytes().to_vec();
    for component in ranges {
        // A denied `{}` block stays opaque in the document, which denies it
        // by its own nesting; any other component is masked.
        if component.block == BlockKind::Curly {
            continue;
        }
        for byte in &mut view[component.range] {
            if !matches!(*byte, b'\n' | b'\r' | b'\x0c') {
                *byte = b' ';
            }
        }
    }
    Cow::Owned(String::from_utf8(view).expect("ASCII masking preserves UTF-8"))
}

/// Parser-owned algorithm state shared by structural and component-value paths.
///
/// The stylesheet root begins at depth zero. A checked guard is the only way to
/// enter a rule block, component-value block, or function, so C04 can extend the
/// same counter without changing the limit or its boundary meaning.
#[derive(Clone)]
pub(crate) struct RecoveryState {
    parser_context: crate::CssParserContext,
    depth: Rc<Cell<u32>>,
    source_snapshot: crate::source::CssSourceSnapshot,
    style_context_captures: StyleContextCaptures,
    namespace_bindings: Rc<RefCell<CssNamespaceBindings>>,
    implicit_openings: Rc<Vec<usize>>,
    retained_implicit_openings: Rc<RefCell<Vec<usize>>>,
    retained_navigation_diagnostics: Rc<RefCell<Vec<crate::diagnostic::CssRecoveryDiagnostic>>>,
    syntax_base_depth: u32,
    syntax_document: Rc<RefCell<Option<Rc<crate::parser::syntax_types::Document<'static>>>>>,
}

impl RecoveryState {
    pub(super) fn at_depth(
        source: &str,
        depth: u32,
        style_context_captures: StyleContextCaptures,
    ) -> Self {
        Self::at_depth_with_snapshot(
            source,
            depth,
            style_context_captures,
            crate::source::CssSourceSnapshot::new(source),
        )
    }

    pub(crate) fn at_depth_with_snapshot(
        source: &str,
        depth: u32,
        style_context_captures: StyleContextCaptures,
        source_snapshot: crate::source::CssSourceSnapshot,
    ) -> Self {
        let namespace_bindings = Rc::clone(&style_context_captures.namespace_bindings);
        Self {
            parser_context: crate::CssParserContext::default(),
            depth: Rc::new(Cell::new(depth)),
            source_snapshot,
            style_context_captures,
            namespace_bindings,
            implicit_openings: Rc::new(unclosed_openings(source)),
            retained_implicit_openings: Rc::new(RefCell::new(Vec::new())),
            retained_navigation_diagnostics: Rc::new(RefCell::new(Vec::new())),
            syntax_base_depth: depth,
            syntax_document: Rc::new(RefCell::new(None)),
        }
    }

    pub(crate) fn with_parser_context(mut self, context: crate::CssParserContext) -> Self {
        self.parser_context = context;
        self
    }

    pub(super) const fn parser_context(&self) -> crate::CssParserContext {
        self.parser_context
    }

    pub(super) fn detached_probe(&self) -> Self {
        Self {
            parser_context: self.parser_context,
            depth: Rc::new(Cell::new(self.depth.get())),
            source_snapshot: self.source_snapshot.clone(),
            style_context_captures: self.style_context_captures.clone(),
            namespace_bindings: Rc::clone(&self.namespace_bindings),
            implicit_openings: Rc::clone(&self.implicit_openings),
            retained_implicit_openings: Rc::new(RefCell::new(Vec::new())),
            retained_navigation_diagnostics: Rc::new(RefCell::new(Vec::new())),
            syntax_base_depth: self.syntax_base_depth,
            syntax_document: Rc::clone(&self.syntax_document),
        }
    }

    /// Selector probes use generated offsets, while keeping the live grammar
    /// environment and structural depth. Their source-relative bookkeeping must
    /// not publish offsets into the enclosing authored snapshot.
    pub(super) fn generated_selector_probe(&self, source: &str) -> Self {
        Self {
            parser_context: self.parser_context,
            depth: Rc::clone(&self.depth),
            source_snapshot: crate::source::CssSourceSnapshot::new(source),
            style_context_captures: self.style_context_captures.clone(),
            namespace_bindings: Rc::clone(&self.namespace_bindings),
            implicit_openings: Rc::new(unclosed_openings(source)),
            retained_implicit_openings: Rc::new(RefCell::new(Vec::new())),
            retained_navigation_diagnostics: Rc::new(RefCell::new(Vec::new())),
            syntax_base_depth: self.depth.get(),
            syntax_document: Rc::new(RefCell::new(None)),
        }
    }

    pub(super) fn syntax_document(
        &self,
        source: &str,
    ) -> Result<
        Rc<crate::parser::syntax_types::Document<'static>>,
        crate::component::CssComponentValueError,
    > {
        if let Some(document) = self.syntax_document.borrow().as_ref() {
            return Ok(Rc::clone(document));
        }
        let normalization_view = source_normalization_view(source, self.syntax_base_depth);
        let document = Rc::new(crate::syntax_consumption::recovery_source_document(
            normalization_view.into_owned(),
            &self.source_snapshot,
            self.syntax_base_depth,
        )?);
        *self.syntax_document.borrow_mut() = Some(Rc::clone(&document));
        Ok(document)
    }

    pub(super) fn pending_component_closures(&self) -> Vec<usize> {
        self.retained_implicit_openings.borrow().clone()
    }

    pub(super) fn source_snapshot(&self) -> &crate::source::CssSourceSnapshot {
        &self.source_snapshot
    }

    pub(super) fn source_position(&self, byte_offset: usize) -> crate::source::CssSourcePosition {
        crate::source::CssSourcePosition::from_byte_offset_in(
            self.source_snapshot.as_str(),
            byte_offset,
        )
    }

    pub(super) fn activate_namespace(
        &self,
        prefix: Option<CssNamespacePrefix>,
        name: CssNamespaceName,
    ) {
        self.namespace_bindings.borrow_mut().activate(prefix, name);
    }

    pub(super) fn has_active_namespace_binding(
        &self,
        prefix: Option<&CssNamespacePrefix>,
        name: &CssNamespaceName,
    ) -> bool {
        self.namespace_bindings
            .borrow()
            .has_active_binding(prefix, name)
    }

    pub(super) fn has_default_namespace(&self) -> bool {
        self.namespace_bindings.borrow().has_default()
    }

    pub(super) fn active_namespace_prefix(&self, prefix: &str) -> Option<CssNamespacePrefix> {
        self.namespace_bindings
            .borrow()
            .active_prefix(prefix)
            .cloned()
    }

    pub(super) fn record_style_context(&self, content_start: usize) {
        self.style_context_captures.record(content_start);
    }

    pub(super) fn enter_rule_block<'i>(
        &self,
        source: &str,
        input: &Parser<'i, '_>,
        enclosing_production: &'static str,
    ) -> Result<RecoveryDepthGuard, ParseError<'i, CssError>> {
        let opening_offset = input.position().byte_index().saturating_sub(1);
        self.enter(source, opening_offset, enclosing_production)
    }

    /// Enters the rule block whose opening token starts at `opening_offset`.
    pub(super) fn enter_rule_block_at<'i>(
        &self,
        source: &str,
        opening_offset: usize,
        enclosing_production: &'static str,
    ) -> Result<RecoveryDepthGuard, ParseError<'i, CssError>> {
        self.enter(source, opening_offset, enclosing_production)
    }

    /// Current entered structural block depth, including the active rule body.
    #[cfg(test)]
    pub(super) fn structural_depth(&self) -> u32 {
        self.depth.get()
    }

    pub(super) fn enter_component_block<'i>(
        &self,
        source: &str,
        input: &Parser<'i, '_>,
        enclosing_production: &'static str,
    ) -> Result<RecoveryDepthGuard, ParseError<'i, CssError>> {
        let opening_offset = input.position().byte_index().saturating_sub(1);
        self.enter(source, opening_offset, enclosing_production)
    }

    /// Enters the component block whose opening token ends at the byte
    /// `opening_offset`, its last byte.
    pub(super) fn enter_component_block_at<'i>(
        &self,
        source: &str,
        opening_offset: usize,
        enclosing_production: &'static str,
    ) -> Result<RecoveryDepthGuard, ParseError<'i, CssError>> {
        self.enter(source, opening_offset, enclosing_production)
    }

    pub(super) fn check_component_values<'i>(
        &self,
        source: &'i str,
        input: &Parser<'i, '_>,
        enclosing_production: &'static str,
    ) -> Result<Vec<usize>, ParseError<'i, CssError>> {
        self.check_component_values_from(
            source,
            input.position().byte_index(),
            enclosing_production,
        )
    }

    /// [`check_component_values`](Self::check_component_values) for a
    /// declaration value that starts at the byte `start`.
    pub(super) fn check_component_values_from<'i>(
        &self,
        source: &str,
        start: usize,
        enclosing_production: &'static str,
    ) -> Result<Vec<usize>, ParseError<'i, CssError>> {
        let end = scan_nested_tokens(
            source,
            start,
            self.depth.get(),
            enclosing_production,
            ScanBoundary::DeclarationValue,
        )?;
        Ok(self.component_openings_in(start..end))
    }

    pub(super) fn check_specialized_components<'i>(
        &self,
        source: &str,
        input: &Parser<'i, '_>,
        enclosing_production: &'static str,
    ) -> Result<Vec<usize>, ParseError<'i, CssError>> {
        self.check_specialized_components_from(
            source,
            input.position().byte_index(),
            enclosing_production,
        )
    }

    /// [`check_specialized_components`](Self::check_specialized_components) for
    /// a prelude that starts at the byte `start`.
    pub(super) fn check_specialized_components_from<'i>(
        &self,
        source: &str,
        start: usize,
        enclosing_production: &'static str,
    ) -> Result<Vec<usize>, ParseError<'i, CssError>> {
        let end = scan_nested_tokens(
            source,
            start,
            self.depth.get(),
            enclosing_production,
            ScanBoundary::SpecializedPrelude,
        )?;
        Ok(self.component_openings_in(start..end))
    }

    /// Checks the contents of a component block that start at the byte `start`.
    pub(super) fn check_component_contents_from<'i>(
        &self,
        source: &str,
        start: usize,
        enclosing_production: &'static str,
    ) -> Result<(), ParseError<'i, CssError>> {
        scan_nested_tokens(
            source,
            start,
            self.depth.get(),
            enclosing_production,
            ScanBoundary::ComponentContents,
        )
        .map(|_| ())
    }

    pub(super) fn check_comma_member_components<'i>(
        &self,
        source: &str,
        input: &Parser<'i, '_>,
        enclosing_production: &'static str,
    ) -> Result<Vec<usize>, ParseError<'i, CssError>> {
        self.check_comma_member_components_from(
            source,
            input.position().byte_index(),
            enclosing_production,
        )
    }

    /// [`check_comma_member_components`](Self::check_comma_member_components)
    /// for a member that starts at the byte `start`.
    pub(super) fn check_comma_member_components_from<'i>(
        &self,
        source: &str,
        start: usize,
        enclosing_production: &'static str,
    ) -> Result<Vec<usize>, ParseError<'i, CssError>> {
        let end = scan_nested_tokens(
            source,
            start,
            self.depth.get(),
            enclosing_production,
            ScanBoundary::CommaMember,
        )?;
        Ok(self.component_openings_in(start..end))
    }

    pub(super) fn component_openings_in(&self, range: std::ops::Range<usize>) -> Vec<usize> {
        // Keep the same opening identities as the structural scanner, including
        // escaped function names and URL tokens, within this grammar unit only.
        self.implicit_openings
            .iter()
            .rev()
            .copied()
            .filter(|opening| range.contains(opening))
            .collect()
    }

    pub(super) fn retain_component_closures(&self, openings: Vec<usize>) {
        let mut retained = self.retained_implicit_openings.borrow_mut();
        for opening in openings {
            if self.implicit_openings.contains(&opening) && !retained.contains(&opening) {
                retained.push(opening);
            }
        }
    }

    // Called only after a complete declaration survived its grammar boundary.
    pub(super) fn retain_navigation_diagnostic(
        &self,
        body: &crate::declaration::CssDeclarationBody,
    ) {
        let crate::declaration::CssDeclarationBody::Known(known) = body else {
            return;
        };
        let navigation = match known.property_value() {
            Some(crate::property::CssKnownPropertyValueRef::NavUp(value)) => value.value(),
            Some(crate::property::CssKnownPropertyValueRef::NavRight(value)) => value.value(),
            Some(crate::property::CssKnownPropertyValueRef::NavDown(value)) => value.value(),
            Some(crate::property::CssKnownPropertyValueRef::NavLeft(value)) => value.value(),
            _ => return,
        };
        let Some(target) = navigation.legacy_target() else {
            return;
        };
        let crate::component::CssValueOrigin::Parsed(origin) = target.origin() else {
            return;
        };
        let span = origin.span();
        let diagnostic = crate::diagnostic::CssRecoveryDiagnostic::new(
            crate::diagnostic::error::legacy_navigation_target(target.as_str(), span.start()),
            span,
            crate::diagnostic::CssRecoveryAction::RetainLegacyNavigationTarget,
        )
        .expect("target error belongs to original string span");
        self.retained_navigation_diagnostics
            .borrow_mut()
            .push(diagnostic);
    }

    pub(super) fn take_implicit_closure_diagnostics(
        &self,
        source: &str,
    ) -> Vec<crate::diagnostic::CssRecoveryDiagnostic> {
        let eof = CssSourcePosition::from_byte_offset_in(source, source.len());
        let Some(span) = crate::source::CssSourceSpan::new(eof, eof) else {
            return Vec::new();
        };
        let mut diagnostics: Vec<_> = self
            .retained_implicit_openings
            .borrow_mut()
            .drain(..)
            .filter_map(|opening| {
                crate::diagnostic::CssRecoveryDiagnostic::new(
                    crate::diagnostic::error::implicit_eof(source),
                    span,
                    crate::diagnostic::CssRecoveryAction::RetainWithImplicitClosure,
                )
                .map(|diagnostic| diagnostic.with_opening_owner(opening))
            })
            .collect();
        diagnostics.extend(self.retained_navigation_diagnostics.borrow_mut().drain(..));
        diagnostics
    }

    pub(super) fn check_failed_rule_block<'i>(
        &self,
        source: &'i str,
        input: &Parser<'i, '_>,
        enclosing_production: &'static str,
    ) -> Option<ParseError<'i, CssError>> {
        self.check_failed_rule_block_at(source, input.position().byte_index(), enclosing_production)
    }

    /// [`check_failed_rule_block`](Self::check_failed_rule_block) for a failed
    /// rule whose block contents start at the byte `content_start`.
    pub(super) fn check_failed_rule_block_at<'i>(
        &self,
        source: &str,
        content_start: usize,
        enclosing_production: &'static str,
    ) -> Option<ParseError<'i, CssError>> {
        let opening_offset = content_start.saturating_sub(1);
        if self.depth.get() >= STRUCTURAL_NESTING_LIMIT {
            return Some(nesting_limit(
                source,
                opening_offset,
                STRUCTURAL_NESTING_LIMIT,
                enclosing_production,
            ));
        }
        scan_nested_tokens(
            source,
            content_start,
            self.depth.get() + 1,
            enclosing_production,
            ScanBoundary::FailedCurlyBlock,
        )
        .err()
    }

    pub(super) fn check_entered_curly_contents<'i>(
        &self,
        source: &'i str,
        input: &Parser<'i, '_>,
        enclosing_production: &'static str,
    ) -> Result<(), ParseError<'i, CssError>> {
        scan_nested_tokens(
            source,
            input.position().byte_index(),
            self.depth.get(),
            enclosing_production,
            ScanBoundary::FailedCurlyBlock,
        )
        .map(|_| ())
    }

    fn enter<'i>(
        &self,
        source: &str,
        opening_offset: usize,
        enclosing_production: &'static str,
    ) -> Result<RecoveryDepthGuard, ParseError<'i, CssError>> {
        let depth = self.depth.get();
        if depth >= STRUCTURAL_NESTING_LIMIT {
            return Err(nesting_limit(
                source,
                opening_offset,
                STRUCTURAL_NESTING_LIMIT,
                enclosing_production,
            ));
        }
        self.depth.set(depth + 1);
        Ok(RecoveryDepthGuard {
            depth: Rc::clone(&self.depth),
            opening_offset,
            implicit_openings: Rc::clone(&self.implicit_openings),
            retained_implicit_openings: Rc::clone(&self.retained_implicit_openings),
            retained: false,
        })
    }
}

#[derive(Clone, Default)]
pub(crate) struct StyleContextCaptures {
    entries: Rc<RefCell<Vec<StyleContextCapture>>>,
    namespace_bindings: Rc<RefCell<CssNamespaceBindings>>,
}

struct StyleContextCapture {
    content_start: usize,
    parsed: bool,
}

impl StyleContextCaptures {
    pub(crate) fn with_namespaces(context: &crate::selector::CssNamespaceContext) -> Self {
        Self {
            entries: Rc::new(RefCell::new(Vec::new())),
            namespace_bindings: Rc::new(RefCell::new(context.0.clone())),
        }
    }

    pub(super) fn register(&self, content_start: usize) {
        let mut entries = self.entries.borrow_mut();
        if entries
            .iter()
            .all(|entry| entry.content_start != content_start)
        {
            entries.push(StyleContextCapture {
                content_start,
                parsed: false,
            });
        }
    }

    fn record(&self, content_start: usize) {
        if let Some(entry) = self
            .entries
            .borrow_mut()
            .iter_mut()
            .find(|entry| entry.content_start == content_start)
        {
            entry.parsed = true;
        }
    }

    pub(super) fn contains_parsed(&self, content_start: usize) -> bool {
        self.entries
            .borrow()
            .iter()
            .any(|entry| entry.content_start == content_start && entry.parsed)
    }
}

// Execution batching is independent of the authored 256-level admission limit.
const STRUCTURAL_PARSE_CHUNK: usize = 32;

// Test-only work counter: source tokens walked by structural preflight.
#[cfg(test)]
thread_local! {
    pub(super) static PREFLIGHT_TOKENS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) struct StructuralPreflight {
    pub(super) unit_start: usize,
    pub(super) unit_end: usize,
    pub(super) parents: Vec<StructuralParent>,
    pub(super) style_context_starts: Vec<usize>,
    pub(super) style_ancestry_starts: Vec<usize>,
    pub(super) parent_depth: u32,
    pub(super) outcome: StructuralPreflightOutcome,
}

pub(super) enum StructuralPreflightOutcome {
    Split,
    NestingLimit {
        opening_offset: usize,
        enclosing_production: &'static str,
    },
}

#[derive(Clone, Copy)]
pub(super) struct StructuralParent {
    pub(super) start: usize,
    pub(super) kind: GroupKind,
}

#[derive(Clone, Copy)]
pub(super) enum GroupKind {
    Layer,
    Media,
    Supports,
    Container,
    When,
    Else,
    Scope,
    Style,
    Component,
    // A genuine root brace with body semantics supplied by its direct front.
    FragmentBody,
    Other,
}

impl GroupKind {
    const AT_RULES: [Self; 7] = [
        Self::Layer,
        Self::Media,
        Self::Supports,
        Self::Container,
        Self::When,
        Self::Else,
        Self::Scope,
    ];

    const fn at_rule_name(self) -> Option<&'static str> {
        match self {
            Self::Layer => Some("layer"),
            Self::Media => Some("media"),
            Self::Supports => Some("supports"),
            Self::Container => Some("container"),
            Self::When => Some("when"),
            Self::Else => Some("else"),
            Self::Scope => Some("scope"),
            Self::Style | Self::Component | Self::FragmentBody | Self::Other => None,
        }
    }

    // A group at-rule's production comes from the parser's one at-rule table.
    fn production(self) -> &'static str {
        match self {
            Self::Style => "baseline.rule.style",
            Self::Component => "css.declaration",
            Self::FragmentBody => "css.block",
            Self::Other => "css.qualified-rule",
            group => super::rule_list::at_rule_production(
                group.at_rule_name().expect("a group at-rule kind"),
            )
            .expect("every group at-rule is admitted"),
        }
    }

    fn can_split(self) -> bool {
        !matches!(self, Self::Other)
    }
}

struct StructuralFrame {
    block: BlockKind,
    group: Option<(usize, GroupKind)>,
}

#[derive(Clone)]
struct StructuralGroup {
    start: usize,
    kind: GroupKind,
    style_context_starts: Vec<usize>,
    style_ancestry_starts: Vec<usize>,
}

// The unit boundaries of one structural list, as its selected Syntax 3
// (2026 CRD) consumer chooses them.
#[derive(Clone, Copy)]
pub(super) enum StructuralListContext {
    /// Consume a stylesheet's contents (§5.5.1): CDO/CDC are discarded.
    Stylesheet,
    /// Parse a rule (Syntax 3 (2026 CRD) §5.4.6): one unit, without CDO/CDC discard or a
    /// qualified-rule stop token.
    OneRule,
    /// Consume a block's contents (§5.5.5), used for every curly block:
    /// a `;` ends or precedes every unit.
    BlockContents,
}

impl StructuralListContext {
    fn ignores_prefix(self, token: &Token<'_>, at_unit_start: bool) -> bool {
        match token {
            Token::WhiteSpace(_) | Token::Comment(_) => true,
            Token::CDO | Token::CDC => at_unit_start && matches!(self, Self::Stylesheet),
            _ => false,
        }
    }

    fn semicolon_ends_unit(self, unit: Option<StructuralUnit>) -> bool {
        matches!(self, Self::BlockContents)
            || unit.is_some_and(|unit| {
                crate::syntax_consumption::semicolon_terminates_rule(unit.dispatch)
            })
    }
}

#[derive(Clone, Copy)]
struct StructuralUnit {
    start: usize,
    dispatch: crate::syntax_consumption::GenericRuleDispatch,
}

pub(super) fn preflight_structural_nesting(
    source: &str,
    base_depth: u32,
    root_style_context: bool,
    root_style_ancestor: bool,
    root_list_context: StructuralListContext,
    fragment_body: bool,
) -> Vec<StructuralPreflight> {
    // Restarting cssparser at each verified token boundary exposes opening and
    // closing tokens without calling `parse_nested_block`; comments, strings,
    // URLs, and escapes therefore keep cssparser's token semantics while this
    // walk keeps its own heap-backed block stack.
    let mut offset = 0;
    let mut frames: Vec<StructuralFrame> = Vec::new();
    let mut groups: Vec<StructuralGroup> = Vec::new();
    let mut unit_starts: Vec<Option<StructuralUnit>> = vec![None];
    // Every bounded unit of this source, in source order. A unit's own deeper
    // chains belong to its isolated replay, so none is collected inside it.
    let mut targets = Vec::new();
    let mut target: Option<StructuralPreflight> = None;
    let mut target_group_depth = 0;

    while let Some((token_start, token_end, token)) = next_source_token(source, offset) {
        #[cfg(test)]
        PREFLIGHT_TOKENS.with(|tokens| tokens.set(tokens.get() + 1));
        offset = token_end;
        if let Some(closing) = closing_block(&token)
            && let Some(frame) = frames.pop_if(|frame| frame.block == closing)
        {
            if frame.group.is_some() {
                groups.pop();
                unit_starts.pop();
                if let Some(parent_start) = unit_starts.last_mut() {
                    *parent_start = None;
                }
                if target.is_some() && groups.len() < target_group_depth {
                    let mut completed = target.take().expect("target exists");
                    completed.unit_end = token_end;
                    targets.push(completed);
                }
            }
            continue;
        }
        // An unmatched closer is an ordinary generic prelude component.

        let directly_in_group = frames.last().is_none_or(|frame| frame.group.is_some());
        // Syntax 3 (2026 CRD) §5.5.4 consumes every curly block, including a
        // body fragment's genuine root, with consume a block's contents.
        let list_context = if groups.is_empty() {
            root_list_context
        } else {
            StructuralListContext::BlockContents
        };
        let unit = unit_starts.last().copied().flatten();
        if directly_in_group && list_context.ignores_prefix(&token, unit.is_none()) {
            continue;
        }
        if directly_in_group
            && matches!(token, Token::Semicolon)
            && list_context.semicolon_ends_unit(unit)
        {
            if let Some(unit_start) = unit_starts.last_mut() {
                *unit_start = None;
            }
            continue;
        }
        if directly_in_group
            && unit_starts
                .last()
                .is_some_and(|unit_start| unit_start.is_none())
            && let Some(unit_start) = unit_starts.last_mut()
        {
            *unit_start = Some(StructuralUnit {
                start: token_start,
                dispatch: crate::syntax_consumption::rule_dispatch(
                    crate::syntax_consumption::native_token_kind(&token),
                ),
            });
        }

        let Some(opening) = opening_block(&token) else {
            continue;
        };
        if opening != BlockKind::Curly || !directly_in_group {
            frames.push(StructuralFrame {
                block: opening,
                group: None,
            });
            continue;
        }

        let unit_start = unit_starts
            .last()
            .and_then(|unit_start| unit_start.map(|unit| unit.start))
            .unwrap_or(token_start);
        // Only an actual prelude-free root brace can be a body
        // fragment. Existing rule/style callers have no override. Descendants
        // retain classification from their genuine authored preludes.
        let is_fragment_root =
            fragment_body && groups.is_empty() && frames.is_empty() && unit_start == token_start;
        let group = if is_fragment_root {
            GroupKind::FragmentBody
        } else {
            group_kind(source, unit_start, token_start)
        };
        if matches!(group, GroupKind::Component) {
            frames.push(StructuralFrame {
                block: opening,
                group: None,
            });
            continue;
        }
        let global_depth = base_depth.saturating_add(groups.len() as u32 + 1);
        let split_chain =
            group.can_split() && groups.iter().all(|ancestor| ancestor.kind.can_split());
        let style_context_starts = groups
            .last()
            .map(|parent| parent.style_context_starts.clone())
            .unwrap_or_else(|| root_style_context.then_some(0).into_iter().collect());
        let style_ancestry_starts = groups
            .last()
            .map(|parent| parent.style_ancestry_starts.clone())
            .unwrap_or_else(|| root_style_ancestor.then_some(0).into_iter().collect());
        if target.is_none() && split_chain {
            let outcome = if global_depth > STRUCTURAL_NESTING_LIMIT {
                Some(StructuralPreflightOutcome::NestingLimit {
                    opening_offset: token_start,
                    enclosing_production: group.production(),
                })
            } else if groups.len() + 1 == STRUCTURAL_PARSE_CHUNK {
                Some(StructuralPreflightOutcome::Split)
            } else {
                None
            };
            if let Some(outcome) = outcome {
                target = Some(StructuralPreflight {
                    unit_start,
                    unit_end: source.len(),
                    parents: groups
                        .iter()
                        .map(|parent| StructuralParent {
                            start: parent.start,
                            kind: parent.kind,
                        })
                        .collect(),
                    style_context_starts: style_context_starts.clone(),
                    style_ancestry_starts: style_ancestry_starts.clone(),
                    parent_depth: global_depth.saturating_sub(1),
                    outcome,
                });
                target_group_depth = groups.len() + 1;
            }
        }

        let child_style_context_starts = match group {
            GroupKind::Style => {
                let mut starts = style_context_starts;
                starts.push(token_end);
                starts
            }
            GroupKind::Layer
            | GroupKind::Media
            | GroupKind::Supports
            | GroupKind::Container
            | GroupKind::When
            | GroupKind::Else
                if !style_context_starts.is_empty() =>
            {
                let mut starts = style_context_starts;
                starts.push(token_end);
                starts
            }
            GroupKind::Layer
            | GroupKind::Media
            | GroupKind::Supports
            | GroupKind::Container
            | GroupKind::When
            | GroupKind::Else
            | GroupKind::Scope
            | GroupKind::Component
            | GroupKind::FragmentBody
            | GroupKind::Other => Vec::new(),
        };
        // Scope changes the child grammar but does not erase style ancestry.
        // Keep ancestry captures separate from the grammar-selection captures.
        let child_style_ancestry_starts = match group {
            GroupKind::Style => {
                let mut starts = style_ancestry_starts;
                starts.push(token_end);
                starts
            }
            GroupKind::Layer
            | GroupKind::Media
            | GroupKind::Supports
            | GroupKind::Container
            | GroupKind::When
            | GroupKind::Else
            | GroupKind::Scope
            | GroupKind::FragmentBody
                if !style_ancestry_starts.is_empty() =>
            {
                let mut starts = style_ancestry_starts;
                starts.push(token_end);
                starts
            }
            _ => Vec::new(),
        };
        groups.push(StructuralGroup {
            start: unit_start,
            kind: group,
            style_context_starts: child_style_context_starts,
            style_ancestry_starts: child_style_ancestry_starts,
        });
        unit_starts.push(None);
        frames.push(StructuralFrame {
            block: opening,
            group: Some((unit_start, group)),
        });
    }

    // A unit still open at EOF ends there.
    targets.extend(target);
    targets
}

fn group_kind(source: &str, unit_start: usize, opening_offset: usize) -> GroupKind {
    let Some(prelude) = source.get(unit_start..opening_offset) else {
        return GroupKind::Other;
    };
    let Some((_, _, Token::AtKeyword(name))) = next_source_token(source, unit_start) else {
        return if looks_like_custom_declaration(prelude.trim_start()) {
            GroupKind::Component
        } else {
            GroupKind::Style
        };
    };
    GroupKind::AT_RULES
        .into_iter()
        .find(|kind| {
            kind.at_rule_name()
                .is_some_and(|known| known.eq_ignore_ascii_case(&name))
        })
        .unwrap_or(GroupKind::Other)
}

fn looks_like_custom_declaration(prelude: &str) -> bool {
    let mut input = ParserInput::new(prelude);
    let mut parser = Parser::new(&mut input);
    parser
        .expect_ident_cloned()
        .ok()
        .is_some_and(|name| name.starts_with("--") && parser.expect_colon().is_ok())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BlockKind {
    Parenthesis,
    Square,
    Curly,
}

#[derive(Clone, Copy)]
enum ScanBoundary {
    DeclarationValue,
    FailedCurlyBlock,
    SpecializedPrelude,
    CommaMember,
    ComponentContents,
}

// Validate nesting and return the exclusive grammar-unit boundary. Token starts
// are not structural opening identities: URL tokens consume their own contents,
// and function tokens end at their actual opening parenthesis.
fn scan_nested_tokens<'i>(
    source: &str,
    start: usize,
    base_depth: u32,
    enclosing_production: &'static str,
    boundary: ScanBoundary,
) -> Result<usize, ParseError<'i, CssError>> {
    let mut offset = start.min(source.len());
    let mut blocks: Vec<BlockKind> = Vec::new();
    while let Some((token_start, token_end, token)) = next_source_token(source, offset) {
        offset = token_end;
        if let Some(closing) = closing_block(&token) {
            if blocks.last().is_some_and(|kind| *kind == closing) {
                blocks.pop();
                continue;
            }
            // The caller has already entered this component. Its closing
            // delimiter ends the scan; following sibling components have their
            // own depth context and must not be counted inside this one.
            if blocks.is_empty() && matches!(boundary, ScanBoundary::ComponentContents) {
                return Ok(token_start);
            }
            if blocks.is_empty()
                && matches!(boundary, ScanBoundary::FailedCurlyBlock)
                && closing == BlockKind::Curly
            {
                return Ok(token_start);
            }
            if blocks.is_empty()
                && matches!(boundary, ScanBoundary::DeclarationValue)
                && closing == BlockKind::Curly
            {
                return Ok(token_start);
            }
            continue;
        }
        if blocks.is_empty()
            && matches!(boundary, ScanBoundary::DeclarationValue)
            && matches!(token, Token::Semicolon)
        {
            return Ok(token_start);
        }
        if blocks.is_empty()
            && matches!(
                boundary,
                ScanBoundary::SpecializedPrelude | ScanBoundary::CommaMember
            )
            && (matches!(token, Token::Semicolon | Token::CurlyBracketBlock)
                || matches!(boundary, ScanBoundary::CommaMember) && matches!(token, Token::Comma))
        {
            return Ok(token_start);
        }
        if let Some(opening) = opening_block(&token) {
            let nested_depth = base_depth.saturating_add(blocks.len() as u32);
            if nested_depth >= STRUCTURAL_NESTING_LIMIT {
                return Err(nesting_limit(
                    source,
                    token_start,
                    STRUCTURAL_NESTING_LIMIT,
                    enclosing_production,
                ));
            }
            blocks.push(opening);
        }
    }
    Ok(source.len())
}

fn unclosed_openings(source: &str) -> Vec<usize> {
    scan_delimiters(source, 0, StructuralListContext::Stylesheet).unclosed
}

pub(super) fn next_source_token<'i>(
    source: &'i str,
    offset: usize,
) -> Option<(usize, usize, Token<'i>)> {
    crate::tokenization::next_source_token(source, offset)
}

fn opening_block(token: &Token<'_>) -> Option<BlockKind> {
    match token {
        Token::Function(_) | Token::ParenthesisBlock => Some(BlockKind::Parenthesis),
        Token::SquareBracketBlock => Some(BlockKind::Square),
        Token::CurlyBracketBlock => Some(BlockKind::Curly),
        _ => None,
    }
}

fn closing_block(token: &Token<'_>) -> Option<BlockKind> {
    match token {
        Token::CloseParenthesis => Some(BlockKind::Parenthesis),
        Token::CloseSquareBracket => Some(BlockKind::Square),
        Token::CloseCurlyBracket => Some(BlockKind::Curly),
        _ => None,
    }
}

pub(super) struct RecoveryDepthGuard {
    depth: Rc<Cell<u32>>,
    opening_offset: usize,
    implicit_openings: Rc<Vec<usize>>,
    retained_implicit_openings: Rc<RefCell<Vec<usize>>>,
    retained: bool,
}

impl RecoveryDepthGuard {
    pub(super) fn retain(&mut self) {
        self.retained = true;
    }
}

impl Drop for RecoveryDepthGuard {
    fn drop(&mut self) {
        if self.retained && self.implicit_openings.contains(&self.opening_offset) {
            let mut retained = self.retained_implicit_openings.borrow_mut();
            if !retained.contains(&self.opening_offset) {
                retained.push(self.opening_offset);
            }
        }
        self.depth.set(self.depth.get().saturating_sub(1));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryLoopOutcome {
    Retained,
    Advanced,
    Terminated,
}

/// One iteration's byte-progress witness for a bounded recovery coordinator.
pub(crate) struct RecoveryProgress {
    start_byte: usize,
}

impl RecoveryProgress {
    pub(super) fn record(input: &Parser<'_, '_>) -> Self {
        Self {
            start_byte: input.position().byte_index(),
        }
    }

    pub(super) fn finish(self, input: &mut Parser<'_, '_>, retained: bool) -> RecoveryLoopOutcome {
        if retained {
            return if input.position().byte_index() > self.start_byte {
                RecoveryLoopOutcome::Retained
            } else {
                RecoveryLoopOutcome::Terminated
            };
        }
        if input.position().byte_index() > self.start_byte {
            return RecoveryLoopOutcome::Advanced;
        }
        if input.next_including_whitespace_and_comments().is_ok()
            && input.position().byte_index() > self.start_byte
        {
            RecoveryLoopOutcome::Advanced
        } else {
            RecoveryLoopOutcome::Terminated
        }
    }
}

pub(crate) fn recovery_action_for_error(
    error: &ParseError<'_, CssError>,
    ordinary: crate::diagnostic::CssRecoveryAction,
) -> crate::diagnostic::CssRecoveryAction {
    if is_nesting_limit_error(error) {
        crate::diagnostic::CssRecoveryAction::StopAtNestingLimit
    } else {
        ordinary
    }
}

pub(super) fn first_non_trivia_position(
    source: &str,
    member_start: usize,
    member_end: usize,
) -> CssSourcePosition {
    let bounded_start = member_start.min(source.len());
    let bounded_end = member_end.min(source.len()).max(bounded_start);
    let Some(member) = source.get(bounded_start..bounded_end) else {
        return CssSourcePosition::from_byte_offset_in(source, bounded_end);
    };
    let mut input = ParserInput::new(member);
    let mut parser = Parser::new(&mut input);
    loop {
        let token_start = parser.position().byte_index();
        match parser.next_including_whitespace_and_comments() {
            Ok(Token::WhiteSpace(_) | Token::Comment(_)) => {}
            Ok(_) => {
                return CssSourcePosition::from_byte_offset_in(
                    source,
                    bounded_start.saturating_add(token_start),
                );
            }
            Err(_) => return CssSourcePosition::from_byte_offset_in(source, bounded_end),
        }
    }
}

pub(super) fn comma_member_span(
    source: &str,
    member_start: usize,
    member_end: usize,
    following_comma: Option<(usize, usize)>,
    preceding_comma: Option<(usize, usize)>,
) -> Option<crate::source::CssSourceSpan> {
    let (span_start, span_end) = if member_start < member_end {
        (member_start, member_end)
    } else if let Some(comma) = following_comma {
        comma
    } else if let Some(comma) = preceding_comma {
        comma
    } else {
        (member_start, member_end)
    };
    crate::source::CssSourceSpan::new(
        CssSourcePosition::from_byte_offset_in(source, span_start),
        CssSourcePosition::from_byte_offset_in(source, span_end),
    )
}

#[cfg(test)]
mod tests {
    use cssparser::{Parser, ParserInput};

    use super::{RecoveryLoopOutcome, RecoveryProgress, unclosed_openings};

    /// The structural scan and the document agree on which blocks are too deep:
    /// the `{}` blocks the scan found past the ceiling are exactly the ones the
    /// document denies on its own, with mixed paren, square and curly nesting
    /// around the ceiling and a nonzero base depth, and the document builds
    /// whatever the scan masked.
    #[test]
    fn the_scan_and_the_document_agree_on_which_blocks_are_too_deep() {
        use super::{BlockKind, scan_delimiters, source_normalization_view};
        use crate::STRUCTURAL_NESTING_LIMIT;
        use crate::source::CssSourceSnapshot;
        use crate::syntax_consumption::recovery_source_document;
        use surgeist_css_parser::document::Component;

        let ceiling = STRUCTURAL_NESTING_LIMIT as usize;
        let patterns = ["(", "{", "[", "({", "{(", "[{(", "{{(", "f("];
        let finals = ["(", "{", "["];
        let mut cases = 0;
        for base in [0_u32, 1, 200, 255] {
            let room = ceiling - base as usize;
            for pattern in patterns {
                for around in [room - 1, room, room + 1, room + 3] {
                    for last in finals {
                        for close in [true, false] {
                            let mut opens = Vec::new();
                            let mut text = String::new();
                            let mut chars = pattern.chars().cycle();
                            while opens.len() < around {
                                let next = chars.next().unwrap();
                                if next == 'f' {
                                    text.push('f');
                                    continue;
                                }
                                text.push(next);
                                opens.push(next);
                            }
                            text.push_str(last);
                            opens.push(last.chars().next().unwrap());
                            text.push_str("a ");
                            if close {
                                for open in opens.iter().rev() {
                                    text.push(match open {
                                        '(' => ')',
                                        '{' => '}',
                                        _ => ']',
                                    });
                                }
                                text.push_str(" @layer kept;");
                            }
                            let scan = scan_delimiters(
                                &text,
                                base,
                                super::StructuralListContext::Stylesheet,
                            );
                            let expected: Vec<_> = scan
                                .over_limit_components
                                .iter()
                                .filter(|component| component.block == BlockKind::Curly)
                                .map(|component| component.range.clone())
                                .collect();
                            let view = source_normalization_view(&text, base).into_owned();
                            let snapshot = CssSourceSnapshot::new(&text);
                            let document = recovery_source_document(view, &snapshot, base)
                                .unwrap_or_else(|error| {
                                    panic!("{pattern:?} {around} {last:?} base {base}: {error:?}")
                                });
                            let denied: Vec<_> = document
                                .nodes()
                                .filter_map(|node| match document.component(node) {
                                    Component::Denied {
                                        range: Some(range), ..
                                    } => Some(range.start.value()..range.end.value()),
                                    _ => None,
                                })
                                .collect();
                            assert_eq!(
                                denied, expected,
                                "{pattern:?} {around} {last:?} close={close} base {base}"
                            );
                            // The document counts the base depth it starts at.
                            assert!(
                                document.metrics().maximum_depth() as usize <= base as usize + room,
                                "{pattern:?} {around} {last:?} base {base}"
                            );
                            cases += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(cases, 4 * 8 * 4 * 3 * 2);
    }

    #[test]
    fn denied_curly_keeps_actual_units_and_charges_only_its_opaque_boundary() {
        use crate::syntax_consumption::{
            self as syntax, DocumentExt, GenericRule, RuleTermination,
        };
        use surgeist_css_parser::document::Component;
        let source = "@page {😀\r\nnot parsed}@layer kept;";
        let state =
            super::RecoveryState::at_depth(source, 256, super::StyleContextCaptures::default());
        let document = state.syntax_document(source).unwrap();
        let selected = syntax::consume_rules(
            &mut document.cursor(document.root_list()),
            syntax::RuleListSyntax::Stylesheet,
        );
        let [Ok(GenericRule::At(page)), Ok(GenericRule::At(layer))] = selected.as_slice() else {
            panic!("denied Page and independent same-parent Layer units");
        };
        assert!(matches!(page.termination, RuleTermination::Block));
        assert!(page.fault.is_none());
        assert!(matches!(layer.termination, RuleTermination::Semicolon(_)));
        assert_eq!(
            document.kind(layer.name),
            syntax::TokenKind::AtKeyword("layer")
        );
        let denied = page.block.unwrap();
        assert!(matches!(
            document.component(denied),
            Component::Denied { .. }
        ));
        assert!(document.children(denied).is_none());
        let opening = source.find('{').unwrap();
        let next = source.find("@layer").unwrap();
        assert_eq!(
            document
                .range(denied)
                .map(|range| range.start.value()..range.end.value()),
            Some(opening..next)
        );
        assert_eq!(
            document
                .boundary(page.range.list, page.range.end)
                .offset
                .unwrap(),
            next
        );
        assert_eq!(document.metrics().components(), 7);
        // The tree counts depth from its base (the state's depth); the denied
        // block adds none.
        assert_eq!(document.metrics().maximum_depth(), 256);
        assert_eq!(document.metrics().spelling_bytes(), source.len());
        let crate::component::CssValueOrigin::Parsed(origin) = &document.origin(denied) else {
            panic!("actual original denied opening");
        };
        assert!(origin.source().same_snapshot(state.source_snapshot()));
        assert_eq!(origin.span().start(), state.source_position(opening));
        assert_eq!(origin.span().end(), state.source_position(opening + 1));
        assert_eq!(origin.span().start().byte_offset().value(), 6);
        assert_eq!(origin.span().start().line().value(), 0);
        assert_eq!(origin.span().start().column().value(), 6);
        assert_eq!(origin.span().end().byte_offset().value(), 7);
        assert_eq!(origin.span().end().column().value(), 7);
        let crate::component::CssValueOrigin::Parsed(layer_origin) = &document.origin(layer.name)
        else {
            panic!("original Layer coordinates after Unicode and CRLF");
        };
        assert_eq!(layer_origin.span().start(), state.source_position(next));
        assert_eq!(layer_origin.span().start().byte_offset().value(), 24);
        assert_eq!(layer_origin.span().start().line().value(), 1);
        assert_eq!(layer_origin.span().start().column().value(), 11);
        let denied_origin = document.origin(denied);
        let promoted = crate::component::values::parse::promote_nodes(
            &document,
            &[denied],
            crate::component::CssComponentValueLimits::default(),
        )
        .unwrap_err();
        assert_eq!(
            promoted.kind(),
            crate::component::values::CssComponentValueErrorKind::NestingLimit
        );
        assert_eq!(promoted.origin(), &denied_origin);
        let direct = syntax::source_document(source, state.source_snapshot(), 256)
            .expect_err("direct source has no recovery admission");
        assert_eq!(
            direct.kind(),
            crate::component::values::CssComponentValueErrorKind::NestingLimit
        );
        assert_eq!(direct.origin(), &denied_origin);
    }

    #[test]
    fn denied_curly_ends_qualified_units_and_preserves_true_statement_eof() {
        use crate::syntax_consumption::{
            self as syntax, DocumentExt, GenericRule, RuleTermination,
        };
        let source = "bad{}@layer kept;";
        let state =
            super::RecoveryState::at_depth(source, 256, super::StyleContextCaptures::default());
        let document = state.syntax_document(source).unwrap();
        let selected = syntax::consume_rules(
            &mut document.cursor(document.root_list()),
            syntax::RuleListSyntax::Stylesheet,
        );
        let [
            Ok(GenericRule::Qualified(qualified)),
            Ok(GenericRule::At(layer)),
        ] = selected.as_slice()
        else {
            panic!("denied qualified body and independent Layer unit");
        };
        assert_eq!(
            document
                .range(qualified.block)
                .map(|range| range.start.value()..range.end.value()),
            Some(3..5)
        );
        assert_eq!(
            document
                .boundary(qualified.range.list, qualified.range.end)
                .offset
                .unwrap(),
            5
        );
        assert!(matches!(layer.termination, RuleTermination::Semicolon(_)));

        for source in ["@page{unclosed payload", "@layer kept"] {
            let state =
                super::RecoveryState::at_depth(source, 256, super::StyleContextCaptures::default());
            let document = state.syntax_document(source).unwrap();
            let selected = syntax::consume_rules(
                &mut document.cursor(document.root_list()),
                syntax::RuleListSyntax::Stylesheet,
            );
            let [Ok(GenericRule::At(rule))] = selected.as_slice() else {
                panic!("one selected unit");
            };
            assert_eq!(document.metrics().spelling_bytes(), source.len());
            if source.starts_with("@page") {
                assert!(matches!(rule.termination, RuleTermination::Block));
                assert!(rule.fault.is_none());
                assert_eq!(
                    document
                        .range(rule.block.unwrap())
                        .map(|range| range.start.value()..range.end.value()),
                    Some(5..source.len())
                );
            } else {
                let RuleTermination::EndOfInput(at) = &rule.termination else {
                    panic!("actual source EOF");
                };
                assert_eq!(at.offset, Some(source.len()));
                assert_eq!(
                    rule.fault.as_ref().unwrap().kind,
                    syntax::GenericFaultKind::AtRuleEndOfInput
                );
            }
        }
    }

    #[test]
    fn generated_selector_probe_shares_environment_but_owns_source_offsets() {
        let source = "original(";
        let captures = super::StyleContextCaptures::default();
        captures.register(9);
        let context = crate::CssParserContext::new(crate::CssParserMode::Quirks)
            .with_svg_glyph_orientation_vertical();
        let state = super::RecoveryState::at_depth(source, 7, captures.clone())
            .with_parser_context(context);
        state.retain_component_closures(vec![8]);
        let prefix = crate::rule::CssNamespacePrefix::try_new("Svg").unwrap();
        let name = crate::rule::CssNamespaceName::new("urn:svg");
        state.activate_namespace(Some(prefix.clone()), name.clone());
        let probe = state.generated_selector_probe(":is(.A)");
        assert_eq!(probe.parser_context(), context);
        assert_eq!(probe.source_snapshot().as_str(), ":is(.A)");
        assert_eq!(state.source_snapshot().as_str(), source);
        assert_eq!(probe.active_namespace_prefix("Svg"), Some(prefix));
        assert!(probe.active_namespace_prefix("svg").is_none());
        probe.activate_namespace(None, name);
        assert!(state.has_default_namespace());
        probe.record_style_context(9);
        assert!(captures.contains_parsed(9));
        let mut input = ParserInput::new("(");
        let mut parser = Parser::new(&mut input);
        parser.next().unwrap();
        {
            let _guard = probe
                .enter_component_block("(", &parser, "baseline.selector.complex")
                .unwrap();
            assert_eq!(state.structural_depth(), 8);
        }
        assert_eq!(state.structural_depth(), 7);
        assert!(probe.pending_component_closures().is_empty());
        assert_eq!(state.pending_component_closures(), [8]);
        assert!(
            probe
                .take_implicit_closure_diagnostics(":is(.A)")
                .is_empty()
        );
        let diagnostics = state.take_implicit_closure_diagnostics(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].error().position().byte_offset().value(),
            source.len()
        );
    }

    #[test]
    fn implicit_closure_scan_ignores_delimiters_in_strings_and_comments() {
        let source = ".x{--v:f(\") }\"/* ] } */x";

        assert_eq!(unclosed_openings(source), [2, 8]);
    }

    #[test]
    fn structural_recovery_zero_progress_failure_advances_one_token() {
        let mut input = ParserInput::new(";later");
        let mut parser = Parser::new(&mut input);
        let progress = RecoveryProgress::record(&parser);

        assert_eq!(
            progress.finish(&mut parser, false),
            RecoveryLoopOutcome::Advanced
        );
        assert_eq!(parser.position().byte_index(), 1);
    }

    #[test]
    fn structural_recovery_zero_progress_at_bounded_end_terminates() {
        let mut input = ParserInput::new("");
        let mut parser = Parser::new(&mut input);
        let progress = RecoveryProgress::record(&parser);

        assert_eq!(
            progress.finish(&mut parser, false),
            RecoveryLoopOutcome::Terminated
        );
    }
}

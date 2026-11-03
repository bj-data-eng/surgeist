use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::diagnostic::error::GrammarFailure;
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::{Offset, Token};

use crate::diagnostic::error::{is_nesting_limit_error, nesting_limit};
use crate::rule::{CssNamespaceName, CssNamespacePrefix};
use crate::source::CssSourcePosition;

use super::CssNamespaceBindings;
use super::structure::{self, ScanBoundary};

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
///
/// The errors are those the lexer records for the complete source
/// (Syntax 3 section 4.3), the same events a token tree lists: a backslash
/// before a newline (section 4.3.1), a backslash that ends the text (section
/// 4.3.7) and a comment without its closing `*/` (section 4.3.2). Strings and URLs consume their own payload,
/// and their existing grammar and end-of-input diagnostics describe that
/// recovery.
pub(crate) fn finish_report<T>(
    source: &str,
    report: crate::CssParseReport<T>,
) -> crate::CssParseReport<T> {
    let (syntax, mut diagnostics) = report.into_parts();
    for diagnostic in &mut diagnostics {
        diagnostic.clear_opening_owner();
    }
    {
        let options = Options::default();
        let mut lexer = Lexer::new(source, &options);
        // A backslash that ends the text belongs to the last token, which is
        // a name for the escape to be a fault of the text; a string or a URL
        // consumes it, and its own diagnostics describe the recovery.
        let mut ends_in_name = false;
        while let Some(lexeme) = lexer.next_lexeme() {
            ends_in_name = matches!(
                lexeme.token(),
                Token::Ident(_)
                    | Token::AtKeyword(_)
                    | Token::Hash { .. }
                    | Token::Dimension { .. }
            );
        }
        // The errors come in source order, so each position is found from
        // the one before it.
        let mut cursor = CssSourcePosition::START;
        let mut eof = None;
        for error in lexer.errors() {
            let range = error.range();
            let (start, end) = (range.start.value(), range.end.value());
            let at = cursor.resolved_near(source, start);
            let after = at.resolved_near(source, end);
            cursor = after;
            if error.is_escaped_newline() {
                diagnostics.push(escape_diagnostic(
                    (at, after),
                    crate::diagnostic::CssEscapeFailure::Newline,
                ));
            } else if error.is_escaped_eof() && ends_in_name {
                diagnostics.push(escape_diagnostic(
                    (at, after),
                    crate::diagnostic::CssEscapeFailure::EndOfInput,
                ));
            } else if error.is_unterminated_comment() {
                let end_of_input =
                    *eof.get_or_insert_with(|| after.resolved_near(source, source.len()));
                if let Some(span) = crate::source::CssSourceSpan::new(at, end_of_input)
                    && let Some(diagnostic) = crate::diagnostic::CssRecoveryDiagnostic::new(
                        crate::diagnostic::error::implicit_eof(end_of_input),
                        span,
                        crate::diagnostic::CssRecoveryAction::IgnoreUnterminatedComment,
                    )
                {
                    diagnostics.push(diagnostic);
                }
            }
        }
    }
    crate::CssParseReport::new(syntax, diagnostics)
}

fn escape_diagnostic(
    (start, end): (CssSourcePosition, CssSourcePosition),
    detail: crate::diagnostic::CssEscapeFailure,
) -> crate::diagnostic::CssRecoveryDiagnostic {
    let responsible = match detail {
        crate::diagnostic::CssEscapeFailure::Newline => start,
        crate::diagnostic::CssEscapeFailure::EndOfInput => end,
    };
    let span = crate::source::CssSourceSpan::new(start, end)
        .expect("consumed escape ranges are ordered UTF-8 boundaries");
    crate::diagnostic::CssRecoveryDiagnostic::new(
        crate::diagnostic::error::escape_parse_error(responsible, detail),
        span,
        crate::diagnostic::CssRecoveryAction::RecoverEscape,
    )
    .expect("escape error is anchored within its consumed backslash range")
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
    /// The text whose tokens are read, when it is not the text of the snapshot:
    /// a view of it with a unit masked.
    text: Option<Rc<str>>,
    namespaces: SharedNamespaces,
    namespace_bindings: Rc<RefCell<CssNamespaceBindings>>,
    /// The blocks the input ended, found in the token tree of the source when
    /// first asked for.
    implicit_openings: Rc<RefCell<Option<Rc<Vec<usize>>>>>,
    retained_implicit_openings: Rc<RefCell<Vec<usize>>>,
    retained_navigation_diagnostics: Rc<RefCell<Vec<crate::diagnostic::CssRecoveryDiagnostic>>>,
    syntax_base_depth: u32,
    /// The token tree of the source as it is lexed, kept for the structural
    /// questions recovery asks of it.
    structure: Rc<RefCell<Option<Rc<crate::parser::syntax_types::Document<'static>>>>>,
    /// The token tree the rule grammars read: the same tree, or the tree of
    /// the view of the source whose components past the nesting limit are masked.
    syntax_document: Rc<RefCell<Option<Rc<crate::parser::syntax_types::Document<'static>>>>>,
}

impl RecoveryState {
    pub(super) fn at_depth(source: &str, depth: u32, namespaces: SharedNamespaces) -> Self {
        Self::at_depth_with_snapshot(
            depth,
            namespaces,
            crate::source::CssSourceSnapshot::new(source),
        )
    }

    pub(crate) fn at_depth_with_snapshot(
        depth: u32,
        namespaces: SharedNamespaces,
        source_snapshot: crate::source::CssSourceSnapshot,
    ) -> Self {
        let namespace_bindings = Rc::clone(&namespaces.namespace_bindings);
        Self {
            parser_context: crate::CssParserContext::default(),
            depth: Rc::new(Cell::new(depth)),
            source_snapshot,
            text: None,
            namespaces,
            namespace_bindings,
            implicit_openings: Rc::new(RefCell::new(None)),
            retained_implicit_openings: Rc::new(RefCell::new(Vec::new())),
            retained_navigation_diagnostics: Rc::new(RefCell::new(Vec::new())),
            syntax_base_depth: depth,
            structure: Rc::new(RefCell::new(None)),
            syntax_document: Rc::new(RefCell::new(None)),
        }
    }

    /// The state that reads the tokens of `text`, a view of the text of the
    /// snapshot with a unit masked, and keeps the snapshot's positions.
    pub(super) fn with_text(&self, text: &str) -> Self {
        Self {
            text: Some(Rc::from(text)),
            implicit_openings: Rc::new(RefCell::new(None)),
            structure: Rc::new(RefCell::new(None)),
            syntax_document: Rc::new(RefCell::new(None)),
            ..self.clone()
        }
    }

    /// The text whose tokens are read.
    fn text(&self) -> &str {
        self.text
            .as_deref()
            .unwrap_or_else(|| self.source_snapshot.as_str())
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
            text: self.text.clone(),
            namespaces: self.namespaces.clone(),
            namespace_bindings: Rc::clone(&self.namespace_bindings),
            implicit_openings: Rc::clone(&self.implicit_openings),
            retained_implicit_openings: Rc::new(RefCell::new(Vec::new())),
            retained_navigation_diagnostics: Rc::new(RefCell::new(Vec::new())),
            syntax_base_depth: self.syntax_base_depth,
            structure: Rc::clone(&self.structure),
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
            text: None,
            namespaces: self.namespaces.clone(),
            namespace_bindings: Rc::clone(&self.namespace_bindings),
            implicit_openings: Rc::new(RefCell::new(None)),
            retained_implicit_openings: Rc::new(RefCell::new(Vec::new())),
            retained_navigation_diagnostics: Rc::new(RefCell::new(Vec::new())),
            syntax_base_depth: self.depth.get(),
            structure: Rc::new(RefCell::new(None)),
            syntax_document: Rc::new(RefCell::new(None)),
        }
    }

    /// The token tree of the source as it is lexed: every block it nests, past
    /// the nesting limit or not. Recovery asks its structural questions of it.
    pub(super) fn structure(
        &self,
    ) -> Result<
        Rc<crate::parser::syntax_types::Document<'static>>,
        crate::component::CssComponentValueError,
    > {
        if let Some(document) = self.structure.borrow().as_ref() {
            return Ok(Rc::clone(document));
        }
        let document = Rc::new(crate::syntax_consumption::recovery_structure_document(
            self.text(),
            &self.source_snapshot,
        )?);
        *self.structure.borrow_mut() = Some(Rc::clone(&document));
        Ok(document)
    }

    /// The token tree the rule grammars read the units of the source from. A
    /// source that stays within the nesting limit is read as it is lexed.
    /// Beyond the limit, every component that starts past it is masked, but a
    /// `{}` block, which the tree denies as one opaque node: it is a unit of
    /// its list, so its payload cannot absorb an independent following rule.
    pub(super) fn syntax_document(
        &self,
    ) -> Result<
        Rc<crate::parser::syntax_types::Document<'static>>,
        crate::component::CssComponentValueError,
    > {
        if let Some(document) = self.syntax_document.borrow().as_ref() {
            return Ok(Rc::clone(document));
        }
        let structure = self.structure()?;
        let reach =
            u64::from(self.syntax_base_depth) + u64::from(structure.metrics().maximum_depth());
        let document = if reach <= u64::from(STRUCTURAL_NESTING_LIMIT) {
            structure
        } else {
            let components = structure::over_limit_components(&structure, self.syntax_base_depth);
            let view = structure::normalization_view(self.text(), &components);
            Rc::new(crate::syntax_consumption::recovery_source_document(
                view.into_owned(),
                &self.source_snapshot,
                self.syntax_base_depth,
            )?)
        };
        *self.syntax_document.borrow_mut() = Some(Rc::clone(&document));
        Ok(document)
    }

    /// The openings of the blocks the input ended instead of closing.
    pub(super) fn implicit_openings(&self) -> Rc<Vec<usize>> {
        if let Some(openings) = self.implicit_openings.borrow().as_ref() {
            return Rc::clone(openings);
        }
        let openings = Rc::new(
            self.structure()
                .map(|document| structure::unclosed_openings(&document))
                .unwrap_or_default(),
        );
        *self.implicit_openings.borrow_mut() = Some(Rc::clone(&openings));
        openings
    }

    /// The nesting of the components that begin at the byte `start`, checked
    /// from the structural depth entered so far, and the end of the grammar unit
    /// they belong to.
    fn scan(
        &self,
        source: &str,
        start: usize,
        depth: u32,
        enclosing_production: &'static str,
        boundary: ScanBoundary,
    ) -> Result<usize, GrammarFailure> {
        let document = self.structure().map_err(|error| {
            crate::diagnostic::error::invalid_component_value(
                crate::parser::block_units::location_at(&self.source_snapshot, 0),
                error,
            )
        })?;
        structure::scan_nested(
            &document,
            source,
            start,
            depth,
            enclosing_production,
            boundary,
        )
    }

    pub(super) fn pending_component_closures(&self) -> Vec<usize> {
        self.retained_implicit_openings.borrow().clone()
    }

    pub(super) fn source_snapshot(&self) -> &crate::source::CssSourceSnapshot {
        &self.source_snapshot
    }

    /// The position of the byte `byte_offset` of the source, found from the
    /// checkpoint of the snapshot before it.
    pub(super) fn source_position(&self, byte_offset: usize) -> crate::source::CssSourcePosition {
        self.source_snapshot.position_or_walk(byte_offset)
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

    /// Enters the rule block whose opening token starts at `at`.
    pub(super) fn enter_rule_block(
        &self,
        at: Offset,
        enclosing_production: &'static str,
    ) -> Result<RecoveryDepthGuard, GrammarFailure> {
        self.enter(at.value(), enclosing_production)
    }

    /// Current entered structural block depth, including the active rule body.
    pub(super) fn structural_depth(&self) -> u32 {
        self.depth.get()
    }

    /// Enters the component block whose opening token ends at the byte
    /// `opening_offset`, its last byte.
    pub(super) fn enter_component_block_at(
        &self,
        opening_offset: usize,
        enclosing_production: &'static str,
    ) -> Result<RecoveryDepthGuard, GrammarFailure> {
        self.enter(opening_offset, enclosing_production)
    }

    /// Checks the nesting of a declaration value that starts at the byte
    /// `start`, and returns the openings of the blocks the input ends in it.
    pub(super) fn check_component_values_from(
        &self,
        source: &str,
        start: usize,
        enclosing_production: &'static str,
    ) -> Result<Vec<usize>, GrammarFailure> {
        let end = self.scan(
            source,
            start,
            self.depth.get(),
            enclosing_production,
            ScanBoundary::DeclarationValue,
        )?;
        Ok(self.component_openings_in(start..end))
    }

    /// Checks the nesting of a prelude that starts at the byte `start`, and
    /// returns the openings of the blocks the input ends in it.
    pub(super) fn check_specialized_components_from(
        &self,
        source: &str,
        start: usize,
        enclosing_production: &'static str,
    ) -> Result<Vec<usize>, GrammarFailure> {
        let end = self.scan(
            source,
            start,
            self.depth.get(),
            enclosing_production,
            ScanBoundary::SpecializedPrelude,
        )?;
        Ok(self.component_openings_in(start..end))
    }

    /// Checks the contents of a component block that start at the byte `start`.
    pub(super) fn check_component_contents_from(
        &self,
        source: &str,
        start: usize,
        enclosing_production: &'static str,
    ) -> Result<(), GrammarFailure> {
        self.scan(
            source,
            start,
            self.depth.get(),
            enclosing_production,
            ScanBoundary::ComponentContents,
        )
        .map(|_| ())
    }

    /// Checks the nesting of a comma-separated member that starts at the byte
    /// `start`, and returns the openings of the blocks the input ends in it.
    pub(super) fn check_comma_member_components_from(
        &self,
        source: &str,
        start: usize,
        enclosing_production: &'static str,
    ) -> Result<Vec<usize>, GrammarFailure> {
        let end = self.scan(
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
        self.implicit_openings()
            .iter()
            .rev()
            .copied()
            .filter(|opening| range.contains(opening))
            .collect()
    }

    pub(super) fn retain_component_closures(&self, openings: Vec<usize>) {
        let implicit = self.implicit_openings();
        let mut retained = self.retained_implicit_openings.borrow_mut();
        for opening in openings {
            if implicit.contains(&opening) && !retained.contains(&opening) {
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
        let Some(diagnostic) = crate::diagnostic::CssRecoveryDiagnostic::new(
            crate::diagnostic::error::legacy_navigation_target(target.as_str(), span.start()),
            span,
            crate::diagnostic::CssRecoveryAction::RetainLegacyNavigationTarget,
        ) else {
            return;
        };
        self.retained_navigation_diagnostics
            .borrow_mut()
            .push(diagnostic);
    }

    pub(super) fn take_implicit_closure_diagnostics(
        &self,
        source: &str,
    ) -> Vec<crate::diagnostic::CssRecoveryDiagnostic> {
        let eof = self.source_position(source.len());
        let Some(span) = crate::source::CssSourceSpan::new(eof, eof) else {
            return Vec::new();
        };
        let mut diagnostics: Vec<_> = self
            .retained_implicit_openings
            .borrow_mut()
            .drain(..)
            .filter_map(|opening| {
                crate::diagnostic::CssRecoveryDiagnostic::new(
                    crate::diagnostic::error::implicit_eof(eof),
                    span,
                    crate::diagnostic::CssRecoveryAction::RetainWithImplicitClosure,
                )
                .map(|diagnostic| diagnostic.with_opening_owner(opening))
            })
            .collect();
        diagnostics.extend(self.retained_navigation_diagnostics.borrow_mut().drain(..));
        diagnostics
    }

    /// Checks a failed rule whose block contents start at the byte
    /// `content_start` for the resources entering it would have been charged.
    pub(super) fn check_failed_rule_block_at(
        &self,
        source: &str,
        content_start: usize,
        enclosing_production: &'static str,
    ) -> Option<GrammarFailure> {
        let opening_offset = content_start.saturating_sub(1);
        if self.depth.get() >= STRUCTURAL_NESTING_LIMIT {
            return Some(nesting_limit(
                self.source_position(opening_offset),
                STRUCTURAL_NESTING_LIMIT,
                enclosing_production,
            ));
        }
        self.scan(
            source,
            content_start,
            self.depth.get() + 1,
            enclosing_production,
            ScanBoundary::FailedCurlyBlock,
        )
        .err()
    }

    /// Checks the contents of an entered `{}`-block that start at the byte
    /// `start` for the resources they would be charged.
    pub(super) fn check_entered_curly_contents(
        &self,
        source: &str,
        start: usize,
        enclosing_production: &'static str,
    ) -> Result<(), GrammarFailure> {
        self.scan(
            source,
            start,
            self.depth.get(),
            enclosing_production,
            ScanBoundary::FailedCurlyBlock,
        )
        .map(|_| ())
    }

    fn enter(
        &self,
        opening_offset: usize,
        enclosing_production: &'static str,
    ) -> Result<RecoveryDepthGuard, GrammarFailure> {
        let depth = self.depth.get();
        if depth >= STRUCTURAL_NESTING_LIMIT {
            return Err(nesting_limit(
                self.source_position(opening_offset),
                STRUCTURAL_NESTING_LIMIT,
                enclosing_production,
            ));
        }
        self.depth.set(depth + 1);
        Ok(RecoveryDepthGuard {
            depth: Rc::clone(&self.depth),
            opening_offset,
            implicit_openings: self.implicit_openings(),
            retained_implicit_openings: Rc::clone(&self.retained_implicit_openings),
            retained: false,
        })
    }
}

/// The namespace bindings of the grammar environment a source is parsed in,
/// shared by every state that reads the source.
#[derive(Clone, Default)]
pub(crate) struct SharedNamespaces {
    namespace_bindings: Rc<RefCell<CssNamespaceBindings>>,
}

impl SharedNamespaces {
    pub(crate) fn with_namespaces(context: &crate::selector::CssNamespaceContext) -> Self {
        Self {
            namespace_bindings: Rc::new(RefCell::new(context.0.clone())),
        }
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

/// One iteration's progress witness for a bounded recovery coordinator: the
/// offset it started at.
pub(crate) struct RecoveryProgress {
    start: Offset,
}

impl RecoveryProgress {
    pub(super) const fn record(at: Offset) -> Self {
        Self { start: at }
    }

    /// Judges an iteration that ended at `at`: progress is the comparison of
    /// the two offsets.
    pub(super) fn finish(self, at: Offset, retained: bool) -> RecoveryLoopOutcome {
        match (at > self.start, retained) {
            (true, true) => RecoveryLoopOutcome::Retained,
            (true, false) => RecoveryLoopOutcome::Advanced,
            (false, _) => RecoveryLoopOutcome::Terminated,
        }
    }
}

pub(crate) fn recovery_action_for_error(
    error: &GrammarFailure,
    ordinary: crate::diagnostic::CssRecoveryAction,
) -> crate::diagnostic::CssRecoveryAction {
    if is_nesting_limit_error(error) {
        crate::diagnostic::CssRecoveryAction::StopAtNestingLimit
    } else {
        ordinary
    }
}

/// The position of the first token of the member `member_start..member_end`
/// of `source` that is no trivia, which `locate` finds.
pub(super) fn first_non_trivia_position(
    source: &str,
    member_start: usize,
    member_end: usize,
    locate: impl Fn(usize) -> CssSourcePosition,
) -> CssSourcePosition {
    locate(structure::first_non_trivia(
        source,
        member_start,
        member_end,
    ))
}

/// The span of the member `member_start..member_end` of a comma-separated
/// list, or of the comma beside it when the member is empty; `locate` finds
/// the position of a byte.
pub(super) fn comma_member_span(
    locate: impl Fn(usize) -> CssSourcePosition,
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
    crate::source::CssSourceSpan::new(locate(span_start), locate(span_end))
}

#[cfg(test)]
mod tests {
    use surgeist_css_parser::Offset;

    use super::{RecoveryLoopOutcome, RecoveryProgress};

    /// The structural query and the document agree on which blocks are too deep:
    /// the `{}` blocks the query finds past the ceiling are exactly the ones the
    /// document denies on its own, with mixed paren, square and curly nesting
    /// around the ceiling and a nonzero base depth, and the document builds
    /// whatever the query masked.
    #[test]
    fn the_query_and_the_document_agree_on_which_blocks_are_too_deep() {
        use crate::STRUCTURAL_NESTING_LIMIT;
        use crate::parser::structure::{normalization_view, over_limit_components};
        use crate::source::CssSourceSnapshot;
        use crate::syntax_consumption::recovery_source_document;
        use surgeist_css_parser::document::BlockKind;
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
                            let snapshot = CssSourceSnapshot::new(&text);
                            let structure = crate::syntax_consumption::recovery_structure_document(
                                &text, &snapshot,
                            )
                            .unwrap();
                            let components = over_limit_components(&structure, base);
                            let expected: Vec<_> = components
                                .iter()
                                .filter(|component| component.block == BlockKind::Curly)
                                .map(|component| component.range.clone())
                                .collect();
                            let view = normalization_view(&text, &components).into_owned();
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
        use crate::syntax_consumption::{self as syntax, DocumentExt};
        use surgeist_css_parser::document::Component;
        let source = "@page {😀\r\nnot parsed}@layer kept;";
        let state = super::RecoveryState::at_depth(source, 256, super::SharedNamespaces::default());
        let document = state.syntax_document().unwrap();
        let root = document.root_list();
        let nodes = document.list(root);
        let layer_index = nodes
            .iter()
            .position(|node| document.kind(*node) == syntax::TokenKind::AtKeyword("layer"))
            .expect("the independent same-parent Layer unit");
        let layer_name = nodes[layer_index];
        let denied = *nodes
            .iter()
            .find(|node| matches!(document.component(**node), Component::Denied { .. }))
            .expect("the denied Page block");
        assert!(document.children(denied).is_none());
        let opening = source.find('{').unwrap();
        let next = source.find("@layer").unwrap();
        assert_eq!(
            document
                .range(denied)
                .map(|range| range.start.value()..range.end.value()),
            Some(opening..next)
        );
        assert_eq!(document.boundary(root, layer_index).offset.unwrap(), next);
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
        let crate::component::CssValueOrigin::Parsed(layer_origin) = &document.origin(layer_name)
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
        use crate::syntax_consumption::{self as syntax, DocumentExt};
        use surgeist_css_parser::document::Component;
        let denied_of = |document: &crate::parser::syntax_types::Document<'_>| {
            let nodes = document.list(document.root_list());
            nodes
                .iter()
                .position(|node| matches!(document.component(*node), Component::Denied { .. }))
                .map(|index| (index, nodes[index]))
        };
        let source = "bad{}@layer kept;";
        let state = super::RecoveryState::at_depth(source, 256, super::SharedNamespaces::default());
        let document = state.syntax_document().unwrap();
        let (index, denied) = denied_of(&document).expect("the denied qualified body");
        assert_eq!(
            document
                .range(denied)
                .map(|range| range.start.value()..range.end.value()),
            Some(3..5)
        );
        assert_eq!(
            document
                .boundary(document.root_list(), index + 1)
                .offset
                .unwrap(),
            5
        );
        assert_eq!(
            document.kind(document.list(document.root_list())[index + 1]),
            syntax::TokenKind::AtKeyword("layer")
        );

        for source in ["@page{unclosed payload", "@layer kept"] {
            let state =
                super::RecoveryState::at_depth(source, 256, super::SharedNamespaces::default());
            let document = state.syntax_document().unwrap();
            assert_eq!(document.metrics().spelling_bytes(), source.len());
            if source.starts_with("@page") {
                let (_, block) = denied_of(&document).expect("the unterminated denied block");
                assert_eq!(
                    document
                        .range(block)
                        .map(|range| range.start.value()..range.end.value()),
                    Some(5..source.len())
                );
            } else {
                assert!(denied_of(&document).is_none());
            }
        }
    }

    #[test]
    fn generated_selector_probe_shares_environment_but_owns_source_offsets() {
        let source = "original(";
        let captures = super::SharedNamespaces::default();
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
        {
            let _guard = probe
                .enter_component_block_at(0, "baseline.selector.complex")
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

        let state = super::RecoveryState::at_depth(source, 0, super::SharedNamespaces::default());
        assert_eq!(*state.implicit_openings(), [2, 8]);
    }

    #[test]
    fn structural_recovery_progress_is_the_comparison_of_two_offsets() {
        let start = Offset::new(3);
        let after = Offset::new(4);
        assert_eq!(
            RecoveryProgress::record(start).finish(after, true),
            RecoveryLoopOutcome::Retained
        );
        assert_eq!(
            RecoveryProgress::record(start).finish(after, false),
            RecoveryLoopOutcome::Advanced
        );
        assert_eq!(
            RecoveryProgress::record(start).finish(start, false),
            RecoveryLoopOutcome::Terminated
        );
        assert_eq!(
            RecoveryProgress::record(start).finish(start, true),
            RecoveryLoopOutcome::Terminated
        );
    }

    // Positions are found by continuing from the last one found. Whatever the
    // order of the queries, a stop between the bytes of a CRLF, a clone that
    // shares the cursor and a probe of another text, every answer is the
    // position of a walk from the start.
    #[test]
    fn source_position_equals_a_walk_from_the_start_in_any_query_order() {
        use crate::source::CssSourcePosition;
        let source = "a\r\nbé😀\n\u{c}x\ry\r\n\r\r\nz";
        let state = super::RecoveryState::at_depth(source, 0, super::SharedNamespaces::default());
        let shared = state.clone();
        let probe = state.detached_probe();
        let mut offsets: Vec<usize> = (0..=source.len())
            .filter(|offset| source.is_char_boundary(*offset))
            .collect();
        let walk = |offset: usize| CssSourcePosition::from_byte_offset_in(source, offset);
        // Ascending, with every stop between a CR and an LF.
        for offset in &offsets {
            assert_eq!(state.source_position(*offset), walk(*offset), "up {offset}");
        }
        // Descending: each query is behind the last one found.
        for offset in offsets.iter().rev() {
            assert_eq!(
                shared.source_position(*offset),
                walk(*offset),
                "down {offset}"
            );
        }
        // Interleaved through the state, its clone and its probe.
        offsets.sort_by_key(|offset| (offset * 7919) % 31);
        for (index, offset) in offsets.iter().enumerate() {
            let asked = [&state, &shared, &probe][index % 3];
            assert_eq!(
                asked.source_position(*offset),
                walk(*offset),
                "mixed {offset}"
            );
        }
        // A probe of another text finds its own positions.
        let other = state.generated_selector_probe("x\ny");
        assert_eq!(
            other.source_position(3),
            CssSourcePosition::from_byte_offset_in("x\ny", 3)
        );
        assert_eq!(
            other.source_position(1),
            CssSourcePosition::from_byte_offset_in("x\ny", 1)
        );
        // Past the end is the end.
        assert_eq!(state.source_position(source.len() + 9), walk(source.len()));
    }
}

//! Explicit authored document mode and glyph definition, independent of namespace bindings,
//! and the configured parser that owns every source-text front.
use crate::CssParseReport;
use crate::component::CssComponentValueLimits;
use crate::declaration::{CssDeclaration, CssDeclarationList, CssImportance};
use crate::property::CssPropertyTextTarget;
use crate::rule::CssSheet;
use crate::rule::font_face::CssFontFaceDescriptors;
use crate::rule::page::CssPageDeclarationBlock;
use crate::source::{
    CssDeclarationBlockParseReport, CssDecodedStylesheetInput, CssParseSource, CssUtf16InputError,
    CssUtf16InputLimits,
};
use std::sync::Arc;

/// The document mode that selects contextual authored CSS grammar.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssParserMode {
    /// Ordinary standards grammar, also required by CSS.supports() method syntax.
    #[default]
    Standards,
    /// Document quirks grammar for the exact property contexts defined by CSS.
    Quirks,
}

/// Immutable authored grammar context. The default context uses Standards
/// and the finite Writing Modes glyph alias. Named SVG selection is independent of mode.
///
/// This context does not resolve colors, evaluate queries, bind DOM objects, or
/// supply selector namespaces. Declarations retain it for strict pending reentry.
/// [`CssParser`] pairs it with component limits for the source-text fronts.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CssParserContext {
    mode: CssParserMode,
    glyph: GlyphDefinition,
}
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
enum GlyphDefinition {
    #[default]
    WritingModes,
    Svg,
}
impl CssParserContext {
    /// Selects one closed authored document mode.
    #[must_use]
    pub const fn new(mode: CssParserMode) -> Self {
        Self {
            mode,
            glyph: GlyphDefinition::WritingModes,
        }
    }
    /// Selects the document mode while preserving all other grammar choices.
    #[must_use]
    pub const fn with_mode(mut self, mode: CssParserMode) -> Self {
        self.mode = mode;
        self
    }
    /// Selects the independent frozen SVG authored definition in property-name lookup.
    /// Explicit finite grammar handles retain their finite meaning. This is idempotent.
    #[must_use]
    pub const fn with_svg_glyph_orientation_vertical(mut self) -> Self {
        self.glyph = GlyphDefinition::Svg;
        self
    }
    pub(crate) fn selects_svg_glyph(self, name: &str) -> bool {
        self.glyph == GlyphDefinition::Svg
            && name.eq_ignore_ascii_case("glyph-orientation-vertical")
    }
    pub(crate) const fn standards(mut self) -> Self {
        self.mode = CssParserMode::Standards;
        self
    }
    /// Returns the immutable selected mode.
    #[must_use]
    pub const fn mode(self) -> CssParserMode {
        self.mode
    }
}

/// Parser configuration for every source-text front: the grammar context and the
/// component limits that charge the input.
///
/// A parser is a small `Copy` value. Fronts are methods of the parser or `parse`
/// constructors of the value they produce, and each takes the parser first. A
/// front charges the component limits only where its grammar has an input
/// allowance; its documentation says which. The context selects the document
/// mode and the SVG glyph definition for the declarations a front admits.
///
/// ```
/// use surgeist_css::CssParser;
/// use surgeist_css::diagnostic::CssRecoveryAction;
///
/// let report = CssParser::default()
///     .parse_sheet(".a { color: red } @unknown x; .b {}");
/// assert_eq!(report.syntax().rules().len(), 2);
/// assert_eq!(report.diagnostics()[0].action(), CssRecoveryAction::DropAtRule);
/// ```
///
/// The fields are private, so a parser is only built through [`CssParser::new`]
/// or [`Default`]:
///
/// ```compile_fail,E0451
/// use surgeist_css::component::CssComponentValueLimits;
/// use surgeist_css::{CssParser, CssParserContext};
///
/// let _ = CssParser {
///     context: CssParserContext::default(),
///     limits: CssComponentValueLimits::default(),
/// };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CssParser {
    context: CssParserContext,
    limits: CssComponentValueLimits,
}

impl CssParser {
    /// Pairs a grammar context with the component limits that charge source input.
    #[must_use]
    pub const fn new(context: CssParserContext, limits: CssComponentValueLimits) -> Self {
        Self { context, limits }
    }

    /// Returns the grammar context the fronts admit declarations under.
    #[must_use]
    pub const fn context(&self) -> CssParserContext {
        self.context
    }

    /// Returns the component limits that charge the fronts which have an input allowance.
    #[must_use]
    pub const fn component_limits(&self) -> CssComponentValueLimits {
        self.limits
    }

    /// Parses a UTF-8 stylesheet into valid authored syntax and recovery diagnostics.
    ///
    /// The ordinary parser retains valid top-level rules in source order and reports
    /// each discarded top-level rule with its complete balanced source span. Parsed
    /// `@charset` rules are unrecognized and dropped; top-level CDO/CDC tokens are
    /// ignored without diagnostics. The UTF-8 input is not decoded. The stylesheet
    /// root is structural depth zero; up to 256 shared rule-block/component/function
    /// levels are retained, and the first level beyond that drops its smallest
    /// enclosing recovery unit. Input deeper than 128 levels is parsed on a bounded
    /// larger-stack thread; if the platform cannot start it, the report retains no
    /// rules and carries a
    /// [`CssErrorCode::ParserUnavailable`](crate::diagnostic::CssErrorCode::ParserUnavailable)
    /// refusal (limit 128, action `StopAtNestingLimit`), followed only by any
    /// tokenizer EOF or escape diagnostics, rather than recursing on the caller's
    /// stack. Recovery does not apply cascade, substitution, selector matching,
    /// contextual resolution, or resource loading. The parser supplies the document
    /// mode; a sheet charges no component limits.
    ///
    /// ```
    /// use surgeist_css::CssParser;
    /// use surgeist_css::diagnostic::CssRecoveryAction;
    /// use surgeist_css::rule::CssRule;
    ///
    /// let report = CssParser::default().parse_sheet(
    ///     ".before { color: red; } @unknown fn({x;y}); .after { color: blue; }",
    /// );
    /// assert_eq!(report.syntax().rules().len(), 2);
    /// assert!(!report.is_clean());
    /// assert!(matches!(report.syntax().rules()[0], CssRule::Style(_)));
    /// assert!(matches!(
    ///     report.diagnostics()[0].action(),
    ///     CssRecoveryAction::DropAtRule
    /// ));
    /// ```
    #[must_use]
    pub fn parse_sheet(&self, source: impl AsRef<str>) -> CssParseReport<CssSheet> {
        crate::parser::parse_sheet_with_context(source.as_ref(), self.context)
    }

    /// Parses the exact snapshot of a decoded stylesheet input.
    ///
    /// Decoder replacement is not itself a CSS diagnostic. The sheet has no
    /// location until [`CssParseReport::with_location`] attaches one; a location
    /// never affects parsing.
    #[must_use]
    pub fn parse_sheet_bytes(&self, input: &CssDecodedStylesheetInput) -> CssParseReport<CssSheet> {
        crate::parser::parse_sheet_snapshot(input.source(), self.context)
    }

    /// Preprocesses shared original UTF16, then parses an ordinary stylesheet.
    ///
    /// Polls during preparation and immediately before/after grammar execution;
    /// the grammar has no fine-grained interruption checkpoint.
    ///
    /// # Errors
    ///
    /// Returns the preparation refusal or the caller's interruption, with no
    /// partial report.
    pub fn parse_sheet_utf16<E>(
        &self,
        original: Arc<Vec<u16>>,
        limits: CssUtf16InputLimits,
        poll: impl FnMut() -> Result<(), E>,
    ) -> Result<CssParseReport<CssSheet>, CssUtf16InputError<E>> {
        crate::source::utf16_input::parse_utf16(original, limits, poll, |source| {
            crate::parser::parse_sheet_snapshot(source, self.context)
        })
    }

    /// Parses block contents: a style attribute, CSSOM `cssText` or a raw declaration list.
    ///
    /// This is CSS Syntax 3 (2026 CRD) §5.5.5 "consume a block's contents" over
    /// unwrapped text; no braces are required or invented. A root `}` ends the
    /// contents, so `color: red } width: 1px` retains only `color`. A unit that
    /// is not a declaration is consumed as a nested rule and dropped from this
    /// declaration-only projection, so `a {} color: red` retains `color`.
    /// Root semicolons, trivia and EOF are empty slots. Declaration admission
    /// uses the ordinary property grammar, including custom-property recovery
    /// through the root semicolon. Ordered occurrences, duplicates, importance
    /// and original name/value source identities remain authored data; CSSOM
    /// winner selection is a separate operation.
    ///
    /// The component limits charge every original component once, including
    /// descendants, trivia and source after the root stop token. A failed
    /// allowance returns an empty projection and its typed resource diagnostic;
    /// no partially admitted declarations escape.
    ///
    /// Input nested deeper than 63 levels runs on the bounded larger-stack parser
    /// thread. If the platform cannot start it, the report is empty and carries one
    /// [`CssErrorCode::ParserUnavailable`](crate::diagnostic::CssErrorCode::ParserUnavailable)
    /// refusal (limit 63, action `StopAtNestingLimit`) over the whole source rather
    /// than recursing on the caller's stack. This is the fragment threshold, below
    /// the 128 levels a stylesheet parses directly.
    ///
    /// ```
    /// use surgeist_css::declaration::CssImportance;
    /// use surgeist_css::CssParser;
    ///
    /// let report = CssParser::default().parse_block_contents("color: red; width: 2px !important");
    /// assert!(report.is_clean());
    /// assert_eq!(report.syntax()[1].importance(), CssImportance::Important);
    /// ```
    #[must_use]
    pub fn parse_block_contents(
        &self,
        source: impl AsRef<str>,
    ) -> CssParseReport<CssDeclarationList> {
        crate::parser::declaration_block::parse_with_context(
            source.as_ref(),
            self.limits,
            self.context,
        )
    }

    /// Preprocesses original UTF16 and parses block contents over the exact source.
    ///
    /// Uses the grammar, recovery and 63-level parser-thread refusal of
    /// [`Self::parse_block_contents`]. The
    /// component limits still admit the complete normalized input, including
    /// text after a root `}`. `input_limits` bounds preparation and its source
    /// backing. Polling occurs during preparation and immediately before/after
    /// grammar execution. General grammar execution is opaque between those
    /// observations: its component, depth and byte admission is separate from
    /// preparation work, and no per-action grammar work/live-storage accounting
    /// is promised. Preparation refusal or interruption returns no partial
    /// source or report.
    ///
    /// # Errors
    ///
    /// Returns the preparation refusal or the caller's interruption.
    pub fn parse_block_contents_utf16<E>(
        &self,
        original: Arc<Vec<u16>>,
        input_limits: CssUtf16InputLimits,
        poll: impl FnMut() -> Result<(), E>,
    ) -> Result<CssDeclarationBlockParseReport, CssUtf16InputError<E>> {
        crate::source::utf16_input::parse_utf16(original, input_limits, poll, |source| {
            let report =
                crate::parser::declaration_block::parse_snapshot(source, self.limits, self.context);
            CssDeclarationBlockParseReport::new(
                CssParseSource::new(source.clone(), self.context),
                report,
            )
        })
    }

    /// Parses raw Page declaration contents without creating margin child rules.
    ///
    /// The exact unbraced input uses block-contents recovery. Page descriptors,
    /// applicable properties and case-sensitive custom declarations retain their
    /// distinct typed occurrences, duplicate order and original origins.
    /// At-rules (including margin rules) and qualified rules are dropped as
    /// complete units; valid declaration neighbors survive. A root `}` stops
    /// consumption.
    ///
    /// `Some(empty)` is successful empty or wholly invalid recovered contents.
    /// `None` with a typed resource diagnostic is failed preparation, never
    /// successful clearing. A CSSOM consumer must reject that failure before
    /// publishing edits. The component limits charge original descendants and
    /// trivia, including input after the root stop token; no retained prefix
    /// escapes a failure.
    #[must_use]
    pub fn parse_page_block_contents(
        &self,
        source: impl AsRef<str>,
    ) -> CssParseReport<Option<CssPageDeclarationBlock>> {
        crate::parser::domain_declaration_contents::parse_page_with_context(
            source.as_ref(),
            self.limits,
            self.context,
        )
    }

    /// Parses raw `@font-face` descriptor contents over the exact unbraced input.
    ///
    /// Existing font-face value, priority and validity rules admit descriptors
    /// in authored order, including duplicates and recovered source-list
    /// members. Invalid descriptors recover locally. Nested structural units are
    /// dropped and a root `}` stops consumption; no containing rule or brace is
    /// invented. `@font-face` grammar is independent of the document mode.
    ///
    /// `Some(empty)` is successful recovered empty contents; `None` retains a
    /// typed resource failure and must not be used to clear a CSSOM block. The
    /// component limits charge source after the root `}`; any input or grammar
    /// resource failure returns `None` with its typed cause.
    ///
    /// ```
    /// use surgeist_css::rule::font_face::CssFontFaceDescriptorKind;
    /// use surgeist_css::CssParser;
    ///
    /// let report = CssParser::default().parse_font_face_block_contents(
    ///     "font-family: Example; bad: value; src: local(Example)",
    /// );
    /// let descriptors = report.syntax().as_ref().unwrap();
    /// assert_eq!(descriptors.occurrences().len(), 2);
    /// assert!(descriptors.effective(CssFontFaceDescriptorKind::Src).is_some());
    /// assert!(!report.is_clean());
    /// ```
    #[must_use]
    pub fn parse_font_face_block_contents(
        &self,
        source: impl AsRef<str>,
    ) -> CssParseReport<Option<CssFontFaceDescriptors>> {
        crate::parser::domain_declaration_contents::parse_font_face_declaration_block_contents_with_limits(
            source.as_ref(),
            self.limits,
        )
    }

    /// Parses a complete raw value with a supplied property name or grammar.
    ///
    /// The source is the value alone: no name, colon, annotation or declaration
    /// delimiter. `importance` is the separate argument. Surrounding whitespace
    /// and comments are accepted; empty custom values and nested custom-property
    /// punctuation are accepted. The declaration has no parsed name or name
    /// position; its parsed value and components share the original source
    /// snapshot. Values remain authored and symbolic. A [`CssPropertyTarget::Grammar`](crate::property::CssPropertyTarget::Grammar)
    /// retains the selected canonical or legacy grammar for ordinary, CSS-wide
    /// and symbolic values.
    ///
    /// Invalid grammar returns `None` with `RejectInput`; resource exhaustion
    /// uses `StopAtNestingLimit`. Implicit EOF closures are reported only on
    /// retention.
    ///
    /// A page or margin value has no source-text front: it is checked from owned
    /// components through [`CssDeclaration::try_from_components`], so
    /// [`CssPropertyTextTarget`] cannot name one.
    ///
    /// ```
    /// use surgeist_css::declaration::CssImportance;
    /// use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTextTarget};
    /// use surgeist_css::CssParser;
    ///
    /// let target = CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width));
    /// let report = CssParser::default().parse_property_value("calc(100% - 12px)", target, CssImportance::Normal);
    /// let declaration = report.into_validation_result().unwrap().expect("declaration");
    /// assert_eq!(declaration.known().unwrap().property(), CssKnownProperty::Width);
    /// ```
    ///
    /// A page or margin target has no source-text front, so this front cannot be
    /// asked for one:
    ///
    /// ```compile_fail,E0308
    /// use surgeist_css::declaration::CssImportance;
    /// use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTarget};
    /// use surgeist_css::CssParser;
    ///
    /// let target = CssPropertyTarget::PageProperty(CssPropertyNameRef::Known(CssKnownProperty::Margin));
    /// let _ = CssParser::default().parse_property_value("1cm", target, CssImportance::Normal);
    /// ```
    #[must_use]
    pub fn parse_property_value(
        &self,
        source: impl AsRef<str>,
        target: CssPropertyTextTarget<'_>,
        importance: CssImportance,
    ) -> CssParseReport<Option<CssDeclaration>> {
        let source = source.as_ref();
        match target {
            CssPropertyTextTarget::Property(property) => {
                crate::parser::fragments::parse_property_value_text_with_context(
                    source,
                    property,
                    importance,
                    self.context,
                )
            }
            CssPropertyTextTarget::Grammar(grammar) => {
                crate::parser::fragments::parse_property_value_text_for_grammar_with_context(
                    source,
                    grammar,
                    importance,
                    self.context,
                )
            }
        }
    }
}

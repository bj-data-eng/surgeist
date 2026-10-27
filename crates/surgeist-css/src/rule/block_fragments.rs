//! Source-produced envelopes for genuine CSS block fragments.

use crate::rule::font_feature_values::CssFontFeatureValueKind;
use crate::selector::CssNamespaceContext;
use crate::source::CssParsedOrigin;
use crate::{CssParseReport, CssParser};

/// An admitted grammar-specific body and the origin of its genuine curly block.
///
/// The origin includes the opening brace and explicit closing brace, or ends at
/// original EOF for an implicit closure. Surrounding trivia remains in the shared
/// source snapshot but lies outside this span. Parsing reports own recovery
/// diagnostics; the presence of a fragment does not imply a clean report.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBlockFragment<T> {
    body: T,
    origin: CssParsedOrigin,
}

impl<T> CssBlockFragment<T> {
    pub(crate) fn from_parsed(body: T, origin: CssParsedOrigin) -> Self {
        Self { body, origin }
    }

    pub(crate) fn into_parts(self) -> (T, CssParsedOrigin) {
        (self.body, self.origin)
    }

    /// Borrows the body retained by its grammar and recovery owner.
    #[must_use]
    pub const fn body(&self) -> &T {
        &self.body
    }

    /// Borrows the original-source range of the consumed curly block.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}

/// An original-source ordinary inner rule list retained by its parsing owner.
///
/// Recovery diagnostics belong to the parsing report. This list supplies no
/// stylesheet metadata or enclosing rule name and has no public assembly path.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRuleList {
    rules: Vec<crate::rule::CssRule>,
}

impl CssRuleList {
    pub(crate) fn from_parsed(rules: Vec<crate::rule::CssRule>) -> Self {
        Self { rules }
    }

    pub(crate) fn rules_mut(&mut self) -> &mut Vec<crate::rule::CssRule> {
        &mut self.rules
    }

    /// Borrows admitted rules in original source order.
    #[must_use]
    pub fn rules(&self) -> &[crate::rule::CssRule] {
        &self.rules
    }
}

/// Actual enclosing style ancestry for an authored scoped body.
///
/// An enclosing style rule supplies declaration and parent-selector permission,
/// including through scopes. This does not bind or invent a parent selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssStyleAncestor {
    /// No enclosing style rule supplies style ancestry.
    Absent,
    /// An enclosing style rule supplies style ancestry.
    Present,
}

/// The grammar a block body is read under, besides the parser's own configuration.
///
/// A body reads only the part of the context it names. A rule list reads its
/// namespace bindings and style ancestry: [`Self::default`] reads it as an
/// ordinary group without namespace bindings. Every other body reads nothing,
/// and a part a body does not read is ignored. A `@font-feature-values`
/// definition block is selected by its kind, which
/// [`CssBlockFragment::parse_definition`] takes as an argument.
///
/// ```
/// use surgeist_css::rule::{CssBlockContext, CssBlockFragment, CssRuleList};
/// use surgeist_css::selector::CssNamespaceContext;
/// use surgeist_css::CssParser;
///
/// let namespaces = CssNamespaceContext::default();
/// let report = CssBlockFragment::<CssRuleList>::parse(
///     &CssParser::default(),
///     "{ .a { color: red } }",
///     &CssBlockContext::group(&namespaces),
/// );
/// assert_eq!(report.syntax().as_ref().unwrap().body().rules().len(), 1);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct CssBlockContext<'a> {
    select: BlockSelect<'a>,
}

#[derive(Clone, Copy, Debug, Default)]
enum BlockSelect<'a> {
    #[default]
    Plain,
    Group(&'a CssNamespaceContext),
    Scope(&'a CssNamespaceContext, CssStyleAncestor),
    ScopedGroup(&'a CssNamespaceContext, CssStyleAncestor),
}

/// The part of a [`CssBlockContext`] a rule list reads.
pub(crate) enum RuleListSelect<'a> {
    Group(Option<&'a CssNamespaceContext>),
    Scope(&'a CssNamespaceContext, CssStyleAncestor),
    ScopedGroup(&'a CssNamespaceContext, CssStyleAncestor),
}

impl<'a> CssBlockContext<'a> {
    /// An ordinary inner rule list under the supplied namespace bindings.
    ///
    /// No enclosing style ancestry is supplied.
    #[must_use]
    pub const fn group(namespaces: &'a CssNamespaceContext) -> Self {
        Self {
            select: BlockSelect::Group(namespaces),
        }
    }

    /// A scope body under the supplied namespace bindings and actual style ancestry.
    ///
    /// `Present` admits direct scoped declaration runs; `Absent` does not.
    /// Scope-body placement excludes `@page` rules. No scope root, limit or
    /// parent selector is invented.
    #[must_use]
    pub const fn scope(namespaces: &'a CssNamespaceContext, ancestry: CssStyleAncestor) -> Self {
        Self {
            select: BlockSelect::Scope(namespaces, ancestry),
        }
    }

    /// An ordinary group body inside a scope, with actual style ancestry.
    ///
    /// Unlike a scope body, an ordinary scoped group without a style ancestor
    /// admits `@page` rules. With `Present`, direct declarations inherit style
    /// permission and global-only rules retain the scoped owner's exclusions.
    #[must_use]
    pub const fn scoped_group(
        namespaces: &'a CssNamespaceContext,
        ancestry: CssStyleAncestor,
    ) -> Self {
        Self {
            select: BlockSelect::ScopedGroup(namespaces, ancestry),
        }
    }

    pub(crate) const fn rule_list(&self) -> RuleListSelect<'a> {
        match self.select {
            BlockSelect::Group(namespaces) => RuleListSelect::Group(Some(namespaces)),
            BlockSelect::Scope(namespaces, ancestry) => RuleListSelect::Scope(namespaces, ancestry),
            BlockSelect::ScopedGroup(namespaces, ancestry) => {
                RuleListSelect::ScopedGroup(namespaces, ancestry)
            }
            BlockSelect::Plain => RuleListSelect::Group(None),
        }
    }
}

/// A grammar-specific body that a real curly block can be parsed into.
///
/// The set is closed: the trait is sealed, and [`CssBlockFragment::parse`] is
/// its only entry. Bodies are rule lists, `@page` and margin bodies, keyframe
/// bodies, the descriptor bodies of the descriptor at-rules, named-supports
/// test bodies and the `@font-feature-values` bodies. The `@font-feature-values`
/// definition block is selected by its kind and is read through
/// [`CssBlockFragment::parse_definition`] instead.
pub trait CssBlockBody: crate::sealed::BlockBody {}

impl<T: crate::sealed::BlockBody> CssBlockBody for T {}

impl<B: CssBlockBody> CssBlockFragment<B> {
    /// Parses exactly one genuine curly block as the grammar of the body type `B`.
    ///
    /// Outer whitespace and comments are accepted and lie outside the origin of
    /// the block; a missing brace or trailing nontrivia rejects the complete
    /// input with `None`. Empty bodies are valid. Declaration and child recovery
    /// is local to its unit and retains admitted neighbors, in original source
    /// order, with original-source diagnostics; non-declaration structural
    /// failures of a descriptor body reject the complete body. The actual brace
    /// counts toward the 256-level nesting limit. Retained implicit closures and
    /// lexical or unit recovery fail clean-report validation. No enclosing
    /// at-keyword, prelude, name, selector or font resource is invented.
    ///
    /// The parser supplies the document mode of the declarations the body admits;
    /// bodies whose grammar is independent of it ignore it. `context` selects the
    /// rule-list variant and is ignored by every other body. The parser's component
    /// limits are not charged: each body's own bounded grammar applies the
    /// 256-level nesting limit.
    ///
    /// A `@font-feature-values` definition block is not a `CssBlockBody`: its
    /// grammar needs a kind, which this front cannot be asked to leave out:
    ///
    /// ```compile_fail,E0599
    /// use surgeist_css::rule::{CssBlockContext, CssBlockFragment};
    /// use surgeist_css::rule::font_feature_values::CssFontFeatureValueBlock;
    /// use surgeist_css::CssParser;
    ///
    /// let _ = CssBlockFragment::<CssFontFeatureValueBlock>::parse(
    ///     &CssParser::default(),
    ///     "{ a: 1 }",
    ///     &CssBlockContext::default(),
    /// );
    /// ```
    ///
    /// ```
    /// use surgeist_css::rule::{CssBlockContext, CssBlockFragment};
    /// use surgeist_css::rule::font_face::CssFontFaceDescriptors;
    /// use surgeist_css::CssParser;
    ///
    /// let report = CssBlockFragment::<CssFontFaceDescriptors>::parse(
    ///     &CssParser::default(),
    ///     " { font-family: Inter; src: local(Inter) } ",
    ///     &CssBlockContext::default(),
    /// );
    /// assert!(report.is_clean());
    /// let fragment = report.syntax().as_ref().unwrap();
    /// assert_eq!(fragment.body().occurrences().len(), 2);
    /// assert_eq!(fragment.origin().span().start().byte_offset().value(), 1);
    /// ```
    #[must_use]
    pub fn parse(
        parser: &CssParser,
        source: impl AsRef<str>,
        context: &CssBlockContext<'_>,
    ) -> CssParseReport<Option<Self>> {
        B::parse_block(parser, source.as_ref(), context)
    }
}

impl CssBlockFragment<crate::rule::font_feature_values::CssFontFeatureValueBlock> {
    /// Parses exactly one genuine curly block as the definition grammar of a
    /// `@font-feature-values` feature of the supplied kind.
    ///
    /// The kind is chosen by the calling code, never by the CSS input. The block,
    /// trivia, recovery and diagnostics follow [`CssBlockFragment::parse`]. The
    /// parser supplies the document mode and its component limits are not charged.
    ///
    /// ```
    /// use surgeist_css::rule::CssBlockFragment;
    /// use surgeist_css::rule::font_feature_values::{CssFontFeatureValueBlock, CssFontFeatureValueKind};
    /// use surgeist_css::CssParser;
    ///
    /// let report = CssBlockFragment::<CssFontFeatureValueBlock>::parse_definition(
    ///     &CssParser::default(),
    ///     "{ swash-a: 1 }",
    ///     CssFontFeatureValueKind::Swash,
    /// );
    /// assert!(report.is_clean());
    /// ```
    #[must_use]
    pub fn parse_definition(
        parser: &CssParser,
        source: impl AsRef<str>,
        kind: CssFontFeatureValueKind,
    ) -> CssParseReport<Option<Self>> {
        crate::parser::fragments::parse_font_feature_value_block(
            source.as_ref(),
            kind,
            parser.context(),
        )
    }
}

macro_rules! block_body {
    ($body:ty, |$parser:ident, $source:ident, $context:ident| $parse:expr) => {
        impl crate::sealed::BlockBody for $body {
            fn parse_block(
                $parser: &CssParser,
                $source: &str,
                $context: &CssBlockContext<'_>,
            ) -> CssParseReport<Option<CssBlockFragment<Self>>> {
                $parse
            }
        }
    };
}

impl crate::sealed::BlockBody for CssRuleList {
    fn parse_block(
        parser: &CssParser,
        source: &str,
        context: &CssBlockContext<'_>,
    ) -> CssParseReport<Option<CssBlockFragment<Self>>> {
        let ordinary = parser.context();
        match context.rule_list() {
            RuleListSelect::Group(namespaces) => {
                let default = CssNamespaceContext::default();
                crate::parser::fragments::parse_group_block_with_context(
                    source,
                    namespaces.unwrap_or(&default),
                    ordinary,
                )
            }
            RuleListSelect::Scope(namespaces, ancestry) => {
                crate::parser::fragments::parse_scope_block_with_context(
                    source, namespaces, ancestry, ordinary,
                )
            }
            RuleListSelect::ScopedGroup(namespaces, ancestry) => {
                crate::parser::fragments::parse_scoped_group_block_with_context(
                    source, namespaces, ancestry, ordinary,
                )
            }
        }
    }
}

block_body!(
    crate::rule::page::CssPageBody,
    |parser, source, _context| {
        crate::parser::fragments::parse_page_block_with_context(source, parser.context())
    }
);
block_body!(
    crate::rule::page::CssMarginDeclarationBlock,
    |parser, source, _context| {
        crate::parser::fragments::parse_margin_block_with_context(source, parser.context())
    }
);
block_body!(
    Vec<crate::rule::keyframes::CssKeyframeBlock>,
    |parser, source, _context| {
        crate::parser::fragments::parse_keyframes_block_with_context(source, parser.context())
    }
);
block_body!(
    crate::rule::keyframes::CssKeyframeDeclarationList,
    |parser, source, _context| {
        crate::parser::fragments::parse_keyframe_declaration_block_with_context(
            source,
            parser.context(),
        )
    }
);
block_body!(
    crate::query::supports::CssSupportsTestBody,
    |parser, source, _context| {
        crate::parser::fragments::parse_supports_test_block(source, parser.context())
    }
);
block_body!(
    crate::rule::font_face::CssFontFaceDescriptors,
    |parser, source, _context| {
        crate::parser::fragments::parse_font_face_block(source, parser.context())
    }
);
block_body!(
    crate::rule::counter_style::CssCounterStyleDescriptors,
    |parser, source, _context| {
        crate::parser::fragments::parse_counter_style_block(source, parser.context())
    }
);
block_body!(
    Vec<crate::rule::font_palette_values::CssFontPaletteDescriptor>,
    |parser, source, _context| {
        crate::parser::fragments::parse_font_palette_values_block(source, parser.context())
    }
);
block_body!(
    Vec<crate::rule::color_profile::CssColorProfileDescriptor>,
    |parser, source, _context| {
        crate::parser::fragments::parse_color_profile_block(source, parser.context())
    }
);
block_body!(
    Vec<crate::rule::font_feature_values::CssFontFeatureValuesItem>,
    |parser, source, _context| {
        crate::parser::fragments::parse_font_feature_values_block(source, parser.context())
    }
);

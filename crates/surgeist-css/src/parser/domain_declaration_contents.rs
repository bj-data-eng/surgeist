//! Recovered unbraced declaration contents in their existing domain grammar.
use crate::{
    CssComponentValueLimits, CssFontFaceDescriptors, CssPageDeclarationBlock, CssParseReport,
    CssParserContext,
};

/// Parses raw Page declaration contents without creating margin child rules.
///
/// The exact unbraced input uses current CSS Syntax block-contents recovery.
/// Page descriptors, applicable properties and case-sensitive custom declarations
/// retain their distinct typed occurrences, duplicate order and original origins.
/// At-rules (including margin rules) and qualified rules are dropped as complete
/// units; valid declaration neighbors survive. A root `}` stops consumption.
///
/// `Some(empty)` is successful empty or wholly invalid recovered contents.
/// `None` with a typed resource diagnostic is failed preparation, never successful
/// clearing. A CSSOM consumer must reject that failure before publishing edits.
/// Report cleanliness alone is not the recovery admission test.
#[must_use]
pub fn parse_page_declaration_block_contents(
    source: &str,
) -> CssParseReport<Option<CssPageDeclarationBlock>> {
    parse_page_declaration_block_contents_with_limits(source, CssComponentValueLimits::default())
}

/// Parses Page contents with cumulative whole-input component/byte/depth limits.
///
/// Limits charge original descendants and trivia, including input after the root
/// stop token. Grammar work failures also reject the complete preparation. No
/// retained prefix escapes; typed causes preserve original coordinates, and an
/// unchanged larger-budget retry is independent. Page winner selection and live
/// child identity stay with their separate owners.
#[must_use]
pub fn parse_page_declaration_block_contents_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssPageDeclarationBlock>> {
    parse_page_with_context(source, limits, CssParserContext::default())
}

pub(crate) fn parse_page_with_context(
    source: &str,
    limits: CssComponentValueLimits,
    context: CssParserContext,
) -> CssParseReport<Option<CssPageDeclarationBlock>> {
    super::page::parse_contents(source, limits, context)
}

/// Parses raw FontFace descriptor contents over the exact unbraced input.
///
/// Existing FontFace value, priority and validity rules admit descriptors in
/// authored order, including duplicates and recovered source-list members.
/// Invalid descriptors recover locally. Nested structural units are dropped,
/// and a root `}` stops consumption; no containing rule or brace is invented.
/// This raw projection is distinct from the real-brace FontFace fragment's
/// existing fatal structural policy.
///
/// `Some(empty)` is successful recovered empty contents; `None` retains a typed
/// resource failure and must not be used to clear a CSSOM block. No partial
/// preparation escapes that failure. FontFace grammar is independent of document
/// quirks mode; selected descriptor getters and live state remain downstream.
///
/// ```
/// use surgeist_css::{CssFontFaceDescriptorKind, parse_font_face_declaration_block_contents};
/// let report = parse_font_face_declaration_block_contents(
///     "font-family: Example; bad: value; src: local(Example)");
/// let descriptors = report.syntax().as_ref().unwrap();
/// assert_eq!(descriptors.occurrences().len(), 2);
/// assert!(descriptors.effective(CssFontFaceDescriptorKind::Src).is_some());
/// assert!(!report.is_clean());
/// ```
#[must_use]
pub fn parse_font_face_declaration_block_contents(
    source: &str,
) -> CssParseReport<Option<CssFontFaceDescriptors>> {
    parse_font_face_declaration_block_contents_with_limits(
        source,
        CssComponentValueLimits::default(),
    )
}

/// Parses FontFace contents with cumulative original-input limits.
///
/// Source after the root `}` is still charged to input limits. Any input or
/// grammar resource failure returns `None` with its typed cause rather than a
/// successful empty or partial descriptor collection. Value/recovery origins
/// retain the full original input; no per-member budget reset is performed.
#[must_use]
pub fn parse_font_face_declaration_block_contents_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssFontFaceDescriptors>> {
    super::font_face::parse_contents(source, limits)
}

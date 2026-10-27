//! Recovered unbraced declaration contents in their existing domain grammar.
use crate::component::CssComponentValueLimits;
use crate::rule::font_face::CssFontFaceDescriptors;
use crate::rule::page::CssPageDeclarationBlock;
use crate::{CssParseReport, CssParserContext};

pub(crate) fn parse_page_with_context(
    source: &str,
    limits: CssComponentValueLimits,
    context: CssParserContext,
) -> CssParseReport<Option<CssPageDeclarationBlock>> {
    super::page::parse_contents(source, limits, context)
}

#[must_use]
pub(crate) fn parse_font_face_declaration_block_contents_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssFontFaceDescriptors>> {
    super::font_face::parse_contents(source, limits)
}

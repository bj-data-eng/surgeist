//! Owned preparation and line-operation boundaries for the standalone engine.
//!
//! Source coordinates are UTF-16. Glyph-buffer byte ranges are private and are
//! always accompanied by their original-source association.

use std::{ops::Range as IndexRange, sync::Arc};

use crate::projection::SourceProjection;
use crate::{Alignment, FontCatalogueIdentity, InlineBox, Point, Range, Source, Style};

pub(crate) mod analysis;
mod bidi;
mod convert;
pub(crate) mod editing;
pub(crate) mod layout;
mod lru_cache;
pub(crate) mod resolve;
pub(crate) mod shape;

#[cfg(test)]
mod tests;

pub(crate) use crate::error::{shaping_refused, text_numeric_error};

/// One atom decoded from original backing, before any rendering substitution.
#[derive(Clone, Debug)]
pub(crate) struct OriginalAtom {
    pub(crate) source: Range,
    pub(crate) glyph_bytes: IndexRange<usize>,
    pub(crate) code_point: u32,
    pub(crate) glyph: char,
}

/// Font-independent original-source analysis, retained by every later phase.
pub(crate) struct AnalyzedData {
    pub(crate) source: Source,
    pub(crate) default_style: Style,
    pub(crate) projection: SourceProjection,
    pub(crate) styles: Vec<resolve::AnalyzedStyleRun>,
    pub(crate) analysis: analysis::Analysis,
}

/// The immutable, resource-retaining result of actual preparation.
///
/// Source and its projection live here once. Finalized lines retain this same
/// allocation; neither selection nor placement reconstructs authored input.
pub(crate) struct PreparedData {
    pub(crate) analyzed: Arc<AnalyzedData>,
    pub(crate) font_catalogue: FontCatalogueIdentity,
    pub(crate) styles: Vec<resolve::ResolvedStyleRun>,
    pub(crate) shaped: layout::data::ShapedData,
    pub(crate) quantize: bool,
}

/// Inputs coupled to one shaping operation, without a mutable System escape.
pub(crate) struct ShapingInput<'a> {
    pub(crate) projection: &'a SourceProjection,
    pub(crate) analysis: &'a analysis::Analysis,
    pub(crate) styles: &'a [resolve::ResolvedStyleRun],
    pub(crate) boxes: &'a [InlineBox],
    pub(crate) font_catalogue: &'a FontCatalogueIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BreakKind {
    Soft,
    Explicit,
    End,
}

/// An actual standalone choice over immutable prepared mixed items.
#[derive(Clone, Debug)]
pub(crate) struct LineSelection {
    pub(crate) items: IndexRange<usize>,
    pub(crate) source: Range,
    pub(crate) break_kind: BreakKind,
    pub(crate) available: Option<f64>,
    pub(crate) indent: f64,
    pub(crate) alignment: Alignment,
}

/// A finalized line keeps the exact preparation and its independently produced
/// line-local rows. These rows never mutate preliminary preparation tables.
pub(crate) struct FinalizedLine {
    pub(crate) prepared: Arc<PreparedData>,
    pub(crate) selection: LineSelection,
    pub(crate) data: layout::data::FinalizedLineData,
}

/// One retained outer placement, kept separate from canonical glyph geometry.
pub(crate) struct PlacedLine {
    pub(crate) finalized: FinalizedLine,
    pub(crate) origin: Point,
}

pub(crate) fn analyze(
    source: crate::ValidatedSource,
    default_style: crate::ValidatedStyle,
) -> crate::Result<Arc<AnalyzedData>> {
    let projection = SourceProjection::new(source.source())?;
    let styles = resolve::analyze_styles(&source, &default_style);
    let analysis = analysis::analyze(&projection, &styles, source.source().boxes())?;
    Ok(Arc::new(AnalyzedData {
        source: source.source().clone(),
        default_style: default_style.authored().clone(),
        projection,
        styles,
        analysis,
    }))
}

pub(crate) fn prepare(
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: Arc<AnalyzedData>,
    options: crate::ValidatedOptions,
    scratch: &mut shape::ShapeContext,
) -> crate::Result<Arc<PreparedData>> {
    let styles = resolve::resolve(
        catalogue.collection_mut(),
        &analyzed.styles,
        options.authored().scale,
    )?;
    let font_catalogue = catalogue.identity().clone();
    let shaped = shape::shape(
        catalogue.query(),
        ShapingInput {
            projection: &analyzed.projection,
            analysis: &analyzed.analysis,
            styles: &styles,
            boxes: analyzed.source.boxes(),
            font_catalogue: &font_catalogue,
        },
        scratch,
    )?;
    Ok(Arc::new(PreparedData {
        analyzed,
        font_catalogue,
        styles,
        shaped,
        quantize: options.authored().quantize,
    }))
}

//! Owned preparation and line-operation boundaries for the standalone engine.
//!
//! Source coordinates are UTF-16. Glyph-buffer byte ranges are private and are
//! always accompanied by their original-source association.

use std::{ops::Range as IndexRange, sync::Arc};

use crate::projection::SourceProjection;
use crate::protocol::{
    PreparationOptions,
    paragraph::ParagraphData,
    prepared::{PreparedAlternativeShape, PreparedTables, SelectedMapping},
    selection::SelectedData,
};
use crate::{Alignment, FontCatalogueIdentity, InlineBox, Point, Range};

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
    pub(crate) processed: Range,
    pub(crate) presentation: Range,
    pub(crate) glyph_bytes: IndexRange<usize>,
    pub(crate) code_point: u32,
    pub(crate) glyph: char,
}

/// Font-independent original-source analysis, retained by every later phase.
pub(crate) struct AnalyzedData {
    pub(crate) processed: Arc<crate::ProcessedSource>,
    pub(crate) paragraph: ParagraphData,
    pub(crate) host_projection: SourceProjection,
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
    /// Actual resolved base-style row retained for the native font-strut policy.
    pub(crate) default_style_index: Option<usize>,
    pub(crate) shaped: layout::data::ShapedData,
    pub(crate) tables: PreparedTables,
    pub(crate) alternative_shapes: Vec<PreparedAlternativeShape>,
}

/// Inputs coupled to one shaping operation, without a mutable System escape.
pub(crate) struct ShapingInput<'a> {
    pub(crate) projection: &'a SourceProjection,
    pub(crate) analysis: &'a analysis::Analysis,
    pub(crate) styles: &'a [resolve::ResolvedStyleRun],
    pub(crate) boxes: &'a [InlineBox],
    pub(crate) font_catalogue: &'a FontCatalogueIdentity,
    pub(crate) paragraph: &'a ParagraphData,
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
    pub(crate) selected: Arc<SelectedData>,
    pub(crate) mapping: Arc<SelectedMapping>,
    pub(crate) data: layout::data::FinalizedLineData,
}

/// One retained outer placement, kept separate from canonical glyph geometry.
pub(crate) struct PlacedLine {
    pub(crate) finalized: Arc<FinalizedLine>,
    pub(crate) anchor: Point,
    pub(crate) atomics: Vec<crate::AtomicPlacement>,
}

pub(crate) fn analyze(
    source: crate::ValidatedSource,
    default_style: crate::ValidatedStyle,
) -> crate::Result<Arc<AnalyzedData>> {
    let paragraph = ParagraphData::standalone(source.source());
    let processed =
        crate::processing::identity(source.source().clone(), default_style.authored().clone())?;
    analyze_paragraph(Arc::new(processed), paragraph)
}

pub(crate) fn analyze_paragraph(
    processed: Arc<crate::ProcessedSource>,
    paragraph: ParagraphData,
) -> crate::Result<Arc<AnalyzedData>> {
    let (source, default_style) = if paragraph.base_direction.is_some() {
        (
            crate::ValidatedSource::for_processed(processed.source().clone())?,
            crate::ValidatedStyle::for_processed(processed.base_style().clone())?,
        )
    } else {
        (
            crate::ValidatedSource::try_from(processed.source().clone())?,
            crate::ValidatedStyle::try_from(processed.base_style().clone())?,
        )
    };
    let host_projection = SourceProjection::host(processed.source(), &paragraph)?;
    let projection = SourceProjection::from_processed(&processed, &paragraph)?;
    let styles = resolve::analyze_styles(&source, &default_style);
    let analysis = analysis::analyze(&projection, &styles, processed.source().boxes(), &paragraph)?;
    Ok(Arc::new(AnalyzedData {
        processed,
        paragraph,
        host_projection,
        projection,
        styles,
        analysis,
    }))
}

/// Actual producer front; composed preparation retains its requested paragraph
/// strut, while the standalone convenience preserves its existing adjacent style.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum PreparationProfile {
    Native,
    Standalone,
}

pub(crate) fn prepare(
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: Arc<AnalyzedData>,
    options: PreparationOptions,
    profile: PreparationProfile,
    scratch: &mut shape::ShapeContext,
) -> crate::Result<Arc<PreparedData>> {
    let mut styles = resolve::resolve(
        catalogue.collection_mut(),
        &analyzed.styles,
        options.scale(),
    )?;
    let default_style_index = if profile == PreparationProfile::Native
        && analyzed.paragraph.paragraph_strut == crate::ParagraphStrut::DefaultFont
    {
        let base = crate::ValidatedStyle::for_processed(analyzed.processed.base_style().clone())?;
        let index = styles.len();
        styles.push(resolve::resolve_default(
            catalogue.collection_mut(),
            &base,
            options.scale(),
        )?);
        Some(index)
    } else {
        None
    };
    let font_catalogue = catalogue.identity().clone();
    let mut shaped = shape::shape(
        catalogue.query(),
        ShapingInput {
            projection: &analyzed.projection,
            analysis: &analyzed.analysis,
            styles: &styles,
            boxes: analyzed.processed.source().boxes(),
            font_catalogue: &font_catalogue,
            paragraph: &analyzed.paragraph,
        },
        scratch,
    )?;
    let metrics_index = default_style_index.or_else(|| {
        (profile == PreparationProfile::Standalone
            && analyzed.projection.atoms.is_empty()
            && analyzed.processed.source().boxes().is_empty())
        .then_some(0)
    });
    if let Some(index) = metrics_index {
        shaped.empty_run = Some(shape::default_metrics(
            catalogue.query(),
            &ShapingInput {
                projection: &analyzed.projection,
                analysis: &analyzed.analysis,
                styles: &styles,
                boxes: analyzed.processed.source().boxes(),
                font_catalogue: &font_catalogue,
                paragraph: &analyzed.paragraph,
            },
            index,
            scratch,
        )?);
    }
    let tables = crate::protocol::prepared::prepare_tables(&analyzed, &shaped)?;
    let alternative_shapes = crate::protocol::prepared::prepare_alternatives(
        catalogue, &analyzed, &styles, &tables, scratch,
    )?;
    Ok(Arc::new(PreparedData {
        analyzed,
        font_catalogue,
        styles,
        default_style_index,
        shaped,
        tables,
        alternative_shapes,
    }))
}

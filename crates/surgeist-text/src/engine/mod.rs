//! Owned preparation and line-operation boundaries for the standalone engine.
//!
//! Source coordinates are UTF-16. Glyph-buffer byte ranges are private and are
//! always accompanied by their original-source association.

use std::{ops::Range as IndexRange, sync::Arc};

use crate::projection::SourceProjection;
use crate::protocol::{
    PreparationOptions,
    paragraph::ParagraphData,
    prepared::{PreparedIndicationFonts, PreparedTables, SelectedMapping},
    selection::SelectedData,
};
use crate::{FontCatalogueIdentity, InlineBox, Point, Range};

pub(crate) mod analysis;
pub(crate) mod bidi;
mod convert;
pub(crate) mod editing;
pub(crate) mod layout;
mod lru_cache;
pub(crate) mod resolve;
pub(crate) mod shape;

#[cfg(test)]
mod tests;

pub(crate) use crate::error::{shaping_refused, text_numeric_error};

/// One semantic scalar with exact original, Phase-I and presentation associations.
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
    pub(crate) core: Arc<AnalyzedCore>,
    pub(crate) wrapping: Vec<Option<crate::Wrap>>,
}

impl std::ops::Deref for AnalyzedData {
    type Target = AnalyzedCore;
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

/// Executed source/Unicode data shared by distinct immutable wrapping contexts.
pub(crate) struct AnalyzedCore {
    pub(crate) processed: Arc<crate::ProcessedSource>,
    pub(crate) paragraph: ParagraphData,
    pub(crate) host_projection: SourceProjection,
    pub(crate) projection: SourceProjection,
    pub(crate) transform: Arc<crate::transform::relation::TransformMap>,
    pub(crate) styles: Vec<resolve::AnalyzedStyleRun>,
    pub(crate) analysis: analysis::Analysis,
    pub(crate) logical: analysis::wrapping::LogicalParagraph,
    pub(crate) words: analysis::hyphenation::Words,
}

/// The immutable, resource-retaining result of actual preparation.
///
/// Source and its projection live here once. Finalized lines retain this same
/// allocation; neither selection nor placement reconstructs authored input.
pub(crate) struct PreparedData {
    pub(crate) options: PreparationOptions,
    pub(crate) tab_metrics: Vec<Option<crate::tabs::ResolvedTab>>,
    pub(crate) analyzed: Arc<AnalyzedData>,
    pub(crate) font_catalogue: FontCatalogueIdentity,
    pub(crate) styles: Vec<resolve::ResolvedStyleRun>,
    /// Actual resolved base-style row retained for the native font-strut policy.
    pub(crate) default_style_index: Option<usize>,
    pub(crate) shaped: layout::data::ShapedData,
    pub(crate) tables: PreparedTables,
    pub(crate) indication_fonts: Vec<PreparedIndicationFonts>,
    pub(crate) joining_domains: Vec<shape::JoiningDomain>,
}

/// Inputs coupled to one shaping operation, without a mutable System escape.
pub(crate) struct ShapingInput<'a> {
    pub(crate) projection: &'a SourceProjection,
    pub(crate) analysis: &'a analysis::Analysis,
    pub(crate) styles: &'a [resolve::ResolvedStyleRun],
    pub(crate) boxes: &'a [InlineBox],
    pub(crate) font_catalogue: &'a FontCatalogueIdentity,
    pub(crate) paragraph: &'a ParagraphData,
    pub(crate) css_tabs: bool,
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
    let (span_styles, default_style) = if paragraph.base_direction.is_some() {
        (
            crate::source_model::processed_span_styles(
                processed.source(),
                paragraph.coverage.as_deref(),
            )?,
            crate::ValidatedStyle::for_processed(
                paragraph
                    .domain_style
                    .as_ref()
                    .unwrap_or(processed.base_style())
                    .clone(),
            )?,
        )
    } else {
        (
            crate::ValidatedSource::try_from(processed.source().clone())?
                .span_styles()
                .to_vec(),
            crate::ValidatedStyle::try_from(processed.base_style().clone())?,
        )
    };
    let mut host_projection = SourceProjection::host(processed.source(), &paragraph)?;
    if let Some(coverage) = &paragraph.coverage {
        host_projection.graphemes.retain(|at| {
            coverage
                .iter()
                .any(|piece| piece.start <= *at && *at <= piece.end)
        });
        host_projection.graphemes.extend(
            paragraph
                .intersections(paragraph.extent(&processed))
                .iter()
                .flat_map(|piece| [piece.start, piece.end]),
        );
        host_projection.graphemes.extend([
            paragraph.extent(&processed).start,
            paragraph.extent(&processed).end,
        ]);
        host_projection.graphemes.sort_unstable();
        host_projection.graphemes.dedup();
    }
    let projection = SourceProjection::from_processed(&processed, &paragraph)?;
    let mut styles =
        resolve::analyze_styles(processed.source(), &span_styles, &default_style, &paragraph);
    for (index, control) in paragraph.controls.iter().enumerate() {
        if let crate::ParagraphControlKind::DiscretionaryBreak { style }
        | crate::ParagraphControlKind::Wbr { style } = &control.kind
        {
            styles.push(resolve::AnalyzedStyleRun {
                owner: resolve::AnalyzedStyleOwner::Control(index),
                style: crate::ValidatedStyle::for_processed((**style).clone())?,
            });
        }
    }
    let (projection, transform) =
        crate::transform::apply(projection, &mut styles, &processed, &paragraph)?;
    let (analysis, words) =
        analysis::analyze(&projection, &styles, processed.source().boxes(), &paragraph)?;
    let words = analysis::hyphenation::Words::new(&projection, &analysis.graphemes, words);
    let logical = analysis::wrapping::LogicalParagraph::new(
        &processed,
        &paragraph,
        &projection,
        &styles,
        &analysis,
    )?;
    let wrapping = logical
        .relations
        .iter()
        .map(|relation| relation.captured_wrap)
        .collect();
    Ok(Arc::new(AnalyzedData {
        core: Arc::new(AnalyzedCore {
            processed,
            paragraph,
            host_projection,
            projection,
            transform,
            styles,
            analysis,
            logical,
            words,
        }),
        wrapping,
    }))
}

/// Actual producer front; composed preparation retains its requested paragraph
/// strut, while the standalone convenience preserves its existing adjacent style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
        let base = crate::ValidatedStyle::for_processed(
            analyzed
                .paragraph
                .domain_style
                .as_ref()
                .unwrap_or(analyzed.processed.base_style())
                .clone(),
        )?;
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
            css_tabs: options.tabs() == crate::TabPreparation::CssStops,
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
                css_tabs: options.tabs() == crate::TabPreparation::CssStops,
            },
            index,
            scratch,
        )?);
    }
    let tables =
        crate::protocol::prepared::prepare_tables(catalogue, &styles, &analyzed, &shaped, profile)?;
    let indication_fonts = crate::protocol::prepared::prepare_indication_fonts(
        catalogue,
        &analyzed,
        &styles,
        &tables,
        scratch,
        options.tabs() == crate::TabPreparation::CssStops,
    )?;
    let joining_domains = shape::prepare_joining_domains(&analyzed, &shaped, &indication_fonts)?;
    let tab_metrics = crate::tabs::prepare(
        catalogue,
        &analyzed,
        &styles,
        &indication_fonts,
        options,
        scratch,
    )?;
    Ok(Arc::new(PreparedData {
        options,
        tab_metrics,
        analyzed,
        font_catalogue,
        styles,
        default_style_index,
        shaped,
        tables,
        indication_fonts,
        joining_domains,
    }))
}

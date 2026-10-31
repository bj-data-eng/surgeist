//! Owned preparation and line-operation boundaries for the standalone engine.
//!
//! Source coordinates are UTF-16. Glyph-buffer byte ranges are private and are
//! always accompanied by their original-source association.

use std::{ops::Range as IndexRange, sync::Arc};

use crate::projection::SourceProjection;
use crate::protocol::{
    PreparationOptions,
    paragraph::{ParagraphContext, ParagraphData},
    prepared::{PreparedIndicationFonts, PreparedTables, SelectedMapping},
    selection::SelectedData,
};
use crate::{AnalysisTypography, FontCatalogueIdentity, InlineBox, Point, Range};

pub(crate) mod analysis;
pub(crate) mod bidi;
mod convert;
pub(crate) mod editing;
pub(crate) mod layout;
mod lru_cache;
pub(crate) mod resolve;
pub(crate) mod scope_alignment;
pub(crate) mod shape;
pub(crate) mod spacing;
pub(crate) mod upright;

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
    pub(crate) typography: AnalysisTypography,
    pub(crate) scope_cuts: std::sync::OnceLock<
        std::result::Result<analysis::scope::ScopeCutIndex, analysis::scope::ScopeIndexCapacity>,
    >,
    pub(crate) processed: Arc<crate::ProcessedSource>,
    pub(crate) paragraph: ParagraphContext,
    pub(crate) sections: Vec<analysis::structural::Section>,
    pub(crate) combination: Option<Arc<crate::transform::combined::CombinedOutput>>,
    pub(crate) host_projection: SourceProjection,
    pub(crate) projection: SourceProjection,
    pub(crate) transform: Arc<crate::transform::relation::TransformMap>,
    pub(crate) styles: Vec<resolve::AnalyzedStyleRun>,
    pub(crate) analysis: analysis::Analysis,
    pub(crate) logical: analysis::wrapping::LogicalParagraph,
    pub(crate) direction: analysis::direction::DirectionIndex,
    pub(crate) words: analysis::hyphenation::Words,
}

/// The immutable, resource-retaining result of actual preparation.
///
/// Source and its projection live here once. Finalized lines retain this same
/// allocation; neither selection nor placement reconstructs authored input.
pub(crate) struct PreparedData {
    pub(crate) options: PreparationOptions,
    pub(crate) scope_alignment: scope_alignment::PreparedScopes,
    pub(crate) tab_metrics: Vec<Option<crate::tabs::ResolvedTab>>,
    pub(crate) analyzed: Arc<AnalyzedData>,
    pub(crate) font_catalogue: FontCatalogueIdentity,
    pub(crate) styles: Vec<resolve::ResolvedStyleRun>,
    /// Actual resolved base-style row retained for the native font-strut policy.
    pub(crate) default_style_index: Option<usize>,
    pub(crate) shaped: layout::data::ShapedData,
    pub(crate) vertical_units: Vec<analysis::upright::VerticalUnit>,
    pub(crate) tables: PreparedTables,
    pub(crate) indication_fonts: Vec<PreparedIndicationFonts>,
}

/// Inputs coupled to one shaping operation, without a mutable System escape.
pub(crate) struct ShapingInput<'a> {
    pub(crate) formatting_breaks: &'a [usize],
    pub(crate) scope_custody: Option<scope_alignment::CustodyInput<'a>>,
    pub(crate) separator_geometry: bool,
    pub(crate) characters: Option<&'a spacing::character::PlanRequirements>,
    pub(crate) profile: upright::ShapingProfile<'a>,
    pub(crate) baseline_reference: Option<crate::FontBaseline>,
    pub(crate) projection: &'a SourceProjection,
    pub(crate) analysis: &'a analysis::Analysis,
    pub(crate) styles: &'a [resolve::ResolvedStyleRun],
    pub(crate) boxes: &'a [InlineBox],
    pub(crate) font_catalogue: &'a FontCatalogueIdentity,
    pub(crate) paragraph: &'a ParagraphContext,
    pub(crate) css_tabs: bool,
}

/// A finalized line keeps the exact preparation and its independently produced
/// line-local rows. These rows never mutate preliminary preparation tables.
pub(crate) struct FinalizedLine {
    pub(crate) selected: Arc<SelectedData>,
    pub(crate) data: layout::data::FinalizedLineData,
}

/// Actual selected provider output before any allocation-dependent operation.
pub(crate) struct SelectedShape {
    pub(crate) mapping: Arc<SelectedMapping>,
    pub(crate) shaped: layout::data::ShapedData,
    pub(crate) characters: spacing::character::Graph,
    pub(crate) order: Vec<usize>,
}

/// One retained outer placement, kept separate from canonical glyph geometry.
pub(crate) struct PlacedLine {
    pub(crate) finalized: Arc<FinalizedLine>,
    pub(crate) anchor: Point,
    pub(crate) orientation: crate::LineOrientation,
    pub(crate) atomics: Vec<crate::AtomicPlacement>,
}

pub(crate) fn analyze(
    source: crate::ValidatedSource,
    default_style: crate::ValidatedStyle,
    typography: AnalysisTypography,
) -> crate::Result<Arc<AnalyzedData>> {
    let paragraph = ParagraphData::standalone(source.source());
    let processed =
        crate::processing::identity(source.source().clone(), default_style.authored().clone())?;
    analyze_paragraph(Arc::new(processed), paragraph, typography)
}

pub(crate) fn analyze_paragraph(
    processed: Arc<crate::ProcessedSource>,
    paragraph: ParagraphData,
    typography: AnalysisTypography,
) -> crate::Result<Arc<AnalyzedData>> {
    // The captured paragraph base is already a used authority. Computed RTL
    // belongs only to the separately retained typography input. The nearest
    // block's tab reference is independent of paragraph bidi.
    if typography.is_upright()
        && paragraph.base_direction == Some(crate::ParagraphBaseDirection::RightToLeft)
    {
        return Err(crate::protocol::protocol_error(
            crate::ProtocolError::MismatchedContext {
                field: crate::ProtocolContextField::BaseDirection,
            },
        ));
    }
    let (span_styles, default_style) = admitted_output_styles(&processed, &paragraph)?;
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
    let mut styles = output_style_runs(&processed, &paragraph, &span_styles, &default_style)?;
    let crate::transform::CompletedTransform {
        projection,
        relation: transform,
        combination,
    } = crate::transform::apply(projection, &mut styles, &processed, &paragraph)?;
    complete_analysis(
        processed,
        typography,
        CompletedPresentation {
            paragraph: paragraph.context,
            host_projection,
            projection,
            transform,
            styles,
            combination,
        },
    )
}

pub(crate) fn admitted_output_styles(
    processed: &crate::ProcessedSource,
    paragraph: &ParagraphContext,
) -> crate::Result<(
    Vec<crate::source_model::ValidatedSpan>,
    crate::ValidatedStyle,
)> {
    if paragraph.base_direction.is_some() {
        Ok((
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
        ))
    } else {
        Ok((
            crate::ValidatedSource::try_from(processed.source().clone())?
                .span_styles()
                .to_vec(),
            crate::ValidatedStyle::try_from(processed.base_style().clone())?,
        ))
    }
}

pub(crate) fn output_style_runs(
    processed: &crate::ProcessedSource,
    paragraph: &ParagraphContext,
    spans: &[crate::source_model::ValidatedSpan],
    default: &crate::ValidatedStyle,
) -> crate::Result<Vec<resolve::AnalyzedStyleRun>> {
    let mut styles = resolve::analyze_styles(processed.source(), spans, default, paragraph);
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
    Ok(styles)
}

/// Completed canonical presentation, ready for the sole Unicode analyzer.
pub(crate) struct CompletedPresentation {
    pub(crate) paragraph: ParagraphContext,
    pub(crate) host_projection: SourceProjection,
    pub(crate) projection: SourceProjection,
    pub(crate) transform: Arc<crate::transform::relation::TransformMap>,
    pub(crate) styles: Vec<resolve::AnalyzedStyleRun>,
    pub(crate) combination: Option<Arc<crate::transform::combined::CombinedOutput>>,
}
pub(crate) fn complete_analysis(
    processed: Arc<crate::ProcessedSource>,
    typography: AnalysisTypography,
    presentation: CompletedPresentation,
) -> crate::Result<Arc<AnalyzedData>> {
    let CompletedPresentation {
        paragraph,
        host_projection,
        projection,
        transform,
        styles,
        combination,
    } = presentation;
    let sections = analysis::structural::sections(&paragraph, &transform, &projection, &processed)?;
    let logical_events = transform.logical_events(&paragraph, processed.source());
    let (analysis, words, direction) = analysis::analyze(
        &projection,
        &styles,
        processed.source().boxes(),
        &paragraph,
        typography,
        analysis::BidiScope::Completed {
            sections: &sections,
            combination: combination.as_deref(),
            events: &logical_events,
        },
    )?;
    let words = analysis::hyphenation::Words::new(&projection, &analysis.graphemes, words);
    let logical = analysis::wrapping::LogicalParagraph::new(
        &processed,
        &paragraph,
        &projection,
        &styles,
        &analysis,
        analysis::wrapping::LogicalInput {
            events: logical_events,
            combination: combination.as_deref(),
        },
    )?;
    let logical_sections =
        analysis::storage::collect(sections.iter().map(|section| section.participants.clone()))?;
    let direction = direction.bind(&logical.events, &logical_sections, combination.as_deref())?;
    let wrapping = logical
        .relations
        .iter()
        .map(|relation| relation.captured_wrap)
        .collect();
    Ok(Arc::new(AnalyzedData {
        core: Arc::new(AnalyzedCore {
            typography,
            scope_cuts: std::sync::OnceLock::new(),
            processed,
            paragraph,
            sections,
            combination,
            host_projection,
            projection,
            transform,
            styles,
            analysis,
            logical,
            direction,
            words,
        }),
        wrapping,
    }))
}

pub(crate) fn analyze_domain(
    processed: Arc<crate::ProcessedSource>,
    member: crate::ProcessedParagraph,
    typography: AnalysisTypography,
) -> crate::Result<crate::AnalyzedDomain> {
    crate::transform::domain::analyze(processed, member.data(), typography)
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
    let has_cells = analyzed
        .combination
        .as_ref()
        .is_some_and(|output| !output.cells.is_empty());
    shape::admit_cells(&analyzed, options.tabs())?;
    if has_cells && options.orientation() != crate::LineOrientation::Vertical {
        return Err(crate::transform::combined::refused(
            crate::TextCombineError::UnsupportedCellPreparation,
        ));
    }

    if (analyzed.typography.is_upright() || analyzed.typography == AnalysisTypography::Mixed)
        && options.orientation() != crate::LineOrientation::Vertical
    {
        return Err(crate::protocol::protocol_error(
            crate::ProtocolError::MismatchedContext {
                field: crate::ProtocolContextField::Orientation,
            },
        ));
    }
    if options.orientation() == crate::LineOrientation::Vertical
        && !analyzed.typography.is_upright()
        && analyzed.typography != AnalysisTypography::Mixed
    {
        return Err(crate::protocol::protocol_error(
            crate::ProtocolError::UnsupportedContext {
                field: crate::ProtocolContextField::Orientation,
            },
        ));
    }
    if !matches!(
        (
            options.orientation(),
            options.requested_baseline_reference()
        ),
        (
            crate::LineOrientation::Vertical,
            None | Some(crate::FontBaseline::Central)
        ) | (
            crate::LineOrientation::Horizontal
                | crate::LineOrientation::SidewaysClockwise
                | crate::LineOrientation::SidewaysCounterclockwise,
            None | Some(crate::FontBaseline::Alphabetic)
        )
    ) {
        return Err(crate::protocol::protocol_error(
            crate::ProtocolError::UnsupportedContext {
                field: crate::ProtocolContextField::BaselineReference,
            },
        ));
    }
    let upright_units = upright::units(
        &analyzed,
        &analyzed.projection,
        &analyzed.analysis,
        &upright::UprightProvenance::Original,
        options.orientation(),
    )?;
    let (mut scope_alignment, mut scope_budget) = scope_alignment::plan(&analyzed)?;
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
    scope_alignment::prepare_cell_resources(
        &mut scope_alignment,
        catalogue,
        &analyzed,
        options,
        &mut scope_budget,
    )?;
    let font_catalogue = catalogue.identity().clone();
    let characters = spacing::character::plan(
        spacing::character::Input::original(&analyzed, &styles, options.scale()),
        false,
        &mut scope_budget,
    )?;
    let mut shaped = shape::prepare(
        catalogue,
        ShapingInput {
            formatting_breaks: &scope_alignment.breaks,
            scope_custody: scope_alignment.custody(&analyzed),
            characters: Some(&characters),
            separator_geometry: spacing::authored_demand(&analyzed.analysis, &styles),
            profile: upright::original_profile(
                options.orientation(),
                &upright_units,
                analyzed.typography,
                &analyzed.styles,
            ),
            baseline_reference: options.resolved_baseline_reference(),
            projection: &analyzed.projection,
            analysis: &analyzed.analysis,
            styles: &styles,
            boxes: analyzed.processed.source().boxes(),
            font_catalogue: &font_catalogue,
            paragraph: &analyzed.paragraph,
            css_tabs: options.tabs() == crate::TabPreparation::CssStops,
        },
        scratch,
        &analyzed,
        &scope_alignment,
        &mut scope_budget,
    )?;
    shaped.character_requirements = Some(characters);
    spacing::authored_original(&analyzed, &styles, &mut shaped)?;
    let characters = spacing::character::build(
        spacing::character::Input::original(&analyzed, &styles, options.scale()),
        false,
        Some(&shaped.character_exclusions),
        &mut scope_budget,
    )?;
    spacing::character::original(&analyzed, &characters, &mut shaped, &mut scope_budget)?;
    characters.drop_scratch(&mut scope_budget)?;
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
                formatting_breaks: &scope_alignment.breaks,
                scope_custody: scope_alignment.custody(&analyzed),
                characters: None,
                separator_geometry: spacing::authored_demand(&analyzed.analysis, &styles),
                profile: upright::original_profile(
                    options.orientation(),
                    &upright_units,
                    analyzed.typography,
                    &analyzed.styles,
                ),
                baseline_reference: options.resolved_baseline_reference(),
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
    let tables = crate::protocol::prepared::prepare_tables(
        catalogue,
        &styles,
        &analyzed,
        &shaped,
        profile,
        &mut scope_budget,
    )?;
    let indication_fonts = crate::protocol::prepared::prepare_indication_fonts(
        catalogue, &analyzed, &styles, &tables, scratch, options,
    )?;
    let tab_metrics = crate::tabs::prepare(
        catalogue,
        &analyzed,
        &styles,
        &indication_fonts,
        options,
        scratch,
    )?;
    scope_alignment::prepare_resources(
        &mut scope_alignment,
        catalogue,
        &analyzed,
        options,
        &shaped,
        &tables,
        scope_budget,
    )?;
    Ok(Arc::new(PreparedData {
        options,
        scope_alignment,
        tab_metrics,
        analyzed,
        font_catalogue,
        styles,
        default_style_index,
        vertical_units: upright_units,
        shaped,
        tables,
        indication_fonts,
    }))
}

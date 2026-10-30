// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Owned rows adapted from Parley 0.9 `layout/data.rs` (SHA-256
//! f994c087a98d65b168fb5eca76102730eb6e0a5768297186075df56f4ebce2a1).
//! Preparation is immutable. Selected lines own their adjustments and positions;
//! indices and source extents are not compressed into provider encodings.

use std::{ops::Range as IndexRange, sync::Arc};

use crate::{FontData, FontSynthesis, Range, Rect};

use super::super::{PlacedLine, PreparedData, analysis::Boundary};

/// Canonical provider glyph offsets and advance, with its authentic source unit extent.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GlyphData {
    pub(crate) id: u32,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) advance: f32,
    pub(crate) source: Range,
    pub(crate) style: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Whitespace {
    None,
    Space,
    NoBreakSpace,
    Tab,
    Newline,
}

impl Whitespace {
    pub(crate) const fn is_space_or_nbsp(self) -> bool {
        matches!(self, Self::Space | Self::NoBreakSpace)
    }
}

/// Same-instance logical cross-axis metrics. The signed distances are from the
/// line reference toward line-over and line-under respectively.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RunMetrics {
    pub(crate) ascent: f64,
    pub(crate) descent: f64,
    /// Initial run-style/default strut height; actual components retain their own.
    pub(crate) line_height: f64,
    pub(crate) basis: RunMetricBasis,
}

impl RunMetrics {
    /// Font-design-zero origin relative to the shared line reference, toward
    /// line-under. Upright glyphs already use their separate Central basis.
    pub(crate) const fn font_origin_offset(&self) -> f64 {
        match self.basis {
            RunMetricBasis::Horizontal(metrics) => metrics.font_origin_offset,
            RunMetricBasis::Upright { .. } => 0.0,
        }
    }
    pub(crate) const fn horizontal(&self) -> Option<&HorizontalRunMetrics> {
        match &self.basis {
            RunMetricBasis::Horizontal(metrics) => Some(metrics),
            RunMetricBasis::Upright { .. } => None,
        }
    }
}

/// Horizontal-only font facts cannot masquerade as upright cross geometry.
#[derive(Clone, Copy, Debug)]
pub(crate) enum RunMetricBasis {
    Horizontal(HorizontalRunMetrics),
    Upright {
        /// Present only when an actual component demands MetricsRelative.
        normal_extent_design: Option<f64>,
        design_scale: f64,
    },
}

impl Default for RunMetricBasis {
    fn default() -> Self {
        Self::Horizontal(HorizontalRunMetrics::default())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct HorizontalRunMetrics {
    /// Signed local line-under displacement from the shared reference to the
    /// actual font-design-zero paint origin.
    pub(crate) font_origin_offset: f64,
    /// Distance from the prepared reference to the underline's top, positive upward.
    pub(crate) underline_top: f64,
    pub(crate) underline_thickness: f64,
    /// Distance from the prepared reference to the strike center, positive upward.
    pub(crate) strike_center: f64,
    pub(crate) strike_thickness: f64,
}

/// One resolved and actually shaped face/instance. Resources remain strong, not copied bytes.
#[derive(Clone, Debug)]
pub(crate) struct PreparedRun {
    pub(crate) font: FontData,
    pub(crate) font_size: f32,
    pub(crate) font_attrs: fontique::Attributes,
    pub(crate) synthesis: FontSynthesis,
    pub(crate) coords: Vec<i16>,
    pub(crate) style: usize,
    pub(crate) script: icu_properties::props::Script,
    pub(crate) level: u8,
    pub(crate) atoms: IndexRange<usize>,
    /// Actual outline axes, independent of the placed line's inline frame.
    pub(crate) glyph_basis: crate::GlyphBasis,
    pub(crate) metrics: RunMetrics,
}

/// Actual font-backed provider output or a glyph-free CSS positioning control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClusterContent {
    Provider(usize),
    CssTab(Option<usize>),
}
impl ClusterContent {
    pub(crate) fn font_run(self) -> Option<usize> {
        match self {
            Self::Provider(i) => Some(i),
            Self::CssTab(i) => i,
        }
    }
}

/// A logical shaped cluster. Ligature components retain separate source associations.
#[derive(Clone, Debug)]
pub(crate) struct PreparedCluster {
    /// Owning component style resolved against its actual font instance.
    /// Font-free controls have no strut contribution.
    pub(crate) line_height: f64,
    /// Actual provider contribution before caller-owned spacing.
    pub(crate) shaped_advance: f64,
    /// Executed letter, word, and selected allocation terms, without regrouping.
    pub(crate) logical_adjustments: [f64; 3],
    pub(crate) source: Range,
    pub(crate) atoms: IndexRange<usize>,
    /// Actual provider group before scalar component extraction.
    pub(crate) provider_group: IndexRange<usize>,
    /// Actual invocation-qualified extraction receipt when separator geometry is demanded.
    pub(crate) provider_receipt: Option<usize>,
    pub(crate) glyphs: IndexRange<usize>,
    pub(crate) content: ClusterContent,
    pub(crate) style: usize,
    pub(crate) level: u8,
    pub(crate) advance: f64,
    pub(crate) boundary: Boundary,
    pub(crate) whitespace: Whitespace,
    pub(crate) is_whitespace: bool,
    pub(crate) ligature_start: bool,
    pub(crate) ligature_component: bool,
}

/// Actual mixed logical order, including boxes with zero source extent.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ShapedItem {
    TextCluster(usize),
    Cell(usize),
    InlineBox {
        occurrence: usize,
        level: u8,
    },
    /// Actual zero-presentation selected unit; it owns no synthetic glyph/metrics.
    Anchor {
        unit: usize,
    },
}

/// Actual horizontal coordinates inside the cell square: x points right and
/// y points down. They are independent of the outer native line axes.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CellFrame {
    pub(crate) x_scale: f64,
    pub(crate) x_offset: f64,
    pub(crate) baseline: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CellClusterPosition {
    pub(crate) cluster: usize,
    pub(crate) pen: f64,
}

/// Provider input absence is source-proved; glyph/ink emptiness is not absence.
#[derive(Clone, Debug)]
pub(crate) enum CellPayload {
    NoProviderInput,
    Shaped {
        runs: IndexRange<usize>,
        clusters: IndexRange<usize>,
        glyphs: IndexRange<usize>,
        scope_relations: IndexRange<usize>,
        advance: IndexRange<f64>,
        positions: Vec<CellClusterPosition>,
        frame: CellFrame,
    },
}

/// One genuine source cell and its already executed horizontal normal payload.
#[derive(Clone, Debug)]
pub(crate) struct PreparedCell {
    /// Local ordinal in the analyzed output's actual CombinedOutput bindings.
    pub(crate) binding: usize,
    pub(crate) metrics: usize,
    pub(crate) level: u8,
    pub(crate) extent: f64,
    pub(crate) before: f64,
    pub(crate) after: f64,
    pub(crate) payload: CellPayload,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ShapedData {
    /// Feature-only custody appended at each actual non-residual glyph insertion;
    /// it follows the same extraction ordering and clone lifetime as glyphs.
    pub(crate) scope_glyph_custody: Vec<super::super::scope_alignment::GlyphCustody>,
    /// Actual extraction-time operand relations in captured frame domains.
    /// This arena follows the same native output lifetime as glyph custody.
    pub(crate) scope_relations: Vec<super::super::scope_alignment::ScopeRelation>,
    pub(crate) font_queries: Vec<super::super::shape::FontQueryWitness>,
    pub(crate) runs: Vec<PreparedRun>,
    pub(crate) clusters: Vec<PreparedCluster>,
    pub(crate) glyphs: Vec<GlyphData>,
    /// Invocation-qualified provider observations, before component apportioning.
    pub(crate) provider_groups: Vec<ProviderGroup>,
    pub(crate) character_gaps: Vec<super::super::spacing::character::Gap>,
    pub(crate) character_invocations: Vec<super::super::spacing::character::Invocation>,
    /// The same immutable upfront requirement used by this actual operation.
    pub(crate) character_requirements:
        Option<Arc<super::super::spacing::character::PlanRequirements>>,
    /// Complete invocation-mapped DI exclusions; absence means eligible input.
    pub(crate) character_exclusions:
        Arc<std::collections::BTreeMap<usize, super::super::spacing::character::Exclusion>>,
    pub(crate) atomic_gaps: OrdinalMap<f64>,
    pub(crate) word_spacing: Vec<super::super::spacing::WordSpacingUnit>,
    pub(crate) items: Vec<ShapedItem>,
    pub(crate) cells: Vec<PreparedCell>,
    /// An actual admitted default face with zero source/glyph extent, for empty metrics.
    pub(crate) empty_run: Option<PreparedRun>,
}

/// Sparse native occurrence index with visible owned buffer capacity. Its
/// sorted keys carry actual ordinals; equal metric values never supply identity.
#[derive(Clone, Debug)]
pub(crate) struct OrdinalMap<T> {
    pub(crate) rows: Vec<(usize, T)>,
}
impl<T> Default for OrdinalMap<T> {
    fn default() -> Self {
        Self { rows: Vec::new() }
    }
}
impl<T> OrdinalMap<T> {
    pub(crate) fn get(&self, key: &usize) -> Option<&T> {
        self.rows
            .binary_search_by_key(key, |row| row.0)
            .ok()
            .map(|index| &self.rows[index].1)
    }
    pub(crate) fn get_mut(&mut self, key: &usize) -> Option<&mut T> {
        self.rows
            .binary_search_by_key(key, |row| row.0)
            .ok()
            .map(|index| &mut self.rows[index].1)
    }
    pub(crate) fn insert(&mut self, key: usize, value: T) -> Option<T> {
        match self.rows.binary_search_by_key(&key, |row| row.0) {
            Ok(index) => Some(std::mem::replace(&mut self.rows[index].1, value)),
            Err(index) => {
                self.rows.insert(index, (key, value));
                None
            }
        }
    }
    pub(crate) fn clear(&mut self) {
        self.rows.clear();
    }
    pub(crate) fn values(&self) -> impl Iterator<Item = &T> {
        self.rows.iter().map(|row| &row.1)
    }
}
impl<T> std::ops::Index<&usize> for OrdinalMap<T> {
    type Output = T;
    fn index(&self, key: &usize) -> &T {
        self.get(key).expect("admitted native ordinal")
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProviderGroup {
    pub(crate) run: usize,
    pub(crate) atoms: IndexRange<usize>,
    pub(crate) glyphs: IndexRange<usize>,
    pub(crate) advance: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LineMetrics {
    pub(crate) ascent: f64,
    pub(crate) descent: f64,
    pub(crate) leading: f64,
    pub(crate) line_height: f64,
    pub(crate) baseline: f64,
    pub(crate) advance: f64,
    pub(crate) trailing_whitespace: f64,
    pub(crate) offset: f64,
    pub(crate) block_min: f64,
    pub(crate) block_max: f64,
    pub(crate) inline_min: f64,
    pub(crate) inline_max: f64,
    pub(crate) occupied_start: f64,
    pub(crate) occupied_end: f64,
    pub(crate) hanging_start: f64,
    pub(crate) hanging_end: f64,
}

/// One paint-order run in a finalized line. Glyph offsets and advances remain
/// canonical f32; logical run-local pens and the run origin remain f64.
#[derive(Clone, Debug)]
pub(crate) struct FinalizedRun {
    pub(crate) frame: FinalizedRunFrame,
    /// Executed common reference, distinct from the font-design paint origin.
    pub(crate) reference: Option<f64>,
    pub(crate) run: usize,
    pub(crate) style: usize,
    pub(crate) level: u8,
    pub(crate) source: Range,
    pub(crate) glyphs: IndexRange<usize>,
    pub(crate) x: f64,
    /// Checked font origin in local line-under coordinates; the common
    /// alignment reference remains in LineMetrics::baseline.
    pub(crate) y: f64,
    pub(crate) advance: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FinalizedRunFrame {
    Line,
    Cell(usize),
}

/// The actual outer square has already received its enclosing scope shift.
/// The retained shift records custody and must not be applied a second time.
#[derive(Clone, Debug)]
pub(crate) struct FinalizedCell {
    pub(crate) unit: usize,
    pub(crate) cell: usize,
    pub(crate) binding: usize,
    pub(crate) reference: f64,
    pub(crate) inline: IndexRange<f64>,
    pub(crate) under: IndexRange<f64>,
    pub(crate) scope_displacement: crate::BaselineShift,
    pub(crate) level: u8,
    pub(crate) frame: Option<CellFrame>,
}

/// One actual logical glyph position, separate from its canonical provider offsets.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FinalizedGlyph {
    pub(crate) glyph: GlyphData,
    /// Run-local logical pen. This includes actual finalized cluster spacing.
    pub(crate) pen: f64,
    /// Independent pre-half operand, never reconstructed from a rounded pen.
    pub(crate) pen_base: f64,
    pub(crate) word_half: f64,
}

/// Pre-half unit origin retained even when its actual glyph inventory is empty.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WordSpacingAnchor {
    pub(crate) base: crate::Point,
    pub(crate) half: f64,
}

#[derive(Clone, Debug)]
pub(crate) struct FinalizedCluster {
    pub(crate) reference: Option<f64>,
    /// The actually selected component's resolved font-backed line height.
    pub(crate) line_height: f64,
    /// Actual provider contribution before caller-owned spacing.
    pub(crate) shaped_advance: f64,
    /// Executed letter, word, and selected allocation terms, without regrouping.
    pub(crate) logical_adjustments: [f64; 3],
    /// Actual provider membership retained independently of operation-specific units.
    pub(crate) provider_atoms: IndexRange<usize>,
    pub(crate) disposition: crate::SpacingSlotDisposition,
    pub(crate) source: Range,
    pub(crate) presentation: Range,
    pub(crate) content: ClusterContent,
    #[cfg(feature = "text-accessibility")]
    pub(crate) style: usize,
    pub(crate) x: f64,
    pub(crate) advance: f64,
    pub(crate) level: u8,
    pub(crate) boundary: Boundary,
    pub(crate) whitespace: Whitespace,
}

#[derive(Clone, Debug)]
pub(crate) struct FinalizedBox {
    pub(crate) occurrence: usize,
    pub(crate) atomic: crate::protocol::PreparedAtomic,
    pub(crate) slot: crate::Point,
    pub(crate) scope_displacement: crate::BaselineShift,
    pub(crate) geometry: FinalizedBoxGeometry,
    pub(crate) following_gap: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FinalizedBoxGeometry {
    InFlow {
        flow: crate::protocol::AtomicFlowAllocation,
        box_geometry: crate::protocol::AtomicBoxAllocation,
        alignment: crate::protocol::LineBoxAlignment,
        rect: Rect,
    },
    OutOfFlow,
}

impl FinalizedBox {
    pub(crate) fn cell_advance(&self) -> f64 {
        self.flow_advance() + self.following_gap
    }
    pub(crate) fn flow_advance(&self) -> f64 {
        match self.geometry {
            FinalizedBoxGeometry::InFlow { flow, .. } => flow.advance,
            FinalizedBoxGeometry::OutOfFlow => 0.0,
        }
    }
}

/// Actual committed mixed L2 order, including zero-advance text and atomic gaps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FinalizedVisualItem {
    Cluster(usize),
    Cell(usize),
    InlineBox(usize),
    Anchor(usize),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FinalizedAnchor {
    pub(crate) reference: Option<f64>,
    pub(crate) unit: usize,
    pub(crate) slot: crate::Point,
    pub(crate) scope_displacement: crate::BaselineShift,
    pub(crate) level: u8,
    pub(crate) inline_end: f64,
}

/// Actual complete-unit feedback in selected-slot order, derived from its
/// retained component membership after finalization.
#[derive(Clone, Debug)]
pub(crate) struct FinalizedSpacingSlot {
    pub(crate) cluster: usize,
    pub(crate) edge: Option<usize>,
    pub(crate) members: Vec<usize>,
    pub(crate) geometry: crate::SpacingSlotGeometry,
    pub(crate) stretchability: crate::SpacingStretchability,
    pub(crate) traversal_order: usize,
    pub(crate) authored: f64,
    pub(crate) letter: f64,
    pub(crate) allocation: f64,
    pub(crate) advance: f64,
    pub(crate) disposition: crate::protocol::SpacingSlotDisposition,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FinalizedLineData {
    /// Preserved unallocated source-cell sizing policy, relative to a reference.
    pub(crate) scope_cell_band: Option<(f64, f64)>,
    pub(crate) scope_allocations: Vec<crate::ScopeFrameAllocation>,
    pub(crate) scope_retention: Option<crate::ScopeAlignmentRetention>,
    pub(crate) scope_usage: Option<crate::ScopeAlignmentUsage>,
    pub(crate) measurements: Option<crate::LineMeasurements>,
    pub(crate) font_runs: Vec<PreparedRun>,
    pub(crate) runs: Vec<FinalizedRun>,
    /// Logical cluster order; each row records its actual visual position.
    pub(crate) clusters: Vec<FinalizedCluster>,
    /// Paint-order canonical local glyphs, with original associations.
    pub(crate) glyphs: Vec<FinalizedGlyph>,
    pub(crate) word_anchors: Vec<WordSpacingAnchor>,
    pub(crate) character_anchors: Vec<WordSpacingAnchor>,
    pub(crate) boxes: Vec<FinalizedBox>,
    pub(crate) cells: Vec<FinalizedCell>,
    pub(crate) visual_order: Vec<FinalizedVisualItem>,
    pub(crate) anchors: Vec<FinalizedAnchor>,
    pub(crate) spacing_slots: Vec<FinalizedSpacingSlot>,
    /// Complete executed allocation amounts, before rounded cluster addition.
    /// Signed zero is canonicalized only for semantic identity.
    pub(crate) spacing_amounts: Vec<f64>,
    pub(crate) metrics: LineMetrics,
    pub(crate) prefix: crate::protocol::prefix::PrefixData,
    pub(crate) fragments: Vec<FinalizedFragmentData>,
}

impl FinalizedLineData {
    /// Visit actual owned output arenas; the enclosing FinalizedLine header
    /// already includes every embedded fixed header in this row.
    pub(crate) fn visit_scope_allocations(
        &self,
        visit: &mut crate::engine::scope_alignment::census::AllocationVisitor<'_>,
    ) -> crate::Result<()> {
        use crate::engine::scope_alignment::census::AllocationWitness as W;
        visit(W::buffer(
            &self.scope_allocations,
            self.scope_allocations.len(),
        )?)?;
        visit(W::buffer(&self.font_runs, self.font_runs.len())?)?;
        for run in &self.font_runs {
            visit(W::buffer(&run.coords, run.coords.len())?)?;
            visit(W::resource(run.font.bytes()))?;
        }
        visit(W::buffer(&self.runs, self.runs.len())?)?;
        visit(W::buffer(&self.clusters, self.clusters.len())?)?;
        visit(W::buffer(&self.glyphs, self.glyphs.len())?)?;
        visit(W::buffer(&self.word_anchors, self.word_anchors.len())?)?;
        visit(W::buffer(
            &self.character_anchors,
            self.character_anchors.len(),
        )?)?;
        visit(W::buffer(&self.boxes, self.boxes.len())?)?;
        visit(W::buffer(&self.cells, self.cells.len())?)?;
        visit(W::buffer(&self.visual_order, self.visual_order.len())?)?;
        visit(W::buffer(&self.anchors, self.anchors.len())?)?;
        visit(W::buffer(&self.spacing_slots, self.spacing_slots.len())?)?;
        for row in &self.spacing_slots {
            visit(W::buffer(&row.members, row.members.len())?)?;
        }
        visit(W::buffer(
            &self.spacing_amounts,
            self.spacing_amounts.len(),
        )?)?;
        visit(W::buffer(&self.fragments, self.fragments.len())?)?;
        if let Some(measurements) = &self.measurements {
            visit(W::buffer(&measurements.edges, measurements.edges.len())?)?;
        }
        self.prefix.visit_scope_allocations(visit)
    }

    /// Source-cell sizing is retained before scope allocation. Only its actual
    /// occurrence reference moves; final line extrema are not another cell size.
    pub(crate) fn cell_band(&self, reference: Option<f64>) -> (f64, f64) {
        match (self.scope_cell_band, reference) {
            (Some((before, after)), Some(reference)) => (reference - before, before + after),
            _ => (
                self.metrics.block_min,
                self.metrics.block_max - self.metrics.block_min,
            ),
        }
    }

    pub(crate) fn cell_rect(&self, inline: f64, advance: f64, reference: Option<f64>) -> Rect {
        let (top, height) = self.cell_band(reference);
        Rect::new(inline, top, advance, height)
    }
}

impl crate::engine::FinalizedLine {
    pub(crate) fn visit_scope_allocations(
        &self,
        visit: &mut crate::engine::scope_alignment::census::AllocationVisitor<'_>,
    ) -> crate::Result<()> {
        visit(crate::engine::scope_alignment::census::AllocationWitness::object(self, 0))?;
        self.data.visit_scope_allocations(visit)
    }
}

/// One authentic selected origin intersection and its actual local association geometry.
#[derive(Clone, Debug)]
pub(crate) struct FinalizedFragmentData {
    pub(crate) association: crate::protocol::paragraph::FragmentAssociation,
    pub(crate) bounds: Option<Rect>,
}

/// The committed standalone result; every line retains this same preparation allocation.
pub(crate) struct PlacedLayout {
    pub(crate) prepared: Arc<PreparedData>,
    pub(crate) lines: Vec<PlacedLine>,
    /// Authentic semantic sequence, including nonformatted correspondence.
    pub(crate) receipts: Vec<Arc<super::super::FinalizedLine>>,
    /// Selected-event cut for each visible line in that full sequence.
    pub(crate) line_cuts: Vec<usize>,
    pub(crate) nonformatted: Vec<(usize, crate::NonFormattedLinePlacement)>,
    pub(crate) bounds: Rect,
    pub(crate) coverage: crate::protocol::placement::PlacedCoverage,
    pub(crate) width: f64,
    pub(crate) full_width: f64,
    pub(crate) height: f64,
    pub(crate) interaction: super::super::editing::InteractionData,
}

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

/// Same-instance logical font metrics. Descent is a nonnegative distance below baseline.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RunMetrics {
    pub(crate) ascent: f64,
    pub(crate) descent: f64,
    pub(crate) line_height: f64,
    /// Distance from baseline to the underline's top, positive upward.
    pub(crate) underline_top: f64,
    pub(crate) underline_thickness: f64,
    /// Distance from baseline to the strike center, positive upward.
    pub(crate) strike_center: f64,
    pub(crate) strike_thickness: f64,
    pub(crate) x_height: f64,
    pub(crate) cap_height: f64,
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
    pub(crate) metrics: RunMetrics,
}

/// A logical shaped cluster. Ligature components retain separate source associations.
#[derive(Clone, Debug)]
pub(crate) struct PreparedCluster {
    pub(crate) source: Range,
    pub(crate) atoms: IndexRange<usize>,
    pub(crate) glyphs: IndexRange<usize>,
    pub(crate) run: usize,
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
    InlineBox {
        occurrence: usize,
        level: u8,
    },
    /// Actual zero-presentation selected unit; it owns no synthetic glyph/metrics.
    Anchor {
        unit: usize,
        level: u8,
    },
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ShapedData {
    pub(crate) runs: Vec<PreparedRun>,
    pub(crate) clusters: Vec<PreparedCluster>,
    pub(crate) glyphs: Vec<GlyphData>,
    pub(crate) items: Vec<ShapedItem>,
    /// An actual admitted default face with zero source/glyph extent, for empty metrics.
    pub(crate) empty_run: Option<PreparedRun>,
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
    pub(crate) run: usize,
    pub(crate) style: usize,
    pub(crate) level: u8,
    pub(crate) source: Range,
    pub(crate) glyphs: IndexRange<usize>,
    pub(crate) x: f64,
    pub(crate) advance: f64,
}

/// One actual logical glyph position, separate from its canonical provider offsets.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FinalizedGlyph {
    pub(crate) glyph: GlyphData,
    /// Run-local logical pen. This includes actual finalized cluster spacing.
    pub(crate) pen: f64,
}

#[derive(Clone, Debug)]
pub(crate) struct FinalizedCluster {
    pub(crate) source: Range,
    pub(crate) presentation: Range,
    pub(crate) run: usize,
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
    pub(crate) geometry: FinalizedBoxGeometry,
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
    InlineBox(usize),
    Anchor(usize),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FinalizedAnchor {
    pub(crate) unit: usize,
    pub(crate) slot: crate::Point,
    pub(crate) level: u8,
}

/// The actual allocation target joined by finalization, in selected-slot order.
/// Logical advance remains on the real finalized cluster, without a copy here.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FinalizedSpacingSlot {
    pub(crate) cluster: usize,
    pub(crate) disposition: crate::protocol::SpacingSlotDisposition,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FinalizedLineData {
    pub(crate) font_runs: Vec<PreparedRun>,
    pub(crate) runs: Vec<FinalizedRun>,
    /// Logical cluster order; each row records its actual visual position.
    pub(crate) clusters: Vec<FinalizedCluster>,
    /// Paint-order canonical local glyphs, with original associations.
    pub(crate) glyphs: Vec<FinalizedGlyph>,
    pub(crate) boxes: Vec<FinalizedBox>,
    pub(crate) visual_order: Vec<FinalizedVisualItem>,
    pub(crate) anchors: Vec<FinalizedAnchor>,
    pub(crate) spacing_slots: Vec<FinalizedSpacingSlot>,
    pub(crate) metrics: LineMetrics,
    pub(crate) fragments: Vec<FinalizedFragmentData>,
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
    pub(crate) bounds: Rect,
    pub(crate) coverage: crate::protocol::placement::PlacedCoverage,
    pub(crate) width: f64,
    pub(crate) full_width: f64,
    pub(crate) height: f64,
    pub(crate) interaction: super::super::editing::InteractionData,
}
